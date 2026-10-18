use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

use crate::generation::proto_chunk::GenerationCache;

pub struct DripstoneClusterFeature;

impl DripstoneClusterFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let radius = random.next_inbetween_i32(8, 14);
        let mut generated = false;
        let search_range = 64;

        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let dist_sq = dx * dx + dz * dz;
                if dist_sq > radius * radius {
                    continue;
                }
                let dist = (dist_sq as f32).sqrt() / (radius as f32);
                let x = pos.0.x + dx;
                let z = pos.0.z + dz;

                let Some(surface) = super::find_cave_surface(chunk, x, pos.0.y, z, search_range)
                else {
                    continue;
                };

                let cave_height = surface.ceiling_y - surface.floor_y - 1;
                if cave_height < 2 {
                    continue;
                }

                let chance = 1.0 - dist * 0.7;
                if random.next_f32() > chance {
                    continue;
                }

                generated = true;

                let base_thickness = random.next_inbetween_i32(1, 3);
                for i in 0..base_thickness {
                    super::gen_dripstone(chunk, BlockPos::new(x, surface.ceiling_y + i, z));
                    super::gen_dripstone(chunk, BlockPos::new(x, surface.floor_y - i, z));
                }

                let max_h = ((1.0 - dist) * 8.0).round() as i32 + random.next_bounded_i32(4);
                let mut stalactite_len =
                    (random.next_inbetween_i32(1, max_h.max(2))).min(cave_height);
                let mut stalagmite_len =
                    (random.next_inbetween_i32(1, max_h.max(2))).min(cave_height);
                let merged = stalactite_len + stalagmite_len >= cave_height;
                if merged {
                    let mid_split = random.next_inbetween_i32(1, (cave_height - 1).max(2));
                    stalactite_len = (cave_height - mid_split).max(1);
                    stalagmite_len = mid_split.min(cave_height - stalactite_len).max(1);
                }

                if stalactite_len > 0 {
                    super::grow_pointed_dripstone(
                        chunk,
                        BlockPos::new(x, surface.ceiling_y - 1, z),
                        BlockDirection::Down,
                        stalactite_len,
                        merged,
                    );
                }

                if stalagmite_len > 0 {
                    super::grow_pointed_dripstone(
                        chunk,
                        BlockPos::new(x, surface.floor_y + 1, z),
                        BlockDirection::Up,
                        stalagmite_len,
                        merged,
                    );
                }
            }
        }
        generated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProtoChunk;
    use crate::generation::get_world_gen;
    use pumpkin_data::Block;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::random::legacy_rand::LegacyRand;
    use pumpkin_util::world_seed::Seed;

    #[test]
    fn dripstone_cluster_generates_stalactites_and_stalagmites() {
        let world_gen = get_world_gen(
            Seed(42),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);
        // Create a cave: solid floor (y=0..=5), air cave (y=6..=20), solid ceiling (y=21..=25)
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..=5 {
                    chunk.set_block_state(x, y, z, Block::STONE.default_state);
                }
                for y in 21..=25 {
                    chunk.set_block_state(x, y, z, Block::STONE.default_state);
                }
            }
        }
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(12345));
        let feature = DripstoneClusterFeature;
        let generated = feature.generate(&mut chunk, &mut random, BlockPos::new(8, 12, 8));
        assert!(generated, "dripstone cluster should generate in cave space");

        let mut found_pointed = false;
        let mut found_dripstone_block = false;
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..30 {
                    let block = chunk
                        .get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z))
                        .to_block_id();
                    if block == Block::POINTED_DRIPSTONE.id {
                        found_pointed = true;
                    }
                    if block == Block::DRIPSTONE_BLOCK.id {
                        found_dripstone_block = true;
                    }
                }
            }
        }
        assert!(found_pointed, "should place pointed dripstone");
        assert!(
            found_dripstone_block,
            "should place dripstone block at base"
        );
    }
}
