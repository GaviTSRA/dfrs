use crate::{definitions::ArgType, node::{ActionNode, ActionType, Arg, ArgValue, ArgValueWithPos, ConditionalNode, ConditionalType, EventNode, Expression, ExpressionNode, FileNode, FunctionNode, FunctionParamNode, VariableNode, VariableType}, token::{Keyword, Position, Selector, Token, TokenWithPos, SELECTORS, TYPES}};

#[derive(Debug)]
pub enum ParseError {
    InvalidToken { found: Option<TokenWithPos>, expected: Vec<Token> },
    UnknownVariable { found: String, start_pos: Position, end_pos: Position },
    InvalidLocation { pos: Position, msg: String },
    InvalidVector { pos: Position, msg: String },
    InvalidSound { pos: Position, msg: String },
    InvalidPotion { pos: Position, msg: String },
    InvalidType { found: Option<TokenWithPos>, start_pos: Position }
}

pub struct Parser {
    tokens: Vec<TokenWithPos>,
    token_index: i32,
    current_token: Option<TokenWithPos>,
    variables: Vec<VariableNode>,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithPos>) -> Parser {
        Parser { tokens, token_index: -1, current_token: None, variables: vec![] }
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
            return Err(ParseError::InvalidToken { found: None, expected: vec![] })
        }

        Ok(token.unwrap())
    }

    fn require_token(&mut self, required_token: Token) -> Result<TokenWithPos, ParseError> {
        let token = self.advance_err()?;
        if token.token == required_token {
            return Ok(token);
        }
        Err(ParseError::InvalidToken { found: Some(token), expected: vec![required_token] })
    }

    pub fn run(&mut self) -> Result<FileNode, ParseError> {
        self.file()
    }

    fn file(&mut self) -> Result<FileNode, ParseError> {
        let mut token = self.advance();
        let mut events: Vec<EventNode> = vec![];
        let mut functions: Vec<FunctionNode> = vec![];
        let start_pos = Position::new(1, 0);

        while token.is_some() {
            match token.clone().unwrap().token {
                Token::At => events.push(self.event()?),
                Token::Keyword { value } => {
                    match value {
                        Keyword::Function => {
                            functions.push(self.function()?);
                        }
                        Keyword::VarGame => {
                            let node = self.variable(VariableType::Game)?;
                            self.variables.push(node);
                        }
                        Keyword::VarSave => {
                            let node = self.variable(VariableType::Save)?;
                            self.variables.push(node);
                        }
                        _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::At, Token::Keyword { value: Keyword::Function }] })
                    }
                }
                _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::At, Token::Keyword { value: Keyword::Function }, Token::Keyword { value: Keyword::VarGame }, Token::Keyword { value: Keyword::VarSave }] })
            }
            token = self.advance();
            self.variables = self.variables.clone().into_iter().filter(|var| var.var_type == VariableType::Game || var.var_type == VariableType::Save).collect::<Vec<VariableNode>>();
        }
        
        let end_pos = if !events.is_empty() {
            events.last().unwrap().end_pos.clone()
        } else {
            start_pos.clone()
        };
        Ok(FileNode { events, functions, start_pos, end_pos })
    }

    fn event(&mut self) -> Result<EventNode, ParseError> {
        let mut expressions: Vec<ExpressionNode> = vec![];
        let start_pos = self.current_token.clone().unwrap().end_pos;
        let mut cancelled = false;

        let name_token = self.advance_err()?;

        let event = match name_token.token {
            Token::Identifier { value } => value,
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: String::from("<any>")}] })
        };

        let mut token = self.advance_err()?;
        match token.token {
            Token::ExclamationMark => {
                cancelled = true;
                self.require_token(Token::OpenParenCurly)?;
            }
            Token::OpenParenCurly => {}
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::OpenParenCurly, Token::ExclamationMark] })
        }

        loop {
            token = self.advance_err()?;
            match token.token {
                Token::CloseParenCurly => break,
                _ => expressions.push(self.expression()?)
            }
        }

        Ok(EventNode { event_type: None, event, expressions, start_pos, name_end_pos: name_token.end_pos, end_pos: token.end_pos, cancelled })
    }

    fn function(&mut self) -> Result<FunctionNode, ParseError> {
        let mut expressions: Vec<ExpressionNode> = vec![];
        let start_pos = self.current_token.clone().unwrap().end_pos;

        let name_token = self.advance_err()?;
        let name = match name_token.token {
            Token::Identifier { value } => value,
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: String::from("<any>")}] })
        };

        self.require_token(Token::OpenParen)?;
        let mut params: Vec<FunctionParamNode> = vec![];

        loop {
            let token = self.advance_err()?;
            let param_name = match token.token {
                Token::Identifier { value } => value,
                Token::CloseParen => break,
                _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: "any".into() }, Token::CloseParen] })
            };

            let mut optional = false;
            let mut multiple = false;
            let mut token;
            loop {
                token = self.advance_err()?;
                match token.token {
                    Token::Colon => break,
                    Token::Multiply => {
                        if !multiple {
                            multiple = true;
                        } else {
                            return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![] })
                        }
                    }
                    Token::QuestionMark => {
                        if !optional {
                            optional = true;
                        } else {
                            return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![] })
                        }
                    }
                    _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![] })
                }
            }

            let token = self.advance_err()?;
            let param_type = match token.token {
                Token::Identifier { value } => {
                    if TYPES.contains_key(&value.clone()) {
                        TYPES.get(&value).unwrap().to_owned()
                    } else {
                        return Err(ParseError::InvalidType { found: self.current_token.clone(), start_pos: token.start_pos })
                    }
                }
                _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: "type".into() }] })
            };

            let mut default = None;
            let token = self.advance_err()?;
            match token.token {
                Token::Equal => {
                    let token = self.advance_err()?;
                    default = Some(match token.token.clone() {
                        Token::Number { value } => {
                            ArgValueWithPos {
                                value: ArgValue::Number { number: value },
                                start_pos: token.start_pos,
                                end_pos: token.end_pos
                            }
                        }
                        Token::Text { value } => {
                            ArgValueWithPos {
                                value: ArgValue::Text { text: value },
                                start_pos: token.start_pos,
                                end_pos: token.end_pos
                            }
                        }
                        Token::String { value } => {
                            ArgValueWithPos {
                                value: ArgValue::String { string: value },
                                start_pos: token.start_pos,
                                end_pos: token.end_pos
                            }
                        }
                        Token::Identifier { value }  => {
                            match value.as_str() {
                                "Location" => {
                                    self.make_location()?
                                }
                                "Vector" => {
                                    self.make_vector()?
                                }
                                "Sound" => {
                                    self.make_sound()?
                                }
                                "Potion" => {
                                    self.make_potion()?
                                }
                                _ => {
                                    return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![] })
                                }
                            }
                        }
                        _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: "any".into() }] })
                    });
                }
                _ => {
                    self.token_index -= 1;
                }
            }

            self.variables.push(VariableNode {
                dfrs_name: param_name.clone(),
                df_name: param_name.clone(),
                var_type: VariableType::Line,
                start_pos: Position::new(0, 0),
                end_pos: Position::new(0, 0),
            });

            params.push(FunctionParamNode {
                name: param_name,
                param_type,
                optional,
                multiple,
                default
            });

            let token = self.advance_err()?;
            match token.token {
                Token::Comma => {},
                Token::CloseParen => break,
                _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![] })
            }
        }

        self.require_token(Token::OpenParenCurly)?;

        let mut token;
        loop {
            token = self.advance_err()?;
            match token.token {
                Token::CloseParenCurly => break,
                _ => expressions.push(self.expression()?)
            }
        }

        Ok(FunctionNode { name, expressions, start_pos, name_end_pos: name_token.end_pos, end_pos: token.end_pos, params })
    }

    fn expression(&mut self) -> Result<ExpressionNode, ParseError> {
        let token = self.current_token.clone().unwrap();
        let node;
        let start_pos = token.start_pos.clone();
        let end_pos;

        match token.token.clone() {
            Token::Keyword { value } => {
                match value {
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
                        let res = self.variable(VariableType::Line)?;
                        end_pos = res.end_pos.clone();
                        node = Expression::Variable { node: res }
                    },
                    Keyword::VarLocal => {
                        let res = self.variable(VariableType::Local)?;
                        end_pos = res.end_pos.clone();
                        node = Expression::Variable { node: res }
                    },
                    _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Keyword { value: Keyword::E }, Token::Keyword { value: Keyword::P }] })
                }
            }
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Keyword { value: Keyword::E }, Token::Keyword { value: Keyword::P }] })
        }

        Ok(ExpressionNode { node: node.clone(), start_pos, end_pos })
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
                    return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Dot]})
                }
                token = self.advance_err()?;
                match token.token {
                    Token::Selector { value } => {
                        selector = value;
                        implicit_selector = false;
                        self.require_token(Token::Dot)?;
                    }
                    _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Selector { value: Selector::AllPlayers }]})
                }
            }
            Token::Dot => {}
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![]})
        }

        token = self.advance_err()?;
        let name = match token.token {
            Token::Identifier { value } => value,
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: String::from("<any>") }]} )
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

        Ok(ActionNode { action_type, selector, name, args, start_pos, selector_start_pos, selector_end_pos, end_pos: token.end_pos })
    }

    fn conditional(&mut self, conditional_type: ConditionalType) -> Result<ConditionalNode, ParseError> {
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

        match token.token {
            Token::Selector { value } => {
                selector = value;
                selector_start_pos = Some(token.start_pos);
                selector_end_pos = Some(token.end_pos);
                self.require_token(Token::Colon)?;
                token = self.advance_err()?;
            }
            _ => {}
        }
        let name = match token.token {
            Token::Identifier { value } => value,
            _ => return Err(ParseError::InvalidToken { found: Some(token), expected: vec![Token::Identifier { value: "any".into() }] })
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
            inverted
        })
    }

    fn variable(&mut self, var_type: VariableType) -> Result<VariableNode, ParseError> {
        let start_pos = self.current_token.clone().unwrap().start_pos;
        let end_pos = start_pos.clone();
        
        let token = self.advance_err()?;
        let dfrs_name = match token.token {
            Token::Identifier { value } => value,
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: "any".into() }] })
        };

        let token = self.advance_err()?;
        match token.token {
            Token::Equal => {}
            Token::Semicolon => {
                return {
                    let node = VariableNode { dfrs_name: dfrs_name.clone(), df_name: dfrs_name, var_type, start_pos, end_pos };
                    self.variables.push(node.clone());
                    Ok(node)
                }
            }
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Equal, Token::Semicolon] })
        };

        let token = self.advance_err()?;
        let df_name = match token.token {
            Token::Variable { value } => value,
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Variable { value: "any".into() }] })
        };

        self.require_token(Token::Semicolon)?;

        let node = VariableNode { dfrs_name, df_name, var_type, start_pos, end_pos };
        self.variables.push(node.clone());
        Ok(node)
    }
    
    fn make_params(&mut self) -> Result<Vec<ArgValueWithPos>, ParseError> {
        let token = self.advance_err()?;
        match token.token {
            Token::OpenParen => {}
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::OpenParen] })
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

        let expected = vec![Token::CloseParen, Token::Text { value: "<any>".into() }, Token::String { value: "<any>".into() }, Token::Number { value: 0.0 }, Token::Identifier { value: "Location".into() }];
        loop {
            let mut token = self.advance_err()?;

            if is_value {
                match token.token {
                    Token::Comma => {
                        is_value = false;
                        comma_pos = self.current_token.clone().unwrap().start_pos;
                    }
                    Token::CloseParen => break,
                    _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Comma, Token::CloseParen] })
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
                                value: ArgValue::Variable { value: var, scope },
                                start_pos: self.current_token.clone().unwrap().start_pos,
                                end_pos: self.current_token.clone().unwrap().end_pos,
                            });
                            is_value = true;
                            self.token_index -= 1;
                        } else {
                            return Err(ParseError::UnknownVariable { found: tag_name.clone(), start_pos: tag_start_pos, end_pos: tag_end_pos });
                        }
                    }
                }
            } else if is_tag {
                is_tag = false;
                match token.token.clone() {
                    Token::String { value } => {
                        params.push(ArgValueWithPos {
                            value: ArgValue::Tag { tag: tag_name.clone(), value, definition: None, name_end_pos: tag_end_pos.clone(), value_start_pos: token.start_pos.clone() },
                            start_pos: tag_start_pos.clone(),
                            end_pos: token.end_pos
                        });
                        is_value = true;
                    }
                    Token::Text { value } => {
                        params.push(ArgValueWithPos {
                            value: ArgValue::Tag { tag: tag_name.clone(), value, definition: None, name_end_pos: tag_end_pos.clone(), value_start_pos: token.start_pos },
                            start_pos: tag_start_pos.clone(),
                            end_pos: token.end_pos
                        });
                        is_value = true;
                    }
                    _ => {
                        return Err(ParseError::InvalidToken { found: Some(token), expected: vec![Token::String { value: "<any>".into() }, Token::Text { value: "<any>".into() }] })
                    }
                }
            } else if is_game_value {
                let mut selector = Selector::Default;
                let mut selector_end_pos = token.start_pos.clone();
                let start_pos = token.start_pos.clone();

                if let Token::Selector { value } = token.token.clone() {
                    selector = value;
                    token = self.advance_err()?;
                    if token.token != Token::Colon {
                        return Err(ParseError::InvalidToken { found: Some(token), expected: vec![Token::Colon]})
                    }
                    selector_end_pos = token.end_pos;
                    token = self.advance_err()?;
                }
                
                match token.token.clone() {
                    Token::Identifier { value } => {
                        params.push(ArgValueWithPos {
                            value: ArgValue::GameValue {
                                value,
                                selector,
                                selector_end_pos
                            },
                            start_pos,
                            end_pos: token.end_pos.clone(),
                        });
                        is_value = true;
                        is_game_value = false;
                    }
                    _ => return Err(ParseError::InvalidToken { found: Some(token), expected: vec![Token::Identifier {value: "<any>".into()}, Token::Selector {value: Selector::Default}] })
                }
            } else {
                match token.token.clone() {
                    Token::Number { value } => {
                        params.push(ArgValueWithPos {
                            value: ArgValue::Number { number: value },
                            start_pos: token.start_pos,
                            end_pos: token.end_pos
                        });
                        is_value = true;
                    }
                    Token::Text { value } => {
                        params.push(ArgValueWithPos {
                            value: ArgValue::Text { text: value },
                            start_pos: token.start_pos,
                            end_pos: token.end_pos
                        });
                        is_value = true;
                    }
                    Token::String { value } => {
                        params.push(ArgValueWithPos {
                            value: ArgValue::String { string: value },
                            start_pos: token.start_pos,
                            end_pos: token.end_pos
                        });
                        is_value = true;
                    }
                    Token::Identifier { value }  => {
                        match value.as_str() {
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
                            "null" => {
                                params.push(ArgValueWithPos {
                                    value: ArgValue::Empty,
                                    start_pos: self.current_token.clone().unwrap().start_pos,
                                    end_pos: self.current_token.clone().unwrap().end_pos
                                });
                                is_value = true;
                            }
                            _ => {
                                could_be_tag = true;
                                tag_name = value;
                                tag_start_pos = self.current_token.clone().unwrap().start_pos;
                                tag_end_pos = self.current_token.clone().unwrap().end_pos;
                            }
                        }
                    }
                    Token::Dollar => is_game_value = true,
                    Token::CloseParen => break,
                    _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected })
                }
            }
        }
        if !params.is_empty() && !is_value {
            return Err(ParseError::InvalidToken { found: Some(TokenWithPos::new(Token::Comma, comma_pos.clone(), comma_pos)), expected })
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
                ArgValue::String { .. } => ArgType::STRING,
                ArgValue::Text { .. } => ArgType::TEXT,
                ArgValue::Location { .. } => ArgType::LOCATION,
                ArgValue::Potion { .. } => ArgType::POTION,
                ArgValue::Sound { .. } => ArgType::SOUND,
                ArgValue::Vector { .. } => ArgType::VECTOR,
                ArgValue::Tag { ..} => ArgType::TAG,
                ArgValue::Variable { .. } => ArgType::VARIABLE,
                ArgValue::GameValue { .. } => ArgType::GameValue
            };
            args.push(Arg { value: param.value, index: i as i32, arg_type, start_pos: param.start_pos, end_pos: param.end_pos});
        }
        Ok(args)
    }

    fn make_location(&mut self) -> Result<ArgValueWithPos, ParseError> {
        let mut pitch = None;
        let mut yaw = None;
        let start_pos = self.current_token.clone().unwrap().start_pos;
        let loc_params = self.make_params()?;

        if loc_params.len() < 3 {
            return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
        }
        let x = match loc_params[0].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid x coordinate".into() })
        };
        let y = match loc_params[1].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid y coordinate".into() })
        };
        let z = match loc_params[2].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid z coordinate".into() })
        };
        if loc_params.len() >= 4 {
            match loc_params[3].value {
                ArgValue::Number { number } => pitch = Some(number),
                _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid pitch".into() })
            }
        }
        if loc_params.len() == 5 {
            match loc_params[4].value {
                ArgValue::Number { number } => yaw = Some(number),
                _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid yaw".into() })
            }
        }
        if loc_params.len() > 5 {
            return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
        }
        Ok(ArgValueWithPos {
            value: ArgValue::Location { x, y, z, pitch, yaw },
            start_pos,
            end_pos: self.current_token.clone().unwrap().end_pos
        })
    }

    fn make_vector(&mut self) -> Result<ArgValueWithPos, ParseError> {
        let start_pos = self.current_token.clone().unwrap().start_pos;
        let vec_params = self.make_params()?;

        if vec_params.len() < 3 {
            return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
        }
        let x = match vec_params[0].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid x coordinate".into() })
        };
        let y = match vec_params[1].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid y coordinate".into() })
        };
        let z = match vec_params[2].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid z coordinate".into() })
        };
        if vec_params.len() > 3 {
            return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
        }
        Ok(ArgValueWithPos {
            value: ArgValue::Vector { x, y, z },
            start_pos,
            end_pos: self.current_token.clone().unwrap().end_pos    
        })
    }

    fn make_sound(&mut self) -> Result<ArgValueWithPos, ParseError> {
        let start_pos = self.current_token.clone().unwrap().start_pos;
        let sound_params = self.make_params()?;

        if sound_params.len() < 3 {
            return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
        }
        let sound = match &sound_params[0].value {
            ArgValue::String { string } => string.clone(),
            ArgValue::Text { text } => text.clone(),
            _ => return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid sound type".into() })
        };
        let volume = match sound_params[1].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid volume".into() })
        };
        let pitch = match sound_params[2].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid pitch".into() })
        };
        if sound_params.len() > 3 {
            return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
        }
        Ok(ArgValueWithPos {
            value: ArgValue::Sound { sound, volume, pitch },
            start_pos,
            end_pos: self.current_token.clone().unwrap().end_pos
        }) 
    }

    fn make_potion(&mut self) -> Result<ArgValueWithPos, ParseError> {
        let start_pos = self.current_token.clone().unwrap().start_pos;
        let potion_params = self.make_params()?;

        if potion_params.len() < 3 {
            return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
        }
        let potion = match &potion_params[0].value {
            ArgValue::String { string } => string.clone(),
            ArgValue::Text { text } => text.clone(),
            _ => return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid potion type".into() })
        };
        let amplifier = match potion_params[1].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid amplifier".into() })
        };
        let duration = match potion_params[2].value {
            ArgValue::Number { number } => number,
            _ => return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid duration".into() })
        };
        if potion_params.len() > 3 {
            return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
        }
        Ok(ArgValueWithPos {
            value: ArgValue::Potion { potion, amplifier, duration },
            start_pos,
            end_pos: self.current_token.clone().unwrap().end_pos
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
                return Some((node.df_name.clone(), scope.to_owned()))
            }
        }

        None
    }
}