// buffers.rs
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
//
// Component that holds GPU buffers during generation (one per generating entity)
#[derive(Component)]
pub struct SurfaceNetsBuffers {
    pub density_field: Buffer,
    pub dimensions: Buffer,
    pub vertices: Buffer,
    pub vertex_valid: Buffer,
    pub vertex_indices: Buffer,
    pub faces: Buffer,
    pub face_valid: Buffer,
    pub face_indices: Buffer,
    pub bind_group: BindGroup,
    pub total_cells: usize,
    pub max_faces: usize,
}

impl SurfaceNetsBuffers {
    pub fn new(
        render_device: &RenderDevice,
        bind_group_layout: &BindGroupLayout,
        density_field: &[f32],
        size: &crate::DensityFieldSize,
    ) -> Self {
        let total_cells = size.cell_count();
        let max_faces = total_cells * 3;

        // Upload density field
        let density_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("density_field"),
            contents: bytemuck::cast_slice(density_field),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // Dimensions uniform
        let dimensions = UVec4::new(size.x as u32, size.y as u32, size.z as u32, 0);
        let dimensions_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("dimensions"),
            contents: bytemuck::cast_slice(&[dimensions]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create output buffers
        let vertices_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("vertices"),
            size: (total_cells * std::mem::size_of::<Vec3>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let vertex_valid_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("vertex_valid"),
            size: (total_cells * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let vertex_indices_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("vertex_indices"),
            size: (total_cells * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let faces_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("faces"),
            size: (max_faces * 4 * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let face_valid_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("face_valid"),
            size: (max_faces * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let face_indices_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("face_indices"),
            size: (max_faces * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = render_device.create_bind_group(
            Some("surface_nets_bind_group"),
            bind_group_layout,
            &BindGroupEntries::sequential((
                density_buffer.as_entire_buffer_binding(),
                dimensions_buffer.as_entire_buffer_binding(),
                vertices_buffer.as_entire_buffer_binding(),
                vertex_valid_buffer.as_entire_buffer_binding(),
                vertex_indices_buffer.as_entire_buffer_binding(),
                faces_buffer.as_entire_buffer_binding(),
                face_valid_buffer.as_entire_buffer_binding(),
                face_indices_buffer.as_entire_buffer_binding(),
            )),
        );

        Self {
            density_field: density_buffer,
            dimensions: dimensions_buffer,
            vertices: vertices_buffer,
            vertex_valid: vertex_valid_buffer,
            vertex_indices: vertex_indices_buffer,
            faces: faces_buffer,
            face_valid: face_valid_buffer,
            face_indices: face_indices_buffer,
            bind_group,
            total_cells,
            max_faces,
        }
    }
}
