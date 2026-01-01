use adsb_globe::adsb::{init_adsb, move_aircraft};
use adsb_globe::earth::system::{EarthMaterialExtension, setup_earth};
use adsb_globe::skybox::system::{init_skybox, update_skybox};
use bevy::{pbr::ExtendedMaterial, prelude::*, render::view::Hdr};
use bevy_volumetric_clouds::fly_camera::{FlyCam, FlyCameraPlugin};
use bevy_where_was_i::{WhereWasI, WhereWasIPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,))
        .add_plugins((FlyCameraPlugin, WhereWasIPlugin::default()))
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, EarthMaterialExtension>,
        >::default())
        .add_systems(Startup, (setup, setup_earth, init_adsb, init_skybox))
        .add_systems(Update, (move_aircraft, update_skybox))
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
