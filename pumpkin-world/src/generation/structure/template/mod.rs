//! NBT Structure Template System
//!
//! This module provides functionality for loading and placing Minecraft structure
//! templates from `.nbt` files. This enables exact vanilla structure matching and
//! dramatically simplifies implementing structures like igloos, shipwrecks, villages, etc.
//!
//! # Architecture
//!
//! - [`StructureTemplate`]: Represents a loaded NBT template with size, palette, and blocks
//! - [`TemplatePiece`]: A structure piece that places blocks from a template
//! - [`Rotation`] and [`Mirror`]: Transform positions and block properties
//! - [`TemplateCache`]: Lazy-loading cache for embedded template files
//!
//! # Example Usage
//!
//! ```ignore
//! use pumpkin_world::generation::structure::template::{TemplateCache, TemplatePiece};
//! use pumpkin_data::Rotation;
//!
//! // Load a template from the cache
//! let template = TemplateCache::get("igloo/top").expect("Template not found");
//!
//! // Create a piece to place the template
//! let piece = TemplatePiece::new(template, rotation, mirror, position);
//! ```

mod block_state_resolver;
mod cache;
pub mod processor;
mod structure_template;
mod template_piece;

use pumpkin_data::Mirror;
use pumpkin_data::Rotation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::block_box::BlockBox;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomImpl, hash_block_pos, legacy_rand::LegacyRand};
use uuid::Uuid;

use crate::ProtoChunk;

pub use block_state_resolver::BlockStateResolver;
pub use cache::{
    TemplateCache, get_pool_elements, get_processor_list_json, get_template,
    get_template_pool_json, global_cache,
};
pub use processor::StructureProcessor;
pub use pumpkin_data::{Mirror as BlockMirror, Rotation as BlockRotation};
pub use structure_template::{PaletteEntry, StructureTemplate, TemplateBlock, TemplateEntity};
pub use template_piece::TemplatePiece;

/// Transforms and queues the entities stored in a template exactly once, in the chunk that
/// contains each entity's transformed block position.
#[allow(clippy::too_many_arguments)]
pub fn enqueue_template_entities(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    template_identity: &str,
    origin: Vector3<i32>,
    offset: (i32, i32),
    rotation: Rotation,
    mirror: Mirror,
    chunk_box: &BlockBox,
) {
    let (rotated_ox, rotated_oz) = rotation.rotate_offset(offset.0, offset.1);
    let world_origin = Vector3::new(origin.x + rotated_ox, origin.y, origin.z + rotated_oz);

    for (entity_index, entity) in template.entities.iter().enumerate() {
        if let Some(nbt) = prepare_template_entity(
            entity,
            template.size,
            template_identity,
            entity_index,
            world_origin,
            rotation,
            mirror,
            chunk_box,
        ) {
            chunk.pending_structure_entities.push(nbt);
        }
    }
}

fn prepare_template_entity(
    entity: &TemplateEntity,
    template_size: Vector3<i32>,
    template_identity: &str,
    entity_index: usize,
    world_origin: Vector3<i32>,
    rotation: Rotation,
    mirror: Mirror,
    chunk_box: &BlockBox,
) -> Option<NbtCompound> {
    let mirrored_block_pos = mirror.transform_pos(entity.block_pos, template_size);
    let local_block_pos = rotation.transform_pos(mirrored_block_pos, template_size);
    let world_block_pos = Vector3::new(
        world_origin.x + local_block_pos.x,
        world_origin.y + local_block_pos.y,
        world_origin.z + local_block_pos.z,
    );
    if !chunk_box.contains_pos(&world_block_pos) {
        return None;
    }

    let mirrored_pos = transform_entity_position(entity.pos, template_size, mirror);
    let local_pos = transform_entity_position(mirrored_pos, template_size, rotation);
    let world_pos = Vector3::new(
        f64::from(world_origin.x) + local_pos.x,
        f64::from(world_origin.y) + local_pos.y,
        f64::from(world_origin.z) + local_pos.z,
    );

    let mut nbt = entity.nbt.clone();
    nbt.child_tags.remove("UUID");
    nbt.child_tags.remove("UUIDMost");
    nbt.child_tags.remove("UUIDLeast");
    let entity_id = nbt
        .get_string("id")
        .unwrap_or("minecraft:unknown")
        .to_owned();
    nbt.put_uuid(
        "UUID",
        deterministic_structure_entity_uuid(
            template_identity,
            entity_index,
            &entity_id,
            world_block_pos,
            world_pos,
        ),
    );
    nbt.child_tags.insert(
        "Pos".into(),
        NbtTag::List(vec![
            NbtTag::Double(world_pos.x),
            NbtTag::Double(world_pos.y),
            NbtTag::Double(world_pos.z),
        ]),
    );
    nbt.child_tags.entry("Motion".into()).or_insert_with(|| {
        NbtTag::List(vec![
            NbtTag::Double(0.0),
            NbtTag::Double(0.0),
            NbtTag::Double(0.0),
        ])
    });

    let (yaw, pitch) = nbt
        .get_list("Rotation")
        .and_then(|values| {
            Some((
                values.first()?.extract_float()?,
                values.get(1)?.extract_float()?,
            ))
        })
        .unwrap_or((0.0, 0.0));
    let yaw = transform_entity_yaw(yaw, rotation, mirror);
    nbt.child_tags.insert(
        "Rotation".into(),
        NbtTag::List(vec![NbtTag::Float(yaw), NbtTag::Float(pitch)]),
    );
    Some(nbt)
}

fn deterministic_structure_entity_uuid(
    template_identity: &str,
    entity_index: usize,
    entity_id: &str,
    block_pos: Vector3<i32>,
    pos: Vector3<f64>,
) -> Uuid {
    let mut identity = Vec::with_capacity(template_identity.len() + entity_id.len() + 44);
    identity.extend_from_slice(template_identity.as_bytes());
    identity.extend_from_slice(&(entity_index as u64).to_be_bytes());
    identity.extend_from_slice(entity_id.as_bytes());
    identity.extend_from_slice(&block_pos.x.to_be_bytes());
    identity.extend_from_slice(&block_pos.y.to_be_bytes());
    identity.extend_from_slice(&block_pos.z.to_be_bytes());
    identity.extend_from_slice(&pos.x.to_bits().to_be_bytes());
    identity.extend_from_slice(&pos.y.to_bits().to_be_bytes());
    identity.extend_from_slice(&pos.z.to_bits().to_be_bytes());
    Uuid::new_v3(&Uuid::NAMESPACE_OID, &identity)
}

fn transform_entity_position<T>(
    pos: Vector3<f64>,
    template_size: Vector3<i32>,
    transform: T,
) -> Vector3<f64>
where
    T: EntityPositionTransform,
{
    transform.transform_entity_position(pos, template_size)
}

trait EntityPositionTransform {
    fn transform_entity_position(
        self,
        pos: Vector3<f64>,
        template_size: Vector3<i32>,
    ) -> Vector3<f64>;
}

impl EntityPositionTransform for Mirror {
    fn transform_entity_position(
        self,
        pos: Vector3<f64>,
        template_size: Vector3<i32>,
    ) -> Vector3<f64> {
        match self {
            Self::None => pos,
            Self::LeftRight => Vector3::new(f64::from(template_size.x) - pos.x, pos.y, pos.z),
            Self::FrontBack => Vector3::new(pos.x, pos.y, f64::from(template_size.z) - pos.z),
        }
    }
}

impl EntityPositionTransform for Rotation {
    fn transform_entity_position(
        self,
        pos: Vector3<f64>,
        template_size: Vector3<i32>,
    ) -> Vector3<f64> {
        match self {
            Self::None => pos,
            Self::Clockwise90 => Vector3::new(f64::from(template_size.z) - pos.z, pos.y, pos.x),
            Self::Rotate180 => Vector3::new(
                f64::from(template_size.x) - pos.x,
                pos.y,
                f64::from(template_size.z) - pos.z,
            ),
            Self::CounterClockwise90 => {
                Vector3::new(pos.z, pos.y, f64::from(template_size.x) - pos.x)
            }
        }
    }
}

fn transform_entity_yaw(yaw: f32, rotation: Rotation, mirror: Mirror) -> f32 {
    let angle = (yaw + 180.0).rem_euclid(360.0) - 180.0;
    let rotated = angle
        + match rotation {
            Rotation::None => 0.0,
            Rotation::Clockwise90 => 90.0,
            Rotation::Rotate180 => 180.0,
            Rotation::CounterClockwise90 => 270.0,
        };
    let mirrored = match mirror {
        Mirror::None => angle,
        Mirror::FrontBack => -angle,
        Mirror::LeftRight => 180.0 - angle,
    };
    rotated + mirrored - angle
}

/// Places a template at a world origin with an un-rotated XZ offset.
///
/// All rotation is handled internally:
/// - The offset is rotated to position the template correctly
/// - Block positions within the template are rotated
/// - Directional block properties (facing, axis, etc.) are rotated
/// - Block entities are created from template NBT data
///
/// `origin` is the base world position (x, y, z).
/// `offset` is the un-rotated XZ offset from origin (`x_offset`, `z_offset`) - rotation is applied automatically.
#[allow(clippy::too_many_arguments)]
pub fn place_template(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    offset: (i32, i32),
    rotation: Rotation,
    skip_air: bool,
    apply_waterlogging: bool,
    processors: &[StructureProcessor],
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
) {
    let (rotated_ox, rotated_oz) = rotation.rotate_offset(offset.0, offset.1);
    let world_x = origin.x + rotated_ox;
    let world_z = origin.z + rotated_oz;

    for block in &template.blocks {
        let palette_entry = &template.palette[block.state as usize];

        // Structure blocks are data markers and structure void preserves the existing block.
        if palette_entry.name == "minecraft:structure_void"
            || palette_entry.name == "minecraft:structure_block"
        {
            continue;
        }

        // Skip air blocks when using IGNORE_AIR processor (e.g. nether fossils)
        if skip_air && palette_entry.name == "minecraft:air" {
            continue;
        }

        let mut block_entity_nbt = block.nbt.clone();
        let mut placed_entry = palette_entry.clone();

        // Jigsaw blocks are replaced during template processing, before block entities are
        // collected. Keeping this in the placement pipeline avoids stale jigsaw entities.
        if palette_entry.name == "minecraft:jigsaw" {
            let final_state = block_entity_nbt
                .as_ref()
                .and_then(|nbt| nbt.get_string("final_state"))
                .unwrap_or("minecraft:air");
            placed_entry = PaletteEntry::from_string(final_state);
            block_entity_nbt = None;
        }

        // Resolve block state with rotation applied to directional properties
        let Some(mut state) =
            BlockStateResolver::resolve(&placed_entry, rotation, Mirror::default())
        else {
            continue;
        };

        // Rotate block position within template bounds
        let local_pos = rotation.transform_pos(block.pos, template.size);

        let wx = world_x + local_pos.x;
        let wy = origin.y + local_pos.y;
        let wz = world_z + local_pos.z;

        if let Some(bbox) = chunk_box
            && (wx < bbox.min.x
                || wx > bbox.max.x
                || wy < bbox.min.y
                || wy > bbox.max.y
                || wz < bbox.min.z
                || wz > bbox.max.z)
        {
            continue;
        }

        let world_pos = Vector3::new(wx, wy, wz);

        if apply_waterlogging
            && chunk.get_block_state(&world_pos).to_block_id() == pumpkin_data::Block::WATER.id
            && let Some((_, waterlogged)) = placed_entry
                .properties
                .iter_mut()
                .find(|(name, _)| name == "waterlogged")
        {
            *waterlogged = "true".to_string();
            if let Some(waterlogged_state) =
                BlockStateResolver::resolve(&placed_entry, rotation, Mirror::default())
            {
                state = waterlogged_state;
            }
        }

        // Apply processors
        let mut should_place = true;
        for processor in processors {
            let Some(processed_state) = processor.process(chunk, world_pos, state) else {
                should_place = false;
                break;
            };
            state = processed_state;
        }
        if !should_place {
            continue;
        }

        chunk.set_block_state(wx, wy, wz, state);

        // Create block entities for interactive blocks (furnaces, chests, etc.)
        let block_entity_id = get_block_entity_id(&placed_entry.name);
        if block_entity_nbt.is_some() || block_entity_id.is_some() {
            let block_entity_id = block_entity_id.unwrap_or(&placed_entry.name);
            let mut placed_nbt = NbtCompound::new();

            placed_nbt.put_string("id", block_entity_id.to_string());
            placed_nbt.put_int("x", wx);
            placed_nbt.put_int("y", wy);
            placed_nbt.put_int("z", wz);

            if let Some(template_nbt) = &block_entity_nbt {
                for (key, value) in &template_nbt.child_tags {
                    if key.as_ref() != "x"
                        && key.as_ref() != "y"
                        && key.as_ref() != "z"
                        && key.as_ref() != "id"
                    {
                        placed_nbt.child_tags.insert(key.clone(), value.clone());
                    }
                }
            }

            if placed_nbt.get_string("LootTable").is_some()
                && placed_nbt.get_long("LootTableSeed").is_none()
            {
                let mut random = LegacyRand::from_seed(hash_block_pos(wx, wy, wz) as u64);
                placed_nbt.put_long("LootTableSeed", random.next_i64());
            }

            chunk.add_block_entity(placed_nbt);
        }
    }
}

/// Returns the block entity ID for blocks that require one, or None if not needed.
pub(crate) fn get_block_entity_id(block_name: &str) -> Option<&'static str> {
    match block_name {
        "minecraft:furnace" => Some("minecraft:furnace"),
        "minecraft:chest" => Some("minecraft:chest"),
        "minecraft:trapped_chest" => Some("minecraft:trapped_chest"),
        "minecraft:barrel" => Some("minecraft:barrel"),
        "minecraft:hopper" => Some("minecraft:hopper"),
        "minecraft:dropper" => Some("minecraft:dropper"),
        "minecraft:dispenser" => Some("minecraft:dispenser"),
        "minecraft:brewing_stand" => Some("minecraft:brewing_stand"),
        "minecraft:blast_furnace" => Some("minecraft:blast_furnace"),
        "minecraft:smoker" => Some("minecraft:smoker"),
        "minecraft:shulker_box" => Some("minecraft:shulker_box"),
        "minecraft:bed" => Some("minecraft:bed"),
        "minecraft:sign"
        | "minecraft:oak_sign"
        | "minecraft:spruce_sign"
        | "minecraft:birch_sign"
        | "minecraft:jungle_sign"
        | "minecraft:acacia_sign"
        | "minecraft:dark_oak_sign"
        | "minecraft:mangrove_sign"
        | "minecraft:cherry_sign"
        | "minecraft:bamboo_sign"
        | "minecraft:crimson_sign"
        | "minecraft:warped_sign" => Some("minecraft:sign"),
        "minecraft:hanging_sign" => Some("minecraft:hanging_sign"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{TemplateEntity, get_template, prepare_template_entity};
    use pumpkin_data::{Mirror, Rotation};
    use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
    use pumpkin_util::math::{block_box::BlockBox, vector3::Vector3};
    use uuid::Uuid;

    fn template_villager() -> TemplateEntity {
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", "minecraft:villager".to_string());
        nbt.put_uuid("UUID", Uuid::nil());
        nbt.put_list(
            "Motion",
            vec![
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
            ],
        );
        nbt.put_list("Rotation", vec![NbtTag::Float(10.0), NbtTag::Float(5.0)]);
        TemplateEntity {
            pos: Vector3::new(0.5, 1.0, 1.5),
            block_pos: Vector3::new(0, 1, 1),
            nbt,
        }
    }

    #[test]
    fn structure_entity_is_rotated_positioned_and_given_a_stable_runtime_identity() {
        let nbt = prepare_template_entity(
            &template_villager(),
            Vector3::new(4, 3, 6),
            "test:villager",
            0,
            Vector3::new(100, 64, 200),
            Rotation::Clockwise90,
            Mirror::None,
            &BlockBox::new(96, 0, 192, 111, 255, 207),
        )
        .expect("entity belongs to this chunk");

        let pos = nbt.get_list("Pos").expect("transformed position");
        assert_eq!(pos[0].extract_double(), Some(104.5));
        assert_eq!(pos[1].extract_double(), Some(65.0));
        assert_eq!(pos[2].extract_double(), Some(200.5));
        assert_eq!(
            nbt.get_list("Rotation")
                .and_then(|rotation| rotation[0].extract_float()),
            Some(100.0)
        );
        let uuid = nbt.get_uuid("UUID").expect("stable runtime identity");
        let repeat = prepare_template_entity(
            &template_villager(),
            Vector3::new(4, 3, 6),
            "test:villager",
            0,
            Vector3::new(100, 64, 200),
            Rotation::Clockwise90,
            Mirror::None,
            &BlockBox::new(96, 0, 192, 111, 255, 207),
        )
        .expect("same entity belongs to this chunk");
        assert_eq!(repeat.get_uuid("UUID"), Some(uuid));

        let second_entry = prepare_template_entity(
            &template_villager(),
            Vector3::new(4, 3, 6),
            "test:villager",
            1,
            Vector3::new(100, 64, 200),
            Rotation::Clockwise90,
            Mirror::None,
            &BlockBox::new(96, 0, 192, 111, 255, 207),
        )
        .expect("second entry belongs to this chunk");
        assert_ne!(second_entry.get_uuid("UUID"), Some(uuid));
    }

    #[test]
    fn structure_entity_is_queued_only_by_its_own_chunk() {
        let result = prepare_template_entity(
            &template_villager(),
            Vector3::new(4, 3, 6),
            "test:villager",
            0,
            Vector3::new(100, 64, 200),
            Rotation::Clockwise90,
            Mirror::None,
            &BlockBox::new(112, 0, 192, 127, 255, 207),
        );

        assert!(result.is_none());
    }

    #[test]
    fn bundled_village_population_templates_retain_their_entities() {
        let villager = get_template("minecraft:village/plains/villagers/unemployed")
            .expect("bundled villager template");
        let golem = get_template("minecraft:village/common/iron_golem")
            .expect("bundled iron golem template");

        assert!(
            villager
                .entities
                .iter()
                .any(|entity| { entity.nbt.get_string("id") == Some("minecraft:villager") })
        );
        assert!(
            golem
                .entities
                .iter()
                .any(|entity| { entity.nbt.get_string("id") == Some("minecraft:iron_golem") })
        );
    }
}
