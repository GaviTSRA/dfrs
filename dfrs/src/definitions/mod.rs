pub mod action_dump;
pub mod game_values;
pub mod events;
pub mod actions;

#[derive(Clone, Debug)]
pub struct DefinedArgBranch {
    pub paths: Vec<Vec<DefinedArg>>,
}

impl DefinedArgBranch {
    pub fn new(paths: Vec<Vec<DefinedArg>>) -> DefinedArgBranch {
        DefinedArgBranch { paths }
    }
}

#[derive(Clone, Debug)]
pub struct DefinedArg {
    pub options: Vec<DefinedArgOption>,
}

impl DefinedArg {
    pub fn new(options: Vec<DefinedArgOption>) -> DefinedArg {
        DefinedArg { options }
    }
}

#[derive(Clone, Debug)]
pub struct DefinedArgOption {
    pub name: String,
    pub arg_type: ArgType,
    pub plural: bool,
    pub optional: bool
}

impl DefinedArgOption {
    pub fn new(name: String, arg_type: ArgType, optional: bool, plural: bool) -> DefinedArgOption {
        DefinedArgOption { name, arg_type, plural, optional }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArgType {
    EMPTY,
    NUMBER,
    TEXT,
    STRING,
    LOCATION,
    VECTOR,
    SOUND,
    POTION,
    PARTICLE,
    ITEM,
    TAG,
    VARIABLE,
    GameValue,
    CONDITION,
    ANY
}

#[derive(Clone, Debug)]
pub struct DefinedTag {
    pub dfrs_name: String,
    pub df_name: String,
    pub slot: i8,
    pub options: Vec<String>,
    pub default: String
}

impl DefinedTag {
    pub fn new(dfrs_name: &str, df_name: &str, slot: i8, options: Vec<String>, default: String) -> DefinedTag {
        DefinedTag {dfrs_name: dfrs_name.to_owned(), df_name: df_name.to_owned(), slot, options, default}
    }
}