use crate::camera::Camera;
use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::{Direction, PlayerTarget, ScreenPosition};
use crate::{InventoryType, PlayerEntity, PlayerPosition, RunState};
use rltk::{Algorithm2D, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let map = ecs.fetch::<Map>();

    let mut gamelog = ecs.write_resource::<GameLog>();

    let combat_stats = ecs.read_storage::<CombatStats>();

    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let entities = ecs.entities();

    let mut ret = RunState::AwaitingInput;

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        let (x, y) = (pos.0.x + delta_x, pos.0.y + delta_y);
        let idx = map.xy_to_idx(x, y);

        for potential_target in map.tile_content[idx].iter() {
            let target = combat_stats.get(*potential_target);
            if target.is_some() {
                // Attack it
                gamelog.log("From Hell's Heart, I stab thee!");
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return RunState::PlayerTurn;
            }
        }

        if !map.blocked[idx] {
            pos.0.x = min(map.width - 1, max(0, x));
            pos.0.y = min(map.height - 1, max(0, y));
            viewshed.dirty = true;
            ret = RunState::PlayerTurn;

            // Update player position:
            let mut ppos = ecs.write_resource::<PlayerPosition>();
            *ppos = PlayerPosition(pos.0);
        }
    }
    ret
}

pub fn get_item(ecs: &mut World) -> RunState {
    let player_pos = ecs.fetch::<PlayerPosition>().0;
    let player_entity = ecs.fetch::<PlayerEntity>();
    let mut gamelog = ecs.write_resource::<GameLog>();
    let positions = ecs.read_storage::<Position>();
    let items = ecs.read_storage::<Item>();
    let entities = ecs.entities();
    let mut wants_to_pickup = ecs.write_storage::<WantsToPickupItem>();

    for (item_entity, pos, _item) in (&entities, &positions, &items).join() {
        if pos.0 == player_pos {
            wants_to_pickup
                .insert(
                    player_entity.0,
                    WantsToPickupItem {
                        collected_by: player_entity.0,
                        item: item_entity,
                    },
                )
                .expect("Add target failed");
            return RunState::PlayerTurn;
        }
    }
    gamelog.log("There is nothing you can pickup here!");

    RunState::AwaitingInput
}

fn init_auto_walk(ecs: &World, pos: ScreenPosition) {
    let camera = ecs.fetch::<Camera>();
    let map = ecs.fetch::<Map>();
    let map_pos = camera.transform_screen_pos(pos);
    let idx = map.map_pos_to_idx(map_pos);

    if camera.is_in_view(map_pos) && map.revealed_tiles[idx] {
        let mut target_pos = ecs.fetch_mut::<PlayerTarget>();
        *target_pos = PlayerTarget::Position(map_pos);
    } else {
        let mut target_pos = ecs.fetch_mut::<PlayerTarget>();
        *target_pos = PlayerTarget::None;
    }
}

pub fn try_auto_walk_player(dir: Direction, ecs: &mut World) {
    let mut player_target = ecs.fetch_mut::<PlayerTarget>();

    *player_target = PlayerTarget::Dir(dir);
}

fn get_auto_walk_dest(ecs: &mut World) -> Option<(i32, i32)> {
    let player_target = ecs.fetch::<PlayerTarget>();
    match *player_target {
        PlayerTarget::Position(map_pos) => {
            let mut map = ecs.fetch_mut::<Map>();
            let player_pos = ecs.fetch::<PlayerPosition>().0;

            let old_idx = map.map_pos_to_idx(player_pos) as i32;
            map.search_only_revealed();
            let path = rltk::a_star_search(old_idx, map.map_pos_to_idx(map_pos) as i32, &mut *map);
            map.search_also_revealed();

            if path.success && path.steps.len() > 1 {
                let new_idx = path.steps[1];
                let new_pos = map.index_to_point2d(new_idx);
                let (dx, dy) = (new_pos.x - player_pos.x, new_pos.y - player_pos.y);
                return Some((dx, dy));
            }
        }
        PlayerTarget::Dir(dir) => {
            let player_pos = ecs.fetch::<PlayerPosition>().0;
            let map = ecs.fetch::<Map>();
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

fn auto_walk(ecs: &mut World) -> RunState {
    let dest = get_auto_walk_dest(ecs);

    if let Some((dx, dy)) = dest {
        if try_move_player(dx, dy, ecs) == RunState::PlayerTurn {
            return RunState::PlayerTurn;
        }
    }
    clear_auto_walk(ecs);
    RunState::AwaitingInput
}

fn clear_auto_walk(ecs: &mut World) {
    let mut player_target = ecs.fetch_mut::<PlayerTarget>();
    *player_target = PlayerTarget::None;
}

pub fn player_input(ecs: &mut World, ctx: &mut Rltk) -> RunState {
    if ctx.left_click {
        let pos = ctx.mouse_pos();
        let pos = ScreenPosition { x: pos.0, y: pos.1 };
        init_auto_walk(ecs, pos);
        auto_walk(ecs)
    } else if ctx.shift {
        clear_auto_walk(ecs);
        if let Some(key) = ctx.key {
            if let Some(dir) = crate::gui::key_to_dir(key) {
                try_auto_walk_player(dir, ecs)
            }
        }
        RunState::AwaitingInput
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
