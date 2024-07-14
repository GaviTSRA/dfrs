use crate::{definitions::ArgType, node::{ActionNode, ActionType, Arg, ArgValue, ArgValueWithPos, EventNode, Expression, ExpressionNode, FileNode, FunctionNode, VariableNode, VariableType}, token::{Keyword, Position, Selector, Token, TokenWithPos, SELECTORS}};

#[derive(Debug)]
pub enum ParseError {
    InvalidToken { found: Option<TokenWithPos>, expected: Vec<Token> },
    UnknownVariable { found: TokenWithPos },
    InvalidLocation { pos: Position, msg: String },
    InvalidVector { pos: Position, msg: String },
    InvalidSound { pos: Position, msg: String },
    InvalidPotion { pos: Position, msg: String },
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
        return self.current_token.clone();
    }

    fn advance_err(&mut self) -> Result<TokenWithPos, ParseError> {
        let token = self.advance();

        if token.is_none() {
            return Err(ParseError::InvalidToken { found: None, expected: vec![] }) //TODO expected
        }

        return Ok(token.unwrap())
    }

    pub fn run(&mut self) -> Result<FileNode, ParseError> {
        Ok(self.file()?)
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
                        _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::At, Token::Keyword { value: Keyword::Function }] })
                    }
                }
                _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::At, Token::Keyword { value: Keyword::Function }] })
            }
            token = self.advance();
        }
        
        let end_pos;
        if events.len() > 0 {
            end_pos = events.get(events.len() - 1).unwrap().end_pos.clone();
        } else {
            end_pos = start_pos.clone();
        }
        Ok(FileNode { events, functions, start_pos, end_pos })
    }

    fn event(&mut self) -> Result<EventNode, ParseError> {
        let event;
        let mut expressions: Vec<ExpressionNode> = vec![];
        let start_pos = self.current_token.clone().unwrap().end_pos;

        let name_token = self.advance_err()?;
        match name_token.token {
            Token::Identifier { value } => event = value,
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: String::from("<any>")}] })
        }

        let mut token = self.advance_err()?;
        match token.token {
            Token::OpenParenCurly => {},
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::OpenParenCurly] })
        }

        loop {
            token = self.advance_err()?;
            match token.token {
                Token::CloseParenCurly => break,
                _ => expressions.push(self.expression()?)
            }
        }

        Ok(EventNode { event_type: None, event, expressions, start_pos, name_end_pos: name_token.end_pos, end_pos: token.end_pos })
    }

    fn function(&mut self) -> Result<FunctionNode, ParseError> {
        let name;
        let mut expressions: Vec<ExpressionNode> = vec![];
        let start_pos = self.current_token.clone().unwrap().end_pos;

        let name_token = self.advance_err()?;
        match name_token.token {
            Token::Identifier { value } => name = value,
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: String::from("<any>")}] })
        }

        let mut token = self.advance_err()?;
        match token.token {
            Token::OpenParenCurly => {},
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::OpenParenCurly] })
        }

        loop {
            token = self.advance_err()?;
            match token.token {
                Token::CloseParenCurly => break,
                _ => expressions.push(self.expression()?)
            }
        }

        Ok(FunctionNode { name, expressions, start_pos, name_end_pos: name_token.end_pos, end_pos: token.end_pos })
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
                    Keyword::VarGame => {
                        let res = self.variable(VariableType::Game)?;
                        end_pos = res.end_pos.clone();
                        node = Expression::Variable { node: res }
                    },
                    Keyword::VarSave => {
                        let res = self.variable(VariableType::Save)?;
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
        let name;
        let mut selector = Selector::Default;
        let mut implicit_selector = true;
        let mut token = self.advance_err()?;
        let mut start_pos = token.start_pos.clone();
        start_pos.col += 1;

        match token.token {
            Token::Colon => {
                token = self.advance_err()?;
                match token.token {
                    Token::Selector { value } => {
                        selector = value;
                        implicit_selector = false;
                        token = self.advance_err()?;
                    }
                    _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Selector { value: Selector::AllPlayers }]})
                }
            }
            _ => {}
        }

        match token.token {
            Token::Dot => {}
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Dot] })
        }

        token = self.advance_err()?;
        match token.token {
            Token::Identifier { value } => {
                name = value;
            }
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Identifier { value: String::from("<any>") }]} )
        }

        let params = self.make_params()?;
        let mut args = vec![];
        let mut i = 0;
        for param in params {
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
                ArgValue::Variable { .. } => ArgType::VARIABLE
            };
            args.push(Arg { value: param.value, index: i, arg_type, start_pos: param.start_pos, end_pos: param.end_pos});
            i += 1
        }

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

        let token = self.advance_err()?;
        match token.token {
            Token::Semicolon => {}
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Semicolon ]})
        }

        Ok(ActionNode { action_type, selector, name, args, start_pos, selector_start_pos, selector_end_pos, end_pos: token.end_pos })
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

        let token = self.advance_err()?;
        match token.token {
            Token::Semicolon => {}
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::Semicolon ]})
        }

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

        let expected = vec![Token::CloseParen, Token::Text { value: "<any>".into() }, Token::String { value: "<any>".into() }, Token::Number { value: 0.0 }, Token::Identifier { value: "Location".into() }];
        loop {
            let token = self.advance_err()?;

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
                            return Err(ParseError::UnknownVariable { found: token })    
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
                                let x;
                                let y;
                                let z;
                                let mut pitch = None;
                                let mut yaw = None;
                                let start_pos = self.current_token.clone().unwrap().start_pos;
                                let loc_params = self.make_params()?;

                                if loc_params.len() < 3 {
                                    return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
                                }
                                match loc_params[0].value {
                                    ArgValue::Number { number } => x = number,
                                    _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid x coordinate".into() })
                                }
                                match loc_params[1].value {
                                    ArgValue::Number { number } => y = number,
                                    _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid y coordinate".into() })
                                }
                                match loc_params[2].value {
                                    ArgValue::Number { number } => z = number,
                                    _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid z coordinate".into() })
                                }
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
                                params.push(ArgValueWithPos {
                                    value: ArgValue::Location { x, y, z, pitch, yaw },
                                    start_pos,
                                    end_pos: self.current_token.clone().unwrap().end_pos
                                });
                                is_value = true;
                            }
                            "Vector" => {
                                let x;
                                let y;
                                let z;
                                let start_pos = self.current_token.clone().unwrap().start_pos;
                                let vec_params = self.make_params()?;

                                if vec_params.len() < 3 {
                                    return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
                                }
                                match vec_params[0].value {
                                    ArgValue::Number { number } => x = number,
                                    _ => return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid x coordinate".into() })
                                }
                                match vec_params[1].value {
                                    ArgValue::Number { number } => y = number,
                                    _ => return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid y coordinate".into() })
                                }
                                match vec_params[2].value {
                                    ArgValue::Number { number } => z = number,
                                    _ => return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid z coordinate".into() })
                                }
                                if vec_params.len() > 3 {
                                    return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
                                }
                                params.push(ArgValueWithPos {
                                    value: ArgValue::Vector { x, y, z },
                                    start_pos,
                                    end_pos: self.current_token.clone().unwrap().end_pos    
                                }); 
                                is_value = true;
                            }
                            "Sound" => {
                                let sound;
                                let volume;
                                let pitch;
                                let start_pos = self.current_token.clone().unwrap().start_pos;
                                let sound_params = self.make_params()?;

                                if sound_params.len() < 3 {
                                    return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
                                }
                                match &sound_params[0].value {
                                    ArgValue::String { string } => sound = string.clone(),
                                    ArgValue::Text { text } => sound = text.clone(),
                                    _ => return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid sound type".into() })
                                }
                                match sound_params[1].value {
                                    ArgValue::Number { number } => volume = number,
                                    _ => return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid volume".into() })
                                }
                                match sound_params[2].value {
                                    ArgValue::Number { number } => pitch = number,
                                    _ => return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid pitch".into() })
                                }
                                if sound_params.len() > 3 {
                                    return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
                                }
                                params.push(ArgValueWithPos {
                                    value: ArgValue::Sound { sound, volume, pitch },
                                    start_pos,
                                    end_pos: self.current_token.clone().unwrap().end_pos
                                }); 
                                is_value = true;
                            }
                            "Potion" => {
                                let potion;
                                let amplifier;
                                let duration;
                                let start_pos = self.current_token.clone().unwrap().start_pos;
                                let potion_params = self.make_params()?;

                                if potion_params.len() < 3 {
                                    return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
                                }
                                match &potion_params[0].value {
                                    ArgValue::String { string } => potion = string.clone(),
                                    ArgValue::Text { text } => potion = text.clone(),
                                    _ => return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid potion type".into() })
                                }
                                match potion_params[1].value {
                                    ArgValue::Number { number } => amplifier = number,
                                    _ => return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid amplifier".into() })
                                }
                                match potion_params[2].value {
                                    ArgValue::Number { number } => duration = number,
                                    _ => return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid duration".into() })
                                }
                                if potion_params.len() > 3 {
                                    return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
                                }
                                params.push(ArgValueWithPos {
                                    value: ArgValue::Potion { potion, amplifier, duration },
                                    start_pos,
                                    end_pos: self.current_token.clone().unwrap().end_pos
                                }); 
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
                    Token::CloseParen => break,
                    _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected })
                }
            }
        }
        if params.len() > 0 && !is_value {
            return Err(ParseError::InvalidToken { found: Some(TokenWithPos::new(Token::Comma, comma_pos.clone(), comma_pos)), expected })
        }
        Ok(params)
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