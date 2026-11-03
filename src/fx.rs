//! Sound and screen juice. The sounds are synthesized here at boot, so the
//! game ships no third-party audio.

use std::cell::Cell;
use std::f32::consts::TAU;

use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};
use macroquad::prelude::*;

const RATE: u32 = 22050;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    /// Rubber stamp on paper.
    Stamp,
    /// Short sour buzz.
    Reject,
    /// Dot matrix printer.
    Printer,
    /// Counter bell.
    Ting,
    /// Modem handshake that never finishes.
    Dialup,
    /// Coins leaving your pocket.
    Coin,
}

const ALL: [Sfx; 6] = [
    Sfx::Stamp,
    Sfx::Reject,
    Sfx::Printer,
    Sfx::Ting,
    Sfx::Dialup,
    Sfx::Coin,
];

fn sine(f: f32, t: f32) -> f32 {
    (TAU * f * t).sin()
}

fn square(f: f32, t: f32) -> f32 {
    if (f * t).fract() < 0.5 {
        1.0
    } else {
        -1.0
    }
}

/// Mono samples in -1..1 for one effect. Deterministic.
pub fn synth(sfx: Sfx) -> Vec<f32> {
    let secs = match sfx {
        Sfx::Stamp => 0.25,
        Sfx::Reject => 0.35,
        Sfx::Printer => 0.7,
        Sfx::Ting => 0.7,
        Sfx::Dialup => 0.9,
        Sfx::Coin => 0.35,
    };
    let n = (secs * RATE as f32) as usize;
    let mut noise_state: u32 = 0x2545_f491;
    let mut noise = move || {
        noise_state ^= noise_state << 13;
        noise_state ^= noise_state >> 17;
        noise_state ^= noise_state << 5;
        noise_state as f32 / u32::MAX as f32 * 2.0 - 1.0
    };
    let mut low = 0.0;
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let s = match sfx {
                Sfx::Stamp => {
                    low += (noise() - low) * 0.15;
                    let thud = sine(60.0 + 40.0 * (-t * 30.0).exp(), t) * (-t * 18.0).exp();
                    thud + low * 2.5 * (-t * 60.0).exp()
                }
                Sfx::Reject => {
                    (square(110.0, t) + square(116.0, t)) * 0.3 * (1.0 - t / secs).max(0.0)
                }
                Sfx::Printer => {
                    let line = ((t * 4.0) as u32).is_multiple_of(2);
                    let tick = (t * 60.0).fract() < 0.3;
                    if line && tick {
                        noise() * 0.4 + sine(900.0, t) * 0.2
                    } else {
                        0.0
                    }
                }
                Sfx::Ting => (sine(1760.0, t) + 0.4 * sine(4400.0, t)) * (-t * 6.0).exp() * 0.6,
                Sfx::Dialup => {
                    let f = if ((t * 16.0) as u32).is_multiple_of(2) {
                        1200.0
                    } else {
                        2100.0
                    };
                    let hiss = if t > 0.6 { noise() * 0.3 } else { 0.0 };
                    sine(f, t) * 0.35 + hiss
                }
                Sfx::Coin => {
                    let f = if t < 0.06 { 1980.0 } else { 2640.0 };
                    sine(f, t) * (-t * 9.0).exp() * 0.5
                }
            };
            s.clamp(-1.0, 1.0)
        })
        .collect()
}

/// 16-bit mono PCM WAV bytes.
pub fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = samples.len() as u32 * 2;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&((s * i16::MAX as f32) as i16).to_le_bytes());
    }
    out
}

/// Sounds, mute, and screen shake. Shared by every scene through `Ctx`.
pub struct Fx {
    sounds: Vec<Sound>,
    muted: Cell<bool>,
    shake: Cell<f32>,
}

impl Fx {
    pub async fn load() -> Fx {
        let mut sounds = Vec::new();
        for sfx in ALL {
            let bytes = wav(&synth(sfx));
            sounds.push(load_sound_from_bytes(&bytes).await.expect("sfx"));
        }
        Fx {
            sounds,
            muted: Cell::new(false),
            shake: Cell::new(0.0),
        }
    }

    pub fn play(&self, sfx: Sfx) {
        if !self.muted.get() {
            play_sound_once(&self.sounds[sfx as usize]);
        }
    }

    pub fn toggle_mute(&self) {
        self.muted.set(!self.muted.get());
    }

    pub fn shake(&self, secs: f32) {
        self.shake.set(self.shake.get().max(secs));
    }

    /// Advance the shake and return this frame's offset in canvas pixels.
    pub fn tick(&self, dt: f32) -> Vec2 {
        let left = self.shake.get();
        if left <= 0.0 {
            return Vec2::ZERO;
        }
        self.shake.set(left - dt);
        let amp = (left * 20.0).min(4.0);
        vec2(
            rand::gen_range(-amp, amp).round(),
            rand::gen_range(-amp, amp).round(),
        )
    }
}

/// A stamp coming down on paper. Plays its sound and shakes when it lands.
pub struct Slam {
    t: f32,
    reject: bool,
}

impl Slam {
    pub fn approve() -> Self {
        Self {
            t: 0.0,
            reject: false,
        }
    }

    pub fn reject() -> Self {
        Self {
            t: 0.0,
            reject: true,
        }
    }

    pub fn rejects(&self) -> bool {
        self.reject
    }

    fn fall(&self) -> f32 {
        if self.reject {
            0.15
        } else {
            0.45
        }
    }

    pub fn update(&mut self, fx: &Fx, dt: f32) {
        let before = self.t;
        self.t += dt;
        if before < self.fall() && self.t >= self.fall() {
            if self.reject {
                fx.play(Sfx::Reject);
                fx.shake(0.3);
            } else {
                fx.play(Sfx::Stamp);
                fx.shake(0.1);
            }
        }
    }

    /// Size multiplier: starts large above the paper, lands at 1.
    pub fn scale(&self) -> f32 {
        let k = (1.0 - self.t / self.fall()).max(0.0);
        1.0 + 2.0 * k * k
    }

    pub fn draw_texture(&self, tex: &Texture2D, cx: f32, cy: f32, size: f32, rotation: f32) {
        let s = (size * self.scale()).round();
        draw_texture_ex(
            tex,
            cx - s / 2.0,
            cy - s / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(s, s)),
                rotation,
                ..Default::default()
            },
        );
    }

    /// A red rubber stamp word, tilted.
    pub fn draw_word(&self, font: &Font, word: &str, cx: f32, cy: f32) {
        let scale = self.scale();
        let w = crate::ui::text_width(font, word, 32) * scale;
        draw_text_ex(
            word,
            cx - w / 2.0,
            cy + 10.0 * scale,
            TextParams {
                font: Some(font),
                font_size: 32,
                font_scale: scale,
                rotation: -0.12,
                color: crate::ui::STAMP,
                ..Default::default()
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_is_audible_and_deterministic() {
        for sfx in ALL {
            let a = synth(sfx);
            assert!(a.len() > RATE as usize / 10, "{sfx:?} too short");
            assert!(a.iter().any(|s| s.abs() > 0.2), "{sfx:?} silent");
            assert!(a.iter().all(|s| (-1.0..=1.0).contains(s)));
            assert_eq!(a, synth(sfx));
        }
    }

    #[test]
    fn wav_header_matches_the_data() {
        let bytes = wav(&[0.0, 1.0, -1.0]);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 36 + 6);
        assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 6);
        assert_eq!(bytes.len(), 44 + 6);
        assert_eq!(i16::from_le_bytes([bytes[46], bytes[47]]), i16::MAX);
    }

    #[test]
    fn slam_lands_at_normal_size() {
        let mut s = Slam::approve();
        assert!(s.scale() > 2.9);
        s.t = 1.0;
        assert_eq!(s.scale(), 1.0);
    }
}
