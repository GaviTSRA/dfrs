use super::action_dump::{get_actions, Action, ActionDump};

#[derive(Debug)]
pub struct PlayerConditionals {
    player_conditionals: Vec<Action>
}

impl PlayerConditionals {
    pub fn new(action_dump: &ActionDump) -> PlayerConditionals {
        let actions = get_actions(action_dump, "IF PLAYER");
        PlayerConditionals {player_conditionals: actions}
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.player_conditionals.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.player_conditionals
    }
}

#[derive(Debug)]
pub struct EntityConditionals {
    entity_conditionals: Vec<Action>
}

impl EntityConditionals {
    pub fn new(action_dump: &ActionDump) -> EntityConditionals {
        let actions = get_actions(action_dump, "IF ENTITY");
        EntityConditionals {entity_conditionals: actions}
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.entity_conditionals.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.entity_conditionals
    }
}

#[derive(Debug)]
pub struct GameConditionals {
    game_conditionals: Vec<Action>
}

impl GameConditionals {
    pub fn new(action_dump: &ActionDump) -> GameConditionals {
        let actions = get_actions(action_dump, "IF GAME");
        GameConditionals {game_conditionals: actions}
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.game_conditionals.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.game_conditionals
    }
}

#[derive(Debug)]
pub struct VariableConditionals {
    variable_conditionals: Vec<Action>
}

impl VariableConditionals {
    pub fn new(action_dump: &ActionDump) -> VariableConditionals {
        let actions = get_actions(action_dump, "IF VARIABLE");
        VariableConditionals {variable_conditionals: actions}
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.variable_conditionals.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.variable_conditionals
    }
}