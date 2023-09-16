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
fn in_unit_sphere(seed: u32) -> vec3f {
    let seed1 = pcg_hash(seed);
    let seed2 = pcg_hash(seed1);
    let x = rand(seed);
    let y = rand(seed1);
    let z = rand(seed2);
    return normalize(vec3f(x * 2. - 1., y * 2. - 1., z * 2. - 1.));


}
struct Globals {
    seed: u32,
    bounces: u32,
    sky_color: vec4f,
}
struct Camera {
    fov: vec2f,
    viewport: vec2f,
    camera_position: vec4f,
    o1: vec4f,
    o2: vec4f,
    inverse_projection: mat4x4f,
    inverse_view: mat4x4f,
    o3: mat4x4f,
    
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
var<uniform> camera: Camera;

@group(0) @binding(2)
var output_texture: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(3)
var<storage> spheres: array<Sphere>;

@group(0) @binding(4)
var<storage> materials: array<Material>;
fn calc_ray_direction(
    invocation_id: vec3u
) -> vec3f {
    var coord = vec2f(f32(invocation_id.x), f32(invocation_id.y)) / camera.viewport ;
    coord = coord * 2. - vec2f(1.);
    let target1 = camera.inverse_projection * vec4f(coord,1.,1.);
    let target2 = normalize(target1.xyz / target1.w);
    let ray_direction = (camera.inverse_view * vec4f(target2, 0.)).xyz;
    return ray_direction;
}
struct HitPayload {
    normal: vec3f,
    hit_position: vec3f,
    sphere_index: i32,
    hit_distance: f32,
}
fn trace_ray(
    ray_origin: vec3f,
    ray_direction: vec3f,
) -> HitPayload {
    var closest_hit_distance = 999999999.;
    var closest_index = -1;
    let sphere_count = i32(arrayLength(&spheres));
    for (var i = 0; i < sphere_count; i++) {
        let sphere = spheres[i];
        let origin = ray_origin - sphere.center; // camera at origin for now
        let a = dot(ray_direction, ray_direction);
        let b = 2. * dot(origin, ray_direction);
        let c = dot(origin,origin) - sphere.radius * sphere.radius;
        let discriminant = b * b - 4. * a * c;
        if (discriminant < 0.) {
            continue;
        }
        let hit_distance = (-b - sqrt(discriminant)) / (2. * a);
        if (hit_distance > 0. && closest_hit_distance > hit_distance ) {
            closest_hit_distance = hit_distance;
            closest_index = i;
        }
        
    }
    var payload: HitPayload;
    payload.sphere_index = closest_index;

    if (closest_index == -1) {
        // no hit
        return payload;
    }
    let sphere = spheres[closest_index];
    let origin = ray_origin - sphere.center; 

    let hit_point = (origin + ray_direction * closest_hit_distance);
    let normal = normalize(hit_point);
    payload.normal = normal;
    payload.hit_position = hit_point + sphere.center;
    payload.hit_distance = closest_hit_distance;
    return payload;
}
@compute
@workgroup_size(8, 8, 1)
fn main(
    @builtin(global_invocation_id)
    invocation_id: vec3<u32>
) {
    var seed = pcg_hash(globals.seed * (invocation_id.y + 3u) + (globals.seed ^ (invocation_id.x * 4389u)));
    var ray_direction = calc_ray_direction(invocation_id);
    var ray_origin = camera.camera_position.xyz;
    var light = vec3f(0.);
    var contribution = vec3f(1.);
    let image_location = vec2i(i32(invocation_id.x), i32(invocation_id.y));
    var bounced = globals.bounces;
    for (var i = 0; i < i32(globals.bounces) ; i++) {
        let payload = trace_ray(ray_origin, ray_direction);
        if (payload.sphere_index == -1) {
                
                light = light + globals.sky_color.xyz * contribution;
                bounced = 1u + u32(i);
                break;
        }
        let sphere = spheres[payload.sphere_index];
        let material = materials[sphere.material_index];
        ray_origin = payload.hit_position + payload.normal * 0.0001;
        seed = pcg_hash(seed + 1u);
        ray_direction = normalize(in_unit_sphere(seed) + payload.normal);
        contribution *= material.albedo.xyz;
    }

    let color = light;
    
    // hit
    textureStore(output_texture, image_location, vec4f(color, 1.));


    // let ran = invocation_id.x + invocation_id.y * 747796405u;
    // let r = rand(ran * globals.seed);
    // let g = rand(ran * 392813u * globals.seed);
    // let b = rand(ran * 436727u * globals.seed);
    // let color = vec4<f32>(r, g, b * spheres[0].radius,1.);
    // textureStore(output_texture, location, color);
}
