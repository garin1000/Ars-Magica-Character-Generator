//! D76 coverage follow-up: `effective/spell.rs::spell_caps` — the public,
//! per-spell-in-the-catalogue cap fold the UI picker reads to grey a spell and
//! offer its "add within focus" action (D81.5) — had NO direct test at all;
//! only its sibling `spell_level_caps` (the Technique×Form GRID, not a
//! specific catalogue spell) was exercised. Every test here runs against the
//! real shipped ruleset.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::spell_caps;
use arm_rules::types::*;
use std::collections::BTreeMap;

/// The shipped core ruleset — same set of files `requisite_level_cap.rs::full_ruleset`
/// loads; duplicated because integration test binaries cannot share private helpers.
fn full_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        mythic_types: Some(include_str!(
            "../../../rules/core/mythic_companion_types.json"
        )),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: None,
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

fn entity() -> Entity {
    Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    )
}

fn art(id: &str, score: u8) -> ArtScore {
    ArtScore::new(Id::new(id), score)
}

/// `spell.the_crystal_dart`: Mu/Te, requisite art.rego (Technique-class).
/// Mu 5, Te 8, Re 20 (higher than Mu, so the lesser-of-fold leaves Mu's own 5
/// in place) -> cap = 5 + 8 + 0 (Int) + 0 (Magic Theory) + 3 = 16, same
/// hand-computation `requisite_level_cap.rs`'s
/// `requisite_higher_than_both_primary_arts_leaves_the_cap_unchanged` already
/// pins through `validate` — reused here to pin the SAME number through the
/// public `spell_caps` function directly, which no test called before.
const CRYSTAL_DART: &str = "spell.the_crystal_dart";

#[test]
fn spell_caps_covers_every_catalogue_spell_and_computes_the_known_ones_cap() {
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 5),
        art("art.terram", 8),
        art("art.rego", 20),
    ];

    let caps = spell_caps(&e, &ruleset);
    assert_eq!(
        caps.len(),
        ruleset.spells().count(),
        "spell_caps must return exactly one SpellCap per catalogue spell"
    );

    let crystal_dart = caps
        .iter()
        .find(|c| c.spell == Id::new(CRYSTAL_DART))
        .expect("the catalogue carries spell.the_crystal_dart");
    assert_eq!(
        crystal_dart.cap, 16,
        "spell_caps must fold the spell's own requisites into its cap the \
         same way validation's spell_level_exceeds_cap does"
    );
    assert_eq!(
        crystal_dart.within_focus_cap, None,
        "an entity holding no Magical Focus must get no within_focus_cap at all"
    );
}

#[test]
fn spell_caps_adds_a_within_focus_cap_only_once_a_magical_focus_is_held() {
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 5),
        art("art.terram", 8),
        art("art.rego", 20),
    ];
    e.selections = vec![Selection::with_params(
        Id::new("virtue.major_magical_focus"),
        BTreeMap::from([("focus".to_string(), Id::new("test"))]),
    )];

    let caps = spell_caps(&e, &ruleset);
    let crystal_dart = caps
        .iter()
        .find(|c| c.spell == Id::new(CRYSTAL_DART))
        .expect("the catalogue carries spell.the_crystal_dart");
    assert!(
        crystal_dart.within_focus_cap.is_some(),
        "an entity holding a Magical Focus must get a computed within_focus_cap \
         for every spell, not just None carried over from the unfocused case"
    );
    assert!(
        crystal_dart.within_focus_cap.unwrap() >= crystal_dart.cap,
        "the Magical Focus doubling can only ever raise the cap, never lower it"
    );
}
