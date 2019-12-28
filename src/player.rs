use super::map::{xy_idx, TileType};
use super::{Player, Position, State};
use rltk::{Rltk, VirtualKeyCode};
use std::cmp::{max, min};

use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let (x, y) = (pos.x + delta_x, pos.y + delta_y);
        if map[xy_idx(x, y)] != TileType::Wall {
            pos.x = min(super::map::MAP_WIDTH - 1, max(0, x));
            pos.y = min(super::map::MAP_HEIGHT - 1, max(0, y));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::H | VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::L | VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::K | VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::J | VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::P => gs.paused = !gs.paused,
            VirtualKeyCode::Escape => ctx.quit(),
            _ => {}
        },
    }
}
