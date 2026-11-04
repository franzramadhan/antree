# Antree

A 2D top-down satire game about getting one piece of paper from the
government of Kuvukiland, a fictional republic.

Play it in the browser: <https://antree.franzramadhan.dev>

You took a day off work to replace a lost identity card. The board by the
road says what you need. The board is wrong. Walk between offices, take a
queue number, fill in forms, chase stamps, and photocopy everything in
triplicate. Every office is a small desk minigame.

Three meters keep score: days, money, and conscience. Every step offers a
shortcut (a fixer, grease money, cutting the line, an inside man). Each one
works, and each one costs you. The ending grades the kind of citizen you
were.

## Features

- Six errands, all open from the start, each with a seeded run you can
  replay from a six-letter code
- Minigames: the queue, forms, stamp hunts, photocopies, and a theory test
- Random obstacles: the system is down, the clerk is on a break, come back
  tomorrow
- A short orientation form that teaches the controls
- Save your progress in the browser and continue later
- Original pixel art, synthesized sound, and an original hold-music loop, no downloads after the first load

## Controls

| Key | Action |
|-----|--------|
| WASD or arrow keys | Walk, move through menus |
| E, Enter, Space | Enter an office, confirm |
| Number keys | Pick a dialog choice |
| H | Go home and end the day |
| Esc | Pause menu: save progress or go back to the main menu |
| M | Sound on or off |

To save progress, pick "Save progress" in the pause menu. Nothing saves on
its own. Saves
live in your browser's local storage, so clearing site data removes them.

## Build

You need Rust with the `wasm32-unknown-unknown` target.

```bash
make dev      # run natively
make web      # build the browser version into dist/
make serve    # serve dist/ on http://localhost:8080
make check    # fmt, clippy, and tests
```

Server image for a Kubernetes cluster with a wasmtime RuntimeClass (needs
the `wasm32-wasip1` target and `wasm-tools`):

```bash
./build.sh
docker build --platform linux/amd64 -t antree:local .
```

`antree-server.wasm` is a wasi:http component that embeds the web build. Run
it locally with `wasmtime serve -Scli antree-server.wasm`. A `v*` tag
publishes `ghcr.io/franzramadhan/antree`.

## Docs

- [Game design](docs/GDD.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Third-party assets](THIRD_PARTY_ASSETS.md)

## Disclaimer

Kuvukiland is a fictional country. Any resemblance to any real country is
purely coincidental, allegedly.

## License

Code is MIT (see `Cargo.toml`). Asset licenses are in
[THIRD_PARTY_ASSETS.md](THIRD_PARTY_ASSETS.md).
