# Antree: Architecture

Single Rust crate, macroquad, wasm32-unknown-unknown. No backend, no network calls at runtime.

## 1. Crates

| Crate | Purpose |
|---|---|
| macroquad | Window, input, rendering, text, asset loading on web |
| serde, ron | Data files |
| quad-storage | localStorage on web, a file on native. The only storage wrapper |
| midly | Reads the background song MIDI |

No other runtime dependencies. RNG is a hand-written SplitMix64 in `game/rng.rs` so determinism does not depend on a crate's version.

## 2. Module layout

```
src/
  main.rs            macroquad entry, game loop, calls scene::update and scene::draw
  canvas.rs          640x360 render target, integer scale (fractional below 2x), letterbox, mouse mapping
  fx.rs              synthesized sounds as WAV bytes, music loop, mute, screen shake, stamp slam animation
  music.rs           parses assets/music/hold.mid (midly) and renders it to samples, pure and tested
  assets.rs          loads the font, sprites, and Data once at boot
  art.rs             sprite sheets, tile fill, character walk frames
  data.rs            Rust structs mirroring the RON files, loading, validation, dialog lookup
  save.rs            quad-storage read and write of Save and the saved run
  ui.rs              text, wrapping, panels, dialog box, choice buttons, picker rows, HUD
  game/              pure logic, no macroquad imports, unit tested
    rng.rs           SplitMix64, tag-derived child streams, 6 char seed codes, shuffle
    clock.rs         day and minute arithmetic, office hours
    meters.rs        Money, Conscience, spend and shortcut
    grade.rs         grade math from GDD sections 3 and 7
    quest.rs         QuestDef, Grant, Task, Inventory, closure() over the document graph
    run.rs           Run: one playthrough, hidden and wrong requirements, office memory
    obstacle.rs      weighted roll with modifiers
    queue.rs         The Queue
    form.rs          Form Filling
    stamp.rs         Stamp Hunt
    photocopy.rs     Photocopy Run
    offline.rs       System Down
    nego.rs          Negotiation
    theory.rs        Queue Theory Test
  scene/
    mod.rs           Scene enum, shared helpers (say, escape_pressed)
    title.rs
    tutorial.rs      orientation form ORI-01, shown until approved once
    quest_select.rs  quest list, lock state, best results, seed entry
    overworld.rs     map, information board, travel
    office.rs        door check, greeting, chain check, obstacle roll, minigame host, issue
    minigame/        one screen per minigame, dispatched through an enum
    ending.rs        grade, save write
```

Rule: `game/` never imports macroquad. Game logic returns dialog keys (`&'static str`), and scenes turn them into text through `Data::line`. Tests live next to the code in `game/`, `data.rs`, and `save.rs`.

Minigames are an enum with `match` dispatch in `scene/minigame/mod.rs`, not a trait. Each screen has `update(ctx, run, office) -> Outcome` and `draw(ctx)`.

```rust
pub enum Outcome { Running, Passed(Option<String>), Granted(String), Failed { line: String, end_day: bool } }
```

`Run` holds meters, clock, inventory, the seed, the picked hidden and wrong requirements, which documents the player has learned about, and per-office memory (declined a bribe, paid a bribe, file under review, visit counts). `Run::door` rolls the guard at each door and remembers bought shoes.

## 3. Scene state machine

```
Boot -> Title -> Tutorial (until approved once) -> QuestSelect -> Overworld
Title -> Overworld             Continue, when a saved run exists
Overworld -> Title             Esc opens the pause menu, Main menu
QuestSelect -> Tutorial        Tab
Overworld -> Office(id)        enter a building
Office: Say | Greet -> [obstacles] -> [minigame steps] -> Issue -> Overworld
Office -> Overworld            walk out (Esc), fail, or office closed
Overworld -> Ending            goal document in inventory, or fail condition
Ending -> QuestSelect
QuestSelect -> Title           Esc
```

`Scene` is an enum. Office internal phases are an enum inside `office.rs`, not separate scenes, because they share the office backdrop. The office builds a stack of minigame steps from the obstacle roll plus the grant's task, and pops them one by one.

`Run::advance(minutes)` moves the clock and reports when the day ends at 15:00. Fail conditions are checked by `Run::status()`: days past the deadline, or money below the official fees still owed.

## 4. Data files

All writer-facing content is RON under `assets/data/`. macroquad loads them with `load_string`, which uses fetch on web. `Data::validate` runs at boot and in a unit test. It checks every document id, that every grant has a task, that the data correction letter is grantable, and that every quest can be won for every hidden and wrong pick.

`quests.ron`

```ron
(
    id: "ktpss_hilang",
    title: "Lost KTPSS",
    intro: "Your KTPSS fell out of your pocket...",
    goal_doc: "ktpss",
    start_docs: ["ksk"],
    par_days: 4, deadline: 8, budget: 400, official_fees: 0,
    board: ["loss_report", "kaling_letter", "district_letter", "ktpss"],
    hidden_pool: [("ksk_copy_3", "ktpss"), ("photo_red", "ktpss")],
    wrong_pool: ["sktpa", "duty_stamps_2"],
)
```

`hidden_pool` entries are `(document, the document that secretly requires it)`. One hidden and one wrong entry are picked per run. The wrong one shows on the board but no grant needs it. Menu order is the file order. `rule_pool` lists the documents a surprise regulation may add to the goal.

`offices.ron`

```ron
(
    id: "pak_kaling",
    name: "Pos Ketua Lingkungan",
    short: "Pak Gnilak",
    pos: (12, 36), size: (88, 56),
    grants: [ (doc: "kaling_letter", requires: ["ksk"]) ],
    tasks: [Form, Queue],
    weights: { SystemDown: 0, CounterMoved: 0, ClerkBreak: 30 },
)
```

A grant may set a `fee`, and a `task` that replaces the random pick from the office `tasks`. `opens_day` keeps the front door locked before that day.

`docs.ron`: id, board label, and an optional `phrase` for how a clerk says it mid-sentence.

`obstacles.ron`: default weights from GDD section 6. Offices override single entries.

`forms.ron`: the player's own data, with label variants and a kind (Text, Date, Number) that decides how it is misformatted.

`theory.ron`: the question pool for the Queue Theory Test.

`strings.ron`: UI strings such as meter labels, grades, hints, and the disclaimer.

`dialog/common.ron` and `dialog/<office>.ron`: maps from a dialog key to a list of variants. An office file overrides single keys, and `common.ron` is the fallback. A missing key renders as `??key??`. The variant is picked from the clock, so it is stable within a moment. `{doc}`, `{fee}`, and similar placeholders are filled by the scene. All strings are English.

## 5. Seeded randomness

The seed is a u64 shown as a 6 character base32 code. Quest select has a text field for it, blank means random. `Run::rng(tag)` derives a child stream from the seed, the tag, and the day, so the same office rolls the same obstacle on the same day whatever route the player takes. `Run::visit_rng(tag, office)` also mixes in the visit count, so a second visit gets a fresh queue or form.

## 6. Save format

One key in quad-storage, `antree.save.v1`, holding RON text. RON rather than JSON because serde_json would be the only reason to add another crate.

```ron
(version:1,best:{"ktpss_hilang":(grade:S,total:89,days:4,money:329,conscience:80,seed:"PVZZ3T")},inside_man:true,tutorial_done:true)
```

Every quest is open. A non-F ending unlocks the inside man. Old saves with an `unlocked` list still load, and the field is ignored. Best keeps the highest total, and an F never replaces a completion. Unknown version or unparsable text: ignore the save and start fresh. `tutorial_done` defaults to false, so saves from before it still load. Written after every ending and when the tutorial is approved.

A run in progress goes in a second key, `antree.run.v1`, only when the player picks Save progress in the pause menu. It holds `(version, run, player)`: the whole `Run` and the player's position. Grants are not stored. They come back from the data files on load, so a data edit never fights an old save. The title shows Continue while this key parses. The ending removes it when its seed and quest match the finished run. Main menu without saving keeps the last saved state.

The town (`overworld::draw_town`) is also the backdrop of the title, quest select, and tutorial screens.

## 7. Rendering and scaling

`canvas.rs` owns a 640x360 render target. Every scene draws into it with a fixed camera. After the scene draws, main computes `scale = floor(min(w / 640, h / 360))` from 2x up, or the exact fraction below 2x so phones fit, and blits the target centred with black bars. Mouse coordinates are mapped back through the same transform. Nearest neighbour filtering on the target.

## 8. Build and deploy

`make web`: `cargo build --release --target wasm32-unknown-unknown`, copy the wasm, `index.html`, `boot.js`, `pad.js`, the vendored and pinned `mq_js_bundle.js`, the quad-storage plugins `sapp_jsutils.js` and `quad-storage.js`, and `assets/` into `dist/`. `.cargo/config.toml` passes `--allow-undefined` to the wasm linker, because recent rustc rejects the JS imports those plugins provide at load time. The `index.html` is static, no bundler.

`pad.js` adds on-screen buttons when the page matches `(pointer: coarse)`. Each button dispatches a `KeyboardEvent` with a `code` to the canvas, which is what the macroquad bundle reads, so the game sees a real key and needs no touch code.

`server/` is `antree-server`, a wasi:http component that embeds `dist/` at build time and serves it with a CSP, `no-cache`, and one ETag per build. `build.sh` runs `make web` and builds the component. The Dockerfile copies `antree-server.wasm` into a `FROM scratch` `linux/amd64` image, which the cluster runs under the runwasi wasmtime shim.

## 9. Testing

`cargo test` on native covers `game/` and `data/`: meter math, grade table including grade X override, quest graph resolution with hidden and wrong requirements, every minigame's rules, obstacle roll distribution over 10000 samples within 2 percent of the weights, RNG determinism, data validation against the real files, save round trip. No rendering tests. `make check` runs `cargo fmt --check`, `cargo clippy -- -D warnings` for both native and wasm targets, and `cargo test`.
