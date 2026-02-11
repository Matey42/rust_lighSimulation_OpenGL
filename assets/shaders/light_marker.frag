#version 330 core

in vec3 v_normal_local;
out vec4 frag_color;

uniform vec3 u_light_color;

void main() {
    // Slight shading so the shape looks 3D (brighter facing camera)
    vec3 n = normalize(v_normal_local);
    float facing = 0.5 + 0.5 * max(n.z, 0.0);
    
    // Compute overall brightness of the color to detect cones (delicate) vs spheres (bright)
    float brightness = dot(u_light_color, vec3(0.299, 0.587, 0.114));
    
    // Emissive glow
    vec3 color = u_light_color * (0.6 + 0.4 * facing);
    
    // White-hot core (subtle for dim colors like cones)
    float core = pow(facing, 4.0) * 0.3;
    color += vec3(core);
    
    // Alpha: bright objects (sphere markers) stay opaque,
    // dim objects (cones) become translucent
    float alpha = clamp(brightness * 2.5, 0.15, 0.95);
    
    frag_color = vec4(color, alpha);
}
