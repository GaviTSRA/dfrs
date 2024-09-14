use phf::phf_map;

use crate::{definitions::{action_dump::{Action, ActionDump}, actions::{EntityActions, GameActions, PlayerActions, VariableActions}, conditionals::{EntityConditionals, GameConditionals, PlayerConditionals, VariableConditionals}, ArgType, DefinedArg}, node::{ActionNode, ActionType, Arg, ArgValue, CallNode, ConditionalNode, ConditionalType, EventNode, Expression, FileNode}, token::Position};
use crate::definitions::game_values::GameValues;

pub static PLAYER_EVENTS: phf::Map<&'static str, &'static str> = phf_map! {
    "join" => "Join",
    "leave" => "Leave",
    "command" => "Command",
    "packLoad" => "PackLoad",
    "packDecline" => "PackDecline",

    "rightClick" => "RightClick",
    "leftClick" => "LeftClick",
    "clickEntity" => "ClickEntity",
    "clickPlayer" => "ClickPlayer",
    "loadCrossbow" => "LoadCrossbow",
    "placeBlock" => "PlaceBlock",
    "breakBlock" => "BreakBlock",
    "swapHands" => "SwapHands",
    "changeSlot" => "ChangeSlot",
    "tameMob" => "TameEntity",

    "walk" => "Walk",
    "jump" => "Jump",
    "sneak" => "Sneak",
    "unsneak" => "Unsneak",
    "startSprint" => "StartSprint",
    "stopSprint" => "StopSprint",
    "startFlight" => "StartFly",
    "stopFlight" => "StopFly",
    "riptide" => "Riptide",
    "dismount" => "Dismount",
    "horseJump" => "HorseJump",
    "vehicleJump" => "VehicleJump",

    "clickMenuSlot" => "ClickMenuSlot",
    "clickInventorySlot" => "ClickInvSlot",
    "pickUpItem" => "PickUpItem",
    "dropItem" => "DropItem",
    "consumeItem" => "Consume",
    "breakItem" => "BreakItem",
    "closeInventory" => "CloseInv",
    "fish" => "Fish",

    "playerTakeDamage" => "PlayerTakeDmg",
    "playerDamagePlayer" => "PlayerDmgPlayer",
    "playerDamageEntity" => "DamageEntity",
    "entityDamagePlayer" => "EntityDmgPlayer",
    "heal" => "PlayerHeal",
    "shootBow" => "ShootBow",
    "shootProjectile" => "ShootProjectile",
    "projectileHit" => "ProjHit",
    "projectileDamagePlayer" => "ProjDmgPlayer",
    "cloudImbuePlayer" => "CloudImbuePlayer",

    "playerDeath" => "Death",
    "killPlayer" => "KillPlayer",
    "playerResurrect" => "PlayerResurrect",
    "killMob" => "KillMob",
    "respawn" => "Respawn"
};

pub static ENTITY_EVENTS: phf::Map<&'static str, &'static str> = phf_map! {
    "entityDamageEntity" => "EntityDmgEntity",
    "entityKillEntity" => "EntityKillEntity",
    "entityTakeDamage" => "EntityDmg",
    "projectileDamageEntity" => "ProjDmgEntity",
    "projectileKillEntity" => "ProjKillEntity",
    "entityDeath" => "EntityDeath",
    "explode" => "EntityExplode",
    "vehicleTakeDamage" => "VehicleDamage",
    "blockFall" => "BlockFall",
    "blockLand" => "FallingBlockLand",
    "entityResurrect" => "EntityResurrect",
    "regrowWool" => "RegrowWool"
};

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
    player_actions: PlayerActions,
    entity_actions: EntityActions,
    game_actions: GameActions,
    variable_actions: VariableActions,
    player_conditionals: PlayerConditionals,
    entity_conditionals: EntityConditionals,
    game_conditionals: GameConditionals,
    variable_conditionals: VariableConditionals,
    game_values: GameValues
}

impl Validator {
    pub fn new() -> Validator {
        let action_dump = ActionDump::load();
        Validator {
            player_actions: PlayerActions::new(&action_dump),
            entity_actions: EntityActions::new(&action_dump),
            game_actions: GameActions::new(&action_dump),
            variable_actions: VariableActions::new(&action_dump),

            player_conditionals: PlayerConditionals::new(&action_dump),
            entity_conditionals: EntityConditionals::new(&action_dump),
            game_conditionals: GameConditionals::new(&action_dump),
            variable_conditionals: VariableConditionals::new(&action_dump),

            game_values: GameValues::new(&action_dump)
        }
    }
    pub fn validate(&self, mut node: FileNode) -> Result<FileNode, ValidateError> {
        for function in node.functions.iter_mut() {
            for expression in function.expressions.iter_mut() {
                match expression.node.clone() {
                    Expression::Action { node } => {
                        expression.node = Expression::Action { node: self.validate_action_node(node)? };
                    }
                    Expression::Conditional { node } => {
                        expression.node = Expression::Conditional { node: self.validate_conditional_node(node)? }
                    }
                    Expression::Call { node } => {
                        expression.node = Expression::Call { node: self.validate_call(node)? }
                    }
                    Expression::Variable { .. } => {}
                }
            }
        }

        for event in node.events.iter_mut() {
            let mut actual_event;
            
            actual_event = PLAYER_EVENTS.get(&event.event).cloned();
            match actual_event {
                Some(actual) => {
                    actual.clone_into(&mut event.event);
                    event.event_type = Some(ActionType::Player);
                }
                None => {
                    actual_event = ENTITY_EVENTS.get(&event.event).cloned();
                    match actual_event {
                        Some(actual) => {
                            actual.clone_into(&mut event.event);
                            event.event_type = Some(ActionType::Entity);
                        }
                        None => {
                            return Err(ValidateError::UnknownEvent { node: event.clone() })
                        }
                    }
                }
            }

            for expression in event.expressions.iter_mut() {
                match expression.node.clone() {
                    Expression::Action { node } => {
                        expression.node = Expression::Action { node: self.validate_action_node(node)? };
                    }
                    Expression::Conditional { node } => {
                        expression.node = Expression::Conditional { node: self.validate_conditional_node(node)? }
                    }
                    Expression::Call { node } => {
                        expression.node = Expression::Call { node: self.validate_call(node)? }
                    }
                    Expression::Variable { .. } => {}
                }
            }
        }

        Ok(node)
    }

    fn validate_action_node(&self, mut action_node: ActionNode) -> Result<ActionNode, ValidateError> {
        let action = match action_node.action_type {
            ActionType::Player => {
                self.player_actions.get(action_node.clone().name)
            }
            ActionType::Entity => {
                self.entity_actions.get(action_node.clone().name)
            }
            ActionType::Game => {
                self.game_actions.get(action_node.clone().name)
            }
            ActionType::Variable => {
                self.variable_actions.get(action_node.clone().name)
            }
        };

        match action {
            Some(res) => action_node = self.validate_action(action_node, res)?,
            None => return Err(ValidateError::UnknownAction { name: action_node.name, start_pos: action_node.start_pos, end_pos: action_node.end_pos })
        };
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
                self.player_conditionals.get(conditional_node.clone().name)
            }
            ConditionalType::Entity => {
                self.entity_conditionals.get(conditional_node.clone().name)
            }
            ConditionalType::Game => {
                self.game_conditionals.get(conditional_node.clone().name)
            }
            ConditionalType::Variable => {
                self.variable_conditionals.get(conditional_node.clone().name)
            }
        };

        match action {
            Some(res) => conditional_node = self.validate_conditional(conditional_node, res)?,
            None => return Err(ValidateError::UnknownAction { name: conditional_node.name, start_pos: conditional_node.start_pos, end_pos: conditional_node.end_pos })
        };

        for expression in conditional_node.expressions.iter_mut() {
            match expression.node.clone() {
                Expression::Action { node } => {
                    expression.node = Expression::Action { node: self.validate_action_node(node)? };
                }
                Expression::Conditional { node } => {
                    expression.node = Expression::Conditional { node: self.validate_conditional_node(node)? }
                }
                Expression::Call { node } => {
                    expression.node = Expression::Call { node: self.validate_call(node)? }
                }
                Expression::Variable { .. } => {}
            }
        }

        for expression in conditional_node.else_expressions.iter_mut() {
            match expression.node.clone() {
                Expression::Action { node } => {
                    expression.node = Expression::Action { node: self.validate_action_node(node)? };
                }
                Expression::Conditional { node } => {
                    expression.node = Expression::Conditional { node: self.validate_conditional_node(node)? }
                }
                Expression::Call { node } => {
                    expression.node = Expression::Call { node: self.validate_call(node)? }
                }
                Expression::Variable { .. } => {}
            }
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
            tags: vec![]
        };
        call_node.args = self.validate_args(call_node.args, &action, call_node.start_pos.clone(), call_node.end_pos.clone())?;
        Ok(call_node)
    }

    fn validate_args(&self, input_args: Vec<Arg>, action: &Action, start_pos: Position, end_pos: Position) -> Result<Vec<Arg>, ValidateError> {
        let mut node_args = input_args;
        let all_provided_args: Vec<Arg> = node_args.clone();
        let mut args: Vec<Arg> = vec![];
        let mut index: i32 = -1;

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

        let mut tags: Vec<Arg> = vec![];
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