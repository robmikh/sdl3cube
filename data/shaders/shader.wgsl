struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct Globals {
    transform: mat4x4<f32>,
};
@group(1)
@binding(0)
var<uniform> r_globals: Globals;

struct Locals {
    transform: mat4x4<f32>,
};
@group(1)
@binding(1)
var<uniform> r_locals: Locals;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    var in_position: vec4<f32>;
    in_position.x = position.x;
    in_position.y = position.y;
    in_position.z = position.z;
    in_position.w = 1.0;

    var out: VertexOutput;
    out.color = color;
    out.position = r_globals.transform * r_locals.transform * in_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
