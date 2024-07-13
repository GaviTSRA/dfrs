use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use crate::{node::{ActionNode, ActionType, ArgValue, EventNode, Expression, FileNode}, token::Selector};

pub fn compile(node: FileNode, debug: bool) -> Vec<String> {
    let mut res: Vec<String> = vec![];
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

fn event_node(event_node: EventNode) -> Result<std::string::String, serde_json::Error> {
    let mut codeline = Codeline { blocks: vec![] };

    let event_block = Block {
        id: "block".to_owned(), 
        block: if event_node.event_type.unwrap() == ActionType::Player { "event".to_owned() } else { "entity_event".to_owned() }, 
        action: event_node.event,
        args: Args { items: vec![] },
        target: None
    };
    codeline.blocks.push(event_block);

    for expr_node in event_node.expressions {
        match expression_node(expr_node.node) {
            Some(block) => codeline.blocks.push(block),
            None => {}
        };
    }

    let res = serde_json::to_string(&codeline)?;

    Ok(res)
}

fn expression_node(node: Expression) -> Option<Block> {
    match node {
        Expression::Action { node } => return Some(action_node(node)),
        Expression::Variable { .. } => return None,
    }
}

fn action_node(node: ActionNode) -> Block {
    let block;
    match node.action_type {
        ActionType::Player => block = "player_action",
        ActionType::Entity => block = "entity_action",
        ActionType::Game => block = "game_action"
    }

    let mut args: Vec<Arg> = vec![];

    for arg in node.args {
        match arg.value {
            ArgValue::Empty => continue,
            ArgValue::Text { text } => {
                args.push( Arg { item: ArgItem { data: ArgValueData::Simple { name: text }, id: String::from("comp") }, slot: arg.index } );        
            }
            ArgValue::Number { number } => {
                args.push( Arg { item: ArgItem { data: ArgValueData::Simple { name: number.to_string() }, id: String::from("num") }, slot: arg.index} );        
            }
            ArgValue::String { string } => {
                args.push( Arg { item: ArgItem { data: ArgValueData::Simple { name: string }, id: String::from("txt") }, slot: arg.index } );  
            }
            ArgValue::Location { x, y, z, pitch, yaw } => {
                args.push( Arg { item: ArgItem { data: ArgValueData::Location { is_block: false, loc: Location { x, y, z, pitch, yaw } }, id: String::from("loc") }, slot: arg.index } );  
            } 
            ArgValue::Vector { x, y, z } => {
                args.push( Arg { item: ArgItem { data: ArgValueData::Vector { x, y, z }, id: String::from("vec") }, slot: arg.index } );  
            }
            ArgValue::Sound { sound, volume, pitch } => {
                args.push( Arg { item: ArgItem { data: ArgValueData::Sound { sound, volume, pitch }, id: String::from("snd") }, slot: arg.index } );  
            }
            ArgValue::Potion { potion, amplifier, duration } => {
                args.push( Arg { item: ArgItem { data: ArgValueData::Potion { potion, amplifier, duration }, id: String::from("pot") }, slot: arg.index } );  
            }
            ArgValue::Tag { tag, value, definition, .. } => {
               args.push( Arg { item: ArgItem { data: ArgValueData::Tag {
                action: node.name.clone(),
                block: block.to_owned(),
                option: value,
                tag
               }, id: String::from("bl_tag")}, slot: definition.unwrap().slot as i32})
            }
            ArgValue::Variable { value, scope } => {
                args.push( Arg { item: ArgItem { data: ArgValueData::Variable { value, scope }, id: String::from("var") }, slot: arg.index } ); 
            }
        }
    }

    Block {
        action: node.name,
        block: block.to_string(),
        id: "block".to_string(),
        target: Some(node.selector),
        args: Args { items: args }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Codeline {
    blocks: Vec<Block>
}

#[derive(Deserialize, Serialize, Debug)]
struct Block {
    id: String,
    block: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    args: Args,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<Selector>
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
    Variable { value: String, scope: String },
    Location { is_block: bool, loc: Location },
    Vector { x: f32, y: f32, z: f32 },
    Sound { sound: String, volume: f32, pitch: f32 },
    Potion { potion: String, amplifier: f32, duration: f32 },
    Tag { action: String, block: String, option: String, tag: String }
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