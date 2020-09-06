use std::sync::{Arc, Mutex};

use act_zero::*;
use raft_zero::messages::*;
use raft_zero::{ConnectionImpl, NodeId};

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

impl Actor for DummyConnection {
    type Error = ();
}

#[act_zero]
impl Connection<DummyApp> for DummyConnection {
    async fn request_vote(&self, req: VoteRequest, res: Sender<VoteResponse>) {
        self.send(Message::VoteRequest(req, res));
    }
    async fn append_entries(
        &self,
        req: AppendEntriesRequest<DummyApp>,
        res: Sender<AppendEntriesResponse>,
    ) {
        self.send(Message::AppendEntriesRequest(req, res));
    }
    async fn install_snapshot(
        &self,
        req: InstallSnapshotRequest,
        res: Sender<InstallSnapshotResponse>,
    ) {
        self.send(Message::InstallSnapshotRequest(req, res));
    }
}
