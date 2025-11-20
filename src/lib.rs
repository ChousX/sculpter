use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_graph::{RenderGraph, RenderLabel},
    },
};

use crate::{
    bind_group::prepare_bind_groups,
    buffers::{SurfaceNetsBuffers, prepare_surface_nets_buffers},
    mesh::build_mesh_from_readback,
    node::SurfaceNetsNode,
    pipeline::init_surface_nets_pipelines,
    readback::setup_readback_for_new_fields,
};

mod bind_group;
mod buffers;
mod mesh;
mod node;
mod pipeline;
mod readback;

pub mod prelude {
    pub use crate::{DensityField, DensityFieldMeshSize, DensityFieldSize, SculpterPlugin};
}

pub struct SculpterPlugin;
impl Plugin for SculpterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DensityFieldSize>()
            .init_resource::<DensityFieldMeshSize>()
            .add_plugins((
                ExtractComponentPlugin::<DensityField>::default(),
                ExtractResourcePlugin::<DensityFieldSize>::default(),
                ExtractComponentPlugin::<SurfaceNetsBuffers>::default(),
            ))
            .add_systems(
                Update,
                (
                    prepare_surface_nets_buffers,
                    setup_readback_for_new_fields,
                    build_mesh_from_readback,
                )
                    .chain(),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            error!("Failed to get render app");
            return;
        };

        render_app
            .add_systems(RenderStartup, init_surface_nets_pipelines)
            .add_systems(
                Render,
                (
                    //prepare_surface_nets_buffers.in_set(RenderSystems::PrepareResources),
                    prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
                )
                    .chain(),
            );
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(SurfaceNetsLabel, SurfaceNetsNode::default());
        render_graph.add_node_edge(SurfaceNetsLabel, bevy::render::graph::CameraDriverLabel);
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
        Self(uvec3(32, 32, 32))
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
