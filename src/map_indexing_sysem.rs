use crate::components::*;
use crate::map::Map;
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, BlocksTile>,
        ReadStorage<'a, Position>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, blocks, positions, entities) = data;

        map.populate_blocked();
        map.clear_content_index();

        for (entity, _block, pos) in (&entities, &blocks, &positions).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            map.blocked[idx] = true;
            map.tile_content[idx].push(entity);
        }
    }
}
