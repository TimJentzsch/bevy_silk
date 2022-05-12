#![allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::option_if_let_else
)]

use crate::cloth::Cloth;
use crate::cloth_rendering::ClothRendering;
use crate::config::ClothConfig;
use crate::wind::Winds;
use crate::ClothBuilder;
use bevy_asset::{Assets, Handle};
use bevy_core::Time;
use bevy_ecs::prelude::*;
use bevy_log::{debug, warn};
use bevy_math::Vec3;
use bevy_render::prelude::Mesh;
use bevy_transform::prelude::GlobalTransform;

#[allow(clippy::cast_possible_truncation)]
pub fn update_cloth(
    mut query: Query<(
        &mut Cloth,
        &mut ClothRendering,
        &GlobalTransform,
        &Handle<Mesh>,
        Option<&ClothConfig>,
    )>,
    config: Res<ClothConfig>,
    wind: Option<Res<Winds>>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let delta_time = time.delta_seconds();
    let wind_force = wind.map_or(Vec3::ZERO, |w| {
        w.current_velocity(time.time_since_startup().as_secs_f32())
    });
    for (mut cloth, mut rendering, transform, handle, custom_config) in query.iter_mut() {
        if let Some(mesh) = meshes.get_mut(handle) {
            let matrix = transform.compute_matrix();
            cloth.update(
                custom_config.unwrap_or(&config),
                delta_time,
                &matrix,
                wind_force,
            );
            rendering.update_positions(cloth.compute_vertex_positions(&matrix));
            rendering.apply(mesh);
        } else {
            warn!("A Cloth has a `ClothRendering` component without a loaded mesh");
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn init_cloth(
    mut commands: Commands,
    query: Query<(Entity, &ClothBuilder, &GlobalTransform, &Handle<Mesh>), Without<Cloth>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, builder, transform, handle) in query.iter() {
        if let Some(mesh) = meshes.get(handle) {
            let matrix = transform.compute_matrix();
            debug!("Initializing Cloth");
            let rendering = ClothRendering::init(mesh, builder.compute_flat_normals).unwrap();
            let cloth = Cloth::new(
                &rendering.vertex_positions,
                &rendering.indices,
                builder.fixed_points.clone(),
                builder.stick_generation,
                builder.stick_length,
                &matrix,
            );
            commands.entity(entity).insert(rendering).insert(cloth);
        }
    }
}
