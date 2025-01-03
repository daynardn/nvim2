use gpui::{
    div, fill, hsla, point, prelude::*, px, relative, rgb, rgba, size, white, Bounds, CursorStyle, ElementId, ElementInputHandler, FocusableView, GlobalElementId, LayoutId, MouseButton, PaintQuad, Pixels, Point, Style, TextRun, UnderlineStyle, ViewContext, WindowContext, WrappedLine
};

use super::text::{TextElement, TextInput};

impl Render for TextInput {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .key_context("TextInput")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .bg(rgb(0xeeeeee))
            .line_height(px(30.))
            .text_size(px(24.))
            .child(div().flex_col().children((0..10).map(|i| {
                div()
                    .flex_col()
                    // .h(px(30. + 4. * 2.))
                    // .border_y(px(i as f32 * 30.))
                    .w_full()
                    //.p(px(4.))
                    .bg(white())
                    .child(TextElement {
                        input: cx.view().clone(),
                        lines_pixels: px(30.),
                        id: i,
                    })
            })))
    }
}

pub struct PrepaintState {
    pub lines: Option<smallvec::SmallVec<[WrappedLine; 1]>>,
    pub cursor: Option<PaintQuad>,
    pub selection: Option<PaintQuad>,
}


impl Element for TextElement {
    type RequestLayoutState = ();

    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        let t_style = cx.text_style();

        style.size.width = relative(1.).into();
        // self.input.read(cx).content.
        let input = self.input.read(cx);
        let content = input.content[self.id].clone();
        
        let run = TextRun {
            len: content.len(),
            font: t_style.font(),
            color: t_style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let font_size = px(24.);
        let line = cx
            .text_system()
            .shape_text(content, font_size, 
                &vec![run], Some(Pixels(500.0)))
            .unwrap();
        
        // style.size.height = (self.lines_pixels).into();
        style.size.height = line[0].size(cx.line_height()).height.into();

        (cx.request_layout(style, []), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content[self.id].clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = cx.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), hsla(0., 0., 0., 0.2))
        } else {
            (content.clone(), style.color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run.clone()
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };
        // println!("{}", display_text);

        let font_size = style.font_size.to_pixels(cx.rem_size());
        let line = cx
            .text_system()
            .shape_text(display_text, font_size, &runs, Some(Pixels(500.0)))
            .unwrap();

        let cursor_pos = line[0].position_for_index(cursor, cx.line_height()).unwrap_or(Point { x: px(0.), y: px(0.) });
        let (mut selection, mut cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos.x, bounds.top() + cursor_pos.y),
                        size(px(2.), cx.line_height()),
                    ),
                    gpui::blue(),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left()
                                + line[0].unwrapped_layout.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left()
                                + line[0].unwrapped_layout.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    rgba(0x3311ff30),
                )),
                None,
            )
        };

        if self.input.read(cx).focused_line != self.id {
            cursor = None;
            selection = None;
        }

        PrepaintState {
            lines: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        if self.input.read(cx).focused_line == self.id {
            cx.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
            );
        }
        if let Some(selection) = prepaint.selection.take() {
            cx.paint_quad(selection)
        }
        for line in prepaint.lines.clone().unwrap() {
            // self.line_size[0] = line.size(cx.line_height());
            line.paint(bounds.origin, cx.line_height(), cx).unwrap();

            if focus_handle.is_focused(cx) {
                if let Some(cursor) = prepaint.cursor.take() {
                    cx.paint_quad(cursor);
                }
            }

            self.lines_pixels = line.size(cx.line_height()).height;
            self.request_layout(id, cx);

            if self.input.read(cx).focused_line == self.id { 
                self.input.update(cx, |input, _cx| {
                    input.last_layout = Some(line);
                    input.last_bounds = Some(bounds);
                });
            }
        }
    }
}