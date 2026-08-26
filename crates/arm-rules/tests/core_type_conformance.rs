//! Regression guard for the M5 mandate: a complete, core-rules-conforming
//! character of every core type (grog, companion, mythic companion, magus) must
//! be enterable via direct input and pass validation with **zero errors**.
//!
//! Each character is built against the real shipped `rules/core/*.json` and run
//! through the engine's top-level [`validate`]. These assert the *structural*
//! invariant "a fully-built legal character validates clean" — never a catalogue
//! total — so pulling in more Abilities/Virtues/spells leaves them green.

use arm_rules::characteristics::Characteristic;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use std::collections::BTreeMap;

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
        aging: None,
    })
    .expect("shipped core ruleset loads")
}

fn base(type_id: &str) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.age = Some(25);
    e
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn ability(id: &str, score: u8) -> AbilityScore {
    AbilityScore {
        ability: Id::new(id),
        score,
        specialty: Some("focus".into()),
        parameter: None,
    }
}

/// Asserts the entity validates with no error-severity issues, printing every
/// error (code + args) on failure so a regression names exactly what broke.
fn assert_valid(label: &str, entity: &Entity, ruleset: &Ruleset) {
    let result = validate(entity, ruleset);
    let errors: Vec<String> = result
        .errors()
        .map(|i| format!("[{}] {:?}", i.code, i.args))
        .collect();
    assert!(
        errors.is_empty(),
        "{label} should validate with zero errors, got {}:\n  {}",
        errors.len(),
        errors.join("\n  ")
    );
}

/// The shipped phase lists after the guided-creation review's #1 and #11: no
/// profile still declares the read-only `type` step, and every one of them offers
/// the `experience` step before the `abilities` step it funds — the funding choice
/// applies to every character type, so a profile without it would leave that type
/// no way to choose how its Abilities are paid for.
///
/// Structural only: which phases a profile declares, never how many.
#[test]
fn every_shipped_profile_declares_experience_before_abilities_and_no_type_phase() {
    let ruleset = full_ruleset();
    assert!(
        ruleset.profiles().next().is_some(),
        "the shipped ruleset declares no character types"
    );

    for profile in ruleset.profiles() {
        let id = &profile.id;
        let phases = &profile.creation_phases;
        assert!(
            !phases.iter().any(|phase| phase.to_string() == "type"),
            "profile '{id}' still declares the removed `type` phase: {phases:?}"
        );

        let experience = phases.iter().position(|p| *p == CreationPhase::Experience);
        let abilities = phases.iter().position(|p| *p == CreationPhase::Abilities);
        assert!(
            experience.is_some(),
            "profile '{id}' declares no `experience` phase, so it cannot choose its funding mode"
        );
        if let (Some(experience), Some(abilities)) = (experience, abilities) {
            assert!(
                experience < abilities,
                "profile '{id}' funds Abilities after buying them: {phases:?}"
            );
        }
    }
}

#[test]
fn grog_full_build_validates() {
    // 3/3 V/F, no majors; categories general/personality/social_status only.
    let mut grog = base("grog");
    grog.characteristics = BTreeMap::from([
        (Characteristic::Sta, 2),
        (Characteristic::Str, 1),
        (Characteristic::Dex, 1),
        (Characteristic::Qik, 1),
    ]);
    grog.selections = vec![
        sel("virtue.tough"),
        sel("flaw.clumsy"),
        sel("flaw.ability_block"),
    ];
    grog.xp_pool = 60;
    grog.ability_scores = vec![
        ability("ability.awareness", 2),
        ability("ability.brawl", 3),
        ability("ability.athletics", 2),
    ];
    assert_valid("grog", &grog, &full_ruleset());
}

#[test]
fn companion_full_build_validates() {
    // 10/10 V/F.
    let mut comp = base("companion");
    comp.characteristics = BTreeMap::from([
        (Characteristic::Int, 2),
        (Characteristic::Com, 1),
        (Characteristic::Pre, 1),
    ]);
    comp.selections = vec![
        sel("virtue.tough"),
        sel("virtue.arcane_lore"),
        sel("flaw.clumsy"),
        sel("flaw.ability_block"),
        sel("flaw.arthritis"),
    ];
    comp.xp_pool = 120;
    comp.ability_scores = vec![
        ability("ability.awareness", 3),
        ability("ability.athletics", 2),
        ability("ability.brawl", 2),
    ];
    assert_valid("companion", &comp, &full_ruleset());
}

#[test]
fn mythic_companion_full_build_validates() {
    // 20 virtue / 10 flaw (rate 2), a mythic type whose grants are all fixed
    // (Nephilim) so no player choice is left unresolved.
    let mut myth = base("mythic_companion");
    myth.mythic_type = Some(Id::new("mythic_type.nephilim"));
    myth.characteristics = BTreeMap::from([(Characteristic::Int, 2), (Characteristic::Sta, 1)]);
    myth.selections = vec![sel("virtue.arcane_lore"), sel("flaw.clumsy")];
    myth.xp_pool = 120;
    myth.ability_scores = vec![ability("ability.awareness", 3)];
    assert_valid("mythic_companion", &myth, &full_ruleset());
}

#[test]
fn magus_full_build_validates() {
    // The Gift + Hermetic Magus (both free), a House with its choice-grant
    // resolved, Arts, and spells within the level cap and spell-levels budget.
    let mut mag = base("magus");
    mag.house = Some(Id::new("house.flambeau"));
    mag.house_choices = BTreeMap::from([(
        "flambeau_puissant".to_string(),
        Selection::with_params(
            Id::new("virtue.puissant_art"),
            BTreeMap::from([("art".to_string(), Id::new("art.ignem"))]),
        ),
    )]);
    mag.characteristics = BTreeMap::from([(Characteristic::Int, 3), (Characteristic::Sta, 1)]);
    mag.selections = vec![sel("virtue.the_gift"), sel("virtue.hermetic_magus")];
    mag.xp_pool = 240;
    mag.ability_scores = vec![
        AbilityScore {
            ability: Id::new("ability.dead_language"),
            score: 4,
            specialty: Some("Latin".into()),
            parameter: Some("Latin".into()),
        },
        ability("ability.artes_liberales", 1),
        ability("ability.magic_theory", 3),
        ability("ability.parma_magica", 1),
        ability("ability.awareness", 2),
    ];
    mag.art_scores = vec![
        ArtScore {
            art: Id::new("art.creo"),
            score: 10,
        },
        ArtScore {
            art: Id::new("art.ignem"),
            score: 10,
        },
        ArtScore {
            art: Id::new("art.rego"),
            score: 5,
        },
        ArtScore {
            art: Id::new("art.vim"),
            score: 4,
        },
    ];
    mag.spells = vec![
        SpellSelection {
            spell: Id::new("spell.bind_wound"),
            level: None,
            mastery: None,
            parameter: None,
            mastery_abilities: Vec::new(),
        },
        SpellSelection {
            spell: Id::new("spell.airs_ghostly_form"),
            level: None,
            mastery: None,
            parameter: None,
            mastery_abilities: Vec::new(),
        },
    ];
    assert_valid("magus", &mag, &full_ruleset());
}
