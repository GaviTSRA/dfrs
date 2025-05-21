use crate::lexer::LexerError;
use crate::parser::ParserError;
use crate::token::{Position, Range};
use crate::validate::ValidateError;

pub struct FormattedError {
  pub message: String,
  pub start: Position,
  pub end: Option<Position>,
}

impl FormattedError {
  pub fn new_without_end(message: String, start: Position) -> FormattedError {
    FormattedError {
      message,
      start,
      end: None,
    }
  }

  pub fn new(message: String, range: Range) -> FormattedError {
    FormattedError {
      message,
      start: range.start,
      end: Some(range.end),
    }
  }
}

pub fn format_lexer_error(error: LexerError) -> FormattedError {
  match error {
    LexerError::InvalidNumber { range } => FormattedError::new("Invalid number".to_string(), range),
    LexerError::InvalidToken { token, pos } => {
      FormattedError::new_without_end(format!("Invalid token '{token}'"), pos)
    }
    LexerError::UnterminatedString { range } => {
      FormattedError::new("Unterminated string".to_string(), range)
    }
    LexerError::UnterminatedText { range } => {
      FormattedError::new("Unterminated text".to_string(), range)
    }
    LexerError::UnterminatedVariable { range } => {
      FormattedError::new("Unterminated variable".to_string(), range)
    }
  }
}

pub fn format_parser_error(error: ParserError) -> FormattedError {
  match error {
    ParserError::InvalidToken {
      found,
      expected,
      range,
    } => {
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

        FormattedError::new(
          format!("Invalid token '{found}', expected: {expected_string}"),
          range,
        )
      } else {
        FormattedError::new(format!("Invalid EOF, expected: {expected:?}"), range)
      }
    }
    ParserError::InvalidCall { range, msg } => {
      FormattedError::new(format!("Invalid function call: {}", msg), range)
    }
    ParserError::InvalidComplexNumber { range, msg } => {
      FormattedError::new(format!("Invalid Number: {}", msg), range)
    }
    ParserError::InvalidLocation { range, msg } => {
      FormattedError::new(format!("Invalid Location: {}", msg), range)
    }
    ParserError::InvalidVector { range, msg } => {
      FormattedError::new(format!("Invalid Vector: {}", msg), range)
    }
    ParserError::InvalidSound { range, msg } => {
      FormattedError::new(format!("Invalid Sound: {}", msg), range)
    }
    ParserError::InvalidPotion { range, msg } => {
      FormattedError::new(format!("Invalid Potion: {}", msg), range)
    }
    ParserError::InvalidParticle { range, msg } => {
      FormattedError::new(format!("Invalid Particle: {}", msg), range)
    }
    ParserError::InvalidItem { range, msg } => {
      FormattedError::new(format!("Invalid Item: {}", msg), range)
    }
    ParserError::UnknownVariable { found, range } => {
      FormattedError::new(format!("Unknown variable: {}", found), range)
    }
    ParserError::InvalidType { found, range } => {
      FormattedError::new(format!("Unknown type: {}", found), range)
    }
    ParserError::InvalidUse { range } => {
      FormattedError::new("Invalid use statement".to_string(), range)
    }
  }
}

pub fn format_validator_error(error: ValidateError) -> FormattedError {
  match error {
    ValidateError::UnknownEvent { node } => FormattedError::new(
      format!("Unknown event '{}'", node.event),
      Range::new(node.range.start, node.name_end_pos),
    ),
    ValidateError::UnknownAction { name, range } => {
      FormattedError::new(format!("Unknown action '{}'", name), range)
    }
    ValidateError::UnknownFunction { name, range } => {
      FormattedError::new(format!("Unknown function '{}'", name), range)
    }
    ValidateError::MissingArgument { options, range } => {
      let message = if options.len() > 1 {
        format!(
          "Missing argument, possible options:\n     - {}",
          options.join("\n     - ")
        )
      } else {
        format!("Missing argument '{}'", options.get(0).unwrap())
      };
      FormattedError::new(message, range)
    }
    ValidateError::WrongArgumentType {
      args,
      index,
      options,
      found_type,
    } => {
      let option_texts: Vec<String> = options
        .iter()
        .map(|option| format!("\n     - {:?} ({})", option.arg_type, option.name).to_owned())
        .collect();
      let message = if options.len() > 1 {
        format!(
          "Wrong argument type, found '{:?}' but expected one of{}",
          found_type,
          option_texts.join(""),
        )
      } else {
        let option = options.get(0).unwrap().clone();
        format!(
          "Wrong argument type for '{}', expected '{:?}' but found '{:?}'",
          option.name, option.arg_type, found_type
        )
      };
      FormattedError::new(message, args.get(index as usize).unwrap().clone().range)
    }
    ValidateError::TooManyArguments { name, range } => {
      FormattedError::new(format!("Too many arguments for action '{}'", name), range)
    }
    ValidateError::InvalidTagOption {
      tag_name,
      provided,
      options,
      range,
    } => FormattedError::new(
      format!(
        "Invalid option '{}' for tag '{}', expected one of {:?}",
        provided, tag_name, options
      ),
      range,
    ),
    ValidateError::UnknownTag {
      tag_name,
      available,
      range,
    } => FormattedError::new(
      format!("Unknown tag '{}', found tags: {:?}", tag_name, available),
      range,
    ),
    ValidateError::UnknownGameValue { game_value, range } => {
      FormattedError::new(format!("Unknown game_value '{game_value}'"), range)
    }
  }
}
