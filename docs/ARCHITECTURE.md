# Antri: Architecture

Single Rust crate, macroquad, wasm32-unknown-unknown. No backend, no network calls at runtime.

## 1. Crates

| Crate | Purpose |
|---|---|
| macroquad | Window, input, rendering, text, asset loading on web |
| serde, ron | Data files |
| quad-storage | localStorage on web, a file on native. The only storage wrapper |

No other runtime dependencies. RNG is a hand-written SplitMix64 in `game/rng.rs` so determinism does not depend on a crate's version.

## 2. Module layout

```
src/
  main.rs            macroquad entry, game loop, calls scene::update and scene::draw
  canvas.rs          640x360 render target, integer scale, letterbox, mouse coordinate mapping
  assets.rs          loads textures, font, sounds, and all RON data once at boot
  save.rs            quad-storage read and write of SaveFile
  data/
    mod.rs           Rust structs mirroring the RON files, with serde derives
    load.rs          parse and validate data files, cross-check document ids
  game/              pure logic, no macroquad imports, fully unit tested
    rng.rs           SplitMix64, seed encoding as 6 char base32 string
    clock.rs         day and tick arithmetic
    meters.rs        Days, Money, Conscience, apply_cost
    grade.rs         grade math from GDD section 3 and 7
    quest.rs         document graph, inventory, next_required, is_complete
    obstacle.rs      weighted roll with modifiers
    queue.rs         The Queue simulation
    form.rs          Form Filling field shuffling and validation
  scene/
    mod.rs           Scene enum and transition function
    title.rs
    quest_select.rs
    overworld.rs
    office.rs        dialog, obstacle roll, minigame host, result
    minigame/        one file per minigame, each implements the Minigame trait
    ending.rs
  ui/
    text.rs          wrapped text, typewriter effect
    widgets.rs       button, meter bar, form field, dialog box
```

Rule: `game/` never imports macroquad. Scenes own the game structs and call into them. Tests live next to the code in `game/` and `data/`.

One trait exists, `Minigame`, because seven implementations exist on day one.

```rust
pub trait Minigame {
    fn update(&mut self, input: &Input, dt: f32, run: &mut Run) -> MinigameStatus;
    fn draw(&self, assets: &Assets);
}
pub enum MinigameStatus { Running, Passed, Failed(FailReason) }
```

`Run` is the single struct holding meters, clock, inventory, rng, quest id, and flags such as `tolak_halus_at: Vec<OfficeId>`.

## 3. Scene state machine

```
Boot -> Title -> QuestSelect -> Overworld
Overworld -> Office(id)        enter a building
Office -> Dialog -> ObstacleRoll -> [Minigame]* -> Result -> Overworld
Office -> Overworld            go home or office closed
Overworld -> Ending            final document in inventory, or fail condition
Ending -> QuestSelect          restart or next quest
Any -> Title                   Esc twice
```

`Scene` is an enum. `transition(scene, event) -> Scene` is a pure function with a table test. Office internal phases are an enum inside `office.rs`, not separate scenes, because they share the office backdrop.

The day advance check runs once per tick in `Run::advance(minutes)`. It returns `DayEnded` when the clock passes 15:00, and the scene handles the transition. Fail conditions are checked in `Run::status()` after every meter change.

## 4. Data files

All writer-facing content is RON under `assets/data/`. macroquad loads them with `load_string`, which uses fetch on web. The loader validates every referenced id at boot and panics with the file and id on a miss, so a typo fails fast in `make dev`.

`quests.ron`

```ron
[
  (
    id: "ktpss_hilang",
    title: "KTPSS Hilang",
    goal_doc: "ktpss",
    par_days: 4, deadline: 8, budget: 400, official_fees: 0,
    offices: ["polsek_sk", "pak_kaling", "kelurahan", "dukcatan"],
    hidden_pool: ["fotokopi_ksk_3", "pas_foto_merah"],
    wrong_pool: ["sktpa", "materai_2"],
    unlocks: "sktpa",
  ),
]
```

`offices.ron`

```ron
[
  (
    id: "dukcatan",
    name: "Dukcatan",
    map_pos: (412, 180),
    open: "08:00", last_number: "11:30", close: "15:00",
    grants: [ (doc: "ktpss", requires: ["surat_kehilangan", "pengantar_kelurahan", "$hidden"]) ],
    minigames: ["antrian", "isi_formulir", "cari_stempel"],
    obstacle_weights: { "sistem_offline": 25 },
    dialog: "dukcatan",
  ),
]
```

`$hidden` and `$wrong` are placeholders resolved per run from the quest pools.

`documents.ron`: id, display name, fee, and whether it is a fotokopi item.

`obstacles.ron`: default weights and effects from GDD section 6.

`forms.ron`: field pools per office with label variants and format variants.

`dialog/<office>.ron`

```ron
(
  lines: {
    "greet": ["Yes, what is it?", "Did you take a number, sir?"],
    "missing": ["You are missing the {doc}, sir. Did you photocopy it?"],
    "offline": ["The system is down. Head office."],
    "nego_hint": ["If you want it fast, there is... an administrative fee. Unofficial."],
  },
  nego: ( pay_min: 100, pay_max: 300 ),
)
```

Lines are arrays, the RNG picks one. `{doc}` is substituted with the document display name. `strings.ron` holds UI strings such as meter labels, grades, and the disclaimer. All strings are English.

## 5. Seeded randomness

The seed is a u64 shown as a 6 character base32 string. Quest select has a text field for it, blank means random. The run creates one SplitMix64 and derives child streams by hashing the seed with a purpose tag: `"hidden"`, `"obstacle:<office>:<day>"`, `"form:<office>:<visit>"`, `"queue:<office>:<day>"`. Derivation by tag means a player taking a different route still sees the same obstacle at the same office on the same day. Test: two runs with the same seed and the same action log produce identical meters.

## 6. Save format

One key in quad-storage, `antri.save.v1`, JSON text via serde. JSON rather than RON here because it is what a player sees if they open devtools.

```json
{
  "version": 1,
  "unlocked": ["ktpss_hilang", "sktpa"],
  "best": {
    "ktpss_hilang": { "grade": "B", "total": 61, "days": 5, "money": 230, "conscience": 70, "seed": "K7Q2MA" }
  },
  "settings": { "sfx": true }
}
```

Unknown version: ignore the save and start fresh. Write after every ending and every settings change. Nothing is saved mid-run, a reload restarts the quest.

## 7. Rendering and scaling

`canvas.rs` owns a 640x360 render target. Every scene draws into it with a fixed camera. After the scene draws, main computes `scale = floor(min(w / 640, h / 360))`, at least 1, and blits the target centred with black bars. Mouse coordinates are mapped back through the same transform. Nearest neighbour filtering on the target.

## 8. Build and deploy

`make web`: `cargo build --release --target wasm32-unknown-unknown`, copy the wasm, a vendored and pinned `mq_js_bundle.js`, `index.html`, and `assets/` into `dist/`. The `index.html` is static, no bundler. nginx serves `dist/` with `application/wasm` for `.wasm`, which `nginx:alpine` already maps.

Dockerfile: stage one is `rust:1-slim` with the wasm target, runs `make web`. Stage two is `nginx:alpine` with `dist/` copied to `/usr/share/nginx/html`. Kustomize in `deploy/` with Deployment, Service, Ingress.

## 9. Testing

`cargo test` on native covers `game/` and `data/`: meter math, grade table including grade X override, quest graph resolution with hidden and wrong requirements, obstacle roll distribution over 10000 samples within 2 percent of the weights, RNG determinism, scene transition table, save round trip. No rendering tests. `make check` runs `cargo fmt --check`, `cargo clippy -- -D warnings` for both native and wasm targets, and `cargo test`.
