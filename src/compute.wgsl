// --- Random ---
fn vec2rand(co: vec2<f32>) -> f32{
  return fract(sin(dot(co.xy ,vec2<f32>(12.9898,78.233))) * 43758.5453);
}
fn randu(seed: ptr<function, u32>) -> u32 {
    var r = *seed;
    r ^= r << 13u;
    r ^= r >> 17u;
    r ^= r << 5u;
    *seed = r;
    return r;
}
fn randf(seed: ptr<function, u32>) -> f32 {
    return f32(randu(seed)) / f32(0xffffffffu);
}
fn randf_range(seed: ptr<function, u32>, min: f32, max: f32) -> f32{
    return randf(seed) * (max - min) + min;
}
fn rand_in_sphere(seed: ptr<function, u32>) -> vec3<f32> {
    loop {
        let x = 2.0 * randf(seed) - 1.0;
        let y = 2.0 * randf(seed) - 1.0;
        let z = 2.0 * randf(seed) - 1.0;

        let len_sqrd = x*x + y*y + z*z;

        if (len_sqrd < 1.0) {
            return vec3<f32>(x, y, z);
        }
    }
    
    // to make compiler happy
    // should be impossible
    return vec3<f32>(0.0, 0.0, 0.0);
}
// --- !Random ---

// --- Ray ---
struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
}
fn ray_new(pos: vec3<f32>, dir: vec3<f32>) -> Ray {
    var ray: Ray;
    ray.pos = pos;
    ray.dir = dir;
    return ray;
}
fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.pos + ray.dir * t;
}
// --- !Ray ---

// --- Camera ---
struct Camera {
    pos: vec3<f32>,
    horizontal: vec3<f32>,
    vertical: vec3<f32>,
    center: vec3<f32>,
}
fn camera_get_ray(camera: Camera, uv: vec2<f32>) -> Ray {
    return ray_new(
        camera.pos,
        normalize(camera.center + camera.horizontal * uv.x + camera.vertical * uv.y - camera.pos)
    );
}
// --- !Camera ---

// --- Hit Record ---
struct HitRecord {
    t: f32,
    pos: vec3<f32>,
    norm: vec3<f32>,

    sphere_index: i32,
}
// --- !Hit Record

// --- Sphere ---
struct Sphere {
    pos: vec3<f32>,
    rad: f32,
    albedo: vec3<f32>,
    roughness: f32
}
fn ray_sphere_intersect(sphere: Sphere, ray: Ray, t_min: f32, t_max: f32, hit_record: ptr<function, HitRecord>) -> bool {
    let pos = ray.pos - sphere.pos;

    let a = dot(ray.dir, ray.dir);
    let b = 2.0 * dot(pos, ray.dir);
    let c = dot(pos, pos) - (sphere.rad * sphere.rad);

    let d = b * b - 4.0 * a * c;
    if d < 0.0 {
        return false;
    }

    let d_sqrt = sqrt(d);
    // let t1 = (-b + d_sqrt) / (2.0 * a);
    let t = (-b - d_sqrt) / (2.0 * a); // this will always be closer

    if t > t_max || t < t_min {
        return false;
    }

    let pos = ray_at(ray, t);

    (*hit_record).t = t;
    (*hit_record).pos = pos;
    (*hit_record).norm = normalize(pos - sphere.pos);

    return true;
}
// --- !Sphere ---
// --- Globals ---
struct Globals {
    seed: u32,
    samples: i32,
    depth: i32,
}
// --- !Globals ---

@group(0) @binding(0)
var tex: texture_storage_2d<rgba32float,read_write>;

@group(0) @binding(1)
var<uniform> camera: Camera;

@group(0) @binding(2)
var<storage> spheres: array<Sphere>;

@group(0) @binding(3)
var<uniform> globals: Globals;

fn closet_hit(ray: Ray, t_min: f32, t_max: f32, hit_record: ptr<function, HitRecord>) -> bool {
    let len = i32(arrayLength(&spheres));
    var closet_hit: HitRecord; // hit record of the closet object
    closet_hit.t = t_max; // set closet distance to max distance
    var has_hit = false;

    for(var i: i32 = 0; i < len; i++) {
        var hit_record: HitRecord;
        if (ray_sphere_intersect(spheres[i], ray, t_min, closet_hit.t, &hit_record)) {
            closet_hit = hit_record;
            closet_hit.sphere_index = i;
            has_hit = true;
        }
    }

    if has_hit {
        *hit_record = closet_hit;
        return true;
    }
    return false;
}
fn miss(dir_y: f32) -> vec3<f32> {
    // return vec3<f32>(0.0, 0.0, 0.0);
    // return vec3<f32>(1.0, 1.0, 1.0);

    let t = (dir_y + 1.0) / 2.0;
    return (1.0 - t) * vec3<f32>(1.0, 1.0, 1.0) + t*vec3<f32>(0.5, 0.7, 1.0);
}
fn trace_path(ray: Ray, seed: ptr<function, u32>) -> vec3<f32> {
    var ray = ray;

    var colour = vec3<f32>(1.0, 1.0, 1.0);
    var light = vec3<f32>(0.0, 0.0, 0.0);

    let t_min = 0.001;
    let t_max = 1.0 / 0.0;

    var i = 0;
    for (; i < globals.depth; i++) {
        var hit_record: HitRecord;
        if closet_hit(ray, t_min, t_max, &hit_record) {
            let sphere = spheres[hit_record.sphere_index];
            let rand_vec = vec3<f32>(
                randf_range(seed, -0.5, 0.5),
                randf_range(seed, -0.5, 0.5),
                randf_range(seed, -0.5, 0.5)
            );
            ray = ray_new(hit_record.pos, normalize(reflect(ray.dir, hit_record.norm + rand_vec * sphere.roughness)));
            colour *= sphere.albedo;
        }
        else {
            light = miss(ray.dir.y);
            break;
        }
    }
    return colour * light;
}

struct In {
    @builtin(workgroup_id) wg_id: vec3<u32>,
    @builtin(local_invocation_id) i_id: vec3<u32>
}

@compute
@workgroup_size(16,16)
fn main( in: In ) {
    // output texture dimensions
    let texture_dimensions = vec2<f32>(textureDimensions(tex));
    // pixel coord for invocation
    let pixel_coords = vec2<f32>(in.wg_id.xy) * 16.0 + vec2<f32>(in.i_id.xy);

    // don't do anything for the few pixels that might be outside the texture
    if pixel_coords.x > texture_dimensions.x {
        return;
    }
    if pixel_coords.y > texture_dimensions.y {
        return;
    }

    // uv [0.0, 1.0]
    let uv = (pixel_coords + vec2<f32>(0.5, 0.5)) / texture_dimensions;

    // normalized device coordinates [-1.0, 1.0]
    var ndc: vec2<f32>;
    ndc.x = uv.x * 2.0 - 1.0;
    ndc.y = -(uv.y * 2.0 - 1.0);

    // seed is combined from the cpu plus invocation pixel coord
    var local_seed = globals.seed + u32(vec2rand(ndc) * f32(0xffffffffu));

    // final accumulated colour
    var final_colour = vec3<f32>(0.0, 0.0, 0.0);

    for (var i: i32 = 0; i < globals.samples; i++) {
        // uv and ndc with random offset within the pixel
        let uv = (pixel_coords + vec2<f32>(randf(&local_seed), randf(&local_seed))) / texture_dimensions;
        var ndc: vec2<f32>;
        ndc.x = uv.x * 2.0 - 1.0;
        ndc.y = -(uv.y * 2.0 - 1.0);

        // get ray
        let ray = camera_get_ray(camera, ndc);

        final_colour += trace_path(ray, &local_seed);
    } 

    let pc_i32 = vec2<i32>(pixel_coords);
    var texture_colour = textureLoad(tex, pc_i32);
    texture_colour += vec4<f32>(final_colour / f32(globals.samples), 0.0);
    textureStore(tex, pc_i32, texture_colour);


    // store the average colour into the output texture
    // textureStore(tex, vec2<i32>(pixel_coords), vec4<f32>(final_colour / f32(globals.samples), 1.0));
    
    // textureStore(tex, vec2<i32>(pixel_coords), vec4(0.0, uv.x, uv.y, 1.0)); // green and blue
    // textureStore(tex, vec2<i32>(pixel_coords), vec4<f32>(uv.x, uv.y, 0.0, 1.0)); // red and green
}