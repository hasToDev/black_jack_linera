use linera_sdk::base::ChainId;
use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext, QueueView};
use black_jack_chain::{Player, GameState, PlayData, History, Insight, VersionAnalytics, Leaderboard};

#[derive(RootView, async_graphql::SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct BlackJack {
    pub game_state: RegisterView<GameState>,
    pub p1: RegisterView<Player>,
    pub p2: RegisterView<Player>,
    pub decks: RegisterView<Vec<u8>>,
    pub play_data: MapView<String, PlayData>,
    // leaderboard chain
    pub leaderboard: RegisterView<Leaderboard>,
    pub history: QueueView<History>,
    pub leaderboard_on: RegisterView<bool>,
    // room status chain
    pub room_status: MapView<ChainId, Insight>,
    // analytics chain
    pub analytics: MapView<String, VersionAnalytics>,
}
