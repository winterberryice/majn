package com.kacpersledz.majn.world;

public class Chunk {

    public static final int CHUNK_WIDTH = 16;
    public static final int CHUNK_HEIGHT = 16;
    public static final int CHUNK_DEPTH = 16;

    private Block[][][] blocks;

    public Chunk() {
        blocks = new Block[CHUNK_WIDTH][CHUNK_HEIGHT][CHUNK_DEPTH];
        // Initialize all blocks to AIR by default
        for (int x = 0; x < CHUNK_WIDTH; x++) {
            for (int y = 0; y < CHUNK_HEIGHT; y++) {
                for (int z = 0; z < CHUNK_DEPTH; z++) {
                    blocks[x][y][z] = new Block(Block.BlockType.AIR);
                }
            }
        }
    }

    public Block getBlock(int x, int y, int z) {
        if (x < 0 || x >= CHUNK_WIDTH || y < 0 || y >= CHUNK_HEIGHT || z < 0 || z >= CHUNK_DEPTH) {
            // Or throw an exception, or return a special 'out of bounds' block
            return null;
        }
        return blocks[x][y][z];
    }

    public void setBlock(int x, int y, int z, Block.BlockType type) {
        if (x < 0 || x >= CHUNK_WIDTH || y < 0 || y >= CHUNK_HEIGHT || z < 0 || z >= CHUNK_DEPTH) {
            // Or throw an exception
            return;
        }
        blocks[x][y][z] = new Block(type);
    }
}
