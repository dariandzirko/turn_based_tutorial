use std::collections::HashMap;

type PlayerId = u64;

#[derive(Default)]
pub struct GameState {
    //Storing players in a map makes it easy to look them up by id.
    //For now just store the player name.
    pub players: HashMap<PlayerId, String>,
    //This is all the events that have been added to this state
    //(What we have been calling the "sequence of events")
    //Useful for undo/redo, but also for debugging!
    history: Vec<GameEvent>,
}

//For now we will just have a single event type: A PlayerJoined event.
#[derive(Clone)]
pub enum GameEvent {
    PlayerJoined { player_id: PlayerId, name: String },
}

impl GameState {
    //Aggregates an event into the GameState
    pub fn reduce(&mut self, valid_event: &GameEvent) {
        use GameEvent::*;

        match valid_event {
            PlayerJoined { player_id, name } => {
                self.players.insert(*player_id, name.to_string());
            }
        }

        self.history.push(valid_event.clone());
    }

    pub fn validate(&self, event: &GameEvent) -> bool {
        use GameEvent::*;

        match event {
            PlayerJoined { player_id, name: _ } => {
                if self.players.contains_key(player_id) {
                    return false;
                }
            }
        }

        //If we can't find something thats wrong
        //with event then it must be okay
        return true;
    }

    //Tries to consume an event by first validating it
    pub fn dispatch(&mut self, event: &GameEvent) -> Result<(), ()> {
        //It's very common to have a "dispatching" function
        //like this to do things like validation and logging
        if !self.validate(event) {
            return Err(());
        }

        self.reduce(event);
        Ok(())
    }
}

fn main() {
    let mut game_state = GameState::default();
    let event = GameEvent::PlayerJoined {
        player_id: 1234,
        name: "Darian".to_string(),
    };

    game_state.dispatch(&event).unwrap();
    game_state.dispatch(&event).unwrap();
}
