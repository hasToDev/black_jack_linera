#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
mod constants;

use std::str::FromStr;
use std::sync::{Arc, Mutex};
use async_graphql::{EmptySubscription, Executor, Schema};
use async_graphql_derive::Object;
use self::state::BlackJack;
use linera_sdk::{
    base::WithServiceAbi,
    views::{View},
    Service, ServiceRuntime,
};
use linera_sdk::base::ChainId;
use linera_sdk::graphql::GraphQLMutationRoot;
use black_jack_chain::{CardOperation, History, Insight, Leaderboard, PlayData};

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
            id: ChainId::from_str("e4854ab09513d0e0b62497a5e190a074ff161c6c39e4dfa07dc5e2c0ee73d284")
                .unwrap_or_else(|_| { panic!("unable to create insight chain id"); }),
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
        PlayData::default()
    }

    async fn get_play_data_for_spectators(&self) -> PlayData {
        let p_one = self.state.p1.get().clone();
        let p_two = self.state.p2.get().clone();

        let p1_play_data = self.state.play_data.get(&p_one.id).await
            .unwrap_or(Some(PlayData::default()))
            .unwrap_or(PlayData::default());

        let p2_play_data = self.state.play_data.get(&p_two.id).await
            .unwrap_or(Some(PlayData::default()))
            .unwrap_or(PlayData::default());

        PlayData {
            my_card: p2_play_data.opponent_card,
            opponent_card: p1_play_data.opponent_card,
            my_score: p2_play_data.opponent_score,
            opponent_score: p1_play_data.opponent_score,
            player_id_turn: p1_play_data.player_id_turn,
            last_action: p1_play_data.last_action,
            winner: p1_play_data.winner,
            game_state: p1_play_data.game_state,
            last_update: p1_play_data.last_update,
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

    async fn get_game_room_status(&self) -> Vec<Insight> {
        let game_room_keys = self.state.room_status.indices().await.unwrap_or_else(|_| { panic!("unable to read room status"); });
        let mut game_room = Vec::new();

        for key in game_room_keys.into_iter() {
            let p = self.state.room_status.get(&key).await.unwrap_or_else(|_| { panic!("unable to get insight"); }).unwrap_or_else(|| { panic!("unable to get insight"); });
            game_room.push(p);
        }

        game_room
    }
}
