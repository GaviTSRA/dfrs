use crate::{definitions::ArgType, token::{Position, Selector}};

pub trait Node {
    fn json(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct FileNode {
    pub events: Vec<EventNode>,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct EventNode {
    pub event_type: Option<ActionType>,
    pub event: String,
    pub expressions: Vec<ExpressionNode>,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct ExpressionNode {
    pub node: Expression,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub enum Expression {
    Action { node: ActionNode }
}

#[derive(Clone, Debug)]
pub struct ActionNode {
    pub action_type: ActionType,
    pub selector: Selector,
    pub name: String,
    pub args: Vec<Arg>,
    pub start_pos: Position,
    pub end_pos: Position,
    pub implicit_selector: bool
}

#[derive(Clone, Debug)]
pub struct Arg {
    pub value: ArgValue,
    pub index: i32,
    pub arg_type: ArgType
}

#[derive(Clone, Debug)]
pub enum ArgValue {
    Empty,
    Number { number: f32 },
    String { string: String },
    Text { text: String },
    Location { x: f32, y: f32, z: f32, pitch: Option<f32>, yaw: Option<f32> },
    Vector { x: f32, y: f32, z: f32},
    Sound { sound: String, volume: f32, pitch: f32 },
    Potion { potion: String, amplifier: f32, duration: f32 },
    Tag { tag: String, value: String }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionType {
    Player,
    Entity,
    Game
}