use crate::definitions::action_dump::ActionDump;
use crate::utility::to_camel_case;

pub struct GameValues {
    game_values: Vec<GameValue>
}

pub struct GameValue {
    pub df_name: String,
    pub dfrs_name: String
}

impl GameValues {
    pub fn new(action_dump: &ActionDump) -> GameValues {
        let mut game_values = vec![];

        for game_value in &action_dump.game_values {
            let new_value = GameValue {
                df_name: game_value.icon.name.clone(),
                dfrs_name: to_camel_case(&game_value.icon.name.clone())
            };
            game_values.push(new_value);
        }

        GameValues {game_values}
    }

    pub fn get(&self, dfrs_name: String) -> Option<&GameValue> {
        self.game_values.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<GameValue> {
        &self.game_values
    }
}