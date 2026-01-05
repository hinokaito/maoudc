use bevy::prelude::*;
use bevy::{
    anti_alias::{
        smaa::{Smaa, SmaaPreset},
        fxaa::Fxaa,
        taa::TemporalAntiAliasing,
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
        Exposure,
        PhysicalCameraParameters
    }
};

use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_egui::EguiPrimaryContextPass;

use crate::game::prelude::*;

#[derive(Component)]
pub struct NeedsGraphicsApply;

#[derive(Resource, Clone)]
pub struct GraphicsSettings {
    pub hdr: bool,
    pub msaa: Msaa,
    pub exposure_mode: ExposureMode,
    pub custom_ev100: f32,
    pub custom_physical: PhysicalCameraParameters,
    pub tonemapping: Tonemapping,
    pub bloom: bool,
    pub dof: bool,
    pub chromatic_aberration: f32,
    pub distance_fog: bool,
    pub fov_deg: f32,
    pub draw_distance: f32,
    pub aa: AaMode,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AaMode {
    None,
    Fxaa,
    SmaaHigh,
    Taa,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            hdr: true,
            msaa: Msaa::Sample4,
            exposure_mode: ExposureMode::CustomPhysical,
            custom_ev100: Exposure::EV100_BLENDER,
            custom_physical: PhysicalCameraParameters {
                aperture_f_stops: 5.6,
                shutter_speed_s: 0.033,
                sensitivity_iso: 6400.0,
                sensor_height: 0.0088,
            },
            tonemapping: Tonemapping::TonyMcMapface,
            bloom: true,
            dof: false,
            chromatic_aberration: 0.03,
            distance_fog: false,
            fov_deg: 70.0,
            draw_distance: 10.0,
            aa: AaMode::None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExposureMode {
    Sunlight,
    Overcast,
    Indoor,
    Blender,
    CustomEv100,
    CustomPhysical,
}


pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphicsSettings>()
        .init_resource::<GraphicsMenuOpen>()
        .add_systems(Startup, init_cursor)
        .add_systems(Update, toggle_graphics_menu)
        .add_systems(EguiPrimaryContextPass, graphics_menu_ui)
        .add_systems(Update, apply_graphics_settings.after(graphics_menu_ui));
    }
}

pub fn apply_graphics_settings(
    settings: Res<GraphicsSettings>,
    mut commands: Commands,
    mut q_cam: Query<(Entity, &mut Projection), With<PlayerCamera>>,
) {
    if !settings.is_changed() {
        return;
    }

    let Ok((cam_e, mut projection)) = q_cam.single_mut() else { return };

    // FOV / 描画距離（far）
    if let Projection::Perspective(ref mut p) = *projection {
        p.fov = settings.fov_deg.to_radians();
        p.far = settings.draw_distance * 100.0;
    }

    let mut e = commands.entity(cam_e);

    // MSAA（最近のBevyはカメラに付けるComponent）:contentReference[oaicite:3]{index=3}
    e.insert(settings.msaa);

    // HDR
    if settings.hdr { e.insert(Hdr); } else { e.remove::<Hdr>(); }

    // Bloom / DoF は「付けると有効・外すと無効」にできる系
    if settings.bloom { e.insert(Bloom::NATURAL); } else { e.remove::<Bloom>(); }
    if settings.dof {
        e.insert(DepthOfField {
            mode: DepthOfFieldMode::Bokeh,
            focal_distance: 3.0,
            aperture_f_stops: 1.8,
            ..default()
        });
    } else {
        e.remove::<DepthOfField>();
    }

    // Chromatic Aberration（カメラに付けるだけで効く例が公式にある）:contentReference[oaicite:4]{index=4}
    if settings.chromatic_aberration <= 0.0 {
        e.remove::<ChromaticAberration>();
    } else {
        e.insert(ChromaticAberration {
            intensity: settings.chromatic_aberration,
            max_samples: 8,
            ..default()
        });
    }

    e.insert(settings.tonemapping);

    let exposure = match settings.exposure_mode {
        ExposureMode::Sunlight => Exposure::SUNLIGHT,
        ExposureMode::Overcast => Exposure::OVERCAST,
        ExposureMode::Indoor   => Exposure::INDOOR,
        ExposureMode::Blender  => Exposure::BLENDER,
        ExposureMode::CustomEv100 => Exposure { ev100: settings.custom_ev100 },
        ExposureMode::CustomPhysical => Exposure::from_physical_camera(settings.custom_physical),
    };
    e.insert(exposure);

    if settings.distance_fog {
        e.insert(DistanceFog {
            color: Color::srgb(0.7, 0.8, 0.9),
            falloff: FogFalloff::Exponential { density: 0.02 },
            ..default()
        });
    } else {
        e.remove::<DistanceFog>();
    }

    let mut msaa = settings.msaa;
    if matches!(settings.aa, AaMode::SmaaHigh) {
        msaa = Msaa::Off;
    }
    e.insert(msaa);

    // AAは切り替え前に全部removeしてから
    e.remove::<Fxaa>();
    e.remove::<Smaa>();
    e.remove::<TemporalAntiAliasing>();

    match settings.aa {
        AaMode::None => {}
        AaMode::Fxaa => { e.insert(Fxaa { enabled: true, ..default() }); }
        AaMode::SmaaHigh => { e.insert(Smaa { preset: SmaaPreset::High }); }
        AaMode::Taa => { e.insert(TemporalAntiAliasing::default()); }
    }
}

use bevy_egui::{egui, EguiContexts};

#[derive(Resource, Default)]
pub struct GraphicsMenuOpen(pub bool);

pub fn graphics_menu_ui(
    mut contexts: EguiContexts,
    mut settings: ResMut<GraphicsSettings>,
    open: Res<GraphicsMenuOpen>,
) {
    if !open.0 { return; }

    egui::Window::new("Graphics").show(contexts.ctx_mut().expect("139"), |ui| {
        ui.checkbox(&mut settings.hdr, "HDR");
        ui.checkbox(&mut settings.bloom, "Bloom");
        ui.checkbox(&mut settings.dof, "Depth Of Field");

        ui.add(egui::Slider::new(&mut settings.chromatic_aberration, 0.0..=0.1)
            .text("Chromatic Aberration"));

        ui.add(egui::Slider::new(&mut settings.fov_deg, 50.0..=110.0).text("FOV"));
        ui.add(egui::Slider::new(&mut settings.draw_distance, 1.0..=10.0).text("Draw Distance"));

        egui::ComboBox::from_label("AA")
            .selected_text(format!("{:?}", settings.aa))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.aa, AaMode::None, "None");
                ui.selectable_value(&mut settings.aa, AaMode::Fxaa, "FXAA");
                ui.selectable_value(&mut settings.aa, AaMode::SmaaHigh, "SMAA High");
                ui.selectable_value(&mut settings.aa, AaMode::Taa, "TAA");
            });

        ui.separator();
        ui.label("MSAA");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut settings.msaa, Msaa::Off, "Off");
            ui.selectable_value(&mut settings.msaa, Msaa::Sample2, "2x");
            ui.selectable_value(&mut settings.msaa, Msaa::Sample4, "4x");
            ui.selectable_value(&mut settings.msaa, Msaa::Sample8, "8x");
        });

        egui::ComboBox::from_label("Tonemapping")
            .selected_text(format!("{:?}", settings.tonemapping))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.tonemapping, Tonemapping::AcesFitted, "AcesFitted");
                ui.selectable_value(&mut settings.tonemapping, Tonemapping::AgX, "AgX");
                ui.selectable_value(&mut settings.tonemapping, Tonemapping::BlenderFilmic, "BlenderFilmic");
                ui.selectable_value(&mut settings.tonemapping, Tonemapping::None, "None");
                ui.selectable_value(&mut settings.tonemapping, Tonemapping::Reinhard, "Reinhard");
                ui.selectable_value(&mut settings.tonemapping, Tonemapping::ReinhardLuminance, "ReinhardLuminance");
                ui.selectable_value(
                    &mut settings.tonemapping,
                    Tonemapping::SomewhatBoringDisplayTransform,
                    "SomewhatBoringDisplayTransform",
                );
                ui.selectable_value(&mut settings.tonemapping, Tonemapping::TonyMcMapface, "TonyMcMapface");
            });

        egui::ComboBox::from_label("Exposure")
            .selected_text(format!("{:?}", settings.exposure_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.exposure_mode, ExposureMode::Sunlight, "Sunlight");
                ui.selectable_value(&mut settings.exposure_mode, ExposureMode::Overcast, "Overcast");
                ui.selectable_value(&mut settings.exposure_mode, ExposureMode::Indoor, "Indoor");
                ui.selectable_value(&mut settings.exposure_mode, ExposureMode::Blender, "Blender");
                ui.separator();
                ui.selectable_value(&mut settings.exposure_mode, ExposureMode::CustomEv100, "Custom (EV100)");
                ui.selectable_value(&mut settings.exposure_mode, ExposureMode::CustomPhysical, "Custom (Physical)");
            });

        match settings.exposure_mode {
            ExposureMode::CustomEv100 => {
                ui.add(
                    egui::Slider::new(&mut settings.custom_ev100, 0.0..=20.0)
                        .text("EV100")
                );
            }
            ExposureMode::CustomPhysical => {
                ui.add(egui::Slider::new(&mut settings.custom_physical.aperture_f_stops, 1.0..=22.0).text("Aperture (f)"));
                ui.add(
                    egui::Slider::new(&mut settings.custom_physical.shutter_speed_s, 1.0/8000.0..=1.0)
                        .logarithmic(true)
                        .text("Shutter (s)")
                );
                ui.add(
                    egui::Slider::new(&mut settings.custom_physical.sensitivity_iso, 50.0..=25600.0)
                        .logarithmic(true)
                        .text("ISO")
                );
                // sensor_height は固定運用なら表示しなくてOK
            }
            _ => {}
        }
    });
}


pub fn init_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
}

// Escで開閉
pub fn toggle_graphics_menu(
    key: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<GraphicsMenuOpen>,
    mut cursor: Single<&mut CursorOptions>,
) {
    if !key.just_pressed(KeyCode::KeyG) {
        return;
    }

    open.0 = !open.0;

    if open.0 {
        // メニュー開：カーソル表示＆ロック解除
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    } else {
        // メニュー閉：カーソル非表示＆ロック
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    }
}

