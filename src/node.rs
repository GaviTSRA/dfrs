use crate::definitions::actions::Action;
use crate::token::Range;
use crate::{
  definitions::{ArgType, DefinedTag},
  token::{Position, Selector, Type},
};
use serde::{Deserialize, Serialize};

pub trait Node {
  fn json(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct FileNode {
  pub uses: Vec<UseNode>,
  pub events: Vec<EventNode>,
  pub functions: Vec<FunctionNode>,
  pub processes: Vec<ProcessNode>,
  pub range: Range,
}

#[derive(Clone, Debug)]
pub struct EventNode {
  pub event_type: Option<ActionType>,
  pub event: String,
  pub expressions: Vec<ExpressionNode>,
  pub range: Range,
  pub name_end_pos: Position,
  pub cancelled: bool,
}

#[derive(Clone, Debug)]
pub struct FunctionNode {
  pub df_name: String,
  pub dfrs_name: String,
  pub params: Vec<FunctionParamNode>,
  pub expressions: Vec<ExpressionNode>,
  pub range: Range,
  pub name_end_pos: Position,
}

#[derive(Clone, Debug)]
pub struct ProcessNode {
  pub name: String,
  pub expressions: Vec<ExpressionNode>,
  pub range: Range,
  pub name_end_pos: Position,
}

#[derive(Clone, Debug)]
pub struct FunctionParamNode {
  pub name: String,
  pub param_type: Type,
  pub optional: bool,
  pub multiple: bool,
  pub default: Option<ArgValueWithPos>,
}

#[derive(Clone, Debug)]
pub struct ExpressionNode {
  pub node: Expression,
  pub range: Range,
}

#[derive(Clone, Debug)]
pub enum Expression {
  Action { node: ActionNode },
  Conditional { node: ConditionalNode },
  Variable { node: VariableNode },
  Call { node: CallNode },
  Start { node: StartNode },
  Repeat { node: RepeatNode },
}

#[derive(Clone, Debug)]
pub struct ActionNode {
  pub action_type: ActionType,
  pub selector: Selector,
  pub name: String,
  pub args: Vec<Arg>,
  pub range: Range,
  pub selector_range: Range,
  pub action: Option<Action>,
}

#[derive(Clone, Debug)]
pub struct ConditionalNode {
  pub conditional_type: ConditionalType,
  pub selector: Selector,
  pub name: String,
  pub args: Vec<Arg>,
  pub range: Range,
  pub selector_range: Option<Range>,
  pub expressions: Vec<ExpressionNode>,
  pub else_expressions: Vec<ExpressionNode>,
  pub inverted: bool,
}

#[derive(Clone, Debug)]
pub struct CallNode {
  pub name: String,
  pub args: Vec<Arg>,
  pub range: Range,
}

#[derive(Clone, Debug)]
pub struct StartNode {
  pub name: String,
  pub args: Vec<Arg>,
  pub range: Range,
}

#[derive(Clone, Debug)]
pub struct RepeatNode {
  pub name: String,
  pub args: Vec<Arg>,
  pub expressions: Vec<ExpressionNode>,
  pub range: Range,
}

#[derive(Clone, Debug)]
pub struct UseNode {
  pub file: String,
  pub range: Range,
}

#[derive(Clone, Debug)]
pub struct Arg {
  pub value: ArgValue,
  pub index: i32,
  pub arg_type: ArgType,
  pub range: Range,
}

#[derive(Clone, Debug)]
pub struct VariableNode {
  pub dfrs_name: String,
  pub df_name: String,
  pub var_variant: VariableVariant,
  pub var_type: Option<ArgType>,
  pub action: Option<ActionNode>,
  pub range: Range,
}

#[derive(Clone, Debug)]
pub enum ArgValue {
  Empty,
  Number {
    number: f32,
  },
  ComplexNumber {
    number: String,
  },
  String {
    string: String,
  },
  Text {
    text: String,
  },
  Location {
    x: f32,
    y: f32,
    z: f32,
    pitch: Option<f32>,
    yaw: Option<f32>,
  },
  Vector {
    x: f32,
    y: f32,
    z: f32,
  },
  Sound {
    sound: String,
    variant: Option<String>,
    volume: f32,
    pitch: f32,
  },
  Potion {
    potion: String,
    amplifier: f32,
    duration: f32,
  },
  Particle {
    particle: String,
    cluster: ParticleCluster,
    data: ParticleData,
  },
  Item {
    item: String,
  },
  Tag {
    tag: String,
    value: Box<ArgValue>,
    definition: Option<DefinedTag>,
    name_end_pos: Position,
    value_start_pos: Position,
  },
  Variable {
    name: String,
    scope: String,
    var_type: Option<ArgType>,
  },
  GameValue {
    df_name: Option<String>,
    dfrs_name: String,
    selector: Selector,
    selector_end_pos: Position,
  },
  Condition {
    name: String,
    args: Vec<Arg>,
    selector: Selector,
    conditional_type: ConditionalType,
    inverted: bool,
  },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ParticleCluster {
  pub amount: i32,
  pub horizontal: f32,
  pub vertical: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParticleData {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub x: Option<f32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub y: Option<f32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub z: Option<f32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub motion_variation: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rgb: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none", rename = "rgb_fade")]
  pub rgb_fade: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub color_variation: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub material: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub size: Option<f32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub size_variation: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub roll: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct ArgValueWithPos {
  pub value: ArgValue,
  pub range: Range,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionType {
  Player,
  Entity,
  Game,
  Variable,
  Control,
  Select,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConditionalType {
  Player,
  Entity,
  Game,
  Variable,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableVariant {
  Line,
  Local,
  Game,
  Save,
}
