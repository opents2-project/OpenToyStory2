//! Diagnostic: whether a scene's sound bank and music load. Run: cargo run --release --example scene_audio -- <game dir> <scene id>...
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let game = std::path::PathBuf::from(&a[1]);
    let exe = ts2::formats::exe::Exe::parse(std::fs::read(game.join("toy2.exe")).unwrap()).unwrap();
    for id in a[2..].iter().map(|s| s.parse::<usize>().unwrap()) {
        let path = ts2::campaign::variant_path(&game, id, 1);
        let level = ts2::formats::exe::level_number(&path);
        let bank = level.map(|l| ts2::audio::SoundBank::load(&exe, &game.join("data").join("sfx"), l).map(|b| b.sample_count()));
        let music = level.map(|l| ts2::music::level_track(&exe, l));
        println!("{id}: {} level {level:?} sounds {bank:?} music {music:?}", path.display());
    }
}
