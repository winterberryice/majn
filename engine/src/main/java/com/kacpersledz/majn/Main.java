package com.kacpersledz.majn;

import static org.lwjgl.glfw.Callbacks.glfwFreeCallbacks;
import static org.lwjgl.glfw.GLFW.GLFW_FALSE;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_ESCAPE;
import static org.lwjgl.glfw.GLFW.GLFW_RELEASE;
import static org.lwjgl.glfw.GLFW.GLFW_RESIZABLE;
// Added these imports for camera control
import static org.lwjgl.glfw.GLFW.GLFW_KEY_W;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_S;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_A;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_D;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_SPACE;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_LEFT_SHIFT;
import static org.lwjgl.glfw.GLFW.GLFW_PRESS;
import static org.lwjgl.glfw.GLFW.glfwSetCursorPosCallback;
import static org.lwjgl.glfw.GLFW.glfwSetInputMode;
import static org.lwjgl.glfw.GLFW.GLFW_CURSOR;
import static org.lwjgl.glfw.GLFW.GLFW_CURSOR_DISABLED;
import static org.lwjgl.glfw.GLFW.GLFW_TRUE;
import static org.lwjgl.glfw.GLFW.GLFW_VISIBLE;
import static org.lwjgl.glfw.GLFW.glfwCreateWindow;
import static org.lwjgl.glfw.GLFW.glfwDefaultWindowHints;
import static org.lwjgl.glfw.GLFW.glfwDestroyWindow;
import static org.lwjgl.glfw.GLFW.glfwGetPrimaryMonitor;
import static org.lwjgl.glfw.GLFW.glfwGetVideoMode;
import static org.lwjgl.glfw.GLFW.glfwGetWindowSize;
import static org.lwjgl.glfw.GLFW.glfwInit;
import static org.lwjgl.glfw.GLFW.glfwMakeContextCurrent;
import static org.lwjgl.glfw.GLFW.glfwPollEvents;
import static org.lwjgl.glfw.GLFW.glfwSetErrorCallback;
import static org.lwjgl.glfw.GLFW.glfwSetKeyCallback;
import static org.lwjgl.glfw.GLFW.glfwSetWindowPos;
import static org.lwjgl.glfw.GLFW.glfwSetWindowShouldClose;
import static org.lwjgl.glfw.GLFW.glfwShowWindow;
import static org.lwjgl.glfw.GLFW.glfwSwapBuffers;
import static org.lwjgl.glfw.GLFW.glfwSwapInterval;
import static org.lwjgl.glfw.GLFW.glfwTerminate;
import static org.lwjgl.glfw.GLFW.glfwWindowHint;
import static org.lwjgl.glfw.GLFW.glfwWindowShouldClose;
import static org.lwjgl.glfw.GLFWErrorCallback.createPrint;
import static org.lwjgl.opengl.GL.createCapabilities;
import static org.lwjgl.opengl.GL11.GL_COLOR_BUFFER_BIT;
import static org.lwjgl.opengl.GL11.GL_DEPTH_BUFFER_BIT;
import static org.lwjgl.opengl.GL11.GL_DEPTH_TEST;
import static org.lwjgl.opengl.GL11.GL_FILL;
import static org.lwjgl.opengl.GL11.GL_FRONT_AND_BACK;
import static org.lwjgl.opengl.GL11.GL_LINE;
import static org.lwjgl.opengl.GL11.GL_MODELVIEW;
import static org.lwjgl.opengl.GL11.GL_PROJECTION;
import static org.lwjgl.opengl.GL11.GL_QUADS;
import static org.lwjgl.opengl.GL11.GL_VERSION;
import static org.lwjgl.opengl.GL11.glBegin;
import static org.lwjgl.opengl.GL11.glClear;
import static org.lwjgl.opengl.GL11.glClearColor;
import static org.lwjgl.opengl.GL11.glColor3f;
import static org.lwjgl.opengl.GL11.glEnable;
import static org.lwjgl.opengl.GL11.glEnd;
import static org.lwjgl.opengl.GL11.glGetString;
import static org.lwjgl.opengl.GL11.glLineWidth;
import static org.lwjgl.opengl.GL11.glLoadIdentity;
import static org.lwjgl.opengl.GL11.glLoadMatrixf; // Added this line
import static org.lwjgl.opengl.GL11.glMatrixMode;
import static org.lwjgl.opengl.GL11.glOrtho;
import static org.lwjgl.opengl.GL11.glPolygonMode;
import static org.lwjgl.opengl.GL11.glPopMatrix;
import static org.lwjgl.opengl.GL11.glPushMatrix;
import static org.lwjgl.opengl.GL11.glRotatef;
import static org.lwjgl.opengl.GL11.glTranslatef;
import static org.lwjgl.opengl.GL11.glVertex3f;
import static org.lwjgl.system.MemoryStack.stackPush;
import static org.lwjgl.system.MemoryUtil.NULL;

import java.nio.IntBuffer;
import org.lwjgl.Version;
import org.lwjgl.glfw.GLFWVidMode;
import com.kacpersledz.majn.controller.Camera;
import org.joml.Matrix4f; // Added for JOML
import com.kacpersledz.majn.world.Block;
import com.kacpersledz.majn.world.Chunk;
import com.kacpersledz.majn.world.World;
import org.lwjgl.system.MemoryStack; // Already here but good to note for glLoadMatrixf

/**
 * @author Paul Nelson Baker
 * @see <a href="https://github.com/paul-nelson-baker/">GitHub</a>
 * @see <a href="https://www.linkedin.com/in/paul-n-baker/">LinkedIn</a>
 * @since 2019-05
 *        <p>
 *        Modified from <a href="https://www.lwjgl.org/guide">original example</a>
 */
public class Main implements AutoCloseable, Runnable {

  private static final String windowTitle = "Hello, World!";
  private static final int windowWidth = 300;
  private static final int windowHeight = 300;
  private long windowHandle;
  private World world;
  private Camera camera;

  // Camera movement state
  private boolean moveForward;
  private boolean moveBackward;
  private boolean moveLeft;
  private boolean moveRight;
  private boolean moveUp;
  private boolean moveDown;

  // Mouse position state
  private double lastMouseX = windowWidth / 2.0;
  private double lastMouseY = windowHeight / 2.0;
  private boolean firstMouse = true;

  public static void main(String... args) {
    try (Main main = new Main()) {
      main.run();
    }
  }

  /**
   * Convienience method that also satisfies Runnable
   */
  public void run() {
    init();
    loop();
  }

  public void init() {
    this.world = new World();

    // Initialize camera at a starting position
    this.camera = new Camera(Chunk.CHUNK_WIDTH / 2.0f, Chunk.CHUNK_HEIGHT / 2.0f + 3.0f, Chunk.CHUNK_DEPTH / 2.0f + 5.0f);

    createPrint(System.err).set();
    System.out.println("Starting LWJGL " + Version.getVersion());
    if (!glfwInit()) {
      throw new IllegalStateException("Unable to initialize GLFW");
    }
    glfwDefaultWindowHints();
    glfwWindowHint(GLFW_VISIBLE, GLFW_FALSE);
    glfwWindowHint(GLFW_RESIZABLE, GLFW_TRUE);
    windowHandle = glfwCreateWindow(windowWidth, windowHeight, windowTitle, NULL, NULL);
    if (windowHandle == NULL) {
      throw new RuntimeException("Failed to create the GLFW window");
    }
    glfwSetKeyCallback(windowHandle, (window, key, scancode, action, mods) -> {
      if (key == GLFW_KEY_ESCAPE && action == GLFW_RELEASE) {
        glfwSetWindowShouldClose(window, true);
      }
      // Camera movement keys
      if (action == GLFW_PRESS || action == GLFW_RELEASE) {
        boolean pressed = (action == GLFW_PRESS);
        switch (key) {
          case GLFW_KEY_W:
            moveForward = pressed;
            break;
          case GLFW_KEY_S:
            moveBackward = pressed;
            break;
          case GLFW_KEY_A:
            moveLeft = pressed;
            break;
          case GLFW_KEY_D:
            moveRight = pressed;
            break;
          case GLFW_KEY_SPACE:
            moveUp = pressed;
            break;
          case GLFW_KEY_LEFT_SHIFT:
            moveDown = pressed;
            break;
        }
      }
    });

    glfwSetCursorPosCallback(windowHandle, (window, xpos, ypos) -> {
      if (firstMouse) {
        lastMouseX = xpos;
        lastMouseY = ypos;
        firstMouse = false;
      }

      float xoffset = (float) (xpos - lastMouseX);
      float yoffset = (float) (ypos - lastMouseY); // Corrected for non-inverted pitch

      lastMouseX = xpos;
      lastMouseY = ypos;

      if (camera != null) { // Ensure camera is initialized
        // Pass dyaw (xoffset) and dpitch (yoffset)
        camera.rotate(yoffset, xoffset);
      }
    });

    try (MemoryStack stack = stackPush()) {
      IntBuffer pWidth = stack.mallocInt(1);
      IntBuffer pHeight = stack.mallocInt(1);
      glfwGetWindowSize(windowHandle, pWidth, pHeight);
      GLFWVidMode vidMode = glfwGetVideoMode(glfwGetPrimaryMonitor());
      glfwSetWindowPos(
          windowHandle,
          (vidMode.width() - pWidth.get(0)) / 2,
          (vidMode.height() - pHeight.get(0)) / 2);
    }
    glfwMakeContextCurrent(windowHandle);
    glfwSwapInterval(1);
    glfwShowWindow(windowHandle);

    // Set cursor mode for FPS-like camera
    glfwSetInputMode(windowHandle, GLFW_CURSOR, GLFW_CURSOR_DISABLED);

    createCapabilities();
    System.out.println("OpenGL: " + glGetString(GL_VERSION));
    glEnable(GL_DEPTH_TEST);
    setupProjection();
    glClearColor(0.0f, 0.0f, 0.2f, 0.0f);
  }

  private void setupProjection() {
    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();
    Matrix4f projectionMatrix = new Matrix4f();
    float aspectRatio = (float)windowWidth / windowHeight;
    projectionMatrix.perspective((float)Math.toRadians(60.0f), aspectRatio, 0.1f, 500.0f);

    try (MemoryStack stack = MemoryStack.stackPush()) {
        java.nio.FloatBuffer fb = stack.mallocFloat(16);
        projectionMatrix.get(fb);
        glLoadMatrixf(fb);
    }
    glMatrixMode(GL_MODELVIEW);
  }

  private void renderWorld() {
    if (this.world == null) {
        return;
    }

    // Clear ModelView matrix
    glMatrixMode(GL_MODELVIEW);
    glLoadIdentity();
    camera.applyTransformations(); // Apply camera view

    glEnable(GL_DEPTH_TEST);
    // Define block size (assuming 1x1x1 blocks)
    float blockSize = 1.0f;

    // Render a small portion of the world, e.g., one chunk (0,0,0)
    int chunkToRenderX = 0;
    int chunkToRenderY = 0;
    int chunkToRenderZ = 0;
    Chunk chunkToRender = this.world.getChunk(chunkToRenderX, chunkToRenderY, chunkToRenderZ);

    if (chunkToRender != null) {
        for (int x = 0; x < Chunk.CHUNK_WIDTH; x++) {
            for (int y = 0; y < Chunk.CHUNK_HEIGHT; y++) {
                for (int z = 0; z < Chunk.CHUNK_DEPTH; z++) {
                    Block block = chunkToRender.getBlock(x, y, z);
                    if (block != null && block.getType() != Block.BlockType.AIR) {

                        if (block.getType() == Block.BlockType.DIRT) {
                            glColor3f(0.6f, 0.4f, 0.2f); // Brownish color for DIRT
                        } else if (block.getType() == Block.BlockType.GRASS) {
                            glColor3f(0.0f, 0.8f, 0.0f); // Green color for GRASS
                        } else {
                            glColor3f(0.5f, 0.5f, 0.5f); // Default grey for other types (if any)
                        }

                        float worldX = (chunkToRenderX * Chunk.CHUNK_WIDTH + x) * blockSize;
                        float worldY = (chunkToRenderY * Chunk.CHUNK_HEIGHT + y) * blockSize;
                        float worldZ = (chunkToRenderZ * Chunk.CHUNK_DEPTH + z) * blockSize;

                        glPushMatrix();
                        glTranslatef(worldX, worldY, worldZ);

                        // Draw a cube (same cube drawing code as before)
                        // Front face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, -blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, blockSize / 2);
                        glEnd();
                        // Back face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glEnd();
                        // Top face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, blockSize / 2, -blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, -blockSize / 2);
                        glEnd();
                        // Bottom face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, blockSize / 2);
                        glVertex3f(-blockSize / 2, -blockSize / 2, blockSize / 2);
                        glEnd();
                        // Right face
                        glBegin(GL_QUADS);
                        glVertex3f(blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, blockSize / 2);
                        glEnd();
                        // Left face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(-blockSize / 2, -blockSize / 2, blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, -blockSize / 2);
                        glEnd();

                        // Draw block borders
                        glColor3f(0.0f, 0.0f, 0.0f); // Black color for borders
                        glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
                        glLineWidth(2.0f); // Make lines a bit thicker

                        // Re-draw the cube (lines will be rendered)
                        // Front face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, -blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, blockSize / 2);
                        glEnd();
                        // Back face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glEnd();
                        // Top face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, blockSize / 2, -blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, -blockSize / 2);
                        glEnd();
                        // Bottom face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, blockSize / 2);
                        glVertex3f(-blockSize / 2, -blockSize / 2, blockSize / 2);
                        glEnd();
                        // Right face
                        glBegin(GL_QUADS);
                        glVertex3f(blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, -blockSize / 2);
                        glVertex3f(blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(blockSize / 2, -blockSize / 2, blockSize / 2);
                        glEnd();
                        // Left face
                        glBegin(GL_QUADS);
                        glVertex3f(-blockSize / 2, -blockSize / 2, -blockSize / 2);
                        glVertex3f(-blockSize / 2, -blockSize / 2, blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, blockSize / 2);
                        glVertex3f(-blockSize / 2, blockSize / 2, -blockSize / 2);
                        glEnd();

                        glPolygonMode(GL_FRONT_AND_BACK, GL_FILL); // Reset to fill mode

                        glPopMatrix();
                    }
                }
            }
        }
    }
  }

  private void processInputAndUpdateCamera() {
    if (camera == null) {
      return;
    }

    // Use Camera.MOVE_SPEED directly
    float forward = 0.0f;
    float right = 0.0f;
    float up = 0.0f;

    if (moveForward) {
      forward += Camera.MOVE_SPEED;
    }
    if (moveBackward) {
      forward -= Camera.MOVE_SPEED;
    }
    if (moveLeft) {
      right -= Camera.MOVE_SPEED;
    }
    if (moveRight) {
      right += Camera.MOVE_SPEED;
    }
    if (moveUp) {
      up += Camera.MOVE_SPEED;
    }
    if (moveDown) {
      up -= Camera.MOVE_SPEED;
    }

    if (forward != 0 || right != 0 || up != 0) {
      // Pass the world instance for collision detection
      camera.moveRelative(forward, right, up, this.world);
    }
  }

  public void loop() {
    while (!glfwWindowShouldClose(windowHandle)) {
      // Process input and update camera (before clearing screen and rendering)
      processInputAndUpdateCamera();

      glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
      renderWorld(); // Call renderWorld here
      glfwSwapBuffers(windowHandle);
      glfwPollEvents();
    }
  }

  @Override
  public void close() {
    glfwFreeCallbacks(windowHandle);
    glfwDestroyWindow(windowHandle);
    glfwTerminate();
    glfwSetErrorCallback(null).free();
  }
}
