mod output_queue;
mod traits;

pub use output_queue::*;
use std::collections::LinkedList;
use std::collections::VecDeque;
pub use traits::*;

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

    SupressDot(bool), // Weather to automatically add a dot or not.
    Capitalize(bool), // Capitalize the next word or not.

    EndOfLine,
}

pub struct FragmentEntry<Entity>(Fragment<Entity>);

pub enum Gender {
    Male,
    Female,
    Neuter,
    Plural,
    Uncountable,
}

impl<Entity> QueueAdapter<Entity> for std::sync::Mutex<VecDeque<FragmentEntry<Entity>>> {
    fn push(&self, frag: FragmentEntry<Entity>) {
        self.lock().unwrap().push_back(frag);
    }

    fn pop(&self) -> Option<FragmentEntry<Entity>> {
        self.lock().unwrap().pop_front()
    }
}

impl<Entity> QueueAdapter<Entity> for std::sync::Mutex<LinkedList<FragmentEntry<Entity>>> {
    fn push(&self, frag: FragmentEntry<Entity>) {
        self.lock().unwrap().push_back(frag);
    }

    fn pop(&self) -> Option<FragmentEntry<Entity>> {
        self.lock().unwrap().pop_front()
    }
}
