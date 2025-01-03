use std::ops::Range;

use gpui::{
    actions, black, div, fill, hsla, opaque_grey, point, prelude::*, px, relative, rgb, rgba, size,
    white, yellow, App, AppContext, Bounds, ClipboardItem, CursorStyle, ElementId,
    ElementInputHandler, FocusHandle, FocusableView, GlobalElementId, KeyBinding, Keystroke,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    ShapedLine, SharedString, SharedUri, Style, TextLayout, TextRun, UTF16Selection,
    UnderlineStyle, View, ViewContext, ViewInputHandler, WindowBounds, WindowContext,
    WindowOptions, WrappedLine,
};

pub struct TextInput {
    pub focus_handle: FocusHandle,
    pub focused_line: usize,
    pub content: Vec<SharedString>,
    pub placeholder: SharedString,
    pub selected_range: Range<usize>,
    pub selection_reversed: bool,
    pub marked_range: Option<Range<usize>>,
    pub last_layout: Option<WrappedLine>,
    pub last_bounds: Option<Bounds<Pixels>>,
    pub is_selecting: bool,
}