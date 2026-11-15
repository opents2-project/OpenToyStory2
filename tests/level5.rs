//! Level 5's script. Skipped unless TS2_GAME_DIR is set.

use glam::Vec3;
use std::{env, fs, path::PathBuf};
use ts2::creatures::Creatures;
use ts2::formats::{exe, leveldat, ngn, packets, terrain};
use ts2::game::{Player, World};
use ts2::script::Script;

struct L2 {
    world: World,
    specials: ts2::specials::Specials,
    script: Box<dyn Script>,
    boxes: ts2::boxes::Boxes,
    cr: Creatures,
}

fn load() -> Option<L2> {
    let game = env::var_os("TS2_GAME_DIR").map(PathBuf::from)?;
    let dir = game.join("data/level05");
    let e = exe::Exe::parse(fs::read(game.join("toy2.exe")).unwrap()).unwrap();
    let n = ngn::parse(&fs::read(dir.join("level.ngn")).unwrap()).unwrap();
    let t = terrain::parse(&fs::read(dir.join("TERRAIN.ALL")).unwrap()).unwrap();
    let lists = leveldat::parse(&fs::read(dir.join("level.dat")).unwrap()).unwrap();
    let world = World::from_terrain(&t);
    let mut specials = ts2::specials::Specials::new(&n);
    let random = fs::read(game.join("data/rand.dat")).unwrap();
    let script = ts2::script::for_level(5, &world, &mut specials, Some(&e), &lists, Some(random.clone())).unwrap();
    let boxes = ts2::boxes::Boxes::new(5, &lists, Some(&e), &world, &mut specials);
    let pk = packets::parse(&fs::read(dir.join("level.raw")).unwrap()).unwrap();
    let spawns: Vec<_> = pk.iter().flat_map(packets::spawns).collect();
    let mut cr = Creatures::new(&spawns, Some(&e), random);
    cr.load_shapes(&game.join("data"));
    Some(L2 { world, specials, script, boxes, cr })
}

/// The gliding platforms move along their lists.
#[test]
fn platforms_glide() {
    let Some(mut l) = load() else { return };
    let mut p = Player::new(Vec3::new(0.0, -100000.0, 0.0));
    l.script.step(&l.world, &mut l.specials, &mut p, &l.boxes);
    let start: Vec<Vec3> = [8, 5, 6, 7, 9, 10, 11, 12].iter().map(|&m| l.world.mover_pos(m).unwrap()).collect();
    let mut far = [0.0f32; 8];
    for _ in 0..600 {
        l.script.step(&l.world, &mut l.specials, &mut p, &l.boxes);
        for (k, m) in [8, 5, 6, 7, 9, 10, 11, 12].into_iter().enumerate() {
            far[k] = far[k].max((l.world.mover_pos(m).unwrap() - start[k]).length());
        }
    }
    eprintln!("furthest: {far:?}");
    assert!(far.iter().all(|&d| d > 100.0));
}

/// In room 2 the clown's challenge starts; after it the bar shows.
#[test]
fn clown_challenge_in_room_2() {
    let Some(mut l) = load() else { return };
    let c = l.cr.slots[3].pos;
    assert_eq!(l.cr.slots[3].id, 0x20, "the clown in slot 3");
    let at = Vec3::from(c.map(|v| v as f32 / 32.0));
    eprintln!("clown at {at:?} room {:?}", l.world.room_at(at));
    let mut p = Player::new(at);
    p.on_floor = true;
    for _ in 0..3 {
        l.cr.tick(&l.world, p.pos_obj(), p.pos_obj());
        l.script.step_creatures(&l.world, &mut l.specials, &p, &mut l.cr, 0);
    }
    assert_eq!(l.cr.boss.state, 1);
    l.script.cutscene = None;
    for _ in 0..5 {
        l.cr.tick(&l.world, p.pos_obj(), p.pos_obj());
        l.script.step_creatures(&l.world, &mut l.specials, &p, &mut l.cr, 0);
    }
    assert_eq!(l.cr.boss.state, 2);
    assert!(l.script.boss_hud);
}

/// The five bones (specials 0x61..0x65) can each be collected once the challenge starts.
#[test]
fn bones_all_collectable() {
    let Some(mut l) = load() else { return };
    let game = env::var_os("TS2_GAME_DIR").map(PathBuf::from).unwrap();
    let e = exe::Exe::parse(fs::read(game.join("toy2.exe")).unwrap()).unwrap();
    let dat = fs::read(game.join("data/level05/level.dat")).unwrap();
    let mut pk = ts2::pickups::Pickups::new(5, &dat, Some(&e)).unwrap();
    let mut p = Player::new(Vec3::new(0.0, -100000.0, 0.0));
    l.script.step(&l.world, &mut l.specials, &mut p, &l.boxes);
    eprintln!("challenge char {:?} id {:#x}", l.cr.slots[0x12].pos, l.cr.slots[0x12].id);
    l.cr.slots[0x12].flags |= 0x200;
    l.script.step_creatures(&l.world, &mut l.specials, &p, &mut l.cr, 0);
    l.script.cutscene = None;
    if std::mem::take(&mut l.script.collect_reset) {
        pk.collectables = 0;
    }
    if let Some(s) = l.script.show_collectables.take() {
        pk.show_collectables(s);
    }
    let bones: Vec<_> = pk.items.iter().filter(|i| (0x61..=0x65).contains(&i.id)).map(|i| (i.id, i.pos, i.kind, i.removed, i.scale)).collect();
    eprintln!("{bones:?}");
    for (id, pos, ..) in bones {
        let at = [pos[0] as f32, pos[1] as f32 + 230.0, pos[2] as f32];
        for _ in 0..3 {
            pk.tick(at, at);
        }
        eprintln!("bone {id:#x}: collected so far {}", pk.collectables);
    }
    assert_eq!(pk.collectables, 5);
}

/// 0x41E880: the zip wires are live. A spark (particle 0x4C, life 0x80) sets off along the
/// next wire every 0x14 + 1 ticks, at a 64th of the wire a tick (halved when made).
#[test]
fn sparks_run_along_the_zip_wires() {
    let Some(mut l) = load() else { return };
    let p = Player::new(Vec3::ZERO);
    let mut sparks = Vec::new();
    for _ in 0..105 {
        l.script
            .step_creatures(&l.world, &mut l.specials, &p, &mut l.cr, 0);
        sparks.extend(l.script.lasting.drain(..));
    }
    assert_eq!(sparks.len(), 5, "one every 21 ticks");
    assert!(
        sparks
            .iter()
            .all(|(r, life)| r.typ == 0x4C && *life == 0x80 && r.vel != [0; 3])
    );
}

/// Buzz pushes `k`'s box along its first segment for `ticks`; how far it went (level units).
fn push(l: &mut L2, k: usize, ticks: usize) -> f32 {
    let b = l.boxes.boxes[k].clone();
    let lists = leveldat::parse(&fs::read(PathBuf::from(env::var_os("TS2_GAME_DIR").unwrap()).join("data/level05/level.dat")).unwrap()).unwrap();
    let path = lists.iter().find(|x| x.kind == b.path).unwrap().points.clone();
    let (p0, p1) = (path[0], path[1]);
    let d = glam::Vec2::new((p1[0] - p0[0]) as f32, (p1[2] - p0[2]) as f32).normalize();
    let start = Vec3::new(p0[0] as f32 - d.x * 300.0, p0[1] as f32 - 50.0, p0[2] as f32 - d.y * 300.0);
    let floor = l.world.floor_below(start.x, start.z, start.y - 200.0).unwrap_or(start.y);
    let mut p = Player::new(Vec3::new(start.x, floor, start.z));
    p.on_floor = true;
    for _ in 0..ticks {
        l.boxes.tick(&mut p, &l.world, &mut l.specials);
        l.script.step(&l.world, &mut l.specials, &mut p, &l.boxes);
        p.tick_pad(&l.world, [d.x, d.y], 0, false, false, false);
    }
    let now = l.boxes.boxes[k].pos;
    ((now[0] - b.pos[0]) as f32).hypot((now[2] - b.pos[2]) as f32) / 32.0
}

/// The gate holds the crate where it is while it stands; pushed, it tips over to lie flat.
#[test]
fn the_gate_falls_when_pushed_and_holds_the_crate_till_then() {
    let Some(mut l) = load() else { return };
    let mut p = Player::new(Vec3::new(0.0, -100000.0, 0.0));
    let standing = l.specials.angles(2);
    for _ in 0..10 {
        l.script.step(&l.world, &mut l.specials, &mut p, &l.boxes);
    }
    assert_eq!(l.specials.angles(2), standing, "nobody pushed it");
    assert_eq!(l.boxes.progress(1), 0, "the crate stays put");
    push(&mut l, 2, 120);
    let fallen = l.specials.angles(2).unwrap();
    assert_eq!((fallen[2] - standing.unwrap()[2]) & 0xFFF, 0x400, "a quarter turn");
}
