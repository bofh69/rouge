use crate::FragmentEntry;
use crate::Gender;

pub trait EntityAdapter<'a, Entity> {
    fn can_see(&self, who: Entity, obj: Entity) -> bool;
    fn is_me(&self, obj: Entity) -> bool;

    fn gender(&self, obj: Entity) -> Gender;
    fn is_thing(&self, obj: Entity) -> bool;
    fn has_short_proper(&self, obj: Entity) -> bool;
    fn short_name(&self, obj: Entity) -> &'a str;
    fn has_long_proper(&self, obj: Entity) -> bool;
    fn long_name(&self, obj: Entity) -> &'a str;
    fn has_short_plural_proper(&self, obj: Entity) -> bool;
    fn short_plural_name(&self, obj: Entity) -> &'a str;
    fn has_long_plural_proper(&self, obj: Entity) -> bool;
    fn long_plural_name(&self, obj: Entity) -> &'a str;

    fn write_text(&mut self, text: &str);
    fn set_color(&mut self, color: (i32, i32, i32));
    fn done(&mut self);
}

pub trait QueueAdapter<Entity> {
    fn push(&self, f: FragmentEntry<Entity>);
    fn pop(&self) -> Option<FragmentEntry<Entity>>;
}
