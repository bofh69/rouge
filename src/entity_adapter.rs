use crate::components::{Item, Name, Position, Viewshed};
use crate::resources::GameLog;
use ::langgen_english::*;
use ::legion::*;
use legion::world::SubWorld;

pub(crate) struct EntityAdapterImpl<'a, 'w> {
    world: &'a mut SubWorld<'w>,
    gamelog: &'a mut GameLog,
    player: Entity,
}

impl<'a, 'w> EntityAdapterImpl<'a, 'w> {
    pub(crate) fn new(
        world: &'a mut SubWorld<'w>,
        gamelog: &'a mut GameLog,
        player: Entity,
    ) -> Self {
        Self {
            world,
            gamelog,
            player,
        }
    }
}

impl EntityAdapter<Entity> for EntityAdapterImpl<'_, '_> {
    fn is_me(&self, who: Entity) -> bool {
        who == self.player
    }
    fn can_see(&self, who: Entity, obj: Entity) -> bool {
        match self.world.entry_ref(who) { Ok(who_entry) => {
            match self.world.entry_ref(obj) { Ok(obj_entry) => {
                match obj_entry.get_component::<Position>() { Ok(pos) => {
                    match who_entry.get_component::<Viewshed>() { Ok(vs) => {
                        vs.visible_tiles.contains(&pos.0)
                    } _ => {
                        true
                    }}
                } _ => {
                    true
                }}
            } _ => {
                true
            }}
        } _ => {
            true
        }}
    }
    fn gender(&self, _: Entity) -> langgen_english::Gender {
        // TODO:
        langgen_english::Gender::Male
    }
    fn is_thing(&self, who: Entity) -> bool {
        self.world
            .entry_ref(who).is_ok_and(|e| e.get_component::<Item>().is_ok())
    }
    fn has_short_proper(&self, who: Entity) -> bool {
        self.world.entry_ref(who).is_ok_and(|e| {
            e.get_component::<Name>().is_ok_and(|n| n.proper_name)
        })
    }
    fn append_short_name(&self, who: Entity, s: &mut String) {
        if let Ok(entry) = self.world.entry_ref(who) {
            if let Ok(name) = entry.get_component::<Name>() {
                s.push_str(&name.name);
                return;
            }
        }
        s.push_str("<unknown>");
    }
    fn has_long_proper(&self, _: Entity) -> bool {
        todo!()
    }
    fn append_long_name(&self, who: Entity, s: &mut String) {
        // TODO
        if let Ok(entry) = self.world.entry_ref(who) {
            if let Ok(name) = entry.get_component::<Name>() {
                s.push_str(&name.name);
                return;
            }
        }
        s.push_str("<unknown>");
    }
    fn append_short_plural_name(&self, _: Entity, _s: &mut String) {
        todo!()
    }
    fn append_long_plural_name(&self, _: Entity, _s: &mut String) {
        todo!()
    }
    fn write_text(&mut self, text: &str) {
        self.gamelog.write_text(text);
    }
    fn set_color(&mut self, color: (u8, u8, u8)) {
        self.gamelog.set_color(color);
    }
    fn done(&mut self) {
        self.gamelog.end_of_line();
    }
}
