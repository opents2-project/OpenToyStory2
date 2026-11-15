//! Diagnostic: Buzz's wing scale (node 11) frame by frame in the animations that open them.
use ts2::formats::{anim, ngn};
fn main() {
    let game = std::path::PathBuf::from(std::env::var("TS2_GAME_DIR").unwrap());
    let g = ngn::parse(&std::fs::read(game.join("data/level02/level.ngn")).unwrap()).unwrap();
    let c = g.creatures.iter().find(|c| c.id == 0).unwrap();
    for ai in [9usize, 11, 22, 29, 31] {
        let a = anim::parse(&c.animations[ai]).unwrap();
        let s: Vec<String> = (0..a.frame_count as usize).map(|f| format!("{:.2}", a.pose(11, f).map_or(-1.0, |p| p.scale[0]))).collect();
        println!("a{ai:02}: {}", s.join(" "));
        for f in [1usize, 8] { println!("   f{f} n11 {:?} n12 {:?}", a.pose(11, f), a.pose(12, f)); }
    }
}
