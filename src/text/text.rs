use std::{collections::HashMap, ops::Range};

use gpui::{
    prelude::*, AppContext, Bounds, FocusHandle, FocusableView, Pixels, SharedString, View, WrappedLine,
};

use crate::lsp::decode::Diagnostics;

// defines what is basically the list of lines that is a file
pub struct TextInput {
    pub focus_handle: FocusHandle,
    pub focused_line: usize,
    pub cursor_pos: usize, // cursor l-r, scolling pos not current, "ideal" not actual
    pub lines: usize,
    pub open_file: String,
    pub content: Vec<SharedString>,
    pub placeholder: SharedString,
    pub selected_lines: Range<usize>, // lines + range of the selection
    pub selected_lines_reversed: bool, // lines + range of the selection
    pub selected_range: Range<usize>, // end..botton + full_lines + 0..top 
    pub selection_reversed: bool,
    pub marked_range: Option<Range<usize>>,
    pub last_layout: Option<WrappedLine>,
    pub last_bounds: Option<Bounds<Pixels>>,
    pub last_cursor_scroll: Pixels, // l-r content offset
    pub is_selecting: bool,
    pub diagnostics: HashMap<usize, Vec<Diagnostics>>,
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