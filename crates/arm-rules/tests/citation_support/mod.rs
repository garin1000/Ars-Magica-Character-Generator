//! Machinery shared by this crate's two citation guards.
//!
//! `rulebook_citations.rs` (which book does a comment cite, and does the line
//! range land inside it) and `source_citations.rs` (does a comment pin a *source
//! file* by line number, and does a cited symbol still exist) ask completely
//! different questions, but they ask them of the same raw material: the comment
//! text of every Rust and web source file in the tree. That walking-and-joining
//! layer lives here so there is one copy of it rather than two that can drift
//! apart — the second guard was written after the first, and duplicating
//! [`comment_blocks`] would have meant a fix to one scanner silently not
//! reaching the other.
//!
//! Nothing rulebook-specific or symbol-specific belongs here; each guard keeps
//! its own detectors. Every item below is used by **both** guards, which is why
//! no `dead_code` allowance is needed.

use std::fs;
use std::path::{Path, PathBuf};

/// `crates/arm-rules` (the manifest dir of the crate these tests belong to) ->
/// the repository root.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Renders `path` relative to [`repo_root`] for a failure message, falling back
/// to the absolute path when it lies outside the repo.
pub fn relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Every file under `dir` (recursively) whose extension is one of `extensions`.
/// Unreadable directories are skipped rather than panicked on, so a walker
/// pointed at a path that does not exist contributes nothing — which is exactly
/// why both guards carry an explicit "this root found a plausible number of
/// files" floor rather than trusting the walk.
pub fn source_files_in(dir: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            source_files_in(&path, extensions, out);
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| extensions.contains(&ext))
        {
            out.push(path);
        }
    }
}

/// Every `.rs` file under `dir`, recursively.
pub fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    source_files_in(dir, &["rs"], out);
}

/// One contiguous run of `///`, `//!`, or `//` comment lines, with the
/// comment leader and a single leading space stripped, joined by a single
/// space. Citations live only in comments in this codebase, and joining a
/// run into one logical string is what lets a citation's continuation list
/// (`, :NNN`) be found even when hand-wrapped prose splits it — or splits a
/// full basename — across two physical comment lines (observed in the wild,
/// e.g. `mythic_companion.rs::bonus_free_virtue_points`: "...Core Rules).md"
/// wraps onto the following `///` line). Returns each block paired with the
/// 1-based source line it started on, for error messages; precision beyond
/// "which block" is not needed since a human re-finds the exact spot by
/// searching.
pub fn comment_blocks(content: &str) -> Vec<(usize, String)> {
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

/// Extracts every `open`..`close` delimited span from `content` — used for
/// `/* */` block comments (including JSDoc `/** */`) and Svelte/HTML
/// `<!-- -->` comments — pairing each with its 1-based starting line and a
/// [`comment_blocks`]-style joined string (each inner line trimmed, blank
/// lines dropped, joined with a single space; a whole span is naturally one
/// block, unlike a `//` run, since the delimiters already bound it). Also
/// returns a same-length copy of `content` with every matched span
/// (delimiters included) blanked to spaces — newlines preserved — so line
/// numbers stay valid for a follow-up scan over the remainder, and a `//`
/// sequence that happens to sit inside an already-extracted span can never be
/// independently re-matched. See [`web_comment_blocks`].
pub fn extract_delimited_blocks(
    content: &str,
    open: &str,
    close: &str,
) -> (Vec<(usize, String)>, String) {
    let mut blocks = Vec::new();
    let mut masked = String::with_capacity(content.len());
    let mut rest = content;
    let mut line = 1usize;
    loop {
        let Some(open_rel) = rest.find(open) else {
            masked.push_str(rest);
            break;
        };
        let before = &rest[..open_rel];
        masked.push_str(before);
        line += before.matches('\n').count();
        let start_line = line;
        let after_open = &rest[open_rel + open.len()..];
        let Some(close_rel) = after_open.find(close) else {
            // Unterminated span: leave the remainder untouched (defensive; a
            // well-formed source file never hits this).
            masked.push_str(&rest[open_rel..]);
            break;
        };
        let inner = &after_open[..close_rel];
        let joined = inner
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !joined.is_empty() {
            blocks.push((start_line, joined));
        }
        let span_end = open_rel + open.len() + close_rel + close.len();
        let span = &rest[open_rel..span_end];
        for ch in span.chars() {
            masked.push(if ch == '\n' { '\n' } else { ' ' });
        }
        line += span.matches('\n').count();
        rest = &rest[span_end..];
    }
    (blocks, masked)
}

/// [`comment_blocks`]'s analogue for the frontend. Block and HTML comments are
/// extracted first via [`extract_delimited_blocks`]; the masked remainder —
/// same length, newlines intact — is then handed to [`comment_blocks`] to
/// pick up every `//` line-comment run, which matches its Rust `//` handling
/// exactly (a plain `//` line falls through [`comment_blocks`]'s `///`/`//!`
/// checks to its `//` branch either way).
pub fn web_comment_blocks(content: &str) -> Vec<(usize, String)> {
    let (mut blocks, masked1) = extract_delimited_blocks(content, "/*", "*/");
    let (html_blocks, masked2) = extract_delimited_blocks(&masked1, "<!--", "-->");
    blocks.extend(html_blocks);
    blocks.extend(comment_blocks(&masked2));
    blocks.sort_by_key(|(line, _)| *line);
    blocks
}
