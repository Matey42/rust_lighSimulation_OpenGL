#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

out vec3 v_normal_local;
out vec3 v_view_pos;

void main() {
    v_normal_local = normal;
    vec4 pos_view = u_view * u_model * vec4(position, 1.0);
    v_view_pos = pos_view.xyz;
    gl_Position = u_projection * pos_view;
}
