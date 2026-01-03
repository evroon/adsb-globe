pub mod adsb;
pub mod earth;
pub mod skybox;

use std::collections::HashMap;

use adsb::{init_adsb, move_aircraft};
use bevy::{pbr::ExtendedMaterial, prelude::*, render::view::Hdr};
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};
use earth::system::{EarthMaterialExtension, setup_earth};
use skybox::system::{init_skybox, update_skybox};

use crate::adsb::task_pool::{DataFetch, handle_tasks, spawn_task};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,))
        .add_plugins((FlyCameraPlugin, WhereWasIPlugin::default()))
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, EarthMaterialExtension>,
        >::default())
        .insert_resource(DataFetch(HashMap::new()))
        .add_systems(
            Startup,
            (
                setup,
                setup_earth,
                init_adsb,
                init_skybox,
                spawn_task.after(init_adsb),
            ),
        )
        .add_systems(Update, (move_aircraft, update_skybox, handle_tasks))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            ..Default::default()
        },
        Hdr,
        FlyCam,
        WhereWasI::camera(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
