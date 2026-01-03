use bevy::prelude::*;
use bevy_hanabi::prelude::{
    AccelModifier, Attribute, ColorOverLifetimeModifier, EffectAsset, ExprWriter, Gradient as HanabiGradient,
    LinearDragModifier, Module, OrientMode, OrientModifier, ParticleEffect, SetAttributeModifier,
    SetPositionCircleModifier, SetPositionSphereModifier, SetVelocitySphereModifier,
    ShapeDimension, SizeOverLifetimeModifier, SpawnerSettings,
};
use bevy_hanabi::{ColorBlendMode, ColorBlendMask};

/// VFX の EffectAsset ハンドル集
#[derive(Resource, Clone)]
pub struct VfxHandles {
    pub test: Handle<EffectAsset>,
    pub flame_core: Handle<EffectAsset>,
}

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_vfx, spawn_default_vfx).chain());
    }
}

// -------------------------------------------------
// Setup / Spawn
// -------------------------------------------------

fn setup_vfx(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let test = effects.add(test_vfx());
    let flame_core = effects.add(fire_pillar());

    commands.insert_resource(VfxHandles { test, flame_core });
}

/// デフォルトで 1個ずつ出す（不要ならこの system を外す）
fn spawn_default_vfx(mut commands: Commands, vfx: Res<VfxHandles>) {
    commands.spawn((
        Name::new("vfx_test"),
        ParticleEffect::new(vfx.test.clone()),
        Transform::from_translation(Vec3::Y),
    ));

    commands.spawn((
        Name::new("flame_core"),
        ParticleEffect::new(vfx.flame_core.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));
}

// -------------------------------------------------
// Effect assets
// -------------------------------------------------

fn test_vfx() -> EffectAsset {
    let mut module = Module::default();

    // 1. グラデーション（赤→透明）
    let mut gradient = HanabiGradient::new();
    gradient.add_key(0.0, Vec4::new(1., 0., 0., 1.));
    gradient.add_key(1.0, Vec4::ZERO);

    // 2. 初期位置（球の表面）
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(2.),
        dimension: ShapeDimension::Surface,
    };

    // 3. 初速
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(6.),
    };

    // 4. 寿命
    let lifetime = module.lit(10.);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // 5. 重力っぽい加速
    let accel = module.lit(Vec3::new(0., -3., 0.));
    let update_accel = AccelModifier::new(accel);

    EffectAsset::new(
        32768,
        SpawnerSettings::rate(500.0.into()),
        module,
    )
    .with_name("EffectTest")
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .update(update_accel)
    .render(ColorOverLifetimeModifier {
        gradient,
        blend: ColorBlendMode::Overwrite,
        mask: ColorBlendMask::RGBA,
    })
}

fn fire_pillar() -> EffectAsset {
    // --- 色（HDR前提で強め）
    let mut color = HanabiGradient::new();
    color.add_key(0.0, Vec4::new(6.0, 6.0, 2.0, 1.0));
    color.add_key(0.2, Vec4::new(6.0, 2.5, 0.2, 1.0));
    color.add_key(0.7, Vec4::new(3.0, 0.2, 0.0, 0.6));
    color.add_key(1.0, Vec4::new(1.0, 0.0, 0.0, 0.0));

    // --- サイズ（少し膨らんで消える）
    let mut size = HanabiGradient::new();
    size.add_key(0.0, Vec3::splat(0.08));
    size.add_key(0.4, Vec3::splat(0.18));
    size.add_key(1.0, Vec3::splat(0.0));

    // Expressions
    let w = ExprWriter::new();

    // 発生位置：小さな円盤（地面）から
    let init_pos = SetPositionCircleModifier {
        center: w.lit(Vec3::ZERO).expr(),
        axis: w.lit(Vec3::Y).expr(),
        radius: w.lit(0.06).expr(),
        dimension: ShapeDimension::Volume,
    };

    // AGE=0
    let init_age = SetAttributeModifier::new(Attribute::AGE, w.lit(0.0).expr());

    // 寿命：0.4〜0.9 秒
    let init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, w.lit(0.4).uniform(w.lit(0.9)).expr());

    // 初速：上 + 横ブレ
    let vmin = w.lit(Vec3::new(-0.35, 1.2, -0.35));
    let vmax = w.lit(Vec3::new(0.35, 2.4, 0.35));
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vmin.uniform(vmax).expr());

    // 減速（まとまり感）
    let update_drag = LinearDragModifier::new(w.lit(1.6).expr());

    EffectAsset::new(
        4096,
        SpawnerSettings::rate(450.0.into()),
        w.finish(),
    )
    .with_name("flame_core")
    .init(init_pos)
    .init(init_age)
    .init(init_lifetime)
    .init(init_vel)
    .update(update_drag)
    .render(ColorOverLifetimeModifier::new(color))
    // どっちか1つでOK（最後に書いた方が効く感じになりがち）
    .render(OrientModifier::new(OrientMode::FaceCameraPosition))
    .render(SizeOverLifetimeModifier {
        gradient: size,
        screen_space_size: false,
    })
}
