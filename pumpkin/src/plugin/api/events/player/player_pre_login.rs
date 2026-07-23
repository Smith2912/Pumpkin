use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::net::SocketAddr;
use uuid::Uuid;

/// An event that occurs after a connection has been authenticated but before
/// Pumpkin constructs and loads the player's in-game state.
///
/// This separate lifecycle phase lets permission providers load user data
/// before the later [`super::player_login::PlayerLoginEvent`] is fired.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPreLoginEvent {
    pub name: String,
    pub uuid: Uuid,
    pub hostname: String,
    pub address: SocketAddr,
    pub real_address: SocketAddr,
    pub kick_message: TextComponent,
}

impl PlayerPreLoginEvent {
    #[must_use]
    pub const fn new(
        name: String,
        uuid: Uuid,
        hostname: String,
        address: SocketAddr,
        real_address: SocketAddr,
        kick_message: TextComponent,
    ) -> Self {
        Self {
            name,
            uuid,
            hostname,
            address,
            real_address,
            kick_message,
            cancelled: false,
        }
    }
}
