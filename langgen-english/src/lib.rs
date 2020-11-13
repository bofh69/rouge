//! ECS friendly way of sending correct English texts to players.
//!
//! Example:
//! ```ignore
//!
//! // This is put as a resource or component in the ECS:
//! let mut output_queue = OutputQueue::new(Mutex::new(VecDeque::new()), PLAYER_ENTITY);
//!
//! // In systems you write things like this:
//! output_queue.the_(monster).v(monster, "swing").my(monster, object);
//!
//! // At the end of the tick/frame, run output_queue to actually process the message.
//! // Do it before entities are removed.
//! output_queue.process_queue(&mut entity_adapter);
//! ```
//! The entity_adapter will in the end be asked out output strings like this:
//! ```text
//! The tall goblin swings her club. (Correct pronoun before the object's name).
//! Gandalf the great swings his staff. (No the before proper names).
//! The band of brothers swing something. (The player can't see the object).
//! You swing your fist. (The player won't see herself in third person).
//! ```
//!
//! OutputQueue queues messages until `process_queue` is called.
//! This is needed to avoid lifetime/concurrency issues when one system
//! is running and the EntityAdapter needs to lookup some Entity's Component.
//!
//! The game will have to be architectured with this in mind whenever the results from
//! the EntityAdapter might change between creating a message and calling `process_queue`.
//! Instead of changing the attributes directly, use some temporary marker, have a system
//! that runs `process_queue` and then update the primary components. Same when removing
//! an Entity.
//!
//! To get a working system, provide:
//!
//! * Entity - Copy type that represents all the characters, things and players.
//! * [EntityAdapter](trait.EntityAdapter.html) - looks up info about the entity and outputs text to the player.
//! * [QueueAdapter](trait.QueueAdapter.html) - handles queueing of messages between threads.
//!
//! The crate provide implementations of QueueAdapter for RefCell/Mutex together with LinkedList/VecDeque.

#![warn(missing_docs)] // warn if there is missing docs

mod output_queue;
mod queue_adapter_impls;
mod traits;

pub use output_queue::*;
pub use queue_adapter_impls::*;
pub use traits::*;

/// Messages between OutputBuilder and OutputQueue.process
#[derive(Debug)]
enum Fragment<Entity> {
    The(Entity),
    A(Entity),
    The_(Entity),
    A_(Entity),
    My(Entity, Entity),
    My_(Entity, Entity),

    Is(Entity),  // Send is/are
    Has(Entity), // Send is/are

    Thes(Entity),   // Your/The X's
    Thes_(Entity),  // Your/The X's
    Thess(Entity),  // Yours/The X's
    Thess_(Entity), // Yours/The X's

    Word(Entity),  // Just the name
    Word_(Entity), // Just the name

    Color((i32, i32, i32)),

    VerbRef(Entity, &'static str),
    VerbString(Entity, String),

    TextRef(&'static str),
    Text(String),

    SupressSpace(bool), // Weather to not automatically add a space or not.
    SupressDot(bool), // Weather to automatically add a dot or not.
    Capitalize(bool), // Capitalize the next word or not.

    EndOfLine,
}

/// Encapsulates messages sent via [QueueAdapter](trait.QueueAdapter.html)s.
pub struct FragmentEntry<Entity>(Fragment<Entity>);

/// Represents the gender & uncountability of an Entity
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum Gender {
    Male,
    Female,
    Neuter,
    Plural,
    Uncountable,
}
