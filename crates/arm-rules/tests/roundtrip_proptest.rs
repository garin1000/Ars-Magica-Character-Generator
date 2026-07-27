//! Property-based serialization tests. Complements the hand-picked example
//! roundtrips in the unit modules by fuzzing arbitrary `Prereq` trees and
//! `Entity` values through serialize -> deserialize, asserting idempotence and
//! canonical (input-order-independent) output.

use arm_rules::{
    Characteristic, Entity, EntityKind, Familiar, Id, MightScore, PersonalityTrait, Prereq, Realm,
    RulesetRef, Selection, SupernaturalPower, Talisman, TalismanAttunement, TalismanEffect,
};
use proptest::prelude::*;

fn arb_id() -> impl Strategy<Value = Id> {
    "[a-z][a-z_.]{0,12}".prop_map(Id::new)
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
        prop::collection::vec(
            (arb_name(), -3i8..4).prop_map(|(name, value)| PersonalityTrait { name, value }),
            0..5,
        ),
        (0u8..6, 0u8..6, 0u8..6),
        prop::collection::vec(
            (arb_name(), 0u16..4).prop_map(|(name, level)| SupernaturalPower { name, level }),
            0..5,
        ),
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

fn arb_entity() -> impl Strategy<Value = Entity> {
    (
        any::<u32>(),
        arb_id(),
        prop_oneof![Just(EntityKind::Character), Just(EntityKind::Covenant)],
        arb_id(),
        prop::collection::vec(arb_selection(), 0..6),
        prop::option::of(arb_talisman()),
        prop::option::of(arb_familiar()),
    )
        .prop_map(
            |(schema_version, rs_id, entity_kind, type_id, selections, talisman, familiar)| {
                let mut entity = Entity::new(entity_kind, type_id, RulesetRef::new(rs_id, "1"));
                entity.schema_version = schema_version;
                entity.selections = selections;
                entity.talisman = talisman;
                entity.familiar = familiar;
                entity.normalize();
                entity
            },
        )
}

/// Reverses every list `Entity::normalize` is responsible for sorting, so
/// re-normalizing has to put them all back in the same canonical order.
fn reverse_every_sorted_list(entity: &mut Entity) {
    entity.selections.reverse();
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
    /// input order: permuting the selections **and** the nested lists of the talisman
    /// (attunements, instilled effects) and the familiar statblock (Personality
    /// Traits, invested powers), then re-normalizing, is byte-stable. That pins every
    /// comparator `normalize()` relies on as a *total* order — a name-only comparator
    /// would flip two rows sharing a name and fail here.
    #[test]
    fn entity_roundtrips_and_serializes_canonically(entity in arb_entity()) {
        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&entity, &back);

        let mut permuted = entity.clone();
        reverse_every_sorted_list(&mut permuted);
        permuted.normalize();
        prop_assert_eq!(
            serde_json::to_string(&entity).unwrap(),
            serde_json::to_string(&permuted).unwrap()
        );
    }
}
