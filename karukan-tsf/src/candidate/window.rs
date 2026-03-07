//! Win32 candidate window implementation.
//!
//! Creates a topmost layered window that displays conversion candidates
//! near the text cursor position. Uses Direct2D for rendering.

/// Candidate window for displaying conversion candidates.
///
/// This is a stub implementation. The full implementation will:
/// - Create an HWND_TOPMOST layered window
/// - Use Direct2D + DirectWrite for Japanese text rendering
/// - Position the window using ITfContextView::GetTextExt
/// - Handle mouse clicks for candidate selection
/// - Display number labels (1-9) for keyboard selection
#[cfg(target_os = "windows")]
pub struct CandidateWindow {
    // hwnd: windows::Win32::Foundation::HWND,
    // visible: bool,
    // candidates: Vec<String>,
    // selected: usize,
}

#[cfg(target_os = "windows")]
impl CandidateWindow {
    /// Create a new candidate window (hidden by default).
    pub fn new() -> Self {
        // TODO: Create Win32 window with Direct2D rendering
        Self {}
    }

    /// Show the candidate window with the given candidates.
    pub fn show(&mut self, _candidates: &[String], _selected: usize) {
        // TODO: Update candidate list, position window, show
    }

    /// Hide the candidate window.
    pub fn hide(&mut self) {
        // TODO: Hide the window
    }

    /// Update the selected candidate index.
    pub fn set_selected(&mut self, _index: usize) {
        // TODO: Redraw with new selection
    }

    /// Move the window to the given screen coordinates.
    pub fn move_to(&mut self, _x: i32, _y: i32) {
        // TODO: SetWindowPos
    }
}
