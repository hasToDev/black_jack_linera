#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
mod constants;

use std::sync::{Arc, Mutex};
use async_graphql::{EmptySubscription, Executor, Schema};
use async_graphql_derive::Object;
use self::state::BlackJack;
use linera_sdk::{
    base::WithServiceAbi,
    views::{View, ViewStorageContext},
    Service, ServiceRuntime,
};
use linera_sdk::base::Timestamp;
use linera_sdk::graphql::GraphQLMutationRoot;
use black_jack_chain::{CardOperation, History, Insight, LastAction, Leaderboard, PlayData, Player, Status};
use crate::constants::MILLENNIUM;

#[derive(Clone)]
pub struct BlackJackService {
    state: Arc<BlackJack>,
    runtime: Arc<Mutex<ServiceRuntime<Self>>>,
}

linera_sdk::service!(BlackJackService);

impl WithServiceAbi for BlackJackService {
    type Abi = black_jack_chain::BlackJackAbi;
}

impl Service for BlackJackService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = BlackJack::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        BlackJackService {
            state: Arc::new(state),
            runtime: Arc::new(Mutex::new(runtime)),
        }
    }

    async fn handle_query(&self, _query: Self::Query) -> Self::QueryResponse {
        let schema = Schema::build(
            self.clone(),
            CardOperation::mutation_root(),
            EmptySubscription,
        ).finish();
        schema.execute(_query).await
    }
}

/// ------------------------------------------------------------------------------------------
#[Object]
impl BlackJackService {
    async fn get_insight(&self) -> Insight {
        Insight {
            game_state: self.state.game_state.get().clone(),
            p_one: self.state.p1.get().clone(),
            p_two: self.state.p2.get().clone(),
        }
    }

    async fn get_play_data(&self, player_id: String) -> PlayData {
        if self.state.play_data.contains_key(&player_id).await.unwrap_or(false) {
            return self.state.play_data.get(&player_id).await
                .unwrap_or_else(|_| {
                    panic!("unable to get play data");
                }).unwrap_or_else(|| {
                panic!("unable to get play data");
            });
        }
        PlayData {
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

    async fn get_history(&self, limit: u32) -> Vec<History> {
        let history_count = self.state.history.count();
        if limit > history_count as u32 {
            return self.state.history.read_back(history_count).await.unwrap_or_else(|_| { panic!("unable to read history"); });
        }
        self.state.history.read_back(limit as usize).await.unwrap_or_else(|_| { panic!("unable to read history"); })
    }

    async fn get_leaderboard(&self) -> Leaderboard {
        let player_keys = self.state.leaderboard.indices().await.unwrap_or_else(|_| { panic!("unable to read leaderboard"); });
        let mut players = Vec::new();

        for key in player_keys.into_iter() {
            let p = self.state.leaderboard.get(&key).await.unwrap_or_else(|_| { panic!("unable to get player"); }).unwrap_or_else(|| { panic!("unable to get player"); });
            players.push(p);
        }

        // Compare win counts -> cmp(&a.win)
        // Compare lose counts -> then(a.lose.cmp(&b.lose))
        players.sort_by(|a, b| {
            b.win
                .cmp(&a.win)
                .then(a.lose.cmp(&b.lose))
        });

        Leaderboard { rank: players, count: self.state.game_count.get().clone() }
    }
}
