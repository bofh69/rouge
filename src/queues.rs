use crate::messages::*;
use ::crossbeam_channel::*;
use legion::*;

pub(crate) struct Queue<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
}

pub(crate) type ReceiveHealthQueue = Queue<ReceiveHealthMessage>;
pub(crate) type RemoveItemQueue = Queue<RemoveItemMessage>;
pub(crate) type SufferDamageQueue = Queue<SufferDamageMessage>;
pub(crate) type WantsToDropQueue = Queue<WantsToDropMessage>;
pub(crate) type WantsToMeleeQueue = Queue<WantsToMeleeMessage>;
pub(crate) type WantsToUseQueue = Queue<WantsToUseMessage>;

pub(crate) fn register_queues(resources: &mut Resources) {
    resources.insert(ReceiveHealthQueue::new());
    resources.insert(RemoveItemQueue::new());
    resources.insert(SufferDamageQueue::new());
    resources.insert(WantsToDropQueue::new());
    resources.insert(WantsToMeleeQueue::new());
    resources.insert(WantsToUseQueue::new());
}

impl<T> Queue<T> {
    fn new() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }

    pub(crate) fn send(&self, msg: T) {
        self.tx.send(msg).expect("Queue full?");
    }

    pub(crate) fn try_iter(&self) -> crossbeam_channel::TryIter<T> {
        self.rx.try_iter()
    }
}
