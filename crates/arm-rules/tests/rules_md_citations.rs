//! B11: the provenance sweep found that `crates/arm-rules/RULES.md`'s
//! *implementation-site* citations rot silently. RULES.md names the function
//! that implements each rule and pins it with a line number — `validate_caps`
//! (:22) — and nothing checked that the number still points at the function.
//! It usually does not: any edit above a function shifts it, and RULES.md grew
//! by a thousand lines in one phase, so the numbers drift within days and a
//! reader who follows one lands in the middle of an unrelated check.
//!
//! `tests/rules_source_provenance.rs` already guards the other direction — the
//! `source: { file, lines }` blocks in `rules/core/*.json` that point *out* at
//! the rulebook Markdown. This file guards the citations that point *in* at
//! Rust code, which that test cannot see.
//!
//! Deliberately bounded, in the same spirit: it does not parse Rust, and it
//! does not attempt to verify the *rulebook* citations in RULES.md prose. A
//! bare `` `:2860` `` inherits its book from the surrounding section, so
//! resolving one needs a heuristic, and the only defect a bounds check could
//! catch there (a line past EOF) is not the defect that actually occurs (an
//! off-by-one, which lands on a real line). The rulebook line ranges that
//! carry *data* are already covered by `rules_source_provenance.rs`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// `crates/arm-rules` (this crate's manifest dir).
fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn repo_root() -> PathBuf {
    crate_dir().join("../..")
}

/// How many lines above a citation to look for the file/symbol it belongs to.
/// RULES.md is hard-wrapped, so a citation is regularly separated from the
/// `.rs` path it refers to by a line break; two lines of slack covers every
/// shape in the document without letting an unrelated paragraph supply the
/// answer.
const CONTEXT_LINES: usize = 2;

/// One `(:NNN)` / `, :NNN` implementation-site citation found in RULES.md.
#[derive(Debug)]
struct CodeCitation {
    /// 1-based line in RULES.md, for the failure message.
    rules_md_line: usize,
    /// The `.rs` path as written (`validation/caps.rs`, `caps.rs`, …).
    file: String,
    /// The item the citation claims lives at `line`.
    symbol: String,
    /// The cited 1-based line in `file`.
    line: usize,
}

/// A backticked token in RULES.md prose, with the byte offset it started at,
/// so callers can take only the tokens that precede a citation.
struct Tick {
    start: usize,
    text: String,
}

/// Splits one RULES.md line into its backtick-delimited tokens. RULES.md never
/// nests backticks, so a plain alternating scan is exact.
fn backticked_tokens(line: &str) -> Vec<Tick> {
    let mut tokens = Vec::new();
    let mut rest = line;
    let mut base = 0usize;
    while let Some(open) = rest.find('`') {
        let after_open = open + 1;
        let Some(close_rel) = rest[after_open..].find('`') else {
            break;
        };
        let close = after_open + close_rel;
        tokens.push(Tick {
            start: base + open,
            text: rest[after_open..close].to_string(),
        });
        base += close + 1;
        rest = &rest[close + 1..];
    }
    tokens
}

/// True for a token that is a bare Rust identifier — the shape a cited
/// function/struct name takes. Rejects paths (`validation/caps.rs`), paths with
/// a member (`Prereq::House`), field accesses and prose.
fn is_plain_identifier(token: &str) -> bool {
    !token.is_empty()
        && token.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Finds every implementation-site citation on `line`: `(:22)` or `, :524`.
///
/// A *rulebook* citation is written inside backticks (`` `:2860` ``) and an
/// implementation-site one is not, which is what keeps the two apart without
/// any knowledge of which numbers are plausible.
fn citation_offsets(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut found = Vec::new();
    for (idx, _) in line.match_indices(':') {
        if idx == 0 {
            continue;
        }
        let opener = bytes[idx - 1];
        let paren = opener == b'(';
        let comma_space = opener == b' ' && idx >= 2 && bytes[idx - 2] == b',';
        if !paren && !comma_space {
            continue;
        }
        let digits_start = idx + 1;
        let digits_end = digits_start
            + line[digits_start..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
        if digits_end == digits_start {
            continue;
        }
        // A closing backtick would make this a rulebook citation; a hyphen
        // would make it a range, which implementation sites never use.
        let terminator = bytes.get(digits_end).copied();
        if !matches!(terminator, None | Some(b')' | b',' | b'.' | b' ')) {
            continue;
        }
        let number: usize = line[digits_start..digits_end].parse().expect("digits");
        found.push((idx - 1, number));
    }
    found
}

/// Resolves the `.rs` file and the symbol a citation at `offset` on line
/// `index` refers to, by scanning backticked tokens backwards from the
/// citation through the current line and up to [`CONTEXT_LINES`] above it.
///
/// The file is the nearest preceding token containing `.rs`; the symbol is that
/// token's `::suffix` if it has one (`validation/magus.rs::validate_xp_pool`),
/// otherwise the nearest preceding bare identifier.
fn resolve_target(lines: &[&str], index: usize, offset: usize) -> Option<(String, String)> {
    let mut preceding: Vec<String> = Vec::new();
    for tick in backticked_tokens(lines[index]) {
        if tick.start < offset {
            preceding.push(tick.text);
        }
    }
    preceding.reverse();
    let start = index.saturating_sub(CONTEXT_LINES);
    for above in (start..index).rev() {
        let mut earlier: Vec<String> = backticked_tokens(lines[above])
            .into_iter()
            .map(|t| t.text)
            .collect();
        earlier.reverse();
        preceding.extend(earlier);
    }

    let file_token = preceding.iter().find(|t| t.contains(".rs"))?;
    let (path, member) = match file_token.split_once("::") {
        Some((path, member)) => (path, Some(member.to_string())),
        None => (file_token.as_str(), None),
    };
    let symbol = match member {
        Some(member) => member,
        None => preceding.iter().find(|t| is_plain_identifier(t)).cloned()?,
    };
    Some((path.to_string(), symbol))
}

fn collect_code_citations(rules_md: &str) -> Vec<CodeCitation> {
    let lines: Vec<&str> = rules_md.lines().collect();
    let mut citations = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        for (offset, number) in citation_offsets(line) {
            if let Some((file, symbol)) = resolve_target(&lines, index, offset) {
                citations.push(CodeCitation {
                    rules_md_line: index + 1,
                    file,
                    symbol,
                    line: number,
                });
            }
        }
    }
    citations
}

/// Every `.rs` file under `crates/arm-rules/src`, so a citation's partial path
/// can be resolved by suffix without hardcoding the module tree.
fn source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            source_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// The Rust definition keywords a cited symbol can be introduced by.
const DEFINITION_KEYWORDS: [&str; 8] = [
    "fn", "struct", "enum", "trait", "const", "static", "type", "mod",
];

/// True when `line` defines `symbol` — `pub(crate) fn validate_caps(`,
/// `pub(crate) struct EffectiveBudget {`, and so on.
fn defines(line: &str, symbol: &str) -> bool {
    DEFINITION_KEYWORDS
        .iter()
        .any(|kw| line.contains(&format!("{kw} {symbol}")))
}

#[test]
fn every_implementation_site_citation_in_rules_md_points_at_the_named_item() {
    let rules_md_path = crate_dir().join("RULES.md");
    let rules_md = fs::read_to_string(&rules_md_path).expect("RULES.md is readable");
    let citations = collect_code_citations(&rules_md);

    // Without this the whole test can silently degrade to a no-op if the
    // citation format is reworded — the same trap `rules_source_provenance.rs`
    // guards with its `> 100` floor.
    assert!(
        citations.len() >= 20,
        "expected RULES.md to carry many implementation-site citations, found {}",
        citations.len()
    );

    let mut all_sources = Vec::new();
    source_files(&crate_dir().join("src"), &mut all_sources);

    let mut errors = Vec::new();
    for citation in &citations {
        let CodeCitation {
            rules_md_line,
            file,
            symbol,
            line,
        } = citation;

        let matches: Vec<&PathBuf> = all_sources
            .iter()
            .filter(|path| path.to_string_lossy().replace('\\', "/").ends_with(file))
            .collect();
        let [path] = matches.as_slice() else {
            errors.push(format!(
                "RULES.md:{rules_md_line}: `{symbol}` (:{line}) names {file}, which resolves to \
                 {} files under crates/arm-rules/src",
                matches.len()
            ));
            continue;
        };

        let content = fs::read_to_string(path).expect("source file is readable");
        let source_lines: Vec<&str> = content.lines().collect();
        let cited = source_lines.get(line - 1).copied().unwrap_or("");
        if defines(cited, symbol) {
            continue;
        }

        let actual: Vec<String> = source_lines
            .iter()
            .enumerate()
            .filter(|(_, l)| defines(l, symbol))
            .map(|(i, _)| (i + 1).to_string())
            .collect();
        let hint = if actual.is_empty() {
            format!("`{symbol}` is not defined in {file} at all")
        } else {
            format!("`{symbol}` is defined at {file}:{}", actual.join(" / "))
        };
        errors.push(format!(
            "RULES.md:{rules_md_line}: cites `{symbol}` at {file}:{line}, but that line reads \
             {cited:?} — {hint}"
        ));
    }

    assert!(
        errors.is_empty(),
        "{} stale implementation-site citation(s) in RULES.md:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// Source files that may carry a `RULES.md:<line>` citation.
fn documented_source_roots() -> Vec<PathBuf> {
    vec![
        repo_root().join("crates/arm-rules/src"),
        repo_root().join("crates/arm-rules/tests"),
        repo_root().join("crates/arm-app/src"),
    ]
}

/// The reverse direction of the test above, and the reason it is worth a test
/// of its own: a *code* comment must never pin RULES.md by line number.
///
/// RULES.md is prose under constant edit — it gained over a thousand lines in a
/// single phase — so a line range written today points somewhere else within
/// days, and nothing tells the reader it moved. This sweep found all three such
/// citations in the tree pointing at unrelated sections: two claimed the
/// `u8::MAX` sentinel convention and landed on the Gift-prerequisite
/// discussion, and one claimed the `missing_param` precedent and landed on the
/// sentinel paragraph.
///
/// Cite RULES.md by **section heading** instead. Headings are stable across
/// edits, greppable, and say what they point at.
#[test]
fn no_source_comment_cites_rules_md_by_line_number() {
    let mut offenders: BTreeSet<String> = BTreeSet::new();
    let mut scanned = 0usize;

    for root in documented_source_roots() {
        let mut files = Vec::new();
        source_files(&root, &mut files);
        for path in files {
            scanned += 1;
            let content = fs::read_to_string(&path).expect("source file is readable");
            for (index, line) in content.lines().enumerate() {
                for (offset, _) in line.match_indices("RULES.md:") {
                    let after = &line[offset + "RULES.md:".len()..];
                    if after.starts_with(|c: char| c.is_ascii_digit()) {
                        offenders.insert(format!(
                            "{}:{}: {}",
                            path.strip_prefix(repo_root()).unwrap_or(&path).display(),
                            index + 1,
                            line.trim()
                        ));
                    }
                }
            }
        }
    }

    assert!(
        scanned > 10,
        "expected to scan many source files, saw {scanned}"
    );
    assert!(
        offenders.is_empty(),
        "{} source comment(s) cite RULES.md by line number; cite the section heading instead:\n{}",
        offenders.len(),
        offenders.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}
