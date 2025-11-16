use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        render_resource::{CommandEncoderDescriptor, ComputePassDescriptor, PipelineCache},
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::buffers::SurfaceNetsBuffers;
use crate::pipeline::SurfaceNetsGpuPipeline;
mod buffers;
mod pipeline;

pub struct SculpterPlugin;
impl Plugin for SculpterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DensityFieldSize>()
            .init_resource::<DensityFieldMeshSize>();
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            error!("Failed to get render app");
            return;
        };

        render_app
            .init_resource::<SurfaceNetsGpuPipeline>()
            .add_systems(
                Render,
                (
                    //copys reveint data form world to render_world for algorithm
                    check_needs_generation.in_set(RenderSystems::PrepareResources),
                    //runs all the shaders
                    queue_surface_nets_compute.in_set(RenderSystems::Queue),
                    //grabs all the mesh data that was generated and builds a mesh for world
                    cleanup_buffers.in_set(RenderSystems::Cleanup),
                )
                    //Don't think we need this chain here they are in defient sets already
                    .chain(),
            );
    }
}

#[derive(Resource, Debug)]
pub struct DensityFieldSize {
    x: usize,
    y: usize,
    z: usize,
}

impl DensityFieldSize {
    pub fn density_count(&self) -> usize {
        self.x * self.y * self.z
    }

    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        z * self.y * self.x + y * self.x + x
    }

    pub fn cell_count(&self) -> usize {
        (self.x.saturating_sub(1)) * (self.y.saturating_sub(1)) * (self.z.saturating_sub(1))
    }
}

impl From<UVec3> for DensityFieldSize {
    fn from(value: UVec3) -> Self {
        Self {
            x: value.x as usize,
            y: value.y as usize,
            z: value.z as usize,
        }
    }
}

impl Default for DensityFieldSize {
    fn default() -> Self {
        Self {
            x: 10,
            y: 10,
            z: 10,
        }
    }
}

#[derive(Resource, Debug)]
pub struct DensityFieldMeshSize(pub Vec3);
impl Default for DensityFieldMeshSize {
    fn default() -> Self {
        Self(vec3(10., 10., 10.))
    }
}

#[derive(Component, Debug)]
pub struct DensityField(pub Vec<f32>);

#[derive(Component, Debug)]
pub struct MeshGenerationTarget(pub Entity);

/// Check if we need to generate a mesh
fn check_needs_generation(
    mut commands: Commands,
    // Query entities that have DensityField but no Mesh3d
    needs_mesh_query: Query<(Entity, &DensityField), Without<Mesh3d>>,
    size: Res<DensityFieldSize>,
    render_device: Res<RenderDevice>,
    pipeline: Res<SurfaceNetsGpuPipeline>,
) {
    for (entity, density_field) in needs_mesh_query.iter() {
        // Create GPU buffers to start generation
        let buffers = SurfaceNetsBuffers::new(
            &render_device,
            &pipeline.bind_group_layout,
            &density_field.0,
            &size,
        );

        // Spawn a temporary entity to hold the buffers during generation
        commands.spawn((buffers, MeshGenerationTarget(entity)));
    }
}

/// Dispatch the compute shader to generate the mesh
fn queue_surface_nets_compute(
    buffer_query: Query<&SurfaceNetsBuffers>,
    size: Res<DensityFieldSize>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline: Res<SurfaceNetsGpuPipeline>,
    pipeline_cache: Res<PipelineCache>,
) {
    for buffers in buffer_query.iter() {
        // Check if all pipelines are ready
        let Some(generate_verts) =
            pipeline_cache.get_compute_pipeline(pipeline.generate_vertices_pipeline)
        else {
            continue;
        };
        let Some(prefix_sum) = pipeline_cache.get_compute_pipeline(pipeline.prefix_sum_pipeline)
        else {
            continue;
        };
        let Some(compact_verts) =
            pipeline_cache.get_compute_pipeline(pipeline.compact_vertices_pipeline)
        else {
            continue;
        };
        let Some(generate_faces) =
            pipeline_cache.get_compute_pipeline(pipeline.generate_faces_pipeline)
        else {
            continue;
        };
        let Some(compact_faces) =
            pipeline_cache.get_compute_pipeline(pipeline.compact_faces_pipeline)
        else {
            continue;
        };

        let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("surface_nets_encoder"),
        });

        let workgroup_size = 8u32;
        let workgroups_x = (size.x as u32).div_ceil(workgroup_size);
        let workgroups_y = (size.y as u32).div_ceil(workgroup_size);
        let workgroups_z = (size.z as u32).div_ceil(workgroup_size);

        // !I don't think we are ending all required buffers to each pipeline!

        // Generate vertices
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("generate_vertices"),
                timestamp_writes: None,
            });
            pass.set_pipeline(generate_verts);
            //I don't think this is correct. I think this is where we add all are buffers right?
            pass.set_bind_group(0, &buffers.bind_group, &[]);
            pass.dispatch_workgroups(workgroups_x, workgroups_y, workgroups_z);
        }

        // Compact vertices
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("compact_vertices"),
                timestamp_writes: None,
            });
            pass.set_pipeline(compact_verts);
            pass.set_bind_group(0, &buffers.bind_group, &[]);
            let threads = (buffers.total_cells as u32 + 255) / 256;
            pass.dispatch_workgroups(threads, 1, 1);
        }

        // Generate faces
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("generate_faces"),
                timestamp_writes: None,
            });
            pass.set_pipeline(generate_faces);
            pass.set_bind_group(0, &buffers.bind_group, &[]);
            pass.dispatch_workgroups(workgroups_x, workgroups_y, workgroups_z);
        }

        // Compact faces
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("compact_faces"),
                timestamp_writes: Nonev,
            });
            pass.set_pipeline(compact_faces);
            pass.set_bind_group(0, &buffers.bind_group, &[]);
            let threads = (buffers.max_faces as u32 + 255) / 256;
            pass.dispatch_workgroups(threads, 1, 1);
        }

        render_queue.submit(std::iter::once(encoder.finish()));
    }
}

/// After GPU work is complete, read back data and create the mesh
/// Then cleanup temporary entities and buffers
fn cleanup_buffers(
    mut commands: Commands,
    buffer_query: Query<(Entity, &SurfaceNetsBuffers, &MeshGenerationTarget)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //Buffers are how we get data back to the cpu right?
    for (temp_entity, buffers, target) in buffer_query.iter() {
        // In a real implementation, you'd:
        // 1. Check if GPU work is done (via fence/query). Is there any way its not done? can
        //    compute shader take longer than one frame?
        // 2. Read back vertex and face data
        // 3. Create Bevy mesh from the data
        // 4. Add Mesh3d to the target entity
        // 5. Despawn this temporary entity. I think this is redundent in the render world

        // For now, just cleanup
        // TODO: Actually create the mesh here
        // let mesh = Mesh::new(...); // from readback data
        // let mesh_handle = meshes.add(mesh);
        // let material_handle = materials.add(StandardMaterial::default());
        //
        // commands.entity(target.entity).insert((
        //     Mesh3d(mesh_handle),
        //     MeshMaterial3d(material_handle),
        // ));

        // Despawn the temporary entity (this will drop the buffers)
        commands.entity(temp_entity).despawn();
    }
}
