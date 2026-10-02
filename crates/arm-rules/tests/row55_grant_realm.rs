//! Row 55 (`docs/open-todos.md`; D74.4, `docs/vf-audit/decisions.md`) — tests
//! for the per-grant Supernatural realm override.
//!
//! D42/D70/D74 gave every Supernatural entry a resolved realm
//! (`effective::resolve_realm`); row 55 extends that to a GRANTED copy, which
//! otherwise carried no realm of its own — the grant machinery
//! (`grant::resolve_grant`, `effective::vf_granted_selections`) now reads
//! `grant::Grant::Fixed::realm` / `types::Effect::GrantsSelection::realm` and
//! stamps it onto the granted `Selection`'s own `association` param
//! (`effective::stamp_realm_override`), so `resolve_realm` itself needed no
//! change:
//!
//! - Strong Faerie Blood's Second Sight is stated "faerie eyes" (ArMDE:5038).
//! - Faerie Doctor's Dowsing and Spirit Votary's Second Sight are each the
//!   mythic type's own realm (ArMDE:2641: "The following types of Mythic
//!   Companions cover all the supernatural realms" — Faerie Doctor is Faerie,
//!   Spirit Votary is Magic, by the same naming the book uses for Devil Child
//!   = Infernal and Nephilim = Divine).
//!
//! This file started as phase 1's RED checkpoint (every test below that
//! asserts the stated realm failed before the stamp was wired and the three
//! JSON sites were authored); each test's own comment still notes which
//! phase-1 state it replaced, for the historical record.

use arm_rules::effective::{entity_grants, resolve_realm};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;

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
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.age = Some(25);
    e.selections = selections;
    e
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

/// Finds the granted copy of `item_ref` among `entity_grants`' output — the
/// position is not stable (the project's own warning: "the same `ref` can
/// arrive twice"), but every scenario here grants each target item exactly
/// once, so the first match is unambiguous.
fn granted<'a>(granted: &'a [Selection], item_ref: &str) -> &'a Selection {
    granted
        .iter()
        .find(|s| s.item_ref == Id::new(item_ref))
        .unwrap_or_else(|| panic!("{item_ref} must be among the granted selections: {granted:?}"))
}

// --- Strong Faerie Blood's Second Sight is stated Faerie (ArMDE:5038) ------

mod strong_faerie_blood {
    use super::*;

    /// Was RED in phase 1: Strong Faerie Blood's `GrantsSelection` effect now
    /// carries `realm: Some(Realm::Faerie)` in the shipped data, and
    /// `vf_granted_selections` stamps it onto the granted copy, so it
    /// resolves Faerie instead of falling through the plain chain to Magic.
    #[test]
    fn granted_second_sight_resolves_faerie() {
        let rs = full_ruleset();
        let e = entity("companion", vec![sel("virtue.strong_faerie_blood")]);
        let grants = entity_grants(&e, &rs);
        let second_sight = granted(&grants, "virtue.second_sight");
        let item = rs.item(&Id::new("virtue.second_sight")).unwrap();
        let resolved = resolve_realm(item, second_sight, None);
        assert_eq!(
            resolved.realm,
            Realm::Faerie,
            "ArMDE:5038 'faerie eyes': {resolved:?}"
        );
    }

    /// Was RED alongside the above: a concept realm must NOT override the stated
    /// per-grant Faerie association (D74.4's "only where stated" is a fixed
    /// fact about THIS grant, not a default the concept can displace — the
    /// granted item's own `realm_association` is itself `None`, so without
    /// the per-grant stamp this test would show the concept realm winning
    /// instead, which is also wrong).
    #[test]
    fn granted_second_sight_faerie_is_not_displaced_by_concept_realm() {
        let rs = full_ruleset();
        let e = entity("companion", vec![sel("virtue.strong_faerie_blood")]);
        let grants = entity_grants(&e, &rs);
        let second_sight = granted(&grants, "virtue.second_sight");
        let item = rs.item(&Id::new("virtue.second_sight")).unwrap();
        let resolved = resolve_realm(item, second_sight, Some(Realm::Divine));
        assert_eq!(
            resolved.realm,
            Realm::Faerie,
            "the stated grant realm must beat a Divine concept: {resolved:?}"
        );
    }

    /// Phase 2: Strong Faerie Blood's `GrantsSelection` effect now authors
    /// `realm: Some(Faerie)` in the shipped data (ArMDE:5038) — the data-level
    /// counterpart to the resolver behavior tested above.
    #[test]
    fn shipped_grant_carries_the_stated_realm() {
        let rs = full_ruleset();
        let item = rs.item(&Id::new("virtue.strong_faerie_blood")).unwrap();
        let found = item
            .effects
            .iter()
            .find_map(|effect| match effect {
                Effect::GrantsSelection { items, realm }
                    if items.contains(&Id::new("virtue.second_sight")) =>
                {
                    Some(*realm)
                }
                _ => None,
            })
            .expect("Strong Faerie Blood must grant Second Sight");
        assert_eq!(
            found,
            Some(Realm::Faerie),
            "ArMDE:5038 'faerie eyes' must be authored on the grant"
        );
    }
}

// --- Faerie Doctor / Spirit Votary grant their mythic type's realm --------
// (ArMDE:2641 — Devil Child/Infernal, Faerie Doctor/Faerie, Nephilim/Divine,
// Spirit Votary/Magic; only the latter two are exercised here, matching the
// todo's examples).

mod mythic_type_realm {
    use super::*;

    fn mythic_companion(type_id: &str) -> Entity {
        let mut e = entity("mythic_companion", vec![]);
        e.mythic_type = Some(Id::new(type_id));
        e
    }

    /// Was RED in phase 1: Faerie Doctor's direct `grant::Grant::Fixed` grant
    /// of Dowsing now carries `realm: Some(Realm::Faerie)`, and
    /// `resolve_grant` stamps it, so the granted Dowsing resolves Faerie
    /// instead of falling through to Magic.
    #[test]
    fn faerie_doctor_grants_dowsing_in_faerie() {
        let rs = full_ruleset();
        let e = mythic_companion("mythic_type.faerie_doctor");
        let grants = entity_grants(&e, &rs);
        let dowsing = granted(&grants, "virtue.dowsing");
        let item = rs.item(&Id::new("virtue.dowsing")).unwrap();
        let resolved = resolve_realm(item, dowsing, None);
        assert_eq!(
            resolved.realm,
            Realm::Faerie,
            "ArMDE:2641 — Faerie Doctor is the Faerie-covering type: {resolved:?}"
        );
    }

    /// Already green even in phase 1: Spirit Votary's granted Second Sight
    /// resolves Magic either way, since the stub's plain-chain fallback
    /// (Magic, no concept_realm set) coincides with Spirit Votary's real
    /// target realm — exactly like `d42_realms.rs`'s
    /// `magic_is_the_final_fallback` note. The
    /// `..._is_not_displaced_by_concept_realm` test below is what actually
    /// discriminates stub from real wiring for this family. Kept anyway as
    /// the locked acceptance criterion for Spirit Votary, so a later
    /// unrelated change to the plain-chain fallback cannot silently make
    /// THIS assertion wrong for the wrong reason.
    #[test]
    fn spirit_votary_grants_second_sight_in_magic() {
        let rs = full_ruleset();
        let e = mythic_companion("mythic_type.spirit_votary");
        let grants = entity_grants(&e, &rs);
        let second_sight = granted(&grants, "virtue.second_sight");
        let item = rs.item(&Id::new("virtue.second_sight")).unwrap();
        let resolved = resolve_realm(item, second_sight, None);
        assert_eq!(
            resolved.realm,
            Realm::Magic,
            "ArMDE:2641 — Spirit Votary is the Magic-covering type: {resolved:?}"
        );
    }

    /// Was RED in phase 1 (the real discriminator for Spirit Votary): a
    /// Faerie concept realm must NOT leak into the stated Magic grant now
    /// that the stamp is wired. In phase 1 (stub) the concept realm DID win,
    /// through the plain chain — this is the assertion that actually
    /// distinguished the two.
    #[test]
    fn spirit_votary_second_sight_magic_is_not_displaced_by_concept_realm() {
        let rs = full_ruleset();
        let e = mythic_companion("mythic_type.spirit_votary");
        let grants = entity_grants(&e, &rs);
        let second_sight = granted(&grants, "virtue.second_sight");
        let item = rs.item(&Id::new("virtue.second_sight")).unwrap();
        let resolved = resolve_realm(item, second_sight, Some(Realm::Faerie));
        assert_eq!(
            resolved.realm,
            Realm::Magic,
            "the stated grant realm must beat a Faerie concept: {resolved:?}"
        );
    }

    /// Phase 2: both types' shipped `Grant::Fixed` entries now author the
    /// type's own realm (ArMDE:2641) directly on the grant.
    #[test]
    fn shipped_grants_carry_the_types_realm() {
        let rs = full_ruleset();
        for (type_id, target, realm) in [
            ("mythic_type.faerie_doctor", "virtue.dowsing", Realm::Faerie),
            (
                "mythic_type.spirit_votary",
                "virtue.second_sight",
                Realm::Magic,
            ),
        ] {
            let mtype = rs.mythic_type(&Id::new(type_id)).unwrap();
            let found = mtype
                .grants
                .iter()
                .find_map(|g| match g {
                    arm_rules::Grant::Fixed {
                        item, realm: found, ..
                    } if *item == Id::new(target) => Some(*found),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{type_id} must grant {target}"));
            assert_eq!(found, Some(realm), "{type_id} must author its own realm");
        }
    }
}

// --- A grant with no stated realm is unaffected (non-Supernatural items) --

mod unaffected_grants {
    use super::*;

    /// Already green, and must stay so: Templar Commander grants Brother-
    /// Knight and Temporal Influence (ArMDE:5113-5116), neither Supernatural
    /// — `item_has_realm_association` is false for both, so row 55 has
    /// nothing to change here. Locks the "no behavior change outside the
    /// Supernatural grants row 55 names" boundary.
    #[test]
    fn templar_commander_grants_carry_no_realm_association() {
        let rs = full_ruleset();
        let e = entity("companion", vec![sel("virtue.templar_commander")]);
        let grants = entity_grants(&e, &rs);
        for target in ["virtue.brother_knight", "virtue.temporal_influence"] {
            let selection = granted(&grants, target);
            let item = rs.item(&selection.item_ref).unwrap();
            assert!(
                !arm_rules::effective::item_has_realm_association(item, selection),
                "{target} must not be realm-eligible"
            );
        }
    }
}

// --- A grant's realm must be a valid realm id (load-time integrity) -------

mod grant_realm_integrity {
    use super::*;

    const COMPANION_TYPE: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 20, "flaw_points": 20 },
        "permitted_categories": ["general", "supernatural"],
        "creation_phases": [] }
    ]"#;

    const FILLER_ITEMS: &str = r#"[
      { "id": "virtue.test_granted", "kind": "virtue", "magnitude": "minor",
        "categories": ["supernatural"], "classification": "narrative" },
      { "id": "virtue.test_granter", "kind": "virtue", "magnitude": "minor",
        "categories": ["general"], "classification": "narrative",
        "effects": [{ "type": "grants_selection", "items": ["virtue.test_granted"] }] },
      { "id": "flaw.filler_personality", "kind": "flaw", "magnitude": "minor",
        "categories": ["personality"], "classification": "narrative" }
    ]"#;

    /// Already green: `Effect::GrantsSelection::realm` is a typed [`Realm`]
    /// (mirroring `Effect::MightGrant::realm`), so serde itself is the load-
    /// time integrity gate — a stated value outside the four realm ids fails
    /// to parse at all, with a message naming the bad value, rather than
    /// silently loading and resolving wrong.
    #[test]
    fn grants_selection_rejects_an_invalid_realm_id_at_load() {
        let point_items = FILLER_ITEMS.replacen(
            "\"grants_selection\", \"items\": [\"virtue.test_granted\"] }",
            "\"grants_selection\", \"items\": [\"virtue.test_granted\"], \"realm\": \"bogus\" }",
            1,
        );
        let err = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: &point_items,
            type_profiles: COMPANION_TYPE,
            ..RulesetSources::default()
        })
        .unwrap_err();
        assert!(
            format!("{err}").to_lowercase().contains("bogus"),
            "got: {err}"
        );
    }

    const MYTHIC_TYPE_WITH_BAD_REALM: &str = r#"{ "types": [
        { "id": "mythic_type.test_bad_realm",
          "grants": [ { "kind": "fixed", "item": "virtue.test_granted", "realm": "bogus" } ] }
    ] }"#;

    /// Already green, the `grant::Grant::Fixed` sibling of the test above.
    #[test]
    fn grant_fixed_rejects_an_invalid_realm_id_at_load() {
        let err = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: FILLER_ITEMS,
            type_profiles: COMPANION_TYPE,
            mythic_types: Some(MYTHIC_TYPE_WITH_BAD_REALM),
            ..RulesetSources::default()
        })
        .unwrap_err();
        assert!(
            format!("{err}").to_lowercase().contains("bogus"),
            "got: {err}"
        );
    }
}

// --- A BOUGHT Second Sight is unaffected — the plain chain, untouched -----

mod bought_selection_unaffected {
    use super::*;

    /// Already green, and must stay so: row 55 only changes how a GRANTED
    /// copy resolves. A bought Second Sight still has no `realm_association`
    /// of its own and still follows override → concept → Magic exactly as
    /// before.
    #[test]
    fn bought_second_sight_still_follows_the_plain_chain() {
        let rs = full_ruleset();
        let e = entity("companion", vec![sel("virtue.second_sight")]);
        let item = rs.item(&Id::new("virtue.second_sight")).unwrap();
        let resolved = resolve_realm(item, &e.selections[0], Some(Realm::Divine));
        assert_eq!(
            resolved.realm,
            Realm::Divine,
            "a bought Second Sight must still follow the concept realm"
        );
    }
}
