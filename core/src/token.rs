use std::fmt::Display;

use phf::phf_map;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Position {
    pub line: u32,
    pub col: u32
}

impl Position {
    pub fn new(line: u32, col: u32) -> Position {
        Position { line, col }
    }

    pub fn advance(&mut self) {
        self.col += 1;
    }

    pub fn next_line(&mut self) {
        self.col = 1;
        self.line += 1;
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[derive(Debug, Clone)]
pub struct TokenWithPos {
    pub token: Token,
    pub start_pos: Position,
    pub end_pos: Position
}

impl TokenWithPos {
    pub fn new(token: Token, start_pos: Position, end_pos: Position) -> TokenWithPos {
        TokenWithPos { token, start_pos, end_pos }
    }
}


#[derive(Debug, Clone)]
pub enum Token {
    Plus,
    Minus,
    Multiply,
    Divide,
    At,
    Colon,
    Dot,
    Comma,
    Equal,
    Semicolon,
    OpenParen,
    CloseParen,
    OpenParenCurly,
    CloseParenCurly,
    Number { value: f32 },
    String { value: String },
    Text { value: String },
    Variable { value: String },
    Identifier { value: String },
    Keyword { value: Keyword },
    Selector { value: Selector }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::At => write!(f, "@"),
            Token::Colon => write!(f, ":"),
            Token::Dot => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::Equal => write!(f, "="),
            Token::Semicolon => write!(f, ";"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenParenCurly => write!(f, "{{"),
            Token::CloseParenCurly => write!(f, "}}"),
            Token::Number { .. } => write!(f, "Number"),
            Token::String { .. } => write!(f, "String"),
            Token::Text { .. } => write!(f, "Text"),
            Token::Variable { .. } => write!(f, "Variable"),
            Token::Identifier { .. } => write!(f, "Identifier"),
            Token::Keyword { value } => write!(f, "Keyword:{}", value),
            Token::Selector { .. } => write!(f, "Selector"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Keyword {
    P,
    E,
    G,
    VarLine,
    VarLocal,
    VarGame,
    VarSave,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::P => write!(f, "p"),
            Keyword::E => write!(f, "e"),
            Keyword::G => write!(f, "g"),
            Keyword::VarLine => write!(f, "line"),
            Keyword::VarLocal => write!(f, "local"),
            Keyword::VarGame => write!(f, "game"),
            Keyword::VarSave => write!(f, "save"),
        }
    }
}

pub static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "p" => Keyword::P,
    "e" => Keyword::E,
    "g" => Keyword::G,
    "line" => Keyword::VarLine,
    "local" => Keyword::VarLocal,
    "game" => Keyword::VarGame,
    "save" => Keyword::VarSave,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Selector {
    Default,
    Selection,
    Killer,
    Damager,
    Shooter,
    Victim,
    AllPlayers,
    Projectile,
    AllEntities,
    AllMobs,
    LastSpawned
}

pub static SELECTORS: phf::Map<&'static str, Selector> = phf_map! {
    "default" => Selector::Default,
    "selection" => Selector::Selection,
    "killer" => Selector::Killer,
    "damager" => Selector::Damager,
    "shooter" => Selector::Shooter,
    "victim" => Selector::Victim,
    "all" => Selector::AllPlayers,
    "projectile" => Selector::Projectile,
    "allEntities" => Selector::AllEntities,
    "allMobs" => Selector::AllMobs,
    "last" => Selector::LastSpawned,
};
