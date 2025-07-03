use crate::chunk::{CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH}; // Chunk import removed as it's not directly used.
use crate::world::World;
// BlockType and Block are not directly used here, block properties are accessed via world/chunk methods.
// use crate::block::{BlockType, Block};
use glam::IVec3;
use std::collections::VecDeque;

const MAX_LIGHT_LEVEL: u8 = 15;

// Node for light propagation BFS queue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LightNode {
    chunk_coord: (i32, i32), // Chunk coordinates of the node
    local_pos: IVec3,        // Local coordinates within the chunk
    light_level: u8,
}

pub fn propagate_sky_light(world: &mut World, chunk_cx: i32, chunk_cz: i32) {
    let mut light_queue: VecDeque<LightNode> = VecDeque::new();

    // Step 1: Initialize sky light sources for the target chunk
    // This part assumes that if a block is at CHUNK_HEIGHT - 1 and transparent, it's a sky source.
    // More robust sky detection would check blocks above in the world, potentially across chunks if at boundary.
    if let Some(chunk) = world.get_chunk_mut(chunk_cx, chunk_cz) {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                for y_rev in 0..CHUNK_HEIGHT {
                    let y = CHUNK_HEIGHT - 1 - y_rev;
                    let block = chunk.get_block(x, y, z);
                    let is_opaque = block.map_or(true, |b| !b.is_transparent());

                    if !is_opaque { // If transparent (or Air)
                        if y == CHUNK_HEIGHT - 1 { // Topmost block in chunk, potential sky source
                            chunk.set_sky_light(x, y, z, MAX_LIGHT_LEVEL);
                            light_queue.push_back(LightNode {
                                chunk_coord: (chunk_cx, chunk_cz),
                                local_pos: IVec3::new(x as i32, y as i32, z as i32),
                                light_level: MAX_LIGHT_LEVEL,
                            });
                        }
                    } else { // Opaque block, blocks sky light for this column in this chunk
                        break; // No more sky light can pass down this column from within this chunk
                    }
                }
            }
        }
    } else {
        // Chunk not found, cannot propagate sky light for it.
        return;
    }

    // Step 2: BFS Propagation
    // Directions: N, S, E, W, Down. Sky light doesn't spread upwards.
    let mut directions = Vec::new();
    directions.push(IVec3::new(0, -1, 0)); // Down
    directions.push(IVec3::new(1, 0, 0));  // East
    directions.push(IVec3::new(-1, 0, 0)); // West
    directions.push(IVec3::new(0, 0, 1));  // South
    directions.push(IVec3::new(0, 0, -1)); // North


    while let Some(node) = light_queue.pop_front() {
        for dir in &directions {
            let neighbor_local_pos = node.local_pos + *dir;
            let mut neighbor_chunk_coord = node.chunk_coord;

            // Handle cross-chunk boundaries
            let mut new_local_x = neighbor_local_pos.x;
            let new_local_y = neighbor_local_pos.y; // Made non-mutable as it's not changed before use in this scope
            let mut new_local_z = neighbor_local_pos.z;

            if new_local_x < 0 {
                neighbor_chunk_coord.0 -= 1;
                new_local_x = CHUNK_WIDTH as i32 - 1;
            } else if new_local_x >= CHUNK_WIDTH as i32 {
                neighbor_chunk_coord.0 += 1;
                new_local_x = 0;
            }
            if new_local_z < 0 {
                neighbor_chunk_coord.1 -= 1;
                new_local_z = CHUNK_DEPTH as i32 - 1;
            } else if new_local_z >= CHUNK_DEPTH as i32 {
                neighbor_chunk_coord.1 += 1;
                new_local_z = 0;
            }

            // Ensure Y is within bounds (0 to CHUNK_HEIGHT -1)
            if new_local_y < 0 || new_local_y >= CHUNK_HEIGHT as i32 {
                continue;
            }

            let target_light_level = node.light_level - 1;
            if target_light_level == 0 && node.light_level <= 1 { // Optimization: don't propagate 0 light unless it's from a source of 1
                 continue;
            }


            if let Some(neighbor_chunk) = world.get_chunk_mut(neighbor_chunk_coord.0, neighbor_chunk_coord.1) {
                let block_at_neighbor = neighbor_chunk.get_block(new_local_x as usize, new_local_y as usize, new_local_z as usize);

                // Light cannot pass through opaque blocks
                if block_at_neighbor.map_or(true, |b| !b.is_transparent()) {
                    continue;
                }

                let current_neighbor_sky_light = neighbor_chunk.get_sky_light(new_local_x as usize, new_local_y as usize, new_local_z as usize);

                if target_light_level > current_neighbor_sky_light {
                    neighbor_chunk.set_sky_light(new_local_x as usize, new_local_y as usize, new_local_z as usize, target_light_level);
                    if target_light_level > 1 { // Only add to queue if it can propagate further
                        light_queue.push_back(LightNode {
                            chunk_coord: neighbor_chunk_coord,
                            local_pos: IVec3::new(new_local_x, new_local_y, new_local_z),
                            light_level: target_light_level,
                        });
                    }
                }
            }
        }
    }
}

pub fn propagate_block_light(world: &mut World, source_chunk_coord: (i32, i32), source_local_pos: IVec3, emission_strength: u8) {
    if emission_strength == 0 {
        return;
    }

    let mut light_queue: VecDeque<LightNode> = VecDeque::new();

    // Initial source block
    if let Some(chunk) = world.get_chunk_mut(source_chunk_coord.0, source_chunk_coord.1) {
        // The source block itself gets the full emission strength for its block_light value.
        // This happens even if the block itself is opaque, as it's the source.
        let current_block_light = chunk.get_block_light(source_local_pos.x as usize, source_local_pos.y as usize, source_local_pos.z as usize);
        if emission_strength > current_block_light {
            chunk.set_block_light(source_local_pos.x as usize, source_local_pos.y as usize, source_local_pos.z as usize, emission_strength);
            // Add to queue only if it can propagate (strength > 1)
            if emission_strength > 1 {
                 light_queue.push_back(LightNode {
                    chunk_coord: source_chunk_coord,
                    local_pos: source_local_pos,
                    light_level: emission_strength,
                });
            }
        } else {
            // If the current light is already stronger or equal, no need to propagate from this source with this strength.
            return;
        }
    } else {
        // Source chunk not found.
        return;
    }

    // BFS Propagation for block light (all 6 directions)
    let directions = [
        IVec3::new(0, 1, 0),  // Up
        IVec3::new(0, -1, 0), // Down
        IVec3::new(1, 0, 0),  // East
        IVec3::new(-1, 0, 0), // West
        IVec3::new(0, 0, 1),  // South
        IVec3::new(0, 0, -1), // North
    ];

    while let Some(node) = light_queue.pop_front() {
        for dir in &directions {
            let neighbor_local_pos = node.local_pos + *dir;
            let mut neighbor_chunk_coord = node.chunk_coord;

            let mut new_local_x = neighbor_local_pos.x;
            let new_local_y = neighbor_local_pos.y; // Made non-mutable
            let mut new_local_z = neighbor_local_pos.z;

            // Boundary checks and transitions for X
            if new_local_x < 0 {
                neighbor_chunk_coord.0 -= 1;
                new_local_x = CHUNK_WIDTH as i32 - 1;
            } else if new_local_x >= CHUNK_WIDTH as i32 {
                neighbor_chunk_coord.0 += 1;
                new_local_x = 0;
            }

            // Boundary checks and transitions for Z
            if new_local_z < 0 {
                neighbor_chunk_coord.1 -= 1;
                new_local_z = CHUNK_DEPTH as i32 - 1;
            } else if new_local_z >= CHUNK_DEPTH as i32 {
                neighbor_chunk_coord.1 += 1;
                new_local_z = 0;
            }

            // Boundary checks for Y (no chunk transition for Y)
            if new_local_y < 0 || new_local_y >= CHUNK_HEIGHT as i32 {
                continue;
            }

            let target_light_level = node.light_level.saturating_sub(1);
            if target_light_level == 0 {
                continue;
            }

            if let Some(neighbor_chunk) = world.get_chunk_mut(neighbor_chunk_coord.0, neighbor_chunk_coord.1) {
                let block_at_neighbor = neighbor_chunk.get_block(new_local_x as usize, new_local_y as usize, new_local_z as usize);

                // Block light cannot pass into opaque blocks.
                // The source block itself is an exception handled before the loop.
                if block_at_neighbor.map_or(true, |b| !b.is_transparent()) {
                    continue;
                }

                let current_neighbor_block_light = neighbor_chunk.get_block_light(new_local_x as usize, new_local_y as usize, new_local_z as usize);

                if target_light_level > current_neighbor_block_light {
                    neighbor_chunk.set_block_light(new_local_x as usize, new_local_y as usize, new_local_z as usize, target_light_level);
                    // Only add to queue if it can propagate further (target_light_level > 1)
                    if target_light_level > 1 {
                        light_queue.push_back(LightNode {
                            chunk_coord: neighbor_chunk_coord,
                            local_pos: IVec3::new(new_local_x, new_local_y, new_local_z),
                            light_level: target_light_level,
                        });
                    }
                }
            }
        }
    }
}


// Node for light removal/re-propagation queue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LightRemovalNode {
    chunk_coord: (i32, i32),
    local_pos: IVec3,
    light_level: u8, // The light level this node *had* before being zeroed
}


pub fn remove_light(world: &mut World,
    source_chunk_coord: (i32, i32),
    source_local_pos: IVec3,
    // old_emission_strength: u8, // This was for the source itself, but we need the actual light value at the pos
    is_sky_light_removal: bool
) {
    let mut removal_queue: VecDeque<LightRemovalNode> = VecDeque::new();
    let mut repropagate_queue: VecDeque<LightNode> = VecDeque::new(); // For sources that need re-lighting

    let initial_light_level_at_source = if let Some(chunk) = world.get_chunk(source_chunk_coord.0, source_chunk_coord.1) {
        if is_sky_light_removal {
            chunk.get_sky_light(source_local_pos.x as usize, source_local_pos.y as usize, source_local_pos.z as usize)
        } else {
            chunk.get_block_light(source_local_pos.x as usize, source_local_pos.y as usize, source_local_pos.z as usize)
        }
    } else {
        return; // Source chunk doesn't exist
    };

    if initial_light_level_at_source == 0 {
        return; // Nothing to remove
    }

    // Set the source's light to 0 and add to removal queue
    if let Some(chunk) = world.get_chunk_mut(source_chunk_coord.0, source_chunk_coord.1) {
        if is_sky_light_removal {
            chunk.set_sky_light(source_local_pos.x as usize, source_local_pos.y as usize, source_local_pos.z as usize, 0);
        } else {
            chunk.set_block_light(source_local_pos.x as usize, source_local_pos.y as usize, source_local_pos.z as usize, 0);
        }
        removal_queue.push_back(LightRemovalNode {
            chunk_coord: source_chunk_coord,
            local_pos: source_local_pos,
            light_level: initial_light_level_at_source,
        });
    }


    let directions = [
        IVec3::new(0, 1, 0), IVec3::new(0, -1, 0),
        IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0),
        IVec3::new(0, 0, 1), IVec3::new(0, 0, -1),
    ];
    // Sky light removal doesn't check upwards for initial spread from source, but neighbors can be up.
    let sky_removal_directions = [
        IVec3::new(0, -1, 0), // Down
        IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0), // E, W
        IVec3::new(0, 0, 1), IVec3::new(0, 0, -1), // S, N
        // IVec3::new(0, 1, 0), // Up, for sky light, if a block above was removed, this is complex.
                           // The current model is: light removed at source_pos, check neighbors.
                           // If sky light is removed due to placing an opaque block, the sky_light_propagate
                           // should have already been limited by this new opaque block.
                           // This remove_light is more for when a light source itself is removed, or a path is opened/closed.
    ];


    while let Some(node) = removal_queue.pop_front() {
        let current_dirs = if is_sky_light_removal { &sky_removal_directions[..] } else { &directions[..] };
        for dir in current_dirs {
            let neighbor_local_pos = node.local_pos + *dir;
            let mut neighbor_chunk_coord = node.chunk_coord;

            let mut new_local_x = neighbor_local_pos.x;
            let new_local_y = neighbor_local_pos.y; // Made non-mutable
            let mut new_local_z = neighbor_local_pos.z;

            // Boundary checks and transitions
            if new_local_x < 0 { neighbor_chunk_coord.0 -= 1; new_local_x = CHUNK_WIDTH as i32 - 1; }
            else if new_local_x >= CHUNK_WIDTH as i32 { neighbor_chunk_coord.0 += 1; new_local_x = 0; }
            if new_local_z < 0 { neighbor_chunk_coord.1 -= 1; new_local_z = CHUNK_DEPTH as i32 - 1; }
            else if new_local_z >= CHUNK_DEPTH as i32 { neighbor_chunk_coord.1 += 1; new_local_z = 0; }
            if new_local_y < 0 || new_local_y >= CHUNK_HEIGHT as i32 { continue; }

            let nl_x_usize = new_local_x as usize;
            let nl_y_usize = new_local_y as usize;
            let nl_z_usize = new_local_z as usize;

            if let Some(neighbor_chunk) = world.get_chunk_mut(neighbor_chunk_coord.0, neighbor_chunk_coord.1) {
                // let neighbor_block_is_opaque = neighbor_chunk.get_block(nl_x_usize, nl_y_usize, nl_z_usize)
                //                                  .map_or(true, |b| !b.is_transparent());
                // This variable `neighbor_block_is_opaque` was unused. Logic directly uses transparency/emission.

                let neighbor_current_light = if is_sky_light_removal {
                    neighbor_chunk.get_sky_light(nl_x_usize, nl_y_usize, nl_z_usize)
                } else {
                    neighbor_chunk.get_block_light(nl_x_usize, nl_y_usize, nl_z_usize)
                };

                if neighbor_current_light == 0 { continue; } // Already no light or not affected.

                // If the neighbor's light was potentially sourced from the removed light node
                // or the neighbor is a light source itself that needs re-propagation
                if neighbor_current_light < node.light_level || (!is_sky_light_removal && neighbor_chunk.get_block(nl_x_usize, nl_y_usize, nl_z_usize).map_or(0, |b| b.get_light_emission()) > 0) {
                    if is_sky_light_removal {
                        neighbor_chunk.set_sky_light(nl_x_usize, nl_y_usize, nl_z_usize, 0);
                    } else {
                        neighbor_chunk.set_block_light(nl_x_usize, nl_y_usize, nl_z_usize, 0);
                    }
                    // Add to removal queue to continue darkening
                    removal_queue.push_back(LightRemovalNode {
                        chunk_coord: neighbor_chunk_coord,
                        local_pos: IVec3::new(new_local_x, new_local_y, new_local_z),
                        light_level: neighbor_current_light, // Pass its old light level
                    });
                }
                // If this neighbor still has light (meaning it's getting it from another source, or is a source)
                // add it to the re-propagate queue.
                // For block light, any block with emission > 0 is a candidate.
                // For sky light, any block with sky_light > 0 (after this iteration's potential zeroing) might be.
                // This logic is tricky: we add it to repropagate if its light value is *now* less than what it *would* be if it were a source,
                // OR if it *was* lit by the removed node and now needs to find a new path.
                // Simpler: if a node's light was set to 0, and it *could* have light (either is a source, or was lit by another path), it needs checking.
                // The repropagate queue should contain nodes that *could* have light from other sources or are sources themselves.
                let (emission, current_light_after_potential_zeroing) = if is_sky_light_removal {
                    // For sky light, "emission" is MAX_LIGHT_LEVEL if it's a sky access block, otherwise 0
                    // This needs a proper check: is it a sky access block? For now, assume if its light was >0, it might be.
                    // This part is complex for sky light, as true sky sources are at the top.
                    // A simpler sky removal is just to zero out and then call propagate_sky_light on affected chunks.
                    // For now, let's focus on block light for repropagation from other sources.
                    (0, neighbor_chunk.get_sky_light(nl_x_usize, nl_y_usize, nl_z_usize))
                } else {
                    let emission_val = neighbor_chunk.get_block(nl_x_usize, nl_y_usize, nl_z_usize).map_or(0, |b| b.get_light_emission());
                    (emission_val, neighbor_chunk.get_block_light(nl_x_usize, nl_y_usize, nl_z_usize))
                };

                if current_light_after_potential_zeroing > 0 || emission > 0 {
                     // If it still has light from another path, or is a source itself, queue for re-propagation.
                     // The light_level here should be its *potential* new light level if it were to propagate.
                     // If it's a source, use its emission. If it's lit by another path, use its current_light_after_potential_zeroing.
                    let level_to_repropagate = if emission > current_light_after_potential_zeroing { emission } else { current_light_after_potential_zeroing };
                    if level_to_repropagate > 0 {
                        repropagate_queue.push_back(LightNode {
                            chunk_coord: neighbor_chunk_coord,
                            local_pos: IVec3::new(new_local_x, new_local_y, new_local_z),
                            light_level: level_to_repropagate, // Use its current light or emission strength
                        });
                    }
                }
            }
        }
    }

    // After darkening, re-propagate light from the collected sources
    // This is essentially calling propagate_block_light or propagate_sky_light for each node in repropagate_queue
    // but ensuring we don't get into infinite loops. The existing propagation functions have checks.
    // A better way for repropagate_queue is to collect *actual* sources (blocks with emission or sky access)
    // that were affected (their light value changed or a path through them changed).

    // For simplicity in this iteration: the repropagate_queue now contains nodes that either
    // ARE light sources that were affected, OR blocks that were lit by paths other than the one removed.
    // We can just run the standard propagation functions for these.
    // This might be inefficient if many slightly-dimmed lights are re-propagated.
    // A more optimized approach involves only adding actual light sources to repropagate_queue.

    while let Some(node) = repropagate_queue.pop_front() {
        // We need to get the actual emission strength if it's a source, or its current light if it's just a lit block
        let (final_emission_strength, block_is_transparent_at_reprop_node) = if let Some(chunk) = world.get_chunk(node.chunk_coord.0, node.chunk_coord.1) {
             let block = chunk.get_block(node.local_pos.x as usize, node.local_pos.y as usize, node.local_pos.z as usize);
             let emission = block.map_or(0, |b| b.get_light_emission());
             let transparent = block.map_or(true, |b| b.is_transparent());
             (emission, transparent)
        } else { (0, true) };


        if is_sky_light_removal {
            // For sky light, if a block was darkened, and it still has sky access (e.g. y == CHUNK_HEIGHT -1 and transparent)
            // or it was lit by another sky path, it needs re-propagation.
            // This is complex. A simpler strategy for sky light removal might be to just identify affected chunks
            // and call full `propagate_sky_light` on them after clearing affected areas.
            // The current `propagate_sky_light` re-initializes from top-down, so it might be okay.
            // For now, if sky light is removed, we might need to re-run `propagate_sky_light` for the chunk(s)
            // if an opaque block was removed, exposing new sky access.
            // If a transparent block was placed in a sky path, `propagate_sky_light` should handle it.
            // This `remove_light` for sky is more about when a sky *source itself* is notionally removed (e.g. world ceiling lowered).
            // Let's assume for now that sky light removal is handled by re-running `propagate_sky_light` on affected chunks
            // after the block modification. This function will primarily focus on block light removal's repropagation needs.
            // So, if is_sky_light_removal, we might skip this repropagate queue or handle it differently.
            // For now, let's focus `repropagate_queue` on block lights.
            if node.light_level > 0 { // If it still has some sky light from another path
                 // Add to a *new* sky propagation queue, as sky light propagates differently.
                 // This is getting too complex for one function.
                 // Let's simplify: after remove_light, the calling code should decide if a full sky_propagate is needed.
            }

        } else { // Block light removal
            if final_emission_strength > 0 { // If it's a source itself
                 propagate_block_light(world, node.chunk_coord, node.local_pos, final_emission_strength);
            } else if node.light_level > 0 && block_is_transparent_at_reprop_node {
                // If it's not a source, but was lit by another path, and is transparent,
                // its current light level (node.light_level) acts as a starting point for limited propagation.
                // This is effectively like a temporary light source.
                propagate_block_light(world, node.chunk_coord, node.local_pos, node.light_level);
            }
        }
    }
}

// Helper to check if a block is opaque.
// Note: This might be slightly different from Block::is_solid(), as some non-solid blocks (like leaves) might still block some light.
// For now, using is_transparent for simplicity.
// fn is_opaque(block_type: Option<BlockType>) -> bool {
//     match block_type {
//         Some(BlockType::Air) => false,
//         Some(BlockType::OakLeaves) => false, // For sky light, leaves are transparent
//         None => true, // No block (outside world bounds treated as opaque for safety)
//         Some(_) => true, // All other blocks are opaque
//     }
// }
// The Block struct already has is_transparent(), which should be sufficient.
// block.map_or(true, |b| !b.is_transparent()) handles the None case by treating it as opaque.
