//! Win32 candidate window implementation.
//!
//! Creates a topmost popup window that displays conversion candidates
//! near the text cursor position. Uses GDI for rendering.

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::*;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::*;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::*;
#[cfg(target_os = "windows")]
use windows::core::*;

#[cfg(target_os = "windows")]
const BASE_CANDIDATE_ITEM_HEIGHT: i32 = 24;
#[cfg(target_os = "windows")]
const BASE_CANDIDATE_PADDING: i32 = 4;
#[cfg(target_os = "windows")]
const BASE_CHAR_WIDTH: i32 = 14;
#[cfg(target_os = "windows")]
const BASE_LABEL_WIDTH: i32 = 28;
#[cfg(target_os = "windows")]
const BASE_MIN_WIDTH: i32 = 120;
#[cfg(target_os = "windows")]
const BASE_FONT_SIZE: i32 = 16;
#[cfg(target_os = "windows")]
const WINDOW_CLASS_NAME: PCWSTR = w!("KarukanCandidateWindow");

/// Get the DPI scale factor for the given window.
/// Returns 1.0 for 96 DPI (100%), 1.25 for 120 DPI (125%), etc.
#[cfg(target_os = "windows")]
fn dpi_scale(hwnd: HWND) -> f64 {
    use windows::Win32::UI::HiDpi::GetDpiForWindow;
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    if dpi == 0 {
        1.0
    } else {
        dpi as f64 / 96.0
    }
}

/// Scale a pixel value by the DPI scale factor.
#[cfg(target_os = "windows")]
fn scale(value: i32, s: f64) -> i32 {
    (value as f64 * s).round() as i32
}

/// Render data shared between the candidate window and the WNDPROC.
#[cfg(target_os = "windows")]
#[derive(Default)]
struct CandidateRenderData {
    candidates: Vec<String>,
    selected: usize,
}

/// Candidate window for displaying conversion candidates.
#[cfg(target_os = "windows")]
pub struct CandidateWindow {
    hwnd: HWND,
}

#[cfg(target_os = "windows")]
#[allow(clippy::new_without_default)]
impl CandidateWindow {
    /// Create a new candidate window (hidden by default).
    ///
    /// Render data is stored per-window via `GWLP_USERDATA` (no global static).
    pub fn new() -> Self {
        register_window_class();
        let hwnd = create_candidate_window();

        // Allocate per-instance render data and attach to the window
        if hwnd.0 as usize != 0 {
            let data = Box::new(CandidateRenderData::default());
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as isize);
            }
        }

        Self { hwnd }
    }

    /// Show the candidate window with the given candidates.
    pub fn show(&mut self, candidates: &[String], selected: usize) {
        if self.hwnd.0 as usize == 0 {
            return;
        }

        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut CandidateRenderData;
            if !ptr.is_null() {
                (*ptr).candidates = candidates.to_vec();
                (*ptr).selected = selected;
            }
        }

        let s = dpi_scale(self.hwnd);
        let item_height = scale(BASE_CANDIDATE_ITEM_HEIGHT, s);
        let padding = scale(BASE_CANDIDATE_PADDING, s);

        let count = candidates.len().max(1) as i32;
        let height = count * item_height + padding * 2;
        let width = self.calculate_width(candidates, s);

        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                0,
                0,
                width,
                height,
                SWP_NOMOVE | SWP_NOACTIVATE,
            );
            let _ = InvalidateRect(self.hwnd, None, true);
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
    }

    /// Hide the candidate window.
    pub fn hide(&mut self) {
        if self.hwnd.0 as usize == 0 {
            return;
        }
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Update the selected candidate index.
    pub fn set_selected(&mut self, index: usize) {
        if self.hwnd.0 as usize == 0 {
            return;
        }
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut CandidateRenderData;
            if !ptr.is_null() {
                (*ptr).selected = index;
            }
            let _ = InvalidateRect(self.hwnd, None, true);
        }
    }

    /// Move the window to the given screen coordinates.
    pub fn move_to(&mut self, x: i32, y: i32) {
        if self.hwnd.0 as usize == 0 {
            return;
        }
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    /// Destroy the window and free the per-instance render data.
    pub fn destroy(&mut self) {
        if self.hwnd.0 as usize != 0 {
            unsafe {
                // Reclaim and drop the per-instance render data
                let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut CandidateRenderData;
                if !ptr.is_null() {
                    SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
                    let _ = Box::from_raw(ptr);
                }
                let _ = DestroyWindow(self.hwnd);
            }
            self.hwnd = HWND::default();
        }
    }

    fn calculate_width(&self, candidates: &[String], s: f64) -> i32 {
        let max_chars = candidates
            .iter()
            .map(|c| c.chars().count())
            .max()
            .unwrap_or(4);
        let char_width = scale(BASE_CHAR_WIDTH, s);
        let label_width = scale(BASE_LABEL_WIDTH, s);
        let padding = scale(BASE_CANDIDATE_PADDING, s);
        (max_chars as i32 * char_width + label_width + padding * 2).max(scale(BASE_MIN_WIDTH, s))
    }
}

#[cfg(target_os = "windows")]
impl Drop for CandidateWindow {
    fn drop(&mut self) {
        self.destroy();
    }
}

#[cfg(target_os = "windows")]
fn register_window_class() {
    use std::sync::Once;
    static REGISTERED: Once = Once::new();
    REGISTERED.call_once(|| unsafe {
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(candidate_wnd_proc),
            hbrBackground: GetSysColorBrush(COLOR_WINDOW),
            lpszClassName: WINDOW_CLASS_NAME,
            ..Default::default()
        };
        RegisterClassW(&wc);
    });
}

#[cfg(target_os = "windows")]
fn create_candidate_window() -> HWND {
    unsafe {
        use windows::Win32::UI::HiDpi::{
            SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };

        // Temporarily set per-monitor DPI awareness for this window creation.
        // This ensures GetDpiForWindow returns correct per-monitor DPI values
        // even if the host application is DPI-unaware.
        let prev_context =
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            WINDOW_CLASS_NAME,
            w!(""),
            WS_POPUP | WS_BORDER,
            0,
            0,
            200,
            100,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_default();

        // Restore previous DPI awareness context
        if !prev_context.is_invalid() {
            SetThreadDpiAwarenessContext(prev_context);
        }

        hwnd
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn candidate_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            unsafe { paint_candidates(hwnd) };
            LRESULT(0)
        }
        WM_DPICHANGED => {
            // When the window moves to a monitor with different DPI,
            // resize to the system-suggested rect and repaint.
            unsafe {
                let suggested_rect = &*(lparam.0 as *const RECT);
                let _ = SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    suggested_rect.left,
                    suggested_rect.top,
                    suggested_rect.right - suggested_rect.left,
                    suggested_rect.bottom - suggested_rect.top,
                    SWP_NOACTIVATE | SWP_NOZORDER,
                );
                let _ = InvalidateRect(hwnd, None, true);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(target_os = "windows")]
unsafe fn paint_candidates(hwnd: HWND) {
    unsafe {
        let s = dpi_scale(hwnd);
        let item_height = scale(BASE_CANDIDATE_ITEM_HEIGHT, s);
        let padding = scale(BASE_CANDIDATE_PADDING, s);
        let text_y_offset = scale(3, s);
        let font_size = scale(BASE_FONT_SIZE, s);

        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);

        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const CandidateRenderData;
        if !ptr.is_null() {
            let data = &*ptr;
            let _ = SetBkMode(hdc, TRANSPARENT);

            // Create a DPI-scaled font for clear rendering
            let font = CreateFontW(
                -font_size,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET.0 as u32,
                OUT_DEFAULT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32,
                CLEARTYPE_QUALITY.0 as u32,
                (FF_DONTCARE.0 | FIXED_PITCH.0) as u32,
                w!("Meiryo UI"),
            );
            let old_font = SelectObject(hdc, font);

            // Highlight color for selected item
            let highlight_brush = CreateSolidBrush(COLORREF(0x00D77800));

            for (i, candidate) in data.candidates.iter().enumerate() {
                let y = padding + i as i32 * item_height;

                if i == data.selected {
                    let rect = RECT {
                        left: 0,
                        top: y,
                        right: ps.rcPaint.right,
                        bottom: y + item_height,
                    };
                    FillRect(hdc, &rect, highlight_brush);
                    SetTextColor(hdc, COLORREF(0x00FFFFFF));
                } else {
                    SetTextColor(hdc, COLORREF(0x00000000));
                }

                // Draw "N. candidate" label
                let label = format!("{}. {}", i + 1, candidate);
                let label_wide: Vec<u16> = label.encode_utf16().collect();
                let _ = TextOutW(hdc, padding, y + text_y_offset, &label_wide);
            }

            let _ = DeleteObject(highlight_brush);
            SelectObject(hdc, old_font);
            let _ = DeleteObject(font);
        }

        let _ = EndPaint(hwnd, &ps);
    }
}
