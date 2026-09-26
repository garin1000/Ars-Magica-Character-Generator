use arm_rules::AbilityCategory;
use arm_rules::Characteristic;
use arm_rules::aging::{
    AgingOutcome, AgingPointAward, AgingPointTarget, AgingTotal, CrisisAllowance, CrisisModifier,
    CrisisModifierSource, CrisisOutcome, CrisisSeverity, CrisisSurvival,
};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationIssue, compute_balance, validate};
use arm_rules::{AgingRowEffect, AgingRules};
use arm_rules::{
    Grant, GrantConstraint, effective_art_score, effective_characteristic_after_aging,
    effective_characteristic_score, open_pick_satisfies, warping_owed_grants,
};
use arm_rules::{LifeStageBlock, LifeStagePlan, XpPoolOrigin, checked_xp_allocation};
use std::collections::{BTreeMap, BTreeSet};

/// The shipped House registry. Every helper below loads it, because the four
/// Outer-Mystery Virtues carry a `House` prerequisite that referential integrity
/// resolves against it — exactly as the production loader does
/// (`arm-app/src/ruleset_io.rs` lists `core/houses.json` as required).
const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

/// The shipped type-profile registry, read raw for [`profile_trait_reference_ids`]
/// — `EntityTypeProfile::required_traits`/`forbidden_traits` are `pub` on the
/// struct, but nothing publicly exposes the *set of all profiles* outside the
/// crate (`Ruleset::type_profiles` is `pub(crate)`), so this test parses the
/// same JSON the loader does rather than reaching for a production accessor
/// that does not exist.
const SHIPPED_TYPE_PROFILES: &str = include_str!("../../../rules/core/character_types.json");

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        houses: Some(SHIPPED_HOUSES),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        ..RulesetSources::default()
    })
    .unwrap()
}

/// The full shipped ruleset including Arts and the spell catalogue — the spell
/// tests need Arts loaded so each spell's Technique/Form resolves.
fn load_ruleset_with_spells() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(SHIPPED_HOUSES),
        mythic_types: None,
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: None,
        equipment: None,
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: None,
    })
    .unwrap()
}

/// The full shipped ruleset including the Spell Mastery special-ability
/// catalogue.
fn load_ruleset_with_mastery_abilities() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(SHIPPED_HOUSES),
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: Some(include_str!(
            "../../../rules/core/spell_mastery_abilities.json"
        )),
        equipment: None,
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: None,
    })
    .unwrap()
}

/// The full shipped ruleset including the equipment catalogue — the equipment
/// tests need Abilities loaded so each weapon's combat Ability resolves.
fn load_ruleset_with_equipment() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(SHIPPED_HOUSES),
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: None,
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: None,
    })
    .unwrap()
}

/// Builds a character entity of `type_id` with the given selections, at the
/// current schema version and empty trait data.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

#[test]
fn shipped_data_passes_integrity_check() {
    let rs = load_ruleset();
    // Catalogue size is data, not code: assert key items are present, never an
    // exact V/F total (which would break when any item is added to the JSON).
    assert!(
        rs.item(&Id::new("virtue.the_gift")).is_some(),
        "virtue.the_gift must be present"
    );
    // All four character-type profiles must load.
    assert!(rs.profile(&Id::new("companion")).is_some());
    assert!(rs.profile(&Id::new("grog")).is_some());
    assert!(rs.profile(&Id::new("magus")).is_some());
    assert!(rs.profile(&Id::new("mythic_companion")).is_some());
    // The magus is the only seeded Hermetic type.
    let magus_profile = rs.profile(&Id::new("magus")).unwrap();
    assert!(
        magus_profile.hermetically_trained,
        "magus profile must carry hermetically_trained"
    );
    assert!(
        magus_profile.order_member,
        "magus profile must carry order_member"
    );
    // Mythic Companions convert each Flaw point into two Virtue points.
    assert_eq!(
        rs.profile(&Id::new("mythic_companion"))
            .unwrap()
            .budget
            .virtue_points_per_flaw_point,
        2,
        "mythic companion funds virtues at 2:1"
    );
}

/// The two Virtues/Flaws that change the later-life experience rate, and the
/// eligibility the same rules line puts on them.
///
/// Poor was absent from the shipped catalogue entirely until this milestone (the
/// extraction plausibly dropped it because six other flaw names begin with
/// "Poor"), so this test is as much a guard against losing it again as a check on
/// its numbers.
///
/// Source: ArMDE:2394 ("Characters with
/// the Wealthy Virtue get 20 experience points per year, while characters with the
/// Poor Flaw get 10 … Note that only companions can take this Virtue or Flaw"),
/// `ArMDE:5235-5238` (Wealthy), `ArMDE:6594-6596` (Poor: "this Flaw is not available to
/// magi").
#[test]
fn wealthy_and_poor_ship_with_their_rates_and_eligibility() {
    let rs = load_ruleset();

    let rate_of = |id: &str| -> u32 {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} ships"));
        item.effects
            .iter()
            .find_map(|e| match e {
                Effect::LaterLifeXpRate { amount } => Some(*amount),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{id} carries a later-life rate"))
    };
    assert_eq!(rate_of("virtue.wealthy"), 20);
    assert_eq!(rate_of("flaw.poor"), 10);

    // Both are Major, per their own entries.
    for id in ["virtue.wealthy", "flaw.poor"] {
        assert_eq!(
            rs.item(&Id::new(id)).unwrap().magnitude,
            Magnitude::Major,
            "{id} is a Major Virtue/Flaw"
        );
    }

    // "Only companions can take this Virtue or Flaw" (D38): stated once, on the
    // entry, as a `Prereq` — not duplicated across every type profile's
    // `forbidden_traits`.
    for id in ["virtue.wealthy", "flaw.poor"] {
        assert_eq!(
            rs.item(&Id::new(id)).unwrap().prerequisites.as_ref(),
            Some(&Prereq::IsCompanion),
            "{id} must restrict itself to companions on the entry (ArMDE:2394, D38)"
        );
    }
    // Mythic Companion counts as a companion (Norbert's ruling, recorded in
    // RULES.md): its profile sets `is_companion`, so it is no longer named in
    // any `forbidden_traits` list for these two.
    assert!(
        rs.profile(&Id::new("mythic_companion"))
            .unwrap()
            .is_companion,
        "mythic_companion must set is_companion (\"mythic companions are companions too\")"
    );
    assert!(
        !rs.profile(&Id::new("magus")).unwrap().is_companion,
        "magus must not set is_companion"
    );
    // The two `forbidden_traits` entries D38 replaces must be gone, or the rule
    // is stated in two places again.
    for type_id in ["magus", "mythic_companion"] {
        let profile = rs.profile(&Id::new(type_id)).expect("profile ships");
        for id in ["virtue.wealthy", "flaw.poor"] {
            assert!(
                !profile.forbidden_traits.contains(&Id::new(id)),
                "{type_id} must no longer list {id} in forbidden_traits (D38 moved it to the entry)"
            );
        }
    }
    // The grog's incidental cover (no Major V/F at all) is untouched by D38 and
    // stays true regardless — it is simply no longer load-bearing for this rule.
    assert_eq!(
        rs.profile(&Id::new("grog")).unwrap().budget.max_major_flaws,
        Some(0),
        "a grog takes no Major Flaw"
    );
}

/// D38: a companion or a mythic companion is exactly who `ArMDE:2394` permits
/// to take Wealthy/Poor ("mythic companions are companions too").
#[test]
fn a_companion_or_mythic_companion_may_take_wealthy_or_poor() {
    let rs = load_ruleset();
    for type_id in ["companion", "mythic_companion"] {
        for id in ["virtue.wealthy", "flaw.poor"] {
            let e = entity(type_id, vec![Selection::new(Id::new(id))]);
            let codes = issue_codes(&e, &rs);
            assert!(
                !codes.contains(&"prereq_not_met".to_string()),
                "{type_id} taking {id} should not report prereq_not_met: {codes:?}"
            );
        }
    }
}

/// D38/F-339: a magus and a grog are neither of them companions, so each must
/// report `prereq_not_met` for Wealthy/Poor — stated, not merely incidental
/// (the grog case used to be covered only by its Major-V/F budget, F-339's
/// asymmetry).
#[test]
fn a_magus_or_grog_may_not_take_wealthy_or_poor() {
    let rs = load_ruleset();
    for type_id in ["magus", "grog"] {
        for id in ["virtue.wealthy", "flaw.poor"] {
            let e = entity(type_id, vec![Selection::new(Id::new(id))]);
            let codes = issue_codes(&e, &rs);
            assert!(
                codes.contains(&"prereq_not_met".to_string()),
                "{type_id} taking {id} must report prereq_not_met (ArMDE:2394, D38): {codes:?}"
            );
        }
    }
}

/// F-553: `virtue.magical_mount`'s "only a companion or magus-level character
/// can take this Virtue" (`ArMDE:4375`) was unencoded before this milestone —
/// a grog could take it. "Companion" reads as `IsCompanion` (D38: also true
/// of `mythic_companion`); "magus-level" is undefined in the passage and is
/// read conservatively as full Order membership (`Prereq::OrderMember`, true
/// only of the `magus` profile today), since nothing in the text supports
/// widening it further. See `RULES.md` for the full rationale, including the
/// Redcap counter-example this reading was weighed against.
#[test]
fn magical_mount_requires_companion_or_order_member() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("virtue.magical_mount"))
        .expect("virtue.magical_mount must ship in the catalogue");
    assert_eq!(
        item.prerequisites.as_ref(),
        Some(&Prereq::Any(vec![Prereq::IsCompanion, Prereq::OrderMember])),
        "ArMDE:4375 names two audiences: a companion, or a magus-level character (F-553)"
    );
}

/// The audiences the passage names, `mythic_companion` included (D38).
#[test]
fn a_companion_mythic_companion_or_magus_may_take_magical_mount() {
    let rs = load_ruleset();
    for type_id in ["companion", "mythic_companion", "magus"] {
        let e = entity(
            type_id,
            vec![Selection::new(Id::new("virtue.magical_mount"))],
        );
        let codes = issue_codes(&e, &rs);
        assert!(
            !codes.contains(&"prereq_not_met".to_string()),
            "{type_id} is exactly who ArMDE:4375 permits: {codes:?}"
        );
    }
}

/// The finding itself: a grog is neither a companion nor an Order member.
#[test]
fn a_grog_may_not_take_magical_mount() {
    let rs = load_ruleset();
    let e = entity(
        "grog",
        vec![Selection::new(Id::new("virtue.magical_mount"))],
    );
    let codes = issue_codes(&e, &rs);
    assert!(
        codes.contains(&"prereq_not_met".to_string()),
        "grog is neither a companion nor an Order member (F-553): {codes:?}"
    );
}

/// Row 47 / V/F-audit F-524: `flaw.unspecialized` (ArMDE:6943-6946) — "The
/// character does not have any specialties for any of her Abilities"
/// (ArMDE:6945) — must ship the marker effect the validator reads, and its
/// classification must have moved off `narrative` now that the rule is
/// computed at creation (D46: classification follows what is computed).
#[test]
fn unspecialized_ships_its_forbids_specialties_effect() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.unspecialized"))
        .expect("flaw.unspecialized ships");
    assert_eq!(item.classification, Classification::CreationEffect);
    assert!(
        item.effects
            .iter()
            .any(|e| matches!(e, Effect::ForbidsAbilitySpecialties)),
        "flaw.unspecialized must carry Effect::ForbidsAbilitySpecialties"
    );
}

/// Foreign Upbringing halves the creation cap on locality-dependent Abilities, so
/// the Flaw must carry the fraction and the three Ability families the passage names
/// must carry the flag.
///
/// > The maximum scores at character creation for locality-dependent Abilities like
/// > Language, Area Lore, or Organization Lore, as well as some social Abilities, are
/// > half (round up) that which his age normally allows.
///
/// Source: ArMDE:6160. The trailing "some
/// social Abilities" is deliberately unflagged — see RULES.md.
#[test]
fn foreign_upbringing_halves_the_locality_dependent_abilities() {
    let rs = load_ruleset();

    let flaw = rs
        .item(&Id::new("flaw.foreign_upbringing"))
        .expect("flaw.foreign_upbringing ships");
    let fraction = flaw
        .effects
        .iter()
        .find_map(|e| match e {
            Effect::LocalityAbilityCapFraction { num, den } => Some((*num, *den)),
            _ => None,
        })
        .expect("it carries the cap fraction");
    assert_eq!(fraction, (1, 2), "half");

    for id in [
        "ability.living_language",
        "ability.dead_language",
        "ability.area_lore",
        "ability.organization_lore",
    ] {
        assert!(
            rs.ability(&Id::new(id))
                .expect("ability ships")
                .locality_dependent,
            "{id} is locality-dependent (ArMDE:6160)"
        );
    }
    // A plainly non-local Ability is not flagged, so the fraction has a real edge.
    assert!(
        !rs.ability(&Id::new("ability.brawl"))
            .unwrap()
            .locality_dependent
    );
}

#[test]
fn shipped_abilities_and_characteristics_load() {
    let rs = load_ruleset();
    // Catalogue size is data, not code: assert the items the engine relies
    // on are present and read correctly, never an exact ability total. `main`
    // ships the full 78-ability Core Rules catalogue; the check stays
    // total-agnostic so a future catalogue edit needs no test change — see
    // crates/arm-rules/RULES.md.
    assert!(rs.ability(&Id::new("ability.awareness")).is_some());
    // The whole childhood restricted list ships (Core Rules 2378).
    for id in [
        "ability.area_lore",
        "ability.athletics",
        "ability.awareness",
        "ability.brawl",
        "ability.charm",
        "ability.folk_ken",
        "ability.guile",
        "ability.living_language",
        "ability.stealth",
        "ability.survival",
        "ability.swim",
    ] {
        assert!(rs.ability(&Id::new(id)).is_some(), "missing {id}");
    }

    // Supernatural Abilities ship and carry the supernatural category.
    for id in [
        "ability.second_sight",
        "ability.premonitions",
        "ability.animal_ken",
    ] {
        let ability = rs
            .ability(&Id::new(id))
            .expect("supernatural ability present");
        assert_eq!(
            ability.category,
            AbilityCategory::Supernatural,
            "{id} must be supernatural"
        );
    }

    // The `*` marker is the per-ability `requires_training` flag (cannot be used
    // untrained), NOT the supernatural category: it spans General/Academic/Arcane
    // too. Source: ArMDE:4157
    // (Jack of All Trades) — heading asterisks set it.
    for (id, expected) in [
        ("ability.artes_liberales", true), // Academic, asterisked
        ("ability.magic_theory", true),    // Arcane, asterisked
        ("ability.area_lore", true),       // General, asterisked
        ("ability.second_sight", true),    // Supernatural, asterisked
        ("ability.awareness", false),      // General, usable untrained
        ("ability.penetration", false),    // Arcane but explicitly not asterisked
    ] {
        let ability = rs.ability(&Id::new(id)).expect("ability present");
        assert_eq!(
            ability.requires_training, expected,
            "{id} requires_training must be {expected}"
        );
    }

    let chars = rs
        .characteristic_rules()
        .expect("characteristic rules present");
    assert_eq!(chars.start_points, 7);
    // The cost table is exactly the printed ±3 range, which is also the buy
    // range — Great/Poor (Characteristic) grant a free delta on top rather than
    // widening it (see characteristic_cost_table_prices_only_the_printed_rows).
    assert_eq!(chars.min_score(), Some(-3));
    assert_eq!(chars.max_score(), Some(3));
    assert_eq!(chars.base_max_score(), Some(3));
    assert_eq!(chars.base_min_score(), Some(-3));
    // The buy range is the whole of what this file limits. There is deliberately
    // no aging floor: `ArMDE:16579` names none, so the engine states none.

    // Advancement table is triangular: score 5 costs 75 xp total.
    assert_eq!(rs.advancement().xp_for_score(5), Some(75));
    assert_eq!(rs.advancement().xp_to_raise(5), Some(25));
}

/// Aging drops run all the way down: the aged score is the arithmetic the rule
/// describes and nothing else. `ArMDE:16579` gives only the drop condition ("Once
/// a character has a number of Aging Points greater than the absolute value of the
/// Characteristic, the Characteristic drops by one point and all Aging Points are
/// lost") and names no minimum, so the engine imposes none — a decrepit character
/// may end up far weaker than any character could be *built*, which is the whole
/// point of a Characteristic that falls with age.
///
/// The result is bounded by the data rather than by a clamp: `Entity::aging_points`
/// is a `u8` per Characteristic, and each successive drop costs one point more than
/// the last, so 255 points on a -3 Stamina buys 19 drops and no more. That is why
/// removing the clamp costs nothing in robustness.
#[test]
fn aging_drops_run_to_the_arithmetic_with_no_invented_floor() {
    let rs = load_ruleset();
    let mut e = entity("grog", vec![]);
    e.characteristics.insert(Characteristic::Sta, -3);

    // The costs run 4, 5, 6, … from a -3 score; 255 points fund 19 of them and
    // leave 8 unspent against a 22-point threshold.
    e.aging_points.insert(Characteristic::Sta, u8::MAX);
    assert_eq!(
        effective_characteristic_after_aging(&e, &rs, Characteristic::Sta),
        -22
    );
}

/// A Characteristic selection targeting `characteristic`, for the Great/Poor
/// tests below.
fn targeting(item: &str, characteristic: Characteristic) -> Selection {
    Selection::with_params(
        Id::new(item),
        BTreeMap::from([("characteristic".to_string(), characteristic.id())]),
    )
}

/// **The point-buy table is exactly the seven rows the rulebook prints.**
///
/// `ArMDE:2346-2354` prints +3→6 through -3→Gain 6 and stops. There is no
/// printed price for +4 or +5, and inventing one to "continue the triangular
/// progression" breaks `CLAUDE.md` → "Rules backed by source, never memory". The
/// absence is also the tell that the cap-shift reading of Great (Characteristic)
/// was wrong: a cap the player buys past needs a price the book never gives.
#[test]
fn characteristic_cost_table_prices_only_the_printed_rows() {
    let rs = load_ruleset();
    let chars = rs
        .characteristic_rules()
        .expect("characteristic rules present");
    for (score, cost) in [(3, 6), (2, 3), (1, 1), (0, 0), (-1, -1), (-2, -3), (-3, -6)] {
        assert_eq!(chars.cost_for(score), Some(cost), "printed row {score}");
    }
    for score in [4, 5, -4, -5] {
        assert_eq!(
            chars.cost_for(score),
            None,
            "score {score} has no printed cost in ArMDE:2346-2354 and must not be priced"
        );
    }
    assert_eq!(chars.max_score(), Some(3));
    assert_eq!(chars.min_score(), Some(-3));
}

/// **Great (Characteristic) grants the point; it does not unlock a purchase.**
///
/// `ArMDE:3989`: "You may **raise** any Characteristic that already has a score
/// of at least +3 **by one point**, to no more than +5." That is the grammar of
/// Giant Blood's "You also gain +1 to both Strength and Stamina"
/// (`ArMDE:3977`), already modelled as a free score delta. `ArMDE:4105` is
/// consistent: +3 is the cap on the **bought** score, and the Virtue carries the
/// character past it by granting the point.
#[test]
fn great_characteristic_grants_a_free_point() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![targeting(
            "virtue.great_characteristic",
            Characteristic::Str,
        )],
    );
    e.characteristics = BTreeMap::from([(Characteristic::Str, 3)]);
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Str),
        4,
        "one Great on a bought +3 must read +4"
    );
    // It targets only the Characteristic the selection names.
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Qik),
        0
    );
}

/// Taken twice for one Characteristic (`max_per_target: 2`), Great reaches the
/// "+5" the passage names — with no clamp of its own: +3 bought plus two granted
/// points *is* +5.
#[test]
fn great_characteristic_twice_reaches_plus_five() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![
            targeting("virtue.great_characteristic", Characteristic::Str),
            targeting("virtue.great_characteristic", Characteristic::Str),
        ],
    );
    e.characteristics = BTreeMap::from([(Characteristic::Str, 3)]);
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Str),
        5
    );
}

/// **Poor (Characteristic) lowers the score itself — the worse half of the
/// defect.** `ArMDE:6600`: "lower one which is already -3 or lower by one
/// point … You may take this Flaw twice for a single Characteristic, lowering it
/// to -5". Priced as a buy-floor shift, the invented -4 row refunded 10 points
/// where the table's own progression gives 6, so the Flaw paid the player twice:
/// once in Flaw points, once in Characteristic points.
#[test]
fn poor_characteristic_lowers_the_score_for_free() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![targeting("flaw.poor_characteristic", Characteristic::Str)],
    );
    e.characteristics = BTreeMap::from([(Characteristic::Str, -3)]);
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Str),
        -4
    );
}

/// **Giant Blood stacked on two Greats reaches +6, and nothing may forbid it.**
///
/// `ArMDE:3977` says so outright: "This bonus may raise your scores in those
/// Characteristics as high as +6." This is the case a naive "+5 ceiling" clamp
/// breaks, so it is pinned: bought +3, two Greats (+2) and Giant Blood (+1).
#[test]
fn giant_blood_over_two_greats_reaches_plus_six() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![
            targeting("virtue.great_characteristic", Characteristic::Str),
            targeting("virtue.great_characteristic", Characteristic::Str),
            Selection::new(Id::new("virtue.giant_blood")),
        ],
    );
    e.characteristics = BTreeMap::from([(Characteristic::Str, 3)]);
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Str),
        6
    );
    // Stamina gets Giant Blood's point alone.
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Sta),
        1
    );
}

/// A bought +4 — legal only under the invented table — is now reported as an
/// off-table score rather than silently priced at an invented 10 points.
#[test]
fn a_bought_score_above_plus_three_is_out_of_range() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![targeting(
            "virtue.great_characteristic",
            Characteristic::Str,
        )],
    );
    e.characteristics = BTreeMap::from([(Characteristic::Str, 4)]);
    let result = validate(&e, &rs);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == ValidationIssue::CODE_CHARACTERISTIC_OUT_OF_RANGE),
        "a bought +4 must be reported off-table; got {:#?}",
        result.issues
    );
}

#[test]
fn english_i18n_covers_all_abilities() {
    let rs = load_ruleset();
    let i18n_en = include_str!("../../../rules/i18n/en/abilities.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for ability in rs.abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "English i18n missing ability '{}'",
            ability.id
        );
    }
}

#[test]
fn german_i18n_covers_all_abilities() {
    let rs = load_ruleset();
    let i18n_de = include_str!("../../../rules/i18n/de/abilities.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for ability in rs.abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "German i18n missing ability '{}'",
            ability.id
        );
    }
}

// --- Round-1 audit, Sabine 12: the three uncovered catalogues ---------------
//
// Six catalogues already have an `*_i18n_covers_all_*` pair (abilities above,
// plus childhoods, spells, mastery abilities, equipment and items below), and
// `aging.json` is covered too, by name rather than by shape
// (`english_and_german_i18n_cover_all_living_conditions` /
// `..._crisis_rows`). `arts`, `houses` and `mythic_companion_types` had none.
//
// All three ship complete today — which is exactly why the gap was invisible.
// An id with no i18n entry does not fail to load: `display_name` returns `None`
// and the UI renders the raw slug, so deleting `art.ignem` from both locales
// left the whole suite green while the Arts grid printed `art.ignem`. Both
// locales are checked, because a key missing from BOTH is perfectly symmetrical
// and locale parity alone cannot see it.

#[test]
fn english_i18n_covers_all_arts() {
    let rs = load_full_ruleset();
    let loc = LocalizedRuleset::new(rs.clone(), include_str!("../../../rules/i18n/en/arts.json"))
        .unwrap();
    for art in rs.arts() {
        assert!(
            loc.display_name(&art.id).is_some(),
            "English i18n missing art '{}'",
            art.id
        );
    }
}

#[test]
fn german_i18n_covers_all_arts() {
    let rs = load_full_ruleset();
    let loc = LocalizedRuleset::new(rs.clone(), include_str!("../../../rules/i18n/de/arts.json"))
        .unwrap();
    for art in rs.arts() {
        assert!(
            loc.display_name(&art.id).is_some(),
            "German i18n missing art '{}'",
            art.id
        );
    }
}

#[test]
fn english_i18n_covers_all_houses() {
    let rs = load_full_ruleset();
    let loc = LocalizedRuleset::new(
        rs.clone(),
        include_str!("../../../rules/i18n/en/houses.json"),
    )
    .unwrap();
    for house in rs.houses() {
        assert!(
            loc.display_name(&house.id).is_some(),
            "English i18n missing house '{}'",
            house.id
        );
    }
}

#[test]
fn german_i18n_covers_all_houses() {
    let rs = load_full_ruleset();
    let loc = LocalizedRuleset::new(
        rs.clone(),
        include_str!("../../../rules/i18n/de/houses.json"),
    )
    .unwrap();
    for house in rs.houses() {
        assert!(
            loc.display_name(&house.id).is_some(),
            "German i18n missing house '{}'",
            house.id
        );
    }
}

#[test]
fn english_i18n_covers_all_mythic_companion_types() {
    let rs = load_full_ruleset();
    let loc = LocalizedRuleset::new(
        rs.clone(),
        include_str!("../../../rules/i18n/en/mythic_companion_types.json"),
    )
    .unwrap();
    for mythic_type in rs.mythic_types() {
        assert!(
            loc.display_name(&mythic_type.id).is_some(),
            "English i18n missing mythic companion type '{}'",
            mythic_type.id
        );
    }
}

#[test]
fn german_i18n_covers_all_mythic_companion_types() {
    let rs = load_full_ruleset();
    let loc = LocalizedRuleset::new(
        rs.clone(),
        include_str!("../../../rules/i18n/de/mythic_companion_types.json"),
    )
    .unwrap();
    for mythic_type in rs.mythic_types() {
        assert!(
            loc.display_name(&mythic_type.id).is_some(),
            "German i18n missing mythic companion type '{}'",
            mythic_type.id
        );
    }
}

/// The shipped Sample Childhood catalogue, read as text so the tests below can
/// check the *file's* own canonical order as well as what the engine parses out
/// of it.
const SHIPPED_CHILDHOODS: &str = include_str!("../../../rules/core/childhoods.json");

/// The shipped equipment catalogue and its two i18n counterparts, read as text so
/// `shipped_equipment_files_are_canonically_id_ordered` can check each file's own
/// on-disk order (`weapon.staff` sat out of canonical order in all three files —
/// full-audit finding V42/G23 — until this guard was added).
const SHIPPED_EQUIPMENT_CORE: &str = include_str!("../../../rules/core/equipment.json");
const SHIPPED_EQUIPMENT_I18N_EN: &str = include_str!("../../../rules/i18n/en/equipment.json");
const SHIPPED_EQUIPMENT_I18N_DE: &str = include_str!("../../../rules/i18n/de/equipment.json");

/// Extracts the top-level object keys of a flat, one-entry-per-line JSON file
/// (the shape every `rules/i18n/<lang>/*.json` file uses), in on-disk order.
/// `serde_json::Value` cannot answer this: this crate does not enable
/// `serde_json`'s `preserve_order` feature, so a parsed `Value::Object` is
/// backed by a `BTreeMap` and always reports keys pre-sorted regardless of the
/// file's real byte order — exactly the drift this check exists to catch.
fn top_level_keys_in_file_order(json_text: &str) -> Vec<&str> {
    json_text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed.strip_prefix('"')?;
            let end = rest.find('"')?;
            Some(&rest[..end])
        })
        .collect()
}

/// Every shipped Sample Childhood package spends exactly the two childhood blocks
/// it is a shortcut for: 45 experience points across the spread and 75 in the
/// native language.
///
/// Source: ArMDE:2378 (the two blocks),
/// `ArMDE:2384-2388` (the five packages), priced off the "ABILITY To Buy" column at
/// `ArMDE:2406-2427`.
///
/// The two figures are deliberately **literals**. `Ruleset::validate_childhood_packages`
/// already prices every package at load, but against `rules/core/life_stages.json`
/// — so an edit that lowered the block and a package together would pass the load
/// in silence. These literals are the outside witness that keeps the transcription
/// honest: they come from the rulebook line, not from another JSON file.
#[test]
fn every_shipped_childhood_package_prices_to_45_and_75() {
    let rs = load_full_ruleset();
    assert!(
        rs.childhoods().next().is_some(),
        "the shipped ruleset must offer Sample Childhood packages"
    );
    for package in rs.childhoods() {
        assert_eq!(
            package.spread_xp(rs.advancement()),
            Some(45),
            "package '{}' must spread exactly 45 experience points",
            package.id
        );
        assert_eq!(
            package.native_xp(rs.advancement()),
            Some(75),
            "package '{}' must spend exactly 75 on its native language",
            package.id
        );
    }
}

/// The shipped apprenticeship block carries the rulebook's own numbers.
///
/// > The fifteen years of apprenticeship give the character 240 experience points
/// > … Magi must have the following minimum Abilities: Parma Magica 1, Magic
/// > Theory 1, Latin 1.
///
/// Source: ArMDE:2435 (the years and the
/// experience), `ArMDE:2437` (the three minimums), `ArMDE:2451-2461` (the four recommended
/// Abilities and their "Total Cost: 90 experience points").
///
/// Every figure is a **literal off the rulebook line**, for the same reason the
/// childhood packages are priced with literals above: the load-time check prices the
/// recommended list against `recommended_xp` in the same file, so an edit that moved
/// both together would pass in silence. These literals are the outside witness.
#[test]
fn shipped_apprenticeship_carries_the_2435_and_2437_numbers() {
    let rs = load_full_ruleset();
    let apprenticeship = rs
        .life_stages()
        .and_then(|rules| rules.apprenticeship.as_ref())
        .expect("the shipped life stages declare an apprenticeship");

    assert_eq!(
        apprenticeship.years, 15,
        "\"The fifteen years\" (ArMDE:2435)"
    );
    assert_eq!(
        apprenticeship.xp, 240,
        "\"240 experience points\" (ArMDE:2435)"
    );

    // "Parma Magica 1, Magic Theory 1, Latin 1" (ArMDE:2437). Latin is one instance of
    // the parameterized dead-language Ability, matched by id (see RULES.md), so no
    // requirement names a parameter.
    let stated: Vec<(&str, u8, Option<&str>)> = apprenticeship
        .minimum_abilities
        .iter()
        .map(|r| (r.ability.as_str(), r.min_score, r.parameter.as_deref()))
        .collect();
    assert_eq!(
        stated,
        vec![
            ("ability.dead_language", 1, None),
            ("ability.magic_theory", 1, None),
            ("ability.parma_magica", 1, None),
        ]
    );

    // "Artes Liberales 1 / Latin 4 / Magic Theory 3 / Parma Magica 1" (ArMDE:2453-2459).
    let recommended: Vec<(&str, u8, Option<&str>)> = apprenticeship
        .recommended_abilities
        .iter()
        .map(|r| (r.ability.as_str(), r.min_score, r.parameter.as_deref()))
        .collect();
    assert_eq!(
        recommended,
        vec![
            ("ability.artes_liberales", 1, None),
            ("ability.dead_language", 4, None),
            ("ability.magic_theory", 3, None),
            ("ability.parma_magica", 1, None),
        ]
    );
    assert_eq!(
        apprenticeship.recommended_xp, 90,
        "\"Total Cost: 90 experience points\" (ArMDE:2461)"
    );

    // The baseline a plan naming no Gauntlet age is read at: "These templates are of
    // a stereotypical member of each House, 25 years old and just out of
    // apprenticeship" (ArMDE:1601). A literal for the same reason the rest are — the only
    // outside witness that the shipped number is the rulebook's.
    assert_eq!(
        apprenticeship.default_gauntlet_age,
        Some(25),
        "\"25 years old and just out of apprenticeship\" (ArMDE:1601)"
    );
}

/// Eleven Virtues that read like sourcebook material — angelic/demonic heritage,
/// Mythic Companion gateways, spirit pacts — are in fact printed in the **core
/// rules**, in the Virtues and Flaws chapter. English core is the source of truth,
/// so their `source` must cite the core-rules file; citing a *Realms of Power*
/// volume for an entry that has a real core-rules heading is simply wrong
/// provenance, and nothing else in the suite would notice (the independent
/// bracket check in `rules_source_provenance.rs` only proves the cited range holds
/// *some* content in the file it names).
///
/// Note that `Curse-Throwing` appears twice in the core rules: the Virtue at
/// `ArMDE:3625` (*Major, Supernatural*, "confers the Supernatural Ability
/// Curse-Throwing 1") and the Supernatural **Ability** at `ArMDE:7396`, which is what
/// `ability.curse_throwing` already cites. `virtue.curse_throwing` is the former.
///
/// Source: ArMDE:3504, 3625, 3649, 3663,
/// 3667, 3671, 3821, 4594, 5006, 5010, 5022.
#[test]
fn core_rules_virtues_cite_the_core_rules_file() {
    let rs = load_full_ruleset();

    let expected = [
        ("virtue.blood_of_the_nephilim", 3504, 3518),
        ("virtue.curse_throwing", 3625, 3628),
        ("virtue.demonic_blood", 3649, 3662),
        ("virtue.demonic_might", 3663, 3666),
        ("virtue.demonic_powers", 3667, 3670),
        ("virtue.devil_child", 3671, 3674),
        ("virtue.faerie_doctor", 3821, 3824),
        ("virtue.nephilim", 4594, 4597),
        ("virtue.spirit_votary", 5006, 5009),
        ("virtue.spiritual_pact", 5010, 5021),
        ("virtue.strong_angelic_heritage", 5022, 5031),
    ];

    for (id, start, end) in expected {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} ships"));
        let source = item
            .source
            .as_ref()
            .unwrap_or_else(|| panic!("{id} carries provenance"));
        assert_eq!(
            source.file, "Ars Magica - Definitive Edition (Core Rules).md",
            "{id} is printed in the core rules, not a sourcebook"
        );
        assert_eq!(
            (source.lines.start, source.lines.end),
            (start, end),
            "{id} brackets its core-rules entry"
        );
    }
}

/// Items whose core-rules type descriptor lists `Tainted` must carry
/// `"tainted": true`, because that flag is what feeds the half-of-taken-points
/// Tainted cap in `validate_tainted_cap`. A missing flag makes the cap
/// under-count and silently lets a character keep more Tainted points than the
/// rules allow — wrong rules output, invisible to every other check.
///
/// Sampled structurally, never as a total (the catalogue may grow): a few
/// descriptor lines that do carry the tag, plus `flaw.tainted_with_evil` as the
/// control — its *name* contains "Tainted" but its descriptor is
/// `*Minor, General*` (ArMDE:6844), so it must NOT be flagged.
///
/// Source: ArMDE:2998-3000 (the cap),
/// and the descriptor lines :3650 (Demonic Blood, "*Major, Supernatural,
/// Tainted*"), :6856 (Tragic Life, "*Major, Story, Tainted*"), :3411, :3427,
/// :6840, :6844.
#[test]
fn core_rules_tainted_virtues_carry_the_tainted_flag() {
    let rs = load_full_ruleset();

    let expected = [
        ("virtue.demonic_blood", true),
        ("virtue.amorphous_major", true),
        ("virtue.aptitude_for_sin", true),
        ("flaw.tragic_life", true),
        ("flaw.tainted_offspring", true),
        ("flaw.tainted_with_evil", false),
    ];

    for (id, tainted) in expected {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} ships"));
        assert_eq!(
            item.tainted, tainted,
            "{id} must have tainted = {tainted}, matching its core-rules type descriptor"
        );
    }
}

/// `virtue.good_teacher` (ArMDE:3971-3974) grants two distinct Quality bonuses:
/// "Add three to the Quality of any books that you write, and five to the
/// Source Quality for anyone who studies with you." The `+3` half is an
/// **authoring** effect — it benefits the reader of a book *this character
/// wrote*, not this character reading someone else's book — so it must use
/// `AdvancementSource::Authoring`, never `AdvancementSource::Book` (the
/// reader-side shape `virtue.book_learner` and `virtue.study_bonus` correctly
/// use). V/F audit F-91 / Q-32: the shipped data used `book` for both rows,
/// pointing the authoring half at the wrong person.
#[test]
fn good_teacher_authoring_bonus_uses_the_authoring_source_not_book() {
    let rs = load_full_ruleset();

    let item = rs
        .item(&Id::new("virtue.good_teacher"))
        .expect("virtue.good_teacher ships");

    assert!(
        item.effects.contains(&Effect::AdvancementMod {
            source: AdvancementSource::Teaching,
            amount: Some(5),
            factor: None,
        }),
        "the Source Quality bonus for being studied with is unchanged: +5 teaching"
    );
    assert!(
        item.effects.contains(&Effect::AdvancementMod {
            source: AdvancementSource::Authoring,
            amount: Some(3),
            factor: None,
        }),
        "the Quality-of-authored-books bonus must be +3 authoring, not +3 book"
    );
    assert!(
        !item.effects.contains(&Effect::AdvancementMod {
            source: AdvancementSource::Book,
            amount: Some(3),
            factor: None,
        }),
        "the old mis-scoped book row must be gone"
    );
}

/// D55 (Q6): `flaw.incomprehensible` (ArMDE:6294-6297) halves along BOTH axes
/// its passage names — "Anyone trying to learn from you **or from a book you
/// have written** must halve their Advancement Total" — so it needs a
/// `teaching` row (being taught by this character) AND an `authoring` row
/// (studying from a book this character wrote), each carrying
/// `AdvancementFactor::Half`, never the old `amount: 0` marker.
/// `flaw.loose_magic` (ArMDE:6354-6357, "Your Advancement Total is halved
/// whenever you try to Master spells") gets the same factor on its one row.
#[test]
fn incomprehensible_and_loose_magic_carry_a_halving_factor_not_a_zero_amount() {
    let rs = load_full_ruleset();

    let incomprehensible = rs
        .item(&Id::new("flaw.incomprehensible"))
        .expect("flaw.incomprehensible ships");
    assert!(
        incomprehensible.effects.contains(&Effect::AdvancementMod {
            source: AdvancementSource::Teaching,
            amount: None,
            factor: Some(AdvancementFactor::Half),
        }),
        "the teaching axis must halve via a factor, not amount: 0"
    );
    assert!(
        incomprehensible.effects.contains(&Effect::AdvancementMod {
            source: AdvancementSource::Authoring,
            amount: None,
            factor: Some(AdvancementFactor::Half),
        }),
        "the authoring axis (\"or from a book you have written\") must be its own row"
    );

    let loose_magic = rs
        .item(&Id::new("flaw.loose_magic"))
        .expect("flaw.loose_magic ships");
    assert!(
        loose_magic.effects.contains(&Effect::AdvancementMod {
            source: AdvancementSource::SpellMastery,
            amount: None,
            factor: Some(AdvancementFactor::Half),
        }),
        "Loose Magic's Spell Mastery halving must be a factor, not amount: 0"
    );
}

/// The four Outer-Mystery Virtues whose descriptors state that taking them makes
/// the character a member of a particular House must carry that House as their
/// prerequisite, so a magus of another House cannot simply buy one.
///
/// > You have been initiated into the Outer Mystery of the Heartbeast (see page
/// > 233), and thus are a member of House Bjornaer.
///
/// Source: ArMDE:4059-4061 (Heartbeast
/// → Bjornaer), `ArMDE:3761` (The Enigma → Criamon), `ArMDE:3827` (Faerie Magic →
/// Merinita), `ArMDE:5217` (Verditius Magic → Verditius). No other free-Virtue
/// descriptor in the core rules makes that claim, so `virtue.hermetic_prestige`
/// (Guernicus) is the control: its House grants it, but the Virtue itself says
/// nothing about membership and must stay House-free.
#[test]
fn outer_mystery_virtues_require_the_house_their_descriptor_confers() {
    let rs = load_full_ruleset();

    let expected = [
        ("virtue.heartbeast", "house.bjornaer"),
        ("virtue.the_enigma", "house.criamon"),
        ("virtue.faerie_magic", "house.merinita"),
        ("virtue.verditius_magic", "house.verditius"),
    ];

    for (virtue, house) in expected {
        let item = rs
            .item(&Id::new(virtue))
            .unwrap_or_else(|| panic!("{virtue} ships"));
        assert_eq!(
            item.prerequisites,
            Some(Prereq::House(Id::new(house))),
            "{virtue} states it makes you a member of {house}, so it must require that House"
        );
    }

    let control = rs
        .item(&Id::new("virtue.hermetic_prestige"))
        .expect("virtue.hermetic_prestige ships");
    assert_eq!(
        control.prerequisites, None,
        "a free Virtue whose descriptor claims no House membership must stay House-free"
    );
}

/// Every issue code the shipped ruleset raises against `virtue.heartbeast`.
/// Scoped by the issue's `context`, so a build's unrelated findings (points
/// balance, missing Abilities, …) never mask or fake the prerequisite result.
fn heartbeast_issue_codes(rs: &Ruleset, e: &Entity) -> Vec<String> {
    validate(e, rs)
        .issues
        .iter()
        .filter(|i| i.context.as_ref() == Some(&Id::new("virtue.heartbeast")))
        .map(|i| i.code.clone())
        .collect()
}

/// A Bjornaer magus's *granted* Heartbeast stays legal once Heartbeast requires
/// House Bjornaer: `validate_prerequisites` walks bought `entity.selections`
/// only, so a granted row is never prereq-checked — and the House's own fixed
/// grant satisfies the prerequisite in any case. Pinned because the House-prereq
/// data would be actively harmful if it fired on the grant that House makes.
///
/// Source: ArMDE:4061 ("all Bjornaer
/// magi gain this Virtue for free at character creation").
#[test]
fn a_bjornaer_magus_keeps_the_house_granted_heartbeast() {
    let rs = load_full_ruleset();
    let mut e = entity("magus", vec![]);
    e.house = Some(Id::new("house.bjornaer"));

    assert!(
        heartbeast_issue_codes(&rs, &e).is_empty(),
        "Bjornaer's own granted Heartbeast must raise nothing: {:?}",
        heartbeast_issue_codes(&rs, &e)
    );
}

/// A magus who has not chosen a House yet gets the *unevaluated* warning, not an
/// error: `Prereq::House` against an absent house is genuinely Unknown, and the
/// House step may simply come later. Pinned so the new data cannot turn an
/// in-progress build into a blocking failure.
#[test]
fn heartbeast_without_a_house_warns_rather_than_failing() {
    let rs = load_full_ruleset();
    let e = entity("magus", vec![Selection::new(Id::new("virtue.heartbeast"))]);
    assert!(e.house.is_none(), "the fixture must set no House");

    let codes = heartbeast_issue_codes(&rs, &e);
    assert!(
        codes.contains(&"prereq_unevaluated".to_string()),
        "an unset House leaves the prerequisite undecided: {codes:?}"
    );
    assert!(
        !codes.contains(&"prereq_not_met".to_string()),
        "an unset House must not be reported as a failed prerequisite: {codes:?}"
    );
}

/// The finding itself: a magus of another House who *buys* Heartbeast is now an
/// error naming the item, where before nothing at all was raised.
#[test]
fn a_bonisagus_magus_cannot_buy_heartbeast() {
    let rs = load_full_ruleset();
    let mut e = entity("magus", vec![Selection::new(Id::new("virtue.heartbeast"))]);
    e.house = Some(Id::new("house.bonisagus"));

    let codes = heartbeast_issue_codes(&rs, &e);
    assert!(
        codes.contains(&"prereq_not_met".to_string()),
        "Heartbeast makes you a Bjornaer, so a Bonisagus may not buy it: {codes:?}"
    );
}

/// The open-grant half of the same rule. Jerbiton's free Minor Virtue and Ex
/// Miscellanea's free Minor Hermetic Virtue are *open* menus, and all four
/// Outer-Mystery Virtues are Minor and Hermetic — so before this change both
/// menus offered them, and a pick made through a grant is never prereq-checked,
/// leaving a Jerbiton with a Heartbeast and no complaint at all. The open-pick
/// constraint now also refuses an item whose House prerequisite the character's
/// own House contradicts.
#[test]
fn a_jerbiton_magus_cannot_take_heartbeast_as_the_free_minor_virtue() {
    let rs = load_full_ruleset();
    let mut e = entity("magus", vec![]);
    e.house = Some(Id::new("house.jerbiton"));
    e.house_choices = BTreeMap::from([(
        "jerbiton_minor_virtue".to_string(),
        Selection::new(Id::new("virtue.heartbeast")),
    )]);

    let codes: Vec<String> = validate(&e, &rs)
        .issues
        .iter()
        .filter(|i| i.context.as_ref() == Some(&Id::new("virtue.heartbeast")))
        .map(|i| i.code.clone())
        .collect();
    assert!(
        codes.contains(&"house_grant_constraint".to_string()),
        "an open House grant must not admit a Virtue that confers a different House: {codes:?}"
    );
}

/// The same open menu must still admit a Virtue that names no House at all —
/// the filter is on House prerequisites, not on prerequisites in general.
#[test]
fn a_jerbiton_magus_may_still_take_an_ordinary_free_minor_virtue() {
    let rs = load_full_ruleset();
    let mut e = entity("magus", vec![]);
    e.house = Some(Id::new("house.jerbiton"));
    e.house_choices = BTreeMap::from([(
        "jerbiton_minor_virtue".to_string(),
        Selection::new(Id::new("virtue.self_confident")),
    )]);

    let codes: Vec<String> = validate(&e, &rs)
        .issues
        .iter()
        .map(|i| i.code.clone())
        .collect();
    assert!(
        !codes.contains(&"house_grant_constraint".to_string()),
        "a House-free Minor Virtue must stay eligible for the open grant: {codes:?}"
    );
}

/// Two known packages load with their entries and provenance intact — never a
/// package total, which is data (a ruleset may ship any number of packages).
/// Athletic is the plain shape, Traveling the one that exercises every feature at
/// once: two instances of one parameterized Ability and a spread language beside
/// the native one.
///
/// Source: ArMDE:2384 (Athletic),
/// `ArMDE:2388` (Traveling).
#[test]
fn known_childhood_packages_ship_with_their_entries_and_provenance() {
    let rs = load_full_ruleset();

    let athletic = rs
        .childhood(&Id::new("childhood.athletic"))
        .expect("childhood.athletic ships");
    let entries: Vec<(&str, Option<&str>, u8, bool)> = athletic
        .entries
        .iter()
        .map(|e| (e.ability.as_str(), e.slot.as_deref(), e.score, e.native))
        .collect();
    assert_eq!(
        entries,
        vec![
            ("ability.athletics", None, 2, false),
            ("ability.brawl", None, 2, false),
            ("ability.living_language", None, 5, true),
            ("ability.swim", None, 2, false),
        ],
        "Athletics 2, Brawl 2, Native Language 5, Swim 2 (ArMDE:2384)"
    );
    let source = athletic
        .source
        .as_ref()
        .expect("Athletic carries provenance");
    assert_eq!(
        source.file,
        "Ars Magica - Definitive Edition (Core Rules).md"
    );
    assert_eq!((source.lines.start, source.lines.end), (2384, 2384));

    let traveling = rs
        .childhood(&Id::new("childhood.traveling"))
        .expect("childhood.traveling ships");
    let slots: Vec<(&str, &str)> = traveling
        .slots()
        .map(|(slot, ability)| (slot, ability.as_str()))
        .collect();
    assert_eq!(
        slots,
        vec![
            ("area_a", "ability.area_lore"),
            ("area_b", "ability.area_lore"),
            ("language", "ability.living_language"),
        ],
        "Area A Lore, Area B Lore and the spread Living Language are asked for \
         (ArMDE:2388)"
    );
    // The native language is never a slot: it is chosen once per character.
    let native = traveling.native_entry().expect("a native-language entry");
    assert_eq!(native.ability, Id::new("ability.living_language"));
    assert_eq!(native.score, 5);
    assert!(native.slot.is_none());
    assert_eq!(
        traveling
            .source
            .as_ref()
            .map(|s| (s.lines.start, s.lines.end)),
        Some((2388, 2388))
    );
}

/// The shipped file is canonically ordered: packages by id, each package's entries
/// by `(ability, slot)`.
///
/// This has to be checked here because nothing else can. `ChildhoodEntry` order is
/// deliberately *preserved* on load — it is the order a UI asks for slot values in
/// — so a mis-sorted file parses and validates perfectly happily, and the only
/// symptom would be a noisy git diff the next time the catalogue is regenerated.
#[test]
fn the_shipped_childhoods_file_is_canonically_ordered() {
    let file: serde_json::Value =
        serde_json::from_str(SHIPPED_CHILDHOODS).expect("the shipped catalogue is valid JSON");
    let packages = file["packages"]
        .as_array()
        .expect("the file carries a package list");

    let ids: Vec<&str> = packages
        .iter()
        .map(|p| p["id"].as_str().expect("every package has an id"))
        .collect();
    let mut id_sorted = ids.clone();
    id_sorted.sort_unstable();
    assert_eq!(ids, id_sorted, "packages must be id-sorted");

    for package in packages {
        // An absent slot sorts before any present one, which is what puts the
        // native-language entry ahead of Traveling's slotted second language.
        let keys: Vec<(&str, &str)> = package["entries"]
            .as_array()
            .expect("every package has entries")
            .iter()
            .map(|e| {
                (
                    e["ability"].as_str().expect("every entry names an ability"),
                    e["slot"].as_str().unwrap_or(""),
                )
            })
            .collect();
        let mut key_sorted = keys.clone();
        key_sorted.sort_unstable();
        assert_eq!(
            keys, key_sorted,
            "entries of '{}' must be sorted by (ability, slot)",
            package["id"]
        );
    }
}

#[test]
fn english_i18n_covers_all_childhood_packages() {
    let rs = load_full_ruleset();
    let i18n_en = include_str!("../../../rules/i18n/en/childhoods.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for package in rs.childhoods() {
        assert!(
            loc.display_name(&package.id).is_some(),
            "English i18n missing childhood package '{}'",
            package.id
        );
    }
}

#[test]
fn german_i18n_covers_all_childhood_packages() {
    let rs = load_full_ruleset();
    let i18n_de = include_str!("../../../rules/i18n/de/childhoods.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for package in rs.childhoods() {
        assert!(
            loc.display_name(&package.id).is_some(),
            "German i18n missing childhood package '{}'",
            package.id
        );
    }
}

#[test]
fn english_i18n_covers_all_spells() {
    let rs = load_ruleset_with_spells();
    let i18n_en = include_str!("../../../rules/i18n/en/spells.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for spell in rs.spells() {
        // A spell tooltip renders the description, so a missing English
        // description leaves an empty tooltip — assert both name and description
        // cover every spell id (fallback-free: this is the raw en file).
        assert!(
            loc.display_name(&spell.id).is_some(),
            "English i18n missing spell name '{}'",
            spell.id
        );
        assert!(
            loc.description(&spell.id).is_some(),
            "English i18n missing spell description '{}'",
            spell.id
        );
    }
}

#[test]
fn german_i18n_covers_all_spells() {
    let rs = load_ruleset_with_spells();
    let i18n_de = include_str!("../../../rules/i18n/de/spells.json");
    // Raw German file (fallback-free `new`): a missing German description here is
    // a real gap. The app layer falls back to English at load, but this gate
    // asserts the shipped German data itself covers every spell — the check that
    // was name-only before and let empty German tooltips ship.
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for spell in rs.spells() {
        assert!(
            loc.display_name(&spell.id).is_some(),
            "German i18n missing spell name '{}'",
            spell.id
        );
        assert!(
            loc.description(&spell.id).is_some(),
            "German i18n missing spell description '{}'",
            spell.id
        );
    }
}

/// The four meta-magic Vim spells whose target `(Form)` is a selection each
/// declare a single `form`-domain parameter, keep their catalogue Vim
/// Technique/Form (the parameter is display + identity only), and the whole
/// shipped catalogue still passes load-time referential integrity.
/// Source: ArMDE:15776-15779,
/// :15791-15794, :15801-15804, :15843-15846.
#[test]
fn parametrized_vim_spells_declare_a_form_parameter() {
    let rs = load_ruleset_with_spells();
    for id in [
        "spell.mirror_of_opposition_form",
        "spell.unravelling_the_fabric_of_form",
        "spell.wizards_boost_form",
        "spell.wizards_reach_form",
    ] {
        let spell = rs
            .spell(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} present"));
        assert_eq!(spell.parameters.len(), 1, "{id} has one parameter");
        let def = &spell.parameters[0];
        assert_eq!(def.key, "form", "{id} parameter key is 'form'");
        assert_eq!(def.domain, ParameterDomain::Form, "{id} domain is Form");
        // The spell's own Technique/Form stay the catalogue Vim Arts.
        assert_eq!(spell.form, Id::new("art.vim"), "{id} form stays Vim");
    }
}

#[test]
fn shipped_equipment_loads_and_exposes_accessors() {
    let rs = load_ruleset_with_equipment();
    // Catalogue size is data, not code: assert representative items are present
    // and read correctly, never exact totals.
    let sword = rs
        .weapon(&Id::new("weapon.sword_long"))
        .expect("weapon.sword_long present");
    assert_eq!(sword.attack_mod, Some(4));
    assert_eq!(sword.damage_mod, Some(6));
    assert_eq!(sword.ability, Id::new("ability.single_weapon"));
    assert!(sword.range.is_none(), "melee weapon has no range");
    // A missile weapon carries a Range and uses Bows.
    let bow = rs
        .weapon(&Id::new("weapon.bow_long"))
        .expect("weapon.bow_long present");
    assert_eq!(bow.range, Some(30));
    assert_eq!(bow.ability, Id::new("ability.bows"));
    // Dodge has no attack/damage/min-Strength (n/a cells).
    let dodge = rs.weapon(&Id::new("weapon.dodge")).expect("dodge present");
    assert_eq!(dodge.attack_mod, None);
    assert_eq!(dodge.min_strength, None);
    // Shields and armor resolve through their own accessors.
    assert_eq!(rs.shield(&Id::new("shield.heater")).unwrap().defense_mod, 3);
    assert_eq!(
        rs.armor_item(&Id::new("armor.chain_mail_full"))
            .unwrap()
            .protection,
        9
    );
    assert!(rs.weapon_count() > 0 && rs.shield_count() > 0 && rs.armor_count() > 0);
}

/// A weapon whose combat `ability` names something that is neither Martial nor
/// Brawl is rejected at load (referential-integrity trust gate).
#[test]
fn weapon_with_non_combat_ability_rejected_at_load() {
    let equipment = r#"{ "weapons": [
      { "id": "weapon.bad", "kind": "melee", "init_mod": 0, "defense_mod": 0,
        "load": 1, "ability": "ability.awareness" }
    ] }"#;
    let err = Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: None,
        houses: Some(SHIPPED_HOUSES),
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: None,
        equipment: Some(equipment),
        characteristics: None,
        life_stages: None,
        childhoods: None,
        aging: None,
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("not a combat Ability"),
        "expected combat-ability rejection, got: {err}"
    );
}

/// A weapon naming a wholly unknown ability id is also rejected at load.
#[test]
fn weapon_with_unknown_ability_rejected_at_load() {
    let equipment = r#"{ "weapons": [
      { "id": "weapon.bad", "kind": "melee", "init_mod": 0, "defense_mod": 0,
        "load": 1, "ability": "ability.nonexistent" }
    ] }"#;
    let err = Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: None,
        houses: Some(SHIPPED_HOUSES),
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: None,
        equipment: Some(equipment),
        characteristics: None,
        life_stages: None,
        childhoods: None,
        aging: None,
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("unknown ability"),
        "expected unknown-ability rejection, got: {err}"
    );
}

#[test]
fn english_i18n_covers_all_mastery_abilities() {
    let rs = load_ruleset_with_mastery_abilities();
    let i18n_en = include_str!("../../../rules/i18n/en/spell_mastery_abilities.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for ability in rs.spell_mastery_abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "English i18n missing mastery-ability name '{}'",
            ability.id
        );
        assert!(
            loc.description(&ability.id).is_some(),
            "English i18n missing mastery-ability description '{}'",
            ability.id
        );
    }
}

#[test]
fn german_i18n_covers_all_mastery_abilities() {
    let rs = load_ruleset_with_mastery_abilities();
    let i18n_de = include_str!("../../../rules/i18n/de/spell_mastery_abilities.json");
    // Raw German file (fallback-free `new`): a missing German entry here is a real
    // gap, mirroring the spell coverage gate.
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for ability in rs.spell_mastery_abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "German i18n missing mastery-ability name '{}'",
            ability.id
        );
        assert!(
            loc.description(&ability.id).is_some(),
            "German i18n missing mastery-ability description '{}'",
            ability.id
        );
    }
}

#[test]
fn english_i18n_covers_all_equipment() {
    let rs = load_ruleset_with_equipment();
    let i18n_en = include_str!("../../../rules/i18n/en/equipment.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for id in rs
        .weapons()
        .map(|w| &w.id)
        .chain(rs.shields().map(|s| &s.id))
        .chain(rs.armor().map(|a| &a.id))
    {
        assert!(
            loc.display_name(id).is_some(),
            "English i18n missing equipment '{id}'"
        );
    }
}

#[test]
fn german_i18n_covers_all_equipment() {
    let rs = load_ruleset_with_equipment();
    let i18n_de = include_str!("../../../rules/i18n/de/equipment.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for id in rs
        .weapons()
        .map(|w| &w.id)
        .chain(rs.shields().map(|s| &s.id))
        .chain(rs.armor().map(|a| &a.id))
    {
        assert!(
            loc.display_name(id).is_some(),
            "German i18n missing equipment '{id}'"
        );
    }
}

/// The shipped equipment catalogue's core file and both i18n counterparts must
/// each list their ids in canonical (ascending, byte-order) order — CLAUDE.md's
/// "canonical serialization" rule. This never hardcodes a catalogue size or item
/// count: it re-derives the expected order from the ids actually present, so a
/// future addition to the catalogue needs no test change.
#[test]
fn shipped_equipment_files_are_canonically_id_ordered() {
    let core: serde_json::Value =
        serde_json::from_str(SHIPPED_EQUIPMENT_CORE).expect("core equipment.json is valid JSON");
    for array_key in ["weapons", "shields", "armor"] {
        let ids: Vec<&str> = core[array_key]
            .as_array()
            .unwrap_or_else(|| panic!("equipment.json carries a '{array_key}' array"))
            .iter()
            .map(|item| item["id"].as_str().expect("every entry has an id"))
            .collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(
            ids, sorted,
            "core equipment.json's '{array_key}' must be id-sorted"
        );
    }

    for (lang, file) in [
        ("en", SHIPPED_EQUIPMENT_I18N_EN),
        ("de", SHIPPED_EQUIPMENT_I18N_DE),
    ] {
        let ids = top_level_keys_in_file_order(file);
        assert!(!ids.is_empty(), "i18n/{lang}/equipment.json carries keys");
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "i18n/{lang}/equipment.json must be id-sorted");
    }
}

/// An [`EquipmentSlot`] referencing an unknown catalogue id errors in validate;
/// an over-heavy equipped weapon warns (advisory, non-blocking); normalize sorts.
#[test]
fn validate_equipment_unknown_ref_and_min_strength() {
    let rs = load_ruleset_with_equipment();
    let mut e = entity("grog", vec![]);
    // Unknown id → error.
    e.equipment = vec![EquipmentSlot {
        item: Id::new("weapon.nonexistent"),
        equipped: true,
        specialization_applies: false,
    }];
    let result = validate(&e, &rs);
    assert!(
        result.issues.iter().any(|i| i.code == "unknown_equipment"),
        "unknown equipment id must error"
    );

    // A Warhammer (min-Strength +2) equipped by a Strength −1 grog warns, not errors.
    e.characteristics.insert(Characteristic::Str, -1);
    e.equipment = vec![EquipmentSlot {
        item: Id::new("weapon.warhammer"),
        equipped: true,
        specialization_applies: false,
    }];
    let result = validate(&e, &rs);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "equipment_min_strength"
                && i.severity == arm_rules::IssueSeverity::Warning),
        "over-heavy equipped weapon must warn"
    );
    assert!(
        !result.issues.iter().any(|i| i.code == "unknown_equipment"),
        "a known weapon must not report unknown_equipment"
    );
}

/// Exercises the shield and armor arms of `validate_equipment`. An equipped shield
/// whose min-Strength exceeds the wielder's Strength raises the same advisory
/// `equipment_min_strength` warning as a weapon (shield arm), while armor — which
/// carries no min-Strength requirement — never warns however weak the wearer (armor
/// arm's `None`). ArMDE:16997.
#[test]
fn validate_equipment_shield_warns_and_armor_never_warns() {
    let rs = load_ruleset_with_equipment();
    let mut e = entity("grog", vec![]);

    // A Heater shield (min-Strength 0) equipped by a Strength −1 grog warns.
    e.characteristics.insert(Characteristic::Str, -1);
    e.equipment = vec![EquipmentSlot {
        item: Id::new("shield.heater"),
        equipped: true,
        specialization_applies: false,
    }];
    let result = validate(&e, &rs);
    let warnings: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code == "equipment_min_strength")
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "exactly one shield min-Strength advisory"
    );
    let warning = warnings[0];
    assert_eq!(warning.severity, arm_rules::IssueSeverity::Warning);
    assert_eq!(
        warning.args.get("item").map(String::as_str),
        Some("shield.heater")
    );
    assert_eq!(warning.args.get("required").map(String::as_str), Some("0"));
    assert_eq!(warning.args.get("strength").map(String::as_str), Some("-1"));
    assert_eq!(warning.context.as_ref(), Some(&Id::new("shield.heater")));

    // Full chain mail equipped by a much weaker grog never warns: armor has no
    // min-Strength requirement (armor arm returns `None`), so no advisory is raised
    // and a known armor id must not be reported as unknown.
    e.characteristics.insert(Characteristic::Str, -5);
    e.equipment = vec![EquipmentSlot {
        item: Id::new("armor.chain_mail_full"),
        equipped: true,
        specialization_applies: false,
    }];
    let result = validate(&e, &rs);
    assert!(
        !result
            .issues
            .iter()
            .any(|i| i.code == "equipment_min_strength"),
        "armor carries no min-Strength requirement, so it must never warn"
    );
    assert!(
        !result.issues.iter().any(|i| i.code == "unknown_equipment"),
        "a known armor id must not report unknown_equipment"
    );
}

/// Issue B: equipping a shield alongside ONLY two-handed weapon(s) raises the
/// advisory `shield_with_two_handed_weapon` warning (the shield's modifiers are
/// dropped, which looks like a bug otherwise). A one-handed weapon in the mix
/// clears the advisory, since the shield is usable with it. Non-blocking.
/// ArMDE:7494, :17063, :16975.
#[test]
fn shield_with_only_two_handed_weapons_warns() {
    let rs = load_ruleset_with_equipment();
    let mut e = entity("grog", vec![]);
    e.characteristics.insert(Characteristic::Str, 3);

    // A great sword (two-handed) + a shield → advisory warning.
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("weapon.sword_great"),
            equipped: true,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("shield.heater"),
            equipped: true,
            specialization_applies: false,
        },
    ];
    let result = validate(&e, &rs);
    let warning = result
        .issues
        .iter()
        .find(|i| i.code == "shield_with_two_handed_weapon")
        .expect("shield + two-handed-only must warn");
    assert_eq!(warning.severity, arm_rules::IssueSeverity::Warning);

    // Add a one-handed weapon: the shield is now usable, so the advisory clears.
    e.equipment.push(EquipmentSlot {
        item: Id::new("weapon.sword_long"),
        equipped: true,
        specialization_applies: false,
    });
    let result = validate(&e, &rs);
    assert!(
        !result
            .issues
            .iter()
            .any(|i| i.code == "shield_with_two_handed_weapon"),
        "a one-handed weapon makes the shield usable — no advisory"
    );
}

/// Issue C: `specialization_applies` is a canonical, additive field — it survives
/// a JSON round-trip and participates in the derived `Ord`, so `normalize` sorts
/// deterministically on it and it serializes only when true (skip-when-false).
#[test]
fn specialization_applies_round_trips_and_sorts() {
    // Skip-when-false keeps the JSON noise-free; true is written.
    let off = EquipmentSlot {
        item: Id::new("weapon.sword_long"),
        equipped: true,
        specialization_applies: false,
    };
    let off_json = serde_json::to_string(&off).unwrap();
    assert!(
        !off_json.contains("specialization_applies"),
        "false is skipped: {off_json}"
    );
    let on = EquipmentSlot {
        specialization_applies: true,
        ..off.clone()
    };
    let on_json = serde_json::to_string(&on).unwrap();
    assert!(on_json.contains("specialization_applies"));
    assert_eq!(serde_json::from_str::<EquipmentSlot>(&on_json).unwrap(), on);

    // The field joins the derived Ord: two otherwise-identical slots order with
    // `false` before `true`, so normalize is deterministic.
    let mut e = entity("grog", vec![]);
    e.equipment = vec![on.clone(), off.clone()];
    e.normalize();
    assert!(!e.equipment[0].specialization_applies);
    assert!(e.equipment[1].specialization_applies);
}

#[test]
fn normalize_sorts_equipment() {
    let mut e = entity("grog", vec![]);
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("weapon.warhammer"),
            equipped: false,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("armor.chain_mail_full"),
            equipped: true,
            specialization_applies: false,
        },
    ];
    e.normalize();
    assert_eq!(e.equipment[0].item, Id::new("armor.chain_mail_full"));
    assert_eq!(e.equipment[1].item, Id::new("weapon.warhammer"));
}

/// The shipped Skilled Parens raises *both* the spell-levels budget (+30) and the
/// general apprenticeship XP pool (+60), proving its two-effect package is wired
/// end-to-end against real data (ArMDE:4964-4966).
#[test]
fn shipped_skilled_parens_raises_both_budgets() {
    let rs = load_ruleset_with_spells();
    let mut e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.skilled_parens"))],
    );
    e.xp_pool = 240;
    assert_eq!(arm_rules::spell_levels_budget(120, &e, &rs), 150);
    assert_eq!(
        arm_rules::checked_xp_allocation(&e, &rs)
            .unwrap()
            .general_pool,
        300
    );
}

/// The shipped Elemental Magic (ArMDE:3731-3737) redistributes Art-XP over the four
/// elemental Forms against the real Arts catalogue: each Form gains half (rounded
/// up) of every other Form's table-XP. With Ignem/Auram/Terram at score 6 (21 XP)
/// and Aquam at score 4 (10 XP), the boosted effective scores are 9/9/9 and 8,
/// while a non-elemental Form (Corpus) at score 6 is untouched.
#[test]
fn shipped_elemental_magic_redistributes_art_xp() {
    let rs = load_ruleset_with_spells();
    let mut e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.elemental_magic"))],
    );
    let row = |art: &str, score: u8| ArtScore {
        art: Id::new(art),
        score,
    };
    e.art_scores = vec![
        row("art.aquam", 4),
        row("art.auram", 6),
        row("art.ignem", 6),
        row("art.terram", 6),
        row("art.corpus", 6),
    ];
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.aquam")), 8);
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.auram")), 9);
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.ignem")), 9);
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.terram")), 9);
    // A non-elemental Form is never touched by the redistribution.
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.corpus")), 6);
}

/// The shipped Weak Parens lowers both budgets (ArMDE:7072-7074).
#[test]
fn shipped_weak_parens_lowers_both_budgets() {
    let rs = load_ruleset_with_spells();
    let mut e = entity("magus", vec![Selection::new(Id::new("flaw.weak_parens"))]);
    e.xp_pool = 240;
    assert_eq!(arm_rules::spell_levels_budget(120, &e, &rs), 90);
    assert_eq!(
        arm_rules::checked_xp_allocation(&e, &rs)
            .unwrap()
            .general_pool,
        180
    );
}

#[test]
fn fully_specified_companion_validates() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );
    // Int +2 (3) + Per +1 (1) + Sta -1 (-1) + others 0 = 3 <= 7 (under -> warning only).
    e.characteristics = BTreeMap::from([
        (Characteristic::Int, 2),
        (Characteristic::Per, 1),
        (Characteristic::Sta, -1),
    ]);
    e.ability_scores = vec![
        AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2,
            specialty: Some("searching".into()),
            parameter: None,
        },
        AbilityScore {
            ability: Id::new("ability.living_language"),
            score: 5,
            specialty: None,
            parameter: Some("German".into()),
        },
    ];
    // Awareness 2 (15 xp) + Living Language 5 (75 xp) = 90 spent; give a pool that
    // covers it (banking the rest).
    e.xp_pool = 120;

    let result = validate(&e, &rs);
    assert!(
        result.is_valid(),
        "fully-specified companion should validate: {:?}",
        result.issues
    );
}

/// Every item id named in some character-type profile's `required_traits` or
/// `forbidden_traits` (`rules/core/character_types.json`). This is the *other*
/// mechanism (besides an item's own `effects`) that wires a V/F id to something
/// the engine actually reads — D46's own worked example is exactly this shape:
/// `virtue.hermetic_magus` carries no `effects`, but the magus profile's
/// `required_traits` enforces the rule its passage states ("All magi must take
/// this as their Social Status").
fn profile_trait_reference_ids() -> BTreeSet<String> {
    let profiles: Vec<serde_json::Value> =
        serde_json::from_str(SHIPPED_TYPE_PROFILES).expect("character_types.json is valid JSON");
    let mut ids = BTreeSet::new();
    for profile in &profiles {
        for field in ["required_traits", "forbidden_traits"] {
            if let Some(list) = profile[field].as_array() {
                for id in list {
                    if let Some(id) = id.as_str() {
                        ids.insert(id.to_string());
                    }
                }
            }
        }
    }
    ids
}

/// True when `item` is computed by *something* — D46 (`docs/vf-audit/decisions.md`):
/// "classification follows what is computed, never where it is computed." Two
/// sources, and only two, currently wire a V/F id to an enforced consequence:
/// the item's own `effects`, or a character-type profile's
/// `required_traits`/`forbidden_traits` naming it (`profile_referenced`).
fn is_computed(item: &PointItem, profile_referenced: &BTreeSet<String>) -> bool {
    !item.effects.is_empty() || profile_referenced.contains(item.id.as_str())
}

/// D46's shrink-only pending work-list (plan § 1, `docs/vf-audit/phase-2-plan.md`):
/// every entry the rewritten [`every_vf_is_classified`] finds misclassified under
/// "classification follows what is computed, never where." `measurements.md`
/// § 8 row 11 names the five effect-less `creation_effect` entries; D46 itself
/// names the other two, `virtue.the_gift` and `virtue.hermetic_magus`, both
/// classified `narrative` today despite being named in a type profile's
/// `required_traits`/`forbidden_traits` — D46 rules the *opposite* correction for
/// each (`the_gift` to `uncomputed_rule`, `hermetic_magus` to `creation_effect`),
/// which is exactly why classification-follows-computation cannot itself decide
/// *which* of the two computed classes an entry belongs to — X2 does that.
/// `(id, why)`; [`pending_d46_classification_entries_still_trip_the_guard`] keeps
/// every row honest.
const PENDING_D46_CLASSIFICATION: &[(&str, &str)] = &[
    (
        "flaw.corrupted_arts",
        "creation_effect, carries no effects, and is named in no type profile's \
         required_traits/forbidden_traits (measurements.md § 8 row 11)",
    ),
    (
        "flaw.savantism",
        "creation_effect, carries no effects, and is named in no type profile's \
         required_traits/forbidden_traits (measurements.md § 8 row 11)",
    ),
    (
        "virtue.devil_child",
        "creation_effect, carries no effects, and is named in no type profile's \
         required_traits/forbidden_traits (measurements.md § 8 row 11)",
    ),
    (
        "virtue.nephilim",
        "creation_effect, carries no effects, and is named in no type profile's \
         required_traits/forbidden_traits (measurements.md § 8 row 11)",
    ),
    (
        "virtue.simple_student",
        "creation_effect, carries no effects, and is named in no type profile's \
         required_traits/forbidden_traits (measurements.md § 8 row 11)",
    ),
    (
        "virtue.the_gift",
        "narrative, but named in the grog profile's forbidden_traits — D46's ruling: \
         becomes uncomputed_rule (ArMDE:2870-2876's \"suffers all the penalties of The \
         Gift\" is the clause that stays uncomputed)",
    ),
    (
        "virtue.hermetic_magus",
        "narrative, but named in the magus profile's required_traits — D46's ruling: \
         stays/becomes creation_effect",
    ),
];

/// Acceptance criterion for M5 slice 5a: every shipped Virtue/Flaw carries a
/// `classification`. The field is required (no serde default), so an unclassified
/// entry would already fail `load_ruleset()`; this test additionally asserts the
/// catalogue is non-trivial and that all four classes are actually used, so the
/// classification pass can never silently collapse to a single bucket.
#[test]
fn every_vf_is_classified() {
    let rs = load_ruleset();
    let mut narrative = 0usize;
    let mut uncomputed = 0usize;
    let mut creation = 0usize;
    let mut in_play = 0usize;
    for item in rs.items() {
        match item.classification {
            Classification::Narrative => narrative += 1,
            Classification::UncomputedRule => uncomputed += 1,
            Classification::CreationEffect => creation += 1,
            Classification::InPlayEffect => in_play += 1,
        }
    }
    // Catalogue size is data, not code: assert only that the catalogue is large
    // and every class is represented, never exact per-class totals.
    assert!(
        narrative + uncomputed + creation + in_play > 600,
        "expected the full V/F catalogue to load"
    );
    assert!(narrative > 0, "some V/F must be narrative");
    assert!(uncomputed > 0, "some V/F must be uncomputed_rule");
    assert!(creation > 0, "some V/F must be creation_effect");
    assert!(in_play > 0, "some V/F must be in_play_effect");

    // D46: classification follows what is computed, never where. The old guard
    // asked only whether `effects` was non-empty, and only in one direction —
    // required on `in_play_effect`, never checked on `creation_effect` — which is
    // why five effect-less `creation_effect` entries passed silently
    // (measurements.md § 8 row 11). This asks the symmetric question of all four
    // classes: the two "something is computed" classes must have a computation
    // source, and the two "nothing is computed" classes must not.
    let profile_referenced = profile_trait_reference_ids();
    let mut offenders = Vec::new();
    for item in rs.items() {
        let id = item.id.as_str();
        if PENDING_D46_CLASSIFICATION
            .iter()
            .any(|(pending, _)| *pending == id)
        {
            continue;
        }
        let computed = is_computed(item, &profile_referenced);
        match item.classification {
            Classification::Narrative | Classification::UncomputedRule if computed => {
                offenders.push(format!(
                    "{id}: classified {:?}, but is computed (effects, or named in a type \
                     profile's required_traits/forbidden_traits) — D46 says that makes it \
                     creation_effect or in_play_effect, never {:?}",
                    item.classification, item.classification
                ));
            }
            Classification::CreationEffect | Classification::InPlayEffect if !computed => {
                offenders.push(format!(
                    "{id}: classified {:?}, but computes nothing — no effects, and named in \
                     no type profile's required_traits/forbidden_traits",
                    item.classification
                ));
            }
            _ => {}
        }
    }
    assert!(
        offenders.is_empty(),
        "D46: classification must follow what is computed, never where it is computed:\n{}",
        offenders.join("\n")
    );

    // M5/5b acceptance, unchanged and stricter than D46 alone: an in_play_effect
    // derived-total modifier must be wired on the entry itself, not merely
    // enforced by a profile elsewhere — a type profile's required/forbidden
    // traits is not how a derived total (Soak, a Lab Total, …) is computed.
    for item in rs.items() {
        if item.classification == Classification::InPlayEffect {
            assert!(
                !item.effects.is_empty(),
                "{} is in_play_effect but carries no effect",
                item.id
            );
        }
    }
}

/// The mirror of `uncomputed_clauses.rs::pending_mechanical_classification_entries_still_trip_the_screen`,
/// for [`PENDING_D46_CLASSIFICATION`]: every pending row must still trip the D46
/// guard, so the list can only shrink as X2 reclassifies or wires each entry —
/// never grow stale.
#[test]
fn pending_d46_classification_entries_still_trip_the_guard() {
    let rs = load_ruleset();
    let profile_referenced = profile_trait_reference_ids();

    for (id, _) in PENDING_D46_CLASSIFICATION {
        let item = rs
            .items()
            .find(|item| item.id.as_str() == *id)
            .unwrap_or_else(|| {
                panic!(
                    "PENDING_D46_CLASSIFICATION row \"{id}\" names an entry that is no longer \
                     in the catalogue — delete the row"
                )
            });
        let computed = is_computed(item, &profile_referenced);
        let still_offends = match item.classification {
            Classification::Narrative | Classification::UncomputedRule => computed,
            Classification::CreationEffect | Classification::InPlayEffect => !computed,
        };
        assert!(
            still_offends,
            "PENDING_D46_CLASSIFICATION row \"{id}\" no longer trips the D46 guard — delete \
             the row"
        );
    }
}

#[test]
fn english_i18n_covers_all_items() {
    let rs = load_ruleset();
    let i18n_en = include_str!("../../../rules/i18n/en/virtues_flaws.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();

    for item in rs.items() {
        assert!(
            loc.display_name(&item.id).is_some(),
            "English i18n missing entry for '{}'",
            item.id
        );
    }
}

#[test]
fn german_i18n_covers_all_items() {
    let rs = load_ruleset();
    let i18n_de = include_str!("../../../rules/i18n/de/virtues_flaws.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();

    for item in rs.items() {
        assert!(
            loc.display_name(&item.id).is_some(),
            "German i18n missing entry for '{}'",
            item.id
        );
    }
}

/// Every `summary` in both locales' `virtues_flaws.json` must end on a
/// sentence boundary.
///
/// The convention these files follow is "the first sentence of the rulebook
/// entry's body text, verbatim" (compare `virtue.inoffensive_to_beings` against
/// ArMDE:4135). An earlier generation
/// pass capped every summary at roughly 240 characters, which silently cut 20
/// strings across 16 entries off **mid-word** — user-facing rules text truncated
/// to nonsense, and nothing in the suite noticed. This is that missing witness.
///
/// The accepted terminators are deliberately a *set*, not just `.`: two entries
/// legitimately end in `!` (`flaw.gullible`, "There is one born every minute -
/// and it is this character!"), and a first sentence can equally close on a
/// parenthesis or a quotation mark. A cut-off string ends on a letter, so a
/// terminator check is enough to separate the two cases without hardcoding any
/// length — the cap was the bug, so no length limit is asserted here.
///
/// Scoped to `virtues_flaws.json` on purpose: the sentence-boundary convention
/// is a property of that file's summaries. Spell descriptions are policed by
/// their own witness, `no_spell_description_carries_markdown_table`, which
/// guards the defect they actually had (leaked Markdown table rows) rather than
/// end punctuation — two spell descriptions legitimately end mid-sentence
/// because the rulebook itself does.
#[test]
fn no_virtue_flaw_summary_ends_mid_sentence() {
    /// Punctuation a complete first sentence may end on: the three sentence
    /// terminators, plus the closers a terminated sentence can hide behind
    /// (`)`, and the ASCII/typographic quotation marks — German closing quotes
    /// included, since the German file is translated prose).
    const SENTENCE_ENDINGS: [char; 8] =
        ['.', '!', '?', ')', '"', '\u{201c}', '\u{201d}', '\u{00bb}'];

    // Every offender is collected before asserting, so one run names the whole
    // set instead of stopping at whichever id happens to sort first.
    let mut offenders: Vec<String> = Vec::new();
    let mut summaries_seen = 0usize;

    for (lang, i18n) in [
        (
            "en",
            include_str!("../../../rules/i18n/en/virtues_flaws.json"),
        ),
        (
            "de",
            include_str!("../../../rules/i18n/de/virtues_flaws.json"),
        ),
    ] {
        let file: serde_json::Value =
            serde_json::from_str(i18n).expect("the shipped i18n file is valid JSON");
        let entries = file
            .as_object()
            .expect("the i18n file is a map of id -> text");
        assert!(
            !entries.is_empty(),
            "i18n/{lang}/virtues_flaws.json is empty"
        );

        for (id, text) in entries {
            let Some(summary) = text.get("summary").and_then(serde_json::Value::as_str) else {
                continue;
            };
            summaries_seen += 1;
            let last = summary
                .chars()
                .next_back()
                .unwrap_or_else(|| panic!("[{lang}] '{id}' has an empty summary"));
            if SENTENCE_ENDINGS.contains(&last) {
                continue;
            }
            // Char-counted, not byte-sliced: the German text is full of
            // multi-byte codepoints, and a byte slice would panic on a
            // non-boundary index instead of reporting the offender.
            let tail: Vec<char> = summary.chars().rev().take(50).collect();
            let tail: String = tail.into_iter().rev().collect();
            offenders.push(format!("[{lang}] {id} — ends on '{last}': ...{tail}"));
        }
    }

    assert!(
        summaries_seen > 0,
        "neither locale's virtues_flaws.json carries any summary at all"
    );
    assert!(
        offenders.is_empty(),
        "{} summary/summaries end mid-sentence — restore each one's full first \
         sentence from its `source` range in rules/source/<lang>/:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

/// No spell `description` in either locale may carry Markdown table markup.
///
/// `scripts/extract_spells.py` accumulated every non-blank line of a spell's
/// body into its description, so where the rulebook interrupts the prose with a
/// table (e.g. *Mists of Change*'s shape-change list) the raw rows arrived
/// complete with `|` cell delimiters and column padding — markup rendered
/// verbatim in the UI, which shows the description as prose and cannot lay out
/// a table. The authoritative tabular text stays in the Markdown source; the
/// description field carries only the surrounding prose.
///
/// The witness is the pipe character, deliberately *not* end punctuation: a
/// leaked row can sit in the middle of a description (*Mists of Change* has a
/// genuine closing paragraph after its table), which an end-of-string check
/// would miss, and two descriptions legitimately end without a full stop
/// because their source lines do — `spell.notes_of_a_delightful_sound`
/// (ArMDE:14676) and
/// `spell.scent_of_peaceful_slumber` (ArMDE:15171). Those are faithful extractions
/// of typographic slips in the rulebook and must not be "corrected" here.
#[test]
fn no_spell_description_carries_markdown_table() {
    // Every offender is collected before asserting, so one run names the whole
    // set instead of stopping at whichever id happens to sort first.
    let mut offenders: Vec<String> = Vec::new();
    let mut descriptions_seen = 0usize;

    for (lang, i18n) in [
        ("en", include_str!("../../../rules/i18n/en/spells.json")),
        ("de", include_str!("../../../rules/i18n/de/spells.json")),
    ] {
        let file: serde_json::Value =
            serde_json::from_str(i18n).expect("the shipped i18n file is valid JSON");
        let entries = file
            .as_object()
            .expect("the i18n file is a map of id -> text");
        assert!(!entries.is_empty(), "i18n/{lang}/spells.json is empty");

        for (id, text) in entries {
            let Some(description) = text.get("description").and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            descriptions_seen += 1;
            if !description.contains('|') {
                continue;
            }
            // Char-counted, not byte-sliced: the German text is full of
            // multi-byte codepoints, and a byte slice would panic on a
            // non-boundary index instead of reporting the offender.
            let excerpt: String = description
                .chars()
                .skip_while(|c| *c != '|')
                .take(60)
                .collect();
            offenders.push(format!("[{lang}] {id} — leaked table markup: {excerpt}"));
        }
    }

    assert!(
        descriptions_seen > 0,
        "neither locale's spells.json carries any description at all"
    );
    assert!(
        offenders.is_empty(),
        "{} spell description(s) carry Markdown table markup — the table belongs \
         in rules/source/<lang>/, not in the prose description:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

#[test]
fn companion_balanced_entity_validates() {
    let rs = load_ruleset();

    let entity = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.keen_vision")),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );

    let result = validate(&entity, &rs);
    assert!(
        result.is_valid(),
        "balanced companion should validate: {:?}",
        result.issues
    );

    let balance = compute_balance(&entity, &rs);
    assert_eq!(balance.virtue_points, 1);
    assert_eq!(balance.flaw_points, 1);
}

#[test]
fn save_load_roundtrip_with_canonical_output() {
    let mut entity = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );
    // Canonical output requires normalize(): Serialize no longer auto-sorts.
    entity.normalize();

    let json1 = serde_json::to_string_pretty(&entity).unwrap();
    let roundtripped: Entity = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&roundtripped).unwrap();

    assert_eq!(json1, json2, "canonical serialization should be stable");
    assert_eq!(entity, roundtripped);
}

#[test]
fn grog_type_restricts_major_virtues() {
    let rs = load_ruleset();

    // One minor virtue funded by one minor flaw, so the points balance and
    // the test isolates the Major-virtue restriction.
    let entity = entity(
        "grog",
        vec![
            Selection::new(Id::new("virtue.keen_vision")),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );

    let result = validate(&entity, &rs);
    assert!(
        result.is_valid(),
        "grog with one balanced minor virtue should be valid: {:?}",
        result.issues
    );
}

#[test]
fn grog_over_budget() {
    let rs = load_ruleset();

    let entity = entity(
        "grog",
        vec![
            Selection::new(Id::new("virtue.keen_vision")),
            Selection::new(Id::new("virtue.large")),
            Selection::new(Id::new("virtue.tough")),
            Selection::new(Id::new("virtue.puissant_ability")),
        ],
    );

    let result = validate(&entity, &rs);
    let codes: Vec<&str> = result.errors().map(|i| i.code.as_str()).collect();
    assert!(
        codes.contains(&"over_budget_virtues"),
        "grog over budget: {codes:?}"
    );
}

#[test]
fn shipped_score_effects_apply() {
    use arm_rules::{characteristic_cap, characteristic_floor, effective_ability_score};
    let rs = load_ruleset();

    // Puissant Ability (+2 bonus) and Great/Poor Characteristic (limit shifts)
    // from the shipped data.
    let mut e = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::with_params(
                Id::new("virtue.great_characteristic"),
                BTreeMap::from([("characteristic".into(), Characteristic::Str.id())]),
            ),
            Selection::with_params(
                Id::new("flaw.poor_characteristic"),
                BTreeMap::from([("characteristic".into(), Characteristic::Qik.id())]),
            ),
        ],
    );
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.awareness"),
        score: 2,
        specialty: None,
        parameter: None,
    }];
    e.characteristics = BTreeMap::from([(Characteristic::Str, 3), (Characteristic::Qik, -3)]);

    // Puissant adds to the effective ability score.
    assert_eq!(
        effective_ability_score(&e, &rs, &Id::new("ability.awareness"), None),
        4,
        "Awareness 2 + Puissant +2"
    );
    // Great grants Strength a free point and Poor takes one off Quickness; the
    // buy range stays the printed ±3 for every characteristic, targeted or not.
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Str),
        4,
        "Great grants Strength its fourth point"
    );
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Qik),
        -4,
        "Poor drops Quickness to -4"
    );
    for characteristic in [
        Characteristic::Str,
        Characteristic::Qik,
        Characteristic::Int,
    ] {
        assert_eq!(characteristic_cap(&rs, characteristic), 3);
        assert_eq!(characteristic_floor(&rs, characteristic), -3);
    }
}

// --- M5 slice 5b: in-play effect variants ---

/// Helper: the set of issue codes `validate` emits for an entity.
fn issue_codes(entity: &Entity, rs: &Ruleset) -> Vec<String> {
    validate(entity, rs)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

/// A magus may hold at most one Magical Focus (ArMDE:4542): two Minor
/// Foci (distinct descriptors, so not a duplicate selection) trip the
/// `multiple_magical_foci` rule, which counts the `MagicalFocus` effect rather
/// than relying on pairwise incompatibility (which cannot catch two Minors).
#[test]
fn two_magical_foci_are_rejected() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![
            Selection::with_params(
                Id::new("virtue.minor_magical_focus"),
                BTreeMap::from([("focus".into(), Id::new("necromancy"))]),
            ),
            Selection::with_params(
                Id::new("virtue.minor_magical_focus"),
                BTreeMap::from([("focus".into(), Id::new("weather"))]),
            ),
        ],
    );
    assert!(
        issue_codes(&e, &rs).contains(&"multiple_magical_foci".to_string()),
        "two foci must be rejected"
    );
}

/// A single Magical Focus is legal — the one-focus rule does not fire.
#[test]
fn one_magical_focus_is_allowed() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("virtue.major_magical_focus"),
            BTreeMap::from([("focus".into(), Id::new("necromancy"))]),
        )],
    );
    assert!(
        !issue_codes(&e, &rs).contains(&"multiple_magical_foci".to_string()),
        "one focus must be allowed"
    );
}

/// Deficient Form's parameter is Form-domain, so targeting a Technique (art.creo)
/// fails parameter resolution — the art-class restriction the slice requires.
#[test]
fn deficient_form_cannot_target_a_technique() {
    let rs = load_ruleset_with_spells();
    let e = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.creo"))]),
        )],
    );
    assert!(
        issue_codes(&e, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Form targeting a Technique must be rejected"
    );
    // A Form target (art.ignem) resolves cleanly.
    let ok = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.ignem"))]),
        )],
    );
    assert!(
        !issue_codes(&ok, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Form targeting a Form must be accepted"
    );
}

/// Deficient Technique's parameter is Technique-domain, so targeting a Form
/// (art.ignem) fails; a Technique (art.creo) resolves.
#[test]
fn deficient_technique_cannot_target_a_form() {
    let rs = load_ruleset_with_spells();
    let bad = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("technique".into(), Id::new("art.ignem"))]),
        )],
    );
    assert!(
        issue_codes(&bad, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Technique targeting a Form must be rejected"
    );
    let ok = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("technique".into(), Id::new("art.creo"))]),
        )],
    );
    assert!(
        !issue_codes(&ok, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Technique targeting a Technique must be accepted"
    );
}

/// In-play effects (Tough/Soak, Method Caster, Enduring Constitution, a Magical
/// Focus, Deficient Form) never perturb creation-legality totals: adding them
/// leaves the XP allocation, characteristic caps, and effective ability scores
/// exactly as they were. They cost/grant only the point-balance their magnitude
/// dictates, computed elsewhere.
#[test]
fn in_play_effects_do_not_perturb_creation_totals() {
    use arm_rules::{checked_xp_allocation, effective_ability_score};
    let rs = load_ruleset_with_spells();

    let mut base = entity("magus", vec![]);
    base.xp_pool = 15;
    base.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.awareness"),
        score: 3,
        specialty: None,
        parameter: None,
    }];
    base.characteristics = BTreeMap::from([(Characteristic::Int, 2)]);

    let mut with_effects = base.clone();
    with_effects.selections = vec![
        Selection::new(Id::new("virtue.tough")),
        Selection::new(Id::new("virtue.method_caster")),
        Selection::new(Id::new("virtue.enduring_constitution")),
        Selection::with_params(
            Id::new("virtue.major_magical_focus"),
            BTreeMap::from([("focus".into(), Id::new("necromancy"))]),
        ),
        Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.ignem"))]),
        ),
    ];

    assert_eq!(
        checked_xp_allocation(&base, &rs).unwrap().total_demand,
        checked_xp_allocation(&with_effects, &rs)
            .unwrap()
            .total_demand,
        "in-play effects must not change XP demand"
    );
    assert_eq!(
        effective_characteristic_score(&base, &rs, Characteristic::Int),
        effective_characteristic_score(&with_effects, &rs, Characteristic::Int),
        "in-play effects must not change effective characteristic scores"
    );
    assert_eq!(
        effective_ability_score(&base, &rs, &Id::new("ability.awareness"), None),
        effective_ability_score(&with_effects, &rs, &Id::new("ability.awareness"), None),
        "in-play effects must not change effective ability scores"
    );
}

// --- M5 slice 5a-wire: creation-effect wiring on the shipped catalogue ---

/// Helper: does an entity's reputation set raise `reputation_not_granted`?
fn reputation_ungranted(entity: &Entity, rs: &Ruleset) -> bool {
    issue_codes(entity, rs).contains(&"reputation_not_granted".to_string())
}

/// Builds a companion holding a single virtue/flaw (no params) plus one
/// player-declared Reputation of `kind`/`score`, to check the grant authorizes it.
fn companion_with_reputation(item: &str, kind: ReputationType, score: u8) -> Entity {
    let mut e = entity("companion", vec![Selection::new(Id::new(item))]);
    e.reputations = vec![Reputation {
        kind,
        score,
        content: "test".into(),
    }];
    e
}

#[test]
fn shipped_reputation_granters_authorize_their_kind() {
    let rs = load_ruleset();
    // Hermetic Prestige → a Hermetic Reputation at 4 (ArMDE:4071-4073).
    let hp = companion_with_reputation("virtue.hermetic_prestige", ReputationType::Hermetic, 4);
    assert!(
        !reputation_ungranted(&hp, &rs),
        "Hermetic Prestige grants Hermetic"
    );
    // Baccalaureus → an Academic Reputation (ArMDE:3472).
    let bac = companion_with_reputation("virtue.baccalaureus", ReputationType::Academic, 1);
    assert!(
        !reputation_ungranted(&bac, &rs),
        "Baccalaureus grants Academic"
    );
    // A Local reputation is NOT authorized by Hermetic Prestige alone.
    let wrong = companion_with_reputation("virtue.hermetic_prestige", ReputationType::Local, 4);
    assert!(
        reputation_ungranted(&wrong, &rs),
        "Hermetic Prestige does not grant Local"
    );
}

#[test]
fn shipped_famous_authorizes_any_reputation_kind() {
    let rs = load_ruleset();
    // Famous (ArMDE:3861-3863): player chooses the type — any single type is legal.
    for kind in ReputationType::ALL {
        let e = companion_with_reputation("virtue.famous", kind, 4);
        assert!(
            !reputation_ungranted(&e, &rs),
            "Famous authorizes a {kind} Reputation",
        );
    }
    // But only ONE: two reputations exceed the single wildcard grant.
    let mut two = companion_with_reputation("virtue.famous", ReputationType::Local, 4);
    two.reputations.push(Reputation {
        kind: ReputationType::Hermetic,
        score: 4,
        content: "second".into(),
    });
    assert!(
        reputation_ungranted(&two, &rs),
        "Famous grants only one Reputation"
    );
}

/// D11/Q5 data fixes — `corrections.md` § 3.11's findings, each a plain data
/// change unblocked by D11's ruling that the model does not grow: `(id, kind,
/// score, max_score, ArMDE line)`.
///
/// - F-80 `virtue.frightful_presence`: "an appropriate Reputation … at a score
///   of 2 among those you have affected" — an ad-hoc audience, not one of the
///   four fixed types, so wildcard.
/// - F-235 `virtue.protection`: "a Reputation (good or bad, your choice) of
///   level 3" — audience and polarity both left to the player.
/// - F-254 `virtue.rosh_beth_din`: "applies across his country" — wider than
///   Local, and not Ecclesiastical/Hermetic/Academic either, so the shipped
///   `kind: "local"` was wrong; wildcard instead.
/// - F-312 `virtue.templar_commander`: "a Reputation of level 3 in his area" —
///   Local, and carried by no effect at all before this fix.
/// - F-316 `virtue.templar_prestige`: "a Reputation of level 4 within the
///   Templars" — an organization, not one of the four fixed types, so
///   wildcard; also reclassified from `narrative` (the passage states a
///   mechanical rule).
/// - F-408 `flaw.excommunicate`: "a bad reputation at level 3 within the
///   Church" — carried by no effect at all before this fix; also reclassified.
/// - F-450 `flaw.infamous`: "a level 4 bad Reputation" — no audience stated;
///   its twin `virtue.famous` already ships the wildcard for the same shape.
/// - F-484 `flaw.outlaw`: "a Reputation at level 2 for whatever got you
///   outlawed" — no audience stated.
/// - F-486 `flaw.outsider_major` / `_minor`: "a bad Reputation of level 1 to
///   3" — the one entry in the whole catalogue the book states a RANGE for,
///   on both magnitudes ("You still have the bad Reputation", `ArMDE:6556`).
type ReputationGrantDataFix = (&'static str, Option<ReputationType>, u8, Option<u8>, u32);
const REPUTATION_GRANT_DATA_FIXES: &[ReputationGrantDataFix] = &[
    ("virtue.frightful_presence", None, 2, None, 3947),
    ("virtue.protection", None, 3, None, 4812),
    ("virtue.rosh_beth_din", None, 2, None, 4882),
    (
        "virtue.templar_commander",
        Some(ReputationType::Local),
        3,
        None,
        5115,
    ),
    ("virtue.templar_prestige", None, 4, None, 5127),
    (
        "flaw.excommunicate",
        Some(ReputationType::Ecclesiastical),
        3,
        None,
        6046,
    ),
    ("flaw.infamous", None, 4, None, 6312),
    ("flaw.outlaw", None, 2, None, 6544),
    (
        "flaw.outsider_major",
        Some(ReputationType::Local),
        1,
        Some(3),
        6554,
    ),
    (
        "flaw.outsider_minor",
        Some(ReputationType::Local),
        1,
        Some(3),
        6556,
    ),
];

#[test]
fn shipped_reputation_grants_match_their_passage_after_d11() {
    let rs = load_ruleset();
    for (id, expected_kind, expected_score, expected_max, line) in REPUTATION_GRANT_DATA_FIXES {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        let grant = item
            .effects
            .iter()
            .find_map(|e| match e {
                Effect::GrantsReputation {
                    kind,
                    score,
                    max_score,
                } => Some((*kind, *score, *max_score)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{id} must carry a grants_reputation effect (ArMDE:{line})"));
        assert_eq!(
            grant,
            (*expected_kind, *expected_score, *expected_max),
            "{id} (ArMDE:{line})"
        );
        assert_eq!(
            item.classification,
            Classification::CreationEffect,
            "{id} carries a reputation grant, so it is a creation_effect (ArMDE:{line})"
        );
    }
}

/// F-194/D29: `virtue.mentored_by_demons` (ArMDE:4496-4499) states "Characters
/// trained by demons may exceed the maximum skill level for a given age
/// provided by the character creation rules" (ArMDE:4498) — a waiver the
/// engine enforced against with no hook, so the app refused the legal
/// character the passage describes. The fix is a `waives_ability_age_cap`
/// effect on the shipped entry, consumed by the D29 resolution point
/// (`effective/reputation_and_caps.rs::ability_age_cap`).
#[test]
fn shipped_mentored_by_demons_carries_the_age_cap_waiver() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("virtue.mentored_by_demons"))
        .expect("virtue.mentored_by_demons must ship");
    assert!(
        item.effects.contains(&Effect::WaivesAbilityAgeCap),
        "virtue.mentored_by_demons must carry waives_ability_age_cap (ArMDE:4498)"
    );
}

/// F-194: the character the passage describes — trained by demons, an Ability
/// bought above the age band — must validate. A companion aged 20 (age-band
/// cap 5, ArMDE:2366-2374) with the Virtue and Brawl bought at 7 is exactly
/// that character.
#[test]
fn mentored_by_demons_character_validates_above_the_age_band() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.mentored_by_demons"))],
    );
    e.age = Some(20);
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.brawl"),
        parameter: None,
        score: 7,
        specialty: None,
    }];
    let result = validate(&e, &rs);
    assert!(
        !result
            .issues
            .iter()
            .any(|i| i.code == ValidationIssue::CODE_ABILITY_ABOVE_AGE_CAP),
        "Mentored by Demons must waive the age cap (ArMDE:4498): {:?}",
        result.issues
    );
}

#[test]
fn shipped_supernatural_virtues_grant_starting_score() {
    use arm_rules::effective_ability_score;
    let rs = load_ruleset();
    for (item, ability) in [
        ("virtue.animal_ken", "ability.animal_ken"),
        ("virtue.shapeshifter", "ability.shapeshifter"),
        ("virtue.enchanting_ability", "ability.enchanting"),
        ("virtue.wilderness_sense", "ability.wilderness_sense"),
    ] {
        let e = entity("companion", vec![Selection::new(Id::new(item))]);
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new(ability), None),
            1,
            "{item} grants {ability} at 1",
        );
    }
    // Strong Faerie Blood grants the Second Sight *Virtue* for free (ArMDE:5038),
    // which in turn floors Second Sight at 1.
    let sfb = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.strong_faerie_blood"))],
    );
    assert_eq!(
        effective_ability_score(&sfb, &rs, &Id::new("ability.second_sight"), None),
        1,
        "Strong Faerie Blood grants Second Sight via a nested Virtue grant",
    );
}

#[test]
fn shipped_xp_granters_add_restricted_pool() {
    use arm_rules::restricted_xp_pools;
    let rs = load_ruleset();
    // Arcane Lore → +50 XP restricted to Arcane abilities (ArMDE:3432).
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.arcane_lore"))],
    );
    let pools = restricted_xp_pools(&e, &rs);
    assert!(
        pools
            .iter()
            .any(|p| p.amount == 50 && p.categories.contains(&AbilityCategory::Arcane)),
        "Arcane Lore grants a 50-xp Arcane-restricted pool",
    );
    // Feral Upbringing → 120 XP on a fixed ability list (ArMDE:6112).
    let fu = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.feral_upbringing"))],
    );
    assert!(
        restricted_xp_pools(&fu, &rs)
            .iter()
            .any(|p| p.amount == 120),
        "Feral Upbringing grants a 120-xp restricted pool",
    );
}

#[test]
fn shipped_confidence_true_faith_and_size_granters() {
    use arm_rules::{Confidence, confidence, size, true_faith};
    let rs = load_ruleset();
    // Ferocity → +1 Confidence Score / +3 Points (ArMDE:3875) over the base.
    let fer = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.ferocity"))],
    );
    assert_eq!(
        confidence(1, 3, &fer, &rs),
        Confidence {
            score: 2,
            points: 6
        },
        "Ferocity adds 1/3"
    );
    // Low Self-Esteem → removes the standard 1/3 Confidence (ArMDE:6364).
    let lse = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.low_self_esteem"))],
    );
    assert_eq!(
        confidence(1, 3, &lse, &rs),
        Confidence {
            score: 0,
            points: 0
        },
        "Low Self-Esteem zeroes Confidence"
    );
    // Relic → True Faith 1 (ArMDE:4854); Powerful Relic → 3 (ArMDE:4783).
    let relic = entity("companion", vec![Selection::new(Id::new("virtue.relic"))]);
    assert_eq!(true_faith(&relic, &rs), 1);
    let prelic = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.powerful_relic"))],
    );
    assert_eq!(true_faith(&prelic, &rs), 3);
    // Blood of the Nephilim → Size +1 (Divine:1945).
    let bon = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.blood_of_the_nephilim"))],
    );
    assert_eq!(
        size(&bon, &rs),
        1,
        "Blood of the Nephilim raises Size to +1"
    );
}

/// The full shipped ruleset — every catalogue loaded, exactly as the app ships
/// it (`ruleset_io.rs`). This is the production data the derived-totals end-to-end
/// test runs against.
fn load_full_ruleset() -> Ruleset {
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
        childhoods: Some(SHIPPED_CHILDHOODS),
        aging: Some(SHIPPED_AGING),
    })
    .unwrap()
}

/// End-to-end proof of the M5 headline capability: a fully-specified magus built
/// on the **real shipped ruleset** is fully *computable* in direct entry. Wires
/// together the whole 5b→5i chain — aura, a Major Magical Focus, Method Caster,
/// Tough, Enduring Constitution, a weapon + shield + armor, a familiar with a
/// Bronze cord, a self-made Longevity Ritual, and stored warping / aging points —
/// then calls `derived_totals` and asserts every play-stat block is populated and
/// internally self-consistent. Numbers are pinned to the shipped catalogue values
/// (verified against `rules/core/*.json`).
#[test]
fn full_magus_derived_totals_are_populated_and_consistent() {
    let rs = load_full_ruleset();
    let mut e = entity(
        "magus",
        vec![
            Selection::new(Id::new("virtue.the_gift")),
            Selection::new(Id::new("virtue.hermetic_magus")),
            Selection::new(Id::new("virtue.method_caster")),
            Selection::new(Id::new("virtue.tough")),
            Selection::new(Id::new("virtue.enduring_constitution")),
            // A Major Magical Focus with a free-text descriptor ("fire").
            Selection::with_params(
                Id::new("virtue.major_magical_focus"),
                BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
            ),
        ],
    );

    // Characteristics.
    for (c, v) in [
        (Characteristic::Int, 3),
        (Characteristic::Sta, 2),
        (Characteristic::Str, 2),
        (Characteristic::Qik, 1),
        (Characteristic::Dex, 2),
    ] {
        e.characteristics.insert(c, v);
    }

    // Abilities the totals read (Magic Theory, Parma, Penetration, Single Weapon).
    let ab = |id: &str, score: u8| AbilityScore {
        ability: Id::new(id),
        parameter: None,
        specialty: None,
        score,
    };
    e.ability_scores = vec![
        ab("ability.magic_theory", 4),
        ab("ability.parma_magica", 3),
        ab("ability.penetration", 2),
        ab("ability.single_weapon", 4),
    ];

    // Arts: Creo 10, Ignem 8, Corpus 12.
    let art = |id: &str, score: u8| ArtScore {
        art: Id::new(id),
        score,
    };
    e.art_scores = vec![
        art("art.creo", 10),
        art("art.ignem", 8),
        art("art.corpus", 12),
    ];

    e.aura = 3;
    e.spells = vec![SpellSelection {
        spell: Id::new("spell.blade_of_the_virulent_flame"),
        level: None,
        mastery: None,
        parameter: None,
        mastery_abilities: Vec::new(),
    }];
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("weapon.axe"),
            equipped: true,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("shield.round"),
            equipped: true,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("armor.chain_mail_partial"),
            equipped: true,
            specialization_applies: false,
        },
    ];
    // A full familiar statblock: a raven (Size -4, ArMDE:17829-17856) with Magic
    // Might 10, human intelligence at Int -3 (ArMDE:10854), the bond's Loyal
    // (partner) +3 entered by hand, and one power invested in the bond.
    e.familiar = Some(Familiar {
        name: "Corvus".to_string(),
        animal: "raven".to_string(),
        might: Some(MightScore {
            realm: Realm::Magic,
            score: 10,
        }),
        characteristics: BTreeMap::from([(Characteristic::Int, -3), (Characteristic::Qik, 4)]),
        size: -4,
        personality_traits: vec![PersonalityTrait {
            name: "Loyal (Marcus)".to_string(),
            value: 3,
        }],
        cord_gold: 1,
        cord_silver: 1,
        cord_bronze: 2,
        powers: vec![SupernaturalPower {
            name: "Mental communication".to_string(),
            level: 20,
            penetration: 0,
        }],
    });
    e.talisman = Some(Talisman {
        description: "An ash staff shod with silver".to_string(),
        attunements: vec![TalismanAttunement {
            description: "Controlling things at a distance".to_string(),
            bonus: 4,
        }],
        effects: vec![TalismanEffect {
            name: "Wielding the Invisible Sling".to_string(),
            level: 15,
        }],
    });
    e.longevity_ritual = Some(LongevityRitual {
        source: LongevitySource::SelfMade,
        bonus: Some(6),
        focus: "A tincture of gold sipped each Midsummer".to_string(),
    });
    e.warping_points = 15;
    // Decrepitude is the SUM of aging points across Characteristics; the drops they
    // force now lower the effective Characteristic (derived from aging_points), so
    // they are placed in Per/Com — which no asserted magic/combat total reads — to
    // exercise Decrepitude without perturbing those totals. 10 + 7 = 17 → Decrepitude 2.
    e.aging_points = BTreeMap::from([(Characteristic::Per, 10), (Characteristic::Com, 7)]);

    let d = arm_rules::derived_totals(&e, &rs);

    // --- Magic totals present (magus) ------------------------------------
    assert!(d.hermetically_trained, "magus profile drives magic totals");
    assert!(!d.lab_totals.is_empty(), "lab totals populated");
    assert!(!d.casting_totals.is_empty(), "casting totals populated");

    // Lab Total (Creo, Corpus) = Int 3 + Magic Theory 4 + Creo 10 + Corpus 12 + Aura 3 = 32.
    let cr_co = d
        .lab_totals
        .iter()
        .find(|l| l.technique.as_str() == "art.creo" && l.form.as_str() == "art.corpus")
        .expect("Creo/Corpus lab cell present");
    assert_eq!(cr_co.total, 32, "Creo+Corpus Lab Total");

    // Casting (Creo, Ignem) formulaic = 10 + 8 + Sta 2 − Enc 1 + Aura 3 + Method Caster 3 = 25.
    let cr_ig = d
        .casting_totals
        .iter()
        .find(|c| c.technique.as_str() == "art.creo" && c.form.as_str() == "art.ignem")
        .expect("Creo/Ignem casting cell present");
    assert_eq!(cr_ig.formulaic, 25, "Creo+Ignem formulaic casting total");
    // Within the focus, the lower applicable Art (Ignem 8) is added again → 33.
    let wf = cr_ig.within_focus.as_ref().expect("focus present on cell");
    assert_eq!(
        wf.formulaic, 33,
        "within-focus adds the lower Art (Ignem 8)"
    );

    // Encumbrance: Load 7 (axe 1 + round shield 2 + partial chain 4) → Burden 3; Str 2 → 1.
    assert_eq!(d.encumbrance.total, 1, "Encumbrance = Burden 3 − Str 2");

    // Per-Form Magic Resistance (Ignem) = Form 8 + 5 × Parma 3 = 23.
    let mr_ig = d
        .magic_resistance
        .iter()
        .find(|m| m.form.as_str() == "art.ignem")
        .expect("Ignem magic resistance present");
    assert_eq!(mr_ig.total, 23, "Ignem MR = Form + 5×Parma");

    // Penetration for the known spell = casting total − level 15 + Penetration 2.
    let pen = d
        .penetration
        .iter()
        .find(|p| p.spell.as_str() == "spell.blade_of_the_virulent_flame")
        .expect("penetration line for the known spell");
    assert_eq!(
        pen.total,
        cr_ig.formulaic - 15 + 2,
        "penetration self-consistent"
    );
    assert_eq!(pen.total, 12);

    // Combat line for the axe, with the round shield's mods combined in.
    let axe = d
        .combat
        .iter()
        .find(|c| c.weapon.as_str() == "weapon.axe")
        .expect("axe combat line present");
    assert_eq!(axe.initiative, 1, "Init = Qik 1 + wpn 1 + shield 0 − Enc 1");
    assert_eq!(
        axe.attack,
        Some(10),
        "Attack = Dex 2 + Ability 4 + wpn 4 + shield 0"
    );
    assert_eq!(
        axe.defense, 7,
        "Defense = Qik 1 + Ability 4 + wpn 0 + shield 2"
    );
    assert_eq!(axe.damage, Some(8), "Damage = Str 2 + wpn 6");

    // Soak = Sta 2 + Armor 6 + Tough 3 + Bronze cord 2 = 13.
    assert_eq!(d.soak.total, 13, "Soak = Sta + Armor + Tough + Bronze cord");

    // Fatigue and wound tracks populated; Enduring Constitution eases the wound penalty.
    assert!(!d.fatigue.is_empty(), "fatigue levels populated");
    assert!(!d.wounds.is_empty(), "wound ranges populated");

    // Longevity: the *entered* bonus is reported verbatim (6), not the 7 that
    // today's Creo Corpus Lab Total of 32 would suggest — the ritual is a frozen
    // past event. The hint carries the suggestion; the bronze cord is surfaced.
    let lon = d
        .longevity
        .expect("longevity present for a magus with a ritual");
    assert_eq!(lon.bonus, 6, "the entered bonus, verbatim");
    assert!(lon.entered);
    assert_eq!(lon.bronze_cord, 2);
    let hint = lon.hint.expect("self-made rituals get a hint");
    assert_eq!(hint.lab_total, 32, "CrCo Lab Total on the real ruleset");
    assert_eq!(hint.suggested_bonus, 7, "ceil(32/5)");
    assert!(!hint.halved);

    // Talisman capacity on the real ruleset = highest Technique (Creo 10) + highest
    // Form (Corpus 12) = 22 pawns of Vim vis (ArMDE:10619). Ignem 8 loses to Corpus.
    let capacity = d
        .talisman_capacity
        .expect("capacity present for a magus with a talisman");
    assert_eq!(capacity.technique.as_str(), "art.creo");
    assert_eq!(capacity.form.as_str(), "art.corpus");
    assert_eq!(capacity.technique_score, 10);
    assert_eq!(capacity.form_score, 12);
    assert_eq!(capacity.pawns, 22);

    // Non-goal guard: the level-15 instilled effect is charged against NOTHING. The
    // item-level budget belongs to the Redcap-only Virtues (ArMDE:4347-4349,
    // :4842-4850), and a Redcap "may not take The Gift" (ArMDE:4850), so it can never
    // fund a magus's talisman. This magus has no such Virtue, so both figures are 0
    // even though he owns a talisman holding 15 levels of effect.
    assert_eq!(
        arm_rules::item_level_used(&e),
        0,
        "talisman effects must not be swept into item_level_used"
    );
    assert_eq!(arm_rules::item_level_budget(&e, &rs), 0);
    assert!(
        validate(&e, &rs)
            .issues
            .iter()
            .all(|i| i.code != "over_item_level"),
        "a talisman effect must raise no item-level issue"
    );

    // Familiar bonding read-out on the real ruleset. Binding level = Magic Might 10
    // + 25 + 5 × Size(-4) = 15 (ArMDE:10824, :10828) — the negative Size takes 20
    // points off. Cords 1/1/2 cost 5 + 5 + 15 = 25 off the curve (ArMDE:10836), which
    // fits inside the best bonding Lab Total. Invested powers total 20 levels and
    // are charged against nothing (ArMDE:10866).
    let fam = d
        .familiar
        .expect("read-out present for a magus with a familiar");
    assert_eq!(fam.binding_level, 15, "Might 10 + 25 + 5 x -4");
    assert_eq!(fam.cord_points_spent, 25, "5 + 5 + 15 off the cord curve");
    assert_eq!(fam.invested_power_levels, 20);
    // The best (Te,Fo) cell is the same Creo/Corpus pair the capacity names, so the
    // bonding Lab Total matches the Creo Corpus Lab Total the longevity hint used.
    assert_eq!(fam.binding.technique.as_str(), "art.creo");
    assert_eq!(fam.binding.form.as_str(), "art.corpus");
    assert_eq!(fam.binding.lab_total, 32);
    // This magus holds a Major Magical Focus, and :10818 lets a focus apply to the
    // bonding Lab Total — so the conditional figure is present (base + lower Art).
    assert_eq!(fam.binding.lab_total_within_focus, Some(42));
    assert!(fam.binding.lab_total_reaches_level, "32 >= 15");
    assert!(fam.binding.cord_points_within_lab_total, "25 <= 32");

    // Guidance-only guard: nothing the familiar carries raises an issue, not even
    // its Faerie-capable Might, its own Characteristics, or 20 levels of invested
    // power. Compare against the same magus with no familiar at all.
    let mut without = e.clone();
    without.familiar = None;
    let codes = |x: &Entity| -> Vec<String> {
        validate(x, &rs)
            .issues
            .iter()
            .map(|i| i.code.clone())
            .collect()
    };
    assert_eq!(
        codes(&e),
        codes(&without),
        "no familiar read-out may produce a ValidationIssue"
    );

    // Warping Score 2 from 15 stored points; Decrepitude 2 from 17 aging points.
    assert_eq!(d.warping_points, 15);
    assert_eq!(d.warping_score, 2, "15 points → Warping Score 2");
    assert_eq!(d.decrepitude_score, 2, "17 aging points → Decrepitude 2");
}

/// A magus with a Creo Corpus Lab Total of 35, built on the **real shipped
/// ruleset**, gets the book's own worked example back: "Longevity Ritual: Lab Total
/// 35, +7 aging bonus".
/// Source: `ArMDE:2573` (the sheet line),
/// `ArMDE:2488` (the same magus's lab season), `ArMDE:10662` (the formula).
#[test]
fn longevity_hint_reproduces_the_books_lab_total_35_example() {
    let rs = load_full_ruleset();
    let mut e = entity(
        "magus",
        vec![
            Selection::new(Id::new("virtue.the_gift")),
            Selection::new(Id::new("virtue.hermetic_magus")),
        ],
    );
    // Int 3 + Magic Theory 4 + Creo 10 + Corpus 13 + Aura 5 = 35.
    e.characteristics.insert(Characteristic::Int, 3);
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.magic_theory"),
        parameter: None,
        specialty: None,
        score: 4,
    }];
    e.art_scores = vec![
        ArtScore {
            art: Id::new("art.creo"),
            score: 10,
        },
        ArtScore {
            art: Id::new("art.corpus"),
            score: 13,
        },
    ];
    e.aura = 5;
    e.longevity_ritual = Some(LongevityRitual {
        source: LongevitySource::SelfMade,
        bonus: None,
        focus: String::new(),
    });

    let lon = arm_rules::derived_totals(&e, &rs)
        .longevity
        .expect("longevity present");
    assert!(!lon.entered, "nothing entered yet");
    assert_eq!(lon.bonus, 0, "placeholder, not a claim");
    let hint = lon.hint.expect("self-made rituals get a hint");
    assert_eq!(hint.lab_total, 35);
    assert_eq!(hint.suggested_bonus, 7);

    // A zero aura does not suppress the suggestion — the Aura Modifier is a plain
    // addend, and no aura simply means no hindrance (ArMDE:10276-10278, :17658).
    // The removed `aura != 0` gate, guarded on the real ruleset.
    e.aura = 0;
    let hint = arm_rules::derived_totals(&e, &rs)
        .longevity
        .expect("longevity present")
        .hint
        .expect("a zero aura still gets a hint");
    assert_eq!(hint.lab_total, 30);
    assert_eq!(hint.suggested_bonus, 6, "ceil(30/5), not 0");

    // Difficult Longevity Ritual halves it on the shipped catalogue too
    // (ArMDE:5962-5964): 30 → 15 → ceil(15/5) = 3.
    e.selections
        .push(Selection::new(Id::new("flaw.difficult_longevity_ritual")));
    let hint = arm_rules::derived_totals(&e, &rs)
        .longevity
        .expect("longevity present")
        .hint
        .expect("has a hint");
    assert_eq!(hint.lab_total, 15);
    assert_eq!(hint.suggested_bonus, 3);
    assert!(hint.halved, "the Flaw's halving is flagged for the UI");
}

/// The shipped Aging tables, read as text. `load_full_ruleset` feeds these bytes
/// to the loader exactly as `ruleset_io.rs` does, so the row-by-row tests below
/// read the tables back off the ruleset rather than deserializing them
/// themselves.
const SHIPPED_AGING: &str = include_str!("../../../rules/core/aging.json");
const SHIPPED_AGING_EN: &str = include_str!("../../../rules/i18n/en/aging.json");
const SHIPPED_AGING_DE: &str = include_str!("../../../rules/i18n/de/aging.json");

fn shipped_aging_rules() -> AgingRules {
    load_full_ruleset()
        .aging()
        .expect("the shipped ruleset carries the aging tables")
        .clone()
}

/// The whole of `## Aging`'s two tables, transcribed row by row: the scalars of
/// `ArMDE:16565`-`ArMDE:16577`, the ten Living Conditions of `ArMDE:16583-16592` (five of them
/// asterisked, i.e. cumulative — `ArMDE:16594`) and the eleven Aging Roll outcomes of
/// `ArMDE:16601-16611`.
///
/// The row values are deliberately **literals** here: nothing else in the engine
/// can witness a mis-transcribed modifier or a swapped Characteristic, since the
/// JSON is the only place those numbers live.
///
/// Source: ArMDE:16563-16615.
#[test]
fn shipped_aging_table_carries_the_16583_to_16611_rows() {
    let rules = shipped_aging_rules();

    // Stated outright rather than left to `load_full_ruleset`'s unwrap: the
    // shipped tables clear every load-time gate of `validate_aging_rules` —
    // contiguous rows up to an open-ended top one, a clamp below the first
    // aging-point row (ArMDE:16575), and no duplicate Living Condition id.
    load_full_ruleset()
        .validate_integrity()
        .expect("the shipped aging tables pass every load-time gate");

    // "Characters begin aging in the Winter after they turn 35" (ArMDE:16565), the
    // "age/10 (round up)" term (ArMDE:16567) and the apparent-aging threshold, which
    // is a question asked of every total rather than a row: "2 or less — No
    // apparent aging" / "3 or more — Apparent age increases by one year"
    // (ArMDE:16599-16600, :16577).
    assert_eq!(rules.start_age, 35);
    assert_eq!(rules.age_divisor, 10);
    assert_eq!(rules.apparent_age_increase_min, 3);
    // "treats all rolls of 10 or more as rolls of 9 until he reaches the age of
    // 35" (ArMDE:16575).
    let clamp = rules
        .longevity_clamp
        .clone()
        .expect("the :16575 clamp ships");
    assert_eq!(clamp.max_total, 9);
    assert_eq!(clamp.until_age, 35);

    let conditions: Vec<(&str, i8)> = rules
        .living_conditions
        .iter()
        .map(|row| (row.id.as_str(), row.modifier))
        .collect();
    assert_eq!(
        conditions,
        vec![
            ("living_condition.average_peasant", 0),
            ("living_condition.leper", -2),
            ("living_condition.live_in_a_leper_colony", -1),
            (
                "living_condition.poor_or_unhealthy_location_typical_town",
                -2
            ),
            (
                "living_condition.typical_spring_or_winter_covenant_magus",
                1
            ),
            (
                "living_condition.typical_summer_or_autumn_covenant_magus",
                2
            ),
            (
                "living_condition.typical_summer_or_autumn_covenant_mundane",
                1
            ),
            ("living_condition.wealthy_or_healthy_location", 2),
            ("living_condition.work_in_a_bad_air_trade", -1),
            ("living_condition.work_in_a_mine", -1),
        ],
        "the ten rows of :16583-16592, id-sorted"
    );

    // "\\* Modifiers marked with an asterisk are cumulative with each other."
    // (ArMDE:16594) — FIVE rows carry it, the three occupational -1s and both -2s.
    let cumulative: Vec<&str> = rules
        .living_conditions
        .iter()
        .filter(|row| row.cumulative)
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(
        cumulative,
        vec![
            "living_condition.leper",
            "living_condition.live_in_a_leper_colony",
            "living_condition.poor_or_unhealthy_location_typical_town",
            "living_condition.work_in_a_bad_air_trade",
            "living_condition.work_in_a_mine",
        ],
        "exactly the five asterisked rows :16588-16592 are cumulative"
    );

    // Every row points at its own line of the table, so a re-transcription can be
    // checked against the source one row at a time.
    let condition_lines: Vec<(&str, u32, u32)> = rules
        .living_conditions
        .iter()
        .map(|row| {
            let source = row.source.as_ref().expect("every row carries provenance");
            assert_eq!(
                source.file,
                "Ars Magica - Definitive Edition (Core Rules).md"
            );
            (row.id.as_str(), source.lines.start, source.lines.end)
        })
        .collect();
    assert_eq!(
        condition_lines,
        vec![
            ("living_condition.average_peasant", 16587, 16587),
            ("living_condition.leper", 16592, 16592),
            ("living_condition.live_in_a_leper_colony", 16588, 16588),
            (
                "living_condition.poor_or_unhealthy_location_typical_town",
                16591,
                16591
            ),
            (
                "living_condition.typical_spring_or_winter_covenant_magus",
                16586,
                16586
            ),
            (
                "living_condition.typical_summer_or_autumn_covenant_magus",
                16584,
                16584
            ),
            (
                "living_condition.typical_summer_or_autumn_covenant_mundane",
                16585,
                16585
            ),
            ("living_condition.wealthy_or_healthy_location", 16583, 16583),
            ("living_condition.work_in_a_bad_air_trade", 16589, 16589),
            ("living_condition.work_in_a_mine", 16590, 16590),
        ]
    );

    // The Aging Roll table (ArMDE:16601-16611): eleven effect rows, ascending, the
    // last one open-ended ("22+").
    let any = |points| AgingRowEffect::AnyCharacteristic { points };
    let named = |characteristics: Vec<Characteristic>| AgingRowEffect::NamedCharacteristics {
        points: 1,
        characteristics,
    };
    let crisis = AgingRowEffect::NextDecrepitudeLevelAndCrisis;
    let outcomes: Vec<(i32, Option<i32>, &AgingRowEffect, u32)> = rules
        .outcomes
        .iter()
        .map(|row| {
            let source = row.source.as_ref().expect("every row carries provenance");
            assert_eq!(
                source.file,
                "Ars Magica - Definitive Edition (Core Rules).md"
            );
            assert_eq!(
                source.lines.start, source.lines.end,
                "a table row spans one line"
            );
            (row.min, row.max, &row.effect, source.lines.start)
        })
        .collect();
    assert_eq!(
        outcomes,
        vec![
            (10, Some(12), &any(1), 16601),
            (13, Some(13), &crisis, 16602),
            (14, Some(14), &named(vec![Characteristic::Qik]), 16603),
            (15, Some(15), &named(vec![Characteristic::Sta]), 16604),
            (16, Some(16), &named(vec![Characteristic::Per]), 16605),
            // "1 Aging Point in Prs" (ArMDE:16606) — the table's abbreviation for
            // Presence, which the engine spells `pre`.
            (17, Some(17), &named(vec![Characteristic::Pre]), 16606),
            (
                18,
                Some(18),
                &named(vec![Characteristic::Str, Characteristic::Sta]),
                16607
            ),
            (
                19,
                Some(19),
                &named(vec![Characteristic::Dex, Characteristic::Qik]),
                16608
            ),
            (
                20,
                Some(20),
                &named(vec![Characteristic::Com, Characteristic::Pre]),
                16609
            ),
            (
                21,
                Some(21),
                &named(vec![Characteristic::Int, Characteristic::Per]),
                16610
            ),
            (22, None, &crisis, 16611),
        ]
    );

    // The table's structural signature: over the eight rows that name
    // Characteristics (ArMDE:16603-16610), the four "physical/social pairs" halves
    // Qik, Sta, Per and Prs each appear twice — once alone, once paired — and
    // Str, Dex, Com and Int exactly once each. A row transcribed with the wrong
    // Characteristic breaks this even if every band still looks plausible.
    let mut tally: BTreeMap<Characteristic, usize> = BTreeMap::new();
    for row in rules.outcomes.iter().filter(|r| (14..=21).contains(&r.min)) {
        let AgingRowEffect::NamedCharacteristics {
            characteristics, ..
        } = &row.effect
        else {
            panic!("rows 14-21 all name their Characteristics: {row:?}");
        };
        for c in characteristics {
            *tally.entry(*c).or_default() += 1;
        }
    }
    assert_eq!(
        tally,
        BTreeMap::from([
            (Characteristic::Int, 1),
            (Characteristic::Per, 2),
            (Characteristic::Pre, 2),
            (Characteristic::Com, 1),
            (Characteristic::Str, 1),
            (Characteristic::Sta, 2),
            (Characteristic::Dex, 1),
            (Characteristic::Qik, 2),
        ])
    );
}

/// One row of the Crisis Table as the transcription below compares it: the row
/// id, its inclusive total range, the outcome it names, and the source line it
/// was taken from. Named rather than written inline because the tuple is what the
/// whole transcription is asserted against, and an anonymous five-field tuple in
/// the assertion says nothing about which field is which.
type CrisisTableRow<'a> = (&'a str, Option<i32>, Option<i32>, &'a CrisisOutcome, u32);

/// The Crisis Table transcribed row by row (`ArMDE:16626-16632`), together with the
/// Simple Die it is rolled on (`ArMDE:474`), the attending doctor (`ArMDE:16634`) and the
/// two Decrepitude thresholds of `ArMDE:16617`.
///
/// The values are deliberately **literals**, for the same reason the Aging Roll
/// transcription above uses them: the shipped JSON is the only place an Ease
/// Factor or a Ritual level lives, so nothing else in the engine can witness a
/// mis-transcribed one.
///
/// Source: ArMDE:474, :16617-16634.
#[test]
fn shipped_crisis_table_carries_the_16626_to_16632_rows() {
    let rules = shipped_aging_rules();
    let crisis = rules.crisis.clone().expect("the shipped crisis table");

    // Stated outright rather than left to `load_full_ruleset`'s unwrap: the
    // shipped Crisis Table clears every load-time gate of `validate_crisis_rules`
    // — rows tiling contiguously between an open-below and an open-above end, an
    // illness ladder whose severity, Ritual level and Ease Factor all climb
    // together (ArMDE:16638), an attendant Ability that resolves, and a frail
    // Decrepitude score below the fatal one (ArMDE:16617).
    load_full_ruleset()
        .validate_integrity()
        .expect("the shipped crisis table passes every load-time gate");

    // "Characters with a Decrepitude score of 4 are extremely frail, and must
    // roll on the Crisis Table … Characters with a Decrepitude score of 5 are
    // bedridden and will die within a few months at most." (ArMDE:16617)
    assert_eq!(rules.frail_decrepitude_score, Some(4));
    assert_eq!(rules.fatal_decrepitude_score, Some(5));

    // "Roll a ten-sided die. Each number counts for its value, except that a zero
    // counts as ten." (ArMDE:474) — the CRISIS TOTAL's Simple die (ArMDE:16621).
    let die = crisis.die.clone().expect("the Simple Die of :474 ships");
    assert_eq!((die.min, die.max), (1, 10));
    let die_source = die.source.expect("the die carries provenance");
    assert_eq!(
        die_source.file,
        "Ars Magica - Definitive Edition (Core Rules).md"
    );
    assert_eq!((die_source.lines.start, die_source.lines.end), (474, 474));

    // "An Int + Medicine roll against an Ease Factor of 6 allows the character to
    // add the attendant's Medicine score to the roll to survive the crisis. …
    // if the doctor botches the character must subtract 3 from the survival
    // roll." (ArMDE:16634) — the penalty is stored signed, as the roll takes it.
    let attendant = crisis
        .attendant
        .clone()
        .expect("the attendant of :16634 ships");
    assert_eq!(attendant.ability.as_str(), "ability.medicine");
    assert_eq!(attendant.characteristic, Characteristic::Int);
    assert_eq!(attendant.ease_factor, 6);
    assert_eq!(attendant.botch_penalty, -3);
    let attendant_source = attendant.source.expect("the attendant carries provenance");
    assert_eq!(
        attendant_source.file,
        "Ars Magica - Definitive Edition (Core Rules).md"
    );
    assert_eq!(
        (attendant_source.lines.start, attendant_source.lines.end),
        (16634, 16634)
    );

    let bedridden = CrisisOutcome::Bedridden;
    let illness = |severity, ease_factor, ritual_level| CrisisOutcome::Illness {
        severity,
        ease_factor,
        ritual_level,
    };
    let rows: Vec<CrisisTableRow<'_>> = crisis
        .rows
        .iter()
        .map(|row| {
            let source = row.source.as_ref().expect("every row carries provenance");
            assert_eq!(
                source.file,
                "Ars Magica - Definitive Edition (Core Rules).md"
            );
            assert_eq!(
                source.lines.start, source.lines.end,
                "a table row spans one line"
            );
            (
                row.id.as_str(),
                row.min,
                row.max,
                &row.outcome,
                source.lines.start,
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            // "8 or less — Bedridden for a week" (ArMDE:16626): open below, so no
            // `min` at all rather than a very small one.
            ("crisis.bedridden_week", None, Some(8), &bedridden, 16626),
            (
                "crisis.bedridden_month",
                Some(9),
                Some(14),
                &bedridden,
                16627
            ),
            (
                "crisis.minor_illness",
                Some(15),
                Some(15),
                &illness(CrisisSeverity::Minor, Some(3), 20),
                16628
            ),
            (
                "crisis.serious_illness",
                Some(16),
                Some(16),
                &illness(CrisisSeverity::Serious, Some(6), 25),
                16629
            ),
            (
                "crisis.major_illness",
                Some(17),
                Some(17),
                &illness(CrisisSeverity::Major, Some(9), 30),
                16630
            ),
            (
                "crisis.critical_illness",
                Some(18),
                Some(18),
                &illness(CrisisSeverity::Critical, Some(12), 35),
                16631
            ),
            // "19+ — **Terminal illness**. CrCo40 required to survive."
            // (ArMDE:16632): open above, and no Stamina roll at all — hence no Ease
            // Factor rather than an unbeatable one.
            (
                "crisis.terminal_illness",
                Some(19),
                None,
                &illness(CrisisSeverity::Terminal, None, 40),
                16632
            ),
        ]
    );

    // The table's structural signature. `crisis.rows` is the ONE array in
    // `rules/core` that ships in band order rather than id order (see RULES.md):
    // the rows ascend by the totals they cover, open below at the top of the
    // table and open above at the bottom. An id-alphabetical sort would leave
    // every value above intact and still break this.
    let first = crisis.rows.first().expect("the crisis table has rows");
    let last = crisis.rows.last().expect("the crisis table has rows");
    assert!(
        first.min.is_none(),
        "the first row is open below: \"8 or less\" (ArMDE:16626)"
    );
    assert!(
        last.max.is_none(),
        "the last row is open above: \"19+\" (ArMDE:16632)"
    );
    for pair in crisis.rows.windows(2) {
        let ceiling = pair[0].max.expect("only the last row is open above");
        let floor = pair[1].min.expect("only the first row is open below");
        assert!(
            ceiling < floor,
            "the rows ascend by band: '{}' ends at {ceiling}, '{}' starts at {floor}",
            pair[0].id,
            pair[1].id
        );
    }

    // "The level of spell required depends on the severity of the crisis, as
    // noted on the table." (ArMDE:16638) — so severity is a ladder that climbs with
    // the band, and the Ritual level climbs with it.
    let illnesses: Vec<(CrisisSeverity, u32)> = crisis
        .rows
        .iter()
        .filter_map(|row| match &row.outcome {
            CrisisOutcome::Illness {
                severity,
                ritual_level,
                ..
            } => Some((*severity, *ritual_level)),
            CrisisOutcome::Bedridden => None,
        })
        .collect();
    assert!(
        illnesses
            .windows(2)
            .all(|pair| pair[0].0 < pair[1].0 && pair[0].1 < pair[1].1),
        "the illness rows climb in severity and Ritual level with the band: {illnesses:?}"
    );
}

/// The AGING TOTAL of a 40-year-old carrying the named shipped items, rolling a
/// 6: `6 + ceil(40/10)` = 10 before any modifier.
fn shipped_aging_total(items: &[&str]) -> AgingTotal {
    let rs = load_full_ruleset();
    let e = entity(
        "companion",
        items
            .iter()
            .map(|id| Selection::new(Id::new(*id)))
            .collect(),
    );
    arm_rules::aging::aging_total(&e, &rs, 40, 6).expect("the shipped ruleset carries aging rules")
}

/// Faerie Blood: "You are resistant to aging, and get -1 to all aging rolls."
/// (ArMDE:3801) — the shipped `aging_roll -1` is ADDED with its stored sign, so
/// the total drops by one.
#[test]
fn faerie_blood_lowers_the_aging_total_by_one() {
    assert_eq!(
        shipped_aging_total(&[]).total,
        10,
        "the unmodified baseline"
    );

    let faerie = shipped_aging_total(&["virtue.faerie_blood"]);
    assert_eq!(faerie.trait_modifier, -1);
    assert_eq!(faerie.total, 9);
}

/// Strong Faerie Blood: "You start making aging rolls at the age of fifty, rather
/// than the normal 35, and get -3 to Aging Rolls, cumulative with any other
/// bonuses." (ArMDE:5036)
///
/// Only the -3 half is implemented. The start-at-fifty half needs a per-trait
/// override of [`arm_rules::AgingRules::start_age`] — machinery no other shipped
/// item asks for — so this character is still scheduled from 36, which is a known
/// gap rather than a reading of the text.
#[test]
fn strong_faerie_blood_lowers_the_aging_total_by_three() {
    let strong = shipped_aging_total(&["virtue.strong_faerie_blood"]);
    assert_eq!(strong.trait_modifier, -3);
    assert_eq!(strong.total, 7);
}

/// The `aging_mod` kinds an item ships, sorted so the assertion does not depend
/// on the order the effects happen to sit in the file.
fn aging_kinds(id: &str) -> Vec<(AgingEffect, i8)> {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new(id))
        .unwrap_or_else(|| panic!("the shipped catalogue carries '{id}'"));
    let mut kinds: Vec<(AgingEffect, i8)> = item
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::AgingMod { kind, amount } => Some((*kind, *amount)),
            _ => None,
        })
        .collect();
    kinds.sort_unstable();
    kinds
}

/// Mild Aging states **two** mechanics in one sentence, and they go to two
/// different places:
///
/// > "The character's aging rolls benefit from a +1 bonus to the Living
/// > Conditions Modifier, in addition to whatever his social standing normally
/// > offers him. Furthermore, he receives a +3 bonus to rolls to survive an aging
/// > crisis." (ArMDE:4530)
///
/// The +1 is a Living Conditions term of the AGING TOTAL; the +3 belongs to the
/// crisis *survival* roll, which `ArMDE:16636` otherwise walls off from aging-roll
/// modifiers entirely. Only the first half shipped until 6b7, so the Virtue read
/// as half a rule. Both halves now ship, and this test is the witness that a
/// later sweep does not drop one again.
///
/// Source: ArMDE:4530, :16636.
#[test]
fn mild_aging_carries_both_halves_of_4530() {
    assert_eq!(
        aging_kinds("virtue.mild_aging"),
        vec![
            (AgingEffect::LivingConditions, 1),
            (AgingEffect::CrisisSurvival, 3),
        ],
    );
}

/// The [`CrisisOutcome`] of one named row of the **shipped** Crisis Table — what
/// a look-up would hand `crisis_survival`, fetched by id so the test does not
/// have to invent a total to reach the row.
fn shipped_crisis_outcome(id: &str) -> CrisisOutcome {
    shipped_aging_rules()
        .crisis
        .as_ref()
        .expect("the shipped crisis table")
        .rows
        .iter()
        .find(|row| row.id.as_str() == id)
        .unwrap_or_else(|| panic!("the shipped Crisis Table carries '{id}'"))
        .outcome
        .clone()
}

/// The crisis-survival read-out for a companion carrying the named shipped
/// items, against the shipped Minor illness row (`ArMDE:16628`).
fn shipped_crisis_survival(items: &[&str]) -> CrisisSurvival {
    let rs = load_full_ruleset();
    let e = entity(
        "companion",
        items
            .iter()
            .map(|id| Selection::new(Id::new(*id)))
            .collect(),
    );
    arm_rules::aging::crisis_survival(&e, &rs, &shipped_crisis_outcome("crisis.minor_illness"))
        .expect("an illness is a roll to describe")
}

/// **"Virtues that affect aging rolls do not affect crisis survival rolls."**
/// (ArMDE:16636) — the single load-bearing sentence of the survival read-out,
/// locked against the shipped catalogue rather than a fixture.
///
/// Three shipped items carry the two kinds the aging roll takes, and all three
/// are witnessed moving the AGING TOTAL and then contributing **nothing** to the
/// survival roll:
///
/// - Faerie Blood, `aging_roll -1` (`ArMDE:3801`)
/// - Strong Faerie Blood, `aging_roll -3` (`ArMDE:5036`)
/// - Poor Living Conditions, `living_conditions -1` (`ArMDE:6620`)
///
/// Mild Aging is the proof case, because `ArMDE:4530` grants both sides in one
/// sentence: "The character's aging rolls benefit from a +1 bonus to the Living
/// Conditions Modifier … Furthermore, he receives a +3 bonus to rolls to survive
/// an aging crisis." The +1 stays out of the crisis; the +3 goes in, alone.
///
/// An implementation that simply summed every `aging_mod` amount would read -5
/// on the first character and -1 on the second, so this test is not satisfiable
/// by accident.
///
/// Source: ArMDE:3801, :4530, :5036,
/// :6620, :16636.
#[test]
fn virtues_that_modify_aging_rolls_do_not_affect_crisis_survival_rolls() {
    let aging_movers = [
        "virtue.faerie_blood",
        "virtue.strong_faerie_blood",
        "flaw.poor_living_conditions",
    ];

    // They demonstrably move the AGING TOTAL: -1 and -3 on the trait modifier,
    // -1 on the Living Conditions term.
    let aging = shipped_aging_total(&aging_movers);
    assert_eq!(aging.trait_modifier, -4);
    assert_eq!(aging.living_conditions.total, -1);

    // …and contribute nothing at all to the survival roll.
    let survival = shipped_crisis_survival(&aging_movers);
    assert_eq!(survival.modifiers, vec![]);
    assert_eq!(survival.modifier_total, 0);

    // Mild Aging's +3 is a grant to *this* roll by name, so it does arrive — and
    // it arrives alone, itemized under the id the UI resolves to a label.
    let mut with_mild = aging_movers.to_vec();
    with_mild.push("virtue.mild_aging");
    let survival = shipped_crisis_survival(&with_mild);
    assert_eq!(
        survival.modifiers,
        vec![CrisisModifier {
            source: CrisisModifierSource::Trait {
                item: Id::new("virtue.mild_aging"),
            },
            amount: 3,
        }],
        "only :4530's crisis-survival half crosses the :16636 wall"
    );
    assert_eq!(survival.modifier_total, 3);

    // The Living Conditions half is not lost, merely elsewhere: -1 from the Flaw
    // and +1 from Mild Aging cancel on the roll that takes them.
    assert_eq!(shipped_aging_total(&with_mild).living_conditions.total, 0);
}

/// "19+ — **Terminal illness**. CrCo40 required to survive." (ArMDE:16632) — the
/// one row that offers no Stamina roll at all. The read-out reports the Ritual
/// that resolves it (`ArMDE:16638`) and **no** Ease Factor, rather than an unbeatable
/// one; and the Minor row beside it shows the ordinary shape, Ease Factor 3 and
/// CrCo20 (`ArMDE:16628`).
///
/// Source: ArMDE:16628, :16632, :16638.
#[test]
fn the_terminal_row_reports_a_ritual_level_and_no_ease_factor() {
    let rs = load_full_ruleset();
    let e = entity("companion", vec![]);

    let terminal = arm_rules::aging::crisis_survival(
        &e,
        &rs,
        &shipped_crisis_outcome("crisis.terminal_illness"),
    )
    .expect("Terminal illness is still a crisis to describe");
    assert_eq!(
        terminal.ease_factor, None,
        "no Stamina roll is offered at 19+"
    );
    assert_eq!(terminal.ritual_level, 40);

    let minor =
        arm_rules::aging::crisis_survival(&e, &rs, &shipped_crisis_outcome("crisis.minor_illness"))
            .expect("Minor illness is survivable");
    assert_eq!(minor.ease_factor, Some(3));
    assert_eq!(minor.ritual_level, 20);
}

/// "An Int + Medicine roll against an Ease Factor of 6 allows the character to
/// add the attendant's Medicine score to the roll to survive the crisis. Only
/// one doctor may usefully attend a patient, and if the doctor botches the
/// character must subtract 3 from the survival roll." (ArMDE:16634)
///
/// The doctor is reported as an **allowance** — what the rules permit — and not
/// as a modifier, because the app has no attendant to score: the Medicine score
/// belongs to another character entirely. Every value comes off the ruleset, and
/// the botch penalty keeps the file's one sign convention (stored signed, added).
///
/// Source: ArMDE:16634.
#[test]
fn the_attending_doctor_is_reported_as_an_allowance() {
    let survival = shipped_crisis_survival(&[]);
    assert_eq!(
        survival.allowances,
        vec![CrisisAllowance::Attendant {
            ability: Id::new("ability.medicine"),
            characteristic: Characteristic::Int,
            ease_factor: 6,
            botch_penalty: -3,
        }],
        "one doctor, with the ruleset's own numbers"
    );
    // An allowance is never a term of the total the character brings.
    assert_eq!(survival.modifier_total, 0);
}

/// One Crisis walked end to end against the **shipped** table, through the
/// crate's public surface: a die and a year in, and the CRISIS TOTAL
/// (ArMDE:16621), the row it lands on (`ArMDE:16624-16632`), what that row costs and
/// what surviving it would take (`ArMDE:16628-16638`) out.
///
/// The fixture tests in `aging.rs` prove the composition; this proves it against
/// the real `rules/core/aging.json`, where the attendant of `ArMDE:16634` actually
/// ships — so the doctor reaches a caller through the composed read-out and not
/// only through a hand-built outcome.
///
/// Source: ArMDE:16621-16638.
#[test]
fn the_shipped_crisis_table_answers_a_total_end_to_end() {
    let rs = load_full_ruleset();
    let mut e = entity("companion", vec![]);
    e.age = Some(40);
    e.aging_points.insert(Characteristic::Sta, 15);

    // `9 + ⌈36/10⌉ + 2 = 15` — Minor illness, Ease Factor 3, CrCo20 (ArMDE:16628).
    let preview = arm_rules::crisis_preview(&e, &rs, 36, 9).expect("the shipped table");
    assert_eq!(preview.total.age_modifier, 4);
    assert_eq!(preview.total.decrepitude_score, 2);
    assert_eq!(preview.total.total, 15);
    assert_eq!(preview.row, Id::new("crisis.minor_illness"));
    assert_eq!(
        preview.outcome,
        CrisisOutcome::Illness {
            severity: CrisisSeverity::Minor,
            ease_factor: Some(3),
            ritual_level: 20,
        }
    );
    let survival = preview.survival.expect("an illness is survivable");
    assert_eq!(survival.ease_factor, Some(3));
    assert_eq!(survival.ritual_level, 20);
    assert_eq!(
        survival.allowances,
        vec![CrisisAllowance::Attendant {
            ability: Id::new("ability.medicine"),
            characteristic: Characteristic::Int,
            ease_factor: 6,
            botch_penalty: -3,
        }],
        "the doctor of :16634 reaches the composed read-out too"
    );

    // "8 or less — Bedridden for a week" (ArMDE:16626) is time, not a roll.
    let unaged = entity("companion", vec![]);
    let bedridden = arm_rules::crisis_preview(&unaged, &rs, 36, 4).expect("the shipped table");
    assert_eq!(bedridden.total.total, 8);
    assert_eq!(bedridden.row, Id::new("crisis.bedridden_week"));
    assert_eq!(bedridden.outcome, CrisisOutcome::Bedridden);
    assert!(bedridden.survival.is_none());

    // The shipped table's own bands, read by the look-up rather than by index.
    let landings: Vec<&str> = [8, 9, 14, 15, 16, 17, 18, 19, 99]
        .into_iter()
        .map(|total| {
            arm_rules::resolve_crisis_row(&rs, total)
                .unwrap_or_else(|| panic!("the shipped table covers {total}"))
                .id
                .as_str()
        })
        .collect();
    assert_eq!(
        landings,
        vec![
            "crisis.bedridden_week",
            "crisis.bedridden_month",
            "crisis.bedridden_month",
            "crisis.minor_illness",
            "crisis.serious_illness",
            "crisis.major_illness",
            "crisis.critical_illness",
            "crisis.terminal_illness",
            "crisis.terminal_illness",
        ]
    );
}

/// One Crisis **written into a character** against the shipped tables, and taken
/// back off again.
///
/// A 40-year-old companion rolls a 9: `9 + ⌈40/10⌉ = 13`, the row of `ArMDE:16602` that
/// reaches the next level in Decrepitude and sends him to the Crisis Table. Five
/// Aging Points is what the shipped advancement curve prices Decrepitude 1 at, and
/// `ArMDE:16619`'s "increase the character's Decrepitude first" is visible in the CRISIS
/// TOTAL: `10 + 4 + 1 = 15`, the **1** being the score this very year raised. That
/// lands on the shipped minor illness — Ease Factor 3, CrCo20, and the doctor of
/// `ArMDE:16634`, which only the real `rules/core/aging.json` ships.
///
/// The fixture tests in `aging.rs` prove the leg; this proves it against the
/// catalogue the app actually loads, and that the year still comes back off byte
/// for byte.
///
/// Source: ArMDE:16602, :16619, :16621,
/// :16628, :16634.
#[test]
fn a_shipped_crisis_year_is_written_into_the_character_and_reverts_exactly() {
    let rs = load_full_ruleset();
    let mut e = entity("companion", vec![]);
    e.age = Some(40);
    let before = serde_json::to_string(&e).expect("a character serializes");

    let request = arm_rules::AgingYearRequest {
        age: 40,
        die: 9,
        distribution: BTreeMap::from([(Characteristic::Sta, 5)]),
        crisis_die: Some(10),
    };
    let resolved = arm_rules::resolve_year(&e, &rs, &request).expect("a shipped crisis year");
    assert_eq!(resolved.total.total, 13);
    assert!(resolved.outcome.crisis);

    let crisis = resolved.crisis.as_ref().expect("the player rolled it");
    assert_eq!(
        crisis.total.decrepitude_score, 1,
        "the score this year raised, not the 0 he started it with"
    );
    assert_eq!(crisis.total.total, 15);
    assert_eq!(crisis.row, Id::new("crisis.minor_illness"));
    let survival = crisis.survival.as_ref().expect("an illness is survivable");
    assert_eq!(survival.ease_factor, Some(3));
    assert_eq!(survival.ritual_level, 20);
    assert_eq!(
        survival.allowances,
        vec![CrisisAllowance::Attendant {
            ability: Id::new("ability.medicine"),
            characteristic: Characteristic::Int,
            ease_factor: 6,
            botch_penalty: -3,
        }],
        "the doctor of :16634 reaches the write-back too"
    );

    // The year records it, and the character is alive and holding exactly the
    // points the aging row awarded.
    let entry = &resolved.entity.aging_log[0];
    assert_eq!(entry.crisis_die, Some(10));
    assert_eq!(entry.crisis_total, Some(15));
    assert_eq!(entry.crisis_row, Some(Id::new("crisis.minor_illness")));
    assert_eq!(entry.crisis_severity, Some(CrisisSeverity::Minor));
    assert_eq!(
        resolved.entity.aging_points,
        BTreeMap::from([(Characteristic::Sta, 5)])
    );

    let reverted = arm_rules::revert_year(&resolved.entity, &rs, 40).expect("comes back off");
    assert_eq!(
        serde_json::to_string(&reverted).expect("a character serializes"),
        before
    );
}

/// Leprosy likewise states two mechanics at once:
///
/// > "A leper has a permanent -2 modifier to her Living Condition …, and whenever
/// > she undergoes an Aging Crisis (page 392) the leper sustains a Heavy Wound in
/// > addition to any other result." (ArMDE:6340)
///
/// The Heavy Wound is a *consequence*, not a number, so it ships as a marker with
/// amount 0 — the `crisis_heavy_wound` kind exists precisely so the shipped 0 is
/// not mistaken for an unfilled modifier.
///
/// Source: ArMDE:6340.
#[test]
fn leprosy_carries_its_crisis_wound_beside_its_living_conditions_penalty() {
    assert_eq!(
        aging_kinds("flaw.leprosy"),
        vec![
            (AgingEffect::LivingConditions, -2),
            (AgingEffect::CrisisHeavyWound, 0),
        ],
    );
}

/// > "Virtues that affect aging rolls do not affect crisis survival rolls."
/// > (ArMDE:16636)
///
/// This is that sentence's **converse**, which Mild Aging is the first shipped
/// item to make expressible: a modifier granted specifically to the crisis
/// survival roll is not an aging-roll modifier either, so nothing of the +3 may
/// reach the AGING TOTAL. Mild Aging moves the Living Conditions term by +1 and
/// nothing else — the trait modifier stays 0, and the total drops by exactly one,
/// because the total *subtracts* the Living Conditions Modifier (`ArMDE:16571`).
///
/// (Step 7 pins the other direction, that `aging_roll` modifiers stay out of the
/// survival roll.)
///
/// Source: ArMDE:4530, :16636.
#[test]
fn a_crisis_survival_modifier_never_reaches_the_aging_total() {
    let baseline = shipped_aging_total(&[]);
    let mild = shipped_aging_total(&["virtue.mild_aging"]);

    assert_eq!(
        mild.living_conditions.from_traits,
        baseline.living_conditions.from_traits + 1,
        "the +1 half is a Living Conditions term"
    );
    assert_eq!(
        mild.trait_modifier, 0,
        "the +3 is not an aging-roll modifier"
    );
    assert_eq!(mild.longevity_bonus, baseline.longevity_bonus);
    assert_eq!(mild.age_modifier, baseline.age_modifier);
    assert_eq!(
        mild.total,
        baseline.total - 1,
        "only the Living Conditions half moves the total"
    );

    // The +3 is nowhere in the total, but it is still surfaced for the player,
    // labelled by its own kind rather than folded into an aging-roll figure.
    let rs = load_full_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.mild_aging"))],
    );
    let surfaced: Vec<(String, i32)> = arm_rules::derived::surfaced_modifiers(&e, &rs)
        .into_iter()
        .filter(|m| m.family == arm_rules::derived::ModifierFamily::Aging)
        .map(|m| (m.detail, m.amount))
        .collect();
    assert!(
        surfaced.contains(&("crisis_survival".to_string(), 3)),
        "the crisis bonus stays visible: {surfaced:?}"
    );
}

/// `flaw.age_quickly` and `flaw.baneful_circumstances` both ship an `aging_roll`
/// modifier of **0**, and that 0 is deliberate — not an unfilled field waiting to
/// be "fixed" into a number.
///
/// Age Quickly doubles the *rate*: "your effective age … increases two years for
/// every year that passes, and you make two aging rolls each year" (ArMDE:5661).
/// Baneful Circumstances adds a *conditional extra roll*: "he must make an
/// additional Aging roll even if he is normally immune to aging" (ArMDE:5689).
/// Both are schedule rules — how many rolls, at what effective age — and neither
/// shifts the total of any one roll. The engine does not implement either
/// schedule yet, so each Flaw contributes nothing to the arithmetic while staying
/// visible in the surfaced-modifier read-out, where a player can act on it.
#[test]
fn age_quickly_contributes_nothing_to_the_total_and_stays_surfaced() {
    let items = ["flaw.age_quickly", "flaw.baneful_circumstances"];
    let both = shipped_aging_total(&items);
    assert_eq!(
        both.trait_modifier, 0,
        "neither Flaw shifts one roll's total"
    );
    assert_eq!(both.total, shipped_aging_total(&[]).total);

    // Still surfaced, so the player sees the mechanics the engine cannot apply.
    let rs = load_full_ruleset();
    let e = entity(
        "companion",
        items
            .iter()
            .map(|id| Selection::new(Id::new(*id)))
            .collect(),
    );
    let surfaced: Vec<(String, i32)> = arm_rules::derived::surfaced_modifiers(&e, &rs)
        .into_iter()
        .filter(|m| m.family == arm_rules::derived::ModifierFamily::Aging)
        .map(|m| (m.detail, m.amount))
        .collect();
    assert_eq!(
        surfaced,
        vec![("aging_roll".to_string(), 0), ("aging_roll".to_string(), 0)],
        "both Flaws stay listed for the player"
    );
}

/// The shipped table's own reading of `total`, for a companion who has already
/// accrued `accrued` Aging Points (parked in Str — Decrepitude counts the
/// character's whole bank, whichever Characteristics hold it, ArMDE:16617).
fn shipped_aging_outcome(total: i32, accrued: u8) -> AgingOutcome {
    let rs = load_full_ruleset();
    let mut e = entity("companion", vec![]);
    if accrued > 0 {
        e.aging_points.insert(Characteristic::Str, accrued);
    }
    arm_rules::aging::resolve_outcome(&e, &rs, total)
        .expect("the shipped ruleset carries aging rules")
}

/// The shipped table resolved row by row, against the shipped advancement curve:
/// the apparent-aging threshold of `ArMDE:16599-16600`, the "any Characteristic" band
/// of `ArMDE:16601`, the named rows of `ArMDE:16603-16610`, and both Decrepitude-and-Crisis
/// rows (`ArMDE:16602`, `ArMDE:16611`).
///
/// The unit fixture in `aging.rs` transcribes these rows by hand; only this test
/// witnesses the ones the app actually ships — and only here does the derived
/// Decrepitude count meet the real advancement curve.
///
/// Source: ArMDE:16599-16617.
#[test]
fn the_shipped_aging_table_resolves_each_row_of_16599_to_16611() {
    // "2 or less — No apparent aging" / "3 or more — Apparent age increases by
    // one year": one threshold asked of every total, not a pair of rows.
    let two = shipped_aging_outcome(2, 0);
    assert!(!two.apparent_age_increases);
    assert!(two.awards.is_empty());
    assert!(!two.crisis);
    let nine = shipped_aging_outcome(9, 0);
    assert!(nine.apparent_age_increases);
    assert!(
        nine.awards.is_empty(),
        "the appearance ages below the first row that costs anything"
    );

    // "10–12 — 1 Aging Point in any Characteristic" (ArMDE:16601), the player placing
    // it (ArMDE:16615).
    let eleven = shipped_aging_outcome(11, 0);
    assert_eq!(
        eleven.awards,
        vec![AgingPointAward {
            target: AgingPointTarget::PlayerChoice,
            points: Some(1),
        }]
    );
    assert!(eleven.apparent_age_increases);
    assert!(!eleven.crisis);

    // The named rows, including the one the book spells "Prs" (ArMDE:16606) and a
    // two-Characteristic row where EACH name takes a point of its own (ArMDE:16608).
    let named = |total: i32| -> Vec<AgingPointAward> { shipped_aging_outcome(total, 0).awards };
    let one_point = |characteristic| AgingPointAward {
        target: AgingPointTarget::Named(characteristic),
        points: Some(1),
    };
    assert_eq!(named(14), vec![one_point(Characteristic::Qik)]);
    assert_eq!(named(17), vec![one_point(Characteristic::Pre)]);
    assert_eq!(
        named(19),
        vec![
            one_point(Characteristic::Dex),
            one_point(Characteristic::Qik),
        ]
    );
    assert!(!shipped_aging_outcome(21, 0).crisis, "only 13 and 22+ do");

    // "Gain sufficient Aging Points … to reach the next level in Decrepitude, and
    // Crisis" (ArMDE:16602, :16611). The count comes off the shipped curve, so the
    // expectation is computed from it rather than written out.
    let rs = load_full_ruleset();
    let to_first_level = rs
        .advancement()
        .xp_for_score(1)
        .expect("the shipped curve prices Decrepitude 1");
    let accrued = 3;
    let owed = vec![AgingPointAward {
        target: AgingPointTarget::NextDecrepitudeLevel,
        points: Some(to_first_level - u32::from(accrued)),
    }];
    for total in [13, 22, 40] {
        let outcome = shipped_aging_outcome(total, accrued);
        assert_eq!(outcome.awards, owed, "total {total}");
        assert!(
            outcome.crisis,
            "total {total} sends him to the Crisis Table"
        );
        assert!(outcome.apparent_age_increases, "total {total}");
    }
}

/// Every Living Condition has a display name in both shipped languages, and no
/// name smuggles the table's cumulative-marker asterisk into the UI — the
/// `cumulative` flag carries that, and a raw `*` in a label would render as one.
#[test]
fn english_and_german_i18n_cover_all_living_conditions() {
    let rules = shipped_aging_rules();
    let en: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(SHIPPED_AGING_EN).expect("the English aging i18n is valid JSON");
    let de: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(SHIPPED_AGING_DE).expect("the German aging i18n is valid JSON");

    for row in &rules.living_conditions {
        for (lang, texts) in [("en", &en), ("de", &de)] {
            let name = texts
                .get(row.id.as_str())
                .and_then(|entry| entry.get("name"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("{lang} i18n missing living condition '{}'", row.id));
            assert!(!name.is_empty(), "{lang} name for '{}' is empty", row.id);
            assert!(
                !name.contains('*'),
                "{lang} name for '{}' carries the cumulative asterisk: {name}",
                row.id
            );
        }
    }
}

/// Every Crisis Table row has a display name in both shipped languages.
///
/// The German names are pinned as literals for the two rows that are a false
/// friend in the other direction: German *Schwere* is **Major** (ArMDE:16630) and
/// *Ernste* is **Serious** (ArMDE:16629), which is the opposite of what the English
/// cognate suggests. `alterung-twilight.md:82-92` agrees with the rulebook body.
#[test]
fn english_and_german_i18n_cover_all_crisis_rows() {
    let rules = shipped_aging_rules();
    let crisis = rules.crisis.clone().expect("the shipped crisis table");
    let en: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(SHIPPED_AGING_EN).expect("the English aging i18n is valid JSON");
    let de: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(SHIPPED_AGING_DE).expect("the German aging i18n is valid JSON");

    let name = |texts: &BTreeMap<String, serde_json::Value>, id: &str, lang: &str| -> String {
        texts
            .get(id)
            .and_then(|entry| entry.get("name"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{lang} i18n missing crisis row '{id}'"))
            .to_owned()
    };

    for row in &crisis.rows {
        for (lang, texts) in [("en", &en), ("de", &de)] {
            let text = name(texts, row.id.as_str(), lang);
            assert!(!text.is_empty(), "{lang} name for '{}' is empty", row.id);
        }
    }

    assert_eq!(
        name(&de, "crisis.serious_illness", "de"),
        "Ernste Erkrankung"
    );
    assert_eq!(
        name(&de, "crisis.major_illness", "de"),
        "Schwere Erkrankung"
    );
}

/// `crisis.rows` ships in BAND order in `rules/core/aging.json` — a deliberate
/// exception to the project's canonical (id-sorted) serialization rule, recorded
/// in RULES.md ("Three things a later sweep must not undo") and guarded by
/// `shipped_crisis_table_carries_the_16626_to_16632_rows` above. Until this test
/// was added, both i18n counterparts instead listed the same seven ids
/// alphabetically, so core and i18n silently disagreed on the table's order
/// (full-audit finding V43). This asserts the two i18n files mirror core's band
/// order rather than keeping their own alphabetical one, using a line-based key
/// reader (not `serde_json::Value`, which is backed by a `BTreeMap` here and
/// would always report keys pre-sorted regardless of the file's real order).
#[test]
fn crisis_row_i18n_order_matches_core_band_order() {
    let rules = shipped_aging_rules();
    let crisis = rules.crisis.clone().expect("the shipped crisis table");
    let core_order: Vec<&str> = crisis.rows.iter().map(|row| row.id.as_str()).collect();

    for (lang, file) in [("en", SHIPPED_AGING_EN), ("de", SHIPPED_AGING_DE)] {
        let i18n_order: Vec<&str> = top_level_keys_in_file_order(file)
            .into_iter()
            .filter(|key| key.starts_with("crisis."))
            .collect();
        assert_eq!(
            i18n_order, core_order,
            "i18n/{lang}/aging.json must list crisis rows in the same band order as core"
        );
    }
}

/// The three shipped items that suspend some part of aging tag the **two
/// independent facts** separately, because the sources state them separately:
///
/// - Unaging — "your aging points do not decrease your Characteristics, only
///   building up to give you Decrepitude points … You may choose your apparent
///   age freely" (ArMDE:5189): both facts.
/// - Bound to (Role) — "This Flaw also includes the effects of the Unaging
///   Virtue, **but** the character's apparent age advances in line with their
///   physical age" (ArMDE:5743): the Characteristic immunity only. That *but* is
///   what proves the two are separable at all.
/// - Bee King — "Bee Kings do not appear to age after reaching maturity"
///   (ArMDE:3488): the appearance only, and nothing about Characteristics.
///
/// All three shipped `no_aging` alone before the tags came apart, which made the
/// Bee King's entry simply wrong. This test is the outside witness that keeps the
/// retag from silently regressing to one tag again.
#[test]
fn the_three_aging_immunities_ship_their_two_facts_separately() {
    let rs = load_ruleset();
    let tagged = |id: &str| -> Vec<AgingEffect> {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("the shipped catalogue carries '{id}'"));
        let mut kinds: Vec<AgingEffect> = item
            .effects
            .iter()
            .filter_map(|effect| match effect {
                Effect::AgingMod { kind, .. } => Some(kind),
                _ => None,
            })
            .copied()
            .collect();
        kinds.sort_unstable();
        kinds
    };

    assert_eq!(
        tagged("virtue.unaging"),
        vec![AgingEffect::NoAging, AgingEffect::NoApparentAging],
        "Unaging states both facts (ArMDE:5189)"
    );
    assert_eq!(
        tagged("flaw.bound_to_role_role"),
        vec![AgingEffect::NoAging],
        "Bound to (Role) keeps ageing in appearance (ArMDE:5743)"
    );
    assert_eq!(
        tagged("virtue.bee_king"),
        vec![AgingEffect::NoApparentAging],
        "a Bee King only stops looking older (ArMDE:3488)"
    );
}

/// Issue F (selection-level): a magus selecting BOTH magnitude variants of the
/// same Virtue — here the prefix pair Major / Minor Magical Focus — must raise
/// the incompatibility issue. Behavioral assertion (no catalogue counts).
/// Source: ArMDE:4405.
#[test]
fn both_magical_focus_variants_are_incompatible() {
    let rs = load_ruleset();
    let focus = |name: &str| {
        Selection::with_params(
            Id::new(name),
            BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
        )
    };
    let e = entity(
        "magus",
        vec![
            focus("virtue.major_magical_focus"),
            focus("virtue.minor_magical_focus"),
        ],
    );
    let result = validate(&e, &rs);
    let flagged = result.issues.iter().any(|i| {
        i.code == arm_rules::validation::ValidationIssue::CODE_INCOMPATIBLE
            && [i.args.get("item"), i.args.get("other")]
                .iter()
                .filter_map(|a| a.map(String::as_str))
                .any(|a| a == "virtue.major_magical_focus")
    });
    assert!(
        flagged,
        "selecting both Magical Focus variants must be flagged incompatible, got: {:?}",
        result.issues
    );
}

// --- Slice 7 (#5): parameters the rules restrict to a Form ------------------

/// The single parameter definition of a shipped V/F, by id.
fn only_parameter(rs: &Ruleset, id: &str) -> ParameterDef {
    let item = rs
        .item(&Id::new(id))
        .unwrap_or_else(|| panic!("{id} present in the shipped catalogue"));
    assert_eq!(item.parameters.len(), 1, "{id} declares one parameter");
    item.parameters[0].clone()
}

/// Deft (Form) — the reported case of #4. Its data was already right; the bug was
/// entirely in the picker, so this pins the data so a "fix" cannot move it.
///
/// Source: ArMDE:3645-3648.
#[test]
fn virtue_deft_form_declares_the_form_domain() {
    let rs = load_ruleset();
    let def = only_parameter(&rs, "virtue.deft_form");
    assert_eq!(def.key, "form");
    assert_eq!(def.domain, ParameterDomain::Form);
}

/// The two **Form-scoped Magic Resistance Flaws**. Each weakens resistance
/// against one named Form, and each says in its own descriptor that it repeats
/// across different Forms:
///
/// > "Your Parma Magica is defective and provides only half the normal Magic
/// > Resistance **against a certain Form**. You may purchase this Flaw more than
/// > once for different Forms." (`ArMDE:6144`)
///
/// > "You gain no bonus from **one of** your Form scores to Magic Resistance …
/// > You may take this Flaw multiple times, for multiple Forms." (`ArMDE:6348`)
///
/// Both sentences are one data shape: a `form` parameter puts each copy's Form
/// into the `(item_ref, params)` duplicate key, after which the **default**
/// `max_per_target` of 1 says exactly "once per Form", and an absent `max_total`
/// says "any number of different Forms". Neither entry may carry the
/// `max_per_target: 255` it shipped with before the Form existed — that number
/// permitted a second, identical copy naming the same Form, which no line of
/// either descriptor allows.
#[test]
fn the_form_scoped_magic_resistance_flaws_name_their_form() {
    let rs = load_ruleset();

    for (id, line) in [
        ("flaw.flawed_parma_magica", 6144),
        ("flaw.limited_magic_resistance", 6348),
    ] {
        let def = only_parameter(&rs, id);
        assert_eq!(def.key, "form", "{id} names its Form under `form`");
        assert_eq!(
            def.domain,
            ParameterDomain::Form,
            "{id}'s target is a Form, not either Art class (ArMDE:{line})"
        );

        let item = rs.item(&Id::new(id)).expect("present");
        assert_eq!(
            item.max_per_target, 1,
            "{id} is taken once for EACH Form (ArMDE:{line}), so two copies naming \
             the same Form must collide"
        );
        assert_eq!(
            item.max_total,
            u8::MAX,
            "{id} states no ceiling on how many DIFFERENT Forms it may name \
             (ArMDE:{line})"
        );
        assert_eq!(
            def.max_per_value, 1,
            "{id} needs no EXPLICIT per-value cap: with one parameter, \
             `max_per_target`'s own key IS the Form, so an explicit cap here \
             would be a second spelling of the same ceiling. D10's default of \
             1 already agrees with `max_per_target: 1` and adds nothing"
        );
    }
}

/// The repeat rule both descriptors state, exercised rather than asserted about:
/// two copies naming two different Forms are legal, a second copy naming the
/// same Form is not. Source: ArMDE:6144, :6348.
#[test]
fn a_form_scoped_mr_flaw_repeats_across_forms_but_never_within_one() {
    let rs = load_ruleset_with_spells();

    for id in ["flaw.flawed_parma_magica", "flaw.limited_magic_resistance"] {
        let pick = |form: &str| {
            Selection::with_params(
                Id::new(id),
                BTreeMap::from([("form".to_string(), Id::new(form))]),
            )
        };

        let different = issue_codes(
            &entity("magus", vec![pick("art.ignem"), pick("art.corpus")]),
            &rs,
        );
        assert!(
            !different.contains(&"duplicate_selection".to_string()),
            "{id} twice for two different Forms is what the descriptor permits: \
             {different:?}"
        );

        let same = issue_codes(
            &entity("magus", vec![pick("art.ignem"), pick("art.ignem")]),
            &rs,
        );
        assert!(
            same.contains(&"duplicate_selection".to_string()),
            "{id} twice for the SAME Form is a repeat the descriptor does not \
             grant: {same:?}"
        );
    }
}

/// A save written before the Form parameter existed holds the selection with no
/// `form` key at all, and the engine **reports** that rather than migrating it:
/// there is no correct Form to invent, and saves store choices, not resolved
/// values. The finding is `missing_param`, one pick clears it permanently, and
/// it is listed in `docs/open-todos.md` → "What an older save still reports on
/// open". Source: ArMDE:6144, :6348.
#[test]
fn an_mr_flaw_selection_written_before_the_form_parameter_reports_missing_param() {
    let rs = load_ruleset_with_spells();

    for id in ["flaw.flawed_parma_magica", "flaw.limited_magic_resistance"] {
        let codes = issue_codes(&entity("magus", vec![Selection::new(Id::new(id))]), &rs);
        assert!(
            codes.contains(&"missing_param".to_string()),
            "{id} with no Form stored must ask for one, not guess: {codes:?}"
        );
    }
}

/// Deficient Form / Deficient Technique were already correct too — one per Art
/// class, and each is the reason the two narrow domains exist at all.
#[test]
fn the_deficient_art_flaws_declare_their_own_art_class() {
    let rs = load_ruleset();
    let form = only_parameter(&rs, "flaw.deficient_form");
    assert_eq!(
        (form.key.as_str(), form.domain),
        ("form", ParameterDomain::Form)
    );
    let technique = only_parameter(&rs, "flaw.deficient_technique");
    assert_eq!(
        (technique.key.as_str(), technique.domain),
        ("technique", ParameterDomain::Technique)
    );
}

/// #5: five V/F whose source restricts the parameter to a **Form** declared the
/// wider `art` domain, which the engine cannot catch because `art` accepts either
/// Art class. Verified against each item's own cited range:
///
/// - `flaw.form_monstrosity` — "a monstrous feature, or mutation, which corresponds
///   to a magical Form", with an examples table headed `Form` listing only Forms
///   (ArMDE:6162-6185).
/// - `flaw.hunger_for_form_magic` — "1 pawn of vis each season, corresponding to the
///   Form that it has been mostly exposed to" (`ArMDE:6276-6279`).
/// - `virtue.extractor_of_form_vis` — "only if the features of the aura exemplify the
///   Form … This Virtue may be taken multiple times (once for each Form)"
///   (`ArMDE:3779-3782`).
/// - `virtue.imbued_with_the_spirit_of_form` — "any being with a Magic Might
///   associated with the Form of this Virtue" (`ArMDE:4085-4094`).
/// - `virtue.master_of_form_creatures` — "beings whose Magic Might is aligned with a
///   particular Form … once for each Form" (`ArMDE:4463-4466`).
#[test]
fn form_restricted_virtues_flaws_declare_the_form_domain() {
    let rs = load_ruleset();
    for id in [
        "flaw.form_monstrosity",
        "flaw.hunger_for_form_magic",
        "virtue.extractor_of_form_vis",
        "virtue.imbued_with_the_spirit_of_form",
        "virtue.master_of_form_creatures",
    ] {
        let def = only_parameter(&rs, id);
        assert_eq!(def.key, "form", "{id} parameter key is 'form'");
        assert_eq!(
            def.domain,
            ParameterDomain::Form,
            "{id} restricts its parameter to a Form"
        );
    }
}

/// The counter-case, so #5 is not over-applied: Affinity with (Art) and Puissant
/// (Art) name **either** Art class, so their `art` domain is correct by design.
#[test]
fn items_legal_for_either_art_class_keep_the_art_domain() {
    let rs = load_ruleset();
    for id in ["virtue.affinity_art", "virtue.puissant_art"] {
        let def = only_parameter(&rs, id);
        assert_eq!(def.key, "art", "{id} parameter key is 'art'");
        assert_eq!(
            def.domain,
            ParameterDomain::Art,
            "{id} accepts either Art class"
        );
    }
}

// --- Slice 7 (#32): the rules' exemplar for a widened requirement -----------

/// The i18n label key a requirement's `exemplar` slug resolves through.
fn exemplar_id(slug: &str) -> Id {
    Id::new(format!("exemplar.{slug}"))
}

/// #32: `ArMDE:2437` names **Latin** three times, but `ability.dead_language` takes a
/// free-text instance, so the engine can only enforce "any Dead Language ≥ N". The
/// widening is permanent (see RULES.md); the honesty fix is to carry the rules' own
/// exemplar as a language-neutral slug and label it per locale.
///
/// Source: ArMDE:2437 (Latin 1), `ArMDE:2455`
/// (the recommended Latin 4), `ArMDE:7151` ("For most characters, Latin 3 is required").
#[test]
fn the_magus_minimum_dead_language_requirement_names_its_exemplar() {
    let rs = load_full_ruleset();
    let apprenticeship = rs
        .life_stages()
        .and_then(|rules| rules.apprenticeship.as_ref())
        .expect("the shipped life stages declare an apprenticeship");

    let minimum = apprenticeship
        .minimum_abilities
        .iter()
        .find(|r| r.ability == Id::new("ability.dead_language"))
        .expect("the minimums demand a dead language");
    assert_eq!(minimum.exemplar.as_deref(), Some("latin"), ":2437 Latin 1");

    let recommended = apprenticeship
        .recommended_abilities
        .iter()
        .find(|r| r.ability == Id::new("ability.dead_language"))
        .expect("the recommendations demand a dead language");
    assert_eq!(
        recommended.exemplar.as_deref(),
        Some("latin"),
        ":2455 Latin 4"
    );

    let scholarly = rs
        .scholarly_language_requirement()
        .expect("the shipped abilities declare a scholarly language");
    assert_eq!(
        scholarly.exemplar.as_deref(),
        Some("latin"),
        ":7151 Latin 3 for most characters"
    );
}

/// The exemplar slug must resolve to translatable text in **both** shipped locales,
/// because `rules/core/` may carry no translatable string.
#[test]
fn the_exemplar_slug_resolves_in_both_locales() {
    let rs = load_full_ruleset();
    let en = LocalizedRuleset::new(
        rs.clone(),
        include_str!("../../../rules/i18n/en/abilities.json"),
    )
    .unwrap();
    let de = LocalizedRuleset::new(
        rs.clone(),
        include_str!("../../../rules/i18n/de/abilities.json"),
    )
    .unwrap();
    assert_eq!(en.display_name(&exemplar_id("latin")), Some("Latin"));
    assert_eq!(de.display_name(&exemplar_id("latin")), Some("Latein"));
}

/// The exemplar is a **label key, not a `ref`**: it names one example the rules
/// themselves name, not an entry in any catalogue (there is no language catalogue and
/// there never will be — see RULES.md). So the loader must not try to resolve it, and
/// a ruleset whose exemplar matches no id at all still loads.
#[test]
fn an_exemplar_slug_is_not_treated_as_a_referential_integrity_ref() {
    let life_stages = r#"{
      "apprenticeship": {
        "minimum_abilities": [
          { "ability": "ability.dead_language", "exemplar": "no_such_catalogue_entry",
            "min_score": 1 }
        ],
        "recommended_abilities": [],
        "recommended_xp": 0,
        "xp": 240,
        "years": 15
      },
      "childhood": {
        "years": 5,
        "native_language_ability": "ability.living_language",
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": ["ability.athletics"]
      },
      "later_life": { "xp_per_year": 15 },
      "post_apprenticeship": {
        "lab_season_cost": 10,
        "max_charged_lab_seasons_per_year": 3,
        "points_per_year": 30
      }
    }"#;
    let load = |life_stages: &str| {
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: include_str!("../../../rules/core/virtues_flaws.json"),
            type_profiles: include_str!("../../../rules/core/character_types.json"),
            abilities: Some(include_str!("../../../rules/core/abilities.json")),
            arts: None,
            houses: Some(SHIPPED_HOUSES),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
            life_stages: Some(life_stages),
            childhoods: None,
            aging: None,
        })
    };
    if let Err(err) = load(life_stages) {
        panic!("an exemplar slug naming no catalogue id must still load, got: {err}");
    }
    // Not vacuous: the requirement's `ability` IS a ref, and a bogus one still fails.
    let bogus_ability = life_stages.replace("ability.dead_language", "ability.nonesuch");
    let err = load(&bogus_ability).unwrap_err();
    assert!(
        err.to_string().contains("unknown ability"),
        "the ability ref must still be resolved, got: {err}"
    );
}

// --- Multi-category Virtues and Flaws ---------------------------------------
//
// A rulebook descriptor may name two categories, and `PointItem.categories` now
// keeps both (primary first) instead of dropping all but the earliest-listed.
// These tests assert the consequences on the SHIPPED catalogue, structurally:
// none of them names a catalogue total, and the cap's size is read out of the
// profile data rather than assumed.

/// The shipped catalogue carries each two-category descriptor's categories in
/// the source's order, primary first.
///
/// Source: ArMDE:5077-5078
/// (*Minor, Social Status, Supernatural*), :6646-6647 (*Major, Story,
/// Supernatural*), :6803-6804 (*Major, Hermetic, Story*), :6985-6986
/// (*Minor, Story, Supernatural*).
#[test]
fn shipped_two_category_items_keep_the_descriptor_order() {
    let rs = load_ruleset();
    let expected: &[(&str, &[&str])] = &[
        ("virtue.sufi", &["social_status", "supernatural"]),
        ("flaw.raised_from_the_dead", &["story", "supernatural"]),
        ("flaw.suppressed_gift", &["hermetic", "story"]),
        ("flaw.visions", &["story", "supernatural"]),
    ];
    for (id, categories) in expected {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship in the catalogue"));
        assert_eq!(
            item.categories, *categories,
            "{id} must keep its descriptor's categories in order"
        );
        assert_eq!(
            item.first_listed_category(),
            categories[0],
            "{id}'s single-bucket tie-break is the descriptor's first-listed category"
        );
    }
}

/// A Flaw whose descriptor names two categories counts against BOTH of their
/// caps. Suppressed Gift is "*Major, Hermetic, Story*", so it is a Story Flaw
/// for the Story cap — which it was invisible to while only the earliest-listed
/// category was stored.
#[test]
fn a_two_category_flaw_counts_against_its_secondary_category_cap() {
    let rs = load_ruleset();
    let suppressed = Id::new("flaw.suppressed_gift");
    let item = rs
        .item(&suppressed)
        .expect("flaw.suppressed_gift must ship in the catalogue");
    assert!(item.has_category("hermetic"), "its primary category");
    assert!(item.has_category("story"), "its secondary category");

    // The cap's size is data. Read it, then fill it exactly with Flaws that
    // carry Story as their ONLY category, so the cap is untripped until the
    // two-category Flaw is added.
    let magus = rs
        .profile(&Id::new("magus"))
        .expect("the magus profile must ship");
    let cap = magus
        .budget
        .flaw_category_caps
        .iter()
        .find(|c| c.category == "story" && !c.major_only)
        .expect("the magus profile must cap Story Flaws");

    let filler: Vec<Selection> = rs
        .items_by_category("story")
        .filter(|i| i.kind == ItemKind::Flaw && i.categories.len() == 1 && i.parameters.is_empty())
        .take(cap.max as usize)
        .map(|i| Selection::new(i.id.clone()))
        .collect();
    assert_eq!(
        filler.len(),
        cap.max as usize,
        "the catalogue must ship enough single-category Story Flaws to fill the cap"
    );

    let at_cap = validate(&entity("magus", filler.clone()), &rs);
    assert!(
        !at_cap
            .issues
            .iter()
            .any(|i| i.code == "too_many_story_flaws"),
        "filling the Story cap exactly must not trip it"
    );

    let mut over_cap = filler;
    over_cap.push(Selection::new(suppressed));
    let over_cap = validate(&entity("magus", over_cap), &rs);
    assert!(
        over_cap
            .issues
            .iter()
            .any(|i| i.code == "too_many_story_flaws"),
        "Suppressed Gift's secondary Story category must count against the Story cap"
    );
}

/// Permitting is an ANY test over the item's categories and forbidding is the
/// mirror of it — an EVERY test — so the two agree instead of contradicting each
/// other. A companion may take Story Flaws, and Suppressed Gift ("*Major,
/// Hermetic, Story*") reaches them through its secondary Story category: neither
/// the permitted nor the forbidden check rules it out.
///
/// The book backs the outcome. `ArMDE:2840` bars a companion from Hermetic Virtues and
/// Flaws "unless you have The Gift" — and a Suppressed-Gift character *does* have
/// The Gift (`ArMDE:6805`: it "does not function", but the social penalties remain),
/// which is why `has_the_gift` flags them through the same `hermetic` category.
/// `ArMDE:6809` then describes the Flaw as a companion's: "If he replaces a companion,
/// he will become much more powerful when the Story Flaw is resolved."
///
/// The fixture deliberately stays **unGifted**, because that is the only state
/// in which this still tests the conjunction: since B5 the companion's
/// `hermetic` rules are conditional on `Has(virtue.the_gift)`, so a *Gifted*
/// companion clears both checks through `hermetic` itself and `story` would be
/// carrying nothing. The price is that the entity is incomplete for a different,
/// correct reason — `ArMDE:6805` gives the Flaw The Gift, so it now carries
/// `prerequisites: Has(virtue.the_gift)` — which the last assertion pins rather
/// than leaves as a surprise.
#[test]
fn a_secondary_category_clears_both_the_permitted_and_the_forbidden_check() {
    let rs = load_ruleset();
    let companion = rs
        .profile(&Id::new("companion"))
        .expect("the companion profile must ship");
    assert!(
        companion.names_permitted_category("story"),
        "a companion may take Story Flaws"
    );
    assert!(
        companion.names_forbidden_category("hermetic"),
        "and the companion profile forbids the Hermetic category"
    );

    let suppressed = Id::new("flaw.suppressed_gift");
    let item = rs
        .item(&suppressed)
        .expect("flaw.suppressed_gift must ship in the catalogue");
    assert!(item.has_category("hermetic") && item.has_category("story"));

    let result = validate(
        &entity("companion", vec![Selection::new(suppressed.clone())]),
        &rs,
    );
    assert!(
        !result
            .issues
            .iter()
            .any(|i| i.code == "category_not_permitted" && i.context.as_ref() == Some(&suppressed)),
        "a permitted secondary category must clear the permitted-categories check"
    );
    assert!(
        !result
            .issues
            .iter()
            .any(|i| i.code == "forbidden_category" && i.context.as_ref() == Some(&suppressed)),
        "and a NON-forbidden secondary category must clear the forbidden one: \
         forbidding fires only when EVERY category is forbidden: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&suppressed)),
        "the category gates are clear; what stops this unGifted companion is \
         `ArMDE:6805`'s own requirement, which is the honest reason"
    );
}

/// Under the conjunction there is no distinguished offender, so the
/// `forbidden_category` issue joins `category_not_permitted` in naming the
/// descriptor's first-listed category as a deterministic tie-break.
///
/// **And no shipped item can produce the old "secondary-position forbidden hit"
/// case at all** — the second half of this test proves it structurally rather
/// than leaving a fixture that silently proves nothing: for a multi-category item
/// to raise the issue, *every* one of its categories must be forbidden, and then
/// the first-listed is forbidden too. The sweep asserts no shipped profile
/// forbids every category of any shipped multi-category item, which is why the
/// grog/Suppressed-Gift fixture below now raises only the permitted-side issue.
///
/// The fixture used to be grog/Visions ("*Minor, Story, Supernatural*"), which
/// stopped demonstrating anything once the grog profile's unsourced
/// `supernatural` restriction was removed (open-to-dos row 20): `supernatural`
/// is on a grog's permitted list now, so Visions clears both category checks and
/// is refused by the sourced Story-Flaw cap instead (`ArMDE:2826`). Suppressed Gift
/// replaces it as the one shipped pairing that still exercises the conjunction
/// for a grog.
///
/// **B5 qualified that claim rather than ending it, and the qualification is
/// asserted below so it cannot lapse silently.** `flaw.suppressed_gift` now
/// carries `prerequisites: Has(virtue.the_gift)` (`ArMDE:6805`, "The character has
/// The Gift but cannot access its power"), and a grog may never hold The Gift
/// (`ArMDE:2830`), so the pairing gained a *fourth* refusal on top of
/// `category_not_permitted`, `gift_forbidden` and the Major cap. That does not
/// weaken what this test demonstrates — the two category validators are
/// independent of prerequisite evaluation and run regardless — but the pairing
/// is emphatically not a build a player could ever complete, and the test does
/// not pretend otherwise: it now asserts the `prereq_not_met` too.
///
/// **There is no replacement, and the sweep at the end proves why**: the
/// conjunction needs a multi-category item at least one of whose categories a
/// grog forbids, a grog forbids only `hermetic`, and `flaw.suppressed_gift` is
/// the only multi-category item in the shipped catalogue carrying it. So this
/// is the fixture or there is none.
#[test]
fn forbidding_fires_only_when_every_category_is_forbidden() {
    let rs = load_ruleset();
    let grog = rs
        .profile(&Id::new("grog"))
        .expect("the grog profile must ship");
    assert!(grog.names_forbidden_category("hermetic"));
    assert!(!grog.names_forbidden_category("story"));
    assert!(!grog.names_permitted_category("story"));

    // Suppressed Gift is "*Major, Hermetic, Story*"
    // (ArMDE:6804, entry :6803-6810). A grog forbids
    // only the first of those, so the item survives the forbidden check — and is
    // still blocked, by the honest reason: neither category is on the grog's
    // permitted list.
    let suppressed = Id::new("flaw.suppressed_gift");
    let item = rs
        .item(&suppressed)
        .expect("flaw.suppressed_gift must ship");
    assert_eq!(item.first_listed_category(), "hermetic");

    let result = validate(
        &entity("grog", vec![Selection::new(suppressed.clone())]),
        &rs,
    );
    assert!(
        !result
            .issues
            .iter()
            .any(|i| i.code == "forbidden_category" && i.context.as_ref() == Some(&suppressed)),
        "one forbidden category out of two must no longer rule the item out: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
    let not_permitted = result
        .issues
        .iter()
        .find(|i| i.code == "category_not_permitted" && i.context.as_ref() == Some(&suppressed))
        .expect("neither of Suppressed Gift's categories is on the grog's permitted list");
    assert_eq!(
        not_permitted.args.get("category").map(String::as_str),
        Some("hermetic"),
        "when every category failed, the issue names the first-listed"
    );
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&suppressed)),
        "and since B5 the pairing is prereq-illegal as well: `ArMDE:6805` gives the \
         Flaw The Gift, `ArMDE:2830` denies a grog one"
    );

    // No replacement fixture exists, so pin the fact rather than discovering it
    // the next time this one is questioned: a grog forbids only `hermetic`, and
    // this is the sole multi-category item in the catalogue that carries it.
    let multi_category_hermetic: Vec<&Id> = rs
        .items()
        .filter(|item| item.categories.len() > 1 && item.has_category("hermetic"))
        .map(|item| &item.id)
        .collect();
    assert_eq!(
        multi_category_hermetic,
        vec![&suppressed],
        "the grog conjunction has exactly one possible fixture"
    );

    // The structural half: the secondary-position forbidden hit is unreachable
    // for the shipped catalogue. Catalogue size stays data — this counts nothing
    // and asserts a property of every item it finds.
    let mut multi_category_items = 0;
    for item in rs.items() {
        if item.categories.len() < 2 {
            continue;
        }
        multi_category_items += 1;
        for profile in rs.profiles() {
            assert!(
                !item
                    .categories
                    .iter()
                    .all(|c| profile.names_forbidden_category(c)),
                "no shipped profile forbids every category of a multi-category \
                 item, so the `forbidden_category` issue can never name anything \
                 but the first-listed: {} vs profile {}",
                item.id,
                profile.id
            );
        }
    }
    assert!(
        multi_category_items > 0,
        "the shipped catalogue must contain multi-category descriptors"
    );
}

/// The cell the book states outright. Sufi is "*Minor, Social Status,
/// Supernatural*", and either category now opens it to a grog:
///
/// > "It is also possible to be an entirely mundane Sufi, in which case you
/// > should take this Virtue as a Social Status Virtue" — `ArMDE:5079`
///
/// > "either as a Minor Social Status Virtue **or** a Minor Supernatural Virtue"
/// > — `ArMDE:5083`
///
/// (ArMDE:5079, :5083.)
///
/// This test no longer exercises the ANY/EVERY conjunction, and says so rather
/// than pretending to: it did when a grog forbade `supernatural` and only
/// `social_status` could rescue the Virtue, but that restriction had no source
/// and is gone (open-to-dos row 20), so `social_status` is not carrying the item
/// alone any more. The conjunction's grog case now lives in
/// `forbidding_fires_only_when_every_category_is_forbidden` (Suppressed Gift)
/// and its companion case in
/// `a_secondary_category_clears_both_the_permitted_and_the_forbidden_check`.
/// What remains here is still worth pinning: `ArMDE:5079` names a mundane Sufi
/// explicitly, and a grog may be one.
#[test]
fn a_grog_may_take_sufi_through_its_social_status_category() {
    let rs = load_ruleset();
    let sufi = Id::new("virtue.sufi");
    let item = rs.item(&sufi).expect("virtue.sufi must ship");
    assert!(item.has_category("social_status") && item.has_category("supernatural"));

    let result = validate(&entity("grog", vec![Selection::new(sufi.clone())]), &rs);
    let category_issues: Vec<&String> = result
        .issues
        .iter()
        .filter(|i| {
            i.context.as_ref() == Some(&sufi)
                && (i.code == "forbidden_category" || i.code == "category_not_permitted")
        })
        .map(|i| &i.code)
        .collect();
    assert!(
        category_issues.is_empty(),
        "a mundane Sufi is a Social Status Virtue a grog may take: {category_issues:?}"
    );
}

// --- Row 20: conditional category rules (B5) ---------------------------------
//
// ":2840" — "You may not take Hermetic Virtues and Flaws, unless you have The
// Gift (this would be highly unusual)" — is a CONDITIONAL category rule, and
// the companion profile encoded only its unconditional half. Both halves of
// the profile now carry a `when`: `hermetic` is permitted while
// `Has(virtue.the_gift)` holds and forbidden while it does not. Permitting is
// ANY and forbidding is EVERY, so relaxing only the forbid would have left
// every single-category Hermetic item refused with `category_not_permitted` —
// the same trap the grog `supernatural` removal recorded (RULES.md, the grog
// profile rows).

/// The cell ":2840" grants and the profile refused: a Gifted companion may take
/// a Hermetic Flaw. `flaw.blatant_gift` is "*Major, Hermetic*" (`ArMDE:5711-5712`)
/// and already carries `prerequisites: Has(virtue.the_gift)`, so it is the
/// shipped consumer of the conditional rule.
#[test]
fn a_gifted_companion_may_take_blatant_gift() {
    let rs = load_ruleset();
    let blatant = Id::new("flaw.blatant_gift");
    let item = rs.item(&blatant).expect("flaw.blatant_gift must ship");
    assert_eq!(
        item.categories,
        vec!["hermetic".to_string()],
        "the fixture is only meaningful while Blatant Gift is single-category \
         Hermetic: a secondary category would clear the permitted check on its own"
    );

    let result = validate(
        &entity(
            "companion",
            vec![
                Selection::new(Id::new("virtue.the_gift")),
                Selection::new(blatant.clone()),
            ],
        ),
        &rs,
    );
    let category_issues: Vec<&String> = result
        .issues
        .iter()
        .filter(|i| {
            i.context.as_ref() == Some(&blatant)
                && (i.code == "forbidden_category" || i.code == "category_not_permitted")
        })
        .map(|i| &i.code)
        .collect();
    assert!(
        category_issues.is_empty(),
        "':2840' permits a Gifted companion the Hermetic category: {category_issues:?}"
    );
}

/// The other side of the same conditional, which must not be lost while
/// relaxing it: an *unGifted* companion is still refused. ":2840" grants the
/// exception only to a Gifted character, and both halves of the profile are
/// keyed on the same condition, so both issues still fire.
#[test]
fn an_ungifted_companion_still_may_not() {
    let rs = load_ruleset();
    let blatant = Id::new("flaw.blatant_gift");

    let result = validate(
        &entity("companion", vec![Selection::new(blatant.clone())]),
        &rs,
    );
    for code in ["category_not_permitted", "forbidden_category"] {
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.code == code && i.context.as_ref() == Some(&blatant)),
            "an unGifted companion must still be refused the Hermetic category \
             ({code} missing): {:?}",
            result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
        );
    }
}

/// Anti-circularity. The condition's leaf is the **id** `Has(virtue.the_gift)`,
/// never a category test, and this pins why that matters.
///
/// `effective::has_the_gift` is category-based: it reads any selection carrying
/// a `gift_categories` category (`["hermetic"]` on every shipped profile) as
/// Gift-bearing. Had the profile's condition been "is this character Gifted?" in
/// that sense, the rule would license itself — you may take a Hermetic item
/// because you are Gifted, and you are Gifted because you hold a Hermetic item.
///
/// Two independent guards, so neither can lapse silently:
/// (a) structural — `virtue.the_gift` is `special` and Free, carries none of the
///     companion's `gift_categories`, and is itself always permitted, so
///     satisfying the condition can never require the category it licenses;
/// (b) behavioural — a companion holding *only* `flaw.blatant_gift` is exactly
///     the character `has_the_gift` would call Gifted, and is still refused.
#[test]
fn a_companion_does_not_gift_himself_with_a_hermetic_virtue() {
    let rs = load_ruleset();
    let companion = rs
        .profile(&Id::new("companion"))
        .expect("the companion profile must ship");

    let gift = rs
        .item(&Id::new("virtue.the_gift"))
        .expect("virtue.the_gift must ship");
    assert_eq!(gift.magnitude, Magnitude::Free);
    assert!(
        !gift
            .categories
            .iter()
            .any(|c| companion.gift_categories.contains(c)),
        "the condition's leaf must not itself be a member of the category it \
         licenses, or permission would be self-granting: {:?} vs {:?}",
        gift.categories,
        companion.gift_categories
    );

    let blatant = Id::new("flaw.blatant_gift");
    assert!(
        rs.item(&blatant)
            .expect("flaw.blatant_gift must ship")
            .categories
            .iter()
            .any(|c| companion.gift_categories.contains(c)),
        "the fixture only proves anything while Blatant Gift is what the \
         category-based Gift test would call Gift-bearing"
    );

    let result = validate(
        &entity("companion", vec![Selection::new(blatant.clone())]),
        &rs,
    );
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "category_not_permitted" && i.context.as_ref() == Some(&blatant)),
        "holding a Hermetic item must not satisfy the condition that permits \
         the Hermetic category: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

/// Gentle Gift's prerequisite was `Has(virtue.hermetic_magus)`, which no line
/// supports, and which left B5 half-done: a Gifted companion could take Blatant
/// Gift but not its twin.
///
/// The two are a matched pair — mutually `incompatible_with`, both descriptors
/// reading "*Major, Hermetic*" (`ArMDE:3956` and `ArMDE:5712`) with **no** prerequisite
/// line in either, and both mentioning magi only in passing (Blatant Gift's own
/// text says "even if they do not know you are a **magus**" and nonetheless
/// ships `Has(virtue.the_gift)`). The magus requirement was inferred from the
/// comparative at `ArMDE:3957`, "Unlike other magi, whose Magical nature disturbs
/// normal people and animals" — which compares, it does not restrict. And the
/// penalty Gentle Gift cancels attaches to The Gift, not to Order membership:
/// `ArMDE:6805` describes a character who "continues to suffer the negative social
/// penalties of The Gift".
#[test]
fn gentle_gift_requires_the_gift_and_not_the_order() {
    let rs = load_ruleset();
    let gentle = Id::new("virtue.gentle_gift");
    assert_eq!(
        rs.item(&gentle)
            .expect("virtue.gentle_gift must ship")
            .prerequisites,
        Some(Prereq::Has(Id::new("virtue.the_gift"))),
        "`ArMDE:3956`/`ArMDE:3957` state a Gift requirement, never an Order one"
    );

    let gifted = validate(
        &entity(
            "companion",
            vec![
                Selection::new(Id::new("virtue.the_gift")),
                Selection::new(gentle.clone()),
            ],
        ),
        &rs,
    );
    assert!(
        !gifted
            .issues
            .iter()
            .any(|i| i.context.as_ref() == Some(&gentle)),
        "a Gifted companion may take Gentle Gift: {:?}",
        gifted.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );

    let ungifted = validate(
        &entity("companion", vec![Selection::new(gentle.clone())]),
        &rs,
    );
    assert!(
        ungifted
            .issues
            .iter()
            .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&gentle)),
        "and an unGifted one may not — there is no Gift for it to soften"
    );
}

/// `ArMDE:5643` — "He **knows Hermetic magic** and can cast spells and enchant
/// items like other magi." Hermetic magic presupposes The Gift, so the Flaw may
/// only be taken alongside it.
///
/// Without the prerequisite the Flaw is **3 points for nothing**: it is *Major,
/// Story*, and the companion profile permits `story` at `max: 1` with
/// `max_major_flaws: null` and `gift_policy: "allowed"` — *allowed*, not
/// required — so a companion who simply never takes The Gift banks a Major
/// Flaw's worth of Virtue points for Hermetic training he cannot have.
///
/// A grog cannot reach it and is not the case to test: `story` is absent from
/// his `permitted_categories`, his `flaw_category_caps` set
/// `{"category": "story", "max": 0}`, his `max_major_flaws` is `0`, and
/// `virtue.the_gift` is in his `forbidden_traits`. Four independent barriers,
/// none of which is this rule.
///
/// The Gift is **required, not granted** — `virtue.the_gift` is `Free`, so
/// requiring it costs the player nothing, and granting it would let the Flaw
/// bootstrap the very permission `effective::has_the_gift` reads to decide
/// whether a companion may touch the `hermetic` category at all.
#[test]
fn abandoned_apprentice_requires_the_gift() {
    let rs = load_ruleset();
    let abandoned = Id::new("flaw.abandoned_apprentice");
    assert_eq!(
        rs.item(&abandoned)
            .expect("flaw.abandoned_apprentice must ship")
            .prerequisites,
        Some(Prereq::Has(Id::new("virtue.the_gift"))),
        "`ArMDE:5643` has him casting spells, which presupposes The Gift"
    );

    let gifted = validate(
        &entity(
            "companion",
            vec![
                Selection::new(Id::new("virtue.the_gift")),
                Selection::new(abandoned.clone()),
            ],
        ),
        &rs,
    );
    assert!(
        !gifted
            .issues
            .iter()
            .any(|i| i.context.as_ref() == Some(&abandoned)),
        "a Gifted companion may have been abandoned mid-training: {:?}",
        gifted.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );

    let ungifted = validate(
        &entity("companion", vec![Selection::new(abandoned.clone())]),
        &rs,
    );
    assert!(
        ungifted
            .issues
            .iter()
            .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&abandoned)),
        "and an unGifted one may not — he would bank 3 Flaw points for Hermetic \
         training he cannot have"
    );
}

/// **Pinned baseline for D3.** D56/A0's whole Group A rewire (sub-slices 1–3)
/// deliberately leaves the shipped `flaw.abandoned_apprentice` untouched — it
/// carries no `Effect::ConfersHermeticTraining` yet, so `is_hermetically_trained`
/// still reads him as untrained, and his XP shape is exactly what it was on
/// `main` before this design note: later life is his GENERAL pool (225 = 15
/// years × 15/yr, ArMDE:2392), with no restricted, Abilities-only LaterLife
/// pool the way a real magus gets. D3 is what must flip this pin — attaching
/// the effect and building the truncated per-year block — and this test is
/// the baseline it flips: if D3 lands and this test is still green unchanged,
/// D3 did not actually wire anything.
#[test]
fn abandoned_apprentice_xp_shape_is_unchanged_pending_d3() {
    let rs = load_ruleset_with_spells();
    let mut e = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.the_gift")),
            Selection::new(Id::new("flaw.abandoned_apprentice")),
        ],
    );
    e.ability_funding = AbilityFunding::LifeStages;
    e.age = Some(20);
    e.life_stages = Some(LifeStagePlan {
        native_language: Some("German".to_string()),
        ..LifeStagePlan::default()
    });
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.living_language"),
        parameter: Some("German".to_string()),
        score: 5,
        specialty: None,
    }];

    let allocation = checked_xp_allocation(&e, &rs).expect("within the solve bound");
    assert_eq!(
        allocation.general_pool, 225,
        "later life (15yr x 15/yr) is still the general pool, unchanged"
    );
    assert!(
        !allocation.restricted.iter().any(|p| matches!(
            p.origin,
            XpPoolOrigin::LifeStage {
                block: LifeStageBlock::LaterLife
            }
        )),
        "no restricted LaterLife pool yet — that is D3's job: {:?}",
        allocation.restricted
    );
}

/// ":3845" — "You may not have The Gift, but if your Gift was not completely
/// destroyed, you may have some Supernatural Abilities."
///
/// Expressed as a symmetric `incompatible_with`, not as a `Nor` prerequisite,
/// because that is how this catalogue already states a flat "may not have The
/// Gift": `virtue.devil_child`, `virtue.faerie_doctor`, `virtue.nephilim` and
/// `virtue.spirit_votary` all do it that way, and `virtue.the_gift` lists each
/// of them back. The one `Nor` in the data (`flaw.offensive_to_beings`, `ArMDE:6530`)
/// is there because that rule is *conditional* — "unless you have the Gentle
/// Gift" — which an incompatibility cannot express. Failed Apprentice's is not.
#[test]
fn failed_apprentice_is_incompatible_with_the_gift() {
    let rs = load_ruleset();
    let failed = Id::new("virtue.failed_apprentice");
    let gift = Id::new("virtue.the_gift");
    assert!(
        rs.item(&failed)
            .expect("virtue.failed_apprentice must ship")
            .incompatible_with
            .contains(&gift),
        "`ArMDE:3845` bars the pairing outright"
    );

    let result = validate(
        &entity(
            "companion",
            vec![Selection::new(gift), Selection::new(failed.clone())],
        ),
        &rs,
    );
    assert!(
        result.issues.iter().any(|i| i.code == "incompatible"),
        "a Failed Apprentice lost his Gift: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

/// ":3420" — "This Virtue may be taken by a child character who has the Gift
/// and who has been accepted by an experienced Hermetic magus, with the
/// troupe's approval." (The descriptor line above it, ":3419", reads "*Free.
/// Social Status*".)
///
/// Only the Gift half is modelled: acceptance by a magus and troupe approval
/// are table decisions with nothing on the character sheet to check them
/// against.
#[test]
fn the_apprentice_virtue_requires_the_gift() {
    let rs = load_ruleset();
    let apprentice = Id::new("virtue.apprentice");
    assert_eq!(
        rs.item(&apprentice)
            .expect("virtue.apprentice must ship")
            .prerequisites,
        Some(Prereq::Has(Id::new("virtue.the_gift")))
    );

    let result = validate(
        &entity("companion", vec![Selection::new(apprentice.clone())]),
        &rs,
    );
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&apprentice)),
        "an unGifted child is nobody's discipulus: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

/// ":6805" — "The character has The Gift but cannot access its power, having
/// temporarily lost his magical ability through mishap or some other
/// misfortune." A Flaw that states outright that its bearer has The Gift must
/// require it, and with `ArMDE:2840` now conditional the pair is what makes a
/// Suppressed-Gift **companion** — the character `ArMDE:6809` describes — a legal
/// build rather than one the profile refuses.
#[test]
fn a_gifted_companion_may_take_suppressed_gift() {
    let rs = load_ruleset();
    let suppressed = Id::new("flaw.suppressed_gift");
    assert_eq!(
        rs.item(&suppressed)
            .expect("flaw.suppressed_gift must ship")
            .prerequisites,
        Some(Prereq::Has(Id::new("virtue.the_gift")))
    );

    let result = validate(
        &entity(
            "companion",
            vec![
                Selection::new(Id::new("virtue.the_gift")),
                Selection::new(suppressed.clone()),
            ],
        ),
        &rs,
    );
    assert!(
        !result
            .issues
            .iter()
            .any(|i| i.context.as_ref() == Some(&suppressed)),
        "`ArMDE:6809` puts this Flaw on a companion: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

// --- Row 19: "taken as" (B2) -------------------------------------------------
//
// `ArMDE:5083` makes Sufi's dual category an explicit player CHOICE, not membership
// in both at once: "This Virtue may be taken by both male and female
// characters, either as a Minor Social Status Virtue or a Minor Supernatural
// Virtue." Recorded as a `params` entry under `ParameterDomain::Category`
// (`taken_as`), resolved everywhere by `PointItem::categories_for` so the five
// membership sites — permitted/forbidden categories, category caps, Gift
// detection, grant-constraint filtering — cannot silently disagree about which
// category a taken-as selection counts as. See `RULES.md`, "Full core
// Virtue/Flaw catalogue".

/// Every shipped item declaring a `taken_as` (`ParameterDomain::Category`)
/// parameter, paired with its key and the line making the choice explicit —
/// `(id, param key, line)`.
const TAKEN_AS_ITEMS: &[(&str, &str, u32)] = &[
    ("flaw.curse_of_slander", "taken_as", 5882),
    ("virtue.sufi", "taken_as", 5083),
];

#[test]
fn shipped_taken_as_items_declare_only_their_own_categories() {
    let rs = load_ruleset();

    for (id, key, line) in TAKEN_AS_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        let param = item
            .parameters
            .iter()
            .find(|p| p.key == *key)
            .unwrap_or_else(|| {
                panic!(
                    "{id} must declare a '{key}' parameter, not {:?}",
                    item.parameters
                )
            });
        assert_eq!(
            param.domain,
            ParameterDomain::Category,
            "{id}'s '{key}' records which of its OWN categories was chosen \
             (ArMDE:{line}), not a \
             closed list of its own"
        );
        assert!(
            !param.values.is_empty(),
            "{id}'s '{key}' must declare at least one value"
        );
        for value in &param.values {
            assert!(
                item.categories.iter().any(|c| c == value.as_str()),
                "{id}'s '{key}' value '{value}' must be one of its own \
                 categories {:?} (ArMDE:{line})",
                item.categories
            );
        }
    }
}

#[test]
fn shipped_taken_as_items_cap_at_one_copy() {
    let rs = load_ruleset();

    for (id, _key, line) in TAKEN_AS_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert_eq!(
            item.max_total, 1,
            "{id} offers a choice between readings of ONE item, not several \
             items (ArMDE:{line})"
        );
    }
}

/// A minimal synthetic ruleset for the taken-as cap half below: no shipped
/// profile caps the `supernatural` category (the magus's own
/// `virtue_category_caps` entry is `hermetic`, major-only), so this proves the
/// resolution against a profile that does, rather than asserting nothing.
fn ruleset_with_supernatural_cap() -> Ruleset {
    let items = r#"[
        {
            "id": "virtue.sufi",
            "kind": "virtue",
            "magnitude": "minor",
            "categories": ["social_status", "supernatural"],
            "classification": "narrative",
            "parameters": [
                { "key": "taken_as", "type": "ref", "domain": "category",
                  "values": ["social_status", "supernatural"] }
            ],
            "max_total": 1
        },
        {
            "id": "virtue.filler_personality",
            "kind": "virtue",
            "magnitude": "free",
            "categories": ["personality"],
            "classification": "narrative"
        }
    ]"#;
    let type_profiles = r#"[
        {
            "id": "grog",
            "budget": {
                "virtue_points": 50,
                "flaw_points": 50,
                "virtue_category_caps": [
                    { "category": "supernatural", "max": 0, "hard": true }
                ]
            },
            "permitted_categories": ["social_status", "supernatural", "personality"],
            "creation_phases": ["virtues_flaws"]
        }
    ]"#;
    Ruleset::from_sources(RulesetSources {
        id: "taken-as-test",
        version: "1.0",
        point_items: items,
        type_profiles,
        ..RulesetSources::default()
    })
    .unwrap()
}

/// The first-failing test for row 19 (B2): a grog Sufi taken as Social Status
/// must not count as a Supernatural Virtue for any membership rule — the
/// defect `RULES.md`'s "Unmodelled, and recorded rather than resolved" note
/// used to record. Both categories are open to a grog either way (`ArMDE:5079`,
/// `ArMDE:5083`), so the permitted/forbidden half is exercised against the real
/// shipped catalogue to pin "still legal"; the cap half needs the synthetic
/// ruleset above since no shipped profile caps `supernatural`.
#[test]
fn a_grog_sufi_taken_as_social_status_is_not_a_supernatural_virtue() {
    let rs = load_ruleset();
    let sufi = Id::new("virtue.sufi");
    let taken_as_social_status = Selection::with_params(
        sufi.clone(),
        BTreeMap::from([("taken_as".to_string(), Id::new("social_status"))]),
    );
    let codes = issue_codes(&entity("grog", vec![taken_as_social_status]), &rs);
    for code in ["category_not_permitted", "forbidden_category"] {
        assert!(
            !codes.contains(&code.to_string()),
            "Social Status is open to a grog either way \
             (ArMDE:5083): {codes:?}"
        );
    }

    let capped_rs = ruleset_with_supernatural_cap();
    let clean_selection = Selection::with_params(
        sufi.clone(),
        BTreeMap::from([("taken_as".to_string(), Id::new("social_status"))]),
    );
    let clean = issue_codes(&entity("grog", vec![clean_selection]), &capped_rs);
    assert!(
        !clean.iter().any(|c| c.starts_with("too_many_supernatural")),
        "taken as Social Status must not count against a Supernatural cap: {clean:?}"
    );

    let capped_selection = Selection::with_params(
        sufi,
        BTreeMap::from([("taken_as".to_string(), Id::new("supernatural"))]),
    );
    let capped = issue_codes(&entity("grog", vec![capped_selection]), &capped_rs);
    assert!(
        capped
            .iter()
            .any(|c| c.starts_with("too_many_supernatural")),
        "taken as Supernatural must still count against the cap: {capped:?}"
    );
}

/// Old-save regression: a `virtue.sufi` selection with no `taken_as` at all
/// (every save written before this slice) must still validate as a legal item
/// — `taken_as` is a newly-declared param on an already-shipped item, so the
/// accepted precedent (the `migration.rs::SCHEMA_VERSION` doc comment, and
/// RULES.md, "Realm parameter domain, and mutually exclusive values" →
/// "Save impact — accepted, not migrated") is a non-blocking `missing_param`,
/// never a hard failure or a silently invented
/// choice — and must still resolve BOTH categories for every membership test,
/// exactly as before this slice.
#[test]
fn a_pre_existing_sufi_selection_with_no_taken_as_reports_missing_param_only() {
    let rs = load_ruleset();
    let sufi = Id::new("virtue.sufi");
    let codes = issue_codes(&entity("grog", vec![Selection::new(sufi)]), &rs);
    assert!(
        codes.contains(&"missing_param".to_string()),
        "an old save naming no taken_as value must be flagged so the player \
         can make the choice explicit: {codes:?}"
    );
    for code in ["category_not_permitted", "forbidden_category"] {
        assert!(
            !codes.contains(&code.to_string()),
            "an unresolved taken_as must fall back to the whole category list, \
             not narrow to nothing: {codes:?}"
        );
    }
}

/// `max_total: 1` is a real behaviour change for a save holding two Sufis
/// (however they were reached — a hand-edited file, or a pre-slice save from
/// before `max_total` existed on this item): the two selections' `params`
/// differ (`taken_as: "social_status"` vs `"supernatural"`), so they are
/// DIFFERENT targets and `max_per_target` (the `(item_ref, params)` duplicate
/// key) never fires — only `validate_total_selection_cap`, keyed on
/// `item_ref` alone, does. This must surface as `too_many_selections`, loud
/// and actionable, never a silent drop of the second copy.
#[test]
fn two_sufis_with_different_taken_as_trip_the_total_cap_not_a_silent_drop() {
    let rs = load_ruleset();
    let sufi = Id::new("virtue.sufi");
    let two_sufis = entity(
        "grog",
        vec![
            Selection::with_params(
                sufi.clone(),
                BTreeMap::from([("taken_as".to_string(), Id::new("social_status"))]),
            ),
            Selection::with_params(
                sufi,
                BTreeMap::from([("taken_as".to_string(), Id::new("supernatural"))]),
            ),
        ],
    );
    let codes = issue_codes(&two_sufis, &rs);
    assert!(
        !codes.contains(&"duplicate_selection".to_string()),
        "different taken_as values are different targets, so max_per_target \
         must not be the mechanism that catches this: {codes:?}"
    );
    assert!(
        codes.contains(&"too_many_selections".to_string()),
        "two readings of the SAME Virtue must still be capped at one \
         (ArMDE:5083): {codes:?}"
    );
}

/// A synthetic ruleset isolating each of the FOUR `PointItem::categories_for`
/// call sites data_integrity.rs can reach through the public `validate()` API
/// (permitted categories, forbidden categories, category caps, Gift
/// detection — the fifth, grant-constraint filtering, is unreachable without a
/// full House/grant fixture and is instead pinned directly against
/// `open_pick_satisfies` by `open_pick_satisfies_is_taken_as_aware` in
/// `validation/mod.rs`'s own test module).
///
/// Four unrelated dual-category items, each with its own made-up category
/// pair, so the four tests below cannot cross-contaminate: `permcheck`'s
/// `p_no` is the only category excluded from `permitted_categories`;
/// `forbidcheck`'s `f_yes` is the only one in `forbidden_categories`;
/// `capcheck`'s `c_capped` is the only one under `virtue_category_caps`;
/// `giftcheck`'s `g_gift` is the only one in `gift_categories`. Every OTHER
/// category is deliberately permitted and uncapped, so each test observes
/// exactly one mechanism.
fn ruleset_with_isolated_taken_as_categories() -> Ruleset {
    let items = r#"[
        { "id": "virtue.permcheck", "kind": "virtue", "magnitude": "minor",
          "categories": ["p_yes", "p_no"], "classification": "narrative",
          "parameters": [{ "key": "taken_as", "type": "ref", "domain": "category",
                            "values": ["p_yes", "p_no"] }],
          "max_total": 1 },
        { "id": "virtue.forbidcheck", "kind": "virtue", "magnitude": "minor",
          "categories": ["f_no", "f_yes"], "classification": "narrative",
          "parameters": [{ "key": "taken_as", "type": "ref", "domain": "category",
                            "values": ["f_no", "f_yes"] }],
          "max_total": 1 },
        { "id": "virtue.capcheck", "kind": "virtue", "magnitude": "minor",
          "categories": ["c_free", "c_capped"], "classification": "narrative",
          "parameters": [{ "key": "taken_as", "type": "ref", "domain": "category",
                            "values": ["c_free", "c_capped"] }],
          "max_total": 1 },
        { "id": "virtue.giftcheck", "kind": "virtue", "magnitude": "minor",
          "categories": ["g_plain", "g_gift"], "classification": "narrative",
          "parameters": [{ "key": "taken_as", "type": "ref", "domain": "category",
                            "values": ["g_plain", "g_gift"] }],
          "max_total": 1 },
        { "id": "virtue.filler_personality", "kind": "virtue", "magnitude": "free",
          "categories": ["personality"], "classification": "narrative" }
    ]"#;
    let type_profiles = r#"[
        {
            "id": "testtype",
            "budget": { "virtue_points": 50, "flaw_points": 50,
                "virtue_category_caps": [
                    { "category": "c_capped", "max": 0, "hard": true }
                ]
            },
            "permitted_categories": [
                "p_yes", "f_no", "f_yes", "c_free", "c_capped", "g_plain",
                "g_gift", "personality"
            ],
            "forbidden_categories": ["f_yes"],
            "gift_policy": "forbidden",
            "gift_categories": ["g_gift"],
            "creation_phases": ["virtues_flaws"]
        }
    ]"#;
    Ruleset::from_sources(RulesetSources {
        id: "taken-as-test",
        version: "1.0",
        point_items: items,
        type_profiles,
        ..RulesetSources::default()
    })
    .unwrap()
}

fn taken_as(item: &str, value: &str) -> Selection {
    Selection::with_params(
        Id::new(item),
        BTreeMap::from([("taken_as".to_string(), Id::new(value))]),
    )
}

/// Site 1/5: `validate_permitted_categories` (`validation/selections.rs`).
#[test]
fn validate_permitted_categories_is_taken_as_aware() {
    let rs = ruleset_with_isolated_taken_as_categories();
    const CODE: &str = "category_not_permitted";

    let whole_list = issue_codes(
        &entity(
            "testtype",
            vec![Selection::new(Id::new("virtue.permcheck"))],
        ),
        &rs,
    );
    assert!(
        !whole_list.contains(&CODE.to_string()),
        "no taken_as recorded must still resolve the whole list (legacy ANY \
         semantics), passing via p_yes: {whole_list:?}"
    );

    let permitted = issue_codes(
        &entity("testtype", vec![taken_as("virtue.permcheck", "p_yes")]),
        &rs,
    );
    assert!(
        !permitted.contains(&CODE.to_string()),
        "taken as p_yes must pass: {permitted:?}"
    );

    let not_permitted = issue_codes(
        &entity("testtype", vec![taken_as("virtue.permcheck", "p_no")]),
        &rs,
    );
    assert!(
        not_permitted.contains(&CODE.to_string()),
        "taken as p_no must be judged on p_no ALONE, not rescued by the whole \
         list's p_yes: {not_permitted:?}"
    );
}

/// Site 2/5: `validate_forbidden_categories` (`validation/selections.rs`).
#[test]
fn validate_forbidden_categories_is_taken_as_aware() {
    let rs = ruleset_with_isolated_taken_as_categories();
    const CODE: &str = "forbidden_category";

    let whole_list = issue_codes(
        &entity(
            "testtype",
            vec![Selection::new(Id::new("virtue.forbidcheck"))],
        ),
        &rs,
    );
    assert!(
        !whole_list.contains(&CODE.to_string()),
        "no taken_as recorded must still resolve the whole list (legacy EVERY \
         semantics): only f_yes is forbidden, and f_no survives as the other \
         reading: {whole_list:?}"
    );

    let clean = issue_codes(
        &entity("testtype", vec![taken_as("virtue.forbidcheck", "f_no")]),
        &rs,
    );
    assert!(
        !clean.contains(&CODE.to_string()),
        "taken as f_no must not be blocked: {clean:?}"
    );

    let blocked = issue_codes(
        &entity("testtype", vec![taken_as("virtue.forbidcheck", "f_yes")]),
        &rs,
    );
    assert!(
        blocked.contains(&CODE.to_string()),
        "taken as f_yes narrows the 'every category is forbidden' test to a \
         singleton that IS forbidden, even though the whole-list case above is \
         clean: {blocked:?}"
    );
}

/// The `category` argument of both category issues must name the category the
/// player actually chose, not the descriptor's first-listed one.
///
/// The tie-break that picks a representative category is only defensible while
/// EVERY category failed the test. Once a selection records `taken_as`, exactly
/// one category was judged — and naming a different one tells the player their
/// Virtue was rejected for a reading they explicitly did not take. Both
/// fixtures below are arranged so the two answers differ: `permcheck` is
/// `["p_yes", "p_no"]` and fails when taken as `p_no`; `forbidcheck` is
/// `["f_no", "f_yes"]` and fails when taken as `f_yes`.
#[test]
fn taken_as_category_issues_name_the_chosen_category() {
    let rs = ruleset_with_isolated_taken_as_categories();

    let arg = |selection: Selection, code: &str| -> String {
        validate(&entity("testtype", vec![selection]), &rs)
            .issues
            .into_iter()
            .find(|i| i.code == code)
            .unwrap_or_else(|| panic!("expected a {code} issue"))
            .args
            .get("category")
            .cloned()
            .unwrap_or_else(|| panic!("a {code} issue must carry a 'category' argument"))
    };

    assert_eq!(
        arg(
            taken_as("virtue.permcheck", "p_no"),
            "category_not_permitted"
        ),
        "p_no",
        "the rejected reading is p_no; p_yes is permitted and was not chosen"
    );
    assert_eq!(
        arg(
            taken_as("virtue.forbidcheck", "f_yes"),
            "forbidden_category"
        ),
        "f_yes",
        "the forbidden reading is f_yes; f_no is open and was not chosen"
    );
}

/// Site 3/5: the category caps (`validation/caps.rs`).
#[test]
fn category_caps_are_taken_as_aware() {
    let rs = ruleset_with_isolated_taken_as_categories();
    const CODE: &str = "too_many_c_capped_virtues";

    let taken_as_free = issue_codes(
        &entity("testtype", vec![taken_as("virtue.capcheck", "c_free")]),
        &rs,
    );
    assert!(
        !taken_as_free.contains(&CODE.to_string()),
        "taken as c_free must not count against the c_capped cap, even though \
         the item's whole category list carries c_capped too: {taken_as_free:?}"
    );

    let taken_as_capped = issue_codes(
        &entity("testtype", vec![taken_as("virtue.capcheck", "c_capped")]),
        &rs,
    );
    assert!(
        taken_as_capped.contains(&CODE.to_string()),
        "taken as c_capped must still trip the cap: {taken_as_capped:?}"
    );
}

/// Site 4/5: Gift detection (`effective/gift_confidence.rs`).
#[test]
fn gift_detection_is_taken_as_aware() {
    let rs = ruleset_with_isolated_taken_as_categories();
    const CODE: &str = "gift_forbidden";

    let taken_as_plain = issue_codes(
        &entity("testtype", vec![taken_as("virtue.giftcheck", "g_plain")]),
        &rs,
    );
    assert!(
        !taken_as_plain.contains(&CODE.to_string()),
        "taken as g_plain must not read as holding The Gift, even though the \
         item's whole category list carries g_gift too: {taken_as_plain:?}"
    );

    let taken_as_gift = issue_codes(
        &entity("testtype", vec![taken_as("virtue.giftcheck", "g_gift")]),
        &rs,
    );
    assert!(
        taken_as_gift.contains(&CODE.to_string()),
        "taken as g_gift must still read as holding The Gift, which this \
         profile forbids: {taken_as_gift:?}"
    );
}

// --- Curse of Slander is General *or* Supernatural (row 13, B4) -------------
//
// `ArMDE:5882` reads "*Minor, General or Supernatural*", and the book indexes the
// Flaw under both of those headings: `ArMDE:5538` under `### Supernatural, Minor`
// (`ArMDE:5534`) and `ArMDE:5575` under `### General, Minor` (`ArMDE:5564`). That "or" is the
// same explicit either/or `ArMDE:5083` spells out in prose for Sufi, so it is
// modelled the same way — both categories shipped, plus a `taken_as` parameter
// naming the reading in force, and `max_total: 1` because the book offers a
// choice between two readings of ONE Flaw.
//
// The *and*-joined descriptors are a different question and are deliberately
// NOT covered by this mechanism — see `RULES.md`, "row 19, half (b)".

/// The shipped Virtue/Flaw catalogue against a synthetic profile that caps the
/// `supernatural` FLAW category at zero. No shipped profile caps
/// `supernatural` — every one of them merely permits it — so a cap has to be
/// supplied for "does this count as a Supernatural Flaw?" to be observable at
/// all. The item under test is nonetheless the REAL shipped
/// `flaw.curse_of_slander`, not a synthetic stand-in.
fn shipped_items_with_supernatural_flaw_cap() -> Ruleset {
    let type_profiles = r#"[
        {
            "id": "captype",
            "budget": {
                "virtue_points": 50,
                "flaw_points": 50,
                "flaw_category_caps": [
                    { "category": "supernatural", "max": 0, "hard": true }
                ]
            },
            "permitted_categories": [
                "general", "hermetic", "mythic_companion", "personality",
                "social_status", "story", "supernatural"
            ],
            "creation_phases": ["virtues_flaws"]
        }
    ]"#;
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles,
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        houses: Some(SHIPPED_HOUSES),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        ..RulesetSources::default()
    })
    .unwrap()
}

/// The descriptor names two categories joined by *or*, so the parameter must
/// offer exactly those two — a `taken_as` that offered only one would make the
/// "choice" no choice at all, and integrity only checks that every offered
/// value is one of the item's own categories, not that both are offered.
#[test]
fn curse_of_slander_offers_both_categories_its_descriptor_names() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.curse_of_slander"))
        .expect("flaw.curse_of_slander must ship");
    let offered: Vec<&str> = item
        .parameters
        .iter()
        .find(|p| p.key == "taken_as")
        .expect("flaw.curse_of_slander must declare a 'taken_as' parameter")
        .values
        .iter()
        .map(|v| v.as_str())
        .collect();
    assert_eq!(
        offered,
        vec!["general", "supernatural"],
        "\"*Minor, General or Supernatural*\" (ArMDE:5882) offers exactly these \
         two readings"
    );
}

/// The first-failing behavioural test for row 13 (B4): the whole point of
/// shipping `supernatural` alongside `general` is that it must NOT make every
/// bearer count as holding a Supernatural Flaw — which is precisely the
/// objection `docs/open-todos.md` row 13 raised against adding the second
/// category flatly.
#[test]
fn curse_of_slander_taken_as_general_is_not_a_supernatural_flaw() {
    let rs = shipped_items_with_supernatural_flaw_cap();
    const CODE: &str = "too_many_supernatural_flaws";

    let as_general = issue_codes(
        &entity(
            "captype",
            vec![taken_as("flaw.curse_of_slander", "general")],
        ),
        &rs,
    );
    assert!(
        !as_general.contains(&CODE.to_string()),
        "taken as General, Curse of Slander is not a Supernatural Flaw \
         (ArMDE:5882): {as_general:?}"
    );

    let as_supernatural = issue_codes(
        &entity(
            "captype",
            vec![taken_as("flaw.curse_of_slander", "supernatural")],
        ),
        &rs,
    );
    assert!(
        as_supernatural.contains(&CODE.to_string()),
        "taken as Supernatural, it must count against a Supernatural cap — \
         otherwise the second category is decorative: {as_supernatural:?}"
    );
}

// --- Two Flaws the book indexes under General were magus-only (row 13) -------
//
// "Hermetic" gates on The Gift, not on magus-hood: "Only characters with The
// Gift can take these Virtues and Flaws, and some are only applicable to
// Hermetic magi who have already completed their training." (`ArMDE:2880`) A
// companion "may not take Hermetic Virtues and Flaws, unless you have The Gift"
// (`ArMDE:2840`); a grog may not at all (`ArMDE:2829`) and is barred from The Gift itself
// (`ArMDE:2830`).
//
// Offensive to (Beings) is "*Minor, Hermetic and General*" (`ArMDE:6525`) and
// Unbearable to (Beings) "*Minor, Hermetic or General*" (`ArMDE:6892`) — dual-indexed
// in the book's own lists (Offensive at `ArMDE:5445` Hermetic and `ArMDE:5608` General;
// Unbearable at `ArMDE:5455` and `ArMDE:5629`). Both shipped `["hermetic"]` alone, and
// `hermetic` is forbidden for grog, companion and mythic companion, so only a
// magus could take them.
//
// They cannot carry `hermetic` as a SECOND category either, because
// `gift_categories` is `["hermetic"]` for every shipped profile and
// `effective::has_the_gift` reads any such category as "has The Gift" — see
// `a_companion_holding_offensive_to_beings_is_not_gifted` below. So the
// eligibility the category was enforcing by accident is modelled explicitly, as
// prerequisites.

/// The two Flaws carry the permissive `general` category alone, plus the
/// eligibility the rulebook states in prose:
///
/// > "Only characters with The Gift or Magical Air may take this Flaw, and it
/// > cannot be combined with the Blatant Gift." — Unbearable, `ArMDE:6895`
///
/// > "Characters with The Gift may take this Flaw only if they have the Gentle
/// > Gift … Characters with Magical Air may not take it at all." — Offensive,
/// > `ArMDE:6530`
///
/// > "UnGifted characters may take this Virtue only if they have the Flaw
/// > Magical Air." — Inoffensive, `ArMDE:4139`
///
/// (ArMDE:6895, :6530, :4139.)
#[test]
fn the_beings_items_carry_general_plus_an_explicit_eligibility_gate() {
    let rs = load_ruleset();
    let the_gift = Prereq::Has(Id::new("virtue.the_gift"));
    let gifted_or_magical_air = Prereq::Any(vec![
        the_gift.clone(),
        Prereq::Has(Id::new("flaw.magical_air")),
    ]);

    for (id, expected_prereq, expected_incompatible) in [
        (
            "flaw.unbearable_to_beings",
            gifted_or_magical_air.clone(),
            vec!["flaw.blatant_gift"],
        ),
        (
            "flaw.offensive_to_beings",
            // "unGifted, or Gifted with the Gentle Gift". The `Nor` variant's
            // wire tag is `"none"`; it is the boolean NOR, not "no prereq".
            Prereq::Any(vec![
                Prereq::Nor(vec![the_gift.clone()]),
                Prereq::Has(Id::new("virtue.gentle_gift")),
            ]),
            vec!["flaw.magical_air"],
        ),
        (
            "virtue.inoffensive_to_beings",
            gifted_or_magical_air.clone(),
            vec![],
        ),
    ] {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must ship in the catalogue"));
        assert_eq!(
            item.categories,
            vec!["general".to_string()],
            "{id} is indexed under General, and must not carry `hermetic`: that \
             slug is the profiles' `gift_categories` and would make an unGifted \
             bearer count as Gifted"
        );
        assert_eq!(
            item.prerequisites.as_ref(),
            Some(&expected_prereq),
            "{id} must state its eligibility as a prerequisite now that the \
             category no longer enforces it by accident"
        );
        assert_eq!(
            item.incompatible_with
                .iter()
                .map(Id::as_str)
                .collect::<Vec<_>>(),
            expected_incompatible,
            "{id}'s stated incompatibilities"
        );
    }
}

/// A companion is neither a magus nor Gifted, and the book lets him take both:
/// Offensive because he is unGifted (`ArMDE:6530` restricts only the *Gifted* case,
/// which proves the unGifted case is the default), Unbearable because he has
/// Magical Air (`ArMDE:6895`).
#[test]
fn a_companion_may_take_the_two_beings_flaws() {
    let rs = load_ruleset();
    let offensive = Id::new("flaw.offensive_to_beings");
    let unbearable = Id::new("flaw.unbearable_to_beings");
    let e = entity(
        "companion",
        vec![
            Selection::new(Id::new("flaw.magical_air")),
            Selection::with_params(
                offensive.clone(),
                BTreeMap::from([("being".to_string(), Id::new("being.animals"))]),
            ),
        ],
    );
    // Offensive is incompatible with Magical Air (`ArMDE:6530`), so the two are tested
    // on separate characters.
    let offensive_alone = entity(
        "companion",
        vec![Selection::with_params(
            offensive.clone(),
            BTreeMap::from([("being".to_string(), Id::new("being.animals"))]),
        )],
    );
    let unbearable_with_air = entity(
        "companion",
        vec![
            Selection::new(Id::new("flaw.magical_air")),
            Selection::with_params(
                unbearable.clone(),
                BTreeMap::from([("being".to_string(), Id::new("being.demons"))]),
            ),
        ],
    );

    for (case, target) in [
        (&offensive_alone, &offensive),
        (&unbearable_with_air, &unbearable),
    ] {
        let issues = validate(case, &rs).issues;
        let blocking: Vec<&String> = issues
            .iter()
            .filter(|i| {
                i.context.as_ref() == Some(target)
                    && matches!(
                        i.code.as_str(),
                        "forbidden_category"
                            | "category_not_permitted"
                            | "prereq_not_met"
                            | "incompatible"
                    )
            })
            .map(|i| &i.code)
            .collect();
        assert!(
            blocking.is_empty(),
            "a companion must be able to take {target}: {blocking:?}"
        );
    }

    // And the pairing the book forbids is still caught, by the honest reason.
    assert!(
        validate(&e, &rs)
            .issues
            .iter()
            .any(|i| i.code == "incompatible"),
        "Magical Air plus Offensive to (Beings) is barred by :6530"
    );
}

/// The other half of dropping `hermetic`: without the prerequisites, a grog
/// could take Unbearable to (Beings) with neither The Gift nor Magical Air —
/// trading one wrong output for another. `ArMDE:6895` bars it, and a grog can have
/// neither (`ArMDE:2830`).
#[test]
fn a_grog_may_not_take_unbearable_to_beings_without_the_gift_or_magical_air() {
    let rs = load_ruleset();
    let unbearable = Id::new("flaw.unbearable_to_beings");
    let result = validate(
        &entity(
            "grog",
            vec![Selection::with_params(
                unbearable.clone(),
                BTreeMap::from([("being".to_string(), Id::new("being.demons"))]),
            )],
        ),
        &rs,
    );
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&unbearable)),
        "the Gift-or-Magical-Air gate (ArMDE:6895) must fire on its own now that the \
         `hermetic` category no longer blocks the Flaw: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

/// The find that belongs in the same slice: `virtue.inoffensive_to_beings` ships
/// the permissive `general` category and shipped **no** eligibility gate at all,
/// so an unGifted character with no Magical Air took it clean.
/// `ArMDE:4139`: "UnGifted characters may take this Virtue only if they have the Flaw
/// Magical Air."
#[test]
fn an_ungifted_character_needs_magical_air_for_inoffensive_to_beings() {
    let rs = load_ruleset();
    let inoffensive = Id::new("virtue.inoffensive_to_beings");
    let being = BTreeMap::from([("being".to_string(), Id::new("being.animals"))]);

    let ungifted = validate(
        &entity(
            "companion",
            vec![Selection::with_params(inoffensive.clone(), being.clone())],
        ),
        &rs,
    );
    assert!(
        ungifted
            .issues
            .iter()
            .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&inoffensive)),
        "an unGifted companion with no Magical Air may not take it (ArMDE:4139): {:?}",
        ungifted.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );

    let with_air = validate(
        &entity(
            "companion",
            vec![
                Selection::new(Id::new("flaw.magical_air")),
                Selection::with_params(inoffensive.clone(), being),
            ],
        ),
        &rs,
    );
    assert!(
        !with_air
            .issues
            .iter()
            .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&inoffensive)),
        "and Magical Air satisfies it: {:?}",
        with_air.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

/// Why `hermetic` may NOT be added back as a second category, pinned so a later
/// "completion" of the dual-category data fails loudly. `has_the_gift`
/// (`effective/gift_confidence.rs`) counts any selection carrying a category in
/// the profile's `gift_categories` — `["hermetic"]` everywhere — so tagging these
/// Flaws Hermetic would make an unGifted companion count as Gifted and silently
/// hand him the Gift's free Supernatural-Ability slot
/// (ArMDE:2874).
#[test]
fn a_companion_holding_offensive_to_beings_is_not_gifted() {
    let rs = load_ruleset();
    let profile = rs
        .profile(&Id::new("companion"))
        .expect("the companion profile must ship");
    assert!(
        profile.gift_categories.contains("hermetic"),
        "the companion profile detects The Gift by the hermetic category"
    );

    let e = entity(
        "companion",
        vec![Selection::with_params(
            Id::new("flaw.offensive_to_beings"),
            BTreeMap::from([("being".to_string(), Id::new("being.animals"))]),
        )],
    );
    assert_eq!(
        arm_rules::supernatural_free_slots(&e, &rs, profile).total,
        0,
        "Offensive to (Beings) is not a Gift Flaw, so it must confer no free \
         Supernatural-Ability slot"
    );
}

// --- `index_categories`: the book's index, kept apart from membership (row 18) -
//
// `7ea4f5b` moved the two Beings Flaws from `categories: ["hermetic"]` to
// `["general"]` because `hermetic` is what `effective::has_the_gift` reads, and
// an unGifted companion holding one was thereby counted as Gifted. That was
// right, but it also silently changed the answer to a second, unrelated
// question: `validate_house`'s "a magus should take at least one Hermetic Flaw"
// guideline (`ArMDE:2860`) read the very same `gift_categories`, so a magus whose one
// Hermetic Flaw was Unbearable to (Beings) was told he had none — though the
// book's own Flaw index lists it under `### Hermetic, Minor` (`ArMDE:5455`).
//
// The two questions now have two fields. `PointItem::index_categories` records
// the headings the book's index files an entry under BEYOND its membership
// `categories`; it is provenance, never membership.
// `EntityTypeProfile::hermetic_flaw_categories` says which categories the
// guideline counts. `validate_house` is the ONLY reader of `index_categories` —
// Gift detection, caps, permitted/forbidden lists, grants, `items_by_category`
// and the Markdown export all stay blind to it, which the leak guards below pin.

/// Every shipped item carrying `index_categories`, with the index heading and
/// the line the book lists it at. Frozen, in the manner of `TAKEN_AS_ITEMS`: the
/// set is small, hand-verified against the `### <Category>, <Magnitude>` blocks
/// under `## List of Virtues` (`ArMDE:3004`) and `## List of Flaws` (`ArMDE:5283`), and a
/// silent addition must fail rather than pass.
/// In id order, which is both the shipped file's canonical order and the order
/// `Ruleset::items()` walks its `BTreeMap`.
const INDEX_CATEGORY_ITEMS: [(&str, &str, u32); 4] = [
    // *Minor, Hermetic and General* (ArMDE:6525) — `### Hermetic, Minor` is :5417.
    ("flaw.offensive_to_beings", "hermetic", 5445),
    // *Minor, Story and Hermetic* (ArMDE:6635) — `### Hermetic, Minor` is :5417.
    ("flaw.primogeniture_lineage", "hermetic", 5447),
    // *Minor, Hermetic or General* (ArMDE:6892) — `### Hermetic, Minor` is :5417.
    ("flaw.unbearable_to_beings", "hermetic", 5455),
    // *Minor, General and Hermetic* (ArMDE:4134) — `### Hermetic, Minor` is :3087.
    ("virtue.inoffensive_to_beings", "hermetic", 3110),
];

/// The provenance data itself: each of the four items the book indexes under a
/// heading its `categories` deliberately omit carries that heading in
/// `index_categories`, and nowhere else.
#[test]
fn the_shipped_catalogue_records_the_books_own_index_headings() {
    let rs = load_ruleset();

    for (id, heading, line) in INDEX_CATEGORY_ITEMS {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert!(
            item.index_categories.iter().any(|c| c == heading),
            "{id} is listed under `### {heading}` at \
             ArMDE:{line}"
        );
        assert!(
            !item.has_category(heading),
            "{id}'s index heading must stay OUT of its membership categories — \
             that is exactly what 7ea4f5b removed"
        );
    }

    let recorded: Vec<&str> = INDEX_CATEGORY_ITEMS.iter().map(|(id, ..)| *id).collect();
    let shipped: Vec<String> = rs
        .items()
        .filter(|i| !i.index_categories.is_empty())
        .map(|i| i.id.to_string())
        .collect();
    assert_eq!(
        shipped, recorded,
        "a new index_categories entry must be added to this frozen table with \
         the index line it was verified against"
    );
}

/// The leak guard row 18 asks for by name. `index_categories` is provenance, so
/// Gift detection must not see it: a companion holding Offensive to (Beings) —
/// which now records `hermetic` as an index heading — is still unGifted and
/// still gets no free Supernatural-Ability slot (`ArMDE:2874`).
#[test]
fn gift_detection_ignores_index_categories() {
    let rs = load_ruleset();
    let offensive = Id::new("flaw.offensive_to_beings");
    let profile = rs
        .profile(&Id::new("companion"))
        .expect("the companion profile must ship");
    assert!(
        profile.gift_categories.contains("hermetic"),
        "the companion profile detects The Gift by the hermetic category"
    );
    assert!(
        rs.item(&offensive)
            .expect("Offensive to (Beings) must ship")
            .index_categories
            .iter()
            .any(|c| c == "hermetic"),
        "the item under test must actually carry the index heading"
    );

    let e = entity(
        "companion",
        vec![Selection::with_params(
            offensive,
            BTreeMap::from([("being".to_string(), Id::new("being.animals"))]),
        )],
    );
    assert_eq!(
        arm_rules::supernatural_free_slots(&e, &rs, profile).total,
        0,
        "an index heading is not membership, so it may not confer The Gift"
    );
}

/// The rest of the leak guard: no membership or browsing surface may read
/// `index_categories`. A grog forbids `hermetic` outright and no profile permits
/// `mythic_companion` except the mythic one, so an index heading leaking into
/// either gate would show up as a refusal; `items_by_category` is the browsing
/// half.
#[test]
fn index_categories_are_invisible_to_every_membership_surface() {
    let rs = load_ruleset();
    let unbearable = Id::new("flaw.unbearable_to_beings");

    assert!(
        !rs.items_by_category("hermetic").any(|i| i.id == unbearable),
        "items_by_category is a membership query over `categories` alone"
    );

    // A grog forbids `hermetic`; the Flaw is `general`, so the only thing that
    // could refuse it on category grounds is a leak of the index heading. (The
    // Gift-or-Magical-Air prerequisite still fires — that is `ArMDE:6895`, not a
    // category rule.)
    let codes = issue_codes(
        &entity(
            "grog",
            vec![Selection::with_params(
                unbearable,
                BTreeMap::from([("being".to_string(), Id::new("being.demons"))]),
            )],
        ),
        &rs,
    );
    for code in ["forbidden_category", "category_not_permitted"] {
        assert!(
            !codes.contains(&code.to_string()),
            "the grog's forbidden `hermetic` must not see an index heading: {codes:?}"
        );
    }
}

/// **The positive form, restored on purpose.** The book indexes both Beings
/// Flaws under Hermetic, so the `ArMDE:2860` guideline — "You should take at least
/// one Hermetic Flaw" — counts them again. It reads
/// `EntityTypeProfile::hermetic_flaw_categories` against the item's
/// `categories` PLUS its `index_categories`, which is why this can be true
/// while `gift_detection_ignores_index_categories` above is also true.
#[test]
fn the_hermetic_flaw_guideline_counts_the_two_beings_flaws() {
    let rs = load_ruleset();
    let magus_base = || {
        vec![
            Selection::new(Id::new("virtue.the_gift")),
            Selection::new(Id::new("virtue.hermetic_magus")),
        ]
    };

    // Unbearable to (Beings): any magus may take it (`ArMDE:6895`).
    let mut selections = magus_base();
    selections.push(Selection::with_params(
        Id::new("flaw.unbearable_to_beings"),
        BTreeMap::from([("being".to_string(), Id::new("being.demons"))]),
    ));
    let codes = issue_codes(&entity("magus", selections), &rs);
    assert!(
        !codes.contains(&"missing_hermetic_flaw".to_string()),
        "the book lists Unbearable to (Beings) under `### Hermetic, Minor` \
         (ArMDE:5455), so it satisfies :2860: {codes:?}"
    );

    // Offensive to (Beings): a Gifted character needs the Gentle Gift (`ArMDE:6530`).
    let mut selections = magus_base();
    selections.push(Selection::new(Id::new("virtue.gentle_gift")));
    selections.push(Selection::with_params(
        Id::new("flaw.offensive_to_beings"),
        BTreeMap::from([("being".to_string(), Id::new("being.animals"))]),
    ));
    let codes = issue_codes(&entity("magus", selections), &rs);
    assert!(
        !codes.contains(&"missing_hermetic_flaw".to_string()),
        "and Offensive to (Beings) is indexed there too (ArMDE:5445): {codes:?}"
    );

    // The guideline still bites when there is genuinely no Hermetic Flaw.
    let codes = issue_codes(&entity("magus", magus_base()), &rs);
    assert!(
        codes.contains(&"missing_hermetic_flaw".to_string()),
        "a magus with no Hermetic Flaw at all is still advised: {codes:?}"
    );
}

/// The magus profile is the one that states the guideline, because `ArMDE:2860` is a
/// magus bullet. No other shipped profile may claim it: `validate_house`
/// returns early for a non-Order-member, and a stray field would be a silent lie.
#[test]
fn only_the_magus_profile_names_hermetic_flaw_categories() {
    let rs = load_ruleset();
    for profile in rs.profiles() {
        let expected: &[&str] = if profile.order_member {
            &["hermetic"]
        } else {
            &[]
        };
        let actual: Vec<&str> = profile
            .hermetic_flaw_categories
            .iter()
            .map(String::as_str)
            .collect();
        assert_eq!(
            actual,
            expected.to_vec(),
            "profile '{}' (ArMDE:2860 \
             is a magus bullet)",
            profile.id
        );
    }
}

/// Load gate 1: an index heading the item already carries as a membership
/// category is not a divergence — it is a duplicate that would double-count the
/// entry and blur the very distinction the field exists to draw.
#[test]
fn an_index_category_repeating_a_membership_category_fails_the_load() {
    let err = Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: r#"[
          { "id": "flaw.x", "kind": "flaw", "magnitude": "minor",
            "classification": "narrative",
            "categories": ["general", "story"],
            "index_categories": ["story"] },
          { "id": "flaw.filler", "kind": "flaw", "magnitude": "minor",
            "classification": "narrative", "categories": ["personality"] }
        ]"#,
        type_profiles: r#"[{ "id": "grog",
            "budget": { "virtue_points": 3, "flaw_points": 3 },
            "creation_phases": [] }]"#,
        ..RulesetSources::default()
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("flaw.x"), "names the item: {err}");
    assert!(err.contains("index_categories"), "names the field: {err}");
    assert!(err.contains("story"), "names the offending slug: {err}");
}

/// Load gate 2: a repeated index heading is an authoring slip, exactly as a
/// repeated membership category is.
#[test]
fn a_repeated_index_category_fails_the_load() {
    let err = Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: r#"[
          { "id": "flaw.x", "kind": "flaw", "magnitude": "minor",
            "classification": "narrative",
            "categories": ["general"],
            "index_categories": ["hermetic", "hermetic"] },
          { "id": "flaw.filler", "kind": "flaw", "magnitude": "minor",
            "classification": "narrative", "categories": ["personality"] }
        ]"#,
        type_profiles: r#"[{ "id": "grog",
            "budget": { "virtue_points": 3, "flaw_points": 3 },
            "creation_phases": [] }]"#,
        ..RulesetSources::default()
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("flaw.x"), "names the item: {err}");
    assert!(err.contains("index_categories"), "names the field: {err}");
    assert!(err.contains("hermetic"), "names the offending slug: {err}");
}

/// Carry-over (a) from B4, checked rather than assumed: a **non-Gifted**
/// character who qualifies for Unbearable to (Beings) through Magical Air
/// (`ArMDE:6895`) must not be detected as Gifted. Both `flaw.magical_air` and
/// `flaw.unbearable_to_beings` are `categories: ["general"]`, so nothing in the
/// pair reaches `gift_categories` — and the new index heading must not change
/// that.
#[test]
fn magical_air_plus_unbearable_to_beings_is_not_gifted() {
    let rs = load_ruleset();
    let profile = rs
        .profile(&Id::new("companion"))
        .expect("the companion profile must ship");
    let e = entity(
        "companion",
        vec![
            Selection::new(Id::new("flaw.magical_air")),
            Selection::with_params(
                Id::new("flaw.unbearable_to_beings"),
                BTreeMap::from([("being".to_string(), Id::new("being.demons"))]),
            ),
        ],
    );

    assert_eq!(
        arm_rules::supernatural_free_slots(&e, &rs, profile).total,
        0,
        "Magical Air is not The Gift (ArMDE:6895 names them as alternatives), so \
         neither Flaw may confer the Gift's free Supernatural-Ability slot"
    );
    let codes = issue_codes(&e, &rs);
    assert!(
        !codes.contains(&"prereq_not_met".to_string()),
        "and Magical Air satisfies Unbearable's own gate: {codes:?}"
    );
}

/// `items_by_category` is a membership query, so a two-category item is listed
/// under both — Sufi ("*Minor, Social Status, Supernatural*") is a Supernatural
/// Virtue as well as a Social Status one.
#[test]
fn items_by_category_finds_an_item_through_its_secondary_category() {
    let rs = load_ruleset();
    let sufi = Id::new("virtue.sufi");
    assert!(
        rs.items_by_category("social_status").any(|i| i.id == sufi),
        "Sufi is listed under its primary category"
    );
    assert!(
        rs.items_by_category("supernatural").any(|i| i.id == sufi),
        "Sufi is listed under its secondary category too"
    );
}

// --- Primogeniture Lineage is for magi of House Verditius only (row 13) -----
//
// > "This Flaw can only be taken by magi of House Verditius, as a maga who has
// > left the House is no longer a candidate for Primus. In her case, it would be
// > no more than an interesting feature of her background."
// > — ArMDE:6636
//
// The Flaw ships `categories: ["story"]`, which the companion, mythic-companion
// and magus profiles all permit, so the restriction was enforced by nothing:
// only the grog was refused, and merely because `story` is not on its permitted
// list. This is a House matter, not a category one — adding `hermetic` as a
// second category would corrupt Gift detection, since `gift_categories` is
// `["hermetic"]` on every shipped profile (open-to-dos row 18) — so it is
// modelled as a prerequisite.
//
// `Prereq::House` ALONE would not do it. That leaf is tri-state: an absent house
// evaluates to `Unknown`, which is the non-blocking `prereq_unevaluated`
// warning, and nothing stops a non-magus entity from carrying a `house` value
// (`validate_house` returns early for a profile whose `order_member` is false, so a
// hand-edited save could set one). A companion would therefore have been merely
// warned, or — with a house in the file — waved through. `All([OrderMember,
// House(house.verditius)])` makes every non-Order-member a definite error while
// leaving a magus who has not reached the House step yet on the warning, exactly
// as the four Outer-Mystery Virtues behave.
//
// **`OrderMember`, not `HermeticallyTrained` (D56/A0 sub-slice 4).** `ArMDE:6636`
// itself draws the line at the House, not at training: "This Flaw can only be
// taken by magi of House Verditius, as a maga **who has left the House** is no
// longer a candidate for Primus" — she has not lost her training by leaving, she
// has lost her House membership. House membership is structurally an Order
// concern (`validate_house` gates on `order_member`), so the disqualifying fact
// this prerequisite must check is Order membership, exactly as
// `docs/vf-audit/design-a0-is-magus-split.md` § 2 already argues.

/// The issue codes raised against `flaw.primogeniture_lineage` itself.
fn primogeniture_codes(rs: &Ruleset, e: &Entity) -> Vec<String> {
    let id = Id::new("flaw.primogeniture_lineage");
    validate(e, rs)
        .issues
        .into_iter()
        .filter(|i| i.context.as_ref() == Some(&id))
        .map(|i| i.code)
        .collect()
}

/// The data itself, pinned so a later sweep cannot quietly drop the
/// `OrderMember` conjunct and silently demote the companion case back to a
/// warning. Migrated from `Prereq::IsMagus` in D56/A0 sub-slice 4 — see the
/// comment above for why `OrderMember`, not `HermeticallyTrained`, is the
/// right encoding.
#[test]
fn primogeniture_lineage_requires_an_order_member_of_house_verditius() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.primogeniture_lineage"))
        .expect("flaw.primogeniture_lineage must ship in the catalogue");
    assert_eq!(
        item.prerequisites.as_ref(),
        Some(&Prereq::All(vec![
            Prereq::OrderMember,
            Prereq::House(Id::new("house.verditius")),
        ])),
        "`ArMDE:6636` restricts the Flaw to magi of House Verditius, and the House \
         leaf alone leaves a non-Order-member merely warned"
    );
}

/// The character the Flaw is written for.
#[test]
fn a_verditius_magus_may_take_primogeniture_lineage() {
    let rs = load_ruleset();
    let mut e = entity(
        "magus",
        vec![Selection::new(Id::new("flaw.primogeniture_lineage"))],
    );
    e.house = Some(Id::new("house.verditius"));

    let codes = primogeniture_codes(&rs, &e);
    assert!(
        codes.is_empty(),
        "a Verditius magus is exactly who may take it: {codes:?}"
    );
}

/// The finding: any other House is now an error naming the item, where before
/// nothing at all was raised.
#[test]
fn a_magus_of_another_house_may_not_take_primogeniture_lineage() {
    let rs = load_ruleset();
    let mut e = entity(
        "magus",
        vec![Selection::new(Id::new("flaw.primogeniture_lineage"))],
    );
    e.house = Some(Id::new("house.flambeau"));

    let codes = primogeniture_codes(&rs, &e);
    assert!(
        codes.contains(&"prereq_not_met".to_string()),
        "a Flambeau is not in line for Primus of Verditius: {codes:?}"
    );
}

/// The half a bare `Prereq::House` could not deliver: a companion has no House
/// at all, so the House leaf is `Unknown` and only `OrderMember` can turn the
/// verdict into an error.
#[test]
fn a_companion_may_not_take_primogeniture_lineage() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.primogeniture_lineage"))],
    );
    assert!(e.house.is_none(), "the fixture must set no House");

    let codes = primogeniture_codes(&rs, &e);
    assert!(
        codes.contains(&"prereq_not_met".to_string()),
        "`ArMDE:6636` says magi, and `story` is on a companion's permitted list, so \
         the prerequisite is the only thing that can refuse this: {codes:?}"
    );
}

/// And the same for a mythic companion, whose profile also permits `story`.
#[test]
fn a_mythic_companion_may_not_take_primogeniture_lineage() {
    let rs = load_ruleset();
    let e = entity(
        "mythic_companion",
        vec![Selection::new(Id::new("flaw.primogeniture_lineage"))],
    );

    let codes = primogeniture_codes(&rs, &e);
    assert!(
        codes.contains(&"prereq_not_met".to_string()),
        "a mythic companion is not a magus either: {codes:?}"
    );
}

/// Even a *house-carrying* non-magus is refused. Nothing in the engine forbids
/// the value, so this pins the conjunct against the one case a bare House leaf
/// would have waved straight through.
#[test]
fn a_companion_carrying_a_house_value_is_still_refused_primogeniture_lineage() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.primogeniture_lineage"))],
    );
    e.house = Some(Id::new("house.verditius"));

    let codes = primogeniture_codes(&rs, &e);
    assert!(
        codes.contains(&"prereq_not_met".to_string()),
        "magus-hood is a separate question from the house field: {codes:?}"
    );
}

/// A magus who has not reached the House step yet is warned, not blocked — the
/// engine's error-that-resolves model, and the same behaviour the Outer-Mystery
/// Virtues have.
#[test]
fn a_magus_with_no_house_yet_only_warns_on_primogeniture_lineage() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![Selection::new(Id::new("flaw.primogeniture_lineage"))],
    );
    assert!(e.house.is_none(), "the fixture must set no House");

    let codes = primogeniture_codes(&rs, &e);
    assert!(
        codes.contains(&"prereq_unevaluated".to_string()),
        "an unset House leaves the conjunction undecided: {codes:?}"
    );
    assert!(
        !codes.contains(&"prereq_not_met".to_string()),
        "an in-progress build must not be reported as a failed prerequisite: \
         {codes:?}"
    );
}

// --- Vendetta's hedged House restriction is a warning, not an error --------
// (F-533/F-550, D16, Q-115, Q-139)
//
// > "This Flaw is generally restricted to magi of House Verditius, as the
// > custom of vendetta is limited to that House."
// > — ArMDE:6957 (entry ArMDE:6955-6958)
//
// Unlike Primogeniture Lineage's unhedged "can only be taken by", this passage
// HEDGES ("generally restricted"), and D16 maps a hedge to a warning rather
// than an error. `flaw.vendetta` therefore carries the restriction as
// `advisory_prerequisites` — a tree separate from the hard `prerequisites`
// one, so the same `Prereq::House` leaf reports through the new
// `advisory_prereq_not_met` (warning) code rather than `prereq_not_met`
// (error). The Flaw's own magus half (Q-139, unhedged) is deliberately NOT
// encoded here — that is slice X5's job, which this machinery unblocks.

/// The issue codes raised against `flaw.vendetta` itself.
fn vendetta_codes(rs: &Ruleset, e: &Entity) -> Vec<String> {
    let id = Id::new("flaw.vendetta");
    validate(e, rs)
        .issues
        .into_iter()
        .filter(|i| i.context.as_ref() == Some(&id))
        .map(|i| i.code)
        .collect()
}

/// The data itself, pinned so a later sweep cannot quietly drop the hedge or
/// promote it back to a hard block.
#[test]
fn vendetta_ships_an_advisory_house_verditius_restriction() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.vendetta"))
        .expect("flaw.vendetta must ship in the catalogue");
    assert_eq!(
        item.advisory_prerequisites.as_ref(),
        Some(&Prereq::House(Id::new("house.verditius"))),
        "`ArMDE:6957` hedges (\"generally restricted\"), so D16 maps it to an \
         advisory, never a hard `prerequisites` entry"
    );
}

/// The character the Flaw is written for: no warning at all.
#[test]
fn a_verditius_magus_holding_vendetta_gets_no_advisory_warning() {
    let rs = load_ruleset();
    let mut e = entity("magus", vec![Selection::new(Id::new("flaw.vendetta"))]);
    e.house = Some(Id::new("house.verditius"));

    let codes = vendetta_codes(&rs, &e);
    assert!(
        codes.is_empty(),
        "a Verditius magus is exactly who this Flaw is written for: {codes:?}"
    );
}

/// The finding: a magus of another House now gets a WARNING naming the item —
/// never the hard `prereq_not_met` error, because the passage hedges.
#[test]
fn a_magus_of_another_house_holding_vendetta_gets_an_advisory_warning_not_an_error() {
    let rs = load_ruleset();
    let mut e = entity("magus", vec![Selection::new(Id::new("flaw.vendetta"))]);
    e.house = Some(Id::new("house.flambeau"));

    let codes = vendetta_codes(&rs, &e);
    assert!(
        codes.contains(&"advisory_prereq_not_met".to_string()),
        "the hedge is a warning: {codes:?}"
    );
    assert!(
        !codes.contains(&"prereq_not_met".to_string()),
        "a hedged restriction must never surface as the hard error code: {codes:?}"
    );
}

/// A companion has no House at all: the leaf is `Unknown`, and D16's hedge is
/// only worth flagging on a DEFINITE mismatch — an unresolved one stays
/// silent, unlike the hard tree's `prereq_unevaluated`.
#[test]
fn a_companion_holding_vendetta_with_no_house_gets_no_advisory_warning() {
    let rs = load_ruleset();
    let e = entity("companion", vec![Selection::new(Id::new("flaw.vendetta"))]);
    assert!(e.house.is_none(), "the fixture must set no House");

    let codes = vendetta_codes(&rs, &e);
    assert!(
        !codes.contains(&"advisory_prereq_not_met".to_string()),
        "an unresolved hedge must not warn: {codes:?}"
    );
}

// --- A grog's Supernatural restriction had no source, and is gone (row 20) ---
//
// The grog profile forbade `supernatural` and left it off `permitted_categories`
// as well. Neither half has a source. The grog guidelines are a list of
// restrictions and Supernatural is not among them:
//
// > - You may take up to 3 points of Flaws, and an equal number of points of
// >   Virtues
// > - You must take one Social Status
// > - You should not take Story Flaws
// > - You should not take more than one Personality Flaw
// > - You may not take Major Virtues or Flaws
// > - You may not take Hermetic Virtues and Flaws
// > - You may not take The Gift
// > — ArMDE:2824-2830
//
// and the `### Supernatural` prose (`ArMDE:2958-2962`) explains realm association and
// Warping immunity, setting no character-type restriction at all.
//
// Removing only the forbid would have changed nothing, because permitting is an
// ANY test: a single-category Supernatural Virtue would still have been refused
// with `category_not_permitted`. Both halves went. `hermetic` stays forbidden —
// `ArMDE:2829` sources it explicitly.

/// Category-gate issue codes raised against one item.
fn category_gate_codes(rs: &Ruleset, e: &Entity, item: &Id) -> Vec<String> {
    validate(e, rs)
        .issues
        .into_iter()
        .filter(|i| {
            i.context.as_ref() == Some(item)
                && (i.code == "forbidden_category" || i.code == "category_not_permitted")
        })
        .map(|i| i.code)
        .collect()
}

/// The grog profile's category lists carry only what the book states.
#[test]
fn the_grog_profile_restricts_only_the_categories_the_book_names() {
    let rs = load_ruleset();
    let grog = rs
        .profile(&Id::new("grog"))
        .expect("the grog profile must ship");
    assert_eq!(
        grog.forbidden_categories
            .iter()
            .map(CategoryRule::category)
            .collect::<Vec<_>>(),
        vec!["hermetic"],
        "`ArMDE:2829` is the only category restriction the grog guidelines state"
    );
    assert!(
        grog.permitted_categories
            .iter()
            .all(|rule| rule.when().is_none()),
        "a grog's restrictions are unconditional: `ArMDE:2822-2830` states no \
         'unless' the way `ArMDE:2840` does for a companion"
    );
    assert!(
        grog.names_permitted_category("supernatural"),
        "and permitting is ANY, so the slug must be on the permitted list too or \
         a single-category Supernatural item stays blocked"
    );
}

/// Second Sight is "*Minor, Supernatural*"
/// (ArMDE:4889, entry :4888-4890) and
/// carries that one category, so nothing else can rescue it: it is open to a
/// grog only because the profile no longer bars Supernatural.
#[test]
fn a_grog_may_take_a_minor_supernatural_virtue() {
    let rs = load_ruleset();
    let second_sight = Id::new("virtue.second_sight");
    let item = rs
        .item(&second_sight)
        .expect("virtue.second_sight must ship in the catalogue");
    assert_eq!(item.magnitude, Magnitude::Minor);
    assert_eq!(
        item.categories,
        vec!["supernatural".to_string()],
        "the fixture must be single-category, or it proves nothing"
    );

    let codes = category_gate_codes(
        &rs,
        &entity("grog", vec![Selection::new(second_sight.clone())]),
        &second_sight,
    );
    assert!(
        codes.is_empty(),
        "no sourced restriction bars a grog from a Minor Supernatural Virtue: \
         {codes:?}"
    );
}

/// The restriction that does the real work, and this one *is* sourced: `ArMDE:2828`
/// "You may not take Major Virtues or Flaws". Bee King is "*Major,
/// Supernatural*" (`ArMDE:3485`, entry `ArMDE:3484-3499`), so the category gate lets it
/// through and the magnitude cap refuses it — the honest issue code.
#[test]
fn a_grog_still_may_not_take_a_major_supernatural_virtue() {
    let rs = load_ruleset();
    let bee_king = Id::new("virtue.bee_king");
    let item = rs
        .item(&bee_king)
        .expect("virtue.bee_king must ship in the catalogue");
    assert_eq!(item.magnitude, Magnitude::Major);
    assert!(item.has_category("supernatural"));

    let e = entity("grog", vec![Selection::new(bee_king.clone())]);
    assert!(
        category_gate_codes(&rs, &e, &bee_king).is_empty(),
        "the category is no longer the thing that blocks it"
    );
    let codes: Vec<String> = validate(&e, &rs)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect();
    assert!(
        codes.contains(&"too_many_major_virtues".to_string()),
        "`ArMDE:2828` still bars every Major Virtue, Supernatural included: {codes:?}"
    );
}

/// `ArMDE:2830` "You may not take The Gift" is untouched: Gift-hood is detected
/// through the profile's `gift_categories` (`["hermetic"]`), so a Supernatural
/// Virtue never confers it. The free Supernatural-Ability slot is the observable
/// consequence of being Gifted, and a grog gets none.
#[test]
fn a_grog_with_a_supernatural_virtue_is_not_thereby_gifted() {
    let rs = load_ruleset();
    let profile = rs
        .profile(&Id::new("grog"))
        .expect("the grog profile must ship");
    let e = entity("grog", vec![Selection::new(Id::new("virtue.second_sight"))]);

    let codes: Vec<String> = validate(&e, &rs)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect();
    assert!(
        !codes.contains(&"gift_forbidden".to_string()),
        "a Supernatural Virtue is not The Gift: {codes:?}"
    );
    assert_eq!(
        arm_rules::supernatural_free_slots(&e, &rs, profile).total,
        0,
        "the free slot belongs to the Gifted, and `ArMDE:2830` bars a grog from The \
         Gift"
    );
}

/// `ArMDE:2824`'s 3-point budget is untouched too: four Minor Supernatural Virtues
/// now clear the category gate and are refused on points instead.
#[test]
fn the_three_point_grog_budget_still_bounds_supernatural_virtues() {
    let rs = load_ruleset();
    let e = entity(
        "grog",
        vec![
            Selection::new(Id::new("virtue.second_sight")),
            Selection::new(Id::new("virtue.premonitions")),
            Selection::new(Id::new("virtue.dowsing")),
            Selection::new(Id::new("virtue.magic_sensitivity")),
        ],
    );

    let codes: Vec<String> = validate(&e, &rs).errors().map(|i| i.code.clone()).collect();
    assert!(
        codes.contains(&"over_budget_virtues".to_string()),
        "four Minor Virtues exceed `ArMDE:2824`'s 3 points: {codes:?}"
    );
}

// --- Mythic Companion is a category, not a marker ---------------------------
//
// `## List of Virtues` groups its entries under `### <Category>, <Magnitude>`
// headings, and one of those headings is `### Mythic Companion, Free`
// (ArMDE:3329-3334), listing Devil
// Child, Faerie Doctor, Nephilim and Spirit Votary — none of which appears under
// `### Social Status, Free` (ArMDE:3336-3354). Their descriptors read
// "*Free, Mythic Companion*" (ArMDE:3672, :3822, :4595, :5007), which is the
// magnitude-then-category shape every other descriptor uses. `Tainted` never
// occupies that slot — it is always a third token after a real category — which
// is why it is a flag and this is not.

/// The four Mythic Companion Virtues carry `mythic_companion` as their category,
/// and none of them is a Social Status Virtue.
#[test]
fn the_mythic_companion_virtues_carry_the_mythic_companion_category() {
    let rs = load_ruleset();
    for id in [
        "virtue.devil_child",
        "virtue.faerie_doctor",
        "virtue.nephilim",
        "virtue.spirit_votary",
    ] {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must ship in the catalogue"));
        assert_eq!(
            item.categories,
            vec!["mythic_companion".to_string()],
            "{id} is indexed under `### Mythic Companion, Free` alone"
        );
        assert!(
            !item.has_category("social_status"),
            "{id} is not listed under `### Social Status, Free`"
        );
    }
}

/// "All Mythic Companions take a Free Virtue which specifies their status. These
/// Virtues are incompatible with each other, and with The Gift, and are not
/// available to grogs." (`ArMDE:2637`) — and each descriptor says the Virtue *makes*
/// the character a Mythic Companion (`ArMDE:3673`, `ArMDE:3823`, `ArMDE:4596`, `ArMDE:5008`). So the
/// category is permitted to the mythic-companion profile and to no other.
#[test]
fn only_the_mythic_companion_profile_permits_the_mythic_companion_category() {
    let rs = load_ruleset();
    let mythic = Id::new("mythic_companion");
    let mut checked = 0;
    for profile in rs.profiles() {
        let permitted = profile.names_permitted_category("mythic_companion");
        assert_eq!(
            permitted,
            profile.id == mythic,
            "{} must {} the mythic_companion category",
            profile.id,
            if profile.id == mythic {
                "permit"
            } else {
                "not permit"
            }
        );
        checked += 1;
    }
    assert!(checked > 1, "the shipped ruleset must declare profiles");
}

/// The consequence on a real character: a grog may not take a Mythic Companion
/// Virtue (`ArMDE:2637`), which the `social_status` mapping used to allow outright.
#[test]
fn a_grog_may_not_take_a_mythic_companion_virtue() {
    let rs = load_ruleset();
    let devil_child = Id::new("virtue.devil_child");
    let result = validate(
        &entity("grog", vec![Selection::new(devil_child.clone())]),
        &rs,
    );
    let issue = result
        .issues
        .iter()
        .find(|i| i.code == "category_not_permitted" && i.context.as_ref() == Some(&devil_child))
        .expect("a grog's permitted categories exclude mythic_companion");
    assert_eq!(
        issue.args.get("category").map(String::as_str),
        Some("mythic_companion")
    );
}

/// The Gift-category test is a membership query as well, so it still recognises
/// a Flaw through the `hermetic` category it now shares with `story`: a grog
/// forbids The Gift, and Suppressed Gift is Hermetic.
#[test]
fn the_gift_category_check_still_fires_for_a_two_category_flaw() {
    let rs = load_ruleset();
    let grog = rs
        .profile(&Id::new("grog"))
        .expect("the grog profile must ship");
    assert!(
        grog.gift_categories.contains("hermetic"),
        "the grog profile detects The Gift by the hermetic category"
    );

    let result = validate(
        &entity(
            "grog",
            vec![Selection::new(Id::new("flaw.suppressed_gift"))],
        ),
        &rs,
    );
    assert!(
        result.issues.iter().any(|i| i.code == "gift_forbidden"),
        "a Hermetic Flaw must still count as having The Gift for a grog"
    );
}

// --- Repeatable Virtues and Flaws (GitHub issue 3) ---
//
// "A Virtue or Flaw may be taken more than once only if the description
// explicitly allows it. Most Virtues and Flaws may only be taken once."
// ArMDE:2814.
//
// The engine keys duplicate selections on `(item_ref, params)` and permits
// `max_per_target` copies of each key. An item whose descriptor allows repeats
// but that carries no target parameter therefore needs its ceiling raised in the
// data, or the app blocks a legal build.

/// Every core-rules item whose descriptor allows repetition **without naming a
/// ceiling**, paired with the line that says so. The convention for "the
/// rulebook states no limit" is `u8::MAX`: the V/F point budget
/// (ArMDE:2638) caps the real count
/// far below it, so the number is unreachable rather than arbitrary.
///
/// `flaw.false_power_minor` used to sit here, because its ceiling — one copy
/// "for each appropriate Supernatural Virtue that the character possesses"
/// (`ArMDE:6096`) — was a per-Virtue target the data model could not express. It now
/// can: the entry carries a `require_possessed` item parameter naming the
/// Virtue, so the ceiling is `max_per_target: 1` per named Virtue plus an
/// unbounded `max_total` across different ones. Pinned by
/// `false_power_names_the_supernatural_virtue_it_taints` instead.
///
/// `flaw.flawed_parma_magica` and `flaw.limited_magic_resistance` left for the
/// same reason and by the same route. Each repeats "for different Forms"
/// (`ArMDE:6144`, `ArMDE:6348`) — never twice for one Form — and each now carries
/// the `form` parameter that makes the Form part of the duplicate key, so the
/// **default** `max_per_target` of 1 states the real ceiling and the 255 they
/// shipped with was over-permissive. Pinned by
/// `the_form_scoped_magic_resistance_flaws_name_their_form` and
/// `a_form_scoped_mr_flaw_repeats_across_forms_but_never_within_one` instead.
///
/// **D10's §3.7 sweep (Q4) removed four more**: `virtue.greater_immunity`
/// ("with a **different immunity each time**", `ArMDE:4015`),
/// `flaw.deteriorating_power` ("if the character has **more than one**
/// Power" — implicitly a different one per copy, the same shape as
/// `flaw.slow_power`/`flaw.restricted_power`/`virtue.variable_power`, which
/// already carry a `power` parameter for exactly this), `virtue.social_contacts`
/// ("**each time specifying a different social group**", `ArMDE:4990`), and
/// `flaw.vulnerable_magic` ("so long as **a different condition** is specified
/// for each", `ArMDE:7009`, F-541's defect). All four are **vary-the-target**,
/// not level-stack: an identical second copy is not what the passage grants.
/// Q4 left them with no target parameter and no `max_per_target`, so each
/// defaulted to the plain once-only ceiling as a safe interim.
///
/// **Slice Q4b (D9 part 1) gave each one, applied early rather than waiting
/// for the catalogue-wide X6 sweep.** `flaw.deteriorating_power` is the exact
/// shape `flaw.slow_power`/`flaw.restricted_power`/`virtue.variable_power`
/// already use — a free-text `power` parameter checked against the character's
/// own `entity.powers` — so it now sits in [`PER_POWER_ITEMS`] beside them. The
/// other three name nothing else the sheet already tracks, so each got its own
/// plain free-text parameter instead: see [`TEXT_TARGET_PARAM_ITEMS`] below.
const UNLIMITED_REPEAT_ITEMS: &[(&str, u32)] = &[
    ("virtue.demonic_might", 3665),
    ("virtue.demonic_powers", 3669),
    ("virtue.focus_power", 3903),
    ("virtue.greater_power", 4021),
    ("virtue.improved_characteristics", 4105),
    ("virtue.lesser_power", 4283),
    ("virtue.magic_items", 4349),
    ("virtue.mastered_spells", 4474),
    ("virtue.mentored_by_demons", 4498),
    ("virtue.minor_enchantments", 4534),
    ("virtue.personal_power", 4724),
    ("virtue.ritual_power", 4874),
    ("virtue.special_circumstances", 5000),
    ("virtue.strong_angelic_heritage", 5030),
    ("virtue.withstand_casting", 5265),
    ("flaw.vulnerable_casting", 6997),
];

/// **Slice Q4b (D9 part 1)**: the three vary-the-target items from
/// [`UNLIMITED_REPEAT_ITEMS`]'s doc comment that are NOT an instance of
/// anything else the sheet already tracks (unlike a power, which resolves
/// against `entity.powers` and so joined [`PER_POWER_ITEMS`] instead) —
/// `(id, target parameter key, the line that grants the repeat)`. Each needs
/// only a plain free-text parameter: the duplicate key becomes
/// `(item_ref, {key: value})`, so two copies naming the same value collide
/// under `max_per_target`'s default of 1, and two copies naming different
/// values are both legal under the explicit `max_total: 255` ("no ceiling the
/// rules state").
const TEXT_TARGET_PARAM_ITEMS: &[(&str, &str, u32)] = &[
    // "so long as a different condition is specified for each" (ArMDE:7009).
    ("flaw.vulnerable_magic", "condition", 7009),
    // "with a different immunity each time" (ArMDE:4015).
    ("virtue.greater_immunity", "hazard", 4015),
    // "each time specifying a different social group" (ArMDE:4990).
    ("virtue.social_contacts", "social_group", 4990),
];

/// Items whose descriptor states a ceiling of exactly two copies, paired with
/// the line that says so.
const TWICE_ONLY_REPEAT_ITEMS: &[(&str, u32)] = &[
    ("virtue.great_characteristic", 3989),
    ("virtue.quiet_magic", 4826),
    ("flaw.poor_characteristic", 6600),
    ("flaw.weak_characteristics", 7058),
];

/// Items whose descriptor **forbids** repetition — the controls for the sweep.
const ONCE_ONLY_ITEMS: &[(&str, u32)] = &[
    ("virtue.inoffensive_to_beings", 4139),
    ("flaw.corrupted_abilities", 5851),
    ("flaw.corrupted_arts", 5857),
    ("flaw.corrupted_spells", 5863),
    ("flaw.fish_out_of_water_terrain", 6132),
    ("flaw.offensive_to_beings", 6530),
    ("flaw.unbearable_to_beings", 6897),
];

/// Items whose descriptor caps the TOTAL number of copies across every
/// distinct parameter target — `(id, line, expected max_total)`. Distinct
/// from `max_per_target`, which caps copies sharing one identical target.
///
/// Four of these — `virtue.inoffensive_to_beings`, `flaw.offensive_to_beings`,
/// `flaw.unbearable_to_beings`, `flaw.fish_out_of_water_terrain` — also
/// appear in `ONCE_ONLY_ITEMS` above. That is deliberate, not a leftover to
/// tidy away: each carries a free-text target parameter (`being`/`terrain`)
/// so two selections could otherwise carry two different target strings and
/// slip past `max_per_target` entirely, even though the rulebook flatly
/// forbids taking the item more than once, period. `max_per_target == 1`
/// blocks a second copy at the SAME target; `max_total == 1` is what actually
/// blocks a second copy at a DIFFERENT one. Both fields are needed and they
/// say different things, so both tables list the item.
const TOTAL_CAP_ITEMS: &[(&str, u32, u8)] = &[
    ("virtue.inoffensive_to_beings", 4139, 1),
    ("flaw.offensive_to_beings", 6530, 1),
    ("flaw.unbearable_to_beings", 6897, 1),
    ("flaw.fish_out_of_water_terrain", 6132, 1),
    // False Power's FIRST instance is the Major one and there is only ever one
    // of it — "in each subsequent instance as a Minor Flaw rather than a Major
    // one" (`ArMDE:6096`). Every subsequent copy is the separate
    // `flaw.false_power_minor` entry, so the Major is capped at one copy total.
    ("flaw.false_power", 6096, 1),
    ("virtue.affinity_art", 3378, 2),
    ("virtue.puissant_art", 4820, 2),
];

/// Items whose descriptor caps the share of their own kind's point total that
/// their copies may account for — `(id, line, numerator, denominator)`. Both
/// Demonic entries say a repeat "can account for no more than half of the
/// character's total Virtues", each about *this* Virtue, so the two ceilings
/// are independent rather than a shared pool.
const SHARE_CAPPED_ITEMS: &[(&str, u32, u8, u8)] = &[
    ("virtue.demonic_might", 3665, 1, 2),
    ("virtue.demonic_powers", 3669, 1, 2),
];

/// Items whose descriptor allows one copy **per supernatural power the
/// character possesses**, paired with the line that says so.
///
/// These sit between `UNLIMITED_REPEAT_ITEMS` and `ONCE_ONLY_ITEMS` and belong
/// to neither. The rulebook allows the repeat, but only across *different*
/// powers — "not more than once for a single power" (`flaw.slow_power`,
/// `ArMDE:6761`). That shape is expressed entirely in data: a free-text `power`
/// parameter makes each copy's target part of the `(item_ref, params)`
/// duplicate key, `max_per_target` stays at its default of 1 so a second copy
/// naming the SAME power collides, and `max_total` stays absent (`u8::MAX`)
/// because the book states no ceiling on the number of powers.
///
/// The target must be free text rather than `domain: "item"`: a "power" is an
/// *instance* of one of the Focus/Greater/Lesser/Personal/Ritual Power Virtues
/// (`ArMDE:6689`), and those Virtues are themselves unparameterized and repeatable,
/// so naming the Virtue would wrongly cap a magus at one copy across all three
/// of his Greater Powers.
const PER_POWER_ITEMS: &[(&str, u32)] = &[
    ("virtue.variable_power", 5205),
    ("flaw.restricted_power", 6689),
    ("flaw.slow_power", 6761),
    // "This Flaw may be taken more than once, if the character has more than
    // one Power" (ArMDE:5948) — added here by slice Q4b (D9 part 1), applied
    // early rather than waiting for X6's catalogue-wide sweep.
    ("flaw.deteriorating_power", 5948),
];

/// The Power Virtues that fund `Entity::powers` — `(id, line, levels granted)`.
///
/// Each is "a supernatural power that he can activate at will" priced in *levels*
/// of a Formulaic (or, for Ritual Power, a Ritual) Hermetic spell, and each spends
/// those levels on the power's level and its Penetration alike, so all four grant
/// into the one `power_levels_budget` the being's `powers` are charged against.
///
/// **Focus Power is deliberately absent.** Its 25 are a *different currency*: "This
/// Virtue grants a pool of 25 points… It costs 2 points to raise the maximum level
/// of effect by 1, and 1 point to raise the Penetration by 1" (`ArMDE:3899`). Adding 25
/// points to a budget denominated in levels would be wrong arithmetic — a Focus
/// Power's 25 points buy at most 12 levels, not 25. The control test below pins
/// that it stays out.
const POWER_LEVEL_ITEMS: &[(&str, u32, u16)] = &[
    // "equivalent to a Formulaic Hermetic spell with a level of 50 or lower"
    // (ArMDE:4019).
    ("virtue.greater_power", 4019, 50),
    // "equivalent to Formulaic Hermetic spells with total levels of 25 or lower"
    // (ArMDE:4281).
    ("virtue.lesser_power", 4281, 25),
    // "equivalent to a Formulaic Hermetic spell with a level of 25 or lower"
    // (ArMDE:4716).
    ("virtue.personal_power", 4716, 25),
    // "equivalent to a Ritual Hermetic spell with a level of 25 or lower" (ArMDE:4872).
    ("virtue.ritual_power", 4872, 25),
];

/// Items whose target parameter is a **closed list the rulebook prints in full**
/// — `(id, param key, the line that prints the list, the value ids)`.
///
/// The value ids are asserted here rather than merely counted, because the whole
/// point of the `enumerated` domain is that the *book's* list is the domain: a
/// value the book does not name must not resolve, and one it does name must.
/// The three (Beings) lists are genuinely different subsets of one another, which
/// is why the enumeration is declared per parameter and not once globally.
///
/// **No count is written down anywhere**, and Folk Magic no longer has a
/// ceiling this list could imply. It may be picked "more than once, to acquire
/// expertise in a different category of spells" (`ArMDE:3919`) and carries neither
/// `max_total` nor `max_per_target` — but the same sentence gives it a **second**
/// axis ("you can align it to the same Realm as before or pick a different
/// one"), so a further copy is legal as soon as it differs in *either* axis and
/// two copies may legitimately share a category. The `(item_ref, params)`
/// duplicate key covers both axes at once, so nothing here counts anything:
/// `folk_magic_repeats_along_either_axis_and_never_across_the_excluded_realms`
/// states the whole rule without a number, and a supplement adding a fifth
/// category needs no edit — which is why this was the recorded fix rather than
/// `max_per_target: 4`.
///
/// `flaw.fish_out_of_water_terrain` is deliberately **absent**: its terrain list
/// ends "…, etc." (`ArMDE:6130`), so open-endedness is what the book means there. The
/// control test below pins that it stays free text.
const ENUMERATED_PARAM_ITEMS: &[(&str, &str, u32, &[&str])] = &[
    // "He can only create spells in one narrow area, which must be one of the
    // following four options" (ArMDE:3909), printed :3911-3917.
    (
        "virtue.folk_magic",
        "category",
        3909,
        &[
            "folk_magic.abjuration",
            "folk_magic.divination",
            "folk_magic.evil_eye",
            "folk_magic.healing",
        ],
    ),
    // "associated with one of five classes of beings: animals, divine beings,
    // faeries, demons, or magical creatures" (ArMDE:4135).
    (
        "virtue.inoffensive_to_beings",
        "being",
        4135,
        &[
            "being.animals",
            "being.demons",
            "being.divine",
            "being.faeries",
            "being.magical_creatures",
        ],
    ),
    // "one of six classes of beings: animals, mundane humans, divine beings,
    // faeries, demons, or magical creatures" (ArMDE:6526) — the five above plus
    // mundane humans.
    (
        "flaw.offensive_to_beings",
        "being",
        6526,
        &[
            "being.animals",
            "being.demons",
            "being.divine",
            "being.faeries",
            "being.magical_creatures",
            "being.mundane_humans",
        ],
    ),
    // "one of three classes of beings: mundane humans, demons, or divine
    // beings" (ArMDE:6893) — a strict subset of the other two.
    (
        "flaw.unbearable_to_beings",
        "being",
        6893,
        &["being.demons", "being.divine", "being.mundane_humans"],
    ),
    // "You may take one group of restricted Abilities during character
    // generation, either Martial, Academic, or Arcane Abilities" (ArMDE:3631)
    // — Phase 2 C1's exclusive-choice gate (W2/F-42).
    (
        "virtue.custos",
        "study",
        3631,
        &[
            "ability_category.academic",
            "ability_category.arcane",
            "ability_category.martial",
        ],
    ),
    // "You may take one RESTRICTED group of Abilities during character
    // creation, such as Academic or Martial Abilities" (ArMDE:5135) — C1's
    // closed reading of an open "such as" list, narrowed to the categories
    // `rules/core/abilities.json`'s `categories_requiring_virtue` actually
    // gates (`general` needs no authorization at all, so offering it would be
    // a meaningless choice — RULES.md, F-317).
    (
        "virtue.templar_specialist",
        "study",
        5135,
        &[
            "ability_category.academic",
            "ability_category.arcane",
            "ability_category.martial",
        ],
    ),
    // "You may take either Arcane or Academic Abilities, but not both, at
    // character creation" (ArMDE:5259) — W2/F-349.
    (
        "virtue.wise_one",
        "study",
        5259,
        &["ability_category.academic", "ability_category.arcane"],
    ),
];

#[test]
fn shipped_enumerated_params_declare_exactly_their_book_values() {
    let rs = load_ruleset();

    for (id, key, line, values) in ENUMERATED_PARAM_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        let param = item
            .parameters
            .iter()
            .find(|p| p.key == *key)
            .unwrap_or_else(|| {
                panic!(
                    "{id} must declare a '{key}' parameter, not {:?}",
                    item.parameters
                )
            });
        assert_eq!(
            param.domain,
            ParameterDomain::Enumerated,
            "{id}'s '{key}' is one of a closed list the book prints \
             (ArMDE:{line}), not free text"
        );
        let declared: Vec<String> = param.values.iter().map(|v| v.to_string()).collect();
        assert_eq!(
            declared,
            values.iter().map(|v| v.to_string()).collect::<Vec<_>>(),
            "{id}'s '{key}' must offer exactly the classes named at \
             ArMDE:{line}"
        );
    }
}

#[test]
fn fish_out_of_water_keeps_a_free_text_terrain() {
    // The control for the sweep above: this list ends "…, etc." (ArMDE:6130), so the
    // book means it to be open. Tightening it to `enumerated` would be a wrong
    // rules output, not a UI improvement.
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.fish_out_of_water_terrain"))
        .expect("flaw.fish_out_of_water_terrain must ship");
    let [param] = item.parameters.as_slice() else {
        panic!("expected exactly one parameter, got {:?}", item.parameters);
    };
    assert_eq!(
        param.domain,
        ParameterDomain::Text,
        "the terrain list ends '…, etc.' \
         (ArMDE:6130), so it stays open"
    );
}

/// Every value id any shipped `enumerated` parameter declares, deduplicated and
/// in canonical order.
fn shipped_enumerated_value_ids(rs: &Ruleset) -> Vec<Id> {
    let mut ids: Vec<Id> = rs
        .items()
        .flat_map(|item| item.parameters.iter())
        .filter(|p| p.domain == ParameterDomain::Enumerated)
        .flat_map(|p| p.values.iter().cloned())
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// The value ids sit outside every other coverage check — they are neither point
/// items nor abilities nor spells — so without this test a missing German label
/// would ship in silence and the picker would render the raw slug, which the
/// "no user-facing string is a raw ID" invariant forbids outright.
#[test]
fn every_enumerated_value_id_has_english_and_german_text() {
    let rs = load_ruleset();
    let values = shipped_enumerated_value_ids(&rs);
    assert!(
        !values.is_empty(),
        "the shipped catalogue must declare at least one enumerated parameter"
    );

    for (lang, i18n) in [
        (
            "English",
            include_str!("../../../rules/i18n/en/virtues_flaws.json"),
        ),
        (
            "German",
            include_str!("../../../rules/i18n/de/virtues_flaws.json"),
        ),
    ] {
        let loc = LocalizedRuleset::new(rs.clone(), i18n).unwrap();
        for value in &values {
            assert!(
                loc.display_name(value).is_some(),
                "{lang} i18n missing enumerated parameter value '{value}'"
            );
        }
    }
}

/// Builds a companion holding one copy of `id` per entry of `values`, each
/// naming that value under `key`.
fn entity_with_param_values(id: &str, key: &str, values: &[&str]) -> Entity {
    let selections = values
        .iter()
        .map(|value| {
            Selection::with_params(
                Id::new(id),
                BTreeMap::from([(key.to_string(), Id::new(*value))]),
            )
        })
        .collect();
    entity("companion", selections)
}

#[test]
fn a_value_outside_an_enumerated_list_does_not_resolve() {
    let rs = load_ruleset();

    for (id, key, line, _) in ENUMERATED_PARAM_ITEMS {
        // Exactly the free text these slots used to accept, and the shape an
        // older save still holds.
        let codes = issue_codes(&entity_with_param_values(id, key, &["dragons"]), &rs);
        assert!(
            codes.contains(&"unknown_param_value".to_string()),
            "{id}'s '{key}' takes only the classes the book names \
             (ArMDE:{line}): {codes:?}"
        );
    }
}

#[test]
fn every_declared_enumerated_value_resolves() {
    let rs = load_ruleset();

    for (id, key, line, values) in ENUMERATED_PARAM_ITEMS {
        for value in *values {
            let codes = issue_codes(&entity_with_param_values(id, key, &[value]), &rs);
            assert!(
                !codes.contains(&"unknown_param_value".to_string()),
                "{id}'s declared value '{value}' must resolve \
                 (ArMDE:{line}): {codes:?}"
            );
        }
    }
}

#[test]
fn folk_magic_repeats_across_categories_but_never_within_one() {
    // "You may pick this Virtue more than once, to acquire expertise in a
    // different category of spells." (ArMDE:3919)
    let rs = load_ruleset();
    let (id, key, _, values) = ENUMERATED_PARAM_ITEMS[0];
    assert_eq!(id, "virtue.folk_magic");

    let different = issue_codes(&entity_with_param_values(id, key, &values[..2]), &rs);
    assert!(
        !different.contains(&"duplicate_selection".to_string()),
        "two copies in different categories are what :3919 permits: {different:?}"
    );

    let same = issue_codes(
        &entity_with_param_values(id, key, &[values[0], values[0]]),
        &rs,
    );
    assert!(
        same.contains(&"duplicate_selection".to_string()),
        "a second copy in the SAME category is a repeat, not an expertise: {same:?}"
    );
}

/// The `realm` parameter Folk Magic carries alongside its spell category, and
/// the one exclusion the rulebook states over it.
///
/// > "The choice of (Realm) Lore also determines which supernatural realm his
/// > magic is aligned to for the purposes of aura modifiers." (`ArMDE:3909`)
///
/// > "Each time you choose this Virtue, you can align it to the same Realm as
/// > before or pick a different one, although a character cannot have access to
/// > both the Divine and Infernal Realms." (`ArMDE:3919`)
///
/// What is stored is the **Realm**, not the (Realm) Lore Ability: the Core
/// Rules print no closed "(Realm) Lore" list, while the four Realms are a
/// closed taxonomy the engine already models and already labels. The exclusion
/// is `at_most_one_of` **data**, so no realm id is named in engine code — the
/// behavioural test below reads the pair out of the catalogue rather than
/// spelling it out either.
fn folk_magic_realm_param(rs: &Ruleset) -> &ParameterDef {
    rs.item(&Id::new("virtue.folk_magic"))
        .expect("virtue.folk_magic must ship")
        .parameters
        .iter()
        .find(|p| p.key == "realm")
        .expect("virtue.folk_magic must declare a 'realm' parameter")
}

#[test]
fn folk_magic_records_the_realm_its_magic_is_aligned_to() {
    let rs = load_ruleset();
    let param = folk_magic_realm_param(&rs);

    assert_eq!(
        param.domain,
        ParameterDomain::Realm,
        "the realm is one of the four the engine models, not free text \
         (ArMDE:3909)"
    );
    assert!(
        param.values.is_empty(),
        "the Realm taxonomy IS the registry; a declared list would be read by \
         nothing: {:?}",
        param.values
    );
    assert_eq!(
        param.at_most_one_of,
        vec![BTreeSet::from([
            Id::new("realm.divine"),
            Id::new("realm.infernal")
        ])],
        "\"a character cannot have access to both the Divine and Infernal \
         Realms\" (ArMDE:3919)"
    );
}

/// Builds a companion holding one copy of Folk Magic per `(category, realm)`
/// pair — the two axes `ArMDE:3909` gives it.
fn folk_magic_copies(pairs: &[(&Id, &Id)]) -> Entity {
    let selections = pairs
        .iter()
        .map(|(category, realm)| {
            Selection::with_params(
                Id::new("virtue.folk_magic"),
                BTreeMap::from([
                    ("category".to_string(), (*category).clone()),
                    ("realm".to_string(), (*realm).clone()),
                ]),
            )
        })
        .collect();
    entity("companion", selections)
}

/// Folk Magic now repeats along **two** axes, and the rulebook says how each
/// behaves.
///
/// > "You may pick this Virtue more than once, to acquire expertise in a
/// > different category of spells. Each time you choose this Virtue, you can
/// > align it to the same Realm as before or pick a different one, although a
/// > character cannot have access to both the Divine and Infernal Realms."
/// > (`ArMDE:3919`)
///
/// So a further copy is legal as soon as it differs in **either** axis; it is a
/// repeat only when it differs in neither; and the one pairing the book rules
/// out is the excluded realm group. **No count appears anywhere in this test.**
/// Both the spell categories and the excluded realms are read out of the
/// catalogue, so a supplement that adds a category — or a second realm-axis
/// Virtue with its own exclusion — needs no edit here. This replaces the old
/// `folk_magics_ceiling_is_the_length_of_its_own_list`, whose "one copy per
/// category is the ceiling" premise the realm axis retired: two copies may now
/// legitimately share a category.
#[test]
fn folk_magic_repeats_along_either_axis_and_never_across_the_excluded_realms() {
    let rs = load_ruleset();
    let (id, key, _, categories) = ENUMERATED_PARAM_ITEMS[0];
    assert_eq!((id, key), ("virtue.folk_magic", "category"));
    let spell_categories: Vec<Id> = categories.iter().map(|v| Id::new(*v)).collect();
    let group = folk_magic_realm_param(&rs)
        .at_most_one_of
        .first()
        .expect("Folk Magic's realm axis carries :3919's exclusion")
        .clone();
    let excluded: Vec<Id> = group.iter().cloned().collect();
    let free: Vec<Id> = Realm::ALL
        .iter()
        .map(|realm| realm.id())
        .filter(|realm| !group.contains(realm))
        .collect();
    // Preconditions, not catalogue totals: the cases below need two of each to
    // be expressible at all.
    assert!(spell_categories.len() >= 2 && excluded.len() >= 2 && free.len() >= 2);

    let clean = |what: &str, pairs: &[(&Id, &Id)]| {
        let codes = issue_codes(&folk_magic_copies(pairs), &rs);
        for code in [
            "duplicate_selection",
            "exclusive_param_values",
            "unknown_param_value",
            "missing_param",
        ] {
            assert!(
                !codes.contains(&code.to_string()),
                "{what} is what :3919 permits: {codes:?}"
            );
        }
    };

    clean(
        "a second copy in a different category",
        &[
            (&spell_categories[0], &free[0]),
            (&spell_categories[1], &free[0]),
        ],
    );
    clean(
        "a second copy aligned to a different Realm",
        &[
            (&spell_categories[0], &free[0]),
            (&spell_categories[0], &free[1]),
        ],
    );

    let same = issue_codes(
        &folk_magic_copies(&[
            (&spell_categories[0], &free[0]),
            (&spell_categories[0], &free[0]),
        ]),
        &rs,
    );
    assert!(
        same.contains(&"duplicate_selection".to_string()),
        "a copy differing in NEITHER axis is a repeat, not an expertise: {same:?}"
    );

    let both = issue_codes(
        &folk_magic_copies(&[
            (&spell_categories[0], &excluded[0]),
            (&spell_categories[1], &excluded[1]),
        ]),
        &rs,
    );
    assert!(
        both.contains(&"exclusive_param_values".to_string()),
        "a character cannot have access to both the Divine and Infernal Realms \
         (ArMDE:3919): {both:?}"
    );
}

/// E5 (open-todos row 29): Necessary (Realm) Aura for (Ability) is capped on
/// its **Ability** key alone, not on the whole `(Realm, Ability)` tuple.
///
/// > "A character may take this Flaw once for any particular Ability."
/// > (`ArMDE:6482`)
///
/// The Realm axis is deliberately untouched by that sentence — nothing in
/// `ArMDE:6480-6487` says a character may hold the Flaw only once overall — so
/// the cap is expressed on the parameter that the sentence names and on no
/// other. The two Abilities and the two Realms are read out of the catalogue
/// and the `Realm` taxonomy, so no ability id is written down here.
#[test]
fn necessary_aura_is_taken_once_for_any_particular_ability() {
    let rs = load_ruleset();
    let id = Id::new("flaw.necessary_realm_aura_for_ability");
    let item = rs.item(&id).expect("the Flaw must ship");
    let realms: Vec<Id> = Realm::ALL.iter().map(|realm| realm.id()).collect();
    let abilities: Vec<Id> = rs.abilities().map(|a| a.id.clone()).take(2).collect();
    // Preconditions, not catalogue totals: the cases below need two of each.
    assert!(realms.len() >= 2 && abilities.len() == 2);

    let ability_param = item
        .parameters
        .iter()
        .find(|p| p.key == "ability")
        .expect("the Flaw must declare an 'ability' parameter");
    assert_eq!(
        ability_param.max_per_value, 1,
        "'once for any particular Ability' (ArMDE:6482) is a cap of one on the \
         Ability key"
    );
    let realm_param = item
        .parameters
        .iter()
        .find(|p| p.key == "realm")
        .expect("the Flaw must declare a 'realm' parameter");
    assert_eq!(
        realm_param.max_per_value,
        u8::MAX,
        "no sentence in ArMDE:6480-6487 caps the Realm axis, so it stays free"
    );

    let copies = |pairs: &[(&Id, &Id)]| {
        entity(
            "companion",
            pairs
                .iter()
                .map(|(ability, realm)| {
                    Selection::with_params(
                        id.clone(),
                        BTreeMap::from([
                            ("ability".to_string(), (*ability).clone()),
                            ("realm".to_string(), (*realm).clone()),
                        ]),
                    )
                })
                .collect(),
        )
    };

    let same_ability = issue_codes(
        &copies(&[(&abilities[0], &realms[0]), (&abilities[0], &realms[1])]),
        &rs,
    );
    assert!(
        same_ability.contains(&"too_many_for_param_value".to_string()),
        "a second copy for the same Ability is what :6482 forbids, whatever the \
         Realm: {same_ability:?}"
    );

    let different_abilities = issue_codes(
        &copies(&[(&abilities[0], &realms[0]), (&abilities[1], &realms[0])]),
        &rs,
    );
    assert!(
        !different_abilities.contains(&"too_many_for_param_value".to_string()),
        "two different Abilities are two particular Abilities, which :6482 \
         permits: {different_abilities:?}"
    );
}

/// E2 (open-todos row 24): **every** shipped parameter that asks for a
/// supernatural Realm names one of the four the engine models — none is free
/// text any more.
///
/// Four items shipped their realm as `domain: "text"` after B7 gave the project
/// `ParameterDomain::Realm`, and each one's own rulebook entry states the closed
/// list in so many words:
///
/// > "Choose the realm (Divine, Faerie, Infernal, or Magic) to which the
/// > character is bound when you take the Flaw." — Bound to (Realm),
/// > `ArMDE:5733`
///
/// > "Due to some connection with a given supernatural realm …" — Necessary
/// > (Realm) Aura for (Ability), `ArMDE:6482`
///
/// > "Pick one of the four Realms of Power" — (Realm) Stigmatic, `ArMDE:6656`
///
/// > "You have been trained in the mystical aspects of one of the four realms of
/// > power (Divine, Faerie, Infernal, or Magic)" — Student of (Realm),
/// > `ArMDE:5054`
///
/// Written as a **sweep over the catalogue**, not a list of four ids: catalogue
/// size is data, so a supplement adding a fifth realm-axis item is held to the
/// same rule with no edit here. The four ids appear only as a non-vacuity
/// precondition, so the sweep cannot pass by finding nothing to check.
#[test]
fn no_shipped_realm_parameter_is_free_text() {
    let rs = load_ruleset();

    let realm_params: Vec<(&Id, &ParameterDef)> = rs
        .items()
        .flat_map(|item| {
            item.parameters
                .iter()
                .filter(|p| p.key == "realm")
                .map(move |p| (&item.id, p))
        })
        .collect();

    // Non-vacuity: the items row 24 names must still be in the sweep's reach.
    for id in [
        "flaw.bound_to_realm",
        "flaw.necessary_realm_aura_for_ability",
        "flaw.realm_stigmatic",
        "virtue.folk_magic",
        "virtue.student_of_realm",
    ] {
        assert!(
            realm_params.iter().any(|(item, _)| item.as_str() == id),
            "{id} must declare a 'realm' parameter for this sweep to mean anything"
        );
    }

    let free_text: Vec<String> = realm_params
        .iter()
        .filter(|(_, p)| p.domain != ParameterDomain::Realm)
        .map(|(id, p)| format!("{id} ({})", p.domain))
        .collect();
    assert!(
        free_text.is_empty(),
        "a Realm is one of the four the engine models, never a typed word: {free_text:?}"
    );
}

/// E2 (open-todos row 24), the half that is about **not losing what a player
/// already typed**. A save written while the realm was free text holds a word
/// like "Faerie" in that slot; `Realm::from_id` resolves none of it.
///
/// No migration rewrites the save — "saves store choices, not resolved values",
/// and guessing which Realm a word meant would be inventing someone's rules
/// choice. The engine *reports* instead, through the established
/// `unknown_param_value`: the same code an unresolvable `item` or `ability` ref
/// already raises, and the only one whose args carry the offending **value**, so
/// the player is shown what they had typed and can pick the right Realm from the
/// picker the domain change gives them.
#[test]
fn a_free_text_realm_from_an_older_save_is_reported_in_the_players_own_words() {
    let rs = load_ruleset();
    let typed = Id::new("Faerie");
    let saved = entity(
        "companion",
        vec![Selection::with_params(
            Id::new("flaw.bound_to_realm"),
            BTreeMap::from([("realm".to_string(), typed.clone())]),
        )],
    );

    // The file is the player's. Loading it back leaves the typed word exactly
    // as written — no fold, no blank, no guessed Realm.
    let json = serde_json::to_string(&saved).expect("an entity serializes");
    let loaded = arm_rules::load_entity_migrating(&json, 1220).expect("an older save still loads");
    assert_eq!(
        loaded.entity.selections[0].params.get("realm"),
        Some(&SelectionParamValue::Single(typed.clone())),
        "the engine reports an unresolvable choice; it never rewrites the save"
    );

    let issue = validate(&loaded.entity, &rs)
        .issues
        .into_iter()
        .find(|i| i.code == ValidationIssue::CODE_UNKNOWN_PARAM_VALUE)
        .expect("a typed realm name resolves to no Realm, so it must be reported");
    assert_eq!(
        (
            issue.args.get("item").map(String::as_str),
            issue.args.get("key").map(String::as_str),
            issue.args.get("domain").map(String::as_str),
        ),
        (Some("flaw.bound_to_realm"), Some("realm"), Some("realm"))
    );
    assert_eq!(
        issue.args.get("value").map(String::as_str),
        Some(typed.as_str()),
        "the finding must carry the player's own words, or the choice is lost"
    );

    // And the remedy is one pick: a real Realm resolves and the finding is gone.
    let fixed = entity(
        "companion",
        vec![Selection::with_params(
            Id::new("flaw.bound_to_realm"),
            BTreeMap::from([("realm".to_string(), Realm::Faerie.id())]),
        )],
    );
    assert!(
        !issue_codes(&fixed, &rs).contains(&ValidationIssue::CODE_UNKNOWN_PARAM_VALUE.to_string()),
        "picking one of the four Realms clears it"
    );
}

#[test]
fn shipped_power_virtues_fund_the_power_levels_budget() {
    use arm_rules::power_levels_budget;
    let rs = load_ruleset();

    for (id, line, levels) in POWER_LEVEL_ITEMS {
        let one = entity("mythic_companion", vec![Selection::new(Id::new(*id))]);
        assert_eq!(
            power_levels_budget(&one, &rs),
            u32::from(*levels),
            "{id} grants {levels} levels of supernatural power \
             (ArMDE:{line})"
        );
    }
}

/// Focus Power's 25 are POINTS, not levels — "It costs 2 points to raise the
/// maximum level of effect by 1, and 1 point to raise the Penetration by 1"
/// (ArMDE:3899). Feeding them into the
/// level-denominated budget would silently double what the Virtue actually buys,
/// so the entry must grant nothing there.
#[test]
fn focus_power_funds_no_power_levels_because_its_pool_is_points() {
    use arm_rules::power_levels_budget;
    let rs = load_ruleset();

    let focused = entity(
        "mythic_companion",
        vec![Selection::new(Id::new("virtue.focus_power"))],
    );
    assert_eq!(
        power_levels_budget(&focused, &rs),
        0,
        "Focus Power's pool is 25 POINTS at 2 points per level of effect \
         (ArMDE:3899), a different \
         currency from the level budget the other Power Virtues fund"
    );
}

/// The other half of the exclusion above: Focus Power funds the pool it really
/// does grant. "This Virtue grants a pool of 25 points" (`ArMDE:3899`), and "This
/// Virtue may be taken more than once, and the points gained may be combined"
/// (`ArMDE:3903`) — so two copies give 50, in a currency of their own.
#[test]
fn shipped_focus_power_funds_a_twenty_five_point_pool_that_copies_combine() {
    use arm_rules::focus_points_budget;
    let rs = load_ruleset();

    let one = entity(
        "mythic_companion",
        vec![Selection::new(Id::new("virtue.focus_power"))],
    );
    assert_eq!(focus_points_budget(&one, &rs), 25);

    let twice = entity(
        "mythic_companion",
        vec![
            Selection::new(Id::new("virtue.focus_power")),
            Selection::new(Id::new("virtue.focus_power")),
        ],
    );
    assert_eq!(focus_points_budget(&twice, &rs), 50);
}

#[test]
fn shipped_per_power_items_carry_a_power_target() {
    let rs = load_ruleset();

    for (id, line) in PER_POWER_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));

        assert_eq!(
            item.max_per_target, 1,
            "{id} may not be taken twice for the SAME power \
             (ArMDE:{line})"
        );
        assert_eq!(
            item.max_total,
            u8::MAX,
            "{id} states no ceiling on the number of DIFFERENT powers it may \
             name (ArMDE:{line})"
        );

        let [param] = item.parameters.as_slice() else {
            panic!(
                "{id} must declare exactly one parameter naming the power, \
                 not {:?}",
                item.parameters
            );
        };
        assert_eq!(
            param.key, "power",
            "{id}'s per-copy target is keyed `power`"
        );
        assert_eq!(
            param.domain,
            ParameterDomain::Text,
            "a power is an anonymous instance of a Power Virtue, so its name \
             is free text the player types, not a registry ref"
        );
        assert!(
            param.require_power,
            "{id} restricts a power the character HAS, so the typed name must \
             match one of `entity.powers` \
             (ArMDE:{line})"
        );
    }
}

/// Builds a mythic companion holding one copy of `id` naming `power`, plus the
/// supernatural powers listed in `powers`.
fn entity_naming_power(id: &str, power: &str, powers: &[(&str, u16)]) -> Entity {
    let mut e = entity(
        "mythic_companion",
        vec![Selection::with_params(
            Id::new(id),
            BTreeMap::from([("power".to_string(), Id::new(power))]),
        )],
    );
    e.powers = powers
        .iter()
        .map(|(name, level)| SupernaturalPower {
            name: (*name).to_string(),
            level: *level,
            penetration: 0,
        })
        .collect();
    e
}

#[test]
fn a_per_power_item_naming_no_held_power_dangles() {
    let rs = load_ruleset();

    for (id, line) in PER_POWER_ITEMS {
        let codes = issue_codes(&entity_naming_power(id, "Wolf Shape", &[]), &rs);
        assert!(
            codes.contains(&"power_dangling_target".to_string()),
            "{id} restricts one of the character's own powers, so a name no \
             power carries restricts nothing \
             (ArMDE:{line}): {codes:?}"
        );
    }
}

#[test]
fn a_per_power_item_naming_a_held_power_is_clean() {
    let rs = load_ruleset();

    for (id, line) in PER_POWER_ITEMS {
        let codes = issue_codes(
            &entity_naming_power(
                id,
                "Wolf Shape",
                &[("Wolf Shape", 10), ("Curse of Sleep", 5)],
            ),
            &rs,
        );
        assert!(
            !codes.contains(&"power_dangling_target".to_string()),
            "{id} may name any power the character holds \
             (ArMDE:{line}): {codes:?}"
        );
    }
}

/// Builds a companion holding one copy of `id` per entry of `powers`, each
/// naming that power.
fn entity_with_powers(id: &str, powers: &[&str]) -> Entity {
    let selections = powers
        .iter()
        .map(|power| {
            Selection::with_params(
                Id::new(id),
                BTreeMap::from([("power".to_string(), Id::new(*power))]),
            )
        })
        .collect();
    entity("companion", selections)
}

#[test]
fn per_power_items_repeated_on_one_power_are_duplicates() {
    let rs = load_ruleset();

    for (id, line) in PER_POWER_ITEMS {
        let codes = issue_codes(&entity_with_powers(id, &["Wolf Shape", "Wolf Shape"]), &rs);
        assert!(
            codes.contains(&"duplicate_selection".to_string()),
            "{id} may not be taken twice for the same power \
             (ArMDE:{line}): {codes:?}"
        );
    }
}

#[test]
fn per_power_items_repeated_across_powers_are_clean() {
    let rs = load_ruleset();

    for (id, line) in PER_POWER_ITEMS {
        let codes = issue_codes(
            &entity_with_powers(id, &["Wolf Shape", "Curse of Sleep", "Summon Mist"]),
            &rs,
        );
        assert!(
            !codes.contains(&"duplicate_selection".to_string()),
            "{id} may be taken once for each power the character possesses \
             (ArMDE:{line}): {codes:?}"
        );
        assert!(
            !codes.contains(&"too_many_selections".to_string()),
            "{id} states no ceiling on the number of different powers \
             (ArMDE:{line}): {codes:?}"
        );
    }
}

#[test]
fn shipped_share_capped_items_carry_their_rulebook_ratio() {
    let rs = load_ruleset();

    for (id, line, numerator, denominator) in SHARE_CAPPED_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert_eq!(
            item.max_share_of_kind,
            Some(Share {
                numerator: *numerator,
                denominator: *denominator,
            }),
            "{id} may account for no more than {numerator}/{denominator} of its \
             kind's point total \
             (ArMDE:{line})"
        );
    }
}

#[test]
fn shipped_repeatable_items_carry_their_rulebook_ceiling() {
    let rs = load_ruleset();

    for (id, line) in UNLIMITED_REPEAT_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert_eq!(
            item.max_per_target,
            u8::MAX,
            "{id} may be taken more than once with no stated ceiling \
             (ArMDE:{line})"
        );
        assert!(
            item.parameters.is_empty(),
            "{id} carries no target parameter, so every copy shares one \
             duplicate key and only max_per_target can permit the repeat"
        );
        // D10: with no parameter, every copy shares ONE `(item_ref, {})` key,
        // so `max_total` and `max_per_target` govern the exact same set of
        // copies. The new `max_total` default of 1 would silently override a
        // higher `max_per_target` unless declared explicitly to match.
        assert_eq!(
            item.max_total,
            u8::MAX,
            "{id} has no parameter to carry a total ceiling separately from \
             its per-target one, so max_total must explicitly match \
             max_per_target's 'no stated ceiling' (ArMDE:{line}, D10)"
        );
    }

    for (id, line) in TWICE_ONLY_REPEAT_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert_eq!(
            item.max_per_target, 2,
            "{id} may be taken exactly twice \
             (ArMDE:{line})"
        );
    }
}

/// Slice Q4b (D9 part 1): each [`TEXT_TARGET_PARAM_ITEMS`] entry declares
/// exactly one free-text parameter keyed as the table states, with no stated
/// ceiling on the number of distinct targets and the default (1) per-target
/// cap, so an identical second copy still collides.
#[test]
fn shipped_text_target_param_items_carry_their_target_param() {
    let rs = load_ruleset();

    for (id, key, line) in TEXT_TARGET_PARAM_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));

        assert_eq!(
            item.max_per_target, 1,
            "{id} may not be taken twice for the SAME target (ArMDE:{line})"
        );
        assert_eq!(
            item.max_total,
            u8::MAX,
            "{id} states no ceiling on the number of DIFFERENT targets it may \
             name (ArMDE:{line})"
        );

        let [param] = item.parameters.as_slice() else {
            panic!(
                "{id} must declare exactly one target parameter, not {:?}",
                item.parameters
            );
        };
        assert_eq!(param.key, *key, "{id}'s per-copy target is keyed `{key}`");
        assert_eq!(
            param.domain,
            ParameterDomain::Text,
            "{id}'s target is open-ended (ArMDE:{line}), not a closed list or a \
             registry ref"
        );
    }
}

/// The repeat rule each descriptor states, exercised rather than asserted
/// about: two copies naming two different targets are legal, a second copy
/// naming the same target is not.
#[test]
fn text_target_param_items_repeat_across_targets_but_never_within_one() {
    let rs = load_ruleset();

    for (id, key, line) in TEXT_TARGET_PARAM_ITEMS {
        let different = issue_codes(&entity_with_param_values(id, key, &["Alpha", "Beta"]), &rs);
        assert!(
            !different.contains(&"unexpected_param".to_string())
                && !different.contains(&"duplicate_selection".to_string()),
            "{id} twice for two different targets is what the descriptor \
             permits (ArMDE:{line}): {different:?}"
        );

        let same = issue_codes(&entity_with_param_values(id, key, &["Alpha", "Alpha"]), &rs);
        assert!(
            same.contains(&"duplicate_selection".to_string()),
            "{id} twice for the SAME target is a repeat the descriptor does \
             not grant (ArMDE:{line}): {same:?}"
        );
    }
}

#[test]
fn shipped_once_only_items_stay_non_repeatable() {
    let rs = load_ruleset();

    for (id, line) in ONCE_ONLY_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert_eq!(
            item.max_per_target, 1,
            "{id} may not be taken more than once \
             (ArMDE:{line})"
        );
    }
}

#[test]
fn shipped_total_cap_items_carry_their_rulebook_ceiling() {
    let rs = load_ruleset();

    for (id, line, expected_max_total) in TOTAL_CAP_ITEMS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert_eq!(
            item.max_total, *expected_max_total,
            "{id} is capped at {expected_max_total} total across all targets \
             (ArMDE:{line})"
        );
    }

    // "You may take this Virtue twice, for two different Arts"
    // (ArMDE:3378, :4820): one copy
    // per Art, two Arts total. If `max_per_target` were ever raised here, the
    // same Art could be doubled up — this guards that it stays 1.
    for id in ["virtue.affinity_art", "virtue.puissant_art"] {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert_eq!(
            item.max_per_target, 1,
            "{id} may not be taken twice for the SAME Art"
        );
    }
}

/// False Power "may be taken multiple times, once for each appropriate
/// Supernatural Virtue that the character possesses, but in each subsequent
/// instance as a Minor Flaw rather than a Major one"
/// (ArMDE:6096; entry :6080-6096).
///
/// `magnitude` belongs to the catalogue entry, never to a selection, so the
/// per-copy magnitude change is expressed as a PAIR of entries — the shipped
/// Major, capped at one copy, plus `flaw.false_power_minor` for every subsequent
/// instance, gated on the Major by prerequisite so a Minor copy can never stand
/// alone. Unlike the `virtue.amorphous_major` / `virtue.amorphous_minor` pair the
/// two must **coexist**, so neither may list the other in `incompatible_with`.
#[test]
fn false_power_ships_as_a_coexisting_major_plus_minor_pair() {
    let rs = load_ruleset();
    let major_id = Id::new("flaw.false_power");
    let minor_id = Id::new("flaw.false_power_minor");
    let major = rs.item(&major_id).expect("flaw.false_power must ship");
    let minor = rs
        .item(&minor_id)
        .expect("flaw.false_power_minor must ship — the subsequent instances");

    assert_eq!(
        major.magnitude,
        Magnitude::Major,
        "the first instance of False Power is a Major Flaw (ArMDE:6081)"
    );
    assert_eq!(
        minor.magnitude,
        Magnitude::Minor,
        "each subsequent instance is a Minor Flaw rather than a Major one (ArMDE:6096)"
    );
    assert_eq!(minor.kind, ItemKind::Flaw);
    assert_eq!(
        minor.categories, major.categories,
        "both entries are the same descriptor's Flaw (*Major, Supernatural, Tainted*, :6081)"
    );
    assert_eq!(minor.classification, major.classification);
    assert_eq!(minor.entity_kinds, major.entity_kinds);
    assert_eq!(
        minor.source, major.source,
        "both entries are read off the same rulebook entry (ArMDE:6080-6096)"
    );
    assert!(
        major.tainted && minor.tainted,
        "False Power is a *Tainted* Flaw in both magnitudes (ArMDE:6081), so both \
         copies must feed the half-of-Flaw-points Tainted cap"
    );
    assert_eq!(
        minor.prerequisites,
        Some(Prereq::Has(major_id.clone())),
        "a Minor copy is a SUBSEQUENT instance, so it presupposes the Major one"
    );
    assert!(
        !major.incompatible_with.contains(&minor_id)
            && !minor.incompatible_with.contains(&major_id),
        "unlike a Major/Minor variant pair the two must coexist: the Minor copies \
         only exist once the Major one has been taken"
    );
}

#[test]
fn a_second_major_false_power_is_capped() {
    let rs = load_ruleset();
    let twice = entity(
        "companion",
        vec![
            Selection::new(Id::new("flaw.false_power")),
            Selection::new(Id::new("flaw.false_power")),
        ],
    );

    let codes = issue_codes(&twice, &rs);
    assert!(
        codes.contains(&"too_many_selections".to_string()),
        "only the FIRST instance of False Power is the Major one \
         (ArMDE:6096): {codes:?}"
    );
}

#[test]
fn a_minor_false_power_without_the_major_is_a_missing_prerequisite() {
    let rs = load_ruleset();
    let orphan = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.false_power_minor"))],
    );

    let codes = issue_codes(&orphan, &rs);
    assert!(
        codes.contains(&"prereq_not_met".to_string()),
        "a Minor False Power is a SUBSEQUENT instance and cannot be the first \
         (ArMDE:6096): {codes:?}"
    );
}

/// The whole point of the entry pair: the second and third copies cost 1 Flaw
/// point each, not 3. A single repeatable Major entry would have charged 9.
///
/// Each copy names a Virtue of its own, and the character holds all three —
/// "once for each appropriate Supernatural Virtue that the character possesses"
/// (`ArMDE:6096`) is what makes three copies legal in the first place, so a fixture
/// of three unnamed copies would no longer be the legal build this asserts.
#[test]
fn false_power_taken_three_times_costs_three_plus_one_plus_one() {
    let rs = load_ruleset();
    let thrice = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.second_sight")),
            Selection::new(Id::new("virtue.premonitions")),
            Selection::new(Id::new("virtue.dowsing")),
            false_power("flaw.false_power", "virtue.second_sight"),
            false_power("flaw.false_power_minor", "virtue.premonitions"),
            false_power("flaw.false_power_minor", "virtue.dowsing"),
        ],
    );

    assert_eq!(
        compute_balance(&thrice, &rs).flaw_points,
        3 + 1 + 1,
        "the first instance is Major (3) and each subsequent one Minor (1) \
         (ArMDE:6096)"
    );

    let codes = issue_codes(&thrice, &rs);
    for blocker in [
        "too_many_selections",
        "duplicate_selection",
        "prereq_not_met",
        "incompatible",
        "param_target_not_possessed",
        "param_target_already_claimed",
    ] {
        assert!(
            !codes.contains(&blocker.to_string()),
            "one Major plus two Minor False Powers is a legal build \
             (ArMDE:6096): {codes:?}"
        );
    }
}

/// The Minor entry rests entirely on its `has` prerequisite, and a False Power
/// can arrive as an off-budget grant rather than a bought row — a warping-owed
/// Major Flaw slot (ArMDE:16561) is
/// filled by choosing a real item. `validate_prerequisites` is handed the folded
/// grant list, so such a copy satisfies the Minor's prerequisite; asserted here
/// rather than assumed, because the whole entry pair rests on it.
#[test]
fn a_granted_major_false_power_satisfies_the_minor_prerequisite() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.false_power_minor"))],
    );
    // Warping Score 6 (105 Warping Points on the 5-per-score advancement curve)
    // owes one Major Flaw (ArMDE:16561).
    e.warping_points = 105;
    e.warping_choices = BTreeMap::from([(
        "warping.major_flaw.0".to_string(),
        Selection::new(Id::new("flaw.false_power")),
    )]);

    assert_eq!(
        arm_rules::warping_owed(&e, &rs).major_flaws,
        1,
        "the fixture must actually owe a Major Flaw slot for the grant to exist"
    );
    assert!(
        arm_rules::entity_grants(&e, &rs)
            .iter()
            .any(|s| s.item_ref == Id::new("flaw.false_power")),
        "the warping fill must fold into the grant list"
    );

    let codes = issue_codes(&e, &rs);
    assert!(
        !codes.contains(&"prereq_not_met".to_string()),
        "a GRANTED Major False Power is still a first instance, so the Minor \
         copy that follows it is legal: {codes:?}"
    );
}

/// One copy of a False Power entry, naming the Virtue it taints.
fn false_power(item: &str, target: &str) -> Selection {
    Selection::with_params(
        Id::new(item),
        BTreeMap::from([("virtue".to_string(), Id::new(target))]),
    )
}

/// "One of the character's Supernatural Virtues is associated with the Infernal
/// realm" (ArMDE:6082), taken "once
/// for each appropriate Supernatural Virtue that the character possesses"
/// (`ArMDE:6096`) — so every copy must NAME its Virtue, and that Virtue must be one
/// the character actually holds and is not already Infernal.
///
/// The three required categories are read off the book's own three examples at
/// `ArMDE:6082` — "Faerie Blood, Diedne Magic, or even The Gift" — which in this
/// catalogue carry `supernatural`, `hermetic` and `special` respectively. A
/// bare `supernatural` would have excluded two Virtues the source names
/// outright.
#[test]
fn false_power_names_the_supernatural_virtue_it_taints() {
    let rs = load_ruleset();

    for id in ["flaw.false_power", "flaw.false_power_minor"] {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        let [param] = item.parameters.as_slice() else {
            panic!(
                "{id} must declare exactly one parameter naming the tainted \
                 Virtue, not {:?}",
                item.parameters
            );
        };
        assert_eq!(param.key, "virtue");
        assert_eq!(
            param.domain,
            ParameterDomain::Item,
            "the target is a catalogue Virtue, not free text (ArMDE:6096)"
        );
        assert_eq!(
            param.require_categories,
            BTreeSet::from([
                "hermetic".to_string(),
                "special".to_string(),
                "supernatural".to_string(),
            ]),
            "the Flaw applies to Supernatural Virtues, and :6082 names Diedne \
             Magic (hermetic) and The Gift (special) among them"
        );
        assert!(
            param.require_possessed,
            "{id} taints a Virtue the character POSSESSES (ArMDE:6096)"
        );
        assert!(
            param.forbid_tainted,
            "{id} cannot apply to a Virtue already affiliated to the Infernal \
             realm (ArMDE:6096)"
        );
        assert_eq!(
            item.max_per_target, 1,
            "{id} is taken once for EACH Virtue (ArMDE:6096), so no two copies may \
             name the same one"
        );
    }

    // The other half of ":6096"'s ceiling, and why the Minor entry no longer
    // belongs in UNLIMITED_REPEAT_ITEMS: one copy per Virtue, but no stated
    // ceiling on how many DIFFERENT Virtues may be tainted. The Major entry
    // keeps its own `max_total: 1` — only the first instance is Major.
    assert_eq!(
        rs.item(&Id::new("flaw.false_power_minor"))
            .unwrap()
            .max_total,
        u8::MAX,
        "the book states no limit on the number of different Virtues tainted \
         (ArMDE:6096)"
    );
}

/// The row's original complaint was that the copies "do not name which
/// Supernatural Virtue they taint", so the *name* has to say it: without a
/// `{virtue}` placeholder the app's row would read "False Power (Minor)" three
/// times over, telling the player nothing. Both locales, since a placeholder in
/// one and not the other is a sheet that changes meaning with the language.
#[test]
fn both_locales_name_the_virtue_a_false_power_taints() {
    for (lang, json) in [
        (
            "en",
            include_str!("../../../rules/i18n/en/virtues_flaws.json"),
        ),
        (
            "de",
            include_str!("../../../rules/i18n/de/virtues_flaws.json"),
        ),
    ] {
        let i18n: serde_json::Value = serde_json::from_str(json).unwrap();
        for id in ["flaw.false_power", "flaw.false_power_minor"] {
            let name = i18n[id]["name"].as_str().unwrap_or_else(|| {
                panic!("{lang} i18n must name {id}");
            });
            assert!(
                name.contains("{virtue}"),
                "{lang} name for {id} must show the Virtue it taints, got {name:?}"
            );
        }
    }
}

#[test]
fn false_power_cannot_taint_a_virtue_the_character_lacks() {
    let rs = load_ruleset();

    let lacking = entity(
        "companion",
        vec![false_power("flaw.false_power", "virtue.second_sight")],
    );
    assert!(
        issue_codes(&lacking, &rs).contains(&"param_target_not_possessed".to_string()),
        "the Flaw taints a Virtue the character possesses \
         (ArMDE:6096): {:?}",
        issue_codes(&lacking, &rs)
    );

    let holding = entity(
        "companion",
        vec![
            false_power("flaw.false_power", "virtue.second_sight"),
            Selection::new(Id::new("virtue.second_sight")),
        ],
    );
    assert!(
        !issue_codes(&holding, &rs).contains(&"param_target_not_possessed".to_string()),
        "(False) Second Sight is the book's own example (ArMDE:6086): {:?}",
        issue_codes(&holding, &rs)
    );
}

/// "Also note that this Flaw cannot apply to Supernatural Virtues that are
/// affiliated to the Infernal realm in the first place" (`ArMDE:6096`). Infernal
/// affiliation is the descriptor's *Tainted* tag, so Demonic Blood — held or
/// not — is outside the parameter's domain.
#[test]
fn false_power_cannot_taint_an_already_infernal_virtue() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            false_power("flaw.false_power", "virtue.demonic_blood"),
            Selection::new(Id::new("virtue.demonic_blood")),
        ],
    );
    assert!(
        issue_codes(&e, &rs).contains(&"unknown_param_value".to_string()),
        "a Tainted Virtue is already Infernal and cannot be made falser (ArMDE:6096): {:?}",
        issue_codes(&e, &rs)
    );
}

/// The gap `max_per_target` is structurally blind to: its duplicate key is
/// `(item_ref, params)`, and the Major and Minor entries are different ids, so
/// nothing stopped both from naming one Virtue. "Once for each appropriate
/// Supernatural Virtue" (`ArMDE:6096`) says they may not.
#[test]
fn a_major_and_a_minor_false_power_cannot_taint_the_same_virtue() {
    let rs = load_ruleset();

    let same = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.second_sight")),
            false_power("flaw.false_power", "virtue.second_sight"),
            false_power("flaw.false_power_minor", "virtue.second_sight"),
        ],
    );
    let claimed: Vec<_> = validate(&same, &rs)
        .issues
        .into_iter()
        .filter(|i| i.code == "param_target_already_claimed")
        .collect();
    assert_eq!(
        claimed.len(),
        1,
        "one Virtue tainted twice is one mistake, reported against the second \
         copy: {claimed:?}"
    );
    assert_eq!(claimed[0].context, Some(Id::new("flaw.false_power_minor")));

    let different = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.second_sight")),
            Selection::new(Id::new("virtue.premonitions")),
            false_power("flaw.false_power", "virtue.second_sight"),
            false_power("flaw.false_power_minor", "virtue.premonitions"),
        ],
    );
    assert!(
        !issue_codes(&different, &rs).contains(&"param_target_already_claimed".to_string()),
        "one copy per Virtue is exactly what :6096 permits: {:?}",
        issue_codes(&different, &rs)
    );
}

/// Possession is by **id**, the `Prereq::Has` notion — a `taken_as` reading is
/// not consulted. Sufi is "either as a Minor Social Status Virtue or a Minor
/// Supernatural Virtue" (`ArMDE:5083`), and a Sufi taken as Social Status is still a
/// Virtue the character holds, so False Power may name it.
///
/// Deliberate, and the same answer the *domain* half already gives: a parameter
/// value is a bare id naming an item, not a `Selection` of one, so
/// `require_categories` reads `PointItem::categories` and admits Sufi through
/// its Supernatural membership whichever reading was taken. Two notions of
/// "is this Virtue Supernatural for this character" inside one parameter would
/// be worse than one lenient one; the troupe adjudicates the rest.
#[test]
fn a_sufi_taken_as_social_status_is_still_a_possessed_false_power_target() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.sufi"),
                BTreeMap::from([("taken_as".to_string(), Id::new("social_status"))]),
            ),
            false_power("flaw.false_power", "virtue.sufi"),
        ],
    );
    assert!(
        !issue_codes(&e, &rs).contains(&"param_target_not_possessed".to_string()),
        "a held Virtue is held whichever category it was taken as: {:?}",
        issue_codes(&e, &rs)
    );

    // The arm that makes the one above mean something: drop the Sufi row and
    // the very same target IS flagged, so the clean result is possession
    // answering "yes", not the check being absent.
    let without_sufi = entity(
        "companion",
        vec![false_power("flaw.false_power", "virtue.sufi")],
    );
    assert!(
        issue_codes(&without_sufi, &rs).contains(&"param_target_not_possessed".to_string()),
        "an unheld Sufi is no target at all: {:?}",
        issue_codes(&without_sufi, &rs)
    );
}

#[test]
fn repeated_selections_are_not_reported_as_duplicates() {
    let rs = load_ruleset();
    let twice = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.improved_characteristics")),
            Selection::new(Id::new("virtue.improved_characteristics")),
        ],
    );

    let codes = issue_codes(&twice, &rs);
    assert!(
        !codes.contains(&"duplicate_selection".to_string()),
        "Improved Characteristics may be taken multiple times \
         (ArMDE:4105): {codes:?}"
    );
}

#[test]
fn repeated_selections_stack_their_effects() {
    use arm_rules::{characteristic_points_granted, power_levels_budget};
    let rs = load_ruleset();

    // "You have an additional three points to spend on buying Characteristics
    // ... You may take this Virtue multiple times." (ArMDE:4105)
    let two_improved = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.improved_characteristics")),
            Selection::new(Id::new("virtue.improved_characteristics")),
        ],
    );
    assert_eq!(
        characteristic_points_granted(&two_improved, &rs),
        6,
        "two copies of Improved Characteristics grant 3 + 3 points"
    );

    // "He gains an extra 20 levels of Infernal Powers ... You may also take
    // this Virtue more than once" (ArMDE:3669).
    let two_demonic = entity(
        "mythic_companion",
        vec![
            Selection::new(Id::new("virtue.demonic_powers")),
            Selection::new(Id::new("virtue.demonic_powers")),
        ],
    );
    assert_eq!(
        power_levels_budget(&two_demonic, &rs),
        40,
        "two copies of Demonic Powers grant 20 + 20 levels"
    );
}

// --- Regression: a House-granted Puissant Art must count against a bought one ---
//
// Commits 8801b03 / bb305fd / a2dc130 fixed a wrong-rules-output bug: House
// Flambeau grants a free choice of Puissant Perdo OR Puissant Ignem
// (`rules/core/houses.json`, `choice_key` `flambeau_puissant`); granted
// selections are resolved at evaluation time and never stored on
// `entity.selections`, so the duplicate/total-cap checks — which used to read
// `entity.selections` alone — could not see them. A Flambeau magus granted
// Puissant Ignem who also BOUGHT Puissant Ignem therefore validated clean and
// stacked +6 onto Ignem, against ArMDE:4820 ("You may take this Virtue
// twice, for two different Arts").
// The three tests below pin the fix against the SHIPPED ruleset
// (`load_full_ruleset()`), not a synthetic one, since the bug lived in the real
// `rules/core/houses.json` grant data and a future data edit (e.g. raising
// `virtue.puissant_art`'s `max_total` past 2) could silently reopen it.

/// Builds a Flambeau magus whose granted `flambeau_puissant` pick is Puissant
/// `granted_art`, plus one bought Puissant Art selection per entry of
/// `bought_arts`.
fn flambeau_magus_with_puissant_arts(granted_art: &str, bought_arts: &[&str]) -> Entity {
    let puissant_art = |art: &str| {
        Selection::with_params(
            Id::new("virtue.puissant_art"),
            BTreeMap::from([("art".to_string(), Id::new(art))]),
        )
    };

    let selections = bought_arts.iter().map(|art| puissant_art(art)).collect();
    let mut e = entity("magus", selections);
    e.house = Some(Id::new("house.flambeau"));
    e.house_choices =
        BTreeMap::from([("flambeau_puissant".to_string(), puissant_art(granted_art))]);
    e
}

/// The regression that matters most: a Flambeau magus granted Puissant Ignem
/// who ALSO buys Puissant Ignem is taking the same Virtue for the same Art
/// twice, which the descriptor forbids.
#[test]
fn flambeau_granted_and_bought_same_art_is_a_duplicate() {
    let rs = load_full_ruleset();
    let e = flambeau_magus_with_puissant_arts("art.ignem", &["art.ignem"]);

    let codes = issue_codes(&e, &rs);
    assert!(
        codes.contains(&"duplicate_selection".to_string()),
        "a granted Puissant Ignem plus a bought Puissant Ignem must be flagged \
         as a duplicate of virtue.puissant_art: {codes:?}"
    );
}

/// The guard against over-correcting: a Flambeau magus granted Puissant Ignem
/// who buys Puissant Perdo instead holds two DIFFERENT Arts — exactly what
/// "twice, for two different Arts" (ArMDE:4820) allows. Neither the per-target
/// duplicate check nor the total cap (`max_total` 2, and this is only 2
/// copies) may fire.
#[test]
fn flambeau_granted_and_bought_different_art_is_clean() {
    let rs = load_full_ruleset();
    let e = flambeau_magus_with_puissant_arts("art.ignem", &["art.perdo"]);

    let codes = issue_codes(&e, &rs);
    assert!(
        !codes.contains(&"duplicate_selection".to_string()),
        "granted Puissant Ignem and bought Puissant Perdo are different \
         targets, not a duplicate: {codes:?}"
    );
    assert!(
        !codes.contains(&"too_many_selections".to_string()),
        "two total copies across two different Arts stays within max_total: {codes:?}"
    );
}

/// A Flambeau magus granted Puissant Ignem who ALSO buys Puissant Perdo AND
/// Puissant Muto holds three copies of `virtue.puissant_art` across three
/// different Arts — over its `max_total` of 2 — even though no two copies
/// share a target, so the per-target duplicate check never fires.
#[test]
fn flambeau_granted_plus_two_bought_arts_exceeds_the_total_cap() {
    let rs = load_full_ruleset();
    let e = flambeau_magus_with_puissant_arts("art.ignem", &["art.perdo", "art.muto"]);

    let codes = issue_codes(&e, &rs);
    assert!(
        codes.contains(&"too_many_selections".to_string()),
        "three total copies of virtue.puissant_art must exceed its max_total \
         of 2: {codes:?}"
    );
}

// ---------------------------------------------------------------------------
// Opening a v0.2.x save (Slice 0)
//
// Three shipped rounds each recorded an "accepted save impact", so a character
// built with v0.2.0 lit up with errors on the first open under 0.3. The `being`
// fold in `load_entity_migrating` recovers what is mechanically recoverable; the
// tests below pin what it recovers, what it must NOT invent, and the one class of
// error that is a genuine rules finding rather than a format problem.
// ---------------------------------------------------------------------------

/// A save in exactly the shape v0.2.x wrote: `schema_version` 14, no
/// `ability_funding` key, and typed free text in the three `being` slots that are
/// enumerated today. Hand-written, because current code cannot produce it.
const V0_2_X_MAGUS_SAVE: &str = r#"{
  "schema_version": 14,
  "ruleset": { "id": "arm5-core", "version": "2024.1" },
  "entity_kind": "character",
  "type_id": "magus",
  "name": "Iohannes filius Bonisagi",
  "xp_pool": 240,
  "selections": [
    { "ref": "virtue.the_gift" },
    { "ref": "virtue.hermetic_magus" },
    { "ref": "flaw.offensive_to_beings", "params": { "being": "Mundane Humans" } },
    { "ref": "flaw.unbearable_to_beings", "params": { "being": "  dämonen " } },
    { "ref": "virtue.inoffensive_to_beings", "params": { "being": "Göttliche Wesen" } },
    { "ref": "flaw.slow_power" },
    { "ref": "virtue.folk_magic" }
  ]
}"#;

/// The `missing_param` issues a validation result carries, as `(item, key)` pairs.
fn missing_param_targets(entity: &Entity, rs: &Ruleset) -> Vec<(String, String)> {
    let mut targets: Vec<(String, String)> = validate(entity, rs)
        .issues
        .into_iter()
        .filter(|issue| issue.code == "missing_param")
        .map(|issue| {
            (
                issue.args.get("item").cloned().unwrap_or_default(),
                issue.args.get("key").cloned().unwrap_or_default(),
            )
        })
        .collect();
    targets.sort();
    targets
}

/// The recoverable half: every typed `being` label the app or the rulebook printed
/// resolves in its enumerated domain after the fold, in both shipped languages, so
/// no `unknown_param_value` is left for the player to puzzle over.
#[test]
fn a_v0_2_x_saves_typed_being_values_resolve_after_migration() {
    let rs = load_ruleset();
    let entity = arm_rules::load_entity_migrating(V0_2_X_MAGUS_SAVE, arm_rules::DEFAULT_SAGA_YEAR)
        .expect("a v0.2.x save still loads")
        .entity;

    let codes = issue_codes(&entity, &rs);
    assert!(
        !codes.contains(&"unknown_param_value".to_string()),
        "the being fold must leave no unresolved param value: {codes:?}"
    );
}

/// The unrecoverable half, and the reason nothing is faked: v0.2.x declared no
/// `power` on the three per-power Flaws and neither `category` nor `realm` on
/// Folk Magic, so there is nothing to migrate *from*. Each stays exactly one
/// actionable `missing_param` naming the item and the key the player must
/// supply — inventing a placeholder would invent a character's rules choices.
///
/// Folk Magic's `realm` (B7, `ArMDE:3909`) joined the list on exactly the standing
/// policy its `category` set: a `ParameterDef` is **ruleset** shape, not save
/// shape, so `SCHEMA_VERSION` neither moves nor could, and no save distinguishes
/// the two eras. The player is asked, not guessed at.
#[test]
fn the_choices_a_v0_2_x_save_never_stored_stay_one_actionable_issue_each() {
    let rs = load_ruleset();
    let entity = arm_rules::load_entity_migrating(V0_2_X_MAGUS_SAVE, arm_rules::DEFAULT_SAGA_YEAR)
        .expect("a v0.2.x save still loads")
        .entity;

    assert_eq!(
        missing_param_targets(&entity, &rs),
        vec![
            ("flaw.slow_power".to_string(), "power".to_string()),
            ("virtue.folk_magic".to_string(), "category".to_string()),
            ("virtue.folk_magic".to_string(), "realm".to_string()),
        ],
        "exactly one issue per unstored choice, each naming its item and key"
    );
}

/// A Flambeau magus granted Puissant Perdo who also bought Puissant Creo and
/// Puissant Muto is over `virtue.puissant_art`'s `max_total` of 2. That is a
/// **genuine rules violation** the engine was previously blind to, not a save-format
/// problem, so the migration must leave it standing while still folding the save's
/// typed `being` value. Nobody may later "fix" this by suppressing it on load.
#[test]
fn a_genuine_too_many_selections_survives_the_being_migration() {
    let rs = load_full_ruleset();
    let save = r#"{
      "schema_version": 14,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "magus",
      "house": "house.flambeau",
      "house_choices": {
        "flambeau_puissant": { "ref": "virtue.puissant_art", "params": { "art": "art.perdo" } }
      },
      "selections": [
        { "ref": "virtue.the_gift" },
        { "ref": "virtue.hermetic_magus" },
        { "ref": "virtue.puissant_art", "params": { "art": "art.creo" } },
        { "ref": "virtue.puissant_art", "params": { "art": "art.muto" } },
        { "ref": "flaw.unbearable_to_beings", "params": { "being": "Demons" } }
      ]
    }"#;
    let entity = arm_rules::load_entity_migrating(save, arm_rules::DEFAULT_SAGA_YEAR)
        .expect("a v0.2.x save still loads")
        .entity;

    let codes = issue_codes(&entity, &rs);
    assert!(
        codes.contains(&"too_many_selections".to_string()),
        "the over-cap Puissant Arts are a rules finding the migration must not \
         paper over: {codes:?}"
    );
    assert!(
        !codes.contains(&"unknown_param_value".to_string()),
        "and the being value in the same save is still folded: {codes:?}"
    );
}

// --- Magic Resistance: the mechanic each passage states, not the one the Flaw's
// --- name suggests ---------------------------------------------------------
//
// Two encodings were parked as `KNOWN_MISENCODINGS` rows by the rules-semantics
// guard (`rules_source_provenance.rs`) and are corrected here. Both tests run
// against the SHIPPED `rules/core/virtues_flaws.json` through `load_full_ruleset`,
// because the wrong encoding lived in that data and a synthetic fixture could not
// have caught it.

/// A magus with Parma Magica 3 and Ignem 8 — enough for a per-Form Magic
/// Resistance number to exist — carrying `selections`.
fn magus_with_parma(selections: Vec<Selection>) -> Entity {
    let mut e = entity("magus", selections);
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.parma_magica"),
        parameter: None,
        specialty: None,
        score: 3,
    }];
    e.art_scores = vec![ArtScore {
        art: Id::new("art.ignem"),
        score: 8,
    }];
    e
}

/// Weak Magic Resistance halves nothing. `ArMDE:7070`: "Any form of Magic
/// Resistance you generate is much weaker under relatively common circumstances
/// which are fairly easy for an opponent to utilize ... If the conditions are met,
/// do not subtract the level of the effect from the casting total before
/// calculating Penetration." Normal Penetration is the Casting Total less the
/// spell level (`ArMDE:7066`), so the Flaw makes an *attacker* skip that
/// subtraction: the size of the effect is the level of the **incoming** spell and
/// the trigger is a scene call. Neither is knowable from this sheet, and the
/// carrier's own Magic Resistance score does not change at all — so no number on
/// the sheet may differ because of this Flaw. `ArMDE:9912` settles the reading,
/// glossing the Clan Ilfetu secret name as "need not subtract the spell level from
/// the Penetration total ... much like the Weak Magic Resistance Flaw".
#[test]
fn weak_magic_resistance_changes_no_number_on_the_sheet() {
    let rs = load_full_ruleset();
    let clean = arm_rules::derived_totals(&magus_with_parma(Vec::new()), &rs);
    let weak = arm_rules::derived_totals(
        &magus_with_parma(vec![Selection::new(Id::new("flaw.weak_magic_resistance"))]),
        &rs,
    );
    assert_eq!(
        weak.magic_resistance, clean.magic_resistance,
        "ArMDE:7070 gives no halving: the carrier's own Magic Resistance is unchanged"
    );
}

/// …and because it is not computed, it must be **listed**. The condition and the
/// incoming spell level are scene facts, so the honest encoding is a surfaced-only
/// Magic Resistance note (amount 0) — the shape `aura_bonus` and the two realm
/// susceptibilities already use for a Magic Resistance rule that cannot be folded
/// into the flat per-Form figure.
#[test]
fn weak_magic_resistance_is_surfaced_as_a_conditional_magic_resistance_note() {
    let rs = load_full_ruleset();
    let d = arm_rules::derived_totals(
        &magus_with_parma(vec![Selection::new(Id::new("flaw.weak_magic_resistance"))]),
        &rs,
    );
    assert!(
        d.surfaced_modifiers
            .iter()
            .any(|m| m.family == arm_rules::ModifierFamily::MagicResistance
                && m.detail == "conditional_penetration_waiver"
                && m.amount == 0),
        "the rule must be listed rather than silently dropped: {:?}",
        d.surfaced_modifiers
    );
}

/// Susceptibility to Divine Power is not a Magic Resistance rule. `ArMDE:6817`:
/// "You are especially sensitive to the Dominion and suffer twice the normal
/// penalties (such as spellcasting modifiers and botch dice) to your magic when in
/// a Divine aura." The Aura Modifier is a term of the Casting Score, so this is a
/// casting-side quirk; Magic Resistance is never mentioned. Its two siblings *are*
/// Magic Resistance rules, and their agreement is what carried the wrong encoding
/// across all three.
#[test]
fn susceptibility_to_divine_power_is_a_casting_quirk_not_a_magic_resistance_mod() {
    let rs = load_full_ruleset();
    let d = arm_rules::derived_totals(
        &magus_with_parma(vec![Selection::new(Id::new(
            "flaw.susceptibility_to_divine_power",
        ))]),
        &rs,
    );
    assert!(
        d.surfaced_modifiers
            .iter()
            .any(|m| m.family == arm_rules::ModifierFamily::SpecialCasting
                && m.detail == "doubled_aura_penalty"
                && m.amount == 0),
        "ArMDE:6817 doubles the aura's casting penalties and botch dice: {:?}",
        d.surfaced_modifiers
    );
    assert!(
        !d.surfaced_modifiers
            .iter()
            .any(|m| m.family == arm_rules::ModifierFamily::MagicResistance),
        "ArMDE:6817 never mentions Magic Resistance: {:?}",
        d.surfaced_modifiers
    );
}

/// The sibling check, pinned so the fix above cannot drag the two correct
/// encodings with it. `ArMDE:6821` (Faerie) and `ArMDE:6825` (Infernal) both do
/// halve Magic Resistance, but each halves it only "against faerie effects" /
/// "against infernal effects" — a realm scope no sheet number can carry. They stay
/// `magic_resistance_mod`s, surfaced at amount 0 rather than folded into the flat
/// per-Form total, which is what keeps the realm scope honest: a global halving
/// would be wrong output on every Form against every attacker.
#[test]
fn the_realm_scoped_susceptibilities_are_surfaced_and_halve_no_flat_total() {
    let rs = load_full_ruleset();
    let clean = arm_rules::derived_totals(&magus_with_parma(Vec::new()), &rs);
    for (id, detail) in [
        ("flaw.susceptibility_to_faerie_power", "susceptible_faerie"),
        (
            "flaw.susceptibility_to_infernal_power",
            "susceptible_infernal",
        ),
    ] {
        let d =
            arm_rules::derived_totals(&magus_with_parma(vec![Selection::new(Id::new(id))]), &rs);
        assert_eq!(
            d.magic_resistance, clean.magic_resistance,
            "{id} is realm-scoped, so it may not halve the flat per-Form total"
        );
        assert!(
            d.surfaced_modifiers
                .iter()
                .any(|m| m.family == arm_rules::ModifierFamily::MagicResistance
                    && m.detail == detail
                    && m.amount == 0),
            "{id} must still be listed: {:?}",
            d.surfaced_modifiers
        );
    }
}

/// Every `CombatMod` figure the four "combat rolls"/"combat scores" Flaws carry.
fn combat_mods_of(rs: &Ruleset, id: &str) -> Vec<(i32, CombatStat, Option<String>)> {
    rs.item(&Id::new(id))
        .unwrap_or_else(|| panic!("{id} is a shipped item"))
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::CombatMod {
                amount,
                target,
                weapon,
            } => Some((
                i32::from(*amount),
                *target,
                weapon.as_ref().map(|w| w.as_str().to_string()),
            )),
            _ => None,
        })
        .collect()
}

/// Row 37's decision, encoded: "combat rolls"/"combat scores" means the totals
/// that take a **Combat Ability** — Attack and Defense. ArMDE:16660 gives ATTACK
/// TOTAL = Dexterity + Combat Ability + Weapon Attack Modifier + Stress Die and
/// ArMDE:16662 gives DEFENSE TOTAL = Quickness + Combat Ability + Weapon Defense
/// Modifier + Stress Die, while ArMDE:16658's INITIATIVE TOTAL = Quickness +
/// Weapon Initiative Modifier - Encumbrance + Stress Die carries none, so
/// Initiative is not a "combat roll" in this sense; ArMDE:16664 and :16666 leave
/// Damage and Soak likewise Ability-free.
///
/// - `flaw.hobbled` — "Her Dodge and other combat rolls are penalized by -6"
///   (ArMDE:6262). One figure, so Dodge needs no separate scope.
/// - `flaw.lame` — "-3 on Dodge, and -1 on other combat scores" (ArMDE:6332).
///   Two figures, and Dodge is a weapon-table row (ArMDE:16959), so the -3 is
///   scoped to `weapon.dodge` and the -1 is left to every other weapon.
/// - `flaw.missing_hand` — "Climbing, combat, and other activities normally
///   requiring both hands are at a penalty of -3 or greater" (ArMDE:6440). It
///   says "combat", so it reaches both Combat-Ability totals.
/// - `flaw.palsied_hands` — "All rolls involving holding or wielding an object
///   are made at -2, including weapon skills" (ArMDE:6580).
#[test]
fn the_combat_roll_flaws_penalize_the_combat_ability_totals() {
    let rs = load_ruleset();

    assert_eq!(
        combat_mods_of(&rs, "flaw.hobbled"),
        vec![
            (-6, CombatStat::Attack, None),
            (-6, CombatStat::Defense, None)
        ]
    );
    assert_eq!(
        combat_mods_of(&rs, "flaw.lame"),
        vec![
            (-1, CombatStat::Attack, None),
            (-1, CombatStat::Defense, None),
            (-3, CombatStat::Defense, Some("weapon.dodge".to_string())),
        ]
    );
    assert_eq!(
        combat_mods_of(&rs, "flaw.missing_hand"),
        vec![
            (-3, CombatStat::Attack, None),
            (-3, CombatStat::Defense, None)
        ]
    );
    assert_eq!(
        combat_mods_of(&rs, "flaw.palsied_hands"),
        vec![
            (-2, CombatStat::Attack, None),
            (-2, CombatStat::Defense, None)
        ]
    );

    // Initiative takes no Combat Ability, so none of the four may touch it.
    for id in [
        "flaw.hobbled",
        "flaw.lame",
        "flaw.missing_hand",
        "flaw.palsied_hands",
    ] {
        assert!(
            combat_mods_of(&rs, id)
                .iter()
                .all(|(_, target, _)| *target != CombatStat::Initiative),
            "{id} must not modify Initiative"
        );
    }
}

/// D54 (Q-111): `ArMDE:6304` ends "If you are not using the rules in City and
/// Guild (page 73), treat this as a Personality Flaw." The app supports no
/// supplements yet, so that condition is satisfied today and the shipped
/// category must be `personality`, not the book's own descriptor/index
/// spelling of `General` (which is the *City and Guild*-in-play reading).
/// Registered in `corrections.md` § 8a for the future supplement-selection
/// feature to flip back.
#[test]
fn independent_craftsman_ships_as_personality_pending_city_and_guild() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.independent_craftsman"))
        .expect("flaw.independent_craftsman must ship");
    assert_eq!(
        item.categories,
        vec!["personality".to_string()],
        "no supplement support yet, so ArMDE:6304's fallback applies (D54)"
    );
}

/// F-349's risk 2 (`docs/vf-audit/design-c0-parameter-model.md` § 3): the
/// exclusive-choice gate is sound only WITHIN one `Selection` — a bought copy
/// and an independently-gated GRANTED copy of the same item, voting
/// oppositely, would union past it (each selection's fold runs on its own;
/// there is no cross-selection dedup before the union). Not live today
/// because no entry in the catalogue grants any of the four gated carriers —
/// this test pins that absence, scanning every static grant target
/// (`grants_selection`, House/Mythic-type `Fixed`/`Choice` grants) plus every
/// Open grant's `kind`/`magnitude`/`category` constraint (House, Mythic-type,
/// and the three warping-fill shapes), so the catalogue cannot silently
/// reopen the trap without this test noticing. The moment one of these four
/// becomes grantable, the fix is grant-aware deduplication (D2's territory),
/// not a defect in this design.
#[test]
fn no_gated_authorization_item_is_ever_granted() {
    let rs = load_ruleset();
    let gated = [
        "virtue.wise_one",
        "virtue.custos",
        "virtue.templar_specialist",
        "virtue.student_of_realm",
    ];

    // 1. `grants_selection` (nested V/F grants) never names one of the four.
    for item in rs.items() {
        for effect in &item.effects {
            if let Effect::GrantsSelection { items } = effect {
                for granted in items {
                    assert!(
                        !gated.contains(&granted.as_str()),
                        "{} carries a grants_selection naming gated item '{}' — F-349's \
                         cross-selection union risk is now live",
                        item.id,
                        granted
                    );
                }
            }
        }
    }

    // 2. Every House/Mythic-type Fixed/Choice grant target never names one of
    //    the four — a SPECIFIC, catalog-editable risk (unlike the Open-grant
    //    finding below, this one genuinely does not exist today).
    let mut fixed_and_choice_targets: Vec<Id> = Vec::new();
    let mut collect = |grants: &[Grant]| {
        for grant in grants {
            match grant {
                Grant::Fixed { item, .. } => fixed_and_choice_targets.push(item.clone()),
                Grant::Choice { options, .. } => {
                    fixed_and_choice_targets.extend(options.iter().map(|s| s.item_ref.clone()));
                }
                Grant::Open { .. } => {}
            }
        }
    };
    for house in rs.houses() {
        collect(&house.grants);
    }
    for mythic_type in rs.mythic_types() {
        collect(&mythic_type.grants);
    }
    for id in gated {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        assert!(
            !fixed_and_choice_targets.contains(&item.id),
            "'{id}' is named by a Fixed/Choice grant target — F-349's cross-selection union \
             risk is now live",
        );
    }

    // 3. Open grants (House/Mythic-type free picks, and the three
    //    warping-fill shapes) are a DIFFERENT, broader story, discovered
    //    while writing this test rather than assumed away: Jerbiton's free
    //    Minor Virtue (`house.jerbiton`, `rules/core/houses.json`) is
    //    `{ kind: virtue, magnitude: minor }` with NO category restriction at
    //    all — and all four gated items are Minor Virtues, so EVERY one of
    //    them (not just Wise One) already satisfies it today, along with
    //    hundreds of other unrelated Minor Virtues in the catalogue. This is
    //    not a defect specific to the four gated items; it is a structural
    //    property of an unconstrained Open grant, and the design note's "not
    //    live today: all four are bought-only" undersold it for this path.
    //    Fixing it is D2's grant-aware-deduplication territory (per the
    //    note's own § 3, "recorded as a coupling risk, not fixed here") — out
    //    of C1's scope. Pinned here as a plain fact, not swept under: if this
    //    ever tightens (Jerbiton's grant gains a category restriction, say),
    //    this assertion starts failing and should be revisited rather than
    //    deleted outright.
    let jerbiton_minor_virtue = GrantConstraint {
        kind: ItemKind::Virtue,
        magnitude: Some(Magnitude::Minor),
        require_categories: BTreeSet::new(),
        forbid_categories: BTreeSet::new(),
    };
    for id in gated {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        let pick = Selection::new(item.id.clone());
        assert!(
            open_pick_satisfies(&pick, &jerbiton_minor_virtue, &rs, None),
            "'{id}' no longer satisfies Jerbiton's unconstrained free-Minor-Virtue grant — \
             either it is no longer a Minor Virtue (update this test) or the grant gained a \
             restriction (update the note above and `docs/vf-audit/design-c0-parameter-model.md`)",
        );
    }

    // The three warping-fill shapes, by contrast, genuinely do NOT reach any
    // of the four: two are Flaw-kind (kind mismatch, all four are Virtues),
    // and the third (a supernatural Minor Virtue) requires the `supernatural`
    // category, which none of the four carries (they are `social_status` /
    // `general`) — built from a companion entity whose Warping Score is high
    // enough to owe all three at once (score >= 6), via the same public
    // function the wizard calls.
    let mut warping_entity = entity("companion", vec![]);
    warping_entity.warping_points = 100;
    let warping_open_grants: Vec<Grant> = warping_owed_grants(&warping_entity, &rs);
    assert!(
        warping_open_grants.len() >= 3,
        "expected at least one Open grant of each warping-fill shape (got {})",
        warping_open_grants.len()
    );
    for id in gated {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must ship"));
        let pick = Selection::new(item.id.clone());
        for grant in &warping_open_grants {
            let Grant::Open { constraint, .. } = grant else {
                panic!("warping_owed_grants must only ever produce Open grants");
            };
            assert!(
                !open_pick_satisfies(&pick, constraint, &rs, None),
                "a warping-fill Open grant constraint now matches gated item '{id}' — F-349's \
                 cross-selection union risk is now live",
            );
        }
    }
}
