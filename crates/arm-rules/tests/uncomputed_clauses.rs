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
//! carries a mechanical token is a dropped rule"*, is **not** here yet. It is
//! the expensive half: the core rulebook's Flaws block alone holds roughly 43
//! `narrative` entries whose passage states a hard clause, so landing it today
//! would put ~43 reds on `main` and the only way to green would be a 60-row
//! exemption list — a backlog with a test around it. It lands once the data is
//! clean, per book.
//!
//! # Scope
//!
//! The assertion below covers **every** `uncomputed_rule` entry, in every book,
//! deliberately unscoped — because unlike the deferred third assertion it has no
//! backlog to work through. Writing an entry's rule into both locales is the
//! same act as classifying it `uncomputed_rule`, so a book-by-book scope here
//! would buy nothing and would silently skip whatever it excluded. The
//! incremental-roots pattern `rulebook_citations.rs` uses is what the *third*
//! assertion will need, when it lands.
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

/// True when `s` contains a signed number — `+3`, `-9`. In these books the
/// signed form is a modifier essentially without exception; unsigned numbers
/// ("1 pawn of vis", "an Ease Factor of 9") are where the noise lives, so
/// requiring the sign is what keeps the token sharp.
fn has_signed_number(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if (c == '-' || c == '+') && chars.peek().is_some_and(char::is_ascii_digit) {
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

/// Every `uncomputed_rule` entry in the catalogue, as `id`.
fn uncomputed_rule_ids() -> Vec<String> {
    let path = rules_dir().join("core/virtues_flaws.json");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
    let items: Vec<Value> = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{} is valid JSON: {e}", path.display()));

    items
        .iter()
        .filter(|item| item["classification"] == "uncomputed_rule")
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
    let ids = uncomputed_rule_ids();
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
