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
use crate::aging::{
    AgingRow, AgingRowEffect, AgingRules, CrisisOutcome, CrisisRow, CrisisSeverity,
};
use crate::art::{Art, ArtType, ArtsFile};
use crate::characteristics::CharacteristicRules;
use crate::childhood::ChildhoodPackage;
use crate::equipment::{Armor, EquipmentFile, Shield, Weapon};
use crate::grant::Grant;
use crate::house::{House, HousesFile};
use crate::life_stage::{ChildhoodRules, LifeStageRules};
use crate::mythic_companion::{MythicCompanionType, MythicCompanionTypesFile};
use crate::spell::{RITUAL_MIN_LEVEL, Spell, SpellDuration, SpellTarget, SpellsFile};
use crate::spell_mastery::{SpellMasteryAbilitiesFile, SpellMasteryAbility};
use crate::types::{
    AURA_MODIFIER_MAX, AURA_MODIFIER_MIN, CreationPhase, Effect, EntityTypeProfile, I18nEntry, Id,
    ItemKind, Magnitude, PREREQ_MAX_DEPTH, ParameterDomain, PointItem, Prereq, RulesetRef,
    SourceRef, SpecialCasting,
};

mod accessors;
mod integrity;
mod parse;

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
///   "ritual_min_level": 20,
///   "aura_modifier_min": -50,
///   "aura_modifier_max": 10,
///   "houses": { "house.bonisagus": { /* House */ } },
///   "childhoods": { "childhood.athletic": { /* ChildhoodPackage */ } },
///   "aging": { /* AgingRules */ },
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
    /// The age → maximum-Ability-score band table (Ars Magica - Definitive Edition (Core Rules).md:2366-2374). Caps *Ability*
    /// scores by age, loaded from `rules/core/abilities.json` beside the Ability
    /// advancement table. Empty for a ruleset that ships no age caps. Serialized
    /// whole to the frontend; the `age_ability_caps` field name is a stable public
    /// contract.
    #[serde(default)]
    pub(crate) age_ability_caps: AgeAbilityCaps,
    /// Ability categories a character may only buy with a permitting Virtue
    /// (Ars Magica - Definitive Edition (Core Rules).md:2315), loaded from `rules/core/abilities.json` beside the age caps.
    /// Empty for a ruleset that gates none, which stands the rule down rather than
    /// letting the engine invent the list. Serialized whole to the frontend; the
    /// `categories_requiring_virtue` field name is a stable public contract.
    #[serde(default)]
    pub(crate) categories_requiring_virtue: BTreeSet<AbilityCategory>,
    /// The scholarly-language expectation Academic Abilities normally carry
    /// (Ars Magica - Definitive Edition (Core Rules).md:7151), loaded beside the categories above. `None` for a ruleset that
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
    /// The aging tables (Living Conditions + Aging Roll), if the ruleset ships
    /// them. `None` for a ruleset without an aging file, which stands the whole
    /// subsystem down rather than letting the engine invent a table. Serialized
    /// whole to the frontend; the `aging` field name is a stable public contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) aging: Option<AgingRules>,
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
    /// The minimum level a Ritual spell may be learned at, derived from
    /// [`crate::spell::RITUAL_MIN_LEVEL`]. Serialized to the frontend so the UI
    /// reads the Ritual floor from engine data instead of re-hardcoding it.
    /// Derived data, not authored (see `magnitude_points`). The
    /// `ritual_min_level` field name is a stable public contract.
    #[serde(default)]
    pub(crate) ritual_min_level: u8,
    /// The lowest rules-legal [`crate::types::Entity::aura`] modifier, mirrored
    /// from [`crate::types::AURA_MODIFIER_MIN`]. Serialized to the frontend so
    /// the aura number inputs (`DerivedTotalsPanel`, `MagicPossessions`) bound
    /// themselves from engine data instead of re-hardcoding the rules range.
    /// Derived data, not authored (see `magnitude_points`). The
    /// `aura_modifier_min` field name is a stable public contract.
    #[serde(default)]
    pub(crate) aura_modifier_min: i32,
    /// The highest rules-legal [`crate::types::Entity::aura`] modifier, mirrored
    /// from [`crate::types::AURA_MODIFIER_MAX`]. See `aura_modifier_min`.
    /// Derived data, not authored (see `magnitude_points`). The
    /// `aura_modifier_max` field name is a stable public contract.
    #[serde(default)]
    pub(crate) aura_modifier_max: i32,
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

impl Ruleset {
    /// Overwrites this `Ruleset`'s six engine-derived fields — fixed
    /// taxonomies and engineering constants that are never authored data (see
    /// `magnitude_points`'s own doc). The single place both construction
    /// paths call: [`parse::assemble_ruleset`] (the fresh-parse path,
    /// `Ruleset::from_sources`) and [`Ruleset::from_serialized`] (the cached
    /// path, which must re-derive rather than trust the incoming JSON — see
    /// that method's doc). Before V46 each path carried its own copy of these
    /// six assignments; drift between them was a silent possibility. Now a
    /// change to any of the six needs one edit, and both paths pick it up.
    fn apply_derived_fields(&mut self) {
        self.magnitude_points = derived_magnitude_points();
        self.ability_category_order = AbilityCategory::ALL.to_vec();
        self.art_type_order = ArtType::ALL.to_vec();
        self.ritual_min_level = RITUAL_MIN_LEVEL;
        self.aura_modifier_min = AURA_MODIFIER_MIN;
        self.aura_modifier_max = AURA_MODIFIER_MAX;
    }
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
    /// Aging-tables JSON (`{ "start_age", "living_conditions", "outcomes", ... }`),
    /// or `None` for a ruleset that ships no aging rules — which stands the aging
    /// subsystem down entirely rather than letting the engine invent a table.
    pub aging: Option<&'a str>,
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
    pub const AGING: &str = "aging";
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
    /// The age → maximum-Ability-score band table (Ars Magica - Definitive Edition (Core Rules).md:2366-2374). Caps *Ability*
    /// scores by age, so it lives beside the Ability advancement table.
    #[serde(default)]
    age_ability_caps: AgeAbilityCaps,
    /// Ability categories a character may only buy with a permitting Virtue
    /// (Ars Magica - Definitive Edition (Core Rules).md:2315). Data, not a hardcoded list, so a ruleset that gates a different
    /// set says so in its own file; empty means the rule is not enforced.
    #[serde(default)]
    categories_requiring_virtue: BTreeSet<AbilityCategory>,
    /// The scholarly-language expectation for Academic Abilities (Ars Magica - Definitive Edition (Core Rules).md:7151), or
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
    /// One example the rules themselves name — "For most characters, Latin 3 is
    /// required" (`:7151`) — as a language-neutral slug, so a UI can say which
    /// language the passage means beside the wider check the engine enforces.
    ///
    /// A **label key, not a `ref`**: see [`crate::AbilityRequirement::exemplar`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exemplar: Option<String>,
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
/// Ars Magica - Definitive Edition (Core Rules).md:2500-2503, :2820) off this exact category. Engine invariant like
/// [`ENGINE_REQUIRED_ABILITIES`] — a ruleset that renamed or dropped it would make
/// the engine silently count zero Major Personality Flaws and wrongly reject every
/// ±6 trait, so [`Ruleset::validate_integrity`] enforces its presence for any
/// ruleset that ships a V/F catalogue.
pub(crate) const ENGINE_REQUIRED_CATEGORY_PERSONALITY: &str = "personality";

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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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

    // --- Ritual creation-legality (5d). Source: Ars Magica - Definitive
    //     Edition (Core Rules).md:12279-12295, :12055,
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
    /// NOT require Ritual (Ars Magica - Definitive Edition (Core Rules).md:12099).
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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

    /// Wraps `IsMagus` (a leaf that needs no ref lookup, so it cannot itself
    /// trigger an "unknown ref" error and confuse the depth assertion) in
    /// `wraps` levels of `Prereq::All`. The leaf then sits at depth
    /// `wraps + 1` (the top-level prerequisite is depth 1).
    fn nested_prereq(wraps: usize) -> Prereq {
        let mut p = Prereq::IsMagus;
        for _ in 0..wraps {
            p = Prereq::All(vec![p]);
        }
        p
    }

    /// Built via `Prereq` values directly rather than JSON: constructing a
    /// 32+-level-deep nested JSON literal by hand is impractical, and going
    /// through `serde_json` would additionally exercise *its* independent
    /// recursion limit (default 128) rather than pinning ours. Mutating the
    /// already-loaded `Ruleset`'s `point_items` map and calling
    /// `validate_integrity()` directly isolates exactly the code under test:
    /// `Ruleset::validate_prereq_refs`'s own depth bound (K8).
    fn ruleset_with_prereq(prereq: Prereq) -> Ruleset {
        // The second item satisfies validate_engine_required_categories's
        // "the catalogue carries a personality-category item" check — an
        // unrelated engine-required-catalogue invariant that would otherwise
        // fail first and mask the depth assertion under test.
        let items = r#"[
          { "id": "virtue.tester", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "category": "general" },
          { "id": "flaw.personality_filler", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "category": "personality" }
        ]"#;
        let mut rs = Ruleset::from_json("t", "1", items, "[]").unwrap();
        rs.point_items
            .get_mut(&Id::new("virtue.tester"))
            .unwrap()
            .prerequisites = Some(prereq);
        rs
    }

    #[test]
    fn a_prereq_nested_exactly_to_the_depth_limit_still_validates() {
        // Leaf at depth PREREQ_MAX_DEPTH (== the limit, not past it) must be
        // accepted — the guard must not reject a merely deep-but-legal tree.
        let rs = ruleset_with_prereq(nested_prereq(PREREQ_MAX_DEPTH - 1));
        assert!(
            rs.validate_integrity().is_ok(),
            "a prerequisite nested exactly to PREREQ_MAX_DEPTH should still validate"
        );
    }

    #[test]
    fn a_prereq_nested_one_level_past_the_depth_limit_is_rejected_cleanly() {
        // Leaf at depth PREREQ_MAX_DEPTH + 1 crosses the limit and must be
        // rejected with a clear, item-naming error — not a stack overflow.
        let rs = ruleset_with_prereq(nested_prereq(PREREQ_MAX_DEPTH));
        let err = rs.validate_integrity().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("virtue.tester") && msg.contains("nests more than"),
            "expected a clear over-depth error naming the item, got: {msg}"
        );
    }

    #[test]
    fn a_pathologically_deep_prereq_is_rejected_cleanly_not_a_stack_overflow() {
        // Far beyond anything a legitimate ruleset (or even a generous
        // future one) could produce — the property under test is that this
        // returns an ordinary Err, not that it aborts the process. Capped at
        // 1,000 (not, say, 50,000): past several tens of thousands the
        // compiler-generated `Drop` for the nested `Vec<Prereq>` chain itself
        // recurses once per level when this function's `rs` goes out of
        // scope, which is a *different* unbounded recursion than the one K8
        // fixes (`Ruleset::validate_prereq_refs`'s own walk, which this test
        // proves stops at depth 33 regardless of how deep the input goes).
        // That Drop recursion is not reachable via the real loading path
        // (`Ruleset::from_sources`/`from_serialized`), because entities only
        // ever get JSON-deserialized `Prereq` trees, and `serde_json` enforces
        // its own recursion limit (default 128) during parsing — this test's
        // whole point is to exercise a tree built directly in Rust, bypassing
        // that limit, precisely to prove `validate_prereq_refs`'s bound is not
        // merely inherited from serde_json.
        let rs = ruleset_with_prereq(nested_prereq(1_000));
        let err = rs.validate_integrity().unwrap_err();
        assert!(err.to_string().contains("virtue.tester"));
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
            aging: None,
        })
        .unwrap();
        let rules = rs.life_stages().expect("life-stage rules loaded");
        assert_eq!(rules.childhood.native_language_xp, 75);
        assert_eq!(rules.later_life.xp_per_year, 15);

        let without = Ruleset::from_json("test", "1", "[]", "[]").unwrap();
        assert!(without.life_stages().is_none());
    }

    /// The aging file is optional too — a ruleset may ship no aging tables, which
    /// stands the whole subsystem down rather than letting the engine invent a
    /// table — and when present its numbers reach the engine.
    #[test]
    fn aging_rules_load_from_their_own_file() {
        let aging = r#"{
          "start_age": 35,
          "age_divisor": 10,
          "apparent_age_increase_min": 3,
          "longevity_clamp": { "max_total": 9, "until_age": 35 },
          "living_conditions": [
            { "id": "living_condition.average_peasant", "modifier": 0 }
          ],
          "outcomes": [
            { "min": 10, "effect": { "type": "any_characteristic", "points": 1 } }
          ]
        }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            aging: Some(aging),
            ..RulesetSources::default()
        })
        .unwrap();
        let rules = rs.aging().expect("aging rules loaded");
        assert_eq!(rules.start_age, 35);
        assert_eq!(rules.age_divisor, 10);
        assert_eq!(rules.apparent_age_increase_min, 3);
        assert_eq!(rules.longevity_clamp.as_ref().map(|c| c.max_total), Some(9));
        assert_eq!(rules.living_conditions.len(), 1);
        assert_eq!(rules.outcomes.len(), 1);
        // The serialized key name is the stable public contract the frontend binds
        // to — pinned here because the golden field-name test's fixture ships no
        // aging file, so the key is skipped there.
        let value = serde_json::to_value(&rs).unwrap();
        assert!(
            value.as_object().expect("object").contains_key("aging"),
            "the aging rules reach the frontend under the 'aging' key"
        );

        // An absent file leaves the whole subsystem stood down, and no empty key
        // reaches the frontend.
        let without = Ruleset::from_json("test", "1", "[]", "[]").unwrap();
        assert!(without.aging().is_none());
        let value = serde_json::to_value(&without).unwrap();
        assert!(
            value.as_object().expect("object").get("aging").is_none(),
            "a ruleset without aging rules must not grow an empty 'aging' key"
        );
    }

    /// The aging file with its scalars and one Living Condition fixed, so the
    /// tiling fixtures below vary the outcome table only. `OUTCOMES` is
    /// substituted per test.
    const AGING_SCALARS: &str = r#"{
      "start_age": 35,
      "age_divisor": 10,
      "apparent_age_increase_min": 3,
      "longevity_clamp": { "max_total": 9, "until_age": 35 },
      "living_conditions": [
        { "id": "living_condition.average_peasant", "modifier": 0 }
      ],
      "outcomes": [ OUTCOMES ]
    }"#;

    /// Loads a ruleset whose aging file is `aging`. The aging tables reference no
    /// other catalogue, so nothing else needs to be present.
    fn aging_ruleset(aging: &str) -> Result<Ruleset, RulesetError> {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            aging: Some(aging),
            ..RulesetSources::default()
        })
    }

    /// Loads a ruleset whose aging outcome table is `outcomes`, against the
    /// scalars of [`AGING_SCALARS`].
    fn aging_outcomes_ruleset(outcomes: &str) -> Result<Ruleset, RulesetError> {
        aging_ruleset(&AGING_SCALARS.replace("OUTCOMES", outcomes))
    }

    /// The Aging Roll table answers *every* total, so its rows must tile the
    /// number line from the first one upwards: a gap would leave a total with no
    /// result at all, an overlap would give it two, and without an open-ended top
    /// row ("22+", Ars Magica - Definitive Edition (Core Rules).md:16611) every high total would fall off the end.
    ///
    /// The check is contiguity, deliberately **not** "must cover 10..=21" — the
    /// shipped table's own numbers are data, and a ruleset that bands its rows
    /// differently is still well-formed.
    #[test]
    fn aging_outcome_rows_must_tile_without_a_gap_or_an_overlap() {
        // The shipped shape, in miniature: contiguous rows and an open-ended top.
        assert!(
            aging_outcomes_ruleset(
                r#"{ "min": 10, "max": 21, "effect": { "type": "any_characteristic", "points": 1 } },
                   { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
            )
            .is_ok()
        );

        // No rows at all: every total would land nowhere.
        let err = aging_outcomes_ruleset("").unwrap_err();
        assert!(
            err.to_string().contains("no outcome rows"),
            "should name the empty table: {err}"
        );

        // A gap: 13 lands on no row.
        let err = aging_outcomes_ruleset(
            r#"{ "min": 10, "max": 12, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 14, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("gap") && msg.contains("12") && msg.contains("14"),
            "should name the rows the gap sits between: {msg}"
        );

        // An overlap: 12, 13 and 14 land on two rows at once.
        let err = aging_outcomes_ruleset(
            r#"{ "min": 10, "max": 14, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 12, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("overlap") && msg.contains("14") && msg.contains("12"),
            "should name the rows that overlap: {msg}"
        );

        // Rows out of order: the table is read top-down, so a `min` that goes
        // backwards is a mis-transcribed table however the bands work out.
        let err = aging_outcomes_ruleset(
            r#"{ "min": 13, "max": 14, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 10, "max": 12, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("ascending") && msg.contains("13") && msg.contains("10"),
            "should name the row that goes backwards: {msg}"
        );

        // No open-ended row: a total of 22 falls off the end of the table.
        let err = aging_outcomes_ruleset(
            r#"{ "min": 10, "max": 12, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 13, "max": 21, "effect": { "type": "any_characteristic", "points": 1 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("open-ended") && msg.contains("21"),
            "should name the row that should have had no upper bound: {msg}"
        );

        // Two open-ended rows: everything from 10 up would take the first one, and
        // the rest of the table would never be reached.
        let err = aging_outcomes_ruleset(
            r#"{ "min": 10, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("open-ended") && msg.contains("10"),
            "should name the row that is open-ended too early: {msg}"
        );

        // A row whose band runs backwards covers nothing at all.
        let err = aging_outcomes_ruleset(
            r#"{ "min": 12, "max": 10, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("12") && msg.contains("10"),
            "should name the inverted band: {msg}"
        );
    }

    /// The Longevity Ritual clamp of `:16575` says what it is *for*: a young
    /// ritual-bearer "is at no risk of actually aging before any other
    /// characters". That holds if and only if the clamped ceiling sits strictly
    /// below the first row that costs Aging Points — so the 9-against-10 the
    /// rulebook ships is a derivable identity between two sentences, not a number
    /// to transcribe and hope for.
    #[test]
    fn the_longevity_clamp_must_sit_below_the_first_aging_point_row() {
        let with_clamp = |max_total: i32| {
            let aging = AGING_SCALARS
                .replace(
                    r#""max_total": 9"#,
                    &format!(r#""max_total": {max_total}"#),
                )
                .replace(
                    "OUTCOMES",
                    r#"{ "min": 10, "max": 21, "effect": { "type": "any_characteristic", "points": 1 } },
                       { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
                );
            aging_ruleset(&aging)
        };

        // The shipped 9-against-10: clamped totals never reach the first row.
        assert!(with_clamp(9).is_ok());

        // A clamp of 10 lands squarely on the first aging-point row, so the ritual
        // would age its bearer exactly as if he had none.
        let err = with_clamp(10).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(":16575"),
            "should cite the sentence the identity comes from: {msg}"
        );
        assert!(
            msg.contains("10"),
            "should name the clamp and the row it fails to clear: {msg}"
        );
    }

    /// The remaining per-file and per-row gates: the apparent-age threshold, the
    /// age divisor, and rows that award nothing or name their Characteristics
    /// badly. Plus the duplicate-id sweep the Living Conditions table joins.
    #[test]
    fn an_apparent_age_threshold_above_the_first_points_row_fails_the_load() {
        // `:16577` gives exactly one exception to "apparent age increases":
        // particularly low rolls. So no row that costs Aging Points may sit below
        // the threshold, or a total could age a character without his looking a day
        // older.
        let err = aging_ruleset(
            &AGING_SCALARS
                .replace(
                    r#""apparent_age_increase_min": 3"#,
                    r#""apparent_age_increase_min": 11"#,
                )
                .replace(
                    "OUTCOMES",
                    r#"{ "min": 10, "effect": { "type": "any_characteristic", "points": 1 } }"#,
                ),
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("11") && msg.contains("10"),
            "should name the threshold and the row it sits above: {msg}"
        );

        // The age term is `age / divisor`, rounded up — a divisor of 0 has no
        // meaning at all.
        let err = aging_ruleset(
            &AGING_SCALARS
                .replace(r#""age_divisor": 10"#, r#""age_divisor": 0"#)
                .replace(
                    "OUTCOMES",
                    r#"{ "min": 10, "effect": { "type": "any_characteristic", "points": 1 } }"#,
                ),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("age_divisor"),
            "should name the divisor: {err}"
        );

        // A row that awards no Aging Points is a no-op that reads as a rule.
        let err = aging_outcomes_ruleset(
            r#"{ "min": 10, "effect": { "type": "any_characteristic", "points": 0 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("10") && msg.contains('0'),
            "should name the row that costs nothing: {msg}"
        );
        let err = aging_outcomes_ruleset(
            r#"{ "min": 10, "effect": { "type": "named_characteristics", "points": 0, "characteristics": ["qik"] } }"#,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("10"),
            "a named-Characteristic row must award something too: {err}"
        );

        // A named-Characteristic row that names none has nowhere to put its points.
        let err = aging_outcomes_ruleset(
            r#"{ "min": 10, "effect": { "type": "named_characteristics", "points": 1, "characteristics": [] } }"#,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("10"),
            "should name the row that names no Characteristic: {err}"
        );

        // "1 Aging Point in Str and Sta" (`:16607`) gives each named Characteristic
        // a point, so naming one twice is a transcription slip that would silently
        // double it.
        let err = aging_outcomes_ruleset(
            r#"{ "min": 10, "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["sta", "sta"] } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("sta") && msg.contains("10"),
            "should name the repeated Characteristic and its row: {msg}"
        );

        // Living Conditions are a catalogue like any other, so their ids join the
        // duplicate sweep.
        let err = aging_ruleset(
            &AGING_SCALARS
                .replace(
                    r#"{ "id": "living_condition.average_peasant", "modifier": 0 }"#,
                    r#"{ "id": "living_condition.average_peasant", "modifier": 0 },
                       { "id": "living_condition.average_peasant", "modifier": 2 }"#,
                )
                .replace(
                    "OUTCOMES",
                    r#"{ "min": 10, "effect": { "type": "any_characteristic", "points": 1 } }"#,
                ),
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("duplicate living condition ID")
                && msg.contains("living_condition.average_peasant"),
            "should name the duplicated condition: {msg}"
        );
    }

    /// A cached ruleset is trusted no further than a freshly parsed one: the aging
    /// gates belong to `validate_integrity`, which [`Ruleset::from_serialized`]
    /// re-runs, so a clamp that no longer clears the first aging-point row is
    /// rejected however the ruleset arrived.
    #[test]
    fn from_serialized_rejects_a_longevity_clamp_that_reaches_the_table() {
        let rs = aging_outcomes_ruleset(
            r#"{ "min": 10, "max": 21, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
        )
        .unwrap();

        let mut serialized = serde_json::to_value(&rs).unwrap();
        serialized["aging"]["longevity_clamp"]["max_total"] = serde_json::Value::from(12);

        let err = Ruleset::from_serialized(&serialized.to_string()).unwrap_err();
        assert_eq!(err.kind(), "integrity");
        assert!(
            err.to_string().contains(":16575"),
            "should cite the sentence the identity comes from: {err}"
        );
    }

    /// Living Conditions are the one duplicate-swept catalogue that can survive a
    /// round trip with its duplicates intact: every other one collapses into a
    /// `BTreeMap` on the way into the [`Ruleset`], where a repeated id simply
    /// cannot exist, while the conditions stay a `Vec` inside [`AgingRules`]. So
    /// the sweep belongs to `validate_aging_rules`, which both load paths run —
    /// otherwise a cached ruleset could carry two rows under one id and the
    /// second would silently shadow the first.
    #[test]
    fn from_serialized_rejects_duplicate_living_condition_ids() {
        let rs = aging_outcomes_ruleset(
            r#"{ "min": 10, "max": 21, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
        )
        .unwrap();

        let mut serialized = serde_json::to_value(&rs).unwrap();
        let conditions = serialized["aging"]["living_conditions"]
            .as_array_mut()
            .expect("the fixture ships a Living Conditions table");
        let repeated = conditions[0].clone();
        conditions.push(repeated);

        let err = Ruleset::from_serialized(&serialized.to_string()).unwrap_err();
        assert_eq!(err.kind(), "integrity");
        let msg = err.to_string();
        assert!(
            msg.contains("duplicate living condition ID")
                && msg.contains("living_condition.average_peasant"),
            "should name the duplicated condition: {msg}"
        );
    }

    /// The aging file with its scalars and a settled outcome table fixed, so the
    /// crisis fixtures below vary the crisis block only. `CRISIS` is substituted
    /// per test.
    const AGING_AROUND_CRISIS: &str = r#"{
      "start_age": 35,
      "age_divisor": 10,
      "apparent_age_increase_min": 3,
      "longevity_clamp": { "max_total": 9, "until_age": 35 },
      "living_conditions": [
        { "id": "living_condition.average_peasant", "modifier": 0 }
      ],
      "outcomes": [
        { "min": 10, "max": 21, "effect": { "type": "any_characteristic", "points": 1 } },
        { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }
      ],
      "crisis": CRISIS
    }"#;

    /// A well-formed Crisis Table in miniature: a bedridden row open below
    /// (Ars Magica - Definitive Edition (Core Rules).md:16626), then two illness rows climbing together, the last of
    /// them open above and offering no Stamina roll at all (`:16632`).
    const CRISIS_ROWS: &str = r#"
      { "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
      { "id": "crisis.minor", "min": 9, "max": 14,
        "outcome": { "type": "illness", "severity": "minor", "ease_factor": 3, "ritual_level": 20 } },
      { "id": "crisis.terminal", "min": 15,
        "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }"#;

    /// Loads a ruleset whose crisis block is `crisis`, against the aging file of
    /// [`AGING_AROUND_CRISIS`].
    fn crisis_ruleset(crisis: &str) -> Result<Ruleset, RulesetError> {
        aging_ruleset(&AGING_AROUND_CRISIS.replace("CRISIS", crisis))
    }

    /// Loads a ruleset whose Crisis Table carries `rows` and neither a die nor an
    /// attendant — the shape most of the gates below vary.
    fn crisis_rows_ruleset(rows: &str) -> Result<Ruleset, RulesetError> {
        crisis_ruleset(&format!(r#"{{ "rows": [ {rows} ] }}"#))
    }

    /// The Crisis Table answers *every* crisis total, so its rows must tile the
    /// number line between two open ends: the row open below ("8 or less",
    /// Ars Magica - Definitive Edition (Core Rules).md:16626) comes first and only it may omit its minimum, the row
    /// open above ("19+", `:16632`) comes last and only it may omit its maximum,
    /// and between them no total may land on two rows or on none.
    ///
    /// The check is contiguity, deliberately **not** "must cover 15..=19": the
    /// shipped table's own bands are data, and a house table that draws them
    /// elsewhere is still well-formed.
    #[test]
    fn crisis_rows_must_tile_ascending_between_two_open_ends() {
        // The shipped shape, in miniature.
        assert!(crisis_rows_ruleset(CRISIS_ROWS).is_ok());

        // No rows at all: every crisis total would land nowhere.
        let err = crisis_rows_ruleset("").unwrap_err();
        assert!(
            err.to_string().contains("no crisis rows"),
            "should name the empty table: {err}"
        );

        // A first row that is not open below: a total of 7 falls off the bottom.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "min": 8, "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.terminal", "min": 9,
                 "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("open below") && msg.contains("crisis.bedridden"),
            "should name the row that should have had no lower bound: {msg}"
        );

        // A second row open below: everything up to 14 takes it, and the first row
        // is never reached.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.minor", "max": 14,
                 "outcome": { "type": "illness", "severity": "minor", "ease_factor": 3, "ritual_level": 20 } },
               { "id": "crisis.terminal", "min": 15,
                 "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("open below") && msg.contains("crisis.minor"),
            "should name the row that is open below too late: {msg}"
        );

        // A last row that is not open above: a total of 20 falls off the top.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.terminal", "min": 9, "max": 19,
                 "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("open above") && msg.contains("crisis.terminal"),
            "should name the row that should have had no upper bound: {msg}"
        );

        // A middle row open above: nothing below it would ever be reached again.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.minor", "min": 9,
                 "outcome": { "type": "illness", "severity": "minor", "ease_factor": 3, "ritual_level": 20 } },
               { "id": "crisis.terminal", "min": 15,
                 "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("open above") && msg.contains("crisis.minor"),
            "should name the row that is open above too early: {msg}"
        );

        // A band that runs backwards covers no total at all.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.minor", "min": 14, "max": 9,
                 "outcome": { "type": "illness", "severity": "minor", "ease_factor": 3, "ritual_level": 20 } },
               { "id": "crisis.terminal", "min": 15,
                 "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("crisis.minor") && msg.contains("14") && msg.contains('9'),
            "should name the inverted band: {msg}"
        );

        // A gap: 9 lands on no row.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.terminal", "min": 10,
                 "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("gap") && msg.contains('8') && msg.contains("10"),
            "should name the rows the gap sits between: {msg}"
        );

        // An overlap: 7 and 8 land on two rows at once.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.terminal", "min": 7,
                 "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("overlap") && msg.contains('8') && msg.contains('7'),
            "should name the rows that overlap: {msg}"
        );
    }

    /// "The level of spell required depends on the severity of the crisis, as noted
    /// on the table." (Ars Magica - Definitive Edition (Core Rules).md:16638) — the illness rows are one ladder, so
    /// severity, required Ritual level and Ease Factor must all climb together
    /// down the table, and the bedridden rows (which have no severity at all) must
    /// sit in front of them.
    #[test]
    fn the_crisis_illness_ladder_must_climb_together() {
        // A bedridden row after an illness row: the table's mildest results would
        // sit above its worst.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.minor", "max": 14,
                 "outcome": { "type": "illness", "severity": "minor", "ease_factor": 3, "ritual_level": 20 } },
               { "id": "crisis.bedridden", "min": 15, "outcome": { "type": "bedridden" } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("crisis.bedridden") && msg.contains("crisis.minor"),
            "should name the bedridden row and the illness it follows: {msg}"
        );

        // Severity going backwards down the table.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.major", "min": 9, "max": 14,
                 "outcome": { "type": "illness", "severity": "major", "ease_factor": 3, "ritual_level": 20 } },
               { "id": "crisis.minor", "min": 15,
                 "outcome": { "type": "illness", "severity": "minor", "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("severity")
                && msg.contains("crisis.major")
                && msg.contains("crisis.minor"),
            "should name the two illness rows whose severities go backwards: {msg}"
        );
        assert!(
            msg.contains(":16638"),
            "should cite the sentence that makes severity a ladder: {msg}"
        );

        // The required Ritual level standing still: two severities would ask for
        // the same spell, so the level would no longer depend on the severity.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.minor", "min": 9, "max": 14,
                 "outcome": { "type": "illness", "severity": "minor", "ease_factor": 3, "ritual_level": 20 } },
               { "id": "crisis.terminal", "min": 15,
                 "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 20 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("crisis.minor") && msg.contains("crisis.terminal") && msg.contains("20"),
            "should name the two rows requiring the same Ritual level: {msg}"
        );

        // The Ease Factor going backwards: a worse illness would be easier to
        // survive.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.minor", "min": 9, "max": 14,
                 "outcome": { "type": "illness", "severity": "minor", "ease_factor": 6, "ritual_level": 20 } },
               { "id": "crisis.terminal", "min": 15,
                 "outcome": { "type": "illness", "severity": "terminal", "ease_factor": 3, "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Ease Factor")
                && msg.contains("crisis.minor")
                && msg.contains("crisis.terminal"),
            "should name the two rows whose Ease Factors go backwards: {msg}"
        );

        // Only the most severe illness may forgo the Stamina roll (`:16632`); a
        // milder row with no Ease Factor would be unsurvivable without magic while
        // a worse one was not.
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.minor", "min": 9, "max": 14,
                 "outcome": { "type": "illness", "severity": "minor", "ritual_level": 20 } },
               { "id": "crisis.terminal", "min": 15,
                 "outcome": { "type": "illness", "severity": "terminal", "ease_factor": 12, "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Ease Factor") && msg.contains("crisis.minor"),
            "should name the milder row that offers no Stamina roll: {msg}"
        );
    }

    /// Two rows under one id would let the second silently shadow the first, so
    /// the Crisis Table joins the duplicate sweep like every other catalogue — and
    /// from `validate_crisis_rules`, because the rows stay a `Vec` and a duplicate
    /// therefore survives a round trip through [`Ruleset::from_serialized`].
    #[test]
    fn duplicate_crisis_row_ids_fail_the_load() {
        let err = crisis_rows_ruleset(
            r#"{ "id": "crisis.bedridden", "max": 8, "outcome": { "type": "bedridden" } },
               { "id": "crisis.bedridden", "min": 9,
                 "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("duplicate crisis row ID") && msg.contains("crisis.bedridden"),
            "should name the duplicated row: {msg}"
        );
    }

    /// The attending doctor's "Int + Medicine roll" (Ars Magica - Definitive Edition (Core Rules).md:16634) names an
    /// Ability by id, so a typo there is a referential-integrity failure like every
    /// other ref in the rules data — the survival read-out would otherwise quietly
    /// find no Medicine score to add.
    #[test]
    fn a_crisis_attendants_ability_must_resolve() {
        let with_attendant = |ability: &str| {
            let crisis = format!(
                r#"{{ "rows": [ {CRISIS_ROWS} ],
                      "attendant": {{ "ability": "{ability}", "characteristic": "int",
                                      "ease_factor": 6, "botch_penalty": -3 }} }}"#
            );
            Ruleset::from_sources(RulesetSources {
                id: "test",
                version: "1",
                point_items: "[]",
                type_profiles: "[]",
                abilities: Some(LIFE_STAGE_ABILITIES),
                aging: Some(&AGING_AROUND_CRISIS.replace("CRISIS", &crisis)),
                ..RulesetSources::default()
            })
        };

        assert!(with_attendant("ability.awareness").is_ok());

        let err = with_attendant("ability.nonesuch").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("attendant") && msg.contains("ability.nonesuch"),
            "should name the unknown ability: {msg}"
        );
    }

    /// "Characters with a Decrepitude score of 4 are extremely frail … Characters
    /// with a Decrepitude score of 5 are bedridden and will die" (`:16617`) — two
    /// thresholds on one ascending track, so the frail one must be reached first.
    /// Transposed, a character would be dead before he ever turned frail.
    #[test]
    fn the_frail_decrepitude_score_must_sit_below_the_fatal_one() {
        let with_scores = |frail: u8, fatal: u8| {
            crisis_ruleset(&format!(
                r#"{{ "rows": [ {CRISIS_ROWS} ] }},
                   "frail_decrepitude_score": {frail},
                   "fatal_decrepitude_score": {fatal}"#
            ))
        };

        // The shipped 4-before-5.
        assert!(with_scores(4, 5).is_ok());

        let err = with_scores(5, 4).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("frail_decrepitude_score")
                && msg.contains("fatal_decrepitude_score")
                && msg.contains('5')
                && msg.contains('4'),
            "should name both thresholds and their values: {msg}"
        );

        // Equal is the same defect: a character would turn frail and die at once.
        assert!(with_scores(5, 5).is_err());
    }

    /// An absent `crisis` block stands the crisis subsystem down and is not an
    /// error — the same house position [`Ruleset::aging`] states for the aging
    /// block as a whole. An aging file written before the Crisis Table existed
    /// must keep loading exactly as it did.
    #[test]
    fn an_aging_block_with_no_crisis_key_loads_cleanly() {
        let rs = aging_outcomes_ruleset(
            r#"{ "min": 10, "max": 21, "effect": { "type": "any_characteristic", "points": 1 } },
               { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }"#,
        )
        .expect("an aging block with no crisis key loads");
        assert!(rs.aging().expect("the aging block").crisis.is_none());
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
            aging: None,
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

    /// The childhood half of a life-stage file, so the apprenticeship fixtures below
    /// vary one block only. `{APPRENTICESHIP}` is substituted per test.
    const APPRENTICESHIP_LIFE_STAGES: &str = r#"{
      "apprenticeship": { APPRENTICESHIP },
      "childhood": {
        "years": 5,
        "native_language_ability": "ability.living_language",
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": ["ability.awareness"]
      },
      "later_life": { "xp_per_year": 15 }
    }"#;

    /// Loads a ruleset whose apprenticeship block is `apprenticeship`, against the
    /// life-stage abilities fixture.
    fn apprenticeship_ruleset(apprenticeship: &str) -> Result<Ruleset, RulesetError> {
        let life_stages = APPRENTICESHIP_LIFE_STAGES.replace("APPRENTICESHIP", apprenticeship);
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            abilities: Some(LIFE_STAGE_ABILITIES),
            life_stages: Some(&life_stages),
            ..RulesetSources::default()
        })
    }

    /// The apprenticeship block names Abilities ("Parma Magica 1, Magic Theory 1,
    /// Latin 1", Ars Magica - Definitive Edition (Core Rules).md:2437), so a typo there would silently drop a
    /// requirement no magus is then held to — a load-time referential failure like
    /// every other ref in the rules data.
    #[test]
    fn an_apprenticeship_ability_requirement_must_resolve() {
        let err = apprenticeship_ruleset(
            r#""years": 15, "xp": 240, "recommended_xp": 0,
               "minimum_abilities": [{ "ability": "ability.nonesuch", "min_score": 1 }],
               "recommended_abilities": []"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("ability.nonesuch"),
            "should name the unresolved apprenticeship ability: {msg}"
        );

        // The recommended list is checked the same way — it is the same shape.
        let err = apprenticeship_ruleset(
            r#""years": 15, "xp": 240, "recommended_xp": 0,
               "minimum_abilities": [],
               "recommended_abilities": [{ "ability": "ability.absent", "min_score": 1 }]"#,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("ability.absent"),
            "should name the unresolved recommended ability: {err}"
        );
    }

    /// A requirement may narrow itself to one instance of a parameterized Ability,
    /// so naming a parameter for an Ability that takes none could never match — a
    /// broken file rather than a requirement nobody meets.
    #[test]
    fn an_apprenticeship_requirement_parameter_needs_a_parameterized_ability() {
        let err = apprenticeship_ruleset(
            r#""years": 15, "xp": 240, "recommended_xp": 0,
               "minimum_abilities": [
                 { "ability": "ability.awareness", "min_score": 1, "parameter": "Latin" }
               ],
               "recommended_abilities": []"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("ability.awareness") && msg.contains("parameter"),
            "should explain why the requirement can never name an instance: {msg}"
        );

        // The parameterized ability takes one happily.
        assert!(
            apprenticeship_ruleset(
                r#""years": 15, "xp": 240, "recommended_xp": 0,
                   "minimum_abilities": [
                     { "ability": "ability.living_language", "min_score": 1, "parameter": "German" }
                   ],
                   "recommended_abilities": []"#,
            )
            .is_ok()
        );
    }

    /// The recommended Abilities carry their own total — "Total Cost: 90 experience
    /// points" (Ars Magica - Definitive Edition (Core Rules).md:2461) — so the list and the total must agree off the
    /// advancement table. This is the **trust gate on transcribed rulebook data**,
    /// the same one `validate_childhood_packages` applies to childhood's 45 and 75: a
    /// mistyped score fails the load instead of shipping a recommendation that costs
    /// something the rulebook never says.
    ///
    /// The *minimum* set is deliberately not priced — `:2437` states no total, so
    /// such a check could only compare the engine to itself.
    #[test]
    fn recommended_apprenticeship_abilities_must_price_to_their_total() {
        // `ability.awareness 1` costs 5 off the fixture's table, not 80.
        let err = apprenticeship_ruleset(
            r#""years": 15, "xp": 240, "recommended_xp": 80,
               "minimum_abilities": [],
               "recommended_abilities": [{ "ability": "ability.awareness", "min_score": 1 }]"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("apprenticeship") && msg.contains("80") && msg.contains('5'),
            "should name the block and both totals: {msg}"
        );

        // The honest total loads.
        assert!(
            apprenticeship_ruleset(
                r#""years": 15, "xp": 240, "recommended_xp": 5,
                   "minimum_abilities": [],
                   "recommended_abilities": [{ "ability": "ability.awareness", "min_score": 1 }]"#,
            )
            .is_ok()
        );

        // A score the table cannot price is reported as itself; the total then stays
        // silent rather than blaming a sum that could not be computed.
        let err = apprenticeship_ruleset(
            r#""years": 15, "xp": 240, "recommended_xp": 5,
               "minimum_abilities": [],
               "recommended_abilities": [{ "ability": "ability.awareness", "min_score": 4 }]"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("ability.awareness") && msg.contains('4'),
            "should name the unpriceable score: {msg}"
        );
        assert!(
            !msg.contains("recommended abilities price to"),
            "an unpriceable score must not also produce a total mismatch: {msg}"
        );
    }

    /// A ruleset that declares Hermetic magi and ships life-stage rules must declare
    /// the apprenticeship block, because a magus's later life runs only *until*
    /// apprenticeship (Ars Magica - Definitive Edition (Core Rules).md:2214, :2364). Without the block the engine would
    /// cost a magus exactly as it costs a companion — every year to its age, funding
    /// Arts out of a child's experience — so this is an engine invariant enforced
    /// where the limit actually lives: in the data, at load, not on each character.
    ///
    /// Gated the same way as [`Self::validate_engine_required_roles`]: only a ruleset
    /// declaring an `is_magus` profile is held to it, and only when it ships life
    /// stages at all.
    #[test]
    fn a_magus_ruleset_shipping_life_stages_must_declare_an_apprenticeship() {
        const MAGUS_TYPES: &str = r#"[
          { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"], "is_magus": true, "creation_phases": [] }
        ]"#;
        // The five abilities any magus ruleset shipping a catalogue must carry, plus
        // the parameterized language the childhood block names.
        const MAGUS_ABILITIES: &str = r#"{
          "advancement": [ { "score": 1, "total_xp": 5 } ],
          "abilities": [
            { "id": "ability.artes_liberales", "category": "academic" },
            { "id": "ability.awareness", "category": "general" },
            { "id": "ability.living_language", "category": "general", "parameter": "language" },
            { "id": "ability.magic_theory", "category": "arcane" },
            { "id": "ability.parma_magica", "category": "arcane" },
            { "id": "ability.penetration", "category": "arcane" },
            { "id": "ability.philosophiae", "category": "academic" }
          ]
        }"#;
        /// Everything a magus ruleset needs except the apprenticeship itself — the
        /// years after the Gauntlet are present so only one block is missing.
        const WITHOUT_APPRENTICESHIP: &str = r#"{
          "childhood": {
            "years": 5,
            "native_language_ability": "ability.living_language",
            "native_language_xp": 75,
            "spread_xp": 45,
            "spread_abilities": ["ability.awareness"]
          },
          "later_life": { "xp_per_year": 15 },
          "post_apprenticeship": { "lab_season_cost": 10,
                                   "max_charged_lab_seasons_per_year": 3,
                                   "points_per_year": 30 }
        }"#;
        let load = |types: &str, life_stages: Option<&str>| {
            Ruleset::from_sources(RulesetSources {
                id: "test",
                version: "1",
                point_items: "[]",
                type_profiles: types,
                abilities: Some(MAGUS_ABILITIES),
                life_stages,
                ..RulesetSources::default()
            })
        };

        let err = load(MAGUS_TYPES, Some(WITHOUT_APPRENTICESHIP)).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("apprenticeship"),
            "should name the missing block: {msg}"
        );

        // Declared — even with no requirements of its own — and it loads.
        let with_block = WITHOUT_APPRENTICESHIP.replace(
            r#"{
          "childhood""#,
            r#"{
          "apprenticeship": { "years": 15, "xp": 240, "recommended_xp": 0,
                              "minimum_abilities": [], "recommended_abilities": [] },
          "childhood""#,
        );
        assert!(load(MAGUS_TYPES, Some(&with_block)).is_ok());

        // A ruleset with no magus profile needs neither block, and a magus ruleset
        // shipping no life stages at all is out of scope of the rule.
        const COMPANION_TYPES: &str = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"], "creation_phases": [] }
        ]"#;
        assert!(load(COMPANION_TYPES, Some(WITHOUT_APPRENTICESHIP)).is_ok());
        assert!(load(MAGUS_TYPES, None).is_ok());
    }

    /// The childhood half of a life-stage file once more, so the
    /// post-apprenticeship fixtures below vary one block only. `POST` is
    /// substituted per test.
    const POST_APPRENTICESHIP_LIFE_STAGES: &str = r#"{
      "childhood": {
        "years": 5,
        "native_language_ability": "ability.living_language",
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": ["ability.awareness"]
      },
      "later_life": { "xp_per_year": 15 },
      "post_apprenticeship": { POST }
    }"#;

    /// Loads a ruleset whose post-apprenticeship block is `post`, against the
    /// life-stage abilities fixture. No magus profile, so the block is under test
    /// on its own terms rather than because something demanded it.
    fn post_apprenticeship_ruleset(post: &str) -> Result<Ruleset, RulesetError> {
        let life_stages = POST_APPRENTICESHIP_LIFE_STAGES.replace("POST", post);
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            abilities: Some(LIFE_STAGE_ABILITIES),
            life_stages: Some(&life_stages),
            ..RulesetSources::default()
        })
    }

    /// A season of lab work costs a magus "10 points from the yearly 30 experience
    /// points" (Ars Magica - Definitive Edition (Core Rules).md:2482), so a cost of nothing is a broken file: every
    /// season would be free and the whole passage would stop applying.
    #[test]
    fn a_post_apprenticeship_lab_season_must_cost_something() {
        // A block that is internally consistent (0 × 3 = 0) and still wrong.
        let err = post_apprenticeship_ruleset(
            r#""lab_season_cost": 0, "max_charged_lab_seasons_per_year": 3,
               "points_per_year": 0"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("lab_season_cost"),
            "should name the free lab season: {msg}"
        );

        assert!(
            post_apprenticeship_ruleset(
                r#""lab_season_cost": 10, "max_charged_lab_seasons_per_year": 3,
                   "points_per_year": 30"#,
            )
            .is_ok()
        );
    }

    /// The charged seasons must exhaust the year exactly. This is not a tidiness
    /// check, it **is** `:2482`: the deduction runs "to a minimum of 0 if three or
    /// four seasons are spent on lab work", so three seasons at 10 have to cancel
    /// the yearly 30 — no more, no less. The trust gate on three hand-transcribed
    /// numbers, the same idiom as re-pricing the apprenticeship's `recommended_xp`.
    #[test]
    fn post_apprenticeship_lab_seasons_must_exhaust_the_year_exactly() {
        // 10 × 4 = 40 overshoots: the fourth season would take points the year
        // never granted.
        let err = post_apprenticeship_ruleset(
            r#""lab_season_cost": 10, "max_charged_lab_seasons_per_year": 4,
               "points_per_year": 30"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("40") && msg.contains("30"),
            "should name what the seasons cost and what the year grants: {msg}"
        );

        // 10 × 3 = 30 falls short of a 35-point year: the minimum of 0 is never
        // reached, so a magus in the lab all year still earns 5.
        let err = post_apprenticeship_ruleset(
            r#""lab_season_cost": 10, "max_charged_lab_seasons_per_year": 3,
               "points_per_year": 35"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("30") && msg.contains("35"),
            "should name both totals: {msg}"
        );
    }

    /// A ruleset that declares Hermetic magi and ships life-stage rules must declare
    /// the post-apprenticeship block, exactly as it must declare the apprenticeship
    /// one. Its stored Gauntlet age already ends the magus's later life
    /// (Ars Magica - Definitive Edition (Core Rules).md:2364); without this block the years after it would grant
    /// nothing back, so the profile would simply lose them.
    ///
    /// Gated like [`Ruleset::validate_apprenticeship_refs`]: only an `is_magus`
    /// ruleset that ships life stages at all is held to it.
    #[test]
    fn a_magus_ruleset_shipping_life_stages_must_declare_a_post_apprenticeship() {
        const MAGUS_TYPES: &str = r#"[
          { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"], "is_magus": true, "creation_phases": [] }
        ]"#;
        const COMPANION_TYPES: &str = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"], "creation_phases": [] }
        ]"#;
        const MAGUS_ABILITIES: &str = r#"{
          "advancement": [ { "score": 1, "total_xp": 5 } ],
          "abilities": [
            { "id": "ability.artes_liberales", "category": "academic" },
            { "id": "ability.awareness", "category": "general" },
            { "id": "ability.living_language", "category": "general", "parameter": "language" },
            { "id": "ability.magic_theory", "category": "arcane" },
            { "id": "ability.parma_magica", "category": "arcane" },
            { "id": "ability.penetration", "category": "arcane" },
            { "id": "ability.philosophiae", "category": "academic" }
          ]
        }"#;
        /// Everything a magus ruleset needs except the years after the Gauntlet.
        const WITHOUT_POST: &str = r#"{
          "apprenticeship": { "years": 15, "xp": 240, "recommended_xp": 0,
                              "minimum_abilities": [], "recommended_abilities": [] },
          "childhood": {
            "years": 5,
            "native_language_ability": "ability.living_language",
            "native_language_xp": 75,
            "spread_xp": 45,
            "spread_abilities": ["ability.awareness"]
          },
          "later_life": { "xp_per_year": 15 }
        }"#;
        let load = |types: &str, life_stages: Option<&str>| {
            Ruleset::from_sources(RulesetSources {
                id: "test",
                version: "1",
                point_items: "[]",
                type_profiles: types,
                abilities: Some(MAGUS_ABILITIES),
                life_stages,
                ..RulesetSources::default()
            })
        };

        let err = load(MAGUS_TYPES, Some(WITHOUT_POST)).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("post-apprenticeship"),
            "should name the missing block: {msg}"
        );

        // Declared, and it loads.
        let with_post = WITHOUT_POST.replace(
            r#""later_life": { "xp_per_year": 15 }"#,
            r#""later_life": { "xp_per_year": 15 },
               "post_apprenticeship": { "lab_season_cost": 10,
                                        "max_charged_lab_seasons_per_year": 3,
                                        "points_per_year": 30 }"#,
        );
        assert!(load(MAGUS_TYPES, Some(&with_post)).is_ok());

        // A ruleset with no magus profile needs the block no more than it needs the
        // apprenticeship one, and a magus ruleset shipping no life stages at all is
        // out of the rule's scope.
        assert!(load(COMPANION_TYPES, Some(WITHOUT_POST)).is_ok());
        assert!(load(MAGUS_TYPES, None).is_ok());
    }

    /// A cached ruleset is trusted no further than a freshly parsed one: the
    /// post-apprenticeship checks belong to `validate_integrity`, which
    /// [`Ruleset::from_serialized`] re-runs, so a block whose lab seasons no longer
    /// exhaust the year is rejected however the ruleset arrived.
    #[test]
    fn from_serialized_rejects_post_apprenticeship_seasons_that_miss_the_year() {
        let rs = post_apprenticeship_ruleset(
            r#""lab_season_cost": 10, "max_charged_lab_seasons_per_year": 3,
               "points_per_year": 30"#,
        )
        .unwrap();

        let mut serialized = serde_json::to_value(&rs).unwrap();
        serialized["life_stages"]["post_apprenticeship"]["points_per_year"] =
            serde_json::Value::from(45);

        let err = Ruleset::from_serialized(&serialized.to_string()).unwrap_err();
        assert_eq!(err.kind(), "integrity");
        assert!(
            err.to_string().contains("45"),
            "should name the year the seasons no longer exhaust: {err}"
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
    /// Ars Magica - Definitive Edition (Core Rules).md:2378 in miniature (one parameterized `(Area) Lore`, one
    /// parameterized `(Living Language)`, three plain ones), plus a known ability
    /// the spread may NOT buy, and the canonical "ABILITY To Buy" prices for
    /// scores 1-5 (Ars Magica - Definitive Edition (Core Rules).md:2406-2427).
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
    /// language, 45 across the spread list (Ars Magica - Definitive Edition (Core Rules).md:2378).
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

    /// The spread is a closed list (Ars Magica - Definitive Edition (Core Rules).md:2378), so a package naming an
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
            aging: None,
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
                // `aging` is absent here for the same reason `life_stages` is:
                // this fixture ships no aging file and the field is skipped when
                // `None`. The aging-bearing shape is asserted by
                // `aging_rules_load_from_their_own_file`.
                "armor",
                "art_advancement",
                "art_type_order",
                "arts",
                "aura_modifier_max",
                "aura_modifier_min",
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
                "ritual_min_level",
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
        // Derived rule constant (VA2): the Ritual spell-level floor, mirrored from
        // `spell::RITUAL_MIN_LEVEL` so the UI never re-hardcodes it.
        assert_eq!(obj["ritual_min_level"], 20);
        // Derived rule constants (round 3, Task 3): the aura modifier's legal
        // range, mirrored from `types::AURA_MODIFIER_MIN`/`MAX` so the aura
        // number inputs (DerivedTotalsPanel, MagicPossessions) read the bound
        // from the engine instead of re-hardcoding -50/10.
        assert_eq!(obj["aura_modifier_min"], -50);
        assert_eq!(obj["aura_modifier_max"], 10);

        // And the whole thing round-trips back through the validating loader.
        let restored = Ruleset::from_serialized(&serde_json::to_string(&rs).unwrap()).unwrap();
        assert_eq!(rs, restored);
    }

    /// V46 characterization test: pins that the six engine-derived fields
    /// (`magnitude_points`, `ability_category_order`, `art_type_order`,
    /// `ritual_min_level`, `aura_modifier_min`, `aura_modifier_max`) come out
    /// byte-for-byte identical whichever of the two construction paths
    /// produced the `Ruleset` — fresh-parsed via `assemble_ruleset`
    /// (`Ruleset::from_sources`) or reconstructed via
    /// `Ruleset::from_serialized`. Written before the V46 dedup so it pins
    /// current (pre-refactor) behaviour; it must stay green once both paths
    /// route through one shared derivation instead of each carrying its own
    /// copy of the six assignments.
    #[test]
    fn from_sources_and_from_serialized_derive_identical_engine_fields() {
        let fresh = Ruleset::from_sources(RulesetSources {
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
            aging: None,
        })
        .unwrap();

        let restored = Ruleset::from_serialized(&serde_json::to_string(&fresh).unwrap()).unwrap();

        assert_eq!(fresh.magnitude_points, restored.magnitude_points);
        assert_eq!(
            fresh.ability_category_order,
            restored.ability_category_order
        );
        assert_eq!(fresh.art_type_order, restored.art_type_order);
        assert_eq!(fresh.ritual_min_level, restored.ritual_min_level);
        assert_eq!(fresh.aura_modifier_min, restored.aura_modifier_min);
        assert_eq!(fresh.aura_modifier_max, restored.aura_modifier_max);
        // Not a vacuous "0 == 0"/"[] == []" pass: each is genuinely non-empty.
        assert!(!fresh.magnitude_points.is_empty());
        assert!(!fresh.ability_category_order.is_empty());
        assert!(!fresh.art_type_order.is_empty());
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
            aging: None,
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
