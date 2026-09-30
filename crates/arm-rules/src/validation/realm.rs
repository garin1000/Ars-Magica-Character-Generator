//! D42/D70/D74 (`docs/vf-audit/decisions.md`; `tmp/d42-handover.md`): turns
//! [`crate::effective::resolve_realm`]'s warning into a `ValidationIssue`, and
//! flags an `association` override that is either not a realm id at all, or
//! (for a [`RealmAssociation::Subset`] entry) a realm outside the allowed
//! list.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;
use crate::effective::{
    REALM_OVERRIDE_PARAM_KEY, RealmWarning, item_has_realm_association, resolve_realm,
};
use crate::types::RealmAssociation;

pub(crate) fn validate_realm_associations(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        if !item_has_realm_association(item, selection) {
            continue;
        }

        // A garbage override (not a `realm.<slug>` id at all) is a crafted-
        // save concern: `resolve_realm` silently ignores it rather than
        // panicking, so it is reported here, on the same
        // `unknown_param_value` code every other unresolved parameter value
        // uses — `association` is legal (see `validate_selection_parameters`)
        // but is not a declared `ParameterDef`, so nothing else checks it.
        if let Some(raw) = selection.params.get(REALM_OVERRIDE_PARAM_KEY)
            && !param_value_is_blank(raw)
            && let Some(raw) = raw.as_single()
        {
            if Realm::from_id(raw).is_none() {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_UNKNOWN_PARAM_VALUE,
                    CreationPhase::VirtuesFlaws,
                    args([
                        ("item", selection.item_ref.to_string()),
                        ("key", REALM_OVERRIDE_PARAM_KEY.to_string()),
                        ("value", raw.to_string()),
                        ("domain", ParameterDomain::Realm.to_string()),
                    ]),
                    Some(selection.item_ref.clone()),
                ));
                continue;
            }
            // D74 Q2: a well-formed override outside a Subset entry's allowed
            // list is a distinct error from the general "unknown value" one
            // above — the realm itself is real, just not legal HERE.
            if let Some(RealmAssociation::Subset { realms }) = &item.realm_association
                && let Some(over) = Realm::from_id(raw)
                && !realms.contains(&over)
            {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_REALM_OVERRIDE_INVALID,
                    CreationPhase::VirtuesFlaws,
                    args([
                        ("item", selection.item_ref.to_string()),
                        ("value", raw.to_string()),
                    ]),
                    Some(selection.item_ref.clone()),
                ));
                continue;
            }
        }

        let resolved = resolve_realm(item, selection, entity.concept_realm);
        match resolved.warning {
            None => {}
            Some(RealmWarning::ChangedDefault) => {
                issues.push(ValidationIssue::warning(
                    ValidationIssue::CODE_REALM_CHANGED_DEFAULT,
                    CreationPhase::VirtuesFlaws,
                    args([("item", selection.item_ref.to_string())]),
                    Some(selection.item_ref.clone()),
                ));
            }
            Some(RealmWarning::UnansweredSubset) => {
                issues.push(ValidationIssue::warning(
                    ValidationIssue::CODE_REALM_UNANSWERED_SUBSET,
                    CreationPhase::VirtuesFlaws,
                    args([("item", selection.item_ref.to_string())]),
                    Some(selection.item_ref.clone()),
                ));
            }
        }
    }
}
