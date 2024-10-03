use std::path::PathBuf;

use dashmap::DashMap;
use crate::compile::compile;
use crate::definitions::action_dump::{ActionDump, RawActionDump};
use crate::definitions::game_values::GameValues;
use crate::lexer::{Lexer, LexerError};
use crate::load_config;
use crate::parser::{ParseError, Parser};
use crate::token::{Keyword, Token};
use crate::validate::{ValidateError, Validator};
use ropey::Rope;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use crate::definitions::events::{EntityEvents, PlayerEvents};

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,

    player_events: PlayerEvents,
    entity_events: EntityEvents,

    action_dump: ActionDump,

    game_values: GameValues
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions { 
                    identifier: Some("dfrs-lsp".to_owned()),
                    inter_file_dependencies: false, 
                    workspace_diagnostics: false, 
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None
                    } 
                })),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(params.text_document)
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
            language_id: "dfrs".into()
        })
        .await
    }

    async fn completion(&self, params: CompletionParams) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let line = params.text_document_position.position.line + 1;
        let col = params.text_document_position.position.character;
        self.get_completions(uri, line, col).await
    }

    async fn diagnostic(&self, params: DocumentDiagnosticParams) -> tower_lsp::jsonrpc::Result<DocumentDiagnosticReportResult> {
        let mut result: Vec<Diagnostic> = vec![];

        let uri = params.text_document.uri.clone();
        let rope = self.document_map.get(&uri.to_string()).unwrap();
        let path = params.text_document.uri.to_file_path().unwrap();

        match compile_file(rope.to_string(), path) {
            Ok(_) => {},
            Err(err) => {
                let mut end_pos = err.pos.clone();
                if err.end_pos.is_some() {
                    end_pos = err.end_pos.unwrap();
                }
                result.push(Diagnostic {
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: err.msg,
                    range: Range {
                        start: Position { line: err.pos.line - 1, character: err.pos.col - 1 },
                        end: Position { line: end_pos.line - 1, character: end_pos.col - 1 }
                    },
                    ..Default::default()
                });
            }
        }

        Ok(DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                result_id: None,
                items: result
            }
        })))
    }
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        let rope = Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }

    async fn get_completions(&self, uri: String, line: u32, col: u32) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let rope = self.document_map.get(&uri).unwrap();

        self.client.log_message(MessageType::INFO, format!("{} {}", line, col)).await;

        let mut lexer = Lexer::new(rope.to_string());
        let tokens = match lexer.run() {
            Ok(res) => res,
            Err(_) => return Ok(None)
        };

        let mut last_token: Option<crate::token::TokenWithPos> = None;
        for token in tokens {
            if token.start_pos.line == line && token.start_pos.col <= col && token.end_pos.col >= col {
                let mut is_event = false;
                let mut is_player_action = false;
                let mut is_entity_action = false;
                let mut is_game_action = false;
                let mut is_variable_action = false;
                let mut is_control_action = false;
                let mut is_select_action = false;
                let mut is_player_conditional = false;
                let mut is_entity_conditional = false;
                let mut is_game_conditional = false;
                let mut is_variable_conditional = false;
                let mut is_game_value = false;

                let mut previous = String::from("");
                match &token.token {
                    Token::At => is_event = true,
                    Token::Dollar => is_game_value = true,
                    Token::Dot => {
                        match last_token.clone() {
                            Some(last) => {
                                match last.token {
                                    Token::Keyword { value } => {
                                        match value {
                                            Keyword::P => is_player_action = true,
                                            Keyword::E => is_entity_action = true,
                                            Keyword::G => is_game_action = true,
                                            Keyword::V => is_variable_action = true,
                                            Keyword::C => is_control_action = true,
                                            Keyword::S => is_select_action = true,
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            None => {}
                        }
                    }
                    _ => {}
                }
                if last_token.is_some() {
                    match last_token.unwrap().token {
                        Token::At => {
                            is_event = true;
                            match token.token.clone() {
                                Token::Identifier { value } => previous += &value,
                                _ => {}
                            }
                        }
                        Token::Dollar => {
                            is_game_value = true;
                            match token.token.clone() {
                                Token::Identifier { value } => previous += &value,
                                _ => {}
                            }
                        }
                        Token::Keyword { value } => {
                            let mut found = true;
                            match value {
                                Keyword::IfP => is_player_conditional = true,
                                Keyword::IfE => is_entity_conditional = true,
                                Keyword::IfG => is_game_conditional = true,
                                Keyword::IfV => is_variable_conditional = true,
                                _ => found = false
                            }
                            if found {
                                match token.token.clone() {
                                    Token::Identifier { value } => previous += &value,
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if is_event {
                    let mut events = vec![];

                    for event in self.player_events.all() {
                        if event.dfrs_name.starts_with(&previous) || event.df_name.starts_with(&previous) {
                            events.push(CompletionItem::new_simple(event.dfrs_name.clone(), event.df_name.clone()));
                        }
                    }
                    for event in self.entity_events.all() {
                        if event.dfrs_name.starts_with(&previous) || event.df_name.starts_with(&previous) {
                            events.push(CompletionItem::new_simple(event.dfrs_name.clone(), event.df_name.clone()));
                        }
                    }

                    return Ok(Some(CompletionResponse::Array(events)))
                }

                let mut all = None;
                if is_player_action {
                    all = Some(self.action_dump.player_actions.all());
                }
                if is_entity_action {
                    all = Some(self.action_dump.entity_actions.all());
                }
                if is_game_action {
                    all = Some(self.action_dump.game_actions.all());
                }
                if is_variable_action {
                    all = Some(self.action_dump.variable_actions.all());
                }
                if is_control_action {
                    all = Some(self.action_dump.control_actions.all());
                }
                if is_select_action {
                    all = Some(self.action_dump.select_actions.all());
                }
                if is_player_conditional {
                    all = Some(self.action_dump.player_conditionals.all());
                }
                if is_entity_conditional {
                    all = Some(self.action_dump.entity_conditionals.all());
                }
                if is_game_conditional {
                    all = Some(self.action_dump.game_conditionals.all());
                }
                if is_variable_conditional {
                    all = Some(self.action_dump.variable_conditionals.all());
                }

                self.client.log_message(MessageType::INFO, format!("ev {} pa {} ea {} ga {} va {} pc {} ec {} gc {} vc {} vl {}", is_event, is_player_action, is_entity_action, is_game_action, is_variable_action, is_player_conditional, is_entity_conditional, is_game_conditional, is_variable_conditional, is_game_value)).await;

                if all.is_some() {
                    let mut actions = vec![];

                    for action in all.unwrap() {
                        if action.dfrs_name.starts_with(&previous) || action.df_name.starts_with(&previous) {
                            actions.push(CompletionItem::new_simple(action.dfrs_name.clone(), action.df_name.clone()));
                        }
                    }
                    return Ok(Some(CompletionResponse::Array(actions)))
                }

                if is_game_value {
                    let game_values = self.game_values.all();
                    let mut result = vec![];

                    for game_value in game_values {
                        if game_value.dfrs_name.starts_with(&previous) || game_value.df_name.starts_with(&previous) {
                            result.push(CompletionItem::new_simple(game_value.dfrs_name.clone(), game_value.df_name.clone()));
                        }
                    }
                    return Ok(Some(CompletionResponse::Array(result)))
                }
            }
            last_token = Some(token);
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

        action_dump: ActionDump::new(&ad),

        game_values: GameValues::new(&ad)
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

struct CompileErr {
    pub pos: crate::token::Position,
    pub end_pos: Option<crate::token::Position>,
    pub msg: String
}

impl CompileErr {
    pub fn new(pos: crate::token::Position, end_pos: Option<crate::token::Position>, msg: String) -> CompileErr {
        CompileErr { pos, end_pos, msg }
    }
}

fn compile_file(data: String, path: PathBuf) -> Result<(), CompileErr> {
    let mut config_path = path.clone();
    config_path.set_file_name("dfrs.toml");
    let config = match load_config(&config_path) {
        Ok(res) => res,
        Err(_) => return Err(CompileErr::new(crate::token::Position::new(0, 0), None, "No config file found".into()))
    };

    let mut lexer = Lexer::new(data.clone());
    let result = lexer.run();

    let res = match result {
        Ok(res) => res,
        Err(err) => {
            return match err {
                LexerError::InvalidNumber { pos } => {
                    Err(CompileErr::new(pos, None, "Invalid number".to_owned()))
                }
                LexerError::InvalidToken { token, pos } => {
                    Err(CompileErr::new(pos, None, format!("Invalid token '{token}'")))
                }
                LexerError::UnterminatedString { pos } => {
                    Err(CompileErr::new(pos, None, "Unterminated string".to_owned()))
                }
                LexerError::UnterminatedText { pos } => {
                    Err(CompileErr::new(pos, None, "Unterminated text".to_owned()))
                }
                LexerError::UnterminatedVariable { pos } => {
                    Err(CompileErr::new(pos, None, "Unterminated variable".to_owned()))
                },
            }
        }
    };

    let mut parser = Parser::new(res);
    let res = parser.run();
    let node;
    match res {
        Ok(res) =>node = res,
        Err(err) => {
            match err {
                ParseError::InvalidToken { found,expected} => {
                    if found.is_some() {
                        let found = found.unwrap();

                        let mut i = 0;
                        let mut expected_string = "".to_owned();
                        for token in expected.clone() {
                            expected_string.push_str(&format!("'{token}'"));
                            if i < expected.len() - 1 {
                                expected_string.push_str(", ");
                            }
                            i += 1;
                        }

                        return Err(CompileErr::new(found.start_pos, Some(found.end_pos), format!("Invalid token '{}', expected: {expected_string}", found.token)))
                    } else {
                        // println!("Invalid EOF, expected: {expected:?}");
                    }
                }
                ParseError::InvalidComplexNumber { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid number '{msg}'")))
                },
                ParseError::InvalidLocation { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid location '{msg}'")))
                },
                ParseError::InvalidVector { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid vector '{msg}'")))
                },
                ParseError::InvalidSound { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid sound '{msg}'")))
                },
                ParseError::InvalidPotion { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid potion '{msg}'")))
                },
                ParseError::InvalidItem { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid item '{msg}'")))
                },
                ParseError::UnknownVariable { found, start_pos, end_pos } => {
                    return Err(CompileErr::new(start_pos, Some(end_pos), format!("Unknown variable '{}'", found)))
                },
                ParseError::InvalidType { found, start_pos } => {
                    return match found {
                        Some(found) => Err(CompileErr::new(found.start_pos, Some(found.end_pos), format!("Unknown type: {}", found.token))),
                        None => Err(CompileErr::new(start_pos, None, "Missing type".into()))
                    }
                },
                ParseError::InvalidCall { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid function call '{msg}'")))
                },
            }
            return Ok(())
        }
    }

    let validated;
    match Validator::new().validate(node) {
        Ok(res) => validated = res,
        Err(err)  => {
            return match err {
                ValidateError::UnknownEvent { node } => {
                    Err(CompileErr::new(node.start_pos, Some(node.end_pos), format!("Unknown event '{}'", node.event)))
                }
                ValidateError::UnknownAction { name, start_pos, end_pos } => {
                    Err(CompileErr::new(start_pos, Some(end_pos), format!("Unknown action '{}'", name)))
                },
                ValidateError::MissingArgument { start_pos, end_pos, name } => {
                    Err(CompileErr::new(start_pos, Some(end_pos), format!("Missing argument '{}'", name)))
                }
                ValidateError::WrongArgumentType { args, index, name, expected_types, found_type } => {
                    Err(CompileErr::new(args.get(index as usize).unwrap().start_pos.clone(), Some(args.get(index as usize).unwrap().end_pos.clone()), format!("Wrong argument type for '{}', expected '{:?}' but found '{:?}'", name, expected_types, found_type)))
                }
                ValidateError::TooManyArguments { start_pos, mut end_pos, name } => {
                    end_pos.col += name.chars().count() as u32;
                    Err(CompileErr::new(start_pos.clone(), Some(start_pos), format!("Too many arguments for action '{}'", name)))
                }
                ValidateError::InvalidTagOption { tag_name, provided, options, start_pos, end_pos } => {
                    Err(CompileErr::new(start_pos, Some(end_pos), format!("Invalid option '{}' for tag '{}', expected one of {:?}", provided, tag_name, options)))
                }
                ValidateError::UnknownTag { tag_name, available, start_pos, end_pos } => {
                    Err(CompileErr::new(start_pos, Some(end_pos), format!("Unknown tag '{}', found tags: {:?}", tag_name, available)))
                }
                ValidateError::UnknownGameValue { game_value, start_pos, end_pos} => {
                    Err(CompileErr::new(start_pos, Some(end_pos), format!("Unknown game value '{}'", game_value)))
                }
            }
        }
    }

    let compiled = compile(validated, config.debug.compile);

    Ok(())
}