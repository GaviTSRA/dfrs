use super::action_dump::{get_actions, Action, ActionDump};

pub struct PlayerActions {
    player_actions: Vec<Action>
}

impl PlayerActions {
    pub fn new(action_dump: &ActionDump) -> PlayerActions {
        let actions = get_actions(action_dump, "PLAYER ACTION");
        return PlayerActions {player_actions: actions};
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        for action in &self.player_actions {
            if action.dfrs_name == dfrs_name {
                return Some(action);
            }
        }
        return None;
    }
}


pub struct EntityActions {
    entity_actions: Vec<Action>
}

impl EntityActions {
    pub fn new(action_dump: &ActionDump) -> EntityActions {
        let actions = get_actions(action_dump, "ENTITY ACTION");
        return EntityActions { entity_actions: actions };
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        for action in &self.entity_actions {
            if action.dfrs_name == dfrs_name {
                return Some(action);
            }
        }
        return None;
    }
}

pub struct GameActions {
    game_actions: Vec<Action>
}

impl GameActions {
    pub fn new(action_dump: &ActionDump) -> GameActions {
        let actions = get_actions(action_dump, "GAME ACTION");
        return GameActions { game_actions: actions };
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        for action in &self.game_actions {
            if action.dfrs_name == dfrs_name {
                return Some(action);
            }
        }
        return None;
    }
}

pub struct VariableActions {
    variable_actions: Vec<Action>
}

impl VariableActions {
    pub fn new(action_dump: &ActionDump) -> VariableActions {
        let actions = get_actions(action_dump, "SET VARIABLE");
        return VariableActions { variable_actions: actions };
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        for action in &self.variable_actions {
            if action.dfrs_name == dfrs_name {
                return Some(action);
            }
        }
        return None;
    }
}
