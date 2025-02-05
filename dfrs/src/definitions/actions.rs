use crate::definitions::action_dump::{ADAction, DFRSValue, RawActionDump};
use crate::definitions::{ArgType, DefinedArg, DefinedArgBranch, DefinedArgOption, DefinedTag};
use crate::utility::{to_camel_case, to_dfrs_name};

#[derive(Debug, Clone)]
pub struct Action {
    pub dfrs_name: String,
    pub df_name: String,
    pub aliases: Vec<String>,
    pub has_conditional_arg: bool,
    pub args: Vec<DefinedArgBranch>,
    pub tags: Vec<DefinedTag>
}

impl Action {
    pub fn new(dfrs_name: String, df_name: &str, aliases: Vec<String>, args: Vec<DefinedArgBranch>, tags: Vec<DefinedTag>, has_conditional_arg: bool) -> Action {
        Action {dfrs_name, df_name: df_name.to_owned(), aliases, args, tags, has_conditional_arg}
    }
}

impl DFRSValue for Action {
    fn dfrs_equals(&self, value: &str) -> bool {
        self.dfrs_name == value
    }
    fn df_equals(&self, value: &str) -> bool {
        self.df_name == value || self.aliases.contains(&value.to_string())
    }
}

pub fn get_actions(action_dump: &RawActionDump, block: &str) -> Vec<Action> {
    let mut actions = vec![];

    for action in &action_dump.actions {
        if action.codeblock_name != block {
            continue;
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
    let mut branches: Vec<DefinedArgBranch> = vec![];
    let mut current_branch: Vec<Vec<DefinedArg>> = vec![];
    let mut current_branch_path: Vec<DefinedArg> = vec![];
    let mut current_options: Vec<DefinedArgOption> = vec![];

    let mut is_or = false;

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
            "VEHICLE" => ArgType::ITEM,
            "ENTITY_TYPE" => ArgType::ITEM,
            "ANY_TYPE" => ArgType::ANY,
            "NONE" => ArgType::EMPTY,
            "OR" => {
                is_or = true;
                continue;
            }
            "" => {
                current_branch_path.push(DefinedArg::new(current_options));
                current_branch.push(current_branch_path);
                branches.push(DefinedArgBranch {
                    paths: current_branch,
                });
                current_options = vec![];
                current_branch_path = vec![];
                current_branch = vec![];
                continue;
            }
            _ => panic!("Unknown arg type: {}", arg.arg_type)
        };
        if is_or || current_options.is_empty() {
            if current_branch_path.len() > 0 {
                current_branch_path.push(DefinedArg::new(current_options));
                current_branch.push(current_branch_path);
                current_branch_path = vec![];
                current_options = vec![DefinedArgOption::new(
                    arg.description.first().or(Some(&String::from(""))).unwrap().clone(),
                    arg_type, arg.optional, arg.plural
                )];
            } else {
                current_options.push(DefinedArgOption::new(
                    arg.description.first().or(Some(&String::from(""))).unwrap().clone(),
                    arg_type, arg.optional, arg.plural
                ));
            }
            is_or = false;
        } else {
            current_branch_path.push(DefinedArg::new(current_options));
            current_options = vec![DefinedArgOption::new(
                arg.description.first().or(Some(&String::from(""))).unwrap().clone(),
                arg_type, arg.optional, arg.plural
            )];
        }
    }
    if action.icon.arguments.len() > 0 {
        if current_options.len() > 0 {
            current_branch_path.push(DefinedArg::new(current_options));
            current_branch.push(current_branch_path);
            branches.push(DefinedArgBranch {
                paths: current_branch,
            });
        }
    }

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
            "VEHICLE" => ArgType::ITEM,
            "ENTITY_TYPE" => ArgType::ITEM,
            "ANY_TYPE" => ArgType::ANY,
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
                    return Action::new(action.name.clone() + "-NotYetSupported", &action.name, action.aliases.clone(), vec![], vec![], action.sub_action_blocks.is_some() && !action.sub_action_blocks.clone().unwrap().is_empty());
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
                let new_arg = DefinedArg::new(vec![DefinedArgOption::new(arg.description.first().or(Some(&String::from(""))).unwrap().clone(), arg_type, true, arg.plural)]);
                current_args.push(new_arg);
            } else {
                current_args.get_mut(index_after_or).unwrap().options.push(DefinedArgOption::new(arg.description.first().or(Some(&String::from(""))).unwrap().clone(), arg_type, arg.optional, arg.plural));
            }
            index_after_or += 1;
        } else {
            let new_arg = DefinedArg::new(vec![DefinedArgOption::new(arg.description.first().or(Some(&String::from(""))).unwrap().clone(), arg_type, arg.plural, arg.plural)]);
            current_args.push(new_arg);
        }
    }
    //if is_or && or_index != current_args.len() && or_index > current_args.len() - or_index {
    //    for i in (current_args.len() - or_index)..=or_index {
    //        current_args.get_mut(i).unwrap().optional = true;
    //    }
    //}
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
    Action::new(name, &action.name, action.aliases.clone(), vec![DefinedArgBranch {
        paths: vec![args]
    }], tags, action.sub_action_blocks.is_some() && !action.sub_action_blocks.clone().unwrap().is_empty())
}