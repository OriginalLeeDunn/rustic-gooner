//! Noise generation for procedural terrain

use noise::{NoiseFn, Perlin};

/// Terrain generator using noise functions
pub struct TerrainGenerator {
    /// Perlin noise generator for height map
    height_noise: Perlin,
    /// Perlin noise generator for biome variation
    biome_noise: Perlin,
}

impl TerrainGenerator {
    /// Create a new terrain generator with the given seed
    pub fn new(seed: u32) -> Self {
        Self {
            height_noise: Perlin::new(seed),
            biome_noise: Perlin::new(seed.wrapping_add(1)),
        }
    }
    
    /// Get the height at a given x,z coordinate
    pub fn get_height(&self, x: f64, z: f64) -> f64 {
        // Scale coordinates to make the terrain more varied
        let scaled_x = x * 0.02;
        let scaled_z = z * 0.02;
        
        // Base height using perlin noise (range -1 to 1)
        let base_height = self.height_noise.get([scaled_x, scaled_z]);
        
        // Add some variation with a different scale
        let detail = self.height_noise.get([scaled_x * 4.0, scaled_z * 4.0]) * 0.25;
        let mountains = self.height_noise.get([scaled_x * 0.5, scaled_z * 0.5]) * 0.5;
        
        // Add more dramatic terrain features
        let combined = base_height + detail + mountains * mountains * 2.0;
        
        // Convert to a height value (0 to ~30)
        combined * 12.0 + 15.0
    }
    
    /// Get the biome value at a given x,z coordinate (0.0 to 1.0)
    pub fn get_biome_value(&self, x: f64, z: f64) -> f64 {
        // Scale coordinates for biome variation
        let scaled_x = x * 0.01;
        let scaled_z = z * 0.01;
        
        // Get biome noise (range -1 to 1) and convert to 0 to 1
        (self.biome_noise.get([scaled_x, scaled_z]) + 1.0) * 0.5
    }
}
