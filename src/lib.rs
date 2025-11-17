use bevy::{
    prelude::*,
    render::{
        RenderApp,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::ExtractResource,
        render_graph::RenderLabel,
        render_resource::{CommandEncoderDescriptor, ComputePassDescriptor, PipelineCache},
        renderer::{RenderDevice, RenderQueue},
        storage::ShaderStorageBuffer,
    },
};

use crate::{buffers::SurfaceNetsBuffers, pipeline::SurfaceNetsPipelines};

mod bind_group;
mod buffers;
mod node;
mod pipeline;

pub struct SculpterPlugin;
impl Plugin for SculpterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DensityFieldSize>()
            .init_resource::<DensityFieldMeshSize>()
            .add_plugins(ExtractComponentPlugin::<DensityField>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            error!("Failed to get render app");
            return;
        };
    }
}

#[derive(Resource, ExtractResource, Deref, DerefMut, Clone, Copy, Debug)]
pub struct DensityFieldSize(pub UVec3);

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct SurfaceNetsLabel;

impl DensityFieldSize {
    pub fn density_count(&self) -> u32 {
        self.x * self.y * self.z
    }

    pub fn index(&self, x: u32, y: u32, z: u32) -> u32 {
        z * self.y * self.x + y * self.x + x
    }

    pub fn cell_count(&self) -> u32 {
        (self.x.saturating_sub(1)) * (self.y.saturating_sub(1)) * (self.z.saturating_sub(1))
    }
}

impl Default for DensityFieldSize {
    fn default() -> Self {
        Self(uvec3(10, 10, 10))
    }
}

#[derive(Resource, Clone, Copy, Deref, DerefMut, Debug)]
pub struct DensityFieldMeshSize(pub Vec3);
impl Default for DensityFieldMeshSize {
    fn default() -> Self {
        Self(vec3(10., 10., 10.))
    }
}

#[derive(Component, ExtractComponent, Clone, DerefMut, Deref, Debug)]
pub struct DensityField(pub Vec<f32>);

#[derive(Component, Debug)]
pub struct MeshGenerationTarget(pub Entity);

/// Prepare Buffers (per entity)
fn prepare_surface_nets_buffers(
    mut commands: Commands,
    // Query entities that have DensityField but no Mesh3d
    needs_mesh_query: Query<(Entity, &DensityField), Without<Mesh3d>>,
    dimensions: Res<DensityFieldSize>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (entity, density_field) in needs_mesh_query.iter() {
        // Create GPU buffers to start generation
        let buffers = SurfaceNetsBuffers::new(density_field, &dimensions, &mut buffers);
        commands.entity(entity).insert(buffers);
    }
}
