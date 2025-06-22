package com.kacpersledz.majn;

import static org.lwjgl.glfw.Callbacks.glfwFreeCallbacks;
import static org.lwjgl.glfw.GLFW.GLFW_CURSOR;
import static org.lwjgl.glfw.GLFW.GLFW_CURSOR_DISABLED;
import static org.lwjgl.glfw.GLFW.GLFW_FALSE;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_A;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_D;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_ESCAPE;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_F;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_F3;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_S;
import static org.lwjgl.glfw.GLFW.GLFW_KEY_SPACE;
// Added these imports for camera control
import static org.lwjgl.glfw.GLFW.GLFW_KEY_W;
import static org.lwjgl.glfw.GLFW.GLFW_PRESS;
import static org.lwjgl.glfw.GLFW.GLFW_RELEASE;
import static org.lwjgl.glfw.GLFW.GLFW_RESIZABLE;
import static org.lwjgl.glfw.GLFW.GLFW_TRUE;
import static org.lwjgl.glfw.GLFW.GLFW_VISIBLE;
import static org.lwjgl.glfw.GLFW.glfwCreateWindow;
import static org.lwjgl.glfw.GLFW.glfwDefaultWindowHints;
import static org.lwjgl.glfw.GLFW.glfwDestroyWindow;
import static org.lwjgl.glfw.GLFW.glfwGetPrimaryMonitor;
import static org.lwjgl.glfw.GLFW.glfwGetTime;
import static org.lwjgl.glfw.GLFW.glfwGetVideoMode;
import static org.lwjgl.glfw.GLFW.glfwGetWindowSize;
import static org.lwjgl.glfw.GLFW.glfwInit;
import static org.lwjgl.glfw.GLFW.glfwMakeContextCurrent;
import static org.lwjgl.glfw.GLFW.glfwPollEvents;
import static org.lwjgl.glfw.GLFW.glfwSetCursorPosCallback;
import static org.lwjgl.glfw.GLFW.glfwSetErrorCallback;
import static org.lwjgl.glfw.GLFW.glfwSetInputMode;
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
// glColor3f is already imported, but explicitly noting for text rendering context
// For GL constants and functions
import static org.lwjgl.opengl.GL11.GL_BLEND;
import static org.lwjgl.opengl.GL11.GL_COLOR_BUFFER_BIT;
import static org.lwjgl.opengl.GL11.GL_DEPTH_BUFFER_BIT;
import static org.lwjgl.opengl.GL11.GL_DEPTH_TEST;
import static org.lwjgl.opengl.GL11.GL_FILL;
import static org.lwjgl.opengl.GL11.GL_FRONT_AND_BACK;
import static org.lwjgl.opengl.GL11.GL_LINE;
import static org.lwjgl.opengl.GL11.GL_LINEAR;
import static org.lwjgl.opengl.GL11.GL_MODELVIEW;
import static org.lwjgl.opengl.GL11.GL_ONE_MINUS_SRC_ALPHA;
import static org.lwjgl.opengl.GL11.GL_PROJECTION;
import static org.lwjgl.opengl.GL11.GL_QUADS;
import static org.lwjgl.opengl.GL11.GL_RED;
import static org.lwjgl.opengl.GL11.GL_SRC_ALPHA;
import static org.lwjgl.opengl.GL11.GL_TEXTURE_2D;
import static org.lwjgl.opengl.GL11.GL_TEXTURE_MAG_FILTER;
import static org.lwjgl.opengl.GL11.GL_TEXTURE_MIN_FILTER;
import static org.lwjgl.opengl.GL11.GL_TEXTURE_WRAP_S;
import static org.lwjgl.opengl.GL11.GL_TEXTURE_WRAP_T;
import static org.lwjgl.opengl.GL11.GL_UNPACK_ALIGNMENT;
import static org.lwjgl.opengl.GL11.GL_UNSIGNED_BYTE;
import static org.lwjgl.opengl.GL11.GL_VERSION;
import static org.lwjgl.opengl.GL11.glBegin;
import static org.lwjgl.opengl.GL11.glBindTexture;
import static org.lwjgl.opengl.GL11.glBlendFunc;
import static org.lwjgl.opengl.GL11.glClear;
import static org.lwjgl.opengl.GL11.glClearColor;
import static org.lwjgl.opengl.GL11.glColor3f;
import static org.lwjgl.opengl.GL11.glColor4f;
import static org.lwjgl.opengl.GL11.glDisable;
import static org.lwjgl.opengl.GL11.glEnable;
import static org.lwjgl.opengl.GL11.glEnd;
import static org.lwjgl.opengl.GL11.glGenTextures;
import static org.lwjgl.opengl.GL11.glGetString;
import static org.lwjgl.opengl.GL11.glLineWidth;
import static org.lwjgl.opengl.GL11.glLoadIdentity;
import static org.lwjgl.opengl.GL11.glLoadMatrixf; // Added this line
import static org.lwjgl.opengl.GL11.glMatrixMode;
import static org.lwjgl.opengl.GL11.glOrtho;
import static org.lwjgl.opengl.GL11.glPixelStorei;
import static org.lwjgl.opengl.GL11.glPolygonMode;
import static org.lwjgl.opengl.GL11.glPopMatrix;
import static org.lwjgl.opengl.GL11.glPushMatrix;
import static org.lwjgl.opengl.GL11.glTexCoord2f;
import static org.lwjgl.opengl.GL11.glTexImage2D;
import static org.lwjgl.opengl.GL11.glTexParameteri;
import static org.lwjgl.opengl.GL11.glTranslatef;
import static org.lwjgl.opengl.GL11.glVertex2f;
import static org.lwjgl.opengl.GL11.glVertex3f;
// For GL_CLAMP_TO_EDGE if used
import static org.lwjgl.opengl.GL12.GL_CLAMP_TO_EDGE;
import static org.lwjgl.system.MemoryStack.stackPush;
import static org.lwjgl.system.MemoryUtil.NULL;
import java.io.File; // For loading from file path (alternative)
import java.io.FileInputStream; // For loading from file path (alternative)
import java.io.IOException;
import java.io.InputStream; // For loading from classpath
import java.nio.ByteBuffer;
import java.nio.FloatBuffer;
import java.nio.IntBuffer;
import java.nio.channels.Channels;
import java.nio.channels.FileChannel;
import java.nio.channels.ReadableByteChannel;
import org.joml.Matrix4f; // Added for JOML
import org.lwjgl.Version;
import org.lwjgl.glfw.GLFWVidMode;
import org.lwjgl.stb.STBTTAlignedQuad;
import org.lwjgl.stb.STBTTFontinfo;
import org.lwjgl.stb.STBTTPackContext;
import org.lwjgl.stb.STBTTPackedchar;
import org.lwjgl.stb.STBTruetype; // Provides stbtt_* static methods
import org.lwjgl.system.MemoryStack; // Already here but good to note for glLoadMatrixf
import org.lwjgl.system.MemoryUtil; // For MemoryUtil.memAllocFloat, etc. if needed directly
import com.kacpersledz.majn.controller.Camera;
import com.kacpersledz.majn.world.Block;
import com.kacpersledz.majn.world.Chunk;
import com.kacpersledz.majn.world.World;

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
  private boolean showDebugInfo = false;

  // TrueType Font data
  private static ByteBuffer ttf;
  private static STBTTFontinfo fontInfo;
  private static STBTTPackedchar.Buffer charData;
  private static int fontAtlasTextureId = -1; // Initialize to -1
  private static float currentFontSize;
  private static final int BITMAP_W = 512; // Width of the font atlas texture
  private static final int BITMAP_H = 512; // Height of the font atlas texture

  // Time tracking for FPS
  private double lastFrameTime = 0.0;
  private int frames = 0;
  private double timeAccumulator = 0.0;
  private double fps = 0.0;

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
    this.camera = new Camera(Chunk.CHUNK_WIDTH / 2.0f, Chunk.CHUNK_HEIGHT / 2.0f + 3.0f,
        Chunk.CHUNK_DEPTH / 2.0f + 5.0f);

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
      if (key == GLFW_KEY_F3 && action == GLFW_RELEASE) {
        showDebugInfo = !showDebugInfo;
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
          case GLFW_KEY_F:
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
    lastFrameTime = glfwGetTime(); // Initialize lastFrameTime for FPS calculation

    try {
      loadTruetypeFont("fonts/MajnFont.otf", 15.0f); // Using 15.0f as a default size
    } catch (IOException e) {
      System.err.println("Failed to load TrueType font:");
      e.printStackTrace();
      // Optionally, set a flag to not attempt rendering text or exit
    }
  }

  private static ByteBuffer ioResourceToByteBuffer(String resource, int bufferSize)
      throws IOException {
    ByteBuffer buffer;
    File file = new File(resource);
    if (file.isFile()) {
      try (FileInputStream fis = new FileInputStream(file);
          FileChannel fc = fis.getChannel()) {
        buffer = MemoryUtil.memAlloc((int) fc.size() + 1); // Use MemoryUtil for direct buffer
        while (fc.read(buffer) != -1) {
          ;
        }
      }
    } else {
      // Try loading from classpath
      try (InputStream source = Main.class.getClassLoader().getResourceAsStream(resource);
          ReadableByteChannel rbc = Channels.newChannel(source)) {
        buffer = MemoryUtil.memAlloc(bufferSize); // Allocate bufferSize
        if (source == null) {
          throw new IOException("Resource not found: " + resource);
        }
        while (true) {
          int bytes = rbc.read(buffer);
          if (bytes == -1) {
            break;
          }
          if (buffer.remaining() == 0) {
            // Option to reallocate a larger buffer if needed, or ensure bufferSize is
            // adequate
            // For simplicity, assume bufferSize is large enough or handle error
            ByteBuffer newBuffer = MemoryUtil.memRealloc(buffer, buffer.capacity() * 2);
            if (newBuffer == null) {
              throw new OutOfMemoryError();
            }
            buffer = newBuffer;
          }
        }
      }
    }
    buffer.flip();
    return buffer;
  }

  private static void loadTruetypeFont(String otfResourcePath, float fontSize) throws IOException {
    fontInfo = STBTTFontinfo.create();
    // Assuming charData will be created and populated in the next step (glyph
    // packing)
    // charData = STBTTPackedchar.create(96); // For ASCII 32-126 (95 chars + 1
    // for safety/indexing)

    // Load the font file.
    // Using a buffer size of 150KB for the font, adjust if necessary.
    // Common .otf files are often < 150KB.
    ttf = ioResourceToByteBuffer(otfResourcePath, 150 * 1024);

    if (!STBTruetype.stbtt_InitFont(fontInfo, ttf)) {
      throw new IllegalStateException("Failed to initialize font information.");
    }

    currentFontSize = fontSize;

    // Initialize charData buffer for 96 characters (ASCII 32-126)
    charData = STBTTPackedchar.create(96);

    // Create a bitmap for the font atlas
    ByteBuffer bitmap = MemoryUtil.memAlloc(BITMAP_W * BITMAP_H); // Grayscale bitmap

    STBTTPackContext pc = STBTTPackContext.create();
    try {
      STBTruetype.stbtt_PackBegin(pc, bitmap, BITMAP_W, BITMAP_H, 0, 1, MemoryUtil.NULL);
      // Pack characters from ASCII 32 (space) to 126 (~)
      // The null for stbtt_PackSetOversampling indicates no oversampling.
      // You could use stbtt_PackSetOversampling(pc, H_OVERSAMPLE, V_OVERSAMPLE)
      // before stbtt_PackFontRange for antialiasing.
      // For simplicity, no oversampling first.
      if (!STBTruetype.stbtt_PackFontRange(pc, ttf, 0, currentFontSize, 32, charData)) {
        throw new IllegalStateException("Failed to pack font characters.");
      }
    } finally {
      STBTruetype.stbtt_PackEnd(pc);
      pc.free(); // Free the pack context
    }

    // Create OpenGL texture for the font atlas
    fontAtlasTextureId = glGenTextures();
    glBindTexture(GL_TEXTURE_2D, fontAtlasTextureId);

    // We are creating a grayscale (alpha) texture
    // GL_ALPHA or GL_RED are common choices for single-channel textures.
    // If using GL_RED, shaders might need to swizzle .r to .a (e.g., color.a =
    // texture(tex, uv).r)
    // For fixed-function pipeline compatibility with glColor, GL_ALPHA is often
    // easier.
    // However, GL_ALPHA textures are sometimes deprecated or less performant in
    // modern GL.
    // Let's try GL_RED first, as it's common for single channel data in modern GL.
    // If issues arise, GL_ALPHA could be an alternative, or converting bitmap to
    // RGBA.
    glPixelStorei(GL_UNPACK_ALIGNMENT, 1); // Important for single-channel textures
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RED, BITMAP_W, BITMAP_H, 0, GL_RED, GL_UNSIGNED_BYTE, bitmap);

    // Set texture parameters
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);

    MemoryUtil.memFree(bitmap); // Free the CPU-side bitmap now that it's on the GPU
  }

  private void setupProjection() {
    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();
    Matrix4f projectionMatrix = new Matrix4f();
    float aspectRatio = (float) windowWidth / windowHeight;
    projectionMatrix.perspective((float) Math.toRadians(60.0f), aspectRatio, 0.1f, 500.0f);

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

      if (showDebugInfo) {
        renderDebugInfo();
      }

      glfwSwapBuffers(windowHandle);
      glfwPollEvents();
    }
  }

  private void renderDebugInfo() {
    // Switch to Orthographic Projection
    int[] w = new int[1], h = new int[1];
    glfwGetWindowSize(windowHandle, w, h);
    int width = w[0];
    int height = h[0];

    glMatrixMode(GL_PROJECTION);
    glPushMatrix();
    glLoadIdentity();
    glOrtho(0.0, width, height, 0.0, -1.0, 1.0); // Top-left origin
    glMatrixMode(GL_MODELVIEW);
    glPushMatrix();
    glLoadIdentity();
    glDisable(GL_DEPTH_TEST); // Disable depth testing for 2D overlay

    // Calculate FPS
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

    // Prepare Debug Strings
    String fpsText = String.format("FPS: %.2f", fps);
    String posText =
        String.format("X: %.2f Y: %.2f Z: %.2f", camera.getX(), camera.getY(), camera.getZ());
    String rotText = String.format("Yaw: %.1f Pitch: %.1f", camera.getYaw(), camera.getPitch());
    String openGLVersionText = "OpenGL: " + glGetString(GL_VERSION);

    // Set text color to white, fully opaque
    glColor4f(1.0f, 1.0f, 1.0f, 1.0f);

    // Render Text using drawString with proper line spacing
    float xPos = 10.0f; // Starting X position for all lines
    float yPos = 10.0f; // Starting Y position for the first line
    // Use currentFontSize as an approximate line height.
    // Add a small factor for padding, e.g., currentFontSize * 1.2f or
    // currentFontSize + 4
    float lineHeight = currentFontSize + 4.0f;

    drawString(xPos, yPos, fpsText);
    yPos += lineHeight;
    drawString(xPos, yPos, posText);
    yPos += lineHeight;
    drawString(xPos, yPos, rotText);
    yPos += lineHeight;
    drawString(xPos, yPos, openGLVersionText);

    // Restore Previous Projection and State
    glEnable(GL_DEPTH_TEST);
    glMatrixMode(GL_PROJECTION);
    glPopMatrix();
    glMatrixMode(GL_MODELVIEW);
    glPopMatrix();
  }

  private void drawString(float x, float y, String text) {
    if (fontAtlasTextureId == -1 || charData == null) {
      System.err.println("Font not loaded, cannot drawString.");
      return; // Font not loaded
    }

    glEnable(GL_TEXTURE_2D);
    glBindTexture(GL_TEXTURE_2D, fontAtlasTextureId);

    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    // Note: glColor4f should be called before drawString to set text color and
    // alpha.
    // The GL_RED texture will be treated as alpha by the blending function
    // when used with glColor's components.

    try (MemoryStack stack = MemoryStack.stackPush()) {
      FloatBuffer xPos = stack.floats(x);
      FloatBuffer yPos = stack.floats(y);
      STBTTAlignedQuad q = STBTTAlignedQuad.mallocStack(stack);

      glBegin(GL_QUADS);
      for (int i = 0; i < text.length(); i++) {
        char character = text.charAt(i);
        if (character < 32 || character >= 127) { // charData has 95 entries (32 to 126)
          // Replace unsupported characters with a default, e.g., '?' (ASCII 63)
          // Ensure '?' is within your packed range. ASCII 63 is index 63-32 = 31.
          character = '?';
        }

        // stbtt_GetPackedQuad advances xPos and yPos according to font metrics
        STBTruetype.stbtt_GetPackedQuad(charData, BITMAP_W, BITMAP_H, character - 32, xPos, yPos, q,
            false);

        glTexCoord2f(q.s0(), q.t0());
        glVertex2f(q.x0(), q.y0());
        glTexCoord2f(q.s1(), q.t0());
        glVertex2f(q.x1(), q.y0());
        glTexCoord2f(q.s1(), q.t1());
        glVertex2f(q.x1(), q.y1());
        glTexCoord2f(q.s0(), q.t1());
        glVertex2f(q.x0(), q.y1());
      }
      glEnd();
    }

    glDisable(GL_BLEND);
    glDisable(GL_TEXTURE_2D);
  }

  @Override
  public void close() {
    glfwFreeCallbacks(windowHandle);
    glfwDestroyWindow(windowHandle);
    glfwTerminate();
    glfwSetErrorCallback(null).free();
  }
}
