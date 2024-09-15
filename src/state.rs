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
    pub leaderboard: MapView<String, Player>,
    pub history: QueueView<History>,
}
