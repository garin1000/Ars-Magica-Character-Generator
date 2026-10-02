//! Spell catalogue stat-line guard — PHASE 1 (test + triage only).
//!
//! Cross-checks every `rules/core/spells.json` entry against what the book
//! actually states at the entry's own cited `source.lines`: Technique, Form,
//! Level, Range/Duration/Target (ArMDE:12001-12099, the RDT chart) and Ritual
//! (Year Duration forces it, ArMDE:12055; Boundary Target forces it,
//! ArMDE:12077), plus Art requisites where the book states them.
//!
//! # Book layout this parser relies on
//!
//! A spell chapter nests four heading levels (ArMDE:12426 "Creo Animal
//! Guidelines" onward is the first full example):
//! - `## <Form> Spells` (e.g. "Animal Spells") — not used by this parser; the
//!   next level already carries both Technique and Form.
//! - `### <Technique> <Form> (Guidelines|Spells)` — sets the current
//!   Technique+Form. Matched by word content only, ignoring both the heading
//!   depth and the trailing word: the book carries two heading-depth
//!   anomalies ("## Muto Herbam Guidelines" ArMDE:13971, "## Creo Mentem
//!   Spells" ArMDE:14860, both "##" where every sibling section is "###") and
//!   one trailing-word typo ("### Rego Corpus Svells" ArMDE:13769, "Svells"
//!   for "Spells") that a depth- or suffix-strict match would silently lose a
//!   whole section to.
//! - `#### LEVEL <n>` or `#### GENERAL` — sets the current Level for every
//!   spell heading until the next one (a General spell's catalogue `level` is
//!   `None`, ArMDE:12349-12353 — see `spell.rs`'s own doc comment).
//! - `##### <Spell Name>` — the spell itself, immediately followed by its
//!   `R: ..., D: ..., T: ...[, Ritual]` stat line and, for some spells, a
//!   `Req:`/`Reg:` requisite line (`Reg:` is the book's own typo for `Req:`,
//!   seen at ArMDE:13222, :15448, :15545 and inline at ArMDE:14016 — this
//!   parser treats both spellings as the same marker, which is exactly how it
//!   catches the four entries that are missing the requisite this typo
//!   states; see the handover's DATA BUG rows).
//!
//! The book also abbreviates and sometimes mis-punctuates the stat line
//! itself (missing commas, trailing periods, "Eve" for "Eye", "Arc" for
//! Arcane Connection, "Per" for Personal, "Spec"/"Special" for a duration or
//! target the fixed enums have no slot for — `rules/core/spells.json` omits
//! the field entirely for those five spells rather than inventing a value,
//! confirmed against the book at each one: `spell.wind_at_the_back`
//! ArMDE:13319, `spell.the_bountiful_feast` :13918, `spell.trackless_step`
//! :15506, `spell.the_earth_split_asunder` :15569, `spell.watching_ward`
//! :15919). `normalize_duration`/`normalize_target` reproduce that mapping.
//! One spell, Mists of Change (ArMDE:13634), states a combined
//! "D: Sun & Year" — `normalize_duration` reads this as Year (the token that
//! actually appears in the RDT chart and the one the data already carries),
//! treating "Sun" as descriptive text about *when* the ritual is cast rather
//! than a second Duration; see the handover's QUESTIONS for whether this
//! deserves a sharper reading.
//!
//! # What this guard does not check
//!
//! `creates_lasting` and `parameters` are out of scope (not stat-line facts);
//! so is cross-checking the `(form)`/`(Form)` meta-magic spells' own
//! parameter domain. Requisite *order* is not checked, only the set — the
//! book's own order (e.g. "Req: Imaginem, Rego") happens to match the data's
//! in every case seen, but nothing says it must.

use std::collections::{BTreeMap, HashSet};
use std::sync::OnceLock;

use regex::Regex;
use serde::Deserialize;

use arm_rules::types::Id;
use arm_rules::{Spell, SpellDuration, SpellRange, SpellTarget};

const SPELLS_JSON: &str = include_str!("../../../rules/core/spells.json");
const CORE_RULES_MD: &str =
    include_str!("../../../rules/source/en/Ars Magica - Definitive Edition (Core Rules).md");

/// Local mirror of `arm_rules::spell::SpellsFile`, which is `pub(crate)` and
/// so not reachable from this integration-test crate.
#[derive(Deserialize)]
struct SpellsFile {
    spells: Vec<Spell>,
}

fn load_spells() -> Vec<Spell> {
    let file: SpellsFile =
        serde_json::from_str(SPELLS_JSON).expect("rules/core/spells.json must parse as JSON");
    file.spells
}

// ---------------------------------------------------------------------------
// Technique/Form word <-> catalogue id
// ---------------------------------------------------------------------------

fn technique_id(word: &str) -> Option<&'static str> {
    Some(match word {
        "Creo" => "art.creo",
        "Intellego" => "art.intellego",
        "Muto" => "art.muto",
        "Perdo" => "art.perdo",
        "Rego" => "art.rego",
        _ => return None,
    })
}

fn form_id(word: &str) -> Option<&'static str> {
    Some(match word {
        "Animal" => "art.animal",
        "Aquam" => "art.aquam",
        "Auram" => "art.auram",
        "Corpus" => "art.corpus",
        "Herbam" => "art.herbam",
        "Ignem" => "art.ignem",
        "Imaginem" => "art.imaginem",
        "Mentem" => "art.mentem",
        "Terram" => "art.terram",
        "Vim" => "art.vim",
        _ => return None,
    })
}

/// A requisite may name either a Technique or a Form (`Req: Rego`,
/// `Req: Terram`, `Req: Imaginem, Rego`).
fn technique_or_form_id(word: &str) -> Option<&'static str> {
    technique_id(word).or_else(|| form_id(word))
}

// ---------------------------------------------------------------------------
// Heading-context pass: which Technique/Form/Level a given `#####` heading
// line falls under.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LevelCtx {
    /// No `#### LEVEL`/`#### GENERAL` heading has been seen since the last
    /// Technique/Form section heading.
    Unset,
    General,
    Fixed(u8),
}

#[derive(Clone, Copy, Debug)]
struct HeadingContext {
    technique: Option<&'static str>,
    form: Option<&'static str>,
    level: LevelCtx,
}

fn section_heading_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^#{1,6}\s+(Creo|Intellego|Muto|Perdo|Rego)\s+(Animal|Aquam|Auram|Corpus|Herbam|Ignem|Imaginem|Mentem|Terram|Vim)\b",
        )
        .unwrap()
    })
}

fn level_heading_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^#{1,6}\s*LEVEL\s+(\d+)\s*$").unwrap())
}

fn general_heading_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^#{1,6}\s*GENERAL\s*$").unwrap())
}

/// One forward pass over the whole source file, recording the Technique/Form/
/// Level context in effect at every `##### ` (spell-name) heading line. A
/// one-pass index rather than per-spell backward scanning, so every spell's
/// lookup is by its own cited heading line number and cannot disagree with
/// another spell's.
fn build_heading_contexts(source: &str) -> BTreeMap<usize, HeadingContext> {
    let section_re = section_heading_regex();
    let level_re = level_heading_regex();
    let general_re = general_heading_regex();

    let mut technique: Option<&'static str> = None;
    let mut form: Option<&'static str> = None;
    let mut level = LevelCtx::Unset;
    let mut out = BTreeMap::new();

    for (idx, line) in source.lines().enumerate() {
        let line_no = idx + 1;
        if let Some(caps) = section_re.captures(line) {
            technique = technique_id(&caps[1]);
            form = form_id(&caps[2]);
            level = LevelCtx::Unset;
            continue;
        }
        if let Some(caps) = level_re.captures(line) {
            if let Ok(n) = caps[1].parse::<u8>() {
                level = LevelCtx::Fixed(n);
            }
            continue;
        }
        if general_re.is_match(line) {
            level = LevelCtx::General;
            continue;
        }
        if line.starts_with("##### ") {
            out.insert(
                line_no,
                HeadingContext {
                    technique,
                    form,
                    level,
                },
            );
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Stat-line parsing
// ---------------------------------------------------------------------------

/// Matches `R: <range>, D: <duration>, T: <target>` tolerantly: a missing
/// comma (space only), a trailing period instead of a comma, and a
/// `<duration-or-range> & <word>` combined token (Mists of Change's
/// "D: Sun & Year", ArMDE:13634) all still match. Does not anchor the end of
/// the line, so trailing junk — a ", Ritual" suffix, or (for
/// `spell.thaumaturgical_transformation_of_plants_to_iron`, ArMDE:14016) an
/// inline "Reg: Terram" the book ran onto the same line with no separating
/// comma — is simply left unconsumed.
fn stat_line_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^R:\s*(?P<range>[A-Za-z]+(?:\s*&\s*[A-Za-z]+)?)\s*[,.]?\s*D:\s*(?P<duration>[A-Za-z]+(?:\s*&\s*[A-Za-z]+)?)\s*[,.]?\s*T:\s*(?P<target>[A-Za-z]+)",
        )
        .unwrap()
    })
}

/// Matches a `Req:`/`Reg:` requisite declaration anywhere in a line (not
/// anchored to the line start), so it also catches the one spell where the
/// book runs it onto the end of the stat line itself with no preceding
/// comma (ArMDE:14016). `Reg:` is the book's own typo for `Req:` (see module
/// doc); both are treated as the same marker.
fn requisites_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:Req|Reg):\s*([A-Za-z]+(?:\s*,\s*[A-Za-z]+)*)").unwrap())
}

fn normalize_range(token: &str) -> Option<SpellRange> {
    Some(match token {
        "Personal" | "Per" => SpellRange::Personal,
        "Touch" => SpellRange::Touch,
        "Eye" | "Eve" => SpellRange::Eye,
        "Voice" => SpellRange::Voice,
        "Sight" => SpellRange::Sight,
        "Arc" | "Arcane" => SpellRange::ArcaneConnection,
        _ => return None,
    })
}

/// `Some(None)` means the book states a duration the fixed enum has no slot
/// for ("Special"/"Spec") — `rules/core/spells.json` omits the field for
/// these, so the expected data value is absence, not an error.
fn normalize_duration(raw: &str) -> Option<Option<SpellDuration>> {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("sun & year") {
        // Mists of Change, ArMDE:13634 — see module doc.
        return Some(Some(SpellDuration::Year));
    }
    Some(match trimmed {
        "Momentary" | "Mom" => Some(SpellDuration::Momentary),
        "Concentration" | "Conc" => Some(SpellDuration::Concentration),
        "Diameter" | "Diam" => Some(SpellDuration::Diameter),
        "Sun" => Some(SpellDuration::Sun),
        "Ring" => Some(SpellDuration::Ring),
        "Moon" => Some(SpellDuration::Moon),
        "Year" => Some(SpellDuration::Year),
        "Special" | "Spec" => None,
        _ => return None,
    })
}

/// `Some(None)` — see [`normalize_duration`]; the same "Special"/"Spec" shape
/// occurs for Target (`spell.trackless_step`, `spell.the_earth_split_asunder`).
fn normalize_target(raw: &str) -> Option<Option<SpellTarget>> {
    Some(match raw.trim() {
        "Individual" | "Ind" => Some(SpellTarget::Individual),
        "Circle" => Some(SpellTarget::Circle),
        "Part" => Some(SpellTarget::Part),
        "Group" => Some(SpellTarget::Group),
        "Room" => Some(SpellTarget::Room),
        "Structure" | "Str" => Some(SpellTarget::Structure),
        "Boundary" | "Bound" => Some(SpellTarget::Boundary),
        "Taste" => Some(SpellTarget::Taste),
        "Touch" => Some(SpellTarget::Touch),
        "Smell" => Some(SpellTarget::Smell),
        "Hearing" => Some(SpellTarget::Hearing),
        "Vision" => Some(SpellTarget::Vision),
        "Special" | "Spec" => None,
        _ => return None,
    })
}

/// Every Technique/Form id named by a `Req:`/`Reg:` line anywhere in
/// `range_lines`, sorted and de-duplicated for set comparison (requisite
/// *order* is not a checked fact — see module doc).
fn book_requisites(range_lines: &[&str]) -> Vec<&'static str> {
    let re = requisites_regex();
    let mut found = Vec::new();
    for line in range_lines {
        if let Some(caps) = re.captures(line) {
            for word in caps[1].split(',') {
                if let Some(id) = technique_or_form_id(word.trim()) {
                    found.push(id);
                }
            }
        }
    }
    found.sort_unstable();
    found.dedup();
    found
}

// ---------------------------------------------------------------------------
// Per-spell comparison
// ---------------------------------------------------------------------------

/// Every way `spell` disagrees with what the book states at its own cited
/// `source.lines`. Empty means a clean match.
fn describe_mismatches(
    spell: &Spell,
    contexts: &BTreeMap<usize, HeadingContext>,
    md_lines: &[&str],
) -> Vec<String> {
    let mut issues = Vec::new();

    let Some(source) = &spell.source else {
        issues.push("has no `source` field to check against".to_string());
        return issues;
    };

    let start = source.lines.start as usize;
    let end = source.lines.end as usize;
    if start == 0 || end == 0 || start > end || end > md_lines.len() {
        issues.push(format!(
            "cited lines [{start}, {end}] are out of bounds for the source file"
        ));
        return issues;
    }
    let range_lines = &md_lines[start - 1..end];

    match contexts.get(&start) {
        None => issues.push(format!(
            "heading line {start} is not inside any recognized Technique/Form section \
             (no `##### ` heading found there under a parsed `### <Technique> <Form>` section)"
        )),
        Some(ctx) => {
            match ctx.technique {
                Some(expected) if expected == spell.technique.as_str() => {}
                Some(expected) => issues.push(format!(
                    "technique: book section says `{expected}`, data has `{}`",
                    spell.technique.as_str()
                )),
                None => issues.push("book section's Technique word was not recognized".to_string()),
            }
            match ctx.form {
                Some(expected) if expected == spell.form.as_str() => {}
                Some(expected) => issues.push(format!(
                    "form: book section says `{expected}`, data has `{}`",
                    spell.form.as_str()
                )),
                None => issues.push("book section's Form word was not recognized".to_string()),
            }
            match ctx.level {
                LevelCtx::Unset => {
                    issues.push("no LEVEL/GENERAL heading seen before this spell".to_string())
                }
                LevelCtx::General => {
                    if let Some(n) = spell.level {
                        issues.push(format!(
                            "level: book has this spell under GENERAL (expects no level), data has `{n}`"
                        ));
                    }
                }
                LevelCtx::Fixed(n) => {
                    if spell.level != Some(n) {
                        issues.push(format!(
                            "level: book says LEVEL {n}, data has `{:?}`",
                            spell.level
                        ));
                    }
                }
            }
        }
    }

    match range_lines.iter().find(|line| line.starts_with("R:")) {
        None => {
            issues.push("no 'R: ..., D: ..., T: ...' stat line found in cited range".to_string())
        }
        Some(line) => match stat_line_regex().captures(line) {
            None => issues.push(format!("stat line did not parse: {line:?}")),
            Some(caps) => {
                let range_tok = &caps["range"];
                let dur_tok = &caps["duration"];
                let tgt_tok = &caps["target"];

                match normalize_range(range_tok) {
                    None => issues.push(format!("unrecognized Range token `{range_tok}`")),
                    Some(expected) => {
                        if spell.range != Some(expected) {
                            issues.push(format!(
                                "range: book says `{range_tok}` ({expected:?}), data has `{:?}`",
                                spell.range
                            ));
                        }
                    }
                }
                match normalize_duration(dur_tok) {
                    None => issues.push(format!("unrecognized Duration token `{dur_tok}`")),
                    Some(expected) => {
                        if spell.duration != expected {
                            issues.push(format!(
                                "duration: book says `{dur_tok}` ({expected:?}), data has `{:?}`",
                                spell.duration
                            ));
                        }
                    }
                }
                match normalize_target(tgt_tok) {
                    None => issues.push(format!("unrecognized Target token `{tgt_tok}`")),
                    Some(expected) => {
                        if spell.target != expected {
                            issues.push(format!(
                                "target: book says `{tgt_tok}` ({expected:?}), data has `{:?}`",
                                spell.target
                            ));
                        }
                    }
                }

                let book_ritual = line.contains("Ritual");
                if book_ritual != spell.ritual {
                    issues.push(format!(
                        "ritual: book {} \"Ritual\" on the stat line, data has ritual={}",
                        if book_ritual {
                            "states"
                        } else {
                            "does not state"
                        },
                        spell.ritual
                    ));
                }
            }
        },
    }

    let expected_requisites = book_requisites(range_lines);
    let mut actual_requisites: Vec<&str> = spell.requisites.iter().map(Id::as_str).collect();
    actual_requisites.sort_unstable();
    if expected_requisites != actual_requisites {
        issues.push(format!(
            "requisites: book states {expected_requisites:?}, data has {actual_requisites:?}"
        ));
    }

    issues
}

// ---------------------------------------------------------------------------
// Ruled exceptions
// ---------------------------------------------------------------------------

/// Entries whose mismatch is a known, ruled departure rather than a defect.
/// Each row's reasoning cites the ruling; [`ruled_exceptions_still_mismatch`]
/// asserts every row still actually mismatches, so the list can only shrink —
/// the same shape as `uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`.
const RULED_EXCEPTIONS: &[(&str, &str)] = &[(
    "spell.piercing_the_magical_veil",
    "D68 item 6 (docs/vf-audit/decisions.md): this catalogue entry's `source.lines` \
     ([1745, 1745]) cites a sample character's spell-list bullet (\"Piercing the \
     Magical Veil ... (see Piercing the Faerie Veil)\"), which carries no heading, no \
     Technique/Form section, and no R:/D:/T: stat line — there is nothing at that \
     citation for this parser to parse. D68.6 rules that the level (InVi 20, from the \
     same bullet) and Range/Duration/Target are inferred from its named sibling \
     `spell.piercing_the_faerie_veil` (ArMDE:15707-15710) instead, and records the \
     inference in RULES.md rather than in a parseable passage.",
)];

fn ruled_exception_ids() -> HashSet<&'static str> {
    RULED_EXCEPTIONS.iter().map(|(id, _)| *id).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn every_spell_matches_its_cited_stat_line() {
    let spells = load_spells();
    let contexts = build_heading_contexts(CORE_RULES_MD);
    let md_lines: Vec<&str> = CORE_RULES_MD.lines().collect();
    let exceptions = ruled_exception_ids();

    let mut failures = Vec::new();
    for spell in &spells {
        if exceptions.contains(spell.id.as_str()) {
            continue;
        }
        let issues = describe_mismatches(spell, &contexts, &md_lines);
        if !issues.is_empty() {
            failures.push(format!("{}: {}", spell.id.as_str(), issues.join("; ")));
        }
    }

    assert!(
        failures.is_empty(),
        "{} spell(s) disagree with their own cited stat line:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Every [`RULED_EXCEPTIONS`] row must still trip the screen — if it no
/// longer does (the data was fixed, or the citation was repointed), the row
/// is stale and must be removed rather than silently kept around.
#[test]
fn ruled_exceptions_still_mismatch() {
    let spells = load_spells();
    let contexts = build_heading_contexts(CORE_RULES_MD);
    let md_lines: Vec<&str> = CORE_RULES_MD.lines().collect();

    for (id, reason) in RULED_EXCEPTIONS {
        let spell = spells
            .iter()
            .find(|s| s.id.as_str() == *id)
            .unwrap_or_else(|| panic!("RULED_EXCEPTIONS names `{id}`, not in the catalogue"));
        let issues = describe_mismatches(spell, &contexts, &md_lines);
        assert!(
            !issues.is_empty(),
            "RULED_EXCEPTIONS entry `{id}` no longer mismatches its cited stat line — remove \
             it from the list (reason on file: {reason})"
        );
    }
}

/// Sanity floor: this file's whole premise depends on actually walking real
/// book headings. If the source file's layout ever changed so drastically
/// that no Technique/Form section matched at all, every other assertion here
/// would pass vacuously (every spell would fail identically on "no context"
/// rather than on anything meaningful) — this test would catch that.
#[test]
fn heading_context_pass_finds_the_known_sections() {
    let contexts = build_heading_contexts(CORE_RULES_MD);
    assert!(
        contexts.len() > 300,
        "expected to index >300 '##### ' spell headings inside Technique/Form sections, found \
         {} — the heading-context pass may no longer be matching the book's layout",
        contexts.len()
    );
}
