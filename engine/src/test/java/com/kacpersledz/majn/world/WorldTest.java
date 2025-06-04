package com.kacpersledz.majn.world;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class WorldTest {

    @Test
    void testGetChunkGeneratesNewChunk() {
        World world = new World();
        Chunk chunk = world.getChunk(0, 0, 0);
        assertNotNull(chunk, "Generated chunk should not be null");
        // Check if a block in the chunk is DIRT as per current generateChunk logic
        assertEquals(Block.BlockType.DIRT, chunk.getBlock(0,0,0).getType(), "Block in new chunk should be DIRT");
    }

    @Test
    void testGetChunkReturnsExistingChunk() {
        World world = new World();
        Chunk firstChunk = world.getChunk(1, 1, 1);
        Chunk secondChunk = world.getChunk(1, 1, 1);
        assertSame(firstChunk, secondChunk, "Requesting the same chunk coordinates should return the same chunk instance");
    }

    @Test
    void testGetBlock() {
        World world = new World();

        // Test block within the first chunk
        Block block1 = world.getBlock(5, 5, 5);
        assertNotNull(block1);
        assertEquals(Block.BlockType.DIRT, block1.getType(), "Block at 5,5,5 should be DIRT");

        // Test block at the edge of the first chunk
        Block block2 = world.getBlock(Chunk.CHUNK_WIDTH - 1, Chunk.CHUNK_HEIGHT - 1, Chunk.CHUNK_DEPTH - 1);
        assertNotNull(block2);
        assertEquals(Block.BlockType.DIRT, block2.getType());

        // Test block that should be in a different, newly generated chunk
        Block block3 = world.getBlock(Chunk.CHUNK_WIDTH, 0, 0); // This is (0,0,0) in chunk (1,0,0)
        assertNotNull(block3);
        assertEquals(Block.BlockType.DIRT, block3.getType());
        assertEquals(Block.BlockType.DIRT, world.getChunk(1,0,0).getBlock(0,0,0).getType());


        Block block4 = world.getBlock(0, Chunk.CHUNK_HEIGHT, 0); // This is (0,0,0) in chunk (0,1,0)
        assertNotNull(block4);
        assertEquals(Block.BlockType.DIRT, block4.getType());
        assertEquals(Block.BlockType.DIRT, world.getChunk(0,1,0).getBlock(0,0,0).getType());


        Block block5 = world.getBlock(-1, -1, -1); // This is (15,15,15) in chunk (-1,-1,-1)
        assertNotNull(block5);
        assertEquals(Block.BlockType.DIRT, block5.getType());
        assertEquals(Block.BlockType.DIRT, world.getChunk(-1,-1,-1).getBlock(Chunk.CHUNK_WIDTH-1,Chunk.CHUNK_HEIGHT-1,Chunk.CHUNK_DEPTH-1).getType());
    }

    @Test
    void testGetBlockCorrectChunkAndLocalCoordinates() {
        World world = new World();

        // Coordinates for a block
        int worldX = 17;
        int worldY = 33;
        int worldZ = -5;

        // Expected chunk coordinates
        int expectedChunkX = Math.floorDiv(worldX, Chunk.CHUNK_WIDTH); // 1
        int expectedChunkY = Math.floorDiv(worldY, Chunk.CHUNK_HEIGHT); // 2
        int expectedChunkZ = Math.floorDiv(worldZ, Chunk.CHUNK_DEPTH); // -1

        // Expected local coordinates within the chunk
        int expectedLocalX = Math.floorMod(worldX, Chunk.CHUNK_WIDTH); // 1
        int expectedLocalY = Math.floorMod(worldY, Chunk.CHUNK_HEIGHT); // 1
        int expectedLocalZ = Math.floorMod(worldZ, Chunk.CHUNK_DEPTH); // 11 (since -5 mod 16 is 11)


        Block block = world.getBlock(worldX, worldY, worldZ);
        assertNotNull(block, "Block should not be null");
        assertEquals(Block.BlockType.DIRT, block.getType(), "Block should be DIRT");

        // Verify it's coming from the correct chunk
        Chunk targetChunk = world.getChunk(expectedChunkX, expectedChunkY, expectedChunkZ);
        assertNotNull(targetChunk, "Target chunk should exist");
        assertSame(block, targetChunk.getBlock(expectedLocalX, expectedLocalY, expectedLocalZ),
                   "The block from world.getBlock should be the same instance as block from targetChunk.getBlock with local coords");
    }
}
