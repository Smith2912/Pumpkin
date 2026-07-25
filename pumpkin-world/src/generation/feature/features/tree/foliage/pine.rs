use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use super::{FoliagePlacer, LeaveValidator};
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;

pub struct PineFoliagePlacer {
    pub height: IntProvider,
}

impl PineFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        iradius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) -> Vec<BlockPos> {
        let mut foliage_positions = Vec::new();
        let mut radius = 0;
        for y in pine_layer_offsets(offset, foliage_height) {
            FoliagePlacer::generate_square(
                &mut foliage_positions,
                self,
                chunk,
                random,
                node.center,
                radius,
                y,
                node.giant_trunk,
                foliage_provider,
            );
            if radius >= 1 && y == offset - foliage_height + 1 {
                radius -= 1;
            } else if radius < iradius + node.foliage_radius {
                radius += 1;
            }
        }
        foliage_positions
    }

    pub fn get_random_radius(
        placer: &FoliagePlacer,
        random: &mut RandomGenerator,
        base_height: i32,
    ) -> i32 {
        placer.radius.get(random) + random.next_bounded_i32((base_height + 1).max(1))
    }

    pub fn get_random_height(&self, random: &mut RandomGenerator, _trunk_height: i32) -> i32 {
        self.height.get(random)
    }
}

fn pine_layer_offsets(offset: i32, foliage_height: i32) -> impl Iterator<Item = i32> {
    (offset - foliage_height..=offset).rev()
}

impl LeaveValidator for PineFoliagePlacer {
    fn is_invalid_for_leaves(
        &self,
        _random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        _y: i32,
        dz: i32,
        radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        dx == radius && dz == radius && radius > 0
    }
}

#[cfg(test)]
mod tests {
    use super::pine_layer_offsets;

    #[test]
    fn pine_crown_includes_top_and_bottom_layers_from_top_down() {
        assert_eq!(
            pine_layer_offsets(1, 4).collect::<Vec<_>>(),
            vec![1, 0, -1, -2, -3]
        );
    }
}
