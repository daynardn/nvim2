use gpui::{div, prelude::*, rgb, App, AppContext, SharedString, ViewContext, WindowOptions};

struct Window {
    text: SharedString,
}

impl Render for Window {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x2e7d32))
            .size_full()
            .justify_center()
            .items_center()
        // .children(files, tabbar)
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|_cx| Window {
                text: "World".into(),
            })
        })
        .unwrap();
    });
}
