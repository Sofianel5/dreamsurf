mod components;
mod home;
use gpui::{
    App, AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, point, px,
    size,
};
use gpui_component::{Root, theme};
use home::Home;

#[tokio::main]
async fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1000.0), px(1000.0)), cx);
        cx.set_global(theme::Theme::default());
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("DreamSurf".to_string().into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(9.0), px(9.0))),
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| Home::new(cx, window));
                    cx.new(|cx| Root::new(view.into(), window, cx))
                },
            )
            .expect("Failed to open window");
        window
            .update(cx, |_, window, _| {
                window.activate_window();
            })
            .expect("Failed to update window.");
    })
}
