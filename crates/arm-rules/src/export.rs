//! Markdown export of a finished entity (M5.6).
//!
//! A **pure formatter**: `(&Entity, &LocalizedRuleset, &labels) → String`. It
//! implements no rules mechanic — it selects values the engine already computes
//! ([`crate::derived`], [`crate::effective`], [`crate::validation`]) and lays them
//! out as a GitHub-Flavored-Markdown document. Nothing is mutated, nothing is
//! persisted, and no file is touched: the caller owns IO.
//!
//! # The localization split
//!
//! Two different kinds of text meet in the document, and they come from two
//! different places:
//!
//! - **Item names** (Virtues, Abilities, Arts, spells, Houses, equipment) come from
//!   the rules i18n via [`LocalizedRuleset::display_name`].
//! - **Document chrome** (section headings, field labels, column headers, yes/no
//!   markers) is **passed in** by the caller as a `key → text` map keyed by Fluent
//!   message name, so the engine hardcodes no user-facing string.
//!
//! Per CLAUDE.md's "never render a raw ID or enum value as a user-facing label",
//! **neither** kind of text is allowed to reach the document unresolved:
//! [`character_markdown`] returns [`ExportError`] instead, naming every offending
//! chrome key and catalogue id it found in one pass — not just the first — the
//! same "list every offending id, not just the first" contract
//! [`Ruleset::from_sources`](crate::ruleset::Ruleset::from_sources) already keeps
//! for referential integrity, so a stale locale or a foreign save file is fixed in
//! one round trip rather than one failure at a time.
//!
//! A **parameter value**, in contrast, is deliberately exempt: it is free text the
//! majority of the time (Area Lore's region, a Magical Focus's field), and the same
//! slot may instead carry a nested catalogue reference — the formatter cannot tell
//! which without an unresolvable id being the normal case, not a defect, so an
//! unresolved parameter value passes through verbatim (see [`Doc::param_value`]).
//!
//! [`LABEL_KEYS`] declares every chrome key the formatter can ask for, so the
//! caller can assemble the map without reading this source. Only **argument-free**
//! Fluent keys are usable: the engine links no Fluent formatter, so a key whose
//! message interpolates `{ $arg }` can never be resolved here.
//!
//! # Entity kinds
//!
//! The formatter is entity-generic. Every section is omitted when its collection or
//! value is empty, so a **covenant** — which has selections (Boons/Hooks) and
//! free-text identity but no Characteristics, Abilities, Arts, spells, equipment or
//! magic — renders exactly the sections it has, with the character-only ones absent.
//! No section is gated on [`crate::types::EntityKind`]; emptiness does the work.
//!
//! # Determinism
//!
//! Output is byte-deterministic: every collection is iterated in `BTreeMap` order,
//! in the entity's own normalized `Vec` order, or in a fixed rules taxonomy's order
//! ([`Characteristic::ALL`], [`ArtType::ALL`]). Two consecutive renders of the same
//! entity are byte-identical.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::art::ArtType;
use crate::characteristics::Characteristic;
use crate::derived::{CombatLine, combat_totals, encumbrance, fatigue_levels, soak, wound_ranges};
use crate::effective::{
    RestrictedXpPool, XpPoolOrigin, ability_score_floors, confidence, decrepitude_score,
    effective_ability_score, effective_art_score, effective_characteristic_after_aging,
    effective_might, effective_spell_mastery, entity_grants, resolved_spell_level, warping,
    xp_allocation,
};
use crate::ruleset::{LocalizedRuleset, Ruleset};
use crate::types::{
    AgingLogEntry, EnchantedDevice, Entity, EntityKind, Id, ItemKind, MightScore, PersonalityTrait,
    Selection, SpellSelection, SupernaturalPower, TalismanEffect,
};
use crate::validation::{compute_balance, effective_point_ceilings};

mod magic;
mod resolve;
mod sections;

/// Every document-chrome label key [`character_markdown`] can ask for, sorted and
/// duplicate-free. The caller resolves these against its own Fluent bundle and
/// hands the results in; a key absent from the map prints as the key itself.
///
/// The list is closed over the keys the formatter *names* and over every key it
/// composes from a fixed Rust taxonomy (`characteristic-<slug>` and friends), so a
/// new taxonomy variant cannot silently surface as a raw slug — a unit test walks
/// each family and asserts membership here.
///
/// Three key *families* are deliberately **not** enumerated, because each is composed
/// from catalogue *data* and listing them would bake the catalogue's size into code:
/// `type-<profile id>` (the character-type label in the subtitle),
/// `param-label-<parameter key>` (the slot label shown for an unfilled parameter), and
/// `category-<item category>` (the Type cell of a Virtue/Flaw row). The locales
/// already ship one key per shipped profile, parameter key and item category.
/// Individual members the formatter names outright are still listed — hence
/// `param-label-ability`, the Combat table's Ability column header.
pub const LABEL_KEYS: &[&str] = &[
    "abilities-title",
    "ability-category-academic",
    "ability-category-arcane",
    "ability-category-general",
    "ability-category-martial",
    "ability-category-supernatural",
    "ability-score-label",
    "ability-specialty-label",
    "age-label",
    "aging-die-label",
    "aging-label",
    "aging-log-crisis-unrolled",
    "aging-log-heading",
    "aging-points-heading",
    "apparent-age-label",
    "art-type-form",
    "art-type-technique",
    "aura-label",
    "characteristic-com",
    "characteristic-description-label",
    "characteristic-dex",
    "characteristic-int",
    "characteristic-per",
    "characteristic-pre",
    "characteristic-qik",
    "characteristic-sta",
    "characteristic-str",
    "characteristics-title",
    "confidence-label",
    "crisis-die-label",
    "crisis-label",
    "crisis-severity-critical",
    "crisis-severity-major",
    "crisis-severity-minor",
    "crisis-severity-serious",
    "crisis-severity-terminal",
    "crisis-total-label",
    "decrepitude-effect-label",
    "decrepitude-label",
    "derived-addend-armor",
    "derived-addend-bronze_cord",
    "derived-addend-form_bonus",
    "derived-addend-soak_mod",
    "derived-addend-stamina",
    "derived-burden",
    "derived-combat-attack",
    "derived-combat-damage",
    "derived-combat-defense",
    "derived-combat-init",
    "derived-combat-shield-joiner",
    "derived-fatigue-dazed",
    "derived-fatigue-fresh",
    "derived-fatigue-tired",
    "derived-fatigue-weary",
    "derived-fatigue-winded",
    "derived-load",
    "derived-range",
    "derived-section-combat",
    "derived-section-encumbrance",
    "derived-section-fatigue",
    "derived-section-soak",
    "derived-section-wounds",
    "derived-wound-dead",
    "derived-wound-heavy",
    "derived-wound-incapacitating",
    "derived-wound-light",
    "derived-wound-medium",
    "device-level-label",
    "equipment-equipped-label",
    "equipment-group-armor",
    "equipment-group-shields",
    "equipment-group-weapons",
    "export-col-effective",
    "export-col-magnitude",
    "export-col-penalty",
    "export-col-points",
    "export-col-spell-code",
    "export-col-total",
    "export-col-type",
    "export-granted",
    "export-items-boons",
    "export-items-hooks",
    "export-no",
    "export-untitled",
    "export-xp-restricted",
    "export-yes",
    "familiar-animal-label",
    "familiar-cord-bronze",
    "familiar-cord-gold",
    "familiar-cord-silver",
    "familiar-label",
    "familiar-might-label",
    "familiar-powers-label",
    "familiar-size-label",
    "house-label",
    "identity-birth-year",
    "identity-concept",
    "identity-covenant",
    "identity-description",
    "identity-gender",
    "identity-label",
    "identity-name",
    "identity-parens",
    "identity-sigil",
    "items-flaws-title",
    "items-virtues-title",
    "living-conditions-label",
    "longevity-bonus-label",
    "longevity-focus-label",
    "longevity-label",
    "longevity-not-entered",
    "longevity-source-external",
    "longevity-source-label",
    "longevity-source-self_made",
    "magnitude-free",
    "magnitude-major",
    "magnitude-minor",
    "param-label-ability",
    "personality-label",
    "possessions-devices-label",
    "power-level-label",
    "realm-divine",
    "realm-faerie",
    "realm-infernal",
    "realm-magic",
    "reputation-type-academic",
    "reputation-type-ecclesiastical",
    "reputation-type-hermetic",
    "reputation-type-local",
    "reputations-label",
    "restricted-xp-list-separator",
    "spell-level-general",
    "spell-mastery-abilities-label",
    "spell-mastery-label",
    "supernatural-might-label",
    "supernatural-powers-label",
    "tab-arts",
    "tab-equipment",
    "tab-possessions",
    "tab-spells",
    "tab-supernatural",
    "tab-virtues-flaws",
    "talisman-attunements-label",
    "talisman-bonus-label",
    "talisman-description-label",
    "talisman-effect-level-label",
    "talisman-effects-label",
    "talisman-label",
    "twilight-scars-label",
    "warping-effect-label",
    "warping-label",
    "warping-points-label",
    "xp-pool",
    "xp-pool-childhood_native_language",
    "xp-pool-childhood_spread",
    "xp-pool-later_life",
];

/// A document-chrome key or catalogue id [`character_markdown`] could not resolve
/// to display text — see the module docs' "localization split" for why each kind
/// is an error rather than a degraded render.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MissingLabel {
    /// A [`LABEL_KEYS`] chrome key absent from the caller's label map.
    Key(String),
    /// A catalogue id ([`crate::types::Id`], stringified) with no entry in the
    /// loaded ruleset's i18n.
    CatalogueId(String),
}

impl fmt::Display for MissingLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MissingLabel::Key(key) => write!(f, "chrome key '{key}'"),
            MissingLabel::CatalogueId(id) => write!(f, "catalogue id '{id}'"),
        }
    }
}

/// Returned by [`character_markdown`] when the document would otherwise have
/// rendered a raw Fluent key or catalogue slug as user-facing text. Carries every
/// offense found while rendering the whole document, sorted and deduplicated, so
/// one failed export names everything that needs fixing rather than one key at a
/// time.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExportError {
    pub missing: Vec<MissingLabel>,
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "export could not resolve: ")?;
        for (i, item) in self.missing.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{item}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ExportError {}

/// Renders `entity` as a Markdown document, or [`ExportError`] naming every
/// document-chrome key or catalogue id the document could not resolve.
///
/// `ruleset` supplies localized **item names**; `labels` supplies localized
/// **document chrome** keyed by the names in [`LABEL_KEYS`]. See the module docs for
/// the split and for the emptiness rule that governs which sections appear.
pub fn character_markdown(
    entity: &Entity,
    ruleset: &LocalizedRuleset,
    labels: &BTreeMap<String, String>,
) -> Result<String, ExportError> {
    let doc = Doc {
        entity,
        ruleset,
        labels,
        missing: RefCell::new(BTreeSet::new()),
    };
    let mut out = String::new();
    doc.write_title(&mut out);
    doc.write_identity(&mut out);
    doc.write_characteristics(&mut out);
    doc.write_virtues_flaws(&mut out);
    doc.write_abilities(&mut out);
    doc.write_arts(&mut out);
    doc.write_spells(&mut out);
    doc.write_equipment(&mut out);
    doc.write_combat(&mut out);
    doc.write_soak(&mut out);
    doc.write_encumbrance(&mut out);
    doc.write_health_tracks(&mut out);
    doc.write_personality_traits(&mut out);
    doc.write_reputations(&mut out);
    doc.write_confidence(&mut out);
    doc.write_supernatural(&mut out);
    doc.write_magic_items(&mut out);
    doc.write_annotations(&mut out);
    let missing = doc.missing.into_inner();
    if !missing.is_empty() {
        return Err(ExportError {
            missing: missing.into_iter().collect(),
        });
    }
    Ok(out)
}

/// Which catalogue a carried [`EquipmentSlot`] belongs to. The slot stores only an
/// id, and the id alone does not say which catalogue holds it, so the three
/// catalogues are probed in turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Carried {
    Weapon,
    Shield,
    Armor,
}

/// The carried-equipment groups in document order, each with the chrome key that
/// names it.
const EQUIPMENT_GROUPS: [(Carried, &str); 3] = [
    (Carried::Weapon, "equipment-group-weapons"),
    (Carried::Shield, "equipment-group-shields"),
    (Carried::Armor, "equipment-group-armor"),
];

/// The point-item kinds in document order, each with the chrome key that names its
/// group. A fixed taxonomy: [`ItemKind`] has four variants and this is the whole
/// mapping, so a new variant fails to compile until it is listed here.
const ITEM_KIND_HEADINGS: [(ItemKind, &str); 4] = [
    (ItemKind::Virtue, "items-virtues-title"),
    (ItemKind::Flaw, "items-flaws-title"),
    (ItemKind::Boon, "export-items-boons"),
    (ItemKind::Hook, "export-items-hooks"),
];

/// Parameter key used when a parameterized entry's catalogue entry is missing, so
/// the chosen value is appended verbatim instead of vanishing. No rules parameter
/// uses this key, so it can never collide with a real `{placeholder}`.
const UNKNOWN_PARAM_KEY: &str = "parameter";

/// The formatter's inputs, bundled so each section reads as one small method.
///
/// `missing` accumulates every chrome key or catalogue id [`Doc::label`] /
/// [`Doc::name`] could not resolve while writing the document. A `RefCell` rather
/// than threading a `Result` through every section method: sections stay simple
/// string-appenders, and [`character_markdown`] turns an empty accumulator into
/// `Ok` or a non-empty one into [`ExportError`] once the whole document — not just
/// the first offending call — has been walked.
struct Doc<'a> {
    entity: &'a Entity,
    ruleset: &'a LocalizedRuleset,
    labels: &'a BTreeMap<String, String>,
    missing: RefCell<BTreeSet<MissingLabel>>,
}

/// A name-and-level table, built by [`Doc::leveled_rows`].
struct LeveledTable {
    headers: [String; 2],
    rows: Vec<Vec<String>>,
}

impl LeveledTable {
    fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// The shared shape of the free-text, level-bearing lists an entity can hold. Lets
/// one helper render all three without collapsing types that carry different budget
/// contracts.
trait Leveled {
    fn leveled_name(&self) -> &str;
    fn level(&self) -> u16;
}

impl Leveled for EnchantedDevice {
    fn leveled_name(&self) -> &str {
        &self.name
    }
    fn level(&self) -> u16 {
        self.level
    }
}

impl Leveled for SupernaturalPower {
    fn leveled_name(&self) -> &str {
        &self.name
    }
    fn level(&self) -> u16 {
        self.level
    }
}

impl Leveled for TalismanEffect {
    fn leveled_name(&self) -> &str {
        &self.name
    }
    fn level(&self) -> u16 {
        self.level
    }
}

// --- Formatting primitives -------------------------------------------------

/// Separator between the subtitle's parts. Punctuation, not prose — no locale owns
/// it.
const SUBTITLE_SEPARATOR: &str = " · ";

/// Separator inside a numeric *range* (a wound band's damage span). An en dash, not
/// the ASCII hyphen the project reserves for negative signs, so `1–5` can never be
/// misread as a minus. Mirrors the frontend's wound list.
const RANGE_DASH: &str = "–";

/// An optional total as a cell: the number, or an empty cell when the weapon has no
/// such total (Dodge has neither Attack nor Damage).
fn optional_number(value: Option<i32>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

/// A signed modifier for display: `+` for positive, an ASCII hyphen-minus `-`
/// (U+002D) for negative, plain for zero. The engine-side twin of `formatSigned`
/// in `ui/src/lib/derive.ts` — the mathematical minus U+2212 is never used, so
/// values stay copy-paste clean and read correctly in assistive tech.
fn signed(n: i32) -> String {
    if n > 0 {
        format!("+{n}")
    } else {
        n.to_string()
    }
}

/// Makes an arbitrary free-text value safe to print inside the document.
///
/// Every value the formatter prints from the entity — the name, concept, an
/// Ability's specialty, a familiar's animal, an aging-log entry — is an
/// unconstrained `String` the user typed. Three characters would otherwise change
/// the document's *structure* rather than its content: a `|` ends a table cell, a
/// newline ends a table row (or the paragraph), and a leading `#` becomes a
/// heading. So every cell goes through here.
fn escape_cell(text: &str) -> String {
    let flattened: String = text
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    // Backslash first: escaping it after adding our own would double-escape them.
    let escaped = flattened.replace('\\', "\\\\").replace('|', "\\|");
    let trimmed = escaped.trim();
    if trimmed.starts_with('#') {
        format!("\\{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// Appends one bullet per Personality Trait, name and signed value. Shared by the
/// character's own traits and the familiar's, which are separate lists of the same
/// shape. The names are free text, so each goes through [`escape_cell`].
fn write_traits(out: &mut String, traits: &[PersonalityTrait]) {
    for trait_ in traits {
        field(
            out,
            &escape_cell(&trait_.name),
            &signed(i32::from(trait_.value)),
        );
    }
    out.push('\n');
}

/// `label: value`, the inline pairing used in the subtitle and in read-out lines.
fn pair(label: &str, value: &str) -> String {
    format!("{label}: {value}")
}

/// Appends an ATX heading of `level` and the blank line after it.
fn heading(out: &mut String, level: usize, text: &str) {
    for _ in 0..level {
        out.push('#');
    }
    out.push(' ');
    out.push_str(text);
    out.push_str("\n\n");
}

/// Appends a `- **label**: value` bullet.
fn field(out: &mut String, label: &str, value: &str) {
    out.push_str("- **");
    out.push_str(label);
    out.push_str("**: ");
    out.push_str(value);
    out.push('\n');
}

/// Appends a GFM table and the blank line after it. A no-op when there are no
/// rows, so a section can never leave a headerless or bodyless table behind.
fn table(out: &mut String, headers: &[String], rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }
    out.push_str("| ");
    out.push_str(&headers.join(" | "));
    out.push_str(" |\n|");
    for _ in headers {
        out.push_str(" --- |");
    }
    out.push('\n');
    for row in rows {
        out.push_str("| ");
        out.push_str(&row.join(" | "));
        out.push_str(" |\n");
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ability::AbilityCategory;
    use crate::aging::CrisisSeverity;
    use crate::ruleset::RulesetSources;
    use crate::types::{
        AbilityScore, AgingLogEntry, ArtScore, EquipmentSlot, Familiar, LongevityRitual,
        LongevitySource, Magnitude, MightScore, PersonalityTrait, Realm, Reputation,
        ReputationType, RulesetRef, Selection, SpellSelection, Talisman, TalismanAttunement,
        TwilightScar,
    };
    use pretty_assertions::assert_eq;

    /// A small magus-capable ruleset with one Characteristic-moving Virtue.
    fn ruleset() -> LocalizedRuleset {
        let items = r#"[
          { "id": "virtue.giant_blood", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "major", "category": "general", "entity_kinds": ["character"],
            "effects": [
              { "type": "characteristic_score_delta", "characteristic": "characteristic.str", "amount": 1 }
            ] },
          { "id": "boon.rich_vis_source", "kind": "boon", "classification": "narrative",
            "magnitude": "minor", "category": "general", "entity_kinds": ["covenant"] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "category": "personality", "entity_kinds": ["character"] },
          { "id": "virtue.puissant_ability", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
            "effects": [{ "type": "ability_bonus", "param": "ability", "amount": 2 }] },
          { "id": "virtue.puissant_art", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
            "effects": [{ "type": "art_bonus", "param": "art", "amount": 3 }] },
          { "id": "virtue.minor_magical_focus", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "parameters": [{ "key": "focus", "type": "ref", "domain": "text" }],
            "effects": [{ "type": "magical_focus", "param": "focus", "major": false }] },
          { "id": "virtue.warrior", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "restricted_ability_xp", "amount": 50, "categories": ["martial"] }] },
          { "id": "virtue.second_sight", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "category": "supernatural", "entity_kinds": ["character"],
            "effects": [{ "type": "ability_score_grant", "ability": "ability.second_sight", "amount": 1 }] },
          { "id": "virtue.educated", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "restricted_ability_xp", "amount": 50,
              "abilities": ["ability.artes_liberales", "ability.dead_language"] }] },
          { "id": "virtue.malformed_name", "kind": "virtue", "classification": "narrative",
            "magnitude": "free", "category": "general", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          { "id": "magus", "is_magus": true,
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": [], "forbidden_categories": [],
            "required_traits": [], "forbidden_traits": [],
            "confidence_score": 1, "confidence_points": 3,
            "gift_policy": "required", "gift_categories": [], "creation_phases": ["concept"] },
          { "id": "covenant",
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": [], "forbidden_categories": [],
            "required_traits": [], "forbidden_traits": [], "creation_phases": ["concept"] }
        ]"#;
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
            { "score": 5, "total_xp": 75 }
          ],
          "abilities": [
            { "id": "ability.awareness", "category": "general" },
            { "id": "ability.artes_liberales", "category": "academic" },
            { "id": "ability.magic_theory", "category": "arcane" },
            { "id": "ability.parma_magica", "category": "arcane" },
            { "id": "ability.penetration", "category": "arcane" },
            { "id": "ability.philosophiae", "category": "academic" },
            { "id": "ability.area_lore", "category": "general", "parameter": "area" },
            { "id": "ability.dead_language", "category": "academic", "parameter": "language" },
            { "id": "ability.second_sight", "category": "supernatural" },
            { "id": "ability.single_weapon", "category": "martial" },
            { "id": "ability.living_language", "category": "general", "parameter": "language" },
            { "id": "ability.brawl", "category": "general", "combat_ability": true }
          ]
        }"#;
        // Inert unless the entity carries a `life_stages` plan — no existing test's
        // document changes — but present so the sheet can be driven for a guided
        // character, whose experience arrives in named blocks rather than one pool.
        let life_stages = r#"{
          "apprenticeship": {
            "years": 15, "xp": 240, "recommended_xp": 0,
            "minimum_abilities": [{ "ability": "ability.parma_magica", "min_score": 1 }],
            "recommended_abilities": []
          },
          "childhood": {
            "years": 5,
            "native_language_ability": "ability.living_language",
            "native_language_xp": 75,
            "spread_xp": 45,
            "spread_abilities": ["ability.awareness", "ability.brawl"]
          },
          "later_life": { "xp_per_year": 15 },
          "post_apprenticeship": {
            "lab_season_cost": 10,
            "max_charged_lab_seasons_per_year": 3,
            "points_per_year": 30
          }
        }"#;
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 5, "total_xp": 15 },
            { "score": 8, "total_xp": 36 }, { "score": 10, "total_xp": 55 }
          ],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.muto", "art_type": "technique" },
            { "id": "art.corpus", "art_type": "form" },
            { "id": "art.ignem", "art_type": "form" },
            { "id": "art.vim", "art_type": "form" }
          ]
        }"#;
        // Two grant-bearing Houses: Bonisagus offers a Puissant Ability *choice* (so
        // it grants nothing until the entity records a pick), Ex Miscellanea fixes a
        // free Flaw (so it grants without any pick).
        let houses = r#"{ "houses": [
          { "id": "house.bonisagus", "lineage_type": "true_lineage",
            "grants": [ { "kind": "choice", "choice_key": "bonisagus_puissant", "options": [
              { "ref": "virtue.puissant_ability", "params": { "ability": "ability.awareness" } },
              { "ref": "virtue.puissant_ability", "params": { "ability": "ability.single_weapon" } }
            ] } ] },
          { "id": "house.ex_miscellanea", "lineage_type": "societas",
            "grants": [ { "kind": "fixed", "item": "flaw.optimistic" } ] }
        ] }"#;
        let spells = r#"{ "spells": [
          { "id": "spell.pilum_of_fire", "technique": "art.creo", "form": "art.ignem",
            "level": 20, "ritual": false },
          { "id": "spell.wizards_boost_form", "technique": "art.muto", "form": "art.vim",
            "parameters": [{ "key": "form", "type": "ref", "domain": "form" }] }
        ] }"#;
        let mastery = r#"{ "abilities": [
          { "id": "spell_mastery_ability.penetration" }
        ] }"#;
        let equipment = r#"{
          "weapons": [
            { "id": "weapon.long_sword", "kind": "melee", "init_mod": 2, "attack_mod": 4,
              "defense_mod": 1, "damage_mod": 6, "min_strength": 0, "load": 1,
              "ability": "ability.single_weapon" },
            { "id": "weapon.sling", "kind": "missile", "init_mod": 0, "attack_mod": 2,
              "defense_mod": 0, "damage_mod": 3, "min_strength": -1, "load": 0,
              "range": 30, "ability": "ability.single_weapon" },
            { "id": "weapon.dodge", "kind": "melee", "init_mod": 0, "defense_mod": 0,
              "load": 0, "ability": "ability.brawl" }
          ],
          "shields": [
            { "id": "shield.round", "init_mod": 0, "attack_mod": 0, "defense_mod": 2,
              "load": 1, "min_strength": 0 }
          ],
          "armor": [
            { "id": "armor.leather_scale", "protection": 3, "load": 1 }
          ]
        }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: Some(arts),
            houses: Some(houses),
            mythic_types: None,
            spells: Some(spells),
            spell_mastery_abilities: Some(mastery),
            equipment: Some(equipment),
            characteristics: None,
            life_stages: Some(life_stages),
            childhoods: None,
            aging: None,
        })
        .unwrap();
        let i18n = r#"{
          "virtue.giant_blood": { "name": "Giant Blood" },
          "virtue.puissant_ability": { "name": "Puissant {ability}" },
          "virtue.puissant_art": { "name": "Puissant {art}" },
          "virtue.minor_magical_focus": { "name": "Minor Magical Focus" },
          "virtue.warrior": { "name": "Warrior" },
          "virtue.second_sight": { "name": "Second Sight" },
          "virtue.educated": { "name": "Educated" },
          "virtue.malformed_name": { "name": "Malformed {template" },
          "flaw.optimistic": { "name": "Optimistic" },
          "boon.rich_vis_source": { "name": "Rich Vis Source" },
          "ability.awareness": { "name": "Awareness" },
          "ability.area_lore": { "name": "{area} Lore" },
          "ability.artes_liberales": { "name": "Artes Liberales" },
          "ability.dead_language": { "name": "{language} (Dead Language)" },
          "ability.second_sight": { "name": "Second Sight" },
          "ability.single_weapon": { "name": "Single Weapon" },
          "ability.magic_theory": { "name": "Magic Theory" },
          "ability.parma_magica": { "name": "Parma Magica" },
          "ability.brawl": { "name": "Brawl" },
          "weapon.dodge": { "name": "Dodge" },
          "art.creo": { "name": "Creo", "abbreviation": "Cr" },
          "art.muto": { "name": "Muto", "abbreviation": "Mu" },
          "art.corpus": { "name": "Corpus", "abbreviation": "Co" },
          "art.ignem": { "name": "Ignem", "abbreviation": "Ig" },
          "art.vim": { "name": "Vim", "abbreviation": "Vi" },
          "spell.pilum_of_fire": { "name": "Pilum of Fire" },
          "spell.wizards_boost_form": { "name": "Wizard's Boost of {form}" },
          "spell_mastery_ability.penetration": { "name": "Penetration" },
          "weapon.long_sword": { "name": "Long Sword" },
          "weapon.sling": { "name": "Sling" },
          "shield.round": { "name": "Round Shield" },
          "armor.leather_scale": { "name": "Leather Scale" },
          "house.bonisagus": { "name": "Bonisagus" },
          "house.ex_miscellanea": { "name": "Ex Miscellanea" },
          "living_condition.leper": { "name": "Leper" },
          "living_condition.work_in_a_mine": { "name": "Work in a mine" },
          "crisis.minor_illness": { "name": "Minor illness" }
        }"#;
        LocalizedRuleset::new(rs, i18n).unwrap()
    }

    /// Every key `character_markdown` can request against the shared `ruleset()`
    /// fixture, resolved to itself: [`LABEL_KEYS`] plus the three catalogue-derived
    /// families its own docs name as deliberately excluded (`type-`,
    /// `param-label-`, `category-`) — mirroring the frontend's own
    /// `composedExportLabelKeys` (`ui/src/lib/state.svelte.ts`), which assembles
    /// the same union before every real export. The baseline for `labels()` /
    /// `no_labels()`, so a test exercising content unrelated to chrome never has
    /// to enumerate the whole key set just to avoid tripping the fail-loud path —
    /// see `labels_without` for a test that specifically wants a key absent.
    fn full_label_baseline() -> BTreeMap<String, String> {
        let rs = ruleset();
        let mut keys: BTreeSet<String> = LABEL_KEYS.iter().map(|k| k.to_string()).collect();
        for id in rs.ruleset.type_profiles.keys() {
            keys.insert(format!("type-{id}"));
        }
        for item in rs.ruleset.point_items.values() {
            keys.insert(format!("category-{}", item.category));
            for param in &item.parameters {
                keys.insert(format!("param-label-{}", param.key));
            }
        }
        for ability in rs.ruleset.abilities.values() {
            if let Some(parameter) = &ability.parameter {
                keys.insert(format!("param-label-{parameter}"));
            }
        }
        for spell in rs.ruleset.spells.values() {
            for param in &spell.parameters {
                keys.insert(format!("param-label-{}", param.key));
            }
        }
        keys.into_iter().map(|k| (k.clone(), k)).collect()
    }

    /// A label map that resolves every key to itself plus a marker, so a test can
    /// tell "the formatter asked for this key" from "this text came from the
    /// data". Starts from [`full_label_baseline`] so every other chrome key
    /// resolves too — a test that does not care about labels never fails the
    /// export just for omitting them.
    fn labels(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        let mut map = full_label_baseline();
        for (k, v) in pairs {
            map.insert((*k).to_string(), (*v).to_string());
        }
        map
    }

    fn no_labels() -> BTreeMap<String, String> {
        labels(&[])
    }

    /// [`full_label_baseline`] with `keys` removed — for a test exercising the
    /// fail-loud path itself, where exactly the named keys must be missing and
    /// nothing else.
    fn labels_without(keys: &[&str]) -> BTreeMap<String, String> {
        let mut map = full_label_baseline();
        for key in keys {
            map.remove(*key);
        }
        map
    }

    /// Test-only convenience shadowing [`super::character_markdown`]: every test
    /// below but the ones in the "labels" section supplies a complete label map
    /// (`labels()` / `no_labels()`, both built from [`full_label_baseline`]) and
    /// only cares about the rendered content, so this unwraps for them rather
    /// than making every call site spell out `.unwrap()`. A test that exercises
    /// the fail-loud path itself calls `super::character_markdown` directly and
    /// asserts on the `Err`.
    fn character_markdown(
        entity: &Entity,
        ruleset: &LocalizedRuleset,
        labels: &BTreeMap<String, String>,
    ) -> String {
        super::character_markdown(entity, ruleset, labels)
            .expect("test label maps are built from the complete baseline")
    }

    fn magus() -> Entity {
        Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        )
    }

    /// A magus with every export-visible surface filled in: the shared fixture for
    /// the contract tests and the section tests that need real derived numbers.
    ///
    /// Chosen so the expected figures are easy to check by hand: Stamina 2 + Leather
    /// Scale 3 = Soak 5; Load 2 → Burden 1, cancelled by Strength 1 → Encumbrance 0;
    /// Size 0 → wound unit 5.
    fn fully_populated_magus() -> Entity {
        let mut e = magus();
        e.name = "Marcus of Bonisagus".to_string();
        e.description = "A bookish theorist".to_string();
        e.concept = "Seeker after lost Hermetic lore".to_string();
        e.gender = "male".to_string();
        e.birth_year = Some(1194);
        e.sigil = "the smell of old parchment".to_string();
        e.covenant_name = "Semita Errabunda".to_string();
        e.parens = "Vittoria of Bonisagus".to_string();
        e.house = Some(Id::new("house.bonisagus"));
        e.age = Some(35);
        e.apparent_age = Some(30);
        e.aura = 3;
        for (c, score) in [
            (Characteristic::Int, 3),
            (Characteristic::Per, 1),
            (Characteristic::Str, 1),
            (Characteristic::Sta, 2),
            (Characteristic::Pre, -1),
            (Characteristic::Com, 1),
            (Characteristic::Dex, 1),
            (Characteristic::Qik, 1),
        ] {
            e.characteristics.insert(c, score);
        }
        e.characteristic_descriptions
            .insert(Characteristic::Int, "quick-witted".to_string());
        e.selections = vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".to_string(), Id::new("ability.awareness"))]),
            ),
            Selection::with_params(
                Id::new("virtue.puissant_art"),
                BTreeMap::from([("art".to_string(), Id::new("art.creo"))]),
            ),
            Selection::with_params(
                Id::new("virtue.minor_magical_focus"),
                BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
            ),
            Selection::new(Id::new("virtue.warrior")),
            Selection::new(Id::new("flaw.optimistic")),
        ];
        e.xp_pool = 240;
        e.ability_scores = vec![
            AbilityScore {
                ability: Id::new("ability.awareness"),
                score: 3,
                specialty: Some("searching".to_string()),
                parameter: None,
            },
            AbilityScore {
                ability: Id::new("ability.area_lore"),
                score: 2,
                specialty: Some("legends".to_string()),
                parameter: Some("Provence".to_string()),
            },
            AbilityScore {
                ability: Id::new("ability.magic_theory"),
                score: 4,
                specialty: None,
                parameter: None,
            },
            AbilityScore {
                ability: Id::new("ability.parma_magica"),
                score: 3,
                specialty: None,
                parameter: None,
            },
            AbilityScore {
                ability: Id::new("ability.single_weapon"),
                score: 4,
                specialty: Some("long sword".to_string()),
                parameter: None,
            },
        ];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.muto"),
                score: 5,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 5,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 8,
            },
            ArtScore {
                art: Id::new("art.vim"),
                score: 5,
            },
        ];
        e.spells = vec![
            SpellSelection {
                spell: Id::new("spell.pilum_of_fire"),
                level: None,
                mastery: Some(2),
                parameter: None,
                mastery_abilities: vec![Id::new("spell_mastery_ability.penetration")],
            },
            SpellSelection {
                spell: Id::new("spell.wizards_boost_form"),
                level: Some(15),
                mastery: None,
                parameter: Some("art.ignem".to_string()),
                mastery_abilities: Vec::new(),
            },
        ];
        e.equipment = vec![
            EquipmentSlot {
                item: Id::new("weapon.long_sword"),
                equipped: true,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("weapon.sling"),
                equipped: true,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("armor.leather_scale"),
                equipped: true,
                specialization_applies: false,
            },
        ];
        e.personality_traits = vec![
            PersonalityTrait {
                name: "Curious".to_string(),
                value: 3,
            },
            PersonalityTrait {
                name: "Brash".to_string(),
                value: -2,
            },
        ];
        e.reputations = vec![Reputation {
            kind: ReputationType::Hermetic,
            score: 2,
            content: "a promising theoretician".to_string(),
        }];
        e.devices = vec![EnchantedDevice {
            name: "Ring of Seeing".to_string(),
            level: 15,
        }];
        e.talisman = Some(Talisman {
            description: "an ash staff shod with silver".to_string(),
            attunements: vec![TalismanAttunement {
                description: "to ward off flame".to_string(),
                bonus: 3,
            }],
            effects: vec![TalismanEffect {
                name: "Lamp Without Flame".to_string(),
                level: 10,
            }],
        });
        e.longevity_ritual = Some(LongevityRitual {
            source: LongevitySource::SelfMade,
            bonus: Some(7),
            focus: "a draught of gold and silver".to_string(),
        });
        e.familiar = Some(Familiar {
            name: "Corvus".to_string(),
            animal: "a raven".to_string(),
            might: Some(MightScore {
                realm: Realm::Magic,
                score: 10,
            }),
            characteristics: BTreeMap::from([(Characteristic::Int, 2), (Characteristic::Qik, 4)]),
            size: -4,
            personality_traits: vec![PersonalityTrait {
                name: "Loyal".to_string(),
                value: 3,
            }],
            cord_gold: 2,
            cord_silver: 1,
            cord_bronze: 0,
            powers: vec![SupernaturalPower {
                name: "Wings of the Storm".to_string(),
                level: 20,
            }],
        });
        e.warping_points = 6;
        e.warping_effect = "his shadow lags a heartbeat behind".to_string();
        e.twilight_scars = vec![TwilightScar {
            description: "his eyes reflect no candlelight".to_string(),
        }];
        // Aging points on Presence: enough to force two drops, and deliberately not
        // on Stamina, so the Soak figure stays the hand-checkable 2 + 3.
        e.aging_points = BTreeMap::from([(Characteristic::Pre, 5)]);
        e.decrepitude_effect = "a persistent cough each winter".to_string();
        e.living_conditions = BTreeSet::from([Id::new("living_condition.work_in_a_mine")]);
        e.aging_log = vec![AgingLogEntry {
            year: Some(1220),
            effect: "an apparent aging crisis, weathered".to_string(),
            ..AgingLogEntry::default()
        }];
        e.normalize();
        e
    }

    fn covenant() -> Entity {
        Entity::new(
            EntityKind::Covenant,
            Id::new("covenant"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        )
    }

    // --- escape_cell ------------------------------------------------------

    #[test]
    fn escape_cell_escapes_a_pipe_so_it_cannot_end_a_table_cell() {
        assert_eq!(escape_cell("a | b"), "a \\| b");
    }

    #[test]
    fn escape_cell_flattens_newlines_so_they_cannot_end_a_row() {
        assert_eq!(escape_cell("first\nsecond\r\nthird"), "first second  third");
    }

    #[test]
    fn escape_cell_escapes_a_leading_hash_so_it_cannot_inject_a_heading() {
        assert_eq!(escape_cell("# Not a heading"), "\\# Not a heading");
    }

    #[test]
    fn escape_cell_escapes_a_backslash_before_adding_its_own() {
        assert_eq!(escape_cell("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn escape_cell_trims_surrounding_whitespace() {
        assert_eq!(escape_cell("  padded  "), "padded");
    }

    #[test]
    fn escape_cell_leaves_ordinary_text_alone() {
        assert_eq!(escape_cell("Bjornaer of Crintera"), "Bjornaer of Crintera");
    }

    // --- signed -----------------------------------------------------------

    #[test]
    fn signed_prefixes_a_plus_for_a_positive_value() {
        assert_eq!(signed(3), "+3");
    }

    #[test]
    fn signed_leaves_zero_unsigned() {
        assert_eq!(signed(0), "0");
    }

    #[test]
    fn signed_uses_an_ascii_hyphen_for_a_negative_value() {
        assert_eq!(signed(-2), "-2");
        assert!(!signed(-2).contains('\u{2212}'), "no mathematical minus");
    }

    // --- labels -----------------------------------------------------------

    #[test]
    fn a_missing_label_key_fails_the_export_naming_the_key() {
        let mut e = magus();
        e.name = "Marcus".to_string();
        let err = super::character_markdown(&e, &ruleset(), &labels_without(&["type-magus"]))
            .expect_err("a missing chrome key must fail the export, not degrade to it");
        assert_eq!(
            err.missing,
            vec![MissingLabel::Key("type-magus".to_string())]
        );
    }

    #[test]
    fn an_export_collects_every_missing_key_in_one_pass() {
        let mut e = magus();
        e.house = Some(Id::new("house.diedne"));
        let err = super::character_markdown(
            &e,
            &ruleset(),
            &labels_without(&["type-magus", "house-label"]),
        )
        .expect_err("multiple omissions must still fail the export");
        assert_eq!(
            err.missing,
            vec![
                MissingLabel::Key("house-label".to_string()),
                MissingLabel::Key("type-magus".to_string()),
                MissingLabel::CatalogueId("house.diedne".to_string()),
            ]
        );
    }

    #[test]
    fn label_keys_are_sorted_and_free_of_duplicates() {
        let mut sorted = LABEL_KEYS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.as_slice(),
            LABEL_KEYS,
            "LABEL_KEYS must be sorted and duplicate-free (canonical order)"
        );
    }

    // --- title ------------------------------------------------------------

    #[test]
    fn the_title_is_the_entity_name() {
        let mut e = magus();
        e.name = "Marcus of Bonisagus".to_string();
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(
            doc.starts_with("# Marcus of Bonisagus\n"),
            "unexpected title: {doc}"
        );
    }

    #[test]
    fn an_unnamed_entity_uses_the_untitled_label_rather_than_a_bare_hash() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("export-untitled", "Untitled character")]),
        );
        assert!(
            doc.starts_with("# Untitled character\n"),
            "unexpected title: {doc}"
        );
    }

    #[test]
    fn the_subtitle_names_the_character_type_the_house_and_both_ages() {
        let mut e = magus();
        e.house = Some(Id::new("house.bonisagus"));
        e.age = Some(35);
        e.apparent_age = Some(30);
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("type-magus", "Magus"),
                ("house-label", "House"),
                ("age-label", "Age"),
                ("apparent-age-label", "Apparent age"),
            ]),
        );
        assert!(
            doc.contains("*Magus · House: Bonisagus · Age: 35 · Apparent age: 30*"),
            "unexpected subtitle: {doc}"
        );
    }

    #[test]
    fn an_unknown_house_id_fails_the_export_naming_it() {
        let mut e = magus();
        e.house = Some(Id::new("house.diedne"));
        let err = super::character_markdown(&e, &ruleset(), &no_labels())
            .expect_err("a House id the ruleset has no name for must fail the export");
        assert_eq!(
            err.missing,
            vec![MissingLabel::CatalogueId("house.diedne".to_string())]
        );
    }

    // --- identity ---------------------------------------------------------

    #[test]
    fn the_identity_section_lists_only_the_filled_fields() {
        let mut e = magus();
        e.concept = "A bookish theorist".to_string();
        e.birth_year = Some(1194);
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("identity-label", "Identity"),
                ("identity-concept", "Concept"),
                ("identity-birth-year", "Birth year"),
                ("identity-gender", "Gender"),
            ]),
        );
        assert!(doc.contains("## Identity\n"), "missing section: {doc}");
        assert!(doc.contains("- **Concept**: A bookish theorist\n"), "{doc}");
        assert!(doc.contains("- **Birth year**: 1194\n"), "{doc}");
        assert!(!doc.contains("Gender"), "an empty field is omitted: {doc}");
    }

    #[test]
    fn the_identity_section_is_omitted_when_every_field_is_empty() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("identity-label", "Identity")]),
        );
        assert!(!doc.contains("## Identity"), "stray heading: {doc}");
    }

    #[test]
    fn free_text_identity_is_escaped() {
        let mut e = magus();
        e.description = "Knight | Crusader".to_string();
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("identity-description", "Short description")]),
        );
        assert!(doc.contains("Knight \\| Crusader"), "{doc}");
    }

    // --- characteristics --------------------------------------------------

    #[test]
    fn the_characteristics_table_lists_bought_scores_and_descriptions() {
        let mut e = magus();
        e.characteristics.insert(Characteristic::Int, 3);
        e.characteristic_descriptions
            .insert(Characteristic::Int, "quick-witted".to_string());
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("characteristics-title", "Characteristics"),
                ("characteristic-int", "Intelligence"),
            ]),
        );
        assert!(doc.contains("## Characteristics\n"), "{doc}");
        assert!(
            doc.contains("| Intelligence | +3 |  | quick-witted |"),
            "unexpected row: {doc}"
        );
    }

    #[test]
    fn the_effective_column_is_filled_only_when_a_virtue_moves_the_score() {
        let mut e = magus();
        e.characteristics.insert(Characteristic::Str, 2);
        e.selections = vec![Selection::new(Id::new("virtue.giant_blood"))];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("characteristic-str", "Strength")]),
        );
        assert!(
            doc.contains("| Strength | +2 | +3 |  |"),
            "unexpected row: {doc}"
        );
    }

    /// Aging lowers the *effective* Characteristic, so the Effective column has to
    /// read the aging-aware value — a sheet that printed the un-aged score would
    /// contradict every derived total, which all read after-aging scores.
    /// A Communication of +2 drops on its 3rd aging point, so +2 becomes +1.
    #[test]
    fn the_effective_column_reflects_aging_drops() {
        let mut e = magus();
        e.characteristics.insert(Characteristic::Com, 2);
        e.aging_points = BTreeMap::from([(Characteristic::Com, 3)]);
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("characteristic-com", "Communication")]),
        );
        assert!(
            doc.contains("| Communication | +2 | +1 |  |"),
            "unexpected row: {doc}"
        );
    }

    /// Aging and a free Virtue delta are separate layers: the drop lowers the bought
    /// score, the delta then adds on top.
    #[test]
    fn the_effective_column_combines_an_aging_drop_with_a_virtue_delta() {
        let mut e = magus();
        e.characteristics.insert(Characteristic::Str, 2);
        e.selections = vec![Selection::new(Id::new("virtue.giant_blood"))];
        e.aging_points = BTreeMap::from([(Characteristic::Str, 5)]);
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("characteristic-str", "Strength")]),
        );
        // Bought +2, two aging drops → 0, Giant Blood +1 → +1 effective.
        assert!(
            doc.contains("| Strength | +2 | +1 |  |"),
            "unexpected row: {doc}"
        );
    }

    #[test]
    fn a_described_characteristic_with_no_bought_score_still_gets_a_row() {
        let mut e = magus();
        e.characteristic_descriptions
            .insert(Characteristic::Pre, "unremarkable".to_string());
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("characteristic-pre", "Presence")]),
        );
        assert!(doc.contains("| Presence | 0 |  | unremarkable |"), "{doc}");
    }

    /// A score of 0 is stored as an *absent* key (the frontend deletes it), so a
    /// Characteristic bought at 0 is indistinguishable from an untouched one. Both
    /// belong on the sheet — every Characteristic is part of a character — so once
    /// the section exists it lists the whole taxonomy, the untouched ones as `0`.
    #[test]
    fn every_characteristic_gets_a_row_once_the_section_exists() {
        let mut e = magus();
        e.characteristics.insert(Characteristic::Int, 3);
        e.characteristics.insert(Characteristic::Pre, -1);
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("characteristic-int", "Intelligence"),
                ("characteristic-per", "Perception"),
                ("characteristic-str", "Strength"),
                ("characteristic-sta", "Stamina"),
                ("characteristic-pre", "Presence"),
                ("characteristic-com", "Communication"),
                ("characteristic-dex", "Dexterity"),
                ("characteristic-qik", "Quickness"),
            ]),
        );
        assert!(doc.contains("| Intelligence | +3 |  |  |"), "{doc}");
        assert!(doc.contains("| Presence | -1 |  |  |"), "{doc}");
        for untouched in [
            "Perception",
            "Strength",
            "Stamina",
            "Communication",
            "Dexterity",
            "Quickness",
        ] {
            assert!(
                doc.contains(&format!("| {untouched} | 0 |  |  |")),
                "an untouched Characteristic still gets an unsigned 0 row: {doc}"
            );
        }
        // One row per Characteristic and no more: the taxonomy is the row set.
        let rows = doc
            .lines()
            .filter(|line| line.starts_with("| ") && line.ends_with(" |  |  |"))
            .count();
        assert_eq!(rows, Characteristic::ALL.len(), "{doc}");
    }

    #[test]
    fn the_characteristics_section_is_omitted_when_nothing_is_entered() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("characteristics-title", "Characteristics")]),
        );
        assert!(!doc.contains("## Characteristics"), "stray heading: {doc}");
    }

    // --- edge cases -------------------------------------------------------

    /// A blank character still has a body, so its Fatigue and Wound tracks are real
    /// content (see `write_health_tracks`); every other section is empty and absent.
    #[test]
    fn an_empty_entity_renders_a_minimal_document() {
        let doc = character_markdown(&magus(), &ruleset(), &no_labels());
        assert!(doc.starts_with("# export-untitled\n"), "{doc}");
        let sections: Vec<&str> = doc.lines().filter(|l| l.starts_with("## ")).collect();
        assert_eq!(
            sections,
            vec![
                "## derived-section-fatigue",
                "## derived-section-wounds",
                "## confidence-label"
            ],
            "only the body's own constants survive an empty entity: {doc}"
        );
        // Exactly the two health-track tables, each with a body: `table` is a no-op
        // without rows, so a separator line can only exist above real rows.
        let separators = doc.lines().filter(|l| l.starts_with("| --- ")).count();
        assert_eq!(separators, 2, "no bodyless tables: {doc}");
    }

    #[test]
    fn an_empty_covenant_renders_only_its_title() {
        let doc = character_markdown(&covenant(), &ruleset(), &no_labels());
        assert!(doc.starts_with("# export-untitled\n"), "{doc}");
        assert!(!doc.contains("## "), "no section at all: {doc}");
    }

    #[test]
    fn a_covenant_omits_the_character_only_sections() {
        let mut e = covenant();
        e.name = "Semita Errabunda".to_string();
        e.description = "A covenant in the Rhine".to_string();
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("identity-label", "Identity"),
                ("identity-description", "Short description"),
                ("characteristics-title", "Characteristics"),
            ]),
        );
        assert!(doc.contains("# Semita Errabunda\n"), "{doc}");
        assert!(doc.contains("## Identity\n"), "{doc}");
        assert!(
            !doc.contains("## Characteristics"),
            "a covenant has no Characteristics: {doc}"
        );
    }

    // --- virtues & flaws --------------------------------------------------

    #[test]
    fn virtues_and_flaws_are_split_by_kind_with_their_magnitude() {
        let mut e = magus();
        e.selections = vec![
            Selection::new(Id::new("virtue.giant_blood")),
            Selection::new(Id::new("flaw.optimistic")),
        ];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("tab-virtues-flaws", "Virtues & Flaws"),
                ("items-virtues-title", "Virtues"),
                ("items-flaws-title", "Flaws"),
                ("category-general", "General"),
                ("category-personality", "Personality"),
                ("magnitude-major", "Major"),
                ("magnitude-minor", "Minor"),
            ]),
        );
        assert!(doc.contains("## Virtues & Flaws\n"), "{doc}");
        assert!(doc.contains("### Virtues\n"), "{doc}");
        assert!(doc.contains("| Giant Blood | General | Major |"), "{doc}");
        assert!(doc.contains("### Flaws\n"), "{doc}");
        assert!(
            doc.contains("| Optimistic | Personality | Minor |"),
            "{doc}"
        );
    }

    /// The item's category is the "type" the in-app badge shows (Hermetic, Social
    /// Status, Story, …), and it belongs on the sheet next to the magnitude: the two
    /// together are how a reader places a Virtue.
    #[test]
    fn a_virtue_row_reports_its_type_beside_its_magnitude() {
        let mut e = magus();
        e.selections = vec![Selection::with_params(
            Id::new("virtue.puissant_art"),
            BTreeMap::from([("art".to_string(), Id::new("art.creo"))]),
        )];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("identity-name", "Name"),
                ("items-virtues-title", "Virtues"),
                ("export-col-type", "Type"),
                ("export-col-magnitude", "Magnitude"),
                ("category-hermetic", "Hermetic"),
                ("magnitude-minor", "Minor"),
            ]),
        );
        assert!(doc.contains("| Name | Type | Magnitude |\n"), "{doc}");
        assert!(
            doc.contains("| Puissant Creo | Hermetic | Minor |"),
            "{doc}"
        );
    }

    #[test]
    fn a_covenants_boons_are_not_filed_under_virtues() {
        let mut e = covenant();
        e.selections = vec![Selection::new(Id::new("boon.rich_vis_source"))];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("items-virtues-title", "Virtues"),
                ("export-items-boons", "Boons"),
            ]),
        );
        assert!(doc.contains("### Boons\n"), "{doc}");
        assert!(!doc.contains("### Virtues"), "{doc}");
        assert!(doc.contains("| Rich Vis Source |"), "{doc}");
    }

    #[test]
    fn a_parameterized_virtue_names_its_chosen_target() {
        let mut e = magus();
        e.selections = vec![Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.awareness"))]),
        )];
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(doc.contains("| Puissant Awareness |"), "{doc}");
    }

    #[test]
    fn a_parameter_the_name_template_ignores_is_appended_rather_than_dropped() {
        let mut e = magus();
        e.selections = vec![Selection::with_params(
            Id::new("virtue.minor_magical_focus"),
            BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
        )];
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(doc.contains("| Minor Magical Focus (fire) |"), "{doc}");
    }

    /// A parameter *value* can itself be a parameterized Ability: Puissant Ability
    /// aimed at the character's Provence Lore stores `{ability: ability.area_lore,
    /// area: "Provence"}` — `area` is the target instance's own discriminator, not a
    /// parameter of Puissant Ability. Rendering the value's name raw leaked the
    /// template ("Puissant {area} Lore (Provence)").
    #[test]
    fn a_virtue_targeting_a_parameterized_ability_names_the_whole_instance() {
        let mut e = magus();
        e.selections = vec![Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([
                ("ability".to_string(), Id::new("ability.area_lore")),
                ("area".to_string(), Id::new("Provence")),
            ]),
        )];
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(doc.contains("| Puissant Provence Lore |"), "{doc}");
        assert!(!doc.contains("{area}"), "no raw placeholder: {doc}");
    }

    /// The same target with no instance chosen yet keeps the slot label, and still
    /// never shows the raw template.
    #[test]
    fn a_virtue_targeting_a_parameterized_ability_shows_the_instance_slot_label() {
        let mut e = magus();
        e.selections = vec![Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.area_lore"))]),
        )];
        let doc = character_markdown(&e, &ruleset(), &labels(&[("param-label-area", "Area")]));
        assert!(doc.contains("| Puissant (Area) Lore |"), "{doc}");
    }

    #[test]
    fn an_unfilled_parameter_shows_its_localized_slot_label() {
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.puissant_ability"))];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("param-label-ability", "Ability")]),
        );
        assert!(doc.contains("| Puissant (Ability) |"), "{doc}");
    }

    #[test]
    fn the_point_balance_reports_used_points_against_the_type_ceiling() {
        let mut e = magus();
        e.selections = vec![
            Selection::new(Id::new("virtue.giant_blood")),
            Selection::new(Id::new("flaw.optimistic")),
        ];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("items-virtues-title", "Virtues"),
                ("items-flaws-title", "Flaws"),
            ]),
        );
        assert!(doc.contains("- **Virtues**: 3 / 10\n"), "{doc}");
        assert!(doc.contains("- **Flaws**: 1 / 10\n"), "{doc}");
    }

    #[test]
    fn the_virtues_and_flaws_section_is_omitted_when_nothing_is_selected() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("tab-virtues-flaws", "Virtues & Flaws")]),
        );
        assert!(!doc.contains("Virtues & Flaws"), "stray heading: {doc}");
    }

    // --- granted (off-budget) virtues & flaws -----------------------------

    /// A magus of a House whose Puissant-Ability choice has been picked.
    fn bonisagus_with_granted_puissant() -> Entity {
        let mut e = magus();
        e.house = Some(Id::new("house.bonisagus"));
        e.house_choices = BTreeMap::from([(
            "bonisagus_puissant".to_string(),
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".to_string(), Id::new("ability.awareness"))]),
            ),
        )]);
        e.selections = vec![Selection::new(Id::new("virtue.giant_blood"))];
        e
    }

    #[test]
    fn a_house_granted_virtue_is_listed_under_virtues_marked_as_granted() {
        let doc = character_markdown(
            &bonisagus_with_granted_puissant(),
            &ruleset(),
            &labels(&[
                ("items-virtues-title", "Virtues"),
                ("export-granted", "Granted"),
                ("category-general", "General"),
                ("magnitude-major", "Major"),
                ("magnitude-minor", "Minor"),
            ]),
        );
        // The bought Virtue keeps its own table; the granted one sits in a marked
        // sub-table of the same kind section.
        assert!(doc.contains("### Virtues\n"), "{doc}");
        assert!(doc.contains("| Giant Blood | General | Major |"), "{doc}");
        assert!(doc.contains("#### Granted\n"), "{doc}");
        assert!(
            doc.contains("| Puissant Awareness | General | Minor |"),
            "{doc}"
        );
    }

    #[test]
    fn a_granted_virtue_does_not_enter_the_point_balance() {
        let rs = ruleset();
        let names = labels(&[
            ("items-virtues-title", "Virtues"),
            ("export-granted", "Granted"),
        ]);
        let granted = bonisagus_with_granted_puissant();
        let mut ungranted = granted.clone();
        ungranted.house_choices.clear();

        let with_grant = character_markdown(&granted, &rs, &names);
        let without_grant = character_markdown(&ungranted, &rs, &names);
        // Giant Blood (Major) alone funds the balance in both documents: the free
        // House Virtue is off-budget, so it moves no number.
        assert!(
            with_grant.contains("- **Virtues**: 3 / 10\n"),
            "{with_grant}"
        );
        assert!(
            without_grant.contains("- **Virtues**: 3 / 10\n"),
            "{without_grant}"
        );
        assert!(with_grant.contains("#### Granted\n"), "{with_grant}");
        assert!(
            !without_grant.contains("Granted"),
            "an unpicked choice grants nothing: {without_grant}"
        );
    }

    #[test]
    fn a_granted_flaw_is_filed_under_flaws_not_virtues() {
        let mut e = magus();
        e.house = Some(Id::new("house.ex_miscellanea"));
        e.selections = vec![Selection::new(Id::new("virtue.giant_blood"))];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("items-virtues-title", "Virtues"),
                ("items-flaws-title", "Flaws"),
                ("export-granted", "Granted"),
                ("category-personality", "Personality"),
                ("magnitude-minor", "Minor"),
            ]),
        );
        // The Flaws section exists purely for the granted row, and the Virtues
        // section — which has only a bought row — grows no granted sub-table.
        assert!(doc.contains("### Flaws\n\n#### Granted\n"), "{doc}");
        assert!(
            doc.contains("| Optimistic | Personality | Minor |"),
            "{doc}"
        );
        assert_eq!(doc.matches("#### Granted").count(), 1, "{doc}");
        // The granted Flaw is off-budget: the Flaw balance stays at zero.
        assert!(doc.contains("- **Flaws**: 0 / 10\n"), "{doc}");
    }

    #[test]
    fn no_granted_sub_table_appears_when_nothing_is_granted() {
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.giant_blood"))];
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(!doc.contains("export-granted"), "phantom heading: {doc}");
    }

    // --- abilities --------------------------------------------------------

    #[test]
    fn abilities_list_specialty_bought_and_effective_scores() {
        let mut e = magus();
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 3,
            specialty: Some("searching".to_string()),
            parameter: None,
        }];
        e.selections = vec![Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.awareness"))]),
        )];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("abilities-title", "Abilities"),
                ("ability-specialty-label", "Specialty"),
            ]),
        );
        assert!(doc.contains("## Abilities\n"), "{doc}");
        assert!(doc.contains("| Awareness | searching | 3 | 5 |"), "{doc}");
    }

    #[test]
    fn two_instances_of_one_ability_stay_separate_rows() {
        let mut e = magus();
        e.ability_scores = vec![
            AbilityScore {
                ability: Id::new("ability.area_lore"),
                score: 2,
                specialty: None,
                parameter: Some("Provence".to_string()),
            },
            AbilityScore {
                ability: Id::new("ability.area_lore"),
                score: 1,
                specialty: Some("legends".to_string()),
                parameter: Some("the Rhine".to_string()),
            },
        ];
        e.normalize();
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(doc.contains("| Provence Lore |  | 2 |"), "{doc}");
        assert!(doc.contains("| the Rhine Lore | legends | 1 |"), "{doc}");
    }

    #[test]
    fn the_xp_pool_reports_the_general_draw_and_each_restricted_pool() {
        let mut e = magus();
        e.xp_pool = 60;
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 3,
            specialty: None,
            parameter: None,
        }];
        e.selections = vec![Selection::new(Id::new("virtue.warrior"))];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("xp-pool", "XP pool"),
                ("export-xp-restricted", "Restricted experience"),
                ("ability-category-martial", "Martial"),
            ]),
        );
        assert!(doc.contains("- **XP pool**: 30 / 60\n"), "{doc}");
        assert!(doc.contains("### Restricted experience\n"), "{doc}");
        assert!(doc.contains("- **Martial**: 0 / 50\n"), "{doc}");
    }

    /// An Ability the character *has* purely because a Virtue seeded it (Second
    /// Sight 1) is stored nowhere in `ability_scores`, so iterating the bought
    /// instances alone drops it from the sheet — even though the character can use
    /// it. The floor row shows a bought 0 against its effective score.
    #[test]
    fn a_granted_but_unbought_ability_gets_its_own_row() {
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.second_sight"))];
        let doc = character_markdown(&e, &ruleset(), &labels(&[("abilities-title", "Abilities")]));
        assert!(
            doc.contains("## Abilities\n"),
            "a granted Ability opens the section on its own: {doc}"
        );
        assert!(doc.contains("| Second Sight |  | 0 | 1 |"), "{doc}");
    }

    /// A granted floor the character has also bought above is already reported by the
    /// bought row (whose effective score is the higher of the two), so it must not
    /// produce a second row for the same Ability.
    #[test]
    fn a_granted_ability_the_character_bought_too_gets_no_second_row() {
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.second_sight"))];
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.second_sight"),
            score: 3,
            specialty: Some("visions".to_string()),
            parameter: None,
        }];
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        // The catalogue names the Virtue and the Ability it grants identically (the
        // real rulebook does too), so the count is scoped to the Ability row's own
        // shape rather than the bare name, which the Virtues/Flaws table also prints.
        assert_eq!(
            doc.matches("| Second Sight | visions |").count(),
            1,
            "one row per Ability: {doc}"
        );
        assert!(doc.contains("| Second Sight | visions | 3 |  |"), "{doc}");
    }

    /// The granted rows follow the bought ones: the sheet's first block is what the
    /// character spent experience on.
    #[test]
    fn granted_ability_rows_follow_the_bought_ones() {
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.second_sight"))];
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2,
            specialty: None,
            parameter: None,
        }];
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        // Scoped to the Abilities table: the catalogue names the Virtue and the
        // Ability it grants identically (the real rulebook does too), and the
        // Virtues/Flaws table — which prints first — has its own "Second Sight" row.
        let abilities = &doc[doc
            .find("## abilities-title")
            .expect("the abilities section")..];
        let bought = abilities.find("| Awareness |").expect("the bought row");
        let granted = abilities.find("| Second Sight |").expect("the granted row");
        assert!(bought < granted, "unexpected row order: {doc}");
    }

    /// A restricted pool's eligibility list names catalogue Abilities, and those names
    /// can carry a `{placeholder}` (Dead Language is "{language} (Dead Language)"), so
    /// the list has to go through the same parameterized formatter every other name
    /// uses — otherwise the sheet shows the raw template.
    #[test]
    fn pool_eligibility_renders_parameterized_names_with_hints() {
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.educated"))];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("export-xp-restricted", "Restricted experience"),
                ("restricted-xp-list-separator", ","),
                ("param-label-language", "Language"),
            ]),
        );
        assert!(
            doc.contains("- **Artes Liberales, (Language) (Dead Language)**: 0 / 50\n"),
            "{doc}"
        );
        assert!(!doc.contains("{language}"), "no raw placeholder: {doc}");
    }

    /// A life-stage block is named for **where the experience came from**, never for
    /// what it may buy: the childhood spread lists the entire childhood Ability
    /// catalogue, both childhood blocks share that list, and later life's is longer
    /// still — so an eligibility enumeration is both unreadable and unable to tell
    /// the blocks apart. The in-app XP bar has always labelled them by origin
    /// (`xp-pool-<block>`); the sheet now says the same thing.
    #[test]
    fn a_life_stage_pool_is_labelled_by_its_origin_not_its_eligibility() {
        let mut e = magus();
        // Fifteen years of apprenticeship after a childhood of five leaves five
        // years of later life, at 15 experience points a year.
        e.age = Some(25);
        e.life_stages = Some(crate::life_stage::LifeStagePlan {
            native_language: Some("German".into()),
            ..crate::life_stage::LifeStagePlan::default()
        });
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("export-xp-restricted", "Restricted experience"),
                ("restricted-xp-list-separator", ","),
                ("xp-pool-childhood_native_language", "Native language"),
                ("xp-pool-childhood_spread", "Early childhood"),
                ("xp-pool-later_life", "Later life"),
            ]),
        );
        assert!(doc.contains("- **Native language**: 0 / 75\n"), "{doc}");
        assert!(doc.contains("- **Early childhood**: 0 / 45\n"), "{doc}");
        assert!(doc.contains("- **Later life**: 0 / 75\n"), "{doc}");
        // The eligibility enumeration these three replace.
        assert!(!doc.contains("Awareness, Brawl"), "{doc}");
    }

    /// A Virtue's grant keeps its eligibility list: the item's own name says nothing
    /// about what the 50 points may buy, and Educated and Warrior differ precisely
    /// there. Same rule the XP bar follows.
    #[test]
    fn a_virtue_granted_pool_still_names_what_it_may_buy() {
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.warrior"))];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("export-xp-restricted", "Restricted experience"),
                ("ability-category-martial", "Martial"),
            ]),
        );
        assert!(doc.contains("- **Martial**: 0 / 50\n"), "{doc}");
    }

    #[test]
    fn the_abilities_section_is_omitted_when_nothing_is_bought_and_no_xp_is_banked() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("abilities-title", "Abilities")]),
        );
        assert!(!doc.contains("## Abilities"), "stray heading: {doc}");
    }

    // --- arts -------------------------------------------------------------

    #[test]
    fn arts_are_split_into_techniques_and_forms() {
        let mut e = magus();
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 8,
            },
        ];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("tab-arts", "Arts"),
                ("art-type-technique", "Techniques"),
                ("art-type-form", "Forms"),
            ]),
        );
        let techniques = doc.find("### Techniques").expect("techniques heading");
        let forms = doc.find("### Forms").expect("forms heading");
        assert!(techniques < forms, "Techniques come first: {doc}");
        assert!(doc.contains("| Creo | 10 |"), "{doc}");
        assert!(doc.contains("| Ignem | 8 |"), "{doc}");
    }

    #[test]
    fn an_art_bonus_fills_the_effective_column() {
        let mut e = magus();
        e.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 10,
        }];
        e.selections = vec![Selection::with_params(
            Id::new("virtue.puissant_art"),
            BTreeMap::from([("art".to_string(), Id::new("art.creo"))]),
        )];
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(doc.contains("| Creo | 10 | 13 |"), "{doc}");
    }

    /// A score of 0 is stored as no score at all, so listing only the *stored* Arts
    /// drops every Art the character has not scored — up to the whole Forms table,
    /// even though the character casts spells of those Forms. The catalogue is the
    /// row set.
    #[test]
    fn every_catalogue_art_gets_a_row_once_any_art_is_scored() {
        let mut e = magus();
        e.art_scores = vec![ArtScore {
            art: Id::new("art.ignem"),
            score: 8,
        }];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("tab-arts", "Arts"),
                ("art-type-technique", "Techniques"),
                ("art-type-form", "Forms"),
            ]),
        );
        // A magus who scored no Technique still gets the Techniques table.
        assert!(doc.contains("### Techniques\n"), "{doc}");
        assert!(doc.contains("| Creo | 0 |  |"), "{doc}");
        assert!(doc.contains("| Muto | 0 |  |"), "{doc}");
        assert!(doc.contains("### Forms\n"), "{doc}");
        assert!(doc.contains("| Ignem | 8 |  |"), "{doc}");
        assert!(doc.contains("| Vim | 0 |  |"), "{doc}");
        // Id order within a group — which is also the sheet's canonical Art order.
        let creo = doc.find("| Creo |").expect("a Creo row");
        let muto = doc.find("| Muto |").expect("a Muto row");
        let corpus = doc.find("| Corpus |").expect("a Corpus row");
        let ignem = doc.find("| Ignem |").expect("an Ignem row");
        assert!(creo < muto && corpus < ignem, "Arts run in id order: {doc}");
    }

    #[test]
    fn the_arts_section_is_omitted_when_no_art_is_bought() {
        let doc = character_markdown(&magus(), &ruleset(), &labels(&[("tab-arts", "Arts")]));
        assert!(!doc.contains("## Arts"), "stray heading: {doc}");
    }

    // --- spells -----------------------------------------------------------

    /// The sheet names a spell's Arts and level the way the rulebook and the app do:
    /// one short code, Technique and Form abbreviations followed by the level
    /// (`CrIg20`), not three separate columns of spelled-out Art names.
    #[test]
    fn spells_list_their_art_code_and_mastery() {
        let mut e = magus();
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: Some(2),
            parameter: None,
            mastery_abilities: vec![Id::new("spell_mastery_ability.penetration")],
        }];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("identity-name", "Name"),
                ("tab-spells", "Spells"),
                ("export-col-spell-code", "TeFo/Level"),
                ("spell-mastery-label", "Mastery"),
                ("spell-mastery-abilities-label", "Mastery abilities"),
            ]),
        );
        assert!(doc.contains("## Spells\n"), "{doc}");
        assert!(
            doc.contains("| Name | TeFo/Level | Mastery | Mastery abilities |\n"),
            "{doc}"
        );
        assert!(
            doc.contains("| Pilum of Fire | CrIg20 | 2 | Penetration |"),
            "{doc}"
        );
    }

    #[test]
    fn a_general_spell_with_no_chosen_level_shows_the_general_marker() {
        let mut e = magus();
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.wizards_boost_form"),
            level: None,
            mastery: None,
            parameter: Some("art.ignem".to_string()),
            mastery_abilities: Vec::new(),
        }];
        let doc = character_markdown(&e, &ruleset(), &labels(&[("spell-level-general", "Gen")]));
        assert!(doc.contains("| Wizard's Boost of Ignem |"), "{doc}");
        // The marker sits a space after the Arts, where a resolved level would abut
        // them: `MuVi15` is one figure, `MuVi Gen` two readable halves.
        assert!(doc.contains("| MuVi Gen |"), "{doc}");
    }

    #[test]
    fn the_spells_section_is_omitted_when_no_spell_is_known() {
        let doc = character_markdown(&magus(), &ruleset(), &labels(&[("tab-spells", "Spells")]));
        assert!(!doc.contains("## Spells"), "stray heading: {doc}");
    }

    // --- equipment --------------------------------------------------------

    #[test]
    fn equipment_is_grouped_by_catalogue_kind_with_the_equipped_marker() {
        let mut e = magus();
        e.equipment = vec![
            EquipmentSlot {
                item: Id::new("weapon.long_sword"),
                equipped: true,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("shield.round"),
                equipped: false,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("armor.leather_scale"),
                equipped: true,
                specialization_applies: false,
            },
        ];
        e.normalize();
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("tab-equipment", "Equipment"),
                ("equipment-group-weapons", "Weapons"),
                ("equipment-group-shields", "Shields"),
                ("equipment-group-armor", "Armor"),
                ("export-yes", "Yes"),
                ("export-no", "No"),
            ]),
        );
        assert!(doc.contains("## Equipment\n"), "{doc}");
        assert!(doc.contains("### Weapons\n"), "{doc}");
        assert!(doc.contains("| Long Sword | Yes |"), "{doc}");
        assert!(doc.contains("### Shields\n"), "{doc}");
        assert!(doc.contains("| Round Shield | No |"), "{doc}");
        assert!(doc.contains("### Armor\n"), "{doc}");
        assert!(doc.contains("| Leather Scale | Yes |"), "{doc}");
    }

    #[test]
    fn the_equipment_section_is_omitted_when_nothing_is_carried() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("tab-equipment", "Equipment")]),
        );
        assert!(!doc.contains("## Equipment"), "stray heading: {doc}");
    }

    // --- combat / soak / encumbrance / fatigue / wounds -------------------

    #[test]
    fn combat_lists_one_row_per_equipped_weapon() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("derived-section-combat", "Combat"),
                ("derived-combat-init", "Init"),
                ("derived-range", "Range"),
            ]),
        );
        assert!(doc.contains("## Combat\n"), "{doc}");
        // Qik 1 + weapon Init 2 + shield 0 - Encumbrance 0; Single Weapon 4.
        assert!(doc.contains("| Long Sword | Single Weapon | 3 |"), "{doc}");
        // A missile weapon carries a Range; the melee weapon leaves the cell blank.
        assert!(doc.contains("| Sling | Single Weapon |"), "{doc}");
        assert!(doc.contains("| 30 |"), "the missile range: {doc}");
    }

    /// With a shield equipped, a one-handed weapon prints twice: the with-shield row
    /// first, named `<weapon> <joiner> <shield>`, then the bare row (Ars Magica - Definitive Edition (Core Rules).md:16656;
    /// ordering Ars Magica - Definitive Edition (Core Rules).md:1467-1472). The joiner is a localized label, never hardcoded.
    #[test]
    fn combat_with_a_shield_emits_with_shield_then_bare_rows() {
        let mut e = fully_populated_magus();
        e.equipment.push(EquipmentSlot {
            item: Id::new("shield.round"),
            equipped: true,
            specialization_applies: false,
        });
        e.normalize();
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("derived-section-combat", "Combat"),
                ("derived-combat-shield-joiner", "&"),
            ]),
        );
        // Defense = Qik 1 + Single Weapon 4 + WpnDef 1 (+ ShieldDef 2 on the first).
        let with_shield = doc.find("| Long Sword & Round Shield | Single Weapon | 2 | 9 | 8 | 7 |");
        let bare = doc.find("| Long Sword | Single Weapon | 2 | 9 | 6 | 7 |");
        assert!(with_shield.is_some(), "no with-shield row: {doc}");
        assert!(bare.is_some(), "no bare row: {doc}");
        assert!(
            with_shield < bare,
            "the with-shield row must come first: {doc}"
        );
    }

    #[test]
    fn the_combat_section_is_omitted_when_no_weapon_is_equipped() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("derived-section-combat", "Combat")]),
        );
        assert!(!doc.contains("## Combat"), "stray heading: {doc}");
    }

    #[test]
    fn soak_lists_every_addend_and_the_total() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("derived-section-soak", "Soak"),
                ("derived-addend-stamina", "Stamina"),
                ("derived-addend-armor", "Armor"),
                ("export-col-total", "Total"),
            ]),
        );
        assert!(doc.contains("## Soak\n"), "{doc}");
        assert!(doc.contains("- **Stamina**: +2\n"), "{doc}");
        assert!(doc.contains("- **Armor**: +3\n"), "{doc}");
        assert!(doc.contains("- **Total**: +5\n"), "{doc}");
    }

    #[test]
    fn the_soak_section_is_omitted_when_every_addend_is_zero() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("derived-section-soak", "Soak")]),
        );
        assert!(!doc.contains("## Soak"), "stray heading: {doc}");
    }

    #[test]
    fn encumbrance_reports_load_burden_and_the_penalty() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("derived-section-encumbrance", "Encumbrance"),
                ("derived-load", "Load"),
                ("derived-burden", "Burden"),
                ("export-col-total", "Total"),
            ]),
        );
        assert!(doc.contains("## Encumbrance\n"), "{doc}");
        assert!(doc.contains("- **Load**: 2\n"), "{doc}");
        assert!(doc.contains("- **Burden**: 1\n"), "{doc}");
        assert!(doc.contains("- **Total**: 0\n"), "{doc}");
    }

    #[test]
    fn the_encumbrance_section_is_omitted_when_nothing_is_carried() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("derived-section-encumbrance", "Encumbrance")]),
        );
        assert!(!doc.contains("## Encumbrance"), "stray heading: {doc}");
    }

    #[test]
    fn the_fatigue_and_wound_tracks_are_printed_for_a_character() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("derived-section-fatigue", "Fatigue"),
                ("derived-fatigue-weary", "Weary"),
                ("derived-section-wounds", "Wounds"),
                ("derived-wound-light", "Light"),
                ("derived-wound-dead", "Dead"),
                ("export-col-penalty", "Penalty"),
            ]),
        );
        assert!(doc.contains("## Fatigue\n"), "{doc}");
        assert!(doc.contains("| Weary | -1 |"), "{doc}");
        assert!(doc.contains("## Wounds\n"), "{doc}");
        assert!(doc.contains("| Light | 1–5 | -1 |"), "{doc}");
        assert!(doc.contains("| Dead | 21+ |  |"), "{doc}");
    }

    #[test]
    fn a_covenant_has_no_fatigue_or_wound_track() {
        let mut e = covenant();
        e.name = "Semita Errabunda".to_string();
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("derived-section-fatigue", "Fatigue"),
                ("derived-section-wounds", "Wounds"),
            ]),
        );
        assert!(!doc.contains("Fatigue"), "{doc}");
        assert!(!doc.contains("Wounds"), "{doc}");
    }

    // --- traits, reputations, confidence ----------------------------------

    #[test]
    fn personality_traits_and_reputations_are_listed_with_signed_values() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("personality-label", "Personality Traits"),
                ("reputations-label", "Reputations"),
                ("reputation-type-hermetic", "Hermetic"),
            ]),
        );
        assert!(doc.contains("## Personality Traits\n"), "{doc}");
        assert!(doc.contains("- **Brash**: -2\n"), "{doc}");
        assert!(doc.contains("- **Curious**: +3\n"), "{doc}");
        assert!(doc.contains("## Reputations\n"), "{doc}");
        assert!(
            doc.contains("- **Hermetic 2**: a promising theoretician\n"),
            "{doc}"
        );
    }

    #[test]
    fn confidence_reports_the_derived_score_and_points() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("confidence-label", "Confidence"),
                ("ability-score-label", "Score"),
                ("export-col-points", "Points"),
            ]),
        );
        assert!(doc.contains("## Confidence\n"), "{doc}");
        assert!(doc.contains("- **Score**: 1\n"), "{doc}");
        assert!(doc.contains("- **Points**: 3\n"), "{doc}");
    }

    #[test]
    fn the_traits_sections_are_omitted_when_empty() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[
                ("personality-label", "Personality Traits"),
                ("reputations-label", "Reputations"),
            ]),
        );
        assert!(!doc.contains("Personality Traits"), "{doc}");
        assert!(!doc.contains("Reputations"), "{doc}");
    }

    // --- magic ------------------------------------------------------------

    #[test]
    fn the_magic_items_block_covers_aura_devices_longevity_and_the_talisman() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("tab-possessions", "Magic Items"),
                ("aura-label", "Aura"),
                ("possessions-devices-label", "Enchanted Devices"),
                ("longevity-label", "Longevity Ritual"),
                ("longevity-source-label", "Source"),
                ("longevity-source-self_made", "Self-made"),
                ("longevity-bonus-label", "Aging bonus"),
                ("longevity-focus-label", "Focus"),
                ("talisman-label", "Talisman"),
                ("talisman-description-label", "Shape and material"),
                ("talisman-attunements-label", "Talisman Attunements"),
                ("talisman-effects-label", "Instilled Effects"),
            ]),
        );
        assert!(doc.contains("## Magic Items\n"), "{doc}");
        assert!(doc.contains("- **Aura**: +3\n"), "{doc}");
        assert!(doc.contains("### Enchanted Devices\n"), "{doc}");
        assert!(doc.contains("| Ring of Seeing | 15 |"), "{doc}");
        assert!(doc.contains("### Longevity Ritual\n"), "{doc}");
        assert!(doc.contains("- **Source**: Self-made\n"), "{doc}");
        assert!(doc.contains("- **Aging bonus**: +7\n"), "{doc}");
        assert!(
            doc.contains("- **Focus**: a draught of gold and silver\n"),
            "{doc}"
        );
        assert!(doc.contains("### Talisman\n"), "{doc}");
        assert!(
            doc.contains("- **Shape and material**: an ash staff shod with silver\n"),
            "{doc}"
        );
        assert!(doc.contains("#### Talisman Attunements\n"), "{doc}");
        assert!(doc.contains("| to ward off flame | +3 |"), "{doc}");
        assert!(doc.contains("#### Instilled Effects\n"), "{doc}");
        assert!(doc.contains("| Lamp Without Flame | 10 |"), "{doc}");
    }

    #[test]
    fn an_unentered_longevity_bonus_says_so_rather_than_claiming_zero() {
        let mut e = magus();
        e.longevity_ritual = Some(LongevityRitual {
            source: LongevitySource::External,
            bonus: None,
            focus: String::new(),
        });
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("longevity-bonus-label", "Aging bonus"),
                ("longevity-not-entered", "Not entered"),
                ("longevity-source-external", "External"),
            ]),
        );
        assert!(doc.contains("- **Aging bonus**: Not entered\n"), "{doc}");
        assert!(
            !doc.contains("longevity-focus-label"),
            "no empty focus: {doc}"
        );
    }

    #[test]
    fn the_familiar_statblock_lists_the_beasts_own_scores_cords_and_powers() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("familiar-label", "Familiar"),
                ("identity-name", "Name"),
                ("familiar-animal-label", "Animal"),
                ("familiar-might-label", "Magic Might"),
                ("realm-magic", "Magic"),
                ("familiar-size-label", "Size"),
                ("familiar-cord-gold", "Gold cord"),
                ("familiar-cord-silver", "Silver cord"),
                ("familiar-cord-bronze", "Bronze cord"),
                ("characteristics-title", "Characteristics"),
                ("characteristic-qik", "Quickness"),
                ("personality-label", "Personality Traits"),
                ("familiar-powers-label", "Invested Powers"),
            ]),
        );
        assert!(doc.contains("### Familiar\n"), "{doc}");
        assert!(doc.contains("- **Name**: Corvus\n"), "{doc}");
        assert!(doc.contains("- **Animal**: a raven\n"), "{doc}");
        assert!(doc.contains("- **Magic Might**: Magic 10\n"), "{doc}");
        assert!(doc.contains("- **Size**: -4\n"), "{doc}");
        assert!(doc.contains("- **Gold cord**: +2\n"), "{doc}");
        assert!(
            !doc.contains("Bronze cord"),
            "a zero cord is omitted: {doc}"
        );
        assert!(doc.contains("#### Characteristics\n"), "{doc}");
        assert!(doc.contains("| Quickness | +4 |"), "{doc}");
        assert!(doc.contains("#### Personality Traits\n"), "{doc}");
        assert!(doc.contains("- **Loyal**: +3\n"), "{doc}");
        assert!(doc.contains("#### Invested Powers\n"), "{doc}");
        assert!(doc.contains("| Wings of the Storm | 20 |"), "{doc}");
    }

    #[test]
    fn a_might_being_reports_its_effective_might_and_powers() {
        let mut e = magus();
        e.might = Some(MightScore {
            realm: Realm::Faerie,
            score: 15,
        });
        e.powers = vec![SupernaturalPower {
            name: "Glamour".to_string(),
            level: 25,
        }];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("tab-supernatural", "Supernatural"),
                ("supernatural-might-label", "Might Score"),
                ("realm-faerie", "Faerie"),
                ("supernatural-powers-label", "Supernatural Powers"),
            ]),
        );
        assert!(doc.contains("## Supernatural\n"), "{doc}");
        assert!(doc.contains("- **Might Score**: Faerie 15\n"), "{doc}");
        assert!(doc.contains("### Supernatural Powers\n"), "{doc}");
        assert!(doc.contains("| Glamour | 25 |"), "{doc}");
    }

    #[test]
    fn the_magic_blocks_are_omitted_for_a_magus_with_no_possessions() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[
                ("tab-possessions", "Magic Items"),
                ("tab-supernatural", "Supernatural"),
            ]),
        );
        assert!(!doc.contains("Magic Items"), "{doc}");
        assert!(!doc.contains("Supernatural"), "{doc}");
    }

    // --- annotations ------------------------------------------------------

    #[test]
    fn the_annotation_block_records_warping_twilight_decrepitude_and_aging() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("aging-label", "Aging"),
                ("warping-label", "Warping"),
                ("ability-score-label", "Score"),
                ("warping-points-label", "Warping points"),
                ("warping-effect-label", "Warping effect"),
                ("twilight-scars-label", "Twilight Scars"),
                ("decrepitude-label", "Decrepitude"),
                ("decrepitude-effect-label", "Decrepitude effect"),
                ("aging-log-heading", "Aging log"),
            ]),
        );
        assert!(doc.contains("## Aging\n"), "{doc}");
        assert!(doc.contains("### Warping\n"), "{doc}");
        assert!(doc.contains("- **Warping points**: 6\n"), "{doc}");
        assert!(
            doc.contains("- **Warping effect**: his shadow lags a heartbeat behind\n"),
            "{doc}"
        );
        assert!(doc.contains("### Twilight Scars\n"), "{doc}");
        assert!(doc.contains("- his eyes reflect no candlelight\n"), "{doc}");
        assert!(doc.contains("### Decrepitude\n"), "{doc}");
        assert!(
            doc.contains("- **Decrepitude effect**: a persistent cough each winter\n"),
            "{doc}"
        );
        assert!(doc.contains("### Aging log\n"), "{doc}");
        assert!(
            doc.contains("- **1220**: an apparent aging crisis, weathered\n"),
            "{doc}"
        );
    }

    /// The accrued aging points are recorded per Characteristic and drive both the
    /// Decrepitude Score and the Characteristic drops, so the sheet has to show them —
    /// without them a reader cannot tell how close a Characteristic is to its next
    /// drop. Two points on a Presence of 0 open the whole block on their own: they are
    /// too few for a Decrepitude Score, so nothing else in the block is non-empty.
    #[test]
    fn aging_points_per_characteristic_are_listed_in_the_aging_block() {
        let mut e = magus();
        e.aging_points = BTreeMap::from([(Characteristic::Pre, 2), (Characteristic::Sta, 0)]);
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("aging-label", "Aging"),
                ("aging-points-heading", "Aging points per Characteristic"),
                ("characteristic-pre", "Presence"),
                ("characteristic-sta", "Stamina"),
            ]),
        );
        assert!(doc.contains("## Aging\n"), "{doc}");
        assert!(
            doc.contains("### Aging points per Characteristic\n"),
            "{doc}"
        );
        assert!(doc.contains("- **Presence**: 2\n"), "{doc}");
        assert!(
            !doc.contains("Stamina"),
            "a zero entry carries no information: {doc}"
        );
    }

    /// The aging log records crises; the points record the accrual toward the next
    /// drop. The points come first, so the block reads from state to history.
    #[test]
    fn the_aging_points_precede_the_aging_log() {
        let doc = character_markdown(
            &fully_populated_magus(),
            &ruleset(),
            &labels(&[
                ("aging-points-heading", "Aging points"),
                ("aging-log-heading", "Aging log"),
            ]),
        );
        let points = doc
            .find("### Aging points")
            .expect("an aging-points heading");
        let log = doc.find("### Aging log").expect("an aging-log heading");
        assert!(points < log, "unexpected order: {doc}");
    }

    /// The Living Conditions are a stored choice and a standing term of every aging
    /// total (Ars Magica - Definitive Edition (Core Rules).md:16567-16569, :16581-16594), so the sheet has to carry
    /// them — a sheet that dropped them would read as data loss. They are catalogue
    /// ids, so they print through the rules i18n and never as the slug. Each logged
    /// year prints the stress die and the total it produced alongside its free text,
    /// so the sheet records what produced the outcome.
    #[test]
    fn the_aging_block_names_the_living_conditions_and_each_years_die_and_total() {
        let mut e = fully_populated_magus();
        e.living_conditions = BTreeSet::from([
            Id::new("living_condition.work_in_a_mine"),
            Id::new("living_condition.leper"),
        ]);
        e.aging_log = vec![AgingLogEntry {
            year: Some(1220),
            age: Some(40),
            effect: "an apparent aging crisis, weathered".to_string(),
            die: Some(9),
            total: Some(13),
            ..AgingLogEntry::default()
        }];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("aging-label", "Aging"),
                ("living-conditions-label", "Living Conditions"),
                ("restricted-xp-list-separator", ","),
                ("aging-log-heading", "Aging log"),
                ("aging-die-label", "Stress die"),
                ("export-col-total", "Total"),
            ]),
        );
        assert!(
            doc.contains("- **Living Conditions**: Leper, Work in a mine\n"),
            "{doc}"
        );
        assert!(
            doc.contains(
                "- **1220**: an apparent aging crisis, weathered · Stress die: 9 · Total: 13\n"
            ),
            "{doc}"
        );
        assert!(
            !doc.contains("living_condition."),
            "a raw slug reached the sheet: {doc}"
        );
    }

    /// A resolved Crisis is the most consequential thing an aging year can record,
    /// and the sheet dropped all four of its fields — so a character who had
    /// weathered a Major illness read exactly like one who had not.
    ///
    /// The row prints through the rules i18n keyed by its id, the severity through
    /// `crisis-severity-<slug>`, and both dice and both totals stand beside each
    /// other so a reader can check the arithmetic of `:16621` off the sheet.
    #[test]
    fn a_logged_crisis_prints_its_row_its_severity_and_the_die_that_found_it() {
        let mut e = fully_populated_magus();
        e.aging_log = vec![AgingLogEntry {
            year: Some(1220),
            age: Some(40),
            effect: String::new(),
            die: Some(9),
            total: Some(13),
            crisis: true,
            crisis_die: Some(10),
            crisis_total: Some(15),
            crisis_row: Some(Id::new("crisis.minor_illness")),
            crisis_severity: Some(CrisisSeverity::Minor),
            ..AgingLogEntry::default()
        }];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("aging-log-heading", "Aging log"),
                ("aging-die-label", "Stress die"),
                ("export-col-total", "Total"),
                ("crisis-label", "Crisis"),
                ("crisis-die-label", "Simple die"),
                ("crisis-total-label", "Crisis total"),
                ("crisis-severity-minor", "Minor"),
            ]),
        );
        assert!(
            doc.contains(
                "- **1220**: Stress die: 9 · Total: 13 · Crisis: Minor illness (Minor) \
                 · Simple die: 10 · Crisis total: 15\n"
            ),
            "{doc}"
        );
        assert!(doc.contains("Minor illness"), "the row is named: {doc}");
        assert!(
            !doc.contains("crisis.minor_illness"),
            "a raw slug reached the sheet: {doc}"
        );
    }

    /// A Crisis the table demanded and nobody has rolled is a state of its own, and
    /// the sheet has to be able to say so — otherwise it reads identically to a year
    /// that never met the Crisis Table at all.
    #[test]
    fn a_crisis_nobody_has_rolled_says_so_rather_than_printing_nothing() {
        let mut e = fully_populated_magus();
        e.aging_log = vec![AgingLogEntry {
            year: Some(1220),
            age: Some(40),
            effect: String::new(),
            die: Some(9),
            total: Some(13),
            crisis: true,
            ..AgingLogEntry::default()
        }];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("aging-log-heading", "Aging log"),
                ("aging-die-label", "Stress die"),
                ("export-col-total", "Total"),
                ("aging-log-crisis-unrolled", "Crisis owed, not yet rolled"),
            ]),
        );
        assert!(doc.contains("· Crisis owed, not yet rolled\n"), "{doc}");
    }

    /// A character with no birth year logs no calendar year, so the entry has no
    /// label to print. It prints as a plain bullet — never a Rust `None`, and
    /// never an empty bold label.
    #[test]
    fn an_undated_aging_log_entry_prints_its_effect_without_a_year_label() {
        let mut e = fully_populated_magus();
        e.aging_log = vec![AgingLogEntry {
            effect: "an apparent aging crisis, weathered".to_string(),
            ..AgingLogEntry::default()
        }];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("aging-log-heading", "Aging log")]),
        );
        assert!(
            doc.contains("- an apparent aging crisis, weathered\n"),
            "{doc}"
        );
        assert!(!doc.contains("None"), "{doc}");
        assert!(!doc.contains("- ****"), "{doc}");
    }

    #[test]
    fn the_annotation_block_is_omitted_for_an_unwarped_unaged_character() {
        let doc = character_markdown(&magus(), &ruleset(), &labels(&[("aging-label", "Aging")]));
        assert!(!doc.contains("## Aging"), "stray heading: {doc}");
    }

    /// A stored-but-zero aging-point entry is no aging at all, so it must not open the
    /// block (an old save can carry one; the frontend deletes it on edit).
    #[test]
    fn an_all_zero_aging_point_map_leaves_the_annotation_block_out() {
        let mut e = magus();
        e.aging_points = BTreeMap::from([(Characteristic::Pre, 0)]);
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("aging-label", "Aging"),
                ("aging-points-heading", "Aging points"),
            ]),
        );
        assert!(!doc.contains("## Aging"), "stray heading: {doc}");
    }

    // --- catalogue gaps and empty shapes ----------------------------------

    /// A brace with no closing partner is literal text, not a placeholder — a
    /// malformed i18n template must not swallow the rest of the name.
    #[test]
    fn an_unterminated_brace_in_a_name_template_is_literal_text() {
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.malformed_name"))];
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(doc.contains("| Malformed {template |"), "{doc}");
    }

    /// Ids no catalogue holds cannot be filed under a kind / Art class / equipment
    /// group, so they are not printed — `validate` reports each as an unknown ref.
    #[test]
    fn entries_absent_from_the_catalogue_are_not_printed() {
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.from_another_ruleset"))];
        e.art_scores = vec![ArtScore {
            art: Id::new("art.imaginem"),
            score: 4,
        }];
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.trebuchet"),
            equipped: true,
            specialization_applies: false,
        }];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("tab-virtues-flaws", "Virtues & Flaws"),
                ("tab-arts", "Arts"),
                ("tab-equipment", "Equipment"),
            ]),
        );
        assert!(!doc.contains("Virtues & Flaws"), "{doc}");
        assert!(!doc.contains("## Arts"), "{doc}");
        assert!(!doc.contains("## Equipment"), "{doc}");
    }

    /// A spell the catalogue does not hold has no name to resolve, so it now fails
    /// the export by the same rule as an unknown House ([`Doc::name`]) rather than
    /// keeping its row with a raw slug.
    #[test]
    fn an_unknown_spell_fails_the_export_naming_it() {
        let mut e = magus();
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.from_another_ruleset"),
            level: None,
            mastery: None,
            parameter: Some("free text".to_string()),
            mastery_abilities: Vec::new(),
        }];
        let err =
            super::character_markdown(&e, &ruleset(), &labels(&[("spell-level-general", "Gen")]))
                .expect_err("a spell id the ruleset has no name for must fail the export");
        assert_eq!(
            err.missing,
            vec![MissingLabel::CatalogueId(
                "spell.from_another_ruleset".to_string()
            )]
        );
    }

    /// An entity whose type profile is unknown still renders: Confidence has no base
    /// to derive from and is omitted, and the balance line drops the ceiling rather
    /// than inventing one.
    #[test]
    fn an_unknown_type_profile_omits_confidence_and_the_point_ceilings() {
        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("hedge_wizard"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        e.selections = vec![Selection::new(Id::new("flaw.optimistic"))];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("items-flaws-title", "Flaws"),
                ("confidence-label", "Confidence"),
                ("type-hedge_wizard", "Hedge Wizard"),
            ]),
        );
        assert!(doc.contains("- **Flaws**: 1\n"), "no ceiling: {doc}");
        assert!(!doc.contains("## Confidence"), "{doc}");
    }

    /// An untouched talisman writes no keys at all, so it must not produce a heading.
    #[test]
    fn an_empty_talisman_is_not_printed() {
        let mut e = magus();
        e.talisman = Some(Talisman::default());
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("tab-possessions", "Magic Items"),
                ("talisman-label", "Talisman"),
            ]),
        );
        assert!(!doc.contains("Talisman"), "{doc}");
        assert!(!doc.contains("Magic Items"), "{doc}");
    }

    /// A familiar entered as nothing but powers still prints them; the empty
    /// identity lines are skipped one by one.
    #[test]
    fn a_bare_familiar_prints_only_what_was_entered() {
        let mut e = magus();
        e.familiar = Some(Familiar {
            powers: vec![SupernaturalPower {
                name: "Ghostly Whispers".to_string(),
                level: 5,
            }],
            ..Familiar::default()
        });
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("familiar-label", "Familiar"),
                ("familiar-powers-label", "Invested Powers"),
                ("identity-name", "Name"),
                ("familiar-animal-label", "Animal"),
            ]),
        );
        assert!(doc.contains("### Familiar\n"), "{doc}");
        assert!(doc.contains("| Ghostly Whispers | 5 |"), "{doc}");
        assert!(!doc.contains("- **Name**"), "{doc}");
        assert!(!doc.contains("Animal"), "{doc}");
    }

    /// A Might-less being that nonetheless holds powers still gets the section.
    #[test]
    fn powers_without_a_might_score_still_render() {
        let mut e = magus();
        e.powers = vec![SupernaturalPower {
            name: "Second Sight".to_string(),
            level: 10,
        }];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("tab-supernatural", "Supernatural"),
                ("supernatural-might-label", "Might Score"),
                ("supernatural-powers-label", "Supernatural Powers"),
            ]),
        );
        assert!(doc.contains("## Supernatural\n"), "{doc}");
        assert!(!doc.contains("Might Score"), "{doc}");
        assert!(doc.contains("| Second Sight | 10 |"), "{doc}");
    }

    /// A weapon with neither Attack nor Damage (Dodge) leaves those cells blank.
    #[test]
    fn a_weapon_without_attack_or_damage_leaves_those_cells_blank() {
        let mut e = magus();
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.dodge"),
            equipped: true,
            specialization_applies: false,
        }];
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(doc.contains("| Dodge | Brawl | 0 |  | 0 |  |  |"), "{doc}");
    }

    // --- contract tests ---------------------------------------------------

    /// The per-line derived read-outs the export deliberately leaves out: they are
    /// working figures for play, not character-sheet content. Each is mapped to a
    /// unique marker so its absence is proven by the marker, not by English wording
    /// that another section might legitimately share.
    const EXCLUDED_READOUT_KEYS: &[&str] = &[
        "derived-section-lab",
        "derived-section-casting",
        "derived-section-lab-casting",
        "derived-section-penetration",
        "derived-section-magic-resistance",
        "derived-section-masterpiece",
        "derived-section-familiar",
        "derived-section-surfaced",
        "derived-lab-total",
        "derived-masterpiece-cap",
        "derived-familiar-binding-level",
        "derived-familiar-cord-points",
        "derived-familiar-invested-levels",
        "derived-longevity-aging-modifier",
        "derived-longevity-suggested",
        "talisman-capacity",
        "talisman-capacity-note",
    ];

    #[test]
    fn the_document_excludes_the_per_line_derived_readouts() {
        let mut excluded = full_label_baseline();
        for (i, key) in EXCLUDED_READOUT_KEYS.iter().enumerate() {
            excluded.insert((*key).to_string(), format!("EXCLUDEDMARKER{i}"));
        }
        let doc = character_markdown(&fully_populated_magus(), &ruleset(), &excluded);
        assert!(
            !doc.contains("EXCLUDEDMARKER"),
            "an excluded read-out leaked into the document: {doc}"
        );
        for key in EXCLUDED_READOUT_KEYS {
            assert!(!doc.contains(key), "the excluded key '{key}' leaked: {doc}");
        }
    }

    fn assert_declared(key: &str) {
        assert!(
            LABEL_KEYS.contains(&key),
            "LABEL_KEYS is missing '{key}' — it would render as a raw slug"
        );
    }

    /// Every key the formatter composes from a fixed rules taxonomy must be declared
    /// in [`LABEL_KEYS`]. Without this, adding a `derived.rs` Soak addend (or a
    /// taxonomy variant) would silently surface its slug as a user-facing label.
    #[test]
    fn every_taxonomy_derived_label_key_is_declared() {
        let rs = ruleset();
        let e = fully_populated_magus();
        for c in Characteristic::ALL {
            assert_declared(&format!("characteristic-{c}"));
        }
        for m in Magnitude::ALL {
            assert_declared(&format!("magnitude-{m}"));
        }
        for c in AbilityCategory::ALL {
            assert_declared(&format!("ability-category-{c}"));
        }
        for t in ArtType::ALL {
            assert_declared(&format!("art-type-{t}"));
        }
        for (_, key) in ITEM_KIND_HEADINGS {
            assert_declared(key);
        }
        for addend in soak(&e, &rs.ruleset).addends {
            assert_declared(&format!("derived-addend-{}", addend.label));
        }
        for level in fatigue_levels(&e, &rs.ruleset) {
            assert_declared(&format!("derived-fatigue-{}", level.level));
        }
        for band in wound_ranges(&e, &rs.ruleset) {
            assert_declared(&format!("derived-wound-{}", band.level));
        }
        for kind in ReputationType::ALL {
            assert_declared(&format!("reputation-type-{kind}"));
        }
        for realm in [Realm::Magic, Realm::Faerie, Realm::Divine, Realm::Infernal] {
            assert_declared(&format!("realm-{realm}"));
        }
        for source in [LongevitySource::SelfMade, LongevitySource::External] {
            assert_declared(&format!("longevity-source-{source}"));
        }
        for severity in CrisisSeverity::ALL {
            assert_declared(&format!("crisis-severity-{severity}"));
        }
        for block in crate::effective::LifeStageBlock::ALL {
            assert_declared(&format!("xp-pool-{block}"));
        }
    }

    /// Complements the family walk above by proving it at the *document* level: with
    /// every declared key resolved to a marker, no key-shaped text may remain in the
    /// output, so no label can reach the reader as a raw slug.
    #[test]
    fn no_undeclared_label_key_reaches_the_document() {
        let e = fully_populated_magus();
        let mut resolved: BTreeMap<String, String> = LABEL_KEYS
            .iter()
            .map(|key| ((*key).to_string(), "RESOLVED".to_string()))
            .collect();
        // The three catalogue-derived families LABEL_KEYS does not enumerate.
        resolved.insert(format!("type-{}", e.type_id), "RESOLVED".to_string());
        for key in [
            "param-label-ability",
            "param-label-art",
            "param-label-focus",
            "category-general",
            "category-hermetic",
            "category-personality",
        ] {
            resolved.insert(key.to_string(), "RESOLVED".to_string());
        }
        let doc = character_markdown(&e, &ruleset(), &resolved);
        for family in [
            "export-",
            "derived-",
            "characteristic-",
            "magnitude-",
            "ability-category-",
            "art-type-",
            "category-",
            "param-label-",
            "identity-",
            "spell-",
            "equipment-",
            "items-",
            "tab-",
            "realm-",
            "reputation-type-",
            "longevity-",
            "familiar-",
            "talisman-",
            "warping-",
            "aging-",
            "living-conditions-",
            "confidence-",
            "decrepitude-",
            "supernatural-",
            "possessions-",
            "personality-",
            "twilight-",
            "power-level-",
            "device-level-",
            "aura-",
            "house-",
            "age-",
            "apparent-age-",
            "xp-pool",
            "restricted-xp-",
        ] {
            assert!(
                !doc.contains(family),
                "an undeclared '{family}*' key reached the document: {doc}"
            );
        }
    }

    #[test]
    fn two_renders_of_the_same_entity_are_byte_identical() {
        let mut e = magus();
        e.name = "Marcus".to_string();
        e.characteristics.insert(Characteristic::Int, 3);
        e.characteristics.insert(Characteristic::Str, -1);
        let rs = ruleset();
        assert_eq!(
            character_markdown(&e, &rs, &no_labels()),
            character_markdown(&e, &rs, &no_labels())
        );
    }
}
