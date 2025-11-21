mod cpu_data;
mod gpu;
pub mod prelude {
    pub use crate::SculpterPlugin;
    pub use crate::cpu_data::{DensityField, DensityFieldMeshSize, DensityFieldSize};
}

use bevy::prelude::*;
pub struct SculpterPlugin;
impl Plugin for SculpterPlugin {
    fn build(&self, app: &mut App) {}
}
