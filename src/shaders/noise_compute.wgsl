struct PrimeIndices {
    data: [[stride(4)]] array<f32>;
};

[[group(0), binding(0)]]
var<storage, write> noise_output: [[stride(4)]] PrimeIndices;

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    noise_output.data[global_id.x + (global_id.y * u32(16)) + (global_id.z * u32(16) * u32(16))] = 1.0; 
}