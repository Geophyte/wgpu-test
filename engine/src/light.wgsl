struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: Camera;

// Lights
struct DirectionalLight {
    color_strength: vec4<f32>,
    direction: vec3<f32>
};
struct PointLight {
    color: vec3<f32>,
    attenuation: vec3<f32>,
    position: vec3<f32>
};
struct SpotLight {
    base: PointLight,
    direction_ccos: vec4<f32>
}
@group(1) @binding(0)
var<uniform> ambient_light: vec4<f32>;
@group(1) @binding(1)
var<uniform> directional_light: DirectionalLight;
@group(1) @binding(2)
var<uniform> point_light: PointLight;
@group(1) @binding(3)
var<uniform> spot_light: SpotLight;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let scale = 0.25;
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position * scale + spot_light.base.position.xyz, 1.0);
    out.color = spot_light.base.color;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}