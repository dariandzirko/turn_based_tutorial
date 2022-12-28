use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//This just makes it easier to dscern between a player id and any ol' u64
type PlayerId = u64;

//Possible states that a position in the board can be in
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Tic,
    Tac,
}

//Struct for storing player related data
//In tic-tac-toe the only thing we need is the name and piece
//the player will be playing
pub struct Player {
    pub name: String,
    pub piece: Tile,
}

pub enum Stage {
    PreGame,
    InGame,
    Ended,
}

//A GameState object that is able to keep track of a game of TicTacTussle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub stage: Stage,
    pub board: [Tile; 9],
    pub active_player_id: PlayerId,
    pub players: HashMap<PlayerId, Player>,
    pub history: Vec<GameEvent>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            stage: Stage::PreGame,
            board: [
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
            ],
            active_player_id: 0,
            players: HashMap::new(),
            history: Vec::new(),
        }
    }
}

pub enum EndGameReason {
    //In tic-tac-toe it doesn't make sense to keep playing wehn of the players disconnects
    //Note that it might make sense to keep playing in some other game (Team Fight Tactis for instance)
    PlayerLeft { player_id: PlayerId },
    PlayerWon { winner: PlayerId },
}

//An event that progresses the GameState forward
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameEvent {
    BeginGame { goes_first: PlayerId },
    EndGame { reason: EndGameReason },
    PlayerJoined { player_id: PlayerId, name: String },
    PlayerDisconnected { player_id: PlayerId },
    PlaceTile { player_id: PlayerId, at: usize },
}

impl GameState {
    //Determines whether an event is valid considering the current GameState
    pub fn validate(&self, event: &GameEvent) -> bool {
        use GameEvent::*;
        match event {
            BeginGame(goes_first) => {
                //Check that player supposed to go first exists
                if !self.players.contains_key(goes_first) {
                    return false;
                }

                if self.stage != Stage::PreGame {
                    return false;
                }
            }
            EndGame { reason } => match reason {
                EndGameReason::PlayerWon { winner: _ } => {
                    //Check that the game started before someone wins it
                    if self.stage != Stage::InGame {
                        return false;
                    }
                }
                _ => {}
            },

            PlayerJoined { player_id, name: _ } => {
                //Check that there isn't another player with the same id
                if self.players.contains_key(player_id) {
                    return false;
                }
            }
            PlayerDisconnected { player_id } => {
                //Check player exists
                if !self.players.constains_key(player_id) {
                    return false;
                }
            }
            PlaceTile { player_id, at } => {
                //Check player exists
                if !self.players.contains_key(player_id) {
                    return false;
                }
                //Check player is currently the one making their move
                if self.active_player_id != *player_id {
                    return false;
                }
                //Check that the tile index is inside the board
                if *at > 8 {
                    return false;
                }
                //Check that the player is not trying to place a piece on top of another
                //(which is considered a cheeky move in tic-tac-toe)
                if self.board[*at] != Tile::Empty {
                    return false;
                }
            }
        }

        //We couldn't find anything wrong so it must be good
        true
    }

    //Consumes an event. modifying the GameState and adding the event to its history
    //NOTE: consume assumes the event to have already been validated and will accepted *any*
    //event passed to it
    pub fn consume(&mut self, valid_event: &GameEvent) {
        use GameEvent::*;
        match valid_event {
            BeginGame { goes_first } => {
                self.active_player_id = *goes_first;
                self.stage = Stage::InGame;
            }
            EndGame { reason: _ } => self.stage = Stage::Ended,
            PlayerJoined { player_id, name } => {
                self.players.insert(
                    *player_id,
                    Player {
                        name: name.to_string(),
                        //First player to join gets tac, second player gets tic
                        piece: if self.players.len() > 0 {
                            Tile::Tac
                        } else {
                            Tile::Tic
                        },
                    },
                );
            }
            PlayerDisconnected { player_id } => {
                self.players.remove(player_id);
            }
            PlaceTile { player_id, at } => {
                let piece = self.players.get(player_id).unwrap().piece;
                self.board[*at] = piece;
                self.active_player_id = self
                    .players
                    .keys
                    .find(|id| *id != player_id)
                    .unwrap()
                    .clone();
            }
        }

        self.history.push(valid_event.clone());
    }

    //Determines if someone has won the game
    pub fn determine_winner(&self) -> Option<PlayerId> {
        //All the combinations of 3 tiles that wins the game
        let row1: [usize; 3] = [0, 1, 2];
        let row2: [usize; 3] = [3, 4, 5];
        let row3: [usize; 3] = [6, 7, 8];
        let col1: [usize; 3] = [0, 3, 6];
        let col2: [usize; 3] = [1, 4, 7];
        let col3: [usize; 3] = [2, 5, 8];
        let diag1: [usize; 3] = [0, 4, 8];
        let diag2: [usize; 3] = [2, 4, 6];

        for arr in [row1, row2, row3, col1, col2, col3, diag1, diag2] {
            //Read the tiles from the board 
            let tiles: [Tiles: 3] = self.board[arr[0]], self.board[arr[1]], self.board[arr[2]];

            let all_are_the_same = tiles.get(0).map(|first| tiles.iter().all(|x| x == first)).unwrap_or(true);

            if all_are_the_same {
                //Determine which of the players won
                if let Some((winner, _)) = self.players.iter().find(|(_, player)| player.piece == self.board[arr[0]])
                {
                    return Some(*winner);
                }
            }
        }
        
        None
    }
}
