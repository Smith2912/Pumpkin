use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

use super::BlockEvent;

/// An event fired before fluid moves from one block position to another.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFromToEvent {
    /// The world containing the fluid movement.
    pub world: Arc<World>,

    /// The position of the source fluid block.
    pub from_position: BlockPos,

    /// The target position that would receive the fluid.
    pub to_position: BlockPos,
}

impl BlockFromToEvent {
    /// Creates a native fluid movement event.
    #[must_use]
    pub fn new(world: Arc<World>, from_position: BlockPos, to_position: BlockPos) -> Self {
        Self {
            world,
            from_position,
            to_position,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockFromToEvent {
    fn get_block(&self) -> &Block {
        self.world.get_block(&self.from_position)
    }
}
