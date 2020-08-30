use conrod_core::widget::Id;
use conrod_core::{UiCell, Widget};

pub trait WidgetExt: Widget {
    fn set_opt<'a, 'b>(self, id: &mut Option<Id>, ui_cell: &'a mut UiCell<'b>) -> Self::Event;
}

impl<T: Widget> WidgetExt for T {
    fn set_opt<'a, 'b>(
        self,
        maybe_id: &mut Option<Id>,
        ui_cell: &'a mut UiCell<'b>,
    ) -> Self::Event {
        let id = if let Some(id) = *maybe_id {
            id
        } else {
            let id = ui_cell.widget_id_generator().next();
            *maybe_id = Some(id);
            id
        };
        self.set(id, ui_cell)
    }
}
