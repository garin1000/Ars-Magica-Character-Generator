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
    // "virtue.guild_apprentice" (F-100, ArMDE:4041-4044) is REMOVED here (X7a-refactor):
    // its one stated mechanical rule — the Poor/Wealthy later-life-XP-rate suppression —
    // is now computed, via `Effect::SuppressesLaterLifeXpRate`
    // (`types.rs`/`life_stage.rs::later_life_rate`), so D67's "any stated rule computed
    // nowhere" no longer applies and the entry reclassifies `uncomputed_rule` ->
    // `in_play_effect` (`rules/core/virtues_flaws.json`). See
    // `crates/arm-rules/RULES.md`'s D47/X7a section.
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
static EMPHASIS_BOLD_STAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*([^*\n]+)\*\*").unwrap());
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
/// `**text**`, `_text_`) and link syntax (`[text](url)`) collapse to their
/// inner text. Nothing else may differ — no synonym swap, no restructuring,
/// no added or removed clause.
///
/// **Bold before single-star, and this order matters.** [`EMPHASIS_STAR_RE`]
/// matches a single `*...*` span; run on `**X**` alone (no
/// [`EMPHASIS_BOLD_STAR_RE`] pass first) it cannot see the outer `**` pair as
/// a unit. It instead matches the *inner* single-star pair — the second `*`
/// of the opening `**` against the first `*` of the closing `**` — leaving
/// one bare `*` stranded on each edge: `**X**` alone becomes `*X*`, not `X`.
/// Two adjacent bold spans compound this into a visible defect rather than a
/// merely redundant one: `**X** OR **Y**` collapsed to `*X OR Y*`, because
/// the stray edge stars end up bracketing the whole run. Stripping `**...**`
/// first removes the double-star pairs as units, so the later single-star
/// pass has nothing left to misparse.
fn normalize_markdown(s: &str) -> String {
    let s = LINK_RE.replace_all(s, "$1");
    let s = EMPHASIS_BOLD_STAR_RE.replace_all(&s, "$1");
    let s = EMPHASIS_STAR_RE.replace_all(&s, "$1");
    let s = EMPHASIS_UNDERSCORE_RE.replace_all(&s, "$1");
    EN_DASH_BEFORE_DIGIT_RE.replace_all(&s, "-$1").into_owned()
}

/// Bug guard (fix-round, 2026-09-29): [`EMPHASIS_STAR_RE`] matches a single
/// `*...*` span, so on `**X**` it cannot see the outer pair as a unit — it
/// finds the *inner* single-star pair first (the second `*` of the opening
/// `**` paired with the first `*` of the closing `**`) and leaves one bare
/// `*` stranded on each edge. Two adjacent bold spans joined by "OR" compound
/// this: `**X** OR **Y**` collapses to `*X OR Y*` instead of `X OR Y`. This
/// is exactly the shape that reached `virtue.performance_magic` and
/// `virtue.personal_power` in both shipped locales.
#[test]
fn normalize_markdown_strips_adjacent_bold_spans() {
    assert_eq!(normalize_markdown("**X** OR **Y**"), "X OR Y");
    assert_eq!(normalize_markdown("**X**"), "X");
    assert_eq!(normalize_markdown("*X*"), "X");
    assert_eq!(
        normalize_markdown("This is **bold** inside a sentence."),
        "This is bold inside a sentence."
    );
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
///
/// X2d (2026-09-30), D70: `virtue.true_faith`'s shipped description composes
/// its own entry citation (ArMDE:5169-5172) with the whole "### True Faith"
/// prose section (ArMDE:17603-17617) that entry's own text points readers to
/// ("For more about True Faith, see page 419"). Unlike `virtue.the_gift`'s
/// second range, this one is not a single paragraph clipped to a section
/// heading — it is the entire section, so its range starts at the `###`
/// heading itself (17603) and runs to 17617, the section's own last content
/// line, one line before the trailing blank (17618) that separates it from
/// the next section, "### Relics" (17619). [`bracketed_verbatim_body`]'s
/// `start + 2` skip still lands correctly past the heading and its blank
/// line onto the first real paragraph (17605), and its existing
/// blank-line-flush loop already handles the section's several paragraphs
/// (including the bolded "TRUE FAITH MAGIC RESISTANCE" line, which
/// [`normalize_markdown`] strips to plain text like any other emphasis) —
/// no new extraction logic needed, only a wider range.
const COMPOSED_DESCRIPTIONS: &[(&str, &[(u32, u32)])] = &[
    ("virtue.the_gift", &[(3967, 3970), (2868, 2870)]),
    ("virtue.true_faith", &[(5169, 5172), (17603, 17617)]),
];

/// The shared scope-tracking list the verbatim-fidelity guard
/// (`x2_shipped_descriptions_match_their_cited_passage_verbatim`) iterates.
/// X2b through X2h append their own ids here as each sub-slice's Phase 2 data
/// lands — this is the single accumulating record, not a per-slice copy.
/// X2a's 51 ids (`tmp/x2-worklist.md` § 1 rows 1-51, `tmp/x2a-verdicts.md`),
/// including `virtue.devil_child` (row 21, routed into X2a's own range by the
/// coordinator rather than a later slice).
const X2_VERBATIM_SCOPE: &[&str] = &[
    // X7c (`tmp/x7c-verdicts.md` Phase 2): three entries whose second stated
    // clause reached the player nowhere (D46/D67) — reclassified to
    // `uncomputed_rule` with the full cited passage as `description`.
    "flaw.age_quickly",
    "flaw.baneful_circumstances",
    "flaw.creative_block",
    // X2e (`tmp/x2-worklist.md` rows 204-260, `tmp/x2e-verdicts.md`): six
    // already-settled entries (Bucket A) whose classification/effects are
    // unchanged this slice — added here only to gate their shipped
    // `description` against the verbatim-fidelity check. Two are expected to
    // fail today (a real Phase 2 finding, not a bug in this test):
    // `flaw.abandoned_apprentice`'s shipped English text flattens the
    // rulebook's four paragraphs (ArMDE:5643-5649) into one running
    // paragraph, and `flaw.a_deal_with_the_devil`'s shipped text normalizes
    // the rulebook's curly apostrophe in "Hell's" (ArMDE:5907) to a straight
    // one, which the verbatim check (only en-dash-before-digit and Markdown
    // normalization are allowed) does not permit.
    "flaw.a_deal_with_the_devil",
    "flaw.abandoned_apprentice",
    "flaw.blatant_gift",
    "flaw.church_upbringing",
    "flaw.cyclic_magic_negative",
    "flaw.deaf",
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
    // X2c (`tmp/x2-worklist.md` § 1 rows 103-153, `tmp/x2c-verdicts.md`), 45 ids
    // with a description obligation — merchant, merchant_adventurer, nuntius
    // and peasant (stay narrative, no rule stated) and master_of_kennels/
    // prestigious_student (fully computed, verified no residue) are
    // deliberately not here, matching X2a/X2b's own precedent.
    "virtue.mamluk",
    "virtue.marshal",
    "virtue.master_bard",
    "virtue.masterpiece",
    "virtue.mazdean_priest",
    "virtue.mendicant_friar",
    "virtue.mercurian_magic",
    "virtue.method_caster",
    "virtue.minor_enchantments",
    "virtue.minor_magical_focus",
    "virtue.muqta_muq_ta",
    "virtue.muse",
    "virtue.mystical_choreography",
    "virtue.mythic_blood",
    "virtue.natural_leader",
    "virtue.nephilim",
    "virtue.notary",
    "virtue.paid_rights",
    "virtue.partner",
    "virtue.perfect_balance",
    "virtue.perfect_eye_for_commodity",
    "virtue.perfectus",
    "virtue.performance_magic",
    "virtue.personal_power",
    "virtue.personal_vis_source",
    "virtue.piercing_gaze",
    "virtue.potent_magic_major",
    "virtue.potent_magic_minor",
    "virtue.powerful_relic",
    "virtue.priest",
    "virtue.protection",
    "virtue.puissant_ability",
    "virtue.puissant_art",
    "virtue.quiet_magic",
    "virtue.rabbi",
    "virtue.rat_up_a_drainpipe",
    "virtue.redcap",
    "virtue.relic",
    "virtue.reserves_of_strength",
    "virtue.ripper",
    "virtue.ritual_power",
    "virtue.rosh_beth_din",
    "virtue.schooled_in_crime",
    "virtue.secondary_insight",
    "virtue.see_in_darkness",
    // X2d (`tmp/x2-worklist.md` § 1 rows 154-203, `tmp/x2d-verdicts.md`), 44 ids
    // with a description obligation — shamash, sofer (owned by X5's prereq
    // work; no X2 description), templar_servant (stays narrative, no rule
    // stated), templar_prestige, templar_specialist (fully computed, no
    // residue) and voice_of_the_land (D8, but its summary already carries the
    // whole passage verbatim — no separate description, the flaw.missing_ear
    // precedent) are deliberately not here, matching earlier slices' own
    // precedent. true_faith (D70) IS here: its description composes its own
    // entry with the "### True Faith" section (ArMDE:17603-17617) via
    // COMPOSED_DESCRIPTIONS, resolving F-330's earlier deferral.
    "virtue.senior_bard",
    "virtue.senior_clergy",
    "virtue.sense_holiness_and_unholiness",
    "virtue.shadchan",
    "virtue.sharp_ears",
    "virtue.side_effect",
    "virtue.simple_student",
    "virtue.skilled_smuggler",
    "virtue.skinchanger",
    "virtue.skinchanger_dove",
    "virtue.social_contacts",
    "virtue.special_circumstances",
    "virtue.spell_improvisation",
    "virtue.spiritual_pact",
    "virtue.strong_angelic_heritage",
    "virtue.strong_faerie_blood",
    "virtue.strong_willed",
    "virtue.study_bonus",
    "virtue.subtle_magic",
    "virtue.sufi",
    "virtue.supernatural_beauty",
    "virtue.tainted_treasure",
    "virtue.templar_administrator",
    "virtue.templar_commander",
    "virtue.templar_confrere_or_consoeur",
    "virtue.templar_office_holder",
    "virtue.temporal_influence",
    "virtue.tethered_magic",
    "virtue.town_magistrate",
    "virtue.troupe_upbringing",
    "virtue.true_faith",
    "virtue.true_love_pc",
    "virtue.turb_trained",
    "virtue.unaffected_by_the_gift",
    "virtue.unaging",
    "virtue.unbound_tongue",
    "virtue.university_grammar_teacher",
    "virtue.variable_power",
    "virtue.venus_blessing",
    "virtue.verditius_magic",
    "virtue.wanderer",
    "virtue.wisdom_from_ignorance",
    "virtue.wise_one",
    "virtue.withstand_casting",
    // X2f (`tmp/x2-worklist.md` rows 261-317, `tmp/x2f-verdicts.md`): already
    // `uncomputed_rule` with a description carrying the full cited passage —
    // added here only to gate it against the verbatim-fidelity check.
    "flaw.lycanthrope",
    // X2f Phase 2: newly swept entries whose `description` is the full cited
    // passage, verbatim in both locales. `flaw.form_monstrosity` is
    // deliberately NOT here — its citation's range includes the "Monstrosity
    // Examples" Markdown table, which is not prose and is not reproduced in
    // the shipped description (`tmp/x2f-handover.md` § 2).
    "flaw.disorientating_magic",
    "flaw.enfeebled",
    "flaw.envied_beauty",
    "flaw.exciting_experimentation",
    "flaw.excommunicate",
    "flaw.exiled_atlantean",
    "flaw.false_power",
    "flaw.false_power_minor",
    "flaw.feral_scent",
    "flaw.fettered_magic",
    "flaw.fluctuating_fortune",
    "flaw.greater_malediction",
    "flaw.harmless_magic",
    "flaw.horrifying_appearance_snake_legs",
    "flaw.imagined_folk_tradition_vulnerability",
    "flaw.incompatible_arts",
    "flaw.judged_unfairly",
    "flaw.leprosy",
    "flaw.lesser_malediction",
    // D20 correction: reclassified to `uncomputed_rule`, but its shipped
    // description is the same severity-comparison text added this slice —
    // gated here for the same reason as the rest of this block.
    "flaw.environmental_magic_condition",
    // X2g (`tmp/x2-worklist.md` rows 318-374, `tmp/x2g-verdicts.md`): newly
    // swept entries whose `description` is the full cited passage, verbatim
    // in both locales. `flaw.prohibition` and `flaw.primogeniture_lineage`
    // are deliberately NOT here — neither ships a `description` at all
    // (classification-only reclassifications).
    "flaw.magical_being_companion",
    "flaw.magical_fascination",
    "flaw.master_of_none",
    "flaw.monastic_vows_hermetic",
    "flaw.monstrous_blood",
    "flaw.motion_sickness",
    "flaw.necessary_condition",
    "flaw.no_hands",
    "flaw.oath_of_fealty",
    "flaw.obese",
    "flaw.outlaw",
    "flaw.outlaw_leader",
    "flaw.painful_magic",
    "flaw.poor_eyesight",
    "flaw.regular",
    "flaw.restricted_power",
    "flaw.restriction",
    "flaw.savantism",
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

/// Bug guard (fix-round, 2026-09-29): a stray `*` in a shipped `description`
/// is always leftover Markdown emphasis/bold that [`normalize_markdown`]
/// failed to fully strip when the description was authored — never
/// intentional shipped text (`uebersetzungsregeln.md` never uses `*` as a
/// display glyph). Catalogue-wide, not scoped to [`X2_VERBATIM_SCOPE`], so it
/// also catches a future slice's data reintroducing the same bug shape.
#[test]
fn no_shipped_description_contains_a_literal_asterisk() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    let mut offenders = Vec::new();
    for item in rs.items() {
        for (lang, loc) in [("en", &loc_en), ("de", &loc_de)] {
            if let Some(desc) = loc.description(&item.id)
                && desc.contains('*')
            {
                offenders.push(format!("{lang}/{}: {desc:?}", item.id.as_str()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "{} shipped description(s) contain a literal '*' (unstripped Markdown emphasis):\n\n{}",
        offenders.len(),
        offenders.join("\n")
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

// ---------------------------------------------------------------------------
// X2c (`tmp/x2-worklist.md` § 1 rows 103-153, ArMDE:4443-4899) — see
// `tmp/x2c-verdicts.md` for the full per-entry citation and rationale this file
// intentionally does not re-derive inline.
// ---------------------------------------------------------------------------

/// X2c: entries that compute nothing today (or, for `virtue.nephilim`, compute
/// only a selection constraint D67 already counts as `creation_effect`), whose
/// passage no existing pending list carries, and that D8/D46/D50/OQ-6 all
/// require to become `uncomputed_rule` with their rule written into
/// `description` in both locales. `(id, why)` — see `tmp/x2c-verdicts.md` for
/// the full reading.
const X2C_RECLASSIFY_WITH_DESCRIPTION: &[(&str, &str)] = &[
    (
        "virtue.muqta_muq_ta",
        "F-202: \"All rules for the Landed Noble Virtue apply, except that most Muslims take \
         the Minor Status Virtue Emir, rather than Knight\" (ArMDE:4559-4562) is a \
         cross-reference instruction inheriting Landed Noble's own (uncomputed_rule, 0-effect) \
         rule; the entry itself computes nothing",
    ),
    (
        "virtue.mystical_choreography",
        "F-204: two numeric ceremony-time reductions, \"five minutes per magnitude\" and, with a \
         prepared space, \"one minute per magnitude\" (ArMDE:4567-4572); 0 effects today",
    ),
    (
        "virtue.personal_vis_source",
        "D50's own worked example: the hedged yield \"should be about one tenth as much as the \
         player covenant expects to gather per year\" (ArMDE:4728-4731) is a quantity the \
         troupe must set",
    ),
    (
        "virtue.rabbi",
        "F-240: a mandatory companion Virtue (\"must take the Educated (Hebrew) Virtue to \
         provide the required Academic Abilities\") and a male-only restriction \
         (ArMDE:4828-4833); 0 effects today",
    ),
    (
        "virtue.rat_up_a_drainpipe",
        "F-241: \"a substantial advantage in opposed Athletics rolls which represent being \
         chased\" (ArMDE:4838-4841) is a real, storyguide-adjudicated modifier; 0 effects today",
    ),
    (
        "virtue.see_in_darkness",
        "D8 capability: \"You can see in complete darkness\", bounded by two limiting clauses \
         on acuity and range (ArMDE:4896-4899) — explicitly named in this file's own S2_IDIOMS \
         comments as too bare for the general capability idiom",
    ),
    (
        "virtue.nephilim",
        "D46/OQ-3: the free Strong Angelic Heritage grant (\"You receive the Strong Angelic \
         Heritage Virtue free\", ArMDE:4594-4597) is computed by no effect at all, though its \
         incompatible_with is computed — D67 allows uncomputed_rule regardless of what else is \
         computed, the same shape as X2a's virtue.devil_child",
    ),
];

#[test]
fn x2c_entries_reclassify_to_uncomputed_rule_with_a_description() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    let mut offenders = Vec::new();
    for (id, why) in X2C_RECLASSIFY_WITH_DESCRIPTION {
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
        "X2c (tmp/x2c-verdicts.md): these entries must reclassify to `uncomputed_rule` and carry \
         a description in every locale — a Phase 2 data change, not yet landed:\n{}",
        offenders.join("\n")
    );
}

/// F-181 (`docs/vf-audit/corrections.md`): virtue.marshal's own summary already
/// states its "50 extra experience points" grant, so the coarse mechanical-token
/// screen never flagged it (its `PENDING_DROPPED_CLAUSE` row instead named a
/// different, since-resolved gap — X1's Martial Abilities authorization).
/// ArMDE:4453's "It functions as the Ability Medicine for the purpose of
/// treating veterinary diseases, and for surgery involving these animals"
/// reaches the player nowhere. `classification` stays `creation_effect`.
#[test]
fn marshal_states_its_medicine_substitution() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.marshal"),
        Classification::CreationEffect,
        "F-181: the 50 XP grant and the Martial Abilities authorization are genuinely computed; \
         only the Medicine-substitution clause is missing, which does not change the \
         classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "functions as the Ability Medicine"),
        ("de", &loc_de, "funktioniert wie die Fertigkeit Medizin"),
    ] {
        let text = displayed_text(loc, "virtue.marshal").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.marshal: displayed text {text:?} does not state the \
                 Medicine-substitution rule (ArMDE:4453)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-181: virtue.marshal's description must state the \
         functions-as-Medicine rule in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-184: virtue.master_bard's own summary already states "at least nine years
/// of study" (a signed-phrase false positive for the coarse screen), so it never
/// flagged the two genuinely dropped clauses: ArMDE:4459's "he has an
/// obligation to teach for at least two seasons a year" and ArMDE:4461's "This
/// Social Status is only available in Ireland." `classification` stays
/// `creation_effect`.
#[test]
fn master_bard_states_its_teaching_obligation_and_ireland_restriction() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.master_bard"),
        Classification::CreationEffect,
        "F-184: the Reputation and XP grants are genuinely computed; only the teaching \
         obligation and the Ireland restriction are missing, which does not change the \
         classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "two seasons a year"),
        ("en", &loc_en, "only available in Ireland"),
        ("de", &loc_de, "zwei Quartale im Jahr"),
        ("de", &loc_de, "nur in Irland verfügbar"),
    ] {
        let text = displayed_text(loc, "virtue.master_bard").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.master_bard: displayed text {text:?} does not state {needle:?} \
                 (ArMDE:4457-4462)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-184: virtue.master_bard's description must state the \
         two-season teaching obligation and the Ireland restriction in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-201: virtue.minor_magical_focus's own summary already states its doubling
/// effect ("add" in English, "doppelt" in German — both mechanical-phrase false
/// positives for the coarse screen), so it never flagged ArMDE:4538's "You
/// cannot be focused on a laboratory activity, such as creating charged items,
/// although a focus does apply to laboratory activities." `classification`
/// stays `in_play_effect`.
#[test]
fn minor_magical_focus_states_its_laboratory_activity_restriction() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.minor_magical_focus"),
        Classification::InPlayEffect,
        "F-201: the doubling itself is genuinely computed; only the lab-activity restriction is \
         missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "cannot be focused on a laboratory activity"),
        ("de", &loc_de, "keinen Fokus auf eine Laboraktivität haben"),
    ] {
        let text = displayed_text(loc, "virtue.minor_magical_focus").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.minor_magical_focus: displayed text {text:?} does not state the \
                 lab-activity restriction (ArMDE:4538)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-201: virtue.minor_magical_focus's description must state \
         the lab-activity restriction in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-236: virtue.protection's own summary already states its Reputation grant
/// (a signed number, "3"), so the coarse screen never flagged the two dropped
/// clauses: ArMDE:4812's "could be higher if your protector is particularly
/// great or well-known" and the good/bad choice ("a Reputation (good or bad,
/// your choice)"). `classification` stays `creation_effect`.
#[test]
fn protection_states_its_variable_reputation_and_choice() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.protection"),
        Classification::CreationEffect,
        "F-236: the level-3 Reputation grant is genuinely computed; only the \"could be higher\" \
         clause and the good/bad choice are missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "could be higher"),
        ("de", &loc_de, "höher sein kann"),
    ] {
        let text = displayed_text(loc, "virtue.protection").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.protection: displayed text {text:?} does not state the \
                 variable-Reputation clause (ArMDE:4810-4813)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-236: virtue.protection's description must state the \
         \"could be higher\" clause in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-237: virtue.puissant_ability's own summary already states its "+2" bonus
/// (a signed number), so the coarse screen never flagged ArMDE:4816's "Note
/// that you do not, in general, use an Ability when learning it, writing about
/// it, or helping someone else to improve it." `classification` stays
/// `creation_effect`.
#[test]
fn puissant_ability_states_its_learning_exclusion() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.puissant_ability"),
        Classification::CreationEffect,
        "F-237: the +2 Ability bonus is genuinely computed; only the learning/writing/helping \
         exclusion is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "helping someone else to improve it"),
        ("de", &loc_de, "jemandem hilfst, sie zu verbessern"),
    ] {
        let text = displayed_text(loc, "virtue.puissant_ability").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.puissant_ability: displayed text {text:?} does not state the \
                 learning/writing/helping exclusion (ArMDE:4814-4816)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-237: virtue.puissant_ability's description must state the \
         learning/writing/helping exclusion in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-238: virtue.puissant_art's own summary already states its "+3" bonus (a
/// signed number), so the coarse screen never flagged ArMDE:4820's "It does not
/// apply when learning, teaching, or writing about the Art." `classification`
/// stays `creation_effect`.
#[test]
fn puissant_art_states_its_learning_exclusion() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.puissant_art"),
        Classification::CreationEffect,
        "F-238: the +3 Art bonus is genuinely computed; only the learning/teaching/writing \
         exclusion is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "does not apply when learning, teaching, or writing",
        ),
        (
            "de",
            &loc_de,
            "gilt nicht beim Lernen, Unterrichten oder Schreiben",
        ),
    ] {
        let text = displayed_text(loc, "virtue.puissant_art").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.puissant_art: displayed text {text:?} does not state the \
                 learning/teaching/writing exclusion (ArMDE:4818-4820)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-238: virtue.puissant_art's description must state the \
         learning/teaching/writing exclusion in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-239: virtue.quiet_magic's own summary already states its "-5" penalty (a
/// signed number), so the coarse screen never flagged ArMDE:4826's "You may
/// take this Virtue twice, and eliminate the penalty altogether."
/// `classification` stays `in_play_effect`.
#[test]
fn quiet_magic_states_its_twice_taken_elimination() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.quiet_magic"),
        Classification::InPlayEffect,
        "F-239: the -5 no-speech penalty is genuinely computed; only the twice-taken elimination \
         is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "eliminate the penalty altogether"),
        ("de", &loc_de, "den Abzug vollständig eliminieren"),
    ] {
        let text = displayed_text(loc, "virtue.quiet_magic").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.quiet_magic: displayed text {text:?} does not state the \
                 twice-taken elimination (ArMDE:4826)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-239: virtue.quiet_magic's description must state the \
         twice-taken elimination in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-257: virtue.relic's own summary already states its True Faith score of
/// one (a signed-looking figure), so the coarse screen never flagged
/// ArMDE:4854's "The relic does not possess any additional powers."
/// `classification` stays `creation_effect`.
#[test]
fn relic_states_it_has_no_additional_powers() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.relic"),
        Classification::CreationEffect,
        "F-257: the True Faith grant is genuinely computed; only the no-additional-powers \
         clause is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "does not possess any additional powers"),
        ("de", &loc_de, "besitzt keine zusätzlichen Kräfte"),
    ] {
        let text = displayed_text(loc, "virtue.relic").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.relic: displayed text {text:?} does not state the \
                 no-additional-powers clause (ArMDE:4852-4855)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-257: virtue.relic's description must state the \
         no-additional-powers clause in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-229: virtue.powerful_relic's own summary already states its True Faith
/// score of 3 (a signed-looking figure), so the coarse screen never flagged
/// ArMDE:4786's "If you ever behave impiously (as judged by the storyguide)
/// your relic will cease to function until suitable penance is made."
/// `classification` stays `creation_effect`.
#[test]
fn powerful_relic_states_its_impiety_consequence() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.powerful_relic"),
        Classification::CreationEffect,
        "F-229: the True Faith grant is genuinely computed; only the impiety/cessation rule is \
         missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "your relic will cease to function until suitable penance",
        ),
        (
            "de",
            &loc_de,
            "hört das Relikt auf zu funktionieren, bis angemessene Buße",
        ),
    ] {
        let text = displayed_text(loc, "virtue.powerful_relic").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.powerful_relic: displayed text {text:?} does not state the \
                 impiety/cessation rule (ArMDE:4782-4787)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-229: virtue.powerful_relic's description must state the \
         impiety/cessation rule in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-258: virtue.schooled_in_crime's own summary already states its 50-XP
/// grant (a bare number, no sign, so this one is genuinely invisible to the
/// coarse screen for a different reason — the passage itself trips on no
/// token at all). ArMDE:4886's "other Abilities may be added to this list with
/// troupe approval" reaches the player nowhere. `classification` stays
/// `creation_effect`.
#[test]
fn schooled_in_crime_states_its_troupe_extension() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.schooled_in_crime"),
        Classification::CreationEffect,
        "F-258: the 50 XP grant on the named Abilities is genuinely computed; only the \
         troupe-approval extension is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "with troupe approval"),
        ("de", &loc_de, "Zustimmung der Spieltruppe"),
    ] {
        let text = displayed_text(loc, "virtue.schooled_in_crime").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.schooled_in_crime: displayed text {text:?} does not state the \
                 troupe-approval extension (ArMDE:4884-4887)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-258: virtue.schooled_in_crime's description must state the \
         troupe-approval extension in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-259: virtue.secondary_insight's own summary already states its
/// Technique-to-Forms grant, so the coarse screen never flagged three of its
/// four stated clauses; ArMDE:4894's "not increased by Affinities or any other
/// factors" is the one this test pins down. `classification` stays
/// `in_play_effect`.
#[test]
fn secondary_insight_states_its_affinity_exclusion() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.secondary_insight"),
        Classification::InPlayEffect,
        "F-259: the Technique-to-Forms bonus XP is genuinely computed; only the residue below is \
         missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "not increased by Affinities"),
        ("de", &loc_de, "durch Affinitäten"),
    ] {
        let text = displayed_text(loc, "virtue.secondary_insight").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.secondary_insight: displayed text {text:?} does not state the \
                 Affinity exclusion (ArMDE:4892-4895)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2c (tmp/x2c-verdicts.md), F-259: virtue.secondary_insight's description must state the \
         Affinity exclusion in both locales:\n{}",
        offenders.join("\n")
    );
}

// ---------------------------------------------------------------------------
// X2d (`tmp/x2-worklist.md` § 1 rows 154-203, ArMDE:4904-5282) — see
// `tmp/x2d-verdicts.md` for the full per-entry citation and rationale this file
// intentionally does not re-derive inline.
// ---------------------------------------------------------------------------

/// X2d: entries that compute nothing today, whose passage no existing pending
/// list carries, and that D8/D50/D60 require to become `uncomputed_rule` with
/// their rule written into `description` in both locales. `(id, why)` — see
/// `tmp/x2d-verdicts.md` for the full reading.
const X2D_RECLASSIFY_WITH_DESCRIPTION: &[(&str, &str)] = &[
    (
        "virtue.tethered_magic",
        "a whole Hermetic spell mechanic (tether control to another caster or an object; the \
         Arcane-Connection side effect) under `narrative`, 0 effects, ArMDE:5141-5144",
    ),
    (
        "virtue.unaffected_by_the_gift",
        "D8 capability: immunity to the negative social effects of others' Gift/Magical Air, \
         \"even a Blatant Gift\", ArMDE:5183-5186",
    ),
    (
        "virtue.unbound_tongue",
        "D8 capability, and the summary drops half of it: ArMDE:5193's second sentence (\"If he \
         is a magus, he may use his voice as normal to cast spells\") while transformed is not in \
         the summary at all",
    ),
    (
        "virtue.voice_of_the_land",
        "D8 capability: speaks with creatures of an associated environment and is not perceived \
         as threat or prey, ArMDE:5219-5222 — the summary already states the whole passage \
         verbatim, so no new description text is owed (flaw.missing_ear precedent)",
    ),
    (
        "virtue.wanderer",
        "D50: \"The Wealthy Major Virtue and Poor Major Flaw affect you normally\" (ArMDE:5225) \
         is a real clarification a player would get wrong by assuming this Social Status \
         excludes them, the same shape as virtue.templar_confrere_or_consoeur",
    ),
];

#[test]
fn x2d_entries_reclassify_to_uncomputed_rule_with_a_description() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    let mut offenders = Vec::new();
    for (id, why) in X2D_RECLASSIFY_WITH_DESCRIPTION {
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
        "X2d (tmp/x2d-verdicts.md): these entries must reclassify to `uncomputed_rule` and carry \
         a description in every locale — a Phase 2 data change, not yet landed:\n{}",
        offenders.join("\n")
    );
}

/// D50/Q-66 (`docs/vf-audit/decisions.md` D50's own worked example): ArMDE:4928
/// "your sensitivity may overwhelm you" in a strong divine/infernal aura is
/// computed nowhere. Overturns the entry's own `COMPUTED_ENTRY_COVERS_WHOLE_
/// PASSAGE` certification (`uncomputed_clauses.rs`) — D67 permits
/// `uncomputed_rule` despite the `ability_score_grant` effect staying.
#[test]
fn sense_holiness_and_unholiness_reclassifies_and_states_its_overwhelm_clause() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.sense_holiness_and_unholiness"),
        Classification::UncomputedRule,
        "D50/Q-66: ArMDE:4928's overwhelm clause reaches the player nowhere; \
         virtue.sense_holiness_and_unholiness must reclassify to uncomputed_rule"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "may overwhelm you"),
        ("de", &loc_de, "überwältigen"),
    ] {
        let text = displayed_text(loc, "virtue.sense_holiness_and_unholiness").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.sense_holiness_and_unholiness: displayed text {text:?} does not \
                 state the overwhelm clause (ArMDE:4926-4929)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), D50/Q-66: virtue.sense_holiness_and_unholiness's description \
         must state the overwhelm clause in both locales:\n{}",
        offenders.join("\n")
    );
}

/// D20 (`docs/vf-audit/decisions.md`): one of the five "number missing"
/// surfaced-only entries, mitigated because its `description` already states
/// the +3 in both locales — but D20 still moves all 19 to `uncomputed_rule`
/// unconditionally. The effect kind/rank fix (if any) is a different slice's
/// concern; this test only pins the classification and that the existing
/// description survives.
#[test]
fn special_circumstances_reclassifies_to_uncomputed_rule() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.special_circumstances"),
        Classification::UncomputedRule,
        "D20: virtue.special_circumstances is one of the 19 surfaced-only entries that must \
         reclassify to uncomputed_rule regardless of whether the number already reaches the \
         player"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    for (lang, loc) in [("en", &loc_en), ("de", &loc_de)] {
        let text = displayed_text(loc, "virtue.special_circumstances").unwrap_or_default();
        assert!(
            text.contains("+3"),
            "{lang}/virtue.special_circumstances: the existing description ({text:?}) must \
             survive the reclassification"
        );
    }
}

/// D20: `virtue.spell_improvisation`'s Casting-Total bonus reaches the player
/// only as a bare `special_casting_mod` label; the non-stacking rule
/// (ArMDE:5004-5005) reaches nobody at all.
#[test]
fn spell_improvisation_reclassifies_and_states_its_bonus_rules() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.spell_improvisation"),
        Classification::UncomputedRule,
        "D20: virtue.spell_improvisation's Casting Total bonus and its non-stacking rule reach \
         the player nowhere under a bare special_casting_mod kind"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "does not stack with other bonuses to his Casting Total",
        ),
        (
            "de",
            &loc_de,
            "lässt sich nicht mit anderen Boni auf die Zaubersumme stapeln",
        ),
    ] {
        let text = displayed_text(loc, "virtue.spell_improvisation").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.spell_improvisation: displayed text {text:?} does not state the \
                 non-stacking rule (ArMDE:5002-5005)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), D20: virtue.spell_improvisation's description must state the \
         non-stacking rule in both locales:\n{}",
        offenders.join("\n")
    );
}

/// D50 re-test (`tmp/x2-worklist.md` row 177) OVERTURNS the entry's own
/// `NO_RULE_DESPITE_TOKEN` certification (`uncomputed_clauses.rs`): the
/// curse's specific consequences are exactly D50's "guaranteed storyguide
/// intervention" shape (its own common_sense worked example), not "pure
/// story consequence" as the pre-D50 reading held.
#[test]
fn tainted_treasure_reclassifies_and_states_its_curse() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.tainted_treasure"),
        Classification::UncomputedRule,
        "D50: ArMDE:5097-5108's curse consequences are a real, storyguide-enforced rule; \
         virtue.tainted_treasure must reclassify to uncomputed_rule"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "the curse moves to these items"),
        ("de", &loc_de, "geht der Fluch auf diese über"),
    ] {
        let text = displayed_text(loc, "virtue.tainted_treasure").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.tainted_treasure: displayed text {text:?} does not state the \
                 curse-transfer clause (ArMDE:5097-5108)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), D50: virtue.tainted_treasure's description must state the \
         curse-transfer clause in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-267/F-268: virtue.shadchan's own summary already states its role and its
/// 50-XP grant, so the coarse screen never flagged the Social-Status
/// compatibility override ("this Virtue is compatible with any other Minor or
/// Free Social Status Virtue", ArMDE:4936) or the community-scoping of the
/// Area Lore half of the XP pool. `classification` stays `creation_effect`.
#[test]
fn shadchan_states_its_compatibility_and_community_scope() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.shadchan"),
        Classification::CreationEffect,
        "F-267/F-268: the 50 XP grant is genuinely computed; only the compatibility override is \
         missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "compatible with any other Minor or Free Social Status Virtue",
        ),
        (
            "de",
            &loc_de,
            "mit jeder anderen Kleinen oder Freien Sozialen Status-Tugend vereinbar",
        ),
    ] {
        let text = displayed_text(loc, "virtue.shadchan").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.shadchan: displayed text {text:?} does not state the \
                 compatibility override (ArMDE:4934-4939)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-267/F-268: virtue.shadchan's description must state the \
         compatibility override in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-300: virtue.study_bonus's own summary already states its two +2 bonuses
/// (signed numbers), so the coarse screen never flagged the eight-row
/// Art-Score-to-environment table (ArMDE:5058-5071) that determines which
/// environments qualify. `classification` stays `in_play_effect`.
#[test]
fn study_bonus_states_its_environment_table() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.study_bonus"),
        Classification::InPlayEffect,
        "F-300: the two +2 bonuses are genuinely computed; only the environment table is \
         missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "Your current Art score determines the magnitude of the surroundings",
        ),
        (
            "de",
            &loc_de,
            "Dein aktueller Kunstwert bestimmt die Magnitude der Umgebung",
        ),
    ] {
        let text = displayed_text(loc, "virtue.study_bonus").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.study_bonus: displayed text {text:?} does not state the \
                 environment-table rule (ArMDE:5056-5072)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-300: virtue.study_bonus's description must state the \
         environment-table rule in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-311: virtue.templar_administrator's own summary already states its role,
/// so the coarse screen never flagged the Status-substitution rule, the
/// male-only restriction (the same D5/F-123 reading that overturned
/// virtue.jurist/virtue.knight, X2b), or "no additional time".
/// `classification` stays `creation_effect`.
#[test]
fn templar_administrator_states_its_status_substitution_and_restrictions() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.templar_administrator"),
        Classification::CreationEffect,
        "F-311: the Academic authorization is genuinely computed; the Status-substitution rule \
         and the male-only restriction are missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "can replace the Brother-Knight, Brother-Sergeant, and Brother-Priest Status Virtues",
        ),
        ("en", &loc_en, "only available to male characters"),
        (
            "de",
            &loc_de,
            "kann die Sozialer-Status-Tugenden Bruder-Ritter, Bruder-Sergeant und \
             Bruder-Priester ersetzen",
        ),
        (
            "de",
            &loc_de,
            "steht nur männlichen Charakteren zur Verfügung",
        ),
    ] {
        let text = displayed_text(loc, "virtue.templar_administrator").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.templar_administrator: displayed text {text:?} does not state \
                 {needle:?} (ArMDE:5109-5112)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-311: virtue.templar_administrator's description must state \
         the Status-substitution rule and the male-only restriction in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-313: virtue.templar_commander's own summary already states its
/// Reputation grant, so the coarse screen never flagged the
/// taxation/tithe/service-fee/judicial powers or the crusading obligation.
/// `classification` stays `creation_effect`.
#[test]
fn templar_commander_states_its_taxation_and_judicial_powers() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.templar_commander"),
        Classification::CreationEffect,
        "F-313: the Reputation and Temporal Influence/Brother-Knight grants are genuinely \
         computed; the taxation/judicial powers are missing, which does not change the \
         classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "levy taxes and tithes"),
        ("de", &loc_de, "Steuern und Zehnten"),
    ] {
        let text = displayed_text(loc, "virtue.templar_commander").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.templar_commander: displayed text {text:?} does not state the \
                 taxation/judicial powers (ArMDE:5113-5116)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-313: virtue.templar_commander's description must state the \
         taxation/judicial powers in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-315: virtue.templar_office_holder's own summary already states its
/// Reputation grant, so the coarse screen never flagged the Templar-Status
/// compatibility override, stated twice (any Templar Status Virtue; the
/// Temporal Influence Minor Virtue). `classification` stays `creation_effect`.
#[test]
fn templar_office_holder_states_its_compatibility_overrides() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.templar_office_holder"),
        Classification::CreationEffect,
        "F-315: the Reputation grant is genuinely computed; the compatibility overrides are \
         missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "compatible with the Temporal Influence Minor Virtue",
        ),
        (
            "de",
            &loc_de,
            "mit der Kleinen Tugend Zeitlicher Einfluss vereinbar",
        ),
    ] {
        let text = displayed_text(loc, "virtue.templar_office_holder").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.templar_office_holder: displayed text {text:?} does not state the \
                 compatibility override (ArMDE:5121-5124)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-315: virtue.templar_office_holder's description must state \
         the compatibility override in both locales:\n{}",
        offenders.join("\n")
    );
}

/// D62 (`docs/vf-audit/decisions.md`: seasons are not modelled, so a
/// seasonal-commitment clause becomes text): virtue.town_magistrate's own
/// summary already trips the coarse mechanical-token screen for an unrelated
/// reason (verified: adding this entry to `PENDING_DROPPED_CLAUSE` produces a
/// false "gap already closed" failure), so this dedicated test is the only
/// carrier for ArMDE:5151's "occupies the character for two seasons each
/// year, but he is free for the remaining two seasons". `classification`
/// stays `creation_effect`.
#[test]
fn town_magistrate_states_its_seasonal_commitment() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.town_magistrate"),
        Classification::CreationEffect,
        "the Academic authorization is genuinely computed; the seasonal-commitment clause is \
         missing, which does not change the classification (D62: seasons stay text)"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "free for the remaining two seasons"),
        (
            "de",
            &loc_de,
            "die restlichen zwei Quartale steht er jedoch frei",
        ),
    ] {
        let text = displayed_text(loc, "virtue.town_magistrate").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.town_magistrate: displayed text {text:?} does not state the \
                 seasonal-commitment clause (ArMDE:5149-5152)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), D62: virtue.town_magistrate's description must state the \
         seasonal-commitment clause in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-346's residual half: virtue.university_grammar_teacher's classification
/// was already fixed to `creation_effect` before this slice, but the
/// certification's own "should have a score in Teaching is soft" reading
/// missed ArMDE:5197's separate, HARD obligation: "They must teach two
/// seasons out of the year." `classification` stays `creation_effect`.
#[test]
fn university_grammar_teacher_states_its_teaching_obligation() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.university_grammar_teacher"),
        Classification::CreationEffect,
        "the Latin/Artes Liberales authorization is genuinely computed; the two-season teaching \
         obligation is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "must teach two seasons out of the year"),
        ("de", &loc_de, "muss zwei Quartale im Jahr unterrichten"),
    ] {
        let text = displayed_text(loc, "virtue.university_grammar_teacher").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.university_grammar_teacher: displayed text {text:?} does not \
                 state the teaching obligation (ArMDE:5195-5198)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md): virtue.university_grammar_teacher's description must state \
         the two-season teaching obligation in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-303: virtue.sufi's own summary already states its role, so the coarse
/// screen never flagged ArMDE:5079's "you should choose an appropriate Minor
/// Story Flaw, such as Mentor, which does not yield any points for buying
/// Virtues" — an exception to the normal Flaw-for-points exchange.
/// `classification` stays `creation_effect`.
#[test]
fn sufi_states_its_no_points_story_flaw_rule() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.sufi"),
        Classification::CreationEffect,
        "F-303: the Academic/Arcane authorization is genuinely computed; the no-points Story \
         Flaw rule is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "does not yield any points for buying Virtues",
        ),
        (
            "de",
            &loc_de,
            "keine Punkte für den Erwerb von Tugenden einbringt",
        ),
    ] {
        let text = displayed_text(loc, "virtue.sufi").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.sufi: displayed text {text:?} does not state the no-points Story \
                 Flaw rule (ArMDE:5077-5084)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-303: virtue.sufi's description must state the no-points \
         Story Flaw rule in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-332: virtue.unaging's own summary ("do not suffer the effects of age" /
/// "leidest nicht ... des Alterns") already trips the coarse mechanical-token
/// screen by accident (the S2 "do not suffer"/"leidest nicht" idiom, added for
/// virtue.gentle_gift — verified: adding this entry to
/// `PENDING_DROPPED_CLAUSE` produces a false "gap already closed" failure), so
/// this dedicated test is the only carrier for the crisis clause and the
/// Decrepitude-4/5 waiver. `classification` stays `in_play_effect`.
#[test]
fn unaging_states_its_crisis_and_decrepitude_exceptions() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.unaging"),
        Classification::InPlayEffect,
        "the two aging_mod exemptions are genuinely computed; the crisis clause and the \
         Decrepitude-4/5 waiver are missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "you die as normal when you reach five"),
        (
            "de",
            &loc_de,
            "stirbst aber wie üblich, wenn du fünf erreichst",
        ),
    ] {
        let text = displayed_text(loc, "virtue.unaging").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.unaging: displayed text {text:?} does not state the \
                 Decrepitude-4/5 waiver (ArMDE:5187-5190)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-332: virtue.unaging's description must state the \
         Decrepitude-4/5 waiver in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-350's residual half: virtue.wise_one's exclusive-choice gate is genuinely
/// fully computed (F-349), but the certification missed the Wealthy/Poor
/// normal-interaction clarification and the male-and-female eligibility note.
/// `classification` stays `creation_effect`.
#[test]
fn wise_one_states_its_wealthy_poor_and_sex_inclusive_clauses() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.wise_one"),
        Classification::CreationEffect,
        "F-349/F-350: the exclusive Arcane/Academic authorization is genuinely computed; the \
         Wealthy/Poor clarification and the sex-inclusive note are missing, which does not \
         change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "Wealthy Virtue and Poor Flaw affect you normally",
        ),
        ("en", &loc_en, "available to male and female characters"),
        (
            "de",
            &loc_de,
            "Die Große Tugend Wohlstand und der Große Fehler Arm betreffen dich normal",
        ),
        (
            "de",
            &loc_de,
            "steht männlichen und weiblichen Charakteren zur Verfügung",
        ),
    ] {
        let text = displayed_text(loc, "virtue.wise_one").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.wise_one: displayed text {text:?} does not state {needle:?} \
                 (ArMDE:5257-5260)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-350: virtue.wise_one's description must state the \
         Wealthy/Poor clarification and the sex-inclusive note in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-301: virtue.subtle_magic's own summary already states its no-gestures
/// permission, so the coarse screen never flagged ArMDE:5075's "You gain no
/// benefits from using normal gestures but gain the normal benefit for
/// exaggerated gestures". `classification` stays `in_play_effect`.
#[test]
fn subtle_magic_states_its_gesture_exclusion() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.subtle_magic"),
        Classification::InPlayEffect,
        "F-301: the no-penalty-without-gestures rule is genuinely computed; the \
         normal/exaggerated-gesture clause is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "gain the normal benefit for exaggerated gestures",
        ),
        (
            "de",
            &loc_de,
            "erhältst aber weiterhin den normalen Bonus für übertriebene Gesten",
        ),
    ] {
        let text = displayed_text(loc, "virtue.subtle_magic").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.subtle_magic: displayed text {text:?} does not state the \
                 exaggerated-gesture clause (ArMDE:5073-5076)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-301: virtue.subtle_magic's description must state the \
         exaggerated-gesture clause in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-353: virtue.withstand_casting's own summary already states its own core
/// mechanic (a signed-shaped "1 less Fatigue level"), so the coarse screen
/// never flagged the 22-line passage's other rules. Not added to
/// `PENDING_DROPPED_CLAUSE` (untested against the false-positive risk
/// `town_magistrate`/`unaging` hit; a dedicated test is the safer carrier).
/// This test pins the Vulnerable-Casting ordering rule (ArMDE:5269).
/// `classification` stays `in_play_effect`.
#[test]
fn withstand_casting_states_its_vulnerable_casting_ordering() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "virtue.withstand_casting"),
        Classification::InPlayEffect,
        "the Fatigue-loss reduction is genuinely computed; the Vulnerable-Casting ordering rule \
         is missing, which does not change the classification"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "apply the extra loss from Vulnerability first",
        ),
        (
            "de",
            &loc_de,
            "wende zuerst den zusätzlichen Verlust durch Anfälligkeit an",
        ),
    ] {
        let text = displayed_text(loc, "virtue.withstand_casting").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/virtue.withstand_casting: displayed text {text:?} does not state the \
                 Vulnerable-Casting ordering rule (ArMDE:5261-5282)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2d (tmp/x2d-verdicts.md), F-353: virtue.withstand_casting's description must state the \
         Vulnerable-Casting ordering rule in both locales:\n{}",
        offenders.join("\n")
    );
}

// ---------------------------------------------------------------------------
// X2e (`tmp/x2-worklist.md` rows 204-260, ArMDE:5641-5979) — see
// `tmp/x2e-verdicts.md` for the full per-entry citation and rationale this file
// intentionally does not re-derive inline.
// ---------------------------------------------------------------------------

/// X2e: `flaw.busybody` is this slice's only reclassification with no
/// existing pending-list mechanism to bite on it — it sat on
/// `uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`, whose row this slice
/// overturns rather than confirms. D50: ArMDE:5763-5764's "unless they choose
/// to apply this Flaw specifically to such people at character creation" is a
/// real creation-time scope choice a player must know to make, the same shape
/// as X2b's `virtue.indescribable_face` overturn.
#[test]
fn busybody_reclassifies_and_states_its_creation_time_scope_choice() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "flaw.busybody"),
        Classification::UncomputedRule,
        "D50: ArMDE:5763-5764's creation-time scope choice (whether the Flaw's gossip network \
         extends to the covenant's lower-class members) reaches the player nowhere; \
         flaw.busybody must reclassify to uncomputed_rule"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "unless they choose to apply this Flaw specifically to such people at character \
             creation",
        ),
        (
            "de",
            &loc_de,
            "es sei denn, sie entscheiden bei der Charaktererschaffung, diesen Fehler \
             ausdrücklich auf solche Personen anzuwenden",
        ),
    ] {
        let text = displayed_text(loc, "flaw.busybody").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/flaw.busybody: displayed text {text:?} does not state the creation-time \
                 scope choice (ArMDE:5761-5764)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2e (tmp/x2e-verdicts.md), D50: flaw.busybody's description must state the \
         creation-time scope choice in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-401: flaw.deleterious_circumstances's own summary already states the
/// halving (a signed-shaped clause, D58's surfaced-modifier family), so the
/// coarse screen never flagged the circumstance taxonomy (state/target/place
/// examples, ArMDE:5917-5920) that determines when the halving applies.
/// **D20/D67 correction (X2f Phase 2):** X2e's original reading kept this
/// `in_play_effect` on the theory that a surfaced-only effect (a bare
/// `special_casting_mod: circumstantial` label with no number) satisfies
/// `in_play_effect` — D20 rules that it does not: a bare category label tells
/// the player nothing, so the rule's only real carrier is `description`,
/// which is what `uncomputed_rule` is for. D67 confirms an `uncomputed_rule`
/// entry keeps whatever it does compute, so `effects` is unchanged.
#[test]
fn deleterious_circumstances_states_its_circumstance_taxonomy() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "flaw.deleterious_circumstances"),
        Classification::UncomputedRule,
        "D20/D67: a bare special_casting_mod marker with no number is a surfaced-only effect, so \
         the rule's only carrier is the description text — uncomputed_rule, not in_play_effect, \
         though the effect itself stays computed"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "such as sitting or wet"),
        ("de", &loc_de, "beispielsweise sitzend oder nass"),
    ] {
        let text = displayed_text(loc, "flaw.deleterious_circumstances").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/flaw.deleterious_circumstances: displayed text {text:?} does not state \
                 the circumstance taxonomy (ArMDE:5917-5920)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2e (tmp/x2e-verdicts.md), F-401: flaw.deleterious_circumstances's description must \
         state the circumstance taxonomy in both locales:\n{}",
        offenders.join("\n")
    );
}

/// F-421: flaw.disjointed_magic computes nothing but a bare
/// `special_casting_mod` marker, and neither of its two stated clauses
/// reaches the player. **D20/D67 correction (X2f Phase 2):** the same
/// surfaced-only reasoning as `flaw.deleterious_circumstances` applies here —
/// `classification` flips to `uncomputed_rule`, `effects` is unchanged.
#[test]
fn disjointed_magic_states_its_two_clauses() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "flaw.disjointed_magic"),
        Classification::UncomputedRule,
        "D20/D67: a bare special_casting_mod marker with no number is a surfaced-only effect, so \
         the rule's only carrier is the description text — uncomputed_rule, not in_play_effect, \
         though the effect itself stays computed"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "no enchantment bonuses from Techniques and Forms",
        ),
        (
            "de",
            &loc_de,
            "keine Verzauberungsboni von Techniken und Formen",
        ),
    ] {
        let text = displayed_text(loc, "flaw.disjointed_magic").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/flaw.disjointed_magic: displayed text {text:?} does not state the \
                 enchantment-bonus clause (ArMDE:5972-5975)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2e (tmp/x2e-verdicts.md), F-421: flaw.disjointed_magic's description must state both \
         clauses in both locales:\n{}",
        offenders.join("\n")
    );
}

// ---------------------------------------------------------------------------
// X2f (`tmp/x2-worklist.md` rows 261-317, ArMDE:5984-6377, `tmp/x2f-verdicts.md`)
// ---------------------------------------------------------------------------

/// D8 (`docs/vf-audit/decisions.md`): every `supernatural`-category
/// `narrative` entry reclassifies to `uncomputed_rule` ("a capability is a
/// rule"), independent of whether the mechanical-token screen recognizes
/// anything in the passage — these four passages state no signed number, no
/// botch term, and no idiom family the screen already carries, so no pending
/// list bites them; this table is the only thing that does. `(id, why)`.
const X2F_RECLASSIFY_WITH_DESCRIPTION: &[(&str, &str)] = &[
    (
        "flaw.fluctuating_fortune",
        "D8: ArMDE:6150-6153 states the character effectively holds Wealthy one year and Poor \
         the next (one season of work one year, three the next) — a concrete, unmodelled \
         toggling mechanic naming both Virtue/Flaw by name, not mere colour",
    ),
    (
        "flaw.form_monstrosity",
        "D8: ArMDE:6162-6185 states a real quantity — \"1 pawn of Muto vis may be extracted from \
         the corpse of a monstrous character\" — computed nowhere (the entry ships only a `form` \
         parameter, no effects)",
    ),
    (
        "flaw.greater_malediction",
        "D8/F-442: ArMDE:6210-6213's curse-design guidance (\"effects... should be comparable to \
         those of other Major Flaws\") is the storyguide-actionable content D50 requires reach \
         the player as text, and the category (supernatural) independently requires \
         uncomputed_rule under D8",
    ),
    (
        "flaw.lesser_malediction",
        "D8/F-442/F-452: the Minor-scale sibling of greater_malediction, ArMDE:6342-6345 — same \
         reading, same trigger",
    ),
];

#[test]
fn x2f_entries_reclassify_to_uncomputed_rule_with_a_description() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    let mut offenders = Vec::new();
    for (id, why) in X2F_RECLASSIFY_WITH_DESCRIPTION {
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
        "X2f (tmp/x2f-verdicts.md): these entries must reclassify to `uncomputed_rule` and carry \
         a description in every locale — a Phase 2 data change, not yet landed:\n{}",
        offenders.join("\n")
    );
}

/// F-… (`tmp/x2f-verdicts.md`): flaw.environmental_magic_condition's own
/// summary already states the halving (D58's surfaced-modifier family, the
/// same `special_casting_mod: circumstantial` shape as
/// flaw.deleterious_circumstances/flaw.disjointed_magic), so the coarse
/// screen never flagged the missing severity comparison
/// (ArMDE:6020-6023's second sentence). **D20/D67 correction (X2f Phase 2):**
/// X2f Phase 1 originally "overturned" this back to `in_play_effect` on the
/// same surfaced-only theory X2e used for its two siblings — D20 rejects that
/// theory for all 19 of its entries, so this one reclassifies to
/// `uncomputed_rule` too, keeping its `special_casting_mod` effect per D67.
#[test]
fn environmental_magic_condition_states_its_severity_comparison() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "flaw.environmental_magic_condition"),
        Classification::UncomputedRule,
        "D20/D67: a bare special_casting_mod marker with no number is a surfaced-only effect, so \
         the rule's only carrier is the description text — uncomputed_rule, not in_play_effect, \
         though the effect itself stays computed"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        (
            "en",
            &loc_en,
            "more restrictive than the Hermetic Flaw Deleterious Circumstances",
        ),
        (
            "de",
            &loc_de,
            "einschränkender sein sollte als der Hermetische Fehler Abträgliche Umstände",
        ),
    ] {
        let text = displayed_text(loc, "flaw.environmental_magic_condition").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/flaw.environmental_magic_condition: displayed text {text:?} does not \
                 state the severity comparison to Deleterious Circumstances (ArMDE:6020-6023)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2f (tmp/x2f-verdicts.md): flaw.environmental_magic_condition's description must state \
         the severity comparison in both locales:\n{}",
        offenders.join("\n")
    );
}

/// `tmp/rules-md-audit-2026-09-30.md` item 5 / D3: flaw.imagined_folk_
/// tradition_vulnerability's "a score of 1 (but no more)" cap on Faerie Lore
/// (ArMDE:6280-6283) is D3-inexpressible (no score-cap effect exists), the
/// same shape as flaw.magical_fascination's own PENDING_DROPPED_CLAUSE row —
/// but unlike that row, the generic screen cannot see this gap at all: the
/// shipped summary IS the passage's own first sentence, "...has other game
/// mechanical effects as well", which happens to trip the "named rulebook
/// term" idiom (`game mechanical effects`) without stating what the effect
/// IS. `classification` stays `creation_effect` (the Faerie Lore permission
/// itself is computed via `ability_authorization`); only the cap is missing.
#[test]
fn imagined_folk_tradition_vulnerability_states_its_faerie_lore_cap() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "flaw.imagined_folk_tradition_vulnerability"),
        Classification::CreationEffect,
        "the Faerie Lore permission is genuinely computed via ability_authorization; only the \
         score-of-1 cap is missing, which does not change the classification (D3)"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "score of 1 (but no more)"),
        ("de", &loc_de, "Wert von 1 (aber nicht mehr)"),
    ] {
        let text =
            displayed_text(loc, "flaw.imagined_folk_tradition_vulnerability").unwrap_or_default();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/flaw.imagined_folk_tradition_vulnerability: displayed text {text:?} does \
                 not state the score-of-1 Faerie Lore cap (ArMDE:6280-6283)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2f (tmp/x2f-verdicts.md), tmp/rules-md-audit-2026-09-30.md item 5: \
         flaw.imagined_folk_tradition_vulnerability's description must state the score-of-1 \
         Faerie Lore cap in both locales:\n{}",
        offenders.join("\n")
    );
}

// ---------------------------------------------------------------------------
// X2g (`tmp/x2-worklist.md` rows 318-374, ArMDE:6382-6708, `tmp/x2g-verdicts.md`)
// ---------------------------------------------------------------------------

/// Four entries this slice's own read finds narrative-but-actionable, none
/// caught by any pending list, plus one PDC entry (`flaw.savantism`) whose fix
/// is a full reclassification rather than merely a description.
/// `flaw.prohibition`/`flaw.restricted_power` are `categories:["supernatural"]`
/// (D8: "a capability is a rule"). `flaw.oath_of_fealty` states a hard
/// eligibility rule — "Magi are forbidden from taking Oaths of Fealty by the
/// Hermetic Code" (ArMDE:6512-6515) — no `Prereq`/`incompatible_with`
/// enforces (D50: a player building a magus would get it wrong without the
/// text). `flaw.regular` states a compulsory seasonal expenditure D62 rules
/// stays text ("The season rules of Landed Noble, License of Absence, Lone
/// Redcap, Redcap, Wealthy, Poor and Regular stay in description"; D62).
/// `flaw.savantism` is D67's own shape: the two `ability_score_cap_*` effects
/// (X6b) compute the score caps, but the halved starting XP, halved future
/// Advancement Totals, and the +3-not-+1 specialization roll (F-510 points
/// 1-3) are computed nowhere, so D67 makes the whole entry `uncomputed_rule`
/// regardless of what else it computes. `(id, why)`.
const X2G_RECLASSIFY_WITH_DESCRIPTION: &[(&str, &str)] = &[
    (
        "flaw.prohibition",
        "D8: categories:[\"supernatural\"], ArMDE:6638-6641 — a Geas's obey-or-be-cursed \
         capability is a rule regardless of table adjudication; the shipped summary already \
         states it, so only the classification is owed, no new description text",
    ),
    (
        "flaw.restricted_power",
        "D8: categories:[\"supernatural\"], ArMDE:6687-6690 — the ceremony/limited-target \
         activation mechanism is computed nowhere and the shipped summary stops before it",
    ),
    (
        "flaw.oath_of_fealty",
        "D50: ArMDE:6512-6515's \"Magi are forbidden from taking Oaths of Fealty by the \
         Hermetic Code\" is a hard eligibility rule no Prereq/incompatible_with enforces",
    ),
    (
        "flaw.regular",
        "D62/F-503: ArMDE:6675-6678's compulsory seasonal worship activity and the \
         Poor-Regular consequence (\"effectively has no free seasons\") are D62's own named \
         case for staying text (\"The season rules of ... Regular stay in description\")",
    ),
    (
        "flaw.savantism",
        "D67/F-510 (points 1-3): the two ability_score_cap_* effects (X6b) compute the score \
         caps, but ArMDE:6703-6708's halved starting XP, halved future Advancement Totals, and \
         +3-not-+1 specialization roll are computed nowhere — D67: any stated rule computed \
         nowhere makes the whole entry uncomputed_rule",
    ),
];

#[test]
fn x2g_entries_reclassify_to_uncomputed_rule_with_a_description() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    let mut offenders = Vec::new();
    for (id, why) in X2G_RECLASSIFY_WITH_DESCRIPTION {
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
        "X2g (tmp/x2g-verdicts.md): these entries must reclassify to `uncomputed_rule` and \
         carry displayed rules text in every locale — a Phase 2 data change, not yet landed:\n{}",
        offenders.join("\n")
    );
}

/// D61/OQ-4 correction: `flaw.poor_eyesight`'s `uncomputed_clauses.rs`
/// `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE` row claimed the whole passage
/// ("rolls involving sight, including rolls to attack and defend, are at –3")
/// is computed via its two `combat_mod` effects — but those only cover
/// attack/defense. D61 rules the broader "rolls involving sight" penalty (a
/// table call, the same shape as Poor Hearing/Sharp Ears/Keen Vision) stays
/// text. So that row is overturned (removed), and this dedicated test — the
/// same shape as `environmental_magic_condition_states_its_severity_comparison`
/// — pins the specific missing clause instead.
#[test]
fn poor_eyesight_states_its_non_combat_sight_penalty() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "flaw.poor_eyesight"),
        Classification::InPlayEffect,
        "D61: the attack/defense -3 stays computed via combat_mod; only the broader sight-roll \
         penalty is text, so the classification itself does not change"
    );

    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();
    let mut offenders = Vec::new();
    for (lang, loc, needle) in [
        ("en", &loc_en, "rolls involving sight"),
        ("de", &loc_de, "würfe, die sehen beinhalten"),
    ] {
        let text = displayed_text(loc, "flaw.poor_eyesight")
            .unwrap_or_default()
            .to_lowercase();
        if !text.contains(needle) {
            offenders.push(format!(
                "{lang}/flaw.poor_eyesight: displayed text {text:?} does not state the general \
                 sight-roll penalty, only attack/defense (ArMDE:6606-6609)"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "X2g (tmp/x2g-verdicts.md), D61: flaw.poor_eyesight's description must state the general \
         sight-roll penalty in both locales:\n{}",
        offenders.join("\n")
    );
}

/// D67: `flaw.primogeniture_lineage`'s `uncomputed_clauses.rs`
/// `NO_RULE_DESPITE_TOKEN` row already establishes that the passage's one
/// genuinely mechanical clause — "This Flaw can only be taken by magi of
/// House Verditius" — is fully computed by the entry's own
/// `prerequisites: all(order_member, house.verditius)`, and nothing else in
/// the passage states a rule (the rest is fictional-succession colour). So
/// D67 places it at `creation_effect`, not `uncomputed_rule` — pinned here
/// because the generic D67 guard (`data_integrity.rs::every_vf_is_classified`)
/// only requires "not narrative if computed," not which computed class.
#[test]
fn primogeniture_lineage_reclassifies_to_creation_effect() {
    let rs = load_ruleset();
    assert_eq!(
        classification_of(&rs, "flaw.primogeniture_lineage"),
        Classification::CreationEffect,
        "X2g (tmp/x2g-verdicts.md), D67: the only stated rule is already computed via \
         `prerequisites`, so this reclassifies to creation_effect, not uncomputed_rule"
    );
}
