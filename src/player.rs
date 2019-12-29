use super::map::Map;
use super::{Player, Position, State};
use crate::components::Viewshed;
use crate::RunState;
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut ret = RunState::Paused;

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let (x, y) = (pos.x + delta_x, pos.y + delta_y);
        let idx = map.xy_idx(x, y);
        if !map.blocked[idx] {
            pos.x = min(map.width - 1, max(0, x));
            pos.y = min(map.height - 1, max(0, y));
            viewshed.dirty = true;
            ret = RunState::Running;

            // Update player position:
            let mut ppos = ecs.write_resource::<rltk::Point>();
            ppos.x = x;
            ppos.y = y;
        }
    }
    ret
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => RunState::Paused, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::H | VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::L | VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::K | VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::J | VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::Space => RunState::Running,

            VirtualKeyCode::Escape => {
                ctx.quit();
                RunState::Paused
            }
            _ => RunState::Paused,
        },
    }
}
