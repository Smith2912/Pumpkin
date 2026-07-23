use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::world::World;

use super::BlockEvent;

/// An event that occurs when a player attempts to build on a block.
///
/// This event contains information about the block to build, whether building is allowed,
/// the player attempting to build, and the block being built upon.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockCanBuildEvent {
    /// The world where the placement is being attempted.
    ///
    /// Legacy WASM event payloads do not carry a world, so this remains optional
    /// until that ABI grows a location field.
    pub world: Option<Arc<World>>,

    /// The position where the new block would be placed.
    ///
    /// Legacy WASM event payloads do not carry a position, so this remains
    /// optional for compatibility with those payloads.
    pub block_position: Option<BlockPos>,

    /// The block that the player is attempting to build.
    pub block_to_build: &'static Block,

    /// A boolean indicating whether building is allowed.
    pub buildable: bool,

    /// The player attempting to build.
    pub player: Arc<Player>,

    /// The block being built upon.
    pub block: &'static Block,
}

impl BlockCanBuildEvent {
    /// Creates a native placement decision with a complete world location.
    #[must_use]
    pub fn new(
        world: Arc<World>,
        block_position: BlockPos,
        block_to_build: &'static Block,
        buildable: bool,
        player: Arc<Player>,
        block: &'static Block,
    ) -> Self {
        Self {
            world: Some(world),
            block_position: Some(block_position),
            block_to_build,
            buildable,
            player,
            block,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockCanBuildEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
