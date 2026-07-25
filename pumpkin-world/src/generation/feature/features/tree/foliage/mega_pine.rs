use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::RandomGenerator,
};

use super::{FoliagePlacer, LeaveValidator};
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;

pub struct MegaPineFoliagePlacer {
    pub crown_height: IntProvider,
}

impl MegaPineFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) -> Vec<BlockPos> {
        let mut foliage_positions = Vec::new();
        let pos = node.center;
        let mut current = 0;
        for y in pos.0.y - foliage_height + offset..=pos.0.y + offset {
            let delta = pos.0.y - y;
            let (smooth_radius, jagged_radius) = mega_pine_layer_radius(
                radius,
                node.foliage_radius,
                delta,
                foliage_height,
                current,
                y,
            );
            FoliagePlacer::generate_square(
                &mut foliage_positions,
                self,
                chunk,
                random,
                BlockPos::new(pos.0.x, y, pos.0.z),
                jagged_radius,
                0,
                node.giant_trunk,
                foliage_provider,
            );
            current = smooth_radius;
        }
        foliage_positions
    }
    pub fn get_random_height(&self, random: &mut RandomGenerator, _trunk_height: i32) -> i32 {
        self.crown_height.get(random)
    }
}

fn mega_pine_layer_radius(
    leaf_radius: i32,
    attachment_radius: i32,
    delta_y: i32,
    foliage_height: i32,
    previous_radius: i32,
    world_y: i32,
) -> (i32, i32) {
    let smooth_radius = leaf_radius
        + attachment_radius
        + (delta_y as f32 / foliage_height as f32 * 3.5).floor() as i32;
    let jagged_radius = if delta_y > 0 && smooth_radius == previous_radius && (world_y & 1) == 0 {
        smooth_radius + 1
    } else {
        smooth_radius
    };
    (smooth_radius, jagged_radius)
}

impl LeaveValidator for MegaPineFoliagePlacer {
    fn is_invalid_for_leaves(
        &self,
        _random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        _y: i32,
        dz: i32,
        radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        if dx + dz >= 7 {
            return true;
        }
        dx * dx + dz * dz > radius * radius
    }
}

#[cfg(test)]
mod tests {
    use super::mega_pine_layer_radius;

    #[test]
    fn lower_mega_pine_layers_expand_beyond_the_base_radius() {
        assert_eq!(mega_pine_layer_radius(1, 0, 6, 6, 0, 10), (4, 4));
    }

    #[test]
    fn repeated_even_mega_pine_layers_get_the_jagged_extension() {
        assert_eq!(mega_pine_layer_radius(1, 0, 2, 6, 2, 10), (2, 3));
    }
}
