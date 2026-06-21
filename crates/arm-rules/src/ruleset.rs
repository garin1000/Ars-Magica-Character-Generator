use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::types::{EntityTypeProfile, I18nEntry, Id, PointItem};

/// Top-level container for all loaded game mechanics.
#[derive(Debug, Clone, Serialize)]
pub struct Ruleset {
    pub id: String,
    pub version: String,
    pub point_items: BTreeMap<Id, PointItem>,
    pub type_profiles: BTreeMap<Id, EntityTypeProfile>,
}

/// A [`Ruleset`] paired with localized display text for a single language.
#[derive(Debug, Clone, Serialize)]
pub struct LocalizedRuleset {
    pub ruleset: Ruleset,
    pub i18n: BTreeMap<Id, I18nEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IntegrityError {
    pub message: String,
}

impl std::fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for IntegrityError {}

#[derive(Debug)]
pub enum RulesetError {
    ParseError(serde_json::Error),
    Integrity(IntegrityError),
}

impl Serialize for RulesetError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        match self {
            RulesetError::ParseError(e) => {
                map.serialize_entry("kind", "parse_error")?;
                map.serialize_entry("message", &e.to_string())?;
            }
            RulesetError::Integrity(e) => {
                map.serialize_entry("kind", "integrity")?;
                map.serialize_entry("message", &e.message)?;
            }
        }
        map.end()
    }
}

impl std::fmt::Display for RulesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RulesetError::ParseError(e) => write!(f, "parse error: {e}"),
            RulesetError::Integrity(e) => write!(f, "integrity error: {e}"),
        }
    }
}

impl std::error::Error for RulesetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RulesetError::ParseError(e) => Some(e),
            RulesetError::Integrity(e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for RulesetError {
    fn from(e: serde_json::Error) -> Self {
        RulesetError::ParseError(e)
    }
}

impl From<IntegrityError> for RulesetError {
    fn from(e: IntegrityError) -> Self {
        RulesetError::Integrity(e)
    }
}

impl Ruleset {
    /// Parses point items and type profiles from JSON, validates referential
    /// integrity, and returns a Ruleset.
    pub fn from_json(
        id: &str,
        version: &str,
        point_items_json: &str,
        type_profiles_json: &str,
    ) -> Result<Self, RulesetError> {
        let items: Vec<PointItem> = serde_json::from_str(point_items_json)?;
        let types: Vec<EntityTypeProfile> = serde_json::from_str(type_profiles_json)?;

        // Detect duplicate point item IDs
        let mut duplicate_items = Vec::new();
        {
            let mut seen = BTreeSet::new();
            for item in &items {
                if !seen.insert(&item.id) {
                    duplicate_items.push(item.id.to_string());
                }
            }
        }

        // Detect duplicate type profile IDs
        let mut duplicate_types = Vec::new();
        {
            let mut seen = BTreeSet::new();
            for t in &types {
                if !seen.insert(&t.id) {
                    duplicate_types.push(t.id.to_string());
                }
            }
        }

        if !duplicate_items.is_empty() || !duplicate_types.is_empty() {
            let mut errors = Vec::new();
            for dup in &duplicate_items {
                errors.push(format!("duplicate point item ID: '{dup}'"));
            }
            for dup in &duplicate_types {
                errors.push(format!("duplicate type profile ID: '{dup}'"));
            }
            return Err(IntegrityError {
                message: errors.join("\n"),
            }
            .into());
        }

        let point_items: BTreeMap<Id, PointItem> = items
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect();
        let type_profiles: BTreeMap<Id, EntityTypeProfile> =
            types.into_iter().map(|t| (t.id.clone(), t)).collect();

        let ruleset = Self {
            id: id.to_string(),
            version: version.to_string(),
            point_items,
            type_profiles,
        };

        ruleset.validate_integrity()?;
        Ok(ruleset)
    }

    /// Checks referential integrity: prerequisite refs, incompatibility symmetry,
    /// and type profile trait refs.
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
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(IntegrityError {
                message: errors.join("\n"),
            })
        }
    }

    fn validate_prereq_refs(
        &self,
        prereq: &crate::types::Prereq,
        context_id: &Id,
        errors: &mut Vec<String>,
    ) {
        use crate::types::Prereq;
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
            // TODO: Validate House IDs when a house registry is added to Ruleset.
            // TODO: Validate AbilityMin/ArtMin IDs when ability/art registries are added.
            Prereq::House(_)
            | Prereq::AbilityMin { .. }
            | Prereq::ArtMin { .. }
            | Prereq::IsMagus => {}
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
    pub fn new(ruleset: Ruleset, i18n_json: &str) -> Result<Self, RulesetError> {
        let entries: BTreeMap<String, I18nEntry> = serde_json::from_str(i18n_json)?;
        let i18n: BTreeMap<Id, I18nEntry> =
            entries.into_iter().map(|(k, v)| (Id::new(k), v)).collect();
        Ok(Self { ruleset, i18n })
    }

    pub fn display_name(&self, id: &Id) -> Option<&str> {
        self.i18n.get(id).map(|e| e.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        "prerequisites": { "has": "virtue.the_gift" }
      },
      {
        "id": "virtue.gentle_gift",
        "kind": "virtue",
        "magnitude": "major",
        "category": "hermetic",
        "entity_kinds": ["character"],
        "prerequisites": { "has": "virtue.hermetic_magus" },
        "incompatible_with": ["flaw.blatant_gift"]
      },
      {
        "id": "flaw.blatant_gift",
        "kind": "flaw",
        "magnitude": "major",
        "category": "hermetic",
        "entity_kinds": ["character"],
        "prerequisites": { "has": "virtue.the_gift" },
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

    #[test]
    fn load_valid_ruleset() {
        let rs = Ruleset::from_json("arm5-core", "2024.1", VALID_ITEMS, VALID_TYPES).unwrap();
        assert_eq!(rs.point_items.len(), 5);
        assert_eq!(rs.type_profiles.len(), 1);
        assert_eq!(rs.id, "arm5-core");
    }

    #[test]
    fn missing_prereq_ref() {
        let items = r#"[{
          "id": "virtue.gentle_gift",
          "kind": "virtue",
          "magnitude": "major",
          "category": "hermetic",
          "entity_kinds": ["character"],
          "prerequisites": { "has": "virtue.nonexistent" }
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

    // --- Finding #21: forbidden_traits_unknown_ref ---

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

    // --- Finding #22: prereq_refs_recursive ---

    #[test]
    fn prereq_refs_recursive() {
        let items = r#"[{
          "id": "virtue.a",
          "kind": "virtue",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "prerequisites": {"all": [{"has": "virtue.nonexistent"}]}
        }]"#;

        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("virtue.nonexistent"),
            "should find nested unknown prereq ref: {msg}"
        );
    }

    // --- Finding #23: from_json_invalid_json ---

    #[test]
    fn from_json_invalid_json() {
        let result = Ruleset::from_json("test", "1", "NOT VALID JSON", "[]");
        assert!(result.is_err(), "malformed JSON should produce an error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("parse error"),
            "should be a parse error: {msg}"
        );
    }

    // --- Finding #24: localized_ruleset_invalid_json ---

    #[test]
    fn localized_ruleset_invalid_json() {
        let rs = Ruleset::from_json("arm5-core", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        let result = LocalizedRuleset::new(rs, "NOT VALID JSON");
        assert!(
            result.is_err(),
            "malformed i18n JSON should produce an error"
        );
    }

    // --- Duplicate point item IDs ---

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

    // --- Duplicate type profile IDs ---

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

    // --- RulesetError Display and source ---

    #[test]
    fn ruleset_error_display() {
        let parse_err = Ruleset::from_json("test", "1", "INVALID", "[]").unwrap_err();
        let msg = format!("{parse_err}");
        assert!(msg.contains("parse error"), "display: {msg}");

        let integrity_err = Ruleset::from_json(
            "test",
            "1",
            r#"[{"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": [], "prerequisites": {"has": "virtue.missing"}}]"#,
            "[]",
        )
        .unwrap_err();
        let msg = format!("{integrity_err}");
        assert!(msg.contains("integrity error"), "display: {msg}");
    }

    #[test]
    fn ruleset_error_source() {
        use std::error::Error;

        let parse_err = Ruleset::from_json("test", "1", "INVALID", "[]").unwrap_err();
        assert!(parse_err.source().is_some());

        let integrity_err = Ruleset::from_json(
            "test",
            "1",
            r#"[{"id":"virtue.a","kind":"virtue","magnitude":"minor","category":"general","entity_kinds":[],"prerequisites":{"has":"virtue.missing"}}]"#,
            "[]",
        )
        .unwrap_err();
        assert!(integrity_err.source().is_some());
    }

    #[test]
    fn ruleset_error_serialize() {
        let parse_err = Ruleset::from_json("test", "1", "INVALID", "[]").unwrap_err();
        let json = serde_json::to_string(&parse_err).unwrap();
        assert!(json.contains("parse_error"), "serialized: {json}");

        let integrity_err = Ruleset::from_json(
            "test",
            "1",
            r#"[{"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": [], "prerequisites": {"has": "virtue.missing"}}]"#,
            "[]",
        )
        .unwrap_err();
        let json = serde_json::to_string(&integrity_err).unwrap();
        assert!(json.contains("integrity"), "serialized: {json}");
    }
}
