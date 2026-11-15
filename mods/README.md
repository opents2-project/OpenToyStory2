# Example mods

Each folder is a mod: a `mod.toml` that names assets from your own Toy Story 2 install. No game data is stored here.

| Mod | What it does |
| --- | --- |
| `play-as-woody` | Plays any level as Woody, using Buzz's controls, physics and moves, with a revolver (the gunslinger's, in his right hand): left mouse fires yellow tracers. Q is first person as a shooter, as the First-Person Mod, with his arm and the gun in view. |
| `first-person` | First-Person Mod: Buzz in first person (Q) as a shooter. W/A/S/D run at full speed (strafing), Space jumps and double jumps, and ledges, zip lines, poles, the mantle, the hook and the boots work without leaving it; the mouse aims with no smoothing. His arms and laser stay; no glass helmet or breathing. |
| `play-as-zurg-robot` | Plays as one of Zurg's red spring robots (level 1's hopping enemy, ZURG1): hold a direction to bounce along; Space is Buzz's full jump and double jump. No laser. |
| `play-as-tinman` | Plays as the level 1 boss at full size: hold a direction to bounce along in low hops; touching enemies knocks them out. Camera further back. No laser. |

## Trying one

From a checkout, with `TS2_GAME_DIR` pointing at your install:

```
cargo run --release --bin ts2-viewer -- --mod mods\play-as-woody "%TS2_GAME_DIR%\data\level01\level.ngn"
```

Press F to start playing. `--mod` can be repeated; later mods win when two set `[play_as]`.

In the launcher: these examples are copied into `%APPDATA%\OpenToyStory2\mods` the first time it starts (or on Rescan). Tick one in the Mods tab and press Play Modded. Your own mods go in the same folder.

## `[first_person]`

First person (Q) as a shooter. Works with or without `[play_as]` (a play-as creature never shows Buzz's helmet or arms). The keys and their defaults:

```toml
[first_person]
move = true        # W/A/S/D run (strafing, full speed), Space jumps; ledges, zip lines, poles, the mantle,
                   # the hook's pull and the boots work without leaving first person; the mouse looks.
                   # false: the original's: standing still, W/A/S/D turn the view
smoothing = false  # true: the view eases after the mouse, as the original
breathing = false  # true: the helmet's breathing sounds and the arms' bob
helmet = false     # true: Buzz's glass helmet around the view
arms = true        # Buzz's first-person arms
```

## `[skate]`

Buzz on a skateboard. B (on the ground) gets on and off; off the board he is Buzz as ever. On it: hold a direction to push off (one push at a time, gaining speed while the foot is down) and to steer (the board carves towards it); let go and he rolls on, slowing a little; hold the opposite way to brake, then turn round. Space is an ollie (Buzz's jump; he keeps his speed in the air), E a kickflip (the board turns over once). Riding, Buzz's own moves are off: no double jump, spin, stomp, laser or first person, and he doesn't catch poles, ledges or zip lines. The board is a level model, drawn under his feet, raised so its wheels stand on the floor.

He is drawn from baked clips rather than his own animations: a JSON file in the mod's folder holding, per frame, a matrix for each of Buzz's 23 nodes and where the board's deck sits, all in his model space (+Y down, travelling along -Z before the half turn the viewer draws him with). No such file ships here: clips baked from another game's animations stay on your own machine.

```toml
[skate]
animations = "skate.json"          # the baked clips, in this folder
ride = "RIDE"                      # rolling (loops)
push = ["PUSH_IN", "PUSH_LOOP"]    # one push off, played in order
pop = "POP"                        # the ollie's take-off, then
air = "AIR"                        # its time in the air (holds its last frame)
flip_pop = "FLIP_POP"             # optional: the kickflip's own pop and air clips
flip_air = "FLIP_AIR"              # (without them it uses the ollie's)
board_model = "data/level07/level.ngn"   # a level file (relative to the game folder)
board_special = 11                 # the special object whose model is the board (level 7's skateboard)
board_scale = [0.24, 0.12, 0.24]   # its scale: across, height, along
top_speed = 0x700                  # from pushing (Buzz runs at 0x380), object units a tick
push_accel = 0x30                  # gained a tick while a push's foot is down
roll_friction = 3                  # lost a tick rolling
turn = 0x28                        # most it turns a tick (a full turn is 0x1000); half in the air
```

The clips file: `{"version": 1, "clips": {"NAME": {"fps": 60, "frames": [{"nodes": [[12 numbers] or null, ...], "board": [x, y, z]}, ...]}}}`. Each node's 12 numbers are a row-vector matrix (three rotation rows, then the translation) from his rest pose, as `scene::creature_batches_matrices` takes them.

## `[play_as]`

```toml
[play_as]
creature = "WOODY"                 # name in creatures.cfg
model = "data/level05/level1.ngn"  # optional, relative to the game folder; default: the level, then every level
animation = "retarget"             # Buzz's moves mapped onto the creature; or "hop": the bounce gait with the creature's own animations (idle = N, hop = N)
scale = 1.0                        # drawn size; collision stays Buzz's
laser = false                      # no firing, charging or first-person mode
map = [[0, 13]]                    # optional: [creature node, Buzz node] overrides to the automatic node map
[[play_as.rest]]                   # optional: turn nodes about a pivot before Buzz's motion applies
nodes = [5, 2, 3]
pivot = [42.0, -355.0, 5.0]
angles = [0.0, 0.0, 60.0]          # degrees about X, Y, Z
```

Also in `[play_as]` (before any `[[play_as.rest]]`), for `retarget`:

```toml
joints = [{ node = 2, parent = 5, at = [110.0, -358.0, 3.0] }]  # this part hangs off its parent at this point (creature rest pose); keeps longer limbs attached
hop_height = 4.0   # hop mode: the walking hop's height, as a multiple of the default low hop (about a third of Buzz's jump)
touch_damage = true   # touching an enemy hits it as Buzz's spin does; harmful ones don't hurt you
camera_distance = 2.5 # the follow camera's distance as a multiple of Buzz's, for big characters
```

`animation = "hop"` (Zurg robot, Tin Man): a held direction on the ground starts a low hop (about a third of Buzz's jump height) on every landing, so it only moves while airborne; Space is Buzz's normal jump and double jump; no direction stands still in the `idle` animation. The `hop` animation is stretched over each airtime. The mod loads the model from `data/level01/level.ngn` in any level.

### `[play_as.weapon]`

A gun held in one hand, for `retarget` mode. Its parts come from another creature (the Woody mod takes node 15 of the gunslinger, `GUNSL`) and follow the hand's final matrix. With a weapon, firing and first-person mode (Q) work again even when `laser = false` (Buzz's auto-aim, damage, charge, targeting with X, hooks and discs), and shots leave the gun's muzzle. Buzz's helmet and arms are not drawn in first person.

```toml
[play_as.weapon]
creature = "GUNSL"                 # where the gun parts come from
model = "data/level05/level1.ngn"  # optional; default: the level, then every level
nodes = [15]                       # the gun's nodes (only these are drawn)
hand = 3                           # play-as creature node it is held in (Woody: 3 right hand, 8 left)
offset = [189.0, -290.0, -41.0]    # gun origin in the hand, creature rest pose, model units
angles = [0.0, 0.0, 0.0]           # degrees about the gun's X, Y, Z before it is placed
muzzle = [0.0, -11.0, -65.0]       # optional, gun model units; default: middle of the end of its longest axis
shot = "bullet"                    # "bullet": a short yellow tracer; "beam": Buzz's own laser beam
```

First person draws the gun gripped in an arm at the bottom of the view, `[play_as.weapon.view]`. Eye space is model units from the eye: X to the creature's left, Y down, Z back.

```toml
[play_as.weapon.view]
nodes = [7, 8]                     # the creature's nodes drawn (Woody's right forearm and hand)
elbow = [-110.0, -358.0, 3.0]      # the forearm, from elbow to wrist (creature rest pose)
wrist = [-174.0, -300.0, -3.0]
at = [-34.0, 46.0, -120.0]         # where the wrist sits: right, down, ahead of the eye
dir = [0.25, -0.15, -1.0]          # where the forearm points
roll = 0.0                         # degrees about the forearm
gun = [0.0, 0.0, 0.0]              # the gun moved from the hand's centre
```

Tap the left mouse button: the arm comes up and the shot fires about a fifth of a second later (Buzz's timing).

The viewer prints the node map it used (`play_as: node map ...`). Unknown keys are an error; a creature that can't be found falls back to Buzz.

These mods are deviations from the original and only apply when enabled.
