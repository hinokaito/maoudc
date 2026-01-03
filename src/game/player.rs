use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy::input::mouse::AccumulatedMouseMotion;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::{ bloom::Bloom, effect_stack::ChromaticAberration };
use bevy::render::view::Hdr;
use bevy::color::palettes::css;

use bevy_steam_audio::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::prelude::*;
use avian3d::prelude::*;

use crate::game::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
           .add_systems(Update, (grab_mouse, mouse_look, toggle_flashlight))
           .add_systems(FixedUpdate, apply_controls.in_set(TnuaUserControlsSystems));
    }
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 見た目（邪魔なら Visibility::Hidden にする）
    let body_mesh = meshes.add(Capsule3d {
        radius: 0.5,
        half_length: 0.5,
    });
    let body_mat = materials.add(Color::from(css::DARK_CYAN));

commands.spawn((
    Player,
    FootstepSfxState::default(),
    Transform::from_xyz(0.0, 5.0, 0.0),
    RigidBody::Dynamic,
    Collider::capsule(0.5, 1.0),
    TnuaController::default(),
    TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
    LockedAxes::ROTATION_LOCKED,
))
.with_children(|p| {
    // 見た目だけ隠す
    p.spawn((
        Mesh3d(body_mesh),
        MeshMaterial3d(body_mat),
        Visibility::Hidden,
    ));

    // 視点ピボット（Hiddenにしない）
    p.spawn((
        PlayerLookPivot,
        CameraSensitivity::default(),
        Transform::from_xyz(0.0, EYE_HEIGHT, 0.0),
    ))
    .with_children(|p| {
        p.spawn((
            Camera3d::default(), 

            SteamAudioListener, // 立体音響

            Projection::Perspective(PerspectiveProjection {
                fov: 67.0_f32.to_radians(),  
                near: 0.05, 
                far: 800.0,
                ..default() 
            }),

            Hdr, 
            Tonemapping::TonyMcMapface, 
            Bloom::NATURAL,

            ChromaticAberration {
                intensity: 0.02,
                max_samples: 16,
                ..default()
            },

            // DistanceFog {
            //     color: Color::srgb_u8(43, 44, 47),
            //     falloff: FogFalloff::Linear { start: 2.0, end: 18.0 },
            //     ..default()
            // },
            
            ));
        p.spawn((
            Flashlight,
            SpotLight {
                color: Color::srgb(1.0, 0.96, 0.90),
                intensity: 35000000.0,
                range: 1800.0,

                outer_angle: 20.0_f32.to_radians(),
                inner_angle: 14.0_f32.to_radians(),
                radius: 0.04,

                shadows_enabled: true,
                shadow_map_near_z: 0.2,
                
                ..default()
            },
            Transform::from_xyz(0.12, -0.12, -0.35),
        ));
        p.spawn((
            Flashlight,
            PointLight {
                color: Color::srgb(1.0, 0.96, 0.90),
                intensity: 35000.0,
                range: 1800.0,
                shadows_enabled: true,
                shadow_map_near_z: 0.2,
                
                ..default()
            },
        ));
    });
});
}

fn toggle_flashlight(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut Visibility, With<Flashlight>>,
) {
    if keys.just_pressed(KeyCode::KeyF) {
        for mut v in &mut q {
            *v = match *v {
                Visibility::Visible => Visibility::Hidden,
                _ => Visibility::Visible,
            };
        }
    }
}

/// 左クリックでロック / Escで解除
fn grab_mouse(
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    }
    if key.just_pressed(KeyCode::Escape) {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    }
}

fn mouse_look(
    motion: Res<AccumulatedMouseMotion>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    pivot: Single<(&mut Transform, &CameraSensitivity), With<PlayerLookPivot>>,
) {
    if cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    let delta = motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    let (mut tf, sens) = pivot.into_inner();

    let delta_yaw = -delta.x * sens.x;
    let delta_pitch = -delta.y * sens.y;

    let (yaw, pitch, roll) = tf.rotation.to_euler(EulerRot::YXZ);
    let yaw = yaw + delta_yaw;

    const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
    let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

    tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    look: Single<&Transform, With<PlayerLookPivot>>,
    mut controller_q: Query<&mut TnuaController, With<Player>>,
) {
    let Ok(mut controller) = controller_q.single_mut() else { return; };

    let mut local = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        local -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        local += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        local -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        local += Vec3::X;
    }

    // pitchの影響を受けないよう yaw だけ抽出して移動方向を作る
    let (yaw, _pitch, _roll) = look.rotation.to_euler(EulerRot::YXZ);
    let yaw_rot = Quat::from_rotation_y(yaw);

    let world_dir = (yaw_rot * local).with_y(0.0).normalize_or_zero();

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: world_dir * MOVE_SPEED,
        float_height: 1.5,
        ..Default::default()
    });

    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            height: 4.0,
            ..Default::default()
        });
    }
}