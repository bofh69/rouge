use crate::components::*;
use crate::map::Map;
use rltk::{Algorithm2D, Point};
use specs::prelude::*;

pub struct MonsterAiSystem {}

impl<'a> System<'a> for MonsterAiSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, WantsToMelee>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            monster,
            mut position,
            mut viewshed,
            mut wants_to_melee,
            entities,
        ) = data;

        for (entity, mut viewshed, _monster, mut pos) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
            let distance =
                rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                // Attack goes here
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *player_entity,
                        },
                    )
                    .expect("Unable to insert attack");
            } else if viewshed.visible_tiles.contains(&*player_pos) {
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
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
