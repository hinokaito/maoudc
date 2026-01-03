use bevy::prelude::*;
use bevy_seedling::{
    prelude::{SamplePlayer, Volume, VolumeNode, sample_effects},
    sample::{AudioSample, PlaybackSettings as SeedPlaybackSettings},
};
use bevy_steam_audio::prelude::*;
use avian3d::prelude::*;
use bevy_tnua::prelude::*; 
use bevy_tnua::TnuaRigidBodyTracker;

use crate::game::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sfx)
           .add_systems(Update, (
            play_bounce_sfx,
            play_footsteps
        ));
    }
}

#[derive(Resource)]
struct Sfx {
    bounce: Handle<AudioSample>,
    footsteps: Vec<Handle<AudioSample>>,
}

// プレイヤー足音：
// - 接地中（!is_airborne）
// - 一定「距離」ごとに鳴らす（FPSに依存しにくい）
fn play_footsteps(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<Sfx>,
    mut player_q: Query<
        (
            &GlobalTransform,
            &TnuaRigidBodyTracker,
            &TnuaController,
            &mut FootstepSfxState,
        ),
        With<Player>,
    >,
) {
    // 調整パラメータ
    const SPEED_MIN: f32 = 0.7; // これ未満は足音なし（微速・壁押し等をカット）
    const STEP_DIST_MIN: f32 = 4.90; // 低速時の「1歩あたり距離」
    const STEP_DIST_MAX: f32 = 6.45; // 高速時の「1歩あたり距離」
    const VOL_MIN: f32 = 0.04;
    const VOL_MAX: f32 = 0.16;

    let dt = time.delta_secs();

    for (tf, tracker, controller, mut st) in &mut player_q {
        // 接地判定（空中なら距離をリセット）
        let airborne = controller.is_airborne().unwrap_or(true);
        if airborne {
            st.distance_accum = 0.0;
            continue;
        }

        // 水平速度
        let v = tracker.velocity; // TnuaRigidBodyTracker に入ってる :contentReference[oaicite:1]{index=1}
        let horiz_speed = Vec3::new(v.x, 0.0, v.z).length();

        if horiz_speed < SPEED_MIN {
            st.distance_accum = 0.0;
            continue;
        }

        // 速度→0..1
        let speed01 = (horiz_speed / MOVE_SPEED).clamp(0.0, 1.0);

        // 速度に応じて「1歩の距離」を少し伸ばす（走るほど歩幅が伸びるイメージ）
        let step_dist = STEP_DIST_MIN + (STEP_DIST_MAX - STEP_DIST_MIN) * speed01;

        // 距離を加算して、閾値を超えたら鳴らす（1フレーム最大1回にして暴発防止）
        st.distance_accum += horiz_speed * dt;
        if st.distance_accum < step_dist {
            continue;
        }
        st.distance_accum -= step_dist;

        // 足音サンプル
        let sample = sfx.footsteps[0].clone();

        // 音量：速度で補間 + 少し揺らす
        let base_vol = VOL_MIN + (VOL_MAX - VOL_MIN) * speed01;
        let volume = (base_vol * (0.92 + 0.16 * rand::random::<f32>())).clamp(0.0, 1.0);

        // ピッチ：速度で少し上げる + 左右で微差 + ランダム微揺れ
        let lr = if st.left { -0.015 } else { 0.015 };
        st.left = !st.left;

        let pitch = (0.95 + 0.12 * speed01 + lr + 0.05 * (rand::random::<f32>() - 0.5))
            .clamp(0.85, 1.25);

        commands.spawn((
            Name::new("footstep_sfx"),
            Transform::from_translation(tf.translation()),
            GlobalTransform::default(),

            SteamAudioPool,

            SamplePlayer::new(sample),
            SeedPlaybackSettings::default().with_speed(pitch as f64),

            sample_effects![VolumeNode {
                volume: Volume::Linear(volume),
                ..Default::default()
            }],
        ));
    }
}

fn play_bounce_sfx(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<Sfx>,

    // 大量の衝突は MessageReader
    mut collision_reader: MessageReader<CollisionStart>,

    // インパルス等の接触データを引く
    collisions: Collisions,

    // ボール判定＆位置取得＆クールダウン
    is_ball: Query<(), With<Ball>>,
    mut ball_q: Query<(&GlobalTransform, &mut BounceSfxCooldown), With<Ball>>,
) {
    const IMPULSE_MIN: f32 = 0.8;      // 小さい接触音は切る
    const IMPULSE_MAX: f32 = 10.0;     // これ以上は最大音量扱い
    const COOLDOWN: f32 = 0.04;        // 40ms
    const MAX_PER_FRAME: usize = 8;    // 同時発音の上限（1000個対策）

    let now = time.elapsed_secs();
    let mut played = 0usize;

    for ev in collision_reader.read() {
        if played >= MAX_PER_FRAME {
            break;
        }

        // どっちがボール？
        let ball_entity = if is_ball.contains(ev.collider1) {
            ev.collider1
        } else if is_ball.contains(ev.collider2) {
            ev.collider2
        } else {
            continue;
        };

        let Ok((ball_tf, mut cd)) = ball_q.get_mut(ball_entity) else { continue; };
        if now - cd.last_played < COOLDOWN {
            continue;
        }

        // 接触ペアから「法線方向の総インパルス量」を取得
        let impulse = collisions
            .get(ev.collider1, ev.collider2)
            .map(|pair| pair.total_normal_impulse_magnitude())
            .unwrap_or(0.0);

        if impulse < IMPULSE_MIN {
            continue;
        }

        cd.last_played = now;

        // インパルス→音量（0..1）
        let t = ((impulse - IMPULSE_MIN) / (IMPULSE_MAX - IMPULSE_MIN)).clamp(0.0, 1.0);
        let volume = 0.15 + 0.85 * t; // 最低音量を少し残す
        let pitch = 0.95 + 0.1 * rand::random::<f32>(); // 少しだけ揺らすと自然

        commands.spawn((
            Name::new("bounce_sfx"),
            Transform::from_translation(ball_tf.translation()),
            GlobalTransform::default(),

            // Steam Audio の処理経路へ
            SteamAudioPool,

            // 音を鳴らす（Seedling）
            SamplePlayer::new(sfx.bounce.clone()),

            // ピッチ（= speed）と、鳴り終わったら despawn（デフォルト） :contentReference[oaicite:11]{index=11}
            SeedPlaybackSettings::default().with_speed(pitch as f64),

            // 音量は VolumeNode をエフェクトとして付けるのが seedling流 :contentReference[oaicite:12]{index=12}
            sample_effects![VolumeNode {
                volume: Volume::Linear(volume),
                ..Default::default()
            }],
        ));


        played += 1;
    }
}

fn load_sfx(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Sfx {
        bounce: assets.load("sfx/bounce.ogg"),
        footsteps: vec![assets.load("sfx/footstep1.ogg")],
    });
}