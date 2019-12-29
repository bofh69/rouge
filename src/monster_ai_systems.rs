use crate::components::*;
use crate::map::{Map, TileType};
use rltk::{console, RandomNumberGenerator};
use specs::prelude::*;
use std::cmp::{max, min};

pub struct MonsterAiSystem {}

fn try_move_monster(delta_x: i32, delta_y: i32, vs: &mut Viewshed, map: &Map, pos: &mut Position) {
    let (x, y) = (pos.x + delta_x, pos.y + delta_y);
    if map.tiles[map.xy_idx(x, y)] != TileType::Wall {
        pos.x = min(map.width - 1, max(0, x));
        pos.y = min(map.height - 1, max(0, y));
        vs.dirty = true;
    }
}

impl<'a> System<'a> for MonsterAiSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadExpect<'a, rltk::Point>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Viewshed>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, ppos, monsters, names, mut positions, mut viewsheds) = data;

        let mut rnd = RandomNumberGenerator::new();

        for (_mon, name, vs, pos) in (&monsters, &names, &mut viewsheds, &mut positions).join() {
            if vs.visible_tiles.contains(&*ppos) {
                match rnd.roll_dice(1, 8) {
                    1 => try_move_monster(-1, 0, vs, &map, pos),
                    2 => try_move_monster(1, 0, vs, &map, pos),
                    3 => try_move_monster(0, -1, vs, &map, pos),
                    4 => try_move_monster(0, 1, vs, &map, pos),
                    5 => console::log(format!("The {} shouts after you!", name.name)),
                    _ => (),
                }
            }
        }
    }
}
