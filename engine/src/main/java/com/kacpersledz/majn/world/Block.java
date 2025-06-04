package com.kacpersledz.majn.world;

public class Block {

    public enum BlockType {
        DIRT,
        AIR
    }

    private BlockType type;

    public Block(BlockType type) {
        this.type = type;
    }

    public BlockType getType() {
        return type;
    }
}
