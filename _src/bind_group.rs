use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{BindGroup, BindGroupEntries, BindGroupLayout, UniformBuffer},
        renderer::{RenderDevice, RenderQueue},
        storage::GpuShaderStorageBuffer,
    },
};

use crate::buffers::SurfaceNetsBuffers;

#[derive(Component)]
pub struct SurfaceNetsBindGroups {
    pub generate_vertices: BindGroup,
    pub prefix_sum_vertices: BindGroup,
    pub compact_vertices: BindGroup,
    pub generate_faces: BindGroup,
    pub prefix_sum_faces: BindGroup,
    pub compact_faces: BindGroup,
}

// Store bind group layouts as a resource
#[derive(Resource)]
pub struct SurfaceNetsBindGroupLayouts {
    pub generate_vertices: BindGroupLayout,
    pub prefix_sum: BindGroupLayout,
    pub compact_vertices: BindGroupLayout,
    pub generate_faces: BindGroupLayout,
    pub compact_faces: BindGroupLayout,
}

pub fn prepare_bind_groups(
    mut commands: Commands,
    layouts: Res<SurfaceNetsBindGroupLayouts>,
    entities_needing_bind_groups: Query<
        (Entity, &SurfaceNetsBuffers),
        Without<SurfaceNetsBindGroups>,
    >,
    gpu_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    for (entity, buffers) in &entities_needing_bind_groups {
        // Get GPU buffers - skip if any are not ready
        let Some(density_field) = gpu_buffers.get(&buffers.density_field) else {
            continue;
        };
        let Some(vertices) = gpu_buffers.get(&buffers.vertices) else {
            continue;
        };
        let Some(vertex_valid) = gpu_buffers.get(&buffers.vertex_valid) else {
            continue;
        };
        let Some(vertex_indices) = gpu_buffers.get(&buffers.vertex_indices) else {
            continue;
        };
        let Some(vertex_count) = gpu_buffers.get(&buffers.vertex_count) else {
            continue;
        };
        let Some(compacted_vertices) = gpu_buffers.get(&buffers.compacted_vertices) else {
            continue;
        };
        let Some(faces) = gpu_buffers.get(&buffers.faces) else {
            continue;
        };
        let Some(face_valid) = gpu_buffers.get(&buffers.face_valid) else {
            continue;
        };
        let Some(face_indices) = gpu_buffers.get(&buffers.face_indices) else {
            continue;
        };
        let Some(face_count) = gpu_buffers.get(&buffers.face_count) else {
            continue;
        };
        let Some(compacted_faces) = gpu_buffers.get(&buffers.compacted_faces) else {
            continue;
        };

        // Create uniform buffer for dimensions
        let mut dimensions_uniform = UniformBuffer::from(buffers.dimensions.0);
        dimensions_uniform.write_buffer(&render_device, &render_queue);

        // Bind Group 1: Generate Vertices
        let generate_vertices_bg = render_device.create_bind_group(
            Some("generate_vertices_bind_group"),
            &layouts.generate_vertices,
            &BindGroupEntries::sequential((
                density_field.buffer.as_entire_buffer_binding(),
                vertices.buffer.as_entire_buffer_binding(),
                vertex_valid.buffer.as_entire_buffer_binding(),
                dimensions_uniform.binding().unwrap(),
            )),
        );

        // Bind Group 2: Prefix Sum (vertices)
        let prefix_sum_vertices_bg = render_device.create_bind_group(
            Some("prefix_sum_vertices_bind_group"),
            &layouts.prefix_sum,
            &BindGroupEntries::sequential((
                vertex_valid.buffer.as_entire_buffer_binding(),
                vertex_indices.buffer.as_entire_buffer_binding(),
                vertex_count.buffer.as_entire_buffer_binding(),
            )),
        );

        // Bind Group 3: Compact Vertices
        let compact_vertices_bg = render_device.create_bind_group(
            Some("compact_vertices_bind_group"),
            &layouts.compact_vertices,
            &BindGroupEntries::sequential((
                vertices.buffer.as_entire_buffer_binding(),
                vertex_valid.buffer.as_entire_buffer_binding(),
                vertex_indices.buffer.as_entire_buffer_binding(),
                compacted_vertices.buffer.as_entire_buffer_binding(),
            )),
        );

        // Bind Group 4: Generate Faces
        let generate_faces_bg = render_device.create_bind_group(
            Some("generate_faces_bind_group"),
            &layouts.generate_faces,
            &BindGroupEntries::sequential((
                vertex_valid.buffer.as_entire_buffer_binding(),
                vertex_indices.buffer.as_entire_buffer_binding(),
                faces.buffer.as_entire_buffer_binding(),
                face_valid.buffer.as_entire_buffer_binding(),
                dimensions_uniform.binding().unwrap(),
            )),
        );

        // Bind Group 5: Prefix Sum (faces)
        let prefix_sum_faces_bg = render_device.create_bind_group(
            Some("prefix_sum_faces_bind_group"),
            &layouts.prefix_sum,
            &BindGroupEntries::sequential((
                face_valid.buffer.as_entire_buffer_binding(),
                face_indices.buffer.as_entire_buffer_binding(),
                face_count.buffer.as_entire_buffer_binding(),
            )),
        );

        // Bind Group 6: Compact Faces
        let compact_faces_bg = render_device.create_bind_group(
            Some("compact_faces_bind_group"),
            &layouts.compact_faces,
            &BindGroupEntries::sequential((
                faces.buffer.as_entire_buffer_binding(),
                face_valid.buffer.as_entire_buffer_binding(),
                face_indices.buffer.as_entire_buffer_binding(),
                compacted_faces.buffer.as_entire_buffer_binding(),
            )),
        );

        // Add bind groups component to this entity
        commands.entity(entity).insert(SurfaceNetsBindGroups {
            generate_vertices: generate_vertices_bg,
            prefix_sum_vertices: prefix_sum_vertices_bg,
            compact_vertices: compact_vertices_bg,
            generate_faces: generate_faces_bg,
            prefix_sum_faces: prefix_sum_faces_bg,
            compact_faces: compact_faces_bg,
        });

        info!("BindGroup prepared for Entity:{entity}");
    }
}
