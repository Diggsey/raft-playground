use std::sync::{Arc, Mutex};
use std::time::Instant;

use act_zero::*;
use async_trait::async_trait;
use raft_zero::messages::Membership;
use raft_zero::{ObservedState, Observer};

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

impl Actor for DummyObserver {}

#[async_trait]
impl Observer for DummyObserver {
    async fn observe_state(&mut self, observed: ObservedState) -> ActorResult<()> {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        if state.observed.election_deadline != observed.election_deadline {
            state.timer_start = Instant::now();
        }
        state.observed = observed;
        Produces::ok(())
    }
    async fn observe_membership(&mut self, membership: Membership) -> ActorResult<()> {
        let state = &mut self.state.lock().unwrap().nodes[self.index];
        state.membership = membership;
        Produces::ok(())
    }
}
