use crate::FragmentEntry;
use crate::Gender;

/// OutputQueue's interface to the game's ECS Components and
/// the games output routines.
///
/// Entity is a Copy type identifying a player, character or thing.
pub trait EntityAdapter<Entity>
where
    Entity: Copy,
{
    /// Can who see obj?
    fn can_see(&self, who: Entity, obj: Entity) -> bool;
    /// Is obj me?
    fn is_me(&self, obj: Entity) -> bool;

    /// Returns obj's gender (which is also plural & uncountable for things).
    fn gender(&self, obj: Entity) -> Gender;

    /// Is obj a thing?
    fn is_thing(&self, obj: Entity) -> bool;

    /// Does obj have a proper name (ie Thomas)?
    fn has_short_proper(&self, obj: Entity) -> bool;

    /// The objects short name. Typically a single word, like "apple".
    fn append_short_name(&self, obj: Entity, s: &mut String);

    /// Is the object's long name a proper name (ie Ada Lovecraft)?
    fn has_long_proper(&self, obj: Entity) -> bool;

    /// The object's long name, ie "red apple".
    fn append_long_name(&self, obj: Entity, s: &mut String);

    /// The objects short, plural name. Typically a single word, like "apples".
    fn append_short_plural_name(&self, obj: Entity, s: &mut String);

    /// The object's long, plural name, ie "red apples".
    fn append_long_plural_name(&self, obj: Entity, s: &mut String);

    /// Writes the given text to the player.
    fn write_text(&mut self, text: &str);

    /// Sets the output color to the given color.
    fn set_color(&mut self, color: (u8, u8, u8));

    /// Called when all the text for the current line/sentance has been
    /// written with write_text.
    ///
    /// Restores the color to the default color.
    fn done(&mut self);
}

/// OutputQueue's interface against queues.
pub trait QueueAdapter<Entity> {
    /// Pushes an entry to the end of the queue.
    fn push(&self, f: FragmentEntry<Entity>);

    /// Pops the next entry from the queue, or None if there are none.
    fn pop(&self) -> Option<FragmentEntry<Entity>>;
}
