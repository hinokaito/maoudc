use bevy::prelude::*;

pub const EYE_HEIGHT: f32 = 1.6;
pub const MOVE_SPEED: f32 = 10.0;
pub const RESTITUTION: f32 = 1.0;

#[derive(Component)]
pub struct BounceSfxCooldown {
    pub last_played: f32,
}

#[derive(Component)]
pub struct Ball {
    pub speed: f32,
}

#[derive(Component)]
pub struct Flashlight;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerLookPivot;

#[derive(Component, Deref, DerefMut)]
pub struct CameraSensitivity(pub Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002))
    }
}

/// 足音の「距離」ベース再生用の状態
#[derive(Component)]
pub struct FootstepSfxState {
    /// 今のフレームまでに歩いた距離（水平移動ぶん）
    pub distance_accum: f32,
    /// 左右の足を交互にする用（音の揺らぎに使う）
    pub left: bool,
}

impl Default for FootstepSfxState {
    fn default() -> Self {
        Self {
            distance_accum: 0.0,
            left: false,
        }
    }
}