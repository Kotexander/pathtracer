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
    inv_dir: vec3<f32>
}
fn ray_new(pos: vec3<f32>, dir: vec3<f32>) -> Ray {
    var ray: Ray;
    ray.pos = pos;
    ray.dir = dir;
    ray.inv_dir = 1.0 / dir;
    return ray;
}
fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.pos + (ray.dir * t);
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

    sphere_index: u32,
    back: bool,
}
// --- !Hit Record
// --- Bounding Box ---
struct BoundingBox {
    min: vec3<f32>,
    max: vec3<f32>,
}
fn ray_bbox_intersect(bbox: BoundingBox, ray: Ray) -> bool {
    let t0s = (bbox.min - ray.pos) / ray.dir;
    let t1s = (bbox.max - ray.pos) / ray.dir;

    let tsmaller = min(t0s, t1s);
    let tbigger = max(t0s, t1s);

    let tmin = max(0.0, max(tsmaller.x, max(tsmaller.y, tsmaller.z)));
    let tmax = min(tbigger.x, min(tbigger.y, tbigger.z));

    return tmin < tmax;
}
// --- !Bounding Box ---
// --- BVHNode ---
struct BVHNode {
    bbox: BoundingBox,

    node_type: u32,
    index: u32,
}
// --- !BVHNode ---


// --- Sphere ---
struct Sphere {
    pos: vec3<f32>,
    rad: f32,
    mat_type: u32,
    mat_index: u32
}
fn ray_sphere_intersect(sphere: Sphere, ray: Ray, t_min: f32, t_max: f32, hit_record: ptr<function, HitRecord>) -> bool {
    let dir = ray.pos - sphere.pos;

    let half_b = dot(dir, ray.dir);
    let c = dot(dir, dir) - (sphere.rad * sphere.rad);

    let d = half_b*half_b - c;
    if d < 0.0 {
        return false;
    }

    let d_sqrt = sqrt(d);
    var t = (-half_b - d_sqrt);
    var back = false;
    if t < t_min || t > t_max {
        t = (-half_b + d_sqrt);
        if t < t_min || t > t_max {
            return false;
        }
        back = true;
    }


    let pos = ray_at(ray, t);

    (*hit_record).t = t;
    (*hit_record).pos = pos;
    (*hit_record).norm = normalize(pos - sphere.pos);
    if back {
        (*hit_record).norm = -(*hit_record).norm; 
        // (*hit_record).norm *= -1.0; 
    }
    (*hit_record).back = back;

    return true;
}
// --- !Sphere ---
// --- Materials ---
struct Light {
    colour: vec3<f32>,
}
struct Lambertian {
    albedo: vec3<f32>,
}
struct Metal {
    albedo: vec3<f32>,
    roughness: f32,
}
struct Glass {
    ir: f32
}
// --- !Materials ---
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
var<uniform> globals: Globals;

@group(1) @binding(0)
var<storage> bvh: array<BVHNode>;
@group(1) @binding(1)
var<storage> spheres: array<Sphere>;
@group(1) @binding(2)
var<storage> lights: array<Light>;
@group(1) @binding(3)
var<storage> lambertians: array<Lambertian>;
@group(1) @binding(4)
var<storage> metals: array<Metal>;
@group(1) @binding(5)
var<storage> glass: array<Glass>;

fn refract(i: vec3<f32>, n: vec3<f32>, etai_over_etat: f32) -> vec3<f32>{
    let cos_theta = min(dot(-i, n), 1.0);
    let r_out_perp =  etai_over_etat * (i + cos_theta*n);
    let r_out_parallel = -sqrt(abs(1.0 - dot(r_out_perp, r_out_perp))) * n;
    return r_out_perp + r_out_parallel;
}
fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    var r0: f32 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0*r0;
    return r0 + (1.0-r0)*pow((1.0 - cosine), 5.0);
}
fn closet_hit(ray: Ray, t_min: f32, t_max: f32, hit_record: ptr<function, HitRecord>) -> bool {
    // --- No BVH ---
    // let len = arrayLength(&spheres);
    // var closet_hit: HitRecord; // hit record of the closet object
    // closet_hit.t = t_max; // set closet distance to max distance
    // var has_hit = false;

    // for(var i = 0u; i < len; i++) {
    //     var hit_record: HitRecord;
    //     if (ray_sphere_intersect(spheres[i], ray, t_min, closet_hit.t, &hit_record)) {
    //         closet_hit = hit_record;
    //         closet_hit.sphere_index = i;
    //         has_hit = true;
    //     }
    // }

    // if has_hit {
    //     *hit_record = closet_hit;
    //     return true;
    // }
    // return false;

    // --- BVH ---
    var closet_hit: HitRecord; // hit record of the closet object
    closet_hit.t = t_max; // set closet distance to max distance
    var has_hit = false;

    let len = arrayLength(&bvh);

    var i = 0u;
    while i < len {
        let node = bvh[i];
        i += 1u;
        switch node.node_type {
            // node
            case 0u {
                if !ray_bbox_intersect(node.bbox, ray) {
                    i = node.index;
                }
            }
            // obj
            case 1u {
                var hit_record: HitRecord;
                if (ray_sphere_intersect(spheres[node.index], ray, t_min, closet_hit.t, &hit_record)) {
                    closet_hit = hit_record;
                    closet_hit.sphere_index = node.index;
                    has_hit = true;
                }
            }
            default {
                return false;
            }
        }
    }
    
    if has_hit {
        *hit_record = closet_hit;
        return true;
    }
    return false;
}
fn miss(dir_y: f32) -> vec3<f32> {
    // return vec3<f32>(0.0, 0.0, 0.0); // black/night
    // return vec3<f32>(1.0, 1.0, 1.0); // white

    // day
    let t = (dir_y + 1.0) / 2.0;
    return (1.0 - t) * vec3<f32>(1.0, 1.0, 1.0) + t*vec3<f32>(0.5, 0.7, 1.0);
}
fn trace_path(ray: Ray, seed: ptr<function, u32>) -> vec3<f32> {
    var ray = ray;

    var colour = vec3<f32>(1.0, 1.0, 1.0);
    var light = vec3<f32>(0.0, 0.0, 0.0);

    let t_min = 0.000;
    let t_max = 1.0 / 0.0;
    let dist = 0.001;

    var not_hit_light = true;
    var i = 0;
    while ( i <= globals.depth && not_hit_light) {
        var hit_record: HitRecord;
        if closet_hit(ray, t_min, t_max, &hit_record) {
            let sphere = spheres[hit_record.sphere_index];
            let new_pos = hit_record.pos + hit_record.norm * dist;
            switch sphere.mat_type {
                // light
                case 0u: {
                    let material = lights[sphere.mat_index];
                    light = material.colour;
                    not_hit_light = false;
                }
                // lambertian
                case 1u: {
                    if hit_record.back {
                        return vec3<f32>(0.0);
                    }
                    let material = lambertians[sphere.mat_index];
                    let scattered = normalize(hit_record.norm + rand_in_sphere(seed));
                    ray = ray_new(new_pos, scattered);
                    colour *= material.albedo;
                }
                // metal 
                case 2u: {
                    if hit_record.back {
                        return vec3<f32>(0.0);
                    }
                    let material = metals[sphere.mat_index];
                    // let rand_vec = vec3<f32>(
                        // randf_range(seed, -0.5, 0.5),
                        // randf_range(seed, -0.5, 0.5),
                        // randf_range(seed, -0.5, 0.5)
                    // );
                    // ray = ray_new(hit_record.pos, (reflect(ray.dir, hit_record.norm + (rand_vec * material.roughness))));
                    let reflected = normalize(reflect(ray.dir, hit_record.norm) + rand_in_sphere(seed) * material.roughness);
                    ray = ray_new(new_pos, reflected);
                    colour *= material.albedo;
                }
                // glass
                case 3u {
                    let material = glass[sphere.mat_index];
                    var ir: f32;
                    if hit_record.back {
                        ir = material.ir;
                    }
                    else {
                        ir = 1.0 / material.ir;
                    }

                    let cos_theta = min(dot(-ray.dir, hit_record.norm), 1.0);
                    let sin_theta = sqrt(1.0 - cos_theta*cos_theta);
                    let cannot_refract = ir * sin_theta > 1.0;
                    var dir: vec3<f32>;

                    var new_pos: vec3<f32>;
                    if cannot_refract || reflectance(cos_theta, ir) > randf(seed) {
                        dir = reflect(ray.dir, hit_record.norm);
                        new_pos = hit_record.pos + hit_record.norm * dist;
                    }
                    else {
                        dir = refract(ray.dir, hit_record.norm, ir);
                        new_pos = hit_record.pos - hit_record.norm * dist;
                    }
                    dir = normalize(dir);

                    ray = ray_new(new_pos, dir);
                }
                default {
                    return vec3<f32>(0.0, 0.0, 0.0);
                }
            }
        }
        else {
            light = miss(ray.dir.y);
            not_hit_light = false;
        }
        i += 1;
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
}