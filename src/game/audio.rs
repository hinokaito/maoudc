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
        app.add_systems(Startup, (load_sfx, setup_rat_audio_settings))
           .add_systems(Update, (
            play_bounce_sfx,
            play_footsteps,
            play_rat_sfx, 
        ));
    }
}

#[derive(Resource)]
pub struct Sfx {
    pub bounce: Handle<AudioSample>,
    pub footsteps: Vec<Handle<AudioSample>>,
    pub rat_footsteps: Vec<Handle<AudioSample>>,
}

fn load_sfx(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Sfx {
        bounce: assets.load("sfx/bounce.ogg"),
        footsteps: vec![assets.load("sfx/footstep1.ogg")],
        rat_footsteps: vec![
            assets.load("sfx/rat/1.ogg"),
            assets.load("sfx/rat/2.ogg"),
            assets.load("sfx/rat/3.ogg"),
            assets.load("sfx/rat/4.ogg"),
            assets.load("sfx/rat/5.ogg"),
        ],
    });
}

fn setup_rat_audio_settings(mut commands: Commands) {
    commands.insert_resource(RatAudioSettings {
        hear_dist: 38.0,        // 聞こえる距離
        stop_dist: 42.0,        // これ以上離れたら止める
        max_active: 12,         // 同時に鳴らす数
        _update_interval: 0.20,  // 更新頻度
    });
    commands.insert_resource(RatAudioTimer(Timer::from_seconds(
        0.15,
        TimerMode::Repeating,
    )));
}


// ネズミの足音 ======================================================
fn play_rat_sfx(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<Sfx>,
    settings: Res<RatAudioSettings>,
    mut timer: ResMut<RatAudioTimer>,

    // リスナー（=カメラ）位置
    listener_q: Query<&GlobalTransform, With<SteamAudioListener>>,

    // ラット
    mut rats: Query<(Entity, &GlobalTransform, &mut RatFootstepAudio), With<Rat>>,
) {
    if sfx.rat_footsteps.is_empty() {
        return;
    }

    // 更新間隔で間引き
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let listener_tf = match listener_q.single() {
        Ok(t) => t,
        Err(_) => return,
    };
    let listener_pos = listener_tf.translation();

    let hear2 = settings.hear_dist * settings.hear_dist;
    let stop2 = settings.stop_dist * settings.stop_dist;

    // 近いラット候補を集める
    let mut candidates: Vec<(f32, Entity)> = Vec::new();

    for (e, tf, a) in rats.iter() {
        let d2 = tf.translation().distance_squared(listener_pos);

        // すでに鳴ってる個体は stop_dist まで粘らせる（ヒステリシス）
        let in_range = if a.is_active { d2 <= stop2 } else { d2 <= hear2 };

        if in_range {
            candidates.push((d2, e));
        }
    }

    // 近い順に並べて、上位 max_active のみ有効化
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut should_be_active = std::collections::HashSet::<Entity>::new();
    for (_, e) in candidates.into_iter().take(settings.max_active) {
        should_be_active.insert(e);
    }

    // 各ラットを on/off
    for (e, _tf, mut a) in rats.iter_mut() {
        let want = should_be_active.contains(&e);

        if want && !a.is_active {
            // --- ON: 子音源spawn
            let sample = sfx.rat_footsteps[a.sample_index.min(sfx.rat_footsteps.len() - 1)].clone();

            let child = commands
                .spawn((
                    Name::new("rat_footstep_loop"),
                    Transform::default(),
                    GlobalTransform::default(),
                    SteamAudioPool,
                    SamplePlayer::new(sample).looping(),
                    // ループなので消えない想定。念のため preserve。
                    SeedPlaybackSettings::default().preserve(),
                    sample_effects![VolumeNode {
                        volume: Volume::Linear(0.02),
                        ..Default::default()
                    }],
                ))
                .id();

            commands.entity(e).add_child(child);
            a.child_emitter = Some(child);
            a.is_active = true;
        } else if !want && a.is_active {
            // --- OFF: 子音源despawn
            if let Some(child) = a.child_emitter.take() {
                commands.entity(child).despawn();
            }
            a.is_active = false;
        }
    }
}


// プレイヤー足音 =====================================================
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
        // let base_vol = VOL_MIN + (VOL_MAX - VOL_MIN) * speed01;
        // let volume = (base_vol * (0.92 + 0.16 * rand::random::<f32>())).clamp(0.0, 1.0);

        let volume = VOL_MAX;

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

// ボールのバウンド音 ===================================================
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
