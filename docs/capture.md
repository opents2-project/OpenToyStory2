# Capture tools

These tools are for making trailers, clips and screenshots from the engine. They are tools only and change nothing in the game: the game's own camera keeps running underneath any view these tools draw, and the laser still aims from the game's camera.

There are four parts:

- **Input recording:** play a stretch yourself; it can be replayed exactly, as often as you like.
- **Scripted rendering:** renders without a window (`--shot`) to a video or a set of PNGs. You can set the size, the frame rate and slow motion, and choose the camera.
- **Director camera:** in the live game, fly a camera of your own while Buzz keeps playing, and print its position for a script.
- **HUD toggle:** hide the HUD in the live game or in a script.

All examples below use `TS2_GAME_DIR` pointing at your install, and `ts2-viewer` from a checkout (`cargo run --release --bin ts2-viewer -- ...`).

## Recording a play session

```
ts2-viewer --level 2 --record-input run.ts2in
```

1. Press **F** to put Buzz at the level's start. Recording starts here.
2. Play.
3. Recording stops and the file is written when any of these happens:
   - you press **F9**;
   - you pause;
   - you press F again;
   - you run out of lives;
   - you close the window.

Mods work as usual (`--mod mods\play-as-woody`); the recording notes which were on.

**Limits:**
- It only works in a level opened directly, with `--level N` or a `level.ngn` path. It doesn't work from the front end.
- Use **F**, not Shift+F: Shift+F drops Buzz under the camera, which a replay can't repeat.
- While recording, the game runs at most one tick per frame, so a replay matches it exactly. Below 60 fps the game slows down a little instead of skipping ahead.

The file is plain text. Each line is an input, stamped with the game tick it arrived before. A replay applies each input before the same tick, so it plays out exactly the same at any frame rate or speed.

## Rendering: `--shot`

```
ts2-viewer --level 2 --size 1920x1080 --shot last.png "<script>"
```

The script is a list of words, run in order. The last frame is always written to the PNG named after `--shot`. `--size` sets the picture size; the default is 1280×720.

The script words:

| Word | What it does |
| --- | --- |
| `N` (a number) | Runs N output frames. |
| `rec:clip.mp4` | Writes every frame from here on into a video, through ffmpeg (H.264 at CRF 12, for editing). `.mkv` and `.mov` also work. ffmpeg must be on your PATH. |
| `rec:folder` | Writes numbered PNGs into the folder instead. |
| `rec:off` | Stops writing frames. |
| `fps:N` | Output frames per second. The default is 60. Set it before `rec:`. |
| `speed:X` | How much game time passes per output frame. `speed:0.25` is 4× slow motion. Between ticks, frames are blended, so slow motion stays smooth. |
| `hud:off` / `hud:on` | Hides or shows the HUD. |
| `replay:run.ts2in` | Puts Buzz at the level's start and plays the recording. Use this instead of `play`. When the recording ends, every key is let go. |
| `ff:N` | Runs N game ticks without drawing, for example to skip to the moment you want. |
| `play` | Puts Buzz at the level's start, as F does. |
| `+Key` / `-Key` | Presses or lets go of a key (winit names: `KeyW`, `Space`, …). |
| `+Fire` / `-Fire` | Presses or lets go of the left mouse button. |
| `tp:x,y,z[,facing]` | Puts Buzz at a point (level units). |
| `?` | Prints Buzz's position. |

Camera words. The view they draw replaces the game camera's picture only:

| Word | Camera |
| --- | --- |
| `view:game` | The game's own camera (the default). |
| `fix:x,y,z,yaw,pitch` | Held still (world units, radians). F7 in the live game prints exactly this word. |
| `track:x,y,z` | Stands at a point and turns to keep Buzz in view. |
| `follow:dx,dy,dz` | Moves with Buzz at an offset, looking at him. Rising with him makes a crane shot. |
| `path` or `path:ease` | Starts a camera path. `ease` slows it at the start and the end. |
| `key:F,x,y,z,yaw,pitch` | A path point at output frame F (counted from `path`). The path passes smoothly through each point. |
| `aim:F,x,y,z,tx,ty,tz` | A path point at frame F, looking at the point `tx,ty,tz`. |

World units: x is east, y is up, and z points the other way from the level's z. One level unit is 1/256 of a world unit. F7 prints both kinds, so you don't need to convert by hand.

### Examples

The coordinates here are placeholders; get real ones with F7.

A slow-motion jump from three angles, all from one recording:

```
ts2-viewer --level 2 --size 1920x1080 --shot a.png "replay:run.ts2in hud:off ff:300 speed:0.25 fix:-19,1,30,0.4,0 rec:out/takeoff.mp4 120 rec:off"
ts2-viewer --level 2 --size 1920x1080 --shot b.png "replay:run.ts2in hud:off ff:330 speed:0.25 track:-20,0.3,31 rec:out/air.mp4 120 rec:off"
ts2-viewer --level 2 --size 1920x1080 --shot c.png "replay:run.ts2in hud:off ff:360 speed:0.25 fix:-24,3,29,1.2,-0.3 rec:out/landing.mp4 120 rec:off"
```

A fly-through of a level, with no Buzz:

```
ts2-viewer --level 5 --size 1920x1080 --shot end.png "hud:off path:ease aim:0,-30,6,25,-21,0,32 aim:120,-10,4,40,-21,0,32 aim:240,5,8,20,0,0,0 rec:out/level05.mp4 240"
```

## Live keys

These keys work in any level window:

| Key | What it does |
| --- | --- |
| F7 | Prints the view as a `fix:` word, plus Buzz's `tp:` and the tick, and adds the line to `camera-poses.txt` in the current folder. |
| F8 | Turns the director camera on or off. While it's on: I/J/K/L fly, U/O go down and up, right Shift goes faster, and right-drag looks round. Buzz keeps playing under the game camera as usual. |
| F9 | Stops an input recording and writes it. |
| F10 | Hides or shows the HUD. |

A good way to plan shots:

1. Record a run.
2. Watch it back with `replay:`.
3. In the live game, put the director camera where you want each angle and press F7.
4. Paste the `fix:` words into the script.
