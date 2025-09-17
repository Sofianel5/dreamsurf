use crate::components::input::InputField;
use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div};
pub struct Home {
    query_input: Entity<InputField>,
}

impl Home {
    pub fn new(cx: &mut Context<Self>, _window: &mut Window) -> Self {
        let query_input = cx.new(InputField::new);
        Self { query_input }
    }
}

impl Render for Home {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        return div()
            .flex()
            .flex_col()
            .justify_center()
            .w_full()
            .h_full()
            .bg(gpui::white())
            .child(
                div()
                    .flex()
                    .w_full()
                    .items_center()
                    .justify_center()
                    .text_color(gpui::black())
                    .pb_4()
                    .child(format!("Welcome to your dream...")),
            )
            .child(
                div()
                    .flex()
                    .w_full()
                    .items_center()
                    .justify_center()
                    .child(self.query_input.clone()),
            );
    }
}
