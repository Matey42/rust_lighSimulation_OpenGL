#version 330 core

layout(location = 0) in vec3 position;

uniform mat4 u_view;
uniform mat4 u_projection;

out vec3 v_world_pos;
out vec3 v_view_pos;
out vec3 v_world_normal;

void main() {
    v_world_pos = position;
    v_world_normal = vec3(0.0, 1.0, 0.0); // floor normal always points up
    vec4 view_pos = u_view * vec4(position, 1.0);
    v_view_pos = view_pos.xyz;
    gl_Position = u_projection * view_pos;
}
