//! D1a: `CLAUDE.md` → "Rules provenance" requires every rule citation to name
//! its sourcebook and an inclusive line range, and forbids a bare line number
//! ("meaningless without the book"). In practice the repo had drifted into
//! two spellings — a bare `` `:NNNN` `` that silently trusts the reader to
//! remember which book the enclosing file is about, and the full basename
//! (`Ars Magica - Definitive Edition (Core Rules).md:2774`) — and nothing
//! checked either one. `docs/audit-2026-08.md` records what that cost: a
//! prior sweep normalizing ~470 shorthand citations to full basenames caught
//! **four wrong line ranges** along the way, each "hiding behind a shorthand
//! citation and caught only by opening the source file".
//!
//! Norbert's decision (recorded in the D1a task brief): keep the shorthand,
//! but make it the *established acronym* from
//! `rules/source/de/translation-tables/grundbegriffe.md:212-229` rather than
//! either the bare form or the full basename, and make that acronym
//! mechanically resolvable so a wrong range can never hide behind it again.
//! This file is that guard. It is deliberately scoped to `crates/arm-rules/src`
//! and `crates/arm-app/src` for D1a; later slices (D1b, D1c) widen
//! [`citation_roots`] to `crates/*/tests`, `crates/arm-rules/RULES.md`,
//! `ui/src`, and `docs/`.
//!
//! This is a different subject from `rules_md_citations.rs`, which guards
//! *implementation-site* citations (`RULES.md` pointing at Rust code, and
//! Rust comments pointing back at `RULES.md`) — not rulebook citations.
//!
//! **Deliberate exclusion: string literals.** [`comment_blocks`] only collects
//! `///`/`//!`/`//` lines, so a full basename inside a Rust **string
//! literal** is invisible to every detector in this file — e.g.
//! `crates/arm-rules/src/ruleset/integrity.rs` spells `Ars Magica -
//! Definitive Edition (Core Rules).md:NNNN` seven times inside
//! `errors.push(format!(...))` diagnostics built by `RulesetIntegrity`'s
//! validators (`IntegrityError`, surfaced through `RulesetError` when a
//! ruleset — including a hand-edited `rules/core/*.json`, which
//! `CLAUDE.md`'s trust model treats as the file the user opens) fails a
//! structural check (non-tiling aging rows, misordered Aging/Decrepitude
//! thresholds, …). This is a **kept-on-purpose** exception, not an oversight:
//! these are `IntegrityError` diagnostics for whoever is editing the rules
//! JSON, in the same spirit as `CLAUDE.md`'s "fail loudly with clear error
//! listing offending IDs" — the reader needs to open a specific rulebook, and
//! spelling it out in full removes any need to know the nine-item acronym
//! table to act on the message. It is not the routed-through-Fluent UI copy
//! `CLAUDE.md` bans from being hardcoded (no Ability/Virtue/label text, no
//! normal-session string); it is data-integrity diagnostic text reached only
//! when the rules data itself is broken, i.e. self-inflicted by whoever
//! edited it. `dotmd_citation_detector_ignores_a_dot_md_path_inside_a_string_literal_but_flags_one_in_a_comment`
//! is the test proving this exclusion is intentional and mechanical (it falls
//! out of `comment_blocks`'s existing "comments only" scope), not accidental.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// `crates/arm-rules` (this crate's manifest dir) -> repo root.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rules_source_en() -> PathBuf {
    repo_root().join("rules/source/en")
}

/// The nine acronyms `CLAUDE.md` → "Rules provenance" defines, each mapped to
/// its basename under `rules/source/en/`. Canonical origin:
/// `rules/source/de/translation-tables/grundbegriffe.md:212-229`, whose
/// `:212` row offers both `ArM5` and `ArMDE` for the core book — this repo
/// uses `ArMDE` only, so `ArM5` is deliberately absent here (see
/// [`REJECTED_SPELLINGS`]).
const BOOK_ACRONYMS: &[(&str, &str)] = &[
    ("ArMDE", "Ars Magica - Definitive Edition (Core Rules).md"),
    (
        "HoH:TL",
        "Ars Magica 5e - Houses of Hermes - True Lineages.md",
    ),
    (
        "HoH:MC",
        "Ars Magica 5e - Houses of Hermes - Mystery Cults.md",
    ),
    ("HoH:S", "Ars Magica 5e - Houses of Hermes - Societates.md"),
    ("HM:RE", "Ars Magica 5e - Magic - Hedge Magic (Revised).md"),
    ("RoP:M", "Ars Magica 5e - Realms of Power - Magic.md"),
    ("RoP:F", "Ars Magica 5e - Realms of Power - Faerie.md"),
    (
        "RoP:D",
        "Ars Magica 5e - Realms of Power - The Divine (Revised).md",
    ),
    ("RoP:I", "Ars Magica 5e - Realms of Power - The Infernal.md"),
];

/// Near-miss spellings that must never appear in a citation, because they
/// would otherwise silently coexist with the canonical acronym above and
/// defeat "there is exactly one spelling". `ArM5` is the one
/// `grundbegriffe.md:212` itself documents as an alternate.
const REJECTED_SPELLINGS: &[&str] = &["ArM5"];

/// The roots this guard scans in slice D1a.
fn citation_roots() -> Vec<PathBuf> {
    vec![
        repo_root().join("crates/arm-rules/src"),
        repo_root().join("crates/arm-app/src"),
    ]
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
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn all_rust_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in citation_roots() {
        rust_files(&root, &mut files);
    }
    files.sort();
    files
}

/// One contiguous run of `///`, `//!`, or `//` comment lines, with the
/// comment leader and a single leading space stripped, joined by a single
/// space. Citations live only in comments in this codebase, and joining a
/// run into one logical string is what lets a citation's continuation list
/// (`, :NNN`) be found even when hand-wrapped prose splits it — or splits a
/// full basename — across two physical comment lines (observed in the wild,
/// e.g. `mythic_companion.rs:76-77`: "...Core Rules).md" wraps onto the
/// following `///` line). Returns each block paired with the 1-based source
/// line it started on, for error messages; precision beyond "which block"
/// is not needed since a human re-finds the exact spot by searching.
fn comment_blocks(content: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim_start();
        let text = trimmed
            .strip_prefix("///")
            .or_else(|| trimmed.strip_prefix("//!"))
            .or_else(|| trimmed.strip_prefix("//"));
        match text {
            Some(text) => {
                let text = text.strip_prefix(' ').unwrap_or(text);
                match &mut current {
                    Some((_, joined)) => {
                        if !joined.is_empty() {
                            joined.push(' ');
                        }
                        joined.push_str(text);
                    }
                    None => current = Some((idx + 1, text.to_string())),
                }
            }
            None => {
                if let Some(block) = current.take() {
                    blocks.push(block);
                }
            }
        }
    }
    if let Some(block) = current.take() {
        blocks.push(block);
    }
    blocks
}

/// One rulebook citation found in a comment block: the resolved acronym/book
/// plus the 1-based inclusive line range it cites. Produced by
/// [`find_citations`] for both the anchoring `ACRONYM:NNNN` citation and any
/// `, :NNNN` continuations chained after it, since a continuation cites a
/// real line in the same book and must be checked too — the audit's four
/// wrong ranges were exactly this shape.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Citation {
    acronym: &'static str,
    book_file: &'static str,
    start: i64,
    end: i64,
}

/// Parses a leading run of ASCII digits from `s`, returning the number and
/// how many bytes it consumed.
fn parse_digits(s: &str) -> Option<(i64, usize)> {
    let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    let len = digits.len();
    digits.parse().ok().map(|n| (n, len))
}

/// Parses one `:NNNN` or `:NNNN-MMMM` citation number starting right after
/// the leading colon at byte `colon_at` in `text`. Returns the (start, end)
/// range and the byte offset immediately after the parsed number(s).
fn parse_citation_number(text: &str, colon_at: usize) -> Option<(i64, i64, usize)> {
    let after = &text[colon_at + 1..];
    let (start, consumed) = parse_digits(after)?;
    let mut end = start;
    let mut total = consumed;
    if let Some(rest) = after[consumed..].strip_prefix('-')
        && let Some((end_num, end_len)) = parse_digits(rest)
    {
        end = end_num;
        total = consumed + 1 + end_len;
    }
    Some((start, end, colon_at + 1 + total))
}

/// True when the byte before `idx` in `text` is not a letter/digit/colon —
/// i.e. an acronym match at `idx` is not a suffix of some longer word.
fn word_boundary_before(text: &str, idx: usize) -> bool {
    match text[..idx].chars().next_back() {
        None => true,
        Some(c) => !(c.is_ascii_alphanumeric() || c == ':'),
    }
}

/// Finds every anchored `ACRONYM:NNNN(-MMMM)?` citation in `text`, plus every
/// `, :NNNN(-MMMM)?` continuation chained directly after one (inheriting its
/// book), by repeatedly re-scanning past each parsed citation. This is the
/// full, "acronym resolves and range is checkable" collector used by
/// `every_rulebook_acronym_resolves_to_a_file_under_rules_source_en` and
/// `every_rulebook_citation_lands_inside_its_file`.
fn find_citations(text: &str) -> Vec<Citation> {
    let mut found = Vec::new();
    for &(acronym, book_file) in BOOK_ACRONYMS {
        let needle = format!("{acronym}:");
        let mut search_from = 0usize;
        while let Some(rel) = text[search_from..].find(needle.as_str()) {
            let acronym_at = search_from + rel;
            let colon_at = acronym_at + acronym.len();
            if !word_boundary_before(text, acronym_at) {
                search_from = colon_at + 1;
                continue;
            }
            let Some((start, end, mut pos)) = parse_citation_number(text, colon_at) else {
                search_from = colon_at + 1;
                continue;
            };
            found.push(Citation {
                acronym,
                book_file,
                start,
                end,
            });
            // Chase `, :NNNN` continuations chained after this citation —
            // they inherit `acronym`/`book_file` without repeating them.
            while let Some(rest) = text[pos..].strip_prefix(", :") {
                let colon_at = pos + 2;
                let Some((cstart, cend, next_pos)) = parse_citation_number(text, colon_at) else {
                    break;
                };
                let _ = rest;
                found.push(Citation {
                    acronym,
                    book_file,
                    start: cstart,
                    end: cend,
                });
                pos = next_pos;
            }
            search_from = pos;
        }
    }
    found
}

/// True when a citation in `text` uses one of [`REJECTED_SPELLINGS`] instead
/// of the canonical acronym.
fn find_rejected_spellings(text: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    for &rejected in REJECTED_SPELLINGS {
        let needle = format!("{rejected}:");
        if let Some(rel) = text.find(needle.as_str())
            && word_boundary_before(text, rel)
        {
            found.push(rejected);
        }
    }
    found
}

/// Finds every full basename spelled out in `text` — the spelling this slice
/// retires in favor of the acronym.
fn find_full_basenames(text: &str) -> Vec<&'static str> {
    BOOK_ACRONYMS
        .iter()
        .map(|&(_, file)| file)
        .filter(|file| text.contains(file))
        .collect()
}

/// Finds every `.md:NNNN` (or `.md:NNNN-MMMM`) shape in `text`, regardless of
/// what precedes the `.md` — a shape-only backstop for [`find_full_basenames`],
/// which only recognises the nine basenames verbatim and so cannot see a
/// *mangled* or *partial* one (e.g. `The Divine (Revised).md:1975`, missing
/// the `Ars Magica 5e - Realms of Power - ` prefix of the real file). After
/// the D1a sweep no comment should ever spell a rulebook as a `.md` path in
/// any form — every real citation uses `ACRONYM:NNNN` — so this test treats
/// the bare shape itself as the defect, without needing to know which book
/// was intended. Returns a short snippet of surrounding context (up to 70
/// bytes before, 15 after) for the failure message, snapped to the nearest
/// UTF-8 char boundary.
fn find_dotmd_citations(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = text[search_from..].find(".md:") {
        let dot_at = search_from + rel;
        let colon_at = dot_at + 3;
        let after_colon = colon_at + 1;
        if text[after_colon..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
        {
            let mut start = dot_at.saturating_sub(70);
            while !text.is_char_boundary(start) {
                start += 1;
            }
            let mut end = (after_colon + 15).min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            found.push(text[start..end].to_string());
        }
        search_from = after_colon;
    }
    found
}

/// Finds every bare rulebook citation in `text`: a backtick or an opening
/// parenthesis immediately followed by `:NNNN(-MMMM)?`, with no acronym in
/// front of the colon. Returns the matched snippet for error messages.
///
/// **Discrimination rule**, chosen to avoid false positives on the near-miss
/// shapes this codebase actually contains (proved by
/// `bare_citation_detector_flags_real_bare_citations_and_ignores_near_misses`):
/// a *continuation* (`, :NNNN`) is comma-space before the colon, never a
/// backtick or paren, so it is never matched here — no special-case exception
/// is needed for it. A `RULES.md:NNNN` reference (a different guard's
/// subject) has a letter before the colon, not a backtick/paren, so it is
/// never matched either. Only the two shapes this codebase's own bare-citation
/// convention actually used — `` `:NNNN` `` and `(:NNNN...)` — are flagged.
fn find_bare_citations(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    for (idx, ch) in text.char_indices() {
        if ch != ':' || idx == 0 {
            continue;
        }
        let opener = bytes[idx - 1];
        if opener != b'`' && opener != b'(' {
            continue;
        }
        let Some((_, _, end)) = parse_citation_number(text, idx) else {
            continue;
        };
        // Include the closing delimiter in the reported snippet when present,
        // so a backtick-bare citation reads `` `:16565` `` rather than the
        // unbalanced `` `:16565 ``.
        let closer = match opener {
            b'`' => Some('`'),
            b'(' => Some(')'),
            _ => None,
        };
        let end_with_closer = match closer {
            Some(c) if text[end..].starts_with(c) => end + c.len_utf8(),
            _ => end,
        };
        found.push(text[idx - 1..end_with_closer].to_string());
    }
    found
}

fn book_line_count(book_file: &str) -> usize {
    let content = fs::read_to_string(rules_source_en().join(book_file))
        .unwrap_or_else(|e| panic!("{book_file} is readable under rules/source/en: {e}"));
    content.lines().count()
}

fn book_is_blank_in_range(book_file: &str, start: i64, end: i64) -> bool {
    let content = fs::read_to_string(rules_source_en().join(book_file))
        .unwrap_or_else(|e| panic!("{book_file} is readable under rules/source/en: {e}"));
    let lines: Vec<&str> = content.lines().collect();
    let bracketed = &lines[(start as usize - 1)..(end as usize)];
    bracketed.iter().all(|line| line.trim().is_empty())
}

/// Every citation found across [`citation_roots`], paired with the file and
/// block-start line it came from (for error messages).
fn collect_all_citations() -> (
    usize, /* files scanned */
    Vec<(PathBuf, usize, Citation)>,
) {
    let files = all_rust_files();
    let mut all = Vec::new();
    for path in &files {
        let content = fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        for (line, text) in comment_blocks(&content) {
            for citation in find_citations(&text) {
                all.push((path.clone(), line, citation));
            }
        }
    }
    (files.len(), all)
}

fn relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

#[test]
fn every_rulebook_acronym_resolves_to_a_file_under_rules_source_en() {
    // Every canonical acronym's book must actually exist — a table typo would
    // otherwise make every citation against it silently unresolvable.
    for &(acronym, book_file) in BOOK_ACRONYMS {
        assert!(
            rules_source_en().join(book_file).is_file(),
            "acronym {acronym} maps to {book_file}, which does not exist under rules/source/en/"
        );
    }

    let (files_scanned, citations) = collect_all_citations();
    assert!(
        files_scanned > 10,
        "expected to scan many source files, saw {files_scanned}"
    );
    // The floor that keeps this test from passing vacuously: before the D1a
    // sweep, zero comments use the acronym form at all, so a walker that
    // scans nothing (or a detector that matches nothing) would otherwise
    // pass by default.
    assert!(
        citations.len() > 100,
        "expected many acronym'd rulebook citations across {files_scanned} files, found {}",
        citations.len()
    );

    let mut offenders: BTreeSet<String> = BTreeSet::new();
    for path in all_rust_files() {
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in comment_blocks(&content) {
            for rejected in find_rejected_spellings(&text) {
                offenders.insert(format!(
                    "{}:{line}: cites the rulebook as `{rejected}`, which is not the canonical \
                     spelling — use `ArMDE`",
                    relative(&path)
                ));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "{} citation(s) use a rejected book spelling:\n{}",
        offenders.len(),
        offenders.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn every_rulebook_citation_lands_inside_its_file() {
    let (files_scanned, citations) = collect_all_citations();
    assert!(
        files_scanned > 10,
        "expected to scan many source files, saw {files_scanned}"
    );
    assert!(
        citations.len() > 100,
        "expected many acronym'd rulebook citations across {files_scanned} files, found {}",
        citations.len()
    );

    let mut errors = Vec::new();
    for (path, line, citation) in &citations {
        let Citation {
            acronym,
            book_file,
            start,
            end,
        } = citation;
        if *start < 1 || end < start {
            errors.push(format!(
                "{}:{line}: cites {acronym}:{start}-{end}, which is not a well-formed 1-based \
                 inclusive range",
                relative(path)
            ));
            continue;
        }
        let total = book_line_count(book_file) as i64;
        if *end > total {
            errors.push(format!(
                "{}:{line}: cites {acronym}:{start}-{end}, but {book_file} has only {total} \
                 lines",
                relative(path)
            ));
            continue;
        }
        if book_is_blank_in_range(book_file, *start, *end) {
            errors.push(format!(
                "{}:{line}: cites {acronym}:{start}-{end}, but every line in that range is blank",
                relative(path)
            ));
        }
    }
    assert!(
        errors.is_empty(),
        "{} rulebook citation(s) do not land inside their book:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

#[test]
fn no_source_comment_cites_a_rulebook_by_bare_line_number() {
    let mut offenders: BTreeSet<String> = BTreeSet::new();
    let mut scanned = 0usize;
    for path in all_rust_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in comment_blocks(&content) {
            for snippet in find_bare_citations(&text) {
                offenders.insert(format!(
                    "{}:{line}: bare rulebook citation {snippet} has no acronym — name the book \
                     (e.g. ArMDE:{snippet})",
                    relative(&path),
                    snippet = snippet
                ));
            }
        }
    }
    assert!(
        scanned > 10,
        "expected to scan many source files, saw {scanned}"
    );
    assert!(
        offenders.is_empty(),
        "{} bare rulebook citation(s); every citation must name its book by acronym:\n{}",
        offenders.len(),
        offenders.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn no_source_comment_spells_a_rulebook_by_full_basename() {
    let mut offenders: BTreeSet<String> = BTreeSet::new();
    let mut scanned = 0usize;
    for path in all_rust_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in comment_blocks(&content) {
            for basename in find_full_basenames(&text) {
                offenders.insert(format!(
                    "{}:{line}: spells the rulebook out by full basename ({basename}) instead of \
                     its acronym",
                    relative(&path)
                ));
            }
        }
    }
    assert!(
        scanned > 10,
        "expected to scan many source files, saw {scanned}"
    );
    assert!(
        offenders.is_empty(),
        "{} citation(s) use the full basename instead of the acronym:\n{}",
        offenders.len(),
        offenders.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

// ---------------------------------------------------------------------------
// Fixture-based unit tests for the detectors above. These do not touch the
// live tree — they prove the discrimination rules against hand-written near-
// misses, per the D1a task brief's requirement that the bare-citation regex
// "not false-positive on things that are not citations".
// ---------------------------------------------------------------------------

#[test]
fn bare_citation_detector_flags_real_bare_citations_and_ignores_near_misses() {
    let fixture = "\
Backtick bare: `:16565` should be flagged.\n\
Paren bare: (:16619-16638) should be flagged.\n\
A continuation is not bare: ArMDE:9245, :4822-4826, :3645-3648.\n\
A timestamp is not bare: the clock read 10:30 that morning.\n\
A CSS-shaped value is not bare: width: 10px is unrelated.\n\
An implementation-site self-citation is not this guard's subject: see spec.md:22 for detail.\n\
A version string is not bare: protocol 2:15 is unrelated.\n\
A byte range is not bare: bytes 0:16 of the packet.\n\
A normal acronym citation is not bare: Source: ArMDE:2774.\n\
";
    let found = find_bare_citations(fixture);
    assert_eq!(
        found,
        vec!["`:16565`".to_string(), "(:16619-16638)".to_string()],
        "expected exactly the two real bare citations, found {found:?}"
    );
}

#[test]
fn citation_finder_resolves_continuations_across_a_wrapped_block_to_the_inherited_acronym() {
    // Mirrors the real wrap in `derived.rs:319-322`: a comma-continuation
    // list where only the first number carries the acronym.
    let text = "Source: Ars Magica - Definitive Edition (Core Rules) becomes ArMDE:9245, \
                :4822-4826, :3645-3648 after the sweep.";
    let citations = find_citations(text);
    assert_eq!(
        citations,
        vec![
            Citation {
                acronym: "ArMDE",
                book_file: "Ars Magica - Definitive Edition (Core Rules).md",
                start: 9245,
                end: 9245,
            },
            Citation {
                acronym: "ArMDE",
                book_file: "Ars Magica - Definitive Edition (Core Rules).md",
                start: 4822,
                end: 4826,
            },
            Citation {
                acronym: "ArMDE",
                book_file: "Ars Magica - Definitive Edition (Core Rules).md",
                start: 3645,
                end: 3648,
            },
        ]
    );
}

#[test]
fn citation_finder_does_not_confuse_a_rejected_spelling_with_the_canonical_acronym() {
    assert_eq!(find_citations("ArM5:2774"), Vec::new());
    assert_eq!(find_rejected_spellings("ArM5:2774"), vec!["ArM5"]);
    assert_eq!(find_citations("ArMDE:2774").len(), 1);
    assert_eq!(find_rejected_spellings("ArMDE:2774"), Vec::<&str>::new());
}

#[test]
fn full_basename_detector_finds_a_basename_split_across_a_wrapped_comment() {
    let source = "\
/// Extra virtue points at no flaw cost. Source: Ars Magica - Definitive\n\
/// Edition (Core Rules).md:2664.\n";
    let blocks = comment_blocks(source);
    assert_eq!(blocks.len(), 1);
    let (_, joined) = &blocks[0];
    assert_eq!(
        find_full_basenames(joined),
        vec!["Ars Magica - Definitive Edition (Core Rules).md"]
    );
}

#[test]
fn dotmd_citation_detector_flags_any_dot_md_colon_digits_shape_including_mangled_basenames() {
    // The real near-misses this backstop exists for: a partial basename
    // (`types.rs:1109`) and a differently-mangled one missing the
    // `Ars Magica - ` / parenthetical parts (`xp.rs:1313`), plus a correctly
    // spelled full basename for good measure (already caught by
    // `find_full_basenames`, but this detector must see it too since it is a
    // superset). None of `RULES.md` (no trailing digits — a different guard's
    // subject), a bare mention of `CLAUDE.md` with no citation, or a
    // timestamp-shaped `10:30` should be flagged.
    let fixture = "\
Mangled partial basename: The Divine (Revised).md:1975 should be flagged.\n\
Differently mangled: Definitive Edition Core Rules.md:4814-4816 should be flagged.\n\
Correctly spelled full basename: Ars Magica - Definitive Edition (Core Rules).md:2774 \
should also be flagged.\n\
Not a citation at all: see CLAUDE.md for details.\n\
Not this guard's subject: RULES.md is a different file, cited without a line number here.\n\
Not a citation: the clock read 10:30 that morning.\n\
";
    let found = find_dotmd_citations(fixture);
    assert_eq!(
        found.len(),
        3,
        "expected exactly the three `.md:NNNN` shapes, found {found:?}"
    );
}

#[test]
fn no_source_comment_cites_a_rulebook_by_any_dot_md_path() {
    // Shape-only backstop for `no_source_comment_spells_a_rulebook_by_full_basename`:
    // that test only recognises the nine basenames verbatim, so a mangled or
    // partial one slips through it. This test flags the bare shape
    // `.md:NNNN` wherever it appears in a comment, regardless of what
    // precedes the `.md` — after the D1a sweep no comment should spell a
    // rulebook as a `.md` path in any spelling at all.
    let mut offenders: BTreeSet<String> = BTreeSet::new();
    let mut scanned = 0usize;
    for path in all_rust_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in comment_blocks(&content) {
            for snippet in find_dotmd_citations(&text) {
                offenders.insert(format!(
                    "{}:{line}: cites a rulebook as a `.md` path ({snippet}) instead of its \
                     acronym",
                    relative(&path)
                ));
            }
        }
    }
    assert!(
        scanned > 10,
        "expected to scan many source files, saw {scanned}"
    );
    assert!(
        offenders.is_empty(),
        "{} citation(s) spell a rulebook as a `.md` path instead of the acronym:\n{}",
        offenders.len(),
        offenders.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn dotmd_citation_detector_ignores_a_dot_md_path_inside_a_string_literal_but_flags_one_in_a_comment()
 {
    // Mirrors the real shape in `ruleset/integrity.rs`: a full basename
    // inside a `format!()` string literal (a deliberate exclusion, documented
    // in this file's module doc comment) alongside one in a `///` comment
    // (which must still be flagged). `comment_blocks` only collects
    // `///`/`//!`/`//` lines, so the string-literal line is never even
    // handed to the detector — this test proves that end-to-end through the
    // same `comment_blocks` -> `find_dotmd_citations` pipeline the live-tree
    // guard uses, not just by asserting on `find_dotmd_citations` in
    // isolation.
    let source = "\
fn validate(errors: &mut Vec<String>) {\n\
    errors.push(format!(\n\
        \"a clamp does not clear the first aging-point row \
(Ars Magica - Definitive Edition (Core Rules).md:16575), which holds only while below it\"\n\
    ));\n\
}\n\
\n\
/// Undocumented and untested: The Divine (Revised).md:1975 (mangled basename).\n\
fn documented() {}\n";
    let mut offenders = Vec::new();
    for (_, text) in comment_blocks(source) {
        offenders.extend(find_dotmd_citations(&text));
    }
    assert_eq!(
        offenders.len(),
        1,
        "expected the string-literal basename to be ignored and only the comment's mangled \
         basename to be flagged, found {offenders:?}"
    );
    assert!(offenders[0].contains("Revised).md:1975"));
}

#[test]
fn comment_blocks_joins_contiguous_doc_comment_lines_and_breaks_on_code() {
    let source = "\
/// First line of a doc comment,\n\
/// second line continues it.\n\
fn not_a_comment() {}\n\
// Then a separate line comment.\n";
    let blocks = comment_blocks(source);
    assert_eq!(
        blocks,
        vec![
            (
                1,
                "First line of a doc comment, second line continues it.".to_string()
            ),
            (4, "Then a separate line comment.".to_string()),
        ]
    );
}
