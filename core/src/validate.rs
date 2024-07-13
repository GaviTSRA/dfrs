use phf::phf_map;

use crate::{definitions::{player_actions::PlayerActions, ArgType}, node::{ActionNode, ActionType, Arg, ArgValue, EventNode, Expression, FileNode}, token::Position};

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
    MissingArgument { node: ActionNode, index: i32, name: String },
    WrongArgumentType { node: ActionNode, index: i32, name: String, expected_type: ArgType, found_type: ArgType },
    TooManyArguments { node: ActionNode },
    InvalidTagOption { node:ActionNode, tag_name: String, provided: String, options: Vec<String>, start_pos: Position, end_pos: Position },
    UnknownTag { node: ActionNode, tag_name: String, available: Vec<String>, start_pos: Position, end_pos: Position }
}

pub struct Validator {
    player_actions: PlayerActions
}

impl Validator {
    pub fn new() -> Validator {
        Validator {player_actions: PlayerActions::new()}
    }
    pub fn validate(&self, mut node: FileNode) -> Result<FileNode, ValidateError> {
        for event in node.events.iter_mut() {
            let mut actual_event;
            
            actual_event = PLAYER_EVENTS.get(&event.event).cloned();
            match actual_event {
                Some(actual) => {
                    event.event = actual.to_owned();
                    event.event_type = Some(ActionType::Player);
                }
                None => {
                    actual_event = ENTITY_EVENTS.get(&event.event).cloned();
                    match actual_event {
                        Some(actual) => {
                            event.event = actual.to_owned();
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
                        expression.node = Expression::Action { node: self.validate_action(node)? };
                    }
                    Expression::Variable { .. } => {}
                }
            }
        }

        Ok(node)
    }

    fn validate_action(&self, mut action_node: ActionNode) -> Result<ActionNode, ValidateError> {
        match action_node.action_type {
            ActionType::Player => {
                let action = self.player_actions.get(action_node.clone().name);
                match action {
                    Some(action) => {
                        action_node.name = action.df_name.clone();
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
                                if all_provided_args.len() == 0 {
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
                                    } else {
                                        if !matched_one {
                                            return Err(ValidateError::MissingArgument { node: action_node, index, name: arg.name})
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                let mut provided_arg = all_provided_args.remove(0);
                                
                                if provided_arg.arg_type == ArgType::EMPTY && !arg.optional {
                                    return Err(ValidateError::MissingArgument { node: action_node, index, name: arg.name})
                                }

                                if provided_arg.arg_type != arg.arg_type && arg.arg_type != ArgType::ANY && provided_arg.arg_type != ArgType::VARIABLE {
                                    if arg.allow_multiple && matched_one {
                                        action_node.args.insert(0, provided_arg);
                                        break;
                                    }    
                                    action_node.args.push(provided_arg.clone());
                                    return Err(ValidateError::WrongArgumentType { node: action_node, index, name: arg.name, expected_type: arg.arg_type, found_type: provided_arg.arg_type })
                                }

                                provided_arg.index = index;
                                args.push(provided_arg);
                                matched_one = true;
                            }
                        }

                        let mut tags: Vec<Arg> = vec![];
                        if all_provided_args.len() > 0 {
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
                    }
                    None => return Err(ValidateError::UnknownAction { node: action_node })
                }
            }
            ActionType::Entity => {

            }
            ActionType::Game => {

            }
        }

        Ok(action_node)
    }
}

// TODO validate potions, sounds, particles etc