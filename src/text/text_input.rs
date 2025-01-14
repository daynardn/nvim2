use std::{cmp::{max, min}, mem::swap, ops::Range};

use gpui::{
    actions, point, px, Bounds, ClipboardItem, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, UTF16Selection, ViewContext, ViewInputHandler
};
use unicode_segmentation::*;

use crate::files::files::save;

use super::text::TextInput;

actions!(
    text_input,
    [
        Backspace,
        Enter,
        Delete,
        Up,
        Down,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
        Save,
    ]
);

impl TextInput {
    pub fn enter(&mut self, _: &Enter, cx: &mut ViewContext<Self>) {
        let range = self.selected_text_range(false, cx).unwrap().range;

        self.content.insert(
            self.focused_line + 1,
            self.content[self.focused_line][range.end..]
                .to_owned()
                .into(),
        );
        self.content[self.focused_line] = self.content[self.focused_line][0..range.start]
            .to_owned()
            .into();

        self.focused_line += 1;
        self.lines += 1;
        // self.cursor_pos = 0;

        self.selected_range = 0..0;
    }
    pub fn save(&mut self, _: &Save, _cx: &mut ViewContext<Self>) {
        println!("saved");
        save(self.open_file.clone(),
            self.content.clone(),
        );
    }
    pub fn down(&mut self, _: &Down, _cx: &mut ViewContext<Self>) {
        self.focused_line = min(self.lines - 1, self.focused_line + 1);
        let pos = min(self.content[self.focused_line].len(), self.cursor_pos);
        self.selected_range = pos..pos; // doesn't affect cursor_pos
        self.selected_lines = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        self.is_selecting = false;
    }
    pub fn up(&mut self, _: &Up, _cx: &mut ViewContext<Self>) {
        // usize would overflow
        self.focused_line = max(0 as i32, self.focused_line as i32 - 1) as usize;
        let pos = min(self.content[self.focused_line].len(), self.cursor_pos);
        self.selected_range = pos..pos; // doesn't affect cursor_pos
        self.selected_lines = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        self.is_selecting = false;
    }
    pub fn left(&mut self, _: &Left, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            // if first char, jump to end of next line
            if self.cursor_pos != 0 {
                self.move_to(self.previous_boundary(self.cursor_offset()), cx);
            }else { // TODO! not in vim
                self.up(&Up, cx);
                let pos = self.content[self.focused_line].len();
                self.selected_range = pos..pos;
                self.cursor_pos = pos;
            }
        } else {
            self.move_to(self.selected_range.start, cx);
        }
        self.selected_lines = 0..0;
    }

    pub fn right(&mut self, _: &Right, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            // if last char, jump to end of past line
            if self.cursor_pos != self.content[self.focused_line].len() {
                self.move_to(self.next_boundary(self.selected_range.end), cx);
            }else {
                self.down(&Down, cx);
                self.selected_range = 0..0;
                self.cursor_pos = 0;
            }
        } else {
            self.move_to(self.selected_range.end, cx);
        }
        self.selected_lines = 0..0;
    }

    pub fn select_left(&mut self, _: &SelectLeft, cx: &mut ViewContext<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
        self.is_selecting = true;
    }

    pub fn select_right(&mut self, _: &SelectRight, cx: &mut ViewContext<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
        self.is_selecting = true;
    }

    pub fn select_up(&mut self, _: &SelectUp, _cx: &mut ViewContext<Self>) {
        if !self.is_selecting {
            // don't set sel range start because it's same
            self.selected_lines.start = self.focused_line;
        }
        self.focused_line = max(0 as i32, self.focused_line as i32 - 1) as usize;
        let pos = min(self.content[self.focused_line].len(), self.cursor_pos);
        if !self.selection_reversed {
            self.selected_range.end = pos;
        }else {
            self.selected_range.start = pos;
        }
        self.selected_lines.end = self.focused_line + 1;
        self.is_selecting = true;
        self.selected_lines_reversed = self.selected_lines.start >= self.selected_lines.end;
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
    }

    pub fn select_down(&mut self, _: &SelectDown, _cx: &mut ViewContext<Self>) {
        if !self.is_selecting {
            self.selected_lines.start = self.focused_line;
        }
        self.focused_line = min(self.lines - 1, self.focused_line + 1);
        let pos = min(self.content[self.focused_line].len(), self.cursor_pos);
        if !self.selection_reversed {
            self.selected_range.end = pos;
        }else {
            self.selected_range.start = pos;
        }
        self.selected_lines.end = self.focused_line + 1; 
        self.is_selecting = true;
        self.selected_lines_reversed = self.selected_lines.start >= self.selected_lines.end;
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
    }

    pub fn select_all(&mut self, _: &SelectAll, cx: &mut ViewContext<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content[self.focused_line].len(), cx)
    }

    pub fn home(&mut self, _: &Home, cx: &mut ViewContext<Self>) {
        self.move_to(0, cx);
    }

    pub fn end(&mut self, _: &End, cx: &mut ViewContext<Self>) {
        self.move_to(self.content[self.focused_line].len(), cx);
    }

    pub fn backspace(&mut self, _: &Backspace, cx: &mut ViewContext<Self>) {
        if !self.is_selecting && self.focused_line != 0 && self.selected_range.start == 0 {
            self.focused_line -= 1;
            // append line to above line
            self.content[self.focused_line] = (self.content[self.focused_line].to_string()
                + &self.content[self.focused_line + 1].to_string())
                .into();
            // remove the line we appended
            self.content.remove(self.focused_line + 1);
            // jump to end of the line
            self.cursor_pos = self.content[self.focused_line].len();
            self.selected_range = self.cursor_pos..self.cursor_pos;
            self.lines -= 1;
            return;
        }

        if !self.is_selecting {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx);
        }

        let this: &mut TextInput = &mut *self;
        let mut range = None
            .as_ref()
            .map(|range_utf16| this.range_from_utf16(range_utf16))
            .or(this.marked_range.clone())
            .unwrap_or(self.normalized_selection_bounds().clone());

            let selected_lines = if !self.selected_lines_reversed {
                self.selected_lines.clone()
            }else {
                println!("{:?}", self.selected_lines);
                (self.selected_lines.end - 1)..(self.selected_lines.start + 1)
            };
            println!("{:?}", selected_lines);

        let mut lines_to_merge = vec![]; // lines to wrap because deleted '\n'

        for line in selected_lines.clone() {
            // single line
            if line == selected_lines.start && line == selected_lines.end - 1 {
                if range.end < range.start {
                    swap(&mut range.end, &mut range.start);
                }
                self.content[line] = (self.content[line][0..range.start].to_owned() + 
                    &self.content[line][range.end..]).into();

                break;
            }

            if line == selected_lines.start {
                println!("test {} , {}", range.start, self.content[line].len());
                // let start = min(range.start, self.content[line].len());
                self.content[line] = self.content[line][0..range.start].to_owned().into();
            }else if line == selected_lines.end - 1 {
                println!("else {} , {}", range.start, self.content[line].len());
                self.content[line] = self.content[line][range.end..].to_owned().into();
                // deletes the "newline" remove the newline
                lines_to_merge.push(line);
            }else {
                self.content[line] = "".into();
                lines_to_merge.push(line);
            }
        }

        for (i, line) in lines_to_merge.into_iter().enumerate() {
            if self.content[line - i] != "" { 
                // if still content (line == selected_lines.end - 1), append before deleting
                self.content[(line - i) - 1] = (self.content[(line - i) - 1].to_string() + 
                        &self.content[line - i].to_string()).into();
            }
            self.content.remove(line - i); // adjust for already taken
            self.lines -= 1;
        }

        cx.notify();

        self.focused_line = selected_lines.start;
        self.selected_lines = self.focused_line..self.focused_line + 1;
        self.selected_range = range.start..range.start;
        self.selected_lines_reversed = false;
        self.is_selecting = false;
    }

    pub fn delete(&mut self, _: &Delete, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", cx)
    }

    pub fn on_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut ViewContext<Self>) {
        self.focused_line = (event.position.y / px(30.0)) as usize;
        
        self.is_selecting = true;

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx)
        }
    }

    pub fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut ViewContext<Self>) {
        self.is_selecting = false;
    }

    pub fn on_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut ViewContext<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    pub fn show_character_palette(&mut self, _: &ShowCharacterPalette, cx: &mut ViewContext<Self>) {
        cx.show_character_palette();
    }

    pub fn paste(&mut self, _: &Paste, cx: &mut ViewContext<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text.replace("\n", " "), cx);
        }
    }

    pub fn copy(&mut self, _: &Copy, cx: &mut ViewContext<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                (&self.content[self.focused_line][self.selected_range.clone()]).to_string(),
            ));
        }
    }
    pub fn cut(&mut self, _: &Copy, cx: &mut ViewContext<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                (&self.content[self.focused_line][self.selected_range.clone()]).to_string(),
            ));
            self.replace_text_in_range(None, "", cx)
        }
    }

    pub fn move_to(&mut self, offset: usize, cx: &mut ViewContext<Self>) {
        self.selected_range = offset..offset;
        self.cursor_pos = offset;
        self.is_selecting = false;
        cx.notify()
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn normalized_selection_bounds(&self) -> Range<usize> {
        // invert on lines and range
        self.normalized_selection_bounds_lines(
            self.normalized_selection_bounds_range(self.selected_range.clone()))
    }

    pub fn normalized_selection_bounds_range(&self, selected_range: Range<usize>) -> Range<usize> {
        if !self.selection_reversed {
            selected_range
        } else {
            Range { start: selected_range.end, end: selected_range.start }
        }
    }

    pub fn normalized_selection_bounds_lines(&self,  selected_range: Range<usize>) -> Range<usize> {
        if !self.selected_lines_reversed {
            selected_range
        } else {
            Range { start: selected_range.end, end: selected_range.start }
        }
    }

    pub fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content[self.focused_line].is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content[self.focused_line].len();
        }
        line.unwrapped_layout
            .closest_index_for_x(position.x - bounds.left())
    }

    pub fn select_to(&mut self, offset: usize, cx: &mut ViewContext<Self>) {
        // self.selection_reversed == l-r
        if self.selection_reversed {
            self.selected_range.start = offset;
            self.cursor_pos = self.selected_range.start
        } else {
            self.selected_range.end = offset;
            self.cursor_pos = self.selected_range.end
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        if !self.is_selecting {
            self.selected_lines.start = self.focused_line;
        }
        self.selected_lines = self.selected_lines.start..(self.focused_line + 1); 
        cx.notify()
    }

    pub fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content[self.focused_line].chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    pub fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content[self.focused_line].chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content[self.focused_line]
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content[self.focused_line]
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content[self.focused_line].len())
    }

    pub fn reset(&mut self) {
        self.content[self.focused_line] = "".into();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        self.is_selecting = false;
    }
}

impl ViewInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _cx: &mut ViewContext<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[self.focused_line][range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _cx: &mut ViewContext<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _cx: &mut ViewContext<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _cx: &mut ViewContext<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        cx: &mut ViewContext<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content[self.focused_line] = (self.content[self.focused_line][0..range.start]
            .to_owned()
            + new_text
            + &self.content[self.focused_line][range.end..])
            .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        cx: &mut ViewContext<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content[self.focused_line] = (self.content[self.focused_line][0..range.start]
            .to_owned()
            + new_text
            + &self.content[self.focused_line][range.end..])
            .into();
        self.marked_range = Some(range.start..range.start + new_text.len());
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _cx: &mut ViewContext<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.unwrapped_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.unwrapped_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }
}
