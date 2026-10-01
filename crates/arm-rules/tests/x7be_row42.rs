//! X7b-e (`docs/vf-audit/phase-2-plan.md` row X7b-e) — behavioural RED tests for
//! the **compute**-verdict entries of `docs/open-todos.md` row 42: the four
//! originally-swept Flaws plus the two of the 2026-09-19 nineteen the D58 scope
//! test (`docs/vf-audit/decisions.md`) rules in-scope for computation.
//!
//! Phase 1 only: every test here is expected to be RED until Phase 2 adds the
//! engine capability and/or the data wiring. See `tmp/x7be-verdicts.md` for the
//! full per-entry citation, D58 reasoning, and the three open questions this
//! slice does NOT decide.
//!
//! Every test runs against the **shipped** `rules/core/virtues_flaws.json`
//! (unmodified by this slice) through the existing `validate`/`effective`
//! entry points — no new engine type, effect variant, or Prereq variant is
//! added anywhere. Each test fails today because the shipped data carries no
//! wiring for the clause under test and/or no validator reads the precondition
//! yet, not because of a missing stub.

use arm_rules::characteristics::Characteristic;
use arm_rules::derived::surfaced_modifiers;
use arm_rules::effective::{
    decrepitude_score, entity_grants, reputation_grants, warping_points_total,
};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationIssue, ValidationResult, validate};
use std::collections::BTreeMap;

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

/// Same shape as `x1_authorization_family.rs::entity` / `data_integrity.rs::entity`,
/// duplicated here since integration test binaries cannot share private helpers.
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

fn issue_codes(result: &ValidationResult) -> Vec<&str> {
    result.issues.iter().map(|i| i.code.as_str()).collect()
}

/// row 42(b): "His Presence and Communication may not be greater than 0"
/// (ArMDE:6919-6922). `effective::characteristic_cap` is ruleset-global (base
/// ±3) and does not read the entity's selections at all, so a companion who
/// buys Presence 1 alongside this Flaw is accepted as legal today. This is the
/// worked example the brief names directly.
#[test]
fn uninspirational_caps_presence_and_communication_to_zero() {
    let rs = load_ruleset();
    let mut companion = entity("companion", vec![sel("flaw.uninspirational")]);
    companion.characteristics.insert(Characteristic::Pre, 1);

    let result = validate(&companion, &rs);

    assert!(
        issue_codes(&result).contains(&ValidationIssue::CODE_CHARACTERISTIC_ABOVE_CAP),
        "Uninspirational must cap Presence at 0, but Presence 1 passed validation cleanly: {:?}",
        result.issues
    );
}

/// row 42(a): "all Personality Traits must be between +1 and -1" (ArMDE:7076-7079).
/// `validate_personality_traits` only rejects a magnitude over 3 (or over 6 for
/// one Major-Personality-Flaw trait) — it has no per-Flaw tightened range, so a
/// Weak Personality character with a trait at +2 is legal today. The brief's
/// other named worked example.
#[test]
fn weak_personality_forbids_a_trait_at_plus_two() {
    let rs = load_ruleset();
    let mut companion = entity("companion", vec![sel("flaw.weak_personality")]);
    companion.personality_traits.push(PersonalityTrait {
        name: "Brave".into(),
        value: 2,
    });

    let result = validate(&companion, &rs);

    assert!(
        issue_codes(&result).contains(&ValidationIssue::CODE_PERSONALITY_TRAIT_OUT_OF_RANGE),
        "Weak Personality must refuse a +2 trait (only +1..-1 is legal), but it passed cleanly: {:?}",
        result.issues
    );
}

/// row 42(c): "Select a Personality Trait at +4, and its opposite at +4"
/// (ArMDE:6122-6124). Nothing validates that a Fickle Nature character
/// actually holds any such pair; a character with no personality traits at all
/// is accepted today. `fickle_nature_trait_pair_missing` is a PROPOSED code —
/// no constant exists for it yet, deliberately: Phase 2 is free to name it
/// differently as long as it fires on this precondition and this test's
/// literal is updated to match. A specific code (rather than "any issue at
/// all") is asserted so this red is not satisfied by an unrelated finding,
/// such as the companion profile's own minimum-Social-Status-Virtue
/// requirement, which this fixture also fails to meet. The "opposite" pairing
/// itself is not checked (see `tmp/x7be-verdicts.md` — it is unverifiable free
/// text), only that at least two distinct traits sit at exactly +4.
#[test]
fn fickle_nature_requires_a_matched_pair_of_traits_at_plus_four() {
    let rs = load_ruleset();
    let companion = entity("companion", vec![sel("flaw.fickle_nature")]);
    // No personality_traits recorded at all — RAW requires two, at +4 each.

    let result = validate(&companion, &rs);

    assert!(
        issue_codes(&result).contains(&"fickle_nature_trait_pair_missing"),
        "Fickle Nature with no +4 trait pair must be flagged, but validation raised: {:?}",
        result.issues
    );
}

/// row 42(d): "multiply whatever the penalty is by 1 + (Decrepitude Score)"
/// (ArMDE:6350-6352). `flaw.lingering_injury` carries zero `effects` in the
/// shipped data, so nothing sourced from it ever appears in the surfaced-modifier
/// list, however high the character's Decrepitude climbs. Existence is asserted
/// rather than an exact multiplied value, since no `ModifierFamily` variant for
/// this shape exists yet for the assertion to name (`tmp/x7be-verdicts.md`).
#[test]
fn lingering_injury_penalty_is_surfaced_and_scales_with_decrepitude() {
    let rs = load_ruleset();
    let mut companion = entity("companion", vec![sel("flaw.lingering_injury")]);
    companion.aging_points.insert(Characteristic::Str, 10);
    assert!(
        decrepitude_score(&companion, &rs) > 0,
        "test setup sanity: this entity must have a nonzero Decrepitude Score"
    );

    let modifiers = surfaced_modifiers(&companion, &rs);

    assert!(
        modifiers
            .iter()
            .any(|m| m.source.as_ref() == Some(&Id::new("flaw.lingering_injury"))),
        "Lingering Injury must surface a Decrepitude-scaled roll penalty, but nothing is sourced from it: {:?}",
        modifiers
    );
}

/// row 42/nineteen, `flaw.servant_of_the_land` (ArMDE:6717-6720): "the character
/// has the Minor Personality Flaw: Prohibition, but this does not count toward
/// the character's total number of Virtues and Flaws". `Effect::GrantsSelection`
/// already exists (Templar Commander) and a granted item is already free of the
/// point budget by construction — this needs NO new engine capability, only a
/// future data change wiring `flaw.servant_of_the_land`'s (currently empty)
/// `effects` to `GrantsSelection { items: ["flaw.prohibition"] }`. Today the
/// shipped entry grants nothing, so the Flaw never appears.
#[test]
fn servant_of_the_land_grants_prohibition_for_free() {
    let rs = load_ruleset();
    let companion = entity("companion", vec![sel("flaw.servant_of_the_land")]);

    let grants = entity_grants(&companion, &rs);

    assert!(
        grants
            .iter()
            .any(|s| s.item_ref == Id::new("flaw.prohibition")),
        "Servant of the (Land) must grant flaw.prohibition for free, but nothing is granted: {:?}",
        grants
    );
}

/// row 42/nineteen, `flaw.university_dean` (ArMDE:6923-6926): "must ... be at
/// least 40 years old". No `Prereq` variant can express a minimum age today, and
/// the shipped entry carries no `prerequisites` at all, so a 30-year-old
/// University Dean is accepted. (The Doctor-in-Faculty prerequisite and the
/// Poor/Bad-Reputation exclusion are data-only against already-built machinery —
/// see `tmp/x7be-verdicts.md` — so this test targets only the genuinely new
/// engine gap, the age floor.)
#[test]
fn university_dean_requires_age_forty() {
    let rs = load_ruleset();
    let mut companion = entity("companion", vec![sel("flaw.university_dean")]);
    companion.age = Some(30);

    let result = validate(&companion, &rs);

    assert!(
        issue_codes(&result).contains(&ValidationIssue::CODE_PREREQ_NOT_MET),
        "University Dean at age 30 must fail the 40-year minimum, but validation raised nothing: {:?}",
        result.issues
    );
}

/// The True-arm counterpart (QA review, coverage): a companion who meets
/// BOTH of University Dean's prerequisites — holds `virtue.doctor_in_faculty`
/// (itself requiring Artes Liberales 5 and Dead Language 5, ArMDE:4321-4322)
/// and is exactly 40 (the boundary, not merely above it) — must raise no
/// `prereq_not_met` at all. Without this, the only existing test
/// (`university_dean_requires_age_forty`, a companion with NEITHER
/// prerequisite met) short-circuits `Prereq::All` on the first child
/// (`Has(virtue.doctor_in_faculty)`), so `Prereq::AgeMin`'s own True arm in
/// `prereq.rs::evaluate_prereq` is never actually reached by any test in the
/// suite.
#[test]
fn university_dean_at_exactly_forty_with_doctor_in_faculty_raises_no_prereq_issue() {
    let rs = load_ruleset();
    let mut companion = entity(
        "companion",
        vec![
            sel("flaw.university_dean"),
            Selection::with_params(
                Id::new("virtue.doctor_in_faculty"),
                BTreeMap::from([("faculty".into(), Id::new("Canon Law"))]),
            ),
        ],
    );
    companion.age = Some(40);
    companion.ability_scores = vec![AbilityScore::new(Id::new("ability.artes_liberales"), 5), {
        let mut a = AbilityScore::new(Id::new("ability.dead_language"), 5);
        a.parameter = Some(AbilityParameterValue::Catalogued {
            id: Id::new("language.latin"),
        });
        a
    }];

    let result = validate(&companion, &rs);

    assert!(
        !issue_codes(&result).contains(&ValidationIssue::CODE_PREREQ_NOT_MET),
        "University Dean at exactly age 40 with Doctor in Faculty must raise no prereq-not-met issue: {:?}",
        result.issues
    );
}

/// row 42/nineteen, `flaw.raised_from_the_dead` (ArMDE:6646-6649), D69.1: "You
/// begin with at least three Warping points, plus one Warping point for every
/// year that has passed since you were resurrected... You also have a level 4
/// reputation in the area where the miracle occurred." The creation-time
/// grant is computed from a `years_since_resurrection` number parameter
/// (D64's pattern, the Abandoned Apprentice's `years_completed` being the
/// precedent); the ongoing "+1 Warping point every year you continue living"
/// clause is NOT creation-time and stays text (D69.1). Today the shipped
/// entry carries no effects and no parameter at all, so neither grant is
/// computed at any value.
#[test]
fn raised_from_the_dead_grants_warping_scaled_by_years_since_resurrection_and_a_level_four_reputation()
 {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![Selection::with_params(
            Id::new("flaw.raised_from_the_dead"),
            BTreeMap::from([("years_since_resurrection".into(), Id::new("5"))]),
        )],
    );

    assert_eq!(
        warping_points_total(&e, &rs),
        8,
        "ArMDE:6648 grants at least 3 Warping points plus 1 per year since \
         resurrection; with 5 years since resurrection the total must be 8, \
         got {}",
        warping_points_total(&e, &rs)
    );

    let reps = reputation_grants(&e, &rs);
    assert!(
        reps.iter()
            .any(|r| r.source == Id::new("flaw.raised_from_the_dead") && r.score == 4),
        "ArMDE:6648 grants a level 4 Reputation in the area the miracle \
         occurred; reputation_grants() returned: {:?}",
        reps
    );
}

/// row 42/nineteen, `flaw.flawed_powers` (ArMDE:6146-6149): "The character must
/// have at least one Major Supernatural Virtue to take this Flaw." No `Prereq`
/// variant can express "holds a category at a given magnitude" today
/// (`Prereq::HasCategory` ignores magnitude entirely), and the shipped entry
/// carries no `prerequisites`, so a character with zero Supernatural Virtues at
/// all is accepted. (The Hermetic-only-Flaw exclusion half is D68 item 4's
/// `requires_hermetic_arts` predicate, coordinated with X4/X10 rather than
/// retested here — see `tmp/x7be-verdicts.md`.)
#[test]
fn flawed_powers_requires_a_major_supernatural_virtue() {
    let rs = load_ruleset();
    let companion = entity("companion", vec![sel("flaw.flawed_powers")]);
    // No other selections: zero Supernatural Virtues of any magnitude.

    let result = validate(&companion, &rs);

    assert!(
        issue_codes(&result).contains(&ValidationIssue::CODE_PREREQ_NOT_MET),
        "Flawed Powers with no Major Supernatural Virtue must be refused, but validation raised nothing: {:?}",
        result.issues
    );
}

/// The True-arm counterpart (QA review, coverage): a companion who DOES hold
/// a Major Supernatural Virtue (`virtue.amorphous_major` — magnitude major,
/// category `supernatural`, ArMDE:3410-3413) alongside Flawed Powers must
/// raise no `prereq_not_met` at all. Without this, no test in the suite ever
/// reaches `Prereq::HasCategoryAtMagnitude`'s True arm in
/// `prereq.rs::evaluate_prereq`.
#[test]
fn flawed_powers_is_satisfied_by_a_major_supernatural_virtue() {
    let rs = load_ruleset();
    let companion = entity(
        "companion",
        vec![sel("flaw.flawed_powers"), sel("virtue.amorphous_major")],
    );

    let result = validate(&companion, &rs);

    assert!(
        !issue_codes(&result).contains(&ValidationIssue::CODE_PREREQ_NOT_MET),
        "Flawed Powers with a Major Supernatural Virtue held must raise no prereq-not-met issue: {:?}",
        result.issues
    );
}

/// The "held but below threshold" arm (QA review, coverage): a companion
/// holding only `virtue.amorphous_minor` (magnitude minor, category
/// `supernatural`, ArMDE:3410-3413) — a Supernatural Virtue, just not a MAJOR
/// one — must still be refused Flawed Powers, distinguishing this from "holds
/// nothing at all" (`flawed_powers_requires_a_major_supernatural_virtue`
/// above). Without this, `Prereq::HasCategoryAtMagnitude`'s `>=` comparison's
/// false-but-present arm (`prereq.rs::evaluate_prereq`) has no regression
/// coverage distinct from the "nothing held" case.
#[test]
fn flawed_powers_is_not_satisfied_by_a_minor_supernatural_virtue() {
    let rs = load_ruleset();
    let companion = entity(
        "companion",
        vec![sel("flaw.flawed_powers"), sel("virtue.amorphous_minor")],
    );

    let result = validate(&companion, &rs);

    assert!(
        issue_codes(&result).contains(&ValidationIssue::CODE_PREREQ_NOT_MET),
        "Flawed Powers with only a Minor Supernatural Virtue held must still be refused: {:?}",
        result.issues
    );
}
