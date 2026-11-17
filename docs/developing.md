# Developing OpenToyStory2

Everything about the project's state and how to work on it, in one place. Players should start with the [README](../README.md).

OpenToyStory2 is a clean-room reimplementation of the 1999 PC game *Toy Story 2: Buzz Lightyear to the Rescue* (Traveller's Tales / Activision), written in Rust. The original exe's behaviour was worked out with Ghidra and a disassembler, then written again from that understanding. **The repository contains no game data and no original code**, apart from the app icon (a picture of the game's Pizza Planet token). Everything else is read at runtime from the player's own copy of the game. It is packaged in the spirit of [OpenMW](https://openmw.org): an installer, a launcher and the engine.

| Document | What's in it |
|---|---|
| This file | Status, known issues, ground rules, building, code layout, tests, tools, deviations |
| [docs/capture.md](capture.md) | Offscreen screenshots, video, camera paths, input recording |
| [docs/opents2-design.md](opents2-design.md) | The launcher and installer's design, and the plans for the mod file system and level editor |
| [mods/README.md](../mods/README.md) | The mod format |
| [docs/formats/](formats/) | The game's file formats |
| [docs/re/functions.md](re/functions.md) | Every identified exe function and global |

## Contents

1. [Status](#status)
2. [Known issues](#known-issues)
3. [Ground rules](#ground-rules)
4. [Building and running](#building-and-running)
5. [How the code is laid out](#how-the-code-is-laid-out)
6. [Tests](#tests)
7. [Tools](#tools)
8. [Working on the port](#working-on-the-port)
9. [Deviations from the original](#deviations-from-the-original)
10. [Roadmap](#roadmap)

## Status

**Beta.** All 15 campaign levels, the front end, saves, music and movies are in, and the game can be played from start to finish.

- **Level 1 (Andy's House)** is signed off by the project's playtester and is the regression reference.
- **Levels 2–15** have been through nine rounds of playtest reports. Every report that could be reproduced has been fixed against the original's code, with a regression test. A full sign-off is still to come.
- **OpenToyStory2 phase 1** (the launcher and installer) is done. Mods are supported through `mod.toml` files; four example mods ship with the launcher.

### What's in

- **Buzz:** running, jumping, double jump, stomp, spin attack and charged super spin; the laser and its charged shot; first-person aiming with target lock; poles, ledge grabs, mantling, zip lines, swing bars, pushing boxes and swimming; springs; rocket boots, hover boots, the disc launcher and the grappling hook; getting hurt, dying and lives.
- **World:** the original collision data (including the invisible boundary walls), movers that switch on and off, slide, tilt and carry Buzz, platforms on scripted paths, conveyors, slippery and sliding floors, and room detection.
- **Camera:** the original follow camera with its collision, the cutscene and talk cameras, look-at scenes and camera shake.
- **Creatures:** the shared behaviour-script interpreter, hit shapes, combat and coin drops, and type-specific routines for the enemies, characters and bosses of every level.
- **Pickups:** coins, batteries, extra lives, tokens (with their appear animation), the shield, power-ups, info signposts and ability-locked items.
- **Level scripts:** all 15 levels, with their characters, gadgets, challenges, races and bosses. Each `src/levelN.rs` module doc lists what it covers and what isn't ported.
- **HUD and conversations:** lives, batteries, coins, charge meters, challenge countdowns and boss bars; typed speech boxes; the "keep playing / exit level" token menu.
- **Effects:** particles from the exe's own tables, floor shadows, lens flares, moving lights, animated textures, rain and lightning.
- **Front end:** the startup screens and logo movies, title and main menu, options (with control remapping), the Etch A Sketch level select with its flying camera and token gates, results and "boss defeated" screens, eight save slots, the film viewer, credits, the attract demo and game over.
- **Sound:** positional sound effects and jingles, the original stereo music chosen as the exe chooses it, and the disc's MPEG-1 movies.
- **Rendering:** the original two-pass level draw (low-detail set in the distance, detailed set close up), back-face culling per material, Level 4's room/portal visibility, and drawing between game ticks so it is smooth on high refresh rate displays.

## Known issues

**Reported but not reproduced** (needs the reporter's spot or steps)
- Level 1: falling out at the living-room windowsill.
- Level 4: Buzz clipping through the hanging iron bar and the digger's scoop. Fixes went in (mover sweep, scoop ride) without a test reproducing the report.
- Level 13: oily footprints. They are ported and appear on ordinary floor after the oil.
- Level 14: "things not animated" (which things is unknown).

**Kept as the original does it**
- Level 2's pool float has no collision on its head; the original has none either.
- Power-ups come back 400 ticks (6.7 s) after being taken, one at a time.

**Collision**
- A body that starts inside an edge cylinder ignores that edge, as 0x4813B0 does.
- Being carried by a mover isn't swept against static walls (as in the original); turning movers carry Buzz only where a level script asks.
- Open edges in levels 5, 7, 8 and 10 found by `examples/clipsweep.rs` are unverified against the original.

**Drawing**
- Room culling is applied to Level 4 only, on purpose. Creatures and pickups aren't room-gated.
- The far set's near clip of 10000 isn't applied; creatures are drawn within 10176 rather than from the original's active list.
- The follow camera aims at Buzz's chest; the exe's aim is flatter (about 4° down instead of 14°). Unchanged pending a decision.

**Bosses**
- Per-type death sounds from the shared kill (0x405D20).
- Level 9: the lights, and the camera's every-0x1E-tick pick.
- Level 15: the swing outside phase 2, and boss 3's hurt voice.
- The dino fires from a fixed offset, not node 1. Campaign level 3's dust uses the boss's height, not the floor's.

**Not done**
- The shield slowing Buzz; some one-off light flashes (level 8's boss hitting Buzz among them).
- Camera shake isn't cleared when the camera resets (it lasts at most 40 ticks).
- With a low ceiling just above Buzz the follow camera can drift through it (the exe does the same).
- Level 5's stomp-started machine and its bubbles, and underwater fog. Level 8's "cannon" is still to be checked against the original.
- Health isn't carried between levels, and the volume settings aren't saved.

## Ground rules

- **No original data in the repository.** No `.all`, `.ngn`, `.raw`, `.wav` or `.mpg` files, no exe bytes and no decompiled C (`.gitignore` enforces the file types). The game is still Disney/Activision copyright. The one exception is the app icon in `assets/icon/`, a picture of the Pizza Planet token made from the player's own install.
- **Behaviour comes from reading the original code**, then writing it again as idiomatic Rust; no line-by-line translation of decompiler output. Name things by what they do and cite the exe address in comments (`0x417680`).
- **Document what you find:** file formats in `docs/formats/`, functions in `docs/re/functions.md`, and every parser gets a test against a real install.
- **Deviations from the original** need a reason. Mark each one "Deviation:" in the code and list it [below](#deviations-from-the-original).
- `re/` (the Ghidra project and decompiler output), `local/` and `out/` are git-ignored and stay private.

### Source material

| Source | Notes |
|---|---|
| `toy2.exe` | MSVC, 32-bit, built 1999-10-29. D3D6 / DirectDraw / DirectInput / DirectSound / DirectShow, on Traveller's Tales' "nu3d" engine. The source of truth for behaviour. |
| `data\` | Levels as `.ngn` (written by TT's `pconv.exe` from the PS1 data), `.raw` packets, `.dat` point lists, `TERRAIN.ALL` collision, `sfx\*.wav`, `creatures.cfg`. |
| PS1 executables in `data\` | `SLUS_008.93`, `SLES_020.67`, `psx.exe`: the same logic in MIPS, for cross-reference. |
| The game disc | Music (23 WAVs in `audio\`) and movies (22 MPEG-1 streams named `.dll` in `rtlibs\`). |

Campaign order differs from the exe's internal level ids: campaign 3 (Bombs Away) is internal 6, and campaign 6 (Slime Time) is internal 3. Levels 11–15 are `level1.*` files in folders `level01`–`level05` (0x452FC0 subtracts 10); the `level11`–`level15` folders only hold converter configurations. See `src/campaign.rs`.

## Building and running

You need stable Rust with the MSVC build tools, and an installed copy of the PC game (by default `C:\Program Files\Disney Interactive\Toy Story 2`; set `TS2_GAME_DIR` if it's elsewhere). For music and movies, set `TS2_CD_DIR` to a folder containing the disc's `audio\` and `rtlibs\`, or put that path in a git-ignored `ts2-media.txt` in the repository folder.

The whole game, from the startup screens:

```bash
cargo run --release --bin ts2-viewer
```

One level by campaign number (1–15), in a free camera; F plays as Buzz and back, Escape closes:

```bash
cargo run --release --bin ts2-viewer -- --level 4
```

A level opened this way starts with every ability, so the gadget cages are open. Other flags: `--fullscreen`, `--mod <folder>` (repeatable), `--shot` (see [capture.md](capture.md)).

The launcher:

```bash
cargo run --release --bin opents2
```

### Building the installer

Needs [Inno Setup 6](https://jrsoftware.org/isinfo.php) (`winget install JRSoftware.InnoSetup`):

```bash
powershell -ExecutionPolicy Bypass -File tools\installer\build.ps1
```

It builds the release `opents2` and `ts2-viewer` and writes `target\installer\OpenToyStory2-<version>-setup.exe`. The version comes from `crates/launcher/Cargo.toml`. The C runtime is linked statically (`.cargo/config.toml`), so players don't need the VC++ redistributable.

The installed launcher keeps its settings in `%APPDATA%\OpenToyStory2\opents2.toml` (`OPENTS2_HOME` overrides), mods in `mods\` beside it, the engine's last output in `last-run.log`, and a disc image's music and movies (about 370 MB) in `disc\`. Saves go to `%APPDATA%\ts2-rust` (`TS2_SAVE_DIR` overrides).

### The icon

The icon is the token's flat, face-on picture (level 1 special 0x39, mesh 395, texture `tex16`), rotated to the tilt it has in game. To regenerate it from your install (needs Pillow):

```bash
cargo run --release --example token_model -- "C:\Program Files\Disney Interactive\Toy Story 2" out\token
```

```bash
python tools/icon/make_icon.py out/token
```

It writes `assets/icon/opents2.ico`, the PNGs and the installer's wizard pictures.

## How the code is laid out

```
Cargo.toml            workspace; also the engine crate `ts2`
src/
  formats/            parsers: ngn, anim, terrain, leveldat, packets, exe, all, bmp
  game.rs             World (collision, movers, rooms, surfaces) and Player (Buzz's 60 Hz controller)
  buzz.rs             move-table animation (0x401000)
  camera.rs, look.rs  follow camera (0x4045E0); first person (0x4038E0)
  creatures.rs        creature slots, the behaviour-script interpreter (0x4076F0), per-type routines
  script.rs           the Script trait and Common level state; for_level builds a level's script
  level1.rs … 15.rs   each level's start-up and frame script, by internal id
  campaign.rs         campaign order and scene paths
  frontend.rs, save.rs, pause.rs, controls.rs
  pickups.rs, boxes.rs, rewards.rs, particles.rs, laser.rs
  hud.rs, dialog.rs, cutscene.rs
  audio.rs, music.rs, fmv.rs
  scene.rs, specials.rs, shadows.rs, flares.rs, lights.rs, texanim.rs, rain.rs
  modding.rs          mod.toml: [play_as], [first_person], [skate]
  exit_pose.rs        the level exit's "To infinity and beyond" pose
  bin/ts2-viewer.rs   the game: wgpu renderer, winit window, rodio audio, input, glue
  bin/ts2-tool.rs     asset audit and exporters
crates/launcher/      the launcher (`opents2`, egui)
examples/             diagnostics (see Tools)
tests/                integration tests against a real install
mods/                 the example mods and the mod format's guide
assets/               the icon and the README's pictures
tools/                icon generator, Inno Setup installer, Ghidra dump script
```

Rendering is wgpu + winit, sound rodio, movie video `oxideav-mpeg12video` and movie sound Symphonia. The launcher uses egui (eframe on the same wgpu and winit), rfd for folder pickers, and serde with toml for settings.

### Units and conventions

- Object units = level units × 32. `Player::pos` is in level units (f32); `pos_obj()` gives object units.
- +Y is **down**; floors face −Y.
- Angles are 12-bit (0x1000 a turn) in game code, 16-bit in `.ngn` instance rotations; the mover table uses 0x4000 a turn.
- 60 ticks a second.
- Scripts address movers by terrain link id − 1. A room is the low byte of a room-map object's flags.
- `level.dat` lists: 58 is reward sparkles and box spots, 60 ledges, 61 poles, 62 zip lines, 63 pickups, 65 room portals.
- Creature matrices turn Z first, then Y, then X. Node matrices are for row vectors (`v' = v · M`).
- Far-set (low-detail) specials take level units like the near set: where the exe passes `>> 7`, pass `>> 5`.
- The shader reads vertex colour RGB from bytes 1–3; anything that tints vertex colours must offset by one.
- Material mode bit 8 means no back-face culling (0x4B6760).
- Negative sound ids from scripts are jingles. Footsteps come from frame-list markers 0x81 / 0x82.
- The viewer draws between ticks (`Interp` in `ts2-viewer.rs`). Anything new that moves every tick and is drawn should be blended the same way, or it judders on fast displays.
- Front-end scenes: scene 0 is `level00\level.ngn`; the map is scene 16 (`level06\level1t1.ngn` for the level select, `level1t2.ngn` for the film viewer). Picture n is texture `bgrNN`. Menus are drawn on a 320-wide screen, the level select on a 512-wide one.

## Tests

Tests that need game data are skipped unless `TS2_GAME_DIR` is set; the movie test also needs the disc files (`TS2_CD_DIR`), and the launcher's disc-image tests need `TS2_DISC_IMAGE`.

```bash
set TS2_GAME_DIR=C:\Program Files\Disney Interactive\Toy Story 2
cargo test --release --workspace
```

About 310 tests cover parsing every level, movement and collision, combat and every boss's attacks, races and challenges, gadgets, the front end, saves and movie decoding. A skipped test isn't coverage, and a passing simulation isn't proof that a level plays correctly: check visual changes with a `--shot` render, and have a person play anything that matters.

If the game is running, cargo can't replace `ts2-viewer.exe`; build into another folder with `--target-dir`.

## Tools

**Offscreen screenshots and video:** `ts2-viewer [--level N] --shot out.png "<script>"` renders without a window and without sound. Script words include frame counts, `play`, `+Key` / `-Key`, `tp:x,y,z`, `fix:` cameras and `?`. The full list, with video, slow motion, camera paths and input replay, is in [capture.md](capture.md).

**Asset audit:** `cargo run --release --bin ts2-tool -- audit-levels "<game dir>"` parses the geometry, textures, models, point lists, spawns and collision of every campaign level.

**Diagnostics** (`cargo run --release --example <name> -- …`; most need `TS2_GAME_DIR`):

| Example | What it does |
|---|---|
| `cast` | Which creatures each level carries, with node and animation counts |
| `creatures [levelNN]` | Creature slots, records and behaviour scripts (`WORDS=n` for long scripts) |
| `nodes <ngn> <NAME>` | A creature's nodes with their rest-pose bounds |
| `bossanim <level> slot x y z [ticks]` | A creature slot tick by tick with Buzz held at a point |
| `exedump <type> <hex va> [n]` | Reads exe data tables by address (`i32`, `i16`, `u8`, `str`, `strs`) |
| `textures <ngn> [index out.bmp]` | An ngn's textures; writes one out |
| `sprite_dump <exe> <scene> [from to]` | The 2D sprite table a scene uses |
| `vcols <ngn> layer index` | An instance's draw batches: blend, texture, vertex colours |
| `materials`, `flagged <ngn>`, `overlap <ngn>` | Material modes; instances by flag bits; coplanar faces that z-fight |
| `under <ngn> box`, `zones <ngn> [zone]` | Instances in a region; instances by zone |
| `sky`, `backdrop <dir> out.png` | Each level's background picture and colours; one level's picture as a PNG |
| `lists <dir>` | `level.dat` point lists (`ALL=1` for every point) |
| `pickups <dir>`, `kinds <dir> <level>` | Pickups and specials; pickups by kind |
| `movers <dir>` | The level's movers |
| `surfaces <dir> x y z r`, `floor_at <dir> x z`, `object_tris <dir> index` | Collision objects and surface types near a point; every floor under a point; one object's triangles |
| `boundaries`, `room <dir> x y z` | Boundary-wall records in every terrain file; the room at a point |
| `rotfit` | How rotated instances sit on the collision, with and without their rotation |
| `pushsim <dir>` | Buzz pushing each of a level's boxes |
| `clipsweep [--level N]` | Drops Buzz on every floor on a grid and reports passing through walls or floors |
| `edge_turns`, `camera_poles` | Buzz turning on edges; the follow camera while climbing each pole |
| `dialog` | A level's conversations through its script, and the speech box's typing |
| `token_model <game> <out>` | Exports the Pizza Planet token for the icon generator |

## Working on the port

The method that took Level 1 to sign-off, and is used for every fix since:

1. **Read the level's module doc** (`src/levelN.rs`) and its frame script in the decompiler output. Start-ups and frames are in the exe's tables at 0x4A1AB8 / 0x4A1B68, indexed by internal id − 1.
2. **Map the level's objects** with the diagnostics (`creatures`, `lists`, `pickups`, `movers`, `under`, `surfaces`) before touching behaviour.
3. **Get a report from someone playing it,** with a screenshot and a description of what the original does.
4. **Find the original code first.** Search the decompiler output for the globals involved and disassemble where the decompiler drops arguments. Compare line by line before deciding the port is wrong: several "bugs" turned out to be the original's behaviour.
5. **Write a diagnostic before a fix** when the cause isn't obvious: a small example or test that loads the level and simulates the case.
6. **Fix it, add or extend a test**, and run the full suite.

### The decompiler output

`re/` is private and git-ignored. To recreate it, import `toy2.exe` into a Ghidra 12 project at `re\proj` and run the dump script headless (about 10 minutes):

```bash
analyzeHeadless.bat re\proj TS2 -import re\toy2.exe -overwrite -scriptPath tools\ghidra -postScript DumpAround.java re\out
```

It writes one C file per function to `re/out/c/<address>.c`, plus `functions.tsv` and `strings.tsv`. Some functions aren't in the dump (level 10's start-up, 0x425B60): disassemble those directly.

### Lessons worth knowing

- **Flag bytes vs flag words.** A byte write at an odd offset (`byte +0x41 & 0xFB`) is a bit of the flag word (0x400). Convert it before porting.
- **Uninitialised "last" values.** Bosses compare their health with a remembered value; set those at start-up, as the original does.
- **Argument order.** Check it from a second call site; the talk camera's position and look-at tracks were once swapped.
- **State set at the event.** When "sometimes it animates" comes up, look for a state written directly at the event (the ledge grab sets Buzz's state itself).
- **Creature slot fields:** +0x40 flags, +0x42 draw distance, +0x50 home, +0x5C target, +0x68 target facing. Several bugs came from writing home or flags instead of target or +0x42.
- **Box table fields** (0x53C680, 0x28 bytes each): +0x18 is the progress along the segment. Level scripts read and write it directly (level 5's gate).
- **Scene changes rebuild the viewer's state.** Anything that must outlive one (a queued movie, the music playing) has to be carried across in `switch_scene`.
- **The repository uses `core.autocrlf=input`.** A checkout as CRLF once broke multi-line patches silently.

## Deviations from the original

A few things are deliberately different, mostly to fix what looked wrong when drawn further or sharper than the original:

- Every creature keeps moving wherever the camera is (the original freezes distant ones); only nearby ones make sound.
- First-person mode has mouse look.
- The PS1 background picture is drawn behind windows and the sky (the PC version only used two colours from it).
- Faces stored pure black in otherwise lit objects take the object's light, and fully transparent-textured glass (the kitchen jars) is drawn as green glass.
- Coins are alpha-tested and write depth (no distance fade).
- Level 1's garage drill bits move and hurt, matched to the original by eye.
- Pushing can start just after leaving the floor, for the sloped kitchen shelf; the push animation only plays while the box moves.
- The pickup jingle plays its first note only.
- Leaving a level after a Pizza Planet token, Buzz spreads his wings and crouches on "To infinity, and", then flies up pumping his fist, as footage of the original shows (this exe's code does an ordinary jump). Both are his own animations (states 0x1B and 0x16); the wings stay open through the flight.
- A double jump with the rocket boots lit shows the wings.
- Inside the shield's energy ball Buzz picks up no slime footprints.
- Blended level faces that are fully opaque (level 2's leaves and bushes) are drawn alpha-tested with depth writes.
- Special objects (lifts, platforms) are drawn between game ticks like Buzz and the camera.
- Display settings: the options' graphics page has two extra rows, windowed or fullscreen and the resolution (window sizes up to the monitor's, 8K included; in fullscreen the monitor's own modes, or native for borderless). F11 / Alt+Enter toggle borderless fullscreen at any time, `--fullscreen` starts in it, and the choice is kept in `display.txt` beside the saves.
- Saves go to `%APPDATA%\ts2-rust` instead of beside the game, which is usually read-only. They use the original's `Toy2NN.sav` format, so saves copy between the two.
- Mods (opt-in, `--mod <dir>` or the launcher): `[play_as]` draws the player as another creature with Buzz's moves retargeted (Woody, with a revolver; Zurg's spring robot and the Tin Man as bouncing players); `[first_person]` turns first person into a shooter; `[skate]` puts Buzz on level 7's skateboard, drawn from clips the mod supplies. The default game is unchanged. See [mods/README.md](../mods/README.md).

## Roadmap

1. **Finish the port:** sign off levels 2–15 and close the known issues above.
2. **Mods (OpenToyStory2 phase 3):** a layered virtual file system (mods over the game's data), asset overrides and scripted behaviour. Design in [opents2-design.md](opents2-design.md).
3. **Level editor (phase 2):** `opents2-editor`, for placing objects, creatures, pickups and paths, saved as mod files.
4. **Other platforms:** Linux and macOS packages; the engine's default game path must stop assuming `C:\`.
