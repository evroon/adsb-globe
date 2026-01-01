use bevy::{
    pbr::{MaterialExtension, OpaqueRendererMethod},
    prelude::*,
    render::render_resource::*,
    shader::ShaderRef,
};

use crate::earth::EARTH_RADIUS;

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/earth.wgsl";

pub fn setup_earth(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(EARTH_RADIUS))),
        Transform::from_xyz(21.0, 0.0, 0.0).with_scale(Vec3::splat(5.0)),
    ));
    // sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(EARTH_RADIUS))),
        MeshMaterial3d(materials.add(StandardMaterial {
            metallic_roughness_texture: Some(
                asset_server.load("textures/earth/earth_bump_roughness_clouds_4096.jpg"),
            ),
            base_color_texture: Some(asset_server.load("textures/earth/earth_day_4096.jpg")),
            specular_texture: Some(asset_server.load("textures/earth/earth_specular_2048.jpg")),
            normal_map_texture: Some(asset_server.load("textures/earth/earth_normal_2048.jpg")),
            metallic: 0.3,
            opaque_render_method: OpaqueRendererMethod::Auto,
            ..Default::default()
        })),
    ));
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct EarthMaterialExtension {
    #[texture(100)]
    #[sampler(101)]
    color_west: Option<Handle<Image>>,
}

impl MaterialExtension for EarthMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
