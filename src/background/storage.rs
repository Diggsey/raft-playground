use std::io::Cursor;
use std::ops::Range;
use std::sync::{Arc, Mutex};

use act_zero::*;
use async_trait::async_trait;
use raft_zero::messages::*;
use raft_zero::{
    BoxAsyncRead, BoxAsyncWrite, HardState, LogIndex, LogRange, LogRangeOrSnapshot, LogState,
    Snapshot, Storage,
};

use super::{DummyApp, DummyLogData, DummyLogResponse, DummyState};
use crate::state::{ClusterState, SnapshotWriter};

pub struct DummyStorage {
    index: usize,
    state: Arc<Mutex<ClusterState>>,
}

impl DummyStorage {
    pub fn new(index: usize, state: Arc<Mutex<ClusterState>>) -> Self {
        Self { index, state }
    }
    async fn compact_log(&mut self) {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        if state.applied == state.compacted {
            return;
        }
        let bytes = serde_json::to_vec(&state.state).unwrap();
        state.current_snapshot = state.snapshots.len();
        state.snapshots.push(Some(bytes.into()));
        state
            .log
            .drain(0..(state.applied - state.compacted) as usize);
        state.compacted = state.applied;
    }
}

impl Actor for DummyStorage {}

#[async_trait]
impl Storage<DummyApp> for DummyStorage {
    async fn init(&mut self) -> ActorResult<HardState> {
        Produces::ok(HardState::default())
    }
    async fn get_log_state(&mut self) -> ActorResult<LogState> {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        Produces::ok(LogState {
            last_log_applied: state.applied,
            last_log_index: state.compacted + state.log.len() as u64 - 1,
            last_membership_applied: state.state.membership.clone(),
        })
    }
    async fn get_log_range(
        &mut self,
        range: Range<LogIndex>,
    ) -> ActorResult<LogRangeOrSnapshot<DummyLogData, usize>> {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        let result = if range.start <= state.compacted {
            LogRangeOrSnapshot::Snapshot(Snapshot {
                id: state.current_snapshot,
                last_log_index: state.compacted,
                last_log_term: state.log[0].term,
            })
        } else {
            let prev_offset = range.start - state.compacted - 1;
            assert!(
                prev_offset < (state.log.len() as u64),
                "Log index out of range"
            );
            let range_len = range.end - range.start;

            let prev_log_term = state.log[prev_offset as usize].term;
            LogRangeOrSnapshot::LogRange(LogRange {
                entries: state
                    .log
                    .iter()
                    .skip((prev_offset + 1) as usize)
                    .take(range_len as usize)
                    .cloned()
                    .collect(),
                prev_log_index: range.start - 1,
                prev_log_term,
            })
        };

        Produces::ok(result)
    }
    async fn append_entry_to_log(
        &mut self,
        entry: Arc<Entry<DummyLogData>>,
    ) -> ActorResult<Result<(), ()>> {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        state.log.push(entry);
        Produces::ok(Ok(()))
    }
    async fn replicate_to_log(&mut self, range: LogRange<DummyLogData>) -> ActorResult<()> {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        let expected_log_len = range.prev_log_index + 1 - state.compacted;
        state.log.truncate(expected_log_len as usize);
        state.log.extend(range.entries);
        Produces::ok(())
    }
    async fn apply_to_state_machine(
        &mut self,
        index: LogIndex,
        entry: Arc<Entry<DummyLogData>>,
    ) -> ActorResult<DummyLogResponse> {
        let needs_compaction = {
            let state = &mut self.state.lock().unwrap().nodes[self.index];
            match &entry.payload {
                EntryPayload::Application(_) => {
                    state.state.num_requests += 1;
                }
                EntryPayload::MembershipChange(m) => state.state.membership = m.membership.clone(),
                EntryPayload::Blank => {}
            }
            state.applied += 1;
            assert_eq!(state.applied, index);

            state.log.len() > 5
        };

        if needs_compaction {
            self.compact_log().await;
        }

        Produces::ok(DummyLogResponse)
    }
    async fn save_hard_state(&mut self, hs: HardState) -> ActorResult<()> {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        state.hs = hs;
        Produces::ok(())
    }

    async fn install_snapshot(&mut self, snapshot: Snapshot<usize>) -> ActorResult<()> {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        let snapshot_state: DummyState =
            serde_json::from_slice(state.snapshots[snapshot.id].as_deref().unwrap()).unwrap();

        assert!(snapshot.last_log_index >= state.compacted);

        let snapshot_offset = snapshot.last_log_index - state.compacted;
        if snapshot_offset < state.log.len() as u64
            && state.log[snapshot_offset as usize].term == snapshot.last_log_term
        {
            state.log.drain(0..snapshot_offset as usize);
        } else {
            state.log.clear();
            state.log.push(Arc::new(Entry {
                term: snapshot.last_log_term,
                payload: EntryPayload::Blank,
            }))
        }
        state.state = snapshot_state;
        state.applied = snapshot.last_log_index;
        state.compacted = snapshot.last_log_index;
        state.current_snapshot = snapshot.id;

        Produces::ok(())
    }
    async fn create_snapshot(&mut self) -> ActorResult<(usize, BoxAsyncWrite)> {
        let writer = SnapshotWriter::new(self.state.clone(), self.index);
        Produces::ok((writer.id(), Box::pin(writer)))
    }
    async fn read_snapshot(&mut self, id: usize) -> ActorResult<BoxAsyncRead> {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        let reader = Cursor::new(state.snapshots[id].clone().unwrap());
        Produces::ok(Box::pin(reader))
    }
}
