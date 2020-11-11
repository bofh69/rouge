mod common;

use common::*;
use langgen_english::*;
use std::collections::vec_deque::VecDeque;
use std::sync::Mutex;

type DebQueue = Mutex<VecDeque<FragmentEntry<EntNum>>>;

type DebOutputQueue<'a> =
    langgen_english::OutputQueue<'a, common::EntNum, DebugEntityAdapter, DebQueue>;

#[test]
fn output_eva() {
    let mut oq = DebOutputQueue::new(Mutex::new(VecDeque::new()));
    oq.the(EntNum(1));
    let mut dea = DebugEntityAdapter {
        buffer: String::new(),
    };

    oq.process_queue(&mut dea);

    assert_eq!("Eva.\n", dea.buffer);
}

#[test]
fn output_the_apple() {
    let mut oq = DebOutputQueue::new(Mutex::new(VecDeque::new()));
    oq.the(EntNum(8 + 0));

    let mut dea = DebugEntityAdapter {
        buffer: String::new(),
    };
    oq.process_queue(&mut dea);

    assert_eq!("The apple.\n", dea.buffer);
}
