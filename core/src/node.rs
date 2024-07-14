use crate::{definitions::{ArgType, DefinedTag}, token::{Position, Selector}};

pub trait Node {
    fn json(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct FileNode {
    pub events: Vec<EventNode>,
    pub functions: Vec<FunctionNode>,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct EventNode {
    pub event_type: Option<ActionType>,
    pub event: String,
    pub expressions: Vec<ExpressionNode>,
    pub start_pos: Position,
    pub name_end_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct FunctionNode {
    pub name: String,
    pub expressions: Vec<ExpressionNode>,
    pub start_pos: Position,
    pub name_end_pos: Position,
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
    Action { node: ActionNode },
    Variable { node: VariableNode }
}

#[derive(Clone, Debug)]
pub struct ActionNode {
    pub action_type: ActionType,
    pub selector: Selector,
    pub name: String,
    pub args: Vec<Arg>,
    pub start_pos: Position,
    pub selector_start_pos: Position,
    pub selector_end_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct Arg {
    pub value: ArgValue,
    pub index: i32,
    pub arg_type: ArgType,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct VariableNode {
    pub dfrs_name: String,
    pub df_name: String,
    pub var_type: VariableType,
    pub start_pos: Position,
    pub end_pos: Position
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
    Tag { tag: String, value: String, definition: Option<DefinedTag>, name_end_pos: Position, value_start_pos: Position },
    Variable { value: String, scope: String }
}

#[derive(Clone, Debug)]
pub struct ArgValueWithPos {
    pub value: ArgValue,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionType {
    Player,
    Entity,
    Game
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableType {
    Line,
    Local,
    Game,
    Save
}