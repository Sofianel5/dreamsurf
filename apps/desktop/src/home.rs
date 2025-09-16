use gpui::{Context, IntoElement, Render, Styled, Window, div};

pub struct Home {}

impl Home {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for Home {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        return div().flex().w_full().h_full().bg(gpui::white());
    }
}
