use crate::model::ModelVertex;
use cgmath::{Rotation, Rotation3};
use itertools::Itertools;

#[derive(Clone)]
pub struct Geometry {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}

impl Geometry {
    pub fn plane(width: f32, height: f32, rows: u32, columns: u32) -> Self {
        let quad_width = width / columns as f32;
        let quad_height = height / rows as f32;

        let start_x = -width / 2.0;
        let start_z = -height / 2.0;
        let mut vertices = (0..=rows)
            .flat_map(|r| {
                (0..=columns).map(move |c| {
                    let position = [
                        start_x + c as f32 * quad_width,
                        0.0,
                        start_z + r as f32 * quad_height,
                    ];
                    let tex_coords = [
                        (-position[0] / start_x + 1.0) / 2.0,
                        (-position[2] / start_z + 1.0) / 2.0,
                    ];
                    let normal = [0.0, 1.0, 0.0];
                    let tangent = [0.0; 3];
                    let bitangent = [0.0; 3];

                    ModelVertex {
                        position,
                        tex_coords,
                        normal,
                        tangent,
                        bitangent,
                    }
                })
            })
            .collect_vec();
        let indices = (0..rows)
            .flat_map(|r| {
                let row_len = columns + 1;
                (0..columns).flat_map(move |c| {
                    [
                        r * row_len + c,
                        (r + 1) * row_len + c,
                        (r + 1) * row_len + c + 1,
                        r * row_len + c,
                        (r + 1) * row_len + c + 1,
                        r * row_len + c + 1,
                    ]
                })
            })
            .collect_vec();

        calculate_tangents_bitangents(&mut vertices, &indices);

        Self { vertices, indices }
    }

    pub fn cube(width: f32, height: f32, length: f32) -> Self {
        let w = width / 2.0;
        let h = height / 2.0;
        let l = length / 2.0;

        // create sides
        let mut top = Geometry::plane(width, length, 1, 1);
        let mut bottom = top.clone();
        let mut front = Geometry::plane(width, height, 1, 1);
        let mut back = front.clone();
        let mut right = Geometry::plane(height, length, 1, 1);
        let mut left = right.clone();

        // rotate and transpose sides
        top.transpose(cgmath::Vector3::new(0.0, h, 0.0));
        bottom.rotate(cgmath::Quaternion::from_angle_x(cgmath::Deg(180.0)));
        bottom.transpose(cgmath::Vector3::new(0.0, -h, 0.0));
        front.rotate(cgmath::Quaternion::from_angle_x(cgmath::Deg(-90.0)));
        front.transpose(cgmath::Vector3::new(0.0, 0.0, -l));
        back.rotate(cgmath::Quaternion::from_angle_x(cgmath::Deg(90.0)));
        back.transpose(cgmath::Vector3::new(0.0, 0.0, l));
        right.rotate(cgmath::Quaternion::from_angle_z(cgmath::Deg(-90.0)));
        right.transpose(cgmath::Vector3::new(w, 0.0, 0.0));
        left.rotate(cgmath::Quaternion::from_angle_z(cgmath::Deg(90.0)));
        left.transpose(cgmath::Vector3::new(-w, 0.0, 0.0));

        // merge vertices and indices
        let mut vertices = top.vertices;
        vertices.append(&mut bottom.vertices);
        vertices.append(&mut front.vertices);
        vertices.append(&mut back.vertices);
        vertices.append(&mut right.vertices);
        vertices.append(&mut left.vertices);

        let mut indices = top.indices;
        indices.append(&mut bottom.indices.iter().map(|i| i + 4).collect_vec());
        indices.append(&mut front.indices.iter().map(|i| i + 8).collect_vec());
        indices.append(&mut back.indices.iter().map(|i| i + 12).collect_vec());
        indices.append(&mut right.indices.iter().map(|i| i + 16).collect_vec());
        indices.append(&mut left.indices.iter().map(|i| i + 20).collect_vec());

        Self { vertices, indices }
    }

    pub fn rotate(&mut self, quaternion: cgmath::Quaternion<f32>) {
        for v in self.vertices.iter_mut() {
            let mut point = cgmath::Point3::<f32> {
                x: v.position[0],
                y: v.position[1],
                z: v.position[2],
            };
            point = quaternion.rotate_point(point);
            v.position = [point.x, point.y, point.z];
            let mut normal = cgmath::Vector3::new(v.normal[0], v.normal[1], v.normal[2]);
            normal = quaternion.rotate_vector(normal);
            v.normal = [normal.x, normal.y, normal.z];
            let mut tangent = cgmath::Vector3::new(v.tangent[0], v.tangent[1], v.tangent[2]);
            tangent = quaternion.rotate_vector(tangent);
            v.tangent = [tangent.x, tangent.y, tangent.z];
            let mut bitangent =
                cgmath::Vector3::new(v.bitangent[0], v.bitangent[1], v.bitangent[2]);
            bitangent = quaternion.rotate_vector(bitangent);
            v.bitangent = [bitangent.x, bitangent.y, bitangent.z];
        }
    }

    pub fn transpose(&mut self, vector: cgmath::Vector3<f32>) {
        for v in self.vertices.iter_mut() {
            let mut point = cgmath::Point3::<f32> {
                x: v.position[0],
                y: v.position[1],
                z: v.position[2],
            };
            point += vector;
            v.position = [point.x, point.y, point.z];
        }
    }
}

pub fn calculate_tangents_bitangents(vertices: &mut Vec<ModelVertex>, indices: &Vec<u32>) {
    let mut triangles_included = vec![0; vertices.len()];

    for c in indices.chunks(3) {
        let v0 = vertices[c[0] as usize];
        let v1 = vertices[c[1] as usize];
        let v2 = vertices[c[2] as usize];

        let pos0: cgmath::Vector3<_> = v0.position.into();
        let pos1: cgmath::Vector3<_> = v1.position.into();
        let pos2: cgmath::Vector3<_> = v2.position.into();

        let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
        let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
        let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

        // Calculate the edges of the triangle
        let delta_pos1 = pos1 - pos0;
        let delta_pos2 = pos2 - pos0;

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        // Solving the following system of equations will
        // give us the tangent and bitangent.
        //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
        //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;

        // Flip the bitangent to enable right-handed normal
        // maps with wgpu texture coordinate system
        let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

        // Use the same tangent/bitangent for each vertex in the triangle
        vertices[c[0] as usize].tangent =
            (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
        vertices[c[1] as usize].tangent =
            (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
        vertices[c[2] as usize].tangent =
            (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();

        vertices[c[0] as usize].bitangent =
            (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
        vertices[c[1] as usize].bitangent =
            (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
        vertices[c[2] as usize].bitangent =
            (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

        // Used to average the tangents/bitangents
        triangles_included[c[0] as usize] += 1;
        triangles_included[c[1] as usize] += 1;
        triangles_included[c[2] as usize] += 1;
    }
    // Average the tangents/bitangents
    for (i, n) in triangles_included.into_iter().enumerate() {
        let denom = 1.0 / n as f32;
        let mut v = &mut vertices[i];
        v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
        v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
    }
}
