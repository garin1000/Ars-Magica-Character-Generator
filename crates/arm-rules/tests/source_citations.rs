//! E3: a comment must cite another **source file** by symbol, never by line
//! number.
//!
//! A rulebook line number is stable — the book is not edited — which is why
//! `rulebook_citations.rs` can let a comment pin one and merely check it lands
//! inside the file. A *source* line number is the opposite: every edit above it
//! moves it, nothing tells the reader it moved, and the defect only surfaces
//! when someone opens the target and finds unrelated code. `docs/open-todos.md`
//! row 22 recorded the case that started this: `effective/xp.rs` pinned four
//! `Entity` fields in `types.rs` by line, D1a found three of the four already
//! drifted, and by the time this guard was written **all five** in-repo Rust
//! citations in the tree were stale — one by 190 lines and one by 1045.
//!
//! **The form is `` `<file>::<symbol>` ``** — the file (a full path or any
//! suffix of one that resolves) and the symbol, joined by Rust's path
//! separator. It was not invented here: thirteen comments across both languages
//! already used it (`effective/xp.rs::charged_cost`,
//! `commands.rs::effective_scores_surface_virtue_flaw_balance`), and
//! `rules_md_citations.rs`'s own `resolve_target` has read the same shape out of
//! `crates/arm-rules/RULES.md` since B11. This file makes it the rule and checks
//! it: the symbol must still be *declared* in the named file, so the citation
//! fails loudly the day the symbol is renamed or deleted instead of pointing at
//! nothing in particular.
//!
//! **It cannot collide with the rulebook guard's form.** A rulebook citation is
//! `ACRONYM:NNNN`, whose colon is preceded by letters that are not a file
//! extension; this one requires a source extension before its `::`, and carries
//! no digits at all. The two detectors can never see each other's citations, so
//! the two guards never fight. `crates/arm-rules/RULES.md` is deliberately
//! outside this guard for the same reason — see [`scanned_roots`], which also
//! records why `docs/` is outside it.
//!
//! **What is not a defect.** Two carve-outs, both checked rather than asserted:
//! a reference into a **vendored dependency** at a pinned version
//! ([`external_crate_package`] — frozen bytes, and the version is in the path,
//! so a bump makes it visibly stale; and
//! [`every_external_crate_reference_names_a_package_cargo_lock_pins`] proves the
//! pinned version is the one in use), and a reference that names a **file with
//! no line number and no symbol** at all, which is imprecise but cannot rot.
//! Only an integer pointing into live source, or a symbol that no longer
//! exists, fails here.
//!
//! **Scope: comments only.** Both sweeps read [`citation_support::comment_blocks`]
//! / [`citation_support::web_comment_blocks`], so a path inside a string literal
//! — a `Cargo.toml` `path = "src/main.rs"`, a test fixture, an error message —
//! is invisible by construction, the same deliberate boundary
//! `rulebook_citations.rs` draws.

mod citation_support;

use citation_support::{
    comment_blocks, relative, repo_root, rust_files, source_files_in, web_comment_blocks,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// The roots this guard scans for source-to-source citations, and the boundary
/// it deliberately stops at.
///
/// **Inside:** `crates/arm-rules/src`, `crates/arm-app/src`,
/// `crates/arm-rules/tests`, `crates/arm-app/tests` (Rust comments) and
/// `ui/src` (TypeScript/Svelte/CSS comments) — i.e. every **live source file**
/// in the repository. A comment in live source describes the code as it is
/// *now*, so a reference in one is a claim about the present tree and must stay
/// true as the tree moves.
///
/// **Outside, on purpose: `docs/`.** It holds the overwhelming majority of the
/// repository's source line citations (several hundred, against the couple of
/// dozen in live source), and every one of them sits in a **dated historical
/// record** — an implementation plan, a review, a manual-testing findings sheet.
/// Those documents are snapshots of what the code looked like on the day they
/// were written, and a line number in one is part of the snapshot. Rewriting
/// them to cite today's symbols would not repair them, it would *falsify* them:
/// the symbol a line held in August is not necessarily the symbol that line
/// holds now, and the document's whole value is that it records the former.
/// This is the same call D1c made for `docs/audit-2026-08.md` under the
/// rulebook guard.
///
/// **Outside, on purpose: `crates/arm-rules/RULES.md`.** Not because it is
/// dated — it is the most live document in the repository — but because it
/// already has a line-citation convention *with its own guard*:
/// `rules_md_citations.rs` reads its `` `symbol` `` + paren-bare
/// implementation-site citations and **verifies** that the cited line really
/// defines the named symbol. A blanket "never cite a line" rule applied there
/// would contradict a working, checked convention rather than add to it. The
/// two guards must never fight, so this one stays out of that document.
///
/// **Inside, too: `ui/e2e`** (`.js` comments, the same `//` and `/** */` syntax
/// [`web_comment_blocks`] reads for `ui/src`). It was once left out on the
/// claim that no comment there cited a source line; `magus-editor.e2e.js`
/// pinning `derived.rs` by line proved otherwise. The e2e specs are live code
/// that moves with the tree like any other.
fn scanned_roots() -> (Vec<PathBuf>, Vec<WebRoot>) {
    let rust = vec![
        repo_root().join("crates/arm-rules/src"),
        repo_root().join("crates/arm-app/src"),
        repo_root().join("crates/arm-rules/tests"),
        repo_root().join("crates/arm-app/tests"),
    ];
    let web: Vec<WebRoot> = vec![
        (repo_root().join("ui/src"), &["ts", "svelte", "css"]),
        (repo_root().join("ui/e2e"), &["js"]),
    ];
    (rust, web)
}

/// A web root and the file extensions scanned under it.
type WebRoot = (PathBuf, &'static [&'static str]);

/// This guard's own test file, excluded from every scan below: its fixtures are
/// deliberately malformed citations — `` `types.rs:3329` ``, an elided symbol —
/// that exist to document and prove the detectors, not to record real
/// provenance. Left unexcluded, the guard would flag its own examples as
/// violations of itself. The same deliberate carve-out
/// `rulebook_citations.rs`'s `SELF_EXCLUDED_FILES` makes, for the same reason.
const SELF_EXCLUDED_FILES: &[&str] = &["source_citations.rs"];

fn is_self_excluded(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| SELF_EXCLUDED_FILES.contains(&name))
}

/// Every comment block in every scanned file, paired with its file and 1-based
/// starting line — the single input both live-tree sweeps below read.
fn scanned_comment_blocks() -> Vec<(PathBuf, usize, String)> {
    let (rust_roots, web_roots) = scanned_roots();
    let mut blocks = Vec::new();
    let mut rust = Vec::new();
    for root in &rust_roots {
        rust_files(root, &mut rust);
    }
    rust.sort();
    for path in rust {
        if is_self_excluded(&path) {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        for (line, text) in comment_blocks(&content) {
            blocks.push((path.clone(), line, text));
        }
    }
    let mut web = Vec::new();
    for (root, extensions) in &web_roots {
        source_files_in(root, extensions, &mut web);
    }
    web.sort();
    for path in web {
        if is_self_excluded(&path) {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        for (line, text) in web_comment_blocks(&content) {
            blocks.push((path.clone(), line, text));
        }
    }
    blocks
}

/// One `<file>::<symbol>` source-to-source citation.
#[derive(Debug, PartialEq, Eq)]
struct SymbolCitation {
    /// The path as written — a full path or any suffix of one.
    file: String,
    /// The `::`-joined symbol path as written (`Entity::selections`).
    symbol_path: String,
    /// The last segment of `symbol_path`: the name actually looked up.
    symbol: String,
}

/// Reads a `::`-joined run of plain identifiers starting at `from`, returning
/// it and the byte offset just past it. `None` when the first segment is not an
/// identifier — which is what makes `"mirrors effective.rs::..."` prose rather
/// than a malformed citation.
fn read_symbol_path(text: &str, from: usize) -> Option<(String, usize)> {
    let mut segments: Vec<&str> = Vec::new();
    let mut at = from;
    loop {
        let rest = &text[at..];
        let len = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        if len == 0 || rest.starts_with(|c: char| c.is_ascii_digit()) {
            break;
        }
        segments.push(&rest[..len]);
        at += len;
        match text[at..].strip_prefix("::") {
            Some(_) => at += 2,
            None => break,
        }
    }
    if segments.is_empty() {
        return None;
    }
    Some((segments.join("::"), at))
}

/// Finds every `<file>::<symbol>` citation in `text`: a path ending in one of
/// [`SOURCE_EXTENSIONS`], `::`, then a `::`-joined run of identifiers.
///
/// Backticks are the house style (`` `types.rs::Entity::selections` ``) but are
/// deliberately **not** required: the shape `<something>.rs::<identifier>` does
/// not occur in ordinary prose, and requiring the backticks would silently
/// exempt the several live citations that were written without them.
fn find_symbol_citations(text: &str) -> Vec<SymbolCitation> {
    let mut found: Vec<(usize, SymbolCitation)> = Vec::new();
    for extension in SOURCE_EXTENSIONS {
        let needle = format!(".{extension}::");
        let mut search_from = 0usize;
        while let Some(rel) = text[search_from..].find(needle.as_str()) {
            let dot_at = search_from + rel;
            let after_separator = dot_at + needle.len();
            search_from = after_separator;
            let Some((symbol_path, end)) = read_symbol_path(text, after_separator) else {
                continue;
            };
            let mut start = dot_at;
            while start > 0 && is_path_char(text[..start].chars().next_back().unwrap_or(' ')) {
                start -= text[..start].chars().next_back().map_or(0, char::len_utf8);
            }
            let symbol = symbol_path
                .rsplit("::")
                .next()
                .unwrap_or(&symbol_path)
                .to_string();
            found.push((
                dot_at,
                SymbolCitation {
                    file: text[start..dot_at + extension.len() + 1].to_string(),
                    symbol_path,
                    symbol,
                },
            ));
            search_from = end;
        }
    }
    found.sort_by_key(|(at, _)| *at);
    found.into_iter().map(|(_, citation)| citation).collect()
}

#[test]
fn symbol_citation_detector_reads_the_form_and_ignores_what_is_not_one() {
    let fixture = "\
The canonical form, backticked: `types.rs::Entity::selections`.\n\
A path-qualified one, unbackticked: crates/arm-rules/src/effective/xp.rs::build_spends.\n\
A bare-file one: `effective.rs::flawless_magic_floors_first_mastery_free`.\n\
An elided one is still a citation, and must not resolve: `x.rs::a_fresh_wizard_magus_`.\n\
An ellipsis after the separator is prose, not a citation: \"mirrors effective.rs::...\".\n\
A Rust intra-doc path has no file extension: [`crate::derived::magic_resistance`].\n\
A line-number citation is the other detector's subject: `types.rs:3329`.\n\
";
    let found = find_symbol_citations(fixture);
    let pairs: Vec<(&str, &str)> = found
        .iter()
        .map(|c| (c.file.as_str(), c.symbol.as_str()))
        .collect();
    assert_eq!(
        pairs,
        vec![
            ("types.rs", "selections"),
            ("crates/arm-rules/src/effective/xp.rs", "build_spends"),
            ("effective.rs", "flawless_magic_floors_first_mastery_free"),
            ("x.rs", "a_fresh_wizard_magus_"),
        ],
        "found {found:?}"
    );
    assert_eq!(found[0].symbol_path, "Entity::selections");
}

/// The keywords a cited symbol can be introduced by, in either language this
/// repository is written in. Deliberately textual: resolving a symbol properly
/// would mean parsing Rust *and* TypeScript *and* Svelte, and a declaration
/// keyword followed by the name is an honest, dependency-free stand-in for
/// "this name is still declared here".
const DECLARATION_KEYWORDS: &[&str] = &[
    "fn",
    "struct",
    "enum",
    "trait",
    "const",
    "static",
    "type",
    "mod",
    "let",
    "var",
    "function",
    "class",
    "interface",
];

/// The characters that may follow a bare symbol at the start of a line for it
/// to be a declaration rather than a use: a field or typed binding (`:`), an
/// enum variant (`,`), a tuple variant or call-shaped form (`(`), a struct
/// variant (`{`). `=>` is excluded by requiring the character itself, so a
/// match arm is never mistaken for a variant declaration.
const DECLARATION_TERMINATORS: &[char] = &[':', ',', '(', '{'];

/// Visibility/export prefixes stripped before the bare-symbol check.
const VISIBILITY_PREFIXES: &[&str] = &["pub(crate) ", "pub(super) ", "pub ", "export "];

/// True when the character at `index` (if any) cannot continue an identifier —
/// what keeps a citation of `foo` from resolving against a declaration of
/// `foo_bar`.
fn ends_identifier(text: &str, index: usize) -> bool {
    !text[index..]
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// True when `line` introduces `symbol` — the textual stand-in for "the symbol
/// still exists in this file". Two shapes: a [`DECLARATION_KEYWORDS`] keyword
/// immediately followed by the name, and the name itself opening the line and
/// followed by a [`DECLARATION_TERMINATORS`] character (a struct field, an enum
/// variant, an interface member).
fn declares(line: &str, symbol: &str) -> bool {
    for keyword in DECLARATION_KEYWORDS {
        let needle = format!("{keyword} {symbol}");
        if let Some(at) = line.find(needle.as_str())
            && ends_identifier(line, at + needle.len())
        {
            return true;
        }
    }
    let mut bare = line.trim_start();
    for prefix in VISIBILITY_PREFIXES {
        if let Some(rest) = bare.strip_prefix(prefix) {
            bare = rest.trim_start();
            break;
        }
    }
    let Some(rest) = bare.strip_prefix(symbol) else {
        return false;
    };
    rest.trim_start()
        .starts_with(|c: char| DECLARATION_TERMINATORS.contains(&c))
}

#[test]
fn declaration_matcher_accepts_every_shape_a_cited_symbol_is_introduced_by() {
    // Rust: keyword forms, a struct field, an enum variant.
    assert!(declares(
        "pub(crate) fn validate_xp_pool(",
        "validate_xp_pool"
    ));
    assert!(declares("pub struct Entity {", "Entity"));
    assert!(declares(
        "    pub selections: Vec<Selection>,",
        "selections"
    ));
    assert!(declares("    MightGrant {", "MightGrant"));
    assert!(declares(
        "const MAX_XP_SOLVE_NODES: usize = 4096;",
        "MAX_XP_SOLVE_NODES"
    ));

    // TypeScript / Svelte: the same two shapes, different keywords.
    assert!(declares(
        "  function trackFocus(event: FocusEvent): void {",
        "trackFocus"
    ));
    assert!(declares(
        "  let lastFocusOutsideDialog: HTMLElement | null = null;",
        "lastFocusOutsideDialog"
    ));
    assert!(declares(
        "export interface CrisisPreview {",
        "CrisisPreview"
    ));

    // A *use* is not a declaration, in any of the three shapes that could be
    // mistaken for one.
    assert!(!declares("    self.selections.push(sel);", "selections"));
    assert!(!declares(
        "        Effect::MightGrant { .. } => {}",
        "MightGrant"
    ));
    assert!(!declares(
        "/// `Entity::selections` is the list.",
        "selections"
    ));

    // The case this guard's second live finding turned on: a symbol broken by
    // a comment line wrap leaves a *prefix* of the real name, and a prefix must
    // not resolve — otherwise a truncated citation reads as a valid one.
    assert!(!declares(
        "fn a_fresh_wizard_magus_already_carries_warnings() {",
        "a_fresh_wizard_magus_"
    ));
}

/// Every `name-version` this workspace's `Cargo.lock` pins, in the same
/// spelling a vendored-source path uses (`muda-0.19.3`). Read line-wise rather
/// than through a TOML parser: a lockfile's `[[package]]` blocks are a fixed,
/// flat shape, and this guard adds no dependency to check a comment.
fn cargo_lock_packages() -> BTreeSet<String> {
    let lock = fs::read_to_string(repo_root().join("Cargo.lock")).expect("Cargo.lock is readable");
    let mut packages = BTreeSet::new();
    let mut name: Option<String> = None;
    let unquote = |value: &str| value.trim().trim_matches('"').to_string();
    for line in lock.lines() {
        if let Some(value) = line.strip_prefix("name = ") {
            name = Some(unquote(value));
        } else if let Some(value) = line.strip_prefix("version = ")
            && let Some(name) = name.take()
        {
            packages.insert(format!("{name}-{}", unquote(value)));
        }
    }
    packages
}

#[test]
fn every_external_crate_reference_names_a_package_cargo_lock_pins() {
    // What turns the external-crate exemption from "unchecked" into "checked":
    // a comment may point into a dependency by line, but only at the version
    // the workspace actually resolves, so a dependency bump that invalidates
    // the line number fails here instead of rotting in silence.
    let pinned = cargo_lock_packages();
    assert!(
        pinned.len() > 50,
        "expected Cargo.lock to pin many packages, parsed {}",
        pinned.len()
    );

    let blocks = scanned_comment_blocks();
    let mut referenced: BTreeSet<(String, String)> = BTreeSet::new();
    for (path, line, text) in &blocks {
        for snippet in find_source_line_citations(text) {
            if let Some(package) = external_crate_package(&snippet) {
                referenced.insert((package.to_string(), format!("{}:{line}", relative(path))));
            }
        }
    }
    assert!(
        !referenced.is_empty(),
        "expected the tree to carry external crate references (muda/tauri/wry); found none, \
         which means the detector or the walker is broken"
    );

    let stale: Vec<String> = referenced
        .iter()
        .filter(|(package, _)| !pinned.contains(package))
        .map(|(package, at)| format!("{at}: cites {package}, which Cargo.lock does not pin"))
        .collect();
    assert!(
        stale.is_empty(),
        "{} external crate reference(s) name a version the workspace no longer uses:\n{}",
        stale.len(),
        stale.join("\n")
    );
}

/// Every file a citation's path can resolve against: every file of a
/// [`SOURCE_EXTENSIONS`] type under the scanned roots.
fn resolvable_source_files() -> Vec<PathBuf> {
    let (mut roots, web_roots) = scanned_roots();
    roots.extend(web_roots.into_iter().map(|(root, _)| root));
    let mut files = Vec::new();
    for root in &roots {
        source_files_in(root, SOURCE_EXTENSIONS, &mut files);
    }
    files.sort();
    files.dedup();
    files
}

/// The files whose path ends with `cited` at a segment boundary. More than one
/// match is not an error: `commands.rs` names both `arm-app/src/commands.rs`
/// and `arm-app/tests/commands.rs`, and a citation that resolves in *either* is
/// a citation the reader can follow. Only "declared in none of them" is a
/// defect.
fn candidates_for<'a>(cited: &str, files: &'a [PathBuf]) -> Vec<&'a PathBuf> {
    let suffix = format!("/{cited}");
    files
        .iter()
        .filter(|path| {
            let text = path.to_string_lossy().replace('\\', "/");
            text.ends_with(&suffix) || text == cited
        })
        .collect()
}

#[test]
fn every_source_symbol_citation_resolves_to_a_declaration() {
    let blocks = scanned_comment_blocks();
    let files = resolvable_source_files();
    assert!(
        files.len() > 100,
        "expected many resolvable source files, found {}",
        files.len()
    );

    let citations: Vec<(&PathBuf, usize, SymbolCitation)> = blocks
        .iter()
        .flat_map(|(path, line, text)| {
            find_symbol_citations(text)
                .into_iter()
                .map(move |citation| (path, *line, citation))
        })
        .collect();
    // The floor that keeps this sweep from passing vacuously: the form is in
    // live use across both languages, so finding none means the detector or the
    // walker is broken, not that the tree is clean.
    assert!(
        citations.len() >= 15,
        "expected the `<file>::<symbol>` form to be in wide use, found {}",
        citations.len()
    );

    let mut errors = Vec::new();
    for (path, line, citation) in &citations {
        let SymbolCitation {
            file,
            symbol_path,
            symbol,
        } = citation;
        let candidates = candidates_for(file, &files);
        if candidates.is_empty() {
            errors.push(format!(
                "{}:{line}: cites `{file}::{symbol_path}`, but no source file in the repository \
                 is named {file}",
                relative(path)
            ));
            continue;
        }
        let resolved = candidates.iter().any(|candidate| {
            fs::read_to_string(candidate)
                .unwrap_or_default()
                .lines()
                .any(|source_line| declares(source_line, symbol))
        });
        if !resolved {
            let where_looked: Vec<String> = candidates.iter().map(|c| relative(c)).collect();
            errors.push(format!(
                "{}:{line}: cites `{file}::{symbol_path}`, but `{symbol}` is declared in none of \
                 {}",
                relative(path),
                where_looked.join(", ")
            ));
        }
    }
    assert!(
        errors.is_empty(),
        "{} source symbol citation(s) no longer resolve:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

#[test]
fn the_self_exclusion_is_load_bearing_and_keeps_this_file_out_of_both_sweeps() {
    // A characterization test, written after the exclusion rather than before
    // it: [`is_self_excluded`] had to exist for `no_comment_cites_a_source_file_by_line_number`
    // to be meaningful at all, because this file's own doc comments quote the
    // malformed shapes they describe. What it pins down is that the carve-out is
    // *needed* — if the fixtures are ever moved out of comments, this fails and
    // the exclusion can be deleted rather than left standing unexplained.
    // `file!()` is workspace-relative; an integration test's working directory
    // is its own package, so it has to be re-anchored at the repo root.
    let own = fs::read_to_string(repo_root().join(file!())).expect("this test file is readable");
    let quoted: usize = comment_blocks(&own)
        .iter()
        .map(|(_, text)| find_source_line_citations(text).len())
        .sum();
    assert!(
        quoted > 0,
        "expected this file's own comments to quote at least one line-number citation as an \
         illustration; if they no longer do, SELF_EXCLUDED_FILES is dead weight"
    );

    let scanned: BTreeSet<String> = scanned_comment_blocks()
        .iter()
        .map(|(path, _, _)| relative(path))
        .collect();
    for excluded in SELF_EXCLUDED_FILES {
        assert!(
            !scanned.iter().any(|path| path.ends_with(excluded)),
            "{excluded} must be excluded from the scan (it is this guard's own test file)"
        );
    }
}

#[test]
fn ui_e2e_files_scan_a_nonzero_floor() {
    // `ui/e2e` was once left out on the claim that no comment there cited a
    // source line; `magus-editor.e2e.js` pinning `derived.rs` by line proved
    // otherwise. The same vacuous-pass guard as the other roots: the e2e
    // helpers and specs must actually be walked and their comments scanned.
    let e2e_root = repo_root().join("ui/e2e");
    let e2e_files: BTreeSet<PathBuf> = scanned_comment_blocks()
        .into_iter()
        .filter(|(path, _, _)| path.starts_with(&e2e_root))
        .map(|(path, _, _)| path)
        .collect();
    assert!(
        e2e_files.len() > 10,
        "expected `ui/e2e` to contribute many commented .js files, found {}",
        e2e_files.len()
    );
}

#[test]
fn no_comment_cites_a_source_file_by_line_number() {
    let blocks = scanned_comment_blocks();
    // The floor that keeps this sweep from passing vacuously: a broken walker
    // or a wrong root would otherwise scan nothing and report clean.
    assert!(
        blocks.len() > 500,
        "expected to scan many comment blocks across the source roots, saw {}",
        blocks.len()
    );

    let mut offenders: BTreeSet<String> = BTreeSet::new();
    for (path, line, text) in &blocks {
        for snippet in find_source_line_citations(text) {
            if external_crate_package(&snippet).is_some() {
                continue;
            }
            offenders.insert(format!(
                "{}:{line}: cites {snippet} by line number; a source line moves under every \
                 edit — name the symbol instead, as `<file>::<symbol>`",
                relative(path)
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "{} comment(s) cite a source file by line number:\n{}",
        offenders.len(),
        offenders.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

/// The file extensions a source-to-source citation can name. Deliberately the
/// languages this repository is written in — a citation of anything else is not
/// this guard's subject.
const SOURCE_EXTENSIONS: &[&str] = &["rs", "ts", "js", "mjs", "cjs", "svelte", "css"];

/// The characters a cited path is built from, walking backwards from its
/// extension: identifier characters plus the path/segment punctuation a real
/// path uses. Anything else (a backtick, a space, an opening paren, a quote)
/// bounds the token, which is what lets the same scan read a backticked, a
/// parenthesised and a bare citation without knowing which it is looking at.
fn is_path_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-')
}

/// Parses a leading run of ASCII digits from `s`, returning how many bytes it
/// consumed.
fn digit_run(s: &str) -> usize {
    s.bytes().take_while(u8::is_ascii_digit).count()
}

/// Parses `:NNN` or `:NNN-MMM` starting at the colon at byte `colon_at`,
/// returning the byte offset just past the number(s).
fn line_number_end(text: &str, colon_at: usize) -> Option<usize> {
    let after = &text[colon_at + 1..];
    let first = digit_run(after);
    if first == 0 {
        return None;
    }
    let mut end = colon_at + 1 + first;
    if let Some(rest) = text[end..].strip_prefix('-') {
        let second = digit_run(rest);
        if second > 0 {
            end += 1 + second;
        }
    }
    Some(end)
}

/// Finds every source-to-source **line-number** citation in `text`: a path
/// ending in one of [`SOURCE_EXTENSIONS`], immediately followed by `:NNN` or
/// `:NNN-MMM`. Returns the matched `path:lines` snippet for the failure message.
///
/// The extension is the whole discrimination rule, and it is what keeps this
/// detector from ever colliding with the rulebook guard's: a rulebook citation
/// is `ACRONYM:NNNN`, whose colon is preceded by letters that are not a file
/// extension, and a `.md:NNNN` shape belongs to `rulebook_citations.rs`. A
/// timestamp (`10:30`) or a version (`2:15`) has no extension before the colon
/// and is never matched.
fn find_source_line_citations(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for extension in SOURCE_EXTENSIONS {
        let needle = format!(".{extension}:");
        let mut search_from = 0usize;
        while let Some(rel) = text[search_from..].find(needle.as_str()) {
            let dot_at = search_from + rel;
            let colon_at = dot_at + needle.len() - 1;
            search_from = colon_at + 1;
            let Some(end) = line_number_end(text, colon_at) else {
                continue;
            };
            let mut start = dot_at;
            while start > 0 && is_path_char(text[..start].chars().next_back().unwrap_or(' ')) {
                start -= text[..start].chars().next_back().map_or(0, char::len_utf8);
            }
            found.push(text[start..end].to_string());
            search_from = end;
        }
    }
    found.sort_by_key(|snippet| text.find(snippet.as_str()).unwrap_or(usize::MAX));
    found
}

/// The `name-x.y.z` package directory a citation's path starts with when it
/// points into a **vendored dependency** rather than into this repository —
/// `muda-0.19.3/src/accelerator.rs`, `wry-0.55.1/src/webview2/mod.rs`. `None`
/// for any path whose first segment is not version-pinned, which includes every
/// in-repo path (`crates/arm-rules/src/...`, `types.rs`) even though this
/// workspace's own directory names are hyphenated.
///
/// Why these are exempt from "cite the symbol, never the line": the whole
/// premise of this guard is that a source line number rots because the file is
/// under edit. A dependency at a pinned version is not — its bytes are frozen
/// for that version, exactly like a rulebook — and the version is written into
/// the path, so a dependency bump makes the citation *visibly* stale rather
/// than silently wrong. It is also not resolvable: the file is not in this
/// repository, so no symbol check could run against it. What can be checked is
/// that the pinned version is the one actually in use, and
/// [`every_external_crate_reference_names_a_package_cargo_lock_pins`] checks it.
fn external_crate_package(snippet: &str) -> Option<&str> {
    let first_segment = snippet.split('/').next()?;
    let (_, version) = first_segment.rsplit_once('-')?;
    let mut parts = version.split('.');
    let numeric = |part: Option<&str>| {
        part.is_some_and(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    };
    let looks_like_semver = numeric(parts.next())
        && numeric(parts.next())
        && numeric(parts.next())
        && parts.next().is_none();
    looks_like_semver.then_some(first_segment)
}

#[test]
fn external_crate_reference_is_recognised_by_its_version_pinned_first_segment() {
    // The three real shapes in the tree, all in `arm-app`'s menu/window code,
    // where a comment has to say what muda/tauri/wry actually do.
    assert_eq!(
        external_crate_package("muda-0.19.3/src/accelerator.rs:539-541"),
        Some("muda-0.19.3")
    );
    assert_eq!(
        external_crate_package("tauri-2.11.3/src/menu/normal.rs:69"),
        Some("tauri-2.11.3")
    );
    assert_eq!(
        external_crate_package("wry-0.55.1/src/webview2/mod.rs:616-619"),
        Some("wry-0.55.1")
    );

    // An in-repo path is never one, including the hyphenated crate directories
    // this workspace happens to use — `arm-rules` is a hyphen with no semver
    // after it, which is exactly the near-miss a naive "contains a hyphen" test
    // would swallow.
    assert_eq!(
        external_crate_package("crates/arm-rules/src/effective/xp.rs:568"),
        None
    );
    assert_eq!(external_crate_package("arm-rules/src/types.rs:3329"), None);
    assert_eq!(external_crate_package("types.rs:3329"), None);
}

#[test]
fn line_citation_detector_flags_a_source_line_citation_and_ignores_near_misses() {
    let fixture = "\
A backticked Rust citation: `types.rs:3329` names a field by line.\n\
A parenthesised one: (App.svelte:196-210) names a range.\n\
A bare one: see aging.rs:1535-1541 for the reasoning.\n\
A path-qualified one: crates/arm-rules/src/effective/xp.rs:568 is still one.\n\
A file with no line number is fine: see derive.ts for the helper.\n\
A symbol citation is the wanted form: `types.rs::Entity::selections`.\n\
A timestamp is not a citation: the clock read 10:30 that morning.\n\
A version is not a citation: protocol 2:15 is unrelated.\n\
";
    let found = find_source_line_citations(fixture);
    assert_eq!(
        found,
        vec![
            "types.rs:3329".to_string(),
            "App.svelte:196-210".to_string(),
            "aging.rs:1535-1541".to_string(),
            "crates/arm-rules/src/effective/xp.rs:568".to_string(),
        ],
        "expected exactly the four line-number citations, found {found:?}"
    );
}
