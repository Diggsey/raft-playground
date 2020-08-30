use std::f64;

use conrod_core::UiCell;
use raft_zero::NodeId;

use super::utils::node_pos;
use super::widget_ext::WidgetExt;
use super::widgets::channel::Channel;
use crate::state::ClusterState;

pub fn gui(ui: &mut UiCell, state: &mut ClusterState) {
    let num_nodes = state.nodes.len();
    let area_size = f64::min(ui.win_w, ui.win_h);

    for (&(from, to), channel) in state.channels.iter_mut() {
        let end_radius = area_size * 0.06;
        let width = area_size * 0.01;
        let pos0 = node_pos(from, area_size, num_nodes);
        let pos1 = node_pos(to, area_size, num_nodes);
        channel.enabled = Channel::new(channel.enabled, pos0, pos1, end_radius, width)
            .set_opt(&mut channel.id, ui);
    }

    let selected_index = state.selected_index;
    for (index, node) in state.nodes.iter_mut().enumerate() {
        if super::node::gui(
            ui,
            NodeId(index as u64),
            node,
            area_size,
            num_nodes,
            selected_index == index,
        ) {
            state.selected_index = index;
        }
    }

    for msg in &mut state.messages {
        let enabled = state
            .channels
            .get(&(msg.from, msg.to))
            .map(|chan| chan.enabled)
            .unwrap_or(true);
        super::message::gui(ui, msg, enabled, area_size, num_nodes);
    }

    super::toolbox::gui(ui, NodeId(state.selected_index as u64), state, area_size);
}
