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
use crate::aging::CrisisSeverity;
use crate::characteristics::Characteristic;
use crate::life_stage::LifeStagePlan;

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
/// ```
///
/// The `"none"` tag is the boolean **NOR** operator (the `Nor` variant): it is
/// satisfied only when *none* of its child prereqs are present. It does **not**
/// mean "no prerequisite" — an entity with no prereqs simply omits the field
/// entirely. The tag stays `"none"` because it is a locked wire contract.
///
/// ```json
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

impl Prereq {
    /// Whether this prerequisite is *definitely* unsatisfiable for a character
    /// belonging to `house`, judging [`Prereq::House`] leaves alone.
    ///
    /// This is deliberately **not** the full evaluator
    /// (`validation::prereq::evaluate_prereq`), which needs an entity, a ruleset,
    /// and a type profile. It answers the narrower question an *open grant menu*
    /// asks: "could this item ever be legal for a character of this House?" —
    /// which depends on the expression and the House and nothing else. Every
    /// non-House leaf is therefore treated as undecided rather than as false, so
    /// a `Has`/`AbilityMin`/`ArtMin`/`IsMagus` prerequisite never excludes an
    /// item from a menu: those resolve as the build progresses, and dropping
    /// them would be an order-dependent exclusion, harsher than the
    /// error-that-resolves model the engine uses everywhere else. A House does
    /// not resolve that way — a magus has exactly one, and picking a Virtue that
    /// confers a *different* House is not something a later step can fix.
    ///
    /// An unknown house (`None`) leaves every House leaf undecided too, mirroring
    /// the full evaluator's `Unknown` for a magus who has not chosen one yet.
    pub fn conflicts_with_house(&self, house: Option<&Id>) -> bool {
        self.house_only_value(house, 1) == Some(false)
    }

    /// Tri-state evaluation (`None` = undecided) of this expression against
    /// `house` alone. `depth` is 1 at the top level and increments once per
    /// `All`/`Any`/`Nor` level; past [`PREREQ_MAX_DEPTH`] the expression counts
    /// as undecided rather than recursing further — the same K8 defense in depth
    /// the full evaluator applies, and equally unreachable for any ruleset that
    /// passed `Ruleset::validate_prereq_refs` at load.
    fn house_only_value(&self, house: Option<&Id>, depth: usize) -> Option<bool> {
        if depth > PREREQ_MAX_DEPTH {
            return None;
        }
        match self {
            // AND: one false child sinks it; all-true makes it true.
            Prereq::All(children) => {
                Self::fold_house_only(children, house, depth, false, false, true)
            }
            // OR: one true child carries it; all-false makes it false.
            Prereq::Any(children) => {
                Self::fold_house_only(children, house, depth, true, true, false)
            }
            // NOR: one true child sinks it; all-false makes it true.
            Prereq::Nor(children) => {
                Self::fold_house_only(children, house, depth, true, false, true)
            }
            Prereq::House(required) => house.map(|h| h == required),
            // Everything else is outside this question's remit: undecided, so it
            // can neither exclude an item nor rescue one.
            Prereq::Has(_)
            | Prereq::AbilityMin { .. }
            | Prereq::ArtMin { .. }
            | Prereq::IsMagus => None,
        }
    }

    /// Tri-state fold shared by the three quantifiers: a child evaluating to
    /// `trigger` short-circuits the whole expression to `short_circuit`; with no
    /// trigger and no undecided child the result is `all_known`; anything else is
    /// undecided.
    fn fold_house_only(
        children: &[Prereq],
        house: Option<&Id>,
        depth: usize,
        trigger: bool,
        short_circuit: bool,
        all_known: bool,
    ) -> Option<bool> {
        let mut saw_undecided = false;
        for child in children {
            match child.house_only_value(house, depth + 1) {
                Some(value) if value == trigger => return Some(short_circuit),
                Some(_) => {}
                None => saw_undecided = true,
            }
        }
        (!saw_undecided).then_some(all_known)
    }
}

/// Engineering limit (not a rules mechanic — needs no source citation) on how
/// deeply a [`Prereq`] boolean expression may nest (K8).
///
/// Every prerequisite currently shipped in `rules/core/*.json` is a single
/// flat `Has` or `House` — depth 1 — so this is generous headroom for compound
/// `All`/`Any`/`Nor` trees the rules could reasonably grow into (a handful of
/// nested clauses), while staying far short of anything that could threaten
/// the stack. Without a bound, a `rules/` directory carrying a
/// pathologically deep `Prereq` tree (crafted or corrupted — the trust
/// boundary is the file the user opens, `CLAUDE.md` → "This is a DESKTOP
/// APPLICATION") recurses without limit in every walker over the tree.
///
/// Enforced at two points, both named in each site's own doc:
/// - `Ruleset::validate_prereq_refs` (load time, the primary guard): a
///   ruleset whose prerequisites nest past this limit never finishes
///   loading, via either `Ruleset::from_sources` or `Ruleset::from_serialized`
///   — so a malformed tree can never reach evaluation at all.
/// - `validation::prereq::evaluate_prereq` (evaluation time, defense in
///   depth): should be unreachable for any ruleset that passed the load-time
///   guard, but degrades to "unevaluable" rather than recursing further if it
///   is ever reached some other way.
/// - [`Prereq::conflicts_with_house`] (open-grant menu filtering), for the same
///   reason and with the same "undecided rather than recurse" degradation.
pub const PREREQ_MAX_DEPTH: usize = 32;

/// The kind of value a parameter slot carries.
///
/// A forward-looking, single-variant discriminant. Today it carries no behavior:
/// parameter validation keys entirely off [`ParameterDomain`] (see
/// `validation::selections::validate_parameters`), which `domain` already
/// subsumes — `Text` and `Characteristic` values are not entity refs at all.
/// The field therefore defaults to `Ref` and is optional in rules JSON; it is
/// retained (rather than deleted) so a future non-ref parameter kind can be
/// added without a wire change.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ParamType {
    /// The parameter value is a reference to another rules entity (an [`Id`]).
    #[default]
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
/// catalogue, `Art` against the art catalogue, `Characteristic` by parsing
/// into [`crate::characteristics::Characteristic`], and `Enumerated` against the
/// parameter definition's own [`ParameterDef::values`] list. A value that does not
/// resolve raises `unknown_param_value` (see `validation::validate_parameters`).
///
/// Adding a variant is **not** caught everywhere by the compiler: only the
/// [`fmt::Display`] impl below is an exhaustive `match`. [`Self::resolves_against_items`]
/// is a `matches!` that silently answers `false`, and the load-time domain checks in
/// `ruleset::integrity` compare with `==`/`!=`. Enumerate those sites by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterDomain {
    /// Value is an ability id (e.g. `ability.awareness`); resolved against the
    /// ability catalogue.
    Ability,
    /// Value is an art id (e.g. `art.creo`); resolved against the art catalogue.
    Art,
    /// Value is a **Technique** art id (e.g. `art.creo`); resolved against the art
    /// catalogue and additionally required to be a Technique (`ArtType::Technique`).
    /// Used by Deficient Technique so it cannot target a Form.
    Technique,
    /// Value is a **Form** art id (e.g. `art.ignem`); resolved against the art
    /// catalogue and additionally required to be a Form (`ArtType::Form`). Used by
    /// Deficient Form so it cannot target a Technique.
    Form,
    /// Value is a characteristic id (e.g. `characteristic.str`). Validated by
    /// parsing into [`crate::characteristics::Characteristic`], not a registry.
    Characteristic,
    /// Value is a point-item id; resolved against the ruleset's point items.
    Item,
    /// Value is one of a closed list the parameter itself declares in
    /// [`ParameterDef::values`] — the domain IS that list, so it lives in the
    /// rules data and no catalogue is consulted. Used where the book prints an
    /// exhaustive set of choices (Folk Magic's four spell categories, the
    /// (Beings) classes), which the picker then shows as a dropdown.
    Enumerated,
    /// Value is free text the player types (e.g. Aptitude for (Sin), Necessary
    /// (Realm) Aura, a (Land)). It references no registry, so any value with
    /// non-whitespace content is legal — the picker shows a text input rather than a
    /// dropdown. **Enforced**, not merely documented: an empty or whitespace-only
    /// value resolves to nothing and is reported as `missing_param`, the same issue
    /// an absent key raises, since a blank text box is a choice not yet made rather
    /// than an unknown value (there is nothing to print). Values are trimmed at
    /// load ([`load_entity_migrating`]) and at every write path, so a padded
    /// descriptor is the same choice as an unpadded one — but case is the player's,
    /// and is never folded.
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
            ParameterDomain::Technique => f.write_str("technique"),
            ParameterDomain::Form => f.write_str("form"),
            ParameterDomain::Characteristic => f.write_str("characteristic"),
            ParameterDomain::Item => f.write_str("item"),
            ParameterDomain::Enumerated => f.write_str("enumerated"),
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
    /// The kind of value the parameter carries. Optional in rules JSON: defaults
    /// to [`ParamType::Ref`], the only current variant (see [`ParamType`]).
    #[serde(rename = "type", default)]
    pub param_type: ParamType,
    /// The domain the parameter value's id must belong to.
    pub domain: ParameterDomain,
    /// The closed list of legal values, for [`ParameterDomain::Enumerated`] only.
    /// Absent from the JSON — and from the serialized form — for every other
    /// domain, where a list would be silently ignored; load-time integrity
    /// rejects both the empty list here and a list anywhere else.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<Id>,
}

impl ParameterDef {
    /// Creates a parameter definition. Use [`Self::enumerated`] for the one
    /// domain that carries its own value list.
    pub fn new(key: impl Into<String>, param_type: ParamType, domain: ParameterDomain) -> Self {
        Self {
            key: key.into(),
            param_type,
            domain,
            values: Vec::new(),
        }
    }

    /// Creates an [`ParameterDomain::Enumerated`] parameter over `values`.
    pub fn enumerated(key: impl Into<String>, values: impl IntoIterator<Item = Id>) -> Self {
        Self {
            key: key.into(),
            param_type: ParamType::Ref,
            domain: ParameterDomain::Enumerated,
            values: values.into_iter().collect(),
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
    /// **Replaces** the later-life experience rate: the character earns `amount`
    /// points per year of later life instead of the ruleset's base rate. Not
    /// additive — the rules state the whole rate ("Characters with the Wealthy
    /// Virtue get 20 experience points per year, while characters with the Poor
    /// Flaw get 10 experience points per year"). Both are Major and, per the same
    /// line, available to companions only; the profiles enforce that.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2394.
    LaterLifeXpRate {
        /// Experience points earned per year of later life.
        amount: u32,
    },
    /// Narrows the age → maximum-Ability-score cap to `num/den` of its normal value
    /// (rounded **up**) for Abilities the catalogue marks `locality_dependent`:
    ///
    /// > The maximum scores at character creation for locality-dependent Abilities
    /// > like Language, Area Lore, or Organization Lore, as well as some social
    /// > Abilities, are half (round up) that which his age normally allows.
    ///
    /// Applies to the *cap*, not the cost: such an Ability is bought at the usual
    /// price, just not as high. Source: Ars Magica - Definitive Edition (Core
    /// Rules).md:6160 (Foreign Upbringing).
    LocalityAbilityCapFraction {
        /// Numerator of the surviving fraction (1 for "half").
        num: u8,
        /// Denominator (2 for "half").
        den: u8,
    },
    /// Permits buying the named Abilities or Ability categories at creation, which
    /// the rules otherwise gate behind a Virtue: "a character must have a Virtue to
    /// buy Academic, Arcane, Martial, or Supernatural Abilities at character
    /// creation … although other Virtues (and some Flaws) also grant access to some
    /// of these Abilities."
    ///
    /// Only needed for a Virtue that permits *without* granting experience — a
    /// [`Effect::RestrictedAbilityXp`] pool already implies permission for what it
    /// funds (Warrior, Arcane Lore), since the grant would otherwise be unspendable.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2315.
    AbilityAuthorization {
        /// Specific Abilities permitted.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        abilities: Vec<Id>,
        /// Whole categories permitted.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        categories: Vec<AbilityCategory>,
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
        /// Advancement-Total multiplier for Spell Mastery Abilities, as an Affinity
        /// "counts as num/den of itself": Flawless Magic doubles all mastery
        /// Advancement Totals (`{2, 1}`), halving the XP charged. Absent in JSON →
        /// `{1, 1}` (no reduction — a plain floor grant). Source: Ars Magica -
        /// Definitive Edition (Core Rules).md:3889.
        #[serde(default = "one_u8", skip_serializing_if = "is_one_u8")]
        advancement_num: u8,
        #[serde(default = "one_u8", skip_serializing_if = "is_one_u8")]
        advancement_den: u8,
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
    /// Marks the character as holding the **Masterpiece** Virtue: at Gauntlet the
    /// magus kept one *lesser enchanted item* his parens let him keep, which he
    /// designed "based on his Lab Totals at character generation". A read-only
    /// marker — the engine surfaces the derived item-level cap (best Lab Total ÷ 2,
    /// per the lesser-enchantment rule) in `derived.rs`; the actual device is still
    /// entered by hand under Magic Items. Consumed only by [`crate::derived`]; a
    /// no-op for effective scores, validation, and referential checks.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4476-4479 (Virtue),
    /// :10410 (lesser-enchantment Lab-Total-≥-2×level rule).
    MasterpieceItem,
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
    /// legal only if backed by such a grant
    /// (Ars Magica - Definitive Edition (Core Rules).md:2514). A `kind` of `None`
    /// is a
    /// **player-chosen-type** grant (Famous,
    /// Ars Magica - Definitive Edition (Core Rules).md:3861-3863: "Choose … one type"):
    /// it authorizes one Reputation of *any* type. Concrete-kind grants authorize
    /// only that type; an item with two audiences (e.g. Senior Clergy, both local
    /// and Church) carries two `GrantsReputation` effects.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:6310-6312 (Infamous),
    /// `:5703-5705` (Black Sheep), `:3861-3863` (Famous, player-chosen kind).
    GrantsReputation {
        /// Which audience the granted Reputation reaches; `None` = player-chosen
        /// (any type).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<ReputationType>,
        /// The level of the granted Reputation.
        score: u8,
    },
    /// Grants a supernatural **Might Score** of `score` in the given `realm` (base
    /// 0, summed across grants of the same Realm on top of any base the entity
    /// enters). Demonic Blood grants Infernal Might 5; Demonic Might adds +2 more.
    /// A being's Magic Resistance is derived from its effective Might Score
    /// ([`crate::derived::magic_resistance`]). A grant of `score` 0 establishes the
    /// Realm + Might without adding points (Strong Angelic Heritage, whose Divine
    /// Might = age ÷ 20 is entered by hand). Consumed by
    /// [`crate::effective::effective_might`]; a no-op for buy budgets / XP.
    ///
    /// Source: Ars Magica 5e - Realms of Power - The Infernal.md:4120 (Demonic
    /// Blood, Infernal Might 5), `:4136` (Demonic Might, +2); The Divine
    /// (Revised).md:1975 (Strong Angelic Heritage, Divine Might age ÷ 20).
    MightGrant {
        /// The Realm the granted Might is aligned to.
        realm: Realm,
        /// Might Score points granted (0 = establish Realm without adding points).
        score: u8,
    },
    /// Grants `amount` levels of **supernatural powers** (base 0, summed) — the
    /// power-levels budget the being's `powers` are charged against, mirroring
    /// [`Effect::ItemLevelBudget`] for enchanted devices. Demonic Blood grants 30,
    /// Demonic Powers +20, Strong Angelic Heritage 30. Consumed by
    /// [`crate::effective::power_levels_budget`].
    ///
    /// Source: Ars Magica 5e - Realms of Power - The Infernal.md:4122 (Demonic
    /// Blood, 30 levels), `:4142` (Demonic Powers, +20); The Divine
    /// (Revised).md:1977 (Strong Angelic Heritage, 30 levels).
    PowerLevels {
        /// Levels of supernatural powers added to the budget.
        amount: u16,
    },

    // --- M5 slice 5b: in-play effect variants ---
    //
    // These modify an in-play / derived total (computed by `derived.rs`, slice
    // 5i), never a character-creation number. Every one is a deliberate no-op in
    // `effective.rs` (balance / caps / XP) — asserted by tests — so wiring them
    // onto V/F cannot perturb creation legality. Variants tagged "surfaced-only"
    // carry data 5i *presents* in the read-out (labelled through Fluent) rather
    // than folding into a simulated number, because the app does not simulate
    // that subsystem (advancement, aging rolls, non-standard casting, recovery).
    /// A Hermetic Magical Focus. Within the descriptor named by the selection's
    /// `params[param]` — a [`ParameterDomain::Text`] sub-Art descriptor such as
    /// "necromancy", **not** an Art (a focus is sub-Art and may span Arts) — the
    /// lowest applicable Art score (which *may* be a requisite) is added twice to
    /// casting and lab totals. `major` distinguishes a Major from a Minor Focus.
    /// A magus may hold **at most one** Focus, enforced by
    /// `validation::validate_magical_focus` (counting this effect), not by
    /// pairwise incompatibility (which cannot catch two Minor foci). Computed by
    /// `derived.rs` (5i) via a per-total "focus applies" toggle, since descriptor
    /// applicability cannot be auto-derived.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4399-4422 (Major),
    /// `:4536-4542` (Minor, one-focus limit at `:4542`).
    MagicalFocus {
        /// Parameter key whose free-text value names the focus descriptor.
        param: String,
        /// `true` for a Major Focus, `false` for a Minor Focus.
        major: bool,
    },
    /// A flat modifier to a magus's Casting Total, restricted to spells of the
    /// given `scope` (Method Caster: +3 to formulaic and ritual totals). Some
    /// carriers are circumstantial (Cyclic Magic, Special Circumstances); 5i
    /// surfaces those as toggleable addends. Computed by `derived.rs` (5i).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4524-4527 (Method
    /// Caster).
    CastingTotalMod {
        /// Points added to (or, when negative, removed from) the Casting Total.
        amount: i8,
        /// Which spells the modifier applies to.
        scope: CastingScope,
    },
    /// A flat modifier to a magus's Lab Total (Inventive Genius +3). Computed by
    /// `derived.rs` (5i).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4151-4154.
    LabTotalMod {
        /// Points added to (or, when negative, removed from) the Lab Total.
        amount: i8,
    },
    /// Deficient Art: all casting and lab totals that add the Technique or Form
    /// named by the selection's `params[param]` are **halved** (Deficient Form
    /// excludes Magic Resistance). The param's domain is [`ParameterDomain::Technique`]
    /// or [`ParameterDomain::Form`], so the class restriction (Deficient Technique
    /// cannot target a Form, and vice-versa) is enforced by parameter-domain
    /// resolution; the halving *scope* is derived at compute time from the
    /// targeted Art's `ArtType`. Computed by `derived.rs` (5i).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:5913-5915
    /// (Technique), `:5909-5912` (Form).
    DeficientArt {
        /// Parameter key whose value names the deficient Technique or Form.
        param: String,
    },
    /// Halves a whole in-play total of the given kind (Weak Enchanter halves lab
    /// totals for enchanting; Weak Magic halves penetration; Flawed Parma halves
    /// Magic Resistance). Computed by `derived.rs` (5i).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:7060-7063 (Weak
    /// Enchanter), `:7064-7067` (Weak Magic), `:6142-6145` (Flawed Parma).
    MagicTotalHalving {
        /// Which in-play total is halved.
        total: HalvableTotal,
    },
    /// A flat modifier to Soak (Tough +3, Frail −1). Consumed by `derived.rs`
    /// `soak()` (5i).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:5145-5147 (Tough),
    /// `:6190-6193` (Frail).
    SoakMod {
        /// Points added to (or, when negative, removed from) Soak.
        amount: i8,
    },
    /// A flat modifier to one combat total (Berserk, Lame, Missing Hand). An item
    /// may carry several (one per affected `target`). Consumed by `derived.rs`
    /// `combat_totals()` (5i).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3500-3503 (Berserk).
    CombatMod {
        /// Points added to (or, when negative, removed from) the combat total.
        amount: i8,
        /// Which combat total the modifier affects.
        target: CombatStat,
    },
    /// A modifier to the penalty on a health track (Enduring Constitution reduces
    /// wound and fatigue penalties). `Recovery` is surfaced-only (the app does not
    /// simulate recovery rolls); the wound and fatigue tracks are consumed by
    /// `derived.rs` wound/fatigue read-outs (5i).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3751-3754 (Enduring
    /// Constitution).
    HealthMod {
        /// Which health track the modifier affects.
        track: HealthTrack,
        /// Signed modifier to that track's penalty (positive reduces the penalty
        /// magnitude in 5i's read-out; the sign convention is pinned there).
        amount: i8,
    },
    /// A non-halving Magic Resistance modifier (Limited Magic Resistance drops the
    /// Form bonus; Susceptibility adds a penalty against one realm; Commanding
    /// Aura adds a bonus while in a matching aura). Halving MR effects use
    /// [`Effect::MagicTotalHalving`]. Consumed by `derived.rs` `magic_resistance()`
    /// (5i).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:6346-6349 (Limited),
    /// `:6819-6826` (Susceptibility), `:3579-3596` (Commanding Aura).
    MagicResistanceMod {
        /// Which Magic Resistance modifier this is.
        kind: MagicResistanceEffect,
    },
    /// An aging / longevity modifier. `kind` selects which aging subsystem it
    /// touches, `amount` the signed modifier — 0 when the `kind` is itself the
    /// whole effect (Unaging carries no number, only its two immunities).
    ///
    /// **Three of the five kinds are consumed** by `aging.rs`:
    /// [`AgingEffect::AgingRoll`] is added to the AGING TOTAL with its stored
    /// sign, [`AgingEffect::LongevityBonus`] moves the ritual term of that total,
    /// and [`AgingEffect::LivingConditions`] joins the modifier the total
    /// subtracts. [`AgingEffect::NoApparentAging`] is read by
    /// `aging::resolve_outcome`. Every kind is *also* surfaced labelled in 5i's
    /// modifier read-out, which is the only home for the ones no computation
    /// reaches — see each variant for what it is worth today.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:5187-5190 (Unaging),
    /// `:16567-16569` (the AGING TOTAL the modifiers feed).
    AgingMod {
        /// Which aging / longevity subsystem the modifier touches.
        kind: AgingEffect,
        /// Signed modifier (0 when `kind` is itself the whole effect, e.g. an
        /// aging immunity).
        amount: i8,
    },
    /// A study / advancement source-quality modifier — **surfaced-only**: the app
    /// does not simulate advancement. `source` names the advancement source,
    /// `amount` the modifier (Apt Student +5 when taught). 5i surfaces these
    /// labelled.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3422-3425 (Apt
    /// Student).
    AdvancementMod {
        /// The advancement source the modifier applies to.
        source: AdvancementSource,
        /// Signed modifier to that source's Source Quality / advancement total.
        amount: i8,
    },
    /// A special casting-style quirk. `kind` names the quirk. The three
    /// non-standard-casting penalty relievers (`quiet_words`, `subtle_gestures`,
    /// `deft_form`) are **computed** by 5i into the per-cell
    /// [`crate::derived::NonStandardCasting`] variants (the residual no-voice /
    /// no-gesture penalties); every other quirk — spontaneous-magic variants
    /// (Diedne, Faerie-Raised, Life-Linked) and circumstantial casting penalties —
    /// is **surfaced-only**, listed labelled rather than simulated.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3645-3648 (Deft
    /// Form), :4822-4826 (Quiet Magic), :5073-5076 (Subtle Magic), :9243-9245
    /// (Words/Gestures penalties), `:3675-3682` (Diedne Magic), `:5917-5920`
    /// (Deleterious Circumstances).
    SpecialCastingMod {
        /// Which casting-style quirk this is.
        kind: SpecialCasting,
        /// For the Form-scoped `deft_form` quirk, the parameter key whose value
        /// names the affected Form (resolved against the selection's params, as
        /// [`Effect::DeficientArt`] resolves its Art). `None` for the unscoped
        /// quirks (Quiet/Subtle Magic apply to every casting), which carry no
        /// parameter.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        param: Option<String>,
    },
    /// A flat modifier to rolls of a specific Ability in the free-text subject
    /// named by the selection's `params[param]` (Academic Concentration: a bonus
    /// to Concentration for one field of study). **Surfaced-only**: it modifies
    /// rolls, not the bought/effective Ability score, so it never perturbs
    /// creation. 5i surfaces it labelled.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3362-3367.
    AbilityRollMod {
        /// Parameter key whose free-text value names the subject/field.
        param: String,
        /// Points added to rolls of the ability in that subject.
        amount: i8,
    },
    /// Elemental Magic (Major Hermetic): a **creation-time Art-XP redistribution**
    /// marker over the four elemental Forms named by `forms` (Aquam, Auram, Ignem,
    /// Terram — data, never hardcoded). It stores no value; at derive time each
    /// listed Form's effective score is boosted by giving it **half (rounded up)**
    /// of every other listed Form's assigned XP. Consumed by
    /// [`crate::effective::effective_art_score`] as an XP-space bonus (nonlinear in
    /// the bought score, unlike a flat [`Effect::ArtBonus`]); a no-op everywhere
    /// else.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3731-3737.
    ElementalMagic {
        /// The elemental Form ids the redistribution pools over.
        forms: std::collections::BTreeSet<Id>,
    },
}

/// Which spells a [`Effect::CastingTotalMod`] applies to. A fixed rules taxonomy
/// (so an enum, rendered via Fluent, never as a raw slug).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CastingScope {
    /// Every casting total (formulaic, ritual, and spontaneous).
    All,
    /// Formulaic spells only.
    Formulaic,
    /// Ritual spells only.
    Ritual,
    /// Formulaic and ritual spells (Method Caster).
    FormulaicRitual,
    /// Spontaneous magic only.
    Spontaneous,
}

impl fmt::Display for CastingScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CastingScope::All => "all",
            CastingScope::Formulaic => "formulaic",
            CastingScope::Ritual => "ritual",
            CastingScope::FormulaicRitual => "formulaic_ritual",
            CastingScope::Spontaneous => "spontaneous",
        })
    }
}

/// An in-play total a [`Effect::MagicTotalHalving`] halves. A fixed rules
/// taxonomy, rendered via Fluent, never as a raw slug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HalvableTotal {
    /// Spontaneous casting totals (Weak Spontaneous Magic).
    SpontaneousCasting,
    /// Lab totals for making enchanted items (Weak Enchanter).
    LabEnchanting,
    /// Lab totals for longevity rituals (Difficult Longevity Ritual).
    LabLongevity,
    /// Penetration totals (Weak Magic).
    Penetration,
    /// Magic Resistance (Flawed Parma Magica, Weak Magic Resistance).
    MagicResistance,
}

impl fmt::Display for HalvableTotal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HalvableTotal::SpontaneousCasting => "spontaneous_casting",
            HalvableTotal::LabEnchanting => "lab_enchanting",
            HalvableTotal::LabLongevity => "lab_longevity",
            HalvableTotal::Penetration => "penetration",
            HalvableTotal::MagicResistance => "magic_resistance",
        })
    }
}

/// A combat total a [`Effect::CombatMod`] modifies. A fixed rules taxonomy,
/// rendered via Fluent, never as a raw slug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombatStat {
    /// Initiative total.
    Initiative,
    /// Attack total.
    Attack,
    /// Defense total.
    Defense,
    /// Damage total.
    Damage,
}

impl fmt::Display for CombatStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CombatStat::Initiative => "initiative",
            CombatStat::Attack => "attack",
            CombatStat::Defense => "defense",
            CombatStat::Damage => "damage",
        })
    }
}

/// A health track a [`Effect::HealthMod`] modifies. A fixed rules taxonomy,
/// rendered via Fluent, never as a raw slug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthTrack {
    /// The penalty imposed by reduced Fatigue levels (Enduring Constitution
    /// reduces it, Low Tolerance increases it). Computed by 5i.
    FatiguePenalty,
    /// The total penalty imposed by wounds (Enduring Constitution reduces it).
    /// Computed by 5i.
    WoundPenalty,
    /// Fatigue / Stamina rolls to avoid fatigue (Long-Winded +3, Obese/Short of
    /// Breath −3). Surfaced-only.
    FatigueRoll,
    /// Fatigue levels lost per spell cast (Vulnerable Casting +1, Withstand
    /// Casting −1, Painful Magic). Surfaced-only.
    CastingFatigue,
    /// Wound-recovery rolls (Rapid Convalescence +3, Fragile Constitution −3).
    /// Surfaced-only.
    Recovery,
}

impl fmt::Display for HealthTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HealthTrack::FatiguePenalty => "fatigue_penalty",
            HealthTrack::WoundPenalty => "wound_penalty",
            HealthTrack::FatigueRoll => "fatigue_roll",
            HealthTrack::CastingFatigue => "casting_fatigue",
            HealthTrack::Recovery => "recovery",
        })
    }
}

/// A non-halving Magic Resistance modifier ([`Effect::MagicResistanceMod`]). A
/// fixed rules taxonomy, rendered via Fluent, never as a raw slug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MagicResistanceEffect {
    /// The relevant Form's contribution to Magic Resistance is dropped (Limited
    /// Magic Resistance: resistance from Parma alone).
    NoFormBonus,
    /// A bonus to Magic Resistance while in a matching aura (Commanding Aura).
    AuraBonus,
    /// A penalty to Magic Resistance against Divine power (Susceptibility).
    SusceptibleDivine,
    /// A penalty to Magic Resistance against Faerie power (Susceptibility).
    SusceptibleFaerie,
    /// A penalty to Magic Resistance against Infernal power (Susceptibility).
    SusceptibleInfernal,
}

impl fmt::Display for MagicResistanceEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MagicResistanceEffect::NoFormBonus => "no_form_bonus",
            MagicResistanceEffect::AuraBonus => "aura_bonus",
            MagicResistanceEffect::SusceptibleDivine => "susceptible_divine",
            MagicResistanceEffect::SusceptibleFaerie => "susceptible_faerie",
            MagicResistanceEffect::SusceptibleInfernal => "susceptible_infernal",
        })
    }
}

/// Which aging / longevity subsystem an [`Effect::AgingMod`] touches. A fixed
/// rules taxonomy, rendered via Fluent (`derived-detail-<slug>`).
///
/// The two immunities are **orthogonal**, because the sources state them
/// separately: not dropping Characteristics and not looking older are different
/// facts, and Bound to (Role) has the first without the second (`:5743`).
/// Collapsing them into one tag is what made the shipped Bee King entry wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgingEffect {
    /// A modifier to aging rolls, added to the AGING TOTAL with its stored sign
    /// (Faerie Blood -1). Consumed by `aging::aging_total`.
    AgingRoll,
    /// A modifier to the longevity-ritual bonus. Consumed by
    /// `aging::aging_total`, and only for a character who actually holds a
    /// ritual.
    LongevityBonus,
    /// **Aging Points do not decrease the character's Characteristics** — they
    /// still accrue, and still count toward Decrepitude: "your aging points do
    /// not decrease your Characteristics, only building up to give you
    /// Decrepitude points" (`:5189`). Carried by Unaging and by Bound to (Role),
    /// which "also includes the effects of the Unaging Virtue" (`:5743`). It says
    /// nothing about the apparent age — see [`Self::NoApparentAging`].
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:5189, :5743.
    NoAging,
    /// **The apparent age never advances**, whatever the roll: "Bee Kings do not
    /// appear to age after reaching maturity" (`:3488`), and Unaging's "You may
    /// choose your apparent age freely" (`:5189`). Consumed by
    /// `aging::resolve_outcome`, so a carrier's
    /// `AgingOutcome::apparent_age_increases` is false at every total.
    ///
    /// Bound to (Role) deliberately does **not** carry it: "but the character's
    /// apparent age advances in line with their physical age" (`:5743`) — the
    /// sentence that proves the two immunities are separable at all. A Bee King
    /// carries this one alone, and so still loses Characteristics.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3488, :5189,
    /// :5743.
    NoApparentAging,
    /// A modifier to accrued Decrepitude. Surfaced only — no shipped item moves
    /// a number here, and the score is derived from the accrued Aging Points.
    Decrepitude,
    /// A modifier to the Living Conditions Modifier that feeds aging rolls
    /// (Poor Living Conditions -1, Leprosy -2, Mild Aging +1). Consumed by
    /// `aging::living_conditions_modifier`.
    LivingConditions,
    /// A modifier to the roll to **survive an aging crisis** — a different roll
    /// from the aging roll, and deliberately sealed off from it: "Virtues that
    /// affect aging rolls do not affect crisis survival rolls" (`:16636`). So a
    /// modifier tagged here never reaches [`Self::AgingRoll`]'s total, and an
    /// `aging_roll` modifier never reaches the survival roll.
    ///
    /// The general prohibition does not silence a *specific* grant: Mild Aging's
    /// "he receives a +3 bonus to rolls to survive an aging crisis" (`:4530`) is
    /// exactly such a grant, and is the first shipped item to carry this kind.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4530, :16636.
    CrisisSurvival,
    /// **A Heavy Wound whenever a crisis lands**: "whenever she undergoes an Aging
    /// Crisis (page 392) the leper sustains a Heavy Wound in addition to any other
    /// result" (`:6340`). A marker — the `amount` is ignored and ships as 0,
    /// because this is a consequence of the crisis, not a number added to any roll.
    ///
    /// Kept separate from [`Self::CrisisSurvival`] rather than folded into one
    /// `crisis` kind: the two are different kinds of fact, and collapsing them
    /// would make the stored `amount` mean a roll modifier for one carrier and
    /// nothing at all for the next.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:6340.
    CrisisHeavyWound,
}

impl AgingEffect {
    /// Every kind, in declaration order — the source of the set for the
    /// `derived-detail-<slug>` locale coverage and the frontend union mirror, so a
    /// new kind fails those tests until it is named and mirrored.
    pub const ALL: [AgingEffect; 8] = [
        AgingEffect::AgingRoll,
        AgingEffect::LongevityBonus,
        AgingEffect::NoAging,
        AgingEffect::NoApparentAging,
        AgingEffect::Decrepitude,
        AgingEffect::LivingConditions,
        AgingEffect::CrisisSurvival,
        AgingEffect::CrisisHeavyWound,
    ];
}

impl fmt::Display for AgingEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            AgingEffect::AgingRoll => "aging_roll",
            AgingEffect::LongevityBonus => "longevity_bonus",
            AgingEffect::NoAging => "no_aging",
            AgingEffect::NoApparentAging => "no_apparent_aging",
            AgingEffect::Decrepitude => "decrepitude",
            AgingEffect::LivingConditions => "living_conditions",
            AgingEffect::CrisisSurvival => "crisis_survival",
            AgingEffect::CrisisHeavyWound => "crisis_heavy_wound",
        })
    }
}

/// The advancement source an [`Effect::AdvancementMod`] applies to
/// (surfaced-only). A fixed rules taxonomy, rendered via Fluent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvancementSource {
    /// Being taught (Apt Student, Poor Student).
    Taught,
    /// Learning from a book (Book Learner).
    Book,
    /// Studying from raw vis (Free Study).
    Vis,
    /// Practice (Independent Study).
    Practice,
    /// Adventure experience (Independent Study).
    Adventure,
    /// Insight into a magical topic (Secondary Insight).
    Insight,
    /// The character teaching others (Good Teacher, Incomprehensible).
    Teaching,
    /// Mastering spells (Loose Magic halves this advancement).
    SpellMastery,
    /// Every advancement source (Study Bonus; Unimaginative Learner).
    All,
}

impl fmt::Display for AdvancementSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            AdvancementSource::Taught => "taught",
            AdvancementSource::Book => "book",
            AdvancementSource::Vis => "vis",
            AdvancementSource::Practice => "practice",
            AdvancementSource::Adventure => "adventure",
            AdvancementSource::Insight => "insight",
            AdvancementSource::Teaching => "teaching",
            AdvancementSource::SpellMastery => "spell_mastery",
            AdvancementSource::All => "all",
        })
    }
}

/// A special casting-style quirk an [`Effect::SpecialCastingMod`] names
/// (surfaced-only). A fixed rules taxonomy, rendered via Fluent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialCasting {
    /// Cast without the normal words penalty (Quiet Magic).
    QuietWords,
    /// Cast without the normal gestures penalty (Subtle Magic).
    SubtleGestures,
    /// No requisite penalty for one Form (Deft Form).
    DeftForm,
    /// Diedne Magic spontaneous-casting method.
    Diedne,
    /// Faerie-Raised Magic spontaneous-casting method.
    FaerieRaised,
    /// Life-Linked Spontaneous Magic.
    LifeLinkedSpontaneous,
    /// Spell Improvisation (spontaneous flexibility).
    SpellImprovisation,
    /// Mercurian Magic ritual/spontaneous method.
    Mercurian,
    /// Life Boost (spend fatigue to raise a casting total).
    LifeBoost,
    /// A circumstantial casting/lab penalty tied to a described condition
    /// (Deleterious Circumstances, Environmental Magic, Short-Ranged Magic,
    /// Corrupted Spells).
    Circumstantial,
}

impl fmt::Display for SpecialCasting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SpecialCasting::QuietWords => "quiet_words",
            SpecialCasting::SubtleGestures => "subtle_gestures",
            SpecialCasting::DeftForm => "deft_form",
            SpecialCasting::Diedne => "diedne",
            SpecialCasting::FaerieRaised => "faerie_raised",
            SpecialCasting::LifeLinkedSpontaneous => "life_linked_spontaneous",
            SpecialCasting::SpellImprovisation => "spell_improvisation",
            SpecialCasting::Mercurian => "mercurian",
            SpecialCasting::LifeBoost => "life_boost",
            SpecialCasting::Circumstantial => "circumstantial",
        })
    }
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
    /// Known within scholarly / university circles (the "Academic Reputation" the
    /// scholastic Social-Status Virtues confer — Baccalaureus, Magister in
    /// Artibus, Doctor in (Faculty), …). Core names it as a Reputation type
    /// alongside the three "main" types at `:1097` ("The most basic type is …").
    Academic,
}

impl ReputationType {
    /// All types in book order (the single source of the serialized ordering).
    pub const ALL: [ReputationType; 4] = [
        ReputationType::Local,
        ReputationType::Ecclesiastical,
        ReputationType::Hermetic,
        ReputationType::Academic,
    ];
}

impl std::fmt::Display for ReputationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ReputationType::Local => "local",
            ReputationType::Ecclesiastical => "ecclesiastical",
            ReputationType::Hermetic => "hermetic",
            ReputationType::Academic => "academic",
        })
    }
}

/// A phase of character creation.
///
/// Two consumers share this vocabulary. An [`EntityTypeProfile`] lists the phases
/// its type is built through, in order, and the guided wizard walks that list; and
/// every [`ValidationIssue`](crate::validation::ValidationIssue) names the phase
/// whose input surface owns the offending value, so the wizard can tell which
/// findings belong to the step the user is on.
///
/// A fixed taxonomy the rules define, so it is an enum rather than data: adding a
/// phase must be a compile error until every issue site, the UI's step table and
/// both locales handle it. [`CreationPhase::ALL`] is the single source of the set,
/// so nothing re-hardcodes its members.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreationPhase {
    /// The character concept and identity: name, description, gender, birth year.
    Concept,
    /// Characteristics.
    Characteristics,
    /// Virtues and Flaws, including their point balance and category caps.
    VirtuesFlaws,
    /// Where the character's experience comes from: one total the player enters, or
    /// the life stages that earn it — and, in that case, the plan those stages are
    /// priced from (the age, a magus's Gauntlet age and post-Gauntlet seasons, the
    /// native language, and the sample childhood)
    /// (Ars Magica - Definitive Edition (Core Rules).md:2364, :2213-2216).
    /// Declared before [`Abilities`](Self::Abilities), because it is what funds it.
    Experience,
    /// Abilities, bought with the experience the previous phase supplies.
    Abilities,
    /// Hermetic Arts.
    Arts,
    /// Spells and spell mastery.
    Spells,
    /// A magus's House, plus the specialisation or free Virtue it grants.
    HouseSpecialisation,
    /// A mythic companion's type and the package it confers.
    MythicType,
    /// Personality Traits and Reputations.
    PersonalityReputations,
    /// The aging a character owes before play: the age itself, the Living
    /// Conditions and Longevity Ritual that modify each aging total, and the
    /// per-year rolls the rules require of anyone past the threshold
    /// (Ars Magica - Definitive Edition (Core Rules).md:2232, :16563-16617).
    Aging,
    /// The terminal phase: everything a finished character carries that no
    /// creation phase owns — equipment, magic items, Might and powers, Warping —
    /// plus a last look at the whole character. A profile may not declare it
    /// (the wizard appends it), so it is the one phase that is never skipped.
    Review,
}

impl CreationPhase {
    /// Every phase, in the order the rules' creation summary walks them
    /// (Ars Magica - Definitive Edition (Core Rules).md:2205-2222), with the synthetic [`Review`](Self::Review)
    /// last. The single source of the phase set: the Fluent `phase-<slug>` keys,
    /// the UI's step table and the issue-contract table are all checked against
    /// it rather than against a second hardcoded list.
    pub const ALL: [CreationPhase; 12] = [
        CreationPhase::Concept,
        CreationPhase::Characteristics,
        CreationPhase::VirtuesFlaws,
        CreationPhase::Experience,
        CreationPhase::Abilities,
        CreationPhase::Arts,
        CreationPhase::Spells,
        CreationPhase::HouseSpecialisation,
        CreationPhase::MythicType,
        CreationPhase::PersonalityReputations,
        CreationPhase::Aging,
        CreationPhase::Review,
    ];
}

impl fmt::Display for CreationPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CreationPhase::Concept => "concept",
            CreationPhase::Characteristics => "characteristics",
            CreationPhase::VirtuesFlaws => "virtues_flaws",
            CreationPhase::Experience => "experience",
            CreationPhase::Abilities => "abilities",
            CreationPhase::Arts => "arts",
            CreationPhase::Spells => "spells",
            CreationPhase::HouseSpecialisation => "house_specialisation",
            CreationPhase::MythicType => "mythic_type",
            CreationPhase::PersonalityReputations => "personality_reputations",
            CreationPhase::Aging => "aging",
            CreationPhase::Review => "review",
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

/// A ceiling expressed as a fraction — `numerator`/`denominator` — of some
/// whole, kept as an exact integer pair rather than a float so the comparison
/// that uses it (`part · denominator > total · numerator`) needs no rounding
/// choice and no floating-point equality.
///
/// The rules state such ceilings in words ("no more than half"), and a *value*
/// a rulebook states belongs in the rules JSON rather than in engine code, so
/// the fraction is data. Load-time integrity rejects a zero denominator and a
/// numerator above its denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Share {
    /// How many parts of the whole the ceiling permits.
    pub numerator: u8,
    /// How many parts the whole is divided into. Never zero.
    pub denominator: u8,
}

/// A virtue, flaw, boon, or hook with its mechanical metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PointItemRepr")]
pub struct PointItem {
    /// Stable slug identifier.
    pub id: Id,
    /// Whether this is a virtue, flaw, boon, or hook.
    pub kind: ItemKind,
    /// Point weight (free/minor/major).
    pub magnitude: Magnitude,
    /// Every grouping category the rulebook descriptor lists for this item, in
    /// the order it lists them.
    ///
    /// Most descriptors name a single category, but some name two: Suppressed
    /// Gift is "*Major, Hermetic, Story*"
    /// (Ars Magica - Definitive Edition (Core Rules).md:6803-6804), and a
    /// character may legitimately reach it through either. **All of them are
    /// equally real** — the book's own indexes list such an item under both
    /// headings (Suppressed Gift at
    /// Ars Magica - Definitive Edition (Core Rules).md:5301 under
    /// "### Hermetic, Major" and again at :5369 under "### Story, Major"), and
    /// there is no "primary" among them. *Membership* tests — permitted/forbidden
    /// categories, category caps, grant constraints, Gift categories — consider
    /// the whole list, and so does the UI's Available picker, which offers the
    /// item under every heading it carries.
    ///
    /// `categories[0]` therefore carries no rules meaning; it is only a
    /// deterministic tie-break for the two places that structurally have room for
    /// exactly one ([`PointItem::first_listed_category`]).
    ///
    /// Order carries the descriptor's own emphasis and is therefore deliberately
    /// exempt from canonical sorting (like [`EntityTypeProfile::creation_phases`]
    /// and the crisis table's rows). Load-time integrity rejects an empty list or
    /// a repeated slug.
    pub categories: Vec<String>,
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
    /// Maximum number of copies of this item, TOTAL across every distinct
    /// parameter target, that may appear in the finished character — counting
    /// granted copies (a House-granted Puissant Ignem counts against the same
    /// ceiling as one the player buys). Default `u8::MAX` (255) = "no ceiling
    /// the rules state" (the same sentinel convention as `max_per_target`; see
    /// `RULES.md:782-787`). Distinct from `max_per_target`, which caps copies
    /// sharing one identical `(id, params)` target: e.g. a Virtue repeatable
    /// "with a different target each time" may need `max_per_target: 1` (no
    /// repeat of the same target) alongside a stated `max_total` (an overall
    /// cap across all targets), or no `max_total` at all (unlimited targets).
    #[serde(
        default = "default_max_total",
        skip_serializing_if = "is_default_max_total"
    )]
    pub max_total: u8,
    /// The largest share of its **own kind's** point total that all copies of
    /// this item — bought and granted alike — may account for. `None` (the
    /// default, and the case for almost every entry) means the rules state no
    /// such ratio.
    ///
    /// Measured in points, against the points actually taken, and split by kind:
    /// a Virtue's copies are weighed against Virtue points, a Flaw's against
    /// Flaw points. Drives the share-of-kind cap in
    /// `validation::caps::validate_share_of_kind_cap`.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:3665 and :3669
    /// (Demonic Might / Demonic Powers, "no more than half of the character's
    /// total Virtues").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_share_of_kind: Option<Share>,
    /// Provenance into the Markdown source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// A `1` default for an Affinity multiplier component (num/den) — the identity
/// ratio, i.e. no cost change. Used by [`Effect::GrantsSpellMastery`]'s optional
/// Advancement-doubling fields.
fn one_u8() -> u8 {
    1
}

fn is_one_u8(value: &u8) -> bool {
    *value == 1
}

/// The default selection multiplicity: an item may be taken once per target.
fn default_max_per_target() -> u8 {
    1
}

fn is_default_max_per_target(value: &u8) -> bool {
    *value == default_max_per_target()
}

/// The default total-selection ceiling: no stated limit. See
/// `PointItem::max_total`'s doc comment and `RULES.md:782-787` for the
/// `u8::MAX` sentinel convention.
fn default_max_total() -> u8 {
    u8::MAX
}

fn is_default_max_total(value: &u8) -> bool {
    *value == default_max_total()
}

/// The default virtue/flaw conversion: one Flaw point funds one Virtue point.
fn default_virtue_points_per_flaw_point() -> u8 {
    1
}

fn is_default_virtue_points_per_flaw_point(value: &u8) -> bool {
    *value == default_virtue_points_per_flaw_point()
}

/// The on-disk shape of a [`PointItem`], deserialized before validation.
///
/// [`PointItem`] is `#[serde(try_from = "PointItemRepr")]` so that the two ways a
/// catalogue can fail to state an item's categories are reported *with the
/// offending item's id* instead of a bare `missing field` at a byte offset. The
/// case that motivates it is a stale `rules/` directory sitting beside a newer
/// binary (the portable layout): the removed singular `category` key must stop
/// the load loudly, never be silently coerced into a one-element list.
#[derive(Deserialize)]
struct PointItemRepr {
    id: Id,
    kind: ItemKind,
    magnitude: Magnitude,
    #[serde(default)]
    categories: Option<Vec<String>>,
    /// The removed singular key, accepted here only so its presence can be
    /// rejected by name. Never stored.
    #[serde(default)]
    category: Option<String>,
    classification: Classification,
    #[serde(default)]
    tainted: bool,
    #[serde(default)]
    entity_kinds: BTreeSet<EntityKind>,
    #[serde(default)]
    prerequisites: Option<Prereq>,
    #[serde(default)]
    incompatible_with: BTreeSet<Id>,
    #[serde(default)]
    parameters: Vec<ParameterDef>,
    #[serde(default)]
    effects: Vec<Effect>,
    #[serde(default = "default_max_per_target")]
    max_per_target: u8,
    #[serde(default = "default_max_total")]
    max_total: u8,
    #[serde(default)]
    max_share_of_kind: Option<Share>,
    #[serde(default)]
    source: Option<SourceRef>,
}

impl TryFrom<PointItemRepr> for PointItem {
    type Error = String;

    fn try_from(repr: PointItemRepr) -> Result<Self, Self::Error> {
        let PointItemRepr {
            id,
            kind,
            magnitude,
            categories,
            category,
            classification,
            tainted,
            entity_kinds,
            prerequisites,
            incompatible_with,
            parameters,
            effects,
            max_per_target,
            max_total,
            max_share_of_kind,
            source,
        } = repr;

        if category.is_some() {
            return Err(format!(
                "point item '{id}' uses the removed singular 'category' key; \
                 it takes 'categories', an array of category slugs in the \
                 descriptor's own order"
            ));
        }
        let Some(categories) = categories else {
            return Err(format!(
                "point item '{id}' is missing the required 'categories' field, \
                 an array of category slugs in the descriptor's own order"
            ));
        };

        Ok(Self {
            id,
            kind,
            magnitude,
            categories,
            classification,
            tainted,
            entity_kinds,
            prerequisites,
            incompatible_with,
            parameters,
            effects,
            max_per_target,
            max_total,
            max_share_of_kind,
            source,
        })
    }
}

impl PointItem {
    /// Sorts the `parameters` vector by key for canonical serialization.
    ///
    /// `categories` is order-significant (the descriptor's own order) and so is
    /// deliberately left untouched.
    pub fn normalize(&mut self) {
        self.parameters.sort_by(|a, b| a.key.cmp(&b.key));
    }

    /// The category the item's rulebook descriptor lists **first**.
    ///
    /// This is a deterministic tie-break, NOT a statement about the item: the
    /// rulebook indexes every category a descriptor names, so no category is
    /// privileged over another. Use it only where the surface has room for
    /// exactly one — the `category_not_permitted` message's `category` argument,
    /// and the UI's Selected list, whose rows are addressed by index and so must
    /// not be repeated. It is never the basis of a membership decision (see
    /// [`PointItem::has_category`]) and never the basis of a *browsing* list (the
    /// Available picker lists the item under all of them).
    ///
    /// Empty only for a catalogue that failed load-time integrity, which rejects
    /// an item with no categories.
    pub fn first_listed_category(&self) -> &str {
        self.categories
            .first()
            .map(String::as_str)
            .unwrap_or_default()
    }

    /// Whether the item carries `category` at all, in any position. Every
    /// membership rule (permitted/forbidden lists, caps, grant constraints, Gift
    /// categories) is expressed with this, so a two-category item counts under
    /// both of them.
    pub fn has_category(&self, category: &str) -> bool {
        self.categories.iter().any(|c| c == category)
    }

    /// The first of the item's categories that is a member of `set`, in the
    /// item's own order — i.e. the category that actually made a membership test
    /// succeed, which is what an issue message should name.
    pub fn first_category_in(&self, set: &BTreeSet<String>) -> Option<&str> {
        self.categories
            .iter()
            .find(|c| set.contains(*c))
            .map(String::as_str)
    }

    /// Whether any of the item's categories is a member of `set`.
    pub fn any_category_in(&self, set: &BTreeSet<String>) -> bool {
        self.first_category_in(set).is_some()
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
    /// he may know at creation). 120 for the magus profile (Ars Magica - Definitive Edition (Core Rules).md:2215-2216,
    /// 2435); 0 (omitted) for every non-magus type, which cannot take spells.
    /// Modified per-character by [`Effect::SpellLevels`] (Skilled/Weak Parens).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub spell_levels: u32,
    /// The character type's starting Confidence Score
    /// (Ars Magica - Definitive Edition (Core Rules).md:2524 — the earlier
    /// citation of this line pointed at the "### Confidence" heading two lines
    /// above the actual "start with a Confidence Score of 1 and 3 Confidence
    /// Points" sentence; corrected against the source). Companions,
    /// magi and mythic companions start at 1; grogs have no Confidence (0/omitted).
    /// The effective score folds in `Effect::ConfidenceBonus`; Confidence is
    /// derived, never stored on the entity.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub confidence_score: u8,
    /// The character type's starting Confidence Points
    /// (Ars Magica - Definitive Edition (Core Rules).md:2524): 3 for
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
    /// Ordered creation phases the guided wizard walks through. Typed, so serde
    /// itself is the load-time validator: a profile naming a phase the engine has
    /// no [`CreationPhase`] for fails the ruleset load rather than reaching the
    /// wizard as a step it cannot render.
    // Order-significant (the wizard walks them in sequence): intentionally
    // exempt from `normalize`'s canonical sorting.
    pub creation_phases: Vec<CreationPhase>,
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
/// (Ars Magica - Definitive Edition (Core Rules).md:12349-12353), so identity is (spell, level, parameter). Kept
/// sorted via [`Entity::normalize`].
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
    /// at 1. Source: Ars Magica - Definitive Edition (Core Rules).md:4471-4474,
    /// :3887-3889.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mastery: Option<u8>,
    /// The chosen value for a parameterized spell (the target `(Form)` of a
    /// meta-magic Vim spell like Wizard's Boost — an Art id such as `art.ignem`).
    /// Part of the spell's identity: the same base spell may be taken once per
    /// distinct parameter (Ars Magica - Definitive Edition (Core
    /// Rules).md:15791-15794). Display + identity only —
    /// it does NOT change the spell's own Technique/Form. `None` for ordinary,
    /// unparameterized spells.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// The chosen Spell Mastery special abilities for this spell, each a
    /// `spell_mastery_ability.*` id from the catalogue. One may be chosen per
    /// effective mastery level (Ars Magica - Definitive Edition (Core
    /// Rules).md:9524-9526); a repeatable ability
    /// (Precise/Quick/Quiet Casting) may appear more than once, so this is a
    /// `Vec` that may hold duplicates, not a set. Additive and serde-defaulted, so
    /// it is backward/forward compatible with saves written before it existed
    /// (SCHEMA_VERSION stays 13): an old save omits the key and deserializes to an
    /// empty vector; a new save with an empty vector omits the key on write. Kept
    /// sorted (stable, with duplicates) by [`Entity::normalize`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mastery_abilities: Vec<Id>,
}

/// A piece of equipment the character carries: a reference to a catalogue weapon,
/// shield, or armor id, plus whether it is currently equipped (wielded / worn).
/// Only the choice is stored — combat totals, Soak, and Encumbrance are derived
/// downstream (5i) from the referenced catalogue row. Kept sorted via
/// [`Entity::normalize`]. Source: Ars Magica - Definitive Edition (Core
/// Rules).md:16944-17011 (the equipment tables),
/// :17103-17123 (Encumbrance, computed in the derived-totals slice).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EquipmentSlot {
    /// The catalogue id of the item (a `weapon.*`, `shield.*`, or `armor.*` id).
    pub item: Id,
    /// Whether the item is currently equipped (wielded/worn). Unequipped items
    /// still count toward carried Load but not toward combat/Soak lines (5i).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub equipped: bool,
    /// Whether this weapon's combat Ability specialization applies to it, granting
    /// +1 to the weapon's Attack and Defense (the specialty must be aligned to this
    /// specific weapon; Damage/Initiative do not use the Ability). Only meaningful
    /// for a weapon slot whose Ability carries a specialty. Additive and
    /// serde-defaulted — old saves omit the key and deserialize to `false`, a new
    /// save with `false` omits it on write (SCHEMA_VERSION unchanged), exactly like
    /// the sibling `equipped` field. Source: Ars Magica - Definitive Edition
    /// (Core Rules).md:7122, :7139.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub specialization_applies: bool,
}

/// A named Personality Trait with a value in −3..+3 (or ±6 for the trait
/// representing a Major Personality Flaw). Free-text name, kept sorted by name in
/// [`Entity::normalize`]. Source: Ars Magica - Definitive Edition (Core
/// Rules).md:2500-2503.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PersonalityTrait {
    /// Free-text trait name (e.g. "Brave", "Loyal").
    pub name: String,
    /// Trait value; ±3 normally, ±6 for a Major Personality Flaw's trait.
    pub value: i8,
}

/// A starting Reputation: score + free-text content + audience type. Only legal
/// when backed by a granting Virtue/Flaw ([`Effect::GrantsReputation`]).
/// Source: Ars Magica - Definitive Edition (Core Rules).md:1091-1101, 2512-2514.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Reputation {
    /// Which audience the Reputation reaches.
    pub kind: ReputationType,
    /// The Reputation's level.
    pub score: u8,
    /// Free-text description of what the Reputation is for.
    pub content: String,
}

/// An enchanted device a magus starts with. Only the choice is stored (name +
/// total effect level); the `level` is charged against the item-level budget the
/// character's Magic Items / Redcap Virtues grant (see
/// [`crate::effective::item_level_budget`]). Kept sorted via [`Entity::normalize`].
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4347-4349 (Magic
/// Items), :4842-4846 (Redcap).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnchantedDevice {
    /// Free-text device name.
    pub name: String,
    /// Total effect level, charged against the item-level budget.
    pub level: u16,
}

/// The four supernatural Realms of Mythic Europe. A being's Might is aligned to
/// exactly one Realm, which determines its Magic Resistance and what powers it may
/// hold. A fixed rules taxonomy, rendered via Fluent, never as a raw slug.
/// Source: Ars Magica 5e - Realms of Power - Magic.md:1470-1472; Ars Magica -
/// Definitive Edition (Core Rules).md:2623-2631.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Realm {
    /// The Magic Realm (magical beasts, spirits, elementals).
    Magic,
    /// The Faerie Realm.
    Faerie,
    /// The Divine Realm (angels, Nephilim, holy creatures).
    Divine,
    /// The Infernal Realm (demons, demon-blooded beings).
    Infernal,
}

impl fmt::Display for Realm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Realm::Magic => "magic",
            Realm::Faerie => "faerie",
            Realm::Divine => "divine",
            Realm::Infernal => "infernal",
        })
    }
}

/// A supernatural being's **Might Score** and the Realm it is aligned to. A Might
/// Score grants blanket Magic Resistance equal to the score (Ars Magica 5e -
/// Realms of Power - Magic.md:1472). Only the choice is stored; the effective
/// score and its Magic Resistance are derived. Source: Ars Magica 5e - Realms
/// of Power - Magic.md:1470-1472.
///
/// The struct is shared by two holders, and Virtue grants apply to only one of them:
/// - [`Entity::might`] — the *character's* own Might. A being may enter a base score
///   (e.g. Strong Angelic Heritage's Divine Might = age ÷ 20, which the engine cannot
///   fix as a constant grant), which Virtue [`Effect::MightGrant`]s of the same Realm
///   add to; [`crate::effective::effective_might`] sums the two.
/// - [`Familiar::might`] — the familiar's **own** Magic Might. No Virtue grant ever
///   stacks on it: `effective_might` reads only `Entity::might`, and the magus's
///   Virtues are not the beast's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MightScore {
    /// The Realm the being's Might is aligned to.
    pub realm: Realm,
    /// The Might Score the player entered. For [`Entity::might`] this is the *base*
    /// score, which Virtue grants of the same Realm add to; for [`Familiar::might`]
    /// it is the whole score, since nothing is ever granted on top.
    pub score: u8,
}

/// A supernatural power a Might-being holds. Only the choice is stored (free-text
/// name + total level); the `level` is charged against the power-levels budget the
/// being's Might Virtues grant (see [`crate::effective::power_levels_budget`]),
/// exactly as an [`EnchantedDevice`] is charged against the item-level budget. The
/// engine is not a power *designer* — powers are entered by hand, like spells.
/// Kept sorted via [`Entity::normalize`]. Source: Ars Magica 5e - Realms of
/// Power - The Infernal.md:4122; Ars Magica 5e - Realms of Power - The Divine
/// (Revised).md:1977.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SupernaturalPower {
    /// Free-text power name.
    pub name: String,
    /// Total power level, charged against the power-levels budget.
    pub level: u16,
}

/// A magus's familiar: the magical beast itself plus the three bond cords.
///
/// "A familiar is a beast that a magus befriends and then magically bonds with,
/// instilling the beast with magical powers in the process" — it "always has its
/// own will, and is not under the control of the magus"
/// (Ars Magica - Definitive Edition (Core Rules).md:10766-10892). It is
/// therefore a *creature*, and the fields below
/// follow the rulebook's own **Creature Format** order (`:17787-17827`) so a save
/// reads like a statblock: Might, Characteristics, Size, Personality Traits, …,
/// Powers.
///
/// **Scope (M5.5c).** Modelled: the animal, its Magic Might, the eight
/// Characteristics, Size, Personality Traits, the three cords, and the powers
/// invested in the bond. **Deliberately deferred**: the familiar's Abilities,
/// Qualities, Virtues/Flaws and Combat/Soak/Fatigue/Wound statlines — the creature
/// lines the Creature Format also carries but that this app does not yet enter for
/// any creature.
///
/// Everything the familiar holds is its **own**, never the magus's: its
/// Characteristics are not bought from the magus's Characteristic points
/// (`:17793`), and its Might is not the magus's Might, so neither the point-buy nor
/// the Might realm-agreement check can ever see them (both invariants are locked by
/// regression tests in `validation/mod.rs`). Nothing is derived here; the read-outs
/// live in [`crate::derived::familiar_readout`].
///
/// Every field beyond `name` is additive `serde(default, skip_serializing_if)`, so
/// a pre-M5.5c familiar (name + cords) loads unchanged and writes identical bytes —
/// hence no `SCHEMA_VERSION` bump.
///
/// Source: `Ars Magica - Definitive Edition (Core Rules).md:10766-10892` (Familiars),
/// `:10840-10844` (the three cords), `:17787-17827` (Creature Format).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Familiar {
    /// Free-text familiar name.
    pub name: String,
    /// The kind of beast, free text ("raven", "tortoiseshell cat"). Deliberately
    /// **not** `species`: the rules reserve *Species* for the Imaginem term
    /// (the sensory image a thing sheds), so the field is named for the animal.
    /// Source: `:10774` ("finding an animal with inherent magic").
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub animal: String,
    /// The familiar's own Magic Might Score + Realm; `None` when not entered.
    /// "the beast is likely to have a Magic Might score, which may be assigned
    /// based on the scores of comparable magical creatures" (`:10774`). This is the
    /// familiar's Might, never the magus's — no Virtue grant stacks on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub might: Option<MightScore>,
    /// The familiar's eight Characteristics (`:17793`), signed and NOT bought from
    /// the magus's Characteristic points. A score of 0 is pruned by
    /// [`Familiar::normalize`] and the map is omitted when empty, so an explicit 0 and
    /// an absent entry serialize identically.
    ///
    /// A bound familiar that lacked human intelligence "gains it, with a score of
    /// –3" (`:10854`), which is an ordinary Intelligence entry — so the fixed
    /// eight-value [`Characteristic`] enum needs no `Cunning` variant. The
    /// *unbound* creature's Cunning (Cun) score the Creature Format mentions
    /// (`:17793`) is a display-only affordance and is deliberately deferred; see
    /// RULES.md so it is not re-litigated.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub characteristics: BTreeMap<Characteristic, i8>,
    /// The familiar's Size — signed, and commonly **negative** (a raven is -4).
    /// It lowers the bonding level: "If the familiar has negative Size, this
    /// reduces the level for the enchantment" (`:10824`). Source: `:17795`
    /// (creature Size), `:17829-17856` (the Size examples table).
    #[serde(default, skip_serializing_if = "is_zero_i8")]
    pub size: i8,
    /// The familiar's Personality Traits (`:17807`). The bond adds Loyal (partner)
    /// +3 (`:10852`), which is surfaced as a note and entered by hand, never
    /// auto-applied. Kept sorted via [`Familiar::normalize`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub personality_traits: Vec<PersonalityTrait>,
    /// Gold cord score (reduces botch dice). Bounded by [`MAX_CORD_SCORE`], which
    /// [`Familiar::normalize`] clamps to.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub cord_gold: u8,
    /// Silver cord score (Personality / mental resistance). Bounded by
    /// [`MAX_CORD_SCORE`], which [`Familiar::normalize`] clamps to.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub cord_silver: u8,
    /// Bronze cord score (Soak & aging-resistance). Bounded by [`MAX_CORD_SCORE`],
    /// which [`Familiar::normalize`] clamps to.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub cord_bronze: u8,
    /// The powers invested in the familiar bond. Charged against **no** budget:
    /// "there is no limit to the number of powers which may be invested in a
    /// familiar" (`:10866`) — unlike a being's own [`Entity::powers`], which the
    /// power-levels budget bounds. Kept sorted via [`Familiar::normalize`].
    /// Source: `:10862-10884` (Empowering the Bond).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub powers: Vec<SupernaturalPower>,
}

/// The highest score a familiar cord can have: "The strength of each of these cords
/// is rated from 0 to +5 … a score of +5 (the maximum)".
///
/// The single home of the rules maximum for the whole engine: [`Familiar::normalize`]
/// clamps the stored fields to it, and [`crate::derived::cord_score`] clamps on read
/// for values that reach a consumer before a normalize pass (a freshly loaded save).
/// Neither restates the number.
///
/// Source: `Ars Magica - Definitive Edition (Core Rules).md:10836`.
pub const MAX_CORD_SCORE: u8 = 5;

impl Familiar {
    /// Sorts both nested lists (Personality Traits by name, invested powers by name),
    /// prunes Characteristics entered as 0, and clamps the three cords to
    /// [`MAX_CORD_SCORE`], for canonical serialization. Called from
    /// [`Entity::normalize`].
    ///
    /// A 0 is pruned rather than kept because a familiar's Characteristics are
    /// display-only — nothing derives from them and none is bought from a point pool
    /// (`:17793`) — so an explicit 0 and an absent entry are the same statement, and
    /// two such familiars must serialize to identical bytes.
    ///
    /// A cord above the maximum is clamped for exactly that reason. The cord fields
    /// are plain `u8`, so a hand-edited or legacy save can carry any value up to 255,
    /// and **every** consumer already routes through
    /// [`crate::derived::cord_score`] — so `cord_bronze: 255` and `cord_bronze: 5`
    /// are indistinguishable to the engine while serializing differently and
    /// *displaying* differently: the panel renders the raw 255 in an input that
    /// declares the maximum, beside read-outs computed from 5. Left alone, saving
    /// rewrites 255 unchanged and the user can never see which figure was used, so
    /// the entered value self-heals on the next save — the same repair, for the same
    /// canonical-serialization reason, as pruning a zero Characteristic.
    pub fn normalize(&mut self) {
        self.personality_traits.sort();
        self.powers.sort();
        self.characteristics.retain(|_, score| *score != 0);
        self.cord_gold = self.cord_gold.min(MAX_CORD_SCORE);
        self.cord_silver = self.cord_silver.min(MAX_CORD_SCORE);
        self.cord_bronze = self.cord_bronze.min(MAX_CORD_SCORE);
    }
}

/// A talisman attunement: a free-text descriptor plus the bonus it confers. The
/// attunement is chosen from the Shape and Material Bonuses Table each time the
/// talisman is prepared or an effect is instilled ("you may also open your
/// talisman to one kind of magic attunement, based on the shape and material of
/// the talisman"). Kept sorted via [`Entity::normalize`].
///
/// **Stored, deliberately not computed.** The bonus is *not* folded into any
/// Casting Total: it applies "only … when the magus is touching the talisman, and
/// only the highest bonus applies", to Casting Scores for Ritual/Formulaic/
/// Spontaneous magic and never to Magic Resistance or lab activities (`:10625`).
/// Which spells an attunement covers is free text ([`Self::description`]), so the
/// engine cannot tell whether a given cell of the casting grid is one it enhances,
/// and "touching the talisman" is a moment of play the model does not represent.
/// It is therefore a situational modifier the player applies at the table — the
/// same call as [`crate::derived::soak`]'s Form bonus, which is surfaced as an
/// entered 0. Recorded as a deferral in RULES.md so it is not read as done.
///
/// Source: `Ars Magica - Definitive Edition (Core Rules).md:10623`, `:10625`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TalismanAttunement {
    /// Free-text description of the attunement (what it enhances).
    pub description: String,
    /// The bonus the attunement confers.
    pub bonus: i8,
}

/// An effect instilled in a magus's talisman: a free-text name and its total
/// level.
///
/// Deliberately **not** an [`EnchantedDevice`]: the two carry different budget
/// contracts and different provenance. A starting *device* is funded by the
/// Redcap-only item-level Virtues and is charged against that budget by
/// [`crate::effective::item_level_used`]; a talisman's instilled effects are
/// funded by nothing the model tracks (see [`Talisman`]) and are charged against
/// nothing. Keeping them apart is the same separation that puts
/// [`SupernaturalPower`] beside [`EnchantedDevice`].
///
/// Source: `Ars Magica - Definitive Edition (Core Rules).md:10621` ("When a magus
/// instills effects into a talisman …").
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TalismanEffect {
    /// Free-text name of the instilled effect.
    pub name: String,
    /// The effect's total level.
    pub level: u16,
}

/// A magus's talisman: his personal enchanted item, not merely a list of
/// attunements.
///
/// "A talisman is a very personal item that contains magics and materials that tie
/// it intimately to you and that can be used as a channel for your magical power"
/// — a magus can have only one at a time. The stored parts are the item's
/// shape/material identity (free text, since shape and material are open-ended),
/// its attunements, and the effects instilled in it. Its *capacity* is derived,
/// never stored: it "depends on the power of the magus to whom it is attuned"
/// (see [`crate::derived::talisman_capacity`]).
///
/// Every field is optional, so an untouched talisman writes no keys at all. Both
/// nested lists are kept sorted via [`Entity::normalize`].
///
/// Source: `Ars Magica - Definitive Edition (Core Rules).md:10603-10625`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Talisman {
    /// Free-text shape/material identity of the item ("an ash staff shod with
    /// silver"). Empty when unset.
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub description: String,
    /// The shape-and-material attunements opened in the talisman.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attunements: Vec<TalismanAttunement>,
    /// The effects instilled in the talisman.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<TalismanEffect>,
}

impl Talisman {
    /// Sorts both nested lists (attunements by description, effects by name) for
    /// canonical serialization. Called from [`Entity::normalize`].
    pub fn normalize(&mut self) {
        self.attunements.sort();
        self.effects.sort();
    }
}

/// Where a magus's Longevity Ritual comes from — a fixed rules taxonomy, rendered
/// via Fluent, never as a raw slug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LongevitySource {
    /// The magus made their own ritual. The bonus is still *entered*, not derived:
    /// it was fixed by the Creo Corpus Lab Total of the season the ritual was made
    /// (Ars Magica - Definitive Edition (Core Rules).md:10662), and the engine only *suggests* a value from today's
    /// Lab Total.
    SelfMade,
    /// An external ritual (e.g. bought or cast by another magus); no suggestion is
    /// offered, since the bonus came from someone else's Lab Total.
    External,
}

impl fmt::Display for LongevitySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LongevitySource::SelfMade => "self_made",
            LongevitySource::External => "external",
        })
    }
}

/// A magus's Longevity Ritual: a stored record of a past event, not a live
/// computation.
///
/// The aging bonus is "+1 bonus for every five points or fraction of Creo Corpus
/// Lab Total" (Ars Magica - Definitive Edition (Core Rules).md:10662) — but that Lab Total is the one the *creating*
/// magus had in the season the ritual was made. Raising Creo/Corpus or moving to a
/// stronger aura afterwards does not improve an existing ritual; the magus must
/// reinvent it, which is a fresh season's work ("If you reinvent the ritual to take
/// advantage of increased Art scores…", Ars Magica - Definitive Edition (Core
/// Rules).md:10670, and a failed ritual's
/// focus is repeated unchanged, :10668). So `bonus` is **player-entered for both
/// sources** and stored: `None` means "not entered yet", never a claimed 0. The
/// engine offers a suggestion from today's Lab Total
/// ([`crate::derived::LongevityHint`]) but never writes it here.
///
/// `focus` is the ritual's culminating focus, "which is appropriate to the magus in
/// question" and must be repeated verbatim if the ritual ever fails
/// (Ars Magica - Definitive Edition (Core Rules).md:10656, :10668) — free text,
/// since the rules give it no mechanics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongevityRitual {
    /// Whether the ritual is self-made or externally provided.
    pub source: LongevitySource,
    /// The player-entered aging bonus; `None` while not yet entered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bonus: Option<i8>,
    /// The ritual's culminating focus, free text; empty while not entered.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:10656.
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub focus: String,
}

/// A Twilight Scar: a minor magical trait (beneficial or annoying) a magus
/// acquires from experiencing Twilight. Free-text — the rules give no mechanical
/// number, only a description — and kept sorted via [`Entity::normalize`].
/// Source: Ars Magica - Definitive Edition (Core Rules).md:9731, :9743.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TwilightScar {
    /// Free-text description of the scar.
    pub description: String,
}

/// One year of a character's aging log.
///
/// The engine does not *roll* — the player types the stress die — but it does
/// everything after that: [`crate::aging::aging_total`] computes the AGING TOTAL
/// and [`crate::aging::resolve_outcome`] resolves it against the table, and a
/// resolved year is recorded here, in full. So an entry answers two different
/// needs with one type:
///
/// - A **resolved** entry carries the whole roll — the `die` the player typed,
///   the `total` it made, the `living_conditions` in force, the `points` awarded
///   and whether the roll advanced the apparent age or called for a Crisis. That
///   is what makes the year exactly reversible: undoing it needs no re-derivation
///   from conditions and a ritual bonus that may since have changed.
/// - A **hand-written** entry carries only what its author typed. `effect` stays
///   authoritative for it: the engine never overwrites free text, and an entry
///   naming no year or age is fully supported.
///
/// Recording the `total` is a historical record of a roll, not a cached
/// derivation, so it does not offend "saves store choices, not resolved values".
///
/// `year` is declared first so the derived `Ord` still sorts the log
/// chronologically via [`Entity::normalize`]. **A log mixing dated and undated
/// entries sorts the undated ones first**, because `None < Some(_)`.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16563-16577 (Aging).
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AgingLogEntry {
    /// The **calendar** year the roll happened. `None` for a character with no
    /// `birth_year` — there is then no calendar year to write — and for a
    /// hand-written entry that names none. First field so `Ord` sorts the log
    /// chronologically; see the type doc for how undated entries sort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    /// The character's age in that year — the key the aging schedule matches a
    /// recorded year on, since a calendar year is unavailable without a birth
    /// year. `None` on a hand-written entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// Free-text description of the year's outcome. Authoritative for a
    /// hand-written entry; a resolved year may leave it empty and let the
    /// structured fields below speak.
    pub effect: String,
    /// The stress die the player typed. The app never rolls.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16567.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub die: Option<i32>,
    /// The AGING TOTAL that die produced, conditions and Longevity Ritual
    /// included. Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16569.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<i32>,
    /// The Living Conditions in force that year, as ids into the
    /// `rules/core/aging.json` table. Recorded per year because the character's
    /// standing [`Entity::living_conditions`] may legitimately change later.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16581-16594.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub living_conditions: BTreeSet<Id>,
    /// The aging points that year awarded, per Characteristic — the player's own
    /// distribution where the table left the choice open (Ars Magica - Definitive Edition (Core Rules).md:16615).
    /// Subtracting exactly these is what reverts the year.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16579.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub points: BTreeMap<Characteristic, u8>,
    /// Whether the roll advanced the character's apparent age by one year.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16577.
    #[serde(default, skip_serializing_if = "is_false")]
    pub apparent_age_increased: bool,
    /// Whether the row called for a Crisis (`:16602`, `:16611`). It says the year
    /// *demanded* one, which is not the same as the Crisis having been rolled: the
    /// four fields below are what records that, and a `true` here with an absent
    /// [`Self::crisis_row`] is a Crisis owed and not yet resolved.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16602, :16611.
    #[serde(default, skip_serializing_if = "is_false")]
    pub crisis: bool,
    /// The Simple Die the player typed for the Crisis, when one was rolled. The
    /// app never rolls this one either.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16621.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crisis_die: Option<i32>,
    /// The CRISIS TOTAL that die produced — "Simple die + age/10 (round up) +
    /// Decrepitude Score", the Decrepitude being the one this very year raised
    /// (`:16619`). Recorded for the same reason [`Self::total`] is: it is the
    /// historical record of a roll, and the score it was made against goes on
    /// climbing afterwards.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16619, :16621.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crisis_total: Option<i32>,
    /// The row of the Crisis Table that total landed on, as an id into
    /// `rules/core/aging.json` — `crisis.minor_illness` and friends. An id, never
    /// a name: the row's text lives in `rules/i18n/<lang>/aging.json`.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16624-16632.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crisis_row: Option<Id>,
    /// How bad that row was, recorded beside its id rather than left to be looked
    /// up again. `AgingRules::crisis` is optional — a ruleset may ship no Crisis
    /// Table at all — so a save can outlive the table that produced it, and the
    /// severity is then the only thing left that says what the character went
    /// through. `None` beside a present [`Self::crisis_row`] is a Bedridden row
    /// (`:16626`, `:16627`), which has no severity because it is time rather than
    /// an illness.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16628-16632.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crisis_severity: Option<CrisisSeverity>,
}

/// `skip_serializing_if` predicate: omits a `u32` field when it is zero.
///
/// Visible crate-wide because the same predicate keeps additive `u32` choices out
/// of a save elsewhere in the engine (`life_stage::LifeStagePlan`); a second copy
/// there would be one more thing to keep in step.
pub(crate) fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// `skip_serializing_if` predicate: omits a `String` field when it is empty.
fn is_empty_str(s: &str) -> bool {
    s.is_empty()
}

/// `skip_serializing_if` predicate: omits an `i32` field when it is zero.
fn is_zero_i32(n: &i32) -> bool {
    *n == 0
}

/// `skip_serializing_if` predicate: omits a `u8` field when it is zero.
fn is_zero_u8(n: &u8) -> bool {
    *n == 0
}

/// `skip_serializing_if` predicate: omits an `i8` field when it is zero.
fn is_zero_i8(n: &i8) -> bool {
    *n == 0
}

/// Where a character's Ability/Art experience comes from — the funding mode, stored
/// explicitly on the entity.
///
/// **Why this is a stored field and not an inference.** Until schema 16 the mode
/// *was* the presence of [`Entity::life_stages`]: a plan meant life-stage funding,
/// no plan meant the typed [`Entity::xp_pool`]. That made switching mode destructive
/// by construction — discarding the plan was how "pool" got recorded — so a round
/// trip through the two modes lost everything typed on either side. With the mode
/// stored, the two sides coexist and the inactive one is merely inert.
///
/// A **closed** enum: every reader `match`es it, so a third funding mode is a
/// compile error until it is handled everywhere.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum AbilityFunding {
    /// [`Entity::xp_pool`] is the authority: one typed total, spent on Abilities and
    /// Arts. The default, and what every directly-entered character uses.
    #[default]
    Pool,
    /// [`Entity::life_stages`] is the authority: the pools are derived from the
    /// stages the character lived through
    /// ([`crate::life_stage::LifeStageRules::budget`]).
    LifeStages,
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
    ///
    /// Read only under [`AbilityFunding::Pool`]; inert (but preserved) under
    /// [`AbilityFunding::LifeStages`], where the pools are derived from the plan
    /// instead. The two are **no longer mutually exclusive** — see
    /// [`Entity::ability_funding`] for why they legitimately coexist.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub xp_pool: u32,
    /// The character's life-stage choices (childhood + later life), read only under
    /// [`AbilityFunding::LifeStages`]. `None` when no plan has ever been recorded —
    /// including every save written before the life stages existed, which is why the
    /// field itself is additive and needed no schema bump of its own.
    ///
    /// **Presence is not the funding mode.** It was until schema 16; it is not now.
    /// A plan kept beside [`AbilityFunding::Pool`] is inert data the player typed and
    /// may come back to, and every site asking "is this character funded by its
    /// stages?" reads [`Entity::ability_funding`] rather than this `Option`.
    ///
    /// Choices only: every figure the stages grant is derived from these plus
    /// [`Entity::age`] (see [`crate::life_stage::LifeStageRules::budget`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub life_stages: Option<LifeStagePlan>,
    /// Which of [`Entity::xp_pool`] / [`Entity::life_stages`] funds this character's
    /// Abilities and Arts. See [`AbilityFunding`] for why the mode is stored rather
    /// than inferred from the plan's presence.
    ///
    /// **Written even when it holds its default**, which departs from this struct's
    /// standing "omit the default" convention. It must be: [`load_entity_migrating`]
    /// dispatches on the key's *absence* to fold a pre-16 save, so omitting `Pool`
    /// would make a pool-funded character that still carries a stored plan reload as
    /// life-stage funded. Absence is meaningful, so presence is mandatory.
    ///
    /// `serde(default)` all the same, so a pre-16 save parses without the key; the
    /// fold then decides its value.
    #[serde(default)]
    pub ability_funding: AbilityFunding,
    /// The furthest guided-wizard phase this character reached, as a **raw slug**.
    ///
    /// **UI/document state on an engine-defined entity, with no validation wired to
    /// it.** It records where the player got to, never anything about the character's
    /// rules legality: no validator reads it, nothing derives from it, and it may
    /// hold a slug this build does not recognise. It lives here rather than in the
    /// frontend store only because it has to survive a save.
    ///
    /// **Deliberately `Option<String>`, not `Option<CreationPhase>`.**
    /// [`CreationPhase`] is a closed `Deserialize` enum, so an unrecognised slug
    /// would be a serde error — and a serde error fails the *whole* load, which would
    /// turn a save carrying a since-removed phase slug into an unopenable file. Saves
    /// carrying the removed `"type"` slug exist, and every future phase rename would
    /// add more. The slug is therefore stored verbatim and resolved **leniently** by
    /// the caller against the loaded profile's `creation_phases`; unresolvable is
    /// treated exactly like absent.
    ///
    /// `None` — no key at all — means "no wizard progress recorded", which is what a
    /// character built in the editor looks like.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wizard_furthest_phase: Option<String>,
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
    /// An optional per-character override of the type profile's base spell-levels
    /// budget (the profile's `spell_levels`, 120 for a magus). A stored *choice*
    /// (saves record choices, so it round-trips): when set it REPLACES the profile
    /// base as the starting budget, then Skilled/Weak Parens
    /// [`Effect::SpellLevels`] modifiers still add on top
    /// ([`crate::effective::spell_levels_base`]). `None` = use the profile base.
    /// Old saves lacking the field default to `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spell_levels_override: Option<u32>,
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
    /// The player's chosen fills for the Virtues/Flaws a non-magus character owes
    /// from its Warping Score ("Effects of Warping", Ars Magica - Definitive Edition (Core Rules).md:16547-16561),
    /// keyed by each owed slot's stable `choice_key` (see
    /// [`crate::effective::warping_owed_grants`]). Off-budget grants: like
    /// `house_choices`/`mythic_choices` the fills are resolved to derived
    /// [`Selection`]s at eval time and never counted against the creation V/F
    /// budget. Empty for magi (exempt — Twilight instead, :16551) and any
    /// character owing nothing. Additive and serde-defaulted so old saves lacking
    /// the field load unchanged (SCHEMA_VERSION stays 13), mirroring
    /// `mastery_abilities`. Kept canonical by the `BTreeMap` key ordering.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub warping_choices: BTreeMap<String, Selection>,
    /// The character's age in years. Drives the age → max-Ability-score cap
    /// (Ars Magica - Definitive Edition (Core Rules).md:2366-2376). `None` when unset (no cap enforced yet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// The character's apparent age in years (Ars Magica - Definitive Edition (Core Rules).md:1155). The resolved
    /// outcome of aging rolls: "the character's apparent age increases by one
    /// year" whenever the AGING TOTAL clears the table's threshold
    /// (Ars Magica - Definitive Edition (Core Rules).md:16577), which [`crate::aging`] resolves and a resolved year
    /// writes here. It stays directly editable — a hand-entered age is never
    /// overwritten — and `None` when unset. Mirrors [`Entity::age`]'s
    /// representation.
    ///
    /// Never an input to the aging roll: the modifier "depends on the character's
    /// **actual, not apparent**, age" (`:16577`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apparent_age: Option<u32>,
    /// Named Personality Traits (value ±3, or ±6 for a Major Personality Flaw's
    /// trait). Kept sorted by name via [`Entity::normalize`]. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub personality_traits: Vec<PersonalityTrait>,
    /// Starting Reputations (each backed by a granting Virtue/Flaw). Kept sorted
    /// via [`Entity::normalize`]. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reputations: Vec<Reputation>,
    /// The realm aura modifier the magus starts under (signed; a Divine aura can be
    /// a penalty). Persisted so the derived-totals slice can reproduce self-made
    /// Longevity / lab totals. Defaults to 0.
    ///
    /// A plain `i32` with no serde-level bound, so [`Entity::normalize`] clamps it
    /// to [`AURA_MODIFIER_MIN`]..=[`AURA_MODIFIER_MAX`] — see that constant pair's
    /// doc for the rules-derived range and its citations.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub aura: i32,
    /// Starting enchanted devices (magi only); each `level` is charged against the
    /// item-level budget. Kept sorted via [`Entity::normalize`]. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<EnchantedDevice>,
    /// The magus's familiar: a full creature statblock — the beast, its own Magic
    /// Might, Characteristics, Size, Personality Traits, the three bond cords and
    /// the powers invested in the bond. See [`Familiar`] for what the statblock
    /// covers and what it deliberately defers. `None` when there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub familiar: Option<Familiar>,
    /// The magus's talisman — identity, attunements and instilled effects.
    /// `None` when there is none (a magus may have at most one).
    /// Replaced the flat `talisman_attunements` list in schema 14; legacy saves
    /// are folded in by [`load_entity_migrating`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub talisman: Option<Talisman>,
    /// The magus's Longevity Ritual. `None` when there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longevity_ritual: Option<LongevityRitual>,
    /// The circumstances the character lives under, as ids into the Living
    /// Conditions table of `rules/core/aging.json`
    /// (Ars Magica - Definitive Edition (Core Rules).md:16581-16594). The other modifier the AGING TOTAL subtracts,
    /// alongside the Longevity Ritual above (`:16567-16569`).
    ///
    /// A **set**, because the asterisked rows are not alternatives: "Modifiers
    /// marked with an asterisk are cumulative with each other" (`:16594`), so a
    /// leper who works in a mine holds two rows and their modifiers add.
    ///
    /// **Empty means the table's baseline, not an unfinished entry.** The table
    /// prints "Average peasant 0" (`:16587`), so a character who names no
    /// condition has a modifier of exactly 0 — the same number the baseline row
    /// carries. Nothing prompts him to choose one.
    ///
    /// Choices, not a resolved value: the modifier is derived by
    /// [`crate::aging::living_conditions_modifier`], while the chosen rows are
    /// what the sheet prints and what a covenant will supply in a later milestone.
    /// An id the table does not know is skipped by that derivation and reported as
    /// a validation finding, never resolved into a silent zero.
    ///
    /// Additive and serde-defaulted, so a save written before the field existed
    /// loads unchanged (no [`SCHEMA_VERSION`] bump). A `BTreeSet` is canonically
    /// ordered by construction, so [`Entity::normalize`] needs no line for it.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub living_conditions: BTreeSet<Id>,
    /// Accrued aging points per Characteristic — the lifetime total gained (the
    /// sheet prints these). Their sum across all Characteristics is the character's
    /// Decrepitude XP ([`crate::effective::decrepitude_points_total`]). The
    /// Characteristic *drops* they force are DERIVED, never stored: once the points
    /// exceed the absolute value of the (aged-down) score the Characteristic drops
    /// and the points reset (see [`crate::effective::aging_drops`] and
    /// [`crate::effective::effective_characteristic_after_aging`]). The drops LOWER
    /// the effective Characteristic used by derived / play stats but never the
    /// bought score creation-legality checks read, so entering an aged character
    /// cannot retroactively make its point-buy illegal.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16579.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub aging_points: BTreeMap<Characteristic, u8>,
    /// Accrued Warping Points. Summed with any grant-derived Warping Points (Warped
    /// by Magic, …) and inverted through the advancement curve to the Warping Score
    /// by [`crate::effective::warping_score`]. Source: Ars Magica - Definitive
    /// Edition (Core Rules).md:16464-16475.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub warping_points: u32,
    /// Free-text description of how the character's Warping manifests (the
    /// source-reflecting Minor/Major Flaw from "Effects of Warping",
    /// Ars Magica - Definitive Edition (Core Rules).md:16547-16561). A pure
    /// annotation carrying NO mechanic: it is
    /// deliberately NOT a Flaw `selection`, because a post-creation warping Flaw
    /// must not count against the creation Virtue/Flaw budget. Empty when unset.
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub warping_effect: String,
    /// Twilight Scars the magus has acquired (free-text). Kept sorted via
    /// [`Entity::normalize`]. Source: Ars Magica - Definitive Edition (Core
    /// Rules).md:9731, :9743.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub twilight_scars: Vec<TwilightScar>,
    /// Free-text narrative of the character's overall aging / decrepitude
    /// (Ars Magica - Definitive Edition (Core Rules).md:16563-16577). A pure
    /// annotation carrying NO mechanic —
    /// Decrepitude is derived from `aging_points`; this only records flavor.
    /// Empty when unset.
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub decrepitude_effect: String,
    /// Per-year aging-roll log (Ars Magica - Definitive Edition (Core
    /// Rules).md:16563-16577). The app never rolls
    /// the die, but it resolves the one the player types — [`crate::aging`]
    /// computes the AGING TOTAL and its outcome — and records the result here as
    /// a structured [`AgingLogEntry`], which is what lets a year be reverted
    /// exactly. Hand-written free-text entries remain fully supported and are
    /// never rewritten. Kept sorted by year via [`Entity::normalize`]; see
    /// [`AgingLogEntry`] for how undated entries sort. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aging_log: Vec<AgingLogEntry>,
    /// The character's name (free-text; no mechanical effect).
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub name: String,
    /// A short one-line description / tagline, e.g. "Knight of the Teutonic
    /// Order" (free-text; no mechanical effect).
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub description: String,
    /// A longer free-text character concept (free-text; no mechanical effect).
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub concept: String,
    /// The character's gender (free-text; no mechanical effect).
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub gender: String,
    /// The character's birth year (flavor; no mechanical effect). `None` when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,
    /// The magus's Wizard's sigil (free-text; no mechanical effect).
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub sigil: String,
    /// The character's covenant name (free-text; no mechanical effect).
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub covenant_name: String,
    /// The magus's parens / master (free-text; no mechanical effect).
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub parens: String,
    /// The weapons, shields, and armor the character carries (each a reference to
    /// a catalogue id). Kept sorted via [`Entity::normalize`]. Defaults to empty.
    /// The derived-totals slice (5i) consumes these to compute combat lines, Soak,
    /// and Encumbrance. Source: Ars Magica - Definitive Edition (Core
    /// Rules).md:16944-17011.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment: Vec<EquipmentSlot>,
    /// The supernatural being's base Might Score + Realm (grog/companion/mythic
    /// companion with a Might Virtue). `None` for ordinary characters. The
    /// effective Might is this base plus same-Realm [`Effect::MightGrant`]s (see
    /// [`crate::effective::effective_might`]). Source: Ars Magica 5e - Realms
    /// of Power - Magic.md:1470-1472.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub might: Option<MightScore>,
    /// The being's supernatural powers; each `level` is charged against the
    /// power-levels budget its Might Virtues grant. Kept sorted via
    /// [`Entity::normalize`]. Defaults to empty. Source: Ars Magica 5e - Realms
    /// of Power - The Infernal.md:4122; Ars Magica 5e - Realms of Power - The
    /// Divine (Revised).md:1977.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub powers: Vec<SupernaturalPower>,
}

/// Current save-format schema version.
///
/// Bumped 9 → 10 when the manual `aging_reductions` map was removed: aging-forced
/// Characteristic drops are now DERIVED from `aging_points`. Old saves are
/// migrated by [`load_entity_migrating`], which folds any legacy `aging_reductions`
/// into `aging_points`.
///
/// Bumped 10 → 11 when the aging/warping annotation fields (`apparent_age`,
/// `warping_effect`, `decrepitude_effect`, `aging_log`) were added. These are
/// purely additive `serde(default)` fields, so old saves load unchanged with no
/// migration code.
///
/// Bumped 11 → 12 when the optional per-character `spell_levels_override` field
/// was added (Issue 11). Purely additive `serde(default)`, so old saves load
/// unchanged with no migration code.
///
/// Bumped 12 → 13 when the optional `parameter` field was added to
/// [`SpellSelection`] (parametrized meta-magic Vim spells like Wizard's Boost,
/// Issue 8/10). Purely additive `serde(default)`, so old saves load unchanged
/// with no migration code.
///
/// The `SpellSelection::mastery_abilities` field and the `warping_choices` map
/// (the off-budget owed-warping V/F fills, Issue E) were both later additive
/// `serde(default)` additions that did NOT bump the version: an old save omits
/// the key and deserializes to the empty default; a new save with the default
/// omits it on write, so version 13 saves remain byte-compatible both ways. The
/// same applies to [`LongevityRitual::focus`] (M5.5a): retaining
/// `bonus: Option<i8>` made the accompanying semantic change — the self-made
/// bonus is now *entered* rather than derived — self-migrating, so a pre-5.5a
/// self-made ritual simply loads as "bonus not entered" and the hint beside the
/// input suggests the number the old build derived. A pre-5.5a *external* ritual
/// that was added but never filled in stored a seeded 0 and now reads as a
/// deliberate 0. Neither is silently repaired: guessing would re-invent the live
/// derivation 5.5a exists to delete. The [`Familiar`] statblock fields (M5.5c —
/// `animal`, `might`, `characteristics`, `size`, `personality_traits`, `powers`)
/// are the same kind of addition: a pre-5.5c familiar carried only a name and the
/// three cords, omits every new key, and writes byte-identical JSON, so the shape
/// grew without a bump.
///
/// Bumped 13 → 14 when the flat `talisman_attunements` list was replaced by
/// [`Entity::talisman`], the magus's talisman as an *item* (identity +
/// attunements + instilled effects). This is a field **move**, not an addition,
/// so [`load_entity_migrating`] folds any legacy list into the new
/// `talisman.attunements`. The fold neither infers nor discards: the attunements are
/// carried over verbatim, the identity/effects the old shape never stored stay empty
/// rather than being invented, and a legacy list that cannot deserialize fails the
/// load instead of quietly folding in nothing. So unlike the `aging_reductions`
/// migration — which *infers* a point total — it needs no [`LoadedEntity`] notice
/// flag.
///
/// Bumped 14 → 15 when [`AgingLogEntry`] widened from `{ year, effect }` into the
/// full record of a resolved aging year (`age`, `die`, `total`,
/// `living_conditions`, `points`, `apparent_age_increased`, `crisis`) and `year`
/// became `Option<i32>`. Every added field is `serde(default,
/// skip_serializing_if)` and serde reads a bare `year` number into `Some`, so a
/// schema-14 save loads unchanged and **no migration code exists**. What is not
/// backward-compatible is the *forward* direction: a schema-15 save may omit
/// `year` entirely (a character with no `birth_year` has no calendar year to
/// write), which a schema-14 reader rejects — and a version number is exactly how
/// an older build learns not to try.
///
/// Bumped 15 → 16 for **two** fields at once — [`Entity::ability_funding`] (the
/// explicit funding discriminator) and [`Entity::wizard_furthest_phase`] (the
/// guided-wizard progress slug). Batching them is deliberate: two sequential bumps
/// would mean two migrations, two round-trip-test updates, and a window in which a
/// save written at 16 is unreadable by a build at 17.
///
/// `ability_funding` earns the bump on its own. It is the **only** field on `Entity`
/// that is written even when it holds its default, because
/// [`load_entity_migrating`] dispatches on its absence to infer the mode a pre-16
/// save recorded only implicitly (a `life_stages` plan meant life-stage funding). So
/// the forward direction breaks in the way a bump exists to announce: a schema-16
/// save may carry both a plan and a nonzero `xp_pool`, a combination a schema-15
/// reader reports as `life_stage_xp_pool_conflict` — a finding this bump retires.
/// `wizard_furthest_phase` alone would have been purely additive
/// (`skip_serializing_if`, so byte-identical when unset) and earned nothing.
pub const SCHEMA_VERSION: u32 = 16;

/// The lowest rules-legal [`Entity::aura`] modifier: a Divine aura acting on
/// Infernal-realm powers, "– (5 x aura)", at the highest aura rating the rules
/// give ("can be rated in power on a scale from 1 to 10").
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:17390 (rating scale),
/// :17404-17409 (Realm Interaction Table, the ×5 Divine-on-Infernal entry).
///
/// Enforced in two places. [`Entity::normalize`] clamps the stored value
/// (self-healing, the same treatment [`MAX_CORD_SCORE`] gets from
/// `Familiar::normalize`), and the arithmetic that sums `aura` into casting and
/// lab totals — `derived::casting::formulaic_casting_score` and `penetration`,
/// `derived::lab::creo_corpus_lab_total`, and the shared `sum()` — routes through
/// `derived::saturating_i32_sum`, which widens to `i64` and clamps back, so an
/// out-of-range value reaching a total before a normalize pass saturates instead
/// of wrapping. (`derived.rs` re-exports all three, so the flatter
/// `crate::derived::…` paths still resolve; the submodules are named here because
/// that is where the arithmetic actually lives.)
pub const AURA_MODIFIER_MIN: i32 = -50;
/// The highest rules-legal [`Entity::aura`] modifier: no table entry in the Realm
/// Interaction Table multiplies the aura rating by more than ×1 in the positive
/// direction ("+aura", a realm's own aura on its own powers), at the highest
/// aura rating the rules give. See [`AURA_MODIFIER_MIN`] for the citations and
/// what is (and is not) enforced.
pub const AURA_MODIFIER_MAX: i32 = 10;

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
            life_stages: None,
            ability_funding: AbilityFunding::Pool,
            wizard_furthest_phase: None,
            art_scores: Vec::new(),
            spells: Vec::new(),
            spell_levels_override: None,
            house: None,
            house_choices: BTreeMap::new(),
            mythic_type: None,
            mythic_choices: BTreeMap::new(),
            warping_choices: BTreeMap::new(),
            age: None,
            apparent_age: None,
            personality_traits: Vec::new(),
            reputations: Vec::new(),
            aura: 0,
            devices: Vec::new(),
            familiar: None,
            talisman: None,
            longevity_ritual: None,
            living_conditions: BTreeSet::new(),
            aging_points: BTreeMap::new(),
            warping_points: 0,
            warping_effect: String::new(),
            twilight_scars: Vec::new(),
            decrepitude_effect: String::new(),
            aging_log: Vec::new(),
            name: String::new(),
            description: String::new(),
            concept: String::new(),
            gender: String::new(),
            birth_year: None,
            sigil: String::new(),
            covenant_name: String::new(),
            parens: String::new(),
            equipment: Vec::new(),
            might: None,
            powers: Vec::new(),
        }
    }

    /// Sort selections, ability scores, art scores, spells, personality traits,
    /// reputations, devices, the familiar's and talisman's nested lists, twilight
    /// scars, the aging log (by year), equipment and powers for canonical
    /// serialization. Also clamps `aura` to
    /// [`AURA_MODIFIER_MIN`]..=[`AURA_MODIFIER_MAX`], the same self-healing
    /// treatment `Familiar::normalize` gives an out-of-range cord score.
    /// (`characteristics` and `aging_points` are `BTreeMap`s, already id-ordered.)
    pub fn normalize(&mut self) {
        self.aura = self.aura.clamp(AURA_MODIFIER_MIN, AURA_MODIFIER_MAX);
        self.selections.sort();
        self.ability_scores.sort();
        self.art_scores.sort();
        // Sort each spell's chosen mastery abilities (stable, keeping duplicates —
        // a repeatable ability may legitimately appear more than once) before
        // sorting the spell list itself, so the whole entity serializes canonically.
        for spell in &mut self.spells {
            spell.mastery_abilities.sort();
        }
        self.spells.sort();
        self.personality_traits.sort();
        self.reputations.sort();
        self.devices.sort();
        if let Some(familiar) = &mut self.familiar {
            familiar.normalize();
        }
        if let Some(talisman) = &mut self.talisman {
            talisman.normalize();
        }
        self.twilight_scars.sort();
        self.aging_log.sort();
        self.equipment.sort();
        self.powers.sort();
    }
}

/// The outcome of loading an entity save, including any schema migration applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedEntity {
    /// The entity, migrated to the current [`SCHEMA_VERSION`].
    pub entity: Entity,
    /// Characteristics whose legacy `aging_reductions` drops were folded into
    /// `aging_points` during migration (empty when nothing was migrated, sorted).
    /// The caller surfaces a localized notice; the engine holds no user-facing
    /// string.
    pub migrated_aging_characteristics: Vec<Characteristic>,
}

/// The minimal lifetime aging-point total that forces exactly `drops`
/// Characteristic drops starting from a `bought` score, under the derived rule
/// (each drop needs one more point than the absolute value of the current
/// aged-down score). Used to reconstruct a legacy `aging_reductions` count as
/// `aging_points`. Source: Ars Magica - Definitive Edition (Core Rules).md:16579.
fn minimal_aging_points_for_drops(bought: i32, drops: u32) -> u32 {
    let mut total = 0u32;
    for i in 0..drops {
        let aged = i64::from(bought) - i64::from(i);
        let threshold = u32::try_from(aged.unsigned_abs()).unwrap_or(u32::MAX);
        total = total.saturating_add(threshold.saturating_add(1));
    }
    total
}

/// Folds a legacy (schema ≤ 13) `talisman_attunements` value into
/// [`Entity::talisman`].
///
/// Two shapes are ambiguous and are decided here:
/// - An **empty** legacy list creates no talisman. A magus with no attunements had
///   no talisman recorded either, and a phantom empty item would show up as an
///   owned talisman in the UI. (An app-written save can never contain one —
///   `skip_serializing_if = "Vec::is_empty"` omits the key — so this only covers a
///   hand-edited file. The version is still bumped by the caller, because the
///   key's presence proves the old shape; the `aging_reductions` fold does the
///   same.)
/// - A save carrying **both** keys keeps the new `talisman` and drops the legacy
///   list unmerged — but only when the new shape already carries **attunements**.
///   `attunements` is the one field the legacy list can migrate into, so it is the
///   only field whose contents can make merging duplicate anything: attunements a
///   player already moved across by hand. The legacy list is then not even parsed —
///   it is discarded by that decision, so its shape cannot matter.
///
///   A talisman with **no** attunements does not win, however much its other fields
///   hold. `"talisman": {}` states nothing at all (every [`Talisman`] field is
///   `skip_serializing_if`, so an untouched talisman the app itself wrote serializes
///   as exactly `{}`, and the key's mere presence is no evidence the player moved
///   anything), and `{"description": "An ash staff"}` or an effects-only talisman
///   says nothing about *attunements* either. In all of those the list is folded into
///   the existing talisman — filling its `attunements` while its own fields are left
///   as written. Letting them take precedence would drop the list unparsed while
///   [`load_entity_migrating`] still stamps [`SCHEMA_VERSION`] — the same permanent
///   loss the error path below exists to prevent.
///
/// A legacy list that **cannot** deserialize (a `bonus` outside `i8`, a bonus
/// written as a JSON string, `null` in place of the list) is an error, propagated to
/// [`load_entity_migrating`]'s caller. Swallowing it would fold in nothing while the
/// caller still stamps the current [`SCHEMA_VERSION`], so the load would report
/// success and the next save would rewrite the file without the legacy key —
/// destroying the attunements. Failing the load leaves the file untouched.
fn fold_legacy_talisman(
    entity: &mut Entity,
    legacy: serde_json::Value,
) -> Result<(), serde_json::Error> {
    let new_shape_carries_attunements = entity
        .talisman
        .as_ref()
        .is_some_and(|talisman| !talisman.attunements.is_empty());
    if new_shape_carries_attunements {
        return Ok(());
    }
    let attunements: Vec<TalismanAttunement> = serde_json::from_value(legacy)?;
    if attunements.is_empty() {
        return Ok(());
    }
    // Fold into the existing talisman so a description-only or effects-only one keeps
    // its own data; only `attunements` (empty, per the gate above) is filled in.
    entity
        .talisman
        .get_or_insert_with(Talisman::default)
        .attunements = attunements;
    Ok(())
}

/// The three items whose `being` parameter changed from a free-text domain to an
/// enumerated one, so a save written before that change holds a typed label where
/// a `being.*` id now belongs.
///
/// `virtue.alluring_to_beings` and `flaw.magical_being_companion` carry a `being`
/// parameter too and are deliberately **absent**: both are still `text` domains
/// (their value is the player's own description), so folding them would rewrite a
/// perfectly legal value into an id their domain never declared.
const REFOLDED_BEING_ITEMS: &[&str] = &[
    "flaw.offensive_to_beings",
    "flaw.unbearable_to_beings",
    "virtue.inoffensive_to_beings",
];

/// The parameter key [`REFOLDED_BEING_ITEMS`] all share.
const BEING_PARAM_KEY: &str = "being";

/// The labels a pre-enumeration save can hold in a `being` slot, mapped to the id
/// each one names: six classes of being × the two shipped languages, plus the three
/// German **dative** forms the rulebook itself prints.
///
/// **These strings are matched, never rendered**, so they are not a breach of the
/// no-hardcoded-user-facing-strings rule: nothing here reaches a user, and the
/// localized labels the UI *does* render still live only in
/// `rules/i18n/<lang>/virtues_flaws.json`.
///
/// The table is frozen on purpose rather than read from those files. Two reasons:
/// [`load_entity_migrating`] holds neither a [`Ruleset`](crate::ruleset::Ruleset)
/// nor an i18n map, so a live lookup is impossible without an API change; and it
/// would be wrong even then, because editing a label later would silently change
/// what a decade-old save migrates *to*. What a v0.2.x app printed is history, and
/// history does not move.
///
/// Sources, all three read at implementation time. The i18n files supply the labels
/// the picker showed:
/// `rules/i18n/en/virtues_flaws.json` and `rules/i18n/de/virtues_flaws.json`, both
/// at the `being.*` entries. The rulebook supplies the wording a player copying
/// from the page would have typed:
/// Ars Magica - Definitive Edition (Core Rules).md:4135, :6526, :6893, and the
/// German mirror at the same line numbers. The English lists and the German lists
/// at `:6526` / `:6893` fold onto the i18n labels exactly; the German `:4135` list
/// is in the dative, which contributes the three extra keys below. No synonym
/// beyond what those print is invented — an unrecognised value is left exactly as
/// typed.
///
/// Every key is distinct under [`fold_being_label`], and no key is ever equal to a
/// `being.*` id, which is what makes the fold both unambiguous and idempotent.
const LEGACY_BEING_LABELS: &[(&str, &str)] = &[
    // rules/i18n/en/virtues_flaws.json
    ("Animals", "being.animals"),
    ("Demons", "being.demons"),
    ("Divine Beings", "being.divine"),
    ("Faeries", "being.faeries"),
    ("Magical Creatures", "being.magical_creatures"),
    ("Mundane Humans", "being.mundane_humans"),
    // rules/i18n/de/virtues_flaws.json
    ("Tiere", "being.animals"),
    ("Dämonen", "being.demons"),
    ("Göttliche Wesen", "being.divine"),
    ("Feen", "being.faeries"),
    ("Magische Kreaturen", "being.magical_creatures"),
    ("Sterbliche Menschen", "being.mundane_humans"),
    // The German **dative** forms, printed by the Inoffensive entry at
    // `:4135` — "…von Wesen verbunden: Tieren, göttlichen Wesen, Feen, Dämonen
    // oder magischen Kreaturen." A player filling the old free-text box while
    // reading that page copied the inflected form off it, so these are recovered
    // too. Transcribed, not declined: "Feen" and "Dämonen" are identical in the
    // dative and are already above, and the fourth class has no dative form here
    // to take — `sterbliche Menschen` appears only in the two entries that print
    // the nominative (`:6526`, `:6893`), so no "sterblichen Menschen" key is
    // invented.
    ("Tieren", "being.animals"),
    ("göttlichen Wesen", "being.divine"),
    ("magischen Kreaturen", "being.magical_creatures"),
];

/// Folds a typed `being` label for comparison: lowercased and stripped of all
/// whitespace, so "  Divine  beings " and "Divine Beings" are one value. The old
/// slot was a free-text box, and case and spacing were never part of the choice.
fn fold_being_label(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Rewrites a legacy typed `being` label into the `being.*` id it names, in place.
///
/// A no-op for anything else: another item, a missing `being` key, a value already
/// holding an id, or a label the frozen table does not recognise. That last case is
/// the deliberate one — the player re-picks it once, and nothing is guessed on
/// their behalf.
fn fold_legacy_being_param(selection: &mut Selection) {
    if !REFOLDED_BEING_ITEMS.contains(&selection.item_ref.as_str()) {
        return;
    }
    let Some(typed) = selection.params.get(BEING_PARAM_KEY) else {
        return;
    };
    let folded = fold_being_label(typed.as_str());
    let Some((_, id)) = LEGACY_BEING_LABELS
        .iter()
        .find(|(label, _)| fold_being_label(label) == folded)
    else {
        return;
    };
    selection
        .params
        .insert(BEING_PARAM_KEY.to_string(), Id::new(*id));
}

/// Applies [`fold_legacy_being_param`] everywhere a save can hold a [`Selection`]:
/// the bought list plus the three resolved-pick maps, since a House choice, a
/// mythic-type pick or a Warping fill can name one of these items too.
fn fold_legacy_being_params(entity: &mut Entity) {
    for selection in &mut entity.selections {
        fold_legacy_being_param(selection);
    }
    for choices in [
        &mut entity.house_choices,
        &mut entity.mythic_choices,
        &mut entity.warping_choices,
    ] {
        for selection in choices.values_mut() {
            fold_legacy_being_param(selection);
        }
    }
}

/// Trims surrounding whitespace off every parameter value of one selection.
///
/// Parameter values establish a selection's **identity** in four places that all
/// compare them byte-for-byte: the duplicate-selection key (the whole params map),
/// a grant's `options.contains(pick)`, a mythic type's required-Virtue check, and
/// [`Entity::normalize`]'s sort. So "Wolf Shape ", "Wolf Shape" and " Wolf Shape"
/// were three distinct targets, and the per-power cap counted them separately.
///
/// Every domain is trimmed, not just [`ParameterDomain::Text`] — this function has
/// no ruleset to ask, and no domain has a legal value with an edge of whitespace:
/// an id never does, and a `text` value's padding was never part of the player's
/// choice. Case is deliberately **left alone**: the rules ask for no case-folding,
/// and two powers a player capitalised differently are their own business.
///
/// A value that trims to nothing is kept as the empty string rather than deleted,
/// so the player sees `missing_param` naming the box to fill — the same issue an
/// absent key raises, which is what makes a blank power and an unrecorded one read
/// identically.
fn trim_selection_params(selection: &mut Selection) {
    let padded: Vec<String> = selection
        .params
        .iter()
        .filter(|(_, value)| value.as_str().trim() != value.as_str())
        .map(|(key, _)| key.clone())
        .collect();
    for key in padded {
        let trimmed = Id::new(selection.params[&key].as_str().trim());
        selection.params.insert(key, trimmed);
    }
}

/// Applies [`trim_selection_params`] everywhere a save can hold a [`Selection`]:
/// the bought list plus the three resolved-pick maps, exactly as
/// [`fold_legacy_being_params`] does — a House choice, a mythic-type pick or a
/// Warping fill carries params of its own, and a padded one falls off its own menu
/// (`options.contains(pick)` is full [`Selection`] equality).
fn trim_all_selection_params(entity: &mut Entity) {
    for selection in &mut entity.selections {
        trim_selection_params(selection);
    }
    for choices in [
        &mut entity.house_choices,
        &mut entity.mythic_choices,
        &mut entity.warping_choices,
    ] {
        for selection in choices.values_mut() {
            trim_selection_params(selection);
        }
    }
}

/// Deserializes an entity from JSON, applying backward-compatible save
/// migrations, and reports what was migrated.
///
/// Saves at schema ≤ 9 carried a manual `aging_reductions` map of completed
/// Characteristic drops. The current model derives those drops from
/// `aging_points` (Ars Magica - Definitive Edition (Core Rules).md:16579), so any legacy reductions are folded into
/// `aging_points` as the minimal point total that reproduces the same number of
/// drops. Because every aging point counts toward Decrepitude — including those
/// "lost" to a drop — this fold also corrects the old model's Decrepitude
/// under-count.
///
/// Saves at schema ≤ 13 carried a flat `talisman_attunements` list. Schema 14
/// models the talisman as an item ([`Talisman`]), so any legacy list is folded
/// into `talisman.attunements`. See [`fold_legacy_talisman`] for the ambiguous
/// shapes it has to decide.
///
/// Saves at schema ≤ 15 carried no `ability_funding` key, because the funding mode
/// was not stored at all — it *was* the presence of `life_stages`. That inference is
/// applied here, **once at load** rather than on every read: a plan present means
/// [`AbilityFunding::LifeStages`], no plan means [`AbilityFunding::Pool`]. From 16 on
/// the key is always written (see [`Entity::ability_funding`]), so its absence is
/// exactly the pre-16 signal and nothing else. `wizard_furthest_phase` needs no fold:
/// absent stays `None`, which its reader treats as "no wizard progress recorded".
///
/// Saves written before the three `being` parameters became enumerated domains hold
/// a **typed label** where a `being.*` id now belongs, so every such save reported
/// `unknown_param_value` on open. Those labels are folded back onto their ids by
/// [`fold_legacy_being_params`]. This one is unlike every fold above it: it has **no
/// version signal at all**, because the closed lists were a *ruleset* change, not a
/// save-format change — nothing in a save distinguishes the two eras, and no
/// `SCHEMA_VERSION` bump could. So the fold is **value-driven and idempotent**: it
/// rewrites the twelve labels it recognises and leaves everything else, an id
/// included, exactly as written. For the same reason it does **not** stamp
/// `SCHEMA_VERSION` — no schema moved, and stamping would rewrite the version of a
/// save that was already current.
///
/// Every parameter value is also **trimmed** here, by
/// [`trim_all_selection_params`]. Nothing trimmed a value arriving from a save
/// file, and parameter values are compared byte-for-byte wherever a selection's
/// identity is decided, so a padded descriptor was a target of its own. This too is
/// value-driven, idempotent and version-free, and it belongs at **load** rather than
/// in [`Entity::normalize`]: normalize runs on every *save* and sorts selections by
/// `(ref, params)`, so trimming there would silently reorder the rows of a file the
/// player had merely opened — against the zero-noise-diff convention. Trimming on
/// load lets the reordering settle once, at open.
///
/// What it deliberately does not do: Folk Magic's `category` and the three per-power
/// Flaws' `power` were never *stored* in the old shape, so there is nothing to
/// migrate from. They stay a visible `missing_param` naming the item and key rather
/// than a fabricated placeholder — inventing one would invent a character's rules
/// choices. Likewise a genuine `too_many_selections` (an over-cap Puissant Art the
/// engine was previously blind to) is a rules finding, not a format problem, and
/// survives untouched.
///
/// Dispatch is on legacy-key *presence*, never on the recorded version: a
/// hand-edited save may carry any `schema_version` alongside either shape. Plain
/// `serde` deserialization still works for current saves; this wrapper only adds
/// the folds and a migration report.
///
/// Either legacy value failing to deserialize fails the **whole load**, exactly as a
/// malformed current field does. A fold that quietly yielded nothing would still get
/// the version stamped, so the load would look successful and the next save would
/// drop the legacy key — losing the data permanently.
pub fn load_entity_migrating(json: &str) -> Result<LoadedEntity, serde_json::Error> {
    let mut value: serde_json::Value = serde_json::from_str(json)?;
    let legacy = value
        .as_object_mut()
        .and_then(|obj| obj.remove("aging_reductions"));
    let legacy_attunements = value
        .as_object_mut()
        .and_then(|obj| obj.remove("talisman_attunements"));
    // Dispatch on the key's absence, never on the recorded `schema_version`: a
    // hand-edited save may carry any version alongside either shape. Read before the
    // deserialization below, because `serde(default)` would make the two
    // indistinguishable afterwards.
    let funding_absent = !value
        .as_object()
        .is_some_and(|obj| obj.contains_key("ability_funding"));
    let mut entity: Entity = serde_json::from_value(value)?;

    // Both folds below are value-driven and idempotent, and deliberately stamp no
    // version — see the `being` and whitespace paragraphs above. The trim runs
    // first, so the being fold sees canonical values.
    trim_all_selection_params(&mut entity);
    fold_legacy_being_params(&mut entity);

    if let Some(legacy_attunements) = legacy_attunements {
        fold_legacy_talisman(&mut entity, legacy_attunements)?;
        entity.schema_version = SCHEMA_VERSION;
    }

    if funding_absent {
        // The pre-16 rule, applied once: the plan's presence WAS the mode. A value
        // that will not deserialize has already failed the whole load above, so this
        // fold cannot silently yield nothing while the version gets stamped.
        entity.ability_funding = if entity.life_stages.is_some() {
            AbilityFunding::LifeStages
        } else {
            AbilityFunding::Pool
        };
        entity.schema_version = SCHEMA_VERSION;
    }

    let mut migrated_aging_characteristics = Vec::new();
    if let Some(legacy) = legacy {
        // The save predates schema 10 (manual `aging_reductions`); fold it in and
        // bump the version. Current saves (no legacy field) are left untouched so
        // a load is a faithful, byte-stable round trip. A map that cannot
        // deserialize fails the load for the same reason as the talisman list: a
        // silently empty fold would still bump the version, and the next save would
        // drop the key.
        let reductions: BTreeMap<Characteristic, u8> = serde_json::from_value(legacy)?;
        for (characteristic, drops) in reductions {
            if drops == 0 {
                continue;
            }
            let bought = entity
                .characteristics
                .get(&characteristic)
                .copied()
                .map_or(0i32, i32::from);
            let add = minimal_aging_points_for_drops(bought, u32::from(drops));
            let slot = entity.aging_points.entry(characteristic).or_insert(0);
            *slot = slot.saturating_add(u8::try_from(add).unwrap_or(u8::MAX));
            migrated_aging_characteristics.push(characteristic);
        }
        entity.schema_version = SCHEMA_VERSION;
    }

    migrated_aging_characteristics.sort();
    Ok(LoadedEntity {
        entity,
        migrated_aging_characteristics,
    })
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
    /// Human-readable display name. May be a template carrying `{param}` tokens
    /// (`"{area} Lore"`), which the caller fills with the chosen instance or with a
    /// localized hint like `"(Area)"`.
    pub name: String,
    /// The name to show when the template's instance is **unfilled**, for the rare
    /// entry whose literal already carries the qualifier the generic hint supplies.
    ///
    /// Opt-in and normally absent: for the great majority of templates the hint is
    /// exactly right ("Puissant (Ability)", "Ways Of The (Land)"), and suppressing it
    /// would leave a dangling head word. Only a name holding **both** a `{token}` and
    /// a parenthetical literal doubles — `"{language} (Dead Language)"` plus the
    /// `"(Language)"` hint reads *"(Language) (Dead Language)"* — and those entries
    /// name their unfilled form here instead (`"Dead Language"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_unfilled: Option<String>,
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

    fn house(id: &str) -> Prereq {
        Prereq::House(Id::new(id))
    }

    /// A bare `House` leaf — the shape all four Outer-Mystery Virtues ship —
    /// excludes only a character known to be in a *different* House. A matching
    /// House and an unknown House both leave the item admissible, the latter
    /// mirroring the full evaluator's `Unknown` for a magus with no House yet.
    #[test]
    fn a_house_leaf_conflicts_only_with_a_known_different_house() {
        let bjornaer = house("house.bjornaer");
        assert!(bjornaer.conflicts_with_house(Some(&Id::new("house.jerbiton"))));
        assert!(!bjornaer.conflicts_with_house(Some(&Id::new("house.bjornaer"))));
        assert!(!bjornaer.conflicts_with_house(None));
    }

    /// Every leaf that is not a `House` is undecided here, so it can neither
    /// exclude an item from an open menu nor rescue one that a `House` leaf
    /// beside it has already ruled out.
    #[test]
    fn a_non_house_prerequisite_never_conflicts() {
        assert!(!Prereq::Has(Id::new("virtue.x")).conflicts_with_house(None));
        assert!(!Prereq::IsMagus.conflicts_with_house(Some(&Id::new("house.bjornaer"))));
        assert!(
            !Prereq::AbilityMin {
                ability: Id::new("ability.awareness"),
                score: 1,
            }
            .conflicts_with_house(Some(&Id::new("house.bjornaer")))
        );
        assert!(
            !Prereq::ArtMin {
                art: Id::new("art.creo"),
                score: 1,
            }
            .conflicts_with_house(Some(&Id::new("house.bjornaer")))
        );
    }

    /// The three quantifiers fold the house-only tri-state the way their boolean
    /// meaning demands, with an undecided child blocking a verdict wherever it
    /// could still change the answer.
    #[test]
    fn quantifiers_fold_the_house_only_tri_state() {
        let elsewhere = Some(Id::new("house.jerbiton"));
        let elsewhere = elsewhere.as_ref();
        let home = Some(Id::new("house.bjornaer"));
        let home = home.as_ref();
        let has = Prereq::Has(Id::new("virtue.x"));

        // All: one conflicting child sinks the whole expression, even beside an
        // undecided sibling; a satisfiable one leaves the verdict to that sibling.
        let all = Prereq::All(vec![house("house.bjornaer"), has.clone()]);
        assert!(all.conflicts_with_house(elsewhere));
        assert!(!all.conflicts_with_house(home));

        // Any: only conflicts when EVERY branch does — an undecided branch keeps
        // the door open.
        let either = Prereq::Any(vec![house("house.bjornaer"), house("house.criamon")]);
        assert!(either.conflicts_with_house(elsewhere));
        assert!(!either.conflicts_with_house(home));
        assert!(
            !Prereq::Any(vec![house("house.bjornaer"), has.clone()])
                .conflicts_with_house(elsewhere)
        );

        // Nor: a satisfied child is what sinks it, so "not a Bjornaer" conflicts
        // for a Bjornaer and not for anyone else.
        let not_bjornaer = Prereq::Nor(vec![house("house.bjornaer")]);
        assert!(not_bjornaer.conflicts_with_house(home));
        assert!(!not_bjornaer.conflicts_with_house(elsewhere));
        assert!(!not_bjornaer.conflicts_with_house(None));
    }

    /// K8 defense in depth, mirroring `evaluate_prereq`'s own guard: past
    /// [`PREREQ_MAX_DEPTH`] the expression counts as undecided rather than
    /// recursing further. Unreachable for any ruleset that passed
    /// `Ruleset::validate_prereq_refs`, so the guard is exercised by calling the
    /// private walker with a depth already over the limit.
    #[test]
    fn house_only_evaluation_stops_at_the_depth_limit() {
        let bjornaer = house("house.bjornaer");
        let elsewhere = Id::new("house.jerbiton");
        assert_eq!(
            bjornaer.house_only_value(Some(&elsewhere), PREREQ_MAX_DEPTH),
            Some(false),
            "at exactly the limit the leaf still evaluates"
        );
        assert_eq!(
            bjornaer.house_only_value(Some(&elsewhere), PREREQ_MAX_DEPTH + 1),
            None,
            "past the limit the expression is undecided, not a conflict"
        );
    }

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
        check(ParameterDomain::Technique);
        check(ParameterDomain::Form);
        check(ParameterDomain::Item);
        check(ParameterDomain::Characteristic);
        check(ParameterDomain::Enumerated);
        check(ParameterDomain::Text);
        check(CastingScope::All);
        check(CastingScope::Formulaic);
        check(CastingScope::Ritual);
        check(CastingScope::FormulaicRitual);
        check(CastingScope::Spontaneous);
        check(HalvableTotal::SpontaneousCasting);
        check(HalvableTotal::LabEnchanting);
        check(HalvableTotal::LabLongevity);
        check(HalvableTotal::Penetration);
        check(HalvableTotal::MagicResistance);
        check(CombatStat::Initiative);
        check(CombatStat::Attack);
        check(CombatStat::Defense);
        check(CombatStat::Damage);
        check(HealthTrack::FatiguePenalty);
        check(HealthTrack::WoundPenalty);
        check(HealthTrack::FatigueRoll);
        check(HealthTrack::CastingFatigue);
        check(HealthTrack::Recovery);
        check(MagicResistanceEffect::NoFormBonus);
        check(MagicResistanceEffect::AuraBonus);
        check(MagicResistanceEffect::SusceptibleDivine);
        check(MagicResistanceEffect::SusceptibleFaerie);
        check(MagicResistanceEffect::SusceptibleInfernal);
        check(AgingEffect::AgingRoll);
        check(AgingEffect::LongevityBonus);
        check(AgingEffect::NoAging);
        check(AgingEffect::NoApparentAging);
        check(AgingEffect::Decrepitude);
        check(AgingEffect::LivingConditions);
        check(AdvancementSource::Taught);
        check(AdvancementSource::Book);
        check(AdvancementSource::Vis);
        check(AdvancementSource::Practice);
        check(AdvancementSource::Adventure);
        check(AdvancementSource::Insight);
        check(AdvancementSource::Teaching);
        check(AdvancementSource::SpellMastery);
        check(AdvancementSource::All);
        check(SpecialCasting::QuietWords);
        check(SpecialCasting::SubtleGestures);
        check(SpecialCasting::DeftForm);
        check(SpecialCasting::Diedne);
        check(SpecialCasting::FaerieRaised);
        check(SpecialCasting::LifeLinkedSpontaneous);
        check(SpecialCasting::SpellImprovisation);
        check(SpecialCasting::Mercurian);
        check(SpecialCasting::LifeBoost);
        check(SpecialCasting::Circumstantial);
        check(LongevitySource::SelfMade);
        check(LongevitySource::External);
        check(crate::validation::IssueSeverity::Error);
        check(crate::validation::IssueSeverity::Warning);
        check(crate::spell::SpellRange::ArcaneConnection);
        check(crate::spell::SpellRange::Personal);
        check(crate::spell::SpellDuration::Year);
        check(crate::spell::SpellDuration::Momentary);
        check(crate::spell::SpellTarget::Boundary);
        check(crate::spell::SpellTarget::Vision);
        check(Realm::Magic);
        check(Realm::Faerie);
        check(Realm::Divine);
        check(Realm::Infernal);
        check(crate::derived::FatigueTier::Fresh);
        check(crate::derived::FatigueTier::Winded);
        check(crate::derived::FatigueTier::Weary);
        check(crate::derived::FatigueTier::Tired);
        check(crate::derived::FatigueTier::Dazed);
        check(crate::derived::WoundBand::Light);
        check(crate::derived::WoundBand::Medium);
        check(crate::derived::WoundBand::Heavy);
        check(crate::derived::WoundBand::Incapacitating);
        check(crate::derived::WoundBand::Dead);
        check(crate::derived::ModifierFamily::Aging);
        check(crate::derived::ModifierFamily::Advancement);
        check(crate::derived::ModifierFamily::SpecialCasting);
        check(crate::derived::ModifierFamily::AbilityRoll);
        check(crate::derived::ModifierFamily::HealthRoll);
        check(crate::derived::ModifierFamily::MagicResistance);
    }

    #[test]
    fn might_and_powers_round_trip() {
        // A supernatural-being's Might score + realm and its free-text powers
        // survive a JSON round-trip on the entity.
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("mythic_companion"),
            RulesetRef {
                id: Id::new("test"),
                version: "1".into(),
            },
        );
        entity.might = Some(MightScore {
            realm: Realm::Infernal,
            score: 5,
        });
        entity.powers = vec![
            SupernaturalPower {
                name: "Curse of Misfortune".into(),
                level: 20,
            },
            SupernaturalPower {
                name: "Coagulation".into(),
                level: 10,
            },
        ];
        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
        assert!(json.contains("\"realm\":\"infernal\""));
    }

    #[test]
    fn might_effects_round_trip() {
        // The two Might/power-budget Effect variants survive a `type`-tagged
        // JSON round-trip. Guards the shape virtues_flaws.json wires against.
        let effects = vec![
            Effect::MightGrant {
                realm: Realm::Infernal,
                score: 5,
            },
            Effect::PowerLevels { amount: 30 },
        ];
        let json = serde_json::to_string(&effects).unwrap();
        let back: Vec<Effect> = serde_json::from_str(&json).unwrap();
        assert_eq!(effects, back);
        assert!(json.contains("\"type\":\"might_grant\""));
        assert!(json.contains("\"type\":\"power_levels\""));
    }

    #[test]
    fn in_play_effects_roundtrip() {
        // Every M5/5b in-play Effect variant survives a JSON round-trip with its
        // `type`-tagged serde form (and, for scalar-enum payloads, the snake_case
        // scalar). Guards the shape the shipped virtues_flaws.json wires against.
        let effects = vec![
            Effect::MagicalFocus {
                param: "focus".into(),
                major: true,
            },
            Effect::CastingTotalMod {
                amount: 3,
                scope: CastingScope::FormulaicRitual,
            },
            Effect::LabTotalMod { amount: 3 },
            Effect::DeficientArt {
                param: "technique".into(),
            },
            Effect::MagicTotalHalving {
                total: HalvableTotal::Penetration,
            },
            Effect::SoakMod { amount: 3 },
            Effect::CombatMod {
                amount: -2,
                target: CombatStat::Defense,
            },
            Effect::HealthMod {
                track: HealthTrack::WoundPenalty,
                amount: 1,
            },
            Effect::MagicResistanceMod {
                kind: MagicResistanceEffect::NoFormBonus,
            },
            Effect::AgingMod {
                kind: AgingEffect::AgingRoll,
                amount: -1,
            },
            Effect::AdvancementMod {
                source: AdvancementSource::Taught,
                amount: 5,
            },
            Effect::SpecialCastingMod {
                kind: SpecialCasting::Diedne,
                param: None,
            },
            Effect::SpecialCastingMod {
                kind: SpecialCasting::DeftForm,
                param: Some("form".into()),
            },
            Effect::AbilityRollMod {
                param: "subject".into(),
                amount: 3,
            },
            Effect::ElementalMagic {
                forms: std::collections::BTreeSet::from([
                    Id::new("art.aquam"),
                    Id::new("art.auram"),
                    Id::new("art.ignem"),
                    Id::new("art.terram"),
                ]),
            },
            Effect::MasterpieceItem,
        ];
        let json = serde_json::to_string(&effects).unwrap();
        let back: Vec<Effect> = serde_json::from_str(&json).unwrap();
        assert_eq!(effects, back);
        // The serde tag is the snake_case variant name.
        assert!(json.contains("\"type\":\"magical_focus\""));
        assert!(json.contains("\"scope\":\"formulaic_ritual\""));
        assert!(json.contains("\"total\":\"penetration\""));
        assert!(json.contains("\"type\":\"elemental_magic\""));
        assert!(json.contains("\"type\":\"masterpiece_item\""));
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
          "categories": ["hermetic"],
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
        assert_eq!(item.categories, vec!["hermetic"]);
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
    fn point_item_keeps_every_descriptor_category_in_source_order() {
        // Suppressed Gift's descriptor reads "Major, Hermetic, Story" — two
        // equally real categories, kept in the descriptor's own order.
        // Source: Ars Magica - Definitive Edition (Core Rules).md:6803-6804.
        let json = r#"{
          "id": "flaw.suppressed_gift",
          "kind": "flaw",
          "classification": "narrative",
          "magnitude": "major",
          "categories": ["hermetic", "story"],
          "entity_kinds": ["character"]
        }"#;

        let item: PointItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.categories, vec!["hermetic", "story"]);
        assert_eq!(item.first_listed_category(), "hermetic");
        assert!(item.has_category("story"));
        assert!(!item.has_category("general"));

        // Order is meaningful, so a round trip must not re-sort it.
        let reserialized = serde_json::to_string(&item).unwrap();
        let roundtripped: PointItem = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(roundtripped.categories, vec!["hermetic", "story"]);
    }

    #[test]
    fn point_item_rejects_the_removed_singular_category_key() {
        // An old `rules/` directory beside a new binary (the portable layout)
        // must fail loudly rather than load an item with no categories at all.
        let json = r#"{
          "id": "flaw.optimistic",
          "kind": "flaw",
          "classification": "narrative",
          "magnitude": "minor",
          "category": "personality"
        }"#;

        let err = serde_json::from_str::<PointItem>(json)
            .unwrap_err()
            .to_string();
        assert!(err.contains("flaw.optimistic"), "names the item: {err}");
        assert!(err.contains("category"), "names the removed key: {err}");
        assert!(err.contains("categories"), "names the replacement: {err}");
    }

    #[test]
    fn point_item_without_categories_names_the_offending_item() {
        let json = r#"{
          "id": "flaw.nameless",
          "kind": "flaw",
          "classification": "narrative",
          "magnitude": "minor"
        }"#;

        let err = serde_json::from_str::<PointItem>(json)
            .unwrap_err()
            .to_string();
        assert!(err.contains("flaw.nameless"), "names the item: {err}");
        assert!(err.contains("categories"), "names the field: {err}");
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
          "categories": ["general"],
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
              "categories": ["general"],
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
              "creation_phases": ["experience", "concept"]
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
        // Declared in non-canonical order on purpose: this asserts `normalize`
        // leaves the phase order alone, which a single-phase list could not show.
        assert_eq!(
            profile.creation_phases,
            vec![CreationPhase::Experience, CreationPhase::Concept]
        );
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
            "concept", "characteristics", "virtues_flaws", "experience",
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
          "creation_phases": ["concept"]
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
            life_stages: None,
            ability_funding: AbilityFunding::Pool,
            wizard_furthest_phase: Some("abilities".into()),
            art_scores: vec![ArtScore {
                art: Id::new("art.creo"),
                score: 5,
            }],
            spells: vec![SpellSelection {
                spell: Id::new("spell.pilum_of_fire"),
                level: None,
                mastery: None,
                parameter: None,
                mastery_abilities: Vec::new(),
            }],
            spell_levels_override: None,
            house: None,
            house_choices: BTreeMap::new(),
            mythic_type: None,
            mythic_choices: BTreeMap::new(),
            warping_choices: BTreeMap::new(),
            age: Some(25),
            apparent_age: None,
            personality_traits: vec![PersonalityTrait {
                name: "Brave".into(),
                value: 3,
            }],
            reputations: Vec::new(),
            aura: 0,
            devices: Vec::new(),
            familiar: None,
            talisman: None,
            longevity_ritual: None,
            living_conditions: BTreeSet::new(),
            aging_points: BTreeMap::new(),
            warping_points: 0,
            warping_effect: String::new(),
            twilight_scars: Vec::new(),
            decrepitude_effect: String::new(),
            aging_log: Vec::new(),
            name: String::new(),
            description: String::new(),
            concept: String::new(),
            gender: String::new(),
            birth_year: None,
            sigil: String::new(),
            covenant_name: String::new(),
            parens: String::new(),
            equipment: Vec::new(),
            might: None,
            powers: Vec::new(),
        };

        let json = serde_json::to_string_pretty(&entity).unwrap();
        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);

        assert!(json.contains(r#""schema_version": 16"#));
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

    /// Two Reputations that share a kind AND a score — the shape a character with
    /// both Apostate and Senior Clergy has, since each grants Ecclesiastical 4 —
    /// survive `normalize` + a save/load round-trip as two distinct rows. Only the
    /// `content` tells them apart, and `normalize` sorts on `(kind, score,
    /// content)`, so the pair must neither collapse nor swap contents. Which grant
    /// the panel attributes each row to is arbitrary (see `reputationRows` in
    /// `ui/src/lib/derive.ts`); that both rows come back intact is not.
    #[test]
    fn two_reputations_of_one_kind_and_score_roundtrip_as_distinct_rows() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.reputations = vec![
            Reputation {
                kind: ReputationType::Ecclesiastical,
                score: 4,
                content: "renounced his vows".into(),
            },
            Reputation {
                kind: ReputationType::Ecclesiastical,
                score: 4,
                content: "archdeacon of Reims".into(),
            },
        ];
        entity.normalize();
        // Sorted on `content`, the only differing field.
        assert_eq!(entity.reputations[0].content, "archdeacon of Reims");
        assert_eq!(entity.reputations[1].content, "renounced his vows");

        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(back.reputations.len(), 2);
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

    /// Every creation phase serializes to the slug the profile data uses, and
    /// `Display` agrees with serde — the phase Fluent keys (`phase-<slug>`) and
    /// the profile's `creation_phases` strings are the same vocabulary.
    #[test]
    fn creation_phase_slugs_are_the_profile_phase_strings() {
        assert_eq!(CreationPhase::ALL.len(), 12);
        for phase in CreationPhase::ALL {
            let json = serde_json::to_string(&phase).unwrap();
            assert_eq!(serde_json::from_str::<CreationPhase>(&json).unwrap(), phase);
            // The Display slug is the serde slug without the JSON quotes.
            assert_eq!(json, format!("\"{phase}\""));
        }
        assert_eq!(
            serde_json::to_string(&CreationPhase::VirtuesFlaws).unwrap(),
            r#""virtues_flaws""#
        );
        assert_eq!(
            serde_json::to_string(&CreationPhase::HouseSpecialisation).unwrap(),
            r#""house_specialisation""#
        );
    }

    /// The read-only `type` step is gone (guided-creation review #1): it asked for
    /// nothing, because the type is fixed before the wizard opens. Asserted over the
    /// slugs rather than the variant, so the check keeps meaning something once the
    /// variant no longer exists to name.
    #[test]
    fn creation_phase_all_has_no_type_phase() {
        let slugs: Vec<String> = CreationPhase::ALL.iter().map(|p| p.to_string()).collect();
        assert!(
            !slugs.iter().any(|slug| slug == "type"),
            "the read-only `type` phase was removed; found it in {slugs:?}"
        );
    }

    /// The `experience` step (review #11) carries the funding choice and the
    /// life-stage plan, and it is declared before the `abilities` step it funds.
    #[test]
    fn creation_phase_experience_serializes_as_experience() {
        assert_eq!(
            serde_json::to_string(&CreationPhase::Experience).unwrap(),
            r#""experience""#
        );
        let position = |wanted: CreationPhase| {
            CreationPhase::ALL
                .iter()
                .position(|phase| *phase == wanted)
                .unwrap_or_else(|| panic!("{wanted} is missing from CreationPhase::ALL"))
        };
        assert!(position(CreationPhase::Experience) < position(CreationPhase::Abilities));
    }

    /// A profile's phases are typed, so a phase string the engine has no phase for
    /// fails the load instead of reaching the wizard as a step it cannot render.
    #[test]
    fn profile_with_an_unknown_creation_phase_fails_to_parse() {
        let err = serde_json::from_str::<EntityTypeProfile>(
            r#"{
              "id": "companion",
              "budget": { "virtue_points": 10, "flaw_points": 10 },
              "creation_phases": ["concept", "not_a_phase"]
            }"#,
        )
        .expect_err("an unknown creation phase must not parse");
        assert!(
            err.to_string().contains("not_a_phase"),
            "the error must name the offending phase: {err}"
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
                parameter: None,
                mastery_abilities: Vec::new(),
            },
            SpellSelection {
                spell: Id::new("spell.aegis_of_the_hearth"),
                level: Some(20),
                mastery: None,
                parameter: None,
                mastery_abilities: Vec::new(),
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
            life_stages: None,
            ability_funding: AbilityFunding::Pool,
            wizard_furthest_phase: None,
            art_scores: Vec::new(),
            spells: Vec::new(),
            spell_levels_override: None,
            house: None,
            house_choices: BTreeMap::new(),
            mythic_type: None,
            mythic_choices: BTreeMap::new(),
            warping_choices: BTreeMap::new(),
            age: None,
            apparent_age: None,
            personality_traits: Vec::new(),
            reputations: Vec::new(),
            aura: 0,
            devices: Vec::new(),
            familiar: None,
            talisman: None,
            longevity_ritual: None,
            living_conditions: BTreeSet::new(),
            aging_points: BTreeMap::new(),
            warping_points: 0,
            warping_effect: String::new(),
            twilight_scars: Vec::new(),
            decrepitude_effect: String::new(),
            aging_log: Vec::new(),
            name: String::new(),
            description: String::new(),
            concept: String::new(),
            gender: String::new(),
            birth_year: None,
            sigil: String::new(),
            covenant_name: String::new(),
            parens: String::new(),
            equipment: Vec::new(),
            might: None,
            powers: Vec::new(),
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

    /// `name_unfilled` is **opt-in**: the overwhelming majority of templated names
    /// read correctly with the generic param hint ("Puissant (Ability)"), so an entry
    /// that omits the field must keep parsing exactly as before.
    #[test]
    fn i18n_entry_without_name_unfilled_still_parses() {
        let json = r#"{ "name": "Puissant {ability}" }"#;
        let entry: I18nEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.name, "Puissant {ability}");
        assert_eq!(entry.name_unfilled, None);
        // Absent stays absent on the way out, so no locale file gains a null field.
        assert_eq!(
            serde_json::to_string(&entry).unwrap(),
            r#"{"name":"Puissant {ability}"}"#
        );
    }

    /// The two entries that need it — a `{token}` plus a parenthetical literal, the
    /// one shape whose hint substitution doubles — carry an explicit unfilled form.
    #[test]
    fn i18n_entry_name_unfilled_roundtrips() {
        let json = r#"{ "name": "{language} (Dead Language)", "name_unfilled": "Dead Language" }"#;
        let entry: I18nEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.name, "{language} (Dead Language)");
        assert_eq!(entry.name_unfilled.as_deref(), Some("Dead Language"));
        let back: I18nEntry =
            serde_json::from_str(&serde_json::to_string(&entry).unwrap()).unwrap();
        assert_eq!(back, entry);
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

    /// A v8 save (predating the M5/5e magic-possessions fields) still deserializes:
    /// the additive `serde(default)` fields fill in as empty/None/0, so no migration
    /// is needed (the same precedent as `v1_save_without_new_fields_still_loads`).
    #[test]
    fn v8_save_without_magic_possessions_still_loads() {
        let v8 = r#"{
          "schema_version": 8,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "selections": [{ "ref": "virtue.the_gift" }]
        }"#;
        let entity: Entity = serde_json::from_str(v8).unwrap();
        assert_eq!(entity.schema_version, 8);
        assert_eq!(entity.aura, 0);
        assert!(entity.devices.is_empty());
        assert!(entity.familiar.is_none());
        assert!(entity.talisman.is_none());
        assert!(entity.longevity_ritual.is_none());
    }

    /// Issue 11: a save predating `spell_levels_override` deserializes with the
    /// additive `serde(default)` field filling in as `None` — no migration needed.
    #[test]
    fn save_without_spell_levels_override_loads_as_none() {
        let old = r#"{
          "schema_version": 11,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "selections": [{ "ref": "virtue.the_gift" }]
        }"#;
        let entity: Entity = serde_json::from_str(old).unwrap();
        assert_eq!(entity.spell_levels_override, None);
    }

    /// An Entity carrying every new magic-possession field round-trips through JSON
    /// unchanged, at the current schema version.
    #[test]
    fn entity_magic_possessions_roundtrip() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.aura = -3;
        entity.devices = vec![EnchantedDevice {
            name: "Ring of Warding".into(),
            level: 20,
        }];
        entity.familiar = Some(Familiar {
            name: "Corax".into(),
            cord_gold: 2,
            cord_silver: 1,
            cord_bronze: 3,
            ..Default::default()
        });
        entity.talisman = Some(Talisman {
            description: "An ash staff shod with silver".into(),
            attunements: vec![TalismanAttunement {
                description: "Attuned to fire".into(),
                bonus: 5,
            }],
            effects: Vec::new(),
        });
        entity.longevity_ritual = Some(LongevityRitual {
            source: LongevitySource::External,
            bonus: Some(4),
            focus: "An amulet of hawthorn".into(),
        });

        let json = serde_json::to_string_pretty(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
        assert!(json.contains(r#""schema_version": 16"#));
        assert!(json.contains(r#""aura": -3"#));
        assert!(json.contains(r#""source": "external""#));
    }

    /// The talisman is the magus's personal enchanted item: a shape/material
    /// identity, its shape-and-material attunements, and the effects instilled in
    /// it. Every part is optional, so a bare talisman serializes to `{}` and a
    /// fully-filled one round-trips.
    #[test]
    fn entity_talisman_roundtrip() {
        let bare = Talisman::default();
        let json = serde_json::to_string(&bare).unwrap();
        assert_eq!(json, "{}", "an untouched talisman writes no keys");
        assert_eq!(serde_json::from_str::<Talisman>(&json).unwrap(), bare);

        let filled = Talisman {
            description: "A yew wand banded with lead".into(),
            attunements: vec![TalismanAttunement {
                description: "Controlling things at a distance".into(),
                bonus: 4,
            }],
            effects: vec![TalismanEffect {
                name: "Wielding the Invisible Sling".into(),
                level: 15,
            }],
        };
        let json = serde_json::to_string(&filled).unwrap();
        assert!(json.contains("yew wand"), "{json}");
        assert!(json.contains(r#""level":15"#), "{json}");
        assert_eq!(serde_json::from_str::<Talisman>(&json).unwrap(), filled);
    }

    /// The familiar is a full statblock, not a name plus three cords: the animal,
    /// its Magic Might, the eight Characteristics, a (commonly negative) Size,
    /// Personality Traits and the powers invested in the bond all round-trip.
    #[test]
    fn entity_familiar_statblock_roundtrip() {
        let filled = Familiar {
            name: "Corax".into(),
            animal: "raven".into(),
            might: Some(MightScore {
                realm: Realm::Magic,
                score: 10,
            }),
            characteristics: BTreeMap::from([
                (Characteristic::Int, -3),
                (Characteristic::Qik, 4),
                (Characteristic::Str, -6),
            ]),
            size: -4,
            personality_traits: vec![PersonalityTrait {
                name: "Loyal (Marcus)".into(),
                value: 3,
            }],
            cord_gold: 2,
            cord_silver: 1,
            cord_bronze: 3,
            powers: vec![SupernaturalPower {
                name: "Mental communication".into(),
                level: 15,
            }],
        };
        let json = serde_json::to_string(&filled).unwrap();
        assert!(json.contains(r#""animal":"raven""#), "{json}");
        // Size is signed and written with the ASCII hyphen-minus.
        assert!(json.contains(r#""size":-4"#), "{json}");
        assert!(json.contains(r#""int":-3"#), "{json}");
        assert!(json.contains(r#""realm":"magic""#), "{json}");
        assert_eq!(serde_json::from_str::<Familiar>(&json).unwrap(), filled);
    }

    /// A familiar that carries only a name and cords — everything the pre-5.5c
    /// model could store — writes **none** of the statblock keys, so an existing
    /// save's bytes are unchanged by the expanded shape.
    #[test]
    fn cords_only_familiar_omits_every_statblock_key() {
        let cords_only = Familiar {
            name: "Corax".into(),
            cord_gold: 2,
            cord_silver: 1,
            cord_bronze: 3,
            ..Default::default()
        };
        let json = serde_json::to_string(&cords_only).unwrap();
        assert_eq!(
            json,
            r#"{"name":"Corax","cord_gold":2,"cord_silver":1,"cord_bronze":3}"#
        );
        assert_eq!(
            serde_json::from_str::<Familiar>(&json).unwrap(),
            cords_only,
            "the omitted statblock keys load back as defaults"
        );
        assert_eq!(Familiar::default(), Familiar::default());
    }

    /// A pre-5.5c save's familiar (name + cords only) still loads: every
    /// statblock field is additive `serde(default)`, so no migration is needed
    /// and `SCHEMA_VERSION` is untouched.
    #[test]
    fn save_with_cords_only_familiar_loads_statblock_as_defaults() {
        let save = r#"{
          "schema_version": 14,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "selections": [{ "ref": "virtue.the_gift" }],
          "familiar": { "name": "Corax", "cord_bronze": 3 }
        }"#;
        let entity: Entity = serde_json::from_str(save).unwrap();
        let familiar = entity.familiar.expect("familiar loads");
        assert_eq!(familiar.name, "Corax");
        assert_eq!(familiar.cord_bronze, 3);
        assert_eq!(familiar.animal, "");
        assert_eq!(familiar.might, None);
        assert_eq!(familiar.size, 0);
        assert!(familiar.characteristics.is_empty());
        assert!(familiar.personality_traits.is_empty());
        assert!(familiar.powers.is_empty());
    }

    /// `Entity::normalize()` reaches into the familiar and sorts its two nested
    /// lists — Personality Traits by name, invested powers by name — so a
    /// statblock serializes canonically.
    #[test]
    fn entity_normalize_sorts_familiar_traits_and_powers() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.familiar = Some(Familiar {
            name: "Corax".into(),
            personality_traits: vec![
                PersonalityTrait {
                    name: "Wary".into(),
                    value: 2,
                },
                PersonalityTrait {
                    name: "Loyal (Marcus)".into(),
                    value: 3,
                },
            ],
            powers: vec![
                SupernaturalPower {
                    name: "Speech".into(),
                    level: 20,
                },
                SupernaturalPower {
                    name: "Mental communication".into(),
                    level: 15,
                },
            ],
            ..Default::default()
        });
        entity.normalize();
        let familiar = entity.familiar.expect("familiar present");
        assert_eq!(familiar.personality_traits[0].name, "Loyal (Marcus)");
        assert_eq!(familiar.powers[0].name, "Mental communication");
    }

    /// `Familiar::normalize()` prunes Characteristics entered as 0. A familiar's
    /// Characteristics are display-only (`:17793`) and nothing is bought with them, so
    /// an explicit 0 and an absent entry say the same thing — and canonical
    /// serialization asks that two semantically identical familiars write identical
    /// bytes. Pruning to empty also lets `skip_serializing_if` drop the key entirely.
    #[test]
    fn entity_normalize_prunes_zero_familiar_characteristics() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.familiar = Some(Familiar {
            name: "Corax".into(),
            characteristics: BTreeMap::from([
                (Characteristic::Int, -3),
                (Characteristic::Per, 0),
                (Characteristic::Sta, 0),
            ]),
            ..Default::default()
        });
        entity.normalize();
        let familiar = entity.familiar.clone().expect("familiar present");
        assert_eq!(
            familiar.characteristics,
            BTreeMap::from([(Characteristic::Int, -3)]),
            "only the non-zero score survives"
        );

        // An all-zero map normalizes to empty, so the key is omitted entirely.
        let mut all_zero = entity.clone();
        all_zero.familiar = Some(Familiar {
            name: "Corax".into(),
            characteristics: BTreeMap::from([(Characteristic::Per, 0)]),
            ..Default::default()
        });
        all_zero.normalize();
        let json = serde_json::to_string(&all_zero).unwrap();
        assert!(!json.contains("characteristics"), "{json}");
    }

    /// `Familiar::normalize()` clamps a cord score above the rules maximum
    /// (0…+5, Ars Magica - Definitive Edition (Core Rules).md:10836) for the same reason it prunes a zero
    /// Characteristic: every consumer already routes through
    /// `derived::cord_score`, so `cord_bronze: 255` and `cord_bronze: 5` are the
    /// same statement to the engine — yet unclamped they serialize differently and
    /// the panel *displays* the raw 255 beside read-outs computed from 5, a
    /// disagreement the user cannot resolve and that saving perpetuates.
    #[test]
    fn entity_normalize_clamps_out_of_range_familiar_cords() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.familiar = Some(Familiar {
            name: "Corax".into(),
            cord_gold: 255,
            cord_silver: 6,
            cord_bronze: 5,
            ..Default::default()
        });
        let mut in_range = entity.clone();
        in_range.familiar = Some(Familiar {
            name: "Corax".into(),
            cord_gold: 5,
            cord_silver: 5,
            cord_bronze: 5,
            ..Default::default()
        });

        entity.normalize();
        let familiar = entity.familiar.as_ref().expect("familiar present");
        assert_eq!(familiar.cord_gold, 5, "255 self-heals to the +5 maximum");
        assert_eq!(familiar.cord_silver, 5, "6 self-heals to the +5 maximum");
        assert_eq!(familiar.cord_bronze, 5, "an in-range score is untouched");

        // Two familiars the engine cannot tell apart must serialize to identical
        // bytes — the canonical-serialization rule that also prunes a zero
        // Characteristic.
        in_range.normalize();
        assert_eq!(
            serde_json::to_string(&entity).unwrap(),
            serde_json::to_string(&in_range).unwrap()
        );
    }

    /// A self-made Longevity Ritual stores the *player-entered* bonus and focus
    /// just like an external one; `None` / `""` mean "not entered yet" and are
    /// omitted from the JSON, so a pre-5.5a save loads as not-entered.
    #[test]
    fn longevity_ritual_stores_entered_bonus_and_focus() {
        let unentered = LongevityRitual {
            source: LongevitySource::SelfMade,
            bonus: None,
            focus: String::new(),
        };
        let json = serde_json::to_string(&unentered).unwrap();
        assert!(json.contains(r#""source":"self_made""#), "{json}");
        assert!(!json.contains("bonus"), "{json}");
        assert!(!json.contains("focus"), "{json}");
        assert_eq!(
            serde_json::from_str::<LongevityRitual>(&json).unwrap(),
            unentered
        );

        let entered = LongevityRitual {
            source: LongevitySource::SelfMade,
            bonus: Some(7),
            focus: "A draught of quicksilver drunk at midwinter".into(),
        };
        let json = serde_json::to_string(&entered).unwrap();
        assert!(json.contains(r#""bonus":7"#), "{json}");
        assert!(json.contains("quicksilver"), "{json}");
        assert_eq!(
            serde_json::from_str::<LongevityRitual>(&json).unwrap(),
            entered
        );
    }

    /// `normalize()` sorts devices (by name) and both of the talisman's nested
    /// lists — attunements by description, instilled effects by name —
    /// deterministically.
    #[test]
    fn entity_normalize_sorts_devices_and_talismans() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.devices = vec![
            EnchantedDevice {
                name: "Wand".into(),
                level: 10,
            },
            EnchantedDevice {
                name: "Amulet".into(),
                level: 15,
            },
        ];
        entity.talisman = Some(Talisman {
            description: String::new(),
            attunements: vec![
                TalismanAttunement {
                    description: "Zephyr".into(),
                    bonus: 2,
                },
                TalismanAttunement {
                    description: "Aegis".into(),
                    bonus: 3,
                },
            ],
            effects: vec![
                TalismanEffect {
                    name: "Wizard's Sidestep".into(),
                    level: 15,
                },
                TalismanEffect {
                    name: "Lamp Without Flame".into(),
                    level: 10,
                },
            ],
        });
        entity.normalize();
        assert_eq!(entity.devices[0].name, "Amulet");
        let talisman = entity.talisman.expect("talisman present");
        assert_eq!(talisman.attunements[0].description, "Aegis");
        assert_eq!(talisman.effects[0].name, "Lamp Without Flame");
    }

    /// The M5/5g fields (aging points, warping points, twilight scars,
    /// identity/flavor) round-trip through JSON unchanged at the current version.
    #[test]
    fn entity_aged_and_identity_fields_roundtrip() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.aging_points.insert(Characteristic::Str, 7);
        entity.aging_points.insert(Characteristic::Qik, 1);
        entity.warping_points = 15;
        entity.twilight_scars = vec![TwilightScar {
            description: "Eyes glow faintly in the dark".into(),
        }];
        entity.name = "Marcus".into();
        entity.description = "Knight of the Teutonic Order, Crusader".into();
        entity.concept =
            "A grim knight who turned to the Order of Hermes\nafter the crusade.".into();
        entity.gender = "male".into();
        entity.birth_year = Some(1194);
        entity.sigil = "the smell of ozone".into();
        entity.covenant_name = "Durenmar".into();
        entity.parens = "Bonisagus of Durenmar".into();

        let json = serde_json::to_string_pretty(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
        assert!(json.contains(r#""schema_version": 16"#));
        assert!(json.contains(r#""warping_points": 15"#));
        assert!(json.contains(r#""name": "Marcus""#));
        assert!(json.contains(r#""description": "Knight of the Teutonic Order, Crusader""#));
        assert!(json.contains(r#""birth_year": 1194"#));

        // A save lacking the new free-text fields still loads (additive, defaulted).
        let without = r#"{
          "schema_version": 10,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "name": "Marcus"
        }"#;
        let loaded: Entity = serde_json::from_str(without).unwrap();
        assert_eq!(loaded.description, "");
        assert_eq!(loaded.concept, "");
    }

    /// A legacy save carrying `aging_reductions` migrates: the completed drops are
    /// folded into `aging_points` as the minimal total reproducing them, the field
    /// is dropped, the schema is bumped, and the migration is reported.
    #[test]
    fn legacy_aging_reductions_migrate_into_aging_points() {
        // Com +2 with one completed aging drop under the old manual model.
        let old = r#"{
          "schema_version": 9,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "characteristics": { "com": 2 },
          "aging_reductions": { "com": 1 }
        }"#;
        let loaded = load_entity_migrating(old).unwrap();
        assert_eq!(
            loaded.migrated_aging_characteristics,
            vec![Characteristic::Com]
        );
        // Minimal points reproducing one drop on a +2 score = |2| + 1 = 3.
        assert_eq!(
            loaded
                .entity
                .aging_points
                .get(&Characteristic::Com)
                .copied(),
            Some(3)
        );
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
        // The bought score is untouched (point-buy stays valid).
        assert_eq!(
            loaded
                .entity
                .characteristics
                .get(&Characteristic::Com)
                .copied(),
            Some(2)
        );
        // Re-serializing carries no `aging_reductions` field.
        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("aging_reductions"));
    }

    /// A current save without `aging_reductions` migrates to a no-op report.
    #[test]
    fn current_save_migrates_without_aging_changes() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.aging_points.insert(Characteristic::Sta, 4);
        let json = serde_json::to_string(&entity).unwrap();
        let loaded = load_entity_migrating(&json).unwrap();
        assert!(loaded.migrated_aging_characteristics.is_empty());
        assert_eq!(
            loaded
                .entity
                .aging_points
                .get(&Characteristic::Sta)
                .copied(),
            Some(4)
        );
    }

    /// A legacy save carrying the flat `talisman_attunements` list migrates into
    /// `Entity.talisman`: the attunements are carried over verbatim (nothing is
    /// inferred, unlike the aging fold), the legacy key is dropped, and the schema
    /// is bumped to 14. A list that cannot deserialize instead fails the load — see
    /// `legacy_talisman_attunements_that_cannot_deserialize_fail_the_load`.
    #[test]
    fn legacy_talisman_attunements_migrate_into_talisman() {
        let old = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": [
            { "description": "Projecting bolts and missiles", "bonus": 3 },
            { "description": "Controlling things at a distance", "bonus": 4 }
          ]
        }"#;
        let loaded = load_entity_migrating(old).unwrap();
        let talisman = loaded
            .entity
            .talisman
            .as_ref()
            .expect("legacy attunements become a talisman");
        assert_eq!(talisman.attunements.len(), 2);
        assert_eq!(talisman.attunements[0].bonus, 3);
        // Only the attunements are known; the item's identity and instilled
        // effects were never stored, so they stay empty rather than invented.
        assert_eq!(talisman.description, "");
        assert!(talisman.effects.is_empty());
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
        // Re-serializing carries no legacy key.
        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("talisman_attunements"), "{json}");
    }

    /// An empty legacy list is not a talisman: no phantom item is created. The
    /// schema is still bumped, because the key's presence proves the save was
    /// written in the old shape (the `aging_reductions` precedent). App-written
    /// saves can never carry an empty list (`skip_serializing_if`), so this only
    /// covers a hand-edited file.
    #[test]
    fn legacy_empty_talisman_attunements_migrate_to_no_talisman() {
        let old = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": []
        }"#;
        let loaded = load_entity_migrating(old).unwrap();
        assert!(loaded.entity.talisman.is_none(), "no phantom talisman");
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// A hand-edited save carrying BOTH shapes, where the new `talisman` already has
    /// `attunements`: the new one wins and the legacy list is dropped unmerged.
    /// Merging would silently duplicate attunements the player already moved across by
    /// hand — which is why populated `attunements`, and only that, is what the early
    /// return gates on (see
    /// `legacy_attunements_fold_into_a_talisman_that_has_only_a_description`).
    ///
    /// The legacy row here is deliberately one that **cannot** deserialize (`bonus`
    /// far outside `i8`), which pins the *order of operations* too: the new shape's
    /// presence is decided before the legacy value is parsed, so its shape genuinely
    /// cannot matter. Parsing first would make this save fail to load — a regression
    /// from a load that succeeds today — and this `unwrap` would catch it.
    #[test]
    fn hand_edited_save_with_both_talisman_shapes_keeps_the_new_one() {
        let both = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman": {
            "description": "An ash staff",
            "attunements": [{ "description": "Warding", "bonus": 5 }]
          },
          "talisman_attunements": [{ "description": "Stale", "bonus": 3000 }]
        }"#;
        let loaded = load_entity_migrating(both).unwrap();
        let talisman = loaded.entity.talisman.as_ref().expect("talisman kept");
        assert_eq!(talisman.description, "An ash staff");
        assert_eq!(talisman.attunements.len(), 1, "legacy row not merged in");
        assert_eq!(talisman.attunements[0].description, "Warding");
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// A hand-edited save carrying an **empty** `"talisman": {}` beside a legacy
    /// list must still fold the attunements in. Every `Talisman` field is
    /// `skip_serializing_if`, so an app-written talisman the user never filled in
    /// serializes as exactly `{}` — the presence of the key therefore proves
    /// nothing, and treating it as "the new shape wins" would drop the legacy list
    /// unparsed while the load still stamps `SCHEMA_VERSION`. The next save would
    /// then rewrite the file without the legacy key: the same permanent silent loss
    /// the error path is built to prevent, reached through the both-keys path.
    ///
    /// The both-keys early return therefore gates on the new shape already carrying
    /// *attunements*, not merely existing — see
    /// `hand_edited_save_with_both_talisman_shapes_keeps_the_new_one` for the
    /// populated case, which still wins.
    #[test]
    fn legacy_attunements_fold_into_an_empty_new_talisman() {
        let both = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman": {},
          "talisman_attunements": [{ "description": "Warding", "bonus": 5 }]
        }"#;
        let loaded = load_entity_migrating(both).unwrap();
        let talisman = loaded
            .entity
            .talisman
            .as_ref()
            .expect("an empty talisman plus a legacy list keeps the attunements");
        assert_eq!(
            talisman.attunements.len(),
            1,
            "the legacy list is folded into the empty talisman, not dropped"
        );
        assert_eq!(talisman.attunements[0].description, "Warding");
        assert_eq!(talisman.attunements[0].bonus, 5);
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("talisman_attunements"), "{json}");
    }

    /// A new-shape talisman that carries data in a field the fold cannot touch — here
    /// only a `description`, no attunements — must **still** take the legacy list.
    /// `attunements` is the sole field the legacy `talisman_attunements` key can
    /// migrate into, so it is the only field whose contents can make merging
    /// duplicate anything. Gating on the whole `Talisman` differing from
    /// `Talisman::default()` would drop this list unparsed while the load still
    /// stamps `SCHEMA_VERSION`, and the next save would rewrite the file without the
    /// legacy key: permanent silent loss.
    ///
    /// The fold must also keep the talisman's own data — the description survives
    /// alongside the migrated attunement, because the list is folded *into* the
    /// existing item rather than replacing it.
    #[test]
    fn legacy_attunements_fold_into_a_talisman_that_has_only_a_description() {
        let both = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman": { "description": "An ash staff" },
          "talisman_attunements": [{ "description": "Warding", "bonus": 5 }]
        }"#;
        let loaded = load_entity_migrating(both).unwrap();
        let talisman = loaded
            .entity
            .talisman
            .as_ref()
            .expect("a described talisman plus a legacy list keeps both");
        assert_eq!(
            talisman.description, "An ash staff",
            "the talisman keeps its own data; the fold does not replace the item"
        );
        assert_eq!(
            talisman.attunements.len(),
            1,
            "the legacy list is folded in, not dropped: `attunements` was empty"
        );
        assert_eq!(talisman.attunements[0].description, "Warding");
        assert_eq!(talisman.attunements[0].bonus, 5);
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("talisman_attunements"), "{json}");
    }

    /// An empty `"talisman": {}` beside an **empty** legacy list stays no-talisman:
    /// the fold invents nothing, so nothing turns the empty item into a filled one.
    /// (An empty legacy list alone is pinned by
    /// `legacy_empty_talisman_attunements_migrate_to_no_talisman`; here the empty
    /// new-shape key is preserved as written, since the fold has nothing to add.)
    #[test]
    fn an_empty_talisman_beside_an_empty_legacy_list_gains_nothing() {
        let both = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman": {},
          "talisman_attunements": []
        }"#;
        let loaded = load_entity_migrating(both).unwrap();
        assert_eq!(
            loaded.entity.talisman,
            Some(Talisman::default()),
            "the empty talisman is kept as written; nothing is invented"
        );
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// A legacy `talisman_attunements` list that cannot deserialize fails the load
    /// **loudly**. Silently folding it into an empty list would create no talisman
    /// while the caller still stamps the current `SCHEMA_VERSION`, so the next save
    /// would rewrite the file without the legacy key and the attunements would be
    /// gone for good. A failed load leaves the file on disk untouched.
    ///
    /// Each case asserts **what** the error says and pairs the malformed fixture with
    /// a byte-identical **positive control** whose one offending value is corrected.
    /// Together they attribute the failure to the legacy row itself: a bare
    /// `is_err()` would stay green if the surrounding document had merely stopped
    /// parsing for an unrelated reason (a newly required `Entity` field, a renamed
    /// `ruleset` shape) while the fold quietly reverted to `unwrap_or_default()`.
    #[test]
    fn legacy_talisman_attunements_that_cannot_deserialize_fail_the_load() {
        // `bonus` is out of `i8` range, so `TalismanAttunement` cannot deserialize.
        let broken = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": [
            { "description": "Projecting bolts and missiles", "bonus": 3000 }
          ]
        }"#;
        let err = load_entity_migrating(broken)
            .expect_err("a malformed legacy attunement list must not load as an empty talisman")
            .to_string();
        assert!(err.contains("3000") && err.contains("i8"), "{err}");
        // Positive control: the same document with the bonus in range loads, so the
        // failure above is the legacy row and nothing else.
        let fixed = broken.replace("3000", "3");
        let loaded = load_entity_migrating(&fixed).expect("the in-range twin loads");
        assert_eq!(
            loaded
                .entity
                .talisman
                .expect("folded into a talisman")
                .attunements[0]
                .bonus,
            3
        );

        // The same for a bonus written as a JSON string, and for a null list.
        let stringly = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": [{ "description": "Warding", "bonus": "5" }]
        }"#;
        let err = load_entity_migrating(stringly)
            .expect_err("a stringly-typed bonus must not load")
            .to_string();
        assert!(err.contains("i8"), "{err}");
        let fixed = stringly.replace("\"5\"", "5");
        load_entity_migrating(&fixed).expect("the numeric twin loads");

        let nulled = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": null
        }"#;
        let err = load_entity_migrating(nulled)
            .expect_err("a null legacy list must not load")
            .to_string();
        assert!(err.contains("null") && err.contains("sequence"), "{err}");
        let fixed = nulled.replace("null", "[]");
        load_entity_migrating(&fixed).expect("the empty-list twin loads");
    }

    /// The same guarantee for the older `aging_reductions` fold: a legacy map that
    /// cannot deserialize (an out-of-`u8` drop count, an unknown Characteristic key)
    /// fails the load instead of folding in nothing and bumping the version.
    ///
    /// As above, each case checks the error text and is paired with a corrected
    /// positive control, so the failure is provably the legacy map's and not the
    /// surrounding document's.
    #[test]
    fn legacy_aging_reductions_that_cannot_deserialize_fail_the_load() {
        let out_of_range = r#"{
          "schema_version": 9,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "characteristics": { "com": 2 },
          "aging_reductions": { "com": 300 }
        }"#;
        let err = load_entity_migrating(out_of_range)
            .expect_err("an out-of-u8 drop count must not load")
            .to_string();
        assert!(err.contains("300") && err.contains("u8"), "{err}");
        let fixed = out_of_range.replace("300", "1");
        let loaded = load_entity_migrating(&fixed).expect("the in-range twin loads");
        assert_eq!(
            loaded.migrated_aging_characteristics,
            vec![Characteristic::Com]
        );

        let unknown_key = r#"{
          "schema_version": 9,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "aging_reductions": { "cun": 1 }
        }"#;
        let err = load_entity_migrating(unknown_key)
            .expect_err("an unknown Characteristic key must not load")
            .to_string();
        assert!(err.contains("cun"), "{err}");
        // Positive control: `int` is a real Characteristic, so the twin loads — the
        // Creature Format's Cunning score is deliberately not a variant (see
        // `Familiar::characteristics`).
        let fixed = unknown_key.replace("cun", "int");
        let loaded = load_entity_migrating(&fixed).expect("the known-key twin loads");
        assert_eq!(
            loaded.migrated_aging_characteristics,
            vec![Characteristic::Int]
        );
    }

    /// A slice-5e v9 save that predates the 5g fields still loads: the additive
    /// `serde(default)` fields fill in empty/None/0.
    #[test]
    fn v9_5e_save_without_aged_or_identity_fields_still_loads() {
        let v9 = r#"{
          "schema_version": 9,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "aura": -3,
          "selections": [{ "ref": "virtue.the_gift" }]
        }"#;
        let entity: Entity = serde_json::from_str(v9).unwrap();
        assert_eq!(entity.aura, -3);
        assert!(entity.aging_points.is_empty());
        assert_eq!(entity.warping_points, 0);
        assert!(entity.twilight_scars.is_empty());
        assert!(entity.name.is_empty());
        assert_eq!(entity.birth_year, None);
    }

    /// `normalize()` sorts twilight scars (by description) deterministically, and
    /// empty aged/identity fields are omitted from canonical JSON.
    #[test]
    fn entity_normalize_sorts_twilight_scars() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.twilight_scars = vec![
            TwilightScar {
                description: "Zealous devotion to symmetry".into(),
            },
            TwilightScar {
                description: "A silver streak in the hair".into(),
            },
        ];
        entity.normalize();
        assert_eq!(
            entity.twilight_scars[0].description,
            "A silver streak in the hair"
        );

        let empty = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        let json = serde_json::to_string(&empty).unwrap();
        assert!(!json.contains("aging_points"));
        assert!(!json.contains("warping_points"));
        assert!(!json.contains("twilight_scars"));
        assert!(!json.contains("\"name\""));
        assert!(!json.contains("birth_year"));
    }

    /// A hand-edited or corrupt save can carry an out-of-range `aura` (the field is
    /// a plain `i32` with no serde-level bound); `normalize()` clamps it to the
    /// rules-derived range so a value that reaches an unchecked consumer before a
    /// normalize pass (a freshly loaded save that has not yet been re-saved) cannot
    /// silently wrap the `i32` arithmetic that sums it into casting/lab totals —
    /// the same self-healing clamp `Familiar::normalize` already applies to an
    /// out-of-range cord score via `MAX_CORD_SCORE`.
    #[test]
    fn entity_normalize_clamps_aura_to_the_rules_range() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.aura = i32::MAX;
        entity.normalize();
        assert_eq!(entity.aura, 10);

        entity.aura = i32::MIN;
        entity.normalize();
        assert_eq!(entity.aura, -50);

        entity.aura = -3;
        entity.normalize();
        assert_eq!(entity.aura, -3, "a rules-legal aura is left untouched");

        // Exact boundary values pass through unchanged (Klaus, round 3, K1) —
        // the prior assertions above only prove the clamp saturates somewhere in
        // the right direction, not that the boundary is exactly
        // AURA_MODIFIER_MIN..=AURA_MODIFIER_MAX rather than off by one.
        entity.aura = AURA_MODIFIER_MAX;
        entity.normalize();
        assert_eq!(
            entity.aura, AURA_MODIFIER_MAX,
            "the max is legal, not clamped"
        );

        entity.aura = AURA_MODIFIER_MIN;
        entity.normalize();
        assert_eq!(
            entity.aura, AURA_MODIFIER_MIN,
            "the min is legal, not clamped"
        );

        // One step past each boundary clamps to the boundary, not one past it.
        entity.aura = AURA_MODIFIER_MAX + 1;
        entity.normalize();
        assert_eq!(entity.aura, AURA_MODIFIER_MAX);

        entity.aura = AURA_MODIFIER_MIN - 1;
        entity.normalize();
        assert_eq!(entity.aura, AURA_MODIFIER_MIN);
    }

    /// The character-only aging/warping annotation fields (apparent age, warping
    /// effect, decrepitude effect, aging log) round-trip; `normalize` sorts the
    /// aging log by year; empty fields are omitted from canonical JSON.
    #[test]
    fn entity_aging_warping_annotations_roundtrip_and_normalize() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.apparent_age = Some(45);
        entity.warping_effect = "A faint aura of ozone clings to him".into();
        entity.decrepitude_effect = "Stooped, slow, and hard of hearing".into();
        entity.aging_log = vec![
            AgingLogEntry {
                year: Some(1230),
                effect: "Survived a crisis".into(),
                ..AgingLogEntry::default()
            },
            AgingLogEntry {
                year: Some(1215),
                effect: "Lost a point of Stamina".into(),
                ..AgingLogEntry::default()
            },
        ];
        entity.normalize();
        // Sorted by year ascending: 1215 before 1230 (year is the first field).
        assert_eq!(entity.aging_log[0].year, Some(1215));
        assert_eq!(entity.aging_log[1].year, Some(1230));

        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
        assert!(json.contains(r#""apparent_age":45"#), "{json}");
        assert!(json.contains(r#""warping_effect":"#), "{json}");
        assert!(json.contains(r#""decrepitude_effect":"#), "{json}");
        assert!(json.contains(r#""aging_log":"#), "{json}");

        // Empty annotation fields are omitted from canonical JSON.
        let empty = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        let json = serde_json::to_string(&empty).unwrap();
        assert!(!json.contains("apparent_age"));
        assert!(!json.contains("warping_effect"));
        assert!(!json.contains("decrepitude_effect"));
        assert!(!json.contains("aging_log"));
    }

    /// A save written before the aging/warping annotation fields existed still
    /// loads: the additive `serde(default)` fields fill in empty/None. Uses the
    /// slim, schema-stamped shape the app writes.
    #[test]
    fn save_without_aging_warping_annotations_still_loads() {
        let older = r#"{
          "schema_version": 7,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "warping_points": 5,
          "selections": [{ "ref": "virtue.the_gift" }]
        }"#;
        let entity: Entity = serde_json::from_str(older).unwrap();
        assert_eq!(entity.warping_points, 5);
        assert_eq!(entity.apparent_age, None);
        assert!(entity.warping_effect.is_empty());
        assert!(entity.decrepitude_effect.is_empty());
        assert!(entity.aging_log.is_empty());
        // The Issue E additive field also defaults on an old save.
        assert!(entity.warping_choices.is_empty());
    }

    /// `living_conditions` is additive and serde-defaulted, so it bumps no schema
    /// version: an entity that records none omits the key entirely, and a save
    /// written before the field existed loads to the empty set — which is not an
    /// incomplete entry but the table's own baseline, "Average peasant 0"
    /// (Ars Magica - Definitive Edition (Core Rules).md:16587).
    #[test]
    fn a_save_without_living_conditions_round_trips_unchanged() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        assert!(entity.living_conditions.is_empty());

        let json = serde_json::to_string(&entity).unwrap();
        assert!(
            !json.contains("living_conditions"),
            "an empty set is the baseline, not a key to write: {json}"
        );
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);

        // A save written before the field existed reads as the baseline too.
        let older = r#"{
          "schema_version": 14,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "selections": []
        }"#;
        let legacy: Entity = serde_json::from_str(older).unwrap();
        assert!(legacy.living_conditions.is_empty());

        // Two cumulative rows (`:16594`) are a set, and the `BTreeSet` orders them
        // canonically without `normalize` having to.
        entity.living_conditions = BTreeSet::from([
            Id::new("living_condition.work_in_a_mine"),
            Id::new("living_condition.leper"),
        ]);
        entity.normalize();
        let json = serde_json::to_string(&entity).unwrap();
        assert!(
            json.contains(
                r#""living_conditions":["living_condition.leper","living_condition.work_in_a_mine"]"#
            ),
            "{json}"
        );
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
    }

    /// A hand-written log entry — the only shape schema 14 could hold — still
    /// deserializes, and re-serializes to exactly those two keys. Every widened
    /// field carries `skip_serializing_if`, so widening the type did not rewrite
    /// a single existing entry.
    #[test]
    fn a_legacy_aging_log_entry_round_trips_to_year_and_effect_alone() {
        let legacy = r#"{"year":1220,"effect":"Lost a point of Stamina"}"#;
        let entry: AgingLogEntry = serde_json::from_str(legacy).unwrap();
        assert_eq!(entry.year, Some(1220));
        assert_eq!(entry.effect, "Lost a point of Stamina");
        assert_eq!(serde_json::to_string(&entry).unwrap(), legacy);
    }

    /// A resolved year records everything that produced it — the die the player
    /// typed, the total it made, the conditions in force and the points awarded —
    /// so [`crate::aging`] can undo the year exactly. All of it round-trips.
    #[test]
    fn a_resolved_aging_log_entry_round_trips_with_every_recorded_field() {
        let entry = AgingLogEntry {
            year: Some(1220),
            age: Some(40),
            effect: "Lost a point of Stamina".into(),
            die: Some(11),
            total: Some(15),
            living_conditions: BTreeSet::from([Id::new("living_condition.work_in_a_mine")]),
            points: BTreeMap::from([(Characteristic::Sta, 1)]),
            apparent_age_increased: true,
            crisis: false,
            crisis_die: None,
            crisis_total: None,
            crisis_row: None,
            crisis_severity: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: AgingLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, back);
        // A `false` flag is not a key: `crisis` stays out of a save that had none.
        assert!(!json.contains("crisis"), "{json}");
    }

    /// A resolved **Crisis** records what the Crisis Table was asked and what it
    /// answered — the Simple Die the player typed (`:16621`), the CRISIS TOTAL it
    /// made, the row it landed on and that row's severity — and all four
    /// round-trip. Absent on every year that saw no Crisis, so a save that had
    /// none carries none of the keys.
    ///
    /// **No `SCHEMA_VERSION` bump**, and this test is where that decision is
    /// pinned. Every field is `serde(default, skip_serializing_if)`, so the
    /// widening is purely additive in *both* directions: a schema-15 save written
    /// before the Crisis leg existed loads unchanged, and one written after it is
    /// still a document an older reader accepts (`AgingLogEntry` declares no
    /// `deny_unknown_fields`). That is exactly the case
    /// `Entity::living_conditions` made and `AgingLogEntry::year` did not — 14 → 15
    /// was earned by `year` *becoming* optional, which an older reader rejects.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16621, :16624-16632.
    #[test]
    fn a_resolved_crisis_round_trips_and_needs_no_schema_bump() {
        // 16 is schema 16's own bump (the funding discriminator plus the wizard
        // progress slug); the Crisis widening contributed nothing to it.
        assert_eq!(
            SCHEMA_VERSION, 16,
            "a purely additive widening earns no bump"
        );

        let entry = AgingLogEntry {
            year: Some(1220),
            age: Some(40),
            effect: String::new(),
            die: Some(9),
            total: Some(13),
            living_conditions: BTreeSet::new(),
            points: BTreeMap::from([(Characteristic::Sta, 5)]),
            apparent_age_increased: true,
            crisis: true,
            crisis_die: Some(7),
            crisis_total: Some(12),
            crisis_row: Some(Id::new("crisis.bedridden_month")),
            crisis_severity: Some(CrisisSeverity::Minor),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: AgingLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, back);

        // A year with no Crisis carries none of the four keys.
        let quiet = AgingLogEntry {
            age: Some(41),
            die: Some(4),
            total: Some(8),
            ..AgingLogEntry::default()
        };
        let json = serde_json::to_string(&quiet).unwrap();
        assert!(!json.contains("crisis"), "{json}");

        // And an entry written before the Crisis leg existed still loads.
        let earlier = r#"{ "year": 1219, "age": 39, "effect": "", "die": 9,
                           "total": 13, "crisis": true }"#;
        let loaded: AgingLogEntry = serde_json::from_str(earlier).unwrap();
        assert!(loaded.crisis, "the year called for one");
        assert_eq!(loaded.crisis_die, None, "and nobody has rolled it yet");
        assert_eq!(loaded.crisis_row, None);
        assert_eq!(loaded.crisis_severity, None);
    }

    /// A character with no birth year has no calendar year to write, so `year`
    /// is omitted entirely — and because `None < Some(_)`, the derived `Ord`
    /// sorts every undated entry ahead of every dated one.
    #[test]
    fn an_undated_aging_log_entry_omits_the_year_and_sorts_before_a_dated_one() {
        let undated = AgingLogEntry {
            effect: "No apparent aging".into(),
            age: Some(36),
            ..AgingLogEntry::default()
        };
        let json = serde_json::to_string(&undated).unwrap();
        assert!(!json.contains(r#""year""#), "{json}");

        let dated = AgingLogEntry {
            year: Some(1220),
            effect: "Lost a point of Stamina".into(),
            ..AgingLogEntry::default()
        };
        let mut log = vec![dated.clone(), undated.clone()];
        log.sort();
        assert_eq!(log, vec![undated, dated]);
    }

    /// The 14 → 15 bump shipped **no migration code** of its own: a schema-14 save is
    /// already a valid schema-15 document, because serde reads a bare `year` number
    /// into `Some` and every widened field defaults. Only the *forward* direction
    /// broke — a schema-15 save may omit `year` entirely, which a schema-14 reader
    /// rejects — and that is what earned the bump. This test pins that the aging log
    /// still loads verbatim.
    ///
    /// **Schema 16 does have a fold**, and it is what now stamps this save: the file
    /// carries no `ability_funding` key, so the funding mode is inferred (no plan →
    /// `Pool`) and the version is stamped, exactly as the `aging_reductions` and
    /// talisman folds do. The aging log itself is still untouched.
    #[test]
    fn a_schema_fourteen_save_loads_without_migration() {
        assert_eq!(SCHEMA_VERSION, 16);
        let schema_14 = r#"{
          "schema_version": 14,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "apparent_age": 45,
          "aging_log": [{ "year": 1220, "effect": "Lost a point of Stamina" }]
        }"#;
        let loaded = load_entity_migrating(schema_14).unwrap();
        assert!(
            loaded.migrated_aging_characteristics.is_empty(),
            "the bump migrates nothing"
        );
        assert_eq!(loaded.entity.aging_log.len(), 1);
        assert_eq!(loaded.entity.aging_log[0].year, Some(1220));
        assert_eq!(loaded.entity.aging_log[0].effect, "Lost a point of Stamina");
        assert_eq!(loaded.entity.aging_log[0].age, None);
        assert_eq!(loaded.entity.apparent_age, Some(45));
        // Nothing in the aging log was migrated, but the missing `ability_funding` key
        // is schema 16's fold, so the version IS stamped. The mode it infers is `Pool`:
        // no `life_stages` plan means the typed pool was the authority.
        assert_eq!(loaded.entity.ability_funding, AbilityFunding::Pool);
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// The 15 → 16 bump, pinned. Two fields arrive together (see
    /// [`SCHEMA_VERSION`]'s doc): the explicit funding discriminator
    /// [`Entity::ability_funding`] and the wizard-progress slug
    /// [`Entity::wizard_furthest_phase`]. Batching them means one migration and one
    /// window instead of two.
    #[test]
    fn schema_version_is_16() {
        assert_eq!(SCHEMA_VERSION, 16);
    }

    /// A save written before the funding discriminator existed carries a
    /// `life_stages` plan and no `ability_funding` key. The mode used to *be* the
    /// plan's presence, so the fold applies that rule once at load: a plan means
    /// life-stage funding.
    #[test]
    fn entity_without_ability_funding_migrates_from_life_stages_presence() {
        let old = r#"{
          "schema_version": 15,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "age": 25,
          "life_stages": { "native_language": "German" }
        }"#;
        let loaded = load_entity_migrating(old).unwrap();
        assert_eq!(loaded.entity.ability_funding, AbilityFunding::LifeStages);
        assert!(loaded.entity.life_stages.is_some(), "the plan is kept");
        // A fold happened, so the version is stamped.
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// The other half of the same rule: no plan means the typed pool was the
    /// authority, so the mode folds to `Pool`.
    #[test]
    fn entity_without_ability_funding_or_plan_migrates_to_pool() {
        let old = r#"{
          "schema_version": 15,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "xp_pool": 240
        }"#;
        let loaded = load_entity_migrating(old).unwrap();
        assert_eq!(loaded.entity.ability_funding, AbilityFunding::Pool);
        assert_eq!(loaded.entity.xp_pool, 240, "the typed pool is kept");
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// A hand-written save in exactly the shape v0.2.x wrote (`schema_version` 14,
    /// no `ability_funding` key), holding every case the `being` fold has to decide:
    ///
    /// - `flaw.offensive_to_beings` — a typed **English** label;
    /// - `flaw.unbearable_to_beings` — a typed **German** label, with stray
    ///   whitespace and the wrong case, since a text box accepted anything;
    /// - `virtue.inoffensive_to_beings` — a value that is **already** an id, i.e. a
    ///   save someone re-picked by hand, or the second load of a migrated one;
    /// - `virtue.alluring_to_beings` — an English *being* label on an item whose
    ///   `being` parameter is **still** free text today, so the fold must not touch
    ///   it however well the value matches;
    /// - `flaw.slow_power` — paramless, because v0.2.x declared no `power`;
    /// - `virtue.folk_magic` — paramless, because v0.2.x declared no `category`.
    ///
    /// Deliberately not round-tripped through `Entity`: a save the current code
    /// wrote could never carry the legacy shapes this fixture exists to exercise.
    const V0_2_X_SAVE: &str = r#"{
      "schema_version": 14,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "magus",
      "name": "Iohannes filius Bonisagi",
      "xp_pool": 240,
      "selections": [
        { "ref": "flaw.offensive_to_beings", "params": { "being": "Mundane Humans" } },
        { "ref": "flaw.unbearable_to_beings", "params": { "being": "  dämonen " } },
        { "ref": "virtue.inoffensive_to_beings", "params": { "being": "being.faeries" } },
        { "ref": "virtue.alluring_to_beings", "params": { "being": "Faeries" } },
        { "ref": "flaw.slow_power" },
        { "ref": "virtue.folk_magic" }
      ]
    }"#;

    /// The `being` value the loaded fixture holds for `item_ref`.
    fn being_param(entity: &Entity, item_ref: &str) -> String {
        entity
            .selections
            .iter()
            .find(|selection| selection.item_ref.as_str() == item_ref)
            .and_then(|selection| selection.params.get("being"))
            .map(|value| value.as_str().to_string())
            .unwrap_or_else(|| panic!("{item_ref} must be in the fixture with a `being` param"))
    }

    /// The three items whose `being` parameter became an enumerated domain carry a
    /// typed label in every v0.2.x save. The fold maps the labels the app and the
    /// rulebook printed — in **both** shipped languages — onto the `being.*` ids,
    /// case- and whitespace-insensitively.
    #[test]
    fn a_v0_2_x_save_migrates_its_typed_being_values_in_both_languages() {
        let loaded = load_entity_migrating(V0_2_X_SAVE).unwrap();
        let entity = &loaded.entity;

        assert_eq!(
            being_param(entity, "flaw.offensive_to_beings"),
            "being.mundane_humans"
        );
        assert_eq!(
            being_param(entity, "flaw.unbearable_to_beings"),
            "being.demons",
            "a German label, mis-cased and padded, is still the same choice"
        );
    }

    /// German `:4135` — the Inoffensive entry — prints its list of classes in the
    /// **dative** ("Tieren, göttlichen Wesen, … magischen Kreaturen"), so a player
    /// copying the class straight off the page typed an inflected form. Those three
    /// forms are printed by the source, not invented, and all three name a class
    /// `virtue.inoffensive_to_beings` actually declares — which is why this test
    /// exercises them on exactly that item.
    #[test]
    fn the_german_dative_forms_the_rulebook_prints_fold_too() {
        for (typed, expected) in [
            ("Tieren", "being.animals"),
            ("göttlichen Wesen", "being.divine"),
            // Mis-cased and padded, so the dative keys go through the same
            // case/whitespace folding as every other key.
            ("  MAGISCHEN   KREATUREN ", "being.magical_creatures"),
        ] {
            let json = format!(
                r#"{{
                  "schema_version": 14,
                  "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
                  "entity_kind": "character",
                  "type_id": "magus",
                  "selections": [
                    {{ "ref": "virtue.inoffensive_to_beings", "params": {{ "being": "{typed}" }} }}
                  ]
                }}"#
            );
            let loaded = load_entity_migrating(&json).unwrap();
            assert_eq!(
                being_param(&loaded.entity, "virtue.inoffensive_to_beings"),
                expected,
                "the dative form '{typed}' printed at Core Rules (de):4135 must fold"
            );
        }
    }

    /// The property the whole fold rests on, asserted rather than eyeballed: no two
    /// frozen labels collapse onto one another under [`fold_being_label`], and no
    /// label is ever equal to a `being.*` id. The first makes the mapping
    /// unambiguous; the second makes it idempotent, since an already-migrated value
    /// can then never be a key.
    #[test]
    fn every_frozen_being_label_is_distinct_and_is_never_an_id() {
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (label, id) in LEGACY_BEING_LABELS {
            let folded = fold_being_label(label);
            if let Some(previous) = seen.insert(folded.clone(), label) {
                panic!("'{label}' and '{previous}' fold onto the same key '{folded}'");
            }
            assert!(
                !label.starts_with("being."),
                "'{label}' is an id, not a label, so the fold would stop being idempotent"
            );
            assert!(
                fold_being_label(id) != folded,
                "'{label}' folds onto its own id '{id}'"
            );
        }
        assert_eq!(
            seen.len(),
            LEGACY_BEING_LABELS.len(),
            "every frozen label must survive folding as its own key"
        );
    }

    /// A value that is already an id is not in the legacy map, so it is left exactly
    /// as written — which is also what makes the fold idempotent.
    #[test]
    fn an_already_migrated_being_value_is_left_alone() {
        let loaded = load_entity_migrating(V0_2_X_SAVE).unwrap();
        assert_eq!(
            being_param(&loaded.entity, "virtue.inoffensive_to_beings"),
            "being.faeries"
        );
    }

    /// `virtue.alluring_to_beings` and `flaw.magical_being_companion` also carry a
    /// `being` parameter, and both are **still** `text` domains — their value is a
    /// player's free description, not one of the closed lists. Folding them would
    /// silently rewrite a legal value into an id the item's domain never declared,
    /// so the fold is keyed on the three items that actually changed.
    #[test]
    fn a_being_param_that_is_still_free_text_is_never_folded() {
        let loaded = load_entity_migrating(V0_2_X_SAVE).unwrap();
        assert_eq!(
            being_param(&loaded.entity, "virtue.alluring_to_beings"),
            "Faeries",
            "a text-domain being value stays exactly as the player typed it"
        );
    }

    /// A typed value the frozen map does not recognise is left exactly as written,
    /// on every one of the three folded items. The player re-picks it once; nothing
    /// is guessed on their behalf.
    #[test]
    fn an_unrecognised_being_value_is_left_exactly_as_typed() {
        for item_ref in [
            "flaw.offensive_to_beings",
            "flaw.unbearable_to_beings",
            "virtue.inoffensive_to_beings",
        ] {
            let json = format!(
                r#"{{
                  "schema_version": 14,
                  "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
                  "entity_kind": "character",
                  "type_id": "magus",
                  "selections": [
                    {{ "ref": "{item_ref}", "params": {{ "being": "dragons" }} }}
                  ]
                }}"#
            );
            let loaded = load_entity_migrating(&json).unwrap();
            assert_eq!(being_param(&loaded.entity, item_ref), "dragons");
        }
    }

    /// The two choices v0.2.x never stored stay unfilled: `flaw.slow_power` has no
    /// `power` and `virtue.folk_magic` no `category`. There is nothing to migrate
    /// *from*, so inventing a placeholder would invent a rules choice — the visible
    /// `missing_param` is the correct outcome and the player supplies it once.
    #[test]
    fn a_choice_the_old_save_never_stored_is_not_invented() {
        let loaded = load_entity_migrating(V0_2_X_SAVE).unwrap();
        for item_ref in ["flaw.slow_power", "virtue.folk_magic"] {
            let selection = loaded
                .entity
                .selections
                .iter()
                .find(|selection| selection.item_ref.as_str() == item_ref)
                .expect("the fixture holds it");
            assert!(
                selection.params.is_empty(),
                "{item_ref} must stay unfilled, not carry a made-up value: {:?}",
                selection.params
            );
        }
    }

    /// No `schema_version` distinguishes a v0.2.x save from a current one — the
    /// `being` lists were a **ruleset** change, not a save-format one — so the fold
    /// is value-driven and must be idempotent. Loading a migrated save again changes
    /// nothing.
    #[test]
    fn migrating_a_v0_2_x_save_twice_changes_nothing() {
        let once = load_entity_migrating(V0_2_X_SAVE).unwrap().entity;
        let json = serde_json::to_string(&once).unwrap();
        let twice = load_entity_migrating(&json).unwrap().entity;
        assert_eq!(once, twice);
    }

    /// A migrated save must not churn on every open: after the fold and
    /// `normalize()`, a save→load→save cycle is byte-identical. (`normalize` sorts
    /// selections by `(ref, params)`, so a fold that changed a param value could in
    /// principle reorder rows — this pins that the reordering settles at the first
    /// save rather than repeating.)
    #[test]
    fn a_migrated_save_is_byte_stable_across_a_save_load_save_cycle() {
        let mut first = load_entity_migrating(V0_2_X_SAVE).unwrap().entity;
        first.normalize();
        let first_bytes = serde_json::to_string_pretty(&first).unwrap();

        let mut second = load_entity_migrating(&first_bytes).unwrap().entity;
        second.normalize();
        let second_bytes = serde_json::to_string_pretty(&second).unwrap();

        assert_eq!(first_bytes, second_bytes);
    }

    /// A save carrying padded parameter values, in every place a [`Selection`] can
    /// live. Written by hand for the same reason [`V0_2_X_SAVE`] is: the current code
    /// trims on load, so it can no longer *produce* the shape under test.
    ///
    /// `flaw.lesser_power` and `virtue.minor_magical_focus` are the case row 10 is
    /// about — a padded free-text descriptor is the same choice as an unpadded one.
    /// `virtue.puissant_ability` shows that a padded *id* is recovered too, and
    /// `flaw.slow_power` holds a whitespace-only value, i.e. a box the player left
    /// blank.
    const PADDED_PARAMS_SAVE: &str = r#"{
      "schema_version": 16,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "magus",
      "ability_funding": "pool",
      "selections": [
        { "ref": "flaw.lesser_power", "params": { "power": "  Wolf Shape " } },
        { "ref": "virtue.academic_concentration", "params": { "subject": "\tTheology\n" } },
        { "ref": "virtue.puissant_ability", "params": { "ability": " ability.awareness " } },
        { "ref": "flaw.slow_power", "params": { "power": "   " } }
      ],
      "house_choices": {
        "virtue.bjornaer_heartbeast": { "ref": "virtue.minor_magical_focus", "params": { "focus": " Wolf Shape " } }
      },
      "mythic_choices": {
        "virtue.mythic_blood": { "ref": "virtue.puissant_art", "params": { "art": "art.ignem " } }
      },
      "warping_choices": {
        "0": { "ref": "flaw.magical_air", "params": { "being": " Faeries" } }
      }
    }"#;

    /// The value `item_ref`'s `key` holds in the loaded fixture's bought list.
    fn param_of(entity: &Entity, item_ref: &str, key: &str) -> String {
        entity
            .selections
            .iter()
            .find(|selection| selection.item_ref.as_str() == item_ref)
            .and_then(|selection| selection.params.get(key))
            .map(|value| value.as_str().to_string())
            .unwrap_or_else(|| panic!("{item_ref} must be in the fixture with a `{key}` param"))
    }

    /// Row 10: parameter values are compared byte-for-byte everywhere that matters —
    /// the duplicate-selection key, a grant's `options.contains(pick)`, a mythic
    /// type's required-Virtue check, `normalize()`'s sort — so "Wolf Shape " and
    /// "Wolf Shape" were two different powers. Nothing trimmed a value arriving from
    /// a save file, so the fix lands **at load**: every param value in every place a
    /// selection can live is trimmed before the entity reaches anyone.
    #[test]
    fn load_trims_the_whitespace_around_every_param_value() {
        let entity = load_entity_migrating(PADDED_PARAMS_SAVE).unwrap().entity;

        assert_eq!(
            param_of(&entity, "flaw.lesser_power", "power"),
            "Wolf Shape"
        );
        assert_eq!(
            param_of(&entity, "virtue.academic_concentration", "subject"),
            "Theology",
            "tabs and newlines are whitespace too"
        );
        assert_eq!(
            param_of(&entity, "virtue.puissant_ability", "ability"),
            "ability.awareness",
            "a padded id resolves against its catalogue again"
        );
        assert_eq!(
            param_of(&entity, "flaw.slow_power", "power"),
            "",
            "a blank box stays blank — and is reported as `missing_param`, not filled in"
        );

        for (map, key, param, expected) in [
            (
                &entity.house_choices,
                "virtue.bjornaer_heartbeast",
                "focus",
                "Wolf Shape",
            ),
            (
                &entity.mythic_choices,
                "virtue.mythic_blood",
                "art",
                "art.ignem",
            ),
            (&entity.warping_choices, "0", "being", "Faeries"),
        ] {
            assert_eq!(
                map.get(key)
                    .and_then(|selection| selection.params.get(param))
                    .map(|value| value.as_str()),
                Some(expected),
                "a resolved pick's params are trimmed too ({key})"
            );
        }
    }

    /// The trim has no version signal either (a padded value is legal JSON at any
    /// `schema_version`), so it is value-driven like the `being` fold: a second load
    /// changes nothing, and a save→load→save cycle is byte-identical. Trimming
    /// reorders `normalize()`'s selection sort, so this pins that the reordering
    /// settles at the first save instead of churning the file on every open.
    #[test]
    fn trimming_params_at_load_is_idempotent_and_byte_stable() {
        let mut first = load_entity_migrating(PADDED_PARAMS_SAVE).unwrap().entity;
        first.normalize();
        let first_bytes = serde_json::to_string_pretty(&first).unwrap();

        let mut second = load_entity_migrating(&first_bytes).unwrap().entity;
        second.normalize();
        let second_bytes = serde_json::to_string_pretty(&second).unwrap();

        assert_eq!(first, second);
        assert_eq!(first_bytes, second_bytes);
    }

    /// `ability_funding` is written **even when it holds its default**, which departs
    /// from this file's standing omit-the-default convention (`is_zero`, `is_false`,
    /// `Option::is_none`). It has to: the migration dispatches on the key's *absence*,
    /// so omitting `Pool` would make a pool-funded character that still carries a
    /// stored plan — the shape schema 16 exists to allow — silently reload as
    /// life-stage funded. Absence is meaningful, so presence is mandatory.
    #[test]
    fn ability_funding_is_written_even_when_it_holds_its_default() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        assert_eq!(entity.ability_funding, AbilityFunding::Pool, "the default");
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains(r#""ability_funding":"pool""#), "{json}");

        // And a pool-funded character keeping a plan round-trips as pool-funded.
        entity.life_stages = Some(crate::life_stage::LifeStagePlan::default());
        let json = serde_json::to_string(&entity).unwrap();
        let back = load_entity_migrating(&json).unwrap().entity;
        assert_eq!(back.ability_funding, AbilityFunding::Pool);
        assert!(
            back.life_stages.is_some(),
            "the plan survives the round trip"
        );
    }

    /// A character that never entered the wizard writes **no** key, so the field is
    /// byte-invisible on every save the editor produces.
    #[test]
    fn wizard_furthest_phase_is_omitted_when_none() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        assert_eq!(entity.wizard_furthest_phase, None);
        let json = serde_json::to_string(&entity).unwrap();
        assert!(!json.contains("wizard_furthest_phase"), "{json}");

        entity.wizard_furthest_phase = Some("abilities".to_string());
        let json = serde_json::to_string(&entity).unwrap();
        assert!(
            json.contains(r#""wizard_furthest_phase":"abilities""#),
            "{json}"
        );
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(back.wizard_furthest_phase, Some("abilities".to_string()));
    }

    /// **The test that pins `Option<String>`.** [`CreationPhase`] is a closed
    /// `Deserialize` enum, so with `Option<CreationPhase>` an unrecognised slug would
    /// be a serde error — and a serde error fails the *whole* load, making the file
    /// unopenable. Two slugs prove it is not hypothetical: `"type"` is a phase this
    /// plan **removed**, so saves carrying it exist, and `"not_a_phase"` stands for
    /// every future rename. Both load; resolving the slug is the caller's job, done
    /// leniently against the loaded profile's `creation_phases`.
    #[test]
    fn a_save_with_an_unknown_wizard_phase_slug_still_loads() {
        for slug in ["type", "not_a_phase"] {
            let save = format!(
                r#"{{
                  "schema_version": 16,
                  "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
                  "entity_kind": "character",
                  "type_id": "magus",
                  "ability_funding": "pool",
                  "wizard_furthest_phase": "{slug}"
                }}"#
            );
            let loaded = load_entity_migrating(&save)
                .unwrap_or_else(|e| panic!("a save carrying '{slug}' must still load: {e}"));
            assert_eq!(
                loaded.entity.wizard_furthest_phase,
                Some(slug.to_string()),
                "the raw slug is kept verbatim, not resolved or dropped"
            );
        }
    }

    /// `warping_choices` round-trips canonically, bumps no schema version of its
    /// own (additive `serde(default)` field), and is omitted from JSON when empty
    /// so old saves lacking the key remain byte-compatible.
    #[test]
    fn warping_choices_roundtrip_is_canonical_and_schema_stable() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        // Empty → omitted from canonical JSON.
        let json = serde_json::to_string(&entity).unwrap();
        assert!(!json.contains("warping_choices"), "{json}");

        entity.warping_choices.insert(
            "warping.minor_flaw.0".to_string(),
            Selection::new(Id::new("flaw.clumsy")),
        );
        entity.normalize();
        let json = serde_json::to_string_pretty(&entity).unwrap();
        assert!(json.contains(r#""warping_choices""#), "{json}");
        assert!(json.contains(r#""schema_version": 16"#), "{json}");

        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
        assert_eq!(back.schema_version, SCHEMA_VERSION);
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
          "categories": ["site"],
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
          "categories": ["site"],
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
          "categories": ["general"],
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
          "categories": ["general"]
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
    fn max_total_deserializes() {
        let json = r#"{
          "id": "virtue.greater_power",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "major",
          "categories": ["general"],
          "max_total": 3
        }"#;
        let item: PointItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.max_total, 3);
    }

    #[test]
    fn max_total_defaults_to_255_and_is_omitted_when_default() {
        let json = r#"{
          "id": "virtue.keen_vision",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "minor",
          "categories": ["general"]
        }"#;
        let item: PointItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.max_total, u8::MAX);
        // The default must not appear in canonical output (zero-noise diffs).
        let out = serde_json::to_string(&item).unwrap();
        assert!(
            !out.contains("max_total"),
            "default should be skipped: {out}"
        );
    }

    #[test]
    fn max_share_of_kind_deserializes_and_is_absent_by_default() {
        let with_share = r#"{
          "id": "virtue.demonic_might",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "minor",
          "categories": ["supernatural"],
          "max_share_of_kind": { "numerator": 1, "denominator": 2 }
        }"#;
        let item: PointItem = serde_json::from_str(with_share).unwrap();
        assert_eq!(
            item.max_share_of_kind,
            Some(Share {
                numerator: 1,
                denominator: 2,
            })
        );

        let without_share = r#"{
          "id": "virtue.keen_vision",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "minor",
          "categories": ["general"]
        }"#;
        let plain: PointItem = serde_json::from_str(without_share).unwrap();
        assert_eq!(plain.max_share_of_kind, None);
        // Absent must stay absent in canonical output (zero-noise diffs): the
        // field exists on two catalogue entries, not on the 500-odd others.
        let out = serde_json::to_string(&plain).unwrap();
        assert!(
            !out.contains("max_share_of_kind"),
            "an absent share must be skipped: {out}"
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
        assert_eq!(format!("{}", ParameterDomain::Enumerated), "enumerated");
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
