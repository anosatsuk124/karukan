//! ITfTextInputProcessorEx implementation — the main TSF entry point.

use std::cell::RefCell;

use windows::Win32::Foundation::*;
use windows::Win32::UI::TextServices::*;
use windows::core::*;

use crate::engine_bridge::EngineBridge;
use crate::globals::{dll_add_ref, dll_release, CLSID_KARUKAN_TEXT_SERVICE, GUID_KARUKAN_PROFILE, LANGID_JAPANESE};

/// The main karukan text service COM object.
///
/// Implements ITfTextInputProcessorEx (and its base ITfTextInputProcessor),
/// ITfKeyEventSink, ITfCompositionSink, and ITfDisplayAttributeProvider.
#[implement(
    ITfTextInputProcessorEx,
    ITfTextInputProcessor,
    ITfKeyEventSink,
    ITfCompositionSink,
    ITfDisplayAttributeProvider,
    ITfThreadMgrEventSink,
)]
pub struct KarukanTextService {
    inner: RefCell<KarukanTextServiceInner>,
}

struct KarukanTextServiceInner {
    thread_mgr: Option<ITfThreadMgr>,
    client_id: u32,
    engine: EngineBridge,
    composition: Option<ITfComposition>,
    /// Cookie for ITfThreadMgrEventSink
    thread_mgr_sink_cookie: u32,
    /// Cookie for ITfKeyEventSink
    keystroke_mgr_cookie: bool,
    /// Cached EngineResult from OnTestKeyDown for reuse in OnKeyDown
    cached_result: Option<karukan_im::EngineResult>,
}

impl KarukanTextService {
    pub fn new() -> Self {
        dll_add_ref();
        Self {
            inner: RefCell::new(KarukanTextServiceInner {
                thread_mgr: None,
                client_id: 0,
                engine: EngineBridge::new(),
                composition: None,
                thread_mgr_sink_cookie: TF_INVALID_COOKIE,
                keystroke_mgr_cookie: false,
                cached_result: None,
            }),
        }
    }
}

impl Drop for KarukanTextService {
    fn drop(&mut self) {
        dll_release();
    }
}

// ITfTextInputProcessor implementation
impl ITfTextInputProcessor_Impl for KarukanTextService_Impl {
    fn Activate(&self, ptim: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> Result<()> {
        let mut inner = self.inner.borrow_mut();

        // Save learning cache before deactivation
        inner.engine.save_learning();

        // Unadvise key event sink
        if inner.keystroke_mgr_cookie {
            if let Some(ref thread_mgr) = inner.thread_mgr {
                unsafe {
                    let keystroke_mgr: Result<ITfKeystrokeMgr> = thread_mgr.cast();
                    if let Ok(km) = keystroke_mgr {
                        let _ = km.UnadviseKeyEventSink(inner.client_id);
                    }
                }
            }
            inner.keystroke_mgr_cookie = false;
        }

        // Unadvise thread manager event sink
        if inner.thread_mgr_sink_cookie != TF_INVALID_COOKIE {
            if let Some(ref thread_mgr) = inner.thread_mgr {
                unsafe {
                    let source: Result<ITfSource> = thread_mgr.cast();
                    if let Ok(src) = source {
                        let _ = src.UnadviseSink(inner.thread_mgr_sink_cookie);
                    }
                }
            }
            inner.thread_mgr_sink_cookie = TF_INVALID_COOKIE;
        }

        // End any active composition
        if let Some(composition) = inner.composition.take() {
            unsafe {
                let _ = composition.EndComposition();
            }
        }

        inner.engine.reset();
        inner.thread_mgr = None;
        inner.client_id = 0;

        Ok(())
    }
}

// ITfTextInputProcessorEx implementation
impl ITfTextInputProcessorEx_Impl for KarukanTextService_Impl {
    fn ActivateEx(
        &self,
        ptim: Option<&ITfThreadMgr>,
        tid: u32,
        _dwflags: u32,
    ) -> Result<()> {
        let mut inner = self.inner.borrow_mut();

        let thread_mgr = ptim.ok_or(E_INVALIDARG)?;
        inner.thread_mgr = Some(thread_mgr.clone());
        inner.client_id = tid;

        // Initialize the engine (loads models, dictionaries, learning cache)
        if let Err(e) = inner.engine.initialize() {
            tracing::error!("Failed to initialize engine: {}", e);
            // Continue anyway — engine works without models (romaji-only mode)
        }

        // Advise key event sink
        unsafe {
            let keystroke_mgr: ITfKeystrokeMgr = thread_mgr.cast()?;
            let this_sink: ITfKeyEventSink = self.cast()?;
            keystroke_mgr.AdviseKeyEventSink(tid, &this_sink, TRUE)?;
            inner.keystroke_mgr_cookie = true;
        }

        // Advise thread manager event sink
        unsafe {
            let source: ITfSource = thread_mgr.cast()?;
            let this_sink: ITfThreadMgrEventSink = self.cast()?;
            let cookie = source.AdviseSink(
                &ITfThreadMgrEventSink::IID,
                &this_sink,
            )?;
            inner.thread_mgr_sink_cookie = cookie;
        }

        Ok(())
    }
}
