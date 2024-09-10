#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::{Arc, Mutex};
use async_graphql::{EmptySubscription, Executor, Schema};
use async_graphql_derive::Object;
use self::state::BlackJack;
use linera_sdk::{
    base::WithServiceAbi,
    views::{View, ViewStorageContext},
    Service, ServiceRuntime,
};
use linera_sdk::graphql::GraphQLMutationRoot;
use black_jack_chain::{CardOperation, Insight, LastAction, PlayData};

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
        }
    }
}
