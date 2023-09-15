fn pcg_hash(v: u32) -> u32 {
    let p1 = 747796405u;
    let p2 = 2891336453u;
    let p3 = 277803737u;
    let state = v * p1 + p2;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * p3;

    return((word >> 22u) ^ word);
}
fn rand(seed: u32) -> f32 {
    let hashed = pcg_hash(seed);
    return(f32(hashed) / 0xffffffff.);
}
struct Globals {
    seed: u32,
    viewport_width: f32,
    viewport_height: f32,
    inverse_projection: mat4x4f,
    inverse_view: mat4x4f,
    camera_position: vec3f,
}
struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material_index: u32,
}
struct Material {
    albedo: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var output_texture: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var<storage> spheres: array<Sphere>;

@group(0) @binding(3)
var<storage> materials: array<Material>;

@compute
@workgroup_size(8, 8, 1)
fn main(
    @builtin(global_invocation_id)
    invocation_id: vec3<u32>
) {
    // calculate ray directions
    var coord = vec2f(f32(invocation_id.x), f32(invocation_id.y)) / vec2f(globals.viewport_width, globals.viewport_height) ;
    coord = coord * 2. - vec2f(1.);
    let target1 = globals.inverse_projection * vec4f(coord,1.,1.);
    let target2 = normalize(target1.xyz / target1.w);
    let ray_direction = (globals.inverse_view * vec4f(target2, 0.)).xyz;
    // trace ray

    let sphere = spheres[0];
    let origin = globals.camera_position - sphere.center; // camera at origin for now
    let radius = sphere.radius;
    let a = dot(ray_direction, ray_direction);
    let b = 2. * dot(origin, ray_direction);
    let c = dot(origin,origin) - radius * radius;
    let discriminant = b * b - 4. * a * c;

    let location = vec2i(i32(invocation_id.x), i32(invocation_id.y));

    if (discriminant < 0.) {
        // no hit
        textureStore(output_texture, location, vec4f(0.));
    } else {
        // hit
        let material = materials[sphere.material_index];
        textureStore(output_texture, location, vec4f(material.albedo));
    }


    // let ran = invocation_id.x + invocation_id.y * 747796405u;
    // let r = rand(ran * globals.seed);
    // let g = rand(ran * 392813u * globals.seed);
    // let b = rand(ran * 436727u * globals.seed);
    // let color = vec4<f32>(r, g, b * spheres[0].radius,1.);
    // textureStore(output_texture, location, color);
}
