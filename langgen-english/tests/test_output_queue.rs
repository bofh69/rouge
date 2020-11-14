mod common;

use common::*;
use langgen_english::*;
use std::collections::vec_deque::VecDeque;
use std::sync::Mutex;

type DebQueue = Mutex<VecDeque<FragmentEntry<i32>>>;

type DebOutputQueue = langgen_english::OutputQueue<i32, DebQueue>;

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

#[test]
fn output_advanced_sentance() {
    let mut oq = DebOutputQueue::new(Mutex::new(VecDeque::new()), 16);
    oq.the(8)
        .v(8, "look")
        .s("like a \"")
        .s("GMO")
        .s("\" apple")
        .s("someone said");

    let mut dea = DebugEntityAdapter::new();
    dea.mock_short_name = "apple";
    dea.mock_has_short_proper = false;
    oq.process_queue(&mut dea);

    assert_eq!(
        "The apple looks like a \"GMO\" apple someone said.\n",
        dea.buffer
    );
}
