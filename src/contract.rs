#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use linera_sdk::{
    base::WithContractAbi,
    views::{RootView, View, ViewStorageContext},
    Contract, ContractRuntime,
};

use self::state::BlackJack;

pub struct BlackJackContract {
    state: BlackJack,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(BlackJackContract);

impl WithContractAbi for BlackJackContract {
    type Abi = black_jack_chain::BlackJackAbi;
}

impl Contract for BlackJackContract {
    type Message = ();
    type Parameters = ();
    type InstantiationArgument = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = BlackJack::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        BlackJackContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {}

    async fn execute_operation(&mut self, _operation: Self::Operation) -> Self::Response {}

    async fn execute_message(&mut self, _message: Self::Message) {}

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}
