//! X9c PHASE 1 (red checkpoint) — `tmp/x9c-plan.md`, `tmp/x9c-verdicts.md`,
//! `tmp/x9c-verdicts-flaws-a.md`, `tmp/x9c-verdicts-flaws-b.md`,
//! `docs/vf-audit/decisions.md` D77, D78 (plus D75's `Prereq::CharacterType`
//! gate, reused by D78.1).
//!
//! GREEN (phase 2 landed): every data/engine fix below is shipped —
//! `virtue.minor_magical_focus`'s range fix, `WarpingGrant.score` dropped,
//! `virtue.rabbi`'s and `flaw.companion_animal`'s prerequisites, and
//! `flaw.pagan_minor` added in core/en/de + the DE anchor sidecar. See
//! `tmp/x9c-handover.md` for the RED→GREEN record and the per-item edit list.
//!
//! D77.1's own fix — extending
//! `rules_source_provenance.rs::no_source_range_runs_past_the_heading_that_follows_it`
//! with `OWN_BLOCKQUOTED_SIDEBARS` — lives in that file, not here: the guard
//! change IS the test the brief describes, and it caught exactly
//! `virtue.perfectus` (F-215) and `virtue.tainted_treasure` (F-309), both now
//! fixed (their ranges no longer reach the sidebar at all).

use serde_json::Value;

use arm_rules::derived::derived_totals;
use arm_rules::effective::warping_score;
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");
const VF_JSON: &str = include_str!("../../../rules/core/virtues_flaws.json");
const EN_VF: &str = include_str!("../../../rules/i18n/en/virtues_flaws.json");
const DE_VF: &str = include_str!("../../../rules/i18n/de/virtues_flaws.json");
const RULES_MD: &str = include_str!("../RULES.md");

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: VF_JSON,
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

/// Same shape as every other X-series integration test file's `entity`
/// helper (e.g. `rc_review_c_fixes.rs`) — duplicated since integration test
/// binaries cannot share private helpers.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn issue_codes(result: &arm_rules::validation::ValidationResult) -> Vec<&str> {
    result.issues.iter().map(|i| i.code.as_str()).collect()
}

fn item<'a>(rs: &'a Ruleset, id: &str) -> &'a PointItem {
    rs.item(&Id::new(id))
        .unwrap_or_else(|| panic!("{id} must exist in the shipped catalogue"))
}

/// The raw `rules/core/virtues_flaws.json` entry for `id`, as a [`Value`] —
/// used where a test must see fields `PointItem` does not deserialize at all
/// (a raw `source.lines` pair) or must see the *absence* of a JSON key
/// (`WarpingGrant.score`, D77.3), which a successfully-deserialized `Effect`
/// can never show either way.
fn raw_entry(id: &str) -> Value {
    let all: Value = serde_json::from_str(VF_JSON).expect("virtues_flaws.json is valid JSON");
    all.as_array()
        .expect("virtues_flaws.json is a top-level array")
        .iter()
        .find(|v| v.get("id").and_then(Value::as_str) == Some(id))
        .unwrap_or_else(|| panic!("{id} must exist in rules/core/virtues_flaws.json"))
        .clone()
}

// ---------------------------------------------------------------------------
// Item 2 — virtue.minor_magical_focus's range is too short (F-200).
// ---------------------------------------------------------------------------

/// F-200 (`tmp/x9c-verdicts.md`): the range previously shipped as
/// `[4536, 4538]` stopped after the entry's first body paragraph, cutting off
/// its own remaining body
/// (ArMDE:4540-4542: "add the lowest applicable Art score twice", "only one
/// Magical Focus") and its own sidebar `> #### Sample Minor Magical Foci`
/// (ArMDE:4544-4557, confirmed to belong to THIS entry, not the next
/// heading's — Muqta' at :4559 is unrelated Social Status content — the same
/// read that puts `virtue.minor_magical_focus` in Guard B's own
/// `OWN_BLOCKQUOTED_SIDEBARS` allowlist). D77.2 ("an entry's range is its
/// heading plus its body") therefore extends the range to the sidebar's own
/// end, exactly as `virtue.major_magical_focus` already does for its sidebar
/// (ArMDE:4399-4422). RULES.md already cites `ArMDE:4536-4542` for this entry
/// (its V/F table and the MagicalFocus row), so the JSON range disagrees with
/// RULES.md too.
#[test]
fn minor_magical_focus_range_reaches_its_own_sidebar() {
    let entry = raw_entry("virtue.minor_magical_focus");
    let lines = entry["source"]["lines"]
        .as_array()
        .expect("source.lines is an array");
    assert_eq!(
        lines,
        &[Value::from(4536), Value::from(4557)],
        "virtue.minor_magical_focus must cite ArMDE:4536-4557 (its own body plus its own \
         \"Sample Minor Magical Foci\" sidebar), not the shipped {lines:?} — F-200"
    );
}

// ---------------------------------------------------------------------------
// Item 3 — D77.3: WarpingGrant.score is an unread, disagreeable second copy.
// ---------------------------------------------------------------------------

/// Pin (expected GREEN already — `effective/warping.rs::warping_grant_points_in`
/// already ignores the stored `score` and derives it from the point total):
/// `flaw.warped_by_magic` grants 5 Warping Points (ArMDE:7019-7021), which the
/// advancement curve (5/15/30/50/75) inverts to Warping Score 1. Surfaced via
/// `derived::derived_totals`, the DTO the frontend actually reads.
#[test]
fn warped_by_magic_shows_warping_score_one_from_its_five_points() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("flaw.warped_by_magic")]);
    assert_eq!(
        warping_score(&e, &rs),
        1,
        "ArMDE:7019-7021: Warped by Magic's 5 Warping Points must derive Warping Score 1"
    );
    let totals = derived_totals(&e, &rs);
    assert_eq!(
        totals.warping_points, 5,
        "the DTO must surface the 5 granted Warping Points"
    );
    assert_eq!(
        totals.warping_score, 1,
        "the DTO must surface the derived Warping Score, not a stored one"
    );
}

/// D77.3: the shipped `warping_grant` effect no longer carries a `score` key
/// — it was an unread second copy of the same fact `warping_score` derives
/// from `points`, and could only ever disagree. The field is dropped from
/// `types.rs::Effect::WarpingGrant` and from its one carrier in
/// `rules/core/virtues_flaws.json` (`flaw.warped_by_magic`).
#[test]
fn warped_by_magic_effect_no_longer_carries_a_stored_score() {
    let entry = raw_entry("flaw.warped_by_magic");
    let effects = entry["effects"]
        .as_array()
        .expect("flaw.warped_by_magic has an effects array");
    let grant = effects
        .iter()
        .find(|e| e.get("type").and_then(Value::as_str) == Some("warping_grant"))
        .expect("flaw.warped_by_magic carries a warping_grant effect");
    assert!(
        grant.get("score").is_none(),
        "D77.3: the warping_grant effect must no longer carry a stored `score` key (an unread \
         second copy of the point-derived Warping Score) — found: {grant:?}"
    );
}

// ---------------------------------------------------------------------------
// Item 4 — F-240: virtue.rabbi requires Educated (Hebrew), undeclared.
// ---------------------------------------------------------------------------

/// F-240 (`tmp/x9c-verdicts.md`): ArMDE:4832 "must take the Educated
/// (Hebrew) Virtue", exactly the same sentence shape already encoded as a
/// `has virtue.educated_hebrew` prerequisite on `virtue.shamash` (ArMDE:4944)
/// and `virtue.sofer` (ArMDE:4996) — mirrors
/// `x5_prerequisites.rs::shamash_requires_educated_hebrew`.
#[test]
fn rabbi_requires_educated_hebrew() {
    let rs = load_ruleset();

    let without = entity("companion", vec![sel("virtue.rabbi")]);
    let without_result = validate(&without, &rs);
    assert!(
        issue_codes(&without_result).contains(&"prereq_not_met"),
        "ArMDE:4832 'must take the Educated (Hebrew) Virtue' — Rabbi without it must be \
         refused, got: {:?}",
        issue_codes(&without_result)
    );

    let with = entity(
        "companion",
        vec![sel("virtue.rabbi"), sel("virtue.educated_hebrew")],
    );
    let with_result = validate(&with, &rs);
    assert!(
        !issue_codes(&with_result).contains(&"prereq_not_met"),
        "Rabbi + Educated (Hebrew) must stay legal, got: {:?}",
        issue_codes(&with_result)
    );
}

// ---------------------------------------------------------------------------
// Item 5 — D78.1: Companion Animal gets D75's animal-character-type gate.
// ---------------------------------------------------------------------------

/// D78.1: `flaw.companion_animal` ("Minor, Social Status, animals only",
/// ArMDE:5806) gets the same gate D75/F-556 put on `virtue.domestic_animal` —
/// a `Prereq::CharacterType` naming an animal character type no profile has,
/// so no human character type can ever satisfy it. Mirrors
/// `rc_review_c_fixes.rs::f556_domestic_animal_is_refused_for_a_human_character_type`.
#[test]
fn companion_animal_is_refused_for_a_human_character_type() {
    let rs = load_ruleset();
    let target = Id::new("flaw.companion_animal");
    let e = entity("grog", vec![sel(target.as_str())]);
    let result = validate(&e, &rs);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.context.as_ref() == Some(&target)),
        "D78.1: a grog selecting flaw.companion_animal raises no issue today (issues: {:?}) — \
         no human character type may take this entry, same gate as virtue.domestic_animal (D75)",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Item 6 — D78.2: flaw.pagan_minor, a Minor twin of flaw.pagan.
// ---------------------------------------------------------------------------

/// D78.2: ArMDE:6571 "*Major or Minor, Personality*" — one entry, one
/// passage, two magnitudes, the same shape `virtue.potent_magic_major`/
/// `_minor` already encode for their own shared passage. `flaw.pagan` keeps
/// its existing bare id for Major (no rename — the False Power precedent, see
/// RULES.md "Dual-magnitude split"); `flaw.pagan_minor` is the new twin.
#[test]
fn pagan_minor_exists_and_is_a_minor_personality_flaw() {
    let rs = load_ruleset();
    let new = item(&rs, "flaw.pagan_minor");
    assert_eq!(new.kind, ItemKind::Flaw, "flaw.pagan_minor must be a Flaw");
    assert_eq!(
        new.magnitude,
        Magnitude::Minor,
        "D78.2: flaw.pagan_minor is the Minor twin of flaw.pagan (ArMDE:6571 \"Major or Minor\")"
    );
    assert_eq!(
        new.categories,
        vec!["personality".to_string()],
        "flaw.pagan_minor must copy flaw.pagan's Personality category"
    );
}

/// D78.2: "mutually incompatible with it" — the same symmetric hard-block
/// shape `x2t_true_friend.rs::flaw_true_friend_major_and_minor_exclude_each_other`
/// pins for True Friend's own Major/Minor twin pair.
#[test]
fn pagan_and_pagan_minor_are_mutually_incompatible() {
    let rs = load_ruleset();
    let pagan = item(&rs, "flaw.pagan");
    let pagan_minor = item(&rs, "flaw.pagan_minor");

    assert!(
        pagan
            .incompatible_with
            .contains(&Id::new("flaw.pagan_minor")),
        "flaw.pagan must declare incompatible_with flaw.pagan_minor"
    );
    assert!(
        pagan_minor
            .incompatible_with
            .contains(&Id::new("flaw.pagan")),
        "flaw.pagan_minor must declare incompatible_with flaw.pagan (symmetric — CLAUDE.md: \
         incompatibilities must be symmetric)"
    );

    let e = entity(
        "companion",
        vec![sel("flaw.pagan"), sel("flaw.pagan_minor")],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"incompatible"),
        "holding both Pagan and Pagan (Minor) must be a hard block, got: {:?}",
        issue_codes(&result)
    );
}

/// D78.2: the same single passage covers both magnitudes, so `source` (file,
/// lines, anchor) must be identical on both entries — there is no separate
/// heading for the Minor version to cite, exactly as
/// `x2t_true_friend.rs`'s twins cite their shared passage.
#[test]
fn pagan_minor_cites_the_same_passage_as_pagan() {
    let pagan = raw_entry("flaw.pagan");
    let pagan_minor_all: Value = serde_json::from_str(VF_JSON).unwrap();
    let pagan_minor = pagan_minor_all
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v.get("id").and_then(Value::as_str) == Some("flaw.pagan_minor"))
        .unwrap_or_else(|| panic!("flaw.pagan_minor must exist in rules/core/virtues_flaws.json"));
    assert_eq!(
        pagan_minor["source"], pagan["source"],
        "flaw.pagan_minor must cite the SAME source (file, lines, anchor) as flaw.pagan — \
         ArMDE:6570-6573, the \"Pagan\" heading, is the only passage either magnitude has"
    );
}

/// D78.2: name/summary are copied verbatim from `flaw.pagan` in both locales
/// — unlike True Friend/True Love, Pagan keeps the SAME name for both
/// magnitudes (no rename), since the book names one entry "Pagan" for both.
#[test]
fn pagan_minor_en_de_name_and_summary_match_pagan_verbatim() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    for (loc, lang) in [(&loc_en, "en"), (&loc_de, "de")] {
        let pagan_name = loc
            .display_name(&Id::new("flaw.pagan"))
            .unwrap_or_else(|| panic!("flaw.pagan must carry a {lang} display name"));
        let new_name = loc
            .display_name(&Id::new("flaw.pagan_minor"))
            .unwrap_or_else(|| panic!("flaw.pagan_minor must carry a {lang} display name"));
        assert_eq!(
            new_name, pagan_name,
            "flaw.pagan_minor's {lang} name must match flaw.pagan's verbatim (same entry name, \
             both magnitudes)"
        );

        let pagan_summary = loc
            .summary(&Id::new("flaw.pagan"))
            .unwrap_or_else(|| panic!("flaw.pagan must carry a {lang} summary"));
        let new_summary = loc
            .summary(&Id::new("flaw.pagan_minor"))
            .unwrap_or_else(|| panic!("flaw.pagan_minor must carry a {lang} summary"));
        assert_eq!(
            new_summary, pagan_summary,
            "flaw.pagan_minor's {lang} summary must be copied verbatim from flaw.pagan — same \
             cited passage, D78.2"
        );
    }
}

// ---------------------------------------------------------------------------
// Item 7 — D78.3: the RULES.md known-source-errata note.
// ---------------------------------------------------------------------------

/// D78.3/D25: a short "known source errata" note in `crates/arm-rules/RULES.md`
/// listing the four index omissions (`virtue.lupus_the_wolf` ArMDE:4336,
/// `virtue.minor_enchantments` :4533, `virtue.turb_trained` :5180,
/// `flaw.true_love_minor` :6872/:6877), the three index disagreements
/// (`flaw.bound_to_role_role` :5736 vs :5392, `flaw.broken_vessel` :5754 vs
/// :5393, `flaw.weak_personality` :7077 vs :5518 — D25's original finding)
/// and the stray Folk Magic entry in the Flaw index (ArMDE:5545 vs the Virtue
/// heading ArMDE:3907) — each with both citations and the side taken (D25: the
/// descriptor always wins; the index is never a data source).
#[test]
fn rules_md_carries_the_known_source_errata_note() {
    assert!(
        RULES_MD.to_lowercase().contains("known source errata"),
        "crates/arm-rules/RULES.md must carry a \"known source errata\" note (D78.3/D25)"
    );

    let entries_with_both_citations = [
        ("Lupus (the Wolf)", "4336"),
        ("Minor Enchantments", "4533"),
        ("Turb Trained", "5180"),
        ("True Love", "6872"),
        ("Bound to (Role) Role", "5736"),
        ("Bound to (Role) Role", "5392"),
        ("Broken Vessel", "5754"),
        ("Broken Vessel", "5393"),
        ("Weak Personality", "7077"),
        ("Weak Personality", "5518"),
        ("Folk Magic", "5545"),
        ("Folk Magic", "3907"),
    ];
    let missing: Vec<&str> = entries_with_both_citations
        .iter()
        .filter(|(name, line)| !RULES_MD.contains(name) || !RULES_MD.contains(line))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        missing.is_empty(),
        "RULES.md's known-source-errata note is missing a citation for: {missing:?} (D78.3)"
    );
}
