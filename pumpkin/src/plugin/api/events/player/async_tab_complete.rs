use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An asynchronous, mutable command-completion request.
///
/// When a listener marks this event as handled, `completions` replaces
/// Pumpkin's native suggestions. Cancellation produces an empty result.
#[cancellable]
#[derive(Event, Clone)]
pub struct AsyncTabCompleteEvent {
    pub player: Arc<Player>,
    pub buffer: String,
    pub completions: Vec<String>,
    pub handled: bool,
}

impl AsyncTabCompleteEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, buffer: String) -> Self {
        Self {
            player,
            buffer,
            completions: Vec::new(),
            handled: false,
            cancelled: false,
        }
    }
}

impl PlayerEvent for AsyncTabCompleteEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
