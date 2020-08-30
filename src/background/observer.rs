use std::sync::{Arc, Mutex};
use std::time::Instant;

use act_zero::*;
use raft_zero::messages::Membership;
use raft_zero::{ObservedState, ObserverImpl};

use crate::state::ClusterState;

pub struct DummyObserver {
    index: usize,
    state: Arc<Mutex<ClusterState>>,
}

impl DummyObserver {
    pub fn new(index: usize, state: Arc<Mutex<ClusterState>>) -> Self {
        Self { index, state }
    }
}

impl Actor for DummyObserver {
    type Error = ();
}
#[act_zero]
impl Observer for DummyObserver {
    async fn observe_state(&self, observed: ObservedState) {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        if state.observed.election_deadline != observed.election_deadline {
            state.timer_start = Instant::now();
        }
        state.observed = observed;
    }
    async fn observe_membership(&self, membership: Membership) {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        state.membership = membership;
    }
}
