mod constants;
mod random;

use std::str::FromStr;
use async_graphql::{Request, Response, scalar};
use async_graphql_derive::{SimpleObject};
use linera_sdk::base::{ChainId, ContractAbi, ServiceAbi, Timestamp};
use linera_sdk::graphql::GraphQLMutationRoot;
use serde::{Deserialize, Serialize};
use crate::constants::{MILLENNIUM};

pub struct BlackJackAbi;

impl ContractAbi for BlackJackAbi {
    type Operation = CardOperation;
    type Response = ();
}

impl ServiceAbi for BlackJackAbi {
    type Query = Request;
    type QueryResponse = Response;
}

/// ------------------------------------------------------------------------------------------
/// [BlackJackParameters]
/// ------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct BlackJackParameters {
    pub leaderboard_chain_id: ChainId,
    pub leaderboard_pass: String,
    pub room_status_chain_id: ChainId,
    pub analytics_chain_id: ChainId,
}

/// ------------------------------------------------------------------------------------------
/// [BlackJackMessage]
/// ------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize, Serialize)]
pub enum BlackJackMessage {
    GameResult {
        p1: String,
        p1gid: String,
        p2: String,
        p2gid: String,
        winner: String,
        winner_gid: String,
        time: Timestamp,
    },
    RoomUpdate {
        id: ChainId,
        status: Insight,
    },
    Analytic {
        version: String,
    },
}

/// ------------------------------------------------------------------------------------------
/// [Operation]
/// ------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum CardOperation {
    Join {
        player_id: String,
        player_name: String,
        version: String,
        gid: String,
    },
    Action {
        player_id: String,
        action: u8,
    },
    IdleActionCheck {
        player_id: String,
    },
    StartLeaderBoard {
        p: String,
    },
    StopLeaderBoard {
        p: String,
    },
    ResetLeaderBoard {
        p: String,
    },
    ResetAnalytics {
        p: String,
    },
}

/// ------------------------------------------------------------------------------------------
/// [Player]
/// ------------------------------------------------------------------------------------------
#[derive(
    Debug,
    Clone,
    Default,
    Deserialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    SimpleObject
)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub gid: String,
    pub win: u32,
    pub lose: u32,
    pub play: u32,
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            id: String::from(""),
            name,
            gid: String::from(""),
            win: 0,
            lose: 0,
            play: 1,
        }
    }
}

/// ------------------------------------------------------------------------------------------
/// [GameState]
/// ------------------------------------------------------------------------------------------
#[derive(
    Debug,
    Clone,
    Deserialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    SimpleObject
)]
pub struct GameState {
    pub status: Status,
    pub last_update: Timestamp,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            status: Status::Idle,
            last_update: Timestamp::from(MILLENNIUM),
        }
    }
}

/// ------------------------------------------------------------------------------------------
/// [Status]
/// ------------------------------------------------------------------------------------------
scalar!(Status);
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
pub enum Status {
    Idle,
    Waiting,
    Started,
    Finish,
}

/// ------------------------------------------------------------------------------------------
/// [Insight]
/// ------------------------------------------------------------------------------------------
#[derive(
    Debug,
    Clone,
    Deserialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    SimpleObject
)]
pub struct Insight {
    pub id: ChainId,
    pub game_state: GameState,
    pub p_one: Player,
    pub p_two: Player,
}

impl Default for Insight {
    fn default() -> Self {
        Self {
            id: ChainId::from_str("e4854ab09513d0e0b62497a5e190a074ff161c6c39e4dfa07dc5e2c0ee73d284").unwrap(),
            game_state: GameState::default(),
            p_one: Player::default(),
            p_two: Player::default(),
        }
    }
}

/// ------------------------------------------------------------------------------------------
/// [PlayData]
/// ------------------------------------------------------------------------------------------
#[derive(
    Debug,
    Clone,
    Deserialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    SimpleObject
)]
pub struct PlayData {
    pub p_one_id: String,
    pub p_two_id: String,
    pub my_card: Vec<u8>,
    pub opponent_card: Vec<u8>,
    pub my_score: u8,
    pub opponent_score: u8,
    pub player_id_turn: String,
    pub last_action: LastAction,
    pub winner: String,
    pub game_state: Status,
    pub last_update: Timestamp,
}

impl Default for PlayData {
    fn default() -> Self {
        Self {
            p_one_id: "".to_string(),
            p_two_id: "".to_string(),
            my_card: vec![],
            opponent_card: vec![],
            my_score: 0,
            opponent_score: 0,
            player_id_turn: "".to_string(),
            last_action: LastAction::None,
            winner: "".to_string(),
            game_state: Status::Idle,
            last_update: Timestamp::from(MILLENNIUM),
        }
    }
}

/// ------------------------------------------------------------------------------------------
/// [LastAction]
/// ------------------------------------------------------------------------------------------
scalar!(LastAction);
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
pub enum LastAction {
    None,
    Stand,
    Hit,
}

/// ------------------------------------------------------------------------------------------
/// [History]
/// ------------------------------------------------------------------------------------------
#[derive(
    Debug,
    Clone,
    Deserialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    SimpleObject
)]
pub struct History {
    pub p1: String,
    pub p2: String,
    pub winner: String,
    pub time: Timestamp,
}

/// ------------------------------------------------------------------------------------------
/// [Leaderboard]
/// ------------------------------------------------------------------------------------------
#[derive(
    Debug,
    Clone,
    Default,
    Deserialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    SimpleObject
)]
pub struct Leaderboard {
    pub rank: Vec<Player>,
    pub count: u32,
}

impl Leaderboard {
    pub fn update_player(&mut self, player_name: &String, winner_name: &String) {
        let player_draw = String::from("");

        let is_draw = winner_name.eq(&player_draw);
        let is_player_win = !is_draw && winner_name.eq(player_name);
        let is_player_lose = !is_draw && winner_name.ne(player_name);

        if let Some(player) = self.rank.iter_mut().find(|p| p.name == *player_name) {
            player.play = player.play.saturating_add(1);

            if is_player_win {
                // Player Win
                player.win = player.win.saturating_add(1);
            } else if is_player_lose {
                // Player Lose
                player.lose = player.lose.saturating_add(1);
            }
        } else {
            let mut new_player = Player::new(player_name.clone());

            if is_player_win {
                // Player Win
                new_player.win = new_player.win.saturating_add(1);
            } else if is_player_lose {
                // Player Lose
                new_player.lose = new_player.lose.saturating_add(1);
            }

            self.rank.push(new_player);
        }
    }

    pub fn sort_rank(&mut self) {
        // Sort by wins (desc), then losses (asc)
        // Compare win counts -> cmp(&a.win)
        // Compare lose counts -> then(a.lose.cmp(&b.lose))
        self.rank.sort_by(|a, b| {
            b.win.cmp(&a.win).then(a.lose.cmp(&b.lose))
        });
    }

    pub fn update_count(&mut self) {
        self.count = self.count.saturating_add(1);
    }
}

/// ------------------------------------------------------------------------------------------
/// [VersionAnalytics]
/// ------------------------------------------------------------------------------------------
#[derive(
    Debug,
    Clone,
    Deserialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    SimpleObject
)]
pub struct VersionAnalytics {
    pub v: String,
    pub c: u32,
}

impl Default for VersionAnalytics {
    fn default() -> Self {
        Self { v: "".to_string(), c: 0 }
    }
}

/// ------------------------------------------------------------------------------------------
/// [GidLeaderboard]
/// ------------------------------------------------------------------------------------------
#[derive(
    Debug,
    Clone,
    Default,
    Deserialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    SimpleObject
)]
pub struct GidLeaderboard {
    pub gid: Vec<Player>,
    pub count: u32,
}

impl GidLeaderboard {
    pub fn update_player(&mut self, player_gid: &String, winner_name: &String) {
        let player_draw = String::from("");

        let is_draw = winner_name.eq(&player_draw);
        let is_player_win = !is_draw && winner_name.eq(player_gid);
        let is_player_lose = !is_draw && winner_name.ne(player_gid);

        if let Some(player) = self.gid.iter_mut().find(|p| p.name == *player_gid) {
            player.play = player.play.saturating_add(1);

            if is_player_win {
                // Player Win
                player.win = player.win.saturating_add(1);
            } else if is_player_lose {
                // Player Lose
                player.lose = player.lose.saturating_add(1);
            }
        } else {
            let mut new_player = Player::new(player_gid.clone());

            if is_player_win {
                // Player Win
                new_player.win = new_player.win.saturating_add(1);
            } else if is_player_lose {
                // Player Lose
                new_player.lose = new_player.lose.saturating_add(1);
            }

            self.gid.push(new_player);
        }
    }
}