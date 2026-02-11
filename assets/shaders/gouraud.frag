#version 330 core

in vec3  v_color;
in vec2  v_tex_coords_out;
in float v_fog_factor;

out vec4 frag_color;

uniform bool      u_has_diffuse_tex;
uniform sampler2D u_diffuse_tex;
uniform bool      u_fog_enabled;
uniform vec3      u_fog_color;

void main() {
    vec3 color = v_color;

    if (u_has_diffuse_tex) {
        color *= texture(u_diffuse_tex, v_tex_coords_out).rgb;
    }

    if (u_fog_enabled) {
        color = mix(u_fog_color, color, v_fog_factor);
    }

    frag_color = vec4(color, 1.0);
}
