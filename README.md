# GK4 – 3D Scene Application

A real-time 3D scene renderer built in **Rust** using **glium** (OpenGL 3.3+).

## Features

### Scene Objects
- **Moving object** – a cube orbiting the scene, performing both translation and rotation
- **Static objects** – ground plane, two spheres (smooth surfaces), a torus, and two cubes
- **OBJ model loading** – `tobj`-based loader with tangent/bitangent computation for normal mapping

### Camera System (4 cameras, press 1–4)
| Key | Camera       | Description                                                |
|-----|--------------|------------------------------------------------------------|
| `1` | **Static**   | Fixed position overlooking the entire scene                |
| `2` | **Tracking** | Fixed position, continuously rotates to follow the mover   |
| `3` | **TPP**      | Third-person perspective attached behind the moving object  |
| `4` | **FPP**      | First-person perspective at the moving object's position   |

### Lighting (5 lights total)
- **Point light** – warm light above the scene centre
- **Fixed spotlight** – cool, aimed at the torus area
- **Directional "sun"** – modulated by the day/night cycle
- **2 × moving spotlights** – "headlights" attached to the moving object
- Light **attenuation** with distance (constant + linear + quadratic)
- **Arrow keys** to manually adjust the headlight direction

### Shading
- **Phong shading** (default) – per-fragment lighting with normal interpolation
- **Gouraud shading** – per-vertex lighting (toggle with `P`)
- **Normal mapping** – supported in the Phong pipeline via TBN matrix

### Environment Effects
- **Fog** – exponential-squared density, toggle with `F`, adjust density with `+`/`-`
- **Day/Night cycle** – automatic smooth transition (toggle with `N`), manual with `O`/`L`
- All lighting calculated in **view (camera) space**
- **Perspective projection**

## Controls

| Key          | Action                              |
|--------------|-------------------------------------|
| `1`–`4`      | Switch camera                       |
| `P`          | Toggle Phong / Gouraud shading      |
| `F`          | Toggle fog                          |
| `+` / `-`    | Increase / decrease fog density     |
| `N`          | Toggle auto day/night cycle         |
| `O` / `L`    | Manual day increase / decrease      |
| `↑↓←→`       | Adjust moving-object spotlight aim  |
| `Esc`        | Quit                                |

## Building & Running

```bash
cargo run --release
```

## Project Structure

```
src/
├── main.rs         – Application entry, event loop, input handling
├── camera.rs       – Static, Tracking, TPP, FPP, and Free camera implementations
├── light.rs        – Point, Spot, and Directional light types with view-space transform
├── model.rs        – OBJ loader (tobj) and procedural mesh builder
├── primitives.rs   – Sphere, torus, cube, and plane generators
├── renderer.rs     – Shader management, dynamic uniform upload, draw calls
├── scene.rs        – Scene graph, moving object logic, default scene builder
├── types.rs        – Material, Transform, matrix conversion helpers
└── vertex.rs       – Vertex struct with position, normal, UV, tangent, bitangent

assets/
├── shaders/
│   ├── phong.vert / phong.frag     – Phong per-fragment shading
│   └── gouraud.vert / gouraud.frag – Gouraud per-vertex shading
└── models/                          – Place .obj files here for loading
```

## Dependencies

| Crate    | Purpose                            |
|----------|------------------------------------|
| `glium`  | OpenGL wrapper + windowing         |
| `cgmath` | Linear algebra (vectors, matrices) |
| `tobj`   | OBJ model file loading             |
| `image`  | Texture loading (for future use)   |
