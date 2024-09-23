use std::path::PathBuf;

use dfrs_core::compile::compile;
use dfrs_core::definitions::action_dump::ActionDump;
use dfrs_core::definitions::actions::{EntityActions, GameActions, PlayerActions, VariableActions, ControlActions, SelectActions};
use dfrs_core::lexer::{Lexer, LexerError};
use dfrs_core::load_config;
use dfrs_core::parser::{ParseError, Parser};
use dfrs_core::token::{Keyword, Token};
use dfrs_core::validate::{ValidateError, Validator};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use dfrs_core::definitions::events::{EntityEvents, PlayerEvents};

#[derive(Debug)]
struct Backend {
    client: Client,

    player_events: PlayerEvents,
    entity_events: EntityEvents,

    player_actions: PlayerActions,
    entity_actions: EntityActions,
    game_actions: GameActions,
    variable_actions: VariableActions,
    control_actions: ControlActions,
    select_actions: SelectActions
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

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) ->  tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let path = params.text_document_position.text_document.uri.to_file_path().unwrap().to_str().unwrap().to_string();
        let data = std::fs::read_to_string(path).expect("could not open file");
        let line = params.text_document_position.position.line + 1;
        let col = params.text_document_position.position.character;

        self.client.log_message(MessageType::INFO, format!("{} {}", line, col)).await;

        let mut lexer = Lexer::new(data);
        let tokens = match lexer.run() {
            Ok(res) => res,
            Err(_) => return Ok(None)
        };

        let mut last_token: Option<dfrs_core::token::TokenWithPos> = None;
        for token in tokens {
            if token.start_pos.line == line && token.start_pos.col <= col && token.end_pos.col >= col {
                let mut is_event = false;
                let mut is_player_action = false;
                let mut is_entity_action = false;
                let mut is_game_action = false;
                let mut is_variable_action = false;
                let mut is_control_action = false;
                let mut is_select_action = false;

                let mut previous = String::from("");
                match &token.token {
                    Token::At => {
                        is_event = true
                    }
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
                if last_token.is_some() {
                    match last_token.unwrap().token {
                        Token::At => {
                            is_event = true;
                            match token.token.clone() {
                                Token::Identifier { value } => {
                                    previous += &value;
                                }
                                _ => {}
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
                    all = Some(self.player_actions.all());
                }
                if is_entity_action {
                    all = Some(self.entity_actions.all());
                }
                if is_game_action {
                    all = Some(self.game_actions.all());
                }
                if is_variable_action {
                    all = Some(self.variable_actions.all());
                }
                if is_control_action {
                    all = Some(self.control_actions.all());
                }
                if is_select_action {
                    all = Some(self.select_actions.all());
                }

                if all.is_some() {
                    let mut actions = vec![];

                    for action in all.unwrap() {
                        if action.dfrs_name.starts_with(&previous) || action.df_name.starts_with(&previous) {
                            actions.push(CompletionItem::new_simple(action.dfrs_name.clone(), action.df_name.clone()));
                        }
                    }
                    return Ok(Some(CompletionResponse::Array(actions)))
                }
            }
            last_token = Some(token);
        }

        Ok(None)
    }

    async fn diagnostic(&self, params: DocumentDiagnosticParams) -> tower_lsp::jsonrpc::Result<DocumentDiagnosticReportResult> {
        let mut result: Vec<Diagnostic> = vec![];

        let path = params.text_document.uri.to_file_path().unwrap().to_str().unwrap().to_string();
        let data = std::fs::read_to_string(path.clone()).expect("could not open file");

        match compile_file(data, path.into()) {
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

#[tokio::main]
pub async fn run_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let ad = ActionDump::load();
    let (service, socket) = LspService::new(|client| Backend {
        client,

        player_events: PlayerEvents::new(&ad),
        entity_events: EntityEvents::new(&ad),

        player_actions: PlayerActions::new(&ad),
        entity_actions: EntityActions::new(&ad), 
        game_actions: GameActions::new(&ad),
        variable_actions: VariableActions::new(&ad),
        control_actions: ControlActions::new(&ad),
        select_actions: SelectActions::new(&ad)
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

struct CompileErr {
    pub pos: dfrs_core::token::Position,
    pub end_pos: Option<dfrs_core::token::Position>,
    pub msg: String
}

impl CompileErr {
    pub fn new(pos: dfrs_core::token::Position, end_pos: Option<dfrs_core::token::Position>, msg: String) -> CompileErr {
        CompileErr { pos, end_pos, msg }
    }
}

fn compile_file(data: String, path: PathBuf) -> Result<(), CompileErr> {
    let mut config_path = path.clone();
    config_path.set_file_name("dfrs.toml");
    let config = match load_config(&config_path) {
        Ok(res) => res,
        Err(_) => return Err(CompileErr::new(dfrs_core::token::Position::new(0, 0), None, "No config file found".into()))
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