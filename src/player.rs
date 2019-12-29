use super::map::Map;
use crate::components::*;
use crate::RunState;
use crate::State;
use rltk::{console, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let map = ecs.fetch::<Map>();

    let combat_stats = ecs.read_storage::<CombatStats>();

    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let entities = ecs.entities();

    let mut ret = RunState::Paused;

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        let (x, y) = (pos.x + delta_x, pos.y + delta_y);
        let idx = map.xy_idx(x, y);

        for potential_target in map.tile_content[idx].iter() {
            let target = combat_stats.get(*potential_target);
            if target.is_some() {
                // Attack it
                console::log(&format!("From Hell's Heart, I stab thee!"));
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return RunState::Running; // So we don't move after attacking
            }
        }

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
