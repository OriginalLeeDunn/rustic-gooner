use bevy::prelude::*;
use bevy::pbr::{StandardMaterial, FogVolume};
use bevy::input::mouse::MouseMotion;
use bevy::window::CursorGrabMode;
use std::collections::HashMap;

mod voxel;

use voxel::*;
use voxel::block_interaction::{block_targeting_system, block_interaction_system};

/// Resource for managing chunks in the world
#[derive(Resource)]
pub struct ChunkManager {
    /// Map of chunk positions to entity IDs
    pub chunks: HashMap<ChunkPos, Entity>,
    /// Map of chunk positions to chunk data
    pub chunk_data: HashMap<ChunkPos, Chunk>,
    /// Terrain generator for procedural generation
    pub terrain_generator: TerrainGenerator,
    /// Render distance in chunks
    pub render_distance: i32,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_data: HashMap::new(),
            terrain_generator: TerrainGenerator::new(42),  // Fixed seed for now
            render_distance: 3,  // Default render distance
        }
    }
}

// Player component
#[derive(Component)]
struct Player {
    /// Movement speed in units per second
    speed: f32,
    /// Mouse sensitivity for camera rotation
    mouse_sensitivity: f32,
    /// Acceleration for smoother movement
    acceleration: f32,
    /// Current velocity vector
    velocity: Vec3,
    /// Maximum velocity magnitude
    max_velocity: f32,
    /// Friction coefficient to slow down when not moving
    friction: f32,
    /// Jump force
    jump_force: f32,
    /// Whether the player is on the ground
    is_grounded: bool,
    /// Gravity force
    gravity: f32,
}

// Resource to store cursor state
#[derive(Resource)]
struct CursorState {
    locked: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self { locked: false }
    }
}

// Resource to track the currently targeted block
#[derive(Resource, Default)]
struct TargetedBlock {
    hit: Option<RaycastHit>,
    highlight_entity: Option<Entity>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<ChunkManager>()
        .init_resource::<CursorState>()
        .init_resource::<TargetedBlock>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            camera_movement_system,
            update_chunk_loading,
            player_movement,
            mouse_look,
            toggle_cursor_lock,
            block_targeting_system,
            block_interaction_system,
        ))
        .run();
}

/// Spawn a chunk at the given position
fn spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_manager: &mut ResMut<ChunkManager>,
    chunk_pos: ChunkPos,
) {
    // Check if the chunk already exists
    if chunk_manager.chunks.contains_key(&chunk_pos) {
        return;
    }
    
    // Create a new chunk
    let mut chunk = Chunk::new(chunk_pos);
    
    // Generate the terrain
    chunk.generate_terrain(&chunk_manager.terrain_generator);
    
    // Build the mesh and material
    let mesh = chunk.build_mesh();
    let material = chunk.build_material();
    
    // Spawn the chunk entity
    let chunk_entity = commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_translation(chunk_pos.to_world_pos()),
        ChunkComponent { pos: chunk_pos },
    )).id();
    
    // Add the chunk to the manager
    chunk_manager.chunks.insert(chunk_pos, chunk_entity);
}

/// Component to mark an entity as a chunk
#[derive(Component)]
struct ChunkComponent {
    pos: ChunkPos,
}

/// System to update chunk loading based on camera position
fn update_chunk_loading(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    chunk_query: Query<(Entity, &ChunkComponent)>,
) {
    // Get the camera position
    let camera_transform = if let Ok(transform) = camera_query.single() {
        transform
    } else {
        return; // No camera found
    };
    
    // Convert camera position to chunk coordinates
    let camera_chunk_pos = ChunkPos::from_world_pos(camera_transform.translation);
    
    // Collect chunks that are too far away and should be unloaded
    let mut chunks_to_unload = Vec::new();
    for (entity, chunk_comp) in chunk_query.iter() {
        let dx = (chunk_comp.pos.x - camera_chunk_pos.x).abs();
        let dz = (chunk_comp.pos.z - camera_chunk_pos.z).abs();
        
        // If the chunk is outside render distance, mark it for unloading
        if dx > chunk_manager.render_distance || dz > chunk_manager.render_distance {
            chunks_to_unload.push((entity, chunk_comp.pos));
        }
    }
    
    // Unload chunks that are too far away
    for (entity, pos) in chunks_to_unload {
        commands.entity(entity).despawn();
        chunk_manager.chunks.remove(&pos);
    }
    
    // Load new chunks that are within render distance
    let render_dist = chunk_manager.render_distance;
    for x in (camera_chunk_pos.x - render_dist)..=(camera_chunk_pos.x + render_dist) {
        for z in (camera_chunk_pos.z - render_dist)..=(camera_chunk_pos.z + render_dist) {
            // Only consider the ground level chunks for now (y=0)
            let pos = ChunkPos::new(x, 0, z);
            
            // If this chunk doesn't exist yet, spawn it
            if !chunk_manager.chunks.contains_key(&pos) {
                spawn_chunk(&mut commands, &mut meshes, &mut materials, &mut chunk_manager, pos);
            }
        }
    }
}

/// Set up a simple 3D scene.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
    ));

    // Calculate a good starting position above the terrain
    let start_x = 0.0;
    let start_z = 0.0;
    let terrain_height = chunk_manager.terrain_generator.get_height(start_x, start_z) as f32 + 2.0; // +2 to be safely above
    
    // Camera with player component
    commands.spawn((
        Camera3d::default(),
        // Position the camera at eye level above the terrain and looking forward
        Transform::from_xyz(start_x as f32, terrain_height + 1.8, start_z as f32)
            .looking_at(Vec3::new(start_x as f32, terrain_height + 1.8, (start_z + 5.0) as f32), Vec3::Y),
        Player {
            speed: 10.0,
            mouse_sensitivity: 0.1,
            acceleration: 50.0,
            velocity: Vec3::ZERO,
            max_velocity: 20.0,
            friction: 5.0,
            jump_force: 8.0,
            is_grounded: false,
            gravity: 20.0,
        },
    ));
    
    // Add global fog for atmospheric effect
    commands.spawn((
        FogVolume {
            fog_color: Color::srgba(0.5, 0.5, 1.0, 1.0),
            density_factor: 0.02,
            scattering: 0.3,
            absorption: 0.2,
            ..default()
        },
        // Make the fog volume cover the entire scene
        GlobalTransform::default(),
        Transform::default(),
    ));
    
    // Generate initial chunks around origin
    for x in -2..=2 {
        for z in -2..=2 {
            let chunk_pos = ChunkPos::new(x, 0, z);
            spawn_chunk(&mut commands, &mut meshes, &mut materials, &mut chunk_manager, chunk_pos);
        }
    }
}

/// Basic camera movement controls.
fn camera_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += *transform.forward();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= *transform.forward();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= *transform.right();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += *transform.right();
        }
        if keyboard_input.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            direction -= Vec3::Y;
        }

        if direction.length() > 0.0 {
            transform.translation += direction.normalize() * time.delta_secs() * 5.0;
        }
    }
}

/// Player movement system with physics and controls
fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor_state: Res<CursorState>,
    mut query: Query<(&mut Player, &mut Transform), With<Camera3d>>,
) {
    if !cursor_state.locked {
        return; // Don't move if cursor is not locked
    }
    
    let dt = time.delta_secs();
    
    // Handle potential errors with query.single_mut()
    let Ok((mut player, mut transform)) = query.single_mut() else {
        return;
    };
    
    // Calculate movement direction based on keyboard input
    let mut direction = Vec3::ZERO;
    
    // Forward/backward
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += *transform.forward();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction += *transform.back();
    }
    
    // Left/right
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction += *transform.left();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += *transform.right();
    }
    
    // Make sure direction is only on the XZ plane for normal movement
    direction.y = 0.0;
    
    // Normalize direction vector to prevent faster diagonal movement
    if direction != Vec3::ZERO {
        direction = direction.normalize();
    }
    
    // Apply acceleration based on input direction
    let acceleration = player.acceleration;
    if direction != Vec3::ZERO {
        player.velocity += direction * acceleration * dt;
    }
    
    // Apply jumping
    let jump_force = player.jump_force;
    let is_grounded = player.is_grounded;
    if keyboard_input.just_pressed(KeyCode::Space) && is_grounded {
        player.velocity.y = jump_force;
        player.is_grounded = false;
    }
    
    // Apply gravity
    let gravity = player.gravity;
    if !player.is_grounded {
        player.velocity.y -= gravity * dt;
    }
    
    // Apply sprint modifier when shift is pressed
    let mut current_speed = player.speed;
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        current_speed *= 1.5; // Sprint multiplier
    }
    
    // Apply movement based on direction and speed
    if direction != Vec3::ZERO {
        // Apply speed to horizontal movement only
        let horizontal_movement = direction * current_speed;
        player.velocity.x = horizontal_movement.x;
        player.velocity.z = horizontal_movement.z;
    } else {
        // Stop horizontal movement when no keys are pressed
        player.velocity.x = 0.0;
        player.velocity.z = 0.0;
    }
    
    // Apply friction to slow down when not moving
    let xz_velocity = Vec3::new(player.velocity.x, 0.0, player.velocity.z);
    let friction = player.friction;
    if direction == Vec3::ZERO && xz_velocity.length_squared() > 0.0 {
        let friction_force = xz_velocity.normalize() * friction * dt;
        let friction_magnitude = friction_force.length();
        let velocity_magnitude = xz_velocity.length();
        
        if friction_magnitude > velocity_magnitude {
            // Prevent overshooting (coming to a complete stop)
            player.velocity.x = 0.0;
            player.velocity.z = 0.0;
        } else {
            // Apply friction
            player.velocity -= friction_force;
        }
    }
    
    // Clamp velocity to maximum speed
    let max_velocity = player.max_velocity;
    if player.velocity.length() > max_velocity {
        player.velocity = player.velocity.normalize() * max_velocity;
    }
    
    // Apply velocity to position
    transform.translation += player.velocity * dt;
    
    // Ground collision detection with chunk data
    let player_pos = transform.translation;
    let feet_y = player_pos.y - 1.8; // Player height/2
    
    // Check if player is on ground
    if feet_y <= 0.0 {
        // Simple ground collision with world floor
        transform.translation.y = 1.8; // Keep player at correct height
        player.velocity.y = 0.0;
        player.is_grounded = true;
    } else {
        // We're above the ground floor, check for blocks below
        player.is_grounded = false;
        
        // This would be better with proper collision detection against blocks
        // For now we just use a simple ground plane at y=0
    }
}

/// Mouse look system for camera rotation
fn mouse_look(
    mut mouse_motion: EventReader<MouseMotion>,
    cursor_state: Res<CursorState>,
    mut query: Query<(&Player, &mut Transform), With<Camera3d>>,
) {
    if !cursor_state.locked {
        return; // Don't rotate if cursor is not locked
    }
    
    let Ok((player, mut transform)) = query.single_mut() else {
        return;
    };
    
    for ev in mouse_motion.read() {
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        
        // Adjust yaw and pitch based on mouse movement
        yaw -= ev.delta.x * player.mouse_sensitivity * 0.01;
        pitch -= ev.delta.y * player.mouse_sensitivity * 0.01;
        
        // Clamp pitch to avoid camera flipping
        pitch = pitch.clamp(-1.5, 1.5);
        
        // Apply rotation
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }
}

/// Toggle cursor lock when pressing Tab
fn toggle_cursor_lock(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    mut cursor_state: ResMut<CursorState>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        cursor_state.locked = !cursor_state.locked;
        
        let Ok(mut window) = windows.single_mut() else {
            return;
        };
        
        if cursor_state.locked {
            // Hide and lock cursor
            window.cursor_options.visible = false;
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
        } else {
            // Show and release cursor
            window.cursor_options.visible = true;
            window.cursor_options.grab_mode = CursorGrabMode::None;
        }
    }
}
