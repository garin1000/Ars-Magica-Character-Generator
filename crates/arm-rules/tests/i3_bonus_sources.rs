//! I3 (try-out finding 15): the Arts' and Abilities' effective badge explains
//! itself. The engine already reports the summed effective-over-bought delta
//! (`effective/art.rs::art_bonuses`, `effective/ability.rs::ability_bonuses`);
//! this surfaces the same delta broken down per SOURCE item — which Virtue
//! contributed how much — so the UI tooltip can name each one, the way the
//! Characteristics tab already names its aging drops and Virtue deltas.
//!
//! The load-bearing invariant: for every listed Art or Ability instance the
//! source amounts sum to exactly today's delta, so the breakdown can never
//! disagree with the number on the badge.
//!
//! Rules exercised (verbatim):
//! - ArMDE:4818-4820 (Puissant Art): "You add 3 to the value of one Art whenever
//!   you use it."
//! - ArMDE:3731-3737 (Elemental Magic): "if you assign 10 experience points to
//!   each of Aquam, Auram, and Terram, and 21 experience points to Ignem, then you
//!   should assign 15 bonus experience points to Ignem (5 from each of Aquam,
//!   Auram, and Terram), and 21 bonus experience points to each of Aquam, Auram,
//!   and Terram (11 from Ignem, and 5 each from the other two Forms)."
//! - ArMDE:4814-4816 (Puissant Ability): "add 2 to its value whenever you use it."
//! - ArMDE:4888-4890 (Second Sight): "Choosing this Virtue confers the Ability
//!   Second Sight 1".

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::{
    ScoreSource, ability_bonus_sources, ability_bonuses, art_bonus_sources, art_bonuses,
    effective_ability_score,
};
use std::collections::BTreeMap;

fn shipped_ruleset() -> Ruleset {
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

fn character(type_id: &str, selections: Vec<Selection>) -> Entity {
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

fn sel_with(id: &str, key: &str, value: &str) -> Selection {
    Selection::with_params(
        Id::new(id),
        BTreeMap::from([(key.to_string(), Id::new(value))]),
    )
}

fn source(id: &str, amount: i32) -> ScoreSource {
    ScoreSource {
        source: Id::new(id),
        amount,
    }
}

/// Sources sorted by item id, so an assertion pins WHICH sources contribute
/// WHAT without also pinning their display order.
fn sorted(mut sources: Vec<ScoreSource>) -> Vec<ScoreSource> {
    sources.sort_by(|a, b| a.source.cmp(&b.source));
    sources
}

/// The elemental magus of the ArMDE:3735 worked example shape, at whole scores:
/// Aquam/Auram/Ignem 6 (21 table-XP each), Terram 4 (10 table-XP), with Puissant
/// (Ignem) on top.
///
/// Ignem: 21 own + ceil(21/2) + ceil(21/2) + ceil(10/2) = 21 + 11 + 11 + 5 = 48
/// XP → score 9 (45 ≤ 48 < 55), so Elemental Magic gives +3; Puissant Art +3.
/// Terram: 10 own + 3 × 11 = 43 XP → score 8 (36 ≤ 43 < 45), Elemental +4.
fn elemental_puissant_ignem_magus() -> Entity {
    let mut e = character(
        "magus",
        vec![
            sel_with("virtue.puissant_art", "art", "art.ignem"),
            sel("virtue.elemental_magic"),
        ],
    );
    e.art_scores = vec![
        ArtScore::new(Id::new("art.aquam"), 6),
        ArtScore::new(Id::new("art.auram"), 6),
        ArtScore::new(Id::new("art.ignem"), 6),
        ArtScore::new(Id::new("art.terram"), 4),
    ];
    e
}

fn art_sources_of(e: &Entity, rs: &Ruleset, art: &str) -> Option<Vec<ScoreSource>> {
    art_bonus_sources(e, rs)
        .into_iter()
        .find(|entry| entry.art == Id::new(art))
        .map(|entry| sorted(entry.sources))
}

#[test]
fn puissant_ignem_and_elemental_magic_are_listed_as_two_sources() {
    let rs = shipped_ruleset();
    let e = elemental_puissant_ignem_magus();
    assert_eq!(
        art_sources_of(&e, &rs, "art.ignem"),
        Some(sorted(vec![
            source("virtue.puissant_art", 3),
            source("virtue.elemental_magic", 3),
        ])),
        "Ignem's +6 must name Puissant Art (+3, ArMDE:4820) and Elemental Magic \
         (+3, ArMDE:3735) separately"
    );
    assert_eq!(
        art_sources_of(&e, &rs, "art.terram"),
        Some(vec![source("virtue.elemental_magic", 4)]),
        "Terram has no Puissant Art: its +4 is Elemental Magic alone"
    );
}

#[test]
fn art_sources_sum_to_the_art_bonuses_delta_and_cover_the_same_arts() {
    let rs = shipped_ruleset();
    let e = elemental_puissant_ignem_magus();
    let deltas = art_bonuses(&e, &rs);
    let breakdown = art_bonus_sources(&e, &rs);
    assert!(!deltas.is_empty(), "fixture must produce some Art bonus");
    assert_eq!(
        breakdown.iter().map(|b| &b.art).collect::<Vec<_>>(),
        deltas.iter().map(|d| &d.art).collect::<Vec<_>>(),
        "one breakdown per boosted Art, in art_bonuses order"
    );
    for (entry, delta) in breakdown.iter().zip(&deltas) {
        assert!(
            entry.sources.iter().all(|s| s.amount != 0),
            "{}: a zero contribution is not a source",
            entry.art
        );
        let sum: i32 = entry.sources.iter().map(|s| s.amount).sum();
        assert_eq!(
            sum, delta.bonus,
            "{}: sources must sum to the delta",
            entry.art
        );
    }
}

fn ability_sources_of(e: &Entity, rs: &Ruleset, ability: &str) -> Option<Vec<ScoreSource>> {
    ability_bonus_sources(e, rs)
        .into_iter()
        .find(|entry| entry.ability == Id::new(ability) && entry.parameter.is_none())
        .map(|entry| sorted(entry.sources))
}

fn second_sight_companion() -> Entity {
    character(
        "companion",
        vec![
            sel("virtue.second_sight"),
            sel_with("virtue.puissant_ability", "ability", "ability.second_sight"),
        ],
    )
}

/// Unbought Second Sight: the granted floor (1, ArMDE:4890) and Puissant Ability
/// (+2, ArMDE:4816) both raise the effective score above the bought 0, so both are
/// named, and together they make the badge's 3.
#[test]
fn granted_floor_and_puissant_ability_are_both_named_when_unbought() {
    let rs = shipped_ruleset();
    let e = second_sight_companion();
    assert_eq!(
        ability_sources_of(&e, &rs, "ability.second_sight"),
        Some(sorted(vec![
            source("virtue.second_sight", 1),
            source("virtue.puissant_ability", 2),
        ]))
    );
    assert_eq!(
        effective_ability_score(&e, &rs, &Id::new("ability.second_sight"), None),
        3
    );
}

/// Bought to 1, the grant floor no longer lifts anything (max(1, 1) = 1), so it
/// contributes nothing and is not listed; only Puissant's +2 explains the badge.
#[test]
fn a_floor_at_or_below_the_bought_score_is_not_a_source() {
    let rs = shipped_ruleset();
    let mut e = second_sight_companion();
    e.ability_scores = vec![AbilityScore::new(Id::new("ability.second_sight"), 1)];
    assert_eq!(
        ability_sources_of(&e, &rs, "ability.second_sight"),
        Some(vec![source("virtue.puissant_ability", 2)])
    );
}

/// A floor alone (Second Sight without Puissant) is not in `ability_bonuses`
/// (that list carries additive bonuses only), but it does move the badge, so it
/// must have a breakdown.
#[test]
fn a_floor_alone_is_listed_with_its_granting_virtue() {
    let rs = shipped_ruleset();
    let e = character("companion", vec![sel("virtue.second_sight")]);
    assert_eq!(
        ability_sources_of(&e, &rs, "ability.second_sight"),
        Some(vec![source("virtue.second_sight", 1)])
    );
}

/// The badge shows `effective_ability_score - bought`; every listed instance's
/// sources must sum to exactly that, and every `ability_bonuses` instance must be
/// listed.
#[test]
fn ability_sources_sum_to_the_effective_over_bought_delta() {
    let rs = shipped_ruleset();
    let mut e = second_sight_companion();
    e.selections.push(sel_with(
        "virtue.puissant_ability",
        "ability",
        "ability.awareness",
    ));
    e.ability_scores = vec![AbilityScore::new(Id::new("ability.awareness"), 3)];
    let breakdown = ability_bonus_sources(&e, &rs);
    assert!(!breakdown.is_empty(), "fixture must produce some breakdown");
    for entry in &breakdown {
        assert!(
            entry.sources.iter().all(|s| s.amount != 0),
            "{}: a zero contribution is not a source",
            entry.ability
        );
        let bought = e
            .ability_scores
            .iter()
            .filter(|a| a.ability == entry.ability)
            .map(|a| i32::from(a.score))
            .max()
            .unwrap_or(0);
        let delta =
            effective_ability_score(&e, &rs, &entry.ability, entry.parameter.as_deref()) - bought;
        let sum: i32 = entry.sources.iter().map(|s| s.amount).sum();
        assert_eq!(
            sum, delta,
            "{}: sources must sum to the delta",
            entry.ability
        );
    }
    for bonus in ability_bonuses(&e, &rs) {
        assert!(
            breakdown
                .iter()
                .any(|b| b.ability == bonus.ability && b.parameter == bonus.parameter),
            "{} carries a bonus but has no breakdown",
            bonus.ability
        );
    }
}
