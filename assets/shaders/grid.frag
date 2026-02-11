#version 330 core

in vec3 v_world_pos;
in vec3 v_view_pos;

out vec4 frag_color;

// ── Grid parameters ──
uniform float u_grid_size;       // spacing of major grid lines (e.g. 1.0)
uniform float u_sub_grid_size;   // spacing of minor grid lines (e.g. 0.25)
uniform float u_fade_radius;     // distance at which grid fully fades out
uniform float u_line_width;      // line thickness in world units

// ── Fog ──
uniform bool  u_fog_enabled;
uniform vec3  u_fog_color;
uniform float u_fog_density;

// ── Day / night ──
uniform float u_ambient_strength;

// ──────────── Pristine grid (anti-aliased) ────────────
// Based on Evan Wallace / Ben Golus anti-aliased grid technique.
// Returns intensity [0,1] for a grid of given cell size at position p.
float grid_line(vec2 p, float cell_size, float line_w) {
    vec2 grid_uv = p / cell_size;
    vec2 deriv   = fwidth(grid_uv);
    vec2 half_lw = vec2(line_w / cell_size) * 0.5;

    // Smoothstep from edge of line to anti-aliased boundary
    vec2 grid_aa = smoothstep(vec2(0.0), deriv, abs(fract(grid_uv - 0.5) - 0.5) - half_lw);
    // Invert: 1 = on line, 0 = off line. Combine X and Z.
    float line = 1.0 - min(grid_aa.x, grid_aa.y);
    return line;
}

// Returns 1.0 if the position is very close to the given axis line.
float axis_line(float coord, float line_w) {
    float d = fwidth(coord);
    return 1.0 - smoothstep(0.0, d, abs(coord) - line_w * 0.5);
}

void main() {
    vec2 world_xz = v_world_pos.xz;
    float dist_from_camera = length(v_view_pos);

    // ── Fade with distance ──
    float fade = 1.0 - smoothstep(u_fade_radius * 0.3, u_fade_radius, dist_from_camera);
    if (fade <= 0.001) discard;

    // ── Base grid colours (neutral greys – professional engine look) ──
    // Darken slightly with night
    float brightness = mix(0.15, 1.0, u_ambient_strength);

    vec3 bg_color       = vec3(0.22, 0.22, 0.22) * brightness;  // dark grey background
    vec3 sub_grid_color = vec3(0.30, 0.30, 0.30) * brightness;  // subtle subdivision
    vec3 main_grid_color= vec3(0.42, 0.42, 0.42) * brightness;  // major grid lines
    vec3 x_axis_color   = vec3(0.75, 0.20, 0.20) * brightness;  // red  = X axis
    vec3 z_axis_color   = vec3(0.20, 0.35, 0.75) * brightness;  // blue = Z axis

    // ── Compute grid lines ──
    float lw_sub  = u_line_width * 0.6;
    float lw_main = u_line_width;

    float sub_line  = grid_line(world_xz, u_sub_grid_size, lw_sub);
    float main_line = grid_line(world_xz, u_grid_size, lw_main);

    // Axis lines (thicker)
    float x_axis = axis_line(v_world_pos.z, lw_main * 3.0);  // X-axis runs along Z=0
    float z_axis = axis_line(v_world_pos.x, lw_main * 3.0);  // Z-axis runs along X=0

    // ── Composite colour ──
    vec3 color = bg_color;
    float alpha = 0.85; // base opacity for the floor

    // Layer sub-grid
    color = mix(color, sub_grid_color, sub_line * 0.6);
    alpha = mix(alpha, 0.92, sub_line * 0.3);

    // Layer major grid (overrides sub)
    color = mix(color, main_grid_color, main_line * 0.9);
    alpha = mix(alpha, 1.0, main_line * 0.4);

    // Layer axis lines (highest priority)
    color = mix(color, x_axis_color, x_axis * 0.85);
    color = mix(color, z_axis_color, z_axis * 0.85);
    alpha = mix(alpha, 1.0, max(x_axis, z_axis) * 0.5);

    // ── Distance fade ──
    alpha *= fade;

    // ── Fog ──
    if (u_fog_enabled) {
        float fog_factor = exp(-pow(u_fog_density * dist_from_camera, 2.0));
        fog_factor = clamp(fog_factor, 0.0, 1.0);
        color = mix(u_fog_color, color, fog_factor);
        // Also fade alpha in fog to avoid harsh edges
        alpha *= mix(0.3, 1.0, fog_factor);
    }

    frag_color = vec4(color, alpha);
}
