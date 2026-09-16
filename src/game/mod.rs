//! Main game plugin and lightweight original audio synthesis.

use std::f32::consts::TAU;
use std::fs::File;
use std::io::{BufWriter, Write};

use avian2d::prelude::*;
use bevy::audio::Volume;
use bevy::prelude::*;

use crate::body::{Body, BodyPlugin, DamageCause, LastDamageCause};
use crate::enemy::EnemyPlugin;
use crate::items::{ItemsPlugin, LastEvent};
use crate::physics::PhysicsGameplayPlugin;
use crate::player::{Player, PlayerPlugin};
use crate::survival::SurvivalPlugin;
use crate::ui::HudPlugin;
use crate::world::{CavePlugin, RecordsPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Descent: Null".into(),
                        resolution: (1280, 800).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            PhysicsPlugins::default().with_length_unit(20.0),
            PhysicsGameplayPlugin,
            BodyPlugin,
            SurvivalPlugin,
            ItemsPlugin,
            EnemyPlugin,
            PlayerPlugin,
            CavePlugin,
            RecordsPlugin,
            HudPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.045, 0.04, 0.045)))
        .insert_resource(Gravity(avian2d::math::Vector::NEG_Y * 1000.0))
        .add_systems(Startup, spawn_music)
        .add_systems(Update, play_death_sound);
    }
}

fn spawn_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    let generated_path = "assets/audio/generated_score.wav";
    let asset_path = if generate_score(generated_path).is_ok() {
        "audio/generated_score.wav"
    } else {
        "audio/ambience.wav"
    };

    let _ = generate_death_sfx("assets/audio/generated_death_fall.wav", DeathTone::Fall);
    let _ = generate_death_sfx("assets/audio/generated_death_trap.wav", DeathTone::Trap);
    let _ = generate_death_sfx("assets/audio/generated_death_enemy.wav", DeathTone::Enemy);
    let _ = generate_death_sfx(
        "assets/audio/generated_death_dehydration.wav",
        DeathTone::Dehydration,
    );

    commands.spawn((
        AudioPlayer::new(asset_server.load(asset_path)),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.42)),
    ));
}

fn play_death_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player: Query<&Body, With<Player>>,
    cause: Res<LastDamageCause>,
    last_event: Res<LastEvent>,
    mut was_dead: Local<bool>,
) {
    let dead = player.single().map(|body| body.0.is_dead()).unwrap_or(false);
    if dead && !*was_dead {
        let event_upper = last_event.text.to_uppercase();
        let asset = if event_upper.contains("TRAP") || event_upper.contains("SPIKE") {
            "audio/generated_death_trap.wav"
        } else {
            match cause.0 {
                DamageCause::Enemy | DamageCause::Poison => "audio/generated_death_enemy.wav",
                DamageCause::Trap => "audio/generated_death_trap.wav",
                DamageCause::Starvation | DamageCause::Dehydration => {
                    "audio/generated_death_dehydration.wav"
                }
                DamageCause::Fall | DamageCause::Unknown => "audio/generated_death_fall.wav",
            }
        };
        commands.spawn((
            AudioPlayer::new(asset_server.load(asset)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.72)),
        ));
    }
    *was_dead = dead;
}

fn write_wav_header(
    writer: &mut BufWriter<File>,
    sample_rate: u32,
    sample_count: u32,
) -> std::io::Result<()> {
    const CHANNELS: u16 = 1;
    const BITS: u16 = 16;
    let bytes_per_sample = (BITS / 8) as u32;
    let data_size = sample_count * CHANNELS as u32 * bytes_per_sample;

    writer.write_all(b"RIFF")?;
    writer.write_all(&(36 + data_size).to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?;
    writer.write_all(&1u16.to_le_bytes())?;
    writer.write_all(&CHANNELS.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    let byte_rate = sample_rate * CHANNELS as u32 * bytes_per_sample;
    writer.write_all(&byte_rate.to_le_bytes())?;
    let block_align = CHANNELS * (BITS / 8);
    writer.write_all(&block_align.to_le_bytes())?;
    writer.write_all(&BITS.to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;
    Ok(())
}

#[derive(Clone, Copy)]
enum DeathTone {
    Fall,
    Trap,
    Enemy,
    Dehydration,
}

fn generate_death_sfx(path: &str, tone: DeathTone) -> std::io::Result<()> {
    const SAMPLE_RATE: u32 = 16_000;
    const SECONDS: f32 = 0.95;
    let sample_count = (SAMPLE_RATE as f32 * SECONDS) as u32;
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    write_wav_header(&mut writer, SAMPLE_RATE, sample_count)?;

    let (start_pitch, end_pitch, noise_amp, second_ratio) = match tone {
        DeathTone::Fall => (185.0, 58.0, 0.10, 0.48),
        DeathTone::Trap => (300.0, 115.0, 0.22, 1.72),
        DeathTone::Enemy => (135.0, 78.0, 0.16, 0.67),
        DeathTone::Dehydration => (112.0, 88.0, 0.06, 1.03),
    };

    let mut noise_state: u32 = match tone {
        DeathTone::Fall => 0x71D2_03A5,
        DeathTone::Trap => 0x145A_991B,
        DeathTone::Enemy => 0x8BE3_201D,
        DeathTone::Dehydration => 0x41A9_77C3,
    };

    for index in 0..sample_count {
        let t = index as f32 / SAMPLE_RATE as f32;
        let env = (-3.8 * t).exp();
        let progress = (t / SECONDS).clamp(0.0, 1.0);
        let pitch = start_pitch + (end_pitch - start_pitch) * progress;
        let mut sample = 0.42 * env * (TAU * pitch * t).sin();
        sample += 0.18 * env * (TAU * pitch * second_ratio * t).sin();

        noise_state = noise_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let noise = ((noise_state >> 8) as f32 / 16_777_215.0) * 2.0 - 1.0;
        sample += noise * noise_amp * (-8.5 * t).exp();

        let pcm = (sample.tanh().clamp(-0.95, 0.95) * i16::MAX as f32) as i16;
        writer.write_all(&pcm.to_le_bytes())?;
    }
    writer.flush()?;
    Ok(())
}

fn generate_score(path: &str) -> std::io::Result<()> {
    const SAMPLE_RATE: u32 = 16_000;
    const SECONDS: f32 = 24.0;
    let sample_count = (SAMPLE_RATE as f32 * SECONDS) as u32;

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    write_wav_header(&mut writer, SAMPLE_RATE, sample_count)?;

    let motif = [293.66_f32, 349.23, 261.63, 220.0, 392.0, 349.23, 293.66, 261.63];
    let mut noise_state: u32 = 0xA53C_1F27;

    for index in 0..sample_count {
        let t = index as f32 / SAMPLE_RATE as f32;
        let mut sample = 0.0_f32;

        for (freq, amp, phase, lfo) in [
            (73.42_f32, 0.19_f32, 0.0_f32, 0.031_f32),
            (110.0, 0.10, 0.6, 0.027),
            (146.83, 0.07, 1.2, 0.021),
            (174.61, 0.045, 2.0, 0.018),
        ] {
            let motion = 0.64 + 0.36 * (TAU * lfo * t + phase).sin();
            sample += amp
                * motion
                * (TAU * freq * t + 0.18 * (TAU * 0.014 * t + phase).sin()).sin();
        }

        let pulse_t = t % 3.0;
        if pulse_t < 1.1 {
            let env = (-4.0 * pulse_t).exp();
            sample += env
                * (0.18 * (TAU * 48.0 * pulse_t).sin()
                    + 0.065 * (TAU * 72.0 * pulse_t).sin());
        }

        let note_slot = ((t / 3.0).floor() as usize) % motif.len();
        let note_t = t % 3.0;
        if note_t < 1.35 {
            let freq = motif[note_slot];
            let env = (-3.6 * note_t).exp();
            sample += env
                * (0.085 * (TAU * freq * note_t).sin()
                    + 0.032 * (TAU * freq * 2.01 * note_t).sin()
                    + 0.018 * (TAU * freq * 0.5 * note_t).sin());
        }

        noise_state = noise_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let noise = ((noise_state >> 8) as f32 / 16_777_215.0) * 2.0 - 1.0;
        let metal_t = (t + 0.75) % 6.0;
        if metal_t < 0.32 {
            sample += noise * 0.045 * (-12.0 * metal_t).exp();
        }

        let distance = (t - 12.0) / 3.2;
        let swell = (-0.5 * distance * distance).exp();
        sample += 0.032 * swell * (TAU * 233.08 * t).sin();

        let edge = t.min(SECONDS - t).clamp(0.0, 0.45) / 0.45;
        sample *= edge;

        let pcm = ((sample * 1.55).tanh().clamp(-0.95, 0.95) * i16::MAX as f32) as i16;
        writer.write_all(&pcm.to_le_bytes())?;
    }

    writer.flush()?;
    Ok(())
}
