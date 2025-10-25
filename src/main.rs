use crate::compile::compile;
use crate::config::Config;
use crate::decompile::Decompiler;
use crate::errors::{
  format_lexer_error, format_parser_error, format_validator_error, FormattedError,
};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::send::send;
use crate::validate::Validator;
use clap::{Parser as _, Subcommand};
use colored::Colorize;
use lsp::run_lsp;
use std::path::PathBuf;
use std::time::Instant;
use std::{cmp, fs};
use tungstenite::{connect, Message};
use url::Url;

pub mod compile;
pub mod config;
pub mod decompile;
pub mod definitions;
pub mod errors;
pub mod lexer;
mod lsp;
mod minimessage;
pub mod node;
pub mod parser;
pub mod send;
pub mod token;
pub mod utility;
pub mod validate;

pub struct ConfigFileNotFoundError {}

pub fn load_config(file: &PathBuf) -> Result<Config, ConfigFileNotFoundError> {
  let data = if !file.exists() {
    return Err(ConfigFileNotFoundError {});
  } else {
    std::fs::read_to_string(file).expect("No config file")
  };

  match toml::from_str(&data) {
    Ok(res) => Ok(res),
    Err(err) => panic!("Failed to parse config: {}", err),
  }
}

fn print_err(error: FormattedError, data: String) {
  let lines = data.split("\n").collect::<Vec<&str>>();
  let line = lines.get((error.start.line - 1) as usize).unwrap();
  let ln = error.start.line;
  let ln_length = ln.to_string().chars().count();

  println!(
    "{} {} at {}",
    "Error:".bright_red(),
    error.message,
    error.start
  );
  println!("{} {}", " ".repeat(ln_length), "|".bright_black());
  println!(
    "{} {} {}",
    ln.to_string().bright_black(),
    "|".bright_black(),
    line
  );
  let arrows;
  match error.end {
    Some(end_pos) => {
      if end_pos.line != error.start.line {
        // TODO
        return;
      }
      arrows = "^"
        .repeat(cmp::max(end_pos.col - error.start.col, 1) as usize)
        .bright_blue();
    }
    None => {
      arrows = "^".bright_blue();
    }
  }
  println!(
    "{} {} {}{}",
    " ".repeat(ln_length),
    "|".bright_black(),
    " ".repeat((error.start.col - 1) as usize),
    arrows
  );
}

fn compile_cmd(file: &PathBuf) {
  println!(
    "{} {}",
    "Compiling".bright_black(),
    file.file_name().unwrap().to_string_lossy()
  );
  let start = Instant::now();

  let mut config_file = file.clone();
  config_file.set_file_name("dfrs.toml");
  let config = match load_config(&config_file) {
    Ok(res) => res,
    Err(_) => {
      println!("{} No config file found", "Error:".bright_red());
      println!(
        "{} dfrs init <path> {}",
        "Use".bright_black(),
        "to create a new config file".bright_black()
      );
      return;
    }
  };

  let data = fs::read_to_string(file).expect("could not open file");

  let input = &data.clone();
  let mut lexer = Lexer::new(input);
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
    Err(error) => {
      print_err(format_lexer_error(error), data);
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
              node::Expression::Action { node } => {
                println!(
                  "{:?} {:?} {:?} {:?}",
                  node.action_type, node.selector, node.name, node.args
                )
              }
              node::Expression::Conditional { node } => {
                println!(
                  "{:?} {:?} {:?} {:?}",
                  node.conditional_type, node.selector, node.name, node.args
                )
              }
              node::Expression::Call { node } => {
                println!("{:?} {:?}", node.name, node.args)
              }
              node::Expression::Start { node } => {
                println!("{:?} {:?}", node.name, node.args)
              }
              node::Expression::Repeat { node } => {
                println!("{:?} {:?}", node.name, node.args)
              }
              node::Expression::Variable { node } => {
                println!(
                  "{:?} {:?} {:?}",
                  node.var_type, node.dfrs_name, node.df_name
                )
              }
            }
          }
        }
        println!("\n");
        for function in &res.functions {
          println!("{} / {}", function.dfrs_name, function.df_name);
          for param in &function.params {
            println!("{:?}", param);
          }
          for expression in &function.expressions {
            match &expression.node {
              node::Expression::Action { node } => {
                println!(
                  "{:?} {:?} {:?} {:?}",
                  node.action_type, node.selector, node.name, node.args
                )
              }
              node::Expression::Conditional { node } => {
                println!(
                  "{:?} {:?} {:?} {:?}",
                  node.conditional_type, node.selector, node.name, node.args
                )
              }
              node::Expression::Call { node } => {
                println!("{:?} {:?}", node.name, node.args)
              }
              node::Expression::Start { node } => {
                println!("{:?} {:?}", node.name, node.args)
              }
              node::Expression::Repeat { node } => {
                println!("{:?} {:?}", node.name, node.args)
              }
              node::Expression::Variable { node } => {
                println!(
                  "{:?} {:?} {:?}",
                  node.var_type, node.dfrs_name, node.df_name
                )
              }
            }
          }
        }
        println!("\n");
      }
      node = res;
    }
    Err(error) => {
      print_err(format_parser_error(error), data);
      std::process::exit(0);
    }
  }

  let validated = match Validator::new().validate(node) {
    Ok(res) => res,
    Err(error) => {
      print_err(format_validator_error(error), data);
      std::process::exit(0);
    }
  };

  let compiled = compile(validated, config.debug.compile);
  let duration = start.elapsed().as_secs_f64();
  println!(
    "{}  {} {} {}",
    "Compiled".green(),
    file.file_name().unwrap().to_string_lossy(),
    "in".bright_black(),
    format!("{:.2}s", duration).bright_black()
  );
  send(compiled, config);
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
  Compile { path: PathBuf },
  Init { path: PathBuf },
  Decompile { code: String, file: Option<PathBuf> },
  DecompilePlot { file: Option<PathBuf> },
  LSP {},
}

fn main() {
  let cli = Cli::parse();

  match &cli.command {
    Some(Commands::Compile { path }) => {
      if !path.exists() {
        println!("{} File not found", "Error:".bright_red());
        return;
      }
      if path.is_dir() {
        let paths = fs::read_dir(path).unwrap();

        println!(
          "{} {}",
          "Compiling project".bright_black(),
          path
            .canonicalize()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
        );
        for path in paths {
          let file = path.unwrap().path();
          if file.is_file() && file.extension().unwrap() == "dfrs" {
            compile_cmd(&file);
          }
        }
      } else {
        compile_cmd(path);
      }
    }
    Some(Commands::Init { path }) => {
      if !path.exists() {
        println!("{} File not found", "Error:".bright_red());
        return;
      }
      if !path.is_dir() {
        println!("{} Path is not a directory", "Error:".bright_red());
        return;
      }
      println!(
        "{} {}",
        "Initializing new project in".bright_black(),
        path.to_string_lossy()
      );
      let new_config = Config::default();
      let mut config_path = path.clone();
      config_path.push("dfrs.toml");
      new_config.save(&config_path);
      println!(
        "{} {}",
        "Created new config".green(),
        config_path.to_string_lossy()
      );
    }
    Some(Commands::Decompile { code, file }) => {
      let mut decompiler = Decompiler::new();
      let result = decompiler.decompile(code);
      if let Some(file) = file {
        fs::write(file, result).expect("Failed to write file");
      } else {
        println!("{}", result)
      }
    }
    Some(Commands::DecompilePlot { file }) => {
      let (mut socket, response) =
        connect(Url::parse("ws://localhost:31375").unwrap()).expect("Can't connect");
      socket
        .send(Message::Text("scopes read_plot".into()))
        .unwrap();

      let msg = socket.read().expect("Error reading message");
      socket.send(Message::Text("scan".into())).unwrap();
      let msg = socket.read().expect("Error reading message");

      let mut result = String::new();
      for line in msg.to_text().unwrap().split('\n') {
        let mut decompiler = Decompiler::new();
        result.push_str(&decompiler.decompile(line));
        result.push_str("\n");
      }

      if let Some(file) = file {
        fs::write(file, result).expect("Failed to write file");
      } else {
        println!("{}", result)
      }
    }
    Some(Commands::LSP {}) => {
      run_lsp();
    }
    None => {}
  }
}
