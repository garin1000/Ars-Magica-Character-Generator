//! A Virtue/Flaw classified `uncomputed_rule` states a mechanical rule the
//! engine deliberately does not model, so **the displayed rules text is that
//! rule's only carrier**. If the text does not state the rule, nothing does —
//! the rule has left the application entirely, silently, and the entry still
//! looks complete.
//!
//! That failure mode is not hypothetical. The V/F `summary` convention stops at
//! the first sentence, so whether a rule reached the user at all used to depend
//! on where the author happened to put the full stop: of the 23 core-rulebook
//! entries whose cited passage contains a botch clause, **19 lost the rule
//! outright** and the two that kept it did so by pure accident of sentence
//! order. This test is what stops that recurring.
//!
//! # What it asserts, and what it deliberately does not
//!
//! One assertion, in every shipped locale: an `uncomputed_rule` entry's
//! displayed rules text (`description` if present, else `summary`) must contain
//! at least one **mechanical token** — a signed number, a botch-dice term, or
//! one of the rulebooks' phrase-shaped idioms ([`MECHANICAL_PHRASES`]: a cap, a
//! target number, a formula, a rounding direction, an absolute, a magnitude
//! step). Reclassifying an entry into `uncomputed_rule` therefore costs more than
//! leaving it alone: you must then write the rule into *both* locales. That is
//! the property that stops the class becoming decorative.
//!
//! Its sibling assertion — `uncomputed_rule` entries carry no `effects` — lives
//! in `data_integrity.rs::every_vf_is_classified`, beside the matching
//! `narrative` one, because that is where the whole classification partition is
//! already checked.
//!
//! The third assertion of the set, *"a swept entry whose cited passage carries
//! a mechanical token this entry drops"*, is
//! [`no_swept_entry_drops_an_uncomputed_mechanical_clause`] — D5's first
//! obligation (`docs/vf-audit/decisions.md`) made it class-agnostic: it asks
//! this of `narrative`, `creation_effect` and `in_play_effect` alike, not only
//! `narrative`. It is the expensive half — every red is a hand extraction out
//! of two rulebooks — so it is **scoped by swept block** ([`SWEPT_BLOCKS`])
//! and widens as the sweep proceeds, exactly as `rulebook_citations.rs`'s
//! roots function does. Landing it unscoped would have meant ~90 reds and the
//! only route to green would have
//! been an exemption list — a backlog with a test around it.
//!
//! # Scope
//!
//! The first assertion below covers **every** `uncomputed_rule` entry, in every
//! book, deliberately unscoped — because unlike the third assertion it has no
//! backlog to work through. Writing an entry's rule into both locales is the
//! same act as classifying it `uncomputed_rule`, so a book-by-book scope here
//! would buy nothing and would silently skip whatever it excluded.
//!
//! The mechanical-token vocabulary is a property of *the rulebooks' language*,
//! not of individual entries — the same distinction `rules_source_provenance.rs`
//! draws for `GUARDED_EFFECT_PHRASES`. A phrase list structurally cannot absolve
//! a specific entry of a specific bug, which is why it is an acceptable input
//! here and a per-item exemption array would not be.
//!
//! That principle has now cut both ways twice, and both times the *list* was
//! what was wrong. `BOTCH_TERMS` gained its German verb forms after a shipped
//! description had been reworded to satisfy a noun-only list. Then the whole
//! token set gained [`MECHANICAL_PHRASES`], because a signed number and a botch
//! die cannot see a rule the books state in words — and while they could not,
//! nineteen entries whose passages state caps, target numbers, formulas and
//! absolutes sat classified `narrative`, thirteen of them inside a block already
//! declared swept and clean. Both blocks below were therefore **re-swept** under
//! the widened screen; the earlier dates record when a block was first read, not
//! when it last passed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

/// Rulebook terms for the botch-dice mechanic, lowercased. `botch` covers
/// "botch die"/"botch dice"/"magical botch"; `patzer` covers "Patzerwürfel",
/// "Patzer" and "gepatzert". All are rulebook proper terms, never loose prose.
///
/// The German entries are deliberately *four* where English needs one. English
/// inflects by suffix, so the stem `botch` already reads "botches", "botching"
/// and "botched"; German does not, and a noun-only list missed every passage
/// phrased with the verb — "Das Patzen bei einem dieser Würfe", "Einige
/// Stresswürfe können nicht patzen". That gap was found by a shipped German
/// description being rewritten to satisfy the detector, which is backwards: this
/// vocabulary is a property of **the rulebooks' language**, so when the books and
/// the list disagree it is the list that is wrong. A bare `patz` stem would cover
/// all of them in one entry and is the reason this is not one — it also matches
/// "patzig" (impertinent), which states nothing.
const BOTCH_TERMS: &[&str] = &["botch", "patzer", "patzen", "patzt", "gepatzt"];

fn rules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules")
}

/// The characters that can carry a numeric sign. Shipped rules text is ASCII
/// only — `rules_i18n_ascii_hyphen.rs` enforces that — but the **source**
/// Markdown writes negatives with U+2013 EN DASH, occasionally U+2212, and
/// (S2, F-514) U+2014 EM DASH — `flaw.poor_characteristic`'s "already —3 or
/// lower" is the real passage that found the gap. Listing them here rather
/// than in two detectors keeps one definition of "a sign".
const SIGN_CHARS: &[char] = &['-', '+', '\u{2013}', '\u{2212}', '\u{2014}'];

/// The sign characters that can plausibly open a *subtraction formula* whose
/// right operand is a named quantity rather than a literal digit — F-469's
/// "10 – Size" (`flaw.magical_being_companion`, ArMDE:6390). Deliberately
/// **not** the ASCII hyphen: page and level ranges ("page 103-105") pair a
/// digit with a bare hyphen constantly, and admitting it here would turn
/// every such citation into a false positive. The rulebook's own
/// typographic convention already keeps the two apart — a formula dash is
/// EN DASH or MINUS SIGN, a range hyphen is ASCII — so this list trusts that
/// convention rather than guessing from context.
const FORMULA_DASH_CHARS: &[char] = &['\u{2013}', '\u{2212}'];

/// True when `s` contains a signed number — `+3`, `-9`, or their en-dash /
/// minus-sign spellings as the rulebooks write them. In these books the signed
/// form is a modifier essentially without exception; a bare number ("1 pawn of
/// vis each season") is where the noise lives, so requiring the sign is what
/// keeps *this* token sharp.
///
/// It used to keep "an Ease Factor of 9" out as well, and that exclusion was
/// asserted here — wrongly. A target number is a rule, not noise; what it is not
/// is a *signed modifier*. [`has_mechanical_phrase`] is where it belongs, and
/// splitting the two is what lets this one stay narrow without the vocabulary as
/// a whole staying blind.
///
/// # Two shapes a plain "sign immediately left of a digit" check missed (S2)
///
/// - **Sign, space, digit** — F-464: `flaw.no_hands` (ArMDE:6498) writes
///   "take a – 5 penalty", with a literal space between the EN DASH and the
///   digit. Verified byte-by-byte with `od -c`.
/// - **Digit, (space,) dash, non-digit** — F-469: `flaw.magical_being_companion`
///   (ArMDE:6390) writes "Magic Might score of 10 – Size" — a subtraction
///   formula whose right operand is a name, so no digit ever follows the
///   dash. Restricted to [`FORMULA_DASH_CHARS`] for the reason its doc
///   comment gives.
fn has_signed_number(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();

    let skip_spaces = |mut idx: usize| {
        while chars.get(idx) == Some(&' ') {
            idx += 1;
        }
        idx
    };

    for (i, &c) in chars.iter().enumerate() {
        if SIGN_CHARS.contains(&c)
            && chars
                .get(skip_spaces(i + 1))
                .is_some_and(char::is_ascii_digit)
        {
            return true;
        }
        if c.is_ascii_digit()
            && chars
                .get(skip_spaces(i + 1))
                .is_some_and(|c| FORMULA_DASH_CHARS.contains(c))
        {
            return true;
        }
    }
    false
}

/// True when `s` names the botch-dice mechanic in any shipped locale.
fn has_botch_term(s: &str) -> bool {
    let lower = s.to_lowercase();
    BOTCH_TERMS.iter().any(|term| lower.contains(term))
}

/// The rulebooks' **phrase-shaped** mechanical idioms, lowercased: the rules
/// they state in words rather than in a signed number or a die name.
///
/// The books are full of these — a cap ("up to 12 human-sized animals"), a
/// target number ("against an Ease Factor of 15"), a formula ("equal to a tenth
/// of his Creo Vim Lab Total"), a rounding direction, an absolute ("cannot die
/// as a result of wounds or old age"), a step counted in magnitudes. Each is as
/// mechanical as a `+3`, and while this screen did not exist every one of them
/// read as pure flavour. That is not a hypothetical either: the Flaws block was
/// declared swept and clean on 2026-09-15 with thirteen of them sitting inside
/// it, because the two tokens that existed could not see a rule stated in words.
///
/// # This screen is deliberately noisier than the other two, and that is the
/// # safe direction
///
/// "equal to" and "at least" occur in ordinary prose, so this list over-reports
/// by design. Over-reporting costs a human one reading, written down as a
/// [`NO_RULE_DESPITE_TOKEN`] row that says what the passage actually says.
/// Under-reporting costs a rule, silently, with the entry still looking
/// complete — which is the exact failure this module exists to prevent. Given
/// the choice, be noisy. The negative assertions in
/// [`the_mechanical_token_detector_reads_real_clauses_and_ignores_near_misses`]
/// are what keep "noisy" from becoming "meaningless": the noise is bounded by
/// tests rather than by hope.
///
/// # One list, both locales
///
/// German phrases sit beside their English equivalents rather than in a list of
/// their own. Two lists drift, and the narrower one then bends the rulebook text
/// to fit it — which has already happened once here, when a shipped German
/// description was rewritten from "Das Patzen" to "Ein Patzer" to satisfy a
/// noun-only `BOTCH_TERMS`. The vocabulary is a property of **the rulebooks'
/// language**, so when the books and the list disagree, the list is what
/// changes.
///
/// Two consequences of German being in the same list:
///
/// - **`multiplizier` is a stem where its English twins are not.** German
///   inflects the ending ("multipliziere"/"multipliziert"), so one stem serves;
///   English splits the stem itself ("multiply"/"multiplied"), so both forms are
///   listed. The tempting single stem `multipl` covers neither pair honestly —
///   it also swallows "may be taken multiple times", a repeatability rule the
///   engine *does* compute (`max_per_target`), so it would flag most of the
///   catalogue and teach nothing.
/// - **German negation is discontinuous**, so a literal like `nicht sterben`
///   cannot express "cannot <verb>" in general: the books' own "kannst aber
///   nicht an Wunden oder Alter sterben" puts a whole clause between the two
///   halves, invisible to a fixed two-word phrase. The list below carries only
///   the contiguous spelling the books also write plainly. D19
///   (`docs/vf-audit/decisions.md`) is why each entry compiles as a regex
///   ([`MECHANICAL_PHRASE_PATTERNS`]) rather than staying a literal substring
///   — a bounded-gap pattern (`kann\b.{0,N}\bnicht`) *can* express the
///   discontinuous form, unlike a substring — but adding that pattern is a
///   new family, not a conversion, so it is slice S2's job, not this one's.
///   Converting this list to regex without widening what it matches is what
///   [`regex_screen_never_loses_an_s1_recorded_flag`] proves.
///
/// The rounding forms are spelled out rather than stemmed because the obvious
/// stem, `round`, is also a unit of combat time; `rounded up`/`round up` and
/// their two downward twins are all in the English book, as are `aufgerundet`
/// and `abgerundet` in the German one.
const MECHANICAL_PHRASES: &[&str] = &[
    // Caps, floors and thresholds.
    "up to",
    "bis zu",
    "at least",
    "mindestens",
    "or greater",
    "oder höher",
    "or more",
    "oder mehr",
    "may not be greater than",
    // F-537 (S2): this needle was "darf nicht größer sein als", assuming
    // English word order. German puts the verb clause-finally instead —
    // `flaw.uninspirational` (ArMDE:6921) writes "darf nicht größer als 0
    // sein" — so the literal matched nothing the book writes. Fixed to the
    // word order the book actually uses; verified against ArMDE:6921 and one
    // other hit.
    "darf nicht größer als",
    "no more than",
    "nicht mehr als",
    // Target numbers.
    "ease factor",
    "schwierigkeitsgrad",
    // Formulas.
    "equal to",
    "entspricht",
    "multiply",
    "multiplied",
    "multiplizier",
    // Rounding, which changes the answer.
    "round up",
    "rounded up",
    "round down",
    "rounded down",
    "aufgerundet",
    "abgerundet",
    // Dice named by kind.
    "simple die",
    "einfachen würfel",
    "stress die",
    "stresswürfel",
    // Absolutes.
    "cannot die",
    "nicht sterben",
    // A step counted in magnitudes.
    "one magnitude",
    "eine magnitude",
];

/// [`MECHANICAL_PHRASES`], compiled once as case-insensitive regexes anchored
/// to a left word boundary — D19 (`docs/vf-audit/decisions.md`): a contiguous
/// substring screen structurally cannot express German discontinuous
/// negation, so the screen matches by regex instead. `\b` reproduces the
/// substring matcher's left-boundary check exactly (no boundary on the
/// right, for the same reason the substring version had none — see
/// [`has_mechanical_phrase`]'s doc comment). `LazyLock` compiles the ~30
/// patterns exactly once rather than once per catalogue entry.
static MECHANICAL_PHRASE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    MECHANICAL_PHRASES
        .iter()
        .map(|phrase| {
            let pattern = format!(r"(?i)\b{}", regex::escape(phrase));
            Regex::new(&pattern)
                .unwrap_or_else(|e| panic!("phrase {phrase:?} compiles as a regex: {e}"))
        })
        .collect()
});

/// True when `s` matches any of [`MECHANICAL_PHRASE_PATTERNS`] — one of
/// [`MECHANICAL_PHRASES`], anchored at a word boundary on the left.
///
/// The boundary is load-bearing, not tidiness. A bare substring search reads
/// "or more" out of "f|or more| details" and so flags every entry whose only
/// sin is pointing at a supplement — `virtue.factor` and `virtue.fidai` both
/// arrived in the sweep that way, which is why the regex conversion is proved
/// inert against exactly those two entries (see
/// [`regex_screen_never_loses_an_s1_recorded_flag`]). A phrase crossing
/// a word boundary is a defect in the screen, not an idiom to argue down in a
/// `NO_RULE_DESPITE_TOKEN` row.
///
/// There is deliberately **no** boundary on the right, so a phrase still
/// matches through a suffix — which is how the `multipli` stem earns its
/// keep.
fn has_mechanical_phrase(s: &str) -> bool {
    MECHANICAL_PHRASE_PATTERNS.iter().any(|re| re.is_match(s))
        || S2_IDIOM_PATTERNS.iter().any(|re| re.is_match(s))
}

/// The guard's detector: does this rules text actually state a mechanical rule?
fn states_a_mechanical_rule(text: &str) -> bool {
    has_signed_number(text) || has_botch_term(text) || has_mechanical_phrase(text)
}

/// The rulebook language an [`S2Idiom`] must be verified against — D19's
/// obligation 3, tightened by F-537: a German pattern is checked against the
/// German source, not against a plausible rendering of the English, and vice
/// versa.
#[derive(Clone, Copy)]
enum Language {
    En,
    De,
}

/// One S2 (`docs/vf-audit/corrections.md` § 3.1, plus the capability family
/// § 2.1b/D8 requires) addition to the mechanical screen.
///
/// `pattern` is complete regex source — already valid, left-`\b`-anchored per
/// [`has_mechanical_phrase`]'s no-right-boundary convention — compiled
/// case-insensitively exactly as written. None of the idioms below contain a
/// regex metacharacter as a literal character, which is why none needs
/// `regex::escape`; an addition that does must escape its literal part by
/// hand before splicing it in. `family` names the § 3.1 row (`"capability"`
/// for D8's, which the table does not itself carry) purely for
/// documentation and failure messages.
struct S2Idiom {
    pattern: &'static str,
    language: Language,
    family: &'static str,
}

/// D19's bounded-gap idiom for German's discontinuous modal negation
/// ("kann … nicht anwenden", "darf … nicht nehmen"): a modal verb, then up to
/// 40 non-period characters (never crossing a full stop, so the gap cannot
/// span a sentence boundary), then "nicht". 40 is not a guess — B17's and
/// this slice's own sample of real gaps (measured from the shipped German
/// text) top out at 23 characters ("darfst die Tugend Wohlhabend nicht"); 40
/// leaves headroom without approaching a typical sentence's length. Matches
/// with a zero-length gap too (`{0,40}?` allows zero repetitions), which is
/// how it also covers the contiguous spellings ("darfst nicht", "kannst
/// nicht") without a second pattern.
const DE_MODAL_NICHT: &str = r"\b(?:kann|kannst|können|darf|darfst|dürfen)\b[^.]{0,40}?\bnicht";

/// Every S2 addition. Grouped and commented by § 3.1's family numbering.
/// Every literal and every bounded-gap idiom here was verified against a
/// real hit in its own language's source file before being added (D19
/// obligation 3) — see [`every_s2_idiom_has_a_real_hit_in_its_own_language`],
/// which checks the same claim mechanically on every future change.
const S2_IDIOMS: &[S2Idiom] = &[
    // --- Capability (D8 / § 2.1b) --------------------------------------
    // D8 reclassifies 48 `supernatural` + `narrative` entries that state a
    // roll-free capability with no signed number and no botch term, so they
    // depend entirely on a family like this one. Derived from the 48
    // themselves (`virtue.amorphous`'s "is able to", `virtue.see_in_darkness`'s
    // "You can see" — too bare to add safely — `virtue.leather_ripper`'s "the
    // supernatural ability to", `virtue.kassalan_exorcism`'s "capable of").
    // Bare "can"/"kann" are deliberately excluded: both are among the most
    // common words in either language's ordinary prose, and would flag most
    // of the catalogue.
    S2Idiom {
        pattern: r"\bis able to",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        pattern: r"\bare able to",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        pattern: r"\bcapable of",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        pattern: r"\bthe ability to",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        pattern: r"\bin der lage",
        language: Language::De,
        family: "capability",
    },
    S2Idiom {
        pattern: r"\bfähigkeit",
        language: Language::De,
        family: "capability",
    },
    // --- Family 1 (Prohibition) + family 3 (Absolutes beyond "cannot die")
    // Consolidated: German negation is discontinuous (D19), so the general
    // fix is one bounded-gap pattern binding a modal verb to a later
    // "nicht", which subsumes every contiguous DE spelling the table lists
    // ("darfst nicht", "kann nicht genommen werden", "kannst nicht", "kann …
    // nicht anwenden") as the zero-gap and small-gap cases of the same
    // idiom. English "cannot" is one word, so stemming it bare subsumes
    // "cannot take"/"cannot walk"/"cannot cast"/"cannot permanently
    // destroy"/"cannot gain"/"cannot die" the same way.
    S2Idiom {
        pattern: r"\bcannot",
        language: Language::En,
        family: "prohibition/absolutes",
    },
    S2Idiom {
        pattern: r"\bcan't",
        language: Language::En,
        family: "prohibition/absolutes",
    },
    S2Idiom {
        pattern: r"\bmay not take",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        pattern: r"\bmay not have",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        pattern: r"\bmay only take this",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        pattern: r"\bis impossible",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        pattern: r"\bunable to learn",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        pattern: DE_MODAL_NICHT,
        language: Language::De,
        family: "prohibition/absolutes",
    },
    S2Idiom {
        pattern: r"\bunmöglich",
        language: Language::De,
        family: "prohibition",
    },
    S2Idiom {
        pattern: r"\bnicht gehen",
        language: Language::De,
        family: "prohibition",
    },
    S2Idiom {
        pattern: r"\bkönnen nichts",
        language: Language::De,
        family: "prohibition",
    },
    S2Idiom {
        pattern: r"\bkannst keine",
        language: Language::De,
        family: "prohibition",
    },
    // --- Family 2 (Permission) ------------------------------------------
    // The DE modal+verb pairs are one bounded-gap pattern for the same
    // reason as the prohibition family: "darfst … wählen" and "darf …
    // erwerben" are the same idiom with a different bound verb, not
    // different idioms.
    S2Idiom {
        pattern: r"\bmay take",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\bmay learn",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\bmay purchase",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\bmay begin with",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\bcan purchase",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\bis allowed to have",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\ballows the character to purchase",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\beven if normally",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\bat character generation",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\bat character creation",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\b(?:darf|darfst|dürfen)\b[^.]{0,40}?\b(?:wählen|erwerben|erlernen|haben)",
        language: Language::De,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\berlaubt\b[^.]{0,60}?\bzu (?:kaufen|erwerben|erlernen|wählen)",
        language: Language::De,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\bselbst wenn du normalerweise",
        language: Language::De,
        family: "permission",
    },
    S2Idiom {
        pattern: r"\bbei der charaktererschaffung",
        language: Language::De,
        family: "permission",
    },
    // --- Family 4 (Eligibility) -------------------------------------------
    S2Idiom {
        pattern: r"\bmust be\b[^.]{0,40}?\bto take this",
        language: Language::En,
        family: "eligibility",
    },
    S2Idiom {
        pattern: r"\bmay only be taken by",
        language: Language::En,
        family: "eligibility",
    },
    S2Idiom {
        pattern: r"\bonly characters with",
        language: Language::En,
        family: "eligibility",
    },
    // The recurring tail of the German "you must be X to take this
    // Virtue/Flaw" idiom — verified against three real passages
    // (ArMDE:6148, :6250, :6885 and their DE mirrors) that phrase the lead-in
    // three different ways ("musst … sein", "muss … besitzen", "muss …
    // sein") but always close "um dies-<en|e> <Fehler|Tugend> zu
    // <wählen|nehmen>". Anchoring on the stable tail rather than the varying
    // lead-in is what makes one pattern cover all three.
    S2Idiom {
        pattern: r"\bum dies\w*\b[^.]{0,20}?\bzu (?:wählen|nehmen)",
        language: Language::De,
        family: "eligibility",
    },
    S2Idiom {
        pattern: r"\bdarf nur von\b[^.]{0,40}?\bgenommen werden",
        language: Language::De,
        family: "eligibility",
    },
    // --- Family 5 (Incompatibility / exclusion) --------------------------
    S2Idiom {
        pattern: r"\bis not compatible with",
        language: Language::En,
        family: "incompatibility",
    },
    S2Idiom {
        pattern: r"\bis incompatible with",
        language: Language::En,
        family: "incompatibility",
    },
    S2Idiom {
        pattern: r"\bmay not also take",
        language: Language::En,
        family: "incompatibility",
    },
    S2Idiom {
        pattern: r"\bmay not be combined with",
        language: Language::En,
        family: "incompatibility",
    },
    S2Idiom {
        pattern: r"\bmay not have both",
        language: Language::En,
        family: "incompatibility",
    },
    S2Idiom {
        pattern: r"\bist nicht\b[^.]{0,40}?\bvereinbar",
        language: Language::De,
        family: "incompatibility",
    },
    S2Idiom {
        pattern: r"\bist unvereinbar mit",
        language: Language::De,
        family: "incompatibility",
    },
    S2Idiom {
        pattern: r"\bdarf nicht zusätzlich",
        language: Language::De,
        family: "incompatibility",
    },
    // --- Family 6 (Item transfer / composition) --------------------------
    // F-403/F-406: exactly 4 hits in the whole English core book, 3 already
    // non-`narrative`.
    S2Idiom {
        pattern: r"\bincludes the effects of",
        language: Language::En,
        family: "item transfer",
    },
    S2Idiom {
        pattern: r"\bschließt die auswirkungen von\b[^.]{0,50}?\bein",
        language: Language::De,
        family: "item transfer",
    },
    // --- Family 6b (Creation-rules substitution) ---------------------------
    // X2a (2026-09-29): `virtue.guest_of_house_criamon` (ArMDE:4037-4040) states
    // a real creation-time permission — being created under a different House's
    // rules while remaining politically Criamon — with no established idiom to
    // catch it. Verified against that one real hit.
    S2Idiom {
        pattern: r"\bmay be created using the rules",
        language: Language::En,
        family: "creation-rules substitution",
    },
    // --- Family 7 (Multiplier stem) ---------------------------------------
    S2Idiom {
        pattern: r"\bmultiplier",
        language: Language::En,
        family: "multiplier stem",
    },
    S2Idiom {
        pattern: r"\bmultiplikator",
        language: Language::De,
        family: "multiplier stem",
    },
    // --- Family 8 (Multiplier in words) ------------------------------------
    S2Idiom {
        pattern: r"\bdouble",
        language: Language::En,
        family: "multiplier in words",
    },
    S2Idiom {
        pattern: r"\btwice",
        language: Language::En,
        family: "multiplier in words",
    },
    S2Idiom {
        pattern: r"\bhalve",
        language: Language::En,
        family: "multiplier in words",
    },
    S2Idiom {
        pattern: r"\bdoppelt",
        language: Language::De,
        family: "multiplier in words",
    },
    S2Idiom {
        pattern: r"\bhalbiert",
        language: Language::De,
        family: "multiplier in words",
    },
    // --- Family 9 ("more" without the "or") ---------------------------------
    // F-383's actual miss is "possibly more", not a gap-widened "or more";
    // adding it as its own literal is simpler than a bounded-gap pattern and
    // does not risk relaxing "or more" itself into something noisier.
    S2Idiom {
        pattern: r"\bpossibly more",
        language: Language::En,
        family: "\"more\" without \"or\"",
    },
    // --- Family 10 (Bare "no more") -----------------------------------------
    S2Idiom {
        pattern: r"\bno more",
        language: Language::En,
        family: "bare \"no more\"",
    },
    S2Idiom {
        pattern: r"\bnicht mehr",
        language: Language::De,
        family: "bare \"no more\"",
    },
    // --- Family 11 (Floor) ---------------------------------------------------
    S2Idiom {
        pattern: r"\bminimum",
        language: Language::En,
        family: "floor",
    },
    S2Idiom {
        pattern: r"\bmindest",
        language: Language::De,
        family: "floor",
    },
    // --- Family 12 (Magnitude/level stem) -------------------------------------
    S2Idiom {
        pattern: r"\bmagnitudes",
        language: Language::En,
        family: "magnitude/level stem",
    },
    S2Idiom {
        pattern: r"\bone level",
        language: Language::En,
        family: "magnitude/level stem",
    },
    S2Idiom {
        pattern: r"\bby more than one level",
        language: Language::En,
        family: "magnitude/level stem",
    },
    S2Idiom {
        pattern: r"\bmagnituden",
        language: Language::De,
        family: "magnitude/level stem",
    },
    S2Idiom {
        pattern: r"\bum mehr als eine stufe",
        language: Language::De,
        family: "magnitude/level stem",
    },
    // --- Family 13 (Bare imperative modifier) ---------------------------------
    // F-489, "the sharpest miss in the batch": the rule is the entry's whole
    // first sentence. "add" is right-bounded (unlike this module's usual
    // convention) because without it the bare stem also matches "address",
    // "additional" and "administrator", none of which state a rule.
    S2Idiom {
        pattern: r"\bsubtract",
        language: Language::En,
        family: "bare imperative modifier",
    },
    S2Idiom {
        pattern: r"\badd\b",
        language: Language::En,
        family: "bare imperative modifier",
    },
    S2Idiom {
        pattern: r"\bziehe",
        language: Language::De,
        family: "bare imperative modifier",
    },
    S2Idiom {
        pattern: r"\baddiere",
        language: Language::De,
        family: "bare imperative modifier",
    },
    // --- Family 14 (Named rulebook terms) -------------------------------------
    S2Idiom {
        pattern: r"\badvancement total",
        language: Language::En,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\bwealth multiplier",
        language: Language::En,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\breputation\b[^.]{0,20}?\b(?:at|of) level",
        language: Language::En,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\bexperience points\b[^.]{0,40}?\bmust be spent on",
        language: Language::En,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\ban additional personality trait of",
        language: Language::En,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\bspend a round",
        language: Language::En,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\bgame mechanical effects",
        language: Language::En,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\btwo dice instead of",
        language: Language::En,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\bfortschrittssumme",
        language: Language::De,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\breputation der stufe",
        language: Language::De,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\berfahrungspunkte\b[^.]{0,40}?\bausgeben",
        language: Language::De,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\beine runde lang",
        language: Language::De,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\bspielmechanische auswirkungen",
        language: Language::De,
        family: "named rulebook term",
    },
    S2Idiom {
        pattern: r"\bzwei würfel statt",
        language: Language::De,
        family: "named rulebook term",
    },
    // --- Family 15 (X2a fix-round additions, 2026-09-29) --------------------
    // Restoring 18 X2a descriptions to their verbatim cited text (previously
    // reworded to satisfy this screen, which is exactly backwards) re-exposed
    // genuine rules this screen did not yet recognize under their real
    // phrasing. Each pattern was verified against the one real passage that
    // motivated it.
    S2Idiom {
        // virtue.common_sense (ArMDE:3597-3600): "common sense (the
        // storyguide) alerts you to the error" — D50's own worked example.
        pattern: r"\balerts you to the error",
        language: Language::En,
        family: "storyguide arbitration",
    },
    S2Idiom {
        // virtue.common_sense (ArMDE:3597-3600, DE): "macht dich der gesunde
        // Menschenverstand (der Spielleiter) auf den Fehler aufmerksam".
        pattern: r"\bauf den fehler aufmerksam",
        language: Language::De,
        family: "storyguide arbitration",
    },
    S2Idiom {
        // virtue.devil_child (ArMDE:3671-3674): "can only be taken for a
        // Mythic Companion" — an eligibility idiom the existing "may only be
        // taken by" phrasing does not cover.
        pattern: r"\bcan only be taken for",
        language: Language::En,
        family: "eligibility",
    },
    S2Idiom {
        // virtue.devil_child (ArMDE:3671-3674, DE): "kann nur für einen
        // Mythischen Gefährten genommen werden".
        pattern: r"\bkann nur für\b[^.]{0,40}?\bgenommen werden",
        language: Language::De,
        family: "eligibility",
    },
    S2Idiom {
        // virtue.feather_messenger (ArMDE:3869-3872): "a character who can
        // take the form of a bird".
        pattern: r"\btake the form of",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        // virtue.feather_messenger (ArMDE:3869-3872, DE): "der die Form eines
        // Vogels annehmen kann" — German's verb-final clause puts the modal
        // after the object, so the capability idiom is anchored on the verb
        // pair itself rather than "in der Lage"/"Fähigkeit".
        pattern: r"\bannehmen kann\b",
        language: Language::De,
        family: "capability",
    },
    S2Idiom {
        // virtue.gender_shift (ArMDE:3951-3954): "Pregnant characters may not
        // use this ability."
        pattern: r"\bmay not use\b",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        // virtue.gentle_gift (ArMDE:3955-3958): "You do not suffer the usual
        // penalties".
        pattern: r"\bdo not suffer\b",
        language: Language::En,
        family: "prohibition/absolutes",
    },
    S2Idiom {
        // virtue.gentle_gift (ArMDE:3955-3958, DE): "Du leidest nicht unter
        // den üblichen Abzügen" — negation on a plain verb, not a modal, so
        // DE_MODAL_NICHT (which requires a modal verb before the gap) does
        // not reach it.
        pattern: r"\bleidest nicht\b",
        language: Language::De,
        family: "prohibition/absolutes",
    },
    S2Idiom {
        // virtue.gorgiastic (ArMDE:3979-3982, DE): "kann ohne die Hilfe von
        // Criamon-Magi oder einen magischen Durchbruch 4 nicht überschreiten"
        // — the same modal-then-"nicht" shape DE_MODAL_NICHT already
        // expresses, but the real gap here (66 characters) exceeds its
        // 40-character bound. Rather than widen that bound for every use
        // (which would relax the "prohibition/absolutes" family's noise floor
        // catalogue-wide), this is a second, narrowly verb-scoped pattern:
        // same modal-verb set, bound raised only for the specific tail
        // "nicht überschreiten" (D19's own precedent for a distinctly-bounded
        // second pattern rather than a blanket widening).
        pattern: r"\b(?:kann|kannst|können|darf|darfst|dürfen)\b[^.]{0,80}?\bnicht überschreiten",
        language: Language::De,
        family: "ceiling (exceed)",
    },
    S2Idiom {
        // virtue.greater_immunity (ArMDE:4009-4016, DE): "Du darfst keine
        // Immunität gegen Altern nehmen" — German can negate a modal
        // permission with "keine" rather than "nicht"; DE_MODAL_NICHT's
        // literal "nicht" cannot see this shape at any gap width. A new,
        // narrow family for the modal+"keine" idiom, distinct from — not a
        // widening of — DE_MODAL_NICHT.
        pattern: r"\b(?:darf|darfst|dürfen|kann|kannst|können)\b[^.]{0,40}?\bkeine\b",
        language: Language::De,
        family: "prohibition via keine",
    },
    S2Idiom {
        // virtue.greater_purifying_touch (ArMDE:4027-4030): "You can only
        // choose a disease, not other types of injury or misfortune."
        pattern: r"\bcan only\b",
        language: Language::En,
        family: "capability/restriction limiter",
    },
    S2Idiom {
        // virtue.greater_purifying_touch (ArMDE:4027-4030, DE): "Du kannst
        // nur eine Krankheit wählen" — the existing DE permission idiom binds
        // only darf/darfst/dürfen, not kannst.
        pattern: r"\bkannst nur\b[^.]{0,40}?\bwählen\b",
        language: Language::De,
        family: "permission",
    },
    S2Idiom {
        // virtue.guest_of_house_criamon (ArMDE:4037-4040, DE): "können jedoch
        // nach den Regeln für jedes andere Haus erschaffen werden" — the DE
        // mirror of the existing EN "may be created using the rules" idiom
        // (family 6b), which does not itself match the German word order.
        pattern: r"\bnach den regeln\b[^.]{0,40}?\berschaffen werden",
        language: Language::De,
        family: "creation-rules substitution",
    },
    S2Idiom {
        // virtue.guild_apprentice (ArMDE:4041-4044): "The character is not
        // able to benefit from either the Poor Flaw or the Wealthy Virtue".
        pattern: r"\bis not able to\b",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        // virtue.guild_apprentice (ArMDE:4041-4044, DE): "Der Charakter kann
        // weder vom Fehler Arm noch von der Tugend Wohlhabend profitieren" —
        // German's "neither...nor" negation, distinct in shape from both
        // DE_MODAL_NICHT and the "keine" family above.
        pattern: r"\bweder\b[^.]{0,40}?\bnoch\b",
        language: Language::De,
        family: "prohibition",
    },
    S2Idiom {
        // virtue.harnessed_magic (ArMDE:4053-4058, DE): "Du kannst jeden
        // deiner Zauber einfach durch Konzentration aufheben" — a positive
        // modal capability the existing noun-based "in der Lage"/"Fähigkeit"
        // idioms do not reach.
        pattern: r"\bkannst\b[^.]{0,40}?\baufheben\b",
        language: Language::De,
        family: "capability",
    },
    S2Idiom {
        // virtue.emir (ArMDE:3745): "This is the same as the Knight Virtue"
        // — a cross-reference the player must act on to get another entry's
        // (Knight's) benefits.
        pattern: r"\bis the same as the\b[^.]{0,40}?\bvirtue\b",
        language: Language::En,
        family: "cross-reference",
    },
];

/// [`S2_IDIOMS`], compiled once.
static S2_IDIOM_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    S2_IDIOMS
        .iter()
        .map(|idiom| {
            let source = format!("(?i){}", idiom.pattern);
            Regex::new(&source)
                .unwrap_or_else(|e| panic!("S2 idiom {:?} compiles as a regex: {e}", idiom.pattern))
        })
        .collect()
});

/// The V/F catalogue, parsed once.
fn catalogue() -> Vec<Value> {
    let path = rules_dir().join("core/virtues_flaws.json");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{} is valid JSON: {e}", path.display()))
}

/// Every entry in the catalogue carrying `classification`, as `id`.
fn ids_classified(classification: &str) -> Vec<String> {
    catalogue()
        .iter()
        .filter(|item| item["classification"] == classification)
        .map(|item| {
            item["id"]
                .as_str()
                .expect("every catalogue entry has a string id")
                .to_string()
        })
        .collect()
}

/// `lang -> id -> displayed rules text`, where the displayed text is
/// `description` if present and `summary` otherwise — exactly the precedence
/// `VirtueFlawTab.svelte`'s tooltip applies.
fn displayed_rules_text_by_language() -> BTreeMap<String, BTreeMap<String, String>> {
    let i18n_dir = rules_dir().join("i18n");
    let mut by_language = BTreeMap::new();

    for lang_entry in
        fs::read_dir(&i18n_dir).unwrap_or_else(|e| panic!("rules/i18n is readable: {e}"))
    {
        let lang_path = lang_entry.unwrap().path();
        if !lang_path.is_dir() {
            continue;
        }
        let lang = lang_path
            .file_name()
            .expect("a language directory has a name")
            .to_string_lossy()
            .to_string();

        let path = lang_path.join("virtues_flaws.json");
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
        let entries: BTreeMap<String, Value> = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{} is valid JSON: {e}", path.display()));

        let displayed = entries
            .into_iter()
            .filter_map(|(id, entry)| {
                let text = entry["description"]
                    .as_str()
                    .or_else(|| entry["summary"].as_str())?;
                Some((id, text.to_string()))
            })
            .collect();
        by_language.insert(lang, displayed);
    }

    assert!(
        !by_language.is_empty(),
        "expected at least one rules/i18n/<lang>/virtues_flaws.json to exist"
    );
    by_language
}

#[test]
fn every_uncomputed_rule_entry_states_its_rule_in_every_locale() {
    let ids = ids_classified("uncomputed_rule");
    assert!(
        !ids.is_empty(),
        "expected the catalogue to contain uncomputed_rule entries; if this \
         fires, the classification has drifted and this guard is checking nothing"
    );

    let by_language = displayed_rules_text_by_language();
    let mut offenders = Vec::new();

    for (lang, displayed) in &by_language {
        for id in &ids {
            match displayed.get(id) {
                None => offenders.push(format!("{lang}/{id}: no rules text at all")),
                Some(text) => {
                    if !states_a_mechanical_rule(text) {
                        offenders.push(format!("{lang}/{id}: {text:?}"));
                    }
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "an `uncomputed_rule` Virtue/Flaw states a rule the engine does not \
         compute, so its displayed rules text is that rule's ONLY carrier — but \
         these carry no mechanical token (no signed number, no botch-dice term, \
         and none of the rulebooks' phrase-shaped idioms). Either the clause was \
         dropped when the text was written, or the entry is not really \
         `uncomputed_rule`:\n{}",
        offenders.join("\n")
    );
}

/// The blocks of source Markdown whose entries have been swept — every cited
/// passage read, every mechanical clause either written into both locales (and
/// the entry reclassified) or confirmed absent.
///
/// Incremental **by design**, mirroring `rulebook_citations.rs`'s roots
/// function: the sweep is a hand extraction out of two rulebooks, so landing
/// [`no_swept_entry_drops_an_uncomputed_mechanical_clause`] over the whole
/// catalogue at once would have meant ~90 simultaneous reds and an exemption
/// list as the only route to green. A row is added here — never
/// removed — when its block is clean, so the guard's coverage only ever grows
/// and a regression inside an already-swept block fails immediately.
///
/// `(file, first line, last line)`, inclusive, against `rules/source/en/`.
const SWEPT_BLOCKS: &[(&str, i64, i64)] = &[(
    // S4 (`docs/vf-audit/phase-2-plan.md`, Phase 1S): the whole Virtues
    // and Flaws catalogue, `## Virtues` (ArMDE:3360) through the end of
    // `#### Wrathful` (ArMDE:7106-7109), stopping at the closing
    // dash-and-pull-quote block (ArMDE:7110-7113) before
    // `# Chapter 5: Abilities` (ArMDE:7114) — never the chapter itself,
    // which is exactly the false-offender trap
    // corrections.md § 2.1a documents (ArMDE:7118's "Ease Factor" would
    // otherwise flag two unrelated `narrative` Flaws). Verified against
    // the source file: the Virtues and Flaws blocks are contiguous,
    // `## Flaws` (ArMDE:5639) landing directly after the Virtues
    // section's own trailing `## List of Flaws` TOC (ArMDE:5283-5638),
    // so one range covers both — and every shipped entry's
    // `source.lines` (checked: min 3362, max 7109) lies inside it.
    //
    // Two blocks, swept incrementally and merged here once contiguous:
    // the Flaws half (originally ArMDE:5639-7113) 2026-09-15, re-swept
    // 2026-09-19 under the widened screen (+13 — the first sweep could
    // only see a signed number or a botch die, so every rule the book
    // states in words read as flavour); the Virtues head (originally
    // ArMDE:3360-3950, "to the end of `#### Frightful Presence`")
    // 2026-09-18, re-swept 2026-09-19 with the same widening (+8). The
    // rest of the Virtues block (ArMDE:3951-5638) was swept and the two
    // ranges merged into one 2026-09-26 (S4), under S2's 14 widened
    // families plus S3's class-agnostic
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` (D5) —
    // `every_vf_entry_lies_inside_a_swept_block` is what proves the merge
    // covers the whole catalogue rather than claiming it.
    "Ars Magica - Definitive Edition (Core Rules).md",
    3360,
    7113,
)];

/// Entries whose cited passage trips the mechanical-token screen but, on
/// reading it, states **no rule** — so `narrative` is correct and the screen is
/// a false positive. Each row carries the sentence somebody had to read to write
/// it, in the shape `rules_source_provenance.rs::PARAPHRASE_EXEMPTIONS` already
/// uses.
///
/// This list is *not* the per-item exemption array this module's header warns
/// against, and the difference is the direction of the claim. There, an
/// exemption would absolve an entry of a rule it demonstrably dropped — a bug,
/// silenced. Here the screen is a **heuristic over rulebook prose**, and a row
/// records that a human opened the book and found nothing to drop. It cannot
/// hide a defect, because the defect it would have to hide does not exist; what
/// it records is a review that happened.
///
/// It also cannot rot: [`exempted_entries_still_trip_the_screen`] asserts every
/// row still trips it, so if the citation is ever repointed at a passage that
/// really does state a rule, the row stops being true and the test says so.
const NO_RULE_DESPITE_TOKEN: &[(&str, &str)] = &[
    (
        "flaw.overconfident_major",
        "ArMDE:6562-6565 uses the word \"botch\" as a bare verb in a roleplaying instruction — \
         \"If you actually botch, you come up with some rationalization as to what 'really' \
         happened\" — and states nothing about botch *dice*. The botch it names happens for \
         reasons entirely outside this Flaw; the Flaw only says how the character reacts to one. \
         No number, no die, no modifier: the passage is pure personality flavour, which is \
         exactly `narrative`. Narrowing the detector to \"botch die\"/\"botch dice\" would remove \
         this false positive but lose `flaw.twilight_prone` (\"resist Twilight on a single \
         magical botch\") and `flaw.broken_vessel` (\"If the roll then botches, the character \
         automatically loses enough experience points…\"), both of which state real rules with \
         the bare verb. A broad screen plus two written-down readings beats a sharp screen that \
         misses two live rules.",
    ),
    (
        "flaw.overconfident_minor",
        "The Minor half of the same entry, citing the same passage (ArMDE:6562-6565). Same \
         reading as flaw.overconfident_major.",
    ),
    (
        "flaw.horrifying_appearance_snake_legs",
        "ArMDE:6264-6267 counts tails, not dice: \"your hips give rise to two or more \
         snake-like tails\". The \"or more\" is the only token, and what it quantifies is the \
         character's anatomy — the passage's one near-mechanical sentence, \"Your movement is \
         not hindered under most circumstances\", explicitly declines to impose a penalty. \
         Nothing rolls, nothing is capped, nothing is modified. Pure body-horror description, \
         which is `narrative`.",
    ),
    (
        "flaw.primogeniture_lineage",
        "ArMDE:6634-6637 trips twice and states a rule neither time. \"She is at least three \
         places removed from the Primus\" places her in a fictional succession the engine has \
         no model of — there is no Primus, no line, and no number that changes. \"It would be \
         no more than an interesting feature of her background\" is a turn of phrase. The one \
         genuinely mechanical clause, \"This Flaw can only be taken by magi of House \
         Verditius\", is already *computed*: the entry carries \
         `prerequisites: all(order_member, house.verditius)`, so the rule is enforced rather \
         than merely described, and describing it again would not be `uncomputed_rule`.",
    ),
    (
        "flaw.true_love_major",
        "ArMDE:6871-6878 trips on \"equal to\" inside \"If the True Love is competent, equal to \
         or better than the player character, then this is only a Minor Flaw\" — a comparison \
         of two people's competence, not a formula. The clause it sits in is the book choosing \
         *which magnitude* of the Flaw applies, and that choice is already data: the catalogue \
         splits this passage into a Major and a Minor entry whose `magnitude` fields carry \
         exactly that rule, and marks them `incompatible_with` each other. Nothing is dropped, \
         because there is no third thing for the text to say. The rest of the passage — the \
         bond cannot be sundered, no magic can make you hate your love, the True Love must be \
         a non-player character — is Story-Flaw premise stated in fiction the engine has no \
         concept of.",
    ),
    (
        "flaw.true_love_minor",
        "The Minor half of the same entry, citing the same passage (ArMDE:6871-6878). Same \
         reading as flaw.true_love_major.",
    ),
    // --- S2 additions (docs/vf-audit/corrections.md § 3.1): growing the screen's
    // families newly trips these, and reading each passage finds no rule —
    // an idiom used non-mechanically, not a dropped clause.
    (
        "flaw.busybody",
        "ArMDE:5761-5764's \"at character creation\" (family 2, permission) marks a narrative \
         choice — whether the character's gossip network extends to the lower-class members of \
         the covenant — not an Ability or XP authorization. Nothing is granted, nothing computed.",
    ),
    (
        "flaw.compassionate_major",
        "ArMDE:5809-5812's \"You cannot bear to see suffering in others\" (bare \"cannot\", \
         family 1/3) is the idiom \"can't bear/stand X\" for a strong dislike, describing the \
         character's temperament — not a capability or action the engine restricts.",
    ),
    (
        "flaw.compassionate_minor",
        "The Minor half of the same entry, citing the same passage (ArMDE:5809-5812). Same \
         reading as flaw.compassionate_major.",
    ),
    (
        "flaw.compulsive_lying_major",
        "ArMDE:5817-5820's \"when he cannot be immediately caught out\" (bare \"cannot\", family \
         1/3) describes a narrative CONDITION under which the compulsion manifests — not \
         something the character himself is prohibited from doing.",
    ),
    (
        "flaw.compulsive_lying_minor",
        "The Minor half of the same entry, citing the same passage (ArMDE:5817-5820). Same \
         reading as flaw.compulsive_lying_major.",
    ),
    (
        "flaw.evil_destiny",
        "ArMDE:6028-6035's \"He cannot discuss this openly for fear that he will be accused of \
         infernalism\" (bare \"cannot\", family 1/3) is roleplay/storyguide guidance about a \
         Story Flaw's secrecy premise, not a mechanical restriction on the character.",
    ),
    (
        "flaw.exiled_atlantean",
        "ArMDE:6048-6051's \"cannot return to her magic regio\" (bare \"cannot\", family 1/3) is \
         setting background — the engine has no regio-travel mechanic and nothing about the \
         character's computed state changes because of this sentence.",
    ),
    (
        "flaw.soft_hearted",
        "ArMDE:6775-6778's \"You cannot bear to witness suffering\" (bare \"cannot\", family 1/3) \
         is the same \"can't bear X\" temperament idiom as flaw.compassionate_major, not a \
         restricted capability.",
    ),
    (
        "flaw.tragic_life",
        "ArMDE:6855-6870's \"their creator cannot usually fashion an alternative situation\" \
         (bare \"cannot\", family 1/3) describes a limit on the demon's narrative planning, not \
         on the tainted character it names.",
    ),
    // --- S4 additions (docs/vf-audit/phase-2-plan.md, Phase 1S): SWEPT_BLOCKS
    // widened to the whole catalogue, newly sweeping ArMDE:3951-5638.
    // virtue.ghostly_warder: OQ-6 re-test (X2a) overturns this row rather than
    // confirming it — the "300 experience points" reading above still holds
    // (NPC stats, not a PC rule), but ArMDE:3963-3966 also states "can leave
    // your presence once per day for up to half an hour", a real
    // storyguide-enforced limit on the Virtue's utility that the original
    // reading did not address. Reclassifies to uncomputed_rule (D50); see
    // tmp/x2a-verdicts.md. Removed from here — the passage genuinely states a
    // rule, so `no_swept_entry_drops_an_uncomputed_mechanical_clause` should
    // bite it directly rather than stay silenced.
    (
        "virtue.indescribable_face",
        "ArMDE:4107-4114 states no roll, bonus, or cap at all — \"can't turn the ability off\", \
         \"can switch her distracting prop\" describe a narrative knack for being forgettable or \
         memorable, with nothing for the engine to compute.",
    ),
    (
        "virtue.magical_warder",
        "ArMDE:4377-4384 describes an NPC ally (a magical being that watches over the \
         character) with no PC-facing number — the same ally-template shape as \
         virtue.ghostly_warder, which this entry's own text cites as an example.",
    ),
    (
        "virtue.paid_rights",
        "ArMDE:4606-4615's \"cannot\"/\"may not\" clauses (a woman \"cannot pay a fine to\" \
         various things) describe in-fiction social restrictions this Virtue lets a female \
         character buy an exception to — a story premise, not a PC-sheet rule with a number or \
         an Ability authorization attached.",
    ),
    (
        "virtue.tainted_treasure",
        "ArMDE:5097-5108 states no PC-facing number — the treasure's curse (ventures fail, \
         buildings burn) is pure story consequence, and the one concrete mechanical path it \
         names (\"transform it into a source of the Wealthy Virtue\") is Wealthy's own already-\
         computed later_life_xp_rate, not a new clause.",
    ),
    // --- X2a fix-round additions (2026-09-29): the new "capability"/
    // "prohibition/absolutes" idioms above newly sweep this one. (Its sibling
    // flaw.painful_magic is NOT narrative — in_play_effect — so it cannot live
    // here; see PENDING_DROPPED_CLAUSE below.)
    (
        "flaw.simple_minded",
        "ArMDE:6741-6744's \"can only\" (family: capability/restriction limiter) is \"You can \
         only think about one thing at a time\" — a Personality Flaw's temperament description \
         (easily confused, needs clear instructions), with no roll, cap, or number attached. \
         Pure narrative flavour.",
    ),
];

/// Entries a **new** S2 phrase family newly flags, where reading the passage
/// finds a real rule the engine does not yet compute — as opposed to
/// [`NO_RULE_DESPITE_TOKEN`], whose rows record that a human read the passage
/// and found *no* rule. This is the opposite kind of row: the rule is real,
/// and something else (reclassification to `uncomputed_rule` plus the
/// `description` text in both locales, which is X2's job; the authorization
/// family, X1's) has to land before the entry can leave `narrative` — not
/// this test-only slice, which is why growing the screen must not itself
/// force a data change here.
///
/// `(id, phrase-or-family)` — the second field names what now trips it, so
/// the next reader does not have to re-derive it. Every commit stays green
/// (plan § 1): each listed id must **still trip** the screen
/// ([`pending_mechanical_classification_entries_still_trip_the_screen`]), so
/// a later fix has to remove the row rather than leave a stale one — the
/// list can only shrink. Every *unlisted* swept `narrative` entry must still
/// pass — [`no_swept_entry_drops_an_uncomputed_mechanical_clause`] enforces
/// that directly, since this list is the only thing it skips besides
/// [`NO_RULE_DESPITE_TOKEN`]. X2 is the slice that empties it.
const PENDING_MECHANICAL_CLASSIFICATION: &[(&str, &str)] = &[
    (
        "flaw.a_deal_with_the_devil",
        "item transfer: \"includes the effects of\" (Plagued By Supernatural Entity), ArMDE:5905-5908",
    ),
    (
        "flaw.ability_block",
        "prohibition: \"completely unable to learn\" a class of Abilities, ArMDE:5651-5654",
    ),
    (
        "flaw.bigamist",
        "multiplier stem: \"Wealth Multiplier\" cost formula, ArMDE:5699-5702",
    ),
    (
        "flaw.blackmail",
        "\"possibly more\" (family 9): a quantified yearly value with an escalation condition, \
         ArMDE:5707-5710",
    ),
    (
        "flaw.blind",
        "prohibition: bare \"cannot\" — \"cannot aim spells without magical aid\", ArMDE:5719-5722",
    ),
    (
        "flaw.bound_magic",
        "prohibition: bare \"cannot\" — incompatible with Harnessed Magic, ArMDE:5727-5730",
    ),
    (
        "flaw.ceremonial_spontaneous_magic",
        "incompatibility: \"is not compatible with\", ArMDE:5781-5784",
    ),
    (
        "flaw.chaotic_magic",
        "magnitude/level stem: \"by more than one level\", ArMDE:5785-5788",
    ),
    (
        "flaw.companion_animal",
        "named rulebook term: \"an additional Personality Trait of\", ArMDE:5805-5808",
    ),
    (
        "flaw.consumed_casting_tools",
        "eligibility: \"may only be taken by\" Verditius magi, ArMDE:5839-5842",
    ),
    (
        "flaw.crippled",
        "prohibition: bare \"cannot\" — \"cannot walk\", ArMDE:5877-5880",
    ),
    (
        "flaw.deteriorating_power",
        "magnitude/level stem: \"reduced by 3 magnitudes\" (D8), ArMDE:5944-5949",
    ),
    (
        "flaw.difficult_spontaneous_magic",
        "prohibition: bare \"cannot\" — \"cannot use Spontaneous magic at all\" in combination, \
         ArMDE:5966-5971",
    ),
    (
        "flaw.difficult_underlings",
        "eligibility: \"may only take this\", ArMDE:5976-5979",
    ),
    (
        "flaw.disorientating_magic",
        "named rulebook term: \"spend a round\", ArMDE:5984-5987",
    ),
    (
        "flaw.enfeebled",
        "prohibition: \"unable to learn\" / bare \"cannot\" — \"cannot train\", ArMDE:6008-6011",
    ),
    (
        "flaw.envied_beauty",
        "prohibition: \"may not have\" this Flaw without a positive Presence, ArMDE:6012-6015",
    ),
    (
        "flaw.exciting_experimentation",
        "named rulebook term: \"two dice instead of\" the normal one, ArMDE:6040-6043",
    ),
    (
        "flaw.false_power",
        "capability: \"the ability to\" sense the taint, plus extensive realm-interaction \
         mechanics the passage states (D8-adjacent), ArMDE:6080-6097",
    ),
    (
        "flaw.false_power_minor",
        "The Minor half of the same entry, citing the same passage. Same trigger as \
         flaw.false_power.",
    ),
    (
        "flaw.fettered_magic",
        "prohibition: bare \"cannot\" — incompatible with Tethered Magic, ArMDE:6114-6117",
    ),
    (
        "flaw.harmless_magic",
        "prohibition: bare \"cannot\" — \"cannot permanently destroy anything\", ArMDE:6230-6235",
    ),
    (
        "flaw.hermetic_patron",
        "eligibility: \"must be\" a Redcap or magus \"to take this\" Flaw, ArMDE:6248-6255",
    ),
    (
        "flaw.incompatible_arts",
        "incompatibility: \"may not be combined with\" a Deficiency, ArMDE:6290-6293",
    ),
    (
        "flaw.judged_unfairly",
        "prohibition + incompatibility: bare \"cannot\" gain a Reputation, \"is incompatible \
         with\", ArMDE:6326-6329",
    ),
    (
        "flaw.magical_air",
        "prohibition: \"may not take\" this Flaw with The Gift, ArMDE:6382-6385",
    ),
    (
        "flaw.magical_being_companion",
        "signed-number blind spot F-469 (S2, has_signed_number): \"Magic Might score of 10 – \
         Size\" — digit, EN DASH, named quantity, ArMDE:6390",
    ),
    (
        "flaw.master_of_none",
        "prohibition: \"can't apply\" experience points earned this year, ArMDE:6418-6421",
    ),
    (
        "flaw.monastic_vows_hermetic",
        "prohibition: bare \"cannot\" — \"cannot own vis\", \"cannot marry\", ArMDE:6450-6453",
    ),
    (
        "flaw.motion_sickness",
        "multiplier in words + floor: \"double the fatigue loss\", \"minimum loss of two Fatigue \
         levels\", ArMDE:6468-6471",
    ),
    (
        "flaw.necessary_condition",
        "prohibition: bare \"cannot\" — \"cannot cast spells at all\", ArMDE:6476-6479",
    ),
    (
        "flaw.no_hands",
        "signed-number blind spot F-464 (S2, has_signed_number): \"take a – 5 penalty\" — EN \
         DASH, space, digit, ArMDE:6498",
    ),
    (
        "flaw.no_sense_of_direction",
        "incompatibility: \"is incompatible with\" the Well-Traveled Virtue, ArMDE:6500-6503",
    ),
    (
        "flaw.outcast",
        "prohibition: \"may not take\" the Wealthy Virtue, ArMDE:6538-6541",
    ),
    (
        "flaw.restricted_learning",
        "permission: \"at character creation\" — the five-Ability restriction itself, \
         ArMDE:6683-6686",
    ),
    (
        "flaw.restriction",
        "prohibition: bare \"cannot\" — \"cannot cast spells at all\" under conditions, \
         ArMDE:6691-6694",
    ),
    (
        "flaw.sheltered_upbringing",
        "prohibition: \"may not take\" several Abilities as beginning Abilities, ArMDE:6721-6724",
    ),
    (
        "flaw.stockade_parma_magica",
        "prohibition: bare \"cannot\" — \"cannot suppress your Parma\", ArMDE:6787-6790",
    ),
    (
        "flaw.study_requirement",
        "permission: \"may take both\" Study Bonus and Study Requirement, ArMDE:6795-6798",
    ),
    (
        "flaw.suppressed_gift",
        "prohibition: bare \"cannot\" — \"cannot perform Hermetic magic\", ArMDE:6803-6810",
    ),
    (
        "flaw.tainted_with_evil",
        "prohibition: \"is impossible\" — gaining a positive Reputation, ArMDE:6843-6846",
    ),
    (
        "flaw.unnatural_magic",
        "prohibition: bare \"cannot\" — \"cannot extract vis from an aura using Creo\", \
         ArMDE:6931-6934",
    ),
    (
        "flaw.vulnerable_magic",
        "incompatibility: \"may not be combined with\" Restrictions/Necessary Conditions — F-537's \
         and B19's own motivating example for this whole family, ArMDE:7005-7010",
    ),
    (
        "flaw.wanderlust",
        "prohibition: bare \"cannot\" — \"cannot spend more than a season in the same place\", \
         ArMDE:7015-7018",
    ),
    // virtue.amorphous_major/_minor, virtue.covenfolk: resolved (X2a) — the
    // capability clause reclassifies to uncomputed_rule (amorphous) or the
    // entry's own incompatible_with turns out to cover the whole passage
    // (covenfolk, creation_effect) — see tmp/x2a-verdicts.md and
    // data_integrity.rs::PENDING_D67_CLASSIFICATION, which carried the D67 half
    // of these three rows.
    // --- S4 additions (docs/vf-audit/phase-2-plan.md, Phase 1S): SWEPT_BLOCKS
    // widened to the whole catalogue, newly sweeping ArMDE:3951-5638.
    // virtue.gorgiastic, virtue.gossip, virtue.greater_benediction,
    // virtue.greater_immunity, virtue.guardian_angel, virtue.harnessed_magic:
    // resolved (X2a) — all six reclassify narrative -> uncomputed_rule, see
    // tmp/x2a-verdicts.md.
    (
        "virtue.homing_instinct",
        "target number: \"an Ease Factor of 6\", ArMDE:4079-4084",
    ),
    (
        "virtue.imbued_with_the_spirit_of_form",
        "named rulebook term + formula: Fatigue-for-vis substitution, \"reduces the vis \
         requirement... by 1\", ArMDE:4085-4094",
    ),
    (
        "virtue.inoffensive_to_beings",
        "capability: the Gift \"does not bother\" beings of the chosen type is a real Gift-\
         penalty exemption with no effect, though the eligibility half is already a \
         `prerequisites` (any(has The Gift, has Magical Air)), ArMDE:4133-4142",
    ),
    (
        "virtue.inspirational",
        "signed number: \"+3 bonus to rolls for appropriate Personality Traits\", \
         ArMDE:4143-4146",
    ),
    (
        "virtue.intuition",
        "target number: \"secretly roll a simple die. On a 6+\", ArMDE:4147-4150",
    ),
    (
        "virtue.kassalan_exorcism",
        "formulas + capability: Casting Total \"(Stamina + Organization Lore... )/2\", \
         Penetration formula, ArMDE:4173-4186",
    ),
    (
        "virtue.keen_sense_of_smell",
        "signed number: \"+3 bonus to all rolls involving\" smell, ArMDE:4191-4194",
    ),
    (
        "virtue.keen_vision",
        "signed number: \"+3 bonus to all rolls involving sight\", ArMDE:4187-4190",
    ),
    (
        "virtue.land_regio_network",
        "target number: \"an Ease Factor of 9\", ArMDE:4211-4218",
    ),
    (
        "virtue.learn_ability_from_mistakes",
        "named rulebook term: \"gain five experience points\" on a botch/near-miss, \
         ArMDE:4241-4244",
    ),
    (
        "virtue.leather_ripper",
        "signed number + formula: \"-9 depending on\" maneuvers, \"PeAn(He) 30 effect\", \
         ArMDE:4245-4248",
    ),
    (
        "virtue.lesser_benediction",
        "signed numbers: the Gift of the Gab/Green Fingers/Pricking Thumbs examples carry \
         concrete roll penalties/bonuses, ArMDE:4253-4274",
    ),
    (
        "virtue.license_of_absence",
        "named rulebook term + eligibility: an extra free season, \"four free seasons in a \
         year\", \"may only be taken by\" Priest, ArMDE:4291-4294",
    ),
    (
        "virtue.luck",
        "signed number (open-ended, GM-adjudicated): \"+1 to +3 (storyguide's discretion)\", \
         ArMDE:4331-4334",
    ),
    (
        "virtue.magical_mount",
        "eligibility: a gifted mount requires \"a Major Story Flaw to represent the \
         consequences\", a real required-companion-Flaw clause, ArMDE:4373-4376",
    ),
    (
        "virtue.maker_of_textured_vessels",
        "signed number: \"+3 bonus in a single Ability\" from a crafted vessel, \
         ArMDE:4423-4430",
    ),
    (
        "virtue.maker_of_water_vessels",
        "capability: \"swap one Ability score for the Craft: Potter score\", ArMDE:4431-4438",
    ),
    (
        "virtue.minor_enchantments",
        "named rulebook term: item power levels \"must be 25 or less\", \"no single power can \
         be greater than 30th level\", ArMDE:4532-4535",
    ),
    (
        "virtue.muse",
        "capability: \"may grant Free Expression\" or \"double the effect of Free Expression\", \
         ArMDE:4563-4566",
    ),
    (
        "virtue.natural_leader",
        "signed number: \"+3 bonus to rolls in social situations\", ArMDE:4590-4593",
    ),
    (
        "virtue.perfect_balance",
        "signed number: \"Add +6 to any roll to avoid falling or tripping\", ArMDE:4624-4627",
    ),
    (
        "virtue.perfect_eye_for_commodity",
        "named rulebook term: \"(3 x Wealth Multiplier) Labor Points per year\" — conditioned \
         on the City and Guild trading rules the app does not model, the same D4/N6 shape as \
         virtue.aristotelian_training, ArMDE:4628-4631",
    ),
    (
        "virtue.performance_magic",
        "target number + formula: \"an Ease Factor of 3\", the words/gestures Ease Factor table, \
         ArMDE:4642-4709",
    ),
    (
        "virtue.piercing_gaze",
        "signed number: \"+3 to rolls involving intimidation\", ArMDE:4736-4739",
    ),
    (
        "virtue.reserves_of_strength",
        "signed number: \"add +3 to your effective Strength score\", ArMDE:4862-4865",
    ),
    (
        "virtue.ripper",
        "formula: \"a PeAn(He) 25 and a PeAn 45 effect\", ArMDE:4866-4869",
    ),
    (
        "virtue.sharp_ears",
        "signed number: \"+3 bonus to all rolls involving hearing\", ArMDE:4950-4953",
    ),
    (
        "virtue.side_effect",
        "signed number (open-ended, player-chosen): \"+1 Presence bonus\", \"a bonus on \
         Concentration rolls\", ArMDE:4954-4957",
    ),
    (
        "virtue.skilled_smuggler",
        "signed number: \"a -9 penalty on Awareness rolls\", ArMDE:4968-4971",
    ),
    (
        "virtue.skinchanger",
        "signed number + range: \"+3 is added to the character's Soak score\", \"Size -10... to \
         Size +2\", ArMDE:4972-4975",
    ),
    (
        "virtue.skinchanger_dove",
        "signed number: \"Soak is +3 higher than usual\", ArMDE:4976-4987",
    ),
    (
        "virtue.social_contacts",
        "target number: \"a simple Presence roll against an Ease Factor of 6\", \
         ArMDE:4988-4991",
    ),
    (
        "virtue.spiritual_pact",
        "formula: \"Presence + Magic Lore + stress die\" for a Might Pool, ArMDE:5010-5021",
    ),
    (
        "virtue.strong_willed",
        "signed number: \"+3 on any roll which may require strength of will\", ArMDE:5048-5051",
    ),
    (
        "virtue.supernatural_beauty",
        "capability (open-ended, GM-adjudicated): \"once per story\" insert a fortunate \
         coincidence, ArMDE:5089-5096",
    ),
    (
        "virtue.temporal_influence",
        "eligibility: \"Grogs may not take this Virtue\", ArMDE:5137-5140",
    ),
    (
        "virtue.troupe_upbringing",
        "signed number: \"receive a +2 modifier\", ArMDE:5165-5168",
    ),
    (
        "virtue.true_love_pc",
        "signed number: \"add +3 to appropriate Personality Trait rolls\", ArMDE:5173-5178",
    ),
    (
        "virtue.variable_power",
        "formula: power level scaling by \"(age / 10)\", \"(Might Score / 5)\", \
         ArMDE:5199-5206",
    ),
    (
        "virtue.venus_blessing",
        "signed number: \"+3 on Communication and Presence rolls\", ArMDE:5211-5214",
    ),
    (
        "virtue.verditius_magic",
        "capability: \"enabling the casting of enchantments through craft\" carries no effect \
         at all, unlike its sibling Outer-Mystery entries (Faerie Magic, Heartbeast, The \
         Enigma), ArMDE:5215-5217",
    ),
    (
        "virtue.wisdom_from_ignorance",
        "capability: books as a training source \"provided they are unable to read the \
         language\", ArMDE:5251-5256",
    ),
    // --- Corrected disposition (post-S4 review): these two were briefly
    // parked in NO_RULE_DESPITE_TOKEN, which is wrong — that list certifies
    // "narrative is correct", and D46 (docs/vf-audit/decisions.md) rules the
    // opposite for both: the stated rule *is* computed, via a character-type
    // profile's required_traits/forbidden_traits naming the entry, so
    // `narrative` is the misclassification, not a false-positive screen hit.
    // Both already sit in data_integrity.rs::PENDING_D46_CLASSIFICATION for
    // exactly this reason; this row is the mirror finding for THIS guard.
    // virtue.the_gift: resolved (X2a) — reclassifies to uncomputed_rule per
    // D46/D67 (`x2_reclassification.rs`); removed from
    // data_integrity.rs::PENDING_D46_CLASSIFICATION too, see
    // tmp/x2a-verdicts.md.
    (
        "virtue.hermetic_magus",
        "permission/eligibility: \"All magi must take this as their Social Status, and only \
         magi may take it\", ArMDE:4067-4070 — computed via the magus profile's required_traits \
         naming virtue.hermetic_magus (D46's own worked example), so `narrative` is wrong; X2 \
         reclassifies to creation_effect per D46's ruling.",
    ),
    // --- X2a fix-round additions (2026-09-29): the new "capability"/
    // "eligibility" idioms above newly sweep these two real, not-yet-landed
    // findings. Neither is on the X2a worklist (`tmp/x2-worklist.md` § 1 rows
    // 1-51); each is a genuine dropped clause for a later X2 slice. (A third,
    // virtue.simple_student, is NOT narrative — creation_effect — so it cannot
    // live here; see PENDING_DROPPED_CLAUSE below.)
    (
        "flaw.spontaneous_casting_tools",
        "eligibility: \"This Flaw can only be taken by Verditius magi\", ArMDE:6779-6782 — \
         `narrative`, no `prerequisites`, no effect; the restriction reaches the player nowhere \
         today",
    ),
    (
        "virtue.lesser_purifying_touch",
        "capability/restriction limiter: \"You can only choose an illness, not an injury or \
         other misfortune\", ArMDE:4287-4290 — `narrative`, the same shape as its sibling \
         virtue.greater_purifying_touch (X2a) before its own restoration",
    ),
];

/// D5's first obligation (`docs/vf-audit/decisions.md`): a `creation_effect` or
/// `in_play_effect` entry whose cited passage states a mechanical clause the
/// screen newly flags, where reading it finds the engine's `effects` already
/// cover the **whole** passage — so nothing is actually dropped, and the
/// screen's hit is a false positive on an already-computed entry. The
/// computed-class mirror of [`NO_RULE_DESPITE_TOKEN`]: a row records that a
/// human read the passage and the entry's own effects, and found no gap
/// between them. [`computed_entry_covers_whole_passage_entries_still_trip_the_screen`]
/// keeps every row honest the same way `exempted_entries_still_trip_the_screen`
/// does for `NO_RULE_DESPITE_TOKEN`. Two shapes of false positive live here
/// side by side, exactly as `NO_RULE_DESPITE_TOKEN` already mixes reasons:
/// most rows are a single clause whose one number is the entry's one effect
/// (nothing to drop); `flaw.black_sheep` and `flaw.poor` are the other shape
/// — the displayed text already states the clause, just not in wording this
/// screen's token vocabulary recognizes (a German synonym, an unlisted
/// quantifier), a screen gap rather than a content gap.
///
/// F2 (design-f0-book-template-engine.md § 2c/Revision 3 MAJOR #3, D61):
/// `virtue.cyclic_magic_positive` and `flaw.cyclic_magic_negative` used to
/// carry a row here reading "both stated bonuses... are computed via two
/// effects" — true only while both `casting_total_mod` and `lab_total_mod`
/// were computed. Deleting the Casting clause's effect makes that claim
/// false, and this guard matches by id only, so a stale row would go on
/// silencing the screen forever. Removed, not rewritten: the added
/// `description` (both locales) alone is enough to satisfy
/// [`no_swept_entry_drops_an_uncomputed_mechanical_clause`] for the
/// now-uncomputed Casting clause, so no exemption is needed at all.
const COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE: &[(&str, &str)] = &[
    (
        "virtue.covenfolk",
        "X2a (tmp/x2a-verdicts.md): the only mechanical clause in ArMDE:3609-3612 — \"You may not \
         take the Wealthy Major Virtue or the Poor Major Flaw\" — is the entry's own \
         `incompatible_with` (Wealthy, Poor); the rest of the passage is pure social-status \
         flavor, so classification moved narrative -> creation_effect (D67) and no separate \
         description is owed.",
    ),
    (
        "flaw.diabolic_past",
        "single clause (\"may purchase... Infernal Lore, even if... not permitted to buy Arcane \
         Abilities\"), fully computed via ability_authorization (X1/D43).",
    ),
    (
        "flaw.faerie_friend",
        "single clause (\"can purchase\" Faerie Lore \"even if... normally restricted\"), fully \
         computed via ability_authorization (X1/D43).",
    ),
    (
        "flaw.faerie_upbringing",
        "single clause (\"may learn\" Faerie Lore \"at character generation\"), fully computed \
         via ability_authorization (X1/D43).",
    ),
    (
        "flaw.pagan",
        "operative clause (\"may begin with\" Magic Lore or Faerie Lore) is computed via \
         ability_authorization (X1/D43); the regional Novgorod exception is a setting note, not \
         a numeric rule.",
    ),
    (
        "virtue.alim",
        "operative clause (\"may purchase\" Academic Abilities) is computed via \
         ability_authorization (X1/D43); \"male characters only\" is a flavor/eligibility note, \
         not a numeric rule.",
    ),
    (
        "virtue.almogaten",
        "operative clause (\"may take\" Martial Abilities) is computed via ability_authorization \
         (X1/D43); the Standard Armaments/Wealthy-Poor variant and the Iberian-availability note \
         are flavor.",
    ),
    (
        "virtue.archieunuch",
        "operative clause (\"may take\" Academic Abilities) is computed via \
         ability_authorization (X1/D43); the eunuch/male eligibility note is flavor.",
    ),
    (
        "virtue.beadle",
        "single clause (\"may purchase\" Academic Abilities \"at character generation\"), fully \
         computed via ability_authorization (X1/D43).",
    ),
    (
        "virtue.brother_chaplain",
        "single clause (\"may purchase\" Academic Abilities), fully computed via \
         ability_authorization (X1/D43).",
    ),
    (
        "virtue.brother_knight",
        "operative clause (\"may take\" Academic and Martial Abilities) is computed via \
         ability_authorization (X1/D43); the equipment grant (\"high-quality weapons and armor, \
         and two horses\") names no signed number the screen recognizes.",
    ),
    (
        "virtue.bureaucrat",
        "single clause (\"may take\" Academic Abilities), fully computed via \
         ability_authorization (X1/D43).",
    ),
    (
        "virtue.clerk",
        "operative clause (\"may take\" Academic Abilities) is computed via \
         ability_authorization (X1/D43); the clergy-order/marriage eligibility prose is flavor.",
    ),
    (
        "virtue.eunuch",
        "operative clause (\"may take\" Academic Abilities) is computed via \
         ability_authorization (X1/D43); the male/castration eligibility note is flavor.",
    ),
    (
        "virtue.failed_apprentice",
        "operative clause (\"may learn\" Academic/Arcane/Martial Abilities) is computed via \
         ability_authorization (X1/D43); \"may not have The Gift\" is already \
         incompatible_with:[virtue.the_gift]; \"you may have some Supernatural Abilities\" \
         names no specific Ability and is gated per-Ability by validate_supernatural_abilities \
         regardless (ArMDE:2315).",
    ),
    (
        "virtue.fidai",
        "single clause (\"may take\" Martial Abilities \"at character creation\"), fully \
         computed via ability_authorization (X1/D43).",
    ),
    (
        "virtue.guild_dean",
        "single clause (\"may select\" Academic Abilities), fully computed via \
         ability_authorization (X1/D43).",
    ),
    (
        "virtue.guild_master",
        "single clause (\"may select\" Academic Abilities), fully computed via \
         ability_authorization (X1/D43).",
    ),
    (
        "virtue.jurist",
        "single clause (\"may purchase\" Latin, Artes Liberales, Civil and Canon Law), fully \
         computed via ability_authorization (X1/D43, id-form).",
    ),
    (
        "virtue.knight",
        "operative clause (\"may take\" Martial Abilities) is computed via \
         ability_authorization (X1/D43); \"the Wealthy Virtue and Poor Flaw affect you \
         normally\" states there is NO special interaction, so nothing is dropped; the \
         male-only eligibility note is flavor.",
    ),
    (
        "virtue.mamluk",
        "operative clause (\"may take\" Martial Abilities, \"may also take\" Theology: Islam) is \
         computed via ability_authorization (X1/D43); the male-only/companion-compatibility \
         notes are flavor.",
    ),
    (
        "virtue.master_of_form_creatures",
        "operative clause (\"may take\" Magic Lore) is computed via ability_authorization \
         (X1/D43); \"may be taken multiple times, once for each Form\" is already max_total:255.",
    ),
    (
        "virtue.mazdean_priest",
        "operative clause (\"may take\" Academic Abilities) is computed via \
         ability_authorization (X1/D43); the Outsider-Flaw suggestion and male-only note are \
         flavor.",
    ),
    (
        "virtue.mercenary_captain",
        "operative clause (\"may take\" Martial Abilities) is computed via \
         ability_authorization (X1/D43); the Poor/Wealthy company-size prose names no signed \
         modifier.",
    ),
    (
        "virtue.notary",
        "operative clause (\"may take\" Academic Abilities) is computed via \
         ability_authorization (X1/D43); \"notaries may not be members of the clergy\" is an \
         eligibility restriction on a Social Status the app does not model as a cross-Virtue \
         exclusion set.",
    ),
    (
        "virtue.prestigious_student",
        "operative clause (\"may purchase\" Academic Abilities) is computed via \
         ability_authorization (X1/D43); \"must take a Social Status Virtue\" is a companion \
         co-requirement already required whole-character (D41).",
    ),
    (
        "virtue.religious",
        "operative clause (\"may take\" Academic Abilities) is computed via \
         ability_authorization (X1/D43); the Wealthy/Poor advisory note and the pointer to \
         alternate Virtues are flavor.",
    ),
    (
        "virtue.senior_master",
        "single clause (\"may select\" Academic Abilities), fully computed via \
         ability_authorization (X1/D43).",
    ),
    (
        "virtue.town_magistrate",
        "operative clause (\"Academic Abilities may be bought\") is computed via \
         ability_authorization (X1/D43); the Ability-3 (Civil and Canon Law) prerequisite is a \
         separate, unencoded `prereq`-kind finding (F-322) the screen does not itself detect.",
    ),
    (
        "virtue.sufi",
        "operative clause (\"may purchase\" Theology: Islam, Islamic Law, Dominion Lore) is \
         computed via ability_authorization (X1/D43, id-form); the no-points Story Flaw \
         mechanic (F-303) is a separate, untracked finding, not this row's concern.",
    ),
    (
        "virtue.templar_administrator",
        "operative clause (\"may take\" Academic Abilities) is computed via \
         ability_authorization (X1/D43); the Status-substitution rule and male-only note are \
         flavor.",
    ),
    (
        "virtue.troubadour",
        "operative clause (\"may take\" Academic skills) is computed via ability_authorization \
         (X1/D43); the Wealthy/Poor advisory note and companion Virtue suggestions are flavor.",
    ),
    (
        "virtue.university_grammar_teacher",
        "single clause (\"may purchase\" Latin and Artes Liberales), fully computed via \
         ability_authorization (X1/D43, id-form); \"should have a score in Teaching\" is a soft \
         recommendation (\"should\", not \"must\").",
    ),
    (
        "flaw.black_sheep",
        "single clause (bad Reputation at level 2), fully computed via grants_reputation. \
         DE summary already states it, using \"Ruf\" rather than the rulebook's \"Reputation\" \
         loanword — a screen vocabulary gap (family 14 only recognizes \"Reputation der Stufe\"), \
         not a dropped rule.",
    ),
    (
        "flaw.covenant_upbringing",
        "operative clause (\"may take Latin\") is computed via ability_authorization \
         (over-permissively — the dead_language/Latin binding is D14's already-tracked \
         defect, not a new one here); the rest of the passage is flavor.",
    ),
    (
        "flaw.deficient_technique",
        "the halving is computed via deficient_art; the Advancement-Total exception and the \
         pre-halving XP basis are refinements of that same mechanism, handled downstream \
         (D12's family).",
    ),
    (
        "flaw.difficult_longevity_ritual",
        "single clause (halve the Lab Total for a Longevity Ritual for this character), fully \
         computed via magic_total_halving/lab_longevity; \"without penalty for others\" is \
         already implicit in that total's scope.",
    ),
    (
        "flaw.dwarf",
        "all three signed numbers (Size -2, Strength -1, Stamina -1) match three effects \
         exactly; the Giant Blood/Large/Small Frame exclusion is in incompatible_with, a \
         separately validated field.",
    ),
    (
        "flaw.failed_student",
        "single clause (Bad Academic Reputation 2), fully computed via grants_reputation.",
    ),
    (
        "flaw.flawed_parma_magica",
        "single clause (half Magic Resistance vs one Form), fully computed via \
         magic_resistance_mod/halved_parma.",
    ),
    (
        "flaw.foreign_upbringing",
        "single clause (locality-dependent Ability caps halved, rounded up), fully computed \
         via locality_ability_cap_fraction; the rest is flavor.",
    ),
    (
        "flaw.fragile_constitution",
        "single clause (-3 to recovery rolls), fully computed via health_mod/recovery.",
    ),
    (
        "flaw.frail",
        "single clause (-3 Soak), fully computed via soak_mod.",
    ),
    (
        "flaw.gabai",
        "single clause (-2 local Reputation \"Tax Collector\"), fully computed via \
         grants_reputation; the Free-Social-Status compatibility note needs no effect.",
    ),
    (
        "flaw.incomprehensible",
        "both stated halvings (teaching, authoring/Lab Texts) are computed via two \
         advancement_mod effects with matching sources.",
    ),
    (
        "flaw.outsider_major",
        "single clause (bad Reputation, level 1-3), fully computed via grants_reputation with \
         a matching max_score.",
    ),
    (
        "flaw.outsider_minor",
        "the Minor half of the same entry/passage. Same reading as flaw.outsider_major.",
    ),
    (
        "flaw.poor",
        "the operative clause is computed via later_life_xp_rate (the shipped \
         wealthy_and_poor_ship_with_their_rates_and_eligibility test covers it), and the \
         displayed text already paraphrases it (\"one fewer season\") — a screen vocabulary \
         gap (no listed idiom for \"one fewer\"), not a dropped rule.",
    ),
    (
        "flaw.poor_eyesight",
        "single clause (-3 to sight-involving rolls, including attack/defense), computed via \
         two combat_mod effects.",
    ),
    (
        "flaw.poor_formulaic_magic",
        "single clause (-5 to Formulaic casting rolls), fully computed via \
         casting_total_mod/formulaic.",
    ),
    (
        "flaw.poor_living_conditions",
        "single clause (-1 Living Conditions Modifier), fully computed via \
         aging_mod/living_conditions.",
    ),
    (
        "flaw.poor_student",
        "both stated halvings (teaching, books) are computed via two advancement_mod effects; \
         the never-below-1 floor is the engine's generic advancement-total invariant.",
    ),
    (
        "flaw.short_ranged_magic",
        "both stated halvings (Casting Totals off Touch, Lab Total for range > Touch) are \
         computed — special_casting_mod and halves_spell_cap_beyond_touch (Q10/D28).",
    ),
    (
        "flaw.slow_reflexes",
        "single clause (-3 Initiative), fully computed via combat_mod/initiative.",
    ),
    (
        "flaw.small_frame",
        "single clause (Size -1), fully computed via size_delta; the Giant Blood/Large/Dwarf \
         exclusion is in incompatible_with.",
    ),
    (
        "flaw.unimaginative_learner",
        "single clause (-3 studying from raw vis), fully computed via advancement_mod/vis.",
    ),
    (
        "flaw.weak_characteristics",
        "single clause (-3 Characteristic points), fully computed via characteristic_points; \
         \"may take twice\" is max_per_target/max_total, not an effect.",
    ),
    (
        "virtue.adept_laboratory_student",
        "single clause (+6 Lab Totals from others' lab texts), fully computed via \
         lab_total_mod.",
    ),
    (
        "virtue.apt_student",
        "single clause (+5 Source Quality when taught), fully computed via \
         advancement_mod/taught.",
    ),
    (
        "virtue.baccalaureus",
        "the XP grant and the Academic Reputation are both computed; the male-characters-unless \
         exception is the same unenforced eligibility footnote every Social-Status Virtue in \
         this block carries, not a new gap.",
    ),
    (
        "virtue.crafters_healing",
        "single clause (confers the Crafter's Healing Ability at 1), fully computed via \
         ability_score_grant.",
    ),
    (
        "virtue.curse_throwing",
        "single clause (confers the Curse-Throwing Ability at 1), fully computed via \
         ability_score_grant.",
    ),
    // virtue.demonic_might: moved to PENDING_DROPPED_CLAUSE (X2a) — the
    // certification below was incomplete: the +2 Infernal Might grant and the
    // Demonic Blood prerequisite are indeed fully computed (and the "no more
    // than half of total Virtues" cap is separately encoded via
    // `max_share_of_kind`), but "her body contains a number of pawns of Corpus
    // vis equal to (Infernal Might / 5), rounding up" upon death is computed by
    // no effect at all. See tmp/x2a-verdicts.md.
    (
        "virtue.demonic_powers",
        "single clause (+20 Infernal Power levels), fully computed via power_levels; the \
         Demonic Blood prerequisite is already a `prerequisites` field.",
    ),
    (
        "virtue.dowsing",
        "single clause (confers the Dowsing Ability at 1), fully computed via \
         ability_score_grant.",
    ),
    (
        "virtue.faerie_magic",
        "single clause (confers the Faerie Magic Ability at 1), fully computed via \
         ability_score_grant plus the house.merinita prerequisite; the free-for-House-members \
         grant is the Houses-phase granted_selections machinery, not this entry's own effect.",
    ),
    (
        "virtue.falconer",
        "single clause (50 XP on the named Ability list), fully computed via \
         restricted_ability_xp.",
    ),
    (
        "virtue.flawless_magic",
        "both stated clauses (starts every spell at Mastery 1, doubled Advancement Totals) are \
         computed in one grants_spell_mastery effect (score plus advancement_num/den).",
    ),
    (
        "virtue.free_study",
        "single clause (+3 Source Quality studying from raw vis), fully computed via \
         advancement_mod/vis.",
    ),
    (
        "virtue.the_enigma",
        "single clause (confers Enigmatic Wisdom at 1), fully computed via ability_score_grant \
         plus the house.criamon prerequisite; the free-for-House-members grant is the \
         Houses-phase granted_selections machinery.",
    ),
    // --- S4 additions (docs/vf-audit/phase-2-plan.md, Phase 1S): SWEPT_BLOCKS
    // widened to the whole catalogue, newly sweeping ArMDE:3951-5638.
    (
        "virtue.giant_blood",
        "all three signed numbers (Size +2, Strength +1, Stamina +1) match three effects \
         exactly; the Large/Small Frame/Dwarf exclusion is in incompatible_with.",
    ),
    (
        "virtue.good_teacher",
        "both stated bonuses (+3 authoring Quality, +5 teaching Source Quality) are computed \
         via two advancement_mod effects.",
    ),
    (
        "virtue.greater_power",
        "the power-level budget (50) is computed via power_levels; the Initiative/Fatigue-cost/\
         Penetration formulas are the same generic power-invocation mechanism shared by \
         Lesser/Personal/Ritual Power, not this entry's own job.",
    ),
    (
        "virtue.heartbeast",
        "single clause (confers the Heartbeast Ability at 1), fully computed via \
         ability_score_grant plus the house.bjornaer prerequisite.",
    ),
    (
        "virtue.hermetic_experience",
        "single clause (50 XP on three named Abilities), fully computed via \
         restricted_ability_xp; the who-may-take-it footnote is non-binding (\"usually no \
         point\", not a prohibition).",
    ),
    (
        "virtue.hermetic_prestige",
        "single clause (Reputation of level 4 within the Order), fully computed via \
         grants_reputation.",
    ),
    (
        "virtue.independent_study",
        "both stated bonuses (+2 practice, +3 adventure) are computed via two advancement_mod \
         effects.",
    ),
    (
        "virtue.ineslemen",
        "both stated clauses (50 XP on named Abilities, the companion Noncombatant Flaw) are \
         computed via restricted_ability_xp and grants_selection.",
    ),
    (
        "virtue.large",
        "single clause (Size +1), fully computed via size_delta; the Giant Blood/Small Frame/\
         Dwarf exclusion is in incompatible_with.",
    ),
    (
        "virtue.lesser_power",
        "the power-level budget (25) is computed via power_levels, same reading as \
         virtue.greater_power.",
    ),
    (
        "virtue.lightning_reflexes",
        "the operative bonus (+9 Initiative) is computed via combat_mod; the stress-die-plus-\
         Quickness trigger roll is table-time GM adjudication, the same shape botch dice are.",
    ),
    (
        "virtue.linguist",
        "single clause (Language Advancement/XP increased by a quarter), fully computed via \
         group_affinity_cost (counts_as_num/den 5/4).",
    ),
    (
        "virtue.lone_redcap",
        "the base numbers (poor Reputation 2, 300 XP, Well-Traveled grant) are all computed; \
         the Poor/Wealthy season-count interactions are conditional narrative guidance with no \
         number of their own beyond what those Virtues/Flaws already state.",
    ),
    (
        "virtue.major_magical_focus",
        "single clause (double the lowest applicable Art), fully computed via magical_focus \
         (major); the worked Lab/Casting-Total example just illustrates that generic effect.",
    ),
    (
        "virtue.mastered_spells",
        "single clause (50 XP for spell mastery), fully computed via spell_mastery_xp; the \
         Flawless Magic compatibility note needs no effect.",
    ),
    (
        "virtue.masterpiece",
        "single clause (a lesser enchanted item at character generation), fully computed via \
         masterpiece_item; the ignore-vis-costs detail is part of that same mechanism.",
    ),
    (
        "virtue.mentored_by_demons",
        "both stated clauses (50 XP on any Ability, exceeding the age-based cap) are computed \
         via restricted_ability_xp and waives_ability_age_cap.",
    ),
    (
        "virtue.method_caster",
        "single clause (+3 Casting Total for Formulaic/Ritual spells), fully computed via \
         casting_total_mod.",
    ),
    (
        "virtue.personal_power",
        "the power-level budget (25) is computed via power_levels, same reading as \
         virtue.greater_power.",
    ),
    (
        "virtue.rapid_convalescence",
        "single clause (+3 to recover-from-wounds rolls), fully computed via health_mod/\
         recovery.",
    ),
    (
        "virtue.second_sight",
        "single clause (confers the Second Sight Ability at 1), fully computed via \
         ability_score_grant.",
    ),
    (
        "virtue.sense_holiness_and_unholiness",
        "single clause (confers the Sense Holiness and Unholiness Ability at 1), fully \
         computed via ability_score_grant.",
    ),
    (
        "virtue.shadchan",
        "single clause (50 XP on named social Abilities), fully computed via \
         restricted_ability_xp.",
    ),
    (
        "virtue.spirit_votary",
        "the Second Sight grant is computed via grants_selection; the two-Virtue-points-per-\
         Flaw-point ratio is the mythic_companion type profile's own \
         virtue_points_per_flaw_point field, not this entry's job (D46's \"computed on a \
         profile\" shape).",
    ),
    (
        "virtue.study_bonus",
        "both stated bonuses (+2 vis roll, +2 text Quality) are computed via two \
         advancement_mod effects; the Art-Score-to-minimum-Presence table is an in-play \
         gating condition on when the bonus applies, the same D4 shape as flaw.creative_block's \
         \"unless using a Lab Text\".",
    ),
    (
        "virtue.custos",
        "single clause (one restricted Ability group, exclusive choice), fully computed via a \
         gated ability_authorization (Phase 2 C1); \"may not take\" Wealthy/Poor is a separate, \
         unmodelled incompatibility this screen does not recognize as a mechanical token either \
         way.",
    ),
    (
        "virtue.templar_commander",
        "both stated clauses (Reputation 3, the Temporal Influence + Brother-Knight grant) are \
         computed via grants_reputation and grants_selection; the wealth/tax/judge powers are \
         flavor with no number of their own.",
    ),
    (
        "virtue.templar_office_holder",
        "single clause (Reputation of level 2), fully computed via grants_reputation.",
    ),
    (
        "virtue.templar_prestige",
        "single clause (Reputation of level 4), fully computed via grants_reputation.",
    ),
    (
        "virtue.templar_specialist",
        "single clause (one restricted Ability group), fully computed via a gated \
         ability_authorization over the three categories categories_requiring_virtue actually \
         gates (Phase 2 C1's open-set reading of \"such as Academic or Martial\" — see \
         RULES.md).",
    ),
    (
        "virtue.unaging",
        "both stated aging exemptions are computed via two aging_mod effects (no_aging, \
         no_apparent_aging), the same kind already relied on for virtue.bee_king.",
    ),
    (
        "virtue.venditor",
        "single clause (50 XP on named social Abilities), fully computed via \
         restricted_ability_xp.",
    ),
    (
        "virtue.wealthy",
        "the operative clause is computed via later_life_xp_rate (the shipped \
         wealthy_and_poor_ship_with_their_rates_and_eligibility test covers it, mirroring \
         flaw.poor); the displayed text is silent on any token this screen recognizes — a \
         screen vocabulary gap, not a dropped rule.",
    ),
    (
        "virtue.wise_one",
        "single clause (either Arcane or Academic, not both), fully computed via an exclusive-\
         choice-gated ability_authorization (Phase 2 C1, W2/F-349).",
    ),
];

/// D5's first obligation: a `creation_effect`/`in_play_effect` entry whose
/// passage states a mechanical clause this screen newly reaches, where the
/// displayed text (`description` if present, else `summary`, in some shipped
/// locale) states none — so a real clause looks dropped and needs a human
/// read. Unlike [`COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE`], this is not yet
/// resolved: the row only records *that* the screen still finds a gap, not
/// that the gap is real or which text should close it — X2's job, per plan
/// § 1 ("guard stops being class-keyed... reds go to a pending work-list").
/// `(id, what the newly-widened screen found)`; shrink-only, like
/// [`PENDING_MECHANICAL_CLASSIFICATION`] —
/// [`pending_dropped_clause_entries_still_trip_the_screen`] enforces it.
const PENDING_DROPPED_CLAUSE: &[(&str, &str)] = &[
    (
        "flaw.branded_criminal",
        "orphan: \"you may not take the Wealthy Virtue\" carries no incompatible_with — F-340's \
         family, a separate slice (X4), not this row's ability_authorization fix, ArMDE:5749-5752",
    ),
    (
        "flaw.magical_fascination",
        "orphan: \"a score of 1 (but no more)\" cap is D3-inexpressible — no score-cap effect \
         exists; the either/or Lore permission itself is computed via ability_authorization, \
         ArMDE:6392-6395",
    ),
    // virtue.almogavar: resolved (X2a) — its F-340/X4 incompatible_with orphan
    // stays unencoded (a separate slice's job), but X2a's own obligation is a
    // description covering some clause of the passage, so this row shrinks;
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` bites directly.
    // See tmp/x2a-verdicts.md.
    (
        "virtue.mendicant_friar",
        "orphan: \"You may not take the Wealthy Virtue or Poor Flaw\" carries no \
         incompatible_with — F-340's family, a separate slice (X4), not this row's \
         ability_authorization fix, ArMDE:4488-4495",
    ),
    (
        "virtue.perfectus",
        "orphan: \"may not take Wealthy\" carries no incompatible_with (F-340's family, X4), and \
         the conditional \"may... take Purity or Transcendence Abilities\" (only with True \
         Faith) is a second, unencoded permission distinct from the plain Academic one this row \
         adds — a separate finding, ArMDE:4632-4641",
    ),
    (
        "virtue.priest",
        "orphan: \"If you are a parish priest, you cannot take the Poor Flaw\" is a conditional \
         prohibition the engine has no \"is a parish priest\" fact to gate on — distinct from \
         the F-340 blanket Wealthy/Poor exclusion family; the Academic-Ability permission itself \
         is computed via ability_authorization, ArMDE:4800",
    ),
    (
        "virtue.turb_trained",
        "orphan: \"prohibited from being Wealthy or Poor\" carries no incompatible_with (F-340's \
         family, X4); the single-dead-language grant needs a player-chosen `language` parameter \
         (X6/D9p1) rather than custos's fixed-Latin shape — both deferred, ArMDE:5179-5182",
    ),
    (
        "flaw.baneful_circumstances",
        "orphan: \"cannot recover Fatigue, heal wounds, or recover Might\" during the \
         circumstance has no effect at all — only the extra Aging roll is computed, \
         ArMDE:5687-5690",
    ),
    (
        "flaw.bound_to_role_role",
        "orphan: the deprivation-check-as-food clause and \"may only be taken by grogs\" carry \
         no effect/prerequisite — only the Unaging half is computed, ArMDE:5735-5748",
    ),
    // `flaw.corrupted_arts` resolved (Phase 2 C5c, D15): reclassified
    // `uncomputed_rule` with the full passage in `description` in both
    // locales, so it no longer trips this creation_effect/in_play_effect-
    // scoped screen at all — it is now covered by
    // `every_uncomputed_rule_entry_states_its_rule_in_every_locale` instead.
    (
        "flaw.creative_block",
        "orphan: \"roll twice as many dice on the experimentation table\" has no effect — only \
         the -3 Lab Total is computed, ArMDE:5873-5876",
    ),
    (
        "flaw.excommunicate",
        "orphan: \"cannot benefit from the sacraments\" has no effect — only the bad Reputation \
         is computed, ArMDE:6044-6047",
    ),
    (
        "flaw.failed_monk",
        "orphan: \"may take Academic Abilities during character creation\" has no \
         ability_authorization/restricted_ability_xp effect — only the two Reputations are \
         computed (authorization family, X1-adjacent), ArMDE:6068-6071",
    ),
    (
        "flaw.feral_scent",
        "orphan: \"-1 penalty to social interactions\" has no effect — only the Reputation is \
         computed, ArMDE:6106-6109",
    ),
    (
        "flaw.leprosy",
        "orphan: \"cannot gain a positive Reputation\" has no effect — only the Living \
         Condition penalty and the Aging-Crisis Heavy Wound are computed, ArMDE:6338-6341",
    ),
    (
        "flaw.monstrous_blood",
        "orphan: \"may learn Magic Lore during character creation\" (no authorization effect) \
         plus the entire four-way Magic-Animal/Human/Spirit/Thing sub-type system (each with its \
         own penalty or power) — only the -1 Aging-roll base clause is computed, \
         ArMDE:6454-6467",
    ),
    (
        "flaw.obese",
        "orphan: \"-1 to all rolls that involve moving quickly or gracefully\" has no effect — \
         only the -3 Fatigue-roll penalty is computed, ArMDE:6516-6519",
    ),
    (
        "flaw.outlaw",
        "orphan: \"may take Martial Abilities at character generation\" has no \
         ability_authorization effect — only the Reputation is computed (authorization family, \
         X1-adjacent), ArMDE:6542-6545",
    ),
    (
        "flaw.outlaw_leader",
        "orphan: \"may take Martial Abilities at character generation\" has no \
         ability_authorization effect — only the Reputation is computed (authorization family, \
         X1-adjacent), ArMDE:6546-6549",
    ),
    (
        "flaw.savantism",
        "effects: null — the halved starting XP, halved future Advancement Totals, the score-3 \
         starting cap, and the favored-Ability exception (+3 specialization, cap 6) are entirely \
         uncomputed (also D46-pending), ArMDE:6703-6708",
    ),
    (
        "flaw.the_constant_expression",
        "orphan: the permanent lost Fatigue level, the Concentration roll to suppress (Ease \
         Factor 3 + Warping), the extra botch dice on Ceremonial/Ritual casting, and the free \
         -3 lab Safety Flaw all have no effect — only a bare \"circumstantial\" marker is \
         computed, ArMDE:5821-5838",
    ),
    (
        "flaw.usurer",
        "orphan: the ~10 pounds of silver yearly income has no effect — only the Reputation is \
         computed, ArMDE:6951-6954",
    ),
    (
        "flaw.warped_by_magic",
        "orphan: \"may spend experience points on Magic Lore during character creation\" has no \
         ability_authorization effect, and the required companion Minor Flaw is unmodelled — \
         only the Warping grant is computed, ArMDE:7019-7022",
    ),
    (
        "flaw.weak_enchanter",
        "F-544 (`docs/vf-audit/corrections.md` § 3.4): \"apply the Deficiency first and then \
         halve the remaining total\" is an ORDERING rule the engine's single halving effect does \
         not encode, ArMDE:7060-7063",
    ),
    (
        "flaw.weak_magic",
        "F-544: \"halve the Penetration Total after subtracting the spell level\" is an ORDERING \
         rule the engine's single halving effect does not encode, ArMDE:7064-7067",
    ),
    (
        "flaw.weak_scholar",
        "F-544 (worked as one edit with its six siblings, per corrections.md § 3.4) — ships no \
         description in either locale, ArMDE:7080-7083",
    ),
    (
        "flaw.weak_spontaneous_magic",
        "F-544: the stress-die-without-casting-bonus clause and the ceremonial-casting \
         exception have no effect beyond the bare halving, ArMDE:7084-7089",
    ),
    // X2a (tmp/x2a-verdicts.md) resolved 14 rows here, all "class stays, desc
    // owed": virtue.academic_concentration_subject, virtue.affinity_ability,
    // virtue.affinity_art, virtue.arcane_lore, virtue.bee_king,
    // virtue.blood_of_the_nephilim, virtue.cathedral_school_master,
    // virtue.clan_ilfetu, virtue.demonic_blood, virtue.doctor_in_faculty,
    // virtue.enduring_constitution, virtue.faerie_blood, virtue.fast_caster —
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` now bites each
    // directly. It also resolved 4 D20 reclassifications (in_play_effect ->
    // uncomputed_rule, dedicated tests in `x2_reclassification.rs`):
    // virtue.aristotelian_training, virtue.commanding_aura,
    // virtue.diedne_magic, virtue.faerie_raised_magic.
    // virtue.demonic_might: resolved (X2a Phase 2) — `description` now carries
    // the vis-on-death formula ("no more than half" trips the screen) in both
    // locales, closing the gap this row recorded. Removed.
    // --- X2a fix-round additions (2026-09-29): the new "capability"/
    // "prohibition/absolutes"/"eligibility" idioms above newly sweep these
    // two computed entries. Neither is on the X2a worklist (`tmp/x2-worklist.md`
    // § 1 rows 1-51); each is a genuine dropped clause for a later X2 slice.
    // (Their narrative siblings, flaw.simple_minded and
    // flaw.spontaneous_casting_tools/virtue.lesser_purifying_touch, are on
    // NO_RULE_DESPITE_TOKEN/PENDING_MECHANICAL_CLASSIFICATION above instead,
    // since those lists require `narrative`.)
    (
        "flaw.painful_magic",
        "ArMDE:6574-6577's \"do not suffer\" (family: prohibition/absolutes) is the clarifying \
         aside \"though you do not suffer any physical damage from pain\" — it says the Fatigue \
         penalty this Flaw already states (and the summary already carries) never converts to \
         wound damage; `in_play_effect` (a `casting_fatigue_mod`), so this is a computed entry \
         with an orphaned clarifying clause, not a `narrative` reading",
    ),
    (
        "virtue.simple_student",
        "eligibility: \"Female characters can only take this Virtue if they are studying to be \
         physicians at Salerno\", ArMDE:4958-4963 — `creation_effect` (a \
         scaled_restricted_ability_xp effect computes the XP clause), but this gender/location \
         restriction is not computed and reaches the player nowhere",
    ),
    // --- S4 additions (docs/vf-audit/phase-2-plan.md, Phase 1S): SWEPT_BLOCKS
    // widened to the whole catalogue, newly sweeping ArMDE:3951-5638.
    (
        "virtue.inventive_genius",
        "orphan: \"If you experiment, you get +6\" has no effect — only the conditional +3 Lab \
         Total is computed, ArMDE:4151-4154",
    ),
    (
        "virtue.leper_magus",
        "orphan: the whole wound-for-vis mechanic (Light Wound = 3 pawns through Deadly Wound \
         = 15 pawns) has no effect — only the granted Life Boost is computed, ArMDE:4249-4252",
    ),
    (
        "virtue.life_boost",
        "orphan: the self-damage-if-over-Fatigue clause (Soak 5 x extra levels + stress die) \
         has no effect — only the +5-per-Fatigue-level casting bonus is computed, \
         ArMDE:4295-4298",
    ),
    (
        "virtue.magian_lineage_major",
        "orphan: the Major half's connected-Abilities XP-sharing mechanic has no effect — only \
         the shared -1 Aging-roll base clause is computed, ArMDE:4339-4346",
    ),
    (
        "virtue.magian_lineage_minor",
        "orphan: the Minor half's own \"+3 bonus to resist the effects of disease\" has no \
         effect — only the shared -1 Aging-roll base clause is computed, ArMDE:4339-4346",
    ),
    (
        "virtue.magic_items",
        "orphan: \"the rate at which your items are improved is increased by one level per \
         year\" has no effect — only the starting +25 item-level budget is computed, \
         ArMDE:4347-4350",
    ),
    (
        "virtue.magic_sensitivity",
        "orphan: \"subtract your Magic Sensitivity score from your Magic Resistance\" has no \
         effect — only the Ability grant is computed, ArMDE:4351-4354",
    ),
    (
        "virtue.magister_in_artibus",
        "orphan: the age-and-Ability-score eligibility floor (25-Int years, Latin/Artes \
         Liberales 5) has no prerequisite — only the Reputation and the XP grant are computed, \
         ArMDE:4385-4394",
    ),
    (
        "virtue.marshal",
        "orphan: \"may take Martial Abilities freely\" has no ability_authorization effect — \
         only the 50 XP grant is computed (authorization family, X1-adjacent), \
         ArMDE:4449-4456",
    ),
    (
        "virtue.master_of_kennels",
        "orphan: \"may take Martial Abilities freely\" has no ability_authorization effect — \
         only the 50 XP grant is computed (authorization family, X1-adjacent), \
         ArMDE:4467-4470",
    ),
    (
        "virtue.mercurian_magic",
        "orphan: the Wizard's Vigil auto-knowledge, the Mastery-score stacking, and the \
         required companion Flaw (Ceremonial Spontaneous Magic, no grants_selection) all have \
         no effect — only a bare \"mercurian\" marker is computed, ArMDE:4514-4523",
    ),
    (
        "virtue.mythic_blood",
        "orphan: the potent-Gift Fatigue-loss reduction, the invokable magic-feat table \
         (level+Penetration by gesture/speech), and the hereditary Personality Flaw grant all \
         have no effect — only the included Minor Magical Focus is computed, ArMDE:4573-4589",
    ),
    (
        "virtue.potent_magic_major",
        "orphan: the whole Potent-Spells subsystem (Potency Score from Casting Items, capped \
         by Magic Theory) has no effect — only the flat +6 Lab/Casting bonus is computed, \
         ArMDE:4740-4781",
    ),
    (
        "virtue.potent_magic_minor",
        "orphan: the same Potent-Spells subsystem as virtue.potent_magic_major — only the flat \
         +3 Lab/Casting bonus is computed, ArMDE:4740-4781",
    ),
    (
        "virtue.redcap",
        "orphan: the per-year item-growth rate, the free Well-Traveled grant (no \
         grants_selection), and the free Longevity Ritual when aging starts all have no effect \
         — only the starting +50 item-level budget is computed, ArMDE:4842-4851",
    ),
    (
        "virtue.ritual_power",
        "orphan: \"you must spend one Confidence Point for every magnitude of the effect\" has \
         no effect — only the power-level budget (25) is computed, ArMDE:4870-4877",
    ),
    (
        "virtue.rosh_beth_din",
        "orphan: the age-and-Ability-score eligibility floor (30-Int years, Hebrew/Rabbinic \
         Law/Theology: Judaism 5) has no prerequisite — only the Reputation, XP grant and \
         Social Contacts grant are computed, ArMDE:4878-4883",
    ),
    (
        "virtue.senior_bard",
        "orphan: the minimum-age-22 and score-5-in-a-named-Lore eligibility floor has no \
         prerequisite — only the Reputation and the XP grant are computed, ArMDE:4904-4909",
    ),
    (
        "virtue.senior_clergy",
        "orphan: \"may purchase Academic Abilities\" has no ability_authorization effect — only \
         the two Reputations are computed (authorization family, X1-adjacent), \
         ArMDE:4910-4921",
    ),
    (
        "virtue.strong_angelic_heritage",
        "orphan: the age-scaled Divine Might formula (age / 20), the vis-on-death yield, and \
         Warping immunity are not visibly keyed to age or otherwise computed beyond a flat \
         might_grant score of 0, ArMDE:5022-5031",
    ),
    (
        "virtue.strong_faerie_blood",
        "orphan: the age-50 aging-roll start (vs. normal 35), \"see normally in total \
         darkness\", \"may learn Faerie Lore\", and the entire inherited Faerie Blood sub-type \
         bonus list all have no effect — only the Second Sight grant and the -3 Aging-roll are \
         computed, ArMDE:5032-5047",
    ),
];

/// True when **every** shipped locale's displayed rules text for `id`
/// (`description` if present, else `summary` —
/// [`displayed_rules_text_by_language`], the exact precedence
/// `VirtueFlawTab.svelte`'s tooltip applies) states a mechanical rule. A
/// locale carrying no displayed text at all counts as failing: there is
/// nothing there to carry the clause. This is deliberately **not** "the same
/// clause the passage names" — the screen has no way to match a specific
/// token to a specific `Effect`, only to ask whether *some* mechanical rule
/// reached the player in text. That coarseness is what makes
/// [`PENDING_DROPPED_CLAUSE`] a to-be-read work-list rather than a settled
/// verdict.
fn displayed_text_states_a_mechanical_rule_in_every_locale(
    id: &str,
    by_language: &BTreeMap<String, BTreeMap<String, String>>,
) -> bool {
    by_language.values().all(|displayed| {
        displayed
            .get(id)
            .is_some_and(|text| states_a_mechanical_rule(text))
    })
}

/// The `source` block of a catalogue entry, as `(file, start, end)`.
fn source_of(item: &Value) -> Option<(String, i64, i64)> {
    let source = item.get("source")?;
    let file = source.get("file")?.as_str()?.to_string();
    let lines = source.get("lines")?.as_array()?;
    let [start, end] = lines.as_slice() else {
        return None;
    };
    Some((file, start.as_i64()?, end.as_i64()?))
}

/// True when this citation lies wholly inside a swept block.
fn is_swept(file: &str, start: i64, end: i64) -> bool {
    SWEPT_BLOCKS
        .iter()
        .any(|(swept_file, lo, hi)| *swept_file == file && start >= *lo && end <= *hi)
}

/// S4 (`docs/vf-audit/phase-2-plan.md`, Phase 1S): the sweep's whole point is
/// that every passage has actually been read, so every catalogue entry's
/// citation must lie inside *some* [`SWEPT_BLOCKS`] range — not just the
/// entries [`no_swept_entry_drops_an_uncomputed_mechanical_clause`] happens to
/// scope in today. An unswept entry is a hole in that guarantee: nothing says
/// which entries it actually covers.
#[test]
fn every_vf_entry_lies_inside_a_swept_block() {
    let mut unswept = Vec::new();
    for item in catalogue() {
        let Some((file, start, end)) = source_of(&item) else {
            continue;
        };
        if !is_swept(&file, start, end) {
            let id = item["id"].as_str().expect("every entry has a string id");
            unswept.push(format!("{id} ({file}:{start}-{end})"));
        }
    }
    assert!(
        unswept.is_empty(),
        "{} Virtue/Flaw(s) cite a passage outside every SWEPT_BLOCKS range, so the sweep has \
         not actually read them:\n{}",
        unswept.len(),
        unswept.join("\n")
    );
}

/// The English passage a citation brackets, reading each source file at most
/// once.
fn bracketed_passage(
    cache: &mut BTreeMap<String, Vec<String>>,
    file: &str,
    start: i64,
    end: i64,
) -> Option<String> {
    let lines = cache.entry(file.to_string()).or_insert_with(|| {
        let path = rules_dir().join("source/en").join(file);
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()))
            .lines()
            .map(str::to_string)
            .collect()
    });
    if start < 1 || end < start || end as usize > lines.len() {
        // Out-of-bounds ranges are `rules_source_provenance.rs`'s finding.
        return None;
    }
    Some(lines[(start as usize - 1)..(end as usize)].join("\n"))
}

/// The third assertion, D5's first obligation (`docs/vf-audit/decisions.md`):
/// **the guard stops being class-keyed.** It used to ask only "is this
/// `narrative`, and does its passage state a rule?" — so a `creation_effect` or
/// `in_play_effect` entry that computes *one* clause and drops the rest of its
/// passage was examined by nothing at all (§ 3.1's second blind spot;
/// F-461/F-465/F-470). It now asks the question D5 actually poses, of all three
/// classes a swept passage can carry: **does this passage state a mechanical
/// clause that reaches the player nowhere?**
///
/// - `narrative` means "pure personality, story, or social-status flavor" — a
///   statement about the *rulebook*, not merely about what the engine
///   computes. A passage that trips the screen is a dropped rule or a
///   misclassification, exactly as before.
/// - `creation_effect`/`in_play_effect` entries compute *something*, but this
///   screen cannot match a specific token to a specific `Effect` — it can only
///   ask whether the *displayed* text (every shipped locale) also states a
///   mechanical rule. If it does not, the passage's clause has no carrier at
///   all: neither an effect provably covering it, nor a sentence stating it.
///   That is exactly D5's "the obligation is that the clause reaches the
///   player" — a `description` in every locale, or (recorded in
///   [`COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE`]) an explicit reading that the
///   effects already cover the whole passage.
/// - `uncomputed_rule` is deliberately excluded here: [`states_a_mechanical_rule`]
///   already gates it, unconditionally and catalogue-wide, in
///   [`every_uncomputed_rule_entry_states_its_rule_in_every_locale`].
///
/// This is the assertion that makes the sweep *checkable* rather than claimed.
/// Without it, "the Flaws block is clean" is a sentence in a report; with it,
/// re-dirtying the block is a failing build.
#[test]
fn no_swept_entry_drops_an_uncomputed_mechanical_clause() {
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let by_language = displayed_rules_text_by_language();
    let mut checked = 0usize;
    let mut offenders = Vec::new();

    for item in catalogue() {
        let classification = item["classification"].as_str().unwrap_or_default();
        if !matches!(
            classification,
            "narrative" | "creation_effect" | "in_play_effect"
        ) {
            continue;
        }
        let Some((file, start, end)) = source_of(&item) else {
            continue;
        };
        if !is_swept(&file, start, end) {
            continue;
        }
        checked += 1;
        let id = item["id"].as_str().expect("every entry has a string id");
        if NO_RULE_DESPITE_TOKEN
            .iter()
            .any(|(exempt, _)| *exempt == id)
        {
            continue;
        }
        if PENDING_MECHANICAL_CLASSIFICATION
            .iter()
            .any(|(pending, _)| *pending == id)
        {
            continue;
        }
        if COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE
            .iter()
            .any(|(allowed, _)| *allowed == id)
        {
            continue;
        }
        if PENDING_DROPPED_CLAUSE
            .iter()
            .any(|(pending, _)| *pending == id)
        {
            continue;
        }
        let Some(passage) = bracketed_passage(&mut cache, &file, start, end) else {
            continue;
        };
        if !states_a_mechanical_rule(&passage) {
            continue;
        }

        if classification == "narrative" {
            offenders.push(format!(
                "{id} ({file}:{start}-{end}): narrative, but the passage states a rule"
            ));
            continue;
        }

        if !displayed_text_states_a_mechanical_rule_in_every_locale(id, &by_language) {
            offenders.push(format!(
                "{id} ({file}:{start}-{end}): {classification}, but its displayed text states \
                 no mechanical rule in at least one shipped locale — D5's first obligation"
            ));
        }
    }

    assert!(
        checked > 50,
        "the swept-block filter matched only {checked} entries, so this guard is checking \
         almost nothing — has a SWEPT_BLOCKS range or a source file name drifted?"
    );

    assert!(
        offenders.is_empty(),
        "{} Virtue/Flaw(s) inside a swept block cite a passage that states a mechanical rule \
         (a signed modifier, a botch-dice clause, or a phrase-shaped idiom — a cap, a target \
         number, a formula, a rounding direction, an absolute) with no carrier reaching the \
         player: a `narrative` entry has dropped the rule outright or is misclassified; a \
         `creation_effect`/`in_play_effect` entry computes something but its displayed text \
         (D5) does not state the clause in every locale:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

/// Every [`NO_RULE_DESPITE_TOKEN`] row exists to silence the screen for one
/// entry, so every row must still *be* silencing something. If one stops
/// tripping the screen — because the citation was repointed, or the detector
/// narrowed — the row has become a stale excuse sitting on top of a check that
/// no longer needs it. Fail then, so it gets deleted rather than outliving its
/// reason. The mirror of `rules_source_provenance.rs::known_misencodings_still_fail_the_guard`.
#[test]
fn exempted_entries_still_trip_the_screen() {
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (id, _) in NO_RULE_DESPITE_TOKEN {
        let item = catalogue()
            .into_iter()
            .find(|item| item["id"] == *id)
            .unwrap_or_else(|| {
                panic!(
                    "NO_RULE_DESPITE_TOKEN row \"{id}\" names an entry that is no longer in the \
                     catalogue — delete the row"
                )
            });

        assert_eq!(
            item["classification"], "narrative",
            "NO_RULE_DESPITE_TOKEN row \"{id}\" is no longer `narrative`, so the guard it \
             silences does not look at it — delete the row"
        );

        let (file, start, end) =
            source_of(&item).unwrap_or_else(|| panic!("\"{id}\" has a source block"));
        assert!(
            is_swept(&file, start, end),
            "NO_RULE_DESPITE_TOKEN row \"{id}\" cites {file}:{start}-{end}, outside every swept \
             block — the guard it silences does not reach it, so delete the row"
        );

        let passage = bracketed_passage(&mut cache, &file, start, end)
            .unwrap_or_else(|| panic!("\"{id}\" cites an in-bounds range"));
        assert!(
            states_a_mechanical_rule(&passage),
            "NO_RULE_DESPITE_TOKEN row \"{id}\" no longer trips the mechanical-token screen, so \
             it is silencing nothing — delete the row"
        );
    }
}

/// The mirror of [`exempted_entries_still_trip_the_screen`], for
/// [`PENDING_MECHANICAL_CLASSIFICATION`] rather than [`NO_RULE_DESPITE_TOKEN`]:
/// every pending row must still trip the screen, so the list can only shrink
/// as X2 reclassifies entries — never grow stale.
#[test]
fn pending_mechanical_classification_entries_still_trip_the_screen() {
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (id, _) in PENDING_MECHANICAL_CLASSIFICATION {
        let item = catalogue()
            .into_iter()
            .find(|item| item["id"] == *id)
            .unwrap_or_else(|| {
                panic!(
                    "PENDING_MECHANICAL_CLASSIFICATION row \"{id}\" names an entry that is no \
                     longer in the catalogue — delete the row"
                )
            });

        assert_eq!(
            item["classification"], "narrative",
            "PENDING_MECHANICAL_CLASSIFICATION row \"{id}\" is no longer `narrative` — X2 has \
             already reclassified it, so delete the row"
        );

        let (file, start, end) =
            source_of(&item).unwrap_or_else(|| panic!("\"{id}\" has a source block"));
        assert!(
            is_swept(&file, start, end),
            "PENDING_MECHANICAL_CLASSIFICATION row \"{id}\" cites {file}:{start}-{end}, outside \
             every swept block — the guard it works around does not reach it, so delete the row"
        );

        let passage = bracketed_passage(&mut cache, &file, start, end)
            .unwrap_or_else(|| panic!("\"{id}\" cites an in-bounds range"));
        assert!(
            states_a_mechanical_rule(&passage),
            "PENDING_MECHANICAL_CLASSIFICATION row \"{id}\" no longer trips the mechanical-token \
             screen — nothing needs it to wait any more, so delete the row"
        );
    }
}

/// Every [`COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE`] row exists to silence
/// [`no_swept_entry_drops_an_uncomputed_mechanical_clause`] for one
/// `creation_effect`/`in_play_effect` entry, so every row must still *be*
/// silencing something. The mirror of `exempted_entries_still_trip_the_screen`,
/// for the computed-class allow-list rather than the `narrative` one.
#[test]
fn computed_entry_covers_whole_passage_entries_still_trip_the_screen() {
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (id, _) in COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE {
        let item = catalogue()
            .into_iter()
            .find(|item| item["id"] == *id)
            .unwrap_or_else(|| {
                panic!(
                    "COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE row \"{id}\" names an entry that is no \
                     longer in the catalogue — delete the row"
                )
            });

        let classification = item["classification"].as_str().unwrap_or_default();
        assert!(
            matches!(classification, "creation_effect" | "in_play_effect"),
            "COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE row \"{id}\" is classified {classification:?}, \
             not creation_effect/in_play_effect — the guard it silences does not look at it, so \
             delete the row"
        );

        let (file, start, end) =
            source_of(&item).unwrap_or_else(|| panic!("\"{id}\" has a source block"));
        assert!(
            is_swept(&file, start, end),
            "COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE row \"{id}\" cites {file}:{start}-{end}, \
             outside every swept block — the guard it silences does not reach it, so delete \
             the row"
        );

        let passage = bracketed_passage(&mut cache, &file, start, end)
            .unwrap_or_else(|| panic!("\"{id}\" cites an in-bounds range"));
        assert!(
            states_a_mechanical_rule(&passage),
            "COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE row \"{id}\" no longer trips the mechanical-\
             token screen on its passage, so it is silencing nothing — delete the row"
        );
    }
}

/// The mirror of [`pending_mechanical_classification_entries_still_trip_the_screen`],
/// for [`PENDING_DROPPED_CLAUSE`]: every pending row must still trip
/// [`no_swept_entry_drops_an_uncomputed_mechanical_clause`], so the list can
/// only shrink as X2 writes the missing `description` (or, in F-544's case,
/// resolves the ordering rule) — never grow stale.
#[test]
fn pending_dropped_clause_entries_still_trip_the_screen() {
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let by_language = displayed_rules_text_by_language();

    for (id, _) in PENDING_DROPPED_CLAUSE {
        let item = catalogue()
            .into_iter()
            .find(|item| item["id"] == *id)
            .unwrap_or_else(|| {
                panic!(
                    "PENDING_DROPPED_CLAUSE row \"{id}\" names an entry that is no longer in \
                     the catalogue — delete the row"
                )
            });

        let classification = item["classification"].as_str().unwrap_or_default();
        assert!(
            matches!(classification, "creation_effect" | "in_play_effect"),
            "PENDING_DROPPED_CLAUSE row \"{id}\" is classified {classification:?}, not \
             creation_effect/in_play_effect — X2 has already reclassified it, so delete the row"
        );

        let (file, start, end) =
            source_of(&item).unwrap_or_else(|| panic!("\"{id}\" has a source block"));
        assert!(
            is_swept(&file, start, end),
            "PENDING_DROPPED_CLAUSE row \"{id}\" cites {file}:{start}-{end}, outside every swept \
             block — the guard it works around does not reach it, so delete the row"
        );

        let passage = bracketed_passage(&mut cache, &file, start, end)
            .unwrap_or_else(|| panic!("\"{id}\" cites an in-bounds range"));
        assert!(
            states_a_mechanical_rule(&passage),
            "PENDING_DROPPED_CLAUSE row \"{id}\" no longer trips the mechanical-token screen on \
             its passage — nothing needs it to wait any more, so delete the row"
        );
        assert!(
            !displayed_text_states_a_mechanical_rule_in_every_locale(id, &by_language),
            "PENDING_DROPPED_CLAUSE row \"{id}\"'s displayed text now states a mechanical rule \
             in every locale — the gap it recorded has been closed, so delete the row"
        );
    }
}

#[test]
fn the_mechanical_token_detector_reads_real_clauses_and_ignores_near_misses() {
    // Signed numbers, in both locales' phrasing.
    assert!(states_a_mechanical_rule(
        "you are at -3 in all related rolls"
    ));
    assert!(states_a_mechanical_rule("You gain +1 to all rolls"));
    assert!(states_a_mechanical_rule(
        "erleidest du -1 auf Angriffswürfe"
    ));

    // Botch terms, in both locales.
    assert!(states_a_mechanical_rule("roll an extra botch die"));
    assert!(states_a_mechanical_rule("two fewer botch dice"));
    assert!(states_a_mechanical_rule(
        "resist Twilight on a single magical botch"
    ));
    assert!(states_a_mechanical_rule("einen zusätzlichen Patzerwürfel"));

    // The German VERB forms of the same mechanic. English is covered by a stem —
    // "botch" already reads "botches", "botching", "botched" — while the German
    // side listed only the noun, so a passage saying the character *botches*
    // rather than *suffers a botch* read as pure flavour. The books write both:
    // "Das Patzen bei einem dieser Würfe führt zu falschen Informationen" (Eye
    // of Hephaestus) and "Einige Stresswürfe können nicht patzen" (the stress-die
    // rules). The asymmetry is the detector's defect, not the text's — the
    // vocabulary is a property of the rulebooks' language, so the term list is
    // what has to match the books rather than the other way round.
    assert!(states_a_mechanical_rule(
        "Das Patzen bei einem dieser Würfe führt zu falschen Informationen."
    ));
    assert!(states_a_mechanical_rule(
        "Einige Stresswürfe können nicht patzen"
    ));
    assert!(states_a_mechanical_rule("Wenn er patzt, fällt er"));
    assert!(states_a_mechanical_rule(
        "wenn einer davon eine Null zeigt, hat man gepatzt"
    ));

    // The adjective shares the stem and states nothing — which is why the terms
    // below are the inflected verb forms rather than a bare "patz" stem.
    assert!(!states_a_mechanical_rule("eine patzige Antwort"));

    // An unsigned number is not a modifier — this is the exclusion that keeps
    // the signed-number token sharp. Its former companion, "an Ease Factor of
    // 9", was asserted here as a non-rule and is now a rule the phrase screen
    // reads: a target number is mechanical, it just is not a *signed modifier*.
    assert!(!states_a_mechanical_rule("1 pawn of vis each season"));

    // A hyphen that is not a sign, and prose with no mechanics at all.
    assert!(!states_a_mechanical_rule("good hand-eye coordination"));
    assert!(!states_a_mechanical_rule("Roleplay your clumsiness."));
    assert!(!states_a_mechanical_rule(
        "You are a full member of the Order of Hermes."
    ));

    // --- The phrase screen -------------------------------------------------
    //
    // Caps and floors. Source: ArMDE:3577 (Command Animals), :6925 (University
    // Dean), :5733 (Bound to (Realm)), :6981 (Viaticarus).
    assert!(states_a_mechanical_rule(
        "the character may command up to 12 human-sized animals"
    ));
    assert!(states_a_mechanical_rule(
        "kann der Charakter bis zu 12 menschengroße Tiere befehligen"
    ));
    assert!(states_a_mechanical_rule("be at least 40 years old"));
    assert!(states_a_mechanical_rule("mindestens 40 Jahre alt sein"));
    assert!(states_a_mechanical_rule(
        "must live in a supernatural aura of 5 or greater"
    ));
    assert!(states_a_mechanical_rule(
        "muss in einer übernatürlichen Aura von 5 oder höher leben"
    ));
    assert!(states_a_mechanical_rule("a (Realm) Lore of 1 or more"));
    assert!(states_a_mechanical_rule(
        "einer (Sphären-)Kunde von 1 oder mehr"
    ));
    assert!(states_a_mechanical_rule(
        "his Presence and Communication may not be greater than 0"
    ));
    assert!(states_a_mechanical_rule(
        "it would be no more than an interesting feature"
    ));

    // Target numbers. Source: ArMDE:3777 (Exotic Casting), :6785 (Stigmatic
    // Catalyst).
    assert!(states_a_mechanical_rule(
        "is made against an Ease Factor of 15"
    ));
    assert!(states_a_mechanical_rule(
        "würfeln gegen einen Schwierigkeitsgrad von 6"
    ));

    // Formulas. Source: ArMDE:3781 (Extractor of (Form) Vis), :3919 (Folk
    // Magic), :3757 (Enduring Magic).
    assert!(states_a_mechanical_rule(
        "a number of pawns of Vis equal to a tenth of his Creo Vim Lab Total"
    ));
    assert!(states_a_mechanical_rule(
        "Die Zaubersumme entspricht (Ausdauer + (Sphären-)Kunde) / 2"
    ));
    assert!(states_a_mechanical_rule(
        "multiply the spell's normal duration by the number rolled"
    ));
    assert!(states_a_mechanical_rule(
        "multipliziere die normale Dauer des Zaubers mit dem gewürfelten Wert"
    ));

    // Rounding, which changes the answer and so is as mechanical as a modifier.
    // Source: ArMDE:3781; the four English spellings and two German ones are all
    // in the books.
    assert!(states_a_mechanical_rule("Lab Total (round up)"));
    assert!(states_a_mechanical_rule("half the total, rounded down"));
    assert!(states_a_mechanical_rule(
        "seiner Laborsumme entspricht (aufgerundet)"
    ));
    assert!(states_a_mechanical_rule("die Hälfte des Werts, abgerundet"));

    // Dice named by kind. Source: ArMDE:3757 (Enduring Magic), :6078 (The
    // Falling Evil).
    assert!(states_a_mechanical_rule(
        "the storyguide secretly rolls a simple die"
    ));
    assert!(states_a_mechanical_rule(
        "Der Spielleiter würfelt heimlich einen einfachen Würfel"
    ));
    assert!(states_a_mechanical_rule(
        "the storyguide should secretly roll a stress die"
    ));
    assert!(states_a_mechanical_rule(
        "sollte der Spielleiter im Geheimen einen Stresswürfel werfen"
    ));

    // Absolutes — a rule stated as a prohibition carries no number at all.
    // Source: ArMDE:3641 (Death Prophecy). The German clause is the *earlier*
    // sentence of the same passage on purpose: German negation is discontinuous
    // ("kannst aber nicht an Wunden oder Alter sterben"), so the fixed literal
    // "nicht sterben" cannot match it, and the spelling the list can carry is
    // the one the books also write plainly. The screen now matches by regex
    // (D19, `docs/vf-audit/decisions.md`), which *can* express the
    // discontinuous form via a bounded-gap pattern — but adding it is a new
    // phrase family (slice S2), not something this literal's conversion
    // grows on its own. See the note on `MECHANICAL_PHRASES`.
    assert!(states_a_mechanical_rule(
        "You heal normally, but cannot die as a result of wounds or old age."
    ));
    assert!(states_a_mechanical_rule(
        "bis diese Bedingung erfüllt ist, wirst du nicht sterben"
    ));

    // A step counted in magnitudes. Source: ArMDE:3893 (Flexible Formulaic
    // Magic).
    assert!(states_a_mechanical_rule(
        "raise or lower the casting level of the spell by one magnitude"
    ));
    assert!(states_a_mechanical_rule(
        "Du kannst die Zauberstufe um eine Magnitude anheben oder senken"
    ));

    // --- What the phrase screen must NOT read as a rule --------------------
    //
    // The screen is deliberately noisier than the other two, so its noise is
    // bounded here rather than left to hope. A phrase is matched as the *whole*
    // idiom: the bare noun or preposition inside it states nothing.
    assert!(!states_a_mechanical_rule(
        "The magnitude of his ambition is hard to overstate."
    ));
    assert!(!states_a_mechanical_rule(
        "Er ist mehr Gelehrter als Krieger."
    ));
    assert!(!states_a_mechanical_rule(
        "His workshop is up the road from the covenant."
    ));
    assert!(!states_a_mechanical_rule("Sie würfelt gern mit den Grogs."));
    assert!(!states_a_mechanical_rule(
        "She rounds on anyone who questions her."
    ));

    // A phrase must begin at a word boundary. Without that, "or more" reads
    // itself out of "f|or more| details" and flags every entry that points at a
    // supplement — which is how virtue.factor at ArMDE:3795 and virtue.fidai at
    // ArMDE:3881 first appeared in the sweep. A substring crossing a word boundary
    // is a defect in the screen, not an idiom to argue down in a
    // `NO_RULE_DESPITE_TOKEN` row.
    assert!(!states_a_mechanical_rule(
        "see City and Guild for more details"
    ));
    assert!(!states_a_mechanical_rule(
        "from page 162, for more detail on the Nizaris"
    ));
    assert!(!states_a_mechanical_rule(
        "His talents are unequal to the task."
    ));

    // Plain flavour, in both locales, tripping nothing.
    assert!(!states_a_mechanical_rule(
        "The character is a member of the lesser nobility."
    ));
    assert!(!states_a_mechanical_rule(
        "Der Charakter ist ein Mitglied des niederen Adels."
    ));
}

/// D19 (`docs/vf-audit/decisions.md`), slice S1: `(narrative_in_swept_blocks,
/// uncomputed_rule_by_locale)` — the two maps both
/// [`print_s1_before_offender_set`] and
/// [`regex_screen_never_loses_an_s1_recorded_flag`] need, computed against
/// whichever [`states_a_mechanical_rule`] is compiled in right now. Run
/// against the pre-conversion substring matcher, this produced the committed
/// `tests/fixtures/s1_before_offenders.json`; run against the regex matcher
/// (S1) or the regex matcher plus S2's added families, every entry that
/// fixture recorded `true` must still be `true`.
fn compute_s1_offender_set() -> (BTreeMap<String, bool>, BTreeMap<String, bool>) {
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut narrative_in_swept_blocks = BTreeMap::new();

    for item in catalogue() {
        if item["classification"] != "narrative" {
            continue;
        }
        let Some((file, start, end)) = source_of(&item) else {
            continue;
        };
        if !is_swept(&file, start, end) {
            continue;
        }
        let id = item["id"]
            .as_str()
            .expect("every entry has a string id")
            .to_string();
        let Some(passage) = bracketed_passage(&mut cache, &file, start, end) else {
            continue;
        };
        narrative_in_swept_blocks.insert(id, states_a_mechanical_rule(&passage));
    }

    let mut uncomputed_rule_by_locale = BTreeMap::new();
    let ids = ids_classified("uncomputed_rule");
    let by_language = displayed_rules_text_by_language();
    for (lang, displayed) in &by_language {
        for id in &ids {
            if let Some(text) = displayed.get(id) {
                uncomputed_rule_by_locale
                    .insert(format!("{lang}/{id}"), states_a_mechanical_rule(text));
            }
        }
    }

    (narrative_in_swept_blocks, uncomputed_rule_by_locale)
}

/// D19 slice S1: the exact before-conversion offender set, computed with the
/// **substring** matcher this slice replaces with regex. Committed as
/// `tests/fixtures/s1_before_offenders.json` so the regex conversion can be
/// proved inert against real shipped data — see
/// [`regex_screen_never_loses_an_s1_recorded_flag`] below. This is a
/// one-shot snapshot helper, not an assertion: it is `#[ignore]`d so it never
/// runs as part of the suite, and its only job is to reproduce the committed
/// fixture by hand if the catalogue ever needs re-snapshotting (it must not,
/// under ordinary use — the fixture is what S1's inert test checks against).
#[test]
#[ignore = "one-shot snapshot for D19 S1; run with `-- --ignored --nocapture` \
            to regenerate the printed set, which must match \
            tests/fixtures/s1_before_offenders.json"]
fn print_s1_before_offender_set() {
    let (narrative_in_swept_blocks, uncomputed_rule_by_locale) = compute_s1_offender_set();

    println!(
        "narrative_in_swept_blocks ({} entries):",
        narrative_in_swept_blocks.len()
    );
    for (id, flagged) in &narrative_in_swept_blocks {
        println!("{flagged}\t{id}");
    }
    println!(
        "uncomputed_rule_by_locale ({} entries):",
        uncomputed_rule_by_locale.len()
    );
    for (key, flagged) in &uncomputed_rule_by_locale {
        println!("{flagged}\t{key}");
    }
}

/// The recorded before-set from `tests/fixtures/s1_before_offenders.json`, as
/// `(narrative_in_swept_blocks, uncomputed_rule_by_locale)` — the same shape
/// [`compute_s1_offender_set`] returns, so the two are directly comparable.
fn recorded_s1_before_set() -> (BTreeMap<String, bool>, BTreeMap<String, bool>) {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/s1_before_offenders.json");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
    let value: Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{} is valid JSON: {e}", path.display()));

    let read_bool_map = |key: &str| -> BTreeMap<String, bool> {
        value[key]
            .as_object()
            .unwrap_or_else(|| panic!("{key} is a JSON object in {}", path.display()))
            .iter()
            .map(|(id, flag)| {
                let flag = flag
                    .as_bool()
                    .unwrap_or_else(|| panic!("{key}/{id} is a boolean in {}", path.display()));
                (id.clone(), flag)
            })
            .collect()
    };

    (
        read_bool_map("narrative_in_swept_blocks"),
        read_bool_map("uncomputed_rule_by_locale"),
    )
}

/// D19's obligation #1 (`docs/vf-audit/decisions.md`): **immediately after
/// the regex conversion landed (S1), and before any new family was added**,
/// this test asserted exact equality with `s1_before_offenders.json` — the
/// conversion had to flag *exactly* the substring matcher's set, proving the
/// mechanical change alone (contiguous substring → regex) altered nothing.
/// That proof is done and frozen in the S1 commit.
///
/// S2 (`docs/vf-audit/corrections.md` § 3.1) deliberately widens the screen
/// with new phrase families, so an exact-equality assertion would go red on
/// this slice's very first new family — correctly, but for a property this
/// module no longer wants: the screen is supposed to widen as the sweep
/// proceeds (see the module doc comment). Re-pinning an exact snapshot every
/// time a family is added would make the fixture a chore, not a guard.
///
/// What stays worth guarding is the **one-directional** half: nothing that
/// used to trip the screen may silently stop tripping it. That is a real
/// regression — a family rewritten to be narrower, a boundary bug — and this
/// is its guard: every entry [`recorded_s1_before_set`] marked `true` must
/// still be `true` today. An entry recorded `false` carries no such
/// obligation; S2 (and later slices) are free to flip those to `true`, and
/// are expected to.
#[test]
fn regex_screen_never_loses_an_s1_recorded_flag() {
    let (expected_narrative, expected_uncomputed) = recorded_s1_before_set();
    let (actual_narrative, actual_uncomputed) = compute_s1_offender_set();

    let mut regressions = Vec::new();

    for (id, expected) in expected_narrative.iter().filter(|(_, flagged)| **flagged) {
        match actual_narrative.get(id) {
            None => regressions.push(format!(
                "narrative_in_swept_blocks/{id}: recorded true but missing from today's catalogue"
            )),
            Some(actual) if actual != expected => regressions.push(format!(
                "narrative_in_swept_blocks/{id}: recorded true, now {actual}"
            )),
            Some(_) => {}
        }
    }

    for (key, expected) in expected_uncomputed.iter().filter(|(_, flagged)| **flagged) {
        match actual_uncomputed.get(key) {
            None => regressions.push(format!(
                "uncomputed_rule_by_locale/{key}: recorded true but missing from today's catalogue"
            )),
            Some(actual) if actual != expected => regressions.push(format!(
                "uncomputed_rule_by_locale/{key}: recorded true, now {actual}"
            )),
            Some(_) => {}
        }
    }

    assert!(
        regressions.is_empty(),
        "the screen must never silently STOP recognizing something it recognized at S1 \
         (tests/fixtures/s1_before_offenders.json) — widening (S2 and later) is fine and \
         expected, narrowing is a regression:\n{}",
        regressions.join("\n")
    );
}

/// D19 obligation 3 (`docs/vf-audit/decisions.md`) and F-537: every
/// [`S2_IDIOMS`] pattern must actually match something the rulebook in its
/// own tagged [`Language`] writes — not a plausible rendering of the other
/// locale, and not nothing at all. F-537 found a German needle already in
/// `MECHANICAL_PHRASES` (S1's 33) that matched zero real lines; this test is
/// what stops S2's additions from repeating that. It runs the *exact* compiled
/// pattern the detector uses, against the *whole* source file, so passing here
/// is not merely plausible — it is a real, reproducible hit.
#[test]
fn every_s2_idiom_has_a_real_hit_in_its_own_language() {
    let en_path = rules_dir()
        .join("source/en")
        .join("Ars Magica - Definitive Edition (Core Rules).md");
    let de_path = rules_dir()
        .join("source/de")
        .join("Ars Magica Definitive Edition Basisregeln.md");
    let en_text = fs::read_to_string(&en_path)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", en_path.display()));
    let de_text = fs::read_to_string(&de_path)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", de_path.display()));

    assert_eq!(
        S2_IDIOMS.len(),
        S2_IDIOM_PATTERNS.len(),
        "S2_IDIOMS and S2_IDIOM_PATTERNS have drifted apart in length"
    );

    let mut inert = Vec::new();
    for (idiom, pattern) in S2_IDIOMS.iter().zip(S2_IDIOM_PATTERNS.iter()) {
        let (text, lang_name) = match idiom.language {
            Language::En => (&en_text, "en"),
            Language::De => (&de_text, "de"),
        };
        if !pattern.is_match(text) {
            inert.push(format!(
                "{:?} (family {:?}, {lang_name}): matches nothing in its own source file",
                idiom.pattern, idiom.family
            ));
        }
    }

    assert!(
        inert.is_empty(),
        "F-537: an idiom that matches nothing the rulebook writes is inert coverage \
         masquerading as coverage — fix the pattern or drop it, never weaken it into a \
         plausible-but-unverified guess:\n{}",
        inert.join("\n")
    );
}

/// One representative, real, shipped passage per S2 family (§ 3.1's 14 plus
/// the capability family § 2.1b/D8 needs), proving each family is actually
/// recognized — not merely that *some* pattern in the combined list matches
/// *some* text. Every EN passage is checked to trip via [`states_a_mechanical_rule`]
/// using **only** that family's contribution where practical; where a passage
/// unavoidably also carries an existing token (rare, and noted inline), the
/// point still holds because the family's own sub-phrase is what a human
/// reading the sentence would point to.
///
/// Before [`S2_IDIOMS`] carried these families, every row below failed — that
/// is this test's RED, and it is the reproducible half of this slice's
/// verbatim RED/GREEN pair (the other half is
/// [`no_swept_entry_drops_an_uncomputed_mechanical_clause`] going red as the
/// newly-recognized families reach already-swept passages).
#[test]
fn each_s2_family_is_recognized_by_a_real_shipped_passage() {
    let cases: &[(&str, &str)] = &[
        (
            "capability (EN, D8/§2.1b)",
            "You are able to fly without the need of wings.",
        ),
        (
            "capability (DE, D8/§2.1b)",
            "Du bist in der Lage, in Kampfsituationen oder bei frustrierenden Umständen in einen \
             blinden Wutanfall zu verfallen.",
        ),
        (
            "prohibition/absolutes (EN, family 1/3, bare \"cannot\")",
            "the caster cannot change the height or diameter of the mystic tower",
        ),
        (
            "prohibition (DE, family 1, bounded-gap modal negation)",
            "Charaktere, die auf Wesen dieser Art abstoßend wirken, können diese Tugend nicht \
             nehmen",
        ),
        (
            "permission (EN, family 2, \"at character generation\")",
            "You may take Arcane Abilities at character generation.",
        ),
        (
            "permission (DE, family 2, bounded-gap modal permission)",
            "Um diese Tugend zu wählen, musst Du Akademische Fertigkeiten erlernen dürfen.",
        ),
        (
            "eligibility (EN, family 4)",
            "You must be a Redcap or magus to take this Flaw.",
        ),
        (
            "eligibility (DE, family 4, bounded-gap)",
            "Du musst eine Rotkappe oder ein Magus sein, um diesen Fehler zu wählen.",
        ),
        (
            "incompatibility (EN, family 5)",
            "This Flaw is not compatible with the Night Terrors Flaw.",
        ),
        (
            "incompatibility (DE, family 5)",
            "Dieser Fehler ist unvereinbar mit der Tugend Vielgereist.",
        ),
        (
            "item transfer (EN, family 6)",
            "This Virtue also includes the effects of the Social Contacts Virtue.",
        ),
        (
            "item transfer (DE, family 6, bounded-gap)",
            "Dieser Fehler schließt die Auswirkungen von Gepeinigt von einem übernatürlichen \
             Wesen ein.",
        ),
        (
            "multiplier stem (EN, family 7)",
            "they gain an extra (3 x Wealth Multiplier) Labor Points per year",
        ),
        (
            "multiplier stem (DE, family 7)",
            "ein Multiplikator von drei",
        ),
        (
            "multiplier in words (EN, family 8)",
            "must halve their Lab Total",
        ),
        (
            "multiplier in words (DE, family 8)",
            "Ein doppelter Patzer zeigt an, dass er etwa auf halbem Weg fällt",
        ),
        (
            "\"possibly more\" (EN, family 9)",
            "This benefit has a yearly value of about 50 silver pennies, possibly more if you \
             keep the pressure on.",
        ),
        (
            "bare \"no more\" (EN, family 10)",
            "he is allowed to have a score of 1 (but no more) in either Magic or Faerie Lore",
        ),
        (
            "bare \"nicht mehr\" (DE, family 10)",
            "Du kannst dir gemessen an deinem Status nicht mehr leisten",
        ),
        (
            "floor (EN, family 11, bare \"minimum\")",
            "Magi must have the following minimum Abilities: Parma Magica 1",
        ),
        (
            "floor (DE, family 11, \"mindest\" stem)",
            "mindestens zwölf Magi aus mindestens vier verschiedenen Konventen",
        ),
        (
            "magnitude stem (EN, family 12, bare \"magnitudes\")",
            "would have one of his powers reduced by 3 magnitudes",
        ),
        (
            "magnitude stem (DE, family 12, bare \"Magnituden\")",
            "die Effektivität einer seiner Kräfte nach 50 Lebensjahren um insgesamt fünf \
             Magnituden gesteigert wird",
        ),
        (
            "bare imperative modifier (EN, family 13, \"Subtract\")",
            "Subtract 3 from all of the character's Intelligence and Perception rolls",
        ),
        (
            "bare imperative modifier (DE, family 13, \"Ziehe\")",
            "Ziehe dein Alter ÷ 10 von allen Fortschrittssummen ab",
        ),
        (
            "named rulebook term (EN, family 14, \"Reputation ... at level\")",
            "have a poor Reputation at level 2 within your House",
        ),
        (
            "named rulebook term (DE, family 14, \"Reputation der Stufe\")",
            "Du hast eine gute Reputation der Stufe 4.",
        ),
    ];

    let mut failures = Vec::new();
    for (label, passage) in cases {
        if !states_a_mechanical_rule(passage) {
            failures.push(format!("{label}: {passage:?}"));
        }
    }

    assert!(
        failures.is_empty(),
        "each of these is a REAL shipped passage that should trip the mechanical screen via its \
         named S2 family, and does not:\n{}",
        failures.join("\n")
    );
}
