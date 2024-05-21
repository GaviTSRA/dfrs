use crate::definitions::{DefinedArg, DefinedTag};

use super::{action_dump::ActionDump, ArgType};

pub struct PlayerAction {
    pub dfrs_name: String,
    pub df_name: String,
    pub args: Vec<DefinedArg>,
    pub tags: Vec<DefinedTag>
}

impl PlayerAction {
    pub fn new(dfrs_name: String, df_name: &str, args: Vec<DefinedArg>, tags: Vec<DefinedTag>) -> PlayerAction {
        return PlayerAction {dfrs_name: dfrs_name, df_name: df_name.to_owned(), args, tags};
    }
}

pub struct PlayerActions {
    player_actions: Vec<PlayerAction>
}

impl PlayerActions {
    pub fn new() -> PlayerActions {
        let mut actions = vec![];

        let action_dump = ActionDump::load();

        for action in action_dump.actions {
            if action.codeblock_name != "PLAYER ACTION" {
                continue
            }

            let mut args = vec![];
            for arg in action.icon.arguments {
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
            for tag in action.tags {
                let mut options = vec![];
                for option in tag.options {
                    options.push(option.name);
                }

                //TODO dfrs tagnames
                let new_tag = DefinedTag::new(&tag.name, &tag.name, tag.slot, options, tag.default_option);
                tags.push(new_tag);
            }

            let mut v: Vec<char> = action.name.clone().chars().collect();
            v[0] = v[0].to_lowercase().nth(0).unwrap();
            let name: String = v.into_iter().collect();
            let new_action = PlayerAction::new(name, &action.name, args, tags);
            actions.push(new_action);
        }

        return PlayerActions {player_actions: actions};
    }

    pub fn get(&self, dfrs_name: String) -> Option<&PlayerAction> {
        for action in &self.player_actions {
            if action.dfrs_name == dfrs_name {
                return Some(action);
            }
        }
        return None;
    }
}
