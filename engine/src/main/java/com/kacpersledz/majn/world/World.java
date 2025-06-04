package com.kacpersledz.majn.world;

import java.util.HashMap;
import java.util.Map;

public class World {

    private Map<String, Chunk> chunks;

    public World() {
        this.chunks = new HashMap<>();
    }

    // Simple key generation for chunk coordinates
    private String getChunkKey(int chunkX, int chunkY, int chunkZ) {
        return chunkX + "," + chunkY + "," + chunkZ;
    }

    public Chunk getChunk(int chunkX, int chunkY, int chunkZ) {
        String key = getChunkKey(chunkX, chunkY, chunkZ);
        if (!chunks.containsKey(key)) {
            generateChunk(chunkX, chunkY, chunkZ);
        }
        return chunks.get(key);
    }

    private void generateChunk(int chunkX, int chunkY, int chunkZ) {
        String key = getChunkKey(chunkX, chunkY, chunkZ);
        Chunk chunk = new Chunk();
        // For now, fill the entire chunk with DIRT
        for (int x = 0; x < Chunk.CHUNK_WIDTH; x++) {
            for (int y = 0; y < Chunk.CHUNK_HEIGHT; y++) {
                for (int z = 0; z < Chunk.CHUNK_DEPTH; z++) {
                    chunk.setBlock(x, y, z, Block.BlockType.DIRT);
                }
            }
        }
        chunks.put(key, chunk);
        System.out.println("Generated chunk at: " + chunkX + ", " + chunkY + ", " + chunkZ); // For debugging
    }

    public Block getBlock(int worldX, int worldY, int worldZ) {
        int chunkX = Math.floorDiv(worldX, Chunk.CHUNK_WIDTH);
        int chunkY = Math.floorDiv(worldY, Chunk.CHUNK_HEIGHT);
        int chunkZ = Math.floorDiv(worldZ, Chunk.CHUNK_DEPTH);

        Chunk chunk = getChunk(chunkX, chunkY, chunkZ);

        int localX = Math.floorMod(worldX, Chunk.CHUNK_WIDTH);
        int localY = Math.floorMod(worldY, Chunk.CHUNK_HEIGHT);
        int localZ = Math.floorMod(worldZ, Chunk.CHUNK_DEPTH);

        return chunk.getBlock(localX, localY, localZ);
    }
}
