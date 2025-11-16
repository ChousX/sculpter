use bevy::prelude::*;
mod buffers;
mod pipeline;

pub struct SculpterPlugin;
impl Plugin for SculpterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DensityFieldSize>()
            .init_resource::<DensityFieldMeshSize>();
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
