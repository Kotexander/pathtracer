@group(0) @binding(0)
var tex: texture_storage_2d<rgba32float,write>;

struct In {
    @builtin(workgroup_id) wg_id: vec3<u32>,
    @builtin(local_invocation_id) i_id: vec3<u32>
}

@compute
@workgroup_size(16,16)
fn main( in: In ) {
    let texture_dimensions = vec2<f32>(textureDimensions(tex));
    let pixel_coords = vec2<f32>(in.wg_id.xy) * 16.0 + vec2<f32>(in.i_id.xy);


    let uv = pixel_coords / texture_dimensions;

    // var ndc: vec2<f32>;
    // ndc.x = uv.x * 2.0 - 1.0;
    // ndc.y = -(uv.y * 2.0 - 1.0);
    
    textureStore(tex, vec2<i32>(pixel_coords), vec4(0.0, uv.x, uv.y, 1.0)); // green and blue
    // textureStore(tex, vec2<i32>(pixel_coords), vec4<f32>(uv.x, uv.y, 0.0, 1.0)); // red and green
}