//! D81.4/D81.14 (`docs/vf-audit/decisions.md`; `tmp/merinita-handover.md`) —
//! tests for Merinita's conditional Warping Point: ArMDE:2280, "Any magus in
//! this House without a faerie-related Virtue or Flaw has a Warping Point,
//! inflicted to allow initiation into the Mystery."
//!
//! Runs against the REAL shipped ruleset (`rules/core/houses.json`,
//! `rules/core/virtues_flaws.json`).
//!
//! D81.14 rules "faerie-related" as: an entry whose realm resolves to Faerie
//! (Faerie Blood, Strong Faerie Blood, and Bound to / Realm Stigmatic /
//! Necessary Aura / Folk Magic when their own `realm` param is Faerie), PLUS
//! an explicit `faerie_related: true` data flag on three entries outside the
//! realm system entirely (Faerie Friend, Faerie Upbringing, Susceptibility to
//! Faerie Power). The House's own Faerie Magic grant does not count.
//!
//! The first four tests below (phase 1) used only `virtue.faerie_blood`,
//! which is faerie-related under every candidate predicate the handover
//! surveyed, so they needed no change once D81.14 landed. The tests after
//! them are phase 2's red-first additions for the ruling's remaining cases:
//! the three flagged entries, plus Bound to (Realm) at Faerie (exempts) and
//! at Magic (a forward guard — must NOT exempt).
//!
//! `effective::warping`'s `merinita_warping_points` was a deliberate phase-1
//! stub: the House-gated, additive wiring was real from phase 1 (so it is not
//! touched here), but `has_faerie_related_vf` returned `false` unconditionally
//! until phase 2 implemented the predicate above. Each test says, in its own
//! comment, whether it was RED before phase 2's implementation or already
//! GREEN because it only pins wiring that was already correct — same shape as
//! `d42_realms.rs`'s phase-1 note.

use std::collections::BTreeMap;

use arm_rules::effective::warping;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;

fn full_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("shipped core ruleset loads")
}

/// Same shape as `d42_realms.rs::entity` / `x7be_row42.rs::entity`, duplicated
/// here since integration test binaries cannot share private helpers.
fn magus(house: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.house = Some(Id::new(house));
    e.selections = selections;
    e
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn faerie_blood() -> Selection {
    Selection::with_params(
        Id::new("virtue.faerie_blood"),
        BTreeMap::from([("heritage".to_string(), Id::new("heritage.sidhe"))]),
    )
}

fn bound_to_realm(realm: &str) -> Selection {
    Selection::with_params(
        Id::new("flaw.bound_to_realm"),
        BTreeMap::from([("realm".to_string(), Id::new(realm))]),
    )
}

/// ArMDE:2280's first clause fires: a Merinita magus with no faerie-related
/// Virtue or Flaw owes the Mystery's initiation Warping Point.
///
/// Already GREEN under the phase-1 stub: with no faerie-related V/F present,
/// the stub's constant "lacks one" is the correct answer for this case — it
/// only disagrees once a faerie-related V/F is actually held (see the next
/// test).
#[test]
fn merinita_without_faerie_related_vf_gets_one_warping_point() {
    let rs = full_ruleset();
    let entity = magus("house.merinita", vec![]);

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 1,
        "a Merinita magus with no faerie-related V/F must owe 1 Warping Point \
         from ArMDE:2280's initiation clause, got {outcome:?}"
    );
}

/// ArMDE:2280's exception: a faerie-related Virtue (here, Faerie Blood) means
/// the magus does not owe the initiation Warping Point.
///
/// RED under the phase-1 stub: `has_faerie_related_vf` always returns
/// `false`, so the stub still charges the point even though Faerie Blood is
/// held — this is the one test that actually exercises the still-unbuilt
/// predicate.
#[test]
fn merinita_with_a_faerie_related_virtue_gets_none_from_this_source() {
    let rs = full_ruleset();
    let entity = magus("house.merinita", vec![faerie_blood()]);

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 0,
        "a Merinita magus holding a faerie-related Virtue must owe NO Warping \
         Point from ArMDE:2280's initiation clause, got {outcome:?}"
    );
}

/// The clause is Merinita-only: a non-Merinita House magus with the same
/// (empty) V/F list owes nothing from it.
///
/// Already GREEN: the House gate itself is real wiring, not part of the
/// stubbed predicate.
#[test]
fn non_merinita_magus_gets_no_warping_point_from_the_clause() {
    let rs = full_ruleset();
    let entity = magus("house.flambeau", vec![]);

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 0,
        "a non-Merinita magus must owe nothing from Merinita's own House \
         clause, got {outcome:?}"
    );
}

/// The conditional point must ADD to any pre-existing accrued Warping Points,
/// not replace or drop them, and must be counted exactly once.
///
/// Already GREEN: the additive `saturating_add` wiring into
/// `warping_points_total` is real; this test will still catch a Phase 2
/// regression that routes the predicate through a path which drops or
/// double-counts the point instead of adding it once.
#[test]
fn merinita_initiation_point_stacks_additively_with_other_warping() {
    let rs = full_ruleset();
    let mut entity = magus("house.merinita", vec![]);
    entity.warping_points = 2;

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 3,
        "the initiation point must add to pre-existing accrued Warping \
         Points (2 stored + 1 from ArMDE:2280), got {outcome:?}"
    );
}

/// The conditional House point must also stack with a GRANT-EFFECT Warping
/// source (`flaw.warped_by_magic`'s `Effect::WarpingGrant { points: 5 }`),
/// not just the flat stored-points field the previous test already covers —
/// review-final.json finding #2 (MINOR): `warping_points_total` sums the
/// House-conditional term and `warping_grant_points_in` via two independent
/// `saturating_add` calls, but no existing test combined both sources on one
/// entity. Warped by Magic's own `realm_association` is `magic`, not
/// `faerie`, so it does not exempt itself from the Merinita clause.
///
/// Already GREEN: both terms are independent additive folds by construction
/// (`warping.rs`'s `warping_points_total`), so this pins the correct result
/// rather than fixing a regression.
#[test]
fn merinita_initiation_point_stacks_additively_with_a_warping_grant_flaw() {
    let rs = full_ruleset();
    let entity = magus("house.merinita", vec![sel("flaw.warped_by_magic")]);

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 6,
        "the initiation point (1, ArMDE:2280 — no faerie-related V/F held) \
         must add to Warped by Magic's own grant (5, Effect::WarpingGrant), \
         neither suppressing the other, got {outcome:?}"
    );
}

// --- D81.14: the three explicitly flagged entries --------------------------

/// D81.14: Faerie Friend (ArMDE:6052) is flagged `faerie_related: true` even
/// though it carries no realm association at all (`categories: ["story"]`).
///
/// RED before phase 2: the predicate ignored the flag entirely.
#[test]
fn merinita_with_faerie_friend_gets_none_from_this_source() {
    let rs = full_ruleset();
    let entity = magus("house.merinita", vec![sel("flaw.faerie_friend")]);

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 0,
        "Faerie Friend is flagged faerie-related (D81.14) and must exempt \
         the magus from ArMDE:2280's initiation point, got {outcome:?}"
    );
}

/// D81.14: Faerie Upbringing (ArMDE:6056) is flagged `faerie_related: true`
/// (`categories: ["personality"]`, no realm association).
///
/// RED before phase 2.
#[test]
fn merinita_with_faerie_upbringing_gets_none_from_this_source() {
    let rs = full_ruleset();
    let entity = magus("house.merinita", vec![sel("flaw.faerie_upbringing")]);

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 0,
        "Faerie Upbringing is flagged faerie-related (D81.14) and must \
         exempt the magus from ArMDE:2280's initiation point, got {outcome:?}"
    );
}

/// D81.14: Susceptibility to Faerie Power (ArMDE:6819) is flagged
/// `faerie_related: true` (`categories: ["hermetic"]`, no realm association —
/// it is a Hermetic Flaw with a `magic_resistance_mod` effect, not a
/// Supernatural entry).
///
/// RED before phase 2.
#[test]
fn merinita_with_susceptibility_to_faerie_power_gets_none_from_this_source() {
    let rs = full_ruleset();
    let entity = magus(
        "house.merinita",
        vec![sel("flaw.susceptibility_to_faerie_power")],
    );

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 0,
        "Susceptibility to Faerie Power is flagged faerie-related (D81.14) \
         and must exempt the magus from ArMDE:2280's initiation point, got \
         {outcome:?}"
    );
}

// --- D81.14: a realm-param entry, both ways ---------------------------------

/// D81.14: Bound to (Realm) with `realm` = Faerie resolves to the Faerie
/// realm via the existing `resolve_realm`/`item_has_realm_association`
/// machinery, so it counts as faerie-related.
///
/// RED before phase 2: the predicate did not read any entry's realm.
#[test]
fn merinita_bound_to_faerie_realm_gets_none_from_this_source() {
    let rs = full_ruleset();
    let entity = magus("house.merinita", vec![bound_to_realm("realm.faerie")]);

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 0,
        "Bound to (Realm) at Faerie resolves to the Faerie realm (D81.14) \
         and must exempt the magus from ArMDE:2280's initiation point, got \
         {outcome:?}"
    );
}

/// D81.14's other half: Bound to (Realm) with `realm` = Magic resolves to the
/// Magic realm, NOT Faerie — it must NOT exempt the magus.
///
/// Already GREEN both before and after phase 2 (a forward guard): the magus
/// still lacks a faerie-related V/F either way, so this pins that the
/// predicate does not over-match every realm-param entry regardless of its
/// chosen realm.
#[test]
fn merinita_bound_to_magic_realm_still_gets_one_warping_point() {
    let rs = full_ruleset();
    let entity = magus("house.merinita", vec![bound_to_realm("realm.magic")]);

    let outcome = warping(&entity, &rs);

    assert_eq!(
        outcome.points, 1,
        "Bound to (Realm) at Magic is not faerie-related and must NOT \
         exempt the magus from ArMDE:2280's initiation point, got {outcome:?}"
    );
}
