use bevy::prelude::*;
use avian3d::prelude::*;
use bevy_seedling::{pool::SamplerPool, sample_effects};
use bevy_steam_audio::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy_seedling::prelude::VolumeNode;
use bevy_seedling::prelude::SamplePlayer;
use rand::Rng;

use crate::game::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            spawn_floor,
            spawn_backrooms,
            setup_objects,
        ));
    }
}

fn spawn_floor(
    mut commands: Commands,
) {
    commands.spawn((
        // Mesh3d(meshes.add(Cuboid::new(1000.0, 0.1, 1.0))),
        Transform::from_xyz(0.0, -20.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(0.1, 0.1, 0.1),
    ));
}

fn spawn_backrooms(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("map/room1.glb"));
    commands.spawn((
        SceneRoot(scene),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.3, 2.0, 1.3)),
        SteamAudioMaterial::default(), // 音響ジオメトリ
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        Friction::new(1.0),
        Restitution::new(RESTITUTION),
    ))
    .with_children(|p| {
        p.spawn((
            SamplerPool(RoomSfxPool),
            sample_effects![VolumeNode {
                volume: bevy_seedling::prelude::Volume::Decibels(-16.0),
                ..default()
            }],
        ));
        p.spawn((
            RoomSfxPool,
            SamplePlayer::new(asset_server.load("sfx/room_tone.ogg")).looping(),
            // SteamAudioPool
        ));
    });
}

fn setup_objects(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::rng();

    let chair_scene: Handle<Scene> = asset_server.load(GltfAssetLabel::Scene(0).from_asset("object/chair.glb"));
    let doll1_scene: Handle<Scene> = asset_server.load(GltfAssetLabel::Scene(0).from_asset("object/doll3.glb"));

    commands.spawn((
        SceneRoot(chair_scene.clone()),
        Transform::from_xyz(rng.random_range(-1.0..1.0), -18.5, 500.0).with_scale(Vec3::splat(3.0)),
        Visibility::default(),
        RigidBody::Static,
    ));

    commands.spawn((
        Interactable {
            prompt: "E".to_string()
        },
        SceneRoot(doll1_scene.clone()),
        Transform::from_xyz(rng.random_range(-1.0..1.0), -18.5, 10.0),
        Visibility::default(),
        RigidBody::Static,
    ));
}