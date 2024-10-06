use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext, QueueView};
use black_jack_chain::{Player, GameState, PlayData, History};

#[derive(RootView, async_graphql::SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct BlackJack {
    pub game_state: RegisterView<GameState>,
    pub p1: RegisterView<Player>,
    pub p2: RegisterView<Player>,
    pub decks: RegisterView<Vec<u8>>,
    pub play_data: MapView<String, PlayData>,
    // leaderboard and history belong to leaderboard chain
    pub leaderboard: MapView<String, Player>,
    pub game_count: RegisterView<u32>,
    pub history: QueueView<History>,
}
