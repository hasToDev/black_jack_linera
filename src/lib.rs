use std::str::FromStr;
use async_graphql::{Request, Response, scalar};
use async_graphql_derive::{SimpleObject};
use linera_sdk::base::{ChainId, ContractAbi, ServiceAbi};
use linera_sdk::graphql::GraphQLMutationRoot;
use serde::{Deserialize, Serialize};

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
}

/// ------------------------------------------------------------------------------------------
/// [BlackJackMessage]
/// ------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize, Serialize)]
pub enum BlackJackMessage {
    Leaderboard,
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
    Start,
    DrawCard,
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
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: String::from(""),
            name: String::from(""),
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
}

impl Default for crate::GameState {
    fn default() -> Self {
        Self {
            status: Status::Idle,
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
