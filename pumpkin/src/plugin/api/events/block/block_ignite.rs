use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

use super::BlockEvent;

/// The native reason a block is being ignited.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IgniteCause {
    /// Existing fire spread naturally to the target position.
    Spread,
}

/// An event fired before Pumpkin places fire at a block position.
///
/// The initial contract covers the natural scheduled fire-spread path. Player,
/// lava, lightning, and projectile ignition paths will use additional causes
/// when those native paths expose the required source data.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockIgniteEvent {
    /// The world where fire would be placed.
    pub world: Arc<World>,

    /// The position that would become fire.
    pub block_position: BlockPos,

    /// The reason for the ignition.
    pub cause: IgniteCause,

    /// The source fire block for natural spread.
    pub igniting_block_position: Option<BlockPos>,
}

impl BlockIgniteEvent {
    /// Creates a natural fire-spread ignition event.
    #[must_use]
    pub fn spread(
        world: Arc<World>,
        block_position: BlockPos,
        igniting_block_position: BlockPos,
    ) -> Self {
        Self {
            world,
            block_position,
            cause: IgniteCause::Spread,
            igniting_block_position: Some(igniting_block_position),
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockIgniteEvent {
    fn get_block(&self) -> &Block {
        self.world.get_block(&self.block_position)
    }
}
