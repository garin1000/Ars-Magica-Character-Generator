//! X2 (`docs/vf-audit/phase-2-plan.md` row X2) — data-integrity tests for the
//! reclassification + description obligations D5/D8/D20/D46/D50/D61/D67
//! (`docs/vf-audit/decisions.md`) place on the Virtue/Flaw catalogue.
//!
//! These tests assert the TARGET shape of the shipped data; a slice's own rows
//! are RED until that slice's Phase 2 data pass lands (X2a's have landed — see
//! `tmp/x2a-verdicts.md` for the full per-entry citation and rationale this
//! file intentionally does not re-derive inline). This file is shared across
//! X2's sub-slices (X2a, X2b, ...); each appends its own table/tests rather
//! than duplicating this preamble.
//!
//! Most of a slice's reclassified entries need **no** test here at all: their
//! obligation is already enforced by an existing guard once the corresponding
//! row is removed from a pending list
//! (`uncomputed_clauses.rs::PENDING_DROPPED_CLAUSE`/
//! `PENDING_MECHANICAL_CLASSIFICATION`,
//! `data_integrity.rs::PENDING_D46_CLASSIFICATION`/`PENDING_D67_CLASSIFICATION`)
//! — see each slice's verdicts file for which. This file covers only entries
//! no existing guard reaches: a reclassification target with no pending-list
//! mechanism to bite on it (the entry computes nothing today and its passage
//! does not trip the mechanical-token screen), or a D20 numeric obligation the
//! generic "some displayed text exists" check cannot express.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use regex::Regex;

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

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

fn classification_of(rs: &Ruleset, id: &str) -> Classification {
    rs.items()
        .find(|item| item.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id} is not in the shipped catalogue"))
        .classification
}

/// The displayed rules text for `id` under `loc` — `description` if present,
/// else `summary` — the same precedence `VirtueFlawTab.svelte`'s tooltip
/// applies and `uncomputed_clauses.rs::displayed_rules_text_by_language`
/// checks catalogue-wide.
fn displayed_text<'a>(loc: &'a LocalizedRuleset, id: &str) -> Option<&'a str> {
    let id = Id::new(id);
    loc.description(&id).or_else(|| loc.summary(&id))
}

const EN_VF: &str = include_str!("../../../rules/i18n/en/virtues_flaws.json");
const DE_VF: &str = include_str!("../../../rules/i18n/de/virtues_flaws.json");

/// X2a (`tmp/x2-worklist.md` § 1 rows 1-51, ArMDE:3362-4061): entries that
/// compute nothing today, whose passage the mechanical-token screen does not
/// flag (so no pending list carries them), and that D8/D20/D46/D50/D65 all the
/// same require to become `uncomputed_rule` with their rule written into
/// `description` in both locales. `(id, why)` — see `tmp/x2a-verdicts.md` for
/// the full reading.
const RECLASSIFY_WITH_DESCRIPTION: &[(&str, &str)] = &[
    (
        "virtue.common_sense",
        "D50's own worked example: \"common sense (the storyguide) alerts you to the error\" \
         (ArMDE:3597-3600) — no number, no roll, but a storyguide who does not know it will not \
         do it",
    ),
    (
        "virtue.emir",
        "F-62: \"This is the same as the Knight Virtue\" (ArMDE:3743-3746) is a cross-reference \
         instruction the player must act on to get Knight's benefits; the entry itself computes \
         nothing",
    ),
    (
        "virtue.feather_messenger",
        "D8 capability: \"can painlessly separate a feather... controlling its movements \
         telepathically\" (ArMDE:3869-3872); the summary does not carry it",
    ),
    (
        "virtue.gender_shift",
        "D8 capability: \"may choose to change genders\" each midnight (ArMDE:3951-3954)",
    ),
    (
        "virtue.greater_purifying_touch",
        "D8 capability + a Fatigue cost: \"cure a single serious disease\" (ArMDE:4027-4030)",
    ),
    (
        "virtue.guest_of_house_criamon",
        "D50: \"may be created using the rules for any other House\" while remaining politically \
         Criamon (ArMDE:4037-4040) is a real creation-rules substitution; 0 effects today",
    ),
    (
        "virtue.guild_apprentice",
        "F-100: \"not able to benefit from either the Poor Flaw or the Wealthy Virtue... until he \
         moves to the journeyman rank\" (ArMDE:4041-4044); encoding the suppression itself is a \
         later slice's job, X2a owns only class+desc",
    ),
    (
        "virtue.aristotelian_training",
        "D4/D65 N6: the +1 Lab Total is conditioned on an Art and Academe activity this app can \
         never verify, so it can never legitimately fire as an in_play_effect (ArMDE:3440-3443); \
         its two sibling clauses are in the same position",
    ),
    (
        "virtue.diedne_magic",
        "D20: the required Major Story Flaw clause (\"does not grant you any points\", \
         ArMDE:3675-3682) is not computed, and the computed casting mechanic itself reaches the \
         player only as a bare surfaced label, not text",
    ),
    (
        "virtue.faerie_raised_magic",
        "D20: \"this Virtue also includes the Virtue Spell Improvisation\" (ArMDE:3829-3842) \
         grants a second Virtue's effect that is not itself present",
    ),
    (
        "virtue.the_gift",
        "D46: ArMDE:2870-2876's \"suffers all the penalties of The Gift\" is computed nowhere, \
         though the entry is named in the grog profile's forbidden_traits",
    ),
    (
        "virtue.devil_child",
        "D67 (coordinator-routed into X2a): ArMDE:3671-3674's free-Virtue-choice grant (\"gets \
         the Demonic Might or Demonic Powers... Minor Virtue free\") is computed by no effect at \
         all, though the entry's incompatible_with is computed — D67 allows uncomputed_rule \
         regardless of what else is computed",
    ),
];

#[test]
fn x2a_entries_reclassify_to_uncomputed_rule_with_a_description() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    let mut offenders = Vec::new();
    for (id, why) in RECLASSIFY_WITH_DESCRIPTION {
        let classification = classification_of(&rs, id);
        if classification != Classification::UncomputedRule {
            offenders.push(format!(
                "{id}: classified {classification:?}, expected UncomputedRule ({why})"
            ));
        }
        for (lang, loc) in [("en", &loc_en), ("de", &loc_de)] {
            if displayed_text(loc, id).is_none() {
                offenders.push(format!(
                    "{lang}/{id}: no displayed rules text at all ({why})"
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "X2a (tmp/x2a-verdicts.md): these entries must reclassify to `uncomputed_rule` and carry \
         a description in every locale — a Phase 2 data change, not yet landed:\n{}",
        offenders.join("\n")
    );
}

/// D20's worst surfaced-only entry (`docs/vf-audit/decisions.md` D20):
/// ArMDE:3579-3596 states eight rank-based Magic Resistance/Soak figures that
/// reach the player nowhere under a bare `aura_bonus` effect kind. X2a owns
/// only the reclassification and the text+numbers; the effect-kind and rank
/// parameter fix is X6's, so this test deliberately does not touch `effects`.
#[test]
fn commanding_aura_reclassifies_and_states_its_eight_figures() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.commanding_aura"),
        Classification::UncomputedRule,
        "D20: ArMDE:3585-3591's eight Pope/Cardinal/Legatus-missus/Archbishop Magic \
         Resistance/Soak figures reach the player nowhere under a bare `aura_bonus` kind; \
         virtue.commanding_aura must reclassify to uncomputed_rule"
    );

    // ArMDE:3585-3591, one (Magic Resistance, Soak bonus) pair per rank. Phase
    // 2 must write all eight numbers into `description`, both locales — checked
    // as bare numeral substrings rather than exact prose, since the wording
    // (and which locale's phrasing) is Phase 2's choice.
    const FIGURES: &[(&str, &str, &str)] = &[
        ("Pope", "25", "+5"),
        ("Cardinal/legatus a latere", "20", "+4"),
        ("Legatus missus", "15", "+3"),
        ("Archbishop", "10", "+2"),
    ];

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc) in [("en", &loc_en), ("de", &loc_de)] {
        let text = displayed_text(loc, "virtue.commanding_aura").unwrap_or_default();
        for (rank, mr, soak) in FIGURES {
            if !text.contains(mr) || !text.contains(soak) {
                offenders.push(format!(
                    "{lang}/virtue.commanding_aura: displayed text {text:?} is missing the \
                     {rank} figures (Magic Resistance {mr}, Soak bonus {soak})"
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "X2a (tmp/x2a-verdicts.md), D20's five-numbers-missing finding for commanding_aura:\n{}",
        offenders.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Verbatim-fidelity guard (fix-round Phase A, 2026-09-29)
// ---------------------------------------------------------------------------
//
// The coordinator rejected 13 of X2a's Phase 2 descriptions: rewording
// rulebook prose to give the mechanical-token screen something to recognize
// inverts the screen's whole purpose (`uncomputed_clauses.rs`'s own header:
// "a phrase list structurally cannot absolve a specific entry of a specific
// bug" — the screen exists to catch the book's own words being dropped, not
// to be satisfied by substituting different words). A shipped `description`
// must be the cited passage, not a paraphrase of it.

const CORE_RULES_FILE_EN: &str = "Ars Magica - Definitive Edition (Core Rules).md";
const CORE_RULES_FILE_DE: &str = "Ars Magica Definitive Edition Basisregeln.md";

fn rules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules")
}

// The target is either the plain `(url)` form or, when the URL itself
// contains a literal `)` (F-1 fix round: `blood_of_the_nephilim`'s and
// `commanding_aura`'s German citations wrap a page-range target around its
// own `(Überarbeitet)` parenthetical, e.g.
// `[Seiten 46–56](<...(Überarbeitet).md#wundersame-wirkungen>)`), the
// angle-bracket-wrapped `(<target>)` form — matched greedily to its own
// closing `>` rather than stopping at the first `)`, which used to land
// inside the target and leave a truncated, garbage tail unstripped. The link
// text is always capture group 1 in both branches.
static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\((?:<[^>]*>|[^)]*)\)").unwrap());
static EMPHASIS_STAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*([^*\n]+)\*").unwrap());
static EMPHASIS_UNDERSCORE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"_([^_\n]+)_").unwrap());
static EN_DASH_BEFORE_DIGIT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\u{2013}(\d)").unwrap());

/// The only two transformations a shipped `description` may apply to its
/// cited passage (`docs/open-todos.md` row 38): an en dash (U+2013)
/// immediately before an ASCII digit becomes the ASCII hyphen-minus (the
/// rulebook Markdown's own convention for a negative number, per
/// `rules_i18n_ascii_hyphen.rs`'s doc comment); Markdown emphasis (`*text*`,
/// `_text_`) and link syntax (`[text](url)`) collapse to their inner text.
/// Nothing else may differ — no synonym swap, no restructuring, no added or
/// removed clause.
fn normalize_markdown(s: &str) -> String {
    let s = LINK_RE.replace_all(s, "$1");
    let s = EMPHASIS_STAR_RE.replace_all(&s, "$1");
    let s = EMPHASIS_UNDERSCORE_RE.replace_all(&s, "$1");
    EN_DASH_BEFORE_DIGIT_RE.replace_all(&s, "-$1").into_owned()
}

/// Every line of `file` under `rules/source/<lang>/`, cached per
/// `(lang, file)` pair so a scope of 51 ids reads each source file once.
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

/// The verbatim body a `[start, end]` citation (1-indexed, inclusive, exactly
/// as shipped in `source.lines`) brackets, once the citation's own two
/// non-body lines are excluded and [`normalize_markdown`] is applied.
///
/// **What the citation excludes, and why.** Checked directly against three
/// cited ranges (`virtue.berserk` ArMDE:3500-3503, `virtue.commanding_aura`
/// ArMDE:3579-3596, `virtue.blood_of_the_nephilim` ArMDE:3504-3518):
/// `start` is always the `#### <Name>` heading line, and `start + 1` is
/// always the `*<Magnitude>, <Category>*<br>` descriptor line — neither
/// states a rule, so the body is `[start + 2, end]` inclusive. `end` is
/// consistently a **trailing blank line** separating the entry from the next
/// `####` heading (confirmed blank for both berserk's line 3503 and
/// blood_of_the_nephilim's line 3518), and an entry may additionally carry a
/// **leading** blank line right after the descriptor, before the first real
/// paragraph (blood_of_the_nephilim: line 3506 is blank, the body starts at
/// 3507; commanding_aura has no such leading blank — its body starts
/// immediately at 3581). Both blanks are handled the same way, without
/// special-casing either: the sliced lines are grouped into paragraphs on
/// blank-line boundaries and empty groups are discarded, which silently
/// absorbs a leading and/or a trailing blank line.
///
/// **Blockquote and heading handling (fix-round Group B, 2026-09-29)**: a
/// leading `>` blockquote marker (with or without a following space) is
/// stripped from each line before it is classified as blank or content — a
/// bare `>` line therefore acts as a paragraph separator, exactly like a
/// truly blank line, which is what lets a boxed callout
/// (`virtue.greater_benediction`'s "> #### Greater Benediction Examples" /
/// "> ##### Flight" / … insert) read as ordinary paragraphs rather than
/// literal markup. A line that, after that stripping, starts with one or
/// more `#` (a Markdown heading, at any level) becomes its own single-line
/// paragraph containing just the heading text — the `#` markers and
/// surrounding whitespace stripped, nothing else added (no colon, no
/// synthesized label). This is a markup-shape normalization, not a wording
/// change: nothing on a heading or blockquote line is reworded, only its
/// Markdown decoration is removed, the same category of transformation
/// [`normalize_markdown`] already applies to emphasis and links.
///
/// **Known limitation, not silently special-cased**: this function still
/// assumes one physical source line per paragraph (true of every entry read
/// so far), and it reads only its own single `[start, end]` citation, so it
/// cannot by itself reproduce a description that deliberately merges text
/// from a second, separately-cited passage (`virtue.the_gift`) —
/// [`COMPOSED_DESCRIPTIONS`] composes those by calling this function once per
/// range and concatenating the results.
fn bracketed_verbatim_body(lines: &[String], start: u32, end: u32) -> String {
    let body_start = start as usize + 2; // 1-indexed first body line
    let body_end = end as usize; // 1-indexed last body line, inclusive
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

    fn flush(current: &mut Vec<&str>, paragraphs: &mut Vec<String>) {
        if !current.is_empty() {
            paragraphs.push(current.join(" "));
            current.clear();
        }
    }

    let mut paragraphs: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for raw_line in slice {
        let trimmed = raw_line.trim();
        let content = trimmed.strip_prefix('>').map(str::trim).unwrap_or(trimmed);
        if content.is_empty() {
            flush(&mut current, &mut paragraphs);
            continue;
        }
        let heading_text = content.trim_start_matches('#');
        if heading_text.len() != content.len() {
            // A `#`-prefixed Markdown heading: its own paragraph, markup
            // stripped, no label synthesized.
            flush(&mut current, &mut paragraphs);
            paragraphs.push(heading_text.trim().to_string());
            continue;
        }
        current.push(content);
    }
    flush(&mut current, &mut paragraphs);

    normalize_markdown(&paragraphs.join("\n\n"))
}

/// X2a (2026-09-29), D46: `virtue.the_gift`'s shipped description legitimately
/// composes two passages — its own entry citation (ArMDE:3967-3970, the
/// standard header-skipped citation) plus the "### The Gift" prose-section's
/// penalties clause. That clause has no two-line entry header of its own
/// (`#### Name` / `*Magnitude, Category*<br>`) to skip, so its range is
/// encoded as `(2868, 2870)`: line 2868 is the "### The Gift"/"### Die Gabe"
/// section heading and 2869 is its blank line, so [`bracketed_verbatim_body`]'s
/// existing `start + 2` skip lands exactly on the real paragraph at 2870
/// without needing a second extraction function. Each range's
/// [`bracketed_verbatim_body`] output is concatenated with `"\n\n"` — the same
/// separator a single citation's own multiple paragraphs already use — so a
/// composed entry reads as one seamless multi-paragraph description. Only
/// `virtue.the_gift` is on this table for now; X2b-h may add more.
const COMPOSED_DESCRIPTIONS: &[(&str, &[(u32, u32)])] =
    &[("virtue.the_gift", &[(3967, 3970), (2868, 2870)])];

/// The shared scope-tracking list the verbatim-fidelity guard
/// (`x2_shipped_descriptions_match_their_cited_passage_verbatim`) iterates.
/// X2b through X2h append their own ids here as each sub-slice's Phase 2 data
/// lands — this is the single accumulating record, not a per-slice copy.
/// X2a's 51 ids (`tmp/x2-worklist.md` § 1 rows 1-51, `tmp/x2a-verdicts.md`),
/// including `virtue.devil_child` (row 21, routed into X2a's own range by the
/// coordinator rather than a later slice).
const X2_VERBATIM_SCOPE: &[&str] = &[
    "virtue.academic_concentration_subject",
    "virtue.affinity_ability",
    "virtue.affinity_art",
    "virtue.almogavar",
    "virtue.amorphous_major",
    "virtue.amorphous_minor",
    "virtue.apprentice",
    "virtue.arcane_lore",
    "virtue.aristotelian_training",
    "virtue.bee_king",
    "virtue.berserk",
    "virtue.blood_of_the_nephilim",
    "virtue.cathedral_school_master",
    "virtue.clan_ilfetu",
    "virtue.commanding_aura",
    "virtue.common_sense",
    "virtue.covenfolk",
    "virtue.craftsman",
    "virtue.demonic_blood",
    "virtue.demonic_might",
    "virtue.devil_child",
    "virtue.diedne_magic",
    "virtue.doctor_in_faculty",
    "virtue.domestic_animal",
    "virtue.elemental_magic",
    "virtue.emir",
    "virtue.enduring_constitution",
    "virtue.factor",
    "virtue.faerie_blood",
    "virtue.faerie_magic",
    "virtue.faerie_raised_magic",
    "virtue.fast_caster",
    "virtue.feather_messenger",
    "virtue.finding_hidden_loot",
    "virtue.flawless_magic",
    "virtue.gender_shift",
    "virtue.gentle_gift",
    "virtue.gentleman",
    "virtue.ghostly_warder",
    "virtue.the_gift",
    "virtue.gorgiastic",
    "virtue.gossip",
    "virtue.greater_benediction",
    "virtue.greater_immunity",
    "virtue.greater_power",
    "virtue.greater_purifying_touch",
    "virtue.guardian_angel",
    "virtue.guest_of_house_criamon",
    "virtue.guild_apprentice",
    "virtue.harnessed_magic",
    "virtue.heartbeast",
    // X2b (`tmp/x2-worklist.md` § 1 rows 52-102, `tmp/x2b-verdicts.md`), 47 ids
    // with a description obligation — hermetic_magus (no desc), journeyman,
    // laborer and latent_magic_ability (stay narrative, no rule stated) are
    // deliberately not here, matching X2a's craftsman/domestic_animal/factor/
    // gentleman precedent.
    "virtue.homing_instinct",
    "virtue.imbued_with_the_spirit_of_form",
    "virtue.immune_to_disease",
    "virtue.immunity_to_cold",
    "virtue.indescribable_face",
    "virtue.infernal_heirloom",
    "virtue.inoffensive_to_beings",
    "virtue.inspirational",
    "virtue.intuition",
    "virtue.inventive_genius",
    "virtue.jurist",
    "virtue.just_an_instant",
    "virtue.kassalan_exorcism",
    "virtue.keen_vision",
    "virtue.keen_sense_of_smell",
    "virtue.knight",
    "virtue.knows_people",
    "virtue.land_regio_network",
    "virtue.landed_noble",
    "virtue.learn_ability_from_mistakes",
    "virtue.leather_ripper",
    "virtue.leper_magus",
    "virtue.lesser_benediction",
    "virtue.lesser_immunity",
    "virtue.lesser_power",
    "virtue.lesser_purifying_touch",
    "virtue.license_of_absence",
    "virtue.life_boost",
    "virtue.life_linked_spontaneous_magic",
    "virtue.linguist",
    "virtue.lone_redcap",
    "virtue.long_winded",
    "virtue.luck",
    "virtue.magian_lineage_major",
    "virtue.magian_lineage_minor",
    "virtue.magic_items",
    "virtue.magic_sensitivity",
    "virtue.magical_memory",
    "virtue.magical_blood",
    "virtue.magical_mount",
    "virtue.magical_warder",
    "virtue.magister_in_artibus",
    "virtue.magister_in_medicina",
    "virtue.major_magical_focus",
    "virtue.maker_of_textured_vessels",
    "virtue.maker_of_water_vessels",
    "virtue.male_guild_sponsor",
];

/// D5/D46: a shipped `description` is a rule's only carrier once the entry
/// leaves `narrative`/stays partly uncomputed, so it must **be** the cited
/// passage, not a paraphrase that happens to satisfy some other guard. Every
/// id in [`X2_VERBATIM_SCOPE`] that ships a `description` in a locale is
/// checked against [`bracketed_verbatim_body`] for that same locale's source
/// file, at the entry's own `source.lines`.
#[test]
fn x2_shipped_descriptions_match_their_cited_passage_verbatim() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut cache: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    let mut offenders = Vec::new();
    for id in X2_VERBATIM_SCOPE {
        let item = rs
            .items()
            .find(|i| i.id.as_str() == *id)
            .unwrap_or_else(|| panic!("{id} is not in the shipped catalogue"));
        let source = item
            .source
            .as_ref()
            .unwrap_or_else(|| panic!("{id} carries no source citation"));
        let start = source.lines.start;
        let end = source.lines.end;
        let composed_ranges = COMPOSED_DESCRIPTIONS
            .iter()
            .find(|(composed_id, _)| composed_id == id)
            .map(|(_, ranges)| *ranges);

        for (lang, loc, source_file) in [
            ("en", &loc_en, CORE_RULES_FILE_EN),
            ("de", &loc_de, CORE_RULES_FILE_DE),
        ] {
            let Some(shipped) = loc.description(&Id::new(*id)) else {
                continue;
            };
            let lines = cached_source_lines(&mut cache, lang, source_file);
            let expected = match composed_ranges {
                Some(ranges) => ranges
                    .iter()
                    .map(|(range_start, range_end)| {
                        bracketed_verbatim_body(lines, *range_start, *range_end)
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n"),
                None => bracketed_verbatim_body(lines, start, end),
            };
            if shipped != expected {
                offenders.push(format!(
                    "{lang}/{id} (lines {start}-{end}):\n    expected: {expected:?}\n    \
                     shipped:  {shipped:?}"
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "{} shipped description(s) in X2_VERBATIM_SCOPE do not match their cited passage \
         verbatim (only the en-dash-before-digit and Markdown emphasis/link normalizations are \
         allowed, docs/open-todos.md row 38):\n\n{}",
        offenders.len(),
        offenders.join("\n\n")
    );
}

/// ArMDE:3745 ("This is the same as the Knight Virtue") means Emir inherits
/// Knight's *computed* mechanics, not just its flavor text. Knight's own
/// passage (ArMDE:4195-4198) states equipment access, a male-only
/// restriction, a Wealthy/Poor null-interaction, and Landed Noble
/// compatibility, of which only the ability-authorization grant is currently
/// computed (`effects: [{ability_authorization, categories: [martial]}]`,
/// X1) — the rest is X2b's problem (Knight's own row is outside X2a's range),
/// not this test's.
///
/// Emir's `classification` does **not** mirror Knight's, and that is
/// deliberate rather than an oversight this test should paper over (fix-round
/// correction, 2026-09-29): D67 (`docs/vf-audit/decisions.md`) rules that
/// carrying a computed effect never forces `creation_effect` while the
/// entry's own cited passage still leaves something uncomputed, and "This is
/// the same as the Knight Virtue" pulls in Knight's *other*, still-uncomputed
/// clauses (equipment access, the male-only restriction, the Wealthy/Poor
/// null-interaction, Landed Noble compatibility) that Emir itself computes
/// none of. So Emir stays `uncomputed_rule` regardless of what Knight's own
/// classification happens to be today — a fixed target, not one tied
/// dynamically to Knight, which is itself `creation_effect` only because its
/// own uncomputed remainder is a separate, not-yet-landed X2b problem.
#[test]
fn emir_carries_the_same_computed_effects_as_knight() {
    let rs = load_ruleset();
    let knight = rs
        .items()
        .find(|i| i.id.as_str() == "virtue.knight")
        .expect("virtue.knight is in the shipped catalogue");
    let emir = rs
        .items()
        .find(|i| i.id.as_str() == "virtue.emir")
        .expect("virtue.emir is in the shipped catalogue");

    assert_eq!(
        emir.effects, knight.effects,
        "ArMDE:3745 \"This is the same as the Knight Virtue\": Emir must carry Knight's \
         computed effects ({:?}), not none",
        knight.effects
    );
    assert_eq!(
        emir.classification,
        Classification::UncomputedRule,
        "D46/D67: Emir carries a computed effect but its own passage (\"This is the same as \
         the Knight Virtue\") still leaves Knight's other clauses uncomputed, so it stays \
         uncomputed_rule regardless of Knight's own classification"
    );
}

// ---------------------------------------------------------------------------
// X2b (`tmp/x2-worklist.md` § 1 rows 52-102, ArMDE:4067-4442) — see
// `tmp/x2b-verdicts.md` for the full per-entry citation and rationale this file
// intentionally does not re-derive inline.
// ---------------------------------------------------------------------------

/// X2b: entries that compute nothing today, whose passage the mechanical-token
/// screen does not flag (so no pending list carries them), and that D8/D50 all
/// the same require to become `uncomputed_rule` with their rule written into
/// `description` in both locales. `(id, why)` — see `tmp/x2b-verdicts.md` for
/// the full reading.
const X2B_RECLASSIFY_WITH_DESCRIPTION: &[(&str, &str)] = &[
    (
        "virtue.immune_to_disease",
        "D8 capability: the character is marked as a demon's property, so the lesser demons \
         that cause most diseases refuse to harm him (ArMDE:4095-4098); no number, no roll, no \
         idiom the screen recognizes",
    ),
    (
        "virtue.immunity_to_cold",
        "D8 capability: \"Normal cold does not harm you\" (ArMDE:4099-4102); no number, no roll",
    ),
    (
        "virtue.infernal_heirloom",
        "D8 capability + a frequency figure: each heirloom \"may create an effect once per day \
         that is equivalent to a Hermetic spell of level 25\" (ArMDE:4127-4132); an unsigned \
         number is not a modifier the screen recognizes (states_a_mechanical_rule's own \
         documented exclusion)",
    ),
    (
        "virtue.lesser_immunity",
        "D8 capability + cross-reference: \"immune to some hazard which is either rare, or not \
         deadly, or both\" (ArMDE:4275-4278); Greater Immunity itself carries no effect to \
         inherit, unlike Emir/Knight",
    ),
    (
        "virtue.just_an_instant",
        "D50: \"does not need to make Awareness checks\" (ArMDE:4169-4172) exempts a roll \
         requirement a player must know applies; no signed number, no recognized idiom",
    ),
    (
        "virtue.knows_people",
        "D50: \"Once per story or session, a character with this Virtue may ask for a bait\" \
         (ArMDE:4199-4206) is a real, invokable ability with a real frequency limit; 0 effects",
    ),
    (
        "virtue.landed_noble",
        "D50: \"you may not impose the death penalty, nor may you mutilate criminals\" and, if \
         Poor, \"you must spend every season managing it, or it may collapse completely\" \
         (ArMDE:4219-4228) are real constraints a player/storyguide must apply; F-131",
    ),
    (
        "virtue.magical_memory",
        "D50: \"You need not keep laboratory texts... to get the benefit of a Lab Text\" \
         (ArMDE:4355-4358) is a real exemption from the normal Lab Text requirement; F-162",
    ),
    (
        "virtue.life_linked_spontaneous_magic",
        "D20: one of the fourteen \"number right\" surfaced-only entries — the per-five-points \
         Fatigue rule and the wound conversion (ArMDE:4299-4306) reach the player only as a bare \
         `special_casting_mod` label, not text; F-148",
    ),
];

#[test]
fn x2b_entries_reclassify_to_uncomputed_rule_with_a_description() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    let mut offenders = Vec::new();
    for (id, why) in X2B_RECLASSIFY_WITH_DESCRIPTION {
        let classification = classification_of(&rs, id);
        if classification != Classification::UncomputedRule {
            offenders.push(format!(
                "{id}: classified {classification:?}, expected UncomputedRule ({why})"
            ));
        }
        for (lang, loc) in [("en", &loc_en), ("de", &loc_de)] {
            if displayed_text(loc, id).is_none() {
                offenders.push(format!(
                    "{lang}/{id}: no displayed rules text at all ({why})"
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "X2b (tmp/x2b-verdicts.md): these entries must reclassify to `uncomputed_rule` and carry \
         a description in every locale — a Phase 2 data change, not yet landed:\n{}",
        offenders.join("\n")
    );
}

/// D20's two other "number missing" entries in this slice (alongside
/// virtue.commanding_aura, X2a): `docs/vf-audit/decisions.md` D20 names
/// `virtue.leper_magus` and `virtue.life_boost` among the five surfaced-only
/// entries whose number reaches the player nowhere. Both share the same
/// `special_casting_mod` effect kind (`life_boost`) with no numeric field, so
/// X2b owns only the reclassification and the text+numbers; the effect-kind
/// fix (if any) is X6's.
#[test]
fn leper_magus_reclassifies_and_states_its_wound_for_vis_figures() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.leper_magus"),
        Classification::UncomputedRule,
        "D20: ArMDE:4249-4252's wound-for-vis figures reach the player nowhere under a bare \
         `special_casting_mod` kind; virtue.leper_magus must reclassify to uncomputed_rule"
    );

    // ArMDE:4249-4252, one pawn figure per wound severity (Light through Deadly).
    // The book itself is inconsistent about digits vs. words: the English passage
    // spells Light/Medium/Heavy as words ("three pawns", "six pawns", "nine
    // pawns") but Incapacitating/Deadly as digits ("12 pawns", "15 pawns"); the
    // German translation spells all five as words. A verbatim description must
    // keep whichever form the book uses, so the check is per-language rather
    // than a single digit list.
    const FIGURES: &[(&str, &str)] = &[
        ("three", "drei"),
        ("six", "sechs"),
        ("nine", "neun"),
        ("12", "zwölf"),
        ("15", "fünfzehn"),
    ];

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, index) in [("en", &loc_en, 0usize), ("de", &loc_de, 1usize)] {
        let text = displayed_text(loc, "virtue.leper_magus").unwrap_or_default();
        for figure_pair in FIGURES {
            let figure = if index == 0 {
                figure_pair.0
            } else {
                figure_pair.1
            };
            if !text.contains(figure) {
                offenders.push(format!(
                    "{lang}/virtue.leper_magus: displayed text {text:?} is missing the {figure}-\
                     pawn wound figure"
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "X2b (tmp/x2b-verdicts.md), D20's missing-number finding for leper_magus:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn life_boost_reclassifies_and_states_its_fatigue_figures() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.life_boost"),
        Classification::UncomputedRule,
        "D20: ArMDE:4295-4298's +5-per-Fatigue-level bonus and its damage formula reach the \
         player nowhere under a bare `special_casting_mod` kind; virtue.life_boost must \
         reclassify to uncomputed_rule"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc) in [("en", &loc_en), ("de", &loc_de)] {
        let text = displayed_text(loc, "virtue.life_boost").unwrap_or_default();
        // ArMDE:4295-4298: "+5" per level, and the worked example's "15" damage
        // (three levels x 5).
        for figure in ["5", "15"] {
            if !text.contains(figure) {
                offenders.push(format!(
                    "{lang}/virtue.life_boost: displayed text {text:?} is missing the {figure} \
                     figure"
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "X2b (tmp/x2b-verdicts.md), D20's missing-number finding for life_boost:\n{}",
        offenders.join("\n")
    );
}

/// F-173's second half (`docs/vf-audit/corrections.md`): virtue.magister_in_medicina's
/// own summary states no mechanical token at all, so the coarse screen never
/// reaches this entry to record a pending row for it — ArMDE:4397-4398's
/// "available to female characters, although they must have graduated from
/// Salerno, male characters may have graduated from any of the medical
/// schools" reaches the player nowhere. `classification` stays `creation_effect`
/// (its Reputation + restricted-XP effects already duplicate Doctor in
/// (Faculty)'s exactly, resolving F-173's other half); only a targeted content
/// check catches this kind of gap.
#[test]
fn magister_in_medicina_states_its_salerno_restriction() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.magister_in_medicina"),
        Classification::CreationEffect,
        "F-173: the Reputation and restricted-XP grants are genuinely computed; only the \
         Salerno restriction is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    // A bare "Salerno" substring is already satisfied by the shipped summary's
    // list of medical schools ("Salerno, Cremona, Montpellier, or Bologna") —
    // that is not the restriction. The needles below are unique to the
    // restriction clause itself (ArMDE:4397's "although they must have
    // graduated from Salerno" / the DE mirror's "an der Schule von Salerno
    // graduiert").
    for (lang, loc, needle) in [
        ("en", &loc_en, "graduated from Salerno"),
        ("de", &loc_de, "an der Schule von Salerno graduiert"),
    ] {
        let text = displayed_text(loc, "virtue.magister_in_medicina").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.magister_in_medicina: displayed text {text:?} does not state the \
                 Salerno restriction (ArMDE:4397-4398)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2b (tmp/x2b-verdicts.md), F-173: virtue.magister_in_medicina's description must state \
         the female-Salerno/male-any-school restriction in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-153 (`docs/vf-audit/corrections.md`): virtue.long_winded's own summary
/// already states its "+3 on all your Fatigue rolls" bonus in both locales, so
/// the coarse mechanical-token screen cannot see the one clause it drops —
/// ArMDE:4329's "This bonus does not apply to casting spells" reaches the
/// player nowhere. `classification` stays `in_play_effect` (the bonus itself is
/// genuinely computed via `health_mod`/`fatigue_roll`); only a targeted content
/// check catches this kind of gap.
#[test]
fn long_winded_states_its_casting_exclusion() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.long_winded"),
        Classification::InPlayEffect,
        "F-153: the +3 Fatigue-roll bonus is genuinely computed; only the casting-spells \
         exclusion is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [("en", &loc_en, "casting"), ("de", &loc_de, "Zaubern")] {
        let text = displayed_text(loc, "virtue.long_winded").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.long_winded: displayed text {text:?} does not state the \
                 casting-spells exclusion (ArMDE:4329)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2b (tmp/x2b-verdicts.md), F-153: virtue.long_winded's description must state \"this \
         bonus does not apply to casting spells\" in both locales:\n{}",
        offenders.join("\n")
    );
}
