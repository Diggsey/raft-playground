use conrod_core::{widget, Color, Colorable, Point, Positionable, Sizeable, Widget};

use crate::ui::utils::Transform;

#[derive(WidgetCommon)]
pub struct Channel {
    /// An object that handles some of the dirty work of rendering a GUI. We don't
    /// really have to worry about it.
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    enabled: bool,
    start_pos: Point,
    end_pos: Point,
    end_radius: f64,
    width: f64,
}

widget_ids! {
    struct Ids {
        arrow_head,
        arrow,
        rect,
    }
}

pub struct State {
    ids: Ids,
}

impl Channel {
    pub fn new(
        enabled: bool,
        start_pos: Point,
        end_pos: Point,
        end_radius: f64,
        width: f64,
    ) -> Self {
        let min_x = start_pos[0].min(end_pos[0]) - width * 2.5;
        let max_x = start_pos[0].max(end_pos[0]) + width * 2.5;
        let min_y = start_pos[1].min(end_pos[01]) - width * 2.5;
        let max_y = start_pos[1].max(end_pos[1]) + width * 2.5;
        Self {
            common: Default::default(),
            enabled,
            start_pos,
            end_pos,
            end_radius,
            width,
        }
        .x_y((min_x + max_x) * 0.5, (min_y + max_y) * 0.5)
        .w_h(max_x - min_x, max_y - min_y)
    }
}

impl Widget for Channel {
    type State = State;
    type Style = ();
    type Event = bool;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {}

    /// Optionally specify a function to use for determining whether or not a point is over a
    /// widget, or if some other widget's function should be used to represent this widget.
    ///
    /// This method is optional to implement. By default, the bounding rectangle of the widget
    /// is used.
    fn is_over(&self) -> widget::IsOverFn {
        use conrod_core::graph::Container;
        use conrod_core::Theme;
        fn is_over_widget(_: &Container, _: Point, _: &Theme) -> widget::IsOver {
            widget::IsOver::Bool(false)
        }
        is_over_widget
    }

    /// Update the state of the button by handling any input that has occurred since the last
    /// update.
    fn update(mut self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, ui, .. } = args;

        let hover = {
            let input = ui.widget_input(id);

            // If the button was clicked, produce `Some` event.
            if input.clicks().left().next().is_some() {
                self.enabled = !self.enabled;
            }

            input.mouse().map(|m| m.is_over()).unwrap_or_default()
        };

        let mut xf = Transform::from_segment(self.start_pos, self.end_pos);
        let scale = xf.rescale();

        let offset_y = self.width * 0.5;
        let head_len = self.width * 2.0;

        let arrow_head_points: Vec<_> = [
            [scale - self.end_radius, offset_y],
            [scale - self.end_radius - head_len, offset_y + head_len],
            [scale - self.end_radius - head_len, offset_y],
        ]
        .iter()
        .map(|p| xf.xform_point(*p))
        .collect();

        let arrow_points: Vec<_> = [
            [self.end_radius, offset_y],
            [scale - self.end_radius - head_len, offset_y],
            [scale - self.end_radius - head_len, offset_y + self.width],
            [self.end_radius, offset_y + self.width],
        ]
        .iter()
        .map(|p| xf.xform_point(*p))
        .collect();

        let color = if self.enabled {
            if hover {
                Color::Rgba(0.4, 0.4, 0.4, 1.0)
            } else {
                Color::Rgba(0.2, 0.2, 0.2, 1.0)
            }
        } else {
            if hover {
                Color::Rgba(0.6, 0.0, 0.0, 1.0)
            } else {
                Color::Rgba(0.3, 0.0, 0.0, 1.0)
            }
        };

        widget::Polygon::abs_fill(arrow_head_points)
            .color(color)
            .graphics_for(id)
            .set(state.ids.arrow_head, ui);

        widget::Polygon::abs_fill(arrow_points)
            .color(color)
            .graphics_for(id)
            .set(state.ids.arrow, ui);

        self.enabled
    }
}
