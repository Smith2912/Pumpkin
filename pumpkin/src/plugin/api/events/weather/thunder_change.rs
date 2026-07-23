use std::sync::Arc;

use pumpkin_macros::{Event, cancellable};

use crate::world::World;

use super::WeatherEvent;

/// Fired before a world starts or stops thundering.
///
/// Cancelling the event keeps the world's current thunder state and timer.
#[cancellable]
#[derive(Event, Clone)]
pub struct ThunderChangeEvent {
    /// The world whose thunder state is changing.
    pub world: Arc<World>,

    /// The proposed thunder state.
    pub to_thunder_state: bool,
}

impl ThunderChangeEvent {
    /// Creates a thunder-change event.
    #[must_use]
    pub const fn new(world: Arc<World>, to_thunder_state: bool) -> Self {
        Self {
            world,
            to_thunder_state,
            cancelled: false,
        }
    }
}

impl WeatherEvent for ThunderChangeEvent {
    fn world(&self) -> &Arc<World> {
        &self.world
    }
}
