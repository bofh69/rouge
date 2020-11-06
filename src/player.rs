use crate::{
    camera::Camera,
    components::{Position, Viewshed, WantsToMelee},
};
use crate::{
    components::{Item, WantsToPickupItem},
    CombatStats,
};
// use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::ecs::Ecs;
use crate::{Direction, PlayerTarget, ScreenPosition};
use crate::{InventoryType, PlayerEntity, PlayerPosition, RunState};
use bracket_lib::prelude::*;
use legion::*;
use std::cmp::{max, min};

pub(crate) fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut Ecs) -> RunState {
    let player_entity = resource_get!(ecs, PlayerEntity).0;

    let mut ret = RunState::AwaitingInput;

    let pos = {
        let map = resource_get!(ecs, Map);

        let (x, y, idx) = {
            let player_entry = ecs.world.entry(player_entity).unwrap();
            let pos = player_entry.into_component::<Position>().unwrap();

            let (x, y) = (pos.0.x + delta_x, pos.0.y + delta_y);
            (x, y, map.xy_to_idx(x, y))
        };

        for potential_target in map.tile_content[idx].iter() {
            if ecs
                .world
                .entry(*potential_target)
                .unwrap()
                .get_component::<CombatStats>()
                .is_ok()
            {
                // Attack it
                let mut gamelog = resource_get_mut!(ecs, GameLog);

                gamelog.log("From Hell's Heart, I stab thee!");
                let mut entry = ecs.world.entry(player_entity).unwrap();
                entry.add_component(WantsToMelee {
                    target: *potential_target,
                });
                return RunState::PlayerTurn;
            }
        }

        if !map.blocked[idx] {
            let mut player_entry = ecs.world.entry(player_entity).unwrap();
            let pos = {
                let pos = player_entry.get_component_mut::<Position>().unwrap();
                pos.0.x = min(map.width - 1, max(0, x));
                pos.0.y = min(map.height - 1, max(0, y));
                pos.0
            };
            let viewshed = player_entry.get_component_mut::<Viewshed>().unwrap();
            viewshed.dirty = true;
            ret = RunState::PlayerTurn;
            Some(pos)
        } else {
            None
        }
    };
    if let Some(pos) = pos {
        // Update player position:
        ecs.resources.insert(PlayerPosition(pos));
    }
    ret
}

pub(crate) fn get_item(ecs: &mut Ecs) -> RunState {
    let player_pos = resource_get!(ecs, PlayerPosition).0;
    let player_entity = resource_get!(ecs, PlayerEntity).0;
    let mut gamelog = resource_get_mut!(ecs, GameLog);

    let mut query = <(Entity, &Position, &Item)>::query();

    let mut found_entity: Option<Entity> = None;

    for (item_entity, pos, _item) in query.iter(&ecs.world) {
        if pos.0 == player_pos {
            found_entity = Some(*item_entity);
            break;
        }
    }
    if let Some(found_entity) = found_entity {
        ecs.world
            .entry(player_entity)
            .unwrap()
            .add_component(WantsToPickupItem {
                collected_by: player_entity,
                item: found_entity,
            });
        RunState::PlayerTurn
    } else {
        gamelog.log("There is nothing you can pickup here!");

        RunState::AwaitingInput
    }
}

fn init_auto_walk(ecs: &Ecs, pos: ScreenPosition) {
    let camera = resource_get!(ecs, Camera);
    let map = resource_get!(ecs, Map);
    let map_pos = camera.transform_screen_pos(pos);
    let idx = map.map_pos_to_idx(map_pos);

    if camera.is_in_view(map_pos) && map.revealed_tiles[idx] {
        let mut target_pos = resource_get_mut!(ecs, PlayerTarget);
        *target_pos = PlayerTarget::Position(map_pos);
    } else {
        let mut target_pos = resource_get_mut!(ecs, PlayerTarget);
        *target_pos = PlayerTarget::None;
    }
}

pub(crate) fn try_auto_walk_player(dir: Direction, ecs: &mut Ecs) {
    let mut player_target = resource_get_mut!(ecs, PlayerTarget);

    *player_target = PlayerTarget::Dir(dir);
}

fn get_auto_walk_dest(ecs: &mut Ecs) -> Option<(i32, i32)> {
    let player_target = resource_get_mut!(ecs, PlayerTarget);
    match *player_target {
        PlayerTarget::Position(map_pos) => {
            let mut map = resource_get_mut!(ecs, Map);
            let player_pos = resource_get_mut!(ecs, PlayerPosition).0;

            let old_idx = map.map_pos_to_idx(player_pos) as i32;
            map.search_only_revealed();
            let path = a_star_search(old_idx, map.map_pos_to_idx(map_pos) as i32, &*map);
            map.search_also_revealed();

            if path.success && path.steps.len() > 1 {
                let new_idx = path.steps[1];
                let new_pos = map.index_to_point2d(new_idx);
                let (dx, dy) = (new_pos.x - player_pos.x, new_pos.y - player_pos.y);
                return Some((dx, dy));
            }
        }
        PlayerTarget::Dir(dir) => {
            let player_pos = resource_get!(ecs, PlayerPosition).0;
            let map = resource_get!(ecs, Map);
            let (dx, dy) = dir.into();
            let new_pos = player_pos + (dx, dy);
            if map.is_exit_valid(new_pos.x, new_pos.y) {
                return Some((dx, dy));
            }
        }
        PlayerTarget::None => (),
    }
    None
}

fn auto_walk(ecs: &mut Ecs) -> RunState {
    let dest = get_auto_walk_dest(ecs);

    if let Some((dx, dy)) = dest {
        if try_move_player(dx, dy, ecs) == RunState::PlayerTurn {
            return RunState::PlayerTurn;
        }
    }
    clear_auto_walk(ecs);
    RunState::AwaitingInput
}

fn clear_auto_walk(ecs: &mut Ecs) {
    let mut player_target = resource_get_mut!(ecs, PlayerTarget);
    *player_target = PlayerTarget::None;
}

pub(crate) fn player_input(ecs: &mut Ecs, ctx: &mut BTerm) -> RunState {
    if ctx.left_click {
        let pos = ctx.mouse_pos();
        let pos = ScreenPosition { x: pos.0, y: pos.1 };
        init_auto_walk(ecs, pos);
        auto_walk(ecs)
    } else if ctx.shift {
        match ctx.key {
            Some(VirtualKeyCode::Q) => {
                return RunState::SaveGame;
            }
            Some(key) => {
                clear_auto_walk(ecs);
                if let Some(dir) = crate::gui::key_to_dir(key) {
                    try_auto_walk_player(dir, ecs)
                }
            }
            _ => (),
        }
        auto_walk(ecs)
    } else {
        match ctx.key {
            Some(key) => {
                clear_auto_walk(ecs);
                match key {
                    // Player movement
                    VirtualKeyCode::H | VirtualKeyCode::Left => try_move_player(-1, 0, ecs),
                    VirtualKeyCode::L | VirtualKeyCode::Right => try_move_player(1, 0, ecs),
                    VirtualKeyCode::K | VirtualKeyCode::Up => try_move_player(0, -1, ecs),
                    VirtualKeyCode::J | VirtualKeyCode::Down => try_move_player(0, 1, ecs),
                    VirtualKeyCode::Y => try_move_player(-1, -1, ecs),
                    VirtualKeyCode::U => try_move_player(1, -1, ecs),
                    VirtualKeyCode::B => try_move_player(-1, 1, ecs),
                    VirtualKeyCode::N => try_move_player(1, 1, ecs),

                    VirtualKeyCode::Space => RunState::PlayerTurn,

                    VirtualKeyCode::Comma => get_item(ecs),

                    VirtualKeyCode::A => RunState::ShowInventory(InventoryType::Apply),
                    VirtualKeyCode::D => RunState::ShowInventory(InventoryType::Drop),

                    VirtualKeyCode::Escape => RunState::ReallyQuit,
                    _ => RunState::AwaitingInput,
                }
            }
            _ => auto_walk(ecs),
        }
    }
}
