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
use crate::life_stage::{LifeStageBlock, LifeStagePlan};
// The save-migration subsystem lives in `migration.rs`; `Entity::new` stamps the
// version, and the doc comments here link to it.
pub(crate) use crate::migration::SCHEMA_VERSION;

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
/// Source: ArMDE:2774 ("Major Virtues
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
/// Source: ArMDE:2774 ("Virtues cost
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
/// V/F catalogue into four disjoint classes and is the definitive input to the
/// effect-wiring slices: `creation_effect` items feed the creation-number wiring
/// (slice 5a-wire), `in_play_effect` items feed the derived-totals `Effect`
/// variant set (slice 5b / `derived.rs`), and `narrative` and `uncomputed_rule`
/// items are deliberately left with no mechanical effect.
///
/// The `narrative` / `uncomputed_rule` split answers a question the original
/// three-way scheme could not: *why* does this entry carry no effect? Two
/// answers were previously indistinguishable in the data — "the rulebook states
/// no rule" and "the rulebook states a rule this engine cannot compute" — so no
/// guard could tell a silently-dropped rule from genuine flavour. Separating
/// them is what makes the dropped-clause guard possible at all.
///
/// Required on [`PointItem`] (no serde default): a catalogue entry that omits it
/// fails to load, so "every V/F is classified" is enforced at load time, not only
/// by the data-integrity test.
///
/// Source: classification scheme documented in `crates/arm-rules/RULES.md` (M5/5a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    /// No mechanical creation number, no in-play/derived-total effect, **and no
    /// mechanical clause in the cited passage at all**: pure personality, story,
    /// or social-status flavor. Never given an invented effect.
    ///
    /// The last condition is the load-bearing one and is easy to misread. An
    /// entry whose rulebook text states a hard rule — a signed modifier, a botch-
    /// dice change, a cap — is **not** narrative even when the engine computes
    /// nothing for it; that is [`Classification::UncomputedRule`].
    Narrative,
    /// The cited passage states a real mechanical rule, but one that is
    /// **genuinely uncomputable at character-generation time**, so the engine
    /// deliberately models nothing and the displayed rules text is the rule's
    /// only carrier.
    ///
    /// "Uncomputable" here means the rule is not a property of the character
    /// sheet at all. The recurring shapes are:
    ///
    /// - **botch dice** — a table-time modification of how a stress roll is
    ///   rolled, not a number on the sheet (`flaw.clumsy`'s extra botch die);
    /// - **scene- or activity-contingent modifiers** — contingent on terrain,
    ///   time of day, who is watching, or what the character is doing, where the
    ///   engine's `Effect` vocabulary models only fixed, always-on modifiers to
    ///   totals it computes (`flaw.nocturnal`'s -1 between dawn and midday);
    /// - **GM judgement and open-ended magnitudes** — "-3 or greater", "as the
    ///   storyguide sees fit".
    ///
    /// Carrying an `Effect` is what distinguishes this from
    /// [`Classification::InPlayEffect`]: an entry the engine *does* compute
    /// something for is `in_play_effect` even if its passage also contains an
    /// uncomputable clause. `uncomputed_rule` entries carry no effects, exactly
    /// like `narrative` ones — the difference is entirely about whether the
    /// **rulebook** said something, not about whether the **engine** did.
    UncomputedRule,
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
            Classification::UncomputedRule => f.write_str("uncomputed_rule"),
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
/// { "kind": "hermetically_trained" }
/// { "kind": "order_member" }
/// { "kind": "is_companion" }
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
    AbilityMin {
        /// The required Ability's id.
        ability: Id,
        /// The minimum effective score required.
        score: u8,
    },
    /// The entity must have the referenced art at or above the given score.
    /// Evaluated against the entity's max effective Art score (bought score plus
    /// virtue bonuses such as Puissant Art); an art the entity lacks counts as 0.
    ArtMin {
        /// The required Art's id.
        art: Id,
        /// The minimum effective score required.
        score: u8,
    },
    /// The entity must be Hermetically trained. Evaluated against
    /// `is_hermetically_trained` (D56/A0): the type profile's own
    /// `hermetically_trained` flag, unioned with any selection carrying
    /// `Effect::ConfersHermeticTraining` (e.g. the Abandoned Apprentice Flaw).
    HermeticallyTrained,
    /// The entity must be a full member of the Order of Hermes. Evaluated
    /// against the type profile's `order_member` flag alone — profile-only,
    /// with no entity-level override (D56/A0).
    OrderMember,
    /// The entity must count as a companion — a narrower audience stated on
    /// the entry itself rather than duplicated across every type profile's
    /// `forbidden_traits` (D38). Evaluated against the type profile's own
    /// `is_companion` flag alone, profile-only like `HermeticallyTrained`/
    /// `OrderMember` (D56/A0) — **not** the profile's `id`, so a future
    /// companion-like profile joins this audience by setting the flag in
    /// data, with no change to any item that already carries this
    /// prerequisite. `mythic_companion` sets it too: "mythic companions are
    /// companions too" (RULES.md records the ruling).
    IsCompanion,
    /// The entity must count as a grog — the audience twin of [`Self::IsCompanion`]
    /// (D68.9): some entries name grogs as the only permitted audience
    /// ("This Flaw may only be taken by grogs", ArMDE:5747), others exclude them
    /// specifically ("Grogs may not take this Virtue", ArMDE:5139) via
    /// [`Self::Nor`] wrapping this leaf. Evaluated against the type profile's own
    /// `is_grog` flag alone, profile-only like `IsCompanion`/`HermeticallyTrained`/
    /// `OrderMember` (D56/A0) — **not** the profile's `id`, so a future grog-like
    /// profile joins this audience by setting the flag in data.
    IsGrog,
    /// The entity must hold (bought or granted) at least one item whose
    /// in-force category is this string — the category-ranging twin of
    /// [`Self::Has`], evaluated the same grant-aware way (D21). A free-form
    /// category string rather than a closed registry, matching
    /// [`PointItem::categories`] — enumerating every item of a category in a
    /// `Prereq::Any` would freeze a catalogue count into code (CLAUDE.md:
    /// "Catalogue size is data, never code").
    ///
    /// Not the right tool whenever the category itself would be vacuous,
    /// though: `flaw.rector`'s "a Social Status Virtue dictating his place
    /// within the university" (ArMDE:6671-6674) reads like a bare
    /// `HasCategory("social_status")` leaf, but every profile already
    /// requires SOME Social Status (D41), so that leaf would never exclude
    /// anything. D68.10 (`RULES.md`, "Rector/Proctor and Male Guild Sponsor —
    /// closed lists, not a bare Social Status") gave `flaw.rector` and
    /// `virtue.male_guild_sponsor` a named, book-defined [`Self::Any`] list of
    /// the specific qualifying Virtues instead — this variant stays for the
    /// cases where "any item of this category" is the actual rule.
    HasCategory(String),
    /// The entity must be at least this many years old (University Dean,
    /// ArMDE:6923-6926: "must ... be at least 40 years old"). Evaluated
    /// against [`Entity::age`]: an unset age is genuinely unknown (mirrors
    /// `House`'s own `None` → `Tri::Unknown`), never a definite failure —
    /// the same "resolves as the build progresses" reasoning
    /// [`Self::conflicts_with_house`]'s doc comment gives for `Has`/
    /// `AbilityMin`/`ArtMin`.
    ///
    /// Source: ArMDE:6923-6926 (D69, X7b-e row 42).
    AgeMin(u32),
    /// The entity must hold (bought or granted) at least one item of the
    /// given [`ItemKind`] whose in-force category is this string, at or
    /// above the given [`Magnitude`] — the magnitude-filtered twin of
    /// [`Self::HasCategory`] (Flawed Powers, ArMDE:6146-6149: "at least one
    /// Major Supernatural Virtue"). `item_kind` is required because
    /// [`PointItem::categories`] is shared free-form vocabulary across
    /// Virtues AND Flaws — `flaw.raised_from_the_dead` is itself a Major
    /// `supernatural`-category FLAW, so a kind-blind category+magnitude test
    /// would be satisfied by a Flaw the passage does not mean at all.
    /// Evaluated the same grant-aware way as `HasCategory` (D21).
    ///
    /// Source: ArMDE:6146-6149 (D69, D68.4; X7b-e owns this prerequisite,
    /// X4 keeps only the separate Hermetic-Flaw import filter).
    HasCategoryAtMagnitude {
        /// The category string to match, e.g. `"supernatural"`.
        category: String,
        /// The minimum magnitude a matching item must carry.
        magnitude: Magnitude,
        /// Which item kind (Virtue vs Flaw) a matching item must be.
        item_kind: ItemKind,
    },
    /// The entity's own type profile must carry this exact id (D38/D75) — the
    /// id-matching capability D38 originally called for and deferred, since
    /// `IsCompanion`/`IsGrog`/`OrderMember`/`HermeticallyTrained` turned out to
    /// cover every audience needed until now via a profile FLAG rather than an
    /// id. F-556 needs the id form instead: `virtue.domestic_animal` is gated
    /// on `character_type.domestic_animal`, an id no shipped profile carries,
    /// so no human character type can ever satisfy it — forward-compatible if
    /// an animal profile is ever added (against D58's present non-goal),
    /// requiring no change to the entry itself. Evaluated against the type
    /// profile's own [`EntityTypeProfile::id`] alone, profile-only like
    /// `IsCompanion`/`IsGrog` (D56/A0): unknown when the profile cannot be
    /// resolved, never a definite answer either way.
    ///
    /// Deliberately **not** referentially checked against the type-profile
    /// registry at load time (unlike [`Self::House`]): the whole point of
    /// F-556's use is to name an id that resolves to NO profile, so requiring
    /// one to exist would make the fix itself illegal to load.
    CharacterType(Id),
    /// The entity's effective score in the named Characteristic must be at
    /// least this value (D81.2) — Supernatural Beauty and Envied Beauty both
    /// require a positive Presence ("A character lacking a positive Presence
    /// score may not have this Virtue/Flaw", `ArMDE:5095, :6014`, i.e.
    /// `score: 1`), and Uncontrollable Strength requires Strength not below 0
    /// ("may not be taken if the character's Strength is below 0",
    /// `ArMDE:6909`, i.e. `score: 0`) — one variant covers both floors rather
    /// than shipping two near-identical ones. Evaluated against
    /// [`Entity::characteristics`]: a Characteristic the entity has not yet
    /// set is genuinely unknown (mirrors [`Self::AgeMin`]'s own unset-age
    /// handling), never a definite failure.
    CharacteristicMin {
        /// The required Characteristic's id (e.g. `characteristic.pre`).
        characteristic: Id,
        /// The minimum effective score required.
        score: i8,
    },
    /// The entity must hold a score of at least this value in some Ability of
    /// the named [`crate::ability::AbilityCategory`] (D81.3, Broken Vessel's
    /// "Supernatural Ability" half — `ArMDE:5755`). A free-form category
    /// string rather than the typed enum, matching [`Self::HasCategory`]'s own
    /// shape; unlike that variant, this ranges over the fixed Ability-category
    /// taxonomy (general/academic/arcane/martial/supernatural), not
    /// [`PointItem::categories`]' open vocabulary, so load-time integrity
    /// requires it to resolve via [`crate::ability::AbilityCategory`]'s own
    /// snake_case spelling. Static (like `HasCategory`), never `Unknown`: an
    /// Ability the entity does not have counts as score 0.
    AbilityCategoryScoreMin {
        /// The Ability category to match, e.g. `"supernatural"`.
        category: String,
        /// The minimum effective score required.
        score: u8,
    },
    /// The entity must hold some Hermetic Art at or above this score (D81.3,
    /// Broken Vessel's "… or Art normally improved through experience points"
    /// half — `ArMDE:5755`). Every entry in the Art registry IS a Hermetic
    /// Art — there is no other kind in this engine — so this ranges over the
    /// whole registry, the Art-side twin of [`Self::AbilityCategoryScoreMin`].
    /// Static, never `Unknown`: an Art the entity does not have counts as 0.
    AnyArtMin {
        /// The minimum effective score required.
        score: u8,
    },
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
    /// a `Has`/`AbilityMin`/`ArtMin`/`HermeticallyTrained`/`OrderMember`/
    /// `IsCompanion`/`IsGrog` prerequisite never excludes an
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
            | Prereq::HermeticallyTrained
            | Prereq::OrderMember
            | Prereq::IsCompanion
            | Prereq::IsGrog
            | Prereq::HasCategory(_)
            | Prereq::AgeMin(_)
            | Prereq::HasCategoryAtMagnitude { .. }
            | Prereq::CharacterType(_)
            | Prereq::CharacteristicMin { .. }
            | Prereq::AbilityCategoryScoreMin { .. }
            | Prereq::AnyArtMin { .. } => None,
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
    /// The parameter value is a bounded integer count, inclusive on both ends
    /// (D35: Simple Student's 1–2 finished years). Paired one-to-one with
    /// [`ParameterDomain::Number`] — see that variant's doc comment for why the
    /// domain half carries no resolution logic of its own. Serializes as
    /// `{ "number": { "min": 1, "max": 2 } }` (an externally-tagged struct
    /// variant), never as a bare string, so it can never be confused with
    /// [`Self::Ref`]'s scalar `"ref"` form.
    Number {
        /// Smallest legal value, inclusive.
        min: i32,
        /// Largest legal value, inclusive.
        max: i32,
    },
    /// The parameter value is an open-ended SET of ids (D9 part 3: "you can
    /// choose to have it affect multiple Abilities", ArMDE:5851 et al. —
    /// `docs/vf-audit/design-c0-parameter-model.md` § 8). Stored as
    /// [`crate::types::SelectionParamValue::Multi`], a `BTreeSet<Id>` rather
    /// than a `Vec<Id>`, so `{A,B}` and `{B,A}` are the same value by
    /// construction and need no separate canonicalization step before
    /// `max_per_target`'s duplicate-target key compares two selections.
    /// Serializes as the bare string `"multi_ref"`, never confused with
    /// [`Self::Ref`]'s own `"ref"` or [`Self::Number`]'s tagged-object form.
    MultiRef,
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamType::Ref => f.write_str("ref"),
            ParamType::Number { min, max } => write!(f, "number[{min}..={max}]"),
            ParamType::MultiRef => f.write_str("multi_ref"),
        }
    }
}

/// The domain a parameter value's [`Id`] must belong to.
///
/// Every domain is resolved when a selection's parameter values are validated:
/// `Item` against the point-item registry, `Ability` against the ability
/// catalogue, `Art` against the art catalogue, `Characteristic` by parsing
/// into [`crate::characteristics::Characteristic`], and `Enumerated`/`Category`
/// against the parameter definition's own [`ParameterDef::values`] list (for
/// `Category`, load-time integrity additionally requires that list to be a
/// subset of the declaring item's own `categories`). A value that does not
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
    /// Value is one of the *declaring item's own* [`PointItem::categories`] —
    /// records which single category reading of a multi-category Virtue/Flaw
    /// the player chose. Sufi is "*Minor, Social Status, Supernatural*"
    /// (ArMDE:5078) and `ArMDE:5083`
    /// makes the choice explicit: "either as a Minor Social Status Virtue or a
    /// Minor Supernatural Virtue". Structurally identical to [`Self::Enumerated`]
    /// at resolution time (`param.values.contains(value)`), but load-time
    /// integrity additionally requires every declared value to be a member of
    /// the SAME item's `categories` (see `ruleset::integrity`), and the picker
    /// labels options through the existing `category-<id>` Fluent family
    /// instead of the rules-i18n `displayName` lookup `Enumerated` uses — a
    /// bare category slug has no rules-i18n entry of its own. Every item
    /// declaring this domain must also cap `max_total` at 1: `ArMDE:5083` offers a
    /// choice between two readings of ONE item, not two items. See
    /// [`PointItem::categories_for`] — the single place every "is this item of
    /// category X" test resolves a `taken_as` selection against, so the five
    /// membership sites (permitted categories, forbidden categories, category
    /// caps, Gift detection, grant-constraint filtering) cannot silently
    /// disagree. Deliberately NOT consulted by browsing/catalogue surfaces —
    /// `Ruleset::items_by_category`, the UI's Available-item picker, and the
    /// Markdown export's Type cell all stay whole-list, since there is no
    /// selection (hence no `taken_as`) to narrow against.
    Category,
    /// Value is one of the four [`Realm`]s, as `realm.<slug>` — resolved by
    /// [`Realm::from_id`], with no catalogue and no declared `values` list,
    /// exactly as [`Self::Characteristic`] is resolved by
    /// `Characteristic::from_id`. The Realms are a closed engine taxonomy the
    /// rules define, so the enum IS the registry.
    ///
    /// Folk Magic is the first user: "The choice of (Realm) Lore also determines
    /// which supernatural realm his magic is aligned to for the purposes of aura
    /// modifiers" (ArMDE:3909), and
    /// `ArMDE:3919` lets each copy "align it to the same Realm as before or pick a
    /// different one". What is stored is the **Realm**, not the (Realm) Lore
    /// Ability: the Core Rules print no closed "(Realm) Lore" list, whereas the
    /// four Realms are closed, already modelled, and already have Fluent labels.
    ///
    /// Labels come from the `realm-<id>` Fluent family (the picker, the Markdown
    /// export's [`crate::export`] taxonomy label) — never from rules i18n, which
    /// has no entry for a bare realm slug.
    Realm,
    /// Value is free text the player types (e.g. Aptitude for (Sin), Necessary
    /// (Realm) Aura, a (Land)). It references no registry, so any value with
    /// non-whitespace content is legal — the picker shows a text input rather than a
    /// dropdown. **Enforced**, not merely documented: an empty or whitespace-only
    /// value resolves to nothing and is reported as `missing_param`, the same issue
    /// an absent key raises, since a blank text box is a choice not yet made rather
    /// than an unknown value (there is nothing to print). Values are trimmed at
    /// load ([`load_entity_migrating`](crate::load_entity_migrating)) and at every
    /// write path, so a padded
    /// descriptor is the same choice as an unpadded one. Case is the player's and
    /// is **stored** as typed; but the checks that weigh an item's copies against
    /// each other (`max_per_target`, `max_per_value`) **compare** text values
    /// case-folded with inner whitespace collapsed (D83.2,
    /// `catalogue.rs::fold_free_text`), so retyping "Fire" as "fire" is still a
    /// repeat.
    Text,
    /// Value is a bounded integer count (D35: Simple Student's 1-2 finished
    /// years). **This is the redundant half of the [`ParamType::Number`] pair.**
    /// It exists solely so [`ParameterDef::domain`] stays a required,
    /// non-optional field — exactly as every other domain requires — and it
    /// carries no resolution logic of its own: unlike every other domain, which
    /// resolves a value against some registry (a catalogue, a closed enum, the
    /// declaring item's own list), the actual min/max bound lives entirely on
    /// [`ParamType::Number`], and this variant's own matching arms below do
    /// nothing beyond naming it. A future reader must not go looking here for
    /// range-checking logic that lives on `ParamType::Number` instead — the
    /// same warning [`ParamType`]'s own doc comment states for its "today it
    /// carries no behavior" case.
    Number,
    /// Value is a spell id (e.g. `spell.pilum_of_fire`) — but resolved against
    /// the OWNING character's own learned spells (`Entity::spells`), never
    /// against the ruleset's whole spell catalogue. Corrupted Spells
    /// (ArMDE:5859-5863) states both its 30-level prerequisite and its
    /// "as many of the character's spells as you wish" scope against spells
    /// **already learned** — there is no "any spell in the rules" reading to
    /// opt out of, unlike [`Self::Item`]'s optional `require_possessed`, so
    /// this domain is inherently possession-scoped and needs no sibling flag
    /// (`docs/vf-audit/design-c0-parameter-model.md` § 8). In practice this
    /// domain only ever pairs with [`ParamType::MultiRef`] (the three
    /// Corrupted entries all let the player name a set), but nothing in the
    /// type system forces that pairing.
    Spell,
    /// Value is one of the closed 5-member [`AbilityCategory`] enum, as
    /// `ability_category.<slug>` — resolved by [`AbilityCategory::from_id`],
    /// with no catalogue and no declared `values` list, exactly as
    /// [`Self::Realm`] resolves against the closed [`Realm`] enum (X6a/e5:
    /// Ability Block's `class` parameter, ArMDE:5651-5654).
    ///
    /// Labels come from the already-shipped `ability-category-<slug>` Fluent
    /// family (the ability filter UI), never from rules i18n, which has no
    /// entry for a bare category slug.
    AbilityCategory,
}

impl ParameterDomain {
    /// Returns `true` if values in this domain resolve against the ruleset's
    /// point-item registry specifically (i.e. `Item`). Other domains resolve
    /// against their own registries (ability / art catalogue, Characteristic
    /// parsing); see `validation::validate_parameters`.
    ///
    /// `Category` was considered and is correctly `false` here: its values are
    /// a subset of the declaring item's OWN `categories`, not a reference to a
    /// *different* item, so it belongs with `Enumerated`/`Text` rather than
    /// `Item`.
    ///
    /// `Realm` likewise: its registry is the four-member [`Realm`] enum, not the
    /// point-item catalogue, so `false` is the right answer here — checked
    /// deliberately, since this is a `matches!` and a new variant answers
    /// `false` with no compiler nudge. Pinned by
    /// `ruleset::tests::domain_resolution_classification`.
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
            ParameterDomain::Category => f.write_str("category"),
            ParameterDomain::Realm => f.write_str("realm"),
            ParameterDomain::Text => f.write_str("text"),
            ParameterDomain::Number => f.write_str("number"),
            ParameterDomain::Spell => f.write_str("spell"),
            ParameterDomain::AbilityCategory => f.write_str("ability_category"),
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
    /// Groups of values of which **at most one** may be named across all copies
    /// of the declaring item. Empty for almost every parameter.
    ///
    /// Folk Magic's realm axis is the first and only user: `ArMDE:3919` grants the
    /// repeat *and* limits it — "you can align it to the same Realm as before or
    /// pick a different one, although a character cannot have access to both the
    /// Divine and Infernal Realms". The excluded pair is therefore **data**, not
    /// a pair of ids hardcoded in Rust: a supplement (or another realm-axis
    /// Virtue) declares its own groups and the engine needs no edit.
    ///
    /// Each group is a set, so `{divine, infernal}` and `{infernal, divine}` are
    /// the same declaration and canonical output is stable. Load-time integrity
    /// requires every member to resolve in the parameter's own domain and every
    /// group to name at least two values (a group of one excludes nothing).
    /// Enforced per entity by `validation::validate_exclusive_param_values`,
    /// which raises [`crate::validation::ValidationIssue::CODE_EXCLUSIVE_PARAM_VALUES`].
    ///
    /// **Not** the shape for a *within-one-copy* cross-parameter restriction:
    /// `ArMDE:3915` ("Infernal Lore cannot be used to produce this type of effect",
    /// Healing) and `ArMDE:3917` (Divine Lore, Evil Eye) constrain one copy's realm
    /// against that same copy's spell category, which excludes nothing *across*
    /// copies. See `crates/arm-rules/RULES.md`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub at_most_one_of: Vec<BTreeSet<Id>>,
    /// The largest number of the declaring item's copies that may name **one
    /// and the same value** for this parameter. Default `1` (D10: "once means
    /// once" is the model's default, not "no stated limit") — an item whose
    /// descriptor explicitly allows repeating one value declares the
    /// `u8::MAX` (255) sentinel for "no ceiling the rules state", the same
    /// sentinel convention [`PointItem::max_total`] uses.
    ///
    /// Necessary (Realm) Aura for (Ability) is the case the rules state: "A
    /// character may take this Flaw once for any particular Ability"
    /// (ArMDE:6482). The Flaw declares two parameters, and that sentence caps
    /// repeats on the `ability` one **alone** — the Realm axis stays free — so
    /// neither of the two [`PointItem`]-level caps can express it:
    /// [`PointItem::max_per_target`]'s duplicate key is `(item_ref, params)`,
    /// the *whole* tuple, so two copies naming one Ability under two Realms
    /// collide in no key; [`PointItem::max_total`] groups by `item_ref` alone
    /// and so cannot say "per Ability" at all. This is the third and narrowest
    /// axis: `(item_ref, one named key's value)`.
    ///
    /// **Why it lives on the parameter and not on the item.** A cap spelled as
    /// an item-level key→max map would repeat a key name this struct already
    /// owns, which is a typo waiting to look enforced; here the cap *is* on the
    /// key, so naming a parameter the item does not declare is unrepresentable
    /// rather than merely rejected. It also sits beside [`Self::at_most_one_of`],
    /// the other per-key constraint judged across an item's copies.
    ///
    /// Enforced by `validation::selections::validate_per_value_cap`, which
    /// raises
    /// [`crate::validation::ValidationIssue::CODE_TOO_MANY_FOR_PARAM_VALUE`].
    /// It counts every copy naming the value, except that copies sharing one
    /// whole tuple count at most `max_per_target` times: the excess of an
    /// identical repeat is `max_per_target`'s finding, so one mistake draws one
    /// finding. Free-text values (and a parameterized Ability's instance text)
    /// compare case- and whitespace-insensitively (D83.2), so `"Fire"` and
    /// `" fire "` are one value. Focus Power declares `255` on its `focus`
    /// ("may be taken more than once", ArMDE:3903). It tests
    /// for the `u8::MAX` sentinel rather than comparing against it, so a
    /// crafted save holding 256 copies of one value cannot trip a ceiling the
    /// rules never state. Load-time integrity rejects a cap of `0`, which no
    /// selection could ever satisfy.
    #[serde(
        default = "default_max_per_value",
        skip_serializing_if = "is_default_max_per_value"
    )]
    pub max_per_value: u8,
    /// Narrows an [`ParameterDomain::Item`] parameter to a category: if non-empty,
    /// the point item the value names must carry at least one of these categories.
    /// Empty (the default, and the shape of every parameter shipped today) means
    /// "any point item", exactly as before this field existed — unless
    /// [`Self::allow_ids`] (D34) also narrows the same parameter, in which case
    /// a value resolves through EITHER test.
    ///
    /// The deliberate mirror of [`crate::grant::GrantConstraint::require_categories`],
    /// which narrows an *open grant's* pick the same way and with the same
    /// non-empty-intersection semantics — one notion of "a Virtue of category X",
    /// spelled the same in both places, rather than two parallel concepts.
    ///
    /// **Which notion of category.** Membership only: the item's own
    /// [`PointItem::categories`], via [`PointItem::has_category`]. NOT
    /// [`PointItem::categories_for`], and NOT [`PointItem::index_categories`].
    /// A parameter value is a bare [`Id`] naming an *item*, not a
    /// [`Selection`] of one, so there is no `params` map to read a `taken_as`
    /// choice out of — the character need not even hold the item. That puts this
    /// check in the same family as `Ruleset::items_by_category` and the picker's
    /// browsing lists, which are whole-list for the same reason. And
    /// `index_categories` is provenance (where the book indexes an entry), never
    /// membership, so it is invisible here as it is everywhere else.
    ///
    /// Enforced by `validation::selections::param_value_resolves`, which narrows
    /// the `Item` domain by this list exactly as [`ParameterDomain::Technique`]
    /// narrows `Art` to one Art class — so a value outside the required
    /// categories raises the existing
    /// [`crate::validation::ValidationIssue::CODE_UNKNOWN_PARAM_VALUE`], not a
    /// code of its own: the value is not in this parameter's domain.
    ///
    /// Load-time integrity rejects the two authoring slips that would otherwise
    /// look enforced and not be (`ruleset::integrity`): the field on any domain
    /// but `item` (nothing would read it), and a category no point item in the
    /// catalogue carries (nothing could ever satisfy it).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub require_categories: BTreeSet<String>,
    /// A one-id whitelist, additive to [`Self::require_categories`]: a value
    /// resolves if EITHER test passes — the category narrowing, OR naming one
    /// of these ids directly. Empty (the default) leaves
    /// [`Self::require_categories`] as the sole narrowing, exactly as before
    /// this field existed.
    ///
    /// **D34** (`docs/vf-audit/decisions.md`): False Power's target is "One of
    /// the character's Supernatural Virtues" (ArMDE:6096), widened by
    /// `ArMDE:6082`'s own examples — "Faerie Blood, Diedne Magic, or even The
    /// Gift" — to two Virtues whose OWN category is not `supernatural`
    /// (Diedne Magic is `hermetic`, The Gift is `special`). A bare
    /// `require_categories: ["hermetic", "special", "supernatural"]` reads
    /// those two in by widening the whole category, at the cost of admitting
    /// every OTHER Virtue either category carries (56 Hermetic Virtues where
    /// the book names one) — invisible over-permission, not a stated rule.
    /// The whitelist closes that: `require_categories: ["supernatural"]` plus
    /// `allow_ids: ["virtue.diedne_magic", "virtue.the_gift"]` admits exactly
    /// what `ArMDE:6082` names, nothing else. Faerie Blood needs no entry — it
    /// is already `supernatural`.
    ///
    /// **What this costs, and why it is the deliberate trade.** `ArMDE:6082`'s
    /// "like" is an open list; a whitelist closes it, so a future
    /// background-defining Hermetic Virtue must be added by hand. A missing id
    /// is visible the moment someone looks for it, whereas the 55
    /// wrongly-admitted ones the bare-category reading produced were invisible
    /// and let a player build something the book never contemplated.
    ///
    /// **Not** routed through a predicate (D33's `exclude_if`, out of this
    /// slice's scope): a predicate needs a data property the catalogue does
    /// not carry ("defines the character's background") and nothing else
    /// would use it; a whitelist is a closed, hand-maintained list of exactly
    /// two ids, not an open-ended computed test. D33's own carrier
    /// (`flaw.flawed_powers`) is a different entry with a different shape and
    /// is not this field's concern.
    ///
    /// Enforced by `validation::selections::param_value_resolves`, on
    /// [`Self::require_categories`]'s own precedent: the narrowing IS the
    /// domain, so a value outside both tests raises the existing
    /// [`crate::validation::ValidationIssue::CODE_UNKNOWN_PARAM_VALUE`], not a
    /// code of its own.
    ///
    /// Load-time integrity rejects the field on any domain but `item` (the
    /// same reason it rejects a stray [`Self::require_categories`]: nothing
    /// else resolves against the point-item catalogue) and rejects a member
    /// that does not resolve to a real point item (the same "excludes/admits
    /// nothing, so it would look enforced and not be" reasoning as
    /// [`Self::require_categories`]'s own catalogue check).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub allow_ids: BTreeSet<Id>,
    /// Narrows an [`ParameterDomain::Ability`] parameter to a non-empty
    /// intersection with these [`AbilityCategory`] values — the `Ability`-domain
    /// mirror of [`Self::require_categories`] (X6a/e3: Performance Magic,
    /// "General Ability", ArMDE:4646-4648). Empty (the default) means "any
    /// category", exactly as before this field existed.
    ///
    /// Enforced by `validation::selections::param_value_resolves`, raising the
    /// existing [`crate::validation::ValidationIssue::CODE_UNKNOWN_PARAM_VALUE`]
    /// — the narrowing IS the domain, not a code of its own.
    ///
    /// Load-time integrity rejects the field on any domain but `ability`.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub require_ability_categories: BTreeSet<AbilityCategory>,
    /// Subtracts these ids from an [`ParameterDomain::Ability`] parameter's
    /// domain — the subtractive mirror of [`Self::allow_ids`] (X6a/e3: Magian
    /// Lineage (Major) excludes True Names even though it is
    /// Supernatural-category, ArMDE:4345). Empty (the default) excludes
    /// nothing.
    ///
    /// Enforced by `validation::selections::param_value_resolves`, raising the
    /// existing [`crate::validation::ValidationIssue::CODE_UNKNOWN_PARAM_VALUE`].
    ///
    /// Load-time integrity rejects the field on any domain but `ability`.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub forbid_ids: BTreeSet<Id>,
    /// The point item an [`ParameterDomain::Item`] value names must be one the
    /// entity actually **holds**. `false` (the default, and the shape of every
    /// parameter shipped before False Power) means the target need not be on
    /// the sheet — the position `ParameterDomain::Ability` targets are in, where
    /// Puissant Ability may name an Ability bought on a later step.
    ///
    /// False Power is the case the rules state: the Flaw is taken "once for each
    /// appropriate Supernatural Virtue that the character **possesses**"
    /// (ArMDE:6096), so a target the
    /// character does not hold is a Flaw attached to nothing.
    ///
    /// **Which notion of possession.** The grants-inclusive one:
    /// `validation::prereq::PrereqCtx::present_ids`, the very set
    /// [`Prereq::Has`] consults — bought selections ++ House / Mythic-type /
    /// warping-fill grants. A House-granted Supernatural Virtue is genuinely
    /// held, so it is a legal target; there is deliberately no second notion of
    /// "possessed" in the engine. Possession is by **id**: a `taken_as`
    /// selection's chosen reading is not consulted, exactly as
    /// [`Self::require_categories`] cannot consult it (a parameter value is a
    /// bare [`Id`], not a [`Selection`]) — see `crates/arm-rules/RULES.md`.
    ///
    /// **A possessed target is claimed exclusively.** The same sentence says
    /// "**once** for each appropriate Supernatural Virtue", so two selections
    /// may not name the same held Virtue. That cannot be expressed with
    /// [`PointItem::max_per_target`], whose duplicate key is `(item_ref, params)`
    /// and so is blind to one Major *and* one Minor False Power — two different
    /// ids — naming the same target. It rides on this one flag rather than a
    /// second field because it is one idea: the parameter claims a Virtue the
    /// character holds.
    ///
    /// Enforced by `validation::selections::validate_possessed_param_targets`,
    /// which raises
    /// [`crate::validation::ValidationIssue::CODE_PARAM_TARGET_NOT_POSSESSED`]
    /// and
    /// [`crate::validation::ValidationIssue::CODE_PARAM_TARGET_ALREADY_CLAIMED`]
    /// — codes of their own, not `unknown_param_value`: the value IS in the
    /// parameter's domain, it is this *character* who cannot name it.
    ///
    /// Load-time integrity rejects the flag on any domain but `item`, for the
    /// same reason it rejects a stray `require_categories`: nothing else
    /// resolves against the point-item catalogue, so the restriction would look
    /// enforced and be read by no one.
    #[serde(default, skip_serializing_if = "is_false")]
    pub require_possessed: bool,
    /// An [`ParameterDomain::Item`] value may not name a [`PointItem::tainted`]
    /// item. `false` (the default) means the flag is not consulted at all.
    ///
    /// "This Flaw cannot apply to Supernatural Virtues that are affiliated to
    /// the Infernal realm in the first place"
    /// (ArMDE:6096). Infernal
    /// affiliation is exactly what [`PointItem::tainted`] records — the
    /// descriptor's *Tainted* type tag, "associated with the Infernal realm"
    /// (`ArMDE:2998-3002`) — so no new field and no id list is needed.
    ///
    /// Enforced by `validation::selections::param_value_resolves` and reported
    /// as the existing
    /// [`crate::validation::ValidationIssue::CODE_UNKNOWN_PARAM_VALUE`], on
    /// [`Self::require_categories`]'s precedent: the narrowing IS the domain,
    /// it needs no entity, and a second code would split one idea across two
    /// messages. That is also what separates it from
    /// [`Self::require_possessed`], which genuinely cannot answer without the
    /// entity.
    ///
    /// Load-time integrity rejects the flag on any domain but `item`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub forbid_tainted: bool,
    /// A [`ParameterDomain::Text`] value must name one of the supernatural powers
    /// the entity holds — an `Entity::powers` entry with that exact name. `false`
    /// (the default) leaves free text free.
    ///
    /// The three per-power items are the case the rules state: Restricted Power,
    /// Slow Power and Variable Power each modify "one of the character's
    /// supernatural powers" (ArMDE:6689,
    /// :6761, :5205), so a name no power carries restricts nothing. This is the
    /// free-text sibling of [`Self::require_possessed`]: the same idea — the
    /// target must be on the sheet — for the one domain that names no registry.
    ///
    /// **Why not a domain of its own.** A power is not a catalogue entry; it is an
    /// anonymous instance the player types (a Greater Power's levels may be spent
    /// on several powers, `ArMDE:4021`), so there is nothing for a domain to resolve
    /// against. The value stays free text and only the *entity* can judge it.
    ///
    /// Matching is exact on the trimmed strings the engine already stores:
    /// `Entity::normalize` trims every parameter value, and case is deliberately
    /// left alone there for the same reason it is left alone here — the rules ask
    /// for no case-folding and two powers a player capitalised differently are
    /// their own business.
    ///
    /// Enforced by `validation::selections::validate_power_targets`, which raises
    /// [`crate::validation::ValidationIssue::CODE_POWER_DANGLING_TARGET`] on
    /// [`crate::CreationPhase::Review`] — the step that owns `powers`, and so the
    /// step where the fix lives.
    ///
    /// Load-time integrity rejects the flag on any domain but `text`, for the same
    /// reason it rejects a stray [`Self::require_possessed`]: no other domain holds
    /// a power's name, so the restriction would look enforced and be read by no one.
    #[serde(default, skip_serializing_if = "is_false")]
    pub require_power: bool,
    /// Narrows a [`ParameterDomain::Item`] parameter by DESCRIPTION rather
    /// than by category: a value resolves only if the named item does NOT
    /// satisfy this [`ItemPredicate`]. `None` (the default) leaves every
    /// other narrowing test as the sole gate, exactly as before this field
    /// existed.
    ///
    /// **D33/D68.4** (`docs/vf-audit/decisions.md`): `flaw.flawed_powers`
    /// imports a Major Hermetic Flaw, but "Any Flaw that is only appropriate
    /// to Hermetic Magic... cannot be taken with this Flaw" (ArMDE:6148) — a
    /// constraint on WHICH Flaw may be imported, not an incompatibility with
    /// holding one in one's own right (D33 corrects an earlier reading that
    /// would have modelled this as an exclusion instead). `exclude_if:
    /// "requires_hermetic_arts"` on the imported-Flaw parameter is the fix:
    /// [`Self::domain`] stays `item` + `require_categories: ["hermetic"]`,
    /// and this field additionally refuses any candidate for which
    /// [`ItemPredicate::RequiresHermeticArts`] holds. (D68.4 amends D33's
    /// original plan to use [`ItemPredicate::Trained`] here — every
    /// candidate is `trained: true`, so `Trained` cannot tell Deficient
    /// Technique/Unstructured Caster apart from Restriction/Necessary
    /// Condition.)
    ///
    /// **Not** the same mechanism as [`Self::allow_ids`] (D34): a whitelist is
    /// a closed, hand-maintained list of exact ids; this is an open-ended
    /// computed test over a property the catalogue does not enumerate by id.
    ///
    /// Enforced by `validation::selections::param_value_resolves`, on
    /// [`Self::forbid_tainted`]'s own precedent: the narrowing IS the domain,
    /// so a value failing this test raises the existing
    /// [`crate::validation::ValidationIssue::CODE_UNKNOWN_PARAM_VALUE`], not a
    /// code of its own.
    ///
    /// Load-time integrity rejects the field on any domain but `item`, for
    /// the same reason it rejects a stray [`Self::allow_ids`]: nothing else
    /// resolves against the point-item catalogue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_if: Option<ItemPredicate>,
    /// This parameter is only REQUIRED (raises `missing_param` when unfilled)
    /// when the OWNING selection's own [`ParamGate`] holds — the
    /// `missing_param` twin of [`Effect::CharacteristicScoreDeltaParam`]'s
    /// `gate` field (B4/Q-51). `None` (the default) means unconditionally
    /// required, exactly as before this field existed.
    ///
    /// Magical Blood's own worked example: `characteristic` is meaningless
    /// for Magic Animal/Spirit/Thing (ArMDE:4359-4372), so it is required
    /// only when `bloodline` equals `bloodline.magic_human` — a Magic Animal
    /// character must never be forced to fill a Characteristic the clause
    /// never reads.
    ///
    /// Load-time integrity validates `gate.param` with the SAME
    /// `ruleset::integrity::validate_param_gate` C1 built for
    /// `AbilityRef`/`CategoryRef`'s own gate — not a new invention.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_if: Option<ParamGate>,
    /// The exact number of DISTINCT values a [`ParamType::MultiRef`] selection
    /// must name — valid only when `param_type` is `multi_ref` (X6a/e4:
    /// Restricted Learning, "choose five Abilities", ArMDE:6685). `None` (the
    /// default) states no count. Counted against
    /// [`SelectionParamValue::Multi`]'s own `BTreeSet<Id>`, so a repeated
    /// value never inflates the count — the point of using a set at all.
    ///
    /// Enforced by `validation::selections::validate_selection_parameters`,
    /// raising the new
    /// [`crate::validation::ValidationIssue::CODE_WRONG_PARAM_COUNT`].
    ///
    /// Load-time integrity rejects the field on any `param_type` but
    /// `multi_ref`, and rejects `0` (no selection could ever satisfy it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_count: Option<u8>,
    /// Whether an absent value for this parameter is reported as
    /// [`crate::validation::ValidationIssue::CODE_MISSING_PARAM`] at all.
    /// `true` (the default) is every parameter shipped before this field
    /// existed: unconditionally required, relaxed only by
    /// [`Self::required_if`]'s CONDITIONAL exemption.
    ///
    /// `false` is the one documented exception (D80,
    /// `docs/vf-audit/decisions.md`): Fida'i/Lasiq's "cover social status"
    /// applies only while the character is away from home on a mission
    /// (ArMDE:4235, "As for a fida'i..."), a fact the engine has no way to
    /// know — so the choice is recorded WHEN MADE and never forced.
    /// `docs/vf-audit/design-x6-parameters.md` §3 already ruled out
    /// repurposing [`Self::required_if`] for severity, and a gate pointing at
    /// a key the item never declares would fail the SAME load-time check
    /// [`Self::required_if`]'s own gate does — so this is a genuinely new,
    /// minimal field, not a reuse of an existing one wearing a trick value.
    ///
    /// Unlike [`Self::required_if`], this is unconditional: it is not
    /// relaxed or tightened by any other parameter's value, and it is not
    /// itself gated. Load-time integrity rejects `required: false` combined
    /// with a `required_if` gate on the SAME parameter — the two are
    /// contradictory ways of saying "sometimes required", and declaring both
    /// would leave one of them silently ignored.
    #[serde(
        default = "default_required",
        skip_serializing_if = "is_default_required"
    )]
    pub required: bool,
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
            at_most_one_of: Vec::new(),
            max_per_value: default_max_per_value(),
            require_categories: Default::default(),
            allow_ids: Default::default(),
            require_ability_categories: Default::default(),
            forbid_ids: Default::default(),
            require_possessed: false,
            forbid_tainted: false,
            require_power: false,
            exclude_if: None,
            required_if: None,
            exact_count: None,
            required: default_required(),
        }
    }

    /// Creates an [`ParameterDomain::Enumerated`] parameter over `values`.
    pub fn enumerated(key: impl Into<String>, values: impl IntoIterator<Item = Id>) -> Self {
        Self {
            key: key.into(),
            param_type: ParamType::Ref,
            domain: ParameterDomain::Enumerated,
            values: values.into_iter().collect(),
            at_most_one_of: Vec::new(),
            max_per_value: default_max_per_value(),
            require_categories: Default::default(),
            allow_ids: Default::default(),
            require_ability_categories: Default::default(),
            forbid_ids: Default::default(),
            require_possessed: false,
            forbid_tainted: false,
            require_power: false,
            exclude_if: None,
            required_if: None,
            exact_count: None,
            required: default_required(),
        }
    }
}

/// A value bound to a literal, or read from the SAME selection's own parameter
/// at evaluation time (D14's two forms, `docs/vf-audit/design-c0-parameter-model.md`
/// § 1). `#[serde(untagged)]`, distinguished by which field is present —
/// `{ "literal": "latin" }` vs `{ "param": "medium" }` — never by a bare string,
/// so a value can never be confused with the [`AbilityRef`]/[`CategoryRef`]
/// shapes that embed it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    /// Fixed at authoring time: `ability.dead_language` + `instance:
    /// { "literal": "latin" }"` (Covenant Upbringing, Custos's spoken-Latin
    /// carve-out).
    Literal {
        /// The fixed instance value.
        literal: String,
    },
    /// Read from `Selection::params[param]` when this effect's owning
    /// selection is evaluated. `param` must be a key the SAME item declares
    /// (checked at load, see `ruleset::integrity::validate_gated_ability_refs`).
    Bound {
        /// The declaring item's own parameter key to read at evaluation time.
        param: String,
    },
}

impl ParamValue {
    /// Resolves this value against `selection`'s own parameters: a
    /// [`Self::Literal`] resolves to itself; a [`Self::Bound`] reads
    /// `selection.params[param]`, `None` if the key is absent or holds a
    /// `Multi` value (which names no single instance to read).
    pub(crate) fn resolve(&self, selection: &Selection) -> Option<String> {
        match self {
            ParamValue::Literal { literal } => Some(literal.clone()),
            ParamValue::Bound { param } => selection
                .params
                .get(param)
                .and_then(SelectionParamValue::as_single)
                .map(|id| id.as_str().to_string()),
        }
    }
}

/// Names a parameter this item declares and the value that activates a
/// gated [`AbilityRef`]/[`CategoryRef`] entry — conditional LIST MEMBERSHIP,
/// not value substitution (see
/// `docs/vf-audit/design-c0-parameter-model.md` § 3 for why: Wise One/Custos's
/// options are Ability *categories*, which have no "instance" axis, and Student
/// of (Realm)'s four Lores are four separate, non-parameterized Ability ids —
/// neither fits "one parameterized ability absorbing the whole family").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamGate {
    /// A parameter key the SAME item declares.
    pub param: String,
    /// The literal value that activates this entry.
    pub equals: Id,
}

impl ParamGate {
    /// Whether this gate is active for `selection`: its named `param` (on the
    /// SAME item) currently holds `equals`. `pub(crate)` (B4/Q-51): the two
    /// real consumer arms this gate's own field guards
    /// (`characteristic_score_bonus`, `reputation_and_caps.rs::reputation_grants`)
    /// live outside `types.rs`, unlike `AbilityRef`/`CategoryRef`'s
    /// `active_for` wrappers, which stay in this module.
    pub(crate) fn holds(&self, selection: &Selection) -> bool {
        selection
            .params
            .get(&self.param)
            .and_then(SelectionParamValue::as_single)
            == Some(&self.equals)
    }
}

/// One entry of [`PointItem::conditional_incompatible_with`] (X6a/e7): while
/// `gate` holds for the declaring item's own selection, every id in `forbids`
/// becomes incompatible with it — a per-VALUE extension of the flat
/// [`PointItem::incompatible_with`], read by
/// `validation::prereq::validate_incompatibilities`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionalIncompatibility {
    /// The condition on the declaring item's OWN parameter that activates
    /// this exclusion.
    pub gate: ParamGate,
    /// The ids forbidden alongside the declaring selection while `gate` holds.
    pub forbids: BTreeSet<Id>,
    /// The same-copy twin of [`Self::forbids`] (RC review-C item 2,
    /// ArMDE:7033): while `gate` holds for the declaring selection, an OTHER
    /// selection of the SAME item whose own value of `gate`'s own `param` key
    /// equals one of these is also forbidden. `forbids` alone cannot express
    /// this — it names OTHER items' ids, and the validator's `selected_ids`
    /// collapses every copy of one parameterized item to a single id, so a
    /// self-referencing `forbids` would wrongly catch every pair of copies,
    /// including legal ones (e.g. Sensitive to Cold + Sensitive to Heat).
    /// "Weak Sight is incompatible with Sensitive Sight" is two copies of
    /// `flaw.warped_senses`, one with `affliction: affliction.weak_sight`, the
    /// other with `affliction: affliction.sensitive_sight` — both read the
    /// SAME `affliction` parameter key `gate.param` already names, so no
    /// second parameter-key field is needed.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub forbids_same_item_values: BTreeSet<Id>,
}

/// One entry of [`PointItem::same_choice_exclusions`] (D69.6): the rulebook
/// sometimes forbids two selections from naming the SAME target rather than
/// forbidding the pair outright regardless of what either names — "You may
/// not take Student of (Realm) and Puissant Ability for the same Lore"
/// (ArMDE:5054), where BOTH sides are parameterized and only a shared target
/// conflicts. A flat [`PointItem::incompatible_with`] cannot express this: it
/// forbids the pair unconditionally, and `Prereq`/`ItemPredicate` narrow on a
/// PROPERTY of the other item, never on a specific PARAMETER VALUE matching
/// this item's own.
///
/// `other`'s own `other_param` value is compared against a target THIS item
/// resolves to: either `fixed_target` (Academic Concentration always means
/// Artes Liberales, regardless of its own unrelated `subject` parameter), or
/// — when `this_param` is set — THIS item's own parameter value, mapped
/// through `via` (Student of Realm's `realm` parameter, mapped to the Lore
/// Ability that realm trains). Load-time integrity requires exactly one of
/// `this_param`/`fixed_target` to be set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SameChoiceExclusion {
    /// The other item id this item may not share a resolved target with.
    pub other: Id,
    /// The parameter key on the OTHER item (`other`) whose value is the
    /// target compared against this item's own.
    pub other_param: String,
    /// The parameter key on THIS item whose value selects a target via
    /// [`Self::via`]. `None` when this item's target is fixed regardless of
    /// its own parameters — see [`Self::fixed_target`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub this_param: Option<String>,
    /// Maps [`Self::this_param`]'s resolved value to the target id compared
    /// against `other_param`. Empty when [`Self::this_param`] is `None`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub via: BTreeMap<Id, Id>,
    /// The fixed target id to compare against when [`Self::this_param`] is
    /// `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_target: Option<Id>,
}

/// An Ability id inside an [`Effect::AbilityAuthorization`] /
/// [`Effect::AbilityBonusGated`] list, carrying D14's two constraints.
/// Deserializes from a bare string for the common, unconstrained case, and
/// serializes back to one whenever both fields are absent — so a plain
/// `"ability.awareness"`-style entry stays byte-identical.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AbilityRef {
    /// A plain, unconstrained ability id.
    Bare(Id),
    /// A scoped reference: optionally restricted to one instance, and/or
    /// active only when a [`ParamGate`] holds.
    Scoped {
        /// The referenced ability id.
        ability: Id,
        /// D14 shape 1: restricts to ONE instance of a *parameterized* Ability
        /// (`ability.dead_language` + `instance: { "literal": "latin" }`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        instance: Option<ParamValue>,
        /// This entry is authorized/active only when the OWN selection's
        /// gate holds. Absent = unconditional.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
    },
}

impl AbilityRef {
    /// The referenced ability id, regardless of variant.
    pub(crate) fn ability(&self) -> &Id {
        match self {
            AbilityRef::Bare(id) => id,
            AbilityRef::Scoped { ability, .. } => ability,
        }
    }

    /// The instance constraint this entry declares, unresolved — `None` for a
    /// plain (any-instance) reference.
    pub(crate) fn instance(&self) -> Option<&ParamValue> {
        match self {
            AbilityRef::Bare(_) => None,
            AbilityRef::Scoped { instance, .. } => instance.as_ref(),
        }
    }

    /// The gate narrowing when this entry is active — `None` for an
    /// unconditional reference.
    pub(crate) fn gate(&self) -> Option<&ParamGate> {
        match self {
            AbilityRef::Bare(_) => None,
            AbilityRef::Scoped { gate, .. } => gate.as_ref(),
        }
    }

    /// Whether this entry counts for `selection`: unconditional, or its gate
    /// holds. F-349's fix reads on this — an entry whose gate does not hold
    /// contributes nothing, by construction (see
    /// `docs/vf-audit/design-c0-parameter-model.md` § 3).
    pub(crate) fn active_for(&self, selection: &Selection) -> bool {
        match self.gate() {
            None => true,
            Some(gate) => gate.holds(selection),
        }
    }

    /// The instance this entry restricts to, resolved against `selection` —
    /// `None` for a plain (any-instance) reference.
    pub(crate) fn resolved_instance(&self, selection: &Selection) -> Option<String> {
        self.instance().and_then(|v| v.resolve(selection))
    }
}

/// An Ability *category* inside an [`Effect::AbilityAuthorization`] list, with
/// the same gate as [`AbilityRef`] (D14 shape 2 / W2's exclusive choice,
/// category-scoped rather than id-scoped: Wise One, Custos, Templar
/// Specialist).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CategoryRef {
    /// A plain, unconstrained category.
    Bare(AbilityCategory),
    /// A category active only when a [`ParamGate`] holds.
    Scoped {
        /// The referenced category.
        category: AbilityCategory,
        /// This entry is authorized only when the OWN selection's gate holds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
    },
}

impl CategoryRef {
    /// The referenced category, regardless of variant.
    pub(crate) fn category(&self) -> AbilityCategory {
        match self {
            CategoryRef::Bare(c) => *c,
            CategoryRef::Scoped { category, .. } => *category,
        }
    }

    /// The gate narrowing when this entry is active — `None` for an
    /// unconditional reference.
    pub(crate) fn gate(&self) -> Option<&ParamGate> {
        match self {
            CategoryRef::Bare(_) => None,
            CategoryRef::Scoped { gate, .. } => gate.as_ref(),
        }
    }

    /// Whether this entry counts for `selection`: unconditional, or its gate
    /// holds.
    pub(crate) fn active_for(&self, selection: &Selection) -> bool {
        match self.gate() {
            None => true,
            Some(gate) => gate.holds(selection),
        }
    }
}

/// Which of the two shapes a [`Effect::CharacteristicScoreDeltaParam`] follows
/// (coordinator review, post-B4): the RULE this states is data, never inferred
/// from whether the effect happens to carry a [`ParamGate`]. A future gated
/// effect with Great-Characteristic-style semantics (raising ABOVE the cap)
/// must not silently inherit Magical Blood's WITHIN-cap clamp merely for being
/// gated — the two are independent axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CharacteristicDeltaCap {
    /// Great (Characteristic) +1 / Poor (Characteristic) −1's own shape
    /// (ArMDE:3987-3989/:6598-6600): the delta requires the bought score to
    /// ALREADY be at the base cap/floor
    /// (`validate_characteristic_delta_preconditions`), and the RESULT is
    /// deliberately left uncapped (Giant Blood reaches +6, `ArMDE:3977`). The
    /// default — every entry that predates this field keeps this behavior
    /// exactly, byte-identical, no `SCHEMA_VERSION` bump.
    #[default]
    AboveBase,
    /// Magical Blood's Magic Human shape (ArMDE:4367, B4/Q-51): no
    /// pre-existing-score precondition at all, but the contribution itself is
    /// clamped so `bought + contribution` never crosses the base cap/floor
    /// (`gated_characteristic_delta_contribution`,
    /// `effective/characteristic.rs`). Independent of `gate` — an UNGATED
    /// `within_base` delta clamps too, and a GATED `above_base` one (the
    /// default) keeps the old precondition.
    WithinBase,
}

/// Where a [`Effect::LabTotalMod`] amount is counted in the **in-play** (5b) Lab
/// Total grid `derived.rs` builds (D4, `docs/vf-audit/decisions.md`) — data, not
/// an id list the engine hardcodes. Independent of
/// `effective::lab_total_mod` (D1), which folds every `LabTotalMod` amount flat
/// and unconditionally regardless of this field, because D1 is a generous
/// creation-time ceiling on which spells may be chosen, not a played-out number
/// a character sheet prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LabTotalModScope {
    /// Counted flat in the ordinary in-play Lab Total grid — the
    /// character-generation-default reading (Inventive Genius, Creative Block:
    /// their qualifying condition, "not using a Lab Text or being taught", is
    /// what character generation already assumes). The default: every entry
    /// shipped before this field existed keeps this behavior exactly,
    /// byte-identical, no `SCHEMA_VERSION` bump (`Effect` lives in ruleset
    /// JSON, not in saves).
    #[default]
    InPlayGrid,
    /// Counted only within the maga's own Potent Magic field, never the
    /// ordinary Lab Total and never folded into a Magical Focus's doubled
    /// total either (Potent Magic Major/Minor — "a bonus in her field of
    /// magic ... much as in a Magical Focus", ArMDE:4740-4744). Read
    /// separately by `derived.rs::in_play_lab_total_mod_within_potent_field`,
    /// added to `within_potent_field` alone (`derived/lab.rs::lab_totals`) —
    /// independent of [`Self::InPlayGrid`]'s `within_focus` figure, since a
    /// character's Potent Magic field and Magical Focus descriptor need not
    /// be the same free text (D79, `docs/vf-audit/decisions.md`). Named
    /// `WithinFocusOnly` before D79, when this same amount was (wrongly)
    /// folded into the Magical-Focus figure instead of its own.
    ///
    /// Source: ArMDE:4740-4781.
    WithinPotentFieldOnly,
    /// Never counted in the in-play grid at all: the entry's qualifying
    /// condition can never hold at character generation (Adept Laboratory
    /// Student and Weak Scholar apply only "when working from the lab texts of
    /// others", which creation cannot be).
    ///
    /// Source: ArMDE:3368-3371 (Adept
    /// Laboratory Student), `ArMDE:7080-7083` (Weak Scholar).
    NeverAtCreation,
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
    /// The parameter-relative sibling of [`Self::CharacteristicScoreDelta`]:
    /// adds a free `amount` to the effective score of the characteristic named
    /// by the selection's `params[param]`, costing no buy points. Great
    /// (Characteristic) +1, Poor (Characteristic) −1 — both *perform the raise
    /// or the drop themselves* rather than unlocking a purchase, so the bought
    /// score stays inside the printed ±3 point-buy table. The "must already be
    /// at ±3" precondition is likewise parameter-relative and derived from the
    /// ruleset's base cap/floor by the sign of `amount`, so it is enforced in
    /// validation rather than stored here.
    ///
    /// Source: ArMDE:3987-3989 (Great,
    /// "raise any Characteristic … by one point, to no more than +5"),
    /// `ArMDE:6598-6600` (Poor, "lower one which is already −3 or lower by one
    /// point").
    CharacteristicScoreDeltaParam {
        /// Parameter key whose value names the target characteristic.
        param: String,
        /// The free effective-score delta per selection (may be negative).
        amount: i8,
        /// This grant applies only when the OWNING selection's own gate holds
        /// (Q-51/B4: Magical Blood's Magic Human clause, gated on its
        /// `bloodline` parameter). Absent for every OTHER existing carrier —
        /// additive, byte-compatible, no `SCHEMA_VERSION` bump (`Effect` lives
        /// in ruleset JSON, not in saves).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
        /// Which precondition/clamp shape this delta follows — see
        /// [`CharacteristicDeltaCap`]. Independent of `gate`: this is the
        /// field the coordinator's review insisted the rule live on, not the
        /// mere presence of a gate. Defaults to [`CharacteristicDeltaCap::AboveBase`],
        /// so every entry shipped before this field keeps its exact behavior.
        #[serde(default, skip_serializing_if = "is_default_characteristic_delta_cap")]
        cap: CharacteristicDeltaCap,
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
    /// Source: ArMDE:3372-3374.
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
    /// Source: ArMDE:3376-3378.
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
    /// Source: ArMDE:4315-4317 (Linguist).
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
    /// `abilities`, its category is in `categories`, **or** it matches one of
    /// `instances` — the three are a **union** (D48,
    /// `docs/vf-audit/decisions.md` D48), not "instances-only when non-empty".
    /// The general [`Entity::xp_pool`] still covers anything; unused restricted
    /// XP is wasted. Stacks across selections. Educated (specific ids), Warrior
    /// / Privileged Upbringing (categories), Marshal / Master Bard (instances).
    ///
    /// Source: ArMDE:3711-3713
    /// (Educated), `ArMDE:5227-5229` (Warrior), `ArMDE:4806-4808` (Privileged Upbringing).
    RestrictedAbilityXp {
        /// Points granted to this restricted pool.
        amount: u32,
        /// Eligible ability ids, any instance (Educated: Artes Liberales).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        abilities: Vec<Id>,
        /// Eligible ability categories (Warrior: Martial; Privileged: General,
        /// Academic, Martial).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        categories: Vec<AbilityCategory>,
        /// Specific ability instances this pool also funds (D48: Marshal's
        /// Profession: Marshal, Master Bard's Profession: Storyteller/Poet) —
        /// reuses [`AbilityRef`], the same literal/bound-instance mechanism D14
        /// built for [`Self::AbilityAuthorization`], rather than a second type
        /// for the same idea. Resolved against the owning selection by
        /// `effective/xp.rs::resolve_ability_refs`, the helper shared with the
        /// authorization fold so the two readings ("what may I own" vs "what
        /// may this pool fund") cannot drift apart.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        instances: Vec<AbilityRef>,
        /// D13: this grant is not additional supply — it earmarks part of the
        /// GENERAL pool the character already has. `general_pool_and_bonus`
        /// subtracts every earmark's `amount` from the general base before this
        /// pool is built as an ordinary source-fed `FlowPool`, so the character's
        /// total budget is unchanged; only which Abilities the earmarked slice may
        /// fund narrows. `false` (the default) preserves every existing carrier's
        /// current, additive meaning — Educated, Warrior, Privileged Upbringing
        /// are real grants, not earmarks, and must not lose XP by this change.
        #[serde(default, skip_serializing_if = "is_false")]
        from_normal_budget: bool,
        /// Names a `multi_ref`/`ability`-domain parameter on the SAME item
        /// whose resolved set is a FOURTH eligibility source, unioned with
        /// `abilities`/`categories`/`instances` (X6a/e5, D48 extended):
        /// Restricted Learning's five player-named Abilities
        /// (ArMDE:6685). `None` (the default) leaves the
        /// union exactly as it was before this field existed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        abilities_param: Option<String>,
    },
    /// D35's parameter-scaled sibling of [`Self::RestrictedAbilityXp`]: the
    /// granted pool's `amount` is `per_unit` times the value the selection's
    /// own `param` (a [`ParameterDomain::Number`] parameter) names, rather than
    /// a fixed total — Simple Student, "He receives 30 experience points per
    /// finished year that he can apply to Latin or Artes Liberales"
    /// (ArMDE:4960), capped at 2 finished years (60 XP) by the parameter's own
    /// `max` (`docs/vf-audit/decisions.md` D35).
    ///
    /// Kept as a **separate** variant rather than an `Option<(String, u32)>`
    /// bolted onto [`Self::RestrictedAbilityXp`], on the same precedent as
    /// [`Self::CharacteristicScoreDelta`]/[`Self::CharacteristicScoreDeltaParam`]:
    /// a fixed-amount effect and a parameter-scaled one are already a
    /// "Foo"/"FooParam" pair elsewhere in this file, and the flow-graph code
    /// that reads `RestrictedAbilityXp`
    /// (`effective/xp.rs::restricted_ability_xp_pools`) must not silently pass
    /// through an unscaled `amount` for an entry that actually needs
    /// `per_unit * bound value`.
    ///
    /// `abilities` reuses [`AbilityRef`] (not a bare `Vec<Id>`) because Simple
    /// Student's own Latin restriction needs D14's literal-instance form from
    /// day one — "he receives 30 experience points... that he can apply to
    /// Latin or Artes Liberales" funds Artes Liberales at any instance (it has
    /// none) but Dead Language only at the Latin instance specifically, never
    /// Ancient Greek. `categories` stays a bare list, unscoped, on
    /// `RestrictedAbilityXp`'s own precedent: an earmark/grant category is
    /// never gated.
    ScaledRestrictedAbilityXp {
        /// Parameter key (a [`ParameterDomain::Number`] parameter on the SAME
        /// item) whose value is the per-unit count.
        param: String,
        /// XP granted per unit named by `param` (Simple Student: 30 per
        /// finished year).
        per_unit: u32,
        /// Eligible ability ids/instances (Simple Student: Artes Liberales at
        /// any instance, Dead Language at the Latin instance only).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        abilities: Vec<AbilityRef>,
        /// Eligible ability categories, unscoped.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        categories: Vec<AbilityCategory>,
    },
    /// D40 (`docs/vf-audit/decisions.md` D40): replaces a named life-stage
    /// block's normal grant with a different total and eligibility, rather
    /// than adding to it — `flaw.feral_upbringing` (ArMDE:6110-6113) and
    /// `virtue.redcap`/`virtue.lone_redcap` (ArMDE:4842-4851, :4319-4326) each
    /// shipped their own bug (F-428/F-439) from reusing the additive
    /// [`Self::RestrictedAbilityXp`] for a passage that replaces a block
    /// instead. Consumed differently per `stage` — see
    /// `life_stage::LifeStageRules::budget`/`effective::xp::build_flow_pools`
    /// — exactly as [`LifeStageBlock`]'s three pre-existing variants already
    /// are (`childhood_native_language_pool`/`childhood_spread_pool`/
    /// `magus_later_life_pool`, three separate functions sharing one enum).
    ReplacesLifeStageXp {
        /// Which block this replaces.
        stage: LifeStageBlock,
        /// The replacement's total XP.
        amount: u32,
        /// Eligible ability ids (empty when eligibility is purely by category —
        /// Redcap/Lone Redcap name only categories).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        abilities: Vec<Id>,
        /// Eligible ability categories (empty when eligibility is purely by
        /// id — Feral Upbringing names only ids).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        categories: Vec<AbilityCategory>,
        /// Years carved out of later life in place of this block — meaningful
        /// ONLY for [`LifeStageBlock::Apprenticeship`]; `0` (skipped) for a
        /// flat block replacement (childhood), which does not touch any span.
        /// Authored per entry rather than derived from the ruleset's own magus
        /// `ApprenticeshipRules.years`, because the carrier's OWN passage
        /// states its span independently (ArMDE:4321: "your fifteen years")
        /// and a companion-shaped ruleset need not ship a magus block at all
        /// for this effect to be well-formed.
        #[serde(default, skip_serializing_if = "is_zero")]
        years: u32,
    },
    /// D56/D62/D3 (`docs/vf-audit/design-d0-xp-modes.md` § 4): the Abandoned
    /// Apprentice's truncated Hermetic training, "you have most of the skills
    /// and knowledge of a fully trained magus" (ArMDE:5641-5650) — funds the
    /// **general** pool (Arts or Abilities alike, ArMDE:2435) at
    /// `ApprenticeshipRules::truncated_xp_per_year` /
    /// `truncated_spell_levels_per_year` (16/8, D56's FIXED derivation from
    /// ArMDE:2435's 240/120 over 15 years) times `param`'s resolved value —
    /// the years of apprenticeship completed before abandonment (D62: a year
    /// count, not an age to subtract). Distinct from
    /// [`Self::ReplacesLifeStageXp`], which replaces a RESTRICTED block's
    /// total: this funds the GENERAL pool and additionally carves `param`'s
    /// years out of later life (`life_stage::extra_apprenticeship_years`),
    /// which `ReplacesLifeStageXp`'s own `years` field does independently for
    /// its own (restricted) shape — reusing `ScaledRestrictedAbilityXp` here
    /// would be wrong for the identical reason `ReplacesLifeStageXp` is not a
    /// wider `RestrictedAbilityXp`.
    ///
    /// Must resolve TOGETHER with a [`Self::ConfersHermeticTrainingIf`] on the
    /// SAME item naming the SAME `param` (F1/R3-1,
    /// `effective::hermetic_training::entity_confers_hermetic_training`) — see
    /// that variant's own doc comment for why a silent zero would otherwise
    /// result the instant the Flaw is picked, before its one parameter is
    /// answered.
    TruncatedApprenticeshipXp {
        /// Parameter key (a [`ParameterDomain::Number`] parameter on the SAME
        /// item) naming the years of apprenticeship completed before
        /// abandonment.
        param: String,
    },
    /// Adjusts the Characteristic-buy budget by `amount` (on top of
    /// [`crate::characteristics::CharacteristicRules::start_points`]). Signed:
    /// Improved Characteristics grants +3 (`ArMDE:4103-4105`), Weak Characteristics
    /// removes 3 (`ArMDE:7056-7058`); both stack, so the grants from every matching
    /// selection are summed.
    ///
    /// Source: ArMDE:4103-4105 (Improved),
    /// `ArMDE:7056-7058` (Weak).
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
    /// The parameter-relative sibling of [`Self::AbilityScoreGrant`] (F-63,
    /// `docs/vf-audit/design-c0-parameter-model.md` § 2): grants the same free
    /// bought-score floor, but restricted to the ONE ability *instance* the
    /// declaring item's own parameter names, rather than to the whole ability
    /// unconditionally. Enchanting (Ability) confers "the Ability Enchanting
    /// (Ability) 1" where "(Ability)" is a player-chosen medium (music, dance,
    /// storytelling, "even craftwork") — `instance` reads that choice via
    /// [`ParamValue::Bound`], so the floor applies to the chosen medium's
    /// instance alone, never to every instance of `ability`. Kept as a
    /// separate "Foo"/"FooParam" variant rather than an `Option<ParamValue>`
    /// field bolted onto [`Self::AbilityScoreGrant`], on the same precedent as
    /// [`Self::CharacteristicScoreDelta`]/[`Self::CharacteristicScoreDeltaParam`].
    ///
    /// Source: ArMDE:3747-3750.
    AbilityScoreGrantParam {
        /// The ability granted a free starting score.
        ability: Id,
        /// The instance this grant restricts to — `None` for a plain
        /// (single-instance) ability. Not expected to occur in practice (a
        /// plain-ability grant has no reason to use this variant over
        /// [`Self::AbilityScoreGrant`]), but the type does not forbid it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        instance: Option<ParamValue>,
        /// The free bought-score floor granted.
        amount: u8,
    },
    /// Adds `amount` levels to the magus's spell-levels budget (on top of the
    /// type profile's [`PointBudget`]-adjacent `spell_levels`). Signed: Skilled
    /// Parens grants +30, Weak Parens −30. The grants from every matching
    /// selection are summed and the total is clamped at 0.
    ///
    /// Source: ArMDE:4964-4966 (Skilled
    /// Parens), `ArMDE:7072-7074` (Weak Parens).
    SpellLevels {
        /// Spell levels added to the budget per selection (may be negative).
        amount: i16,
    },
    /// Adds `amount` experience points to the general apprenticeship XP pool
    /// (spendable on Arts *or* Abilities, on top of [`Entity::xp_pool`]). Signed:
    /// Skilled Parens grants +60, Weak Parens −60. Summed across selections and
    /// clamped at 0.
    ///
    /// Source: ArMDE:4964-4966 (Skilled
    /// Parens), `ArMDE:7072-7074` (Weak Parens).
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
    /// Source: ArMDE:2394.
    LaterLifeXpRate {
        /// Experience points earned per year of later life.
        amount: u32,
    },
    /// Suppresses every OTHER selection's [`Self::LaterLifeXpRate`] outright,
    /// falling straight through to the ruleset's base rate — Guild Apprentice,
    /// who is "not able to benefit from either the Poor Flaw or the Wealthy
    /// Virtue … until he moves to the journeyman stage" (D47,
    /// `docs/vf-audit/decisions.md`). A bare marker with no fields, matching
    /// the shape of [`Self::ConfersHermeticTraining`] and
    /// [`Self::WaivesAbilityAgeCap`] above: D47 explicitly rejects a general
    /// "nullify any effect" mechanism for this one caller, so this names the
    /// one family it suppresses rather than generalizing. Consumed only by
    /// `life_stage.rs::later_life_rate`.
    ///
    /// Source: ArMDE:4041-4044.
    SuppressesLaterLifeXpRate,
    /// Narrows the age → maximum-Ability-score cap to `num/den` of its normal value
    /// (rounded **up**) for Abilities the catalogue marks `locality_dependent`:
    ///
    /// > The maximum scores at character creation for locality-dependent Abilities
    /// > like Language, Area Lore, or Organization Lore, as well as some social
    /// > Abilities, are half (round up) that which his age normally allows.
    ///
    /// Applies to the *cap*, not the cost: such an Ability is bought at the usual
    /// price, just not as high. Source: ArMDE:6160 (Foreign Upbringing).
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
    /// Source: ArMDE:2315.
    ///
    /// Each entry may be gated on the declaring item's own parameter (D14 shape
    /// 1/2, W2's exclusive choice — see [`AbilityRef`]/[`CategoryRef`]): an
    /// entry whose gate does not hold for a given selection contributes
    /// nothing, which is what lets Wise One's "either Arcane or Academic, but
    /// not both" (`ArMDE:5259`) be modelled as one list rather than as an
    /// unconditional (and over-permissive) union of both.
    AbilityAuthorization {
        /// Specific Abilities permitted.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        abilities: Vec<AbilityRef>,
        /// Whole categories permitted.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        categories: Vec<CategoryRef>,
    },
    /// A competence bonus targeting one of several fixed Ability ids, each
    /// active only when its own [`ParamGate`] holds — the gated sibling of
    /// [`Effect::AbilityBonus`] for a target that is a **fixed list of Ability
    /// ids**, not the selection's own free parameter (Student of (Realm)'s "+2
    /// bonus on all uses of the appropriate Lore", row 50(a)): the four realm
    /// Lores are four separate, non-parameterized Ability ids, not one
    /// parameterized Ability whose value the `realm` parameter could name
    /// directly (see `docs/vf-audit/design-c0-parameter-model.md` § 3).
    ///
    /// A gate-active target is also itself permission to own that Ability —
    /// the bonus could never apply to an Ability the character may not buy —
    /// so `ability_authorizations()` folds this the same way it folds
    /// [`Effect::AbilityAuthorization`], which is what implements the
    /// passage's own final clause ("You may take that Lore at character
    /// generation even if you cannot learn other Arcane Abilities").
    ///
    /// Source: ArMDE:5052-5055.
    AbilityBonusGated {
        /// The candidate targets; only the gate-active ones apply.
        targets: Vec<AbilityRef>,
        /// Points added to the effective score of each active target.
        amount: i8,
    },
    /// Adjusts the character's derived Confidence Score and Points (on top of the
    /// type profile's defaults). Signed and additive; e.g. Self-Confident grants
    /// `{ score: 1, points: 2 }` (raising the 1/3 default to 2/5). Confidence is
    /// never stored on the entity — it is `profile default + Σ this effect`.
    ///
    /// Source: ArMDE:4900-4902 (Self-Confident).
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
    /// Source: ArMDE:4471-4474.
    SpellMasteryXp {
        /// Mastery experience points granted per selection.
        amount: u16,
    },
    /// Floors the Spell Mastery Ability score of *every* known spell at `score`.
    /// Flawless Magic auto-masters every spell learned (Mastery 1); the effective
    /// mastery of a spell is `max(bought, this floor)`.
    ///
    /// Source: ArMDE:3887-3889.
    GrantsSpellMastery {
        /// The mastery-score floor granted to every known spell.
        score: u8,
        /// Advancement-Total multiplier for Spell Mastery Abilities, as an Affinity
        /// "counts as num/den of itself": Flawless Magic doubles all mastery
        /// Advancement Totals (`{2, 1}`), halving the XP charged. Absent in JSON →
        /// `{1, 1}` (no reduction — a plain floor grant). Source: ArMDE:3889.
        #[serde(default = "one_u8", skip_serializing_if = "is_one_u8")]
        advancement_num: u8,
        /// Denominator of the Advancement-Total multiplier above. Absent in
        /// JSON → `1` (no reduction).
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
    /// Source: ArMDE:5113-5116 (Templar
    /// Commander).
    GrantsSelection {
        /// The Virtue/Flaw ids granted for free.
        items: std::collections::BTreeSet<Id>,
        /// D74.4/row 55 (`docs/open-todos.md`, `docs/vf-audit/decisions.md`): the
        /// realm this specific grant stamps onto every granted item, where the
        /// book states one — Strong Faerie Blood's Second Sight is "faerie eyes"
        /// (ArMDE:5038), so this is `Some(Realm::Faerie)` there, while most
        /// `GrantsSelection` effects (Templar Commander's Brother-Knight) carry
        /// `None` and the granted copy resolves through the plain chain exactly
        /// like a bought one. Consumed by `crate::effective::vf_granted_selections`,
        /// which stamps it onto the granted `Selection`'s own `association`
        /// param ([`stamp_realm_override`]) — `crate::effective::resolve_realm`
        /// itself needs no change, since it already reads that param off any
        /// `Selection`, bought or granted.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        realm: Option<Realm>,
    },
    /// An open, id-less grant that counts toward a category cap without
    /// naming a real point item (D68.11) — Mythic Blood's hereditary "the
    /// character also gains a Minor Personality Flaw... both at no extra
    /// cost" (ArMDE:4588): the rulebook fixes no specific Flaw, so there is
    /// nothing to grant via [`Self::GrantsSelection`] (no id exists to grant,
    /// `Prereq::Has` it, or export by name) — only a category/magnitude/kind
    /// this entity is treated as holding one more of, purely for
    /// `validate_caps`'s per-category ceiling/floor counts (ArMDE:2820).
    /// Budget-exempt like every other grant, and read from the BOUGHT
    /// selection carrying it alone, never from a granted row (mirrors
    /// `validate_caps`'s existing "counts `entity.selections` only" scope).
    ///
    /// Source: ArMDE:4588.
    GrantsCategoryCount {
        /// The category this entity is treated as holding one more item of.
        category: String,
        /// The magnitude that phantom item carries.
        magnitude: Magnitude,
        /// Which item kind (Virtue vs Flaw) the phantom item counts as.
        item_kind: ItemKind,
    },
    /// Grants `amount` starting levels of enchanted devices (base 0, summed).
    /// Magic Items grants +25 (stackable), Redcap 50.
    ///
    /// Source: ArMDE:4347-4349 (Magic
    /// Items), `ArMDE:4842-4846` (Redcap).
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
    /// Source: ArMDE:4476-4479 (Virtue),
    /// :10410 (lesser-enchantment Lab-Total-≥-2×level rule).
    MasterpieceItem,
    /// Grants a derived True Faith Score (base 0, summed across grants). True
    /// Faith is a special score with its own rules, not a Supernatural Ability.
    /// The True Faith Virtue confers Score 1.
    ///
    /// Source: ArMDE:5169-5171.
    TrueFaithGrant {
        /// True Faith Score added.
        score: u8,
    },
    /// F-256: a Relic's own True Faith Score (ArMDE:17607: "Only by possessing
    /// the True Faith Major Virtue may a **character** have a True Faith
    /// score"). Deliberately **not** consumed by
    /// [`crate::effective::true_faith`] — that reads only
    /// [`Effect::TrueFaithGrant`], the character's own Virtue. Surfaced-only
    /// today: the relic's "usable by its bearer as Confidence" and "grants
    /// Magic Resistance equal to ten times its True Faith score to its
    /// bearer" (ArMDE:17623) are not yet separately modelled.
    ///
    /// Source: ArMDE:17607, :17623.
    RelicTrueFaith {
        /// The relic's own True Faith Score (not the bearer's).
        score: u8,
    },
    /// Grants Warping Points (base 0, summed across grants). Warped by Magic
    /// confers 5 Warping Points. No `score` field: the Warping Score is
    /// always derived from the point total alone
    /// (`effective/warping.rs::warping_score`), never authored directly, so a
    /// stored score could only ever be a disagreeable, unread second copy of
    /// the same fact (D77.3 — a field this variant used to carry and X9c's
    /// descriptor sweep found nothing ever read).
    ///
    /// Source: ArMDE:7019-7021.
    WarpingGrant {
        /// Warping Points added.
        points: u8,
    },
    /// The parameterized sibling of [`Self::WarpingGrant`] (Raised from the
    /// Dead, D69.1): grants `base_points` Warping Points unconditionally, plus
    /// one per unit named by the OWNING selection's `params[param]` — "at
    /// least three Warping points, plus one Warping point for every year that
    /// has passed since you were resurrected" (ArMDE:6648). An unanswered
    /// parameter contributes 0 extra, following every other parameterized
    /// grant's "additive on top of an otherwise-unaffected base" convention
    /// (see [`Self::ConfersHermeticTrainingIf`]'s doc comment) — never a
    /// missing-data error, since the floor points are unconditional. Carries
    /// no `score` field, same as `WarpingGrant` (D77.3): the Warping Score is
    /// always derived from the point total alone
    /// (`effective/warping.rs::warping_score`), never authored directly, so
    /// neither grant carries anything that could disagree with it. The
    /// ongoing "+1 Warping point every year you continue living" clause is
    /// NOT this effect's concern — it is a per-year accrual after creation,
    /// not a creation-time constant (D69.1), and stays text.
    ///
    /// Source: ArMDE:6646-6649.
    WarpingGrantParam {
        /// Parameter key (a `ParameterDomain::Number` parameter on the SAME
        /// item) naming the years since resurrection.
        param: String,
        /// Warping Points granted regardless of the parameter.
        base_points: u8,
    },
    /// Adds `amount` to the character's derived Size (base 0). Size is not a
    /// bought Characteristic; it is a separate racial stat modified only by these
    /// grants (Large +1, Giant Blood +2, Small Frame −1, Dwarf −2, Blood of the
    /// Nephilim +1). Summed across selections.
    ///
    /// **The complete Size-affecting sweep** (D69.5,
    /// `docs/vf-audit/decisions.md`): these five carriers are the only shipped
    /// items that change Size, and [`ItemPredicate::AffectsSize`] ranges over
    /// exactly this effect to enforce Blood of the Nephilim's own "Virtues or
    /// Flaws that affect your Size, such as Giant..." exclusion
    /// (ArMDE:3517) without hand-enumerating a closed id list.
    ///
    /// Source: ArMDE:3975-3978 (Giant
    /// Blood +2), `ArMDE:4229-4231` (Large +1), `ArMDE:6767-6769` (Small Frame −1),
    /// `ArMDE:5996-5998` (Dwarf −2), `ArMDE:3504-3518` (Blood of the Nephilim +1).
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
    /// Source: ArMDE:3975-3978 (Giant
    /// Blood, +1 Str/Sta to +6), `ArMDE:5996-5998` (Dwarf, −1 Str/Sta to −6).
    CharacteristicScoreDelta {
        /// The Characteristic id (`characteristic.str`, …) this bonus targets.
        characteristic: Id,
        /// The free effective-score bonus per selection (may be negative).
        amount: i8,
        /// This grant applies only when the OWNING selection's own gate holds
        /// — the fixed-target twin of [`Self::CharacteristicScoreDeltaParam`]'s
        /// own `gate` (X6a/e1: Faerie Blood's Sidhe clause, +1 Presence only
        /// for that heritage). Absent for every existing carrier — additive,
        /// no `SCHEMA_VERSION` bump (`Effect` lives in ruleset JSON, not
        /// saves).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
    },
    /// Lowers the **buy cap** of a fixed Characteristic while this selection
    /// is in effect (bought or granted) — Uninspirational, "His Presence and
    /// Communication may not be greater than 0" (ArMDE:6919-6922). The
    /// sign-mirror of [`Self::CharacteristicScoreDelta`]'s free bonus: this
    /// narrows the printed ±3 buy RANGE itself rather than adding a free
    /// delta on top. Folded by
    /// [`crate::effective::characteristic_cap`] as the lowest `max` any
    /// effective selection carries for the named Characteristic, floored
    /// against the ruleset's own base cap never being raised by this effect
    /// (only lowered). Two instances on Uninspirational's own entry (Pre,
    /// Com), each `max: 0`.
    ///
    /// Source: ArMDE:6919-6922 (D69, X7b-e row 42).
    CharacteristicMax {
        /// The Characteristic id (`characteristic.pre`, …) this cap targets.
        characteristic: Id,
        /// The lowered buy cap.
        max: i8,
    },
    /// Authorizes the character to start with one Reputation of the given `kind`
    /// at the given `score` (content is player-supplied). A starting Reputation is
    /// legal only if backed by such a grant
    /// (ArMDE:2514). A `kind` of `None`
    /// is a
    /// **player-chosen-type** grant (Famous,
    /// ArMDE:3861-3863: "Choose … one type"):
    /// it authorizes one Reputation of *any* type. Concrete-kind grants authorize
    /// only that type; an item with two audiences (e.g. Senior Clergy, both local
    /// and Church) carries two `GrantsReputation` effects.
    ///
    /// Source: ArMDE:6310-6312 (Infamous),
    /// `ArMDE:5703-5705` (Black Sheep), `ArMDE:3861-3863` (Famous, player-chosen kind).
    GrantsReputation {
        /// Which audience the granted Reputation reaches; `None` = player-chosen
        /// (any type).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<ReputationType>,
        /// The level of the granted Reputation. Exact by default — the entity's
        /// Reputation must equal this — unless `max_score` states a range
        /// (D11/Q5, ArMDE:2514).
        score: u8,
        /// The upper bound of a stated range, absent for the exact-by-default
        /// case. **Exactly one** of the 31 granting entries states a range —
        /// Outsider, "a bad Reputation of level 1 to 3" (ArMDE:6554) — so this is
        /// `score: 1, max_score: Some(3)` on both magnitudes; every other
        /// granter leaves it absent, and 30 of the 31 entries' JSON is
        /// therefore unchanged.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_score: Option<u8>,
        /// This grant applies only when the OWNING selection's own gate holds
        /// — same meaning as [`Self::CharacteristicScoreDeltaParam`]'s own
        /// `gate` field (Q-51/B4).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
    },
    /// Marks that this item grants (in the book's own narrative sense) a
    /// Personality Trait — e.g. Berserk's "Angry +2" (Q-resolutions #5) or
    /// Companion Animal's fixed trait (F-389). **Consumed only by
    /// [`ItemPredicate::GrantsPersonalityTrait`]'s derivation** (B3/F-542
    /// clause 2: "Virtues or Flaws that grant Personality Traits" is a
    /// predicate, not a category — `flaw.weak_personality`'s second clause).
    /// The trait itself stays player-recorded free text in
    /// `Entity::personality_traits` (D3): this effect adds no fold of its
    /// own into that list, on the same "narrative grant, not an engine
    /// total" precedent every existing "grants a Personality Trait" entry
    /// already carries as `Classification::UncomputedRule` — the V/F audit's
    /// batch-11 and batch-19 passes verified directly that no prior `Effect`
    /// variant modelled this at all, so B3 introduces the shape the
    /// predicate needs to derive from, without changing how any trait is
    /// actually recorded.
    ///
    /// **A fieldless marker, deliberately** (orchestrator amendment,
    /// 2026-09-28): the trait's name is exactly the translatable string
    /// `rules/core/` may never carry (CLAUDE.md, "strict separation of data
    /// kinds") — its wording already lives in the entry's own i18n
    /// `description`, and D3 keeps the value itself free text on the entity.
    /// A `name`/`value` pair here would duplicate that text in the
    /// language-neutral layer for no consumer's benefit.
    GrantsPersonalityTrait,
    /// Tightens the universal ±3 Personality Trait range
    /// (`validation/scores.rs::validate_personality_traits`, ArMDE:2500-2503)
    /// to `max` while this selection is in effect (bought or granted) — Weak
    /// Personality, "all Personality Traits must be between +1 and -1"
    /// (ArMDE:7076-7079). The universal Major-Personality-Flaw ±6 budget
    /// exception is skipped entirely once any selection carries this effect,
    /// since the tightened range replaces the whole scheme rather than
    /// composing with it — no shipped entry needs both at once.
    ///
    /// Source: ArMDE:7076-7079 (D69, X7b-e row 42).
    PersonalityTraitRange {
        /// The tightened `|value|` ceiling.
        max: i8,
    },
    /// Requires at least two distinct [`crate::types::PersonalityTrait`]
    /// entries at exactly `value` while this selection is in effect — Fickle
    /// Nature, "Select a Personality Trait at +4, and its opposite at +4"
    /// (ArMDE:6122-6124). The "opposite" pairing itself (Happy/Sad, etc.) is a
    /// table judgement over free text — the passage's own "Typical
    /// Personality Traits are:" is illustrative, not a closed list — so only
    /// the count at the exact value is verified, never which two traits.
    /// Consumed by `validation/scores.rs::validate_personality_trait_pairs`.
    ///
    /// Source: ArMDE:6122-6124 (D69, X7b-e row 42).
    RequiresPersonalityTraitPair {
        /// The exact trait value both members of the pair must carry.
        value: i8,
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
    /// Source: RoP:I:4120 (Demonic
    /// Blood, Infernal Might 5), `RoP:I:4136` (Demonic Might, +2); RoP:D:1975
    /// (Strong Angelic Heritage, Divine Might age ÷ 20).
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
    /// Source: RoP:I:4122 (Demonic
    /// Blood, 30 levels), `RoP:I:4142` (Demonic Powers, +20); RoP:D:1977
    /// (Strong Angelic Heritage, 30 levels).
    PowerLevels {
        /// Levels of supernatural powers added to the budget.
        amount: u16,
    },
    /// Grants `amount` **Focus Power points** (base 0, summed) — a second,
    /// separate power currency from [`Effect::PowerLevels`].
    ///
    /// "This Virtue grants a pool of 25 points. The maximum level of effect and
    /// Penetration both start at zero. It costs 2 points to raise the maximum
    /// level of effect by 1, and 1 point to raise the Penetration by 1"
    /// (ArMDE:3899), and "This Virtue may be taken more than once, and the points
    /// gained may be combined" (`ArMDE:3903`) — hence base 0 and summed, exactly
    /// like [`Effect::PowerLevels`].
    ///
    /// It is deliberately **not** `PowerLevels`: at 2 points per level these 25
    /// buy at most 12 levels, so feeding them into the level-denominated budget
    /// would let a Focus Power pay for 25 levels the Virtue cannot buy. Consumed
    /// by [`crate::effective::focus_points_budget`]; the spending side is
    /// [`crate::effective::focus_points_used`].
    ///
    /// Source: ArMDE:3899, :3903.
    FocusPoints {
        /// Focus Power points added to the pool.
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
    /// Source: ArMDE:4399-4422 (Major),
    /// `ArMDE:4536-4542` (Minor, one-focus limit at `ArMDE:4542`).
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
    /// Source: ArMDE:4524-4527 (Method
    /// Caster).
    CastingTotalMod {
        /// Points added to (or, when negative, removed from) the Casting Total.
        amount: i8,
        /// Which spells the modifier applies to.
        scope: CastingScope,
        /// This amount is counted only within the maga's own Potent Magic
        /// field, never the unconditional Casting Total (Potent Magic
        /// Major/Minor — ArMDE:4740-4748, D79: "a bonus in her field of
        /// magic", not everywhere). Orthogonal to `scope`, which is the
        /// cast-TYPE axis (formulaic/ritual/spontaneous); this is the
        /// field-gating axis, mirroring [`LabTotalModScope::WithinPotentFieldOnly`]
        /// on the Lab Total side. `false` for every carrier that predates
        /// this field (Method Caster, Cyclic Magic, Special Circumstances),
        /// so every pre-D79 entry's JSON is unchanged.
        #[serde(default, skip_serializing_if = "is_false")]
        potent_field_only: bool,
    },
    /// A flat modifier to a magus's Lab Total (Inventive Genius +3). Computed by
    /// `derived.rs` (5i).
    ///
    /// Source: ArMDE:4151-4154.
    LabTotalMod {
        /// Points added to (or, when negative, removed from) the Lab Total.
        amount: i8,
        /// Where this amount counts in the in-play (D4) grid — see
        /// [`LabTotalModScope`]. Defaults to [`LabTotalModScope::InPlayGrid`],
        /// so every entry shipped before this field existed keeps its exact
        /// behavior.
        #[serde(default, skip_serializing_if = "is_default_lab_total_mod_scope")]
        scope: LabTotalModScope,
        /// This amount is EXCLUDED from the in-play grid while the OWNING
        /// selection's own gate holds — Cyclic Magic (Negative)'s D52 cycle
        /// gate: the penalty applies "unless [the cycle] is seasonal", since a
        /// seasonal cycle's negative half aligns with season boundaries,
        /// reintroducing the same "which season am I in" uncertainty that
        /// keeps the Virtue's own bonus out of the grid entirely (D4). `None`
        /// for every other carrier — additive, byte-compatible, no
        /// `SCHEMA_VERSION` bump.
        ///
        /// Source: ArMDE:3635-3638.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        suppressed_when: Option<ParamGate>,
    },
    /// Halves the creation-time spell-level cap
    /// ([`crate::effective::spell_level_cap`]) for a spell whose Range is
    /// beyond Touch — Eye, Voice, Sight, or Arcane Connection — matched **by
    /// name** and never derived from [`crate::spell::SpellRange`]'s ordering:
    /// Eye is the same RDT difficulty as Touch, yet the book names it
    /// explicitly (Short-Ranged Magic). D28 (`docs/vf-audit/decisions.md`).
    ///
    /// A no-op everywhere else: the Flaw's sibling clause ("Halve your
    /// Casting Totals whenever you are not touching the target of the
    /// spell") is a per-cast circumstance the engine cannot resolve at
    /// character-generation time, so it is not modelled by any effect and
    /// stays textual (the catalogue entry's `description`, D20).
    ///
    /// Source: ArMDE:6739.
    HalvesSpellCapBeyondTouch,
    /// Deficient Art: all casting and lab totals that add the Technique or Form
    /// named by the selection's `params[param]` are **halved** (Deficient Form
    /// excludes Magic Resistance). The param's domain is [`ParameterDomain::Technique`]
    /// or [`ParameterDomain::Form`], so the class restriction (Deficient Technique
    /// cannot target a Form, and vice-versa) is enforced by parameter-domain
    /// resolution; the halving *scope* is derived at compute time from the
    /// targeted Art's `ArtType`. Computed by `derived.rs` (5i).
    ///
    /// Source: ArMDE:5913-5915
    /// (Technique), `ArMDE:5909-5912` (Form).
    DeficientArt {
        /// Parameter key whose value names the deficient Technique or Form.
        param: String,
    },
    /// Halves a whole in-play total of the given kind (Weak Enchanter halves lab
    /// totals for enchanting; Weak Magic halves penetration). Computed by
    /// `derived.rs` (5i).
    ///
    /// Magic Resistance is deliberately **not** a member of
    /// [`HalvableTotal`]: the one Flaw that halved it, Flawed Parma Magica,
    /// halves one *addend* of it (the Parma contribution) against one *Form*,
    /// which is neither a whole total nor blanket — it is
    /// [`MagicResistanceEffect::HalvedParma`] instead.
    ///
    /// Source: ArMDE:7060-7063 (Weak
    /// Enchanter), `ArMDE:7064-7067` (Weak Magic).
    MagicTotalHalving {
        /// Which in-play total is halved.
        total: HalvableTotal,
    },
    /// A flat modifier to Soak (Tough +3, Frail −1). Consumed by `derived.rs`
    /// `soak()` (5i).
    ///
    /// Source: ArMDE:5145-5147 (Tough),
    /// `ArMDE:6190-6193` (Frail).
    SoakMod {
        /// Points added to (or, when negative, removed from) Soak.
        amount: i8,
        /// This modifier applies only when the OWNING selection's own gate
        /// holds (X6a/e1: Repellent's "scales" branch, +3 Soak only for that
        /// enumerated choice). Absent for every existing carrier — additive,
        /// no `SCHEMA_VERSION` bump (`Effect` lives in ruleset JSON, not
        /// saves).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
    },
    /// A flat modifier to one combat total (Berserk, Lame, Missing Hand). An item
    /// may carry several (one per affected `target`). Consumed by `derived.rs`
    /// `combat_totals()` (5i).
    ///
    /// Source: ArMDE:3500-3503 (Berserk).
    CombatMod {
        /// Points added to (or, when negative, removed from) the combat total.
        amount: i8,
        /// Which combat total the modifier affects.
        target: CombatStat,
        /// Restricts the modifier to the combat lines of one weapon, and
        /// **replaces** the same item's unscoped figure for that weapon and
        /// `target`. `None` — the default, and the shape every entry but Lame
        /// uses — applies to every weapon.
        ///
        /// The scope is a weapon `Id` rather than an enum because the weapon
        /// catalogue is data, not a fixed taxonomy. It exists because the book
        /// gives some penalties *two* figures split by how you defend: Lame is
        /// "-3 on Dodge, and -1 on other combat scores" (ArMDE:6332), and Dodge
        /// is a row of the weapon table using Brawl (ArMDE:16959), so the two
        /// figures land on two genuinely different Defense Totals. Replacement
        /// rather than addition is what "**other** combat scores" means: the
        /// -1 never applied to Dodge in the first place.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        weapon: Option<Id>,
    },
    /// A modifier to the penalty on a health track (Enduring Constitution reduces
    /// wound and fatigue penalties). `Recovery` is surfaced-only (the app does not
    /// simulate recovery rolls); the wound and fatigue tracks are consumed by
    /// `derived.rs` wound/fatigue read-outs (5i).
    ///
    /// Source: ArMDE:3751-3754 (Enduring
    /// Constitution).
    HealthMod {
        /// Which health track the modifier affects.
        track: HealthTrack,
        /// Signed modifier to that track's penalty (positive reduces the penalty
        /// magnitude in 5i's read-out; the sign convention is pinned there).
        amount: i8,
    },
    /// A Magic Resistance modifier (Limited Magic Resistance drops one Form's
    /// bonus; Flawed Parma Magica halves the Parma contribution against one Form;
    /// Susceptibility to Faerie/Infernal Power halves resistance against one
    /// realm's effects; Commanding Aura adds a bonus while in a matching aura;
    /// Weak Magic Resistance waives an attacker's spell-level subtraction under a
    /// stated condition). Consumed by `derived.rs` `magic_resistance()` (5i).
    ///
    /// `param` names the selection parameter carrying the **Form** the modifier
    /// is scoped to, for the two kinds the rulebook scopes that way
    /// ([`MagicResistanceEffect::NoFormBonus`] and
    /// [`MagicResistanceEffect::HalvedParma`]). It is `None` for the realm- and
    /// scene-conditional kinds, which name no Form at all. A `param` naming a key
    /// the selection has not filled leaves the modifier **unapplied** — the
    /// missing choice is `validation::selections`'s `missing_param`, and guessing
    /// a Form here would invent a rules choice the save never stored.
    ///
    /// Source: ArMDE:6346-6349 (Limited),
    /// `ArMDE:6142-6145` (Flawed Parma), `ArMDE:6819-6826` (Susceptibility),
    /// `ArMDE:3579-3596` (Commanding Aura), `ArMDE:7068-7070` (Weak Magic
    /// Resistance).
    MagicResistanceMod {
        /// Which Magic Resistance modifier this is.
        kind: MagicResistanceEffect,
        /// Parameter key whose value names the Form this modifier is scoped to.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        param: Option<String>,
        /// The flat Magic Resistance bonus (X6a/e1-e2: [`MagicResistanceEffect::AuraBonus`]'s
        /// Commanding Aura figure — Pope rank grants 25). `0` (the default)
        /// for every kind that carries no number of its own. Additive, no
        /// `SCHEMA_VERSION` bump (`Effect` lives in ruleset JSON, not saves).
        #[serde(default, skip_serializing_if = "is_zero_i8")]
        amount: i8,
        /// This modifier applies only when the OWNING selection's own gate
        /// holds (Commanding Aura's `rank` enumeration). Absent for every
        /// existing carrier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
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
    /// Source: ArMDE:5187-5190 (Unaging),
    /// `ArMDE:16567-16569` (the AGING TOTAL the modifiers feed).
    AgingMod {
        /// Which aging / longevity subsystem the modifier touches.
        kind: AgingEffect,
        /// Signed modifier (0 when `kind` is itself the whole effect, e.g. an
        /// aging immunity).
        amount: i8,
    },
    /// A study / advancement source-quality modifier — **surfaced-only**: the app
    /// does not simulate advancement. `source` names the advancement source.
    /// Exactly one of `amount`/`factor` is present — validated at load,
    /// `ruleset/integrity.rs::validate_advancement_mod_shape` — never neither
    /// and never both (D55, V/F audit Q-113/Q-32). `amount` is a flat signed
    /// modifier to that source's Source Quality/Advancement Total (Apt Student
    /// +5 when taught); `factor` multiplies the assembled Advancement Total
    /// instead (Incomprehensible, Loose Magic — both state a halving, never an
    /// amount). `amount: 0` used to double as a "halved" marker on those two
    /// entries; it no longer does. 5i surfaces these labelled.
    ///
    /// Source: ArMDE:3422-3425 (Apt
    /// Student), :6294-6297 (Incomprehensible), :6354-6357 (Loose Magic).
    AdvancementMod {
        /// The advancement source the modifier applies to.
        source: AdvancementSource,
        /// Signed flat modifier to that source's Source Quality/Advancement
        /// Total. Mutually exclusive with `factor`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        amount: Option<i8>,
        /// Multiplies the source's assembled Advancement Total instead of
        /// adding to it. Mutually exclusive with `amount`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        factor: Option<AdvancementFactor>,
    },
    /// A special casting-style quirk. `kind` names the quirk. The three
    /// non-standard-casting penalty relievers (`quiet_words`, `subtle_gestures`,
    /// `deft_form`) are **computed** by 5i into the per-cell
    /// [`crate::derived::NonStandardCasting`] variants (the residual no-voice /
    /// no-gesture penalties); every other quirk — spontaneous-magic variants
    /// (Diedne, Faerie-Raised, Life-Linked) and circumstantial casting penalties —
    /// is **surfaced-only**, listed labelled rather than simulated.
    ///
    /// Source: ArMDE:3645-3648 (Deft
    /// Form), :4822-4826 (Quiet Magic), :5073-5076 (Subtle Magic), :9243-9245
    /// (Words/Gestures penalties), `ArMDE:3675-3682` (Diedne Magic), `ArMDE:5917-5920`
    /// (Deleterious Circumstances), `ArMDE:6815-6817` (Susceptibility to Divine
    /// Power's doubled aura penalties).
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
    /// A flat modifier to rolls of a FIXED Ability, named directly by this
    /// entry rather than by a player-chosen parameter (Poor Hearing: -3 to
    /// Awareness rolls) — the fixed-target twin of
    /// [`Self::AbilityRollModParam`]'s free-text subject, on the same
    /// "base name = fixed target, `...Param` = parameter-relative target"
    /// convention [`Self::AbilityScoreGrant`]/[`Self::AbilityScoreGrantParam`]
    /// and [`Self::CharacteristicScoreDelta`]/[`Self::CharacteristicScoreDeltaParam`]
    /// already follow (B5/F-489: `AbilityRollMod` predates the convention,
    /// which is why the OLD parameter-relative shape is the one that got
    /// renamed, not this one). **Surfaced-only**: it modifies rolls, not the
    /// bought/effective Ability score, so it never perturbs creation. 5i
    /// surfaces it labelled.
    ///
    /// Source: ArMDE:6614-6617 (Poor Hearing, the worked example).
    AbilityRollMod {
        /// The Ability this modifier always targets.
        ability: Id,
        /// Points added to rolls of that ability.
        amount: i8,
        /// This modifier applies only when the OWNING selection's own gate
        /// holds (X6a/e1: Faerie Blood's Dwarf clause, +1 Craft only for that
        /// heritage). Absent for every existing carrier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
    },
    /// A flat modifier to rolls of a specific Ability in the free-text subject
    /// named by the selection's `params[param]` (Academic Concentration: a bonus
    /// to Concentration for one field of study). **Surfaced-only**: it modifies
    /// rolls, not the bought/effective Ability score, so it never perturbs
    /// creation. 5i surfaces it labelled.
    ///
    /// Renamed from `AbilityRollMod` (B5/F-489): the base name now names the
    /// FIXED-target twin above. Ruleset-JSON-only rename (no `SCHEMA_VERSION`
    /// bump — `Effect` lives in `rules/core/`, not in saves): the one known
    /// carrier, `virtue.academic_concentration_subject`, is updated in the
    /// same commit.
    ///
    /// Source: ArMDE:3362-3367.
    AbilityRollModParam {
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
    /// Source: ArMDE:3731-3737.
    ElementalMagic {
        /// The elemental Form ids the redistribution pools over.
        forms: std::collections::BTreeSet<Id>,
    },
    /// Marks the character as forbidden from having a specialty on **any**
    /// Ability — Unspecialized, "The character does not have any specialties
    /// for any of her Abilities." A no-op for effective scores: nothing here
    /// changes a computed number. Consumed only by
    /// `validation/scores.rs::validate_ability_specialty_permitted`, which
    /// refuses a non-empty `AbilityScore::specialty` while the holding
    /// selection is in effect (bought or granted) — the passage forbids the
    /// *state*, not the specialization bonus
    /// (`derived/combat.rs::specialization_bonus`), which stays a no-op for
    /// this Flaw rather than learning to return 0.
    ///
    /// Source: ArMDE:6943-6946, rule
    /// `ArMDE:6945`.
    ForbidsAbilitySpecialties,
    /// Marks the character as unable to **cast** Ritual magic — Rigid Magic,
    /// "you cannot use vis when you cast spells. Thus, you cannot … cast
    /// Ritual magic." A no-op for effective scores and for spell selection
    /// itself: D27 rules the spell list models spells *known*, not spells
    /// *castable*, so holding a Ritual stays legal (a magus may have learned it
    /// before acquiring the Flaw, or keep it to teach or copy). Consumed only
    /// by `validation/magus.rs::validate_ritual_casting_restriction`, which
    /// raises an **advisory warning** — never an error — on every known spell
    /// whose catalogue entry sets `spell.rs::Spell::ritual`, while the holding
    /// selection is in effect (bought or granted).
    ///
    /// Source: ArMDE:6695-6698, rule
    /// `ArMDE:6697`.
    ForbidsRitualCasting,
    /// Waives the age → maximum-Ability-score cap entirely, for every Ability —
    /// Mentored by Demons, "Characters trained by demons may exceed the maximum
    /// skill level for a given age provided by the character creation rules."
    /// Unlike [`Effect::AffinityAbilityCost`]'s +2 (which still leaves a ceiling)
    /// this removes the ceiling outright, and unlike
    /// [`Effect::LocalityAbilityCapFraction`] (which narrows one flagged Ability)
    /// it applies to every Ability the character holds: the passage names no
    /// list. A bare marker with no target, consumed by the single age-cap
    /// resolution point (D29, `effective/reputation_and_caps.rs::ability_age_cap`)
    /// rather than by a second check beside it.
    ///
    /// A favored-Ability override that raises (not waives) one *named* Ability's
    /// cap above the age band — Savantism's "may not begin with an Ability above
    /// 3 [general], except for one favored Ability, which is limited to a score
    /// of 6" — is a different shape (D9's parameter work) and unshipped; this
    /// variant states no favored Ability because Mentored by Demons names none.
    ///
    /// Source: ArMDE:4498.
    WaivesAbilityAgeCap,
    /// The favored-Ability override this variant's own doc comment above
    /// anticipated (X6a/e6, D9): the Ability named by the OWNING selection's
    /// `params[param]` caps at `max` INSTEAD OF the age-band figure — Savantism,
    /// "except for one favored Ability, which is limited to a score of 6"
    /// (ArMDE:6705-6706). Folds into the single
    /// age-cap resolution point (D29,
    /// `effective/reputation_and_caps.rs::ability_age_cap`), never a second
    /// check beside it: an override naming the asked-about ability returns
    /// `max` outright, bypassing the age band entirely (so it may raise the
    /// cap, not just lower it).
    ///
    /// Source: ArMDE:6705-6706.
    AbilityScoreCapOverrideParam {
        /// Parameter key whose value names the favored Ability.
        param: String,
        /// The flat cap the favored Ability gets instead of the age band.
        max: u8,
    },
    /// Savantism's sibling clause (X6a/e6): every OTHER Ability — every one
    /// but the one named by the SAME item's `param` — caps at `max`, lowering
    /// (never raising) the otherwise-applicable age band. Folds into the same
    /// single age-cap resolution point as
    /// [`Self::AbilityScoreCapOverrideParam`].
    ///
    /// Source: ArMDE:6705-6706.
    AbilityScoreCapAllExcept {
        /// Parameter key whose value names the ONE Ability this cap does
        /// NOT apply to (Savantism's own favored Ability).
        param: String,
        /// The lowered cap for every other Ability.
        max: u8,
    },
    /// Marks the character as Hermetically trained without the type profile
    /// itself declaring so — the Abandoned Apprentice Flaw, "you have most of
    /// the skills and knowledge of a fully trained magus, but you were never
    /// able to complete your training and gain The Gift" (D56/A0). A bare
    /// marker with no fields: it names no ability, no amount, nothing to
    /// resolve, matching the shape of [`Effect::MasterpieceItem`],
    /// [`Effect::ForbidsAbilitySpecialties`], [`Effect::ForbidsRitualCasting`]
    /// and [`Effect::WaivesAbilityAgeCap`] above. Consumed only by
    /// `effective/hermetic_training.rs::entity_confers_hermetic_training`,
    /// whose union with the type profile's own `hermetically_trained` flag
    /// (`effective/hermetic_training.rs::is_hermetically_trained`) is the
    /// single fact every "trained" production site must read — see
    /// `docs/vf-audit/design-a0-is-magus-split.md` § 1.
    ///
    /// Source: ArMDE:5641-5650.
    ConfersHermeticTraining,
    /// The conditional sibling of [`Self::ConfersHermeticTraining`] (same
    /// "Foo"/"FooParam"-style precedent as
    /// [`Self::AbilityScoreGrant`]/[`Self::AbilityScoreGrantParam`], Revision 4
    /// architect finding R3-1): confers Hermetic training only once the OWNING
    /// selection's own `params[param]` resolves to a value — any value, a
    /// PRESENCE test, not [`ParamGate`]'s equality test, so the two stay
    /// deliberately separate mechanisms. D3's own carrier
    /// (`flaw.abandoned_apprentice`) is the first and, today, only user: the
    /// training marker and [`Self::TruncatedApprenticeshipXp`]'s own payoff
    /// must resolve TOGETHER, or an Abandoned Apprentice reads as
    /// trained-but-funded-with-nothing the instant the Flaw is picked, before
    /// its `years_completed` parameter is answered — a real, ordinary-use bug
    /// (F1, `docs/vf-audit/design-d0-xp-modes.md` § 4), not a crafted-input
    /// edge case: every OTHER parameterized grant in this engine is additive
    /// on top of an otherwise-unaffected base, so an unanswered parameter
    /// costs nothing; this is the first case where a SIBLING effect switches
    /// which base applies at all. Widening the bare marker in place (Revision
    /// 3's first draft) was rejected: it would force `{ .. }` onto every one
    /// of that variant's seven existing production sites for a field none of
    /// them read. `entity_confers_hermetic_training` gains a second match arm
    /// for this variant rather than widening its first.
    ///
    /// Source: ArMDE:5641-5650.
    ConfersHermeticTrainingIf {
        /// Parameter key on the SAME item; presence (not any particular
        /// value) activates this marker.
        param: String,
    },
    /// Forbids the character from holding any Ability of the given
    /// [`AbilityCategory`] — Ability Block, "completely unable to learn a
    /// certain class of Abilities... This may be Martial Abilities, or a more
    /// limited set of the others" (D21/F-355). The Ability-axis half of D21's
    /// category-prohibition pair; see [`Effect::ForbidsItemCategory`] for the
    /// V/F-axis twin. Consumed by a dedicated grant-aware validator
    /// (`validation/selections.rs`), not by any score fold — a no-op
    /// everywhere in [`crate::effective`].
    ///
    /// Source: ArMDE:5651-5654.
    ForbidsAbilityCategory {
        /// The forbidden Ability category.
        category: AbilityCategory,
    },
    /// The parameter-relative sibling of [`Self::ForbidsAbilityCategory`]
    /// (X6a/e5): the forbidden category is named by the OWNING selection's
    /// own `params[param]` rather than fixed by the item — Ability Block,
    /// "This may be Martial Abilities, or a more limited set of the others"
    /// (ArMDE:5651-5654). Consumed by the SAME grant-aware validator, gaining
    /// an arm that resolves `param` against
    /// [`crate::types::ParameterDomain::AbilityCategory`] instead of matching
    /// a fixed variant.
    ///
    /// Source: ArMDE:5651-5654.
    ForbidsAbilityCategoryParam {
        /// Parameter key whose value names the forbidden Ability category.
        param: String,
    },
    /// Forbids the character from holding any OTHER Virtue/Flaw whose
    /// in-force category is this string — Weak Personality, "The character
    /// may have no other Personality Flaws or Virtues..." (D21/F-542 clause
    /// 1). `String`, not [`AbilityCategory`]: [`PointItem::categories`] is
    /// free-form, so this is the item-axis twin of
    /// [`Effect::ForbidsAbilityCategory`], not the same field reused — the two
    /// domains legally share bare strings (`"general"` names both an Ability
    /// category and nothing on the item axis), so one untagged type across
    /// both would be ambiguous at the wire.
    ///
    /// Source: ArMDE:7076-7079.
    ForbidsItemCategory {
        /// The forbidden item category.
        category: String,
    },
    /// Forbids the character from holding any of these specific Ability ids
    /// as a BEGINNING Ability — Sheltered Upbringing, "You may not take
    /// Bargain, Charm, Etiquette, Folk Ken, Guile, Intrigue, or Leadership as
    /// beginning Abilities, but you may learn them in play" (D21/F-511). A
    /// third, id-list sub-shape distinct from both category forbids above —
    /// the passage names specific Abilities, not a whole category.
    ///
    /// Source: ArMDE:6721-6724.
    ForbidsAbilities {
        /// The forbidden Ability ids.
        abilities: std::collections::BTreeSet<Id>,
    },
    /// A roll penalty over an unenumerated category of rolls ("physical
    /// activity"), scaled by `1 + Decrepitude Score` — Lingering Injury, "a
    /// -1 to your physical activity rolls (this can be as high as -3 if the
    /// wound was particularly severe, or aggravated by a botch), multiplied
    /// by whatever the penalty is by 1 + (Decrepitude Score)"
    /// (ArMDE:6350-6352). `amount` carries only the un-aggravated base (-1):
    /// the aggravated alternative (-3) depends on whether the wound was
    /// caused by a botch, a fact [`crate::types::Entity`] does not track
    /// anywhere today, so that branch stays text (a partly-computed entry
    /// stays `uncomputed_rule`, D67). Surfaced rather than folded into a
    /// simulated total, on the same precedent as every other
    /// `ModifierFamily` — `crate::derived::ModifierFamily::PhysicalActivity`
    /// is the family this produces, since no existing family fits a
    /// category-wide, Decrepitude-scaled penalty.
    ///
    /// Source: ArMDE:6350-6352 (D69, X7b-e row 42).
    DecrepitudeScaledRollMod {
        /// The un-aggravated base penalty, before the Decrepitude multiplier.
        amount: i8,
    },
}

/// A property-based test over a [`PointItem`], for an exclusion the rulebook
/// states by DESCRIPTION rather than by id ([`PointItem::incompatible_with`])
/// or by category ([`Effect::ForbidsItemCategory`]) — D23/D33
/// (`docs/vf-audit/design-c0-parameter-model.md` § 7,
/// `docs/vf-audit/design-b0-ranging-and-predicates.md` § 2). Shared vocabulary
/// for BOTH of this family's consumers, so a predicate is spelled once
/// regardless of which reads it: [`ParameterDef::exclude_if`] (D33, a
/// parameter-domain narrowing — `flaw.flawed_powers` may only import a Flaw
/// this predicate does NOT hold for) and [`PointItem::excluded_if_holds`]
/// (D23, a point-item-level exclusion — `flaw.university_dean` is illegal
/// while ANY other effective selection this predicate DOES hold for is also
/// held).
///
/// A closed, serde-checked enum (B3's own scope — C0 § 7 designed this
/// vocabulary but did not build it; verified directly,
/// `grep -rn "ItemPredicate" crates/arm-rules/src` was empty before this
/// slice).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemPredicate {
    /// D12's intrinsic/trained classification: an item that "operates on
    /// Techniques, Forms, spells, Casting/Lab Totals, Parma Magica, certámen
    /// or Twilight" (Q-138, ArMDE:6146-6148, Flawed Powers — "Any Flaw that
    /// is only appropriate to Hermetic Magic... cannot be taken with this
    /// Flaw"). Reads [`PointItem::trained`], which D12's classification pass
    /// (X3) populates wholesale; every item defaults to `false` until then,
    /// so this predicate never excludes anything in the SHIPPED catalogue
    /// today — B3 builds the machinery and proves it against hand-authored
    /// fixtures that set the flag directly (design-b0 § 1 point 5), not
    /// against real data.
    Trained,
    /// Carries an [`Effect::GrantsReputation`] effect AND is a **Flaw**
    /// (Q-137, `flaw.university_dean`, ArMDE:6923-6926: "can not have the
    /// Poor Flaw or any other Flaw that grants a Bad Reputation" — the kind
    /// check is in the passage itself: "any OTHER FLAW"). Mostly derivable
    /// with no new data: `item.kind == ItemKind::Flaw && item.effects.iter().any(|e|
    /// matches!(e, Effect::GrantsReputation { .. }))` — the kind half matters
    /// because `virtue.doctor_in_faculty`, University Dean's own required
    /// Virtue, ALSO grants a Reputation (an academic one, not Bad), and the
    /// passage excludes only Flaws.
    GrantsReputation,
    /// Carries an [`Effect::GrantsPersonalityTrait`] effect (F-542 clause 2,
    /// `flaw.weak_personality`, ArMDE:7076-7079: "...or Virtues or Flaws that
    /// grant Personality Traits"). Same derivable shape as
    /// [`Self::GrantsReputation`], scanning for
    /// [`Effect::GrantsPersonalityTrait`] instead. Unlike `GrantsReputation`,
    /// the passage names both kinds ("Virtues or Flaws"), so no kind check.
    GrantsPersonalityTrait,
    /// D68.4 (`docs/vf-audit/decisions.md`): this item's mechanic is
    /// inherently Hermetic-Arts-specific, per
    /// [`PointItem::requires_hermetic_arts`]. Supersedes an earlier D23/D33
    /// plan to reuse [`Self::Trained`] for `flaw.flawed_powers`'s import
    /// filter (ArMDE:6148, Q-138): every candidate is `trained: true`, so
    /// `Trained` cannot distinguish Deficient Technique/Unstructured Caster
    /// (excluded) from Restriction/Necessary Condition (importable).
    RequiresHermeticArts,
    /// D69.5 (`docs/vf-audit/decisions.md`): carries an [`Effect::SizeDelta`]
    /// effect — Blood of the Nephilim's "Virtues or Flaws that affect your
    /// Size, such as Giant..." (ArMDE:3517) is open-ended, so rather than
    /// hand-enumerating a closed list, this predicate ranges over every item
    /// whose mechanic actually changes Size, exactly the same way
    /// [`Self::GrantsReputation`]/[`Self::GrantsPersonalityTrait`] range over
    /// their own effects. [`Effect::SizeDelta`]'s doc comment is the complete
    /// sweep: Giant Blood, Large, Small Frame, Dwarf, and Blood of the
    /// Nephilim's own +1 are the only five carriers in the shipped catalogue
    /// (verified: no entry changes Size any other way).
    AffectsSize,
}

impl ItemPredicate {
    /// Whether `item` satisfies this predicate — the ONE evaluator shared by
    /// [`ParameterDef::exclude_if`] and [`PointItem::excluded_if_holds`]
    /// (design-c0 § 11's open convergence point, closed here) so the two
    /// mechanisms cannot drift on what a predicate means.
    pub(crate) fn holds_for(&self, item: &PointItem) -> bool {
        match self {
            ItemPredicate::Trained => item.trained,
            ItemPredicate::GrantsReputation => {
                item.kind == ItemKind::Flaw
                    && item
                        .effects
                        .iter()
                        .any(|e| matches!(e, Effect::GrantsReputation { .. }))
            }
            ItemPredicate::GrantsPersonalityTrait => item
                .effects
                .iter()
                .any(|e| matches!(e, Effect::GrantsPersonalityTrait)),
            ItemPredicate::RequiresHermeticArts => item.requires_hermetic_arts,
            ItemPredicate::AffectsSize => item
                .effects
                .iter()
                .any(|e| matches!(e, Effect::SizeDelta { .. })),
        }
    }
}

impl std::fmt::Display for ItemPredicate {
    /// Renders the same snake_case tag serde uses, for validation-issue
    /// message arguments (`ValidationIssue::CODE_EXCLUDED_BY_PREDICATE`'s
    /// `$predicate`), on [`ParameterDomain`]'s own precedent.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemPredicate::Trained => f.write_str("trained"),
            ItemPredicate::GrantsReputation => f.write_str("grants_reputation"),
            ItemPredicate::GrantsPersonalityTrait => f.write_str("grants_personality_trait"),
            ItemPredicate::RequiresHermeticArts => f.write_str("requires_hermetic_arts"),
            ItemPredicate::AffectsSize => f.write_str("affects_size"),
        }
    }
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
}

impl fmt::Display for HalvableTotal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HalvableTotal::SpontaneousCasting => "spontaneous_casting",
            HalvableTotal::LabEnchanting => "lab_enchanting",
            HalvableTotal::LabLongevity => "lab_longevity",
            HalvableTotal::Penetration => "penetration",
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
    /// Fatigue levels lost per spell cast, positive = fewer levels lost
    /// (Withstand Casting +1; Vulnerable Casting and Painful Magic are
    /// negative). Surfaced-only.
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
    /// The named Form's contribution to Magic Resistance is dropped (Limited
    /// Magic Resistance: against that Form, resistance from Parma alone).
    ///
    /// Source: ArMDE:6346-6349.
    NoFormBonus,
    /// The **Parma contribution** to Magic Resistance is halved against the
    /// named Form, and nothing else is (Flawed Parma Magica).
    ///
    /// Only the Parma half, because the Flaw's subject is "Your Parma Magica"
    /// (`ArMDE:6144`) while `ArMDE:9396` puts the rest of the resistance on "a
    /// maga's Form scores" — a defective Parma cannot reduce a number it does
    /// not produce. And only against one Form, because the Flaw is bought "for
    /// different Forms", one copy each.
    ///
    /// Source: ArMDE:6142-6145, :9390, :9396, :9398.
    HalvedParma,
    /// A bonus to Magic Resistance while in a matching aura (Commanding Aura).
    AuraBonus,
    /// A penalty to Magic Resistance against Faerie power (Susceptibility).
    SusceptibleFaerie,
    /// A penalty to Magic Resistance against Infernal power (Susceptibility).
    SusceptibleInfernal,
    /// Under a described, character-specific condition, an attacker does not
    /// subtract the spell level from the casting total before calculating
    /// Penetration against this character (Weak Magic Resistance).
    ///
    /// Surfaced-only, and necessarily so: the Flaw halves nothing and leaves the
    /// carrier's own Magic Resistance score untouched. Its two inputs — whether
    /// the condition is met, and the level of the *incoming* spell — are scene
    /// facts, so no number on this sheet can carry it.
    ///
    /// Source: ArMDE:7068-7070 (the Flaw), :7066 (normal Penetration subtracts
    /// the spell level), :9912 (the book's own gloss: "need not subtract the
    /// spell level from the Penetration total ... much like the Weak Magic
    /// Resistance Flaw").
    ConditionalPenetrationWaiver,
}

impl fmt::Display for MagicResistanceEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MagicResistanceEffect::NoFormBonus => "no_form_bonus",
            MagicResistanceEffect::HalvedParma => "halved_parma",
            MagicResistanceEffect::AuraBonus => "aura_bonus",
            MagicResistanceEffect::SusceptibleFaerie => "susceptible_faerie",
            MagicResistanceEffect::SusceptibleInfernal => "susceptible_infernal",
            MagicResistanceEffect::ConditionalPenetrationWaiver => "conditional_penetration_waiver",
        })
    }
}

/// Which aging / longevity subsystem an [`Effect::AgingMod`] touches. A fixed
/// rules taxonomy, rendered via Fluent (`derived-detail-<slug>`).
///
/// The two immunities are **orthogonal**, because the sources state them
/// separately: not dropping Characteristics and not looking older are different
/// facts, and Bound to (Role) has the first without the second (`ArMDE:5743`).
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
    /// Decrepitude points" (`ArMDE:5189`). Carried by Unaging and by Bound to (Role),
    /// which "also includes the effects of the Unaging Virtue" (`ArMDE:5743`). It says
    /// nothing about the apparent age — see [`Self::NoApparentAging`].
    ///
    /// Source: ArMDE:5189, :5743.
    NoAging,
    /// **The apparent age never advances**, whatever the roll: "Bee Kings do not
    /// appear to age after reaching maturity" (`ArMDE:3488`), and Unaging's "You may
    /// choose your apparent age freely" (`ArMDE:5189`). Consumed by
    /// `aging::resolve_outcome`, so a carrier's
    /// `AgingOutcome::apparent_age_increases` is false at every total.
    ///
    /// Bound to (Role) deliberately does **not** carry it: "but the character's
    /// apparent age advances in line with their physical age" (`ArMDE:5743`) — the
    /// sentence that proves the two immunities are separable at all. A Bee King
    /// carries this one alone, and so still loses Characteristics.
    ///
    /// Source: ArMDE:3488, :5189,
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
    /// affect aging rolls do not affect crisis survival rolls" (`ArMDE:16636`). So a
    /// modifier tagged here never reaches [`Self::AgingRoll`]'s total, and an
    /// `aging_roll` modifier never reaches the survival roll.
    ///
    /// The general prohibition does not silence a *specific* grant: Mild Aging's
    /// "he receives a +3 bonus to rolls to survive an aging crisis" (`ArMDE:4530`) is
    /// exactly such a grant, and is the first shipped item to carry this kind.
    ///
    /// Source: ArMDE:4530, :16636.
    CrisisSurvival,
    /// **A Heavy Wound whenever a crisis lands**: "whenever she undergoes an Aging
    /// Crisis (page 392) the leper sustains a Heavy Wound in addition to any other
    /// result" (`ArMDE:6340`). A marker — the `amount` is ignored and ships as 0,
    /// because this is a consequence of the crisis, not a number added to any roll.
    ///
    /// Kept separate from [`Self::CrisisSurvival`] rather than folded into one
    /// `crisis` kind: the two are different kinds of fact, and collapsing them
    /// would make the stored `amount` mean a roll modifier for one carrier and
    /// nothing at all for the next.
    ///
    /// Source: ArMDE:6340.
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
    /// The character having authored a book others study from — as opposed to
    /// `Book`, which is this character reading someone *else's* (Good
    /// Teacher's Quality-of-authored-books bonus).
    Authoring,
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
            AdvancementSource::Authoring => "authoring",
            AdvancementSource::SpellMastery => "spell_mastery",
            AdvancementSource::All => "all",
        })
    }
}

/// The multiplier an [`Effect::AdvancementMod`] applies to a source's
/// Advancement Total, when the modifier is multiplicative rather than a flat
/// `amount`.
///
/// A small enum, not a `num`/`den` pair. Contrast
/// [`Effect::GrantsSpellMastery`]'s `advancement_num`/`advancement_den`, which
/// genuinely needs a pair — Flawless Magic *doubles* a Spell Mastery
/// Advancement Total (ArMDE:3889), a different multiplier on a different
/// effect. Every stated `AdvancementMod` factor in the core rules is a
/// **halving** (Incomprehensible ArMDE:6296, Loose Magic ArMDE:6356), so an
/// enum carrying only that one value cannot express the nonsense a fraction
/// pair can (a zero denominator, a reversed ratio) — see
/// `ruleset/integrity.rs::validate_item_ratios`'s doc comment for how much
/// load-time validation a `num`/`den` pair needs to stay safe. A second stated
/// factor later is a new variant; the exhaustive `match` in every consumer
/// (`derived.rs`, the Fluent-key coverage test) turns that into a compile
/// error until it is handled, exactly as a new [`AdvancementSource`] does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvancementFactor {
    /// Halves the source's Advancement Total, rounded down: neither
    /// Incomprehensible (ArMDE:6296) nor Loose Magic (ArMDE:6356) states a
    /// rounding direction of its own, so the core book's stated default
    /// applies (ArMDE:547: "if it does not [specify], round down").
    Half,
}

impl fmt::Display for AdvancementFactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            AdvancementFactor::Half => "half",
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
    /// The aura's own penalties to this character's magic — the Aura Modifier on
    /// the Casting Score and the botch dice it adds — are **doubled** in one
    /// realm's aura (Susceptibility to Divine Power).
    ///
    /// Surfaced-only: the engine models neither an aura of a foreign realm nor
    /// botch dice, and the doubling has no value of its own — it scales whatever
    /// the scene's aura rating happens to be.
    ///
    /// Source: ArMDE:6815-6817.
    DoubledAuraPenalty,
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
            SpecialCasting::DoubledAuraPenalty => "doubled_aura_penalty",
        })
    }
}

/// The audience a Reputation reaches — a fixed rules taxonomy (so an enum, like
/// [`crate::art::ArtType`]), rendered via Fluent, never as a raw slug.
///
/// Source: ArMDE:1091-1101.
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
    /// alongside the three "main" types at `ArMDE:1097` ("The most basic type is …").
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
    /// (ArMDE:2364, :2213-2216).
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
    /// (ArMDE:2232, :16563-16617).
    Aging,
    /// The terminal phase: everything a finished character carries that no
    /// creation phase owns — equipment, magic items, Might and powers, Warping —
    /// plus a last look at the whole character. A profile may not declare it
    /// (the wizard appends it), so it is the one phase that is never skipped.
    Review,
}

impl CreationPhase {
    /// Every phase, in the order the rules' creation summary walks them
    /// (ArMDE:2205-2222), with the synthetic [`Review`](Self::Review)
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
///
/// Field order is alphabetical (`anchor`, `file`, `lines`) so the serialized
/// form satisfies `CLAUDE.md` → "Canonical serialization" without a custom
/// `Serialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    /// The Markdown heading anchor the item is defined under, as the source
    /// file's own generated cross-links spell it (`abandoned-apprentice` for
    /// `#### Abandoned Apprentice`) — **the durable half of this reference**.
    ///
    /// [`Self::lines`] is a derived coordinate: an upstream edit anywhere above
    /// the item shifts it, and the guards can only prove a range lands on
    /// non-blank lines, never that it lands on the rule the citation claims. A
    /// wholesale re-sync therefore yields a green suite and hundreds of
    /// citations silently pointing at the wrong passage (`docs/rules-source-resync.md`).
    /// An anchor survives an edit anywhere else in the book, and when it does
    /// break — because the heading was renamed, which is a semantic change worth
    /// noticing — it fails **loudly** rather than resolving to the wrong text.
    /// That asymmetry is the whole argument for carrying both.
    ///
    /// Mandatory catalogue-wide (D30.1): every catalogue that carries a
    /// `source` block at all has had its anchor sweep completed (see
    /// `rules_source_provenance.rs`'s `FULLY_ANCHORED_CATALOGUES`), so a
    /// `source` block missing this key is a data defect, not an entry still
    /// awaiting its sweep — it fails to deserialize rather than silently
    /// defaulting to an absent anchor. Always the **English** anchor —
    /// `rules/core/` is the canonical-ID language (`CLAUDE.md` → "Rules
    /// provenance"); per-language anchors live in `rules/i18n/<lang>/source_anchors.json`.
    pub anchor: String,
    /// Basename of the Markdown source file.
    pub file: String,
    /// Inclusive line range the item was extracted from.
    pub lines: LineRange,
}

impl SourceRef {
    /// Creates a source reference, with its mandatory anchor (D30.1).
    pub fn new(file: impl Into<String>, lines: LineRange, anchor: impl Into<String>) -> Self {
        Self {
            anchor: anchor.into(),
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
    /// (ArMDE:6803-6804), and a
    /// character may legitimately reach it through either. **All of them are
    /// equally real** — the book's own indexes list such an item under both
    /// headings (Suppressed Gift at
    /// ArMDE:5301 under
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
    /// Headings the book's own **index** files this entry under, *in addition
    /// to* the ones its descriptor makes membership [`Self::categories`]. This
    /// is **provenance, not membership**.
    ///
    /// The two can legitimately disagree. `hermetic` is the slug the engine
    /// also uses to decide a character has The Gift
    /// ([`EntityTypeProfile::gift_categories`]), so the two Beings Flaws the
    /// book indexes under *both* Hermetic and General
    /// (ArMDE:5445 and :5455, against
    /// `### Hermetic, Minor` at `ArMDE:5417`) may not carry `hermetic` as a
    /// membership category — an unGifted companion holding one would count as
    /// Gifted and be handed the Gift's free Supernatural-Ability slot. Recording
    /// the index heading here keeps the book's placement without granting
    /// membership anywhere.
    ///
    /// **Exactly one consumer**: `validation::magus::validate_house`'s
    /// `ArMDE:2860` "at least one Hermetic Flaw" guideline, which is a question about
    /// what the book lists, not about what the character *is*. Every membership
    /// surface — permitted/forbidden categories, category caps, grant
    /// constraints, Gift detection, [`Self::categories_for`] — and every
    /// browsing surface, `Ruleset::items_by_category` and the Markdown export's
    /// Type cell included, is deliberately blind to it.
    ///
    /// Unlike `categories`, the list carries no authored emphasis (an index is
    /// alphabetical), so it IS canonically sorted by [`Self::normalize`].
    /// Load-time integrity rejects a repeat and rejects a slug the item already
    /// carries in `categories`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub index_categories: Vec<String>,
    /// How this V/F impacts a character mechanically (M5 slice 5a). Required (no
    /// serde default): an unclassified entry fails to load. See [`Classification`].
    pub classification: Classification,
    /// The descriptor's optional "Type" tag. `true` for a Tainted Virtue/Flaw:
    /// associated with the Infernal realm, and any Supernatural Ability it grants
    /// is an Infernal power. Drives the half-of-taken-points Tainted cap
    /// (no more than half a character's Virtue points — and likewise Flaw points —
    /// may be Tainted).
    ///
    /// Source: ArMDE:2998-3002.
    #[serde(default, skip_serializing_if = "is_false")]
    pub tainted: bool,
    /// D42/D70/D74: how this entry's realm association resolves beyond the
    /// plain override/concept/Magic chain. `None` for every Supernatural
    /// entry the book leaves free. See [`RealmAssociation`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realm_association: Option<RealmAssociation>,
    /// D81.14 (`docs/vf-audit/decisions.md`): `true` for an entry ArMDE:2280's
    /// "faerie-related Virtue or Flaw" (Merinita's conditional Warping Point)
    /// reads as faerie-related despite carrying no [`Self::realm_association`]
    /// at all — Faerie Friend, Faerie Upbringing, Susceptibility to Faerie
    /// Power. An entry whose realm resolves to [`Realm::Faerie`] (Faerie
    /// Blood, Bound to (Realm) at Faerie, …) needs no flag; the predicate
    /// reads `realm_association` for those. Never set on `virtue.faerie_magic`
    /// itself — the House's own grant, which every Merinita magus holds by
    /// construction, so flagging it would make the clause permanently
    /// vacuous.
    #[serde(default, skip_serializing_if = "is_false")]
    pub faerie_related: bool,
    /// D12's intrinsic/trained classification: `true` when this item
    /// operates on Techniques, Forms, spells, Casting/Lab Totals, Parma
    /// Magica, certámen, or Twilight — things that exist only after
    /// apprenticeship — as opposed to an "intrinsic" item operating on The
    /// Gift itself. Read by [`ItemPredicate::Trained`] (Q-138/B3,
    /// `flaw.flawed_powers`'s "only appropriate to Hermetic Magic" import
    /// constraint).
    ///
    /// **Populated wholesale by D12's classification pass (X3), not this
    /// slice.** `false` (the default) is every shipped item's value today —
    /// B3 builds the field and the machinery that reads it; hand-authored
    /// test fixtures set it directly to exercise that machinery ahead of
    /// X3's real catalogue pass (design-b0 § 1 point 5).
    #[serde(default, skip_serializing_if = "is_false")]
    pub trained: bool,
    /// D68.4 (`docs/vf-audit/decisions.md`): `true` when this item's mechanic
    /// is inherently Hermetic-Arts-specific — it operates on Techniques,
    /// Forms, or normal Hermetic casting so directly that it cannot sensibly
    /// apply to a Supernatural Virtue at all (Deficient Technique,
    /// Unstructured Caster) — as opposed to a Flaw that merely happens to
    /// carry `categories: ["hermetic"]` while stating a general restriction
    /// or narrative condition (Restriction, Necessary Condition), which
    /// remains importable.
    ///
    /// **Narrower than [`Self::trained`] on purpose, and NOT a reuse of it.**
    /// All four candidate entries are `trained: true` (D12's classification
    /// pass), so `Trained` cannot tell them apart — D68.4 amends the earlier
    /// D23/D33 plan to reuse [`ItemPredicate::Trained`] here and introduces
    /// [`ItemPredicate::RequiresHermeticArts`], read by this field alone.
    ///
    /// `flaw.flawed_powers`'s "only appropriate to Hermetic Magic... cannot
    /// be taken with this Flaw" (ArMDE:6148, Q-138) is this predicate's only
    /// consumer today, via [`ParameterDef::exclude_if`] on the imported-Flaw
    /// parameter.
    #[serde(default, skip_serializing_if = "is_false")]
    pub requires_hermetic_arts: bool,
    /// Entity kinds this item may be selected for. Empty means any kind.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub entity_kinds: BTreeSet<EntityKind>,
    /// Prerequisite expression that must hold for this item to be legal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerequisites: Option<Prereq>,
    /// A HEDGED prerequisite — the rulebook's own "generally", "normally",
    /// "should" rather than "may not"/"only" (F-550/D16/Q-115). Evaluated by
    /// the same [`Prereq`] tri-state machinery as `prerequisites`, but a
    /// `Tri::False` reports the non-blocking `advisory_prereq_not_met` warning
    /// (`validation/prereq.rs::validate_prerequisites`) instead of the hard
    /// `prereq_not_met` error; a `Tri::Unknown` stays silent (unlike the hard
    /// tree's `prereq_unevaluated`), since a hedge that cannot yet be resolved
    /// is not something D16 asks the engine to nag about.
    ///
    /// A sibling field, not a wrapper `Prereq` variant: the hard and advisory
    /// trees are evaluated independently, so no new fold semantics are needed
    /// for what a `Prereq::All`/`Any`/`Nor` containing a "soft" child would
    /// even mean. It is fully additive over the wire — every existing entry's
    /// JSON is unchanged, since this defaults to absent — and the UI needs no
    /// parity guard the way a new `Prereq` variant would
    /// (`ui/src/lib/prereq-parity.test.ts`), because `Prereq` itself did not
    /// change.
    ///
    /// First carrier: `flaw.vendetta` (ArMDE:6957, "generally restricted to
    /// magi of House Verditius").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisory_prerequisites: Option<Prereq>,
    /// Items that may not be selected alongside this one (must be symmetric).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub incompatible_with: BTreeSet<Id>,
    /// D68.8 (`docs/vf-audit/decisions.md`): exempts this item from
    /// `ruleset::integrity::validate_magnitude_variant_exclusivity`, the
    /// load-time guard that otherwise FORCES every detected `_major`/`_minor`
    /// (or `major_`/`minor_`) sibling pair to declare each other in
    /// [`Self::incompatible_with`] or refuses to load at all.
    ///
    /// D68.8's ruling is that each twin exclusion needs its OWN passage —
    /// ArMDE:2814 is not a blanket source — and a pair without one loses its
    /// exclusion. Since the guard cannot tell "no passage" from "an authoring
    /// slip" on its own, the exemption is carried in data: set `true` on
    /// BOTH sides of a pair the guard must stop forcing (the 26 personality-
    /// Flaw pairs, plus Potent Magic and Beloved Rival — the latter's hard
    /// block also becomes [`Self::advisory_prerequisites`], D16). Left
    /// `false` (the default) for the one pair the ruling keeps sourced
    /// (Magical Focus, ArMDE:4405) and the four D44-entailed pairs (Outsider,
    /// True Love, Amorphous, Magian Lineage), which stay hard-blocked.
    #[serde(default, skip_serializing_if = "is_false")]
    pub skip_magnitude_variant_guard: bool,
    /// A per-VALUE extension of [`Self::incompatible_with`] (X6a/e7): each
    /// entry's `forbids` applies only while its own `gate` holds for the
    /// selection — Warped Senses' sight-only clause forbids Keen Vision only
    /// when `sense` names sight, never for the hearing/smell/touch/taste
    /// branches (D58, ArMDE:7029-7037: the incompatibility is absolute even
    /// though the -2 penalty itself stays text, D61). Consumed by
    /// `validation::prereq::validate_incompatibilities` as one more
    /// forbidden-id source per selection, active only when the gate holds —
    /// reuses the existing [`crate::validation::ValidationIssue::CODE_INCOMPATIBLE`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditional_incompatible_with: Vec<ConditionalIncompatibility>,
    /// This item is illegal while ANY OTHER effective (bought or granted)
    /// selection satisfies one of these predicates — D23/B3,
    /// `flaw.university_dean`: "can not have the Poor Flaw or any other Flaw
    /// that grants a Bad Reputation" names [`ItemPredicate::GrantsReputation`]
    /// (`Poor` itself is a plain [`Self::incompatible_with`] id; it does not
    /// "grant a Reputation", it is the OTHER named exclusion).
    ///
    /// **One-directional**, unlike [`Self::incompatible_with`]: the 16
    /// Reputation-granting Flaws need no reciprocal declaration back onto
    /// themselves. Grant-aware on both sides, exactly like
    /// `validation::selections::validate_category_effect_prohibitions` (D2,
    /// closing the F-466 reachability trap
    /// [`Self::incompatible_with`]/`validate_forbidden_categories` are
    /// deliberately bought-only about, B15) — whether THIS item is itself in
    /// effect, and whether the OTHER item satisfying the predicate is present
    /// at all, are each read bought-or-granted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_if_holds: Vec<ItemPredicate>,
    /// D69.6 (`docs/vf-audit/decisions.md`): this item may not be held
    /// alongside another selection that resolves to the SAME target as this
    /// one — "You may not take Student of (Realm) and Puissant Ability for
    /// the same Lore" (ArMDE:5054), "incompatible with... Puissant Artes
    /// Liberales" (ArMDE:3364). See [`SameChoiceExclusion`] for how the two
    /// sides resolve.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub same_choice_exclusions: Vec<SameChoiceExclusion>,
    /// Parameter slots a selection of this item must fill.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ParameterDef>,
    /// Groups of this item's own parameter keys that together name ONE
    /// unordered "combination" for duplicate-detection purposes (D81.8,
    /// Incompatible Arts: `[["technique_1","form_1"],["technique_2","form_2"]]`
    /// — the Flaw's two Technique+Form pairs are interchangeable, so a copy
    /// naming the same two pairs with the groups swapped is the same copy
    /// restated, not a second distinct target). Empty for every item except
    /// Incompatible Arts today.
    ///
    /// Consumed by `validation/selections.rs::validate_duplicate_selections`,
    /// which canonicalizes each named group's values into an order-independent
    /// tuple before building its `(item_ref, params)` duplicate key — the
    /// plain key is order-SENSITIVE on which slot a value sits in, so without
    /// this it cannot tell "pair A in slot 1, pair B in slot 2" from "pair B
    /// in slot 1, pair A in slot 2".
    ///
    /// Load-time integrity requires every named key to be one of this item's
    /// own declared [`Self::parameters`]
    /// (`ruleset/integrity.rs::validate_unordered_param_groups`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unordered_param_groups: Vec<Vec<String>>,
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
    /// RULES.md, "Selection multiplicity — `max_total`").
    /// Distinct from `max_per_target`, which caps copies
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
    /// Source: ArMDE:3665 and :3669
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

/// The default per-parameter-value ceiling: once. `ArMDE:2814` — "A Virtue or
/// Flaw may be taken more than once only if the description explicitly allows
/// it. Most Virtues and Flaws may only be taken once." — is a statement about
/// the ABSENT case, so the model's default must be once, not "no stated
/// limit" (D10). An item whose descriptor allows repeating a value must
/// declare so explicitly, with the `u8::MAX` sentinel documented on
/// [`ParameterDef::max_per_value`], which it shares with [`PointItem::max_total`].
pub(crate) fn default_max_per_value() -> u8 {
    1
}

pub(crate) fn is_default_max_per_value(value: &u8) -> bool {
    *value == default_max_per_value()
}

/// The default for [`ParameterDef::required`]: a declared parameter is
/// required unless the item explicitly opts out (D80 — Fida'i/Lasiq's "cover
/// social status" is the one documented exception).
pub(crate) fn default_required() -> bool {
    true
}

pub(crate) fn is_default_required(value: &bool) -> bool {
    *value == default_required()
}

/// `skip_serializing_if` for [`CharacteristicDeltaCap`] — every entry shipped
/// before this field existed is `AboveBase`, so this keeps every one of them
/// byte-identical.
pub(crate) fn is_default_characteristic_delta_cap(cap: &CharacteristicDeltaCap) -> bool {
    *cap == CharacteristicDeltaCap::AboveBase
}

/// `skip_serializing_if` for [`LabTotalModScope`] — every entry shipped before
/// this field existed is `InPlayGrid`, so this keeps every one of them
/// byte-identical.
pub(crate) fn is_default_lab_total_mod_scope(scope: &LabTotalModScope) -> bool {
    *scope == LabTotalModScope::InPlayGrid
}

/// The default selection multiplicity: an item may be taken once per target.
fn default_max_per_target() -> u8 {
    1
}

fn is_default_max_per_target(value: &u8) -> bool {
    *value == default_max_per_target()
}

/// The default total-selection ceiling: once. `ArMDE:2814` — "A Virtue or
/// Flaw may be taken more than once only if the description explicitly allows
/// it. Most Virtues and Flaws may only be taken once." — so absence must mean
/// once, not "no stated limit" (D10 — the model's default used to be exactly
/// inverted from the book's). An item whose descriptor allows repeating
/// across targets must declare so explicitly, with the `u8::MAX` sentinel
/// documented on this field's own doc comment and RULES.md, "Selection
/// multiplicity — `max_total`".
fn default_max_total() -> u8 {
    1
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
    #[serde(default)]
    index_categories: Vec<String>,
    classification: Classification,
    #[serde(default)]
    tainted: bool,
    #[serde(default)]
    realm_association: Option<RealmAssociation>,
    #[serde(default)]
    faerie_related: bool,
    #[serde(default)]
    trained: bool,
    #[serde(default)]
    requires_hermetic_arts: bool,
    #[serde(default)]
    entity_kinds: BTreeSet<EntityKind>,
    #[serde(default)]
    prerequisites: Option<Prereq>,
    #[serde(default)]
    advisory_prerequisites: Option<Prereq>,
    #[serde(default)]
    incompatible_with: BTreeSet<Id>,
    #[serde(default)]
    skip_magnitude_variant_guard: bool,
    #[serde(default)]
    conditional_incompatible_with: Vec<ConditionalIncompatibility>,
    #[serde(default)]
    excluded_if_holds: Vec<ItemPredicate>,
    #[serde(default)]
    same_choice_exclusions: Vec<SameChoiceExclusion>,
    #[serde(default)]
    parameters: Vec<ParameterDef>,
    #[serde(default)]
    unordered_param_groups: Vec<Vec<String>>,
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
            index_categories,
            classification,
            tainted,
            realm_association,
            faerie_related,
            trained,
            requires_hermetic_arts,
            entity_kinds,
            prerequisites,
            advisory_prerequisites,
            incompatible_with,
            skip_magnitude_variant_guard,
            conditional_incompatible_with,
            excluded_if_holds,
            same_choice_exclusions,
            parameters,
            unordered_param_groups,
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
            index_categories,
            classification,
            tainted,
            realm_association,
            faerie_related,
            trained,
            requires_hermetic_arts,
            entity_kinds,
            prerequisites,
            advisory_prerequisites,
            incompatible_with,
            skip_magnitude_variant_guard,
            conditional_incompatible_with,
            excluded_if_holds,
            same_choice_exclusions,
            parameters,
            unordered_param_groups,
            effects,
            max_per_target,
            max_total,
            max_share_of_kind,
            source,
        })
    }
}

impl PointItem {
    /// Sorts the `parameters` vector by key, and `index_categories` by slug,
    /// for canonical serialization.
    ///
    /// `categories` is order-significant (the descriptor's own order) and so is
    /// deliberately left untouched. `index_categories` is not: an index has no
    /// authored emphasis to preserve, so it sorts like any other unordered list.
    pub fn normalize(&mut self) {
        self.parameters.sort_by(|a, b| a.key.cmp(&b.key));
        self.index_categories.sort();
        // Each group is itself unordered (D81.8/Q3), so both its own keys and
        // the outer list of groups sort canonically — zero-noise git diffs.
        for group in &mut self.unordered_param_groups {
            group.sort();
        }
        self.unordered_param_groups.sort();
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

    /// The category or categories "in force" for a selection of this item —
    /// the single resolution every taken-as-aware membership test calls,
    /// so the answer cannot drift between call sites (the risk row 19's plan
    /// names as the largest correctness hazard in this phase).
    ///
    /// If `params` records a value for one of this item's [`ParameterDomain::Category`]
    /// parameters (Sufi's `taken_as`), only THAT chosen category is in force —
    /// `ArMDE:5083` is an explicit "either/or" choice between two readings of one
    /// item, not membership in both at once. Otherwise every category the
    /// descriptor lists is in force, exactly as before `taken_as` existed.
    ///
    /// A recorded value that is not one of this item's own categories cannot
    /// happen through the picker (load-time integrity requires the param's
    /// `values` to be a subset of `categories`, and `param_value_resolves`
    /// rejects anything outside `values`), but a hand-edited save could still
    /// carry a stale one. Falling back to the whole list in that case — rather
    /// than resolving to nothing — is deliberate: the value already fails
    /// `unknown_param_value` on its own, and a membership test silently seeing
    /// "no categories" would UNDER-count (e.g. wrongly clearing a forbidden or
    /// capped category) rather than over-count.
    ///
    /// Callers: `validate_permitted_categories` / `validate_forbidden_categories`
    /// (`validation/selections.rs`), the category caps (`validation/caps.rs`),
    /// Gift detection (`effective/gift_confidence.rs`), and grant-constraint
    /// filtering (`grant.rs`). Deliberately NOT called by `items_by_category`
    /// (`ruleset/accessors.rs`) or any UI browsing surface — those have no
    /// selection to narrow against.
    pub(crate) fn categories_for(
        &self,
        params: &BTreeMap<String, SelectionParamValue>,
    ) -> &[String] {
        for param in &self.parameters {
            if param.domain != ParameterDomain::Category {
                continue;
            }
            let Some(value) = params
                .get(&param.key)
                .and_then(SelectionParamValue::as_single)
            else {
                continue;
            };
            if let Some(pos) = self
                .categories
                .iter()
                .position(|c| c.as_str() == value.as_str())
            {
                return &self.categories[pos..=pos];
            }
        }
        &self.categories
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
    /// Source: ArMDE:2638 ("you may
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
    /// Source: ArMDE:2774 ("A central
    /// character may have up to ten points of Flaws, but no more than five Minor
    /// Flaws"); grogs :1009 ("no more than three Minor Flaws").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_minor_flaws: Option<u8>,
    /// Per-category flaw count caps (e.g. Personality, Story). Each entry names
    /// the flaw category it applies to as DATA, so the engine never hardcodes a
    /// category slug. `major_only` restricts the count to Major-magnitude flaws;
    /// `hard` makes the cap a blocking error (otherwise a non-blocking warning).
    ///
    /// Source: ArMDE:2820 ("A
    /// character may not have more than one Major Personality Flaw"; "A
    /// character should normally not have more than two Personality Flaws in
    /// total"); :2818 ("A character should not have more than one Story Flaw").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flaw_category_caps: Vec<CategoryCap>,
    /// Per-category *virtue* count caps. Structurally identical to
    /// `flaw_category_caps` but counts `Virtue`-kind items. The magus type uses
    /// this for the `≤1 Major Hermetic Virtue` rule.
    ///
    /// Source: ArMDE:2855-2861 ("You
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
    /// Minimum required count (a floor, additive to the existing ceiling-only
    /// `max`) — D21/F-427, ArMDE:2816: "All characters must take one Social
    /// Status". `None` (the default, and every cap shipped before D41) states
    /// no floor. B2 (D41) is this mechanism's first data user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<u8>,
    /// If true, falling below `min` is a blocking error; otherwise a
    /// non-blocking warning. Independent of `hard`, which governs the
    /// ceiling alone — D41 needs floor=hard, ceiling=soft on the SAME row
    /// (Social Status: mandatory, but a second one is only ever a warning).
    /// Meaningless with `min` absent (load-time integrity rejects that
    /// combination).
    #[serde(default, skip_serializing_if = "is_false")]
    pub min_hard: bool,
    /// Counts BOTH Virtues and Flaws sharing this category, regardless of
    /// which array (`virtue_category_caps`/`flaw_category_caps`) this row
    /// lives in — B2/ArMDE:2816: "All characters must take one Social
    /// Status", and the book's own Social Status entries are a mix of
    /// Virtues (`virtue.gentleman`) and Flaws (`flaw.outlaw`), so a
    /// Virtue-only or Flaw-only count would wrongly refuse a character
    /// legitimately represented by the other kind. `false` (the default, and
    /// every cap shipped before D41) preserves the existing convention: the
    /// surrounding array alone determines which kind counts (a Major
    /// Hermetic FLAW like `flaw.suppressed_gift` must NOT count toward the
    /// Major Hermetic VIRTUE cap, which stays kind-scoped).
    #[serde(default, skip_serializing_if = "is_false")]
    pub both_kinds: bool,
}

/// `skip_serializing_if` predicate: omits a `bool` field from canonical JSON
/// when it holds its `false` default, keeping the common case out of the data.
pub(crate) fn is_false(b: &bool) -> bool {
    !*b
}

/// One entry on an [`EntityTypeProfile`]'s `permitted_categories` or
/// `forbidden_categories` list: either a bare category slug, or a slug guarded
/// by a `when` prerequisite.
///
/// **An entry is in force iff `when` is absent, or `when` evaluates to
/// [`Tri::True`](crate::validation) against the entity.** Both `False` and
/// `Unknown` leave it out of force: `Unknown` resolves in the player's favour,
/// matching the existing non-blocking `prereq_unevaluated` model, where an
/// answer that hinges on data the entity does not carry yet never blocks.
/// The resolution lives in exactly one place — `categories_in_force`
/// (`validation/selections.rs`) — so the permitted and forbidden gates cannot
/// disagree about which entries apply.
///
/// The rule this exists for is
/// ArMDE:2840 — "You may not take
/// Hermetic Virtues and Flaws, unless you have The Gift (this would be highly
/// unusual)" — which the companion profile could previously encode only as its
/// unconditional half.
///
/// `#[serde(untagged)]` so a bare slug stays a bare slug in the JSON, both on
/// the way in and on the way out: every pre-existing `rules/` file loads
/// unchanged and the canonical writer re-emits it byte-identically.
///
/// The cost of admitting the object form is that the two fields can no longer be
/// self-canonicalising `BTreeSet`s: [`EntityTypeProfile::normalize`] sorts them
/// explicitly, and `ruleset/integrity.rs` rejects a repeated category, which
/// the set made unrepresentable by construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CategoryRule {
    /// A bare slug — unconditionally in force.
    Always(String),
    /// A slug in force only while `when` holds.
    When {
        /// The category this rule governs.
        category: String,
        /// The condition under which the rule applies.
        when: Prereq,
    },
}

impl CategoryRule {
    /// The category this rule names, whatever its form.
    pub fn category(&self) -> &str {
        match self {
            CategoryRule::Always(category) => category,
            CategoryRule::When { category, .. } => category,
        }
    }

    /// The rule's condition, or `None` for the unconditional form.
    pub fn when(&self) -> Option<&Prereq> {
        match self {
            CategoryRule::Always(_) => None,
            CategoryRule::When { when, .. } => Some(when),
        }
    }
}

/// One entry on an [`EntityTypeProfile`]'s `creation_phases` list: either a bare
/// phase, or a phase guarded by a `when` prerequisite. Mirrors [`CategoryRule`]
/// exactly, for the same reason (D56/A0, `docs/vf-audit/design-a0-is-magus-split.md`
/// § 6) — a phase can apply conditionally on Hermetic training or Order
/// membership (the Arts/Spells phases for a type that may be trained by
/// selection, e.g. the Abandoned Apprentice companion), and this is the one
/// mechanism for "applies conditionally" the engine already has.
///
/// **An entry is in force iff `when` is absent, or `when` evaluates to
/// [`Tri::True`](crate::validation) against the entity.** Both `False` and
/// `Unknown` leave it out of force — the same convention `CategoryRule`
/// established. The resolution lives in exactly one place — `phases_in_force`
/// (`validation/selections.rs`).
///
/// `#[serde(untagged)]` so a bare phase slug stays a bare slug in the JSON, both
/// on the way in and on the way out: every pre-existing `rules/` file loads
/// unchanged and the canonical writer re-emits it byte-identically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PhaseRule {
    /// A bare phase — unconditionally in force.
    Always(CreationPhase),
    /// A phase in force only while `when` holds.
    When {
        /// The phase this rule governs.
        phase: CreationPhase,
        /// The condition under which the rule applies.
        when: Prereq,
    },
}

impl PhaseRule {
    /// The phase this rule names, whatever its form.
    pub fn phase(&self) -> CreationPhase {
        match self {
            PhaseRule::Always(phase) => *phase,
            PhaseRule::When { phase, .. } => *phase,
        }
    }

    /// The rule's condition, or `None` for the unconditional form.
    pub fn when(&self) -> Option<&Prereq> {
        match self {
            PhaseRule::Always(_) => None,
            PhaseRule::When { when, .. } => Some(when),
        }
    }
}

/// Data-driven profile defining constraints for an entity type
/// (grog, companion, magus, etc.).
///
/// As with [`Entity`], the character-only fields here (`hermetically_trained`,
/// `order_member`, `gift_policy`, `gift_id`, `gift_categories`) live flat on this generic
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
    /// Each entry is a [`CategoryRule`] — a bare slug, or a slug conditional on
    /// a `when` prerequisite.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permitted_categories: Vec<CategoryRule>,
    /// Items whose `category` is listed may never be selected. Conditional in
    /// the same way as `permitted_categories`, and note the two must be kept in
    /// step: permitting is ANY and forbidding is EVERY, so relaxing a forbid
    /// alone leaves a single-category item refused with `category_not_permitted`
    /// instead.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_categories: Vec<CategoryRule>,
    /// Item ids that must be selected.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub required_traits: BTreeSet<Id>,
    /// Item ids that may never be selected.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub forbidden_traits: BTreeSet<Id>,
    /// Whether this character type is Hermetically trained (has the skills and
    /// knowledge of a fully trained magus), independent of Order membership —
    /// D56/A0 splits the old single `is_magus` flag into this and
    /// [`Self::order_member`] because a selection can confer training without
    /// the type profile itself declaring it (the Abandoned Apprentice Flaw,
    /// ArMDE:5641-5650): see
    /// `effective/hermetic_training.rs::is_hermetically_trained`, the single
    /// fact every "trained" production site must read instead of this field
    /// alone (the profile-only sites are the documented exception — D56).
    /// Independent of `gift_policy`: an unGifted Redcap is a companion (not
    /// Hermetically trained) and a Gifted hedge wizard has The Gift but is not
    /// Hermetically trained. Defaults to false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hermetically_trained: bool,
    /// Whether this character type is a full member of the Order of Hermes.
    /// The other half of the old `is_magus` flag (D56/A0): unlike
    /// [`Self::hermetically_trained`], no entity-level override exists or is
    /// asked for today — Houses (`validation/magus.rs::validate_house`) are
    /// the only consequence this gates, and every House-bearing type also sets
    /// [`Self::hermetically_trained`], so the two agree for every profile that
    /// ships today. `virtue.redcap`/`virtue.lone_redcap` (ArMDE:4842-4851,
    /// 4319-4326) are a recorded open risk — an Order member with no Hermetic
    /// training — re-examined when either is encoded as a mechanic (D2/X5); see
    /// `docs/vf-audit/design-a0-is-magus-split.md` § 1. Defaults to false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub order_member: bool,
    /// Whether this character type counts as a companion for the audience
    /// `Prereq::IsCompanion` reads (D38): true for the plain `companion`
    /// profile and for `mythic_companion` — "mythic companions are companions
    /// too" — so a future companion-like profile joins the audience by
    /// setting this flag alone, with no change to any item's prerequisites.
    /// A capability flag parallel to `has_mythic_type`; never a hardcoded
    /// type id. Defaults to false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_companion: bool,
    /// Whether this character type counts as a grog for the audience
    /// `Prereq::IsGrog` reads (D68.9): true only for the plain `grog` profile
    /// today. A capability flag parallel to `is_companion`, never a hardcoded
    /// type id. Defaults to false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_grog: bool,
    /// Whether this character type chooses a Mythic Companion *type* (which
    /// confers a free status/Minor Virtue and a required V/F package). A
    /// capability flag parallel to `is_magus`; the type selector and
    /// `validate_mythic_type` read it, never a hardcoded type id. Defaults to
    /// false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_mythic_type: bool,
    /// The magus's starting spell-levels budget (the sum of the levels of spells
    /// he may know at creation). 120 for the magus profile (ArMDE:2215-2216,
    /// 2435); 0 (omitted) for every non-magus type, which cannot take spells.
    /// Modified per-character by [`Effect::SpellLevels`] (Skilled/Weak Parens).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub spell_levels: u32,
    /// The character type's starting Confidence Score
    /// (ArMDE:2524 — the earlier
    /// citation of this line pointed at the "### Confidence" heading two lines
    /// above the actual "start with a Confidence Score of 1 and 3 Confidence
    /// Points" sentence; corrected against the source). Companions,
    /// magi and mythic companions start at 1; grogs have no Confidence (0/omitted).
    /// The effective score folds in `Effect::ConfidenceBonus`; Confidence is
    /// derived, never stored on the entity.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub confidence_score: u8,
    /// The character type's starting Confidence Points
    /// (ArMDE:2524): 3 for
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
    /// Categories whose Flaws satisfy this type's "at least one Hermetic Flaw"
    /// guideline. `["hermetic"]` on the magus profile; empty everywhere else,
    /// which switches the guideline off entirely.
    ///
    /// > You should take at least one Hermetic Flaw
    ///
    /// Source: ArMDE:2860 (a bullet
    /// under `#### Magi`, `ArMDE:2853`).
    ///
    /// Deliberately **not** [`Self::gift_categories`], though both name
    /// `hermetic` today. They answer different questions — "is this character
    /// Gifted?" versus "does this Flaw count as a Hermetic Flaw?" — and the two
    /// Beings Flaws prove the answers can differ: the book indexes them under
    /// Hermetic (so the guideline counts them) while the engine must not read
    /// them as Gift-bearing (so they carry `general`, with the index heading in
    /// [`PointItem::index_categories`]). One field serving both meant fixing
    /// Gift detection silently broke the guideline.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub hermetic_flaw_categories: BTreeSet<String>,
    /// Ordered creation phases the guided wizard walks through. Each entry is a
    /// [`PhaseRule`] — a bare phase, or a phase conditional on a `when`
    /// prerequisite (D56/A0 § 6) — resolved per-entity by `phases_in_force`.
    /// Typed, so serde itself is the load-time validator: a profile naming a
    /// phase the engine has no [`CreationPhase`] for fails the ruleset load
    /// rather than reaching the wizard as a step it cannot render.
    // Order-significant (the wizard walks them in sequence): intentionally
    // exempt from `normalize`'s canonical sorting.
    pub creation_phases: Vec<PhaseRule>,
}

impl EntityTypeProfile {
    /// Sorts the profile's unordered nested vectors for canonical
    /// serialization. The trait sets are `BTreeSet`s (already id-ordered), and
    /// `creation_phases` is order-significant and so left untouched. The
    /// unordered vectors are the budget's `flaw_category_caps` and
    /// `virtue_category_caps`, and — since [`CategoryRule`] made them `Vec`s
    /// rather than self-ordering `BTreeSet`s — the two category lists, all
    /// sorted here by category. A stable sort is enough to be deterministic
    /// because `ruleset/integrity.rs` rejects a profile that names the same
    /// category twice, so the key is unique.
    pub fn normalize(&mut self) {
        self.budget
            .flaw_category_caps
            .sort_by(|a, b| a.category.cmp(&b.category));
        self.budget
            .virtue_category_caps
            .sort_by(|a, b| a.category.cmp(&b.category));
        self.permitted_categories
            .sort_by(|a, b| a.category().cmp(b.category()));
        self.forbidden_categories
            .sort_by(|a, b| a.category().cmp(b.category()));
    }

    /// Whether `permitted_categories` names this category **at all**, ignoring
    /// any `when` guard. This is a question about what the profile declares, not
    /// about what applies to a given character — for that, the validators
    /// resolve the rules against a `PrereqCtx`.
    pub fn names_permitted_category(&self, category: &str) -> bool {
        self.permitted_categories
            .iter()
            .any(|rule| rule.category() == category)
    }

    /// Whether `forbidden_categories` names this category at all, ignoring any
    /// `when` guard. Mirror of [`Self::names_permitted_category`].
    pub fn names_forbidden_category(&self, category: &str) -> bool {
        self.forbidden_categories
            .iter()
            .any(|rule| rule.category() == category)
    }
}

/// A value bound to one of a [`Selection`]'s parameters: a single `Id` (every
/// parameter type except `multi_ref`) or a set of them (D9 part 3's
/// multi-valued parameter — `docs/vf-audit/design-c0-parameter-model.md` § 8).
///
/// `#[serde(untagged)]` makes this **wire-compatible with the prior bare-`Id`
/// shape**: `Single` serializes/deserializes as exactly the JSON string a
/// `BTreeMap<String, Id>` value used to be, so every existing save round-trips
/// byte-identically and this slice (C0b) owes no `SCHEMA_VERSION` bump — the
/// bump belongs to C5a, which is the slice that actually needs to migrate a
/// pre-existing shape. `Multi` is the value of a `multi_ref` parameter, which
/// shipped rules data declares (e.g. `flaw.corrupted_spells`,
/// `flaw.corrupted_arts`, `flaw.restricted_learning`). C5a added the
/// wrong-shape check (`validation/selections.rs`); no fold is needed, since
/// `BTreeSet` gives `Multi` its canonical form on deserialize (see
/// `migration.rs`'s `SCHEMA_VERSION` doc comment).
///
/// `BTreeSet`, not `Vec`, for the same reason [`ParameterDef::at_most_one_of`]
/// already uses it: a `BTreeSet` makes `{A,B}` and `{B,A}` compare equal by
/// construction, which is what lets a duplicate-target check key on this value
/// directly with no separate canonicalization step.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SelectionParamValue {
    /// One bound `Id` — the value of every parameter type except `multi_ref`.
    Single(Id),
    /// A set of bound `Id`s — the value of a `multi_ref` parameter; see the
    /// enum's doc comment.
    Multi(BTreeSet<Id>),
}

impl SelectionParamValue {
    /// The single `Id`, when this value is [`Self::Single`] — `None` for
    /// [`Self::Multi`]. Every read site that resolves a parameter's value
    /// against a registry, a gate, or an instance discriminator wants exactly
    /// one `Id`; a `Multi` slot simply has none to give (that resolution is
    /// C5b/C5c's job, once a producer exists), so callers treat `None` here
    /// the same way they already treat an absent key.
    pub fn as_single(&self) -> Option<&Id> {
        match self {
            Self::Single(id) => Some(id),
            Self::Multi(_) => None,
        }
    }
}

impl From<Id> for SelectionParamValue {
    fn from(id: Id) -> Self {
        Self::Single(id)
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
    pub params: BTreeMap<String, SelectionParamValue>,
}

impl Selection {
    /// Creates a new selection with no parameters.
    pub fn new(item_ref: Id) -> Self {
        Self {
            item_ref,
            params: BTreeMap::new(),
        }
    }

    /// Creates a new selection with the given parameter values, each a single
    /// `Id` — this engine produces no other shape today (see
    /// [`SelectionParamValue`]'s doc comment). Kept accepting
    /// `BTreeMap<String, Id>` rather than `BTreeMap<String, SelectionParamValue>`
    /// so the ~150 existing call sites across the crate's tests need no
    /// change for this purely additive type widening.
    pub fn with_params(item_ref: Id, params: BTreeMap<String, Id>) -> Self {
        Self {
            item_ref,
            params: params
                .into_iter()
                .map(|(k, v)| (k, SelectionParamValue::Single(v)))
                .collect(),
        }
    }
}

/// A parameterized [`AbilityScore`]'s player-supplied value (D14; see
/// `docs/vf-audit/design-cv-catalogued-values.md` § 3) — three disjoint shapes,
/// discriminated structurally by which keys are present:
///
/// - [`Self::Catalogued`] — a chosen entry from the [`crate::catalogue::Catalogue`]
///   named by the ability's `parameter` key (e.g. `language.latin`). Matches a
///   rules-authored `ParamValue::Literal` **by id**, exactly (design § 4).
/// - [`Self::Linked`] — follows a Virtue/Flaw selection's own parameter live
///   (e.g. "the guild I'm already a member of via Craft Guild Training"),
///   rather than a second, independently-typed copy of the same fact. The type
///   lands in CV4; nothing produces or resolves one until CV5 wires up § 4.1's
///   Bound/Link matching.
/// - [`Self::Text`] — free text: no catalogue, or the player picked "Other…", or
///   a link target vanished/became ambiguous and was converted here.
///
/// `#[serde(untagged)]`, exactly like [`SelectionParamValue`]: the three
/// variants' field-name sets never overlap, so serde discriminates
/// unambiguously.
///
/// **`deny_unknown_fields`** (design § 3.3): a value naming keys from more than
/// one variant at once (e.g. `{"id": …, "item": …, "param": …}`) is not a shape
/// any writer of this format produces — only a hand-edited or adversarial save
/// could contain it. Without this attribute, serde's untagged default would try
/// each variant in order and silently accept the first structural match while
/// dropping the unrecognised extra fields (here, matching `Catalogued { id }`
/// and silently discarding `item`/`param`). With it, every variant's parse
/// attempt fails on the other variants' fields, so the untagged enum as a whole
/// fails with "data did not match any variant" and the whole entity load fails
/// — loud and safe, rather than a quiet misread.
///
/// **Wire-compatibility.** A bare JSON string (every pre-CV4 save) is not a
/// shape any of these three variants accepts — [`crate::migration`]'s raw
/// pre-pass ([`crate::migration::wrap_legacy_ability_parameters`]) rewraps it
/// into `{"text": …}` before the typed parse ever sees it, regardless of the
/// save's claimed `schema_version`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum AbilityParameterValue {
    /// A chosen catalogue entry: matches a `Literal` by id, exactly.
    Catalogued {
        /// The catalogue value's id (e.g. `language.latin`).
        id: Id,
    },
    /// Resolves against the CURRENT value of `item`'s own `param` at read time —
    /// so renaming the guild on the Virtue renames every linked Ability row too.
    /// Not yet produced or resolved anywhere in the engine (CV5).
    Linked {
        /// The declaring selection's item id (e.g. `virtue.craft_guild_training`).
        item: Id,
        /// The declaring item's own parameter key to read at evaluation time.
        param: String,
    },
    /// Free text, exactly like the pre-CV4 bare string.
    Text {
        /// The player-typed value.
        text: String,
    },
}

impl AbilityParameterValue {
    /// Wraps `text` as a [`Self::Text`] value. A small, test-and-call-site
    /// convenience (design § 5.6a) — turns `Some("Latin".to_string())`-shaped
    /// construction into `Some(AbilityParameterValue::text("Latin"))` rather than
    /// a hand-written struct literal at each of the ~25 sites across the crate
    /// that build a free-text parameter value.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// A plain string identity key, for the sites that only need to know
    /// "which instance is this" (grouping bought scores by `(ability,
    /// parameter)`, duplicate detection, "is a parameter present at all",
    /// picking which already-bought instance a caller means) — **never** for
    /// testing whether this value satisfies a rules-authored
    /// `ParamValue::Literal`. That comparison is design § 4 rule 1's job,
    /// implemented in `effective/xp.rs` (`AbilityInstanceRef::satisfied_by`):
    /// a `Literal` is satisfied ONLY by `Catalogued` with a matching id, never
    /// by `Text` holding the identical letters, which is exactly the
    /// distinction collapsing to a bare string here would erase. `Linked` has
    /// no resolver yet (CV5), so it yields no key at all (never guessed).
    pub(crate) fn match_key(&self) -> Option<&str> {
        match self {
            AbilityParameterValue::Catalogued { id } => Some(id.as_str()),
            AbilityParameterValue::Text { text } => Some(text.as_str()),
            AbilityParameterValue::Linked { .. } => None,
        }
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
    pub parameter: Option<AbilityParameterValue>,
    /// XP already banked toward the NEXT score, in raw table-XP currency
    /// (`xp_for_score`'s scale — not charged XP, not [`Entity::xp_pool`]). The
    /// book prints "X (Z)" beside a score (ArMDE:1177-1179): X = [`Self::score`],
    /// Z = this field. X10b. Appended last (not after `score`): every
    /// pre-X10b save has `banked_xp == 0` throughout, so a trailing tie-break
    /// key leaves [`Entity::normalize`]'s sort byte-identical for old saves.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub banked_xp: u32,
}

impl AbilityScore {
    /// A bought ability score with no specialty, no parameter, and no banked
    /// XP — the shape every pre-X10b save had throughout. Set the optional
    /// fields on the returned value for the sites that need them.
    pub fn new(ability: Id, score: u8) -> Self {
        Self {
            ability,
            score,
            specialty: None,
            parameter: None,
            banked_xp: 0,
        }
    }
}

/// A whole bought Hermetic Art score. Arts are not parameterized and carry no
/// specialty, so an instance is identified by `art` alone. Like Abilities, the XP
/// to reach the score is priced from the ruleset's *Art* advancement table (a
/// separate, cheaper curve), and the leftover XP banks against
/// [`Self::banked_xp`], counted from the one shared [`Entity::xp_pool`]. The
/// *effective* score (bought + Puissant Art) is computed at validation time,
/// never stored.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArtScore {
    /// The Art's id (e.g. `art.creo`).
    pub art: Id,
    /// The whole bought score.
    pub score: u8,
    /// XP already banked toward the next score, same currency and printed
    /// form as [`AbilityScore::banked_xp`] (X10b). Appended last for the same
    /// byte-identity reason.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub banked_xp: u32,
}

impl ArtScore {
    /// A bought Art score with no banked XP.
    pub fn new(art: Id, score: u8) -> Self {
        Self {
            art,
            score,
            banked_xp: 0,
        }
    }
}

/// A spell the character knows (magi only). Saves store the choice, not the
/// resolved value: the catalogue supplies a fixed spell's level, so `level` is
/// `Some` only for a **General** spell — the per-character learned level. Two
/// General versions of one spell at different levels are distinct spells
/// (ArMDE:12349-12353), so identity is (spell, level, parameter). Kept
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
    /// at 1. Source: ArMDE:4471-4474,
    /// :3887-3889.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mastery: Option<u8>,
    /// The chosen value for a parameterized spell (the target `(Form)` of a
    /// meta-magic Vim spell like Wizard's Boost — an Art id such as `art.ignem`).
    /// Part of the spell's identity: the same base spell may be taken once per
    /// distinct parameter (ArMDE:15791-15794). Display + identity only —
    /// it does NOT change the spell's own Technique/Form. `None` for ordinary,
    /// unparameterized spells.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// The chosen Spell Mastery special abilities for this spell, each a
    /// `spell_mastery_ability.*` id from the catalogue. One may be chosen per
    /// effective mastery level (ArMDE:9524-9526); a repeatable ability
    /// (Precise/Quick/Quiet Casting) may appear more than once, so this is a
    /// `Vec` that may hold duplicates, not a set. Additive and serde-defaulted, so
    /// it is backward/forward compatible with saves written before it existed
    /// (SCHEMA_VERSION stays 13): an old save omits the key and deserializes to an
    /// empty vector; a new save with an empty vector omits the key on write. Kept
    /// sorted (stable, with duplicates) by [`Entity::normalize`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mastery_abilities: Vec<Id>,
    /// The player's own claim that this spell falls within the character's
    /// Magical Focus (ArMDE:4399-4422). Free-text [`Effect::MagicalFocus`]
    /// can't supply this (MAG8: which Technique/Form cells a free-text focus
    /// covers is table judgement, not something the engine derives), so it is
    /// a recorded per-spell choice. Harmless if the character holds no Focus
    /// at all — see `derived/casting.rs::spell_casting_total`. X10c. Appended
    /// last, same byte-identity reasoning as [`AbilityScore::banked_xp`].
    #[serde(default, skip_serializing_if = "is_false")]
    pub within_focus: bool,
    /// The player's own claim that this spell falls within the character's
    /// Potent Magic field (ArMDE:4740-4748). A free-text `field` parameter
    /// can't supply this either, for the same reason `within_focus` can't
    /// (MAG8's shape, generalised): which Technique/Form cells a free-text
    /// field covers is table judgement. Independent of [`Self::within_focus`]
    /// — a spell may be within a Magical Focus, within a Potent Magic field,
    /// both, or neither, because the two free-text themes need not coincide
    /// (D79, `docs/vf-audit/decisions.md`). Harmless if the character holds no
    /// Potent Magic Virtue at all — see
    /// `derived/casting.rs::spell_casting_total`. Appended last, same
    /// byte-identity reasoning as [`Self::within_focus`].
    #[serde(default, skip_serializing_if = "is_false")]
    pub within_potent_field: bool,
}

impl SpellSelection {
    /// An ordinary, unmastered, unparameterized spell selection (a fixed-level
    /// spell with no mastery, no meta-magic parameter, and not within Focus).
    /// Set the optional fields on the returned value for the sites that need
    /// them.
    pub fn new(spell: Id) -> Self {
        Self {
            spell,
            level: None,
            mastery: None,
            parameter: None,
            mastery_abilities: Vec::new(),
            within_focus: false,
            within_potent_field: false,
        }
    }
}

/// How a piece of equipment is currently carried. A fixed rules taxonomy
/// (CLAUDE.md), replacing the K2-era `EquipmentSlot::equipped: bool`, which
/// conflated two independent facts (K5, `docs/vf-audit/design-f0-book-template-
/// engine.md` § 1): whether the slot yields a Combat row, and whether it
/// contributes Load. Derives matched to [`EquipmentSlot`]'s own stack, since
/// `EquipmentSlot` is kept sorted by [`Entity::normalize`], which needs every
/// field — `loadout` included — to be `Ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoadoutState {
    /// Not currently carried on the character's person: no Combat row, no Load.
    #[default]
    Stowed,
    /// Carried and wieldable, but not currently wielded: yields a Combat row,
    /// contributes no Load (the Knight's carried great sword, K5).
    Carried,
    /// Actively wielded/worn: yields a Combat row AND contributes Load (K2's
    /// existing behavior, unchanged).
    Wielded,
}

impl fmt::Display for LoadoutState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LoadoutState::Stowed => "stowed",
            LoadoutState::Carried => "carried",
            LoadoutState::Wielded => "wielded",
        })
    }
}

/// A piece of equipment the character carries: a reference to a catalogue weapon,
/// shield, or armor id, plus how it is currently carried (K5). Only the choice is
/// stored — combat totals, Soak, and Encumbrance are derived downstream (5i) from
/// the referenced catalogue row. Kept sorted via [`Entity::normalize`]. Source:
/// ArMDE:16944-17011 (the equipment tables), :17103-17123 (Encumbrance, computed
/// in the derived-totals slice).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EquipmentSlot {
    /// The catalogue id of the item (a `weapon.*`, `shield.*`, or `armor.*` id).
    pub item: Id,
    /// How this item is currently carried (K5). [`LoadoutState::Stowed`] is
    /// inert: it yields no combat/Soak line (5i) and adds no Load, so a stowed
    /// spare weapon costs its owner no Encumbrance. [`LoadoutState::Carried`]
    /// yields a Combat row but no Load — the book's own Knight template
    /// (ArMDE:1447-1486) wants exactly that for his alternate great sword, which
    /// one boolean could not express. [`LoadoutState::Wielded`] is K2's existing
    /// behavior (Combat row AND Load), unchanged. See
    /// `derived/combat.rs::encumbrance` for the Load reasoning and the arithmetic.
    #[serde(default, skip_serializing_if = "is_stowed")]
    pub loadout: LoadoutState,
    /// Whether this weapon's combat Ability specialization applies to it, granting
    /// +1 to the weapon's Attack and Defense (the specialty must be aligned to this
    /// specific weapon; Damage/Initiative do not use the Ability). Only meaningful
    /// for a weapon slot whose Ability carries a specialty. Additive and
    /// serde-defaulted — old saves omit the key and deserialize to `false`, a new
    /// save with `false` omits it on write (SCHEMA_VERSION unchanged), exactly like
    /// the sibling `loadout` field. Source: ArMDE:7122, :7139.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub specialization_applies: bool,
}

/// `skip_serializing_if` predicate: omits `loadout` from canonical JSON when it
/// holds its `Stowed` default, keeping the common case out of the data.
fn is_stowed(loadout: &LoadoutState) -> bool {
    *loadout == LoadoutState::Stowed
}

/// A named Personality Trait with a value in −3..+3 (or ±6 for the trait
/// representing a Major Personality Flaw). Free-text name, kept sorted by name in
/// [`Entity::normalize`]. Source: ArMDE:2500-2503.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PersonalityTrait {
    /// Free-text trait name (e.g. "Brave", "Loyal").
    pub name: String,
    /// Trait value; ±3 normally, ±6 for a Major Personality Flaw's trait.
    pub value: i8,
}

/// A starting Reputation: score + free-text content + audience type. Only legal
/// when backed by a granting Virtue/Flaw ([`Effect::GrantsReputation`]).
/// Source: ArMDE:1091-1101, 2512-2514.
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
/// Source: ArMDE:4347-4349 (Magic
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
/// Source: RoP:M:1470-1472; ArMDE:2623-2631.
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

impl Realm {
    /// All four Realms, in canonical order.
    pub const ALL: [Realm; 4] = [Realm::Magic, Realm::Faerie, Realm::Divine, Realm::Infernal];

    /// The slug-style id for this Realm, e.g. `realm.divine`.
    pub fn id(self) -> Id {
        Id::new(format!("realm.{self}"))
    }

    /// Parses a `realm.<slug>` [`Id`] back into a Realm, or `None` if it is not
    /// a valid realm id. The exact mirror of
    /// [`crate::characteristics::Characteristic::from_id`], and the resolution
    /// [`ParameterDomain::Realm`] uses: the Realms are a closed engine taxonomy,
    /// so the "registry" a realm-domain parameter resolves against is this enum
    /// rather than any catalogue.
    pub fn from_id(id: &Id) -> Option<Self> {
        let slug = id.as_str().strip_prefix("realm.")?;
        Self::ALL
            .into_iter()
            .find(|realm| realm.to_string() == slug)
    }
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

/// The selection param key an override is stored under. Deliberately not
/// `"realm"`, which `flaw.bound_to_realm`, `flaw.realm_stigmatic`,
/// `flaw.necessary_realm_aura_for_ability` and `virtue.folk_magic` already use
/// for a different (or, for Folk Magic, the same) realm-shaped value.
///
/// Lives here in `types`, beside [`stamp_realm_override`], rather than in
/// `effective::realm` where its only other reader (`override_realm`) lives:
/// `crate::grant::resolve_grant` needs to call [`stamp_realm_override`] too,
/// and `types` is the one layer both `grant` and `effective` already sit
/// above, so defining it here keeps neither depending on the other. Stays
/// `pub` (re-exported at `effective::REALM_OVERRIDE_PARAM_KEY`, its established
/// path): `tests/d42_realms.rs` reads it from outside the crate, unlike
/// [`stamp_realm_override`], which no external test calls.
pub const REALM_OVERRIDE_PARAM_KEY: &str = "association";

/// Stamps `realm`, if stated, onto `selection`'s [`REALM_OVERRIDE_PARAM_KEY`]
/// param — row 55 (`docs/open-todos.md`; D74.4,
/// `docs/vf-audit/decisions.md`): a granted copy of a Supernatural entry
/// carries a realm only where the grant states one
/// (`Effect::GrantsSelection::realm`, `crate::grant::Grant::Fixed::realm`),
/// independent of the granted item's own `realm_association`. A no-op when
/// `realm` is `None`, so a grant that states nothing leaves the granted
/// [`Selection`] exactly as [`Selection::new`]/[`Selection::with_params`]
/// built it, resolving through the plain chain exactly like a bought copy.
///
/// The single call site for every grant mechanism that can carry a per-grant
/// realm (`crate::grant::resolve_grant`,
/// `crate::effective::vf_granted_selections`) so
/// [`crate::effective::resolve_realm`] itself needs no change: it already
/// reads [`REALM_OVERRIDE_PARAM_KEY`] off any [`Selection`], bought or
/// granted, through the exact same `override_realm` step.
pub(crate) fn stamp_realm_override(selection: &mut Selection, realm: Option<Realm>) {
    if let Some(realm) = realm {
        selection.params.insert(
            REALM_OVERRIDE_PARAM_KEY.to_string(),
            SelectionParamValue::Single(realm.id()),
        );
    }
}

/// How a Supernatural [`PointItem`]'s realm association (ArMDE:2960, "All
/// Supernatural Virtues and Flaws are associated with one of the four
/// realms") resolves, for the ~37 entries where the book says more than
/// nothing (D42/D70/D74, `docs/vf-audit/decisions.md`). Absent (`None`) for
/// every other Supernatural entry: it resolves through the plain chain
/// (override, else [`Entity::concept_realm`], else [`Realm::Magic`]) that
/// [`crate::effective::resolve_realm`] implements, with no warning ever.
///
/// A `tainted: true` item (ArMDE:3000) resolves Infernal by that rule alone,
/// checked ahead of this field — so only the two entries the book fixes
/// Infernal by cross-entry reference rather than by their own Tainted tag
/// (Demonic Might/Powers, ArMDE:3661) need [`Self::Fixed`] data for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RealmAssociation {
    /// The book states this entry's realm outright (Faerie Blood, Strong
    /// Faerie Blood, Kassalan Exorcism, Strong Angelic Heritage, Blood of the
    /// Nephilim, Viaticarus, Bee King, Commanding Aura, Raised from the
    /// Dead). Shown read-only in the UI; an override present in data (e.g. a
    /// pre-ruling save) is silently ignored by the resolver rather than
    /// rejected.
    Fixed {
        /// The fixed Realm the book states for this entry.
        realm: Realm,
    },
    /// The book states a default the concept does not override (Spiritual
    /// Pact, Warped by Magic, Hex, Sufi, Cursed Guile). An override is legal
    /// but raises the `realm_changed_default` warning when it differs.
    Default {
        /// The Realm that applies unless the concept overrides it.
        realm: Realm,
    },
    /// The book restricts this entry to a named subset of realms (Manifest
    /// Sin: Divine/Infernal). No override and no concept realm inside the
    /// subset raises `realm_unset_subset` rather than silently falling back
    /// to Magic, which would be illegal here. An override outside the subset
    /// is a validator error (`realm_override_invalid`), not resolved here.
    Subset {
        /// The Realms this entry is restricted to.
        realms: BTreeSet<Realm>,
    },
    /// This entry already declares a named-realm parameter of its own (Bound
    /// to (Realm)'s `realm` key, (Realm) Stigmatic's, Necessary (Realm) Aura
    /// for (Ability)'s) that supplies the default; no warning either way.
    FromParam {
        /// The parameter key supplying the realm (e.g. `"realm"`).
        key: String,
    },
}

/// A supernatural being's **Might Score** and the Realm it is aligned to. A Might
/// Score grants blanket Magic Resistance equal to the score (RoP:M:1472). Only the choice is stored; the effective
/// score and its Magic Resistance are derived. Source: RoP:M:1470-1472.
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
/// Kept sorted via [`Entity::normalize`]. Source: RoP:I:4122; RoP:D:1977.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SupernaturalPower {
    /// Free-text power name.
    pub name: String,
    /// Total power level, charged against the power-levels budget.
    pub level: u16,
    /// Levels spent one-for-one to give the power Penetration; 0 when none were.
    ///
    /// "You may also spend levels one-for-one to give the power Penetration;
    /// otherwise, it has a Penetration of zero" — so these levels come out of the
    /// SAME pool as [`Self::level`] and are charged against the same budget by
    /// [`crate::effective::powers_used`]. The book's own worked example (`ArMDE:4021`)
    /// spends two copies of Greater Power, 100 levels, as "a power with a level of
    /// 60 and a Penetration of 0, and a second power with a level and Penetration
    /// of 20 each": 60 + 0 + 20 + 20 = 100.
    ///
    /// Additive, so [`SCHEMA_VERSION`] is unchanged and no migration exists: a save
    /// written before the field reads 0, and a power that bought no Penetration
    /// serializes exactly as it did before.
    ///
    /// Source: ArMDE:4019, :4021.
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub penetration: u16,
}

/// A Focus Power the character may exercise at will, bought out of the Focus
/// Power Virtue's point pool.
///
/// Deliberately **not** a [`SupernaturalPower`], because the two store different
/// quantities in different currencies. A `SupernaturalPower`'s `level` is the
/// level of a power that was *made*, spent one-for-one out of the level budget
/// ([`crate::effective::powers_used`]). A Focus Power stores a **ceiling**: "the
/// maximum level of effect", raised at **2 points each**, with "the character may
/// create any effect within the scope of the power, up to the level of the
/// effect" (`ArMDE:3899`). Keeping them apart makes it structurally impossible
/// for a focus power to be charged against the level budget — the confusion B10
/// avoided by leaving Focus Power out of [`Effect::PowerLevels`] entirely.
///
/// The Virtue's scope is a free-text descriptor, "like a magical focus … related
/// to a specialty that is smaller than a single Hermetic Form" (`ArMDE:3897`), so
/// [`Self::name`] carries it exactly as [`SupernaturalPower::name`] does.
///
/// Kept sorted via [`Entity::normalize`]. Source: `ArMDE:3895-3903`.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FocusPower {
    /// Free-text name / scope of the focus power.
    pub name: String,
    /// The maximum level of effect the character may create, at **2 points** per
    /// level out of the Focus Power pool. Source: `ArMDE:3899`.
    pub max_level: u16,
    /// The power's Penetration, at **1 point** per point out of the same pool;
    /// both "start at zero". Source: `ArMDE:3899`.
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub penetration: u16,
}

/// A magus's familiar: the magical beast itself plus the three bond cords.
///
/// "A familiar is a beast that a magus befriends and then magically bonds with,
/// instilling the beast with magical powers in the process" — it "always has its
/// own will, and is not under the control of the magus"
/// (ArMDE:10766-10892). It is
/// therefore a *creature*, and the fields below
/// follow the rulebook's own **Creature Format** order (`ArMDE:17787-17827`) so a save
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
/// (`ArMDE:17793`), and its Might is not the magus's Might, so neither the point-buy nor
/// the Might realm-agreement check can ever see them (both invariants are locked by
/// regression tests in `validation/mod.rs`). Nothing is derived here; the read-outs
/// live in [`crate::derived::familiar_readout`].
///
/// Every field beyond `name` is additive `serde(default, skip_serializing_if)`, so
/// a pre-M5.5c familiar (name + cords) loads unchanged and writes identical bytes —
/// hence no `SCHEMA_VERSION` bump.
///
/// Source: `ArMDE:10766-10892` (Familiars),
/// `ArMDE:10840-10844` (the three cords), `ArMDE:17787-17827` (Creature Format).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Familiar {
    /// Free-text familiar name.
    pub name: String,
    /// The kind of beast, free text ("raven", "tortoiseshell cat"). Deliberately
    /// **not** `species`: the rules reserve *Species* for the Imaginem term
    /// (the sensory image a thing sheds), so the field is named for the animal.
    /// Source: `ArMDE:10774` ("finding an animal with inherent magic").
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub animal: String,
    /// The familiar's own Magic Might Score + Realm; `None` when not entered.
    /// "the beast is likely to have a Magic Might score, which may be assigned
    /// based on the scores of comparable magical creatures" (`ArMDE:10774`). This is the
    /// familiar's Might, never the magus's — no Virtue grant stacks on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub might: Option<MightScore>,
    /// The familiar's eight Characteristics (`ArMDE:17793`), signed and NOT bought from
    /// the magus's Characteristic points. A score of 0 is pruned by
    /// [`Familiar::normalize`] and the map is omitted when empty, so an explicit 0 and
    /// an absent entry serialize identically.
    ///
    /// A bound familiar that lacked human intelligence "gains it, with a score of
    /// –3" (`ArMDE:10854`), which is an ordinary Intelligence entry — so the fixed
    /// eight-value [`Characteristic`] enum needs no `Cunning` variant. The
    /// *unbound* creature's Cunning (Cun) score the Creature Format mentions
    /// (`ArMDE:17793`) is a display-only affordance and is deliberately deferred; see
    /// RULES.md so it is not re-litigated.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub characteristics: BTreeMap<Characteristic, i8>,
    /// The familiar's Size — signed, and commonly **negative** (a raven is -4).
    /// It lowers the bonding level: "If the familiar has negative Size, this
    /// reduces the level for the enchantment" (`ArMDE:10824`). Source: `ArMDE:17795`
    /// (creature Size), `ArMDE:17829-17856` (the Size examples table).
    #[serde(default, skip_serializing_if = "is_zero_i8")]
    pub size: i8,
    /// The familiar's Personality Traits (`ArMDE:17807`). The bond adds Loyal (partner)
    /// +3 (`ArMDE:10852`), which is surfaced as a note and entered by hand, never
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
    /// familiar" (`ArMDE:10866`) — unlike a being's own [`Entity::powers`], which the
    /// power-levels budget bounds. Kept sorted via [`Familiar::normalize`].
    /// Source: `ArMDE:10862-10884` (Empowering the Bond).
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
/// Source: `ArMDE:10836`.
pub const MAX_CORD_SCORE: u8 = 5;

impl Familiar {
    /// Sorts both nested lists (Personality Traits by name, invested powers by name),
    /// prunes Characteristics entered as 0, and clamps the three cords to
    /// [`MAX_CORD_SCORE`], for canonical serialization. Called from
    /// [`Entity::normalize`].
    ///
    /// A 0 is pruned rather than kept because a familiar's Characteristics are
    /// display-only — nothing derives from them and none is bought from a point pool
    /// (`ArMDE:17793`) — so an explicit 0 and an absent entry are the same statement, and
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
/// Spontaneous magic and never to Magic Resistance or lab activities (`ArMDE:10625`).
/// Which spells an attunement covers is free text ([`Self::description`]), so the
/// engine cannot tell whether a given cell of the casting grid is one it enhances,
/// and "touching the talisman" is a moment of play the model does not represent.
/// It is therefore a situational modifier the player applies at the table — the
/// same call as [`crate::derived::soak`]'s Form bonus, which is surfaced as an
/// entered 0. Recorded as a deferral in RULES.md so it is not read as done.
///
/// Source: `ArMDE:10623`, `ArMDE:10625`.
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
/// Source: `ArMDE:10621` ("When a magus
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
/// Source: `ArMDE:10603-10625`.
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
    /// (ArMDE:10662), and the engine only *suggests* a value from today's
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
/// Lab Total" (ArMDE:10662) — but that Lab Total is the one the *creating*
/// magus had in the season the ritual was made. Raising Creo/Corpus or moving to a
/// stronger aura afterwards does not improve an existing ritual; the magus must
/// reinvent it, which is a fresh season's work ("If you reinvent the ritual to take
/// advantage of increased Art scores…", ArMDE:10670, and a failed ritual's
/// focus is repeated unchanged, :10668). So `bonus` is **player-entered for both
/// sources** and stored: `None` means "not entered yet", never a claimed 0. The
/// engine offers a suggestion from today's Lab Total
/// ([`crate::derived::LongevityHint`]) but never writes it here.
///
/// `focus` is the ritual's culminating focus, "which is appropriate to the magus in
/// question" and must be repeated verbatim if the ritual ever fails
/// (ArMDE:10656, :10668) — free text,
/// since the rules give it no mechanics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongevityRitual {
    /// Whether the ritual is self-made or externally provided.
    pub source: LongevitySource,
    /// The player-entered aging bonus; `None` while not yet entered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bonus: Option<i8>,
    /// The ritual's culminating focus, free text; empty while not entered.
    /// Source: ArMDE:10656.
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub focus: String,
}

/// A Twilight Scar: a minor magical trait (beneficial or annoying) a magus
/// acquires from experiencing Twilight. Free-text — the rules give no mechanical
/// number, only a description — and kept sorted via [`Entity::normalize`].
/// Source: ArMDE:9731, :9743.
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
/// Source: ArMDE:16563-16577 (Aging).
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
    /// Source: ArMDE:16567.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub die: Option<i32>,
    /// The AGING TOTAL that die produced, conditions and Longevity Ritual
    /// included. Source: ArMDE:16567-16569.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<i32>,
    /// The Living Conditions in force that year, as ids into the
    /// `rules/core/aging.json` table. Recorded per year because the character's
    /// standing [`Entity::living_conditions`] may legitimately change later.
    /// Source: ArMDE:16581-16594.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub living_conditions: BTreeSet<Id>,
    /// The aging points that year awarded, per Characteristic — the player's own
    /// distribution where the table left the choice open (ArMDE:16615).
    /// Subtracting exactly these is what reverts the year.
    /// Source: ArMDE:16579.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub points: BTreeMap<Characteristic, u8>,
    /// Whether the roll advanced the character's apparent age by one year.
    /// Source: ArMDE:16577.
    #[serde(default, skip_serializing_if = "is_false")]
    pub apparent_age_increased: bool,
    /// Whether the row called for a Crisis (`ArMDE:16602`, `ArMDE:16611`). It says the year
    /// *demanded* one, which is not the same as the Crisis having been rolled: the
    /// four fields below are what records that, and a `true` here with an absent
    /// [`Self::crisis_row`] is a Crisis owed and not yet resolved.
    /// Source: ArMDE:16602, :16611.
    #[serde(default, skip_serializing_if = "is_false")]
    pub crisis: bool,
    /// The Simple Die the player typed for the Crisis, when one was rolled. The
    /// app never rolls this one either.
    /// Source: ArMDE:16621.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crisis_die: Option<i32>,
    /// The CRISIS TOTAL that die produced — "Simple die + age/10 (round up) +
    /// Decrepitude Score", the Decrepitude being the one this very year raised
    /// (`ArMDE:16619`). Recorded for the same reason [`Self::total`] is: it is the
    /// historical record of a roll, and the score it was made against goes on
    /// climbing afterwards.
    /// Source: ArMDE:16619, :16621.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crisis_total: Option<i32>,
    /// The row of the Crisis Table that total landed on, as an id into
    /// `rules/core/aging.json` — `crisis.minor_illness` and friends. An id, never
    /// a name: the row's text lives in `rules/i18n/<lang>/aging.json`.
    /// Source: ArMDE:16624-16632.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crisis_row: Option<Id>,
    /// How bad that row was, recorded beside its id rather than left to be looked
    /// up again. `AgingRules::crisis` is optional — a ruleset may ship no Crisis
    /// Table at all — so a save can outlive the table that produced it, and the
    /// severity is then the only thing left that says what the character went
    /// through. `None` beside a present [`Self::crisis_row`] is a Bedridden row
    /// (`ArMDE:16626`, `ArMDE:16627`), which has no severity because it is time rather than
    /// an illness.
    /// Source: ArMDE:16628-16632.
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

/// `skip_serializing_if` predicate: omits a `u16` field when it is zero.
fn is_zero_u16(n: &u16) -> bool {
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
    /// standing "omit the default" convention. It must be:
    /// [`load_entity_migrating`](crate::load_entity_migrating)
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
    /// from its Warping Score ("Effects of Warping", ArMDE:16547-16561),
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
    /// (ArMDE:2366-2376). `None` when unset (no cap enforced yet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// The character's apparent age in years (ArMDE:1155). The resolved
    /// outcome of aging rolls: "the character's apparent age increases by one
    /// year" whenever the AGING TOTAL clears the table's threshold
    /// (ArMDE:16577), which [`crate::aging`] resolves and a resolved year
    /// writes here. It stays directly editable — a hand-entered age is never
    /// overwritten — and `None` when unset. Mirrors [`Entity::age`]'s
    /// representation.
    ///
    /// Never an input to the aging roll: the modifier "depends on the character's
    /// **actual, not apparent**, age" (`ArMDE:16577`).
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
    /// are folded in by [`load_entity_migrating`](crate::load_entity_migrating).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub talisman: Option<Talisman>,
    /// The magus's Longevity Ritual. `None` when there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longevity_ritual: Option<LongevityRitual>,
    /// The circumstances the character lives under, as ids into the Living
    /// Conditions table of `rules/core/aging.json`
    /// (ArMDE:16581-16594). The other modifier the AGING TOTAL subtracts,
    /// alongside the Longevity Ritual above (`ArMDE:16567-16569`).
    ///
    /// A **set**, because the asterisked rows are not alternatives: "Modifiers
    /// marked with an asterisk are cumulative with each other" (`ArMDE:16594`), so a
    /// leper who works in a mine holds two rows and their modifiers add.
    ///
    /// **Empty means the table's baseline, not an unfinished entry.** The table
    /// prints "Average peasant 0" (`ArMDE:16587`), so a character who names no
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
    /// Source: ArMDE:16579.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub aging_points: BTreeMap<Characteristic, u8>,
    /// Accrued Warping Points. Summed with any grant-derived Warping Points (Warped
    /// by Magic, …) and inverted through the advancement curve to the Warping Score
    /// by [`crate::effective::warping_score`]. Source: ArMDE:16464-16475.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub warping_points: u32,
    /// Free-text description of how the character's Warping manifests (the
    /// source-reflecting Minor/Major Flaw from "Effects of Warping",
    /// ArMDE:16547-16561). A pure
    /// annotation carrying NO mechanic: it is
    /// deliberately NOT a Flaw `selection`, because a post-creation warping Flaw
    /// must not count against the creation Virtue/Flaw budget. Empty when unset.
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub warping_effect: String,
    /// Twilight Scars the magus has acquired (free-text). Kept sorted via
    /// [`Entity::normalize`]. Source: ArMDE:9731, :9743.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub twilight_scars: Vec<TwilightScar>,
    /// Free-text narrative of the character's overall aging / decrepitude
    /// (ArMDE:16563-16577). A pure
    /// annotation carrying NO mechanic —
    /// Decrepitude is derived from `aging_points`; this only records flavor.
    /// Empty when unset.
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub decrepitude_effect: String,
    /// Per-year aging-roll log (ArMDE:16563-16577). The app never rolls
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
    /// D42: the concept's default realm — a **default source, nothing else**
    /// (ArMDE:2960's "this should be the choice if the character concept does
    /// not suggest another option"). Every Supernatural entry's realm
    /// resolves from its own override, else this, else [`Realm::Magic`]
    /// ([`crate::effective::resolve_realm`]); it is NOT the character's own
    /// realm and carries no mechanical force on its own (D42 "What it is
    /// not"). `None` when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concept_realm: Option<Realm>,
    /// The calendar year the saga this entity was built for stands in (schema 17).
    ///
    /// **Document state, not a preference.** It was a machine-global app setting until
    /// C8, which made it wrong for every saga but one on the machine of a storyguide
    /// running two: opening a 1220 Rhine magus while the setting said 1197 reported
    /// the wrong age and could raise a spurious `saga_year_before_birth_year`.
    ///
    /// Drives [`crate::age_in_saga_year`] and [`crate::birth_year_in_saga_year`], and
    /// nothing else — it is the reference `age` and `birth_year` are two views
    /// against, and changing it rewrites neither (D3.3).
    ///
    /// **Always serialized, even at its default.** The second such field, after
    /// [`Entity::ability_funding`], and for the same reason:
    /// [`load_entity_migrating`](crate::load_entity_migrating)
    /// dispatches on this key's *absence* to fill a pre-17 save from the default its
    /// caller supplies, so a `skip_serializing_if` here would turn every save that
    /// happened to sit at 1220 back into a migration candidate. `serde(default)` is
    /// only for a hand-edited schema-17 file that drops the key.
    #[serde(default = "default_saga_year")]
    pub saga_year: i32,
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
    /// and Encumbrance. Source: ArMDE:16944-17011.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment: Vec<EquipmentSlot>,
    /// The supernatural being's base Might Score + Realm (grog/companion/mythic
    /// companion with a Might Virtue). `None` for ordinary characters. The
    /// effective Might is this base plus same-Realm [`Effect::MightGrant`]s (see
    /// [`crate::effective::effective_might`]). Source: RoP:M:1470-1472.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub might: Option<MightScore>,
    /// The being's supernatural powers; each `level` is charged against the
    /// power-levels budget its Might Virtues grant. Kept sorted via
    /// [`Entity::normalize`]. Defaults to empty. Source: RoP:I:4122; RoP:D:1977.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub powers: Vec<SupernaturalPower>,
    /// The character's Focus Powers, bought out of the Focus Power point pool
    /// ([`crate::effective::focus_points_budget`]) rather than the level budget
    /// [`Self::powers`] draws on — a separate currency at 2 points per level of
    /// effect and 1 per point of Penetration. Kept sorted via
    /// [`Entity::normalize`]. Defaults to empty, so a save written before this
    /// field loads unchanged and writes identical bytes, and [`SCHEMA_VERSION`]
    /// does not move. Source: `ArMDE:3899`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub focus_powers: Vec<FocusPower>,
    /// Whether the character is currently fighting mounted (K3): `combat_totals`
    /// adds a second, mounted [`crate::derived::CombatLine`] per line whose weapon
    /// is not a body attack, adding `min(Ride, 3)` to Attack and Defense. Additive
    /// and serde-defaulted: absent/false means every existing save's behavior is
    /// unchanged (no mounted lines), so no [`SCHEMA_VERSION`] bump is needed for
    /// this field (design-f0-book-template-engine.md § 6). Source: ArMDE:16837-16839.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub mounted: bool,
}

/// The engine's own fallback for [`Entity::saga_year`].
fn default_saga_year() -> i32 {
    crate::validation::DEFAULT_SAGA_YEAR
}

/// The lowest rules-legal [`Entity::aura`] modifier: a Divine aura acting on
/// Infernal-realm powers, "– (5 x aura)", at the highest aura rating the rules
/// give ("can be rated in power on a scale from 1 to 10").
///
/// Source: ArMDE:17390 (rating scale),
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
            concept_realm: None,
            saga_year: crate::validation::DEFAULT_SAGA_YEAR,
            sigil: String::new(),
            covenant_name: String::new(),
            parens: String::new(),
            equipment: Vec::new(),
            might: None,
            powers: Vec::new(),
            focus_powers: Vec::new(),
            mounted: false,
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
        self.focus_powers.sort();
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
    // Two tests below are about a *field's* serialization and reach for the loader
    // only as the route a save takes in; the migration subsystem itself is tested in
    // `migration.rs`.
    use crate::load_entity_migrating;
    use crate::ruleset::{Ruleset, RulesetSources};
    use crate::validation::DEFAULT_SAGA_YEAR;
    use pretty_assertions::assert_eq;
    use std::collections::{BTreeMap, BTreeSet};

    /// An empty ruleset + empty catalogue names — the two tests below route
    /// through [`load_entity_migrating`] only for a field-serialization round
    /// trip, unrelated to catalogued parameters.
    fn empty_ruleset_and_names() -> (Ruleset, BTreeMap<Id, Vec<String>>) {
        let ruleset = Ruleset::from_sources(RulesetSources {
            point_items: "[]",
            type_profiles: "[]",
            ..RulesetSources::default()
        })
        .expect("an empty ruleset loads");
        (ruleset, BTreeMap::new())
    }

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
        assert!(
            !Prereq::HermeticallyTrained.conflicts_with_house(Some(&Id::new("house.bjornaer")))
        );
        assert!(!Prereq::OrderMember.conflicts_with_house(Some(&Id::new("house.bjornaer"))));
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

    /// Every `Prereq::CharacterType` leaf a tree contains, as the referenced
    /// character-type ids (there may be more than one, e.g. inside an `Any`).
    /// Mirrors `ruleset/integrity.rs::validate_prereq_refs`'s own recursion
    /// shape, but collects rather than validates — exhaustive, so a future
    /// `Prereq` variant is a compile error here too, not a silently-missed leaf.
    fn character_type_leaves<'a>(prereq: &'a Prereq, out: &mut Vec<&'a Id>) {
        match prereq {
            Prereq::All(children) | Prereq::Any(children) | Prereq::Nor(children) => {
                for child in children {
                    character_type_leaves(child, out);
                }
            }
            Prereq::CharacterType(id) => out.push(id),
            Prereq::Has(_)
            | Prereq::House(_)
            | Prereq::AbilityMin { .. }
            | Prereq::ArtMin { .. }
            | Prereq::HermeticallyTrained
            | Prereq::OrderMember
            | Prereq::IsCompanion
            | Prereq::IsGrog
            | Prereq::HasCategory(_)
            | Prereq::AgeMin(_)
            | Prereq::HasCategoryAtMagnitude { .. }
            | Prereq::CharacteristicMin { .. }
            | Prereq::AbilityCategoryScoreMin { .. }
            | Prereq::AnyArtMin { .. } => {}
        }
    }

    /// Every `Prereq::CharacterType` (D38/D75, F-556) carrier in the SHIPPED
    /// catalogue's point-item `prerequisites`/`advisory_prerequisites`, as
    /// `(item id, character-type id)` pairs. `CharacterType` is the only
    /// `Prereq` variant carrying a real id that gets NO referential check at
    /// load (`ruleset/integrity.rs::Ruleset::validate_prereq_refs` deliberately
    /// skips it — see that variant's own doc comment for why: F-556's whole
    /// point is naming an id that resolves to no shipped profile). A typo on
    /// either side therefore fails closed — the Virtue becomes permanently
    /// unselectable — with no load error and no other test failure, unless
    /// this enumeration catches it: a carrier added, removed, or retargeted
    /// without updating this list fails the assertion below instead of
    /// silently drifting.
    ///
    /// Scoped to `PointItem`'s own two prerequisite trees — a type profile's
    /// `CategoryRule`/`PhaseRule` `when` field also carries a `Prereq` (see
    /// `validate_prereq_refs`'s other two call sites), but no shipped profile
    /// uses `CharacterType` there today; that is a separate carrier class this
    /// enumeration does not claim to cover.
    #[test]
    fn character_type_prereq_carriers_match_the_shipped_catalogue() {
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: include_str!("../../../rules/core/virtues_flaws.json"),
            type_profiles: include_str!("../../../rules/core/character_types.json"),
            abilities: Some(include_str!("../../../rules/core/abilities.json")),
            arts: Some(include_str!("../../../rules/core/arts.json")),
            houses: Some(include_str!("../../../rules/core/houses.json")),
            mythic_types: Some(include_str!(
                "../../../rules/core/mythic_companion_types.json"
            )),
            spells: Some(include_str!("../../../rules/core/spells.json")),
            spell_mastery_abilities: None,
            equipment: Some(include_str!("../../../rules/core/equipment.json")),
            characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
            life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
            childhoods: None,
            aging: Some(include_str!("../../../rules/core/aging.json")),
            parameter_catalogues: Some(include_str!(
                "../../../rules/core/parameter_catalogues.json"
            )),
        })
        .expect("shipped core ruleset loads");

        let mut carriers: BTreeSet<(&str, &str)> = BTreeSet::new();
        for (id, item) in &rs.point_items {
            for tree in [&item.prerequisites, &item.advisory_prerequisites]
                .into_iter()
                .flatten()
            {
                let mut leaves = Vec::new();
                character_type_leaves(tree, &mut leaves);
                for character_type in leaves {
                    carriers.insert((id.as_str(), character_type.as_str()));
                }
            }
        }

        let expected: BTreeSet<(&str, &str)> = [
            ("virtue.domestic_animal", "character_type.domestic_animal"),
            // D78.1: Companion Animal reuses the SAME never-profiled sentinel id
            // as Domestic Animal — both are "the animal character type no
            // profile has", not two distinct ids.
            ("flaw.companion_animal", "character_type.domestic_animal"),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            carriers, expected,
            "a Prereq::CharacterType carrier was added, removed, or retargeted in the \
             catalogue without updating this enumeration — CharacterType gets no \
             referential check at load (by design, F-556), so a mismatch here would \
             otherwise fail closed and silent"
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

    /// The fourth `Classification`. `Narrative` used to mean two different
    /// things at once — "the rulebook states no rule" and "the rulebook states
    /// a rule the engine cannot compute" — so no guard could tell a dropped
    /// rule from genuine flavour. This asserts the catalogue can carry the
    /// separating value, spelled `uncomputed_rule` in the JSON.
    #[test]
    fn uncomputed_rule_is_a_classification_the_catalogue_can_carry() {
        let parsed: Classification = serde_json::from_str("\"uncomputed_rule\"")
            .expect("`uncomputed_rule` deserializes as a Classification");
        assert_eq!(parsed.to_string(), "uncomputed_rule");
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
        check(Classification::UncomputedRule);
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
        check(ParameterDomain::Number);
        check(CastingScope::All);
        check(CastingScope::Formulaic);
        check(CastingScope::Ritual);
        check(CastingScope::FormulaicRitual);
        check(CastingScope::Spontaneous);
        check(HalvableTotal::SpontaneousCasting);
        check(HalvableTotal::LabEnchanting);
        check(HalvableTotal::LabLongevity);
        check(HalvableTotal::Penetration);
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
        check(MagicResistanceEffect::SusceptibleFaerie);
        check(MagicResistanceEffect::SusceptibleInfernal);
        check(MagicResistanceEffect::ConditionalPenetrationWaiver);
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
        check(AdvancementSource::Authoring);
        check(AdvancementSource::SpellMastery);
        check(AdvancementFactor::Half);
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
        check(SpecialCasting::DoubledAuraPenalty);
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
                penetration: 0,
            },
            SupernaturalPower {
                name: "Coagulation".into(),
                level: 10,
                penetration: 5,
            },
        ];
        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
        assert!(json.contains("\"realm\":\"infernal\""));
    }

    #[test]
    fn power_penetration_defaults_to_zero_and_is_omitted_when_unspent() {
        // `penetration` is additive, so SCHEMA_VERSION stays put: a save written
        // before the field existed carries no key and must read as 0, and a power
        // that spent no levels on Penetration must round-trip without growing one
        // (canonical serialization, zero-noise diffs). A power that DID buy
        // Penetration keeps it (ArMDE:4019).
        let older: SupernaturalPower =
            serde_json::from_str(r#"{"name":"Curse of Sleep","level":25}"#).unwrap();
        assert_eq!(older.penetration, 0);

        let json = serde_json::to_string(&older).unwrap();
        assert!(
            !json.contains("penetration"),
            "an unspent Penetration must not appear in the save: {json}"
        );

        let spent = SupernaturalPower {
            name: "Wolf Shape".into(),
            level: 20,
            penetration: 20,
        };
        let json = serde_json::to_string(&spent).unwrap();
        assert!(json.contains("\"penetration\":20"), "{json}");
        assert_eq!(
            serde_json::from_str::<SupernaturalPower>(&json).unwrap(),
            spent
        );
    }

    /// `focus_powers` is additive, so a save written before it carries no key,
    /// reads as an empty list, and writes identical bytes — SCHEMA_VERSION stays
    /// put. `normalize` sorts the list like every other, for zero-noise diffs.
    #[test]
    fn focus_powers_round_trip_are_sorted_and_absent_when_empty() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef {
                id: Id::new("test"),
                version: "1".into(),
            },
        );
        // Empty: the key must not appear at all.
        let json = serde_json::to_string(&entity).unwrap();
        assert!(!json.contains("focus_powers"), "{json}");

        entity.focus_powers = vec![
            FocusPower {
                name: "Wolves".into(),
                max_level: 10,
                penetration: 5,
            },
            FocusPower {
                name: "Beasts of the wood".into(),
                max_level: 5,
                penetration: 0,
            },
        ];
        entity.normalize();
        assert_eq!(
            entity
                .focus_powers
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Beasts of the wood", "Wolves"]
        );

        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
        // An unspent Penetration is omitted, exactly as on a SupernaturalPower.
        assert!(
            json.contains("{\"name\":\"Beasts of the wood\",\"max_level\":5}"),
            "{json}"
        );
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
                potent_field_only: false,
            },
            Effect::LabTotalMod {
                amount: 3,
                scope: LabTotalModScope::InPlayGrid,
                suppressed_when: None,
            },
            Effect::DeficientArt {
                param: "technique".into(),
            },
            Effect::MagicTotalHalving {
                total: HalvableTotal::Penetration,
            },
            Effect::SoakMod {
                amount: 3,
                gate: None,
            },
            Effect::CombatMod {
                amount: -2,
                target: CombatStat::Defense,
                weapon: None,
            },
            Effect::HealthMod {
                track: HealthTrack::WoundPenalty,
                amount: 1,
            },
            Effect::MagicResistanceMod {
                kind: MagicResistanceEffect::NoFormBonus,
                param: Some("form".into()),
                amount: 0,
                gate: None,
            },
            Effect::MagicResistanceMod {
                kind: MagicResistanceEffect::HalvedParma,
                param: Some("form".into()),
                amount: 0,
                gate: None,
            },
            Effect::MagicResistanceMod {
                kind: MagicResistanceEffect::SusceptibleFaerie,
                param: None,
                amount: 0,
                gate: None,
            },
            Effect::AgingMod {
                kind: AgingEffect::AgingRoll,
                amount: -1,
            },
            Effect::AdvancementMod {
                source: AdvancementSource::Taught,
                amount: Some(5),
                factor: None,
            },
            Effect::AdvancementMod {
                source: AdvancementSource::Authoring,
                amount: Some(3),
                factor: None,
            },
            Effect::AdvancementMod {
                source: AdvancementSource::Teaching,
                amount: None,
                factor: Some(AdvancementFactor::Half),
            },
            Effect::SpecialCastingMod {
                kind: SpecialCasting::Diedne,
                param: None,
            },
            Effect::SpecialCastingMod {
                kind: SpecialCasting::DeftForm,
                param: Some("form".into()),
            },
            Effect::AbilityRollModParam {
                param: "subject".into(),
                amount: 3,
            },
            Effect::AbilityRollMod {
                ability: Id::new("ability.awareness"),
                amount: -3,
                gate: None,
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
        assert!(json.contains("\"source\":\"authoring\""));
        assert!(json.contains("\"factor\":\"half\""));
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
          "source": { "anchor": "gentle-gift", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [120, 135] }
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
                anchor: "gentle-gift".to_string(),
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
        // Source: ArMDE:6803-6804.
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
        let source = SourceRef::new("file.md", LineRange::new(10, 20), "anchor");
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("[10,20]"), "lines as array: {json}");
    }

    /// A line number is a *derived* coordinate: it moves under every upstream
    /// edit above it, and `rules_source_provenance.rs` can only prove a range
    /// lands on non-blank lines, never that it lands on the right rule. The
    /// heading anchor is the durable half of the same reference — it survives an
    /// edit anywhere else in the book and fails loudly when it breaks. So a
    /// `SourceRef` must carry one, round-trip, without losing it to serde.
    #[test]
    fn source_ref_preserves_the_heading_anchor() {
        let json = r#"{"anchor":"clumsy","file":"file.md","lines":[10,20]}"#;
        let source: SourceRef = serde_json::from_str(json).unwrap();
        let round_tripped = serde_json::to_string(&source).unwrap();
        assert!(
            round_tripped.contains(r#""anchor":"clumsy""#),
            "a SourceRef must keep the heading anchor it was given: {round_tripped}"
        );
    }

    /// **X9a-10.** D30.1 makes `anchor` mandatory catalogue-wide: a
    /// `SourceRef` with no `anchor` key in its JSON must fail to deserialize
    /// rather than quietly default to an absent anchor. `anchor` is a plain
    /// `String` with no `#[serde(default)]`, so a missing key is a hard
    /// deserialization error.
    #[test]
    fn source_ref_without_an_anchor_key_fails_to_deserialize() {
        let json = r#"{"file":"file.md","lines":[10,20]}"#;
        let result: Result<SourceRef, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "D30 makes anchor mandatory: a SourceRef with no anchor key must fail to \
             deserialize, not default to None: {result:?}"
        );
    }

    /// **X9a-10 — replaces `source_ref_without_an_anchor_serializes_without_the_key`.**
    /// That test asserted the behavior D30 now rejects: that an anchor-less
    /// `SourceRef` serializes cleanly with the key omitted. D30 says the
    /// opposite — every `SourceRef` carries an anchor, so the serialized form
    /// must always carry the key.
    #[test]
    fn source_ref_new_always_serializes_the_given_anchor() {
        let source = SourceRef::new("file.md", LineRange::new(10, 20), "clumsy");
        let json = serde_json::to_string(&source).unwrap();
        assert!(
            json.contains(r#""anchor":"clumsy""#),
            "a SourceRef must always carry the anchor it was constructed with: {json}"
        );
    }

    /// **X9a-10 — pins the field itself, not just the JSON round-trip,** so a
    /// bug in `Serialize`/`Deserialize` cannot hide a constructor that never
    /// stored the value in the first place.
    #[test]
    fn source_ref_new_stores_the_given_anchor() {
        let source = SourceRef::new("file.md", LineRange::new(10, 20), "clumsy");
        assert_eq!(
            source.anchor, "clumsy",
            "SourceRef::new must store the anchor it is given, not ignore it"
        );
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
          "source": { "anchor": "puissant-ability", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [240, 251] }
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

    /// `index_categories` is provenance — the headings the book's own index
    /// files an entry under beyond the ones its descriptor makes membership
    /// categories. The index has no meaningful order (unlike a descriptor,
    /// whose order carries its emphasis), so the list IS canonically sorted,
    /// and `normalize` is where that happens.
    #[test]
    fn point_item_normalize_sorts_index_categories() {
        let mut item: PointItem = serde_json::from_str(
            r#"{
              "id": "flaw.unbearable_to_beings",
              "kind": "flaw",
              "classification": "narrative",
              "magnitude": "minor",
              "categories": ["general"],
              "index_categories": ["hermetic", "another"],
              "entity_kinds": ["character"]
            }"#,
        )
        .unwrap();

        // Membership order is the descriptor's own and stays untouched.
        assert_eq!(item.categories, vec!["general"]);

        item.normalize();
        assert_eq!(item.index_categories, vec!["another", "hermetic"]);
    }

    /// The field is optional and absent by default — the overwhelming majority
    /// of entries are indexed exactly where their descriptor says, so they
    /// carry no `index_categories` key at all and must not gain an empty one on
    /// a round trip.
    #[test]
    fn point_item_index_categories_default_empty_and_are_omitted() {
        let item: PointItem = serde_json::from_str(
            r#"{
              "id": "flaw.optimistic",
              "kind": "flaw",
              "classification": "narrative",
              "magnitude": "minor",
              "categories": ["personality"]
            }"#,
        )
        .unwrap();
        assert!(item.index_categories.is_empty());

        let reserialized = serde_json::to_string(&item).unwrap();
        assert!(
            !reserialized.contains("index_categories"),
            "an empty provenance list is omitted: {reserialized}"
        );
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
            vec![
                PhaseRule::Always(CreationPhase::Experience),
                PhaseRule::Always(CreationPhase::Concept)
            ]
        );
    }

    /// Mirrors [`CategoryRule`]'s own bare-slug test: a plain phase string in
    /// `creation_phases` must deserialize to the unconditional `Always` form,
    /// not force every existing profile onto the object shape.
    #[test]
    fn phase_rule_bare_slug_stays_always_form() {
        let phases: Vec<PhaseRule> = serde_json::from_str(r#"["concept", "arts"]"#).unwrap();
        assert_eq!(
            phases,
            vec![
                PhaseRule::Always(CreationPhase::Concept),
                PhaseRule::Always(CreationPhase::Arts)
            ]
        );
    }

    /// The conditional `{phase, when}` shape (A2/D56): a phase gated on a
    /// `Prereq`, mirroring `CategoryRule::When` exactly.
    #[test]
    fn phase_rule_conditional_form_roundtrips() {
        let json = r#"{"phase": "arts", "when": {"kind": "hermetically_trained"}}"#;
        let rule: PhaseRule = serde_json::from_str(json).unwrap();
        assert_eq!(
            rule,
            PhaseRule::When {
                phase: CreationPhase::Arts,
                when: Prereq::HermeticallyTrained
            }
        );
        assert_eq!(rule.phase(), CreationPhase::Arts);
        assert_eq!(rule.when(), Some(&Prereq::HermeticallyTrained));

        let reserialized = serde_json::to_string(&rule).unwrap();
        let roundtripped: PhaseRule = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(rule, roundtripped);
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
    fn prereq_hermetically_trained() {
        let json = r#"{ "kind": "hermetically_trained" }"#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(prereq, Prereq::HermeticallyTrained);

        // The unit variant round-trips with no `value` key.
        let reserialized = serde_json::to_string(&prereq).unwrap();
        assert_eq!(reserialized, r#"{"kind":"hermetically_trained"}"#);
        let roundtripped: Prereq = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(prereq, roundtripped);
    }

    #[test]
    fn prereq_order_member() {
        let json = r#"{ "kind": "order_member" }"#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(prereq, Prereq::OrderMember);

        // The unit variant round-trips with no `value` key.
        let reserialized = serde_json::to_string(&prereq).unwrap();
        assert_eq!(reserialized, r#"{"kind":"order_member"}"#);
        let roundtripped: Prereq = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(prereq, roundtripped);
    }

    #[test]
    fn prereq_is_companion() {
        let json = r#"{ "kind": "is_companion" }"#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(prereq, Prereq::IsCompanion);

        // The unit variant round-trips with no `value` key.
        let reserialized = serde_json::to_string(&prereq).unwrap();
        assert_eq!(reserialized, r#"{"kind":"is_companion"}"#);
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
        assert!(profile.names_forbidden_category("hermetic"));
        // A bare slug stays the bare `Always` form — the untagged
        // `CategoryRule` must not turn every existing rules file into the
        // object shape on the way in or out.
        assert_eq!(
            profile.forbidden_categories,
            vec![CategoryRule::Always("hermetic".to_string())]
        );
        assert_eq!(profile.creation_phases.len(), 6);

        let reserialized = serde_json::to_string(&profile).unwrap();
        let roundtripped: EntityTypeProfile = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(profile, roundtripped);
    }

    /// The category lists were `BTreeSet`s and canonicalised themselves;
    /// [`CategoryRule`] made them `Vec`s, so `normalize` has to sort them
    /// explicitly or the canonical writer emits authoring order. Both forms
    /// sort on the same key — the category — so a conditional entry lands
    /// exactly where its bare twin would.
    #[test]
    fn normalize_sorts_the_category_lists_by_category() {
        let json = r#"{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": [
            "supernatural",
            "general",
            { "category": "hermetic", "when": { "kind": "has", "value": "virtue.the_gift" } }
          ],
          "creation_phases": []
        }"#;

        let mut profile: EntityTypeProfile = serde_json::from_str(json).unwrap();
        profile.normalize();

        assert_eq!(
            profile
                .permitted_categories
                .iter()
                .map(CategoryRule::category)
                .collect::<Vec<_>>(),
            vec!["general", "hermetic", "supernatural"]
        );
        assert!(profile.permitted_categories[1].when().is_some());
    }

    #[test]
    fn entity_type_profile_hermetically_trained_and_order_member_default_false_and_omitted() {
        // Absent `hermetically_trained`/`order_member` deserialize to false...
        let json = r#"{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "creation_phases": []
        }"#;
        let profile: EntityTypeProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.hermetically_trained);
        assert!(!profile.order_member);

        // ...and a false flag is omitted from canonical JSON.
        let serialized = serde_json::to_string(&profile).unwrap();
        assert!(
            !serialized.contains("hermetically_trained"),
            "false hermetically_trained must be omitted: {serialized}"
        );
        assert!(
            !serialized.contains("order_member"),
            "false order_member must be omitted: {serialized}"
        );
    }

    #[test]
    fn entity_type_profile_hermetically_trained_and_order_member_true_roundtrip() {
        let json = r#"{
          "id": "magus",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "hermetically_trained": true,
          "order_member": true,
          "creation_phases": []
        }"#;
        let profile: EntityTypeProfile = serde_json::from_str(json).unwrap();
        assert!(profile.hermetically_trained);
        assert!(profile.order_member);

        let serialized = serde_json::to_string(&profile).unwrap();
        assert!(
            serialized.contains(r#""hermetically_trained":true"#),
            "{serialized}"
        );
        assert!(
            serialized.contains(r#""order_member":true"#),
            "{serialized}"
        );
        let roundtripped: EntityTypeProfile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(profile, roundtripped);
    }

    #[test]
    fn entity_type_profile_hermetically_trained_and_order_member_are_independent() {
        // A Redcap-shaped profile: Order member without Hermetic training
        // (ArMDE:4842-4851) — the two fields must not be coupled to each
        // other by the struct itself, only by today's shipped data.
        let json = r#"{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "order_member": true,
          "creation_phases": []
        }"#;
        let profile: EntityTypeProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.hermetically_trained);
        assert!(profile.order_member);
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
            ability_scores: vec![{
                let mut a = AbilityScore::new(Id::new("ability.awareness"), 3);
                a.specialty = Some("searching".into());
                a
            }],
            xp_pool: 30,
            life_stages: None,
            ability_funding: AbilityFunding::Pool,
            wizard_furthest_phase: Some("abilities".into()),
            art_scores: vec![ArtScore::new(Id::new("art.creo"), 5)],
            spells: vec![SpellSelection::new(Id::new("spell.pilum_of_fire"))],
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
            concept_realm: None,
            saga_year: crate::validation::DEFAULT_SAGA_YEAR,
            sigil: String::new(),
            covenant_name: String::new(),
            parens: String::new(),
            equipment: Vec::new(),
            might: None,
            powers: Vec::new(),
            focus_powers: Vec::new(),
            mounted: false,
        };

        let json = serde_json::to_string_pretty(&entity).unwrap();
        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);

        assert!(json.contains(r#""schema_version": 21"#));
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

    /// C1's serde-error-text obligation (`docs/vf-audit/design-c0-parameter-model.md`
    /// § 10): `AbilityRef`'s `#[serde(untagged)]` is exactly the shape known to
    /// produce poor error text, on the same precedent as
    /// [`profile_with_an_unknown_creation_phase_fails_to_parse`] just below.
    /// Captured verbatim today: `data did not match any variant of untagged
    /// enum AbilityRef` — this pins that the message still NAMES the wrapper,
    /// so a future serde/dependency bump that drops even that (e.g. collapsing
    /// to a bare "invalid type" with no enum name at all) is caught here
    /// rather than silently shipping worse diagnostics.
    #[test]
    fn a_malformed_untagged_ability_ref_names_the_wrapper_in_its_error() {
        let wrong_type = serde_json::from_str::<AbilityRef>(r#"{"ability": 5}"#)
            .expect_err("a numeric ability id must not parse");
        assert!(
            wrong_type.to_string().contains("AbilityRef"),
            "the error must name the untagged wrapper it failed to match: {wrong_type}"
        );

        let missing_field =
            serde_json::from_str::<AbilityRef>(r#"{"gate": {"param": "p", "equals": "x"}}"#)
                .expect_err("an object missing 'ability' must not parse");
        assert!(
            missing_field.to_string().contains("AbilityRef"),
            "the error must name the untagged wrapper it failed to match: {missing_field}"
        );
    }

    /// A profile's phases are typed, so a phase string the engine has no phase for
    /// fails the load instead of reaching the wizard as a step it cannot render.
    ///
    /// Since `PhaseRule` (A2/D56) wraps `CreationPhase` in an untagged enum —
    /// required so a bare phase slug keeps deserializing unchanged — serde's
    /// untagged-enum error reporting no longer echoes the offending token
    /// itself (it names the wrapper, not the value); the load-time refusal
    /// itself is what this test guards, not the message's wording.
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
            err.to_string().contains("PhaseRule"),
            "the error must name the untagged wrapper it failed to match: {err}"
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
        entity.spells = vec![SpellSelection::new(Id::new("spell.unseen_arm")), {
            let mut s = SpellSelection::new(Id::new("spell.aegis_of_the_hearth"));
            s.level = Some(20);
            s
        }];
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
            concept_realm: None,
            saga_year: crate::validation::DEFAULT_SAGA_YEAR,
            sigil: String::new(),
            covenant_name: String::new(),
            parens: String::new(),
            equipment: Vec::new(),
            might: None,
            powers: Vec::new(),
            focus_powers: Vec::new(),
            mounted: false,
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
        assert!(json.contains(r#""schema_version": 21"#));
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
                penetration: 0,
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
                    penetration: 0,
                },
                SupernaturalPower {
                    name: "Mental communication".into(),
                    level: 15,
                    penetration: 0,
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
    /// Characteristics are display-only (`ArMDE:17793`) and nothing is bought with them, so
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
    /// (0…+5, ArMDE:10836) for the same reason it prunes a zero
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
        assert!(json.contains(r#""schema_version": 21"#));
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
    /// (ArMDE:16587).
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

        // Two cumulative rows (`ArMDE:16594`) are a set, and the `BTreeSet` orders them
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
    /// answered — the Simple Die the player typed (`ArMDE:16621`), the CRISIS TOTAL it
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
    /// Source: ArMDE:16621, :16624-16632.
    #[test]
    fn a_resolved_crisis_round_trips_and_needs_no_schema_bump() {
        // 21 is X9b's own bump (`virtue.rard` -> `virtue.bard`); the Crisis
        // widening contributed nothing to it, nor to 20's bump
        // (`EquipmentSlot::loadout`, F1), 19's multi-valued parameter type
        // (C5a), 16's funding discriminator, 17's saga year, or 18's
        // ability-parameter type widening (CV4).
        assert_eq!(
            SCHEMA_VERSION, 21,
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
        let (rs, names) = empty_ruleset_and_names();
        let back = load_entity_migrating(&json, DEFAULT_SAGA_YEAR, &rs, &names)
            .unwrap()
            .entity;
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
            let (rs, names) = empty_ruleset_and_names();
            let loaded = load_entity_migrating(&save, DEFAULT_SAGA_YEAR, &rs, &names)
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
        assert!(json.contains(r#""schema_version": 21"#), "{json}");

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
            AbilityScore::new(Id::new("ability.swim"), 2),
            AbilityScore::new(Id::new("ability.awareness"), 3),
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
          "effects": [
            { "type": "characteristic_score_delta_param", "param": "characteristic", "amount": 1 }
          ],
          "max_per_target": 2
        }"#;
        let item: PointItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.max_per_target, 2);
        assert_eq!(
            item.effects,
            vec![Effect::CharacteristicScoreDeltaParam {
                param: "characteristic".into(),
                amount: 1,
                gate: None,
                cap: CharacteristicDeltaCap::AboveBase,
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

    /// D10: absent `max_total` means ONCE, not "no ceiling" — the model's
    /// default used to be exactly the inverse of `ArMDE:2814`'s "Most Virtues
    /// and Flaws may only be taken once."
    #[test]
    fn max_total_defaults_to_one_and_is_omitted_when_default() {
        let json = r#"{
          "id": "virtue.keen_vision",
          "kind": "virtue",
          "classification": "narrative",
          "magnitude": "minor",
          "categories": ["general"]
        }"#;
        let item: PointItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.max_total, 1);
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
    fn restricted_ability_xp_from_normal_budget_defaults_false_and_stays_out_of_json() {
        // D13: every pre-existing carrier's JSON (Educated, Warrior, Privileged
        // Upbringing, ...) predates `from_normal_budget` and must keep meaning
        // "additive grant", byte-identical, now that the field exists.
        let old_json =
            r#"{"type":"restricted_ability_xp","amount":50,"abilities":["ability.magic_lore"]}"#;
        let effect: Effect = serde_json::from_str(old_json).unwrap();
        assert_eq!(
            effect,
            Effect::RestrictedAbilityXp {
                amount: 50,
                abilities: vec![Id::new("ability.magic_lore")],
                categories: Vec::new(),
                instances: Vec::new(),
                from_normal_budget: false,
                abilities_param: None,
            }
        );
        let round_tripped = serde_json::to_string(&effect).unwrap();
        assert_eq!(
            round_tripped, old_json,
            "an entry that never set the flag must not grow a `from_normal_budget` key"
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

    /// The Realm taxonomy is addressable by id, exactly as
    /// [`crate::characteristics::Characteristic`] is: `realm.<slug>` in, the
    /// enum member out. That round trip is what lets a parameter value name a
    /// Realm without any registry — Folk Magic's realm axis
    /// (ArMDE:3909, :3919).
    #[test]
    fn a_realm_id_round_trips_through_from_id() {
        for realm in Realm::ALL {
            assert_eq!(Realm::from_id(&realm.id()), Some(realm));
        }
        assert_eq!(Realm::from_id(&Id::new("realm.nope")), None);
        assert_eq!(Realm::from_id(&Id::new("characteristic.str")), None);
        assert_eq!(Realm::from_id(&Id::new("divine")), None);
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
            Some(&SelectionParamValue::Single(Id::new("ability.awareness")))
        );
    }

    #[test]
    fn ability_score_new_defaults_optional_fields() {
        let score = AbilityScore::new(Id::new("ability.awareness"), 3);
        assert_eq!(score.ability, Id::new("ability.awareness"));
        assert_eq!(score.score, 3);
        assert_eq!(score.specialty, None);
        assert_eq!(score.parameter, None);
        assert_eq!(score.banked_xp, 0);
    }

    #[test]
    fn art_score_new_defaults_banked_xp() {
        let score = ArtScore::new(Id::new("art.creo"), 5);
        assert_eq!(score.art, Id::new("art.creo"));
        assert_eq!(score.score, 5);
        assert_eq!(score.banked_xp, 0);
    }

    #[test]
    fn spell_selection_new_defaults_optional_fields() {
        let spell = SpellSelection::new(Id::new("spell.pilum_of_fire"));
        assert_eq!(spell.spell, Id::new("spell.pilum_of_fire"));
        assert_eq!(spell.level, None);
        assert_eq!(spell.mastery, None);
        assert_eq!(spell.parameter, None);
        assert!(spell.mastery_abilities.is_empty());
        assert!(!spell.within_focus);
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
        let s = SourceRef::new("f.md", LineRange::new(1, 2), "anchor");
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

    /// D10: absent `max_per_value` means ONCE, not "no ceiling" — the same
    /// inversion `max_total` gets, and for the same reason (`ArMDE:2814`).
    #[test]
    fn max_per_value_defaults_to_one_and_is_omitted_when_default() {
        let json = r#"{ "key": "ability", "type": "ref", "domain": "ability" }"#;
        let param: ParameterDef = serde_json::from_str(json).unwrap();
        assert_eq!(param.max_per_value, 1);
        let out = serde_json::to_string(&param).unwrap();
        assert!(
            !out.contains("max_per_value"),
            "default should be skipped: {out}"
        );
    }

    #[test]
    fn require_categories_defaults_to_empty_and_is_omitted_when_empty() {
        // Additive field: every ParameterDef already in `rules/` omits it, so it
        // must default to "no narrowing" AND stay out of canonical output, or the
        // shipped files would no longer re-serialize byte-identically.
        let json = r#"{ "key": "target", "type": "ref", "domain": "item" }"#;
        let param: ParameterDef = serde_json::from_str(json).unwrap();
        assert!(param.require_categories.is_empty());
        let out = serde_json::to_string(&param).unwrap();
        assert!(
            !out.contains("require_categories"),
            "empty require_categories should be skipped: {out}"
        );
    }

    #[test]
    fn require_categories_serializes_sorted() {
        // Canonical serialization: the set of required categories is unordered
        // data, so an authoring order must not survive into the output.
        let json = r#"{ "key": "target", "type": "ref", "domain": "item",
                        "require_categories": ["supernatural", "hermetic"] }"#;
        let param: ParameterDef = serde_json::from_str(json).unwrap();
        let out = serde_json::to_string(&param).unwrap();
        assert!(
            out.contains(r#""require_categories":["hermetic","supernatural"]"#),
            "{out}"
        );
    }

    #[test]
    fn require_possessed_and_forbid_tainted_default_false_and_are_omitted_when_false() {
        // Additive booleans, same contract as `require_categories` above: every
        // ParameterDef already in `rules/` omits them, so they must default to
        // "no restriction" AND stay out of canonical output, or the shipped
        // files would no longer re-serialize byte-identically.
        let json = r#"{ "key": "virtue", "type": "ref", "domain": "item" }"#;
        let param: ParameterDef = serde_json::from_str(json).unwrap();
        assert!(!param.require_possessed);
        assert!(!param.forbid_tainted);
        let out = serde_json::to_string(&param).unwrap();
        assert!(
            !out.contains("require_possessed") && !out.contains("forbid_tainted"),
            "false flags should be skipped: {out}"
        );
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

    /// B1/D21 (F-502): `Prereq::HasCategory` survives a `kind`-tagged JSON
    /// round-trip, matching `prereq_art_min_roundtrip`'s pattern for the
    /// existing scalar-payload variants.
    #[test]
    fn prereq_has_category_roundtrip() {
        let json = r#"{"kind": "has_category", "value": "social_status"}"#;
        let prereq: Prereq = serde_json::from_str(json).unwrap();
        assert_eq!(prereq, Prereq::HasCategory("social_status".into()));

        let reserialized = serde_json::to_string(&prereq).unwrap();
        let roundtripped: Prereq = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(prereq, roundtripped);
    }

    /// B1/D21 (F-355, F-542, F-511): the three category/ability-prohibition
    /// `Effect` variants survive a `type`-tagged JSON round-trip, on
    /// `might_effects_round_trip`'s pattern. (A fourth,
    /// `RestrictsAbilityCategoryToAbilities`, was removed by D63/B1c: no
    /// shipped data ever used it.)
    #[test]
    fn b1_category_and_ability_prohibition_effects_roundtrip() {
        let effects = vec![
            Effect::ForbidsAbilityCategory {
                category: AbilityCategory::Martial,
            },
            Effect::ForbidsItemCategory {
                category: "personality".into(),
            },
            Effect::ForbidsAbilities {
                abilities: std::collections::BTreeSet::from([
                    Id::new("ability.bargain"),
                    Id::new("ability.charm"),
                ]),
            },
        ];
        let json = serde_json::to_string(&effects).unwrap();
        let back: Vec<Effect> = serde_json::from_str(&json).unwrap();
        assert_eq!(effects, back);
        assert!(json.contains("\"type\":\"forbids_ability_category\""));
        assert!(json.contains("\"type\":\"forbids_item_category\""));
        assert!(json.contains("\"type\":\"forbids_abilities\""));
    }

    /// B1/D41: `CategoryCap.min`/`min_hard` are additive and independent of
    /// the existing ceiling fields — absent by default (an old cap's JSON is
    /// unaffected), present only when authored, matching `hard`'s own
    /// `skip_serializing_if` convention.
    #[test]
    fn category_cap_min_round_trip() {
        let bare = CategoryCap {
            category: "story".into(),
            max: 1,
            major_only: false,
            hard: false,
            min: None,
            min_hard: false,
            both_kinds: false,
        };
        let json = serde_json::to_string(&bare).unwrap();
        assert!(
            !json.contains("min"),
            "an absent floor must not appear: {json}"
        );

        let floored = CategoryCap {
            category: "social_status".into(),
            max: 1,
            major_only: false,
            hard: false,
            min: Some(1),
            min_hard: true,
            both_kinds: true,
        };
        let json = serde_json::to_string(&floored).unwrap();
        let back: CategoryCap = serde_json::from_str(&json).unwrap();
        assert_eq!(floored, back);
        assert!(json.contains("\"min\":1"));
        assert!(json.contains("\"min_hard\":true"));
    }
}
