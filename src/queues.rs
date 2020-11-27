use crate::messages::*;
use ::crossbeam_channel::*;
use legion::*;

pub(crate) struct Queue<T> {
    pub tx: Sender<T>,
    pub rx: Receiver<T>,
}

pub(crate) type ReceiveHealthQueue = Queue<ReceiveHealthMessage>;
pub(crate) type SufferDamageQueue = Queue<SufferDamageMessage>;
pub(crate) type WantsToMeleeQueue = Queue<WantsToMeleeMessage>;

pub(crate) fn register_queues(resources: &mut Resources) {
    let (tx, rx) = unbounded();
    resources.insert(ReceiveHealthQueue { tx, rx });

    let (tx, rx) = unbounded();
    resources.insert(SufferDamageQueue { tx, rx });

    let (tx, rx) = unbounded();
    resources.insert(WantsToMeleeQueue { tx, rx });
}
