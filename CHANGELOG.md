# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2024-04-26

### Added
- **Lighting and Day/Night Cycle:**
  - Implemented per-block sky light and block light levels (0-15).
  - Light propagation system using BFS for spreading and removing light.
  - Dynamic day/night cycle (20-minute cycle) that influences global skylight levels.
  - Global skylight changes trigger recalculation and propagation of light.
  - Caves and unlit areas become dark based on propagated light levels.
  - Light emission property for blocks (e.g., `Torch`, `Glowstone` defined with emission values).
  - Integrated light levels into the rendering pipeline:
    - Vertex data now includes a normalized light level.
    - Shaders (vertex and fragment) updated to use this light level to modulate final block color.
    - Added a minimum ambient light factor in the fragment shader to ensure visibility in very dark areas.
- **Rendering:** Static chunks of blocks (e.g., dirt, grass) with basic face culling (CPU-side mesh generation and GPU-side).
- **Player Controller:**
    - Grounded "walking" player controller (replaces previous fly-cam).
    - Keyboard input for movement (forward, backward, left, right, jump).
    - Mouse input for orientation (yaw, pitch).
    - Mouse grabbing (confined cursor) with 'Escape' key toggle.
- **Physics:**
    - Player Axis-Aligned Bounding Box (AABB).
    - Gravity and velocity implementation.
    - Jumping functionality.
    - "Collide and Slide" physics: Implemented via axis-by-axis AABB resolution, allowing smoother movement along walls when colliding at an angle.
    - Rudimentary friction for horizontal movement.
- **UI / Debug:**
    - Debug overlay (`wgpu_text`) displaying FPS and player 3D coordinates.
    - F3 key toggles debug overlay visibility.
    - 2D crosshair rendered in the center of the screen.
- **World / Chunk:**
    - Single chunk generation with flat terrain (dirt and grass).
    - Mesh generation with basic culling of hidden faces.
- **Textures:**
    - Corrected grass block side texture orientation.
- **Interaction:**
    - Raycasting for block identification and selection.
    - Block placement functionality.
    - Block removal functionality.

### Changed
- Player controller from fly-cam to a grounded walking controller.
- Physics collision response from "collide and stop" to "collide and slide".

### Fixed
- Grass block side texture orientation.

[Unreleased]: https://github.com/placeholder-username/placeholder-repo/compare/v0.1.0...HEAD
<!-- Possible future release link -->
<!-- [0.1.0]: https://github.com/placeholder-username/placeholder-repo/releases/tag/v0.1.0 -->
