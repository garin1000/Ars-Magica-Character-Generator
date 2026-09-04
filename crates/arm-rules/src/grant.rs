//! Data-driven free-selection grants, shared by any type-linked profile that
//! confers Virtues/Flaws at character creation.
//!
//! A [`Grant`] describes *what* a profile gives — a fixed Virtue, a choice
//! between options, or an open player-chosen Virtue/Flaw constrained by
//! category/magnitude. The Hermetic [`crate::house::House`] and the
//! [`crate::mythic_companion::MythicCompanionType`] both carry a `Vec<Grant>`;
//! the engine resolves those grants into ordinary [`Selection`]s at evaluation
//! time via [`resolve_grants`] (see [`crate::effective`]). The save stores only
//! the profile choice plus the player's picks, never the resolved free-Virtue
//! rows (the "saves store choices, not resolved values" invariant).
//!
//! Extracted from the House machinery so mythic-companion types reuse the exact
//! same grant model rather than duplicating it.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::ruleset::Ruleset;
use crate::types::{Id, ItemKind, Magnitude, Selection};

/// A constraint on a player-chosen open grant: the kind of item, an optional
/// magnitude, and category allow/deny lists. Purely declarative — enforced in
/// [`crate::validation`], with no profile id hardcoded.
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

/// One thing a profile grants at creation.
///
/// Internally tagged on `kind` so each grant is a self-describing object in JSON
/// (`{ "kind": "fixed", "item": … }`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Grant {
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
        /// Stable key the entity's choices map uses for the pick.
        choice_key: String,
        /// The selectable options.
        options: Vec<Selection>,
    },
    /// An open, player-chosen Virtue/Flaw constrained by `constraint`, e.g.
    /// Jerbiton's free Minor Virtue, or Ex Miscellanea's Major non-Hermetic
    /// Virtue. The pick is a full [`Selection`] stored under `choice_key`.
    Open {
        /// Stable key the entity's choices map uses for the pick.
        choice_key: String,
        /// What the open pick must satisfy.
        constraint: GrantConstraint,
    },
}

/// Resolves a list of grants against the player's stored `choice_key` picks into
/// the free-Virtue [`Selection`] rows they contribute. The single source of the
/// derived grant model; every effect / prerequisite consumer folds this list
/// into `entity.selections` (see [`crate::effective`]), while balance and caps
/// stay on the bought selections alone so grants are free and uncapped.
///
/// Resolution per grant kind:
/// - **Fixed** → the granted item with its fixed params.
/// - **Choice** → the player's pick keyed by `choice_key`, but only if it is one
///   of the offered `options`; an absent or off-menu pick emits nothing (the
///   discrepancy surfaces as a validation error, not here).
/// - **Open** → the player's pick keyed by `choice_key`, verbatim; eligibility
///   against the `constraint` is checked in validation, not here.
pub fn resolve_grants(grants: &[Grant], choices: &BTreeMap<String, Selection>) -> Vec<Selection> {
    grants
        .iter()
        .filter_map(|grant| resolve_grant(grant, choices))
        .collect()
}

/// Resolves one grant to the [`Selection`] it contributes, if any.
fn resolve_grant(grant: &Grant, choices: &BTreeMap<String, Selection>) -> Option<Selection> {
    match grant {
        Grant::Fixed { item, params } => Some(Selection::with_params(item.clone(), params.clone())),
        Grant::Choice {
            choice_key,
            options,
        } => {
            let pick = choices.get(choice_key)?;
            options.contains(pick).then(|| pick.clone())
        }
        Grant::Open { choice_key, .. } => choices.get(choice_key).cloned(),
    }
}

/// Whether an Open grant's pick satisfies its constraint: the picked item must
/// resolve and match the required kind, the magnitude (when the constraint fixes
/// one), and the category allow/deny lists, and it must not demand a House other
/// than `house` — the character's own. An unresolvable pick fails; it cannot
/// satisfy anything.
///
/// Both category lists are matched against *every* category the item carries:
/// `require_categories` needs a non-empty intersection, `forbid_categories` an
/// empty one. So a descriptor's secondary category both admits a pick and rules
/// one out, which is what "the item is of that category" means in the rulebook.
///
/// The House check exists because an open grant's pick is *never* prerequisite-
/// checked: `validate_prerequisites` walks `entity.selections`, and a grant pick
/// lives in `house_choices`/`mythic_choices`/`warping_choices` instead. Without
/// this, a Jerbiton magus could take Heartbeast — a Virtue whose own descriptor
/// makes its bearer a Bjornaer — through the House's free-Minor-Virtue menu with
/// no complaint at all. Only [`Prereq::House`] is consulted (see
/// [`Prereq::conflicts_with_house`]); a `Has`/`AbilityMin`/… prerequisite stays
/// out of it deliberately, since those resolve as the build progresses.
pub fn open_pick_satisfies(
    pick: &Selection,
    constraint: &GrantConstraint,
    ruleset: &Ruleset,
    house: Option<&Id>,
) -> bool {
    let Some(item) = ruleset.point_items.get(&pick.item_ref) else {
        return false;
    };
    if item.kind != constraint.kind {
        return false;
    }
    if let Some(magnitude) = constraint.magnitude
        && item.magnitude != magnitude
    {
        return false;
    }
    if !constraint.require_categories.is_empty()
        && !item.any_category_in(&constraint.require_categories)
    {
        return false;
    }
    if item.any_category_in(&constraint.forbid_categories) {
        return false;
    }
    if item
        .prerequisites
        .as_ref()
        .is_some_and(|p| p.conflicts_with_house(house))
    {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fixed_grant_roundtrips() {
        let json = r#"{ "kind": "fixed", "item": "virtue.self_confident" }"#;
        let grant: Grant = serde_json::from_str(json).unwrap();
        assert_eq!(
            grant,
            Grant::Fixed {
                item: Id::new("virtue.self_confident"),
                params: BTreeMap::new(),
            }
        );
        let back = serde_json::to_string(&grant).unwrap();
        assert_eq!(serde_json::from_str::<Grant>(&back).unwrap(), grant);
    }

    #[test]
    fn fixed_grant_carries_params() {
        let json = r#"{ "kind": "fixed", "item": "virtue.minor_magical_focus",
                        "params": { "focus": "certamen" } }"#;
        let Grant::Fixed { item, params } = serde_json::from_str(json).unwrap() else {
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
        let Grant::Choice {
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
        let Grant::Open {
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

    fn puissant_art(art: &str) -> Selection {
        Selection::with_params(
            Id::new("virtue.puissant_art"),
            BTreeMap::from([("art".to_string(), Id::new(art))]),
        )
    }

    #[test]
    fn resolve_grants_emits_fixed_and_valid_picks_only() {
        let grants = vec![
            Grant::Fixed {
                item: Id::new("virtue.self_confident"),
                params: BTreeMap::new(),
            },
            Grant::Choice {
                choice_key: "puissant".to_string(),
                options: vec![puissant_art("art.perdo"), puissant_art("art.ignem")],
            },
            Grant::Open {
                choice_key: "open".to_string(),
                constraint: GrantConstraint {
                    kind: ItemKind::Virtue,
                    magnitude: None,
                    require_categories: BTreeSet::new(),
                    forbid_categories: BTreeSet::new(),
                },
            },
        ];
        let mut choices = BTreeMap::new();
        choices.insert("puissant".to_string(), puissant_art("art.ignem"));
        choices.insert(
            "open".to_string(),
            Selection::new(Id::new("virtue.affinity")),
        );

        assert_eq!(
            resolve_grants(&grants, &choices),
            vec![
                Selection::new(Id::new("virtue.self_confident")),
                puissant_art("art.ignem"),
                Selection::new(Id::new("virtue.affinity")),
            ]
        );
    }

    #[test]
    fn resolve_grants_drops_absent_and_off_menu_choice_picks() {
        let grants = vec![Grant::Choice {
            choice_key: "puissant".to_string(),
            options: vec![puissant_art("art.perdo"), puissant_art("art.ignem")],
        }];
        // Absent pick → nothing.
        assert!(resolve_grants(&grants, &BTreeMap::new()).is_empty());
        // Off-menu pick (Creo not offered) → nothing.
        let mut choices = BTreeMap::new();
        choices.insert("puissant".to_string(), puissant_art("art.creo"));
        assert!(resolve_grants(&grants, &choices).is_empty());
    }
}
