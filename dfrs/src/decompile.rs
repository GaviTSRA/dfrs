use std::io::{Cursor, Read};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use flate2::read::GzDecoder;
use crate::compile::{ArgValueData, Block, Codeline, FunctionDefaultItemData};
use crate::definitions::action_dump::ActionDump;
use crate::definitions::actions::{ControlActions, EntityActions, GameActions, PlayerActions, SelectActions, VariableActions};
use crate::definitions::conditionals::{EntityConditionals, GameConditionals, PlayerConditionals};
use crate::definitions::repeats::Repeats;
use crate::node::{ActionType, ConditionalType};
use crate::token::{Selector, SELECTORS};
use crate::utility::{to_camel_case, to_dfrs_name};

fn decompress(compressed_code: &str) -> String {
    let compressed_data = match BASE64_STANDARD.decode(compressed_code) {
        Ok(data) => data,
        Err(err) => panic!("Failed to decode base64: {}", err),
    };

    let mut decoder = GzDecoder::new(Cursor::new(compressed_data));
    let mut decompressed_data = String::new();

    match decoder.read_to_string(&mut decompressed_data) {
        Ok(_) => {},
        Err(err) => panic!("Failed to decompress data: {}", err),
    }

    decompressed_data
}

pub struct Decompiler {
    indentation: i32,

    player_actions: PlayerActions,
    entity_actions: EntityActions,
    game_actions: GameActions,
    variable_actions: VariableActions,
    control_actions: ControlActions,
    select_actions: SelectActions,

    player_conditionals: PlayerConditionals,
    entity_conditionals: EntityConditionals,
    game_conditionals: GameConditionals,

    repeats: Repeats
}

impl Decompiler {
    pub fn new() -> Decompiler {
        let ad = ActionDump::load();
        Decompiler {
            indentation: 0,

            player_actions: PlayerActions::new(&ad),
            entity_actions: EntityActions::new(&ad),
            game_actions: GameActions::new(&ad),
            variable_actions: VariableActions::new(&ad),
            control_actions: ControlActions::new(&ad),
            select_actions: SelectActions::new(&ad),

            player_conditionals: PlayerConditionals::new(&ad),
            entity_conditionals: EntityConditionals::new(&ad),
            game_conditionals: GameConditionals::new(&ad),

            repeats: Repeats::new(&ad)
        }
    }

    fn add(&self, line: &str) {
        let indentation = " ".repeat((self.indentation*2) as usize);
        println!("{indentation}{line}");
    }

    fn indent(&mut self) {
        self.indentation += 1;
    }

    fn unindent(&mut self) {
        self.indentation -= 1;
    }

    pub fn decompile(&mut self, code: &str) {
        let json = decompress(code);
        let line: Codeline = serde_json::from_str(&json).unwrap();
        let mut vars = vec![];

        for block in &line.blocks {
            if let Some(args) = &block.args {
                for arg in &args.items {
                    match &arg.item.data {
                        ArgValueData::Variable { name, scope} => {
                            match scope.as_str() {
                                "unsaved" => println!("define GAME var {name} !"),
                                "saved" => println!("define SAVE var {name} !"),
                                "local" => vars.push(format!("local {name} = `{name}`;")),
                                "line" => vars.push(format!("line {name} = `{name}`;")),
                                err => println!("ERR: Unknown var type {err}")
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        vars.sort();
        vars.dedup();

        for block in line.blocks {
            match block.id.as_str() {
                "block" => {
                    self.decompile_block(block, vars.clone());
                },
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

    fn decompile_block(&mut self, block: Block, vars: Vec<String>) {
        if let Some(block_name) = block.block.clone() {
            match block_name.as_str() {
                "event" => {
                    self.decompile_event(block, vars);
                }
                "func" => {
                    self.decompile_function(block, vars);
                }
                "process" => {
                    self.decompile_process(block, vars);
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

    fn decompile_event(&mut self, block: Block, vars: Vec<String>) {
        let extra = if block.attribute.is_some() && block.attribute.unwrap() == "LS-CANCEL" {
            "!"
        } else {
            ""
        };
        self.add(&format!("@{}{extra} {{", to_dfrs_name(&block.action.unwrap())));
        self.indent();
        for var in vars {
            self.add(&var);
        }
    }

    fn decompile_function(&mut self, block: Block, vars: Vec<String>) {
        let mut result = String::from("");
        if let Some(args) = block.args {
            let mut is_first_iter = true;
            for arg in args.items {
                match arg.item.data {
                    ArgValueData::FunctionParam { name, optional, plural, param_type, default_value} => {
                        let is_optional = if optional { "?" } else { "" };
                        let is_plural = if plural { "*" } else { "" };
                        let default = if let Some(default_val) = default_value {
                            let end = match default_val.data {
                                FunctionDefaultItemData::Simple { name } => {
                                    match arg.item.id.as_str() {
                                        "comp" => format!("\"{name}\""),
                                        "num" => format!("{name}"),
                                        "txt" => format!("'{name}'"),
                                        other => panic!("ERR: Unhandled simple function arg {other}")
                                    }
                                }
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
                                FunctionDefaultItemData::Sound { sound, pitch, volume } => {
                                    format!("Sound(\"{sound}\", {pitch}, {volume})")
                                }
                                FunctionDefaultItemData::Potion { potion, amplifier, duration } => {
                                    format!("Potion(\"{potion}\", {amplifier}, {duration})")
                                }
                            };
                            format!("={end}")
                        } else { "".into() };
                        if !is_first_iter {
                            result.push_str(", ");
                        } else {
                            is_first_iter = false;
                        }
                        let value_type = match param_type.as_str() {
                            "str" => "string",
                            "txt" => "text",
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
                            _ => panic!("unkown param type")
                        };
                        result.push_str(&format!("{name}: {value_type}{is_optional}{is_plural}{default}"))
                    }
                    ArgValueData::Id { .. } => {}
                    ArgValueData::Tag { .. } => {}
                    other => panic!("Found {other:?}")
                }
            }
        }
        self.add(&format!("fn {}({}) {{", block.data.unwrap(), result));
        self.indent();
        for var in vars {
            self.add(&var);
        }
    }

    fn decompile_process(&mut self, block: Block, vars: Vec<String>) {
        self.add(&format!("proc {} {{", &block.data.unwrap()));
        self.indent();
        for var in vars {
            self.add(&var);
        }
    }

    fn decompile_action(&self, block: Block, action_type: ActionType) {
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
            None => ""
        };
        self.add(&format!("{prefix}{selector}.{}({});", to_dfrs_name(&block.action.clone().unwrap()).replace("+=", "addDirect").replace("-=", "subDirect")
            .replace('+', "add").replace('-', "sub")
            .replace('%', "mod").replace('/', "div").replace('=', "equal"), self.decompile_params(block)))
    }

    fn decompile_conditional(&self, block: Block, conditional_type: ConditionalType) {
        let prefix = match conditional_type {
            ConditionalType::Player => "ifp",
            ConditionalType::Entity => "ife",
            ConditionalType::Game => "ifg",
            ConditionalType::Variable => "ifv"
        };
        let selector = match block.target.clone() {
            Some(res) => &format!("{}:", SELECTORS.entries().find(|e| e.1 == &res).unwrap().0),
            None => ""
        };
        self.add(&format!("{prefix} {selector}{}({}) {{", to_dfrs_name(&block.action.clone().unwrap()).replace('=', "equal"), self.decompile_params(block)))
    }

    fn decompile_repeat(&self, block: Block) {
        self.add(&format!("repeat {}({}) {{", to_dfrs_name(&block.action.clone().unwrap()), self.decompile_params(block)))
    }

    fn decompile_call(&self, block: Block) {
        self.add(&format!("call(\"{}\", {});", to_dfrs_name(&block.data.clone().unwrap()), self.decompile_params(block)));
    }

    fn decompile_start(&self, block: Block) {
        self.add(&format!("start(\"{}\", {});", to_dfrs_name(&block.data.clone().unwrap()), self.decompile_params(block)));
    }

    fn decompile_params(&self, block: Block) -> String {
        let mut result = String::from("");
        if let Some(args) = block.args {
            let mut is_first_iter = true;
            for arg in args.items {
                if !is_first_iter {
                    result.push_str(", ");
                } else {
                    is_first_iter = false;
                }
                match arg.item.data {
                    ArgValueData::Simple { name } => {
                        match arg.item.id.as_str() {
                            "comp" => result.push_str(&format!("\"{name}\"")),
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
                            },
                            "txt" => result.push_str(&format!("'{name}'")),
                            other => println!("WARN: Unhandled simple arg {other}")
                        }
                    }
                    ArgValueData::Id { .. } => {}
                    ArgValueData::Item { item } => {
                        result.push_str(&format!("Item(\"{}\")", item.replace("\"", "\\\"")));
                    }
                    ArgValueData::GameValue { game_value, target } => {
                        let selector = if target == Selector::Default {
                            ""
                        } else {
                            &format!("{}:", SELECTORS.entries().find(|e| e.1 == &target).unwrap().0)
                        };
                        result.push_str(&format!("${selector}{}", to_dfrs_name(&game_value)))
                    }
                    ArgValueData::Variable { name, .. } => result.push_str(&format!("{name}")),
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
                    ArgValueData::Sound { sound, pitch, volume } => {
                        result.push_str(&format!("Sound(\"{sound}\", {pitch}, {volume})"));
                    }
                    ArgValueData::Potion { potion, amplifier, duration } => {
                        result.push_str(&format!("Potion(\"{potion}\", {amplifier}, {duration})"));
                    }
                    ArgValueData::Tag { tag, option, .. } => {
                        result.push_str(&format!("{}=\"{option}\"", to_camel_case(&tag)));
                    }
                    ArgValueData::FunctionParam { .. } => {}
                }
            }
        }
        result
    }
}
