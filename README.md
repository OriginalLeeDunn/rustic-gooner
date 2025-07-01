# Rustic Gooner

A Rust-based game client and server implementation using Bevy and Rocket.

## 🛠️ Technical Highlights

This project aims to implement the following features, inspired by classic voxel games.

-   **GLFW window & input setup**: Initializes a window, sets up mouse/keyboard controls.
-   **OpenGL rendering pipeline**: Implements vertex buffers, shaders, and a preliminary textured cube.
-   **Camera control**: Uses a linear algebra library for view/projection matrices to navigate in 3D.
-   **Texture atlas**: Creates a single texture sheet for block textures, mapping UVs accordingly.
-   **Chunk system**: Renders blocks in 16x16x256 chunks, culling internal faces for efficiency.
-   **Infinite world streaming**: Dynamically loads and unloads chunks near the camera.
-   **Block placement & destruction**: Implements ray-casting from the camera to interact with blocks.
-   **Procedural terrain**: Uses noise functions for terrain generation.
-   **Transparency sorting**: Splits and depth-sorts opaque/transparent voxel meshes before rendering.
-   **Liquids and ambient occlusion**: Adds water/lava and vertex AO for visual polish.

## 🎮 Planning a Bevy 0.16 Learning Project

Here's a roadmap to recreate this pipeline in Rust with Bevy 0.16:

| Phase | Task                               | Bevy 0.16 Focus                                                 |
| :---: | :--------------------------------- | :-------------------------------------------------------------- |
|  1️⃣   | Setup window and camera movement   | `WindowPlugin`, `Transform`, `bevy_input`, 3D camera entity     |
|  2️⃣   | Mesh a single cube with texture    | `Mesh`, `StandardMaterial`, load textures via `AssetServer`     |
|  3️⃣   | Texture atlas mapping              | UV coordinates in mesh generation, custom material              |
|  4️⃣   | Chunk management                   | ECS chunk component, generate meshes per-chunk                  |
|  5️⃣   | Face culling optimization          | Only include visible face mesh quads                            |
|  6️⃣   | Procedural world gen               | Noise libs (e.g. `noise` crate), populate chunks                |
|  7️⃣   | Chunk streaming                    | On camera movement, spawn/despawn nearby chunks                 |
|  8️⃣   | Ray casting                        | Use Bevy's raycast or simple ray-voxel math for block interaction |
|  9️⃣   | Transparency & liquids             | Separate mesh sets and render transparent with `AlphaMode::Blend` |
|  🔟   | Ambient occlusion & shader effects | Extend PBR via shader customization                             |

## ✅ Starter Tasks in Bevy

### Project setup

Add dependencies to `Cargo.toml`:
```toml
bevy = "0.16"
noise = "0.8"
```
Then, initialize the Bevy app, window, and camera controls.

### Cube entity
Spawn a cube mesh with a standard material and make it respond to WASD/mouse input.

### Procedural chunk
Define a chunk structure, generate flat ground using a noise function, and add cube instances.

### Face culling
In the chunk mesh builder, skip faces that have neighboring blocks.

### Dynamic chunk loading
Query the camera position each frame to spawn/destroy chunk entities within a certain radius.

### Block interaction
Cast a ray from the camera to intersect voxels, allowing for adding/removing blocks on click and rebuilding the affected chunk mesh.

### Advanced features (optional)
-   **Liquids**: Generate a heightmap-based water mesh.
-   **AO/Shaders**: Implement a custom material for ambient occlusion.
-   **Textures**: Create a texture atlas with multiple block types.

## 🧠 Tips & Resources
-   Check out Bevy's examples repository on GitHub for voxel engine demos.
-   Study the `noise` crate for terrain generation techniques.
-   Use mesh builders like `bevy_mesh_builder` or create your own to handle UVs and AO.
-   The chunking logic is the trickiest part. Refactor mesh creation for chunks to support streaming and dynamic updates.
