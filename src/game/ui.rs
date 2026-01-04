use bevy::prelude::*;

use crate::game::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FocusedInteractable>()
            .add_systems(Startup, setup_interact_ui)
            .add_systems(PostUpdate, (detect_focused_interactable, update_interact_ui.run_if(resource_exists::<FocusedInteractable>)).chain());
    }
}
fn setup_interact_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 22.0,
            ..default()
        },
        TextColor::WHITE,
        TextLayout::new_with_justify(Justify::Left),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(40),
            left: px(20),
            ..default()
        },
        InteractPromptText,
        Visibility::Hidden,
    ));
}

fn update_interact_ui(
    focused: Res<FocusedInteractable>,
    interact_q: Query<&Interactable>,
    mut text_q: Query<(&mut Text, &mut Visibility), With<InteractPromptText>>,
) {
    if let Ok(x) = text_q.single_mut() {

        let mut text = x.0;
        let mut vis = x.1;

        if let Some(target) = focused.0 {
            if let Ok(interactable) = interact_q.get(target) {
                text.0 = interactable.prompt.clone();
                *vis = Visibility::Visible;
                return;
            }
        }
        text.0.clear();
        *vis = Visibility::Hidden;
    }
}

const MAX_INTERACT_DIST: f32 = 10.0;
// 20度以内を「向いてる」とする
const MAX_ANGLE_DEG: f32 = 20.0;

fn detect_focused_interactable(
    cam_q: Query<&GlobalTransform, With<PlayerCamera>>,
    interact_q: Query<(Entity, &GlobalTransform), With<Interactable>>,
    mut focused: ResMut<FocusedInteractable>,
) {
    if let Ok(cam_tf) = cam_q.single() {

        let cam_pos = cam_tf.translation();
        let cam_forward = cam_tf.forward(); // 視線方向（-Zが前の座標系）

        let min_dot = (MAX_ANGLE_DEG.to_radians()).cos();

        let mut best: Option<(Entity, f32, f32)> = None; // (entity, dot, dist)

        for (e, tf) in &interact_q {
            let to = tf.translation() - cam_pos;
            let dist = to.length();
            if dist > MAX_INTERACT_DIST || dist <= f32::EPSILON {
                continue;
            }

            let dir = to / dist;
            let dot = cam_forward.dot(dir); // 1.0に近いほど真正面
            if dot < min_dot {
                continue;
            }

            // 優先度: まず dot が高い / 同程度なら近い方
            match best {
                None => best = Some((e, dot, dist)),
                Some((_be, bdot, bdist)) => {
                    if dot > bdot + 0.0001 || ((dot - bdot).abs() <= 0.0001 && dist < bdist) {
                        best = Some((e, dot, dist));
                    }
                }
            }
        }
        focused.0 = best.map(|(e, _, _)| e);
    }
}

