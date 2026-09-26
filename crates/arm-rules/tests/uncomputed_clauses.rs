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
//! The third assertion of the set, *"a `narrative` entry whose cited passage
//! carries a mechanical token is a dropped rule"*, is
//! [`no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause`]. It is the
//! expensive half — every red is a hand extraction out of two rulebooks — so it
//! is **scoped by swept block** ([`SWEPT_BLOCKS`]) and widens as the sweep
//! proceeds, exactly as `rulebook_citations.rs`'s roots function does. Landing
//! it unscoped would have meant ~90 reds and the only route to green would have
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
/// Markdown writes negatives with U+2013 EN DASH and occasionally U+2212, so a
/// detector pointed at a rulebook passage must read those too. Listing them
/// here rather than in two detectors keeps one definition of "a sign".
const SIGN_CHARS: &[char] = &['-', '+', '\u{2013}', '\u{2212}'];

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
fn has_signed_number(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if SIGN_CHARS.contains(&c) && chars.peek().is_some_and(char::is_ascii_digit) {
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
///   [`regex_matcher_reproduces_the_recorded_s1_before_set`] proves.
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
    "darf nicht größer sein als",
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
/// [`regex_matcher_reproduces_the_recorded_s1_before_set`]). A phrase crossing
/// a word boundary is a defect in the screen, not an idiom to argue down in a
/// `NO_RULE_DESPITE_TOKEN` row.
///
/// There is deliberately **no** boundary on the right, so a phrase still
/// matches through a suffix — which is how the `multipli` stem earns its
/// keep.
fn has_mechanical_phrase(s: &str) -> bool {
    MECHANICAL_PHRASE_PATTERNS.iter().any(|re| re.is_match(s))
}

/// The guard's detector: does this rules text actually state a mechanical rule?
fn states_a_mechanical_rule(text: &str) -> bool {
    has_signed_number(text) || has_botch_term(text) || has_mechanical_phrase(text)
}

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

/// The blocks of source Markdown whose `narrative` entries have been swept —
/// every cited passage read, every mechanical clause either written into both
/// locales (and the entry reclassified) or confirmed absent.
///
/// Incremental **by design**, mirroring `rulebook_citations.rs`'s roots
/// function: the sweep is a hand extraction out of two rulebooks, so landing
/// [`no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause`] over the
/// whole catalogue at once would have meant ~90 simultaneous reds and an
/// exemption list as the only route to green. A row is added here — never
/// removed — when its block is clean, so the guard's coverage only ever grows
/// and a regression inside an already-swept block fails immediately.
///
/// `(file, first line, last line)`, inclusive, against `rules/source/en/`.
const SWEPT_BLOCKS: &[(&str, i64, i64)] = &[
    (
        // The Flaws block: `## Flaws` to the end of `#### Wrathful`,
        // ArMDE:5639-7113. Swept 2026-09-15; re-swept 2026-09-19 under the
        // widened screen, which found 13 more — the first sweep could only see
        // a signed number or a botch die, so every rule the book states in
        // words read as flavour.
        "Ars Magica - Definitive Edition (Core Rules).md",
        5639,
        7113,
    ),
    (
        // The head of the Virtues block: `## Virtues` to the end of
        // `#### Frightful Presence`, ArMDE:3360-3950. Swept 2026-09-18,
        // re-swept 2026-09-19 with the same widening (8 more). The rest of the
        // block (ArMDE:3951-5282) is still unswept.
        "Ars Magica - Definitive Edition (Core Rules).md",
        3360,
        3950,
    ),
];

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
         `prerequisites: all(is_magus, house.verditius)`, so the rule is enforced rather than \
         merely described, and describing it again would not be `uncomputed_rule`.",
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
];

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

/// The third assertion: **a `narrative` entry whose cited passage states a
/// mechanical rule has dropped that rule.**
///
/// `narrative` means "pure personality, story, or social-status flavor" — a
/// statement about the *rulebook*, not merely about what the engine computes.
/// So an entry classified `narrative` whose own passage carries a signed
/// modifier or a botch-dice clause is one of two bugs, and both are real: either
/// the rule reaches the player nowhere (the engine models nothing *and* the
/// displayed text, capped at the summary's first sentence, omits it), or the
/// entry is misclassified and belongs in `uncomputed_rule` / `in_play_effect`.
///
/// This is the assertion that makes the sweep *checkable* rather than claimed.
/// Without it, "the Flaws block is clean" is a sentence in a report; with it,
/// re-dirtying the block is a failing build.
#[test]
fn no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause() {
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut checked = 0usize;
    let mut offenders = Vec::new();

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
        checked += 1;
        let id = item["id"].as_str().expect("every entry has a string id");
        if NO_RULE_DESPITE_TOKEN
            .iter()
            .any(|(exempt, _)| *exempt == id)
        {
            continue;
        }
        let Some(passage) = bracketed_passage(&mut cache, &file, start, end) else {
            continue;
        };
        if states_a_mechanical_rule(&passage) {
            offenders.push(format!("{id} ({file}:{start}-{end})"));
        }
    }

    assert!(
        checked > 50,
        "the swept-block filter matched only {checked} narrative entries, so this guard is \
         checking almost nothing — has a SWEPT_BLOCKS range or a source file name drifted?"
    );

    assert!(
        offenders.is_empty(),
        "{} `narrative` Virtue/Flaw(s) inside a swept block cite a passage that states a \
         mechanical rule (a signed modifier, a botch-dice clause, or a phrase-shaped idiom — \
         a cap, a target number, a formula, a rounding direction, an absolute). `narrative` \
         means the RULEBOOK says nothing mechanical, not merely that the engine computes \
         nothing — so \
         each of these has either dropped a rule the player never sees, or is misclassified \
         and belongs in `uncomputed_rule` (fill `description` in every locale) or \
         `in_play_effect`:\n{}",
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
/// [`regex_matcher_reproduces_the_recorded_s1_before_set`] need, computed
/// against whichever [`states_a_mechanical_rule`] is compiled in right now.
/// Run against the pre-conversion substring matcher, this produced the
/// committed `tests/fixtures/s1_before_offenders.json`; run against the
/// regex matcher, it must reproduce that fixture exactly.
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
/// [`regex_matcher_reproduces_the_recorded_s1_before_set`] below. This is a
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

/// D19's obligation #1 (`docs/vf-audit/decisions.md`): the regex conversion
/// must reproduce **exactly** the before-set recorded from the substring
/// matcher, on the real shipped catalogue — not a hand-picked sample. A
/// widened or narrowed screen would change which entries the two guards
/// above see, invalidating the measurements B11, B12 and B17 took, and
/// widening what the screen can see is S2's job, not this slice's.
#[test]
fn regex_matcher_reproduces_the_recorded_s1_before_set() {
    let (expected_narrative, expected_uncomputed) = recorded_s1_before_set();
    let (actual_narrative, actual_uncomputed) = compute_s1_offender_set();

    let mut mismatches = Vec::new();

    for (id, expected) in &expected_narrative {
        match actual_narrative.get(id) {
            None => mismatches.push(format!(
                "narrative_in_swept_blocks/{id}: recorded but missing from today's catalogue"
            )),
            Some(actual) if actual != expected => mismatches.push(format!(
                "narrative_in_swept_blocks/{id}: expected {expected}, got {actual}"
            )),
            Some(_) => {}
        }
    }
    for id in actual_narrative.keys() {
        if !expected_narrative.contains_key(id) {
            mismatches.push(format!(
                "narrative_in_swept_blocks/{id}: in today's catalogue but not in the recorded set"
            ));
        }
    }

    for (key, expected) in &expected_uncomputed {
        match actual_uncomputed.get(key) {
            None => mismatches.push(format!(
                "uncomputed_rule_by_locale/{key}: recorded but missing from today's catalogue"
            )),
            Some(actual) if actual != expected => mismatches.push(format!(
                "uncomputed_rule_by_locale/{key}: expected {expected}, got {actual}"
            )),
            Some(_) => {}
        }
    }
    for key in actual_uncomputed.keys() {
        if !expected_uncomputed.contains_key(key) {
            mismatches.push(format!(
                "uncomputed_rule_by_locale/{key}: in today's catalogue but not in the recorded set"
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "D19 S1 requires the regex conversion to be INERT: it must flag exactly the entries the \
         substring matcher flagged (tests/fixtures/s1_before_offenders.json), before any new \
         phrase family is added — that widening is slice S2, not this one. Disagreements:\n{}",
        mismatches.join("\n")
    );
}
