use conrod_core::{widget, Color, Colorable, Point, Positionable, Sizeable, Widget};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NodeRole {
    Leader,
    Member,
    Learner,
    None,
}

#[derive(WidgetCommon)]
pub struct Node {
    /// An object that handles some of the dirty work of rendering a GUI. We don't
    /// really have to worry about it.
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    role: NodeRole,
    selected: bool,
    radius: f64,
    elapsed: f64,
    text: String,
}

widget_ids! {
    struct Ids {
        background,
        elapsed_outline,
        selected_outline,
        text,
    }
}

pub struct State {
    ids: Ids,
}

impl Node {
    pub fn new(role: NodeRole, selected: bool, radius: f64, elapsed: f64, text: String) -> Self {
        Self {
            common: Default::default(),
            role,
            selected,
            radius,
            elapsed,
            text,
        }
        .w_h(radius * 2.2, radius * 2.2)
    }
}

impl Widget for Node {
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
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id,
            state,
            ui,
            rect,
            ..
        } = args;

        let clicked = ui.widget_input(id).clicks().left().next().is_some();

        let color = match self.role {
            NodeRole::Leader => Color::Rgba(1.0, 1.0, 0.0, 1.0),
            NodeRole::Member => Color::Rgba(1.0, 1.0, 1.0, 1.0),
            NodeRole::Learner => Color::Rgba(0.5, 1.0, 0.5, 1.0),
            NodeRole::None => Color::Rgba(0.5, 0.5, 0.5, 1.0),
        };

        if self.selected {
            widget::Circle::fill(self.radius * 1.1)
                .color(Color::Rgba(0.5, 1.0, 1.0, 1.0))
                .xy(rect.xy())
                .graphics_for(id)
                .set(state.ids.selected_outline, ui);
        }

        if self.elapsed > 0.0 {
            let elapsed_angle = self.elapsed * 2.0 * std::f64::consts::PI;
            widget::Circle::fill(self.radius * 1.1)
                .color(Color::Rgba(1.0, 0.0, 0.0, 1.0))
                .xy(rect.xy())
                .section(elapsed_angle)
                .offset_radians(std::f64::consts::PI * 0.5 - elapsed_angle)
                .graphics_for(id)
                .set(state.ids.elapsed_outline, ui);
        }

        widget::Circle::fill(self.radius)
            .color(color)
            .xy(rect.xy())
            .graphics_for(id)
            .set(state.ids.background, ui);

        widget::Text::new(&self.text)
            .color(conrod_core::color::BLACK)
            .xy(rect.xy())
            .font_size((self.radius * 0.3) as u32)
            .graphics_for(id)
            .set(state.ids.text, ui);

        clicked
    }
}
