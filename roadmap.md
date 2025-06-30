# Project Roadmap

This is our high-level plan. We will tackle these items one by one.

1.  **Player Controller**
    *   [x] 3D Fly-Cam (Superseded by walking controller)
    *   [x] Implement AABB (Axis-Aligned Bounding Box) for the player.
    *   [x] Implement gravity and velocity.
    *   [/] Implement "collide and stop" physics (slide mechanics pending).
2.  **Visuals & World**
    *   [x] Texture blocks using a texture atlas. (Grass block side texture orientation fixed)
    *   [ ] Implement procedural world generation with a noise function. (Current generation is flat)
3.  **Interaction**
    *   [ ] Implement raycasting for block selection.
    *   [ ] Add block placement and removal logic.
4.  **Engine Architecture**
    *   [/] Continue refactoring code into logical modules (`player`, `world`, `renderer`, `physics`, `debug_overlay` etc. - Good progress made).
5.  **UI**
    *   [x] Basic debug overlay (FPS, coordinates).
    *   [x] Implement a crosshair.
