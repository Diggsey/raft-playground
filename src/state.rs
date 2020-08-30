use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use act_zero::Sender;
use conrod_core::widget::Id;
use raft_zero::messages::*;
use raft_zero::*;

use crate::background::{DummyApp, DummyLogData};

#[derive(Debug)]
pub enum Message {
    VoteRequest(VoteRequest, Sender<VoteResponse>),
    VoteResponse(VoteResponse, Sender<VoteResponse>),
    AppendEntriesRequest(
        AppendEntriesRequest<DummyApp>,
        Sender<AppendEntriesResponse>,
    ),
    AppendEntriesResponse(AppendEntriesResponse, Sender<AppendEntriesResponse>),
    ClientRequest(ClientRequest<DummyLogData>),
    SetMembers(SetMembersRequest),
    SetLearners(SetLearnersRequest),
}

impl Message {
    pub fn info(&self) -> String {
        match self {
            Self::VoteRequest(req, _) => format!("VoteRequest({})", req.candidate_id.0),
            Self::VoteResponse(req, _) => format!("VoteResponse({})", req.vote_granted),
            Self::AppendEntriesRequest(req, _) => format!(
                "AppendEntriesRequest({}, {})",
                req.prev_log_index.0,
                req.entries.len(),
            )
            .replace("{", "{\n"),
            Self::AppendEntriesResponse(req, _) => {
                format!("AppendEntriesResponse({})", req.success)
            }
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
    pub log: Vec<Arc<Entry<DummyLogData>>>,
    pub hs: HardState,
    pub observed: ObservedState,
    pub membership: Membership,
    pub timer_start: Instant,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            id: None,
            log: vec![Arc::new(Entry {
                term: Term(0),
                payload: EntryPayload::Blank,
            })],
            observed: ObservedState::default(),
            membership: Membership::default(),
            hs: HardState::default(),
            timer_start: Instant::now(),
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
    pub responses: Vec<ClientResult<DummyApp>>,
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
