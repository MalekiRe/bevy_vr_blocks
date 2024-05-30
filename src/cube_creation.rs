use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::audio::{AudioBundle, AudioSink, PlaybackMode, PlaybackSettings, Volume};
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{default, AudioSinkPlayback, Color, Commands, Cuboid, Deref, DerefMut, Entity, Gizmos, GlobalTransform, Local, Mesh, Query, Res, ResMut, Resource, Transform, With, Event, EventWriter, EventReader};
use bevy_xpbd_3d::components::{LinearVelocity, RigidBody};
use bevy_xpbd_3d::prelude::Collider;
use bevy_xr::hands::{HandBone, LeftHand, RightHand};
use random_number::random;
use std::ops::Deref;

pub struct CubeCreationPlugin;

impl Plugin for CubeCreationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_audio);
        app.add_systems(Update, (create_cube, draw_cube));
        app.add_event::<MakeCube>();
    }
}

fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let creation_hum = commands
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
    commands.insert_resource(CreationHum(creation_hum));
}

#[derive(Resource, Deref, DerefMut, Copy, Clone)]
pub struct CreationHum(pub Entity);

#[derive(Event, Copy, Clone)]
pub enum MakeCube {
    StartMaking,
    FinishMaking,
}

fn create_cube(
    mut audio_query: Query<&mut AudioSink>,
    audio_thing: Res<CreationHum>,
    mut event_writer: EventWriter<MakeCube>,
    left_hand: Query<(&HandBone, &GlobalTransform), With<LeftHand>>,
    right_hand: Query<(&HandBone, &GlobalTransform), With<RightHand>>,
) {
    const THUMB_INDEX_DISTANCE: f32 = 0.07;
    for (left_bone, index_pos) in left_hand.iter() {
        if !matches!(left_bone, HandBone::IndexTip) {
            continue;
        }
        for (left_bone, thumb_pos) in left_hand.iter() {
            if !matches!(left_bone, HandBone::ThumbTip) {
                continue;
            }
            if index_pos.translation().distance(thumb_pos.translation()) >= THUMB_INDEX_DISTANCE {
                event_writer.send(MakeCube::FinishMaking);
                return;
            }
        }
    }

    for (right_bone, index_pos) in right_hand.iter() {
        if !matches!(right_bone, HandBone::IndexTip) {
            continue;
        }
        for (right_bone, thumb_pos) in right_hand.iter() {
            if !matches!(right_bone, HandBone::ThumbTip) {
                continue;
            }
            if index_pos.translation().distance(thumb_pos.translation()) >= THUMB_INDEX_DISTANCE {
                event_writer.send(MakeCube::FinishMaking);
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
                event_writer.send(MakeCube::StartMaking);
                audio_query.get(audio_thing.0).unwrap().set_volume(0.7);
            }
        }
    }
}

fn draw_cube(
    mut gizmos: Gizmos,
    left_hand: Query<(&HandBone, &GlobalTransform), With<LeftHand>>,
    right_hand: Query<(&HandBone, &GlobalTransform), With<RightHand>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    audio_thing: Res<CreationHum>,
    mut audio_query: Query<&mut AudioSink>,
    mut make_cube: EventReader<MakeCube>,
    mut current_cube_stage: Local<Option<MakeCube>>,
) {
    for e in make_cube.read() {
        match e {
            MakeCube::StartMaking => {}
            MakeCube::FinishMaking => {
                if current_cube_stage.is_none() {
                    return;
                }
            }
        }
        current_cube_stage.replace(e.clone());
    }

    let Some(cube_stage) = current_cube_stage.as_ref().cloned() else { return };

    for (left_bone, left_pos) in left_hand.iter() {
        if !matches!(left_bone, HandBone::IndexTip) {
            continue;
        }
        for (right_bone, right_pos) in right_hand.iter() {
            if !matches!(right_bone, HandBone::IndexTip) {
                continue;
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

            match cube_stage {
                MakeCube::StartMaking => {
                    gizmos.cuboid(transform, Color::rgb_u8(0, 255, 0));
                }
                MakeCube::FinishMaking => {
                    let mut new_cube = commands.spawn((PbrBundle {
                        mesh: meshes.add(Cuboid::from_size(scale)),
                        material: materials.add(Color::rgb_u8(
                            random!(0..255),
                            random!(0..255),
                            random!(0..255),
                        )),
                        transform: Transform::from_translation(transform.translation),
                        ..default()
                    },));
                    new_cube.insert((
                        RigidBody::Dynamic,
                        LinearVelocity(Vec3::new(0.0, 0.0, 0.0)),
                        Collider::cuboid(transform.scale.x, transform.scale.y, transform.scale.z),
                    ));
                    audio_query.get(audio_thing.0).unwrap().set_volume(0.00);
                    current_cube_stage.take();
                }
            }

        }
    }
}
