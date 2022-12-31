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

// srgb -> linear
fn correct(colour: vec4<f32>) -> vec4<f32> {
    let exp = 2.2;

    return vec4<f32>(
        pow(colour.x, exp),
        pow(colour.y, exp),
        pow(colour.z, exp),
        colour.w
    );
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // return textureSample(tex, sam, in.uv);
    return correct(textureSample(tex, sam, in.uv));
}