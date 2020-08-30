use std::collections::HashSet;

use conrod_core::{widget, Color, Colorable, Labelable, Positionable, Sizeable, UiCell, Widget};
use raft_zero::messages::{ClientRequest, ResponseMode, SetLearnersRequest, SetMembersRequest};
use raft_zero::NodeId;

use super::widget_ext::WidgetExt;
use crate::background::DummyLogData;
use crate::state::{ClusterState, Message};

fn parse_node_ids(s: &str) -> Option<HashSet<NodeId>> {
    s.split(",")
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| part.parse().map(NodeId))
        .collect::<Result<_, _>>()
        .ok()
}

pub fn gui(ui: &mut UiCell, node_id: NodeId, cluster_state: &mut ClusterState, area_size: f64) {
    let toolbox = &mut cluster_state.toolbox;
    let state = &mut cluster_state.nodes[node_id.0 as usize];
    let title_font_size = (area_size * 0.02) as u32;
    let font_size = (area_size * 0.015) as u32;
    let spacing = area_size * 0.01;

    // Main controls window
    widget::Canvas::new()
        .title_bar("Controls")
        .title_bar_color(Color::Rgba(0.0, 0.0, 0.5, 1.0))
        .color(Color::Rgba(0.0, 0.0, 0.0, 0.5))
        .label_color(conrod_core::color::WHITE)
        .label_font_size(title_font_size)
        .w_h(area_size * 0.5, area_size * 0.8)
        .top_right()
        .pad(spacing)
        .set_opt(&mut toolbox.canvas_id, ui);

    // Log view
    widget::Text::new("Log entries")
        .parent(toolbox.canvas_id.unwrap())
        .color(conrod_core::color::WHITE)
        .font_size(font_size)
        .top_left()
        .set_opt(&mut toolbox.log_label_id, ui);

    widget::Canvas::new()
        .color(Color::Rgba(0.0, 0.0, 0.0, 0.5))
        .down(spacing)
        .kid_area_w_of(toolbox.canvas_id.unwrap())
        .h(area_size * 0.2)
        .set_opt(&mut toolbox.log_bg_id, ui);

    let (mut items, scrollbar) = widget::List::flow_down(state.log.len())
        .parent(toolbox.log_bg_id.unwrap())
        .top_left()
        .scrollbar_next_to()
        .scrollbar_color(conrod_core::color::WHITE)
        .set_opt(&mut toolbox.log_list_id, ui);

    while let Some(item) = widget::list::Items::next(&mut items, ui) {
        let log_index = state.log.len() - item.i - 1;
        let entry = &state.log[log_index];
        let color = if log_index as u64 <= state.observed.committed_index.0 {
            conrod_core::color::WHITE
        } else {
            Color::Rgba(1.0, 0.8, 0.4, 1.0)
        };
        item.set(
            widget::Text::new(&format!("{}: {:?}\n", log_index, entry))
                .color(color)
                .font_size(font_size),
            ui,
        );
    }

    if let Some(scrollbar) = scrollbar {
        scrollbar.set(ui);
    }

    // Controls
    widget::Text::new("Requests")
        .parent(toolbox.canvas_id.unwrap())
        .color(conrod_core::color::WHITE)
        .font_size(font_size)
        .down_from(toolbox.log_bg_id.unwrap(), spacing)
        .set_opt(&mut toolbox.buttons_label_id, ui);

    let client_request_clicked = widget::Button::new()
        .parent(toolbox.canvas_id.unwrap())
        .w_h(area_size * 0.15, area_size * 0.05)
        .down(spacing)
        .label("Client Request")
        .label_font_size(font_size)
        .set_opt(&mut toolbox.request_button_id, ui)
        .was_clicked();

    let set_learners_clicked = widget::Button::new()
        .parent(toolbox.canvas_id.unwrap())
        .w_h(area_size * 0.15, area_size * 0.05)
        .right(spacing)
        .label("Set Learners")
        .label_font_size(font_size)
        .set_opt(&mut toolbox.set_learners_button_id, ui)
        .was_clicked();

    let set_members_clicked = widget::Button::new()
        .parent(toolbox.canvas_id.unwrap())
        .w_h(area_size * 0.15, area_size * 0.05)
        .right(spacing)
        .label("Set Members")
        .label_font_size(font_size)
        .set_opt(&mut toolbox.set_members_button_id, ui)
        .was_clicked();

    if let Some(new_args) = widget::TextBox::new(&toolbox.request_args)
        .parent(toolbox.canvas_id.unwrap())
        .kid_area_w_of(toolbox.canvas_id.unwrap())
        .down_from(toolbox.request_button_id.unwrap(), spacing)
        .font_size(font_size)
        .set_opt(&mut toolbox.request_args_id, ui)
        .into_iter()
        .filter_map(|ev| {
            if let widget::text_box::Event::Update(args) = ev {
                Some(args)
            } else {
                None
            }
        })
        .last()
    {
        toolbox.request_args = new_args;
    }

    // Responses
    widget::Text::new("Responses")
        .parent(toolbox.canvas_id.unwrap())
        .color(conrod_core::color::WHITE)
        .font_size(font_size)
        .down_from(toolbox.request_args_id.unwrap(), spacing)
        .set_opt(&mut toolbox.responses_label_id, ui);

    widget::Canvas::new()
        .color(Color::Rgba(0.0, 0.0, 0.0, 0.5))
        .down(spacing)
        .kid_area_w_of(toolbox.canvas_id.unwrap())
        .h(area_size * 0.2)
        .set_opt(&mut toolbox.responses_bg_id, ui);

    let (mut items, scrollbar) = widget::List::flow_down(cluster_state.responses.len())
        .parent(toolbox.responses_bg_id.unwrap())
        .top_left()
        .scrollbar_next_to()
        .scrollbar_color(conrod_core::color::WHITE)
        .set_opt(&mut toolbox.responses_list_id, ui);

    while let Some(item) = widget::list::Items::next(&mut items, ui) {
        let response_index = cluster_state.responses.len() - item.i - 1;
        let entry = &cluster_state.responses[response_index];
        item.set(
            widget::Text::new(&format!("{:?}\n", entry))
                .color(conrod_core::color::WHITE)
                .font_size(font_size),
            ui,
        );
    }

    if let Some(scrollbar) = scrollbar {
        scrollbar.set(ui);
    }

    if set_learners_clicked {
        if let Some(ids) = parse_node_ids(&cluster_state.toolbox.request_args) {
            cluster_state.toolbox.request_args.clear();

            cluster_state.send_message(
                NodeId::INVALID,
                node_id,
                Message::SetLearners(SetLearnersRequest { ids }),
            );
        }
    }

    if set_members_clicked {
        if let Some(ids) = parse_node_ids(&cluster_state.toolbox.request_args) {
            cluster_state.toolbox.request_args.clear();

            cluster_state.send_message(
                NodeId::INVALID,
                node_id,
                Message::SetMembers(SetMembersRequest {
                    ids,
                    fault_tolerance: 1,
                }),
            );
        }
    }

    if client_request_clicked {
        cluster_state.send_message(
            NodeId::INVALID,
            node_id,
            Message::ClientRequest(ClientRequest {
                data: DummyLogData {
                    msg: cluster_state.toolbox.request_args.clone(),
                },
                response_mode: ResponseMode::Applied,
            }),
        );
    }
}
