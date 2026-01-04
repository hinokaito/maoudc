pub mod prelude;
mod components;
mod player;
mod audio;
mod balls;
mod level;
mod vfx;
mod rat;
mod ui;
mod settings;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                bevy::DefaultPlugins,
                avian3d::prelude::PhysicsPlugins::default()
                    .build()
                    .disable::<avian3d::prelude::PhysicsInterpolationPlugin>(),
                
                // avian3d::prelude::PhysicsDebugPlugin::default(),

                bevy_tnua::prelude::TnuaControllerPlugin::new(bevy::prelude::FixedUpdate),
                bevy_tnua_avian3d::prelude::TnuaAvian3dPlugin::new(bevy::prelude::FixedUpdate),

                bevy_hanabi::prelude::HanabiPlugin,
                bevy_egui::EguiPlugin::default(),

                bevy_seedling::SeedlingPlugin::default(),
                bevy_steam_audio::prelude::SteamAudioPlugin::default(),
                bevy_steam_audio::scene::mesh_backend::Mesh3dSteamAudioScenePlugin::default(),

                bevy::dev_tools::fps_overlay::FpsOverlayPlugin {
                    config: bevy::dev_tools::fps_overlay::FpsOverlayConfig {
                        text_config: bevy::text::TextFont {
                            font_size: 10.0,
                            font: default(),
                            font_smoothing: bevy::text::FontSmoothing::default(),
                            ..default()
                        },
                        text_color: OverlayColor::GREEN,
                        refresh_interval: core::time::Duration::from_millis(100),
                        enabled: true,
                        frame_time_graph_config: bevy::dev_tools::fps_overlay::FrameTimeGraphConfig {
                            enabled: true,
                            min_fps: 10.0,
                            target_fps: 30.0,
                        },
                    },
                },
            ))
            .insert_resource(bevy::prelude::AmbientLight::NONE)
            .insert_resource(avian3d::prelude::SubstepCount(1))
            .insert_resource(avian3d::dynamics::solver::SolverConfig {
                restitution_iterations: 0,
                restitution_threshold: 2.0,
                ..default()
            })
            // 機能別プラグイン
            .add_plugins(level::LevelPlugin)
            .add_plugins((
                // balls::BallsPlugin,
                rat::RatPlugin,
                audio::AudioPlugin,
                // vfx::VfxPlugin,
                player::PlayerPlugin,
                ui::UiPlugin,
                settings::SettingsPlugin,
            ));
    }
}

struct OverlayColor;
impl OverlayColor {
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}
