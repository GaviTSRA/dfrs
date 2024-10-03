use crate::{definitions::{action_dump::{Action, ActionDump}, ArgType, DefinedArg}, node::{ActionNode, ActionType, Arg, ArgValue, CallNode, ConditionalNode, ConditionalType, EventNode, Expression, FileNode, RepeatNode}, token::Position};
use crate::definitions::action_dump::RawActionDump;
use crate::definitions::events::{EntityEvents, PlayerEvents};
use crate::definitions::game_values::GameValues;
use crate::node::{ExpressionNode, StartNode};

pub enum ValidateError {
    UnknownEvent { node: EventNode },
    UnknownAction { name: String, start_pos: Position, end_pos: Position },
    UnknownGameValue { start_pos: Position, end_pos: Position, game_value: String },
    MissingArgument { name: String, start_pos: Position, end_pos: Position },
    WrongArgumentType { args: Vec<Arg>, index: i32, name: String, expected_types: Vec<ArgType>, found_type: ArgType },
    TooManyArguments { name: String, start_pos: Position, end_pos: Position },
    InvalidTagOption { tag_name: String, provided: String, options: Vec<String>, start_pos: Position, end_pos: Position },
    UnknownTag { tag_name: String, available: Vec<String>, start_pos: Position, end_pos: Position }
}

pub struct Validator {
    player_events: PlayerEvents,
    entity_events: EntityEvents,

    action_dump: ActionDump,

    game_values: GameValues
}

impl Validator {
    pub fn new() -> Validator {
        let action_dump = RawActionDump::load();
        Validator {
            player_events: PlayerEvents::new(&action_dump),
            entity_events: EntityEvents::new(&action_dump),

            action_dump: ActionDump::new(&action_dump),

            game_values: GameValues::new(&action_dump)
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
                            return Err(ValidateError::UnknownEvent { node: event.clone() })
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

    fn validate_expression_node(&self, mut expression_node: &mut ExpressionNode) -> Result<(), ValidateError> {
        match expression_node.node.clone() {
            Expression::Action { node } => {
                expression_node.node = Expression::Action { node: self.validate_action_node(node)? };
            }
            Expression::Conditional { node } => {
                expression_node.node = Expression::Conditional { node: self.validate_conditional_node(node)? }
            }
            Expression::Call { node } => {
                expression_node.node = Expression::Call { node: self.validate_call(node)? }
            }
            Expression::Start { node } => {
                expression_node.node = Expression::Start { node: self.validate_start(node)? }
            }
            Expression::Repeat { node } => {
                expression_node.node = Expression::Repeat { node: self.validate_repeat_node(node)? }
            }
            Expression::Variable { .. } => {}
        }
        Ok(())
    }

    fn validate_action_node(&self, mut action_node: ActionNode) -> Result<ActionNode, ValidateError> {
        let mut action = match action_node.action_type {
            ActionType::Player => {
                self.action_dump.player_actions.get(action_node.clone().name)
            }
            ActionType::Entity => {
                self.action_dump.entity_actions.get(action_node.clone().name)
            }
            ActionType::Game => {
                self.action_dump.game_actions.get(action_node.clone().name)
            }
            ActionType::Variable => {
                self.action_dump.variable_actions.get(action_node.clone().name)
            }
            ActionType::Control => {
                self.action_dump.control_actions.get(action_node.clone().name)
            }
            ActionType::Select => {
                self.action_dump.select_actions.get(action_node.clone().name)
            }
        };

        let mut old_args = vec![];
        let mut old_name = "".into();
        let mut was_condition = false;

        if !action_node.args.is_empty() && action_node.args.get(0).unwrap().arg_type == ArgType::CONDITION {
            match action_node.args.get(0).unwrap().clone().value {
                ArgValue::Condition { name, args, conditional_type, .. } => {
                    old_args = action_node.args;
                    
                    match action {
                        Some(res) => old_name = res.df_name.clone(),
                        None => return Err(ValidateError::UnknownAction { name: action_node.name, start_pos: action_node.start_pos, end_pos: action_node.end_pos })
                    };

                    action_node.args = args;
                    was_condition = true;
                    action = match conditional_type {
                        ConditionalType::Player => self.action_dump.player_conditionals.get(name),
                        ConditionalType::Entity => self.action_dump.entity_conditionals.get(name),
                        ConditionalType::Game => self.action_dump.game_conditionals.get(name),
                        ConditionalType::Variable => self.action_dump.variable_conditionals.get(name),
                    }
                }
                _ => unreachable!()
            }
        }

        match action {
            Some(res) => action_node = self.validate_action(action_node, res)?,
            None => return Err(ValidateError::UnknownAction { name: action_node.name, start_pos: action_node.start_pos, end_pos: action_node.end_pos })
        };

        if was_condition {
            match old_args.get(0).unwrap().clone().value {
                ArgValue::Condition { selector, conditional_type, inverted, .. } => {
                    old_args.get_mut(0).unwrap().value = ArgValue::Condition { name: action_node.name, args: action_node.args.clone(), selector, conditional_type, inverted };
                    action_node.args = old_args;
                    action_node.name = old_name;
                }
                _ => unreachable!()
            }
        }

        Ok(action_node)
    }

    fn validate_action(&self, mut action_node: ActionNode, action: &Action) -> Result<ActionNode, ValidateError> {
        action_node.name.clone_from(&action.df_name);
        action_node.args = self.validate_args(action_node.args, action, action_node.start_pos.clone(), action_node.end_pos.clone())?;
        Ok(action_node)
    }

    fn validate_conditional_node(&self, mut conditional_node: ConditionalNode) -> Result<ConditionalNode, ValidateError> {
        let action = match conditional_node.conditional_type {
            ConditionalType::Player => {
                self.action_dump.player_conditionals.get(conditional_node.clone().name)
            }
            ConditionalType::Entity => {
                self.action_dump.entity_conditionals.get(conditional_node.clone().name)
            }
            ConditionalType::Game => {
                self.action_dump.game_conditionals.get(conditional_node.clone().name)
            }
            ConditionalType::Variable => {
                self.action_dump.variable_conditionals.get(conditional_node.clone().name)
            }
        };

        match action {
            Some(res) => conditional_node = self.validate_conditional(conditional_node, res)?,
            None => return Err(ValidateError::UnknownAction { name: conditional_node.name, start_pos: conditional_node.start_pos, end_pos: conditional_node.end_pos })
        };

        for expression in conditional_node.expressions.iter_mut() {
            self.validate_expression_node(expression)?;
        }

        for expression in conditional_node.else_expressions.iter_mut() {
            self.validate_expression_node(expression)?;
        }

        Ok(conditional_node)
    }

    fn validate_conditional(&self, mut conditional_node: ConditionalNode, action: &Action) -> Result<ConditionalNode, ValidateError> {
        conditional_node.name.clone_from(&action.df_name);
        conditional_node.args = self.validate_args(conditional_node.args, action, conditional_node.start_pos.clone(), conditional_node.end_pos.clone())?;
        Ok(conditional_node)
    }

    fn validate_call(&self, mut call_node: CallNode) -> Result<CallNode, ValidateError> {
        // TODO proper validation
        let mut args = vec![];
        for arg in &call_node.args {
            args.push(DefinedArg {
                arg_types: vec![ArgType::ANY],
                name: "".into(),
                allow_multiple: false,
                optional: false,
            })
        }
        let action = Action {
            df_name: "internal".into(),
            dfrs_name: "internal".into(),
            args,
            tags: vec![],
            has_conditional_arg: false
        };
        call_node.args = self.validate_args(call_node.args, &action, call_node.start_pos.clone(), call_node.end_pos.clone())?;
        Ok(call_node)
    }

    fn validate_start(&self, mut start_node: StartNode) -> Result<StartNode, ValidateError> {
        start_node.args = self.validate_args(start_node.args, &self.action_dump.start_process_action, start_node.start_pos.clone(), start_node.end_pos.clone())?;
        Ok(start_node)
    }

    fn validate_repeat_node(&self, mut repeat_node: RepeatNode) -> Result<RepeatNode, ValidateError> {
        let mut action = self.action_dump.repeats.get(repeat_node.clone().name);
        let mut old_args = vec![];
        let mut old_name = "".into();
        let mut was_condition = false;

        if !repeat_node.args.is_empty() && repeat_node.args.get(0).unwrap().arg_type == ArgType::CONDITION {
            match repeat_node.args.get(0).unwrap().clone().value {
                ArgValue::Condition { name, args, conditional_type, .. } => {
                    old_args = repeat_node.args;
                    
                    match action {
                        Some(res) => old_name = res.df_name.clone(),
                        None => return Err(ValidateError::UnknownAction { name: repeat_node.name, start_pos: repeat_node.start_pos, end_pos: repeat_node.end_pos })
                    };

                    repeat_node.args = args;
                    was_condition = true;
                    action = match conditional_type {
                        ConditionalType::Player => self.action_dump.player_conditionals.get(name),
                        ConditionalType::Entity => self.action_dump.entity_conditionals.get(name),
                        ConditionalType::Game => self.action_dump.game_conditionals.get(name),
                        ConditionalType::Variable => self.action_dump.variable_conditionals.get(name),
                    }
                }
                _ => unreachable!()
            }
        }

        match action {
            Some(res) => repeat_node = self.validate_repeat(repeat_node, res)?,
            None => return Err(ValidateError::UnknownAction { name: repeat_node.name, start_pos: repeat_node.start_pos, end_pos: repeat_node.end_pos })
        };
        if was_condition {
            match old_args.get(0).unwrap().clone().value {
                ArgValue::Condition { selector, conditional_type, inverted, .. } => {
                    old_args.get_mut(0).unwrap().value = ArgValue::Condition { name: repeat_node.name, args: repeat_node.args.clone(), selector, conditional_type, inverted };
                    repeat_node.args = old_args;
                    repeat_node.name = old_name;
                }
                _ => unreachable!()
            }
        }

        for expression in repeat_node.expressions.iter_mut() {
            self.validate_expression_node(expression)?;
        }

        Ok(repeat_node)
    }

    fn validate_repeat(&self, mut repeat_node: RepeatNode, action: &Action) -> Result<RepeatNode, ValidateError> {
        repeat_node.name.clone_from(&action.df_name);
        repeat_node.args = self.validate_args(repeat_node.args, action, repeat_node.start_pos.clone(), repeat_node.end_pos.clone())?;
        Ok(repeat_node)
    }

    fn validate_args(&self, input_args: Vec<Arg>, action: &Action, start_pos: Position, end_pos: Position) -> Result<Vec<Arg>, ValidateError> {
        let mut node_args = input_args;
        let all_provided_args: Vec<Arg> = node_args.clone();
        let mut args: Vec<Arg> = vec![];
        let mut index: i32 = -1;

        let mut tags: Vec<Arg> = vec![];
        for arg in action.args.clone() {
            let mut match_more = true;
            let mut matched_one = false;
            while match_more {
                if !arg.allow_multiple {
                    match_more = false;
                }
                index += 1;
                if node_args.is_empty() {
                    if arg.optional {
                        if !matched_one {
                            args.push(Arg {
                                arg_type: ArgType::EMPTY,
                                value: ArgValue::Empty ,
                                index,
                                start_pos: Position::new(0, 0),
                                end_pos: Position::new(0, 0)
                            });
                        }
                        break;
                    } else if !matched_one {
                        return Err(ValidateError::MissingArgument { name: arg.name, start_pos, end_pos })
                    } else {
                        break;
                    }
                }
                let mut provided_arg = node_args.remove(0);

                if provided_arg.arg_type == ArgType::TAG {
                    tags.push(provided_arg);
                    match_more = true;
                    continue;
                }

                if provided_arg.arg_type == ArgType::EMPTY && !arg.optional {
                    return Err(ValidateError::MissingArgument { name: arg.name, start_pos, end_pos })
                }

                if let ArgValue::GameValue { value, selector, selector_end_pos } = provided_arg.value {
                    let actual_game_value = self.game_values.get(value.clone());
                    match actual_game_value {
                        Some(res) => provided_arg.value = ArgValue::GameValue {
                            value: res.df_name.clone(),
                            selector,
                            selector_end_pos
                        },
                        None => return Err(ValidateError::UnknownGameValue {
                            game_value: value,
                            start_pos: provided_arg.start_pos,
                            end_pos: provided_arg.end_pos
                        })
                    }
                }

                if !arg.arg_types.contains(&provided_arg.arg_type) && !arg.arg_types.contains(&ArgType::ANY) && provided_arg.arg_type != ArgType::VARIABLE && provided_arg.arg_type != ArgType::GameValue {
                    if arg.allow_multiple && matched_one {
                        node_args.insert(0, provided_arg);
                        index -= 1;
                        break;
                    }
                    return Err(ValidateError::WrongArgumentType { args: all_provided_args, index, name: arg.name, expected_types: arg.arg_types, found_type: provided_arg.arg_type })
                }

                provided_arg.index = index;
                args.push(provided_arg);
                matched_one = true;
            }
        }

        if !node_args.is_empty() {
            for val in node_args.clone() {
                if val.arg_type != ArgType::TAG {
                    return Err(ValidateError::TooManyArguments { name: action.dfrs_name.clone(), start_pos, end_pos })
                }
                tags.push(val)
            }
        }

        for given_tag in tags.clone() {
            match given_tag.value {
                ArgValue::Tag { tag: tag_name, value: _, definition: _, name_end_pos, value_start_pos: _ } => {
                    let mut found = false;
                    let mut available = vec![];
                    for tag in action.tags.clone() {
                        available.push(tag.dfrs_name.clone());
                        if tag.dfrs_name == tag_name {
                            found = true;
                        }
                    }
                    if !found {
                        return Err(ValidateError::UnknownTag { tag_name, available, start_pos: given_tag.start_pos, end_pos: name_end_pos });
                    }
                }
                _ => unreachable!()
            }
        }

        for tag in action.tags.clone() {
            let mut matched = false;
            for given_tag in tags.clone() {
                match given_tag.value {
                    ArgValue::Tag { tag: tag_name, value, name_end_pos, value_start_pos , ..} => {
                        if tag.dfrs_name == tag_name {
                            if tag.options.contains(&value) {
                                matched = true;
                                args.push(Arg {
                                    arg_type: ArgType::TAG,
                                    value: ArgValue::Tag { tag: tag.df_name.clone(), value, definition: Some(tag.clone()), name_end_pos, value_start_pos },
                                    index: tag.slot as i32,
                                    start_pos: given_tag.start_pos,
                                    end_pos: given_tag.end_pos
                                });
                            } else {
                                return Err(ValidateError::InvalidTagOption { tag_name, provided: value, options: tag.options, start_pos: value_start_pos, end_pos: given_tag.end_pos });
                            }
                        }
                    }
                    _ => unreachable!()
                }
            }
            if !matched {
                args.push(Arg {
                    arg_type: ArgType::TAG,
                    value: ArgValue::Tag { tag: tag.df_name.clone(), value: tag.default.clone(), definition: Some(tag.clone()), name_end_pos: Position::new(0, 0), value_start_pos: Position::new(0, 0) },
                    index: tag.slot as i32,
                    start_pos: Position::new(0, 0),
                    end_pos: Position::new(0, 0)
                });
            }
        }

        Ok(args)
    }
}

// TODO validate potions, sounds, particles etc