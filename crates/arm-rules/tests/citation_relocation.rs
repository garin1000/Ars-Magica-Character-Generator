//! P8-1/T2: text-search relocation for bare rulebook citations — the tooling
//! `docs/vf-audit/design-p8-resync.md` § "What moves" (c)/(d)/(g) and § "P8-1"
//! describes for the class of citation that carries **no anchor**: `ArMDE:NNNN`
//! comments in `crates/*/src`/`crates/*/tests`/`ui/src`, `RULES.md`'s citations,
//! and the line-pinning tests enumerated by `book_templates.rs` and friends.
//! D85 (`docs/vf-audit/decisions.md`) answered the two open questions this
//! tool depends on: **Q5** picks text-search relocation over building a full
//! diff/alignment table, and **Q6** says an OCR-class scan-error fix upstream
//! should refresh the stored text rather than be preserved as-is.
//!
//! **Spec** (from the design): given an OLD file, a NEW file, and a cited
//! line or inclusive range in the OLD file, produce `(old, new, confidence)`,
//! confidence being `Exact` / `Fuzzy` / `Unresolved`. The tool must **never
//! guess**: an ambiguous or genuinely changed passage resolves to
//! `Unresolved`, not a best-effort pick. [`relocate_line`] is the single-line
//! primitive; [`relocate_range`] relocates a range by relocating its start and
//! then checking the end still sits the same span away (never assuming it
//! does); [`relocate_citation`] is the thin wrapper that returns the
//! `(old, new, confidence)` triple shape the brief specifies.
//!
//! This mirrors, at the text-search level, the same "relocation self-verifies
//! because it's the same text" property `rulebook_citations.rs`'s P8-1/T3
//! excerpt-snapshot machinery (`citation_key`, `normalize_excerpt`,
//! `BookLinesCache`) already proves for *checking* a citation against a
//! snapshot — this file is the inverse direction, *finding* where a citation's
//! old line went in a resynced file. It deliberately does not share code with
//! that file: this is throwaway migration tooling for one resync event, kept
//! in its own test file rather than touching the engine crate (`arm-rules`
//! must have zero tauri/IO/migration-tool dependencies — "Engine purity").
//!
//! **Is an 80-character first-line excerpt enough to disambiguate a repeated
//! line (e.g. `"*Minor, General*<br>"`, which recurs once per Virtue/Flaw
//! entry taken at that magnitude)? No — decided here, acted on now.** A
//! citation whose line's own text recurs verbatim elsewhere in the same book
//! cannot be told apart from its siblings by that text alone, no matter how
//! generous the character cap; disambiguation needs something *besides* the
//! cited line itself. [`RelocationContext::next_line`] models the cheapest
//! such signal: the line immediately following the cited one in the OLD file
//! — already free-standing text, costs nothing to capture, and distinguishes
//! "Alpha's minor-magnitude line" from "Beta's minor-magnitude line" the
//! moment their following description text differs (see
//! `relocate_line_disambiguates_a_duplicate_line_using_captured_next_line_context`
//! below). **This is the reason P8-1 runs before P8-2's copy-in, not after**:
//! the context must be captured against *today's* file, before any line moves,
//! the same reasoning `design-p8-resync.md` gives for snapshotting T3's
//! excerpts first. Phase 2, wiring this into the live corpus, must capture a
//! `next_line` alongside every stored excerpt for exactly this reason — a
//! citation discovered to collide only after the sync has already lost the
//! context that would have resolved it.
//!
//! **Phase 2: the exact/fuzzy/context search is implemented below** (this
//! file's own unit level — `relocate_line`/`relocate_range`/
//! `relocate_citation` over an in-memory `old_lines`/`new_lines` pair). What
//! is **not** done yet, and stays open for P8-4: wiring this over the real
//! P8-0 census (reading `old_lines`/`new_lines` from `git show <pre-sync-sha>`
//! and the resynced file, feeding every bare citation through it, and sorting
//! the results into the exact/fuzzy/unresolved buckets P8-4 consumes).

/// How confidently a citation's old line/range was relocated into the new
/// file. Never upgraded by a best-effort guess — see this file's module doc
/// comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confidence {
    /// The old line's (trimmed) text matches exactly one line in the new
    /// file (directly, or after context disambiguated several identical
    /// candidates).
    Exact,
    /// No exact match, but exactly one line in the new file is within a
    /// single-character edit of the old line's text (the OCR-fix class:
    /// `"Subernatural"` → `"Supernatural"`, `"guarter"` → `"quarter"`).
    Fuzzy,
    /// Zero candidates (the line sits inside a rewritten hunk), or more than
    /// one equally-plausible candidate that context did not disambiguate.
    /// Always means "a human must look at this", never "best guess applied".
    Unresolved,
}

/// Disambiguating context captured **before** a resync, alongside a citation's
/// excerpt — see this file's module doc comment for why this must exist
/// ahead of time rather than be reconstructed after the fact.
#[derive(Debug, Clone, Default)]
struct RelocationContext {
    /// The (trimmed) text of the line immediately following the cited line
    /// in the OLD file, if any. Used only to narrow multiple candidate
    /// matches down to one; a citation with a unique line never needs it.
    next_line: Option<String>,
    /// The (trimmed) text of the nearest Markdown heading line preceding the
    /// cited line in the OLD file, if any. A second, independent tiebreaker
    /// for when `next_line` alone still leaves several candidates tied — the
    /// real corpus has duplicate passages (e.g. repeated statblock fields)
    /// whose *following* line is itself duplicated across occurrences, so
    /// `next_line` cannot narrow them, but their enclosing section heading
    /// differs. See
    /// `relocate_line_disambiguates_using_heading_when_next_line_also_collides`.
    heading: Option<String>,
}

/// Mirrors `rulebook_citations.rs`'s own nearest-preceding-heading capture —
/// deliberately not shared code (see this file's module doc comment). Scans
/// `lines[..idx]` backward for the nearest line whose trimmed text starts
/// with `#`, trimmed; `""` if none precedes `idx`.
fn nearest_preceding_heading(lines: &[&str], idx: usize) -> String {
    lines[..idx.min(lines.len())]
        .iter()
        .rev()
        .find(|line| line.trim_start().starts_with('#'))
        .map(|line| line.trim().to_string())
        .unwrap_or_default()
}

/// The result of relocating one cited **line** (`old_line`, 1-based) from the
/// OLD file into the NEW file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LineRelocation {
    old_line: usize,
    /// The resolved 1-based line in the new file. `None` exactly when
    /// `confidence` is [`Confidence::Unresolved`].
    new_line: Option<usize>,
    confidence: Confidence,
}

/// The result of relocating one cited **range** (`old_start..=old_end`,
/// 1-based inclusive) from the OLD file into the NEW file. The end is never
/// assumed to keep the same span as the start — it is checked, and a broken
/// span is reported rather than silently kept.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RangeRelocation {
    old_start: usize,
    old_end: usize,
    /// `None` when the start itself could not be relocated.
    new_start: Option<usize>,
    /// `None` when the start relocated but the end's text no longer sits the
    /// same span away in the new file (a broken span is always a report, not
    /// a guess — see
    /// `relocate_range_reports_rather_than_guesses_when_the_span_no_longer_matches`).
    new_end: Option<usize>,
    confidence: Confidence,
}

/// One relocated citation in the `(old, new, confidence)` shape
/// `design-p8-resync.md` § "P8-1" specifies: `old`/`new` as inclusive 1-based
/// `(start, end)` ranges (a single-line citation has `start == end`), `new`
/// absent when `confidence` is [`Confidence::Unresolved`].
#[derive(Debug, Clone, PartialEq, Eq)]
struct CitationRelocation {
    old: (usize, usize),
    new: Option<(usize, usize)>,
    confidence: Confidence,
}

/// True when `a` and `b` are different but sit at most one character-edit
/// (a single substitution, insertion, or deletion) apart — the OCR-fix class
/// (`"Subernatural"` → `"Supernatural"`, `"guarter"` → `"quarter"`).
/// Identical strings return `false`: that is the exact bucket's job, not
/// this one's, and keeping them disjoint is what lets a caller try exact
/// first and fuzzy only when exact found nothing.
fn within_one_edit(a: &str, b: &str) -> bool {
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
        // Same length: the only single edit possible is one substitution.
        return short
            .iter()
            .zip(long.iter())
            .filter(|(x, y)| x != y)
            .count()
            == 1;
    }
    // Lengths differ by exactly one: walk both, allowing one skip in `long`.
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

/// Narrows a list of same-ranked candidate line indices (0-based, into
/// `new_lines`) down to a single resolved one, or reports `None` when it
/// cannot: zero candidates has nothing to resolve; exactly one needs no
/// disambiguation; more than one tries `context.next_line` against each
/// candidate's own following line first, then — if that still leaves several
/// tied (the real corpus has passages whose cited line *and* its follower are
/// both duplicated) — `context.heading` against each candidate's own nearest
/// preceding heading. Succeeds only when a stage narrows to exactly one
/// survivor; never guesses among several.
fn resolve_candidates(
    candidates: &[usize],
    new_lines: &[&str],
    context: &RelocationContext,
) -> Option<usize> {
    let mut current: Vec<usize> = candidates.to_vec();
    if current.len() <= 1 {
        return current.first().copied();
    }

    if let Some(next_line_ctx) = context.next_line.as_deref() {
        let next_line_ctx = next_line_ctx.trim();
        let narrowed: Vec<usize> = current
            .iter()
            .copied()
            .filter(|&idx| {
                new_lines
                    .get(idx + 1)
                    .is_some_and(|l| l.trim() == next_line_ctx)
            })
            .collect();
        match narrowed.as_slice() {
            [only] => return Some(*only),
            [] => {}
            _ => current = narrowed,
        }
    }
    if current.len() <= 1 {
        return current.first().copied();
    }

    let heading_ctx = context.heading.as_deref()?.trim();
    let narrowed: Vec<usize> = current
        .iter()
        .copied()
        .filter(|&idx| nearest_preceding_heading(new_lines, idx) == heading_ctx)
        .collect();
    match narrowed.as_slice() {
        [only] => Some(*only),
        _ => None,
    }
}

/// Relocates one cited line from `old_lines` into `new_lines`: finds every
/// line in `new_lines` whose trimmed text exactly equals the old line's; one
/// hit is `Exact`, several try [`resolve_candidates`]' context narrowing
/// (`Exact` if it succeeds, `Unresolved` if not — never fall through to a
/// fuzzy guess once the real text is known to recur). No exact hit at all
/// falls back to [`within_one_edit`] candidates, scored the same way but as
/// `Fuzzy`. No candidates of either kind is `Unresolved`.
fn relocate_line(
    old_lines: &[&str],
    new_lines: &[&str],
    old_line: usize,
    context: &RelocationContext,
) -> LineRelocation {
    let target = old_lines.get(old_line - 1).map(|l| l.trim()).unwrap_or("");

    let exact: Vec<usize> = new_lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim() == target)
        .map(|(i, _)| i)
        .collect();
    if !exact.is_empty() {
        return match resolve_candidates(&exact, new_lines, context) {
            Some(idx) => LineRelocation {
                old_line,
                new_line: Some(idx + 1),
                confidence: Confidence::Exact,
            },
            None => LineRelocation {
                old_line,
                new_line: None,
                confidence: Confidence::Unresolved,
            },
        };
    }

    let fuzzy: Vec<usize> = new_lines
        .iter()
        .enumerate()
        .filter(|(_, l)| within_one_edit(l.trim(), target))
        .map(|(i, _)| i)
        .collect();
    match resolve_candidates(&fuzzy, new_lines, context) {
        Some(idx) => LineRelocation {
            old_line,
            new_line: Some(idx + 1),
            confidence: Confidence::Fuzzy,
        },
        None => LineRelocation {
            old_line,
            new_line: None,
            confidence: Confidence::Unresolved,
        },
    }
}

/// Relocates one cited range from `old_lines` into `new_lines`: relocates
/// `old_start` via [`relocate_line`], then — for an actual multi-line range —
/// checks that `old_end`'s text (exactly or within one edit) still sits
/// `old_end - old_start` lines after the resolved start. A single-line
/// citation (`old_start == old_end`) needs no separate end check. A broken
/// span is reported (`new_end: None`, `Unresolved`), never guessed by
/// keeping the old span.
fn relocate_range(
    old_lines: &[&str],
    new_lines: &[&str],
    old_start: usize,
    old_end: usize,
    context: &RelocationContext,
) -> RangeRelocation {
    let start = relocate_line(old_lines, new_lines, old_start, context);
    let Some(new_start) = start.new_line else {
        return RangeRelocation {
            old_start,
            old_end,
            new_start: None,
            new_end: None,
            confidence: Confidence::Unresolved,
        };
    };
    if old_start == old_end {
        return RangeRelocation {
            old_start,
            old_end,
            new_start: Some(new_start),
            new_end: Some(new_start),
            confidence: start.confidence,
        };
    }

    let span = old_end - old_start;
    let candidate_new_end = new_start + span;
    let old_end_text = old_lines.get(old_end - 1).map(|l| l.trim()).unwrap_or("");
    let new_end_text = new_lines
        .get(candidate_new_end - 1)
        .map(|l| l.trim())
        .unwrap_or("");
    let end_matches = old_end_text == new_end_text || within_one_edit(old_end_text, new_end_text);

    if end_matches {
        RangeRelocation {
            old_start,
            old_end,
            new_start: Some(new_start),
            new_end: Some(candidate_new_end),
            confidence: start.confidence,
        }
    } else {
        RangeRelocation {
            old_start,
            old_end,
            new_start: Some(new_start),
            new_end: None,
            confidence: Confidence::Unresolved,
        }
    }
}

/// Thin wrapper over [`relocate_range`] returning the `(old, new, confidence)`
/// triple shape the P8-1/T2 brief specifies.
fn relocate_citation(
    old_lines: &[&str],
    new_lines: &[&str],
    old_start: usize,
    old_end: usize,
    context: &RelocationContext,
) -> CitationRelocation {
    let r = relocate_range(old_lines, new_lines, old_start, old_end, context);
    CitationRelocation {
        old: (r.old_start, r.old_end),
        new: match (r.new_start, r.new_end) {
            (Some(s), Some(e)) => Some((s, e)),
            _ => None,
        },
        confidence: r.confidence,
    }
}

#[test]
fn relocate_line_follows_an_unchanged_region_through_an_offset_shift() {
    let old_lines = vec!["Heading A", "Give quarter to a defeated foe.", "Next line"];
    let new_lines = vec![
        "New preamble line one",
        "New preamble line two",
        "Heading A",
        "Give quarter to a defeated foe.",
        "Next line",
    ];

    let result = relocate_line(&old_lines, &new_lines, 2, &RelocationContext::default());

    assert_eq!(
        result,
        LineRelocation {
            old_line: 2,
            new_line: Some(4),
            confidence: Confidence::Exact,
        },
        "an unchanged line pushed two lines later by an inserted preamble must relocate \
         exactly to its new position, not merely be confirmed as still present"
    );
}

#[test]
fn relocate_line_inside_a_changed_hunk_is_unresolved_never_guessed() {
    let old_lines = vec![
        "Heading B",
        "The old wording that gets rewritten entirely.",
        "Trailing line",
    ];
    let new_lines = vec![
        "Heading B",
        "A completely different sentence with no resemblance.",
        "Trailing line",
    ];

    let result = relocate_line(&old_lines, &new_lines, 2, &RelocationContext::default());

    assert_eq!(
        result,
        LineRelocation {
            old_line: 2,
            new_line: None,
            confidence: Confidence::Unresolved,
        },
        "a line rewritten beyond a single-character fix has no safe candidate and must be \
         flagged for a human, never guessed at"
    );
}

#[test]
fn relocate_line_tolerates_a_single_character_ocr_fix() {
    let old_lines = vec![
        "Heading C",
        "A guarter share of the vis goes to the covenant.",
        "Footer",
    ];
    let new_lines = vec![
        "Preamble",
        "Heading C",
        "A quarter share of the vis goes to the covenant.",
        "Footer",
    ];

    let result = relocate_line(&old_lines, &new_lines, 2, &RelocationContext::default());

    assert_eq!(
        result,
        LineRelocation {
            old_line: 2,
            new_line: Some(3),
            confidence: Confidence::Fuzzy,
        },
        "a single-character OCR fix (guarter -> quarter, upstream's own correction) must still \
         resolve, as Fuzzy rather than Exact since the text is not byte-identical"
    );
}

#[test]
fn relocate_line_does_not_guess_between_two_equally_plausible_fuzzy_candidates() {
    let old_lines = vec!["Give guarter to a defeated foe."];
    let new_lines = vec![
        "Give quarter to a defeated foe.", // the real upstream fix
        "Give guerter to a defeated foe.", // a coincidentally similar, wrong, decoy line
    ];

    let result = relocate_line(&old_lines, &new_lines, 1, &RelocationContext::default());

    assert_eq!(
        result,
        LineRelocation {
            old_line: 1,
            new_line: None,
            confidence: Confidence::Unresolved,
        },
        "two candidates at the same single-character edit distance must not be resolved by a \
         coin flip — fuzzy tolerance must be neither too strict (missing the real fix) nor too \
         loose (matching a coincidentally similar wrong line)"
    );
}

#[test]
fn relocate_line_is_unresolved_on_a_duplicate_line_without_disambiguating_context() {
    let old_lines = vec![
        "## Entry Alpha",
        "*Minor, General*<br>",
        "Alpha description text.",
        "## Entry Beta",
        "*Minor, General*<br>",
        "Beta description text.",
    ];
    let new_lines = vec![
        "Intro paragraph.",
        "## Entry Alpha",
        "*Minor, General*<br>",
        "Alpha description text.",
        "## Entry Beta",
        "*Minor, General*<br>",
        "Beta description text.",
    ];

    // Citing old line 5 — Beta's identical "*Minor, General*<br>" marker.
    let result = relocate_line(&old_lines, &new_lines, 5, &RelocationContext::default());

    assert_eq!(
        result,
        LineRelocation {
            old_line: 5,
            new_line: None,
            confidence: Confidence::Unresolved,
        },
        "the exact text recurs at two places in the new file; without context neither \
         occurrence may be preferred over the other"
    );
}

#[test]
fn relocate_line_disambiguates_a_duplicate_line_using_captured_next_line_context() {
    let old_lines = vec![
        "## Entry Alpha",
        "*Minor, General*<br>",
        "Alpha description text.",
        "## Entry Beta",
        "*Minor, General*<br>",
        "Beta description text.",
    ];
    let new_lines = vec![
        "Intro paragraph.",
        "## Entry Alpha",
        "*Minor, General*<br>",
        "Alpha description text.",
        "## Entry Beta",
        "*Minor, General*<br>",
        "Beta description text.",
    ];
    let context = RelocationContext {
        next_line: Some("Beta description text.".to_string()),
        ..RelocationContext::default()
    };

    let result = relocate_line(&old_lines, &new_lines, 5, &context);

    assert_eq!(
        result,
        LineRelocation {
            old_line: 5,
            new_line: Some(6),
            confidence: Confidence::Exact,
        },
        "capturing the following line's text at snapshot time — before the resync — must \
         disambiguate the duplicate that plain text search alone cannot"
    );
}

#[test]
fn relocate_line_is_unresolved_when_next_line_also_collides_without_heading_context() {
    // Both entries share not only the cited line's text but also the text
    // that follows it — the real corpus has this shape (e.g. a repeated
    // statblock field followed by the same next field name in every
    // instance). `next_line` alone cannot narrow this; it must stay
    // `Unresolved` without a further tiebreaker.
    let old_lines = vec![
        "## Entry Alpha",
        "*Minor, General*<br>",
        "Shared description text.",
        "## Entry Beta",
        "*Minor, General*<br>",
        "Shared description text.",
    ];
    let new_lines = vec![
        "Intro paragraph.",
        "## Entry Alpha",
        "*Minor, General*<br>",
        "Shared description text.",
        "## Entry Beta",
        "*Minor, General*<br>",
        "Shared description text.",
    ];
    let context = RelocationContext {
        next_line: Some("Shared description text.".to_string()),
        ..RelocationContext::default()
    };

    let result = relocate_line(&old_lines, &new_lines, 5, &context);

    assert_eq!(
        result,
        LineRelocation {
            old_line: 5,
            new_line: None,
            confidence: Confidence::Unresolved,
        },
        "next_line context that matches every candidate equally narrows nothing; without a \
         further tiebreaker this must stay Unresolved, never guessed"
    );
}

#[test]
fn relocate_line_disambiguates_using_heading_when_next_line_also_collides() {
    let old_lines = vec![
        "## Entry Alpha",
        "*Minor, General*<br>",
        "Shared description text.",
        "## Entry Beta",
        "*Minor, General*<br>",
        "Shared description text.",
    ];
    let new_lines = vec![
        "Intro paragraph.",
        "## Entry Alpha",
        "*Minor, General*<br>",
        "Shared description text.",
        "## Entry Beta",
        "*Minor, General*<br>",
        "Shared description text.",
    ];
    // Same colliding next_line as above, but now the nearest preceding
    // heading — captured against the OLD file, before any resync — is also
    // supplied, and it differs between the two candidates.
    let context = RelocationContext {
        next_line: Some("Shared description text.".to_string()),
        heading: Some("## Entry Beta".to_string()),
    };

    let result = relocate_line(&old_lines, &new_lines, 5, &context);

    assert_eq!(
        result,
        LineRelocation {
            old_line: 5,
            new_line: Some(6),
            confidence: Confidence::Exact,
        },
        "the enclosing heading differs even though the cited line and its follower do not; it \
         must resolve the duplicate that next_line alone could not"
    );
}

#[test]
fn relocate_range_follows_an_unchanged_paragraph_through_an_offset_shift() {
    let old_lines = vec![
        "Header",
        "Paragraph line one of the ruling.",
        "Paragraph line two of the ruling.",
        "Paragraph line three of the ruling.",
        "Footer",
    ];
    let new_lines = vec![
        "Pre A",
        "Pre B",
        "Header",
        "Paragraph line one of the ruling.",
        "Paragraph line two of the ruling.",
        "Paragraph line three of the ruling.",
        "Footer",
    ];

    let result = relocate_range(&old_lines, &new_lines, 2, 4, &RelocationContext::default());

    assert_eq!(
        result,
        RangeRelocation {
            old_start: 2,
            old_end: 4,
            new_start: Some(4),
            new_end: Some(6),
            confidence: Confidence::Exact,
        },
        "both ends of an unchanged paragraph must relocate together, preserving the original \
         span"
    );
}

#[test]
fn relocate_range_reports_rather_than_guesses_when_the_span_no_longer_matches() {
    let old_lines = vec![
        "Header",
        "Paragraph line one of the ruling.",
        "Paragraph line two of the ruling.",
        "Paragraph line three of the ruling.",
        "Footer",
    ];
    let new_lines = vec![
        "Header",
        "Paragraph line one of the ruling.",
        "Paragraph line one point five, newly inserted.",
        "Paragraph line two of the ruling.",
        "Paragraph line three of the ruling.",
        "Footer",
    ];

    let result = relocate_range(&old_lines, &new_lines, 2, 4, &RelocationContext::default());

    assert_eq!(
        result,
        RangeRelocation {
            old_start: 2,
            old_end: 4,
            new_start: Some(2),
            new_end: None,
            confidence: Confidence::Unresolved,
        },
        "the start relocates cleanly, but an inserted line breaks the original span; the end \
         must be flagged for a human rather than silently kept at start + old span"
    );
}

#[test]
fn relocate_citation_returns_the_old_new_confidence_triple_the_brief_specifies() {
    let old_lines = vec!["Heading A", "Give quarter to a defeated foe.", "Next line"];
    let new_lines = vec![
        "New preamble line one",
        "New preamble line two",
        "Heading A",
        "Give quarter to a defeated foe.",
        "Next line",
    ];

    let result = relocate_citation(&old_lines, &new_lines, 2, 2, &RelocationContext::default());

    assert_eq!(
        result,
        CitationRelocation {
            old: (2, 2),
            new: Some((4, 4)),
            confidence: Confidence::Exact,
        },
        "a single-line citation's range triple must carry the resolved new position with \
         start == end preserved"
    );
}
