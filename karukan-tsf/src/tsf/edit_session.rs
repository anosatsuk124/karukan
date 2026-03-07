//! ITfEditSession implementation — applies engine actions to the TSF context.
//!
//! TSF requires all text modifications to happen within an EditSession callback.
//! This module provides session types for preedit updates and text commits.

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::*;
#[cfg(target_os = "windows")]
use windows::Win32::UI::TextServices::*;
#[cfg(target_os = "windows")]
use windows::core::*;

use karukan_im::EngineAction;

/// An edit session that applies a list of EngineActions to the TSF context.
#[cfg(target_os = "windows")]
#[implement(ITfEditSession)]
pub struct ActionEditSession {
    context: ITfContext,
    actions: Vec<EngineAction>,
    composition: Option<ITfComposition>,
    client_id: u32,
}

#[cfg(target_os = "windows")]
impl ActionEditSession {
    pub fn new(
        context: ITfContext,
        actions: Vec<EngineAction>,
        composition: Option<ITfComposition>,
        client_id: u32,
    ) -> Self {
        Self {
            context,
            actions,
            composition,
            client_id,
        }
    }

    /// Request and execute this edit session synchronously.
    pub fn execute(self) -> Result<()> {
        let context = self.context.clone();
        let session: ITfEditSession = self.into();
        unsafe {
            let mut hr = HRESULT::default();
            context.RequestEditSession(
                self.client_id,
                &session,
                TF_ES_SYNC | TF_ES_READWRITE,
                &mut hr,
            )?;
            hr.ok()
        }
    }
}

#[cfg(target_os = "windows")]
impl ITfEditSession_Impl for ActionEditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        for action in &self.actions {
            match action {
                EngineAction::UpdatePreedit(preedit) => {
                    self.update_preedit(ec, preedit)?;
                }
                EngineAction::Commit(text) => {
                    self.commit_text(ec, text)?;
                }
                EngineAction::ShowCandidates(_candidates) => {
                    // TODO: Show candidate window
                    tracing::debug!("EditSession: ShowCandidates (not yet implemented)");
                }
                EngineAction::HideCandidates => {
                    // TODO: Hide candidate window
                    tracing::debug!("EditSession: HideCandidates (not yet implemented)");
                }
                EngineAction::UpdateAuxText(text) => {
                    tracing::debug!("EditSession: UpdateAuxText: {}", text);
                }
                EngineAction::HideAuxText => {
                    tracing::debug!("EditSession: HideAuxText");
                }
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl ActionEditSession {
    /// Update the preedit (composition) text in the TSF context.
    fn update_preedit(
        &self,
        ec: u32,
        preedit: &karukan_im::core::preedit::Preedit,
    ) -> Result<()> {
        let text = preedit.text();
        let text_wide: Vec<u16> = text.encode_utf16().collect();

        if let Some(ref composition) = self.composition {
            unsafe {
                let range = composition.GetRange()?;
                range.SetText(ec, 0, &text_wide)?;
            }
        } else {
            // Start a new composition if needed
            // TODO: Create composition via ITfContextComposition
            tracing::debug!("EditSession: Need to start composition for preedit: {}", text);
        }

        Ok(())
    }

    /// Commit text to the application and end the composition.
    fn commit_text(
        &self,
        ec: u32,
        text: &str,
    ) -> Result<()> {
        let text_wide: Vec<u16> = text.encode_utf16().collect();

        if let Some(ref composition) = self.composition {
            unsafe {
                let range = composition.GetRange()?;
                range.SetText(ec, 0, &text_wide)?;
                composition.EndComposition(ec)?;
            }
        }

        Ok(())
    }
}
