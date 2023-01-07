struct VertexIn {
    @location(0) pos: vec2<f32>
}
struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(model: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(model.pos, 0.5, 1.0);
    out.uv.x = (model.pos.x + 1.0) / 2.0;
    out.uv.y = (-model.pos.y + 1.0) / 2.0;
    return out;
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var sam: sampler;
@group(0) @binding(2)
var<uniform> samples: i32;

// srgb -> linear
fn correct(colour: vec3<f32>) -> vec3<f32> {
    let exp = 2.2;

    return vec3<f32>(
        pow(colour.x, exp),
        pow(colour.y, exp),
        pow(colour.z, exp),
    );
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // return textureSample(tex, sam, in.uv);
    let colour = textureSample(tex, sam, in.uv);
    let rgb = colour.xyz / f32(samples);
    return vec4<f32>(correct(rgb), colour.w);
}