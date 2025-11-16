// pipeline.rs
use bevy::prelude::*;
use bevy::render::render_resource::binding_types::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;

#[derive(Resource)]
pub struct SurfaceNetsGpuPipeline {
    pub generate_vertices_pipeline: CachedComputePipelineId,
    pub compact_vertices_pipeline: CachedComputePipelineId,
    pub generate_faces_pipeline: CachedComputePipelineId,
    pub compact_faces_pipeline: CachedComputePipelineId,
    // Store the actual layout, not just an ID
    pub bind_group_layout: BindGroupLayout,
}

impl FromWorld for SurfaceNetsGpuPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let render_device = world.resource::<RenderDevice>();

        // Create bind group layout entries
        let bind_group_layout_entries = BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                // 0: Density field (read-only storage buffer)
                storage_buffer_read_only::<f32>(false),
                // 1: Dimensions (uniform)
                uniform_buffer::<UVec4>(false),
                // 2: Vertices output (storage buffer)
                storage_buffer::<Vec3>(false),
                // 3: Vertex valid flags (storage buffer)
                storage_buffer::<u32>(false),
                // 4: Vertex indices after compaction (storage buffer)
                storage_buffer::<u32>(false),
                // 5: Faces output (storage buffer)
                storage_buffer::<UVec4>(false),
                // 6: Face valid flags (storage buffer)
                storage_buffer::<u32>(false),
                // 7: Face indices after compaction (storage buffer)
                storage_buffer::<u32>(false),
            ),
        );

        // Create the bind group layout using RenderDevice
        let bind_group_layout = render_device.create_bind_group_layout(
            Some("surface_nets_bind_group_layout"),
            &bind_group_layout_entries,
        );

        // Load shaders
        let generate_vertices_shader = asset_server.load("shaders/generate_vertices.wgsl");
        let compact_vertices_shader = asset_server.load("shaders/compact_vertices.wgsl");
        let generate_faces_shader = asset_server.load("shaders/generate_faces.wgsl");
        let compact_faces_shader = asset_server.load("shaders/compact_faces.wgsl");

        // Queue compute pipelines
        let generate_vertices_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("generate_vertices_pipeline".into()),
                layout: vec![bind_group_layout.clone()],
                shader: generate_vertices_shader,
                entry_point: Some("main".into()),
                ..default()
            });

        let compact_vertices_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("compact_vertices_pipeline".into()),
                layout: vec![bind_group_layout.clone()],
                shader: compact_vertices_shader,
                entry_point: Some("main".into()),
                ..default()
            });

        let generate_faces_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("generate_faces_pipeline".into()),
                layout: vec![bind_group_layout.clone()],
                shader: generate_faces_shader,
                entry_point: Some("main".into()),
                ..default()
            });

        let compact_faces_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("compact_faces_pipeline".into()),
                layout: vec![bind_group_layout.clone()],
                shader: compact_faces_shader,
                entry_point: Some("main".into()),
                ..default()
            });

        Self {
            generate_vertices_pipeline,
            compact_vertices_pipeline,
            generate_faces_pipeline,
            compact_faces_pipeline,
            bind_group_layout,
        }
    }
}
