// Vertex shader

@group(0) @binding(0)
var tex: texture_2d<f32>;
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
   // -1 -1 = left bottom
    var xy: vec2<f32>;
    if (i32(in_vertex_index) == 0 || i32(in_vertex_index) == 5) {
        xy = vec2<f32>(-1.,-1.);
    }
    else if (i32(in_vertex_index) == 1)  {
    xy = vec2<f32>(1.,-1.);
    }
    else if (i32(in_vertex_index) == 2 || i32(in_vertex_index) == 3) {
    xy = vec2<f32>(1.,1.);
    }
    else if (i32(in_vertex_index) == 4) {
    xy = vec2<f32>(-1.,1.);
    }
    out.clip_position = vec4<f32>(xy, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureLoad(tex, vec2<u32>(u32(in.clip_position.x), u32(in.clip_position.y)),0);
    color[3] = 0.5;
    return color;
}
 