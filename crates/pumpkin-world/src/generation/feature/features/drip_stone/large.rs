use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use crate::generation::proto_chunk::GenerationCache;

pub struct LargeDripstoneFeature;

impl LargeDripstoneFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let search_range = 64;
        let Some(surface) =
            super::find_cave_surface(chunk, pos.0.x, pos.0.y, pos.0.z, search_range)
        else {
            return false;
        };

        let cave_height = surface.ceiling_y - surface.floor_y - 1;
        if cave_height < 4 {
            return false;
        }

        let scale = random.next_inbetween_i32(2, 4);
        let mut generated = false;

        // Build stalagmite (from floor up)
        for dx in -scale..=scale {
            for dz in -scale..=scale {
                let dist = ((dx * dx + dz * dz) as f32).sqrt();
                if dist > scale as f32 {
                    continue;
                }
                let norm = 1.0 - (dist / scale as f32);
                let col_h = (norm
                    * (cave_height as f32 * 0.45 + random.next_inbetween_i32(1, 3) as f32))
                    .round() as i32;
                if col_h > 0 {
                    let col_x = pos.0.x + dx;
                    let col_z = pos.0.z + dz;
                    for y in 0..col_h {
                        let target = BlockPos::new(col_x, surface.floor_y + y, col_z);
                        chunk.set_block_state(&target.0, Block::DRIPSTONE_BLOCK.default_state);
                        generated = true;
                    }
                    if dx == 0 && dz == 0 && surface.floor_y + col_h < surface.ceiling_y - 1 {
                        super::grow_pointed_dripstone(
                            chunk,
                            BlockPos::new(col_x, surface.floor_y + col_h, col_z),
                            BlockDirection::Up,
                            random.next_inbetween_i32(1, 3),
                            false,
                        );
                    }
                }
            }
        }

        // Build stalactite (from ceiling down)
        for dx in -scale..=scale {
            for dz in -scale..=scale {
                let dist = ((dx * dx + dz * dz) as f32).sqrt();
                if dist > scale as f32 {
                    continue;
                }
                let norm = 1.0 - (dist / scale as f32);
                let col_h = (norm
                    * (cave_height as f32 * 0.45 + random.next_inbetween_i32(1, 3) as f32))
                    .round() as i32;
                if col_h > 0 {
                    let col_x = pos.0.x + dx;
                    let col_z = pos.0.z + dz;
                    for y in 0..col_h {
                        let target = BlockPos::new(col_x, surface.ceiling_y - y, col_z);
                        chunk.set_block_state(&target.0, Block::DRIPSTONE_BLOCK.default_state);
                        generated = true;
                    }
                    if dx == 0 && dz == 0 && surface.ceiling_y - col_h > surface.floor_y + 1 {
                        super::grow_pointed_dripstone(
                            chunk,
                            BlockPos::new(col_x, surface.ceiling_y - col_h, col_z),
                            BlockDirection::Down,
                            random.next_inbetween_i32(1, 3),
                            false,
                        );
                    }
                }
            }
        }

        generated
    }
}
