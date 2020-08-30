use std::time::Instant;

use conrod_core::{Positionable, UiCell};
use raft_zero::NodeId;

use super::utils::node_pos;
use super::widget_ext::WidgetExt;
use super::widgets::node::{Node, NodeRole};
use crate::state::NodeState;

pub fn gui(
    ui: &mut UiCell,
    node_id: NodeId,
    state: &mut NodeState,
    area_size: f64,
    num_nodes: usize,
    selected: bool,
) -> bool {
    let circle_radius = area_size * 0.05;
    let pos = node_pos(node_id, area_size, num_nodes);

    let elapsed = if let Some(election_deadline) = state.observed.election_deadline {
        let election_interval = election_deadline - state.timer_start;
        let elapsed_interval = Instant::now() - state.timer_start;
        elapsed_interval.as_secs_f64() / election_interval.as_secs_f64()
    } else {
        0.0
    };

    let role = if Some(node_id) == state.observed.leader_id {
        NodeRole::Leader
    } else if state.membership.is_learner_or_unknown(node_id) {
        if state.membership.learners.contains(&node_id) {
            NodeRole::Learner
        } else {
            NodeRole::None
        }
    } else {
        NodeRole::Member
    };
    Node::new(
        role,
        selected,
        circle_radius,
        elapsed,
        format!(
            "Log: {}\nTerm: {}\nCommit: {}\nVoted: {}",
            state.observed.last_log_index.0,
            state.observed.current_term.0,
            state.observed.committed_index.0,
            if let Some(voted_for) = state.observed.voted_for {
                voted_for.0.to_string()
            } else {
                "-".into()
            }
        ),
    )
    .xy(pos)
    .set_opt(&mut state.id, ui)
}
