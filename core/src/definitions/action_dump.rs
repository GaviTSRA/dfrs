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
    #[serde(skip)]
    pub game_values: String,
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

pub struct Action {
    pub dfrs_name: String,
    pub df_name: String,
    pub args: Vec<DefinedArg>,
    pub tags: Vec<DefinedTag>
}

impl Action {
    pub fn new(dfrs_name: String, df_name: &str, args: Vec<DefinedArg>, tags: Vec<DefinedTag>) -> Action {
        return Action {dfrs_name: dfrs_name, df_name: df_name.to_owned(), args, tags};
    }
}

pub fn get_actions(action_dump: &ActionDump, block: &str) -> Vec<Action> {
    let mut actions = vec![];

    for action in &action_dump.actions {
        if action.codeblock_name != block {
            continue
        }

        let mut args = vec![];
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
                "ANY_TYPE" => ArgType::ANY, // TODO test

                //TODO all below
                "LIST" => ArgType::EMPTY,
                "VEHICLE" => ArgType::EMPTY,
                "ITEM" => ArgType::ITEM,
                "BLOCK" => ArgType::ITEM,
                "BLOCK_TAG" => ArgType::STRING,
                "PROJECTILE" => ArgType::ITEM,
                "SPAWN_EGG" => ArgType::ITEM,
                "VARIABLE" => ArgType::EMPTY,
                "NONE" => ArgType::EMPTY,
                "OR" => ArgType::EMPTY,
                "" => continue,
                
                _ => panic!("Unknown arg type: {}", arg.arg_type)
            };
            
            if arg.description.get(0).is_none() { // TODO remove after OR type is implemented
                continue;
            }

            let new_arg = DefinedArg::new(arg.description.get(0).expect("No description"), arg_type, arg.optional, arg.plural);     
            args.push(new_arg);
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

        let mut v: Vec<char> = action.name.clone().chars().collect();
        v[0] = v[0].to_lowercase().nth(0).unwrap();
        let name: String = v.into_iter().collect();
        let new_action = Action::new(name, &action.name, args, tags);
        actions.push(new_action);
    }

    actions
}