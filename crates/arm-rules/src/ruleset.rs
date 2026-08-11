//! The ruleset container and its JSON loaders.
//!
//! [`Ruleset`] holds the language-neutral mechanics (point items, abilities,
//! characteristic rules, type profiles, advancement table).
//! [`Ruleset::from_sources`] is the canonical loader; the other constructors
//! are convenience wrappers over it. Loading validates referential integrity
//! and fails loudly on unresolved refs, asymmetric incompatibilities, or
//! malformed data ([`RulesetError`]). [`LocalizedRuleset`] joins a ruleset
//! with an i18n layer for display.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::ability::{Ability, AbilityCategory, AdvancementTable, AgeAbilityCaps};
use crate::art::{Art, ArtType, ArtsFile};
use crate::characteristics::CharacteristicRules;
use crate::childhood::ChildhoodPackage;
use crate::equipment::{Armor, EquipmentFile, Shield, Weapon};
use crate::grant::Grant;
use crate::house::{House, HousesFile};
use crate::life_stage::LifeStageRules;
use crate::mythic_companion::{MythicCompanionType, MythicCompanionTypesFile};
use crate::spell::{Spell, SpellDuration, SpellTarget, SpellsFile};
use crate::spell_mastery::{SpellMasteryAbilitiesFile, SpellMasteryAbility};
use crate::types::{
    CreationPhase, Effect, EntityTypeProfile, I18nEntry, Id, ItemKind, Magnitude, ParameterDomain,
    PointItem, Prereq, RulesetRef, SpecialCasting,
};

/// Top-level container for all loaded game mechanics.
///
/// Built via the validating loaders ([`Ruleset::from_sources`] is the
/// canonical one; `from_json`, `from_core_json`, and `from_json_with_abilities`
/// are convenience wrappers, and `from_serialized` reloads trusted cached
/// JSON), never field-by-field by callers; `PartialEq` is provided for tests
/// and diffing.
///
/// # JSON shape
///
/// Serialized whole as a Tauri command return value, so the frontend binds
/// directly to these top-level field names — they are a **stable public
/// contract**; renaming any silently breaks the TS consumer with no Rust error.
/// The data maps (`point_items`, `type_profiles`, `abilities`, `arts`,
/// `houses`, `characteristic_rules`) serialize as JSON objects keyed by id;
/// `advancement` / `art_advancement` / `age_ability_caps` are bare arrays (see
/// [`AdvancementTable`]):
///
/// ```json
/// {
///   "id": "arm5-core",
///   "version": "2024.1",
///   "point_items": { "virtue.x": { /* PointItem */ } },
///   "type_profiles": { "magus": { /* EntityTypeProfile */ } },
///   "abilities": { "ability.awareness": { /* Ability */ } },
///   "advancement": [ { "score": 1, "total_xp": 5 } ],
///   "age_ability_caps": [ { "max_age": 29, "max_score": 5 } ],
///   "characteristic_rules": { /* CharacteristicRules */ },
///   "mythic_companion_types": { "mythic.x": { /* MythicCompanionType */ } },
///   "magnitude_points": { "free": 0, "minor": 1, "major": 3 },
///   "ability_category_order": [ "general", "academic", "arcane", "martial", "supernatural" ],
///   "arts": { "art.creo": { /* Art */ } },
///   "art_advancement": [ { "score": 1, "total_xp": 1 } ],
///   "art_type_order": [ "technique", "form" ],
///   "houses": { "house.bonisagus": { /* House */ } },
///   "childhoods": { "childhood.athletic": { /* ChildhoodPackage */ } },
///   "spells": { "spell.pilum_of_fire": { /* Spell */ } },
///   "spell_mastery_abilities": { "spell_mastery_ability.penetration": { /* SpellMasteryAbility */ } },
///   "weapons": { "weapon.long_sword": { /* Weapon */ } },
///   "shields": { "shield.heater": { /* Shield */ } },
///   "armor": { "armor.chain_mail_full": { /* Armor */ } }
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
    /// The age → maximum-Ability-score band table (Core:2366-2374). Caps *Ability*
    /// scores by age, loaded from `rules/core/abilities.json` beside the Ability
    /// advancement table. Empty for a ruleset that ships no age caps. Serialized
    /// whole to the frontend; the `age_ability_caps` field name is a stable public
    /// contract.
    #[serde(default)]
    pub(crate) age_ability_caps: AgeAbilityCaps,
    /// Ability categories a character may only buy with a permitting Virtue
    /// (Core:2315), loaded from `rules/core/abilities.json` beside the age caps.
    /// Empty for a ruleset that gates none, which stands the rule down rather than
    /// letting the engine invent the list. Serialized whole to the frontend; the
    /// `categories_requiring_virtue` field name is a stable public contract.
    #[serde(default)]
    pub(crate) categories_requiring_virtue: BTreeSet<AbilityCategory>,
    /// The scholarly-language expectation Academic Abilities normally carry
    /// (Core:7151), loaded beside the categories above. `None` for a ruleset that
    /// states none. Serialized whole to the frontend; the `scholarly_language` field
    /// name is a stable public contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) scholarly_language: Option<ScholarlyLanguageRequirement>,
    /// The Characteristic point-buy rules (cost table + starting points), if the
    /// ruleset ships them. `None` for rulesets without a characteristics file.
    /// Serialized whole to the frontend; the `characteristic_rules` field name is
    /// a stable public contract.
    #[serde(default)]
    pub(crate) characteristic_rules: Option<CharacteristicRules>,
    /// The life-stage experience rules (childhood + later life), if the ruleset
    /// ships them. `None` for a ruleset without a life-stages file, which leaves
    /// [`crate::types::Entity::xp_pool`] the only source of experience. Serialized
    /// whole to the frontend; the `life_stages` field name is a stable public
    /// contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) life_stages: Option<LifeStageRules>,
    /// Sample Childhood packages keyed by their id — ready-made Ability spreads a
    /// player may take instead of dividing the childhood experience by hand.
    /// Defaulted so older serialized rulesets (no packages) still deserialize, and
    /// always serialized like `houses`, so the frontend's record is empty rather
    /// than absent for a ruleset shipping none. The `childhoods` field name is a
    /// stable public contract.
    #[serde(default)]
    pub(crate) childhoods: BTreeMap<Id, ChildhoodPackage>,
    /// Magnitude→point-weight table, derived from [`Magnitude::points`]. Serialized
    /// to the frontend so the UI reads point values from the engine instead of
    /// re-hardcoding them. Derived data, not authored: populated at construction
    /// and re-derived in [`Ruleset::from_serialized`], never trusted from input
    /// JSON. The `magnitude_points` field name is a stable public contract.
    #[serde(default)]
    pub(crate) magnitude_points: BTreeMap<Magnitude, u8>,
    /// Ability categories in canonical book order, derived from
    /// [`AbilityCategory::ALL`]. Serialized to the frontend so the UI orders
    /// ability groups from engine data instead of re-hardcoding the order. Derived
    /// data, not authored (see `magnitude_points`). The `ability_category_order`
    /// field name is a stable public contract.
    #[serde(default)]
    pub(crate) ability_category_order: Vec<AbilityCategory>,
    /// All Hermetic Arts keyed by their id. Defaulted so older serialized
    /// rulesets (no arts) still deserialize. Serialized whole to the frontend;
    /// the `arts` field name is a stable public contract.
    #[serde(default)]
    pub(crate) arts: BTreeMap<Id, Art>,
    /// The Art XP advancement table (the cheaper triangular "ART To Buy" curve).
    /// Serialized whole to the frontend; the `art_advancement` field name is a
    /// stable public contract.
    #[serde(default)]
    pub(crate) art_advancement: AdvancementTable,
    /// Art classes (Technique, Form) in canonical book order, derived from
    /// [`ArtType::ALL`]. Serialized to the frontend so the UI orders Art groups
    /// from engine data instead of re-hardcoding the order. Derived data, not
    /// authored (see `magnitude_points`). The `art_type_order` field name is a
    /// stable public contract.
    #[serde(default)]
    pub(crate) art_type_order: Vec<ArtType>,
    /// All Hermetic Houses keyed by their id. Defaulted so older serialized
    /// rulesets (no houses) still deserialize. Serialized whole to the frontend;
    /// the `houses` field name is a stable public contract.
    #[serde(default)]
    pub(crate) houses: BTreeMap<Id, House>,
    /// All Mythic Companion types keyed by their id. Defaulted so older
    /// serialized rulesets (no mythic types) still deserialize. Serialized whole
    /// to the frontend; the `mythic_companion_types` field name is a stable
    /// public contract.
    #[serde(default)]
    pub(crate) mythic_companion_types: BTreeMap<Id, MythicCompanionType>,
    /// All Hermetic spells keyed by their id. Defaulted so older serialized
    /// rulesets (no spells) still deserialize. Serialized whole to the frontend;
    /// the `spells` field name is a stable public contract.
    #[serde(default)]
    pub(crate) spells: BTreeMap<Id, Spell>,
    /// All Spell Mastery special abilities keyed by their id. Defaulted so older
    /// serialized rulesets (no mastery catalogue) still deserialize. Serialized
    /// whole to the frontend; the `spell_mastery_abilities` field name is a stable
    /// public contract.
    #[serde(default)]
    pub(crate) spell_mastery_abilities: BTreeMap<Id, SpellMasteryAbility>,
    /// All weapons keyed by their id. Defaulted so older serialized rulesets (no
    /// equipment) still deserialize. Serialized whole to the frontend; the
    /// `weapons` field name is a stable public contract.
    #[serde(default)]
    pub(crate) weapons: BTreeMap<Id, Weapon>,
    /// All shields keyed by their id. Defaulted like `weapons`. Serialized whole
    /// to the frontend; the `shields` field name is a stable public contract.
    #[serde(default)]
    pub(crate) shields: BTreeMap<Id, Shield>,
    /// All armor keyed by their id. Defaulted like `weapons`. Serialized whole to
    /// the frontend; the `armor` field name is a stable public contract.
    #[serde(default)]
    pub(crate) armor: BTreeMap<Id, Armor>,
}

/// The magnitude→points table, derived from the canonical [`Magnitude::points`].
fn derived_magnitude_points() -> BTreeMap<Magnitude, u8> {
    Magnitude::ALL
        .into_iter()
        .map(|m| (m, m.points()))
        .collect()
}

/// The set of language-neutral JSON source strings a [`Ruleset`] is built from.
///
/// A named struct rather than a growing list of positional `&str` arguments: the
/// optional inputs (`abilities`, `characteristics`) are honest `Option`s instead
/// of sentinel `""`/`"{}"` strings, and each field is named at the call site.
/// Use [`Ruleset::from_sources`].
///
/// `Default` yields the empty ruleset (`id`/`version`/JSON fields `""`, every
/// optional source `None`); tests build a partial ruleset by setting only the
/// fields they exercise and filling the rest with `..RulesetSources::default()`.
#[derive(Debug, Clone, Copy, Default)]
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
    /// Arts-file JSON (`{ "advancement": [...], "arts": [...] }`), or `None` for
    /// a ruleset without an Art registry.
    pub arts: Option<&'a str>,
    /// Houses-file JSON (`{ "houses": [...] }`), or `None` for a ruleset without
    /// a House registry.
    pub houses: Option<&'a str>,
    /// Mythic-companion-types JSON (`{ "types": [...] }`), or `None` for a
    /// ruleset without a Mythic Companion type registry.
    pub mythic_types: Option<&'a str>,
    /// Spells-file JSON (`{ "spells": [...] }`), or `None` for a ruleset without
    /// a spell catalogue.
    pub spells: Option<&'a str>,
    /// Spell-mastery-abilities-file JSON (`{ "abilities": [...] }`), or `None` for
    /// a ruleset without a Spell Mastery special-ability catalogue.
    pub spell_mastery_abilities: Option<&'a str>,
    /// Equipment-file JSON (`{ "weapons": [...], "shields": [...], "armor": [...] }`),
    /// or `None` for a ruleset without an equipment catalogue.
    pub equipment: Option<&'a str>,
    /// Characteristic point-buy JSON (`{ "start_points", "costs" }`), or `None`
    /// for a ruleset that ships no characteristic rules.
    pub characteristics: Option<&'a str>,
    /// Life-stage experience JSON (`{ "childhood", "later_life" }`), or `None` for
    /// a ruleset that ships no life stages (which leaves `Entity::xp_pool` the only
    /// source of experience, as before).
    pub life_stages: Option<&'a str>,
    /// Sample-Childhood-packages JSON (`{ "packages": [...] }`), or `None` for a
    /// ruleset that ships no packages — which offers the player no shortcut but
    /// leaves the childhood block itself perfectly usable by hand.
    pub childhoods: Option<&'a str>,
}

/// A [`Ruleset`] paired with localized display text for a single language.
///
/// Built via [`LocalizedRuleset::new`] (single i18n source) or
/// [`LocalizedRuleset::from_merged`] (merging several); `PartialEq` is provided
/// for tests.
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
    pub const ARTS: &str = "arts";
    pub const HOUSES: &str = "houses";
    pub const MYTHIC_TYPES: &str = "mythic companion types";
    pub const SPELLS: &str = "spells";
    pub const SPELL_MASTERY_ABILITIES: &str = "spell mastery abilities";
    pub const EQUIPMENT: &str = "equipment";
    pub const CHARACTERISTICS: &str = "characteristics";
    pub const LIFE_STAGES: &str = "life stages";
    pub const CHILDHOODS: &str = "childhoods";
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
    /// `RulesetError::parse` at call sites that know which input failed.
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
/// Internal deserialize-only wrapper, like the sibling arts/houses file shapes:
/// never part of the crate's public API (consumers see the assembled
/// [`Ruleset`], not raw file shapes). Kept module-private here because — unlike
/// `ArtsFile`/`HousesFile`, which live in their catalogue modules and are read
/// across the module boundary — it is parsed only within this module.
#[derive(Deserialize)]
struct AbilitiesFile {
    #[serde(default)]
    advancement: AdvancementTable,
    #[serde(default)]
    abilities: Vec<Ability>,
    /// The age → maximum-Ability-score band table (Core:2366-2374). Caps *Ability*
    /// scores by age, so it lives beside the Ability advancement table.
    #[serde(default)]
    age_ability_caps: AgeAbilityCaps,
    /// Ability categories a character may only buy with a permitting Virtue
    /// (Core:2315). Data, not a hardcoded list, so a ruleset that gates a different
    /// set says so in its own file; empty means the rule is not enforced.
    #[serde(default)]
    categories_requiring_virtue: BTreeSet<AbilityCategory>,
    /// The scholarly-language expectation for Academic Abilities (Core:7151), or
    /// absent for a ruleset that states none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scholarly_language: Option<ScholarlyLanguageRequirement>,
}

/// The scholarly language an Academic Ability normally expects, and at what score.
///
/// > learning an Academic Knowledge normally requires a Latin, Greek, Hebrew, or
/// > Arabic score of at least 3, depending on the region of Europe you are from.
///
/// Data rather than four hardcoded ids: which language qualifies is regional, so the
/// ruleset names the *ability* (the parameterized dead language) and the minimum
/// score, and any instance of it satisfies the expectation. Source: Ars Magica -
/// Definitive Edition (Core Rules).md:7151.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScholarlyLanguageRequirement {
    /// The ability a scholarly language is an instance of.
    pub ability: Id,
    /// The score it is normally expected to reach.
    pub min_score: u8,
}

/// On-disk shape of `rules/core/childhoods.json`: the Sample Childhood package
/// catalogue. Defaults to empty so `"{}"` is a valid empty file. Internal
/// deserialize-only wrapper, module-private for the same reason as
/// [`AbilitiesFile`]: it is parsed only within this module.
#[derive(Deserialize)]
struct ChildhoodsFile {
    #[serde(default)]
    packages: Vec<ChildhoodPackage>,
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

/// Abilities the engine dereferences by hardcoded id when computing a magus's
/// play-time totals (`derived.rs`: Casting/Lab Totals, Magic Resistance,
/// Penetration; `validation.rs`: the spell-level cap). These are **engine
/// invariants, not catalogue data**: the code looks them up by these exact
/// slugs, so a Hermetic ruleset that renames or omits one would silently compute
/// wrong numbers (e.g. Magic Resistance as if Parma Magica = 0) rather than fail.
/// [`Ruleset::validate_integrity`] enforces their presence for any ruleset that
/// declares a magus profile and ships an Arts catalogue. Kept sorted for a
/// deterministic error order.
/// Artes Liberales — ritual-casting bonus (`derived.rs`).
pub(crate) const ID_ARTES_LIBERALES: &str = "ability.artes_liberales";
/// Magic Theory — Lab Total addend (`derived.rs`) and the spell-level cap
/// (`validation/magus.rs`).
pub(crate) const ID_MAGIC_THEORY: &str = "ability.magic_theory";
/// Parma Magica — Magic Resistance base (`derived.rs`).
pub(crate) const ID_PARMA_MAGICA: &str = "ability.parma_magica";
/// Penetration — Penetration Total addend (`derived.rs`).
pub(crate) const ID_PENETRATION: &str = "ability.penetration";
/// Philosophiae — ritual-casting bonus (`derived.rs`).
pub(crate) const ID_PHILOSOPHIAE: &str = "ability.philosophiae";
const ENGINE_REQUIRED_ABILITIES: [&str; 5] = [
    ID_ARTES_LIBERALES,
    ID_MAGIC_THEORY,
    ID_PARMA_MAGICA,
    ID_PENETRATION,
    ID_PHILOSOPHIAE,
];

/// Arts the engine dereferences by hardcoded id (`derived.rs` reads Creo+Corpus
/// for the self-made-Longevity-Ritual Lab Total). Engine invariants like
/// [`ENGINE_REQUIRED_ABILITIES`]; enforced under the same condition.
/// Corpus — self-made-Longevity-Ritual Lab Total (`derived.rs`).
pub(crate) const ID_CORPUS: &str = "art.corpus";
/// Creo — self-made-Longevity-Ritual Lab Total (`derived.rs`) and the
/// Momentary-Creo lasting-effect ritual check (`validate_spell_refs`).
pub(crate) const ID_CREO: &str = "art.creo";
const ENGINE_REQUIRED_ARTS: [&str; 2] = [ID_CORPUS, ID_CREO];

/// V/F category slug the engine dereferences by hardcoded string:
/// `validation/scores.rs` keys the Major-Personality-Flaw rule (each Major
/// Personality Flaw permits one personality trait with `|value|` up to ±6;
/// Core:2500-2503, :2820) off this exact category. Engine invariant like
/// [`ENGINE_REQUIRED_ABILITIES`] — a ruleset that renamed or dropped it would make
/// the engine silently count zero Major Personality Flaws and wrongly reject every
/// ±6 trait, so [`Ruleset::validate_integrity`] enforces its presence for any
/// ruleset that ships a V/F catalogue.
pub(crate) const ENGINE_REQUIRED_CATEGORY_PERSONALITY: &str = "personality";

impl Ruleset {
    /// Parses point items and type profiles from JSON, validates referential
    /// integrity, and returns a Ruleset with no abilities.
    ///
    /// Convenience constructor over [`Ruleset::from_sources`] for the many tests
    /// that do not exercise the ability registry.
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
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
    }

    /// Like [`Ruleset::from_json`] but also loads the abilities file (catalogue +
    /// advancement table). No arts or characteristic rules.
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
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
    }

    /// Parses every language-neutral core source — point items, type profiles,
    /// abilities (catalogue + advancement), and the Characteristic point-buy rules
    /// — validates referential integrity, and returns a Ruleset.
    ///
    /// `characteristics_json` is `Some` of an object `{ "start_points", "costs" }`,
    /// or `None` for a ruleset that ships no characteristic rules.
    pub fn from_core_json(
        id: &str,
        version: &str,
        point_items_json: &str,
        type_profiles_json: &str,
        abilities_json: &str,
        characteristics_json: Option<&str>,
    ) -> Result<Self, RulesetError> {
        Self::from_sources(RulesetSources {
            id,
            version,
            point_items: point_items_json,
            type_profiles: type_profiles_json,
            abilities: Some(abilities_json),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: characteristics_json,
            life_stages: None,
            childhoods: None,
        })
    }

    /// Parses every language-neutral core source named in `sources`, validates
    /// referential integrity, and returns a Ruleset. This is the canonical
    /// loader (used in production by `arm-app`); [`Ruleset::from_json`],
    /// [`Ruleset::from_json_with_abilities`], and [`Ruleset::from_core_json`] are
    /// thin convenience constructors over it for tests that need only a subset of
    /// the sources.
    pub fn from_sources(sources: RulesetSources) -> Result<Self, RulesetError> {
        let RulesetSources {
            id,
            version,
            point_items: point_items_json,
            type_profiles: type_profiles_json,
            abilities,
            arts,
            houses,
            mythic_types,
            spells,
            spell_mastery_abilities,
            equipment,
            characteristics,
            life_stages,
            childhoods,
        } = sources;

        let items: Vec<PointItem> = serde_json::from_str(point_items_json)
            .map_err(|e| RulesetError::parse(parse_source::POINT_ITEMS, e))?;
        let types: Vec<EntityTypeProfile> = serde_json::from_str(type_profiles_json)
            .map_err(|e| RulesetError::parse(parse_source::TYPE_PROFILES, e))?;
        // An absent abilities file is equivalent to an empty `"{}"`.
        let abilities_file: AbilitiesFile = serde_json::from_str(abilities.unwrap_or("{}"))
            .map_err(|e| RulesetError::parse(parse_source::ABILITIES, e))?;
        // An absent arts file is equivalent to an empty `"{}"`.
        let arts_file: ArtsFile = serde_json::from_str(arts.unwrap_or("{}"))
            .map_err(|e| RulesetError::parse(parse_source::ARTS, e))?;
        // An absent houses file is equivalent to an empty `"{}"`.
        let houses_file: HousesFile = serde_json::from_str(houses.unwrap_or("{}"))
            .map_err(|e| RulesetError::parse(parse_source::HOUSES, e))?;
        // An absent mythic-types file is equivalent to an empty `"{}"`.
        let mythic_types_file: MythicCompanionTypesFile =
            serde_json::from_str(mythic_types.unwrap_or("{}"))
                .map_err(|e| RulesetError::parse(parse_source::MYTHIC_TYPES, e))?;
        // An absent spells file is equivalent to an empty `"{}"`.
        let spells_file: SpellsFile = serde_json::from_str(spells.unwrap_or("{}"))
            .map_err(|e| RulesetError::parse(parse_source::SPELLS, e))?;
        // An absent spell-mastery-abilities file is equivalent to an empty `"{}"`.
        let spell_mastery_abilities_file: SpellMasteryAbilitiesFile =
            serde_json::from_str(spell_mastery_abilities.unwrap_or("{}"))
                .map_err(|e| RulesetError::parse(parse_source::SPELL_MASTERY_ABILITIES, e))?;
        // An absent equipment file is equivalent to an empty `"{}"`.
        let equipment_file: EquipmentFile = serde_json::from_str(equipment.unwrap_or("{}"))
            .map_err(|e| RulesetError::parse(parse_source::EQUIPMENT, e))?;
        let characteristic_rules: Option<CharacteristicRules> = match characteristics {
            None => None,
            Some(json) => Some(
                serde_json::from_str(json)
                    .map_err(|e| RulesetError::parse(parse_source::CHARACTERISTICS, e))?,
            ),
        };
        let life_stage_rules: Option<LifeStageRules> = match life_stages {
            None => None,
            Some(json) => Some(
                serde_json::from_str(json)
                    .map_err(|e| RulesetError::parse(parse_source::LIFE_STAGES, e))?,
            ),
        };
        // An absent childhoods file is equivalent to an empty `"{}"`.
        let childhoods_file: ChildhoodsFile = serde_json::from_str(childhoods.unwrap_or("{}"))
            .map_err(|e| RulesetError::parse(parse_source::CHILDHOODS, e))?;

        // Detect duplicate IDs across each registry.
        let mut errors = Vec::new();
        collect_duplicates(items.iter().map(|i| &i.id), "point item", &mut errors);
        collect_duplicates(types.iter().map(|t| &t.id), "type profile", &mut errors);
        collect_duplicates(
            abilities_file.abilities.iter().map(|a| &a.id),
            "ability",
            &mut errors,
        );
        collect_duplicates(arts_file.arts.iter().map(|a| &a.id), "art", &mut errors);
        collect_duplicates(
            houses_file.houses.iter().map(|h| &h.id),
            "house",
            &mut errors,
        );
        collect_duplicates(
            mythic_types_file.types.iter().map(|t| &t.id),
            "mythic companion type",
            &mut errors,
        );
        collect_duplicates(
            spells_file.spells.iter().map(|s| &s.id),
            "spell",
            &mut errors,
        );
        collect_duplicates(
            spell_mastery_abilities_file.abilities.iter().map(|a| &a.id),
            "spell mastery ability",
            &mut errors,
        );
        collect_duplicates(
            equipment_file.weapons.iter().map(|w| &w.id),
            "weapon",
            &mut errors,
        );
        collect_duplicates(
            equipment_file.shields.iter().map(|s| &s.id),
            "shield",
            &mut errors,
        );
        collect_duplicates(
            equipment_file.armor.iter().map(|a| &a.id),
            "armor",
            &mut errors,
        );
        collect_duplicates(
            childhoods_file.packages.iter().map(|p| &p.id),
            "childhood package",
            &mut errors,
        );
        // The scholarly-language expectation names an ability, which must resolve and
        // be parameterized (a scholarly language is one instance of a dead language).
        if let Some(requirement) = &abilities_file.scholarly_language {
            match abilities_file
                .abilities
                .iter()
                .find(|a| a.id == requirement.ability)
            {
                None => errors.push(format!(
                    "scholarly-language requirement names unknown ability '{}'",
                    requirement.ability
                )),
                Some(ability) if ability.parameter.is_none() => errors.push(format!(
                    "scholarly-language ability '{}' takes no parameter, so it cannot name one language",
                    requirement.ability
                )),
                Some(_) => {}
            }
        }

        // The childhood block's own ability refs are checked in
        // `validate_childhood_refs`, called from `validate_integrity` — so a
        // cached ruleset arriving through `from_serialized` is held to the same
        // standard as a freshly parsed one.

        // A declared creation flow must be walkable: no phase twice (the second
        // visit's Back would land where the user just was), and never `review`,
        // which the wizard appends itself as the terminal catch-all step. An empty
        // list is legal and simply means the type has no guided flow.
        for profile in &types {
            let mut seen = BTreeSet::new();
            for phase in &profile.creation_phases {
                if *phase == CreationPhase::Review {
                    errors.push(format!(
                        "type profile '{}' declares the synthetic '{phase}' phase, which the wizard appends itself",
                        profile.id
                    ));
                } else if !seen.insert(phase) {
                    errors.push(format!(
                        "type profile '{}' repeats creation phase '{phase}'",
                        profile.id
                    ));
                }
            }
        }
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
        let arts: BTreeMap<Id, Art> = arts_file
            .arts
            .into_iter()
            .map(|a| (a.id.clone(), a))
            .collect();
        let houses: BTreeMap<Id, House> = houses_file
            .houses
            .into_iter()
            .map(|h| (h.id.clone(), h))
            .collect();
        let mythic_companion_types: BTreeMap<Id, MythicCompanionType> = mythic_types_file
            .types
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect();
        let spells: BTreeMap<Id, Spell> = spells_file
            .spells
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();
        let spell_mastery_abilities: BTreeMap<Id, SpellMasteryAbility> =
            spell_mastery_abilities_file
                .abilities
                .into_iter()
                .map(|a| (a.id.clone(), a))
                .collect();
        let weapons: BTreeMap<Id, Weapon> = equipment_file
            .weapons
            .into_iter()
            .map(|w| (w.id.clone(), w))
            .collect();
        let shields: BTreeMap<Id, Shield> = equipment_file
            .shields
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();
        let armor: BTreeMap<Id, Armor> = equipment_file
            .armor
            .into_iter()
            .map(|a| (a.id.clone(), a))
            .collect();
        let childhoods: BTreeMap<Id, ChildhoodPackage> = childhoods_file
            .packages
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();

        let ruleset = Self {
            id: Id::new(id),
            version: version.to_string(),
            point_items,
            type_profiles,
            abilities,
            advancement: abilities_file.advancement,
            age_ability_caps: abilities_file.age_ability_caps,
            categories_requiring_virtue: abilities_file.categories_requiring_virtue,
            scholarly_language: abilities_file.scholarly_language,
            characteristic_rules,
            life_stages: life_stage_rules,
            childhoods,
            magnitude_points: derived_magnitude_points(),
            ability_category_order: AbilityCategory::ALL.to_vec(),
            arts,
            art_advancement: arts_file.advancement,
            art_type_order: ArtType::ALL.to_vec(),
            houses,
            mythic_companion_types,
            spells,
            spell_mastery_abilities,
            weapons,
            shields,
            armor,
        };

        ruleset.validate_integrity()?;
        Ok(ruleset)
    }

    /// Deserializes a previously-serialized [`Ruleset`] and RE-RUNS referential
    /// integrity validation. Use this for trusted/cached data; untrusted JSON must
    /// still go through [`Ruleset::from_json`]. Deriving `Deserialize` alone does
    /// NOT validate integrity — always reconstruct via this method.
    pub fn from_serialized(json: &str) -> Result<Self, RulesetError> {
        let mut ruleset: Ruleset = serde_json::from_str(json)?;
        // These are derived constants, not authored data: re-derive them rather
        // than trusting the incoming JSON, so an older payload missing the fields
        // still yields a correct, non-empty ruleset.
        ruleset.magnitude_points = derived_magnitude_points();
        ruleset.ability_category_order = AbilityCategory::ALL.to_vec();
        ruleset.art_type_order = ArtType::ALL.to_vec();
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
    ///
    /// The `houses` and `mythic_companion_types` catalogues are intentionally
    /// exempt: a grant's list (and each `Grant::Choice`'s `options`), and a
    /// mythic type's required package, are order-significant authored data — like
    /// `creation_phases` — sourced from the already-canonical, pipeline-generated
    /// `rules/core/*.json`, and the assembled `Ruleset` is only ever a transient
    /// frontend payload, never written back to disk. So there is no
    /// canonical-write to normalize for.
    pub fn normalize(&mut self) {
        for item in self.point_items.values_mut() {
            item.normalize();
        }
        for profile in self.type_profiles.values_mut() {
            profile.normalize();
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

    /// The age → maximum-Ability-score band table (Core:2366-2374). Empty when the
    /// ruleset ships no age caps.
    pub fn age_ability_caps(&self) -> &AgeAbilityCaps {
        &self.age_ability_caps
    }

    /// Looks up an Art by id.
    pub fn art(&self, id: &Id) -> Option<&Art> {
        self.arts.get(id)
    }

    /// Iterates all Arts in id order.
    pub fn arts(&self) -> impl Iterator<Item = &Art> {
        self.arts.values()
    }

    /// The ids of every Art of one class (Technique or Form), **sorted**.
    ///
    /// The sort is part of the contract, not an accident of storage: callers pair
    /// Techniques against Forms to build grids that are compared and serialized by
    /// position (`spell_level_caps`, `lab_totals`, `casting_totals`), so the order
    /// must not depend on how a ruleset's `arts.json` happened to list them. It
    /// costs nothing today — the catalogue is a `BTreeMap`, so iteration is already
    /// id-ordered — and it keeps that guarantee if the storage ever changes.
    pub fn art_ids_of(&self, art_type: ArtType) -> Vec<Id> {
        let mut ids: Vec<Id> = self
            .arts()
            .filter(|a| a.art_type == art_type)
            .map(|a| a.id.clone())
            .collect();
        ids.sort();
        ids
    }

    /// Looks up a House by id.
    pub fn house(&self, id: &Id) -> Option<&House> {
        self.houses.get(id)
    }

    /// Iterates all Houses in id order.
    pub fn houses(&self) -> impl Iterator<Item = &House> {
        self.houses.values()
    }

    /// Number of Houses in the catalogue.
    pub fn house_count(&self) -> usize {
        self.houses.len()
    }

    /// Looks up a Mythic Companion type by id.
    pub fn mythic_type(&self, id: &Id) -> Option<&MythicCompanionType> {
        self.mythic_companion_types.get(id)
    }

    /// Iterates all Mythic Companion types in id order.
    pub fn mythic_types(&self) -> impl Iterator<Item = &MythicCompanionType> {
        self.mythic_companion_types.values()
    }

    /// Number of Mythic Companion types in the catalogue.
    pub fn mythic_type_count(&self) -> usize {
        self.mythic_companion_types.len()
    }

    /// Looks up a spell by id.
    pub fn spell(&self, id: &Id) -> Option<&Spell> {
        self.spells.get(id)
    }

    /// Iterates all spells in id order.
    pub fn spells(&self) -> impl Iterator<Item = &Spell> {
        self.spells.values()
    }

    /// Number of spells in the catalogue.
    pub fn spell_count(&self) -> usize {
        self.spells.len()
    }

    /// Looks up a Spell Mastery special ability by id.
    pub fn spell_mastery_ability(&self, id: &Id) -> Option<&SpellMasteryAbility> {
        self.spell_mastery_abilities.get(id)
    }

    /// Iterates all Spell Mastery special abilities in id order.
    pub fn spell_mastery_abilities(&self) -> impl Iterator<Item = &SpellMasteryAbility> {
        self.spell_mastery_abilities.values()
    }

    /// Number of Spell Mastery special abilities in the catalogue.
    pub fn spell_mastery_ability_count(&self) -> usize {
        self.spell_mastery_abilities.len()
    }

    /// Looks up a weapon by id.
    pub fn weapon(&self, id: &Id) -> Option<&Weapon> {
        self.weapons.get(id)
    }

    /// Iterates all weapons in id order.
    pub fn weapons(&self) -> impl Iterator<Item = &Weapon> {
        self.weapons.values()
    }

    /// Number of weapons in the catalogue.
    pub fn weapon_count(&self) -> usize {
        self.weapons.len()
    }

    /// Looks up a shield by id.
    pub fn shield(&self, id: &Id) -> Option<&Shield> {
        self.shields.get(id)
    }

    /// Iterates all shields in id order.
    pub fn shields(&self) -> impl Iterator<Item = &Shield> {
        self.shields.values()
    }

    /// Number of shields in the catalogue.
    pub fn shield_count(&self) -> usize {
        self.shields.len()
    }

    /// Looks up an armor entry by id.
    ///
    /// Named `armor_item` rather than the sibling singular `armor` because the
    /// plural iterator already claims `armor()` — "armor" is an uncountable noun,
    /// so it has no distinct plural to hand the iterator. This is the one
    /// deliberate exception to the singular-noun lookup convention used by every
    /// other catalogue accessor (`item`, `ability`, `weapon`, `shield`, …).
    pub fn armor_item(&self, id: &Id) -> Option<&Armor> {
        self.armor.get(id)
    }

    /// Iterates all armor entries in id order.
    pub fn armor(&self) -> impl Iterator<Item = &Armor> {
        self.armor.values()
    }

    /// Number of armor entries in the catalogue.
    pub fn armor_count(&self) -> usize {
        self.armor.len()
    }

    /// Number of Arts in the catalogue.
    pub fn art_count(&self) -> usize {
        self.arts.len()
    }

    /// The Art XP advancement table.
    pub fn art_advancement(&self) -> &AdvancementTable {
        &self.art_advancement
    }

    /// The Characteristic point-buy rules, if the ruleset ships them.
    pub fn characteristic_rules(&self) -> Option<&CharacteristicRules> {
        self.characteristic_rules.as_ref()
    }

    /// The life-stage experience rules, if the ruleset ships them.
    pub fn life_stages(&self) -> Option<&LifeStageRules> {
        self.life_stages.as_ref()
    }

    /// Looks up a Sample Childhood package by id.
    pub fn childhood(&self, id: &Id) -> Option<&ChildhoodPackage> {
        self.childhoods.get(id)
    }

    /// Iterates all Sample Childhood packages in id order.
    pub fn childhoods(&self) -> impl Iterator<Item = &ChildhoodPackage> {
        self.childhoods.values()
    }

    /// Ability categories that may only be bought with a permitting Virtue
    /// (Core:2315). Empty for a ruleset that gates none.
    pub fn categories_requiring_virtue(&self) -> &BTreeSet<AbilityCategory> {
        &self.categories_requiring_virtue
    }

    /// The scholarly-language expectation for Academic Abilities (Core:7151), if the
    /// ruleset states one.
    pub fn scholarly_language_requirement(&self) -> Option<&ScholarlyLanguageRequirement> {
        self.scholarly_language.as_ref()
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
            self.validate_effect_refs(item, id, &mut errors);

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
        self.validate_magnitude_variant_exclusivity(&mut errors);

        // The advancement table must have unique scores and non-decreasing
        // total_xp, or xp_to_raise's step subtraction would underflow later.
        errors.extend(self.advancement.validation_errors());
        // Same invariant for the Art advancement table.
        errors.extend(self.art_advancement.validation_errors());

        self.validate_childhood_refs(&mut errors);
        self.validate_childhood_packages(&mut errors);

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
            // Intentionally unchecked: the profile's category-typed fields
            // (`permitted_categories`, `forbidden_categories`, `gift_categories`,
            // and the budget's `flaw_category_caps`) are NOT validated against the
            // set of categories carried by point items. Categories are an open,
            // forward-declared namespace: the `point_items` catalogue is extracted
            // incrementally from the rules source, so a profile legitimately names
            // a category (e.g. `personality`, `story`, `supernatural`) before any
            // item in that category has been extracted yet — the shipped
            // `rules/core` data does exactly this. Requiring a backing item would
            // reject valid data, so this referential check is deliberately omitted
            // (tracked here rather than left silent).
        }

        for house in self.houses.values() {
            self.validate_house_refs(house, &mut errors);
        }

        for mtype in self.mythic_companion_types.values() {
            self.validate_mythic_type_refs(mtype, &mut errors);
        }

        for spell in self.spells.values() {
            self.validate_spell_refs(spell, &mut errors);
        }

        for ability in self.spell_mastery_abilities.values() {
            if let Some(ref source) = ability.source
                && !source.lines.is_valid()
            {
                errors.push(format!(
                    "spell mastery ability '{}': source line range start ({}) exceeds end ({})",
                    ability.id, source.lines.start, source.lines.end
                ));
            }
        }

        for weapon in self.weapons.values() {
            self.validate_weapon_refs(weapon, &mut errors);
        }
        for shield in self.shields.values() {
            if let Some(ref source) = shield.source
                && !source.lines.is_valid()
            {
                errors.push(format!(
                    "shield '{}': source line range start ({}) exceeds end ({})",
                    shield.id, source.lines.start, source.lines.end
                ));
            }
        }
        for armor in self.armor.values() {
            if let Some(ref source) = armor.source
                && !source.lines.is_valid()
            {
                errors.push(format!(
                    "armor '{}': source line range start ({}) exceeds end ({})",
                    armor.id, source.lines.start, source.lines.end
                ));
            }
        }

        self.validate_engine_required_roles(&mut errors);
        self.validate_engine_required_categories(&mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(IntegrityError::new(errors))
        }
    }

    /// Checks that every engine-required Hermetic role ([`ENGINE_REQUIRED_ABILITIES`],
    /// [`ENGINE_REQUIRED_ARTS`]) resolves, failing loudly with the offending id.
    ///
    /// Gated on the ruleset declaring a magus profile — the condition under which
    /// the engine dereferences these ids while computing a magus's play-stats. A
    /// non-magus fixture is exempt entirely. The two role families are then gated
    /// separately on the catalogue each one lives in, mirroring
    /// [`Self::validate_engine_required_categories`]: the required **abilities**
    /// (e.g. Parma Magica, which `magic_resistance()` looks up by slug and
    /// silently treats as 0 if absent) are enforced whenever the ruleset ships an
    /// abilities catalogue, independent of Arts; the required **arts** are
    /// enforced only once an Arts catalogue is shipped. A magus fixture that ships
    /// neither catalogue is exempt from the corresponding family. This keeps the
    /// guard from rejecting valid partial rulesets while still catching a real
    /// Hermetic ruleset that renamed or dropped one of the roles — in particular a
    /// magus ruleset that ships abilities but no Arts is no longer waved through.
    fn validate_engine_required_roles(&self, errors: &mut Vec<String>) {
        let has_magus = self.type_profiles.values().any(|p| p.is_magus);
        if !has_magus {
            return;
        }
        // The engine-required abilities are dereferenced for a magus (e.g.
        // `magic_resistance()` looks up Parma Magica by slug and yields 0 if
        // absent) whenever the ruleset ships an abilities catalogue — the exact
        // condition under which one of them going missing is a real defect. This
        // is gated independently of the Arts catalogue (a magus ruleset can ship
        // abilities without Arts) and, mirroring the personality-category check,
        // exempts a fixture that ships no abilities catalogue at all.
        if !self.abilities.is_empty() {
            for required in ENGINE_REQUIRED_ABILITIES {
                let id = Id::new(required);
                if !self.abilities.contains_key(&id) {
                    errors.push(format!(
                        "engine-required ability '{id}' is missing from the catalogue"
                    ));
                }
            }
        }
        // The engine-required Arts are only dereferenced once the ruleset ships an
        // Arts catalogue; a magus ruleset with no Arts at all needs none of them.
        if self.arts.is_empty() {
            return;
        }
        for required in ENGINE_REQUIRED_ARTS {
            let id = Id::new(required);
            if !self.arts.contains_key(&id) {
                errors.push(format!(
                    "engine-required art '{id}' is missing from the catalogue"
                ));
            }
        }
    }

    /// Checks that a ruleset shipping a V/F catalogue carries the engine-required
    /// personality category ([`ENGINE_REQUIRED_CATEGORY_PERSONALITY`]), failing
    /// loudly with the category name if no point item declares it.
    ///
    /// Gated on the ruleset shipping any point items — the exact condition under
    /// which the engine's Major-Personality-Flaw rule (`validation/scores.rs`)
    /// dereferences the category. An empty V/F catalogue (a minimal or non-standard
    /// fixture) ships no Personality Flaws and needs none of it, so it is exempt;
    /// this mirrors how the Hermetic-role check is gated on the ruleset actually
    /// shipping the relevant catalogue.
    fn validate_engine_required_categories(&self, errors: &mut Vec<String>) {
        if self.point_items.is_empty() {
            return;
        }
        let has_personality = self
            .point_items
            .values()
            .any(|item| item.category == ENGINE_REQUIRED_CATEGORY_PERSONALITY);
        if !has_personality {
            errors.push(format!(
                "engine-required V/F category '{ENGINE_REQUIRED_CATEGORY_PERSONALITY}' \
                 is missing from the catalogue"
            ));
        }
    }

    /// Validates the childhood block's own ability references: every ability on
    /// the spread list must resolve, and so must the native-language ability —
    /// which must additionally be parameterized, since "the character's native
    /// language" is one instance among many and a plain Ability could not tell
    /// German from any other language.
    ///
    /// A typo here would silently shrink the list the guided flow offers instead
    /// of failing the load, which is why it is a load-time referential check like
    /// every other ref in the rules data. It runs from
    /// [`Ruleset::validate_integrity`] rather than [`Ruleset::from_sources`] so a
    /// cached ruleset returning through [`Ruleset::from_serialized`] — the
    /// documented integrity gate — is held to exactly the same standard.
    fn validate_childhood_refs(&self, errors: &mut Vec<String>) {
        let Some(rules) = &self.life_stages else {
            return;
        };
        for ability in &rules.childhood.spread_abilities {
            if !self.abilities.contains_key(ability) {
                errors.push(format!(
                    "life-stage childhood spread names unknown ability '{ability}'"
                ));
            }
        }
        let native = &rules.childhood.native_language_ability;
        match self.abilities.get(native) {
            None => errors.push(format!(
                "life-stage childhood names unknown native-language ability '{native}'"
            )),
            Some(ability) if ability.parameter.is_none() => errors.push(format!(
                "life-stage childhood native-language ability '{native}' takes no parameter, so it cannot name one language"
            )),
            Some(_) => {}
        }
    }

    /// Validates the Sample Childhood packages against the abilities catalogue,
    /// the advancement table, and the childhood blocks they are a shortcut for.
    ///
    /// This is the **trust gate on transcribed rulebook data**. A package is one
    /// way of spending the two childhood blocks
    /// ("75 experience points in their native language … and 45 experience points
    /// to divide between …", Core Rules.md:2378), so its entries must price to
    /// exactly those blocks: a mistyped score fails the load rather than shipping
    /// a package that quietly costs 40 or 50 experience points. It also makes
    /// [`ChildhoodPackage::native_entry`]'s "at most one native entry" a real
    /// guarantee rather than a hope.
    ///
    /// Errors accumulate — a broken file reports every problem at once.
    fn validate_childhood_packages(&self, errors: &mut Vec<String>) {
        if self.childhoods.is_empty() {
            return;
        }
        // Without the blocks there is nothing to price a package against, so the
        // pricing rules below stay silent rather than blaming each package for
        // the missing file.
        let childhood = match self.life_stages {
            Some(ref rules) => Some(&rules.childhood),
            None => {
                errors.push(
                    "childhood packages are shipped without life-stage rules, \
                     so their experience cannot be priced"
                        .to_string(),
                );
                None
            }
        };

        for package in self.childhoods.values() {
            let id = &package.id;
            let mut slots_seen: BTreeSet<&str> = BTreeSet::new();
            let mut native_entries = 0usize;

            for entry in &package.entries {
                let ability = &entry.ability;
                match self.abilities.get(ability) {
                    None => errors.push(format!(
                        "childhood package '{id}' names unknown ability '{ability}'"
                    )),
                    // A parameterized ability needs a slot key for the player's
                    // answer to arrive under; the native language is the
                    // exception, being chosen once for the character as a whole.
                    Some(known) if known.parameter.is_some() => {
                        if entry.slot.is_none() && !entry.native {
                            errors.push(format!(
                                "childhood package '{id}' entry for parameterized ability \
                                 '{ability}' names no slot, so its value could never be asked for"
                            ));
                        }
                    }
                    Some(_) => {
                        if let Some(slot) = entry.slot.as_deref() {
                            errors.push(format!(
                                "childhood package '{id}' entry for plain ability '{ability}' \
                                 names slot '{slot}', which it has no parameter to fill"
                            ));
                        }
                    }
                }

                // Two entries of one parameterized ability are told apart by slot
                // alone, so a repeated key would collapse them into one row.
                if let Some(slot) = entry.slot.as_deref()
                    && !slots_seen.insert(slot)
                {
                    errors.push(format!("childhood package '{id}' repeats slot '{slot}'"));
                }

                if entry.native {
                    native_entries += 1;
                }

                // An unpriceable score is reported here, per entry; the two sum
                // checks below then stay silent rather than blaming the total.
                if self.advancement.xp_for_score(entry.score).is_none() {
                    errors.push(format!(
                        "childhood package '{id}' entry for '{ability}' has score {}, \
                         which the advancement table does not price",
                        entry.score
                    ));
                }

                if let Some(childhood) = childhood
                    && !entry.native
                    && !childhood.spread_abilities.contains(ability)
                {
                    errors.push(format!(
                        "childhood package '{id}' names ability '{ability}', \
                         which the childhood spread cannot buy"
                    ));
                }
            }

            if native_entries != 1 {
                errors.push(format!(
                    "childhood package '{id}' has {native_entries} native entries, \
                     but exactly one is required"
                ));
            }

            if let Some(childhood) = childhood {
                if let Some(native) = package.native_entry()
                    && native.ability != childhood.native_language_ability
                {
                    errors.push(format!(
                        "childhood package '{id}' native entry names ability '{}', not the \
                         childhood's native-language ability '{}'",
                        native.ability, childhood.native_language_ability
                    ));
                }
                if let Some(sum) = package.spread_xp(&self.advancement)
                    && sum != childhood.spread_xp
                {
                    errors.push(format!(
                        "childhood package '{id}' spread entries price to {sum} experience, \
                         not the childhood spread's {}",
                        childhood.spread_xp
                    ));
                }
                if let Some(sum) = package.native_xp(&self.advancement)
                    && sum != childhood.native_language_xp
                {
                    errors.push(format!(
                        "childhood package '{id}' native entry prices to {sum} experience, \
                         not the childhood's {}",
                        childhood.native_language_xp
                    ));
                }
            }

            if let Some(ref source) = package.source
                && !source.lines.is_valid()
            {
                errors.push(format!(
                    "childhood package '{id}': source line range start ({}) exceeds end ({})",
                    source.lines.start, source.lines.end
                ));
            }
        }
    }

    /// Validates a House record: every Virtue/Flaw it can grant must resolve to a
    /// known point item, and its source line range (if any) must be well-formed.
    ///
    /// A `Fixed` grant and each `Choice` option name a concrete point-item id, so
    /// those are integrity-checked here. An `Open` grant carries no item id (the
    /// player picks one at runtime, validated against its `GrantConstraint` in
    /// [`crate::validation`]), so there is nothing to resolve at load. Grant
    /// `params` values (the `ability`/`art` a Puissant targets) are deliberately
    /// NOT registry-checked, mirroring the forward-declared Ability/Art-domain
    /// policy on parameter values elsewhere.
    fn validate_house_refs(&self, house: &House, errors: &mut Vec<String>) {
        let id = &house.id;
        self.validate_grant_refs("house", id, &house.grants, errors);
        if let Some(ref source) = house.source
            && !source.lines.is_valid()
        {
            errors.push(format!(
                "house '{id}': source line range start ({}) exceeds end ({})",
                source.lines.start, source.lines.end
            ));
        }
    }

    /// Validates that every concrete item id a grant list names resolves. A
    /// `Fixed` grant and each `Choice` option name a point-item id; an `Open`
    /// grant carries none (checked at runtime against its `GrantConstraint`).
    /// Shared by House and Mythic-Companion-type integrity checks; `kind` and
    /// `owner` label the offending record in the error message. Grant `params`
    /// values are deliberately NOT registry-checked, mirroring the
    /// forward-declared Ability/Art-domain policy on parameter values elsewhere.
    fn validate_grant_refs(
        &self,
        kind: &str,
        owner: &Id,
        grants: &[Grant],
        errors: &mut Vec<String>,
    ) {
        for grant in grants {
            match grant {
                Grant::Fixed { item, .. } => {
                    if !self.point_items.contains_key(item) {
                        errors.push(format!(
                            "{kind} '{owner}': fixed grant references unknown item '{item}'"
                        ));
                    }
                }
                Grant::Choice { options, .. } => {
                    for option in options {
                        if !self.point_items.contains_key(&option.item_ref) {
                            errors.push(format!(
                                "{kind} '{owner}': choice option references unknown item '{}'",
                                option.item_ref
                            ));
                        }
                    }
                }
                // Open grants resolve to a player pick at runtime — nothing here.
                Grant::Open { .. } => {}
            }
        }
    }

    /// Validates a Mythic Companion type: every grant item, every required Virtue,
    /// and every required Flaw's default must resolve to a known point item, and
    /// its source line range (if any) must be well-formed. This is the load-time
    /// trust gate that a type's package can only ship once all its items are
    /// seeded. Required-flaw substitute *constraints* name categories (an open,
    /// forward-declared namespace), so they are not registry-checked — mirroring
    /// the profile category-field policy above.
    fn validate_mythic_type_refs(&self, mtype: &MythicCompanionType, errors: &mut Vec<String>) {
        let id = &mtype.id;
        self.validate_grant_refs("mythic companion type", id, &mtype.grants, errors);
        for req in &mtype.required_virtues {
            if !self.point_items.contains_key(&req.item_ref) {
                errors.push(format!(
                    "mythic companion type '{id}': required virtue references unknown item '{}'",
                    req.item_ref
                ));
            }
        }
        for flaw in &mtype.required_flaws {
            if !self.point_items.contains_key(&flaw.default.item_ref) {
                errors.push(format!(
                    "mythic companion type '{id}': required flaw default references unknown item '{}'",
                    flaw.default.item_ref
                ));
            }
        }
        if let Some(ref source) = mtype.source
            && !source.lines.is_valid()
        {
            errors.push(format!(
                "mythic companion type '{id}': source line range start ({}) exceeds end ({})",
                source.lines.start, source.lines.end
            ));
        }
    }

    /// Validates a spell: its Technique must resolve to a Technique-class Art, its
    /// Form to a Form-class Art, every requisite to a known Art, and its source
    /// line range (if any) must be well-formed. This is the load-time trust gate
    /// that a spell can only ship once its Arts exist.
    fn validate_spell_refs(&self, spell: &Spell, errors: &mut Vec<String>) {
        let id = &spell.id;
        match self.arts.get(&spell.technique) {
            None => errors.push(format!(
                "spell '{id}': technique references unknown art '{}'",
                spell.technique
            )),
            Some(art) if art.art_type != ArtType::Technique => errors.push(format!(
                "spell '{id}': technique '{}' is not a Technique-class Art",
                spell.technique
            )),
            Some(_) => {}
        }
        match self.arts.get(&spell.form) {
            None => errors.push(format!(
                "spell '{id}': form references unknown art '{}'",
                spell.form
            )),
            Some(art) if art.art_type != ArtType::Form => errors.push(format!(
                "spell '{id}': form '{}' is not a Form-class Art",
                spell.form
            )),
            Some(_) => {}
        }
        for req in &spell.requisites {
            if !self.arts.contains_key(req) {
                errors.push(format!(
                    "spell '{id}': requisite references unknown art '{req}'"
                ));
            }
        }
        // Ritual creation-legality (Core Rules.md:12279-12295, :12055, :12077,
        // :12039/:12115). Rituals are floored at level 20; Formulaic/Spontaneous
        // spells are capped at level 50; Year duration and Boundary target each
        // force a Ritual; a Momentary Creo spell that creates a lasting thing must
        // be a Ritual. Vision, though Boundary-level in difficulty, does NOT
        // (Core Rules.md:12099).
        if let Some(level) = spell.level {
            if spell.ritual && level < 20 {
                errors.push(format!(
                    "spell '{id}': a ritual spell must be at least level 20 (has {level})"
                ));
            }
            if !spell.ritual && level > 50 {
                errors.push(format!(
                    "spell '{id}': a non-ritual spell may not exceed level 50 (has {level})"
                ));
            }
        }
        if !spell.ritual {
            if spell.duration == Some(SpellDuration::Year) {
                errors.push(format!(
                    "spell '{id}': Year duration requires the spell to be a ritual"
                ));
            }
            if spell.target == Some(SpellTarget::Boundary) {
                errors.push(format!(
                    "spell '{id}': Boundary target requires the spell to be a ritual"
                ));
            }
            if spell.duration == Some(SpellDuration::Momentary)
                && spell.technique == Id::new(ID_CREO)
                && spell.creates_lasting
            {
                errors.push(format!(
                    "spell '{id}': a Momentary Creo spell that creates a lasting effect must be a ritual"
                ));
            }
        }
        if let Some(ref source) = spell.source
            && !source.lines.is_valid()
        {
            errors.push(format!(
                "spell '{id}': source line range start ({}) exceeds end ({})",
                source.lines.start, source.lines.end
            ));
        }
    }

    /// Validates a weapon: its combat `ability` must resolve to a known Ability and
    /// be a combat-appropriate one (a Martial Ability, or an Ability flagged
    /// `combat_ability` in data — Brawl, which the rules categorize as General but
    /// which is the combat Ability for unarmed and improvised weapons); and its
    /// source line range (if any) must be well-formed. This is the load-time trust
    /// gate that a weapon can only ship once its combat Ability exists. Source: Ars
    /// Magica - Definitive Edition (Core Rules).md:16988 (the "Ability" column names
    /// the Weapon Ability needed to use the weapon).
    fn validate_weapon_refs(&self, weapon: &Weapon, errors: &mut Vec<String>) {
        let id = &weapon.id;
        // The non-Martial combat Abilities (Brawl) are marked in data by the
        // `combat_ability` flag rather than a hardcoded slug, so a ruleset that
        // slugs unarmed combat differently just sets the flag.
        match self.abilities.get(&weapon.ability) {
            None => errors.push(format!(
                "weapon '{id}': ability references unknown ability '{}'",
                weapon.ability
            )),
            Some(ability)
                if ability.category != crate::ability::AbilityCategory::Martial
                    && !ability.combat_ability =>
            {
                errors.push(format!(
                    "weapon '{id}': ability '{}' is not a combat Ability (must be Martial or Brawl)",
                    weapon.ability
                ))
            }
            Some(_) => {}
        }
        if let Some(ref source) = weapon.source
            && !source.lines.is_valid()
        {
            errors.push(format!(
                "weapon '{id}': source line range start ({}) exceeds end ({})",
                source.lines.start, source.lines.end
            ));
        }
    }

    /// Recursively validates that prerequisite refs resolve to known registries:
    /// [`Prereq::Has`] against point items, [`Prereq::AbilityMin`] against the
    /// ability catalogue, [`Prereq::ArtMin`] against the Art catalogue, and
    /// [`Prereq::House`] against the House registry. `IsMagus` carries no
    /// reference at all, so there is nothing to check for it.
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
            Prereq::ArtMin { art, .. } => {
                if !self.arts.contains_key(art) {
                    errors.push(format!(
                        "{context_id}: prerequisite references unknown art '{art}'"
                    ));
                }
            }
            Prereq::House(ref_id) => {
                if !self.houses.contains_key(ref_id) {
                    errors.push(format!(
                        "{context_id}: prerequisite references unknown house '{ref_id}'"
                    ));
                }
            }
            // IsMagus carries no reference at all, so there is nothing to check.
            Prereq::IsMagus => {}
        }
    }

    /// Fails if `ability` does not resolve to a known Ability, naming the effect
    /// `kind` in the message. Shared by the fixed-ability-target effects.
    fn validate_ability_ref(&self, ability: &Id, kind: &str, id: &Id, errors: &mut Vec<String>) {
        if !self.abilities.contains_key(ability) {
            errors.push(format!(
                "{id}: effect '{kind}' references unknown ability '{ability}'"
            ));
        }
    }

    /// Fails for every id in a fixed ability list that does not resolve
    /// (`restricted_ability_xp`, `group_affinity_cost`).
    fn validate_ability_list_effect<'a>(
        &self,
        abilities: impl IntoIterator<Item = &'a Id>,
        kind: &str,
        id: &Id,
        errors: &mut Vec<String>,
    ) {
        for ability in abilities {
            self.validate_ability_ref(ability, kind, id, errors);
        }
    }

    /// Fails for every id in a fixed point-item list that does not resolve
    /// (`grants_selection`).
    fn validate_item_list_effect<'a>(
        &self,
        items: impl IntoIterator<Item = &'a Id>,
        kind: &str,
        id: &Id,
        errors: &mut Vec<String>,
    ) {
        for granted in items {
            if !self.point_items.contains_key(granted) {
                errors.push(format!(
                    "{id}: effect '{kind}' references unknown item '{granted}'"
                ));
            }
        }
    }

    /// Fails for every Form id in a fixed Art list that does not resolve
    /// (`elemental_magic`). Only checked when the Arts catalogue is loaded (the
    /// point-items file loads even in the Arts-less `from_core_json` path; the full
    /// app + spell paths load Arts and do enforce this), exactly like
    /// `validate_spell_refs`.
    fn validate_art_list_effect<'a>(
        &self,
        forms: impl IntoIterator<Item = &'a Id>,
        kind: &str,
        id: &Id,
        errors: &mut Vec<String>,
    ) {
        if self.arts.is_empty() {
            return;
        }
        for form in forms {
            if !self.arts.contains_key(form) {
                errors.push(format!(
                    "{id}: effect '{kind}' references unknown art '{form}'"
                ));
            }
        }
    }

    /// Validates a `deficient_art` effect: its declared parameter must exist and
    /// carry a Technique- or Form-domain (either fixes the class).
    fn validate_deficient_art_effect(
        &self,
        item: &PointItem,
        param: &str,
        id: &Id,
        errors: &mut Vec<String>,
    ) {
        match item.parameters.iter().find(|p| p.key.as_str() == param) {
            None => errors.push(format!(
                "{id}: effect 'deficient_art' references unknown parameter '{param}'"
            )),
            Some(def)
                if def.domain != ParameterDomain::Technique
                    && def.domain != ParameterDomain::Form =>
            {
                errors.push(format!(
                    "{id}: effect 'deficient_art' parameter '{param}' has domain '{}', expected 'technique' or 'form'",
                    def.domain
                ))
            }
            Some(_) => {}
        }
    }

    /// Validates that every [`Effect`] names a declared parameter whose domain
    /// matches the effect kind (`ability_bonus` → an `ability`-domain param,
    /// `characteristic_limit` → a `characteristic`-domain param). Effects resolve
    /// the target through that parameter, so a missing key or domain mismatch
    /// would silently never apply — fail loudly at load instead. Effects that carry
    /// a directly-stored ref instead of a parameter are validated inline via the
    /// small `validate_*_effect` helpers.
    fn validate_effect_refs(&self, item: &PointItem, id: &Id, errors: &mut Vec<String>) {
        for effect in &item.effects {
            let (param, expected, kind) = match effect {
                Effect::AbilityBonus { param, .. } => {
                    (param, ParameterDomain::Ability, "ability_bonus")
                }
                Effect::CharacteristicLimit { param, .. } => (
                    param,
                    ParameterDomain::Characteristic,
                    "characteristic_limit",
                ),
                Effect::ArtBonus { param, .. } => (param, ParameterDomain::Art, "art_bonus"),
                Effect::AffinityAbilityCost { param, .. } => {
                    (param, ParameterDomain::Ability, "affinity_ability_cost")
                }
                Effect::AffinityArtCost { param, .. } => {
                    (param, ParameterDomain::Art, "affinity_art_cost")
                }
                // Magical Focus / Academic Concentration name a free-text
                // descriptor the player types (a sub-Art focus, a study field).
                Effect::MagicalFocus { param, .. } => {
                    (param, ParameterDomain::Text, "magical_focus")
                }
                Effect::AbilityRollMod { param, .. } => {
                    (param, ParameterDomain::Text, "ability_roll_mod")
                }
                // Deficient Art targets a Technique OR a Form; the declared
                // param's domain (technique/form) is what fixes the class.
                Effect::DeficientArt { param } => {
                    self.validate_deficient_art_effect(item, param, id, errors);
                    continue;
                }
                // Fixed target: validate the directly-stored ability id resolves.
                Effect::AbilityScoreGrant { ability, .. } => {
                    self.validate_ability_ref(ability, "ability_score_grant", id, errors);
                    continue;
                }
                // Fixed eligibility list: validate each named ability id resolves,
                // like AbilityScoreGrant.ability and AbilityMin. The eligible
                // categories are a loose namespace matched at eval, not a registry,
                // so they are not checked here.
                Effect::RestrictedAbilityXp { abilities, .. } => {
                    self.validate_ability_list_effect(
                        abilities,
                        "restricted_ability_xp",
                        id,
                        errors,
                    );
                    continue;
                }
                // Fixed group of abilities the Affinity covers (Linguist).
                Effect::GroupAffinityCost { abilities, .. } => {
                    self.validate_ability_list_effect(abilities, "group_affinity_cost", id, errors);
                    continue;
                }
                // Fixed nested grant: every granted id must resolve to a point item
                // (a Virtue/Flaw), like a House grant's `item`.
                Effect::GrantsSelection { items } => {
                    self.validate_item_list_effect(items, "grants_selection", id, errors);
                    continue;
                }
                // Fixed target set: every elemental Form id the redistribution pools
                // over must resolve to a known Art.
                Effect::ElementalMagic { forms } => {
                    self.validate_art_list_effect(forms, "elemental_magic", id, errors);
                    continue;
                }
                // Fixed target: validate the directly-stored characteristic id
                // resolves to one of the eight Characteristics.
                Effect::CharacteristicScoreDelta { characteristic, .. } => {
                    if crate::characteristics::Characteristic::from_id(characteristic).is_none() {
                        errors.push(format!(
                            "{id}: effect 'characteristic_score_delta' references unknown characteristic '{characteristic}'"
                        ));
                    }
                    continue;
                }
                // The Form-scoped `deft_form` quirk names a Form via its param,
                // resolved against the selection's params exactly as
                // `deficient_art` resolves its Art (see the SpecialCastingMod doc
                // in types.rs); the declared param must exist and carry the Form
                // domain. A missing key or non-Form domain would make the waiver
                // silently never apply in derived.rs::in_play_mods.
                Effect::SpecialCastingMod {
                    kind: SpecialCasting::DeftForm,
                    param: Some(p),
                } => (p, ParameterDomain::Form, "special_casting_mod"),
                // deft_form REQUIRES a param naming the affected Form:
                // derived.rs::in_play_mods guards on `param.as_ref()`, so a
                // param-less deft_form would silently never apply its waiver.
                // Fail loudly rather than fall into the param-less catch-all.
                Effect::SpecialCastingMod {
                    kind: SpecialCasting::DeftForm,
                    param: None,
                } => {
                    errors.push(format!(
                        "{id}: effect 'special_casting_mod' kind 'deft_form' requires a param naming the affected Form"
                    ));
                    continue;
                }
                // No parameter or ref to resolve: the grant is intrinsic. The
                // param-less / non-`deft_form` SpecialCasting quirks fall here.
                Effect::SpellMasteryXp { .. }
                | Effect::GrantsSpellMastery { .. }
                | Effect::ItemLevelBudget { .. }
                | Effect::MasterpieceItem
                | Effect::TrueFaithGrant { .. }
                | Effect::WarpingGrant { .. }
                | Effect::SizeDelta { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::LaterLifeXpRate { .. }
                | Effect::AbilityAuthorization { .. }
                | Effect::LocalityAbilityCapFraction { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::GrantsReputation { .. }
                | Effect::MightGrant { .. }
                | Effect::PowerLevels { .. }
                // M5/5b in-play effects with no parameter or ref to resolve:
                // consumed intrinsically by derived.rs (5i).
                | Effect::CastingTotalMod { .. }
                | Effect::LabTotalMod { .. }
                | Effect::MagicTotalHalving { .. }
                | Effect::SoakMod { .. }
                | Effect::CombatMod { .. }
                | Effect::HealthMod { .. }
                | Effect::MagicResistanceMod { .. }
                | Effect::AgingMod { .. }
                | Effect::AdvancementMod { .. }
                | Effect::SpecialCastingMod { .. } => {
                    continue;
                }
            };
            match item.parameters.iter().find(|p| &p.key == param) {
                None => errors.push(format!(
                    "{id}: effect '{kind}' references unknown parameter '{param}'"
                )),
                Some(def) if def.domain != expected => errors.push(format!(
                    "{id}: effect '{kind}' parameter '{param}' has domain '{}', expected '{expected}'",
                    def.domain
                )),
                Some(_) => {}
            }
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

    /// Enforces that the Major and Minor variants of the SAME Virtue/Flaw mutually
    /// exclude — a character may only take one magnitude of a given item. Variant
    /// pairs are detected by a shared stem under two naming conventions: the suffix
    /// form `<stem>_major` / `<stem>_minor` and the prefix form
    /// `major_<stem>` / `minor_<stem>` (the latter covers Major / Minor Magical
    /// Focus). Only the Major side is inspected, so each pair is reported once, and
    /// a pair is considered only when BOTH members exist — a lone `*_major` (or a
    /// `*_minor` whose `*_major` counterpart is a different, absent concept, e.g.
    /// `virtue.minor_enchantments`) is never flagged. For every detected pair, both
    /// members must list each other in `incompatible_with`; otherwise this fails
    /// loudly naming both offending ids.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4405 ("A character
    /// can have only one Magical Focus, either major or minor").
    fn validate_magnitude_variant_exclusivity(&self, errors: &mut Vec<String>) {
        for (major_id, major_item) in &self.point_items {
            let Some(minor_id) = minor_variant_sibling(major_id) else {
                continue;
            };
            let Some(minor_item) = self.point_items.get(&minor_id) else {
                continue;
            };
            if !major_item.incompatible_with.contains(&minor_id)
                || !minor_item.incompatible_with.contains(major_id)
            {
                errors.push(format!(
                    "magnitude variants '{major_id}' and '{minor_id}' must be mutually incompatible_with each other"
                ));
            }
        }
    }
}

/// Given an item id, returns the id of its Minor sibling when the id names the
/// Major member of a magnitude-variant pair, under either the `<stem>_major`
/// suffix or the `major_<stem>` prefix convention. The `namespace.` prefix is
/// split off first so the magnitude affix is matched on the name, never the
/// namespace. The swap is an exact-stem substitution (`_major`↔`_minor`,
/// `major_`↔`minor_`) leaving the stem byte-identical, so unrelated stems never
/// collide. Returns `None` for any id that is not a Major variant.
fn minor_variant_sibling(id: &Id) -> Option<Id> {
    let s = id.as_str();
    let (namespace, name) = match s.split_once('.') {
        Some((ns, name)) => (Some(ns), name),
        None => (None, s),
    };
    let minor_name = if let Some(stem) = name.strip_suffix("_major") {
        format!("{stem}_minor")
    } else if let Some(stem) = name.strip_prefix("major_") {
        format!("minor_{stem}")
    } else {
        return None;
    };
    let minor_id = match namespace {
        Some(ns) => format!("{ns}.{minor_name}"),
        None => minor_name,
    };
    Some(Id::new(minor_id))
}

/// Parses and merges several JSON id-to-entry maps into one i18n map. A given id
/// may appear in only one source; a collision across files is an integrity error,
/// since the id namespaces (`virtue.*`, `ability.*`, …) are meant to be disjoint.
fn merge_i18n_sources(i18n_sources: &[&str]) -> Result<BTreeMap<Id, I18nEntry>, RulesetError> {
    let mut i18n: BTreeMap<Id, I18nEntry> = BTreeMap::new();
    let mut collisions = Vec::new();
    for source in i18n_sources {
        let entries: BTreeMap<String, I18nEntry> =
            serde_json::from_str(source).map_err(|e| RulesetError::parse(parse_source::I18N, e))?;
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
    Ok(i18n)
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
        let i18n = merge_i18n_sources(i18n_sources)?;
        Ok(Self { ruleset, i18n })
    }

    /// Like [`LocalizedRuleset::from_merged`], but fills each entry's missing
    /// optional text (summary, description, abbreviation, specialties) from a
    /// fallback locale — used so a not-yet-translated field surfaces the source
    /// language instead of rendering as empty (e.g. an English spell description
    /// when the German one is absent). The fallback is applied **per field**, not
    /// per entry: a present primary field is never overwritten. An id present only
    /// in the fallback is added whole, so nothing the source language documents is
    /// lost. `from_merged` / `new` stay fallback-free so completeness checks can
    /// assert on a single locale's raw coverage.
    pub fn from_merged_with_fallback(
        ruleset: Ruleset,
        primary_sources: &[&str],
        fallback_sources: &[&str],
    ) -> Result<Self, RulesetError> {
        let mut i18n = merge_i18n_sources(primary_sources)?;
        let fallback = merge_i18n_sources(fallback_sources)?;
        for (id, fb) in fallback {
            match i18n.get_mut(&id) {
                Some(entry) => {
                    if entry.summary.is_none() {
                        entry.summary = fb.summary;
                    }
                    if entry.description.is_none() {
                        entry.description = fb.description;
                    }
                    if entry.abbreviation.is_none() {
                        entry.abbreviation = fb.abbreviation;
                    }
                    if entry.specialties.is_empty() {
                        entry.specialties = fb.specialties;
                    }
                }
                None => {
                    i18n.insert(id, fb);
                }
            }
        }
        Ok(Self { ruleset, i18n })
    }

    /// Returns the full localized [`I18nEntry`] for the given id, if present.
    /// Use this when the caller needs more than the display name (name, summary,
    /// and description together); the field-specific helpers
    /// ([`LocalizedRuleset::display_name`], [`LocalizedRuleset::summary`],
    /// [`LocalizedRuleset::description`], [`LocalizedRuleset::abbreviation`],
    /// [`LocalizedRuleset::specialties`]) are thin wrappers over it.
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

    /// Returns the localized short abbreviation for the given id, if both the entry
    /// and its (optional) abbreviation are present. Carried by the Arts (`Cr`, `Ig`),
    /// whose sheet notation is the two abbreviations plus a level (`CrIg20`).
    pub fn abbreviation(&self, id: &Id) -> Option<&str> {
        self.i18n.get(id).and_then(|e| e.abbreviation.as_deref())
    }

    /// Returns the localized full description for the given id, if both the entry
    /// and its (optional) description are present.
    pub fn description(&self, id: &Id) -> Option<&str> {
        self.i18n.get(id).and_then(|e| e.description.as_deref())
    }

    /// Returns the localized example specialties for the given id. Yields an
    /// empty slice when the entry is missing or lists no specialties.
    pub fn specialties(&self, id: &Id) -> &[String] {
        self.i18n
            .get(id)
            .map(|e| e.specialties.as_slice())
            .unwrap_or(&[])
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
        "classification": "narrative",
        "magnitude": "free",
        "category": "special",
        "entity_kinds": ["character"]
      },
      {
        "id": "virtue.hermetic_magus",
        "kind": "virtue",
        "classification": "narrative",
        "magnitude": "free",
        "category": "social_status",
        "entity_kinds": ["character"],
        "prerequisites": { "kind": "has", "value": "virtue.the_gift" }
      },
      {
        "id": "virtue.gentle_gift",
        "kind": "virtue",
        "classification": "narrative",
        "magnitude": "major",
        "category": "hermetic",
        "entity_kinds": ["character"],
        "prerequisites": { "kind": "has", "value": "virtue.hermetic_magus" },
        "incompatible_with": ["flaw.blatant_gift"]
      },
      {
        "id": "flaw.blatant_gift",
        "kind": "flaw",
        "classification": "narrative",
        "magnitude": "major",
        "category": "hermetic",
        "entity_kinds": ["character"],
        "prerequisites": { "kind": "has", "value": "virtue.the_gift" },
        "incompatible_with": ["virtue.gentle_gift"]
      },
      {
        "id": "virtue.puissant_ability",
        "kind": "virtue",
        "classification": "narrative",
        "magnitude": "minor",
        "category": "general",
        "entity_kinds": ["character"],
        "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }]
      },
      {
        "id": "flaw.optimistic",
        "kind": "flaw",
        "classification": "narrative",
        "magnitude": "major",
        "category": "personality",
        "entity_kinds": ["character"]
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
        assert_eq!(rs.item_count(), 6);
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
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap();
        assert_eq!(rs.item_count(), 6);
        assert_eq!(rs.ability_count(), 2);
        assert_eq!(rs.id, Id::new("arm5-core"));

        // Omitting abilities yields an empty registry, matching from_json.
        let no_abilities = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap();
        assert_eq!(no_abilities.ability_count(), 0);
    }

    #[test]
    fn weapon_accepts_data_flagged_combat_ability() {
        // A non-Martial Ability flagged `combat_ability` in data is accepted as a
        // weapon's combat Ability, driven by the flag rather than a hardcoded slug.
        let abilities = r#"{
          "advancement": [],
          "abilities": [
            { "id": "ability.unarmed", "category": "general", "combat_ability": true }
          ]
        }"#;
        let equipment = r#"{ "weapons": [
          { "id": "weapon.fist", "kind": "melee", "init_mod": 0, "defense_mod": 0,
            "load": 0, "ability": "ability.unarmed" }
        ] }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: Some(abilities),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: Some(equipment),
            characteristics: None,
            life_stages: None,
            childhoods: None,
        });
        assert!(
            rs.is_ok(),
            "a data-flagged combat Ability should be accepted: {rs:?}"
        );
    }

    #[test]
    fn weapon_rejects_unflagged_non_martial_ability() {
        // A non-Martial Ability without the `combat_ability` flag is not a valid
        // weapon Ability.
        let abilities = r#"{
          "advancement": [],
          "abilities": [
            { "id": "ability.chatter", "category": "general" }
          ]
        }"#;
        let equipment = r#"{ "weapons": [
          { "id": "weapon.bad", "kind": "melee", "init_mod": 0, "defense_mod": 0,
            "load": 0, "ability": "ability.chatter" }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: Some(abilities),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: Some(equipment),
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        assert!(
            err.to_string().contains("not a combat Ability"),
            "expected combat-ability rejection, got: {err}"
        );
    }

    const VALID_HOUSES: &str = r#"{
      "houses": [
        { "id": "house.tytalus", "lineage_type": "societas", "grants": [
          { "kind": "fixed", "item": "virtue.puissant_ability" } ] },
        { "id": "house.bonisagus", "lineage_type": "true_lineage" }
      ]
    }"#;

    #[test]
    fn from_sources_loads_house_registry() {
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: Some(VALID_ABILITIES),
            arts: None,
            houses: Some(VALID_HOUSES),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap();
        assert_eq!(rs.house_count(), 2);
        assert!(rs.house(&Id::new("house.tytalus")).is_some());
        assert!(rs.house(&Id::new("house.missing")).is_none());
        assert_eq!(rs.houses().count(), 2);

        // Omitting houses yields an empty registry.
        let none = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap();
        assert_eq!(none.house_count(), 0);
    }

    #[test]
    fn from_sources_exposes_the_art_catalogue_accessors() {
        const VALID_ARTS: &str = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 },
            { "score": 2, "total_xp": 3 }
          ],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.ignem", "art_type": "form" }
          ]
        }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: Some(VALID_ARTS),
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap();
        assert_eq!(rs.art_count(), 2);
        assert!(rs.art(&Id::new("art.creo")).is_some());
        assert!(rs.art(&Id::new("art.missing")).is_none());
        assert_eq!(rs.arts().count(), 2);
        // The Art advancement table loads independently of the Ability table.
        assert_eq!(rs.art_advancement().xp_for_score(2), Some(3));
    }

    const SPELL_ARTS: &str = r#"{ "arts": [
      { "id": "art.creo", "art_type": "technique" },
      { "id": "art.intellego", "art_type": "technique" },
      { "id": "art.rego", "art_type": "technique" },
      { "id": "art.ignem", "art_type": "form" },
      { "id": "art.vim", "art_type": "form" }
    ] }"#;

    fn ruleset_with_spells(spells: &str) -> Result<Ruleset, RulesetError> {
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: Some(SPELL_ARTS),
            houses: None,
            mythic_types: None,
            spells: Some(spells),
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
    }

    #[test]
    fn from_sources_exposes_the_spell_catalogue_accessors() {
        let rs = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.pilum_of_fire", "technique": "art.creo", "form": "art.ignem", "level": 20 }
            ] }"#,
        )
        .unwrap();
        assert_eq!(rs.spell_count(), 1);
        assert!(rs.spell(&Id::new("spell.pilum_of_fire")).is_some());
        assert!(rs.spell(&Id::new("spell.missing")).is_none());
        assert_eq!(rs.spells().count(), 1);
    }

    #[test]
    fn from_sources_exposes_the_mythic_type_and_spell_mastery_accessors() {
        let mythic = r#"{ "types": [
          { "id": "mythic_type.alpha" },
          { "id": "mythic_type.beta" }
        ] }"#;
        let mastery = r#"{ "abilities": [
          { "id": "spell_mastery_ability.penetration" },
          { "id": "spell_mastery_ability.fast_casting" }
        ] }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: Some(mythic),
            spells: None,
            spell_mastery_abilities: Some(mastery),
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap();
        // Mythic Companion type accessors.
        assert_eq!(rs.mythic_type_count(), 2);
        assert_eq!(rs.mythic_types().count(), 2);
        assert!(
            rs.mythic_types()
                .any(|m| m.id == Id::new("mythic_type.alpha")),
            "the mythic-type iterator must yield the loaded types"
        );
        // Spell Mastery ability accessors.
        assert_eq!(rs.spell_mastery_ability_count(), 2);
        assert_eq!(rs.spell_mastery_abilities().count(), 2);
    }

    #[test]
    fn spell_with_form_as_technique_is_rejected() {
        // art.ignem is a Form, so using it as the Technique must fail integrity.
        let err = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.bad", "technique": "art.ignem", "form": "art.ignem", "level": 5 }
            ] }"#,
        )
        .unwrap_err();
        assert!(
            format!("{err:?}").contains("not a Technique-class Art"),
            "{err:?}"
        );
    }

    #[test]
    fn spell_with_unknown_form_is_rejected() {
        let err = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.bad", "technique": "art.creo", "form": "art.missing", "level": 5 }
            ] }"#,
        )
        .unwrap_err();
        assert!(
            format!("{err:?}").contains("unknown art 'art.missing'"),
            "{err:?}"
        );
    }

    #[test]
    fn duplicate_spell_id_is_rejected() {
        let err = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.x", "technique": "art.creo", "form": "art.ignem", "level": 5 },
              { "id": "spell.x", "technique": "art.creo", "form": "art.ignem", "level": 10 }
            ] }"#,
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("spell"), "{err:?}");
    }

    // --- Ritual creation-legality (5d). Source: Core Rules.md:12279-12295, :12055,
    //     :12077, :12039/:12115. ---

    #[test]
    fn ritual_spell_below_level_20_is_rejected() {
        let err = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.bad", "technique": "art.creo", "form": "art.ignem", "level": 15, "ritual": true }
            ] }"#,
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("ritual"), "{err:?}");
    }

    #[test]
    fn ritual_spell_at_level_20_loads() {
        let rs = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.ok", "technique": "art.creo", "form": "art.ignem", "level": 20, "ritual": true }
            ] }"#,
        );
        assert!(rs.is_ok(), "{rs:?}");
    }

    #[test]
    fn non_ritual_spell_above_level_50_is_rejected() {
        let err = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.bad", "technique": "art.creo", "form": "art.ignem", "level": 55 }
            ] }"#,
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("50"), "{err:?}");
    }

    #[test]
    fn year_duration_requires_ritual() {
        let err = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.bad", "technique": "art.creo", "form": "art.ignem", "level": 30, "duration": "year" }
            ] }"#,
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("Year"), "{err:?}");
    }

    #[test]
    fn boundary_target_requires_ritual() {
        let err = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.bad", "technique": "art.creo", "form": "art.ignem", "level": 30, "target": "boundary" }
            ] }"#,
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("Boundary"), "{err:?}");
    }

    /// Vision target is Boundary-level in difficulty but, unlike Boundary, does
    /// NOT require Ritual (Core Rules.md:12099).
    #[test]
    fn vision_target_does_not_require_ritual() {
        let rs = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.ok", "technique": "art.intellego", "form": "art.ignem", "level": 30, "target": "vision" }
            ] }"#,
        );
        assert!(rs.is_ok(), "{rs:?}");
    }

    #[test]
    fn momentary_creo_creating_lasting_requires_ritual() {
        let err = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.bad", "technique": "art.creo", "form": "art.ignem", "level": 30,
                "duration": "momentary", "creates_lasting": true }
            ] }"#,
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("ritual"), "{err:?}");
    }

    #[test]
    fn duplicate_house_id_is_rejected() {
        let dup = r#"{ "houses": [
          { "id": "house.tytalus", "lineage_type": "societas" },
          { "id": "house.tytalus", "lineage_type": "societas" }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: Some(dup),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => {
                assert!(
                    e.errors().iter().any(|m| m.contains("duplicate house ID")),
                    "expected a duplicate-house error, got {:?}",
                    e.errors()
                );
            }
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// A `Prereq::House` ref into the registry must resolve, now that a House
    /// registry exists (Phase 4 un-skips the previously-deferred check).
    #[test]
    fn prereq_house_ref_to_unknown_house_is_rejected() {
        let items = r#"[
          { "id": "virtue.tester", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "prerequisites": { "kind": "house", "value": "house.missing" } }
        ]"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: items,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: Some(VALID_HOUSES),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("unknown house 'house.missing'")),
                "expected an unknown-house prereq error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn house_fixed_grant_to_unknown_item_is_rejected() {
        let houses = r#"{ "houses": [
          { "id": "house.tytalus", "lineage_type": "societas", "grants": [
            { "kind": "fixed", "item": "virtue.does_not_exist" } ] }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: Some(houses),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("house.tytalus") && m.contains("virtue.does_not_exist")),
                "expected an unknown grant-item error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn house_choice_option_to_unknown_item_is_rejected() {
        let houses = r#"{ "houses": [
          { "id": "house.flambeau", "lineage_type": "societas", "grants": [
            { "kind": "choice", "choice_key": "x", "options": [
              { "ref": "virtue.puissant_ability" },
              { "ref": "virtue.nope" } ] } ] }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: Some(houses),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors().iter().any(|m| m.contains("virtue.nope")),
                "expected an unknown choice-option error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn house_invalid_source_range_is_rejected() {
        let houses = r#"{ "houses": [
          { "id": "house.tytalus", "lineage_type": "societas",
            "source": { "file": "x.md", "lines": [50, 10] } }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: Some(houses),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("house.tytalus") && m.contains("source line range")),
                "expected a source-range error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
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
    fn non_monotonic_advancement_table_is_rejected_at_load() {
        // total_xp must not decrease as score rises; otherwise xp_to_raise would
        // underflow. A malformed table must fail loudly at load.
        let bad = r#"{ "abilities": [], "advancement": [
          { "score": 1, "total_xp": 15 },
          { "score": 2, "total_xp": 5 }
        ] }"#;
        let err =
            Ruleset::from_json_with_abilities("t", "1", VALID_ITEMS, VALID_TYPES, bad).unwrap_err();
        match err {
            RulesetError::Integrity(e) => {
                assert!(
                    e.errors().iter().any(|m| m.contains("total_xp decreases")),
                    "expected a decreasing-total_xp error, got {:?}",
                    e.errors()
                );
            }
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_advancement_score_is_rejected_at_load() {
        let bad = r#"{ "abilities": [], "advancement": [
          { "score": 1, "total_xp": 5 },
          { "score": 1, "total_xp": 5 }
        ] }"#;
        let err =
            Ruleset::from_json_with_abilities("t", "1", VALID_ITEMS, VALID_TYPES, bad).unwrap_err();
        match err {
            RulesetError::Integrity(e) => {
                assert!(
                    e.errors().iter().any(|m| m.contains("duplicate score")),
                    "expected a duplicate-score error, got {:?}",
                    e.errors()
                );
            }
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn ability_min_prereq_must_resolve() {
        let items = r#"[{
          "id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
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
        let items = r#"[
          { "id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "prerequisites": { "kind": "ability_min", "value": { "ability": "ability.awareness", "score": 2 } } },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] }
        ]"#;
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
        assert_eq!(flaws, 2);

        let hermetic = rs.items_by_category("hermetic").count();
        assert_eq!(hermetic, 2);
    }

    #[test]
    fn missing_prereq_ref() {
        let items = r#"[{
          "id": "virtue.gentle_gift",
          "kind": "virtue",
          "classification": "narrative",
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
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "incompatible_with": ["flaw.b"]
          },
          {
            "id": "flaw.b",
            "kind": "flaw",
            "classification": "narrative",
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

    /// Issue F (load-level): Major and Minor variants of the same Virtue/Flaw
    /// (detected by shared stem under the `<stem>_major`/`<stem>_minor` suffix or
    /// `major_<stem>`/`minor_<stem>` prefix conventions) MUST be mutually
    /// `incompatible_with`. A variant pair that fails to declare it is rejected at
    /// load, naming both offending ids.
    #[test]
    fn magnitude_variant_pair_without_mutual_incompatibility_is_rejected() {
        let items = r#"[
          {
            "id": "virtue.foo_major",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "major",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.foo_minor",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "flaw.optimistic",
            "kind": "flaw",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "personality",
            "entity_kinds": ["character"]
          }
        ]"#;

        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("virtue.foo_major") && msg.contains("virtue.foo_minor"),
            "error should name both variant ids: {msg}"
        );
    }

    /// The prefix convention (`major_<stem>` / `minor_<stem>`) is detected too:
    /// a prefix pair that is not mutually incompatible is rejected.
    #[test]
    fn prefix_magnitude_variant_pair_without_incompatibility_is_rejected() {
        let items = r#"[
          {
            "id": "virtue.major_focus",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "major",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.minor_focus",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "flaw.optimistic",
            "kind": "flaw",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "personality",
            "entity_kinds": ["character"]
          }
        ]"#;

        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("virtue.major_focus") && msg.contains("virtue.minor_focus"),
            "error should name both prefix-variant ids: {msg}"
        );
    }

    /// False-positive guard: a lone `*_major` (or a `*_minor` whose `*_major`
    /// counterpart is a different, absent concept — e.g. the real
    /// `virtue.minor_enchantments`) has no sibling and MUST NOT be flagged.
    #[test]
    fn lone_magnitude_variant_without_sibling_is_not_flagged() {
        let items = r#"[
          {
            "id": "virtue.bar_major",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "major",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.minor_enchantments",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "flaw.optimistic",
            "kind": "flaw",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "personality",
            "entity_kinds": ["character"]
          }
        ]"#;

        // No sibling for either item -> no variant pair -> loads cleanly.
        let rs = Ruleset::from_json("test", "1", items, "[]").unwrap();
        assert_eq!(rs.item_count(), 3);
    }

    #[test]
    fn unknown_incompatible_ref() {
        let items = r#"[{
          "id": "virtue.a",
          "kind": "virtue",
          "classification": "narrative",
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
          "classification": "narrative",
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
    fn effect_referencing_unknown_parameter_is_rejected() {
        let items = r#"[{
          "id": "virtue.puissant_ability",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "effects": [{ "type": "ability_bonus", "param": "ability", "amount": 2 }]
        }]"#;
        // No `parameters` declared, so the effect's `ability` param is unknown.
        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown parameter 'ability'"), "{msg}");
    }

    #[test]
    fn restricted_ability_xp_referencing_unknown_ability_is_rejected() {
        let items = r#"[{
          "id": "virtue.educated",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "effects": [{ "type": "restricted_ability_xp", "amount": 50,
                        "abilities": ["ability.latin", "ability.does_not_exist"] }]
        }]"#;
        let abilities = r#"{ "advancement": [{ "score": 1, "total_xp": 5 }],
          "abilities": [{ "id": "ability.latin", "category": "academic" }] }"#;
        // The restricted pool's eligibility list names a non-existent ability;
        // like AbilityScoreGrant.ability and AbilityMin, it must fail at load.
        let err =
            Ruleset::from_json_with_abilities("test", "1", items, "[]", abilities).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("unknown ability 'ability.does_not_exist'"),
            "{msg}"
        );
    }

    #[test]
    fn effect_with_mismatched_parameter_domain_is_rejected() {
        let items = r#"[{
          "id": "virtue.great_characteristic",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "parameters": [{ "key": "characteristic", "type": "ref", "domain": "ability" }],
          "effects": [{ "type": "characteristic_limit", "param": "characteristic", "amount": 1 }]
        }]"#;
        // characteristic_limit needs a characteristic-domain param, not ability.
        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("expected 'characteristic'"), "{msg}");
    }

    #[test]
    fn deft_form_effect_referencing_unknown_parameter_is_rejected() {
        // A `deft_form` SpecialCastingMod names a Form via its `param`, resolved
        // against the selection's params exactly as `deficient_art` resolves its
        // Art. A param key that is not declared would silently never resolve, so
        // it must fail loudly at load.
        let items = r#"[
          { "id": "virtue.deft_form",
            "kind": "virtue",
            "classification": "in_play_effect",
            "magnitude": "minor",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "deft_form", "param": "form" }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        // No `parameters` declared, so the effect's `form` param is unknown.
        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("special_casting_mod") && msg.contains("unknown parameter 'form'"),
            "{msg}"
        );
    }

    #[test]
    fn deft_form_effect_with_mismatched_parameter_domain_is_rejected() {
        // deft_form names a Form; a param declared with a non-Form domain would
        // point the waiver at the wrong kind of Art (or none), so reject it.
        let items = r#"[
          { "id": "virtue.deft_form",
            "kind": "virtue",
            "classification": "in_play_effect",
            "magnitude": "minor",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "form", "type": "ref", "domain": "technique" }],
            "effects": [{ "type": "special_casting_mod", "kind": "deft_form", "param": "form" }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("special_casting_mod") && msg.contains("expected 'form'"),
            "{msg}"
        );
    }

    #[test]
    fn well_formed_deft_form_effect_loads() {
        // A `form`-domain param backing the deft_form quirk is well formed and
        // must load clean, mirroring production `virtue.deft_form`.
        let items = r#"[
          { "id": "virtue.deft_form",
            "kind": "virtue",
            "classification": "in_play_effect",
            "magnitude": "minor",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "form", "type": "ref", "domain": "form" }],
            "effects": [{ "type": "special_casting_mod", "kind": "deft_form", "param": "form" }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let rs = Ruleset::from_json("test", "1", items, "[]");
        assert!(rs.is_ok(), "{:?}", rs.err());
    }

    #[test]
    fn deft_form_effect_without_a_parameter_is_rejected() {
        // deft_form REQUIRES a param naming the affected Form: derived.rs's
        // in_play_mods guards on `param.as_ref()`, so a param-less deft_form
        // would silently never apply its waiver. It must fail loudly at load.
        let items = r#"[
          { "id": "virtue.deft_form",
            "kind": "virtue",
            "classification": "in_play_effect",
            "magnitude": "minor",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "deft_form" }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let err = Ruleset::from_json("test", "1", items, "[]").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("special_casting_mod")
                && msg.contains("deft_form")
                && msg.contains("requires a param naming the affected Form"),
            "{msg}"
        );
    }

    #[test]
    fn characteristic_domain_param_value_resolves() {
        use crate::types::{EntityKind, RulesetRef, Selection};
        use std::collections::BTreeMap;

        let items = r#"[
          { "id": "virtue.great_characteristic",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "characteristic", "type": "ref", "domain": "characteristic" }],
            "effects": [{ "type": "characteristic_limit", "param": "characteristic", "amount": 1 }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();

        let mut entity = crate::types::Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        // A valid characteristic id resolves; a bogus one is flagged.
        entity.selections = vec![Selection::with_params(
            Id::new("virtue.great_characteristic"),
            BTreeMap::from([("characteristic".into(), Id::new("characteristic.bogus"))]),
        )];
        let result = crate::validation::validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().map(|i| i.code.as_str()).collect();
        assert!(codes.contains(&"unknown_param_value"), "{codes:?}");
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
    fn from_merged_with_fallback_fills_missing_fields_from_fallback_locale() {
        let rs = Ruleset::from_json("arm5-core", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        // Primary (a de-like locale): a name but no description; and a second
        // entry the fallback also lacks a description for.
        let primary = r#"{
          "virtue.gentle_gift": { "name": "Sanfte Gabe" },
          "virtue.puissant_ability": { "name": "Kraftvoll (Fähigkeit)" }
        }"#;
        // Fallback (the source language): full text, plus an id absent from primary.
        let fallback = r#"{
          "virtue.gentle_gift": {
            "name": "Gentle Gift",
            "summary": "Your Gift is not disturbing.",
            "description": "People do not react with mistrust to you."
          },
          "flaw.blatant_gift": { "name": "Blatant Gift", "description": "Everyone distrusts you." }
        }"#;
        let loc = LocalizedRuleset::from_merged_with_fallback(rs, &[primary], &[fallback]).unwrap();

        // A present primary field wins — the name stays in the primary language.
        assert_eq!(
            loc.display_name(&Id::new("virtue.gentle_gift")),
            Some("Sanfte Gabe")
        );
        // Missing optional fields fall back to the source language, per field.
        assert_eq!(
            loc.description(&Id::new("virtue.gentle_gift")),
            Some("People do not react with mistrust to you.")
        );
        assert_eq!(
            loc.summary(&Id::new("virtue.gentle_gift")),
            Some("Your Gift is not disturbing.")
        );
        // An id present only in the fallback is added whole.
        assert_eq!(
            loc.display_name(&Id::new("flaw.blatant_gift")),
            Some("Blatant Gift")
        );
        // A primary entry with no fallback description stays without one.
        assert_eq!(loc.description(&Id::new("virtue.puissant_ability")), None);
    }

    /// The Art abbreviation is localized data (`Cr`, `Ig`), and the exported spell
    /// list composes its short Art+Level code out of it, so it needs the same kind of
    /// field-specific accessor the name and summary already have.
    #[test]
    fn localized_ruleset_abbreviation() {
        let rs = Ruleset::from_json("arm5-core", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        let i18n = r#"{
          "virtue.gentle_gift": { "name": "Gentle Gift", "abbreviation": "GG" },
          "virtue.puissant_ability": { "name": "Puissant (Ability)" }
        }"#;
        let loc = LocalizedRuleset::new(rs, i18n).unwrap();

        assert_eq!(loc.abbreviation(&Id::new("virtue.gentle_gift")), Some("GG"));
        // An entry that omits the abbreviation, and a missing id, both yield None.
        assert_eq!(loc.abbreviation(&Id::new("virtue.puissant_ability")), None);
        assert_eq!(loc.abbreviation(&Id::new("virtue.nonexistent")), None);
    }

    #[test]
    fn localized_ruleset_specialties() {
        let rs = Ruleset::from_json("arm5-core", "1", VALID_ITEMS, VALID_TYPES).unwrap();
        let i18n = r#"{
          "virtue.gentle_gift": {
            "name": "Gentle Gift",
            "specialties": ["alertness", "searching"]
          },
          "virtue.puissant_ability": { "name": "Puissant (Ability)" }
        }"#;
        let loc = LocalizedRuleset::new(rs, i18n).unwrap();

        // Specialties round-trip through the entry and the helper.
        assert_eq!(
            loc.entry(&Id::new("virtue.gentle_gift"))
                .unwrap()
                .specialties,
            vec!["alertness".to_string(), "searching".to_string()]
        );
        assert_eq!(
            loc.specialties(&Id::new("virtue.gentle_gift")),
            &["alertness".to_string(), "searching".to_string()]
        );
        // An entry that omits specialties yields an empty slice, as does a miss.
        assert!(
            loc.specialties(&Id::new("virtue.puissant_ability"))
                .is_empty()
        );
        assert!(loc.specialties(&Id::new("virtue.nonexistent")).is_empty());
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
          "classification": "narrative",
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
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "major", "category": "general", "entity_kinds": ["character"]}
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

    /// `review` is the wizard's synthetic terminal phase — it collects the issues
    /// no creation phase owns and is appended to every flow — so a profile that
    /// declares it would give the user two of them.
    /// Abilities for the life-stage fixtures: the native-language block names a
    /// PARAMETERIZED ability, since one language among many has to be nameable.
    const LIFE_STAGE_ABILITIES: &str = r#"{
      "advancement": [ { "score": 1, "total_xp": 5 } ],
      "abilities": [
        { "id": "ability.awareness", "category": "general" },
        { "id": "ability.living_language", "category": "general", "parameter": "language" }
      ]
    }"#;

    /// The life-stage file is optional (a ruleset may ship no life stages), and
    /// when present its numbers reach the engine.
    #[test]
    fn life_stage_rules_load_from_their_own_file() {
        let life_stages = r#"{
          "childhood": {
            "years": 5,
            "native_language_ability": "ability.living_language",
            "native_language_xp": 75,
            "spread_xp": 45,
            "spread_abilities": ["ability.awareness"]
          },
          "later_life": { "xp_per_year": 15 }
        }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            abilities: Some(LIFE_STAGE_ABILITIES),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: Some(life_stages),
            childhoods: None,
        })
        .unwrap();
        let rules = rs.life_stages().expect("life-stage rules loaded");
        assert_eq!(rules.childhood.native_language_xp, 75);
        assert_eq!(rules.later_life.xp_per_year, 15);

        let without = Ruleset::from_json("test", "1", "[]", "[]").unwrap();
        assert!(without.life_stages().is_none());
    }

    /// The childhood spread names abilities, so a typo there would silently shrink
    /// the list the wizard offers — a load-time referential-integrity failure, like
    /// every other ref in the rules data.
    #[test]
    fn a_childhood_spread_ability_must_resolve() {
        let life_stages = r#"{
          "childhood": {
            "years": 5,
            "native_language_ability": "ability.living_language",
            "native_language_xp": 75,
            "spread_xp": 45,
            "spread_abilities": ["ability.nonesuch"]
          },
          "later_life": { "xp_per_year": 15 }
        }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            abilities: Some(LIFE_STAGE_ABILITIES),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: Some(life_stages),
            childhoods: None,
        })
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("ability.nonesuch"),
            "should name the unresolved childhood ability: {msg}"
        );
    }

    /// "the character's native language" is one instance among many, so the ability
    /// the block names must be parameterized — a plain one could not tell German
    /// from every other language.
    #[test]
    fn the_native_language_ability_must_be_parameterized() {
        let life_stages = r#"{
          "childhood": {
            "years": 5,
            "native_language_ability": "ability.awareness",
            "native_language_xp": 75,
            "spread_xp": 45,
            "spread_abilities": ["ability.awareness"]
          },
          "later_life": { "xp_per_year": 15 }
        }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            abilities: Some(LIFE_STAGE_ABILITIES),
            life_stages: Some(life_stages),
            ..RulesetSources::default()
        })
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("ability.awareness") && msg.contains("parameter"),
            "should explain why the ability cannot name one language: {msg}"
        );
    }

    /// A cached ruleset is trusted no further than a freshly parsed one: the
    /// childhood cross-file checks belong to `validate_integrity`, which
    /// [`Ruleset::from_serialized`] re-runs, so a spread naming a nonexistent
    /// ability is rejected however the ruleset arrived.
    #[test]
    fn from_serialized_rejects_a_broken_childhood_spread() {
        let life_stages = r#"{
          "childhood": {
            "years": 5,
            "native_language_ability": "ability.living_language",
            "native_language_xp": 75,
            "spread_xp": 45,
            "spread_abilities": ["ability.awareness"]
          },
          "later_life": { "xp_per_year": 15 }
        }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            abilities: Some(LIFE_STAGE_ABILITIES),
            life_stages: Some(life_stages),
            ..RulesetSources::default()
        })
        .unwrap();

        let mut serialized = serde_json::to_value(&rs).unwrap();
        serialized["life_stages"]["childhood"]["spread_abilities"] =
            serde_json::Value::from(vec!["ability.nonesuch"]);

        let err = Ruleset::from_serialized(&serialized.to_string()).unwrap_err();
        assert_eq!(err.kind(), "integrity");
        assert!(
            err.to_string().contains("ability.nonesuch"),
            "should name the unresolved childhood spread ability: {err}"
        );
    }

    /// Sample Childhood packages are a catalogue of their own, so they load from
    /// their own file and are reachable by id — like every other registry. A
    /// ruleset that ships no package file simply offers none, which stands the
    /// shortcut down rather than making the childhood block unusable.
    #[test]
    fn childhood_packages_load_from_their_own_file() {
        let rs = Ruleset::from_sources(childhood_sources(VALID_CHILDHOOD)).unwrap();
        let package = rs
            .childhood(&Id::new("childhood.athletic"))
            .expect("the package is reachable by id");
        assert_eq!(package.entries.len(), 4);
        assert_eq!(rs.childhoods().count(), 1);

        let without = Ruleset::from_json("test", "1", "[]", "[]").unwrap();
        assert_eq!(without.childhoods().count(), 0);
    }

    /// Abilities for the Sample Childhood fixtures: the closed spread list of
    /// Core Rules.md:2378 in miniature (one parameterized `(Area) Lore`, one
    /// parameterized `(Living Language)`, three plain ones), plus a known ability
    /// the spread may NOT buy, and the canonical "ABILITY To Buy" prices for
    /// scores 1-5 (Core Rules.md:2406-2427).
    const CHILDHOOD_ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 },
        { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 },
        { "score": 4, "total_xp": 50 },
        { "score": 5, "total_xp": 75 }
      ],
      "abilities": [
        { "id": "ability.area_lore", "category": "general", "parameter": "area" },
        { "id": "ability.athletics", "category": "general" },
        { "id": "ability.awareness", "category": "general" },
        { "id": "ability.living_language", "category": "general", "parameter": "language" },
        { "id": "ability.magic_theory", "category": "general" },
        { "id": "ability.swim", "category": "general" }
      ]
    }"#;

    /// The two childhood blocks a package is a shortcut for: 75 in the native
    /// language, 45 across the spread list (Core Rules.md:2378).
    const CHILDHOOD_LIFE_STAGES: &str = r#"{
      "childhood": {
        "years": 5,
        "native_language_ability": "ability.living_language",
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": [
          "ability.area_lore",
          "ability.athletics",
          "ability.awareness",
          "ability.living_language",
          "ability.swim"
        ]
      },
      "later_life": { "xp_per_year": 15 }
    }"#;

    /// A package that satisfies every integrity rule: three plain spread entries
    /// pricing to 15+15+15 = 45, and one native language at 5 = 75.
    const VALID_CHILDHOOD: &str = r#"{
      "packages": [
        { "id": "childhood.athletic",
          "entries": [
            { "ability": "ability.athletics", "score": 2 },
            { "ability": "ability.awareness", "score": 2 },
            { "ability": "ability.living_language", "score": 5, "native": true },
            { "ability": "ability.swim", "score": 2 }
          ],
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [2384, 2384] } }
      ]
    }"#;

    /// A ruleset shipping the given package file against the childhood fixtures.
    fn childhood_sources(childhoods: &str) -> RulesetSources<'_> {
        RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            abilities: Some(CHILDHOOD_ABILITIES),
            life_stages: Some(CHILDHOOD_LIFE_STAGES),
            childhoods: Some(childhoods),
            ..RulesetSources::default()
        }
    }

    /// The integrity errors the given package file produces — every one of them,
    /// since a broken file must report all its problems at once.
    fn childhood_integrity_errors(childhoods: &str) -> Vec<String> {
        match Ruleset::from_sources(childhood_sources(childhoods)).unwrap_err() {
            RulesetError::Integrity(e) => e.errors().to_vec(),
            other => panic!("expected an integrity error, got {other:?}"),
        }
    }

    /// Asserts that some reported error carries `needle`.
    fn assert_childhood_error(errors: &[String], needle: &str) {
        assert!(
            errors.iter().any(|m| m.contains(needle)),
            "expected an error containing {needle:?}, got {errors:?}"
        );
    }

    /// A typo in an entry's ability would silently drop a score the package is
    /// supposed to grant, so the id must resolve like every other ref.
    #[test]
    fn a_childhood_entry_must_name_a_known_ability() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.nonesuch", "score": 2 },
                    { "ability": "ability.awareness", "score": 2 },
                    { "ability": "ability.living_language", "score": 5, "native": true },
                    { "ability": "ability.swim", "score": 2 }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.athletic' names unknown ability 'ability.nonesuch'",
        );
    }

    /// "(Area) Lore" needs to know WHICH area, and the slot key is where the
    /// player's answer goes — without one the entry's value could never be asked
    /// for. The native language is the exception: it is chosen once for the
    /// character, not per package.
    #[test]
    fn a_parameterized_entry_must_carry_a_slot() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.exploring",
                  "entries": [
                    { "ability": "ability.area_lore", "score": 2 },
                    { "ability": "ability.athletics", "score": 2 },
                    { "ability": "ability.living_language", "score": 5, "native": true },
                    { "ability": "ability.swim", "score": 2 }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.exploring' entry for parameterized ability 'ability.area_lore' names no slot",
        );
    }

    /// A slot on a plain ability would ask the player for something the Ability
    /// has no parameter to hold.
    #[test]
    fn a_plain_entry_must_not_carry_a_slot() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.athletics", "score": 2, "slot": "sport" },
                    { "ability": "ability.awareness", "score": 2 },
                    { "ability": "ability.living_language", "score": 5, "native": true },
                    { "ability": "ability.swim", "score": 2 }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.athletic' entry for plain ability 'ability.athletics' names slot 'sport'",
        );
    }

    /// Traveling's two Area Lores are told apart by slot alone, so a repeated key
    /// would collapse them into one row and quietly lose the second entry's
    /// experience.
    #[test]
    fn a_repeated_slot_is_rejected() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.traveling",
                  "entries": [
                    { "ability": "ability.area_lore", "score": 1, "slot": "area" },
                    { "ability": "ability.area_lore", "score": 1, "slot": "area" },
                    { "ability": "ability.athletics", "score": 2 },
                    { "ability": "ability.awareness", "score": 1 },
                    { "ability": "ability.living_language", "score": 5, "native": true },
                    { "ability": "ability.swim", "score": 2 }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.traveling' repeats slot 'area'",
        );
    }

    /// Every package in the book grants exactly one native language, and
    /// `native_entry` promises as much: none would leave the 75-point block
    /// unspent, two would make which one wins depend on file order.
    #[test]
    fn a_package_must_have_exactly_one_native_entry() {
        let none = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.athletics", "score": 2 },
                    { "ability": "ability.awareness", "score": 2 },
                    { "ability": "ability.swim", "score": 2 }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &none,
            "childhood package 'childhood.athletic' has 0 native entries, but exactly one is required",
        );

        let two = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.athletics", "score": 2 },
                    { "ability": "ability.awareness", "score": 2 },
                    { "ability": "ability.living_language", "score": 5, "native": true },
                    { "ability": "ability.living_language", "score": 5, "native": true },
                    { "ability": "ability.swim", "score": 2 }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &two,
            "childhood package 'childhood.athletic' has 2 native entries, but exactly one is required",
        );
    }

    /// The 75-point block funds the childhood's native-language Ability and
    /// nothing else, so a package flagging some other Ability as native would
    /// spend a block that cannot pay for it.
    #[test]
    fn the_native_entry_must_be_the_childhoods_native_language_ability() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.athletics", "score": 2 },
                    { "ability": "ability.awareness", "score": 5, "native": true },
                    { "ability": "ability.living_language", "score": 2, "slot": "language" },
                    { "ability": "ability.swim", "score": 2 }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.athletic' native entry names ability 'ability.awareness', not the childhood's native-language ability 'ability.living_language'",
        );
    }

    /// The spread is a closed list (Core Rules.md:2378), so a package naming an
    /// Ability outside it would smuggle in experience the block may not spend.
    #[test]
    fn a_spread_entry_must_be_on_the_childhood_spread_list() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.athletics", "score": 2 },
                    { "ability": "ability.living_language", "score": 5, "native": true },
                    { "ability": "ability.magic_theory", "score": 2 },
                    { "ability": "ability.swim", "score": 2 }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.athletic' names ability 'ability.magic_theory', which the childhood spread cannot buy",
        );
    }

    /// The trust gate on the transcription: a mistyped score must fail the load
    /// rather than ship a package that quietly costs 30 instead of 45.
    #[test]
    fn a_mispriced_spread_is_rejected() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.athletics", "score": 2 },
                    { "ability": "ability.awareness", "score": 2 },
                    { "ability": "ability.living_language", "score": 5, "native": true }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.athletic' spread entries price to 30 experience, not the childhood spread's 45",
        );
    }

    /// Same gate on the other block: "Native Language 5" is 75 experience, and a
    /// 4 would leave the block overfunded by 25.
    #[test]
    fn a_mispriced_native_language_is_rejected() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.athletics", "score": 2 },
                    { "ability": "ability.awareness", "score": 2 },
                    { "ability": "ability.living_language", "score": 4, "native": true },
                    { "ability": "ability.swim", "score": 2 }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.athletic' native entry prices to 50 experience, not the childhood's 75",
        );
    }

    /// A score with no row in the advancement table cannot be priced at all, so
    /// it is reported as such — and NOT also reported as a mispriced spread,
    /// which would blame the sum for a single bad entry.
    #[test]
    fn an_off_table_score_is_rejected() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.athletics", "score": 7 },
                    { "ability": "ability.living_language", "score": 5, "native": true }
                  ] }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.athletic' entry for 'ability.athletics' has score 7, which the advancement table does not price",
        );
        assert!(
            !errors.iter().any(|m| m.contains("spread entries price to")),
            "an unpriceable entry must not also be reported as a mispriced sum: {errors:?}"
        );
    }

    /// Without life-stage rules there are no blocks to price a package against,
    /// so shipping packages alone is a data error rather than a silently
    /// unchecked catalogue.
    #[test]
    fn childhood_packages_without_life_stage_rules_are_rejected() {
        let err = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            abilities: Some(CHILDHOOD_ABILITIES),
            childhoods: Some(VALID_CHILDHOOD),
            ..RulesetSources::default()
        })
        .unwrap_err();
        let errors = match err {
            RulesetError::Integrity(e) => e.errors().to_vec(),
            other => panic!("expected an integrity error, got {other:?}"),
        };
        assert_childhood_error(
            &errors,
            "childhood packages are shipped without life-stage rules, so their experience cannot be priced",
        );
        // The pricing rules cannot run without the blocks, so they must stay
        // silent rather than blame the package for the missing file.
        assert!(
            !errors.iter().any(|m| m.contains("price to")
                || m.contains("prices to")
                || m.contains("native-language ability")
                || m.contains("spread cannot buy")),
            "the pricing rules must not double-report: {errors:?}"
        );
    }

    /// Provenance is checked like every other source range in the ruleset.
    #[test]
    fn an_inverted_childhood_source_range_is_rejected() {
        let errors = childhood_integrity_errors(
            r#"{
              "packages": [
                { "id": "childhood.athletic",
                  "entries": [
                    { "ability": "ability.athletics", "score": 2 },
                    { "ability": "ability.awareness", "score": 2 },
                    { "ability": "ability.living_language", "score": 5, "native": true },
                    { "ability": "ability.swim", "score": 2 }
                  ],
                  "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [2388, 2384] } }
              ]
            }"#,
        );
        assert_childhood_error(
            &errors,
            "childhood package 'childhood.athletic': source line range start (2388) exceeds end (2384)",
        );
    }

    /// Two packages under one id would make `childhood()` return whichever won
    /// the map insert, so the collision is a load-time failure like every other
    /// registry's.
    #[test]
    fn duplicate_childhood_id_is_rejected() {
        let dup = r#"{
          "packages": [
            { "id": "childhood.athletic", "entries": [] },
            { "id": "childhood.athletic", "entries": [] }
          ]
        }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            childhoods: Some(dup),
            ..RulesetSources::default()
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => {
                assert!(
                    e.errors()
                        .iter()
                        .any(|m| m.contains("duplicate childhood package ID")),
                    "expected a duplicate-childhood error, got {:?}",
                    e.errors()
                );
            }
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    #[test]
    fn profile_may_not_declare_the_synthetic_review_phase() {
        let types = r#"[
          {"id": "companion", "budget": {"virtue_points": 10, "flaw_points": 10}, "creation_phases": ["concept", "review"]}
        ]"#;

        let err = Ruleset::from_json("test", "1", "[]", types).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("review") && msg.contains("companion"),
            "should name the offending phase and profile: {msg}"
        );
    }

    /// A repeated phase would be walked twice, with the second visit's Back
    /// landing on the same step the user just left.
    #[test]
    fn profile_may_not_repeat_a_creation_phase() {
        let types = r#"[
          {"id": "companion", "budget": {"virtue_points": 10, "flaw_points": 10}, "creation_phases": ["concept", "abilities", "concept"]}
        ]"#;

        let err = Ruleset::from_json("test", "1", "[]", types).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("concept") && msg.contains("companion"),
            "should name the repeated phase and profile: {msg}"
        );
    }

    /// An empty phase list is legal and means "this type has no guided flow": the
    /// wizard is not offered for it. Only a *declared* flow is validated. Many
    /// test fixtures rely on this, and a covenant profile will until M8 gives it
    /// real phases.
    #[test]
    fn profile_may_declare_no_creation_phases() {
        let types = r#"[
          {"id": "companion", "budget": {"virtue_points": 10, "flaw_points": 10}, "creation_phases": []}
        ]"#;

        let ruleset = Ruleset::from_json("test", "1", "[]", types).expect("empty phases are legal");
        assert!(
            ruleset.type_profiles[&Id::new("companion")]
                .creation_phases
                .is_empty()
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
            r#"[{"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": [], "prerequisites": {"kind": "has", "value": "virtue.missing"}}]"#,
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
            r#"[{"id":"virtue.a","kind":"virtue", "classification": "narrative","magnitude":"minor","category":"general","entity_kinds":[],"prerequisites":{"kind": "has", "value":"virtue.missing"}}]"#,
            "[]",
        )
        .unwrap_err();
        assert!(integrity_err.source().is_some());
    }

    #[test]
    fn integrity_error_exposes_individual_messages() {
        // Carries a personality-category item so the only integrity failures are
        // the two unresolved prerequisites (not a missing-category error).
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.x"}},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.y"}},
          {"id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"]}
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
            r#"[{"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": [], "prerequisites": {"kind": "has", "value": "virtue.missing"}}]"#,
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
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
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
                "ability_category_order",
                "advancement",
                "age_ability_caps",
                "armor",
                "art_advancement",
                "art_type_order",
                "arts",
                "categories_requiring_virtue",
                "characteristic_rules",
                // Always present, like `houses`: the frontend's record of Sample
                // Childhood packages is empty for a ruleset shipping none, never
                // absent.
                "childhoods",
                "houses",
                "id",
                // `life_stages` is absent here on purpose: this fixture ships no
                // life-stage file, and the field is skipped when `None` (a ruleset
                // without life stages must not grow an empty key). The
                // life-stage-bearing shape is asserted by
                // `life_stage_rules_load_from_their_own_file`.
                "magnitude_points",
                "mythic_companion_types",
                "point_items",
                "shields",
                "spell_mastery_abilities",
                "spells",
                "type_profiles",
                "version",
                "weapons",
            ],
            "Ruleset top-level field names are a stable public contract"
        );
        // The data maps are objects; advancement is a bare array.
        assert!(obj["point_items"].is_object());
        assert!(obj["type_profiles"].is_object());
        assert!(obj["abilities"].is_object());
        assert!(obj["advancement"].is_array());
        assert!(obj["age_ability_caps"].is_array());
        // Derived taxonomy surfaced to the UI: points map keyed by magnitude slug,
        // categories as an ordered array.
        assert_eq!(obj["magnitude_points"]["minor"], 1);
        assert!(obj["ability_category_order"].is_array());

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
              "classification": "narrative",
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
            r#"[{"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": [], "prerequisites": {"kind": "has", "value": "virtue.missing"}}]"#,
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
        let items = r#"[
          { "id": "virtue.x",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [
              { "key": "second", "type": "ref", "domain": "art" },
              { "key": "first", "type": "ref", "domain": "ability" }
            ] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let mut rs = Ruleset::from_json("test", "1", items, "[]").unwrap();
        rs.normalize();
        let item = rs.item(&Id::new("virtue.x")).unwrap();
        let keys: Vec<&str> = item.parameters.iter().map(|p| p.key.as_str()).collect();
        assert_eq!(keys, vec!["first", "second"]);
    }

    // --- Engine-required Hermetic roles (Finding A) --------------------------

    /// A magus type profile. Shipping this alongside an Arts catalogue is what
    /// makes the engine-required Hermetic role check fire.
    const MAGUS_TYPE: &str = r#"[{
      "id": "magus", "is_magus": true,
      "budget": { "virtue_points": 10, "flaw_points": 10 },
      "permitted_categories": ["general", "hermetic"],
      "gift_policy": "required",
      "creation_phases": []
    }]"#;

    /// An abilities catalogue carrying every engine-required Hermetic ability.
    const HERMETIC_ABILITIES: &str = r#"{ "abilities": [
      { "id": "ability.artes_liberales", "category": "academic" },
      { "id": "ability.magic_theory", "category": "arcane" },
      { "id": "ability.parma_magica", "category": "arcane" },
      { "id": "ability.penetration", "category": "arcane" },
      { "id": "ability.philosophiae", "category": "academic" }
    ] }"#;

    /// An Arts catalogue carrying every engine-required Art (Creo + Corpus).
    const HERMETIC_ARTS: &str = r#"{ "arts": [
      { "id": "art.creo", "art_type": "technique" },
      { "id": "art.corpus", "art_type": "form" }
    ] }"#;

    #[test]
    fn complete_hermetic_ruleset_passes_engine_role_check() {
        Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: MAGUS_TYPE,
            abilities: Some(HERMETIC_ABILITIES),
            arts: Some(HERMETIC_ARTS),
            ..RulesetSources::default()
        })
        .expect("a complete Hermetic ruleset must load");
    }

    #[test]
    fn missing_engine_required_ability_fails_integrity() {
        // An otherwise-complete Hermetic ruleset with Parma Magica removed.
        let abilities = r#"{ "abilities": [
          { "id": "ability.artes_liberales", "category": "academic" },
          { "id": "ability.magic_theory", "category": "arcane" },
          { "id": "ability.penetration", "category": "arcane" },
          { "id": "ability.philosophiae", "category": "academic" }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: MAGUS_TYPE,
            abilities: Some(abilities),
            arts: Some(HERMETIC_ARTS),
            ..RulesetSources::default()
        })
        .expect_err("a Hermetic ruleset missing Parma Magica must fail integrity");
        let msg = err.to_string();
        assert!(
            msg.contains("ability.parma_magica"),
            "the error must name the offending id, got: {msg}"
        );
    }

    #[test]
    fn missing_engine_required_art_fails_integrity() {
        let arts = r#"{ "arts": [ { "id": "art.creo", "art_type": "technique" } ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: MAGUS_TYPE,
            abilities: Some(HERMETIC_ABILITIES),
            arts: Some(arts),
            ..RulesetSources::default()
        })
        .expect_err("a Hermetic ruleset missing Corpus must fail integrity");
        assert!(
            err.to_string().contains("art.corpus"),
            "the error must name the offending id"
        );
    }

    #[test]
    fn engine_role_check_skips_non_hermetic_ruleset() {
        // A non-magus ruleset shipping a single Art (to exercise Art mechanics)
        // needs none of the Hermetic engine roles: the check is gated on a magus
        // profile being present.
        let arts = r#"{ "arts": [ { "id": "art.creo", "art_type": "technique" } ] }"#;
        Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: Some("{}"),
            arts: Some(arts),
            ..RulesetSources::default()
        })
        .expect("a non-Hermetic ruleset must not require the engine roles");
    }

    #[test]
    fn magus_without_arts_still_requires_engine_abilities() {
        // A magus ruleset that ships an abilities catalogue but NO Arts must
        // still enforce the engine-required abilities: `magic_resistance()`
        // dereferences Parma Magica by slug and silently yields 0 if absent, so
        // a magus ruleset missing Parma would compute MR as if Parma=0. The
        // ability guard must not be skipped just because the Arts catalogue is
        // empty. Regression guard for the shared `arts.is_empty()` early return.
        let abilities = r#"{ "abilities": [
          { "id": "ability.artes_liberales", "category": "academic" },
          { "id": "ability.magic_theory", "category": "arcane" },
          { "id": "ability.penetration", "category": "arcane" },
          { "id": "ability.philosophiae", "category": "academic" }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: MAGUS_TYPE,
            abilities: Some(abilities),
            arts: None,
            ..RulesetSources::default()
        })
        .expect_err("a magus ruleset missing Parma Magica must fail even with no Arts");
        assert!(
            err.to_string().contains("ability.parma_magica"),
            "the error must name the missing required ability, got: {}",
            err
        );
    }

    // --- Engine-required V/F category (personality) --------------------------

    #[test]
    fn missing_engine_required_personality_category_fails_integrity() {
        // A V/F catalogue that ships point items but none in the personality
        // category: the Major-Personality-Flaw rule (validation/scores.rs) would
        // silently count zero and wrongly reject every ±6 trait, so load must fail.
        let items = r#"[
          { "id": "virtue.keen_vision", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"] }
        ]"#;
        let err = Ruleset::from_json("t", "1", items, VALID_TYPES)
            .expect_err("a V/F catalogue lacking the personality category must fail integrity");
        let msg = err.to_string();
        assert!(
            msg.contains(ENGINE_REQUIRED_CATEGORY_PERSONALITY),
            "the error must name the missing category, got: {msg}"
        );
    }

    #[test]
    fn personality_category_check_skips_empty_vf_catalogue() {
        // An empty V/F catalogue ships no Personality Flaws, so the category is
        // not engine-required — mirroring how the Hermetic-role check is gated on
        // the ruleset actually shipping the relevant catalogue.
        Ruleset::from_json("t", "1", "[]", "[]")
            .expect("an empty V/F catalogue must not require the personality category");
    }

    #[test]
    fn vf_catalogue_with_personality_category_passes() {
        // The standard example fixture carries a personality-category item, so it
        // satisfies the engine-required-category check.
        Ruleset::from_json("arm5-core", "2024.1", VALID_ITEMS, VALID_TYPES)
            .expect("a V/F catalogue carrying the personality category must load");
    }

    // --- Negative "fail loudly" tests for effect / grant / source-range refs. ---
    // Each fixture is otherwise integrity-valid (a personality-category Flaw is
    // present, no magus profile / Arts) so ONLY the intended defect triggers.

    /// A `grants_selection` effect naming an item that is not in the catalogue
    /// must fail integrity at load.
    #[test]
    fn grants_selection_effect_referencing_unknown_item_is_rejected() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] },
          { "id": "virtue.templar", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
            "category": "general", "entity_kinds": ["character"],
            "effects": [ { "type": "grants_selection", "items": ["virtue.does_not_exist"] } ] }
        ]"#;
        let err = Ruleset::from_json("t", "1", items, VALID_TYPES).unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("grants_selection") && m.contains("virtue.does_not_exist")),
                "expected an unknown grants_selection-item error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// A `characteristic_score_delta` effect naming an id that is not one of the
    /// eight Characteristics must fail integrity at load.
    #[test]
    fn characteristic_score_delta_effect_referencing_unknown_characteristic_is_rejected() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] },
          { "id": "virtue.giant_blood", "kind": "virtue", "classification": "narrative", "magnitude": "major",
            "category": "general", "entity_kinds": ["character"],
            "effects": [ { "type": "characteristic_score_delta", "characteristic": "characteristic.bogus", "amount": 1 } ] }
        ]"#;
        let err = Ruleset::from_json("t", "1", items, VALID_TYPES).unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("characteristic_score_delta")
                        && m.contains("characteristic.bogus")),
                "expected an unknown-characteristic error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// A Mythic Companion type whose `required_virtues` names an item not in the
    /// catalogue must fail integrity at load.
    #[test]
    fn mythic_type_required_virtue_referencing_unknown_item_is_rejected() {
        let mythic = r#"{ "types": [
          { "id": "mythic_type.test", "required_virtues": [ { "ref": "virtue.does_not_exist" } ] }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: Some(mythic),
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("required virtue") && m.contains("virtue.does_not_exist")),
                "expected an unknown required-virtue error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// A Mythic Companion type whose `required_flaws` default names an item not in
    /// the catalogue must fail integrity at load.
    #[test]
    fn mythic_type_required_flaw_default_referencing_unknown_item_is_rejected() {
        let mythic = r#"{ "types": [
          { "id": "mythic_type.test", "required_flaws": [
            { "default": { "ref": "flaw.does_not_exist" },
              "constraint": { "kind": "flaw", "magnitude": "major" } } ] }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: Some(mythic),
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("required flaw default")
                        && m.contains("flaw.does_not_exist")),
                "expected an unknown required-flaw error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// A Mythic Companion type whose source line range is inverted (start > end)
    /// must fail integrity at load.
    #[test]
    fn mythic_type_invalid_source_range_is_rejected() {
        let mythic = r#"{ "types": [
          { "id": "mythic_type.test",
            "source": { "file": "x.md", "lines": [50, 10] } }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: Some(mythic),
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("mythic_type.test") && m.contains("source line range")),
                "expected a mythic-type source-range error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// A Spell Mastery ability whose source line range is inverted (start > end)
    /// must fail integrity at load.
    #[test]
    fn spell_mastery_ability_invalid_source_range_is_rejected() {
        let mastery = r#"{ "abilities": [
          { "id": "spell_mastery_ability.penetration",
            "source": { "file": "x.md", "lines": [50, 10] } }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: Some(mastery),
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("spell_mastery_ability.penetration")
                        && m.contains("source line range")),
                "expected a spell-mastery source-range error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// A weapon whose source line range is inverted (start > end) must fail
    /// integrity at load. Its combat Ability resolves so only the range defect
    /// triggers.
    #[test]
    fn weapon_invalid_source_range_is_rejected() {
        let abilities = r#"{ "advancement": [], "abilities": [
          { "id": "ability.single_weapon", "category": "martial" }
        ] }"#;
        let equipment = r#"{ "weapons": [
          { "id": "weapon.longsword", "kind": "melee", "init_mod": 2, "defense_mod": 1,
            "load": 1, "ability": "ability.single_weapon",
            "source": { "file": "x.md", "lines": [50, 10] } }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: Some(abilities),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: Some(equipment),
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("weapon.longsword") && m.contains("source line range")),
                "expected a weapon source-range error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// A shield whose source line range is inverted (start > end) must fail
    /// integrity at load.
    #[test]
    fn shield_invalid_source_range_is_rejected() {
        let equipment = r#"{ "shields": [
          { "id": "shield.heater", "init_mod": 0, "attack_mod": 0, "defense_mod": 2, "load": 1,
            "min_strength": 0, "source": { "file": "x.md", "lines": [50, 10] } }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: Some(equipment),
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("shield.heater") && m.contains("source line range")),
                "expected a shield source-range error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// An armor whose source line range is inverted (start > end) must fail
    /// integrity at load.
    #[test]
    fn armor_invalid_source_range_is_rejected() {
        let equipment = r#"{ "armor": [
          { "id": "armor.chain_mail_full", "protection": 9, "load": 6,
            "source": { "file": "x.md", "lines": [50, 10] } }
        ] }"#;
        let err = Ruleset::from_sources(RulesetSources {
            id: "t",
            version: "1",
            point_items: VALID_ITEMS,
            type_profiles: VALID_TYPES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: Some(equipment),
            characteristics: None,
            life_stages: None,
            childhoods: None,
        })
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => {
                assert!(
                    e.errors()
                        .iter()
                        .any(|m| m.contains("armor.chain_mail_full")
                            && m.contains("source line range")),
                    "expected an armor source-range error, got {:?}",
                    e.errors()
                )
            }
            other => panic!("expected integrity error, got {other:?}"),
        }
    }

    /// A spell whose source line range is inverted (start > end) must fail
    /// integrity at load.
    #[test]
    fn spell_invalid_source_range_is_rejected() {
        let err = ruleset_with_spells(
            r#"{ "spells": [
              { "id": "spell.ok", "technique": "art.creo", "form": "art.ignem", "level": 20,
                "source": { "file": "x.md", "lines": [50, 10] } }
            ] }"#,
        )
        .unwrap_err();
        match err {
            RulesetError::Integrity(e) => assert!(
                e.errors()
                    .iter()
                    .any(|m| m.contains("spell.ok") && m.contains("source line range")),
                "expected a spell source-range error, got {:?}",
                e.errors()
            ),
            other => panic!("expected integrity error, got {other:?}"),
        }
    }
}
