//! Diagnostic: the sprite table 0x447D40 builds for a scene. Run: cargo run --release --example sprite_dump -- <toy2.exe> <scene> [from to]
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let exe = ts2::formats::exe::Exe::parse(std::fs::read(&a[1]).unwrap()).unwrap();
    let t = ts2::hud::sprite_table(&exe, a[2].parse().unwrap());
    let from = a.get(3).and_then(|s| usize::from_str_radix(s, 16).ok()).unwrap_or(0);
    let to = a.get(4).and_then(|s| usize::from_str_radix(s, 16).ok()).unwrap_or(0x80);
    for (k, s) in t.iter().enumerate().take(to).skip(from) {
        println!("{k:#04x} {:?}", s.as_ref().map(|s| (s.texture, s.size, &s.cells[..s.cells.len().min(4)])));
    }
}
