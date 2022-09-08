struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: Camera;

// Lights
struct DirectionalLight {
    color_strength: vec4<f32>,
    direction: vec3<f32>,
};
struct PointLight {
    color: vec3<f32>,
    attenuation: vec3<f32>,
    position: vec3<f32>,
};
struct SpotLight {
    base: PointLight,
    direction_ccos: vec4<f32>,
};
struct LightBuffer {
    ambients: array<vec4<f32>, 1>,
    dirs: array<DirectionalLight, 10>,
    points: array<PointLight, 256>,
    spots: array<SpotLight, 256>,
    lens: vec4<u32>,
}
@group(2) @binding(0)
var<uniform> lights: LightBuffer;

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
    @location(1) tangent_position: vec3<f32>,
    @location(2) tangent_view_position: vec3<f32>,
    @location(3) world_position: vec4<f32>,
    @location(4) world_tangent: vec3<f32>,
    @location(5) world_bitangent: vec3<f32>,
    @location(6) world_normal: vec3<f32>,
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
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    out.world_position = world_position;
    out.world_tangent = world_tangent;
    out.world_bitangent = world_bitangent;
    out.world_normal = world_normal;
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


fn calculate_directional_light_color(light: DirectionalLight, object_normal: vec4<f32>, input: VertexOutput, tangent_light_position: vec3<f32>) -> vec3<f32> {
    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let light_dir = normalize(tangent_light_position - input.tangent_position);
    let view_dir = normalize(input.tangent_view_position - input.tangent_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0) * light.color_strength.w;
    let diffuse_color = light.color_strength.xyz * diffuse_strength;

    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0) * light.color_strength.w;
    let specular_color = light.color_strength.xyz * specular_strength;

    return diffuse_color + specular_color;
}

fn calculate_point_light_color(light: PointLight, object_normal: vec4<f32>, input: VertexOutput, tangent_light_position: vec3<f32>) -> vec3<f32> {
    var base: DirectionalLight;
    base.color_strength = vec4<f32>(light.color, 1.0);
    base.direction = vec3<f32>(0.0, 0.0, 0.0);

    let result = calculate_directional_light_color(base, object_normal, input, tangent_light_position);

    let distance = length(tangent_light_position - input.tangent_position);
    let atteniuation = light.attenuation.x + light.attenuation.y * distance + light.attenuation.z * distance * distance;

    return result / atteniuation;
}

fn calculate_spot_light_color(light: SpotLight, object_normal: vec4<f32>, input: VertexOutput, tangent_light_position: vec3<f32>, tangent_light_direction: vec3<f32>) -> vec3<f32> {
    let result = calculate_point_light_color(light.base, object_normal, input, tangent_light_position);

    let light_to_pixel = normalize(input.tangent_position - tangent_light_position);
    let spot_factor = dot(light_to_pixel, tangent_light_direction);

    if (spot_factor > light.direction_ccos.w) {
        let spot_light_intensity = 1.0 - (1.0 - spot_factor) / (1.0 - light.direction_ccos.w);
        return result * spot_light_intensity;
    } else {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, input.tex_coord);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, input.tex_coord);
    let tangent_matrix = transpose(mat3x3<f32>(
        input.world_tangent,
        input.world_bitangent,
        input.world_normal
    ));

    var result = vec3<f32>(0.0, 0.0, 0.0);
    for(var i = 0u; i < lights.lens[0]; i++) {
        result += lights.ambients[i].xyz * lights.ambients[i].w;
    }
    for(var i = 0u; i < lights.lens[1]; i++) {
        result += calculate_directional_light_color(lights.dirs[i], object_normal, input, tangent_matrix * (input.world_position.xyz - normalize(lights.dirs[i].direction)));
    }
    for(var i = 0u; i < lights.lens[2]; i++) {
        result += calculate_point_light_color(lights.points[i], object_normal, input, tangent_matrix * lights.points[i].position);
    }
    for(var i = 0u; i < lights.lens[3]; i++) {
        result += calculate_spot_light_color(lights.spots[i], object_normal, input, tangent_matrix * lights.spots[i].base.position, normalize(tangent_matrix * lights.spots[i].direction_ccos.xyz));
    }
    result *= object_color.xyz;

    return vec4<f32>(result, object_color.a);
}