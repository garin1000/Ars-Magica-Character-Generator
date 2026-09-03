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

/// The ordering the dangling-ability-target re-file rests on
/// (manual-testing-findings-2026-09-03 #5): `ability_bonus_dangling_target` is filed
/// on the `abilities` step, so a profile that bought Abilities BEFORE Virtues &
/// Flaws would file the finding on a step already walked past — the wizard would
/// carry it as an unreachable block. Structural, over whatever profiles ship.
#[test]
fn every_shipped_profile_buys_abilities_after_virtues_flaws() {
    let ruleset = full_ruleset();
    for profile in ruleset.profiles() {
        let phases = &profile.creation_phases;
        let virtues = phases
            .iter()
            .position(|p| *p == CreationPhase::VirtuesFlaws);
        let abilities = phases.iter().position(|p| *p == CreationPhase::Abilities);
        if let (Some(virtues), Some(abilities)) = (virtues, abilities) {
            assert!(
                virtues < abilities,
                "profile '{}' buys Abilities before Virtues & Flaws: {phases:?}",
                profile.id
            );
        }
    }
}

/// End-to-end over the REAL shipped catalogue: a magus who takes Puissant Ability
/// pointed at an Ability not yet bought must be held on the **Abilities** step, not
/// on the Virtues & Flaws step where the Virtue is taken. Puissant Ability is
/// "choose one Ability" with no requirement that a score exists
/// (Ars Magica - Definitive Edition (Core Rules).md:4814-4816), and Abilities are
/// bought later — filing it on `virtues_flaws` deadlocked the guided wizard, because
/// that step gates on its own findings and could offer no fix.
#[test]
fn puissant_ability_on_an_unbought_ability_holds_the_abilities_step_not_virtues_flaws() {
    let ruleset = full_ruleset();
    let mut magus = base("magus");
    magus.house = Some(Id::new("house.bonisagus"));
    magus.selections = vec![Selection::with_params(
        Id::new("virtue.puissant_ability"),
        BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
    )];

    let result = validate(&magus, &ruleset);
    let dangling: Vec<CreationPhase> = result
        .errors()
        .filter(|issue| issue.code == "ability_bonus_dangling_target")
        .map(|issue| issue.phase)
        .collect();
    assert_eq!(
        dangling,
        vec![CreationPhase::Abilities],
        "the dangling target belongs to the step that buys the Ability"
    );

    let on_virtues_flaws: Vec<&str> = result
        .errors()
        .filter(|issue| issue.phase == CreationPhase::VirtuesFlaws)
        .map(|issue| issue.code.as_str())
        .collect();
    assert!(
        !on_virtues_flaws.contains(&"ability_bonus_dangling_target"),
        "the V/F step must not block on an Ability bought later: {on_virtues_flaws:?}"
    );
    assert!(
        !on_virtues_flaws.contains(&"missing_param"),
        "a plain ability target needs no further key: {on_virtues_flaws:?}"
    );
}

/// Every shipped profile lets its type record Personality Traits.
///
/// The rules put no type outside this, and single out the one the guided flow used to
/// omit: "For major characters, such as magi and companions, they are normally nothing
/// more than an aide memoire … For grogs, they are more significant. As grogs are often
/// shared between players, or at least played rarely, the numbers attached to
/// Personality Traits can be used as a concrete guide to playing the character. …
/// 'Loyal' is a particularly important Trait, as it reflects the grog's attachment to
/// the covenant, while 'Brave' is just as important for warrior grogs. A third Trait
/// should be something distinctive about that grog."
/// (Ars Magica - Definitive Edition (Core Rules).md:1073-1075; the character-sheet
/// listing at :1165 names Personality Traits unconditionally.)
///
/// So the type whose traits the rules treat as mechanically load-bearing was the one
/// type whose guided flow could not record them (#33) — exactly backwards. The editor
/// always offered the tab, so this was a gap in the wizard alone.
///
/// Structural only: which phases a profile declares, never how many.
#[test]
fn every_shipped_profile_declares_the_personality_reputations_phase() {
    let ruleset = full_ruleset();
    assert!(
        ruleset.profiles().next().is_some(),
        "the shipped ruleset declares no character types"
    );

    for profile in ruleset.profiles() {
        let id = &profile.id;
        let phases = &profile.creation_phases;
        assert!(
            phases.contains(&CreationPhase::PersonalityReputations),
            "profile '{id}' declares no `personality_reputations` phase, so a character \
             of this type cannot record Personality Traits in the guided flow: {phases:?}"
        );
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

/// Measures the noise the guided wizard's rail would carry if it marked EVERY
/// phase holding a warning (manual-testing-findings #4c).
///
/// The character here is exactly what `startWizard('magus')` produces before the
/// user touches anything: the profile's mandatory free traits, no house, no
/// characteristics, no abilities, no arts, no spells, no age. Warnings are
/// advisories, so several fire on such a character at once — and they are spread
/// across *different* creation phases, which is what makes an ungated marker
/// light up steps the player has never opened. The frontend therefore shows the
/// pending-warning marker only for phases already reached
/// (`wizard_furthest_phase`); this test is the measurement that decision rests on.
/// Measured on 2026-09-03 against the shipped catalogue: 7 warnings —
/// `missing_hermetic_flaw`, `house_unset`, `spell_levels_unspent` and four
/// `magus_recommended_ability` — over 4 of the magus rail's 11 steps
/// (`virtues_flaws`, `house_specialisation`, `abilities`, `spells`), none of which
/// the player has opened while standing on step 1.
///
/// Structural, never a total: it asserts the spread is wide (more than one phase
/// beyond the first step), not how many warnings the catalogue happens to emit.
#[test]
fn a_fresh_wizard_magus_already_carries_warnings_on_several_unreached_phases() {
    let ruleset = full_ruleset();
    let mut mag = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    mag.selections = vec![sel("virtue.the_gift"), sel("virtue.hermetic_magus")];

    let result = validate(&mag, &ruleset);
    let warned: std::collections::BTreeSet<CreationPhase> =
        result.warnings().map(|issue| issue.phase).collect();
    let codes: Vec<&str> = result.warnings().map(|issue| issue.code.as_str()).collect();

    assert!(
        warned.len() > 1,
        "a fresh magus warns on {} phase(s): {warned:?} from {codes:?}",
        warned.len()
    );
    assert!(
        warned.iter().any(|p| *p != CreationPhase::Concept),
        "every warning sits on the wizard's first step, so no gate would be needed: {codes:?}"
    );
}
