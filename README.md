# GK4 — 3D Scene Application

A real-time 3D scene renderer written in **Rust** using **glium** (OpenGL 3.3 core) with an **egui** control panel.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Controls](#controls)
3. [Lighting Model — View-Space Calculation](#lighting-model--view-space-calculation)
4. [Phong Shading vs Gouraud Shading](#phong-shading-vs-gouraud-shading)
5. [Normal Mapping / Bump Mapping](#normal-mapping--bump-mapping)
6. [Project Structure](#project-structure)

---

## Getting Started

### Prerequisites

| Tool | Version |
|------|---------|
| **Rust** (stable) | ≥ 1.70 |
| **Cargo** | comes with Rust |
| GPU drivers with **OpenGL 3.3** support | any modern GPU |

### Build & Run

```bash
# Clone the repository
git clone <repo-url>
cd gk4

# Build in release mode (recommended for smooth FPS)
cargo build --release

# Run
cargo run --release
```

The window opens at **1280 × 720** with the egui options panel on the left.

### Controls

| Key | Action |
|-----|--------|
| **1 – 5** | Switch camera (Static / Tracking / TPP / FPP / Free) |
| **WASD** | Move (Free camera) or Drive (manual drive mode) |
| **Q / E** | Down / Up (Free camera) |
| **RMB** | Look around (Free camera) |
| **Arrow keys** | Aim headlights (works in parallel with WASD) |
| **Space** | Toggle animation / manual drive |
| **P** | Toggle Phong ↔ Gouraud shading |
| **F** | Toggle fog |
| **+ / −** | Adjust fog density |
| **N** | Toggle auto day/night cycle |
| **O / L** | Manual day ↑ / night ↓ |
| **Tab** | Show/hide options panel |
| **Esc** | Quit |

---

## Lighting Model — View-Space Calculation

All lighting is computed in **view space** (also called *eye space* or *camera space*). This means that every position, direction, and normal is transformed by the **View matrix** before any lighting math is performed.

### Why View Space?

In view space the camera sits at the origin (0,0,0) and looks down the −Z axis. This gives us two key advantages:

1. **The view direction is trivial.** For any fragment at position **P_view**, the direction toward the camera is simply:

   **V̂ = normalize(−P_view)**

   No need to pass or compute the camera's world position in the shader.

2. **Light positions transform once per frame.** Each light's world-space position **L_w** is converted on the CPU:

   **L_view = V · L_w**

   where V is the 4×4 view matrix. Directions (like spotlight aim or sun direction) use the same matrix with w = 0:

   **d̂_view = normalize(V · (d_x, d_y, d_z, 0)ᵀ)**

### Normal Transformation

Surface normals cannot be transformed by the regular Model-View matrix when non-uniform scaling is present. Instead we use the **normal matrix**:

**N = ((V · M)⁻¹)ᵀ**

which is the transpose of the inverse of the upper-left 3×3 of the model-view matrix. This ensures normals remain perpendicular to the surface after transformation.

### Attenuation

Point lights and spotlights use distance-based attenuation:

**F_att = 1 / (k_c + k_l · d + k_q · d²)**

where d is the distance from the fragment to the light, and k_c, k_l, k_q are the constant, linear, and quadratic attenuation coefficients.

### Spotlight Cone

Spotlights add a smooth angular falloff between an inner cone angle θ_inner and an outer cone angle θ_outer:

**I_spot = clamp((cos α − cos θ_outer) / (cos θ_inner − cos θ_outer), 0, 1)**

where α is the angle between the spotlight direction and the vector from the light to the fragment. This gives a hard bright core that fades smoothly at the edges.

---

## Phong Shading vs Gouraud Shading

The application supports **two shading models**, toggled at runtime with the **P** key or the UI panel.

### Phong Shading (per-fragment)

In Phong shading, the lighting equation is evaluated **for every fragment** (pixel). The vertex shader passes interpolated view-space positions and normals to the fragment shader, which then computes:

**I = I_a + Σ F_att,i · S_i · (I_d,i + I_s,i)**

where for each light i:

- **Ambient:**

  **I_a = k_a · L_a · σ_ambient**

- **Diffuse (Lambertian):**

  **I_d = k_d · L_d · max(N̂ · L̂, 0)**

  The surface is brightest when the normal N̂ is aligned with the light direction L̂ and dark when perpendicular or facing away.

- **Specular (Phong reflection):**

  **I_s = k_s · L_s · [max(V̂ · R̂, 0)]^α**

  where R̂ = reflect(−L̂, N̂) is the mirror reflection direction and α is the material shininess. Higher α produces a smaller, sharper highlight.

**Advantages:** Smooth specular highlights, correct per-pixel lighting, accurate spotlight edges.

### Gouraud Shading (per-vertex)

In Gouraud shading, the **same lighting equation** is evaluated at each **vertex**. The resulting color is then interpolated across the triangle by the GPU rasterizer.

**Advantages:** Faster (fewer lighting calculations).

**Disadvantages:** Specular highlights can be missed entirely on large triangles if the highlight falls between vertices. Color banding is visible on low-poly meshes.

### Comparison

| Property | Phong (per-fragment) | Gouraud (per-vertex) |
|----------|---------------------|----------------------|
| Lighting evaluated at | every pixel | every vertex |
| Specular quality | smooth, accurate | may miss highlights |
| Performance cost | higher | lower |
| Best for | smooth surfaces, close-ups | distant objects, performance |

---

## Normal Mapping / Bump Mapping

Normal mapping is a technique that adds fine surface detail **without adding geometry**. Instead of increasing the polygon count, we store per-texel surface normals in a texture (the *normal map*) and use them during lighting instead of the interpolated vertex normal.

### The Normal Map Texture

A normal map is an RGB image where each texel encodes a surface normal in **tangent space**:

**n̂_tangent = texture(normalMap, uv).rgb · 2 − 1**

The encoding maps the [−1, 1] range into [0, 1] for storage. The blue channel (Z) is typically dominant, which is why normal maps appear mostly blue — this represents a normal pointing straight "out" of the surface.

### Tangent Space and the TBN Matrix

Each vertex carries three vectors that define a local coordinate frame on the surface:

- **T̂** — **Tangent** (along the U texture axis)
- **B̂** — **Bitangent** (along the V texture axis)
- **N̂** — **Normal** (perpendicular to the surface)

These are transformed into view space using the normal matrix and assembled into the **TBN matrix**:

```
TBN = | T_x  B_x  N_x |
      | T_y  B_y  N_y |
      | T_z  B_z  N_z |
```

This matrix converts a normal from tangent space to view space:

**n̂_view = normalize(TBN · n̂_tangent)**

### In the Shader

The Phong fragment shader checks whether a normal map is bound:

```glsl
vec3 calc_normal() {
    if (u_has_normal_map) {
        vec3 n = texture(u_normal_map, v_tex_coords).rgb;
        n = n * 2.0 - 1.0;           // decode [0,1] → [-1,1]
        return normalize(v_tbn * n);  // tangent → view space
    }
    return normalize(v_normal_view);  // fallback: smooth vertex normal
}
```

The resulting view-space normal is then used in the standard Phong lighting equation, providing the illusion of bumps, cracks, and grooves without any extra geometry.

### In This Application

Two normal maps can be toggled from the panel:

| Object | Normal Map File |
|--------|----------------|
| Sphere (red, front-right) | `assets/brick_normalmap.png` |
| BigSphere (grey, far back) | `assets/normal_map.jpg` |

---

## Project Structure

```
gk4/
├── assets/
│   ├── shaders/
│   │   ├── phong.vert / phong.frag       # Per-fragment (Phong) shading
│   │   ├── gouraud.vert / gouraud.frag   # Per-vertex (Gouraud) shading
│   │   ├── grid.vert / grid.frag         # Infinite grid floor
│   │   └── light_marker.vert / .frag     # Light source markers & cones
│   ├── models/car/                        # OBJ + MTL car model
│   ├── brick_normalmap.png                # Brick normal map
│   └── normal_map.jpg                     # Abstract normal map
├── src/
│   ├── main.rs           # Entry point, event loop, input handling
│   ├── scene.rs          # Scene objects, moving object, lights
│   ├── renderer.rs       # Uniform management, draw calls
│   ├── camera.rs         # 5 camera types (Static, Tracking, TPP, FPP, Free)
│   ├── light.rs          # Light struct (Point, Spot, Directional)
│   ├── light_markers.rs  # Glowing spheres & direction cones at light positions
│   ├── grid.rs           # Procedural grid floor
│   ├── model.rs          # OBJ/MTL loader with mesh merging
│   ├── primitives.rs     # Sphere, torus, cube, cone generators
│   ├── types.rs          # Material, Transform
│   ├── vertex.rs         # Vertex struct (position, normal, UV, TBN)
│   └── ui.rs             # egui options panel
├── Cargo.toml
└── README.md
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| `glium` 0.36 | OpenGL 3.3 context & rendering |
| `egui` 0.33 + `egui_glium` 0.33 | Immediate-mode GUI |
| `cgmath` 0.18 | Linear algebra (vectors, matrices) |
| `tobj` 4.0 | OBJ/MTL model loading |
| `image` 0.25 | Texture image loading (PNG, JPG) |
