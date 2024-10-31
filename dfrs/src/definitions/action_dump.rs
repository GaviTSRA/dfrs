use serde::Deserialize;
use crate::utility::{to_camel_case, to_dfrs_name};

use super::{ArgType, DefinedArg, DefinedTag};

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct RawActionDump {
    pub codeblocks: Vec<ADCodeBlock>,
    pub actions: Vec<ADAction>,
    #[serde(skip)]
    pub game_value_categories: String,
    pub game_values: Vec<ADGameValue>,
    #[serde(skip)]
    pub particle_categories: String,
    pub particles: Vec<ADParticle>,
    #[serde(skip)]
    pub sound_categories: String,
    pub sounds: Vec<ADSound>,
    pub potions: Vec<ADPotion>,
    #[serde(skip)]
    pub cosmetics: String,
    #[serde(skip)]
    pub shops: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct ADParticle {
    particle: String,
    icon: ADIcon,
    category: Option<String>,
    fields: Vec<String>
}

impl DFRSValue for ADParticle {
    fn dfrs_name(&self) -> String {
        self.particle.clone()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ADSound {
    sound: String,
    icon: ADIcon,
}

impl DFRSValue for ADSound {
    fn dfrs_name(&self) -> String {
        self.sound.clone()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ADPotion {
    potion: String,
    icon: ADIcon,
}

impl DFRSValue for ADPotion {
    fn dfrs_name(&self) -> String {
        self.potion.clone()
    }
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
    pub icon: ADIcon,
    pub sub_action_blocks: Option<Vec<String>>
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

#[derive(Deserialize, Debug, Clone)]
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
    pub arguments: Vec<ADArgument>,
    #[serde(default="default_vec_return", rename="returnValues")]
    pub return_values: Vec<ADReturnValue>,
    pub return_type: Option<String>
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct ADReturnValue {
    #[serde(rename="type", alias="text")]
    pub return_type: String,
    #[serde(default="default_vec_string")]
    pub description: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct ADGameValue {
    pub aliases: Vec<String>,
    pub category: String,
    pub icon: ADIcon
}

impl RawActionDump {
    pub fn load() -> RawActionDump {
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

fn default_vec_return() -> Vec<ADReturnValue> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct Action {
    pub dfrs_name: String,
    pub df_name: String,
    pub has_conditional_arg: bool,
    pub args: Vec<DefinedArg>,
    pub tags: Vec<DefinedTag>
}

impl Action {
    pub fn new(dfrs_name: String, df_name: &str, args: Vec<DefinedArg>, tags: Vec<DefinedTag>, has_conditional_arg: bool) -> Action {
        Action {dfrs_name, df_name: df_name.to_owned(), args, tags, has_conditional_arg}
    }
}

impl DFRSValue for Action {
    fn dfrs_name(&self) -> String {
        self.dfrs_name.clone()
    }
}

pub fn get_actions(action_dump: &RawActionDump, block: &str) -> Vec<Action> {
    let mut actions = vec![];

    for action in &action_dump.actions {
        if action.codeblock_name != block {
            continue
        }
        if action.icon.return_values.len() == 0 && action.icon.arguments.len() == 0 && action.icon.material == "STONE" && action.codeblock_name != "START PROCESS" {
            continue;
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
            "ITEM" => ArgType::ITEM,
            "BLOCK" => ArgType::ITEM,
            "BLOCK_TAG" => ArgType::STRING,
            "PROJECTILE" => ArgType::ITEM,
            "SPAWN_EGG" => ArgType::ITEM,
            "ANY_TYPE" => ArgType::ANY,
            "NONE" => ArgType::EMPTY,

            //TODO all below
            "VEHICLE" => ArgType::EMPTY,
            "ENTITY_TYPE" => ArgType::EMPTY,
            "OR" => {
                or_index = index - 1;
                index_after_or = 0;
                args_before_or = current_args.len();
                is_or = true;
                continue;
            },
            "" => {
                if is_or {
                    return Action::new(action.name.clone() + "-NotYetSupported", &action.name, vec![], vec![], action.sub_action_blocks.is_some() && !action.sub_action_blocks.clone().unwrap().is_empty());
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

    let name = to_dfrs_name(&action.name);
    Action::new(name, &action.name, args, tags, action.sub_action_blocks.is_some() && !action.sub_action_blocks.clone().unwrap().is_empty())
}

trait DFRSValue {
    fn dfrs_name(&self) -> String;
}

#[derive(Debug)]
pub struct ValueList<T> where T: DFRSValue {
    pub values: Vec<T>,
}

impl<T> ValueList<T> where T: DFRSValue {
    pub fn new(values: Vec<T>) -> ValueList<T> {
        ValueList { values }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&T> {
        self.values.iter().find(|&action| action.dfrs_name() == dfrs_name)
    }

    pub fn all(&self) -> &Vec<T> {
        &self.values
    }
}

#[derive(Debug)]
pub struct ActionDump {
    pub player_actions: ValueList<Action>,
    pub entity_actions: ValueList<Action>,
    pub game_actions: ValueList<Action>,
    pub variable_actions: ValueList<Action>,
    pub control_actions: ValueList<Action>,
    pub select_actions: ValueList<Action>,
    pub start_process_action: Action,

    pub player_conditionals: ValueList<Action>,
    pub entity_conditionals: ValueList<Action>,
    pub game_conditionals: ValueList<Action>,
    pub variable_conditionals: ValueList<Action>,

    pub repeats: ValueList<Action>,

    pub sounds: ValueList<ADSound>,
    pub potions: ValueList<ADPotion>,
    pub particles: ValueList<ADParticle>
}

impl ActionDump {
    pub fn new(action_dump: &RawActionDump) -> ActionDump {
        let actions  = get_actions(&action_dump, "START PROCESS");
        let action = actions.get(0).unwrap();
        let start_process_action = Action {
            args: action.args.clone(),
            df_name: action.df_name.clone(),
            dfrs_name: action.dfrs_name.clone(),
            tags: action.tags.clone(),
            has_conditional_arg: action.has_conditional_arg.clone()
        };

        ActionDump {
            player_actions: ValueList::new(get_actions(action_dump, "PLAYER ACTION")),
            entity_actions: ValueList::new(get_actions(action_dump, "ENTITY ACTION")),
            game_actions: ValueList::new(get_actions(action_dump, "GAME ACTION")),
            variable_actions: ValueList::new(get_actions(action_dump, "SET VARIABLE")),
            control_actions: ValueList::new(get_actions(action_dump, "CONTROL")),
            select_actions: ValueList::new(get_actions(action_dump, "SELECT OBJECT")),
            start_process_action,

            player_conditionals: ValueList::new(get_actions(action_dump, "IF PLAYER")),
            entity_conditionals: ValueList::new(get_actions(action_dump, "IF ENTITY")),
            game_conditionals: ValueList::new(get_actions(action_dump, "IF GAME")),
            variable_conditionals: ValueList::new(get_actions(action_dump, "IF VARIABLE")),

            repeats: ValueList::new(get_actions(action_dump, "REPEAT")),

            sounds: ValueList::new(action_dump.sounds.clone()),
            potions: ValueList::new(action_dump.potions.clone()),
            particles: ValueList::new(action_dump.particles.clone())
        }
    }
}