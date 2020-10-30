use super::map::Map;
use crate::components::Monster;
use crate::components::Player;
use crate::{MapPosition, PlayerTarget, Position, Viewshed};
use bracket_lib::prelude::field_of_view;
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        WriteExpect<'a, PlayerTarget>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, mut player_target, entities, mut viewshed, pos, player, monsters) = data;

        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.visible_tiles.clear();

                /* The points here are in map space */
                viewshed.visible_tiles = field_of_view(pos.0.into(), viewshed.range, &*map)
                    .iter()
                    .filter(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height)
                    .map(|p| MapPosition { x: p.x, y: p.y })
                    .collect();

                // If this is the player, reveal what they can see
                let p: Option<&Player> = player.get(ent);
                if p.is_some() {
                    for vt in map.visible_tiles.iter_mut() {
                        *vt = false;
                    }
                    let mut curr_target = *player_target;

                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.map_pos_to_idx(*vis);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                        if PlayerTarget::None != curr_target {
                            for ent in map.tile_content[idx].iter() {
                                if monsters.get(*ent).is_some() {
                                    curr_target = PlayerTarget::None
                                }
                            }
                        }
                    }
                    *player_target = curr_target;
                }

                viewshed.dirty = false;
            }
        }
    }
}
