# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - YYYY-MM-DD (date of update)

### Added
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
    - Basic "collide and stop" physics (axis-by-axis resolution against block world).
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

### Fixed
- Grass block side texture orientation. *(Duplicated in "Added" as it was a specific fix leading to a feature state)*

[Unreleased]: https://github.com/placeholder-username/placeholder-repo/compare/v0.1.0...HEAD
<!-- Possible future release link -->
<!-- [0.1.0]: https://github.com/placeholder-username/placeholder-repo/releases/tag/v0.1.0 -->
