use bevy::render::render_resource::*;
use bevy::render::storage::ShaderStorageBuffer;
use bevy::{prelude::*, render::extract_component::ExtractComponent};

use crate::{DensityField, DensityFieldSize};

// Component that holds GPU buffers during generation (one per generating entity)
#[derive(Component, Clone)]
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

impl ExtractComponent for SurfaceNetsBuffers {
    type QueryData = &'static SurfaceNetsBuffers;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(
        item: bevy::ecs::query::QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self> {
        Some(item.clone())
    }
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
        density_buffer.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_DST;

        // Stage 1 buffers: Generate Vertices
        let mut vertices_buffer =
            ShaderStorageBuffer::from(vec![0.0f32; (cell_count * 3) as usize]);
        vertices_buffer.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_SRC;

        let mut vertex_valid_buffer = ShaderStorageBuffer::from(vec![0u32; cell_count as usize]);
        vertex_valid_buffer.buffer_description.usage |=
            BufferUsages::STORAGE | BufferUsages::COPY_SRC;

        // Stage 2 buffers: Prefix Sum (vertices)
        let mut vertex_indices_buffer = ShaderStorageBuffer::from(vec![0u32; cell_count as usize]);
        vertex_indices_buffer.buffer_description.usage |=
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

        let mut vertex_count_buffer = ShaderStorageBuffer::from(vec![0u32; 1]);
        vertex_count_buffer.buffer_description.usage |=
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

        // Stage 3 buffers: Compact Vertices
        let mut compacted_vertices_buffer =
            ShaderStorageBuffer::from(vec![0.0f32; (cell_count * 3) as usize]);
        compacted_vertices_buffer.buffer_description.usage |=
            BufferUsages::STORAGE | BufferUsages::COPY_SRC;

        // Stage 4 buffers: Generate Faces
        let mut faces_buffer = ShaderStorageBuffer::from(vec![0u32; (max_faces * 4) as usize]);
        faces_buffer.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::COPY_SRC;

        let mut face_valid_buffer = ShaderStorageBuffer::from(vec![0u32; max_faces as usize]);
        face_valid_buffer.buffer_description.usage |=
            BufferUsages::STORAGE | BufferUsages::COPY_SRC;

        // Stage 5 buffers: Prefix Sum (faces)
        let mut face_indices_buffer = ShaderStorageBuffer::from(vec![0u32; max_faces as usize]);
        face_indices_buffer.buffer_description.usage =
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

        let mut face_count_buffer = ShaderStorageBuffer::from(vec![0u32; 1]);
        face_count_buffer.buffer_description.usage |=
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

        // Stage 6 buffers: Compact Faces
        let mut compacted_faces_buffer =
            ShaderStorageBuffer::from(vec![0u32; (max_faces * 4) as usize]);
        compacted_faces_buffer.buffer_description.usage |=
            BufferUsages::STORAGE | BufferUsages::COPY_SRC;

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

/// Prepare Buffers (per entResMut<Assets<ShaderStorageBuffer>>ity)
pub fn prepare_surface_nets_buffers(
    mut commands: Commands,
    // Query entities that have DensityField but no Mesh3d
    needs_mesh_query: Query<
        (Entity, &DensityField),
        (Without<SurfaceNetsBuffers>, Without<Mesh3d>),
    >,
    dimensions: Res<DensityFieldSize>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (entity, density_field) in needs_mesh_query.iter() {
        // Create GPU buffers to start generation
        let buffers = SurfaceNetsBuffers::new(density_field, &dimensions, &mut buffers);
        commands.entity(entity).insert(buffers);
    }
}
