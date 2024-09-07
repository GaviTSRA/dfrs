use super::action_dump::{get_actions, Action, ActionDump};

#[derive(Debug)]
pub struct PlayerActions {
    player_actions: Vec<Action>
}

impl PlayerActions {
    pub fn new(action_dump: &ActionDump) -> PlayerActions {
        let actions = get_actions(action_dump, "PLAYER ACTION");
        PlayerActions {player_actions: actions}
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.player_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.player_actions
    }
}

#[derive(Debug)]
pub struct EntityActions {
    entity_actions: Vec<Action>
}

impl EntityActions {
    pub fn new(action_dump: &ActionDump) -> EntityActions {
        let actions = get_actions(action_dump, "ENTITY ACTION");
        EntityActions { entity_actions: actions }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.entity_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.entity_actions
    }
}

#[derive(Debug)]
pub struct GameActions {
    game_actions: Vec<Action>
}

impl GameActions {
    pub fn new(action_dump: &ActionDump) -> GameActions {
        let actions = get_actions(action_dump, "GAME ACTION");
        GameActions { game_actions: actions }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.game_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.game_actions
    }
}

#[derive(Debug)]
pub struct VariableActions {
    variable_actions: Vec<Action>
}

impl VariableActions {
    pub fn new(action_dump: &ActionDump) -> VariableActions {
        let actions = get_actions(action_dump, "SET VARIABLE");
        VariableActions { variable_actions: actions }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.variable_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.variable_actions
    }
}
