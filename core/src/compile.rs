use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use crate::{node::{ActionNode, ActionType, ConditionalNode, ConditionalType, EventNode, Expression, FileNode, FunctionNode}, token::{get_type_str, Selector}};

pub fn compile(node: FileNode, debug: bool) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for function in node.functions.clone() {
        match function_node(function) {
            Ok(result) => {
                res.push(result.clone());
                if debug {
                    println!("{:?}", result);
                }
            }
            Err(err) => {
                panic!("Failed to compile: {}", err)
            }
        }
    }
    for event in node.events.clone() {
        match event_node(event) {
            Ok(result) => {
                res.push(result.clone());
                if debug {
                    println!("{:?}", result);
                }
            }
            Err(err) => {
                panic!("Failed to compile: {}", err)
            }
        }
    }
    res
}

fn event_node(event_node: EventNode) -> Result<String, serde_json::Error> {
    let mut codeline = Codeline { blocks: vec![] };

    let event_block = Block {
        id: "block".to_owned(), 
        block: if event_node.event_type.unwrap() == ActionType::Player { Some("event".to_owned()) } else { Some("entity_event".to_owned()) }, 
        action: Some(event_node.event),
        args: Some(Args { items: vec![] }),
        target: None,
        data: None,
        direct: None,
        bracket_type: None
    };
    codeline.blocks.push(event_block);

    for expr_node in event_node.expressions {
        if let Some(blocks) = expression_node(expr_node.node) {
            for block in blocks {
                codeline.blocks.push(block);
            }
        }
    }

    let res = serde_json::to_string(&codeline)?;

    Ok(res)
}

fn function_node(function_node: FunctionNode) -> Result<String, serde_json::Error> {
    let mut codeline = Codeline { blocks: vec![] };

    let mut items = vec![
        Arg { item: ArgItem { data: ArgValueData::Id { id: "function".into() }, id: "hint".into() }, slot: 25 },
        Arg { item: ArgItem { data: ArgValueData::Tag { action: "dynamic".into(), block: "func".into(), option: "False".into(),tag: "Is Hidden".into() }, id: "bl_tag".into() }, slot: 26 }
    ];

    for (slot, param) in function_node.params.into_iter().enumerate() {
        let mut default = None;
        if let Some(param_default) = param.default {
            let default_data = arg_val_from_arg(crate::node::Arg {
                value: param_default.value,
                index: 0,
                arg_type: crate::definitions::ArgType::ANY,
                start_pos: param_default.start_pos,
                end_pos: param_default.end_pos,
            }, "".into(), "".into()).unwrap().item;
            
            default = Some(FunctionDefaultItem {
                data: match default_data.data {
                    ArgValueData::Simple { name } => FunctionDefaultItemData::Simple { name },
                    ArgValueData::Id { id } => FunctionDefaultItemData::Id { id },
                    ArgValueData::Location { is_block, loc } => FunctionDefaultItemData::Location { is_block, loc },
                    ArgValueData::Vector { x, y, z } => FunctionDefaultItemData::Vector { x, y, z },
                    ArgValueData::Sound { sound, volume, pitch } => FunctionDefaultItemData::Sound { sound, volume, pitch },
                    ArgValueData::Potion { potion, amplifier, duration } => FunctionDefaultItemData::Potion { potion, amplifier, duration },
                    _ => unreachable!()
                },
                id: default_data.id,
            })
        }
        
        items.push(Arg {
            item: ArgItem {
                data: ArgValueData::FunctionParam {
                    default_value: default,
                    name: param.name,
                    optional: param.optional,
                    plural: param.multiple,
                    param_type: get_type_str(param.param_type),
                },
                id: "pn_el".into(),
            },
            slot: slot as i32
        });
    }

    let function_block = Block {
        id: "block".to_owned(), 
        block: Some("func".to_owned()), 
        action: None,
        args: Some(Args { items }),
        target: None,
        data: Some(function_node.name),
        direct: None,
        bracket_type: None
    };
    codeline.blocks.push(function_block);

    for expr_node in function_node.expressions {
        if let Some(blocks) = expression_node(expr_node.node) { 
            for block in blocks {
                codeline.blocks.push(block)
            }
        }
    }

    let res = serde_json::to_string(&codeline)?;

    Ok(res)
}

fn expression_node(node: Expression) -> Option<Vec<Block>> {
    match node {
        Expression::Action { node } => Some(vec![action_node(node)]),
        Expression::Conditional { node } => Some(conditional_node(node)),
        Expression::Variable { .. } => None,
    }
}

fn conditional_node(node: ConditionalNode) -> Vec<Block> {
    let block = match node.conditional_type {
        ConditionalType::Player => "if_player",
        ConditionalType::Entity => "if_entity",
        ConditionalType::Game => "if_game",
        ConditionalType::Variable => "if_var"
    };

    let mut args: Vec<Arg> = vec![];

    for arg in node.args {
        let arg = match arg_val_from_arg(arg, node.name.clone(), block.to_owned()) {
            Some(res) => res,
            None => continue
        };
        args.push(arg);
    }

    let mut blocks = vec![
        Block {
            action: Some(node.name),
            block: Some(block.to_string()),
            id: "block".to_string(),
            target: match node.conditional_type {
                ConditionalType::Game => None,
                ConditionalType::Variable => None,
                _ => Some(node.selector)
            },
            args: Some(Args { items: args }),
            data: None,
            direct: None,
            bracket_type: None
        },
        Block {
            id: "bracket".into(),
            direct: Some("open".into()),
            bracket_type: Some("norm".into()),
            block: None, 
            args: None, 
            action: None,
            target: None, 
            data: None
        },
    ];

    for expression in node.expressions {
        if let Some(expression_blocks) = expression_node(expression.node) {
            for block in expression_blocks {
                blocks.push(block);
            }
        }
    }

    blocks.push(Block {
        id:"bracket".into(),
        direct: Some("close".into()),
        bracket_type: Some("norm".into()), 
        block: None, 
        args: None, 
        action: None,
        target: None, 
        data: None
    });
    blocks
}

fn action_node(node: ActionNode) -> Block {
    let block = match node.action_type {
        ActionType::Player => "player_action",
        ActionType::Entity => "entity_action",
        ActionType::Game => "game_action",
        ActionType::Variable => "set_var"
    };

    let mut args: Vec<Arg> = vec![];

    for arg in node.args {
        let arg = match arg_val_from_arg(arg, node.name.clone(), block.to_owned()) {
            Some(res) => res,
            None => continue
        };
        args.push(arg);
    }

    Block {
        action: Some(node.name),
        block: Some(block.to_string()),
        id: "block".to_string(),
        target: match node.action_type {
            ActionType::Game => None,
            ActionType::Variable => None,
            _ => Some(node.selector)
        },
        args: Some(Args { items: args }),
        data: None,
        direct: None,
        bracket_type: None
    }
}

fn arg_val_from_arg(arg: crate::node::Arg, node_name: String, block: String) -> Option<Arg> {
    match arg.value {
        crate::node::ArgValue::Empty => None,
        crate::node::ArgValue::Text { text } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Simple { name: text }, id: String::from("comp") }, slot: arg.index } )       
        }
        crate::node::ArgValue::Number { number } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Simple { name: number.to_string() }, id: String::from("num") }, slot: arg.index} )
        }
        crate::node::ArgValue::String { string } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Simple { name: string }, id: String::from("txt") }, slot: arg.index } )
        }
        crate::node::ArgValue::Location { x, y, z, pitch, yaw } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Location { is_block: false, loc: Location { x, y, z, pitch, yaw } }, id: String::from("loc") }, slot: arg.index } )
        } 
        crate::node::ArgValue::Vector { x, y, z } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Vector { x, y, z }, id: String::from("vec") }, slot: arg.index } )
        }
        crate::node::ArgValue::Sound { sound, volume, pitch } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Sound { sound, volume, pitch }, id: String::from("snd") }, slot: arg.index } )
        }
        crate::node::ArgValue::Potion { potion, amplifier, duration } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Potion { potion, amplifier, duration }, id: String::from("pot") }, slot: arg.index } )
        }
        crate::node::ArgValue::Tag { tag, value, definition, .. } => {
           Some( Arg { item: ArgItem { data: ArgValueData::Tag {
            action: node_name,
            block,
            option: value,
            tag
           }, id: String::from("bl_tag")}, slot: definition.unwrap().slot as i32})
        }
        crate::node::ArgValue::Variable { value, scope } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Variable { value, scope }, id: String::from("var") }, slot: arg.index } )
        }
        crate::node::ArgValue::GameValue { value, selector, .. } => {
            Some ( Arg { item: ArgItem { data: ArgValueData::GameValue { game_value: value, target: selector }, id: String::from("g_val") }, slot: arg.index })
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Codeline {
    blocks: Vec<Block>
}

#[derive(Deserialize, Serialize, Debug)]
struct Block {
    id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    block: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Args>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<Selector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    direct: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename="type")]
    bracket_type: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Args {
    items: Vec<Arg>
}

#[derive(Deserialize, Serialize, Debug)]
struct Arg {
    item: ArgItem,
    slot: i32
}

#[derive(Deserialize, Serialize, Debug)]
struct ArgItem {
    data: ArgValueData, 
    id: String
}

#[derive(Deserialize, Debug)]
enum ArgValueData {
    Simple { name: String },
    Id { id: String },
    GameValue {
        #[serde(rename="type")]
        game_value: String,
        target: Selector
    },
    Variable { value: String, scope: String },
    Location { is_block: bool, loc: Location },
    Vector { x: f32, y: f32, z: f32 },
    Sound { sound: String, volume: f32, pitch: f32 },
    Potion { potion: String, amplifier: f32, duration: f32 },
    Tag { action: String, block: String, option: String, tag: String },
    FunctionParam { 
        #[serde(skip_serializing_if="Option::is_none")]
        default_value: Option<FunctionDefaultItem>,
        name: String,
        optional: bool,
        plural: bool,
        #[serde(rename="type")]    
        param_type: String 
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct FunctionDefaultItem {
    data: FunctionDefaultItemData, 
    id: String
}

#[derive(Deserialize, Debug)]
enum FunctionDefaultItemData {
    Simple { name: String },
    Id { id: String },
    Location { is_block: bool, loc: Location },
    Vector { x: f32, y: f32, z: f32 },
    Sound { sound: String, volume: f32, pitch: f32 },
    Potion { potion: String, amplifier: f32, duration: f32 },
}

impl Serialize for ArgValueData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ArgValueData::Simple { name } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("name", name)?;
                state.end()
            }
            ArgValueData::Id { id } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("id", id)?;
                state.end()
            }
            ArgValueData::GameValue { target, game_value } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("type", game_value)?;
                state.serialize_field("target", target)?;
                state.end()
            }
            ArgValueData::Variable { value, scope } => {
                let mut state = serializer.serialize_struct("MyEnum", 2)?;
                state.serialize_field("name", value)?;
                state.serialize_field("scope", scope)?;
                state.end()
            }
            ArgValueData::Location { is_block, loc } => {
                let mut state = serializer.serialize_struct("MyEnum", 2)?;
                state.serialize_field("isBlock", is_block)?;
                state.serialize_field("loc", loc)?;
                state.end()
            }
            ArgValueData::Vector { x, y, z } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("x", x)?;
                state.serialize_field("y", y)?;
                state.serialize_field("z", z)?;
                state.end()
            }
            ArgValueData::Sound { sound, volume, pitch } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("sound", sound)?;
                state.serialize_field("vol", volume)?;
                state.serialize_field("pitch", pitch)?;
                state.end()
            }
            ArgValueData::Potion { potion, amplifier, duration } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("pot", potion)?;
                state.serialize_field("amp", amplifier)?;
                state.serialize_field("dur", duration)?;
                state.end()
            }
            ArgValueData::Tag { action, block, option, tag } => {
                let mut state = serializer.serialize_struct("MyEnum", 4)?;
                state.serialize_field("action", action)?;
                state.serialize_field("block", block)?;
                state.serialize_field("option", option)?;
                state.serialize_field("tag", tag)?;
                state.end()
            }
            ArgValueData::FunctionParam { default_value, name, optional, plural, param_type } => {
                let mut state = serializer.serialize_struct("MyEnum", 4)?;
                if default_value.is_some() {
                    state.serialize_field("default_value", default_value)?;
                }
                state.serialize_field("name", name)?;
                state.serialize_field("optional", optional)?;
                state.serialize_field("plural", plural)?;
                state.serialize_field("type", param_type)?;
                state.end()
            }
        }
    }
}

impl Serialize for FunctionDefaultItemData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FunctionDefaultItemData::Simple { name } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("name", name)?;
                state.end()
            }
            FunctionDefaultItemData::Id { id } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("id", id)?;
                state.end()
            }
            FunctionDefaultItemData::Location { is_block, loc } => {
                let mut state = serializer.serialize_struct("MyEnum", 2)?;
                state.serialize_field("isBlock", is_block)?;
                state.serialize_field("loc", loc)?;
                state.end()
            }
            FunctionDefaultItemData::Vector { x, y, z } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("x", x)?;
                state.serialize_field("y", y)?;
                state.serialize_field("z", z)?;
                state.end()
            }
            FunctionDefaultItemData::Sound { sound, volume, pitch } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("sound", sound)?;
                state.serialize_field("vol", volume)?;
                state.serialize_field("pitch", pitch)?;
                state.end()
            }
            FunctionDefaultItemData::Potion { potion, amplifier, duration } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("pot", potion)?;
                state.serialize_field("amp", amplifier)?;
                state.serialize_field("dur", duration)?;
                state.end()
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct Location {
    x: f32,
    y: f32,
    z: f32,
    pitch: Option<f32>,
    yaw: Option<f32>,
}

impl Serialize for Location {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MyEnum", 5)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("z", &self.y)?;
        state.serialize_field("y", &self.z)?;
        if self.pitch.is_none() {
            state.serialize_field("pitch", &0)?;
        } else {
            state.serialize_field("pitch", &self.pitch.unwrap())?;
        }
        if self.yaw.is_none() {
            state.serialize_field("yaw", &0)?;
        } else {
            state.serialize_field("yaw", &self.yaw.unwrap())?;
        }
        state.end()
    }
}