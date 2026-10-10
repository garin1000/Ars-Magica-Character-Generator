//! Spell description verbatim-fidelity guard (P8-5,
//! `docs/vf-audit/design-p8-resync.md` § "What moves, and how each is
//! re-derived" (f); D85 Q6). Mirrors `x2_reclassification.rs`'s own
//! `x2_shipped_descriptions_match_their_cited_passage_verbatim` for the
//! Virtues/Flaws catalogue, but for `rules/core/spells.json`: no equivalent
//! guard existed for a spell's `description` text before this file —
//! `spell_stat_line.rs` checks only the mechanical stat line
//! (Technique/Form/Level/R/D/T/Ritual/requisites) and explicitly declares
//! prose out of its own scope.
//!
//! Every spell's shipped `description`, in both locales, must be the book's
//! own prose for that spell — not a paraphrase — subject to the same
//! normalizations `x2_reclassification.rs` allows for Virtues/Flaws (an en
//! dash or the mathematical minus sign immediately before a digit becomes the
//! ASCII hyphen-minus, and Markdown emphasis/link syntax collapses to its
//! inner text). Running this for the first time against the P8-resynced
//! catalogue found three upstream OCR fixes already transcribed verbatim
//! (`spell.aegis_of_the_hearth` "vears"->"years",
//! `spell.steed_of_vengeance`/`spell.the_falcons_hood` "eves"->"eyes"), a
//! genuine upstream German unit-convention change (metric "Meter" to the
//! traditional "Schritt"/pace) across most of the German catalogue, and
//! several pre-existing defects the resync merely exposed — see
//! `regenerate_spell_descriptions`'s own doc comment for the full breakdown.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use arm_rules::Spell;
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use regex::Regex;
use serde::Deserialize;

const CORE_RULES_FILE_EN: &str = "Ars Magica - Definitive Edition (Core Rules).md";
const CORE_RULES_FILE_DE: &str = "Ars Magica Definitive Edition Basisregeln.md";
const EN_SPELLS: &str = include_str!("../../../rules/i18n/en/spells.json");
const DE_SPELLS: &str = include_str!("../../../rules/i18n/de/spells.json");
const SPELLS_JSON: &str = include_str!("../../../rules/core/spells.json");
const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

fn rules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules")
}

fn tmp_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp")
}

/// Local mirror of `arm_rules::spell::SpellsFile`, which is `pub(crate)` and
/// so not reachable from this integration-test crate (same shape as
/// `spell_stat_line.rs`'s own copy).
#[derive(Deserialize)]
struct SpellsFile {
    spells: Vec<Spell>,
}

fn load_spells() -> Vec<Spell> {
    let file: SpellsFile =
        serde_json::from_str(SPELLS_JSON).expect("rules/core/spells.json must parse as JSON");
    file.spells
}

/// A minimal ruleset, spells excluded on purpose: `LocalizedRuleset::description`
/// reads only the i18n map, never the ruleset's own catalogue, so a `Ruleset`
/// with no `spells` source loaded is sufficient here (same minimal shape
/// `x2_reclassification.rs::load_ruleset` uses for the Virtues/Flaws-only
/// case).
fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        houses: Some(SHIPPED_HOUSES),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .unwrap()
}

// ---------------------------------------------------------------------------
// Markdown/quote normalization — duplicated from x2_reclassification.rs
// (each `tests/*.rs` file is its own crate, so sharing is by copy, matching
// that file's own precedent for e.g. `SpellsFile`).
// ---------------------------------------------------------------------------

static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\((?:<[^>]*>|[^)]*)\)").unwrap());
static EMPHASIS_BOLD_STAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*([^*\n]+)\*\*").unwrap());
static EMPHASIS_STAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*([^*\n]+)\*").unwrap());
static EMPHASIS_UNDERSCORE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"_([^_\n]+)_").unwrap());
static GERMAN_QUOTE_PAIR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("„([^„\"“]*)\"").unwrap());
static EN_DASH_BEFORE_DIGIT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\u{2013}(\d)").unwrap());
// The German book also uses the mathematical minus sign (U+2212, not the en
// dash U+2013) before a digit in running prose (e.g. ArMDE:14886 "erleidet
// −3 auf..."); CLAUDE.md's "Negative signs are ASCII hyphens" rule applies
// the same fold to it as to the en dash.
static MINUS_SIGN_BEFORE_DIGIT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\u{2212}(\d)").unwrap());

/// Folds each `„…"` pair to `„…“`, the shipped i18n's normalised form — see
/// `x2_reclassification.rs::normalize_quote_pairs` (same logic, same reason:
/// the German sources close `„` with an ASCII `"` as house style, and shipped
/// i18n normalises the pair on the way out).
fn normalize_quote_pairs(s: &str) -> String {
    GERMAN_QUOTE_PAIR_RE.replace_all(s, "„$1“").into_owned()
}

/// The only two transformations a shipped `description` may apply to its
/// cited passage: an en dash (U+2013) immediately before an ASCII digit
/// becomes the ASCII hyphen-minus, and Markdown emphasis/link syntax
/// collapses to its inner text. See `x2_reclassification.rs::normalize_markdown`.
fn normalize_markdown(s: &str) -> String {
    let s = LINK_RE.replace_all(s, "$1");
    let s = EMPHASIS_BOLD_STAR_RE.replace_all(&s, "$1");
    let s = EMPHASIS_STAR_RE.replace_all(&s, "$1");
    let s = EMPHASIS_UNDERSCORE_RE.replace_all(&s, "$1");
    let s = EN_DASH_BEFORE_DIGIT_RE.replace_all(&s, "-$1");
    MINUS_SIGN_BEFORE_DIGIT_RE
        .replace_all(&s, "-$1")
        .into_owned()
}

#[cfg(test)]
mod normalize_markdown_tests {
    use super::normalize_markdown;

    #[test]
    fn folds_the_mathematical_minus_sign_before_a_digit_to_an_ascii_hyphen() {
        // ArMDE:14886, `spell.confusion_of_the_insane_vibrations`: the German
        // book uses U+2212 (MINUS SIGN), not U+2013 (en dash), before a
        // digit. Confirmed byte-for-byte (`e2 88 92`, not `e2 80 93`).
        assert_eq!(
            normalize_markdown("erleidet \u{2212}3 auf Angriffswerte"),
            "erleidet -3 auf Angriffswerte"
        );
    }

    #[test]
    fn still_folds_the_en_dash_before_a_digit() {
        assert_eq!(normalize_markdown("a \u{2013}6 penalty"), "a -6 penalty");
    }
}

// ---------------------------------------------------------------------------
// Source reading
// ---------------------------------------------------------------------------

/// Every line of `file` under `rules/source/<lang>/`, cached per `(lang, file)`
/// pair (same shape as `x2_reclassification.rs::cached_source_lines`).
fn cached_source_lines<'a>(
    cache: &'a mut BTreeMap<(String, String), Vec<String>>,
    lang: &str,
    file: &str,
) -> &'a [String] {
    cache
        .entry((lang.to_string(), file.to_string()))
        .or_insert_with(|| {
            let path = rules_dir().join("source").join(lang).join(file);
            fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()))
                .lines()
                .map(str::to_string)
                .collect()
        })
}

/// The verbatim prose a spell's `[start, end]` citation (1-indexed, inclusive,
/// exactly as shipped in `source.lines`) brackets.
///
/// **What the citation excludes, and why.** `start` is always the
/// `##### <Spell Name>` heading line, and `start + 1` is always the
/// `R: ..., D: ..., T: ...` stat line (optionally carrying an inline
/// `, Ritual` or a run-on `Req:`/`Reg:` requisite, ArMDE:14090) — neither is
/// prose. A standalone `Req:`/`Reg:` line (its own line between the stat line
/// and the description, ArMDE:13294 style) is excluded the same way, wherever
/// it falls. A standalone trailing level-derivation clause — `(Base N[, +mod
/// ...])`, `(Special spell)`, `(Mercurian Ritual)`, `(Grundstufe ...)`,
/// `(Grundzauber)`, … — is excluded too, matched structurally (opens the
/// line with `(`, checked only in last position) rather than by an
/// enumerated phrase list — see `opens_with_a_parenthesis`'s own doc comment
/// for why. Everything else is grouped into paragraphs on blank-line
/// boundaries —
/// a multi-paragraph Ritual's prose, e.g. `spell.aegis_of_the_hearth`, spans
/// several — each line's trailing `<br>` stripped before joining, then the
/// whole body is [`normalize_markdown`]-ed.
fn spell_verbatim_body(lines: &[String], start: u32, end: u32) -> String {
    let body_start = start as usize + 2; // 1-indexed: skip heading + stat line
    let body_end = end as usize;
    assert!(
        body_end <= lines.len() && body_start <= body_end + 1,
        "citation [{start}, {end}] (body [{body_start}, {body_end}]) out of bounds for a \
         {}-line file",
        lines.len()
    );
    let slice = if body_start > body_end {
        &[]
    } else {
        &lines[(body_start - 1)..body_end]
    };

    // English: "Req:"/"Reg:" (ArMDE:13294's own typo for "Req:"). German:
    // "Req.:" (ArMDE:14251 "Req.: Intellego") — note the period before the
    // colon, which is not present in the English form.
    fn is_requisite_only_line(line: &str) -> bool {
        line.starts_with("Req:") || line.starts_with("Reg:") || line.starts_with("Req.:")
    }
    // The book's level-derivation annotation after a spell's prose takes
    // several fixed shapes — "(Base N[, +mod ...])", "(Base spell)",
    // "(Special spell)", "(Mercurian Ritual)", "(Variable base)" in English;
    // "(Grundstufe N[, +mod ...])", "(Grundzauber)" in German — too open-ended
    // a vocabulary to enumerate by phrase. What every one of them shares, and
    // no genuine prose sentence does, is opening its own line with `(` in
    // last position (checked only there, never mid-body, so an ordinary
    // parenthetical remark elsewhere in the prose is untouched). Matched by
    // prefix only, not requiring the line to also end on `)`: ArMDE:12705
    // runs "(Grundstufe 15, +2 Stimme)" directly into the next section's
    // heading with no line break ("...+2 Stimme)### Rego Animal —
    // Leitlinien"), a markdown defect in the upstream book itself (not a
    // citation-boundary bug — `spell.cripple_the_howling_wolf`'s own citation
    // ends exactly on this line) — `rules/source/` may not be hand-repaired
    // (`CLAUDE.md`), so the whole line is dropped rather than requiring a
    // clean close.
    fn opens_with_a_parenthesis(line: &str) -> bool {
        line.starts_with('(')
    }
    fn flush(current: &mut Vec<&str>, paragraphs: &mut Vec<String>) {
        if !current.is_empty() {
            paragraphs.push(current.join(" "));
            current.clear();
        }
    }

    let mut content_lines: Vec<&str> = Vec::new();
    for raw_line in slice {
        let trimmed = raw_line.trim();
        let content = trimmed.strip_suffix("<br>").unwrap_or(trimmed).trim();
        if content.is_empty() {
            content_lines.push("");
            continue;
        }
        if is_requisite_only_line(content) {
            continue;
        }
        content_lines.push(content);
    }
    // Drop a trailing level-derivation annotation, and any blank lines after
    // it (`cripple_the_howling_wolf`'s run-on case leaves none, but a clean
    // citation's final line is usually followed by the range's own trailing
    // blank — which the citation already excludes by ending `end` on content,
    // per D30.2 — so this is a defensive strip, not the common case).
    while matches!(content_lines.last(), Some(&"")) {
        content_lines.pop();
    }
    if matches!(content_lines.last(), Some(line) if opens_with_a_parenthesis(line)) {
        content_lines.pop();
    }

    let mut paragraphs: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for content in content_lines {
        if content.is_empty() {
            flush(&mut current, &mut paragraphs);
            continue;
        }
        current.push(content);
    }
    flush(&mut current, &mut paragraphs);

    normalize_markdown(&paragraphs.join("\n\n"))
}

#[cfg(test)]
mod spell_verbatim_body_tests {
    use super::spell_verbatim_body;

    fn lines(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn extracts_the_prose_only_skipping_heading_stat_line_and_base_clause() {
        let l = lines(&[
            "##### Test Spell",
            "R: Touch, D: Sun, T: Ind<br>",
            "This is the spell's effect.<br>",
            "(Base 5, +1 Touch)",
        ]);
        assert_eq!(spell_verbatim_body(&l, 1, 4), "This is the spell's effect.");
    }

    #[test]
    fn keeps_a_parenthetical_remark_that_is_not_in_last_position() {
        // `opens_with_a_parenthesis` is checked only on the final content
        // line, so an ordinary parenthetical aside mid-paragraph (unlike
        // `spell.enchantment_of_the_scrying_pool`'s trailing "(Special
        // spell)") is prose, not a level-derivation clause, and stays.
        let l = lines(&[
            "##### Test Spell",
            "R: Touch, D: Sun, T: Ind<br>",
            "This spell (an old Mercurian one) does something.<br>",
            "It has no further effect.",
        ]);
        assert_eq!(
            spell_verbatim_body(&l, 1, 4),
            "This spell (an old Mercurian one) does something. It has no further effect."
        );
    }

    #[test]
    fn excludes_an_arbitrarily_worded_trailing_annotation() {
        // The book's level-derivation vocabulary is open-ended ("(Special
        // spell)", "(Mercurian Ritual)", "(Variable base)", ArMDE:12913,
        // :15655, :13551 — none sharing a word with "Base"), so the match is
        // structural (opens the line with `(`), not an enumerated phrase.
        let l = lines(&[
            "##### Test Spell",
            "R: Touch, D: Sun, T: Ind<br>",
            "This is the spell's effect.<br>",
            "(An arbitrarily worded annotation)",
        ]);
        assert_eq!(spell_verbatim_body(&l, 1, 4), "This is the spell's effect.");
    }

    #[test]
    fn excludes_a_base_clause_even_when_the_book_runs_the_next_heading_onto_the_same_line() {
        // ArMDE:12705, `spell.cripple_the_howling_wolf`: the upstream book
        // concatenates "(Grundstufe 15, +2 Stimme)" directly onto
        // "### Rego Animal — Leitlinien" with no line break — a defect in
        // `rules/source/` itself, which may not be hand-repaired, so the
        // extraction tolerates it instead.
        let l = lines(&[
            "##### Test Spell",
            "R: Touch, D: Sun, T: Ind<br>",
            "This is the spell's effect.<br>",
            "(Base 5, +1 Touch)### Next Section Heading",
        ]);
        assert_eq!(spell_verbatim_body(&l, 1, 4), "This is the spell's effect.");
    }

    #[test]
    fn excludes_the_german_base_spell_marker() {
        let l = lines(&[
            "##### Testzauber",
            "R: Berührung, D: Ring, Z: Kreis",
            "Dies ist die Wirkung des Zaubers.",
            "(Grundzauber)",
        ]);
        assert_eq!(
            spell_verbatim_body(&l, 1, 4),
            "Dies ist die Wirkung des Zaubers."
        );
    }

    #[test]
    fn skips_the_german_base_clause_and_requisite_spellings() {
        // Found by a real mismatch in `report_spell_description_staleness`'s
        // first run: the German book spells Base as "(Grundstufe ...)" and
        // Requisite as "Req.:" (with a period), neither of which the
        // English-only patterns caught, so 300+ German spells falsely
        // "mismatched" by carrying their own Base clause into the body.
        let l = lines(&[
            "##### Testzauber",
            "R: Berührung, D: Sonne, Z: Einzelperson",
            "Req.: Terram",
            "Dies ist die Wirkung des Zaubers.",
            "(Grundstufe 5, +1 Berührung)",
        ]);
        assert_eq!(
            spell_verbatim_body(&l, 1, 5),
            "Dies ist die Wirkung des Zaubers."
        );
    }

    #[test]
    fn skips_a_standalone_requisite_line_wherever_it_falls() {
        let l = lines(&[
            "##### Test Spell",
            "R: Touch, D: Sun, T: Ind<br>",
            "Req: Terram<br>",
            "This is the spell's effect.<br>",
        ]);
        assert_eq!(spell_verbatim_body(&l, 1, 4), "This is the spell's effect.");
    }

    #[test]
    fn joins_multiple_paragraphs_on_blank_line_boundaries() {
        let l = lines(&[
            "##### Test Ritual",
            "R: Touch, D: Year, T: Bound, Ritual<br>",
            "First paragraph.",
            "",
            "Second paragraph.",
        ]);
        assert_eq!(
            spell_verbatim_body(&l, 1, 5),
            "First paragraph.\n\nSecond paragraph."
        );
    }

    #[test]
    fn a_spell_with_no_base_clause_keeps_its_last_paragraph() {
        let l = lines(&[
            "##### Test Ritual",
            "R: Touch, D: Year, T: Bound, Ritual<br>",
            "Only paragraph, no Base line follows.",
        ]);
        assert_eq!(
            spell_verbatim_body(&l, 1, 3),
            "Only paragraph, no Base line follows."
        );
    }
}

// ---------------------------------------------------------------------------
// Diagnostic report (run manually after a rulebook re-sync)
// ---------------------------------------------------------------------------

/// Writes `tmp/p8-5-diff-report.md`: every spell id / locale whose shipped
/// `description` does not match [`spell_verbatim_body`] at its own current
/// `source.lines`, with the shipped (presumed stale) text and the current
/// source's text side by side. `#[ignore]`d because it writes into `tmp/`,
/// like the P8 sweep tools in `source_relocation.rs`/`rulebook_citations.rs`.
/// Ids whose cited `source.lines` carries no book prose to compare against —
/// same shape and same single entry as `spell_stat_line.rs::RULED_EXCEPTIONS`
/// (D68 item 6, `docs/vf-audit/decisions.md`): `spell.piercing_the_magical_veil`
/// cites a sample character's spell-list bullet ([1813, 1813], a single line
/// with no heading and no stat line), and its shipped `description` is an
/// editorial note explaining the absence, not a book quote.
const NOT_A_BOOK_PASSAGE: &[&str] = &["spell.piercing_the_magical_veil"];

/// Ids whose citation range includes a Markdown table (a roll-result lookup)
/// that is not prose and is not reproduced in the shipped `description` —
/// the same shape as `x2_reclassification.rs`'s own precedent for
/// `flaw.form_monstrosity` ("its citation's range includes the ... table,
/// which is not prose and is not reproduced in the shipped description").
/// `spell_verbatim_body` has no generic table-stripping logic (a one-off
/// formatting quirk, not a systematic rule), so these three are named
/// exceptions rather than guessed at generically.
const TABLE_NOT_REPRODUCED: &[&str] = &[
    "spell.mists_of_change",
    "spell.the_shadow_of_life_renewed",
    "spell.visions_of_the_infernal_terrors",
];

#[test]
#[ignore = "diagnostic report only, writes tmp/p8-5-diff-report.md"]
fn report_spell_description_staleness() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_SPELLS).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_SPELLS).unwrap();
    let spells = load_spells();
    let mut cache: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    let mut rows = Vec::new();
    for spell in &spells {
        if NOT_A_BOOK_PASSAGE.contains(&spell.id.as_str())
            || TABLE_NOT_REPRODUCED.contains(&spell.id.as_str())
        {
            continue;
        }
        let Some(source) = &spell.source else {
            continue;
        };
        let start = source.lines.start;
        let end = source.lines.end;
        for (lang, loc, source_file) in [
            ("en", &loc_en, CORE_RULES_FILE_EN),
            ("de", &loc_de, CORE_RULES_FILE_DE),
        ] {
            let Some(shipped) = loc.description(&spell.id) else {
                continue;
            };
            let lines = cached_source_lines(&mut cache, lang, source_file);
            let expected = normalize_quote_pairs(&spell_verbatim_body(lines, start, end));
            if shipped != expected {
                rows.push(format!(
                    "## {} / {lang} (lines {start}-{end})\n\n- shipped (stale): {shipped:?}\n- \
                     current source:  {expected:?}\n",
                    spell.id.as_str()
                ));
            }
        }
    }

    let report = format!(
        "# P8-5 spell description staleness report\n\n{} mismatch(es) found against the \
         post-resync rulebook.\n\n{}\n",
        rows.len(),
        rows.join("\n")
    );
    fs::write(tmp_dir().join("p8-5-diff-report.md"), report).expect("tmp/ is writable");
}

// ---------------------------------------------------------------------------
// Regenerator (run deliberately, once, to apply the report's findings)
// ---------------------------------------------------------------------------

/// The first unescaped `"` in `s`, i.e. the end of a JSON string value whose
/// opening quote is not itself included in `s`.
fn find_unescaped_quote(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

/// Rewrites the `"description"` value of exactly the ids named in `updates`,
/// inside `rules/i18n/<lang>/spells.json`'s canonical one-line-per-entry
/// shape: `"spell.<id>": {"name": "...", "description": "..."}` (verified:
/// every such line's `"description"` is its last field, immediately followed
/// by the entry's closing `}` and nothing else before the trailing `,` or
/// newline). A targeted substring replace rather than a `serde_json`
/// round-trip — re-serializing the whole document would reflow every entry
/// and bury the real diff, the same reason
/// `x2_reclassification.rs::rewrite_description_lines` gives for
/// virtues_flaws.json's own (one-field-per-line) shape.
fn rewrite_spell_description_lines(
    text: &str,
    updates: &BTreeMap<String, String>,
) -> (String, usize) {
    let mut replaced = 0usize;
    let mut out_lines = Vec::new();

    for line in text.lines() {
        let id = line
            .trim_start()
            .strip_prefix('"')
            .and_then(|rest| rest.split_once("\": {"))
            .map(|(id, _)| id);

        let rewritten = id
            .and_then(|id| updates.get(id))
            .and_then(|new_description| {
                let marker = "\"description\": \"";
                let start = line.find(marker)?;
                let value_start = start + marker.len();
                let value_end = find_unescaped_quote(&line[value_start..])?;
                let encoded = serde_json::to_string(new_description).expect("a string serializes");
                let mut new_line = String::with_capacity(line.len());
                new_line.push_str(&line[..start]);
                new_line.push_str("\"description\": ");
                new_line.push_str(&encoded);
                new_line.push_str(&line[value_start + value_end + 1..]);
                Some(new_line)
            });

        match rewritten {
            Some(new_line) => {
                replaced += 1;
                out_lines.push(new_line);
            }
            None => out_lines.push(line.to_string()),
        }
    }
    (format!("{}\n", out_lines.join("\n")), replaced)
}

#[test]
fn rewrite_spell_description_lines_replaces_only_the_named_ids_description() {
    let text = "{\n  \"spell.foo\": {\"name\": \"Foo\", \"description\": \"Old foo text.\"},\n  \
                \"spell.bar\": {\"name\": \"Bar\", \"description\": \"Old bar text.\"}\n}\n";
    let mut updates = BTreeMap::new();
    updates.insert(
        "spell.foo".to_string(),
        "New foo text with \"quotes\".".to_string(),
    );
    let (out, count) = rewrite_spell_description_lines(text, &updates);
    assert_eq!(count, 1);
    assert!(out.contains("\"description\": \"New foo text with \\\"quotes\\\".\""));
    assert!(out.contains("\"description\": \"Old bar text.\""));
    assert!(out.contains("\"spell.foo\": {\"name\": \"Foo\""));
}

/// P8-5's fix pass (`design-p8-resync.md` § "What moves" (f)): once every
/// spell's `source.lines` pointed at its real post-sync position,
/// [`report_spell_description_staleness`] found shipped descriptions that no
/// longer match the resynced book — three upstream OCR fixes already
/// transcribed verbatim (`spell.aegis_of_the_hearth` "vears"->"years",
/// `spell.steed_of_vengeance`/`spell.the_falcons_hood` "eves"->"eyes"), a
/// genuine upstream German unit-convention change (metric "Meter" to the
/// traditional "Schritt"/pace the rest of the book uses) across most of the
/// German catalogue, and several **pre-existing** defects the resync merely
/// exposed by being the first thing to ever check spell prose against its
/// citation: a leaked `Reg: <Art>` requisite marker baked into three English
/// descriptions (`spell.hands_of_the_grasping_earth`, `spell.rain_of_stones`,
/// `spell.obliteration_of_the_metallic_barrier`), a spurious paragraph break
/// where the book has none (`spell.soothe_pains_of_the_beast`), and ten
/// English descriptions that never collapsed a self-referential `*Spell
/// Name*` italic to plain text on the way into the catalogue. Mirrors the X7a/
/// x9c/X2 precedent of this project's other `regenerate_*` tools (writing
/// corrected data back from a verified source reading) rather than
/// hand-transcribing each entry across two locales and ~90 ids.
///
/// **Never writes what it has not itself verified**: `expected` here is
/// computed by the exact same `spell_verbatim_body` + `normalize_quote_pairs`
/// pipeline the live guard
/// (`every_spell_description_matches_its_cited_passage_verbatim`) checks
/// against, so a passing guard afterward is not circular — it is the same
/// computation, re-run, confirming the file now holds what it just wrote.
#[test]
#[ignore = "writes into rules/i18n; run once to apply tmp/p8-5-diff-report.md's findings"]
fn regenerate_spell_descriptions() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_SPELLS).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_SPELLS).unwrap();
    let spells = load_spells();
    let mut cache: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    let mut updates_en: BTreeMap<String, String> = BTreeMap::new();
    let mut updates_de: BTreeMap<String, String> = BTreeMap::new();

    for spell in &spells {
        if NOT_A_BOOK_PASSAGE.contains(&spell.id.as_str())
            || TABLE_NOT_REPRODUCED.contains(&spell.id.as_str())
        {
            continue;
        }
        let Some(source) = &spell.source else {
            continue;
        };
        let start = source.lines.start;
        let end = source.lines.end;
        for (lang, loc, source_file, updates) in [
            ("en", &loc_en, CORE_RULES_FILE_EN, &mut updates_en),
            ("de", &loc_de, CORE_RULES_FILE_DE, &mut updates_de),
        ] {
            let Some(shipped) = loc.description(&spell.id) else {
                continue;
            };
            let lines = cached_source_lines(&mut cache, lang, source_file);
            let expected = normalize_quote_pairs(&spell_verbatim_body(lines, start, end));
            if shipped != expected {
                updates.insert(spell.id.as_str().to_string(), expected);
            }
        }
    }

    for (lang, updates) in [("en", &updates_en), ("de", &updates_de)] {
        if updates.is_empty() {
            continue;
        }
        let path = rules_dir().join("i18n").join(lang).join("spells.json");
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let (rewritten, count) = rewrite_spell_description_lines(&text, updates);
        assert_eq!(
            count,
            updates.len(),
            "{lang}/spells.json: expected to rewrite {} description(s), rewrote {count}",
            updates.len()
        );
        fs::write(&path, rewritten).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
}

/// D5/D46's "the shipped description is the rule's only carrier" obligation,
/// applied to spells rather than Virtues/Flaws: every spell's shipped
/// `description`, in both locales, must be the book's own cited passage —
/// see the module doc comment and [`spell_verbatim_body`]'s own doc comment
/// for what "verbatim" excludes (the heading, stat line, requisite line, and
/// trailing level-derivation clause) and what normalization is still
/// allowed. [`NOT_A_BOOK_PASSAGE`] and [`TABLE_NOT_REPRODUCED`] are the two
/// named, reasoned exceptions; everything else in the catalogue is in scope.
#[test]
fn every_spell_description_matches_its_cited_passage_verbatim() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_SPELLS).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_SPELLS).unwrap();
    let spells = load_spells();
    let mut cache: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    let mut offenders = Vec::new();
    for spell in &spells {
        if NOT_A_BOOK_PASSAGE.contains(&spell.id.as_str())
            || TABLE_NOT_REPRODUCED.contains(&spell.id.as_str())
        {
            continue;
        }
        let Some(source) = &spell.source else {
            continue;
        };
        let start = source.lines.start;
        let end = source.lines.end;
        for (lang, loc, source_file) in [
            ("en", &loc_en, CORE_RULES_FILE_EN),
            ("de", &loc_de, CORE_RULES_FILE_DE),
        ] {
            let Some(shipped) = loc.description(&spell.id) else {
                continue;
            };
            let lines = cached_source_lines(&mut cache, lang, source_file);
            let expected = normalize_quote_pairs(&spell_verbatim_body(lines, start, end));
            if shipped != expected {
                offenders.push(format!(
                    "{lang}/{} (lines {start}-{end}):\n    expected: {expected:?}\n    shipped:  \
                     {shipped:?}",
                    spell.id.as_str()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "{} shipped spell description(s) do not match their cited passage verbatim (only the \
         en-dash/minus-sign-before-digit and Markdown emphasis/link normalizations are allowed):\
         \n\n{}",
        offenders.len(),
        offenders.join("\n\n")
    );
}
