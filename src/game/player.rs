use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::color::palettes::css;

use bevy_steam_audio::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::prelude::*;
use avian3d::prelude::*;

use crate::game::settings::NeedsGraphicsApply;
use crate::game::settings::GraphicsMenuOpen;

// CAMERA
use bevy::{
    anti_alias::{
        smaa::{Smaa, SmaaPreset},
        fxaa::{Sensitivity, Fxaa},
        taa::TemporalAntiAliasing,
        contrast_adaptive_sharpening::ContrastAdaptiveSharpening,
    },
    core_pipeline::{
        tonemapping::Tonemapping,
    },
    post_process::{
        bloom::Bloom,
        effect_stack::ChromaticAberration,
        dof::{DepthOfField, DepthOfFieldMode},
    },
    render::{
        view::Hdr,
    },
    camera::{
        MainPassResolutionOverride,
        Exposure,
        PhysicalCameraParameters
    }
};

use crate::game::settings::GraphicsSettings;

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
    gfx: Res<GraphicsSettings>,
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
                PlayerCamera,
                IsDefaultUiCamera,
                Camera3d::default(), 
                SteamAudioListener, // 立体音響

                Projection::Perspective(PerspectiveProjection {
                    fov: gfx.fov_deg.to_radians(),
                    far: gfx.draw_distance,
                    ..default()
                }),

                gfx.msaa,
                NeedsGraphicsApply,
            ));
            p.spawn((
                Flashlight,
                Visibility::Hidden,
                SpotLight {
                    color: Color::srgb(1.0, 0.96, 0.90),
                    intensity: 2_0500_000.0,
                    range: 300.0,

                    outer_angle: 30.0_f32.to_radians(),
                    inner_angle: 5.0_f32.to_radians(),
                    radius: 0.04,

                    shadows_enabled: true,
                    shadow_map_near_z: 0.2,
                    
                    ..default()
                },
                Transform::from_xyz(0.12, -0.12, -2.),
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
    open: Res<GraphicsMenuOpen>,  
) {
    if open.0 {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
        return;
    }
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
    open: Res<GraphicsMenuOpen>,
) {
    if open.0 {
        return; // メニュー中は視点を回さない
    }

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

