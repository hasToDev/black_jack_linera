#![cfg_attr(target_arch = "wasm32", no_main)]

mod count;
mod constants;
mod random;
mod state;

use linera_sdk::{
    base::WithContractAbi,
    views::{RootView, View, ViewStorageContext},
    Contract, ContractRuntime,
};
use black_jack_chain::{BlackJackParameters, BlackJackMessage, CardOperation, Status, PlayData, LastAction};
use self::state::BlackJack;
use crate::count::*;
use crate::random::*;
use crate::constants::*;

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
                        player_two.id = player_id.clone();
                        player_two.name = player_name.clone();
                        game_state.status = Status::Started;
                        self.start_game(player_id, player_name).await;
                    }
                    Status::Started => {
                        panic!("blackjack have started");
                    }
                    Status::Finish => {
                        // TODO: implement this, should do the same action with Status::Idle
                    }
                }
            }
            CardOperation::Start => {}
            CardOperation::Action { player_id, action } => {
                log::info!("CardOperation::Action");

                self.check_game_state();
                self.check_player(player_id.clone()).await;

                match action {
                    0 => {
                        // Stand
                        self.stand(player_id).await;
                    }
                    1 => {
                        // Hit
                        self.hit(player_id).await;
                    }
                    _ => {
                        panic!("action not recognized");
                    }
                }
            }
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

    fn check_game_state(&mut self) {
        let state = self.state.game_state.get();
        if state.status != Status::Started {
            panic!("game not started yet");
        }
    }

    async fn check_player(&mut self, player_id: String) {
        if self.state.play_data.contains_key(&player_id).await.unwrap_or(false) {
            let p = self.state.play_data.get(&player_id).await
                .unwrap_or_else(|_| { panic!("unable to get play data"); }).unwrap_or_else(|| { panic!("unable to get play data"); });
            if p.player_id_turn != player_id {
                panic!("not your turn");
            }
        } else {
            panic!("player not exist");
        }
    }


    async fn stand(&mut self, player_id: String) {
        let player_one = self.state.p1.get();
        let player_two = self.state.p2.get();

        // decide next turn id
        let mut next_turn = player_id.clone();
        if player_one.id == player_id {
            next_turn = player_two.id.clone();
        } else {
            next_turn = player_one.id.clone();
        }

        // load game data
        let mut p1_data = self.state.play_data.get(&player_one.id).await
            .unwrap_or_else(|_| { panic!("unable to get play data"); }).unwrap_or_else(|| { panic!("unable to get play data"); });
        let mut p2_data = self.state.play_data.get(&player_two.id).await
            .unwrap_or_else(|_| { panic!("unable to get play data"); }).unwrap_or_else(|| { panic!("unable to get play data"); });

        // check last action
        // if last action is stand, then the game must end because both player action choose to stand
        // the winner is player with the biggest score
        if p1_data.last_action == LastAction::Stand {
            // TODO: process with winner, find out who's winning, then end the game
            // TODO: don't forget to reset all contract state
        } else {
            // update data
            p1_data.player_id_turn = next_turn.clone();
            p1_data.last_action = LastAction::Stand;
            p2_data.player_id_turn = next_turn;
            p2_data.last_action = LastAction::Stand;

            // save data to state
            self.state.play_data.insert(&player_one.id, p1_data).unwrap_or_else(|_| {
                panic!("Failed to update Play Data for {:?} - {:?}", player_one.name, player_one.id);
            });
            self.state.play_data.insert(&player_two.id, p2_data).unwrap_or_else(|_| {
                panic!("Failed to update Play Data for {:?} - {:?}", player_two.name, player_two.id);
            });
        }
    }

    async fn hit(&mut self, player_id: String) {
        let player_one = self.state.p1.get();
        let player_two = self.state.p2.get();

        // decide next turn id
        let mut next_turn = player_id.clone();
        if player_one.id == player_id {
            next_turn = player_two.id.clone();
        } else {
            next_turn = player_one.id.clone();
        }

        // load game data
        let current_decks = self.state.decks.get_mut();
        let mut p1_data = self.state.play_data.get(&player_one.id).await
            .unwrap_or_else(|_| { panic!("unable to get play data"); }).unwrap_or_else(|| { panic!("unable to get play data"); });
        let mut p2_data = self.state.play_data.get(&player_two.id).await
            .unwrap_or_else(|_| { panic!("unable to get play data"); }).unwrap_or_else(|| { panic!("unable to get play data"); });


        // player turn
        if player_one.id == player_id {
            // P1
            let ts = self.runtime.system_time().to_string();
            let index = random_index(ts, current_decks.len() as u8, player_one.id.clone(), "P1".to_string());
            let chosen_card = current_decks.get(index as usize).unwrap_or_else(|| {
                panic!("unable to draw card");
            });
            p1_data.my_card.push(*chosen_card);
            p2_data.opponent_card.push(*chosen_card);
            p1_data.my_score = p1_data.my_score.saturating_add(calculate_player_score(*chosen_card, &p1_data.my_card, p1_data.my_score));
            p2_data.opponent_score = p2_data.opponent_score.saturating_add(calculate_player_score(*chosen_card, &p2_data.opponent_card, p2_data.opponent_score));
            current_decks.swap_remove(index as usize);
        } else {
            // P2
            let ts = self.runtime.system_time().to_string();
            let index = random_index(ts, current_decks.len() as u8, player_two.id.clone(), "P2".to_string());
            let chosen_card = current_decks.get(index as usize).unwrap_or_else(|| {
                panic!("unable to draw card");
            });
            p2_data.my_card.push(*chosen_card);
            p1_data.opponent_card.push(*chosen_card);
            p2_data.my_score = p2_data.my_score.saturating_add(calculate_player_score(*chosen_card, &p2_data.my_card, p2_data.my_score));
            p1_data.opponent_score = p1_data.opponent_score.saturating_add(calculate_player_score(*chosen_card, &p1_data.opponent_card, p1_data.opponent_score));
            current_decks.swap_remove(index as usize);
        }

        // update data
        p1_data.player_id_turn = next_turn.clone();
        p1_data.last_action = LastAction::Hit;
        p2_data.player_id_turn = next_turn;
        p2_data.last_action = LastAction::Hit;

        // save data to state
        self.state.play_data.insert(&player_one.id, p1_data).unwrap_or_else(|_| {
            panic!("Failed to update Play Data for {:?} - {:?}", player_one.name, player_one.id);
        });
        self.state.play_data.insert(&player_two.id, p2_data).unwrap_or_else(|_| {
            panic!("Failed to update Play Data for {:?} - {:?}", player_two.name, player_two.id);
        });

        // TODO: compare both player score, if any of the player score is 21 or more, then the game must end
        // TODO: find out who's winning, then end the game
        // TODO: don't forget to reset all contract state
    }

    async fn start_game(&mut self, p2_id: String, p2_name: String) {
        let mut new_decks = Vec::from(CARD_DECKS);

        let mut p1_card: Vec<u8> = Vec::new();
        let mut p2_card: Vec<u8> = Vec::new();
        let mut p1_card_for_opponent: Vec<u8> = Vec::new();
        let mut p2_card_for_opponent: Vec<u8> = Vec::new();

        let mut p1_score: u8 = 0;
        let mut p2_score: u8 = 0;
        let mut p1_score_for_opponent: u8 = 0;
        let mut p2_score_for_opponent: u8 = 0;

        let player_one = self.state.p1.get();

        // P1 First Card
        let mut ts = self.runtime.system_time().to_string();
        let mut index = random_index(ts, new_decks.len() as u8, player_one.id.clone(), "f".to_string());
        let mut chosen_card = new_decks.get(index as usize).unwrap_or_else(|| {
            panic!("unable to draw card");
        });
        p1_card.push(*chosen_card);
        p1_score = p1_score.saturating_add(calculate_player_score(*chosen_card, &p1_card, p1_score));
        p1_card_for_opponent.push(0);
        new_decks.swap_remove(index as usize);

        // P2 First Card
        ts = self.runtime.system_time().to_string();
        index = random_index(ts, new_decks.len() as u8, p2_id.clone(), "f".to_string());
        chosen_card = new_decks.get(index as usize).unwrap_or_else(|| {
            panic!("unable to draw card");
        });
        p2_card.push(*chosen_card);
        p2_score = p2_score.saturating_add(calculate_player_score(*chosen_card, &p2_card, p2_score));
        p2_card_for_opponent.push(0);
        new_decks.swap_remove(index as usize);

        // P1 Second Card
        ts = self.runtime.system_time().to_string();
        index = random_index(ts, new_decks.len() as u8, player_one.id.clone(), "s".to_string());
        chosen_card = new_decks.get(index as usize).unwrap_or_else(|| {
            panic!("unable to draw card");
        });
        p1_card.push(*chosen_card);
        p1_card_for_opponent.push(*chosen_card);
        p1_score = p1_score.saturating_add(calculate_player_score(*chosen_card, &p1_card, p1_score));
        p1_score_for_opponent = p1_score_for_opponent.saturating_add(calculate_player_score(*chosen_card, &p1_card_for_opponent, p1_score_for_opponent));
        new_decks.swap_remove(index as usize);

        // P2 Second Card
        ts = self.runtime.system_time().to_string();
        index = random_index(ts, new_decks.len() as u8, p2_id.clone(), "s".to_string());
        chosen_card = new_decks.get(index as usize).unwrap_or_else(|| {
            panic!("unable to draw card");
        });
        p2_card.push(*chosen_card);
        p2_card_for_opponent.push(*chosen_card);
        p2_score = p2_score.saturating_add(calculate_player_score(*chosen_card, &p2_card, p2_score));
        p2_score_for_opponent = p2_score_for_opponent.saturating_add(calculate_player_score(*chosen_card, &p2_card_for_opponent, p2_score_for_opponent));
        new_decks.swap_remove(index as usize);

        // save play data
        self.state.play_data.insert(&player_one.id, PlayData {
            my_card: p1_card,
            opponent_card: p2_card_for_opponent,
            my_score: p1_score,
            opponent_score: p2_score_for_opponent,
            player_id_turn: player_one.id.clone(),
            last_action: LastAction::None,
        },
        ).unwrap_or_else(|_| {
            panic!("Failed to update Play Data for {:?} - {:?}", player_one.name, player_one.id);
        });
        self.state.play_data.insert(&p2_id, PlayData {
            my_card: p2_card,
            opponent_card: p1_card_for_opponent,
            my_score: p2_score,
            opponent_score: p1_score_for_opponent,
            player_id_turn: player_one.id.clone(),
            last_action: LastAction::None,
        },
        ).unwrap_or_else(|_| {
            panic!("Failed to update Play Data for {:?} - {:?}", p2_name, p2_id);
        });

        // save card deck
        self.state.decks.set(new_decks);
    }
}
