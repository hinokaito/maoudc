use bevy::prelude::*;
use bevy_seedling::prelude::PoolLabel;

// player.rs
pub const EYE_HEIGHT: f32 = 1.6;
pub const MOVE_SPEED: f32 = 11.0;
pub const RESTITUTION: f32 = 1.0;
pub const DRAW_DISTANCE: f32 = 1000.0;

// audio.rs
pub const SPEED_MIN: f32 = 0.7;      // これ未満は足音なし（微速・壁押し等をカット）
pub const STEP_DIST_MIN: f32 = 4.90; // 低速時の「1歩あたり距離」
pub const STEP_DIST_MAX: f32 = 6.45; // 高速時の「1歩あたり距離」
pub const _VOL_MIN: f32 = 0.001;     // 最小音量
pub const VOL_MAX: f32 = 0.0001;     // 最大音量
pub const IMPULSE_MIN: f32 = 0.8;    // 小さい接触音は切る
pub const IMPULSE_MAX: f32 = 10.0;   // これ以上は最大音量扱い
pub const COOLDOWN: f32 = 0.04;      // 40ms
pub const MAX_PER_FRAME: usize = 8;  // 同時発音の上限（1000個対策）

// ui.rs
pub const MAX_INTERACT_DIST: f32 = 10.0; // 反応範囲
pub const MAX_ANGLE_DEG: f32 = 20.0;     // 見てる判定の許容角度


#[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomSfxPool;

#[derive(Component)]
pub struct BounceSfxCooldown {
    pub last_played: f32,
}

#[derive(Component)]
pub struct Ball {
    pub _speed: f32,
}

#[derive(Component)]
pub struct Flashlight;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct InteractPromptText;


#[derive(Component)]
pub struct Rat;

#[derive(Component)]
pub struct RatPatrol {
    pub dir: Vec3,          // 現在の進行方向（XZ平面）
    pub speed: f32,         // 移動速度
    pub turn_lerp: f32,     // 方向転換の追従（大きいほどキビキビ）
    pub jitter: f32,        // ランダム旋回の強さ
    pub look_ahead: f32,    // 前方チェック距離
    pub avoid_strength: f32,// 回避の強さ
}

#[derive(Component)]
pub struct RatFootstepAudio {
    pub sample_index: usize,      // どのoggを鳴らすか（ラットごとに固定）
    pub child_emitter: Option<Entity>, // 生成した子音源
    pub is_active: bool,          // 今有効か（ヒステリシス用）
}

#[derive(Resource)]
pub struct RatAudioSettings {
    pub hear_dist: f32,        // ここまで鳴らす
    pub stop_dist: f32,        // ここを超えたら止める（hear_distより少し大きく）
    pub max_active: usize,     // 同時発音数上限
    pub _update_interval: f32,  // 何秒ごとに判定するか
}

#[derive(Resource)]
pub struct RatAudioTimer(pub Timer);

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

#[derive(Component)]
pub struct Interactable {
    pub prompt: String,
}

#[derive(Resource, Default)]
pub struct FocusedInteractable(pub Option<Entity>);

#[derive(Event)]
pub struct _InteractEvent {
    pub actor: Entity,
    pub target: Entity,
}

