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
//!
//! A second, sharper guard lives beside that one:
//! [`every_guarded_effect_cites_a_passage_that_names_its_own_mechanic`]. The
//! bracketing check above proves a citation points *somewhere*; it cannot
//! prove it points at a passage that supports the mechanic encoded beside it,
//! which is what `CLAUDE.md` → "Rules provenance" makes the whole purpose of
//! the `source` field. That is a **cross-site** property — it is not a
//! statement about any one catalogue entry but about the relation between
//! every entry encoding the same effect kind — and it is the shape of test
//! this repo already uses for enum↔Fluent-key parity and profile
//! conformance, aimed for the first time at the rules-semantics axis.

use std::collections::{BTreeMap, BTreeSet};
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
    /// The `type` of every entry in the sibling `effects` array on the same
    /// object, in document order — the mechanic(s) this citation is being
    /// offered as the source for. Empty for the many entries that carry a
    /// `source` block and no effects at all (narrative Virtues/Flaws, spells,
    /// abilities, …).
    effect_kinds: Vec<String>,
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
                let effect_kinds = map
                    .get("effects")
                    .and_then(Value::as_array)
                    .map(|effects| {
                        effects
                            .iter()
                            .filter_map(|effect| effect.get("type").and_then(Value::as_str))
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();
                out.push(FoundSourceRef {
                    core_file: core_file.to_string(),
                    entry_label,
                    source_file: file.to_string(),
                    start,
                    end,
                    effect_kinds,
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
        effect_kinds: _,
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

/// Every `source` block in every `rules/core/*.json` file, walked once.
fn all_source_refs() -> Vec<FoundSourceRef> {
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
    all_refs
}

#[test]
fn every_core_source_citation_brackets_real_content_in_its_named_file() {
    let source_dir = rules_dir().join("source/en");
    let all_refs = all_source_refs();

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
    let cited_files: BTreeSet<String> = all_source_refs()
        .into_iter()
        .map(|r| r.source_file)
        .collect();

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

// --- The mechanic-vs-passage guard -----------------------------------------

/// The effect families this guard covers, each with the phrase(s) the
/// rulebook uses for that mechanic. An item encoding one of these effects
/// must cite a passage containing at least one of its phrases (matched
/// case-insensitively).
///
/// Deliberately **two** families rather than all of `types.rs::Effect`. These
/// are the two where the rulebook's own vocabulary is consistent enough for
/// the check to be crisp, and a guard that is loud on two families beats one
/// that is noisy on thirty-five and grows an exemption list nobody reads. Add
/// a family here when its vocabulary is shown to be as consistent — not
/// speculatively.
const GUARDED_EFFECT_PHRASES: &[(&str, &[&str])] = &[
    ("lab_total_mod", &["lab total"]),
    ("casting_total_mod", &["casting total", "casting score"]),
];

/// Encodings whose cited passage states the mechanic in different words, with
/// the reason the paraphrase is sound. Every row is a sentence somebody had
/// to mean: writing one down is the point of the exemption, not its cost.
const PARAPHRASE_EXEMPTIONS: &[(&str, &str, &str)] = &[(
    "flaw.poor_formulaic_magic",
    "casting_total_mod",
    "ArMDE:6612 says \"Subtract 5 from every roll that you make to cast Formulaic \
     spells. This does not apply to Ritual spells.\"; ArMDE:9103 defines FORMULAIC \
     CASTING TOTAL = Casting Score + Die Roll, so the roll the passage penalizes IS \
     the Formulaic Casting Total, and the excluded Rituals are exactly what the \
     effect's \"formulaic\" scope excludes. A wording difference, not a mechanic \
     difference.",
)];

/// The minimum number of items each guarded family must actually match. A
/// floor, never a total — `CLAUDE.md` → "Catalogue size is data, never code"
/// forbids asserting exact catalogue counts, but without *some* floor this
/// test passes vacuously the day a family is renamed and matches nothing.
const MIN_ITEMS_PER_GUARDED_FAMILY: usize = 5;

/// Reads the lines a citation brackets, lowercased and joined, caching each
/// source file so the guard reads a multi-megabyte rulebook once rather than
/// once per citation.
fn bracketed_passage(
    source_dir: &Path,
    cache: &mut BTreeMap<String, Vec<String>>,
    found: &FoundSourceRef,
) -> Option<String> {
    let lines = cache.entry(found.source_file.clone()).or_insert_with(|| {
        fs::read_to_string(source_dir.join(&found.source_file))
            .map(|text| text.lines().map(str::to_lowercase).collect())
            .unwrap_or_default()
    });
    if found.start < 1 || found.end < found.start || found.end as usize > lines.len() {
        // Malformed or out-of-bounds ranges are the other test's finding; do
        // not report the same citation twice.
        return None;
    }
    Some(lines[(found.start as usize - 1)..(found.end as usize)].join("\n"))
}

/// The cross-site invariant: **every effect encoded against a guarded family
/// must cite a passage that actually talks about that family's mechanic.**
///
/// `every_core_source_citation_brackets_real_content_in_its_named_file` proves
/// a citation points somewhere real. It cannot prove it points at a passage
/// that *supports the mechanic encoded beside it* — and that is the property
/// `CLAUDE.md` → "Rules provenance" actually demands ("Every rule … encoded as
/// data MUST be taken from the authoritative Markdown … Implementing a rule
/// from training-data recollection is prohibited"). JSON carries no comments,
/// so there is no second place the claim is written down and could be
/// cross-read.
///
/// The failure mode it exists for: a slice encodes a Virtue or Flaw against
/// the nearest available `types.rs::Effect` variant because no exact one
/// exists. Every gate stays green — the JSON parses, referential integrity
/// holds, the citation is in bounds and non-blank — and the wrong number then
/// reaches every derived read-out that consumes that family, on every
/// character carrying the item, in both locales. This test is the only thing
/// that can see it without a human reading the rulebook beside the JSON.
#[test]
fn every_guarded_effect_cites_a_passage_that_names_its_own_mechanic() {
    let source_dir = rules_dir().join("source/en");
    let all_refs = all_source_refs();
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut matched: BTreeMap<&str, usize> = GUARDED_EFFECT_PHRASES
        .iter()
        .map(|(kind, _)| (*kind, 0))
        .collect();
    let mut errors = Vec::new();

    for found in &all_refs {
        for kind in &found.effect_kinds {
            let Some((_, phrases)) = GUARDED_EFFECT_PHRASES
                .iter()
                .find(|(guarded, _)| guarded == kind)
            else {
                continue;
            };
            *matched.get_mut(kind.as_str()).expect("family is seeded") += 1;

            if PARAPHRASE_EXEMPTIONS
                .iter()
                .any(|(id, exempt_kind, _)| *id == found.entry_label && exempt_kind == kind)
            {
                continue;
            }
            let Some(passage) = bracketed_passage(&source_dir, &mut cache, found) else {
                continue;
            };
            if !phrases.iter().any(|phrase| passage.contains(phrase)) {
                errors.push(format!(
                    "{}: \"{}\" encodes a {kind} effect, but the passage it cites \
                     ({}:{}-{}) never says {}. Either the effect is encoded against the \
                     wrong mechanic, or the citation points at the wrong passage, or the \
                     passage paraphrases the mechanic — in which case add a row to \
                     PARAPHRASE_EXEMPTIONS stating why.",
                    found.core_file,
                    found.entry_label,
                    found.source_file,
                    found.start,
                    found.end,
                    phrases
                        .iter()
                        .map(|p| format!("\"{p}\""))
                        .collect::<Vec<_>>()
                        .join(" or "),
                ));
            }
        }
    }

    for (kind, count) in &matched {
        assert!(
            *count >= MIN_ITEMS_PER_GUARDED_FAMILY,
            "the {kind} guard matched only {count} item(s) in rules/core/*.json — below the \
             floor of {MIN_ITEMS_PER_GUARDED_FAMILY}, so this test would pass without having \
             checked anything. Was the effect kind renamed?"
        );
    }

    assert!(
        errors.is_empty(),
        "{} effect(s) in rules/core/*.json are encoded against a mechanic their own cited \
         passage does not state:\n{}",
        errors.len(),
        errors.join("\n")
    );
}
