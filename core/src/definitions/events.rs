use crate::definitions::action_dump::ActionDump;

#[derive(Debug)]
pub struct Event {
    pub dfrs_name: String,
    pub df_name: String
}

#[derive(Debug)]
pub struct PlayerEvents {
    events: Vec<Event>
}

impl PlayerEvents {
    pub fn new(action_dump: &ActionDump) -> PlayerEvents {
        let mut events = vec![];
        for entry in &action_dump.actions {
            if entry.codeblock_name == "PLAYER EVENT" {
                let v = entry.name.clone().trim().to_owned();
                let mut vv: Vec<char> = v.chars().collect();
                vv[0] = vv[0].to_lowercase().next().unwrap();
                let name: String = vv.into_iter().collect();
                events.push(Event {
                    df_name: entry.name.clone(),
                    dfrs_name: name
                })
            }
        }
        PlayerEvents { events }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Event> {
        self.events.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Event> {
        &self.events
    }
}

#[derive(Debug)]
pub struct EntityEvents {
    events: Vec<Event>
}

impl EntityEvents {
    pub fn new(action_dump: &ActionDump) -> EntityEvents {
        let mut events = vec![];
        for entry in &action_dump.actions {
            if entry.codeblock_name == "ENTITY EVENT" {
                let v = entry.name.clone().trim().to_owned();
                let mut vv: Vec<char> = v.chars().collect();
                vv[0] = vv[0].to_lowercase().next().unwrap();
                let name: String = vv.into_iter().collect();
                events.push(Event {
                    df_name: entry.name.clone(),
                    dfrs_name: name
                })
            }
        }
        EntityEvents { events }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Event> {
        self.events.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Event> {
        &self.events
    }
}