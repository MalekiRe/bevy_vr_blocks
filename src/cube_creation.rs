use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::audio::{AudioBundle, AudioSink, PlaybackMode, PlaybackSettings, Volume};
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    default, AudioSinkPlayback, Color, Commands, Cuboid, Entity, Gizmos, GlobalTransform, Local,
    Mesh, Query, Res, ResMut, Resource, Transform, With,
};
use bevy_xpbd_3d::components::{LinearVelocity, RigidBody};
use bevy_xpbd_3d::prelude::Collider;
use bevy_xr::hands::{HandBone, LeftHand, RightHand};

pub struct CubeCreationPlugin;

impl Plugin for CubeCreationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            |mut commands: Commands, asset_server: Res<AssetServer>| {
                let e = commands
                    .spawn(
                        (AudioBundle {
                            source: asset_server.load("embedded://hum.ogg"),
                            settings: PlaybackSettings {
                                mode: PlaybackMode::Loop,
                                volume: Volume::new(0.7),
                                speed: 1.0,
                                paused: false,
                                spatial: false,
                                spatial_scale: None,
                            },
                        }),
                    )
                    .id();
                commands.insert_resource(MakingCube(false));
                commands.insert_resource(AudioThing(e));
            },
        );
        app.add_systems(Update, (create_cube, draw_cube));
    }
}

#[derive(Resource)]
pub struct MakingCube((bool));

#[derive(Resource)]
pub struct AudioThing(pub Entity);

fn create_cube(
    left_hand: Query<(&HandBone, &GlobalTransform), With<LeftHand>>,
    right_hand: Query<(&HandBone, &GlobalTransform), With<RightHand>>,
    mut making_cube: ResMut<MakingCube>,
    mut local: Local<Option<Entity>>,
    audio_thing: Res<AudioThing>,
    mut audio_query: Query<&mut AudioSink>,
) {
    for (left_bone, index_pos) in left_hand.iter() {
        match left_bone {
            HandBone::IndexTip => {}
            _ => continue,
        }
        for (left_bone, thumb_pos) in left_hand.iter() {
            match left_bone {
                HandBone::ThumbTip => {}
                _ => continue,
            }
            if index_pos.translation().distance(thumb_pos.translation()) >= 0.07 {
                local.take();
                return;
            }
        }
    }

    for (right_bone, index_pos) in right_hand.iter() {
        match right_bone {
            HandBone::IndexTip => {}
            _ => continue,
        }
        for (right_bone, thumb_pos) in right_hand.iter() {
            match right_bone {
                HandBone::ThumbTip => {}
                _ => continue,
            }
            if index_pos.translation().distance(thumb_pos.translation()) >= 0.07 {
                local.take();
                return;
            }
        }
    }

    for (left_bone, left_pos) in left_hand.iter() {
        match left_bone {
            HandBone::IndexTip => {}
            _ => continue,
        }
        for (right_bone, right_pos) in right_hand.iter() {
            match right_bone {
                HandBone::IndexTip => {}
                _ => continue,
            }

            if left_pos.translation().distance(right_pos.translation()) <= 0.03 {
                making_cube.0 = true;
                audio_query.get(audio_thing.0).unwrap().set_volume(0.7);
            }
        }
    }
}

fn draw_cube(
    mut making_cube: ResMut<MakingCube>,
    mut gizmos: Gizmos,
    left_hand: Query<(&HandBone, &GlobalTransform), With<LeftHand>>,
    right_hand: Query<(&HandBone, &GlobalTransform), With<RightHand>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut playback_settings: Query<&mut PlaybackSettings>,
    asset_server: Res<AssetServer>,
    audio_thing: Res<AudioThing>,
    mut audio_query: Query<&mut AudioSink>,
    mut local: Local<i32>,
) {
    if !making_cube.0 {
        return;
    }

    for (left_bone, index_pos) in left_hand.iter() {
        match left_bone {
            HandBone::IndexTip => {}
            _ => continue,
        }
        for (left_bone, thumb_pos) in left_hand.iter() {
            match left_bone {
                HandBone::ThumbTip => {}
                _ => continue,
            }
            if index_pos.translation().distance(thumb_pos.translation()) >= 0.07 {
                making_cube.0 = false;
            }
        }
    }

    for (right_bone, index_pos) in right_hand.iter() {
        match right_bone {
            HandBone::IndexTip => {}
            _ => continue,
        }
        for (right_bone, thumb_pos) in right_hand.iter() {
            match right_bone {
                HandBone::ThumbTip => {}
                _ => continue,
            }
            if index_pos.translation().distance(thumb_pos.translation()) >= 0.07 {
                making_cube.0 = false;
            }
        }
    }

    for (left_bone, left_pos) in left_hand.iter() {
        match left_bone {
            HandBone::IndexTip => {}
            _ => continue,
        }
        for (right_bone, right_pos) in right_hand.iter() {
            match right_bone {
                HandBone::IndexTip => {}
                _ => continue,
            }

            let mut scale = left_pos.translation() - right_pos.translation();

            scale.y = scale.y.abs();
            scale.x = scale.x.abs();
            scale.z = scale.z.abs();

            let mut left_corner = left_pos.translation();

            if right_pos.translation().x < left_corner.x {
                left_corner.x = right_pos.translation().x;
            }

            if right_pos.translation().y < left_corner.y {
                left_corner.y = right_pos.translation().y;
            }

            if right_pos.translation().z < left_corner.z {
                left_corner.z = right_pos.translation().z;
            }

            audio_query
                .get(audio_thing.0)
                .unwrap()
                .set_speed((scale.length() / 1.0) + 0.1);

            let transform =
                Transform::from_scale(scale).with_translation(left_corner + scale / 2.0);

            if !making_cube.0 {
                let mut e = commands.spawn((PbrBundle {
                    mesh: meshes.add(Cuboid::from_size(scale)),
                    material: materials.add(Color::rgb_u8(124, 144, 255)),
                    transform: Transform::from_translation(transform.translation),
                    ..default()
                },));
                e.insert((
                    RigidBody::Dynamic,
                    LinearVelocity(Vec3::new(0.0, -0.3, 0.0)),
                    Collider::cuboid(transform.scale.x, transform.scale.y, transform.scale.z),
                ));
                audio_query.get(audio_thing.0).unwrap().set_volume(0.01);
            }

            gizmos.cuboid(transform, Color::rgb_u8(0, 255, 0));
        }
    }
}
