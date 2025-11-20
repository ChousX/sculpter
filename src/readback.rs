use bevy::{
    prelude::*,
    render::gpu_readback::{Readback, ReadbackComplete},
};

use crate::buffers::SurfaceNetsBuffers;

#[derive(Component, Default)]
pub struct ReadbackBuffers {
    pub vertex_count: Option<u32>,
    pub vertices: Option<Vec<f32>>,
    pub face_count: Option<u32>,
    pub faces: Option<Vec<u32>>,
}

pub fn setup_readback_for_new_fields(
    mut commands: Commands,
    new_buffers: Query<
        (Entity, &SurfaceNetsBuffers),
        (Added<SurfaceNetsBuffers>, Without<ReadbackBuffers>),
    >,
) {
    for (parent_entity, buffers) in new_buffers {
        let vertex_count_entity = commands
            .spawn(Readback::buffer(buffers.vertex_count.clone()))
            .observe(
                |event: On<ReadbackComplete>,
                 children_of: Query<&ChildOf>,
                 mut commands: Commands,
                 mut readback_buffers: Query<&mut ReadbackBuffers>| {
                    let parent = children_of
                        .get(event.entity)
                        .expect("Readback is not a child of anything")
                        .parent();

                    let mut buffers = readback_buffers
                        .get_mut(parent)
                        .expect("parent of readback does not have ReadbackBuffers");

                    let data: Vec<u32> = event.to_shader_type();
                    //get the vertex count and if there is none set it to 0
                    let vertex_count = data.get(0).copied().unwrap_or(0);

                    info!("VertexCount Readback Complete for:{parent}");
                    #[cfg(feature = "verbose_readback_vertex_count")]
                    info!("VertexCount:{vertex_count}");
                    buffers.vertex_count = Some(vertex_count);

                    commands.entity(event.entity).despawn();
                },
            )
            .id();

        let vertices_entity = commands
            .spawn(Readback::buffer(buffers.vertices.clone()))
            .observe(
                |event: On<ReadbackComplete>,
                 children_of: Query<&ChildOf>,
                 mut commands: Commands,
                 mut readback_buffers: Query<&mut ReadbackBuffers>| {
                    let parent = children_of
                        .get(event.entity)
                        .expect("Readback is not a child of anything")
                        .parent();

                    let mut buffers = readback_buffers
                        .get_mut(parent)
                        .expect("parent of readback does not have ReadbackBuffers");

                    let vertices: Vec<f32> = event.to_shader_type();

                    info!("Vertices Readback Complete for:{parent}");
                    #[cfg(feature = "verbose_readback_vertices")]
                    info!("Vertices:{vertices:?}");
                    buffers.vertices = Some(vertices);

                    commands.entity(event.entity).despawn();
                },
            )
            .id();
        let face_count_entity = commands
            .spawn(Readback::buffer(buffers.face_count.clone()))
            .observe(
                |event: On<ReadbackComplete>,
                 children_of: Query<&ChildOf>,
                 mut commands: Commands,
                 mut readback_buffers: Query<&mut ReadbackBuffers>| {
                    let parent = children_of
                        .get(event.entity)
                        .expect("Readback is not a child of anything")
                        .parent();

                    let mut buffers = readback_buffers
                        .get_mut(parent)
                        .expect("parent of readback does not have ReadbackBuffers");
                    let data: Vec<u32> = event.to_shader_type();
                    //get the vertex count and if there is none set it to 0
                    let face_count = data.get(0).copied().unwrap_or(0);

                    info!("FaceCount Readback Complete for:{parent}");
                    #[cfg(feature = "verbose_readback_face_count")]
                    info!("FaceCount:{face_count}");

                    buffers.face_count = Some(face_count);

                    commands.entity(event.entity).despawn();
                },
            )
            .id();
        let faces_entity = commands
            .spawn(Readback::buffer(buffers.faces.clone()))
            .observe(
                |event: On<ReadbackComplete>,
                 children_of: Query<&ChildOf>,
                 mut commands: Commands,
                 mut readback_buffers: Query<&mut ReadbackBuffers>| {
                    let parent = children_of
                        .get(event.entity)
                        .expect("Readback is not a child of anything")
                        .parent();

                    let mut buffers = readback_buffers
                        .get_mut(parent)
                        .expect("parent of readback does not have ReadbackBuffers");
                    let faces: Vec<u32> = event.to_shader_type();

                    info!("Faces Readback Complete for:{parent}");
                    #[cfg(feature = "verbose_readback_faces")]
                    info!("Faces:{faces:?}");
                    buffers.faces = Some(faces);

                    commands.entity(event.entity).despawn();
                },
            )
            .id();

        commands
            .entity(parent_entity)
            .insert(ReadbackBuffers::default())
            .add_children(&[
                vertex_count_entity,
                vertices_entity,
                face_count_entity,
                faces_entity,
            ]);
    }
}
