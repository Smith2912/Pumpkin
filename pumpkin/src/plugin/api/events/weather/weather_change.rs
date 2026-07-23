use std::sync::Arc;

use pumpkin_macros::{Event, cancellable};

use crate::world::World;

use super::WeatherEvent;

/// Fired before a world starts or stops raining.
///
/// Cancelling the event keeps the world's current rain state and timers.
#[cancellable]
#[derive(Event, Clone)]
pub struct WeatherChangeEvent {
    /// The world whose rain state is changing.
    pub world: Arc<World>,

    /// The proposed rain state.
    pub to_weather_state: bool,
}

impl WeatherChangeEvent {
    /// Creates a weather-change event.
    #[must_use]
    pub const fn new(world: Arc<World>, to_weather_state: bool) -> Self {
        Self {
            world,
            to_weather_state,
            cancelled: false,
        }
    }
}

impl WeatherEvent for WeatherChangeEvent {
    fn world(&self) -> &Arc<World> {
        &self.world
    }
}
