use super::World;
use pumpkin_protocol::java::client::play::{CGameEvent, GameEvent};
use rand::RngExt;

// Weather timing constants
const RAIN_DELAY_MIN: i32 = 12_000;
const RAIN_DELAY_MAX: i32 = 180_000;
const RAIN_DURATION_MIN: i32 = 12_000;
const RAIN_DURATION_MAX: i32 = 24_000;
const THUNDER_DELAY_MIN: i32 = 12_000;
const THUNDER_DELAY_MAX: i32 = 180_000;
const THUNDER_DURATION_MIN: i32 = 3_600;
const THUNDER_DURATION_MAX: i32 = 15_600;

const WEATHER_TRANSITION_SPEED: f32 = 0.01;

pub struct Weather {
    pub clear_weather_time: i32,
    pub raining: bool,
    pub rain_time: i32,
    pub thundering: bool,
    pub thunder_time: i32,

    pub rain_level: f32,
    pub old_rain_level: f32,
    pub thunder_level: f32,
    pub old_thunder_level: f32,

    pub weather_cycle_enabled: bool,
}

impl Default for Weather {
    fn default() -> Self {
        Self::new()
    }
}

impl Weather {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            clear_weather_time: 0,
            raining: false,
            rain_time: 0,
            thundering: false,
            thunder_time: 0,
            rain_level: 0.0,
            old_rain_level: 0.0,
            thunder_level: 0.0,
            old_thunder_level: 0.0,
            weather_cycle_enabled: true,
        }
    }

    pub(super) fn apply_weather_parameters(
        &mut self,
        world: &World,
        clear_time: i32,
        rain_time: i32,
        thunder_time: i32,
        raining: bool,
        thundering: bool,
    ) {
        let was_raining = self.raining;

        self.clear_weather_time = clear_time;
        self.rain_time = rain_time;
        self.thunder_time = thunder_time;
        self.raining = raining;
        self.thundering = thundering;

        if was_raining != raining {
            if was_raining {
                world.broadcast_packet_all(&CGameEvent::new(GameEvent::EndRaining, 0.0));
            } else {
                world.broadcast_packet_all(&CGameEvent::new(GameEvent::BeginRaining, 0.0));
            }
        }
    }

    pub(super) fn tick_weather_levels(&mut self, world: &World) {
        // Update visual transitions
        self.old_rain_level = self.rain_level;
        self.old_thunder_level = self.thunder_level;

        if self.raining {
            self.rain_level = (self.rain_level + WEATHER_TRANSITION_SPEED).min(1.0);
        } else {
            self.rain_level = (self.rain_level - WEATHER_TRANSITION_SPEED).max(0.0);
        }

        if self.thundering {
            self.thunder_level = (self.thunder_level + WEATHER_TRANSITION_SPEED).min(1.0);
        } else {
            self.thunder_level = (self.thunder_level - WEATHER_TRANSITION_SPEED).max(0.0);
        }

        // Broadcast level changes if needed
        if (self.old_rain_level - self.rain_level).abs() > f32::EPSILON {
            world.broadcast_packet_all(&CGameEvent::new(
                GameEvent::RainLevelChange,
                self.rain_level,
            ));
        }

        if (self.old_thunder_level - self.thunder_level).abs() > f32::EPSILON {
            world.broadcast_packet_all(&CGameEvent::new(
                GameEvent::ThunderLevelChange,
                self.thunder_level,
            ));
        }
    }

    /// Advances weather timers and returns a requested state transition.
    ///
    /// State changes are returned instead of applied directly so the world can
    /// fire cancellable plugin events before committing them.
    pub(super) fn advance_weather_cycle(&mut self) -> Option<(bool, bool)> {
        if !self.weather_cycle_enabled {
            return None;
        }

        let mut raining = self.raining;
        let mut thundering = self.thundering;

        // Removed async since there are no await calls
        if self.clear_weather_time > 0 {
            self.clear_weather_time -= 1;
            self.thunder_time = i32::from(!self.thundering);
            self.rain_time = i32::from(!self.raining);
            thundering = false;
            raining = false;
        } else {
            // Handle thunder timing
            if self.thunder_time > 0 {
                self.thunder_time -= 1;
                if self.thunder_time == 0 {
                    thundering = !self.thundering;
                }
            } else if self.thundering {
                self.thunder_time =
                    rand::rng().random_range(THUNDER_DURATION_MIN..=THUNDER_DURATION_MAX);
            } else {
                self.thunder_time = rand::rng().random_range(THUNDER_DELAY_MIN..=THUNDER_DELAY_MAX);
            }

            // Handle rain timing
            if self.rain_time > 0 {
                self.rain_time -= 1;
                if self.rain_time == 0 {
                    raining = !self.raining;
                }
            } else if self.raining {
                self.rain_time = rand::rng().random_range(RAIN_DURATION_MIN..=RAIN_DURATION_MAX);
            } else {
                self.rain_time = rand::rng().random_range(RAIN_DELAY_MIN..=RAIN_DELAY_MAX);
            }
        }

        ((raining, thundering) != (self.raining, self.thundering)).then_some((raining, thundering))
    }
}

impl Clone for Weather {
    fn clone(&self) -> Self {
        Self {
            clear_weather_time: self.clear_weather_time,
            raining: self.raining,
            rain_time: self.rain_time,
            thundering: self.thundering,
            thunder_time: self.thunder_time,
            rain_level: self.rain_level,
            old_rain_level: self.old_rain_level,
            thunder_level: self.thunder_level,
            old_thunder_level: self.old_thunder_level,
            weather_cycle_enabled: self.weather_cycle_enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Weather;

    #[test]
    fn disabled_weather_cycle_does_not_advance_or_request_transition() {
        let mut weather = Weather::new();
        weather.weather_cycle_enabled = false;
        weather.rain_time = 1;

        assert_eq!(weather.advance_weather_cycle(), None);
        assert_eq!(weather.rain_time, 1);
        assert!(!weather.raining);
    }

    #[test]
    fn weather_cycle_requests_transition_before_mutating_state() {
        let mut weather = Weather::new();
        weather.rain_time = 1;

        assert_eq!(weather.advance_weather_cycle(), Some((true, false)));
        assert_eq!(weather.rain_time, 0);
        assert!(!weather.raining);
    }
}
