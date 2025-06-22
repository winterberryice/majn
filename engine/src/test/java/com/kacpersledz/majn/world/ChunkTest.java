package com.kacpersledz.majn.world;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class ChunkTest {

    @Test
    void testChunkInitialization() {
        Chunk chunk = new Chunk(0, 0, 0);
        for (int x = 0; x < Chunk.CHUNK_WIDTH; x++) {
            for (int y = 0; y < Chunk.CHUNK_HEIGHT; y++) {
                for (int z = 0; z < Chunk.CHUNK_DEPTH; z++) {
                    assertNotNull(chunk.getBlock(x, y, z), "Block at " + x + "," + y + "," + z + " should not be null");
                    assertEquals(Block.BlockType.AIR, chunk.getBlock(x, y, z).getType(), "Default block type should be AIR");
                }
            }
        }
    }

    @Test
    void testSetAndGetBlock() {
        Chunk chunk = new Chunk(0, 0, 0);
        chunk.setBlock(0, 0, 0, Block.BlockType.DIRT);
        assertEquals(Block.BlockType.DIRT, chunk.getBlock(0, 0, 0).getType(), "Block type should be DIRT after setBlock");

        chunk.setBlock(5, 5, 5, Block.BlockType.DIRT);
        assertEquals(Block.BlockType.DIRT, chunk.getBlock(5, 5, 5).getType());

        // Test overriding a block
        chunk.setBlock(0, 0, 0, Block.BlockType.AIR);
        assertEquals(Block.BlockType.AIR, chunk.getBlock(0, 0, 0).getType(), "Block type should be AIR after override");
    }

    @Test
    void testGetBlockOutOfBounds() {
        Chunk chunk = new Chunk(0, 0, 0);
        assertNull(chunk.getBlock(-1, 0, 0), "Getting block out of bounds (x-negative) should return null");
        assertNull(chunk.getBlock(Chunk.CHUNK_WIDTH, 0, 0), "Getting block out of bounds (x-positive) should return null");
        assertNull(chunk.getBlock(0, -1, 0), "Getting block out of bounds (y-negative) should return null");
        assertNull(chunk.getBlock(0, Chunk.CHUNK_HEIGHT, 0), "Getting block out of bounds (y-positive) should return null");
        assertNull(chunk.getBlock(0, 0, -1), "Getting block out of bounds (z-negative) should return null");
        assertNull(chunk.getBlock(0, 0, Chunk.CHUNK_DEPTH), "Getting block out of bounds (z-positive) should return null");
    }

    @Test
    void testSetBlockOutOfBounds() {
        Chunk chunk = new Chunk(0, 0, 0);
        // These calls should not throw exceptions and the chunk should remain intact
        chunk.setBlock(-1, 0, 0, Block.BlockType.DIRT);
        chunk.setBlock(Chunk.CHUNK_WIDTH, 0, 0, Block.BlockType.DIRT);
        // Verify a known block is unchanged
        assertEquals(Block.BlockType.AIR, chunk.getBlock(0,0,0).getType());
    }
}
