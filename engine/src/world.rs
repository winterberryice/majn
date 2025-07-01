use std::collections::{HashMap, VecDeque};
use crate::chunk::{Chunk, CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH};
use crate::block::{Block, BlockType}; // Added BlockType back

const MAX_LIGHT: u8 = 15; // Same as in chunk.rs

// Structure to hold information for light updates across chunks
// Made public so chunk.rs can use it for queue types.
pub struct LightNode {
    pub pos: glam::IVec3, // World coordinates
    pub light_level: u8,
}

pub struct World {
    chunks: HashMap<(i32, i32), Chunk>,
    sky_light_queue: VecDeque<LightNode>, // For BFS spreading skylight
    block_light_queue: VecDeque<LightNode>, // For BFS spreading block light
    sky_light_removal_queue: VecDeque<LightNode>, // For BFS removing skylight
    block_light_removal_queue: VecDeque<LightNode>, // For BFS removing block light
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

    // Gets a reference to a chunk if it exists, otherwise generates/loads it.
    // Now also calculates initial light for new chunks.
    pub fn get_or_create_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &mut Chunk {
        self.chunks
            .entry((chunk_x, chunk_z))
            .or_insert_with(|| {
                let mut new_chunk = Chunk::new(chunk_x, chunk_z);
                new_chunk.generate_terrain();
                // Initialize light values after terrain generation
                new_chunk.calculate_initial_skylight();
                new_chunk.initialize_block_light();
                // After basic initialization, add all light sources to the world's propagation queue
                for x_in_chunk in 0..CHUNK_WIDTH {
                    for y_in_chunk in 0..CHUNK_HEIGHT {
                        for z_in_chunk in 0..CHUNK_DEPTH {
                            let block = new_chunk.get_block(x_in_chunk, y_in_chunk, z_in_chunk).unwrap(); // Safe within bounds
                            let world_x = chunk_x * CHUNK_WIDTH as i32 + x_in_chunk as i32;
                            let world_y = y_in_chunk as i32;
                            let world_z = chunk_z * CHUNK_DEPTH as i32 + z_in_chunk as i32;

                            if block.sky_light_level > 0 {
                                self.sky_light_queue.push_back(LightNode {
                                    pos: glam::ivec3(world_x, world_y, world_z),
                                    light_level: block.sky_light_level,
                                });
                            }
                            if block.block_light_level > 0 {
                                self.block_light_queue.push_back(LightNode {
                                    pos: glam::ivec3(world_x, world_y, world_z),
                                    light_level: block.block_light_level,
                                });
                            }
                        }
                    }
                }
                // The actual propagation will be handled by a separate world update step
                new_chunk
            })
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        self.chunks.get(&(chunk_x, chunk_z))
    }

    // Gets a mutable reference to a chunk.
    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(chunk_x, chunk_z))
    }

    // Converts world block coordinates to chunk coordinates and local block coordinates.
    // Now takes IVec3 for world coordinates.
    pub fn world_to_chunk_coords_ivec3(world_pos: glam::IVec3) -> ((i32, i32), (usize, usize, usize)) {
        let chunk_x = (world_pos.x as f32 / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_z = (world_pos.z as f32 / CHUNK_DEPTH as f32).floor() as i32;

        let local_x = ((world_pos.x % CHUNK_WIDTH as i32) + CHUNK_WIDTH as i32) % CHUNK_WIDTH as i32;
        let local_y = world_pos.y; // y is absolute within chunk height range
        let local_z = ((world_pos.z % CHUNK_DEPTH as i32) + CHUNK_DEPTH as i32) % CHUNK_DEPTH as i32;

        // Assuming world_pos.y is already validated or will be by the caller to be within [0, CHUNK_HEIGHT)
        ((chunk_x, chunk_z), (local_x as usize, local_y as usize, local_z as usize))
    }


    // Gets a block at absolute world coordinates.
    pub fn get_block_at_world(&self, world_pos: glam::IVec3) -> Option<&Block> {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return None;
        }
        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) = World::world_to_chunk_coords_ivec3(world_pos);

        match self.get_chunk(chunk_x, chunk_z) {
            Some(chunk) => chunk.get_block(local_x, local_y, local_z),
            None => None,
        }
    }

    // Gets a mutable block at absolute world coordinates.
    pub fn get_block_at_world_mut(&mut self, world_pos: glam::IVec3) -> Option<&mut Block> {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return None;
        }
        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) = World::world_to_chunk_coords_ivec3(world_pos);

        match self.get_chunk_mut(chunk_x, chunk_z) {
            Some(chunk) => chunk.get_block_mut(local_x, local_y, local_z),
            None => None,
        }
    }


    pub fn set_block(&mut self, world_block_pos: glam::IVec3, block_type: BlockType) -> Result<(i32, i32), &'static str> {
        if world_block_pos.y < 0 || world_block_pos.y >= CHUNK_HEIGHT as i32 {
            return Err("Y coordinate out of world bounds");
        }

        let old_block_opt = self.get_block_at_world(world_block_pos).copied(); // Copy if exists

        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) =
            World::world_to_chunk_coords_ivec3(world_block_pos);

        if let Some(chunk) = self.get_chunk(chunk_x, chunk_z) {
            if let Some(existing_block) = chunk.get_block(local_x, local_y, local_z) {
                if existing_block.block_type == BlockType::Bedrock && block_type != BlockType::Bedrock {
                    return Err("Cannot replace Bedrock");
                }
            }
        }

        // Perform the set_block operation
        let chunk = self.get_or_create_chunk(chunk_x, chunk_z);
        chunk.set_block(local_x, local_y, local_z, block_type)?; // Propagate error using ?

        // --- Light update logic after setting a block ---
        let new_block = chunk.get_block(local_x, local_y, local_z).unwrap(); // Must exist now

        // 1. Handle removal of old light
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
            // Also add neighbors of the old block to the main propagation queue,
            // as their light might need to spread *into* the newly changed block
            // or be re-evaluated if the changed block became more transparent.
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dy == 0 && dz == 0 { continue; }
                        let neighbor_pos = world_block_pos + glam::ivec3(dx, dy, dz);
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

        // 2. Handle new light source / new skylight exposure
        // If the new block is a light emitter, add it to the block light propagation queue.
        if new_block.emission() > 0 {
            // The block's own light level is set in chunk.set_block
            self.block_light_queue.push_back(LightNode {
                pos: world_block_pos,
                light_level: new_block.emission(),
            });
        }

        // If the block became transparent and is exposed to the sky, it might need skylight recalc.
        // This is complex. For now, assume calculate_initial_skylight in chunk and then world propagation handles it.
        // If the block is now AIR or transparent, and was opaque, skylight might flow in.
        // The safest is to re-calculate skylight for this column if an opaque block was removed.
        if new_block.opacity() < MAX_LIGHT && old_block_opt.map_or(false, |b| b.opacity() >= MAX_LIGHT) {
            // An opaque block was removed/made transparent. Recalculate skylight for this column.
            let ((update_chunk_x, update_chunk_z), (lx, _ly, lz)) = World::world_to_chunk_coords_ivec3(world_block_pos);
            if let Some(update_chunk) = self.get_chunk_mut(update_chunk_x, update_chunk_z) {
                // Simplified: just add affected block to sky queue, actual column recalc is harder here
                // update_chunk.recalculate_skylight_for_column(lx, lz); // This function would be in Chunk
                // For now, just add the current block to the sky queue if it's exposed.
                // This will be refined.
                // A robust way: if block became transparent, check block above. If it has skylight, propagate from there.
                let pos_above = world_block_pos + glam::ivec3(0, 1, 0);
                if let Some(block_above) = self.get_block_at_world(pos_above) {
                    if block_above.sky_light_level > 0 {
                         self.sky_light_queue.push_back(LightNode{pos: world_block_pos, light_level: block_above.sky_light_level});
                    }
                } else { // Block above is outside world or in unloaded chunk, assume max skylight if at top
                    if world_block_pos.y == CHUNK_HEIGHT as i32 -1 {
                         self.sky_light_queue.push_back(LightNode{pos: world_block_pos, light_level: MAX_LIGHT});
                    }
                }
            }
        }

        // Ensure the changed block itself is in the queue if it has light
        if new_block.sky_light_level > 0 {
             self.sky_light_queue.push_back(LightNode{pos: world_block_pos, light_level: new_block.sky_light_level});
        }
         if new_block.block_light_level > 0 { // This includes emission
             self.block_light_queue.push_back(LightNode{pos: world_block_pos, light_level: new_block.block_light_level});
        }


        // The actual processing of these queues happens in `propagate_all_light`.
        Ok((chunk_x, chunk_z))
    }

    // Processes the light propagation and removal queues.
    pub fn propagate_all_light(&mut self) {
        // Process removal queues first
        self.process_light_removal_queue(true); // Skylight
        self.process_light_removal_queue(false); // Block light

        // Process propagation queues
        self.process_light_propagation_queue(true); // Skylight
        self.process_light_propagation_queue(false); // Block light
    }

    fn process_light_removal_queue(&mut self, is_sky_light: bool) {
        let queue = if is_sky_light {
            &mut self.sky_light_removal_queue
        } else {
            &mut self.block_light_removal_queue
        };

        let mut temp_propagate_queue = VecDeque::new(); // To re-propagate light from sources that were dimmed

        while let Some(node) = queue.pop_front() {
            // For each neighbor of the node
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dy == 0 && dz == 0 { continue; }
                        // Cardinal only for simplicity for now
                        if (dx.abs() + dy.abs() + dz.abs()) > 1 { continue; }


                        let neighbor_pos = node.pos + glam::ivec3(dx, dy, dz);

                        if let Some(neighbor_block) = self.get_block_at_world_mut(neighbor_pos) {
                            let current_neighbor_light = if is_sky_light {
                                neighbor_block.sky_light_level
                            } else {
                                neighbor_block.block_light_level
                            };

                            if current_neighbor_light == 0 { continue; }

                            // If neighbor's light was potentially sourced from 'node'
                            // The light level of 'node' *before* it was set to 0 (node.light_level)
                            // determined the neighbor's light.
                            // Simplified: if neighbor is not an emitter itself and its light is less than node's old light
                            let opacity = neighbor_block.opacity(); // Opacity of the medium light is *in* (neighbor)
                            let reduction = if is_sky_light && dy == -1 && opacity == 0 && node.light_level == MAX_LIGHT { 0 } else { 1 + opacity };

                            if current_neighbor_light < node.light_level.saturating_sub(reduction) || current_neighbor_light == node.light_level.saturating_sub(reduction) {
                                // This neighbor's light might have come from 'node'.
                                // Set its light to 0 and add to removal queue.
                                if is_sky_light {
                                    neighbor_block.sky_light_level = 0;
                                } else {
                                    neighbor_block.block_light_level = 0;
                                }
                                queue.push_back(LightNode {
                                    pos: neighbor_pos,
                                    light_level: current_neighbor_light, // Pass its old light level
                                });
                            } else if current_neighbor_light > 0 {
                                // This neighbor has light, but it wasn't from 'node' (or it's stronger).
                                // It might need to re-propagate its light. Add to main propagation queue.
                                temp_propagate_queue.push_back(LightNode {
                                    pos: neighbor_pos,
                                    light_level: current_neighbor_light,
                                });
                            }
                        }
                    }
                }
            }
        }
        // Add collected nodes that need re-propagation back to the main propagation queues
        let main_prop_queue = if is_sky_light { &mut self.sky_light_queue } else { &mut self.block_light_queue };
        main_prop_queue.extend(temp_propagate_queue);
    }


    fn process_light_propagation_queue(&mut self, is_sky_light: bool) {
        let queue = if is_sky_light {
            &mut self.sky_light_queue
        } else {
            &mut self.block_light_queue
        };

        while let Some(node) = queue.pop_front() {
            // Current light at node.pos might have changed due to other paths, so re-fetch
            let source_light = match self.get_block_at_world(node.pos) {
                Some(b) => if is_sky_light { b.sky_light_level } else { b.block_light_level },
                None => continue, // Block unloaded or out of bounds
            };

            // If the queued light_level is weaker than what's actually at the source block now,
            // it means a stronger path already processed this or the source itself was dimmed.
            // However, the original node.light_level was the value *when it was added*.
            // We should propagate based on the actual current light of the source block.
            if source_light == 0 { continue; }


            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dy == 0 && dz == 0 { continue; }
                        // Cardinal only
                         if (dx.abs() + dy.abs() + dz.abs()) > 1 { continue; }

                        let neighbor_pos = node.pos + glam::ivec3(dx, dy, dz);

                        if neighbor_pos.y < 0 || neighbor_pos.y >= CHUNK_HEIGHT as i32 {
                            continue; // Neighbor is out of vertical bounds
                        }

                        // We need mutable access to the neighbor to set its light
                        if let Some(neighbor_block) = self.get_block_at_world_mut(neighbor_pos) {
                            let opacity = neighbor_block.opacity();
                            if opacity >= MAX_LIGHT && !(is_sky_light && neighbor_pos.y < node.pos.y && source_light == MAX_LIGHT) { // Allow skylight to enter top of opaque block
                                continue; // Light cannot enter fully opaque blocks (unless it's skylight initial pass)
                            }

                            let reduction = if is_sky_light {
                                // Skylight specific reduction
                                if dy == -1 && opacity == 0 && source_light == MAX_LIGHT { // Traveling straight down in transparent medium from full sky
                                    0
                                } else {
                                    1u8.saturating_add(opacity) // Standard spread reduction
                                }
                            } else {
                                // Block light reduction
                                1u8.saturating_add(opacity)
                            };

                            let new_light_level = source_light.saturating_sub(reduction);

                            let current_neighbor_light = if is_sky_light {
                                neighbor_block.sky_light_level
                            } else {
                                neighbor_block.block_light_level
                            };

                            if new_light_level > current_neighbor_light {
                                if is_sky_light {
                                    neighbor_block.sky_light_level = new_light_level;
                                } else {
                                    neighbor_block.block_light_level = new_light_level;
                                }
                                // Add to queue only if it's strong enough to propagate further
                                if new_light_level > 1 { // Optimization: light of 1 cannot light up neighbors further
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
        // Need to collect chunk coordinates first because we can't borrow self.chunks mutably
        // while also passing mutable borrows of the queues to chunk methods.
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
        // After this, State::update() should call self.world.propagate_all_light()
        // which is already happening in the current State::update logic.
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
