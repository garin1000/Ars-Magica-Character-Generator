use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Slug-style identifier for rules entities (e.g. `virtue.gentle_gift`, `ability.awareness`).
/// Ordered for use as `BTreeMap` keys.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(String);

impl Id {
    /// Creates an identifier from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrows the identifier as a string slice.
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

/// Enables map lookups (`BTreeMap<Id, _>::get`) keyed by a plain `&str` without
/// allocating an [`Id`].
impl Borrow<str> for Id {
    fn borrow(&self) -> &str {
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
    /// A grog, companion, mythic companion, or magus.
    Character,
    /// A covenant.
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
/// Ordered `Free < Minor < Major` to reflect increasing point weight.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2774 ("Major Virtues
/// cost three points ... Minor Virtues and Flaws cost and grant ... one point");
/// :2209; Free virtues at :2886-2896.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Magnitude {
    /// Costs/grants 0 points.
    Free,
    /// Costs/grants 1 point.
    Minor,
    /// Costs/grants 3 points.
    Major,
}

impl Magnitude {
    /// Returns the point weight of this magnitude (Free 0, Minor 1, Major 3).
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
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2774 ("Virtues cost
/// points, while Flaws grant points").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// A character virtue (positive).
    Virtue,
    /// A character flaw (negative).
    Flaw,
    /// A covenant boon (positive).
    Boon,
    /// A covenant hook (negative).
    Hook,
}

impl ItemKind {
    /// Returns `true` for kinds that cost points (Virtue, Boon).
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
///
/// `House`, `AbilityMin`, and `ArtMin` reference IDs (house / ability / art)
/// for which no registry yet exists; those refs are intentionally NOT checked
/// for referential integrity (see [`crate::ruleset::Ruleset::validate_integrity`]).
///
/// # JSON shape
///
/// Adjacently tagged: every variant is a uniform object carrying a `kind`
/// discriminant, and data variants put their payload under `value`:
///
/// ```json
/// { "kind": "has",   "value": "virtue.x" }
/// { "kind": "all",   "value": [ /* nested prereqs */ ] }
/// { "kind": "any",   "value": [ /* ... */ ] }
/// { "kind": "none",  "value": [ /* ... */ ] }
/// { "kind": "house", "value": "house.x" }
/// { "kind": "ability_min", "value": { "ability": "ability.x", "score": 1 } }
/// { "kind": "art_min",     "value": { "art": "art.x", "score": 1 } }
/// { "kind": "is_magus" }
/// ```
///
/// Adjacent tagging is used rather than serde's internal tagging
/// (`#[serde(tag = "kind")]`) because `Has` and `House` are newtype variants
/// wrapping a scalar (a string `Id`): internal tagging cannot represent a
/// variant whose content is a non-map value, so it rejects those two variants
/// at compile time. Adjacent tagging supports every variant shape — unit,
/// newtype, tuple, and struct — while still giving each variant a uniform
/// object form with a `kind` discriminant for the TS/Svelte consumer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Prereq {
    /// All children must be satisfied (AND).
    All(Vec<Prereq>),
    /// At least one child must be satisfied (OR).
    Any(Vec<Prereq>),
    /// None of the children may be satisfied (NOR).
    None(Vec<Prereq>),
    /// The entity must have the referenced item selected.
    Has(Id),
    /// The entity must belong to the referenced house. Currently unevaluable
    /// (the entity carries no house metadata yet).
    House(Id),
    /// The entity must have the referenced ability at or above the given score.
    /// Currently unevaluable (no ability scores on the entity yet).
    AbilityMin { ability: Id, score: u8 },
    /// The entity must have the referenced art at or above the given score.
    /// Currently unevaluable (no art scores on the entity yet).
    ArtMin { art: Id, score: u8 },
    /// The entity must be a magus. Evaluated against the type profile's
    /// explicit `is_magus` flag.
    IsMagus,
}

/// The kind of value a parameter slot carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamType {
    /// The parameter value is a reference to another rules entity (an [`Id`]).
    Ref,
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamType::Ref => f.write_str("ref"),
        }
    }
}

/// The domain a parameter value's [`Id`] must belong to.
///
/// `Ability` and `Art` have no in-engine registry yet, so values in those
/// domains are accepted without referential-integrity checks. `Item` resolves
/// against the ruleset's point-item registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterDomain {
    /// Value is an ability id (e.g. `ability.awareness`). No registry yet.
    Ability,
    /// Value is an art id (e.g. `art.creo`). No registry yet.
    Art,
    /// Value is a point-item id; resolved against the ruleset's point items.
    Item,
}

impl ParameterDomain {
    /// Returns `true` if values in this domain are resolved against the
    /// ruleset's point-item registry. `Ability` and `Art` have no registry yet.
    pub fn resolves_against_items(self) -> bool {
        matches!(self, ParameterDomain::Item)
    }
}

impl fmt::Display for ParameterDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParameterDomain::Ability => f.write_str("ability"),
            ParameterDomain::Art => f.write_str("art"),
            ParameterDomain::Item => f.write_str("item"),
        }
    }
}

/// Describes a parameter slot on a parameterized virtue/flaw
/// (e.g. Puissant Ability requires an `ability` parameter).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterDef {
    /// Stable key the selection's `params` map must use.
    pub key: String,
    /// The kind of value the parameter carries.
    #[serde(rename = "type")]
    pub param_type: ParamType,
    /// The domain the parameter value's id must belong to.
    pub domain: ParameterDomain,
}

impl ParameterDef {
    /// Creates a parameter definition.
    pub fn new(key: impl Into<String>, param_type: ParamType, domain: ParameterDomain) -> Self {
        Self {
            key: key.into(),
            param_type,
            domain,
        }
    }
}

/// An inclusive line range `[start, end]` into a Markdown source file.
/// Serialized as a two-element JSON array to match the shipped rules data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "[u32; 2]", into = "[u32; 2]")]
pub struct LineRange {
    /// First line of the range (inclusive, 1-based).
    pub start: u32,
    /// Last line of the range (inclusive, 1-based).
    pub end: u32,
}

impl LineRange {
    /// Creates a line range.
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Returns `true` if `start <= end`.
    pub fn is_valid(&self) -> bool {
        self.start <= self.end
    }
}

impl From<[u32; 2]> for LineRange {
    fn from([start, end]: [u32; 2]) -> Self {
        Self { start, end }
    }
}

impl From<LineRange> for [u32; 2] {
    fn from(r: LineRange) -> Self {
        [r.start, r.end]
    }
}

/// Provenance into the authoritative Markdown rules source: the file name
/// (relative to `rules/source/<lang>/`, in the canonical-ID language) and the
/// inclusive line range the item was extracted from.
///
/// Pipeline-generated, never hand-edited: re-running extraction recomputes the
/// line range, so it self-heals when the source Markdown is reformatted. There
/// is deliberately no rulebook page number — the Markdown source has lines, not
/// pages, and the source files are where edits actually happen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    /// Basename of the Markdown source file.
    pub file: String,
    /// Inclusive line range the item was extracted from.
    pub lines: LineRange,
}

impl SourceRef {
    /// Creates a source reference.
    pub fn new(file: impl Into<String>, lines: LineRange) -> Self {
        Self {
            file: file.into(),
            lines,
        }
    }
}

/// A virtue, flaw, boon, or hook with its mechanical metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointItem {
    /// Stable slug identifier.
    pub id: Id,
    /// Whether this is a virtue, flaw, boon, or hook.
    pub kind: ItemKind,
    /// Point weight (free/minor/major).
    pub magnitude: Magnitude,
    /// Grouping category used by type-profile permit/forbid rules
    /// (e.g. `general`, `hermetic`, `social_status`).
    pub category: String,
    /// Entity kinds this item may be selected for. Empty means any kind.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub entity_kinds: BTreeSet<EntityKind>,
    /// Prerequisite expression that must hold for this item to be legal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerequisites: Option<Prereq>,
    /// Items that may not be selected alongside this one (must be symmetric).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub incompatible_with: BTreeSet<Id>,
    /// Parameter slots a selection of this item must fill.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ParameterDef>,
    /// Provenance into the Markdown source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

impl PointItem {
    /// Sorts the `parameters` vector by key for canonical serialization.
    pub fn normalize(&mut self) {
        self.parameters.sort_by(|a, b| a.key.cmp(&b.key));
    }
}

/// Whether The Gift is required, allowed, or forbidden for an entity type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GiftPolicy {
    /// The Gift must be present (the type denotes a magus).
    Required,
    /// The Gift may optionally be present.
    Allowed,
    /// The Gift must not be present.
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
    /// Maximum total virtue points.
    pub virtue_points: u8,
    /// Maximum total flaw points.
    pub flaw_points: u8,
    /// Optional cap on the number of Major virtues.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_major_virtues: Option<u8>,
    /// Optional cap on the number of Major flaws.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_major_flaws: Option<u8>,
    /// Optional cap on the number of Minor flaws (hard rule).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2774 ("A central
    /// character may have up to ten points of Flaws, but no more than five Minor
    /// Flaws"); grogs :1009 ("no more than three Minor Flaws").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_minor_flaws: Option<u8>,
    /// Per-category flaw count caps (e.g. Personality, Story). Each entry names
    /// the flaw category it applies to as DATA, so the engine never hardcodes a
    /// category slug. `major_only` restricts the count to Major-magnitude flaws;
    /// `hard` makes the cap a blocking error (otherwise a non-blocking warning).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2820 ("A
    /// character may not have more than one Major Personality Flaw"; "A
    /// character should normally not have more than two Personality Flaws in
    /// total"); :2818 ("A character should not have more than one Story Flaw").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flaw_category_caps: Vec<FlawCategoryCap>,
}

/// A cap on how many flaws of a given category an entity may take.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlawCategoryCap {
    /// The flaw category this cap applies to (e.g. `personality`, `story`).
    pub category: String,
    /// Maximum allowed count.
    pub max: u8,
    /// If true, only Major-magnitude flaws count toward this cap.
    #[serde(default, skip_serializing_if = "is_false")]
    pub major_only: bool,
    /// If true the cap is a blocking error; otherwise a non-blocking warning.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hard: bool,
}

/// `skip_serializing_if` predicate: omits a `bool` field from canonical JSON
/// when it holds its `false` default, keeping the common case out of the data.
fn is_false(b: &bool) -> bool {
    !*b
}

/// Data-driven profile defining constraints for an entity type
/// (grog, companion, magus, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTypeProfile {
    /// Stable slug identifier of the type (e.g. `grog`, `companion`, `magus`).
    pub id: Id,
    /// Point budget and caps.
    pub budget: PointBudget,
    /// If non-empty, only items whose `category` is listed may be selected.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub permitted_categories: BTreeSet<String>,
    /// Items whose `category` is listed may never be selected.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub forbidden_categories: BTreeSet<String>,
    /// Item ids that must be selected.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub required_traits: BTreeSet<Id>,
    /// Item ids that may never be selected.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub forbidden_traits: BTreeSet<Id>,
    /// Whether this character type is a Hermetic magus (possesses the Hermetic
    /// Magus Social Status). Independent of `gift_policy`: an unGifted Redcap is a
    /// companion (not a magus) and a Gifted hedge wizard has The Gift but is not a
    /// magus. Drives `Prereq::IsMagus`. Defaults to false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_magus: bool,
    /// Whether The Gift is required/allowed/forbidden. `None` = not applicable
    /// (e.g. covenants).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gift_policy: Option<GiftPolicy>,
    /// The specific item id that represents The Gift, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gift_id: Option<Id>,
    /// Categories that count as carrying The Gift (e.g. `hermetic`).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub gift_categories: BTreeSet<String>,
    /// Ordered creation phases the guided wizard walks through.
    pub creation_phases: Vec<String>,
}

/// A user's choice of a virtue/flaw with optional parameters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Selection {
    /// The selected item's id. Serialized as `ref` (Rust keyword avoidance) —
    /// the JSON/save key is `ref`, not `item_ref`.
    #[serde(rename = "ref")]
    pub item_ref: Id,
    /// Parameter values keyed by [`ParameterDef::key`].
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

    /// Creates a new selection with the given parameter values.
    pub fn with_params(item_ref: Id, params: BTreeMap<String, Id>) -> Self {
        Self { item_ref, params }
    }
}

/// The save format for a character or covenant under construction.
///
/// Saves store choices, not resolved values; `selections` is kept sorted on
/// serialization (see [`Entity::normalize`]) for zero-noise git diffs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    /// Save-format schema version.
    pub schema_version: u32,
    /// The ruleset (id + version) this entity was built against.
    pub ruleset: RulesetRef,
    /// Whether this is a character or covenant.
    pub entity_kind: EntityKind,
    /// The entity type profile id (e.g. `companion`).
    pub type_id: Id,
    /// The user's selections. Kept sorted via [`Entity::normalize`].
    #[serde(default)]
    pub selections: Vec<Selection>,
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

    /// Sort selections by `item_ref` (then params) for canonical serialization.
    pub fn normalize(&mut self) {
        self.selections.sort();
    }
}

/// Identifies which ruleset (id + version) an entity was built against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulesetRef {
    /// Ruleset identifier.
    pub id: Id,
    /// Ruleset version string.
    pub version: String,
}

impl RulesetRef {
    /// Creates a ruleset reference.
    pub fn new(id: Id, version: impl Into<String>) -> Self {
        Self {
            id,
            version: version.into(),
        }
    }
}

/// Localized display text for a rules item, keyed elsewhere by [`Id`].
///
/// These are **rules-domain** localized strings (the rulebook text the frontend
/// renders for an item), sourced from `rules/i18n/<lang>/`. They are distinct
/// from Fluent UI chrome (`locales/<lang>/*.ftl`): the two-file separation in
/// CLAUDE.md keeps rules text out of the UI-string layer and vice versa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct I18nEntry {
    /// Human-readable display name.
    pub name: String,
    /// Optional short summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Optional full description.
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

    /// Guards against drift between a scalar enum's hand-written `Display` and
    /// its `#[serde(rename_all = "snake_case")]` scalar form. Both feed the
    /// Fluent key mapping, so they must agree for every variant. Asserts
    /// `serde scalar == Display` exhaustively.
    #[test]
    fn display_matches_serde_scalar_for_every_enum() {
        fn check<T: Serialize + std::fmt::Display>(variant: T) {
            let serde_scalar = serde_json::to_value(&variant)
                .unwrap()
                .as_str()
                .expect("scalar enum serializes to a JSON string")
                .to_string();
            assert_eq!(serde_scalar, variant.to_string());
        }

        check(EntityKind::Character);
        check(EntityKind::Covenant);
        check(Magnitude::Free);
        check(Magnitude::Minor);
        check(Magnitude::Major);
        check(ItemKind::Virtue);
        check(ItemKind::Flaw);
        check(ItemKind::Boon);
        check(ItemKind::Hook);
        check(GiftPolicy::Required);
        check(GiftPolicy::Allowed);
        check(GiftPolicy::Forbidden);
        check(ValidationMode::Enforced);
        check(ValidationMode::Advisory);
        check(ValidationMode::Silent);
        check(ParamType::Ref);
        check(ParameterDomain::Ability);
        check(ParameterDomain::Art);
        check(ParameterDomain::Item);
    }

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
    fn id_borrow_str_lookup() {
        let mut map = BTreeMap::new();
        map.insert(Id::new("virtue.a"), 1);
        // Lookup with a plain &str, no Id allocation.
        assert_eq!(map.get("virtue.a"), Some(&1));
    }

    #[test]
    fn magnitude_points() {
        assert_eq!(Magnitude::Free.points(), 0);
        assert_eq!(Magnitude::Minor.points(), 1);
        assert_eq!(Magnitude::Major.points(), 3);
    }

    #[test]
    fn magnitude_ordering() {
        assert!(Magnitude::Free < Magnitude::Minor);
        assert!(Magnitude::Minor < Magnitude::Major);
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
          "prerequisites": { "kind": "has", "value": "virtue.hermetic_magus" },
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
                lines: LineRange::new(120, 135)
            })
        );

        let reserialized = serde_json::to_string(&item).unwrap();
        let roundtripped: PointItem = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(item, roundtripped);
    }

    #[test]
    fn line_range_serializes_as_array() {
        let source = SourceRef::new("file.md", LineRange::new(10, 20));
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("[10,20]"), "lines as array: {json}");
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
        assert_eq!(item.parameters[0].param_type, ParamType::Ref);
        assert_eq!(item.parameters[0].domain, ParameterDomain::Ability);
    }

    #[test]
    fn point_item_normalize_sorts_parameters() {
        let mut item: PointItem = serde_json::from_str(
            r#"{
              "id": "virtue.x",
              "kind": "virtue",
              "magnitude": "minor",
              "category": "general",
              "entity_kinds": ["character"],
              "parameters": [
                { "key": "second", "type": "ref", "domain": "art" },
                { "key": "first", "type": "ref", "domain": "ability" }
              ]
            }"#,
        )
        .unwrap();
        item.normalize();
        let keys: Vec<&str> = item.parameters.iter().map(|p| p.key.as_str()).collect();
        assert_eq!(keys, vec!["first", "second"]);
    }

    #[test]
    fn prereq_complex_expression_roundtrip() {
        let json = r#"{
          "kind": "all",
          "value": [
            { "kind": "has", "value": "virtue.hermetic_magus" },
            { "kind": "any", "value": [
              { "kind": "house", "value": "house.bjornaer" },
              { "kind": "ability_min", "value": { "ability": "ability.animal_ken", "score": 1 } }
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
        let json =
            r#"{ "kind": "none", "value": [{ "kind": "has", "value": "flaw.blatant_gift" }] }"#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(
            prereq,
            Prereq::None(vec![Prereq::Has(Id::new("flaw.blatant_gift"))])
        );
    }

    #[test]
    fn prereq_is_magus() {
        let json = r#"{ "kind": "is_magus" }"#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(prereq, Prereq::IsMagus);

        // The unit variant round-trips with no `value` key.
        let reserialized = serde_json::to_string(&prereq).unwrap();
        assert_eq!(reserialized, r#"{"kind":"is_magus"}"#);
        let roundtripped: Prereq = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(prereq, roundtripped);
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
    fn entity_type_profile_is_magus_defaults_false_and_omitted() {
        // Absent `is_magus` deserializes to false...
        let json = r#"{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "creation_phases": []
        }"#;
        let profile: EntityTypeProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.is_magus);

        // ...and a false flag is omitted from canonical JSON.
        let serialized = serde_json::to_string(&profile).unwrap();
        assert!(
            !serialized.contains("is_magus"),
            "false is_magus must be omitted: {serialized}"
        );
    }

    #[test]
    fn entity_type_profile_is_magus_true_roundtrip() {
        let json = r#"{
          "id": "magus",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "is_magus": true,
          "creation_phases": []
        }"#;
        let profile: EntityTypeProfile = serde_json::from_str(json).unwrap();
        assert!(profile.is_magus);

        let serialized = serde_json::to_string(&profile).unwrap();
        assert!(serialized.contains(r#""is_magus":true"#), "{serialized}");
        let roundtripped: EntityTypeProfile = serde_json::from_str(&serialized).unwrap();
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
            ruleset: RulesetRef::new(Id::new("arm5-core"), "2024.1"),
            entity_kind: EntityKind::Character,
            type_id: Id::new("companion"),
            selections: vec![
                Selection::with_params(
                    Id::new("flaw.deficient_technique"),
                    BTreeMap::from([("technique".into(), Id::new("art.creo"))]),
                ),
                Selection::with_params(
                    Id::new("virtue.puissant_ability"),
                    BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
                ),
            ],
        };

        let json = serde_json::to_string_pretty(&entity).unwrap();
        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);

        assert!(json.contains(r#""schema_version": 1"#));
        assert!(json.contains(r#""ref": "flaw.deficient_technique""#));
    }

    #[test]
    fn entity_serializes_selections_sorted() {
        let entity = Entity {
            schema_version: 1,
            ruleset: RulesetRef::new(Id::new("arm5-core"), "2024.1"),
            entity_kind: EntityKind::Character,
            type_id: Id::new("companion"),
            selections: vec![
                Selection::new(Id::new("virtue.tough")),
                Selection::new(Id::new("flaw.poor_student")),
                Selection::new(Id::new("ability.awareness")),
            ],
        };

        // Serialization is canonical only after normalize(); derive-based
        // Serialize emits selections in their in-memory order.
        let mut normalized = entity.clone();
        normalized.normalize();
        let json = serde_json::to_string(&normalized).unwrap();

        let first = json.find("ability.awareness").unwrap();
        let second = json.find("flaw.poor_student").unwrap();
        let third = json.find("virtue.tough").unwrap();
        assert!(
            first < second && second < third,
            "selections sorted: {json}"
        );
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

    #[test]
    fn covenant_entity_roundtrip() {
        let entity = Entity {
            schema_version: 1,
            ruleset: RulesetRef::new(Id::new("arm5-core"), "2024.1"),
            entity_kind: EntityKind::Covenant,
            type_id: Id::new("standard_covenant"),
            selections: vec![Selection::new(Id::new("boon.healthy_feature"))],
        };

        let json = serde_json::to_string_pretty(&entity).unwrap();
        assert!(json.contains(r#""entity_kind": "covenant""#));

        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);
    }

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
    fn param_type_and_domain_display() {
        assert_eq!(format!("{}", ParamType::Ref), "ref");
        assert_eq!(format!("{}", ParameterDomain::Ability), "ability");
        assert_eq!(format!("{}", ParameterDomain::Art), "art");
        assert_eq!(format!("{}", ParameterDomain::Item), "item");
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
    fn selection_with_params() {
        let sel = Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
        );
        assert_eq!(
            sel.params.get("ability"),
            Some(&Id::new("ability.awareness"))
        );
    }

    #[test]
    fn entity_new() {
        let entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "1.0"),
        );
        assert_eq!(entity.schema_version, 1);
        assert_eq!(entity.entity_kind, EntityKind::Character);
        assert_eq!(entity.type_id, Id::new("companion"));
        assert!(entity.selections.is_empty());
    }

    #[test]
    fn ruleset_ref_new() {
        let r = RulesetRef::new(Id::new("arm5-core"), "2024.1");
        assert_eq!(r.id, Id::new("arm5-core"));
        assert_eq!(r.version, "2024.1");
    }

    #[test]
    fn source_ref_new() {
        let s = SourceRef::new("f.md", LineRange::new(1, 2));
        assert_eq!(s.file, "f.md");
        assert_eq!(s.lines, LineRange::new(1, 2));
    }

    #[test]
    fn parameter_def_new() {
        let p = ParameterDef::new("ability", ParamType::Ref, ParameterDomain::Ability);
        assert_eq!(p.key, "ability");
        assert_eq!(p.param_type, ParamType::Ref);
        assert_eq!(p.domain, ParameterDomain::Ability);
    }

    #[test]
    fn line_range_validity() {
        assert!(LineRange::new(1, 5).is_valid());
        assert!(LineRange::new(5, 5).is_valid());
        assert!(!LineRange::new(6, 5).is_valid());
    }

    #[test]
    fn prereq_art_min_roundtrip() {
        let json = r#"{"kind": "art_min", "value": {"art": "art.creo", "score": 5}}"#;
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
