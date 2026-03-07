//! ITfKeyEventSink implementation — handles keyboard input for TSF.

use windows::Win32::Foundation::*;
use windows::Win32::UI::TextServices::*;
use windows::core::*;

use crate::tsf::text_input_processor::KarukanTextService;

impl ITfKeyEventSink_Impl for KarukanTextService_Impl {
    /// Called before OnKeyDown — determines if the key will be consumed.
    ///
    /// We process the key through the engine here and cache the result.
    /// If consumed, TSF will call OnKeyDown where we apply the cached actions.
    fn OnTestKeyDown(
        &self,
        _pic: Option<&ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
        pfeaten: *mut BOOL,
    ) -> Result<()> {
        unsafe {
            if pfeaten.is_null() {
                return Err(E_POINTER.into());
            }

            let vk = wparam.0 as u32;
            let (shift, control, alt, win) = get_modifier_state();
            let unicode_char = vk_to_unicode(vk, shift);

            let mut inner = self.inner.borrow_mut();
            let result = inner.engine.process_key(
                vk, unicode_char, shift, control, alt, win, true,
            );

            let consumed = result.consumed;
            inner.cached_result = Some(result);

            *pfeaten = BOOL::from(consumed);
            Ok(())
        }
    }

    /// Called when a key is pressed and OnTestKeyDown returned TRUE.
    ///
    /// Applies the cached EngineResult actions via an EditSession.
    fn OnKeyDown(
        &self,
        pic: Option<&ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
        pfeaten: *mut BOOL,
    ) -> Result<()> {
        unsafe {
            if pfeaten.is_null() {
                return Err(E_POINTER.into());
            }

            let mut inner = self.inner.borrow_mut();

            if let Some(result) = inner.cached_result.take() {
                if result.consumed && !result.actions.is_empty() {
                    // Apply actions via EditSession
                    if let Some(context) = pic {
                        drop(inner); // Release borrow before edit session
                        apply_engine_actions(self, context, &result.actions)?;
                        *pfeaten = TRUE;
                        return Ok(());
                    }
                }
                *pfeaten = BOOL::from(result.consumed);
            } else {
                // No cached result — should not happen, but handle gracefully
                *pfeaten = FALSE;
            }

            Ok(())
        }
    }

    /// Called before OnKeyUp — we generally don't consume key-up events.
    fn OnTestKeyUp(
        &self,
        _pic: Option<&ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
        pfeaten: *mut BOOL,
    ) -> Result<()> {
        unsafe {
            if !pfeaten.is_null() {
                *pfeaten = FALSE;
            }
            Ok(())
        }
    }

    /// Called when a key is released — we generally don't consume key-up events.
    fn OnKeyUp(
        &self,
        _pic: Option<&ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
        pfeaten: *mut BOOL,
    ) -> Result<()> {
        unsafe {
            if !pfeaten.is_null() {
                *pfeaten = FALSE;
            }
            Ok(())
        }
    }

    /// Called for preserved keys (e.g., Hankaku/Zenkaku toggle).
    fn OnPreservedKey(
        &self,
        _pic: Option<&ITfContext>,
        _rguid: *const GUID,
        pfeaten: *mut BOOL,
    ) -> Result<()> {
        unsafe {
            if !pfeaten.is_null() {
                // TODO: Handle IME toggle (Hankaku/Zenkaku key)
                *pfeaten = FALSE;
            }
            Ok(())
        }
    }
}

/// Get the current modifier key state using GetKeyState.
#[cfg(target_os = "windows")]
fn get_modifier_state() -> (bool, bool, bool, bool) {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;

    unsafe {
        let shift = GetKeyState(0x10) < 0; // VK_SHIFT
        let control = GetKeyState(0x11) < 0; // VK_CONTROL
        let alt = GetKeyState(0x12) < 0; // VK_MENU
        let win = GetKeyState(0x5B) < 0 || GetKeyState(0x5C) < 0; // VK_LWIN | VK_RWIN
        (shift, control, alt, win)
    }
}

#[cfg(not(target_os = "windows"))]
fn get_modifier_state() -> (bool, bool, bool, bool) {
    (false, false, false, false)
}

/// Convert a VK code to a Unicode character using ToUnicode.
#[cfg(target_os = "windows")]
fn vk_to_unicode(vk: u32, _shift: bool) -> Option<char> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    unsafe {
        let scan_code = MapVirtualKeyW(vk, MAP_VIRTUAL_KEY_TYPE(0)); // MAPVK_VK_TO_VSC
        let mut keyboard_state = [0u8; 256];
        GetKeyboardState(&mut keyboard_state).ok()?;

        let mut buf = [0u16; 4];
        let result = ToUnicode(vk, scan_code, Some(&keyboard_state), &mut buf, 0);
        if result == 1 {
            char::from_u32(buf[0] as u32)
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn vk_to_unicode(_vk: u32, _shift: bool) -> Option<char> {
    None
}

/// Apply engine actions to the TSF context via an EditSession.
///
/// This is a placeholder — the actual implementation will be in edit_session.rs.
fn apply_engine_actions(
    _service: &KarukanTextService_Impl,
    _context: &ITfContext,
    actions: &[karukan_im::EngineAction],
) -> Result<()> {
    // TODO: Implement EditSession-based action application
    // For now, log the actions for debugging
    for action in actions {
        tracing::debug!("TSF action: {:?}", action);
    }
    Ok(())
}
