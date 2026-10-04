//! Catalogued parameter values (CV1) — closed, versioned lists of values a
//! rules-authored `ParamValue::Literal` instance may name, so that matching a
//! player's typed `AbilityScore.parameter` against one no longer depends on
//! exact-string equality in a single language (D14; see
//! `docs/vf-audit/design-cv-catalogued-values.md`).
//!
//! This module loads `rules/core/parameter_catalogues.json` (the
//! language-neutral catalogue + value ids, each with a [`SourceRef`]) and
//! validates it against the i18n name files
//! (`rules/i18n/{en,de}/parameter_catalogue.json`) independently of any single
//! active UI language — deliberately outside [`crate::ruleset::LocalizedRuleset`],
//! which only ever holds one language's text at a time (design note § 2.3,
//! § 5.3).
//!
//! **CV2 wires catalogues into [`crate::ruleset::Ruleset`] itself**
//! (`Ruleset::from_sources` parses + pre-integrity-checks
//! `RulesetSources::parameter_catalogues` via [`parse_parameter_catalogues_file`]/
//! [`parameter_catalogue_integrity_errors`] below, exactly like every other
//! catalogue file; `Ruleset::validate_integrity` cross-checks a `catalogued`
//! Ability's `parameter` key against it). [`load_parameter_catalogues`] stays a
//! public standalone convenience wrapper over the same two functions — CV1's
//! tests use it directly, and it costs nothing to keep. [`load_catalogue_names`]
//! stays standalone too, by design: it needs BOTH locales at once, which
//! `Ruleset`/`LocalizedRuleset` never carry together.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::ruleset::{IntegrityError, RulesetError};
use crate::types::{Id, SourceRef};

/// One entry in a [`Catalogue`]: a single named value (e.g. `language.latin`),
/// with its rulebook provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogueValue {
    /// Stable slug id, unique within its catalogue (`language.latin`).
    pub id: Id,
    /// Provenance into the authoritative Markdown rules source.
    pub source: SourceRef,
}

/// A closed list of values a parameterized Ability's `catalogued: true`
/// parameter may resolve against (e.g. `catalogue.language`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalogue {
    /// Stable slug id (`catalogue.language`).
    pub id: Id,
    /// This catalogue's values, canonically sorted by id.
    pub values: Vec<CatalogueValue>,
}

impl Catalogue {
    /// Looks up one of this catalogue's values by id.
    pub fn value(&self, id: &Id) -> Option<&CatalogueValue> {
        self.values.iter().find(|v| &v.id == id)
    }
}

/// On-disk shape of `rules/core/parameter_catalogues.json`.
#[derive(Debug, Clone, Default, Deserialize)]
struct ParameterCataloguesFile {
    #[serde(default)]
    catalogues: Vec<Catalogue>,
}

/// Parses `rules/core/parameter_catalogues.json` into a raw, unindexed list —
/// no integrity checks yet. Used by [`crate::ruleset::parse::parse_sources`]
/// (CV2), which accumulates every catalog's pre-integrity errors together
/// (via [`parameter_catalogue_integrity_errors`]) before failing, exactly like
/// `check_duplicate_ids` does for abilities/houses/etc. — so a raw parse here
/// must not itself reject a duplicate id; only a malformed-JSON parse error is
/// fatal at this stage.
pub(crate) fn parse_parameter_catalogues_file(json: &str) -> Result<Vec<Catalogue>, RulesetError> {
    let file: ParameterCataloguesFile =
        serde_json::from_str(json).map_err(|e| RulesetError::parse("parameter catalogues", e))?;
    Ok(file.catalogues)
}

/// The pre-integrity checks CV1 defined for the parsed (not yet indexed)
/// catalogue list (design note § 2.2/§ 7), failing loudly and naming every
/// offending id:
/// - catalogue ids unique across the file;
/// - each catalogue's `values` non-empty;
/// - value ids unique within their own catalogue;
/// - catalogues, and each catalogue's values, sorted by id (canonical
///   serialization — `CLAUDE.md` → "Canonical serialization").
///
/// Shared by [`load_parameter_catalogues`] (standalone) and
/// `Ruleset::from_sources`'s pre-integrity pipeline (CV2) — one implementation,
/// not two that could disagree.
pub(crate) fn parameter_catalogue_integrity_errors(catalogues: &[Catalogue]) -> Vec<String> {
    let mut errors = Vec::new();

    collect_duplicates(
        catalogues.iter().map(|c| &c.id),
        "parameter catalogue",
        &mut errors,
    );
    if !is_sorted_by_id(catalogues, |c| &c.id) {
        errors.push("parameter catalogues are not sorted by id".to_string());
    }

    for catalogue in catalogues {
        if catalogue.values.is_empty() {
            errors.push(format!(
                "parameter catalogue '{}' has no values",
                catalogue.id
            ));
            continue;
        }
        collect_duplicates(
            catalogue.values.iter().map(|v| &v.id),
            &format!("parameter catalogue value in '{}'", catalogue.id),
            &mut errors,
        );
        if !is_sorted_by_id(&catalogue.values, |v| &v.id) {
            errors.push(format!(
                "parameter catalogue '{}' values are not sorted by id",
                catalogue.id
            ));
        }
    }

    errors
}

/// Parses `rules/core/parameter_catalogues.json` into catalogues keyed by id,
/// validating it standalone (see [`parameter_catalogue_integrity_errors`]).
/// Public convenience wrapper over the two functions above, kept for CV1's own
/// tests and any other caller that wants a catalogue file's own integrity
/// checked in isolation, without a whole `Ruleset` around it.
pub fn load_parameter_catalogues(json: &str) -> Result<BTreeMap<Id, Catalogue>, RulesetError> {
    let catalogues = parse_parameter_catalogues_file(json)?;
    let errors = parameter_catalogue_integrity_errors(&catalogues);
    if !errors.is_empty() {
        return Err(IntegrityError::new(errors).into());
    }
    Ok(catalogues.into_iter().map(|c| (c.id.clone(), c)).collect())
}

/// Pushes a `"duplicate {label} ID: '{id}'"` error for every id seen more than
/// once, in encounter order — mirrors `ruleset.rs::collect_duplicates`
/// (kept as a separate copy since that one is private to its module).
fn collect_duplicates<'a>(
    ids: impl Iterator<Item = &'a Id>,
    label: &str,
    errors: &mut Vec<String>,
) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            errors.push(format!("duplicate {label} ID: '{id}'"));
        }
    }
}

/// Returns `true` if `items` is already sorted, strictly ascending, by the key
/// `id_of` extracts (which also implies no duplicates — checked separately for
/// a clearer, id-naming error message).
fn is_sorted_by_id<T>(items: &[T], id_of: impl Fn(&T) -> &Id) -> bool {
    items.windows(2).all(|w| id_of(&w[0]) < id_of(&w[1]))
}

/// One entry in `rules/i18n/{en,de}/parameter_catalogue.json`.
#[derive(Debug, Clone, Deserialize)]
struct CatalogueNameEntry {
    id: Id,
    name: String,
}

/// On-disk shape of `rules/i18n/{en,de}/parameter_catalogue.json`.
#[derive(Debug, Clone, Default, Deserialize)]
struct ParameterCatalogueNamesFile {
    #[serde(default)]
    names: Vec<CatalogueNameEntry>,
}

/// Parses a `parameter_catalogue.json` name file into an id → name map.
fn parse_names(json: &str) -> Result<BTreeMap<Id, String>, RulesetError> {
    let file: ParameterCatalogueNamesFile = serde_json::from_str(json)
        .map_err(|e| RulesetError::parse("parameter catalogue names", e))?;
    Ok(file.names.into_iter().map(|n| (n.id, n.name)).collect())
}

/// Parses ONE locale's `parameter_catalogue.json` into an id → name map for
/// that locale alone — unlike [`load_catalogue_names`], which deliberately
/// combines both locales for migration matching, this is what
/// `arm-app::ruleset_io::load_ruleset_from_dir` needs to merge a catalogue
/// value's display name into [`crate::ruleset::LocalizedRuleset::i18n`] for
/// the single ACTIVE UI language (design § 2.3/§ 6.4, CV7) — the same map
/// every other id's display name already lives in.
pub fn parse_catalogue_names(json: &str) -> Result<BTreeMap<Id, String>, RulesetError> {
    parse_names(json)
}

/// Folds a name for case-insensitive, trimmed comparison (design note § 2.2).
/// `pub(crate)`: also the SAME fold [`crate::migration`]'s catalogue-matching
/// load fold and `effective::xp`'s live Bound/Link rule-2 content match use —
/// one place that knows "does this text spell out this value's name",
/// design-cv-catalogued-values.md § 4/§ 5.3, never two independent
/// implementations that could disagree.
pub(crate) fn fold_name(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Folds a free-text parameter value for comparison (R2, D83.2): case-folded,
/// trimmed, and every inner run of whitespace collapsed to one space, so
/// `"Fire"`, `" fire  "` and `"hot   fire"`/`"Hot Fire"` compare as the value
/// the player meant. Stricter than [`fold_name`] (which keeps inner runs) and
/// deliberately separate from it, so the catalogue-matching callers of that
/// fold are untouched. Only for text the player TYPES — never for an id.
pub(crate) fn fold_free_text(text: &str) -> String {
    fold_name(&text.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Whether a bought instance of `ability` (its `parameter`) is the instance
/// `wanted` — the one matching rule every "is this language X" check shares
/// (L2, try-out finding 8):
///
/// - a `Catalogued` value is `wanted` when its id is `wanted`;
/// - for a **catalogued** Ability, `Text` is `wanted` when it spells one of the
///   value's display names in any locale the app attached
///   ([`crate::Ruleset::with_catalogue_names`]), trimmed and case-folded with the
///   load fold's own [`fold_name`]. Text spelling the id itself never is: an id is
///   not a name;
/// - for an uncatalogued Ability, `wanted` is free text, so `Text` matches it
///   under the same fold;
/// - `Linked` and an absent value never match.
pub(crate) fn instance_is(
    ruleset: &crate::Ruleset,
    ability: &Id,
    parameter: Option<&crate::types::AbilityParameterValue>,
    wanted: &str,
) -> bool {
    use crate::types::AbilityParameterValue;
    let catalogued = ruleset
        .ability(ability)
        .and_then(crate::Ability::catalogue_id)
        .is_some();
    match parameter {
        Some(AbilityParameterValue::Catalogued { id }) => id.as_str() == wanted,
        Some(AbilityParameterValue::Text { text }) if catalogued => ruleset
            .catalogue_value_names(&Id::new(wanted))
            .iter()
            .any(|name| fold_name(name) == fold_name(text)),
        Some(AbilityParameterValue::Text { text }) => fold_name(text) == fold_name(wanted),
        Some(AbilityParameterValue::Linked { .. }) | None => false,
    }
}

/// What a typed parameter value names on one Ability (N4a, D84.1): the catalogue
/// value it spells, or — naming none — its own folded text. Resolved once, with the
/// ruleset, by [`resolve_typed_instance`]; matched afterwards without it
/// ([`Self::is_satisfied_by`]), which is what lets a pool's
/// `effective/xp.rs::AbilityInstanceRef` carry it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedInstance {
    /// The text spells a display name of this catalogue value. `names` holds every
    /// name the ruleset knows for it, in any locale, folded with [`fold_name`].
    Catalogued {
        /// The catalogue value (`language.arabic`).
        id: Id,
        /// Its names, folded.
        names: Vec<String>,
    },
    /// The text names no value of the Ability's catalogue (or the Ability has none).
    Text {
        /// The text, folded with [`fold_name`].
        folded: String,
    },
}

impl ResolvedInstance {
    /// Whether a bought `parameter` is this instance — the [`instance_is`] rule:
    /// `Catalogued` by id; `Text` when it spells one of the value's names, or, for
    /// text naming no value, the same text, all under [`fold_name`]. Free text never
    /// matches a `Catalogued` row, and `Linked` or an absent value never matches.
    pub(crate) fn is_satisfied_by(
        &self,
        parameter: Option<&crate::types::AbilityParameterValue>,
    ) -> bool {
        use crate::types::AbilityParameterValue;
        match (self, parameter) {
            (
                Self::Catalogued { id, .. },
                Some(AbilityParameterValue::Catalogued { id: bought }),
            ) => id == bought,
            (Self::Catalogued { names, .. }, Some(AbilityParameterValue::Text { text })) => {
                names.contains(&fold_name(text))
            }
            (Self::Text { folded }, Some(AbilityParameterValue::Text { text })) => {
                fold_name(text) == *folded
            }
            _ => false,
        }
    }

    /// The value to store for this instance: the catalogue value, or the player's
    /// own `typed` spelling when it names none.
    pub(crate) fn to_parameter(&self, typed: &str) -> crate::types::AbilityParameterValue {
        use crate::types::AbilityParameterValue;
        match self {
            Self::Catalogued { id, .. } => AbilityParameterValue::Catalogued { id: id.clone() },
            Self::Text { .. } => AbilityParameterValue::text(typed),
        }
    }
}

/// Resolves typed `text` against `ability`'s own catalogue (N4a, D84.1): the value
/// whose display name, in any locale the app attached
/// ([`crate::Ruleset::with_catalogue_names`]), it spells under [`fold_name`] — the
/// rule [`instance_is`] and the load fold (`migration.rs::fold_catalogue_matching`)
/// use — else the folded text itself. Every check that compares a typed language
/// with a bought one goes through this: childhood's native-language pool, the
/// "native language not bought" warning, and a Sample Childhood's slot checks. A
/// ruleset without names attached resolves nothing, so it matches text with text.
pub(crate) fn resolve_typed_instance(
    ruleset: &crate::Ruleset,
    ability: &Id,
    text: &str,
) -> ResolvedInstance {
    let folded = fold_name(text);
    let catalogue = ruleset
        .ability(ability)
        .and_then(crate::Ability::catalogue_id)
        .and_then(|id| ruleset.catalogue(&id));
    let named = |value: &&CatalogueValue| {
        ruleset
            .catalogue_value_names(&value.id)
            .iter()
            .any(|name| fold_name(name) == folded)
    };
    match catalogue.and_then(|catalogue| catalogue.values.iter().find(named)) {
        Some(value) => catalogued_instance(ruleset, &value.id),
        None => ResolvedInstance::Text { folded },
    }
}

/// The plan's native language as the instance it names (N4b): a catalogue value
/// is that value, with every name the ruleset knows for it; typed text resolves as
/// [`resolve_typed_instance`] resolves it.
pub(crate) fn resolve_native_language(
    ruleset: &crate::Ruleset,
    ability: &Id,
    language: &crate::life_stage::NativeLanguage,
) -> ResolvedInstance {
    use crate::life_stage::NativeLanguage;
    match language {
        NativeLanguage::Catalogued { id } => catalogued_instance(ruleset, id),
        NativeLanguage::Text { text } => resolve_typed_instance(ruleset, ability, text),
    }
}

/// The catalogue value `id` with its names in every locale, folded.
fn catalogued_instance(ruleset: &crate::Ruleset, id: &Id) -> ResolvedInstance {
    ResolvedInstance::Catalogued {
        id: id.clone(),
        names: ruleset
            .catalogue_value_names(id)
            .iter()
            .map(|name| fold_name(name))
            .collect(),
    }
}

/// Loads and validates both locales' catalogue value names against an
/// already-loaded set of catalogues (design note § 2.2/§ 7):
///
/// - every catalogue value named in `catalogues` must have an `en` **and** a
///   `de` name — missing either fails loudly with the offending id;
/// - within one catalogue, no two values may collide under trimmed,
///   case-folded comparison across the union of their `en`/`de` names —
///   rejects the second value, citing both colliding ids.
///
/// Returns, per catalogue value id, every known display name (English then
/// German) — the same shape the design note's CV4 `catalogue_names` migration
/// parameter needs (§ 5.6).
pub fn load_catalogue_names(
    catalogues: &BTreeMap<Id, Catalogue>,
    en_names_json: &str,
    de_names_json: &str,
) -> Result<BTreeMap<Id, Vec<String>>, RulesetError> {
    let en_names = parse_names(en_names_json)?;
    let de_names = parse_names(de_names_json)?;

    let mut errors = Vec::new();
    let mut result = BTreeMap::new();

    for catalogue in catalogues.values() {
        // Folded name -> the first value id that produced it, within this
        // catalogue only (a name may repeat across different catalogues
        // without ambiguity, since a Literal/Bound match is always scoped to
        // one ability's own catalogue).
        let mut seen_names: BTreeMap<String, Id> = BTreeMap::new();

        for value in &catalogue.values {
            let en = en_names.get(&value.id);
            let de = de_names.get(&value.id);

            if en.is_none() {
                errors.push(format!(
                    "parameter catalogue value '{}' has no English name",
                    value.id
                ));
            }
            if de.is_none() {
                errors.push(format!(
                    "parameter catalogue value '{}' has no German name",
                    value.id
                ));
            }

            let mut names = Vec::new();
            for name in [en, de].into_iter().flatten() {
                names.push(name.clone());
                let folded = fold_name(name);
                match seen_names.get(&folded) {
                    Some(other_id) if other_id != &value.id => {
                        errors.push(format!(
                            "parameter catalogue '{}' values '{}' and '{}' collide under \
                             case-folded name comparison ('{name}')",
                            catalogue.id, other_id, value.id
                        ));
                    }
                    _ => {
                        seen_names.insert(folded, value.id.clone());
                    }
                }
            }
            result.insert(value.id.clone(), names);
        }
    }

    if !errors.is_empty() {
        return Err(IntegrityError::new(errors).into());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_free_text_ignores_case_padding_and_inner_whitespace_runs() {
        assert_eq!(fold_free_text("  Hot \t  FIRE \n"), "hot fire");
    }

    #[test]
    fn fold_free_text_keeps_different_words_different() {
        assert_ne!(fold_free_text("hot fire"), fold_free_text("hotfire"));
    }
}
