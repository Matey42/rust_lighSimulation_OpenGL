#version 330 core

in vec3 v_world_pos;
in vec3 v_view_pos;
in vec3 v_world_normal;

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

// ── Spotlight data for headlight illumination on the floor ──
// We support up to 4 spotlights on the grid
const int MAX_GRID_SPOTS = 4;
uniform int   u_num_spots;
uniform vec3  u_spot_position[MAX_GRID_SPOTS];
uniform vec3  u_spot_direction[MAX_GRID_SPOTS];
uniform vec3  u_spot_color[MAX_GRID_SPOTS];
uniform float u_spot_cutoff[MAX_GRID_SPOTS];       // cos(inner)
uniform float u_spot_outer_cutoff[MAX_GRID_SPOTS];  // cos(outer)

// ──────────── Pristine grid (anti-aliased) ────────────
float grid_line(vec2 p, float cell_size, float line_w) {
    vec2 grid_uv = p / cell_size;
    vec2 deriv   = fwidth(grid_uv);
    vec2 half_lw = vec2(line_w / cell_size) * 0.5;

    vec2 grid_aa = smoothstep(vec2(0.0), deriv, abs(fract(grid_uv - 0.5) - 0.5) - half_lw);
    float line = 1.0 - min(grid_aa.x, grid_aa.y);
    return line;
}

float axis_line(float coord, float line_w) {
    float d = fwidth(coord);
    return 1.0 - smoothstep(0.0, d, abs(coord) - line_w * 0.5);
}

// ── Compute spotlight contribution on the floor ──
vec3 compute_spot_lighting(vec3 world_pos, vec3 world_normal) {
    vec3 total = vec3(0.0);

    for (int i = 0; i < MAX_GRID_SPOTS; i++) {
        if (i >= u_num_spots) break;

        vec3 light_to_frag = world_pos - u_spot_position[i];
        float dist = length(light_to_frag);
        vec3 light_dir = normalize(light_to_frag);

        // Spotlight cone
        vec3 spot_dir = normalize(u_spot_direction[i]);
        float theta = dot(light_dir, spot_dir);
        float epsilon = u_spot_cutoff[i] - u_spot_outer_cutoff[i];
        float intensity = clamp((theta - u_spot_outer_cutoff[i]) / max(epsilon, 0.001), 0.0, 1.0);

        if (intensity <= 0.0) continue;

        // Diffuse (floor normal is always up)
        float ndotl = max(dot(world_normal, -light_dir), 0.0);

        // Attenuation
        float attenuation = 1.0 / (1.0 + 0.09 * dist + 0.032 * dist * dist);

        total += u_spot_color[i] * ndotl * intensity * attenuation;
    }

    return total;
}

void main() {
    vec2 world_xz = v_world_pos.xz;
    float dist_from_camera = length(v_view_pos);

    // ── Fade with distance ──
    float fade = 1.0 - smoothstep(u_fade_radius * 0.3, u_fade_radius, dist_from_camera);
    if (fade <= 0.001) discard;

    // ── Base grid colours ──
    float brightness = mix(0.15, 1.0, u_ambient_strength);

    vec3 bg_color       = vec3(0.22, 0.22, 0.22) * brightness;
    vec3 sub_grid_color = vec3(0.30, 0.30, 0.30) * brightness;
    vec3 main_grid_color= vec3(0.42, 0.42, 0.42) * brightness;
    vec3 x_axis_color   = vec3(0.75, 0.20, 0.20) * brightness;
    vec3 z_axis_color   = vec3(0.20, 0.35, 0.75) * brightness;

    // ── Compute grid lines ──
    float lw_sub  = u_line_width * 0.6;
    float lw_main = u_line_width;

    float sub_line  = grid_line(world_xz, u_sub_grid_size, lw_sub);
    float main_line = grid_line(world_xz, u_grid_size, lw_main);

    float x_axis = axis_line(v_world_pos.z, lw_main * 3.0);
    float z_axis = axis_line(v_world_pos.x, lw_main * 3.0);

    // ── Composite colour ──
    vec3 color = bg_color;
    float alpha = 0.85;

    color = mix(color, sub_grid_color, sub_line * 0.6);
    alpha = mix(alpha, 0.92, sub_line * 0.3);

    color = mix(color, main_grid_color, main_line * 0.9);
    alpha = mix(alpha, 1.0, main_line * 0.4);

    color = mix(color, x_axis_color, x_axis * 0.85);
    color = mix(color, z_axis_color, z_axis * 0.85);
    alpha = mix(alpha, 1.0, max(x_axis, z_axis) * 0.5);

    // ── Spotlight illumination on the floor ──
    vec3 spot_contrib = compute_spot_lighting(v_world_pos, v_world_normal);
    // Add spotlight as additive light on the grid — warm pool of light
    color += spot_contrib * 1.5;

    // ── Distance fade ──
    alpha *= fade;

    // ── Fog ──
    if (u_fog_enabled) {
        float fog_factor = clamp(exp(-pow(u_fog_density * dist_from_camera, 2.0)), 0.0, 1.0);
        color = mix(u_fog_color, color, fog_factor);
        alpha *= fog_factor;
    }

    frag_color = vec4(color, alpha);
}
