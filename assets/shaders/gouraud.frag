#version 330 core

in vec3  v_color;
in vec2  v_tex_coords_out;
in float v_fog_factor;

out vec4 frag_color;

uniform bool      u_has_diffuse_tex;
uniform sampler2D u_diffuse_tex;
uniform bool      u_has_normal_map;
uniform sampler2D u_normal_map;
uniform bool      u_fog_enabled;
uniform vec3      u_fog_color;

void main() {
    vec3 color = v_color;

    if (u_has_diffuse_tex) {
        color *= texture(u_diffuse_tex, v_tex_coords_out).rgb;
    }

    float alpha = 1.0;
    if (u_fog_enabled) {
        color = mix(u_fog_color, color, v_fog_factor);
        alpha = v_fog_factor;
    }

    frag_color = vec4(color, alpha);
}
