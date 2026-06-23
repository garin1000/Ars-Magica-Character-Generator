use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::ability::{Ability, AdvancementTable};
use crate::types::{EntityTypeProfile, I18nEntry, Id, ItemKind, PointItem, Prereq, RulesetRef};

/// Top-level container for all loaded game mechanics.
///
/// Built only via [`Ruleset::from_json`] (which validates referential
/// integrity), never field-by-field by callers; `PartialEq` is provided for
/// tests and diffing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ruleset {
    /// Stable ruleset identifier (matches [`RulesetRef::id`]).
    pub id: Id,
    /// Ruleset version string.
    pub version: String,
    /// All point items keyed by their id.
    pub(crate) point_items: BTreeMap<Id, PointItem>,
    /// All entity type profiles keyed by their id.
    pub(crate) type_profiles: BTreeMap<Id, EntityTypeProfile>,
    /// All abilities keyed by their id. Defaulted so older serialized rulesets
    /// (pre-M3, no abilities) still deserialize via [`Ruleset::from_serialized`].
    #[serde(default)]
    pub(crate) abilities: BTreeMap<Id, Ability>,
    /// The Ability XP advancement table.
    #[serde(default)]
    pub(crate) advancement: AdvancementTable,
}

/// A [`Ruleset`] paired with localized display text for a single language.
///
/// Built only via [`LocalizedRuleset::new`]; `PartialEq` is provided for tests.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesetError {
    /// JSON failed to parse or deserialize. Holds the formatted parse message.
    Parse(String),
    /// JSON parsed but referential integrity checks failed.
    Integrity(IntegrityError),
}

impl RulesetError {
    /// A stable machine-readable discriminant for this error.
    pub fn kind(&self) -> &'static str {
        match self {
            RulesetError::Parse(_) => "parse",
            RulesetError::Integrity(_) => "integrity",
        }
    }
}

impl Serialize for RulesetError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("kind", self.kind())?;
        match self {
            RulesetError::Parse(message) => {
                map.serialize_entry("message", message)?;
            }
            RulesetError::Integrity(e) => {
                map.serialize_entry("errors", e.errors())?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for RulesetError {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// Mirrors the custom `Serialize` map shape: `{kind, message}` for Parse,
        /// `{kind, errors}` for Integrity.
        #[derive(Deserialize)]
        struct Raw {
            kind: String,
            #[serde(default)]
            message: Option<String>,
            #[serde(default)]
            errors: Option<Vec<String>>,
        }

        let raw = Raw::deserialize(deserializer)?;
        match raw.kind.as_str() {
            "parse" => Ok(RulesetError::Parse(raw.message.unwrap_or_default())),
            "integrity" => Ok(RulesetError::Integrity(IntegrityError::new(
                raw.errors.unwrap_or_default(),
            ))),
            other => Err(serde::de::Error::custom(format!(
                "unknown RulesetError kind '{other}'"
            ))),
        }
    }
}

impl std::fmt::Display for RulesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RulesetError::Parse(message) => write!(f, "parse error: {message}"),
            RulesetError::Integrity(e) => write!(f, "integrity error: {e}"),
        }
    }
}

impl std::error::Error for RulesetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RulesetError::Parse(_) => None,
            RulesetError::Integrity(e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for RulesetError {
    fn from(e: serde_json::Error) -> Self {
        RulesetError::Parse(e.to_string())
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
        Self::from_json_with_abilities(id, version, point_items_json, type_profiles_json, "{}")
    }

    /// Parses point items, type profiles, and the abilities file (catalogue +
    /// advancement table) from JSON, validates referential integrity, and returns
    /// a Ruleset.
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
        let items: Vec<PointItem> = serde_json::from_str(point_items_json)?;
        let types: Vec<EntityTypeProfile> = serde_json::from_str(type_profiles_json)?;
        let abilities_file: AbilitiesFile = serde_json::from_str(abilities_json)?;

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
            Prereq::All(children) | Prereq::Any(children) | Prereq::None(children) => {
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
        let entries: BTreeMap<String, I18nEntry> = serde_json::from_str(i18n_json)?;
        let i18n: BTreeMap<Id, I18nEntry> =
            entries.into_iter().map(|(k, v)| (Id::new(k), v)).collect();
        Ok(Self { ruleset, i18n })
    }

    /// Returns the localized display name for the given id, if present.
    pub fn display_name(&self, id: &Id) -> Option<&str> {
        self.i18n.get(id).map(|e| e.name.as_str())
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
