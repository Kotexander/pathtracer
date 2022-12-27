struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.5, 1.0);
    out.tex_coords = (model.position + vec2<f32>(1.0, 1.0)) / 2.0;
    out.tex_coords.y -= 1.0;
    out.tex_coords.y *= -1.0; 
    return out;
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0)@binding(1)
var sam: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, sam, in.tex_coords);
}