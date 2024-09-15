use crate::token::{Position, Token, TokenWithPos, KEYWORDS, SELECTORS};

#[derive(Debug)]
pub enum LexerError {
    InvalidNumber { pos: Position },
    InvalidToken { token: char, pos: Position },
    UnterminatedString { pos: Position },
    UnterminatedText { pos: Position },
    UnterminatedVariable { pos: Position }
}

pub struct Lexer {
    char_pos: i32,
    input: String,
    position: Position,
    current_char: Option<char>,
    next_char_in_new_line: bool
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        Lexer { input, current_char: None, char_pos: -1, position: Position::new(1, 0), next_char_in_new_line: false }
    }

    fn advance(&mut self) {
        self.char_pos += 1;
        self.position.advance();

        if self.char_pos >= self.input.chars().count() as i32 { 
            self.current_char = None 
        } else {
            self.current_char = Some(self.input.chars().nth(self.char_pos as usize).unwrap())
        }

        if self.next_char_in_new_line {
            self.next_char_in_new_line = false;
            self.position.next_line();
        }
        if self.current_char.is_some() && self.current_char.unwrap() == '\n' {
            self.next_char_in_new_line = true;
        }
    }

    fn make_number(&mut self) -> Result<TokenWithPos, LexerError> {
        let mut num_string: String = String::from("");
        let mut dot_count = 0;
        let start_pos = self.position.clone();

        while self.current_char.is_some() && (self.current_char.unwrap().is_ascii_digit() || self.current_char.unwrap() == '.' || self.current_char.unwrap() == '-') {
            if self.current_char.unwrap() == '.' { dot_count += 1 }
            if self.current_char.unwrap() == '-' {
                if !num_string.is_empty() {
                    return Err(LexerError::InvalidNumber {pos: start_pos});
                }
            }
            if dot_count > 1 { return Err(LexerError::InvalidNumber{ pos: self.position.clone() }) }
            num_string.push_str(&self.current_char.unwrap().to_string());
            self.advance();
        }

        if num_string.is_empty() {
            return Err(LexerError::InvalidNumber { pos: start_pos })
        }

        Ok(TokenWithPos { token: Token::Number { value: num_string.parse::<f32>().unwrap() }, start_pos, end_pos: self.position.clone()})
    }

    fn make_string(&mut self) -> Result<TokenWithPos, LexerError> {
        let mut string: String = String::from("");
        let mut escape = false;
        let mut is_escaped;
        let start_pos = self.position.clone();

        loop {
            self.advance();
            if self.current_char.is_none() {
                return Err(LexerError::UnterminatedString { pos: start_pos })
            }

            is_escaped = escape;
            escape = false;

            let char = self.current_char.unwrap();

            if !is_escaped && char == '\'' {
                self.advance();
                break;
            }

            string.push_str(&char.to_string());

            if !is_escaped && char == '\\' {
                escape = true;
            }
        }

        Ok(TokenWithPos { token: Token::String { value: string }, start_pos, end_pos: self.position.clone() })
    }

    fn make_text(&mut self) -> Result<TokenWithPos, LexerError> {
        let mut string: String = String::from("");
        let mut escape = false;
        let mut is_escaped;
        let start_pos = self.position.clone();

        loop {
            self.advance();
            if self.current_char.is_none() {
                return Err(LexerError::UnterminatedText { pos: start_pos })
            }

            is_escaped = escape;
            escape = false;

            let char = self.current_char.unwrap();

            if !is_escaped && char == '\"' {
                self.advance();
                break;
            }

            string.push_str(&char.to_string());

            if !is_escaped && char == '\\' {
                escape = true;
            }
        }

        Ok(TokenWithPos { token: Token::Text { value: string }, start_pos, end_pos: self.position.clone() })
    }

    fn make_variable(&mut self) -> Result<TokenWithPos, LexerError> {
        let mut string: String = String::from("");
        let mut escape = false;
        let mut is_escaped;
        let start_pos = self.position.clone();

        loop {
            self.advance();
            if self.current_char.is_none() {
                return Err(LexerError::UnterminatedVariable { pos: start_pos })
            }

            is_escaped = escape;
            escape = false;

            let char = self.current_char.unwrap();

            if !is_escaped && char == '`' {
                self.advance();
                break;
            }

            string.push_str(&char.to_string());

            if !is_escaped && char == '\\' {
                escape = true;
            }
        }

        Ok(TokenWithPos { token: Token::Variable { value: string }, start_pos, end_pos: self.position.clone() })
    }

    fn make_identifier_or_keyword(&mut self) -> Result<TokenWithPos, LexerError> {
        let mut value: String = String::from("");
        let start_pos = self.position.clone();

        loop {
            if self.current_char.is_none() {
                break;
            }

            let char = self.current_char.unwrap();

            if !char.is_ascii_alphanumeric() && char != '_' {
                break;
            }

            value.push_str(&char.to_string());
            self.advance();
        }

        let keyword = KEYWORDS.get(&value).cloned();
        if let Some(keyword) = keyword {
            return Ok(TokenWithPos { token: Token::Keyword { value: keyword }, start_pos, end_pos: self.position.clone()})
        }

        let selector = SELECTORS.get(&value).cloned();
        if let Some(selector) = selector {
            return Ok(TokenWithPos { token: Token::Selector { value: selector }, start_pos, end_pos: self.position.clone()})
        }

        Ok(TokenWithPos { token: Token::Identifier { value }, start_pos, end_pos: self.position.clone() })
    }

    pub fn run(&mut self) -> Result<Vec<TokenWithPos>, LexerError> {
        self.advance();

        let mut result: Vec<TokenWithPos> = vec![];
        let mut comment = 0;
        let mut is_comment = false;

        while self.current_char.is_some() {
            let current = self.current_char.unwrap();
        
            if current != '/' {
                comment = 0;
            }

            if is_comment {
                if current == '\n' {
                    is_comment = false;
                    comment = 0;
                } else {
                    self.advance();
                    continue;
                }
            }

            match current {
                ' ' => self.advance(),
                '\t' => self.advance(),
                '\n' => self.advance(),
                '\r' => self.advance(),
                '(' => {
                    result.push(self.token(Token::OpenParen));
                    self.advance();
                }
                ')' => {
                    result.push(self.token(Token::CloseParen));
                    self.advance();
                }
                '{' => {
                    result.push(self.token(Token::OpenParenCurly));
                    self.advance();
                }
                '}' => {
                    result.push(self.token(Token::CloseParenCurly));
                    self.advance();
                }
                '+' => {
                    result.push(self.token(Token::Plus));
                    self.advance();
                }
                '-' => {
                    let token = match self.make_number() {
                        Ok(res) => res,
                        Err(_) => {
                            self.advance();
                            self.token(Token::Minus)
                        }
                    };
                    result.push(token);
                }
                '*' => {
                    result.push(self.token(Token::Multiply));
                    self.advance();
                }
                '/' => {
                    comment += 1;
                    if comment == 2 {
                        is_comment = true;
                        result.pop();
                    } else {
                        result.push(self.token(Token::Divide));
                    }
                    self.advance();
                }
                '@' => {
                    result.push(self.token(Token::At));
                    self.advance();
                }
                ':' => {
                    result.push(self.token(Token::Colon));
                    self.advance();
                }
                '!' => {
                    result.push(self.token(Token::ExclamationMark));
                    self.advance();
                }
                '.' => {
                    result.push(self.token(Token::Dot));
                    self.advance();
                }
                ',' => {
                    result.push(self.token(Token::Comma));
                    self.advance();
                }
                '=' => {
                    result.push(self.token(Token::Equal));
                    self.advance();
                }
                ';' => {
                    result.push(self.token(Token::Semicolon));
                    self.advance();
                }
                '?' => {
                    result.push(self.token(Token::QuestionMark));
                    self.advance();
                }
                '$' => {
                    result.push(self.token(Token::Dollar));
                    self.advance();
                }
                '0'..='9' => result.push(self.make_number()?),
                '\'' => result.push(self.make_string()?),
                '"' => result.push(self.make_text()?),
                '`' => result.push(self.make_variable()?),
                'a'..='z' => result.push(self.make_identifier_or_keyword()?),
                'A'..='Z' => result.push(self.make_identifier_or_keyword()?),
                '_' => result.push(self.make_identifier_or_keyword()?),
                _ => {
                    return Err(LexerError::InvalidToken { token: current, pos: self.position.clone() });
                }
            }
        }

        Ok(result)
    }

    fn token(&self, token: Token) -> TokenWithPos {
        TokenWithPos::new(token, self.position.clone(), self.position.clone())
    }
}