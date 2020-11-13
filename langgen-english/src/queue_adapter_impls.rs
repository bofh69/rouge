use crate::{FragmentEntry, QueueAdapter};
use ::std::cell::RefCell;
use ::std::collections::{LinkedList, VecDeque};
use ::std::sync::Mutex;

/// A suitable QueueAdapter when different threads read/write to the same Queue.
///
/// Note: there is no locking employed for the whole sentance so
/// different systems' output can interleave.
impl<Entity> QueueAdapter<Entity> for Mutex<VecDeque<FragmentEntry<Entity>>> {
    fn push(&self, frag: FragmentEntry<Entity>) {
        self.lock().unwrap().push_back(frag);
    }

    fn pop(&self) -> Option<FragmentEntry<Entity>> {
        self.lock().unwrap().pop_front()
    }
}

/// A suitable QueueAdapter when different threads read/write to the same Queue.
///
/// Note: there is no locking employed for the whole sentance so
/// different systems' output can interleave.
impl<Entity> QueueAdapter<Entity> for Mutex<LinkedList<FragmentEntry<Entity>>> {
    fn push(&self, frag: FragmentEntry<Entity>) {
        self.lock().unwrap().push_back(frag);
    }

    fn pop(&self) -> Option<FragmentEntry<Entity>> {
        self.lock().unwrap().pop_front()
    }
}

/// A suitable QueueAdapter when only one thread can read/write to the same Queue.
impl<Entity> QueueAdapter<Entity> for RefCell<VecDeque<FragmentEntry<Entity>>> {
    fn push(&self, frag: FragmentEntry<Entity>) {
        self.borrow_mut().push_back(frag);
    }

    fn pop(&self) -> Option<FragmentEntry<Entity>> {
        self.borrow_mut().pop_front()
    }
}

/// A suitable QueueAdapter when only one thread can read/write to the same Queue.
impl<Entity> QueueAdapter<Entity> for RefCell<LinkedList<FragmentEntry<Entity>>> {
    fn push(&self, frag: FragmentEntry<Entity>) {
        self.borrow_mut().push_back(frag);
    }

    fn pop(&self) -> Option<FragmentEntry<Entity>> {
        self.borrow_mut().pop_front()
    }
}
