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
            PlayerAction::new("sendMessage", "SendMessage", 
                vec![
                    DefinedArg::text("Message to send", 0, true, true)
                ],
                vec![
                    DefinedTag::new("inherit_styles", vec!["True","False"]),
                    DefinedTag::new("merge_values", vec!["Add spaces","No spaces"]),
                    DefinedTag::new("alignment", vec!["Regular", "Centered"])
                ]
            ),

            PlayerAction::new("playSound", "PlaySound",
                vec![
                    DefinedArg::sound("Sound to play", 0, false, true),
                    DefinedArg::location("Playback location", 1, true, false)
                ],
                vec![] // TODO
            )
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
