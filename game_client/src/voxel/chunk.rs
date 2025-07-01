//! Chunk system for the voxel world

use bevy::prelude::*;
// No imports needed here

use super::{BlockType, TerrainGenerator};

/// Size of a chunk in blocks (x, y, z)
pub const CHUNK_SIZE: usize = 16;

/// Represents a 3D position in chunk coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    
    /// Convert world position to chunk position
    pub fn from_world_pos(pos: Vec3) -> Self {
        Self {
            x: (pos.x / CHUNK_SIZE as f32).floor() as i32,
            y: (pos.y / CHUNK_SIZE as f32).floor() as i32,
            z: (pos.z / CHUNK_SIZE as f32).floor() as i32,
        }
    }
    
    /// Get the world position of the chunk's origin (minimum corner)
    pub fn to_world_pos(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CHUNK_SIZE as f32,
            self.y as f32 * CHUNK_SIZE as f32,
            self.z as f32 * CHUNK_SIZE as f32,
        )
    }
}

/// Represents a position within a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl BlockPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }
    
    /// Convert local block position to a flat array index
    pub fn to_index(&self) -> usize {
        self.y * CHUNK_SIZE * CHUNK_SIZE + self.z * CHUNK_SIZE + self.x
    }
    
    /// Check if the position is within chunk bounds
    pub fn is_valid(&self) -> bool {
        self.x < CHUNK_SIZE && self.y < CHUNK_SIZE && self.z < CHUNK_SIZE
    }
}

/// A chunk of blocks in the voxel world
pub struct Chunk {
    /// Position of this chunk in chunk coordinates
    pub position: ChunkPos,
    /// 3D array of blocks stored as a flat array
    blocks: [BlockType; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
    /// Whether the chunk mesh needs to be rebuilt
    pub dirty: bool,
}

impl Chunk {
    /// Static method to get the index in the blocks array for the given coordinates
    pub fn get_index_static(x: usize, y: usize, z: usize) -> usize {
        y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x
    }
    
    /// Create a new empty chunk at the given position
    pub fn new(position: ChunkPos) -> Self {
        Self {
            position,
            blocks: [BlockType::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
            dirty: true,
        }
    }
    
    /// Generate terrain for this chunk using the provided generator
    pub fn generate_terrain(&mut self, generator: &TerrainGenerator) {
        let world_pos = self.position.to_world_pos();
        
        // For each column in the chunk
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Calculate world position for this column
                let world_x = world_pos.x as f64 + x as f64;
                let world_z = world_pos.z as f64 + z as f64;
                
                // Get the terrain height at this position
                let height = generator.get_height(world_x, world_z) as usize;
                let biome_value = generator.get_biome_value(world_x, world_z);
                
                // Fill the column with appropriate blocks
                for y in 0..CHUNK_SIZE {
                    let world_y = world_pos.y as usize + y;
                    
                    let block_type = if world_y > height {
                        // Above ground is air
                        BlockType::Air
                    } else if world_y == height {
                        // Surface layer depends on biome
                        if biome_value > 0.6 {
                            BlockType::Sand
                        } else {
                            BlockType::Grass
                        }
                    } else if height > 3 && world_y > height - 3 {
                        // A few layers of dirt below surface
                        BlockType::Dirt
                    } else {
                        // Deep underground is stone
                        BlockType::Stone
                    };
                    
                    self.set_block(BlockPos::new(x, y, z), block_type);
                }
            }
        }
        
        self.dirty = true;
    }
    
    /// Get a block at the specified position
    pub fn get_block(&self, pos: BlockPos) -> BlockType {
        if pos.is_valid() {
            self.blocks[pos.to_index()]
        } else {
            BlockType::Air
        }
    }
    
    /// Set a block at the specified position
    pub fn set_block(&mut self, pos: BlockPos, block_type: BlockType) {
        if pos.is_valid() {
            self.blocks[pos.to_index()] = block_type;
            self.dirty = true;
        }
    }
    
    /// Check if a face should be rendered based on neighboring blocks
    pub fn should_render_face(&self, pos: BlockPos, direction: Direction) -> bool {
        let block = self.get_block(pos);
        
        // Air blocks don't have faces
        if block == BlockType::Air {
            return false;
        }
        
        // Get the position of the neighboring block
        let neighbor_pos = match direction {
            Direction::PosY => {
                if pos.y >= CHUNK_SIZE - 1 { return true; }
                BlockPos::new(pos.x, pos.y + 1, pos.z)
            },
            Direction::NegY => {
                if pos.y == 0 { return true; }
                BlockPos::new(pos.x, pos.y - 1, pos.z)
            },
            Direction::PosZ => {
                if pos.z >= CHUNK_SIZE - 1 { return true; }
                BlockPos::new(pos.x, pos.y, pos.z + 1)
            },
            Direction::NegZ => {
                if pos.z == 0 { return true; }
                BlockPos::new(pos.x, pos.y, pos.z - 1)
            },
            Direction::PosX => {
                if pos.x >= CHUNK_SIZE - 1 { return true; }
                BlockPos::new(pos.x + 1, pos.y, pos.z)
            },
            Direction::NegX => {
                if pos.x == 0 { return true; }
                BlockPos::new(pos.x - 1, pos.y, pos.z)
            },
        };
        
        // Check if the neighbor is outside the chunk bounds
        if neighbor_pos.x >= CHUNK_SIZE || neighbor_pos.y >= CHUNK_SIZE || neighbor_pos.z >= CHUNK_SIZE {
            return true; // Render faces at chunk boundaries
        }
        
        // Get the neighboring block
        let neighbor = self.get_block(neighbor_pos);
        
        // Render the face if the neighbor is air or transparent
        neighbor == BlockType::Air || neighbor.is_transparent()
    }
    
    /// Builds a mesh for the chunk
    pub fn build_mesh(&self) -> Mesh {
        // We'll store all the mesh data here
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        let mut colors = Vec::new();
        
        // For each block in the chunk
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block_pos = BlockPos::new(x, y, z);
                    let block_type = self.blocks[Chunk::get_index_static(x, y, z)];
                    
                    // Skip air blocks
                    if block_type == BlockType::Air {
                        continue;
                    }
                    
                    // Check each face of the block
                    for &direction in &[Direction::PosY, Direction::NegY, Direction::PosZ, Direction::NegZ, Direction::PosX, Direction::NegX] {
                        if self.should_render_face(block_pos, direction) {
                            // Add the face to the mesh
                            self.add_face_to_mesh(
                                &mut positions,
                                &mut normals,
                                &mut uvs,
                                &mut indices,
                                &mut colors,
                                block_pos,
                                direction,
                                block_type,
                            );
                        }
                    }
                }
            }
        }
        
        // If there are no blocks to render, return an empty mesh
        if positions.is_empty() {
            return Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD);
        }
        
        // Create the mesh
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.clone());
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices.clone()));
        
        mesh
    }
    
    /// Calculate ambient occlusion value for a corner
    fn calculate_ao(&self, x: i32, y: i32, z: i32, side1: (i32, i32, i32), side2: (i32, i32, i32)) -> f32 {
        let side1_solid = self.is_position_solid(x + side1.0, y + side1.1, z + side1.2);
        let side2_solid = self.is_position_solid(x + side2.0, y + side2.1, z + side2.2);
        let corner_solid = self.is_position_solid(x + side1.0 + side2.0, y + side1.1 + side2.1, z + side1.2 + side2.2);
        
        if side1_solid && side2_solid {
            return 0.5; // Darkest
        }
        
        if corner_solid && (!side1_solid && !side2_solid) {
            return 0.7; // Medium dark
        }
        
        if side1_solid || side2_solid || corner_solid {
            return 0.8; // Slightly dark
        }
        
        1.0 // No occlusion
    }
    
    /// Check if a position is solid (for AO calculation)
    fn is_position_solid(&self, x: i32, y: i32, z: i32) -> bool {
        if x < 0 || y < 0 || z < 0 || x >= CHUNK_SIZE as i32 || y >= CHUNK_SIZE as i32 || z >= CHUNK_SIZE as i32 {
            // For blocks outside this chunk, we'd need to check neighboring chunks
            // For simplicity, assume outside blocks are solid
            return true;
        }
        
        let block = self.blocks[Chunk::get_index_static(x as usize, y as usize, z as usize)];
        block != BlockType::Air
    }
    
    /// Add a face to the mesh for a specific block and direction
    fn add_face_to_mesh(
        &self,
        positions: &mut Vec<[f32; 3]>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
        indices: &mut Vec<u32>,
        colors: &mut Vec<[f32; 4]>,
        block_pos: BlockPos,
        direction: Direction,
        block_type: BlockType,
    ) {
        // Get the world position of the block
        let x = block_pos.x as f32;
        let y = block_pos.y as f32;
        let z = block_pos.z as f32;
        
        // Get the color of the block
        let color = block_type.get_color();
        let color_array = match color {
            Color::Srgba(c) => [c.red, c.green, c.blue, c.alpha],
            _ => [0.5, 0.5, 0.5, 1.0], // Default gray for other color types
        };
        
        // Add the face with ambient occlusion
        self.add_face(positions, normals, colors, indices, direction, x, y, z, color_array);
        
        // Add UVs (simple 0-1 mapping for all vertices)
        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([1.0, 1.0]);
        uvs.push([0.0, 1.0]);
    }
    
    /// Add a face to the mesh data with ambient occlusion
    fn add_face(&self, 
                positions: &mut Vec<[f32; 3]>, 
                normals: &mut Vec<[f32; 3]>, 
                colors: &mut Vec<[f32; 4]>,
                indices: &mut Vec<u32>,
                direction: Direction, 
                x: f32, y: f32, z: f32, 
                color: [f32; 4]) {
        let index_offset = positions.len() as u32;
        let ix = x as i32;
        let iy = y as i32;
        let iz = z as i32;
        
        // Calculate ambient occlusion values for each corner of the face
        let (ao_values, vertex_positions) = match direction {
            Direction::PosY => {
                // Top face (y+)
                let ao0 = self.calculate_ao(ix, iy+1, iz, (0, 0, -1), (-1, 0, 0));
                let ao1 = self.calculate_ao(ix+1, iy+1, iz, (0, 0, -1), (1, 0, 0));
                let ao2 = self.calculate_ao(ix+1, iy+1, iz+1, (0, 0, 1), (1, 0, 0));
                let ao3 = self.calculate_ao(ix, iy+1, iz+1, (0, 0, 1), (-1, 0, 0));
                
                let positions = [
                    [x, y + 1.0, z],
                    [x + 1.0, y + 1.0, z],
                    [x + 1.0, y + 1.0, z + 1.0],
                    [x, y + 1.0, z + 1.0],
                ];
                
                ([ao0, ao1, ao2, ao3], positions)
            }
            Direction::NegY => {
                // Bottom face (y-)
                let ao0 = self.calculate_ao(ix, iy-1, iz, (0, 0, -1), (-1, 0, 0));
                let ao1 = self.calculate_ao(ix, iy-1, iz+1, (0, 0, 1), (-1, 0, 0));
                let ao2 = self.calculate_ao(ix+1, iy-1, iz+1, (0, 0, 1), (1, 0, 0));
                let ao3 = self.calculate_ao(ix+1, iy-1, iz, (0, 0, -1), (1, 0, 0));
                
                let positions = [
                    [x, y, z],
                    [x, y, z + 1.0],
                    [x + 1.0, y, z + 1.0],
                    [x + 1.0, y, z],
                ];
                
                ([ao0, ao1, ao2, ao3], positions)
            }
            Direction::PosZ => {
                // Front face (z+)
                let ao0 = self.calculate_ao(ix, iy, iz+1, (-1, 0, 0), (0, -1, 0));
                let ao1 = self.calculate_ao(ix, iy+1, iz+1, (-1, 0, 0), (0, 1, 0));
                let ao2 = self.calculate_ao(ix+1, iy+1, iz+1, (1, 0, 0), (0, 1, 0));
                let ao3 = self.calculate_ao(ix+1, iy, iz+1, (1, 0, 0), (0, -1, 0));
                
                let positions = [
                    [x, y, z + 1.0],
                    [x, y + 1.0, z + 1.0],
                    [x + 1.0, y + 1.0, z + 1.0],
                    [x + 1.0, y, z + 1.0],
                ];
                
                ([ao0, ao1, ao2, ao3], positions)
            }
            Direction::NegZ => {
                // Back face (z-)
                let ao0 = self.calculate_ao(ix, iy, iz-1, (-1, 0, 0), (0, -1, 0));
                let ao1 = self.calculate_ao(ix+1, iy, iz-1, (1, 0, 0), (0, -1, 0));
                let ao2 = self.calculate_ao(ix+1, iy+1, iz-1, (1, 0, 0), (0, 1, 0));
                let ao3 = self.calculate_ao(ix, iy+1, iz-1, (-1, 0, 0), (0, 1, 0));
                
                let positions = [
                    [x, y, z],
                    [x + 1.0, y, z],
                    [x + 1.0, y + 1.0, z],
                    [x, y + 1.0, z],
                ];
                
                ([ao0, ao1, ao2, ao3], positions)
            }
            Direction::PosX => {
                // Right face (x+)
                let ao0 = self.calculate_ao(ix+1, iy, iz, (0, -1, 0), (0, 0, -1));
                let ao1 = self.calculate_ao(ix+1, iy, iz+1, (0, -1, 0), (0, 0, 1));
                let ao2 = self.calculate_ao(ix+1, iy+1, iz+1, (0, 1, 0), (0, 0, 1));
                let ao3 = self.calculate_ao(ix+1, iy+1, iz, (0, 1, 0), (0, 0, -1));
                
                let positions = [
                    [x + 1.0, y, z],
                    [x + 1.0, y, z + 1.0],
                    [x + 1.0, y + 1.0, z + 1.0],
                    [x + 1.0, y + 1.0, z],
                ];
                
                ([ao0, ao1, ao2, ao3], positions)
            }
            Direction::NegX => {
                // Left face (x-)
                let ao0 = self.calculate_ao(ix-1, iy, iz, (0, -1, 0), (0, 0, -1));
                let ao1 = self.calculate_ao(ix-1, iy+1, iz, (0, 1, 0), (0, 0, -1));
                let ao2 = self.calculate_ao(ix-1, iy+1, iz+1, (0, 1, 0), (0, 0, 1));
                let ao3 = self.calculate_ao(ix-1, iy, iz+1, (0, -1, 0), (0, 0, 1));
                
                let positions = [
                    [x, y, z],
                    [x, y + 1.0, z],
                    [x, y + 1.0, z + 1.0],
                    [x, y, z + 1.0],
                ];
                
                ([ao0, ao1, ao2, ao3], positions)
            }
        };
        
        // Add vertices with positions, normals, and colors with AO
        let face_normal = match direction {
            Direction::PosY => [0.0, 1.0, 0.0],
            Direction::NegY => [0.0, -1.0, 0.0],
            Direction::PosZ => [0.0, 0.0, 1.0],
            Direction::NegZ => [0.0, 0.0, -1.0],
            Direction::PosX => [1.0, 0.0, 0.0],
            Direction::NegX => [-1.0, 0.0, 0.0],
        };
        
        // Add all vertices with their AO values
        for i in 0..4 {
            positions.push(vertex_positions[i]);
            normals.push(face_normal);
            
            // Apply ambient occlusion to vertex color
            let ao_color = [
                color[0] * ao_values[i],
                color[1] * ao_values[i],
                color[2] * ao_values[i],
                color[3],
            ];
            colors.push(ao_color);
        }
        
        // Add indices for the face (two triangles)
        // Check which diagonal produces better AO results
        let ao0 = ao_values[0];
        let ao1 = ao_values[1];
        let ao2 = ao_values[2];
        let ao3 = ao_values[3];
        
        // Choose the diagonal that maximizes AO quality
        if ao0 + ao2 > ao1 + ao3 {
            // Use diagonal 0-2
            indices.push(index_offset);
            indices.push(index_offset + 1);
            indices.push(index_offset + 2);
            indices.push(index_offset);
            indices.push(index_offset + 2);
            indices.push(index_offset + 3);
        } else {
            // Use diagonal 1-3
            indices.push(index_offset);
            indices.push(index_offset + 1);
            indices.push(index_offset + 3);
            indices.push(index_offset + 1);
            indices.push(index_offset + 2);
            indices.push(index_offset + 3);
        }
    }
    
    /// Create a material
    pub fn build_material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            ..default()
        }
    }
}

/// Represents the six possible directions for a block face
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}
