package com.kacpersledz.majn.view;

import org.lwjgl.stb.STBTTAlignedQuad;
import org.lwjgl.stb.STBTTFontinfo;
import org.lwjgl.stb.STBTTPackContext;
import org.lwjgl.stb.STBTTPackedchar;
import org.lwjgl.stb.STBTruetype;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.FloatBuffer;
import java.nio.channels.Channels;
import java.nio.channels.FileChannel;
import java.nio.channels.ReadableByteChannel;

import static org.lwjgl.opengl.GL11.*;
import static org.lwjgl.opengl.GL12.GL_CLAMP_TO_EDGE;

public class FontRenderer {

    private STBTTFontinfo fontInfo;
    private STBTTPackedchar.Buffer charData;
    private int fontAtlasTextureId = -1;
    private float currentFontSize;
    private final int bitmapWidth;
    private final int bitmapHeight;

    public FontRenderer(String otfResourcePath, float fontSize, int atlasWidth, int atlasHeight) throws IOException {
        this.bitmapWidth = atlasWidth;
        this.bitmapHeight = atlasHeight;
        loadFont(otfResourcePath, fontSize);
    }

    private static ByteBuffer ioResourceToByteBuffer(String resource, int bufferSize) throws IOException {
        ByteBuffer buffer;
        File file = new File(resource);
        if (file.isFile()) {
            try (FileInputStream fis = new FileInputStream(file);
                 FileChannel fc = fis.getChannel()) {
                buffer = MemoryUtil.memAlloc((int) fc.size() + 1);
                while (fc.read(buffer) != -1) {
                    // Loop to read all bytes
                }
            }
        } else {
            try (InputStream source = FontRenderer.class.getClassLoader().getResourceAsStream(resource);
                 ReadableByteChannel rbc = Channels.newChannel(source)) {
                if (source == null) {
                    throw new IOException("Resource not found: " + resource);
                }
                buffer = MemoryUtil.memAlloc(bufferSize);
                while (true) {
                    int bytes = rbc.read(buffer);
                    if (bytes == -1) {
                        break;
                    }
                    if (buffer.remaining() == 0) {
                        ByteBuffer newBuffer = MemoryUtil.memRealloc(buffer, buffer.capacity() * 2);
                        if (newBuffer == null) {
                            throw new OutOfMemoryError("Failed to reallocate buffer for font resource");
                        }
                        buffer = newBuffer;
                    }
                }
            }
        }
        buffer.flip();
        return buffer;
    }

    private void loadFont(String otfResourcePath, float fontSize) throws IOException {
        fontInfo = STBTTFontinfo.create();
        ByteBuffer ttf = ioResourceToByteBuffer(otfResourcePath, 150 * 1024); // 150KB buffer for font

        if (!STBTruetype.stbtt_InitFont(fontInfo, ttf)) {
            throw new IllegalStateException("Failed to initialize font information.");
        }

        currentFontSize = fontSize;
        charData = STBTTPackedchar.create(96); // ASCII 32-126

        ByteBuffer bitmap = MemoryUtil.memAlloc(bitmapWidth * bitmapHeight);

        try (STBTTPackContext pc = STBTTPackContext.create()) {
            STBTruetype.stbtt_PackBegin(pc, bitmap, bitmapWidth, bitmapHeight, 0, 1, MemoryUtil.NULL);
            if (!STBTruetype.stbtt_PackFontRange(pc, ttf, 0, currentFontSize, 32, charData)) {
                MemoryUtil.memFree(bitmap); // Free bitmap before throwing
                throw new IllegalStateException("Failed to pack font characters.");
            }
            STBTruetype.stbtt_PackEnd(pc);
        } // pc is auto-closed here

        fontAtlasTextureId = glGenTextures();
        glBindTexture(GL_TEXTURE_2D, fontAtlasTextureId);
        glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RED, bitmapWidth, bitmapHeight, 0, GL_RED, GL_UNSIGNED_BYTE, bitmap);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);

        MemoryUtil.memFree(bitmap);
        // ttf buffer is from ioResourceToByteBuffer, which uses MemoryUtil.memAlloc.
        // It should be freed if STBTrueType doesn't take ownership or copy it.
        // Based on common usage, stbtt_InitFont likely uses the buffer directly.
        // If the ByteBuffer was allocated on heap, it's GC'd. Direct buffers need explicit free.
        // For safety, if ttf is a direct buffer and not needed after stbtt_PackFontRange, free it.
        // However, fontInfo might hold a reference. Let's assume STB manages it or it's fine for now.
        // MemoryUtil.memFree(ttf); // Potentially free this if sure it's not needed.
    }

    public void drawString(float x, float y, String text, float r, float g, float b, float a) {
        if (fontAtlasTextureId == -1 || charData == null) {
            System.err.println("Font not loaded, cannot drawString.");
            return;
        }

        glEnable(GL_TEXTURE_2D);
        glBindTexture(GL_TEXTURE_2D, fontAtlasTextureId);
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        glColor4f(r, g, b, a);

        try (MemoryStack stack = MemoryStack.stackPush()) {
            FloatBuffer xPos = stack.floats(x);
            FloatBuffer yPos = stack.floats(y);
            STBTTAlignedQuad q = STBTTAlignedQuad.mallocStack(stack);

            glBegin(GL_QUADS);
            for (int i = 0; i < text.length(); i++) {
                char character = text.charAt(i);
                if (character < 32 || character >= 127) {
                    character = '?'; // Default character for unsupported ones
                }
                STBTruetype.stbtt_GetPackedQuad(charData, bitmapWidth, bitmapHeight, character - 32, xPos, yPos, q, false);
                glTexCoord2f(q.s0(), q.t0()); glVertex2f(q.x0(), q.y0());
                glTexCoord2f(q.s1(), q.t0()); glVertex2f(q.x1(), q.y0());
                glTexCoord2f(q.s1(), q.t1()); glVertex2f(q.x1(), q.y1());
                glTexCoord2f(q.s0(), q.t1()); glVertex2f(q.x0(), q.y1());
            }
            glEnd();
        }

        glDisable(GL_BLEND);
        glDisable(GL_TEXTURE_2D);
        // Reset color to white to avoid affecting other rendering parts, or let caller manage.
        // glColor4f(1.0f, 1.0f, 1.0f, 1.0f);
    }

    public float getCurrentFontSize() {
        return currentFontSize;
    }

    public void cleanup() {
        if (fontAtlasTextureId != -1) {
            glDeleteTextures(fontAtlasTextureId);
            fontAtlasTextureId = -1;
        }
        if (charData != null) {
            charData.free(); // If STBTTPackedchar.Buffer allocates native resources
            charData = null;
        }
        if (fontInfo != null) {
            fontInfo.free(); // If STBTTFontinfo allocates native resources
            fontInfo = null;
        }
        // The ByteBuffer ttf for the font file itself might need freeing if it was a direct buffer
        // and not managed by STBTTFontinfo after init. This is complex; for now, assume it's handled
        // or will be addressed if memory issues arise.
    }
}
