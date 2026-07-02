//! Hermetic Houses: the catalogue of the twelve Houses of the Order and the
//! free Virtue(s) each grants a magus at character creation.
//!
//! A House grant is **data**, not code: each [`House`] carries a list of
//! [`HouseGrant`]s describing what the House gives (a fixed Virtue, a choice
//! between options, or an open player-chosen Virtue/Flaw constrained by
//! category/magnitude). The engine resolves those grants into ordinary
//! [`Selection`]s at evaluation time (see [`crate::effective`]); the save stores
//! only the chosen House plus the player's specialisation picks, never the
//! resolved free-Virtue rows (the "saves store choices, not resolved values"
//! invariant).
//!
//! The three lineage classes (True Lineage, Mystery Cult, Societas) are a fixed
//! taxonomy and so an enum; the Houses and their grants are data.
//!
//! Source: Ars Magica - Definitive Edition (Core Rules).md:2270-2283 (the House
//! benefit table), :2855-2861 (the magus's free House Virtue).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::ruleset::Ruleset;
use crate::types::{Entity, Id, ItemKind, Magnitude, Selection, SourceRef};

/// The three structural classes of Hermetic House. Flavor/grouping only — drives
/// no mechanics; mirrors [`crate::art::ArtType`].
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2270-2283 (the `Type`
/// column of the House table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageType {
    /// A True Lineage (e.g. Bonisagus, Guernicus, Mercere, Tremere).
    TrueLineage,
    /// A Mystery Cult (e.g. Bjornaer, Criamon, Merinita, Verditius).
    MysteryCult,
    /// A Societas (e.g. Flambeau, Jerbiton, Tytalus, Ex Miscellanea).
    Societas,
}

impl LineageType {
    /// All three classes in a stable canonical order, the single source the
    /// serialized ordering is derived from.
    pub const ALL: [LineageType; 3] = [
        LineageType::TrueLineage,
        LineageType::MysteryCult,
        LineageType::Societas,
    ];
}

impl fmt::Display for LineageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LineageType::TrueLineage => "true_lineage",
            LineageType::MysteryCult => "mystery_cult",
            LineageType::Societas => "societas",
        })
    }
}

/// A constraint on a player-chosen open grant: the kind of item, an optional
/// magnitude, and category allow/deny lists. Purely declarative — enforced in
/// [`crate::validation`], with no House id hardcoded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrantConstraint {
    /// The item kind the open pick must be (Virtue or Flaw).
    pub kind: ItemKind,
    /// Required magnitude, if any (e.g. `major`). `None` = any magnitude.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magnitude: Option<Magnitude>,
    /// If non-empty, the pick's category must be one of these.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub require_categories: BTreeSet<String>,
    /// The pick's category must not be any of these.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub forbid_categories: BTreeSet<String>,
}

/// One thing a House grants its magi at creation.
///
/// Internally tagged on `kind` so each grant is a self-describing object in
/// `houses.json` (`{ "kind": "fixed", "item": … }`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HouseGrant {
    /// A fixed free Virtue (optionally parameterized), e.g. Tytalus →
    /// Self-Confident, or Tremere → Minor Magical Focus (certamen).
    Fixed {
        /// The granted point-item id.
        item: Id,
        /// Parameter values the grant fixes (e.g. `focus` = `certamen`).
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        params: BTreeMap<String, Id>,
    },
    /// A choice between a fixed set of options, e.g. Flambeau → Puissant Perdo
    /// **or** Puissant Ignem. The player's pick is stored on the entity keyed by
    /// `choice_key` and must be one of `options`.
    Choice {
        /// Stable key the entity's `house_choices` map uses for the pick.
        choice_key: String,
        /// The selectable options.
        options: Vec<Selection>,
    },
    /// An open, player-chosen Virtue/Flaw constrained by `constraint`, e.g.
    /// Jerbiton's free Minor Virtue, or Ex Miscellanea's Major non-Hermetic
    /// Virtue. The pick is a full [`Selection`] stored under `choice_key`.
    Open {
        /// Stable key the entity's `house_choices` map uses for the pick.
        choice_key: String,
        /// What the open pick must satisfy.
        constraint: GrantConstraint,
    },
}

/// A single Hermetic House in the catalogue. Its display name and description
/// live in `rules/i18n`, keyed by `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct House {
    /// Slug-style id, e.g. `house.bjornaer`.
    pub id: Id,
    /// The House's structural class.
    pub lineage_type: LineageType,
    /// The free Virtue(s)/Flaw(s) this House grants at creation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grants: Vec<HouseGrant>,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// The on-disk shape of `rules/core/houses.json`: the House catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HousesFile {
    /// The twelve core Houses.
    #[serde(default)]
    pub houses: Vec<House>,
}

/// Derives the free-Virtue [`Selection`] rows the entity's House grants, from
/// the stored `(house, house_choices)` choices — the single source of the
/// derived grant model. The save never persists these resolved rows; every
/// effect / prerequisite consumer that must "see" grants folds this list into
/// `entity.selections` (see [`crate::effective`]), while balance and caps stay
/// on the bought selections alone so grants are free and uncapped.
///
/// Resolution per grant kind:
/// - **Fixed** → the granted item with its fixed params.
/// - **Choice** → the player's pick keyed by `choice_key`, but only if it is one
///   of the offered `options`; an absent or off-menu pick emits nothing (the
///   discrepancy surfaces as a validation error, not here).
/// - **Open** → the player's pick keyed by `choice_key`, verbatim; eligibility
///   against the `constraint` is checked in validation, not here.
///
/// An entity with no House, or one naming a House absent from the ruleset,
/// grants nothing.
pub fn granted_selections(entity: &Entity, ruleset: &Ruleset) -> Vec<Selection> {
    let Some(house_id) = &entity.house else {
        return Vec::new();
    };
    let Some(house) = ruleset.house(house_id) else {
        return Vec::new();
    };
    house
        .grants
        .iter()
        .filter_map(|grant| resolve_grant(grant, &entity.house_choices))
        .collect()
}

/// Resolves one grant to the [`Selection`] it contributes, if any.
fn resolve_grant(grant: &HouseGrant, choices: &BTreeMap<String, Selection>) -> Option<Selection> {
    match grant {
        HouseGrant::Fixed { item, params } => {
            Some(Selection::with_params(item.clone(), params.clone()))
        }
        HouseGrant::Choice {
            choice_key,
            options,
        } => {
            let pick = choices.get(choice_key)?;
            options.contains(pick).then(|| pick.clone())
        }
        HouseGrant::Open { choice_key, .. } => choices.get(choice_key).cloned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn lineage_type_display_matches_serde_scalar() {
        for t in LineageType::ALL {
            let scalar = serde_json::to_value(t).unwrap();
            assert_eq!(scalar.as_str().unwrap(), t.to_string());
        }
    }

    #[test]
    fn lineage_type_all_is_canonical_order() {
        assert_eq!(
            LineageType::ALL,
            [
                LineageType::TrueLineage,
                LineageType::MysteryCult,
                LineageType::Societas
            ]
        );
        assert!(LineageType::TrueLineage < LineageType::Societas);
    }

    #[test]
    fn fixed_grant_roundtrips() {
        let json = r#"{ "kind": "fixed", "item": "virtue.self_confident" }"#;
        let grant: HouseGrant = serde_json::from_str(json).unwrap();
        assert_eq!(
            grant,
            HouseGrant::Fixed {
                item: Id::new("virtue.self_confident"),
                params: BTreeMap::new(),
            }
        );
        let back = serde_json::to_string(&grant).unwrap();
        assert_eq!(serde_json::from_str::<HouseGrant>(&back).unwrap(), grant);
    }

    #[test]
    fn fixed_grant_carries_params() {
        let json = r#"{ "kind": "fixed", "item": "virtue.minor_magical_focus",
                        "params": { "focus": "certamen" } }"#;
        let HouseGrant::Fixed { item, params } = serde_json::from_str(json).unwrap() else {
            panic!("expected a fixed grant");
        };
        assert_eq!(item, Id::new("virtue.minor_magical_focus"));
        assert_eq!(params.get("focus"), Some(&Id::new("certamen")));
    }

    #[test]
    fn choice_grant_roundtrips_with_ref_keyed_options() {
        let json = r#"{ "kind": "choice", "choice_key": "flambeau_puissant", "options": [
            { "ref": "virtue.puissant_art", "params": { "art": "art.perdo" } },
            { "ref": "virtue.puissant_art", "params": { "art": "art.ignem" } }
        ] }"#;
        let HouseGrant::Choice {
            choice_key,
            options,
        } = serde_json::from_str(json).unwrap()
        else {
            panic!("expected a choice grant");
        };
        assert_eq!(choice_key, "flambeau_puissant");
        assert_eq!(options.len(), 2);
        assert_eq!(options[0].item_ref, Id::new("virtue.puissant_art"));
        assert_eq!(options[1].params.get("art"), Some(&Id::new("art.ignem")));
    }

    #[test]
    fn open_grant_carries_constraint() {
        let json = r#"{ "kind": "open", "choice_key": "ex_misc_major_flaw",
            "constraint": { "kind": "flaw", "magnitude": "major", "require_categories": ["hermetic"] } }"#;
        let HouseGrant::Open {
            choice_key,
            constraint,
        } = serde_json::from_str(json).unwrap()
        else {
            panic!("expected an open grant");
        };
        assert_eq!(choice_key, "ex_misc_major_flaw");
        assert_eq!(constraint.kind, ItemKind::Flaw);
        assert_eq!(constraint.magnitude, Some(Magnitude::Major));
        assert!(constraint.require_categories.contains("hermetic"));
        assert!(constraint.forbid_categories.is_empty());
    }

    #[test]
    fn house_roundtrips() {
        let json = r#"{
            "id": "house.bjornaer",
            "lineage_type": "mystery_cult",
            "grants": [ { "kind": "fixed", "item": "virtue.heartbeast" } ],
            "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [2270, 2283] }
        }"#;
        let house: House = serde_json::from_str(json).unwrap();
        assert_eq!(house.id, Id::new("house.bjornaer"));
        assert_eq!(house.lineage_type, LineageType::MysteryCult);
        assert_eq!(house.grants.len(), 1);
        let back = serde_json::to_string(&house).unwrap();
        assert_eq!(serde_json::from_str::<House>(&back).unwrap(), house);
    }

    #[test]
    fn houses_file_loads_catalogue() {
        let json = r#"{
            "houses": [
                { "id": "house.bonisagus", "lineage_type": "true_lineage", "grants": [
                    { "kind": "choice", "choice_key": "bonisagus_puissant", "options": [
                        { "ref": "virtue.puissant_ability", "params": { "ability": "ability.magic_theory" } },
                        { "ref": "virtue.puissant_ability", "params": { "ability": "ability.intrigue" } }
                    ] } ] },
                { "id": "house.tytalus", "lineage_type": "societas", "grants": [
                    { "kind": "fixed", "item": "virtue.self_confident" } ] }
            ]
        }"#;
        let file: HousesFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.houses.len(), 2);
        assert_eq!(file.houses[0].id, Id::new("house.bonisagus"));
        assert_eq!(file.houses[1].lineage_type, LineageType::Societas);
    }

    #[test]
    fn empty_houses_file_defaults_to_no_houses() {
        let file: HousesFile = serde_json::from_str("{}").unwrap();
        assert!(file.houses.is_empty());
    }

    // --- granted_selections resolver -------------------------------------

    use crate::ruleset::{Ruleset, RulesetSources};
    use crate::types::{Entity, EntityKind, RulesetRef};

    /// Point items the grant-resolver test houses reference — enough for the
    /// `validate_house_refs` integrity check (Fixed.item + Choice.options[].ref)
    /// to pass at load.
    const GRANT_ITEMS: &str = r#"[
        { "id": "virtue.self_confident", "kind": "virtue", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"] },
        { "id": "virtue.puissant_art", "kind": "virtue", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "parameters": [{ "key": "art", "type": "ref", "domain": "art" }] },
        { "id": "virtue.affinity_with_art", "kind": "virtue", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "parameters": [{ "key": "art", "type": "ref", "domain": "art" }] }
    ]"#;

    /// Houses exercising each grant kind: a fixed Virtue (Tytalus), a Choice
    /// between two Puissant Arts (Flambeau), and an Open Minor Virtue (Jerbiton).
    const GRANT_HOUSES: &str = r#"{ "houses": [
        { "id": "house.tytalus", "lineage_type": "societas",
          "grants": [ { "kind": "fixed", "item": "virtue.self_confident" } ] },
        { "id": "house.flambeau", "lineage_type": "societas",
          "grants": [ { "kind": "choice", "choice_key": "flambeau_puissant", "options": [
            { "ref": "virtue.puissant_art", "params": { "art": "art.perdo" } },
            { "ref": "virtue.puissant_art", "params": { "art": "art.ignem" } }
          ] } ] },
        { "id": "house.jerbiton", "lineage_type": "societas",
          "grants": [ { "kind": "open", "choice_key": "jerbiton_virtue",
            "constraint": { "kind": "virtue", "magnitude": "minor" } } ] }
    ] }"#;

    fn grant_ruleset() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: GRANT_ITEMS,
            type_profiles: "[]",
            abilities: None,
            arts: None,
            houses: Some(GRANT_HOUSES),
            characteristics: None,
        })
        .unwrap()
    }

    fn magus_in(house: &str) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.house = Some(Id::new(house));
        entity
    }

    fn puissant_art(art: &str) -> Selection {
        Selection::with_params(
            Id::new("virtue.puissant_art"),
            BTreeMap::from([("art".to_string(), Id::new(art))]),
        )
    }

    #[test]
    fn no_house_grants_nothing() {
        let rs = grant_ruleset();
        let mut entity = magus_in("house.tytalus");
        entity.house = None;
        assert!(granted_selections(&entity, &rs).is_empty());
    }

    #[test]
    fn unknown_house_grants_nothing() {
        let rs = grant_ruleset();
        let entity = magus_in("house.does_not_exist");
        assert!(granted_selections(&entity, &rs).is_empty());
    }

    #[test]
    fn fixed_grant_emits_the_item() {
        let rs = grant_ruleset();
        let entity = magus_in("house.tytalus");
        assert_eq!(
            granted_selections(&entity, &rs),
            vec![Selection::new(Id::new("virtue.self_confident"))]
        );
    }

    #[test]
    fn choice_grant_emits_the_pick_when_it_is_an_option() {
        let rs = grant_ruleset();
        let mut entity = magus_in("house.flambeau");
        entity
            .house_choices
            .insert("flambeau_puissant".to_string(), puissant_art("art.ignem"));
        assert_eq!(
            granted_selections(&entity, &rs),
            vec![puissant_art("art.ignem")]
        );
    }

    #[test]
    fn choice_grant_emits_nothing_when_pick_absent() {
        let rs = grant_ruleset();
        let entity = magus_in("house.flambeau");
        assert!(granted_selections(&entity, &rs).is_empty());
    }

    #[test]
    fn choice_grant_emits_nothing_when_pick_not_an_option() {
        let rs = grant_ruleset();
        let mut entity = magus_in("house.flambeau");
        // Puissant Creo is not one of the offered options (Perdo / Ignem).
        entity
            .house_choices
            .insert("flambeau_puissant".to_string(), puissant_art("art.creo"));
        assert!(granted_selections(&entity, &rs).is_empty());
    }

    #[test]
    fn open_grant_emits_the_pick_verbatim() {
        let rs = grant_ruleset();
        let mut entity = magus_in("house.jerbiton");
        let pick = Selection::new(Id::new("virtue.affinity_with_art"));
        entity
            .house_choices
            .insert("jerbiton_virtue".to_string(), pick.clone());
        assert_eq!(granted_selections(&entity, &rs), vec![pick]);
    }

    #[test]
    fn open_grant_emits_nothing_when_pick_absent() {
        let rs = grant_ruleset();
        let entity = magus_in("house.jerbiton");
        assert!(granted_selections(&entity, &rs).is_empty());
    }
}
