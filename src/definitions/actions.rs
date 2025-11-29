use crate::definitions::action_dump::{ADAction, DFRSValue, RawActionDump};
use crate::definitions::{ArgType, DefinedArg, DefinedArgBranch, DefinedArgOption, DefinedTag};
use crate::utility::{to_camel_case, to_dfrs_name};

#[derive(Debug, Clone)]
pub struct Action {
  pub dfrs_name: String,
  pub df_name: String,
  pub aliases: Vec<String>,
  pub args: Vec<DefinedArgBranch>,
  pub tags: Vec<DefinedTag>,
  pub return_type: Option<ArgType>,
  pub description: String,
}

impl Action {
  pub fn new(
    dfrs_name: String,
    df_name: &str,
    aliases: Vec<String>,
    args: Vec<DefinedArgBranch>,
    tags: Vec<DefinedTag>,
    return_type: Option<ArgType>,
    description: String,
  ) -> Action {
    Action {
      dfrs_name,
      df_name: df_name.to_owned(),
      aliases,
      args,
      tags,
      return_type,
      description,
    }
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
    if action.icon.return_values.len() == 0
      && action.icon.arguments.len() == 0
      && action.icon.material == "STONE"
      && action.codeblock_name != "START PROCESS"
    {
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

  for (index, arg) in action.icon.arguments.iter().enumerate() {
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
      "BYTE" => ArgType::STRING, // TODO
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
      _ => panic!("Unknown arg type: {}", arg.arg_type),
    };

    if is_or || current_options.is_empty() {
      let mut has_multiple_after_or = false;
      if let Some(next_arg) = &action.icon.arguments.get(index + 1) {
        if next_arg.arg_type != "" && next_arg.arg_type != "OR" {
          has_multiple_after_or = true;
        }
      }
      if !is_or {
        has_multiple_after_or = false;
      }
      if current_branch_path.len() > 0 || has_multiple_after_or {
        current_branch_path.push(DefinedArg::new(current_options));
        current_branch.push(current_branch_path);
        current_branch_path = vec![];
        current_options = vec![DefinedArgOption::new(
          arg
            .description
            .first()
            .or(Some(&String::from("")))
            .unwrap()
            .clone(),
          arg_type,
          arg.optional,
          arg.plural,
        )];
      } else {
        current_options.push(DefinedArgOption::new(
          arg
            .description
            .first()
            .or(Some(&String::from("")))
            .unwrap()
            .clone(),
          arg_type,
          arg.optional,
          arg.plural,
        ));
      }
      is_or = false;
    } else {
      current_branch_path.push(DefinedArg::new(current_options));
      current_options = vec![DefinedArgOption::new(
        arg
          .description
          .first()
          .or(Some(&String::from("")))
          .unwrap()
          .clone(),
        arg_type,
        arg.optional,
        arg.plural,
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

  let mut tags = vec![];
  for tag in &action.tags {
    let mut options = vec![];
    for option in &tag.options {
      options.push(option.name.clone());
    }

    let dfrs_name = to_camel_case(&tag.name);
    let new_tag = DefinedTag::new(
      &dfrs_name,
      &tag.name,
      tag.slot,
      options,
      tag.default_option.clone(),
    );
    tags.push(new_tag);
  }

  let mut return_type = None;
  if !action.icon.return_values.is_empty() {
    let entry = &action.icon.return_values.get(0).unwrap();
    return_type = Some(match entry.return_type.as_str() {
      "NUMBER" => ArgType::NUMBER,
      "COMPONENT" => ArgType::TEXT,
      "TEXT" => ArgType::STRING,
      "LOCATION" => ArgType::LOCATION,
      "POTION" => ArgType::POTION,
      "SOUND" => ArgType::SOUND,
      "VECTOR" => ArgType::VECTOR,
      "PARTICLE" => ArgType::PARTICLE,
      "ITEM" => ArgType::ITEM,
      "LIST" => ArgType::VARIABLE,
      "DICT" => ArgType::VARIABLE,
      "ANY_TYPE" => ArgType::ANY,
      err => panic!("Unknown return type: {:?}", err),
    });
  }

  if return_type.is_none() && action.icon.arguments.len() > 0 {
    match action.icon.arguments.get(0).unwrap().description.last() {
      Some(desc) => {
        if desc.ends_with("to set") {
          // TODO improve this argtype
          return_type = Some(ArgType::ANY);
        }
      }
      None => {}
    }
  }

  let name = to_dfrs_name(&action.name);

  // TODO better arg/return types, include worksWith, example, requiredRank, requireTokens, requireRankAndTokens
  let mut description = String::new();
  description.push_str(&format!("### {name}  \n"));
  description.push_str(&format!("{}  \n", action.icon.description.join(" ")));
  if !action.icon.additional_info.is_empty() {
    for info in &action.icon.additional_info {
      description.push_str(&format!("{}  \n", info.join(" ")));
    }
  }
  if !action.icon.arguments.is_empty() {
    description.push_str(&format!("  ***  \n"));
    description.push_str(&format!(
      "### Arguments:  \n{}  \n",
      action
        .icon
        .arguments
        .iter()
        .map(|arg| if arg.arg_type == "OR" {
          "OR".to_owned()
        } else if arg.arg_type == "" {
          "  \n".to_owned()
        } else {
          format!(
            "{} - {}{}{}",
            arg.description.join(" "),
            arg.arg_type,
            if arg.plural { "*" } else { "" },
            if arg.optional { "?" } else { "" }
          )
        })
        .collect::<Vec<String>>()
        .join("  \n")
    ));
  }
  if !action.tags.is_empty() {
    description.push_str(&format!("  ***  \n"));
    description.push_str(&format!(
      "### Tags:  \n{}  \n",
      action
        .tags
        .iter()
        .map(|tag| format!(
          "**{}**  \n{}  \n",
          tag.name,
          tag
            .options
            .iter()
            .map(|option| format!(
              "- {} {}{}",
              option.name,
              if option.name == tag.default_option {
                "_(default)_ "
              } else {
                ""
              },
              if option.icon.description.is_empty() {
                "".into()
              } else {
                format!("- {}", option.icon.description.join(" "))
              }
            ))
            .collect::<Vec<String>>()
            .join("  \n"),
        ))
        .collect::<Vec<String>>()
        .join("  \n")
    ));
  }
  if let Some(returns) = &return_type {
    description.push_str(&format!("  ***  \n"));
    description.push_str(&format!("#### Returns:  \n{returns:?}"));
  }

  Action::new(
    name,
    &action.name,
    action.aliases.clone(),
    branches,
    tags,
    return_type,
    description,
  )
}
