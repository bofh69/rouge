use crate::components::{Energy, Monster, Position, Viewshed};
use crate::messages::WantsToMeleeMessage;
use crate::queues::WantsToMeleeQueue;
use crate::resources::{Map, PlayerEntity, PlayerPosition};
use crate::RunState;
use bracket_lib::prelude::*;
use legion::{system, world::SubWorld, Entity, IntoQuery};

#[system]
#[read_component(Monster)]
#[write_component(Viewshed)]
#[write_component(Position)]
#[write_component(Energy)]
pub(crate) fn monster_ai(
    world: &mut SubWorld,
    #[resource] rs: &RunState,
    #[resource] map: &mut Map,
    #[resource] player_pos: &mut PlayerPosition,
    #[resource] player_entity: &mut PlayerEntity,
    #[resource] wants_to_melee_queue: &WantsToMeleeQueue,
) {
    if *rs != RunState::Tick && *rs != RunState::EnergylessTick {
        return;
    }

    let player_pos = player_pos.0;
    let player_entity = player_entity.0;

    let mut ready: Vec<_> = <(Entity, &mut Viewshed, &mut Position, &mut Energy)>::query()
        .filter(legion::query::component::<Monster>())
        .iter_mut(world)
        .filter(|(_, _, _, energy)| energy.energy >= 0)
        .collect();

    ready.sort_by_key(|(_, _, _, energy)| -energy.energy);

    for (entity, viewshed, pos, energy) in ready {
        let distance =
            DistanceAlg::Chebyshev.distance2d(Point::new(pos.0.x, pos.0.y), player_pos.into());
        if distance < 1.5 {
            // Attack goes here
            wants_to_melee_queue.send(WantsToMeleeMessage {
                attacker: *entity,
                target: player_entity,
            });
        } else if viewshed.visible_tiles.contains(&player_pos) {
            let path = a_star_search(
                map.pos_to_idx(*pos) as i32,
                map.map_pos_to_idx(player_pos) as i32,
                &*map,
            );
            if path.success && path.steps.len() > 1 {
                // TODO: Move to some action system.
                // Walk towards player:
                energy.energy = -100;
                let old_idx = map.pos_to_idx(*pos);
                let new_idx = path.steps[1];
                let new_pos = map.index_to_point2d(new_idx);
                if !map.blocked[new_idx] {
                    pos.0.x = new_pos.x;
                    pos.0.y = new_pos.y;
                    map.blocked[old_idx] = false;
                    map.blocked[new_idx] = true;
                    map.dangerous[old_idx] = false;
                    map.dangerous[new_idx] = true;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
