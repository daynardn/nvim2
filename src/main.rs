mod text;
use crate::text::text_input::*;

use gpui::{
    div, prelude::*, px, rgb, size, App, AppContext, Bounds, FocusHandle, FocusableView,
    KeyBinding, Keystroke, MouseUpEvent, View, ViewContext, WindowBounds, WindowOptions,
};
use text::text::TextInput;

struct InputExample {
    text_input: View<TextInput>,
    recent_keystrokes: Vec<Keystroke>,
    focus_handle: FocusHandle,
}

impl FocusableView for InputExample {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl InputExample {
    fn on_reset_click(&mut self, _: &MouseUpEvent, cx: &mut ViewContext<Self>) {
        self.recent_keystrokes.clear();
        self.text_input
            .update(cx, |text_input, _cx| text_input.reset());
        cx.notify();
    }
}

impl Render for InputExample {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0xaaaaaa))
            .track_focus(&self.focus_handle(cx))
            .flex()
            .flex_col()
            .size_full()
            // .child(
            //     div()
            //         .bg(white())
            //         .border_b_1()
            //         .border_color(black())
            //         .flex()
            //         .flex_row()
            //         .justify_between()
            //         .child(format!("Keyboard {}", cx.keyboard_layout()))
            //         .child(
            //             div()
            //                 .border_1()
            //                 .border_color(black())
            //                 .px_2()
            //                 .bg(yellow())
            //                 .child("Reset")
            //                 .hover(|style| {
            //                     style
            //                         .bg(yellow().blend(opaque_grey(0.5, 0.5)))
            //                         .cursor_pointer()
            //                 })
            //                 .on_mouse_up(MouseButton::Left, cx.listener(Self::on_reset_click)),
            //         ),
            // )
            .child(self.text_input.clone())
            // .children(self.recent_keystrokes.iter().rev().map(|ks| {
            //     format!(
            //         "{:} {}",
            //         ks.unparse(),
            //         if let Some(key_char) = ks.key_char.as_ref() {
            //             format!("-> {:?}", key_char)
            //         } else {
            //             "".to_owned()
            //         }
            //     )
            // }))
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        let bounds = Bounds::centered(None, size(px(300.0), px(300.0)), cx);
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("up", Up, None),
            KeyBinding::new("down", Down, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("cmd-v", Paste, None),
            KeyBinding::new("cmd-c", Copy, None),
            KeyBinding::new("cmd-x", Cut, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
        ]);

        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |cx| {
                    let text_input = cx.new_view(|cx| TextInput {
                        focus_handle: cx.focus_handle(),
                        focused_line: 0,
                        content: vec!["".into(); 10],
                        placeholder: "".into(),
                        selected_range: 0..0,
                        selection_reversed: false,
                        marked_range: None,
                        last_layout: None,
                        last_bounds: None,
                        is_selecting: false,
                    });
                    cx.new_view(|cx| InputExample {
                        text_input,
                        recent_keystrokes: vec![],
                        focus_handle: cx.focus_handle(),
                    })
                },
            )
            .unwrap();
        cx.observe_keystrokes(move |ev, cx| {
            window
                .update(cx, |view, cx| {
                    view.recent_keystrokes.push(ev.keystroke.clone());
                    cx.notify();
                })
                .unwrap();
        })
        .detach();
        cx.on_keyboard_layout_change({
            move |cx| {
                window.update(cx, |_, cx| cx.notify()).ok();
            }
        })
        .detach();

        window
            .update(cx, |view, cx| {
                cx.focus_view(&view.text_input);
                cx.activate(true);
            })
            .unwrap();
    });
}
