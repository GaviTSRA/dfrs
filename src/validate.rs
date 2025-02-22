use crate::definitions::action_dump::RawActionDump;
use crate::definitions::actions::Action;
use crate::definitions::events::{EntityEvents, PlayerEvents};
use crate::definitions::game_values::GameValues;
use crate::definitions::{DefinedArgBranch, DefinedArgOption};
use crate::node::{ExpressionNode, StartNode, VariableType};
use crate::{
  definitions::{action_dump::ActionDump, ArgType, DefinedArg},
  node::{
    ActionNode, ActionType, Arg, ArgValue, CallNode, ConditionalNode, ConditionalType, EventNode,
    Expression, FileNode, RepeatNode,
  },
  token::Position,
};

#[derive(Debug)]
pub enum ValidateError {
  UnknownEvent {
    node: EventNode,
  },
  UnknownAction {
    name: String,
    start_pos: Position,
    end_pos: Position,
  },
  UnknownGameValue {
    start_pos: Position,
    end_pos: Position,
    game_value: String,
  },
  MissingArgument {
    options: Vec<String>,
    start_pos: Position,
    end_pos: Position,
  },
  WrongArgumentType {
    args: Vec<Arg>,
    index: i32,
    options: Vec<DefinedArgOption>,
    found_type: ArgType,
  },
  TooManyArguments {
    name: String,
    start_pos: Position,
    end_pos: Position,
  },
  InvalidTagOption {
    tag_name: String,
    provided: String,
    options: Vec<String>,
    start_pos: Position,
    end_pos: Position,
  },
  UnknownTag {
    tag_name: String,
    available: Vec<String>,
    start_pos: Position,
    end_pos: Position,
  },
}

enum PathError {
  MissingArgument {
    options: Vec<String>,
  },
  WrongArgumentType {
    found_type: ArgType,
    options: Vec<DefinedArgOption>,
  },
  UnknownGameValue {
    start_pos: Position,
    end_pos: Position,
    game_value: String,
  },
}

pub struct Validator {
  player_events: PlayerEvents,
  entity_events: EntityEvents,
  action_dump: ActionDump,
  game_values: GameValues,
}

impl Validator {
  pub fn new() -> Validator {
    let action_dump = RawActionDump::load();
    Validator {
      player_events: PlayerEvents::new(&action_dump),
      entity_events: EntityEvents::new(&action_dump),
      action_dump: ActionDump::new(&action_dump),
      game_values: GameValues::new(&action_dump),
    }
  }
  pub fn validate(&self, mut node: FileNode) -> Result<FileNode, ValidateError> {
    for function in node.functions.iter_mut() {
      for expression in function.expressions.iter_mut() {
        self.validate_expression_node(expression)?;
      }
    }

    for process in node.processes.iter_mut() {
      for expression in process.expressions.iter_mut() {
        self.validate_expression_node(expression)?;
      }
    }

    for event in node.events.iter_mut() {
      let mut actual_event;

      actual_event = self.player_events.get(event.event.clone());
      match actual_event {
        Some(actual) => {
          actual.df_name.clone_into(&mut event.event);
          event.event_type = Some(ActionType::Player);
        }
        None => {
          actual_event = self.entity_events.get(event.event.clone());
          match actual_event {
            Some(actual) => {
              actual.df_name.clone_into(&mut event.event);
              event.event_type = Some(ActionType::Entity);
            }
            None => {
              return Err(ValidateError::UnknownEvent {
                node: event.clone(),
              })
            }
          }
        }
      }

      for expression in event.expressions.iter_mut() {
        self.validate_expression_node(expression)?
      }
    }

    Ok(node)
  }

  fn validate_expression_node(
    &self,
    expression_node: &mut ExpressionNode,
  ) -> Result<(), ValidateError> {
    match expression_node.node.clone() {
      Expression::Action { node } => {
        expression_node.node = Expression::Action {
          node: self.validate_action_node(node)?,
        };
      }
      Expression::Conditional { node } => {
        expression_node.node = Expression::Conditional {
          node: self.validate_conditional_node(node)?,
        }
      }
      Expression::Call { node } => {
        expression_node.node = Expression::Call {
          node: self.validate_call(node)?,
        }
      }
      Expression::Start { node } => {
        expression_node.node = Expression::Start {
          node: self.validate_start(node)?,
        }
      }
      Expression::Repeat { node } => {
        expression_node.node = Expression::Repeat {
          node: self.validate_repeat_node(node)?,
        }
      }
      Expression::Variable { node } => {
        if let Some(mut action) = node.action.clone() {
          action.args.insert(
            0,
            Arg {
              arg_type: ArgType::VARIABLE,
              index: -1,
              value: ArgValue::Variable {
                name: node.df_name,
                scope: match node.var_type {
                  VariableType::Line => "line".to_owned(),
                  VariableType::Local => "local".to_owned(),
                  VariableType::Game => "unsaved".to_owned(),
                  VariableType::Save => "saved".to_owned(),
                },
              },
              start_pos: node.start_pos,
              end_pos: node.end_pos,
            },
          );
          expression_node.node = Expression::Action {
            node: self.validate_action_node(action)?,
          }
        }
      }
    }
    Ok(())
  }

  fn validate_action_node(&self, mut action_node: ActionNode) -> Result<ActionNode, ValidateError> {
    let mut action = match action_node.action_type {
      ActionType::Player => self.action_dump.player_actions.get(&action_node.name),
      ActionType::Entity => self.action_dump.entity_actions.get(&action_node.name),
      ActionType::Game => self.action_dump.game_actions.get(&action_node.name),
      ActionType::Variable => self.action_dump.variable_actions.get(&action_node.name),
      ActionType::Control => self.action_dump.control_actions.get(&action_node.name),
      ActionType::Select => self.action_dump.select_actions.get(&action_node.name),
    };

    let mut old_args = vec![];
    let mut old_name = "".into();
    let mut was_condition = false;

    if !action_node.args.is_empty()
      && action_node.args.get(0).unwrap().arg_type == ArgType::CONDITION
    {
      match action_node.args.get(0).unwrap().clone().value {
        ArgValue::Condition {
          name,
          args,
          conditional_type,
          ..
        } => {
          old_args = action_node.args;

          match action {
            Some(res) => old_name = res.df_name.clone(),
            None => {
              return Err(ValidateError::UnknownAction {
                name: action_node.name,
                start_pos: action_node.start_pos,
                end_pos: action_node.end_pos,
              })
            }
          };

          action_node.args = args;
          was_condition = true;
          action = match conditional_type {
            ConditionalType::Player => self.action_dump.player_conditionals.get(&name),
            ConditionalType::Entity => self.action_dump.entity_conditionals.get(&name),
            ConditionalType::Game => self.action_dump.game_conditionals.get(&name),
            ConditionalType::Variable => self.action_dump.variable_conditionals.get(&name),
          }
        }
        _ => unreachable!(),
      }
    }

    match action {
      Some(res) => action_node = self.validate_action(action_node, res)?,
      None => {
        return Err(ValidateError::UnknownAction {
          name: action_node.name,
          start_pos: action_node.start_pos,
          end_pos: action_node.end_pos,
        })
      }
    };

    if was_condition {
      match old_args.get(0).unwrap().clone().value {
        ArgValue::Condition {
          selector,
          conditional_type,
          inverted,
          ..
        } => {
          old_args.get_mut(0).unwrap().value = ArgValue::Condition {
            name: action_node.name,
            args: action_node.args.clone(),
            selector,
            conditional_type,
            inverted,
          };
          action_node.args = old_args;
          action_node.name = old_name;
        }
        _ => unreachable!(),
      }
    }

    Ok(action_node)
  }

  fn validate_action(
    &self,
    mut action_node: ActionNode,
    action: &Action,
  ) -> Result<ActionNode, ValidateError> {
    action_node.name.clone_from(&action.df_name);
    action_node.args = self.validate_args(
      action_node.args,
      action,
      action_node.start_pos.clone(),
      action_node.end_pos.clone(),
    )?;
    Ok(action_node)
  }

  fn validate_conditional_node(
    &self,
    mut conditional_node: ConditionalNode,
  ) -> Result<ConditionalNode, ValidateError> {
    let action = match conditional_node.conditional_type {
      ConditionalType::Player => self
        .action_dump
        .player_conditionals
        .get(&conditional_node.name),
      ConditionalType::Entity => self
        .action_dump
        .entity_conditionals
        .get(&conditional_node.name),
      ConditionalType::Game => self
        .action_dump
        .game_conditionals
        .get(&conditional_node.name),
      ConditionalType::Variable => self
        .action_dump
        .variable_conditionals
        .get(&conditional_node.name),
    };

    match action {
      Some(res) => conditional_node = self.validate_conditional(conditional_node, res)?,
      None => {
        return Err(ValidateError::UnknownAction {
          name: conditional_node.name,
          start_pos: conditional_node.start_pos,
          end_pos: conditional_node.end_pos,
        })
      }
    };

    for expression in conditional_node.expressions.iter_mut() {
      self.validate_expression_node(expression)?;
    }

    for expression in conditional_node.else_expressions.iter_mut() {
      self.validate_expression_node(expression)?;
    }

    Ok(conditional_node)
  }

  fn validate_conditional(
    &self,
    mut conditional_node: ConditionalNode,
    action: &Action,
  ) -> Result<ConditionalNode, ValidateError> {
    conditional_node.name.clone_from(&action.df_name);
    conditional_node.args = self.validate_args(
      conditional_node.args,
      action,
      conditional_node.start_pos.clone(),
      conditional_node.end_pos.clone(),
    )?;
    Ok(conditional_node)
  }

  fn validate_call(&self, mut call_node: CallNode) -> Result<CallNode, ValidateError> {
    // TODO proper validation
    let mut args = vec![];
    for arg in &call_node.args {
      args.push(DefinedArg {
        options: vec![DefinedArgOption::new("".into(), ArgType::ANY, false, false)],
      })
    }
    let action = Action {
      df_name: "internal".into(),
      dfrs_name: "internal".into(),
      aliases: vec![],
      args: vec![DefinedArgBranch { paths: vec![args] }],
      tags: vec![],
      has_conditional_arg: false,
    };
    call_node.args = self.validate_args(
      call_node.args,
      &action,
      call_node.start_pos.clone(),
      call_node.end_pos.clone(),
    )?;
    Ok(call_node)
  }

  fn validate_start(&self, mut start_node: StartNode) -> Result<StartNode, ValidateError> {
    start_node.args = self.validate_args(
      start_node.args,
      &self.action_dump.start_process_action,
      start_node.start_pos.clone(),
      start_node.end_pos.clone(),
    )?;
    Ok(start_node)
  }

  fn validate_repeat_node(&self, mut repeat_node: RepeatNode) -> Result<RepeatNode, ValidateError> {
    let mut action = self.action_dump.repeats.get(&repeat_node.name);
    let mut old_args = vec![];
    let mut old_name = "".into();
    let mut was_condition = false;

    if !repeat_node.args.is_empty()
      && repeat_node.args.get(0).unwrap().arg_type == ArgType::CONDITION
    {
      match repeat_node.args.get(0).unwrap().clone().value {
        ArgValue::Condition {
          name,
          args,
          conditional_type,
          ..
        } => {
          old_args = repeat_node.args;

          match action {
            Some(res) => old_name = res.df_name.clone(),
            None => {
              return Err(ValidateError::UnknownAction {
                name: repeat_node.name,
                start_pos: repeat_node.start_pos,
                end_pos: repeat_node.end_pos,
              })
            }
          };

          repeat_node.args = args;
          was_condition = true;
          action = match conditional_type {
            ConditionalType::Player => self.action_dump.player_conditionals.get(&name),
            ConditionalType::Entity => self.action_dump.entity_conditionals.get(&name),
            ConditionalType::Game => self.action_dump.game_conditionals.get(&name),
            ConditionalType::Variable => self.action_dump.variable_conditionals.get(&name),
          }
        }
        _ => unreachable!(),
      }
    }

    match action {
      Some(res) => repeat_node = self.validate_repeat(repeat_node, res)?,
      None => {
        return Err(ValidateError::UnknownAction {
          name: repeat_node.name,
          start_pos: repeat_node.start_pos,
          end_pos: repeat_node.end_pos,
        })
      }
    };
    if was_condition {
      match old_args.get(0).unwrap().clone().value {
        ArgValue::Condition {
          selector,
          conditional_type,
          inverted,
          ..
        } => {
          old_args.get_mut(0).unwrap().value = ArgValue::Condition {
            name: repeat_node.name,
            args: repeat_node.args.clone(),
            selector,
            conditional_type,
            inverted,
          };
          repeat_node.args = old_args;
          repeat_node.name = old_name;
        }
        _ => unreachable!(),
      }
    }

    for expression in repeat_node.expressions.iter_mut() {
      self.validate_expression_node(expression)?;
    }

    Ok(repeat_node)
  }

  fn validate_repeat(
    &self,
    mut repeat_node: RepeatNode,
    action: &Action,
  ) -> Result<RepeatNode, ValidateError> {
    repeat_node.name.clone_from(&action.df_name);
    repeat_node.args = self.validate_args(
      repeat_node.args,
      action,
      repeat_node.start_pos.clone(),
      repeat_node.end_pos.clone(),
    )?;
    Ok(repeat_node)
  }

  fn validate_args(
    &self,
    input_args: Vec<Arg>,
    action: &Action,
    start_pos: Position,
    end_pos: Position,
  ) -> Result<Vec<Arg>, ValidateError> {
    let mut node_args = vec![];
    let all_provided_args: Vec<Arg> = input_args.clone();
    let mut args: Vec<Arg> = vec![];
    let mut index: i32 = 0;

    let mut tags: Vec<Arg> = vec![];

    for arg in input_args {
      if arg.arg_type == ArgType::TAG {
        tags.push(arg);
      } else {
        node_args.push(arg)
      }
    }

    for branch in &action.args {
      let mut completed = false;
      let mut errors = vec![];

      for (path_index, path) in branch.paths.iter().enumerate() {
        let result = self.try_path(&mut path.clone(), &mut node_args, &mut args, &mut index);
        match result {
          Ok(()) => completed = true,
          Err(err) => {
            if path_index + 1 == branch.paths.len() {
              errors.push(err);
            }
          }
        }
      }

      if !completed {
        let err = errors.get(0).unwrap();
        return match err {
          PathError::MissingArgument { options } => Err(ValidateError::MissingArgument {
            options: options.clone(),
            start_pos,
            end_pos,
          }),
          PathError::WrongArgumentType {
            found_type,
            options,
          } => Err(ValidateError::WrongArgumentType {
            args: all_provided_args,
            index,
            options: options.clone(),
            found_type: found_type.clone(),
          }),
          PathError::UnknownGameValue {
            game_value,
            start_pos,
            end_pos,
          } => Err(ValidateError::UnknownGameValue {
            game_value: game_value.clone(),
            start_pos: start_pos.clone(),
            end_pos: end_pos.clone(),
          }),
        };
      }
    }

    if !node_args.is_empty() {
      for val in node_args.clone() {
        if val.arg_type != ArgType::TAG {
          return Err(ValidateError::TooManyArguments {
            name: action.dfrs_name.clone(),
            start_pos,
            end_pos,
          });
        }
        tags.push(val)
      }
    }

    for given_tag in tags.clone() {
      match given_tag.value {
        ArgValue::Tag {
          tag: tag_name,
          value: _,
          definition: _,
          name_end_pos,
          value_start_pos: _,
        } => {
          let mut found = false;
          let mut available = vec![];
          for tag in action.tags.clone() {
            available.push(tag.dfrs_name.clone());
            if tag.dfrs_name == tag_name {
              found = true;
            }
          }
          if !found {
            return Err(ValidateError::UnknownTag {
              tag_name,
              available,
              start_pos: given_tag.start_pos,
              end_pos: name_end_pos,
            });
          }
        }
        _ => unreachable!(),
      }
    }

    for tag in action.tags.clone() {
      let mut matched = false;
      for given_tag in tags.clone() {
        match given_tag.value {
          ArgValue::Tag {
            tag: tag_name,
            value,
            name_end_pos,
            value_start_pos,
            ..
          } => {
            let actual = match value.clone().as_ref() {
              ArgValue::Text { text } => text.clone(),
              err => {
                return Err(ValidateError::InvalidTagOption {
                  tag_name,
                  provided: format!("{err:?}"),
                  options: tag.options,
                  start_pos: value_start_pos,
                  end_pos: given_tag.end_pos,
                })
              }
            };
            if tag.dfrs_name == tag_name {
              if tag.options.contains(&actual) {
                matched = true;
                args.push(Arg {
                  arg_type: ArgType::TAG,
                  value: ArgValue::Tag {
                    tag: tag.df_name.clone(),
                    value,
                    definition: Some(tag.clone()),
                    name_end_pos,
                    value_start_pos,
                  },
                  index: tag.slot as i32,
                  start_pos: given_tag.start_pos,
                  end_pos: given_tag.end_pos,
                });
              } else {
                return Err(ValidateError::InvalidTagOption {
                  tag_name,
                  provided: actual,
                  options: tag.options,
                  start_pos: value_start_pos,
                  end_pos: given_tag.end_pos,
                });
              }
            }
          }
          _ => unreachable!(),
        }
      }
      if !matched {
        let data = Box::new(ArgValue::Text {
          text: tag.default.clone(),
        });
        args.push(Arg {
          arg_type: ArgType::TAG,
          value: ArgValue::Tag {
            tag: tag.df_name.clone(),
            value: data,
            definition: Some(tag.clone()),
            name_end_pos: Position::new(1, 1),
            value_start_pos: Position::new(1, 1),
          },
          index: tag.slot as i32,
          start_pos: Position::new(1, 1),
          end_pos: Position::new(1, 1),
        });
      }
    }

    Ok(args)
  }

  fn try_path(
    &self,
    path: &mut Vec<DefinedArg>,
    node_args: &mut Vec<Arg>,
    args: &mut Vec<Arg>,
    index: &mut i32,
  ) -> Result<(), PathError> {
    let mut missing: Option<Vec<String>> = None;
    let mut wrong_type: Option<(ArgType, Vec<DefinedArgOption>)> = None;
    let mut previous_plural = None;

    for arg in path {
      let mut arg_complete = false;

      loop {
        for (option_index, option) in arg.options.iter().enumerate() {
          if node_args.len() == 0 {
            if option.optional {
              arg_complete = true;
              break;
            }
            missing = Some(
              arg
                .options
                .iter()
                .map(|option| return option.name.clone())
                .collect(),
            );
            break;
          }

          let mut current_arg = node_args.get(0).unwrap().clone();

          if current_arg.arg_type == ArgType::EMPTY {
            if option.optional {
              *index += 1;
              node_args.remove(0);
              break;
            }
            missing = Some(
              arg
                .options
                .iter()
                .map(|option| return option.name.clone())
                .collect(),
            );
          }

          if let ArgValue::GameValue {
            df_name,
            dfrs_name,
            selector,
            selector_end_pos,
          } = current_arg.value.clone()
          {
            let actual_game_value = self.game_values.get(dfrs_name.clone());
            match actual_game_value {
              Some(res) => {
                current_arg.value = ArgValue::GameValue {
                  df_name: Some(res.df_name.clone()),
                  dfrs_name,
                  selector,
                  selector_end_pos,
                };
                current_arg.arg_type = res.value_type.clone();
              }
              None => {
                return Err(PathError::UnknownGameValue {
                  game_value: dfrs_name,
                  start_pos: current_arg.start_pos,
                  end_pos: current_arg.end_pos,
                })
              }
            }
          }

          if current_arg.arg_type == option.arg_type
            || current_arg.arg_type == ArgType::VARIABLE
            || option.arg_type == ArgType::ANY
            || current_arg.arg_type == ArgType::GameValue
          {
            arg_complete = true;
            previous_plural = if option.plural {
              Some(option.arg_type.clone())
            } else {
              None
            };
            node_args.remove(0);
            current_arg.index = *index;
            args.push(current_arg);
            *index += 1;
            break;
          }

          if option_index + 1 == arg.options.len() {
            if let Some(plural_type) = &previous_plural {
              if plural_type == &current_arg.arg_type {
                node_args.remove(0);
                current_arg.index = *index;
                args.push(current_arg);
                *index += 1;
                break;
              }
            }
            if option.optional {
              arg_complete = true;
              break;
            }
            wrong_type = Some((current_arg.arg_type.clone(), arg.options.clone()));
          }
        }

        if arg_complete || missing.is_some() || wrong_type.is_some() {
          break;
        }
      }

      if !arg_complete {
        if let Some(missing) = missing {
          return Err(PathError::MissingArgument {
            options: missing.clone(),
          });
        }
        if let Some(wrong_type) = wrong_type {
          return Err(PathError::WrongArgumentType {
            found_type: wrong_type.0.clone(),
            options: wrong_type.1.clone(),
          });
        }
      }
    }

    if let Some(plural_type) = &previous_plural {
      while node_args.len() > 0 {
        let current_arg = node_args.get(0).unwrap();

        if plural_type == &current_arg.arg_type
          || plural_type == &ArgType::ANY
          || current_arg.arg_type == ArgType::VARIABLE
          || current_arg.arg_type == ArgType::GameValue
        {
          let mut ok_arg = node_args.remove(0);
          ok_arg.index = *index;
          *index += 1;
          args.push(ok_arg);
        } else {
          break;
        }
      }
    }

    Ok(())
  }
}

// TODO validate potions, sounds, particles etc

#[cfg(test)]
mod tests {
  use super::*;

  fn get_single_arg(arg_type: ArgType, optional: bool, plural: bool) -> DefinedArg {
    DefinedArg {
      options: vec![DefinedArgOption {
        arg_type,
        name: "".to_owned(),
        optional,
        plural,
      }],
    }
  }

  fn get_single_arg_action(optional: bool, plural: bool) -> Action {
    Action::new(
      String::from(""),
      "",
      vec![],
      vec![DefinedArgBranch::new(vec![vec![get_single_arg(
        ArgType::NUMBER,
        optional,
        plural,
      )]])],
      vec![],
      false,
    )
  }

  fn get_triple_arg_action(optional: bool, plural: bool) -> Action {
    Action::new(
      String::from(""),
      "",
      vec![],
      vec![DefinedArgBranch::new(vec![vec![
        get_single_arg(ArgType::NUMBER, false, plural),
        get_single_arg(ArgType::NUMBER, optional, false),
        get_single_arg(ArgType::TEXT, false, false),
      ]])],
      vec![],
      false,
    )
  }

  #[test]
  pub fn single_arg_valid() {
    let validator: Validator = Validator::new();
    let correct_arg = vec![Arg {
      arg_type: ArgType::NUMBER,
      value: ArgValue::Number { number: 1.0 },
      index: 0,
      start_pos: Position::new(0, 0),
      end_pos: Position::new(0, 0),
    }];

    let result = validator.validate_args(
      correct_arg,
      &get_single_arg_action(false, false),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }

  #[test]
  pub fn single_arg_invalid() {
    let incorrect_arg = vec![Arg {
      arg_type: ArgType::TEXT,
      value: ArgValue::Text {
        text: "".to_owned(),
      },
      index: 0,
      start_pos: Position::new(0, 0),
      end_pos: Position::new(0, 0),
    }];

    let validator = Validator::new();
    let result = validator.validate_args(
      incorrect_arg,
      &get_single_arg_action(false, false),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    match result {
      Ok(_) => panic!("Incorrect arg should fail"),
      Err(err) => match err {
        ValidateError::WrongArgumentType { .. } => {}
        _ => panic!("Should be wrong argument type"),
      },
    }
  }

  #[test]
  pub fn single_arg_too_many() {
    let too_many_args = vec![
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
    ];

    let validator = Validator::new();
    let result = validator.validate_args(
      too_many_args,
      &get_single_arg_action(false, false),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    match result {
      Ok(_) => panic!("Too many args should fail"),
      Err(err) => match err {
        ValidateError::TooManyArguments { .. } => {}
        _ => panic!("Should be too many arguments"),
      },
    }
  }

  #[test]
  pub fn single_arg_missing() {
    let validator = Validator::new();
    let result = validator.validate_args(
      vec![],
      &get_single_arg_action(false, false),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    match result {
      Ok(_) => panic!("Missing arg should fail"),
      Err(err) => match err {
        ValidateError::MissingArgument { .. } => {}
        _ => panic!("Should be missing argument"),
      },
    }
  }

  #[test]
  pub fn single_arg_optional_missing() {
    let validator = Validator::new();
    let result = validator.validate_args(
      vec![],
      &get_single_arg_action(true, false),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }

  #[test]
  pub fn multi_arg_valid() {
    let validator: Validator = Validator::new();
    let correct_arg = vec![
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::TEXT,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
    ];

    let result = validator.validate_args(
      correct_arg,
      &get_triple_arg_action(false, false),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }

  #[test]
  pub fn multi_arg_optional_valid() {
    let validator: Validator = Validator::new();
    let correct_arg = vec![
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::TEXT,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
    ];

    let result = validator.validate_args(
      correct_arg,
      &get_triple_arg_action(true, false),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }

  #[test]
  pub fn multi_arg_optional_missing_valid() {
    let validator: Validator = Validator::new();
    let correct_arg = vec![
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::TEXT,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
    ];

    let result = validator.validate_args(
      correct_arg,
      &get_triple_arg_action(true, false),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }

  #[test]
  pub fn multi_arg_optional_missing_invalid() {
    let validator: Validator = Validator::new();
    let correct_arg = vec![
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
    ];

    let result = validator.validate_args(
      correct_arg,
      &get_triple_arg_action(true, false),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    match result {
      Ok(_) => panic!("Missing arg should fail"),
      Err(err) => match err {
        ValidateError::MissingArgument { .. } => {}
        _ => panic!("Should be missing argument"),
      },
    }
  }

  #[test]
  pub fn single_arg_or() {
    let action = Action::new(
      String::from(""),
      "",
      vec![],
      vec![DefinedArgBranch::new(vec![vec![DefinedArg {
        options: vec![
          DefinedArgOption {
            arg_type: ArgType::NUMBER,
            name: "".to_owned(),
            optional: false,
            plural: false,
          },
          DefinedArgOption {
            arg_type: ArgType::TEXT,
            name: "".to_owned(),
            optional: false,
            plural: false,
          },
        ],
      }]])],
      vec![],
      false,
    );

    let validator: Validator = Validator::new();
    let result = validator.validate_args(
      vec![Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      }],
      &action,
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());

    let result = validator.validate_args(
      vec![Arg {
        arg_type: ArgType::TEXT,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      }],
      &action,
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }

  #[test]
  pub fn multi_arg_or() {
    let action = Action::new(
      String::from(""),
      "",
      vec![],
      vec![
        DefinedArgBranch::new(vec![vec![DefinedArg {
          options: vec![
            DefinedArgOption {
              arg_type: ArgType::NUMBER,
              name: "".to_owned(),
              optional: false,
              plural: false,
            },
            DefinedArgOption {
              arg_type: ArgType::TEXT,
              name: "".to_owned(),
              optional: false,
              plural: false,
            },
          ],
        }]]),
        DefinedArgBranch::new(vec![vec![DefinedArg {
          options: vec![
            DefinedArgOption {
              arg_type: ArgType::NUMBER,
              name: "".to_owned(),
              optional: false,
              plural: false,
            },
            DefinedArgOption {
              arg_type: ArgType::TEXT,
              name: "".to_owned(),
              optional: false,
              plural: false,
            },
          ],
        }]]),
      ],
      vec![],
      false,
    );

    let validator: Validator = Validator::new();
    let result = validator.validate_args(
      vec![
        Arg {
          arg_type: ArgType::NUMBER,
          value: ArgValue::Number { number: 1.0 },
          index: 0,
          start_pos: Position::new(0, 0),
          end_pos: Position::new(0, 0),
        },
        Arg {
          arg_type: ArgType::NUMBER,
          value: ArgValue::Number { number: 1.0 },
          index: 0,
          start_pos: Position::new(0, 0),
          end_pos: Position::new(0, 0),
        },
      ],
      &action,
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());

    let result = validator.validate_args(
      vec![
        Arg {
          arg_type: ArgType::TEXT,
          value: ArgValue::Number { number: 1.0 },
          index: 0,
          start_pos: Position::new(0, 0),
          end_pos: Position::new(0, 0),
        },
        Arg {
          arg_type: ArgType::NUMBER,
          value: ArgValue::Number { number: 1.0 },
          index: 0,
          start_pos: Position::new(0, 0),
          end_pos: Position::new(0, 0),
        },
      ],
      &action,
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());

    let result = validator.validate_args(
      vec![
        Arg {
          arg_type: ArgType::TEXT,
          value: ArgValue::Number { number: 1.0 },
          index: 0,
          start_pos: Position::new(0, 0),
          end_pos: Position::new(0, 0),
        },
        Arg {
          arg_type: ArgType::TEXT,
          value: ArgValue::Number { number: 1.0 },
          index: 0,
          start_pos: Position::new(0, 0),
          end_pos: Position::new(0, 0),
        },
      ],
      &action,
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());

    let result = validator.validate_args(
      vec![
        Arg {
          arg_type: ArgType::NUMBER,
          value: ArgValue::Number { number: 1.0 },
          index: 0,
          start_pos: Position::new(0, 0),
          end_pos: Position::new(0, 0),
        },
        Arg {
          arg_type: ArgType::TEXT,
          value: ArgValue::Number { number: 1.0 },
          index: 0,
          start_pos: Position::new(0, 0),
          end_pos: Position::new(0, 0),
        },
      ],
      &action,
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }

  #[test]
  pub fn single_arg_plural() {
    let single_arg = vec![Arg {
      arg_type: ArgType::NUMBER,
      value: ArgValue::Number { number: 1.0 },
      index: 0,
      start_pos: Position::new(0, 0),
      end_pos: Position::new(0, 0),
    }];
    let multi_arg = vec![
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
    ];

    let validator = Validator::new();
    let result = validator.validate_args(
      single_arg,
      &get_single_arg_action(false, true),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());

    let result = validator.validate_args(
      multi_arg,
      &get_single_arg_action(false, true),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }

  #[test]
  pub fn multi_arg_plural() {
    let single_arg = vec![
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::TEXT,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
    ];
    let multi_arg = vec![
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
      Arg {
        arg_type: ArgType::TEXT,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      },
    ];

    let validator = Validator::new();
    let result = validator.validate_args(
      single_arg,
      &get_triple_arg_action(false, true),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());

    let result = validator.validate_args(
      multi_arg,
      &get_triple_arg_action(false, true),
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }

  #[test]
  pub fn branch() {
    let action = Action::new(
      String::from(""),
      "",
      vec![],
      vec![DefinedArgBranch::new(vec![
        vec![DefinedArg {
          options: vec![DefinedArgOption {
            arg_type: ArgType::NUMBER,
            name: "".to_owned(),
            optional: false,
            plural: false,
          }],
        }],
        vec![DefinedArg {
          options: vec![DefinedArgOption {
            arg_type: ArgType::TEXT,
            name: "".to_owned(),
            optional: false,
            plural: false,
          }],
        }],
      ])],
      vec![],
      false,
    );

    let validator = Validator::new();
    let result = validator.validate_args(
      vec![Arg {
        arg_type: ArgType::NUMBER,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      }],
      &action,
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());

    let result = validator.validate_args(
      vec![Arg {
        arg_type: ArgType::TEXT,
        value: ArgValue::Number { number: 1.0 },
        index: 0,
        start_pos: Position::new(0, 0),
        end_pos: Position::new(0, 0),
      }],
      &action,
      Position::new(0, 0),
      Position::new(0, 0),
    );
    assert!(result.is_ok());
  }
}
