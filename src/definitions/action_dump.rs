use crate::definitions::actions::{get_actions, Action};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawActionDump {
  pub codeblocks: Vec<ADCodeBlock>,
  pub actions: Vec<ADAction>,
  #[serde(skip)]
  pub game_value_categories: String,
  pub game_values: Vec<ADGameValue>,
  #[serde(skip)]
  pub particle_categories: String,
  pub particles: Vec<ADParticle>,
  #[serde(skip)]
  pub sound_categories: String,
  pub sounds: Vec<ADSound>,
  pub potions: Vec<ADPotion>,
  #[serde(skip)]
  pub cosmetics: String,
  #[serde(skip)]
  pub shops: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ADParticle {
  particle: String,
  icon: ADIcon,
  category: Option<String>,
  fields: Vec<String>,
}

impl DFRSValue for ADParticle {
  fn dfrs_equals(&self, value: &str) -> bool {
    self.particle == value
  }
  fn df_equals(&self, value: &str) -> bool {
    self.particle == value
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ADSound {
  sound: String,
  icon: ADIcon,
}

impl DFRSValue for ADSound {
  fn dfrs_equals(&self, value: &str) -> bool {
    self.sound == value
  }
  fn df_equals(&self, value: &str) -> bool {
    self.sound == value
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ADPotion {
  potion: String,
  icon: ADIcon,
}

impl DFRSValue for ADPotion {
  fn dfrs_equals(&self, value: &str) -> bool {
    self.potion == value
  }
  fn df_equals(&self, value: &str) -> bool {
    self.potion == value
  }
}

#[derive(Deserialize)]
pub struct ADCodeBlock {
  pub name: String,
  pub identifier: String,
  pub item: ADIcon,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ADAction {
  pub name: String,
  pub codeblock_name: String,
  pub tags: Vec<ADTag>,
  pub aliases: Vec<String>,
  pub icon: ADIcon,
  pub sub_action_blocks: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ADTag {
  pub name: String,
  pub options: Vec<ADTagOption>,
  pub default_option: String,
  pub slot: i8,
}

#[derive(Deserialize)]
pub struct ADTagOption {
  pub name: String,
  pub icon: ADIcon,
  pub aliases: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ADIcon {
  pub material: String,
  pub name: String,
  pub deprecated_note: Vec<String>,
  pub description: Vec<String>,
  pub example: Vec<String>,
  pub works_with: Vec<String>,
  pub additional_info: Vec<Vec<String>>,
  pub required_rank: String,
  pub require_tokens: bool,
  pub require_rank_and_tokens: bool,
  pub advanced: bool,
  pub loaded_item: String,
  #[serde(default = "default_i32")]
  pub tags: i32,
  #[serde(default = "default_vec_arg")]
  pub arguments: Vec<ADArgument>,
  #[serde(default = "default_vec_return", rename = "returnValues")]
  pub return_values: Vec<ADReturnValue>,
  pub return_type: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ADArgument {
  #[serde(rename = "type", alias = "text")]
  pub arg_type: String,
  #[serde(default = "default_bool")]
  pub plural: bool,
  #[serde(default = "default_bool")]
  pub optional: bool,
  #[serde(default = "default_vec_string")]
  pub description: Vec<String>,
  #[serde(default = "default_vec_vec_string")]
  pub notes: Vec<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ADReturnValue {
  #[serde(rename = "type", alias = "text")]
  pub return_type: String,
  #[serde(default = "default_vec_string")]
  pub description: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ADGameValue {
  pub aliases: Vec<String>,
  pub category: String,
  pub icon: ADIcon,
}

impl RawActionDump {
  pub fn load() -> RawActionDump {
    let file = include_str!("action_dump.json");
    serde_json::from_str(file).expect("Failed to parse action dump")
  }
}

fn default_i32() -> i32 {
  0
}

fn default_bool() -> bool {
  false
}

fn default_vec_string() -> Vec<String> {
  vec![]
}

fn default_vec_vec_string() -> Vec<Vec<String>> {
  vec![]
}

fn default_vec_arg() -> Vec<ADArgument> {
  vec![]
}

fn default_vec_return() -> Vec<ADReturnValue> {
  vec![]
}

pub trait DFRSValue {
  fn dfrs_equals(&self, value: &str) -> bool;
  fn df_equals(&self, value: &str) -> bool;
}

#[derive(Debug)]
pub struct ValueList<T>
where
  T: DFRSValue,
{
  pub values: Vec<T>,
}

impl<T> ValueList<T>
where
  T: DFRSValue,
{
  pub fn new(values: Vec<T>) -> ValueList<T> {
    ValueList { values }
  }

  pub fn get(&self, dfrs_name: &str) -> Option<&T> {
    self
      .values
      .iter()
      .find(|&action| action.dfrs_equals(dfrs_name))
  }

  pub fn get_df(&self, df_name: &str) -> Option<&T> {
    self.values.iter().find(|&action| action.df_equals(df_name))
  }

  pub fn all(&self) -> &Vec<T> {
    &self.values
  }
}

#[derive(Debug)]
pub struct ActionDump {
  pub player_actions: ValueList<Action>,
  pub entity_actions: ValueList<Action>,
  pub game_actions: ValueList<Action>,
  pub variable_actions: ValueList<Action>,
  pub control_actions: ValueList<Action>,
  pub select_actions: ValueList<Action>,
  pub start_process_action: Action,

  pub player_conditionals: ValueList<Action>,
  pub entity_conditionals: ValueList<Action>,
  pub game_conditionals: ValueList<Action>,
  pub variable_conditionals: ValueList<Action>,

  pub repeats: ValueList<Action>,

  pub sounds: ValueList<ADSound>,
  pub potions: ValueList<ADPotion>,
  pub particles: ValueList<ADParticle>,
}

impl ActionDump {
  pub fn new(action_dump: &RawActionDump) -> ActionDump {
    let actions = get_actions(&action_dump, "START PROCESS");
    let action = actions.get(0).unwrap();
    let start_process_action = Action {
      args: action.args.clone(),
      df_name: action.df_name.clone(),
      dfrs_name: action.dfrs_name.clone(),
      aliases: action.aliases.clone(),
      tags: action.tags.clone(),
      return_type: None,
      description: "".into(),
    };

    ActionDump {
      player_actions: ValueList::new(get_actions(action_dump, "PLAYER ACTION")),
      entity_actions: ValueList::new(get_actions(action_dump, "ENTITY ACTION")),
      game_actions: ValueList::new(get_actions(action_dump, "GAME ACTION")),
      variable_actions: ValueList::new(get_actions(action_dump, "SET VARIABLE")),
      control_actions: ValueList::new(get_actions(action_dump, "CONTROL")),
      select_actions: ValueList::new(get_actions(action_dump, "SELECT OBJECT")),
      start_process_action,

      player_conditionals: ValueList::new(get_actions(action_dump, "IF PLAYER")),
      entity_conditionals: ValueList::new(get_actions(action_dump, "IF ENTITY")),
      game_conditionals: ValueList::new(get_actions(action_dump, "IF GAME")),
      variable_conditionals: ValueList::new(get_actions(action_dump, "IF VARIABLE")),

      repeats: ValueList::new(get_actions(action_dump, "REPEAT")),

      sounds: ValueList::new(action_dump.sounds.clone()),
      potions: ValueList::new(action_dump.potions.clone()),
      particles: ValueList::new(action_dump.particles.clone()),
    }
  }
}
