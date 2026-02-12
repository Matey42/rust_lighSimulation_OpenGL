#version 330 core

in vec3 v_normal_local;
in vec3 v_view_pos;
in vec3 v_local_pos;
out vec4 frag_color;

uniform vec3 u_light_color;
uniform float u_alpha;

// ── Fog ──
uniform bool  u_fog_enabled;
uniform vec3  u_fog_color;
uniform float u_fog_density;

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
    float alpha = clamp(brightness * 2.5, 0.15, 0.95) * u_alpha;
    
    // Cone fade: v_local_pos.z goes from 0 (tip) to -1 (base) in the unit cone.
    // Smoothly fade out the last 60% of the cone length so it dissolves into
    // the background like real light — bright near the source, invisible at range.
    float t = clamp(-v_local_pos.z, 0.0, 1.0); // 0=tip, 1=base
    float cone_fade = 1.0 - smoothstep(0.15, 1.0, t);
    alpha *= cone_fade;
    
    // Fog — markers fade and vanish in fog just like everything else
    if (u_fog_enabled) {
        float dist = length(v_view_pos);
        float fog_factor = clamp(exp(-pow(u_fog_density * dist, 2.0)), 0.0, 1.0);
        color = mix(u_fog_color, color, fog_factor);
        alpha *= fog_factor;
    }
    
    frag_color = vec4(color, alpha);
}
