use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// The asynchronous phase of publishing a command tree to a player.
///
/// Listeners may remove top-level command names before the synchronous
/// [`super::player_commands_send::PlayerCommandsSendEvent`] phase runs.
#[derive(Event, Clone)]
pub struct AsyncPlayerCommandsSendEvent {
    pub player: Arc<Player>,
    pub commands: Vec<String>,
}

impl AsyncPlayerCommandsSendEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, commands: Vec<String>) -> Self {
        Self { player, commands }
    }
}

impl PlayerEvent for AsyncPlayerCommandsSendEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
