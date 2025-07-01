//! Block types and properties for the voxel world

use bevy::prelude::*;

/// Represents different types of blocks in the voxel world
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Water,
}

impl BlockType {
    /// Returns whether the block is solid (blocks movement and vision)
    pub fn is_solid(&self) -> bool {
        match self {
            BlockType::Air | BlockType::Water => false,
            _ => true,
        }
    }
    
    /// Returns whether the block is transparent (allows light to pass through)
    pub fn is_transparent(&self) -> bool {
        match self {
            BlockType::Air | BlockType::Water => true,
            _ => false,
        }
    }
    
    /// Returns the color for this block type
    pub fn get_color(&self) -> Color {
        match self {
            BlockType::Air => Color::srgba(0.0, 0.0, 0.0, 0.0),
            BlockType::Grass => Color::srgb(0.3, 0.7, 0.3),
            BlockType::Dirt => Color::srgb(0.6, 0.4, 0.2),
            BlockType::Stone => Color::srgb(0.5, 0.5, 0.5),
            BlockType::Sand => Color::srgb(0.9, 0.8, 0.6),
            BlockType::Water => Color::srgba(0.2, 0.4, 0.8, 0.7),
        }
    }
}
