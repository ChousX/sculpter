// pipeline.rs
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;

use crate::bind_group::SurfaceNetsBindGroupLayouts;

// Shader paths
const GENERATE_VERTICES_SHADER: &str = "shaders/surface_nets/generate_vertices.wgsl";
const PREFIX_SUM_SHADER: &str = "shaders/surface_nets/prefix_sum.wgsl";
const COMPACT_VERTICES_SHADER: &str = "shaders/surface_nets/compact_vertices.wgsl";
const GENERATE_FACES_SHADER: &str = "shaders/surface_nets/generate_faces.wgsl";
const COMPACT_FACES_SHADER: &str = "shaders/surface_nets/compact_faces.wgsl";

#[derive(Resource)]
pub struct SurfaceNetsPipelines {
    pub generate_vertices_pipeline: CachedComputePipelineId,

    pub prefix_sum_pipeline: CachedComputePipelineId,

    pub compact_vertices_pipeline: CachedComputePipelineId,

    pub generate_faces_pipeline: CachedComputePipelineId,

    pub compact_faces_pipeline: CachedComputePipelineId,
}

pub fn init_surface_nets_pipelines(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
) {
    use binding_types::*;

    // Layout 1: Generate Vertices
    let generate_vertices_layout = render_device.create_bind_group_layout(
        "GenerateVerticesLayout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<f32>>(false), // density_field
                storage_buffer::<Vec<f32>>(false),           // vertices (output)
                storage_buffer::<Vec<u32>>(false),           // vertex_valid (output)
                uniform_buffer::<UVec3>(false),              // dimensions
            ),
        ),
    );

    // Layout 2: Prefix Sum
    let prefix_sum_layout = render_device.create_bind_group_layout(
        "PrefixSumLayout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<u32>>(false), // input (valid flags)
                storage_buffer::<Vec<u32>>(false),           // output (indices)
                storage_buffer::<u32>(false),                // count
            ),
        ),
    );

    // Layout 3: Compact Vertices
    let compact_vertices_layout = render_device.create_bind_group_layout(
        "CompactVerticesLayout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<f32>>(false), // vertices (input)
                storage_buffer_read_only::<Vec<u32>>(false), // vertex_valid
                storage_buffer_read_only::<Vec<u32>>(false), // vertex_indices
                storage_buffer::<Vec<f32>>(false),           // compacted_vertices (output)
            ),
        ),
    );

    // Layout 4: Generate Faces
    let generate_faces_layout = render_device.create_bind_group_layout(
        "GenerateFacesLayout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<u32>>(false), // vertex_valid
                storage_buffer_read_only::<Vec<u32>>(false), // vertex_indices
                storage_buffer::<Vec<u32>>(false),           // faces (output)
                storage_buffer::<Vec<u32>>(false),           // face_valid (output)
                uniform_buffer::<UVec3>(false),              // dimensions
            ),
        ),
    );

    // Layout 5: Compact Faces
    let compact_faces_layout = render_device.create_bind_group_layout(
        "CompactFacesLayout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<u32>>(false), // faces (input)
                storage_buffer_read_only::<Vec<u32>>(false), // face_valid
                storage_buffer_read_only::<Vec<u32>>(false), // face_indices
                storage_buffer::<Vec<u32>>(false),           // compacted_faces (output)
            ),
        ),
    );

    // Queue compute pipelines
    let generate_vertices_pipeline =
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("generate_vertices_pipeline".into()),
            layout: vec![generate_vertices_layout.clone()],
            shader: asset_server.load(GENERATE_VERTICES_SHADER),
            entry_point: Some("generate_vertices".into()),
            ..default()
        });

    let prefix_sum_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("prefix_sum_pipeline".into()),
        layout: vec![prefix_sum_layout.clone()],
        shader: asset_server.load(PREFIX_SUM_SHADER),
        entry_point: Some("prefix_sum".into()),
        ..default()
    });

    let compact_vertices_pipeline =
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("compact_vertices_pipeline".into()),
            layout: vec![compact_vertices_layout.clone()],
            shader: asset_server.load(COMPACT_VERTICES_SHADER),
            entry_point: Some("compact_vertices".into()),
            ..default()
        });

    let generate_faces_pipeline =
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("generate_faces_pipeline".into()),
            layout: vec![generate_faces_layout.clone()],
            shader: asset_server.load(GENERATE_FACES_SHADER),
            entry_point: Some("generate_faces".into()),
            ..default()
        });

    let compact_faces_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("compact_faces_pipeline".into()),
        layout: vec![compact_faces_layout.clone()],
        shader: asset_server.load(COMPACT_FACES_SHADER),
        entry_point: Some("compact_faces".into()),
        ..default()
    });

    commands.insert_resource(SurfaceNetsPipelines {
        generate_vertices_pipeline,
        prefix_sum_pipeline,
        compact_vertices_pipeline,
        generate_faces_pipeline,
        compact_faces_pipeline,
    });

    // Store bind group layouts
    commands.insert_resource(SurfaceNetsBindGroupLayouts {
        generate_vertices: generate_vertices_layout,
        prefix_sum: prefix_sum_layout,
        compact_vertices: compact_vertices_layout,
        generate_faces: generate_faces_layout,
        compact_faces: compact_faces_layout,
    });
}
