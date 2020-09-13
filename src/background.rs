use std::future::Future;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use act_zero::*;
use futures::task::{FutureObj, Spawn, SpawnError};
use raft_zero::messages::*;
use raft_zero::*;
use serde::{Deserialize, Serialize};
use tokio::time::delay_for;

use crate::state::*;

mod connection;
mod observer;
mod storage;

pub fn run(state: Arc<Mutex<ClusterState>>) {
    thread::spawn(|| background(state));
}

struct TokioSpawn;

impl Spawn for TokioSpawn {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        tokio::spawn(future);
        Ok(())
    }
}

fn spawn_actor<A: Actor>(actor: A) -> Addr<Local<A>> {
    spawn(&TokioSpawn, actor).expect("TokioSpawn to be infallible")
}

#[derive(Debug, Clone)]
pub struct DummyLogData {
    pub msg: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DummyState {
    pub num_requests: u64,
    pub membership: Membership,
}

#[derive(Debug, Clone)]
pub struct DummyLogResponse;

impl LogData for DummyLogData {}
impl LogResponse for DummyLogResponse {}

#[derive(Debug)]
pub struct DummyApp {
    node_id: NodeId,
    config: Arc<Config>,
    storage: Addr<dyn Storage<Self>>,
    observer: Addr<dyn Observer>,
    state: Arc<Mutex<ClusterState>>,
}

impl DummyApp {
    pub fn new(node_id: NodeId, config: Arc<Config>, state: Arc<Mutex<ClusterState>>) -> Self {
        Self {
            node_id,
            config,
            storage: spawn_actor(storage::DummyStorage::new(
                node_id.0 as usize,
                state.clone(),
            ))
            .upcast(),
            observer: spawn_actor(observer::DummyObserver::new(
                node_id.0 as usize,
                state.clone(),
            ))
            .upcast(),
            state,
        }
    }
}

impl Application for DummyApp {
    type LogData = DummyLogData;
    type LogResponse = DummyLogResponse;
    type LogError = ();
    type SnapshotId = usize;

    fn config(&self) -> Arc<Config> {
        self.config.clone()
    }
    fn storage(&self) -> Addr<dyn Storage<Self>> {
        self.storage.clone()
    }
    fn observer(&self) -> Addr<dyn Observer> {
        self.observer.clone()
    }
    fn establish_connection(&mut self, node_id: NodeId) -> Addr<dyn Connection<Self>> {
        spawn_actor(connection::DummyConnection::new(
            self.node_id,
            node_id,
            self.state.clone(),
        ))
        .upcast()
    }
}

fn handle_client_response(
    state: Arc<Mutex<ClusterState>>,
    fut: impl Future<Output = Result<ClientResult<DummyApp>, Canceled>> + Send + 'static,
) {
    tokio::spawn(async move {
        let resp = fut.await.ok();
        state.lock().unwrap().responses.push(resp);
    });
}

fn handle_message_response<R>(
    to: NodeId,
    from: NodeId,
    state: Arc<Mutex<ClusterState>>,
    fut: impl Future<Output = Result<R, Canceled>> + Send + 'static,
    f: impl FnOnce(R) -> Message + Send + 'static,
) {
    tokio::spawn(async move {
        if let Ok(resp) = fut.await {
            state.lock().unwrap().send_message(to, from, f(resp));
        }
    });
}

#[tokio::main]
async fn background(state: Arc<Mutex<ClusterState>>) {
    let config = Arc::new({
        let mut config = Config::new();
        config
            .set_heartbeat_interval(Duration::from_secs(5))
            .set_max_append_entries_len(5)
            .set_election_timeout(Duration::from_secs(10), Duration::from_secs(20))
            .set_max_in_flight_requests(Some(5))
            .set_max_replication_buffer_len(10);
        config
    });
    let node_actors: Vec<_> = {
        let guard = state.lock().unwrap();
        guard
            .nodes
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let node_id = NodeId(index as u64);
                let app = DummyApp::new(node_id, config.clone(), state.clone());
                NodeActor::spawn(NodeId(index as u64), app)
            })
            .collect()
    };

    let _ = node_actors[0].call_bootstrap_cluster(BootstrapRequest {
        members: (0..node_actors.len())
            .map(|index| NodeId(index as u64))
            .collect(),
        learners: Default::default(),
    });

    loop {
        delay_for(Duration::from_millis(50)).await;

        let sent_time = Instant::now() - Duration::from_secs_f64(crate::CHANNEL_DELAY);
        while state
            .lock()
            .unwrap()
            .messages
            .front()
            .map(|msg| msg.sent_at <= sent_time)
            .unwrap_or(false)
        {
            let state = state.clone();
            let InFlightMessage { msg, to, from, .. } = {
                let mut guard = state.lock().unwrap();
                let msg = guard.messages.pop_front().unwrap();
                if msg.from != NodeId::INVALID {
                    if !guard.channels[&(msg.from, msg.to)].enabled {
                        continue;
                    }
                }
                msg
            };

            match msg {
                Message::VoteRequest(req, res) => {
                    let fut = node_actors[to.0 as usize].call_request_vote(req);
                    handle_message_response(to, from, state, fut, |resp| {
                        Message::VoteResponse(resp, res)
                    });
                }
                Message::VoteResponse(resp, res) => {
                    res.send(resp).ok();
                }
                Message::PreVoteRequest(req, res) => {
                    let fut = node_actors[to.0 as usize].call_request_pre_vote(req);
                    handle_message_response(to, from, state, fut, |resp| {
                        Message::PreVoteResponse(resp, res)
                    });
                }
                Message::PreVoteResponse(resp, res) => {
                    res.send(resp).ok();
                }
                Message::AppendEntriesRequest(req, res) => {
                    let fut = node_actors[to.0 as usize].call_append_entries(req);
                    handle_message_response(to, from, state, fut, |resp| {
                        Message::AppendEntriesResponse(resp, res)
                    });
                }
                Message::AppendEntriesResponse(resp, res) => {
                    res.send(resp).ok();
                }
                Message::InstallSnapshotRequest(req, res) => {
                    let fut = node_actors[to.0 as usize].call_install_snapshot(req);
                    handle_message_response(to, from, state, fut, |resp| {
                        Message::InstallSnapshotResponse(resp, res)
                    });
                }
                Message::InstallSnapshotResponse(resp, res) => {
                    res.send(resp).ok();
                }
                Message::ClientRequest(req) => {
                    let fut = node_actors[to.0 as usize].call_client_request(req);
                    handle_client_response(state, fut);
                }
                Message::SetLearners(req) => {
                    let fut = node_actors[to.0 as usize].call_set_learners(req);
                    handle_client_response(state, fut);
                }
                Message::SetMembers(req) => {
                    let fut = node_actors[to.0 as usize].call_set_members(req);
                    handle_client_response(state, fut);
                }
            }
        }
    }
}
