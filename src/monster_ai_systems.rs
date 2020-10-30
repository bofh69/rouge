use crate::components::*;
use crate::map::Map;
use crate::{PlayerEntity, PlayerPosition, RunState};
use bracket_lib::prelude::*;
use specs::prelude::*;

pub struct MonsterAiSystem {}

impl<'a> System<'a> for MonsterAiSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, PlayerEntity>,
        ReadExpect<'a, RunState>,
        ReadExpect<'a, PlayerPosition>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, WantsToMelee>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_entity,
            run_state,
            player_pos,
            monster,
            mut position,
            mut viewshed,
            mut wants_to_melee,
            entities,
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _monster, mut pos) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
            let distance = DistanceAlg::Pythagoras
                .distance2d(Point::new(pos.0.x, pos.0.y), player_pos.0.into());
            if distance < 1.5 {
                // Attack goes here
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: player_entity.0,
                        },
                    )
                    .expect("Unable to insert attack");
            } else if viewshed.visible_tiles.contains(&player_pos.0) {
                let path = a_star_search(
                    map.pos_to_idx(*pos) as i32,
                    map.map_pos_to_idx(player_pos.0) as i32,
                    &mut *map,
                );
                if path.success && path.steps.len() > 1 {
                    let old_idx = map.pos_to_idx(*pos);
                    let new_idx = path.steps[1] as usize;
                    let new_pos = map.index_to_point2d(new_idx);
                    if !map.blocked[new_idx] {
                        pos.0.x = new_pos.x;
                        pos.0.y = new_pos.y;
                        map.blocked[old_idx] = false;
                        map.blocked[new_idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}
