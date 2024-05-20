use crate::definitions::{DefinedArg, DefinedTag};

pub struct PlayerAction {
    pub dfrs_name: String,
    pub df_name: String,
    pub args: Vec<DefinedArg>,
    pub tags: Vec<DefinedTag>
}

impl PlayerAction {
    pub fn new(dfrs_name: &str, df_name: &str, args: Vec<DefinedArg>, tags: Vec<DefinedTag>) -> PlayerAction {
        return PlayerAction {dfrs_name: dfrs_name.to_owned(), df_name: df_name.to_owned(), args, tags};
    }
}

pub struct PlayerActions {
    player_actions: Vec<PlayerAction>
}

impl PlayerActions {
    pub fn new() -> PlayerActions {
        let actions = vec![
            // Item Manipulation
            // TODO 9
            PlayerAction::new("clearInventory", "ClearInv",
                vec![],
                vec![
                    DefinedTag::new("clear_crafting_and_cursor", "Clear Crafting and Cursor", 25, vec!["True", "False"]),
                    DefinedTag::new("clear", "Clear Mode", 26, vec!["Entire inventory", "Main inventory", "Upper inventory", "Hotbar", "Armor"])
                ]
            ),
            // TODO 1
            PlayerAction::new("saveInventory", "SaveInv", vec![], vec![]),
            PlayerAction::new("loadInventory", "LoadInv", vec![], vec![]),
            // TODO 1

            // Communication
            PlayerAction::new("sendMessage", "SendMessage", 
                vec![
                    DefinedArg::text("Message to send", true, true) // FIXME is type actually any?
                ],
                vec![
                    DefinedTag::new("inherit_styles", "Inherit Styles", 24, vec!["True","False"]),
                    DefinedTag::new("merge_values", "Text Value Merging", 25, vec!["Add spaces","No spaces"]),
                    DefinedTag::new("alignment", "Alignment Mode", 26, vec!["Regular", "Centered"])
                ]
            ),
            PlayerAction::new("sendMessageSeq", "SendMessageSeq",
                vec![
                    DefinedArg::text("Messages to send", false, true),
                    DefinedArg::number("Message delay tickcs", true, false)
                ],
                vec![
                    DefinedTag::new("alignment", "Alignment Mode", 26, vec!["Regular", "Centered"])
                ]
            ),
            PlayerAction::new("sendTitle", "SendTitle",
                vec![
                    DefinedArg::text("Title text", false, false),
                    DefinedArg::text("Subtitle text", true, false),
                    DefinedArg::number("Title duration", true, false),
                    DefinedArg::number("Fade in length", true, false),
                    DefinedArg::number("Fade out length", true, false),
                ], 
                vec![]
            ),
            PlayerAction::new("sendActionBar", "ActionBar",
                vec![
                    DefinedArg::text("Message to send", false, true)
                ],
                vec![
                    DefinedTag::new("inherit_styles", "Inherit Styles", 25, vec!["True","False"]),
                    DefinedTag::new("merge_values", "Text Value Merging", 26, vec!["Add spaces","No spaces"]),
                ]
            ),
            // TODO 1
            PlayerAction::new("setBossBar", "SetBossBar", 
                vec![
                    DefinedArg::text("Title", true, false),
                    DefinedArg::number("Current health", true, false),
                    DefinedArg::number("Maximum health", true, false),
                    DefinedArg::number("Boss bar position", true, false)
                ],
                vec![
                    DefinedTag::new("sky", "Sky Effect", 24, vec!["None", "Create fog", "Darken sky", "Both"]),
                    DefinedTag::new("style", "Bar Style", 25, vec!["Solid", "6  segments", "10 segments", "12 segments", "20 segments"])
                ]
            ),
            PlayerAction::new("removeBossBar", "RemoveBossBar",
                vec![
                    DefinedArg::number("Boss bar position", true, false)
                ],
                vec![]
            ),
            // TODO 1
            PlayerAction::new("setTabInfo", "SetTabListInfo", 
                vec![
                    DefinedArg::text("Header/footer text", true, true)
                ],
                vec![
                    DefinedTag::new("inherit_styles", "Inherit Styles", 24, vec!["True","False"]),
                    DefinedTag::new("merge_values", "Text Value Merging", 25, vec!["Add spaces","No spaces"]),
                    DefinedTag::new("position", "Player List Field", 26, vec!["Header", "Footer"])
                ]
            ),
            PlayerAction::new("playSound", "PlaySound",
                vec![
                    DefinedArg::sound("Sound to play", false, true),
                    DefinedArg::location("Playback location", true, false)
                ],
                vec![
                    DefinedTag::new("source", "Sound Source", 26, vec!["Master", "Music", "Jukebox/Note Blocks", "Weather", "Blocks", "Hostile Creatures", "Friendly Creatures", "Players", "Ambient/Environment", "Voice/Speech"])
                ]
            ),
            PlayerAction::new("stopSound", "StopSound", 
                vec![
                    DefinedArg::sound("Sounds to stop", true, true)
                ],
                vec![
                    DefinedTag::new("source", "Sound Source", 26, vec!["Master", "Music", "Jukebox/Note Blocks", "Weather", "Blocks", "Hostile Creatures", "Friendly Creatures", "Players", "Ambient/Environment", "Voice/Speech"])
                ]
            ),
            PlayerAction::new("playSoundSeq", "PlaySoundSeq",
                vec![
                    DefinedArg::sound("Sound to play", false, true),
                    DefinedArg::number("Sound delay", true, false),
                    DefinedArg::location("Playback location", true, false)
                ],
                vec![
                    DefinedTag::new("source", "Sound Source", 26, vec!["Master", "Music", "Jukebox/Note Blocks", "Weather", "Blocks", "Hostile Creatures", "Friendly Creatures", "Players", "Ambient/Environment", "Voice/Speech"])
                ]
            )
            // TODO 1
        ];
        return PlayerActions {player_actions: actions};
    }
    pub fn get(&self, dfrs_name: String) -> Option<&PlayerAction> {
        for action in &self.player_actions {
            if action.dfrs_name == dfrs_name {
                return Some(action);
            }
        }
        return None;
    }
}
