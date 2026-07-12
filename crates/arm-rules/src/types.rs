//! Core data model: the language-neutral types every other module builds on.
//!
//! Defines stable slug-style [`Id`]s, the entity-generic [`Entity`] and its
//! [`EntityTypeProfile`], the [`PointItem`] catalogue (virtues, flaws,
//! abilities, …), the recursive [`Prereq`] expression tree, and [`Effect`]s
//! that feed the effective-score layer. Carries no user-facing prose — all
//! translatable text lives in the Fluent/i18n layers, keyed by these ids.

use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::ability::AbilityCategory;
use crate::characteristics::Characteristic;

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

impl From<Id> for String {
    fn from(id: Id) -> Self {
        id.0
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
    /// All magnitudes in canonical (ascending point-weight) order. The single
    /// source the serialized magnitude→points table is derived from, so the UI
    /// never re-hardcodes the 0/1/3 values.
    pub const ALL: [Magnitude; 3] = [Magnitude::Free, Magnitude::Minor, Magnitude::Major];

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

/// Coarse classification of a Virtue/Flaw by *what kind of mechanical impact* it
/// has, assigned to every catalogue entry (M5 slice 5a). It partitions the whole
/// V/F catalogue into three disjoint classes and is the definitive input to the
/// effect-wiring slices: `creation_effect` items feed the creation-number wiring
/// (slice 5a-wire), `in_play_effect` items feed the derived-totals `Effect`
/// variant set (slice 5b / `derived.rs`), and `narrative` items are deliberately
/// left with no mechanical effect.
///
/// Required on [`PointItem`] (no serde default): a catalogue entry that omits it
/// fails to load, so "every V/F is classified" is enforced at load time, not only
/// by the data-integrity test.
///
/// Source: classification scheme documented in `crates/arm-rules/RULES.md` (M5/5a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    /// No mechanical creation number and no in-play/derived-total effect: pure
    /// personality, story, or social-status flavor. Never given an invented effect.
    Narrative,
    /// Changes a character-creation number or state (starting scores, XP grants,
    /// Confidence, Size/characteristic deltas, reputation grants, spell-levels,
    /// item-level budget, free starting Supernatural Ability score, …). Includes
    /// every entry that already carries `effects`.
    CreationEffect,
    /// Does not change a creation number, but modifies an in-play/derived total the
    /// engine computes (casting, lab, penetration, magic resistance, combat, Soak,
    /// wound/fatigue penalties, study source-quality, aging/longevity).
    InPlayEffect,
}

impl fmt::Display for Classification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Classification::Narrative => f.write_str("narrative"),
            Classification::CreationEffect => f.write_str("creation_effect"),
            Classification::InPlayEffect => f.write_str("in_play_effect"),
        }
    }
}

/// Recursive boolean expression tree for prerequisites.
/// All = AND, Any = OR, Nor = NOR (none may be present; serde tag `"none"`).
///
/// `Has`, `House`, `AbilityMin`, and `ArtMin` reference IDs that ARE checked for
/// referential integrity at load: each must resolve against its registry (point
/// items / houses / abilities / arts) or the ruleset fails to load
/// (see [`crate::ruleset::Ruleset::validate_integrity`]).
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
    /// None of the children may be satisfied (NOR). The serde tag stays `"none"`
    /// (the variant is named `Nor` only to avoid colliding with `Option::None`).
    #[serde(rename = "none")]
    Nor(Vec<Prereq>),
    /// The entity must have the referenced item selected.
    Has(Id),
    /// The entity must belong to the referenced house. Evaluated against the
    /// entity's own house: a matching house satisfies it, a different house does
    /// not, and no house at all (a non-magus, or a magus who has not picked one)
    /// is genuinely unknown.
    House(Id),
    /// The entity must have the referenced ability at or above the given score.
    /// Evaluated against the entity's effective ability score (bought score
    /// plus virtue bonuses such as Puissant Ability); an ability the entity
    /// lacks counts as 0.
    AbilityMin { ability: Id, score: u8 },
    /// The entity must have the referenced art at or above the given score.
    /// Evaluated against the entity's max effective Art score (bought score plus
    /// virtue bonuses such as Puissant Art); an art the entity lacks counts as 0.
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
/// Every domain is resolved when a selection's parameter values are validated:
/// `Item` against the point-item registry, `Ability` against the ability
/// catalogue, `Art` against the art catalogue, and `Characteristic` by parsing
/// into [`crate::characteristics::Characteristic`]. A value that does not resolve
/// raises `unknown_param_value` (see `validation::validate_parameters`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterDomain {
    /// Value is an ability id (e.g. `ability.awareness`); resolved against the
    /// ability catalogue.
    Ability,
    /// Value is an art id (e.g. `art.creo`); resolved against the art catalogue.
    Art,
    /// Value is a characteristic id (e.g. `characteristic.str`). Validated by
    /// parsing into [`crate::characteristics::Characteristic`], not a registry.
    Characteristic,
    /// Value is a point-item id; resolved against the ruleset's point items.
    Item,
    /// Value is free text the player types (e.g. Aptitude for (Sin), Necessary
    /// (Realm) Aura, a (Land)). It references no registry, so any non-empty value
    /// is legal — the picker shows a text input rather than a dropdown.
    Text,
}

impl ParameterDomain {
    /// Returns `true` if values in this domain resolve against the ruleset's
    /// point-item registry specifically (i.e. `Item`). Other domains resolve
    /// against their own registries (ability / art catalogue, Characteristic
    /// parsing); see `validation::validate_parameters`.
    pub fn resolves_against_items(self) -> bool {
        matches!(self, ParameterDomain::Item)
    }
}

impl fmt::Display for ParameterDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParameterDomain::Ability => f.write_str("ability"),
            ParameterDomain::Art => f.write_str("art"),
            ParameterDomain::Characteristic => f.write_str("characteristic"),
            ParameterDomain::Item => f.write_str("item"),
            ParameterDomain::Text => f.write_str("text"),
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

/// A mechanical effect a virtue/flaw applies to a character's scores.
///
/// Effects are *parameter-relative*: each names the parameter key (see
/// [`ParameterDef::key`]) whose value on a [`Selection`] identifies the target
/// ability or characteristic. Bonus amounts and preconditions are data — the
/// engine hardcodes no virtue IDs. Effective scores are always computed from
/// these effects, never stored (see [`crate::effective`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    /// Adds `amount` to the effective score of the ability named by the
    /// selection's `params[param]` (e.g. Puissant Ability, +2).
    AbilityBonus {
        /// Parameter key whose value names the target ability.
        param: String,
        /// Points added to the effective score.
        amount: i8,
    },
    /// Shifts a base-score *limit* for the characteristic named by the
    /// selection's `params[param]`, by `amount` per selection. A positive amount
    /// raises the buy cap (Great Characteristic, +1 → up to +5); a negative
    /// amount lowers the buy floor (Poor Characteristic, −1 → down to −5). It
    /// grants no points: the score must still be bought/sold against the cost
    /// table. The "must already be at ±3" precondition is parameter-relative and
    /// derived from the ruleset's base cap/floor by the sign of `amount`, so it
    /// is enforced in validation rather than stored here.
    CharacteristicLimit {
        /// Parameter key whose value names the target characteristic.
        param: String,
        /// Limit shift per selection: positive raises the cap, negative lowers
        /// the floor.
        amount: i8,
    },
    /// Adds `amount` to the effective score of the Art named by the selection's
    /// `params[param]` (e.g. Puissant Art, +3). Arts are not parameterized, so the
    /// target is matched by Art id alone.
    ArtBonus {
        /// Parameter key whose value names the target Art.
        param: String,
        /// Points added to the effective score.
        amount: i8,
    },
    /// Affinity with (Ability): creation experience points put into the ability
    /// named by the selection's `params[param]` count as `counts_as_num /
    /// counts_as_den` of themselves (Affinity = 3/2, "increased by one half,
    /// rounded up"), so the XP *charged* against the pool for a bought score is
    /// `ceil(table_xp · counts_as_den / counts_as_num)`. A rational (two `u8`s),
    /// not a float, so [`Effect`] keeps deriving `Eq`. The age-cap exemption the
    /// same Virtue grants is read off the presence of this effect (Phase 6).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3372-3374.
    AffinityAbilityCost {
        /// Parameter key whose value names the target ability.
        param: String,
        /// Numerator of the "counts as" multiplier (Affinity = 3).
        counts_as_num: u8,
        /// Denominator of the "counts as" multiplier (Affinity = 2).
        counts_as_den: u8,
    },
    /// Affinity with (Art): the Art analogue of [`Effect::AffinityAbilityCost`],
    /// targeting the Art named by the selection's `params[param]`.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3376-3378.
    AffinityArtCost {
        /// Parameter key whose value names the target Art.
        param: String,
        /// Numerator of the "counts as" multiplier (Affinity = 3).
        counts_as_num: u8,
        /// Denominator of the "counts as" multiplier (Affinity = 2).
        counts_as_den: u8,
    },
    /// An Affinity applying to a fixed *group* of Abilities (matched by id, any
    /// instance), rather than a single player-chosen one. Linguist gives a 5/4
    /// Affinity to every Language (Living and Dead), so XP put into any language
    /// "counts as" `num/den` of itself, exactly like [`Self::AffinityAbilityCost`]
    /// but auto-applied to the whole group.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4315-4317 (Linguist).
    GroupAffinityCost {
        /// The Ability ids the Affinity covers (Linguist: living + dead language).
        abilities: std::collections::BTreeSet<Id>,
        /// Numerator of the "counts as" multiplier (Linguist = 5).
        counts_as_num: u8,
        /// Denominator of the "counts as" multiplier (Linguist = 4).
        counts_as_den: u8,
    },
    /// A restricted pool of experience points, spendable only on Abilities (never
    /// Arts) the grant is eligible for: an ability qualifies if its id is in
    /// `abilities` **or** its category is in `categories`. The general
    /// [`Entity::xp_pool`] still covers anything; unused restricted XP is wasted.
    /// Stacks across selections. Educated (specific ids), Warrior / Privileged
    /// Upbringing (categories).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3711-3713
    /// (Educated), `:5227-5229` (Warrior), `:4806-4808` (Privileged Upbringing).
    RestrictedAbilityXp {
        /// Points granted to this restricted pool.
        amount: u32,
        /// Eligible ability ids (Educated: Latin + Artes Liberales).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        abilities: Vec<Id>,
        /// Eligible ability categories (Warrior: Martial; Privileged: General,
        /// Academic, Martial).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        categories: Vec<AbilityCategory>,
    },
    /// Adjusts the Characteristic-buy budget by `amount` (on top of
    /// [`crate::characteristics::CharacteristicRules::start_points`]). Signed:
    /// Improved Characteristics grants +3 (`:4103-4105`), Weak Characteristics
    /// removes 3 (`:7056-7058`); both stack, so the grants from every matching
    /// selection are summed.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4103-4105 (Improved),
    /// `:7056-7058` (Weak).
    CharacteristicPoints {
        /// Points added to (or, when negative, removed from) the characteristic
        /// budget per selection.
        amount: i8,
    },
    /// Grants a free bought-score *floor* of `amount` in a fixed `ability`,
    /// costing no experience: the effective score is `max(bought, amount) +
    /// bonuses`, and an ability with no bought row still shows the granted score
    /// at 0 XP. The target is fixed by the virtue (Second Sight always confers
    /// Second Sight 1), not player-chosen — so the ability id is stored directly,
    /// not read from a selection parameter. (Phase 4 Mystery Houses reuse this to
    /// seed a Supernatural Ability at 1.)
    AbilityScoreGrant {
        /// The ability granted a free starting score.
        ability: Id,
        /// The free bought-score floor granted.
        amount: u8,
    },
    /// Adds `amount` levels to the magus's spell-levels budget (on top of the
    /// type profile's [`PointBudget`]-adjacent `spell_levels`). Signed: Skilled
    /// Parens grants +30, Weak Parens −30. The grants from every matching
    /// selection are summed and the total is clamped at 0.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4964-4966 (Skilled
    /// Parens), `:7072-7074` (Weak Parens).
    SpellLevels {
        /// Spell levels added to the budget per selection (may be negative).
        amount: i16,
    },
    /// Adds `amount` experience points to the general apprenticeship XP pool
    /// (spendable on Arts *or* Abilities, on top of [`Entity::xp_pool`]). Signed:
    /// Skilled Parens grants +60, Weak Parens −60. Summed across selections and
    /// clamped at 0.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4964-4966 (Skilled
    /// Parens), `:7072-7074` (Weak Parens).
    GeneralXp {
        /// Experience points added to the general pool per selection (may be
        /// negative).
        amount: i16,
    },
    /// Adjusts the character's derived Confidence Score and Points (on top of the
    /// type profile's defaults). Signed and additive; e.g. Self-Confident grants
    /// `{ score: 1, points: 2 }` (raising the 1/3 default to 2/5). Confidence is
    /// never stored on the entity — it is `profile default + Σ this effect`.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4900-4902 (Self-Confident).
    ConfidenceBonus {
        /// Confidence Score added per selection.
        score: i8,
        /// Confidence Points added per selection.
        points: i8,
    },
    /// Grants `amount` experience points to spend on Spell Mastery Abilities (a
    /// restricted pool, distinct from the general/ability XP pools — mastery is
    /// spent per known spell). Mastered Spells grants 50, stackable.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4471-4474.
    SpellMasteryXp {
        /// Mastery experience points granted per selection.
        amount: u16,
    },
    /// Floors the Spell Mastery Ability score of *every* known spell at `score`.
    /// Flawless Magic auto-masters every spell learned (Mastery 1); the effective
    /// mastery of a spell is `max(bought, this floor)`.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3887-3889.
    GrantsSpellMastery {
        /// The mastery-score floor granted to every known spell.
        score: u8,
    },
    /// Grants the listed Virtues/Flaws for free (budget-exempt), folded into the
    /// entity's derived grants like a House grant. A fixed nested grant — e.g.
    /// Templar Commander "grants the Temporal Influence Minor Virtue" and
    /// "includes the effects of the Brother-Knight Virtue". Each id must resolve
    /// to a point item. Only bought selections are scanned for this effect (one
    /// level of nesting; a granted item's own `grants_selection` is not applied).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:5113-5116 (Templar
    /// Commander).
    GrantsSelection {
        /// The Virtue/Flaw ids granted for free.
        items: std::collections::BTreeSet<Id>,
    },
    /// Grants `amount` starting levels of enchanted devices (base 0, summed).
    /// Magic Items grants +25 (stackable), Redcap 50.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4347-4349 (Magic
    /// Items), `:4842-4846` (Redcap).
    ItemLevelBudget {
        /// Levels of enchanted devices added.
        amount: u16,
    },
    /// Grants a derived True Faith Score (base 0, summed across grants). True
    /// Faith is a special score with its own rules, not a Supernatural Ability.
    /// The True Faith Virtue confers Score 1.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:5169-5171.
    TrueFaithGrant {
        /// True Faith Score added.
        score: u8,
    },
    /// Grants a derived Warping Score and Warping Points (base 0 each, summed
    /// across grants). Warped by Magic confers Warping Score 1 + 5 Warping Points.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:7019-7021.
    WarpingGrant {
        /// Warping Score added.
        score: u8,
        /// Warping Points added.
        points: u8,
    },
    /// Adds `amount` to the character's derived Size (base 0). Size is not a
    /// bought Characteristic; it is a separate racial stat modified only by these
    /// grants (Large +1, Giant Blood +2, Small Frame −1, Dwarf −2). Summed across
    /// selections.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3975-3978 (Giant
    /// Blood +2), `:4229-4231` (Large +1), `:6767-6769` (Small Frame −1),
    /// `:5996-5998` (Dwarf −2).
    SizeDelta {
        /// Size adjustment per selection (may be negative).
        amount: i8,
    },
    /// Adds a free `amount` bonus to the effective score of a fixed Characteristic
    /// (`characteristic.<slug>`), costing no buy points. Unlike the bought score
    /// (capped ±3, or ±5 with Great/Poor), this bonus stacks on top and may raise
    /// the effective score beyond the normal ceiling — Giant Blood's +1 to
    /// Strength/Stamina "may raise your scores … as high as +6". The target is
    /// fixed by the virtue, so the id is stored directly (not read from a param).
    /// Summed across selections.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3975-3978 (Giant
    /// Blood, +1 Str/Sta to +6), `:5996-5998` (Dwarf, −1 Str/Sta to −6).
    CharacteristicScoreDelta {
        /// The Characteristic id (`characteristic.str`, …) this bonus targets.
        characteristic: Id,
        /// The free effective-score bonus per selection (may be negative).
        amount: i8,
    },
    /// Authorizes the character to start with one Reputation of the given `kind`
    /// at the given `score` (content is player-supplied). A starting Reputation is
    /// legal only if backed by such a grant (Core:2514).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:6310-6312 (Infamous),
    /// `:5703-5705` (Black Sheep).
    GrantsReputation {
        /// Which audience the granted Reputation reaches.
        kind: ReputationType,
        /// The level of the granted Reputation.
        score: u8,
    },
}

/// The audience a Reputation reaches — a fixed rules taxonomy (so an enum, like
/// [`crate::art::ArtType`]), rendered via Fluent, never as a raw slug.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:1091-1101.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReputationType {
    /// Known to those who live near the character (the default).
    Local,
    /// Known within the Church.
    Ecclesiastical,
    /// Known within the Order of Hermes.
    Hermetic,
}

impl ReputationType {
    /// All types in book order (the single source of the serialized ordering).
    pub const ALL: [ReputationType; 3] = [
        ReputationType::Local,
        ReputationType::Ecclesiastical,
        ReputationType::Hermetic,
    ];
}

impl std::fmt::Display for ReputationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ReputationType::Local => "local",
            ReputationType::Ecclesiastical => "ecclesiastical",
            ReputationType::Hermetic => "hermetic",
        })
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
    /// How this V/F impacts a character mechanically (M5 slice 5a). Required (no
    /// serde default): an unclassified entry fails to load. See [`Classification`].
    pub classification: Classification,
    /// The descriptor's optional "Type" tag. `true` for a Tainted Virtue/Flaw:
    /// associated with the Infernal realm, and any Supernatural Ability it grants
    /// is an Infernal power. Drives the half-of-taken-points Tainted cap
    /// (no more than half a character's Virtue points — and likewise Flaw points —
    /// may be Tainted).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2998-3002.
    #[serde(default, skip_serializing_if = "is_false")]
    pub tainted: bool,
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
    /// Mechanical effects this item applies (e.g. Puissant Ability +2, Great
    /// Characteristic raising a buy cap). Empty for items with no effect.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<Effect>,
    /// Maximum number of selections that may share the same `(id, params)`
    /// target. Default 1 — an item may be taken at most once per distinct
    /// target; Great Characteristic raises this to 2 per characteristic.
    #[serde(
        default = "default_max_per_target",
        skip_serializing_if = "is_default_max_per_target"
    )]
    pub max_per_target: u8,
    /// Provenance into the Markdown source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// The default selection multiplicity: an item may be taken once per target.
fn default_max_per_target() -> u8 {
    1
}

fn is_default_max_per_target(value: &u8) -> bool {
    *value == default_max_per_target()
}

/// The default virtue/flaw conversion: one Flaw point funds one Virtue point.
fn default_virtue_points_per_flaw_point() -> u8 {
    1
}

fn is_default_virtue_points_per_flaw_point(value: &u8) -> bool {
    *value == default_virtue_points_per_flaw_point()
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
    /// How many virtue points each flaw point funds. Default 1 (one Flaw point
    /// buys one Virtue point). Mythic Companions get 2 (each Flaw point is worth
    /// two Virtue points). Data-driven so the engine never hardcodes a type.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2638 ("you may
    /// take up to ten points of Flaws, and each point of Flaws is worth two
    /// points of Virtues. This produces a maximum of 21 points of Virtues and 10
    /// points of Flaws").
    #[serde(
        default = "default_virtue_points_per_flaw_point",
        skip_serializing_if = "is_default_virtue_points_per_flaw_point"
    )]
    pub virtue_points_per_flaw_point: u8,
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
    pub flaw_category_caps: Vec<CategoryCap>,
    /// Per-category *virtue* count caps. Structurally identical to
    /// `flaw_category_caps` but counts `Virtue`-kind items. The magus type uses
    /// this for the `≤1 Major Hermetic Virtue` rule.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2855-2861 ("You
    /// may take a maximum of one Major Hermetic Virtue during character
    /// creation").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub virtue_category_caps: Vec<CategoryCap>,
}

/// A cap on how many items of a given category an entity may take. Shared by
/// `flaw_category_caps` and `virtue_category_caps`; the kind counted is fixed by
/// which list the cap lives in, not by the cap itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryCap {
    /// The category this cap applies to (e.g. `personality`, `story`,
    /// `hermetic`).
    pub category: String,
    /// Maximum allowed count.
    pub max: u8,
    /// If true, only Major-magnitude items count toward this cap.
    #[serde(default, skip_serializing_if = "is_false")]
    pub major_only: bool,
    /// If true the cap is a blocking error; otherwise a non-blocking warning.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hard: bool,
}

/// `skip_serializing_if` predicate: omits a `bool` field from canonical JSON
/// when it holds its `false` default, keeping the common case out of the data.
pub(crate) fn is_false(b: &bool) -> bool {
    !*b
}

/// Data-driven profile defining constraints for an entity type
/// (grog, companion, magus, etc.).
///
/// As with [`Entity`], the character-only fields here (`is_magus`,
/// `gift_policy`, `gift_id`, `gift_categories`) live flat on this generic
/// profile type as a deliberate KISS trade-off rather than in a separate
/// character-specific profile. A covenant type simply leaves them
/// empty/default/`None`; the engine never assumes a covenant profile populates
/// them.
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
    /// Whether this character type chooses a Mythic Companion *type* (which
    /// confers a free status/Minor Virtue and a required V/F package). A
    /// capability flag parallel to `is_magus`; the type selector and
    /// `validate_mythic_type` read it, never a hardcoded type id. Defaults to
    /// false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_mythic_type: bool,
    /// The magus's starting spell-levels budget (the sum of the levels of spells
    /// he may know at creation). 120 for the magus profile (Core Rules.md:2215-2216,
    /// 2435); 0 (omitted) for every non-magus type, which cannot take spells.
    /// Modified per-character by [`Effect::SpellLevels`] (Skilled/Weak Parens).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub spell_levels: u32,
    /// The character type's starting Confidence Score (Core:2521). Companions,
    /// magi and mythic companions start at 1; grogs have no Confidence (0/omitted).
    /// The effective score folds in `Effect::ConfidenceBonus`; Confidence is
    /// derived, never stored on the entity.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub confidence_score: u8,
    /// The character type's starting Confidence Points (Core:2521): 3 for
    /// companions/magi/mythic, 0 for grogs.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub confidence_points: u8,
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
    // Order-significant (the wizard walks them in sequence): intentionally
    // exempt from `normalize`'s canonical sorting.
    pub creation_phases: Vec<String>,
}

impl EntityTypeProfile {
    /// Sorts the profile's unordered nested vectors for canonical
    /// serialization. The category/trait sets are `BTreeSet`s (already
    /// id-ordered), and `creation_phases` is order-significant and so left
    /// untouched; the unordered vectors are the budget's `flaw_category_caps`
    /// and `virtue_category_caps`, sorted here by category.
    pub fn normalize(&mut self) {
        self.budget
            .flaw_category_caps
            .sort_by(|a, b| a.category.cmp(&b.category));
        self.budget
            .virtue_category_caps
            .sort_by(|a, b| a.category.cmp(&b.category));
    }
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

/// A character's whole bought score in one Ability, with an optional specialty.
///
/// The score is the *bought* value (ability XP is spent in whole points, so an
/// ability never holds partial XP — leftover XP is the pool minus what scores
/// cost, see [`Entity::xp_pool`]). The
/// *effective* score (bought + virtue bonuses) is computed at validation time,
/// never stored. Keyed by (ability, specialty): the same parameterized Ability
/// (e.g. Area Lore, Living Language) can appear more than once with different
/// specialties.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AbilityScore {
    /// The ability's id (e.g. `ability.awareness`).
    pub ability: Id,
    /// The whole bought score.
    pub score: u8,
    /// Free-text specialty (e.g. a focus like "searching" for Awareness).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specialty: Option<String>,
    /// Player-supplied value for a parameterized ability (e.g. the area for
    /// `(Area) Lore` or the language for `(Living Language)`). Part of the
    /// ability's identity: instances with different parameters are distinct, so a
    /// character may hold several `(Area) Lore`s. `None` for plain abilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
}

/// A whole bought Hermetic Art score. Arts are not parameterized and carry no
/// specialty, so an instance is identified by `art` alone. Like Abilities, the XP
/// to reach the score is priced from the ruleset's *Art* advancement table (a
/// separate, cheaper curve), and the leftover XP banks against
/// [`Entity::art_xp_pool`]. The *effective* score (bought + Puissant Art) is
/// computed at validation time, never stored.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArtScore {
    /// The Art's id (e.g. `art.creo`).
    pub art: Id,
    /// The whole bought score.
    pub score: u8,
}

/// A spell the character knows (magi only). Saves store the choice, not the
/// resolved value: the catalogue supplies a fixed spell's level, so `level` is
/// `Some` only for a **General** spell — the per-character learned level. Two
/// General versions of one spell at different levels are distinct spells
/// (Core Rules.md:12349-12353), so identity is (spell, level). Kept sorted via
/// [`Entity::normalize`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SpellSelection {
    /// The spell's id (e.g. `spell.pilum_of_fire`).
    pub spell: Id,
    /// The learned level for a General spell; `None` for a fixed-level spell (the
    /// catalogue level is authoritative — a stray value here is ignored at eval).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    /// The bought Spell Mastery Ability score for this spell, spent from the
    /// mastery-XP pool (Mastered Spells). `None`/0 = unmastered. The effective
    /// mastery is `max(this, granted floor)` — Flawless Magic floors every spell
    /// at 1. Source: Core Rules.md:4471-4474, :3887-3889.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mastery: Option<u8>,
}

/// A named Personality Trait with a value in −3..+3 (or ±6 for the trait
/// representing a Major Personality Flaw). Free-text name, kept sorted by name in
/// [`Entity::normalize`]. Source: Core Rules.md:2500-2503.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PersonalityTrait {
    /// Free-text trait name (e.g. "Brave", "Loyal").
    pub name: String,
    /// Trait value; ±3 normally, ±6 for a Major Personality Flaw's trait.
    pub value: i8,
}

/// A starting Reputation: score + free-text content + audience type. Only legal
/// when backed by a granting Virtue/Flaw ([`Effect::GrantsReputation`]).
/// Source: Core Rules.md:1091-1101, 2512-2514.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Reputation {
    /// Which audience the Reputation reaches.
    pub kind: ReputationType,
    /// The Reputation's level.
    pub score: u8,
    /// Free-text description of what the Reputation is for.
    pub content: String,
}

/// `skip_serializing_if` predicate: omits a `u32` field when it is zero.
fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// `skip_serializing_if` predicate: omits a `u8` field when it is zero.
fn is_zero_u8(n: &u8) -> bool {
    *n == 0
}

/// The save format for a character or covenant under construction.
///
/// Saves store choices, not resolved values; `selections` and `ability_scores`
/// are kept sorted on serialization (see [`Entity::normalize`]) for zero-noise
/// git diffs.
///
/// # Flat schema for character-only fields (deliberate KISS trade-off)
///
/// `Entity` is the single generic buildable-entity type shared by characters and
/// covenants (see the entity-generic design invariant). The character-only
/// fields — `characteristics`, `characteristic_descriptions`, `ability_scores`,
/// `xp_pool` — live flat on this generic type rather than in a separate
/// character-specific struct. This is intentional, not an oversight: it keeps one
/// concrete save shape and one validation path. A covenant entity simply leaves
/// these fields empty/default (and the engine gates the character-only
/// validators on `entity_kind`, so it never assumes a covenant carries them).
/// Mirror trade-off on [`EntityTypeProfile`] for the type-level fields.
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
    /// The user's virtue/flaw selections. Kept sorted via [`Entity::normalize`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selections: Vec<Selection>,
    /// Chosen Characteristic scores (point-buy). Defaults to empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub characteristics: BTreeMap<Characteristic, i8>,
    /// Optional free-text description per Characteristic (the character sheet's
    /// "description" field — flavor, not a rules mechanic). Defaults to empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub characteristic_descriptions: BTreeMap<Characteristic, String>,
    /// Whole bought Ability scores. Kept sorted via [`Entity::normalize`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ability_scores: Vec<AbilityScore>,
    /// Total experience points available to spend on Abilities **and** Arts —
    /// one shared bank (the rules give apprenticeship XP as a single pool the
    /// magus splits freely between the two). The amount *spent* is derived
    /// (Σ XP-to-reach each bought Ability score, priced from the Ability
    /// advancement table, plus each bought Art score, priced from the Art table);
    /// the leftover (`xp_pool` − spent) is the character's banked XP. Spending
    /// more than the pool is an error (surfaced in Advisory/Enforced modes); the
    /// M4 life-stage flow sets this pool and blocks overspending up front.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub xp_pool: u32,
    /// Whole bought Hermetic Art scores (magi only). Kept sorted via
    /// [`Entity::normalize`]. Defaults to empty. Priced from the Art advancement
    /// table against the shared [`Entity::xp_pool`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub art_scores: Vec<ArtScore>,
    /// The spells the character knows (magi only). Each entry consumes the magus's
    /// spell-levels budget; kept sorted via [`Entity::normalize`]. Defaults to
    /// empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spells: Vec<SpellSelection>,
    /// The Hermetic House this entity belongs to (magi only). The save stores
    /// only the choice; the free House Virtue is derived at eval time, never
    /// persisted (honors "saves store choices, not resolved values"). `None` for
    /// non-magi and for a magus who has not yet picked a House.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub house: Option<Id>,
    /// The House specialisation picks, keyed by each grant's `choice_key` (e.g.
    /// Flambeau's Puissant Perdo-or-Ignem choice, or an Ex Miscellanea open
    /// grant). Empty when the House has no player choices. The derived grant
    /// resolver reads these to emit the granted selections.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub house_choices: BTreeMap<String, Selection>,
    /// The Mythic Companion type this entity is (mythic companions only). The
    /// save stores only the choice; the free status/Minor Virtue is derived at
    /// eval time, never persisted (honors "saves store choices, not resolved
    /// values"). `None` for non-mythic-companions and for one not yet chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mythic_type: Option<Id>,
    /// The Mythic Companion type's grant picks, keyed by each grant's
    /// `choice_key` (e.g. Devil Child's Demonic Might-or-Powers choice). Empty
    /// when the type has no player choices. The derived grant resolver reads
    /// these to emit the granted selections. Parallel to `house_choices`; the two
    /// are mutually exclusive by profile (a character is a magus or a mythic
    /// companion, never both).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mythic_choices: BTreeMap<String, Selection>,
    /// The character's age in years. Drives the age → max-Ability-score cap
    /// (Core:2366-2376). `None` when unset (no cap enforced yet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// Named Personality Traits (value ±3, or ±6 for a Major Personality Flaw's
    /// trait). Kept sorted by name via [`Entity::normalize`]. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub personality_traits: Vec<PersonalityTrait>,
    /// Starting Reputations (each backed by a granting Virtue/Flaw). Kept sorted
    /// via [`Entity::normalize`]. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reputations: Vec<Reputation>,
}

/// Current save-format schema version.
pub const SCHEMA_VERSION: u32 = 8;

impl Entity {
    /// Creates a new entity at the current [`SCHEMA_VERSION`] with empty trait
    /// data.
    pub fn new(entity_kind: EntityKind, type_id: Id, ruleset_ref: RulesetRef) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            ruleset: ruleset_ref,
            entity_kind,
            type_id,
            selections: Vec::new(),
            characteristics: BTreeMap::new(),
            characteristic_descriptions: BTreeMap::new(),
            ability_scores: Vec::new(),
            xp_pool: 0,
            art_scores: Vec::new(),
            spells: Vec::new(),
            house: None,
            house_choices: BTreeMap::new(),
            mythic_type: None,
            mythic_choices: BTreeMap::new(),
            age: None,
            personality_traits: Vec::new(),
            reputations: Vec::new(),
        }
    }

    /// Sort selections, ability scores, art scores, spells, personality traits and
    /// reputations for canonical serialization. (`characteristics` is a
    /// `BTreeMap`, already id-ordered.)
    pub fn normalize(&mut self) {
        self.selections.sort();
        self.ability_scores.sort();
        self.art_scores.sort();
        self.spells.sort();
        self.personality_traits.sort();
        self.reputations.sort();
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
    /// Two-letter Art abbreviation (e.g. "Cr"). Present only for Arts; `None`
    /// for every other item kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,
    /// Example specialties for the item (e.g. an Ability's example
    /// specializations). Empty when none are given.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub specialties: Vec<String>,
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
        check(Classification::Narrative);
        check(Classification::CreationEffect);
        check(Classification::InPlayEffect);
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
        check(ParameterDomain::Characteristic);
        check(crate::validation::IssueSeverity::Error);
        check(crate::validation::IssueSeverity::Warning);
        check(crate::spell::SpellRange::ArcaneConnection);
        check(crate::spell::SpellRange::Personal);
        check(crate::spell::SpellDuration::Year);
        check(crate::spell::SpellDuration::Momentary);
        check(crate::spell::SpellTarget::Boundary);
        check(crate::spell::SpellTarget::Vision);
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
          "classification": "narrative",
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
        assert_eq!(item.classification, Classification::Narrative);
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
          "classification": "narrative",
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
              "classification": "narrative",
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
    fn entity_type_profile_normalize_sorts_flaw_category_caps() {
        let mut profile: EntityTypeProfile = serde_json::from_str(
            r#"{
              "id": "companion",
              "budget": {
                "virtue_points": 10,
                "flaw_points": 10,
                "flaw_category_caps": [
                  { "category": "story", "max": 1 },
                  { "category": "personality", "max": 2 }
                ]
              },
              "creation_phases": ["concept", "boons_hooks"]
            }"#,
        )
        .unwrap();
        profile.normalize();
        let cats: Vec<&str> = profile
            .budget
            .flaw_category_caps
            .iter()
            .map(|c| c.category.as_str())
            .collect();
        assert_eq!(cats, vec!["personality", "story"]);
        // Order-significant phases stay in their declared order.
        assert_eq!(profile.creation_phases, vec!["concept", "boons_hooks"]);
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
    fn prereq_nor_variant() {
        let json =
            r#"{ "kind": "none", "value": [{ "kind": "has", "value": "flaw.blatant_gift" }] }"#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(
            prereq,
            Prereq::Nor(vec![Prereq::Has(Id::new("flaw.blatant_gift"))])
        );
    }

    #[test]
    fn prereq_nor_serializes_with_none_tag() {
        // The variant is named `Nor` in Rust, but the JSON tag MUST remain
        // `"none"` so saved data and the frontend contract stay stable.
        let prereq = Prereq::Nor(vec![Prereq::Has(Id::new("flaw.blatant_gift"))]);
        let json = serde_json::to_string(&prereq).unwrap();
        assert!(
            json.contains(r#""kind":"none""#),
            "serde tag must stay \"none\": {json}"
        );
        let roundtripped: Prereq = serde_json::from_str(&json).unwrap();
        assert_eq!(prereq, roundtripped);
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
            schema_version: SCHEMA_VERSION,
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
            characteristics: BTreeMap::from([(Characteristic::Int, 2), (Characteristic::Sta, -1)]),
            characteristic_descriptions: BTreeMap::new(),
            ability_scores: vec![AbilityScore {
                ability: Id::new("ability.awareness"),
                score: 3,
                specialty: Some("searching".into()),
                parameter: None,
            }],
            xp_pool: 30,
            art_scores: vec![ArtScore {
                art: Id::new("art.creo"),
                score: 5,
            }],
            spells: vec![SpellSelection {
                spell: Id::new("spell.pilum_of_fire"),
                level: None,
                mastery: None,
            }],
            house: None,
            house_choices: BTreeMap::new(),
            mythic_type: None,
            mythic_choices: BTreeMap::new(),
            age: Some(25),
            personality_traits: vec![PersonalityTrait {
                name: "Brave".into(),
                value: 3,
            }],
            reputations: Vec::new(),
        };

        let json = serde_json::to_string_pretty(&entity).unwrap();
        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);

        assert!(json.contains(r#""schema_version": 8"#));
        assert!(json.contains(r#""ref": "flaw.deficient_technique""#));
        assert!(json.contains(r#""xp_pool": 30"#));
        assert!(json.contains(r#""art": "art.creo""#));
        assert!(json.contains(r#""spell": "spell.pilum_of_fire""#));
        assert!(json.contains(r#""age": 25"#));
        assert!(json.contains(r#""name": "Brave""#));
    }

    /// Reputations round-trip with a snake_case `kind`, and `normalize` sorts both
    /// personality traits (by name) and reputations canonically.
    #[test]
    fn entity_reputations_and_personality_normalize_roundtrip() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.personality_traits = vec![
            PersonalityTrait {
                name: "Loyal".into(),
                value: 2,
            },
            PersonalityTrait {
                name: "Brave".into(),
                value: -1,
            },
        ];
        entity.reputations = vec![Reputation {
            kind: ReputationType::Hermetic,
            score: 3,
            content: "Dedicated Hoplite".into(),
        }];
        entity.normalize();
        // Traits sorted by name: Brave before Loyal.
        assert_eq!(entity.personality_traits[0].name, "Brave");

        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains(r#""kind":"hermetic""#), "{json}");
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
    }

    /// ReputationType serializes to its snake_case scalar for every variant.
    #[test]
    fn reputation_type_serde_roundtrip() {
        for kind in ReputationType::ALL {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(serde_json::from_str::<ReputationType>(&json).unwrap(), kind);
        }
        assert_eq!(
            serde_json::to_string(&ReputationType::Ecclesiastical).unwrap(),
            r#""ecclesiastical""#
        );
    }

    /// A General spell round-trips its chosen level, and `normalize` sorts the
    /// spell list canonically (by spell id, then level).
    #[test]
    fn entity_spells_normalize_and_general_level_roundtrip() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.spells = vec![
            SpellSelection {
                spell: Id::new("spell.unseen_arm"),
                level: None,
                mastery: None,
            },
            SpellSelection {
                spell: Id::new("spell.aegis_of_the_hearth"),
                level: Some(20),
                mastery: None,
            },
        ];
        entity.normalize();
        // Sorted by spell id: aegis before unseen_arm.
        assert_eq!(entity.spells[0].spell, Id::new("spell.aegis_of_the_hearth"));
        assert_eq!(entity.spells[0].level, Some(20));

        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
        assert!(json.contains(r#""level":20"#));
    }

    #[test]
    fn entity_serializes_selections_sorted() {
        let entity = Entity {
            schema_version: SCHEMA_VERSION,
            ruleset: RulesetRef::new(Id::new("arm5-core"), "2024.1"),
            entity_kind: EntityKind::Character,
            type_id: Id::new("companion"),
            selections: vec![
                Selection::new(Id::new("virtue.tough")),
                Selection::new(Id::new("flaw.poor_student")),
                Selection::new(Id::new("ability.awareness")),
            ],
            characteristics: BTreeMap::new(),
            characteristic_descriptions: BTreeMap::new(),
            ability_scores: Vec::new(),
            xp_pool: 0,
            art_scores: Vec::new(),
            spells: Vec::new(),
            house: None,
            house_choices: BTreeMap::new(),
            mythic_type: None,
            mythic_choices: BTreeMap::new(),
            age: None,
            personality_traits: Vec::new(),
            reputations: Vec::new(),
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
        let mut entity = Entity::new(
            EntityKind::Covenant,
            Id::new("standard_covenant"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.selections = vec![Selection::new(Id::new("boon.healthy_feature"))];

        let json = serde_json::to_string_pretty(&entity).unwrap();
        assert!(json.contains(r#""entity_kind": "covenant""#));

        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);
    }

    #[test]
    fn house_and_choices_roundtrip_and_default_absent() {
        // A magus save carries a House plus its specialisation picks, keyed by
        // grant `choice_key`. Both must survive a serde round-trip.
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.house = Some(Id::new("house.flambeau"));
        entity.house_choices = BTreeMap::from([(
            "flambeau_puissant".to_string(),
            Selection::with_params(
                Id::new("virtue.puissant_art"),
                BTreeMap::from([("art".into(), Id::new("art.ignem"))]),
            ),
        )]);

        let json = serde_json::to_string_pretty(&entity).unwrap();
        assert!(json.contains(r#""house": "house.flambeau""#));
        assert!(json.contains(r#""flambeau_puissant""#));

        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);

        // A fresh non-magus entity leaves both empty and omits them from JSON.
        let fresh = Entity::new(
            EntityKind::Covenant,
            Id::new("covenant"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        assert_eq!(fresh.house, None);
        assert!(fresh.house_choices.is_empty());
        let fresh_json = serde_json::to_string(&fresh).unwrap();
        assert!(!fresh_json.contains("house"));
    }

    #[test]
    fn v1_save_without_new_fields_still_loads() {
        // A schema_version 1 save predates characteristics/ability_scores/xp_pool.
        let v1 = r#"{
          "schema_version": 1,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "selections": [{ "ref": "virtue.tough" }]
        }"#;
        let entity: Entity = serde_json::from_str(v1).unwrap();
        assert_eq!(entity.schema_version, 1);
        assert!(entity.characteristics.is_empty());
        assert!(entity.ability_scores.is_empty());
        assert_eq!(entity.xp_pool, 0);
    }

    #[test]
    fn empty_trait_fields_are_omitted_from_json() {
        let entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        let json = serde_json::to_string(&entity).unwrap();
        assert!(!json.contains("characteristics"));
        assert!(!json.contains("ability_scores"));
        assert!(!json.contains("xp_pool"));
    }

    #[test]
    fn entity_serializes_ability_scores_sorted() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.ability_scores = vec![
            AbilityScore {
                ability: Id::new("ability.swim"),
                score: 2,
                specialty: None,
                parameter: None,
            },
            AbilityScore {
                ability: Id::new("ability.awareness"),
                score: 3,
                specialty: None,
                parameter: None,
            },
        ];
        entity.normalize();
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.find("ability.awareness").unwrap() < json.find("ability.swim").unwrap());
    }

    #[test]
    fn boon_hook_deserialization() {
        let boon_json = r#"{
          "id": "boon.healthy_feature",
          "kind": "boon",
          "classification": "narrative",
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
          "classification": "narrative",
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
    fn effects_and_max_per_target_deserialize() {
        let json = r#"{
          "id": "virtue.great_characteristic",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "minor",
          "category": "general",
          "entity_kinds": ["character"],
          "parameters": [{ "key": "characteristic", "type": "ref", "domain": "characteristic" }],
          "effects": [{ "type": "characteristic_limit", "param": "characteristic", "amount": 1 }],
          "max_per_target": 2
        }"#;
        let item: PointItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.max_per_target, 2);
        assert_eq!(
            item.effects,
            vec![Effect::CharacteristicLimit {
                param: "characteristic".into(),
                amount: 1,
            }]
        );
    }

    #[test]
    fn max_per_target_defaults_to_one_and_is_omitted_when_default() {
        let json = r#"{
          "id": "virtue.keen_vision",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "minor",
          "category": "general"
        }"#;
        let item: PointItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.max_per_target, 1);
        assert!(item.effects.is_empty());
        // The default must not appear in canonical output (zero-noise diffs).
        let out = serde_json::to_string(&item).unwrap();
        assert!(
            !out.contains("max_per_target"),
            "default should be skipped: {out}"
        );
        assert!(
            !out.contains("effects"),
            "empty effects should be skipped: {out}"
        );
    }

    #[test]
    fn ability_bonus_effect_roundtrips() {
        let effect = Effect::AbilityBonus {
            param: "ability".into(),
            amount: 2,
        };
        let json = serde_json::to_string(&effect).unwrap();
        assert_eq!(serde_json::from_str::<Effect>(&json).unwrap(), effect);
        assert!(json.contains("\"type\":\"ability_bonus\""));
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
        assert_eq!(
            format!("{}", ParameterDomain::Characteristic),
            "characteristic"
        );
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
    fn string_from_id() {
        let id = Id::new("test.id");
        let s: String = id.into();
        assert_eq!(s, "test.id");
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
        assert_eq!(entity.schema_version, SCHEMA_VERSION);
        assert_eq!(entity.entity_kind, EntityKind::Character);
        assert_eq!(entity.type_id, Id::new("companion"));
        assert!(entity.selections.is_empty());
        assert!(entity.characteristics.is_empty());
        assert!(entity.ability_scores.is_empty());
        assert_eq!(entity.xp_pool, 0);
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
