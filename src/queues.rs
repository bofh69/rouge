use crate::ReceiveHealthMessage;
use ::crossbeam_channel::*;
use legion::*;

pub(crate) struct ReceiveHealthQueue {
    pub tx: Sender<ReceiveHealthMessage>,
    pub rx: Receiver<ReceiveHealthMessage>,
}

pub(crate) fn register_queues(resources: &mut Resources) {
    let (tx, rx) = unbounded();
    let queue = ReceiveHealthQueue { tx, rx };
    resources.insert(queue);
}
