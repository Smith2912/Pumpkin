use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// Describes the native reason for a player teleport.
///
/// Pumpkin callers should use the narrowest cause they know. Compatibility
/// layers can preserve richer source-API causes when they dispatch their own
/// event before calling Pumpkin's post-event teleport path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeleportCause {
    ChorusFruit,
    Command,
    EndGateway,
    EndPortal,
    EntityPortal,
    NetherPortal,
    Plugin,
    Spectate,
    Unknown,
}

/// An event that occurs when a player teleports.
///
/// If the event is cancelled, the teleportation will not happen.
///
/// This event contains information about the player, the position from which the player teleported, and the position to which the player teleported.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerTeleportEvent {
    /// The player who teleported.
    pub player: Arc<Player>,

    /// The position from which the player teleported.
    pub from: Vector3<f64>,

    /// The position to which the player teleported.
    pub to: Vector3<f64>,

    /// The player's orientation before teleporting, when supplied by the
    /// native caller. Legacy plugin adapters may leave this unset.
    pub from_yaw: Option<f32>,

    /// The player's pitch before teleporting, when supplied by the caller.
    pub from_pitch: Option<f32>,

    /// The requested orientation after teleporting. Listeners may update
    /// these values together with [`Self::to`].
    pub to_yaw: Option<f32>,

    /// The requested pitch after teleporting.
    pub to_pitch: Option<f32>,

    /// The native cause of the teleport.
    pub cause: TeleportCause,
}

impl PlayerTeleportEvent {
    /// Creates a new instance of `PlayerTeleportEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who teleported.
    /// - `from`: The position from which the player teleported.
    /// - `to`: The position to which the player teleported.
    ///
    /// # Returns
    /// A new instance of `PlayerTeleportEvent`.
    pub const fn new(
        player: Arc<Player>,
        from: Vector3<f64>,
        to: Vector3<f64>,
        from_yaw: Option<f32>,
        from_pitch: Option<f32>,
        to_yaw: Option<f32>,
        to_pitch: Option<f32>,
        cause: TeleportCause,
    ) -> Self {
        Self {
            player,
            from,
            to,
            from_yaw,
            from_pitch,
            to_yaw,
            to_pitch,
            cause,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerTeleportEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
