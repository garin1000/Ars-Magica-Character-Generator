//! G1: `flaw.poor_characteristic`'s summary text used the mathematical minus
//! U+2212 ("already at −3 down to −5") instead of the ASCII hyphen-minus
//! U+002D, in both `rules/i18n/en/virtues_flaws.json` and
//! `rules/i18n/de/virtues_flaws.json` — breaching `CLAUDE.md`'s "Negative
//! signs are ASCII hyphens" rule for rules text specifically (the rule's own
//! `formatSigned()` half is guarded by the frontend suite; nothing guarded
//! the rules-i18n half). Fixing the one known instance without a mechanical
//! check leaves the same mistake free to recur the next time someone
//! authors or edits rules text with a word processor or an editor that
//! auto-substitutes a "proper" minus sign.
//!
//! This test walks every string value in every `rules/i18n/**/*.json` file —
//! discovered rather than hardcoded, so it scales with however many
//! languages/domains exist — and fails on the first U+2212 it finds, naming
//! the file and a snippet of the offending string.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const MATH_MINUS: char = '\u{2212}';

fn rules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules")
}

/// Every `rules/i18n/<lang>/*.json` file, discovered via `fs::read_dir` over
/// each language directory rather than a hardcoded file list or language
/// list — a new language or a new domain file is picked up automatically.
fn i18n_json_files() -> Vec<PathBuf> {
    let i18n_dir = rules_dir().join("i18n");
    let mut files = Vec::new();
    for lang_entry in
        fs::read_dir(&i18n_dir).unwrap_or_else(|e| panic!("rules/i18n is readable: {e}"))
    {
        let lang_path = lang_entry.unwrap().path();
        if !lang_path.is_dir() {
            continue;
        }
        for file_entry in fs::read_dir(&lang_path)
            .unwrap_or_else(|e| panic!("{} is readable: {e}", lang_path.display()))
        {
            let path = file_entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "json") {
                files.push(path);
            }
        }
    }
    files.sort();
    assert!(
        !files.is_empty(),
        "expected at least one rules/i18n/<lang>/*.json file to exist"
    );
    files
}

/// Recursively collects every string in `value` that contains the
/// mathematical minus U+2212, formatted as `"<snippet>"` for the failure
/// message.
fn find_math_minus_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            if s.contains(MATH_MINUS) {
                out.push(s.clone());
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                find_math_minus_strings(v, out);
            }
        }
        Value::Array(items) => {
            for v in items {
                find_math_minus_strings(v, out);
            }
        }
        _ => {}
    }
}

#[test]
fn no_rules_i18n_text_uses_the_mathematical_minus() {
    let mut offenders: Vec<String> = Vec::new();

    for path in i18n_json_files() {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
        let value: Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{} is valid JSON: {e}", path.display()));
        let mut found = Vec::new();
        find_math_minus_strings(&value, &mut found);
        for s in found {
            offenders.push(format!("{}: {s:?}", path.display()));
        }
    }

    assert!(
        offenders.is_empty(),
        "found U+2212 (mathematical minus) in rules i18n text — use the ASCII \
         hyphen-minus U+002D instead (CLAUDE.md: \"Negative signs are ASCII \
         hyphens\"):\n{}",
        offenders.join("\n")
    );
}
