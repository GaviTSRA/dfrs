use serde::Deserialize;

use crate::utility::to_camel_case;

use super::{ArgType, DefinedArg, DefinedTag};

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ActionDump {
    pub codeblocks: Vec<ADCodeBlock>,
    pub actions: Vec<ADAction>,
    #[serde(skip)]
    pub game_value_categories: String,
    pub game_values: Vec<ADGameValue>,
    #[serde(skip)]
    pub particle_categories: String,
    #[serde(skip)]
    pub particles: String,
    #[serde(skip)]
    pub sound_categories: String,
    #[serde(skip)]
    pub sounds: String,
    #[serde(skip)]
    pub potions: String,
    #[serde(skip)]
    pub cosmetics: String,
    #[serde(skip)]
    pub shops: String
}

#[derive(Deserialize)]
pub struct ADCodeBlock {
    pub name: String,
    pub identifier: String,
    pub item: ADIcon
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ADAction {
    pub name: String,
    pub codeblock_name: String,
    pub tags: Vec<ADTag>,
    pub aliases: Vec<String>,
    pub icon: ADIcon
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ADTag {
    pub name: String,
    pub options: Vec<ADTagOption>,
    pub default_option: String,
    pub slot: i8
}

#[derive(Deserialize)]
pub struct ADTagOption {
    pub name: String,
    pub icon: ADIcon,
    pub aliases: Vec<String>
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ADIcon {
    pub material: String,
    pub name: String,
    pub deprecated_note: Vec<String>,
    pub description: Vec<String>,
    pub example: Vec<String>,
    pub works_with: Vec<String>,
    pub additional_info: Vec<Vec<String>>,
    pub required_rank: String,
    pub require_tokens: bool,
    pub require_rank_and_tokens: bool,
    pub advanced: bool,
    pub loaded_item: String,
    #[serde(default="default_i32")]
    pub tags: i32,
    #[serde(default="default_vec_arg")]
    pub arguments: Vec<ADArgument>
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ADArgument {
    #[serde(rename="type", alias="text")]
    pub arg_type: String,
    #[serde(default="default_bool")]
    pub plural: bool,
    #[serde(default="default_bool")]
    pub optional: bool,
    #[serde(default="default_vec_string")]
    pub description: Vec<String>,
    #[serde(default="default_vec_vec_string")]
    pub notes: Vec<Vec<String>>
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ADGameValue {
    pub aliases: Vec<String>,
    pub category: String,
    pub icon: ADIcon
}

impl ActionDump {
    pub fn load() -> ActionDump {
        let file = include_str!("action_dump.json");
        serde_json::from_str(file).expect("Failed to parse action dump")
    }
}

fn default_i32() -> i32 {
    0
}

fn default_bool() -> bool {
    false
}

fn default_vec_string() -> Vec<String> {
    vec![]
}

fn default_vec_vec_string() -> Vec<Vec<String>> {
    vec![]
}


fn default_vec_arg() -> Vec<ADArgument> {
    vec![]
}

#[derive(Debug)]
pub struct Action {
    pub dfrs_name: String,
    pub df_name: String,
    pub args: Vec<DefinedArg>,
    pub tags: Vec<DefinedTag>
}

impl Action {
    pub fn new(dfrs_name: String, df_name: &str, args: Vec<DefinedArg>, tags: Vec<DefinedTag>) -> Action {
        Action {dfrs_name, df_name: df_name.to_owned(), args, tags}
    }
}

pub fn get_actions(action_dump: &ActionDump, block: &str) -> Vec<Action> {
    let mut actions = vec![];

    for action in &action_dump.actions {
        if action.codeblock_name != block {
            continue
        }

        let new_action = get_action(action);
        actions.push(new_action);
    }

    actions
}

pub fn get_action(action: &ADAction) -> Action {
    let mut args: Vec<DefinedArg> = vec![];
    let mut is_or = false;
    let mut index = 0;
    let mut index_after_or = 0;
    let mut args_before_or= 0;
    let mut or_index= 0;
    let mut current_args: Vec<DefinedArg> = vec![];

    for arg in &action.icon.arguments {
        let arg_type = match &arg.arg_type as &str {
            "NUMBER" => ArgType::NUMBER,
            "COMPONENT" => ArgType::TEXT,
            "TEXT" => ArgType::STRING,
            "LOCATION" => ArgType::LOCATION,
            "POTION" => ArgType::POTION,
            "SOUND" => ArgType::SOUND,
            "VECTOR" => ArgType::VECTOR,
            "PARTICLE" => ArgType::PARTICLE,
            "LIST" => ArgType::VARIABLE,
            "DICT" => ArgType::VARIABLE,
            "VARIABLE" => ArgType::VARIABLE,
            "ANY_TYPE" => ArgType::ANY, // TODO test

            //TODO all below
            "VEHICLE" => ArgType::EMPTY,
            "ENTITY_TYPE" => ArgType::EMPTY,
            "ITEM" => ArgType::ITEM,
            "BLOCK" => ArgType::ITEM,
            "BLOCK_TAG" => ArgType::STRING,
            "PROJECTILE" => ArgType::ITEM,
            "SPAWN_EGG" => ArgType::ITEM,
            "NONE" => ArgType::EMPTY,
            "OR" => {
                or_index = index - 1;
                index_after_or = 0;
                args_before_or = current_args.len();
                is_or = true;
                continue;
            },
            "" => {
                if is_or {
                    return Action::new(action.name.clone() + "-NotYetSupported", &action.name, vec![], vec![]);
                }
                for arg in current_args {
                    args.push(arg);
                }
                current_args = vec![];
                index = 0;
                continue;
            },
            
            _ => panic!("Unknown arg type: {}", arg.arg_type)
        };
        index += 1;

        if is_or {
            if index_after_or > args_before_or - 1 {
                let new_arg = DefinedArg::new(arg.description.first().expect("No description"), vec![arg_type], true, arg.plural);
                current_args.push(new_arg);
            } else {
                current_args.get_mut(index_after_or).unwrap().arg_types.push(arg_type);
            }
            index_after_or += 1;
        } else {
            let new_arg = DefinedArg::new(arg.description.first().expect("No description"), vec![arg_type], arg.optional, arg.plural);
            current_args.push(new_arg);
        }
    }
    if is_or && or_index != current_args.len() && or_index > current_args.len() - or_index {
        for i in (current_args.len() - or_index)..=or_index {
            current_args.get_mut(i).unwrap().optional = true;
        }
    }
    for arg in current_args {
        args.push(arg);
    }

    let mut tags = vec![];
    for tag in &action.tags {
        let mut options = vec![];
        for option in &tag.options {
            options.push(option.name.clone());
        }

        let dfrs_name = to_camel_case(&tag.name);
        let new_tag = DefinedTag::new(&dfrs_name, &tag.name, tag.slot, options, tag.default_option.clone());
        tags.push(new_tag);
    }

    let mut v: String = action.name.clone().trim()
        .replace("+=", "addDirect").replace("-=", "subDirect")
        .replace('+', "add").replace('-', "sub")
        .replace('%', "mod").replace('/', "div").replace('=', "equal");
    if v == *"x" {
        v = "mul".into();
    }
    let mut vv: Vec<char> = v.chars().collect();
    vv[0] = vv[0].to_lowercase().next().unwrap();
    let name: String = vv.into_iter().collect();
    Action::new(name, &action.name, args, tags)
}