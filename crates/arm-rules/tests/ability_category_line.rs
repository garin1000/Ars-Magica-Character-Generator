//! Category-line guard (data-integrity, PHASE 1 — test + triage only, see
//! `tmp/ability-guard-handover.md`): every `rules/core/abilities.json`
//! entry's shipped `category` + `requires_training` must match what the
//! English core rulebook itself states about that Ability.
//!
//! Two independent rulebook sources are read and cross-checked:
//!
//! 1. The trailing `(<Category>)` parenthetical "the Ability List"
//!    (ArMDE:7269-7788) prints at the end of each entry's own passage
//!    (ArMDE:7271: "The type of the Ability is given in parentheses at the
//!    end of its description."). Scanned over the entry's own
//!    `source.lines[0]..=lines[1]` range (the same heading-to-next-heading
//!    span the data already cites), not just the line right after the
//!    heading — unlike a Virtue/Flaw descriptor, this line sits at the END of
//!    a (sometimes multi-paragraph, sometimes table-bearing) passage, not
//!    directly under the heading.
//! 2. The "Abilities by Type" summary list (ArMDE:7177-7268), which groups
//!    every Ability's `(#anchor)` link under one of five `#### <Category>
//!    Abilities` headings.
//!
//! Three entries — Embitterment, Summon Animals, Whistle Up The Wind
//! (ArMDE:7443-7444, :7752-7753, :7783-7784) — are stub passages that only
//! cross-reference the Hermetic Magic chapter and carry no trailing
//! parenthetical at all, so source 1 is silent for them and source 2 (which
//! lists all three under Supernatural Abilities) is what settles their
//! category. Conversely Music (ArMDE:7654-7656, own passage ends
//! "(General)") is missing from the General Abilities summary list entirely
//! — a book omission source 2 cannot see — so source 1 settles it instead.
//! The two sources are therefore mutual fallbacks: an entry seen by only one
//! is read from that one, an entry seen by both must agree, and an entry seen
//! by neither is a genuine parser gap, not a silent skip.
//!
//! `requires_training` comes from a third, independent place: whether the
//! entry's own `####` heading line ends in a literal `\*` (ArMDE:7128:
//! asterisked Abilities cannot be used without at least one experience
//! point). This does not depend on either category source.
//!
//! No `rules/core/` or `src` change lands in this phase — see
//! `vf_descriptor_line.rs` for the model this guard follows (same
//! `RULED_EXCEPTIONS` shape, same "list every mismatch in one failure"
//! contract), applied here to Abilities' single `category` field rather than
//! Virtue/Flaw's `categories` list. [`RULED_EXCEPTIONS`] is empty: the Phase 1
//! triage found zero Ability catalogue mismatches.
//!
//! Specialty examples (the free-text list each passage's `*Specialties:*`
//! clause names) are NOT checked here: `rules/core/abilities.json` does not
//! store them at all (a character's `AbilityScore::specialty` is free text
//! the player supplies, not drawn from a catalogued list), so there is
//! nothing in the data for such a check to compare against.

use serde_json::Value;
use std::collections::BTreeMap;

const ABILITIES_JSON: &str = include_str!("../../../rules/core/abilities.json");
const EN_CORE_RULES: &str =
    include_str!("../../../rules/source/en/Ars Magica - Definitive Edition (Core Rules).md");

/// One `rules/core/abilities.json` catalogue entry, read straight from the
/// raw JSON rather than `arm_rules::ability::Ability` — this guard needs the
/// bare `source.lines`/`source.anchor` fields to index into the Markdown
/// source and join against the summary list, which `Ability`'s
/// provenance-only `source` field does not expose.
struct Entry {
    id: String,
    category: String,
    requires_training: bool,
    anchor: String,
    heading_line: usize,
    end_line: usize,
}

fn entries() -> Vec<Entry> {
    let all: Value = serde_json::from_str(ABILITIES_JSON).expect("abilities.json is valid JSON");
    all["abilities"]
        .as_array()
        .expect("abilities.json has a top-level \"abilities\" array")
        .iter()
        .map(|v| Entry {
            id: v["id"].as_str().expect("entry has an id").to_string(),
            category: v["category"]
                .as_str()
                .expect("entry has a category")
                .to_string(),
            requires_training: v["requires_training"].as_bool().unwrap_or(false),
            anchor: v["source"]["anchor"]
                .as_str()
                .expect("entry has source.anchor")
                .to_string(),
            heading_line: v["source"]["lines"][0]
                .as_u64()
                .expect("entry has source.lines[0]") as usize,
            end_line: v["source"]["lines"][1]
                .as_u64()
                .expect("entry has source.lines[1]") as usize,
        })
        .collect()
}

/// Maps a descriptor's own category word (exactly as the book capitalizes
/// it) to the slug `rules/core/abilities.json` uses.
fn category_slug(word: &str) -> Option<&'static str> {
    match word {
        "General" => Some("general"),
        "Academic" => Some("academic"),
        "Arcane" => Some("arcane"),
        "Martial" => Some("martial"),
        "Supernatural" => Some("supernatural"),
        _ => None,
    }
}

/// Scans `text` lines `start..=end` (1-based, inclusive — the same range
/// `source.lines` already cites) for the LAST line whose trimmed end is a
/// bare `(<Category>)` parenthetical naming one of the five category words.
/// Abilities carry a single category (unlike Virtue/Flaw's `categories`
/// list), so no connective-splitting is needed; "last" rather than "first" is
/// defensive against an embedded sub-table inside a long entry (Hex,
/// Induction, Corpse Magic) that might otherwise also end a line in a
/// parenthesis — in practice every entry in this catalogue has at most one
/// such line. `None` means the entry's own passage never states a category
/// (the three Hermetic-Magic-chapter stub entries), which the caller falls
/// back to the summary list for rather than treating as unparseable on its
/// own.
fn own_descriptor_category(text: &str, start: usize, end: usize) -> Option<&'static str> {
    let mut found = None;
    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        if line_no < start || line_no > end {
            continue;
        }
        let trimmed = line.trim_end();
        for word in ["General", "Academic", "Arcane", "Martial", "Supernatural"] {
            if trimmed.ends_with(&format!("({word})")) {
                found = category_slug(word);
            }
        }
    }
    found
}

/// Whether the entry's own `####` heading line (`source.lines[0]`) ends in a
/// literal `\*` — the book's own asterisk marker for an Ability that
/// "cannot even attempt rolls on an asterisked Ability without at least one
/// experience point in it" (ArMDE:4157, the Jack of All Trades Virtue
/// spelling out the rule this flag gates). `None` means `heading_line` is not
/// actually a `#### ` heading line at all — a parser-gap failure, surfaced
/// rather than defaulted.
fn heading_requires_training(text: &str, heading_line: usize) -> Option<bool> {
    let line = text.lines().nth(heading_line - 1)?;
    let trimmed = line.trim_end();
    trimmed
        .starts_with("#### ")
        .then(|| trimmed.ends_with("\\*"))
}

/// Parses the "Abilities by Type" summary list (ArMDE:7175 "## Abilities by
/// Type" through just before ArMDE:7269 "## Ability List"): five `####
/// <Category> Abilities` headings, each followed by `[<Label>](#<anchor>)<br>`
/// lines. Returns anchor -> category slug. Used only as a fallback for the
/// few entries whose own passage carries no trailing category parenthetical
/// (see module doc) — deliberately NOT cross-checked against the label's own
/// asterisk, since `requires_training` already has its own, independent,
/// single source of truth ([`heading_requires_training`]).
fn summary_list_categories(text: &str) -> BTreeMap<String, &'static str> {
    let mut map = BTreeMap::new();
    let mut current_category: Option<&'static str> = None;
    let mut in_section = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "## Abilities by Type" {
            in_section = true;
            continue;
        }
        if trimmed == "## Ability List" {
            break;
        }
        if !in_section {
            continue;
        }
        if let Some(heading) = trimmed.strip_prefix("#### ") {
            current_category = heading.strip_suffix(" Abilities").and_then(category_slug);
            continue;
        }
        let Some(category) = current_category else {
            continue;
        };
        let Some(rest) = trimmed.strip_prefix('[') else {
            continue;
        };
        let Some(close_bracket) = rest.find(']') else {
            continue;
        };
        let after_label = &rest[close_bracket..];
        let Some(anchor_marker) = after_label.find("(#") else {
            continue;
        };
        let after_marker = &after_label[anchor_marker + 2..];
        let Some(close_paren) = after_marker.find(')') else {
            continue;
        };
        map.insert(after_marker[..close_paren].to_string(), category);
    }
    map
}

/// Entries whose own data is known, by a recorded ruling, to disagree with
/// what the rulebook states — same shape as
/// `vf_descriptor_line.rs::RULED_EXCEPTIONS`. Empty: the Phase 1 triage of
/// this guard found zero Ability catalogue mismatches to except. Kept (rather
/// than deleted) so a future mismatch has a ready-made, already-wired place to
/// record a ruling, and so [`ruled_exceptions_still_mismatch`] keeps meaning
/// what it says if one is ever added.
const RULED_EXCEPTIONS: &[(&str, &str)] = &[];

fn exception_reason(id: &str) -> Option<&'static str> {
    RULED_EXCEPTIONS
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| *r)
}

/// Compares one entry's shipped `category`/`requires_training` against the
/// rulebook. `Err` carries a human-readable mismatch description; `Ok` means
/// everything agrees.
fn check(entry: &Entry, summary: &BTreeMap<String, &'static str>) -> Result<(), String> {
    let descriptor_category =
        own_descriptor_category(EN_CORE_RULES, entry.heading_line, entry.end_line);
    let summary_category = summary.get(&entry.anchor).copied();

    let effective_category = match (descriptor_category, summary_category) {
        (Some(d), Some(s)) if d != s => {
            return Err(format!(
                "{}: own passage's trailing parenthetical says \"({d})\" but the \"Abilities by \
                 Type\" summary list files anchor \"{}\" under \"{s}\" (source lines {}-{})",
                entry.id, entry.anchor, entry.heading_line, entry.end_line
            ));
        }
        (Some(d), _) => d,
        (None, Some(s)) => s,
        (None, None) => {
            return Err(format!(
                "{}: no category found — neither a trailing \"(Category)\" line in its own \
                 passage (source lines {}-{}) nor an entry in the \"Abilities by Type\" summary \
                 list (anchor \"{}\")",
                entry.id, entry.heading_line, entry.end_line, entry.anchor
            ));
        }
    };

    if entry.category != effective_category {
        return Err(format!(
            "{}: shipped category \"{}\" does not match the rulebook's \"{effective_category}\" \
             (source lines {}-{})",
            entry.id, entry.category, entry.heading_line, entry.end_line
        ));
    }

    match heading_requires_training(EN_CORE_RULES, entry.heading_line) {
        None => Err(format!(
            "{}: source line {} is not a \"#### \" heading line as expected — parser gap, not a \
             data defect",
            entry.id, entry.heading_line
        )),
        Some(asterisked) if asterisked != entry.requires_training => Err(format!(
            "{}: shipped requires_training={} does not match its own heading's asterisk \
             (requires_training should be {asterisked}) at source line {}",
            entry.id, entry.requires_training, entry.heading_line
        )),
        Some(_) => Ok(()),
    }
}

/// Catalogue size is data, never code (CLAUDE.md "Architecture invariants"):
/// this does not pin a literal entry count, it independently re-reads
/// `abilities.json`'s own top-level `"abilities"` array length and asserts
/// [`entries()`] saw every one of them (and that the array isn't empty, so a
/// future truncation can't pass by vacuously matching 0 == 0).
#[test]
fn every_entry_matches_the_rulebook() {
    let summary = summary_list_categories(EN_CORE_RULES);
    let all = entries();

    let raw: Value = serde_json::from_str(ABILITIES_JSON).expect("abilities.json is valid JSON");
    let shipped_count = raw["abilities"]
        .as_array()
        .expect("abilities.json has a top-level \"abilities\" array")
        .len();
    assert!(
        shipped_count > 0,
        "abilities.json's own \"abilities\" array is empty — nothing to guard"
    );
    assert_eq!(
        all.len(),
        shipped_count,
        "entries() read {} entries but abilities.json's own \"abilities\" array has {} — this \
         guard must see every entry, never a subset",
        all.len(),
        shipped_count
    );

    let mut failures = Vec::new();
    for entry in &all {
        if exception_reason(&entry.id).is_some() {
            continue;
        }
        if let Err(msg) = check(entry, &summary) {
            failures.push(msg);
        }
    }
    assert!(
        failures.is_empty(),
        "{} Ability entries disagree with the rulebook's own category/training markers:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Every [`RULED_EXCEPTIONS`] row exists to silence
/// [`every_entry_matches_the_rulebook`] for one entry whose rulebook text
/// genuinely disagrees with the shipped data by a recorded ruling. If a
/// future data change makes the comparison agree again, the row must be
/// deleted — this test fails loudly instead of leaving a dead, unverifiable
/// row sitting in the list.
#[test]
fn ruled_exceptions_still_mismatch() {
    let all = entries();
    let summary = summary_list_categories(EN_CORE_RULES);
    let mut stale = Vec::new();
    for (id, _) in RULED_EXCEPTIONS {
        let entry = all.iter().find(|e| e.id == *id).unwrap_or_else(|| {
            panic!(
                "RULED_EXCEPTIONS names \"{id}\", which is no longer in rules/core/abilities.json"
            )
        });
        if check(entry, &summary).is_ok() {
            stale.push(*id);
        }
    }
    assert!(
        stale.is_empty(),
        "RULED_EXCEPTIONS row(s) {stale:?} now MATCH the rulebook — the defect they recorded is \
         gone, so remove the row(s)"
    );
}

/// Proves [`check`] can actually fail — [`every_entry_matches_the_rulebook`]
/// never once went RED against the real catalogue (see
/// `tmp/ability-guard-handover.md`), so nothing else demonstrates this guard
/// would catch a real defect rather than vacuously passing everything. Each
/// case below starts from a real entry's real source range/anchor (so the
/// rulebook text being compared against is genuine) and corrupts exactly one
/// thing a shipped `rules/core/abilities.json` entry could get wrong.
#[test]
fn check_rejects_synthetic_wrong_entries() {
    let summary = summary_list_categories(EN_CORE_RULES);

    // Wrong category: Animal Handling (ArMDE:7273-7275) is really General.
    let wrong_category = Entry {
        id: "ability.animal_handling".to_string(),
        category: "arcane".to_string(),
        requires_training: false,
        anchor: "animal-handling".to_string(),
        heading_line: 7273,
        end_line: 7275,
    };
    assert!(
        check(&wrong_category, &summary).is_err(),
        "a deliberately wrong category must be rejected"
    );

    // Wrong requires_training: Animal Ken (ArMDE:7277-7279) really requires
    // training (its heading carries the book's own `\*`).
    let wrong_training = Entry {
        id: "ability.animal_ken".to_string(),
        category: "supernatural".to_string(),
        requires_training: false,
        anchor: "animal-ken-1".to_string(),
        heading_line: 7277,
        end_line: 7279,
    };
    assert!(
        check(&wrong_training, &summary).is_err(),
        "a deliberately wrong requires_training must be rejected"
    );

    // No descriptor in its own range AND absent from the summary list.
    // Embitterment's real range (ArMDE:7443-7444) genuinely carries no
    // trailing category parenthetical — but its real anchor ("embitterment-1")
    // IS in the summary list, so a bogus anchor is substituted to make the
    // summary-list lookup miss too, landing in check()'s (None, None) branch.
    let unparseable = Entry {
        id: "ability.embitterment".to_string(),
        category: "supernatural".to_string(),
        requires_training: true,
        anchor: "not-a-real-anchor".to_string(),
        heading_line: 7443,
        end_line: 7444,
    };
    assert!(
        check(&unparseable, &summary).is_err(),
        "an entry with no descriptor in its own range and no summary-list match must be rejected"
    );
}
