use crate::definitions::action_dump::RawActionDump;
use crate::utility::to_dfrs_name;

#[derive(Debug)]
pub struct Event {
  pub dfrs_name: String,
  pub df_name: String,
}

#[derive(Debug)]
pub struct PlayerEvents {
  events: Vec<Event>,
}

impl PlayerEvents {
  pub fn new(action_dump: &RawActionDump) -> PlayerEvents {
    let mut events = vec![];
    for entry in &action_dump.actions {
      if entry.codeblock_name == "PLAYER EVENT" {
        let name = to_dfrs_name(&entry.name.clone());
        events.push(Event {
          df_name: entry.name.clone(),
          dfrs_name: name,
        })
      }
    }
    PlayerEvents { events }
  }

  pub fn get(&self, dfrs_name: String) -> Option<&Event> {
    self
      .events
      .iter()
      .find(|&action| action.dfrs_name == dfrs_name)
  }

  pub fn all(&self) -> &Vec<Event> {
    &self.events
  }
}

#[derive(Debug)]
pub struct EntityEvents {
  events: Vec<Event>,
}

impl EntityEvents {
  pub fn new(action_dump: &RawActionDump) -> EntityEvents {
    let mut events = vec![];
    for entry in &action_dump.actions {
      if entry.codeblock_name == "ENTITY EVENT" {
        let name: String = to_dfrs_name(&entry.name.clone());
        events.push(Event {
          df_name: entry.name.clone(),
          dfrs_name: name,
        })
      }
    }
    EntityEvents { events }
  }

  pub fn get(&self, dfrs_name: String) -> Option<&Event> {
    self
      .events
      .iter()
      .find(|&action| action.dfrs_name == dfrs_name)
  }

  pub fn all(&self) -> &Vec<Event> {
    &self.events
  }
}
