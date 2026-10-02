//! Descriptor-line guard (data-integrity, PHASE 1 — test + triage only, see
//! `tmp/descriptor-guard-handover.md`): every `rules/core/virtues_flaws.json`
//! entry's shipped `magnitude` + `categories` must match the italic
//! descriptor line the English core rulebook prints directly under the
//! entry's own `####` heading (`source.lines[0]`) — a line of the shape
//! `*<Magnitude>, <Category>[, <Category>...]*`, where `<Magnitude>` is
//! `Free`/`Minor`/`Major` (or two of those joined by `" or "` for a
//! Major/Minor twin pair sharing one passage) and each `<Category>` is one of
//! the book's own category words, occasionally joined by `" and "`/`" or "`
//! instead of a comma, occasionally carrying a `Tainted` type tag or an
//! `"… only"` restriction note, and occasionally mangled by the source
//! Markdown's own OCR slips (a period standing in for a comma, or the
//! separator dropped outright).
//!
//! No `rules/core/` or `src` change lands in this phase — [`RULED_EXCEPTIONS`]
//! is the only departure from a literal descriptor-vs-data comparison, and
//! each row cites the ruling that departs (same shape as
//! `uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`: a row records a human
//! reading, not a license, and [`ruled_exceptions_still_mismatch`] asserts
//! every row still fails the plain comparison, so the list can only shrink).

use serde_json::Value;
use std::collections::BTreeSet;

const VF_JSON: &str = include_str!("../../../rules/core/virtues_flaws.json");
const EN_CORE_RULES: &str =
    include_str!("../../../rules/source/en/Ars Magica - Definitive Edition (Core Rules).md");

/// One `rules/core/virtues_flaws.json` entry, read straight from the raw
/// JSON rather than `arm_rules::types::PointItem` — this guard needs
/// `source.lines[0]` as a bare line number to index into the Markdown
/// source, which `PointItem` does not expose at all (by design; see its own
/// doc comment on why `source` stays provenance-only).
struct Entry {
    id: String,
    magnitude: String,
    categories: Vec<String>,
    heading_line: usize,
}

fn entries() -> Vec<Entry> {
    let all: Value = serde_json::from_str(VF_JSON).expect("virtues_flaws.json is valid JSON");
    all.as_array()
        .expect("virtues_flaws.json is a top-level array")
        .iter()
        .map(|v| Entry {
            id: v["id"].as_str().expect("entry has an id").to_string(),
            magnitude: v["magnitude"]
                .as_str()
                .expect("entry has a magnitude")
                .to_string(),
            categories: v["categories"]
                .as_array()
                .expect("entry has a categories array")
                .iter()
                .map(|c| c.as_str().expect("category is a string").to_string())
                .collect(),
            heading_line: v["source"]["lines"][0]
                .as_u64()
                .expect("entry has source.lines[0]") as usize,
        })
        .collect()
}

/// A descriptor line's parsed content: every magnitude word it names (more
/// than one only for a `"Major or Minor"` twin split) and every category
/// word it names (more than one for a comma/`" and "`/`" or "`-joined list),
/// plus any token neither table recognizes — a parser gap, surfaced rather
/// than silently dropped.
#[derive(Default, Debug)]
struct Descriptor {
    magnitudes: BTreeSet<String>,
    categories: BTreeSet<String>,
    unrecognized: Vec<String>,
}

/// Maps a descriptor's own category word (already lowercased) to the slug
/// `rules/core/virtues_flaws.json` uses — see `crates/arm-rules/RULES.md`
/// "Full core Virtue/Flaw catalogue" for the id spellings.
fn category_slug(word: &str) -> Option<&'static str> {
    match word {
        "general" => Some("general"),
        "hermetic" => Some("hermetic"),
        "social status" => Some("social_status"),
        "supernatural" => Some("supernatural"),
        // ArMDE:5011 ships this exact typo in the source Markdown itself
        // ("Major, Subernatural") — rules/source/ is a COPY (CLAUDE.md:
        // "repair defects upstream, not here"), so the parser recognizes the
        // typo instead of a local hand-edit papering over it.
        "subernatural" => Some("supernatural"),
        "story" => Some("story"),
        "personality" => Some("personality"),
        "special" => Some("special"),
        "mythic companion" => Some("mythic_companion"),
        _ => None,
    }
}

fn magnitude_word(word: &str) -> Option<&'static str> {
    match word {
        "free" => Some("free"),
        "minor" => Some("minor"),
        "major" => Some("major"),
        _ => None,
    }
}

/// Splits `" and "`/`" or "` (case-insensitive, already-lowercased input) out
/// of a single comma-delimited segment, so `"general and hermetic"` and
/// `"general or supernatural"` both yield two tokens from one segment.
fn split_connectives(segment: &str) -> Vec<String> {
    segment
        .replace(" and ", ",")
        .replace(" or ", ",")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parses one descriptor line's bracketed content (already stripped of the
/// leading/trailing `*` and any trailing `<br>`). Handles every shape the
/// book actually uses: comma lists, `" and "`/`" or "` joins (of either the
/// magnitude or the category part), a `Tainted` tag, an `"… only"`
/// restriction note (`animals only`), and the two OCR slips the source
/// Markdown ships — a period standing in for the comma (`"Minor. Hermetic"`)
/// and the separator dropped entirely (`"Minor General"`,
/// `"Free Social Status"`).
fn parse_descriptor(inner: &str) -> Descriptor {
    let normalized = inner.to_lowercase().replace('.', ",");
    let mut segments: Vec<String> = normalized
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // The missing-separator OCR slip: a magnitude word glued directly to the
    // following category word with a bare space ("minor general"). Must NOT
    // fire on a legitimate `" or "`/`" and "`-joined magnitude phrase ("major
    // or minor") — those are a real connective, handled by
    // `split_connectives` below, not a dropped separator.
    if let Some(first) = segments.first().cloned() {
        for word in ["free", "minor", "major"] {
            let prefix = format!("{word} ");
            if let Some(rest) = first.strip_prefix(&prefix) {
                if rest.starts_with("or ") || rest.starts_with("and ") {
                    break;
                }
                segments[0] = word.to_string();
                let rest = rest.trim().to_string();
                if !rest.is_empty() {
                    segments.insert(1, rest);
                }
                break;
            }
        }
    }

    let mut descriptor = Descriptor::default();
    for segment in &segments {
        for token in split_connectives(segment) {
            if let Some(m) = magnitude_word(&token) {
                descriptor.magnitudes.insert(m.to_string());
            } else if let Some(c) = category_slug(&token) {
                descriptor.categories.insert(c.to_string());
            } else if token == "tainted" {
                // The descriptor's "Type" tag, not a category — not asserted
                // by this guard (PointItem::tainted is a separate field).
            } else if token.ends_with(" only") {
                // A restriction note ("animals only") — not a category.
            } else {
                descriptor.unrecognized.push(token);
            }
        }
    }
    descriptor
}

/// Finds the first non-blank line strictly after `heading_line` (1-based) in
/// `text`, and — if it looks like a descriptor line at all — returns its
/// bracketed content. A descriptor line is self-contained italics and
/// nothing else: `*<content>*` optionally followed by `<br>`, with no prose
/// trailing the closing `*` (unlike a body paragraph that merely opens on an
/// italicized word, e.g. `` *Minor:* Your character's lineage... ``, which
/// has substantial text after its closing `*`). This also accepts
/// `virtue.prestigious_student`'s reversed-order descriptor (`*General,
/// Minor*`, ArMDE:4793) without special-casing it — this gate does not care
/// about word order, only shape. `None` means "no descriptor line found
/// immediately under this heading", which the caller reports as a mismatch
/// rather than panicking: the whole point of this guard is to list every
/// failure in one message.
fn descriptor_after(text: &str, heading_line: usize) -> Option<String> {
    let line = text
        .lines()
        .skip(heading_line)
        .find(|l| !l.trim().is_empty())?
        .trim();
    let stripped = line.strip_prefix('*')?;
    let end = stripped.find('*')?;
    let inner = &stripped[..end];
    let remainder = stripped[end + 1..].trim();
    (remainder.is_empty() || remainder == "<br>").then(|| inner.to_string())
}

/// Entries whose own descriptor line is known, by a recorded ruling, to
/// disagree with the shipped `magnitude`/`categories` — see
/// `tmp/descriptor-guard-handover.md` for the triage that produced this list.
/// Each row must still fail [`check`] ([`ruled_exceptions_still_mismatch`]),
/// so the list can only shrink as a reclassification lands, never grow
/// silently to hide a new defect.
const RULED_EXCEPTIONS: &[(&str, &str)] = &[
    (
        "flaw.independent_craftsman",
        "D54 (docs/vf-audit/decisions.md): ArMDE:6304 ends 'If you are not using \
         the rules in City and Guild..., treat this as a Personality Flaw.' The \
         app supports no supplements, so that stated condition holds today — \
         Norbert's ruling ships categories: [\"personality\"], overriding the \
         entry's own descriptor ('Minor, General', ArMDE:6303).",
    ),
    (
        "virtue.inoffensive_to_beings",
        "RULES.md 'Full core Virtue/Flaw catalogue', Norbert's 2026-09-13 \
         ruling: an *and*-joined descriptor ('General and Hermetic', \
         ArMDE:4134) models 'either route', not both-at-once, and hermetic \
         cannot be a membership category here — gift_categories is \
         [\"hermetic\"] on every profile, so admitting it would make holding \
         this entry itself grant The Gift. The book's own Hermetic-heading \
         index listing is preserved as index_categories: [\"hermetic\"] \
         instead of categories.",
    ),
    (
        "flaw.offensive_to_beings",
        "Same 2026-09-13 ruling as virtue.inoffensive_to_beings — descriptor \
         'Hermetic and General' (ArMDE:6525); categories stays [\"general\"], \
         index_categories carries [\"hermetic\"].",
    ),
    (
        "flaw.primogeniture_lineage",
        "Same 2026-09-13 ruling as virtue.inoffensive_to_beings — descriptor \
         'Story and Hermetic' (ArMDE:6635); categories stays [\"story\"], \
         index_categories carries [\"hermetic\"]. The entry's real \
         Verditius-magi-only restriction is modelled as a prerequisite \
         (All([OrderMember, House(house.verditius)])), not a category.",
    ),
    (
        "flaw.unbearable_to_beings",
        "Same 2026-09-13 ruling, for the one *or*-joined case: descriptor \
         'Hermetic or General' (ArMDE:6892) stays categories: [\"general\"] \
         because hermetic is overloaded with Gift detection; the real \
         eligibility is already the Any([Has(virtue.the_gift), \
         Has(flaw.magical_air)]) prerequisite, and index_categories carries \
         [\"hermetic\"].",
    ),
    (
        "flaw.vengeful_powers",
        "RULES.md ('flaw.vengeful_powers' taken_as note): the descriptor \
         (ArMDE:6960) states only 'Major, Story, Tainted' — no Hermetic. \
         ArMDE:6975 ('Vengeful Powers may be taken as a Hermetic Flaw... more \
         commonly associated with Supernatural Abilities') is body text, not \
         the descriptor, and is what the shipped taken_as parameter \
         ({\"hermetic\", \"story\"}, max_total: 1) and the widened categories: \
         [\"story\", \"hermetic\"] both derive from.",
    ),
    (
        "flaw.false_power_minor",
        "The shared heading's own descriptor (ArMDE:6081) states only 'Major, \
         Supernatural, Tainted' — no 'or Minor'. The Minor magnitude comes from \
         body text instead (ArMDE:6096: '...in each subsequent instance as a \
         Minor Flaw rather than a Major one'), which this guard does not read.",
    ),
];

fn exception_reason(id: &str) -> Option<&'static str> {
    RULED_EXCEPTIONS
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| *r)
}

/// Compares one entry's shipped `magnitude`/`categories` against its own
/// descriptor line. `Err` carries a human-readable mismatch description;
/// `Ok` means the two agree.
fn check(entry: &Entry) -> Result<(), String> {
    let descriptor_text = descriptor_after(EN_CORE_RULES, entry.heading_line).ok_or_else(|| {
        format!(
            "{}: no descriptor line found immediately under its own heading (source line {})",
            entry.id, entry.heading_line
        )
    })?;
    let parsed = parse_descriptor(&descriptor_text);

    if !parsed.unrecognized.is_empty() {
        return Err(format!(
            "{}: descriptor \"*{descriptor_text}*\" (source line {}) has unrecognized token(s) \
             {:?} — parser gap, not a data defect",
            entry.id, entry.heading_line, parsed.unrecognized
        ));
    }
    if parsed.magnitudes.is_empty() {
        return Err(format!(
            "{}: descriptor \"*{descriptor_text}*\" (source line {}) names no magnitude at all",
            entry.id, entry.heading_line
        ));
    }
    if !parsed.magnitudes.contains(&entry.magnitude) {
        return Err(format!(
            "{}: shipped magnitude \"{}\" is not among the descriptor's {:?} (\"*{descriptor_text}*\", \
             source line {})",
            entry.id, entry.magnitude, parsed.magnitudes, entry.heading_line
        ));
    }
    // A `"Major or Minor"` descriptor accepts either magnitude for EITHER
    // twin id sharing it, which cannot by itself catch a transposed pair
    // (the Major twin shipped as `minor` and vice versa) — every twin in the
    // catalogue names its own magnitude in its id (`_major`/`_minor`
    // suffix), so pin that convention directly as a second, independent
    // check.
    if entry.id.ends_with("_major") && entry.magnitude != "major" {
        return Err(format!(
            "{}: id ends \"_major\" but shipped magnitude is \"{}\"",
            entry.id, entry.magnitude
        ));
    }
    if entry.id.ends_with("_minor") && entry.magnitude != "minor" {
        return Err(format!(
            "{}: id ends \"_minor\" but shipped magnitude is \"{}\"",
            entry.id, entry.magnitude
        ));
    }

    let shipped: BTreeSet<String> = entry.categories.iter().cloned().collect();
    if shipped != parsed.categories {
        return Err(format!(
            "{}: shipped categories {:?} do not match the descriptor's {:?} (\"*{descriptor_text}*\", \
             source line {})",
            entry.id, entry.categories, parsed.categories, entry.heading_line
        ));
    }

    Ok(())
}

#[test]
fn every_entry_matches_its_own_descriptor_line() {
    let mut failures = Vec::new();
    for entry in entries() {
        if exception_reason(&entry.id).is_some() {
            continue;
        }
        if let Err(msg) = check(&entry) {
            failures.push(msg);
        }
    }
    assert!(
        failures.is_empty(),
        "{} Virtue/Flaw entries disagree with their own descriptor line:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Every [`RULED_EXCEPTIONS`] row exists to silence [`every_entry_matches_its_own_descriptor_line`]
/// for one entry whose descriptor genuinely disagrees with the shipped data
/// by a recorded ruling. If a future data change makes the comparison agree
/// again (the exception's rationale no longer applies), the row must be
/// deleted — this test fails loudly instead of leaving a dead, unverifiable
/// row sitting in the list.
#[test]
fn ruled_exceptions_still_mismatch() {
    let all = entries();
    let mut stale = Vec::new();
    for (id, _) in RULED_EXCEPTIONS {
        let entry = all.iter().find(|e| e.id == *id).unwrap_or_else(|| {
            panic!("RULED_EXCEPTIONS names \"{id}\", which is no longer in rules/core/virtues_flaws.json")
        });
        if check(entry).is_ok() {
            stale.push(*id);
        }
    }
    assert!(
        stale.is_empty(),
        "RULED_EXCEPTIONS row(s) {stale:?} now MATCH their own descriptor line — the defect they \
         recorded is gone, so remove the row(s)"
    );
}
