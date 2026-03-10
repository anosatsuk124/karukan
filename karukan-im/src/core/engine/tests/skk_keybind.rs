use super::*;
use crate::config::settings::KeybindingProfile;
use crate::core::candidate::{Candidate, CandidateList};

fn make_skk_engine() -> InputMethodEngine {
    let config = EngineConfig {
        keybinding_profile: KeybindingProfile::Skk,
        ..EngineConfig::default()
    };
    InputMethodEngine::with_config(config)
}

/// Set engine to Conversion state with given reading and candidates
fn set_conversion_state(engine: &mut InputMethodEngine, reading: &str, candidates: &[&str]) {
    engine.input_buf.text = reading.to_string();
    engine.input_buf.cursor_pos = reading.chars().count();
    let candidate_list = CandidateList::new(
        candidates
            .iter()
            .enumerate()
            .map(|(i, &text)| Candidate::with_reading(text, reading).with_index(i))
            .collect(),
    );
    engine.state = InputState::Conversion {
        preedit: Preedit::with_text(candidates[0]),
        candidates: candidate_list,
    };
}

fn has_action(result: &EngineResult, check: impl Fn(&EngineAction) -> bool) -> bool {
    result.actions.iter().any(|a| check(a))
}

#[test]
fn test_skk_l_in_empty_hiragana_switches_to_alphabet() {
    let mut engine = make_skk_engine();
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    let result = engine.process_key(&press('l'));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Alphabet);
}

#[test]
fn test_skk_l_in_empty_alphabet_passes_through() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Alphabet;

    let result = engine.process_key(&press('l'));
    // In SKK alphabet mode, all keys pass through to the application
    assert!(!result.consumed);
    assert_eq!(engine.input_mode, InputMode::Alphabet);
}

#[test]
fn test_skk_q_toggle_hiragana_to_katakana() {
    let mut engine = make_skk_engine();
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    let result = engine.process_key(&press('q'));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Katakana);
}

#[test]
fn test_skk_q_toggle_katakana_to_hiragana() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Katakana;

    let result = engine.process_key(&press('q'));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_skk_ctrl_j_enters_hiragana_from_alphabet() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Alphabet;

    let result = engine.process_key(&press_ctrl(Keysym::KEY_J));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_skk_ctrl_j_enters_hiragana_from_katakana() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Katakana;

    let result = engine.process_key(&press_ctrl(Keysym::KEY_J));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_skk_ctrl_j_noop_in_hiragana() {
    let mut engine = make_skk_engine();
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    let result = engine.process_key(&press_ctrl(Keysym::KEY_J));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_skk_ctrl_q_enters_halfwidth_katakana() {
    let mut engine = make_skk_engine();
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    let result = engine.process_key(&press_ctrl(Keysym::KEY_Q));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::HalfWidthKatakana);
}

#[test]
fn test_default_profile_l_passes_through() {
    // With default profile, 'l' should NOT be intercepted by SKK
    let mut engine = InputMethodEngine::new();
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    let result = engine.process_key(&press('l'));
    // 'l' starts romaji composing in default mode (not SKK intercept)
    assert!(result.consumed);
    // Should remain in Hiragana mode (romaji 'l' buffer)
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_default_profile_q_passes_through() {
    // With default profile, 'q' should NOT be intercepted by SKK
    let mut engine = InputMethodEngine::new();
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    let result = engine.process_key(&press('q'));
    // 'q' is a passthrough character in romaji, auto-commits
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_skk_l_in_composing_switches_to_alphabet() {
    let mut engine = make_skk_engine();

    // Type "a" to enter composing state
    engine.process_key(&press('a'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    // Press 'l' to switch to alphabet — composing text should be committed
    let result = engine.process_key(&press('l'));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Alphabet);
    // State should be Empty after commit
    assert!(matches!(engine.state(), InputState::Empty));
    // Should have a Commit action with the composed text
    assert!(
        result
            .actions
            .iter()
            .any(|a| matches!(a, EngineAction::Commit(t) if t == "あ"))
    );
}

#[test]
fn test_skk_alphabet_mode_keys_pass_through() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Alphabet;

    // q should pass through in alphabet mode
    let result = engine.process_key(&press('q'));
    assert!(!result.consumed);

    // Ctrl+q should pass through in alphabet mode
    let result = engine.process_key(&press_ctrl(Keysym::KEY_Q));
    assert!(!result.consumed);

    // Only Ctrl+j should be consumed
    let result = engine.process_key(&press_ctrl(Keysym::KEY_J));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_skk_q_in_composing_commits_katakana() {
    let mut engine = make_skk_engine();

    // Type "a" to get "あ" in composing state
    engine.process_key(&press('a'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));

    // Press 'q' — should commit as katakana and return to hiragana
    let result = engine.process_key(&press('q'));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Empty));
    assert_eq!(engine.input_mode, InputMode::Hiragana);
    assert!(
        result
            .actions
            .iter()
            .any(|a| matches!(a, EngineAction::Commit(t) if t == "ア"))
    );
}

#[test]
fn test_skk_l_in_composing_katakana_commits_katakana() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Katakana;

    // Type "a" to get composing state (hiragana internally)
    engine.process_key(&press('a'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));

    // Press 'l' — should commit as katakana then switch to alphabet
    let result = engine.process_key(&press('l'));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Alphabet);
    assert!(matches!(engine.state(), InputState::Empty));
    assert!(
        result
            .actions
            .iter()
            .any(|a| matches!(a, EngineAction::Commit(t) if t == "ア"))
    );
}

#[test]
fn test_skk_ctrl_q_in_composing_commits_halfwidth_katakana() {
    let mut engine = make_skk_engine();

    // Type "a" to get "あ" in composing state
    engine.process_key(&press('a'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));

    // Press Ctrl+q — should commit as half-width katakana and return to hiragana
    let result = engine.process_key(&press_ctrl(Keysym::KEY_Q));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Empty));
    assert_eq!(engine.input_mode, InputMode::Hiragana);
    assert!(
        result
            .actions
            .iter()
            .any(|a| matches!(a, EngineAction::Commit(t) if t == "ｱ"))
    );
}

#[test]
fn test_skk_ctrl_q_in_empty_switches_mode() {
    let mut engine = make_skk_engine();
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    // Ctrl+q in empty state switches to half-width katakana mode
    let result = engine.process_key(&press_ctrl(Keysym::KEY_Q));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::HalfWidthKatakana);
    assert!(matches!(engine.state(), InputState::Empty));
}

// --- Zenkaku/Hankaku, HENKAN, MUHENKAN tests ---

#[test]
fn test_skk_zenkaku_hankaku_hiragana_to_alphabet() {
    let mut engine = make_skk_engine();
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    let result = engine.process_key(&press_key(Keysym::ZENKAKU_HANKAKU));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Alphabet);
}

#[test]
fn test_skk_zenkaku_hankaku_alphabet_to_hiragana() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Alphabet;

    let result = engine.process_key(&press_key(Keysym::ZENKAKU_HANKAKU));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_skk_zenkaku_hankaku_composing_commits_and_toggles() {
    let mut engine = make_skk_engine();

    // Type "a" to enter composing state with "あ"
    engine.process_key(&press('a'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));

    // Zenkaku/Hankaku should commit composing text and switch to alphabet
    let result = engine.process_key(&press_key(Keysym::ZENKAKU_HANKAKU));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Alphabet);
    assert!(matches!(engine.state(), InputState::Empty));
    assert!(
        result
            .actions
            .iter()
            .any(|a| matches!(a, EngineAction::Commit(t) if t == "あ"))
    );
}

#[test]
fn test_skk_muhenkan_switches_to_alphabet() {
    let mut engine = make_skk_engine();
    assert_eq!(engine.input_mode, InputMode::Hiragana);

    let result = engine.process_key(&press_key(Keysym::MUHENKAN));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Alphabet);
}

#[test]
fn test_skk_henkan_switches_to_hiragana() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Katakana;

    let result = engine.process_key(&press_key(Keysym::HENKAN));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_skk_henkan_from_alphabet() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Alphabet;

    let result = engine.process_key(&press_key(Keysym::HENKAN));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

// --- HideCandidates tests for mode switching ---

#[test]
fn test_skk_l_in_composing_hides_candidates() {
    let mut engine = make_skk_engine();
    engine.process_key(&press('a'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));

    let result = engine.process_key(&press('l'));
    assert!(result.consumed);
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_l_in_conversion_commits_and_hides() {
    let mut engine = make_skk_engine();
    set_conversion_state(&mut engine, "きょう", &["今日", "京", "恭"]);

    let result = engine.process_key(&press('l'));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Alphabet);
    assert!(matches!(engine.state(), InputState::Empty));
    assert!(has_action(&result, |a| matches!(
        a,
        EngineAction::Commit(t) if t == "今日"
    )));
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_q_in_composing_hides_candidates() {
    let mut engine = make_skk_engine();
    engine.process_key(&press('a'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));

    let result = engine.process_key(&press('q'));
    assert!(result.consumed);
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_q_in_conversion_commits_and_hides() {
    let mut engine = make_skk_engine();
    set_conversion_state(&mut engine, "きょう", &["今日", "京", "恭"]);

    let result = engine.process_key(&press('q'));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Empty));
    assert!(has_action(&result, |a| matches!(
        a,
        EngineAction::Commit(t) if t == "今日"
    )));
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_ctrl_q_in_composing_hides_candidates() {
    let mut engine = make_skk_engine();
    engine.process_key(&press('a'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));

    let result = engine.process_key(&press_ctrl(Keysym::KEY_Q));
    assert!(result.consumed);
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_ctrl_q_in_conversion_commits_and_hides() {
    let mut engine = make_skk_engine();
    set_conversion_state(&mut engine, "きょう", &["今日", "京", "恭"]);

    let result = engine.process_key(&press_ctrl(Keysym::KEY_Q));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Empty));
    assert!(has_action(&result, |a| matches!(
        a,
        EngineAction::Commit(t) if t == "今日"
    )));
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
    assert_eq!(engine.input_mode, InputMode::HalfWidthKatakana);
}

#[test]
fn test_skk_ctrl_j_in_conversion_commits_and_hides() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Katakana;
    set_conversion_state(&mut engine, "きょう", &["今日", "京", "恭"]);

    let result = engine.process_key(&press_ctrl(Keysym::KEY_J));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Empty));
    assert!(has_action(&result, |a| matches!(
        a,
        EngineAction::Commit(t) if t == "今日"
    )));
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
    assert_eq!(engine.input_mode, InputMode::Hiragana);
}

#[test]
fn test_skk_zenkaku_hankaku_in_conversion_commits_and_hides() {
    let mut engine = make_skk_engine();
    set_conversion_state(&mut engine, "きょう", &["今日", "京", "恭"]);

    let result = engine.process_key(&press_key(Keysym::ZENKAKU_HANKAKU));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Empty));
    assert!(has_action(&result, |a| matches!(
        a,
        EngineAction::Commit(t) if t == "今日"
    )));
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
    assert_eq!(engine.input_mode, InputMode::Alphabet);
}

// --- HideCandidates tests for commit_composing and Empty state mode switches ---

#[test]
fn test_commit_composing_hides_candidates() {
    let mut engine = make_skk_engine();

    // Type "a" to enter composing state with "あ"
    engine.process_key(&press('a'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));

    // Press Enter to commit — should emit HideCandidates
    let result = engine.process_key(&press_key(Keysym::RETURN));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Empty));
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_l_in_empty_hides_candidates() {
    let mut engine = make_skk_engine();
    assert!(matches!(engine.state(), InputState::Empty));

    let result = engine.process_key(&press('l'));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Alphabet);
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_q_in_empty_hides_candidates() {
    let mut engine = make_skk_engine();
    assert!(matches!(engine.state(), InputState::Empty));

    let result = engine.process_key(&press('q'));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Katakana);
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_ctrl_q_in_empty_hides_candidates() {
    let mut engine = make_skk_engine();
    assert!(matches!(engine.state(), InputState::Empty));

    let result = engine.process_key(&press_ctrl(Keysym::KEY_Q));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::HalfWidthKatakana);
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_ctrl_j_in_empty_from_katakana_hides_candidates() {
    let mut engine = make_skk_engine();
    engine.input_mode = InputMode::Katakana;
    assert!(matches!(engine.state(), InputState::Empty));

    let result = engine.process_key(&press_ctrl(Keysym::KEY_J));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}

#[test]
fn test_skk_ctrl_j_noop_in_hiragana_hides_candidates() {
    let mut engine = make_skk_engine();
    assert_eq!(engine.input_mode, InputMode::Hiragana);
    assert!(matches!(engine.state(), InputState::Empty));

    let result = engine.process_key(&press_ctrl(Keysym::KEY_J));
    assert!(result.consumed);
    assert_eq!(engine.input_mode, InputMode::Hiragana);
    assert!(has_action(&result, |a| matches!(a, EngineAction::HideCandidates)));
}
