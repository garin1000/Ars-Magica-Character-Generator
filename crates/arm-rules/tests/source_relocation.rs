//! P8-1/T1 (`docs/vf-audit/design-p8-resync.md` § "What moves" (a)/(b), §
//! "Heading-sequence diff", § "P8-1"; `docs/vf-audit/decisions.md` D30, D85)
//! — the direction `rules_source_provenance.rs` does not yet provide: given a
//! recorded `source.anchor` and the **new** post-resync Markdown, find the
//! item's new `source.lines`. Today only the reverse exists
//! (`rules_source_provenance.rs::regenerate_source_anchors` walks
//! `lines[0]` -> finds the heading there -> records its anchor); nothing
//! derives a line *from* an anchor, which is exactly what a rulebook re-sync
//! needs — every line number shifts at once, which is D18's whole argument
//! for why `anchor` (`types.rs::SourceRef::anchor`) exists at all.
//!
//! **Design decision — how the new `end` is derived.** Two candidates were on
//! the table: (1) keep the old range's length
//! (`new_end = new_start + (old_end - old_start)`), or (2) recompute the
//! section boundary in the **new** file (the next heading at the same level
//! or shallower, stepped back to the last non-blank line — exactly
//! `rules_source_provenance.rs::section_boundary_after` + D30.2). This file
//! picks (2), per `design-p8-resync.md` § "What moves" (a) ("re-derive
//! `lines[1]` by reusing the existing boundary logic ... then stepping back
//! to the last non-blank line"). Keeping the old span length is wrong
//! whenever the passage itself grew or shrank during the resync — which is
//! the ordinary case, not the exception: the Core Rules' own errata section
//! alone grew by 68 lines between editions
//! (`design-p8-resync.md` § "Body-text sampling"). This file's
//! [`a_line_added_inside_the_section_is_covered_by_the_new_boundary_not_the_old_span_length`]
//! is the concrete case: a line inserted *inside* the section (not merely
//! above it) is covered by the boundary strategy and missed by the
//! span-length one.
//!
//! **Row-key anchors** (`heading/row-key`, `heading/table-key/row-key`,
//! D72.1) — an anchor whose range does not open on a heading at all —
//! relocate in two steps, per design (a): their heading segment the same way
//! as a plain heading anchor, then the row/table-key segment re-resolves
//! against whatever text now sits in that (possibly restructured) section,
//! duplicating `rules_source_provenance.rs::resolve_row_key_anchor`'s own
//! table/row-key derivation logic but applying it to both the OLD and NEW
//! file (for the same old-vs-new count-parity safety check as the heading
//! case) rather than one file alone. [`relocate_row_key_anchor`] below.
//!
//! Self-contained rather than sharing code with `rules_source_provenance.rs`:
//! each integration test file is its own compilation unit, so importing a
//! handful of its functions would need either a shared module whose unused
//! half `cargo clippy --all-targets` flags in *this* binary, or `mod
//! rules_source_provenance;`, which would compile its whole 2400+-line test
//! suite a second time under this binary's name. `x9b_bard_rename.rs`
//! documents the identical tradeoff for the same reason and resolves it the
//! same way.
//!
//! **Phase 1 (RED, done) → Phase 2 (GREEN, this state).**
//! [`relocate_heading_anchor`] and [`relocate_row_key_anchor`] now carry real
//! bodies; every test below is green. The `#[ignore]`d sweep
//! ([`regenerate_source_relocations`], `rules_source_provenance.rs
//! ::regenerate_source_anchors`'s sibling for this direction) applies them
//! across `rules/core/*.json` and the DE sidecar — run deliberately, after an
//! actual resync (P8-2) lands, or as a same-file dry run to prove the tool is
//! an identity on unchanged input (`tmp/t1-handover.md` records that run's
//! counts).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// One Markdown heading, located and slugified — a trimmed duplicate of
/// `rules_source_provenance.rs::Heading`, plus the one field that file does
/// not carry and this one needs: the heading's own title text. That is what
/// lets [`relocate_heading_anchor`] tell an ordinary line-shift apart from a
/// genuine duplicate (see its doc comment) — an anchor string alone cannot
/// be split back into "base slug" and "disambiguation suffix" without
/// guessing, since a heading's *own* title can itself end in digits.
#[derive(Clone, Debug)]
struct Heading {
    /// 1-based line number the `#` sits on.
    line: usize,
    /// How many `#` characters opened it.
    level: usize,
    /// The heading text itself, exactly as printed (pre-slugification).
    title: String,
    /// The anchor the generator gives it, `-N` disambiguation included.
    anchor: String,
}

/// Slugifies a Markdown heading the way the sources' own generated cross-links
/// do — duplicated verbatim from `rules_source_provenance.rs::slugify_heading`.
fn slugify_heading(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

/// Every Markdown heading in `text`, in document order, with the generator's
/// `-1`/`-2` disambiguation for repeated base slugs — duplicated from
/// `rules_source_provenance.rs::headings` (minus the blockquote tracking that
/// file's `Heading` carries, which [`section_boundary_after`] below does not
/// need: it always exempts blockquotes, see its own doc comment).
fn headings(text: &str) -> Vec<Heading> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut found = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let trimmed = trimmed.strip_prefix("> ").unwrap_or(trimmed).trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        let level = trimmed.chars().take_while(|c| *c == '#').count();
        let title = trimmed.trim_start_matches('#').trim();
        if title.is_empty() {
            continue;
        }
        let base = slugify_heading(title);
        let count = seen.entry(base.clone()).or_insert(0);
        let anchor = if *count == 0 {
            base.clone()
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        found.push(Heading {
            line: index + 1,
            level,
            title: title.to_string(),
            anchor,
        });
    }

    found
}

/// The heading that ends `opening`'s section: the next one, at the same
/// level or shallower — duplicated from
/// `rules_source_provenance.rs::section_boundary_after`, always exempting
/// blockquoted sidebars. That file's per-entry `OWN_BLOCKQUOTED_SIDEBARS`
/// exemption list is keyed by catalogue entry id, which belongs to the real
/// sweep tool (Phase 2), not this pure function — a synthetic fixture has no
/// entry id to key by in the first place.
fn section_boundary_after<'a>(
    file_headings: &'a [Heading],
    opening: &Heading,
) -> Option<&'a Heading> {
    file_headings
        .iter()
        .find(|heading| heading.line > opening.line && heading.level <= opening.level)
}

/// The outcome of relocating one heading-anchored `source.lines` range from an
/// old Markdown file to a new one.
#[derive(Debug, PartialEq, Eq)]
enum Relocation {
    Resolved { start: usize, end: usize },
    Unresolved(String),
}

/// Relocates a heading-anchored citation: given the text of the source file
/// **before** a resync (`old_text`), the text **after** (`new_text`), the
/// recorded `anchor`, and the heading level the catalogue expects (the second
/// element of `rules_source_provenance.rs::FULLY_ANCHORED_CATALOGUES`'s own
/// per-catalogue tuple), returns the new `[start, end]` or a reason it could
/// not be determined safely.
///
/// Phase 2's algorithm (committed to by this doc comment, not yet
/// implemented): resolve `anchor` against `old_text`'s own headings to learn
/// its title and how many other headings in `old_text` share its base slug
/// (`old_count`); count how many headings in `new_text` share that same base
/// slug (`new_count`).
///
/// - `new_count == 0` — the heading was reworded or removed. Unresolved,
///   never guessed at (this is D18's whole point: a heading rename is a
///   semantic change worth noticing, not something to paper over with a
///   fuzzy match).
/// - `new_count != old_count` (and not zero) — a heading sharing that title
///   was added or removed *somewhere* in the document. Anchor disambiguation
///   (`-1`, `-2`, ...) is purely positional, so a changed count means the
///   Nth occurrence may no longer be the same heading as before — reported as
///   unresolved (a "duplicated heading", in the brief's words) rather than
///   trusting a direct lookup that could now silently point at the wrong one.
/// - `new_count == old_count` (including the common case `1 == 1`) — `anchor`
///   is looked up directly against `new_text`'s own heading map, trusting
///   that an unchanged count means an unchanged occurrence order, and its
///   level is checked against `level` (a heading promoted/demoted across the
///   resync — the "structurally heavy" risk `design-p8-resync.md` measured
///   for Hedge Magic/Realms of Power — is also unresolved, not silently
///   accepted at the wrong level).
///
/// The new `end` is then the new file's own section boundary — the next
/// heading at the same level or shallower — stepped back to the last
/// non-blank line (D30.2), **not** `new_start + (old_end - old_start)`: see
/// this file's module doc comment for why.
fn relocate_heading_anchor(
    old_text: &str,
    new_text: &str,
    anchor: &str,
    level: usize,
) -> Relocation {
    let (new_line, new_level) = match relocate_heading_line(old_text, new_text, anchor) {
        Err(reason) => return Relocation::Unresolved(reason),
        Ok(found) => found,
    };
    if new_level != level {
        return Relocation::Unresolved(format!(
            "anchor \"#{anchor}\" resolves to a level-{new_level} heading in the new file, but \
             this catalogue's entries are defined under a level-{level} heading — a \
             promoted/demoted heading, not the one this citation named"
        ));
    }

    let new_headings = headings(new_text);
    let opening = new_headings
        .iter()
        .find(|heading| heading.line == new_line)
        .expect("relocate_heading_line only returns a line it found a real heading on");
    Relocation::Resolved {
        start: new_line,
        end: section_end(new_text, &new_headings, opening),
    }
}

/// Resolves `anchor`'s heading-segment move from `old_text` to `new_text`,
/// independent of any expected heading level (row-key anchors relocate their
/// heading segment the same way, at whatever level that heading happens to
/// be — D72.1's own anchor-store comment: "ignored for a catalogue whose
/// entries are all row-key anchors"). Returns the new 1-based line and the
/// level of the heading found there, or `Err` with a reason.
///
/// The algorithm: resolve `anchor` against `old_text`'s own headings to learn
/// its title and how many other headings in `old_text` share its base slug
/// (`old_count`); count how many headings in `new_text` share that same base
/// slug (`new_count`). `new_count == 0` means reworded or removed. `new_count
/// != old_count` (and nonzero) means a same-titled heading appeared or
/// disappeared somewhere in the document — since anchor disambiguation
/// (`-1`, `-2`, ...) is purely positional, a changed count means the Nth
/// occurrence may no longer be the same heading as before, so this is
/// unresolved rather than trusted. Only when the count agrees is `anchor`
/// looked up directly against `new_text`'s own heading map.
fn relocate_heading_line(
    old_text: &str,
    new_text: &str,
    anchor: &str,
) -> Result<(usize, usize), String> {
    let old_headings = headings(old_text);
    let Some(old_heading) = old_headings.iter().find(|heading| heading.anchor == anchor) else {
        return Err(format!(
            "anchor \"#{anchor}\" does not resolve against the OLD file either — there is \
             nothing to relocate from"
        ));
    };
    let base = slugify_heading(&old_heading.title);
    let old_count = old_headings
        .iter()
        .filter(|heading| slugify_heading(&heading.title) == base)
        .count();

    let new_headings = headings(new_text);
    let new_count = new_headings
        .iter()
        .filter(|heading| slugify_heading(&heading.title) == base)
        .count();

    if new_count == 0 {
        return Err(format!(
            "no heading titled like \"{}\" survives in the new file — it was reworded or \
             removed",
            old_heading.title
        ));
    }
    if new_count != old_count {
        return Err(format!(
            "\"{}\" appears {new_count} time(s) in the new file but {old_count} time(s) in the \
             old — ambiguous, a duplicate heading may have been introduced or removed, so which \
             occurrence this anchor now names cannot be trusted",
            old_heading.title
        ));
    }

    let Some(new_heading) = new_headings.iter().find(|heading| heading.anchor == anchor) else {
        return Err(format!(
            "\"{}\" appears the same number of times ({new_count}) in both files, but anchor \
             \"#{anchor}\" does not resolve directly against the new file's own generated \
             anchors — the occurrence order may have changed",
            old_heading.title
        ));
    };
    Ok((new_heading.line, new_heading.level))
}

/// The last non-blank line of `opening`'s own section in `text`: the line
/// before the next heading at the same level or shallower (D30.2), stepped
/// back over any trailing blank lines. `None` boundary (section runs to EOF)
/// falls back to the file's own last line.
fn section_end(text: &str, file_headings: &[Heading], opening: &Heading) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    let boundary = section_boundary_after(file_headings, opening).map(|heading| heading.line);
    let mut end = boundary.map(|line| line - 1).unwrap_or(lines.len());
    while end > opening.line
        && lines
            .get(end - 1)
            .is_some_and(|line| line.trim().is_empty())
    {
        end -= 1;
    }
    end
}

// --- Fixtures ---------------------------------------------------------------

#[test]
fn an_unchanged_heading_resolves_to_its_own_line() {
    let old = "#### Foo\n\nBody line 1.\nBody line 2.\n\n#### Bar\n\nMore.\n";
    let new = old;
    assert_eq!(
        relocate_heading_anchor(old, new, "foo", 4),
        Relocation::Resolved { start: 1, end: 4 }
    );
}

/// The core case a re-sync produces everywhere: nothing about the entry's own
/// text changed, but an edit earlier in the file shifted its line number.
#[test]
fn a_line_inserted_before_the_heading_shifts_the_resolved_start() {
    let old = "#### Foo\n\nBody line 1.\nBody line 2.\n\n#### Bar\n\nMore.\n";
    let new = "Some new preamble line that did not exist before.\n\n#### Foo\n\nBody line 1.\nBody line 2.\n\n#### Bar\n\nMore.\n";
    assert_eq!(
        relocate_heading_anchor(old, new, "foo", 4),
        Relocation::Resolved { start: 3, end: 6 }
    );
}

/// The case that justifies recomputing the section boundary instead of
/// keeping the old range's length: a line added *inside* the section (not
/// merely above it). The old range here would be `[1, 4]` (span length 3).
/// Adding "Body line 3" inside the section means the correct new end is `5`
/// — `new_start + old_span_length` (`1 + 3 = 4`) would stop one line short
/// and silently drop the resync's own new content from the citation.
#[test]
fn a_line_added_inside_the_section_is_covered_by_the_new_boundary_not_the_old_span_length() {
    let old = "#### Foo\n\nBody line 1.\nBody line 2.\n\n#### Bar\n";
    let new = "#### Foo\n\nBody line 1.\nBody line 2.\nBody line 3 \u{2014} new content added by the resync.\n\n#### Bar\n";
    assert_eq!(
        relocate_heading_anchor(old, new, "foo", 4),
        Relocation::Resolved { start: 1, end: 5 }
    );
}

/// A reworded heading (the clean group's dominant failure mode per
/// `design-p8-resync.md`'s measurement pass, e.g. `Rard` -> `Bard`): the old
/// anchor's base slug has zero occurrences in the new file. Must be reported,
/// never guessed at by fuzzy-matching a similarly-spelled heading.
#[test]
fn a_reworded_heading_is_unresolved() {
    let old = "#### Rard\n\nBody text.\n\n#### Bar\n\nMore.\n";
    let new = "#### Bard\n\nBody text.\n\n#### Bar\n\nMore.\n";
    assert!(matches!(
        relocate_heading_anchor(old, new, "rard", 4),
        Relocation::Unresolved(_)
    ));
}

/// A heading removed outright (the section it opened is simply gone from the
/// new file). Same failure family as a rewording — zero occurrences survive
/// — but it is its own named fixture because a removal and a rewording are
/// different real-world events even though they share one code path.
#[test]
fn a_removed_heading_is_unresolved() {
    let old = "#### Foo\n\nBody.\n\n#### Bar\n\nMore.\n";
    let new = "#### Bar\n\nMore.\n";
    assert!(matches!(
        relocate_heading_anchor(old, new, "foo", 4),
        Relocation::Unresolved(_)
    ));
}

/// A heading title that used to be unique now appears twice in the new file.
/// Anchor disambiguation is positional, so even though the literal string
/// "foo" still resolves to *a* heading via the new file's own generated
/// anchor map, there is no way to tell from the new file alone whether that
/// first occurrence is the same entry as before or a newly inserted one that
/// happens to sort earlier in the document. Comparing the occurrence count
/// against the old file is what catches this — reported as unresolved.
#[test]
fn a_duplicated_heading_is_unresolved() {
    let old = "#### Foo\n\nBody.\n\n#### Bar\n\nMore.\n";
    let new = "#### Foo\n\nBody changed.\n\n#### Foo\n\nA different, newly split entry.\n\n#### Bar\n\nMore.\n";
    assert!(matches!(
        relocate_heading_anchor(old, new, "foo", 4),
        Relocation::Unresolved(_)
    ));
}

/// A heading that survives with the same title and the same occurrence count
/// but at a different nesting depth — the "structurally heavy" risk
/// `design-p8-resync.md` measured for Hedge Magic/Realms of Power, where a
/// resync promotes sidebar titles to real headings and can deepen the
/// hierarchy around an existing one. Resolving at the wrong level would
/// silently point a catalogue entry at the wrong kind of heading, so this is
/// unresolved rather than accepted.
#[test]
fn a_heading_whose_level_changed_is_unresolved() {
    let old = "#### Foo\n\nBody.\n\n#### Bar\n";
    let new = "##### Foo\n\nBody.\n\n#### Bar\n";
    assert!(matches!(
        relocate_heading_anchor(old, new, "foo", 4),
        Relocation::Unresolved(_)
    ));
}

/// Per `design-p8-resync.md` § "What moves" (b): the DE sidecar
/// (`rules/i18n/de/source_anchors.json`) relocates "in lock-step" with the
/// English store, via the same line-parity invariant — meaning the same pure
/// function serves both languages, called against the German file instead of
/// the English one. This is that reuse, demonstrated against German heading
/// text (including the `ß`/`ü` that `slugify_heading` must stay
/// Unicode-aware for).
#[test]
fn the_same_function_relocates_a_german_heading_anchor() {
    let old = "#### Verdorben\n\nEin Text.\n\n#### Barde\n";
    let new = "Eine neue Vorrede-Zeile, die es vorher nicht gab.\n\n#### Verdorben\n\nEin Text.\n\n#### Barde\n";
    assert_eq!(
        relocate_heading_anchor(old, new, "verdorben", 4),
        Relocation::Resolved { start: 3, end: 5 }
    );
}

// --- Row-key anchors (`heading/row-key`, `heading/table-key/row-key`, D72.1) -

/// What one non-heading source line's anchor key derives to — a trimmed
/// duplicate of `rules_source_provenance.rs::{LineKind, RowKey,
/// normalize_dashes, slugify_row_text, derive_row_key}`.
#[derive(Clone, Debug, PartialEq, Eq)]
enum LineKind {
    TableRow(usize),
    ListItem,
    Prose,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RowKey {
    kind: LineKind,
    key: String,
}

fn normalize_dashes(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\u{2013}' | '\u{2014}' => '-',
            other => other,
        })
        .collect()
}

fn slugify_row_text(text: &str) -> String {
    slugify_heading(&normalize_dashes(text))
}

fn derive_row_key(line: &str) -> Option<RowKey> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix('|') {
        let cells: Vec<&str> = rest
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let first_cell = *cells.first()?;
        if !first_cell.is_empty() && first_cell.chars().all(|c| c == '-' || c == ':') {
            return None;
        }
        let key = slugify_row_text(first_cell);
        if key.is_empty() {
            return None;
        }
        return Some(RowKey {
            kind: LineKind::TableRow(cells.len()),
            key,
        });
    }

    if let Some(rest) = trimmed.strip_prefix("- ") {
        let text = match rest.strip_prefix('[') {
            Some(after_bracket) => after_bracket.split(']').next().unwrap_or(rest),
            None => rest.split(':').next().unwrap_or(rest),
        };
        let key = slugify_row_text(text);
        if key.is_empty() {
            return None;
        }
        return Some(RowKey {
            kind: LineKind::ListItem,
            key,
        });
    }

    let words: Vec<&str> = trimmed.split_whitespace().take(4).collect();
    if words.is_empty() {
        return None;
    }
    let key = slugify_row_text(&words.join(" "));
    if key.is_empty() {
        return None;
    }
    Some(RowKey {
        kind: LineKind::Prose,
        key,
    })
}

/// Whether an anchor names a row/list/prose line inside a heading's section
/// (`heading/row-key`) rather than the heading itself — duplicated from
/// `rules_source_provenance.rs::is_row_key_anchor`.
fn is_row_key_anchor(anchor: &str) -> bool {
    anchor.contains('/')
}

/// The 1-based line numbers, within `heading`'s own section (bounded by
/// [`section_boundary_after`]), that derive `row_key` — optionally narrowed
/// first to the contiguous table block whose header derives `table_key`
/// (the 3-segment form's own table lookup, D72.1) — a trimmed duplicate of
/// `rules_source_provenance.rs::table_block_with_key` folded together with
/// its caller's candidate-line selection.
fn row_key_candidates(
    file_headings: &[Heading],
    lines: &[&str],
    heading: &Heading,
    table_key: Option<&str>,
    row_key: &str,
) -> Result<Vec<usize>, String> {
    let boundary_line = section_boundary_after(file_headings, heading)
        .map(|h| h.line)
        .unwrap_or(lines.len() + 1);

    let candidate_lines: Vec<usize> = match table_key {
        None => ((heading.line + 1)..boundary_line).collect(),
        Some(table_key) => {
            let is_table_line = |line_no: usize| {
                lines
                    .get(line_no - 1)
                    .is_some_and(|t| t.trim_start().starts_with('|'))
            };
            let mut line_no = heading.line + 1;
            let mut block = None;
            while line_no < boundary_line {
                if !is_table_line(line_no) {
                    line_no += 1;
                    continue;
                }
                let block_start = line_no;
                let mut block_end = line_no;
                while block_end + 1 < boundary_line && is_table_line(block_end + 1) {
                    block_end += 1;
                }
                let header_key = lines
                    .get(block_start - 1)
                    .and_then(|text| derive_row_key(text))
                    .map(|key| key.key);
                if header_key.as_deref() == Some(table_key) {
                    block = Some(((block_start + 1)..=block_end).collect());
                    break;
                }
                line_no = block_end + 1;
            }
            block.ok_or_else(|| {
                format!(
                    "no table with key \"{table_key}\" in \"#{}\"'s section",
                    heading.anchor
                )
            })?
        }
    };

    Ok(candidate_lines
        .into_iter()
        .filter(|&line_no| {
            lines
                .get(line_no - 1)
                .and_then(|text| derive_row_key(text))
                .is_some_and(|key| key.key == row_key)
        })
        .collect())
}

/// Relocates a row-key anchor (`heading/row-key` or
/// `heading/table-key/row-key`, D72.1): its heading segment moves exactly
/// like [`relocate_heading_anchor`]'s (minus the level check, which D72.1
/// deliberately does not apply to this form), then the row-key segment
/// re-resolves within that section, counted old-vs-new exactly like
/// [`relocate_heading_line`]'s title count — the same positional-ambiguity
/// argument applies: a row key that used to be unique in its section and now
/// derives from two lines cannot be trusted to still name the same one.
/// Row-key citations are always single-line (`start == end`), matching every
/// shipped example (`childhoods.json`, `equipment.json`, `aging.json`).
fn relocate_row_key_anchor(old_text: &str, new_text: &str, anchor: &str) -> Relocation {
    let segments: Vec<&str> = anchor.split('/').collect();
    let (heading_slug, table_key, row_key) = match segments.as_slice() {
        [heading_slug, row_key] => (*heading_slug, None, *row_key),
        [heading_slug, table_key, row_key] => (*heading_slug, Some(*table_key), *row_key),
        _ => {
            return Relocation::Unresolved(format!(
                "anchor \"#{anchor}\" is not the 2-segment `heading/row-key` or 3-segment \
                 `heading/table-key/row-key` form"
            ));
        }
    };

    let (new_heading_line, _) = match relocate_heading_line(old_text, new_text, heading_slug) {
        Err(reason) => {
            return Relocation::Unresolved(format!(
                "row-key anchor \"#{anchor}\"'s heading segment: {reason}"
            ));
        }
        Ok(found) => found,
    };

    let old_headings = headings(old_text);
    let old_lines: Vec<&str> = old_text.lines().collect();
    let Some(old_heading) = old_headings.iter().find(|h| h.anchor == heading_slug) else {
        return Relocation::Unresolved(format!(
            "row-key anchor \"#{anchor}\": heading segment \"#{heading_slug}\" does not resolve \
             against the OLD file either"
        ));
    };
    let old_candidates =
        match row_key_candidates(&old_headings, &old_lines, old_heading, table_key, row_key) {
            Ok(c) => c,
            Err(reason) => {
                return Relocation::Unresolved(format!("row-key anchor \"#{anchor}\": {reason}"));
            }
        };

    let new_headings = headings(new_text);
    let new_lines: Vec<&str> = new_text.lines().collect();
    let new_heading = new_headings
        .iter()
        .find(|h| h.line == new_heading_line)
        .expect("relocate_heading_line only returns a line it found a real heading on");
    let new_candidates =
        match row_key_candidates(&new_headings, &new_lines, new_heading, table_key, row_key) {
            Ok(c) => c,
            Err(reason) => {
                return Relocation::Unresolved(format!("row-key anchor \"#{anchor}\": {reason}"));
            }
        };

    if new_candidates.is_empty() {
        return Relocation::Unresolved(format!(
            "row-key anchor \"#{anchor}\": no line in the new file's section derives the key \
             \"{row_key}\" — the row was reworded or removed"
        ));
    }
    if new_candidates.len() != old_candidates.len() {
        return Relocation::Unresolved(format!(
            "row-key anchor \"#{anchor}\": \"{row_key}\" derives from {} line(s) in the new \
             section but {} in the old — ambiguous",
            new_candidates.len(),
            old_candidates.len()
        ));
    }
    if old_candidates.len() != 1 {
        return Relocation::Unresolved(format!(
            "row-key anchor \"#{anchor}\": \"{row_key}\" is not unique even in the old file's \
             section ({} matches) — cannot relocate safely",
            old_candidates.len()
        ));
    }

    let line = new_candidates[0];
    Relocation::Resolved {
        start: line,
        end: line,
    }
}

/// The ordinary case: a row-key anchor whose row survives unchanged, shifted
/// down by a line inserted earlier in the file.
#[test]
fn a_row_key_anchor_shifts_with_its_section() {
    let old = "#### Sample Childhoods\n\n- Athletic Childhood: Athletics 2, Brawl 2\n- Exploring Childhood: Awareness 2, Survival 2\n\n#### Bar\n";
    let new = "Preamble.\n\n#### Sample Childhoods\n\n- Athletic Childhood: Athletics 2, Brawl 2\n- Exploring Childhood: Awareness 2, Survival 2\n\n#### Bar\n";
    assert_eq!(
        relocate_row_key_anchor(old, new, "sample-childhoods/athletic-childhood"),
        Relocation::Resolved { start: 5, end: 5 }
    );
}

/// A row-key anchor whose row was removed from its section entirely.
#[test]
fn a_removed_row_key_is_unresolved() {
    let old = "#### Sample Childhoods\n\n- Athletic Childhood: Athletics 2, Brawl 2\n- Exploring Childhood: Awareness 2, Survival 2\n\n#### Bar\n";
    let new =
        "#### Sample Childhoods\n\n- Exploring Childhood: Awareness 2, Survival 2\n\n#### Bar\n";
    assert!(matches!(
        relocate_row_key_anchor(old, new, "sample-childhoods/athletic-childhood"),
        Relocation::Unresolved(_)
    ));
}

/// A row-key anchor whose key now derives from two lines in its new section
/// (a duplicate list item introduced by the resync) — the row-key twin of
/// [`a_duplicated_heading_is_unresolved`].
#[test]
fn a_duplicated_row_key_is_unresolved() {
    let old = "#### Sample Childhoods\n\n- Athletic Childhood: Athletics 2, Brawl 2\n\n#### Bar\n";
    let new = "#### Sample Childhoods\n\n- Athletic Childhood: Athletics 2, Brawl 2\n- Athletic Childhood: a differently-described duplicate\n\n#### Bar\n";
    assert!(matches!(
        relocate_row_key_anchor(old, new, "sample-childhoods/athletic-childhood"),
        Relocation::Unresolved(_)
    ));
}

/// The 3-segment form (D72.1, aging): the table-key segment disambiguates two
/// tables in the same section whose row-keys collide.
#[test]
fn a_three_segment_row_key_anchor_resolves_within_its_own_table() {
    let old = "## Aging\n\n| Aging Roll | Result |\n| ---------- | ------ |\n| 15         | 1 Aging Point in Sta |\n\n| Crisis Roll | Result |\n| ----------- | ------ |\n| 15          | Minor illness |\n";
    let new = "Preamble.\n\n## Aging\n\n| Aging Roll | Result |\n| ---------- | ------ |\n| 15         | 1 Aging Point in Sta |\n\n| Crisis Roll | Result |\n| ----------- | ------ |\n| 15          | Minor illness |\n";
    assert_eq!(
        relocate_row_key_anchor(old, new, "aging/aging-roll/15"),
        Relocation::Resolved { start: 7, end: 7 }
    );
    assert_eq!(
        relocate_row_key_anchor(old, new, "aging/crisis-roll/15"),
        Relocation::Resolved { start: 11, end: 11 }
    );
}

/// The formula reproduces a plain, unremarkable section's own history
/// exactly — the common case.
#[test]
fn old_boundary_self_check_passes_when_the_formula_already_matches_history() {
    let text = "#### Foo\n\nBody line 1.\nBody line 2.\n\n#### Bar\n";
    assert!(old_boundary_is_self_consistent(text, 1, 4));
}

/// The real-world case the sweep's first identity run discovered beyond the
/// two known guard-exemption lists: `flaw.wrathful_*` (ArMDE:7177-7179)
/// stops at its own last body line, but the generic formula would walk past
/// a horizontal rule and an unrelated blockquoted flavour-text paragraph
/// (the Chapter 5 break's own decoration) to reach the next real heading —
/// overshooting by several lines. The self-check must catch this from the
/// OLD file alone, which is exactly what lets the sweep route it to manual
/// review instead of silently writing the wrong end.
#[test]
fn old_boundary_self_check_fails_when_the_formula_overshoots_unrelated_trailing_content() {
    let text = "#### Foo\n\nBody line 1.\n\n---\n\n> Unrelated flavour text that is not part of \
                Foo's own entry.\n\n#### Bar\n";
    assert!(!old_boundary_is_self_consistent(text, 1, 3));
}

/// The bug a real (non-identity) sweep run caught: writing the relocated
/// `new_start` next to the STALE `old_end` mixes new-file and old-file
/// coordinates — for `flaw.wrathful_*` this produced `start (7177) exceeds
/// end (7108)`, an inverted range `rules_source_provenance.rs`'s own
/// integrity check rightly refused to load. The fix is to carry the entry's
/// OWN span forward instead.
#[test]
fn manual_boundary_end_preserves_the_old_entrys_own_span() {
    assert_eq!(manual_boundary_end(100, 50, 53), 103);
}

// --- The sweep tool: `#[ignore]`d, writes into the repository --------------
//
// Not a test the gate runs — the sibling, for this direction, of
// `rules_source_provenance.rs::regenerate_source_anchors`. Given the OLD and NEW text of every
// cited book (today, both are the SAME text read from `rules/source/` — see
// `tmp/t1-handover.md` for why a real future resync needs a different `old`
// source, namely the pre-sync commit via `git show`, not yet wired up here),
// it relocates every entry of every catalogue in [`CATALOGUES`] plus the DE
// sidecar, and rewrites `rules/core/*.json` / `rules/i18n/de/
// source_anchors.json` canonically — but only once every entry has resolved;
// otherwise it writes nothing and reports every unresolved entry instead.

/// Dispatches a recorded anchor to [`relocate_heading_anchor`] or
/// [`relocate_row_key_anchor`] depending on its shape (D72.1).
fn relocate_anchor(old_text: &str, new_text: &str, anchor: &str, level: usize) -> Relocation {
    if is_row_key_anchor(anchor) {
        relocate_row_key_anchor(old_text, new_text, anchor)
    } else {
        relocate_heading_anchor(old_text, new_text, anchor, level)
    }
}

/// P8-3's documented, hand-verified exceptions: two EN entries whose OLD
/// heading does not survive in the NEW file under *any* spelling, so
/// `relocate_heading_anchor`'s count-parity check correctly reports
/// Unresolved (`tmp/t1-relocation-report.md`) — "never guess" (D18) means a
/// rename needs a human to read the new file and confirm it's the *same*
/// passage, not a tool guessing from similarity. Confirmed by reading the new
/// file at the passage's own body text, not the stale heading:
/// - `virtue.bard` (`#rard`) — an OCR fix; the heading now reads "Bard"
///   (ArMDE:3547), anchor `bard`.
/// - `spell.sense_the_feet_that_thread_the_earth`
///   (`#sense-the-feet-that-thread-the-earth`) — reworded "Thread" ->
///   "Tread" (same file, line 15411), anchor
///   `sense-the-feet-that-tread-the-earth`.
const ANCHOR_OVERRIDES: &[(&str, &str)] = &[
    ("virtue.bard", "bard"),
    (
        "spell.sense_the_feet_that_thread_the_earth",
        "sense-the-feet-that-tread-the-earth",
    ),
];

/// Resolves an [`ANCHOR_OVERRIDES`] entry directly against the NEW file: finds
/// the heading whose own generated anchor equals `new_anchor` at the expected
/// `level`. Exactly one match resolves; zero or more than one is Unresolved
/// rather than guessed — an override records *where a human already looked*,
/// it does not relax the "never guess" rule for the lookup itself.
fn resolve_override(new_text: &str, new_anchor: &str, level: usize) -> Relocation {
    let candidates: Vec<Heading> = headings(new_text)
        .into_iter()
        .filter(|h| h.anchor == new_anchor && h.level == level)
        .collect();
    match candidates.as_slice() {
        [only] => {
            let all_new_headings = headings(new_text);
            Relocation::Resolved {
                start: only.line,
                end: section_end(new_text, &all_new_headings, only),
            }
        }
        [] => Relocation::Unresolved(format!(
            "override anchor '{new_anchor}' at level {level} names no heading in the new file"
        )),
        _ => Relocation::Unresolved(format!(
            "override anchor '{new_anchor}' at level {level} names more than one heading in the \
             new file"
        )),
    }
}

/// DE sidecar fallback (design-p8-resync.md § "What moves" (b), brief's
/// "20 are DE sidecar entries"): the German files mirror the English ones
/// **line by line** (CLAUDE.md -> "Rules source files" -> "German source
/// files mirror the English ones line-by-line throughout"), so when a DE
/// anchor is unresolved but its English counterpart *did* resolve, the
/// German heading's new line is already known — it's the English entry's own
/// relocated `start`. This reads the German heading actually sitting there
/// and derives its anchor the same way [`headings`] always does, rather than
/// trying to match the STALE German anchor string against new text (which is
/// exactly what just failed). Returns `None` — never a guess — unless that
/// line really is a heading at the expected level, so a German-only
/// restructuring that breaks line-parity surfaces as still-unresolved rather
/// than silently naming the wrong line's heading.
fn de_fallback_anchor(new_de_text: &str, line: usize, expected_level: usize) -> Option<String> {
    let found = headings(new_de_text);
    let heading = found.iter().find(|h| h.line == line)?;
    if heading.level != expected_level {
        return None;
    }
    Some(heading.anchor.clone())
}

#[test]
fn an_override_anchor_resolves_directly_against_the_new_file() {
    let new_text = "# Title\n\n#### Baccalaureus\nOld text.\n\n#### Bard\nNew text.\n\n#### Beadle\n\
                     More.\n";
    assert_eq!(
        resolve_override(new_text, "bard", 4),
        Relocation::Resolved { start: 6, end: 7 }
    );
}

#[test]
fn an_override_anchor_is_unresolved_when_it_names_no_heading() {
    assert!(matches!(
        resolve_override("# Title\n\nNo such heading here.\n", "bard", 4),
        Relocation::Unresolved(_)
    ));
}

#[test]
fn an_override_anchor_is_unresolved_when_the_level_does_not_match() {
    let new_text = "# Title\n\n##### Bard\nText.\n";
    assert!(matches!(
        resolve_override(new_text, "bard", 4),
        Relocation::Unresolved(_)
    ));
}

#[test]
fn de_fallback_anchor_derives_the_anchor_from_the_heading_at_the_english_line() {
    let new_de = "# Titel\n\n#### Alter Titel\nText.\n\n#### Neuer Titel\nText.\n";
    assert_eq!(
        de_fallback_anchor(new_de, 6, 4),
        Some("neuer-titel".to_string())
    );
}

#[test]
fn de_fallback_anchor_is_none_when_that_line_is_not_a_heading() {
    let new_de = "# Titel\n\nKein Titel hier.\nText.\n";
    assert_eq!(de_fallback_anchor(new_de, 3, 4), None);
}

#[test]
fn de_fallback_anchor_is_none_when_the_level_does_not_match() {
    let new_de = "# Titel\n\n##### Zu Tief\nText.\n";
    assert_eq!(de_fallback_anchor(new_de, 3, 4), None);
}

/// `crates/arm-rules` (this crate's manifest dir) -> repo root -> `rules/`.
fn rules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules")
}

/// `crates/arm-rules` (this crate's manifest dir) -> repo root -> `tmp/`,
/// this project's sanctioned home for a run's scratch artifacts
/// (`CLAUDE.md` -> "Scratch and log artifacts live in the project-local
/// `tmp/`").
fn tmp_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp")
}

/// The catalogues this sweep relocates, paired with the heading level their
/// *plain* heading-anchored entries are defined under (ignored for a
/// catalogue whose entries are all row-key anchors) — duplicated from
/// `rules_source_provenance.rs::FULLY_ANCHORED_CATALOGUES`. This is test-side
/// tooling, not shipped code, and that file's own guards are what actually
/// enforce this list against `rules/core/*.json`; a drift here would surface
/// as this sweep mis-leveling or skipping a catalogue, never as silent data
/// corruption, because nothing below writes a catalogue this list omits.
const CATALOGUES: &[(&str, usize)] = &[
    ("virtues_flaws.json", 4),
    ("arts.json", 4),
    ("spell_mastery_abilities.json", 4),
    ("mythic_companion_types.json", 3),
    ("abilities.json", 4),
    ("spells.json", 5),
    ("parameter_catalogues.json", 4),
    ("houses.json", 3),
    ("childhoods.json", 4),
    ("equipment.json", 4),
    ("aging.json", 3),
];

/// Whether [`section_end`]'s generic "next heading at the same level or
/// shallower, stepped back to the last non-blank line" rule can be trusted
/// for one entry — checked by **self-consistency against the OLD file**:
/// recompute the end from the OLD heading's own position and compare it to
/// the entry's recorded `end`. If they disagree, the formula has a
/// demonstrated blind spot for this entry's structure (it is one of
/// `rules_source_provenance.rs::OWN_BLOCKQUOTED_SIDEBARS`'s owned sidebars,
/// one of its `SUBDIVIDED_ITEMS`, or — discovered by this very sweep's first
/// identity run, see `tmp/t1-handover.md` — a case neither list had caught:
/// `flaw.wrathful_*` undershoots into an unrelated blockquoted flavour-text
/// paragraph the Chapter 5 break prints, while `art.vim` *overshoots* a
/// trailing "Pronounced '...'" line the original citation simply omitted).
/// Whatever the reason, a structural quirk the OLD file already has will
/// almost certainly still be there in the NEW one, so this check is what
/// decides whether to trust the formula for the NEW file too — not a
/// hand-maintained id list, which the first identity run proved incomplete
/// (17 more entries disagreed than the two guard-exemption lists named).
///
/// Only meaningful for a plain heading anchor: a row-key citation is always
/// single-line (`start == end` by construction, see
/// [`relocate_row_key_anchor`]), so it carries no such boundary question.
fn old_boundary_is_self_consistent(old_text: &str, old_start: usize, recorded_end: usize) -> bool {
    let old_headings = headings(old_text);
    let Some(opening) = old_headings.iter().find(|h| h.line == old_start) else {
        // The recorded start isn't even a heading in the OLD file — nothing
        // to self-check against; treat as inconsistent (never guess).
        return false;
    };
    section_end(old_text, &old_headings, opening) == recorded_end
}

/// The `end` to write for a "manual boundary review" entry — one whose
/// generic boundary formula is already known (via
/// [`old_boundary_is_self_consistent`]) not to reproduce this entry's own
/// recorded end against the OLD file, so the formula's *new*-file recompute
/// cannot be trusted either and is never written. Writing the new `start`
/// alongside the stale OLD `end` (an earlier version of this sweep's own
/// bug) mixes two coordinate systems — the kind of structural quirk that
/// made the formula untrustworthy here (a trailing pronunciation line, an
/// unrelated blockquoted paragraph, …) is something "the OLD file already
/// has will almost certainly still be there in the NEW one" (this file's own
/// module doc comment), so the entry's **own span** (`old_end - old_start`)
/// is the one thing still safe to carry forward — exactly the span-length
/// strategy the module doc comment rejects as the *general* rule, but
/// correct precisely for the cases already flagged as exceptions to the
/// general (boundary-recompute) rule. Still written to the non-blocking
/// "manual boundary review" list for a human to confirm, never silently
/// trusted as final.
fn manual_boundary_end(new_start: usize, old_start: usize, old_end: usize) -> usize {
    new_start + (old_end - old_start)
}

/// Hand-verified exceptions to [`manual_boundary_end`]'s own general rule —
/// read directly against both files (`tmp/p8-old-source/en/...` and
/// `rules/source/en/...`) during the P8-3 real-sync run that first wrote
/// these 34 entries. Old-span preservation assumes the OLD citation's own
/// end was already correct and only the generic formula is untrustworthy;
/// true for 31 of the 34, but these 3 ids are the ones where the OLD
/// citation was *itself* short — the same "missing its own last line(s)"
/// defect `tmp/t1-handover.md` already found for `art.vim` alone (a
/// pronunciation note, ArMDE(old):8981 vs :8983), and which this slice found
/// twice more by spot-checking every one of the 34 against real text rather
/// than trusting the general rule uniformly:
/// - `art.vim`: old end (8981) stops short of its own "Pronounced '...'"
///   line (8983/new 9055).
/// - `profession.marshal`: old end (4453) stops short of its own closing
///   paragraph ("A marshal may overseen other specialists...", old 4455/new
///   4526).
/// - `magical_focus.certamen`: old end (4281) stops short of the Hermetic
///   Houses Summary table's last two rows (Tytalus/Verditius, old
///   4282-4283/new 2350-2351).
///
/// For exactly these three, the naive NEW-file recompute (`new_end`) is the
/// correct value instead — confirmed by reading the real new-file text at
/// that position, not assumed. A future resync's own 34-row list will differ
/// and needs the same by-hand check; this list is not expected to carry
/// forward unchanged.
const MANUAL_BOUNDARY_RECOMPUTE_OVERRIDES: &[&str] =
    &["art.vim", "profession.marshal", "magical_focus.certamen"];

/// Picks which candidate end a manual-boundary-review entry actually writes:
/// the recomputed NEW-file value for the three documented exceptions above,
/// otherwise the entry's own OLD span carried forward
/// ([`manual_boundary_end`]).
fn resolved_manual_boundary_end(
    ids: &[String],
    new_start: usize,
    new_end: usize,
    old_start: usize,
    old_end: usize,
) -> usize {
    if ids
        .iter()
        .any(|id| MANUAL_BOUNDARY_RECOMPUTE_OVERRIDES.contains(&id.as_str()))
    {
        new_end
    } else {
        manual_boundary_end(new_start, old_start, old_end)
    }
}

#[test]
fn resolved_manual_boundary_end_uses_the_recompute_for_a_documented_override() {
    assert_eq!(
        resolved_manual_boundary_end(&["art.vim".to_string()], 9049, 9055, 8977, 8981),
        9055
    );
}

#[test]
fn resolved_manual_boundary_end_uses_the_old_span_otherwise() {
    assert_eq!(
        resolved_manual_boundary_end(
            &["flaw.form_monstrosity".to_string()],
            6233,
            6239,
            6162,
            6184
        ),
        6255
    );
}

/// One `source` block found in a `rules/core/*.json` catalogue, plus its
/// sibling `id` — a trimmed duplicate of
/// `rules_source_provenance.rs::FoundSourceRef`, keeping only what this sweep
/// needs.
struct Entry {
    id: String,
    anchor: String,
    file: String,
}

fn collect_entries(value: &serde_json::Value, out: &mut Vec<Entry>) {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            if let Some(source) = map.get("source")
                && let Some(id) = map.get("id").and_then(Value::as_str)
                && let Some(anchor) = source.get("anchor").and_then(Value::as_str)
                && let Some(file) = source.get("file").and_then(Value::as_str)
            {
                out.push(Entry {
                    id: id.to_string(),
                    anchor: anchor.to_string(),
                    file: file.to_string(),
                });
            }
            for nested in map.values() {
                collect_entries(nested, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_entries(item, out);
            }
        }
        _ => {}
    }
}

/// One line of the sweep's report: an entry that needs a human's attention,
/// either because it did not resolve at all or because it resolved but
/// [`old_boundary_is_self_consistent`] could not validate its boundary
/// formula.
struct ReportRow {
    catalogue: String,
    id: String,
    anchor: String,
    reason: String,
}

fn render_report(
    outcome: &str,
    unresolved: &[ReportRow],
    manual_boundary: &[ReportRow],
    boundary_mismatches: &[String],
    resolved_count: usize,
    de_resolved_count: usize,
) -> String {
    let mut out = format!(
        "# P8-1 T1 relocation report\n\n**Outcome:** {outcome}\n\n{} EN entries resolved \
         (lines unchanged or relocated), {} DE sidecar entries confirmed to still resolve.\n",
        resolved_count, de_resolved_count
    );
    out.push_str(&format!(
        "\n## Unresolved ({}) — blocks every write until fixed\n\n",
        unresolved.len()
    ));
    if unresolved.is_empty() {
        out.push_str("(none)\n");
    } else {
        out.push_str("| Catalogue | id | anchor | reason |\n|---|---|---|---|\n");
        for row in unresolved {
            out.push_str(&format!(
                "| {} | {} | #{} | {} |\n",
                row.catalogue, row.id, row.anchor, row.reason
            ));
        }
    }
    out.push_str(&format!(
        "\n## Manual boundary review ({}) — non-blocking, `end` left unchanged\n\n",
        manual_boundary.len()
    ));
    if manual_boundary.is_empty() {
        out.push_str("(none)\n");
    } else {
        out.push_str("| Catalogue | id | anchor | note |\n|---|---|---|---|\n");
        for row in manual_boundary {
            out.push_str(&format!(
                "| {} | {} | #{} | {} |\n",
                row.catalogue, row.id, row.anchor, row.reason
            ));
        }
    }
    out.push_str(&format!(
        "\n## Relocated entries ({}) — trusted, already-applied offset shifts (the recomputed \
         end differs from the pre-sync end only because content moved earlier in the book; \
         `old_boundary_is_self_consistent` passed, so this is what was written, not a review \
         item — contrast the genuine review rows under \"Manual boundary review\" above)\n\n",
        boundary_mismatches.len()
    ));
    if boundary_mismatches.is_empty() {
        out.push_str("(none)\n");
    } else {
        for line in boundary_mismatches {
            out.push_str(&format!("- {line}\n"));
        }
    }
    out
}

/// P8-3 triage (`tmp/t1-relocation-report.md`'s "Fresh boundary disagreements
/// (1209)"): sampled against the actual pre-/post-sync files
/// (`flaw.a_deal_with_the_devil` #a-deal-with-the-devil 5905-5907 -> 5976-5978,
/// `art.creo` #creo-cr-i-create 8863-8935 confirmed similarly), every one of
/// these 1209 rows is a **trusted, already-applied** relocation
/// (`old_boundary_is_self_consistent` passed, so the recomputed end is exactly
/// what got written) that differs from the pre-sync end for the ordinary
/// reason a resync exists at all: upstream inserted or removed lines earlier
/// in the book, so the *section* moved — not because the entry's own content
/// changed shape or needs a human look. Calling that a "disagreement ... worth
/// checking" reads as a review item sitting alongside the genuine
/// `manual_boundary` rows above it, which is backwards: those rows are the
/// ones the self-check could not trust; these are the ones it could.
#[test]
fn render_report_labels_a_relocated_entry_as_routine_not_a_disagreement() {
    let report = render_report(
        "every entry resolved — 0 catalogue file(s) actually changed and were written",
        &[],
        &[],
        &["virtues_flaws.json: \"flaw.a_deal_with_the_devil\" (#a-deal-with-the-devil) relocated \
           end=5978, was end=5907"
            .to_string()],
        1,
        1,
    );
    assert!(
        report.contains("## Relocated entries (1)"),
        "a trusted, already-applied relocation should be reported under a neutral 'relocated' \
         heading, not a 'disagreement' heading implying it still needs review — report was:\n\
         {report}"
    );
    assert!(
        !report.to_lowercase().contains("disagreement"),
        "an offset shift from inserted/removed lines earlier in the book is the ordinary outcome \
         of a resync, not a disagreement — report was:\n{report}"
    );
}

/// Applies the DE sidecar's [`de_fallback_anchor`] results to the sidecar's
/// own text, one entry per line (`rules/i18n/de/source_anchors.json`'s
/// canonical shape: `"<id>": { "anchor": "<slug>", "file": "<book>" },`,
/// one id per line, sorted). Rewritten as a targeted substring replace on
/// just the named id's own line — never a round-trip through
/// `serde_json::to_string_pretty` over the whole map, which would reflow
/// every entry and bury the real diff (`design-p8-resync.md`'s own Risks:
/// "never round-trip the whole JSON document through `serde_json`").
/// Returns the rewritten text plus whether anything actually changed, so the
/// caller only touches the file on disk when `changed` is true.
fn rewrite_sidecar_anchors(
    sidecar_text: &str,
    updates: &BTreeMap<String, String>,
) -> (String, bool) {
    let mut changed = false;
    let mut out_lines = Vec::new();
    for line in sidecar_text.lines() {
        let mut replaced_line = line.to_string();
        for (id, new_anchor) in updates {
            let needle = format!("\"{id}\": {{ \"anchor\": \"");
            let Some(pos) = line.find(&needle) else {
                continue;
            };
            let start = pos + needle.len();
            let Some(end_rel) = line[start..].find('"') else {
                continue;
            };
            let end = start + end_rel;
            replaced_line = format!("{}{}{}", &line[..start], new_anchor, &line[end..]);
            changed = true;
            break;
        }
        out_lines.push(replaced_line);
    }
    (format!("{}\n", out_lines.join("\n")), changed)
}

#[test]
fn rewrite_sidecar_anchors_replaces_only_the_named_ids_anchor_value() {
    let sidecar = "{\n  \"flaw.busybody\": { \"anchor\": \"klatschbase-1\", \"file\": \"X.md\" },\n  \
                   \"virtue.gossip\": { \"anchor\": \"klatschbase\", \"file\": \"X.md\" }\n}\n";
    let mut updates = BTreeMap::new();
    updates.insert("flaw.busybody".to_string(), "neuer-slug".to_string());
    let (out, changed) = rewrite_sidecar_anchors(sidecar, &updates);
    assert!(changed);
    assert!(out.contains("\"flaw.busybody\": { \"anchor\": \"neuer-slug\", \"file\": \"X.md\" }"));
    assert!(out.contains("\"virtue.gossip\": { \"anchor\": \"klatschbase\", \"file\": \"X.md\" }"));
}

#[test]
fn rewrite_sidecar_anchors_reports_unchanged_when_no_id_matches() {
    let sidecar =
        "{\n  \"virtue.gossip\": { \"anchor\": \"klatschbase\", \"file\": \"X.md\" }\n}\n";
    let updates = BTreeMap::new();
    let (out, changed) = rewrite_sidecar_anchors(sidecar, &updates);
    assert!(!changed);
    assert_eq!(out, sidecar);
}

/// Where the sweep reads the PRE-sync books from. Before the copy-in, today's
/// `rules/source/` is saved to `tmp/p8-old-source/` (same `en/`, `de/`
/// layout); once that copy exists, it is the old side and `rules/source/` is
/// the new side. Without it, both sides are `rules/source/` (identity run).
fn old_source_root(source_root: &Path, saved_copy: &Path, saved_copy_exists: bool) -> PathBuf {
    if saved_copy_exists {
        return saved_copy.to_path_buf();
    }
    source_root.to_path_buf()
}

#[test]
fn the_old_side_is_the_saved_pre_sync_copy_once_it_exists() {
    let source = Path::new("rules/source");
    let saved = Path::new("tmp/p8-old-source");
    assert_eq!(old_source_root(source, saved, true), saved);
    assert_eq!(old_source_root(source, saved, false), source);
}

#[test]
#[ignore = "writes into rules/; run deliberately — see tmp/t1-handover.md"]
fn regenerate_source_relocations() {
    let new_root = rules_dir().join("source");
    let saved_copy = tmp_dir().join("p8-old-source");
    let old_root = old_source_root(&new_root, &saved_copy, saved_copy.is_dir());
    let mut book_text_cache: BTreeMap<String, String> = BTreeMap::new();
    let mut book_text = |root: &Path, lang: &str, file: &str| -> String {
        let path = root.join(lang).join(file);
        book_text_cache
            .entry(path.display().to_string())
            .or_insert_with(|| fs::read_to_string(&path).unwrap_or_default())
            .clone()
    };

    let mut unresolved: Vec<ReportRow> = Vec::new();
    let mut manual_boundary: Vec<ReportRow> = Vec::new();
    let mut boundary_mismatches: Vec<String> = Vec::new();
    let mut resolved_count = 0usize;
    let mut id_levels: BTreeMap<String, usize> = BTreeMap::new();
    let mut rewritten_catalogues: Vec<(&'static str, PathBuf, String, bool)> = Vec::new();
    // Every id's new EN start line, recorded whenever its EN relocation
    // resolves (trusted formula or manual-boundary-review alike — either way
    // the *start* is known) — the DE sidecar fallback's own input, per
    // CLAUDE.md's "German source files mirror the English ones line-by-line".
    let mut new_start_by_id: BTreeMap<String, usize> = BTreeMap::new();

    for &(catalogue_name, level) in CATALOGUES {
        let catalogue_path = rules_dir().join("core").join(catalogue_name);
        let text = fs::read_to_string(&catalogue_path)
            .unwrap_or_else(|e| panic!("{catalogue_name} is readable: {e}"));
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{catalogue_name} is valid JSON: {e}"));
        let mut entries = Vec::new();
        collect_entries(&parsed, &mut entries);
        // Keyed by (file, anchor) rather than relied on in document order:
        // `serde_json::Value::Object` (no `preserve_order` feature in this
        // workspace) iterates its keys in sorted order, so the parsed
        // `entries` list's order is not guaranteed to match the textual
        // line-by-line scan below.
        //
        // The *value* is a `Vec`, not a single id, because an anchor is
        // unique per *heading*, not per *entry*: a "split Minor/Major" item
        // prints ONE entry in the book at either magnitude
        // (`project memory — "A split 'Minor or Major' entry..."`, N11), and
        // `virtue.potent_magic_major`/`.minor` are documented
        // (`OWN_BLOCKQUOTED_SIDEBARS`) to cite the byte-identical
        // `{anchor, file, lines}` block for the same reason — so two
        // *different* ids can legitimately share one (file, anchor) key.
        // Collapsing that to a single id would silently mislabel one of them
        // in the report below; keeping every id that shares the key makes the
        // collision visible instead; the relocation *values* themselves do
        // not depend on which id is asking, so this never affects what gets
        // written.
        let mut id_by_file_anchor: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
        for entry in &entries {
            id_by_file_anchor
                .entry((entry.file.clone(), entry.anchor.clone()))
                .or_default()
                .push(entry.id.clone());
        }
        for entry in &entries {
            id_levels.insert(entry.id.clone(), level);
        }

        let mut rewritten_lines = Vec::new();
        let mut file_changed = false;
        let mut source_lines = text.lines().peekable();
        while let Some(first) = source_lines.next() {
            let Some((prefix, rest)) = first.split_once("\"source\": {") else {
                rewritten_lines.push(first.to_string());
                continue;
            };
            let mut block_text = rest.to_string();
            while !block_text.contains('}') {
                block_text.push(' ');
                block_text.push_str(source_lines.next().expect("the block closes").trim());
            }
            let (body, suffix) = block_text.split_once('}').expect("the block closes");
            let block: serde_json::Value = serde_json::from_str(&format!("{{{body}}}"))
                .unwrap_or_else(|e| panic!("{catalogue_name}: source block parses: {e}"));
            let file = block["file"].as_str().expect("source.file is a string");
            let anchor = block["anchor"]
                .as_str()
                .expect("source.anchor is mandatory (D30.1)");
            let start = block["lines"][0].as_u64().expect("lines[0] is a number") as usize;
            let end = block["lines"][1].as_u64().expect("lines[1] is a number") as usize;

            let ids = id_by_file_anchor
                .get(&(file.to_string(), anchor.to_string()))
                .cloned()
                .unwrap_or_else(|| {
                    vec![format!(
                        "<unmatched: {catalogue_name} cites {file}#{anchor}>"
                    )]
                });
            // Joined for display only — see `id_by_file_anchor`'s doc comment
            // on why more than one id can legitimately share one key.
            let id = ids.join(" / ");

            let old_text = book_text(&old_root, "en", file);
            let new_text = book_text(&new_root, "en", file);
            // `ANCHOR_OVERRIDES`: a hand-verified exception whose OLD heading
            // was reworded/removed, so plain anchor relocation cannot (and
            // must not) resolve it — the override records where a human
            // already confirmed the same passage sits in the NEW file.
            let override_new_anchor = ids
                .iter()
                .find_map(|id| ANCHOR_OVERRIDES.iter().find(|(oid, _)| oid == id))
                .map(|(_, new_anchor)| *new_anchor);
            let relocation = match override_new_anchor {
                Some(new_anchor) => resolve_override(&new_text, new_anchor, level),
                None => relocate_anchor(&old_text, &new_text, anchor, level),
            };
            let anchor_to_write = override_new_anchor.unwrap_or(anchor);

            match relocation {
                Relocation::Unresolved(reason) => {
                    unresolved.push(ReportRow {
                        catalogue: catalogue_name.to_string(),
                        id,
                        anchor: anchor.to_string(),
                        reason,
                    });
                    rewritten_lines.push(first.to_string());
                }
                Relocation::Resolved {
                    start: new_start,
                    end: new_end,
                } => {
                    for entry_id in &ids {
                        new_start_by_id.insert(entry_id.clone(), new_start);
                    }
                    // Row-key citations are always single-line by
                    // construction (`relocate_row_key_anchor`) — no boundary
                    // question to self-check.
                    let trust_formula_end = is_row_key_anchor(anchor)
                        || old_boundary_is_self_consistent(&old_text, start, end);

                    if trust_formula_end {
                        resolved_count += 1;
                        if new_end != end {
                            boundary_mismatches.push(format!(
                                "{catalogue_name}: \"{id}\" (#{anchor}) relocated end={new_end}, \
                                 was end={end}"
                            ));
                        }
                        if new_start != start || new_end != end || anchor_to_write != anchor {
                            file_changed = true;
                        }
                        rewritten_lines.push(format!(
                            "{prefix}\"source\": {{ \"anchor\": {}, \"file\": {}, \"lines\": \
                             [{new_start}, {new_end}] }}{suffix}",
                            serde_json::to_string(anchor_to_write).expect("a string serializes"),
                            serde_json::to_string(file).expect("a string serializes"),
                        ));
                    } else {
                        let carried_end =
                            resolved_manual_boundary_end(&ids, new_start, new_end, start, end);
                        let used_override = ids
                            .iter()
                            .any(|i| MANUAL_BOUNDARY_RECOMPUTE_OVERRIDES.contains(&i.as_str()));
                        manual_boundary.push(ReportRow {
                            catalogue: catalogue_name.to_string(),
                            id: id.clone(),
                            anchor: anchor.to_string(),
                            reason: if used_override {
                                format!(
                                    "the boundary formula does not reproduce this entry's own \
                                     recorded end against the OLD file (self-check failed); this \
                                     id is a documented MANUAL_BOUNDARY_RECOMPUTE_OVERRIDES \
                                     exception (the OLD citation was itself short), so the NEW-file \
                                     recompute end={carried_end} was written instead of the old \
                                     span (was end={end}) — confirm by hand"
                                )
                            } else {
                                format!(
                                    "the boundary formula does not reproduce this entry's own \
                                     recorded end against the OLD file (self-check failed) — not \
                                     trusted for the NEW file either; naive recompute would give \
                                     end={new_end}; the entry's own span was carried forward \
                                     instead, giving end={carried_end} (was end={end}) — confirm by \
                                     hand"
                                )
                            },
                        });
                        if new_start != start || carried_end != end || anchor_to_write != anchor {
                            file_changed = true;
                        }
                        rewritten_lines.push(format!(
                            "{prefix}\"source\": {{ \"anchor\": {}, \"file\": {}, \"lines\": \
                             [{new_start}, {carried_end}] }}{suffix}",
                            serde_json::to_string(anchor_to_write).expect("a string serializes"),
                            serde_json::to_string(file).expect("a string serializes"),
                        ));
                    }
                }
            }
        }

        rewritten_catalogues.push((
            catalogue_name,
            catalogue_path,
            format!("{}\n", rewritten_lines.join("\n")),
            file_changed,
        ));
    }

    // The DE sidecar: no `lines` field to relocate (D30's own sidecar design
    // — see this file's module doc comment and `ANCHOR_SIDECAR`'s original
    // rationale in `rules_source_provenance.rs`), so there is nothing to
    // rewrite on a successful resolution; the anchor itself cannot change
    // without the lookup failing (`relocate_heading_line` only succeeds when
    // the given anchor string matches the new file's own generated anchor
    // exactly). This pass therefore only confirms every DE anchor still
    // resolves against the DE file, surfacing a reworded German heading
    // (D18/D31's own worked examples) as an unresolved manual-review item
    // rather than silently accepting stale text.
    let sidecar_path = rules_dir().join("i18n/de").join("source_anchors.json");
    let sidecar_text = fs::read_to_string(&sidecar_path)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", sidecar_path.display()));
    let sidecar: BTreeMap<String, serde_json::Value> = serde_json::from_str(&sidecar_text)
        .unwrap_or_else(|e| panic!("DE sidecar is valid JSON: {e}"));
    let mut de_resolved_count = 0usize;
    let mut sidecar_updates: BTreeMap<String, String> = BTreeMap::new();
    for (id, row) in &sidecar {
        let Some(&level) = id_levels.get(id) else {
            unresolved.push(ReportRow {
                catalogue: "source_anchors.json".to_string(),
                id: id.clone(),
                anchor: String::new(),
                reason: "no EN catalogue entry (and thus no known heading level) was found for \
                         this id in any catalogue in CATALOGUES"
                    .to_string(),
            });
            continue;
        };
        let file = row["file"]
            .as_str()
            .unwrap_or_else(|| panic!("de/{id}: source_anchors.json file is a string"));
        let anchor = row["anchor"]
            .as_str()
            .unwrap_or_else(|| panic!("de/{id}: source_anchors.json anchor is a string"));
        let old_text = book_text(&old_root, "de", file);
        let new_text = book_text(&new_root, "de", file);
        match relocate_anchor(&old_text, &new_text, anchor, level) {
            Relocation::Resolved { .. } => de_resolved_count += 1,
            Relocation::Unresolved(reason) => {
                // German mirrors English line-by-line (CLAUDE.md), so when
                // the EN counterpart resolved, its new start line is also
                // where the German heading now sits — read it directly
                // rather than retrying the stale anchor string.
                let fallback = new_start_by_id
                    .get(id)
                    .and_then(|&new_start| de_fallback_anchor(&new_text, new_start, level));
                match fallback {
                    Some(new_anchor) => {
                        de_resolved_count += 1;
                        sidecar_updates.insert(id.clone(), new_anchor);
                    }
                    None => unresolved.push(ReportRow {
                        catalogue: "source_anchors.json".to_string(),
                        id: id.clone(),
                        anchor: anchor.to_string(),
                        reason,
                    }),
                }
            }
        }
    }

    let outcome = if unresolved.is_empty() {
        let mut catalogues_written = 0usize;
        for (name, path, new_text, changed) in &rewritten_catalogues {
            if *changed {
                fs::write(path, new_text).unwrap_or_else(|e| panic!("{name} is writable: {e}"));
                catalogues_written += 1;
            }
        }
        let (sidecar_rewritten, sidecar_changed) =
            rewrite_sidecar_anchors(&sidecar_text, &sidecar_updates);
        let mut outcome_text = format!(
            "every entry resolved — {catalogues_written} catalogue file(s) actually changed and \
             were written (the rest were byte-identical and left untouched)"
        );
        if sidecar_changed {
            fs::write(&sidecar_path, &sidecar_rewritten)
                .unwrap_or_else(|e| panic!("{} is writable: {e}", sidecar_path.display()));
            outcome_text.push_str(&format!(
                "; DE sidecar rewrote {} fallback-derived anchor(s)",
                sidecar_updates.len()
            ));
        }
        outcome_text
    } else {
        format!(
            "{} entries unresolved — NO rules/ files were written; see the table below",
            unresolved.len()
        )
    };

    let report = render_report(
        &outcome,
        &unresolved,
        &manual_boundary,
        &boundary_mismatches,
        resolved_count,
        de_resolved_count,
    );
    fs::write(tmp_dir().join("t1-relocation-report.md"), &report).expect("tmp/ is writable");

    println!(
        "resolved={resolved_count} manual_boundary_review={} de_resolved={de_resolved_count} \
         unresolved={} boundary_mismatches={}",
        manual_boundary.len(),
        unresolved.len(),
        boundary_mismatches.len()
    );

    assert!(
        unresolved.is_empty(),
        "{} entries unresolved; see tmp/t1-relocation-report.md — no rules/ files were written",
        unresolved.len()
    );
}
