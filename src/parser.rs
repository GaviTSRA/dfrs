use crate::node::{ParticleCluster, ParticleData, StartNode};
use crate::{
  definitions::ArgType,
  node::{
    ActionNode, ActionType, Arg, ArgValue, ArgValueWithPos, CallNode, ConditionalNode,
    ConditionalType, EventNode, Expression, ExpressionNode, FileNode, FunctionNode,
    FunctionParamNode, ProcessNode, RepeatNode, VariableNode, VariableType,
  },
  token::{Keyword, Position, Selector, Token, TokenWithPos, SELECTORS, TYPES},
};

#[derive(Debug)]
pub enum ParseError {
  InvalidToken {
    found: Option<TokenWithPos>,
    expected: Vec<Token>,
  },
  UnknownVariable {
    found: String,
    start_pos: Position,
    end_pos: Position,
  },
  InvalidCall {
    pos: Position,
    msg: String,
  },
  InvalidComplexNumber {
    pos: Position,
    msg: String,
  },
  InvalidLocation {
    pos: Position,
    msg: String,
  },
  InvalidVector {
    pos: Position,
    msg: String,
  },
  InvalidSound {
    pos: Position,
    msg: String,
  },
  InvalidPotion {
    pos: Position,
    msg: String,
  },
  InvalidParticle {
    pos: Position,
    msg: String,
  },
  InvalidItem {
    pos: Position,
    msg: String,
  },
  InvalidType {
    found: Option<TokenWithPos>,
    start_pos: Position,
  },
}

pub struct Parser {
  tokens: Vec<TokenWithPos>,
  token_index: i32,
  current_token: Option<TokenWithPos>,
  variables: Vec<VariableNode>,
}

// TODO expected types for basically everything

impl Parser {
  pub fn new(tokens: Vec<TokenWithPos>) -> Parser {
    Parser {
      tokens,
      token_index: -1,
      current_token: None,
      variables: vec![],
    }
  }

  fn peak(&self) -> Option<TokenWithPos> {
    let index = self.token_index + 1;
    if index < self.tokens.len() as i32 {
      Some(self.tokens[index as usize].clone())
    } else {
      None
    }
  }

  fn advance(&mut self) -> Option<TokenWithPos> {
    self.token_index += 1;
    if self.token_index < self.tokens.len() as i32 {
      self.current_token = Some(self.tokens[self.token_index as usize].clone());
    } else {
      self.current_token = None
    }
    self.current_token.clone()
  }

  fn advance_err(&mut self) -> Result<TokenWithPos, ParseError> {
    let token = self.advance();

    if token.is_none() {
      return Err(ParseError::InvalidToken {
        found: None,
        expected: vec![],
      });
    }

    Ok(token.unwrap())
  }

  fn require_token(&mut self, required_token: Token) -> Result<TokenWithPos, ParseError> {
    let token = self.advance_err()?;
    if token.token == required_token {
      return Ok(token);
    }
    Err(ParseError::InvalidToken {
      found: Some(token),
      expected: vec![required_token],
    })
  }

  pub fn run(&mut self) -> Result<FileNode, ParseError> {
    self.file()
  }

  fn file(&mut self) -> Result<FileNode, ParseError> {
    let mut token = self.advance();
    let mut events: Vec<EventNode> = vec![];
    let mut functions: Vec<FunctionNode> = vec![];
    let mut processes: Vec<ProcessNode> = vec![];
    let start_pos = Position::new(1, 0);

    while token.is_some() {
      match token.clone().unwrap().token {
        Token::At => events.push(self.event()?),
        Token::Keyword { value } => match value {
          Keyword::Function => {
            functions.push(self.function()?);
          }
          Keyword::Process => {
            processes.push(self.process()?);
          }
          Keyword::VarGame => {
            let node = self.variable(VariableType::Game, None)?;
            self.variables.push(node);
          }
          Keyword::VarSave => {
            let node = self.variable(VariableType::Save, None)?;
            self.variables.push(node);
          }
          _ => {
            return Err(ParseError::InvalidToken {
              found: self.current_token.clone(),
              expected: vec![
                Token::At,
                Token::Keyword {
                  value: Keyword::Function,
                },
              ],
            })
          }
        },
        _ => {
          return Err(ParseError::InvalidToken {
            found: self.current_token.clone(),
            expected: vec![
              Token::At,
              Token::Keyword {
                value: Keyword::Function,
              },
              Token::Keyword {
                value: Keyword::VarGame,
              },
              Token::Keyword {
                value: Keyword::VarSave,
              },
            ],
          })
        }
      }
      token = self.advance();
      self.variables = self
        .variables
        .clone()
        .into_iter()
        .filter(|var| var.var_type == VariableType::Game || var.var_type == VariableType::Save)
        .collect::<Vec<VariableNode>>();
    }

    let end_pos = if !events.is_empty() {
      events.last().unwrap().end_pos.clone()
    } else {
      start_pos.clone()
    };
    Ok(FileNode {
      events,
      functions,
      processes,
      start_pos,
      end_pos,
    })
  }

  fn event(&mut self) -> Result<EventNode, ParseError> {
    let mut expressions: Vec<ExpressionNode> = vec![];
    let start_pos = self.current_token.clone().unwrap().end_pos;
    let mut cancelled = false;

    let name_token = self.advance_err()?;

    let event = match name_token.token {
      Token::Identifier { value } => value,
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![Token::Identifier {
            value: String::from("<any>"),
          }],
        })
      }
    };

    let mut token = self.advance_err()?;
    match token.token {
      Token::ExclamationMark => {
        cancelled = true;
        self.require_token(Token::OpenParenCurly)?;
      }
      Token::OpenParenCurly => {}
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![Token::OpenParenCurly, Token::ExclamationMark],
        })
      }
    }

    loop {
      token = self.advance_err()?;
      match token.token {
        Token::CloseParenCurly => break,
        _ => expressions.push(self.expression()?),
      }
    }

    Ok(EventNode {
      event_type: None,
      event,
      expressions,
      start_pos,
      name_end_pos: name_token.end_pos,
      end_pos: token.end_pos,
      cancelled,
    })
  }

  fn function(&mut self) -> Result<FunctionNode, ParseError> {
    let mut expressions: Vec<ExpressionNode> = vec![];
    let start_pos = self.current_token.clone().unwrap().end_pos;

    let name_token = self.advance_err()?;
    let dfrs_name = match name_token.token {
      Token::Identifier { value } => value,
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![Token::Identifier {
            value: String::from("<any>"),
          }],
        })
      }
    };
    let mut df_name = dfrs_name.clone();

    let mut token = self.advance_err()?;
    match token.token {
      Token::OpenParen => {}
      Token::Colon => {
        token = self.advance_err()?;
        match token.token {
          Token::Variable { value } => {
            df_name = value;
          }
          _ => {
            return Err(ParseError::InvalidToken {
              found: self.current_token.clone(),
              expected: vec![Token::Variable {
                value: "any".into(),
              }],
            })
          }
        }
        self.require_token(Token::OpenParen)?;
      }
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![Token::OpenParen],
        })
      }
    }

    let mut params: Vec<FunctionParamNode> = vec![];

    loop {
      let token = self.advance_err()?;
      let param_name = match token.token {
        Token::Identifier { value } => value,
        Token::CloseParen => break,
        _ => {
          return Err(ParseError::InvalidToken {
            found: self.current_token.clone(),
            expected: vec![
              Token::Identifier {
                value: "any".into(),
              },
              Token::CloseParen,
            ],
          })
        }
      };

      self.require_token(Token::Colon)?;

      let token = self.advance_err()?;
      let param_type = match token.token {
        Token::Identifier { value } => {
          if TYPES.contains_key(&value.clone()) {
            TYPES.get(&value).unwrap().to_owned()
          } else {
            return Err(ParseError::InvalidType {
              found: self.current_token.clone(),
              start_pos: token.start_pos,
            });
          }
        }
        _ => {
          return Err(ParseError::InvalidToken {
            found: self.current_token.clone(),
            expected: vec![Token::Identifier {
              value: "type".into(),
            }],
          })
        }
      };

      let mut optional = false;
      let mut multiple = false;
      let mut default = None;
      let mut token;
      loop {
        token = self.advance_err()?;
        match token.token {
          Token::Comma => {
            self.token_index -= 1;
            break;
          }
          Token::CloseParen => {
            self.token_index -= 1;
            break;
          }
          Token::Multiply => {
            if !multiple {
              multiple = true;
            } else {
              return Err(ParseError::InvalidToken {
                found: self.current_token.clone(),
                expected: vec![
                  Token::Comma,
                  Token::CloseParen,
                  Token::Multiply,
                  Token::QuestionMark,
                  Token::Equal,
                ],
              });
            }
          }
          Token::QuestionMark => {
            if !optional {
              optional = true;
            } else {
              return Err(ParseError::InvalidToken {
                found: self.current_token.clone(),
                expected: vec![
                  Token::Multiply,
                  Token::CloseParen,
                  Token::Comma,
                  Token::Equal,
                ],
              });
            }
          }
          Token::Equal => {
            let token = self.advance_err()?;
            default = Some(match token.token.clone() {
              Token::Number { value } => ArgValueWithPos {
                value: ArgValue::Number { number: value },
                start_pos: token.start_pos,
                end_pos: token.end_pos,
              },
              Token::Text { value } => ArgValueWithPos {
                value: ArgValue::Text { text: value },
                start_pos: token.start_pos,
                end_pos: token.end_pos,
              },
              Token::String { value } => ArgValueWithPos {
                value: ArgValue::String { string: value },
                start_pos: token.start_pos,
                end_pos: token.end_pos,
              },
              Token::Identifier { value } => match value.as_str() {
                "Location" => self.make_location()?,
                "Vector" => self.make_vector()?,
                "Sound" => self.make_sound()?,
                "Potion" => self.make_potion()?,
                _ => {
                  return Err(ParseError::InvalidToken {
                    found: self.current_token.clone(),
                    expected: vec![],
                  })
                }
              },
              _ => {
                return Err(ParseError::InvalidToken {
                  found: self.current_token.clone(),
                  expected: vec![Token::Identifier {
                    value: "any".into(),
                  }],
                })
              }
            });
            let token = self.advance_err()?;
            match token.token {
              Token::Comma => {
                self.token_index -= 1;
                break;
              }
              Token::CloseParen => {
                self.token_index -= 1;
                break;
              }
              _ => {
                return Err(ParseError::InvalidToken {
                  found: self.current_token.clone(),
                  expected: vec![Token::Comma, Token::CloseParen],
                })
              }
            }
          }
          _ => {
            return Err(ParseError::InvalidToken {
              found: self.current_token.clone(),
              expected: vec![],
            })
          }
        }
      }

      self.variables.push(VariableNode {
        dfrs_name: param_name.clone(),
        df_name: param_name.clone(),
        action: None,
        var_type: VariableType::Line,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      });

      params.push(FunctionParamNode {
        name: param_name,
        param_type,
        optional,
        multiple,
        default,
      });

      let token = self.advance_err()?;
      match token.token {
        Token::Comma => {}
        Token::CloseParen => break,
        _ => {
          return Err(ParseError::InvalidToken {
            found: self.current_token.clone(),
            expected: vec![Token::Comma, Token::CloseParen],
          })
        }
      }
    }

    self.require_token(Token::OpenParenCurly)?;

    let mut token;
    loop {
      token = self.advance_err()?;
      match token.token {
        Token::CloseParenCurly => break,
        _ => expressions.push(self.expression()?),
      }
    }

    Ok(FunctionNode {
      df_name,
      dfrs_name,
      expressions,
      start_pos,
      name_end_pos: name_token.end_pos,
      end_pos: token.end_pos,
      params,
    })
  }

  fn process(&mut self) -> Result<ProcessNode, ParseError> {
    let mut expressions: Vec<ExpressionNode> = vec![];
    let start_pos = self.current_token.clone().unwrap().end_pos;

    let name_token = self.advance_err()?;
    let name = match name_token.token {
      Token::Identifier { value } => value,
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![Token::Identifier {
            value: String::from("<any>"),
          }],
        })
      }
    };

    self.require_token(Token::OpenParenCurly)?;

    let mut token;
    loop {
      token = self.advance_err()?;
      match token.token {
        Token::CloseParenCurly => break,
        _ => expressions.push(self.expression()?),
      }
    }

    Ok(ProcessNode {
      name,
      expressions,
      start_pos,
      name_end_pos: name_token.end_pos,
      end_pos: token.end_pos,
    })
  }

  fn expression(&mut self) -> Result<ExpressionNode, ParseError> {
    let token = self.current_token.clone().unwrap();
    let node;
    let start_pos = token.start_pos.clone();
    let end_pos;

    match token.token.clone() {
      Token::Keyword { value } => match value {
        Keyword::P => {
          let res = self.action(ActionType::Player)?;
          end_pos = res.end_pos.clone();
          node = Expression::Action { node: res };
        }
        Keyword::E => {
          let res = self.action(ActionType::Entity)?;
          end_pos = res.end_pos.clone();
          node = Expression::Action { node: res };
        }
        Keyword::G => {
          let res = self.action(ActionType::Game)?;
          end_pos = res.end_pos.clone();
          node = Expression::Action { node: res };
        }
        Keyword::V => {
          let res = self.action(ActionType::Variable)?;
          end_pos = res.end_pos.clone();
          node = Expression::Action { node: res };
        }
        Keyword::C => {
          let res = self.action(ActionType::Control)?;
          end_pos = res.end_pos.clone();
          node = Expression::Action { node: res };
        }
        Keyword::S => {
          let res = self.action(ActionType::Select)?;
          end_pos = res.end_pos.clone();
          node = Expression::Action { node: res };
        }
        Keyword::IfP => {
          let res = self.conditional(ConditionalType::Player)?;
          end_pos = res.end_pos.clone();
          node = Expression::Conditional { node: res };
        }
        Keyword::IfE => {
          let res = self.conditional(ConditionalType::Entity)?;
          end_pos = res.end_pos.clone();
          node = Expression::Conditional { node: res };
        }
        Keyword::IfG => {
          let res = self.conditional(ConditionalType::Game)?;
          end_pos = res.end_pos.clone();
          node = Expression::Conditional { node: res };
        }
        Keyword::IfV => {
          let res = self.conditional(ConditionalType::Variable)?;
          end_pos = res.end_pos.clone();
          node = Expression::Conditional { node: res };
        }
        Keyword::VarLine => {
          let res = self.variable(VariableType::Line, None)?;
          end_pos = res.end_pos.clone();
          node = Expression::Variable { node: res }
        }
        Keyword::VarLocal => {
          let res = self.variable(VariableType::Local, None)?;
          end_pos = res.end_pos.clone();
          node = Expression::Variable { node: res }
        }
        Keyword::Call => {
          let res = self.call()?;
          end_pos = res.end_pos.clone();
          node = Expression::Call { node: res }
        }
        Keyword::Start => {
          let res = self.start()?;
          end_pos = res.end_pos.clone();
          node = Expression::Start { node: res }
        }
        Keyword::Repeat => {
          let res = self.repeat()?;
          end_pos = res.end_pos.clone();
          node = Expression::Repeat { node: res }
        }
        _ => {
          return Err(ParseError::InvalidToken {
            found: self.current_token.clone(),
            expected: vec![
              Token::Keyword { value: Keyword::E },
              Token::Keyword { value: Keyword::P },
            ],
          })
        }
      },
      Token::Identifier { value } => {
        if let Some((variable, scope)) = self.get_variable(value) {
          let var_type = match scope.as_str() {
            "line" => VariableType::Line,
            "local" => VariableType::Local,
            "unsaved" => VariableType::Game,
            "saved" => VariableType::Save,
            _ => unreachable!("unknown type {scope}"),
          };

          let res = self.variable(var_type, Some(variable))?;
          end_pos = res.end_pos.clone();
          node = Expression::Variable { node: res }
        } else {
          return Err(ParseError::InvalidToken {
            found: self.current_token.clone(),
            expected: vec![
              Token::Keyword { value: Keyword::E },
              Token::Keyword { value: Keyword::P },
            ],
          });
        }
      }
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![
            Token::Keyword { value: Keyword::E },
            Token::Keyword { value: Keyword::P },
          ],
        })
      }
    }

    Ok(ExpressionNode {
      node: node.clone(),
      start_pos,
      end_pos,
    })
  }

  fn action(&mut self, action_type: ActionType) -> Result<ActionNode, ParseError> {
    let mut selector = Selector::Default;
    let mut implicit_selector = true;
    let mut token = self.advance_err()?;
    let mut start_pos = token.start_pos.clone();
    start_pos.col += 1;

    match token.token {
      Token::Colon => {
        if action_type == ActionType::Variable {
          return Err(ParseError::InvalidToken {
            found: self.current_token.clone(),
            expected: vec![Token::Dot],
          });
        }
        token = self.advance_err()?;
        match token.token {
          Token::Identifier { value } => {
            if let Some(value) = SELECTORS.get(&value).cloned() {
              selector = value;
              implicit_selector = false;
              self.require_token(Token::Dot)?;
            } else {
              return Err(ParseError::InvalidToken {
                found: self.current_token.clone(),
                expected: vec![Token::Identifier {
                  value: "<selector>".to_string(),
                }],
              });
            }
          }
          _ => {
            return Err(ParseError::InvalidToken {
              found: self.current_token.clone(),
              expected: vec![Token::Identifier {
                value: "<selector>".to_string(),
              }],
            })
          }
        }
      }
      Token::Dot => {}
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![],
        })
      }
    }

    token = self.advance_err()?;
    let name = match token.token {
      Token::Identifier { value } => value,
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![Token::Identifier {
            value: String::from("<any>"),
          }],
        })
      }
    };

    let args = self.make_args()?;

    let mut selector_start_pos = start_pos.clone();
    selector_start_pos.col += 2;
    let mut selector_end_pos = selector_start_pos.clone();
    if !implicit_selector {
      for (name, sel) in SELECTORS.entries() {
        if sel == &selector {
          selector_end_pos.col += 1 + name.len() as u32;
        }
      }
    }

    self.require_token(Token::Semicolon)?;

    Ok(ActionNode {
      action_type,
      selector,
      name,
      args,
      start_pos,
      selector_start_pos,
      selector_end_pos,
      end_pos: token.end_pos,
    })
  }

  fn conditional(
    &mut self,
    conditional_type: ConditionalType,
  ) -> Result<ConditionalNode, ParseError> {
    let mut token = self.advance_err()?;
    let mut selector = Selector::Default;
    let start_pos = token.start_pos.clone();
    let mut selector_start_pos = None;
    let mut selector_end_pos = None;
    let mut inverted = false;

    match token.token {
      Token::ExclamationMark => {
        inverted = true;
        token = self.advance_err()?;
      }
      _ => {}
    }

    match &token.token {
      Token::Identifier { value } => {
        if let Some(value) = SELECTORS.get(&value).cloned() {
          selector = value;
          selector_start_pos = Some(token.start_pos);
          selector_end_pos = Some(token.end_pos);
          self.require_token(Token::Colon)?;
          token = self.advance_err()?;
        }
      }
      _ => {}
    }
    let name = match token.token {
      Token::Identifier { value } => value,
      _ => {
        return Err(ParseError::InvalidToken {
          found: Some(token),
          expected: vec![Token::Identifier {
            value: "any".into(),
          }],
        })
      }
    };

    let args = self.make_args()?;
    let end_pos = token.end_pos;

    self.require_token(Token::OpenParenCurly)?;
    let mut expressions = vec![];
    loop {
      token = self.advance_err()?;
      match token.token {
        Token::CloseParenCurly => break,
        _ => {
          let expression = self.expression()?;
          expressions.push(expression);
        }
      }
    }

    let mut else_expressions = vec![];
    match self.peak() {
      Some(token) => match token.token {
        Token::Keyword { value } => match value {
          Keyword::Else => {
            self.advance_err()?;
            self.require_token(Token::OpenParenCurly)?;
            loop {
              let token = self.advance_err()?;
              match token.token {
                Token::CloseParenCurly => break,
                _ => {
                  let expression = self.expression()?;
                  else_expressions.push(expression);
                }
              }
            }
          }
          _ => {}
        },
        _ => {}
      },
      None => {}
    }

    Ok(ConditionalNode {
      conditional_type,
      selector,
      name,
      args,
      selector_start_pos,
      selector_end_pos,
      start_pos,
      end_pos,
      expressions,
      else_expressions,
      inverted,
    })
  }

  fn call(&mut self) -> Result<CallNode, ParseError> {
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let mut args = self.make_args()?;

    if args.is_empty() {
      return Err(ParseError::InvalidCall {
        pos: start_pos,
        msg: "Missing function name".into(),
      });
    }
    let name_arg = args.remove(0);
    let name = match name_arg.value {
      ArgValue::Text { text } => text,
      _ => {
        return Err(ParseError::InvalidCall {
          pos: start_pos,
          msg: "Invalid function name param type".into(),
        })
      }
    };
    for arg in args.iter_mut() {
      arg.index -= 1;
    }
    self.require_token(Token::Semicolon)?;

    let end_pos = self.current_token.clone().unwrap().end_pos;

    Ok(CallNode {
      name,
      args,
      start_pos,
      end_pos,
    })
  }

  fn start(&mut self) -> Result<StartNode, ParseError> {
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let mut args = self.make_args()?;

    if args.is_empty() {
      return Err(ParseError::InvalidCall {
        pos: start_pos,
        msg: "Missing process name".into(),
      });
    }
    let name_arg = args.remove(0);
    let name = match name_arg.value {
      ArgValue::Text { text } => text,
      _ => {
        return Err(ParseError::InvalidCall {
          pos: start_pos,
          msg: "Invalid process name param type".into(),
        })
      }
    };
    for arg in args.iter_mut() {
      arg.index -= 1;
    }
    self.require_token(Token::Semicolon)?;

    let end_pos = self.current_token.clone().unwrap().end_pos;

    Ok(StartNode {
      name,
      args,
      start_pos,
      end_pos,
    })
  }

  fn repeat(&mut self) -> Result<RepeatNode, ParseError> {
    let mut token = self.advance_err()?;
    let start_pos = token.start_pos.clone();

    let name = match token.token {
      Token::Identifier { value } => value,
      _ => {
        return Err(ParseError::InvalidToken {
          found: Some(token),
          expected: vec![Token::Identifier {
            value: "any".into(),
          }],
        })
      }
    };

    let args = self.make_args()?;
    let end_pos = token.end_pos;

    self.require_token(Token::OpenParenCurly)?;
    let mut expressions = vec![];
    loop {
      token = self.advance_err()?;
      match token.token {
        Token::CloseParenCurly => break,
        _ => {
          let expression = self.expression()?;
          expressions.push(expression);
        }
      }
    }

    Ok(RepeatNode {
      name,
      args,
      start_pos,
      end_pos,
      expressions,
    })
  }

  fn variable(
    &mut self,
    var_type: VariableType,
    name: Option<String>,
  ) -> Result<VariableNode, ParseError> {
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let end_pos = start_pos.clone();

    let dfrs_name = if let Some(name) = name {
      name
    } else {
      let token = self.advance_err()?;
      match token.token {
        Token::Identifier { value } => value,
        _ => {
          return Err(ParseError::InvalidToken {
            found: self.current_token.clone(),
            expected: vec![Token::Identifier {
              value: "any".into(),
            }],
          })
        }
      }
    };

    let token = self.advance_err()?;
    let mut df_name = dfrs_name.clone();
    match token.token {
      Token::Colon => {
        let token = self.advance_err()?;
        df_name = match token.token {
          Token::Variable { value } => value,
          _ => {
            return Err(ParseError::InvalidToken {
              found: self.current_token.clone(),
              expected: vec![Token::Variable {
                value: "any".into(),
              }],
            })
          }
        };
        let token = self.advance_err()?;
        match token.token {
          Token::Equal => {}
          Token::Semicolon => {
            return {
              let node = VariableNode {
                dfrs_name: dfrs_name.clone(),
                df_name,
                var_type,
                action: None,
                start_pos,
                end_pos,
              };
              self.variables.push(node.clone());
              Ok(node)
            }
          }
          _ => {
            return Err(ParseError::InvalidToken {
              found: self.current_token.clone(),
              expected: vec![Token::Equal, Token::Semicolon],
            })
          }
        }
      }
      Token::Equal => {}
      Token::Semicolon => {
        return {
          let node = VariableNode {
            dfrs_name: dfrs_name.clone(),
            df_name,
            var_type,
            action: None,
            start_pos,
            end_pos,
          };
          self.variables.push(node.clone());
          Ok(node)
        }
      }
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![Token::Equal, Token::Semicolon],
        })
      }
    };

    let token = self.advance_err()?;
    let action = match token.token {
      Token::Keyword { value } => match value {
        Keyword::P => self.action(ActionType::Player)?,
        Keyword::E => self.action(ActionType::Entity)?,
        Keyword::G => self.action(ActionType::Game)?,
        Keyword::V => self.action(ActionType::Variable)?,
        Keyword::C => self.action(ActionType::Control)?,
        Keyword::S => self.action(ActionType::Select)?,
        _ => {
          return Err(ParseError::InvalidToken {
            found: self.current_token.clone(),
            expected: vec![Token::Keyword { value: Keyword::P }],
          })
        }
      },
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![Token::Keyword { value: Keyword::P }],
        })
      }
    };

    let node = VariableNode {
      dfrs_name,
      df_name,
      action: Some(action),
      var_type,
      start_pos,
      end_pos,
    };
    self.variables.push(node.clone());
    Ok(node)
  }

  fn make_params(&mut self) -> Result<Vec<ArgValueWithPos>, ParseError> {
    let token = self.advance_err()?;
    match token.token {
      Token::OpenParen => {}
      _ => {
        return Err(ParseError::InvalidToken {
          found: self.current_token.clone(),
          expected: vec![Token::OpenParen],
        })
      }
    }

    let mut params: Vec<ArgValueWithPos> = vec![];
    let mut is_value = false;
    let mut could_be_tag = false;
    let mut tag_name: String = "".into();
    let mut is_tag = false;
    let mut tag_start_pos = Position::new(0, 0);
    let mut tag_end_pos = Position::new(0, 0);
    let mut comma_pos = Position::new(0, 0);
    let mut is_game_value = false;

    let expected = vec![Token::CloseParen];
    loop {
      let mut token = self.advance_err()?;

      if is_value {
        match token.token {
          Token::Comma => {
            is_value = false;
            comma_pos = self.current_token.clone().unwrap().start_pos;
          }
          Token::CloseParen => break,
          _ => {
            return Err(ParseError::InvalidToken {
              found: self.current_token.clone(),
              expected: vec![Token::Comma, Token::CloseParen],
            })
          }
        }
      } else if could_be_tag {
        could_be_tag = false;
        match token.token.clone() {
          Token::Equal => {
            is_tag = true;
          }
          _ => {
            if let Some((var, scope)) = self.get_variable(tag_name.clone()) {
              params.push(ArgValueWithPos {
                value: ArgValue::Variable { name: var, scope },
                start_pos: self.current_token.clone().unwrap().start_pos,
                end_pos: self.current_token.clone().unwrap().end_pos,
              });
              is_value = true;
              self.token_index -= 1;
            } else {
              return Err(ParseError::UnknownVariable {
                found: tag_name.clone(),
                start_pos: tag_start_pos,
                end_pos: tag_end_pos,
              });
            }
          }
        }
      } else if is_tag {
        is_tag = false;
        match token.token.clone() {
          Token::String { value } => {
            let data = Box::new(ArgValue::Text { text: value });
            params.push(ArgValueWithPos {
              value: ArgValue::Tag {
                tag: tag_name.clone(),
                value: data,
                definition: None,
                name_end_pos: tag_end_pos.clone(),
                value_start_pos: token.start_pos.clone(),
              },
              start_pos: tag_start_pos.clone(),
              end_pos: token.end_pos,
            });
            is_value = true;
          }
          Token::Text { value } => {
            let data = Box::new(ArgValue::Text { text: value });
            params.push(ArgValueWithPos {
              value: ArgValue::Tag {
                tag: tag_name.clone(),
                value: data,
                definition: None,
                name_end_pos: tag_end_pos.clone(),
                value_start_pos: token.start_pos,
              },
              start_pos: tag_start_pos.clone(),
              end_pos: token.end_pos,
            });
            is_value = true;
          }
          Token::Number { value } => {
            let data = Box::new(ArgValue::Number { number: value });
            params.push(ArgValueWithPos {
              value: ArgValue::Tag {
                tag: tag_name.clone(),
                value: data,
                definition: None,
                name_end_pos: tag_end_pos.clone(),
                value_start_pos: token.start_pos,
              },
              start_pos: tag_start_pos.clone(),
              end_pos: token.end_pos,
            });
            is_value = true;
          }
          Token::Identifier { value } => match value.as_str() {
            "Vector" => {
              let vector = self.make_vector()?;
              let data = Box::new(vector.value);
              params.push(ArgValueWithPos {
                value: ArgValue::Tag {
                  tag: tag_name.clone(),
                  value: data,
                  definition: None,
                  name_end_pos: tag_end_pos.clone(),
                  value_start_pos: token.start_pos,
                },
                start_pos: tag_start_pos.clone(),
                end_pos: token.end_pos,
              });
              is_value = true;
            }
            _ => {
              return Err(ParseError::InvalidToken {
                found: Some(token),
                expected: vec![
                  Token::String {
                    value: "<any>".into(),
                  },
                  Token::Text {
                    value: "<any>".into(),
                  },
                ],
              })
            }
          },
          _ => {
            return Err(ParseError::InvalidToken {
              found: Some(token),
              expected: vec![
                Token::String {
                  value: "<any>".into(),
                },
                Token::Text {
                  value: "<any>".into(),
                },
              ],
            })
          }
        }
      } else if is_game_value {
        let mut selector = Selector::Default;
        let mut selector_end_pos = token.start_pos.clone();
        let start_pos = token.start_pos.clone();

        if let Token::Identifier { value } = token.token.clone() {
          if let Some(value) = SELECTORS.get(&value).cloned() {
            selector = value;
            token = self.advance_err()?;
            if token.token != Token::Colon {
              return Err(ParseError::InvalidToken {
                found: Some(token),
                expected: vec![Token::Colon],
              });
            }
            selector_end_pos = token.end_pos;
            token = self.advance_err()?;
          }
        }

        match token.token.clone() {
          Token::Identifier { value } => {
            params.push(ArgValueWithPos {
              value: ArgValue::GameValue {
                dfrs_name: value,
                df_name: None,
                selector,
                selector_end_pos,
              },
              start_pos,
              end_pos: token.end_pos.clone(),
            });
            is_value = true;
            is_game_value = false;
          }
          _ => {
            return Err(ParseError::InvalidToken {
              found: Some(token),
              expected: vec![
                Token::Identifier {
                  value: "<any>".into(),
                },
                Token::Identifier {
                  value: "<selector>".into(),
                },
              ],
            })
          }
        }
      } else {
        match token.token.clone() {
          Token::Number { value } => {
            params.push(ArgValueWithPos {
              value: ArgValue::Number { number: value },
              start_pos: token.start_pos,
              end_pos: token.end_pos,
            });
            is_value = true;
          }
          Token::Text { value } => {
            params.push(ArgValueWithPos {
              value: ArgValue::Text { text: value },
              start_pos: token.start_pos,
              end_pos: token.end_pos,
            });
            is_value = true;
          }
          Token::String { value } => {
            params.push(ArgValueWithPos {
              value: ArgValue::String { string: value },
              start_pos: token.start_pos,
              end_pos: token.end_pos,
            });
            is_value = true;
          }
          Token::Identifier { value } => match value.as_str() {
            "Number" => {
              params.push(self.make_complex_number()?);
              is_value = true;
            }
            "Location" => {
              params.push(self.make_location()?);
              is_value = true;
            }
            "Vector" => {
              params.push(self.make_vector()?);
              is_value = true;
            }
            "Sound" => {
              params.push(self.make_sound()?);
              is_value = true;
            }
            "Potion" => {
              params.push(self.make_potion()?);
              is_value = true;
            }
            "Particle" => {
              params.push(self.make_particle()?);
              is_value = true;
            }
            "Item" => {
              params.push(self.make_item()?);
              is_value = true;
            }
            "null" => {
              params.push(ArgValueWithPos {
                value: ArgValue::Empty,
                start_pos: self.current_token.clone().unwrap().start_pos,
                end_pos: self.current_token.clone().unwrap().end_pos,
              });
              is_value = true;
            }
            _ => {
              could_be_tag = true;
              tag_name = value;
              tag_start_pos = self.current_token.clone().unwrap().start_pos;
              tag_end_pos = self.current_token.clone().unwrap().end_pos;
            }
          },
          Token::Dollar => is_game_value = true,
          Token::Keyword { value } => {
            let arg = match value {
              Keyword::IfP => self.conditional_arg(ConditionalType::Player)?,
              Keyword::IfE => self.conditional_arg(ConditionalType::Entity)?,
              Keyword::IfG => self.conditional_arg(ConditionalType::Game)?,
              Keyword::IfV => self.conditional_arg(ConditionalType::Variable)?,
              _ => {
                return Err(ParseError::InvalidToken {
                  found: self.current_token.clone(),
                  expected,
                })
              }
            };
            params.push(arg);
            is_value = true;
          }
          Token::CloseParen => break,
          _ => {
            return Err(ParseError::InvalidToken {
              found: self.current_token.clone(),
              expected,
            })
          }
        }
      }
    }
    if !params.is_empty() && !is_value {
      return Err(ParseError::InvalidToken {
        found: Some(TokenWithPos::new(
          Token::Comma,
          comma_pos.clone(),
          comma_pos,
        )),
        expected,
      });
    }
    Ok(params)
  }

  fn make_args(&mut self) -> Result<Vec<Arg>, ParseError> {
    let params = self.make_params()?;
    let mut args = vec![];
    for (i, param) in params.into_iter().enumerate() {
      let arg_type = match param.value {
        ArgValue::Empty => ArgType::EMPTY,
        ArgValue::Number { .. } => ArgType::NUMBER,
        ArgValue::ComplexNumber { .. } => ArgType::NUMBER,
        ArgValue::String { .. } => ArgType::STRING,
        ArgValue::Text { .. } => ArgType::TEXT,
        ArgValue::Location { .. } => ArgType::LOCATION,
        ArgValue::Potion { .. } => ArgType::POTION,
        ArgValue::Sound { .. } => ArgType::SOUND,
        ArgValue::Particle { .. } => ArgType::PARTICLE,
        ArgValue::Item { .. } => ArgType::ITEM,
        ArgValue::Vector { .. } => ArgType::VECTOR,
        ArgValue::Tag { .. } => ArgType::TAG,
        ArgValue::Variable { .. } => ArgType::VARIABLE,
        ArgValue::GameValue { .. } => ArgType::GameValue,
        ArgValue::Condition { .. } => ArgType::CONDITION,
      };
      args.push(Arg {
        value: param.value,
        index: i as i32,
        arg_type,
        start_pos: param.start_pos,
        end_pos: param.end_pos,
      });
    }
    Ok(args)
  }

  fn make_complex_number(&mut self) -> Result<ArgValueWithPos, ParseError> {
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let params = self.make_params()?;

    if params.len() < 1 {
      return Err(ParseError::InvalidComplexNumber {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Not enough arguments".into(),
      });
    }
    let number = match params[0].value.clone() {
      ArgValue::Text { text } => text,
      _ => {
        return Err(ParseError::InvalidComplexNumber {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid value, should be text".into(),
        })
      }
    };
    if params.len() > 1 {
      return Err(ParseError::InvalidComplexNumber {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::ComplexNumber { number },
      start_pos,
      end_pos: self.current_token.clone().unwrap().end_pos,
    })
  }

  fn make_location(&mut self) -> Result<ArgValueWithPos, ParseError> {
    let mut pitch = None;
    let mut yaw = None;
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let loc_params = self.make_params()?;

    if loc_params.len() < 3 {
      return Err(ParseError::InvalidLocation {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Not enough arguments".into(),
      });
    }
    let x = match loc_params[0].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidLocation {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid x coordinate".into(),
        })
      }
    };
    let y = match loc_params[1].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidLocation {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid y coordinate".into(),
        })
      }
    };
    let z = match loc_params[2].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidLocation {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid z coordinate".into(),
        })
      }
    };
    if loc_params.len() >= 4 {
      match loc_params[3].value {
        ArgValue::Number { number } => pitch = Some(number),
        _ => {
          return Err(ParseError::InvalidLocation {
            pos: self.current_token.clone().unwrap().start_pos,
            msg: "Invalid pitch".into(),
          })
        }
      }
    }
    if loc_params.len() == 5 {
      match loc_params[4].value {
        ArgValue::Number { number } => yaw = Some(number),
        _ => {
          return Err(ParseError::InvalidLocation {
            pos: self.current_token.clone().unwrap().start_pos,
            msg: "Invalid yaw".into(),
          })
        }
      }
    }
    if loc_params.len() > 5 {
      return Err(ParseError::InvalidLocation {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::Location {
        x,
        y,
        z,
        pitch,
        yaw,
      },
      start_pos,
      end_pos: self.current_token.clone().unwrap().end_pos,
    })
  }

  fn make_vector(&mut self) -> Result<ArgValueWithPos, ParseError> {
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let vec_params = self.make_params()?;

    if vec_params.len() < 3 {
      return Err(ParseError::InvalidVector {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Not enough arguments".into(),
      });
    }
    let x = match vec_params[0].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidVector {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid x coordinate".into(),
        })
      }
    };
    let y = match vec_params[1].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidVector {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid y coordinate".into(),
        })
      }
    };
    let z = match vec_params[2].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidVector {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid z coordinate".into(),
        })
      }
    };
    if vec_params.len() > 3 {
      return Err(ParseError::InvalidVector {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::Vector { x, y, z },
      start_pos,
      end_pos: self.current_token.clone().unwrap().end_pos,
    })
  }

  fn make_sound(&mut self) -> Result<ArgValueWithPos, ParseError> {
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let sound_params = self.make_params()?;

    if sound_params.len() < 3 {
      return Err(ParseError::InvalidSound {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Not enough arguments".into(),
      });
    }
    let sound = match &sound_params[0].value {
      ArgValue::String { string } => string.clone(),
      ArgValue::Text { text } => text.clone(),
      _ => {
        return Err(ParseError::InvalidSound {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid sound type".into(),
        })
      }
    };
    let volume = match sound_params[1].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidSound {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid volume".into(),
        })
      }
    };
    let pitch = match sound_params[2].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidSound {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid pitch".into(),
        })
      }
    };
    if sound_params.len() > 3 {
      return Err(ParseError::InvalidSound {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::Sound {
        sound,
        volume,
        pitch,
      },
      start_pos,
      end_pos: self.current_token.clone().unwrap().end_pos,
    })
  }

  fn make_potion(&mut self) -> Result<ArgValueWithPos, ParseError> {
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let potion_params = self.make_params()?;

    if potion_params.len() < 3 {
      return Err(ParseError::InvalidPotion {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Not enough arguments".into(),
      });
    }
    let potion = match &potion_params[0].value {
      ArgValue::String { string } => string.clone(),
      ArgValue::Text { text } => text.clone(),
      _ => {
        return Err(ParseError::InvalidPotion {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid potion type".into(),
        })
      }
    };
    let amplifier = match potion_params[1].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidPotion {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid amplifier".into(),
        })
      }
    };
    let duration = match potion_params[2].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidPotion {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid duration".into(),
        })
      }
    };
    if potion_params.len() > 3 {
      return Err(ParseError::InvalidPotion {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::Potion {
        potion,
        amplifier,
        duration,
      },
      start_pos,
      end_pos: self.current_token.clone().unwrap().end_pos,
    })
  }

  fn make_particle(&mut self) -> Result<ArgValueWithPos, ParseError> {
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let mut particle_params = self.make_params()?;

    if particle_params.len() < 4 {
      return Err(ParseError::InvalidParticle {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Not enough arguments".into(),
      });
    }
    let particle = match particle_params.remove(0).value {
      ArgValue::String { string } => string.clone(),
      ArgValue::Text { text } => text.clone(),
      _ => {
        return Err(ParseError::InvalidParticle {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid particle type".into(),
        })
      }
    };
    let amount = match particle_params.remove(0).value {
      ArgValue::Number { number } => number as i32,
      _ => {
        return Err(ParseError::InvalidParticle {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid particle amount".into(),
        })
      }
    };
    let horizontal = match particle_params.remove(0).value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidParticle {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid particle horizontal spread".into(),
        })
      }
    };
    let vertical = match particle_params.remove(0).value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParseError::InvalidParticle {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid particle vertical spread".into(),
        })
      }
    };

    let mut x: Option<f32> = None;
    let mut y: Option<f32> = None;
    let mut z: Option<f32> = None;
    let mut motion_variation: Option<i32> = None;
    let mut rgb: Option<i32> = None;
    let mut rgb_fade: Option<i32> = None;
    let mut color_variation: Option<i32> = None;
    let mut material: Option<String> = None;
    let mut size: Option<f32> = None;
    let mut size_variation: Option<i32> = None;
    let mut roll: Option<f32> = None;
    for arg in particle_params {
      match arg.value {
        ArgValue::Tag { tag, value, .. } => match tag.as_str() {
          "motion" => match value.as_ref() {
            ArgValue::Vector {
              x: x2,
              y: y2,
              z: z2,
            } => {
              x = Some(x2.clone());
              y = Some(y2.clone());
              z = Some(z2.clone());
            }
            _ => {
              return Err(ParseError::InvalidParticle {
                pos: self.current_token.clone().unwrap().start_pos,
                msg: "Expected motion to be vector".into(),
              })
            }
          },
          "motionVariation" => match value.as_ref() {
            ArgValue::Number { number } => motion_variation = Some(number.clone() as i32),
            _ => {
              return Err(ParseError::InvalidParticle {
                pos: self.current_token.clone().unwrap().start_pos,
                msg: "Expected motion variation to be number".into(),
              })
            }
          },
          "rgb" => match value.as_ref() {
            ArgValue::Number { number } => rgb = Some(number.clone() as i32),
            _ => {
              return Err(ParseError::InvalidParticle {
                pos: self.current_token.clone().unwrap().start_pos,
                msg: "Expected rgb to be number".into(),
              })
            }
          },
          "rgbFade" => match value.as_ref() {
            ArgValue::Number { number } => rgb_fade = Some(number.clone() as i32),
            _ => {
              return Err(ParseError::InvalidParticle {
                pos: self.current_token.clone().unwrap().start_pos,
                msg: "Expected rgb fade to be number".into(),
              })
            }
          },
          "colorVariation" => match value.as_ref() {
            ArgValue::Number { number } => color_variation = Some(number.clone() as i32),
            _ => {
              return Err(ParseError::InvalidParticle {
                pos: self.current_token.clone().unwrap().start_pos,
                msg: "Expected color variation to be number".into(),
              })
            }
          },
          "material" => match value.as_ref() {
            ArgValue::Text { text } => material = Some(text.clone()),
            _ => {
              return Err(ParseError::InvalidParticle {
                pos: self.current_token.clone().unwrap().start_pos,
                msg: "Expected material to be text".into(),
              })
            }
          },
          "size" => match value.as_ref() {
            ArgValue::Number { number } => size = Some(number.clone()),
            _ => {
              return Err(ParseError::InvalidParticle {
                pos: self.current_token.clone().unwrap().start_pos,
                msg: "Expected size to be number".into(),
              })
            }
          },
          "sizeVariation" => match value.as_ref() {
            ArgValue::Number { number } => size_variation = Some(number.clone() as i32),
            _ => {
              return Err(ParseError::InvalidParticle {
                pos: self.current_token.clone().unwrap().start_pos,
                msg: "Expected size variation to be number".into(),
              })
            }
          },
          "roll" => match value.as_ref() {
            ArgValue::Number { number } => roll = Some(number.clone()),
            _ => {
              return Err(ParseError::InvalidParticle {
                pos: self.current_token.clone().unwrap().start_pos,
                msg: "Expected roll to be number".into(),
              })
            }
          },
          _ => {
            return Err(ParseError::InvalidParticle {
              pos: self.current_token.clone().unwrap().start_pos,
              msg: "Unknown tag".into(),
            })
          }
        },
        _ => {
          return Err(ParseError::InvalidParticle {
            pos: self.current_token.clone().unwrap().start_pos,
            msg: "Too many arguments".into(),
          })
        }
      }
    }

    Ok(ArgValueWithPos {
      value: ArgValue::Particle {
        particle,
        cluster: ParticleCluster {
          amount,
          horizontal,
          vertical,
        },
        data: ParticleData {
          x,
          y,
          z,
          motion_variation,
          rgb,
          rgb_fade,
          color_variation,
          material,
          size,
          size_variation,
          roll,
        },
      },
      start_pos,
      end_pos: self.current_token.clone().unwrap().end_pos,
    })
  }

  fn make_item(&mut self) -> Result<ArgValueWithPos, ParseError> {
    let start_pos = self.current_token.clone().unwrap().start_pos;
    let item_params = self.make_params()?;

    if item_params.len() < 1 {
      return Err(ParseError::InvalidItem {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Not enough arguments".into(),
      });
    }
    let item = match &item_params[0].value {
      ArgValue::String { string } => string.clone(),
      ArgValue::Text { text } => text.clone(),
      _ => {
        return Err(ParseError::InvalidItem {
          pos: self.current_token.clone().unwrap().start_pos,
          msg: "Invalid item arg type".into(),
        })
      }
    };
    if item_params.len() > 1 {
      return Err(ParseError::InvalidItem {
        pos: self.current_token.clone().unwrap().start_pos,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::Item { item },
      start_pos,
      end_pos: self.current_token.clone().unwrap().end_pos,
    })
  }

  fn conditional_arg(
    &mut self,
    conditional_type: ConditionalType,
  ) -> Result<ArgValueWithPos, ParseError> {
    let mut token = self.advance_err()?;
    let mut selector = Selector::Default;
    let start_pos = token.start_pos.clone();
    let mut inverted = false;

    match token.token {
      Token::ExclamationMark => {
        inverted = true;
        token = self.advance_err()?;
      }
      _ => {}
    }

    match &token.token {
      Token::Identifier { value } => {
        if let Some(value) = SELECTORS.get(&value).cloned() {
          selector = value;
          self.require_token(Token::Colon)?;
          token = self.advance_err()?;
        }
      }
      _ => {}
    }
    let name = match token.token {
      Token::Identifier { value } => value,
      _ => {
        return Err(ParseError::InvalidToken {
          found: Some(token),
          expected: vec![Token::Identifier {
            value: "any".into(),
          }],
        })
      }
    };

    let args = self.make_args()?;
    let end_pos = token.end_pos;

    Ok(ArgValueWithPos {
      value: ArgValue::Condition {
        name,
        args,
        selector,
        inverted,
        conditional_type,
      },
      start_pos,
      end_pos,
    })
  }

  fn get_variable(&self, value: String) -> Option<(String, String)> {
    for node in &self.variables {
      if node.dfrs_name == value {
        let scope = match node.var_type {
          VariableType::Line => "line",
          VariableType::Local => "local",
          VariableType::Game => "unsaved",
          VariableType::Save => "saved",
        };
        return Some((node.df_name.clone(), scope.to_owned()));
      }
    }

    None
  }
}
