/// Korean 2-bul Hangul IME composition engine.
///
/// Implements the standard Hangul syllable block composition algorithm
/// matching CE GWES behaviour for the Korean keyboard layout (0x0412).

/// A Hangul Jamo typed on the Korean 2-bul keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HangulJamo {
    /// Initial consonant (초성) index 0-18.
    Consonant(u8),
    /// Vowel (중성) index 0-20.
    Vowel(u8),
}

/// Korean 2-bul keyboard layout: virtual-key code → Hangul Jamo.
/// Only returns `Some` when the keyboard layout is Korean (0x0412) and IME is open.
pub fn vk_to_hangul_jamo(vk: u32, shift: bool) -> Option<HangulJamo> {
    use HangulJamo::*;
    match vk {
        0x51 => Some(Consonant(if shift { 8 } else { 7 })), // Q: ㅃ/ㅂ
        0x57 => Some(Consonant(if shift { 13 } else { 12 })), // W: ㅉ/ㅈ
        0x45 => Some(Consonant(if shift { 4 } else { 3 })), // E: ㄸ/ㄷ
        0x52 => Some(Consonant(if shift { 1 } else { 0 })), // R: ㄲ/ㄱ
        0x54 => Some(Consonant(if shift { 10 } else { 9 })), // T: ㅆ/ㅅ
        0x59 => Some(Vowel(12)),                            // Y: ㅛ
        0x55 => Some(Vowel(6)),                             // U: ㅕ
        0x49 => Some(Vowel(2)),                             // I: ㅑ
        0x4F => Some(Vowel(if shift { 3 } else { 1 })),     // O: ㅒ/ㅐ
        0x50 => Some(Vowel(if shift { 7 } else { 5 })),     // P: ㅖ/ㅔ
        0x41 => Some(Consonant(6)),                         // A: ㅁ
        0x53 => Some(Consonant(2)),                         // S: ㄴ
        0x44 => Some(Consonant(11)),                        // D: ㅇ
        0x46 => Some(Consonant(5)),                         // F: ㄹ
        0x47 => Some(Consonant(18)),                        // G: ㅎ
        0x48 => Some(Vowel(8)),                             // H: ㅗ
        0x4A => Some(Vowel(4)),                             // J: ㅓ
        0x4B => Some(Vowel(0)),                             // K: ㅏ
        0x4C => Some(Vowel(20)),                            // L: ㅣ
        0x5A => Some(Consonant(15)),                        // Z: ㅋ
        0x58 => Some(Consonant(16)),                        // X: ㅌ
        0x43 => Some(Consonant(14)),                        // C: ㅊ
        0x56 => Some(Consonant(17)),                        // V: ㅍ
        0x42 => Some(Vowel(17)),                            // B: ㅠ
        0x4E => Some(Vowel(13)),                            // N: ㅜ
        0x4D => Some(Vowel(18)),                            // M: ㅡ
        _ => None,
    }
}

/// Initial consonant index (0-18) → compatibility Jamo Unicode code point.
pub const INITIAL_TO_COMPAT: [u16; 19] = [
    0x3131, // 0: ㄱ
    0x3132, // 1: ㄲ
    0x3134, // 2: ㄴ
    0x3137, // 3: ㄷ
    0x3138, // 4: ㄸ
    0x3139, // 5: ㄹ
    0x3141, // 6: ㅁ
    0x3142, // 7: ㅂ
    0x3143, // 8: ㅃ
    0x3145, // 9: ㅅ
    0x3146, // 10: ㅆ
    0x3147, // 11: ㅇ
    0x3148, // 12: ㅈ
    0x3149, // 13: ㅉ
    0x314A, // 14: ㅊ
    0x314B, // 15: ㅋ
    0x314C, // 16: ㅌ
    0x314D, // 17: ㅍ
    0x314E, // 18: ㅎ
];

/// Vowel index (0-20) → compatibility Jamo Unicode code point.
pub const VOWEL_TO_COMPAT: [u16; 21] = [
    0x314F, // 0: ㅏ
    0x3150, // 1: ㅐ
    0x3151, // 2: ㅑ
    0x3152, // 3: ㅒ
    0x3153, // 4: ㅓ
    0x3154, // 5: ㅔ
    0x3155, // 6: ㅕ
    0x3156, // 7: ㅖ
    0x3157, // 8: ㅗ
    0x3158, // 9: ㅘ
    0x3159, // 10: ㅙ
    0x315A, // 11: ㅚ
    0x315B, // 12: ㅛ
    0x315C, // 13: ㅜ
    0x315D, // 14: ㅝ
    0x315E, // 15: ㅞ
    0x315F, // 16: ㅟ
    0x3160, // 17: ㅠ
    0x3161, // 18: ㅡ
    0x3162, // 19: ㅢ
    0x3163, // 20: ㅣ
];

/// Initial consonant index (0-18) → final consonant index (1-27), or 0 if this
/// consonant cannot appear as a final consonant (e.g. tensed ㄸ, ㅃ, ㅉ).
pub const INITIAL_TO_FINAL: [u8; 19] = [
    1,  // 0: ㄱ → final 1
    2,  // 1: ㄲ → final 2
    4,  // 2: ㄴ → final 4
    7,  // 3: ㄷ → final 7
    0,  // 4: ㄸ → no final
    8,  // 5: ㄹ → final 8
    16, // 6: ㅁ → final 16
    17, // 7: ㅂ → final 17
    0,  // 8: ㅃ → no final
    19, // 9: ㅅ → final 19
    20, // 10: ㅆ → final 20
    21, // 11: ㅇ → final 21
    22, // 12: ㅈ → final 22
    0,  // 13: ㅉ → no final
    23, // 14: ㅊ → final 23
    24, // 15: ㅋ → final 24
    25, // 16: ㅌ → final 25
    26, // 17: ㅍ → final 26
    27, // 18: ㅎ → final 27
];

/// Simple final consonant index (1-27) → initial consonant index (0-18) for when
/// a plain final moves to become the initial of the next syllable.
/// Compound finals (3, 5, 6, 9-15, 18) return 0; they are handled via
/// `split_compound_final`.
pub const FINAL_TO_INITIAL: [u8; 28] = [
    0,  // 0: none
    0,  // 1: ㄱ → initial 0
    1,  // 2: ㄲ → initial 1
    0,  // 3: ㄳ (compound — use split_compound_final)
    2,  // 4: ㄴ → initial 2
    0,  // 5: ㄵ (compound)
    0,  // 6: ㄶ (compound)
    3,  // 7: ㄷ → initial 3
    5,  // 8: ㄹ → initial 5
    0,  // 9: ㄺ (compound)
    0,  // 10: ㄻ (compound)
    0,  // 11: ㄼ (compound)
    0,  // 12: ㄽ (compound)
    0,  // 13: ㄾ (compound)
    0,  // 14: ㄿ (compound)
    0,  // 15: ㅀ (compound)
    6,  // 16: ㅁ → initial 6
    7,  // 17: ㅂ → initial 7
    0,  // 18: ㅄ (compound)
    9,  // 19: ㅅ → initial 9
    10, // 20: ㅆ → initial 10
    11, // 21: ㅇ → initial 11
    12, // 22: ㅈ → initial 12
    14, // 23: ㅊ → initial 14
    15, // 24: ㅋ → initial 15
    16, // 25: ㅌ → initial 16
    17, // 26: ㅍ → initial 17
    18, // 27: ㅎ → initial 18
];

/// Attempt to combine two consonants into a compound final.
/// `f1` is the current final index (1-27); `c2` is the initial consonant index
/// (0-18) of the new consonant being pressed.
/// Returns the compound final index if a valid combination exists.
pub fn compound_final(f1: u8, c2: u8) -> Option<u8> {
    match (f1, c2) {
        (1, 9) => Some(3),   // ㄱ + ㅅ → ㄳ
        (4, 12) => Some(5),  // ㄴ + ㅈ → ㄵ
        (4, 18) => Some(6),  // ㄴ + ㅎ → ㄶ
        (8, 0) => Some(9),   // ㄹ + ㄱ → ㄺ
        (8, 6) => Some(10),  // ㄹ + ㅁ → ㄻ
        (8, 7) => Some(11),  // ㄹ + ㅂ → ㄼ
        (8, 9) => Some(12),  // ㄹ + ㅅ → ㄽ
        (8, 16) => Some(13), // ㄹ + ㅌ → ㄾ
        (8, 17) => Some(14), // ㄹ + ㅍ → ㄿ
        (8, 18) => Some(15), // ㄹ + ㅎ → ㅀ
        (17, 9) => Some(18), // ㅂ + ㅅ → ㅄ
        _ => None,
    }
}

/// Split a compound final when a vowel follows.
/// Returns `(remaining_final, split_initial)` where `remaining_final` stays in
/// the current syllable and `split_initial` becomes the initial of the next one.
pub fn split_compound_final(f: u8) -> Option<(u8, u8)> {
    match f {
        3 => Some((1, 9)),   // ㄳ → ㄱ stays, ㅅ becomes initial
        5 => Some((4, 12)),  // ㄵ → ㄴ stays, ㅈ becomes initial
        6 => Some((4, 18)),  // ㄶ → ㄴ stays, ㅎ becomes initial
        9 => Some((8, 0)),   // ㄺ → ㄹ stays, ㄱ becomes initial
        10 => Some((8, 6)),  // ㄻ → ㄹ stays, ㅁ becomes initial
        11 => Some((8, 7)),  // ㄼ → ㄹ stays, ㅂ becomes initial
        12 => Some((8, 9)),  // ㄽ → ㄹ stays, ㅅ becomes initial
        13 => Some((8, 16)), // ㄾ → ㄹ stays, ㅌ becomes initial
        14 => Some((8, 17)), // ㄿ → ㄹ stays, ㅍ becomes initial
        15 => Some((8, 18)), // ㅀ → ㄹ stays, ㅎ becomes initial
        18 => Some((17, 9)), // ㅄ → ㅂ stays, ㅅ becomes initial
        _ => None,
    }
}

/// Try to combine two vowels into a compound vowel.
pub fn compound_vowel(v1: u8, v2: u8) -> Option<u8> {
    match (v1, v2) {
        (8, 0) => Some(9),    // ㅗ + ㅏ → ㅘ
        (8, 1) => Some(10),   // ㅗ + ㅐ → ㅙ
        (8, 20) => Some(11),  // ㅗ + ㅣ → ㅚ
        (13, 4) => Some(14),  // ㅜ + ㅓ → ㅝ
        (13, 5) => Some(15),  // ㅜ + ㅔ → ㅞ
        (13, 20) => Some(16), // ㅜ + ㅣ → ㅟ
        (18, 20) => Some(19), // ㅡ + ㅣ → ㅢ
        _ => None,
    }
}

/// Build a Hangul syllable Unicode code point from initial/vowel/final indices.
pub fn syllable_char(initial: u8, vowel: u8, final_: u8) -> u16 {
    (0xAC00u32 + (initial as u32) * 21 * 28 + (vowel as u32) * 28 + (final_ as u32)) as u16
}

/// Actions produced by the state machine that the caller translates into
/// WM_IME_* messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HangulImeAction {
    /// Send WM_IME_STARTCOMPOSITION.
    StartComposition,
    /// Update HIMC composition string to `ch` and send WM_IME_COMPOSITION(GCS_COMPSTR).
    UpdateComposition(u16),
    /// Send WM_IME_CHAR with the committed character, then WM_IME_ENDCOMPOSITION.
    CommitChar(u16),
    /// Send WM_IME_ENDCOMPOSITION without a committed character (e.g. backspace on lone consonant).
    EndCompositionOnly,
}

/// Hangul composition state stored in the IME context.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HangulComposeState {
    /// Initial consonant index (0-18), or None if no composition in progress.
    pub initial: Option<u8>,
    /// Vowel index (0-20), or None.
    pub vowel: Option<u8>,
    /// First (or only) final consonant index (1-27), 0 = no final yet.
    pub final1: u8,
    /// Initial-consonant index of the second consonant in a compound final, 0 = simple.
    pub final2_initial: u8,
}

impl HangulComposeState {
    pub fn is_active(&self) -> bool {
        self.initial.is_some() || self.vowel.is_some() || self.final1 != 0
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Effective final index for syllable construction (0 = no final).
    pub fn effective_final(&self) -> u8 {
        if self.final1 == 0 || self.final2_initial == 0 {
            self.final1
        } else {
            compound_final(self.final1, self.final2_initial).unwrap_or(self.final1)
        }
    }

    /// The character currently being composed as a single UTF-16 code unit.
    pub fn current_char(&self) -> Option<u16> {
        match (self.initial, self.vowel) {
            (None, None) => None,
            (Some(i), None) => Some(INITIAL_TO_COMPAT[i as usize]),
            (None, Some(v)) => Some(VOWEL_TO_COMPAT[v as usize]),
            (Some(i), Some(v)) => Some(syllable_char(i, v, self.effective_final())),
        }
    }
}

/// Feed one Jamo into the composition engine and return the resulting actions.
///
/// The caller is responsible for:
/// 1. Applying returned `UpdateComposition(ch)` to the HIMC composition string.
/// 2. Posting the corresponding WM_IME_* messages (with GCS_COMPSTR for updates
///    and GCS_RESULTSTR for committed chars).
pub fn process_hangul_jamo(
    state: &mut HangulComposeState,
    jamo: HangulJamo,
) -> Vec<HangulImeAction> {
    match jamo {
        HangulJamo::Vowel(v) => process_vowel(state, v),
        HangulJamo::Consonant(c) => process_consonant(state, c),
    }
}

/// Commit the entire current composition (used when a non-Hangul key interrupts).
/// Returns the actions needed; clears the state.
pub fn commit_composition(state: &mut HangulComposeState) -> Vec<HangulImeAction> {
    let Some(ch) = state.current_char() else {
        return Vec::new();
    };
    state.clear();
    vec![HangulImeAction::CommitChar(ch)]
}

/// Handle a backspace while in composition.
/// Returns the actions needed; updates or clears the state.
pub fn backspace_composition(state: &mut HangulComposeState) -> Vec<HangulImeAction> {
    if state.final2_initial != 0 {
        // Remove the second part of a compound final → revert to simple final.
        state.final2_initial = 0;
        let ch = state.current_char().unwrap();
        return vec![HangulImeAction::UpdateComposition(ch)];
    }
    if state.final1 != 0 {
        // Remove the final consonant → go back to CV syllable.
        state.final1 = 0;
        let ch = state.current_char().unwrap();
        return vec![HangulImeAction::UpdateComposition(ch)];
    }
    if state.vowel.is_some() {
        // Remove vowel → back to initial consonant only (or empty if no initial).
        state.vowel = None;
        if let Some(i) = state.initial {
            let ch = INITIAL_TO_COMPAT[i as usize];
            return vec![HangulImeAction::UpdateComposition(ch)];
        }
        state.clear();
        return vec![HangulImeAction::EndCompositionOnly];
    }
    if state.initial.is_some() {
        // Remove initial → composition ends.
        state.clear();
        return vec![HangulImeAction::EndCompositionOnly];
    }
    Vec::new()
}

// --- private helpers ---

fn process_vowel(state: &mut HangulComposeState, v: u8) -> Vec<HangulImeAction> {
    // Case 1: currently composing a compound vowel on an existing CV syllable (no final yet).
    if state.initial.is_some() && state.vowel.is_some() && state.final1 == 0 {
        let v1 = state.vowel.unwrap();
        if let Some(cv) = compound_vowel(v1, v) {
            state.vowel = Some(cv);
            let ch = state.current_char().unwrap();
            return vec![HangulImeAction::UpdateComposition(ch)];
        }
        // Commit current CV syllable, start new composition with just the vowel.
        let commit = state.current_char().unwrap();
        state.clear();
        state.vowel = Some(v);
        let new_ch = VOWEL_TO_COMPAT[v as usize];
        return vec![
            HangulImeAction::CommitChar(commit),
            HangulImeAction::StartComposition,
            HangulImeAction::UpdateComposition(new_ch),
        ];
    }

    // Case 2: there is a final consonant → it splits off to become the new initial.
    if state.final1 != 0 {
        let (remaining_final, new_initial) = if state.final2_initial != 0 {
            // Compound final: base stays, extension becomes new initial.
            (state.final1, state.final2_initial)
        } else if let Some(split) = split_compound_final(state.final1) {
            // Simple compound final (shouldn't normally happen here, but defensive).
            split
        } else {
            // Plain simple final: move entirely to next syllable initial.
            (0, FINAL_TO_INITIAL[state.final1 as usize])
        };

        // Build the committed syllable (with only the remaining_final).
        let committed_syllable = syllable_char(
            state.initial.unwrap_or(11), // ㅇ as placeholder if no initial
            state.vowel.unwrap_or(0),
            remaining_final,
        );

        state.initial = Some(new_initial);
        state.vowel = Some(v);
        state.final1 = 0;
        state.final2_initial = 0;

        let new_ch = state.current_char().unwrap();
        return vec![
            HangulImeAction::CommitChar(committed_syllable),
            HangulImeAction::StartComposition,
            HangulImeAction::UpdateComposition(new_ch),
        ];
    }

    // Case 3: have an initial consonant only → form CV pair.
    if state.initial.is_some() && state.vowel.is_none() {
        state.vowel = Some(v);
        let ch = state.current_char().unwrap();
        return vec![HangulImeAction::UpdateComposition(ch)];
    }

    // Case 4: standalone vowel only (no initial) — try compound with existing.
    if state.initial.is_none() && state.vowel.is_some() {
        let v1 = state.vowel.unwrap();
        if let Some(cv) = compound_vowel(v1, v) {
            state.vowel = Some(cv);
            let ch = VOWEL_TO_COMPAT[cv as usize];
            return vec![HangulImeAction::UpdateComposition(ch)];
        }
        // Commit existing standalone vowel, start new.
        let commit = VOWEL_TO_COMPAT[v1 as usize];
        state.clear();
        state.vowel = Some(v);
        let new_ch = VOWEL_TO_COMPAT[v as usize];
        return vec![
            HangulImeAction::CommitChar(commit),
            HangulImeAction::StartComposition,
            HangulImeAction::UpdateComposition(new_ch),
        ];
    }

    // Case 5: empty state → start new standalone vowel composition.
    state.clear();
    state.vowel = Some(v);
    let ch = VOWEL_TO_COMPAT[v as usize];
    vec![
        HangulImeAction::StartComposition,
        HangulImeAction::UpdateComposition(ch),
    ]
}

fn process_consonant(state: &mut HangulComposeState, c: u8) -> Vec<HangulImeAction> {
    // Case 1: have a CV pair → try to add c as a final consonant.
    if state.initial.is_some() && state.vowel.is_some() && state.final1 == 0 {
        let f = INITIAL_TO_FINAL[c as usize];
        if f != 0 {
            state.final1 = f;
            let ch = state.current_char().unwrap();
            return vec![HangulImeAction::UpdateComposition(ch)];
        }
        // c cannot be a final consonant → commit CV, start new consonant.
        let commit = state.current_char().unwrap();
        state.clear();
        state.initial = Some(c);
        let new_ch = INITIAL_TO_COMPAT[c as usize];
        return vec![
            HangulImeAction::CommitChar(commit),
            HangulImeAction::StartComposition,
            HangulImeAction::UpdateComposition(new_ch),
        ];
    }

    // Case 2: have a CVC syllable → try to extend to compound final.
    if state.final1 != 0 && state.final2_initial == 0 {
        if compound_final(state.final1, c).is_some() {
            state.final2_initial = c;
            let ch = state.current_char().unwrap();
            return vec![HangulImeAction::UpdateComposition(ch)];
        }
        // Can't extend → commit current CVC syllable, start new consonant.
        let commit = state.current_char().unwrap();
        let was_composing = state.is_active();
        state.clear();
        state.initial = Some(c);
        let new_ch = INITIAL_TO_COMPAT[c as usize];
        let mut actions = if was_composing {
            vec![HangulImeAction::CommitChar(commit)]
        } else {
            Vec::new()
        };
        actions.push(HangulImeAction::StartComposition);
        actions.push(HangulImeAction::UpdateComposition(new_ch));
        return actions;
    }

    // Case 3: have a compound final (final2_initial set) → cannot extend further.
    // Commit the current CVC syllable, start new consonant.
    if state.final2_initial != 0 {
        let commit = state.current_char().unwrap();
        state.clear();
        state.initial = Some(c);
        let new_ch = INITIAL_TO_COMPAT[c as usize];
        return vec![
            HangulImeAction::CommitChar(commit),
            HangulImeAction::StartComposition,
            HangulImeAction::UpdateComposition(new_ch),
        ];
    }

    // Case 4: have only an initial consonant (no vowel yet) → commit it, start new.
    if state.initial.is_some() && state.vowel.is_none() {
        let commit = INITIAL_TO_COMPAT[state.initial.unwrap() as usize];
        state.clear();
        state.initial = Some(c);
        let new_ch = INITIAL_TO_COMPAT[c as usize];
        return vec![
            HangulImeAction::CommitChar(commit),
            HangulImeAction::StartComposition,
            HangulImeAction::UpdateComposition(new_ch),
        ];
    }

    // Case 5: standalone vowel only → commit it, start new consonant.
    if state.initial.is_none() && state.vowel.is_some() {
        let v = state.vowel.unwrap();
        let commit = VOWEL_TO_COMPAT[v as usize];
        state.clear();
        state.initial = Some(c);
        let new_ch = INITIAL_TO_COMPAT[c as usize];
        return vec![
            HangulImeAction::CommitChar(commit),
            HangulImeAction::StartComposition,
            HangulImeAction::UpdateComposition(new_ch),
        ];
    }

    // Case 6: empty state → start a new initial-consonant composition.
    state.clear();
    state.initial = Some(c);
    let ch = INITIAL_TO_COMPAT[c as usize];
    vec![
        HangulImeAction::StartComposition,
        HangulImeAction::UpdateComposition(ch),
    ]
}

// --- tests ---
#[cfg(test)]
mod tests {
    use super::*;

    fn compose(inputs: &[(u32, bool)]) -> Vec<HangulImeAction> {
        let mut state = HangulComposeState::default();
        let mut actions = Vec::new();
        for &(vk, shift) in inputs {
            if let Some(jamo) = vk_to_hangul_jamo(vk, shift) {
                actions.extend(process_hangul_jamo(&mut state, jamo));
            }
        }
        actions
    }

    #[test]
    fn consonant_alone_starts_composition() {
        // VK_R = ㄱ (initial 0)
        let actions = compose(&[(0x52, false)]);
        assert_eq!(
            actions,
            vec![
                HangulImeAction::StartComposition,
                HangulImeAction::UpdateComposition(0x3131), // ㄱ compat
            ]
        );
    }

    #[test]
    fn consonant_vowel_forms_syllable() {
        // VK_R = ㄱ, VK_K = ㅏ → 가 (0xAC00)
        let actions = compose(&[(0x52, false), (0x4B, false)]);
        assert_eq!(
            actions,
            vec![
                HangulImeAction::StartComposition,
                HangulImeAction::UpdateComposition(0x3131), // ㄱ alone
                HangulImeAction::UpdateComposition(0xAC00), // 가
            ]
        );
    }

    #[test]
    fn cv_plus_final_forms_cvc_syllable() {
        // ㄱ + ㅏ + ㄴ → 간 (0xAC04: ㄱ=0, ㅏ=0, ㄴ=4)
        // 0xAC00 + 0*21*28 + 0*28 + 4 = 0xAC04
        let actions = compose(&[(0x52, false), (0x4B, false), (0x53, false)]);
        let last = actions.last().unwrap().clone();
        assert_eq!(
            last,
            HangulImeAction::UpdateComposition(0xAC04), // 간
        );
    }

    #[test]
    fn cvc_plus_vowel_splits_final_to_next_initial() {
        // ㄱ + ㅏ + ㄴ + ㅏ → 가 committed, 나 composing
        // 0xAC00 = 가, 0xB098 = 나 (ㄴ=2, ㅏ=0 → 0xAC00 + 2*21*28 + 0 + 0 = 0xAC00 + 1176 = 0xB098)
        let actions = compose(&[(0x52, false), (0x4B, false), (0x53, false), (0x4B, false)]);
        assert!(actions.contains(&HangulImeAction::CommitChar(0xAC00))); // 가
        assert!(actions.contains(&HangulImeAction::UpdateComposition(0xB098))); // 나
    }

    #[test]
    fn compound_final_forms_correctly() {
        // ㄱ + ㅏ + ㄱ(final) + ㅅ(final2) → 갃 (ㄳ compound final, index 3)
        // 0xAC00 + 0*21*28 + 0*28 + 3 = 0xAC03
        let actions = compose(&[(0x52, false), (0x4B, false), (0x52, false), (0x54, false)]);
        let last = actions.last().unwrap().clone();
        assert_eq!(last, HangulImeAction::UpdateComposition(0xAC03)); // 갃
    }

    #[test]
    fn compound_final_splits_on_vowel() {
        // ㄱ + ㅏ + ㄱ(f1) + ㅅ(f2) + ㅏ → commit 각(f=ㄱ only), compose 사
        // 각 = 0xAC01, 사 = 0xC0AC (ㅅ=9 initial, ㅏ=0 → 0xAC00 + 9*21*28 + 0 + 0 = 0xAC00 + 5292 = 0xBFAC)
        // Wait: 0xAC00 + 9*588 = 0xAC00 + 5292 = 0xC12C? Let me recalculate.
        // ㅅ = initial index 9, ㅏ = vowel index 0
        // syllable = 0xAC00 + 9*21*28 + 0*28 + 0 = 0xAC00 + 5292 = 0xC12C
        let actions = compose(&[
            (0x52, false), // ㄱ
            (0x4B, false), // ㅏ
            (0x52, false), // ㄱ as final
            (0x54, false), // ㅅ extending to ㄳ compound
            (0x4B, false), // ㅏ → split: 각 committed, 사 starts
        ]);
        // 각 = 0xAC01 (ㄱ+ㅏ+ㄱ)
        assert!(actions.contains(&HangulImeAction::CommitChar(0xAC01)));
        // 사 = 0xC0AC? Let me check: ㅅ=initial 9, ㅏ=vowel 0
        // 0xAC00 + 9*21*28 + 0*28 + 0 = 0xAC00 + 5292 = 0xC12C
        // Actually: 9*21 = 189, 189*28 = 5292, 0xAC00 + 5292 = 0xAC00 + 0x14AC = 0xC0AC
        // Let me verify: 0xAC00 = 44032, + 5292 = 49324 = 0xC0AC ✓
        assert!(actions.contains(&HangulImeAction::UpdateComposition(0xC0AC)));
    }

    #[test]
    fn backspace_removes_final_consonant() {
        let mut state = HangulComposeState::default();
        // Build 간 state
        process_hangul_jamo(&mut state, HangulJamo::Consonant(0)); // ㄱ
        process_hangul_jamo(&mut state, HangulJamo::Vowel(0)); // ㅏ
        process_hangul_jamo(&mut state, HangulJamo::Consonant(2)); // ㄴ as final (initial 2 → final 4)
        assert_eq!(state.final1, 4);
        // Backspace should remove the final
        let actions = backspace_composition(&mut state);
        assert_eq!(state.final1, 0);
        assert_eq!(actions, vec![HangulImeAction::UpdateComposition(0xAC00)]); // back to 가
    }

    #[test]
    fn compound_vowel_forms_correctly() {
        // ㅗ(H) + ㅏ(K) → ㅘ (vowel index 9, 0x3158)
        // First with a preceding consonant: ㄱ + ㅗ + ㅏ → 과
        // 과 = 0xAC00 + 0*21*28 + 9*28 + 0 = 0xAC00 + 252 = 0xACFC
        let actions = compose(&[(0x52, false), (0x48, false), (0x4B, false)]);
        let last = actions.last().unwrap().clone();
        assert_eq!(last, HangulImeAction::UpdateComposition(0xACFC)); // 과
    }

    #[test]
    fn commit_composition_returns_current_char() {
        let mut state = HangulComposeState::default();
        process_hangul_jamo(&mut state, HangulJamo::Consonant(0)); // ㄱ
        process_hangul_jamo(&mut state, HangulJamo::Vowel(0)); // ㅏ
        let actions = commit_composition(&mut state);
        assert_eq!(actions, vec![HangulImeAction::CommitChar(0xAC00)]); // 가
        assert!(!state.is_active());
    }
}
