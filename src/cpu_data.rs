use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, extract_resource::ExtractResource},
};
#[derive(Resource, ExtractResource, Deref, DerefMut, Clone, Copy, Debug)]
pub struct DensityFieldSize(pub UVec3);

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
