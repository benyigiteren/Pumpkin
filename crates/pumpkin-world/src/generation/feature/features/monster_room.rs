use pumpkin_data::{Block, BlockDirection};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

const DUNGEON_MOBS: [&str; 4] = [
    "minecraft:skeleton",
    "minecraft:zombie",
    "minecraft:zombie",
    "minecraft:spider",
];

pub struct DungeonFeature;

impl DungeonFeature {
    #[expect(clippy::too_many_lines)]
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let xr = random.next_bounded_i32(2) + 2;
        let zr = random.next_bounded_i32(2) + 2;

        let min_x = -xr - 1;
        let max_x = xr + 1;
        let min_z = -zr - 1;
        let max_z = zr + 1;

        let mut hole_count = 0i32;
        for dx in min_x..=max_x {
            for dy in -1..=4i32 {
                for dz in min_z..=max_z {
                    let check_pos = pos.0.add(&Vector3::new(dx, dy, dz));
                    let solid = GenerationCache::get_block_state(chunk, &check_pos)
                        .to_state()
                        .is_solid();

                    if dy == -1 && !solid {
                        return false;
                    }
                    if dy == 4 && !solid {
                        return false;
                    }

                    let on_wall = dx == min_x || dx == max_x || dz == min_z || dz == max_z;
                    if on_wall
                        && dy == 0
                        && chunk.is_air(&check_pos)
                        && chunk.is_air(&check_pos.add(&Vector3::new(0, 1, 0)))
                    {
                        hole_count += 1;
                    }
                }
            }
        }

        if !(1..=5).contains(&hole_count) {
            return false;
        }

        let cannot_replace = |id: pumpkin_data::BlockId| {
            id.has_tag(pumpkin_data::tag::Block::MINECRAFT_FEATURES_CANNOT_REPLACE)
        };

        for dx in min_x..=max_x {
            for dy in (-1..=3i32).rev() {
                for dz in min_z..=max_z {
                    let wall_pos = pos.0.add(&Vector3::new(dx, dy, dz));

                    let on_boundary = dx == min_x
                        || dy == -1
                        || dz == min_z
                        || dx == max_x
                        || dy == 4
                        || dz == max_z;

                    if on_boundary {
                        let below_pos = wall_pos.add(&Vector3::new(0, -1, 0));
                        let below_solid = GenerationCache::get_block_state(chunk, &below_pos)
                            .to_state()
                            .is_solid();
                        let cur_state =
                            GenerationCache::get_block_state(chunk, &wall_pos).to_state();
                        let cur_id = cur_state.id.to_block_id();
                        if cannot_replace(cur_id) {
                            continue;
                        }
                        let cur_solid = cur_state.is_solid();
                        let is_chest = cur_id == Block::CHEST.id;

                        let world_min_y = chunk.bottom_y() as i32;
                        if wall_pos.y >= world_min_y && !below_solid {
                            chunk.set_block_state(&wall_pos, Block::CAVE_AIR.default_state);
                        } else if cur_solid && !is_chest {
                            if dy == -1 && random.next_bounded_i32(4) != 0 {
                                chunk.set_block_state(
                                    &wall_pos,
                                    Block::MOSSY_COBBLESTONE.default_state,
                                );
                            } else {
                                chunk.set_block_state(&wall_pos, Block::COBBLESTONE.default_state);
                            }
                        }
                    } else {
                        let state = GenerationCache::get_block_state(chunk, &wall_pos).to_state();
                        let cur_id = state.id.to_block_id();
                        if cannot_replace(cur_id)
                            || cur_id == Block::CHEST.id
                            || cur_id == Block::SPAWNER.id
                        {
                            continue;
                        }
                        chunk.set_block_state(&wall_pos, Block::CAVE_AIR.default_state);
                    }
                }
            }
        }

        for _ in 0..2 {
            for _ in 0..3 {
                let cx = pos.0.x + random.next_bounded_i32(xr * 2 + 1) - xr;
                let cy = pos.0.y;
                let cz = pos.0.z + random.next_bounded_i32(zr * 2 + 1) - zr;
                let chest_pos = Vector3::new(cx, cy, cz);

                if !chunk.is_air(&chest_pos) {
                    continue;
                }

                let wall_count = BlockDirection::horizontal()
                    .iter()
                    .filter(|d| {
                        GenerationCache::get_block_state(chunk, &chest_pos.add(&d.to_offset()))
                            .to_state()
                            .is_solid()
                    })
                    .count();

                if wall_count == 1 {
                    let chest_state =
                        crate::generation::structure::structures::StructurePiece::reorient(
                            &chest_pos,
                            Block::CHEST.default_state,
                            |p| GenerationCache::get_block_state(chunk, p),
                        );

                    chunk.set_block_state(&chest_pos, chest_state);

                    let loot_seed = random.next_i64();
                    let mut chest_nbt = NbtCompound::new();
                    chest_nbt.put_string("id", "minecraft:chest".to_string());
                    chest_nbt.put_int("x", chest_pos.x);
                    chest_nbt.put_int("y", chest_pos.y);
                    chest_nbt.put_int("z", chest_pos.z);
                    chest_nbt
                        .put_string("LootTable", "minecraft:chests/simple_dungeon".to_string());
                    chest_nbt.put_long("LootTableSeed", loot_seed);
                    chunk.add_block_entity(&chest_pos, chest_nbt);

                    break;
                }
            }
        }

        let mob = DUNGEON_MOBS[random.next_bounded_i32(DUNGEON_MOBS.len() as i32) as usize];
        chunk.set_block_state(&pos.0, Block::SPAWNER.default_state);

        let mut entity_nbt = NbtCompound::new();
        entity_nbt.put_string("id", "minecraft:mob_spawner".to_string());
        entity_nbt.put_int("x", pos.0.x);
        entity_nbt.put_int("y", pos.0.y);
        entity_nbt.put_int("z", pos.0.z);

        let mut spawn_entry = NbtCompound::new();
        let mut entity_nbt_inner = NbtCompound::new();
        entity_nbt_inner.put_string("id", mob.to_string());
        spawn_entry.put_compound("entity", entity_nbt_inner);
        entity_nbt.put_compound("SpawnData", spawn_entry);

        chunk.add_block_entity(&pos.0, entity_nbt);

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProtoChunk;
    use crate::generation::get_world_gen;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::random::legacy_rand::LegacyRand;
    use pumpkin_util::world_seed::Seed;

    #[test]
    fn dungeon_generates_spawner_and_cobblestone() {
        let world_gen = get_world_gen(
            Seed(42),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..10 {
                    chunk.set_block_state(x, y, z, Block::STONE.default_state);
                }
            }
        }
        chunk.set_block_state(5, 5, 8, Block::CAVE_AIR.default_state);
        chunk.set_block_state(5, 6, 8, Block::CAVE_AIR.default_state);
        chunk.set_block_state(11, 5, 8, Block::CAVE_AIR.default_state);
        chunk.set_block_state(11, 6, 8, Block::CAVE_AIR.default_state);

        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(42));
        let generated = DungeonFeature::generate(
            &mut chunk,
            -64,
            384,
            pumpkin_data::placed_feature::PlacedFeature::MonsterRoom,
            &mut random,
            BlockPos::new(8, 5, 8),
        );
        if generated {
            let spawner = chunk.get_block_state(&Vector3::new(8, 5, 8));
            assert_eq!(spawner.to_block_id(), Block::SPAWNER.id);
        }
    }
}
