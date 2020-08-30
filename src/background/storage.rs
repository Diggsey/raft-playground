use std::ops::Range;
use std::sync::{Arc, Mutex};

use act_zero::*;
use raft_zero::messages::*;
use raft_zero::{HardState, InitialState, LogIndex, LogRange, StorageImpl};

use super::{DummyApp, DummyLogData, DummyLogResponse};
use crate::state::ClusterState;

pub struct DummyStorage {
    index: usize,
    state: Arc<Mutex<ClusterState>>,
}

impl DummyStorage {
    pub fn new(index: usize, state: Arc<Mutex<ClusterState>>) -> Self {
        Self { index, state }
    }
}

impl Actor for DummyStorage {
    type Error = ();
}
#[act_zero]
impl Storage<DummyApp> for DummyStorage {
    async fn get_initial_state(&mut self, res: Sender<Option<InitialState>>) {
        res.send(None).ok();
    }
    async fn get_log_range(&mut self, range: Range<LogIndex>, res: Sender<LogRange<DummyLogData>>) {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        let prev_log_index = LogIndex(range.start.0 - 1);
        assert!(
            (prev_log_index.0 as usize) < state.log.len(),
            "Log index out of range"
        );
        let prev_log_term = state.log[prev_log_index.0 as usize].term;
        res.send(LogRange {
            entries: state
                .log
                .iter()
                .skip(range.start.0 as usize)
                .take((range.end.0 - range.start.0) as usize)
                .cloned()
                .collect(),
            prev_log_index,
            prev_log_term,
        })
        .ok();
    }
    async fn append_entry_to_log(
        &mut self,
        entry: Arc<Entry<DummyLogData>>,
        res: Sender<Result<(), ()>>,
    ) {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        state.log.push(entry);
        res.send(Ok(())).ok();
    }
    async fn replicate_to_log(&mut self, range: LogRange<DummyLogData>, res: Sender<()>) {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        state.log.truncate(range.prev_log_index.0 as usize + 1);
        state.log.extend(range.entries);
        res.send(()).ok();
    }
    async fn apply_to_state_machine(
        &mut self,
        _index: LogIndex,
        _entry: Arc<Entry<DummyLogData>>,
        res: Sender<DummyLogResponse>,
    ) {
        res.send(DummyLogResponse).ok();
    }
    async fn save_hard_state(&mut self, hs: HardState, res: Sender<()>) {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        state.hs = hs;
        res.send(()).ok();
    }
}
