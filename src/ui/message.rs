use std::time::Instant;

use conrod_core::widget;
use conrod_core::{Color, Colorable, Positionable, UiCell};

use raft_zero::NodeId;

use super::utils::{node_pos, vec_lerp, vec_mul};
use super::widget_ext::WidgetExt;
use crate::state::InFlightMessage;

pub fn gui(
    ui: &mut UiCell,
    msg: &mut InFlightMessage,
    enabled: bool,
    area_size: f64,
    num_nodes: usize,
) {
    let msg_radius = area_size * 0.01;
    let pos1 = node_pos(msg.to, area_size, num_nodes);
    let pos0 = if msg.from != NodeId::INVALID {
        node_pos(msg.from, area_size, num_nodes)
    } else {
        vec_mul(pos1, 1.5)
    };
    let progress = ((Instant::now() - msg.sent_at).as_secs_f64() / crate::CHANNEL_DELAY).min(1.0);
    let pos = vec_lerp(pos0, pos1, progress);
    let text_pos = vec_mul(vec_lerp(pos0, pos1, progress * 0.5 + 0.25), 1.25);
    let alpha = if enabled { 1.0 } else { 1.0 - progress };
    widget::Circle::fill(msg_radius)
        .xy(pos)
        .color(Color::Rgba(0.2, 0.3, 1.0, alpha as f32))
        .set_opt(&mut msg.id, ui);
    widget::Text::new(&msg.msg.info())
        .xy(text_pos)
        .color(conrod_core::color::WHITE)
        .font_size(16)
        .set_opt(&mut msg.text_id, ui);
}
