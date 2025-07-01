use bevy::prelude::*;
use super::{BlockPos, ChunkPos, Chunk, BlockType, CHUNK_SIZE};

/// Maximum distance for block interaction
pub const MAX_INTERACTION_DISTANCE: f32 = 5.0;

/// Result of a raycast hit
#[derive(Debug, Clone)]
pub struct RaycastHit {
    /// Position of the chunk containing the block
    pub chunk_pos: ChunkPos,
    /// Position of the block that was hit
    pub block_pos: BlockPos,
    /// Normal direction of the hit (points outward from the block)
    pub normal: Vec3,
    /// Distance from the ray origin to the hit point
    /// This field is currently unused but kept for future features like distance-based interaction
    #[allow(dead_code)]
    pub distance: f32,
}

/// Cast a ray from the camera and find the first block hit
pub fn raycast_block(
    origin: Vec3,
    direction: Vec3,
    chunks: &[(ChunkPos, &Chunk)],
) -> Option<RaycastHit> {
    // Normalize the direction
    let direction = direction.normalize();
    
    // Use DDA (Digital Differential Analysis) algorithm for ray casting
    // This is an efficient algorithm for traversing a voxel grid
    
    // Current position in the grid
    let mut current_pos = origin;
    
    // Step size along each axis (how far we need to go to cross a grid cell)
    let step_size = Vec3::new(
        (1.0 / direction.x).abs(),
        (1.0 / direction.y).abs(),
        (1.0 / direction.z).abs(),
    );
    
    // Direction to step in grid coordinates
    let step_dir = Vec3::new(
        direction.x.signum(),
        direction.y.signum(),
        direction.z.signum(),
    );
    
    // Distance to next grid boundary along each axis
    let next_boundary = Vec3::new(
        if step_dir.x > 0.0 { current_pos.x.ceil() - current_pos.x } else { current_pos.x - current_pos.x.floor() },
        if step_dir.y > 0.0 { current_pos.y.ceil() - current_pos.y } else { current_pos.y - current_pos.y.floor() },
        if step_dir.z > 0.0 { current_pos.z.ceil() - current_pos.z } else { current_pos.z - current_pos.z.floor() },
    );
    
    // Distance to next boundary in terms of ray length
    let mut t_max = Vec3::new(
        if direction.x != 0.0 { next_boundary.x * step_size.x } else { f32::INFINITY },
        if direction.y != 0.0 { next_boundary.y * step_size.y } else { f32::INFINITY },
        if direction.z != 0.0 { next_boundary.z * step_size.z } else { f32::INFINITY },
    );
    
    // Track the total distance traveled
    let mut distance = 0.0;
    
    // Normal of the hit face
    let mut normal = Vec3::ZERO;
    
    // Limit the number of iterations to avoid infinite loops
    for _ in 0..100 {
        // Check if we've gone too far
        if distance > MAX_INTERACTION_DISTANCE {
            return None;
        }
        
        // Get the current block position
        let block_x = current_pos.x.floor() as i32;
        let block_y = current_pos.y.floor() as i32;
        let block_z = current_pos.z.floor() as i32;
        
        // Calculate chunk position
        let chunk_x = (block_x as f32 / CHUNK_SIZE as f32).floor() as i32;
        let chunk_y = (block_y as f32 / CHUNK_SIZE as f32).floor() as i32;
        let chunk_z = (block_z as f32 / CHUNK_SIZE as f32).floor() as i32;
        let chunk_pos = ChunkPos::new(chunk_x, chunk_y, chunk_z);
        
        // Calculate local block position within chunk
        let local_x = ((block_x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32;
        let local_y = ((block_y % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32;
        let local_z = ((block_z % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32;
        let block_pos = BlockPos::new(local_x as usize, local_y as usize, local_z as usize);
        
        // Find the chunk
        if let Some((_, chunk)) = chunks.iter().find(|(pos, _)| *pos == chunk_pos) {
            // Check if this block is solid
            if chunk.get_block(block_pos) != BlockType::Air {
                return Some(RaycastHit {
                    block_pos,
                    chunk_pos,
                    normal,
                    distance,
                });
            }
        }
        
        // Determine which axis to step along next (the one with smallest t_max)
        if t_max.x < t_max.y && t_max.x < t_max.z {
            // Step along X axis
            current_pos.x += step_dir.x;
            distance = t_max.x;
            t_max.x += step_size.x;
            normal = Vec3::new(-step_dir.x, 0.0, 0.0);
        } else if t_max.y < t_max.z {
            // Step along Y axis
            current_pos.y += step_dir.y;
            distance = t_max.y;
            t_max.y += step_size.y;
            normal = Vec3::new(0.0, -step_dir.y, 0.0);
        } else {
            // Step along Z axis
            current_pos.z += step_dir.z;
            distance = t_max.z;
            t_max.z += step_size.z;
            normal = Vec3::new(0.0, 0.0, -step_dir.z);
        }
    }
    
    // If we get here, we didn't find a hit within our iteration limit
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChunkManager, TerrainGenerator, ChunkPos, BlockPos};
    use crate::voxel::chunk::Chunk;
    use crate::voxel::block::BlockType;
    use std::collections::HashMap;
    
    // Helper function to create a test chunk manager with a simple chunk
    fn create_test_chunk_manager() -> ChunkManager {
        let mut chunk_manager = ChunkManager {
            chunks: HashMap::new(),
            chunk_data: HashMap::new(),
            terrain_generator: TerrainGenerator::new(0),
            render_distance: 1,
        };
        
        // Create a single chunk at origin
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let mut chunk = Chunk::new(chunk_pos);
        
        // Create a floor at y=0
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set_block(BlockPos::new(x, 0, z), BlockType::Stone);
            }
        }
        
        // Add the chunk to the manager
        chunk_manager.chunk_data.insert(chunk_pos, chunk);
        
        chunk_manager
    }
    
    #[test]
    fn test_raycast_hit_floor() {
        let chunk_manager = create_test_chunk_manager();
        
        // Cast ray from above looking down
        let origin = Vec3::new(5.0, 5.0, 5.0);
        let direction = Vec3::new(0.0, -1.0, 0.0).normalize();
        
        // Prepare chunks data in the format expected by raycast_block
        let chunks_data: Vec<(ChunkPos, &Chunk)> = chunk_manager.chunk_data.iter()
            .map(|(pos, chunk)| (*pos, chunk))
            .collect();
        
        let hit = raycast_block(origin, direction, &chunks_data);
        
        assert!(hit.is_some(), "Ray should hit the floor");
        
        if let Some(hit_info) = hit {
            assert_eq!(hit_info.chunk_pos, ChunkPos::new(0, 0, 0));
            assert_eq!(hit_info.block_pos.y, 0);
            assert_eq!(hit_info.normal, Vec3::new(0.0, 1.0, 0.0));
        }
    }
    
    #[test]
    fn test_raycast_miss() {
        let chunk_manager = create_test_chunk_manager();
        
        // Cast ray upward (should miss all blocks)
        let origin = Vec3::new(5.0, 5.0, 5.0);
        let direction = Vec3::new(0.0, 1.0, 0.0).normalize();
        
        // Prepare chunks data in the format expected by raycast_block
        let chunks_data: Vec<(ChunkPos, &Chunk)> = chunk_manager.chunk_data.iter()
            .map(|(pos, chunk)| (*pos, chunk))
            .collect();
        
        let hit = raycast_block(origin, direction, &chunks_data);
        
        assert!(hit.is_none(), "Ray should not hit anything");
    }
}
