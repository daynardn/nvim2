mod text;
mod files;
mod lsp;

use std::{arch::x86_64::_mm_pause, env, thread::sleep, time};

use files::files::load_file;
use gpui::{
    div, prelude::*, px, rgb, size, App, AppContext, Bounds, Context, FocusHandle, FocusableView, KeyBinding, SharedString, TaskLabel, View, ViewContext, WindowBounds, WindowOptions
};
use lsp::lsp::run_lsp;
use text::{text::TextInput, text_input::*};
use std::error::Error;

use crate::lsp::lsp::start_lsp;

struct Warning {
    warning: String, // just for lsp diagnostics testing
}

struct File {
    text_input: View<TextInput>, // file lines
    focus_handle: FocusHandle,
}

impl FocusableView for File {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for File {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0xaaaaaa))
            .track_focus(&self.focus_handle(cx))
            .flex()
            .flex_col()
            .size_full()
            .child(self.text_input.clone())
    }
}

fn main()  -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("No homepage yet :(, input a filename");
        todo!()
    }
    if !args[1].starts_with("/") {
        // ./exe test.txt -> ./exe /test.txt
        args[1] =  "/".to_string() + &args[1];
    }
    let filename = env::current_dir().unwrap().as_os_str().to_str().unwrap().to_owned() + &args[1];

    let lsp = start_lsp(env::current_dir().unwrap().as_os_str().to_str().unwrap().to_owned().clone());
    println!("waiting");
    let app = App::new();

    app.run(|cx: &mut AppContext| {
        let bounds = Bounds::centered(None, size(px(300.0), px(300.0)), cx);
        cx.bind_keys([
            KeyBinding::new("enter", Enter, None),
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("up", Up, None),
            KeyBinding::new("down", Down, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("shift-up", SelectUp, None),
            KeyBinding::new("shift-down", SelectDown, None),
            KeyBinding::new("ctrl-a", SelectAll, None),
            KeyBinding::new("ctrl-v", Paste, None),
            KeyBinding::new("ctrl-c", Copy, None),
            KeyBinding::new("ctrl-x", Cut, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-shift-space", ShowCharacterPalette, None),
            KeyBinding::new("ctrl-s", Save, None),
        ]);

        let lines = load_file(filename.clone());

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
                        cursor_pos: 0,
                        open_file: filename,
                        lines: lines.len(),
                        content: lines,
                        placeholder: "".into(),
                        selected_lines: 0..0,
                        selected_lines_reversed: false,
                        selected_range: 0..0,
                        selection_reversed: false,
                        marked_range: None,
                        last_layout: None,
                        last_bounds: None,
                        last_cursor_scroll: px(0.0),
                        is_selecting: false,
                    });
                    cx.new_view(|cx| File {
                        text_input,
                        focus_handle: cx.focus_handle(),
                    })
                },
            )
            .unwrap();
        cx.observe_keystrokes(move |_ev, cx| {
            window
                .update(cx, |_view, cx| {
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

        // run lsp
        cx.spawn(|cx: gpui::AsyncAppContext| async move {
            let results = cx.background_executor().spawn({
                run_lsp(lsp)
            }).await;
            // signal that lsp ran
            cx.update(|cx| {
                window.update(cx, |view, cx| {
                    cx.update_model(&view.text_input.model, |a, b| {
                        a.lines = 0;
                        let mut shared_string_vec = vec![];
                        let str: String = serde_json::to_string_pretty(&results.unwrap().1).unwrap();
                        for line in str.lines() {
                            shared_string_vec.push(line.to_string().into()); // Convert &str to String
                            a.lines += 1;
                        }
                        
                        a.content = shared_string_vec;
                    });
                    cx.notify()
                }).ok();
            })
            .ok();
            // println!("{:#?}", results);
        }).detach();
            
    });
    Ok(())
}
