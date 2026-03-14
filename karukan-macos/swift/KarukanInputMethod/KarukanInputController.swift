import Cocoa
import InputMethodKit

// MARK: - Debug Logging

#if DEBUG
private func debugLog(_ message: String) {
    let logDir = FileManager.default.homeDirectoryForCurrentUser
        .appendingPathComponent("Library/Logs/Karukan")
    try? FileManager.default.createDirectory(at: logDir, withIntermediateDirectories: true)
    let logFile = logDir.appendingPathComponent("debug.log")
    let timestamp = ISO8601DateFormatter().string(from: Date())
    let line = "[\(timestamp)] \(message)\n"
    if let handle = try? FileHandle(forWritingTo: logFile) {
        handle.seekToEndOfFile()
        handle.write(line.data(using: .utf8)!)
        handle.closeFile()
    } else {
        try? line.write(to: logFile, atomically: true, encoding: .utf8)
    }
}
#endif

/// Main input controller for the Karukan input method.
/// Each client application gets its own instance of this controller.
@objc(KarukanInputController)
class KarukanInputController: IMKInputController {
    private var engine: OpaquePointer?
    private var imkCandidates: IMKCandidates?
    private var currentCandidates: [(String, String)] = []
    private var currentCandidateCursor: Int = 0

    // MARK: - Lifecycle

    override init!(server: IMKServer!, delegate: Any!, client inputClient: Any!) {
        super.init(server: server, delegate: delegate, client: inputClient)
        engine = karukan_macos_engine_new()
        imkCandidates = IMKCandidates(server: server, panelType: kIMKSingleColumnScrollingCandidatePanel)
    }

    deinit {
        if let engine = engine {
            karukan_macos_engine_free(engine)
        }
    }

    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        if let engine = engine {
            let result = karukan_macos_engine_init(engine)
            if result != 0 {
                NSLog("Karukan: Engine initialization failed")
            }
        }
    }

    override func deactivateServer(_ sender: Any!) {
        // Commit any pending text before deactivation
        if let engine = engine, let client = sender as? (any IMKTextInput) {
            if karukan_macos_commit(engine) != 0 {
                let commitText = getString(karukan_macos_get_commit(engine))
                if !commitText.isEmpty {
                    client.insertText(commitText, replacementRange: NSRange(location: NSNotFound, length: 0))
                }
            }
            karukan_macos_save_learning(engine)
        }
        imkCandidates?.hide()
        super.deactivateServer(sender)
    }

    // MARK: - Key Handling

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event = event, let engine = engine, let client = sender as? (any IMKTextInput) else {
            return false
        }

        // Only handle key down events (IMKit doesn't usually send key up)
        guard event.type == .keyDown || event.type == .keyUp else {
            return false
        }

        let isPress = event.type == .keyDown
        let keycode = event.keyCode
        let modifierFlags = event.modifierFlags

        // Extract unicode character from the event
        var unicodeChar: UInt32 = 0
        var hasUnicode: Int32 = 0
        if let chars = event.characters, let scalar = chars.unicodeScalars.first {
            unicodeChar = scalar.value
            hasUnicode = 1
        }

        // Extract modifier states
        let shift: Int32 = modifierFlags.contains(.shift) ? 1 : 0
        let ctrl: Int32 = modifierFlags.contains(.control) ? 1 : 0
        let opt: Int32 = modifierFlags.contains(.option) ? 1 : 0
        let cmd: Int32 = modifierFlags.contains(.command) ? 1 : 0

        // Pass Command key events through (standard macOS shortcuts)
        if cmd != 0 {
            return false
        }

        let consumed = karukan_macos_process_key(
            engine,
            keycode,
            unicodeChar,
            hasUnicode,
            shift, ctrl, opt, cmd,
            isPress ? 1 : 0
        )

        // Process results
        processEngineResult(client: client)

        return consumed != 0
    }

    // MARK: - IMKCandidates Delegate

    override func candidates(_ sender: Any!) -> [Any]! {
        return currentCandidates.map { text, annotation in
            annotation.isEmpty ? text : "\(text)  \(annotation)"
        }
    }

    override func candidateSelected(_ candidateString: NSAttributedString!) {
        guard let text = candidateString?.string, let client = self.client() else { return }
        client.insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0))
    }

    // MARK: - Engine Result Processing

    private func processEngineResult(client: any IMKTextInput) {
        guard let engine = engine else { return }

        // Handle commit first (before preedit update)
        if karukan_macos_has_commit(engine) != 0 {
            let commitText = getString(karukan_macos_get_commit(engine))
            if !commitText.isEmpty {
                client.insertText(commitText, replacementRange: NSRange(location: NSNotFound, length: 0))
            }
        }

        // Handle preedit update
        if karukan_macos_has_preedit(engine) != 0 {
            let preeditText = getString(karukan_macos_get_preedit(engine))
            let caretBytes = Int(karukan_macos_get_preedit_caret(engine))

            if preeditText.isEmpty {
                // Clear preedit
                client.setMarkedText(
                    NSAttributedString(string: ""),
                    selectionRange: NSRange(location: 0, length: 0),
                    replacementRange: NSRange(location: NSNotFound, length: 0)
                )
            } else {
                // Convert byte offset to character offset for NSRange
                let caretCharOffset = preeditText.utf8.prefix(caretBytes).count == caretBytes
                    ? String(preeditText.utf8.prefix(caretBytes))?.count ?? preeditText.count
                    : preeditText.count

                let attrString = buildPreeditAttributedString(preeditText)
                client.setMarkedText(
                    attrString,
                    selectionRange: NSRange(location: caretCharOffset, length: 0),
                    replacementRange: NSRange(location: NSNotFound, length: 0)
                )
            }
        }

        // Handle candidates
        if karukan_macos_has_candidates(engine) != 0 {
            if karukan_macos_should_hide_candidates(engine) != 0 {
                imkCandidates?.hide()
            } else {
                let count = Int(karukan_macos_get_candidate_count(engine))
                currentCandidateCursor = Int(karukan_macos_get_candidate_cursor(engine))
                currentCandidates = (0..<count).map { i in
                    let text = getString(karukan_macos_get_candidate(engine, UInt32(i)))
                    let annotation = getString(karukan_macos_get_candidate_annotation(engine, UInt32(i)))
                    return (text, annotation)
                }
                imkCandidates?.update()
                imkCandidates?.show(kIMKLocateCandidatesBelowHint)
            }
        }
    }

    // MARK: - Helpers

    private func getString(_ cstr: UnsafePointer<CChar>?) -> String {
        guard let cstr = cstr else { return "" }
        return String(cString: cstr)
    }

    private func buildPreeditAttributedString(_ text: String) -> NSAttributedString {
        let attrStr = NSMutableAttributedString(string: text)
        let range = NSRange(location: 0, length: attrStr.length)
        // Apply underline to the entire preedit text (standard IME visual)
        attrStr.addAttribute(.underlineStyle, value: NSUnderlineStyle.single.rawValue, range: range)
        attrStr.addAttribute(.markedClauseSegment, value: 0, range: range)
        return attrStr
    }
}
