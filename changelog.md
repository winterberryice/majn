# Changelog & Project Status

This document tracks the major changes and current status of the voxel engine project.

## Version 0.1.0

*Initial baseline after documentation refactor. This version includes the features listed below.*

### Current Status (as of last AGENTS.md update before refactor)

*   **Rendering:** Can render static chunks of blocks (e.g., dirt, grass) with basic face culling (CPU-side mesh generation and GPU-side).
*   **Player Controller:**
    *   A grounded "walking" player controller is implemented, replacing the previous fly-cam.
    *   Controls include keyboard input for movement (forward, backward, left, right, jump) and mouse input for orientation (yaw, pitch).
    *   Mouse grabbing (confined cursor) with 'Escape' key toggle is functional.
*   **Physics:**
    *   Player has an Axis-Aligned Bounding Box (AABB).
    *   Gravity and velocity are implemented.
    *   Jumping functionality is present.
    *   Basic "collide and stop" physics are implemented using an axis-by-axis resolution against the block world. True "collide and slide" mechanics are not yet implemented.
    *   Rudimentary friction is applied to horizontal movement.
*   **UI / Debug:**
    *   A debug overlay using `wgpu_text` is functional.
    *   It displays FPS and the player's 3D coordinates.
    *   Visibility can be toggled with the F3 key.
    *   A 2D crosshair is implemented and rendered in the center of the screen.
*   **World / Chunk:**
    *   A single chunk with a flat terrain of dirt and grass is generated.
    *   Mesh generation includes basic culling of hidden faces.
*   **Current Task Focus:** Grass block side texture orientation has been corrected. Next steps could involve refining player physics (e.g., implementing "collide and slide") or starting procedural world generation.
