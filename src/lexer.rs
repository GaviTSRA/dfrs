use crate::token::{Position, Token, TokenWithPos, KEYWORDS, SELECTORS};

#[derive(Debug, Clone)]
pub enum LexerError {
  InvalidNumber { pos: Position },
  InvalidToken { token: char, pos: Position },
  UnterminatedString { pos: Position },
  UnterminatedText { pos: Position },
  UnterminatedVariable { pos: Position },
}

pub struct Lexer<'a> {
  input: String,
  chars: std::str::Chars<'a>,
  char_stack: Vec<char>,
  position: Position,
  current_char: Option<char>,
  next_char_in_new_line: bool,
}

impl<'a> Lexer<'a> {
  pub fn new(input: &str) -> Lexer {
    Lexer {
      input: input.to_owned(),
      current_char: None,
      chars: input.chars(),
      char_stack: Vec::new(),
      position: Position::new(1, 0),
      next_char_in_new_line: false,
    }
  }

  fn advance(&mut self) {
    if let Some(c) = self.current_char {
      self.char_stack.push(c);
    }

    self.position.advance();
    self.current_char = self.chars.next();

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

    while self.current_char.is_some()
      && (self.current_char.unwrap().is_ascii_digit()
        || self.current_char.unwrap() == '.'
        || self.current_char.unwrap() == '_'
        || self.current_char.unwrap() == '-')
    {
      if self.current_char.unwrap() == '_' {
        self.advance();
        continue;
      }
      if self.current_char.unwrap() == '.' {
        dot_count += 1
      }
      if self.current_char.unwrap() == '-' {
        if !num_string.is_empty() {
          return Err(LexerError::InvalidNumber { pos: start_pos });
        }
      }
      if dot_count > 1 {
        return Err(LexerError::InvalidNumber {
          pos: self.position.clone(),
        });
      }
      num_string.push_str(&self.current_char.unwrap().to_string());
      self.advance();
    }

    if num_string.is_empty() || num_string == "-" {
      return Err(LexerError::InvalidNumber { pos: start_pos });
    }

    Ok(TokenWithPos {
      token: Token::Number {
        value: match num_string.parse::<f32>() {
          Ok(res) => res,
          Err(_) => return Err(LexerError::InvalidNumber { pos: start_pos }),
        },
      },
      start_pos,
      end_pos: self.position.clone(),
    })
  }

  fn make_string(&mut self) -> Result<TokenWithPos, LexerError> {
    let mut string: String = String::from("");
    let mut escape = false;
    let mut is_escaped;
    let start_pos = self.position.clone();

    loop {
      self.advance();
      if self.current_char.is_none() {
        return Err(LexerError::UnterminatedString { pos: start_pos });
      }

      is_escaped = escape;
      escape = false;

      let char = self.current_char.unwrap();

      if !is_escaped && char == '\'' {
        self.advance();
        break;
      }

      if !is_escaped && char == '\\' {
        escape = true;
      } else {
        string.push_str(&char.to_string());
      }
    }

    Ok(TokenWithPos {
      token: Token::String { value: string },
      start_pos,
      end_pos: self.position.clone(),
    })
  }

  fn make_text(&mut self) -> Result<TokenWithPos, LexerError> {
    let mut string: String = String::from("");
    let mut escape = false;
    let mut is_escaped;
    let start_pos = self.position.clone();

    loop {
      self.advance();
      if self.current_char.is_none() {
        return Err(LexerError::UnterminatedText { pos: start_pos });
      }

      is_escaped = escape;
      escape = false;

      let char = self.current_char.unwrap();

      if !is_escaped && char == '\"' {
        self.advance();
        break;
      }

      if !is_escaped && char == '\\' {
        escape = true;
      } else {
        string.push_str(&char.to_string());
      }
    }

    Ok(TokenWithPos {
      token: Token::Text { value: string },
      start_pos,
      end_pos: self.position.clone(),
    })
  }

  fn make_variable(&mut self) -> Result<TokenWithPos, LexerError> {
    let mut string: String = String::from("");
    let mut escape = false;
    let mut is_escaped;
    let start_pos = self.position.clone();

    loop {
      self.advance();
      if self.current_char.is_none() {
        return Err(LexerError::UnterminatedVariable { pos: start_pos });
      }

      is_escaped = escape;
      escape = false;

      let char = self.current_char.unwrap();

      if !is_escaped && char == '`' {
        self.advance();
        break;
      }

      if !is_escaped && char == '\\' {
        escape = true;
      } else {
        string.push_str(&char.to_string());
      }
    }

    Ok(TokenWithPos {
      token: Token::Variable { value: string },
      start_pos,
      end_pos: self.position.clone(),
    })
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
      return Ok(TokenWithPos {
        token: Token::Keyword { value: keyword },
        start_pos,
        end_pos: self.position.clone(),
      });
    }

    let selector = SELECTORS.get(&value).cloned();
    if let Some(selector) = selector {
      return Ok(TokenWithPos {
        token: Token::Selector { value: selector },
        start_pos,
        end_pos: self.position.clone(),
      });
    }

    Ok(TokenWithPos {
      token: Token::Identifier { value },
      start_pos,
      end_pos: self.position.clone(),
    })
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
              self.position.rewind();
              let token = self.token(Token::Minus);
              self.position.advance();
              token
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
          return Err(LexerError::InvalidToken {
            token: current,
            pos: self.position.clone(),
          });
        }
      }
    }

    Ok(result)
  }

  fn token(&self, token: Token) -> TokenWithPos {
    TokenWithPos::new(token, self.position.clone(), self.position.clone())
  }
}

#[cfg(test)]
mod tests {
  use crate::token::{Keyword, Selector};

  use super::*;

  #[test]
  pub fn basic_tokens() {
    let values = vec![
      ("+", Token::Plus),
      ("-", Token::Minus),
      ("*", Token::Multiply),
      ("/", Token::Divide),
      ("@", Token::At),
      (":", Token::Colon),
      ("!", Token::ExclamationMark),
      (".", Token::Dot),
      (",", Token::Comma),
      ("=", Token::Equal),
      (";", Token::Semicolon),
      ("?", Token::QuestionMark),
      ("$", Token::Dollar),
      ("(", Token::OpenParen),
      (")", Token::CloseParen),
      ("{", Token::OpenParenCurly),
      ("}", Token::CloseParenCurly),
    ];

    for value in values {
      println!("Testing token {:?} to be '{:?}'", value.0, value.1);
      let result = Lexer::new(value.0).run().expect("Lexer failed");
      let token = result.get(0).expect("Lexer did not return tokens");
      assert_eq!(token.token, value.1);
    }
  }

  #[test]
  pub fn multiple_tokens() {
    let result = Lexer::new("+-*/@:!.,=;?$(){}").run().expect("Lexer failed");
    let tokens: Vec<&Token> = result.iter().map(|t| &t.token).collect();
    assert_eq!(
      tokens,
      vec![
        &Token::Plus,
        &Token::Minus,
        &Token::Multiply,
        &Token::Divide,
        &Token::At,
        &Token::Colon,
        &Token::ExclamationMark,
        &Token::Dot,
        &Token::Comma,
        &Token::Equal,
        &Token::Semicolon,
        &Token::QuestionMark,
        &Token::Dollar,
        &Token::OpenParen,
        &Token::CloseParen,
        &Token::OpenParenCurly,
        &Token::CloseParenCurly
      ]
    );
  }

  #[test]
  pub fn whitespace() {
    let result = Lexer::new(" \n\t\r").run().expect("Lexer failed");
    let tokens: Vec<&Token> = result.iter().map(|t| &t.token).collect();
    assert_eq!(tokens.len(), 0);
  }

  #[test]
  pub fn comments() {
    let values = vec![("//abc\n+", Token::Plus), ("+ //abc", Token::Plus)];

    for value in values {
      println!("Testing token {:?} to be '{:?}'", value.0, value.1);
      let result = Lexer::new(value.0).run().expect("Lexer failed");
      let token = result.get(0).expect("Lexer did not return tokens");
      assert_eq!(token.token, value.1);
    }
  }

  #[test]
  pub fn numbers() {
    let values = vec![
      ("1", Token::Number { value: 1.0 }),
      ("254", Token::Number { value: 254.0 }),
      ("0.2", Token::Number { value: 0.2 }),
      ("17.24", Token::Number { value: 17.24 }),
      ("172_424", Token::Number { value: 172424.0 }),
      ("172_424.51", Token::Number { value: 172424.51 }),
      ("-172_424.51", Token::Number { value: -172424.51 }),
      ("-1", Token::Number { value: -1.0 }),
      ("-153", Token::Number { value: -153.0 }),
      ("-92.45", Token::Number { value: -92.45 }),
    ];

    for value in values {
      println!("Testing token {:?} to be '{:?}'", value.0, value.1);
      let result = Lexer::new(value.0).run().expect("Lexer failed");
      let token = result.get(0).expect("Lexer did not return tokens");
      assert_eq!(token.token, value.1);
    }
  }

  #[test]
  pub fn invalid_numbers() {
    let values = vec!["0.0.0", "1.-2", "0.."];
    for value in values {
      println!("Testing {:?}", value);
      let result = Lexer::new(value).run();
      assert!(result.is_err());
    }
  }

  #[test]
  pub fn strings() {
    let values = vec![
      (
        "'abc'",
        Token::String {
          value: "abc".to_string(),
        },
      ),
      (
        "'a\"ce'",
        Token::String {
          value: "a\"ce".to_string(),
        },
      ),
      (
        "'a b c'",
        Token::String {
          value: "a b c".to_string(),
        },
      ),
      (
        "'a b '",
        Token::String {
          value: "a b ".to_string(),
        },
      ),
      (
        "'abc\\'abc'",
        Token::String {
          value: "abc'abc".to_string(),
        },
      ),
    ];

    for value in values {
      println!("Testing token {:?} to be '{:?}'", value.0, value.1);
      let result = Lexer::new(value.0).run().expect("Lexer failed");
      let token = result.get(0).expect("Lexer did not return tokens");
      assert_eq!(token.token, value.1);
    }
  }

  #[test]
  pub fn string_must_terminate() {
    let result = Lexer::new("'abc").run();
    assert!(result.is_err());
  }

  #[test]
  pub fn text() {
    let values = vec![
      (
        "\"abc\"",
        Token::Text {
          value: "abc".to_string(),
        },
      ),
      (
        "\"a'ce\"",
        Token::Text {
          value: "a'ce".to_string(),
        },
      ),
      (
        "\"a b c\"",
        Token::Text {
          value: "a b c".to_string(),
        },
      ),
      (
        "\"a b \"",
        Token::Text {
          value: "a b ".to_string(),
        },
      ),
      (
        "\"abc\\\"abc\"",
        Token::Text {
          value: "abc\"abc".to_string(),
        },
      ),
    ];

    for value in values {
      println!("Testing token {:?} to be '{:?}'", value.0, value.1);
      let result = Lexer::new(value.0).run().expect("Lexer failed");
      let token = result.get(0).expect("Lexer did not return tokens");
      assert_eq!(token.token, value.1);
    }
  }

  #[test]
  pub fn text_must_terminate() {
    let result = Lexer::new("\"abc").run();
    assert!(result.is_err());
  }

  #[test]
  pub fn variables() {
    let values = vec![
      (
        "`abc`",
        Token::Variable {
          value: "abc".to_string(),
        },
      ),
      (
        "`a b c`",
        Token::Variable {
          value: "a b c".to_string(),
        },
      ),
      (
        "`a b `",
        Token::Variable {
          value: "a b ".to_string(),
        },
      ),
      (
        "`abc\\`abc`",
        Token::Variable {
          value: "abc`abc".to_string(),
        },
      ),
    ];

    for value in values {
      println!("Testing token {:?} to be '{:?}'", value.0, value.1);
      let result = Lexer::new(value.0).run().expect("Lexer failed");
      let token = result.get(0).expect("Lexer did not return tokens");
      assert_eq!(token.token, value.1);
    }
  }

  #[test]
  pub fn variable_must_terminate() {
    let result = Lexer::new("`abc").run();
    assert!(result.is_err());
  }

  #[test]
  pub fn keywords() {
    let values = vec![
      ("p", Token::Keyword { value: Keyword::P }),
      ("e", Token::Keyword { value: Keyword::E }),
      ("g", Token::Keyword { value: Keyword::G }),
      ("v", Token::Keyword { value: Keyword::V }),
      ("c", Token::Keyword { value: Keyword::C }),
      ("s", Token::Keyword { value: Keyword::S }),
      (
        "ifp",
        Token::Keyword {
          value: Keyword::IfP,
        },
      ),
      (
        "ife",
        Token::Keyword {
          value: Keyword::IfE,
        },
      ),
      (
        "ifg",
        Token::Keyword {
          value: Keyword::IfG,
        },
      ),
      (
        "ifv",
        Token::Keyword {
          value: Keyword::IfV,
        },
      ),
      (
        "else",
        Token::Keyword {
          value: Keyword::Else,
        },
      ),
      (
        "line",
        Token::Keyword {
          value: Keyword::VarLine,
        },
      ),
      (
        "local",
        Token::Keyword {
          value: Keyword::VarLocal,
        },
      ),
      (
        "game",
        Token::Keyword {
          value: Keyword::VarGame,
        },
      ),
      (
        "save",
        Token::Keyword {
          value: Keyword::VarSave,
        },
      ),
      (
        "fn",
        Token::Keyword {
          value: Keyword::Function,
        },
      ),
      (
        "proc",
        Token::Keyword {
          value: Keyword::Process,
        },
      ),
      (
        "call",
        Token::Keyword {
          value: Keyword::Call,
        },
      ),
      (
        "start",
        Token::Keyword {
          value: Keyword::Start,
        },
      ),
      (
        "repeat",
        Token::Keyword {
          value: Keyword::Repeat,
        },
      ),
    ];

    for value in values {
      println!("Testing token {:?} to be '{:?}'", value.0, value.1);
      let result = Lexer::new(value.0).run().expect("Lexer failed");
      let token = result.get(0).expect("Lexer did not return tokens");
      assert_eq!(token.token, value.1);
    }
  }

  #[test]
  pub fn selectors() {
    let values = vec![
      (
        "default",
        Token::Selector {
          value: Selector::Default,
        },
      ),
      (
        "selection",
        Token::Selector {
          value: Selector::Selection,
        },
      ),
      (
        "killer",
        Token::Selector {
          value: Selector::Killer,
        },
      ),
      (
        "damager",
        Token::Selector {
          value: Selector::Damager,
        },
      ),
      (
        "shooter",
        Token::Selector {
          value: Selector::Shooter,
        },
      ),
      (
        "victim",
        Token::Selector {
          value: Selector::Victim,
        },
      ),
      (
        "allPlayers",
        Token::Selector {
          value: Selector::AllPlayers,
        },
      ),
      (
        "projectile",
        Token::Selector {
          value: Selector::Projectile,
        },
      ),
      (
        "allEntities",
        Token::Selector {
          value: Selector::AllEntities,
        },
      ),
      (
        "allMobs",
        Token::Selector {
          value: Selector::AllMobs,
        },
      ),
      (
        "lastEntity",
        Token::Selector {
          value: Selector::LastEntity,
        },
      ),
    ];

    for value in values {
      println!("Testing token {:?} to be '{:?}'", value.0, value.1);
      let result = Lexer::new(value.0).run().expect("Lexer failed");
      let token = result.get(0).expect("Lexer did not return tokens");
      assert_eq!(token.token, value.1);
    }
  }

  #[test]
  pub fn identifiers() {
    let values = vec![
      (
        "test",
        Token::Identifier {
          value: "test".to_string(),
        },
      ),
      (
        "test_2",
        Token::Identifier {
          value: "test_2".to_string(),
        },
      ),
    ];

    for value in values {
      println!("Testing token {:?} to be '{:?}'", value.0, value.1);
      let result = Lexer::new(value.0).run().expect("Lexer failed");
      let token = result.get(0).expect("Lexer did not return tokens");
      assert_eq!(token.token, value.1);
    }
  }

  #[test]
  pub fn positions() {
    let result = Lexer::new("+  -\n*/\n'abc'`test`  \n123 ")
      .run()
      .expect("Lexer failed");
    assert_eq!(
      result,
      vec![
        TokenWithPos::new(Token::Plus, Position::new(1, 1), Position::new(1, 1)),
        TokenWithPos::new(Token::Minus, Position::new(1, 4), Position::new(1, 4)),
        TokenWithPos::new(Token::Multiply, Position::new(2, 1), Position::new(2, 1)),
        TokenWithPos::new(Token::Divide, Position::new(2, 2), Position::new(2, 2)),
        TokenWithPos::new(
          Token::String {
            value: "abc".to_owned()
          },
          Position::new(3, 1),
          Position::new(3, 6)
        ),
        TokenWithPos::new(
          Token::Variable {
            value: "test".to_owned()
          },
          Position::new(3, 6),
          Position::new(3, 12)
        ),
        TokenWithPos::new(
          Token::Number { value: 123.0 },
          Position::new(4, 1),
          Position::new(4, 4)
        ),
      ]
    );
  }
}
