//! Property-based serialization tests. Complements the hand-picked example
//! roundtrips in the unit modules by fuzzing arbitrary `Prereq` trees and
//! `Entity` values through serialize -> deserialize, asserting idempotence and
//! canonical (input-order-independent) output.

use arm_rules::{
    AbilityFunding, AbilityScore, AgingLogEntry, ArtScore, Characteristic, EnchantedDevice, Entity,
    EntityKind, EquipmentSlot, Familiar, FocusPower, Id, MightScore, PersonalityTrait, Prereq,
    Realm, Reputation, ReputationType, RulesetRef, Selection, SpellSelection, SupernaturalPower,
    Talisman, TalismanAttunement, TalismanEffect, TwilightScar,
};
use proptest::prelude::*;

fn arb_id() -> impl Strategy<Value = Id> {
    "[a-z][a-z_.]{0,12}".prop_map(Id::new)
}

/// A deliberately **tiny** id alphabet, for exactly the reason [`arb_name`] is
/// tiny. Most of the lists `Entity::normalize` sorts are keyed by an [`Id`]
/// first (`ability_scores` by ability, `equipment` by item, …), so a comparator
/// that looked *only* at that id would be order-dependent precisely when two
/// rows share one — and the permutation half of the canonical-serialization
/// property can only catch that if duplicate ids are actually generated. The
/// broad [`arb_id`] above has far too large a space to collide.
fn arb_small_id() -> impl Strategy<Value = Id> {
    "[ab]{1,2}".prop_map(Id::new)
}

/// A deliberately **tiny** free-text alphabet, so duplicate names occur constantly.
/// `Entity::normalize` must sort the nested lists by a *total* order: a comparator
/// that looked only at a row's name would be order-dependent exactly when two rows
/// share one, and the permutation half of the canonical-serialization property below
/// can only see that if duplicates actually get generated.
fn arb_name() -> impl Strategy<Value = String> {
    "[ab]{1,2}"
}

/// Arbitrary prerequisite expression tree, bounded in depth and breadth.
fn arb_prereq() -> impl Strategy<Value = Prereq> {
    let leaf = prop_oneof![
        arb_id().prop_map(Prereq::Has),
        arb_id().prop_map(Prereq::House),
        (arb_id(), any::<u8>()).prop_map(|(ability, score)| Prereq::AbilityMin { ability, score }),
        (arb_id(), any::<u8>()).prop_map(|(art, score)| Prereq::ArtMin { art, score }),
        Just(Prereq::IsMagus),
    ];
    leaf.prop_recursive(3, 16, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Prereq::All),
            prop::collection::vec(inner.clone(), 0..4).prop_map(Prereq::Any),
            prop::collection::vec(inner, 0..4).prop_map(Prereq::Nor),
        ]
    })
}

fn arb_selection() -> impl Strategy<Value = Selection> {
    (
        arb_id(),
        prop::collection::btree_map("[a-z]{1,6}", arb_id(), 0..3),
    )
        .prop_map(|(item_ref, params)| Selection::with_params(item_ref, params))
}

/// A supernatural power, shared by [`Entity::powers`] and the familiar's invested
/// powers — the same struct sorted by the same derived comparator in both places.
fn arb_power() -> impl Strategy<Value = SupernaturalPower> {
    (arb_name(), 0u16..4, 0u16..4).prop_map(|(name, level, penetration)| SupernaturalPower {
        name,
        level,
        penetration,
    })
}

fn arb_ability_score() -> impl Strategy<Value = AbilityScore> {
    (
        arb_small_id(),
        0u8..4,
        prop::option::of(arb_name()),
        prop::option::of(arb_name()),
    )
        .prop_map(|(ability, score, specialty, parameter)| AbilityScore {
            ability,
            score,
            specialty,
            parameter,
        })
}

fn arb_art_score() -> impl Strategy<Value = ArtScore> {
    (arb_small_id(), 0u8..4).prop_map(|(art, score)| ArtScore { art, score })
}

/// A spell selection including its own nested `mastery_abilities` list — the
/// thirteenth list `normalize()` sorts, and the only one nested inside another
/// list it also sorts.
fn arb_spell() -> impl Strategy<Value = SpellSelection> {
    (
        arb_small_id(),
        prop::option::of(0u8..4),
        prop::option::of(0u8..4),
        prop::option::of(arb_name()),
        prop::collection::vec(arb_small_id(), 0..4),
    )
        .prop_map(
            |(spell, level, mastery, parameter, mastery_abilities)| SpellSelection {
                spell,
                level,
                mastery,
                parameter,
                mastery_abilities,
            },
        )
}

fn arb_personality_trait() -> impl Strategy<Value = PersonalityTrait> {
    (arb_name(), -3i8..4).prop_map(|(name, value)| PersonalityTrait { name, value })
}

fn arb_reputation() -> impl Strategy<Value = Reputation> {
    (
        prop_oneof![
            Just(ReputationType::Local),
            Just(ReputationType::Ecclesiastical),
            Just(ReputationType::Hermetic),
            Just(ReputationType::Academic),
        ],
        0u8..4,
        arb_name(),
    )
        .prop_map(|(kind, score, content)| Reputation {
            kind,
            score,
            content,
        })
}

fn arb_device() -> impl Strategy<Value = EnchantedDevice> {
    (arb_name(), 0u16..4).prop_map(|(name, level)| EnchantedDevice { name, level })
}

fn arb_twilight_scar() -> impl Strategy<Value = TwilightScar> {
    arb_name().prop_map(|description| TwilightScar { description })
}

fn arb_equipment_slot() -> impl Strategy<Value = EquipmentSlot> {
    (arb_small_id(), any::<bool>(), any::<bool>()).prop_map(
        |(item, equipped, specialization_applies)| EquipmentSlot {
            item,
            equipped,
            specialization_applies,
        },
    )
}

fn arb_focus_power() -> impl Strategy<Value = FocusPower> {
    (arb_name(), 0u16..4, 0u16..4).prop_map(|(name, max_level, penetration)| FocusPower {
        name,
        max_level,
        penetration,
    })
}

/// An aging-log row. `year` is drawn from a deliberately narrow span so rows
/// sharing one are common: `AgingLogEntry`'s derived `Ord` puts `year` first, so
/// a comparator keyed on the year alone is order-dependent exactly there.
fn arb_aging_log_entry() -> impl Strategy<Value = AgingLogEntry> {
    (
        prop::option::of(1220i32..1223),
        prop::option::of(30u32..33),
        arb_name(),
        prop::option::of(-2i32..11),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(year, age, effect, die, apparent_age_increased, crisis)| AgingLogEntry {
                year,
                age,
                effect,
                die,
                apparent_age_increased,
                crisis,
                ..AgingLogEntry::default()
            },
        )
}

/// A talisman with arbitrary-length attunement and effect lists (M5.5b's two nested
/// lists), over [`arb_name`] so rows sharing a description/name are common.
fn arb_talisman() -> impl Strategy<Value = Talisman> {
    (
        arb_name(),
        prop::collection::vec(
            (arb_name(), -5i8..6)
                .prop_map(|(description, bonus)| TalismanAttunement { description, bonus }),
            0..5,
        ),
        prop::collection::vec(
            (arb_name(), 0u16..4).prop_map(|(name, level)| TalismanEffect { name, level }),
            0..5,
        ),
    )
        .prop_map(|(description, attunements, effects)| Talisman {
            description,
            attunements,
            effects,
        })
}

/// A familiar statblock (M5.5c): the Characteristics map plus the two nested lists
/// `Familiar::normalize` sorts (Personality Traits, invested powers).
fn arb_familiar() -> impl Strategy<Value = Familiar> {
    (
        arb_name(),
        arb_name(),
        prop::option::of(
            (
                prop_oneof![
                    Just(Realm::Magic),
                    Just(Realm::Faerie),
                    Just(Realm::Divine),
                    Just(Realm::Infernal)
                ],
                0u8..15,
            )
                .prop_map(|(realm, score)| MightScore { realm, score }),
        ),
        prop::collection::btree_map(
            prop_oneof![
                Just(Characteristic::Int),
                Just(Characteristic::Sta),
                Just(Characteristic::Qik)
            ],
            -3i8..4,
            0..3,
        ),
        -5i8..3,
        prop::collection::vec(arb_personality_trait(), 0..5),
        (0u8..6, 0u8..6, 0u8..6),
        prop::collection::vec(arb_power(), 0..5),
    )
        .prop_map(
            |(
                name,
                animal,
                might,
                characteristics,
                size,
                personality_traits,
                (cord_gold, cord_silver, cord_bronze),
                powers,
            )| Familiar {
                name,
                animal,
                might,
                characteristics,
                size,
                personality_traits,
                cord_gold,
                cord_silver,
                cord_bronze,
                powers,
            },
        )
}

/// A wizard-progress slug, deliberately **not** drawn from `CreationPhase::ALL`:
/// the field stores a raw slug precisely so a vocabulary the build no longer speaks
/// still round-trips (see `a_save_with_an_unknown_wizard_phase_slug_still_loads`).
/// `None` is generated too, which is the shape that must add no key at all.
fn arb_wizard_phase() -> impl Strategy<Value = Option<String>> {
    prop::option::of(prop_oneof![
        Just("abilities".to_string()),
        Just("review".to_string()),
        Just("type".to_string()),
        Just("not_a_phase".to_string()),
    ])
}

/// Every top-level list `Entity::normalize` sorts apart from `selections`,
/// bundled into one value purely so [`arb_entity`] stays inside proptest's
/// tuple-arity limit. Each is generated over a small value space (see
/// [`arb_small_id`] and [`arb_name`]) so duplicate sort keys are common.
#[derive(Debug, Clone)]
struct SortedLists {
    ability_scores: Vec<AbilityScore>,
    art_scores: Vec<ArtScore>,
    spells: Vec<SpellSelection>,
    personality_traits: Vec<PersonalityTrait>,
    reputations: Vec<Reputation>,
    devices: Vec<EnchantedDevice>,
    twilight_scars: Vec<TwilightScar>,
    aging_log: Vec<AgingLogEntry>,
    equipment: Vec<EquipmentSlot>,
    powers: Vec<SupernaturalPower>,
    focus_powers: Vec<FocusPower>,
}

impl SortedLists {
    fn install(self, entity: &mut Entity) {
        entity.ability_scores = self.ability_scores;
        entity.art_scores = self.art_scores;
        entity.spells = self.spells;
        entity.personality_traits = self.personality_traits;
        entity.reputations = self.reputations;
        entity.devices = self.devices;
        entity.twilight_scars = self.twilight_scars;
        entity.aging_log = self.aging_log;
        entity.equipment = self.equipment;
        entity.powers = self.powers;
        entity.focus_powers = self.focus_powers;
    }

    /// Reverses each list in place — the permutation the canonical-serialization
    /// property re-normalizes away.
    fn reverse_each(entity: &mut Entity) {
        entity.ability_scores.reverse();
        entity.art_scores.reverse();
        for spell in &mut entity.spells {
            spell.mastery_abilities.reverse();
        }
        entity.spells.reverse();
        entity.personality_traits.reverse();
        entity.reputations.reverse();
        entity.devices.reverse();
        entity.twilight_scars.reverse();
        entity.aging_log.reverse();
        entity.equipment.reverse();
        entity.powers.reverse();
        entity.focus_powers.reverse();
    }
}

fn arb_sorted_lists() -> impl Strategy<Value = SortedLists> {
    (
        prop::collection::vec(arb_ability_score(), 0..4),
        prop::collection::vec(arb_art_score(), 0..4),
        prop::collection::vec(arb_spell(), 0..4),
        prop::collection::vec(arb_personality_trait(), 0..4),
        prop::collection::vec(arb_reputation(), 0..4),
        prop::collection::vec(arb_device(), 0..4),
        prop::collection::vec(arb_twilight_scar(), 0..4),
        prop::collection::vec(arb_aging_log_entry(), 0..4),
        prop::collection::vec(arb_equipment_slot(), 0..4),
        prop::collection::vec(arb_power(), 0..4),
        prop::collection::vec(arb_focus_power(), 0..4),
    )
        .prop_map(
            |(
                ability_scores,
                art_scores,
                spells,
                personality_traits,
                reputations,
                devices,
                twilight_scars,
                aging_log,
                equipment,
                powers,
                focus_powers,
            )| SortedLists {
                ability_scores,
                art_scores,
                spells,
                personality_traits,
                reputations,
                devices,
                twilight_scars,
                aging_log,
                equipment,
                powers,
                focus_powers,
            },
        )
}

fn arb_entity() -> impl Strategy<Value = Entity> {
    (
        any::<u32>(),
        arb_id(),
        prop_oneof![Just(EntityKind::Character), Just(EntityKind::Covenant)],
        arb_id(),
        prop::collection::vec(arb_selection(), 0..6),
        prop::option::of(arb_talisman()),
        prop::option::of(arb_familiar()),
        prop_oneof![Just(AbilityFunding::Pool), Just(AbilityFunding::LifeStages)],
        arb_wizard_phase(),
        arb_sorted_lists(),
        // Deliberately generated outside the legal aura band as well, so the
        // clamp `normalize()` applies is pinned as idempotent: re-normalizing an
        // already-normalized entity must not move it again.
        -30i32..31,
    )
        .prop_map(
            |(
                schema_version,
                rs_id,
                entity_kind,
                type_id,
                selections,
                talisman,
                familiar,
                ability_funding,
                wizard_furthest_phase,
                sorted_lists,
                aura,
            )| {
                let mut entity = Entity::new(entity_kind, type_id, RulesetRef::new(rs_id, "1"));
                entity.schema_version = schema_version;
                entity.selections = selections;
                entity.talisman = talisman;
                entity.familiar = familiar;
                entity.ability_funding = ability_funding;
                entity.wizard_furthest_phase = wizard_furthest_phase;
                entity.aura = aura;
                sorted_lists.install(&mut entity);
                entity.normalize();
                entity
            },
        )
}

/// Reverses every list `Entity::normalize` is responsible for sorting, so
/// re-normalizing has to put them all back in the same canonical order.
fn reverse_every_sorted_list(entity: &mut Entity) {
    entity.selections.reverse();
    SortedLists::reverse_each(entity);
    if let Some(talisman) = &mut entity.talisman {
        talisman.attunements.reverse();
        talisman.effects.reverse();
    }
    if let Some(familiar) = &mut entity.familiar {
        familiar.personality_traits.reverse();
        familiar.powers.reverse();
    }
}

proptest! {
    /// Every adjacently-tagged `Prereq` tree survives a serialize -> deserialize
    /// round trip unchanged.
    #[test]
    fn prereq_roundtrips(prereq in arb_prereq()) {
        let json = serde_json::to_string(&prereq).unwrap();
        let back: Prereq = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(prereq, back);
    }

    /// Arbitrary entities round-trip, and canonical serialization is invariant under
    /// input order: permuting **every** list `Entity::normalize` sorts, then
    /// re-normalizing, is byte-stable.
    ///
    /// That is all thirteen of them — the twelve top-level lists (`selections`,
    /// `ability_scores`, `art_scores`, `spells`, `personality_traits`,
    /// `reputations`, `devices`, `twilight_scars`, `aging_log`, `equipment`,
    /// `powers`, `focus_powers`) plus each spell's nested `mastery_abilities` —
    /// and the two nested structs that normalize themselves, the talisman
    /// (attunements, instilled effects) and the familiar statblock (Personality
    /// Traits, invested powers). It therefore pins every comparator `normalize()`
    /// relies on as a *total* order: an id- or name-only comparator would flip two
    /// rows sharing that key and fail here, which is why the generators above draw
    /// ids and names from deliberately tiny alphabets.
    ///
    /// Keep this list in step with `types.rs::Entity::normalize` — a list added
    /// there and not here is a comparator nothing pins.
    #[test]
    fn entity_roundtrips_and_serializes_canonically(entity in arb_entity()) {
        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&entity, &back);

        // Schema 16's two fields, byte-level. `ability_funding` is always written —
        // absence is what the migration dispatches on, so omitting the default would
        // flip a pool-funded character that keeps a plan back to life-stage funding.
        // `wizard_furthest_phase` is the opposite: skip-if-none, so a character that
        // never entered the wizard adds no key.
        prop_assert!(json.contains(r#""ability_funding":"#), "{}", json);
        prop_assert_eq!(
            json.contains("wizard_furthest_phase"),
            entity.wizard_furthest_phase.is_some(),
            "{}", json
        );

        let mut permuted = entity.clone();
        reverse_every_sorted_list(&mut permuted);
        permuted.normalize();
        prop_assert_eq!(
            serde_json::to_string(&entity).unwrap(),
            serde_json::to_string(&permuted).unwrap()
        );
    }
}
