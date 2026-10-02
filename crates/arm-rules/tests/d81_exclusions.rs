//! D81.1-3 (`docs/vf-audit/decisions.md` D81, `tmp/incompat-audit.md` findings
//! #1, #4-6, #7).
//!
//! - D81.1: `flaw.incompatible_arts` carries a symmetric `incompatible_with`
//!   toward `flaw.deficient_technique`/`flaw.deficient_form` (ArMDE:6292,
//!   "may not be combined with a Deficiency").
//! - D81.2: `virtue.supernatural_beauty` (ArMDE:5095), `flaw.envied_beauty`
//!   (ArMDE:6014), and `flaw.uncontrollable_strength` (ArMDE:6909) each carry
//!   a `Prereq::CharacteristicMin` `prerequisites` field.
//! - D81.3: `flaw.broken_vessel` (ArMDE:5755) carries
//!   `Any([AbilityCategoryScoreMin{category:"supernatural",score:1},
//!   AnyArtMin{score:1}])` — "at least one Supernatural Ability or Art
//!   normally improved through experience points" is a disjunction, not a
//!   single predicate, so the composition lives in data (`Prereq::Any`), not
//!   as one hard-coded engine variant (CLAUDE.md's data-driven rule: the
//!   engine must not assume *which* category a rule names).

use arm_rules::Characteristic;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use std::collections::BTreeMap;

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
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

/// Same shape as `x4_incompatibilities.rs::entity` — duplicated since
/// integration test binaries cannot share private helpers.
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

fn issue_codes(e: &Entity, rs: &Ruleset) -> Vec<String> {
    validate(e, rs).issues.into_iter().map(|i| i.code).collect()
}

// ---------------------------------------------------------------------------
// D81.1 — Incompatible Arts excludes the Deficiencies outright. ArMDE:6292:
// "This Flaw may be taken repeatedly with different combinations, but may not
// be combined with a Deficiency (see page 125)." A flat, symmetric
// `incompatible_with` — none of the three entries carries it today. Both
// items require `hermetically_trained`, so the test entity is a magus.
// ---------------------------------------------------------------------------

#[test]
fn incompatible_arts_excludes_deficient_technique() {
    let rs = load_ruleset();
    rs.item(&Id::new("flaw.incompatible_arts"))
        .expect("flaw.incompatible_arts must exist in the shipped catalogue");
    rs.item(&Id::new("flaw.deficient_technique"))
        .expect("flaw.deficient_technique must exist in the shipped catalogue");

    let e = entity(
        "magus",
        vec![
            sel("flaw.incompatible_arts"),
            Selection::with_params(
                Id::new("flaw.deficient_technique"),
                BTreeMap::from([("technique".into(), Id::new("art.creo"))]),
            ),
        ],
    );
    let codes = issue_codes(&e, &rs);
    assert!(
        codes.contains(&"incompatible".to_string()),
        "ArMDE:6292 'may not be combined with a Deficiency' — \
         flaw.incompatible_arts + flaw.deficient_technique must be incompatible, got: {codes:?}"
    );
}

#[test]
fn incompatible_arts_excludes_deficient_form() {
    let rs = load_ruleset();
    rs.item(&Id::new("flaw.incompatible_arts"))
        .expect("flaw.incompatible_arts must exist in the shipped catalogue");
    rs.item(&Id::new("flaw.deficient_form"))
        .expect("flaw.deficient_form must exist in the shipped catalogue");

    let e = entity(
        "magus",
        vec![
            sel("flaw.incompatible_arts"),
            Selection::with_params(
                Id::new("flaw.deficient_form"),
                BTreeMap::from([("form".into(), Id::new("art.ignem"))]),
            ),
        ],
    );
    let codes = issue_codes(&e, &rs);
    assert!(
        codes.contains(&"incompatible".to_string()),
        "ArMDE:6292 'may not be combined with a Deficiency' — \
         flaw.incompatible_arts + flaw.deficient_form must be incompatible, got: {codes:?}"
    );
}

// ---------------------------------------------------------------------------
// D81.2 — Characteristic-floor prerequisite. Supernatural Beauty (ArMDE:5095)
// and Envied Beauty (ArMDE:6014): "A character lacking a positive Presence
// score may not have this Virtue/Flaw" (Presence >= 1). Uncontrollable
// Strength (ArMDE:6909): "This Flaw may not be taken if the character's
// Strength is below 0" (Strength >= 0). None of the three carries a
// `prerequisites` field today, so nothing enforces this yet.
// ---------------------------------------------------------------------------

/// Exercises all three states a `CharacteristicMin` floor must distinguish:
/// refused one below the floor, accepted exactly at the floor, and Unknown
/// (a warning, not a silent pass) when the Characteristic was never set —
/// mirroring `Prereq::AgeMin`'s own unset-is-Unknown contract.
fn assert_characteristic_floor(
    rs: &Ruleset,
    item_id: &str,
    characteristic: Characteristic,
    floor: i8,
) {
    rs.item(&Id::new(item_id))
        .unwrap_or_else(|| panic!("{item_id} must exist in the shipped catalogue"));

    let mut below = entity("companion", vec![sel(item_id)]);
    below.characteristics.insert(characteristic, floor - 1);
    let codes = issue_codes(&below, rs);
    assert!(
        codes.contains(&"prereq_not_met".to_string()),
        "{item_id}: {characteristic} {} (one below the floor of {floor}) must be refused, \
         got: {codes:?}",
        floor - 1
    );

    let mut at_floor = entity("companion", vec![sel(item_id)]);
    at_floor.characteristics.insert(characteristic, floor);
    let codes = issue_codes(&at_floor, rs);
    assert!(
        !codes.contains(&"prereq_not_met".to_string()),
        "{item_id}: {characteristic} {floor} (exactly at the floor) must be accepted, \
         got: {codes:?}"
    );

    let unset = entity("companion", vec![sel(item_id)]);
    let codes = issue_codes(&unset, rs);
    assert!(
        codes.contains(&"prereq_unevaluated".to_string()),
        "{item_id}: an unset {characteristic} is genuinely unknown and must warn as \
         unevaluated, not silently pass, got: {codes:?}"
    );
}

#[test]
fn supernatural_beauty_requires_positive_presence() {
    let rs = load_ruleset();
    assert_characteristic_floor(&rs, "virtue.supernatural_beauty", Characteristic::Pre, 1);
}

#[test]
fn envied_beauty_requires_positive_presence() {
    let rs = load_ruleset();
    assert_characteristic_floor(&rs, "flaw.envied_beauty", Characteristic::Pre, 1);
}

#[test]
fn uncontrollable_strength_requires_non_negative_strength() {
    let rs = load_ruleset();
    assert_characteristic_floor(&rs, "flaw.uncontrollable_strength", Characteristic::Str, 0);
}

// ---------------------------------------------------------------------------
// D81.3 — Broken Vessel (ArMDE:5755): "Characters may only take this Flaw if
// they have at least one Supernatural Ability or Art normally improved
// through experience points." No `prerequisites` field today.
// ---------------------------------------------------------------------------

#[test]
fn broken_vessel_requires_a_supernatural_ability_or_art() {
    let rs = load_ruleset();
    rs.item(&Id::new("flaw.broken_vessel"))
        .expect("flaw.broken_vessel must exist in the shipped catalogue");

    let none = entity("companion", vec![sel("flaw.broken_vessel")]);
    let codes = issue_codes(&none, &rs);
    assert!(
        codes.contains(&"prereq_not_met".to_string()),
        "ArMDE:5755: no Supernatural Ability or Art must refuse Broken Vessel, got: {codes:?}"
    );

    let mut with_ability = entity("companion", vec![sel("flaw.broken_vessel")]);
    with_ability.ability_scores = vec![AbilityScore::new(Id::new("ability.dowsing"), 1)];
    let codes = issue_codes(&with_ability, &rs);
    assert!(
        !codes.contains(&"prereq_not_met".to_string()),
        "ArMDE:5755: a Supernatural Ability at 1 must satisfy Broken Vessel, got: {codes:?}"
    );

    let mut with_art = entity("magus", vec![sel("flaw.broken_vessel")]);
    with_art.art_scores = vec![ArtScore::new(Id::new("art.creo"), 1)];
    let codes = issue_codes(&with_art, &rs);
    assert!(
        !codes.contains(&"prereq_not_met".to_string()),
        "ArMDE:5755: an Art at 1 must satisfy Broken Vessel, got: {codes:?}"
    );
}

// ---------------------------------------------------------------------------
// Wizard dead end (review-final.json finding #1, MAJOR): Broken Vessel is
// selected in the VirtuesFlaws phase, but its prerequisite can only ever be
// satisfied by a later purchase (a Supernatural Ability in the Abilities
// phase, or an Art in the Arts phase — `CreationPhase::ALL` orders both
// after `VirtuesFlaws`, and every shipped type profile's own
// `creation_phases` agrees: `rules/core/character_types.json` declares
// `virtues_flaws` before `abilities` before `arts` everywhere both appear).
// A `prereq_not_met` issue hard-coded to `CreationPhase::VirtuesFlaws`
// therefore blocks `wizard-navigation.svelte.ts`'s `canAdvance` on the
// CURRENT phase forever, since the fix lives on a phase the player cannot
// reach until Next unblocks — a dead end with no later phase ever able to
// clear it. The issue must be attributed to the phase whose input surface
// can actually satisfy it (a Supernatural Ability OR an Art — i.e. the
// LATER of the two, Arts, since either purchase clears the `Any`).
// ---------------------------------------------------------------------------

#[test]
fn broken_vessel_prereq_issue_is_reported_on_a_reachable_phase() {
    let rs = load_ruleset();
    let none = entity("magus", vec![sel("flaw.broken_vessel")]);
    let result = validate(&none, &rs);
    let issue = result
        .issues
        .iter()
        .find(|i| {
            i.code == "prereq_not_met" && i.context.as_ref() == Some(&Id::new("flaw.broken_vessel"))
        })
        .expect("flaw.broken_vessel must carry a prereq_not_met issue with no Ability/Art bought");
    assert_eq!(
        issue.phase,
        CreationPhase::Arts,
        "the issue must be attributed to the LATEST phase that can satisfy the Any \
         (Arts, since either a later Ability or a later Art purchase clears it), not \
         to VirtuesFlaws — the phase the player is stuck on with no way to reach \
         Abilities/Arts and fix it, got {:?}",
        issue.phase
    );
}

// ---------------------------------------------------------------------------
// Serde round-trips for the three new `Prereq` variants (D81.2/.3). These
// pass already — the `#[serde(tag = "kind", content = "value")]` derive
// needs no extra code for a new variant — but pin the wire shape now so the
// TS mirror (`ui/src/lib/types.ts`) has a known-good contract to match.
// ---------------------------------------------------------------------------

#[test]
fn characteristic_min_roundtrips() {
    let prereq = Prereq::CharacteristicMin {
        characteristic: Id::new("characteristic.pre"),
        score: 1,
    };
    let json = serde_json::to_string(&prereq).unwrap();
    let back: Prereq = serde_json::from_str(&json).unwrap();
    assert_eq!(prereq, back);
}

#[test]
fn ability_category_score_min_roundtrips() {
    let prereq = Prereq::AbilityCategoryScoreMin {
        category: "supernatural".to_string(),
        score: 1,
    };
    let json = serde_json::to_string(&prereq).unwrap();
    let back: Prereq = serde_json::from_str(&json).unwrap();
    assert_eq!(prereq, back);
}

#[test]
fn any_art_min_roundtrips() {
    let prereq = Prereq::AnyArtMin { score: 1 };
    let json = serde_json::to_string(&prereq).unwrap();
    let back: Prereq = serde_json::from_str(&json).unwrap();
    assert_eq!(prereq, back);
}
