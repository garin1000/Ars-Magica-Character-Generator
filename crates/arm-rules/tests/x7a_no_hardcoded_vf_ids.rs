//! X7a-refactor guard: the engine must never hardcode a Virtue/Flaw catalogue
//! id to drive rules BEHAVIOUR — CLAUDE.md's "Catalogue size is data, never
//! code" invariant. Before this slice, `derived.rs` carried
//! `D4_EXCLUDED_FROM_LAB_GRID`/`D4_WITHIN_FOCUS_ONLY`/`CYCLIC_MAGIC_NEGATIVE`
//! id lists and `life_stage.rs::later_life_rate` matched
//! `"virtue.guild_apprentice"` directly; both are now DATA
//! (`Effect::LabTotalMod::scope`/`suppressed_when`,
//! `Effect::SuppressesLaterLifeXpRate`) on the ruleset entries themselves. This
//! guard is the regression lock: a future contributor reaching for the same
//! shortcut fails loudly here instead of shipping silently.
//!
//! **Scope: `crates/arm-rules/src` only** — the pure engine, never `tests/`
//! (fixtures routinely construct a synthetic ruleset by literal id, which is
//! not a rules-behaviour hardcode) and never `crates/arm-app` (owns no rules
//! logic). A hit is excluded when it is (a) inside a `#[cfg(test)]` module —
//! every scanned file's own unit tests, which sit in one trailing module by
//! convention (`grep`-verified file-by-file while triaging this slice) — or
//! (b) on a comment line (`//`, `///`, `//!`), since a doc comment illustrating
//! the JSON shape (`types.rs`'s ``"kind": "has", "value": "virtue.x"`` example)
//! names no real behaviour.

use std::fs;
use std::path::{Path, PathBuf};

// Deliberately self-contained rather than reusing `tests/citation_support`:
// that module's `pub fn`s are all consumed collectively by
// `rulebook_citations.rs`/`source_citations.rs` (each imports the whole
// surface), which is what lets its own doc comment claim "no dead_code
// allowance is needed" — a third binary using only a subset of it would
// reintroduce exactly the per-binary dead-code warnings that doc comment
// says do not occur. `repo_root`/`relative`/a directory walker are ~15 lines;
// duplicating them here is cheaper than adding an `#[allow(dead_code)]` to
// shared guard infrastructure two other tests already rely on being clean.

/// `crates/arm-rules` (this crate's manifest dir) -> the repository root.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Renders `path` relative to [`repo_root`] for a failure message.
fn relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Explicit, reasoned exceptions: a file where a `"virtue.`/`"flaw.` string
/// literal in production code is NOT a rules-behaviour hardcode, so this guard
/// must not flag it. Every entry needs a reason; an entry with no real hit
/// left in the file is dead weight the next contributor should notice and
/// remove (asserted below).
const ALLOWED: &[(&str, &str)] = &[(
    "migration.rs",
    "REFOLDED_BEING_ITEMS: a frozen, one-time SAVE-FORMAT migration list, not \
     rules behaviour. It names the three items whose `being` parameter changed \
     from free-text to an enumerated domain, so `load_entity_migrating` can \
     recognise and refold a pre-change save. Deliberately NOT read from live \
     ruleset data: `load_entity_migrating` holds no `Ruleset`, and even if it \
     did, a live lookup would let editing a label later silently change what a \
     decade-old save migrates to — the table is a historical snapshot, exactly \
     like `LEGACY_BEING_LABELS` beside it.",
)];

/// Whether `file_name` (e.g. `"migration.rs"`) is on the [`ALLOWED`] list.
fn is_allowed(file_name: &str) -> bool {
    ALLOWED.iter().any(|(name, _)| *name == file_name)
}

/// Whether `trimmed` is the exact line that opens this file's own trailing
/// unit-test module — the boundary past which nothing is scanned. Exact-match
/// only (not a `contains`), so a comment merely *mentioning* the attribute
/// (`crates/arm-rules/src/validation/scores.rs`'s own doc comment about a past
/// `#[cfg(test)] mod` mishap) is not mistaken for the real one.
fn opens_test_module(trimmed: &str) -> bool {
    trimmed == "#[cfg(test)]"
}

/// Whether `trimmed` is a comment line (`//`, `///`, or `//!` all share this
/// prefix).
fn is_comment(trimmed: &str) -> bool {
    trimmed.starts_with("//")
}

/// One `"virtue.`/`"flaw.` string-literal hit outside a test module and
/// outside a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hit {
    path: PathBuf,
    line: usize,
    text: String,
}

/// Scans `content` (one source file's text) for hardcoded id hits, per this
/// guard's own doc comment. Pure and file-agnostic so the detector itself can
/// be unit-tested against an inline fixture below.
fn scan_for_hardcoded_ids(path: &Path, content: &str) -> Vec<Hit> {
    let mut hits = Vec::new();
    for (idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim_start();
        if opens_test_module(trimmed) {
            break;
        }
        if is_comment(trimmed) {
            continue;
        }
        if raw_line.contains("\"virtue.") || raw_line.contains("\"flaw.") {
            hits.push(Hit {
                path: path.to_path_buf(),
                line: idx + 1,
                text: raw_line.trim().to_string(),
            });
        }
    }
    hits
}

#[test]
fn detector_finds_a_production_hit_ignores_a_comment_and_stops_at_the_test_module() {
    let fixture = "\
const D4_EXCLUDED: &[&str] = &[\"virtue.adept_laboratory_student\"];\n\
// a comment mentioning \"flaw.weak_scholar\" is not a hit\n\
/// nor is a doc comment: `{ \"kind\": \"has\", \"value\": \"virtue.x\" }`\n\
#[cfg(test)]\n\
mod tests {\n\
    const T: &str = \"virtue.only_in_tests\";\n\
}\n\
";
    let hits = scan_for_hardcoded_ids(Path::new("fixture.rs"), fixture);
    assert_eq!(
        hits.len(),
        1,
        "expected exactly one production hit, got {hits:?}"
    );
    assert_eq!(hits[0].line, 1);
    assert!(hits[0].text.contains("virtue.adept_laboratory_student"));
}

/// Every hit `crates/arm-rules/src` carries today, [`ALLOWED`] entries
/// included — the single input both tests below share.
fn all_hits() -> Vec<Hit> {
    let root = repo_root().join("crates/arm-rules/src");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    files.sort();
    assert!(
        files.len() > 20,
        "expected many files under crates/arm-rules/src, found {} — the walker \
         or the root is broken",
        files.len()
    );
    let mut hits = Vec::new();
    for path in files {
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        hits.extend(scan_for_hardcoded_ids(&path, &content));
    }
    hits
}

#[test]
fn every_allow_listed_file_still_carries_a_real_hit() {
    // Keeps `ALLOWED` honest: an entry whose file no longer hits at all is a
    // stale exception nobody re-checked, not a legitimate carve-out.
    let hits = all_hits();
    for (name, _) in ALLOWED {
        let carries = hits.iter().any(|hit| {
            hit.path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|found| found == *name)
        });
        assert!(
            carries,
            "{name} is on the ALLOWED list but no longer carries a \
             \"virtue.\"/\"flaw.\" hit — remove the stale entry"
        );
    }
}

#[test]
fn engine_src_never_hardcodes_a_virtue_or_flaw_id_for_rules_behaviour() {
    let hits = all_hits();
    let offenders: Vec<String> = hits
        .into_iter()
        .filter(|hit| {
            !hit.path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_allowed)
        })
        .map(|hit| format!("{}:{}: {}", relative(&hit.path), hit.line, hit.text))
        .collect();
    assert!(
        offenders.is_empty(),
        "{} line(s) in crates/arm-rules/src hardcode a Virtue/Flaw catalogue id \
         to drive rules behaviour (CLAUDE.md: \"Catalogue size is data, never \
         code\") — move the rule onto the ruleset entry itself as a new/extended \
         Effect field instead, following the `gate: Option<ParamGate>` idiom \
         (see X7a's `Effect::LabTotalMod::scope`/`suppressed_when` and \
         `Effect::SuppressesLaterLifeXpRate` for the pattern), or add a reasoned \
         entry to this test's ALLOWED list if the hit is genuinely not a rules \
         hardcode (e.g. a one-time save-format migration table):\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}
