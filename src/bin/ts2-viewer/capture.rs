//! Capture tools (deviation: tools, not in the original): input recordings that replay a play
//! session exactly, and camera paths for scripted shots. See docs/capture.md.

use glam::Vec3;
use winit::keyboard::KeyCode;

/// One input, as the viewer received it (keys after the player's bindings).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ev {
    Down(KeyCode),
    Up(KeyCode),
    /// Left mouse button (fire).
    Lmb(bool),
    /// Right mouse button (look / orbit).
    Rmb(bool),
    /// Raw mouse motion.
    Move(f64, f64),
}

/// A play session's inputs, each stamped with the number of game ticks run before it arrived
/// (from Buzz's spawn). Replayed before the tick after that stamp, it gives the same game.
#[derive(Default)]
pub struct InputLog {
    /// `level <path>`, `mod <dir>`, `camera active|passive`: what the replay must match.
    pub header: Vec<(String, String)>,
    pub events: Vec<(u64, Ev)>,
}

const MAGIC: &str = "# opents2 input v1";

impl InputLog {
    pub fn push(&mut self, tick: u64, ev: Ev) {
        // Mouse motion adds up within a tick: one line, not one per mouse report.
        if let (Ev::Move(dx, dy), Some((t, Ev::Move(x, y)))) = (ev, self.events.last_mut())
            && *t == tick
        {
            *x += dx;
            *y += dy;
            return;
        }
        self.events.push((tick, ev));
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.header.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    pub fn to_text(&self, ticks: u64) -> String {
        let mut s = format!("{MAGIC}\n");
        for (k, v) in &self.header {
            s += &format!("{k} {v}\n");
        }
        s += &format!("ticks {ticks}\n");
        for (t, e) in &self.events {
            s += &match e {
                Ev::Down(k) => format!("{t} down {k:?}\n"),
                Ev::Up(k) => format!("{t} up {k:?}\n"),
                Ev::Lmb(b) => format!("{t} lmb {}\n", *b as u8),
                Ev::Rmb(b) => format!("{t} rmb {}\n", *b as u8),
                Ev::Move(x, y) => format!("{t} move {x} {y}\n"),
            };
        }
        s
    }

    /// Parses a recording; the second value is its length in ticks.
    pub fn parse(text: &str) -> Result<(InputLog, u64), String> {
        let mut lines = text.lines();
        if lines.next().map(str::trim) != Some(MAGIC) {
            return Err("not an OpenToyStory2 input recording".into());
        }
        let mut log = InputLog::default();
        let mut ticks = 0;
        for (n, line) in lines.enumerate() {
            let w: Vec<&str> = line.split_whitespace().collect();
            let bad = || format!("line {}: {line}", n + 2);
            match w.as_slice() {
                [] => {}
                ["ticks", v] => ticks = v.parse().map_err(|_| bad())?,
                [t, kind, rest @ ..] if t.parse::<u64>().is_ok() => {
                    let t: u64 = t.parse().unwrap();
                    let ev = match (*kind, rest) {
                        ("down", [k]) => Ev::Down(key_code(k).ok_or_else(bad)?),
                        ("up", [k]) => Ev::Up(key_code(k).ok_or_else(bad)?),
                        ("lmb", [b]) => Ev::Lmb(*b == "1"),
                        ("rmb", [b]) => Ev::Rmb(*b == "1"),
                        ("move", [x, y]) => Ev::Move(x.parse().map_err(|_| bad())?, y.parse().map_err(|_| bad())?),
                        _ => return Err(bad()),
                    };
                    log.events.push((t, ev));
                }
                [k, v @ ..] => log.header.push((k.to_string(), v.join(" "))),
            }
        }
        Ok((log, ticks))
    }
}

/// Keys a recording can hold, by winit name.
const KEYS: &[KeyCode] = {
    use KeyCode::*;
    &[
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
        Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
        Space, Enter, NumpadEnter, Escape, Tab, Backspace, Delete, Insert, Home, End, PageUp, PageDown,
        ArrowUp, ArrowDown, ArrowLeft, ArrowRight, ShiftLeft, ShiftRight, ControlLeft, ControlRight, AltLeft, AltRight,
        BracketLeft, BracketRight, Comma, Period, Slash, Semicolon, Quote, Minus, Equal, Backslash, Backquote,
        F1, F2, F3, F4, F5, F6, F11, F12,
        Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    ]
};

pub fn key_code(name: &str) -> Option<KeyCode> {
    KEYS.iter().copied().find(|k| format!("{k:?}") == name)
}

/// A camera pose: position (world units) and yaw, pitch (radians), as the viewer's camera.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Pose {
    /// Standing at `pos`, looking at `at`.
    pub fn looking(pos: Vec3, at: Vec3) -> Pose {
        let d = (at - pos).normalize_or_zero();
        Pose { pos, yaw: d.x.atan2(-d.z), pitch: d.y.clamp(-1.0, 1.0).asin() }
    }
}

/// A camera path: poses at output frames, passed through smoothly (Catmull-Rom), yaw the
/// short way round between keys.
#[derive(Clone, Default, Debug)]
pub struct Path {
    pub keys: Vec<(f32, Pose)>,
    /// Ease in at the first key and out at the last.
    pub ease: bool,
}

impl Path {
    pub fn add(&mut self, frame: f32, mut p: Pose) {
        if let Some((_, last)) = self.keys.last() {
            let tau = std::f32::consts::TAU;
            p.yaw = last.yaw + ((p.yaw - last.yaw + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI);
        }
        self.keys.push((frame, p));
        self.keys.sort_by(|a, b| a.0.total_cmp(&b.0));
    }


    pub fn at(&self, frame: f32) -> Option<Pose> {
        let k = &self.keys;
        let (first, last) = (k.first()?, k.last()?);
        if k.len() == 1 || frame <= first.0 {
            return Some(first.1);
        }
        if frame >= last.0 {
            return Some(last.1);
        }
        let mut f = frame;
        if self.ease {
            let u = (f - first.0) / (last.0 - first.0);
            f = first.0 + u * u * (3.0 - 2.0 * u) * (last.0 - first.0);
        }
        let i = k.iter().rposition(|(t, _)| *t <= f).unwrap_or(0).min(k.len() - 2);
        let (t0, t1) = (k[i].0, k[i + 1].0);
        let t = if t1 > t0 { (f - t0) / (t1 - t0) } else { 0.0 };
        let p = |j: isize| k[(i as isize + j).clamp(0, k.len() as isize - 1) as usize].1;
        let (a, b, c, d) = (p(-1), p(0), p(1), p(2));
        let cr = |a: f32, b: f32, c: f32, d: f32| {
            0.5 * (2.0 * b + (c - a) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t * t + (3.0 * b - a - 3.0 * c + d) * t * t * t)
        };
        let v = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| Vec3::new(cr(a.x, b.x, c.x, d.x), cr(a.y, b.y, c.y, d.y), cr(a.z, b.z, c.z, d.z));
        Some(Pose {
            pos: v(a.pos, b.pos, c.pos, d.pos),
            yaw: cr(a.yaw, b.yaw, c.yaw, d.yaw),
            pitch: cr(a.pitch, b.pitch, c.pitch, d.pitch).clamp(-1.55, 1.55),
        })
    }
}

/// What the drawn view does (the game's own camera keeps running underneath).
#[derive(Clone, Debug)]
pub enum View {
    /// Held at one pose.
    Fixed(Pose),
    /// Along a path, by output frame.
    Path(Path),
    /// At Buzz's drawn position plus an offset (world units), looking at him.
    Follow(Vec3),
    /// Standing at a point, turning to keep Buzz in view.
    Track(Vec3),
}

/// Where the view looks on Buzz: about his chest (world units above his feet).
pub const BUZZ_CHEST: f32 = 1.0;

impl View {
    /// The pose at output frame `frame` (since the view was set), Buzz's feet at `buzz`.
    pub fn pose(&self, frame: f32, buzz: Option<Vec3>) -> Option<Pose> {
        let chest = buzz.map(|b| b + Vec3::Y * BUZZ_CHEST);
        match self {
            View::Fixed(p) => Some(*p),
            View::Path(p) => p.at(frame),
            View::Follow(o) => chest.map(|c| Pose::looking(c + *o, c)),
            View::Track(at) => chest.map(|c| Pose::looking(*at, c)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_round_trips() {
        let mut log = InputLog::default();
        log.header.push(("level".into(), "data/level02/level.ngn".into()));
        log.push(0, Ev::Down(KeyCode::KeyW));
        log.push(5, Ev::Move(1.0, 2.0));
        log.push(5, Ev::Move(0.5, -1.0));
        log.push(9, Ev::Lmb(true));
        log.push(12, Ev::Up(KeyCode::KeyW));
        let (back, ticks) = InputLog::parse(&log.to_text(40)).unwrap();
        assert_eq!(ticks, 40);
        assert_eq!(back.header("level"), Some("data/level02/level.ngn"));
        assert_eq!(back.events, vec![(0, Ev::Down(KeyCode::KeyW)), (5, Ev::Move(1.5, 1.0)), (9, Ev::Lmb(true)), (12, Ev::Up(KeyCode::KeyW))]);
    }

    #[test]
    fn path_passes_through_its_keys_and_turns_the_short_way() {
        let mut p = Path::default();
        p.add(0.0, Pose { pos: Vec3::ZERO, yaw: 3.0, pitch: 0.0 });
        p.add(10.0, Pose { pos: Vec3::X, yaw: -3.0, pitch: 0.0 });
        p.add(20.0, Pose { pos: Vec3::new(1.0, 0.0, 1.0), yaw: -3.0, pitch: 0.5 });
        assert_eq!(p.at(10.0).unwrap().pos, Vec3::X);
        // 3.0 to -3.0 is 0.28 rad across ±π, not 6 the long way.
        let mid = p.at(5.0).unwrap().yaw;
        assert!(mid > 3.0 && mid < 3.0 + 0.3, "{mid}");
        assert_eq!(p.at(99.0).unwrap().pitch, 0.5);
    }

    #[test]
    fn looking_at_a_point_ahead_is_level() {
        let p = Pose::looking(Vec3::ZERO, Vec3::new(0.0, 0.0, -5.0));
        assert!(p.yaw.abs() < 1e-6 && p.pitch.abs() < 1e-6);
    }
}
