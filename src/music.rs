//! Background music: reads a MIDI file (assets/music/hold.mid, written by
//! tools/music.py) and synthesizes it as a chiptune loop at boot.
//! Channel 9 is drums (36 kick, 38 snare, 42 hat), channel 1 is bass, every
//! other channel plays a pulse lead.

use std::f32::consts::TAU;

use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};

pub const RATE: u32 = 22050;

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub channel: u8,
    pub key: u8,
    /// 0..1
    pub vel: f32,
    /// Seconds from the start of the song.
    pub start: f32,
    pub len: f32,
}

/// The notes of a MIDI file and the loop length in seconds (the last event,
/// rounded up to a whole beat).
pub fn parse(mid: &[u8]) -> Result<(Vec<Note>, f32), midly::Error> {
    let smf = Smf::parse(mid)?;
    let ppq = match smf.header.timing {
        Timing::Metrical(t) => t.as_int() as u64,
        Timing::Timecode(..) => 480,
    };

    // (tick, event) from every track, in time order.
    let mut events = Vec::new();
    for track in &smf.tracks {
        let mut tick = 0u64;
        for e in track {
            tick += e.delta.as_int() as u64;
            events.push((tick, e.kind));
        }
    }
    events.sort_by_key(|(tick, _)| *tick);

    // Tempo changes as (tick, seconds at that tick, seconds per tick).
    let mut tempo = vec![(0u64, 0.0f64, 0.5 / ppq as f64)];
    for (tick, kind) in &events {
        if let TrackEventKind::Meta(MetaMessage::Tempo(us)) = kind {
            let &(t0, s0, per) = tempo.last().unwrap();
            let at = s0 + (tick - t0) as f64 * per;
            tempo.push((*tick, at, us.as_int() as f64 / 1e6 / ppq as f64));
        }
    }
    let seconds = |tick: u64| {
        let &(t0, s0, per) = tempo.iter().rev().find(|(t, ..)| *t <= tick).unwrap();
        (s0 + (tick - t0) as f64 * per) as f32
    };

    let mut open: Vec<(u8, u8, u8, u64)> = Vec::new();
    let mut notes = Vec::new();
    let mut last = 0u64;
    for (tick, kind) in &events {
        last = last.max(*tick);
        let TrackEventKind::Midi { channel, message } = kind else {
            continue;
        };
        let channel = channel.as_int();
        let (key, vel, on) = match message {
            MidiMessage::NoteOn { key, vel } => (key.as_int(), vel.as_int(), vel.as_int() > 0),
            MidiMessage::NoteOff { key, vel } => (key.as_int(), vel.as_int(), false),
            _ => continue,
        };
        if on {
            open.push((channel, key, vel, *tick));
        } else if let Some(i) = open.iter().position(|o| o.0 == channel && o.1 == key) {
            let (_, _, vel, start) = open.remove(i);
            notes.push(Note {
                channel,
                key,
                vel: vel as f32 / 127.0,
                start: seconds(start),
                len: seconds(*tick) - seconds(start),
            });
        }
    }
    let end = last.div_ceil(ppq) * ppq;
    Ok((notes, seconds(end)))
}

fn hz(key: u8) -> f32 {
    440.0 * 2f32.powf((key as f32 - 69.0) / 12.0)
}

/// One note's sample at `t` seconds after its start. `noise` is per note.
fn voice(n: &Note, t: f32, noise: &mut impl FnMut() -> f32) -> f32 {
    let held = t < n.len;
    let release = if held {
        1.0
    } else {
        (1.0 - (t - n.len) / 0.04).max(0.0)
    };
    let sample = match n.channel {
        9 => match n.key {
            36 => (TAU * (50.0 + 100.0 * (-t * 35.0).exp()) * t).sin() * (-t * 14.0).exp() * 0.5,
            38 => (noise() * 0.6 + (TAU * 180.0 * t).sin() * 0.3) * (-t * 22.0).exp() * 0.35,
            _ => noise() * (-t * 70.0).exp() * 0.12,
        },
        1 => {
            let phase = (hz(n.key) * t).fract();
            let tri = 4.0 * (phase - 0.5).abs() - 1.0;
            tri * (0.55 + 0.45 * (-t * 5.0).exp()) * release * 0.32
        }
        _ => {
            let phase = (hz(n.key) * t).fract();
            let pulse = if phase < 0.25 { 0.75 } else { -0.25 };
            let attack = (t / 0.005).min(1.0);
            pulse * attack * (0.7 + 0.3 * (-t * 12.0).exp()) * release * 0.3
        }
    };
    sample * n.vel
}

/// Mono samples in -1..1 for exactly one loop. Tails past the end wrap to
/// the start, so the loop has no click.
pub fn render(notes: &[Note], end: f32) -> Vec<f32> {
    let len = (end * RATE as f32) as usize;
    let len = len.max(1);
    let mut out = vec![0.0f32; len];
    for (i, n) in notes.iter().enumerate() {
        let tail = if n.channel == 9 { 0.25 } else { n.len + 0.04 };
        let first = (n.start * RATE as f32) as usize;
        let mut state = 0x9e37_79b9u32 ^ (i as u32).wrapping_mul(0x85eb_ca6b);
        let mut noise = move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as f32 / u32::MAX as f32 * 2.0 - 1.0
        };
        for k in 0..(tail * RATE as f32) as usize {
            let t = k as f32 / RATE as f32;
            out[(first + k) % len] += voice(n, t, &mut noise);
        }
    }
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 0.9 {
        out.iter_mut().for_each(|s| *s *= 0.9 / peak);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped() -> (Vec<Note>, f32) {
        parse(&std::fs::read("assets/music/hold.mid").unwrap()).unwrap()
    }

    #[test]
    fn the_shipped_song_has_lead_bass_and_drums_in_a_whole_loop() {
        let (notes, end) = shipped();
        for channel in [0, 1, 9] {
            assert!(notes.iter().any(|n| n.channel == channel), "{channel}");
        }
        // 32 bars of 2/4 at 126 bpm
        assert!((end - 32.0 * 2.0 * 60.0 / 126.0).abs() < 0.01, "{end}");
        assert!(notes.iter().all(|n| n.len > 0.0 && n.start + n.len <= end));
    }

    #[test]
    fn render_is_one_loop_audible_and_deterministic() {
        let (notes, end) = shipped();
        let a = render(&notes, end);
        assert_eq!(a.len(), (end * RATE as f32) as usize);
        let peak = a.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(peak > 0.3 && peak <= 0.9 + 1e-6, "{peak}");
        assert_eq!(a, render(&notes, end));
    }
}
