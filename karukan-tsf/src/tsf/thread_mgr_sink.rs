//! ITfThreadMgrEventSink implementation — tracks document focus changes.

use windows::Win32::UI::TextServices::*;
use windows::core::*;

use crate::tsf::text_input_processor::KarukanTextService;

impl ITfThreadMgrEventSink_Impl for KarukanTextService_Impl {
    /// Called when a document (context) gets focus.
    fn OnInitDocumentMgr(
        &self,
        _pdim: Option<&ITfDocumentMgr>,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when a document manager is being removed.
    fn OnUninitDocumentMgr(
        &self,
        _pdim: Option<&ITfDocumentMgr>,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when focus changes to a new document.
    fn OnSetFocus(
        &self,
        _pdimfocus: Option<&ITfDocumentMgr>,
        _pdimprevfocus: Option<&ITfDocumentMgr>,
    ) -> Result<()> {
        // Reset engine state when focus changes to avoid stale state
        let mut inner = self.inner.borrow_mut();
        inner.engine.reset();
        inner.cached_result = None;
        Ok(())
    }

    /// Called when a context is pushed onto the document manager.
    fn OnPushContext(
        &self,
        _pic: Option<&ITfContext>,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when a context is popped from the document manager.
    fn OnPopContext(
        &self,
        _pic: Option<&ITfContext>,
    ) -> Result<()> {
        Ok(())
    }
}
