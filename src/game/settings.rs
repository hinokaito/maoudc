use bevy::prelude::*;
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

use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use crate::game::prelude::*;

#[derive(Component)]
pub struct NeedsGraphicsApply;

#[derive(Resource, Clone)]
pub struct GraphicsSettings {
    pub msaa: Msaa,
    pub hdr: bool,
    pub bloom: bool,
    pub dof: bool,
    pub chromatic_aberration: f32,
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
            msaa: Msaa::Sample4,
            hdr: true,
            bloom: true,
            dof: true,
            chromatic_aberration: 0.03,
            fov_deg: 70.0,
            draw_distance: 300.0,
            aa: AaMode::None,
        }
    }
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
        p.far = settings.draw_distance;
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

    // AAは「切り替え前に全部removeしてから、必要なのだけinsert」が事故りにくい
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

        ui.add(egui::Slider::new(&mut settings.fov_deg, 50.0..=100.0).text("FOV"));
        ui.add(egui::Slider::new(&mut settings.draw_distance, 50.0..=1000.0).text("Draw Distance"));

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

