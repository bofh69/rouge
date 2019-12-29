use crate::components::*;
use crate::map::Map;
use rltk::{console, Algorithm2D, Point};
use specs::prelude::*;

pub struct MonsterAiSystem {}

impl<'a> System<'a> for MonsterAiSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, mut viewshed, monster, name, mut position) = data;

        for (mut viewshed, _monster, name, mut pos) in
            (&mut viewshed, &monster, &name, &mut position).join()
        {
            let distance =
                rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                // Attack goes here
                console::log(&format!("{} shouts insults", name.name));
            } else if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!("{} shouts insults", name.name));
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
