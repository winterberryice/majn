// engine/src/ui/item.rs

use crate::block::BlockType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Block(BlockType),
}

#[derive(Debug, Clone, Copy)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub count: u8,
}

impl ItemStack {
    pub fn new(item_type: ItemType, count: u8) -> Self {
        Self { item_type, count }
    }
}
