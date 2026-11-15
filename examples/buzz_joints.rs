//! Diagnostic: Buzz's joint seams (patches) as node pairs with the centre of their vertices.
use ts2::formats::ngn;
use std::collections::BTreeMap;
fn main() {
    let game = std::path::PathBuf::from(std::env::var("TS2_GAME_DIR").expect("TS2_GAME_DIR"));
    let g = ngn::parse(&std::fs::read(game.join("data/level01/level.ngn")).unwrap()).unwrap();
    let c = g.creatures.iter().find(|c| c.name == "BUZZ").unwrap();
    let mut pairs: BTreeMap<(usize, usize), Vec<[f32; 3]>> = BTreeMap::new();
    for p in &c.patches {
        let vs = p.vertices();
        let mut ns: Vec<usize> = vs.iter().map(|v| v.1).collect();
        ns.sort(); ns.dedup();
        let key = (ns[0], *ns.last().unwrap());
        for (vi, _) in vs { pairs.entry(key).or_default().push(c.shared_vertices[vi].pos); }
    }
    for ((a, b), v) in pairs {
        let n = v.len() as f32;
        let m = [0, 1, 2].map(|k| v.iter().map(|p| p[k]).sum::<f32>() / n);
        println!("{a:2}-{b:2} n={:3} centre [{:.0}, {:.0}, {:.0}]", v.len(), m[0], m[1], m[2]);
    }
}
