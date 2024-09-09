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
use black_jack_chain::{CardOperation, Insight};

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
}
