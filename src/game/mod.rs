//! Main game plugin.

use std::f32::consts::TAU;
use std::fs::File;
use std::io::{BufWriter, Write};

use avian2d::prelude::*;
use bevy::audio::Volume;
use bevy::prelude::*;

use crate::body::BodyPlugin;
use crate::enemy::EnemyPlugin;
use crate::items::ItemsPlugin;
use crate::physics::PhysicsGameplayPlugin;
use crate::player::PlayerPlugin;
use crate::survival::SurvivalPlugin;
use crate::ui::HudPlugin;
use crate::world::CavePlugin;

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
            HudPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.045, 0.04, 0.045)))
        .insert_resource(Gravity(avian2d::math::Vector::NEG_Y * 1000.0))
        .add_systems(Startup, spawn_music);
    }
}

/// Build a small original soundtrack at startup instead of relying on the old
/// static drone. This is intentionally simple synthesis, but it has an actual
/// harmonic bed, pulse, sparse melody and metallic cave accents.
fn spawn_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    let generated_path = "assets/audio/generated_score.wav";
    let asset_path = if generate_score(generated_path).is_ok() {
        "audio/generated_score.wav"
    } else {
        // Read-only install / unusual launch directory: keep a safe fallback.
        "audio/ambience.wav"
    };

    commands.spawn((
        AudioPlayer::new(asset_server.load(asset_path)),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.42)),
    ));
}

fn generate_score(path: &str) -> std::io::Result<()> {
    const SAMPLE_RATE: u32 = 16_000;
    const SECONDS: f32 = 24.0;
    const CHANNELS: u16 = 1;
    const BITS: u16 = 16;

    let sample_count = (SAMPLE_RATE as f32 * SECONDS) as u32;
    let bytes_per_sample = (BITS / 8) as u32;
    let data_size = sample_count * CHANNELS as u32 * bytes_per_sample;

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // PCM WAV header.
    writer.write_all(b"RIFF")?;
    writer.write_all(&(36 + data_size).to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?;
    writer.write_all(&1u16.to_le_bytes())?; // PCM
    writer.write_all(&CHANNELS.to_le_bytes())?;
    writer.write_all(&SAMPLE_RATE.to_le_bytes())?;
    let byte_rate = SAMPLE_RATE * CHANNELS as u32 * bytes_per_sample;
    writer.write_all(&byte_rate.to_le_bytes())?;
    let block_align = CHANNELS * (BITS / 8);
    writer.write_all(&block_align.to_le_bytes())?;
    writer.write_all(&BITS.to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;

    let motif = [293.66_f32, 349.23, 261.63, 220.0, 392.0, 349.23, 293.66, 261.63];
    let mut noise_state: u32 = 0xA53C_1F27;

    for index in 0..sample_count {
        let t = index as f32 / SAMPLE_RATE as f32;

        // Evolving D-minor-ish cave bed.
        let mut sample = 0.0_f32;
        for (freq, amp, phase, lfo) in [
            (73.42_f32, 0.19_f32, 0.0_f32, 0.031_f32),
            (110.0, 0.10, 0.6, 0.027),
            (146.83, 0.07, 1.2, 0.021),
            (174.61, 0.045, 2.0, 0.018),
        ] {
            let motion = 0.64 + 0.36 * (TAU * lfo * t + phase).sin();
            sample += amp * motion * (TAU * freq * t + 0.18 * (TAU * 0.014 * t + phase).sin()).sin();
        }

        // Low heartbeat-like pulse every three seconds.
        let pulse_t = t % 3.0;
        if pulse_t < 1.1 {
            let env = (-4.0 * pulse_t).exp();
            sample += env
                * (0.18 * (TAU * 48.0 * pulse_t).sin()
                    + 0.065 * (TAU * 72.0 * pulse_t).sin());
        }

        // A sparse repeating melodic phrase; enough pitch movement to read as
        // music rather than an unchanging ambience tone.
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

        // Metallic tick on the off-beat. LCG noise keeps this deterministic.
        noise_state = noise_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let noise = ((noise_state >> 8) as f32 / 16_777_215.0) * 2.0 - 1.0;
        let metal_t = (t + 0.75) % 6.0;
        if metal_t < 0.32 {
            sample += noise * 0.045 * (-12.0 * metal_t).exp();
        }

        // Mid-loop tension swell.
        let centre = 12.0;
        let distance = (t - centre) / 3.2;
        let swell = (-0.5 * distance * distance).exp();
        sample += 0.032 * swell * (TAU * 233.08 * t).sin();

        // Gentle fade at the loop edges avoids clicks and creates a brief
        // breathing space each cycle.
        let edge = t.min(SECONDS - t).clamp(0.0, 0.45) / 0.45;
        sample *= edge;

        let sample = (sample * 1.55).tanh().clamp(-0.95, 0.95);
        let pcm = (sample * i16::MAX as f32) as i16;
        writer.write_all(&pcm.to_le_bytes())?;
    }

    writer.flush()?;
    Ok(())
}
