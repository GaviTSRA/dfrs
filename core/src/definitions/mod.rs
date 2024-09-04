pub mod actions;
pub mod action_dump;
pub mod game_values;

#[derive(Clone, Debug)]
pub struct DefinedArg {
    pub arg_types: Vec<ArgType>,
    pub name: String,
    pub allow_multiple: bool,
    pub optional: bool
}

impl DefinedArg {
    pub fn new(name: &str, arg_types: Vec<ArgType>, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_types, allow_multiple, optional};
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
    GAME_VALUE,
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
        return DefinedTag {dfrs_name: dfrs_name.to_owned(), df_name: df_name.to_owned(), slot, options, default}
    }
}