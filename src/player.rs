use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::{InventoryType, PlayerEntity, PlayerPosition, RunState, State};
use rltk::{Rltk, VirtualKeyCode};
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

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::H | VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::L | VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::K | VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::J | VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Space => RunState::PlayerTurn,

            VirtualKeyCode::Comma => get_item(&mut gs.ecs),

            VirtualKeyCode::A => RunState::ShowInventory(InventoryType::Apply),
            VirtualKeyCode::D => RunState::ShowInventory(InventoryType::Drop),

            VirtualKeyCode::Escape => RunState::ReallyQuit,
            _ => RunState::AwaitingInput,
        },
    }
}
