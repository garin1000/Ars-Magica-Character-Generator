//! V22: nothing previously checked that `rules/core/*.json` stays in sync
//! with `rules/source/**` — the authoritative, human-authored Markdown
//! (`CLAUDE.md` → "Rules source files" / "Rules provenance"). Drift (a
//! citation left stale after the source Markdown was edited, a `source`
//! block naming the wrong file, a line range shifted out of bounds) would go
//! undetected until a user noticed wrong rules data.
//!
//! Deliberately bounded: this does **not** re-derive the catalogue or
//! re-parse the rulebooks into structured data (fragile, out of proportion —
//! see `CLAUDE.md` → "Catalogue size is data, never code" and the
//! `rules/core/` extraction note). Instead it walks every `source: { file,
//! lines: [start, end] }` block that already exists in `rules/core/*.json`
//! — whatever catalogue it is in, however many there are — and asserts each
//! one actually brackets real content in its named source file: the file
//! exists, the range is well-formed and in-bounds, and the bracketed lines
//! are not blank. This catches the concrete failure modes drift produces
//! (renamed/moved source file, an edit that shifted everything below it so
//! the old line numbers now point at the wrong passage or past EOF, a
//! transposed start/end) without asserting anything about catalogue *size*.
//!
//! `CLAUDE.md` → "Rules provenance": `source` in `rules/core/` always points
//! at the **English** file, so every reference below resolves against
//! `rules/source/en/`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// `crates/arm-rules` (this crate's manifest dir) -> repo root -> `rules/`.
fn rules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules")
}

/// One `source` block found somewhere in a `rules/core/*.json` file, plus
/// enough context to name it in a failure message.
struct FoundSourceRef {
    /// The `rules/core/<name>.json` file it was found in (for error messages).
    core_file: String,
    /// The sibling `"id"` of the object the `source` block sits on, if any
    /// (most catalogue entries have one; falls back to a positional label).
    entry_label: String,
    source_file: String,
    start: i64,
    end: i64,
}

/// Recursively walks a parsed JSON document looking for `source: { "file":
/// ..., "lines": [start, end] }` blocks — the shape [`SourceRef`] serializes
/// to (see `crates/arm-rules/src/types.rs`). Generic over which catalogue it
/// is walking: every `rules/core/*.json` file uses the same shape for
/// provenance, so one walker covers virtues/flaws, abilities, arts, spells,
/// spell mastery abilities, houses, mythic companion types, equipment,
/// childhoods, and aging rows alike, with zero per-catalogue code.
fn collect_source_refs(core_file: &str, value: &Value, out: &mut Vec<FoundSourceRef>) {
    match value {
        Value::Object(map) => {
            if let Some(source) = map.get("source")
                && let (Some(file), Some(lines)) = (
                    source.get("file").and_then(Value::as_str),
                    source.get("lines").and_then(Value::as_array),
                )
                && let [start, end] = lines.as_slice()
                && let (Some(start), Some(end)) = (start.as_i64(), end.as_i64())
            {
                let entry_label = map
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("<no id, cites {file}:{start}-{end}>"));
                out.push(FoundSourceRef {
                    core_file: core_file.to_string(),
                    entry_label,
                    source_file: file.to_string(),
                    start,
                    end,
                });
            }
            for nested in map.values() {
                collect_source_refs(core_file, nested, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_source_refs(core_file, item, out);
            }
        }
        _ => {}
    }
}

/// Validates one [`FoundSourceRef`] against the actual Markdown file, pushing
/// a descriptive message onto `errors` for every way it can be stale:
/// malformed range, missing file, out-of-bounds range, or a range that
/// brackets nothing but blank lines.
fn validate_source_ref(source_dir: &Path, found: &FoundSourceRef, errors: &mut Vec<String>) {
    let FoundSourceRef {
        core_file,
        entry_label,
        source_file,
        start,
        end,
    } = found;

    if *start < 1 || end < start {
        errors.push(format!(
            "{core_file}: \"{entry_label}\" cites {source_file}:{start}-{end}, which is not a \
             well-formed 1-based inclusive range"
        ));
        return;
    }

    let path = source_dir.join(source_file);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            errors.push(format!(
                "{core_file}: \"{entry_label}\" cites source file {source_file}, which could not \
                 be read at {path:?}: {e}"
            ));
            return;
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len() as i64;
    if *end > total {
        errors.push(format!(
            "{core_file}: \"{entry_label}\" cites {source_file}:{start}-{end}, but the file has \
             only {total} lines — the citation is stale or the file was trimmed"
        ));
        return;
    }

    let bracketed = &lines[(*start as usize - 1)..(*end as usize)];
    if bracketed.iter().all(|line| line.trim().is_empty()) {
        errors.push(format!(
            "{core_file}: \"{entry_label}\" cites {source_file}:{start}-{end}, but every line in \
             that range is blank — the citation no longer brackets any passage"
        ));
    }
}

/// Every `rules/core/*.json` file, discovered rather than hardcoded — a new
/// catalogue file is picked up automatically, matching "catalogue size is
/// data, never code".
fn core_json_files() -> Vec<PathBuf> {
    let dir = rules_dir().join("core");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("rules/core is readable: {e}"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "expected at least one rules/core/*.json file to exist"
    );
    files
}

#[test]
fn every_core_source_citation_brackets_real_content_in_its_named_file() {
    let source_dir = rules_dir().join("source/en");
    let mut all_refs = Vec::new();

    for core_path in core_json_files() {
        let core_file = core_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap()
            .to_string();
        let text = fs::read_to_string(&core_path)
            .unwrap_or_else(|e| panic!("{core_file} is readable: {e}"));
        let value: Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{core_file} is valid JSON: {e}"));
        collect_source_refs(&core_file, &value, &mut all_refs);
    }

    // This is the property that makes the test meaningful rather than
    // vacuous: if nothing carries a `source` block, every check below passes
    // trivially without having verified anything.
    assert!(
        all_refs.len() > 100,
        "expected many source citations across rules/core/*.json, found {}",
        all_refs.len()
    );

    let mut errors = Vec::new();
    for found in &all_refs {
        validate_source_ref(&source_dir, found, &mut errors);
    }

    assert!(
        errors.is_empty(),
        "{} stale/invalid source citation(s) in rules/core/*.json:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// Every `source.file` named anywhere in `rules/core/*.json` must be one of
/// the English sourcebooks `CLAUDE.md` documents as shipped — catching a
/// citation that names a book with no English source at all (which
/// `CLAUDE.md` → "Rules provenance" forbids: "A rule from a book with no
/// English source in `rules/source/en/` yet ... cannot be implemented").
#[test]
fn every_cited_source_file_exists_under_rules_source_en() {
    let source_dir = rules_dir().join("source/en");
    let mut cited_files: BTreeSet<String> = BTreeSet::new();

    for core_path in core_json_files() {
        let core_file = core_path.file_name().and_then(|n| n.to_str()).unwrap();
        let text = fs::read_to_string(&core_path).unwrap();
        let value: Value = serde_json::from_str(&text).unwrap();
        let mut refs = Vec::new();
        collect_source_refs(core_file, &value, &mut refs);
        cited_files.extend(refs.into_iter().map(|r| r.source_file));
    }

    assert!(
        !cited_files.is_empty(),
        "expected at least one cited source file across rules/core/*.json"
    );

    let missing: Vec<&String> = cited_files
        .iter()
        .filter(|file| !source_dir.join(file).is_file())
        .collect();
    assert!(
        missing.is_empty(),
        "rules/core/*.json cites source file(s) not present under rules/source/en/: {missing:?}"
    );
}
