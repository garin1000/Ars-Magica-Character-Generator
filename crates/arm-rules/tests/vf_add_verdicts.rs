//! Manual-test findings #2/#19 (`docs/manual-testing-findings-2026-10-18.md`):
//! the picker offers every catalogue Virtue/Flaw regardless of whether the
//! engine already knows, right now, that adding it is illegal — a House-gated
//! Flaw (Brutal Artist) is offered to everyone, and every Hermetic entry is
//! offered to a Gift-less companion even though the profile forbids the
//! `hermetic` category for one. `selections.rs::vf_add_verdicts` is the
//! engine's one verdict behind the picker's Add-button gate; these tests pin
//! its behavior directly, without going through a picked selection at all.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{
    IssueSeverity, ValidationIssue, VfAddBlockReason, forbidden_categories_in_force, validate,
    vf_add_verdicts,
};

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

/// Same shape as `x3_trained_gate.rs::entity` — duplicated since integration
/// test binaries cannot share private helpers.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

/// A companion with no selections at all — no Gift, no House.
fn bare_companion() -> Entity {
    entity("companion", vec![])
}

#[test]
fn companion_without_the_gift_is_blocked_from_a_hermetic_virtue_by_category() {
    let rs = load_ruleset();
    let e = bare_companion();
    let verdicts = vf_add_verdicts(&e, &rs);
    assert_eq!(
        verdicts.get(&Id::new("virtue.puissant_art")),
        Some(&VfAddBlockReason::ForbiddenCategory),
        "a Gift-less companion must be blocked from a Hermetic Virtue by category, \
         got {verdicts:?}"
    );
}

#[test]
fn the_gift_itself_is_never_blocked_for_a_gift_less_companion() {
    let rs = load_ruleset();
    let e = bare_companion();
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("virtue.the_gift")),
        "virtue.the_gift must never appear in the blocked map, even though its own \
         category (special) is not in the companion's permitted set — got {verdicts:?}"
    );
}

#[test]
fn magus_of_a_different_house_is_blocked_from_brutal_artist_by_prereq() {
    let rs = load_ruleset();
    let mut e = entity("magus", vec![]);
    e.house = Some(Id::new("house.tremere"));
    let verdicts = vf_add_verdicts(&e, &rs);
    assert_eq!(
        verdicts.get(&Id::new("flaw.brutal_artist")),
        Some(&VfAddBlockReason::PrereqNotMet),
        "a magus of House Tremere must be blocked from Brutal Artist by its own \
         House(house.jerbiton) prerequisite, got {verdicts:?}"
    );
}

#[test]
fn magus_of_house_jerbiton_is_not_blocked_from_brutal_artist() {
    let rs = load_ruleset();
    let mut e = entity("magus", vec![]);
    e.house = Some(Id::new("house.jerbiton"));
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("flaw.brutal_artist")),
        "a magus of House Jerbiton must not be blocked from Brutal Artist, got {verdicts:?}"
    );
}

/// `flaw.university_dean`'s prerequisite is
/// `all[has(virtue.doctor_in_faculty), age_min(40)]`. With
/// `virtue.doctor_in_faculty` held and no age set, `age_min` resolves
/// `Tri::Unknown` (unset age is Unknown, not False — the `Prereq::AgeMin` doc
/// comment), so the whole `all` resolves Unknown too (no child is a definite
/// False to short-circuit on) — and an Unknown outcome must never block.
///
/// This is the general rule for a NON-House Unknown. `Prereq::House`'s own
/// `None` arm is the one deliberate exception (F8/D93.2, Norbert
/// 2026-11-15): `vf_add_verdicts` flags its context so an unset House counts
/// as a definite mismatch at pick time — see
/// `magus_with_house_unset_is_blocked_from_brutal_artist` below.
#[test]
fn an_unevaluable_age_prerequisite_never_blocks() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.doctor_in_faculty"))],
    );
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("flaw.university_dean")),
        "an item whose prerequisite cannot yet be decided (age unset) must not be \
         blocked, got {verdicts:?}"
    );
}

/// F8/D93.2 (Norbert, 2026-11-15): "greyed out whenever the House is not
/// Jerbiton, INCLUDING House not set." Unlike the AgeMin case above, an unset
/// House at PICK TIME must now be treated as a definite mismatch, not an
/// Unknown that lets the item through.
#[test]
fn magus_with_house_unset_is_blocked_from_brutal_artist() {
    let rs = load_ruleset();
    let e = entity("magus", vec![]);
    assert!(e.house.is_none(), "the fixture must set no House");
    let verdicts = vf_add_verdicts(&e, &rs);
    assert_eq!(
        verdicts.get(&Id::new("flaw.brutal_artist")),
        Some(&VfAddBlockReason::PrereqNotMet),
        "F8/D93.2: a magus with no House set yet must be blocked from Brutal Artist \
         exactly like a magus of a confirmed different House, got {verdicts:?}"
    );
}

/// F8/D93.2's other half: the flag is set ONLY in `vf_add_verdicts`, never in
/// `validate`'s own context — so an ALREADY-HELD Brutal Artist with House
/// unset must keep today's Unknown-path behavior (a non-blocking
/// `prereq_unevaluated` warning, never `prereq_not_met`), unchanged by this
/// fix. Mirrors `data_integrity.rs::heartbeast_without_a_house_warns_rather_than_failing`,
/// pinned here for Brutal Artist specifically since it is this finding's item.
#[test]
fn held_brutal_artist_with_house_unset_still_warns_rather_than_blocks() {
    let rs = load_ruleset();
    let e = entity("magus", vec![Selection::new(Id::new("flaw.brutal_artist"))]);
    assert!(e.house.is_none(), "the fixture must set no House");

    let result = validate(&e, &rs);
    let codes: Vec<&str> = result
        .issues
        .iter()
        .filter(|i| i.context.as_ref() == Some(&Id::new("flaw.brutal_artist")))
        .map(|i| i.code.as_str())
        .collect();
    assert!(
        codes.contains(&"prereq_unevaluated"),
        "validating an already-held Brutal Artist with House unset must stay on the \
         Unknown path (unflagged context), got {codes:?}"
    );
    assert!(
        !codes.contains(&"prereq_not_met"),
        "an unset House on an already-held item must never become a hard error, got {codes:?}"
    );
}

#[test]
fn a_legal_item_for_the_profile_is_absent_from_the_map() {
    let rs = load_ruleset();
    let e = bare_companion();
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("virtue.bureaucrat")),
        "virtue.bureaucrat (social_status, no prerequisite) must be legal for a bare \
         companion, got {verdicts:?}"
    );
}

// --- E2/F7 (manual-test findings 2026-11-15 #7): a HARD magnitude/category
// cap must refuse the pick itself, not only the validator after the fact. A
// grog's `max_major_virtues: 0` (ArMDE:2898) lets Redcap (Major, Social
// Status, ArMDE:4915-4923) clear every category/prereq gate the add-verdict
// already checks, so only a cap-aware verdict catches it. ---

#[test]
fn grog_is_blocked_from_redcap_by_the_major_virtue_cap() {
    let rs = load_ruleset();
    let e = entity("grog", vec![]);
    let verdicts = vf_add_verdicts(&e, &rs);
    assert_eq!(
        verdicts.get(&Id::new("virtue.redcap")),
        Some(&VfAddBlockReason::ExceedsCap),
        "a grog (max_major_virtues: 0) must be blocked from the Major virtue \
         Redcap by the cap, got {verdicts:?}"
    );
}

#[test]
fn a_companion_is_not_blocked_from_redcap_by_any_cap() {
    let rs = load_ruleset();
    let e = entity("companion", vec![]);
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("virtue.redcap")),
        "a companion has no max_major_virtues cap, so Redcap must stay \
         addable, got {verdicts:?}"
    );
}

#[test]
fn a_soft_category_cap_at_its_max_stays_addable() {
    let rs = load_ruleset();
    // Companion's social_status virtue cap is `{ max: 1, hard: false }` (D41) —
    // a second Social Status item is only ever a warning, so it must never
    // appear in the add-blocked map, even though the count would already be
    // at its max.
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.bureaucrat"))],
    );
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("virtue.almogaten")),
        "a SOFT category cap must never block an add, even at its max, got {verdicts:?}"
    );
}

#[test]
fn a_hard_category_cap_at_its_max_blocks_the_next_item_of_that_category() {
    let rs = load_ruleset();
    // Companion's personality-flaw cap: `{ category: personality, max: 1,
    // major_only: true, hard: true }` — a second Major Personality Flaw must
    // be refused at pick time, the same cap `validate_caps` already enforces
    // after the fact.
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.ambitious_major"))],
    );
    let verdicts = vf_add_verdicts(&e, &rs);
    assert_eq!(
        verdicts.get(&Id::new("flaw.avaricious_major")),
        Some(&VfAddBlockReason::ExceedsCap),
        "a second Major Personality Flaw must be blocked by the hard \
         category cap, got {verdicts:?}"
    );
}

/// E2/F7 taken_as rule: a category cap that applies to only ONE of an item's
/// alternative `taken_as` readings (Sufi, Vengeful Powers: "either/or", not
/// both at once — `PointItem::categories_for`) must block the add only when
/// EVERY permitted reading would breach a hard cap. A small inline ruleset
/// (no shipped item has two readings each hitting its OWN hard cap, so a real
/// catalogue item cannot pin this half of the rule).
#[test]
fn a_taken_as_item_is_blocked_only_when_every_reading_breaches_a_hard_cap() {
    let point_items = r#"[
      {
        "id": "virtue.test_dual_category",
        "kind": "virtue",
        "magnitude": "minor",
        "categories": ["test_cat_a", "test_cat_b"],
        "classification": "narrative",
        "parameters": [
          { "key": "taken_as", "type": "ref", "domain": "category", "values": ["test_cat_a", "test_cat_b"] }
        ]
      },
      {
        "id": "flaw.test_personality_filler",
        "kind": "flaw",
        "magnitude": "minor",
        "categories": ["personality"],
        "classification": "narrative"
      }
    ]"#;
    let profiles = r#"[
      {
        "id": "both_capped",
        "creation_phases": ["virtues_flaws"],
        "budget": {
          "virtue_points": 10,
          "flaw_points": 10,
          "virtue_category_caps": [
            { "category": "test_cat_a", "max": 0, "hard": true },
            { "category": "test_cat_b", "max": 0, "hard": true }
          ]
        }
      },
      {
        "id": "one_capped",
        "creation_phases": ["virtues_flaws"],
        "budget": {
          "virtue_points": 10,
          "flaw_points": 10,
          "virtue_category_caps": [
            { "category": "test_cat_a", "max": 0, "hard": true }
          ]
        }
      }
    ]"#;
    let rs = Ruleset::from_json("t", "1", point_items, profiles).expect("synthetic ruleset loads");

    let blocked_everywhere = entity("both_capped", vec![]);
    let verdicts = vf_add_verdicts(&blocked_everywhere, &rs);
    assert_eq!(
        verdicts.get(&Id::new("virtue.test_dual_category")),
        Some(&VfAddBlockReason::ExceedsCap),
        "every taken_as reading breaches a hard cap (both at max 0), so the \
         item must be blocked, got {verdicts:?}"
    );

    let only_one_blocked = entity("one_capped", vec![]);
    let verdicts = vf_add_verdicts(&only_one_blocked, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("virtue.test_dual_category")),
        "the test_cat_b reading still fits (uncapped), so the item must stay \
         addable even though the test_cat_a reading would breach, got {verdicts:?}"
    );
}

/// A hard-cap "too many" issue: the three fixed budget caps
/// (`too_many_major_virtues`/`too_many_major_flaws`/`too_many_minor_flaws`)
/// plus every data-driven category cap's derived code
/// (`too_many_<category>_<virtues|flaws>`, or the `too_many_major_<category>_…`
/// variant — `validation/caps.rs::push_category_cap_issues`'s own naming
/// convention). A SOFT cap shares the identical code string but raises a
/// `Warning`, never an `Error`, so filtering on severity alone already keeps
/// soft ceilings out — this never needs to name a category.
fn is_hard_cap_error(issue: &ValidationIssue) -> bool {
    issue.severity == IssueSeverity::Error
        && issue.code.starts_with("too_many_")
        && (issue.code.ends_with("_virtues") || issue.code.ends_with("_flaws"))
}

/// Agreement sweep: for every shipped character type and every catalogue
/// Virtue/Flaw the add-verdict does NOT already refuse on a minimal entity of
/// that type, adding it must never make [`arm_rules::validation::validate`]
/// raise a hard-cap error — the verdict and the validator must agree on every
/// real catalogue entry, not only the Redcap case above. No catalogue totals
/// are asserted (CLAUDE.md: catalogue size is data).
#[test]
fn the_add_verdict_agrees_with_validate_on_every_hard_cap_for_every_type() {
    let rs = load_ruleset();
    for profile in rs.profiles() {
        let base = entity(profile.id.as_str(), vec![]);
        let verdicts = vf_add_verdicts(&base, &rs);
        for item in rs.items() {
            if !matches!(item.kind, ItemKind::Virtue | ItemKind::Flaw) {
                continue;
            }
            if verdicts.contains_key(&item.id) {
                continue; // already refused — nothing to check here
            }
            let mut candidate = base.clone();
            candidate.selections.push(Selection::new(item.id.clone()));
            let result = validate(&candidate, &rs);
            let hard_cap_issues: Vec<&ValidationIssue> =
                result.errors().filter(|i| is_hard_cap_error(i)).collect();
            assert!(
                hard_cap_issues.is_empty(),
                "{}: adding unblocked item {} raised a hard-cap error the \
                 verdict should have caught: {hard_cap_issues:?}",
                profile.id,
                item.id,
            );
        }
    }
}

// --- E5/F27 (manual-test findings 2026-11-15 #27): the profile's forbidden
// categories, resolved IN FORCE for this entity right now — the new DTO field
// `VirtueFlawTab.svelte::sourceGroups` reads so it can drop a forbidden
// heading for an item that keeps another, permitted category (Vengeful
// Powers: "story", "hermetic") without re-deriving the profile rule itself. ---

#[test]
fn companion_without_the_gift_has_hermetic_forbidden_in_force() {
    let rs = load_ruleset();
    let e = bare_companion();
    let forbidden = forbidden_categories_in_force(&e, &rs);
    assert!(
        forbidden.contains("hermetic"),
        "a Gift-less companion's forbidden-in-force set must contain \"hermetic\" \
         (ArMDE:2369 \"Companions who do not have The Gift may not choose Hermetic \
         Virtues or Flaws\", restated at :2910), got {forbidden:?}"
    );
}

#[test]
fn companion_with_the_gift_has_no_forbidden_categories_in_force() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.the_gift"))],
    );
    let forbidden = forbidden_categories_in_force(&e, &rs);
    assert!(
        !forbidden.contains("hermetic"),
        "a Gifted companion must not have \"hermetic\" forbidden-in-force, \
         got {forbidden:?}"
    );
}

#[test]
fn magus_has_no_forbidden_categories_in_force() {
    let rs = load_ruleset();
    let e = entity("magus", vec![]);
    let forbidden = forbidden_categories_in_force(&e, &rs);
    assert!(
        forbidden.is_empty(),
        "a magus profile declares no forbidden_categories at all, got {forbidden:?}"
    );
}
