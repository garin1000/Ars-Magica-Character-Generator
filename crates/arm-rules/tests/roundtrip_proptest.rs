//! Property-based serialization tests. Complements the hand-picked example
//! roundtrips in the unit modules by fuzzing arbitrary `Prereq` trees and
//! `Entity` values through serialize -> deserialize, asserting idempotence and
//! canonical (input-order-independent) output.

use arm_rules::{Entity, EntityKind, Id, Prereq, RulesetRef, Selection};
use proptest::prelude::*;

fn arb_id() -> impl Strategy<Value = Id> {
    "[a-z][a-z_.]{0,12}".prop_map(Id::new)
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

fn arb_entity() -> impl Strategy<Value = Entity> {
    (
        any::<u32>(),
        arb_id(),
        prop_oneof![Just(EntityKind::Character), Just(EntityKind::Covenant)],
        arb_id(),
        prop::collection::vec(arb_selection(), 0..6),
    )
        .prop_map(
            |(schema_version, rs_id, entity_kind, type_id, selections)| {
                let mut entity = Entity::new(entity_kind, type_id, RulesetRef::new(rs_id, "1"));
                entity.schema_version = schema_version;
                entity.selections = selections;
                entity.normalize();
                entity
            },
        )
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

    /// Arbitrary entities round-trip, and canonical serialization is invariant
    /// under selection input order (permuting then re-normalizing is byte-stable).
    #[test]
    fn entity_roundtrips_and_serializes_canonically(entity in arb_entity()) {
        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&entity, &back);

        let mut permuted = entity.clone();
        permuted.selections.reverse();
        permuted.normalize();
        prop_assert_eq!(
            serde_json::to_string(&entity).unwrap(),
            serde_json::to_string(&permuted).unwrap()
        );
    }
}
