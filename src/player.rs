use crate::{
    components::{CombatStats, Energy, Item, Position, Viewshed},
    messages::{WantsToMeleeMessage, WantsToPickupMessage},
    queues::{WantsToMeleeQueue, WantsToPickupQueue},
    resources::{Camera, Map, OutputQueue, PlayerEntity, PlayerPosition, PlayerTarget},
};
// use crate::components::*;
use crate::ecs::Ecs;
use crate::positions::{Direction, ScreenPosition};
use crate::{InventoryType, RunState};
use bracket_lib::prelude::*;
use legion::*;

pub(crate) fn try_move_player(dir: Direction, ecs: &mut Ecs) -> RunState {
    let (delta_x, delta_y) = dir.into();
    let player_entity = resource_get!(ecs, PlayerEntity).0;

    let mut ret = RunState::AwaitingInput;

    let pos = {
        let map = resource_get_mut!(ecs, Map);

        let (x, y, idx) = {
            let player_entry = ecs.world.entry(player_entity).unwrap();
            let pos = player_entry.into_component::<Position>().unwrap().0;

            let (x, y) = (pos.x + delta_x, pos.y + delta_y);
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
                // let mut output = resource_get_mut!(ecs, OutputQueue);
                // output.s("From Hell's Heart, I stab thee!");
                let wants_to_melee_queue = resource_get!(ecs, WantsToMeleeQueue);
                wants_to_melee_queue.send(WantsToMeleeMessage {
                    attacker: player_entity,
                    target: *potential_target,
                });
                return RunState::EnergylessTick;
            }
        }

        if map.blocked[idx] {
            None
        } else {
            let mut player_entry = ecs.world.entry(player_entity).unwrap();
            let pos = {
                let pos = player_entry.get_component_mut::<Position>().unwrap();
                pos.0.x = x.clamp(0, map.width - 1);
                pos.0.y = y.clamp(0, map.height - 1);
                pos.0
            };
            let viewshed = player_entry.get_component_mut::<Viewshed>().unwrap();
            viewshed.dirty = true;
            player_entry.get_component_mut::<Energy>().unwrap().energy = -100;
            ret = RunState::EnergylessTick;
            Some(pos)
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
    let output = resource_get!(ecs, OutputQueue);

    let mut query = <(Entity, &Position, &Item)>::query();

    let mut found_entity: Option<Entity> = None;

    for (item_entity, pos, _item) in query.iter(&ecs.world) {
        if pos.0 == player_pos {
            found_entity = Some(*item_entity);
            break;
        }
    }
    if let Some(found_entity) = found_entity {
        let queue = resource_get!(ecs, WantsToPickupQueue);
        queue.send(WantsToPickupMessage {
            who: player_entity,
            item: found_entity,
        });
        RunState::EnergylessTick
    } else {
        output.s("There is nothing you can pickup here!");

        RunState::AwaitingInput
    }
}

fn init_auto_walk(ecs: &Ecs, pos: ScreenPosition) {
    let camera = resource_get!(ecs, Camera);
    let map = resource_get!(ecs, Map);
    let map_pos = camera.transform_screen_pos(pos);
    let idx = map.map_pos_to_idx(map_pos);

    let mut target_pos = resource_get_mut!(ecs, PlayerTarget);
    if camera.is_in_view(map_pos) && map.revealed_tiles[idx] {
        *target_pos = PlayerTarget::Position(map_pos);
    } else {
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
                let dir = (new_pos.x - player_pos.x, new_pos.y - player_pos.y);
                return Some(dir);
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
        if try_move_player((dx, dy).into(), ecs) == RunState::EnergylessTick {
            return RunState::EnergylessTick;
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
        let pos = ctx.mouse_point();
        let pos: ScreenPosition = pos.into();
        init_auto_walk(ecs, pos);
        auto_walk(ecs)
    } else if ctx.shift {
        match ctx.key {
            Some(VirtualKeyCode::Q) => {
                return RunState::SaveGame;
            }
            Some(VirtualKeyCode::X) => {
                let target = {
                    let map = resource_get!(ecs, Map);
                    let player_pos = resource_get!(ecs, PlayerPosition).0;
                    map.find_closest_unknown(player_pos)
                };
                if let Some(target) = target {
                    let mut target_pos = resource_get_mut!(ecs, PlayerTarget);
                    *target_pos = PlayerTarget::Position(target);
                }
                return auto_walk(ecs);
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
                    VirtualKeyCode::H | VirtualKeyCode::Left => {
                        try_move_player(Direction::West, ecs)
                    }
                    VirtualKeyCode::L | VirtualKeyCode::Right => {
                        try_move_player(Direction::East, ecs)
                    }
                    VirtualKeyCode::K | VirtualKeyCode::Up => {
                        try_move_player(Direction::North, ecs)
                    }
                    VirtualKeyCode::J | VirtualKeyCode::Down => {
                        try_move_player(Direction::South, ecs)
                    }
                    VirtualKeyCode::Y => try_move_player(Direction::NorthWest, ecs),
                    VirtualKeyCode::U => try_move_player(Direction::NorthEast, ecs),
                    VirtualKeyCode::B => try_move_player(Direction::SouthWest, ecs),
                    VirtualKeyCode::N => try_move_player(Direction::SouthEast, ecs),

                    VirtualKeyCode::Space => {
                        let player_entity = resource_get!(ecs, PlayerEntity).0;
                        let mut player_entry = ecs.world.entry(player_entity).unwrap();
                        player_entry.get_component_mut::<Energy>().unwrap().energy = -50;
                        RunState::EnergylessTick
                    }
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
