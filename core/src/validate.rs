use phf::phf_map;

use crate::{definitions::{player_actions::PlayerActions, ArgType}, node::{ActionNode, ActionType, Arg, ArgValue, EventNode, Expression, FileNode}};

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

pub static PLAYER_ACTIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "giveItems" => "GiveItems(items:item*,amount:number?)",
    "setHotbar" => "SetHotbar(items:item*)",
    "setInventory" => "SetInventory(items:item*)",
    "setSlot" => "SetSlotItem(item:item?,slot:number)",
    "setEquipment" => "SetEquipment(item:item?,TAG)",
    "setArmor" => "SetArmor(items:item*)",
    "replaceItem" => "ReplaceItems(replace:items*?,with:item,amount:number?)",
    "removeItems" => "RemoveItems(items:item*)",
    "clearItems" => "ClearItems(items:item*)",
    "clearInventory" => "ClearInv(TAG,TAG)",
    "setCursor" => "SetCursorItem(item:item?)",
    "saveInventory" => "SaveInv",
    "loadInventory" => "LoadInv",
    "setItemCcoooldown" => "SetItemCooldown(item:item,time:number)",

    "sendMessage" => "SendMessage(msg:text*,TAG,TAG)",
    "playSound" => "PlaySound(sound:Sound)",
    "givePotion" => "GivePotion(sound:Sound)",
    "sendMessageSeq" => "SendMessageSeq(msg:text*,delay:num?,TAG)"
};

pub enum ValidateError {
    UnknownEvent { node: EventNode },
    UnknownAction { node: ActionNode },
    MissingArgument { node: ActionNode, index: i32, name: String },
    WrongArgumentType { node: ActionNode, index: i32, name: String, expected_type: ArgType, found_type: ArgType },
    TooManyArguments { node: ActionNode },
    InvalidTagOption { node:ActionNode, tag_name: String, provided: String, options: Vec<String> },
    UnknownTag { node: ActionNode, tag_name: String, available: Vec<String> }
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
                    crate::node::Expression::Action { node } => {
                        expression.node = Expression::Action { node: self.validate_action(node)? };
                    }
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

                        for arg in action.args.clone() {
                            let mut match_more = true;
                            let mut matched_one = false;
                            while match_more {
                                if !arg.allow_multiple {
                                    match_more = false;
                                }
                                index += 1;
                                if action_node.args.len() == 0 {
                                    if arg.optional {
                                        if !matched_one {
                                            args.push(Arg { arg_type: ArgType::EMPTY, value: ArgValue::Empty , index} );
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
                                let mut provided_arg = action_node.args.remove(0);
                                
                                if provided_arg.arg_type == ArgType::EMPTY && !arg.optional {
                                    return Err(ValidateError::MissingArgument { node: action_node, index, name: arg.name})
                                }

                                if provided_arg.arg_type != arg.arg_type {
                                    if arg.allow_multiple && matched_one {
                                        action_node.args.insert(0, provided_arg);
                                        break;
                                    }
                                    return Err(ValidateError::WrongArgumentType { node: action_node, index: index, name: arg.name, expected_type: arg.arg_type, found_type: provided_arg.arg_type })
                                }

                                provided_arg.index = index;
                                args.push(provided_arg);
                                matched_one = true;
                            }
                        }
                        let mut tags = vec![];
                        if action_node.args.len() > 0 {
                            for val in action_node.args.clone() {
                                if val.arg_type != ArgType::TAG {
                                    return Err(ValidateError::TooManyArguments { node: action_node })
                                }
                                tags.push(val.value)
                            }
                        }

                        for given_tag in tags.clone() {
                            match given_tag {
                                ArgValue::Tag { tag: tag_name, value: _, definition: _ } => {
                                    let mut found = false;
                                    let mut available = vec![];
                                    for tag in action.tags.clone() {
                                        available.push(tag.dfrs_name.clone());
                                        if tag.dfrs_name == tag_name {
                                            found = true;
                                        }
                                    }
                                    if !found {
                                        return Err(ValidateError::UnknownTag { node: action_node, tag_name, available });
                                    }
                                }
                                _ => unreachable!()
                            }
                        }

                        for tag in action.tags.clone() {
                            let mut matched = false;
                            for given_tag in tags.clone() {
                                match given_tag {
                                    ArgValue::Tag { tag: tag_name, value, .. } => {
                                        if tag.dfrs_name == tag_name {
                                            if tag.options.contains(&value) {
                                                matched = true;
                                                args.push(Arg {
                                                    arg_type: ArgType::TAG,
                                                    value: ArgValue::Tag { tag: tag.df_name.clone(), value, definition: Some(tag.clone()) },
                                                    index: tag.slot as i32
                                                });
                                            } else {
                                                return Err(ValidateError::InvalidTagOption { node: action_node, tag_name, provided: value, options: tag.options });
                                            }
                                        }
                                    }
                                    _ => unreachable!()
                                }
                            }
                            if !matched {
                                args.push(Arg {
                                    arg_type: ArgType::TAG,
                                    value: ArgValue::Tag { tag: tag.df_name.clone(), value: tag.options.get(0).unwrap().to_owned(), definition: Some(tag.clone()) },
                                    index: tag.slot as i32
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

// TODO validate potions, sounds