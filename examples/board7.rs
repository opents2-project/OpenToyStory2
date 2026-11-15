//! Diagnostic: level 7's skateboard (special 0xB): its instance and mesh bounds.
use ts2::formats::ngn;
fn main() {
    let game = std::path::PathBuf::from(std::env::var("TS2_GAME_DIR").expect("TS2_GAME_DIR"));
    let g = ngn::parse(&std::fs::read(game.join("data/level07/level.ngn")).unwrap()).unwrap();
    for id in [0xB, 0x18] {
        let s = g.specials.iter().find(|s| s.id == id).unwrap();
        let inst = &g.layers[s.layer].instances[s.instance];
        let mesh = &g.layers[s.layer].meshes[inst.mesh as usize];
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for v in &mesh.vertices { for k in 0..3 { lo[k] = lo[k].min(v.pos[k]); hi[k] = hi[k].max(v.pos[k]); } }
        println!("special {id:#x} layer {} inst {} rot {:?} scale {:?} verts {} tex {:?} lo {lo:?} hi {hi:?}", s.layer, s.instance, inst.rot.map(|r| r & 0xFFFF), inst.scale, mesh.vertices.len(), mesh.textures);
    }
}
