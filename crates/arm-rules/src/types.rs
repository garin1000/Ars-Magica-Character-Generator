use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Slug-style identifier for rules entities (e.g. `virtue.gentle_gift`, `ability.awareness`).
/// Ordered for use as BTreeMap keys.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(String);

impl Id {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Id {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The two first-class entity types: characters and covenants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Character,
    Covenant,
}

impl fmt::Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityKind::Character => f.write_str("character"),
            EntityKind::Covenant => f.write_str("covenant"),
        }
    }
}

/// Point cost/grant magnitude. Free = 0, Minor = 1, Major = 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Magnitude {
    Free,
    Minor,
    Major,
}

impl Magnitude {
    pub fn points(self) -> u8 {
        match self {
            Magnitude::Free => 0,
            Magnitude::Minor => 1,
            Magnitude::Major => 3,
        }
    }
}

impl fmt::Display for Magnitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Magnitude::Free => f.write_str("free"),
            Magnitude::Minor => f.write_str("minor"),
            Magnitude::Major => f.write_str("major"),
        }
    }
}

/// Whether a rules item is positive (costs points) or negative (grants points).
/// Virtue/Boon are positive; Flaw/Hook are negative. Boon/Hook are covenant-specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Virtue,
    Flaw,
    Boon,
    Hook,
}

impl ItemKind {
    pub fn is_positive(self) -> bool {
        matches!(self, ItemKind::Virtue | ItemKind::Boon)
    }
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemKind::Virtue => f.write_str("virtue"),
            ItemKind::Flaw => f.write_str("flaw"),
            ItemKind::Boon => f.write_str("boon"),
            ItemKind::Hook => f.write_str("hook"),
        }
    }
}

/// Recursive boolean expression tree for prerequisites.
/// All = AND, Any = OR, None = NOR (none may be present).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Prereq {
    /// All children must be satisfied (AND).
    All(Vec<Prereq>),
    /// At least one child must be satisfied (OR).
    Any(Vec<Prereq>),
    /// None of the children may be satisfied (NOR).
    None(Vec<Prereq>),
    /// The entity must have the referenced item selected.
    Has(Id),
    /// The entity must belong to the referenced house.
    House(Id),
    /// The entity must have the referenced ability at or above the given score.
    AbilityMin { ability: Id, score: u8 },
    /// The entity must have the referenced art at or above the given score.
    ArtMin { art: Id, score: u8 },
    /// The entity must be a magus.
    IsMagus,
}

/// Describes a parameter slot on a parameterized virtue/flaw
/// (e.g. Puissant Ability requires an ability parameter).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterDef {
    pub key: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub domain: String,
}

/// Provenance into the authoritative Markdown rules source: the file name
/// (relative to `rules/source/<lang>/`, in the canonical-ID language) and the
/// inclusive `[start, end]` line range the item was extracted from.
///
/// Pipeline-generated, never hand-edited: re-running extraction recomputes the
/// line range, so it self-heals when the source Markdown is reformatted. There
/// is deliberately no rulebook page number — the Markdown source has lines, not
/// pages, and the source files are where edits actually happen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub file: String,
    pub lines: [u32; 2],
}

/// A virtue, flaw, boon, or hook with its mechanical metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointItem {
    pub id: Id,
    pub kind: ItemKind,
    pub magnitude: Magnitude,
    pub category: String,
    #[serde(default)]
    pub entity_kinds: BTreeSet<EntityKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerequisites: Option<Prereq>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub incompatible_with: BTreeSet<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ParameterDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// Whether The Gift is required, allowed, or forbidden for an entity type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GiftPolicy {
    Required,
    Allowed,
    Forbidden,
}

impl fmt::Display for GiftPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GiftPolicy::Required => f.write_str("required"),
            GiftPolicy::Allowed => f.write_str("allowed"),
            GiftPolicy::Forbidden => f.write_str("forbidden"),
        }
    }
}

/// Point limits for an entity type profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointBudget {
    pub virtue_points: u8,
    pub flaw_points: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_major_virtues: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_major_flaws: Option<u8>,
}

/// Data-driven profile defining constraints for an entity type
/// (grog, companion, magus, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTypeProfile {
    pub id: Id,
    pub budget: PointBudget,
    #[serde(default)]
    pub permitted_categories: BTreeSet<String>,
    #[serde(default)]
    pub forbidden_categories: BTreeSet<String>,
    #[serde(default)]
    pub required_traits: BTreeSet<Id>,
    #[serde(default)]
    pub forbidden_traits: BTreeSet<Id>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gift_policy: Option<GiftPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gift_id: Option<Id>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub gift_categories: BTreeSet<String>,
    pub creation_phases: Vec<String>,
}

/// A user's choice of a virtue/flaw with optional parameters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Selection {
    #[serde(rename = "ref")]
    pub item_ref: Id,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, Id>,
}

impl Selection {
    /// Creates a new selection with no parameters.
    pub fn new(item_ref: Id) -> Self {
        Self {
            item_ref,
            params: BTreeMap::new(),
        }
    }
}

/// The save format for a character or covenant under construction.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Entity {
    pub schema_version: u32,
    pub ruleset: RulesetRef,
    pub entity_kind: EntityKind,
    pub type_id: Id,
    #[serde(default)]
    pub selections: Vec<Selection>,
}

impl Serialize for Entity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut sorted_selections = self.selections.clone();
        sorted_selections.sort();

        let mut state = serializer.serialize_struct("Entity", 5)?;
        state.serialize_field("schema_version", &self.schema_version)?;
        state.serialize_field("ruleset", &self.ruleset)?;
        state.serialize_field("entity_kind", &self.entity_kind)?;
        state.serialize_field("type_id", &self.type_id)?;
        state.serialize_field("selections", &sorted_selections)?;
        state.end()
    }
}

impl Entity {
    /// Creates a new entity with `schema_version` 1 and empty selections.
    pub fn new(entity_kind: EntityKind, type_id: Id, ruleset_ref: RulesetRef) -> Self {
        Self {
            schema_version: 1,
            ruleset: ruleset_ref,
            entity_kind,
            type_id,
            selections: Vec::new(),
        }
    }

    /// Sort selections by `item_ref` for canonical serialization.
    pub fn normalize(&mut self) {
        self.selections.sort();
    }
}

/// Identifies which ruleset version an entity was built against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulesetRef {
    pub id: Id,
    pub version: String,
}

/// Localized display text for a rules item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct I18nEntry {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Controls how validation results are enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationMode {
    /// Blocks illegal states (guided + direct-validated modes).
    Enforced,
    /// Shows violations as non-blocking warnings.
    Advisory,
    /// Suppresses validation display (unchecked mode).
    Silent,
}

impl fmt::Display for ValidationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationMode::Enforced => f.write_str("enforced"),
            ValidationMode::Advisory => f.write_str("advisory"),
            ValidationMode::Silent => f.write_str("silent"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn id_display_and_equality() {
        let id1 = Id::new("virtue.gentle_gift");
        let id2 = Id::new("virtue.gentle_gift");
        let id3 = Id::new("flaw.blatant_gift");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(id1.as_str(), "virtue.gentle_gift");
        assert_eq!(format!("{id1}"), "virtue.gentle_gift");
    }

    #[test]
    fn id_ordering_for_btreemap() {
        let a = Id::new("ability.awareness");
        let b = Id::new("virtue.puissant_ability");
        assert!(a < b);
    }

    #[test]
    fn magnitude_points() {
        assert_eq!(Magnitude::Free.points(), 0);
        assert_eq!(Magnitude::Minor.points(), 1);
        assert_eq!(Magnitude::Major.points(), 3);
    }

    #[test]
    fn item_kind_polarity() {
        assert!(ItemKind::Virtue.is_positive());
        assert!(ItemKind::Boon.is_positive());
        assert!(!ItemKind::Flaw.is_positive());
        assert!(!ItemKind::Hook.is_positive());
    }

    #[test]
    fn point_item_roundtrip() {
        let json = r#"{
          "id": "virtue.gentle_gift",
          "kind": "virtue",
          "magnitude": "major",
          "category": "hermetic",
          "entity_kinds": ["character"],
          "prerequisites": { "has": "virtue.hermetic_magus" },
          "incompatible_with": ["flaw.blatant_gift"],
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [120, 135] }
        }"#;

        let item: PointItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, Id::new("virtue.gentle_gift"));
        assert_eq!(item.kind, ItemKind::Virtue);
        assert_eq!(item.magnitude, Magnitude::Major);
        assert_eq!(item.category, "hermetic");
        assert_eq!(item.entity_kinds, BTreeSet::from([EntityKind::Character]));
        assert_eq!(
            item.prerequisites,
            Some(Prereq::Has(Id::new("virtue.hermetic_magus")))
        );
        assert_eq!(
            item.incompatible_with,
            BTreeSet::from([Id::new("flaw.blatant_gift")])
        );
        assert_eq!(
            item.source,
            Some(SourceRef {
                file: "Ars Magica - Definitive Edition (Core Rules).md".to_string(),
                lines: [120, 135]
            })
        );

        let reserialized = serde_json::to_string(&item).unwrap();
        let roundtripped: PointItem = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(item, roundtripped);
    }

    #[test]
    fn point_item_with_parameters() {
        let json = r#"{
          "id": "virtue.puissant_ability",
          "kind": "virtue",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [240, 251] }
        }"#;

        let item: PointItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.parameters.len(), 1);
        assert_eq!(item.parameters[0].key, "ability");
        assert_eq!(item.parameters[0].domain, "ability");
    }

    #[test]
    fn prereq_complex_expression_roundtrip() {
        let json = r#"{
          "all": [
            { "has": "virtue.hermetic_magus" },
            { "any": [
              { "house": "house.bjornaer" },
              { "ability_min": { "ability": "ability.animal_ken", "score": 1 } }
            ]}
          ]
        }"#;

        let prereq: Prereq = serde_json::from_str(json).unwrap();
        let expected = Prereq::All(vec![
            Prereq::Has(Id::new("virtue.hermetic_magus")),
            Prereq::Any(vec![
                Prereq::House(Id::new("house.bjornaer")),
                Prereq::AbilityMin {
                    ability: Id::new("ability.animal_ken"),
                    score: 1,
                },
            ]),
        ]);
        assert_eq!(prereq, expected);

        let reserialized = serde_json::to_string(&prereq).unwrap();
        let roundtripped: Prereq = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(prereq, roundtripped);
    }

    #[test]
    fn prereq_none_variant() {
        let json = r#"{ "none": [{ "has": "flaw.blatant_gift" }] }"#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(
            prereq,
            Prereq::None(vec![Prereq::Has(Id::new("flaw.blatant_gift"))])
        );
    }

    #[test]
    fn prereq_is_magus() {
        let json = r#""is_magus""#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(prereq, Prereq::IsMagus);
    }

    #[test]
    fn entity_type_profile_roundtrip() {
        let json = r#"{
          "id": "companion",
          "budget": {
            "virtue_points": 10,
            "flaw_points": 10,
            "max_major_virtues": 1,
            "max_major_flaws": null
          },
          "permitted_categories": ["general", "social_status", "supernatural"],
          "forbidden_categories": ["hermetic"],
          "required_traits": [],
          "forbidden_traits": ["virtue.the_gift"],
          "gift_policy": "forbidden",
          "creation_phases": [
            "concept", "type", "characteristics", "virtues_flaws",
            "abilities", "personality_reputations"
          ]
        }"#;

        let profile: EntityTypeProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.id, Id::new("companion"));
        assert_eq!(profile.budget.virtue_points, 10);
        assert_eq!(profile.budget.flaw_points, 10);
        assert_eq!(profile.budget.max_major_virtues, Some(1));
        assert_eq!(profile.budget.max_major_flaws, None);
        assert_eq!(profile.gift_policy, Some(GiftPolicy::Forbidden));
        assert!(profile.forbidden_categories.contains("hermetic"));
        assert_eq!(profile.creation_phases.len(), 6);

        let reserialized = serde_json::to_string(&profile).unwrap();
        let roundtripped: EntityTypeProfile = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(profile, roundtripped);
    }

    #[test]
    fn entity_type_profile_without_gift_policy() {
        let json = r#"{
          "id": "standard_covenant",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "creation_phases": ["concept", "boons_hooks"]
        }"#;

        let profile: EntityTypeProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.gift_policy, None);
    }

    #[test]
    fn entity_save_roundtrip() {
        let entity = Entity {
            schema_version: 1,
            ruleset: RulesetRef {
                id: Id::new("arm5-core"),
                version: "2024.1".into(),
            },
            entity_kind: EntityKind::Character,
            type_id: Id::new("companion"),
            selections: vec![
                Selection {
                    item_ref: Id::new("flaw.deficient_technique"),
                    params: BTreeMap::from([("technique".into(), Id::new("art.creo"))]),
                },
                Selection {
                    item_ref: Id::new("virtue.puissant_ability"),
                    params: BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
                },
            ],
        };

        let json = serde_json::to_string_pretty(&entity).unwrap();
        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);

        assert!(json.contains(r#""schema_version": 1"#));
        assert!(json.contains(r#""ref": "flaw.deficient_technique""#));
    }

    #[test]
    fn i18n_entry_roundtrip() {
        let json = r#"{
          "name": "Puissant (Ability)",
          "summary": "+2 to all rolls with one Ability.",
          "description": "You are particularly adept with one Ability."
        }"#;

        let entry: I18nEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.name, "Puissant (Ability)");
        assert!(entry.summary.is_some());
        assert!(entry.description.is_some());
    }

    #[test]
    fn i18n_entry_minimal() {
        let json = r#"{ "name": "Gentle Gift" }"#;
        let entry: I18nEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.name, "Gentle Gift");
        assert_eq!(entry.summary, None);
    }

    #[test]
    fn validation_mode_roundtrip() {
        for (json, expected) in [
            (r#""enforced""#, ValidationMode::Enforced),
            (r#""advisory""#, ValidationMode::Advisory),
            (r#""silent""#, ValidationMode::Silent),
        ] {
            let mode: ValidationMode = serde_json::from_str(json).unwrap();
            assert_eq!(mode, expected);
        }
    }

    #[test]
    fn selections_sorted_by_ref_in_btreemap() {
        let mut map = BTreeMap::new();
        map.insert(Id::new("virtue.puissant_ability"), ());
        map.insert(Id::new("flaw.blatant_gift"), ());
        map.insert(Id::new("ability.awareness"), ());

        let keys: Vec<_> = map.keys().map(Id::as_str).collect();
        assert_eq!(
            keys,
            vec![
                "ability.awareness",
                "flaw.blatant_gift",
                "virtue.puissant_ability"
            ]
        );
    }

    // --- Finding #25: covenant_entity_roundtrip ---

    #[test]
    fn covenant_entity_roundtrip() {
        let entity = Entity {
            schema_version: 1,
            ruleset: RulesetRef {
                id: Id::new("arm5-core"),
                version: "2024.1".into(),
            },
            entity_kind: EntityKind::Covenant,
            type_id: Id::new("standard_covenant"),
            selections: vec![Selection {
                item_ref: Id::new("boon.healthy_feature"),
                params: BTreeMap::new(),
            }],
        };

        let json = serde_json::to_string_pretty(&entity).unwrap();
        assert!(json.contains(r#""entity_kind": "covenant""#));

        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);
    }

    // --- Finding #26: boon_hook_deserialization ---

    #[test]
    fn boon_hook_deserialization() {
        let boon_json = r#"{
          "id": "boon.healthy_feature",
          "kind": "boon",
          "magnitude": "minor",
          "category": "site",
          "entity_kinds": ["covenant"]
        }"#;
        let boon: PointItem = serde_json::from_str(boon_json).unwrap();
        assert_eq!(boon.kind, ItemKind::Boon);
        assert_eq!(boon.magnitude, Magnitude::Minor);

        let hook_json = r#"{
          "id": "hook.road",
          "kind": "hook",
          "magnitude": "minor",
          "category": "site",
          "entity_kinds": ["covenant"]
        }"#;
        let hook: PointItem = serde_json::from_str(hook_json).unwrap();
        assert_eq!(hook.kind, ItemKind::Hook);
    }

    // --- Display impls coverage ---

    #[test]
    fn entity_kind_display() {
        assert_eq!(format!("{}", EntityKind::Character), "character");
        assert_eq!(format!("{}", EntityKind::Covenant), "covenant");
    }

    #[test]
    fn magnitude_display() {
        assert_eq!(format!("{}", Magnitude::Free), "free");
        assert_eq!(format!("{}", Magnitude::Minor), "minor");
        assert_eq!(format!("{}", Magnitude::Major), "major");
    }

    #[test]
    fn item_kind_display() {
        assert_eq!(format!("{}", ItemKind::Virtue), "virtue");
        assert_eq!(format!("{}", ItemKind::Flaw), "flaw");
        assert_eq!(format!("{}", ItemKind::Boon), "boon");
        assert_eq!(format!("{}", ItemKind::Hook), "hook");
    }

    #[test]
    fn gift_policy_display() {
        assert_eq!(format!("{}", GiftPolicy::Required), "required");
        assert_eq!(format!("{}", GiftPolicy::Allowed), "allowed");
        assert_eq!(format!("{}", GiftPolicy::Forbidden), "forbidden");
    }

    #[test]
    fn validation_mode_display() {
        assert_eq!(format!("{}", ValidationMode::Enforced), "enforced");
        assert_eq!(format!("{}", ValidationMode::Advisory), "advisory");
        assert_eq!(format!("{}", ValidationMode::Silent), "silent");
    }

    #[test]
    fn id_from_string() {
        let id: Id = String::from("test.id").into();
        assert_eq!(id.as_str(), "test.id");
    }

    #[test]
    fn id_from_str_ref() {
        let id: Id = Id::from("test.id");
        assert_eq!(id.as_str(), "test.id");
    }

    #[test]
    fn id_as_ref() {
        let id = Id::new("test.id");
        let s: &str = id.as_ref();
        assert_eq!(s, "test.id");
    }

    #[test]
    fn selection_new() {
        let sel = Selection::new(Id::new("virtue.a"));
        assert_eq!(sel.item_ref, Id::new("virtue.a"));
        assert!(sel.params.is_empty());
    }

    #[test]
    fn entity_new() {
        let entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef {
                id: Id::new("arm5-core"),
                version: "1.0".into(),
            },
        );
        assert_eq!(entity.schema_version, 1);
        assert_eq!(entity.entity_kind, EntityKind::Character);
        assert_eq!(entity.type_id, Id::new("companion"));
        assert!(entity.selections.is_empty());
    }

    #[test]
    fn prereq_art_min_roundtrip() {
        let json = r#"{"art_min": {"art": "art.creo", "score": 5}}"#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(
            prereq,
            Prereq::ArtMin {
                art: Id::new("art.creo"),
                score: 5,
            }
        );

        let reserialized = serde_json::to_string(&prereq).unwrap();
        let roundtripped: Prereq = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(prereq, roundtripped);
    }
}
