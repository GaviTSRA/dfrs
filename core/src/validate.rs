use phf::phf_map;

use crate::{definitions::{action_dump::{Action, ActionDump}, actions::{EntityActions, GameActions, PlayerActions, VariableActions}, conditionals::{EntityConditionals, GameConditionals, PlayerConditionals, VariableConditionals}, ArgType}, node::{ActionNode, ActionType, Arg, ArgValue, ConditionalNode, ConditionalType, EventNode, Expression, FileNode}, token::Position};
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
    UnknownAction { node: ActionNode },
    UnknownGameValue { start_pos: Position, end_pos: Position, game_value: String },
    MissingArgument { node: ActionNode, index: i32, name: String },
    WrongArgumentType { node: ActionNode, index: i32, name: String, expected_types: Vec<ArgType>, found_type: ArgType },
    TooManyArguments { node: ActionNode },
    InvalidTagOption { node:ActionNode, tag_name: String, provided: String, options: Vec<String>, start_pos: Position, end_pos: Position },
    UnknownTag { node: ActionNode, tag_name: String, available: Vec<String>, start_pos: Position, end_pos: Position }
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
            None => return Err(ValidateError::UnknownAction { node: action_node })
        };
        Ok(action_node)
    }

    fn validate_action(&self, mut action_node: ActionNode, action: &Action) -> Result<ActionNode, ValidateError> {
        action_node.name.clone_from(&action.df_name);
        let mut args: Vec<Arg> = vec![];
        let mut index: i32 = -1;

        let mut all_provided_args = action_node.args.clone();
        action_node.args = vec![];

        for arg in action.args.clone() {
            let mut match_more = true;
            let mut matched_one = false;
            while match_more {
                if !arg.allow_multiple {
                    match_more = false;
                }
                index += 1;
                if all_provided_args.is_empty() {
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
                            return Err(ValidateError::MissingArgument { node: action_node, index, name: arg.name})
                    } else {
                        break;
                    }
                }
                let mut provided_arg = all_provided_args.remove(0);
                
                if provided_arg.arg_type == ArgType::EMPTY && !arg.optional {
                    return Err(ValidateError::MissingArgument { node: action_node, index, name: arg.name})
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
                        action_node.args.insert(0, provided_arg);
                        break;
                    }    
                    action_node.args = all_provided_args;
                    return Err(ValidateError::WrongArgumentType { node: action_node, index, name: arg.name, expected_types: arg.arg_types, found_type: provided_arg.arg_type })
                }

                provided_arg.index = index;
                args.push(provided_arg);
                matched_one = true;
            }
        }

        let mut tags: Vec<Arg> = vec![];
        if !all_provided_args.is_empty() {
            for val in all_provided_args.clone() {
                if val.arg_type != ArgType::TAG {
                    return Err(ValidateError::TooManyArguments { node: action_node })
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
                        return Err(ValidateError::UnknownTag { node: action_node, tag_name, available, start_pos: given_tag.start_pos, end_pos: name_end_pos });
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
                                return Err(ValidateError::InvalidTagOption { node: action_node, tag_name, provided: value, options: tag.options, start_pos: value_start_pos, end_pos: given_tag.end_pos });
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

        action_node.args = args;
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
            // None => return Err(ValidateError::UnknownAction { node: conditional_node })
            None => panic!("Unknown action")
        };

        for expression in conditional_node.expressions.iter_mut() {
            match expression.node.clone() {
                Expression::Action { node } => {
                    expression.node = Expression::Action { node: self.validate_action_node(node)? };
                }
                Expression::Conditional { node } => {
                    expression.node = Expression::Conditional { node: self.validate_conditional_node(node)? }
                }
                Expression::Variable { .. } => {}
            }
        }

        Ok(conditional_node)
    }

    fn validate_conditional(&self, mut conditional_node: ConditionalNode, action: &Action) -> Result<ConditionalNode, ValidateError> {
        conditional_node.name.clone_from(&action.df_name);
        let mut args: Vec<Arg> = vec![];
        let mut index: i32 = -1;

        let mut all_provided_args: Vec<Arg> = conditional_node.args.clone();
        conditional_node.args = vec![];

        for arg in action.args.clone() {
            let mut match_more = true;
            let mut matched_one = false;
            while match_more {
                if !arg.allow_multiple {
                    match_more = false;
                }
                index += 1;
                if all_provided_args.is_empty() {
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
                        panic!("Missing arg");
                            // return Err(ValidateError::MissingArgument { node: conditional_node, index, name: arg.name})
                    } else {
                        break;
                    }
                }
                let mut provided_arg = all_provided_args.remove(0);
                
                if provided_arg.arg_type == ArgType::EMPTY && !arg.optional {
                    panic!("missing arg");
                    // return Err(ValidateError::MissingArgument { node: conditional_node, index, name: arg.name})
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
                        conditional_node.args.insert(0, provided_arg);
                        break;
                    }    
                    conditional_node.args = all_provided_args;
                    panic!("wrong arg type");
                    // return Err(ValidateError::WrongArgumentType { node: conditional_node, index, name: arg.name, expected_types: arg.arg_types, found_type: provided_arg.arg_type })
                }

                provided_arg.index = index;
                args.push(provided_arg);
                matched_one = true;
            }
        }

        let mut tags: Vec<Arg> = vec![];
        if !all_provided_args.is_empty() {
            for val in all_provided_args.clone() {
                if val.arg_type != ArgType::TAG {
                    panic!("too many args");
                    // return Err(ValidateError::TooManyArguments { node: conditional_node })
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
                        panic!("unkown tag")
                        // return Err(ValidateError::UnknownTag { node: conditional_node, tag_name, available, start_pos: given_tag.start_pos, end_pos: name_end_pos });
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
                                panic!("invalid tag option")
                                // return Err(ValidateError::InvalidTagOption { node: conditional_node, tag_name, provided: value, options: tag.options, start_pos: value_start_pos, end_pos: given_tag.end_pos });
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
        
        conditional_node.args = args;
        Ok(conditional_node)
    }
}

// TODO validate potions, sounds, particles etc