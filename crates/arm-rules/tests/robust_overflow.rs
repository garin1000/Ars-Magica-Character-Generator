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
//!
//! The `k3`/`k4`/`k5`/`row58` tests below are a second, later batch: K3-K6 and
//! E3/E5 of the 2026-10-12 full-audit round 1, plus the already-tracked
//! `docs/open-todos.md` row 58 — five more bare `.sum()`/`+=` folds over
//! `Entity`-supplied, caller-uncapped collections, found by the same "no
//! `saturating_add`" pattern as F5 above. `effective/xp.rs::general_pool_and_bonus`'s
//! `earmarked: u32 = ...sum()` (the fifth site K6/E3 named) is deliberately NOT
//! tested here: `capped_earmark_amounts` caps every pushed term at
//! `amount.min(remaining)` and shrinks `remaining` by exactly that amount each
//! iteration (`remaining = remaining.saturating_sub(effective)`), so by
//! induction the running total can never exceed the loop's own starting
//! `remaining` — itself a `u32` already built with `saturating_add`
//! (`pre_earmark_general_xp`). The sum is mathematically bounded by a value
//! that already fits in `u32`, so it cannot overflow no matter how many
//! earmark effects a crafted save carries. Confirmed by reading
//! `capped_earmark_amounts` and `pre_earmark_general_xp` (`effective/xp.rs`) in
//! full — writing a test that claims to turn this red would be dishonest.

use std::collections::BTreeMap;

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use arm_rules::{
    Characteristic, aging_total, checked_xp_allocation, crisis_total, derived_totals,
    effective_might, encumbrance, familiar_invested_power_levels, focus_points_used,
    item_level_used, powers_used, resolve_outcome, restricted_xp_pools, spell_levels_used,
    true_faith, warping_points_total,
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
    // +2 is SUBTRACTED from the total (ArMDE:16648), so i32::MIN - 2 underflows.
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

// --- K3-K5, row 58: four more sums over `u8`-bounded, save-controlled terms -
//
// Unlike F5's `u16`-bounded rows above, each term these four sites sum is
// bounded by `u8::MAX` (a Load, an aging-log Characteristic point, a
// Might/True-Faith/spell-level grant), so crossing `u32::MAX` needs
// `ceil(2^32 / 255) == 16_843_010` rows rather than F5's 65_538:
// `16_843_009 * 255 == u32::MAX` exactly, so one more row pushes the sum past
// it. Reachable only from a hand-edited save — the UI has no way to equip the
// same slot, grant the same Virtue, or learn the same spell millions of times
// over — but `Entity::equipment`/`selections`/`spells` are all plain `Vec`s
// with no length cap at deserialization.

const ROWS_PAST_U32_OVER_U8_MAX: usize = 16_843_010;

/// A ruleset with one `armor.overflow_test` item at Load 255 (`u8::MAX`) — the
/// real core catalogue's heaviest armour tops out at 7
/// (`armor.metal_scale_full`), which would need over 600 million equipped
/// rows to reach the same total as 255 does at this row count.
fn overflow_equipment_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: "[]",
        type_profiles: "[]",
        abilities: None,
        arts: None,
        houses: None,
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: None,
        equipment: Some(
            r#"{"weapons":[],"shields":[],"armor":[
                {"id":"armor.overflow_test","protection":0,"load":255}
            ]}"#,
        ),
        characteristics: None,
        life_stages: None,
        childhoods: None,
        aging: None,
        parameter_catalogues: None,
    })
    .expect("minimal one-armor ruleset loads")
}

/// K3: `derived/combat.rs::encumbrance`'s Load sum.
#[test]
fn k3_maxed_wielded_equipment_saturates_encumbrance_load_without_panicking() {
    let ruleset = overflow_equipment_ruleset();
    let mut e = entity("grog");
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("armor.overflow_test"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        };
        ROWS_PAST_U32_OVER_U8_MAX
    ];

    let total = encumbrance(&e, &ruleset);
    assert_eq!(
        total.load,
        u32::MAX,
        "the Load sum saturates rather than overflowing"
    );
}

/// `ceil(2^32 / (8 * u8::MAX))`: the fewest aging-log entries, each already
/// carrying the maximum point award at every one of the eight Characteristics
/// (`8 * 255 == 2040` per entry), whose summed points cross `u32::MAX`.
/// `2_105_376 * 2040 == 4_294_967_040`, 256 short of `u32::MAX + 1`, so one
/// more row is enough.
const ROWS_PAST_U32_OVER_EIGHT_CHARACTERISTICS: usize = 2_105_377;

/// K4: `aging.rs::decrepitude_points_as_of`'s `later` sum — reached only
/// through its one public caller, [`crisis_total`] (ArMDE:16696's "Increase
/// the character's Decrepitude first, and then roll on the Crisis Table").
/// Age 60 (less than every logged 61) makes every crafted entry count as
/// "later" and so feed the overflowing sum.
#[test]
fn k4_maxed_aging_log_saturates_decrepitude_points_as_of_without_panicking() {
    let ruleset = full_ruleset();
    let mut e = entity("companion");
    e.age = Some(60);
    let mut points = BTreeMap::new();
    for characteristic in Characteristic::ALL {
        points.insert(characteristic, u8::MAX);
    }
    e.aging_log = vec![
        AgingLogEntry {
            age: Some(61),
            points: points.clone(),
            ..AgingLogEntry::default()
        };
        ROWS_PAST_U32_OVER_EIGHT_CHARACTERISTICS
    ];

    let total = crisis_total(&e, &ruleset, 60, 0).expect("the core ruleset ships a crisis");
    assert_eq!(
        total.decrepitude_score, 0,
        "a later-points sum saturated at u32::MAX subtracts down to 0 (saturating_sub), \
         never underflows or panics"
    );
}

/// A ruleset with two Virtues whose effects grant the maximum single-item
/// amount (`u8::MAX` = 255) of Might and True Faith respectively — the real
/// core catalogue's largest Might grant is Demonic Blood's 5
/// (`virtue.demonic_blood`) and its only True Faith grant is 1
/// (`virtue.true_faith`), either of which would need well over a billion
/// selections to reach the same total as 255 does at this row count.
fn overflow_grants_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: r#"[
            {
                "id": "virtue.overflow_might",
                "kind": "virtue", "classification": "creation_effect",
                "magnitude": "minor", "categories": ["supernatural"],
                "entity_kinds": ["character"],
                "effects": [{ "type": "might_grant", "realm": "infernal", "score": 255 }]
            },
            {
                "id": "virtue.overflow_true_faith",
                "kind": "virtue", "classification": "creation_effect",
                "magnitude": "major", "categories": ["general"],
                "entity_kinds": ["character"],
                "effects": [{ "type": "true_faith_grant", "score": 255 }]
            },
            {
                "id": "flaw.overflow_personality_filler",
                "kind": "flaw", "classification": "narrative",
                "magnitude": "minor", "categories": ["personality"],
                "entity_kinds": ["character"]
            }
        ]"#,
        type_profiles: "[]",
        abilities: None,
        arts: None,
        houses: None,
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: None,
        equipment: None,
        characteristics: None,
        life_stages: None,
        childhoods: None,
        aging: None,
        parameter_catalogues: None,
    })
    .expect("minimal two-virtue ruleset loads")
}

/// K5: `effective/might.rs::effective_might`'s `granted` sum (and the plain
/// `base + granted` add that follows it — both must survive a saturated
/// `granted`, since `.sum()` panicking before the final `u8::try_from(...)
/// .unwrap_or(u8::MAX)` fallback is exactly the bug: the fallback never gets a
/// chance to run).
#[test]
fn k5_maxed_might_grants_saturate_effective_might_without_panicking() {
    let ruleset = overflow_grants_ruleset();
    let mut e = entity("companion");
    e.selections =
        vec![Selection::new(Id::new("virtue.overflow_might")); ROWS_PAST_U32_OVER_U8_MAX];

    let might = effective_might(&e, &ruleset).expect("a MightGrant establishes a Realm");
    assert_eq!(
        might.score,
        u8::MAX,
        "an overflowing granted sum clamps the Might Score at u8::MAX"
    );
}

/// K5: `effective/might.rs::true_faith`'s running `score +=` accumulator.
#[test]
fn k5_maxed_true_faith_grants_saturate_true_faith_without_panicking() {
    let ruleset = overflow_grants_ruleset();
    let mut e = entity("companion");
    e.selections =
        vec![Selection::new(Id::new("virtue.overflow_true_faith")); ROWS_PAST_U32_OVER_U8_MAX];

    assert_eq!(
        true_faith(&e, &ruleset),
        u8::MAX,
        "an overflowing granted sum clamps True Faith at u8::MAX"
    );
}

/// Row 58 (`docs/open-todos.md`): `effective/spell.rs::spell_levels_used`'s
/// sum. `spell.aegis_of_the_hearth` is a General spell in the shipped core
/// catalogue (no fixed `level`), so the per-character chosen level — clamped
/// to `u8::MAX` here — is what `resolved_spell_level` sums.
#[test]
fn row58_maxed_general_spell_levels_saturate_spell_levels_used_without_panicking() {
    let ruleset = full_ruleset();
    let mut e = entity("magus");
    let mut sel = SpellSelection::new(Id::new("spell.aegis_of_the_hearth"));
    sel.level = Some(u8::MAX);
    e.spells = vec![sel; ROWS_PAST_U32_OVER_U8_MAX];

    assert_eq!(
        spell_levels_used(&e, &ruleset),
        u32::MAX,
        "the level sum saturates rather than overflowing"
    );
}
