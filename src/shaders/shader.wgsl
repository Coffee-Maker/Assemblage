// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>;
    view_pos: vec4<f32>;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position : vec3<f32>;
    [[location(1)]] color : vec3<f32>;
    [[location(2)]] normal : vec3<f32>;
    [[location(3)]] uv : vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] position : vec3<f32>;
    [[location(1)]] color : vec3<f32>;
    [[location(2)]] normal : vec3<f32>;
    [[location(3)]] uv : vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(in : VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.position = in.position;
    out.color = in.color;
    out.normal = in.normal;
    out.uv = in.uv;
    return out;
}

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return (a * (1.0 - t)) + (b * t);
}
fn lerp4(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32>{
    return vec4<f32>(lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t), lerp(a.w, b.w, t));
}

 // Fragment shader
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var col: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.uv);

    var light_dir: vec3<f32> = normalize(vec3<f32>(-0.5, 0.6, -0.3));
    var ambient_light: f32 = 0.5;
    var light_dot: f32 = clamp(dot(in.normal, light_dir), 0.0, 1.0);

    var shading: f32 = light_dot;

    var specular_intensity: f32 = clamp(dot(light_dir, reflect(normalize(in.position - camera.view_pos.xyz), in.normal)), 0.0, 1.0);
    specular_intensity = pow(specular_intensity, 2.0) * 1.0;
    
    var metalicity = 1.0;
    var specular_color: vec4<f32> = lerp4(specular_intensity * vec4<f32>(1.0, 1.0, 1.0, 1.0), specular_intensity * col, metalicity); // Blend metalic
    specular_color.a = 0.0;

    col = vec4<f32>(col.xyz * (shading + ambient_light), 1.0);
    col = col + specular_color;

    return col;
}