use std::collections::{HashMap, VecDeque};
use crate::chunk::{Chunk, CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH, MAX_LIGHT_LEVEL as CHUNK_MAX_LIGHT};
use crate::block::{Block, BlockType, MAX_LIGHT_LEVEL};

#[derive(Debug, Clone, Copy)]
pub struct LightNode {
    pub pos: glam::IVec3,
    pub light_level: u8,
}

pub struct World {
    chunks: HashMap<(i32, i32), Chunk>,
    sky_light_queue: VecDeque<LightNode>,
    block_light_queue: VecDeque<LightNode>,
    sky_light_removal_queue: VecDeque<LightNode>,
    block_light_removal_queue: VecDeque<LightNode>,
}

impl World {
    pub fn new() -> Self {
        World {
            chunks: HashMap::new(),
            sky_light_queue: VecDeque::new(),
            block_light_queue: VecDeque::new(),
            sky_light_removal_queue: VecDeque::new(),
            block_light_removal_queue: VecDeque::new(),
        }
    }

    pub fn get_or_create_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &mut Chunk {
        self.chunks
            .entry((chunk_x, chunk_z))
            .or_insert_with(|| {
                let mut new_chunk = Chunk::new(chunk_x, chunk_z);
                new_chunk.generate_terrain();
                new_chunk.calculate_initial_skylight();
                new_chunk.initialize_block_light();

                let world_chunk_x_offset = chunk_x * CHUNK_WIDTH as i32;
                let world_chunk_z_offset = chunk_z * CHUNK_DEPTH as i32;

                for x_in_chunk in 0..CHUNK_WIDTH {
                    for y_in_chunk in 0..CHUNK_HEIGHT {
                        for z_in_chunk in 0..CHUNK_DEPTH {
                            let block = new_chunk.get_block(x_in_chunk, y_in_chunk, z_in_chunk).unwrap();
                            let world_pos = glam::ivec3(
                                world_chunk_x_offset + x_in_chunk as i32,
                                y_in_chunk as i32,
                                world_chunk_z_offset + z_in_chunk as i32,
                            );

                            if block.sky_light_level > 0 {
                                self.sky_light_queue.push_back(LightNode {
                                    pos: world_pos,
                                    light_level: block.sky_light_level,
                                });
                            }
                            if block.block_light_level > 0 {
                                self.block_light_queue.push_back(LightNode {
                                    pos: world_pos,
                                    light_level: block.block_light_level,
                                });
                            }
                        }
                    }
                }
                new_chunk
            })
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        self.chunks.get(&(chunk_x, chunk_z))
    }

    // pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut Chunk> {
    //     self.chunks.get_mut(&(chunk_x, chunk_z))
    // }

    pub fn world_to_chunk_coords_ivec3(world_pos: glam::IVec3) -> ((i32, i32), (usize, usize, usize)) {
        let chunk_x = (world_pos.x as f32 / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_z = (world_pos.z as f32 / CHUNK_DEPTH as f32).floor() as i32;

        let local_x = ((world_pos.x % CHUNK_WIDTH as i32) + CHUNK_WIDTH as i32) % CHUNK_WIDTH as i32;
        let local_y = world_pos.y;
        let local_z = ((world_pos.z % CHUNK_DEPTH as i32) + CHUNK_DEPTH as i32) % CHUNK_DEPTH as i32;

        ((chunk_x, chunk_z), (local_x as usize, local_y as usize, local_z as usize))
    }

    pub fn get_block_at_world(&self, world_pos: glam::IVec3) -> Option<&Block> {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return None;
        }
        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) = World::world_to_chunk_coords_ivec3(world_pos);

        match self.chunks.get(&(chunk_x, chunk_z)) {
            Some(chunk) => chunk.get_block(local_x, local_y, local_z),
            None => None,
        }
    }

    pub fn get_block_at_world_mut(&mut self, world_pos: glam::IVec3) -> Option<&mut Block> {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return None;
        }
        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) = World::world_to_chunk_coords_ivec3(world_pos);

        match self.chunks.get_mut(&(chunk_x, chunk_z)) {
            Some(chunk) => chunk.get_block_mut(local_x, local_y, local_z),
            None => None,
        }
    }

    pub fn set_block(&mut self, world_block_pos: glam::IVec3, block_type: BlockType) -> Result<(i32, i32), &'static str> {
        if world_block_pos.y < 0 || world_block_pos.y >= CHUNK_HEIGHT as i32 {
            return Err("Y coordinate out of world bounds");
        }

        let old_block_opt = self.get_block_at_world(world_block_pos).copied();

        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) =
            World::world_to_chunk_coords_ivec3(world_block_pos);

        if let Some(current_chunk_ref) = self.chunks.get(&(chunk_x, chunk_z)) {
            if let Some(existing_block) = current_chunk_ref.get_block(local_x, local_y, local_z) {
                if existing_block.block_type == BlockType::Bedrock && block_type != BlockType::Bedrock {
                    return Err("Cannot replace Bedrock");
                }
            }
        }

        let (new_block_emission, new_block_opacity, new_block_sky_light, new_block_block_light) = {
            let chunk = self.chunks.entry((chunk_x, chunk_z))
                .or_insert_with(|| {
                    let mut new_chunk = Chunk::new(chunk_x, chunk_z);
                    new_chunk.generate_terrain();
                    new_chunk.calculate_initial_skylight();
                    new_chunk.initialize_block_light();
                    new_chunk
                });

            chunk.set_block(local_x, local_y, local_z, block_type)?;

            let new_block_ref = chunk.get_block(local_x, local_y, local_z)
                .ok_or("Block not found after setting (internal error in set_block)")?;
            (
                new_block_ref.emission(),
                new_block_ref.opacity(),
                new_block_ref.sky_light_level,
                new_block_ref.block_light_level,
            )
        };

        if let Some(old_block) = old_block_opt {
            if old_block.sky_light_level > 0 {
                self.sky_light_removal_queue.push_back(LightNode {
                    pos: world_block_pos,
                    light_level: old_block.sky_light_level,
                });
            }
            if old_block.block_light_level > 0 {
                 self.block_light_removal_queue.push_back(LightNode {
                    pos: world_block_pos,
                    light_level: old_block.block_light_level,
                });
            }
            for dx_i32 in -1..=1_i32 {
                for dy_i32 in -1..=1_i32 {
                    for dz_i32 in -1..=1_i32 {
                        if dx_i32 == 0 && dy_i32 == 0 && dz_i32 == 0 { continue; }
                        let neighbor_pos = world_block_pos + glam::ivec3(dx_i32, dy_i32, dz_i32);
                        if let Some(nb) = self.get_block_at_world(neighbor_pos) {
                            if nb.sky_light_level > 0 {
                                self.sky_light_queue.push_back(LightNode { pos: neighbor_pos, light_level: nb.sky_light_level });
                            }
                            if nb.block_light_level > 0 {
                                self.block_light_queue.push_back(LightNode { pos: neighbor_pos, light_level: nb.block_light_level });
                            }
                        }
                    }
                }
            }
        }

        if new_block_emission > 0 {
            self.block_light_queue.push_back(LightNode {
                pos: world_block_pos,
                light_level: new_block_block_light,
            });
        }

        if new_block_opacity < MAX_LIGHT_LEVEL && old_block_opt.map_or(true, |b| b.opacity() >= MAX_LIGHT_LEVEL) {
            let pos_above = world_block_pos + glam::ivec3(0, 1, 0);
            let mut light_from_above = 0;
            if let Some(block_above) = self.get_block_at_world(pos_above) {
                light_from_above = block_above.sky_light_level;
            } else if world_block_pos.y == CHUNK_HEIGHT as i32 - 1 {
                light_from_above = MAX_LIGHT_LEVEL;
            }

            if light_from_above > 0 {
                let effective_light_for_new_block = if new_block_opacity == 0 { light_from_above } else { new_block_sky_light };
                if effective_light_for_new_block > 0 {
                    if let Some(current_block_mut) = self.get_block_at_world_mut(world_block_pos) {
                        if current_block_mut.sky_light_level < effective_light_for_new_block {
                             current_block_mut.sky_light_level = effective_light_for_new_block;
                        }
                    }
                    self.sky_light_queue.push_back(LightNode {
                        pos: world_block_pos,
                        light_level: effective_light_for_new_block,
                    });
                }
            }
        }

        if new_block_sky_light > 0 {
             self.sky_light_queue.push_back(LightNode{pos: world_block_pos, light_level: new_block_sky_light});
        }
        if new_block_block_light > 0 && new_block_emission == 0 {
             self.block_light_queue.push_back(LightNode{pos: world_block_pos, light_level: new_block_block_light});
        }

        let ((_update_chunk_x, _update_chunk_z), (_lx, _ly, _lz)) = World::world_to_chunk_coords_ivec3(world_block_pos);
        Ok((chunk_x, chunk_z))
    }

    pub fn propagate_all_light(&mut self) {
        self.process_light_removal_queue_typed(true);
        self.process_light_removal_queue_typed(false);
        self.process_light_propagation_queue_typed(true);
        self.process_light_propagation_queue_typed(false);
    }

    fn process_light_removal_queue_typed(&mut self, is_sky_light: bool) {
        let queue_ref = if is_sky_light {
            &mut self.sky_light_removal_queue
        } else {
            &mut self.block_light_removal_queue
        };
        let mut temp_propagate_queue = VecDeque::new();

        while let Some(node) = queue_ref.pop_front() {
            for dx_i32 in -1..=1_i32 { // Explicitly typed
                for dy_i32 in -1..=1_i32 { // Explicitly typed
                    for dz_i32 in -1..=1_i32 { // Explicitly typed
                        if dx_i32 == 0 && dy_i32 == 0 && dz_i32 == 0 { continue; }
                        if (dx_i32.abs() + dy_i32.abs() + dz_i32.abs()) > 1 { continue; } // Cardinal only

                        let neighbor_pos = node.pos + glam::ivec3(dx_i32, dy_i32, dz_i32);

                        if neighbor_pos.y < 0 || neighbor_pos.y >= CHUNK_HEIGHT as i32 {
                            continue;
                        }

                        let (neighbor_old_light, neighbor_opacity_opt): (u8, Option<u8>) = {
                             if let Some(neighbor_block_ref) = self.get_block_at_world(neighbor_pos){
                                (if is_sky_light { neighbor_block_ref.sky_light_level } else { neighbor_block_ref.block_light_level }, Some(neighbor_block_ref.opacity()))
                             } else {
                                 (0, None)
                             }
                        };

                        if neighbor_old_light == 0 || neighbor_opacity_opt.is_none() { continue; }
                        let neighbor_opacity = neighbor_opacity_opt.unwrap();

                        let opacity_of_medium_light_was_in =
                                self.get_block_at_world(node.pos).map_or(CHUNK_MAX_LIGHT, |b| b.opacity());

                        let reduction_into_neighbor = if is_sky_light {
                            if dy_i32 == -1 && neighbor_opacity == 0 && node.light_level == MAX_LIGHT_LEVEL { 0 }
                            else { 1u8.saturating_add(neighbor_opacity) }
                        } else {
                            1u8.saturating_add(neighbor_opacity)
                        };

                        // This was the original problematic line for removal logic.
                        // Corrected logic: if neighbor's light *could have come from* node's old light value.
                        // The light neighbor_pos would have received from node.pos (before node.pos changed) is `node.light_level - reduction_from_node_to_neighbor`
                        // where reduction_from_node_to_neighbor is based on the properties of the medium the light *was* in at node.pos
                        // and the medium it *entered* at neighbor_pos.
                        // For simplicity, let's use reduction based on the neighbor's opacity.
                        let light_neighbor_would_get_from_node_old_state = node.light_level.saturating_sub(reduction_into_neighbor);


                        if neighbor_old_light <= light_neighbor_would_get_from_node_old_state {
                            if let Some(nb_mut) = self.get_block_at_world_mut(neighbor_pos) {
                                if is_sky_light {
                                    nb_mut.sky_light_level = 0;
                                } else {
                                    nb_mut.block_light_level = 0;
                                }
                                queue_ref.push_back(LightNode {
                                    pos: neighbor_pos,
                                    light_level: neighbor_old_light,
                                });
                            }
                        } else {
                            temp_propagate_queue.push_back(LightNode {
                                pos: neighbor_pos,
                                light_level: neighbor_old_light,
                            });
                        }
                    }
                }
            }
        }
        let main_prop_queue = if is_sky_light { &mut self.sky_light_queue } else { &mut self.block_light_queue };
        main_prop_queue.extend(temp_propagate_queue);
    }

    fn process_light_propagation_queue_typed(&mut self, is_sky_light: bool) {
        let queue = if is_sky_light {
            &mut self.sky_light_queue
        } else {
            &mut self.block_light_queue
        };

        while let Some(node) = queue.pop_front() {
            let source_light = match self.get_block_at_world(node.pos) {
                Some(b) => if is_sky_light { b.sky_light_level } else { b.block_light_level },
                None => continue,
            };
            if source_light == 0 { continue; }

            for dx_i32 in -1..=1_i32 { // Explicitly typed
                for dy_i32 in -1..=1_i32 { // Explicitly typed
                    for dz_i32 in -1..=1_i32 { // Explicitly typed
                        if dx_i32 == 0 && dy_i32 == 0 && dz_i32 == 0 { continue; }
                        if (dx_i32.abs() + dy_i32.abs() + dz_i32.abs()) > 1 { continue; } // Cardinal only

                        let neighbor_pos = node.pos + glam::ivec3(dx_i32, dy_i32, dz_i32);

                        if neighbor_pos.y < 0 || neighbor_pos.y >= CHUNK_HEIGHT as i32 {
                            continue;
                        }

                        if let Some(neighbor_block_mut) = self.get_block_at_world_mut(neighbor_pos) {
                            let opacity_of_neighbor = neighbor_block_mut.opacity();
                            if opacity_of_neighbor >= MAX_LIGHT_LEVEL &&
                               !(is_sky_light && dy_i32 == -1 && source_light == MAX_LIGHT_LEVEL && neighbor_block_mut.sky_light_level == 0) {
                                if !(is_sky_light && dy_i32 == -1 && source_light == MAX_LIGHT_LEVEL && neighbor_block_mut.sky_light_level == 0 && opacity_of_neighbor >= MAX_LIGHT_LEVEL) {
                                   continue;
                                }
                            }

                            let reduction = if is_sky_light {
                                if dy_i32 == -1 && opacity_of_neighbor == 0 && source_light == MAX_LIGHT_LEVEL {
                                    0
                                } else {
                                    if opacity_of_neighbor >= MAX_LIGHT_LEVEL { MAX_LIGHT_LEVEL.saturating_add(1) } else { opacity_of_neighbor.saturating_add(1) }
                                }
                            } else {
                                1u8.saturating_add(opacity_of_neighbor)
                            };

                            let new_light_level = source_light.saturating_sub(reduction);

                            let current_neighbor_light_val = if is_sky_light {
                                neighbor_block_mut.sky_light_level
                            } else {
                                neighbor_block_mut.block_light_level
                            };

                            if new_light_level > current_neighbor_light_val {
                                if is_sky_light {
                                    neighbor_block_mut.sky_light_level = new_light_level;
                                } else {
                                    neighbor_block_mut.block_light_level = new_light_level;
                                }
                                if new_light_level > 1 || (is_sky_light && new_light_level > 0 && dy_i32 == -1 && opacity_of_neighbor == 0) {
                                     queue.push_back(LightNode {
                                        pos: neighbor_pos,
                                        light_level: new_light_level,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn update_global_skylight(&mut self, new_global_skylight_level: u8) {
        let chunk_coords: Vec<(i32, i32)> = self.chunks.keys().copied().collect();

        for (cx, cz) in chunk_coords {
            if let Some(chunk) = self.chunks.get_mut(&(cx, cz)) {
                let chunk_world_x_offset = cx * CHUNK_WIDTH as i32;
                let chunk_world_z_offset = cz * CHUNK_DEPTH as i32;

                chunk.recalculate_skylight_based_on_global(
                    new_global_skylight_level,
                    &mut self.sky_light_removal_queue,
                    &mut self.sky_light_queue,
                    chunk_world_x_offset,
                    chunk_world_z_offset,
                );
            }
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
