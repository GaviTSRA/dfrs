use crate::{definitions::ArgType, node::{ActionNode, ActionType, Arg, ArgValue, EventNode, Expression, ExpressionNode, FileNode}, token::{Keyword, Position, Selector, Token, TokenWithPos}};

#[derive(Debug)]
pub enum ParseError {
    InvalidToken { found: Option<TokenWithPos>, expected: Vec<Token> },
    InvalidLocation { pos: Position, msg: String },
    InvalidVector { pos: Position, msg: String },
    InvalidSound { pos: Position, msg: String },
    InvalidPotion { pos: Position, msg: String },
}

pub struct Parser {
    tokens: Vec<TokenWithPos>,
    token_index: i32,
    current_token: Option<TokenWithPos>
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithPos>) -> Parser {
        Parser { tokens, token_index: -1, current_token: None }
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
        let start_pos = Position::new(1, 0);

        while token.is_some() {
            match token.clone().unwrap().token {
                Token::At => events.push(self.event()?),
                _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::At] })
            }
            token = self.advance();
        }
        
        let end_pos;
        if events.len() > 0 {
            end_pos = events.get(events.len() - 1).unwrap().end_pos.clone();
        } else {
            end_pos = start_pos.clone();
        }
        Ok(FileNode { events, start_pos, end_pos })
    }

    fn event(&mut self) -> Result<EventNode, ParseError> {
        let event;
        let mut expressions: Vec<ExpressionNode> = vec![];
        let start_pos = self.current_token.clone().unwrap().end_pos;

        let token = self.advance_err()?;
        match token.token {
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

        Ok(EventNode { event_type: None, event, expressions, start_pos, end_pos: token.end_pos })
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
            let arg_type = match param {
                ArgValue::Empty => ArgType::EMPTY,
                ArgValue::Number { number: _ } => ArgType::NUMBER,
                ArgValue::String { string: _ } => ArgType::STRING,
                ArgValue::Text { text:_ } => ArgType::TEXT,
                ArgValue::Location { x: _, y: _, z: _, pitch: _, yaw: _ } => ArgType::LOCATION,
                ArgValue::Potion { potion: _, amplifier: _, duration: _ } => ArgType::POTION,
                ArgValue::Sound { sound: _, volume: _, pitch: _ } => ArgType::SOUND,
                ArgValue::Vector { x: _, y: _, z: _ } => ArgType::VECTOR,
                ArgValue::Tag { tag: _, value: _, definition: _ } => ArgType::TAG
            };
            args.push(Arg { value: param, index: i, arg_type});
            i += 1
        }

        Ok(ActionNode { action_type, selector, name, args, start_pos, end_pos: token.end_pos, implicit_selector })
    }
    
    fn make_params(&mut self) -> Result<Vec<ArgValue>, ParseError> {
        let token = self.advance_err()?;
        match token.token {
            Token::OpenParen => {}
            _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected: vec![Token::OpenParen] })
        }

        let mut params: Vec<ArgValue> = vec![];
        let mut is_value = false;
        let mut could_be_tag = false;
        let mut tag_name: String = "".into();
        let mut is_tag = false;

        let expected = vec![Token::CloseParen, Token::Text { value: "<any>".into() }, Token::String { value: "<any>".into() }, Token::Number { value: 0.0 }, Token::Identifier { value: "Location".into() }];
        loop {
            let token = self.advance_err()?;

            if is_value {
                match token.token {
                    Token::Comma => {
                        is_value = false;
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
                        return Err(ParseError::InvalidToken { found: Some(token), expected: vec![Token::Equal] })
                    }
                }
            } else if is_tag {
                is_tag = false;
                match token.token.clone() {
                    Token::String { value } => {
                        params.push(ArgValue::Tag { tag: tag_name.clone(), value, definition: None });
                        is_value = true;
                    }
                    Token::Text { value } => {
                        params.push(ArgValue::Tag { tag: tag_name.clone(), value, definition: None });
                        is_value = true;
                    }
                    _ => {
                        return Err(ParseError::InvalidToken { found: Some(token), expected: vec![Token::String { value: "<any>".into() }, Token::Text { value: "<any>".into() }] })
                    }
                }
            } else {
                match token.token.clone() {
                    Token::Number { value } => {
                        params.push(ArgValue::Number { number: value });
                        is_value = true;
                    }
                    Token::Text { value } => {
                        params.push(ArgValue::Text { text: value });
                        is_value = true;
                    }
                    Token::String { value } => {
                        params.push(ArgValue::String { string: value });
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
                                let loc_params = self.make_params()?;

                                if loc_params.len() < 3 {
                                    return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
                                }
                                match loc_params[0] {
                                    ArgValue::Number { number } => x = number,
                                    _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid x coordinate".into() })
                                }
                                match loc_params[1] {
                                    ArgValue::Number { number } => y = number,
                                    _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid y coordinate".into() })
                                }
                                match loc_params[2] {
                                    ArgValue::Number { number } => z = number,
                                    _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid z coordinate".into() })
                                }
                                if loc_params.len() >= 4 {
                                    match loc_params[3] {
                                        ArgValue::Number { number } => pitch = Some(number),
                                        _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid pitch".into() })
                                    }
                                }
                                if loc_params.len() == 5 {
                                    match loc_params[4] {
                                        ArgValue::Number { number } => yaw = Some(number),
                                        _ => return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid yaw".into() })
                                    }
                                }
                                if loc_params.len() > 5 {
                                    return Err(ParseError::InvalidLocation { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
                                }
                                params.push(ArgValue::Location { x, y, z, pitch, yaw });
                                is_value = true;
                            }
                            "Vector" => {
                                let x;
                                let y;
                                let z;
                                let vec_params = self.make_params()?;

                                if vec_params.len() < 3 {
                                    return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
                                }
                                match vec_params[0] {
                                    ArgValue::Number { number } => x = number,
                                    _ => return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid x coordinate".into() })
                                }
                                match vec_params[1] {
                                    ArgValue::Number { number } => y = number,
                                    _ => return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid y coordinate".into() })
                                }
                                match vec_params[2] {
                                    ArgValue::Number { number } => z = number,
                                    _ => return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid z coordinate".into() })
                                }
                                if vec_params.len() > 3 {
                                    return Err(ParseError::InvalidVector { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
                                }
                                params.push(ArgValue::Vector { x, y, z }); 
                                is_value = true;
                            }
                            "Sound" => {
                                let sound;
                                let volume;
                                let pitch;
                                let sound_params = self.make_params()?;

                                if sound_params.len() < 3 {
                                    return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
                                }
                                match &sound_params[0] {
                                    ArgValue::String { string } => sound = string.clone(),
                                    ArgValue::Text { text } => sound = text.clone(),
                                    _ => return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid sound type".into() })
                                }
                                match sound_params[1] {
                                    ArgValue::Number { number } => volume = number,
                                    _ => return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid volume".into() })
                                }
                                match sound_params[2] {
                                    ArgValue::Number { number } => pitch = number,
                                    _ => return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid pitch".into() })
                                }
                                if sound_params.len() > 3 {
                                    return Err(ParseError::InvalidSound { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
                                }
                                params.push(ArgValue::Sound { sound, volume, pitch }); 
                                is_value = true;
                            }
                            "Potion" => {
                                let potion;
                                let amplifier;
                                let duration;
                                let potion_params = self.make_params()?;

                                if potion_params.len() < 3 {
                                    return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Not enough arguments".into() })
                                }
                                match &potion_params[0] {
                                    ArgValue::String { string } => potion = string.clone(),
                                    ArgValue::Text { text } => potion = text.clone(),
                                    _ => return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid potion type".into() })
                                }
                                match potion_params[1] {
                                    ArgValue::Number { number } => amplifier = number,
                                    _ => return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid amplifier".into() })
                                }
                                match potion_params[2] {
                                    ArgValue::Number { number } => duration = number,
                                    _ => return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Invalid duration".into() })
                                }
                                if potion_params.len() > 3 {
                                    return Err(ParseError::InvalidPotion { pos: self.current_token.clone().unwrap().start_pos, msg: "Too many arguments".into() })
                                }
                                params.push(ArgValue::Potion { potion, amplifier, duration }); 
                                is_value = true;
                            }
                            "null" => {
                                params.push(ArgValue::Empty);
                                is_value = true;
                            }
                            _ => {
                                could_be_tag = true;
                                tag_name = value;
                            }
                        }
                    }
                    Token::CloseParen => break,
                    _ => return Err(ParseError::InvalidToken { found: self.current_token.clone(), expected })
                }
            }
        }
        Ok(params)
    }
}