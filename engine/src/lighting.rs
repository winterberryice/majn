use crate::world::World;
use crate::chunk::{CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH};
// BlockType is not directly used here, but Block's methods (is_opaque_for_light) are.
// We access blocks via world.get_block_world_space.
use glam::IVec3;
use std::collections::VecDeque;

const MAX_LIGHT_LEVEL: u8 = 15;

// Renamed to reflect it's for the entire world, not just one chunk.
// Or, more accurately, it's initiated *for* a chunk, but propagation is world-wide from its sources.
pub fn calculate_initial_sky_light_for_chunk_columns(world: &mut World, chunk_coord_x: i32, chunk_coord_z: i32, light_queue: &mut VecDeque<LightNode>) {
    // Phase 1: Initial sky light sources for the specified chunk.
    // Iterate through each column (x, z) in the specified chunk.
    for x_in_chunk in 0..CHUNK_WIDTH {
        for z_in_chunk in 0..CHUNK_DEPTH {
            let world_x = chunk_coord_x * CHUNK_WIDTH as i32 + x_in_chunk as i32;
            let world_z = chunk_coord_z * CHUNK_DEPTH as i32 + z_in_chunk as i32;

            // Start from the top of the world and go down for this column.
            for y_world in (0..CHUNK_HEIGHT).rev() {
                let current_world_pos = IVec3::new(world_x, y_world as i32, world_z);

                // Check if chunk for current_world_pos is loaded. If not, we can't know if it's opaque.
                // For initial generation, the relevant chunk should be loaded.
                // This part assumes that the world provides a way to check block opacity at current_world_pos.
                let block_opt = world.get_block_world_space(current_world_pos);

                if let Some(block) = block_opt {
                    if !block.is_opaque_for_light() {
                        // This block is transparent and has a direct view of the sky
                        // (or blocks above it in this column were also transparent)
                        // Set light and add to global BFS queue
                        world.set_sky_light_world_space(current_world_pos, MAX_LIGHT_LEVEL);
                        light_queue.push_back(LightNode { pos: current_world_pos, level: MAX_LIGHT_LEVEL });
                    } else {
                        // This block is opaque, so sky light stops here for this column from direct downward path.
                        break;
                    }
                } else {
                    // Block (and likely chunk) doesn't exist at this position.
                    // Consider this as transparent for sky light to continue downwards.
                    // If it's above any generated terrain, it should get full light.
                    // If it's below, it means we are checking ungenerated parts of a column.
                    // This case should be handled carefully. If generating chunk by chunk,
                    // this y-loop should ideally only scan within the currently generating chunk's vertical extent
                    // or rely on existing (already generated) chunks above.
                    // For now, assume if no block, it's like air.
                    world.set_sky_light_world_space(current_world_pos, MAX_LIGHT_LEVEL);
                    light_queue.push_back(LightNode { pos: current_world_pos, level: MAX_LIGHT_LEVEL });
                }
            }
        }
    }
}

pub fn propagate_queued_light(world: &mut World, light_queue: &mut VecDeque<LightNode>, is_sky_light: bool) {
    // Phase 2: BFS Propagation using world coordinates
    while let Some(node) = light_queue.pop_front() {
        let current_pos_world = node.pos;
        let current_light_level = node.level;

        // Iterate over neighbors (6 cardinal directions)
        for dx_i32 in -1..=1 {
            for dy_i32 in -1..=1 {
                for dz_i32 in -1..=1 {
                    let dx: i32 = dx_i32;
                    let dy: i32 = dy_i32;
                    let dz: i32 = dz_i32;
                    if dx.abs() + dy.abs() + dz.abs() != 1 {
                        continue; // Skip self and diagonals
                    }

                    let neighbor_world_pos = current_pos_world + IVec3::new(dx, dy, dz);

                    // Boundary checks for Y
                    if neighbor_world_pos.y < 0 || neighbor_world_pos.y >= CHUNK_HEIGHT as i32 {
                        continue;
                    }

                    // Calculate new light level. Sky light also considers downward propagation preference.
                    let mut attenuation = 1;
                    if is_sky_light && dy == -1 && world.get_sky_light_world_space(current_pos_world).unwrap_or(0) == MAX_LIGHT_LEVEL {
                        // If current is max sky light and we are going straight down, no attenuation.
                        // This helps sunlight "pour" down directly without dimming.
                        attenuation = 0;
                    }

                    let new_light_level = current_light_level.saturating_sub(attenuation);

                    if new_light_level == 0 {
                        continue;
                    }

                    // Check if neighbor block is opaque
                    if let Some(block) = world.get_block_world_space(neighbor_world_pos) {
                        if block.is_opaque_for_light() {
                            // If the opaque block itself is the neighbor_world_pos, it cannot receive this light.
                            // However, if we are calculating for block light sources, the source itself might be opaque
                            // but still emit light. This distinction is handled by how sources are added.
                            // For propagation, light stops *entering* an opaque block.
                            continue;
                        }
                    } else {
                        // No block at neighbor_world_pos (e.g., in an unloaded chunk or air).
                        // Light should propagate through it. If the chunk is unloaded,
                        // world.set_sky_light_world_space might do nothing or might need to handle it.
                        // For now, assume propagation continues.
                    }

                    let existing_light = if is_sky_light {
                        world.get_sky_light_world_space(neighbor_world_pos).unwrap_or(0)
                    } else {
                        world.get_block_light_world_space(neighbor_world_pos).unwrap_or(0)
                    };

                    if new_light_level > existing_light {
                        if is_sky_light {
                            world.set_sky_light_world_space(neighbor_world_pos, new_light_level);
                        } else {
                            world.set_block_light_world_space(neighbor_world_pos, new_light_level);
                        }
                        light_queue.push_back(LightNode { pos: neighbor_world_pos, level: new_light_level });
                    }
                }
            }
        }
    }
}


pub fn initialize_sky_light_for_new_chunk(world: &mut World, chunk_coord_x: i32, chunk_coord_z: i32) {
    let mut sky_light_bfs_queue: VecDeque<LightNode> = VecDeque::new();

    // Step 1: Identify initial sky light sources for the columns in the new chunk.
    // This fills the part of the queue with blocks that have direct sky access.
    calculate_initial_sky_light_for_chunk_columns(world, chunk_coord_x, chunk_coord_z, &mut sky_light_bfs_queue);

    // Step 2: Propagate light from these sources throughout the world.
    // The queue might now contain sources from this chunk.
    // The propagation needs to be able to cross into neighboring chunks.
    propagate_queued_light(world, &mut sky_light_bfs_queue, true);
}


pub fn propagate_block_light(world: &mut World, sources: Vec<LightNode>) {
    // The `sources` are LightNode structs containing world position and emission level.
    let mut block_light_queue: VecDeque<LightNode> = VecDeque::new();

    for source_node in sources {
        // Set the initial light level at the source block's position.
        // It's important to do this before adding to queue, so the propagation
        // algorithm doesn't try to re-light the source block itself from its neighbors
        // if it starts with a lower light value.
        let existing_block_light = world.get_block_light_world_space(source_node.pos).unwrap_or(0);

        // Only update and add to queue if this source is brighter than existing light at that position.
        // Or if the existing light is 0 - meaning it can be lit.
        // If existing_block_light is already >= source_node.level, this source won't add anything new from this spot.
        if source_node.level > existing_block_light {
            world.set_block_light_world_space(source_node.pos, source_node.level);
            block_light_queue.push_back(source_node); // Add to queue for propagation only if it's a new, brighter source
        } else if existing_block_light == 0 && source_node.level > 0 {
            // If the spot is dark and source has light, set it and propagate
            world.set_block_light_world_space(source_node.pos, source_node.level);
            block_light_queue.push_back(source_node);
        }
        // If source_node.level <= existing_block_light (and existing_block_light > 0),
        // this specific source won't propagate from this exact block position,
        // as it's already lit sufficiently or brighter.
        // However, if multiple sources are passed, others might still be processed.
    }

    // Only proceed with propagation if there are actual sources in the queue.
    if !block_light_queue.is_empty() {
        propagate_queued_light(world, &mut block_light_queue, false);
    }
}


// This function "removes" light by setting it to 0 and finding sources that need re-propagation.
// It returns a queue of sources from which light needs to be spread again.
fn unpropagate_light(
    world: &mut World,
    start_pos_world: IVec3,
    is_sky_light: bool,
) -> VecDeque<LightNode> {
    let mut removal_queue: VecDeque<(IVec3, u8)> = VecDeque::new();
    let mut repropagate_sources_queue: VecDeque<LightNode> = VecDeque::new();

    let initial_light_level = if is_sky_light {
        world.get_sky_light_world_space(start_pos_world).unwrap_or(0)
    } else {
        world.get_block_light_world_space(start_pos_world).unwrap_or(0)
    };

    if initial_light_level > 0 {
        if is_sky_light {
            world.set_sky_light_world_space(start_pos_world, 0);
        } else {
            world.set_block_light_world_space(start_pos_world, 0);
        }
        removal_queue.push_back((start_pos_world, initial_light_level));
    }

    while let Some((current_pos, prev_removed_level)) = removal_queue.pop_front() {
        for dx_i32 in -1..=1 {
            for dy_i32 in -1..=1 {
                for dz_i32 in -1..=1 {
                    let dx: i32 = dx_i32;
                    let dy: i32 = dy_i32;
                    let dz: i32 = dz_i32;
                    if dx.abs() + dy.abs() + dz.abs() != 1 {
                        continue; // Skip self and diagonals
                    }
                    let neighbor_pos = current_pos + IVec3::new(dx, dy, dz);

                    if neighbor_pos.y < 0 || neighbor_pos.y >= CHUNK_HEIGHT as i32 {
                        continue;
                    }

                    let neighbor_light_level = if is_sky_light {
                        world.get_sky_light_world_space(neighbor_pos).unwrap_or(0)
                    } else {
                        world.get_block_light_world_space(neighbor_pos).unwrap_or(0)
                    };

                    if neighbor_light_level == 0 {
                        continue; // Already dark or unlit
                    }

                    // If the neighbor was lit by the current path of light being removed
                    // (i.e., its light level is consistent with being one step away from prev_removed_level)
                    // or simply weaker than the light level we are removing.
                    // A simple check: if its light is less than the light level of the block that was just darkened.
                    // More accurately: if neighbor_light_level == prev_removed_level - 1 (or for sky pouring down, neighbor_light_level == prev_removed_level)
                    // For simplicity now: if neighbor_light_level < prev_removed_level, it might have depended on it.
                    // A key condition: light propagates from higher to lower. If neighbor_light_level is equal or greater,
                    // it means it has its own source or an alternative path of equal or greater strength.
                    // If neighbor_light_level < prev_removed_level, it *could* have been lit by current_pos.
                    // If neighbor_light_level == prev_removed_level - 1 (this is the most direct dependency)

                    let expected_neighbor_light_if_dependent = prev_removed_level.saturating_sub(1);
                    // Special sky light direct down propagation:
                    let mut expected_sky_downward_light_if_dependent = prev_removed_level; // No attenuation if prev was MAX
                    if prev_removed_level < MAX_LIGHT_LEVEL { // If prev wasn't max, then it attenuates
                        expected_sky_downward_light_if_dependent = prev_removed_level.saturating_sub(1);
                    }


                    if neighbor_light_level > 0 &&
                       (neighbor_light_level < prev_removed_level || // General case: it was dimmer, so potentially dependent
                        (is_sky_light && dy == -1 && current_pos.y > neighbor_pos.y && neighbor_light_level == expected_sky_downward_light_if_dependent) || // Sky light going down
                        (!is_sky_light && neighbor_light_level == expected_neighbor_light_if_dependent) // Block light or non-direct sky
                       )
                    {
                        // This neighbor's light might have depended on current_pos.
                        // Set its light to 0 and add it to the removal queue.
                        if is_sky_light {
                            world.set_sky_light_world_space(neighbor_pos, 0);
                        } else {
                            world.set_block_light_world_space(neighbor_pos, 0);
                        }
                        removal_queue.push_back((neighbor_pos, neighbor_light_level));
                    } else if neighbor_light_level > 0 {
                        // This neighbor is lit, but not (solely) by the path we are currently removing.
                        // It's either a source itself or lit by another path. Add it to repropagate.
                        // Ensure not to add duplicates if it's already processed or will be.
                        // The repropagate_sources_queue will be processed by propagate_queued_light,
                        // which handles "if new_light > existing_light".
                        repropagate_sources_queue.push_back(LightNode {
                            pos: neighbor_pos,
                            level: neighbor_light_level,
                        });
                    }
                }
            }
        }
    }
    repropagate_sources_queue
}

// TODO: Implement light removal and update logic
// pub fn remove_light(world: &mut World, removed_block_pos: IVec3, old_block_type_info, affected_chunks_queue: &mut VecDeque<(i32,i32)> )

pub fn update_light_after_block_change(
    world: &mut World,
    changed_pos_world: IVec3,
    old_block_was_opaque: bool, // Opacity of the block *before* change
    new_block_is_opaque: bool,   // Opacity of the block *after* change
    _old_emission: u8, // Prefixed with underscore as it's not directly used in this function body
                       // Its conceptual importance is that if it was > 0, unpropagate_light for block light
                       // would have handled removing its effects.
    new_emission: u8,
) {
    // Placeholder for the full update logic.
    // This will involve:
    // 1. Removing old light (sky and block) using unpropagate_light.
    //    This returns lists of sources that need re-spreading.
    // 2. Calculating new direct sky light if a block was removed and exposed sky.
    // 3. Adding new block light sources if a light-emitting block was placed.
    // 4. Propagating all collected new/repropagated sources.

    let mut sky_reprop_sources = unpropagate_light(world, changed_pos_world, true);
    let mut block_reprop_sources = unpropagate_light(world, changed_pos_world, false);

    // If an opaque block was removed, or a transparent block became opaque, sky light might change.
    if old_block_was_opaque && !new_block_is_opaque { // Opaque block removed, sky might pour in
        // Recalculate direct sky light for the column starting from changed_pos_world downwards.
        for y_scan in (0..=changed_pos_world.y).rev() {
            let current_scan_pos = IVec3::new(changed_pos_world.x, y_scan, changed_pos_world.z);
            if let Some(block_at_scan) = world.get_block_world_space(current_scan_pos) {
                if !block_at_scan.is_opaque_for_light() {
                    world.set_sky_light_world_space(current_scan_pos, MAX_LIGHT_LEVEL);
                    sky_reprop_sources.push_back(LightNode{pos: current_scan_pos, level: MAX_LIGHT_LEVEL});
                } else {
                    break; // Sky path blocked
                }
            } else { // No block means air, sky light passes
                world.set_sky_light_world_space(current_scan_pos, MAX_LIGHT_LEVEL);
                sky_reprop_sources.push_back(LightNode{pos: current_scan_pos, level: MAX_LIGHT_LEVEL});
            }
        }
    }
    // (More complex sky scenarios, e.g. placing an opaque block high up, are covered by unpropagation + repropagation)

    // Handle block light emission changes
    if new_emission > 0 {
        // If there's new emission, add it as a source.
        // `propagate_block_light` handles if it's brighter than existing.
        block_reprop_sources.push_back(LightNode{pos: changed_pos_world, level: new_emission});
    }
    // If emission was removed (old_emission > 0 and new_emission == 0),
    // `unpropagate_light` for block light should have handled removing its influence.

    // Now, repropagate all affected lights
    if !sky_reprop_sources.is_empty() {
        propagate_queued_light(world, &mut sky_reprop_sources, true);
    }
    if !block_reprop_sources.is_empty() {
        propagate_queued_light(world, &mut block_reprop_sources, false);
    }
}


pub fn initialize_lighting_for_generated_chunk(world: &mut World, chunk_x: i32, chunk_z: i32) {
    // Initialize Sky Light
    initialize_sky_light_for_new_chunk(world, chunk_x, chunk_z);

    // Initialize Block Light
    let mut block_light_sources: Vec<LightNode> = Vec::new();
    // Iterate over the just-generated chunk to find all blocks with emission > 0
    // We need to access the chunk data directly here, as world.get_block_world_space might
    // trigger loading of other chunks if we are not careful with coordinates.
    // However, for a newly generated chunk, its data should be primary.
    if let Some(chunk) = world.get_chunk(chunk_x, chunk_z) { // Get immutable borrow first
        for lx in 0..CHUNK_WIDTH {
            for ly in 0..CHUNK_HEIGHT {
                for lz in 0..CHUNK_DEPTH {
                    // We need local coordinates for get_block, not world coordinates.
                    // The IVec3 for local_pos in chunk is (lx, ly, lz)
                    if let Some(block) = chunk.get_block(lx, ly, lz) {
                        let emission = block.get_light_emission();
                        if emission > 0 {
                            let world_pos = IVec3::new(
                                chunk_x * CHUNK_WIDTH as i32 + lx as i32,
                                ly as i32, // y is already world space index
                                chunk_z * CHUNK_DEPTH as i32 + lz as i32
                            );
                            block_light_sources.push(LightNode{pos: world_pos, level: emission});
                            // Initial setting of the light at source is handled by propagate_block_light
                        }
                    }
                }
            }
        }
    }

    if !block_light_sources.is_empty() {
        propagate_block_light(world, block_light_sources);
    }
}

// Helper struct for the light propagation queue items
// Made public for use in other modules if complex light updates need to build queues.
// Or keep it private if only this module manages queues. For now, private.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LightNode { // Changed to pub(crate)
    pos: IVec3, // World position
    level: u8,
}
