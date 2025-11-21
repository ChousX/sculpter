use crate::{DensityFieldMeshSize, DensityFieldSize, readback::ReadbackBuffers};
use bevy::{asset::RenderAssetUsages, mesh::Indices, prelude::*};

pub fn build_mesh_from_readback(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_size: Res<DensityFieldMeshSize>,
    dimensions: Res<DensityFieldSize>,
    query: Query<(Entity, &ReadbackBuffers)>,
) {
    for (entity, data) in query.iter() {
        let Some(vertex_count) = data.vertex_count else {
            continue;
        };
        let Some(ref vertices) = data.vertices else {
            continue;
        };
        let Some(face_count) = data.face_count else {
            continue;
        };
        let Some(ref faces) = data.faces else {
            continue;
        };

        info!("Building Mesh from readback buffers");

        let scale = **mesh_size / dimensions.as_vec3();
        let mut world_positions = Vec::with_capacity(vertex_count as usize);
        for i in 0..vertex_count as usize {
            let base = i * 3;
            if base + 2 < vertices.len() {
                let grid_pos = Vec3::new(vertices[base], vertices[base + 1], vertices[base + 2]);
                let world_pos = grid_pos * scale; //+ offset
                world_positions.push([world_pos.x, world_pos.y, world_pos.z]);
            }
        }

        info!("Vertices: {world_positions:?}");

        let mut triangle_indices = Vec::with_capacity(face_count as usize * 6);
        for i in 0..face_count as usize {
            let base = i * 4;
            if base + 3 < faces.len() {
                let v0 = faces[base];
                let v1 = faces[base + 1];
                let v2 = faces[base + 2];
                let v3 = faces[base + 3];
                //triangle 1
                triangle_indices.push(v0);
                triangle_indices.push(v1);
                triangle_indices.push(v2);

                //triangle 2
                triangle_indices.push(v0);
                triangle_indices.push(v2);
                triangle_indices.push(v3);
            }
        }

        info!("TriangleIndices: {triangle_indices:?}");

        let normals = compute_flat_normals(&world_positions, &triangle_indices);

        info!("Normals: {normals:?}");

        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, world_positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_indices(Indices::U32(triangle_indices));

        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            metallic: 0.0,
            perceptual_roughness: 0.5,
            ..default()
        });

        commands
            .entity(entity)
            .insert((Mesh3d(mesh_handle), MeshMaterial3d(material_handle)))
            .remove::<ReadbackBuffers>();
    }
}
fn compute_flat_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0, 0.0, 0.0]; positions.len()];
    let mut normal_counts = vec![0u32; positions.len()];

    // For each triangle, compute its normal and add to vertices
    for triangle in indices.chunks_exact(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let v0 = Vec3::from(positions[i0]);
        let v1 = Vec3::from(positions[i1]);
        let v2 = Vec3::from(positions[i2]);

        // Compute face normal using cross product
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize_or_zero();

        // Add to each vertex of the triangle
        for &idx in &[i0, i1, i2] {
            normals[idx][0] += normal.x;
            normals[idx][1] += normal.y;
            normals[idx][2] += normal.z;
            normal_counts[idx] += 1;
        }
    }

    // Average the normals
    for i in 0..normals.len() {
        if normal_counts[i] > 0 {
            let count = normal_counts[i] as f32;
            let normal = Vec3::new(
                normals[i][0] / count,
                normals[i][1] / count,
                normals[i][2] / count,
            )
            .normalize_or_zero();

            normals[i] = [normal.x, normal.y, normal.z];
        }
    }

    normals
}
