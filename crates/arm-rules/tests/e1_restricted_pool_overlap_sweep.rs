//! E1 follow-up (Norbert, manual-testing-findings-2026-11-15 #28): a
//! real-shipped-ruleset sweep proving D93.1's tiered phase 1
//! (`effective/xp.rs::two_phase_max_flow`/`tiered_pool_order`/`pool_scope`)
//! resolves every genuinely overlapping PAIR of restricted-ability-xp pools
//! the hand-picked Well-Traveled/childhood tests in `effective.rs` and
//! `x1_authorization_family.rs` already pin for ONE pair: the pool covering
//! FEWER catalogue Abilities drains first, so neither pool is left with
//! unused capacity while a strictly broader pool funds a spend it also
//! covers.
//!
//! **Enumeration scope.** Every `Effect::RestrictedAbilityXp` in the shipped
//! `rules/core/virtues_flaws.json` that a selection ALONE materializes —
//! resolved against the live `Ruleset` exactly as `effective/xp.rs::pool_scope`
//! resolves a category (`abilities` ∪ categories expanded via
//! `Ruleset::abilities()`, the public accessor — this file is a black-box
//! integration test over `arm_rules`'s public surface, not an internal unit
//! test). Four things are deliberately OUT of this generic, selection-only
//! scope, each for a reason that would need its own per-item scaffolding
//! rather than one generic sweep:
//! - Any `RestrictedAbilityXp` with a non-empty `instances` list (Master
//!   Bard/Craft Guild Training/Educated-shaped instance scoping, D48) — the
//!   ENGINE's own `effective/xp.rs::pool_scope` folds `instances` into a
//!   pool's true covered-count, but matching an instance correctly needs the
//!   specific value a character actually bought, which this catalogue-level
//!   sweep has no entity to resolve against. Computing a covered-set that
//!   ignores `instances` would silently UNDER-count such a pool against a
//!   sibling and assert the WRONG winner — found exactly this way while
//!   writing this sweep (Master Bard vs Ineslemen on Faerie Lore) — so the
//!   whole pool is skipped, not just its `instances` contribution.
//! - `Effect::ScaledRestrictedAbilityXp` (Simple Student is the only shipped
//!   carrier) — its `amount` is zero until the granting selection's own
//!   `years` parameter is filled in, so it needs a dedicated entity, not a
//!   generic one.
//! - A `RestrictedAbilityXp` whose `amount` is `0` (Hedge Wizard: its real
//!   coverage lives entirely in the dynamic `abilities_param`, itself out of
//!   a static sweep) — zero capacity never competes with anything, so
//!   including it would only assert a vacuous "the 0-point pool stayed at
//!   0/0", not the invariant under test.
//! - `Effect::ReplacesLifeStageXp` (Feral Upbringing's childhood-collapsing
//!   replacement; Redcap/Lone Redcap's apprenticeship-shaped one) — these
//!   only turn into a `FlowPool` inside `effective/xp.rs::build_flow_pools`'s
//!   `life_stage_budget` branch, which needs `age`/`life_stages` scaffolding
//!   this generic, selections-only harness does not build (a bare selection
//!   of `flaw.feral_upbringing` with no life-stage plan materializes NO pool
//!   at all — confirmed by hand while writing this file, RED against the
//!   first draft of this sweep, which tried to fold it in generically and
//!   found the pool simply absent). Both are swept individually below
//!   instead, each with the scaffolding it actually needs.
//!
//! Each excluded shape IS still covered, individually, by its own existing
//! test elsewhere —
//! `x1_authorization_family.rs::well_traveled_carries_its_fifty_point_pool`
//! and friends pin the shipped shapes; `effective.rs`'s D43 tests pin the
//! funding-vs-ownership split — so this file adds the ORDERING invariant on
//! top, for the pools it CAN build generically plus two dedicated tests
//! below for the life-stage-shaped ones the generic harness cannot drive
//! without `age`/`life_stages` scaffolding.
//!
//! The Spell-Mastery pool is excluded with no test at all, because the
//! exclusion is TYPE-level, not empirical: `effective/xp.rs::pool_covers`
//! never lets a `PoolEligibility::Mastery` pool fund an Ability spend (or the
//! reverse), so no pair involving one can ever overlap in the first place.
//!
//! No catalogue totals are asserted anywhere in this file (CLAUDE.md).

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::{
    AbilityCategory, AbilityFunding, LifeStageBlock, LifeStagePlan, RestrictedXpPool, XpAllocation,
    XpPoolOrigin, checked_xp_allocation,
};
use std::collections::BTreeSet;

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

/// Same shipped-ruleset load every X1 test uses — duplicated rather than
/// shared, since integration test binaries cannot share private helpers
/// across files (see `x1_authorization_family.rs`'s own copy).
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

fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn score(ability: &str, value: u8) -> AbilityScore {
    AbilityScore::new(Id::new(ability), value)
}

/// One restricted-ability-xp-granting selection the shipped catalogue offers,
/// with the catalogue Abilities it covers (see the module doc for exactly
/// what is and is not folded in).
struct PoolSource {
    /// What to add to a minimal entity to grant it.
    grant: Id,
    covered: BTreeSet<Id>,
}

/// `categories`, expanded to the catalogue Abilities that currently carry
/// one of them — the same resolution `effective/xp.rs::pool_scope` performs
/// against the live `Ruleset`, recomputed here via the public
/// `Ruleset::abilities()` accessor.
fn expand_categories(rs: &Ruleset, categories: &[AbilityCategory]) -> BTreeSet<Id> {
    if categories.is_empty() {
        return BTreeSet::new();
    }
    rs.abilities()
        .filter(|a| categories.contains(&a.category))
        .map(|a| a.id.clone())
        .collect()
}

/// Every selection-granted restricted-ability-xp pool in `rs`, within this
/// file's enumeration scope (see the module doc).
fn pool_sources(rs: &Ruleset) -> Vec<PoolSource> {
    let mut sources = Vec::new();
    for item in rs.items() {
        for effect in &item.effects {
            match effect {
                Effect::RestrictedAbilityXp {
                    amount,
                    abilities,
                    categories,
                    instances,
                    ..
                    // A non-empty `instances` list (Master Bard, Craft Guild
                    // Training, Educated's own Latin carve-out, …) makes this
                    // pool's TRUE scope wider than `abilities`/`categories`
                    // alone say — the engine's own `pool_scope` folds
                    // `instances` in too. Leaving such a pool in this
                    // external, instances-blind covered-set would silently
                    // UNDER-count it against a sibling pool and assert the
                    // wrong winner — caught exactly this way while writing
                    // this sweep (Master Bard vs Ineslemen on Faerie Lore:
                    // Master Bard's declared `abilities` alone made it look
                    // narrower than Ineslemen, but Master Bard's own
                    // `instances` (2 Profession entries) make it the
                    // genuinely broader pool, which is what the engine
                    // correctly chose). So any such pool is skipped here
                    // entirely, not merely instances-blind.
                } if *amount > 0 && instances.is_empty() => {
                    let mut covered: BTreeSet<Id> = abilities.iter().cloned().collect();
                    covered.extend(expand_categories(rs, categories));
                    if !covered.is_empty() {
                        sources.push(PoolSource {
                            grant: item.id.clone(),
                            covered,
                        });
                    }
                }
                // Every other effect variant is either irrelevant to a
                // restricted-ability-xp pool, or named in the module doc's
                // "deliberately out of scope" list above.
                _ => {}
            }
        }
    }
    sources
}

/// Finds a restricted pool by a predicate over its origin — the generic form
/// both the sweep and the dedicated tests below use to pull a named pool out
/// of an allocation.
fn pool_where(
    allocation: &XpAllocation,
    mut matches: impl FnMut(&XpPoolOrigin) -> bool,
) -> &RestrictedXpPool {
    allocation
        .restricted
        .iter()
        .find(|p| matches(&p.origin))
        .unwrap_or_else(|| panic!("no matching restricted pool in {:?}", allocation.restricted))
}

fn item_pool<'a>(allocation: &'a XpAllocation, item: &str) -> &'a RestrictedXpPool {
    pool_where(
        allocation,
        |origin| matches!(origin, XpPoolOrigin::Item { item: granting } if *granting == Id::new(item)),
    )
}

/// The real-ruleset sweep itself. For every pair of distinct pool sources
/// whose coverage overlaps, builds a minimal entity holding BOTH grants (no
/// House, no budget, no prerequisite — `checked_xp_allocation` evaluates
/// effects only, exactly as `effective::selections_for_effects` does, so an
/// otherwise-"illegal" pairing is a legitimate allocation fixture) plus a
/// single score 1 (5 xp — comfortably below every shipped pool's smallest
/// amount, 25) in one overlapping Ability, and asserts the narrower pool
/// (fewer catalogue Abilities covered) pays it, leaving the broader pool at
/// its full, untouched amount.
#[test]
fn every_overlapping_pair_of_selection_granted_pools_drains_the_narrower_one_first() {
    let rs = load_ruleset();
    let sources = pool_sources(&rs);
    assert!(
        sources.len() >= 10,
        "expected several shipped restricted-pool sources within this sweep's scope, found {}",
        sources.len()
    );

    let mut pairs_checked = 0usize;
    for i in 0..sources.len() {
        for j in (i + 1)..sources.len() {
            let (narrower, broader) = if sources[i].covered.len() <= sources[j].covered.len() {
                (&sources[i], &sources[j])
            } else {
                (&sources[j], &sources[i])
            };
            // A tie in covered-count is a legitimate draw under D93.1's own
            // rule 3 (pool index breaks it) — not this sweep's business to
            // referee, since either order is correct depending on which one
            // `build_flow_pools` happens to place first. Only a STRICT
            // narrower-vs-broader pair is asserted.
            if narrower.covered.len() == broader.covered.len() {
                continue;
            }
            let Some(shared) = narrower.covered.intersection(&broader.covered).next() else {
                continue;
            };

            // BROADER selection listed FIRST, deliberately: `flow_pools`'
            // vector order follows `entity.selections`' order
            // (`effective/xp.rs::restricted_ability_xp_pools` iterates
            // `selections_for_effects` in that order), and pool-vector index
            // is D93.1's own LAST tie-break (rule 3) — so narrower-first
            // would still pass under the PRE-FIX single combined phase 1 by
            // accident of list position, proving nothing. Listing the
            // broader pool first is the adversarial order: only the fixed,
            // scope-driven tiering in `tiered_pool_order` makes the
            // genuinely narrower pool win despite sitting at the HIGHER
            // index.
            let mut e = entity(
                "companion",
                vec![
                    Selection::new(broader.grant.clone()),
                    Selection::new(narrower.grant.clone()),
                ],
            );
            // Generous enough for the largest shipped `from_normal_budget`
            // earmark to carve its full nominal amount out of (D13/D0 § 2):
            // an earmark pool's EFFECTIVE amount is capped at the entity's
            // own general budget, which direct-entry mode reads off
            // `xp_pool` (`effective/xp.rs::pre_earmark_general_xp`).
            e.xp_pool = 10_000;
            e.ability_scores = vec![score(shared.as_str(), 1)]; // 5 xp

            let allocation = checked_xp_allocation(&e, &rs).unwrap_or_else(|bound| {
                panic!("{} vs {}: {bound:?}", narrower.grant, broader.grant)
            });
            assert_eq!(
                allocation.max_flow, 5,
                "{} vs {shared}: must be fully funded",
                narrower.grant
            );

            let narrower_pool = item_pool(&allocation, narrower.grant.as_str());
            let broader_pool = item_pool(&allocation, broader.grant.as_str());
            assert_eq!(
                narrower_pool.used,
                5,
                "{} ({} abilities) must drain before {} ({} abilities) on shared ability {shared}: {:?}",
                narrower.grant,
                narrower.covered.len(),
                broader.grant,
                broader.covered.len(),
                allocation.restricted,
            );
            assert_eq!(
                broader_pool.used, 0,
                "{} must stay untouched while narrower {} has room: {:?}",
                broader.grant, narrower.grant, allocation.restricted,
            );
            pairs_checked += 1;
        }
    }

    assert!(
        pairs_checked > 0,
        "expected at least one genuinely overlapping pair in the shipped catalogue"
    );
}

/// Feral Upbringing's childhood-collapsing replacement (D40/D2, 9 covered
/// Abilities including Area Lore) and Well-Traveled's grant (8 covered
/// Abilities, `ArMDE:5314`) overlap on exactly one Ability — Area Lore. This
/// needs the life-stage scaffolding `every_overlapping_pair_of_…` above does
/// not build (the replacement only materializes inside
/// `effective/xp.rs::build_flow_pools`'s `life_stage_budget` branch), so it
/// is its own dedicated test rather than a generic-sweep candidate.
#[test]
fn feral_upbringings_childhood_replacement_defers_area_lore_to_the_narrower_well_traveled_grant() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![
            Selection::new(Id::new("flaw.feral_upbringing")),
            Selection::new(Id::new("virtue.well_traveled")),
        ],
    );
    e.age = Some(30);
    e.ability_funding = AbilityFunding::LifeStages;
    // No native language: Feral Upbringing replaces BOTH ordinary childhood
    // pools, so there is no native-language block to set one for.
    e.life_stages = Some(LifeStagePlan::default());
    e.ability_scores = vec![score("ability.area_lore", 2)]; // 15 xp

    let allocation = checked_xp_allocation(&e, &rs).expect("within the solve bound");
    assert_eq!(allocation.max_flow, 15, "fully funded");

    let well_traveled = item_pool(&allocation, "virtue.well_traveled");
    assert_eq!(
        well_traveled.used, 15,
        "Well-Traveled (8 abilities) is narrower than Feral Upbringing's replacement \
         (9 abilities) and must drain first: {:?}",
        allocation.restricted
    );
    let feral = pool_where(
        &allocation,
        |origin| matches!(origin, XpPoolOrigin::LifeStage { block } if *block == LifeStageBlock::ChildhoodSpread),
    );
    assert_eq!(
        feral.used, 0,
        "the broader replacement must stay untouched while Well-Traveled has room: {:?}",
        allocation.restricted
    );
}

/// Lone Redcap grants itself TWO overlapping pools at once: its own
/// apprenticeship-shaped replacement (`ArMDE:4392-4398`, covering every
/// gated AND ungated category — academic, arcane, general, martial,
/// supernatural — the broadest possible scope) and, via `grants_selection`,
/// Well-Traveled's narrower 8-ability grant. One selection is enough to hold
/// both, which is a tidier fixture than Feral Upbringing's pair above, but it
/// still needs the same life-stage scaffolding for the replacement to
/// materialize.
#[test]
fn lone_redcaps_own_apprenticeship_replacement_defers_to_its_narrower_granted_well_traveled() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.lone_redcap"))],
    );
    e.age = Some(30);
    e.ability_funding = AbilityFunding::LifeStages;
    e.life_stages = Some(LifeStagePlan::default());
    e.ability_scores = vec![score("ability.charm", 2)]; // 15 xp; Charm is General, so the
    // apprenticeship replacement's "general" category covers it too.

    let allocation = checked_xp_allocation(&e, &rs).expect("within the solve bound");
    assert_eq!(allocation.max_flow, 15, "fully funded");

    let well_traveled = item_pool(&allocation, "virtue.well_traveled");
    assert_eq!(
        well_traveled.used, 15,
        "the granted Well-Traveled pool (8 abilities) is narrower than Lone Redcap's \
         own every-category replacement and must drain first: {:?}",
        allocation.restricted
    );
    let apprenticeship = pool_where(
        &allocation,
        |origin| matches!(origin, XpPoolOrigin::LifeStage { block } if *block == LifeStageBlock::Apprenticeship),
    );
    assert_eq!(
        apprenticeship.used, 0,
        "Lone Redcap's own, far broader replacement must stay untouched: {:?}",
        allocation.restricted
    );
}
