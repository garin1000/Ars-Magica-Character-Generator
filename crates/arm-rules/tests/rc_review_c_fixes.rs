//! RC (review-2026-09-30c) fix slice — RED CHECKPOINT.
//!
//! Each test below pins a REAL DEFECT confirmed against the rulebook source
//! and `docs/vf-audit/decisions.md` (see `tmp/rc-verdicts.md` for the full
//! per-finding verdict table, including the Review C findings judged FALSE
//! POSITIVE and therefore NOT pinned here). Every test runs against the
//! SHIPPED `rules/core/*.json` and fails today (phase 2, data/engine work,
//! has not landed yet).

use std::collections::BTreeMap;

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;

fn load_ruleset() -> Ruleset {
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
/// helper (e.g. `x6b_parameter_data.rs`) — duplicated since integration test
/// binaries cannot share private helpers.
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
/// fixture needs (`virtue.the_gift` + `virtue.hermetic_magus`).
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

fn issue_codes(entity: &Entity, ruleset: &Ruleset) -> Vec<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

// --- Item 2: Warped Senses' same-copy pairing (ArMDE:7033) ------------------

/// ArMDE:7033: "Weak Sight is incompatible with Sensitive Sight, Keen Vision,
/// and Blind". The cross-item half (Keen Vision, Blind) is already encoded;
/// the same-copy half (Sensitive Sight, a DIFFERENT value of the SAME
/// `affliction` parameter on the SAME entry) is not — `forbids` only lists
/// OTHER items' ids, and self-referencing `flaw.warped_senses` would wrongly
/// also catch legal pairs (e.g. Sensitive to Cold + Sensitive to Heat).
/// x6b-handover.md gap #2.
#[test]
fn warped_senses_forbids_pairing_weak_sight_with_sensitive_sight() {
    let rs = load_ruleset();
    let e = entity(
        "grog",
        vec![
            sel_with(
                "flaw.warped_senses",
                param("affliction", "affliction.weak_sight"),
            ),
            sel_with(
                "flaw.warped_senses",
                param("affliction", "affliction.sensitive_sight"),
            ),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        result.issues.iter().any(|i| i.code == "incompatible"),
        "ArMDE:7033: Weak Sight is incompatible with Sensitive Sight, but selecting both \
         copies of flaw.warped_senses raises no incompatibility today: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

/// Same defect, hearing side: ArMDE:7033 "you cannot take Weak Hearing with
/// Sensitive Hearing, Sharp Ears, or Deaf" — Sharp Ears/Deaf are encoded,
/// Sensitive Hearing (same-copy) is not.
#[test]
fn warped_senses_forbids_pairing_weak_hearing_with_sensitive_hearing() {
    let rs = load_ruleset();
    let e = entity(
        "grog",
        vec![
            sel_with(
                "flaw.warped_senses",
                param("affliction", "affliction.weak_hearing"),
            ),
            sel_with(
                "flaw.warped_senses",
                param("affliction", "affliction.sensitive_hearing"),
            ),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        result.issues.iter().any(|i| i.code == "incompatible"),
        "ArMDE:7033: Weak Hearing is incompatible with Sensitive Hearing, but selecting both \
         copies of flaw.warped_senses raises no incompatibility today: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

// --- Item 3: Goblin heritage's stealth bonus (ArMDE:3811) -------------------

/// ArMDE:3811: "Goblin Blood: ... you get a +1 bonus on all totals involving
/// stealth" — a single named Ability (`ability.stealth`), exactly the same
/// computable shape as the already-encoded Dwarf (+1 Craft) bonus on both
/// entries.
#[test]
fn goblin_heritage_grants_a_stealth_roll_bonus() {
    let rs = load_ruleset();
    let expected = Effect::AbilityRollMod {
        ability: Id::new("ability.stealth"),
        amount: 1,
        gate: Some(ParamGate {
            param: "heritage".into(),
            equals: Id::new("heritage.goblin"),
        }),
    };
    for id in ["virtue.faerie_blood", "virtue.strong_faerie_blood"] {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert!(
            item.effects.contains(&expected),
            "ArMDE:3811: Goblin Blood's +1 stealth bonus is unmodeled on {id} (effects: {:?})",
            item.effects
        );
    }
}

// --- Item 4: Performance Magic's Ability filter (ArMDE:4646/:4676/:4684) ----

/// ArMDE:4676/:4684: Bows, Great Weapon, Single Weapon, Thrown Weapon, and
/// Brawl/Martial Abilities are explicitly legal Performance Magic choices
/// (with a combat-casting restriction, not a ban). `require_ability_categories:
/// ["general"]` wrongly rejects every Martial ability.
#[test]
fn performance_magic_accepts_a_martial_ability() {
    let rs = load_ruleset();
    let e = magus(vec![sel_with(
        "virtue.performance_magic",
        param("ability", "ability.bows"),
    )]);
    assert!(
        !issue_codes(&e, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "ArMDE:4676/:4684: Bows (a Martial Ability) is an explicitly legal Performance Magic \
         choice, but is rejected today"
    );
}

/// ArMDE:4646: "You may not choose any Language, Supernatural, Academic, or
/// Arcane Ability." `ability.living_language` is catalogued under `general`
/// (only `ability.dead_language` is `academic`), so the category filter alone
/// does not exclude it — `forbid_ids` (already used by
/// `virtue.magian_lineage_major` to exclude True Names) is the existing
/// mechanism for this shape.
#[test]
fn performance_magic_rejects_a_language_ability() {
    let rs = load_ruleset();
    let e = magus(vec![sel_with(
        "virtue.performance_magic",
        param("ability", "ability.living_language"),
    )]);
    assert!(
        issue_codes(&e, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "ArMDE:4646: Living Language must not resolve as a Performance Magic Ability, but is \
         accepted today"
    );
}

// --- Item 9: F-556 — virtue.domestic_animal is takeable by a human ---------

/// F-556/Q-11 (corrections.md): ArMDE:3701 "The character is an animal who is
/// the property of a covenant or character" — D58 rules animal characters a
/// deliberate non-goal (no Cunning characteristic, no animal profile), so the
/// entry must be gated so NO current (human) character type can select it.
/// Today it ships with no `prerequisites` at all, so a grog can take it and
/// `validate()` raises nothing about it — satisfying the mandatory Social
/// Status slot (D41) for free.
#[test]
fn f556_domestic_animal_is_refused_for_a_human_character_type() {
    let rs = load_ruleset();
    let target = Id::new("virtue.domestic_animal");
    let e = entity("grog", vec![sel(target.as_str())]);
    let result = validate(&e, &rs);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.context.as_ref() == Some(&target)),
        "F-556: a grog selecting virtue.domestic_animal raises no issue today (issues: {:?}) — \
         no human character type may take this entry (D58/Q-11)",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}
