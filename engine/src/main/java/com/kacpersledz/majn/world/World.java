package com.kacpersledz.majn.world;

import java.util.ArrayList;
import java.util.Collection;
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
        Chunk chunk = new Chunk(chunkX, chunkY, chunkZ);
        int grassLevel = Chunk.CHUNK_HEIGHT / 2 - 1; // e.g., 16 / 2 - 1 = 7

        // For now, generate a flat world with a layer of grass on top of dirt
        for (int x = 0; x < Chunk.CHUNK_WIDTH; x++) {
            for (int y = 0; y < Chunk.CHUNK_HEIGHT; y++) {
                for (int z = 0; z < Chunk.CHUNK_DEPTH; z++) {
                    if (y < grassLevel) {
                        chunk.setBlock(x, y, z, Block.BlockType.DIRT);
                    } else if (y == grassLevel) {
                        chunk.setBlock(x, y, z, Block.BlockType.GRASS);
                    } else {
                        // This is important: ensure blocks above grass are AIR
                        // The Chunk constructor already initializes to AIR,
                        // so explicitly setting to AIR here is for clarity
                        // or if the default Chunk initialization changes.
                        chunk.setBlock(x, y, z, Block.BlockType.AIR);
                    }
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

    public Collection<Chunk> getChunksAroundPlayer(int playerX, int playerY, int playerZ) {
        int playerChunkX = Math.floorDiv(playerX, Chunk.CHUNK_WIDTH);
        int playerChunkY = Math.floorDiv(playerY, Chunk.CHUNK_HEIGHT);
        int playerChunkZ = Math.floorDiv(playerZ, Chunk.CHUNK_DEPTH);

        Collection<Chunk> nearbyChunks = new ArrayList<>();

        // Player's current chunk
        nearbyChunks.add(getChunk(playerChunkX, playerChunkY, playerChunkZ));

        // Immediate N, S, E, W neighbors
        nearbyChunks.add(getChunk(playerChunkX + 1, playerChunkY, playerChunkZ)); // East
        nearbyChunks.add(getChunk(playerChunkX - 1, playerChunkY, playerChunkZ)); // West
        nearbyChunks.add(getChunk(playerChunkX, playerChunkY, playerChunkZ + 1)); // South
        nearbyChunks.add(getChunk(playerChunkX, playerChunkY, playerChunkZ - 1)); // North

        return nearbyChunks;
    }
}
