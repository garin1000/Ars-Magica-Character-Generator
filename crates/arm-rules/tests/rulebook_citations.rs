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
//! This file is that guard.
//!
//! **D1a** scoped it to `crates/arm-rules/src` and `crates/arm-app/src`.
//! **D1b** widened [`citation_roots`] to `crates/arm-rules/tests` and
//! `crates/arm-app/tests`, and added [`markdown_citation_files`] +
//! [`markdown_blocks`] so `crates/arm-rules/RULES.md` — prose, not Rust
//! comments — is covered too. **D1c** (this slice) adds the last two roots:
//! [`web_source_files`] (`ui/src`'s `.ts`/`.svelte`/`.css`, via the
//! [`web_comment_blocks`] multi-comment-syntax pipeline) and
//! [`docs_markdown_files`] (every `.md` directly under `docs/`, folded into
//! [`markdown_citation_files`]). D1c called the root list complete, but it
//! missed `ui/e2e`: P8-0's census (`docs/vf-audit/p8-0-census.md` § 1c) found
//! some sixty full-basename, `Core Rules.md`, `Core:` and bare citations there,
//! unseen by any guard. **P8-0** folds [`ui_e2e_root`]'s `.js` files into
//! [`web_source_files`], whose comment syntax they share.
//!
//! This is a different subject from `rules_md_citations.rs`, which guards
//! *implementation-site* citations (`RULES.md` pointing at Rust code, and
//! Rust comments pointing back at `RULES.md`) — not rulebook citations.
//!
//! **Deliberate exclusion: string literals in `.rs` source.**
//! [`comment_blocks`] only collects `///`/`//!`/`//` lines, so a citation
//! inside a Rust **string literal** is invisible to every `.rs` detector in
//! this file — e.g. `crates/arm-rules/src/ruleset/integrity.rs` spells `Ars
//! Magica - Definitive Edition (Core Rules).md:NNNN` seven times inside
//! `errors.push(format!(...))` diagnostics built by `RulesetIntegrity`'s
//! validators (`IntegrityError`, surfaced through `RulesetError` when a
//! ruleset — including a hand-edited `rules/core/*.json`, which
//! `CLAUDE.md`'s trust model treats as the file the user opens) fails a
//! structural check. This is **kept on purpose**: these are diagnostics for
//! whoever is editing the rules JSON, in the same spirit as `CLAUDE.md`'s
//! "fail loudly with clear error listing offending IDs" — the reader needs to
//! open a specific rulebook, and spelling it out in full removes any need to
//! know the nine-item acronym table to act on the message.
//!
//! **E4 makes that premise an honest one.** When this exclusion was written the
//! messages went nowhere: `AppError::Ruleset` carried them across IPC and the
//! frontend read the field in no component, so "the reader" was hypothetical and
//! the carve-out rested on nobody ever seeing the text. They are now *shown*,
//! and shown **as developer diagnostics** — printed to stderr by
//! `error.rs::write_ruleset_diagnostics` on a failed load, and offered behind a
//! collapsed "technical details" disclosure (`ErrorDetails.svelte`) on both of
//! the app's error surfaces. The payload stays English by decision, for the same
//! reason the basename is spelled out in full: its value is matching, verbatim,
//! the files and ids the rules editor is looking at. So the exclusion is no
//! longer "invisible text nobody has to read" but "text deliberately addressed
//! to someone who has the rulebook open", which is a stronger reason to keep the
//! full basename, not a weaker one.
//! `dotmd_citation_detector_ignores_a_dot_md_path_inside_a_string_literal_but_flags_one_in_a_comment`
//! proves this exclusion is intentional and mechanical (it falls out of
//! `comment_blocks`'s existing "comments only" scope), not accidental.
//!
//! D1b's sweep of `crates/*/tests` found the same shape recurring in
//! `assert!`/`assert_eq!` failure-message string literals (e.g.
//! `data_integrity.rs`: `"{id} is locality-dependent (Ars Magica -
//! Definitive Edition (Core Rules).md:6160)"`). Those are **not** given the
//! `ruleset/integrity.rs` exemption above — a test failure message has no
//! rules-editor reading it without the acronym table, and every sample
//! checked was a trailing message argument (never the value under
//! comparison), so converting it to the acronym is safe and was done by hand
//! alongside the comment sweep, even though this guard (matching
//! `comment_blocks`'s scope) does not and will not enforce it there.
//!
//! **Markdown scanning mode.** RULES.md's whole structure is prose — rule,
//! verbatim rulebook excerpt, source citation, implementing function — so
//! there is no comment leader to key off the way `comment_blocks` does for
//! Rust. [`markdown_blocks`] instead joins contiguous non-blank lines (a
//! Markdown paragraph/list-item/table run, the closest analogue to a
//! comment's contiguous run) and drops fenced code block bodies entirely.
//! Two hazards drove that shape, both real in the live document:
//!
//!  - **Fenced code blocks quote JSON/pseudocode**, never a rulebook
//!    citation in this document (verified: none of RULES.md's three fences
//!    contain one) — but a future example could coincidentally contain a
//!    `key: 123`-shaped shape, so the scanner drops fence bodies rather than
//!    trusting that to stay true. [`markdown_blocks_skips_fenced_code_bodies`]
//!    proves a citation-shaped token inside a fence is invisible, and one
//!    just outside it is not.
//!  - **Verbatim rulebook excerpts** (block-quoted book prose) legitimately
//!    contain arbitrary digits and colons (ratios, times, page-internal
//!    numbering) that must not be invented into citations. No new
//!    discrimination logic was needed for this: [`find_bare_citations`]
//!    already requires a backtick or open-paren *immediately* before the
//!    colon (proved by
//!    `bare_citation_detector_flags_real_bare_citations_and_ignores_near_misses`,
//!    which includes exactly this kind of near-miss), a shape ordinary quoted
//!    prose does not produce by accident.
//!
//! **Book declarations without a citation are not citations.** RULES.md
//! names a book twice without citing a line: the `## <basename>.md` section
//! heading, and the "Other available books" list of not-yet-implemented
//! sourcebooks. Rather than special-case "heading" or "list item" lines (a
//! classification that would need to be kept in sync with the document's
//! structure), [`find_full_basename_citations`] requires the basename to be
//! immediately followed by `:NNNN` before counting it — the same test
//! [`find_dotmd_citations`] already applies. A bare mention with no line
//! number therefore is never a citation, in a heading or anywhere else, with
//! no separate exemption list to maintain.
//! [`full_basename_citation_detector_ignores_a_bare_book_mention_but_flags_a_real_citation`]
//! proves both halves.
//!
//! `RULES.md:NNNN` self-references belong to `rules_md_citations.rs`, not
//! this file — [`find_bare_citations`]'s doc comment already explains why a
//! letter (not a backtick/paren) before the colon keeps the two apart.
//!
//! **Paren-bare citations in RULES.md are implementation sites, not
//! rulebook citations.** `rules_md_citations.rs` already owns a *different*
//! bare-citation convention in this same document: `` `validate_caps`
//! (:22) `` pins a Rust function at line 22 of a nearby-cited `.rs` file, and
//! that guard's own `citation_offsets` comment notes "a hyphen would make it
//! a range, which implementation sites never use". Reusing
//! [`find_bare_citations`] naively against RULES.md therefore misreads every
//! one of those (~24 in the live document) as an *unlabelled rulebook*
//! citation and "fixes" it into nonsense (`` `validate_caps` (ArMDE:22) ``,
//! silently corrupting the very citation `rules_md_citations.rs` checks —
//! caught only by running that file's own test suite after the sweep, which
//! is exactly why both guards are exercised together in CI). A **range**
//! paren-bare citation (`` (:4399-4422) ``) is unambiguous — implementation
//! sites never use one — and is handled by the ordinary path. A **single-number**
//! paren-bare citation is ambiguous by shape alone, so
//! [`has_preceding_rs_backtick_token`] applies the same signal
//! `rules_md_citations.rs`'s own `resolve_target` uses: a backtick-wrapped
//! `.rs` token earlier in the same block means "implementation site, not
//! ours". [`find_bare_rulebook_citations_in_markdown`] is the wrapper that
//! applies this only to the Markdown scan; Rust-source scanning is
//! unaffected and still uses [`find_bare_citations`] directly, since no
//! `.rs` comment in this codebase uses the implementation-site convention.
//!
//! **This guard excludes its own test files, and `rules_md_citations.rs`.**
//! Widening [`citation_roots`] to `crates/arm-rules/tests` makes the guard
//! scan its own source for the first time, and its doc comments necessarily
//! discuss citation *shapes* with realistic-looking illustrative examples —
//! `` `:22` `` and `The Divine (Revised).md:1975` in `rules_md_citations.rs`
//! and this file's own module doc comment, deliberately mangled fixtures
//! that exist to document and test the detectors above, not to record real
//! project provenance. Left unexcluded, the guard would flag its own
//! examples as violations of itself. [`SELF_EXCLUDED_FILES`] is the same
//! kind of deliberate carve-out as the string-literal exclusion above, not
//! an oversight.
//!
//! **German-provenance citations are a different, valid convention, not a
//! mangled English one.** Widening [`find_dotmd_citations`] to RULES.md
//! surfaced a real false-positive class: RULES.md and `commands.rs`
//! legitimately cite German sourcebooks and translation tables
//! (`Ars Magica Definitive Edition Basisregeln.md:9524`,
//! `translation-tables/grundbegriffe.md:112`) per `CLAUDE.md`'s own "Rules
//! provenance" / "German translation tables" sections — a `file.md:line`
//! shape that has nothing to do with the nine-book English acronym system
//! this guard enforces. [`german_source_basenames`] walks the real
//! `rules/source/de/` tree so this exclusion self-maintains as translation
//! tables are added, rather than a hardcoded list drifting out of sync.
//! [`GERMAN_SHORTHAND_BASENAMES`] additionally covers the one shorthand this
//! repo's prose actually uses (`Basisregeln.md` for the German core book) —
//! deliberately *not* generalised to "last word of every German title",
//! because `Societates.md` would then collide with a genuine shorthand
//! attempt at the *English* Houses of Hermes: Societates book, which must
//! still be caught. Genuinely mangled English shorthands survive this
//! exclusion untouched — `commands.rs` also has six `Core Rules.md:NNNN`
//! citations (missing the `Ars Magica - Definitive Edition (` prefix), and
//! those are still flagged and were fixed in the sweep.
//!
//! **P8-1/T3** (`docs/vf-audit/design-p8-resync.md` § "P8-1", `docs/open-todos.md`
//! "P8 is a line-number earthquake") strengthens this guard's central blind
//! spot: every test above checks that a cited range lands on non-blank
//! lines, never that those lines say what the citing comment claims. A
//! wholesale line-number shift (a rulebook re-sync) would leave every
//! citation here passing while pointing at the wrong text. T3 adds an
//! **expected-excerpt snapshot**: each distinct `(acronym, range)` a citation
//! names gets a stored first-line excerpt
//! (`crates/arm-rules/tests/fixtures/citation_excerpts.json`, produced by the
//! `#[ignore]`d `regenerate_citation_excerpts`, mirroring
//! `rules_source_provenance.rs`'s `regenerate_source_anchors`), and the guard
//! fails on a mismatch, on a citation with no snapshot entry, and on a stale
//! snapshot entry no live citation resolves to (so the file can't silently
//! rot either direction). [`citation_key`] dedupes by acronym + cited range
//! rather than by citing file, since the excerpt depends only on the
//! rulebook — this keeps the snapshot sized by distinct passages, not by
//! citation occurrences. This slice (Phase 1) lands the checking logic and
//! its tests against synthetic fixtures only, proven against known-good data
//! before Phase 2 wires it into the live corpus and generates the real
//! snapshot — see Q7's recommendation to land T3 before the resync itself.

mod citation_support;

use citation_support::{
    comment_blocks, relative, repo_root, rust_files, source_files_in, web_comment_blocks,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn rules_source_en() -> PathBuf {
    repo_root().join("rules/source/en")
}

fn rules_source_de() -> PathBuf {
    repo_root().join("rules/source/de")
}

/// Every `.md` basename under `rules/source/de/` (recursively, so
/// `translation-tables/` is included), gathered from the real directory tree
/// rather than hardcoded, so a new German sourcebook or translation table is
/// picked up automatically. See this file's module doc comment,
/// "German-provenance citations are a different, valid convention".
fn german_source_basenames() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_md_basenames(&rules_source_de(), &mut names);
    names
}

fn collect_md_basenames(dir: &Path, out: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_md_basenames(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md")
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
        {
            out.insert(name.to_string());
        }
    }
}

/// The one shorthand this repo's own prose actually uses for a German
/// source file instead of its full basename: `Basisregeln.md` for the German
/// core rulebook. Not generalised to "last word of every German basename" —
/// see the module doc comment for why (`Societates.md` collision risk).
const GERMAN_SHORTHAND_BASENAMES: &[&str] = &["Basisregeln.md"];

/// Project-document basenames that are legitimately cross-referenced with a
/// line number (`` `crates/arm-rules/RULES.md:NNNN` ``, `` `CLAUDE.md:NNNN` `` —
/// spelled with a placeholder here rather than a real line number, since a
/// real one would itself trip `rules_md_citations.rs`'s own
/// `no_source_comment_cites_rules_md_by_line_number`, which has no
/// self-exclusion and scans this file too)
/// and so must not be flagged by [`find_dotmd_citations`] — a different class
/// from the German-provenance exclusion below, but the same kind of
/// deliberate carve-out. D1c's widening to `docs/` and `ui/src` is what
/// surfaced these: `crates/*/src` and `crates/*/tests` comments never
/// happened to cite either file by line number, so D1a/D1b never needed this
/// entry, but project docs cross-reference both routinely.
const NON_RULEBOOK_PROJECT_DOC_BASENAMES: &[&str] = &["RULES.md", "CLAUDE.md"];

/// The full set of `.md` basenames that are a legitimate non-rulebook
/// citation and must not be flagged by [`find_dotmd_citations`].
fn non_rulebook_md_exclusions() -> BTreeSet<String> {
    let mut names = german_source_basenames();
    names.extend(GERMAN_SHORTHAND_BASENAMES.iter().map(|s| s.to_string()));
    names.extend(
        NON_RULEBOOK_PROJECT_DOC_BASENAMES
            .iter()
            .map(|s| s.to_string()),
    );
    names
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
/// `grundbegriffe.md:212` itself documents as an alternate. `Core` is a third
/// shorthand D1c's `ui/src` sweep found still in use (`` (Core:2437) ``,
/// `` (Core:16547-16561) ``) — exactly the `Core:NNNN` shape
/// `docs/audit-2026-08.md:127` already names as one of the ad-hoc forms a
/// prior pass thought it had normalised away; it survived in `ui/src`
/// because that root was out of scope until now.
const REJECTED_SPELLINGS: &[&str] = &["ArM5", "Core"];

/// The two roots D1a scanned.
fn src_roots() -> Vec<PathBuf> {
    vec![
        repo_root().join("crates/arm-rules/src"),
        repo_root().join("crates/arm-app/src"),
    ]
}

/// The two roots D1b adds.
fn test_roots() -> Vec<PathBuf> {
    vec![
        repo_root().join("crates/arm-rules/tests"),
        repo_root().join("crates/arm-app/tests"),
    ]
}

/// Every root this guard scans for Rust-comment citations.
fn citation_roots() -> Vec<PathBuf> {
    let mut roots = src_roots();
    roots.extend(test_roots());
    roots
}

/// Every `.md` file directly under `docs/` (a flat directory as of D1c — no
/// subdirectories to recurse into).
fn docs_markdown_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let dir = repo_root().join("docs");
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// The Markdown files this guard scans in prose-scanning mode: `RULES.md`
/// (D1b) plus every file under `docs/` (D1c, the guard's final root).
fn markdown_citation_files() -> Vec<PathBuf> {
    let mut files = vec![repo_root().join("crates/arm-rules/RULES.md")];
    files.extend(docs_markdown_files());
    files
}

/// `ui/src` — D1c's non-Rust root. Its citations live in TypeScript/Svelte
/// `//` and `/* */` comments, Svelte template `<!-- -->` comments, and CSS
/// `/* */` comments — none of which [`comment_blocks`] recognises (it only
/// knows Rust's `///`/`//!`/`//`) — so this root is walked and scanned by a
/// dedicated pipeline ([`web_source_files`] + [`web_comment_blocks`]) rather
/// than folded into [`citation_roots`].
fn ui_src_root() -> PathBuf {
    repo_root().join("ui/src")
}

/// `ui/e2e` — the WebdriverIO helpers and specs, plain `.js`. P8-0 adds it: its
/// `//` and `/** */` comments are the same syntax [`web_comment_blocks`] already
/// reads for `ui/src`, so it joins that pipeline rather than getting its own.
fn ui_e2e_root() -> PathBuf {
    repo_root().join("ui/e2e")
}

/// Every `.ts`, `.svelte`, or `.css` file under [`ui_src_root`], and every `.js`
/// file under [`ui_e2e_root`], recursively.
fn web_source_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    source_files_in(&ui_src_root(), &["ts", "svelte", "css"], &mut files);
    source_files_in(&ui_e2e_root(), &["js"], &mut files);
    files.sort();
    files
}

/// True when `cell` (already trimmed) is *exactly* a bare citation number —
/// `:NNNN` or `:NNNN-MMMM`, nothing else — the shape a Markdown table cell
/// uses (`| :5006 |`) instead of prose's backtick/paren wrapping. A table
/// cell has no backtick or paren immediately before its colon for
/// [`find_bare_citations`] to key off, so this is a separate, narrower check:
/// requiring the *entire* cell to be the citation is what a real table row
/// produces, and what an incidental colon (a time, a ratio) inside a longer
/// cell does not.
fn is_bare_table_cell_citation(cell: &str) -> bool {
    if !cell.starts_with(':') {
        return false;
    }
    matches!(parse_citation_number(cell, 0), Some((_, _, end)) if end == cell.len())
}

/// Finds every bare table-cell citation in a Markdown block — see
/// [`is_bare_table_cell_citation`]. Splitting on `|` is meaningless for a
/// non-table block (it simply yields the block's own text as a single
/// "cell", which fails the check and contributes nothing).
fn find_bare_table_cell_citations(text: &str) -> Vec<String> {
    text.split('|')
        .map(str::trim)
        .filter(|cell| is_bare_table_cell_citation(cell))
        .map(str::to_string)
        .collect()
}

/// This guard's own test files, excluded from every scan below — see the
/// module doc comment, "This guard excludes its own test files".
const SELF_EXCLUDED_FILES: &[&str] = &["rulebook_citations.rs", "rules_md_citations.rs"];

fn all_rust_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in citation_roots() {
        rust_files(&root, &mut files);
    }
    files.retain(|f| {
        f.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| !SELF_EXCLUDED_FILES.contains(&name))
    });
    files.sort();
    files
}

/// The subset of [`all_rust_files`] that live under one of [`test_roots`].
fn test_root_rust_files() -> Vec<PathBuf> {
    all_rust_files()
        .into_iter()
        .filter(|f| test_roots().iter().any(|root| f.starts_with(root)))
        .collect()
}

/// The subset of [`all_rust_files`] that live under one of [`src_roots`] — D1a's
/// original two roots, which (unlike [`test_roots`], [`ui_src_root`], and
/// [`docs_markdown_files`]) never got their own vacuous-pass floor when they were
/// the only roots this guard scanned. D1c closes that gap while completing the
/// root list.
fn src_root_rust_files() -> Vec<PathBuf> {
    all_rust_files()
        .into_iter()
        .filter(|f| src_roots().iter().any(|root| f.starts_with(root)))
        .collect()
}

/// Markdown's analogue of [`comment_blocks`]: joins each contiguous run of
/// non-blank lines (a paragraph, list, table, or block-quote run — Markdown's
/// blank line is the paragraph boundary the way a non-comment line is Rust's
/// comment-run boundary) into one logical string, and drops fenced code
/// block bodies entirely so an embedded JSON/pseudocode example can never
/// manufacture a citation. See this file's module doc comment ("Markdown
/// scanning mode") for why fences are dropped rather than scanned, and why
/// no separate heading/list-item exemption is needed here.
fn markdown_blocks(content: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut current: Option<(usize, String)> = None;
    let mut in_fence = false;
    let flush = |current: &mut Option<(usize, String)>, blocks: &mut Vec<(usize, String)>| {
        if let Some(block) = current.take() {
            blocks.push(block);
        }
    };
    for (idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            flush(&mut current, &mut blocks);
            continue;
        }
        if in_fence || trimmed.is_empty() {
            flush(&mut current, &mut blocks);
            continue;
        }
        match &mut current {
            Some((_, joined)) => {
                joined.push(' ');
                joined.push_str(trimmed);
            }
            None => current = Some((idx + 1, trimmed.to_string())),
        }
    }
    flush(&mut current, &mut blocks);
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
/// of the canonical acronym. Requires an actual `:NNNN` number after the
/// spelling (not just the word followed by a colon) — needed once `Core` was
/// added to [`REJECTED_SPELLINGS`], because `docs/audit-2026-08.md` names
/// `` `Core:NNNN` `` (literal placeholder letters, no digits) as an
/// *illustrative example* of the old shorthand it once normalised away, which
/// is prose about the convention, not a live citation using it.
fn find_rejected_spellings(text: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    for &rejected in REJECTED_SPELLINGS {
        let needle = format!("{rejected}:");
        let mut search_from = 0usize;
        while let Some(rel) = text[search_from..].find(needle.as_str()) {
            let at = search_from + rel;
            let colon_at = at + rejected.len();
            if word_boundary_before(text, at) && parse_citation_number(text, colon_at).is_some() {
                found.push(rejected);
                break;
            }
            search_from = colon_at + 1;
        }
    }
    found
}

/// Finds every full basename spelled out in `text` — the spelling this slice
/// retires in favor of the acronym. Used for `.rs` comments, where
/// `comment_blocks` only ever hands this function a citation context, so any
/// basename mention found is one.
fn find_full_basenames(text: &str) -> Vec<&'static str> {
    BOOK_ACRONYMS
        .iter()
        .map(|&(_, file)| file)
        .filter(|file| text.contains(file))
        .collect()
}

/// Markdown analogue of [`find_full_basenames`]: only counts a basename
/// occurrence immediately followed by `:NNNN` — i.e. actually used as a
/// citation, not merely a book-name mention (a `## <basename>.md` heading, or
/// the "Other available books" list). See this file's module doc comment for
/// why this extra requirement is what tells the two apart, in place of a
/// heading/list-item exemption list.
fn find_full_basename_citations(text: &str) -> Vec<&'static str> {
    BOOK_ACRONYMS
        .iter()
        .map(|&(_, file)| file)
        .filter(|file| {
            let needle = format!("{file}:");
            text.find(needle.as_str())
                .and_then(|idx| text[idx + needle.len()..].chars().next())
                .is_some_and(|c| c.is_ascii_digit())
        })
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
/// was intended. `exclude` is checked as a suffix ending exactly at the `.md`
/// — any basename in it (a legitimate German-provenance citation; see the
/// module doc comment) is skipped rather than flagged. Returns a short
/// snippet of surrounding context (up to 70 bytes before, 15 after) for the
/// failure message, snapped to the nearest UTF-8 char boundary.
fn find_dotmd_citations(text: &str, exclude: &BTreeSet<String>) -> Vec<String> {
    let mut found = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = text[search_from..].find(".md:") {
        let dot_at = search_from + rel;
        let colon_at = dot_at + 3;
        let after_colon = colon_at + 1;
        let is_excluded = exclude.iter().any(|name| text[..colon_at].ends_with(name));
        if !is_excluded
            && text[after_colon..]
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
/// This same discrimination rule is what keeps a quoted rulebook excerpt's
/// own digits (times, ratios, in-book numbering) from being misread as a
/// citation when this function is reused for Markdown prose — see this
/// file's module doc comment, "Markdown scanning mode".
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

/// True when `text[..before]` contains a backtick-wrapped token with `.rs`
/// in it — the same signal `rules_md_citations.rs`'s own `resolve_target`
/// uses to resolve an implementation-site citation's file. See this file's
/// module doc comment, "Paren-bare citations in RULES.md are implementation
/// sites, not rulebook citations".
fn has_preceding_rs_backtick_token(text: &str, before: usize) -> bool {
    let bound = before.min(text.len());
    let mut search_from = 0usize;
    while let Some(rel) = text[search_from..bound].find('`') {
        let open = search_from + rel;
        let Some(close_rel) = text[open + 1..].find('`') else {
            break;
        };
        let close = open + 1 + close_rel;
        if close > bound {
            break;
        }
        if text[open + 1..close].contains(".rs") {
            return true;
        }
        search_from = close + 1;
    }
    false
}

/// Markdown-specific wrapper around [`find_bare_citations`]: drops a
/// paren-bare match preceded (within the same block) by a backtick-wrapped
/// `.rs` token, since that shape is RULES.md's own implementation-site
/// citation convention, not an unlabelled rulebook citation. A backtick-bare
/// match is never dropped this way — RULES.md's implementation sites are
/// exclusively paren- or comma-continuation-shaped, never backtick-wrapped.
fn find_bare_rulebook_citations_in_markdown(text: &str) -> Vec<String> {
    let mut found: Vec<String> = find_bare_citations(text)
        .into_iter()
        .filter(|snippet| {
            if !snippet.starts_with('(') {
                return true;
            }
            match text.find(snippet.as_str()) {
                Some(pos) => !has_preceding_rs_backtick_token(text, pos),
                None => true,
            }
        })
        .collect();
    found.extend(find_bare_table_cell_citations(text));
    found
}

/// Every [`BOOK_ACRONYMS`] book's lines, read from `rules/source/en/` exactly
/// **once** per book and reused by every citation check that needs book
/// content — `every_rulebook_citation_lands_inside_its_file`'s range check,
/// the P8-1/T3 excerpt check, and `regenerate_citation_excerpts`. Before this,
/// `book_line_count`/`book_is_blank_in_range` each re-read the whole book file
/// from disk **per citation occurrence** (thousands of times across the live
/// corpus), which is almost certainly why this suite's runtime sat at ~75s;
/// loading each of the 9 books exactly once brings that down to 9 reads total
/// regardless of how many citations are checked against them.
struct BookLinesCache {
    books: BTreeMap<&'static str, Vec<String>>,
}

impl BookLinesCache {
    /// Reads every book in [`BOOK_ACRONYMS`] from `rules/source/en/` once.
    fn load() -> Self {
        let mut books = BTreeMap::new();
        for &(_, book_file) in BOOK_ACRONYMS {
            let content = fs::read_to_string(rules_source_en().join(book_file))
                .unwrap_or_else(|e| panic!("{book_file} is readable under rules/source/en: {e}"));
            books.insert(book_file, content.lines().map(str::to_string).collect());
        }
        Self { books }
    }

    fn lines(&self, book_file: &str) -> &[String] {
        self.books
            .get(book_file)
            .unwrap_or_else(|| panic!("{book_file} is not among the cached books"))
    }

    fn line_count(&self, book_file: &str) -> usize {
        self.lines(book_file).len()
    }

    fn is_blank_in_range(&self, book_file: &str, start: i64, end: i64) -> bool {
        let lines = self.lines(book_file);
        lines[(start as usize - 1)..(end as usize)]
            .iter()
            .all(|line| line.trim().is_empty())
    }

    /// Borrows every cached book's lines as `&str`, the shape
    /// [`check_citation_excerpts`] takes — so the same single read backs both
    /// the existing range check and the new excerpt check.
    fn as_str_map(&self) -> BTreeMap<&'static str, Vec<&str>> {
        self.books
            .iter()
            .map(|(&book, lines)| (book, lines.iter().map(String::as_str).collect()))
            .collect()
    }
}

/// Every citation found across [`all_rust_files`] (via [`comment_blocks`])
/// plus [`markdown_citation_files`] (via [`markdown_blocks`]), paired with the
/// file and block-start line it came from (for error messages).
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
    let mut files_scanned = files.len();
    for path in markdown_citation_files() {
        files_scanned += 1;
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        for (line, text) in markdown_blocks(&content) {
            for citation in find_citations(&text) {
                all.push((path.clone(), line, citation));
            }
        }
    }
    for path in web_source_files() {
        files_scanned += 1;
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        for (line, text) in web_comment_blocks(&content) {
            for citation in find_citations(&text) {
                all.push((path.clone(), line, citation));
            }
        }
    }
    (files_scanned, all)
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
    for path in web_source_files() {
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in web_comment_blocks(&content) {
            for rejected in find_rejected_spellings(&text) {
                offenders.insert(format!(
                    "{}:{line}: cites the rulebook as `{rejected}`, which is not the canonical \
                     spelling — use `ArMDE`",
                    relative(&path)
                ));
            }
        }
    }
    for path in markdown_citation_files() {
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in markdown_blocks(&content) {
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

/// True for a path under `docs/` — the dated historical records
/// `design-p8-resync.md` § "What moves" (e) and D1c both exempt from ever
/// being rewritten to a post-resync line: "the line number is part of the
/// snapshot and rewriting it to today's symbol would falsify the record."
/// The corollary this function exists for: once a resync actually lands,
/// such a frozen citation can legitimately land on blank or out-of-range
/// content in the *current* book — that is not a defect to report, it is the
/// snapshot aging exactly as designed. [`every_rulebook_citation_lands_inside_its_file`]
/// must not flag it.
fn is_dated_historical_record(path: &Path) -> bool {
    path.starts_with(repo_root().join("docs"))
}

#[test]
fn is_dated_historical_record_recognizes_only_the_docs_root() {
    assert!(is_dated_historical_record(
        &repo_root().join("docs/audit-2026-08.md")
    ));
    assert!(!is_dated_historical_record(
        &repo_root().join("crates/arm-rules/RULES.md")
    ));
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

    let cache = BookLinesCache::load();
    let mut errors = Vec::new();
    for (path, line, citation) in &citations {
        if is_dated_historical_record(path) {
            continue;
        }
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
        let total = cache.line_count(book_file) as i64;
        if *end > total {
            errors.push(format!(
                "{}:{line}: cites {acronym}:{start}-{end}, but {book_file} has only {total} \
                 lines",
                relative(path)
            ));
            continue;
        }
        if cache.is_blank_in_range(book_file, *start, *end) {
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
    for path in markdown_citation_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in markdown_blocks(&content) {
            for snippet in find_bare_rulebook_citations_in_markdown(&text) {
                offenders.insert(format!(
                    "{}:{line}: bare rulebook citation {snippet} has no acronym — name the book \
                     (e.g. ArMDE:{snippet})",
                    relative(&path),
                    snippet = snippet
                ));
            }
        }
    }
    for path in web_source_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in web_comment_blocks(&content) {
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
    for path in markdown_citation_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in markdown_blocks(&content) {
            for basename in find_full_basename_citations(&text) {
                offenders.insert(format!(
                    "{}:{line}: spells the rulebook out by full basename ({basename}) instead of \
                     its acronym",
                    relative(&path)
                ));
            }
        }
    }
    for path in web_source_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in web_comment_blocks(&content) {
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

#[test]
fn no_source_comment_cites_a_rulebook_by_any_dot_md_path() {
    // Shape-only backstop for `no_source_comment_spells_a_rulebook_by_full_basename`:
    // that test only recognises the nine basenames verbatim, so a mangled or
    // partial one slips through it. This test flags the bare shape
    // `.md:NNNN` wherever it appears in a comment, regardless of what
    // precedes the `.md` — after the D1a sweep no comment should spell a
    // rulebook as a `.md` path in any spelling at all.
    let exclude = non_rulebook_md_exclusions();
    let mut offenders: BTreeSet<String> = BTreeSet::new();
    let mut scanned = 0usize;
    for path in all_rust_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in comment_blocks(&content) {
            for snippet in find_dotmd_citations(&text, &exclude) {
                offenders.insert(format!(
                    "{}:{line}: cites a rulebook as a `.md` path ({snippet}) instead of its \
                     acronym",
                    relative(&path)
                ));
            }
        }
    }
    for path in markdown_citation_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in markdown_blocks(&content) {
            for snippet in find_dotmd_citations(&text, &exclude) {
                offenders.insert(format!(
                    "{}:{line}: cites a rulebook as a `.md` path ({snippet}) instead of its \
                     acronym",
                    relative(&path)
                ));
            }
        }
    }
    for path in web_source_files() {
        scanned += 1;
        let content = fs::read_to_string(&path).unwrap();
        for (line, text) in web_comment_blocks(&content) {
            for snippet in find_dotmd_citations(&text, &exclude) {
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
        offenders.extend(find_dotmd_citations(&text, &BTreeSet::new()));
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

// ---------------------------------------------------------------------------
// D1b: widened-root and Markdown-scanning-mode fixture tests.
// ---------------------------------------------------------------------------

#[test]
fn src_roots_scan_a_nonzero_floor() {
    // D1a's original two roots (`crates/arm-rules/src`, `crates/arm-app/src`)
    // never had a dedicated floor of their own — they were the only roots this
    // guard scanned at the time, so `every_rulebook_acronym_resolves_to_a_file_under_rules_source_en`'s
    // global floor covered them implicitly. Now that D1c completes the root
    // list, every root gets the same explicit protection: a `src_roots` that
    // resolved to an empty/nonexistent directory must not let this guard pass
    // vacuously.
    let src_files = src_root_rust_files();
    assert!(
        src_files.len() > 20,
        "expected `crates/arm-rules/src` + `crates/arm-app/src` to contribute many .rs files, \
         found {}",
        src_files.len()
    );

    let mut src_citations = 0usize;
    for path in &src_files {
        let content = fs::read_to_string(path).unwrap();
        for (_, text) in comment_blocks(&content) {
            src_citations += find_citations(&text).len();
        }
    }
    assert!(
        src_citations > 100,
        "expected many acronym'd rulebook citations across the original src roots, found {}",
        src_citations
    );
}

#[test]
fn citation_roots_include_the_new_test_directories_and_scan_a_nonzero_floor() {
    // The floor that keeps a widened root from passing vacuously: a root
    // that resolves to an empty/nonexistent directory would otherwise let
    // `rust_files` silently contribute zero files and zero citations.
    let test_files = test_root_rust_files();
    assert!(
        test_files.len() > 5,
        "expected `crates/arm-rules/tests` + `crates/arm-app/tests` to contribute several .rs \
         files, found {}",
        test_files.len()
    );

    let mut test_citations = 0usize;
    for path in &test_files {
        let content = fs::read_to_string(path).unwrap();
        for (_, text) in comment_blocks(&content) {
            test_citations += find_citations(&text).len();
        }
    }
    assert!(
        test_citations > 100,
        "expected many acronym'd rulebook citations across the widened test roots, found {}",
        test_citations
    );
}

#[test]
fn all_rust_files_excludes_this_guards_own_test_files() {
    let names: BTreeSet<String> = all_rust_files()
        .iter()
        .filter_map(|f| f.file_name().and_then(|n| n.to_str()).map(String::from))
        .collect();
    for excluded in SELF_EXCLUDED_FILES {
        assert!(
            !names.contains(*excluded),
            "{excluded} must be excluded from the scan (it is this guard's own test file), \
             found it among {names:?}"
        );
    }
}

#[test]
fn markdown_citation_files_scan_a_nonzero_floor() {
    let files = markdown_citation_files();
    assert!(
        !files.is_empty(),
        "expected at least one Markdown file to be registered for scanning"
    );

    let mut markdown_citations = 0usize;
    for path in &files {
        assert!(path.is_file(), "{path:?} does not exist");
        let content = fs::read_to_string(path).unwrap();
        for (_, text) in markdown_blocks(&content) {
            markdown_citations += find_citations(&text).len();
        }
    }
    // RULES.md alone carries well over a thousand citations post-sweep; 500
    // is a generous floor that still catches a scanner silently matching
    // nothing (e.g. a fenced-block off-by-one that blanks the whole file).
    assert!(
        markdown_citations > 500,
        "expected RULES.md to carry many acronym'd rulebook citations, found {}",
        markdown_citations
    );
}

#[test]
fn markdown_blocks_skips_fenced_code_bodies() {
    let source = "\
Prose before the fence names ArMDE:100.\n\
\n\
```json\n\
\"not_a_citation\": \"`:200`\"\n\
```\n\
\n\
Prose after the fence names ArMDE:300.\n";
    let mut citations = Vec::new();
    for (_, text) in markdown_blocks(source) {
        citations.extend(find_citations(&text));
    }
    let starts: Vec<i64> = citations.iter().map(|c| c.start).collect();
    assert_eq!(
        starts,
        vec![100, 300],
        "expected the fenced block's citation-shaped content to be invisible, found {starts:?}"
    );
}

#[test]
fn markdown_blocks_joins_a_paragraph_and_breaks_on_a_blank_line() {
    let source = "\
Source: `Ars Magica - Definitive Edition (Core\n\
Rules).md:2774` continues a wrapped basename.\n\
\n\
A new paragraph after the blank line.\n";
    let blocks = markdown_blocks(source);
    assert_eq!(
        blocks,
        vec![
            (
                1,
                "Source: `Ars Magica - Definitive Edition (Core Rules).md:2774` continues a \
                 wrapped basename."
                    .to_string()
            ),
            (4, "A new paragraph after the blank line.".to_string()),
        ]
    );
}

#[test]
fn full_basename_citation_detector_ignores_a_bare_book_mention_but_flags_a_real_citation() {
    let heading = "## Ars Magica - Definitive Edition (Core Rules).md — ArMDE";
    assert_eq!(
        find_full_basename_citations(heading),
        Vec::<&str>::new(),
        "a heading names the book but cites no line, so it must not be flagged"
    );

    let list_item = "- Ars Magica 5e - Realms of Power - Faerie.md";
    assert_eq!(
        find_full_basename_citations(list_item),
        Vec::<&str>::new(),
        "a book-availability list entry cites no line, so it must not be flagged"
    );

    let real_citation = "Source: Ars Magica - Definitive Edition (Core Rules).md:2774.";
    assert_eq!(
        find_full_basename_citations(real_citation),
        vec!["Ars Magica - Definitive Edition (Core Rules).md"],
        "a basename immediately followed by `:NNNN` is a real citation and must be flagged"
    );
}

#[test]
fn markdown_bare_citation_wrapper_excludes_an_implementation_site_citation_but_keeps_a_range_and_a_backtick_one()
 {
    // Each case is its own block (as `markdown_blocks` would hand it to this
    // function one paragraph at a time), so a `.rs` token in one bullet
    // cannot "bleed" into a later, unrelated one.
    let implementation_site =
        "- Implementation: `crates/arm-rules/src/validation/caps.rs` — `validate_caps` (:22)";
    assert_eq!(
        find_bare_rulebook_citations_in_markdown(implementation_site),
        Vec::<String>::new(),
        "a paren-bare citation right after a `.rs` token is an implementation site, not ours"
    );

    let a_range = "- **Magical Focus (major/minor)** — `virtue.major_magical_focus` (:4399-4422)";
    assert_eq!(
        find_bare_rulebook_citations_in_markdown(a_range),
        vec!["(:4399-4422)".to_string()],
        "a range is never an implementation site (they never use ranges), so it must survive"
    );

    let a_backtick_one = "- Savantism `:6703` *halves* starting XP";
    assert_eq!(
        find_bare_rulebook_citations_in_markdown(a_backtick_one),
        vec!["`:6703`".to_string()],
        "a backtick-bare citation is never RULES.md's implementation-site shape"
    );

    let unlabelled_with_no_rs_nearby =
        "- A truly unlabelled paren-bare rulebook citation with no source file nearby (:9999)";
    assert_eq!(
        find_bare_rulebook_citations_in_markdown(unlabelled_with_no_rs_nearby),
        vec!["(:9999)".to_string()],
        "a paren-bare citation with no preceding `.rs` token must still be flagged"
    );
}

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
    // Mirrors the real wrap in `derived.rs::residual_voice_penalty`: a comma-continuation
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
fn a_continuation_wrapped_in_its_own_backticks_is_not_chained_and_reads_as_bare() {
    // D1c found this exact shape live in `ipc.ts::CrisisPreview`'s doc comment:
    // `` (`ArMDE:16626`, `:16627`) `` — unlike every other continuation in the
    // codebase (e.g. the fixture above, or the real
    // `(ArMDE:16602, :16611)` in `aging-workflow.svelte.ts`), the second
    // number sits in its **own** pair of backticks rather than trailing
    // inside the first citation's. [`find_citations`]'s continuation chase
    // requires a literal `, :` immediately after the previous number — here
    // the closing backtick of the first citation sits between the comma and
    // the colon (`` `, `: ``), so the chase never fires: the acronym is
    // resolved for the first number only, and the second is left exactly as
    // bare as if no acronym had ever appeared in the block. This is
    // deliberate, not a gap to special-case: [`find_bare_citations`] still
    // catches the orphaned `` `:16627` `` (proved below), so the shape is
    // already rejected by `no_source_comment_cites_a_rulebook_by_bare_line_number`
    // without the continuation-chaser needing to learn a second syntax for
    // "how do I know this trailing citation belongs to the one before it".
    // The fix at the real site was to normalize to the established one-pair
    // form (`` `ArMDE:16626, :16627` ``), matching every other continuation
    // in this codebase, rather than teaching the parser a second accepted
    // shape.
    let double_backtick_form = "which is time rather than a roll (`ArMDE:16626`, `:16627`).";
    let citations = find_citations(double_backtick_form);
    assert_eq!(
        citations,
        vec![Citation {
            acronym: "ArMDE",
            book_file: "Ars Magica - Definitive Edition (Core Rules).md",
            start: 16626,
            end: 16626,
        }],
        "expected only the first number to resolve as a citation; the second is not chained"
    );
    assert_eq!(
        find_bare_citations(double_backtick_form),
        vec!["`:16627`".to_string()],
        "expected the un-chained second number to still read as a bare citation, so the \
         no-bare-citations guard rejects this shape on its own"
    );

    let normalized_one_pair_form = "which is time rather than a roll (`ArMDE:16626, :16627`).";
    let citations = find_citations(normalized_one_pair_form);
    assert_eq!(
        citations,
        vec![
            Citation {
                acronym: "ArMDE",
                book_file: "Ars Magica - Definitive Edition (Core Rules).md",
                start: 16626,
                end: 16626,
            },
            Citation {
                acronym: "ArMDE",
                book_file: "Ars Magica - Definitive Edition (Core Rules).md",
                start: 16627,
                end: 16627,
            },
        ],
        "the normalized single-backtick-pair form must chain both numbers to ArMDE"
    );
    assert!(
        find_bare_citations(normalized_one_pair_form).is_empty(),
        "the normalized form must leave no bare citation behind"
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
    // The real near-misses this backstop exists for: a partial basename (in
    // `types.rs::MightGrant`'s doc comment) and a differently-mangled one
    // missing the `Ars Magica - ` / parenthetical parts (in
    // `xp.rs::ability_authorizations_reads_only_the_three_permission_granting_effects`'s),
    // plus a correctly
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
    let found = find_dotmd_citations(fixture, &BTreeSet::new());
    assert_eq!(
        found.len(),
        3,
        "expected exactly the three `.md:NNNN` shapes, found {found:?}"
    );
}

#[test]
fn dotmd_citation_detector_excludes_a_known_non_rulebook_md_file_but_still_flags_a_real_mangled_one()
 {
    // RULES.md and `commands.rs` legitimately cite German sourcebooks and
    // translation tables this way — see the module doc comment,
    // "German-provenance citations are a different, valid convention".
    let exclude: BTreeSet<String> = [
        "Ars Magica Definitive Edition Basisregeln.md",
        "Basisregeln.md",
        "grundbegriffe.md",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let fixture = "\
German shorthand is not a rulebook citation: Basisregeln.md:3329.\n\
Full German name is not a rulebook citation either: Ars Magica Definitive Edition \
Basisregeln.md:9524.\n\
A translation table is not a rulebook citation: grundbegriffe.md:112.\n\
A real mangled English citation must still be flagged: Core Rules.md:2437.\n\
";
    let found = find_dotmd_citations(fixture, &exclude);
    assert_eq!(
        found.len(),
        1,
        "expected only the mangled English citation to survive the exclusion, found {found:?}"
    );
    assert!(found[0].contains("Core Rules.md:2437"));
}

#[test]
fn german_source_basenames_finds_the_german_core_rulebook_on_disk() {
    // A floor against the walker silently finding nothing (e.g. a wrong
    // `rules_source_de` path), mirroring the same floor pattern used
    // throughout this file for the English side.
    let names = german_source_basenames();
    assert!(
        names.contains("Ars Magica Definitive Edition Basisregeln.md"),
        "expected the German core rulebook among {names:?}"
    );
    assert!(
        names.contains("grundbegriffe.md"),
        "expected a translation table among {names:?}"
    );
}

// ---------------------------------------------------------------------------
// D1c: `ui/src` (multi-comment-syntax) and `docs/` (Markdown, final root).
// ---------------------------------------------------------------------------

#[test]
fn ui_src_files_scan_a_nonzero_floor() {
    // The same vacuous-pass guard as `citation_roots_include_the_new_test_directories…`
    // (D1b), applied to the new non-Rust root: a wrong `ui_src_root` or a
    // `web_source_files` walker that silently finds nothing must not let this
    // guard pass by default.
    let files = web_source_files();
    assert!(
        files.len() > 20,
        "expected `ui/src` to contribute many .ts/.svelte/.css files, found {}",
        files.len()
    );

    let mut citations = 0usize;
    for path in &files {
        let content = fs::read_to_string(path).unwrap();
        for (_, text) in web_comment_blocks(&content) {
            citations += find_citations(&text).len();
        }
    }
    assert!(
        citations > 50,
        "expected many acronym'd rulebook citations across ui/src, found {}",
        citations
    );
}

#[test]
fn ui_e2e_files_scan_a_nonzero_floor() {
    // P8-0's census (`docs/vf-audit/p8-0-census.md` § 1c) found `ui/e2e` outside
    // every root, so its citations had drifted to the full-basename and
    // `Core Rules.md` forms unseen. The same vacuous-pass guard as the other
    // roots: the e2e helpers and specs must actually be walked and scanned.
    let e2e_root = ui_e2e_root();
    let files: Vec<PathBuf> = web_source_files()
        .into_iter()
        .filter(|f| f.starts_with(&e2e_root))
        .collect();
    assert!(
        files.len() > 10,
        "expected `ui/e2e` to contribute many .js files, found {}",
        files.len()
    );

    let mut citations = 0usize;
    for path in &files {
        let content = fs::read_to_string(path).unwrap();
        for (_, text) in web_comment_blocks(&content) {
            citations += find_citations(&text).len();
        }
    }
    assert!(
        citations > 10,
        "expected many acronym'd rulebook citations across ui/e2e, found {}",
        citations
    );
}

#[test]
fn docs_markdown_files_scan_a_nonzero_floor() {
    let files = docs_markdown_files();
    assert!(
        files.len() > 3,
        "expected several files directly under docs/, found {}",
        files.len()
    );

    let mut citations = 0usize;
    for path in &files {
        let content = fs::read_to_string(path).unwrap();
        for (_, text) in markdown_blocks(&content) {
            citations += find_citations(&text).len();
        }
    }
    assert!(
        citations > 100,
        "expected many acronym'd rulebook citations across docs/, found {}",
        citations
    );
}

#[test]
fn web_comment_blocks_finds_a_citation_in_a_line_comment_and_ignores_surrounding_prose() {
    let source = "\
const x = 1; // not a comment line, so this citation must be invisible: ArMDE:9999\n\
// Source: ArMDE:2774 is inside a real line-comment run\n\
// and continues onto a second line.\n\
const y = 2;\n";
    let mut citations = Vec::new();
    for (_, text) in web_comment_blocks(source) {
        citations.extend(find_citations(&text));
    }
    let starts: Vec<i64> = citations.iter().map(|c| c.start).collect();
    assert_eq!(
        starts,
        vec![2774],
        "expected only the `//`-comment citation, found {starts:?}"
    );
}

#[test]
fn web_comment_blocks_finds_a_citation_in_a_block_comment_and_ignores_surrounding_code() {
    let source = "\
const notACitation = 'ArMDE:9999';\n\
/* A floor of about three rows (ArMDE:2774): this list is the only way to\n\
   pick anything. */\n\
.selector { color: red; }\n";
    let mut citations = Vec::new();
    for (_, text) in web_comment_blocks(source) {
        citations.extend(find_citations(&text));
    }
    let starts: Vec<i64> = citations.iter().map(|c| c.start).collect();
    assert_eq!(
        starts,
        vec![2774],
        "expected only the block-comment citation, found {starts:?}"
    );
}

#[test]
fn web_comment_blocks_finds_a_citation_in_a_jsdoc_comment() {
    let source = "\
/**\n\
 * One Crisis's total (ArMDE:2774). All three terms are added, and\n\
 * the rest of this line is prose, not a citation: 9999.\n\
 */\n\
export interface CrisisTotal {}\n";
    let mut citations = Vec::new();
    for (_, text) in web_comment_blocks(source) {
        citations.extend(find_citations(&text));
    }
    let starts: Vec<i64> = citations.iter().map(|c| c.start).collect();
    assert_eq!(
        starts,
        vec![2774],
        "expected only the JSDoc citation, found {starts:?}"
    );
}

#[test]
fn web_comment_blocks_finds_a_citation_in_an_html_comment_and_ignores_surrounding_markup() {
    let source = "\
<p data-testid=\"ArMDE:9999\">not a comment, must be invisible</p>\n\
<!-- \"Terminal illness.\" (ArMDE:2774) —\n\
     an absent Ease Factor is NO roll. -->\n\
<p>rendered text</p>\n";
    let mut citations = Vec::new();
    for (_, text) in web_comment_blocks(source) {
        citations.extend(find_citations(&text));
    }
    let starts: Vec<i64> = citations.iter().map(|c| c.start).collect();
    assert_eq!(
        starts,
        vec![2774],
        "expected only the HTML-comment citation, found {starts:?}"
    );
}

#[test]
fn web_comment_blocks_finds_a_citation_in_a_css_comment() {
    let source = "\
.selector {\n\
  content: 'ArMDE:9999'; /* not a real comment body, this whole line is code */\n\
}\n\
/* Placement is row-major, so the book's own order (ArMDE:2774)\n\
   reads across each row and then down. */\n\
.other { display: grid; }\n";
    let mut citations = Vec::new();
    for (_, text) in web_comment_blocks(source) {
        citations.extend(find_citations(&text));
    }
    let starts: Vec<i64> = citations.iter().map(|c| c.start).collect();
    assert_eq!(
        starts,
        vec![2774],
        "expected only the CSS block-comment citation, found {starts:?}"
    );
}

#[test]
fn find_bare_table_cell_citations_flags_a_pure_citation_cell_but_ignores_a_normal_cell() {
    let table_row = "| `virtue.spirit_votary` | :5006 | RoP: Magic |";
    assert_eq!(
        find_bare_table_cell_citations(table_row),
        vec![":5006".to_string()],
        "expected exactly the pure-citation cell to be flagged"
    );

    let range_cell = "| Some Virtue | :3331-3334 | Core |";
    assert_eq!(
        find_bare_table_cell_citations(range_cell),
        vec![":3331-3334".to_string()],
        "a range-shaped pure-citation cell must be flagged too"
    );

    let non_citation_row = "| `virtue.something` | see :5006 in prose | Core |";
    assert_eq!(
        find_bare_table_cell_citations(non_citation_row),
        Vec::<String>::new(),
        "a cell that merely contains a bare citation amid other words is not this shape — \
         `find_bare_citations` already handles a backtick/paren-preceded one, and prose \
         inside a cell is not this guard's subject"
    );
}

#[test]
fn docs_markdown_files_includes_a_known_file_and_excludes_non_markdown() {
    let names: BTreeSet<String> = docs_markdown_files()
        .iter()
        .filter_map(|f| f.file_name().and_then(|n| n.to_str()).map(String::from))
        .collect();
    assert!(
        names.contains("open-todos.md"),
        "expected docs/open-todos.md among {names:?}"
    );
}

// ---------------------------------------------------------------------------
// P8-1/T3: excerpt-snapshot strengthening. See this file's module doc
// comment for the design. Landed for real in `efbccd51`: `fixtures/
// citation_excerpts.json` is a live, regenerated snapshot and
// `every_citation_excerpt_matches_its_snapshot` runs against the real corpus.
//
// **Extended for P8-1/T2** (text-search relocation,
// `crates/arm-rules/tests/citation_relocation.rs`): each snapshot entry now
// also carries `next_line` — the normalized text of the line immediately
// following the cited one. T2's relocation tool needs this to disambiguate a
// cited line whose own text recurs elsewhere in the same book (e.g.
// `"*Minor, General*<br>"`), and it must be captured **now**, against today's
// pre-resync files, or the context is gone once the sync happens. See
// `check_citation_excerpts_flags_a_mismatched_next_line_even_when_the_excerpt_itself_still_matches`
// for the new field's own red-then-green test.
// ---------------------------------------------------------------------------

/// The maximum byte length of a normalized excerpt — long enough to catch a
/// real wording change, short enough to keep the committed snapshot small.
const EXCERPT_MAX_CHARS: usize = 80;

/// Trims `line` and caps it to [`EXCERPT_MAX_CHARS`] bytes, snapped to the
/// nearest earlier UTF-8 char boundary so a multi-byte character (e.g. a
/// German umlaut) is never split mid-codepoint.
fn normalize_excerpt(line: &str) -> String {
    let trimmed = line.trim();
    let mut end = trimmed.len().min(EXCERPT_MAX_CHARS);
    while end > 0 && !trimmed.is_char_boundary(end) {
        end -= 1;
    }
    trimmed[..end].to_string()
}

/// The canonical snapshot key for a citation: `ACRONYM:START` for a
/// single-line citation, `ACRONYM:START-END` for a range. Two citations into
/// the same range — however many files repeat it — share one key, which is
/// why the snapshot is sized by distinct passages, not by citation
/// occurrences (see this file's module doc comment).
fn citation_key(citation: &Citation) -> String {
    if citation.start == citation.end {
        format!("{}:{}", citation.acronym, citation.start)
    } else {
        format!("{}:{}-{}", citation.acronym, citation.start, citation.end)
    }
}

/// One committed snapshot entry: the cited line's own normalized excerpt,
/// plus the normalized text of the nearest **non-blank** line following it —
/// the disambiguating context `citation_relocation.rs`'s relocation tool
/// needs (see this section's header comment). Blank lines are skipped
/// deliberately: a Markdown paragraph is almost always followed by a blank
/// separator line, so the literal next line is empty for roughly half the
/// live corpus (measured while building this) and would carry no
/// disambiguating signal at all. `next_line` is `""` only when no non-blank
/// line remains before the end of the book.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ExcerptSnapshotEntry {
    excerpt: String,
    next_line: String,
    /// The nearest Markdown heading line preceding the cited one, normalized.
    /// A second, independent disambiguation signal alongside `next_line` —
    /// the live corpus has passages (repeated statblock fields) whose cited
    /// line *and* its follower are both duplicated across occurrences, so
    /// `next_line` alone cannot narrow them, but their enclosing section
    /// heading differs. Captured **now**, against today's pre-resync files,
    /// for the same reason `next_line` is (see this section's header
    /// comment). `""` when no heading precedes the cited line.
    heading: String,
}

/// Finds the nearest non-blank line at or after `start_index` (0-based) in
/// `lines`, normalized — see [`ExcerptSnapshotEntry::next_line`] for why
/// blank lines are skipped rather than taking the literal next one. Returns
/// `""` when every remaining line is blank (or `start_index` is past the end).
fn next_non_blank_excerpt<S: AsRef<str>>(lines: &[S], start_index: usize) -> String {
    let Some(rest) = lines.get(start_index..) else {
        return String::new();
    };
    rest.iter()
        .map(AsRef::as_ref)
        .find(|line| !line.trim().is_empty())
        .map(normalize_excerpt)
        .unwrap_or_default()
}

/// Finds the nearest Markdown heading line (trimmed text starting with `#`)
/// strictly before `before_index` (0-based — callers pass the cited line's
/// own index, so the cited line itself is never mistaken for its own
/// heading), scanning backward, normalized. See
/// [`ExcerptSnapshotEntry::heading`]. Returns `""` if no heading precedes it.
fn nearest_preceding_heading_excerpt<S: AsRef<str>>(lines: &[S], before_index: usize) -> String {
    lines[..before_index.min(lines.len())]
        .iter()
        .map(AsRef::as_ref)
        .rev()
        .find(|line| line.trim_start().starts_with('#'))
        .map(normalize_excerpt)
        .unwrap_or_default()
}

/// One excerpt-snapshot discrepancy, bucketed by kind so a test can assert on
/// each kind independently.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ExcerptCheckResult {
    /// A citation whose snapshot entry exists but no longer matches the
    /// cited range's actual (normalized) first line. Each entry names the
    /// citing file:line, the cited range, and the expected vs. actual text.
    mismatches: Vec<String>,
    /// A citation with no snapshot entry at all, naming the citing
    /// file:line and the cited range.
    missing: Vec<String>,
    /// A snapshot entry that no citation's [`citation_key`] resolves to,
    /// naming the stale key.
    stale: Vec<String>,
}

/// Checks every found citation's cited range against `snapshot`, using
/// `book_lines` (book basename -> that book's lines, in cited-line order) as
/// the rulebook content. A real run reads this once per book from
/// `rules/source/en/`; a test supplies a tiny synthetic book so the check can
/// be proven without touching the filesystem.
///
/// Message shapes:
/// - mismatch: `"{file}:{line}: cites {key}, expected first line {expected:?} (next line {expected_next:?}), found {actual:?} (next line {actual_next:?})"`
/// - missing: `"{file}:{line}: cites {key}, which has no snapshot entry"`
/// - stale: `"{key}: stale snapshot entry — no citation uses this range"`
fn check_citation_excerpts(
    citations: &[(PathBuf, usize, Citation)],
    book_lines: &BTreeMap<&'static str, Vec<&str>>,
    snapshot: &BTreeMap<String, ExcerptSnapshotEntry>,
) -> ExcerptCheckResult {
    let mut result = ExcerptCheckResult::default();
    let mut used_keys: BTreeSet<String> = BTreeSet::new();
    for (path, line, citation) in citations {
        let key = citation_key(citation);
        used_keys.insert(key.clone());
        let lines = book_lines.get(citation.book_file);
        let actual_excerpt = lines
            .and_then(|lines| lines.get((citation.start as usize).saturating_sub(1)))
            .map(|text| normalize_excerpt(text))
            .unwrap_or_default();
        let actual_next_line = lines
            .map(|lines| next_non_blank_excerpt(lines.as_slice(), citation.start as usize))
            .unwrap_or_default();
        let actual_heading = lines
            .map(|lines| {
                nearest_preceding_heading_excerpt(
                    lines.as_slice(),
                    (citation.start as usize).saturating_sub(1),
                )
            })
            .unwrap_or_default();
        match snapshot.get(&key) {
            None => {
                result.missing.push(format!(
                    "{}:{line}: cites {key}, which has no snapshot entry",
                    relative(path)
                ));
            }
            Some(expected)
                if expected.excerpt != actual_excerpt
                    || expected.next_line != actual_next_line
                    || expected.heading != actual_heading =>
            {
                let expected_excerpt = &expected.excerpt;
                let expected_next_line = &expected.next_line;
                let expected_heading = &expected.heading;
                result.mismatches.push(format!(
                    "{}:{line}: cites {key}, expected first line {expected_excerpt:?} (next line \
                     {expected_next_line:?}, heading {expected_heading:?}), found \
                     {actual_excerpt:?} (next line {actual_next_line:?}, heading \
                     {actual_heading:?})",
                    relative(path)
                ));
            }
            Some(_) => {}
        }
    }
    for key in snapshot.keys() {
        if !used_keys.contains(key) {
            result.stale.push(format!(
                "{key}: stale snapshot entry — no citation uses this range"
            ));
        }
    }
    result
}

/// Builds a synthetic [`Citation`] for the tests below — shorthand so each
/// fixture reads as data, not boilerplate.
fn fixture_citation(
    acronym: &'static str,
    book_file: &'static str,
    start: i64,
    end: i64,
) -> Citation {
    Citation {
        acronym,
        book_file,
        start,
        end,
    }
}

/// Builds a synthetic [`ExcerptSnapshotEntry`] for the tests below.
fn fixture_excerpt_entry(excerpt: &str, next_line: &str, heading: &str) -> ExcerptSnapshotEntry {
    ExcerptSnapshotEntry {
        excerpt: excerpt.to_string(),
        next_line: next_line.to_string(),
        heading: heading.to_string(),
    }
}

#[test]
fn normalize_excerpt_trims_whitespace_and_caps_long_lines_at_a_char_boundary() {
    assert_eq!(
        normalize_excerpt("   Give quarter to a defeated foe.   "),
        "Give quarter to a defeated foe."
    );

    // A line far longer than EXCERPT_MAX_CHARS, with a multi-byte character
    // (ß, 2 bytes in UTF-8) sitting right at the cut point — proves the cap
    // never splits a codepoint even when the naive byte offset would.
    let long_line = format!(
        "{}ß{}",
        "A".repeat(EXCERPT_MAX_CHARS - 1),
        "more text that must be dropped entirely by the cap"
    );
    let normalized = normalize_excerpt(&long_line);
    assert!(
        normalized.len() <= EXCERPT_MAX_CHARS,
        "expected at most {EXCERPT_MAX_CHARS} bytes, got {} ({normalized:?})",
        normalized.len()
    );
    assert!(
        long_line.starts_with(&normalized),
        "expected the normalized excerpt to be an exact prefix of the original line, got \
         {normalized:?}"
    );
}

#[test]
fn citation_key_uses_a_single_number_for_a_single_line_and_a_range_for_a_span() {
    let single = fixture_citation(
        "ArMDE",
        "Ars Magica - Definitive Edition (Core Rules).md",
        2774,
        2774,
    );
    assert_eq!(citation_key(&single), "ArMDE:2774");

    let range = fixture_citation(
        "ArMDE",
        "Ars Magica - Definitive Edition (Core Rules).md",
        4822,
        4826,
    );
    assert_eq!(citation_key(&range), "ArMDE:4822-4826");
}

#[test]
fn check_citation_excerpts_passes_when_every_citation_matches_its_snapshot() {
    let book_lines: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::from([(
        "Tiny Rulebook.md",
        vec!["Heading", "Give quarter to a defeated foe.", "Next line"],
    )]);
    let citations = vec![(
        PathBuf::from("crates/arm-rules/src/fake.rs"),
        10,
        fixture_citation("Fake", "Tiny Rulebook.md", 2, 2),
    )];
    let snapshot = BTreeMap::from([(
        "Fake:2".to_string(),
        fixture_excerpt_entry("Give quarter to a defeated foe.", "Next line", ""),
    )]);

    let result = check_citation_excerpts(&citations, &book_lines, &snapshot);
    assert_eq!(
        result,
        ExcerptCheckResult::default(),
        "expected no mismatch, missing, or stale entries when everything matches"
    );
}

#[test]
fn check_citation_excerpts_flags_a_citation_whose_cited_line_no_longer_matches_its_snapshot() {
    let book_lines: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::from([(
        "Tiny Rulebook.md",
        vec!["Heading", "Give quarter to a defeated foe.", "Next line"],
    )]);
    let citations = vec![(
        PathBuf::from("crates/arm-rules/src/fake.rs"),
        10,
        fixture_citation("Fake", "Tiny Rulebook.md", 2, 2),
    )];
    // The snapshot still records the OLD wording — simulating the rulebook
    // text having moved under this citation (e.g. a re-sync, or an upstream
    // fix landing on the same line). The next-line context still matches, so
    // this test isolates an excerpt-text mismatch alone.
    let snapshot = BTreeMap::from([(
        "Fake:2".to_string(),
        fixture_excerpt_entry("Give guarter to a defeated foe.", "Next line", ""),
    )]);

    let result = check_citation_excerpts(&citations, &book_lines, &snapshot);
    assert_eq!(
        result.missing,
        Vec::<String>::new(),
        "a mismatch is not a missing entry"
    );
    assert_eq!(
        result.stale,
        Vec::<String>::new(),
        "a mismatch is not a stale entry"
    );
    assert_eq!(
        result.mismatches.len(),
        1,
        "expected exactly one mismatch, found {:?}",
        result.mismatches
    );
    let message = &result.mismatches[0];
    assert!(
        message.contains("fake.rs:10"),
        "expected the citing file:line in the message: {message}"
    );
    assert!(
        message.contains("Fake:2"),
        "expected the cited range's key in the message: {message}"
    );
    assert!(
        message.contains("Give guarter"),
        "expected the snapshot's expected text in the message: {message}"
    );
    assert!(
        message.contains("Give quarter"),
        "expected the book's actual text in the message: {message}"
    );
}

#[test]
fn check_citation_excerpts_flags_a_mismatched_next_line_even_when_the_excerpt_itself_still_matches()
{
    let book_lines: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::from([(
        "Tiny Rulebook.md",
        vec!["Heading", "Give quarter to a defeated foe.", "Next line"],
    )]);
    let citations = vec![(
        PathBuf::from("crates/arm-rules/src/fake.rs"),
        10,
        fixture_citation("Fake", "Tiny Rulebook.md", 2, 2),
    )];
    // The excerpt itself still matches, but the snapshot's captured
    // disambiguation context (the line that used to follow it) is stale —
    // simulating a line inserted, removed, or reworded immediately after the
    // cited line. `citation_relocation.rs`'s relocation tool depends on this
    // context staying accurate, so a stale `next_line` must be caught even
    // though the citation "passes" by excerpt text alone.
    let snapshot = BTreeMap::from([(
        "Fake:2".to_string(),
        fixture_excerpt_entry(
            "Give quarter to a defeated foe.",
            "A line that no longer follows it.",
            "",
        ),
    )]);

    let result = check_citation_excerpts(&citations, &book_lines, &snapshot);
    assert_eq!(
        result.missing,
        Vec::<String>::new(),
        "a next-line mismatch is not a missing entry"
    );
    assert_eq!(
        result.stale,
        Vec::<String>::new(),
        "a next-line mismatch is not a stale entry"
    );
    assert_eq!(
        result.mismatches.len(),
        1,
        "expected exactly one mismatch for the stale next-line context, found {:?}",
        result.mismatches
    );
    let message = &result.mismatches[0];
    assert!(
        message.contains("Fake:2"),
        "expected the cited range's key in the message: {message}"
    );
    assert!(
        message.contains("A line that no longer follows it."),
        "expected the snapshot's stale next-line text in the message: {message}"
    );
    assert!(
        message.contains("Next line"),
        "expected the book's actual next-line text in the message: {message}"
    );
}

#[test]
fn check_citation_excerpts_flags_a_mismatched_heading_even_when_excerpt_and_next_line_match() {
    let book_lines: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::from([(
        "Tiny Rulebook.md",
        vec![
            "## Entry One",
            "Give quarter to a defeated foe.",
            "Next line",
        ],
    )]);
    let citations = vec![(
        PathBuf::from("crates/arm-rules/src/fake.rs"),
        10,
        fixture_citation("Fake", "Tiny Rulebook.md", 2, 2),
    )];
    // Excerpt and next_line both still match; only the captured heading
    // context is stale — simulating the section heading above this passage
    // having been renamed or moved even though the passage's own text and its
    // immediate follower are untouched. `citation_relocation.rs`'s heading
    // tiebreaker depends on this staying accurate, so a stale `heading` must
    // be caught even though the citation "passes" by excerpt and next_line
    // alone.
    let snapshot = BTreeMap::from([(
        "Fake:2".to_string(),
        fixture_excerpt_entry(
            "Give quarter to a defeated foe.",
            "Next line",
            "## A Different Entry",
        ),
    )]);

    let result = check_citation_excerpts(&citations, &book_lines, &snapshot);
    assert_eq!(
        result.missing,
        Vec::<String>::new(),
        "a heading mismatch is not a missing entry"
    );
    assert_eq!(
        result.stale,
        Vec::<String>::new(),
        "a heading mismatch is not a stale entry"
    );
    assert_eq!(
        result.mismatches.len(),
        1,
        "expected exactly one mismatch for the stale heading context, found {:?}",
        result.mismatches
    );
    let message = &result.mismatches[0];
    assert!(
        message.contains("Fake:2"),
        "expected the cited range's key in the message: {message}"
    );
    assert!(
        message.contains("## A Different Entry"),
        "expected the snapshot's stale heading text in the message: {message}"
    );
    assert!(
        message.contains("## Entry One"),
        "expected the book's actual heading text in the message: {message}"
    );
}

#[test]
fn check_citation_excerpts_flags_a_citation_with_no_snapshot_entry() {
    let book_lines: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::from([(
        "Tiny Rulebook.md",
        vec!["Heading", "Give quarter to a defeated foe.", "Next line"],
    )]);
    let citations = vec![(
        PathBuf::from("crates/arm-rules/src/fake.rs"),
        10,
        fixture_citation("Fake", "Tiny Rulebook.md", 2, 2),
    )];
    let snapshot: BTreeMap<String, ExcerptSnapshotEntry> = BTreeMap::new();

    let result = check_citation_excerpts(&citations, &book_lines, &snapshot);
    assert_eq!(
        result.mismatches,
        Vec::<String>::new(),
        "a missing entry is not a mismatch"
    );
    assert_eq!(
        result.stale,
        Vec::<String>::new(),
        "a missing entry is not a stale entry"
    );
    assert_eq!(
        result.missing.len(),
        1,
        "expected exactly one missing entry, found {:?}",
        result.missing
    );
    let message = &result.missing[0];
    assert!(
        message.contains("fake.rs:10"),
        "expected the citing file:line in the message: {message}"
    );
    assert!(
        message.contains("Fake:2"),
        "expected the cited range's key in the message: {message}"
    );
}

#[test]
fn check_citation_excerpts_flags_a_snapshot_entry_no_citation_uses() {
    let book_lines: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::from([(
        "Tiny Rulebook.md",
        vec!["Heading", "Give quarter to a defeated foe.", "Next line"],
    )]);
    let citations = vec![(
        PathBuf::from("crates/arm-rules/src/fake.rs"),
        10,
        fixture_citation("Fake", "Tiny Rulebook.md", 2, 2),
    )];
    // "Fake:3" matches no citation above — it must be reported stale rather
    // than silently ignored, which is what lets the committed snapshot file
    // be trusted not to rot.
    let snapshot = BTreeMap::from([
        (
            "Fake:2".to_string(),
            fixture_excerpt_entry("Give quarter to a defeated foe.", "Next line", ""),
        ),
        (
            "Fake:3".to_string(),
            fixture_excerpt_entry("Next line", "", ""),
        ),
    ]);

    let result = check_citation_excerpts(&citations, &book_lines, &snapshot);
    assert_eq!(
        result.mismatches,
        Vec::<String>::new(),
        "an unused snapshot entry is not a mismatch"
    );
    assert_eq!(
        result.missing,
        Vec::<String>::new(),
        "an unused snapshot entry is not a missing entry"
    );
    assert_eq!(
        result.stale.len(),
        1,
        "expected exactly one stale entry, found {:?}",
        result.stale
    );
    assert!(
        result.stale[0].contains("Fake:3"),
        "expected the stale key in the message: {}",
        result.stale[0]
    );
}

/// `crates/arm-rules/tests/fixtures/citation_excerpts.json`'s path, resolved
/// at runtime from [`repo_root`] (itself `CARGO_MANIFEST_DIR`-based) — never a
/// hardcoded absolute path, so the guard works from any checkout.
fn citation_excerpts_fixture_path() -> PathBuf {
    repo_root().join("crates/arm-rules/tests/fixtures/citation_excerpts.json")
}

/// Reads and parses the committed excerpt snapshot.
fn load_citation_excerpts_snapshot() -> BTreeMap<String, ExcerptSnapshotEntry> {
    let path = citation_excerpts_fixture_path();
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", relative(&path)));
    serde_json::from_str(&content).unwrap_or_else(|e| {
        panic!(
            "{} parses as the excerpt-snapshot map: {e}",
            relative(&path)
        )
    })
}

/// **Regenerates `crates/arm-rules/tests/fixtures/citation_excerpts.json` from
/// the live corpus — not a test the gate runs.** Walks every citation
/// [`collect_all_citations`] finds, dedupes by [`citation_key`] (the excerpt
/// depends only on the rulebook, not on which file cites it), and records
/// each distinct range's [`normalize_excerpt`]d first line, plus the
/// following line's excerpt as `next_line` (P8-1/T2's disambiguation
/// context — see this section's header comment), from [`BookLinesCache`].
/// Written with `serde_json::to_string_pretty` over a `BTreeMap`, whose
/// iteration order is already sorted by key — satisfying `CLAUDE.md`'s
/// "Canonical serialization" (sorted keys) without a separate sort step.
///
/// `#[ignore]`d like its siblings in `rules_source_provenance.rs`
/// (`regenerate_source_anchors`, `regenerate_blank_range_ends`) because it
/// writes into the repository: `cargo test -p arm-rules --test
/// rulebook_citations -- --ignored regenerate_citation_excerpts`. Its output
/// is then reviewed as a diff like any other change, and
/// `every_citation_excerpt_matches_its_snapshot` is what proves the result
/// stays correct afterwards.
#[test]
#[ignore = "writes crates/arm-rules/tests/fixtures/citation_excerpts.json; run deliberately"]
fn regenerate_citation_excerpts() {
    let (_, citations) = collect_all_citations();
    let cache = BookLinesCache::load();

    let mut snapshot: BTreeMap<String, ExcerptSnapshotEntry> = BTreeMap::new();
    for (_, _, citation) in &citations {
        let key = citation_key(citation);
        let lines = cache.lines(citation.book_file);
        let line_index = (citation.start as usize).saturating_sub(1);
        let excerpt = lines
            .get(line_index)
            .map(|text| normalize_excerpt(text))
            .unwrap_or_default();
        let next_line = next_non_blank_excerpt(lines, citation.start as usize);
        let heading = nearest_preceding_heading_excerpt(lines, line_index);
        snapshot.insert(
            key,
            ExcerptSnapshotEntry {
                excerpt,
                next_line,
                heading,
            },
        );
    }

    let json =
        serde_json::to_string_pretty(&snapshot).expect("the excerpt-snapshot map serializes");
    fs::write(citation_excerpts_fixture_path(), format!("{json}\n"))
        .expect("citation_excerpts.json is writable");
}

/// **Diagnostic, not part of the gate — prints and asserts nothing.** For
/// every distinct cited passage ([`citation_key`]-deduped), counts how many
/// lines in its book share its normalized excerpt, its captured `next_line`,
/// **and** its captured `heading` — i.e. would still be ambiguous for
/// `citation_relocation.rs`'s relocation tool even with every disambiguation
/// signal P8-1/T2 captures. A count greater than one (the cited line itself
/// always counts once) means no context this slice captures can resolve that
/// passage on its own; P8-4's human-review pass needs to know this risk
/// exists going in. Run deliberately: `cargo test -p arm-rules --test
/// rulebook_citations -- --ignored --nocapture
/// report_citation_excerpt_duplicates_even_with_full_context`.
#[test]
#[ignore = "diagnostic only; prints to stdout, run deliberately"]
fn report_citation_excerpt_duplicates_even_with_full_context() {
    let (_, citations) = collect_all_citations();
    let cache = BookLinesCache::load();

    // Dedupe by citation_key first — the risk is per distinct cited passage,
    // not per citing occurrence (mirrors `regenerate_citation_excerpts`).
    let mut by_key: BTreeMap<String, Citation> = BTreeMap::new();
    for (_, _, citation) in &citations {
        by_key
            .entry(citation_key(citation))
            .or_insert_with(|| citation.clone());
    }

    let mut duplicate_count = 0usize;
    let mut examples: Vec<String> = Vec::new();
    for (key, citation) in &by_key {
        let lines = cache.lines(citation.book_file);
        let line_index = (citation.start as usize).saturating_sub(1);
        let Some(cited_text) = lines.get(line_index) else {
            continue;
        };
        let excerpt = normalize_excerpt(cited_text);
        let next_line = next_non_blank_excerpt(lines, citation.start as usize);
        let heading = nearest_preceding_heading_excerpt(lines, line_index);

        let occurrences = lines
            .iter()
            .enumerate()
            .filter(|(idx, text)| {
                normalize_excerpt(text) == excerpt
                    && next_non_blank_excerpt(lines, idx + 1) == next_line
                    && nearest_preceding_heading_excerpt(lines, *idx) == heading
            })
            .count();

        if occurrences > 1 {
            duplicate_count += 1;
            if examples.len() < 10 {
                examples.push(format!(
                    "{key}: {excerpt:?} (next: {next_line:?}, heading: {heading:?}) recurs \
                     {occurrences} times in {}",
                    citation.book_file
                ));
            }
        }
    }

    println!(
        "{duplicate_count} of {} distinct cited passages remain ambiguous even with \
         excerpt+next_line+heading context",
        by_key.len()
    );
    for example in &examples {
        println!("{example}");
    }
}

/// **The live guard**: every citation [`collect_all_citations`] finds across
/// the whole corpus must resolve to a snapshot entry, and that entry must
/// still match the cited range's actual first line — the check this file's
/// module doc comment (P8-1/T3) describes. A future rulebook re-sync that
/// silently shifts every line number would otherwise leave every other test
/// in this file green while every citation points at the wrong text; this is
/// the one that catches it. The snapshot must also carry no entry that
/// nothing cites, so it cannot quietly accumulate dead rows across edits.
#[test]
fn every_citation_excerpt_matches_its_snapshot() {
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

    let cache = BookLinesCache::load();
    let book_lines = cache.as_str_map();
    let snapshot = load_citation_excerpts_snapshot();
    assert!(
        !snapshot.is_empty(),
        "expected a non-empty citation-excerpt snapshot"
    );

    let result = check_citation_excerpts(&citations, &book_lines, &snapshot);
    let mut errors = result.mismatches;
    errors.extend(result.missing);
    errors.extend(result.stale);
    assert!(
        errors.is_empty(),
        "{} citation-excerpt snapshot discrepancy(ies):\n{}",
        errors.len(),
        errors.join("\n")
    );
}

// ===========================================================================
// P8-4: bare-citation relocation sweep (`docs/vf-audit/design-p8-resync.md`
// § "What moves" (c)/(d)/(g); D85 Q5/Q6; `citation_relocation.rs`'s own
// algorithm, adapted here for the live corpus).
//
// This reuses `citation_relocation.rs`'s exact/fuzzy/context algorithm —
// deliberately NOT by importing it (each integration test is its own
// compilation unit, see that file's and `source_relocation.rs`'s own doc
// comments on why duplication beats a shared module here) — with one
// adaptation: disambiguation context (`next_line`/`heading`) is compared
// through `normalize_excerpt` on both sides, because the context here is read
// fresh from the OLD file directly (full-fidelity, never truncated), while
// the *candidate* line in the NEW file is compared the same way
// `citation_excerpts.json`'s own snapshot entries were built — truncated at
// `EXCERPT_MAX_CHARS`. Comparing a truncated 80-char prefix against an
// untruncated real line with plain equality would spuriously fail for any
// context line longer than 80 characters; `normalize_excerpt` on both sides
// fixes that the same way the snapshot already handles it.
// ===========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SweepConfidence {
    Exact,
    Fuzzy,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SweepLineRelocation {
    new_line: Option<usize>,
    confidence: SweepConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SweepRangeRelocation {
    new_start: Option<usize>,
    new_end: Option<usize>,
    confidence: SweepConfidence,
}

/// True when `a` and `b` are different but sit at most one character-edit
/// apart — duplicated verbatim from `citation_relocation.rs::within_one_edit`
/// (pure, already proven there; see this section's header comment on why
/// duplication rather than a shared import).
fn sweep_within_one_edit(a: &str, b: &str) -> bool {
    if a == b {
        return false;
    }
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (short, long) = if a.len() <= b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    if long.len() - short.len() > 1 {
        return false;
    }
    if long.len() == short.len() {
        return short
            .iter()
            .zip(long.iter())
            .filter(|(x, y)| x != y)
            .count()
            == 1;
    }
    let mut i = 0;
    let mut j = 0;
    let mut skipped = false;
    while i < short.len() && j < long.len() {
        if short[i] == long[j] {
            i += 1;
            j += 1;
        } else if !skipped {
            skipped = true;
            j += 1;
        } else {
            return false;
        }
    }
    true
}

/// [`resolve_candidates`]'s sweep analogue: narrows same-ranked candidate
/// line indices (0-based into `new_lines`) to one, using `next_line` context
/// first and `heading` context second — identical staging to
/// `citation_relocation.rs::resolve_candidates`, but every comparison runs
/// both sides through [`normalize_excerpt`] (see this section's header
/// comment for why).
fn sweep_resolve_candidates(
    candidates: &[usize],
    new_lines: &[String],
    next_line_ctx: &str,
    heading_ctx: &str,
) -> Option<usize> {
    let mut current: Vec<usize> = candidates.to_vec();
    if current.len() <= 1 {
        return current.first().copied();
    }

    let narrowed: Vec<usize> = current
        .iter()
        .copied()
        .filter(|&idx| {
            new_lines
                .get(idx + 1)
                .map(|l| normalize_excerpt(l))
                .unwrap_or_default()
                == next_line_ctx
        })
        .collect();
    match narrowed.as_slice() {
        [only] => return Some(*only),
        [] => {}
        _ => current = narrowed,
    }
    if current.len() <= 1 {
        return current.first().copied();
    }

    let narrowed: Vec<usize> = current
        .iter()
        .copied()
        .filter(|&idx| nearest_preceding_heading_excerpt(new_lines, idx) == heading_ctx)
        .collect();
    match narrowed.as_slice() {
        [only] => Some(*only),
        _ => None,
    }
}

/// Relocates one cited line (1-based) from `old_lines` into `new_lines`,
/// reading its own disambiguation context fresh from `old_lines` (full-
/// fidelity, never the committed snapshot — see this section's header
/// comment). Mirrors `citation_relocation.rs::relocate_line` exactly in
/// shape: exact-match candidates first, [`sweep_within_one_edit`] fuzzy
/// candidates only when no exact hit exists.
fn sweep_relocate_line(
    old_lines: &[String],
    new_lines: &[String],
    old_line: usize,
) -> SweepLineRelocation {
    let target = old_lines
        .get(old_line - 1)
        .map(|l| l.trim())
        .unwrap_or_default();
    let next_line_ctx = next_non_blank_excerpt(old_lines, old_line);
    let heading_ctx = nearest_preceding_heading_excerpt(old_lines, old_line - 1);

    let exact: Vec<usize> = new_lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim() == target)
        .map(|(i, _)| i)
        .collect();
    if !exact.is_empty() {
        return match sweep_resolve_candidates(&exact, new_lines, &next_line_ctx, &heading_ctx) {
            Some(idx) => SweepLineRelocation {
                new_line: Some(idx + 1),
                confidence: SweepConfidence::Exact,
            },
            None => SweepLineRelocation {
                new_line: None,
                confidence: SweepConfidence::Unresolved,
            },
        };
    }

    let fuzzy: Vec<usize> = new_lines
        .iter()
        .enumerate()
        .filter(|(_, l)| sweep_within_one_edit(l.trim(), target))
        .map(|(i, _)| i)
        .collect();
    match sweep_resolve_candidates(&fuzzy, new_lines, &next_line_ctx, &heading_ctx) {
        Some(idx) => SweepLineRelocation {
            new_line: Some(idx + 1),
            confidence: SweepConfidence::Fuzzy,
        },
        None => SweepLineRelocation {
            new_line: None,
            confidence: SweepConfidence::Unresolved,
        },
    }
}

/// Relocates one cited range (1-based inclusive) — mirrors
/// `citation_relocation.rs::relocate_range`: relocate the start, then (for an
/// actual multi-line range) verify the end's text still sits the same span
/// away; a broken span is reported, never guessed.
fn sweep_relocate_range(
    old_lines: &[String],
    new_lines: &[String],
    old_start: usize,
    old_end: usize,
) -> SweepRangeRelocation {
    let start = sweep_relocate_line(old_lines, new_lines, old_start);
    let Some(new_start) = start.new_line else {
        return SweepRangeRelocation {
            new_start: None,
            new_end: None,
            confidence: SweepConfidence::Unresolved,
        };
    };
    if old_start == old_end {
        return SweepRangeRelocation {
            new_start: Some(new_start),
            new_end: Some(new_start),
            confidence: start.confidence,
        };
    }

    let span = old_end - old_start;
    let candidate_new_end = new_start + span;
    let old_end_text = old_lines
        .get(old_end - 1)
        .map(|l| l.trim())
        .unwrap_or_default();
    let new_end_text = new_lines
        .get(candidate_new_end - 1)
        .map(|l| l.trim())
        .unwrap_or_default();
    let end_matches =
        old_end_text == new_end_text || sweep_within_one_edit(old_end_text, new_end_text);

    if end_matches {
        SweepRangeRelocation {
            new_start: Some(new_start),
            new_end: Some(candidate_new_end),
            confidence: start.confidence,
        }
    } else {
        SweepRangeRelocation {
            new_start: Some(new_start),
            new_end: None,
            confidence: SweepConfidence::Unresolved,
        }
    }
}

#[test]
fn sweep_relocate_line_finds_an_unchanged_line_after_an_offset_shift() {
    let old: Vec<String> = ["Heading A", "Give quarter to a defeated foe.", "Next line"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let new: Vec<String> = [
        "An inserted preamble line.",
        "Heading A",
        "Give quarter to a defeated foe.",
        "Next line",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(
        sweep_relocate_line(&old, &new, 2),
        SweepLineRelocation {
            new_line: Some(3),
            confidence: SweepConfidence::Exact,
        }
    );
}

#[test]
fn sweep_relocate_line_is_unresolved_inside_a_changed_hunk() {
    let old: Vec<String> = ["Heading A", "This passage was entirely rewritten."]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let new: Vec<String> = ["Heading A", "A completely different sentence now."]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        sweep_relocate_line(&old, &new, 2),
        SweepLineRelocation {
            new_line: None,
            confidence: SweepConfidence::Unresolved,
        }
    );
}

#[test]
fn sweep_relocate_line_tolerates_a_single_character_ocr_fix() {
    let old: Vec<String> = ["Give guarter to a defeated foe."]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let new: Vec<String> = ["Preamble.", "Give quarter to a defeated foe."]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        sweep_relocate_line(&old, &new, 1),
        SweepLineRelocation {
            new_line: Some(2),
            confidence: SweepConfidence::Fuzzy,
        }
    );
}

/// The delta from `citation_relocation.rs`'s own test suite this file adds:
/// disambiguation must still work when the real candidate's following line
/// is **longer than `EXCERPT_MAX_CHARS`** — comparing a truncated context
/// against the untruncated real text with plain equality would wrongly fail
/// here; both sides must go through [`normalize_excerpt`].
#[test]
fn sweep_resolve_candidates_matches_a_next_line_longer_than_the_excerpt_cap() {
    let long_line = "A".repeat(120);
    let new_lines: Vec<String> = vec![
        "*Minor, General*<br>".to_string(),
        "Decoy follower, short and irrelevant.".to_string(),
        "*Minor, General*<br>".to_string(),
        long_line.clone(),
    ];
    let next_line_ctx = normalize_excerpt(&long_line);
    let resolved = sweep_resolve_candidates(&[0, 2], &new_lines, &next_line_ctx, "");
    assert_eq!(resolved, Some(2));
}

#[test]
fn sweep_relocate_range_follows_an_unchanged_paragraph() {
    let old: Vec<String> = ["Heading", "Line one.", "Line two."]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let new: Vec<String> = ["Preamble.", "Heading", "Line one.", "Line two."]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        sweep_relocate_range(&old, &new, 2, 3),
        SweepRangeRelocation {
            new_start: Some(3),
            new_end: Some(4),
            confidence: SweepConfidence::Exact,
        }
    );
}

#[test]
fn sweep_relocate_range_reports_rather_than_guesses_when_the_span_breaks() {
    let old: Vec<String> = ["Heading", "Line one.", "Line two."]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let new: Vec<String> = ["Heading", "Line one.", "An inserted line.", "Line two."]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        sweep_relocate_range(&old, &new, 1, 3),
        SweepRangeRelocation {
            new_start: Some(1),
            new_end: None,
            confidence: SweepConfidence::Unresolved,
        }
    );
}

/// Where the sweep reads the PRE-sync books from — `source_relocation.rs`'s
/// own `tmp/p8-old-source` copy (committed by the P8-1/T1 slice), the same
/// convention that file's `old_source_root` documents.
fn old_rules_source_en() -> PathBuf {
    repo_root().join("tmp/p8-old-source/en")
}

/// Reads every book in [`BOOK_ACRONYMS`] from `root`, as plain lines — a
/// parameterized twin of [`BookLinesCache::load`] (which is hardcoded to
/// [`rules_source_en`]) so the same loader serves both the OLD and NEW side.
fn load_book_lines(root: &Path) -> BTreeMap<&'static str, Vec<String>> {
    let mut books = BTreeMap::new();
    for &(_, book_file) in BOOK_ACRONYMS {
        let content = fs::read_to_string(root.join(book_file))
            .unwrap_or_else(|e| panic!("{book_file} is readable under {root:?}: {e}"));
        books.insert(book_file, content.lines().map(str::to_string).collect());
    }
    books
}

/// One row of the P8-4 relocation map: a distinct cited passage
/// ([`citation_key`]-deduped — the relocation depends only on the book, not
/// on which file cites it) and where it now resolves to.
struct RelocationRow {
    key: String,
    book_file: &'static str,
    new: Option<(usize, usize)>,
    confidence: SweepConfidence,
}

fn render_relocation_map(rows: &[RelocationRow]) -> String {
    let exact = rows
        .iter()
        .filter(|r| r.confidence == SweepConfidence::Exact)
        .count();
    let fuzzy: Vec<&RelocationRow> = rows
        .iter()
        .filter(|r| r.confidence == SweepConfidence::Fuzzy)
        .collect();
    let unresolved: Vec<&RelocationRow> = rows
        .iter()
        .filter(|r| r.confidence == SweepConfidence::Unresolved)
        .collect();

    let mut out = format!(
        "# P8-4 citation relocation map\n\n**Outcome:** {} distinct cited passage(s) — {exact} \
         exact, {} fuzzy, {} unresolved.\n",
        rows.len(),
        fuzzy.len(),
        unresolved.len()
    );

    out.push_str(&format!(
        "\n## Unresolved ({}) — never applied, needs a human read\n\n",
        unresolved.len()
    ));
    if unresolved.is_empty() {
        out.push_str("(none)\n");
    } else {
        out.push_str("| old | book |\n|---|---|\n");
        for row in &unresolved {
            out.push_str(&format!("| {} | {} |\n", row.key, row.book_file));
        }
    }

    out.push_str(&format!(
        "\n## Fuzzy ({}) — applied, single-character-edit match; worth a spot check\n\n",
        fuzzy.len()
    ));
    if fuzzy.is_empty() {
        out.push_str("(none)\n");
    } else {
        out.push_str("| old | new | book |\n|---|---|---|\n");
        for row in &fuzzy {
            let new = row
                .new
                .map(|(s, e)| {
                    if s == e {
                        s.to_string()
                    } else {
                        format!("{s}-{e}")
                    }
                })
                .unwrap_or_default();
            out.push_str(&format!("| {} | {new} | {} |\n", row.key, row.book_file));
        }
    }

    out.push_str(&format!("\n## Exact ({exact}) — applied\n\n"));
    out.push_str("| old | new | book |\n|---|---|---|\n");
    for row in rows
        .iter()
        .filter(|r| r.confidence == SweepConfidence::Exact)
    {
        let new = row
            .new
            .map(|(s, e)| {
                if s == e {
                    s.to_string()
                } else {
                    format!("{s}-{e}")
                }
            })
            .unwrap_or_default();
        out.push_str(&format!("| {} | {new} | {} |\n", row.key, row.book_file));
    }

    out
}

/// **Builds the P8-4 relocation map — not a test the gate runs.** For every
/// distinct cited passage [`collect_all_citations`] finds (deduped by
/// [`citation_key`], since the relocation depends only on the book), relocates
/// it from [`old_rules_source_en`] into [`rules_source_en`] via
/// [`sweep_relocate_range`], and writes the result to
/// `tmp/p8-4-relocation-map.md`. Read-only against the repository proper —
/// only `tmp/` is written — so safe to run repeatedly while reviewing.
///
/// `#[ignore]`d like its siblings (`regenerate_source_relocations`,
/// `regenerate_citation_excerpts`): `cargo test -p arm-rules --test
/// rulebook_citations -- --ignored --nocapture
/// build_p8_4_relocation_map`.
#[test]
#[ignore = "writes tmp/p8-4-relocation-map.md; run deliberately — see tmp/p8-34-handover.md"]
fn build_p8_4_relocation_map() {
    let (_, citations) = collect_all_citations();
    let old_books = load_book_lines(&old_rules_source_en());
    let new_books = load_book_lines(&rules_source_en());

    let mut distinct: BTreeMap<String, (&'static str, i64, i64)> = BTreeMap::new();
    for (_, _, citation) in &citations {
        distinct.entry(citation_key(citation)).or_insert((
            citation.book_file,
            citation.start,
            citation.end,
        ));
    }

    let mut rows = Vec::new();
    for (key, (book_file, start, end)) in &distinct {
        let old_lines = old_books
            .get(book_file)
            .unwrap_or_else(|| panic!("{book_file} is not among the cached OLD books"));
        let new_lines = new_books
            .get(book_file)
            .unwrap_or_else(|| panic!("{book_file} is not among the cached NEW books"));
        let relocation = sweep_relocate_range(old_lines, new_lines, *start as usize, *end as usize);
        rows.push(RelocationRow {
            key: key.clone(),
            book_file,
            new: match (relocation.new_start, relocation.new_end) {
                (Some(s), Some(e)) => Some((s, e)),
                _ => None,
            },
            confidence: relocation.confidence,
        });
    }

    let report = render_relocation_map(&rows);
    let out_path = repo_root().join("tmp/p8-4-relocation-map.md");
    fs::write(&out_path, &report).unwrap_or_else(|e| panic!("{out_path:?} is writable: {e}"));

    let exact = rows
        .iter()
        .filter(|r| r.confidence == SweepConfidence::Exact)
        .count();
    let fuzzy = rows
        .iter()
        .filter(|r| r.confidence == SweepConfidence::Fuzzy)
        .count();
    let unresolved = rows.len() - exact - fuzzy;
    println!(
        "distinct={} exact={exact} fuzzy={fuzzy} unresolved={unresolved}",
        rows.len()
    );
}

// ===========================================================================
// P8-4 phase 2: applying the relocation — byte-exact, self-verifying rewrite.
//
// Operates on each file's RAW text directly (not the block-joined text
// `collect_all_citations` uses for *detection*), because joining erases byte
// positions. This is safe for this corpus specifically: every
// `ACRONYM:NNNN`-shaped substring found below is independently re-verified
// against its expected old digits immediately before being touched (see
// [`apply_citation_relocations`]) — a wrong match is therefore never written,
// only silently skipped, so an imperfect position computation can degrade
// coverage but can never corrupt a file.
// ===========================================================================

/// One `ACRONYM:NNNN` or `ACRONYM:NNNN-MMMM` citation occurrence located in a
/// file's raw text, with the exact byte span of each number so it can be
/// replaced independently. A chained `, :NNNN` continuation produces its own
/// occurrence, keyed under the acronym it inherits — see
/// [`find_citation_occurrences`].
#[derive(Debug, Clone, PartialEq, Eq)]
struct CitationOccurrence {
    /// The [`citation_key`]-shaped lookup key: `ACRONYM:START` or
    /// `ACRONYM:START-END`.
    key: String,
    /// Byte range of the start number's digits in the file text.
    start_span: (usize, usize),
    /// Byte range of the end number's digits, only when this occurrence is
    /// an actual `-MMMM` range (never set for a single-line citation, even
    /// though `key`'s start == end in that case).
    end_span: Option<(usize, usize)>,
}

/// `(start, end, start digits' byte span, end digits' byte span if a range,
/// byte offset after the parse)`, as returned by [`parse_number_span`].
type ParsedNumberSpan = (i64, i64, (usize, usize), Option<(usize, usize)>, usize);

/// Parses one `NNNN` or `NNNN-MMMM` number starting at byte `at` in `text`.
/// Returns the parsed (start, end) values, each number's own byte span, and
/// the byte offset immediately after the whole parse — the position-tracking
/// twin of [`parse_citation_number`].
fn parse_number_span(text: &str, at: usize) -> Option<ParsedNumberSpan> {
    let (start, start_len) = parse_digits(&text[at..])?;
    let mut end = start;
    let mut end_span = None;
    let mut after = at + start_len;
    if let Some(rest) = text[after..].strip_prefix('-')
        && let Some((end_num, end_len)) = parse_digits(rest)
    {
        end = end_num;
        let end_at = after + 1;
        end_span = Some((end_at, end_at + end_len));
        after = end_at + end_len;
    }
    Some((start, end, (at, at + start_len), end_span, after))
}

/// Finds every citation occurrence in `content`'s raw text (whole-file, not
/// block-joined — see this section's header comment), chasing `, :NNNN`
/// continuations exactly as [`find_citations`] does. Independent of that
/// function: this one tracks byte spans for rewriting rather than collecting
/// `Citation` values for detection.
fn find_citation_occurrences(content: &str) -> Vec<CitationOccurrence> {
    let mut found = Vec::new();
    for &(acronym, _) in BOOK_ACRONYMS {
        let needle = format!("{acronym}:");
        let mut search_from = 0usize;
        while let Some(rel) = content[search_from..].find(needle.as_str()) {
            let acronym_at = search_from + rel;
            let colon_at = acronym_at + acronym.len();
            if !word_boundary_before(content, acronym_at) {
                search_from = colon_at + 1;
                continue;
            }
            let Some((start, end, start_span, end_span, mut pos)) =
                parse_number_span(content, colon_at + 1)
            else {
                search_from = colon_at + 1;
                continue;
            };
            found.push(CitationOccurrence {
                key: if start == end {
                    format!("{acronym}:{start}")
                } else {
                    format!("{acronym}:{start}-{end}")
                },
                start_span,
                end_span,
            });
            while let Some(_rest) = content[pos..].strip_prefix(", :") {
                let colon_at = pos + 2;
                let Some((cstart, cend, cstart_span, cend_span, next_pos)) =
                    parse_number_span(content, colon_at + 1)
                else {
                    break;
                };
                found.push(CitationOccurrence {
                    key: if cstart == cend {
                        format!("{acronym}:{cstart}")
                    } else {
                        format!("{acronym}:{cstart}-{cend}")
                    },
                    start_span: cstart_span,
                    end_span: cend_span,
                });
                pos = next_pos;
            }
            search_from = pos;
        }
    }
    found
}

/// Applies `relocations` (key -> new `(start, end)`) to every occurrence
/// [`find_citation_occurrences`] finds in `content`, self-verifying each
/// number span against its own expected old digits before touching it — see
/// this section's header comment. A span whose live bytes do not match what
/// the occurrence itself claims (should never happen, but "never guess"
/// applies to the writer's own assumptions too) is skipped, not forced.
/// Returns the rewritten text and how many number spans were actually
/// replaced.
fn apply_citation_relocations(
    content: &str,
    relocations: &BTreeMap<String, (usize, usize)>,
) -> (String, usize) {
    let occurrences = find_citation_occurrences(content);
    let mut edits: Vec<((usize, usize), String)> = Vec::new();

    for occ in &occurrences {
        let Some(&(new_start, new_end)) = relocations.get(&occ.key) else {
            continue;
        };
        let old_key_numbers: Vec<&str> = occ
            .key
            .rsplit(':')
            .next()
            .unwrap_or("")
            .split('-')
            .collect();
        let expected_start = old_key_numbers.first().copied().unwrap_or("");
        let expected_end = old_key_numbers.get(1).copied();

        let (s, e) = occ.start_span;
        if content.get(s..e) == Some(expected_start) {
            edits.push(((s, e), new_start.to_string()));
        }
        if let (Some(end_span), Some(expected_end)) = (occ.end_span, expected_end) {
            let (s, e) = end_span;
            if content.get(s..e) == Some(expected_end) {
                edits.push(((s, e), new_end.to_string()));
            }
        }
    }

    edits.sort_by_key(|a| std::cmp::Reverse(a.0.0));
    let mut rewritten = content.to_string();
    let mut applied = 0usize;
    for ((s, e), replacement) in edits {
        rewritten.replace_range(s..e, &replacement);
        applied += 1;
    }
    (rewritten, applied)
}

#[test]
fn find_citation_occurrences_tracks_the_byte_span_of_each_number() {
    let content = "// See ArMDE:100 and ArMDE:200-205 for details.";
    let found = find_citation_occurrences(content);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].key, "ArMDE:100");
    assert_eq!(
        &content[found[0].start_span.0..found[0].start_span.1],
        "100"
    );
    assert_eq!(found[0].end_span, None);
    assert_eq!(found[1].key, "ArMDE:200-205");
    assert_eq!(
        &content[found[1].start_span.0..found[1].start_span.1],
        "200"
    );
    let end_span = found[1].end_span.expect("a range citation has an end span");
    assert_eq!(&content[end_span.0..end_span.1], "205");
}

#[test]
fn find_citation_occurrences_chases_a_continuation_inheriting_the_acronym() {
    let content = "// ArMDE:100, :150 continues without repeating the acronym.";
    let found = find_citation_occurrences(content);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].key, "ArMDE:100");
    assert_eq!(found[1].key, "ArMDE:150");
}

#[test]
fn apply_citation_relocations_rewrites_exactly_the_mapped_numbers() {
    let content = "// ArMDE:100 and ArMDE:200-205, plus ArMDE:999 (unmapped).";
    let mut relocations = BTreeMap::new();
    relocations.insert("ArMDE:100".to_string(), (171, 171));
    relocations.insert("ArMDE:200-205".to_string(), (271, 276));
    let (rewritten, applied) = apply_citation_relocations(content, &relocations);
    assert_eq!(applied, 3);
    assert_eq!(
        rewritten,
        "// ArMDE:171 and ArMDE:271-276, plus ArMDE:999 (unmapped)."
    );
}

/// A relocation map entry whose key is not actually cited in `content` must
/// never fire — the lookup is keyed by the occurrence actually *found*, not
/// walked the other way round from the map. (The per-span text check inside
/// [`apply_citation_relocations`] is a tautology by construction — the
/// expected digits come from the very same parse that produced the span, so
/// it can never disagree with itself; it costs nothing to keep, but the real
/// safety boundary for this tool is scanning *raw* file text rather than a
/// block-joined copy, so no position-translation step exists to get wrong in
/// the first place. See this section's header comment.)
#[test]
fn apply_citation_relocations_never_fires_a_map_entry_the_content_does_not_cite() {
    let content = "// ArMDE:100 here.";
    let mut relocations = BTreeMap::new();
    relocations.insert("ArMDE:999".to_string(), (1, 1));
    let (rewritten, applied) = apply_citation_relocations(content, &relocations);
    assert_eq!(applied, 0);
    assert_eq!(rewritten, content);
}

/// Hand-verified overrides for citations [`sweep_relocate_range`] could not
/// resolve mechanically — read directly against both files during this P8-4
/// slice, same discipline as `source_relocation.rs`'s `ANCHOR_OVERRIDES`
/// and `MANUAL_BOUNDARY_RECOMPUTE_OVERRIDES`. Every entry's reasoning:
///
/// - `ArMDE:12385-15941` -> `12457-16018`: `## Animal Spells` (start) to the
///   blank line before `# Chapter 10: Long Term Events` (end) — a whole-
///   chapter descriptive range in RULES.md, both ends confirmed by heading
///   search.
/// - `ArMDE:14016` -> `14090`: the stat line of "Thaumaturgical
///   Transformation of Plants to Iron"; upstream dropped the book's own
///   erroneous inline `Reg: Terram` (a typo for `Req:`, already a documented
///   special case in `spell_stat_line.rs`) — the shipped `requisites` data
///   was already correct, only the comment text describing the book's old
///   typo needs to follow it.
/// - `ArMDE:4740-4742`/`:4742`/`:4742-4748` -> `4811-4813`/`:4813`/
///   `:4813-4819`: Potent Magic's own entry; upstream fixed a genuine OCR
///   duplication ("unlike a Magical Focus" tripled) confirmed by reading
///   both files — more than a single-character edit, so never guessed by
///   the fuzzy threshold, confirmed instead by heading match.
/// - `ArMDE:5909-5911`/`:5911` -> `5980-5982`/`:5982`: Deficient Form;
///   upstream fixed "actual value of the Technique" -> "...of the Form" (a
///   real rules-text bug fix, confirmed by heading match).
/// - `ArMDE:7177-7268` -> `7248-7340`: "General Abilities" heading through
///   the blank line before `## Ability List`, confirmed by heading search
///   both ends.
/// - `ArMDE:364` -> `432`: "the middle ages" -> "the Middle Ages"
///   (capitalization fix, confirmed unique).
/// - `ArMDE:4793` -> `4864`: Prestigious Student's magnitude line; upstream
///   fixed the book's own "*General, Minor*" ordering to the house style
///   "*Minor, General*" (confirmed unique, by heading match).
/// - `ArMDE:5775` -> `5846`: Reckless's own passage; upstream fixed "this
///   Virtue is taken" -> "this Flaw is taken" (confirmed unique).
/// - `ArMDE:5417` -> `5486`: the Flaws TOC's own `### Hermetic, Minor`
///   sub-heading (two occurrences exist — Virtues TOC and Flaws TOC;
///   confirmed by matching neighbors `[Repellent]`/`[Bound Casting Tools]`).
/// - `ArMDE:5518` -> `5568`: the Flaws TOC's `[Weak Personality]` link;
///   upstream added a trailing `<br>` other entries in that list already
///   carry (confirmed unique).
/// - `ArMDE:16594` -> `16671`: the Living Conditions table's asterisk
///   footnote (two occurrences of the same generic sentence exist in the
///   book; `16671` is the one inside this table's own neighborhood,
///   confirmed by the surrounding lines already relocated during P8-3).
/// - `ArMDE:16988`/`:16992`/`:17017` -> `17065`/`:17069`/`:17094`: the three
///   weapon-statistics-legend duplicates `citation_excerpts.json`'s own
///   duplicate report already named as unresolved even with full context —
///   each book has the SAME legend twice (a main Combat-chapter table and a
///   back-of-book quick-reference repeat, confirmed present in both the OLD
///   and NEW files, so not a resync artifact); resolved to the FIRST
///   (main-chapter) occurrence in each case, matching ordinary citation
///   practice. A genuine pre-existing bug surfaced along the way: RULES.md
///   cited `ArMDE:16988` (the asterisk footnote) while describing the
///   "Ability" column, which is actually `ArMDE:16992`'s text — fixed
///   directly in `RULES.md` (unrelated to the resync, caught while
///   confirming this override).
///
/// Not overridden, left genuinely unresolved — `ArMDE:3006-3358`,
/// `:3087`, `:3135-3185`, `:3187-3237`, `:5373`, `:5501-5518`: all sit inside
/// the Virtues/Flaws table-of-contents region, whose own sub-headings
/// (`### Hermetic, Minor` etc.) and link text recur or were reformatted
/// (confirmed: `### Hermetic, Minor` and `### Story, Minor` each have
/// multiple book-wide occurrences, and `[True Love (NPC)]`'s own link text
/// no longer exists verbatim) in a way that span-preservation cannot safely
/// resolve without a dedicated TOC-structure comparison — flagged for a
/// follow-up pass, not guessed here.
const RELOCATION_OVERRIDES: &[(&str, (usize, usize))] = &[
    ("ArMDE:12385-15941", (12457, 16018)),
    ("ArMDE:14016", (14090, 14090)),
    ("ArMDE:4740-4742", (4811, 4813)),
    ("ArMDE:4742", (4813, 4813)),
    ("ArMDE:4742-4748", (4813, 4819)),
    ("ArMDE:5909-5911", (5980, 5982)),
    ("ArMDE:5911", (5982, 5982)),
    ("ArMDE:7177-7268", (7248, 7340)),
    ("ArMDE:364", (432, 432)),
    ("ArMDE:4793", (4864, 4864)),
    ("ArMDE:5775", (5846, 5846)),
    ("ArMDE:5417", (5486, 5486)),
    ("ArMDE:5518", (5568, 5568)),
    ("ArMDE:16594", (16671, 16671)),
    ("ArMDE:16988", (17065, 17065)),
    ("ArMDE:16992", (17069, 17069)),
    ("ArMDE:17017", (17094, 17094)),
];

#[test]
fn relocation_overrides_keys_are_well_formed_citation_keys() {
    for (key, _) in RELOCATION_OVERRIDES {
        assert!(
            key.starts_with("ArMDE:"),
            "{key} must be an ArMDE citation key"
        );
        let numbers = key.rsplit(':').next().unwrap();
        assert!(
            numbers.chars().all(|c| c.is_ascii_digit() || c == '-'),
            "{key}'s number part must be digits and at most one dash"
        );
    }
}

/// Every root this P8-4 rewrite pass touches — exactly
/// [`collect_all_citations`]'s own file sets, **minus [`docs_markdown_files`]**:
/// `docs/`'s citations are dated historical records (D1c; CLAUDE.md's
/// "Cross-reference to another source file" rule draws the same line for
/// source-file citations, and `design-p8-resync.md` § "What moves" (e) draws
/// it explicitly for rulebook citations too) and must keep the line number
/// they were written against, never rewritten to the post-sync position.
fn p8_4_rewrite_files() -> Vec<PathBuf> {
    let mut files = all_rust_files();
    files.push(repo_root().join("crates/arm-rules/RULES.md"));
    files.extend(web_source_files());
    files
}

/// **Applies the P8-4 relocation across the live corpus — not a test the
/// gate runs.** Builds the same relocation map
/// [`build_p8_4_relocation_map`] does (recomputed rather than read back from
/// `tmp/p8-4-relocation-map.md`, since that file is a report, not a
/// machine-readable intermediate), folds in [`RELOCATION_OVERRIDES`], then
/// runs [`apply_citation_relocations`] over every file
/// [`p8_4_rewrite_files`] names, writing back only the files that actually
/// changed. Re-renders the relocation map afterward so it reflects the
/// overrides too.
///
/// `#[ignore]`d like every writer in this suite:
/// `cargo test -p arm-rules --test rulebook_citations -- --ignored
/// --nocapture apply_p8_4_citation_relocations`.
#[test]
#[ignore = "rewrites citations across crates/*/src, crates/*/tests, RULES.md, ui/src, ui/e2e; \
            run deliberately — see tmp/p8-34-handover.md"]
fn apply_p8_4_citation_relocations() {
    let (_, citations) = collect_all_citations();
    let old_books = load_book_lines(&old_rules_source_en());
    let new_books = load_book_lines(&rules_source_en());

    let mut distinct: BTreeMap<String, (&'static str, i64, i64)> = BTreeMap::new();
    for (_, _, citation) in &citations {
        distinct.entry(citation_key(citation)).or_insert((
            citation.book_file,
            citation.start,
            citation.end,
        ));
    }

    let mut rows = Vec::new();
    let mut relocations: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (key, (book_file, start, end)) in &distinct {
        if let Some(&(_, new)) = RELOCATION_OVERRIDES.iter().find(|(k, _)| k == key) {
            relocations.insert(key.clone(), new);
            rows.push(RelocationRow {
                key: key.clone(),
                book_file,
                new: Some(new),
                confidence: SweepConfidence::Exact,
            });
            continue;
        }
        let old_lines = old_books
            .get(book_file)
            .unwrap_or_else(|| panic!("{book_file} is not among the cached OLD books"));
        let new_lines = new_books
            .get(book_file)
            .unwrap_or_else(|| panic!("{book_file} is not among the cached NEW books"));
        let relocation = sweep_relocate_range(old_lines, new_lines, *start as usize, *end as usize);
        if let (Some(s), Some(e)) = (relocation.new_start, relocation.new_end)
            && relocation.confidence != SweepConfidence::Unresolved
        {
            relocations.insert(key.clone(), (s, e));
        }
        rows.push(RelocationRow {
            key: key.clone(),
            book_file,
            new: match (relocation.new_start, relocation.new_end) {
                (Some(s), Some(e)) => Some((s, e)),
                _ => None,
            },
            confidence: relocation.confidence,
        });
    }

    let mut files_changed = 0usize;
    let mut citations_rewritten = 0usize;
    for path in p8_4_rewrite_files() {
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        let (rewritten, applied) = apply_citation_relocations(&content, &relocations);
        if applied > 0 {
            fs::write(&path, &rewritten).unwrap_or_else(|e| panic!("{path:?} is writable: {e}"));
            files_changed += 1;
            citations_rewritten += applied;
        }
    }

    let report = render_relocation_map(&rows);
    let out_path = repo_root().join("tmp/p8-4-relocation-map.md");
    fs::write(&out_path, &report).unwrap_or_else(|e| panic!("{out_path:?} is writable: {e}"));

    println!("files_changed={files_changed} citations_rewritten={citations_rewritten}");
}
