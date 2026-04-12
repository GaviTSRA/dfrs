use std::path::PathBuf;
use std::{vec};

use crate::compile::compile;
use crate::definitions::action_dump::{ActionDump, RawActionDump};
use crate::definitions::events::{EntityEvents, GameEvents, PlayerEvents};
use crate::definitions::game_values::GameValues;
use crate::errors::{format_lexer_error, format_parser_error, format_validator_error};
use crate::lexer::Lexer;
use crate::load_config;
use crate::parser::Parser;
use crate::token::{Keyword, Token};
use crate::validate::Validator;
use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
  client: Client,
  document_map: DashMap<String, Rope>,

  player_events: PlayerEvents,
  entity_events: EntityEvents,
  game_events: GameEvents,

  action_dump: ActionDump,

  game_values: GameValues,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, _: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
    Ok(InitializeResult {
      server_info: None,
      capabilities: ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
          resolve_provider: Some(false),
          trigger_characters: Some(vec![".".into(), "$".into()]),
          work_done_progress_options: Default::default(),
          all_commit_characters: None,
          ..Default::default()
        }),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
          identifier: Some("dfrs-lsp".to_owned()),
          inter_file_dependencies: true,
          workspace_diagnostics: false,
          work_done_progress_options: WorkDoneProgressOptions {
            work_done_progress: None,
          },
        })),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        ..ServerCapabilities::default()
      },
      ..Default::default()
    })
  }

  async fn initialized(&self, _: InitializedParams) {
    self
      .client
      .log_message(MessageType::INFO, "server initialized!")
      .await;
  }

  async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
    Ok(())
  }

  async fn did_open(&self, params: DidOpenTextDocumentParams) {
    self
      .client
      .log_message(MessageType::INFO, "file opened!")
      .await;
    self.on_change(params.text_document).await
  }

  async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
    self
      .on_change(TextDocumentItem {
        uri: params.text_document.uri,
        text: std::mem::take(&mut params.content_changes[0].text),
        version: params.text_document.version,
        language_id: "dfrs".into(),
      })
      .await
  }

  async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
    let uri = params
      .text_document_position_params
      .text_document
      .uri
      .clone();
    let rope = self.document_map.get(&uri.to_string()).unwrap();
    let data = rope.to_string();
    let line = params.text_document_position_params.position.line;
    let col = params.text_document_position_params.position.character;

    let mut lines = data.lines();
    let line_data = lines.nth(line as usize).unwrap();
    let mut lexer = Lexer::new(&line_data);
    let result = lexer.run();

    let res = match result {
      Ok(res) => res,
      Err(error) => {
        return Ok(None);
      }
    };

    for (index, token) in res.iter().enumerate() {
      if token.range.start.col <= col && token.range.end.col >= col && index >= 1 {
        let token_before_2 = if index >= 2 {
          Some(&res[index - 2].token)
        } else {
          None
        };

        let token_before_3 = if index >= 3 {
          Some(&res[index - 3].token)
        } else {
          None
        };

        let token_before_4 = if index >= 4 {
          Some(&res[index - 4].token)
        } else {
          None
        };

        let action = match &token.token {
          Token::Identifier { value } => {
            match (
              token_before_4,
              token_before_3,
              token_before_2,
              &res[index - 1].token,
            ) {
              (_, _, Some(Token::Keyword { value: keyword }), Token::Dot)
              | (
                Some(Token::Keyword { value: keyword }),
                Some(Token::Colon),
                Some(Token::Identifier { value: _ }),
                Token::Dot,
              ) => match keyword {
                Keyword::P => self.action_dump.player_actions.get(&value),
                Keyword::E => self.action_dump.entity_actions.get(&value),
                Keyword::G => self.action_dump.game_actions.get(&value),
                Keyword::V => self.action_dump.variable_actions.get(&value),
                Keyword::C => self.action_dump.control_actions.get(&value),
                Keyword::S => self.action_dump.select_actions.get(&value),
                _ => return Ok(None),
              },
              (_, _, _, Token::Keyword { value: keyword })
              | (_, _, Some(Token::Keyword { value: keyword }), Token::ExclamationMark) => {
                match keyword {
                  Keyword::IfP => self.action_dump.player_conditionals.get(&value),
                  Keyword::IfE => self.action_dump.entity_conditionals.get(&value),
                  Keyword::IfG => self.action_dump.game_conditionals.get(&value),
                  Keyword::IfV => self.action_dump.variable_conditionals.get(&value),
                  Keyword::Repeat => self.action_dump.repeats.get(&value),
                  _ => return Ok(None),
                }
              }
              _ => return Ok(None),
            }
          }
          _ => return Ok(None),
        };

        if let Some(action) = action {
          return Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
              kind: MarkupKind::Markdown,
              value: action.description.clone(),
            }),
            range: None,
          }));
        }
      }
    }

    Ok(None)
  }

  async fn completion(
    &self,
    params: CompletionParams,
  ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri.to_string();
    let line = params.text_document_position.position.line + 1;
    let col = params.text_document_position.position.character;
    self.get_completions(uri, line, col).await
  }

  async fn diagnostic(
    &self,
    params: DocumentDiagnosticParams,
  ) -> tower_lsp::jsonrpc::Result<DocumentDiagnosticReportResult> {
    let mut result: Vec<Diagnostic> = vec![];

    let uri = params.text_document.uri.clone();
    let rope = self.document_map.get(&uri.to_string()).unwrap();
    let path = params.text_document.uri.to_file_path().unwrap();

    match compile_file(rope.to_string(), path) {
      Ok(_) => {}
      Err(err) => {
        let mut end_pos = err.pos.clone();
        if err.end_pos.is_some() {
          end_pos = err.end_pos.unwrap();
        }
        result.push(Diagnostic {
          severity: Some(DiagnosticSeverity::ERROR),
          message: err.msg,
          range: Range {
            start: Position {
              line: err.pos.line - 1,
              character: err.pos.col - 1,
            },
            end: Position {
              line: end_pos.line - 1,
              character: end_pos.col - 1,
            },
          },
          ..Default::default()
        });
      }
    }

    Ok(DocumentDiagnosticReportResult::Report(
      DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
        related_documents: None,
        full_document_diagnostic_report: FullDocumentDiagnosticReport {
          result_id: None,
          items: result,
        },
      }),
    ))
  }
}

impl Backend {
  async fn on_change(&self, params: TextDocumentItem) {
    let rope = Rope::from_str(&params.text);
    self
      .document_map
      .insert(params.uri.to_string(), rope.clone());
  }

  async fn get_completions(
    &self,
    uri: String,
    line: u32,
    col: u32,
  ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
    let rope = self.document_map.get(&uri).unwrap();
    let line_string =rope.line((line-1).try_into().unwrap());//gets the current line (-1 from 0 index)

    self
      .client
      .log_message(MessageType::INFO, format!("{} {}", line, col))
      .await;

    //let input = &rope.to_string();
    let input = &line_string.to_string();
    let mut lexer = Lexer::new(input);
    let mut tokens = match lexer.run() {
      Ok(res) => res,
      Err(_) => return Ok(None),
    };

    
    let mut focused_token_index=1;
    for token in &tokens {

      let mut tokencolend=token.range.end.col;//due to words taking up one extra space in range this counters that
      match &token.token {
        Token::Keyword { value }=>tokencolend-=1,
        _=>{}
      }

      if token.range.start.col <= col
        && tokencolend >= col
      {
        break;
      }
      focused_token_index=focused_token_index+1;
    }
    if focused_token_index>tokens.len()//if the token isnt found
    {
      return Ok(None);
    }

    _=tokens.split_off(focused_token_index);//only has the preivious tokens and allows use of pop
    let mut token=tokens.pop().unwrap();
    self
      .client
      .log_message(MessageType::INFO, format!("{} {:?}", token.token, tokens))
      .await;

    let mut is_event = false;
    // let mut is_player_action = false;
    // let mut is_entity_action = false;
    // let mut is_game_action = false;
    // let mut is_variable_action = false;
    // let mut is_control_action = false;
    // let mut is_select_action = false;
    // let mut is_player_conditional = false;
    // let mut is_entity_conditional = false;
    // let mut is_game_conditional = false;
    // let mut is_variable_conditional = false;
    let mut is_game_value = false;

    let mut previous = String::from("");//if current is identifier set them equal
    let mut all = None;

    //if ident, sack it off and throw into prev
    match token.token.clone() {
      Token::Identifier { value } => {
        previous += &value;
        match tokens.pop(){
          Some(res)=>token=res,
          None=>{}
        }
      }
      Token::Text { value }=>// found in the form [Identifier]([text]
      {
        previous += &value;
        match tokens.pop(){
          Some(res)=>{
            token=res;
            match token.token.clone() {
              Token::OpenParen =>{
                match tokens.pop(){
                  Some(res)=>{
                    token=res;
                    let mut completion =vec![];
                    match token.token.clone() {
                      Token::Identifier { value } =>{
                        match value.as_str() {
                          "Potion"=>{ 
                            for potion in self.action_dump.potions.all() {
                              if potion.potion.starts_with(&previous) {
                                completion.push(CompletionItem::new_simple(
                                  potion.potion.clone(),String::from("dfrs.Potion")
                                ));
                              }
                            }
                            return Ok(Some(CompletionResponse::Array(completion)));
                          }
                          "Sound"=>{
                            for potion in self.action_dump.sounds.all() {
                              if potion.sound.starts_with(&previous) {
                                completion.push(CompletionItem::new_simple(
                                  potion.sound.clone(),String::from("dfrs.Sound")
                                ));
                              }
                            }
                            return Ok(Some(CompletionResponse::Array(completion)));
                          }
                          "Particle"=>{ 
                            for potion in self.action_dump.particles.all() {
                              if potion.particle.starts_with(&previous) {
                                completion.push(CompletionItem::new_simple(
                                  potion.particle.clone(),String::from("dfrs.Particle")
                                ));
                              }
                            }
                            return Ok(Some(CompletionResponse::Array(completion)));
                          }
                          _=>{}

                        }
                      }
                      _ =>{}
                    }
                  }
                  _=>{}
                }
              }
              _ =>{}
            }
          }
          None=>{}
        }
      
        //consumes this token
      }
      
      _ => {}
    }
    let mut repeat=true;
    //need to account for selection
    
    while repeat {
      repeat=false;
    
      match &token.token {
        Token::At => is_event = true,
        Token::Dollar => is_game_value = true,
        Token::Dot => match tokens.pop().clone() {
          Some(last) => match &last.token {
            Token::Keyword { value } => match value {
              Keyword::P => all = Some(self.action_dump.player_actions.all()),
              Keyword::E => all = Some(self.action_dump.entity_actions.all()),
              Keyword::G => all = Some(self.action_dump.game_actions.all()),
              Keyword::V => all = Some(self.action_dump.variable_actions.all()),
              Keyword::C => all = Some(self.action_dump.control_actions.all()),
              Keyword::S => all = Some(self.action_dump.select_actions.all()),
              _ => {}
            },
            _ => {}
          },
          _ => {}
        },
        Token::Keyword { value } => {
            match value {
              Keyword::IfP => {all = Some(self.action_dump.player_conditionals.all()) ;break;},
              Keyword::IfE => {all = Some(self.action_dump.entity_conditionals.all());break;},
              Keyword::IfG => {all = Some(self.action_dump.game_conditionals.all());break;},
              Keyword::IfV => {all = Some(self.action_dump.variable_conditionals.all());break;},
              Keyword::Repeat=> {all=Some(self.action_dump.repeats.all());break;},
              _ => {
                previous += &value.to_string();
                repeat=true;
              },//if action or something, treat token as identifier, then repeat
            }
          }
        _ => {}
      }
      if repeat
      {
        match tokens.pop(){
          Some(res)=>token=res,
          None=>{break;}
        }
      }
    }


    if is_event {
      let mut events = vec![];//would make sense to change the detail to be the event type 

      for event in self.player_events.all() {
        if event.dfrs_name.starts_with(&previous) || event.df_name.starts_with(&previous) {
          events.push(CompletionItem::new_simple(
            event.dfrs_name.clone(),
            event.df_name.clone(),
          ));
        }
      }
      for event in self.entity_events.all() {
        if event.dfrs_name.starts_with(&previous) || event.df_name.starts_with(&previous) {
          events.push(CompletionItem::new_simple(
            event.dfrs_name.clone(),
            event.df_name.clone(),
          ));
        }
      }
      for event in self.game_events.all() {
        if event.dfrs_name.starts_with(&previous) || event.df_name.starts_with(&previous) {
          events.push(CompletionItem::new_simple(
            event.dfrs_name.clone(),
            event.df_name.clone(),
          ));
        }
      }

      return Ok(Some(CompletionResponse::Array(events)));
    }

    if all.is_some() {
      let mut actions = vec![];

      for action in all.unwrap() {
        if action.dfrs_name.starts_with(&previous) || action.df_name.starts_with(&previous) {
          actions.push(CompletionItem::new_simple(
            action.dfrs_name.clone(),
            action.df_name.clone(),
          ));
        }
      }
      return Ok(Some(CompletionResponse::Array(actions)));
    }

    if is_game_value {
      let game_values = self.game_values.all();
      let mut result = vec![];

      for game_value in game_values {
        if game_value.dfrs_name.starts_with(&previous)
          || game_value.df_name.starts_with(&previous)
        {
          result.push(CompletionItem::new_simple(
            game_value.dfrs_name.clone(),
            game_value.df_name.clone(),
          ));
        }
      }
      return Ok(Some(CompletionResponse::Array(result)));
    }

    Ok(None)
  }
}

#[tokio::main]
pub async fn run_lsp() {
  let stdin = tokio::io::stdin();
  let stdout = tokio::io::stdout();

  let ad = RawActionDump::load();
  let (service, socket) = LspService::new(|client| Backend {
    client,
    document_map: DashMap::new(),

    player_events: PlayerEvents::new(&ad),
    entity_events: EntityEvents::new(&ad),
    game_events: GameEvents::new(&ad),

    action_dump: ActionDump::new(&ad),

    game_values: GameValues::new(&ad),
  });
  Server::new(stdin, stdout, socket).serve(service).await;
}

struct CompileErr {
  pub pos: crate::token::Position,
  pub end_pos: Option<crate::token::Position>,
  pub msg: String,
}

impl CompileErr {
  pub fn new(
    pos: crate::token::Position,
    end_pos: Option<crate::token::Position>,
    msg: String,
  ) -> CompileErr {
    CompileErr { pos, end_pos, msg }
  }
}

fn compile_file(data: String, path: PathBuf) -> Result<(), CompileErr> {
  let mut config_path = path.clone();
  config_path.set_file_name("dfrs.toml");
  let config = match load_config(&config_path) {
    Ok(res) => res,
    Err(_) => {
      return Err(CompileErr::new(
        crate::token::Position::new(0, 0),
        None,
        "No config file found".into(),
      ))
    }
  };

  let input = &data.clone();
  let mut lexer = Lexer::new(input);
  let result = lexer.run();

  let res = match result {
    Ok(res) => res,
    Err(error) => {
      let formatted = format_lexer_error(error);
      return Err(CompileErr::new(
        formatted.start,
        formatted.end,
        formatted.message,
      ));
    }
  };

  let mut parser = Parser::new(res);
  let res = parser.run();
  let node;
  match res {
    Ok(res) => node = res,
    Err(error) => {
      let formatted = format_parser_error(error);
      return Err(CompileErr::new(
        formatted.start,
        formatted.end,
        formatted.message,
      ));
    }
  }

  let validated;
  match Validator::new().validate(node) {
    Ok(res) => validated = res,
    Err(error) => {
      let formatted = format_validator_error(error);
      return Err(CompileErr::new(
        formatted.start,
        formatted.end,
        formatted.message,
      ));
    }
  }

  let compiled = compile(validated, config.debug.compile);

  Ok(())
}
