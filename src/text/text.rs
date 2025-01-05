use std::ops::Range;

use gpui::{
    prelude::*, AppContext, Bounds, FocusHandle, FocusableView, Pixels, SharedString, View, WrappedLine,
};

// defines what is basically the list of lines that is a file
pub struct TextInput {
    pub focus_handle: FocusHandle,
    pub focused_line: usize,
    pub cursor_pos: usize, // cursor l-r, scolling pos not current, "ideal" not actual
    pub lines: usize,
    pub open_file: String,
    pub content: Vec<SharedString>,
    pub content_offset: Pixels, // how far scrolled 
    pub placeholder: SharedString,
    pub selected_range: Range<usize>,
    pub selection_reversed: bool,
    pub marked_range: Option<Range<usize>>,
    pub last_layout: Option<WrappedLine>,
    pub last_bounds: Option<Bounds<Pixels>>,
    pub is_selecting: bool,
}

// one line of a file
pub struct TextElement {
    pub input: View<TextInput>,
    pub lines_pixels: Pixels, // wrapped not \n
    pub id: usize,
    pub wrap: Option<Pixels>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl FocusableView for TextInput {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}