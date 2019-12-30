use crate::components::*;
use crate::map::Map;
use crate::PlayerEntity;
use crate::PlayerPosition;
use crate::RunState;
use rltk::{Algorithm2D, Point};
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
            let distance = rltk::DistanceAlg::Pythagoras
                .distance2d(Point::new(pos.x, pos.y), (*player_pos).into());
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
            } else if viewshed.visible_tiles.contains(&(*player_pos).into()) {
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.0, player_pos.1) as i32,
                    &mut *map,
                );
                if path.success && path.steps.len() > 1 {
                    let old_idx = map.xy_idx(pos.x, pos.y);
                    let new_pos = map.index_to_point2d(path.steps[1]);
                    let new_idx = map.xy_idx(new_pos.x, new_pos.y);
                    if !map.blocked[new_idx] {
                        pos.x = new_pos.x;
                        pos.y = new_pos.y;
                        map.blocked[old_idx] = false;
                        map.blocked[new_idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}
