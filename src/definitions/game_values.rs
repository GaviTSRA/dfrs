use crate::definitions::action_dump::RawActionDump;
use crate::definitions::ArgType;
use crate::utility::to_dfrs_name;

#[derive(Debug)]
pub struct GameValues {
  game_values: Vec<GameValue>,
}

#[derive(Debug)]
pub struct GameValue {
  pub df_name: String,
  pub dfrs_name: String,
  pub value_type: ArgType,
}

impl GameValues {
  pub fn new(action_dump: &RawActionDump) -> GameValues {
    let mut game_values = vec![];

    for game_value in action_dump.game_values.clone() {
      let value_type = match game_value.icon.return_type.unwrap().as_str() {
        "NUMBER" => ArgType::NUMBER,
        "COMPONENT" => ArgType::TEXT,
        "TEXT" => ArgType::STRING,
        "LOCATION" => ArgType::LOCATION,
        "VECTOR" => ArgType::VECTOR,
        "ITEM" => ArgType::ITEM,
        "LIST" => ArgType::VARIABLE,
        "SPAWN_EGG" => ArgType::ITEM,
        val => panic!("Unknown game value type: {val}"),
      };

      let new_value = GameValue {
        df_name: game_value.icon.name.clone(),
        dfrs_name: to_dfrs_name(&game_value.icon.name.clone()),
        value_type,
      };
      game_values.push(new_value);
    }

    GameValues { game_values }
  }

  pub fn get(&self, dfrs_name: String) -> Option<&GameValue> {
    self
      .game_values
      .iter()
      .find(|&action| action.dfrs_name == dfrs_name)
  }

  pub fn all(&self) -> &Vec<GameValue> {
    &self.game_values
  }
}
