use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// A mutable view of the top-level commands about to be published to a player.
///
/// This is separate from [`super::player_command_send::PlayerCommandSendEvent`],
/// which is fired before a player executes a command. Removing a name here
/// hides that command from the client's command tree without cancelling command
/// execution.
#[derive(Event, Clone)]
pub struct PlayerCommandsSendEvent {
    pub player: Arc<Player>,
    pub commands: Vec<String>,
}

impl PlayerCommandsSendEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, commands: Vec<String>) -> Self {
        Self { player, commands }
    }
}

impl PlayerEvent for PlayerCommandsSendEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
