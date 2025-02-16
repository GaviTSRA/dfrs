use crate::compile::{ArgValueData, Block, Codeline, FunctionDefaultItemData};
use crate::definitions::action_dump::{ActionDump, RawActionDump};
use crate::definitions::actions::Action;
use crate::definitions::{ArgType, DefinedArg, DefinedArgBranch, DefinedArgOption};
use crate::node::{ActionType, ConditionalType};
use crate::token::{Selector, SELECTORS};
use crate::utility::{to_camel_case, to_dfrs_name};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::{Cursor, Read};

fn decompress(compressed_code: &str) -> String {
  let compressed_data = match BASE64_STANDARD.decode(compressed_code) {
    Ok(data) => data,
    Err(err) => panic!("Failed to decode base64: {}", err),
  };

  let mut decoder = GzDecoder::new(Cursor::new(compressed_data));
  let mut decompressed_data = String::new();

  match decoder.read_to_string(&mut decompressed_data) {
    Ok(_) => {}
    Err(err) => panic!("Failed to decompress data: {}", err),
  }

  decompressed_data
}

pub struct Decompiler {
  indentation: i32,
  action_dump: ActionDump,
  vars: HashMap<String, String>,
  result: String,
}

impl Decompiler {
  pub fn new() -> Decompiler {
    let ad = RawActionDump::load();
    Decompiler {
      indentation: 0,
      action_dump: ActionDump::new(&ad),
      vars: HashMap::new(),
      result: String::new(),
    }
  }

  fn add(&mut self, line: &str) {
    let indentation = " ".repeat((self.indentation * 2) as usize);
    self.result.push_str(&format!("{indentation}{line}\n"));
  }

  fn indent(&mut self) {
    self.indentation += 1;
  }

  fn unindent(&mut self) {
    self.indentation -= 1;
  }

  fn set_var(&mut self, old_name: &str, new_name: &str) {
    self.vars.insert(old_name.to_string(), new_name.to_string());
  }

  pub fn decompile(&mut self, code: &str) -> String {
    let json = decompress(code);
    let line: Codeline = serde_json::from_str(&json).unwrap();
    let mut global_vars = vec![];
    let mut vars = vec![];
    let mut direct_vars = vec![];

    for block in &line.blocks {
      if let Some(args) = &block.args.clone() {
        for (arg_index, arg) in args.items.iter().enumerate() {
          match &arg.item.data {
            ArgValueData::Variable { name, scope } => {
              let new_name = name
                .replace("-", "_")
                .replace("%", "")
                .replace(" ", "_")
                .replace("(", "_")
                .replace(")", "");

              let var = if &new_name != name {
                self.set_var(name, &new_name);
                format!("{new_name}: `{name}`")
              } else {
                self.set_var(name, name);
                name.to_string()
              };

              let mut is_normal_action = false;
              if let Some(block) = &block.block {
                is_normal_action = match block.as_str() {
                  "player_action" => true,
                  "entity_action" => true,
                  "game_action" => true,
                  "set_var" => true,
                  _ => false,
                };
              }

              match scope.as_str() {
                "unsaved" => global_vars.push(format!("game {var};")),
                "saved" => global_vars.push(format!("save {var};")),
                "local" => {
                  if arg_index == 0 && is_normal_action && !vars.contains(&format!("local {var};"))
                  {
                    direct_vars.push(format!("local {var};"));
                  }
                  vars.push(format!("local {var};"))
                }
                "line" => {
                  if arg_index == 0 && is_normal_action && !vars.contains(&format!("line {var};")) {
                    direct_vars.push(format!("line {var};"));
                  }
                  vars.push(format!("line {var};"))
                }

                err => println!("ERR: Unknown var type {err}"),
              }
            }
            _ => {}
          }
        }
      }
    }

    global_vars.sort();
    global_vars.dedup();
    for var in global_vars {
      self.add(&var);
    }

    vars.sort();
    vars.dedup();

    for block in line.blocks {
      match block.id.as_str() {
        "block" => {
          self.decompile_block(block, vars.clone(), direct_vars.clone());
        }
        "bracket" => {
          self.decompile_bracket(block);
        }
        other => {
          println!("WARN: Unhandled block id {other}")
        }
      }
    }
    self.unindent();
    self.add("}");
    self.result.clone()
  }

  fn decompile_bracket(&mut self, block: Block) {
    match block.direct.unwrap().as_str() {
      "open" => {
        self.indent();
      }
      "close" => {
        self.unindent();
        self.add("}");
      }
      _ => {}
    }
  }

  fn decompile_block(&mut self, block: Block, vars: Vec<String>, direct_vars: Vec<String>) {
    if let Some(block_name) = block.block.clone() {
      match block_name.as_str() {
        "event" => {
          self.decompile_event(block, vars, direct_vars);
        }
        "func" => {
          self.decompile_function(block, vars, direct_vars);
        }
        "process" => {
          self.decompile_process(block, vars, direct_vars);
        }
        "player_action" => {
          self.decompile_action(block, ActionType::Player);
        }
        "entity_action" => {
          self.decompile_action(block, ActionType::Entity);
        }
        "game_action" => {
          self.decompile_action(block, ActionType::Game);
        }
        "set_var" => {
          self.decompile_action(block, ActionType::Variable);
        }
        "control" => {
          self.decompile_action(block, ActionType::Control);
        }
        "select_obj" => {
          self.decompile_action(block, ActionType::Select);
        }
        "if_player" => {
          self.decompile_conditional(block, ConditionalType::Player);
        }
        "if_entity" => {
          self.decompile_conditional(block, ConditionalType::Entity);
        }
        "if_game" => {
          self.decompile_conditional(block, ConditionalType::Game);
        }
        "if_var" => {
          self.decompile_conditional(block, ConditionalType::Variable);
        }
        "repeat" => {
          self.decompile_repeat(block);
        }
        "else" => {
          self.add("else {");
        }
        "call_func" => {
          self.decompile_call(block);
        }
        "start_process" => {
          self.decompile_start(block);
        }
        other => {
          println!("WARN: Unhandled block block {other}")
        }
      }
    }
  }

  fn decompile_event(&mut self, block: Block, vars: Vec<String>, direct_vars: Vec<String>) {
    let extra = if block.attribute.is_some() && block.attribute.unwrap() == "LS-CANCEL" {
      "!"
    } else {
      ""
    };
    self.add(&format!(
      "@{}{extra} {{",
      to_dfrs_name(&block.action.unwrap())
    ));
    self.indent();
    for var in vars {
      if direct_vars.contains(&var) {
        continue;
      }
      self.add(&var);
    }
  }

  fn decompile_function(&mut self, block: Block, vars: Vec<String>, direct_vars: Vec<String>) {
    let mut result = String::from("");
    if let Some(args) = block.args {
      let mut is_first_iter = true;
      for arg in args.items {
        match arg.item.data {
          ArgValueData::FunctionParam {
            name,
            optional,
            plural,
            param_type,
            default_value,
          } => {
            let is_optional = if optional { "?" } else { "" };
            let is_plural = if plural { "*" } else { "" };
            let default = if let Some(default_val) = default_value {
              let end = match default_val.data {
                FunctionDefaultItemData::Simple { name } => match default_val.id.as_str() {
                  "comp" => format!("\"{name}\""),
                  "num" => format!("{name}"),
                  "txt" => format!("'{name}'"),
                  other => {
                    println!("ERR: Unhandled simple function arg {other}");
                    "".into()
                  }
                },
                FunctionDefaultItemData::Id { .. } => "".into(),
                FunctionDefaultItemData::Location { loc, .. } => {
                  let mut res_loc = format!("Location({}, {}, {}", loc.x, loc.y, loc.z);
                  if let Some(pitch) = loc.pitch {
                    res_loc.push_str(&format!(", {}", pitch));
                  }
                  if let Some(yaw) = loc.yaw {
                    res_loc.push_str(&format!(", {}", yaw));
                  }
                  res_loc.push_str(")");
                  res_loc
                }
                FunctionDefaultItemData::Vector { x, y, z } => {
                  format!("Vector({x}, {y}, {z})")
                }
                FunctionDefaultItemData::Sound {
                  sound,
                  pitch,
                  volume,
                } => {
                  format!("Sound(\"{sound}\", {pitch}, {volume})")
                }
                FunctionDefaultItemData::Potion {
                  potion,
                  amplifier,
                  duration,
                } => {
                  format!("Potion(\"{potion}\", {amplifier}, {duration})")
                }
                FunctionDefaultItemData::Particle {
                  particle,
                  cluster,
                  data,
                } => {
                  // TODO
                  "".into()
                }
              };
              format!("={end}")
            } else {
              "".into()
            };
            if !is_first_iter {
              result.push_str(", ");
            } else {
              is_first_iter = false;
            }
            let value_type = match param_type.as_str() {
              "txt" => "string",
              "comp" => "text",
              "num" => "number",
              "loc" => "location",
              "vec" => "vector",
              "snd" => "sound",
              "par" => "particle",
              "pot" => "potion",
              "item" => "item",
              "any" => "any",
              "var" => "variable",
              "list" => "list",
              "dict" => "dict",
              err => panic!("unknown param type '{err}'"),
            };
            result.push_str(&format!(
              "{name}: {value_type}{is_optional}{is_plural}{default}"
            ))
          }
          ArgValueData::Id { .. } => {}
          ArgValueData::Tag { .. } => {}
          ArgValueData::Item { .. } => {}
          other => panic!("Found {other:?}"),
        }
      }
    }
    let name = block.data.clone().unwrap();
    let new_name = name
      .replace("-", "_")
      .replace("%", "")
      .replace(" ", "_")
      .replace("(", "_")
      .replace(")", "");
    if new_name != name {
      self.add(&format!("fn {}: `{}`({}) {{", new_name, name, result));
    } else {
      self.add(&format!("fn {}({}) {{", new_name, result));
    }
    self.indent();
    for var in vars {
      if direct_vars.contains(&var) {
        continue;
      }
      self.add(&var);
    }
  }

  fn decompile_process(&mut self, block: Block, vars: Vec<String>, direct_vars: Vec<String>) {
    self.add(&format!("proc {} {{", &block.data.unwrap()));
    self.indent();
    for var in vars {
      if direct_vars.contains(&var) {
        continue;
      }
      self.add(&var);
    }
  }

  fn decompile_action(&mut self, block: Block, action_type: ActionType) {
    let name = to_dfrs_name(&block.action.clone().unwrap());
    let action = match match action_type {
      ActionType::Player => self.action_dump.player_actions.get(&name),
      ActionType::Entity => self.action_dump.entity_actions.get(&name),
      ActionType::Game => self.action_dump.game_actions.get(&name),
      ActionType::Variable => self.action_dump.variable_actions.get(&name),
      ActionType::Control => self.action_dump.control_actions.get(&name),
      ActionType::Select => self.action_dump.select_actions.get(&name),
    } {
      Some(res) => res,
      None => {
        println!("ERROR DECOMPILING ACTION: {action_type:?} {name:?}");
        return;
      }
    };
    let prefix = match action_type {
      ActionType::Player => "p",
      ActionType::Entity => "e",
      ActionType::Game => "g",
      ActionType::Variable => "v",
      ActionType::Control => "c",
      ActionType::Select => "s",
    };
    let selector = match block.target.clone() {
      Some(res) => &format!(":{}", SELECTORS.entries().find(|e| e.1 == &res).unwrap().0),
      None => "",
    };

    if let Some(mut args) = block.args.clone() {
      if args.items.len() > 0 {
        let arg = args.items.get(0).unwrap();
        match arg.item.data.clone() {
          ArgValueData::Variable {
            name: var_name,
            scope,
          } => {
            if scope == "line" || scope == "local" {
              let new_name = var_name
                .replace("-", "_")
                .replace("%", "")
                .replace(" ", "_")
                .replace("(", "_")
                .replace(")", "");

              let var_assignment = if new_name == var_name {
                ""
              } else {
                &format!(": `{var_name}`")
              };

              let mut is_normal_action = false;
              if let Some(block) = &block.block {
                is_normal_action = match block.as_str() {
                  "player_action" => true,
                  "entity_action" => true,
                  "game_action" => true,
                  "set_var" => true,
                  _ => false,
                };
              }

              if is_normal_action {
                let mut new_block = block.clone();
                args.items.remove(0);
                new_block.args = Some(args);
                return self.add(&format!(
                  "{scope} {new_name}{var_assignment} = {prefix}{selector}.{}({});",
                  name,
                  self.decompile_params(new_block, action)
                ));
              }

              return self.add(&format!(
                "{prefix}{selector}.{}({});",
                name,
                self.decompile_params(block, action)
              ));
            }
          }
          _ => {}
        }
      }
    }

    self.add(&format!(
      "{prefix}{selector}.{}({});",
      name,
      self.decompile_params(block, action)
    ))
  }

  fn decompile_conditional(&mut self, block: Block, conditional_type: ConditionalType) {
    let name = to_dfrs_name(&block.action.clone().unwrap());
    let action = match conditional_type {
      ConditionalType::Player => self.action_dump.player_conditionals.get(&name),
      ConditionalType::Entity => self.action_dump.entity_conditionals.get(&name),
      ConditionalType::Game => self.action_dump.game_conditionals.get(&name),
      ConditionalType::Variable => self.action_dump.variable_conditionals.get(&name),
    }
    .unwrap()
    .clone();
    let prefix = match conditional_type {
      ConditionalType::Player => "ifp",
      ConditionalType::Entity => "ife",
      ConditionalType::Game => "ifg",
      ConditionalType::Variable => "ifv",
    };
    let selector = match block.target.clone() {
      Some(res) => &format!("{}:", SELECTORS.entries().find(|e| e.1 == &res).unwrap().0),
      None => "",
    };
    let inverted =
      if block.attribute.is_some() && block.attribute.clone().unwrap() == "NOT".to_string() {
        "!"
      } else {
        ""
      };
    self.add(&format!(
      "{prefix} {inverted}{selector}{}({}) {{",
      name,
      self.decompile_params(block, &action)
    ))
  }

  fn decompile_repeat(&mut self, block: Block) {
    let name = to_dfrs_name(&block.action.clone().unwrap());
    let action = self.action_dump.repeats.get(&name).unwrap().clone();
    self.add(&format!(
      "repeat {}({}) {{",
      name,
      self.decompile_params(block, &action)
    ))
  }

  fn decompile_call(&mut self, block: Block) {
    let mut args = vec![];
    for _ in &block.args {
      args.push(DefinedArg {
        options: vec![DefinedArgOption::new(
          String::from(""),
          ArgType::ANY,
          false,
          false,
        )],
      })
    }
    let action = &Action {
      df_name: "internal".into(),
      dfrs_name: "internal".into(),
      aliases: vec![],
      args: vec![DefinedArgBranch { paths: vec![args] }],
      tags: vec![],
      has_conditional_arg: false,
    };
    if block.args.is_some() && block.args.clone().unwrap().items.len() > 0 {
      self.add(&format!(
        "call(\"{}\", {});",
        to_dfrs_name(&block.data.clone().unwrap()),
        self.decompile_params(block.clone(), action)
      ));
    } else {
      self.add(&format!(
        "call(\"{}\");",
        to_dfrs_name(&block.data.clone().unwrap())
      ));
    }
  }

  fn decompile_start(&mut self, block: Block) {
    let params = self.decompile_params(
      block.clone(),
      &self.action_dump.start_process_action.clone(),
    );
    if &params == "" {
      self.add(&format!(
        "start(\"{}\");",
        to_dfrs_name(&block.data.clone().unwrap())
      ));
    } else {
      self.add(&format!(
        "start(\"{}\", {});",
        to_dfrs_name(&block.data.clone().unwrap()),
        params
      ));
    }
  }

  fn decompile_params(&self, block: Block, action: &Action) -> String {
    let mut result = String::from("");

    if let Some(sub_action_name) = block.sub_action.clone() {
      if let Some(action) = &self
        .action_dump
        .player_conditionals
        .get_df(&sub_action_name)
      {
        result.push_str(&format!("ifp {}(", action.dfrs_name));
      } else if let Some(action) = &self
        .action_dump
        .entity_conditionals
        .get_df(&sub_action_name)
      {
        result.push_str(&format!("ife {}(", action.dfrs_name));
      } else if let Some(action) = &self.action_dump.game_conditionals.get_df(&sub_action_name) {
        result.push_str(&format!("ifg {}(", action.dfrs_name));
      } else if let Some(action) = &self
        .action_dump
        .variable_conditionals
        .get_df(&sub_action_name)
      {
        result.push_str(&format!("ifv {}(", action.dfrs_name));
      } else {
        println!("ERR decompiling subaction {}", sub_action_name);
      }
    }

    if let Some(args) = block.args {
      let mut is_first_iter = true;
      for arg in args.items {
        match arg.item.data {
          ArgValueData::Tag { .. } => {}
          _ => {
            if !is_first_iter {
              result.push_str(", ");
            } else {
              is_first_iter = false;
            }
          }
        }
        match arg.item.data {
          ArgValueData::Simple { name } => match arg.item.id.as_str() {
            "comp" => result.push_str(&format!("\"{}\"", name.replace("\"", "\\\""))),
            "num" => {
              let mut done = false;
              for char in name.clone().chars() {
                if !char.is_numeric() {
                  result.push_str(&format!("Number(\"{name}\")"));
                  done = true;
                  break;
                }
              }
              if !done {
                result.push_str(&format!("{name}"))
              }
            }
            "txt" => result.push_str(&format!("'{}'", name.replace("'", "\\'"))),
            other => println!("WARN: Unhandled simple arg {other}"),
          },
          ArgValueData::Id { .. } => {}
          ArgValueData::Item { item } => {
            result.push_str(&format!("Item(\"{}\")", item.replace("\"", "\\\"")));
          }
          ArgValueData::GameValue { game_value, target } => {
            let selector = if target == Selector::Default {
              ""
            } else {
              &format!(
                "{}:",
                SELECTORS.entries().find(|e| e.1 == &target).unwrap().0
              )
            };
            result.push_str(&format!("${selector}{}", to_dfrs_name(&game_value)))
          }
          ArgValueData::Variable { name, .. } => {
            result.push_str(&format!("{}", self.vars.get(&name).unwrap()))
          }
          ArgValueData::Location { loc, .. } => {
            let mut res_loc = format!("Location({}, {}, {}", loc.x, loc.y, loc.z);
            if let Some(pitch) = loc.pitch {
              res_loc.push_str(&format!(", {}", pitch));
            }
            if let Some(yaw) = loc.yaw {
              res_loc.push_str(&format!(", {}", yaw));
            }
            res_loc.push_str(")");
            result.push_str(&res_loc);
          }
          ArgValueData::Vector { x, y, z } => {
            result.push_str(&format!("Vector({x}, {y}, {z})"));
          }
          ArgValueData::Sound {
            sound,
            pitch,
            volume,
          } => {
            result.push_str(&format!("Sound(\"{sound}\", {pitch}, {volume})"));
          }
          ArgValueData::Potion {
            potion,
            amplifier,
            duration,
          } => {
            result.push_str(&format!("Potion(\"{potion}\", {amplifier}, {duration})"));
          }
          ArgValueData::Tag { tag, option, .. } => {
            for action_tag in &action.tags {
              if &action_tag.df_name == &tag {
                if &option != &action_tag.default {
                  if !is_first_iter {
                    result.push_str(", ");
                  } else {
                    is_first_iter = false;
                  }
                  result.push_str(&format!("{}=\"{option}\"", to_camel_case(&tag)));
                }
                break;
              }
            }
          }
          ArgValueData::FunctionParam { .. } => {}
          ArgValueData::Particle {
            particle,
            cluster,
            data,
          } => {
            let mut tags = String::new();
            if let (Some(x), Some(y), Some(z)) = (data.x, data.y, data.z) {
              tags.push_str(&format!(", motion=Vector({x},{y},{z})"))
            }
            if let Some(motion_variation) = data.motion_variation {
              tags.push_str(&format!(", motionVariation={motion_variation}"))
            }
            if let Some(rgb) = data.rgb {
              tags.push_str(&format!(", rgb={rgb}"))
            }
            if let Some(rgb_fade) = data.rgb_fade {
              tags.push_str(&format!(", rgb_fade={rgb_fade}"))
            }
            if let Some(color_variation) = data.color_variation {
              tags.push_str(&format!(", colorVariation={color_variation}"))
            }
            if let Some(material) = data.material {
              tags.push_str(&format!(", material=\"{material}\""))
            }
            if let Some(size) = data.size {
              tags.push_str(&format!(", size={size}"))
            }
            if let Some(size_variation) = data.size_variation {
              tags.push_str(&format!(", sizeVariation={size_variation}"))
            }
            if let Some(roll) = data.roll {
              tags.push_str(&format!(", roll={roll}"))
            }

            result.push_str(&format!(
              "Particle(\"{particle}\", {}, {}, {}{tags})",
              cluster.amount, cluster.horizontal, cluster.vertical
            ))
          }
        }
      }
    }
    if block.sub_action.is_some() {
      result.push_str(")");
    }
    result
  }
}
