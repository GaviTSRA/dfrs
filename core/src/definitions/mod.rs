pub mod player_actions;

#[derive(Clone, Debug)]
pub struct DefinedArg {
    pub arg_type: ArgType,
    pub name: String,
    pub allow_multiple: bool,
    pub optional: bool
}

impl DefinedArg {
    pub fn number(name: &str, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::NUMBER, allow_multiple, optional};
    }
    pub fn text(name: &str, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::TEXT, allow_multiple, optional};
    }
    pub fn string(name: &str, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::STRING, allow_multiple, optional};
    }   
    pub fn location(name: &str, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::LOCATION, allow_multiple, optional};
    }
    pub fn vector(name: &str, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::VECTOR, allow_multiple, optional};
    }
    pub fn sound(name: &str, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::SOUND, allow_multiple, optional};
    }
    pub fn potion(name: &str, optional: bool, allow_multiple: bool) -> DefinedArg {
        return DefinedArg {name: name.to_owned(), arg_type: ArgType::POTION, allow_multiple, optional};
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

#[derive(Clone, Debug)]
pub struct DefinedTag {
    pub dfrs_name: String,
    pub df_name: String,
    pub slot: i8,
    pub options: Vec<String>
}

impl DefinedTag {
    pub fn new(dfrs_name: &str, df_name: &str, slot: i8, options: Vec<&str>) -> DefinedTag {
        let mut new_options = vec![];
        for option in options {
            new_options.push(option.to_owned());
        }
        return DefinedTag {dfrs_name: dfrs_name.to_owned(), df_name: df_name.to_owned(), slot, options: new_options}
    }
}