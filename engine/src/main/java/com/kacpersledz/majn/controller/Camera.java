package com.kacpersledz.majn.controller;

import static org.lwjgl.opengl.GL11.glRotatef;
import static org.lwjgl.opengl.GL11.glTranslatef;
import com.kacpersledz.majn.world.World; // Import World
import com.kacpersledz.majn.world.Block; // Import Block

public class Camera {
    private float x, y, z;
    private float pitch, yaw; // Pitch: rotation around X-axis, Yaw: rotation around Y-axis

    // Made MOVE_SPEED public and static to be accessible from Main or other classes if needed
    public static final float MOVE_SPEED = 0.1f;
    private static final float MOUSE_SENSITIVITY = 0.1f;

    // Player dimensions (for collision) - very simple bounding box
    private static final float PLAYER_HEIGHT = 1.8f;
    private static final float PLAYER_RADIUS = 0.3f; // For horizontal collision

    public Camera(float x, float y, float z) {
        this.x = x;
        this.y = y;
        this.z = z;
        this.pitch = 0;
        this.yaw = 0;
    }

    public float getX() { return x; }
    public float getY() { return y; }
    public float getZ() { return z; }
    public float getPitch() { return pitch; }
    public float getYaw() { return yaw; }

    public void moveRelative(float forward, float right, float up, World world) {
        float dx = 0;
        float dz = 0;

        if (forward != 0) {
            dx += forward * (float)Math.sin(Math.toRadians(yaw)) * (float)Math.cos(Math.toRadians(pitch));
            // FPS-style flying/movement:
            // this.y -= forward * (float)Math.sin(Math.toRadians(pitch));
            dz -= forward * (float)Math.cos(Math.toRadians(yaw)) * (float)Math.cos(Math.toRadians(pitch));
        }
        if (right != 0) {
            dx += right * (float)Math.sin(Math.toRadians(yaw + 90));
            dz -= right * (float)Math.cos(Math.toRadians(yaw + 90));
        }

        // New position before collision checks
        float targetX = this.x + dx;
        float targetY = this.y + up; // Vertical movement is separate
        float targetZ = this.z + dz;

        // Perform collision detection for X, Y, Z axes independently (slide along walls)
        // X-axis movement
        if (!isColliding(targetX, this.y, this.z, world)) {
            this.x = targetX;
        }
        // Y-axis movement
        if (!isColliding(this.x, targetY, this.z, world)) {
            this.y = targetY;
        }
        // Z-axis movement
        if (!isColliding(this.x, this.y, targetZ, world)) {
            this.z = targetZ;
        }
    }

    private boolean isColliding(float checkX, float checkY, float checkZ, World world) {
        if (world == null) return false; // No world, no collision

        // Eye position is (checkX, checkY, checkZ)
        // Feet position is (checkX, checkY - PLAYER_HEIGHT, checkZ)

        // Check a few points:
        // 1. Point at feet center
        // 2. Point at head center (slightly below eye level to avoid ceiling clipping issues)
        // 3. Points at player radius at feet level
        // 4. Points at player radius at head level

        float feetY = checkY - PLAYER_HEIGHT + 0.1f; // Small offset to prevent floor sticking
        float headInternalY = checkY - 0.1f; // Slightly below eye level

        // 1. Feet center
        if (isBlockSolid(world, checkX, feetY, checkZ)) return true;
        // 2. Head center
        if (isBlockSolid(world, checkX, headInternalY, checkZ)) return true;

        // 3 & 4. Horizontal radius checks at feet and head levels
        float[] xOffsets = {PLAYER_RADIUS, -PLAYER_RADIUS, 0, 0};
        float[] zOffsets = {0, 0, PLAYER_RADIUS, -PLAYER_RADIUS};
        // Also check diagonal points for better corner detection
        float diagonalOffset = PLAYER_RADIUS * (float)Math.sqrt(0.5); // approx 0.707 * radius

        float[] allXOffsets = {PLAYER_RADIUS, -PLAYER_RADIUS, 0, 0, diagonalOffset, -diagonalOffset, diagonalOffset, -diagonalOffset};
        float[] allZOffsets = {0, 0, PLAYER_RADIUS, -PLAYER_RADIUS, diagonalOffset, diagonalOffset, -diagonalOffset, -diagonalOffset};


        for (int i = 0; i < allXOffsets.length; i++) {
            float offsetX = allXOffsets[i];
            float offsetZ = allZOffsets[i];

            // Feet level with radius
            if (isBlockSolid(world, checkX + offsetX, feetY, checkZ + offsetZ)) return true;
            // Head level with radius
            if (isBlockSolid(world, checkX + offsetX, headInternalY, checkZ + offsetZ)) return true;
        }

        // An additional check slightly above feet for step-up or minor floor variations
        // (could be more sophisticated with actual step height logic)
        // float slightlyAboveFeetY = checkY - PLAYER_HEIGHT + 0.5f; // e.g. 0.5 block height
        // if (isBlockSolid(world, checkX, slightlyAboveFeetY, checkZ)) return true;


        return false; // No collision
    }

    private boolean isBlockSolid(World world, float worldX, float worldY, float worldZ) {
        Block block = world.getBlock((int)Math.floor(worldX), (int)Math.floor(worldY), (int)Math.floor(worldZ));
        return block != null && block.getType() != Block.BlockType.AIR;
    }

    public void rotate(float dpitch, float dyaw) {
        this.pitch += dpitch * MOUSE_SENSITIVITY;
        this.yaw += dyaw * MOUSE_SENSITIVITY;

        // Clamp pitch to avoid flipping
        if (this.pitch > 89.0f) {
            this.pitch = 89.0f;
        }
        if (this.pitch < -89.0f) {
            this.pitch = -89.0f;
        }

        // Keep yaw within 0-360 degrees (optional, but good for consistency)
        this.yaw %= 360;
        if (this.yaw < 0) {
            this.yaw += 360;
        }
    }

    public void applyTransformations() {
        // Apply rotations first, then translation
        glRotatef(pitch, 1.0f, 0.0f, 0.0f); // Rotate around X-axis (pitch)
        glRotatef(yaw, 0.0f, 1.0f, 0.0f);   // Rotate around Y-axis (yaw)
        glTranslatef(-x, -y, -z);           // Translate to the negative of camera's position
    }
}
