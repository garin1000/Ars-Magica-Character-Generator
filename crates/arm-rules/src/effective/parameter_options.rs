//! `AbilityParameterOptions`: the engine-built option list for an Ability's
//! parameter picker (design § 6.3,
//! `docs/vf-audit/design-cv-catalogued-values.md`). Split out because it
//! draws on both the catalogue (ruleset-wide) and the character's own Bound/
//! Link sources ([`ability_authorizations`], [`resolve_link`]).

use super::*;
use crate::ability::Ability;

/// One once-only Virtue/Flaw item's own parameter, `Bound` to THIS Ability in
/// a currently-active effect the character holds (design § 6.3) — the SAME
/// `(item, param)` pair a bought [`AbilityParameterValue::Linked`] value would
/// name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LinkTarget {
    /// The declaring item's id (e.g. `virtue.craft_guild_training`).
    pub item: Id,
    /// The declaring item's own parameter key.
    pub param: String,
}

/// The engine-built parameter-picker options for one catalogued-or-linkable
/// Ability (design § 6.3) — "the UI must not derive the option list itself."
///
/// A transient, freshly-computed IPC payload (like [`AbilityBonus`]/
/// [`RestrictedXpPool`]), never a save file — so, unlike `Entity`'s own
/// fields, nothing here is `skip_serializing_if`: the TS mirror's fields are
/// all required, matching what every call always emits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityParameterOptions {
    /// The ability these options are for.
    pub ability: Id,
    /// This ability's catalogue values, in catalogue order (empty if
    /// uncatalogued). Ids only — the UI localizes the name through the i18n
    /// it already has; the engine never resolves a display name here.
    pub catalogued: Vec<Id>,
    /// Every once-only item the character holds whose own parameter is
    /// `Bound` to this ability and resolves UNAMBIGUOUSLY (design § 4.1 — an
    /// ambiguous source is never offered as a link target, since offering it
    /// would let the player link to a value that cannot itself resolve).
    pub linked: Vec<LinkTarget>,
    /// Design § 11 item 2 (Norbert's decision, replacing an earlier "static,
    /// always shown" recommendation): set only when a bought `Text` value on
    /// this ability leaves a Literal or Bound instance — from one of the
    /// character's own items — unmet. The free-text choice costs the player
    /// something in that case (a pool/authorization it could otherwise
    /// satisfy); otherwise no hint is shown, so an uncatalogued/unlinked
    /// ability (Craft, Area Lore, absent any Bound source) carries no
    /// permanent noise. The engine decides this and surfaces it here; the UI
    /// never queries the ruleset itself.
    pub hint: bool,
}

/// Builds one [`AbilityParameterOptions`] entry per ability that is either
/// catalogued or offers at least one link target to THIS character — an
/// ability with neither is skipped, since its picker stays a plain free-text
/// input (design § 6.1) with nothing for this struct to add.
///
/// Iterates [`Ruleset::abilities`] (a `BTreeMap`, so already id-ordered) for a
/// deterministic outer order; `linked` is sorted (via [`LinkTarget`]'s derived
/// `Ord`) and deduplicated for the same reason.
pub fn ability_parameter_options(
    entity: &Entity,
    ruleset: &Ruleset,
) -> Vec<AbilityParameterOptions> {
    let (authorized, _categories) = ability_authorizations(entity, ruleset);

    let mut options = Vec::new();
    for ability in ruleset.abilities() {
        let catalogued = catalogued_values(ability, ruleset);

        let mut linked: Vec<LinkTarget> = authorized
            .iter()
            .filter(|a| a.ability == ability.id && !a.ambiguous)
            .filter_map(|a| a.bound_source.clone())
            .map(|(item, param)| LinkTarget { item, param })
            .collect();
        linked.sort();
        linked.dedup();

        if catalogued.is_empty() && linked.is_empty() {
            continue;
        }

        let hint = ability_needs_hint(&ability.id, entity, &authorized);

        options.push(AbilityParameterOptions {
            ability: ability.id.clone(),
            catalogued,
            linked,
            hint,
        });
    }
    options
}

/// This ability's catalogue values, in catalogue order (empty if uncatalogued
/// or if its own `parameter` key names no shipped catalogue) — a property of
/// the ruleset alone, independent of what any character holds.
fn catalogued_values(ability: &Ability, ruleset: &Ruleset) -> Vec<Id> {
    if !ability.catalogued {
        return Vec::new();
    }
    let Some(key) = ability.parameter.as_deref() else {
        return Vec::new();
    };
    let catalogue_id = Id::new(format!("catalogue.{key}"));
    ruleset
        .parameter_catalogues()
        .get(&catalogue_id)
        .map(|catalogue| catalogue.values.iter().map(|v| v.id.clone()).collect())
        .unwrap_or_default()
}

/// Design § 11 item 2: true when a bought `Text` value on `ability` leaves at
/// least one SCOPED authorization entry — a `Literal` or non-ambiguous `Bound`
/// instance from one of the character's own items — unmet. An unscoped entry
/// (`instance: None`, no `bound_source`) authorizes any instance, so there is
/// nothing a Text value could fail; an ambiguous entry satisfies nothing no
/// matter what is typed, so it offers no hint-worthy target either — both are
/// excluded from the scoped set this checks against.
fn ability_needs_hint(
    ability: &Id,
    entity: &Entity,
    authorized: &BTreeSet<AuthorizedAbility>,
) -> bool {
    let scoped: Vec<&AuthorizedAbility> = authorized
        .iter()
        .filter(|a| {
            a.ability == *ability
                && !a.ambiguous
                && (a.instance.is_some() || a.bound_source.is_some())
        })
        .collect();
    if scoped.is_empty() {
        return false;
    }
    entity.ability_scores.iter().any(|score| {
        score.ability == *ability
            && matches!(&score.parameter, Some(AbilityParameterValue::Text { .. }))
            && scoped
                .iter()
                .any(|entry| !entry.covers(ability, score.parameter.as_ref()))
    })
}
