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
//! at least one **mechanical token** — a signed number, or a botch-dice term.
//! Reclassifying an entry into `uncomputed_rule` therefore costs more than
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

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// Rulebook terms for the botch-dice mechanic, lowercased. `botch` covers
/// "botch die"/"botch dice"/"magical botch"; `patzer` covers "Patzerwürfel" and
/// "Patzer". Both are rulebook proper terms, never loose prose.
const BOTCH_TERMS: &[&str] = &["botch", "patzer"];

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
/// form is a modifier essentially without exception; unsigned numbers ("1 pawn
/// of vis", "an Ease Factor of 9") are where the noise lives, so requiring the
/// sign is what keeps the token sharp.
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

/// The guard's detector: does this rules text actually state a mechanical rule?
fn states_a_mechanical_rule(text: &str) -> bool {
    has_signed_number(text) || has_botch_term(text)
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
         these carry no mechanical token (no signed number, no botch-dice term). \
         Either the clause was dropped when the text was written, or the entry is \
         not really `uncomputed_rule`:\n{}",
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
const SWEPT_BLOCKS: &[(&str, i64, i64)] = &[(
    // The Flaws block: `## Flaws` to the end of `#### Wrathful`,
    // ArMDE:5639-7113. Swept 2026-09-15.
    "Ars Magica - Definitive Edition (Core Rules).md",
    5639,
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
         mechanical rule (a signed modifier or a botch-dice clause). `narrative` means the \
         RULEBOOK says nothing mechanical, not merely that the engine computes nothing — so \
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

    // An unsigned number is not a modifier — this is the exclusion that keeps
    // the signed-number token sharp.
    assert!(!states_a_mechanical_rule("1 pawn of vis each season"));
    assert!(!states_a_mechanical_rule("an Ease Factor of 9"));

    // A hyphen that is not a sign, and prose with no mechanics at all.
    assert!(!states_a_mechanical_rule("good hand-eye coordination"));
    assert!(!states_a_mechanical_rule("Roleplay your clumsiness."));
    assert!(!states_a_mechanical_rule(
        "You are a full member of the Order of Hermes."
    ));
}
