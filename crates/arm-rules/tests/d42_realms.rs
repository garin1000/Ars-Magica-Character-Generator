//! D42/D70/D74 (`docs/vf-audit/decisions.md`; `tmp/d42-handover.md`) — RED-
//! checkpoint phase 1 tests for the per-entry Supernatural realm association.
//!
//! Runs against the REAL shipped ruleset (`rules/core/virtues_flaws.json`)
//! wherever an id is enough to exercise the behavior; a handful of tests need
//! a `realm_association` shape phase 2 has not yet authored onto the shipped
//! data, so they clone the shipped item and attach the association phase 2 is
//! expected to add (`with_association` below) rather than inventing a
//! fixture unrelated to the real catalogue. Two tests (Manifest Sin's
//! override-outside-subset error, and the "no error" shape for a fixed
//! entry's override) use a small in-test ruleset because they exercise
//! `validate()`'s wiring rather than any single item's data.
//!
//! [`arm_rules::effective::resolve_realm`] is a deliberate phase-1 stub
//! (always `Realm::Magic`, no warning) and `validate_realm_associations`
//! emits nothing yet, so most tests here are RED on their assertion. A few
//! are already green — each such test says so in its own comment — because
//! they lock an invariant phase 1's real (non-stub) wiring already provides
//! (the `association` key's missing/unexpected-param exemption) or an
//! infrastructure fact (the field is additive, the Subset shape loads).

use std::collections::BTreeMap;

use arm_rules::effective::{REALM_OVERRIDE_PARAM_KEY, RealmWarning, resolve_realm};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationIssue, validate};

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

fn sel_with_override(id: &str, realm: Realm) -> Selection {
    Selection::with_params(
        Id::new(id),
        BTreeMap::from([(REALM_OVERRIDE_PARAM_KEY.to_string(), realm.id())]),
    )
}

fn has_code(issues: &[ValidationIssue], code: &str) -> bool {
    issues.iter().any(|i| i.code == code)
}

/// Clones a shipped item and attaches the `realm_association` phase 2 is
/// expected to author for it (D70/D74), so resolver logic can be exercised
/// against the real item's other fields ahead of that data landing.
fn with_association(rs: &Ruleset, id: &str, assoc: RealmAssociation) -> PointItem {
    let mut item = rs
        .item(&Id::new(id))
        .unwrap_or_else(|| panic!("shipped item {id} not found"))
        .clone();
    item.realm_association = Some(assoc);
    item
}

// --- Resolution chain: override > concept > Magic (D70 Q-X6-5) -------------

mod resolution_chain {
    use super::*;

    #[test]
    fn override_beats_concept_realm() {
        let rs = full_ruleset();
        let item = rs.item(&Id::new("virtue.animal_ken")).unwrap(); // free (no association)
        let selection = sel_with_override("virtue.animal_ken", Realm::Divine);
        let resolved = resolve_realm(item, &selection, Some(Realm::Faerie));
        assert_eq!(resolved.realm, Realm::Divine);
        assert_eq!(resolved.warning, None);
    }

    #[test]
    fn concept_realm_beats_magic_fallback_when_no_override() {
        let rs = full_ruleset();
        let item = rs.item(&Id::new("virtue.animal_ken")).unwrap();
        let selection = sel("virtue.animal_ken");
        let resolved = resolve_realm(item, &selection, Some(Realm::Faerie));
        assert_eq!(resolved.realm, Realm::Faerie);
    }

    /// Already green: the stub's dumb default coincides with the real one
    /// here, since Magic IS the final fallback with no override and no
    /// concept (ArMDE:2960). Not diagnostic on its own — see the two tests
    /// above for the cases that actually distinguish the stub from D70.
    #[test]
    fn magic_is_the_final_fallback() {
        let rs = full_ruleset();
        let item = rs.item(&Id::new("virtue.animal_ken")).unwrap();
        let selection = sel("virtue.animal_ken");
        let resolved = resolve_realm(item, &selection, None);
        assert_eq!(resolved.realm, Realm::Magic);
        assert_eq!(resolved.warning, None);
    }
}

// --- Fixed realms: the book states them outright ---------------------------

mod fixed_realms {
    use super::*;

    /// Faerie Blood / Strong Faerie Blood: the two examples ArMDE:2960 itself
    /// names. A concept realm and an override must not move them.
    #[test]
    fn faerie_bloods_are_fixed_faerie() {
        let rs = full_ruleset();
        for id in ["virtue.faerie_blood", "virtue.strong_faerie_blood"] {
            let item = with_association(
                &rs,
                id,
                RealmAssociation::Fixed {
                    realm: Realm::Faerie,
                },
            );
            let selection = sel_with_override(id, Realm::Infernal);
            let resolved = resolve_realm(&item, &selection, Some(Realm::Divine));
            assert_eq!(resolved.realm, Realm::Faerie, "{id} must stay Faerie");
            assert_eq!(
                resolved.warning, None,
                "{id} override is ignored, not warned"
            );
        }
    }

    #[test]
    fn kassalan_exorcism_is_fixed_magic() {
        let rs = full_ruleset();
        let item = with_association(
            &rs,
            "virtue.kassalan_exorcism",
            RealmAssociation::Fixed {
                realm: Realm::Magic,
            },
        );
        let selection = sel("virtue.kassalan_exorcism");
        let resolved = resolve_realm(&item, &selection, Some(Realm::Divine));
        assert_eq!(resolved.realm, Realm::Magic);
    }

    /// D70 Q-X6-6 + D74: Strong Angelic Heritage, Blood of the Nephilim (new
    /// in D74) and Viaticarus (new in D74) are fixed Divine.
    #[test]
    fn divine_fixed_entries() {
        let rs = full_ruleset();
        for id in [
            "virtue.strong_angelic_heritage",
            "virtue.blood_of_the_nephilim",
            "flaw.viaticarus",
        ] {
            let item = with_association(
                &rs,
                id,
                RealmAssociation::Fixed {
                    realm: Realm::Divine,
                },
            );
            let selection = sel(id);
            let resolved = resolve_realm(&item, &selection, Some(Realm::Faerie));
            assert_eq!(resolved.realm, Realm::Divine, "{id} must be Divine");
        }
    }

    /// D70 Q-X6-6's three entailed fixes beyond the book's own two examples:
    /// Bee King (Faerie, ArMDE:3486), Commanding Aura (Divine, ArMDE:3581),
    /// Raised from the Dead (Divine, ArMDE:6648).
    #[test]
    fn entailed_fixed_entries() {
        let rs = full_ruleset();
        let cases = [
            ("virtue.bee_king", Realm::Faerie),
            ("virtue.commanding_aura", Realm::Divine),
            ("flaw.raised_from_the_dead", Realm::Divine),
        ];
        for (id, realm) in cases {
            let item = with_association(&rs, id, RealmAssociation::Fixed { realm });
            let resolved = resolve_realm(&item, &sel(id), Some(Realm::Infernal));
            assert_eq!(resolved.realm, realm, "{id} must be {realm}");
        }
    }

    /// ArMDE:3000: every Tainted entry resolves Infernal by the general
    /// Tainted rule, derived from `tainted: true` alone — no
    /// `realm_association` needed. Exercised against the REAL shipped data
    /// (already `tainted: true`).
    #[test]
    fn tainted_entries_resolve_infernal_by_rule() {
        let rs = full_ruleset();
        for id in [
            "virtue.amorphous_major",
            "virtue.amorphous_minor",
            "virtue.command_animals",
            "virtue.demonic_blood",
            "virtue.immune_to_disease",
            "virtue.infernal_heirloom",
            "flaw.corrupted_abilities",
        ] {
            let item = rs.item(&Id::new(id)).unwrap();
            assert!(item.tainted, "{id} must be tainted in shipped data");
            let resolved = resolve_realm(item, &sel(id), Some(Realm::Faerie));
            assert_eq!(
                resolved.realm,
                Realm::Infernal,
                "{id} must resolve Infernal"
            );
        }
    }

    /// Demonic Might/Powers are NOT `tainted`-tagged (verified against the
    /// shipped data), so they need `Fixed { realm: Infernal }` data directly
    /// — fixed via the ArMDE:3661 cross-entry rule on Demonic Blood, not via
    /// the Tainted tag.
    #[test]
    fn demonic_might_and_powers_fixed_infernal_via_cross_entry_rule() {
        let rs = full_ruleset();
        for id in ["virtue.demonic_might", "virtue.demonic_powers"] {
            let shipped = rs.item(&Id::new(id)).unwrap();
            assert!(
                !shipped.tainted,
                "{id} is not tainted-tagged in shipped data"
            );
            let item = with_association(
                &rs,
                id,
                RealmAssociation::Fixed {
                    realm: Realm::Infernal,
                },
            );
            let resolved = resolve_realm(&item, &sel(id), Some(Realm::Faerie));
            assert_eq!(resolved.realm, Realm::Infernal);
        }
    }
}

// --- A fixed entry's override is IGNORED, not rejected (D74 Q4) ------------

mod fixed_entry_override_decision {
    use super::*;

    const FIXED_ITEM: &str = r#"[
      { "id": "virtue.test_fixed_realm", "kind": "virtue", "magnitude": "minor",
        "categories": ["supernatural"], "classification": "uncomputed_rule",
        "entity_kinds": ["character"],
        "realm_association": { "kind": "fixed", "realm": "faerie" } },
      { "id": "flaw.filler_personality", "kind": "flaw", "magnitude": "minor",
        "categories": ["personality"], "classification": "narrative" }
    ]"#;

    const COMPANION_TYPE: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 20, "flaw_points": 20 },
        "permitted_categories": ["general", "supernatural"],
        "creation_phases": [] }
    ]"#;

    fn ruleset() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: FIXED_ITEM,
            type_profiles: COMPANION_TYPE,
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// An override on a Fixed entry raises NEITHER `realm_changed_default`
    /// nor `realm_override_invalid` — it is silently ignored (forward/back
    /// compatible with a save from before the entry was fixed).
    #[test]
    fn override_on_a_fixed_entry_raises_no_issue() {
        let rs = ruleset();
        let e = entity(
            "companion",
            vec![sel_with_override(
                "virtue.test_fixed_realm",
                Realm::Infernal,
            )],
        );
        let issues = validate(&e, &rs).issues;
        assert!(!has_code(&issues, "realm_changed_default"));
        assert!(!has_code(&issues, "realm_override_invalid"));
    }

    #[test]
    fn override_on_a_fixed_entry_is_ignored_by_the_resolver() {
        let rs = ruleset();
        let item = rs.item(&Id::new("virtue.test_fixed_realm")).unwrap();
        let selection = sel_with_override("virtue.test_fixed_realm", Realm::Infernal);
        let resolved = resolve_realm(item, &selection, None);
        assert_eq!(resolved.realm, Realm::Faerie);
    }
}

// --- Manifest Sin's subset (D74 Q2) -----------------------------------------

mod manifest_sin_subset {
    use super::*;

    fn subset() -> std::collections::BTreeSet<Realm> {
        std::collections::BTreeSet::from([Realm::Divine, Realm::Infernal])
    }

    #[test]
    fn unanswered_subset_warns_instead_of_falling_back_to_magic() {
        let rs = full_ruleset();
        let item = with_association(
            &rs,
            "flaw.manifest_sin",
            RealmAssociation::Subset { realms: subset() },
        );
        let resolved = resolve_realm(&item, &sel("flaw.manifest_sin"), None);
        assert_eq!(resolved.warning, Some(RealmWarning::UnansweredSubset));
        assert!(
            subset().contains(&resolved.realm),
            "must not silently fall back to Magic"
        );
    }

    #[test]
    fn concept_realm_inside_the_subset_is_used_without_warning() {
        let rs = full_ruleset();
        let item = with_association(
            &rs,
            "flaw.manifest_sin",
            RealmAssociation::Subset { realms: subset() },
        );
        let resolved = resolve_realm(&item, &sel("flaw.manifest_sin"), Some(Realm::Infernal));
        assert_eq!(resolved.realm, Realm::Infernal);
        assert_eq!(resolved.warning, None);
    }

    #[test]
    fn concept_realm_outside_the_subset_still_warns() {
        let rs = full_ruleset();
        let item = with_association(
            &rs,
            "flaw.manifest_sin",
            RealmAssociation::Subset { realms: subset() },
        );
        let resolved = resolve_realm(&item, &sel("flaw.manifest_sin"), Some(Realm::Magic));
        assert_eq!(resolved.warning, Some(RealmWarning::UnansweredSubset));
    }

    #[test]
    fn override_inside_the_subset_is_accepted_without_warning() {
        let rs = full_ruleset();
        let item = with_association(
            &rs,
            "flaw.manifest_sin",
            RealmAssociation::Subset { realms: subset() },
        );
        let selection = sel_with_override("flaw.manifest_sin", Realm::Infernal);
        let resolved = resolve_realm(&item, &selection, None);
        assert_eq!(resolved.realm, Realm::Infernal);
        assert_eq!(resolved.warning, None);
    }

    const SUBSET_ITEM: &str = r#"[
      { "id": "flaw.test_subset_realm", "kind": "flaw", "magnitude": "minor",
        "categories": ["supernatural"], "classification": "uncomputed_rule",
        "entity_kinds": ["character"],
        "realm_association": { "kind": "subset", "realms": ["divine", "infernal"] } },
      { "id": "flaw.filler_personality", "kind": "flaw", "magnitude": "minor",
        "categories": ["personality"], "classification": "narrative" }
    ]"#;

    const COMPANION_TYPE: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 20, "flaw_points": 20 },
        "permitted_categories": ["general", "supernatural"],
        "creation_phases": [] }
    ]"#;

    fn ruleset() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: SUBSET_ITEM,
            type_profiles: COMPANION_TYPE,
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// Already green: this is an infrastructure/loader proof (D74 Q2's "a
    /// general loader change"), not the resolver behavior under test above.
    #[test]
    fn the_subset_shape_loads() {
        assert!(
            Ruleset::from_sources(RulesetSources {
                id: "test",
                version: "1",
                point_items: SUBSET_ITEM,
                type_profiles: COMPANION_TYPE,
                ..RulesetSources::default()
            })
            .is_ok()
        );
    }

    #[test]
    fn unanswered_subset_warns_end_to_end_through_validate() {
        let rs = ruleset();
        let e = entity("companion", vec![sel("flaw.test_subset_realm")]);
        let issues = validate(&e, &rs).issues;
        assert!(has_code(&issues, "realm_unset_subset"));
    }

    /// D74 Q2: an override naming a realm outside the subset is an error
    /// (analogy with `unknown_param_value`'s treatment of any other
    /// out-of-domain parameter value), not resolved silently.
    #[test]
    fn override_outside_the_subset_is_a_validator_error() {
        let rs = ruleset();
        let e = entity(
            "companion",
            vec![sel_with_override("flaw.test_subset_realm", Realm::Magic)],
        );
        let issues = validate(&e, &rs).issues;
        assert!(has_code(&issues, "realm_override_invalid"));
    }
}

// --- Entry-level default that beats concept; changing it warns (D70/D74) ---

mod changed_default_class {
    use super::*;

    /// Hex (Infernal, ArMDE:4077 "most often"), Spiritual Pact (Magic,
    /// ArMDE:5012), Warped by Magic (Magic, ArMDE:7021), Sufi (Divine,
    /// D74), Cursed Guile (Infernal, D74). Each: the default applies with no
    /// override and no warning; an override raises `realm_changed_default`
    /// EVEN when the concept realm would otherwise have agreed with the
    /// override, since a `Default` entry never consults the concept at all.
    #[test]
    fn default_realm_applies_with_no_override_and_no_warning() {
        let rs = full_ruleset();
        let cases = [
            ("virtue.hex", Realm::Infernal),
            ("virtue.spiritual_pact", Realm::Magic),
            ("flaw.warped_by_magic", Realm::Magic),
            ("virtue.sufi", Realm::Divine),
            ("flaw.cursed_guile", Realm::Infernal),
        ];
        for (id, default_realm) in cases {
            let item = with_association(
                &rs,
                id,
                RealmAssociation::Default {
                    realm: default_realm,
                },
            );
            let resolved = resolve_realm(&item, &sel(id), Some(Realm::Faerie));
            assert_eq!(
                resolved.realm, default_realm,
                "{id} defaults to {default_realm}"
            );
            assert_eq!(resolved.warning, None, "{id} unset raises no warning");
        }
    }

    #[test]
    fn overriding_the_default_warns() {
        let rs = full_ruleset();
        let cases = [
            ("virtue.hex", Realm::Infernal),
            ("virtue.spiritual_pact", Realm::Magic),
            ("flaw.warped_by_magic", Realm::Magic),
            ("virtue.sufi", Realm::Divine),
            ("flaw.cursed_guile", Realm::Infernal),
        ];
        for (id, default_realm) in cases {
            let item = with_association(
                &rs,
                id,
                RealmAssociation::Default {
                    realm: default_realm,
                },
            );
            let other = if default_realm == Realm::Magic {
                Realm::Faerie
            } else {
                Realm::Magic
            };
            let selection = sel_with_override(id, other);
            let resolved = resolve_realm(&item, &selection, None);
            assert_eq!(resolved.realm, other, "{id} override is honored");
            assert_eq!(
                resolved.warning,
                Some(RealmWarning::ChangedDefault),
                "{id} changing the default must warn"
            );
        }
    }

    const DEFAULT_ITEM: &str = r#"[
      { "id": "virtue.test_default_realm", "kind": "virtue", "magnitude": "minor",
        "categories": ["supernatural"], "classification": "uncomputed_rule",
        "entity_kinds": ["character"],
        "realm_association": { "kind": "default", "realm": "infernal" } },
      { "id": "flaw.filler_personality", "kind": "flaw", "magnitude": "minor",
        "categories": ["personality"], "classification": "narrative" }
    ]"#;

    const COMPANION_TYPE: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 20, "flaw_points": 20 },
        "permitted_categories": ["general", "supernatural"],
        "creation_phases": [] }
    ]"#;

    /// `validate_realm_associations`'s own `ChangedDefault` arm (QA review,
    /// coverage): every other test of the `ChangedDefault` warning calls
    /// `resolve_realm` directly, so the `validate()`-level issue-emitting
    /// branch (`validation/realm.rs`) was never actually exercised end to
    /// end. An override differing from a Default entry's stated realm must
    /// raise `realm_changed_default` through the real `validate()` entry
    /// point, not merely through the resolver it wraps.
    #[test]
    fn overriding_the_default_raises_the_issue_through_validate() {
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: DEFAULT_ITEM,
            type_profiles: COMPANION_TYPE,
            ..RulesetSources::default()
        })
        .unwrap();
        let e = entity(
            "companion",
            vec![sel_with_override("virtue.test_default_realm", Realm::Magic)],
        );
        let issues = validate(&e, &rs).issues;
        assert!(has_code(&issues, "realm_changed_default"));
    }
}

// --- Named-realm Flaws: default from their own param, no warning (D74 Q3) --

mod named_realm_flaws {
    use super::*;

    /// Bound to (Realm), (Realm) Stigmatic, Necessary (Realm) Aura for
    /// (Ability) each already declare their own `realm` parameter (the
    /// bound-to / sensitive / aura realm). D74 Q3: the association defaults
    /// to THAT param's value, overridable, with no warning either way.
    #[test]
    fn defaults_to_the_entrys_own_named_realm_param() {
        let rs = full_ruleset();
        for id in [
            "flaw.bound_to_realm",
            "flaw.realm_stigmatic",
            "flaw.necessary_realm_aura_for_ability",
        ] {
            let item = with_association(
                &rs,
                id,
                RealmAssociation::FromParam {
                    key: "realm".to_string(),
                },
            );
            let selection = Selection::with_params(
                Id::new(id),
                BTreeMap::from([("realm".to_string(), Realm::Divine.id())]),
            );
            let resolved = resolve_realm(&item, &selection, Some(Realm::Faerie));
            assert_eq!(
                resolved.realm,
                Realm::Divine,
                "{id} follows its own realm param"
            );
            assert_eq!(resolved.warning, None);
        }
    }

    #[test]
    fn overriding_the_named_realm_raises_no_warning() {
        let rs = full_ruleset();
        let item = with_association(
            &rs,
            "flaw.bound_to_realm",
            RealmAssociation::FromParam {
                key: "realm".to_string(),
            },
        );
        let mut params = BTreeMap::from([("realm".to_string(), Realm::Divine.id())]);
        params.insert(REALM_OVERRIDE_PARAM_KEY.to_string(), Realm::Infernal.id());
        let selection = Selection::with_params(Id::new("flaw.bound_to_realm"), params);
        let resolved = resolve_realm(&item, &selection, None);
        assert_eq!(
            resolved.realm,
            Realm::Infernal,
            "override beats the named param"
        );
        assert_eq!(resolved.warning, None, "D74 Q3: no warning on this class");
    }
}

// --- The `association` key is legal-but-never-required (real, not stubbed) -

mod association_key_param_exemption {
    use super::*;

    /// Already green: this is phase 1's real (non-stub) wiring in
    /// `validate_selection_parameters` — an unset override must never raise
    /// `missing_param` on an old save (D70 Q-X6-5: "only overrides are
    /// stored").
    #[test]
    fn unset_association_on_a_supernatural_entry_is_not_missing_param() {
        let rs = full_ruleset();
        let e = entity("companion", vec![sel("virtue.animal_ken")]);
        let issues = validate(&e, &rs).issues;
        assert!(!issues.iter().any(|i| i.code == "missing_param"
            && i.args.get("item").map(String::as_str) == Some("virtue.animal_ken")));
    }

    /// Already green: providing the override on a qualifying entry is not
    /// `unexpected_param`.
    #[test]
    fn association_override_on_a_supernatural_entry_is_not_unexpected_param() {
        let rs = full_ruleset();
        let e = entity(
            "companion",
            vec![sel_with_override("virtue.animal_ken", Realm::Divine)],
        );
        let issues = validate(&e, &rs).issues;
        assert!(!has_code(&issues, "unexpected_param"));
    }
}

// --- The shipped data actually carries what phase 2 authored ---------------

mod shipped_data_matches_the_verdicts {
    use super::*;
    use std::collections::BTreeSet;

    /// Locks `rules/core/virtues_flaws.json`'s authored `realm_association`
    /// values against the verdicts table + D74 — the tests elsewhere in this
    /// file exercise `resolve_realm`'s *logic* via `with_association` clones,
    /// which would stay green even if the shipped JSON were wrong or missing
    /// entirely. This is the one place that reads it unmodified.
    #[test]
    fn fixed_entries() {
        let rs = full_ruleset();
        let cases = [
            ("virtue.faerie_blood", Realm::Faerie),
            ("virtue.strong_faerie_blood", Realm::Faerie),
            ("virtue.bee_king", Realm::Faerie),
            ("virtue.kassalan_exorcism", Realm::Magic),
            ("virtue.magical_blood", Realm::Magic),
            ("flaw.monstrous_blood", Realm::Magic),
            ("virtue.strong_angelic_heritage", Realm::Divine),
            ("virtue.blood_of_the_nephilim", Realm::Divine),
            ("flaw.viaticarus", Realm::Divine),
            ("virtue.commanding_aura", Realm::Divine),
            ("flaw.raised_from_the_dead", Realm::Divine),
            ("virtue.demonic_might", Realm::Infernal),
            ("virtue.demonic_powers", Realm::Infernal),
        ];
        for (id, realm) in cases {
            let item = rs.item(&Id::new(id)).unwrap();
            assert_eq!(
                item.realm_association,
                Some(RealmAssociation::Fixed { realm }),
                "{id} shipped realm_association"
            );
        }
    }

    #[test]
    fn default_entries() {
        let rs = full_ruleset();
        let cases = [
            ("virtue.hex", Realm::Infernal),
            ("virtue.spiritual_pact", Realm::Magic),
            ("flaw.warped_by_magic", Realm::Magic),
            ("virtue.sufi", Realm::Divine),
            ("flaw.cursed_guile", Realm::Infernal),
        ];
        for (id, realm) in cases {
            let item = rs.item(&Id::new(id)).unwrap();
            assert_eq!(
                item.realm_association,
                Some(RealmAssociation::Default { realm }),
                "{id} shipped realm_association"
            );
        }
    }

    #[test]
    fn subset_entry() {
        let rs = full_ruleset();
        let item = rs.item(&Id::new("flaw.manifest_sin")).unwrap();
        assert_eq!(
            item.realm_association,
            Some(RealmAssociation::Subset {
                realms: BTreeSet::from([Realm::Divine, Realm::Infernal])
            })
        );
    }

    #[test]
    fn from_param_entries() {
        let rs = full_ruleset();
        for id in [
            "flaw.bound_to_realm",
            "flaw.realm_stigmatic",
            "flaw.necessary_realm_aura_for_ability",
            "virtue.folk_magic",
        ] {
            let item = rs.item(&Id::new(id)).unwrap();
            assert_eq!(
                item.realm_association,
                Some(RealmAssociation::FromParam {
                    key: "realm".to_string()
                }),
                "{id} shipped realm_association"
            );
        }
    }

    /// The Tainted entries in the 17-fixed tally derive Infernal from
    /// `tainted` alone (ArMDE:3000) — they must NOT also carry a
    /// `realm_association` (that would store the same fact twice, and the
    /// two could silently disagree after an edit to one but not the other).
    #[test]
    fn tainted_entries_carry_no_separate_association_data() {
        let rs = full_ruleset();
        for id in [
            "virtue.amorphous_major",
            "virtue.amorphous_minor",
            "virtue.command_animals",
            "virtue.demonic_blood",
            "virtue.immune_to_disease",
            "virtue.infernal_heirloom",
            "flaw.corrupted_abilities",
            "flaw.false_power",
            "flaw.false_power_minor",
        ] {
            let item = rs.item(&Id::new(id)).unwrap();
            assert!(item.tainted, "{id} must be tainted");
            assert_eq!(
                item.realm_association, None,
                "{id} must not double-store the Infernal association"
            );
        }
    }
}

// --- Load-time integrity rejects the two authoring slips `RealmAssociation`
// makes possible, rather than letting either look enforced and not be (an
// empty Subset) or resolve silently wrong (a FromParam key typo). -----------

mod realm_association_integrity {
    use super::*;

    const COMPANION_TYPE: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 20, "flaw_points": 20 },
        "permitted_categories": ["general", "supernatural"],
        "creation_phases": [] }
    ]"#;

    const FILLER: &str = r#"
      { "id": "flaw.filler_personality", "kind": "flaw", "magnitude": "minor",
        "categories": ["personality"], "classification": "narrative" }
    "#;

    #[test]
    fn an_empty_subset_is_rejected_at_load() {
        let point_items = format!(
            r#"[
              {{ "id": "flaw.test_empty_subset", "kind": "flaw", "magnitude": "minor",
                "categories": ["supernatural"], "classification": "uncomputed_rule",
                "realm_association": {{ "kind": "subset", "realms": [] }} }},
              {FILLER}
            ]"#
        );
        let err = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: &point_items,
            type_profiles: COMPANION_TYPE,
            ..RulesetSources::default()
        })
        .unwrap_err();
        assert!(format!("{err}").contains("empty 'subset'"), "got: {err}");
    }

    #[test]
    fn a_from_param_key_the_item_does_not_declare_is_rejected_at_load() {
        let point_items = format!(
            r#"[
              {{ "id": "flaw.test_bad_from_param", "kind": "flaw", "magnitude": "minor",
                "categories": ["supernatural"], "classification": "uncomputed_rule",
                "realm_association": {{ "kind": "from_param", "key": "realm" }} }},
              {FILLER}
            ]"#
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
            format!("{err}").contains("declares no parameter"),
            "got: {err}"
        );
    }

    #[test]
    fn a_from_param_key_of_the_wrong_domain_is_rejected_at_load() {
        let point_items = format!(
            r#"[
              {{ "id": "flaw.test_wrong_domain_from_param", "kind": "flaw", "magnitude": "minor",
                "categories": ["supernatural"], "classification": "uncomputed_rule",
                "parameters": [{{ "key": "realm", "type": "ref", "domain": "text" }}],
                "realm_association": {{ "kind": "from_param", "key": "realm" }} }},
              {FILLER}
            ]"#
        );
        let err = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: &point_items,
            type_profiles: COMPANION_TYPE,
            ..RulesetSources::default()
        })
        .unwrap_err();
        assert!(format!("{err}").contains("not 'realm'"), "got: {err}");
    }
}

// --- A garbage override must never panic (CLAUDE.md's trust boundary) -----

mod garbage_override_safety {
    use super::*;

    fn sel_with_raw_override(id: &str, raw: &str) -> Selection {
        Selection::with_params(
            Id::new(id),
            BTreeMap::from([(REALM_OVERRIDE_PARAM_KEY.to_string(), Id::new(raw))]),
        )
    }

    /// A hand-edited save's `association` value that is not a `realm.<slug>`
    /// id at all must not panic the resolver — it is silently ignored there
    /// (falls through the rest of the chain) and reported once by the
    /// validator on the general `unknown_param_value` code, exactly as any
    /// other unresolved parameter value.
    #[test]
    fn resolve_realm_ignores_an_unparseable_override_without_panicking() {
        let rs = full_ruleset();
        let item = rs.item(&Id::new("virtue.animal_ken")).unwrap();
        let selection = sel_with_raw_override("virtue.animal_ken", "realm.nonsense");
        let resolved = resolve_realm(item, &selection, Some(Realm::Faerie));
        assert_eq!(
            resolved.realm,
            Realm::Faerie,
            "falls through to the concept realm"
        );
        assert_eq!(resolved.warning, None);
    }

    /// A BLANK override (the key present with an empty/whitespace value, the
    /// shape `setParamAt` in the UI writes when a control is cleared) is not
    /// "garbage" — it is the same "nothing chosen yet" `param_value_is_blank`
    /// already treats as unfilled for every declared parameter. Must resolve
    /// exactly like an absent key, not raise `unknown_param_value`.
    #[test]
    fn a_blank_override_resolves_like_an_absent_one() {
        let rs = full_ruleset();
        let item = rs.item(&Id::new("virtue.animal_ken")).unwrap();
        let selection = sel_with_raw_override("virtue.animal_ken", "  ");
        let resolved = resolve_realm(item, &selection, Some(Realm::Faerie));
        assert_eq!(resolved.realm, Realm::Faerie);

        let e = entity("companion", vec![selection]);
        let issues = validate(&e, &rs).issues;
        assert!(!has_code(&issues, "unknown_param_value"));
    }

    #[test]
    fn validate_flags_an_unparseable_override_as_unknown_param_value() {
        let rs = full_ruleset();
        let e = entity(
            "companion",
            vec![sel_with_raw_override("virtue.animal_ken", "realm.nonsense")],
        );
        let issues = validate(&e, &rs).issues;
        let found = issues.iter().find(|i| i.code == "unknown_param_value");
        let found = found.expect("an unparseable association value must be flagged");
        assert_eq!(
            found.args.get("value").map(String::as_str),
            Some("realm.nonsense")
        );
    }
}

// --- Old-save compatibility: the new fields are purely additive ------------

mod old_save_compatibility {
    use super::*;

    /// Already green: `concept_realm` is `serde(default, skip_serializing_if
    /// = "Option::is_none")`, so an entity that never sets it serializes
    /// exactly as it did before this field existed, and round-trips
    /// byte-identical. Locks the invariant CLAUDE.md requires (no
    /// SCHEMA_VERSION bump, no new error on an old save).
    #[test]
    fn concept_realm_is_absent_from_json_when_unset_and_round_trips() {
        let mut original = entity("companion", vec![sel("virtue.animal_ken")]);
        original.name = "Old Save".to_string();
        let json = serde_json::to_string(&original).expect("serializes");
        assert!(
            !json.contains("concept_realm"),
            "an unset concept_realm must not appear in the save at all"
        );
        let reloaded: Entity = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(reloaded, original);
        assert_eq!(reloaded.concept_realm, None);
    }

    /// The Some-case counterpart (QA review): a saved character that HAS
    /// chosen a concept realm, and ALSO carries a per-entry `association`
    /// override on one selection, must round-trip both fields with full
    /// fidelity — CLAUDE.md rates save/load round-trip fidelity as a
    /// high-severity, load-bearing product guarantee.
    #[test]
    fn concept_realm_and_a_per_entry_override_round_trip_when_set() {
        let mut original = entity(
            "companion",
            vec![sel_with_override("virtue.animal_ken", Realm::Divine)],
        );
        original.name = "Realm Round Trip".to_string();
        original.concept_realm = Some(Realm::Faerie);

        let json = serde_json::to_string(&original).expect("serializes");
        assert!(
            json.contains("\"concept_realm\":\"faerie\""),
            "a set concept_realm must serialize its realm tag, got: {json}"
        );
        assert!(
            json.contains(&format!("\"{REALM_OVERRIDE_PARAM_KEY}\":\"realm.divine\"")),
            "the per-entry override param must serialize its realm id, got: {json}"
        );

        let reloaded: Entity = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(reloaded, original);
        assert_eq!(reloaded.concept_realm, Some(Realm::Faerie));
    }
}
