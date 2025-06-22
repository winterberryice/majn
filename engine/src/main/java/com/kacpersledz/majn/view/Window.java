package com.kacpersledz.majn.view;

import com.kacpersledz.majn.game.Game; // Updated package for Game class
import com.kacpersledz.majn.controller.InputHandler;
import org.lwjgl.glfw.GLFWErrorCallback;
import org.lwjgl.glfw.GLFWVidMode;
import org.lwjgl.opengl.GL;
import org.lwjgl.system.MemoryStack;

import java.nio.IntBuffer;

import static org.lwjgl.glfw.Callbacks.glfwFreeCallbacks;
import static org.lwjgl.glfw.GLFW.*;
import static org.lwjgl.opengl.GL11.glClearColor;
import static org.lwjgl.opengl.GL11.glViewport;
import static org.lwjgl.system.MemoryUtil.NULL;

public class Window implements AutoCloseable {

    private long windowHandle;
    private int width;
    private int height;
    private String title;
    private InputHandler inputHandler; // To delegate input events. Will be set by Main/Game.

    private Game gameApplication;      // Reference to the Game application

    // Default dimensions if not specified
    private static final String DEFAULT_TITLE = "Majn Game";
    private static final int DEFAULT_WIDTH = 854;
    private static final int DEFAULT_HEIGHT = 480;


    // Constructor now takes the Game application instance
    public Window(String title, int width, int height, Game gameApp) {
        this.title = title;
        this.width = width;
        this.height = height;
        this.gameApplication = gameApp; // Store reference to Game
        // inputHandler is set via setInputHandler after construction by Game
    }

    public Window(Game gameApp) { // Constructor for default size
        this(DEFAULT_TITLE, DEFAULT_WIDTH, DEFAULT_HEIGHT, gameApp);
    }

    public void setInputHandler(InputHandler inputHandler) {
        this.inputHandler = inputHandler;

        // Set callbacks now that inputHandler is available
        // Setup key callbacks - delegate to InputHandler
        glfwSetKeyCallback(windowHandle, (win, key, scancode, action, mods) -> {
            if (this.inputHandler != null) {
                this.inputHandler.handleKeyInput(win, key, scancode, action, mods);
            }
        });

        // Setup mouse position callbacks - delegate to InputHandler
        glfwSetCursorPosCallback(windowHandle, (win, xpos, ypos) -> {
            if (this.inputHandler != null) {
                this.inputHandler.handleMouseMovement(win, xpos, ypos);
            }
        });

        // Initialize mouse position in InputHandler
        if (this.inputHandler != null) {
            this.inputHandler.setInitialMousePosition(this.width / 2.0, this.height / 2.0);
        }
    }


    public void init() {
        GLFWErrorCallback.createPrint(System.err).set();

        if (!glfwInit()) {
            throw new IllegalStateException("Unable to initialize GLFW");
        }

        glfwDefaultWindowHints();
        glfwWindowHint(GLFW_VISIBLE, GLFW_FALSE); // Window initially invisible
        glfwWindowHint(GLFW_RESIZABLE, GLFW_TRUE); // Window resizable

        windowHandle = glfwCreateWindow(this.width, this.height, this.title, NULL, NULL);
        if (windowHandle == NULL) {
            glfwTerminate(); // Terminate GLFW if window creation fails
            throw new RuntimeException("Failed to create the GLFW window");
        }

        // Framebuffer size callback for window resizing
        // This callback should be set before the window is shown or context is made current ideally
        // It's fine here as well.
        glfwSetFramebufferSizeCallback(windowHandle, (win, newWidth, newHeight) -> {
            this.width = newWidth;
            this.height = newHeight;
            glViewport(0, 0, newWidth, newHeight); // Update OpenGL viewport
            if (this.gameApplication != null) {
                // Notify the main application, which will then notify the renderer
                this.gameApplication.onWindowResize(newWidth, newHeight);
            }
        });


        // Center the window
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer pWidth = stack.mallocInt(1);
            IntBuffer pHeight = stack.mallocInt(1);
            glfwGetWindowSize(windowHandle, pWidth, pHeight);
            GLFWVidMode vidMode = glfwGetVideoMode(glfwGetPrimaryMonitor());
            if (vidMode != null) {
                glfwSetWindowPos(
                        windowHandle,
                        (vidMode.width() - pWidth.get(0)) / 2,
                        (vidMode.height() - pHeight.get(0)) / 2);
            }
        }

        glfwMakeContextCurrent(windowHandle);
        glfwSwapInterval(1); // Enable V-Sync
        glfwShowWindow(windowHandle);

        // This line is critical for LWJGL's interoperation with GLFW's
        // OpenGL context, or any context that is managed externally.
        // GL.createCapabilities() detects the context and makes the OpenGL
        // bindings available for use.
        GL.createCapabilities();

        // Set the clear color (moved from Main.init) - or this could be in Renderer.init
        glClearColor(0.0f, 0.0f, 0.2f, 0.0f);
    }

    public boolean shouldClose() {
        return glfwWindowShouldClose(windowHandle);
    }

    public void swapBuffers() {
        glfwSwapBuffers(windowHandle);
    }

    public void pollEvents() {
        glfwPollEvents();
    }

    @Override
    public void close() {
        glfwFreeCallbacks(windowHandle);
        glfwDestroyWindow(windowHandle);
        glfwTerminate();
        GLFWErrorCallback errorCallback = glfwSetErrorCallback(null);
        if (errorCallback != null) {
            errorCallback.free();
        }
    }

    public long getWindowHandle() {
        return windowHandle;
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    public String getTitle() {
        return title;
    }

    public void setCursorDisabled(boolean disabled) {
        glfwSetInputMode(windowHandle, GLFW_CURSOR, disabled ? GLFW_CURSOR_DISABLED : GLFW_CURSOR_NORMAL);
    }
}
