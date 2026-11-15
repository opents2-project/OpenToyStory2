//! Deviation (owner's request): Buzz's pose for the token exit's "To infinity, and beyond!",
//! as the owner's footage of the original shows it, from Buzz's own animations:
//!
//! - From the voice line (BUZZINF, sound 0xA9) to the launch: state 0x1B's animation (31) at
//!   its move-table rate, a quarter frame a tick. The wings snap open, the arms go out with
//!   the fists low, and he leans into a crouch.
//! - The flight (0x44 ticks): state 0x16's animation (26) at half a frame a tick, through
//!   that state's frame list (0–11, 5–11 twice more, 12–17): his right fist pumps up over
//!   his head.
//!
//! Animation 26 has no wings of its own (their scale is 1/1024), so in the flight they keep
//! animation 31's last, fully open frame, carried on the chest so they sit on this body as
//! they sat on that one. This port's earlier exit used the double jump (state 8), which folds
//! them away half a second in.
//!
//! Matrices are for row vectors (`v' = v · M`), in model space (`formats::anim`).

use crate::formats::anim::Animation;
use crate::scene;

type M = [[f32; 4]; 4];

/// Ticks from the start of the voice line to the launch (0x4A0F80: the victory jumps when
/// its 1000-tick timer passes 0x37A).
pub const LAUNCH: u32 = 1000 - 0x37A;

const STANCE: usize = 31;
const STANCE_RATE: f32 = 0.25;
const FLY: usize = 26;
const FLY_RATE: f32 = 0.5;
/// State 0x16's frame list, as stored (`0xFF 1` after it: the last frame holds).
const FLY_LIST: [u8; 32] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 5, 6, 7, 8, 9, 10, 11, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
/// Eased over from the pose before, in ticks.
const BLEND: f32 = 6.0;

const IDLE_BASE: usize = 3;
const IDLE_OVERLAY: usize = 2;
const CHEST: usize = 13;
const WINGS: [usize; 4] = [11, 12, 17, 18];

/// Buzz's node matrices `tick` game ticks after the voice line started (fractions blend).
/// `anims` are his animations; None if the ones this needs are missing.
pub fn matrices(anims: &[Animation], nodes: usize, tick: f32) -> Option<Vec<Option<M>>> {
    let (stance, fly) = (anims.get(STANCE)?, anims.get(FLY)?);
    let launch = LAUNCH as f32;
    let whole = |a: &Animation, f: f32| scene::layered_matrices(nodes, Some((a, f)), Some((a, f)));
    let stance_at = |t: f32| whole(stance, (t * STANCE_RATE).clamp(0.0, (stance.frame_count - 1) as f32));
    if tick < launch {
        // Eased from his standing idle.
        let (base, over) = (anims.get(IDLE_BASE)?, anims.get(IDLE_OVERLAY)?);
        let idle = scene::layered_matrices(nodes, Some((base, 0.0)), Some((over, 0.0)));
        return Some(blend(&idle, &stance_at(tick), tick / BLEND));
    }
    let entry = (tick - launch) * FLY_RATE;
    let i = (entry.floor() as usize).min(FLY_LIST.len() - 1);
    let next = (i + 1).min(FLY_LIST.len() - 1);
    let (f0, f1) = (FLY_LIST[i] as f32, FLY_LIST[next] as f32);
    // A step back in the list (a loop) jumps rather than playing backwards.
    let f = if f1 > f0 { f0 + (f1 - f0) * entry.fract() } else { f0 };
    let mut m = whole(fly, f);
    // The wings: animation 31's last frame, from its chest onto this one.
    let open = (stance.frame_count - 1) as f32;
    if let (Some(their_chest), Some(chest)) = (stance.matrix_at(CHEST, open), m.get(CHEST).copied().flatten()) {
        let carry = mul(&inverse(&their_chest), &chest);
        for n in WINGS {
            if let (Some(w), Some(slot)) = (stance.matrix_at(n, open), m.get_mut(n)) {
                *slot = Some(mul(&w, &carry));
            }
        }
    }
    Some(blend(&stance_at(launch), &m, (tick - launch) / BLEND))
}

/// `a` towards `b` by `t` (0..1, eased), node by node.
fn blend(a: &[Option<M>], b: &[Option<M>], t: f32) -> Vec<Option<M>> {
    let t = t.clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    a.iter()
        .zip(b)
        .map(|(x, y)| match (x, y) {
            (Some(x), Some(y)) => Some(std::array::from_fn(|r| std::array::from_fn(|c| x[r][c] + (y[r][c] - x[r][c]) * t))),
            (_, y) => *y,
        })
        .collect()
}

fn mul(a: &M, b: &M) -> M {
    std::array::from_fn(|i| std::array::from_fn(|j| (0..4).map(|k| a[i][k] * b[k][j]).sum()))
}

/// The inverse of an affine node matrix (any scale).
fn inverse(m: &M) -> M {
    let a = |i: usize, j: usize| m[i][j];
    let det = a(0, 0) * (a(1, 1) * a(2, 2) - a(1, 2) * a(2, 1)) - a(0, 1) * (a(1, 0) * a(2, 2) - a(1, 2) * a(2, 0)) + a(0, 2) * (a(1, 0) * a(2, 1) - a(1, 1) * a(2, 0));
    let d = if det.abs() < 1e-12 { 1e-12 } else { det };
    let mut r = [[0.0; 4]; 4];
    r[0][0] = (a(1, 1) * a(2, 2) - a(1, 2) * a(2, 1)) / d;
    r[0][1] = (a(0, 2) * a(2, 1) - a(0, 1) * a(2, 2)) / d;
    r[0][2] = (a(0, 1) * a(1, 2) - a(0, 2) * a(1, 1)) / d;
    r[1][0] = (a(1, 2) * a(2, 0) - a(1, 0) * a(2, 2)) / d;
    r[1][1] = (a(0, 0) * a(2, 2) - a(0, 2) * a(2, 0)) / d;
    r[1][2] = (a(0, 2) * a(1, 0) - a(0, 0) * a(1, 2)) / d;
    r[2][0] = (a(1, 0) * a(2, 1) - a(1, 1) * a(2, 0)) / d;
    r[2][1] = (a(0, 1) * a(2, 0) - a(0, 0) * a(2, 1)) / d;
    r[2][2] = (a(0, 0) * a(1, 1) - a(0, 1) * a(1, 0)) / d;
    r[3][3] = 1.0;
    for j in 0..3 {
        r[3][j] = -(0..3).map(|k| m[3][k] * r[k][j]).sum::<f32>();
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flight_list_fits_the_exits_flight() {
        // 0x44 ticks of flight at half an entry a tick reach the list's last (held) frame.
        assert!(0x44 as f32 * FLY_RATE >= (FLY_LIST.len() - 1) as f32);
        assert_eq!(LAUNCH, 110);
    }

    #[test]
    fn inverse_undoes_a_node_matrix() {
        let (s, c) = 0.7f32.sin_cos();
        let m = [[2.0 * c, 2.0 * s, 0.0, 0.0], [-2.0 * s, 2.0 * c, 0.0, 0.0], [0.0, 0.0, 2.0, 0.0], [3.0, -4.0, 5.0, 1.0]];
        let id = mul(&m, &inverse(&m));
        for (i, row) in id.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                assert!((v - if i == j { 1.0 } else { 0.0 }).abs() < 1e-5, "{id:?}");
            }
        }
    }
}
