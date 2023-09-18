const MIN_HIT_DISTANCE: f32 = 0.000000001;
fn pcg_hash(v: u32) -> u32 {
    let p1 = 747796405u;
    let p2 = 2891336453u;
    let p3 = 277803737u;
    let state = v * p1 + p2;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * p3;

    return((word >> 22u) ^ word);
}
fn randf(seed: u32) -> f32 {
    let hashed = pcg_hash(seed) % 0xffffffffu;
    return(f32(hashed) / 0xffffffff.);
}

// source http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
const PI: f32 = 3.14159265358979;
fn radical_inverse_vdc(bits_input: u32) -> f32 {
    var bits = (bits_input << 16u) | (bits_input >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}
fn hammersley2d(i: u32, n: u32) -> vec2f {
    return vec2f(f32(i)/f32(n), radical_inverse_vdc(i));
}

fn hemisphereSample_uniform(u: f32, v: f32) -> vec3f {
    let phi = v * 2.0 * PI;
    let cosTheta = 1.0 - u;
    let sinTheta = sqrt(1.0 - cosTheta * cosTheta);
    return vec3f(cos(phi) * sinTheta, sin(phi) * sinTheta, cosTheta);
}

fn in_unit_sphere(i: u32, n: u32) -> vec3f {
    let hammersley = hammersley2d(i,n);
    return(hemisphereSample_uniform(hammersley.x, hammersley.y));
}

fn refract(uv: vec3f, n: vec3f, etai_over_etat: f32) -> vec3f{
    let cos_theta = min(dot(-uv, n), 1.);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -sqrt(abs(1. - dot(r_out_perp, r_out_perp))) * n;
    return r_out_parallel + r_out_perp;
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
    _offset1: vec4f,
    _offset2: vec4f,
    inverse_projection: mat4x4f,
    inverse_view: mat4x4f,
    _offset3: mat4x4f,
}
struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material_index: u32,
}
struct Material {
    dialectric: f32,
    specular_intensity: f32,
    roughness: f32,
    fog: f32,
    albedo: vec4<f32>,
    emission_color: vec3<f32>,
    emission_strength: f32,
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
    let target1 = transpose(camera.inverse_projection) * vec4f(coord,1.,1.);
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
    var closest_hit_distance = 0xffffffff.;
    var closest_index = -1;
    let sphere_count = i32(arrayLength(&spheres));
    for (var i = 0; i < sphere_count; i++) {
        let sphere = spheres[i];
        let origin = ray_origin - sphere.center;
        let a = dot(ray_direction, ray_direction);
        let b = 2. * dot(origin, ray_direction);
        let c = dot(origin,origin) - sphere.radius * sphere.radius;
        let discriminant = b * b - 4. * a * c;
        if (discriminant < 0.) {
            continue;
        }
        let hit_distance = (-b - sqrt(discriminant)) / (2. * a);
        if (hit_distance > MIN_HIT_DISTANCE && closest_hit_distance > hit_distance ) {
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
@workgroup_size(16, 16, 1)
fn main(
    @builtin(global_invocation_id)
    invocation_id: vec3<u32>,
) {
    var seed = pcg_hash(globals.seed + (invocation_id.x + 747796405u) * (invocation_id.y + 48327948u));
    let n_max = 45794357u;
    var ii = (0u + seed) % 45794357u;
    var ray_direction = calc_ray_direction(invocation_id);
    var ray_origin = camera.camera_position.xyz;
    var light = vec3f(0.);
    var contribution = vec3f(1.);
    let image_location = vec2i(i32(invocation_id.x), i32(invocation_id.y));
    var bounced = globals.bounces;
    for (var i: u32 = 0u; i < globals.bounces ; i = i + 1u) {
        let payload = trace_ray(ray_origin, ray_direction);
        if (payload.sphere_index == -1) {
            light = light + globals.sky_color.xyz * contribution;
            bounced = 1u + i;
            break;
        }
        let sphere = spheres[payload.sphere_index];
        let material = materials[sphere.material_index];
        ray_origin = payload.hit_position;
        seed = pcg_hash(seed + 5u);

        let computed_roughness = pow(material.roughness, 2.);
        let n = normalize(payload.normal + computed_roughness * in_unit_sphere(ii, n_max));
        ii = ii + 1u;
        let reflected = reflect(ray_direction, n);
        ray_direction = reflected;
        
        contribution *= material.albedo.xyz * material.specular_intensity;
        light += material.emission_color * material.emission_strength * contribution;
    }

    let color = light / f32(bounced);
    
    // hit
    let old_color = textureLoad(output_texture, image_location);
    textureStore(output_texture, image_location, vec4f(color, 1.) + old_color);
}
