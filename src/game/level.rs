use bevy::prelude::*;
use avian3d::prelude::*;
use bevy_steam_audio::prelude::*;
use bevy::gltf::GltfAssetLabel;

use crate::game::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            spawn_floor,
            // setup_lights, 
            // setup_light, 
            // setup_level, 
            spawn_backrooms
        ));
    }
}

fn setup_light(
    mut commands: Commands
) {
    for z in 1..10 {
        for x in 1..10 {
            commands.spawn((
                PointLight {
                    intensity: 100_000.0,
                    shadows_enabled: false,
                    range: 1_000.0,
                    ..default()
                },
                Transform::from_xyz(((x*10)-25) as f32, 10.0, ((z*10)-25) as f32)
            ));
        }
    }
}

fn setup_lights(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(2.0, 0.5, 0.5));
    let mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::rgb(0.1, 0.1, 0.1),
        ..default()
    });

    for z in 1..5 {
        for x in 1..5 {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(((x*10)-25) as f32, 5.0, ((z*10)-25) as f32)
            ));
        }
    }
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let y = 5.0_f32;
    spawn_hollow_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(50.0, y, 50.0), // 外寸 (幅, 高さ, 奥行き)
        0.2,                     // 壁の厚み
        Vec3::new(0.0, y/2.0, 0.0),              // 中心位置
        true,                    // 上面あり（falseで“上が開いた箱”）
    );
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 0.1, 1.0))),
        Transform::from_xyz(0.0, -0.1, 0.0),
        RigidBody::Static,
        Collider::cuboid(1.0, 0.1, 1.0),
    ));
}

fn spawn_backrooms(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("room1.glb"));
    commands.spawn((
        SceneRoot(scene),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.3, 2.0, 1.3)),
        SteamAudioMaterial::default(), // 音響ジオメトリ
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        Friction::new(1.0),
        Restitution::new(RESTITUTION),
    ));
}

fn spawn_hollow_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    outer_size: Vec3,
    thickness: f32,
    center: Vec3,
    with_ceiling: bool,
) {
    let w = outer_size.x;
    let h = outer_size.y;
    let d = outer_size.z;

    // 厚みがデカすぎると破綻するので軽くガード
    let t = thickness.min(w * 0.49).min(h * 0.49).min(d * 0.49);

    let mat = materials.add(Color::srgb(0.75, 0.73, 0.70));

    let mut wall = |sx: f32, sy: f32, sz: f32, offset: Vec3| {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(sx, sy, sz))),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(center + offset),
            SteamAudioMaterial::default(), // 音響ジオメトリ
            RigidBody::Static,
            Collider::cuboid(sx, sy, sz),
            Friction::new(1.0),
            Restitution::new(RESTITUTION),
        ));
    };

    // 床・天井
    wall(w, t, d, Vec3::new(0.0, -h * 0.5 + t * 0.5, 0.0));
    if with_ceiling {
        wall(w, t, d, Vec3::new(0.0,  h * 0.5 - t * 0.5, 0.0));
    }

    // 左右（内側面が見えるように高さは少し引く）
    wall(t, h - 2.0 * t, d, Vec3::new(-w * 0.5 + t * 0.5, 0.0, 0.0));
    wall(t, h - 2.0 * t, d, Vec3::new( w * 0.5 - t * 0.5, 0.0, 0.0));

    // 前後（幅・高さは少し引く）
    wall(w - 2.0 * t, h - 2.0 * t, t, Vec3::new(0.0, 0.0, -d * 0.5 + t * 0.5));
    wall(w - 2.0 * t, h - 2.0 * t, t, Vec3::new(0.0, 0.0,  d * 0.5 - t * 0.5));
}