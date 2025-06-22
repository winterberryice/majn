package com.kacpersledz.majn.controller;

// import com.kacpersledz.majn.Main; // No longer needed
import com.kacpersledz.majn.view.Window;
import org.lwjgl.glfw.GLFW;

import static org.lwjgl.glfw.GLFW.*;

public class InputHandler {

    private Camera camera;
    // private Main mainApp; // No longer needed
    private Window window; // Reference to window to control cursor mode

    // Camera movement state
    private boolean moveForward;
    private boolean moveBackward;
    private boolean moveLeft;
    private boolean moveRight;
    private boolean moveUp;
    private boolean moveDown;

    // Mouse position state
    private double lastMouseX = 0;
    private double lastMouseY = 0;
    private boolean firstMouse = true;

    // Game state
    private boolean isPaused = false;
    private boolean showDebugInfo = false;

    public InputHandler(Camera camera, Window window) { // Main app reference removed
        this.camera = camera;
        this.window = window;
        // Initialize lastMouseX/Y to window center, assumes window is somewhat initialized
        // This might need adjustment if window dimensions aren't known yet.
        // For now, let's assume they can be fetched or are default.
        if (window != null) {
            this.lastMouseX = window.getWidth() / 2.0;
            this.lastMouseY = window.getHeight() / 2.0;
        }
    }

    public void setInitialMousePosition(double x, double y) {
        this.lastMouseX = x;
        this.lastMouseY = y;
        this.firstMouse = true;
    }


    public void handleKeyInput(long windowHandle, int key, int scancode, int action, int mods) {
        if (key == GLFW_KEY_ESCAPE && action == GLFW_RELEASE) {
            isPaused = !isPaused;
            // mainApp.setPaused(isPaused); // No longer needed, Main queries InputHandler
            if (this.window != null) {
                this.window.setCursorDisabled(!isPaused);
            }
            if (!isPaused) {
                firstMouse = true; // Reset for smooth mouse capture when unpausing
            }
        }
        if (key == GLFW_KEY_F3 && action == GLFW_RELEASE) {
            showDebugInfo = !showDebugInfo;
            // mainApp.toggleShowDebugInfo(); // No longer needed, Main queries InputHandler
        }

        // Camera movement keys
        if (action == GLFW_PRESS || action == GLFW_RELEASE) {
            boolean pressed = (action == GLFW_PRESS);
            switch (key) {
                case GLFW_KEY_W:
                    moveForward = pressed;
                    mainApp.setMoveForward(pressed); // Update Main
                    break;
                case GLFW_KEY_S:
                    moveBackward = pressed;
                    mainApp.setMoveBackward(pressed); // Update Main
                    break;
                case GLFW_KEY_A:
                    moveLeft = pressed;
                    mainApp.setMoveLeft(pressed); // Update Main
                    break;
                case GLFW_KEY_D:
                    moveRight = pressed;
                    mainApp.setMoveRight(pressed); // Update Main
                    break;
                case GLFW_KEY_SPACE:
                    moveUp = pressed;
                    mainApp.setMoveUp(pressed); // Update Main
                    break;
                case GLFW_KEY_F: // Assuming F is for down
                    moveDown = pressed;
                    mainApp.setMoveDown(pressed); // Update Main
                    break;
            }
        }
    }

    public void handleMouseMovement(long windowHandle, double xpos, double ypos) {
        if (isPaused) { // Do not process mouse movement if paused
            return;
        }

        if (firstMouse) {
            lastMouseX = xpos;
            lastMouseY = ypos;
            firstMouse = false;
        }

        float xoffset = (float) (xpos - lastMouseX);
        float yoffset = (float) (ypos - lastMouseY); // Corrected for non-inverted pitch

        lastMouseX = xpos;
        lastMouseY = ypos;

        if (camera != null) {
            camera.rotate(yoffset, xoffset);
        }
    }

    public void handleFramebufferResize(int width, int height) {
        // If the InputHandler needs to react to framebuffer size changes,
        // e.g., re-centering mouse or adjusting sensitivity, do it here.
        // For now, this is a placeholder.
        // The Renderer will primarily handle this for projection matrix updates.
        if (camera != null && mainApp != null) { // mainApp for setupProjection, will move to Renderer
            // This call to setupProjection will be moved to Renderer
            // mainApp.setupProjection(); // This is incorrect here, Main will call Renderer
        }
    }

    // Getters for game state (Main/Game class will query these)
    public boolean isMoveForward() { return moveForward; }
    public boolean isMoveBackward() { return moveBackward; }
    public boolean isMoveLeft() { return moveLeft; }
    public boolean isMoveRight() { return moveRight; }
    public boolean isMoveUp() { return moveUp; }
    public boolean isMoveDown() { return moveDown; }
    public boolean isPaused() { return isPaused; }
    public boolean isShowingDebugInfo() { return showDebugInfo; }

    public void resetFirstMouse() {
        this.firstMouse = true;
    }
}
