#version 330 core

in vec3 v_normal_local;
out vec4 frag_color;

uniform vec3 u_light_color;

void main() {
    // Slight shading so the sphere looks 3D (brighter facing camera)
    vec3 n = normalize(v_normal_local);
    float facing = 0.5 + 0.5 * max(n.z, 0.0); // z = toward camera in view space approx.
    
    // Emissive glow — always bright, not affected by scene lighting
    vec3 color = u_light_color * (0.6 + 0.4 * facing);
    
    // Add a white-hot core
    float core = pow(facing, 4.0) * 0.5;
    color += vec3(core);
    
    frag_color = vec4(color, 0.95);
}
