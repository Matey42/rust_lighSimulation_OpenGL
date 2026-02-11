#version 330 core

// ── Interpolated from vertex shader ──
in vec3 v_position_view;
in vec3 v_normal_view;
in vec2 v_tex_coords;
in mat3 v_tbn;

// ── Output ──
out vec4 frag_color;

// ── Material ──
uniform vec3  u_material_ambient;
uniform vec3  u_material_diffuse;
uniform vec3  u_material_specular;
uniform float u_material_shininess;
uniform bool  u_has_diffuse_tex;
uniform sampler2D u_diffuse_tex;
uniform bool  u_has_normal_map;
uniform sampler2D u_normal_map;

// ── Light structures (max 8) ──
#define MAX_LIGHTS 8

struct Light {
    int   type;           // 0 = point, 1 = spot, 2 = directional
    vec3  position_view;  // in view space
    vec3  direction_view; // in view space (for spot / directional)
    vec3  ambient;
    vec3  diffuse;
    vec3  specular;
    float constant_att;
    float linear_att;
    float quadratic_att;
    float cutoff;         // cos(inner angle) for spot
    float outer_cutoff;   // cos(outer angle) for spot
};

uniform int   u_num_lights;
uniform Light u_lights[MAX_LIGHTS];

// ── Fog ──
uniform bool  u_fog_enabled;
uniform vec3  u_fog_color;
uniform float u_fog_density;

// ── Day / night ambient multiplier ──
uniform float u_ambient_strength;

// ── Functions ──

vec3 calc_normal() {
    if (u_has_normal_map) {
        vec3 n = texture(u_normal_map, v_tex_coords).rgb;
        n = n * 2.0 - 1.0; // [0,1] → [-1,1]
        return normalize(v_tbn * n);
    }
    return normalize(v_normal_view);
}

float calc_attenuation(Light light, float dist) {
    return 1.0 / (light.constant_att + light.linear_att * dist +
                   light.quadratic_att * dist * dist);
}

vec3 calc_light(Light light, vec3 normal, vec3 frag_pos, vec3 view_dir, vec3 base_color) {
    vec3 light_dir;
    float attenuation = 1.0;

    if (light.type == 2) {
        // Directional
        light_dir = normalize(-light.direction_view);
    } else {
        vec3 to_light = light.position_view - frag_pos;
        float dist = length(to_light);
        light_dir = normalize(to_light);
        attenuation = calc_attenuation(light, dist);
    }

    // Spotlight cone
    float spot_intensity = 1.0;
    if (light.type == 1) {
        float theta = dot(light_dir, normalize(-light.direction_view));
        float epsilon = light.cutoff - light.outer_cutoff;
        spot_intensity = clamp((theta - light.outer_cutoff) / epsilon, 0.0, 1.0);
    }

    // Phong components
    vec3 ambient  = light.ambient * base_color * u_ambient_strength;

    float diff    = max(dot(normal, light_dir), 0.0);
    vec3 diffuse  = light.diffuse * diff * base_color;

    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec    = pow(max(dot(view_dir, reflect_dir), 0.0), u_material_shininess);
    vec3 specular = light.specular * spec * u_material_specular;

    return ambient + (diffuse + specular) * attenuation * spot_intensity;
}

void main() {
    vec3 normal   = calc_normal();
    vec3 view_dir = normalize(-v_position_view); // camera at origin in view space

    vec3 base_color = u_material_diffuse;
    if (u_has_diffuse_tex) {
        base_color *= texture(u_diffuse_tex, v_tex_coords).rgb;
    }

    vec3 result = vec3(0.0);
    for (int i = 0; i < u_num_lights && i < MAX_LIGHTS; i++) {
        result += calc_light(u_lights[i], normal, v_position_view, view_dir, base_color);
    }

    // Fog (exponential squared) — blend toward fog color = sky color
    // This naturally causes contrast loss (all colors converge) and vanishing
    float alpha = 1.0;
    if (u_fog_enabled) {
        float dist = length(v_position_view);
        float fog_factor = clamp(exp(-pow(u_fog_density * dist, 2.0)), 0.0, 1.0);
        result = mix(u_fog_color, result, fog_factor);
        alpha = fog_factor;
    }

    frag_color = vec4(result, alpha);
}
