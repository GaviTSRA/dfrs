use dfrs_core::compile::compile;
use dfrs_core::lexer::{Lexer, LexerError};
use dfrs_core::load_config;
use dfrs_core::parser::{ParseError, Parser};
use dfrs_core::token::Token;
use dfrs_core::validate::{validate, ValidateError, PLAYER_EVENTS};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
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

        let mut last_token: Option<dfrs_core::token::TokenWithPos>= None;
        for token in tokens {
            if token.start_pos.line == line && token.start_pos.col <= col && token.end_pos.col >= col {
                let mut is_event = false;
                let mut previous = String::from("");
                match token.token {
                    Token::At => {
                        is_event = true
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

                    for (event, df_event) in PLAYER_EVENTS.entries() {
                        if event.starts_with(&previous) || df_event.starts_with(&previous) {
                            events.push(CompletionItem::new_simple(event.to_string(), df_event.to_string()));
                        }
                    }

                    return Ok(Some(CompletionResponse::Array(events)))
                }
            }
            last_token = Some(token);
        }

        Ok(None)
    }

    async fn diagnostic(&self, params: DocumentDiagnosticParams) -> tower_lsp::jsonrpc::Result<DocumentDiagnosticReportResult> {
        let mut result: Vec<Diagnostic> = vec![];

        let path = params.text_document.uri.to_file_path().unwrap().to_str().unwrap().to_string();
        let data = std::fs::read_to_string(path).expect("could not open file");

        match compile_file(data) {
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

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
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

fn compile_file(data: String) -> Result<(), CompileErr> {
    let config = load_config();

    let mut lexer = Lexer::new(data.clone());
    let result = lexer.run();

    let res = match result {
        Ok(res) => res,
        Err(err) => {
            match err {
                LexerError::InvalidNumber { pos } => {
                    return Err(CompileErr::new(pos, None, "Invalid number".to_owned()))
                    // print_err(format!("Invalid number in line {pos}"), data, pos, None);
                }
                LexerError::InvalidToken { token, pos } => {
                    // print_err(format!("Invalid token '{token}' in line {pos}"), data, pos, None);
                    return Err(CompileErr::new(pos, None, format!("Invalid token '{token}'")))
                }
                LexerError::UnterminatedString { pos } => {
                    // print_err(format!("Unterminated string in line {pos}"), data, pos, None);
                    return Err(CompileErr::new(pos, None, "Unterminated string".to_owned()))
                }
                LexerError::UnterminatedText { pos } => {
                    // print_err(format!("Unterminated text in line {pos}"), data, pos, None);
                    return Err(CompileErr::new(pos, None, "Unterminated text".to_owned()))
                }
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
                        // print_err(format!("Invalid token '{}', expected: {expected_string}", found.token), data, found.start_pos, Some(found.end_pos));
                    } else {
                        // println!("Invalid EOF, expected: {expected:?}");
                    }
                }
                ParseError::InvalidLocation { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid location '{msg}'")))
                },
                ParseError::InvalidVector { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid vecttor '{msg}'")))
                },
                ParseError::InvalidSound { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid sound '{msg}'")))
                },
                ParseError::InvalidPotion { pos, msg } => {
                    return Err(CompileErr::new(pos, None, format!("Invalid potion '{msg}'")))
                },
            }
            return Ok(())
        }
    }

    let validated;
    match validate(node) {
        Ok(res) => validated = res,
        Err(err)  => {
            match err {
                ValidateError::UnknownEvent { node } => {
                    let mut end_pos = node.start_pos.clone();
                    end_pos.col += 1 + node.event.chars().count() as u32;
                    return Err(CompileErr::new(node.start_pos, Some(node.end_pos), format!("Unknown event '{}'", node.event)))
                    // print_err(format!("Unknown event '{}'", node.event), data, node.start_pos, Some(end_pos));
                }
                ValidateError::UnknownAction { node } => {
                    let mut end_pos = node.start_pos.clone();
                    end_pos.col += 1 + node.name.chars().count() as u32;
                    return Err(CompileErr::new(node.start_pos, Some(node.end_pos), format!("Unknown action '{}'", node.name)))
                },
            }
        }
    }

    let compiled = compile(validated, config.debug.compile);

    Ok(())
}