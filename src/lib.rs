mod constants;
mod random;

use std::str::FromStr;
use async_graphql::{Request, Response, scalar};
use async_graphql_derive::{SimpleObject};
use linera_sdk::base::{ChainId, ContractAbi, ServiceAbi, Timestamp};
use linera_sdk::graphql::GraphQLMutationRoot;
use serde::{Deserialize, Serialize};
use crate::constants::MILLENNIUM;

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
    /// Chain ID for leaderboard
    pub leaderboard_chain_id: ChainId,
    pub leaderboard_pass: String,
}

/// ------------------------------------------------------------------------------------------
/// [BlackJackMessage]
/// ------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize, Serialize)]
pub enum BlackJackMessage {
    GameResult {
        p1: String,
        p2: String,
        winner: String,
        time: Timestamp,
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
    },
    Action {
        player_id: String,
        action: u8,
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
}

/// ------------------------------------------------------------------------------------------
/// [Player]
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
pub struct Player {
    pub id: String,
    pub name: String,
    pub win: u32,
    pub lose: u32,
    pub play: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: String::from(""),
            name: String::from(""),
            win: 0,
            lose: 0,
            play: 0,
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
    pub game_state: GameState,
    pub p_one: Player,
    pub p_two: Player,
}

impl Default for Insight {
    fn default() -> Self {
        Self {
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

impl Default for Leaderboard {
    fn default() -> Self {
        Self {
            rank: Vec::new(),
            count: 0,
        }
    }
}