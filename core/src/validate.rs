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
    WrongArgumentType { node: ActionNode, index: i32, name: String, expected_type: ArgType, found_type: ArgType }
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

                        // TODO allow_multiple ; too many args
                        for arg in action.args.clone() {
                            index += 1;
                            let mut provided_arg = match action_node.args.get(index as usize) {
                                Some(provided) => provided.clone(),
                                None => {
                                    if arg.optional {
                                        args.push(Arg { arg_type: ArgType::EMPTY, value: ArgValue::Empty , index} );
                                        continue;
                                    } else {
                                        return Err(ValidateError::MissingArgument { node: action_node, index, name: arg.name})
                                    }
                                }
                            };
                            
                            if provided_arg.arg_type == ArgType::EMPTY && !arg.optional {
                                return Err(ValidateError::MissingArgument { node: action_node, index, name: arg.name})
                            }

                            if provided_arg.arg_type != arg.arg_type {
                                return Err(ValidateError::WrongArgumentType { node: action_node, index: index, name: arg.name, expected_type: arg.arg_type, found_type: provided_arg.arg_type })
                            }

                            provided_arg.index = index;
                            args.push(provided_arg);
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

// TODO validate params
// TODO validate potions, sounds