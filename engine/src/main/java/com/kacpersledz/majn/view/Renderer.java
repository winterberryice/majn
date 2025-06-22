package com.kacpersledz.majn.view;

import com.kacpersledz.majn.controller.Camera;
import com.kacpersledz.majn.world.Block;
import com.kacpersledz.majn.world.Chunk;
import com.kacpersledz.majn.world.World;
import org.joml.Matrix4f;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.opengl.GL11;
import org.lwjgl.system.MemoryStack;

import java.nio.FloatBuffer;
import java.util.Collection;

import static org.lwjgl.opengl.GL11.*;
import static org.lwjgl.glfw.GLFW.glfwGetTime; // For FPS in debug, consider a separate timer class

public class Renderer {

    private Window window;
    private Camera camera;
    private World world;
    private FontRenderer fontRenderer; // Will be provided by Main/Game

    // FPS calculation fields (might move to a dedicated game timer or stats class)
    private double lastFrameTime = 0.0;
    private int frames = 0;
    private double timeAccumulator = 0.0;
    private double fps = 0.0;


    public Renderer(Window window, Camera camera, World world, FontRenderer fontRenderer) {
        this.window = window;
        this.camera = camera;
        this.world = world;
        this.fontRenderer = fontRenderer;
    }

    public void init() {
        // Initialize OpenGL settings
        glEnable(GL_DEPTH_TEST);
        glClearColor(0.0f, 0.0f, 0.2f, 0.0f); // Default clear color (blueish)
        // Other GL settings like blend modes for transparency can be set here if always needed
        // glGetString(GL_VERSION) is good for logging, Main can do it or Renderer can return it

        // Initial projection setup
        setupProjection();
        lastFrameTime = glfwGetTime(); // Initialize for FPS calculation
    }

    public void setupProjection() {
        glMatrixMode(GL_PROJECTION);
        glLoadIdentity();
        Matrix4f projectionMatrix = new Matrix4f();
        float aspectRatio = (window.getHeight() > 0) ? (float) window.getWidth() / window.getHeight() : 1.0f;
        projectionMatrix.perspective((float) Math.toRadians(60.0f), aspectRatio, 0.1f, 500.0f);

        try (MemoryStack stack = MemoryStack.stackPush()) {
            FloatBuffer fb = stack.mallocFloat(16);
            projectionMatrix.get(fb);
            glLoadMatrixf(fb);
        }
        glMatrixMode(GL_MODELVIEW); // Switch back to ModelView matrix
        glLoadIdentity(); // Reset ModelView matrix
    }

    public void clear() {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    }

    public void render() {
        // Clear the screen - this might be done by the Game loop before calling renderWorld, renderUI etc.
        // clear(); // Let's assume Game class calls clear() then renderer.renderWorld(), renderer.renderUI() etc.

        renderWorld();
        // UI elements will be rendered on top, potentially by separate methods
    }


    private void renderWorld() {
        if (this.world == null || this.camera == null) {
            return;
        }

        glMatrixMode(GL_MODELVIEW);
        glLoadIdentity(); // Reset model-view matrix
        camera.applyTransformations(); // Apply camera view

        glEnable(GL_DEPTH_TEST); // Ensure depth test is enabled for 3D scene

        float blockSize = 1.0f;
        int playerX = (int) Math.floor(camera.getX());
        int playerY = (int) Math.floor(camera.getY());
        int playerZ = (int) Math.floor(camera.getZ());

        Collection<Chunk> chunksToRender = world.getChunksAroundPlayer(playerX, playerY, playerZ);

        for (Chunk currentChunk : chunksToRender) {
            if (currentChunk == null) continue;

            int chunkBaseX = currentChunk.getChunkX();
            int chunkBaseY = currentChunk.getChunkY();
            int chunkBaseZ = currentChunk.getChunkZ();

            for (int localX = 0; localX < Chunk.CHUNK_WIDTH; localX++) {
                for (int localY = 0; localY < Chunk.CHUNK_HEIGHT; localY++) {
                    for (int localZ = 0; localZ < Chunk.CHUNK_DEPTH; localZ++) {
                        Block block = currentChunk.getBlock(localX, localY, localZ);
                        if (block != null && block.getType() != Block.BlockType.AIR) {

                            if (block.getType() == Block.BlockType.DIRT) {
                                glColor3f(0.6f, 0.4f, 0.2f);
                            } else if (block.getType() == Block.BlockType.GRASS) {
                                glColor3f(0.0f, 0.8f, 0.0f);
                            } else {
                                glColor3f(0.5f, 0.5f, 0.5f);
                            }

                            float worldX = (chunkBaseX * Chunk.CHUNK_WIDTH + localX) * blockSize;
                            float worldY = (chunkBaseY * Chunk.CHUNK_HEIGHT + localY) * blockSize;
                            float worldZ = (chunkBaseZ * Chunk.CHUNK_DEPTH + localZ) * blockSize;

                            glPushMatrix();
                            glTranslatef(worldX, worldY, worldZ);
                            drawCube(blockSize); // Abstracted cube drawing
                            glPopMatrix();
                        }
                    }
                }
            }
        }
        glMatrixMode(GL_MODELVIEW); // Ensure modelview is active for other rendering
    }

    private void drawCube(float size) {
        float halfSize = size / 2;
        // Filled cube
        glBegin(GL_QUADS);
        // Front face
        glVertex3f(-halfSize, -halfSize, halfSize);
        glVertex3f(halfSize, -halfSize, halfSize);
        glVertex3f(halfSize, halfSize, halfSize);
        glVertex3f(-halfSize, halfSize, halfSize);
        // Back face
        glVertex3f(-halfSize, -halfSize, -halfSize);
        glVertex3f(-halfSize, halfSize, -halfSize);
        glVertex3f(halfSize, halfSize, -halfSize);
        glVertex3f(halfSize, -halfSize, -halfSize);
        // Top face
        glVertex3f(-halfSize, halfSize, -halfSize);
        glVertex3f(-halfSize, halfSize, halfSize);
        glVertex3f(halfSize, halfSize, halfSize);
        glVertex3f(halfSize, halfSize, -halfSize);
        // Bottom face
        glVertex3f(-halfSize, -halfSize, -halfSize);
        glVertex3f(halfSize, -halfSize, -halfSize);
        glVertex3f(halfSize, -halfSize, halfSize);
        glVertex3f(-halfSize, -halfSize, halfSize);
        // Right face
        glVertex3f(halfSize, -halfSize, -halfSize);
        glVertex3f(halfSize, halfSize, -halfSize);
        glVertex3f(halfSize, halfSize, halfSize);
        glVertex3f(halfSize, -halfSize, halfSize);
        // Left face
        glVertex3f(-halfSize, -halfSize, -halfSize);
        glVertex3f(-halfSize, -halfSize, halfSize);
        glVertex3f(-halfSize, halfSize, halfSize);
        glVertex3f(-halfSize, halfSize, -halfSize);
        glEnd();

        // Draw block borders
        glColor3f(0.0f, 0.0f, 0.0f); // Black color for borders
        glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
        glLineWidth(2.0f);

        glBegin(GL_QUADS);
        // Re-draw for lines (same vertices)
        // Front face
        glVertex3f(-halfSize, -halfSize, halfSize);
        glVertex3f(halfSize, -halfSize, halfSize);
        glVertex3f(halfSize, halfSize, halfSize);
        glVertex3f(-halfSize, halfSize, halfSize);
        // Back face
        glVertex3f(-halfSize, -halfSize, -halfSize);
        glVertex3f(-halfSize, halfSize, -halfSize);
        glVertex3f(halfSize, halfSize, -halfSize);
        glVertex3f(halfSize, -halfSize, -halfSize);
        // Top face
        glVertex3f(-halfSize, halfSize, -halfSize);
        glVertex3f(-halfSize, halfSize, halfSize);
        glVertex3f(halfSize, halfSize, halfSize);
        glVertex3f(halfSize, halfSize, -halfSize);
        // Bottom face
        glVertex3f(-halfSize, -halfSize, -halfSize);
        glVertex3f(halfSize, -halfSize, -halfSize);
        glVertex3f(halfSize, -halfSize, halfSize);
        glVertex3f(-halfSize, -halfSize, halfSize);
        // Right face
        glVertex3f(halfSize, -halfSize, -halfSize);
        glVertex3f(halfSize, halfSize, -halfSize);
        glVertex3f(halfSize, halfSize, halfSize);
        glVertex3f(halfSize, -halfSize, halfSize);
        // Left face
        glVertex3f(-halfSize, -halfSize, -halfSize);
        glVertex3f(-halfSize, -halfSize, halfSize);
        glVertex3f(-halfSize, halfSize, halfSize);
        glVertex3f(-halfSize, halfSize, -halfSize);
        glEnd();

        glPolygonMode(GL_FRONT_AND_BACK, GL_FILL); // Reset to fill mode
    }


    public void renderPauseScreen() {
        if (fontRenderer == null) return;

        prepare2DDrawing();

        String pauseText = "PAUSE";
        float textScale = 3.0f;
        float fontSize = fontRenderer.getCurrentFontSize();

        // Approximate text dimensions for centering
        float approxCharWidth = fontSize * 0.6f; // Rough estimate
        float textWidthUnscaled = pauseText.length() * approxCharWidth;
        // float textHeightUnscaled = fontSize; // Baseline to top of typical char

        float textWidthScaled = textWidthUnscaled * textScale;
        // float textHeightScaled = textHeightUnscaled * textScale; // Not strictly needed for y if using baseline

        float drawX = (window.getWidth() - textWidthScaled) / 2.0f;
        // For STBTT, y is typically baseline. If we want to center vertically based on font's ascent/descent,
        // we'd need more font metrics. For simplicity, center based on font size.
        float drawY = (window.getHeight() / 2.0f) - (fontSize * textScale / 2.0f) + (fontSize * textScale); // Shift to center better

        glPushMatrix();
        glTranslatef(drawX, drawY, 0);
        glScalef(textScale, textScale, 1.0f);
        fontRenderer.drawString(0, 0, pauseText, 1.0f, 1.0f, 1.0f, 1.0f); // White text
        glPopMatrix();

        finish2DDrawing();
    }

    public void renderDebugInfo() {
        if (fontRenderer == null || camera == null) return;

        prepare2DDrawing();
        calculateFPS();

        String fpsText = String.format("FPS: %.2f", fps);
        String posText = String.format("X: %.2f Y: %.2f Z: %.2f", camera.getX(), camera.getY(), camera.getZ());
        String rotText = String.format("Yaw: %.1f Pitch: %.1f", camera.getYaw(), camera.getPitch());
        String openGLVersionText = "OpenGL: " + GL11.glGetString(GL_VERSION); // Direct call ok here

        float xPos = 10.0f;
        float yPos = 10.0f + fontRenderer.getCurrentFontSize(); // Start Y at first line's baseline
        float lineHeight = fontRenderer.getCurrentFontSize() + 4.0f;

        fontRenderer.drawString(xPos, yPos, fpsText, 1.0f, 1.0f, 1.0f, 1.0f);
        yPos += lineHeight;
        fontRenderer.drawString(xPos, yPos, posText, 1.0f, 1.0f, 1.0f, 1.0f);
        yPos += lineHeight;
        fontRenderer.drawString(xPos, yPos, rotText, 1.0f, 1.0f, 1.0f, 1.0f);
        yPos += lineHeight;
        fontRenderer.drawString(xPos, yPos, openGLVersionText, 1.0f, 1.0f, 1.0f, 1.0f);

        finish2DDrawing();
    }

    private void calculateFPS() {
        double currentTime = glfwGetTime();
        double deltaTime = currentTime - lastFrameTime;
        lastFrameTime = currentTime;
        timeAccumulator += deltaTime;
        frames++;
        if (timeAccumulator >= 1.0) {
            fps = frames / timeAccumulator;
            frames = 0;
            timeAccumulator -= 1.0;
        }
    }

    private void prepare2DDrawing() {
        glMatrixMode(GL_PROJECTION);
        glPushMatrix();
        glLoadIdentity();
        glOrtho(0.0, window.getWidth(), window.getHeight(), 0.0, -1.0, 1.0); // Top-left origin
        glMatrixMode(GL_MODELVIEW);
        glPushMatrix();
        glLoadIdentity();
        glDisable(GL_DEPTH_TEST); // Disable depth testing for 2D overlay
    }

    private void finish2DDrawing() {
        glEnable(GL_DEPTH_TEST); // Re-enable depth test
        glMatrixMode(GL_PROJECTION);
        glPopMatrix();
        glMatrixMode(GL_MODELVIEW);
        glPopMatrix();
    }

    public void onWindowResize(int width, int height) {
        // This can be called by Window's framebuffer resize callback
        // or by Main/Game if it coordinates this.
        setupProjection(); // Recalculate projection matrix with new aspect ratio
    }

    public void cleanup() {
        // If Renderer itself created OpenGL resources (e.g., shaders, VBOs), clean them here.
        // FontRenderer has its own cleanup.
        if (fontRenderer != null) {
            fontRenderer.cleanup();
        }
    }
}
