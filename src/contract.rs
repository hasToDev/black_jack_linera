#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use linera_sdk::{
    base::WithContractAbi,
    views::{RootView, View, ViewStorageContext},
    Contract, ContractRuntime,
};
use black_jack_chain::{BlackJackParameters, BlackJackMessage, CardOperation, Status};
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
    type Message = BlackJackMessage;
    type Parameters = BlackJackParameters;
    type InstantiationArgument = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = BlackJack::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        BlackJackContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
        log::info!("App Initialization");

        // validate that the application parameters were configured correctly.
        let app_params = self.runtime.application_parameters();
        log::info!("Leaderboard Chain ID: {}", app_params.leaderboard_chain_id);

        if let Some(_owner) = self.runtime.authenticated_signer() {
            let chain_id = self.runtime.chain_id();

            // make sure runtime Chain ID is equal with Leaderboard Chain ID from parameters
            assert_eq!(
                chain_id,
                app_params.leaderboard_chain_id,
                "runtime ChainID doesn't match ChainID parameters"
            );
        }
    }

    async fn execute_operation(&mut self, _operation: Self::Operation) -> Self::Response {
        // root chain are not allowed to play
        self.check_root_invocation();

        match _operation {
            CardOperation::Join { player_id, player_name } => {
                log::info!("CardOperation::Join");

                let game_state = self.state.game_state.get_mut();

                match game_state.status {
                    Status::Idle => {
                        let player_one = self.state.p1.get_mut();
                        player_one.id = player_id;
                        player_one.name = player_name;
                        game_state.status = Status::Waiting;
                    }
                    Status::Waiting => {
                        let player_two = self.state.p2.get_mut();
                        player_two.id = player_id;
                        player_two.name = player_name;
                        game_state.status = Status::Started;

                        // TODO: create and shuffle a list of deck that can be chosen by player
                    }
                    Status::Started => {
                        panic!("blackjack have started");
                    }
                }
            }
            CardOperation::Start => {}
            CardOperation::DrawCard => {}
        }
    }

    async fn execute_message(&mut self, _message: Self::Message) {}

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl BlackJackContract {
    fn check_root_invocation(&mut self) {
        assert_ne!(
            self.runtime.chain_id(),
            self.runtime.application_parameters().leaderboard_chain_id,
            "Leaderboard chain are not allowed to play"
        )
    }
}
