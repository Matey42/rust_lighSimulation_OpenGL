#version 330 core

// ── Per-vertex attributes ──
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coords;
layout(location = 3) in vec3 tangent;
layout(location = 4) in vec3 bitangent;

// ── Uniform matrices ──
uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat3 normal_matrix; // transpose(inverse(view * model))

// ── Outputs to fragment shader ──
out vec3 v_position_view;   // position in view space
out vec3 v_normal_view;     // normal  in view space
out vec2 v_tex_coords;
out mat3 v_tbn;             // tangent-space → view-space

void main() {
    vec4 pos_view = view * model * vec4(position, 1.0);
    v_position_view = pos_view.xyz;

    v_normal_view = normalize(normal_matrix * normal);

    // Build TBN matrix in view space for normal mapping
    vec3 T = normalize(normal_matrix * tangent);
    vec3 B = normalize(normal_matrix * bitangent);
    vec3 N = v_normal_view;
    v_tbn = mat3(T, B, N);

    v_tex_coords = tex_coords;

    gl_Position = projection * pos_view;
}
