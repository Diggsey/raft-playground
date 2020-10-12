use std::collections::{HashMap, VecDeque};
use std::mem;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use act_zero::*;
use conrod_core::widget::Id;
use futures::channel::oneshot;
use raft_zero::messages::*;
use raft_zero::*;
use tokio::io::AsyncWrite;

use crate::background::{DummyApp, DummyLogData, DummyState};

#[derive(Debug)]
pub enum Message {
    VoteRequest(VoteRequest, oneshot::Sender<Produces<VoteResponse>>),
    VoteResponse(VoteResponse, oneshot::Sender<Produces<VoteResponse>>),
    PreVoteRequest(PreVoteRequest, oneshot::Sender<Produces<PreVoteResponse>>),
    PreVoteResponse(PreVoteResponse, oneshot::Sender<Produces<PreVoteResponse>>),
    AppendEntriesRequest(
        AppendEntriesRequest<DummyApp>,
        oneshot::Sender<Produces<AppendEntriesResponse>>,
    ),
    AppendEntriesResponse(
        AppendEntriesResponse,
        oneshot::Sender<Produces<AppendEntriesResponse>>,
    ),
    InstallSnapshotRequest(
        InstallSnapshotRequest,
        oneshot::Sender<Produces<InstallSnapshotResponse>>,
    ),
    InstallSnapshotResponse(
        InstallSnapshotResponse,
        oneshot::Sender<Produces<InstallSnapshotResponse>>,
    ),
    ClientRequest(ClientRequest<DummyLogData>),
    SetMembers(SetMembersRequest),
    SetLearners(SetLearnersRequest),
}

impl Message {
    pub fn info(&self) -> String {
        match self {
            Self::VoteRequest(req, _) => format!("VoteRequest({})", req.candidate_id.0),
            Self::VoteResponse(req, _) => format!("VoteResponse({})", req.vote_granted),
            Self::PreVoteRequest(req, _) => format!("PreVoteRequest({})", req.candidate_id.0),
            Self::PreVoteResponse(req, _) => format!("PreVoteResponse({})", req.vote_granted),
            Self::AppendEntriesRequest(req, _) => format!(
                "AppendEntriesRequest({}, {})",
                req.prev_log_index.0,
                req.entries.len(),
            )
            .replace("{", "{\n"),
            Self::AppendEntriesResponse(req, _) => {
                format!("AppendEntriesResponse({})", req.success)
            }
            Self::InstallSnapshotRequest(req, _) => format!("InstallSnapshotRequest({})", req.done),
            Self::InstallSnapshotResponse(_req, _) => "InstallSnapshotResponse".into(),
            Self::ClientRequest(req) => format!("ClientRequest({:?})", req.data.msg),
            Self::SetMembers(req) => {
                format!("SetMembers({}, {})", req.ids.len(), req.fault_tolerance)
            }
            Self::SetLearners(req) => format!("SetLearners({})", req.ids.len()),
        }
    }
}

#[derive(Debug)]
pub struct InFlightMessage {
    pub id: Option<Id>,
    pub text_id: Option<Id>,
    pub sent_at: Instant,
    pub from: NodeId,
    pub to: NodeId,
    pub msg: Message,
}

#[derive(Debug, Clone)]
pub struct NodeState {
    pub id: Option<Id>,
    pub state: DummyState,
    pub log: Vec<Arc<Entry<DummyLogData>>>,
    pub applied: LogIndex,
    pub compacted: LogIndex,
    pub hs: HardState,
    pub observed: ObservedState,
    pub membership: Membership,
    pub timer_start: Instant,
    pub snapshots: Vec<Option<Arc<[u8]>>>,
    pub current_snapshot: usize,
}

pub struct SnapshotWriter {
    state: Arc<Mutex<ClusterState>>,
    node_index: usize,
    snapshot_id: usize,
    data: Vec<u8>,
}

impl SnapshotWriter {
    pub fn new(state: Arc<Mutex<ClusterState>>, node_index: usize) -> Self {
        let snapshot_id = {
            let node_state = &mut state.lock().unwrap().nodes[node_index];
            let snapshot_id = node_state.snapshots.len();
            node_state.snapshots.push(None);
            snapshot_id
        };
        Self {
            state,
            node_index,
            snapshot_id,
            data: Vec::new(),
        }
    }
    pub fn id(&self) -> usize {
        self.snapshot_id
    }
}

impl AsyncWrite for SnapshotWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.data).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.data).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = &mut *self;
        let res = Pin::new(&mut this.data).poll_shutdown(cx);
        this.state.lock().unwrap().nodes[this.node_index].snapshots[this.snapshot_id] =
            Some(mem::replace(&mut this.data, Vec::new()).into());
        res
    }
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            id: None,
            log: vec![Arc::new(Entry {
                term: Term(0),
                payload: EntryPayload::Blank,
            })],
            state: Default::default(),
            applied: LogIndex::ZERO,
            observed: ObservedState::default(),
            membership: Membership::default(),
            hs: HardState::default(),
            timer_start: Instant::now(),
            snapshots: Vec::new(),
            compacted: LogIndex::ZERO,
            current_snapshot: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelState {
    pub id: Option<Id>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Toolbox {
    pub canvas_id: Option<Id>,
    pub log_label_id: Option<Id>,
    pub log_bg_id: Option<Id>,
    pub log_list_id: Option<Id>,
    pub buttons_label_id: Option<Id>,
    pub request_button_id: Option<Id>,
    pub set_learners_button_id: Option<Id>,
    pub set_members_button_id: Option<Id>,
    pub request_args_id: Option<Id>,
    pub responses_label_id: Option<Id>,
    pub responses_bg_id: Option<Id>,
    pub responses_list_id: Option<Id>,
    pub request_args: String,
}

#[derive(Debug)]
pub struct ClusterState {
    pub nodes: Vec<NodeState>,
    pub messages: VecDeque<InFlightMessage>,
    pub channels: HashMap<(NodeId, NodeId), ChannelState>,
    pub selected_index: usize,
    pub toolbox: Toolbox,
    pub responses: Vec<Option<ClientResult<DummyApp>>>,
}

impl ClusterState {
    pub fn new(num_nodes: usize) -> Self {
        Self {
            nodes: vec![NodeState::default(); num_nodes],
            messages: VecDeque::new(),
            channels: (0..num_nodes)
                .flat_map(|from| {
                    (0..num_nodes).map(move |to| {
                        (
                            (NodeId(from as u64), NodeId(to as u64)),
                            ChannelState {
                                id: None,
                                enabled: true,
                            },
                        )
                    })
                })
                .collect(),
            selected_index: 0,
            toolbox: Default::default(),
            responses: Vec::new(),
        }
    }
    pub fn send_message(&mut self, from: NodeId, to: NodeId, msg: Message) {
        self.messages.push_back(InFlightMessage {
            id: None,
            text_id: None,
            from,
            to,
            sent_at: Instant::now(),
            msg,
        })
    }
}
