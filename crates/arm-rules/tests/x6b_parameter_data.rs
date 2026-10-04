//! X6b (`docs/vf-audit/design-x6-parameters.md` § 2) — data for the 16
//! rule-driving parameters landed on X6a's engine (`a3ed3de`, `c379bd5`).
//!
//! One behavioral test per entry, against the SHIPPED `rules/core/*.json`
//! (never a minimal in-test fixture) — proving the actual data, not the
//! engine machinery X6a's own `x6a_parameter_engine.rs` already covers.

use std::collections::BTreeMap;

use arm_rules::effective::ability_age_cap;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use arm_rules::{
    Characteristic, characteristic_score_bonus, magic_resistance, reputation_grants, soak,
};

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
        mythic_types: Some(include_str!(
            "../../../rules/core/mythic_companion_types.json"
        )),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
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
/// helper (e.g. `x5b_ability_minimums.rs`) — duplicated since integration
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

/// A "magus" entity with the two selections every Hermetically-trained
/// fixture needs (`virtue.the_gift` + `virtue.hermetic_magus`) so the
/// `hermetically_trained` prerequisite the Hermetic-category entries below
/// carry is satisfied — same precedent as `x7bd_wrong_numbers.rs`.
fn magus(mut selections: Vec<Selection>) -> Entity {
    let mut all = vec![sel("virtue.the_gift"), sel("virtue.hermetic_magus")];
    all.append(&mut selections);
    entity("magus", all)
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn sel_with(id: &str, params: BTreeMap<String, Id>) -> Selection {
    Selection::with_params(Id::new(id), params)
}

fn param(key: &str, value: &str) -> BTreeMap<String, Id> {
    BTreeMap::from([(key.to_string(), Id::new(value))])
}

fn ability_score(ability: &str, score: u8) -> AbilityScore {
    AbilityScore::new(Id::new(ability), score)
}

fn issue_codes(entity: &Entity, ruleset: &Ruleset) -> Vec<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

// --- virtue.commanding_aura ---------------------------------------------

#[test]
fn commanding_aura_pope_gives_mr_25_and_soak_5() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![sel_with(
            "virtue.commanding_aura",
            param("rank", "rank.pope"),
        )],
    );
    let mr = magic_resistance(&e, &rs);
    let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
    assert_eq!(ignem.total, 25, "Pope: Magic Resistance 25 (ArMDE:3585)");
    let soak_mod = soak(&e, &rs)
        .addends
        .iter()
        .find(|a| a.label == "soak_mod")
        .unwrap()
        .value;
    assert_eq!(soak_mod, 5, "Pope: Soak bonus +5 (ArMDE:3585)");
}

#[test]
fn commanding_aura_king_gives_mr_10_and_soak_2() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![sel_with(
            "virtue.commanding_aura",
            param("rank", "rank.king"),
        )],
    );
    let mr = magic_resistance(&e, &rs);
    let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
    assert_eq!(ignem.total, 10, "King: Magic Resistance 10 (ArMDE:17651)");
    let soak_mod = soak(&e, &rs)
        .addends
        .iter()
        .find(|a| a.label == "soak_mod")
        .unwrap()
        .value;
    assert_eq!(soak_mod, 2, "King: Soak bonus +2 (ArMDE:17651)");
}

#[test]
fn commanding_aura_missing_rank_reports_missing_param() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("virtue.commanding_aura")]);
    assert!(
        issue_codes(&e, &rs).iter().any(|c| c == "missing_param"),
        "an old save with no `rank` must report missing_param (Q-X6-4/D70)"
    );
}

// --- flaw.savantism ------------------------------------------------------

#[test]
fn savantism_favored_ability_caps_at_6_regardless_of_age() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![sel_with(
            "flaw.savantism",
            param("favored", "ability.craft"),
        )],
    );
    e.age = Some(25);
    assert_eq!(
        ability_age_cap(&e, &rs, &Id::new("ability.craft"), None),
        Some(6),
        "the favored Ability is limited to a score of 6 as a starting character \
         (ArMDE:6705) — ABOVE the age-25 band's own cap of 5"
    );
}

#[test]
fn savantism_other_abilities_cap_at_3_under_the_normal_age_cap() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![sel_with(
            "flaw.savantism",
            param("favored", "ability.craft"),
        )],
    );
    e.age = Some(25);
    assert_eq!(
        ability_age_cap(&e, &rs, &Id::new("ability.awareness"), None),
        Some(3),
        "he may not begin with an Ability above 3, other than his favored one \
         (ArMDE:6705)"
    );
}

// --- flaw.restricted_learning ---------------------------------------------

fn multi_ref_sel(id: &str, key: &str, values: &[&str]) -> Selection {
    let mut params = BTreeMap::new();
    params.insert(
        key.to_string(),
        SelectionParamValue::Multi(values.iter().map(|v| Id::new(*v)).collect()),
    );
    Selection {
        item_ref: Id::new(id),
        params,
    }
}

#[test]
fn restricted_learning_refuses_a_sixth_ability() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![multi_ref_sel(
            "flaw.restricted_learning",
            "abilities",
            &[
                "ability.artes_liberales",
                "ability.philosophiae",
                "ability.awareness",
                "ability.brawl",
                "ability.athletics",
                "ability.chirurgy",
            ],
        )],
    );
    assert!(
        issue_codes(&e, &rs)
            .iter()
            .any(|c| c == "wrong_param_count"),
        "six named Abilities against exact_count: 5 must be reported (ArMDE:6685)"
    );
}

#[test]
fn restricted_learning_blocks_xp_outside_the_five_and_supernatural() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![multi_ref_sel(
            "flaw.restricted_learning",
            "abilities",
            &[
                "ability.artes_liberales",
                "ability.philosophiae",
                "ability.awareness",
                "ability.brawl",
                "ability.athletics",
            ],
        )],
    );
    // In scope: named directly.
    e.ability_scores.push(ability_score("ability.awareness", 3));
    // In scope: the Supernatural-category union member (ArMDE:6685: "Any
    // Supernatural Abilities from Virtues she possesses are added").
    e.ability_scores.push(ability_score("ability.dowsing", 1));
    // Out of scope: neither named nor Supernatural.
    e.ability_scores
        .push(ability_score("ability.single_weapon", 1));
    assert!(
        issue_codes(&e, &rs)
            .iter()
            .any(|c| c == "ability_outside_restricted_scope"),
        "Single Weapon is neither named nor Supernatural: must be refused"
    );
}

// --- flaw.ability_block ---------------------------------------------------

#[test]
fn ability_block_martial_blocks_martial_xp() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![sel_with(
            "flaw.ability_block",
            BTreeMap::from([
                ("scope".to_string(), Id::new("scope.category")),
                ("class".to_string(), Id::new("ability_category.martial")),
            ]),
        )],
    );
    e.ability_scores
        .push(ability_score("ability.single_weapon", 2));
    assert!(
        issue_codes(&e, &rs)
            .iter()
            .any(|c| c == "ability_forbidden_by_effect"),
        "a Martial Ability score must be refused under Ability Block (Martial), ArMDE:5651-5654"
    );
}

#[test]
fn ability_block_custom_branch_stays_uncomputed() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![sel_with(
            "flaw.ability_block",
            BTreeMap::from([
                ("scope".to_string(), Id::new("scope.custom")),
                ("custom".to_string(), Id::new("no_Latin_or_all_Laws")),
            ]),
        )],
    );
    e.ability_scores
        .push(ability_score("ability.single_weapon", 2));
    assert!(
        !issue_codes(&e, &rs)
            .iter()
            .any(|c| c == "ability_forbidden_by_effect"),
        "the custom, free-text branch names no closed category and forbids nothing \
         computationally (ArMDE:5651-5654, D9's stated boundary)"
    );
}

// --- virtue.faerie_blood / virtue.strong_faerie_blood ----------------------

#[test]
fn faerie_blood_sidhe_adds_one_presence_dwarf_adds_nothing_to_it() {
    let rs = load_ruleset();
    let sidhe = entity(
        "companion",
        vec![sel_with(
            "virtue.faerie_blood",
            param("heritage", "heritage.sidhe"),
        )],
    );
    assert_eq!(
        characteristic_score_bonus(&sidhe, &rs, Characteristic::Pre),
        1,
        "Sidhe Blood: +1 to Presence (ArMDE:3815)"
    );

    let dwarf = entity(
        "companion",
        vec![sel_with(
            "virtue.faerie_blood",
            param("heritage", "heritage.dwarf"),
        )],
    );
    assert_eq!(
        characteristic_score_bonus(&dwarf, &rs, Characteristic::Pre),
        0,
        "Dwarf Blood grants no Presence bonus"
    );
}

#[test]
fn faerie_blood_dwarf_surfaces_a_craft_roll_bonus() {
    let rs = load_ruleset();
    let dwarf = entity(
        "companion",
        vec![sel_with(
            "virtue.faerie_blood",
            param("heritage", "heritage.dwarf"),
        )],
    );
    let rows = arm_rules::surfaced_modifiers(&dwarf, &rs);
    assert!(
        rows.iter()
            .any(|r| r.ability.as_ref().map(Id::as_str) == Some("ability.craft") && r.amount == 1),
        "Dwarf Blood: +1 bonus to any total including a Craft Ability (ArMDE:3809)"
    );
}

#[test]
fn strong_faerie_blood_requires_a_quirk_and_resolves_undine() {
    let rs = load_ruleset();
    let missing_quirk = entity(
        "companion",
        vec![sel_with(
            "virtue.strong_faerie_blood",
            param("heritage", "heritage.undine"),
        )],
    );
    assert!(
        issue_codes(&missing_quirk, &rs)
            .iter()
            .any(|c| c == "missing_param"),
        "the physical quirk is unconditional (ArMDE:5042) and must be required"
    );

    let complete = entity(
        "companion",
        vec![sel_with(
            "virtue.strong_faerie_blood",
            BTreeMap::from([
                ("heritage".to_string(), Id::new("heritage.undine")),
                ("quirk".to_string(), Id::new("webbed_fingers")),
            ]),
        )],
    );
    assert!(
        !issue_codes(&complete, &rs)
            .iter()
            .any(|c| c == "missing_param" || c == "unknown_param_value"),
        "a filled heritage + quirk must resolve cleanly"
    );
}

// --- flaw.monstrous_blood ---------------------------------------------------

#[test]
fn monstrous_blood_magic_human_lowers_a_characteristic_and_grants_reputation() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![sel_with(
            "flaw.monstrous_blood",
            BTreeMap::from([
                ("bloodline".to_string(), Id::new("bloodline.magic_human")),
                ("characteristic".to_string(), Id::new("characteristic.pre")),
            ]),
        )],
    );
    assert_eq!(
        characteristic_score_bonus(&e, &rs, Characteristic::Pre),
        -1,
        "Magic Human: decrease one Characteristic by 1 (ArMDE:6462)"
    );
    let grants = reputation_grants(&e, &rs);
    assert!(
        grants
            .iter()
            .any(|g| g.source.as_str() == "flaw.monstrous_blood" && g.score == 3),
        "Magic Human: a poor Reputation of level 3 among other magic beings (ArMDE:6462)"
    );
}

#[test]
fn monstrous_blood_magic_animal_grants_no_characteristic_delta_or_reputation() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![sel_with(
            "flaw.monstrous_blood",
            param("bloodline", "bloodline.magic_animal"),
        )],
    );
    assert_eq!(characteristic_score_bonus(&e, &rs, Characteristic::Pre), 0);
    assert!(
        reputation_grants(&e, &rs)
            .iter()
            .all(|g| g.source.as_str() != "flaw.monstrous_blood"),
        "Magic Animal carries no Reputation grant"
    );
}

// --- flaw.vengeful_powers ---------------------------------------------------

#[test]
fn vengeful_powers_resolves_hermetic_and_story_but_not_a_third_reading() {
    let rs = load_ruleset();
    let hermetic = entity(
        "magus",
        vec![
            sel("virtue.the_gift"),
            sel("virtue.hermetic_magus"),
            sel_with("flaw.vengeful_powers", param("taken_as", "hermetic")),
        ],
    );
    assert!(
        !issue_codes(&hermetic, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "Vengeful Powers may be taken as a Hermetic Flaw (ArMDE:6975)"
    );

    let story = entity(
        "companion",
        vec![sel_with("flaw.vengeful_powers", param("taken_as", "story"))],
    );
    assert!(
        !issue_codes(&story, &rs)
            .iter()
            .any(|c| c == "unknown_param_value")
    );

    let bogus = entity(
        "companion",
        vec![sel_with(
            "flaw.vengeful_powers",
            param("taken_as", "supernatural"),
        )],
    );
    assert!(
        issue_codes(&bogus, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "Vengeful Powers declares only story/hermetic as categories"
    );
}

// --- virtue.potent_magic_major / _minor -------------------------------------

#[test]
fn potent_magic_field_is_required_and_two_distinct_fields_are_legal() {
    let rs = load_ruleset();
    let missing = magus(vec![sel("virtue.potent_magic_major")]);
    assert!(
        issue_codes(&missing, &rs)
            .iter()
            .any(|c| c == "missing_param"),
        "Potent Magic's field must be required (D9 records every choice)"
    );

    let two_fields = magus(vec![
        sel_with("virtue.potent_magic_major", param("field", "Ignem")),
        sel_with("virtue.potent_magic_minor", param("field", "Auram")),
    ]);
    assert!(
        !issue_codes(&two_fields, &rs)
            .iter()
            .any(|c| c == "too_many_for_param_value"),
        "a maga may have more than one area of Potent Magic (ArMDE:4742)"
    );
}

// --- virtue.special_circumstances -------------------------------------------

#[test]
fn special_circumstances_two_different_circumstances_allowed_one_repeated_blocked() {
    let rs = load_ruleset();
    let two_different = magus(vec![
        sel_with(
            "virtue.special_circumstances",
            param("circumstance", "during a storm"),
        ),
        sel_with(
            "virtue.special_circumstances",
            param("circumstance", "while touching the target"),
        ),
    ]);
    assert!(
        !issue_codes(&two_different, &rs)
            .iter()
            .any(|c| c == "too_many_for_param_value"),
        "you may take this Virtue more than once (ArMDE:5000)"
    );

    let repeated = magus(vec![
        sel_with(
            "virtue.special_circumstances",
            param("circumstance", "during a storm"),
        ),
        sel_with(
            "virtue.special_circumstances",
            param("circumstance", "during a storm"),
        ),
    ]);
    assert!(
        issue_codes(&repeated, &rs)
            .iter()
            .any(|c| c == "duplicate_selection"),
        "F-541/F-287: two copies naming the SAME circumstance must collide — the parameter \
         makes them an exact-tuple duplicate under the default max_per_target of 1, the same \
         shape `text_target_param_items_repeat_across_targets_but_never_within_one` checks for \
         its own family"
    );
}

// --- virtue.performance_magic ------------------------------------------------

#[test]
fn performance_magic_requires_a_general_ability() {
    let rs = load_ruleset();
    let good = magus(vec![sel_with(
        "virtue.performance_magic",
        param("ability", "ability.awareness"),
    )]);
    assert!(
        !issue_codes(&good, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "a General Ability legally resolves (ArMDE:4646)"
    );

    let bad = magus(vec![sel_with(
        "virtue.performance_magic",
        param("ability", "ability.philosophiae"),
    )]);
    assert!(
        issue_codes(&bad, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "an Academic Ability must not resolve (ArMDE:4646: \"You may not choose \
         any Language, Supernatural, Academic, or Arcane Ability\")"
    );
}

// --- virtue.magian_lineage_major ---------------------------------------------

#[test]
fn magian_lineage_major_requires_exactly_three_arcane_or_supernatural_abilities() {
    let rs = load_ruleset();
    let three = entity(
        "companion",
        vec![multi_ref_sel(
            "virtue.magian_lineage_major",
            "abilities",
            &[
                "ability.dominion_lore",
                "ability.faerie_lore",
                "ability.magic_lore",
            ],
        )],
    );
    assert!(
        !issue_codes(&three, &rs)
            .iter()
            .any(|c| c == "wrong_param_count" || c == "unknown_param_value"),
        "three Arcane Abilities satisfy exact_count: 3 (ArMDE:4345)"
    );

    let two = entity(
        "companion",
        vec![multi_ref_sel(
            "virtue.magian_lineage_major",
            "abilities",
            &["ability.dominion_lore", "ability.faerie_lore"],
        )],
    );
    assert!(
        issue_codes(&two, &rs)
            .iter()
            .any(|c| c == "wrong_param_count"),
        "two Abilities against exact_count: 3 must be reported"
    );

    let martial = entity(
        "companion",
        vec![multi_ref_sel(
            "virtue.magian_lineage_major",
            "abilities",
            &[
                "ability.dominion_lore",
                "ability.faerie_lore",
                "ability.single_weapon",
            ],
        )],
    );
    assert!(
        issue_codes(&martial, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "a Martial Ability must not resolve Magian Lineage [Major]'s Arcane/Supernatural-only \
         parameter"
    );
}

#[test]
fn magian_lineage_major_data_forbids_true_names() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("virtue.magian_lineage_major"))
        .expect("virtue.magian_lineage_major must ship");
    let [param] = item.parameters.as_slice() else {
        panic!("expected exactly one parameter, got {:?}", item.parameters);
    };
    assert!(
        param.forbid_ids.contains(&Id::new("ability.true_names")),
        "ArMDE:4345: \"which cannot be True Names\" — no catalogued ability.true_names exists \
         yet to prove behaviorally, so this is a direct data assertion"
    );
}

// --- flaw.repellent ----------------------------------------------------------

#[test]
fn repellent_scales_gives_soak_3_natural_weapons_gives_nothing() {
    let rs = load_ruleset();
    let scales = entity(
        "companion",
        vec![sel_with(
            "flaw.repellent",
            param("feature", "feature.scales"),
        )],
    );
    let soak_mod = soak(&scales, &rs)
        .addends
        .iter()
        .find(|a| a.label == "soak_mod")
        .map(|a| a.value)
        .unwrap_or(0);
    assert_eq!(
        soak_mod, 3,
        "a scaled character might have a Soak bonus of +3 (ArMDE:6681)"
    );

    let natural_weapons = entity(
        "companion",
        vec![sel_with(
            "flaw.repellent",
            param("feature", "feature.natural_weapons"),
        )],
    );
    let soak_mod = soak(&natural_weapons, &rs)
        .addends
        .iter()
        .find(|a| a.label == "soak_mod")
        .map(|a| a.value)
        .unwrap_or(0);
    assert_eq!(
        soak_mod, 0,
        "natural weapons grant a melee use, not a Soak bonus — nothing is computed for it"
    );
}

// --- virtue.turb_trained ------------------------------------------------------

#[test]
fn turb_trained_authorizes_only_the_chosen_dead_language() {
    let rs = load_ruleset();
    let latin_chosen = entity(
        "companion",
        vec![sel_with(
            "virtue.turb_trained",
            param("language", "language.latin"),
        )],
    );
    let mut with_latin = latin_chosen;
    let mut a = AbilityScore::new(Id::new("ability.dead_language"), 1);
    a.parameter = Some(AbilityParameterValue::Catalogued {
        id: Id::new("language.latin"),
    });
    with_latin.ability_scores.push(a);
    assert!(
        !issue_codes(&with_latin, &rs)
            .iter()
            .any(|c| c == "ability_category_requires_virtue"),
        "the chosen language must be authorized"
    );

    let mut with_hebrew = entity(
        "companion",
        vec![sel_with(
            "virtue.turb_trained",
            param("language", "language.latin"),
        )],
    );
    let mut a = AbilityScore::new(Id::new("ability.dead_language"), 1);
    a.parameter = Some(AbilityParameterValue::Catalogued {
        id: Id::new("language.hebrew"),
    });
    with_hebrew.ability_scores.push(a);
    assert!(
        issue_codes(&with_hebrew, &rs)
            .iter()
            .any(|c| c == "ability_category_requires_virtue"),
        "a language other than the one chosen must NOT be authorized (ArMDE:5181: \"whichever \
         single dead language\")"
    );
}

// --- flaw.warped_senses --------------------------------------------------------

#[test]
fn warped_senses_weak_sight_excludes_keen_vision_but_weak_hearing_does_not() {
    let rs = load_ruleset();
    let weak_sight = entity(
        "companion",
        vec![
            sel_with(
                "flaw.warped_senses",
                param("affliction", "affliction.weak_sight"),
            ),
            sel("virtue.keen_vision"),
        ],
    );
    assert!(
        issue_codes(&weak_sight, &rs)
            .iter()
            .any(|c| c == "incompatible"),
        "Weak Sight is incompatible with Keen Vision (ArMDE:7033)"
    );

    let weak_hearing = entity(
        "companion",
        vec![
            sel_with(
                "flaw.warped_senses",
                param("affliction", "affliction.weak_hearing"),
            ),
            sel("virtue.keen_vision"),
        ],
    );
    assert!(
        !issue_codes(&weak_hearing, &rs)
            .iter()
            .any(|c| c == "incompatible"),
        "Weak Hearing carries no incompatibility with a sight-only entry"
    );
}
