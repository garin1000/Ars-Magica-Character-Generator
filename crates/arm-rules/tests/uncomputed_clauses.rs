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
    // X2f (2026-09-30): `flaw.form_monstrosity` (ArMDE:6164) states "which
    // corresponds to a magical Form" — the same equivalence idiom as "equal
    // to"/"entspricht", just a different English verb for it.
    "corresponds to",
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
    // X2d: `virtue.unaffected_by_the_gift`'s "is not affected by"/"nicht
    // betroffen" and `virtue.unbound_tongue`'s "with no impediment"/"ohne
    // Einschränkung" are D8 capability grants (immunity, permission) with no
    // signed number to carry them.
    S2Idiom {
        pattern: r"\bnot affected by",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        pattern: r"\bnicht betroffen",
        language: Language::De,
        family: "capability",
    },
    S2Idiom {
        pattern: r"\bwith no impediment",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        pattern: r"\bohne einschränkung",
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
    // "nehmen" added (X2e, 2026-09-30): `flaw.ability_block`'s "Du darfst
    // diesen Fehler nur einmal nehmen" (ArMDE:5651-5654) and
    // `flaw.difficult_underlings`'s "Du darfst diesen Geschichte-Fehler nur
    // nehmen, wenn..." (ArMDE:5976-5979) both close the same modal-verb
    // permission idiom with "nehmen" ("take this Flaw"), a verb this list did
    // not yet carry.
    S2Idiom {
        pattern: r"\b(?:darf|darfst|dürfen)\b[^.]{0,40}?\b(?:wählen|erwerben|erlernen|haben|nehmen)",
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
    // X2e (2026-09-30): `flaw.ceremonial_spontaneous_magic` (ArMDE:5781-5784)
    // widens this gap from 40 to 70 — its DE mirror's "Dieser Fehler ist
    // nicht mit Schwieriger Spontaner Magie oder Schwacher Spontaner Magie
    // vereinbar" puts 64 characters between "ist nicht" and "vereinbar" (the
    // two full Flaw names in between), wider than any hit seen when 40 was
    // chosen. Verified this is still the same idiom, not a different one:
    // the passage states a plain mutual-exclusion rule, same shape as every
    // other hit on this pattern.
    S2Idiom {
        pattern: r"\bist nicht\b[^.]{0,70}?\bvereinbar",
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
    // D79.2 (2026-10-01): `virtue.relic`/`virtue.powerful_relic`'s composed
    // Relics clause (ArMDE:17623) states the bearer's Magic Resistance as a
    // tenfold multiplier of the relic's True Faith score. German renders
    // "ten times" as the nominalized adjective "Zehnfachen" ("Magieresistenz
    // in Höhe des Zehnfachen ihres Wahrer-Glaube-Wertes"), not as
    // "multipliziert"/"multiplikator", so neither existing multiplier idiom
    // catches it. The English sibling already trips `MECHANICAL_PHRASES`'
    // "equal to" ("grants Magic Resistance equal to ten times its True Faith
    // score"), so only this German counterpart is needed.
    S2Idiom {
        pattern: r"\bzehnfache",
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
    // X2e (2026-09-30): `flaw.blackmail`'s German mirror (ArMDE:5707-5710)
    // states the same "more" without "or" idiom as its own EN entry's
    // "possibly more" — "möglicherweise mehr, wenn du den Druck
    // aufrechterhältst" — with no DE counterpart on this family until now.
    S2Idiom {
        pattern: r"\bmöglicherweise mehr",
        language: Language::De,
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
    // X2e (2026-09-30): `flaw.companion_animal`'s German mirror
    // (ArMDE:5805-5808) states the same named-rulebook-term idiom as the EN
    // "an additional personality trait of" above — "eine zusätzliche
    // Persönlichkeitseigenschaft" — with no DE counterpart until now.
    S2Idiom {
        pattern: r"\bzusätzliche persönlichkeitseigenschaft",
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
    // --- Family 16 (X2b Phase 2 additions, 2026-09-29) ---------------------
    // Writing X2b's verbatim descriptions re-exposed genuine D8/D50-shaped
    // rules this screen did not yet recognize under their real phrasing. Each
    // pattern was verified against the one real passage that motivated it.
    S2Idiom {
        // virtue.immune_to_disease (ArMDE:4095-4098): "the lesser demons that
        // cause most diseases refuse to harm him".
        pattern: r"\brefuse to harm\b",
        language: Language::En,
        family: "capability/immunity",
    },
    S2Idiom {
        // virtue.immune_to_disease (ArMDE:4095-4098, DE): "die niederen
        // Dämonen ... weigern sich, ihm Schaden zuzufügen".
        pattern: r"\bweigern sich\b[^.]{0,40}?\bzuzufügen\b",
        language: Language::De,
        family: "capability/immunity",
    },
    S2Idiom {
        // virtue.immunity_to_cold (ArMDE:4099-4102): "Normal cold does not
        // harm you".
        pattern: r"\bdoes not harm\b",
        language: Language::En,
        family: "capability/immunity",
    },
    S2Idiom {
        // virtue.immunity_to_cold (ArMDE:4099-4102, DE): "Normale Kälte
        // schadet dir nicht" — negation on a plain verb, not a modal, so
        // DE_MODAL_NICHT (which requires a modal verb before the gap) does
        // not reach it; narrower-bound than DE_MODAL_NICHT since "schadet"
        // is not itself a modal.
        pattern: r"\bschadet\b[^.]{0,20}?\bnicht\b",
        language: Language::De,
        family: "capability/immunity",
    },
    S2Idiom {
        // virtue.infernal_heirloom (ArMDE:4127-4132): "an effect once per day
        // that is equivalent to a Hermetic spell of level 25" — the formula
        // family's existing "equal to"/"entspricht" pair does not cover this
        // synonym.
        pattern: r"\bequivalent to\b",
        language: Language::En,
        family: "formula",
    },
    S2Idiom {
        // virtue.just_an_instant (ArMDE:4169-4172): "does not need to make
        // Awareness checks" — a roll exemption the existing "does not
        // suffer"/"do not suffer" idiom does not reach.
        pattern: r"\bdoes not need to\b",
        language: Language::En,
        family: "exemption",
    },
    S2Idiom {
        // virtue.just_an_instant (ArMDE:4169-4172, DE): "muss keine
        // Wahrnehmungswürfe ablegen" — an obligation negated with "keine"
        // bound to a later verb, distinct from the modal+keine prohibition
        // family (whose verb set is darf/darfst/dürfen/kann/kannst/können,
        // not muss).
        pattern: r"\bmuss keine\b[^.]{0,40}?\bablegen\b",
        language: Language::De,
        family: "exemption",
    },
    S2Idiom {
        // virtue.knows_people (ArMDE:4199-4206): "a character with this
        // Virtue may ask for a bait" — a real, invokable ability the existing
        // permission family's "may take"/"may learn"/"may purchase" trio does
        // not cover.
        pattern: r"\bmay ask for\b",
        language: Language::En,
        family: "permission",
    },
    S2Idiom {
        // virtue.landed_noble (ArMDE:4219-4228): "you may not impose the
        // death penalty" — the existing "may not take"/"may not have"
        // prohibition idioms do not cover this verb.
        pattern: r"\bmay not impose\b",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        // virtue.lesser_immunity (ArMDE:4275-4278): "You are immune to some
        // hazard".
        pattern: r"\bimmune to\b",
        language: Language::En,
        family: "capability/immunity",
    },
    S2Idiom {
        // virtue.lesser_immunity (ArMDE:4275-4278, DE): "Du bist immun gegen
        // eine Gefahr".
        pattern: r"\bimmun gegen\b",
        language: Language::De,
        family: "capability/immunity",
    },
    S2Idiom {
        // virtue.life_linked_spontaneous_magic (ArMDE:4299-4306): "you must
        // expend one additional Fatigue level per five points" — the German
        // mirror already trips the screen via a hyphenated "Stufe-10-Effekt"
        // (has_signed_number's hyphen-before-digit case), so no DE pattern is
        // needed here.
        pattern: r"\bmust expend\b",
        language: Language::En,
        family: "named rulebook term",
    },
    S2Idiom {
        // virtue.magical_memory (ArMDE:4355-4358): "You need not keep
        // laboratory texts".
        pattern: r"\bneed not\b",
        language: Language::En,
        family: "exemption",
    },
    S2Idiom {
        // virtue.magical_memory (ArMDE:4355-4358, DE): "Du brauchst keine
        // Labortexte ... aufzubewahren".
        pattern: r"\bbrauchst keine\b",
        language: Language::De,
        family: "exemption",
    },
    S2Idiom {
        // virtue.male_guild_sponsor (ArMDE:4439-4442): "his field of work,
        // which is otherwise restricted to men".
        pattern: r"\brestricted to\b",
        language: Language::En,
        family: "eligibility",
    },
    S2Idiom {
        // virtue.male_guild_sponsor (ArMDE:4439-4442, DE): "sein
        // Arbeitsbereich ... der andernfalls Männern vorbehalten ist".
        pattern: r"\bvorbehalten\b",
        language: Language::De,
        family: "eligibility",
    },
    S2Idiom {
        // flaw.vendetta (ArMDE:6955-6958, DE): "Dieser Fehler ist im
        // Allgemeinen auf Magi des Hauses Verditius beschränkt" — the German
        // rendering of the same "restricted to" eligibility idiom.
        pattern: r"\bbeschränkt\b",
        language: Language::De,
        family: "eligibility",
    },
    // --- Family 17 (X2c Phase 2 additions, 2026-09-29) ----------------------
    // Writing X2c's verbatim descriptions re-exposed genuine D5/D8/D50-shaped
    // rules this screen did not yet recognize under their real phrasing. Each
    // pattern was verified against the one real passage that motivated it.
    S2Idiom {
        // virtue.muqta_muq_ta (ArMDE:4559-4562): "All rules for the Landed
        // Noble Virtue apply" — a cross-reference to another entry's rules,
        // in different words than Emir's own "is the same as the ... Virtue".
        pattern: r"\brules for the\b[^.]{0,40}?\bapply\b",
        language: Language::En,
        family: "cross-reference",
    },
    S2Idiom {
        // virtue.muqta_muq_ta (ArMDE:4559-4562, DE): "Es gelten alle Regeln
        // für die Tugend Landadliger" — German's verb-first word order for
        // the same cross-reference.
        pattern: r"\bes gelten\b[^.]{0,40}?\bregeln für\b",
        language: Language::De,
        family: "cross-reference",
    },
    S2Idiom {
        // virtue.muse (ArMDE:4563-4566, DE): "kann ... verdoppeln" — the
        // infinitive of the existing "doppelt" stem, which only covers the
        // participle ("verdoppelt"); "verdoppeln" shares no suffix with it.
        pattern: r"\bverdoppeln\b",
        language: Language::De,
        family: "multiplier in words",
    },
    S2Idiom {
        // virtue.mystical_choreography (ArMDE:4567-4572): "five minutes per
        // magnitude", "one minute per magnitude" — a rate scaled by
        // magnitude, distinct from the existing "magnitudes" (plural) stem.
        pattern: r"\bper magnitude\b",
        language: Language::En,
        family: "magnitude/level stem",
    },
    S2Idiom {
        // virtue.mystical_choreography (ArMDE:4567-4572, DE): "fünf Minuten
        // pro Magnitude", "eine Minute pro Magnitude".
        pattern: r"\bpro magnitude\b",
        language: Language::De,
        family: "magnitude/level stem",
    },
    S2Idiom {
        // virtue.nephilim (ArMDE:4594-4597): "You receive the Strong Angelic
        // Heritage Virtue free" — a real, zero-cost grant, unlike anything
        // the existing "add"/"grants_*" idioms name.
        pattern: r"\breceive the\b[^.]{0,60}?\bfree\b",
        language: Language::En,
        family: "grant",
    },
    S2Idiom {
        // virtue.nephilim (ArMDE:4594-4597, DE): "Du erhältst die Tugend
        // Starkes Engelserbe kostenlos".
        pattern: r"\berhältst die\b[^.]{0,60}?\bkostenlos\b",
        language: Language::De,
        family: "grant",
    },
    S2Idiom {
        // virtue.perfect_eye_for_commodity (ArMDE:4628-4631, DE): "(3 ×
        // Wohlstandsmultiplikator)" — German compounds glue the stem onto
        // the preceding noun with no word boundary, which the existing
        // `\bmultiplikator` (left-anchored) cannot cross.
        pattern: r"multiplikator",
        language: Language::De,
        family: "multiplier stem",
    },
    S2Idiom {
        // virtue.personal_vis_source (ArMDE:4728-4731): "about one tenth as
        // much as" — D50's own hedged-fraction worked example.
        pattern: r"\bone tenth\b",
        language: Language::En,
        family: "formula",
    },
    S2Idiom {
        // virtue.personal_vis_source (ArMDE:4728-4731, DE): "etwa ein
        // Zehntel dessen".
        pattern: r"\bein zehntel\b",
        language: Language::De,
        family: "formula",
    },
    S2Idiom {
        // virtue.rabbi (ArMDE:4828-4833), also virtue.mazdean_priest
        // (ArMDE:4480-4487) and virtue.mamluk (ArMDE:4443-4448): "This
        // Virtue is only available to male characters" — a sex-eligibility
        // restriction (D5/F-123) in a shape none of the existing eligibility
        // idioms ("may only be taken by", "only characters with") covers.
        pattern: r"\bonly available to\b[^.]{0,20}?\bcharacters\b",
        language: Language::En,
        family: "eligibility",
    },
    S2Idiom {
        // The same restriction's two DE verb phrasings: mamluk/mazdean_priest
        // write "ist nur männlichen Charakteren zugänglich", rabbi writes
        // "steht nur männlichen Charakteren zur Verfügung" — anchored on the
        // stable "nur ... Charakteren" core rather than either verb.
        pattern: r"\bnur\b[^.]{0,20}?\bcharakteren\b[^.]{0,20}?\b(?:zugänglich|verfügung)\b",
        language: Language::De,
        family: "eligibility",
    },
    S2Idiom {
        // virtue.rat_up_a_drainpipe (ArMDE:4838-4841): "a substantial
        // advantage in opposed Athletics rolls" — F-241's hedged,
        // storyguide-adjudicated modifier (D50), unsigned and unquantified.
        pattern: r"\bsubstantial advantage\b",
        language: Language::En,
        family: "hedged bonus",
    },
    S2Idiom {
        // virtue.rat_up_a_drainpipe (ArMDE:4838-4841, DE): "einen
        // erheblichen Vorteil bei gegnerischen Athletik-Würfen".
        pattern: r"\berheblichen vorteil\b",
        language: Language::De,
        family: "hedged bonus",
    },
    S2Idiom {
        // virtue.see_in_darkness (ArMDE:4896-4899): "your eyesight is not
        // more acute than ordinary people's" — the limiting clause on an
        // otherwise-bare capability; this file's own header already
        // documents "You can see" as too bare to add safely, so the limiter
        // is the idiom instead.
        pattern: r"\bnot more acute than\b",
        language: Language::En,
        family: "capability limiter",
    },
    S2Idiom {
        // virtue.see_in_darkness (ArMDE:4896-4899, DE): "Dein Sehvermögen
        // nicht schärfer als das normaler Menschen" — negation on a plain
        // predicate adjective, not a modal verb, so DE_MODAL_NICHT does not
        // reach it.
        pattern: r"\bnicht schärfer als\b",
        language: Language::De,
        family: "capability limiter",
    },
    S2Idiom {
        // virtue.puissant_ability (ArMDE:4814-4816, DE): "addierst 2 zu
        // ihrem Wert"; virtue.puissant_art (ArMDE:4818-4820, DE): "addierst
        // 3 zum Wert einer Kunst" — the second-person conjugation of the
        // existing "addiere" stem, which covers the imperative form only.
        pattern: r"\baddierst\b",
        language: Language::De,
        family: "bare imperative modifier",
    },
    // X2d (2026-09-30): four idioms added while closing out
    // `every_uncomputed_rule_entry_states_its_rule_in_every_locale`'s last
    // five offenders (`tmp/x2d-handover.md`). Each verified against a real
    // hit via `every_s2_idiom_has_a_real_hit_in_its_own_language`, and the
    // whole suite re-run afterward per D19 obligation 3.
    S2Idiom {
        // virtue.sense_holiness_and_unholiness (ArMDE:4928, DE): "kann Deine
        // Sensibilität Dich überwältigen" — a positive (non-negated) modal
        // capability-with-consequence, the mirror shape of DE_MODAL_NICHT
        // but without "nicht". Also verified against a second real hit,
        // ArMDE:7721 (the Sense Holiness and Unholiness Ability's own text):
        // "kann dein Gespür dich überwältigen".
        pattern: r"\bkann\b[^.]{0,40}?\büberwältigen\b",
        language: Language::De,
        family: "overwhelm capability",
    },
    S2Idiom {
        // virtue.wanderer (ArMDE:5223-5226): "The Wealthy Major Virtue and
        // Poor Major Flaw affect you normally" — states that the two
        // background-wealth V/F apply unmodified rather than being
        // suppressed by this Virtue's own background, which is itself the
        // rule (absent it, a storyguide could reasonably read the entry as
        // excluding them). Checked catalogue-wide before adding (`grep -n
        // "affect you normally"` against the EN source): 12 entries carry
        // this exact clause, not just virtue.wanderer — the others are
        // filed as newly-swept in PENDING_MECHANICAL_CLASSIFICATION
        // (virtue.craftsman/gentleman/merchant/peasant, `narrative`) and
        // PENDING_DROPPED_CLAUSE (virtue.clerk/failed_apprentice/priest/
        // troubadour, `creation_effect`); virtue.knight, virtue.notary and
        // virtue.wise_one already state a mechanical rule in both locales'
        // shipped `description` (notary's DE text ships the full passage —
        // "still trip the screen" verification, run after this table was
        // drafted, confirmed it and not just an assumption) and need no row.
        pattern: r"\baffect you normally\b",
        language: Language::En,
        family: "wealthy/poor unaffected",
    },
    S2Idiom {
        // The DE phrasing knight/troubadour/wanderer/wise_one share: "Die
        // (Große) Tugend Wohl(hab|stan)d und der (Große) Fehler Arm
        // betreffen dich normal". Other entries in the same 12-strong EN
        // family use a different German verb ("wirken sich normal auf dich
        // aus") that this pattern deliberately does NOT match — widening to
        // catch it too risked also sweeping virtue.nuntius's third-person
        // "wirken sich normal auf ihn aus" and possibly others not yet
        // audited, so those entries' DE gaps are filed on the pending lists
        // above instead of chased with a broader idiom.
        pattern: r"\bbetreffen dich normal\b",
        language: Language::De,
        family: "wealthy/poor unaffected",
    },
    S2Idiom {
        // virtue.tethered_magic (ArMDE:5143): "all of your spells and the
        // effects of any magic items you activate are Arcane Connections to
        // you" — the rule is the Arcane Connection itself, a real and severe
        // side effect, not flavour. Narrowly scoped to this exact
        // construction rather than the far more common "Arcane Connection to"
        // (33 hits catalogue-wide): checked, "are Arcane Connections? to
        // you" has exactly two real hits, this entry and flaw.fettered_magic
        // (ArMDE:6114-6117, "You cannot take this with the Virtue Tethered
        // Magic, as the Virtue already includes this effect") — see the
        // newly-swept-entries note this slice files for that second hit.
        pattern: r"\bare arcane connections? to you\b",
        language: Language::En,
        family: "arcane connection side effect",
    },
    S2Idiom {
        // virtue.voice_of_the_land (ArMDE:5221): "the character is not
        // normally perceived as either a threat or a prey object by these
        // creatures" — a real, if qualitative, in-fiction consequence
        // (creatures do not treat the character as game or a predator),
        // stated nowhere else in the catalogue (`grep -c "prey object"` = 1).
        pattern: r"\bprey object\b",
        language: Language::En,
        family: "capability limiter",
    },
    // --- Family 16 (Arcane Connection duration) ------------------------------
    // X2e (2026-09-30): flaw.bound_casting_tools (ArMDE:5723-5726) states a
    // real mechanical fact — its casting tools become lasting Arcane
    // Connections, where a regular one only lasts weeks — using neither
    // family "arcane connection side effect" above's "are Arcane
    // Connections? to you" construction (this passage says "to him", not
    // "to you", and "become lasting", not "are") nor any signed number or
    // botch term. Distinct enough from that family to need its own pair
    // rather than a widened pattern.
    S2Idiom {
        pattern: r"\blasting arcane connections?",
        language: Language::En,
        family: "arcane connection duration",
    },
    S2Idiom {
        pattern: r"\bdauerhaften arkanen verbindungen",
        language: Language::De,
        family: "arcane connection duration",
    },
    // --- Family 17 (X2f Phase 2 additions, 2026-09-30) -----------------------
    S2Idiom {
        // flaw.fluctuating_fortune (ArMDE:6150-6153): "followed by the Poor
        // Flaw the next... followed by a year in which he has to work three"
        // — a concrete, unmodelled toggling mechanic the screen's existing
        // idioms do not recognize.
        pattern: r"\bfollowed by a year",
        language: Language::En,
        family: "toggling condition",
    },
    S2Idiom {
        // flaw.fluctuating_fortune (ArMDE:6150-6153, DE): "gefolgt von einem
        // Jahr mit dem Fehler Arm" / "gefolgt von einem Jahr, in dem er drei
        // arbeiten muss".
        pattern: r"\bgefolgt von einem jahr",
        language: Language::De,
        family: "toggling condition",
    },
    S2Idiom {
        // flaw.greater_malediction/flaw.lesser_malediction (ArMDE:6210-6213,
        // 6342-6345): "The effects of the curse should be comparable to
        // those of other Major Flaws" / "...should be about as bad as other
        // Minor General Flaws" — a real, storyguide-actionable severity
        // calibration (D8/F-442), not mere colour.
        pattern: r"\beffects of the curse should be",
        language: Language::En,
        family: "curse severity",
    },
    S2Idiom {
        // Same two entries, DE mirror: "Die Auswirkungen des Fluchs sollten
        // denen anderer Großer Fehler vergleichbar sein" / "Die Auswirkungen
        // des Fluches sollten ungefähr so schlimm sein wie andere Kleine
        // Allgemeine Fehler" — declension differs (Fluchs/Fluches), so the
        // stem is matched with no right boundary, same convention as
        // `\bhalve`/`\bhalbiert` above.
        pattern: r"\bauswirkungen des fluch",
        language: Language::De,
        family: "curse severity",
    },
    S2Idiom {
        // flaw.judged_unfairly (ArMDE:6326-6329, DE): "dieser Fehler ist mit
        // jeder Tugend unvereinbar" — the same incompatibility idiom as
        // `\bist unvereinbar mit`/`\bist nicht ... vereinbar` above, but in
        // the word order "ist mit X unvereinbar" (verb-final unvereinbar),
        // which neither existing incompatibility pattern covers. Verified: a
        // single real hit in the whole DE source.
        pattern: r"\bist mit\b[^.]{0,40}?\bunvereinbar",
        language: Language::De,
        family: "incompatibility",
    },
    // X2g (tmp/x2g-verdicts.md): screen misses this slice's own newly-swept
    // entries tripped, each verified against a real hit below.
    S2Idiom {
        // flaw.oath_of_fealty (ArMDE:6512-6515): "Magi are forbidden from
        // taking Oaths of Fealty by the Hermetic Code" — a hard eligibility
        // rule, the same shape as the existing "unmöglich"/"is impossible"
        // pair but a different verb.
        pattern: r"\bforbidden\b",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        // flaw.oath_of_fealty (ArMDE:6512-6515, DE): "Magi ist es durch den
        // Hermetischen Kodex verboten, Treueeide zu leisten" — the German
        // mirror of "forbidden", a standalone word like "unmöglich" rather
        // than a bounded-gap idiom.
        pattern: r"\bverboten\b",
        language: Language::De,
        family: "prohibition",
    },
    S2Idiom {
        // flaw.prohibition (ArMDE:6638-6641): "must obey the restrictions of
        // your prohibition or be penalized by the curse". Scoped to the
        // fuller phrase rather than the bare "must obey": that verb alone
        // also hits virtue.apprentice's unrelated "you must obey the
        // dictates ... of your master" (ArMDE:3418-3421), pure
        // master/apprentice colour with no computed rule to display.
        pattern: r"\brestrictions of your prohibition\b",
        language: Language::En,
        family: "obligation",
    },
    S2Idiom {
        // flaw.prohibition (ArMDE:6638-6641, DE): "musst die Einschränkungen
        // deines Verbots befolgen" — "befolgen" ("to obey/comply with") is
        // the German mirror of "must obey", standalone like "aufwenden"
        // below rather than a bounded-gap modal pattern.
        pattern: r"\bbefolgen\b",
        language: Language::De,
        family: "obligation",
    },
    S2Idiom {
        // flaw.regular (ArMDE:6675-6678): "The character must spend one of
        // his free seasons on the seasonal activity of worship" — a
        // compulsory-expenditure obligation, the same family as "must obey".
        pattern: r"\bmust spend\b",
        language: Language::En,
        family: "obligation",
    },
    S2Idiom {
        // flaw.regular (ArMDE:6675-6678, DE): "Der Charakter muss eines
        // seiner freien Quartale für die Quartalsaktivität Andacht
        // aufwenden" — "aufwenden" ("to expend") is the German mirror,
        // standalone rather than bounded to "muss" since the real gap (63
        // characters) exceeds the existing DE_MODAL_NICHT bound and the verb
        // alone is distinctive enough not to need one.
        pattern: r"\baufwenden\b",
        language: Language::De,
        family: "obligation",
    },
    S2Idiom {
        // flaw.restricted_power (ArMDE:6687-6690): "The character must
        // perform some special ceremony to activate it". Scoped to the
        // fuller phrase rather than the bare "must perform": that verb alone
        // also hits flaw.vow's unrelated "you must perform some kind of
        // atonement" (ArMDE:6989-6992), pure Story-Flaw colour with no
        // computed rule to display.
        pattern: r"\bspecial ceremony\b",
        language: Language::En,
        family: "obligation",
    },
    S2Idiom {
        // flaw.restricted_power (ArMDE:6687-6690, DE): "Der Charakter muss
        // eine besondere Zeremonie durchführen, um sie zu aktivieren" — a
        // modal-then-verb obligation with a 25-character gap, well inside
        // DE_MODAL_NICHT's own 40-character bound but that pattern only
        // recognizes "nicht", not an unnegated obligation, hence its own
        // pattern here.
        pattern: r"\bmuss\b[^.]{0,40}?\bdurchführen\b",
        language: Language::De,
        family: "obligation",
    },
    S2Idiom {
        // flaw.restriction (ArMDE:6691-6694, DE): "Du kannst unter
        // bestimmten, seltenen Bedingungen überhaupt keine Zauber wirken" —
        // the English mirror ("cannot") already matches the existing
        // "prohibition/absolutes" family's bare `\bcannot`, but the German
        // "kannst ... keine" gap (51 characters) exceeds the existing
        // "prohibition via keine" idiom's 40-character bound. Rather than
        // widen that bound catalogue-wide, this is the distinctive phrase
        // actually adjoining "keine" in this passage.
        pattern: r"\büberhaupt keine\b",
        language: Language::De,
        family: "prohibition",
    },
    // --- X2h (tmp/x2h-verdicts.md, ArMDE:6709-7109) ---------------------
    // Phase 2 reclassified 21 entries in this range to `uncomputed_rule`;
    // `every_uncomputed_rule_entry_states_its_rule_in_every_locale` then
    // requires the shipped description to trip the screen in BOTH locales.
    // Each row below was verified against the actual rulebook passage
    // (D19 obligation 3) before being added, not merely against the shipped
    // description text.
    S2Idiom {
        // flaw.slow_caster (ArMDE:6755-6758): "Your Formulaic spells take
        // two rounds to cast" — a real timing penalty (double the normal
        // one-round cast), spelled as a word rather than a digit.
        pattern: r"\btwo rounds\b",
        language: Language::En,
        family: "cap",
    },
    S2Idiom {
        // flaw.slow_caster (ArMDE:6755-6758, DE): "Deine Formulaischen
        // Zauber benötigen zwei Runden zum Wirken" — same word-spelled
        // timing penalty.
        pattern: r"\bzwei runden\b",
        language: Language::De,
        family: "cap",
    },
    S2Idiom {
        // flaw.stuck_in_your_ways (ArMDE:6791-6794): "this character uses
        // the lower of that Ability score and his current Covenant Lore
        // Ability" — a real roll-substitution rule with no signed number.
        pattern: r"\bthe lower of\b",
        language: Language::En,
        family: "formula",
    },
    S2Idiom {
        // flaw.stuck_in_your_ways (ArMDE:6791-6794, DE): "verwendet dieser
        // Charakter den niedrigeren Wert aus seiner Fertigkeit..." — the
        // same roll-substitution rule.
        pattern: r"\bniedrigeren wert\b",
        language: Language::De,
        family: "formula",
    },
    S2Idiom {
        // flaw.susceptibility_to_warping (ArMDE:6831-6838): "the character
        // gains one additional Warping Point" / "he gains four additional
        // Warping Points" — the extra-Warping-Point rule is spelled with
        // word numbers ("one"/"four"), not digits, so it carries no signed
        // number for `has_signed_number` to catch.
        pattern: r"\badditional warping point\b",
        language: Language::En,
        family: "formula",
    },
    S2Idiom {
        // flaw.susceptibility_to_warping (ArMDE:6831-6838, DE): "erhält der
        // Charakter einen zusätzlichen Verzerrungspunkt" — same word-spelled
        // rule.
        pattern: r"\bzusätzlichen verzerrungspunkt\b",
        language: Language::De,
        family: "formula",
    },
    S2Idiom {
        // flaw.unruly_air (ArMDE:6939-6942): "others with Magic Resistance
        // are not influenced by him" — a real, absolute exemption (D8
        // capability), phrased differently from the existing "not affected
        // by"/"nicht betroffen" pair.
        pattern: r"\bnot influenced\b",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        // flaw.unruly_air (ArMDE:6939-6942, DE): "andere mit Magieresistenz
        // nicht von ihm beeinflusst werden" — same exemption; "nicht" and
        // "beeinflusst" sit 12 characters apart ("von ihm "), inside a
        // bounded gap rather than contiguous.
        pattern: r"\bnicht\b[^.]{0,20}?\bbeeinflusst",
        language: Language::De,
        family: "capability",
    },
    S2Idiom {
        // flaw.visions (ArMDE:6985-6988): "The visions come purely at the
        // storyguide's discretion" — D8/story: the rule that visions are
        // entirely GM-adjudicated, not a signed number or phrase idiom
        // already in the base list. Scoped to the fuller "purely at..."
        // phrase rather than the bare "storyguide's discretion": that
        // shorter form also hits flaw.demonic_familiar's "At the
        // storyguide's discretion, this Flaw may be taken to represent
        // other sorts of demons" (ArMDE:5930) and flaw.favors' "at the
        // storyguide's discretion" (ArMDE:6100) and
        // virtue.latent_magic_ability's "At the storyguide's discretion,
        // this quality might appear" (ArMDE:4239) — three unrelated,
        // already-settled narrative entries (X2b: latent_magic_ability
        // "stays narrative, no rule stated") whose own discretion clause is
        // pure reflavoring permission, not a rule this entry drops.
        pattern: r"\bpurely at the storyguide's discretion\b",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        // flaw.visions (ArMDE:6985-6988, DE): "Die Visionen kommen
        // ausschließlich nach dem Ermessen des Spielleiters" — same
        // GM-discretion rule, same narrowing reason as the EN row above (the
        // bare "Ermessen des Spielleiters" also hits the German mirrors of
        // demonic_familiar/favors/latent_magic_ability).
        pattern: r"\bausschließlich nach dem ermessen des spielleiters\b",
        language: Language::De,
        family: "capability",
    },
    S2Idiom {
        // flaw.vow_major/flaw.vow_minor (ArMDE:6989-6992): "A Vow that is a
        // Major Flaw must be a vow to do something, rather than refrain from
        // something" — OQ-09's real creation-time constraint on which kind
        // of vow qualifies as Major. Distinct from the "must perform some
        // kind of atonement" clause the `restricted_power`/"special
        // ceremony" idiom's own comment already flags as NOT a computed rule
        // in this entry — this is the separate, genuinely mechanical clause.
        pattern: r"\brather than refrain\b",
        language: Language::En,
        family: "obligation",
    },
    S2Idiom {
        // flaw.vow_major/flaw.vow_minor (ArMDE:6989-6992, DE): "muss ein
        // Gelübde sein, etwas zu tun, anstatt etwas zu unterlassen" — same
        // Major-vow constraint.
        pattern: r"\banstatt etwas zu unterlassen\b",
        language: Language::De,
        family: "obligation",
    },
    S2Idiom {
        // flaw.warped_magic (ArMDE:7023-7026): "with increasing intensity
        // according to the level of the spell" — a real scaling rule (side
        // effect severity tracks spell level) stated in words.
        pattern: r"\baccording to the level of the spell\b",
        language: Language::En,
        family: "formula",
    },
    S2Idiom {
        // flaw.warped_magic (ArMDE:7023-7026, DE): "mit zunehmender
        // Intensität entsprechend der Stufe des Zaubers" — same scaling
        // rule.
        pattern: r"\bzunehmender intensität\b",
        language: Language::De,
        family: "formula",
    },
    S2Idiom {
        // flaw.short_lived_magic (ArMDE:6729-6732): "Diameter, Concentration,
        // Ring, and Momentary spells are not affected" — an absolute
        // exemption from the duration-reduction rule, phrased without "by"
        // (an object), so it does not trip the existing "not affected by"
        // idiom.
        pattern: r"\bare not affected\b",
        language: Language::En,
        family: "capability",
    },
    S2Idiom {
        // flaw.slow_power (ArMDE:6759-6762): "not more than once for a
        // single power" — the repeatability cap, spelled "not more than"
        // rather than the base list's "no more than".
        pattern: r"\bnot more than\b",
        language: Language::En,
        family: "cap",
    },
    S2Idiom {
        // flaw.unstructured_caster (ArMDE:6947-6950): "you may not learn
        // Ritual spells at all" — a real, absolute restriction distinct from
        // the base list's "may not take"/"may not be" phrasing.
        pattern: r"\bmay not learn\b",
        language: Language::En,
        family: "prohibition",
    },
    S2Idiom {
        // flaw.short_ranged_magic (ArMDE:6737-6740, DE): "Halbiere deine
        // Zaubersummen..." / "Halbiere deine Laborsumme..."; flaw.weak_magic
        // (ArMDE:7064-7067, DE): "Du halbierst die normale
        // Penetrationssumme..." — the imperative ("du") and 2nd-person
        // ("du ... -st") conjugations of "halve", neither of which the
        // existing `\bhalbiert` (3rd person/past participle) idiom matches.
        // Widened to the bare stem, no right boundary — the same shape as
        // the existing `\bmultipl`-family stems — rather than one idiom per
        // conjugation.
        pattern: r"\bhalbier",
        language: Language::De,
        family: "formula",
    },
    S2Idiom {
        // flaw.sheltered_upbringing (ArMDE:6721-6724, DE): "Du darfst
        // Feilschen, Charme, Etikette, Menschenkenntnis, Täuschung, Intrige
        // oder Führung nicht als Anfangsfertigkeiten nehmen" — the English
        // mirror already matches the base list's "may not take", but the
        // German modal ("darfst") and its "nicht" sit more than
        // [`DE_MODAL_NICHT`]'s 40-character bound apart (the seven named
        // Abilities sit between them), so this is the distinctive tail
        // phrase instead.
        pattern: r"\bnicht als anfangsfertigkeiten nehmen\b",
        language: Language::De,
        family: "prohibition",
    },
    S2Idiom {
        // flaw.spontaneous_casting_tools (ArMDE:6779-6782, DE): "Dieser
        // Fehler kann nur von Verditius-Magi gewählt werden" — the English
        // mirror already matches the base list's "can only", but the German
        // uses "kann nur von ... gewählt werden", which neither the existing
        // "darf nur von ... genommen werden" nor "kann nur für ... genommen
        // werden" idioms cover (different modal/verb pairing).
        pattern: r"\bkann nur von\b[^.]{0,40}?\bgewählt werden",
        language: Language::De,
        family: "prohibition",
    },
    S2Idiom {
        // flaw.usurer (ArMDE:6951-6954): "You receive the equivalent of
        // approximately ten pounds of silver each year from interest
        // payments" — F-544's dropped clause, an amount spelled in words
        // rather than digits.
        pattern: r"\bten pounds of silver\b",
        language: Language::En,
        family: "formula",
    },
    S2Idiom {
        // flaw.usurer (ArMDE:6951-6954, DE): "Du erhältst jährlich das
        // Äquivalent von ungefähr zehn Pfund Silber" — same word-spelled
        // amount.
        pattern: r"\bzehn pfund silber\b",
        language: Language::De,
        family: "formula",
    },
    S2Idiom {
        // flaw.warped_by_magic (ArMDE:7019-7022): "His encounters allow you
        // to spend experience points on Magic Lore during character
        // creation" — F-544's dropped clause, a creation-time XP grant.
        pattern: r"\bexperience points on magic lore\b",
        language: Language::En,
        family: "formula",
    },
    S2Idiom {
        // flaw.warped_by_magic (ArMDE:7019-7022, DE): "erlauben es dir, ...
        // Erfahrungspunkte auf Magiekunde auszugeben" — same XP grant.
        pattern: r"\berfahrungspunkte auf magiekunde\b",
        language: Language::De,
        family: "formula",
    },
    S2Idiom {
        // flaw.tragic_life (ArMDE:6855-6870, DE): "...dass ihr Schöpfer
        // normalerweise keine alternative Situation herbeiführen kann..." —
        // the English mirror already matches the base list's bare
        // `\bcannot`, but the German subordinate clause puts the modal verb
        // LAST ("keine ... kann", not "kann ... keine"), the reverse of the
        // existing bounded-gap idiom's word order. Scoped to "keine
        // alternative" rather than a bare "keine ... kann": the broader gap
        // also closed `PENDING_DROPPED_CLAUSE`'s virtue.redcap row for an
        // unrelated reason — "keine hermetische Magie wirken kannst" ("cannot
        // work Hermetic magic") is just this Virtue's own defining trait, not
        // the tracked gap (the per-year item-growth rate, the free
        // Well-Traveled grant, the free Longevity Ritual) — so the pattern is
        // narrowed to the word actually adjoining "keine" in *this* passage.
        pattern: r"\bkeine alternative\b[^.]{0,40}?\bkann\b",
        language: Language::De,
        family: "prohibition",
    },
    S2Idiom {
        // X9c/D77.1: fixing virtue.tainted_treasure's source range (F-309,
        // dropping a sidebar that was actually Templar Administrator's own
        // background text) re-exposed this entry's real rule, previously
        // masked by the Knights Templar paragraph's unrelated "restricted to"
        // hit. ArMDE:5101: "If it is used for a nonsinful purpose, the
        // treasure destroys itself" — the GM-judgement/open-ended-magnitude
        // shape `uncomputed_rule` exists for (no number, no roll, just a
        // guaranteed-but-unspecified consequence), the same family as
        // "cannot die". Verified unique in the book (one hit, this passage).
        pattern: r"\bdestroys itself\b",
        language: Language::En,
        family: "absolutes",
    },
    S2Idiom {
        // virtue.tainted_treasure (ArMDE:5101, DE): "zerstört der Schatz sich
        // selbst" — same idiom, German word order puts the subject between
        // the verb and "sich selbst", so this is a bounded gap (never
        // crossing a sentence) rather than a contiguous phrase, mirroring
        // DE_MODAL_NICHT's own reason for being a gap pattern. The book's
        // other "sich selbst" (the Fire/Air Elemental's "mit sich selbst zu
        // vereinen") sits in a different sentence from any "zerstört", so the
        // period boundary keeps this from crossing into it.
        pattern: r"\bzerstört\b[^.]{0,30}?\bsich selbst\b",
        language: Language::De,
        family: "absolutes",
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
    // flaw.horrifying_appearance_snake_legs: OVERTURNED (X2f) — this reading
    // (no rule, "or more" only counts tails) still holds on its own terms, but
    // D8 overrides it regardless: the entry is categories:["supernatural"],
    // and D8 reclassifies every `supernatural` + `narrative` entry to
    // `uncomputed_rule` on the "a capability is a rule" argument, independent
    // of whether the mechanical-token screen would itself have caught
    // anything. Removing the row lets
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` bite it directly
    // (the "or more" token still trips the screen, which is sufficient).
    // See tmp/x2f-verdicts.md.
    (
        "flaw.lecherous_major",
        "ArMDE:6334-6337's \"you need not be any good at seduction; skill here merely changes \
         the kinds of problems you encounter\" states no restriction, cap, or roll — a bare \
         disclaimer that no skill gates this Flaw, the same non-mechanical \"need not\" shape \
         (family 1/3-adjacent) as flaw.slothful/flaw.tzadik_nistar. Moved from \
         PENDING_MECHANICAL_CLASSIFICATION (X2f): re-read against D50 finds nothing a player or \
         storyguide must act on.",
    ),
    (
        "flaw.lecherous_minor",
        "The Minor half of the same entry, citing the same passage (ArMDE:6334-6337). Same \
         reading as flaw.lecherous_major.",
    ),
    // "flaw.primogeniture_lineage" (ArMDE:6634-6637) is REMOVED here (X2g,
    // tmp/x2g-verdicts.md, D67): this row's own reading ("the one genuinely
    // mechanical clause... is already computed via `prerequisites`") was an
    // argument for reclassifying to `creation_effect`, not for staying
    // `narrative` — `exempted_entries_still_trip_the_screen` requires every
    // row here to still cite a `narrative` entry, and this one no longer is.
    // See `primogeniture_lineage_reclassifies_to_creation_effect`
    // (`x2_reclassification.rs`) for the pinned target classification.
    // "flaw.true_love_major"/"flaw.true_love_minor" (ArMDE:6871-6878) are
    // REMOVED here (X2h, tmp/x2h-verdicts.md, D67): this row's own reading
    // ("nothing is dropped, because there is no third thing for the text to
    // say") was an argument for reclassifying to `creation_effect`, not for
    // staying `narrative` — both ids are ALSO on
    // `data_integrity.rs::PENDING_D67_CLASSIFICATION` (declare
    // `incompatible_with` each other), whose own documentation states
    // unambiguously that `is_computed` "cannot itself decide which computed
    // class an entry belongs to, only that it is not narrative." The one
    // genuinely mechanical clause (the Major/Minor magnitude split) is fully
    // computed via the two-entry split + `incompatible_with`; the rest is
    // Story-Flaw fiction (D60's own reasoning for the twin PC Virtue). Same
    // shape as `flaw.primogeniture_lineage`'s X2g resolution.
    // `exempted_entries_still_trip_the_screen` requires every row here to
    // still cite a `narrative` entry, and neither one is, once reclassified.
    // See `true_love_reclassifies_to_creation_effect`
    // (`x2_reclassification.rs`) for the pinned target classification. X2t
    // (True Friend, D60.3) copies True Love's classification too — flagged in
    // the handover.
    // --- S2 additions (docs/vf-audit/corrections.md § 3.1): growing the screen's
    // families newly trips these, and reading each passage finds no rule —
    // an idiom used non-mechanically, not a dropped clause.
    // flaw.busybody: OVERTURNED (X2e) — this reading pre-dates D50. Re-read
    // against D50's actual test ("would a player/storyguide get it wrong
    // without knowing it"), not the narrower "is it computed" question this
    // row asked: ArMDE:5763-5764's "magi probably don't have much knowledge
    // of what's going on among the lower-class members of their covenant
    // unless they choose to apply this Flaw specifically to such people at
    // character creation" is a real creation-time scope choice — the same
    // shape as X2b's virtue.indescribable_face overturn (a mandatory-if-
    // desired choice the player must know to make, even though neither
    // choice's behavior is itself computed). Reclassifies to
    // uncomputed_rule; see tmp/x2e-verdicts.md.
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
    // flaw.exiled_atlantean: OVERTURNED (X2f) — D8 overrides this reading
    // regardless of whether the passage's "cannot return to her magic regio"
    // is itself a computed rule. D8: "A capability is a rule", and applies to
    // every `supernatural`-category `narrative` entry without exception — the
    // ruling's own count (48 of 115) is not scoped to Virtues, and this Flaw
    // is categories:["supernatural"]. Reclassifies to uncomputed_rule with the
    // full passage in description (both locales); removing the row lets
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` bite it directly.
    // See tmp/x2f-verdicts.md.
    (
        "flaw.soft_hearted",
        "ArMDE:6775-6778's \"You cannot bear to witness suffering\" (bare \"cannot\", family 1/3) \
         is the same \"can't bear X\" temperament idiom as flaw.compassionate_major, not a \
         restricted capability.",
    ),
    // "flaw.tragic_life" (ArMDE:6855-6870) is REMOVED here (X2h,
    // tmp/x2h-verdicts.md, OQ-6): this row's own reading addressed only the
    // "cannot" token ("describes a limit on the demon's narrative planning"),
    // which still holds, but it missed ArMDE:6859's separate, real
    // creation-time instruction — "The predisposition toward sin at the
    // character's pivotal moment should be represented with a sinful
    // Personality Trait" — the same D50/OQ-6 shape that already overturned
    // `ghostly_warder`/`magical_warder`/`paid_rights`/`tainted_treasure`.
    // Reclassifies to `uncomputed_rule`. See
    // `tragic_life_states_its_sinful_personality_trait_instruction`
    // (`x2_reclassification.rs`).
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
    // virtue.indescribable_face: OQ-6 re-test (X2b) overturns this row — the
    // "no roll, bonus, or cap" reading missed that ArMDE:4109's "A player who
    // selects this Virtue for his character needs to select which form of the
    // Virtue his character has" is a real, mandatory creation-time choice
    // (D50: a player who does not know it will not make it), even though
    // neither form's behavior is itself computed. Reclassifies to
    // uncomputed_rule; see tmp/x2b-verdicts.md.
    // virtue.magical_warder: OQ-6 re-test (X2b) overturns this row too, on the
    // same clause X2a already reversed for its own cited sibling: ArMDE:4381
    // "leave your presence for up to half a day" is the identical
    // storyguide-enforced utility limit that flipped virtue.ghostly_warder.
    // Reclassifies to uncomputed_rule; see tmp/x2b-verdicts.md.
    // virtue.paid_rights: OQ-6 re-test (X2c) overturns this row — the "cannot
    // pay a fine to" readings still hold (in-fiction social premise), but
    // ArMDE:4606-4615's own permission grant ("only available to female
    // characters, and is compatible with any Social Status that is normally
    // restricted to men") is the same real, separately-uncomputed eligibility/
    // compatibility shape X2b already reclassified for
    // virtue.male_guild_sponsor. Reclassifies to uncomputed_rule; see
    // tmp/x2c-verdicts.md.
    // virtue.tainted_treasure: OVERTURNED (X2d) — this reading pre-dates D50.
    // Re-read against D50's actual test ("would a player/storyguide get it
    // wrong without knowing it"): the curse's specific consequences (trading
    // moves the curse; a non-sinful use destroys the treasure; ventures/
    // buildings eventually fail/burn) are exactly the kind of "guaranteed
    // storyguide intervention" D50's common_sense example names — a
    // storyguide who does not know the treasure self-destructs on a charitable
    // gift will run it wrong. Reclassifies to uncomputed_rule; see
    // tmp/x2d-verdicts.md. (F-309's source.lines/sidebar defect is a separate,
    // unrelated finding — not this slice's to fix.)
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
    // --- X2h additions (tmp/x2h-verdicts.md): moved from
    // PENDING_MECHANICAL_CLASSIFICATION — re-read against D50 finds nothing a
    // player or storyguide must act on, the same "need not" false-positive
    // shape as flaw.lecherous_major/_minor above.
    (
        "flaw.slothful",
        "ArMDE:6751-6754's \"need not\" idiom — \"very good at coming up with excuses as to why \
         things need not be done right now\" — states no restriction, cap, or roll: a Personality \
         Flaw's temperament description, pure narrative flavour.",
    ),
    (
        "flaw.tzadik_nistar",
        "ArMDE:6883-6886's \"need not\" idiom — \"a character need not be Jewish to take this \
         Flaw\" — is a pure eligibility disclaimer (removing a possible misconception), not a \
         restriction. The rest of the passage (consequences \"defined by the storyguide\") is \
         story colour, the same shape as flaw.visions/flaw.tainted_offspring.",
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
    // X2e (`tmp/x2-worklist.md` rows 204-260, `tmp/x2e-verdicts.md`): all 13 of
    // this slice's rows below (ability_block, bigamist, blackmail, blind,
    // bound_magic, ceremonial_spontaneous_magic, chaotic_magic,
    // companion_animal, consumed_casting_tools, crippled, deteriorating_power,
    // difficult_spontaneous_magic, difficult_underlings) resolved the same
    // way: each passage genuinely states an uncomputed rule (D46/D50), so the
    // reclassification target is `uncomputed_rule`, not a misclassification —
    // Phase 2's job. `no_swept_entry_drops_an_uncomputed_mechanical_clause`
    // bites each directly now that the row is gone.
    // X2f (`tmp/x2-worklist.md` rows 261-317, `tmp/x2f-verdicts.md`): all 10
    // rows below (disorientating_magic, enfeebled, envied_beauty,
    // exciting_experimentation, false_power, false_power_minor,
    // fettered_magic, harmless_magic, incompatible_arts, judged_unfairly)
    // resolved the same way: each passage genuinely states an uncomputed rule
    // (D46/D50), reclassification target `uncomputed_rule`, Phase 2's job.
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` bites each
    // directly now that the row is gone.
    // flaw.hermetic_patron: resolved (X5a) — the "must be a Redcap or magus"
    // eligibility clause is now a `prerequisites` gate (D68.2); reclassifies
    // narrative -> creation_effect. See tmp/x5-verdicts.md.
    // flaw.lecherous_major, flaw.lecherous_minor: OVERTURNED (X2f) — this was
    // a placeholder row from the X2b Phase 2 fix-round ("filed here per that
    // row's classification precondition rather than resolved"), never a
    // confirmed drop. ArMDE:6334-6337's "you need not be any good at
    // seduction; skill here merely changes the kinds of problems you
    // encounter" states no restriction, cap, or roll — it is the same
    // non-mechanical "need not" shape as flaw.slothful/flaw.tzadik_nistar's
    // own family, a false positive of the idiom, not a dropped rule. Moved to
    // NO_RULE_DESPITE_TOKEN; see tmp/x2f-verdicts.md.
    // flaw.magical_being_companion: resolved (X2g, `tmp/x2g-verdicts.md`) —
    // reclassifies narrative -> uncomputed_rule, full three-paragraph passage
    // (ArMDE:6386-6391, including the "10 – Size" Magic Might formula) as
    // description in both locales, a Phase 2 data change not yet landed.
    // flaw.master_of_none: resolved (X2g) — reclassifies narrative ->
    // uncomputed_rule, full passage (ArMDE:6418-6421, the lost-XP rule) as
    // description in both locales.
    // flaw.monastic_vows_hermetic: resolved (X2g) — reclassifies narrative ->
    // uncomputed_rule (D67: the existing `prerequisites: hermetically_trained`
    // already computes something, but the "cannot own vis"/"cannot marry"
    // clauses are a real, separately uncomputed rule), full passage
    // (ArMDE:6450-6453) as description in both locales.
    // flaw.motion_sickness: resolved (X2g) — reclassifies narrative ->
    // uncomputed_rule, full passage (ArMDE:6468-6471, the double-fatigue/
    // two-level-floor formula) as description in both locales.
    // flaw.necessary_condition: resolved (X2g) — reclassifies narrative ->
    // uncomputed_rule (same D67 shape as monastic_vows_hermetic above), full
    // passage (ArMDE:6476-6479) as description in both locales.
    // flaw.no_hands: resolved (X2g) — reclassifies narrative ->
    // uncomputed_rule, full passage (ArMDE:6496-6499, the "– 5" Casting Score
    // penalty — note the source's own EN DASH + space + digit spacing, which
    // the verbatim checker's digit-adjacent normalization does not touch) as
    // description in both locales.
    // flaw.no_sense_of_direction, flaw.outcast: resolved (X4/X2e fallout,
    // 2026-09-30) — a concurrent slice's data change gave each entry an
    // `incompatible_with` (well_traveled / wealthy respectively), which D67
    // counts as computed, so both already reclassified to `creation_effect`
    // and stopped tripping this guard. Removed per shrink-only list
    // semantics, same convention as the 13 X2e rows above.
    // flaw.restricted_learning: resolved (X6b, design-x6-parameters.md e5) —
    // the five-Ability restriction now carries a `restricted_ability_xp`
    // effect (with `abilities_param` unioning the Supernatural-category
    // grant) and a dedicated `validate_ability_xp_scope` validator, so it
    // reclassified to `creation_effect` and stopped tripping this guard
    // (which requires `narrative`). Removed per shrink-only list semantics.
    // flaw.restriction: resolved (X2g) — reclassifies narrative ->
    // uncomputed_rule (same D67 shape as monastic_vows_hermetic/
    // necessary_condition above: the existing `prerequisites` computes
    // something, but the conditional-restriction-plus-enchanted-item clause is
    // a real, separately uncomputed rule), full passage (ArMDE:6691-6694) as
    // description in both locales.
    // flaw.sheltered_upbringing, flaw.stockade_parma_magica,
    // flaw.study_requirement, flaw.suppressed_gift, flaw.tainted_with_evil,
    // flaw.unnatural_magic, flaw.vulnerable_magic, flaw.wanderlust: resolved
    // (X2h, tmp/x2h-verdicts.md) — all eight reclassify narrative ->
    // uncomputed_rule, each passage's real clause is a genuine dropped rule
    // (D50/D67). `no_swept_entry_drops_an_uncomputed_mechanical_clause` now
    // bites each directly.
    // --- X2b Phase 2 fix-round additions (2026-09-29): new S2 idioms written
    // to satisfy X2b's own entries' verbatim descriptions newly sweep these
    // six entries elsewhere in the catalogue, all still `narrative`, all
    // outside X2b's own range (ArMDE:4067-4442) — filed here per that row's
    // classification precondition rather than resolved, exactly as any other
    // PMC row awaiting its own slice. (flaw.lecherous_major/_minor:
    // OVERTURNED and moved to NO_RULE_DESPITE_TOKEN — see X2f above.)
    (
        "flaw.slothful",
        "\"need not\" idiom: \"very good at coming up with excuses as to why things need not be \
         done right now\", ArMDE:6751-6754",
    ),
    (
        "flaw.tzadik_nistar",
        "\"need not\" idiom: \"a character need not be Jewish to take this Flaw\", ArMDE:6883-6886",
    ),
    // flaw.vendetta: resolved (X5a) — the "restricted to magi of House
    // Verditius" clause is now a hard `order_member` prerequisite for the
    // magus half (the House half stays the existing advisory hedge, D16);
    // reclassifies narrative -> creation_effect. See tmp/x5-verdicts.md.
    // virtue.templar_confrere_or_consoeur: resolved (X2d) — reclassifies to
    // uncomputed_rule; the "need not" idiom hit was a false trigger, but a real
    // clause survives it ("he may possess other Social Status Virtues or
    // Flaws", "women may also become associate members", fewer rights than
    // full members). See tmp/x2d-verdicts.md.
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
    // virtue.homing_instinct, virtue.imbued_with_the_spirit_of_form,
    // virtue.inspirational, virtue.intuition, virtue.kassalan_exorcism,
    // virtue.keen_sense_of_smell, virtue.keen_vision, virtue.land_regio_network,
    // virtue.learn_ability_from_mistakes, virtue.leather_ripper,
    // virtue.lesser_benediction, virtue.license_of_absence, virtue.luck,
    // virtue.maker_of_textured_vessels, virtue.maker_of_water_vessels: resolved
    // (X2b) — all reclassify narrative -> uncomputed_rule, see
    // tmp/x2b-verdicts.md. virtue.inoffensive_to_beings and virtue.magical_mount
    // also resolved (X2b): both reclassify to uncomputed_rule too (a real,
    // separately uncomputed clause survives their own `prerequisites`), so both
    // are removed from here AND from
    // data_integrity.rs::PENDING_D67_CLASSIFICATION.
    // virtue.minor_enchantments, virtue.muse, virtue.natural_leader,
    // virtue.perfect_balance, virtue.perfect_eye_for_commodity,
    // virtue.performance_magic, virtue.piercing_gaze,
    // virtue.reserves_of_strength, virtue.ripper: resolved (X2c) — all nine
    // reclassify narrative -> uncomputed_rule, see tmp/x2c-verdicts.md.
    // virtue.sharp_ears, virtue.side_effect, virtue.skilled_smuggler,
    // virtue.skinchanger, virtue.skinchanger_dove, virtue.social_contacts,
    // virtue.spiritual_pact, virtue.strong_willed, virtue.supernatural_beauty,
    // virtue.temporal_influence, virtue.troupe_upbringing, virtue.true_love_pc,
    // virtue.variable_power, virtue.venus_blessing, virtue.verditius_magic,
    // virtue.wisdom_from_ignorance: resolved (X2d) — all sixteen reclassify
    // narrative -> uncomputed_rule, see tmp/x2d-verdicts.md.
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
    // virtue.hermetic_magus: resolved (X2b) — reclassifies to creation_effect
    // per D46's own ruling (ArMDE:4067-4070, computed via the magus profile's
    // required_traits); removed from data_integrity.rs::PENDING_D46_CLASSIFICATION
    // too, see tmp/x2b-verdicts.md.
    // --- X2a fix-round additions (2026-09-29): the new "capability"/
    // "eligibility" idioms above newly sweep these two real, not-yet-landed
    // findings. Neither is on the X2a worklist (`tmp/x2-worklist.md` § 1 rows
    // 1-51); each is a genuine dropped clause for a later X2 slice. (A third,
    // virtue.simple_student, is NOT narrative — creation_effect — so it cannot
    // live here; see PENDING_DROPPED_CLAUSE below.)
    // flaw.spontaneous_casting_tools: resolved (X2h, tmp/x2h-verdicts.md) —
    // a concurrent slice landed `prerequisites: all(order_member,
    // house.verditius)` for this entry since this row was filed, so the
    // eligibility clause it names ("can only be taken by Verditius magi") is
    // now computed — but ArMDE:6779-6782's casting-tools requirement ("must
    // use casting tools to cast spontaneous spells") is a real, separately
    // uncomputed rule that survives it (D67), so the entry still reclassifies
    // narrative -> uncomputed_rule, just not for the reason this row
    // originally gave.
    // virtue.lesser_purifying_touch: resolved (X2b) — reclassifies to
    // uncomputed_rule (ArMDE:4287-4290, "You can only choose an illness, not an
    // injury or other misfortune" is a real restriction the summary does not
    // carry), see tmp/x2b-verdicts.md.
    // --- X2d fix-round additions (2026-09-30): the new "wealthy/poor
    // unaffected" idiom (added to close virtue.wanderer's own screen miss)
    // newly sweeps four more `narrative` entries whose passage states the
    // identical "Wealthy/Poor affect(s) you normally" clause. None was on
    // X2d's own worklist (rows 154-203); each is a genuine dropped clause for
    // a later X2 slice, not fixed here.
    (
        "virtue.craftsman",
        "\"The Wealthy Major Virtue and Poor Major Flaw affect you normally\", \
         ArMDE:3621-3624 — `narrative`, no description carrying it",
    ),
    (
        "virtue.gentleman",
        "\"The Wealthy Virtue and Poor Flaw affect you normally\", ArMDE:3959-3962 — \
         `narrative`, no description carrying it",
    ),
    (
        "virtue.merchant",
        "\"The Wealthy Major Virtue and Poor Major Flaw affect you normally\", \
         ArMDE:4506-4509 — `narrative`, no description carrying it",
    ),
    (
        "virtue.peasant",
        "\"The Wealthy Major Virtue and Poor Major Flaw affect you normally\", \
         ArMDE:4620-4623 — `narrative`, no description carrying it",
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
        "virtue.hermetic_magus",
        "X2b (tmp/x2b-verdicts.md), D46's own named case: the whole passage in ArMDE:4067-4070 \
         (\"All magi must take this as their Social Status, and only magi may take it\") is \
         computed by the magus character-type profile's `required_traits` naming this entry, not \
         by any effect/description of the entry's own; classification moved narrative -> \
         creation_effect and no separate description is owed (the entry also already carries a \
         `prerequisites: has(virtue.the_gift)`, computed twice over).",
    ),
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
        "flaw.pagan_minor",
        "D78.2: the Minor twin of flaw.pagan, citing the SAME passage (ArMDE:6570-6573) and \
         carrying the identical ability_authorization effect — same reading as flaw.pagan above.",
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
    // virtue.jurist, virtue.knight: moved to PENDING_DROPPED_CLAUSE (X2b) — F-123
    // (`docs/vf-audit/corrections.md`) is a live finding neither this row nor the
    // rest of this block's "male-only eligibility note is flavor" precedent
    // accounted for: D5 obliges the male-only restriction to reach the player as
    // TEXT even though D58/Q-05 (no sex model) means it is never computed. Both
    // entries' own ability_authorization clause is genuinely fully computed;
    // only that one clause was dropped from the certification (Knight's
    // equipment-access clause is additionally uncomputed, but that gap is K5/F0
    // engine work — `docs/vf-audit/design-f0-book-template-engine.md` — tracked
    // separately, not X2's). See tmp/x2b-verdicts.md.
    // virtue.mamluk, virtue.mazdean_priest, virtue.notary: moved to
    // PENDING_DROPPED_CLAUSE (X2c) — each entry's own male-only (or,
    // notary's, clergy/secular-law) restriction is a real, separately
    // uncomputed clause, not flavor, on the same D5/F-123 reading that
    // overturned virtue.jurist/virtue.knight in X2b: an eligibility note
    // reaches the player as TEXT even though D58/Q-05 (no sex model) means it
    // is never computed. Each entry's own ability_authorization clause is
    // genuinely fully computed; only that one clause was dropped from the
    // certification. See tmp/x2c-verdicts.md.
    (
        "virtue.master_of_form_creatures",
        "operative clause (\"may take\" Magic Lore) is computed via ability_authorization \
         (X1/D43); \"may be taken multiple times, once for each Form\" is already max_total:255.",
    ),
    (
        "virtue.master_of_kennels",
        "both stated clauses (50 XP pool on named abilities, \"may take Martial Abilities \
         freely\") are computed via restricted_ability_xp/ability_authorization (X1/D43); the \
         staff/privilege-of-riding/respect prose is flavor. Moved here from \
         PENDING_DROPPED_CLAUSE (X2c) — its own orphan comment predated X1's authorization work \
         and is now stale; re-reading finds no residue. See tmp/x2c-verdicts.md.",
    ),
    (
        "virtue.mercenary_captain",
        "operative clause (\"may take\" Martial Abilities) is computed via \
         ability_authorization (X1/D43); the Poor/Wealthy company-size prose names no signed \
         modifier.",
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
    // virtue.town_magistrate: moved to PENDING_DROPPED_CLAUSE (X2d) — the
    // Ability-3 prerequisite (F-322) is correctly out of this screen's reach
    // (a `prereq`-kind finding), but D62 (seasons are not modelled, so a
    // seasonal-commitment clause becomes text) applies to ArMDE:5151's "occupies
    // the character for two seasons each year, but he is free for the
    // remaining two seasons" — a real clause this certification did not
    // account for. See tmp/x2d-verdicts.md.
    // virtue.sufi: moved to PENDING_DROPPED_CLAUSE (X2d) — F-303 (a live
    // finding) disagrees with "not this row's concern": the no-points Story
    // Flaw rule is exactly the kind of uncomputed clause D5 obliges onto
    // `description`, regardless of whether a different finding first raised
    // it. See tmp/x2d-verdicts.md.
    // virtue.templar_administrator: moved to PENDING_DROPPED_CLAUSE (X2d) —
    // F-311 (a live finding) disagrees with "flavor": the Status-substitution
    // rule, the male-only restriction, and "no additional time" are each a
    // real clause reaching the player nowhere, on the same D5/F-123 reading
    // that overturned virtue.jurist/virtue.knight (X2b) and
    // virtue.mamluk/mazdean_priest/notary (X2c). See tmp/x2d-verdicts.md.
    (
        "virtue.troubadour",
        "operative clause (\"may take\" Academic skills) is computed via ability_authorization \
         (X1/D43); the Wealthy/Poor advisory note and companion Virtue suggestions are flavor.",
    ),
    // virtue.university_grammar_teacher: moved to PENDING_DROPPED_CLAUSE
    // (X2d) — this certification correctly reads "should have a score in
    // Teaching" as soft, but misses ArMDE:5197's own separate, hard
    // obligation: "They must teach two seasons out of the year." See
    // tmp/x2d-verdicts.md.
    (
        "flaw.black_sheep",
        "single clause (bad Reputation at level 2), fully computed via grants_reputation. \
         DE summary already states it, using \"Ruf\" rather than the rulebook's \"Reputation\" \
         loanword — a screen vocabulary gap (family 14 only recognizes \"Reputation der Stufe\"), \
         not a dropped rule.",
    ),
    // flaw.covenant_upbringing, flaw.deficient_technique,
    // flaw.difficult_longevity_ritual: resolved (X2e) — each certification
    // was wrong; a real clause reaches neither locale (F-395/F-399/F-420).
    // Removing the row is enough: `no_swept_entry_drops_an_uncomputed_
    // mechanical_clause` bites each directly (each still computes
    // *something*, so classification does not change). See
    // tmp/x2e-verdicts.md.
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
        "flaw.failed_monk",
        "all three stated clauses (bad Reputation 2 local, bad Reputation 2 ecclesiastical, \
         Academic-Ability authorization) are computed via two grants_reputation effects plus \
         ability_authorization; \"no longer need to observe monastic vows\" and the Failed-Nun \
         naming note are flavor. Moved here from PENDING_DROPPED_CLAUSE (X2f) — its own orphan \
         comment predated the authorization work and is now stale; re-reading finds no residue. \
         See tmp/x2f-verdicts.md.",
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
        "flaw.painful_magic",
        "X2g (tmp/x2g-verdicts.md): the operative clause (\"suffer the equivalent of one Fatigue \
         level in pain for each spell you cast\") is computed via health_mod/casting_fatigue \
         (amount -1); the newly-added description also states the clarifying clause (\"though \
         you do not suffer any physical damage from pain\") the finding wanted, but neither \
         sentence spells its number as a digit, so the screen's vocabulary does not recognize \
         either — a screen vocabulary gap, not a dropped rule.",
    ),
    (
        "flaw.poor",
        "the operative clause is computed via later_life_xp_rate (the shipped \
         wealthy_and_poor_ship_with_their_rates_and_eligibility test covers it), and the \
         displayed text already paraphrases it (\"one fewer season\") — a screen vocabulary \
         gap (no listed idiom for \"one fewer\"), not a dropped rule.",
    ),
    // flaw.poor_eyesight: OVERTURNED (X2g, `tmp/x2g-verdicts.md`, D61/OQ-4) —
    // this row's claim that "rolls involving sight" is fully covered by the
    // two combat_mod (attack/defense) effects is wrong: D61 rules the general
    // sight-roll penalty (a table call, like Poor Hearing/Sharp Ears/Keen
    // Vision) stays text, broader than the combat pair. Classification stays
    // in_play_effect; the missing clause is pinned by its own dedicated test,
    // `poor_eyesight_states_its_non_combat_sight_penalty` in
    // x2_reclassification.rs, rather than this generic list.
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
        "flaw.primogeniture_lineage",
        "X2g (tmp/x2g-verdicts.md), D67: moved here from NO_RULE_DESPITE_TOKEN now that the \
         entry reclassifies narrative -> creation_effect (ArMDE:6634-6637). The one genuinely \
         mechanical clause, \"This Flaw can only be taken by magi of House Verditius\", is fully \
         computed via `prerequisites: all(order_member, house.verditius)`; the newly-tripped \
         phrase (\"at least three places removed\") sits in the fictional-succession premise the \
         engine has no model of, and \"an interesting feature of her background\" is a turn of \
         phrase — neither is a rule this entry drops. No separate description is owed.",
    ),
    // flaw.short_ranged_magic: OVERTURNED (X2h, tmp/x2h-verdicts.md, D20/OQ-2)
    // — both halvings really are computed, but D20 rules that a numerically
    // correct surfaced effect is not enough: the rule must reach the player
    // as a computed number OR as text, and this entry displays a bare
    // category label with no description stating which two halvings apply.
    // Reclassifies in_play_effect -> uncomputed_rule (it already has a full
    // verbatim description in both locales, so this is classification-only).
    // See `x2h_d20_entries_reclassify_and_state_their_figures`
    // (`x2_reclassification.rs`).
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
    // virtue.lesser_power: moved to PENDING_DROPPED_CLAUSE (X2b) — F-142
    // (`docs/vf-audit/corrections.md`) is a live finding this row did not
    // account for: the Initiative formula (Quickness - twice power magnitude),
    // the Fatigue-cost scaling by level, and the one-for-one Penetration
    // purchase (ArMDE:4279-4286) reach the player nowhere. This departs from
    // X2a's "no action" call on the identically-shaped virtue.greater_power
    // (same gap, same live F-96 finding, apparently not read this closely) —
    // flagged here for the record, not fixed (out of this slice's range). See
    // tmp/x2b-verdicts.md.
    (
        "virtue.lightning_reflexes",
        "the operative bonus (+9 Initiative) is computed via combat_mod; the stress-die-plus-\
         Quickness trigger roll is table-time GM adjudication, the same shape botch dice are.",
    ),
    // virtue.linguist: moved to PENDING_DROPPED_CLAUSE (X2b) — F-150: ArMDE:4315's
    // "All Advancement Totals for any Language are increased by a quarter" is a
    // distinct in-play XP-gain-rate rule from "any experience points you put
    // into any language at character generation" (ArMDE:4316-4317), which is the
    // one `group_affinity_cost` actually implements (its own doc comment: "XP
    // put into any language 'counts as' num/den of itself"). The Advancement
    // Totals half reaches the player nowhere. See tmp/x2b-verdicts.md.
    // virtue.lone_redcap: moved to PENDING_DROPPED_CLAUSE (X2b) — F-151's `desc`
    // half: the two-seasons-of-service obligation ("or... declared Orbus"), the
    // Poor/Wealthy season-count interactions, and the mundane-Social-Status
    // compatibility note (ArMDE:4319-4326) reach the player nowhere; the base
    // numbers this row cites (Reputation, 300 XP, Well-Traveled) genuinely are
    // computed. F-151's `rep` half (the Reputation's polarity) is a possible
    // numeric-correctness defect, not a description gap — X7's concern, not
    // noted further here. See tmp/x2b-verdicts.md.
    // virtue.major_magical_focus: moved to PENDING_DROPPED_CLAUSE (X2b) — F-174:
    // the requisite rule ("the lowest applicable score may be one of the
    // requisites"), the lab-activity restriction ("cannot be focused on
    // laboratory activities, although a focus does apply to laboratory
    // activities"), and the breadth rule ("smaller than a single Art, but may be
    // spread over several Arts") reach the player nowhere (ArMDE:4399-4422); only
    // the doubling itself is computed. See tmp/x2b-verdicts.md.
    (
        "virtue.mastered_spells",
        "single clause (50 XP for spell mastery), fully computed via spell_mastery_xp; the \
         Flawless Magic compatibility note needs no effect.",
    ),
    // virtue.masterpiece, virtue.method_caster, virtue.personal_power: moved
    // to PENDING_DROPPED_CLAUSE (X2c) — each certification missed a real,
    // separately uncomputed clause: masterpiece's own vis-cost waiver
    // ("You ignore vis costs"), method_caster's own "if you vary at all...
    // you do not get this bonus" gate, and personal_power's own Initiative
    // formula/Fatigue-cost scaling/Penetration-purchase rule — the last
    // being exactly the reading that already overturned virtue.greater_power
    // (X2b) for the identically-worded "same reading as greater_power"
    // claim. See tmp/x2c-verdicts.md.
    (
        "virtue.mentored_by_demons",
        "both stated clauses (50 XP on any Ability, exceeding the age-based cap) are computed \
         via restricted_ability_xp and waives_ability_age_cap.",
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
    // virtue.sense_holiness_and_unholiness: OVERTURNED (X2d) — this
    // certification missed ArMDE:4928's "your sensitivity may overwhelm you"
    // in a strong divine/infernal aura, D50's own worked example (Q-66). D67
    // applies: reclassifies to uncomputed_rule despite the ability_score_grant
    // effect staying. See tmp/x2d-verdicts.md.
    // virtue.shadchan: moved to PENDING_DROPPED_CLAUSE (X2d) — F-267/F-268 are
    // live findings this certification did not account for. See
    // tmp/x2d-verdicts.md.
    (
        "virtue.spirit_votary",
        "the Second Sight grant is computed via grants_selection; the two-Virtue-points-per-\
         Flaw-point ratio is the mythic_companion type profile's own \
         virtue_points_per_flaw_point field, not this entry's job (D46's \"computed on a \
         profile\" shape).",
    ),
    // virtue.study_bonus: moved to PENDING_DROPPED_CLAUSE (X2d) — F-300 (a
    // live finding) disagrees with this row's "gating condition, not a
    // dropped rule" reading: the eight-row Art-Score table is itself
    // information the player needs (what counts as a qualifying
    // environment), not merely a condition the engine evaluates. See
    // tmp/x2d-verdicts.md.
    (
        "virtue.custos",
        "single clause (one restricted Ability group, exclusive choice), fully computed via a \
         gated ability_authorization (Phase 2 C1); \"may not take\" Wealthy/Poor is a separate, \
         unmodelled incompatibility this screen does not recognize as a mechanical token either \
         way.",
    ),
    // virtue.templar_commander: moved to PENDING_DROPPED_CLAUSE (X2d) — F-313
    // (a live finding) disagrees with "flavor with no number": the
    // taxation/tithe/service-fee/judge powers and the crusading obligation
    // are real, actionable clauses reaching the player nowhere. See
    // tmp/x2d-verdicts.md.
    // virtue.templar_office_holder: moved to PENDING_DROPPED_CLAUSE (X2d) —
    // F-315 (a live finding): the Templar-Status compatibility override is
    // stated twice (compatible with any Templar Status Virtue; compatible
    // with Temporal Influence) and encoded on neither side. See
    // tmp/x2d-verdicts.md.
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
    // virtue.unaging: moved to PENDING_DROPPED_CLAUSE (X2d) — F-332 (a live
    // finding): the crisis clause ("if a crisis is not potentially fatal, you
    // suffer no ill-effects... may die from terminal and potentially fatal
    // crises") and the Decrepitude-4/5 waiver reach the player nowhere as
    // text, whatever the aging_mod pair's engine-side semantics already
    // handle. See tmp/x2d-verdicts.md.
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
    // virtue.wise_one: moved to PENDING_DROPPED_CLAUSE (X2d) — the exclusive-
    // choice gate is genuinely fully computed (F-349's own concern), but
    // F-350 (a live finding) names three further clauses this certification
    // did not account for: the Wealthy/Poor normal-interaction clarification,
    // the male-and-female eligibility note, and the fear/awe/respect standing
    // description. See tmp/x2d-verdicts.md.
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
    // flaw.branded_criminal: resolved (X2e) — the Wealthy exclusion is now X4's
    // territory (`incompatible_with`), landed separately; this row's own
    // description obligation (the mark-in-cheek social consequence and the
    // martial-abilities choice reach neither locale) is real and unaffected
    // by that. Removing the row is enough —
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` bites it
    // directly (classification stays creation_effect). See
    // tmp/x2e-verdicts.md.
    // flaw.magical_fascination: resolved (X2g, `tmp/x2g-verdicts.md`) — the
    // Lore-1 cap orphan is real and still unaddressed; removing the row lets
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` bite it directly
    // (classification stays creation_effect; D5 obliges the cap into
    // description, full passage ArMDE:6392-6395).
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
         adds — a separate finding, ArMDE:4632-4634",
    ),
    (
        "virtue.priest",
        "orphan: \"If you are a parish priest, you cannot take the Poor Flaw\" is a conditional \
         prohibition the engine has no \"is a parish priest\" fact to gate on — distinct from \
         the F-340 blanket Wealthy/Poor exclusion family; the Academic-Ability permission itself \
         is computed via ability_authorization, ArMDE:4800",
    ),
    // virtue.turb_trained: resolved — the Wealthy/Poor exclusion landed
    // (X4, `incompatible_with`), and the single-dead-language grant now
    // carries a player-chosen `language` parameter bound to
    // `ability.dead_language` via `AbilityRef::Scoped` (X6b,
    // design-x6-parameters.md e5, ArMDE:5179-5182). Both clauses this row
    // recorded are computed; removed.
    // flaw.baneful_circumstances resolved (X7c, Phase 2, D46/D67): reclassified
    // `uncomputed_rule` with the full passage in `description` in both
    // locales, so it no longer trips this creation_effect/in_play_effect-
    // scoped screen — see tmp/x7c-verdicts.md.
    // flaw.bound_to_role_role: resolved (X2e) — the deprivation-check-as-food
    // clause and "may only be taken by grogs" (F-375, ArMDE:5735-5748) reach
    // neither locale; only the Unaging half is computed. Removing the row is
    // enough — `no_swept_entry_drops_an_uncomputed_mechanical_clause` bites
    // it directly (classification stays in_play_effect). See
    // tmp/x2e-verdicts.md.
    // `flaw.corrupted_arts` resolved (Phase 2 C5c, D15): reclassified
    // `uncomputed_rule` with the full passage in `description` in both
    // locales, so it no longer trips this creation_effect/in_play_effect-
    // scoped screen at all — it is now covered by
    // `every_uncomputed_rule_entry_states_its_rule_in_every_locale` instead.
    // flaw.creative_block resolved (X7c, Phase 2, D46/D67): reclassified
    // `uncomputed_rule` with the full passage in `description` in both
    // locales — see tmp/x7c-verdicts.md.
    // flaw.excommunicate, flaw.feral_scent, flaw.leprosy: resolved (X2f) —
    // each orphan is real and still unaddressed; removing the rows lets
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` bite each
    // directly (classification stays creation_effect/in_play_effect; D5
    // obliges the missing clause into description). See tmp/x2f-verdicts.md.
    // flaw.failed_monk: resolved (X2f) — this row's own finding (missing
    // ability_authorization) is now STALE: the shipped entry already carries
    // `ability_authorization categories:[academic]` alongside both
    // Reputations, so all three of the passage's mechanical clauses are
    // computed; the rest ("no longer need to observe monastic vows", "Female
    // characters may take this Flaw as Failed Nun") is flavor/naming, not a
    // rule. Same shape as virtue.master_of_kennels's own X2c move: moved to
    // COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE below. See tmp/x2f-verdicts.md.
    // flaw.imagined_folk_tradition_vulnerability: NOT added here (X2f) — its
    // real D3-inexpressible cap ("a score of 1 (but no more)" on Faerie Lore,
    // ArMDE:6280-6283, flagged by tmp/rules-md-audit-2026-09-30.md item 5, same
    // shape as flaw.magical_fascination above) is invisible to THIS list's own
    // validity check: the entry's shipped summary is the passage's own first
    // sentence, "...has other game mechanical effects as well" / "...hat...
    // auch andere spielmechanische Auswirkungen", which itself trips the
    // "named rulebook term" idiom pair (`game mechanical effects` /
    // `spielmechanische auswirkungen`) in both locales — a screen false
    // positive: the phrase *claims* an effect exists without stating it, yet
    // satisfies `displayed_text_states_a_mechanical_rule_in_every_locale`
    // regardless. `pending_dropped_clause_entries_still_trip_the_screen`
    // therefore refuses this row ("displayed text now states a mechanical
    // rule in every locale — delete the row"). See instead
    // `imagined_folk_tradition_vulnerability_states_its_faerie_lore_cap` in
    // x2_reclassification.rs, a dedicated test the screen cannot fool.
    // flaw.monstrous_blood: resolved (X2g) — Magic Animal's -3, Magic
    // Spirit's -3 Intelligence/Perception, and Magic Thing's Lesser Power
    // orphans are real and still unaddressed; removing the row lets
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` bite it directly
    // (classification stays in_play_effect; D5 obliges the full passage,
    // ArMDE:6454-6467, into description in both locales).
    // flaw.obese: resolved (X2g) — the non-Fatigue "-1 to rolls that involve
    // moving quickly or gracefully" orphan is real and still unaddressed;
    // removing the row lets the guard bite it directly (classification stays
    // in_play_effect; full passage ArMDE:6516-6519 owed as description).
    // flaw.outlaw: resolved (X2g) — despite the Martial-Abilities
    // ability_authorization and Reputation both already being computed, the
    // displayed text (summary only, no description) states neither figure in
    // any locale; removing the row lets the guard bite it directly
    // (classification stays creation_effect; full passage ArMDE:6542-6545
    // owed as description).
    // flaw.outlaw_leader: resolved (X2g) — same shape as flaw.outlaw: the
    // Reputation, ability_authorization, and the `is_grog` Nor-prerequisite
    // (X5, "Grogs may not take this Flaw") are all now computed, but no
    // locale displays any of it; removing the row lets the guard bite it
    // directly (classification stays creation_effect; full passage
    // ArMDE:6546-6549 owed as description).
    // flaw.savantism: resolved (X2g) as a full reclassification, not just a
    // description fix — see X2G_RECLASSIFY_WITH_DESCRIPTION in
    // x2_reclassification.rs (D67: the halved starting XP, halved future
    // Advancement Totals, and +3-not-+1 specialization roll, F-510 points
    // 1-3, are computed nowhere, so the whole entry becomes uncomputed_rule
    // regardless of the two ability_score_cap_* effects X6b already gave it).
    // Removed here because a PDC row requires creation_effect/in_play_effect,
    // which this entry no longer is once Phase 2 lands.
    // flaw.the_constant_expression: resolved (X2e) — the permanent lost
    // Fatigue level, the Concentration roll to suppress (Ease Factor 3 +
    // Warping), the extra botch dice on Ceremonial/Ritual casting, and the
    // free -3 lab Safety Flaw (F-392, ArMDE:5821-5838) all reach neither
    // locale; only a bare "circumstantial" marker is computed. D58 governs:
    // the surfaced-modifier family stays computed as-is (a doubling/marker
    // still cannot be computed where the engine does not know the base) —
    // classification stays in_play_effect, description is owed. Removing the
    // row is enough — `no_swept_entry_drops_an_uncomputed_mechanical_clause`
    // bites it directly. See tmp/x2e-verdicts.md.
    // flaw.usurer, flaw.warped_by_magic, flaw.weak_enchanter, flaw.weak_magic,
    // flaw.weak_scholar, flaw.weak_spontaneous_magic: resolved (X2h,
    // tmp/x2h-verdicts.md, F-544 group) — all six get a description stating
    // their own dropped clause, classification unchanged.
    // `no_swept_entry_drops_an_uncomputed_mechanical_clause` bites each
    // directly now that the row is gone.
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
    // flaw.painful_magic: resolved (X2g) — the "do not suffer any physical
    // damage from pain" clarifying clause is real and still unaddressed;
    // removing the row lets the guard bite it directly (classification stays
    // in_play_effect; full passage ArMDE:6574-6577 owed as description).
    (
        "virtue.simple_student",
        "eligibility: \"Female characters can only take this Virtue if they are studying to be \
         physicians at Salerno\", ArMDE:4958-4963 — `creation_effect` (a \
         scaled_restricted_ability_xp effect computes the XP clause), but this gender/location \
         restriction is not computed and reaches the player nowhere",
    ),
    // --- S4 additions (docs/vf-audit/phase-2-plan.md, Phase 1S): SWEPT_BLOCKS
    // widened to the whole catalogue, newly sweeping ArMDE:3951-5638.
    // virtue.inventive_genius resolved (X7c, Phase 2, D46/D67): reclassified
    // `uncomputed_rule` with the full passage in `description` in both
    // locales — see tmp/x7c-verdicts.md.
    // virtue.leper_magus, virtue.life_boost: removed (X2b, D20) — both are two
    // of D20's five "number missing" surfaced-only entries
    // (`docs/vf-audit/decisions.md` D20), so a description alone does not
    // resolve them while they stay in_play_effect: both reclassify fully to
    // uncomputed_rule, with dedicated numeric tests in `x2_reclassification.rs`
    // (mirroring X2a's virtue.commanding_aura). See tmp/x2b-verdicts.md.
    // virtue.magian_lineage_major/_minor resolved (X7c, Phase 2, D46/D67):
    // both reclassified `uncomputed_rule` with the full passage (including the
    // disease-resistance clause) in `description` in both locales — the
    // Major's connected-Abilities XP-sharing clause remains a separate, still
    // open gap (`tmp/x6-scope.md` § 1 "M-group", F-157/F-158) that this fix
    // does not touch. See tmp/x7c-verdicts.md.
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
        "orphan (X2b widened this reading — F-170 alongside the existing F-171): the \
         age-and-Ability-score eligibility floor (25-Int years, Latin/Artes Liberales 5) and the \
         two-seasons teaching obligation have no prerequisite/effect, and \"only available to \
         male characters\" (F-170) reaches the player nowhere either — only the Reputation and \
         the XP grant are computed, ArMDE:4385-4394",
    ),
    // virtue.greater_power, virtue.jurist, virtue.knight, virtue.lesser_power,
    // virtue.linguist, virtue.lone_redcap, virtue.major_magical_focus: resolved
    // (X2b Phase 2) — each now ships the full cited passage as its
    // `description` in both locales, which by construction restates whatever
    // token got the row flagged onto this list in the first place; removed
    // from here (own class unchanged: creation_effect except major_magical_focus,
    // which stays in_play_effect). virtue.knight's equipment-access clause is
    // still real engine work (K5/F0,
    // `docs/vf-audit/design-f0-book-template-engine.md`) — the text now states
    // it (D5), but computing it is still out of X2's scope. See
    // tmp/x2b-verdicts.md.
    // virtue.marshal: gap closed (X2e, 2026-09-30, collateral effect of
    // widening the DE permission-modal idiom to cover "nehmen") — its DE
    // description's "...aufteilen darf, und kann Kampffertigkeiten frei
    // nehmen" now matches the modal+"nehmen" pattern added for
    // flaw.ability_block/flaw.difficult_underlings, so the row's own recorded
    // gap ("no residue beyond the already-computed XP grant and Martial
    // Abilities authorization") is moot: the text states a real permission
    // clause the screen now sees. Removed per shrink-only list semantics.
    // virtue.master_of_kennels: moved to COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE
    // (X2c) — its own orphan comment above predated X1's authorization work
    // and is now stale; re-reading finds no residue beyond flavor. See
    // tmp/x2c-verdicts.md.
    // virtue.mamluk, virtue.mazdean_priest, virtue.masterpiece,
    // virtue.method_caster, virtue.notary, virtue.personal_power: resolved
    // (X2c Phase 2) — each now ships the full cited passage as its
    // `description` in both locales, restating the clause that got each row
    // flagged onto this list in the first place; removed from here (own
    // classification unchanged for every one of the six). See
    // tmp/x2c-verdicts.md.
    (
        "virtue.mercurian_magic",
        "orphan: the Wizard's Vigil auto-knowledge and the Mastery-score stacking still have no \
         effect — only a bare \"mercurian\" marker is computed. The required companion Flaw \
         (Ceremonial Spontaneous Magic, \"also have\") is resolved (X5a/D51 row 1): a \
         `prerequisites` gate, not a grant, ArMDE:4514-4523",
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
        "virtue.senior_clergy",
        "orphan (X2d corrected this reading — the Academic-Abilities claim is stale: X1 already \
         added it): the Wealthy/Poor choice note and the Abbess-only/not-ordained women's \
         restriction have no effect at all — only the two Reputations and the Academic \
         authorization are computed, ArMDE:4910-4921",
    ),
    (
        "virtue.strong_angelic_heritage",
        "orphan: the age-scaled Divine Might formula (age / 20), the vis-on-death yield, and \
         Warping immunity are not visibly keyed to age or otherwise computed beyond a flat \
         might_grant score of 0, ArMDE:5022-5031",
    ),
    // virtue.strong_faerie_blood resolved (X7c, Phase 2, D46/D67): reclassified
    // `uncomputed_rule` with the full passage in `description` in both
    // locales — see tmp/x7c-verdicts.md.
    // --- X2d (tmp/x2d-verdicts.md, 2026-09-29): ten rows moved here from
    // COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE — each certification missed a real,
    // live-finding-backed residue. virtue.shadchan's gap was closed in Phase 2
    // (description now states the compatibility override) and removed from
    // this list.
    // virtue.study_bonus, virtue.templar_administrator, virtue.templar_commander,
    // virtue.templar_office_holder, virtue.university_grammar_teacher, virtue.sufi,
    // and virtue.wise_one: all had their gaps closed in Phase 2 (each now carries
    // a description stating its residue) and were removed from this list.
    // --- X2c Phase 2 fix-round (2026-09-29): the new "eligibility" idiom
    // ("only available to ... characters") this slice added to catch its own
    // mamluk/mazdean_priest/rabbi finding newly sweeps these two entries too
    // — both outside X2c's own range (`tmp/x2-worklist.md` rows 103-153),
    // real dropped clauses, not this slice's to fix. Filed here per the same
    // D5 shape rather than left silently uncaught.
    (
        "virtue.brother_sergeant",
        "orphan: \"This Virtue is only available to male characters\" reaches the player \
         nowhere — the Martial Abilities authorization is genuinely computed, ArMDE:3537-3540",
    ),
    (
        "virtue.trained_assassin",
        "orphan: \"This Virtue is only available to characters with one of the Social Status \
         Virtues of the Nizaris\" (an eligibility restriction on another Virtue, not a \
         sex-eligibility one) reaches the player nowhere — the 50 XP grant is genuinely \
         computed, ArMDE:5153-5156",
    ),
    // --- X2d fix-round (2026-09-30): the new "wealthy/poor unaffected" idiom
    // (added to close virtue.wanderer's own screen miss) newly sweeps these
    // four `creation_effect` entries too — none on X2d's own worklist (rows
    // 154-203), each a genuine dropped clause for a later X2 slice.
    (
        "virtue.clerk",
        "orphan: \"The Wealthy Virtue and Poor Flaw affect you normally\" reaches the player \
         nowhere — the Academic Abilities authorization is genuinely computed, \
         ArMDE:3571-3574",
    ),
    (
        "virtue.failed_apprentice",
        "orphan: \"The Wealthy Virtue and Poor Flaw affect you normally\" reaches the player \
         nowhere — the Academic/Arcane/Martial Abilities authorization is genuinely computed, \
         ArMDE:3843-3846",
    ),
    (
        "virtue.priest",
        "orphan: \"the Wealthy Virtue and Poor Flaw affect you normally\" reaches the player \
         nowhere — the Academic Abilities authorization is genuinely computed, \
         ArMDE:4796-4805",
    ),
    (
        "virtue.troubadour",
        "orphan: \"The Wealthy Virtue and Poor Flaw affect you normally\" reaches the player \
         nowhere — the Academic Abilities authorization is genuinely computed, \
         ArMDE:5157-5164",
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
