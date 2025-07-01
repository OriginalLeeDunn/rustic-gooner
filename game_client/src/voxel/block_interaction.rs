use bevy::prelude::*;
use bevy::input::mouse::MouseButton;
use bevy::input::keyboard::KeyCode;
use bevy::math::primitives::Cuboid;
use super::*;
use crate::{ChunkManager, Player, CursorState, TargetedBlock, ChunkComponent};

/// System to detect which block the player is looking at
pub fn block_targeting_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut targeted_block: ResMut<TargetedBlock>,
    chunk_manager: Res<ChunkManager>,
    cursor_state: Res<CursorState>,
    player_query: Query<&Transform, With<Player>>,
    _chunks_query: Query<(&ChunkComponent, &Transform)>,
) {
    // Don't do targeting when cursor is unlocked
    if !cursor_state.locked {
        // Remove highlight if it exists
        if let Some(entity) = targeted_block.highlight_entity {
            commands.entity(entity).despawn();
            targeted_block.highlight_entity = None;
        }
        targeted_block.hit = None;
        return;
    }

    // Get player transform
    // Get player position and forward direction
    let (player_pos, forward) = if let Some(transform) = player_query.iter().next() {
        (transform.translation, transform.forward())
    } else {
        // No camera found, can't do targeting
        targeted_block.hit = None;
        return;
    };

    // Get all chunks and their data for raycasting
    let mut chunks_data = Vec::new();
    for (chunk_component, _) in _chunks_query.iter() {
        // Add chunk data to the list for raycasting
        // We need to get the chunk from the chunk manager since ChunkComponent only stores position
        if let Some(chunk) = chunk_manager.chunk_data.get(&chunk_component.pos) {
            chunks_data.push((chunk_component.pos, chunk));
        }
    }

    // Cast a ray to find the block the player is looking at
    let hit = raycast_block(player_pos, forward.into(), &chunks_data);
    
    // Remove old highlight if it exists
    if let Some(highlight_entity) = targeted_block.highlight_entity {
        commands.entity(highlight_entity).despawn();
        targeted_block.highlight_entity = None;
    }

    // If we hit a block, highlight it
    if let Some(hit_info) = hit {
        // Create a wireframe cube to highlight the targeted block
        let highlight_mesh = Mesh::from(Cuboid::new(1.05, 1.05, 1.05));
        let highlight_material = StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.3),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        };

        // Calculate world position of the block
        let block_world_pos = Vec3::new(
            (hit_info.chunk_pos.x * CHUNK_SIZE as i32 + hit_info.block_pos.x as i32) as f32 + 0.5,
            (hit_info.chunk_pos.y * CHUNK_SIZE as i32 + hit_info.block_pos.y as i32) as f32 + 0.5,
            (hit_info.chunk_pos.z * CHUNK_SIZE as i32 + hit_info.block_pos.z as i32) as f32 + 0.5,
        );

        // Spawn highlight entity
        let highlight_entity = commands.spawn((
            Mesh3d(meshes.add(highlight_mesh)),
            MeshMaterial3d(materials.add(highlight_material)),
            Transform::from_translation(block_world_pos),
        )).id();

        // Update targeted block
        targeted_block.highlight_entity = Some(highlight_entity);
        targeted_block.hit = Some(hit_info);
    } else {
        targeted_block.hit = None;
    }
}

/// System to handle block interaction (breaking and placing blocks)
pub fn block_interaction_system(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    targeted_block: Res<TargetedBlock>,
    cursor_state: Res<CursorState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Don't do interaction when cursor is unlocked
    if !cursor_state.locked {
        return;
    }

    // Get the block the player is looking at
    let hit_info = match &targeted_block.hit {
        Some(hit) => hit,
        None => return,
    };

    // Handle block breaking (left click)
    if mouse_input.just_pressed(MouseButton::Left) {
        // Get the chunk
        // First get and copy the chunk entity to avoid borrowing issues
        let chunk_entity = if let Some(entity) = chunk_manager.chunks.get(&hit_info.chunk_pos) {
            *entity
        } else {
            return;
        };
        
        // Now we can borrow chunk_manager as mutable
        if let Some(chunk) = chunk_manager.chunk_data.get_mut(&hit_info.chunk_pos) {
            // Break the block (set to air)
            chunk.set_block(hit_info.block_pos, BlockType::Air);
            
            // Rebuild the chunk mesh
            let mesh = chunk.build_mesh();
            let material = chunk.build_material();
            
            // Update the chunk entity
            if let Ok(mut entity_commands) = commands.get_entity(chunk_entity) {
                entity_commands.insert((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(material)),
                ));
            }
        }
    }

    // Handle block placing (right click)
    if mouse_input.just_pressed(MouseButton::Right) {
        // Calculate the position of the new block based on the hit normal
        let mut new_block_pos = BlockPos::new(
            (hit_info.block_pos.x as i32 + hit_info.normal.x as i32) as usize,
            (hit_info.block_pos.y as i32 + hit_info.normal.y as i32) as usize,
            (hit_info.block_pos.z as i32 + hit_info.normal.z as i32) as usize,
        );
        
        // Calculate the chunk position for the new block
        let mut new_chunk_pos = hit_info.chunk_pos;
        
        // Handle chunk boundaries
        if new_block_pos.x >= CHUNK_SIZE {
            new_chunk_pos.x += 1;
            new_block_pos.x = 0;
        } else if new_block_pos.x == 0 {
            new_chunk_pos.x -= 1;
            new_block_pos.x = CHUNK_SIZE - 1;
        }
        
        if new_block_pos.y >= CHUNK_SIZE {
            new_chunk_pos.y += 1;
            new_block_pos.y = 0;
        }
        else if new_block_pos.y == 0 && hit_info.normal.y < 0.0 {
            new_chunk_pos.y -= 1;
            new_block_pos.y = CHUNK_SIZE - 1;
        }
        
        if new_block_pos.z >= CHUNK_SIZE {
            new_chunk_pos.z += 1;
            new_block_pos.z = 0;
        }
        else if new_block_pos.z == 0 && hit_info.normal.z < 0.0 {
            new_chunk_pos.z -= 1;
            new_block_pos.z = CHUNK_SIZE - 1;
        }
        
        // Determine block type to place based on keyboard input
        let block_type = if keyboard_input.pressed(KeyCode::Digit1) {
            BlockType::Dirt
        } else if keyboard_input.pressed(KeyCode::Digit2) {
            BlockType::Stone
        } else if keyboard_input.pressed(KeyCode::Digit3) {
            BlockType::Grass
        } else {
            BlockType::Dirt // Default
        };
        
        // Get or create the chunk
        let chunk_entity = if let Some(entity) = chunk_manager.chunks.get(&new_chunk_pos) {
            *entity
        } else {
            // Create a new chunk if it doesn't exist
            let mut chunk = Chunk::new(new_chunk_pos);
            chunk.generate_terrain(&chunk_manager.terrain_generator);
            
            // Build mesh and material
            let mesh = chunk.build_mesh();
            let material = chunk.build_material();
            
            // Spawn chunk entity
            let entity = commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(material)),
                Transform::from_translation(new_chunk_pos.to_world_pos()),
                ChunkComponent { pos: new_chunk_pos },
            )).id();
            
            // Add chunk to manager
            chunk_manager.chunks.insert(new_chunk_pos, entity);
            chunk_manager.chunk_data.insert(new_chunk_pos, chunk);
            
            entity
        };
        
        // Get the chunk data
        if let Some(chunk) = chunk_manager.chunk_data.get_mut(&new_chunk_pos) {
            // Place the block
            chunk.set_block(new_block_pos, block_type);
            
            // Rebuild the chunk mesh
            let mesh = chunk.build_mesh();
            let material = chunk.build_material();
            
            // Update the chunk entity
            if let Ok(mut entity_commands) = commands.get_entity(chunk_entity) {
                entity_commands.insert((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(material)),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Import only what we need for tests
    use bevy::prelude::{App, ButtonInput, Camera3d, GlobalTransform, KeyCode, MouseButton, Transform, Vec3};
    use crate::voxel::chunk::Chunk;
    use crate::voxel::block::BlockType;
    // No need for HashMap import since we're using init_resource
    
    // Helper function to create a test app with necessary systems and resources
    fn setup_test_app() -> App {
        let mut app = App::new();
        
        // Add required resources
        app.init_resource::<ChunkManager>();
        app.insert_resource(CursorState { locked: true });
        app.insert_resource(TargetedBlock { hit: None, highlight_entity: None });
        
        // Add mesh and material assets required by block_interaction_system
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        
        // Add input resources required by block_interaction_system
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        
        // Add required systems
        app.add_systems(Update, (block_targeting_system, block_interaction_system));
        
        app
    }
    
    // Helper function to create a simple test chunk
    fn create_test_chunk(pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new(pos);
        
        // Add a floor of blocks at y=0
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set_block(BlockPos::new(x, 0, z), BlockType::Stone);
            }
        }
        
        chunk
    }
    
    #[test]
    fn test_block_breaking() {
        let mut app = setup_test_app();
        
        // Create a chunk at origin
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let mut chunk = create_test_chunk(chunk_pos);
        
        // Add a specific block to break
        let block_pos = BlockPos::new(5, 0, 5);
        chunk.set_block(block_pos, BlockType::Dirt);
        
        // Add chunk to the chunk manager
        let mut chunk_manager = app.world_mut().resource_mut::<ChunkManager>();
        chunk_manager.chunk_data.insert(chunk_pos, chunk);
        chunk_manager.chunks.insert(chunk_pos, Entity::from_raw(1));
        
        // Create a camera entity with Player component
        app.world_mut().spawn((
            Camera3d::default(),
            Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::new(5.0, 0.0, 5.0), Vec3::Y),
            GlobalTransform::default(),
            Player {
                speed: 5.0,
                mouse_sensitivity: 0.1,
                acceleration: 0.0,
                velocity: Vec3::ZERO,
                max_velocity: 10.0,
                friction: 0.9,
                jump_force: 10.0,
                is_grounded: false,
                gravity: 20.0,
            },
        ));
        
        // Add targeted block resource
        app.world_mut().insert_resource(TargetedBlock {
            hit: Some(RaycastHit {
                chunk_pos,
                block_pos,
                normal: Vec3::Y,
                distance: 5.0,
            }),
            highlight_entity: None,
        });
        
        // Simulate left mouse button press
        app.world_mut().insert_resource(ButtonInput::<MouseButton>::default());
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        
        // Run the interaction system
        app.update();
        
        // Check if the block was removed
        let chunk_manager = app.world().resource::<ChunkManager>();
        let chunk = chunk_manager.chunk_data.get(&chunk_pos).unwrap();
        assert_eq!(chunk.get_block(block_pos), BlockType::Air, "Block should be broken");
    }
    
    #[test]
    fn test_block_placing() {
        let mut app = setup_test_app();
        
        // Create a chunk at origin
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let chunk = create_test_chunk(chunk_pos);
        
        // Add chunk to the chunk manager
        let mut chunk_manager = app.world_mut().resource_mut::<ChunkManager>();
        chunk_manager.chunk_data.insert(chunk_pos, chunk);
        chunk_manager.chunks.insert(chunk_pos, Entity::from_raw(1));
        
        // Create a camera entity with Player component
        app.world_mut().spawn((
            Camera3d::default(),
            Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::new(5.0, 0.0, 5.0), Vec3::Y),
            GlobalTransform::default(),
            Player {
                speed: 5.0,
                mouse_sensitivity: 0.1,
                acceleration: 0.0,
                velocity: Vec3::ZERO,
                max_velocity: 10.0,
                friction: 0.9,
                jump_force: 10.0,
                is_grounded: false,
                gravity: 20.0,
            },
        ));
        
        // Add targeted block resource - targeting the floor
        let block_pos = BlockPos::new(5, 0, 5);
        app.world_mut().insert_resource(TargetedBlock {
            hit: Some(RaycastHit {
                chunk_pos,
                block_pos,
                normal: Vec3::Y,  // Pointing up, so new block should be at y=1
                distance: 5.0,
            }),
            highlight_entity: None,
        });
        
        // Simulate right mouse button press and key 2 for stone
        app.world_mut().insert_resource(ButtonInput::<MouseButton>::default());
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Right);
        
        app.world_mut().insert_resource(ButtonInput::<KeyCode>::default());
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Digit2);
        
        // Run the interaction system
        app.update();
        
        // Check if the block was placed
        let chunk_manager = app.world().resource::<ChunkManager>();
        let chunk = chunk_manager.chunk_data.get(&chunk_pos).unwrap();
        assert_eq!(chunk.get_block(BlockPos::new(5, 1, 5)), BlockType::Stone, "Stone block should be placed above the floor");
    }
    
    #[test]
    fn test_block_type_selection() {
        let mut app = setup_test_app();
        
        // Create a chunk at origin
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let chunk = create_test_chunk(chunk_pos);
        
        // Add chunk to the chunk manager
        let mut chunk_manager = app.world_mut().resource_mut::<ChunkManager>();
        chunk_manager.chunk_data.insert(chunk_pos, chunk);
        chunk_manager.chunks.insert(chunk_pos, Entity::from_raw(1));
        
        // Create a camera entity with Player component
        app.world_mut().spawn((
            Camera3d::default(),
            Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::new(5.0, 0.0, 5.0), Vec3::Y),
            GlobalTransform::default(),
            Player {
                speed: 5.0,
                mouse_sensitivity: 0.1,
                acceleration: 0.0,
                velocity: Vec3::ZERO,
                max_velocity: 10.0,
                friction: 0.9,
                jump_force: 10.0,
                is_grounded: false,
                gravity: 20.0,
            },
        ));
        
        // Test different block types
        let test_cases = [
            (KeyCode::Digit1, BlockType::Dirt),
            (KeyCode::Digit2, BlockType::Stone),
            (KeyCode::Digit3, BlockType::Grass),
        ];
        
        for (key, expected_type) in test_cases {
            // Add targeted block resource
            let block_pos = BlockPos::new(5, 0, 5);
            app.world_mut().insert_resource(TargetedBlock {
                hit: Some(RaycastHit {
                    chunk_pos,
                    block_pos,
                    normal: Vec3::Y,
                    distance: 5.0,
                }),
                highlight_entity: None,
            });
            
            // Reset inputs
            app.world_mut().insert_resource(ButtonInput::<MouseButton>::default());
            app.world_mut().insert_resource(ButtonInput::<KeyCode>::default());
            
            // Press right mouse button and the specific key
            app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Right);
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(key);
            
            // Run the interaction system
            app.update();
            
            // Check if the block was placed with correct type
            let chunk_manager = app.world().resource::<ChunkManager>();
            let chunk = chunk_manager.chunk_data.get(&chunk_pos).unwrap();
            assert_eq!(chunk.get_block(BlockPos::new(5, 1, 5)), expected_type, 
                       "Block should be of type {:?}", expected_type);
        }
    }
}
