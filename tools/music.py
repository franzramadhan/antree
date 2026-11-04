"""Write assets/music/hold.mid, "Please Hold (Ministry Waltz Not Included)".

An original composition for Antree, released CC0 with the rest of the
generated assets. A jaunty 2/4 oom-pah, the kind that plays while a call
centre puts you on hold forever: verse, verse with an "uh-oh" ending, a
stuck "please hold" bridge, verse. The game loops it.

Channels: 0 melody, 1 bass, 9 drums (General MIDI: 36 kick, 38 snare,
42 hat). src/music.rs reads the file and synthesizes it, so edit it in any
MIDI editor or change this script and run `make music`.
"""

import struct
from pathlib import Path

PPQ = 480
EIGHTH = PPQ // 2
TEMPO_BPM = 126

NOTE = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}


def key(name):
    """'C5', 'Bb4', 'F#3' to a MIDI key number (C4 = 60)."""
    pitch = NOTE[name[0]]
    rest = name[1:]
    if rest[0] in "b#":
        pitch += -1 if rest[0] == "b" else 1
        rest = rest[1:]
    return 12 * (int(rest) + 1) + pitch


# Melody as (note or "-" for a rest, length in eighths). A bar of 2/4 is 4.
VERSE = [
    ("E5", 2), ("G5", 1), ("E5", 1),
    ("D5", 2), ("C5", 2),
    ("D5", 1), ("E5", 1), ("F5", 1), ("D5", 1),
    ("E5", 4),
    ("E5", 2), ("G5", 1), ("E5", 1),
    ("D5", 2), ("C5", 2),
    ("B4", 1), ("C5", 1), ("D5", 1), ("B4", 1),
]
VERSE_END = [("C5", 4)]
UH_OH_END = [("C5", 1), ("C5", 1), ("Bb4", 2)]
HOLD = [
    ("G5", 1), ("G5", 1), ("G5", 1), ("G5", 1),
    ("G5", 1), ("G5", 1), ("A5", 1), ("G5", 1),
    ("F5", 1), ("F5", 1), ("F5", 1), ("F5", 1),
    ("F5", 1), ("F5", 1), ("G5", 1), ("F5", 1),
    ("E5", 1), ("E5", 1), ("E5", 1), ("E5", 1),
    ("E5", 1), ("F5", 1), ("E5", 1), ("D5", 1),
    ("C5", 2), ("E5", 2),
    ("C6", 1), ("-", 3),
]
MELODY = VERSE + VERSE_END + VERSE + UH_OH_END + HOLD + VERSE + VERSE_END

# One chord root per bar, for the oom-pah bass.
VERSE_CHORDS = ["C", "G", "G", "C", "C", "G", "G", "C"]
HOLD_CHORDS = ["G", "G", "F", "F", "C", "C", "G", "C"]
CHORDS = VERSE_CHORDS + VERSE_CHORDS + HOLD_CHORDS + VERSE_CHORDS
FIFTH = {"C": "G2", "G": "D3", "F": "C3"}


def track(events):
    """events: (tick, status, data1, data2). Returns an MTrk chunk."""
    out = bytearray()
    now = 0
    for tick, status, a, b in sorted(events, key=lambda e: (e[0], e[1] & 0xF0 == 0x90)):
        out += varlen(tick - now) + bytes([status, a, b])
        now = tick
    out += varlen(0) + b"\xff\x2f\x00"
    return b"MTrk" + struct.pack(">I", len(out)) + out


def varlen(n):
    out = [n & 0x7F]
    n >>= 7
    while n:
        out.insert(0, (n & 0x7F) | 0x80)
        n >>= 7
    return bytes(out)


def note(events, channel, at, length, k, velocity):
    events.append((at, 0x90 | channel, k, velocity))
    events.append((at + length, 0x80 | channel, k, 0))


def main():
    melody, bass, drums = [], [], []
    at = 0
    for name, length in MELODY:
        if name != "-":
            # a hair short of the full length, so repeated notes re-attack
            note(melody, 0, at, length * EIGHTH - 30, key(name), 100)
        at += length * EIGHTH
    bars = len(CHORDS)
    assert at == bars * 4 * EIGHTH, (at, bars)

    for bar, root in enumerate(CHORDS):
        start = bar * 4 * EIGHTH
        note(bass, 1, start, EIGHTH, key(root + "2"), 110)
        note(bass, 1, start + 2 * EIGHTH, EIGHTH, key(FIFTH[root]), 95)
        note(drums, 9, start, 60, 36, 110)
        note(drums, 9, start + 2 * EIGHTH, 60, 38, 90)
        for e in (1, 3):
            note(drums, 9, start + e * EIGHTH, 30, 42, 60)

    tempo = 60_000_000 // TEMPO_BPM
    meta = (
        varlen(0) + b"\xff\x51\x03" + tempo.to_bytes(3, "big")
        + varlen(0) + b"\xff\x58\x04\x02\x02\x18\x08"
        + varlen(0) + b"\xff\x2f\x00"
    )
    chunks = [b"MTrk" + struct.pack(">I", len(meta)) + meta]
    chunks += [track(t) for t in (melody, bass, drums)]
    header = b"MThd" + struct.pack(">IHHH", 6, 1, len(chunks), PPQ)

    out = Path(__file__).resolve().parent.parent / "assets/music/hold.mid"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(header + b"".join(chunks))
    print(f"wrote {out} ({bars} bars, {bars * 2 * 60 / TEMPO_BPM:.1f} s)")


if __name__ == "__main__":
    main()
