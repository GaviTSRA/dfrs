use crate::definitions::ARG_TYPES;
use crate::node::{ParticleCluster, ParticleData, StartNode, UseNode};
use crate::token::Range;
use crate::{
  definitions::ArgType,
  node::{
    ActionNode, ActionType, Arg, ArgValue, ArgValueWithPos, CallNode, ConditionalNode,
    ConditionalType, EventNode, Expression, ExpressionNode, FileNode, FunctionNode,
    FunctionParamNode, ProcessNode, RepeatNode, VariableNode, VariableVariant,
  },
  token::{Keyword, Position, Selector, Token, TokenWithPos, SELECTORS, TYPES},
};

#[derive(Debug)]
pub enum ParserError {
  InvalidToken {
    found: Option<Token>,
    expected: Vec<Token>,
    range: Range,
  },
  UnknownVariable {
    found: String,
    range: Range,
  },
  InvalidCall {
    msg: String,
    range: Range,
  },
  InvalidComplexNumber {
    msg: String,
    range: Range,
  },
  InvalidLocation {
    msg: String,
    range: Range,
  },
  InvalidVector {
    msg: String,
    range: Range,
  },
  InvalidSound {
    msg: String,
    range: Range,
  },
  InvalidPotion {
    msg: String,
    range: Range,
  },
  InvalidParticle {
    msg: String,
    range: Range,
  },
  InvalidItem {
    msg: String,
    range: Range,
  },
  InvalidType {
    found: String,
    range: Range,
  },
  InvalidUse {
    range: Range,
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

  fn advance_err(&mut self) -> Result<TokenWithPos, ParserError> {
    let token = self.advance();

    if token.is_none() {
      let end = match self.tokens.last() {
        Some(token) => token.range.end.clone(),
        None => Position::new(0, 0),
      };

      return Err(ParserError::InvalidToken {
        found: None,
        expected: vec![],
        range: Range::new(end.clone(), end),
      });
    }

    Ok(token.unwrap())
  }

  fn invalid_token(token: TokenWithPos, expected: Vec<Token>) -> ParserError {
    ParserError::InvalidToken {
      found: Some(token.token),
      expected,
      range: token.range,
    }
  }

  fn require_token(&mut self, required_token: Token) -> Result<TokenWithPos, ParserError> {
    let token = self.advance_err()?;
    if token.token == required_token {
      return Ok(token);
    }
    Err(Self::invalid_token(token, vec![required_token]))
  }

  pub fn run(&mut self) -> Result<FileNode, ParserError> {
    self.file()
  }

  fn make_name(&mut self, token: Option<TokenWithPos>) -> Result<String, ParserError> {
    let name_token = match token {
      Some(token) => token,
      None => self.advance_err()?,
    };
    match name_token.token {
      Token::Identifier { value } => Ok(value),
      _ => Err(Self::invalid_token(
        name_token,
        vec![Token::Identifier {
          value: String::from("<any>"),
        }],
      )),
    }
  }

  fn file(&mut self) -> Result<FileNode, ParserError> {
    let mut token = self.advance();
    let mut uses: Vec<UseNode> = vec![];
    let mut events: Vec<EventNode> = vec![];
    let mut functions: Vec<FunctionNode> = vec![];
    let mut processes: Vec<ProcessNode> = vec![];
    let start_pos = Position::new(1, 0);

    while token.is_some() {
      let current = token.unwrap();
      match current.token {
        Token::At => events.push(self.event()?),
        Token::Keyword { value } => match &value {
          Keyword::Function => {
            functions.push(self.function()?);
          }
          Keyword::Process => {
            processes.push(self.process()?);
          }
          Keyword::VarGame => {
            let node = self.variable(VariableVariant::Game, None)?;
            self.variables.push(node);
          }
          Keyword::VarSave => {
            let node = self.variable(VariableVariant::Save, None)?;
            self.variables.push(node);
          }
          Keyword::Use => {
            let node = self.use_statement()?;
            uses.push(node);
          }
          _ => {
            return Err(Self::invalid_token(
              self.current_token.clone().unwrap(),
              vec![
                Token::At,
                Token::Keyword {
                  value: Keyword::Function,
                },
              ],
            ))
          }
        },
        _ => {
          return Err(Self::invalid_token(
            current,
            vec![
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
          ))
        }
      }
      token = self.advance();
      self.variables = self
        .variables
        .clone()
        .into_iter()
        .filter(|var| {
          var.var_variant == VariableVariant::Game || var.var_variant == VariableVariant::Save
        })
        .collect::<Vec<VariableNode>>();
    }

    let end_pos = if !events.is_empty() {
      events.last().unwrap().range.end.clone()
    } else {
      start_pos.clone()
    };
    Ok(FileNode {
      uses,
      events,
      functions,
      processes,
      range: Range::new(start_pos, end_pos),
    })
  }

  fn event(&mut self) -> Result<EventNode, ParserError> {
    let mut expressions: Vec<ExpressionNode> = vec![];
    let start_pos = self.current_token.clone().unwrap().range.end;
    let mut cancelled = false;

    let event = self.make_name(None)?;
    let name_token = self.current_token.clone().unwrap();

    let mut token = self.advance_err()?;
    match token.token {
      Token::ExclamationMark => {
        cancelled = true;
        self.require_token(Token::OpenParenCurly)?;
      }
      Token::OpenParenCurly => {}
      _ => {
        return Err(Self::invalid_token(
          token,
          vec![Token::OpenParenCurly, Token::ExclamationMark],
        ))
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
      range: Range::new(start_pos, token.range.end),
      name_end_pos: name_token.range.end,
      cancelled,
    })
  }

  fn function(&mut self) -> Result<FunctionNode, ParserError> {
    let mut expressions: Vec<ExpressionNode> = vec![];
    let start_pos = self.current_token.clone().unwrap().range.end;

    let dfrs_name = self.make_name(None)?;
    let name_token = self.current_token.clone().unwrap();

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
            return Err(Self::invalid_token(
              token,
              vec![Token::Variable {
                value: "any".into(),
              }],
            ))
          }
        }
        self.require_token(Token::OpenParen)?;
      }
      _ => return Err(Self::invalid_token(token, vec![Token::OpenParen])),
    }

    let mut params: Vec<FunctionParamNode> = vec![];

    loop {
      let token = self.advance_err()?;
      let param_name = match token.token {
        Token::Identifier { value } => value,
        Token::CloseParen => break,
        _ => {
          return Err(Self::invalid_token(
            token,
            vec![
              Token::Identifier {
                value: "any".into(),
              },
              Token::CloseParen,
            ],
          ))
        }
      };

      self.require_token(Token::Colon)?;

      let token = self.advance_err()?;
      let param_type = match token.token {
        Token::Identifier { value } => {
          if TYPES.contains_key(&value.clone()) {
            TYPES.get(&value).unwrap().to_owned()
          } else {
            return Err(ParserError::InvalidType {
              found: value,
              range: token.range,
            });
          }
        }
        _ => {
          return Err(Self::invalid_token(
            token,
            vec![Token::Identifier {
              value: "type".into(),
            }],
          ))
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
              return Err(Self::invalid_token(
                token,
                vec![
                  Token::Comma,
                  Token::CloseParen,
                  Token::Multiply,
                  Token::QuestionMark,
                  Token::Equal,
                ],
              ));
            }
          }
          Token::QuestionMark => {
            if !optional {
              optional = true;
            } else {
              return Err(Self::invalid_token(
                token,
                vec![
                  Token::Multiply,
                  Token::CloseParen,
                  Token::Comma,
                  Token::Equal,
                ],
              ));
            }
          }
          Token::Equal => {
            let token = self.advance_err()?;
            default = Some(match token.token.clone() {
              Token::Number { value } => ArgValueWithPos {
                value: ArgValue::Number { number: value },
                range: token.range,
              },
              Token::Text { value } => ArgValueWithPos {
                value: ArgValue::Text { text: value },
                range: token.range,
              },
              Token::String { value } => ArgValueWithPos {
                value: ArgValue::String { string: value },
                range: token.range,
              },
              Token::Identifier { value } => match value.as_str() {
                "Location" => self.make_location()?,
                "Vector" => self.make_vector()?,
                "Sound" => self.make_sound()?,
                "Potion" => self.make_potion()?,
                _ => return Err(Self::invalid_token(token, vec![])),
              },
              _ => {
                return Err(Self::invalid_token(
                  token,
                  vec![Token::Identifier {
                    value: "any".into(),
                  }],
                ))
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
                return Err(Self::invalid_token(
                  token,
                  vec![Token::Comma, Token::CloseParen],
                ))
              }
            }
          }
          _ => return Err(Self::invalid_token(token, vec![])),
        }
      }

      self.variables.push(VariableNode {
        dfrs_name: param_name.clone(),
        df_name: param_name.clone(),
        action: None,
        var_variant: VariableVariant::Line,
        var_type: None,
        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
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
          return Err(Self::invalid_token(
            token,
            vec![Token::Comma, Token::CloseParen],
          ))
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
      range: Range::new(start_pos, token.range.end),
      name_end_pos: name_token.range.end,
      params,
    })
  }

  fn process(&mut self) -> Result<ProcessNode, ParserError> {
    let mut expressions: Vec<ExpressionNode> = vec![];
    let start_pos = self.current_token.clone().unwrap().range.end;

    let name = self.make_name(None)?;
    let name_token = self.current_token.clone().unwrap();

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
      range: Range::new(start_pos, token.range.end),
      name_end_pos: name_token.range.end,
    })
  }

  fn expression(&mut self) -> Result<ExpressionNode, ParserError> {
    let token = self.current_token.clone().unwrap();
    let node;
    let start_pos = token.range.start.clone();
    let end_pos;

    match token.token.clone() {
      Token::Keyword { value } => match value {
        Keyword::P => {
          let res = self.action(ActionType::Player)?;
          end_pos = res.range.end.clone();
          node = Expression::Action { node: res };
        }
        Keyword::E => {
          let res = self.action(ActionType::Entity)?;
          end_pos = res.range.end.clone();
          node = Expression::Action { node: res };
        }
        Keyword::G => {
          let res = self.action(ActionType::Game)?;
          end_pos = res.range.end.clone();
          node = Expression::Action { node: res };
        }
        Keyword::V => {
          let res = self.action(ActionType::Variable)?;
          end_pos = res.range.end.clone();
          node = Expression::Action { node: res };
        }
        Keyword::C => {
          let res = self.action(ActionType::Control)?;
          end_pos = res.range.end.clone();
          node = Expression::Action { node: res };
        }
        Keyword::S => {
          let res = self.action(ActionType::Select)?;
          end_pos = res.range.end.clone();
          node = Expression::Action { node: res };
        }
        Keyword::IfP => {
          let res = self.conditional(ConditionalType::Player)?;
          end_pos = res.range.end.clone();
          node = Expression::Conditional { node: res };
        }
        Keyword::IfE => {
          let res = self.conditional(ConditionalType::Entity)?;
          end_pos = res.range.end.clone();
          node = Expression::Conditional { node: res };
        }
        Keyword::IfG => {
          let res = self.conditional(ConditionalType::Game)?;
          end_pos = res.range.end.clone();
          node = Expression::Conditional { node: res };
        }
        Keyword::IfV => {
          let res = self.conditional(ConditionalType::Variable)?;
          end_pos = res.range.end.clone();
          node = Expression::Conditional { node: res };
        }
        Keyword::VarLine => {
          let res = self.variable(VariableVariant::Line, None)?;
          end_pos = res.range.end.clone();
          node = Expression::Variable { node: res }
        }
        Keyword::VarLocal => {
          let res = self.variable(VariableVariant::Local, None)?;
          end_pos = res.range.end.clone();
          node = Expression::Variable { node: res }
        }
        Keyword::Start => {
          let res = self.start()?;
          end_pos = res.range.end.clone();
          node = Expression::Start { node: res }
        }
        Keyword::Repeat => {
          let res = self.repeat()?;
          end_pos = res.range.end.clone();
          node = Expression::Repeat { node: res }
        }
        _ => {
          return Err(Self::invalid_token(
            token,
            vec![
              Token::Keyword { value: Keyword::E },
              Token::Keyword { value: Keyword::P },
            ],
          ))
        }
      },
      Token::Identifier { value } => {
        if let Some((variable, scope, _var_type)) = self.get_variable(value.clone()) {
          let var_variant = match scope.as_str() {
            "line" => VariableVariant::Line,
            "local" => VariableVariant::Local,
            "unsaved" => VariableVariant::Game,
            "saved" => VariableVariant::Save,
            _ => unreachable!("unknown type {scope}"),
          };

          let res = self.variable(var_variant, Some(variable))?;
          end_pos = res.range.end.clone();
          node = Expression::Variable { node: res }
        } else if let Some(token) = self.peak() {
          if token.token == Token::OpenParen {
            let res = self.call(value, false)?;
            end_pos = res.range.end.clone();
            node = Expression::Call { node: res }
          } else {
            return Err(Self::invalid_token(token, vec![Token::OpenParen]));
          }
        } else {
          return Err(Self::invalid_token(
            token,
            vec![
              Token::Keyword { value: Keyword::E },
              Token::Keyword { value: Keyword::P },
            ],
          ));
        }
      }
      Token::String { value } => {
        if let Some(token) = self.peak() {
          if token.token == Token::OpenParen {
            let res = self.call(value, true)?;
            end_pos = res.range.end.clone();
            node = Expression::Call { node: res }
          } else {
            return Err(Self::invalid_token(token, vec![Token::OpenParen]));
          }
        } else {
          return Err(Self::invalid_token(
            token,
            vec![
              Token::Keyword { value: Keyword::E },
              Token::Keyword { value: Keyword::P },
            ],
          ));
        }
      }

      _ => {
        return Err(Self::invalid_token(
          token,
          vec![
            Token::Keyword { value: Keyword::E },
            Token::Keyword { value: Keyword::P },
          ],
        ))
      }
    }

    Ok(ExpressionNode {
      node: node.clone(),
      range: Range::new(start_pos, end_pos),
    })
  }

  fn action(&mut self, action_type: ActionType) -> Result<ActionNode, ParserError> {
    let mut selector = Selector::Default;
    let mut implicit_selector = true;
    let mut token = self.advance_err()?;
    let mut start_pos = token.range.start.clone();
    start_pos.col += 1;

    match token.token {
      Token::Colon => {
        if action_type == ActionType::Variable {
          return Err(Self::invalid_token(token, vec![Token::Dot]));
        }
        token = self.advance_err()?;
        match token.token {
          Token::Identifier { value } => {
            if let Some(value) = SELECTORS.get(&value).cloned() {
              selector = value;
              implicit_selector = false;
              self.require_token(Token::Dot)?;
            } else {
              return Err(Self::invalid_token(
                self.current_token.clone().unwrap(),
                vec![Token::Identifier {
                  value: "<selector>".to_string(),
                }],
              ));
            }
          }
          _ => {
            return Err(Self::invalid_token(
              token,
              vec![Token::Identifier {
                value: "<selector>".to_string(),
              }],
            ))
          }
        }
      }
      Token::Dot => {}
      _ => return Err(Self::invalid_token(token, vec![])),
    }

    let name = self.make_name(None)?;
    let end_pos = self.current_token.clone().unwrap().range.end;

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
      range: Range::new(start_pos, end_pos),
      selector_range: Range::new(selector_start_pos, selector_end_pos),
      action: None,
    })
  }

  fn conditional(
    &mut self,
    conditional_type: ConditionalType,
  ) -> Result<ConditionalNode, ParserError> {
    let mut token = self.advance_err()?;
    let mut selector = Selector::Default;
    let start_pos = token.range.start.clone();
    let mut selector_start_pos = None;
    let mut selector_end_pos = None;
    let mut inverted = false;

    if token.token == Token::ExclamationMark {
      inverted = true;
      token = self.advance_err()?;
    }

    if let Token::Identifier { value } = &token.token {
      if let Some(value) = SELECTORS.get(value).cloned() {
        selector = value;
        selector_start_pos = Some(token.range.start);
        selector_end_pos = Some(token.range.end);
        self.require_token(Token::Colon)?;
        token = self.advance_err()?;
      }
    }
    let name = self.make_name(Some(token.clone()))?;

    let args = self.make_args()?;
    let end_pos = token.range.end;

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
    if let Some(token) = self.peak() {
      if let Token::Keyword { value } = token.token {
        if value == Keyword::Else {
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
      }
    }

    let mut selector_range = None;
    if let Some(start) = selector_start_pos {
      if let Some(end) = selector_end_pos {
        selector_range = Some(Range::new(start, end))
      }
    }

    Ok(ConditionalNode {
      conditional_type,
      selector,
      name,
      args,
      range: Range::new(start_pos, end_pos),
      selector_range,
      expressions,
      else_expressions,
      inverted,
      action: None,
    })
  }

  fn call(&mut self, name: String, is_unsafe_call: bool) -> Result<CallNode, ParserError> {
    let start_pos = self.current_token.clone().unwrap().range.start;
    let mut args = self.make_args()?;

    for arg in args.iter_mut() {
      arg.index -= 1;
    }
    self.require_token(Token::Semicolon)?;

    let end_pos = self.current_token.clone().unwrap().range.end;

    Ok(CallNode {
      name,
      args,
      range: Range::new(start_pos, end_pos),
      is_unsafe_call,
    })
  }

  fn start(&mut self) -> Result<StartNode, ParserError> {
    let range = self.current_token.clone().unwrap().range;
    let mut args = self.make_args()?;

    if args.is_empty() {
      return Err(ParserError::InvalidCall {
        range,
        msg: "Missing process name".into(),
      });
    }
    let name_arg = args.remove(0);
    let name = match name_arg.value {
      ArgValue::Text { text } => text,
      _ => {
        return Err(ParserError::InvalidCall {
          range,
          msg: "Invalid process name param type".into(),
        })
      }
    };
    for arg in args.iter_mut() {
      arg.index -= 1;
    }
    self.require_token(Token::Semicolon)?;

    let end_pos = self.current_token.clone().unwrap().range.end;

    Ok(StartNode {
      name,
      args,
      range: Range::new(range.start, end_pos),
    })
  }

  fn repeat(&mut self) -> Result<RepeatNode, ParserError> {
    let mut token = self.advance_err()?;
    let start_pos = token.range.start.clone();

    let name = self.make_name(Some(token.clone()))?;

    let args = self.make_args()?;
    let end_pos = token.range.end;

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
      range: Range::new(start_pos, end_pos),
      expressions,
      action: None,
    })
  }

  fn variable(
    &mut self,
    var_variant: VariableVariant,
    name: Option<String>,
  ) -> Result<VariableNode, ParserError> {
    let start_pos = self.current_token.clone().unwrap().range.start;
    let end_pos = start_pos.clone();
    let mut var_type = ArgType::ANY;

    let dfrs_name = if let Some(name) = name {
      name
    } else {
      self.make_name(None)?
    };

    let mut token = self.advance_err()?;
    let mut df_name = dfrs_name.clone();
    if token.token == Token::Tilde {
      token = self.advance_err()?;
      df_name = match token.token {
        Token::Variable { value } => value,
        _ => {
          return Err(Self::invalid_token(
            token,
            vec![Token::Variable {
              value: "any".into(),
            }],
          ))
        }
      };
      token = self.advance_err()?;
    }
    if token.token == Token::Colon {
      token = self.advance_err()?;
      var_type = match token.token {
        Token::Identifier { value } => {
          if ARG_TYPES.contains_key(&value.clone()) {
            ARG_TYPES.get(&value).unwrap().to_owned()
          } else {
            return Err(ParserError::InvalidType {
              found: value,
              range: token.range,
            });
          }
        }
        _ => {
          return Err(Self::invalid_token(
            token,
            vec![Token::Identifier {
              value: "type".into(),
            }],
          ))
        }
      };
      token = self.advance_err()?;
    }
    match token.token {
      Token::Equal => {}
      Token::Semicolon => {
        let node = VariableNode {
          dfrs_name: dfrs_name.clone(),
          df_name,
          var_variant,
          var_type: Some(var_type),
          action: None,
          range: Range::new(start_pos, end_pos),
        };
        self.variables.push(node.clone());
        return Ok(node);
      }
      _ => {
        return Err(Self::invalid_token(
          token,
          vec![Token::Equal, Token::Semicolon],
        ))
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
          return Err(Self::invalid_token(
            self.current_token.clone().unwrap(),
            vec![Token::Keyword { value: Keyword::P }],
          ))
        }
      },
      _ => {
        return Err(Self::invalid_token(
          token,
          vec![Token::Keyword { value: Keyword::P }],
        ))
      }
    };

    let node = VariableNode {
      dfrs_name,
      df_name,
      action: Some(action),
      var_variant,
      var_type: None,
      range: Range::new(start_pos, end_pos),
    };
    self.variables.push(node.clone());
    Ok(node)
  }

  fn use_statement(&mut self) -> Result<UseNode, ParserError> {
    let token = self.advance_err()?;
    let res = match token.token {
      Token::Text { value } => UseNode {
        file: value,
        range: token.range,
      },
      _ => return Err(ParserError::InvalidUse { range: token.range }),
    };
    self.require_token(Token::Semicolon)?;
    Ok(res)
  }

  fn make_params(&mut self) -> Result<Vec<ArgValueWithPos>, ParserError> {
    self.require_token(Token::OpenParen)?;

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
            comma_pos = self.current_token.clone().unwrap().range.start;
          }
          Token::CloseParen => break,
          _ => {
            return Err(Self::invalid_token(
              token,
              vec![Token::Comma, Token::CloseParen],
            ))
          }
        }
      } else if could_be_tag {
        could_be_tag = false;
        match token.token.clone() {
          Token::Equal => {
            is_tag = true;
          }
          _ => {
            if let Some((var, scope, var_type)) = self.get_variable(tag_name.clone()) {
              params.push(ArgValueWithPos {
                value: ArgValue::Variable {
                  name: var,
                  scope,
                  var_type,
                },
                range: Range::new(
                  self.current_token.clone().unwrap().range.start,
                  self.current_token.clone().unwrap().range.end,
                ),
              });
              is_value = true;
              self.token_index -= 1;
            } else {
              return Err(ParserError::UnknownVariable {
                found: tag_name.clone(),
                range: Range::new(tag_start_pos, tag_end_pos),
              });
            }
          }
        }
      } else if is_tag {
        is_tag = false;
        match token.token.clone() {
          Token::String { value } => {
            let data = Box::new(ArgValue::String { string: value });
            params.push(ArgValueWithPos {
              value: ArgValue::Tag {
                tag: tag_name.clone(),
                value: data,
                definition: None,
                name_range: Range::new(tag_start_pos.clone(), tag_end_pos.clone()),
                value_range: token.range.clone(),
              },
              range: Range::new(tag_start_pos.clone(), token.range.end),
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
                name_range: Range::new(tag_start_pos.clone(), tag_end_pos.clone()),
                value_range: token.range.clone(),
              },
              range: Range::new(tag_start_pos.clone(), token.range.end),
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
                name_range: Range::new(tag_start_pos.clone(), tag_end_pos.clone()),
                value_range: token.range.clone(),
              },
              range: Range::new(tag_start_pos.clone(), token.range.end),
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
                  name_range: Range::new(tag_start_pos.clone(), tag_end_pos.clone()),
                  value_range: token.range.clone(),
                },
                range: Range::new(tag_start_pos.clone(), token.range.end),
              });
              is_value = true;
            }
            _ => {
              return Err(Self::invalid_token(
                token,
                vec![
                  Token::String {
                    value: "<any>".into(),
                  },
                  Token::Text {
                    value: "<any>".into(),
                  },
                ],
              ))
            }
          },
          _ => {
            return Err(Self::invalid_token(
              token,
              vec![
                Token::String {
                  value: "<any>".into(),
                },
                Token::Text {
                  value: "<any>".into(),
                },
              ],
            ))
          }
        }
      } else if is_game_value {
        let mut selector = Selector::Default;
        let mut selector_end_pos = token.range.start.clone();
        let start_pos = token.range.start.clone();

        if let Token::Identifier { value } = token.token.clone() {
          if let Some(value) = SELECTORS.get(&value).cloned() {
            selector = value;
            token = self.advance_err()?;
            if token.token != Token::Colon {
              return Err(Self::invalid_token(token, vec![Token::Colon]));
            }
            selector_end_pos = token.range.end;
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
              range: Range::new(start_pos, token.range.end.clone()),
            });
            is_value = true;
            is_game_value = false;
          }
          _ => {
            return Err(Self::invalid_token(
              token,
              vec![
                Token::Identifier {
                  value: "<any>".into(),
                },
                Token::Identifier {
                  value: "<selector>".into(),
                },
              ],
            ))
          }
        }
      } else {
        match token.token.clone() {
          Token::Number { value } => {
            params.push(ArgValueWithPos {
              value: ArgValue::Number { number: value },
              range: token.range,
            });
            is_value = true;
          }
          Token::Text { value } => {
            params.push(ArgValueWithPos {
              value: ArgValue::Text { text: value },
              range: token.range,
            });
            is_value = true;
          }
          Token::String { value } => {
            params.push(ArgValueWithPos {
              value: ArgValue::String { string: value },
              range: token.range,
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
                range: self.current_token.clone().unwrap().range,
              });
              is_value = true;
            }
            _ => {
              could_be_tag = true;
              tag_name = value;
              tag_start_pos = self.current_token.clone().unwrap().range.start;
              tag_end_pos = self.current_token.clone().unwrap().range.end;
            }
          },
          Token::Dollar => is_game_value = true,
          Token::Keyword { value } => {
            let arg = match value {
              Keyword::IfP => self.conditional_arg(ConditionalType::Player)?,
              Keyword::IfE => self.conditional_arg(ConditionalType::Entity)?,
              Keyword::IfG => self.conditional_arg(ConditionalType::Game)?,
              Keyword::IfV => self.conditional_arg(ConditionalType::Variable)?,
              _ => return Err(Self::invalid_token(token, expected)),
            };
            params.push(arg);
            is_value = true;
          }
          Token::CloseParen => break,
          _ => return Err(Self::invalid_token(token, expected)),
        }
      }
    }
    if !params.is_empty() && !is_value {
      return Err(Self::invalid_token(
        TokenWithPos::new(Token::Comma, Range::new(comma_pos.clone(), comma_pos)),
        expected,
      ));
    }
    Ok(params)
  }

  fn make_args(&mut self) -> Result<Vec<Arg>, ParserError> {
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
        ArgValue::GameValue { .. } => ArgType::ANY,
        ArgValue::Condition { .. } => ArgType::CONDITION,
      };
      args.push(Arg {
        value: param.value,
        index: i as i32,
        arg_type,
        range: param.range,
      });
    }
    Ok(args)
  }

  fn make_complex_number(&mut self) -> Result<ArgValueWithPos, ParserError> {
    let params = self.make_params()?;

    if params.is_empty() {
      return Err(ParserError::InvalidComplexNumber {
        msg: "Not enough arguments".into(),
        range: self.current_token.clone().unwrap().range,
      });
    }
    let number = match params[0].value.clone() {
      ArgValue::Text { text } => text,
      _ => {
        return Err(ParserError::InvalidComplexNumber {
          msg: "Invalid value, should be text".into(),
          range: self.current_token.clone().unwrap().range,
        })
      }
    };
    if params.len() > 1 {
      return Err(ParserError::InvalidComplexNumber {
        msg: "Too many arguments".into(),
        range: self.current_token.clone().unwrap().range,
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::ComplexNumber { number },
      range: self.current_token.clone().unwrap().range,
    })
  }

  fn make_location(&mut self) -> Result<ArgValueWithPos, ParserError> {
    let mut pitch = None;
    let mut yaw = None;
    let start_pos = self.current_token.clone().unwrap().range.start;
    let loc_params = self.make_params()?;

    if loc_params.len() < 3 {
      return Err(ParserError::InvalidLocation {
        range: self.current_token.clone().unwrap().range,
        msg: "Not enough arguments".into(),
      });
    }
    let x = match loc_params[0].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidLocation {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid x coordinate".into(),
        })
      }
    };
    let y = match loc_params[1].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidLocation {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid y coordinate".into(),
        })
      }
    };
    let z = match loc_params[2].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidLocation {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid z coordinate".into(),
        })
      }
    };
    if loc_params.len() >= 4 {
      match loc_params[3].value {
        ArgValue::Number { number } => pitch = Some(number),
        _ => {
          return Err(ParserError::InvalidLocation {
            range: self.current_token.clone().unwrap().range,
            msg: "Invalid pitch".into(),
          })
        }
      }
    }
    if loc_params.len() == 5 {
      match loc_params[4].value {
        ArgValue::Number { number } => yaw = Some(number),
        _ => {
          return Err(ParserError::InvalidLocation {
            range: self.current_token.clone().unwrap().range,
            msg: "Invalid yaw".into(),
          })
        }
      }
    }
    if loc_params.len() > 5 {
      return Err(ParserError::InvalidLocation {
        range: self.current_token.clone().unwrap().range,
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
      range: Range::new(start_pos, self.current_token.clone().unwrap().range.end),
    })
  }

  fn make_vector(&mut self) -> Result<ArgValueWithPos, ParserError> {
    let start_pos = self.current_token.clone().unwrap().range.start;
    let vec_params = self.make_params()?;

    if vec_params.len() < 3 {
      return Err(ParserError::InvalidVector {
        range: self.current_token.clone().unwrap().range,
        msg: "Not enough arguments".into(),
      });
    }
    let x = match vec_params[0].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidVector {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid x coordinate".into(),
        })
      }
    };
    let y = match vec_params[1].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidVector {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid y coordinate".into(),
        })
      }
    };
    let z = match vec_params[2].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidVector {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid z coordinate".into(),
        })
      }
    };
    if vec_params.len() > 3 {
      return Err(ParserError::InvalidVector {
        range: self.current_token.clone().unwrap().range,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::Vector { x, y, z },
      range: Range::new(start_pos, self.current_token.clone().unwrap().range.end),
    })
  }

  fn make_sound(&mut self) -> Result<ArgValueWithPos, ParserError> {
    let start_pos = self.current_token.clone().unwrap().range.start;
    let sound_params = self.make_params()?;

    if sound_params.len() < 3 {
      return Err(ParserError::InvalidSound {
        range: self.current_token.clone().unwrap().range,
        msg: "Not enough arguments".into(),
      });
    }
    let sound = match &sound_params[0].value {
      ArgValue::String { string } => string.clone(),
      ArgValue::Text { text } => text.clone(),
      _ => {
        return Err(ParserError::InvalidSound {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid sound type".into(),
        })
      }
    };
    let volume = match sound_params[1].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidSound {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid volume".into(),
        })
      }
    };
    let pitch = match sound_params[2].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidSound {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid pitch".into(),
        })
      }
    };
    let mut variant = None;
    if sound_params.len() >= 4 {
      variant = match &sound_params[3].value {
        ArgValue::String { string } => Some(string.clone()),
        _ => {
          return Err(ParserError::InvalidSound {
            range: self.current_token.clone().unwrap().range,
            msg: "Invalid variant".into(),
          })
        }
      };
    }

    if sound_params.len() > 4 {
      return Err(ParserError::InvalidSound {
        range: self.current_token.clone().unwrap().range,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::Sound {
        sound,
        variant,
        volume,
        pitch,
      },
      range: Range::new(start_pos, self.current_token.clone().unwrap().range.end),
    })
  }

  fn make_potion(&mut self) -> Result<ArgValueWithPos, ParserError> {
    let start_pos = self.current_token.clone().unwrap().range.start;
    let potion_params = self.make_params()?;

    if potion_params.len() < 3 {
      return Err(ParserError::InvalidPotion {
        range: self.current_token.clone().unwrap().range,
        msg: "Not enough arguments".into(),
      });
    }
    let potion = match &potion_params[0].value {
      ArgValue::String { string } => string.clone(),
      ArgValue::Text { text } => text.clone(),
      _ => {
        return Err(ParserError::InvalidPotion {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid potion type".into(),
        })
      }
    };
    let amplifier = match potion_params[1].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidPotion {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid amplifier".into(),
        })
      }
    };
    let duration = match potion_params[2].value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidPotion {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid duration".into(),
        })
      }
    };
    if potion_params.len() > 3 {
      return Err(ParserError::InvalidPotion {
        range: self.current_token.clone().unwrap().range,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::Potion {
        potion,
        amplifier,
        duration,
      },
      range: Range::new(start_pos, self.current_token.clone().unwrap().range.end),
    })
  }

  fn make_particle(&mut self) -> Result<ArgValueWithPos, ParserError> {
    let start_pos = self.current_token.clone().unwrap().range.start;
    let mut particle_params = self.make_params()?;

    if particle_params.len() < 4 {
      return Err(ParserError::InvalidParticle {
        range: self.current_token.clone().unwrap().range,
        msg: "Not enough arguments".into(),
      });
    }
    let particle = match particle_params.remove(0).value {
      ArgValue::String { string } => string.clone(),
      ArgValue::Text { text } => text.clone(),
      _ => {
        return Err(ParserError::InvalidParticle {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid particle type".into(),
        })
      }
    };
    let amount = match particle_params.remove(0).value {
      ArgValue::Number { number } => number as i32,
      _ => {
        return Err(ParserError::InvalidParticle {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid particle amount".into(),
        })
      }
    };
    let horizontal = match particle_params.remove(0).value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidParticle {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid particle horizontal spread".into(),
        })
      }
    };
    let vertical = match particle_params.remove(0).value {
      ArgValue::Number { number } => number,
      _ => {
        return Err(ParserError::InvalidParticle {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid particle vertical spread".into(),
        })
      }
    };

    // TODO values dont properly compile if supported options are unset
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
              x = Some(*x2);
              y = Some(*y2);
              z = Some(*z2);
            }
            _ => {
              return Err(ParserError::InvalidParticle {
                range: self.current_token.clone().unwrap().range,
                msg: "Expected motion to be vector".into(),
              })
            }
          },
          "motionVariation" => match value.as_ref() {
            ArgValue::Number { number } => motion_variation = Some(*number as i32),
            _ => {
              return Err(ParserError::InvalidParticle {
                range: self.current_token.clone().unwrap().range,
                msg: "Expected motion variation to be number".into(),
              })
            }
          },
          "rgb" => match value.as_ref() {
            ArgValue::Number { number } => rgb = Some(*number as i32),
            _ => {
              return Err(ParserError::InvalidParticle {
                range: self.current_token.clone().unwrap().range,
                msg: "Expected rgb to be number".into(),
              })
            }
          },
          "rgbFade" => match value.as_ref() {
            ArgValue::Number { number } => rgb_fade = Some(*number as i32),
            _ => {
              return Err(ParserError::InvalidParticle {
                range: self.current_token.clone().unwrap().range,
                msg: "Expected rgb fade to be number".into(),
              })
            }
          },
          "colorVariation" => match value.as_ref() {
            ArgValue::Number { number } => color_variation = Some(*number as i32),
            _ => {
              return Err(ParserError::InvalidParticle {
                range: self.current_token.clone().unwrap().range,
                msg: "Expected color variation to be number".into(),
              })
            }
          },
          "material" => match value.as_ref() {
            ArgValue::Text { text } => material = Some(text.clone()),
            _ => {
              return Err(ParserError::InvalidParticle {
                range: self.current_token.clone().unwrap().range,
                msg: "Expected material to be text".into(),
              })
            }
          },
          "size" => match value.as_ref() {
            ArgValue::Number { number } => size = Some(*number),
            _ => {
              return Err(ParserError::InvalidParticle {
                range: self.current_token.clone().unwrap().range,
                msg: "Expected size to be number".into(),
              })
            }
          },
          "sizeVariation" => match value.as_ref() {
            ArgValue::Number { number } => size_variation = Some(*number as i32),
            _ => {
              return Err(ParserError::InvalidParticle {
                range: self.current_token.clone().unwrap().range,
                msg: "Expected size variation to be number".into(),
              })
            }
          },
          "roll" => match value.as_ref() {
            ArgValue::Number { number } => roll = Some(*number),
            _ => {
              return Err(ParserError::InvalidParticle {
                range: self.current_token.clone().unwrap().range,
                msg: "Expected roll to be number".into(),
              })
            }
          },
          _ => {
            return Err(ParserError::InvalidParticle {
              range: self.current_token.clone().unwrap().range,
              msg: "Unknown tag".into(),
            })
          }
        },
        _ => {
          return Err(ParserError::InvalidParticle {
            range: self.current_token.clone().unwrap().range,
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
      range: Range::new(start_pos, self.current_token.clone().unwrap().range.end),
    })
  }

  fn make_item(&mut self) -> Result<ArgValueWithPos, ParserError> {
    let start_pos = self.current_token.clone().unwrap().range.start;
    let item_params = self.make_params()?;

    if item_params.is_empty() {
      return Err(ParserError::InvalidItem {
        range: self.current_token.clone().unwrap().range,
        msg: "Not enough arguments".into(),
      });
    }
    let item = match &item_params[0].value {
      ArgValue::String { string } => string.clone(),
      ArgValue::Text { text } => text.clone(),
      ArgValue::Tag { .. } => {
        return self.make_complex_item(start_pos, item_params);
      }
      _ => {
        return Err(ParserError::InvalidItem {
          range: self.current_token.clone().unwrap().range,
          msg: "Invalid item arg type".into(),
        })
      }
    };
    if item_params.len() > 1 {
      return Err(ParserError::InvalidItem {
        range: self.current_token.clone().unwrap().range,
        msg: "Too many arguments".into(),
      });
    }
    Ok(ArgValueWithPos {
      value: ArgValue::Item { item },
      range: Range::new(start_pos, self.current_token.clone().unwrap().range.end),
    })
  }

  fn make_complex_item(
    &self,
    start_pos: Position,
    params: Vec<ArgValueWithPos>,
  ) -> Result<ArgValueWithPos, ParserError> {
    let mut id: Option<String> = None;
    let mut count: f32 = 1.0;
    let mut components: Vec<String> = vec![];
    let mut other: Option<String> = None;

    for param in params {
      match param.value {
        ArgValue::Tag {
          tag,
          value,
          name_range,
          value_range,
          ..
        } => match tag.as_str() {
          "id" => match *value {
            ArgValue::String { string } => id = Some(string),
            _ => {
              return Err(ParserError::InvalidItem {
                msg: "Invalid id type".into(),
                range: value_range,
              })
            }
          },
          "count" => match *value {
            ArgValue::Number { number } => count = number,
            _ => {
              return Err(ParserError::InvalidItem {
                msg: "Invalid count type".into(),
                range: value_range,
              })
            }
          },
          "other" => match *value {
            ArgValue::String { string } => other = Some(string),
            _ => {
              return Err(ParserError::InvalidItem {
                msg: "Invalid other type".into(),
                range: value_range,
              })
            }
          },
          _ => {
            return Err(ParserError::InvalidItem {
              msg: "Unknown item property".into(),
              range: name_range,
            })
          }
        },
        _ => {
          return Err(ParserError::InvalidItem {
            msg: "Unexpected value".into(),
            range: self.current_token.clone().unwrap().range,
          })
        }
      }
    }

    if id.is_none() {
      return Err(ParserError::InvalidItem {
        msg: "Missing id property".into(),
        range: Range::new(start_pos, self.current_token.clone().unwrap().range.end),
      });
    }

    let mut component_data = components.join(",");
    if let Some(other) = other {
      if component_data.len() > 0 {
        component_data.push(',');
      }
      component_data.push_str(other.as_str());
    }

    let item = format!(
      "{{id:\"{}\",count:{count},components:{{{}}}}}",
      id.unwrap(),
      component_data
    );

    println!("{item}");

    Ok(ArgValueWithPos {
      value: ArgValue::Item { item },
      range: Range::new(start_pos, self.current_token.clone().unwrap().range.end),
    })
  }

  fn conditional_arg(
    &mut self,
    conditional_type: ConditionalType,
  ) -> Result<ArgValueWithPos, ParserError> {
    let mut token = self.advance_err()?;
    let mut selector = Selector::Default;
    let start_pos = token.range.start.clone();
    let mut inverted = false;

    if token.token == Token::ExclamationMark {
      inverted = true;
      token = self.advance_err()?;
    }

    if let Token::Identifier { value } = &token.token {
      if let Some(value) = SELECTORS.get(value).cloned() {
        selector = value;
        self.require_token(Token::Colon)?;
        token = self.advance_err()?;
      }
    }
    let name = match token.token {
      Token::Identifier { value } => value,
      _ => {
        return Err(Self::invalid_token(
          token,
          vec![Token::Identifier {
            value: "any".into(),
          }],
        ))
      }
    };

    let args = self.make_args()?;
    let end_pos = token.range.end;

    Ok(ArgValueWithPos {
      value: ArgValue::Condition {
        name,
        args,
        selector,
        inverted,
        conditional_type,
      },
      range: Range::new(start_pos, end_pos),
    })
  }

  fn get_variable(&self, value: String) -> Option<(String, String, Option<ArgType>)> {
    for node in &self.variables {
      if node.dfrs_name == value {
        let scope = match node.var_variant {
          VariableVariant::Line => "line",
          VariableVariant::Local => "local",
          VariableVariant::Game => "unsaved",
          VariableVariant::Save => "saved",
        };
        return Some((
          node.df_name.clone(),
          scope.to_owned(),
          node.var_type.clone(),
        ));
      }
    }

    None
  }
}
