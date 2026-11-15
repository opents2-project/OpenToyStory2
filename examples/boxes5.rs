//! Diagnostic: a level's push boxes, their paths and the terrain objects near each path.
//! Run: cargo run --release --example boxes5 -- <levelNN dir> (TS2_GAME_DIR set)
use std::{env, fs, path::PathBuf};
use ts2::formats::{exe, leveldat, ngn, terrain};
fn main() {
    let game = PathBuf::from(env::var("TS2_GAME_DIR").unwrap());
    let dir = game.join("data").join(env::args().nth(1).unwrap());
    let level = exe::level_number(&dir.join("level.ngn")).unwrap_or(1);
    let e = exe::Exe::parse(fs::read(game.join("toy2.exe")).unwrap()).unwrap();
    let n = ngn::parse(&fs::read(dir.join("level.ngn")).unwrap()).unwrap();
    let t = terrain::parse(&fs::read(dir.join("TERRAIN.ALL")).unwrap()).unwrap();
    let lists = leveldat::parse(&fs::read(dir.join("level.dat")).unwrap()).unwrap();
    let world = ts2::game::World::from_terrain(&t);
    let mut specials = ts2::specials::Specials::new(&n);
    let boxes = ts2::boxes::Boxes::new(level, &lists, Some(&e), &world, &mut specials);
    for (k, b) in boxes.boxes.iter().enumerate() {
        let path = lists.iter().find(|l| l.kind == b.path).map(|l| l.points.clone()).unwrap_or_default();
        println!("box {k}: special {} mover {} pos {:?} path {} {:?}", b.special, b.mover, b.pos, b.path, path);
    }
}
