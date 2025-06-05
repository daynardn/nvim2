mod text;
mod files;
mod lsp;

use std::{arch::x86_64::_mm_pause, collections::HashMap, env, hash::Hash, thread::sleep, time};

use files::files::load_file;
use gpui::{
    div, prelude::*, px, rgb, size, App, AppContext, Bounds, Context, FocusHandle, FocusableView, KeyBinding, SharedString, TaskLabel, View, ViewContext, WindowBounds, WindowOptions
};
use lsp::lsp::run_lsp;
use text::{text::TextInput, text_input::*};
use std::error::Error;

use crate::lsp::{decode::Diagnostics, lsp::start_lsp};

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
                        diagnostics: HashMap::new(),
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
                        let results = results.unwrap().1.unwrap();
                        println!("{:?}", results.get("params").unwrap().get("diagnostics"));
                        let diagnostics = results.get("params").unwrap().get("diagnostics").unwrap();
                        
                        // println!("{}", serde_json::to_string_pretty(diagnostics).unwrap());
                        // println!("\n\nTWO: \n{}", serde_json::to_string_pretty(&diagnostics.get(1)).unwrap());
                        a.diagnostics = HashMap::new();
                        let mut i = 0;
                        while diagnostics.get(i).is_some() {
                            let error = diagnostics.get(i).unwrap();
                            let range = error.get("range").unwrap();
                            let start = range.get("start").unwrap().get("character").unwrap().as_u64().unwrap() as usize;
                            let end = range.get("end").unwrap().get("character").unwrap().as_u64().unwrap() as usize;
                            // TODO line could be range
                            let line = range.get("start").unwrap().get("line").unwrap().as_u64().unwrap() as usize;
                            // TODO check file, and multiple error same line

                            let diagnostic = Diagnostics {
                                diagnostic_range: start..end,
                                is_error: false,
                                message: "Warning".to_string(),
                            };

                            if a.diagnostics.contains_key(&line) {
                                a.diagnostics.get_mut(&line).unwrap_or(&mut vec![]).push(diagnostic);
                            }else {
                                a.diagnostics.insert(line, vec![diagnostic]);
                            }
                            println!("\n\nTWO: \n{}", serde_json::to_string_pretty(error).unwrap());
                            i += 1;
                        }
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
