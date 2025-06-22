package com.kacpersledz.majn.game; // Corrected package

// Static imports for GLFW and OpenGL are now mostly within Window, Renderer, FontRenderer
import static org.lwjgl.glfw.GLFW.glfwGetTime; // Retained for main loop timing if needed, though Renderer handles FPS
import static org.lwjgl.opengl.GL11.glGetString; // For OpenGL version logging
import static org.lwjgl.opengl.GL11.GL_VERSION;  // Added import for GL_VERSION
// GL11 imports are no longer needed here, Renderer handles all GL calls.
// GL12 imports (GL_CLAMP_TO_EDGE) are in FontRenderer.
// MemoryStack.stackPush might be used if Main does any direct JOM/LWJGL struct work, but unlikely now.

import com.kacpersledz.majn.view.Window;
import com.kacpersledz.majn.controller.InputHandler;
import com.kacpersledz.majn.view.Renderer;      // Import Renderer
import com.kacpersledz.majn.view.FontRenderer;  // Import FontRenderer
import java.io.IOException; // For FontRenderer creation
// Other specific imports like File, FileInputStream, ByteBuffer, etc. are no longer needed in Main
// Collection, Matrix4f, STB classes, MemoryUtil are also not directly used in Main anymore.
import org.lwjgl.Version;
import com.kacpersledz.majn.controller.Camera;
import com.kacpersledz.majn.world.Chunk; // Still needed for initial camera positioning.
import com.kacpersledz.majn.world.World;
// Block is used by Renderer.

/**
 * @author Paul Nelson Baker
 * @see <a href="https://github.com/paul-nelson-baker/">GitHub</a>
 * @see <a href="https://www.linkedin.com/in/paul-n-baker/">LinkedIn</a>
 * @since 2019-05
 *        <p>
 *        Modified from <a href="https://www.lwjgl.org/guide">original example</a>
 */
public class Game implements Runnable { // Renamed from Main

  // Window will be managed by the Window class
  private Window window;
  private InputHandler inputHandler;
  private Renderer renderer;
  private FontRenderer fontRenderer;


  private World world;
  private Camera camera;

  public static void main(String... args) {
    Game gameApp = new Game();
    gameApp.run();
  }

  public void run() {
    init();
    loop();
    cleanup();
  }

  public void init() {
    System.out.println("Starting LWJGL " + Version.getVersion());

    window = new Window(this);
    window.init();

    this.camera = new Camera(Chunk.CHUNK_WIDTH / 2.0f, Chunk.CHUNK_HEIGHT / 2.0f + 3.0f,
        Chunk.CHUNK_DEPTH / 2.0f + 5.0f);

    this.world = new World();

    try {
      fontRenderer = new FontRenderer("fonts/MajnFont.otf", 15.0f, 512, 512);
    } catch (IOException e) {
      System.err.println("Failed to load font. Text rendering will not work.");
      e.printStackTrace();
    }

    inputHandler = new InputHandler(camera, window);

    window.setInputHandler(inputHandler);

    renderer = new Renderer(window, camera, world, fontRenderer);
    renderer.init();

    System.out.println("OpenGL: " + glGetString(GL_VERSION));

    window.setCursorDisabled(true);
  }

  private void processInputAndUpdateCamera() {
    if (camera == null || inputHandler == null) {
      return;
    }

    float forward = 0.0f, right = 0.0f, up = 0.0f;
    if (inputHandler.isMoveForward()) forward += Camera.MOVE_SPEED;
    if (inputHandler.isMoveBackward()) forward -= Camera.MOVE_SPEED;
    if (inputHandler.isMoveLeft()) right -= Camera.MOVE_SPEED;
    if (inputHandler.isMoveRight()) right += Camera.MOVE_SPEED;
    if (inputHandler.isMoveUp()) up += Camera.MOVE_SPEED;
    if (inputHandler.isMoveDown()) up -= Camera.MOVE_SPEED;

    if (forward != 0 || right != 0 || up != 0) {
      camera.moveRelative(forward, right, up, this.world);
    }
  }


  public void loop() {
    while (!window.shouldClose()) {
        boolean currentIsPaused = inputHandler.isPaused();

        if (!currentIsPaused) {
            processInputAndUpdateCamera();
        }

        renderer.clear();
        renderer.render();

        if (currentIsPaused) {
            renderer.renderPauseScreen();
        } else if (inputHandler.isShowingDebugInfo()) {
            renderer.renderDebugInfo();
        }

        window.swapBuffers();
        window.pollEvents();
    }
  }

  public void onWindowResize(int width, int height) {
    if (renderer != null) {
        renderer.onWindowResize(width, height);
    }
  }

  private void cleanup() {
    System.out.println("Cleaning up resources...");
    if (renderer != null) {
        renderer.cleanup();
    }
    if (window != null) {
        window.close();
    }
  }
}
