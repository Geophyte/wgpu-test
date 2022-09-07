struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: Camera;

// Lights
struct DirectionalLight {
    color_strength: vec4<f32>,
    direction: vec3<f32>
};
@group(2) @binding(0)
var<uniform> ambient_light: vec4<f32>;
@group(2) @binding(1)
var<uniform> directional_light: DirectionalLight;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>
    //@location(1) tangent_position: vec3<f32>,
    //@location(2) tangent_view_position: vec3<f32>,
    //@location(3) tangent_light_position: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2
    );
    let world_normal = (model_matrix * vec4<f32>(model.normal, 0.0)).xyz;
    let world_tangent = normalize(normal_matrix * model.tangent);
    let world_bitangent = normalize(normal_matrix * model.bitangent);
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal
    ));

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coord = model.tex_coord;
    out.world_normal = (model_matrix * vec4<f32>(model.normal, 0.0)).xyz;
    out.world_position = (model_matrix * vec4<f32>(model.position, 1.0)).xyz;
    //out.tangent_position = tangent_matrix * world_position.xyz;
    //out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    //out.tangent_light_position = normalize(tangent_matrix * (model.position + directional_light.direction));
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0)@binding(3)
var s_normal: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    //let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, input.tex_coord);
    //let object_normal: vec4<f32> = textureSample(t_normal, s_normal, input.tex_coord);

    //let ambient_strength = 0.1;
    //let ambient_color = light.color * ambient_strength;

    //let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    //let light_dir = normalize(input.tangent_light_position - input.tangent_position);
    //let view_dir = normalize(input.tangent_view_position - input.tangent_position);
    //let half_dir = normalize(view_dir + light_dir);

    //let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
    //let diffuse_color = light.color * diffuse_strength;

    //let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
    //let specular_color = light.color * specular_strength;

    //let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, input.tex_coord);

    let ambient_strength = ambient_light.w;
    let ambient_color = ambient_light.xyz * ambient_strength;

    let light_dir = normalize(directional_light.direction);
    let view_dir = normalize(camera.view_pos.xyz - input.world_position);
    let reflect_dir = reflect(-light_dir, input.world_normal);

    let diffuse_strength = max(dot(input.world_normal, light_dir), 0.0) * directional_light.color_strength.w;
    let diffuse_color = directional_light.color_strength.xyz * diffuse_strength;

    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0) * directional_light.color_strength.w;
    let specular_color = specular_strength * directional_light.color_strength.xyz;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}