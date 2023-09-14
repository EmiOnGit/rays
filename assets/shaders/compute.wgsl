
@group(0) @binding(0)
var<uniform> seed: u32;

@group(0) @binding(1)
var tex: texture_storage_2d<rgba32float, read_write>;

@compute
@workgroup_size(8, 8, 1)
fn main(
    @builtin(global_invocation_id)
    invocation_id: vec3<u32>
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let color = vec4<f32>(0.,0.,f32(seed) / 256.,1.);
    textureStore(tex,location, color);
}