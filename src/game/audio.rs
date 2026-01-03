use bevy::prelude::*;
use bevy_seedling::{
    prelude::{SamplePlayer, Volume, VolumeNode, sample_effects},
    sample::{AudioSample, PlaybackSettings as SeedPlaybackSettings},
};
use bevy_steam_audio::prelude::*;
use avian3d::prelude::*;

use crate::game::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sfx)
           .add_systems(Update, play_bounce_sfx);
    }
}

#[derive(Resource)]
struct Sfx {
    bounce: Handle<AudioSample>,
}


fn play_bounce_sfx(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<Sfx>,

    // ✅ 大量の衝突は MessageReader 推奨
    mut collision_reader: MessageReader<CollisionStart>,

    // ✅ インパルス等の接触データを引く
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
    });
}