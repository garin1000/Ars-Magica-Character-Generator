//! D42/D70/D74 (`docs/vf-audit/decisions.md`; `tmp/d42-handover.md`): resolves
//! a Supernatural entry's realm association — computed, never stored, exactly
//! as every other `effective/*` score.
//!
//! [`resolve_realm`] is pure and never panics, even on a garbage override
//! value (a hand-edited or otherwise crafted save, CLAUDE.md's trust
//! boundary): an override that does not parse as a [`Realm`], or a
//! [`crate::types::RealmAssociation::Subset`] override naming a realm outside
//! the allowed list, is simply treated as absent here and the resolver falls
//! through the rest of the chain. Reporting either condition to the player is
//! `validation::realm::validate_realm_associations`'s job, not this
//! function's — see its doc comment for the two codes.

use crate::types::{PointItem, Realm, RealmAssociation, Selection, SelectionParamValue};

/// The selection param key an override is stored under. Deliberately not
/// `"realm"`, which `flaw.bound_to_realm`, `flaw.realm_stigmatic`,
/// `flaw.necessary_realm_aura_for_ability` and `virtue.folk_magic` already use
/// for a different (or, for Folk Magic, the same) realm-shaped value.
pub const REALM_OVERRIDE_PARAM_KEY: &str = "association";

/// A non-blocking condition [`resolve_realm`] detected while resolving one
/// entry's realm. Turned into a `ValidationIssue` by
/// `validation::realm::validate_realm_associations`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealmWarning {
    /// The override differs from a [`crate::types::RealmAssociation::Default`]
    /// entry's stated default.
    ChangedDefault,
    /// A [`crate::types::RealmAssociation::Subset`] entry has no override and
    /// no concept realm inside its subset, so the resolver could not honestly
    /// fall back to [`Realm::Magic`].
    UnansweredSubset,
}

/// The outcome of resolving one Supernatural entry's realm association.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedRealm {
    pub realm: Realm,
    pub warning: Option<RealmWarning>,
}

/// Whether `item`, as `selection` names it, carries a realm association at
/// all — Tainted items (ArMDE:3000), or items currently read as the
/// `supernatural` category ([`PointItem::categories_for`], so a
/// multi-category entry like Sufi only qualifies when `taken_as` that
/// category). Used to admit [`REALM_OVERRIDE_PARAM_KEY`] as a legal (but
/// never required) selection param key.
///
/// Deliberately NOT `|| item.realm_association.is_some()`: every shipped
/// entry that carries one is already `supernatural`-categoried (or Tainted),
/// so that clause would only ever matter for Sufi — and there it would be
/// WRONG, forcing the association to apply even `taken_as` Social Status,
/// which the book never associates with a realm at all.
pub fn item_has_realm_association(item: &PointItem, selection: &Selection) -> bool {
    item.tainted
        || item
            .categories_for(&selection.params)
            .iter()
            .any(|c| c == "supernatural")
}

/// Reads [`REALM_OVERRIDE_PARAM_KEY`] off `selection` and parses it as a
/// [`Realm`]. `None` for both "no override stored" and "stored, but not a
/// realm id at all" — the latter a crafted-save concern this function stays
/// silent about by design; see the module doc comment.
fn override_realm(selection: &Selection) -> Option<Realm> {
    selection
        .params
        .get(REALM_OVERRIDE_PARAM_KEY)
        .and_then(SelectionParamValue::as_single)
        .and_then(Realm::from_id)
}

/// Resolves the [`Realm`] `selection` (of `item`) is associated with, given
/// the entity's `concept_realm`. Pure; no I/O; never panics (see the module
/// doc comment).
///
/// The chain (D42/D70 Q-X6-5, D74):
/// 1. `tainted: true` (ArMDE:3000) → Infernal, no warning — checked first, so
///    the general Tainted rule cannot disagree with a stale/never-added
///    [`RealmAssociation`] on the same item.
/// 2. [`RealmAssociation::Fixed`] → that realm; an override is ignored,
///    silently (D74 Q4 — read-only in the UI, and this keeps a save from
///    before the entry was fixed loading clean).
/// 3. [`RealmAssociation::Default`] → the override if present (any realm,
///    `ChangedDefault` warning when it differs from the stated default), else
///    the default with no warning.
/// 4. [`RealmAssociation::Subset`] → the override if present and a member;
///    else the concept realm if a member; else the subset's first (canonical)
///    member with `UnansweredSubset` — never a silent `Realm::Magic` fallback,
///    which would be illegal here. An override present but NOT a member is
///    treated the same as no override (the validator reports it separately,
///    `CODE_REALM_OVERRIDE_INVALID`).
/// 5. [`RealmAssociation::FromParam`] → the override if present, else the
///    named parameter's own value, no warning either way (D74 Q3). Falls
///    through to step 6 if the named parameter is itself missing or
///    unparseable (a crafted save; never a panic).
/// 6. No association at all → override, else `concept_realm`, else
///    [`Realm::Magic`] (ArMDE:2960) — never a warning.
pub fn resolve_realm(
    item: &PointItem,
    selection: &Selection,
    concept_realm: Option<Realm>,
) -> ResolvedRealm {
    let plain_chain = || ResolvedRealm {
        realm: override_realm(selection)
            .or(concept_realm)
            .unwrap_or(Realm::Magic),
        warning: None,
    };

    if item.tainted {
        return ResolvedRealm {
            realm: Realm::Infernal,
            warning: None,
        };
    }

    match &item.realm_association {
        None => plain_chain(),
        Some(RealmAssociation::Fixed { realm }) => ResolvedRealm {
            realm: *realm,
            warning: None,
        },
        Some(RealmAssociation::Default { realm: default }) => match override_realm(selection) {
            Some(over) if over != *default => ResolvedRealm {
                realm: over,
                warning: Some(RealmWarning::ChangedDefault),
            },
            Some(over) => ResolvedRealm {
                realm: over,
                warning: None,
            },
            None => ResolvedRealm {
                realm: *default,
                warning: None,
            },
        },
        Some(RealmAssociation::Subset { realms }) => {
            if let Some(over) = override_realm(selection)
                && realms.contains(&over)
            {
                return ResolvedRealm {
                    realm: over,
                    warning: None,
                };
            }
            if let Some(concept) = concept_realm
                && realms.contains(&concept)
            {
                return ResolvedRealm {
                    realm: concept,
                    warning: None,
                };
            }
            ResolvedRealm {
                // `BTreeSet<Realm>` iterates in `Realm`'s own `Ord` sequence
                // (Magic, Faerie, Divine, Infernal), so the first member is
                // deterministic. Load-time integrity
                // (`ruleset::integrity::validate_realm_association`) rejects
                // an empty `Subset` list in the shipped rules, but this stays
                // defensive rather than `.expect()`-panicking on one anyway —
                // `rules/` is CLAUDE.md's other trust boundary, loaded from a
                // plain directory beside the binary, not just the save file.
                realm: realms.iter().next().copied().unwrap_or(Realm::Magic),
                warning: Some(RealmWarning::UnansweredSubset),
            }
        }
        Some(RealmAssociation::FromParam { key }) => {
            if let Some(over) = override_realm(selection) {
                return ResolvedRealm {
                    realm: over,
                    warning: None,
                };
            }
            let named = selection
                .params
                .get(key)
                .and_then(SelectionParamValue::as_single)
                .and_then(Realm::from_id);
            match named {
                Some(realm) => ResolvedRealm {
                    realm,
                    warning: None,
                },
                None => plain_chain(),
            }
        }
    }
}
