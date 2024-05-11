pub mod player_actions;

#[derive(Clone, Debug)]
pub struct DefinedArg {
    pub arg_type: ArgType,
    pub name: String,
    pub allow_multiple: bool,
    pub slot: i32,
    pub optional: bool
}

impl DefinedArg {
    pub fn number(name: &str, slot: i32, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::NUMBER, slot, allow_multiple, optional};
    }
    pub fn text(name: &str, slot: i32, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::TEXT, slot, allow_multiple, optional};
    }
    pub fn string(name: &str, slot: i32, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::STRING, slot, allow_multiple, optional};
    }   
    pub fn location(name: &str, slot: i32, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::LOCATION, slot, allow_multiple, optional};
    }
    pub fn vector(name: &str, slot: i32, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::VECTOR, slot, allow_multiple, optional};
    }
    pub fn sound(name: &str, slot: i32, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::SOUND, slot, allow_multiple, optional};
    }
    pub fn potion(name: &str, slot: i32, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::POTION, slot, allow_multiple, optional};
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
    TAG
}

pub struct DefinedTag {
    pub dfrs_name: String,
    pub options: Vec<String>
}

impl DefinedTag {
    pub fn new(dfrs_name: &str, options: Vec<&str>) -> DefinedTag {
        let mut new_options = vec![];
        for option in options {
            new_options.push(option.to_owned());
        }
        return DefinedTag {dfrs_name: dfrs_name.to_owned(), options: new_options}
    }
}