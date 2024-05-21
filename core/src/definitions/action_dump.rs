use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ActionDump {
    pub codeblocks: Vec<CodeBlock>,
    pub actions: Vec<Action>,
    #[serde(skip)]
    pub game_value_categories: String,
    #[serde(skip)]
    pub game_values: String,
    #[serde(skip)]
    pub particle_categories: String,
    #[serde(skip)]
    pub particles: String,
    #[serde(skip)]
    pub sound_categories: String,
    #[serde(skip)]
    pub sounds: String,
    #[serde(skip)]
    pub potions: String,
    #[serde(skip)]
    pub cosmetics: String,
    #[serde(skip)]
    pub shops: String
}

#[derive(Deserialize)]
pub struct CodeBlock {
    pub name: String,
    pub identifier: String,
    pub item: Icon
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Action {
    pub name: String,
    pub codeblock_name: String,
    pub tags: Vec<Tag>,
    pub aliases: Vec<String>,
    pub icon: Icon
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Tag {
    pub name: String,
    pub options: Vec<TagOption>,
    pub default_option: String,
    pub slot: i8
}

#[derive(Deserialize)]
pub struct TagOption {
    pub name: String,
    pub icon: Icon,
    pub aliases: Vec<String>
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Icon {
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
    #[serde(default="default_i32")]
    pub tags: i32,
    #[serde(default="default_vec_arg")]
    pub arguments: Vec<Argument>
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Argument {
    #[serde(rename="type", alias="text")]
    pub arg_type: String,
    #[serde(default="default_bool")]
    pub plural: bool,
    #[serde(default="default_bool")]
    pub optional: bool,
    #[serde(default="default_vec_string")]
    pub description: Vec<String>,
    #[serde(default="default_vec_vec_string")]
    pub notes: Vec<Vec<String>>
}

impl ActionDump {
    pub fn load() -> ActionDump {
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


fn default_vec_arg() -> Vec<Argument> {
    vec![]
}