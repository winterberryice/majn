# Project Roadmap

This document outlines the planned features and high-level goals for the future development of this voxel engine. Items are grouped by their target release version.

---

## Version 0.1.0 (Foundations & Core Mechanics)

This version represents the current, functional state of the engine. The goal of this release is to have a solid base with a playable character in a static world.

* - [x] **Player Controller:**
    * Grounded "walking" controller with AABB.
    * Gravity, velocity, and jump mechanics.
    * "Collide and Slide" physics (axis-by-axis AABB resolution).
* - [x] **Rendering & Visuals:**
    * Static chunk rendering with basic face culling.
    * Texture mapping support via a texture atlas.
    * Basic lighting system (ambient light + one directional light for the sun).
* - [x] **Interaction:**
    * Raycasting for block selection.
    * Block placement and removal.
* - [x] **UI:**
    * Debug overlay (FPS, coordinates).
    * Static crosshair.
* - [x] **Architecture & Build:**
    * Modular code structure (`player`, `world`, `renderer`, etc.).
    * Automated builds for Windows, Linux, and macOS (Universal) via GitHub Actions.

---

## Version 0.2.0 (The Polish & Physics Update)

The primary goal of this version is to refine the existing gameplay loop and improve the overall feel and visual consistency of the core experience.

* - [ ] **Player Physics:**
    * Refine player movement values (acceleration, friction, jump height) for better game feel.
* - [ ] **Visual Polish:**
    * Implement biome tints (e.g., for grass, foliage) in the shader to add color variety to the world.
    * Add simple visual/audio feedback for block placement and removal.
* - [ ] **Engine Refinements:**
    * Improve the raycasting algorithm for more precision.
    * Refactor engine components as needed to support new features cleanly.

---

## Backlog (Future Ideas - Not Assigned to a Version)

This is a list of major features and ideas for consideration in future development cycles.

fix fps drop on camera movement caused by commit (e19f4d4e0aad359b9c5607298b6b79438fad31ee)

* **World Generation:**
    * Integrate a noise library (e.g., `noise-rs`).
    * Implement procedural chunk generation using a noise-based heightmap.
    * More complex world generation (caves, ores, structures).
* **Multi-Chunk System:**
    * Create a `ChunkManager` to handle a grid of chunks around the player.
    * Implement logic for loading new chunks as the player moves and unloading distant ones.
    * Multi-threaded chunk generation.
* **UI Systems:**
    * **HUD (Heads-Up Display):** Implement a hotbar UI with selectable item slots.
    * **Inventory & Crafting Screens:** Design a full inventory UI and a crafting grid.
    * **Pause & Settings Menu:** Create a pause menu with settings like "Render Distance".
* **Gameplay Systems:**
    * **Inventory Logic:** Data structures and logic for managing the player's inventory.
    * **Item System:** A system for defining different types of items (blocks, tools, etc.).
    * **Crafting Logic:** A recipe system that works with the crafting grid UI.
    * **Saving & Loading:** Implement logic to save the world state to a file and load it back.
    * **Health & Damage System.**
* **Advanced Physics & Player Controller:**
    * Add swimming/water physics.
* **Advanced Rendering:**
    * Frustum Culling (don't process chunks that are outside the camera's view).
    * Support for transparent blocks (e.g., water, glass).
* **Entities:**
    * Basic Entity Component System (ECS) for mobs and other dynamic objects.
* **Networking:**
    * Design a basic client-server architecture for multiplayer.
