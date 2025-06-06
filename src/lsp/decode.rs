use std::ops::Range;

#[derive(Clone)]
pub struct Diagnostics {
    pub diagnostic_range: Range<usize>,
    pub is_error: bool, // else error
    pub message: String,
}