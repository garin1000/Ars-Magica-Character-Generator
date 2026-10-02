//! Row 38 (`docs/open-todos.md`): the German rulebook sources close a German
//! opening quote „ (U+201E) with an ASCII `"` rather than the correct German
//! closing quote " (U+201C) — 1425-to-6 across `rules/source/de/`, which makes
//! it the corpus's house style, not a slip (see `docs/open-todos.md`, "German
//! quote glyphs are the corpus's house style"). `rules/source/de/` is never
//! touched for it. The policy instead is that the pairing is normalized **on
//! the way out**: shipped `rules/i18n/de/*.json` rules text, and hand-authored
//! `locales/de/*.ftl` UI text, must pair every „ with a proper ", never with
//! the ASCII `"`.
//!
//! This test walks every string value in every `rules/i18n/de/*.json` file —
//! parsed with `serde_json`, never scanned as raw bytes, so it cannot be
//! fooled by a quote glyph split across an escape sequence — plus the raw
//! text of every `locales/de/*.ftl` file, and fails on the first `„…"` pair it
//! finds, naming the file, the field path (or line) it came from, and the
//! offending snippet.
//!
//! Quotes are tracked with a stack rather than a single flag so a nested
//! `„…„…"…"` is handled correctly: each closing glyph (either `"` or the
//! correct `"`) closes the innermost still-open `„`. A `„` with no closing
//! glyph at all in the same string is not flagged — only an actual
//! ASCII-closed pair is a defect.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const OPEN: char = '\u{201E}'; // „
const CLOSE: char = '\u{201C}'; // "
const ASCII_QUOTE: char = '"';

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `rules/i18n/de/*.json` file, discovered via `fs::read_dir` rather
/// than a hardcoded list — a new file in that directory is picked up
/// automatically.
fn de_i18n_json_files() -> Vec<PathBuf> {
    let dir = repo_root().join("rules/i18n/de");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "expected at least one rules/i18n/de/*.json file to exist"
    );
    files
}

/// Every `locales/de/*.ftl` file, discovered the same way.
fn de_ftl_files() -> Vec<PathBuf> {
    let dir = repo_root().join("locales/de");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "ftl"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "expected at least one locales/de/*.ftl file to exist"
    );
    files
}

/// Finds every `„…"` pair in `s` — a „ (U+201E) closed by an ASCII `"` rather
/// than the correct " (U+201C) — returning the exact substring from the
/// opening glyph to the offending close, for the failure message.
fn find_ascii_closed_pairs(s: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    let mut opens: Vec<usize> = Vec::new();
    for (idx, c) in s.char_indices() {
        match c {
            OPEN => opens.push(idx),
            CLOSE => {
                opens.pop();
            }
            ASCII_QUOTE => {
                if let Some(start) = opens.pop() {
                    offenders.push(s[start..idx + ASCII_QUOTE.len_utf8()].to_string());
                }
            }
            _ => {}
        }
    }
    offenders
}

/// Recursively walks a JSON value, applying `find_ascii_closed_pairs` to
/// every string, and recording `(dotted field path, snippet)` for each hit.
fn walk_json(value: &Value, path: &str, out: &mut Vec<(String, String)>) {
    match value {
        Value::String(s) => {
            for snippet in find_ascii_closed_pairs(s) {
                out.push((path.to_string(), snippet));
            }
        }
        Value::Object(map) => {
            for (key, v) in map {
                walk_json(v, &format!("{path}.{key}"), out);
            }
        }
        Value::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                walk_json(v, &format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}

#[test]
fn de_i18n_json_never_closes_a_german_opening_quote_with_an_ascii_quote() {
    let mut offenders: Vec<String> = Vec::new();

    for path in de_i18n_json_files() {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
        let value: Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{} is valid JSON: {e}", path.display()));
        let mut found = Vec::new();
        walk_json(&value, "", &mut found);
        for (field_path, snippet) in found {
            offenders.push(format!("{}:{field_path}: {snippet:?}", path.display()));
        }
    }

    assert!(
        offenders.is_empty(),
        "found a German opening quote „ closed by an ASCII \" instead of \" in \
         rules/i18n/de — normalize the closing glyph on the way out \
         (docs/open-todos.md row 38, \"German quote glyphs are the corpus's \
         house style\"):\n{}",
        offenders.join("\n")
    );
}

/// `.ftl` is UI text, but not every line in the file ships: a `#`-prefixed
/// line is a developer comment, never rendered, so it is out of scope for the
/// same reason `rules/source/de/` itself is out of scope — it is not shipped
/// text the normalize-on-the-way-out policy applies to. Only actual message
/// lines (the key/value lines Fluent compiles into UI strings) are checked.
#[test]
fn de_ftl_never_closes_a_german_opening_quote_with_an_ascii_quote() {
    let mut offenders: Vec<String> = Vec::new();

    for path in de_ftl_files() {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
        for (line_no, line) in text.lines().enumerate() {
            if line.trim_start().starts_with('#') {
                continue;
            }
            for snippet in find_ascii_closed_pairs(line) {
                offenders.push(format!("{}:{}: {snippet:?}", path.display(), line_no + 1));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "found a German opening quote „ closed by an ASCII \" instead of \" in \
         a locales/de/*.ftl message (docs/open-todos.md row 38):\n{}",
        offenders.join("\n")
    );
}
