use pumpkin_macros::Event;
use std::sync::Arc;

use crate::{entity::player::Player, world::World};

use super::PlayerEvent;

/// A notification fired after a player's active world has changed.
///
/// This is deliberately separate from
/// [`super::player_change_world::PlayerChangeWorldEvent`], which is a
/// cancellable pre-change event. Listeners to this event observe the player
/// after world membership and the entity's world reference are updated.
#[derive(Event, Clone)]
pub struct PlayerChangedWorldEvent {
    pub player: Arc<Player>,
    pub previous_world: Arc<World>,
}

impl PlayerChangedWorldEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, previous_world: Arc<World>) -> Self {
        Self {
            player,
            previous_world,
        }
    }
}

impl PlayerEvent for PlayerChangedWorldEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
