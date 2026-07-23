pub mod thunder_change;
pub mod weather_change;

use std::sync::Arc;

use crate::world::World;

/// Common data exposed by events that change a world's weather state.
pub trait WeatherEvent: Send + Sync {
    /// Returns the world whose weather is changing.
    fn world(&self) -> &Arc<World>;
}
