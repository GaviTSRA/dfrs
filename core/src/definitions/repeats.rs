use super::action_dump::{get_actions, Action, ActionDump};

#[derive(Debug)]
pub struct Repeats {
    repeats: Vec<Action>
}

impl Repeats {
    pub fn new(action_dump: &ActionDump) -> Repeats {
        let actions = get_actions(action_dump, "REPEAT");
        Repeats {repeats: actions}
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.repeats.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.repeats
    }
}
