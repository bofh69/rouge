mod common;

use common::*;
use langgen_english::*;
use std::collections::vec_deque::VecDeque;
use std::sync::Mutex;

type DebQueue = Mutex<VecDeque<FragmentEntry<i32>>>;

type DebOutputQueue<'a> = langgen_english::OutputQueue<'a, i32, DebugEntityAdapter, DebQueue>;

#[test]
fn output_kim() {
    let mut oq = DebOutputQueue::new(Mutex::new(VecDeque::new()), 0);
    oq.the(1);
    let mut dea = DebugEntityAdapter::new();

    oq.process_queue(&mut dea);

    assert_eq!("Kim.\n", dea.buffer);
}

#[test]
fn output_the_apple() {
    let mut oq = DebOutputQueue::new(Mutex::new(VecDeque::new()), 16);
    oq.the(8);

    let mut dea = DebugEntityAdapter::new();
    dea.mock_short_name = "apple";
    dea.mock_has_short_proper = false;
    oq.process_queue(&mut dea);

    assert_eq!("The apple.\n", dea.buffer);
}
