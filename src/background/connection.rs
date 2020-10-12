use std::sync::{Arc, Mutex};

use act_zero::*;
use async_trait::async_trait;
use futures::channel::oneshot;
use raft_zero::messages::*;
use raft_zero::{Connection, NodeId};

use super::DummyApp;
use crate::state::{ClusterState, Message};

pub struct DummyConnection {
    from: NodeId,
    to: NodeId,
    state: Arc<Mutex<ClusterState>>,
}

impl DummyConnection {
    pub fn new(from: NodeId, to: NodeId, state: Arc<Mutex<ClusterState>>) -> Self {
        Self { from, to, state }
    }
    fn send(&self, msg: Message) {
        self.state
            .lock()
            .unwrap()
            .send_message(self.from, self.to, msg);
    }
}

impl Actor for DummyConnection {}

#[async_trait]
impl Connection<DummyApp> for DummyConnection {
    async fn request_vote(&mut self, req: VoteRequest) -> ActorResult<VoteResponse> {
        let (tx, rx) = oneshot::channel();
        self.send(Message::VoteRequest(req, tx));
        Ok(Produces::Deferred(rx))
    }
    async fn request_pre_vote(&mut self, req: PreVoteRequest) -> ActorResult<PreVoteResponse> {
        let (tx, rx) = oneshot::channel();
        self.send(Message::PreVoteRequest(req, tx));
        Ok(Produces::Deferred(rx))
    }
    async fn append_entries(
        &mut self,
        req: AppendEntriesRequest<DummyApp>,
    ) -> ActorResult<AppendEntriesResponse> {
        let (tx, rx) = oneshot::channel();
        self.send(Message::AppendEntriesRequest(req, tx));
        Ok(Produces::Deferred(rx))
    }
    async fn install_snapshot(
        &mut self,
        req: InstallSnapshotRequest,
    ) -> ActorResult<InstallSnapshotResponse> {
        let (tx, rx) = oneshot::channel();
        self.send(Message::InstallSnapshotRequest(req, tx));
        Ok(Produces::Deferred(rx))
    }
}
