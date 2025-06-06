#![cfg_attr(target_arch = "wasm32", no_main)]

mod count;
mod constants;
mod random;
mod state;

use linera_sdk::{
    base::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};
use black_jack_chain::{BlackJackParameters, BlackJackMessage, CardOperation, Status, PlayData, LastAction, History, Player, GameState, Insight, VersionAnalytics, PlayerStatus};
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

            // make sure runtime Chain ID is :
            // 1. equal with Leaderboard Chain ID
            // 2. different from the Room Status Chain ID
            // 3. different from the Analytics Chain ID
            // 4. different from the Player Status Chain ID
            assert_eq!(chain_id, app_params.leaderboard_chain_id, "runtime ChainID doesn't match ChainID parameters");
            assert_ne!(chain_id, app_params.room_status_chain_id, "runtime ChainID must be different than Room Status ChainID");
            assert_ne!(chain_id, app_params.analytics_chain_id, "runtime ChainID must be different than Analytics ChainID");
            assert_ne!(chain_id, app_params.player_status_chain_id, "runtime ChainID must be different than Player Status ChainID");
            assert_ne!(app_params.room_status_chain_id, app_params.analytics_chain_id, "Room Status ChainID must be different than Analytics ChainID");
            assert_ne!(app_params.analytics_chain_id, app_params.player_status_chain_id, "Analytics ChainID must be different than Player Status ChainID");
            assert_ne!(app_params.room_status_chain_id, app_params.player_status_chain_id, "Room Status ChainID must be different than Player Status ChainID");

            // set leaderboard to accept stats
            self.state.leaderboard_on.set(true);
        }
    }

    async fn execute_operation(&mut self, _operation: Self::Operation) -> Self::Response {
        match _operation {
            CardOperation::Join { player_id, player_name, version, gid } => {
                log::info!("CardOperation::Join");

                // root chain are not allowed to play
                self.check_root_invocation();

                let game_state = self.state.game_state.get_mut();
                let current_time = self.runtime.system_time();

                match game_state.status {
                    Status::Idle => {
                        let player_one = self.state.p1.get_mut();
                        player_one.id = player_id;
                        player_one.name = player_name.clone();
                        player_one.gid = gid.clone();
                        game_state.status = Status::Waiting;
                        game_state.last_update = current_time;
                    }
                    Status::Waiting => {
                        let time_elapsed = current_time.micros() - game_state.last_update.micros();

                        // reset if last game status update is more than 18 seconds
                        if time_elapsed >= UNIX_MICRO_IN_18_SECONDS {
                            // change status to Waiting for Player 2
                            game_state.status = Status::Waiting;
                            game_state.last_update = current_time;

                            // let new people join because previous game is inactive for more than 18 seconds
                            self.reset_and_register_new_player(player_id, player_name.clone(), gid.clone());

                            // send message for room status update, analytics, and player status
                            self.send_room_status_update().await;
                            self.send_app_version_analytics(version).await;
                            self.send_player_join_update(player_name, gid).await;

                            return;
                        }

                        let player_one = self.state.p1.get();
                        if player_one.name.to_lowercase() == player_name.to_lowercase() || player_one.id == player_id {
                            panic!("unable to start, both players have similar name or ID");
                        }

                        let player_two = self.state.p2.get_mut();
                        player_two.id = player_id.clone();
                        player_two.name = player_name.clone();
                        player_two.gid = gid.clone();
                        game_state.status = Status::Started;
                        game_state.last_update = current_time;
                        self.start_game(player_id, player_name.clone()).await;
                    }
                    Status::Started => {
                        let time_elapsed = current_time.micros() - game_state.last_update.micros();

                        // panic if last game status update is less than 18 seconds
                        if time_elapsed <= UNIX_MICRO_IN_18_SECONDS {
                            panic!("blackjack have started");
                        }

                        // change status to Waiting for Player 2
                        game_state.status = Status::Waiting;
                        game_state.last_update = current_time;

                        // let new people join because previous game is inactive for more than 18 seconds
                        self.reset_and_register_new_player(player_id, player_name.clone(), gid.clone());
                    }
                    Status::Finish => {
                        // change status to Waiting for Player 2
                        game_state.status = Status::Waiting;
                        game_state.last_update = current_time;

                        // start new game
                        self.reset_and_register_new_player(player_id, player_name.clone(), gid.clone());
                    }
                }

                // send message for room status update, analytics, and player status
                self.send_room_status_update().await;
                self.send_app_version_analytics(version).await;
                self.send_player_join_update(player_name, gid).await;
            }
            CardOperation::Action { player_id, action } => {
                log::info!("CardOperation::Action");

                // root chain are not allowed to play
                self.check_root_invocation();

                self.check_game_state();
                self.check_player(player_id.clone(), false).await;

                match action {
                    0 => {
                        // Stand
                        self.stand(player_id, false).await;
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
            CardOperation::IdleActionCheck { player_id } => {
                log::info!("CardOperation::IdleActionCheck");

                // root chain are not allowed to play
                self.check_root_invocation();

                self.check_game_state();
                self.check_player(player_id.clone(), true).await;

                // panic if last game status update is less than 10 seconds
                let time_elapsed = self.runtime.system_time().micros() - self.state.game_state.get().last_update.micros();
                if time_elapsed < UNIX_MICRO_IN_10_SECONDS {
                    panic!("too early for idle action check");
                }

                // Stand
                self.stand(player_id, true).await;
            }
            CardOperation::StartLeaderBoard { p } => {
                log::info!("CardOperation::StartLeaderBoard");

                // check Leaderboard authorization
                self.check_p(p);

                self.state.leaderboard_on.set(true);
            }
            CardOperation::StopLeaderBoard { p } => {
                log::info!("CardOperation::StopLeaderBoard");

                // check Leaderboard authorization
                self.check_p(p);

                self.state.leaderboard_on.set(false);
            }
            CardOperation::ResetLeaderBoard { p } => {
                log::info!("CardOperation::ResetLeaderBoard");

                // check Leaderboard authorization
                self.check_p(p);

                self.state.leaderboard.clear();
                self.state.history.clear();
            }
            CardOperation::ResetAnalytics { p } => {
                log::info!("CardOperation::ResetAnalytics");

                // check Analytics authorization
                self.check_p(p);

                self.state.analytics.clear();
            }
        }
    }

    async fn execute_message(&mut self, _message: Self::Message) {
        let is_bouncing = self
            .runtime
            .message_is_bouncing()
            .unwrap_or_else(|| {
                panic!("Message delivery status has to be available when executing a message");
            });

        match _message {
            BlackJackMessage::GameResult { p1, p1gid, p2, p2gid, winner, winner_gid, time } => {
                log::info!("BlackJackMessage::GameResult");
                // BlackJackMessage::GameResult not being tracked
                // Even if it does, bouncing message should do nothing.
                if is_bouncing {
                    return;
                }

                // prevent add stats to leaderboard if status is off
                if !self.state.leaderboard_on.get() {
                    panic!("Leaderboard is closed at the moment");
                }

                // load leaderboard
                let current_leaderboard = self.state.leaderboard.get_mut();

                // update leaderboard
                current_leaderboard.update_player(&p1, &winner);
                current_leaderboard.update_player(&p2, &winner);
                // current_leaderboard.sort_rank();
                current_leaderboard.update_count();

                // load gid leaderboard
                let current_gid_leaderboard = self.state.gid_leaderboard.get_mut();

                // update gid leaderboard
                current_gid_leaderboard.update_player(&p1gid, &winner_gid);
                current_gid_leaderboard.update_player(&p2gid, &winner_gid);

                // add game history
                self.state.history.push_back(History { p1: p1.clone(), p2: p2.clone(), winner, time });

                // update player status
                self.send_player_finish_update(p1, p2).await;
            }
            BlackJackMessage::RoomUpdate { id, status } => {
                log::info!("BlackJackMessage::RoomUpdate");
                // BlackJackMessage::RoomUpdate not being tracked
                // Even if it does, bouncing message should do nothing.
                if is_bouncing {
                    return;
                }

                // get game status
                let game_status = status.game_state.status;

                // remove status
                if game_status.eq(&Status::Idle) || game_status.eq(&Status::Finish) {
                    self.state.room_status.remove(&id).unwrap_or_else(|_| { panic!("Room status does not exist for {:?}", id); });
                    return;
                }

                // save or update status
                if game_status.eq(&Status::Waiting) || game_status.eq(&Status::Started) {
                    self.state.room_status.insert(&id, status).unwrap_or_else(|_| { panic!("Failed to update room status for {:?}", id); });
                }
            }
            BlackJackMessage::Analytic { version } => {
                log::info!("BlackJackMessage::Analytic");
                // BlackJackMessage::Analytic not being tracked
                // Even if it does, bouncing message should do nothing.
                if is_bouncing {
                    return;
                }

                // load analytics
                let mut analytics = self.state.analytics.get(&version).await
                    .unwrap_or(Some(VersionAnalytics::default()))
                    .unwrap_or(VersionAnalytics::default());

                // update analytics
                if analytics.v.eq(&String::from("")) {
                    analytics.v = version.clone();
                }
                analytics.c = analytics.c.saturating_add(1);

                // save analytics
                self.state.analytics.insert(&version, analytics).unwrap_or_else(|_| { panic!("Failed to update analytics for {:?}", version); });
            }
            BlackJackMessage::PlayerJoin { name, gid } => {
                log::info!("BlackJackMessage::PlayerJoin");
                // BlackJackMessage::PlayerJoin not being tracked
                // Even if it does, bouncing message should do nothing.
                if is_bouncing {
                    return;
                }

                // create and save player status
                let player_status = PlayerStatus { gid, time: self.runtime.system_time() };
                self.state.player_status.insert(&name, player_status).unwrap_or_else(|_| { panic!("Failed to insert {:?}", name); });
            }
            BlackJackMessage::PlayerFinish { p1, p2 } => {
                log::info!("BlackJackMessage::PlayerFinish");
                // BlackJackMessage::PlayerFinish not being tracked
                // Even if it does, bouncing message should do nothing.
                if is_bouncing {
                    return;
                }

                // remove Player 1
                if self.state.player_status.contains_key(&p1).await.unwrap_or(false) {
                    self.state.player_status.remove(&p1).unwrap_or_else(|_| { panic!("Failed to remove {:?}", p1); });
                }

                // remove Player 2
                if self.state.player_status.contains_key(&p2).await.unwrap_or(false) {
                    self.state.player_status.remove(&p2).unwrap_or_else(|_| { panic!("Failed to remove {:?}", p2); });
                }
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl BlackJackContract {
    fn check_root_invocation(&mut self) {
        assert_ne!(self.runtime.chain_id(), self.runtime.application_parameters().leaderboard_chain_id, "Leaderboard chain are not allowed to play");
        assert_ne!(self.runtime.chain_id(), self.runtime.application_parameters().room_status_chain_id, "Room status chain are not allowed to play");
        assert_ne!(self.runtime.chain_id(), self.runtime.application_parameters().analytics_chain_id, "Analytics chain are not allowed to play");
        assert_ne!(self.runtime.chain_id(), self.runtime.application_parameters().player_status_chain_id, "Player Status chain are not allowed to play")
    }

    fn check_p(&mut self, p: String) {
        assert_eq!(p, self.runtime.application_parameters().leaderboard_pass, "You are not authorized to execute Leaderboard and/or Analytics operation")
    }

    fn check_game_state(&mut self) {
        let state = self.state.game_state.get();
        if state.status != Status::Started {
            panic!("game not started yet");
        }
    }

    async fn check_player(&mut self, player_id: String, idle_action_check: bool) {
        if self.state.play_data.contains_key(&player_id).await.unwrap_or(false) {
            let p = self.state.play_data.get(&player_id).await
                .unwrap_or_else(|_| { panic!("unable to get play data"); }).unwrap_or_else(|| { panic!("unable to get play data"); });
            let is_invoker_the_current_player_turn = p.player_id_turn == player_id;
            if idle_action_check && !is_invoker_the_current_player_turn {
                // only opponent of current player that can invoke idle action check
                return;
            }
            if idle_action_check && is_invoker_the_current_player_turn {
                panic!("current player can't do idle action check");
            }
            if !is_invoker_the_current_player_turn {
                panic!("not your turn");
            }
        } else {
            panic!("player not exist");
        }
    }

    fn reset_and_register_new_player(&mut self, player_id: String, player_name: String, gid: String) {
        // create Player 1
        let mut player_one = Player::default();
        player_one.id = player_id;
        player_one.name = player_name;
        player_one.gid = gid;

        // update Player 1 data and reset previous game stats
        self.state.p1.set(player_one);
        self.state.p2.set(Player::default());
        self.state.decks.set(Vec::new());
        self.state.play_data.clear();
    }

    async fn send_game_finish_message(&mut self, p1: String, p1gid: String, p2: String, p2gid: String, winner: String, winner_gid: String) {
        // send message to leaderboard chain
        let message = BlackJackMessage::GameResult {
            p1,
            p1gid,
            p2,
            p2gid,
            winner,
            winner_gid,
            time: self.runtime.system_time(),
        };
        self.runtime
            .prepare_message(message)
            .send_to(self.runtime.application_parameters().leaderboard_chain_id);
    }

    async fn send_room_status_update(&mut self) {
        let new_status = Insight {
            id: self.runtime.chain_id(),
            game_state: self.state.game_state.get().clone(),
            p_one: self.state.p1.get().clone(),
            p_two: self.state.p2.get().clone(),
        };

        // send message to room status chain
        let message = BlackJackMessage::RoomUpdate { id: self.runtime.chain_id(), status: new_status };
        self.runtime
            .prepare_message(message)
            .send_to(self.runtime.application_parameters().room_status_chain_id);
    }

    async fn send_app_version_analytics(&mut self, version: String) {
        // send message to analytics chain
        let message = BlackJackMessage::Analytic { version };
        self.runtime
            .prepare_message(message)
            .send_to(self.runtime.application_parameters().analytics_chain_id);
    }

    async fn send_player_join_update(&mut self, name: String, gid: String) {
        // send message to analytics chain
        let message = BlackJackMessage::PlayerJoin { name, gid };
        self.runtime
            .prepare_message(message)
            .send_to(self.runtime.application_parameters().player_status_chain_id);
    }

    async fn send_player_finish_update(&mut self, p1: String, p2: String) {
        // send message to analytics chain
        let message = BlackJackMessage::PlayerFinish { p1, p2 };
        self.runtime
            .prepare_message(message)
            .send_to(self.runtime.application_parameters().player_status_chain_id);
    }

    async fn stand(&mut self, player_id: String, idle_action_check: bool) {
        let player_one = self.state.p1.get();
        let player_two = self.state.p2.get();

        // decide next turn id
        let mut next_turn = player_id.clone();
        if player_one.id == player_id {
            next_turn = player_two.id.clone();
        } else {
            next_turn = player_one.id.clone();
        }

        // set next turn to invoker player id on idle action check
        if idle_action_check {
            next_turn = player_id.clone();
        }

        // load game data
        let mut p1_data = self.state.play_data.get(&player_one.id).await
            .unwrap_or_else(|_| { panic!("unable to get play data"); }).unwrap_or_else(|| { panic!("unable to get play data"); });
        let mut p2_data = self.state.play_data.get(&player_two.id).await
            .unwrap_or_else(|_| { panic!("unable to get play data"); }).unwrap_or_else(|| { panic!("unable to get play data"); });

        let current_time = self.runtime.system_time();

        // check last action
        // if last action is stand, then the game must end because both player action choose to stand
        // the winner is player with the biggest score
        if p1_data.last_action == LastAction::Stand || p2_data.last_action == LastAction::Stand {
            let p1_score = p1_data.my_score;
            let p2_score = p2_data.my_score;

            let mut winner = String::from("");
            let mut winner_gid = String::from("");

            if p1_score == p2_score {
                // Draw
            } else if p1_score > p2_score && p1_score <= 21 || p2_score > 21 {
                // Player 1 win
                winner = player_one.name.clone();
                winner_gid = player_one.gid.clone();
            } else if p2_score > p1_score && p2_score <= 21 || p1_score > 21 {
                // Player 2 win
                winner = player_two.name.clone();
                winner_gid = player_two.gid.clone();
            }

            // update data
            p1_data.winner = winner.clone();
            p1_data.game_state = Status::Finish;
            p1_data.last_update = current_time;
            p1_data.player_id_turn = "".to_string();
            p1_data.opponent_score = p2_data.my_score;
            p1_data.opponent_card = p2_data.my_card.clone();

            p2_data.winner = winner.clone();
            p2_data.game_state = Status::Finish;
            p2_data.last_update = current_time;
            p2_data.player_id_turn = "".to_string();
            p2_data.opponent_score = p1_data.my_score;
            p2_data.opponent_card = p1_data.my_card.clone();

            // save data to state
            self.state.game_state.set(GameState { status: Status::Finish, last_update: current_time });
            self.state.play_data.insert(&player_one.id, p1_data).unwrap_or_else(|_| {
                panic!("Failed to update Play Data for {:?} - {:?}", player_one.name, player_one.id);
            });
            self.state.play_data.insert(&player_two.id, p2_data).unwrap_or_else(|_| {
                panic!("Failed to update Play Data for {:?} - {:?}", player_two.name, player_two.id);
            });

            // send message to leaderboard chain
            self.send_game_finish_message(
                player_one.name.clone(),
                player_one.gid.clone(),
                player_two.name.clone(),
                player_two.gid.clone(),
                winner,
                winner_gid,
            ).await;

            // send room status update
            self.send_room_status_update().await;
        } else {
            // update data
            p1_data.player_id_turn = next_turn.clone();
            p1_data.last_action = LastAction::Stand;
            p1_data.last_update = current_time;
            p2_data.player_id_turn = next_turn;
            p2_data.last_action = LastAction::Stand;
            p2_data.last_update = current_time;

            // save data to state
            self.state.game_state.set(GameState { status: Status::Started, last_update: current_time });
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

        // initial check to find out if any player have blackjack (21) score
        let p1_have_blackjack = p1_data.my_score == 21;
        let p2_have_blackjack = p2_data.my_score == 21;

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
            p1_data.my_score = calculate_player_score(*chosen_card, &p1_data.my_card, p1_data.my_score);
            p2_data.opponent_score = calculate_player_score(*chosen_card, &p2_data.opponent_card, p2_data.opponent_score);
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
            p2_data.my_score = calculate_player_score(*chosen_card, &p2_data.my_card, p2_data.my_score);
            p1_data.opponent_score = calculate_player_score(*chosen_card, &p1_data.opponent_card, p1_data.opponent_score);
            current_decks.swap_remove(index as usize);
        }

        // check turn result for winner
        let p1_score = p1_data.my_score;
        let p2_score = p2_data.my_score;

        let mut winner_exist = false;
        let mut winner = String::from("");
        let mut winner_gid = String::from("");

        if p1_have_blackjack && p2_have_blackjack {
            // Draw
            winner_exist = true;
        } else if p1_have_blackjack {
            // Player 1 win
            winner_exist = true;
            winner = player_one.name.clone();
            winner_gid = player_one.gid.clone();
        } else if p2_have_blackjack {
            // Player 2 win
            winner_exist = true;
            winner = player_two.name.clone();
            winner_gid = player_two.gid.clone();
        } else if p1_score == 21 || p2_score > 21 {
            // Player 1 win
            winner_exist = true;
            winner = player_one.name.clone();
            winner_gid = player_one.gid.clone();
        } else if p2_score == 21 || p1_score > 21 {
            // Player 2 win
            winner_exist = true;
            winner = player_two.name.clone();
            winner_gid = player_two.gid.clone();
        }

        let current_time = self.runtime.system_time();

        if winner_exist {
            // update data
            p1_data.winner = winner.clone();
            p1_data.game_state = Status::Finish;
            p1_data.last_action = LastAction::Hit;
            p1_data.last_update = current_time;
            p1_data.player_id_turn = "".to_string();
            p1_data.opponent_score = p2_data.my_score;
            p1_data.opponent_card = p2_data.my_card.clone();

            p2_data.winner = winner.clone();
            p2_data.game_state = Status::Finish;
            p2_data.last_action = LastAction::Hit;
            p2_data.last_update = current_time;
            p2_data.player_id_turn = "".to_string();
            p2_data.opponent_score = p1_data.my_score;
            p2_data.opponent_card = p1_data.my_card.clone();

            // save data to state
            self.state.game_state.set(GameState { status: Status::Finish, last_update: current_time });
            self.state.play_data.insert(&player_one.id, p1_data).unwrap_or_else(|_| {
                panic!("Failed to update Play Data for {:?} - {:?}", player_one.name, player_one.id);
            });
            self.state.play_data.insert(&player_two.id, p2_data).unwrap_or_else(|_| {
                panic!("Failed to update Play Data for {:?} - {:?}", player_two.name, player_two.id);
            });

            // send message to leaderboard chain
            self.send_game_finish_message(
                player_one.name.clone(),
                player_one.gid.clone(),
                player_two.name.clone(),
                player_two.gid.clone(),
                winner,
                winner_gid,
            ).await;

            // send room status update
            self.send_room_status_update().await;
        } else {
            // update data
            p1_data.player_id_turn = next_turn.clone();
            p1_data.last_action = LastAction::Hit;
            p1_data.last_update = current_time;
            p2_data.player_id_turn = next_turn;
            p2_data.last_action = LastAction::Hit;
            p2_data.last_update = current_time;

            // save data to state
            self.state.game_state.set(GameState { status: Status::Started, last_update: current_time });
            self.state.play_data.insert(&player_one.id, p1_data).unwrap_or_else(|_| {
                panic!("Failed to update Play Data for {:?} - {:?}", player_one.name, player_one.id);
            });
            self.state.play_data.insert(&player_two.id, p2_data).unwrap_or_else(|_| {
                panic!("Failed to update Play Data for {:?} - {:?}", player_two.name, player_two.id);
            });
        }
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
        p1_score = calculate_player_score(*chosen_card, &p1_card, p1_score);
        p1_card_for_opponent.push(0);
        new_decks.swap_remove(index as usize);

        // P2 First Card
        ts = self.runtime.system_time().to_string();
        index = random_index(ts, new_decks.len() as u8, p2_id.clone(), "f".to_string());
        chosen_card = new_decks.get(index as usize).unwrap_or_else(|| {
            panic!("unable to draw card");
        });
        p2_card.push(*chosen_card);
        p2_score = calculate_player_score(*chosen_card, &p2_card, p2_score);
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
        p1_score = calculate_player_score(*chosen_card, &p1_card, p1_score);
        p1_score_for_opponent = calculate_player_score(*chosen_card, &p1_card_for_opponent, p1_score_for_opponent);
        new_decks.swap_remove(index as usize);

        // P2 Second Card
        ts = self.runtime.system_time().to_string();
        index = random_index(ts, new_decks.len() as u8, p2_id.clone(), "s".to_string());
        chosen_card = new_decks.get(index as usize).unwrap_or_else(|| {
            panic!("unable to draw card");
        });
        p2_card.push(*chosen_card);
        p2_card_for_opponent.push(*chosen_card);
        p2_score = calculate_player_score(*chosen_card, &p2_card, p2_score);
        p2_score_for_opponent = calculate_player_score(*chosen_card, &p2_card_for_opponent, p2_score_for_opponent);
        new_decks.swap_remove(index as usize);

        let current_time = self.runtime.system_time();

        // save play data
        self.state.play_data.insert(&player_one.id, PlayData {
            p_one_id: player_one.id.clone(),
            p_two_id: p2_id.clone(),
            my_card: p1_card,
            opponent_card: p2_card_for_opponent,
            my_score: p1_score,
            opponent_score: p2_score_for_opponent,
            player_id_turn: player_one.id.clone(),
            last_action: LastAction::None,
            winner: String::from(""),
            game_state: Status::Started,
            last_update: current_time,
        },
        ).unwrap_or_else(|_| {
            panic!("Failed to update Play Data for {:?} - {:?}", player_one.name, player_one.id);
        });
        self.state.play_data.insert(&p2_id, PlayData {
            p_one_id: player_one.id.clone(),
            p_two_id: p2_id.clone(),
            my_card: p2_card,
            opponent_card: p1_card_for_opponent,
            my_score: p2_score,
            opponent_score: p1_score_for_opponent,
            player_id_turn: player_one.id.clone(),
            last_action: LastAction::None,
            winner: String::from(""),
            game_state: Status::Started,
            last_update: current_time,
        },
        ).unwrap_or_else(|_| {
            panic!("Failed to update Play Data for {:?} - {:?}", p2_name, p2_id);
        });

        // save card deck
        self.state.decks.set(new_decks);
    }
}
