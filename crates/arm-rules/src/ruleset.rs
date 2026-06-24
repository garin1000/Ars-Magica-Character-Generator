use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::ability::{Ability, AdvancementTable};
use crate::characteristics::CharacteristicRules;
use crate::types::{EntityTypeProfile, I18nEntry, Id, ItemKind, PointItem, Prereq, RulesetRef};

/// Top-level container for all loaded game mechanics.
///
/// Built only via [`Ruleset::from_json`] (which validates referential
/// integrity), never field-by-field by callers; `PartialEq` is provided for
/// tests and diffing.
///
/// # JSON shape
///
/// Serialized whole as a Tauri command return value, so the frontend binds
/// directly to these top-level field names — they are a **stable public
/// contract**; renaming any silently breaks the TS consumer with no Rust error.
/// The data maps (`point_items`, `type_profiles`, `abilities`,
/// `characteristic_rules`) serialize as JSON objects keyed by id; `advancement`
/// is a bare array (see [`AdvancementTable`]):
///
/// ```json
/// {
///   "id": "arm5-core",
///   "version": "2024.1",
///   "point_items": { "virtue.x": { /* PointItem */ } },
///   "type_profiles": { "magus": { /* EntityTypeProfile */ } },
///   "abilities": { "ability.awareness": { /* Ability */ } },
///   "advancement": [ { "score": 1, "total_xp": 5 } ],
///   "characteristic_rules": { /* CharacteristicRules */ }
/// }
/// ```
///
/// The golden test `serialized_field_names_are_the_stable_contract` pins these
/// names so an accidental rename fails the build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ruleset {
    /// Stable ruleset identifier (matches [`RulesetRef::id`]). Serialized name
    /// is a stable public contract the frontend binds to.
    pub id: Id,
    /// Ruleset version string. Serialized name is a stable public contract.
    pub version: String,
    /// All point items keyed by their id. Serialized whole to the frontend; the
    /// `point_items` field name is a stable public contract.
    pub(crate) point_items: BTreeMap<Id, PointItem>,
    /// All entity type profiles keyed by their id. Serialized whole to the
    /// frontend; the `type_profiles` field name is a stable public contract.
    pub(crate) type_profiles: BTreeMap<Id, EntityTypeProfile>,
    /// All abilities keyed by their id. Defaulted so older serialized rulesets
    /// (pre-M3, no abilities) still deserialize via [`Ruleset::from_serialized`].
    /// Serialized whole to the frontend; the `abilities` field name is a stable
    /// public contract.
    #[serde(default)]
    pub(crate) abilities: BTreeMap<Id, Ability>,
    /// The Ability XP advancement table. Serialized whole to the frontend; the
    /// `advancement` field name is a stable public contract.
    #[serde(default)]
    pub(crate) advancement: AdvancementTable,
    /// The Characteristic point-buy rules (cost table + starting points), if the
    /// ruleset ships them. `None` for rulesets without a characteristics file.
    /// Serialized whole to the frontend; the `characteristic_rules` field name is
    /// a stable public contract.
    #[serde(default)]
    pub(crate) characteristic_rules: Option<CharacteristicRules>,
}

/// The set of language-neutral JSON source strings a [`Ruleset`] is built from.
///
/// A named struct rather than a growing list of positional `&str` arguments: the
/// optional inputs (`abilities`, `characteristics`) are honest `Option`s instead
/// of sentinel `""`/`"{}"` strings, and each field is named at the call site.
/// Use [`Ruleset::from_sources`].
#[derive(Debug, Clone, Copy)]
pub struct RulesetSources<'a> {
    /// Stable ruleset identifier.
    pub id: &'a str,
    /// Ruleset version string.
    pub version: &'a str,
    /// Point-items JSON (array of [`PointItem`]).
    pub point_items: &'a str,
    /// Type-profiles JSON (array of [`EntityTypeProfile`]).
    pub type_profiles: &'a str,
    /// Abilities-file JSON (`{ "advancement": [...], "abilities": [...] }`), or
    /// `None` for a ruleset without an ability registry.
    pub abilities: Option<&'a str>,
    /// Characteristic point-buy JSON (`{ "start_points", "costs" }`), or `None`
    /// for a ruleset that ships no characteristic rules.
    pub characteristics: Option<&'a str>,
}

/// A [`Ruleset`] paired with localized display text for a single language.
///
/// Built only via [`LocalizedRuleset::new`]; `PartialEq` is provided for tests.
///
/// # JSON shape
///
/// Serialized as a Tauri command return value; the `ruleset` and `i18n` field
/// names are a **stable public contract** the frontend binds to. `i18n` is a
/// JSON object keyed by id, each value an [`I18nEntry`]:
///
/// ```json
/// {
///   "ruleset": { /* Ruleset */ },
///   "i18n": { "virtue.gentle_gift": { "name": "Gentle Gift" } }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedRuleset {
    /// The language-neutral ruleset.
    pub ruleset: Ruleset,
    /// Localized text keyed by item/profile id.
    pub i18n: BTreeMap<Id, I18nEntry>,
}

/// A referential-integrity failure raised while loading a [`Ruleset`].
///
/// Carries every individual offending message; [`IntegrityError::errors`]
/// exposes them as a list, and [`std::fmt::Display`] joins them with newlines.
///
/// # JSON shape
///
/// Serialize-only; reaches the frontend embedded in a [`RulesetError`]'s
/// `errors` array (see [`RulesetError`]'s JSON shape). On its own it serializes
/// as `{ "errors": [ "...", "..." ] }` — the `errors` field name is a stable
/// public contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IntegrityError {
    /// One human-readable message per detected integrity violation.
    errors: Vec<String>,
}

impl IntegrityError {
    /// Creates an integrity error from a list of messages.
    pub fn new(errors: Vec<String>) -> Self {
        Self { errors }
    }

    /// The individual offending messages, one per violation.
    pub fn errors(&self) -> &[String] {
        &self.errors
    }
}

impl std::fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.errors.join("\n"))
    }
}

impl std::error::Error for IntegrityError {}

/// An error raised while loading a [`Ruleset`] or [`LocalizedRuleset`].
///
/// Does not leak `serde_json::Error`: the parse message is captured as a
/// `String` so the public surface stays stable. Use [`RulesetError::kind`]
/// for a machine-stable discriminant.
///
/// # JSON shape
///
/// Serialized as a tagged object whose `kind` discriminates the variant; this
/// is a stable public contract the frontend binds to:
///
/// ```json
/// { "kind": "parse",     "source": "type profiles", "message": "..." }
/// { "kind": "integrity", "errors": [ "...", "..." ] }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesetError {
    /// JSON failed to parse or deserialize. `source` names which input failed
    /// (e.g. `"point items"`, `"type profiles"`, `"abilities"`,
    /// `"characteristics"`, `"i18n"`) so a UI can point at the offending file;
    /// `message` is the formatted parse message.
    Parse {
        /// Identifier of the input that failed to parse.
        source: String,
        /// Formatted parse message (the underlying `serde_json::Error`).
        message: String,
    },
    /// JSON parsed but referential integrity checks failed.
    Integrity(IntegrityError),
}

/// Identifies which JSON input a [`RulesetError::Parse`] came from. These are
/// stable machine identifiers, not user-facing prose, and appear verbatim in
/// the serialized `source` field.
pub(crate) mod parse_source {
    pub const POINT_ITEMS: &str = "point items";
    pub const TYPE_PROFILES: &str = "type profiles";
    pub const ABILITIES: &str = "abilities";
    pub const CHARACTERISTICS: &str = "characteristics";
    pub const I18N: &str = "i18n";
    /// Fallback used by the blanket `From<serde_json::Error>` conversion, where
    /// the failing input is not named at the call site.
    pub const UNKNOWN: &str = "unknown";
}

impl RulesetError {
    /// A stable machine-readable discriminant for this error.
    pub fn kind(&self) -> &'static str {
        match self {
            RulesetError::Parse { .. } => "parse",
            RulesetError::Integrity(_) => "integrity",
        }
    }

    /// Builds a parse error naming which input (`source`) failed.
    pub(crate) fn parse(source: &str, e: serde_json::Error) -> Self {
        RulesetError::Parse {
            source: source.to_string(),
            message: e.to_string(),
        }
    }
}

impl Serialize for RulesetError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            RulesetError::Parse { source, message } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("kind", self.kind())?;
                map.serialize_entry("source", source)?;
                map.serialize_entry("message", message)?;
                map.end()
            }
            RulesetError::Integrity(e) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("kind", self.kind())?;
                map.serialize_entry("errors", e.errors())?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RulesetError {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// Mirrors the custom `Serialize` map shape: `{kind, source, message}`
        /// for Parse, `{kind, errors}` for Integrity.
        #[derive(Deserialize)]
        struct Raw {
            kind: String,
            #[serde(default)]
            source: Option<String>,
            #[serde(default)]
            message: Option<String>,
            #[serde(default)]
            errors: Option<Vec<String>>,
        }

        let raw = Raw::deserialize(deserializer)?;
        match raw.kind.as_str() {
            // Each discriminant requires its matching payload key: a `parse`
            // without `source`/`message` or an `integrity` without `errors` is
            // malformed and must fail rather than default to an empty payload.
            "parse" => {
                let source = raw
                    .source
                    .ok_or_else(|| serde::de::Error::missing_field("source"))?;
                let message = raw
                    .message
                    .ok_or_else(|| serde::de::Error::missing_field("message"))?;
                Ok(RulesetError::Parse { source, message })
            }
            "integrity" => {
                let errors = raw
                    .errors
                    .ok_or_else(|| serde::de::Error::missing_field("errors"))?;
                Ok(RulesetError::Integrity(IntegrityError::new(errors)))
            }
            other => Err(serde::de::Error::custom(format!(
                "unknown RulesetError kind '{other}'"
            ))),
        }
    }
}

impl std::fmt::Display for RulesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RulesetError::Parse { source, message } => {
                write!(f, "parse error in {source}: {message}")
            }
            RulesetError::Integrity(e) => write!(f, "integrity error: {e}"),
        }
    }
}

impl std::error::Error for RulesetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RulesetError::Parse { .. } => None,
            RulesetError::Integrity(e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for RulesetError {
    /// Blanket conversion used where the failing input is not named; prefer
    /// [`RulesetError::parse`] at call sites that know which input failed.
    fn from(e: serde_json::Error) -> Self {
        RulesetError::parse(parse_source::UNKNOWN, e)
    }
}

impl From<IntegrityError> for RulesetError {
    fn from(e: IntegrityError) -> Self {
        RulesetError::Integrity(e)
    }
}

/// On-disk shape of `rules/core/abilities.json`: the advancement table plus the
/// ability catalogue. Both default to empty so `"{}"` is a valid empty file.
#[derive(Deserialize)]
struct AbilitiesFile {
    #[serde(default)]
    advancement: AdvancementTable,
    #[serde(default)]
    abilities: Vec<Ability>,
}

/// Pushes a `"duplicate <label> ID: '<id>'"` error for each id seen more than
/// once.
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

impl Ruleset {
    /// Parses point items and type profiles from JSON, validates referential
    /// integrity, and returns a Ruleset with no abilities.
    ///
    /// Convenience wrapper over [`Ruleset::from_json_with_abilities`] for callers
    /// (and the many tests) that do not exercise the ability registry.
    pub fn from_json(
        id: &str,
        version: &str,
        point_items_json: &str,
        type_profiles_json: &str,
    ) -> Result<Self, RulesetError> {
        Self::from_sources(RulesetSources {
            id,
            version,
            point_items: point_items_json,
            type_profiles: type_profiles_json,
            abilities: None,
            characteristics: None,
        })
    }

    /// Like [`Ruleset::from_json`] but also loads the abilities file (catalogue +
    /// advancement table). No characteristic rules.
    ///
    /// `abilities_json` is an object `{ "advancement": [...], "abilities": [...] }`;
    /// both keys default to empty, so `"{}"` is a valid empty file.
    pub fn from_json_with_abilities(
        id: &str,
        version: &str,
        point_items_json: &str,
        type_profiles_json: &str,
        abilities_json: &str,
    ) -> Result<Self, RulesetError> {
        Self::from_sources(RulesetSources {
            id,
            version,
            point_items: point_items_json,
            type_profiles: type_profiles_json,
            abilities: Some(abilities_json),
            characteristics: None,
        })
    }

    /// Parses every language-neutral core source — point items, type profiles,
    /// abilities (catalogue + advancement), and the Characteristic point-buy rules
    /// — validates referential integrity, and returns a Ruleset.
    ///
    /// `characteristics_json` is an object `{ "start_points", "costs" }`; an empty
    /// string means the ruleset ships no characteristic rules.
    pub fn from_core_json(
        id: &str,
        version: &str,
        point_items_json: &str,
        type_profiles_json: &str,
        abilities_json: &str,
        characteristics_json: &str,
    ) -> Result<Self, RulesetError> {
        Self::from_sources(RulesetSources {
            id,
            version,
            point_items: point_items_json,
            type_profiles: type_profiles_json,
            abilities: Some(abilities_json),
            // Preserve the existing sentinel: an empty string means "no
            // characteristic rules" for this back-compat wrapper.
            characteristics: (!characteristics_json.is_empty()).then_some(characteristics_json),
        })
    }

    /// Parses every language-neutral core source named in `sources`, validates
    /// referential integrity, and returns a Ruleset. This is the canonical
    /// loader; [`Ruleset::from_json`], [`Ruleset::from_json_with_abilities`], and
    /// [`Ruleset::from_core_json`] are thin back-compat wrappers over it.
    pub fn from_sources(sources: RulesetSources) -> Result<Self, RulesetError> {
        let RulesetSources {
            id,
            version,
            point_items: point_items_json,
            type_profiles: type_profiles_json,
            abilities,
            characteristics,
        } = sources;

        let items: Vec<PointItem> = serde_json::from_str(point_items_json)
            .map_err(|e| RulesetError::parse(parse_source::POINT_ITEMS, e))?;
        let types: Vec<EntityTypeProfile> = serde_json::from_str(type_profiles_json)
            .map_err(|e| RulesetError::parse(parse_source::TYPE_PROFILES, e))?;
        // An absent abilities file is equivalent to an empty `"{}"`.
        let abilities_file: AbilitiesFile = serde_json::from_str(abilities.unwrap_or("{}"))
            .map_err(|e| RulesetError::parse(parse_source::ABILITIES, e))?;
        let characteristic_rules: Option<CharacteristicRules> = match characteristics {
            None => None,
            Some(json) => Some(
                serde_json::from_str(json)
                    .map_err(|e| RulesetError::parse(parse_source::CHARACTERISTICS, e))?,
            ),
        };

        // Detect duplicate IDs across each registry.
        let mut errors = Vec::new();
        collect_duplicates(items.iter().map(|i| &i.id), "point item", &mut errors);
        collect_duplicates(types.iter().map(|t| &t.id), "type profile", &mut errors);
        collect_duplicates(
            abilities_file.abilities.iter().map(|a| &a.id),
            "ability",
            &mut errors,
        );
        if !errors.is_empty() {
            return Err(IntegrityError::new(errors).into());
        }

        let point_items: BTreeMap<Id, PointItem> = items
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect();
        let type_profiles: BTreeMap<Id, EntityTypeProfile> =
            types.into_iter().map(|t| (t.id.clone(), t)).collect();
        let abilities: BTreeMap<Id, Ability> = abilities_file
            .abilities
            .into_iter()
            .map(|a| (a.id.clone(), a))
            .collect();

        let ruleset = Self {
            id: Id::new(id),
            version: version.to_string(),
            point_items,
            type_profiles,
            abilities,
            advancement: abilities_file.advancement,
            characteristic_rules,
        };

        ruleset.validate_integrity()?;
        Ok(ruleset)
    }

    /// Deserializes a previously-serialized [`Ruleset`] and RE-RUNS referential
    /// integrity validation. Use this for trusted/cached data; untrusted JSON must
    /// still go through [`Ruleset::from_json`]. Deriving `Deserialize` alone does
    /// NOT validate integrity — always reconstruct via this method.
    pub fn from_serialized(json: &str) -> Result<Self, RulesetError> {
        let ruleset: Ruleset = serde_json::from_str(json)?;
        ruleset.validate_integrity()?;
        Ok(ruleset)
    }

    /// Iterates all point items in id order.
    pub fn items(&self) -> impl Iterator<Item = &PointItem> {
        self.point_items.values()
    }

    /// Iterates all type profiles in id order.
    pub fn profiles(&self) -> impl Iterator<Item = &EntityTypeProfile> {
        self.type_profiles.values()
    }

    /// Number of point items.
    pub fn item_count(&self) -> usize {
        self.point_items.len()
    }

    /// Number of type profiles.
    pub fn profile_count(&self) -> usize {
        self.type_profiles.len()
    }

    /// Sorts each point item's parameters for canonical serialization. The
    /// ruleset's own maps are already id-ordered (`BTreeMap`); this is the single
    /// runtime entry point that normalizes the nested item data
    /// (see [`crate::types::PointItem::normalize`]).
    pub fn normalize(&mut self) {
        for item in self.point_items.values_mut() {
            item.normalize();
        }
    }

    /// Returns a [`RulesetRef`] identifying this ruleset (id + version).
    pub fn reference(&self) -> RulesetRef {
        RulesetRef::new(self.id.clone(), self.version.clone())
    }

    /// Returns `true` if `reference` names this ruleset by id and version.
    pub fn matches(&self, reference: &RulesetRef) -> bool {
        self.id == reference.id && self.version == reference.version
    }

    /// Looks up a point item by id.
    pub fn item(&self, id: &Id) -> Option<&PointItem> {
        self.point_items.get(id)
    }

    /// Looks up an entity type profile by id.
    pub fn profile(&self, id: &Id) -> Option<&EntityTypeProfile> {
        self.type_profiles.get(id)
    }

    /// Looks up an ability by id.
    pub fn ability(&self, id: &Id) -> Option<&Ability> {
        self.abilities.get(id)
    }

    /// Iterates all abilities in id order.
    pub fn abilities(&self) -> impl Iterator<Item = &Ability> {
        self.abilities.values()
    }

    /// Number of abilities in the catalogue.
    pub fn ability_count(&self) -> usize {
        self.abilities.len()
    }

    /// The Ability XP advancement table.
    pub fn advancement(&self) -> &AdvancementTable {
        &self.advancement
    }

    /// The Characteristic point-buy rules, if the ruleset ships them.
    pub fn characteristic_rules(&self) -> Option<&CharacteristicRules> {
        self.characteristic_rules.as_ref()
    }

    /// Iterates over point items of the given [`ItemKind`].
    pub fn items_by_kind(&self, kind: ItemKind) -> impl Iterator<Item = &PointItem> {
        self.point_items.values().filter(move |i| i.kind == kind)
    }

    /// Iterates over point items in the given category.
    pub fn items_by_category<'a>(
        &'a self,
        category: &'a str,
    ) -> impl Iterator<Item = &'a PointItem> {
        self.point_items
            .values()
            .filter(move |i| i.category == category)
    }

    /// Checks referential integrity: prerequisite refs, incompatibility symmetry,
    /// type profile trait refs, parameter domain refs, and source line ranges.
    pub fn validate_integrity(&self) -> Result<(), IntegrityError> {
        let mut errors = Vec::new();

        for (id, item) in &self.point_items {
            if let Some(ref prereq) = item.prerequisites {
                self.validate_prereq_refs(prereq, id, &mut errors);
            }

            for incompat_id in &item.incompatible_with {
                if !self.point_items.contains_key(incompat_id) {
                    errors.push(format!(
                        "{id}: incompatible_with references unknown ID '{incompat_id}'"
                    ));
                }
            }

            // Parameter domains are validated at parse time by the
            // ParameterDomain enum; concrete param VALUES are resolved per
            // selection in validation::validate_parameters.
            if let Some(ref source) = item.source
                && !source.lines.is_valid()
            {
                errors.push(format!(
                    "{id}: source line range start ({}) exceeds end ({})",
                    source.lines.start, source.lines.end
                ));
            }
        }

        self.validate_incompatibility_symmetry(&mut errors);

        for (type_id, profile) in &self.type_profiles {
            for trait_id in &profile.required_traits {
                if !self.point_items.contains_key(trait_id) {
                    errors.push(format!(
                        "type profile '{type_id}': required_trait references unknown ID '{trait_id}'"
                    ));
                }
            }
            for trait_id in &profile.forbidden_traits {
                if !self.point_items.contains_key(trait_id) {
                    errors.push(format!(
                        "type profile '{type_id}': forbidden_trait references unknown ID '{trait_id}'"
                    ));
                }
            }
            if let Some(ref gift_id) = profile.gift_id
                && !self.point_items.contains_key(gift_id)
            {
                errors.push(format!(
                    "type profile '{type_id}': gift_id references unknown ID '{gift_id}'"
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(IntegrityError::new(errors))
        }
    }

    /// Recursively validates that prerequisite refs resolve to known registries:
    /// [`Prereq::Has`] against point items and [`Prereq::AbilityMin`] against the
    /// ability catalogue.
    ///
    /// `House` and `ArtMin` carry refs into house/art registries that do not exist
    /// yet; those refs are INTENTIONALLY left unchecked (a deferred check),
    /// narrowing the integrity contract explicitly so the gap is tracked rather
    /// than silent — it must be revisited when those registries are added (M5).
    /// `IsMagus` carries no reference at all, so there is nothing to check for it.
    fn validate_prereq_refs(&self, prereq: &Prereq, context_id: &Id, errors: &mut Vec<String>) {
        match prereq {
            Prereq::All(children) | Prereq::Any(children) | Prereq::Nor(children) => {
                for child in children {
                    self.validate_prereq_refs(child, context_id, errors);
                }
            }
            Prereq::Has(ref_id) => {
                if !self.point_items.contains_key(ref_id) {
                    errors.push(format!(
                        "{context_id}: prerequisite references unknown ID '{ref_id}'"
                    ));
                }
            }
            Prereq::AbilityMin { ability, .. } => {
                if !self.abilities.contains_key(ability) {
                    errors.push(format!(
                        "{context_id}: prerequisite references unknown ability '{ability}'"
                    ));
                }
            }
            // Intentionally unchecked: no house/art registry exists yet (M5).
            Prereq::House(_) | Prereq::ArtMin { .. } | Prereq::IsMagus => {}
        }
    }

    fn validate_incompatibility_symmetry(&self, errors: &mut Vec<String>) {
        for (id, item) in &self.point_items {
            for incompat_id in &item.incompatible_with {
                if let Some(other) = self.point_items.get(incompat_id)
                    && !other.incompatible_with.contains(id)
                {
                    errors.push(format!(
                        "asymmetric incompatibility: '{id}' lists '{incompat_id}' but not vice versa"
                    ));
                }
            }
        }
    }
}

impl LocalizedRuleset {
    /// Pairs a ruleset with localized text parsed from a JSON id-to-entry map.
    pub fn new(ruleset: Ruleset, i18n_json: &str) -> Result<Self, RulesetError> {
        Self::from_merged(ruleset, &[i18n_json])
    }

    /// Pairs a ruleset with localized text merged from several JSON id-to-entry
    /// maps (e.g. one file per rules domain: virtues/flaws, abilities, …).
    ///
    /// A given id may appear in only one source; a collision across files is an
    /// integrity error, since the id namespaces (`virtue.*`, `ability.*`, …) are
    /// meant to be disjoint.
    pub fn from_merged(ruleset: Ruleset, i18n_sources: &[&str]) -> Result<Self, RulesetError> {
        let mut i18n: BTreeMap<Id, I18nEntry> = BTreeMap::new();
        let mut collisions = Vec::new();
        for source in i18n_sources {
            let entries: BTreeMap<String, I18nEntry> = serde_json::from_str(source)
                .map_err(|e| RulesetError::parse(parse_source::I18N, e))?;
            for (key, value) in entries {
                let id = Id::new(key);
                if i18n.insert(id.clone(), value).is_some() {
                    collisions.push(format!("duplicate i18n entry for '{id}'"));
                }
            }
        }
        if !collisions.is_empty() {
            return Err(IntegrityError::new(collisions).into());
        }
        Ok(Self { ruleset, i18n })
    }

    /// Returns the full localized [`I18nEntry`] for the given id, if present.
    /// Use this when the caller needs more than the display name (name, summary,
    /// and description together); the field-specific helpers
    /// ([`LocalizedRuleset::display_name`], [`LocalizedRuleset::summary`],
    /// [`LocalizedRuleset::description`]) are thin wrappers over it.
    pub fn entry(&self, id: &Id) -> Option<&I18nEntry> {
        self.i18n.get(id)
    }

    /// Returns the localized display name for the given id, if present.
    pub fn display_name(&self, id: &Id) -> Option<&str> {
        self.i18n.get(id).map(|e| e.name.as_str())
    }

    /// Returns the localized short summary for the given id, if both the entry
    /// and its (optional) summary are present.
    pub fn summary(&self, id: &Id) -> Option<&str> {
        self.i18n.get(id).and_then(|e| e.summary.as_deref())
    }

    /// Returns the localized full description for the given id, if both the entry
    /// and its (optional) description are present.
    pub fn description(&self, id: &Id) -> Option<&str> {
        self.i18n.get(id).and_then(|e| e.description.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ParameterDomain;
    use pretty_assertions::assert_eq;

    const VALID_ITEMS: &str = r#"[
      {
        "id": "virtue.the_gift",
        "kind": "virtue",
        "magnitude": "free",
        "category": "special",
        "entity_kinds": ["character"]
      },
      {
        "id": "virtue.hermetic_magus",
        "kind": "virtue",
        "magnitude": "free",
        "category": "social_status",
        "entity_kinds": ["character"],
        "prerequisites": { "kind": "has", "value": "virtue.the_gift" }
      },
      {
        "id": "virtue.gentle_gift",
        "kind": "virtue",
        "magnitude": "major",
        "category": "hermetic",
        "entity_kinds": ["character"],
        "prerequisites": { "kind": "has", "value": "virtue.hermetic_magus" },
        "incompatible_with": ["flaw.blatant_gift"]
      },
      {
        "id": "flaw.blatant_gift",
        "kind": "flaw",
        "magnitude": "major",
        "category": "hermetic",
        "entity_kinds": ["character"],
        "prerequisites": { "kind": "has", "value": "virtue.the_gift" },
        "incompatible_with": ["virtue.gentle_gift"]
      },
      {
        "id": "virtue.puissant_ability",
        "kind": "virtue",
        "magnitude": "minor",
        "category": "general",
        "entity_kinds": ["character"],
        "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }]
      }
    ]"#;

    const VALID_TYPES: &str = r#"[
      {
        "id": "companion",
        "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "social_status", "supernatural"],
        "forbidden_categories": ["hermetic"],
        "required_traits": [],
        "forbidden_traits": [],
        "gift_policy": "forbidden",
        "creation_phases": ["concept", "virtues_flaws", "abilities"]
      }
    ]"#;

    const VALID_ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 },
        { "score": 2, "total_xp": 15 }
      ],
      "abilities": [
        { "id": "ability.awareness", "category": "general" },
        { "id": "ability.magic_theory", "category": "arcane" }
      ]
    }"#;

    #[test]
    fn load_valid_ruleset() {
        let rs = Ruleset::from_json("arm5-core", "2024.1", VALID_ITEMS, VALID_TYPES).unwrap();
        assert_eq!(rs.item_count(), 5);
        assert_eq!(rs.profile_count(), 1);
        assert_eq!(rs.ability_count(), 0);
        assert_eq!(rs.id, Id::new("arm5-core"));
    }

    #[test]
    fn from_sources_named_struct_form() {
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: Some(VALID_ABILITIES),
            characteristics: None,
        })
        .unwrap();
        assert_eq!(rs.item_count(), 5);
        assert_eq!(rs.ability_count(), 2);
        assert_eq!(rs.id, Id::new("arm5-core"));

        // Omitting abilities yields an empty registry, matching from_json.
        let no_abilities = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            characteristics: None,
        })
        .unwrap();
        assert_eq!(no_abilities.ability_count(), 0);
    }

    #[test]
    fn from_json_defaults_to_no_abilities() {
        let rs = Ruleset::from_json("t", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        assert_eq!(rs.ability_count(), 0);
        assert_eq!(rs.advancement().rows().len(), 0);
    }

    #[test]
    fn load_ability_registry_and_advancement() {
        let rs = Ruleset::from_json_with_abilities(
            "arm5-core",
            "2024.1",
            VALID_ITEMS,
            VALID_TYPES,
            VALID_ABILITIES,
        )
        .unwrap();
        assert_eq!(rs.ability_count(), 2);
        assert!(rs.ability(&Id::new("ability.awareness")).is_some());
        assert!(rs.ability(&Id::new("ability.missing")).is_none());
        assert_eq!(rs.advancement().xp_for_score(2), Some(15));
    }

    #[test]
    fn duplicate_ability_id_is_rejected() {
        let dup = r#"{ "abilities": [
          { "id": "ability.awareness", "category": "general" },
          { "id": "ability.awareness", "category": "general" }
        ] }"#;
        let err =
            Ruleset::from_json_with_abilities("t", "1", VALID_ITEMS, VALID_TYPES, dup).unwrap_err();
        match err {
            RulesetError::Integrity(e) => {
                assert!(
                    e.errors()
                        .iter()
                        .any(|m| m.contains("duplicate ability ID"))
                );
            }
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn ability_min_prereq_must_resolve() {
        let items = r#"[{
          "id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general",
          "entity_kinds": ["character"],
          "prerequisites": { "kind": "ability_min", "value": { "ability": "ability.unknown", "score": 2 } }
        }]"#;
        let err =
            Ruleset::from_json_with_abilities("t", "1", items, "[]", VALID_ABILITIES).unwrap_err();
        match err {
            RulesetError::Integrity(e) => {
                assert!(e.errors().iter().any(|m| m.contains("unknown ability")));
            }
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn ability_min_prereq_resolves_against_registry() {
        let items = r#"[{
          "id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general",
          "entity_kinds": ["character"],
          "prerequisites": { "kind": "ability_min", "value": { "ability": "ability.awareness", "score": 2 } }
        }]"#;
        let rs = Ruleset::from_json_with_abilities("t", "1", items, "[]", VALID_ABILITIES);
        assert!(rs.is_ok());
    }

    #[test]
    fn reference_and_matches() {
        let rs = Ruleset::from_json("arm5-core", "2024.1", VALID_ITEMS, VALID_TYPES).unwrap();
        let reference = rs.reference();
        assert_eq!(reference.id, Id::new("arm5-core"));
        assert_eq!(reference.version, "2024.1");
        assert!(rs.matches(&reference));
        assert!(!rs.matches(&RulesetRef::new(Id::new("arm5-core"), "9.9")));
    }

    #[test]
    fn lookup_helpers() {
        let rs = Ruleset::from_json("arm5-core", "2024.1", VALID_ITEMS, VALID_TYPES).unwrap();
        assert!(rs.item(&Id::new("virtue.the_gift")).is_some());
        assert!(rs.item(&Id::new("virtue.missing")).is_none());
        assert!(rs.profile(&Id::new("companion")).is_some());
        assert!(rs.profile(&Id::new("nope")).is_none());

        let virtues = rs.items_by_kind(ItemKind::Virtue).count();
        assert_eq!(virtues, 4);
        let flaws = rs.items_by_kind(ItemKind::Flaw).count();
        assert_eq!(flaws, 1);

        let hermetic = rs.items_by_category("hermetic").count();
        assert_eq!(hermetic, 2);
    }

    #[test]
    fn missing_prereq_ref() {
        let items = r#"[{
          "id": "virtue.gentle_gift",
          "kind": "virtue",
          "magnitude": "major",
          "category": "hermetic",
          "entity_kinds": ["character"],
          "prerequisites": { "kind": "has", "value": "virtue.nonexistent" }
        }]"#;

        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("virtue.nonexistent"),
            "error should name the missing ID: {msg}"
        );
    }

    #[test]
    fn asymmetric_incompatibility() {
        let items = r#"[
          {
            "id": "virtue.a",
            "kind": "virtue",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "incompatible_with": ["flaw.b"]
          },
          {
            "id": "flaw.b",
            "kind": "flaw",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "incompatible_with": []
          }
        ]"#;

        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("asymmetric"),
            "error should mention asymmetry: {msg}"
        );
    }

    #[test]
    fn unknown_incompatible_ref() {
        let items = r#"[{
          "id": "virtue.a",
          "kind": "virtue",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "incompatible_with": ["flaw.nonexistent"]
        }]"#;

        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("flaw.nonexistent"),
            "error should name the missing ID: {msg}"
        );
    }

    #[test]
    fn invalid_source_line_range() {
        let items = r#"[{
          "id": "virtue.a",
          "kind": "virtue",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "source": { "file": "f.md", "lines": [50, 10] }
        }]"#;

        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("line range"),
            "should flag inverted line range: {msg}"
        );
    }

    #[test]
    fn localized_ruleset_display_name() {
        let rs = Ruleset::from_json("arm5-core", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        let i18n = r#"{
          "virtue.gentle_gift": { "name": "Gentle Gift", "summary": "Your Gift is not disturbing." },
          "virtue.puissant_ability": { "name": "Puissant (Ability)" }
        }"#;

        let loc = LocalizedRuleset::new(rs, i18n).unwrap();
        assert_eq!(
            loc.display_name(&Id::new("virtue.gentle_gift")),
            Some("Gentle Gift")
        );
        assert_eq!(loc.display_name(&Id::new("virtue.nonexistent")), None);
    }

    #[test]
    fn localized_ruleset_entry_summary_description() {
        let rs = Ruleset::from_json("arm5-core", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        let i18n = r#"{
          "virtue.gentle_gift": {
            "name": "Gentle Gift",
            "summary": "Your Gift is not disturbing.",
            "description": "People do not react with mistrust to you."
          },
          "virtue.puissant_ability": { "name": "Puissant (Ability)" }
        }"#;
        let loc = LocalizedRuleset::new(rs, i18n).unwrap();

        // Full entry lookup.
        let entry = loc.entry(&Id::new("virtue.gentle_gift")).unwrap();
        assert_eq!(entry.name, "Gentle Gift");
        assert_eq!(
            entry.summary.as_deref(),
            Some("Your Gift is not disturbing.")
        );
        assert!(loc.entry(&Id::new("virtue.nonexistent")).is_none());

        // Field-specific helpers.
        assert_eq!(
            loc.summary(&Id::new("virtue.gentle_gift")),
            Some("Your Gift is not disturbing.")
        );
        assert_eq!(
            loc.description(&Id::new("virtue.gentle_gift")),
            Some("People do not react with mistrust to you.")
        );
        // An entry that exists but omits summary/description returns None.
        assert_eq!(loc.summary(&Id::new("virtue.puissant_ability")), None);
        assert_eq!(loc.description(&Id::new("virtue.puissant_ability")), None);
        // A missing id returns None for all helpers.
        assert_eq!(loc.summary(&Id::new("virtue.nonexistent")), None);
    }

    #[test]
    fn character_type_unknown_required_trait() {
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "required_traits": ["virtue.nonexistent"],
          "forbidden_traits": [],
          "gift_policy": "forbidden",
          "creation_phases": []
        }]"#;

        let err = Ruleset::from_json("test", "1", "[]", types).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("virtue.nonexistent"));
    }

    #[test]
    fn character_type_unknown_gift_id() {
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "required_traits": [],
          "forbidden_traits": [],
          "gift_policy": "required",
          "gift_id": "virtue.nonexistent_gift",
          "creation_phases": []
        }]"#;

        let err = Ruleset::from_json("test", "1", "[]", types).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("virtue.nonexistent_gift"),
            "should flag unknown gift_id ref: {msg}"
        );
    }

    #[test]
    fn forbidden_traits_unknown_ref() {
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "required_traits": [],
          "forbidden_traits": ["virtue.nonexistent"],
          "creation_phases": []
        }]"#;

        let err = Ruleset::from_json("test", "1", "[]", types).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("virtue.nonexistent"),
            "should flag unknown forbidden_trait ref: {msg}"
        );
    }

    #[test]
    fn prereq_refs_recursive() {
        let items = r#"[{
          "id": "virtue.a",
          "kind": "virtue",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "prerequisites": {"kind": "all", "value": [{"kind": "has", "value": "virtue.nonexistent"}]}
        }]"#;

        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("virtue.nonexistent"),
            "should find nested unknown prereq ref: {msg}"
        );
    }

    #[test]
    fn from_json_invalid_json() {
        let result = Ruleset::from_json("test", "1", "NOT VALID JSON", "[]");
        assert!(result.is_err(), "malformed JSON should produce an error");
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "parse");
        assert!(
            err.to_string().contains("parse error"),
            "should be a parse error: {err}"
        );
    }

    #[test]
    fn from_merged_rejects_duplicate_id_across_sources() {
        // The id namespaces are meant to be disjoint, so the same id defined in
        // two i18n sources is an integrity error naming the offending id.
        let rs = Ruleset::from_json("arm5-core", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        let first = r#"{ "virtue.a": { "name": "First A" } }"#;
        let second = r#"{ "virtue.a": { "name": "Second A" } }"#;

        let err = LocalizedRuleset::from_merged(rs, &[first, second]).unwrap_err();
        match err {
            RulesetError::Integrity(e) => {
                assert!(
                    e.errors().iter().any(|m| m.contains("virtue.a")),
                    "collision error should name the offending id: {:?}",
                    e.errors()
                );
            }
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn localized_ruleset_invalid_json() {
        let rs = Ruleset::from_json("arm5-core", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        let result = LocalizedRuleset::new(rs, "NOT VALID JSON");
        assert!(
            result.is_err(),
            "malformed i18n JSON should produce an error"
        );
    }

    #[test]
    fn duplicate_point_item_ids() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "magnitude": "major", "category": "general", "entity_kinds": ["character"]}
        ]"#;

        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("duplicate"),
            "should flag duplicate IDs: {msg}"
        );
    }

    #[test]
    fn duplicate_type_profile_ids() {
        let types = r#"[
          {"id": "companion", "budget": {"virtue_points": 10, "flaw_points": 10}, "creation_phases": []},
          {"id": "companion", "budget": {"virtue_points": 5, "flaw_points": 5}, "creation_phases": []}
        ]"#;

        let err = Ruleset::from_json("test", "1", "[]", types).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("duplicate"),
            "should flag duplicate type profile IDs: {msg}"
        );
    }

    #[test]
    fn ruleset_error_display_and_kind() {
        let parse_err = Ruleset::from_json("test", "1", "INVALID", "[]").unwrap_err();
        assert_eq!(parse_err.kind(), "parse");
        assert!(format!("{parse_err}").contains("parse error"));

        let integrity_err = Ruleset::from_json(
            "test",
            "1",
            r#"[{"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": [], "prerequisites": {"kind": "has", "value": "virtue.missing"}}]"#,
            "[]",
        )
        .unwrap_err();
        assert_eq!(integrity_err.kind(), "integrity");
        assert!(format!("{integrity_err}").contains("integrity error"));
    }

    #[test]
    fn ruleset_error_source() {
        use std::error::Error;

        // Parse errors no longer leak the underlying serde_json::Error.
        let parse_err = Ruleset::from_json("test", "1", "INVALID", "[]").unwrap_err();
        assert!(parse_err.source().is_none());

        let integrity_err = Ruleset::from_json(
            "test",
            "1",
            r#"[{"id":"virtue.a","kind":"virtue","magnitude":"minor","category":"general","entity_kinds":[],"prerequisites":{"kind": "has", "value":"virtue.missing"}}]"#,
            "[]",
        )
        .unwrap_err();
        assert!(integrity_err.source().is_some());
    }

    #[test]
    fn integrity_error_exposes_individual_messages() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.x"}},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.y"}}
        ]"#;
        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        match err {
            RulesetError::Integrity(e) => {
                assert_eq!(e.errors().len(), 2, "two distinct integrity errors");
            }
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn ruleset_error_serialize() {
        let parse_err = Ruleset::from_json("test", "1", "INVALID", "[]").unwrap_err();
        let json = serde_json::to_string(&parse_err).unwrap();
        assert!(json.contains("parse"), "serialized: {json}");

        let integrity_err = Ruleset::from_json(
            "test",
            "1",
            r#"[{"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": [], "prerequisites": {"kind": "has", "value": "virtue.missing"}}]"#,
            "[]",
        )
        .unwrap_err();
        let json = serde_json::to_string(&integrity_err).unwrap();
        assert!(json.contains("integrity"), "serialized: {json}");
        assert!(json.contains("errors"), "serialized errors list: {json}");
    }

    #[test]
    fn domain_resolution_classification() {
        assert!(ParameterDomain::Item.resolves_against_items());
        assert!(!ParameterDomain::Ability.resolves_against_items());
        assert!(!ParameterDomain::Art.resolves_against_items());
    }

    #[test]
    fn serialized_field_names_are_the_stable_contract() {
        // The frontend binds directly to these top-level field names; an
        // accidental rename must fail here rather than silently break TS.
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: Some(VALID_ABILITIES),
            characteristics: None,
        })
        .unwrap();

        let value: serde_json::Value = serde_json::to_value(&rs).unwrap();
        let obj = value.as_object().expect("Ruleset serializes as an object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "abilities",
                "advancement",
                "characteristic_rules",
                "id",
                "point_items",
                "type_profiles",
                "version",
            ],
            "Ruleset top-level field names are a stable public contract"
        );
        // The data maps are objects; advancement is a bare array.
        assert!(obj["point_items"].is_object());
        assert!(obj["type_profiles"].is_object());
        assert!(obj["abilities"].is_object());
        assert!(obj["advancement"].is_array());

        // And the whole thing round-trips back through the validating loader.
        let restored = Ruleset::from_serialized(&serde_json::to_string(&rs).unwrap()).unwrap();
        assert_eq!(rs, restored);
    }

    #[test]
    fn from_serialized_roundtrips_valid_ruleset() {
        let rs = Ruleset::from_json("arm5-core", "2024.1", VALID_ITEMS, VALID_TYPES).unwrap();
        let json = serde_json::to_string(&rs).unwrap();
        let restored = Ruleset::from_serialized(&json).unwrap();
        assert_eq!(rs, restored);
    }

    #[test]
    fn from_serialized_rejects_dangling_prereq_ref() {
        // A serialized ruleset whose JSON carries a dangling Has(...) prereq must
        // be rejected: from_serialized re-runs referential integrity.
        let json = r#"{
          "id": "arm5-core",
          "version": "2024.1",
          "point_items": {
            "virtue.a": {
              "id": "virtue.a",
              "kind": "virtue",
              "magnitude": "minor",
              "category": "general",
              "entity_kinds": ["character"],
              "prerequisites": { "kind": "has", "value": "virtue.missing" }
            }
          },
          "type_profiles": {}
        }"#;
        let err = Ruleset::from_serialized(json).unwrap_err();
        assert_eq!(err.kind(), "integrity");
        assert!(
            err.to_string().contains("virtue.missing"),
            "should name the dangling ref: {err}"
        );
    }

    #[test]
    fn from_serialized_reports_unknown_source_on_malformed_json() {
        // Syntactically invalid JSON cannot name a specific input, so the blanket
        // `From<serde_json::Error>` conversion maps it to the UNKNOWN source.
        let err = Ruleset::from_serialized("NOT VALID JSON").unwrap_err();
        assert_eq!(err.kind(), "parse");
        match err {
            RulesetError::Parse { source, .. } => {
                assert_eq!(source, parse_source::UNKNOWN);
            }
            other => panic!("expected a parse error, got {other:?}"),
        }
    }

    #[test]
    fn ruleset_error_serde_roundtrip() {
        let parse_err = Ruleset::from_json("test", "1", "INVALID", "[]").unwrap_err();
        let json = serde_json::to_string(&parse_err).unwrap();
        let restored: RulesetError = serde_json::from_str(&json).unwrap();
        assert_eq!(parse_err, restored);

        let integrity_err = Ruleset::from_json(
            "test",
            "1",
            r#"[{"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": [], "prerequisites": {"kind": "has", "value": "virtue.missing"}}]"#,
            "[]",
        )
        .unwrap_err();
        let json = serde_json::to_string(&integrity_err).unwrap();
        let restored: RulesetError = serde_json::from_str(&json).unwrap();
        assert_eq!(integrity_err, restored);

        // An unknown discriminant is a deserialization error, not a silent default.
        let err = serde_json::from_str::<RulesetError>(r#"{"kind":"bogus"}"#).unwrap_err();
        assert!(
            err.to_string().contains("bogus"),
            "should name the unknown kind: {err}"
        );
    }

    #[test]
    fn ruleset_error_missing_payload_is_rejected() {
        // A discriminant without its matching payload must fail to deserialize,
        // not default to an empty Parse / empty IntegrityError.
        let no_source = serde_json::from_str::<RulesetError>(r#"{"kind":"parse"}"#).unwrap_err();
        assert!(
            no_source.to_string().contains("source"),
            "parse without source should be rejected: {no_source}"
        );

        let no_message =
            serde_json::from_str::<RulesetError>(r#"{"kind":"parse","source":"abilities"}"#)
                .unwrap_err();
        assert!(
            no_message.to_string().contains("message"),
            "parse without message should be rejected: {no_message}"
        );

        let integrity_err =
            serde_json::from_str::<RulesetError>(r#"{"kind":"integrity"}"#).unwrap_err();
        assert!(
            integrity_err.to_string().contains("errors"),
            "integrity without errors should be rejected: {integrity_err}"
        );
    }

    #[test]
    fn parse_error_names_the_failing_source() {
        // A bad type-profiles input must surface which input failed, so a UI can
        // point at the offending file rather than show one undifferentiated blob.
        let err = Ruleset::from_json("test", "1", "[]", "NOT VALID JSON").unwrap_err();
        match &err {
            RulesetError::Parse { source, .. } => {
                assert_eq!(source, "type profiles");
            }
            other => panic!("expected a parse error, got {other:?}"),
        }
        assert!(
            err.to_string().contains("type profiles"),
            "Display should name the source: {err}"
        );

        // A different input fails with a different source identifier.
        let abilities_err =
            Ruleset::from_json_with_abilities("test", "1", "[]", "[]", "NOT VALID JSON")
                .unwrap_err();
        assert!(
            matches!(abilities_err, RulesetError::Parse { ref source, .. } if source == "abilities"),
            "abilities parse failure should name 'abilities': {abilities_err}"
        );

        // The serialized form carries the source for the frontend.
        let json = serde_json::to_string(&err).unwrap();
        assert!(
            json.contains(r#""source":"type profiles""#),
            "serialized: {json}"
        );
    }

    #[test]
    fn accessors_iterate_in_id_order() {
        let rs = Ruleset::from_json("arm5-core", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        let item_ids: Vec<&str> = rs.items().map(|i| i.id.as_str()).collect();
        let mut sorted = item_ids.clone();
        sorted.sort_unstable();
        assert_eq!(item_ids, sorted, "items() iterates in id order");
        assert_eq!(rs.items().count(), rs.item_count());
        assert_eq!(rs.profiles().count(), rs.profile_count());
        assert_eq!(rs.profiles().next().unwrap().id, Id::new("companion"));
    }

    #[test]
    fn ruleset_normalize_sorts_item_parameters() {
        let items = r#"[{
          "id": "virtue.x",
          "kind": "virtue",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "parameters": [
            { "key": "second", "type": "ref", "domain": "art" },
            { "key": "first", "type": "ref", "domain": "ability" }
          ]
        }]"#;
        let mut rs = Ruleset::from_json("test", "1", items, "[]").unwrap();
        rs.normalize();
        let item = rs.item(&Id::new("virtue.x")).unwrap();
        let keys: Vec<&str> = item.parameters.iter().map(|p| p.key.as_str()).collect();
        assert_eq!(keys, vec!["first", "second"]);
    }
}
