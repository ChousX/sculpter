use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::storage::ShaderStorageBuffer;

use crate::{DensityField, DensityFieldSize};

// Component that holds GPU buffers during generation (one per generating entity)
#[derive(Component)]
pub struct SurfaceNetsBuffers {
    // Stage 0: Inputs
    pub density_field: Handle<ShaderStorageBuffer>,
    //Dimensions of the Input
    pub dimensions: DensityFieldSize,
    //pub dimensions: Handle<ShaderStorageBuffer>,

    // Stage 1: Generate Vertices
    pub vertices: Handle<ShaderStorageBuffer>,
    pub vertex_valid: Handle<ShaderStorageBuffer>,

    // Stage 2: Prefix Sum (vertices)
    pub vertex_indices: Handle<ShaderStorageBuffer>,
    pub vertex_count: Handle<ShaderStorageBuffer>,
    pub compacted_vertices: Handle<ShaderStorageBuffer>,

    // Stage 3: Generate Faces
    pub faces: Handle<ShaderStorageBuffer>,
    pub face_valid: Handle<ShaderStorageBuffer>,

    // Stage 4: Prefix Sum (faces)
    pub face_indices: Handle<ShaderStorageBuffer>,
    pub face_count: Handle<ShaderStorageBuffer>,
    pub compacted_faces: Handle<ShaderStorageBuffer>,
}

impl SurfaceNetsBuffers {
    pub fn new(
        density_field: &DensityField,
        dimensions: &DensityFieldSize,
        buffers: &mut ResMut<Assets<ShaderStorageBuffer>>,
    ) -> Self {
        let cell_count = dimensions.cell_count();
        let max_faces = cell_count * 3;

        // Create density field buffer
        let mut density_buffer = ShaderStorageBuffer::from(density_field.0.clone());
        density_buffer.buffer_description.usage |= BufferUsages::STORAGE;

        // Stage 1 buffers: Generate Vertices
        let vertices_buffer = ShaderStorageBuffer::from(vec![0.0f32; (cell_count * 3) as usize]);
        let vertex_valid_buffer = ShaderStorageBuffer::from(vec![0u32; cell_count as usize]);

        // Stage 2 buffers: Prefix Sum (vertices)
        let vertex_indices_buffer = ShaderStorageBuffer::from(vec![0u32; cell_count as usize]);
        let vertex_count_buffer = ShaderStorageBuffer::from(vec![0u32; 1]);

        // Stage 3 buffers: Compact Vertices
        let compacted_vertices_buffer =
            ShaderStorageBuffer::from(vec![0.0f32; (cell_count * 3) as usize]);

        // Stage 4 buffers: Generate Faces
        let faces_buffer = ShaderStorageBuffer::from(vec![0u32; (max_faces * 4) as usize]);
        let face_valid_buffer = ShaderStorageBuffer::from(vec![0u32; max_faces as usize]);

        // Stage 5 buffers: Prefix Sum (faces)
        let face_indices_buffer = ShaderStorageBuffer::from(vec![0u32; max_faces as usize]);
        let face_count_buffer = ShaderStorageBuffer::from(vec![0u32; 1]);

        // Stage 6 buffers: Compact Faces
        let compacted_faces_buffer =
            ShaderStorageBuffer::from(vec![0u32; (max_faces * 4) as usize]);

        SurfaceNetsBuffers {
            density_field: buffers.add(density_buffer),
            vertices: buffers.add(vertices_buffer),
            vertex_valid: buffers.add(vertex_valid_buffer),
            vertex_indices: buffers.add(vertex_indices_buffer),
            vertex_count: buffers.add(vertex_count_buffer),
            compacted_vertices: buffers.add(compacted_vertices_buffer),
            faces: buffers.add(faces_buffer),
            face_valid: buffers.add(face_valid_buffer),
            face_indices: buffers.add(face_indices_buffer),
            face_count: buffers.add(face_count_buffer),
            compacted_faces: buffers.add(compacted_faces_buffer),
            dimensions: *dimensions,
        }
    }
}
