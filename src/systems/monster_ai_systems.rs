use crate::components::{Energy, Monster, Position, Viewshed, WantsToMelee};
use crate::resources::{Map, PlayerEntity, PlayerPosition};
use crate::RunState;
use bracket_lib::prelude::*;
use legion::{Entity, IntoQuery};

// TODO: Change to proper monster_ai_system
pub(crate) fn monster_ai_system(ecs: &mut crate::ecs::Ecs) {
    let rs = *resource_get!(ecs, RunState);
    if rs != RunState::Tick && rs != RunState::EnergylessTick {
        return;
    }
    let mut map = resource_get_mut!(ecs, Map);
    let player_pos = resource_get!(ecs, PlayerPosition).0;
    let player_entity = resource_get!(ecs, PlayerEntity).0;

    let mut cb = legion::systems::CommandBuffer::new(&ecs.world);

    let mut ready: Vec<_> = <(Entity, &mut Viewshed, &mut Position, &mut Energy)>::query()
        .filter(legion::query::component::<Monster>())
        .iter_mut(&mut ecs.world)
        .filter(|(_, _, _, energy)| energy.energy >= 0)
        .collect();

    ready.sort_by_key(|(_, _, _, energy)| -energy.energy);

    for (entity, mut viewshed, mut pos, mut energy) in ready {
        let distance =
            DistanceAlg::Manhattan.distance2d(Point::new(pos.0.x, pos.0.y), player_pos.into());
        if distance < 1.5 {
            // Attack goes here
            cb.add_component(
                *entity,
                WantsToMelee {
                    target: player_entity,
                },
            );
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
                let new_idx = path.steps[1] as usize;
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
    cb.flush(&mut ecs.world);
}
