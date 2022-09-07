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
struct PointLight {
    color: vec3<f32>,
    attenuation: vec3<f32>,
    position: vec3<f32>
};
struct SpotLight {
    base: PointLight,
    direction_ccos: vec4<f32>
}
@group(2) @binding(0)
var<uniform> ambient_light: vec4<f32>;
@group(2) @binding(1)
var<uniform> directional_light: DirectionalLight;
@group(2) @binding(2)
var<uniform> point_light: PointLight;
@group(2) @binding(3)
var<uniform> spot_light: SpotLight;

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

fn calculate_directional_light_color(light: DirectionalLight, input: VertexOutput) -> vec3<f32> {
    let light_dir = normalize(light.direction);
    let view_dir = normalize(camera.view_pos.xyz - input.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(input.world_normal, light_dir), 0.0) * light.color_strength.w;
    let diffuse_color = light.color_strength.xyz * diffuse_strength;

    let specular_strength = pow(max(dot(input.world_normal, half_dir), 0.0), 32.0) * light.color_strength.w;
    let specular_color = specular_strength * light.color_strength.xyz;

    return diffuse_color + specular_color;
}

fn calculate_point_light_color(light: PointLight, input: VertexOutput) -> vec3<f32> {
    var base: DirectionalLight;
    base.direction = light.position - input.world_position;
    base.color_strength = vec4<f32>(light.color, 1.0);
    let color = calculate_directional_light_color(base, input);

    let distance = length(light.position - input.world_position);
    let atteniuation = light.attenuation.x + light.attenuation.y * distance + light.attenuation.z * distance * distance;

    return color / atteniuation;
}

fn calculate_spot_light_color(light: SpotLight, input: VertexOutput) -> vec3<f32> {
    let color = calculate_point_light_color(light.base, input);

    let light_to_pixel = normalize(input.world_position - light.base.position);
    let spot_factor = dot(light_to_pixel, light.direction_ccos.xyz);

    var result = vec3<f32>(0.0, 0.0, 0.0);
    if (spot_factor > light.direction_ccos.w) {
        let spot_light_intensity = 1.0 - (1.0 - spot_factor) / (1.0 - light.direction_ccos.w);
        result = color * spot_light_intensity;
    }

    return result;
}

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

    let result = (ambient_color + calculate_directional_light_color(directional_light, input) + calculate_point_light_color(point_light, input) + calculate_spot_light_color(spot_light, input)) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}