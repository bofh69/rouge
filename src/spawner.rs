use crate::components::*;
use crate::map::MAP_WIDTH;
use crate::ecs::Ecs;
use crate::MapPosition;
use bracket_lib::prelude::*;
use legion::*;

pub(crate) const MAX_MONSTERS: i32 = 5;
pub(crate) const MAX_ITEMS: i32 = 3;

pub(crate) fn player(ecs: &mut Ecs, player_x: i32, player_y: i32) -> Entity {
    ecs.world.push((
        Position(MapPosition {
            x: player_x,
            y: player_y,
        }),
        Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
            render_order: 0,
        },
        Player {},
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        },
        Name {
            name: "Player".to_string(),
        },
        CombatStats {
            hp: 30,
            max_hp: 30,
            power: 5,
            defense: 2,
        },
    ))
}

/// Spawns a random monster at a given location
pub(crate) fn random_monster(ecs: &mut Ecs, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.resources.get_mut::<RandomNumberGenerator>().unwrap();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => lamotte(ecs, x, y),
        _ => janouch(ecs, x, y),
    }
}

fn lamotte(ecs: &mut Ecs, x: i32, y: i32) {
    monster(ecs, x, y, to_cp437('l'), "Lamotte");
}
fn janouch(ecs: &mut Ecs, x: i32, y: i32) {
    monster(ecs, x, y, to_cp437('j'), "Janouch");
}

fn monster<S: ToString>(ecs: &mut Ecs, x: i32, y: i32, glyph: u16, name: S) {
    ecs.world.push((
        Position(MapPosition { x, y }),
        Renderable {
            glyph,
            fg: RGB::named(RED),
            bg: RGB::named(BLACK),
            render_order: 1,
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        },
        Monster {},
        Name {
            name: name.to_string(),
        },
        BlocksTile {},
        CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        },
    ));
}

/// Spawns a random monster at a given location
pub(crate) fn random_item(ecs: &mut Ecs, x: i32, y: i32) {
    let roll = {
        let mut rng = ecs.resources.get_mut::<RandomNumberGenerator>().unwrap();
        rng.roll_dice(1, 4)
    };
    match roll {
        1 => health_potion(ecs, x, y),
        2 => magic_missile_scroll(ecs, x, y),
        3 => fireball_scroll(ecs, x, y),
        _ => ball(ecs, x, y),
    }
}

fn health_potion(ecs: &mut Ecs, x: i32, y: i32) {
    ecs.world.push((
        Position(MapPosition { x, y }),
        Renderable {
            glyph: to_cp437('¡'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2,
        },
        Name {
            name: "Health Potion".to_string(),
        },
        Item {},
        Consumable {},
        HealthProvider { heal_amount: 8 },
    ));
}

fn ball(ecs: &mut Ecs, x: i32, y: i32) {
    ecs.world.push((
        Position(MapPosition { x, y }),
        Renderable {
            glyph: to_cp437('*'),
            fg: RGB::named(PURPLE),
            bg: RGB::named(BLACK),
            render_order: 2,
        },
        Name {
            name: "Ball".to_string(),
        },
        Item {},
    ));
}

fn magic_missile_scroll(ecs: &mut Ecs, x: i32, y: i32) {
    ecs.world.push((
        Position(MapPosition { x, y }),
        Renderable {
            glyph: to_cp437('?'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        },
        Name {
            name: "Magic Missile Scroll".to_string(),
        },
        Item {},
        Consumable {},
        Ranged { range: 6 },
        InflictsDamage { damage: 8 },
    ));
}

fn fireball_scroll(ecs: &mut Ecs, x: i32, y: i32) {
    ecs.world.push((
        Position(MapPosition { x, y }),
        Renderable {
            glyph: to_cp437('?'),
            fg: RGB::named(ORANGE),
            bg: RGB::named(BLACK),
            render_order: 2,
        },
        Name {
            name: "Fireball Scroll".to_string(),
        },
        Item {},
        Consumable {},
        Ranged { range: 6 },
        InflictsDamage { damage: 20 },
        AreaOfEffect { radius: 3 },
    ));
}

/// Fills a room with stuff!
pub(crate) fn spawn_room(ecs: &mut Ecs, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.resources.get_mut::<RandomNumberGenerator>().unwrap();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));
                let idx = ((y * MAP_WIDTH) + x) as usize;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));
                let idx = (y * MAP_WIDTH + x) as usize;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAP_WIDTH as usize;
        let y = *idx / MAP_WIDTH as usize;
        random_monster(ecs, x as i32, y as i32);
    }
    // Actually spawn the items
    for idx in item_spawn_points.iter() {
        let x = *idx % MAP_WIDTH as usize;
        let y = *idx / MAP_WIDTH as usize;
        random_item(ecs, x as i32, y as i32);
    }
}
