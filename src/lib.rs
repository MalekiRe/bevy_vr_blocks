//! A simple 3D scene with light shining over a cube sitting on a plane

use crate::cube_creation::CubeCreationPlugin;
use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_mod_openxr::{add_xr_plugins, init::OxrInitPlugin, types::OxrExtensions};
use avian3d::prelude::*;
use bevy_mod_openxr::session::OxrSession;
use bevy_mod_xr::hands::HandBoneRadius;

pub mod cube_creation;

#[bevy_main]
pub fn main() {
    let mut app = App::new();
    app.add_plugins(add_xr_plugins(DefaultPlugins).set(OxrInitPlugin {
        app_info: default(),
        exts: {
            let mut exts = OxrExtensions::default();
            exts.enable_fb_passthrough();
            exts.enable_hand_tracking();
            //exts.enable_custom_refresh_rates();
            exts
        },
        blend_modes: default(),
        backends: default(),
        formats: default(),
        resolutions: default(),
        synchronous_pipeline_compilation: default(),
    }))
    // System for requesting refresh rate ( should refactor and upstream into bevy_openxr )
    //.add_systems(Update, set_requested_refresh_rate)
    // Our plugins
    .add_plugins((CubeCreationPlugin, CustomPhysicsIntegrations))
    // Third party plugins
    .add_plugins((
        EmbeddedAssetPlugin::default(),
        PhysicsPlugins::default(),
        bevy_xr_utils::hand_gizmos::HandGizmosPlugin,
    ))
    // Setup
    .add_systems(Startup, setup)
    // Realtime lighting is expensive, use ambient light instead
    .insert_resource(AmbientLight {
        color: Default::default(),
        brightness: 500.0,
    })
    .insert_resource(Msaa::Off)
    .insert_resource(ClearColor(Color::NONE))
    .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawns a plane
    // For improved performance set the `unlit` value to true on standard materials
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 0.002, 1.0),
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..default()
        },
    ));
}

pub struct CustomPhysicsIntegrations;

impl Plugin for CustomPhysicsIntegrations {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hand_collider);
        app.add_systems(Update, play_sound_when_colliding);
    }
}

fn hand_collider(
    query: Query<(Entity, &HandBoneRadius), Without<Collider>>,
    mut commands: Commands,
) {
    for (entity, radius) in &query {
        commands
            .entity(entity)
            .insert((Collider::sphere(radius.0), RigidBody::Static, ColliderDensity(10.0)));
    }
}

fn play_sound_when_colliding(
    query: Query<&CollidingEntities>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    velocities: Query<&LinearVelocity>,
) {
    for colliding_entities in &query {
        for e in colliding_entities.0.iter() {
            if let Ok(t) = transforms.get(*e) {
                if velocities.get(*e).is_ok_and(|a| a.length() >= 1.0) {
                    commands.spawn((
                        AudioBundle {
                            source: asset_server.load("embedded://plastic-hit.ogg"),
                            settings: PlaybackSettings {
                                mode: PlaybackMode::Despawn,
                                volume: Volume::new(velocities.get(*e).unwrap().length() * 2.0),
                                speed: 1.0,
                                paused: false,
                                spatial: true,
                                spatial_scale: None,
                            },
                        },
                        SpatialBundle::from_transform(t.compute_transform()),
                    ));
                    return;
                }
            }
        }
    }
}

fn set_requested_refresh_rate(mut local: Local<bool>, mut session: Option<ResMut<OxrSession>>) {
    if session.is_none() {
        return;
    }
    if *local {
        return;
    }
    *local = true;
    session
        .as_mut()
        .unwrap()
        .request_display_refresh_rate(72.0)
        .unwrap();
}
