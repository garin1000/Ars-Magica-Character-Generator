//! Mythic Companion types: the catalogue of the supernatural "kinds" a Mythic
//! Companion can be (Devil Child, Faerie Doctor, Nephilim, Spirit Votary) and
//! the free Virtues + required V/F package each imposes.
//!
//! Parallels the Hermetic [`crate::house`] system: a Mythic Companion's *type*
//! is a data-driven profile carrying a list of point-free [`Grant`]s (its free
//! "status" Virtue + free Minor Virtue) resolved through the shared
//! [`crate::grant`] model, **plus** a required V/F package that counts against
//! the point budget normally, **plus** per-type budget bonuses.
//!
//! Free grants are budget-exempt (resolved, never stored) but **not**
//! cap-exempt: they fold into the list [`crate::validation::validate`] checks,
//! so a granted copy counts toward the selection caps. The required
//! package is ordinary bought [`Selection`]s the UI auto-seeds; the required
//! Flaws are swappable for a "suitable substitute agreed with the troupe", so
//! a missing/removed required slot is a non-blocking warning, never a hard
//! block. Per-type bonus points raise the balance ceilings (see
//! [`crate::validation`]).
//!
//! Source: Ars Magica - Definitive Edition (Core Rules).md:2635-2639 (general
//! rules), :2643-2765 (the four types), :2842-2851 (V/F guidelines).

use serde::{Deserialize, Serialize};

use crate::grant::{Grant, GrantConstraint, resolve_grants};
use crate::ruleset::Ruleset;
use crate::types::{Entity, Id, Selection, SourceRef};

/// `true` when a `u8` budget bonus is its default of 0 (omitted from canonical
/// JSON).
fn is_zero_u8(n: &u8) -> bool {
    *n == 0
}

/// A required Flaw a Mythic Companion type imposes, with the rules-specified
/// `default` and the `constraint` a "suitable substitute agreed with the troupe"
/// must satisfy (same magnitude/category). The UI seeds `default` and offers the
/// constraint-eligible Flaws as substitutes.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2660, :2689, :2754
/// ("or a suitable substitute agreed with the troupe").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredFlaw {
    /// The Flaw the rules name for this type (the pre-filled default).
    pub default: Selection,
    /// What an acceptable substitute must satisfy.
    pub constraint: GrantConstraint,
}

/// A single Mythic Companion type in the catalogue. Its display name and
/// description live in `rules/i18n`, keyed by `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MythicCompanionType {
    /// Slug-style id, e.g. `mythic_type.devil_child`.
    pub id: Id,
    /// Point-free grants: the free "status" Virtue + the free Minor Virtue.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grants: Vec<Grant>,
    /// Fixed required Virtues (budgeted). Stored as [`Selection`]s so a
    /// parameterized/duplicated requirement is exact — Nephilim needs *both*
    /// Great Stamina and Great Strength (`virtue.great_characteristic` twice with
    /// different `characteristic` params), and Puissant Guile is
    /// `virtue.puissant_ability` + `ability=guile`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_virtues: Vec<Selection>,
    /// Required Flaws (budgeted), each with a default + a substitute constraint.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_flaws: Vec<RequiredFlaw>,
    /// Extra Flaw points this type may take beyond the base ceiling (Devil Child
    /// and Spirit Votary get +7). Each still funds virtue points at the type's
    /// rate. Source: Ars Magica - Definitive Edition (Core Rules).md:2664;
    /// Ars Magica 5e - Realms of Power - Magic.md:5486.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub bonus_flaw_points: u8,
    /// Extra virtue points at no flaw cost (Devil Child gets +3, to balance the
    /// compulsory Major Flaw). Source: Ars Magica - Definitive Edition
    /// (Core Rules).md:2664.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub bonus_free_virtue_points: u8,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

impl MythicCompanionType {
    /// The full required package (Virtues + each Flaw's default) as the
    /// [`Selection`]s the UI auto-seeds and the validator checks are present.
    pub fn required_selections(&self) -> Vec<Selection> {
        self.required_virtues
            .iter()
            .cloned()
            .chain(self.required_flaws.iter().map(|f| f.default.clone()))
            .collect()
    }
}

/// The on-disk shape of `rules/core/mythic_companion_types.json`. Internal
/// deserialize-only wrapper (`pub(crate)`), mirroring [`crate::house::HousesFile`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct MythicCompanionTypesFile {
    /// The core Mythic Companion types.
    #[serde(default)]
    pub types: Vec<MythicCompanionType>,
}

/// Derives the point-free Virtue [`Selection`] rows the entity's Mythic
/// Companion type grants, from the stored `(mythic_type, mythic_choices)`
/// choices. Thin wrapper over [`crate::grant::resolve_grants`]: an entity with
/// no type, or one naming a type absent from the ruleset, grants nothing.
pub fn granted_selections(entity: &Entity, ruleset: &Ruleset) -> Vec<Selection> {
    let Some(type_id) = &entity.mythic_type else {
        return Vec::new();
    };
    let Some(mtype) = ruleset.mythic_type(type_id) else {
        return Vec::new();
    };
    resolve_grants(&mtype.grants, &entity.mythic_choices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grant::GrantConstraint;
    use crate::ruleset::{Ruleset, RulesetSources};
    use crate::types::{EntityKind, ItemKind, RulesetRef};
    use pretty_assertions::assert_eq;
    use std::collections::{BTreeMap, BTreeSet};

    /// Point items the test types reference, enough for `validate_mythic_type_refs`
    /// (grant items + required virtue/flaw refs) to pass at load.
    const ITEMS: &str = r#"[
        { "id": "virtue.devil_child", "kind": "virtue", "classification": "narrative", "magnitude": "free",
          "categories": ["social_status"], "entity_kinds": ["character"] },
        { "id": "virtue.demonic_might", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "categories": ["supernatural"], "entity_kinds": ["character"] },
        { "id": "virtue.demonic_powers", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "categories": ["supernatural"], "entity_kinds": ["character"] },
        { "id": "virtue.demonic_blood", "kind": "virtue", "classification": "narrative", "magnitude": "major",
          "categories": ["supernatural"], "entity_kinds": ["character"] },
        { "id": "flaw.tragic_life", "kind": "flaw", "classification": "narrative", "magnitude": "major",
          "categories": ["supernatural"], "entity_kinds": ["character"] },
        { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
          "categories": ["personality"], "entity_kinds": ["character"] }
    ]"#;

    const TYPES: &str = r#"{ "types": [
        { "id": "mythic_type.devil_child",
          "grants": [
            { "kind": "fixed", "item": "virtue.devil_child" },
            { "kind": "choice", "choice_key": "devil_child_might", "options": [
              { "ref": "virtue.demonic_might" }, { "ref": "virtue.demonic_powers" } ] } ],
          "required_virtues": [ { "ref": "virtue.demonic_blood" } ],
          "required_flaws": [ { "default": { "ref": "flaw.tragic_life" },
            "constraint": { "kind": "flaw", "magnitude": "major", "require_categories": ["supernatural"] } } ],
          "bonus_flaw_points": 7, "bonus_free_virtue_points": 3 }
    ] }"#;

    fn ruleset() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: "[]",
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: Some(TYPES),
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
        })
        .unwrap()
    }

    fn devil_child() -> Entity {
        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("mythic_companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        e.mythic_type = Some(Id::new("mythic_type.devil_child"));
        e
    }

    #[test]
    fn type_roundtrips_with_bonuses_and_package() {
        let mtype: MythicCompanionType = serde_json::from_str(
            r#"{ "id": "mythic_type.devil_child",
                 "grants": [ { "kind": "fixed", "item": "virtue.devil_child" } ],
                 "required_virtues": [ { "ref": "virtue.demonic_blood" } ],
                 "required_flaws": [ { "default": { "ref": "flaw.tragic_life" },
                   "constraint": { "kind": "flaw", "magnitude": "major" } } ],
                 "bonus_flaw_points": 7, "bonus_free_virtue_points": 3 }"#,
        )
        .unwrap();
        assert_eq!(mtype.id, Id::new("mythic_type.devil_child"));
        assert_eq!(mtype.bonus_flaw_points, 7);
        assert_eq!(mtype.bonus_free_virtue_points, 3);
        assert_eq!(mtype.required_virtues.len(), 1);
        assert_eq!(
            mtype.required_flaws[0].default.item_ref,
            Id::new("flaw.tragic_life")
        );
        let back = serde_json::to_string(&mtype).unwrap();
        assert_eq!(
            serde_json::from_str::<MythicCompanionType>(&back).unwrap(),
            mtype
        );
    }

    #[test]
    fn bonuses_default_to_zero_and_are_omitted() {
        let mtype: MythicCompanionType =
            serde_json::from_str(r#"{ "id": "mythic_type.faerie_doctor" }"#).unwrap();
        assert_eq!(mtype.bonus_flaw_points, 0);
        assert_eq!(mtype.bonus_free_virtue_points, 0);
        let json = serde_json::to_string(&mtype).unwrap();
        assert!(!json.contains("bonus_flaw_points"));
        assert!(!json.contains("bonus_free_virtue_points"));
    }

    #[test]
    fn required_selections_flattens_virtues_and_flaw_defaults() {
        let rs = ruleset();
        let mtype = rs.mythic_type(&Id::new("mythic_type.devil_child")).unwrap();
        assert_eq!(
            mtype.required_selections(),
            vec![
                Selection::new(Id::new("virtue.demonic_blood")),
                Selection::new(Id::new("flaw.tragic_life")),
            ]
        );
    }

    #[test]
    fn no_type_grants_nothing() {
        let rs = ruleset();
        let mut e = devil_child();
        e.mythic_type = None;
        assert!(granted_selections(&e, &rs).is_empty());
    }

    /// A `mythic_type` id absent from the ruleset resolves to no grants (the
    /// `ruleset.mythic_type(...)` lookup returns `None`), never a panic.
    #[test]
    fn unknown_type_grants_nothing() {
        let rs = ruleset();
        let mut e = devil_child();
        e.mythic_type = Some(Id::new("mythic_type.does_not_exist"));
        assert!(
            rs.mythic_type(&Id::new("mythic_type.does_not_exist"))
                .is_none()
        );
        assert!(granted_selections(&e, &rs).is_empty());
    }

    #[test]
    fn fixed_and_chosen_grants_resolve() {
        let rs = ruleset();
        let mut e = devil_child();
        e.mythic_choices.insert(
            "devil_child_might".to_string(),
            Selection::new(Id::new("virtue.demonic_powers")),
        );
        assert_eq!(
            granted_selections(&e, &rs),
            vec![
                Selection::new(Id::new("virtue.devil_child")),
                Selection::new(Id::new("virtue.demonic_powers")),
            ]
        );
    }

    #[test]
    fn free_minor_choice_omitted_until_picked() {
        let rs = ruleset();
        let e = devil_child();
        // Only the fixed status Virtue resolves; the Might/Powers choice is unset.
        assert_eq!(
            granted_selections(&e, &rs),
            vec![Selection::new(Id::new("virtue.devil_child"))]
        );
    }

    #[test]
    fn constraint_shape_is_usable() {
        // Guards against the RequiredFlaw constraint drifting from GrantConstraint.
        let c = GrantConstraint {
            kind: ItemKind::Flaw,
            magnitude: None,
            require_categories: BTreeSet::from(["supernatural".to_string()]),
            forbid_categories: BTreeSet::new(),
        };
        let f = RequiredFlaw {
            default: Selection::new(Id::new("flaw.tragic_life")),
            constraint: c,
        };
        assert_eq!(f.constraint.kind, ItemKind::Flaw);
        let _ = BTreeMap::<String, Selection>::new();
    }
}
