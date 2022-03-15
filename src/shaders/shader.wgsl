// Vertex shader
struct CameraUniform {
    projection: mat4x4<f32>;
    transform: mat4x4<f32>;
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
    [[location(4)]] camera_position : vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(in : VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.projection * camera.transform * vec4<f32>(in.position, 1.0);
    out.position = in.position;
    out.color = in.color;
    out.normal = in.normal;
    out.uv = in.uv;
    out.camera_position = (camera.transform * vec4<f32>(0.0, 0.0, 0.0, 0.0)).xyz;
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
    var col: vec4<f32> = vec4<f32>(in.color, 1.0); //textureSample(t_diffuse, s_diffuse, in.uv) * in.color;

    var light_dir: vec3<f32> = normalize(vec3<f32>(-0.5, 0.6, -0.3));
    var ambient_light: f32 = 0.3;
    var light_dot: f32 = clamp(dot(in.normal, light_dir), 0.0, 1.0);

    var shading: f32 = light_dot;

    col = vec4<f32>(col.xyz * (shading + ambient_light), 1.0);
    col = col;

    return col;
}