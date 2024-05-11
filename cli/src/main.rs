use std::cmp;

use dfrs_core::send::send;
use dfrs_core::token::{Position, SELECTORS};
use dfrs_core::compile::compile;
use dfrs_core::lexer::{Lexer, LexerError};
use dfrs_core::load_config;
use dfrs_core::parser::{ParseError, Parser};
use dfrs_core::validate::{Validator, ValidateError};

use colored::Colorize;

fn print_err(message: String, data: String, start_pos: Position, end_pos: Option<Position>) {
    let lines = data.split("\n").collect::<Vec<&str>>();
    let line = lines.get((start_pos.line - 1) as usize).unwrap();
    let ln = start_pos.line;
    let ln_length = ln.to_string().chars().count();

    println!("{} {}", "Error:".bright_red(), message);
    println!("{} {}", " ".repeat(ln_length), "|".bright_black());
    println!("{} {} {}", ln.to_string().bright_black(), "|".bright_black(), line);
    let arrows;
    match end_pos {
        Some(end_pos) => {
            if end_pos.line != start_pos.line {
                // TODO
            }
            arrows = "^".repeat(cmp::max(end_pos.col - start_pos.col, 1) as usize).bright_blue();
        }
        None => {
            arrows = "^".bright_blue();
        }
    }
    println!("{} {} {}{}", " ".repeat(ln_length), "|".bright_black(), " ".repeat((start_pos.col - 1) as usize), arrows);
}

fn main() {
    let config = load_config();

    let data = std::fs::read_to_string("test_project/test.df").expect("could not open file");

    let mut lexer = Lexer::new(data.clone());
    let result = lexer.run();

    let res = match result {
        Ok(res) => {
            if config.debug.tokens {
                for token in &res {
                    println!("{:?}", token);
                }
                println!("\n");
            }
            res
        }
        Err(err) => {
            match err {
                LexerError::InvalidNumber { pos } => {
                    print_err(format!("Invalid number in line {pos}"), data, pos, None);
                }
                LexerError::InvalidToken { token, pos } => {
                    print_err(format!("Invalid token '{token}' in line {pos}"), data, pos, None);
                }
                LexerError::UnterminatedString { pos } => {
                    print_err(format!("Unterminated string in line {pos}"), data, pos, None);
                }
                LexerError::UnterminatedText { pos } => {
                    print_err(format!("Unterminated text in line {pos}"), data, pos, None);
                }
            }
            std::process::exit(0);
        }
    };

    let mut parser = Parser::new(res);
    let res = parser.run();
    let node;
    match res {
        Ok(res) => {
            if config.debug.nodes {
                for event in &res.events {
                    println!("{}", event.event);
                    for expression in &event.expressions {
                        match &expression.node {
                            dfrs_core::node::Expression::Action { node } => {
                                println!("{:?} {:?} {:?} {:?}", node.action_type, node.selector, node.name, node.args)
                            }
                        }
                    }
                }
                println!("\n");
            }
            node = res;
        }
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

                        print_err(format!("Invalid token '{}', expected: {expected_string}", found.token), data, found.start_pos, Some(found.end_pos));
                    } else {
                        println!("Invalid EOF, expected: {expected:?}");
                    }
                }
                ParseError::InvalidLocation { pos, msg } => {
                    print_err(format!("Invalid Location: {}", msg), data, pos, None)
                }
                ParseError::InvalidVector { pos, msg } => {
                    print_err(format!("Invalid Vector: {}", msg), data, pos, None)
                }
                ParseError::InvalidSound { pos, msg } => {
                    print_err(format!("Invalid Sound: {}", msg), data, pos, None)
                }
                ParseError::InvalidPotion { pos, msg } => {
                    print_err(format!("Invalid Potion: {}", msg), data, pos, None)
                }
            }
            std::process::exit(0);
        }
    }

    let validated;
    match Validator::new().validate(node) {
        Ok(res) => validated = res,
        Err(err)  => {
            match err {
                ValidateError::UnknownEvent { node } => {
                    let mut end_pos = node.start_pos.clone();
                    end_pos.col += 1 + node.event.chars().count() as u32;
                    print_err(format!("Unknown event '{}'", node.event), data, node.start_pos, Some(end_pos));
                }
                ValidateError::UnknownAction { node } => {
                    let mut start_pos = node.start_pos;
                    let mut end_pos = start_pos.clone();
                    end_pos.col += node.name.chars().count() as u32;
                    if !node.implicit_selector {
                        for (name, selector) in SELECTORS.entries() {
                            if selector == &node.selector {
                                start_pos.col += 1 + name.len() as u32;
                                end_pos.col += 1 + name.len() as u32;
                            }
                        }
                    }
                    print_err(format!("Unknown action '{}'", node.name), data, start_pos, Some(end_pos));
                }
                ValidateError::MissingArgument { node, index, name } => {
                    // TODO pos
                    print_err(format!("Missing argument '{}'", name), data, node.start_pos, None);
                }
                ValidateError::WrongArgumentType { node, index, name, expected_type, found_type } => {
                    // TODO pos
                    print_err(format!("Wrong argument type for '{}', expected '{:?}' but found '{:?}'", name, expected_type, found_type), data, node.start_pos, None);
                }
                ValidateError::TooManyArguments { node } => {
                    let mut start_pos = node.start_pos;
                    let mut end_pos = start_pos.clone();
                    end_pos.col += node.name.chars().count() as u32;
                    if !node.implicit_selector {
                        for (name, selector) in SELECTORS.entries() {
                            if selector == &node.selector {
                                start_pos.col += 1 + name.len() as u32;
                                end_pos.col += 1 + name.len() as u32;
                            }
                        }
                    }
                    // TODO pos
                    print_err(format!("Too many arguments for action '{}'", node.name), data, start_pos, Some(end_pos));
                }
            }
            std::process::exit(0);
        }
    }

    let compiled = compile(validated, config.debug.compile);
    send(compiled, config);
}