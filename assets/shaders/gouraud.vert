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
uniform mat3 normal_matrix;

// ── Material ──
uniform vec3  u_material_ambient;
uniform vec3  u_material_diffuse;
uniform vec3  u_material_specular;
uniform float u_material_shininess;

// ── Normal mapping ──
uniform bool      u_has_normal_map;
uniform sampler2D u_normal_map;

// ── Light structures (max 8) ──
#define MAX_LIGHTS 8

struct Light {
    int   type;
    vec3  position_view;
    vec3  direction_view;
    vec3  ambient;
    vec3  diffuse;
    vec3  specular;
    float constant_att;
    float linear_att;
    float quadratic_att;
    float cutoff;
    float outer_cutoff;
};

uniform int   u_num_lights;
uniform Light u_lights[MAX_LIGHTS];
uniform float u_ambient_strength;

// ── Fog ──
uniform bool  u_fog_enabled;
uniform vec3  u_fog_color;
uniform float u_fog_density;

// ── Outputs ──
out vec3 v_color;
out vec2 v_tex_coords_out;
out float v_fog_factor;

void main() {
    vec4 pos_view   = view * model * vec4(position, 1.0);
    vec3 frag_pos   = pos_view.xyz;
    vec3 norm       = normalize(normal_matrix * normal);

    // Normal mapping: perturb vertex normal via TBN matrix
    if (u_has_normal_map) {
        vec3 T = normalize(normal_matrix * tangent);
        vec3 B = normalize(normal_matrix * bitangent);
        vec3 N = norm;
        mat3 TBN = mat3(T, B, N);
        vec3 map_n = textureLod(u_normal_map, tex_coords, 0.0).rgb;
        map_n = map_n * 2.0 - 1.0;
        norm = normalize(TBN * map_n);
    }

    vec3 view_dir   = normalize(-frag_pos);

    // Fog factor (per-vertex)
    float cam_dist = length(frag_pos);
    v_fog_factor = 1.0;
    if (u_fog_enabled) {
        v_fog_factor = clamp(exp(-pow(u_fog_density * cam_dist, 2.0)), 0.0, 1.0);
    }

    vec3 base_color = u_material_diffuse;
    vec3 result     = vec3(0.0);

    for (int i = 0; i < u_num_lights && i < MAX_LIGHTS; i++) {
        Light light = u_lights[i];
        vec3 light_dir;
        float attenuation = 1.0;

        if (light.type == 2) {
            light_dir = normalize(-light.direction_view);
        } else {
            vec3 to_light = light.position_view - frag_pos;
            float dist = length(to_light);
            light_dir = normalize(to_light);
            attenuation = 1.0 / (light.constant_att + light.linear_att * dist +
                                  light.quadratic_att * dist * dist);
        }

        float spot_intensity = 1.0;
        if (light.type == 1) {
            float theta = dot(light_dir, normalize(-light.direction_view));
            float epsilon = light.cutoff - light.outer_cutoff;
            spot_intensity = clamp((theta - light.outer_cutoff) / epsilon, 0.0, 1.0);
        }

        vec3 ambient  = light.ambient * base_color * u_ambient_strength;
        float diff    = max(dot(norm, light_dir), 0.0);
        vec3 diffuse  = light.diffuse * diff * base_color;

        vec3 reflect_dir = reflect(-light_dir, norm);
        float spec    = pow(max(dot(view_dir, reflect_dir), 0.0), u_material_shininess);
        vec3 specular = light.specular * spec * u_material_specular;

        result += ambient + (diffuse + specular) * attenuation * spot_intensity;
    }

    v_color = result;
    v_tex_coords_out = tex_coords;
    gl_Position = projection * pos_view;
}
