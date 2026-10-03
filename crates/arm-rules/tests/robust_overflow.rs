//! Overflow robustness on the untrusted input path — findings F1, F3, F4 and F5
//! of the 2026-10-03 robustness review (`tmp/robust-review.md`).
//!
//! The root `Cargo.toml` sets `[profile.release] overflow-checks = true`
//! (locked by `overflow_checks_profile.rs`), so in the shipped binary an
//! unchecked `+` or `Iterator::sum` that overflows **panics**: the app dies
//! and the unsaved document goes with it. Each test below feeds the crafted
//! (or UI-typeable) input the review names through the public API the app
//! calls, and asserts two things: no panic, and a sane saturated or clamped
//! result. None of them changes a rules result for a realistic input; those
//! stay pinned by the existing unit tests beside each function.
//!
//! F2 (an absurd `age` materialising one schedule row per year) is a product
//! decision, not a mechanical fix, and is analysed in `tmp/ovf-handover.md`
//! instead of tested here. F6 is out of scope.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use arm_rules::{
    aging_total, checked_xp_allocation, crisis_total, derived_totals,
    familiar_invested_power_levels, focus_points_used, item_level_used, powers_used,
    resolve_outcome, restricted_xp_pools, warping_points_total,
};

/// The shipped core ruleset — same set of files `x10bc_banked_xp_and_within_focus.rs`
/// loads; duplicated because integration test binaries cannot share private
/// helpers.
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

fn entity(type_id: &str) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.ability_funding = AbilityFunding::Pool;
    e
}

fn ability_banked_at_max(ability: &str) -> AbilityScore {
    let mut a = AbilityScore::new(Id::new(ability), 1);
    a.banked_xp = u32::MAX;
    a
}

fn art_banked_at_max(art: &str) -> ArtScore {
    let mut a = ArtScore::new(Id::new(art), 1);
    a.banked_xp = u32::MAX;
    a
}

fn has_issue(e: &Entity, ruleset: &Ruleset, code: &str) -> bool {
    validate(e, ruleset).issues.iter().any(|i| i.code == code)
}

// --- F1: the XP flow solve's demand sum -------------------------------------
//
// Reachable from plain UI typing: the banked-XP boxes clamp to `U32_MAX`, so
// two rows typed as `9999999999` are enough.

#[test]
fn f1_two_abilities_banked_at_u32_max_report_not_enough_xp_without_panicking() {
    let ruleset = full_ruleset();
    let mut e = entity("grog");
    e.ability_scores = vec![
        ability_banked_at_max("ability.awareness"),
        ability_banked_at_max("ability.athletics"),
    ];

    let allocation = checked_xp_allocation(&e, &ruleset).expect("well under the node bound");
    assert_eq!(
        allocation.total_demand,
        u32::MAX,
        "the demand saturates rather than overflowing"
    );
    assert!(allocation.total_demand > allocation.max_flow);
    assert!(
        has_issue(&e, &ruleset, "not_enough_xp"),
        "a saturated demand is still over budget"
    );
    let _ = restricted_xp_pools(&e, &ruleset);
}

#[test]
fn f1_two_arts_banked_at_u32_max_report_not_enough_xp_without_panicking() {
    let ruleset = full_ruleset();
    let mut e = entity("magus");
    e.art_scores = vec![art_banked_at_max("art.creo"), art_banked_at_max("art.rego")];

    let allocation = checked_xp_allocation(&e, &ruleset).expect("well under the node bound");
    assert_eq!(allocation.total_demand, u32::MAX);
    assert!(has_issue(&e, &ruleset, "not_enough_xp"));
}

#[test]
fn f1_one_ability_and_one_art_banked_at_u32_max_report_not_enough_xp_without_panicking() {
    let ruleset = full_ruleset();
    let mut e = entity("magus");
    e.ability_scores = vec![ability_banked_at_max("ability.awareness")];
    e.art_scores = vec![art_banked_at_max("art.creo")];

    assert!(has_issue(&e, &ruleset, "not_enough_xp"));
}

/// The max-flow side of F1: with a general pool typed at `U32_MAX` plus a
/// restricted pool (Warrior's 50 martial XP), the two solve phases together
/// fund more than `u32::MAX`, so `restricted_flow + max_flow(..)` overflows
/// even once the demand sum is safe. Asserts no panic and figures that stay
/// inside `u32`; see the handover for why "over budget" cannot be asserted
/// when supply AND demand are both beyond `u32::MAX`.
#[test]
fn f1_a_u32_max_pool_plus_a_restricted_pool_does_not_overflow_the_flow_total() {
    let ruleset = full_ruleset();
    let mut e = entity("grog");
    e.xp_pool = u32::MAX;
    e.selections = vec![Selection::new(Id::new("virtue.warrior"))];
    e.ability_scores = vec![
        ability_banked_at_max("ability.single_weapon"),
        ability_banked_at_max("ability.athletics"),
    ];

    let allocation = checked_xp_allocation(&e, &ruleset).expect("well under the node bound");
    assert_eq!(allocation.total_demand, u32::MAX);
    assert!(allocation.max_flow <= allocation.total_demand);
    assert!(allocation.general_used <= allocation.general_pool);
    let _ = validate(&e, &ruleset);
    let _ = restricted_xp_pools(&e, &ruleset);
}

// --- F3: the aging and crisis totals ----------------------------------------
//
// Reachable from plain UI typing: the aging calculator's die boxes clamp only
// to `I32_MAX`, and every owed year adds an age modifier of at least 4.

#[test]
fn f3_a_stress_die_at_i32_max_clamps_the_aging_total_without_panicking() {
    let ruleset = full_ruleset();
    let mut e = entity("companion");
    e.age = Some(60);

    let total = aging_total(&e, &ruleset, 60, i32::MAX).expect("the core ruleset ships aging");
    assert_eq!(total.uncapped_total, i32::MAX, "clamped, not wrapped");
    assert_eq!(total.total, i32::MAX);
    assert!(
        resolve_outcome(&e, &ruleset, total.total).is_some(),
        "the clamped total still lands on the table's top row"
    );
}

#[test]
fn f3_a_die_at_i32_min_clamps_the_aging_total_without_panicking() {
    let ruleset = full_ruleset();
    let mut e = entity("companion");
    e.age = Some(60);
    // +2 is SUBTRACTED from the total (ArMDE:16571), so i32::MIN - 2 underflows.
    e.living_conditions = [Id::new("living_condition.wealthy_or_healthy_location")]
        .into_iter()
        .collect();

    let total = aging_total(&e, &ruleset, 0, i32::MIN).expect("the core ruleset ships aging");
    assert_eq!(total.uncapped_total, i32::MIN, "clamped, not wrapped");
    assert_eq!(total.total, i32::MIN);
}

#[test]
fn f3_a_crisis_die_at_i32_max_clamps_the_crisis_total_without_panicking() {
    let ruleset = full_ruleset();
    let mut e = entity("companion");
    e.age = Some(60);

    let total = crisis_total(&e, &ruleset, 60, i32::MAX).expect("the core ruleset ships a crisis");
    assert_eq!(total.total, i32::MAX, "clamped, not wrapped");
}

// --- F4: Raised from the Dead's years parameter -----------------------------
//
// Crafted save only: the UI clamps the box to the declared 0..=200.

#[test]
fn f4_a_crafted_years_since_resurrection_saturates_warping_without_panicking() {
    let ruleset = full_ruleset();
    let mut e = entity("companion");
    let mut s = Selection::new(Id::new("flaw.raised_from_the_dead"));
    s.params.insert(
        "years_since_resurrection".into(),
        SelectionParamValue::Single(Id::new(u32::MAX.to_string())),
    );
    e.selections = vec![s];

    assert_eq!(
        warping_points_total(&e, &ruleset),
        u32::MAX,
        "3 base points + u32::MAX years saturates"
    );
    let _ = validate(&e, &ruleset);
    let _ = derived_totals(&e, &ruleset);
}

// --- F5: four sums over save-controlled `u16` rows ---------------------------
//
// Crafted save only: tens of thousands of maxed rows. 65_538 * 65_535 is
// 4_295_032_830, just past u32::MAX (4_294_967_295).

const ROWS_PAST_U32: usize = 65_538;

#[test]
fn f5_maxed_devices_saturate_the_item_level_used_without_panicking() {
    let mut e = entity("companion");
    e.devices = vec![
        EnchantedDevice {
            name: String::new(),
            level: u16::MAX,
        };
        ROWS_PAST_U32
    ];
    assert_eq!(item_level_used(&e), u32::MAX);
}

#[test]
fn f5_maxed_powers_saturate_the_powers_used_without_panicking() {
    let mut e = entity("companion");
    e.powers = vec![
        SupernaturalPower {
            name: String::new(),
            level: u16::MAX,
            penetration: u16::MAX,
        };
        ROWS_PAST_U32 / 2 + 1
    ];
    assert_eq!(powers_used(&e), u32::MAX);
}

#[test]
fn f5_maxed_focus_powers_saturate_the_focus_points_used_without_panicking() {
    let mut e = entity("companion");
    e.focus_powers = vec![
        FocusPower {
            name: String::new(),
            max_level: u16::MAX,
            penetration: u16::MAX,
        };
        ROWS_PAST_U32 / 3 + 1
    ];
    assert_eq!(focus_points_used(&e), u32::MAX);
}

#[test]
fn f5_maxed_familiar_powers_saturate_the_invested_levels_without_panicking() {
    let familiar = Familiar {
        powers: vec![
            SupernaturalPower {
                name: String::new(),
                level: u16::MAX,
                penetration: 0,
            };
            ROWS_PAST_U32
        ],
        ..Familiar::default()
    };
    assert_eq!(familiar_invested_power_levels(&familiar), u32::MAX);
}
