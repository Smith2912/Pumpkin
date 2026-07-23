use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

use super::BlockEvent;

/// An event that occurs when a block is burned.
///
/// This event contains information about the block that ignited the fire and the block that is burning.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockBurnEvent {
    /// The world where the block is burning.
    ///
    /// Legacy WASM event payloads do not carry a world, so this remains optional
    /// until that ABI grows a location field.
    pub world: Option<Arc<World>>,

    /// The position of the fire that caused the burn.
    pub igniting_block_position: Option<BlockPos>,

    /// The position of the block being burned.
    pub block_position: Option<BlockPos>,

    /// The block that is igniting the fire.
    pub igniting_block: &'static Block,

    /// The block that is burning.
    pub block: &'static Block,
}

impl BlockBurnEvent {
    /// Creates a native burn event with complete world positions.
    #[must_use]
    pub fn new(
        world: Arc<World>,
        igniting_block_position: BlockPos,
        block_position: BlockPos,
        igniting_block: &'static Block,
        block: &'static Block,
    ) -> Self {
        Self {
            world: Some(world),
            igniting_block_position: Some(igniting_block_position),
            block_position: Some(block_position),
            igniting_block,
            block,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockBurnEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
