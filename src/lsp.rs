use std::path::PathBuf;
use std::vec;

use crate::compile::compile;
use crate::definitions::action_dump::{ActionDump, RawActionDump};
use crate::definitions::actions::Action;
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
    let line = params.text_document_position_params.position.line;
    let col = params.text_document_position_params.position.character;

    let line_data = rope.line(line as usize).to_string();
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
    let line = params.text_document_position.position.line;
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
    let input = rope.line(line as usize).to_string();

    let mut lexer = Lexer::new(&input);
    let tokens = match lexer.run() {
      Ok(res) => res,
      Err(_) => return Ok(None),
    };

    for (index, token) in tokens.iter().enumerate() {
      if token.range.start.col <= col && token.range.end.col >= col {
        let mut token = &tokens[index].token;
        let mut previous: Option<String> = None;

        match token {
          Token::Identifier { value } => previous = Some(value.clone()),
          Token::Text { value } => {
            previous = Some(value.clone());

            if index >= 2 {
              let token_before_1 = &tokens[index - 1].token;
              let token_before_2 = &tokens[index - 2].token;
              if let (Token::Identifier { value: identifier }, Token::OpenParen) =
                (token_before_2, token_before_1)
              {
                let mut completion = vec![];
                match identifier.as_str() {
                  "Potion" => {
                    for potion in self.action_dump.potions.all() {
                      if potion.potion.starts_with(value) {
                        completion.push(CompletionItem::new_simple(
                          potion.potion.clone(),
                          String::from("Potion"),
                        ));
                      }
                    }
                    return Ok(Some(CompletionResponse::Array(completion)));
                  }
                  "Sound" => {
                    for potion in self.action_dump.sounds.all() {
                      if potion.sound.starts_with(value) {
                        completion.push(CompletionItem::new_simple(
                          potion.sound.clone(),
                          String::from("Sound"),
                        ));
                      }
                    }
                    return Ok(Some(CompletionResponse::Array(completion)));
                  }
                  "Particle" => {
                    for potion in self.action_dump.particles.all() {
                      if potion.particle.starts_with(value) {
                        completion.push(CompletionItem::new_simple(
                          potion.particle.clone(),
                          String::from("Particle"),
                        ));
                      }
                    }
                    return Ok(Some(CompletionResponse::Array(completion)));
                  }
                  _ => {}
                }
              }
            }
          }
          Token::Keyword { value } => previous = Some(format!("{}", value.clone())),
          _ => {}
        }

        let mut token_before_2: Option<&Token> = None;
        let mut token_before_3: Option<&Token> = None;
        let mut token_before_4: Option<&Token> = None;

        if previous.is_some() && index >= 1 {
          token = &tokens[index - 1].token;

          if index >= 2 {
            token_before_2 = Some(&tokens[index - 2].token);
          }
          if index >= 3 {
            token_before_3 = Some(&tokens[index - 3].token);
          }
          if index >= 4 {
            token_before_4 = Some(&tokens[index - 4].token);
          }
        } else {
          if index >= 1 {
            token_before_2 = Some(&tokens[index - 1].token);
          }
          if index >= 2 {
            token_before_3 = Some(&tokens[index - 2].token);
          }
          if index >= 3 {
            token_before_4 = Some(&tokens[index - 3].token);
          }

          previous = Some(String::from(""));
        }

        let previous = previous.unwrap();
        let mut actions: Option<&Vec<Action>> = None;

        match (token_before_4, token_before_3, token_before_2, token) {
          // Events
          (_, _, _, Token::At) => {
            let mut events = vec![];

            for event in self.player_events.all() {
              if event.dfrs_name.starts_with(&previous) || event.df_name.starts_with(&previous) {
                events.push(CompletionItem::new_simple(
                  event.dfrs_name.clone(),
                  String::from("Player Event"),
                ));
              }
            }
            for event in self.entity_events.all() {
              if event.dfrs_name.starts_with(&previous) || event.df_name.starts_with(&previous) {
                events.push(CompletionItem::new_simple(
                  event.dfrs_name.clone(),
                  String::from("Entity Event"),
                ));
              }
            }
            for event in self.game_events.all() {
              if event.dfrs_name.starts_with(&previous) || event.df_name.starts_with(&previous) {
                events.push(CompletionItem::new_simple(
                  event.dfrs_name.clone(),
                  String::from("Game Event"),
                ));
              }
            }

            return Ok(Some(CompletionResponse::Array(events)));
          }

          // Actions
          (_, _, Some(Token::Keyword { value: keyword }), Token::Dot)
          | (
            Some(Token::Keyword { value: keyword }),
            Some(Token::Colon),
            Some(Token::Identifier { value: _ }),
            Token::Dot,
          ) => match keyword {
            Keyword::P => actions = Some(self.action_dump.player_actions.all()),
            Keyword::E => actions = Some(self.action_dump.entity_actions.all()),
            Keyword::G => actions = Some(self.action_dump.game_actions.all()),
            Keyword::V => actions = Some(self.action_dump.variable_actions.all()),
            Keyword::C => actions = Some(self.action_dump.control_actions.all()),
            Keyword::S => actions = Some(self.action_dump.select_actions.all()),
            _ => {}
          },

          // Conditionals, Repeat
          (_, _, _, Token::Keyword { value: keyword })
          | (_, _, Some(Token::Keyword { value: keyword }), Token::ExclamationMark) => {
            match keyword {
              Keyword::IfP => actions = Some(self.action_dump.player_conditionals.all()),
              Keyword::IfE => actions = Some(self.action_dump.entity_conditionals.all()),
              Keyword::IfG => actions = Some(self.action_dump.game_conditionals.all()),
              Keyword::IfV => actions = Some(self.action_dump.variable_conditionals.all()),
              Keyword::Repeat => actions = Some(self.action_dump.repeats.all()),
              _ => {}
            }
          }

          // Game Values
          (_, _, _, Token::Dollar) => {
            let mut result = vec![];

            for game_value in self.game_values.all() {
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
          _ => return Ok(None),
        }

        if let Some(actions) = actions {
          let mut result = vec![];

          for action in actions {
            if action.dfrs_name.starts_with(&previous) || action.df_name.starts_with(&previous) {
              result.push(CompletionItem::new_simple(
                action.dfrs_name.clone(),
                action.df_name.clone(),
              ));
            }
          }
          return Ok(Some(CompletionResponse::Array(result)));
        }
      }
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
