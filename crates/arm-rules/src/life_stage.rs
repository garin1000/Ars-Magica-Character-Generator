//! Life-stage experience: the blocks of XP a character accumulates before play.
//!
//! A character's Abilities are bought with experience earned in blocks, not from
//! one undifferentiated bank: "For grogs and companions they are acquired in two
//! blocks: early childhood, and later life."
//! (Source: ArMDE:2364.) This module
//! turns an age into those blocks; the blocks then become funding pools for the
//! existing allocation solve in [`crate::effective::xp_allocation`], which is why
//! nothing here charges or spends anything itself.
//!
//! See `RULES.md` for the provenance of every value the shipped
//! `rules/core/life_stages.json` carries.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

use crate::effective::selections_for_effects;
use crate::ruleset::Ruleset;
use crate::types::{
    AbilityFunding, AbilityParameterValue, Effect, Entity, Id, SelectionParamValue, is_zero,
};

/// The life-stage experience rules, loaded from `rules/core/life_stages.json`.
// No `Default`: every field is authored data with no meaningful zero (a childhood
// of no years granting no experience is not a default, it is a broken file), and
// the ruleset holds these as an `Option` for the absent case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeStageRules {
    /// The magus's apprenticeship, when the ruleset ships one. Optional because
    /// `ArMDE:2364` calls apprenticeship and life as a magus "two **more** periods" — a
    /// ruleset with no Hermetic magi needs neither.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apprenticeship: Option<ApprenticeshipRules>,
    /// The first years of life, before any chosen advancement.
    pub childhood: ChildhoodRules,
    /// Every year after childhood, up to the character's age.
    pub later_life: LaterLifeRules,
    /// The years a magus lives after its Gauntlet, when the ruleset ships them.
    /// Optional for the same reason as [`Self::apprenticeship`]: `ArMDE:2364` calls
    /// apprenticeship and life as a magus "two **more** periods", so a ruleset
    /// with no Hermetic magi ships neither.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_apprenticeship: Option<PostApprenticeshipRules>,
}

/// Apprenticeship: the fixed block of years a magus spends being trained, and the
/// Abilities the Order expects at its end.
///
/// > The fifteen years of apprenticeship give the character 240 experience points,
/// > and 120 levels of spells. These experience points can be spent on Arts or
/// > Abilities, including Arcane, Academic, and Martial Abilities.
///
/// The **120 spell levels are deliberately not here**: they are the magus type
/// profile's `spell_levels` (`rules/core/character_types.json`), which
/// [`crate::effective::spell_levels_base`] is the single selector of. The two
/// numbers of `ArMDE:2435` live in two files on purpose; see `RULES.md`.
///
/// Source: ArMDE:2433-2437.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprenticeshipRules {
    /// The age a plan naming no Gauntlet age is read at — the rules' own baseline
    /// magus, "25 years old and just out of apprenticeship"
    /// (ArMDE:1601), which the Darius
    /// example builds too: apprenticed at 10 (`ArMDE:2402`) plus the fifteen years of
    /// `ArMDE:2435`, with his years as a magus then counted "from 26 to 33" (`ArMDE:2486`).
    ///
    /// Here rather than in Rust because it is a rule's number like `years` and `xp`:
    /// a ruleset whose magi are gauntleted at another age says so in its own file.
    /// Clamped to the character's age when it is read, so a magus too young to have
    /// reached the baseline still stands at its Gauntlet.
    ///
    /// `None` for a ruleset that states no baseline, which then keeps the reading
    /// that predates this field: a plan with no Gauntlet age means the magus stands
    /// at its Gauntlet. Additive, so no save and no ruleset migrates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_gauntlet_age: Option<u32>,
    /// Abilities the Order demands of every magus: "Magi must have the following
    /// minimum Abilities: Parma Magica 1, Magic Theory 1, Latin 1. Characters with
    /// lower scores would not be admitted to the Order." (`ArMDE:2437`.)
    pub minimum_abilities: Vec<AbilityRequirement>,
    /// The Abilities of `#### Hermetic Magi Recommended Minimum Abilities`
    /// (`ArMDE:2451-2461`) — advice, not admission, so a shortfall is a warning.
    pub recommended_abilities: Vec<AbilityRequirement>,
    /// What [`Self::recommended_abilities`] costs off the advancement table:
    /// "Total Cost: 90 experience points" (`ArMDE:2461`). Carried as data so the load
    /// can re-price the list against it — the trust gate on transcribed numbers.
    pub recommended_xp: u32,
    /// Experience the years of apprenticeship grant ("240 experience points",
    /// `ArMDE:2435`), spendable on Arts or Abilities alike.
    pub xp: u32,
    /// Years apprenticeship covers ("The fifteen years of apprenticeship", `ArMDE:2435`).
    pub years: u32,
    /// Experience a truncated apprenticeship (D56/D62/D3 — the Abandoned
    /// Apprentice, ArMDE:5641-5650) grants per year completed — FIXED at 16,
    /// derived from `xp`/`years` above (`240/15`, `decisions.md` D56 — "not
    /// to be reopened as a house rule"). Required (not `#[serde(default)]`):
    /// a ruleset shipping this block must state it explicitly, on the same
    /// footing as `xp`/`years` themselves, so the mandatory bound-consistency
    /// integrity check (`ruleset/integrity.rs`) always has a real number to
    /// verify against rather than a silently-defaulted one.
    pub truncated_xp_per_year: u32,
    /// Spell levels a truncated apprenticeship grants per year completed —
    /// FIXED at 8, derived from the magus type profile's `spell_levels`/
    /// `years` (`120/15`), the same `decisions.md` D56 derivation as
    /// [`Self::truncated_xp_per_year`]. Required for the identical reason.
    pub truncated_spell_levels_per_year: u32,
}

/// Life as a magus after the Gauntlet: what each year out of apprenticeship is
/// worth, and what a season of lab work costs against it.
///
/// > For every year, the magus gets 30 points. Each point can be an experience
/// > point in an Art or Ability or one level of spell.
///
/// Every field is counted in **points**, not experience: `ArMDE:2471` makes a point
/// fungible between an experience point and a level of spell, and the player
/// decides which each one becomes. Calling them "xp" would name only half of what
/// they buy — which is why this struct says `points_per_year` where
/// [`ApprenticeshipRules`] says `xp` (`ArMDE:2435` grants experience and spell levels
/// as two separate, non-interchangeable numbers).
///
/// Source: ArMDE:2467-2482.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostApprenticeshipRules {
    /// What one season of lab work costs the year that holds it: "the character
    /// loses 10 points from the yearly 30 experience points" (`ArMDE:2482`).
    pub lab_season_cost: u32,
    /// How many lab seasons in one year actually cost anything — the deduction
    /// stops "at a minimum of 0 if three or four seasons are spent on lab work"
    /// (`ArMDE:2482`), so a fourth season is free because there is nothing left to take.
    pub max_charged_lab_seasons_per_year: u32,
    /// The points one year out of apprenticeship grants: "For every year, the
    /// magus gets 30 points" (`ArMDE:2471`).
    pub points_per_year: u32,
}

/// An Ability score some rule demands, as data: which Ability, at what score, and
/// optionally at which instance.
///
/// Shaped like [`crate::ruleset::ScholarlyLanguageRequirement`] plus `parameter`,
/// and deliberately NOT unified with it: they live in different files and gate
/// different rules, so sharing a type would couple two unrelated edits.
///
/// `parameter` is `None` throughout the shipped data — "Latin 1" is matched by
/// Ability id, since an instance value is free-text player input with no
/// localization path (a German player types "Latein"). The match therefore stays
/// **deliberately wider than the rules' letter**, permanently: languages are
/// troupe-defined free text and the rulebook publishes no language list, so
/// this requirement stays instance-blind by design even after CV
/// (`docs/vf-audit/design-cv-catalogued-values.md`) adds a `language` catalogue
/// for literal instances elsewhere — narrowing to one instance here would
/// misstate ArMDE:7151/:2437's *any-qualifying-language* rule, not merely widen
/// an implementation gap. See `RULES.md`.
/// [`Self::exemplar`] is the honesty fix for that widening.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityRequirement {
    /// The Ability the requirement is about.
    pub ability: Id,
    /// One example instance the **rules themselves name**, as a language-neutral
    /// slug (`"latin"`), so a UI can show what the passage actually demands beside
    /// the wider check the engine enforces.
    ///
    /// A **label key, not a `ref`**: it names no catalogue entry, so the loader's
    /// referential-integrity pass deliberately does not resolve it (see
    /// `ruleset::integrity`). Its translated text lives in `rules/i18n/<lang>/` under
    /// `exemplar.<slug>`, keeping `rules/core/` free of translatable strings. It is
    /// one named example, never an enumeration.
    ///
    /// Source: ArMDE:2437 (Latin 1),
    /// `ArMDE:2455` (the recommended Latin 4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exemplar: Option<String>,
    /// The score it must reach.
    pub min_score: u8,
    /// The instance it must be, for a parameterized Ability. `None` accepts any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
}

/// Early childhood: a fixed block of years granting a native language and a
/// restricted spread of the Abilities a child picks up in play.
///
/// Source: ArMDE:2378.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildhoodRules {
    /// Years childhood covers ("the first five years of life").
    pub years: u32,
    /// The ability the native-language experience buys ("75 experience points in
    /// their native language (see page 167 for the Language Ability)"). Data rather
    /// than a hardcoded slug, so a ruleset naming its language ability differently
    /// still resolves; the id is checked at load like every other ref.
    pub native_language_ability: Id,
    /// Experience granted in the native language alone, spendable on nothing else
    /// ("75 experience points in their native language").
    pub native_language_xp: u32,
    /// Experience to divide between [`Self::spread_abilities`] ("and 45 experience
    /// points to divide between …").
    pub spread_xp: u32,
    /// The closed list the childhood spread may be spent on. Data, so a supplement
    /// widening childhood is a rules-file change: the engine only ever asks whether
    /// an ability is in the set. A `BTreeSet` so the file's order does not matter
    /// and serialization stays canonical.
    pub spread_abilities: BTreeSet<Id>,
}

/// Later life: experience per year from the end of childhood to the character's
/// age. The base rate here is modified per character by
/// [`crate::types::Effect::LaterLifeXpRate`] (Wealthy, Poor).
///
/// Source: ArMDE:2392.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaterLifeRules {
    /// Experience per year of later life ("15 experience points per year").
    pub xp_per_year: u32,
}

/// Which sort of demand an [`AbilityRequirement`] is: one the Order enforces, or one
/// the rulebook merely recommends.
///
/// An enum rather than a bool, so a third kind is a compile error until every reader
/// has decided what to do with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbilityRequirementKind {
    /// A minimum without which the character "would not be admitted to the Order"
    /// (ArMDE:2437) — an error.
    Required,
    /// One of the `#### Hermetic Magi Recommended Minimum Abilities` (`ArMDE:2451-2461`) —
    /// advice, so a warning.
    Recommended,
}

/// One row of a magus's Hermetic-minimums checklist: what is demanded, what the
/// character bought, and whether that satisfies it.
///
/// Derived, never stored. Produced by [`magus_minimum_abilities`], which is the
/// single reading of `ArMDE:2437`/`ArMDE:2451-2461` — both the validator and the effective
/// scores consume this, so the finding and the display cannot disagree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MagusMinimumAbility {
    /// The Ability demanded.
    pub ability: Id,
    /// One example instance the rules name for this requirement, carried straight
    /// through from [`AbilityRequirement::exemplar`] so the checklist and the
    /// validation finding state the same demand the same way — "Latin 1", with the
    /// wider check the engine really performs trailing the score as a note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exemplar: Option<String>,
    /// The instance demanded, when the requirement names one (`None` throughout the
    /// shipped data — see [`AbilityRequirement::parameter`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// The score demanded.
    pub min_score: u8,
    /// The highest score the character **bought** in a matching instance.
    pub score: u8,
    /// Whether `score` reaches `min_score`.
    pub met: bool,
    /// Whether falling short is an error or merely advice.
    pub requirement: AbilityRequirementKind,
}

/// The Hermetic minimum-Ability checklist for this character: the minimums of
/// `ArMDE:2437` first, then the recommendations of `ArMDE:2451-2461`, each in the order the
/// rules data declares them.
///
/// > Magi must have the following minimum Abilities: Parma Magica 1, Magic Theory 1,
/// > Latin 1. Characters with lower scores would not be admitted to the Order.
///
/// **The single reading of those two passages**, consumed by both
/// `validate_magus_minimum_abilities` and `EffectiveScores`, so the error and the
/// checklist a UI shows can never drift apart.
///
/// Empty for anyone who is not a magus — `ArMDE:2437` is about admission to the Order —
/// and for a ruleset shipping no apprenticeship block, which states no minimums.
/// Independent of the funding mode: `ArMDE:2437` is unconditional, so a magus built from a
/// flat experience pool is held to it exactly as a guided one is.
///
/// `score` is the highest **bought** score across the matching instances: a Virtue's
/// +2 to *use* (Puissant) is not training the Order can examine, and
/// [`crate::effective::effective_ability_score`] would report 2 for a magus with no
/// Parma row at all.
///
/// Source: ArMDE:2437, :2451-2461.
pub fn magus_minimum_abilities(entity: &Entity, ruleset: &Ruleset) -> Vec<MagusMinimumAbility> {
    let Some(apprenticeship) = ruleset
        .life_stages()
        .and_then(|rules| rules.apprenticeship_of(entity, ruleset))
    else {
        return Vec::new();
    };
    let kinds = [
        (
            &apprenticeship.minimum_abilities,
            AbilityRequirementKind::Required,
        ),
        (
            &apprenticeship.recommended_abilities,
            AbilityRequirementKind::Recommended,
        ),
    ];
    let mut rows = Vec::new();
    for (requirements, kind) in kinds {
        for requirement in requirements {
            let score = entity
                .ability_scores
                .iter()
                .filter(|bought| {
                    bought.ability == requirement.ability
                        && requirement.parameter.as_ref().is_none_or(|wanted| {
                            bought
                                .parameter
                                .as_ref()
                                .and_then(AbilityParameterValue::match_key)
                                == Some(wanted.as_str())
                        })
                })
                .map(|bought| bought.score)
                .max()
                .unwrap_or(0);
            rows.push(MagusMinimumAbility {
                ability: requirement.ability.clone(),
                exemplar: requirement.exemplar.clone(),
                parameter: requirement.parameter.clone(),
                min_score: requirement.min_score,
                score,
                met: score >= requirement.min_score,
                requirement: kind,
            });
        }
    }
    rows
}

/// A character's life-stage *choices* — never its resolved numbers.
///
/// Saves store choices, not derived values, so this records only what the player
/// decided: which language is native, and which Sample Childhood package (if any)
/// was applied. Every figure follows from these plus [`Entity::age`] and the
/// ruleset, so a rules edit re-derives an old save rather than leaving it with
/// stale totals.
///
/// The package's parameterized slot values are deliberately NOT recorded here:
/// they persist as the `parameter` of the Ability rows the package wrote, which is
/// where the engine reads them, so keeping a second copy would only let the two
/// representations diverge.
///
/// Its presence is **not** the switch between the two ways of buying Abilities — it
/// was until schema 16, and is not now. [`Entity::ability_funding`] is: under
/// [`crate::AbilityFunding::LifeStages`] the budget below is authoritative, and under
/// `Pool` this whole plan is inert while [`Entity::xp_pool`] pays. A plan may sit
/// beside a nonzero pool, and nothing double-counts, because [`LifeStageRules::budget`]
/// reads the mode.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LifeStagePlan {
    /// The language the character grew up speaking — the one the childhood's
    /// native-language experience may be spent on, and the one a second Living
    /// Language may not be ("Living Language (other than the character's native
    /// language)", ArMDE:2378). A `living_language` instance value, so it is
    /// the player's own text, not an id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_language: Option<String>,
    /// The Sample Childhood package the player took, if any — a **record of the
    /// decision**, nothing more. The Abilities it grants live in
    /// [`Entity::ability_scores`] as ordinary bought rows, exactly as a
    /// hand-divided 45 experience points would, so nothing is derived from this
    /// field.
    ///
    /// It is deliberately **not** cross-checked against those rows: "Note that you
    /// can spend the 45 experience points for yourself, as well"
    /// (ArMDE:2382) leaves a package open
    /// to adjustment after it has been taken, so a character whose scores no longer
    /// match the package is legal, not an error.
    ///
    /// `None` for a character who divided the childhood experience by hand — and for
    /// every save written before the packages existed, which is why the field is
    /// additive and needs no schema bump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub childhood_package: Option<Id>,
    /// The age at which the magus was gauntleted — **the one number this slice
    /// stores**. `age = childhood + later life + apprenticeship + post-Gauntlet` is
    /// a single equation in two unknowns (where apprenticeship starts, and how long
    /// ago it ended), so exactly one of them has to be recorded and every other
    /// figure derived from it.
    ///
    /// **`None` means the ruleset's baseline** —
    /// [`ApprenticeshipRules::default_gauntlet_age`], 25 in the shipped data ("25
    /// years old and just out of apprenticeship",
    /// ArMDE:1601), clamped to
    /// [`Entity::age`] so a younger magus still stands at its Gauntlet. It read as
    /// the age itself until that baseline existed, which made a blank field mean
    /// "zero years as a magus" — the least likely magus there is. The field stays
    /// additive: nothing is written here on the player's behalf, so a save records
    /// only what was actually chosen.
    ///
    /// Storing `post_gauntlet_years` instead was rejected: raising a magus's age
    /// would then stretch the *childhood-to-apprenticeship* span — the years before
    /// it was taken as an apprentice — rather than its years as a magus, which is the
    /// opposite of what a player raising the age means.
    ///
    /// Source: ArMDE:2216, :2467-2471.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gauntlet_age: Option<u32>,
    /// Lab seasons **charged** against the yearly 30 points, totalled across every
    /// post-Gauntlet year: "the character loses 10 points from the yearly 30
    /// experience points, to a minimum of 0 if three or four seasons are spent on lab
    /// work" (ArMDE:2482).
    ///
    /// One total rather than a season list per year, because the arithmetic cannot
    /// tell the difference: the deduction stops at the third season of any year, so
    /// every legal per-year distribution totals at most `3 × years`, every total in
    /// that range is realizable by some distribution, and all of them cost the same.
    ///
    /// It counts **charged** seasons, not seasons actually worked — the fourth season
    /// of a year is free (`ArMDE:2482` has already reached 0 by the third), so 16 seasons
    /// worked costs nothing across four years but 30 points across five, and a single
    /// total of *worked* seasons could not tell those two apart.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub post_gauntlet_lab_seasons: u32,
    /// How many of the post-Gauntlet points the player took as **levels of spells**
    /// rather than experience: "Each point can be an experience point in an Art or
    /// Ability or one level of spell" (ArMDE:2471). The rest are experience.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub post_gauntlet_spell_levels: u32,
}

/// The experience a life-stage plan earns, split into the blocks the rules grant
/// it in. Derived — never stored (see [`LifeStagePlan`]).
///
/// The blocks fund different things, which is the whole reason they are kept apart
/// rather than summed: the native-language points buy one language and nothing else,
/// the spread buys only the childhood Abilities, later life buys anything the
/// character is permitted, and the years after a magus's Gauntlet buy Arts,
/// Abilities or levels of spells alike — which is why that block alone is reported
/// as points *and* as the experience/spell-level split the player chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeStageBudget {
    /// Experience for the native language alone (75).
    pub childhood_native_xp: u32,
    /// Experience for the childhood spread (45).
    pub childhood_spread_xp: u32,
    /// Years of later life lived (age − childhood years) — for a character
    /// funding a truncated apprenticeship (D64), the PRE-span alone (before
    /// the truncated block's hypothetical start), since the years after it
    /// are a separate field ([`Self::truncated_training_post_span_years`])
    /// with different eligibility (Arts as well as Abilities).
    pub later_life_years: u32,
    /// Experience earned per year of later life for this character (15/20/10).
    pub later_life_rate: u32,
    /// Experience from later life (`later_life_years × later_life_rate`) —
    /// Abilities-only and restricted, exactly like a real magus's own
    /// pre-apprenticeship later life ([`crate::effective`]'s
    /// `magus_later_life_pool`, keyed on `is_hermetically_trained`
    /// generically, not on a real magus profile).
    pub later_life_xp: u32,
    /// Years of apprenticeship served (15 for a magus, 0 for anyone else).
    pub apprenticeship_years: u32,
    /// Experience from apprenticeship (240 for a magus, 0 for anyone else) — the
    /// **general** pool, since it alone may buy Arts as well as Abilities
    /// (ArMDE:2435).
    ///
    /// The pool the solve actually funds from is this plus any
    /// [`Effect::GeneralXp`] (Skilled/Weak Parens), surfaced as
    /// `EffectiveScores::xp_general_pool`. This field is the block's base, which is
    /// why nothing displays it as "the pool".
    pub apprenticeship_xp: u32,
    /// The age this character was gauntleted at: [`LifeStagePlan::gauntlet_age`] when
    /// the plan names one, otherwise the ruleset's
    /// [`ApprenticeshipRules::default_gauntlet_age`] — or, for anyone serving no
    /// apprenticeship, simply the age later life runs to. Clamped to the age (so a
    /// magus younger than the baseline stands at its Gauntlet), and 0 while the age
    /// is unset, like every other age-dependent figure here.
    pub gauntlet_age: u32,
    /// Years lived after the Gauntlet (age − [`Self::gauntlet_age`]), 0 for a
    /// character standing at it (ArMDE:2216).
    pub post_gauntlet_years: u32,
    /// What those years grant, lab work deducted: `post_gauntlet_years × 30 −
    /// charged lab seasons × 10` (ArMDE:2471, :2482).
    ///
    /// **Points, not experience:** each one is "an experience point in an Art or
    /// Ability or one level of spell" (`ArMDE:2471`), so this is the sum of the two fields
    /// below and never itself funds a pool.
    pub post_gauntlet_points: u32,
    /// How many of [`Self::post_gauntlet_points`] the player took as levels of spells
    /// (`ArMDE:2471`) — not experience, so `total()` excludes them.
    pub post_gauntlet_spell_levels: u32,
    /// The rest of [`Self::post_gauntlet_points`], which is experience.
    pub post_gauntlet_xp: u32,
    /// Years of a truncated apprenticeship this character completed before
    /// abandonment (D56/D62/D3) — `0` for anyone not carrying an effective
    /// [`crate::types::Effect::TruncatedApprenticeshipXp`] selection whose
    /// sibling [`crate::types::Effect::ConfersHermeticTrainingIf`] gate has
    /// resolved, a real magus included. Distinct from
    /// [`Self::apprenticeship_years`], which is a real magus's own
    /// apprenticeship alone.
    pub truncated_training_years: u32,
    /// Experience the truncated years grant, at the FIXED per-year rate
    /// (`ApprenticeshipRules::truncated_xp_per_year`, D56 — 16/year) — folded
    /// into the **general** pool alongside [`Self::apprenticeship_xp`]/
    /// [`Self::post_gauntlet_xp`] (`effective/xp.rs::general_pool_and_bonus`).
    pub truncated_training_xp: u32,
    /// Spell levels the truncated years grant, at the FIXED per-year rate
    /// (`ApprenticeshipRules::truncated_spell_levels_per_year`, D56 —
    /// 8/year), folded into `effective/spell.rs::life_stage_spell_levels`.
    pub truncated_training_spell_levels: u32,
    /// D64: years lived AFTER the truncated block, up to the character's own
    /// age — `age − (start + truncated_training_years)`, where `start` is
    /// the apprenticeship's hypothetical beginning
    /// ([`LifeStageRules::truncated_apprenticeship_start`]). `0` for anyone
    /// not funding a truncated apprenticeship.
    pub truncated_training_post_span_years: u32,
    /// Experience the post-span years grant, at the ordinary later-life rate
    /// — but **general** (Arts or Abilities alike, D64: "the character
    /// 'knows Hermetic magic' with his Arts already opened", ArMDE:5643),
    /// unlike the PRE-span's Abilities-only [`Self::later_life_xp`]. Folded
    /// into the general pool alongside [`Self::truncated_training_xp`]
    /// (`effective/xp.rs::general_pool_and_bonus`).
    pub truncated_training_post_span_xp: u32,
    // Deliberately NOT a field: what a post-Gauntlet year is worth. Unlike
    // `later_life_rate`, which Wealthy and Poor change per character (`ArMDE:2394`), the 30
    // of `ArMDE:2471` never varies — so there is nothing per-character to report, and a UI
    // that shows the rate reads it off the ruleset's `post_apprenticeship` block,
    // which is the one place it lives.
}

impl LifeStageBudget {
    /// Every **experience point** the character has earned, across every block.
    ///
    /// [`Self::post_gauntlet_spell_levels`] is excluded on purpose: a level of spell
    /// is not experience — `ArMDE:2471` has the player split the yearly points between the
    /// two — and counting it here would spend it twice, once as experience and once
    /// against the spell-levels budget.
    pub fn total(&self) -> u32 {
        self.childhood_native_xp
            .saturating_add(self.childhood_spread_xp)
            .saturating_add(self.later_life_xp)
            .saturating_add(self.apprenticeship_xp)
            .saturating_add(self.post_gauntlet_xp)
    }
}

/// A block of life-stage experience that funds purchases on its own terms.
///
/// A fixed taxonomy (the rules grant exactly these), so an enum: adding a block is
/// a compile error until the UI labels it.
///
/// **A real magus's own apprenticeship is absent, and later life is present.**
/// Whichever block funds anything the character may learn is the *general*
/// pool and needs no slug: for a magus that is apprenticeship, whose
/// experience "can be spent on Arts or Abilities" (ArMDE:2435). Later life
/// buys "any **Abilities**" (`ArMDE:2214`, `ArMDE:2392`) and, for a magus,
/// ends where apprenticeship begins — so it is a restricted pool of its own,
/// listed here. For a grog or companion later life is still the general pool;
/// the enum names the blocks that *can* be restricted, and which pools a
/// character actually gets is decided in [`crate::effective::xp_allocation`].
///
/// [`Self::Apprenticeship`] (D40/D2) is a DIFFERENT thing from a real magus's
/// own apprenticeship, which is why the paragraph above still holds: it is a
/// replacement-effect-driven, apprenticeship-*shaped* restricted pool for a
/// character who is NOT hermetically trained by profile (Redcap, Lone
/// Redcap), so unlike a magus's own it does need a slug — it does not fund
/// anything the character may learn (Arts included), only Abilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifeStageBlock {
    /// Childhood's native-language experience: spendable only on the native
    /// language instance (ArMDE:2378).
    ChildhoodNativeLanguage,
    /// Childhood's restricted spread: spendable only on the childhood Ability list,
    /// and never on the native language (`ArMDE:2378`). Also carries a Feral
    /// Upbringing character's whole-childhood replacement (D40/D2): the same
    /// tag, reused rather than a fourth one, since the replacement pool is
    /// exactly this shape (no native-language carve-out) with a different
    /// eligibility list.
    ChildhoodSpread,
    /// Later life: for a magus, the years before apprenticeship, spendable on
    /// Abilities alone and never on an Art (`ArMDE:2214`, `ArMDE:2392`).
    LaterLife,
    /// A replacement-effect-driven, apprenticeship-shaped block for a
    /// character who is NOT hermetically trained by profile (Redcap, Lone
    /// Redcap) — distinct from a real magus's own apprenticeship, which
    /// needs no slug of its own (it folds straight into the general pool;
    /// see [`LifeStageBudget::apprenticeship_xp`]'s own doc comment). D40/D2.
    Apprenticeship,
}

impl LifeStageBlock {
    /// Every block, the single source of the set (the UI's labels are checked
    /// against it).
    pub const ALL: [LifeStageBlock; 4] = [
        LifeStageBlock::ChildhoodNativeLanguage,
        LifeStageBlock::ChildhoodSpread,
        LifeStageBlock::LaterLife,
        LifeStageBlock::Apprenticeship,
    ];
}

impl fmt::Display for LifeStageBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LifeStageBlock::ChildhoodNativeLanguage => "childhood_native_language",
            LifeStageBlock::ChildhoodSpread => "childhood_spread",
            LifeStageBlock::LaterLife => "later_life",
            LifeStageBlock::Apprenticeship => "apprenticeship",
        })
    }
}

/// D40/D2's years-carving replacement (Redcap, Lone Redcap): the largest
/// `years` named by any effective selection's `Effect::ReplacesLifeStageXp{
/// stage: Apprenticeship, years, ..}` — 0 when none carries one. **Not**
/// widened by D3/D64 (an earlier design draft planned to; superseded — see
/// `docs/vf-audit/design-d0-xp-modes.md` Revision 5): a truncated
/// apprenticeship carves a gap out of the MIDDLE of later life, leaving a
/// pre-span and a post-span, which this single "years to remove from the
/// end" helper cannot express — [`truncated_apprenticeship_start`] and
/// [`truncated_apprentice_years_completed`] are D3's own, separate
/// machinery. `pub(crate)`: also read by
/// `validation/life_stage.rs::validate_life_stage_age_meets_minimum`'s
/// Redcap-shaped branch.
pub(crate) fn extra_apprenticeship_years(entity: &Entity, ruleset: &Ruleset) -> u32 {
    selections_for_effects(entity, ruleset)
        .iter()
        .filter_map(|selection| ruleset.point_items.get(&selection.item_ref))
        .flat_map(|item| &item.effects)
        .filter_map(|effect| match effect {
            Effect::ReplacesLifeStageXp {
                stage: LifeStageBlock::Apprenticeship,
                years,
                ..
            } => Some(*years),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

/// D3/D64: the truncated apprenticeship's own `years_completed`, once BOTH
/// halves of F1's paired gate resolve on the SAME item — a
/// `Effect::TruncatedApprenticeshipXp{param}` alongside a
/// `Effect::ConfersHermeticTrainingIf{param}` naming the IDENTICAL `param`.
/// `None` while unanswered (F1's own "never a silent zero" — a caller must
/// treat `None` as "not a truncated apprentice", never as 0 years), and
/// `None` for a real magus even if one somehow carried the effect (callers
/// additionally gate on [`LifeStageRules::apprenticeship_of`] returning
/// `None`, the D56 asymmetry).
pub(crate) fn truncated_apprentice_years_completed(
    entity: &Entity,
    ruleset: &Ruleset,
) -> Option<u32> {
    selections_for_effects(entity, ruleset)
        .iter()
        .filter_map(|selection| {
            let item = ruleset.point_items.get(&selection.item_ref)?;
            let param = item.effects.iter().find_map(|effect| match effect {
                Effect::TruncatedApprenticeshipXp { param } => Some(param),
                _ => None,
            })?;
            let gate_resolved = item.effects.iter().any(|effect| {
                matches!(effect, Effect::ConfersHermeticTrainingIf { param: p } if p == param)
            });
            if !gate_resolved {
                return None;
            }
            selection
                .params
                .get(param)
                .and_then(SelectionParamValue::as_single)
                .and_then(|v| v.as_str().parse::<u32>().ok())
        })
        .next()
}

impl LifeStageRules {
    /// The experience this character has earned through its life stages, or `None`
    /// when it is not funded by them — where [`Entity::xp_pool`] is the authority.
    ///
    /// **The mode is read off [`Entity::ability_funding`], not off the plan's
    /// presence.** Since schema 16 a pool-funded character may keep a life-stage plan
    /// on file (switching funding mode no longer destroys the side switched away
    /// from), and such a plan is inert: it earns nothing. Reading its presence as the
    /// mode — which is what every reader did before 16 — would fund that character
    /// twice. `None` therefore covers two shapes: pool funding, and life-stage funding
    /// with no plan recorded yet.
    ///
    /// This is the single funnel the mode-sensitive engine paths go through — the XP
    /// allocation's restricted pools and general pool
    /// ([`crate::effective::xp_allocation`]), the spell-level budget, and the
    /// life-stage validators all ask this function rather than the entity's `Option`.
    ///
    /// An **unset age** still yields a budget: childhood is granted "in the first
    /// five years of life" with no further condition
    /// (ArMDE:2378), so only later life
    /// scales with an age and an ageless plan simply lives 0 later-life years.
    /// Withholding the whole budget instead would leave childhood's two restricted
    /// pools at nothing and report every childhood row as unfunded — the validator
    /// names the missing age itself (`life_stage_age_unset`).
    ///
    /// **The age is spent around the Gauntlet, not up to it.** The years before the
    /// Gauntlet are childhood, later life and apprenticeship; the years after it earn
    /// "30 points per year" (`ArMDE:2216`, `ArMDE:2471`). So every figure here is derived from
    /// the *Gauntlet* age — [`LifeStagePlan::gauntlet_age`], or the ruleset's
    /// [`ApprenticeshipRules::default_gauntlet_age`] when the plan names none — and
    /// raising a magus's age lengthens its life as a magus, never the
    /// childhood-to-apprenticeship span behind it.
    pub fn budget(&self, entity: &Entity, ruleset: &Ruleset) -> Option<LifeStageBudget> {
        match entity.ability_funding {
            AbilityFunding::Pool => return None,
            AbilityFunding::LifeStages => {}
        }
        let plan = entity.life_stages.as_ref()?;
        let apprenticeship = self.apprenticeship_of(entity, ruleset);
        // F2 (`docs/vf-audit/design-d0-xp-modes.md` § 3): sourced ONLY from a
        // real magus's own block — this is what the struct field below and
        // `apprenticeship_xp` both read, and it must keep meaning exactly
        // what its doc comment states ("15 for a magus, 0 for anyone else")
        // for a D2/D3 carrier too.
        let magus_apprenticeship_years = apprenticeship.map_or(0, |block| block.years);
        // A SEPARATE, struct-field-BLIND local: widened by an
        // apprenticeship-shaped `ReplacesLifeStageXp` replacement (Redcap,
        // Lone Redcap), which carves its own years out of later life exactly
        // as a real magus's apprenticeship does. `.max()`, not
        // `.saturating_add()` — mutually exclusive in practice, since
        // nothing in the catalogue lets a character be both. Feeds ONLY
        // `later_life_years` below.
        let later_life_carve_years =
            magus_apprenticeship_years.max(extra_apprenticeship_years(entity, ruleset));
        // D3/D64: a truncated apprenticeship (Abandoned Apprentice) — mutually
        // exclusive with the Redcap-shaped carve above in practice (nothing
        // in the catalogue grants both), and with a real magus's own
        // apprenticeship (D56's asymmetry: `apprenticeship_of` stays
        // profile-only, so a real magus never resolves here even if some
        // future data mistakenly granted him the effect).
        let years_completed = truncated_apprentice_years_completed(entity, ruleset)
            .filter(|_| apprenticeship.is_none());
        // "**Hermetic Magi Only (Optional):** Years after apprenticeship"
        // (ArMDE:2216), so the stored
        // Gauntlet age is read for a character that serves an apprenticeship and
        // ignored on every other plan — gating on the *points* being zero instead
        // would let a hand-edited companion plan carrying one cut its later life
        // short.
        //
        // A plan that names no Gauntlet age falls back to the ruleset's own baseline
        // ([`ApprenticeshipRules::default_gauntlet_age`], 25 in the shipped data:
        // "25 years old and just out of apprenticeship",
        // ArMDE:1601) rather than to the
        // character's age, which would have read a blank field as "zero years as a
        // magus" — the least likely magus there is. Only a character that serves an
        // apprenticeship reaches the fallback at all, so a companion still runs its
        // later life to its own age.
        //
        // Clamped to the age either way, because Advisory and Silent validation do
        // not block a Gauntlet after the character's own age and an unclamped value
        // would grant later-life years never lived. The clamp is also why the
        // baseline reaches no validator: a magus younger than it simply stands at its
        // Gauntlet, exactly as it did before.
        let gauntlet_age = entity.age.map_or(0, |age| {
            apprenticeship
                .and_then(|block| plan.gauntlet_age.or(block.default_gauntlet_age))
                .map_or(age, |gauntlet| gauntlet.min(age))
        });
        let later_life_rate = self.later_life_rate(entity, ruleset);
        // D3/D64: when a truncated apprenticeship is active, `later_life_years`
        // becomes the PRE-span alone (childhood to the hypothetical `start`,
        // Abilities-only) rather than running to `gauntlet_age`/the
        // character's own age — the years after `start + years_completed`
        // are a separate, GENERAL-funding post-span, computed alongside the
        // truncated block below. Every other character (real magus, Redcap-
        // shaped, ordinary companion) keeps today's single merged span,
        // unaffected.
        let (later_life_years, post_span_years) = match years_completed.and_then(|years| {
            self.truncated_apprenticeship_start(plan)
                .map(|start| (years, start))
        }) {
            Some((years, start)) => {
                let pre = self.later_life_years(start, 0);
                let post = entity
                    .age
                    .unwrap_or(0)
                    .saturating_sub(start.saturating_add(years));
                (pre, post)
            }
            None => (
                self.later_life_years(gauntlet_age, later_life_carve_years),
                0,
            ),
        };
        let post_gauntlet_years = entity.age.unwrap_or(0).saturating_sub(gauntlet_age);
        let post_gauntlet_points = self.post_gauntlet_points(plan, post_gauntlet_years);
        // "Each point can be an experience point in an Art or Ability or one level of
        // spell" (`ArMDE:2471`): the player's split, held to the points that exist so a
        // stored figure the years cannot pay for takes nothing away from the rest
        // (the validator reports it).
        let post_gauntlet_spell_levels = plan.post_gauntlet_spell_levels.min(post_gauntlet_points);
        // D56/D3: the truncated block's own 16/8-per-year rates — FIXED data
        // on the ruleset (`ApprenticeshipRules::truncated_xp_per_year`/
        // `truncated_spell_levels_per_year`), `saturating_mul`'d exactly like
        // `ScaledRestrictedAbilityXp`'s own precedent (F3,
        // `effective/xp.rs::restricted_ability_xp_pools`) so a crafted save's
        // absurd `years_completed` cannot panic under `overflow-checks =
        // true`. `0` unless BOTH a resolved `years_completed` AND an
        // `apprenticeship` block exist — R3-3's integrity check guarantees
        // the latter whenever the former is possible in a loaded ruleset.
        let (truncated_training_xp, truncated_training_spell_levels) =
            match (years_completed, self.apprenticeship.as_ref()) {
                (Some(years), Some(rules)) => (
                    rules.truncated_xp_per_year.saturating_mul(years),
                    rules.truncated_spell_levels_per_year.saturating_mul(years),
                ),
                _ => (0, 0),
            };
        Some(LifeStageBudget {
            childhood_native_xp: self.childhood.native_language_xp,
            childhood_spread_xp: self.childhood.spread_xp,
            later_life_years,
            later_life_rate,
            later_life_xp: later_life_years.saturating_mul(later_life_rate),
            apprenticeship_years: magus_apprenticeship_years,
            // Apprenticeship is a fixed block like childhood, so it does not scale
            // with an age; 0 for anyone who serves none.
            apprenticeship_xp: apprenticeship.map_or(0, |block| block.xp),
            gauntlet_age,
            post_gauntlet_years,
            post_gauntlet_points,
            post_gauntlet_spell_levels,
            post_gauntlet_xp: post_gauntlet_points.saturating_sub(post_gauntlet_spell_levels),
            truncated_training_years: years_completed.unwrap_or(0),
            truncated_training_xp,
            truncated_training_spell_levels,
            truncated_training_post_span_years: post_span_years,
            truncated_training_post_span_xp: post_span_years.saturating_mul(later_life_rate),
        })
    }

    /// What `post_gauntlet_years` out of apprenticeship are worth to this plan:
    /// "For every year, the magus gets 30 points" (`ArMDE:2471`), less what its lab work
    /// took — "For each season that your magus spends working on a lab project, the
    /// character loses 10 points from the yearly 30 experience points, to a minimum
    /// of 0 if three or four seasons are spent on lab work" (`ArMDE:2482`).
    ///
    /// The stored season total is capped at `max_charged × years` first, which is that
    /// per-year minimum of 0 read across the whole span: no year can lose more than
    /// three seasons' worth, so no span can. Everything saturates, so a stored total
    /// beyond the cap costs the same as the cap rather than underflowing — a validator
    /// reports it instead.
    ///
    /// 0 throughout for a ruleset shipping no `post_apprenticeship` block: the rate
    /// is data, so with no block there is no number to grant.
    ///
    /// Source: ArMDE:2471, :2482.
    fn post_gauntlet_points(&self, plan: &LifeStagePlan, post_gauntlet_years: u32) -> u32 {
        let Some(rules) = self.post_apprenticeship.as_ref() else {
            return 0;
        };
        let charged = charged_lab_seasons(
            plan.post_gauntlet_lab_seasons,
            post_gauntlet_years,
            rules.max_charged_lab_seasons_per_year,
        );
        post_gauntlet_years
            .saturating_mul(rules.points_per_year)
            .saturating_sub(charged.saturating_mul(rules.lab_season_cost))
    }
}

/// The seasons in a year, all of which a magus may spend in the lab: "to a minimum
/// of 0 if three or four seasons are spent on lab work" (`ArMDE:2482`).
pub const SEASONS_PER_YEAR: u32 = 4;

/// How many of `seasons` post-Gauntlet lab seasons cost points over `years`.
///
/// Norbert 2026-10-03 (F1): the stored total is read as packed into full lab
/// years, as `ArMDE:2482` recommends ("it is most cost effective to have the magus
/// engage in a full year of lab work at a time"). Each full year of four seasons
/// charges `max_per_year`, the remainder charges one per season up to that same
/// ceiling, and the span holds at most four seasons a year. Recording seasons per
/// year instead is planned (it needs a save-format change).
fn charged_lab_seasons(seasons: u32, years: u32, max_per_year: u32) -> u32 {
    let seasons = seasons.min(SEASONS_PER_YEAR.saturating_mul(years));
    let full_years = seasons / SEASONS_PER_YEAR;
    let remainder = seasons % SEASONS_PER_YEAR;
    full_years
        .saturating_mul(max_per_year)
        .saturating_add(remainder.min(max_per_year))
}

impl LifeStageRules {
    /// Years of later life lived up to `stop_age`: every year after childhood, minus
    /// the `apprenticeship_years` that follow it (0 for anyone who serves no
    /// apprenticeship). Both blocks are fixed spans, so a `stop_age` inside them
    /// yields 0 rather than a negative one (the validator reports such an age
    /// separately).
    ///
    /// **A magus's later life ends where its apprenticeship begins.** "**Later
    /// Life.** 15 experience points per year (until apprenticeship for magi)"
    /// (ArMDE:2214), the same four periods
    /// `ArMDE:2364` sets out. Counting every year to a magus's age would over-grant, and
    /// since Abilities and Arts buy from one shared pool the surplus would fund Arts
    /// as well.
    ///
    /// `stop_age` is therefore the age at which later life **stops**, not
    /// necessarily the character's own: the **Gauntlet age** for a magus — which is
    /// its age only while it stands at its Gauntlet — and the age itself for anyone
    /// else, who never leaves later life. That is why a magus of 60 gauntleted at 25
    /// has the same five later-life years as one of 25, exactly the Darius example's
    /// "75 experience points to spend from those five years" (`ArMDE:2402`) for a boy
    /// apprenticed at 10. The years after the Gauntlet are a block of their own
    /// (`ArMDE:2216`, `ArMDE:2471`), counted in [`Self::budget`].
    pub fn later_life_years(&self, stop_age: u32, apprenticeship_years: u32) -> u32 {
        stop_age.saturating_sub(self.childhood.years.saturating_add(apprenticeship_years))
    }

    /// D64: the truncated apprenticeship's hypothetical starting age — the
    /// Gauntlet age a real apprenticeship would have reached (the plan's own
    /// [`LifeStagePlan::gauntlet_age`], else the ruleset's
    /// [`ApprenticeshipRules::default_gauntlet_age`]), minus
    /// [`ApprenticeshipRules::years`]. Deliberately **not** clamped to the
    /// character's own age, unlike a real magus's own Gauntlet age
    /// ([`Self::budget`]'s `gauntlet_age` local): this is a hypothetical
    /// milestone the character never reached, not an event they lived
    /// through, so clamping it would let a young Abandoned Apprentice's
    /// "start" silently collapse toward their own age instead of correctly
    /// reporting an impossible timeline.
    ///
    /// `None` when the ruleset ships no `apprenticeship` block, or the block
    /// and the plan both name no Gauntlet age — R3-3's mandatory integrity
    /// check refuses any ruleset shipping an item that needs this while
    /// shipping no block at all, so `None` here means "not applicable to
    /// this ruleset", never "silently 0".
    pub(crate) fn truncated_apprenticeship_start(&self, plan: &LifeStagePlan) -> Option<u32> {
        let rules = self.apprenticeship.as_ref()?;
        let gauntlet = plan.gauntlet_age.or(rules.default_gauntlet_age)?;
        Some(gauntlet.saturating_sub(rules.years))
    }

    /// The apprenticeship this character serves, if any: the block for a magus, and
    /// `None` for anyone else — a grog or companion serves no apprenticeship at all,
    /// and a ruleset shipping no block declares none (`Ruleset::validate_integrity`
    /// refuses that combination for a ruleset that declares magi).
    ///
    /// Read off the type profile's `hermetically_trained` flag alone, never a type
    /// id, and — deliberately — never the entity-level union
    /// (`effective/hermetic_training.rs::is_hermetically_trained`) either.
    /// **Profile-only, on purpose (D56/A0):** an Abandoned Apprentice is trained by
    /// selection but "never able to complete [his] training" — he has NOT served
    /// (and completed) an apprenticeship, so `apprenticeship.minimum_abilities` must
    /// not be enforced against him. Reading the union here would be exactly the bug
    /// D56 forbids; see `docs/vf-audit/design-a0-is-magus-split.md` § 4 row 9 and § 7
    /// risk 4.
    pub(crate) fn apprenticeship_of(
        &self,
        entity: &Entity,
        ruleset: &Ruleset,
    ) -> Option<&ApprenticeshipRules> {
        if !ruleset
            .profile(&entity.type_id)
            .is_some_and(|profile| profile.hermetically_trained)
        {
            return None;
        }
        self.apprenticeship.as_ref()
    }

    /// The youngest a magus can be gauntleted: childhood plus the fifteen years of
    /// apprenticeship (ArMDE:2435), so twenty against the shipped data.
    ///
    /// A floor on the **Gauntlet age**, which is a floor on the character's age too:
    /// the years after the Gauntlet (`ArMDE:2216`) run forward from it, so a magus is at
    /// least this old whether it stands at its Gauntlet or is a century past it.
    ///
    /// Falls back to childhood alone for a ruleset shipping no apprenticeship block —
    /// there is then no further span to clear.
    pub fn minimum_gauntlet_age(&self) -> u32 {
        self.childhood.years.saturating_add(
            self.apprenticeship
                .as_ref()
                .map_or(0, |apprenticeship| apprenticeship.years),
        )
    }

    /// Experience per year of later life for this character: the ruleset's base
    /// rate, unless a selection replaces it ([`Effect::LaterLifeXpRate`] — Wealthy
    /// 20, Poor 10).
    ///
    /// The rate is replaced, not adjusted, because the rules state it whole. If
    /// several selections name a rate — which the shipped data prevents, since
    /// Wealthy and Poor are the only two and both are Major General — the lowest
    /// wins: nothing in the text ranks them, so the engine takes the conservative
    /// reading rather than depending on declaration order.
    ///
    /// A Guild Apprentice is "not able to benefit from either the Poor Flaw or
    /// the Wealthy Virtue … until he moves to the journeyman stage" (ArMDE:4041-4044,
    /// D47) — so any named rate is suppressed outright for a holder of
    /// [`Effect::SuppressesLaterLifeXpRate`] (data, not a hardcoded id), falling
    /// straight through to the base rate. This is a single narrow effect (D47
    /// explicitly rejects a general "nullify any effect" mechanism for one
    /// caller), not an `incompatible_with` (the book neutralises rather than
    /// forbids holding both).
    ///
    /// Source: ArMDE:2392, :2394.
    pub fn later_life_rate(&self, entity: &Entity, ruleset: &Ruleset) -> u32 {
        let selections = selections_for_effects(entity, ruleset);
        let suppressed = selections.iter().any(|selection| {
            ruleset
                .point_items
                .get(&selection.item_ref)
                .is_some_and(|item| {
                    item.effects
                        .iter()
                        .any(|e| matches!(e, Effect::SuppressesLaterLifeXpRate))
                })
        });
        if suppressed {
            return self.later_life.xp_per_year;
        }
        // The base rate is not one of the candidates: a named rate replaces it
        // outright, so `min` is taken over the named rates only (or the base stands
        // when nothing names one). Folding the base in would make Wealthy's 20 lose
        // to it.
        let mut named: Option<u32> = None;
        for selection in selections.iter() {
            let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
                continue;
            };
            for effect in &item.effects {
                if let Effect::LaterLifeXpRate { amount } = effect {
                    named = Some(named.map_or(*amount, |current: u32| current.min(*amount)));
                }
            }
        }
        named.unwrap_or(self.later_life.xp_per_year)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ruleset::Ruleset;
    use crate::types::{
        AbilityFunding, Entity, EntityKind, Id, RulesetRef, SCHEMA_VERSION, Selection,
    };

    /// The shipped file's shape, so a rename or a retype fails here.
    const SHIPPED: &str = r#"{
      "childhood": {
        "years": 5,
        "native_language_ability": "ability.living_language",
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": ["ability.athletics", "ability.swim"]
      },
      "later_life": { "xp_per_year": 15 }
    }"#;

    /// The same shape plus the two magus-only periods — apprenticeship, and the
    /// years lived after the Gauntlet — so both optional blocks parse with every
    /// field the rules state.
    const SHIPPED_WITH_APPRENTICESHIP: &str = r#"{
      "apprenticeship": {
        "years": 15,
        "xp": 240,
        "default_gauntlet_age": 25,
        "minimum_abilities": [
          { "ability": "ability.dead_language", "min_score": 1 },
          { "ability": "ability.magic_theory", "min_score": 1 },
          { "ability": "ability.parma_magica", "min_score": 1 }
        ],
        "recommended_abilities": [
          { "ability": "ability.artes_liberales", "min_score": 1 },
          { "ability": "ability.dead_language", "min_score": 4 },
          { "ability": "ability.magic_theory", "min_score": 3 },
          { "ability": "ability.parma_magica", "min_score": 1 }
        ],
        "recommended_xp": 90,
        "truncated_xp_per_year": 16,
        "truncated_spell_levels_per_year": 8
      },
      "childhood": {
        "years": 5,
        "native_language_ability": "ability.living_language",
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": ["ability.athletics", "ability.swim"]
      },
      "later_life": { "xp_per_year": 15 },
      "post_apprenticeship": {
        "lab_season_cost": 10,
        "max_charged_lab_seasons_per_year": 3,
        "points_per_year": 30
      }
    }"#;

    fn rules() -> LifeStageRules {
        serde_json::from_str(SHIPPED).expect("the shipped life-stage shape parses")
    }

    /// The same rules with the magus's apprenticeship block declared.
    fn rules_with_apprenticeship() -> LifeStageRules {
        serde_json::from_str(SHIPPED_WITH_APPRENTICESHIP).expect("the apprenticeship shape parses")
    }

    /// "For magi, there are two more periods to consider: apprenticeship, and life
    /// as a magus after that." (ArMDE:2364.) Apprenticeship is the third
    /// block a ruleset may ship: "The fifteen years of apprenticeship give the
    /// character 240 experience points" (`ArMDE:2435`), with the minimum Abilities the
    /// Order demands (`ArMDE:2437`) and the recommended package priced at 90 experience
    /// points (`ArMDE:2451-2461`).
    ///
    /// Optional, because `ArMDE:2364` calls these "two **more** periods": a non-Hermetic
    /// ruleset ships none, and the block is simply absent.
    #[test]
    fn apprenticeship_rules_carry_the_years_the_xp_and_the_ability_lists() {
        let parsed: LifeStageRules = serde_json::from_str(SHIPPED_WITH_APPRENTICESHIP)
            .expect("the apprenticeship shape parses");
        let apprenticeship = parsed
            .apprenticeship
            .expect("the file declares an apprenticeship block");
        assert_eq!(apprenticeship.years, 15);
        assert_eq!(apprenticeship.xp, 240);
        assert_eq!(apprenticeship.recommended_xp, 90);
        assert_eq!(
            apprenticeship.minimum_abilities,
            vec![
                AbilityRequirement {
                    ability: Id::new("ability.dead_language"),
                    exemplar: None,
                    min_score: 1,
                    parameter: None,
                },
                AbilityRequirement {
                    ability: Id::new("ability.magic_theory"),
                    exemplar: None,
                    min_score: 1,
                    parameter: None,
                },
                AbilityRequirement {
                    ability: Id::new("ability.parma_magica"),
                    exemplar: None,
                    min_score: 1,
                    parameter: None,
                },
            ]
        );
        assert_eq!(
            apprenticeship
                .recommended_abilities
                .iter()
                .map(|r| (r.ability.as_str(), r.min_score))
                .collect::<Vec<_>>(),
            vec![
                ("ability.artes_liberales", 1),
                ("ability.dead_language", 4),
                ("ability.magic_theory", 3),
                ("ability.parma_magica", 1),
            ]
        );

        // A ruleset shipping no apprenticeship parses just as well, and says so.
        assert!(rules().apprenticeship.is_none());
    }

    /// The age a plan naming no Gauntlet age is read at: "These templates are of a
    /// stereotypical member of each House, 25 years old and just out of
    /// apprenticeship" (ArMDE:1601). The
    /// number belongs to the ruleset like every other figure on this block, not to a
    /// constant in the engine.
    ///
    /// Optional, so a ruleset stating no baseline keeps the older reading — a plan
    /// with no Gauntlet age means the magus stands at its Gauntlet — rather than
    /// having one invented for it.
    #[test]
    fn apprenticeship_carries_the_default_gauntlet_age() {
        let parsed: LifeStageRules = serde_json::from_str(SHIPPED_WITH_APPRENTICESHIP)
            .expect("the apprenticeship shape parses");
        let apprenticeship = parsed
            .apprenticeship
            .expect("the file declares an apprenticeship block");
        assert_eq!(apprenticeship.default_gauntlet_age, Some(25));

        // A block stating no baseline parses, and writes no key.
        let silent = ApprenticeshipRules {
            default_gauntlet_age: None,
            ..apprenticeship
        };
        let json = serde_json::to_string(&silent).expect("the block serializes");
        assert!(!json.contains("default_gauntlet_age"), "{json}");
    }

    /// The fourth period: "For every year, the magus gets 30 points"
    /// (ArMDE:2471), less the "10 points from the yearly 30 experience
    /// points" a season of lab work costs, "to a minimum of 0 if three or four
    /// seasons are spent on lab work" (`ArMDE:2482`).
    ///
    /// Additive like every optional field before it: a ruleset shipping no such
    /// block parses, and serializes without the key.
    #[test]
    fn post_apprenticeship_rules_carry_the_yearly_points_and_the_lab_season_cost() {
        let parsed: LifeStageRules = serde_json::from_str(SHIPPED_WITH_APPRENTICESHIP)
            .expect("the post-apprenticeship shape parses");
        let post = parsed
            .post_apprenticeship
            .clone()
            .expect("the file declares a post-apprenticeship block");
        assert_eq!(post.points_per_year, 30);
        assert_eq!(post.lab_season_cost, 10);
        assert_eq!(post.max_charged_lab_seasons_per_year, 3);

        // Serialize → deserialize → equal, so a cached ruleset carries the block.
        let json = serde_json::to_string(&parsed).expect("the block serializes");
        assert_eq!(
            serde_json::from_str::<LifeStageRules>(&json).expect("the block round-trips"),
            parsed
        );

        // A ruleset shipping no post-apprenticeship block parses, and writes no key.
        let without = rules();
        assert!(without.post_apprenticeship.is_none());
        let json = serde_json::to_string(&without).expect("the blockless rules serialize");
        assert!(!json.contains("post_apprenticeship"), "{json}");
    }

    /// A requirement may name one instance of a parameterized Ability, so the field
    /// exists — and is omitted from JSON when unset, which is what keeps the shipped
    /// file free of a `"parameter": null` on every row.
    #[test]
    fn an_ability_requirement_omits_an_unset_parameter() {
        let requirement = AbilityRequirement {
            ability: Id::new("ability.dead_language"),
            exemplar: None,
            min_score: 1,
            parameter: None,
        };
        let json = serde_json::to_string(&requirement).unwrap();
        assert!(!json.contains("parameter"), "{json}");

        let named = AbilityRequirement {
            parameter: Some("Latin".into()),
            ..requirement
        };
        let json = serde_json::to_string(&named).unwrap();
        assert!(json.contains(r#""parameter":"Latin""#), "{json}");
        assert_eq!(
            serde_json::from_str::<AbilityRequirement>(&json).unwrap(),
            named
        );
    }

    /// The exemplar is the same shape as `parameter` — optional, omitted when unset —
    /// but a very different thing: `parameter` NARROWS what satisfies the requirement,
    /// while `exemplar` only labels what the rules named, changing nothing the engine
    /// enforces. Both are on the same struct, so pin the difference.
    #[test]
    fn an_ability_requirement_carries_an_optional_exemplar() {
        let plain = AbilityRequirement {
            ability: Id::new("ability.magic_theory"),
            exemplar: None,
            min_score: 1,
            parameter: None,
        };
        assert!(
            !serde_json::to_string(&plain).unwrap().contains("exemplar"),
            "a requirement without an exemplar writes no key"
        );

        let json = r#"{"ability":"ability.dead_language","exemplar":"latin","min_score":1}"#;
        let parsed: AbilityRequirement = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.exemplar.as_deref(), Some("latin"));
        // Purely a label: what satisfies the requirement is untouched.
        assert_eq!(parsed.parameter, None);
        assert_eq!(
            serde_json::from_str::<AbilityRequirement>(&serde_json::to_string(&parsed).unwrap())
                .unwrap(),
            parsed
        );
    }

    #[test]
    fn life_stage_rules_carry_the_childhood_and_later_life_numbers() {
        let rules = rules();
        assert_eq!(rules.childhood.years, 5);
        assert_eq!(rules.childhood.native_language_xp, 75);
        assert_eq!(rules.childhood.spread_xp, 45);
        assert!(
            rules
                .childhood
                .spread_abilities
                .contains(&Id::new("ability.swim"))
        );
        assert_eq!(rules.later_life.xp_per_year, 15);
    }

    /// Later life starts where childhood ends, so the years that earn the yearly
    /// experience are `age - childhood.years` (ArMDE:2378, :2392).
    #[test]
    fn later_life_years_start_after_childhood() {
        let rules = rules();
        assert_eq!(rules.later_life_years(25, 0), 20);
        // A five-year-old has finished childhood and no more.
        assert_eq!(rules.later_life_years(5, 0), 0);
    }

    /// An age below the childhood span is not a shorter childhood — childhood is a
    /// fixed five-year block — so it earns no later-life years at all rather than
    /// underflowing into a huge count.
    #[test]
    fn an_age_inside_childhood_earns_no_later_life_years() {
        assert_eq!(rules().later_life_years(3, 0), 0);
    }

    /// A ruleset carrying the two rate-bearing items, so the per-year rate can be
    /// read off a character's selections rather than hardcoded.
    fn rate_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "virtue.wealthy", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "later_life_xp_rate", "amount": 20 }] },
          { "id": "flaw.poor", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "later_life_xp_rate", "amount": 10 }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"], "creation_phases": [] },
          { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"],
            "hermetically_trained": true, "order_member": true, "creation_phases": [] }
        ]"#;
        Ruleset::from_json("test", "1", items, types).unwrap()
    }

    /// A companion **funded from its life stages** — the mode every budget assertion
    /// in this module needs. Since schema 16 the mode is stored
    /// ([`AbilityFunding`]) rather than inferred from the plan's presence, so setting
    /// it here once is what keeps a fixture's plan live; a fixture that only assigned
    /// `life_stages` would build a pool-funded character whose plan earns nothing.
    /// The tests that pin the pool side set `ability_funding` themselves.
    fn companion(selections: Vec<&str>) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.ability_funding = AbilityFunding::LifeStages;
        entity.selections = selections
            .into_iter()
            .map(|r| Selection::new(Id::new(r)))
            .collect();
        entity
    }

    /// A magus of `age`, built through its life stages.
    fn planned_magus(age: u32) -> Entity {
        let mut entity = companion(vec![]);
        entity.type_id = Id::new("magus");
        entity.age = Some(age);
        entity.life_stages = Some(LifeStagePlan::default());
        entity
    }

    /// Wealthy and Poor replace the yearly rate rather than adding to it: "get 20
    /// experience points per year" / "get 10 experience points per year".
    /// Source: ArMDE:2394.
    #[test]
    fn wealthy_and_poor_replace_the_yearly_rate() {
        let rs = rate_ruleset();
        let rules = rules();
        assert_eq!(rules.later_life_rate(&companion(vec![]), &rs), 15);
        assert_eq!(
            rules.later_life_rate(&companion(vec!["virtue.wealthy"]), &rs),
            20
        );
        assert_eq!(
            rules.later_life_rate(&companion(vec!["flaw.poor"]), &rs),
            10
        );
    }

    /// The two are mutually exclusive in the shipped data, but nothing in the rules
    /// text makes one override the other, so the engine takes the lowest rate it is
    /// told about rather than picking by declaration order — the conservative
    /// reading, and a deterministic one.
    #[test]
    fn conflicting_rates_resolve_to_the_lowest() {
        let rs = rate_ruleset();
        assert_eq!(
            rules().later_life_rate(&companion(vec!["virtue.wealthy", "flaw.poor"]), &rs),
            10
        );
    }

    /// A magus's later life ends where its apprenticeship begins: "**Later Life.** 15
    /// experience points per year (until apprenticeship for magi)"
    /// (ArMDE:2214). Apprenticeship is fifteen years (`ArMDE:2435`) and the stop
    /// age is the **Gauntlet** age, so a magus gauntleted at 25 was taken as an
    /// apprentice at 10 and lived five later-life years however old it is now —
    /// exactly the arithmetic of the Darius
    /// example, whose master "picks 10 as a nice, round number" and who then "has 75
    /// experience points to spend from those five years" (`ArMDE:2402`).
    #[test]
    fn a_magus_later_life_stops_at_the_gauntlet() {
        let rules = rules();
        // Fifteen years of apprenticeship take their span out of later life.
        assert_eq!(rules.later_life_years(25, 15), 5);
        // A grog or companion has no apprenticeship, so nothing changes for it.
        assert_eq!(rules.later_life_years(25, 0), 20);
        // An age inside childhood-plus-apprenticeship earns no later-life year
        // rather than underflowing into a huge count.
        assert_eq!(rules.later_life_years(19, 15), 0);

        // Through the budget: the five years are worth 75 experience points.
        let rs = rate_ruleset();
        let rules = rules_with_apprenticeship();
        let budget = rules
            .budget(&planned_magus(25), &rs)
            .expect("a magus with a plan");
        assert_eq!(budget.later_life_years, 5);
        assert_eq!(budget.later_life_xp, 75);

        // The same age as a companion keeps every year it always had.
        let mut entity = companion(vec![]);
        entity.age = Some(25);
        entity.life_stages = Some(LifeStagePlan::default());
        let budget = rules.budget(&entity, &rs).expect("a companion with a plan");
        assert_eq!(budget.later_life_years, 20);
        assert_eq!(budget.later_life_xp, 300);
    }

    /// A character with a life-stage plan earns the two childhood blocks plus one
    /// per-year block, so a 25-year-old companion has 75 + 45 + 20×15 = 420 points
    /// across three differently-restricted pools.
    #[test]
    fn the_budget_is_the_childhood_blocks_plus_the_yearly_ones() {
        let rs = rate_ruleset();
        let mut entity = companion(vec![]);
        entity.age = Some(25);
        entity.life_stages = Some(LifeStagePlan {
            native_language: Some("German".into()),
            ..LifeStagePlan::default()
        });

        let budget = rules()
            .budget(&entity, &rs)
            .expect("a character with a plan and an age has a budget");
        assert_eq!(budget.childhood_native_xp, 75);
        assert_eq!(budget.childhood_spread_xp, 45);
        assert_eq!(budget.later_life_years, 20);
        assert_eq!(budget.later_life_rate, 15);
        assert_eq!(budget.later_life_xp, 300);
        assert_eq!(budget.total(), 420);
    }

    /// A magus earns a fourth block on top of the three: "The fifteen years of
    /// apprenticeship give the character 240 experience points"
    /// (ArMDE:2435). So a magus of 25 has 75 + 45 + 75 + 240 = 435 points,
    /// where a companion of the same age has 420.
    #[test]
    fn the_budget_of_a_guided_magus_adds_its_apprenticeship() {
        let rs = rate_ruleset();
        let rules = rules_with_apprenticeship();

        let budget = rules
            .budget(&planned_magus(25), &rs)
            .expect("a magus with a plan");
        assert_eq!(budget.apprenticeship_years, 15);
        assert_eq!(budget.apprenticeship_xp, 240);
        assert_eq!(budget.later_life_xp, 75);
        assert_eq!(budget.total(), 435);

        // A companion serves no apprenticeship, so its budget is untouched.
        let mut entity = companion(vec![]);
        entity.age = Some(25);
        entity.life_stages = Some(LifeStagePlan::default());
        let budget = rules.budget(&entity, &rs).expect("a companion with a plan");
        assert_eq!(budget.apprenticeship_years, 0);
        assert_eq!(budget.apprenticeship_xp, 0);
        assert_eq!(budget.total(), 420);
    }

    /// Apprenticeship is a fixed block of fifteen years (`ArMDE:2435`), exactly as
    /// childhood is a fixed five, so it does not wait for an age either: only later
    /// life is counted in years up to one. A guided magus whose age is not yet typed
    /// therefore holds its 240 points and lives no later-life year — the missing age
    /// is reported on its own (`life_stage_age_unset`).
    #[test]
    fn an_unset_age_still_earns_the_apprenticeship_block() {
        let rs = rate_ruleset();
        let mut magus = planned_magus(25);
        magus.age = None;

        let budget = rules_with_apprenticeship()
            .budget(&magus, &rs)
            .expect("apprenticeship does not depend on an age");
        assert_eq!(budget.apprenticeship_years, 15);
        assert_eq!(budget.apprenticeship_xp, 240);
        assert_eq!(budget.later_life_years, 0);
        assert_eq!(budget.later_life_xp, 0);
        assert_eq!(budget.total(), 360);
    }

    // --- D40/D2: the apprenticeship-shaped replacement (Redcap, Lone Redcap) --

    /// A ruleset carrying one companion-shaped Virtue with a
    /// `replaces_life_stage_xp` effect naming `apprenticeship` — the shape
    /// `virtue.redcap`/`virtue.lone_redcap` will ship, with `years`/`amount`
    /// left as constructor arguments so the crafted-maximum-values red can
    /// reuse the same fixture.
    fn apprenticeship_replacement_ruleset(years: u32, amount: u32) -> Ruleset {
        let items = format!(
            r#"[
              {{ "id": "virtue.test_redcap", "kind": "virtue", "classification": "creation_effect",
                 "magnitude": "major", "categories": ["social_status"], "entity_kinds": ["character"],
                 "effects": [{{ "type": "replaces_life_stage_xp", "stage": "apprenticeship",
                                "amount": {amount}, "years": {years},
                                "categories": ["academic", "arcane", "general", "martial", "supernatural"] }}] }},
              {{ "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
                 "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }}
            ]"#
        );
        let types = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general", "academic", "arcane", "martial", "supernatural"],
            "creation_phases": [] }
        ]"#;
        Ruleset::from_json("test", "1", &items, types).unwrap()
    }

    /// D40/D2's F2 fix: an apprenticeship-shaped replacement carves its own
    /// `years` out of later life exactly as a real magus's apprenticeship
    /// does, while `LifeStageBudget.apprenticeship_years` — documented "15
    /// for a magus, 0 for anyone else" — stays 0 for this NON-hermetically-
    /// trained-by-profile carrier: the widened local feeds `later_life_years`
    /// alone, never that struct field. A 25-year-old Redcap-shaped companion
    /// therefore has 5 years of later life left (25 − 5 childhood − 15
    /// carved), at 15/year = 75.
    #[test]
    fn an_apprenticeship_shaped_replacement_carves_later_life_but_not_the_struct_field() {
        let rs = apprenticeship_replacement_ruleset(15, 300);
        let mut entity = companion(vec!["virtue.test_redcap"]);
        entity.age = Some(25);
        entity.life_stages = Some(LifeStagePlan::default());

        let budget = rules()
            .budget(&entity, &rs)
            .expect("a companion with a plan");
        assert_eq!(
            budget.later_life_years, 5,
            "25 - 5 (childhood) - 15 (carved) = 5"
        );
        assert_eq!(budget.later_life_xp, 75);
        assert_eq!(
            budget.apprenticeship_years, 0,
            "NOT a real magus's own apprenticeship — the struct field keeps its \
             documented meaning"
        );
        assert_eq!(budget.apprenticeship_xp, 0);
    }

    /// F3-style robustness (the design note's "crafted-maximum-values" red):
    /// a ruleset-authored `years`/`amount` at `u32::MAX` — the parameter has
    /// no runtime bound of its own the way D3's `years_completed` does, since
    /// `years`/`amount` here are fixed, ruleset-authored numbers, not a
    /// player-facing parameter — must saturate rather than panic under
    /// `overflow-checks = true` (the dev/test profile's default, `Cargo.
    /// toml:17-26`'s own reasoning for why release enables it too).
    #[test]
    fn an_apprenticeship_shaped_replacement_at_u32_max_saturates_without_panicking() {
        let rs = apprenticeship_replacement_ruleset(u32::MAX, u32::MAX);
        let mut entity = companion(vec!["virtue.test_redcap"]);
        entity.age = Some(25);
        entity.life_stages = Some(LifeStagePlan::default());

        let budget = rules()
            .budget(&entity, &rs)
            .expect("a companion with a plan");
        assert_eq!(
            budget.later_life_years, 0,
            "later life saturates to 0 rather than underflowing"
        );
        assert_eq!(budget.later_life_xp, 0);
        assert_eq!(budget.apprenticeship_years, 0);
    }

    // --- life as a magus after the Gauntlet (M6/6b5) -------------------------

    /// A magus of `age` gauntleted at `gauntlet_age`, having charged `lab_seasons`
    /// seasons of lab work against its yearly points and taken `spell_levels` of
    /// them as levels of spells.
    fn magus_out_of_apprenticeship(
        age: u32,
        gauntlet_age: u32,
        lab_seasons: u32,
        spell_levels: u32,
    ) -> Entity {
        let mut entity = planned_magus(age);
        entity.life_stages = Some(LifeStagePlan {
            gauntlet_age: Some(gauntlet_age),
            post_gauntlet_lab_seasons: lab_seasons,
            post_gauntlet_spell_levels: spell_levels,
            ..LifeStagePlan::default()
        });
        entity
    }

    /// At the ruleset's own baseline age the stored value and the default coincide:
    /// a magus of 25 with no Gauntlet age on file stands at its Gauntlet either way,
    /// so every save written before the field existed keeps these numbers.
    #[test]
    fn a_magus_at_the_default_gauntlet_age_stands_at_its_gauntlet() {
        let budget = rules_with_apprenticeship()
            .budget(&planned_magus(25), &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.gauntlet_age, 25);
        assert_eq!(budget.later_life_years, 5);
        assert_eq!(budget.apprenticeship_xp, 240);
        assert_eq!(budget.post_gauntlet_years, 0);
        assert_eq!(budget.post_gauntlet_points, 0);
        assert_eq!(budget.post_gauntlet_xp, 0);
        assert_eq!(budget.total(), 435);
    }

    /// A plan naming no Gauntlet age reads the ruleset's baseline, not the
    /// character's own age: "These templates are of a stereotypical member of each
    /// House, 25 years old and just out of apprenticeship"
    /// (ArMDE:1601). So a magus of 60 that
    /// never filled the field in is thirty-five years out of apprenticeship, which is
    /// what the rules' own worked example does — Darius counts his years as a magus
    /// "from 26 to 33" (`ArMDE:2486`), not from the age he happens to be.
    ///
    /// Reading the age itself, as this did before, made the blank field mean "zero
    /// years as a magus" — the least likely magus there is, and the one that made an
    /// otherwise fine lab-season entry look impossible.
    #[test]
    fn a_magus_with_no_stored_gauntlet_age_takes_the_rulesets_default() {
        let budget = rules_with_apprenticeship()
            .budget(&planned_magus(60), &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.gauntlet_age, 25);
        assert_eq!(budget.later_life_years, 5);
        assert_eq!(budget.post_gauntlet_years, 35);
        assert_eq!(budget.post_gauntlet_points, 1050);
    }

    /// The default is clamped to the age like a stored value is, so a magus younger
    /// than the baseline still stands at its Gauntlet and no new finding becomes
    /// reachable: `life_stage_gauntlet_age_after_age` reads the *stored* number,
    /// which is still absent here, and `life_stage_age_before_gauntlet` sees the same
    /// age it saw before.
    #[test]
    fn a_magus_younger_than_the_default_gauntlet_age_stands_at_its_gauntlet() {
        let budget = rules_with_apprenticeship()
            .budget(&planned_magus(22), &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.gauntlet_age, 22);
        assert_eq!(budget.later_life_years, 2);
        assert_eq!(budget.post_gauntlet_years, 0);
        assert_eq!(budget.post_gauntlet_points, 0);
    }

    /// A ruleset stating no baseline keeps the reading that predates the field: the
    /// magus stands at its Gauntlet, whatever its age.
    #[test]
    fn a_ruleset_with_no_default_gauntlet_age_leaves_the_magus_at_its_gauntlet() {
        let mut rules = rules_with_apprenticeship();
        rules
            .apprenticeship
            .as_mut()
            .expect("the fixture declares an apprenticeship")
            .default_gauntlet_age = None;
        let budget = rules
            .budget(&planned_magus(60), &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.gauntlet_age, 60);
        assert_eq!(budget.post_gauntlet_years, 0);
    }

    /// "For every year, the magus gets 30 points."
    /// (ArMDE:2471.) The years counted are
    /// the ones after the Gauntlet — and the years *before* it are untouched by the
    /// character's age, which is the whole reason the Gauntlet age is the stored
    /// number: a magus of 60 gauntleted at 25 still lived the same five later-life
    /// years as a magus of 25.
    #[test]
    fn the_years_after_the_gauntlet_earn_thirty_points_each() {
        let budget = rules_with_apprenticeship()
            .budget(&magus_out_of_apprenticeship(60, 25, 0, 0), &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.gauntlet_age, 25);
        assert_eq!(budget.post_gauntlet_years, 35);
        assert_eq!(budget.post_gauntlet_points, 1050);
        assert_eq!(budget.post_gauntlet_xp, 1050);
        // The span before the Gauntlet is exactly the one a magus of 25 lives.
        assert_eq!(budget.later_life_years, 5);
        assert_eq!(budget.later_life_xp, 75);
        assert_eq!(budget.apprenticeship_years, 15);
        assert_eq!(budget.total(), 435 + 1050);
    }

    /// "For each season that your magus spends working on a lab project, the
    /// character loses 10 points from the yearly 30 experience points" (`ArMDE:2482`), so
    /// ten charged seasons cost 100 of the 1050. Thirteen stored seasons pack as three
    /// full lab years (three charged each) plus one, so ten are charged (F1).
    #[test]
    fn each_charged_lab_season_costs_ten_points() {
        let budget = rules_with_apprenticeship()
            .budget(&magus_out_of_apprenticeship(60, 25, 13, 0), &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.post_gauntlet_points, 950);
        assert_eq!(budget.post_gauntlet_xp, 950);
    }

    /// Norbert 2026-10-03 (F1): the stored total is read as packed into full lab
    /// years, as `ArMDE:2482` recommends, so a fourth season fills a year rather than
    /// costing the next one. Darius's nine years with one full lab year are worth
    /// the book's 240 (`ArMDE:2486`, :2488), not 230.
    #[test]
    fn darius_nine_years_with_one_full_lab_year_are_worth_240() {
        let budget = rules_with_apprenticeship()
            .budget(&magus_out_of_apprenticeship(34, 25, 4, 0), &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.post_gauntlet_years, 9);
        assert_eq!(budget.post_gauntlet_points, 240);
    }

    /// The deduction runs "to a minimum of 0 if three or four seasons are spent on
    /// lab work" (`ArMDE:2482`), so the fourth season of a year is free — the year has
    /// nothing left to lose. A magus one year out of apprenticeship that spent every
    /// season in the lab is at 0 points, never below.
    #[test]
    fn a_fourth_lab_season_in_a_year_costs_nothing() {
        let rules = rules_with_apprenticeship();
        let rs = rate_ruleset();
        let three = rules
            .budget(&magus_out_of_apprenticeship(26, 25, 3, 0), &rs)
            .expect("a magus with a plan");
        assert_eq!(three.post_gauntlet_years, 1);
        assert_eq!(three.post_gauntlet_points, 0);

        let four = rules
            .budget(&magus_out_of_apprenticeship(26, 25, 4, 0), &rs)
            .expect("a magus with a plan");
        assert_eq!(four.post_gauntlet_points, 0);

        // Over 35 years the same cap holds: 105 charged seasons exhaust the 1050,
        // and a stored total beyond that neither costs more nor underflows.
        let all_lab = rules
            .budget(&magus_out_of_apprenticeship(60, 25, 200, 0), &rs)
            .expect("a magus with a plan");
        assert_eq!(all_lab.post_gauntlet_points, 0);
        assert_eq!(all_lab.post_gauntlet_xp, 0);
    }

    /// "Each point can be an experience point in an Art or Ability or one level of
    /// spell" (`ArMDE:2471`), so the split is the player's: 300 of the 950 taken as spell
    /// levels leave 650 experience points. `total()` counts the 650 alone — a level
    /// of spell is not experience, and folding it in would spend it twice.
    #[test]
    fn points_taken_as_spell_levels_are_not_experience() {
        let budget = rules_with_apprenticeship()
            .budget(
                &magus_out_of_apprenticeship(60, 25, 13, 300),
                &rate_ruleset(),
            )
            .expect("a magus with a plan");
        assert_eq!(budget.post_gauntlet_points, 950);
        assert_eq!(budget.post_gauntlet_spell_levels, 300);
        assert_eq!(budget.post_gauntlet_xp, 650);
        assert_eq!(budget.total(), 435 + 650);

        // More levels than there are points buys only the points that exist.
        let greedy = rules_with_apprenticeship()
            .budget(
                &magus_out_of_apprenticeship(60, 25, 13, 5_000),
                &rate_ruleset(),
            )
            .expect("a magus with a plan");
        assert_eq!(greedy.post_gauntlet_spell_levels, 950);
        assert_eq!(greedy.post_gauntlet_xp, 0);
    }

    /// A Gauntlet age past the character's age is clamped to it rather than
    /// underflowing. The clamp is load-bearing: `ValidationMode::Advisory` and
    /// `Silent` do not block the error such a plan raises, so the budget has to stay
    /// arithmetically sane on its own — an unclamped value would hand the magus
    /// later-life years it never lived.
    #[test]
    fn a_gauntlet_age_above_the_characters_age_is_clamped_to_it() {
        let budget = rules_with_apprenticeship()
            .budget(&magus_out_of_apprenticeship(25, 40, 0, 0), &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.gauntlet_age, 25);
        assert_eq!(budget.later_life_years, 5);
        assert_eq!(budget.post_gauntlet_years, 0);
        assert_eq!(budget.post_gauntlet_points, 0);
    }

    /// The Gauntlet age is read for a character that serves an apprenticeship and
    /// nobody else — `ArMDE:2216` is "**Hermetic Magi Only (Optional):** Years after
    /// apprenticeship". Gating on the field instead would let a hand-edited companion
    /// plan carrying one silently lose every later-life year between the two ages: 35
    /// years, 525 experience points.
    #[test]
    fn a_companion_ignores_a_stored_gauntlet_age() {
        let rules = rules_with_apprenticeship();
        let rs = rate_ruleset();
        let mut plain = companion(vec![]);
        plain.age = Some(60);
        plain.life_stages = Some(LifeStagePlan::default());

        let mut edited = plain.clone();
        edited.life_stages = Some(LifeStagePlan {
            gauntlet_age: Some(25),
            post_gauntlet_lab_seasons: 4,
            post_gauntlet_spell_levels: 300,
            ..LifeStagePlan::default()
        });

        assert_eq!(
            rules.budget(&edited, &rs).expect("a companion with a plan"),
            rules.budget(&plain, &rs).expect("a companion with a plan")
        );
        let budget = rules.budget(&edited, &rs).expect("a companion with a plan");
        assert_eq!(budget.later_life_years, 55);
        assert_eq!(budget.post_gauntlet_years, 0);
    }

    /// Age-dependent figures wait for an age, exactly as later life does: a magus
    /// whose age is not yet typed lives no year after its Gauntlet either. The
    /// missing age is reported on its own (`life_stage_age_unset`).
    #[test]
    fn an_unset_age_earns_no_post_gauntlet_year() {
        let mut magus = magus_out_of_apprenticeship(60, 25, 4, 100);
        magus.age = None;

        let budget = rules_with_apprenticeship()
            .budget(&magus, &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.post_gauntlet_years, 0);
        assert_eq!(budget.post_gauntlet_points, 0);
        assert_eq!(budget.post_gauntlet_spell_levels, 0);
        assert_eq!(budget.post_gauntlet_xp, 0);
        assert_eq!(budget.later_life_years, 0);
        assert_eq!(budget.total(), 360);
    }

    /// A ruleset shipping no post-apprenticeship block grants nothing for the years
    /// after the Gauntlet — the rate lives in the rules data, so there is no fallback
    /// number to invent. (The shipped combination is refused at load; this is the
    /// arithmetic behind that refusal.)
    #[test]
    fn without_a_post_apprenticeship_block_the_years_after_the_gauntlet_grant_nothing() {
        let mut rules = rules_with_apprenticeship();
        rules.post_apprenticeship = None;

        let budget = rules
            .budget(&magus_out_of_apprenticeship(60, 25, 0, 0), &rate_ruleset())
            .expect("a magus with a plan");
        assert_eq!(budget.post_gauntlet_years, 35);
        assert_eq!(budget.post_gauntlet_points, 0);
        assert_eq!(budget.post_gauntlet_spell_levels, 0);
        assert_eq!(budget.post_gauntlet_xp, 0);
    }

    // --- the Hermetic minimum Abilities (M6/6b4) -----------------------------

    /// A ruleset whose life stages carry the shipped apprenticeship requirements, so
    /// the checklist can be read off a real ruleset rather than a bare
    /// [`LifeStageRules`].
    fn minimum_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "virtue.puissant_ability", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
            "effects": [{ "type": "ability_bonus", "param": "ability", "amount": 2 }] }
        ]"#;
        let types = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"], "creation_phases": [] },
          { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"],
            "hermetically_trained": true, "order_member": true, "creation_phases": [] }
        ]"#;
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 3, "total_xp": 30 },
            { "score": 4, "total_xp": 50 }
          ],
          "abilities": [
            { "id": "ability.artes_liberales", "category": "academic" },
            { "id": "ability.athletics", "category": "general" },
            { "id": "ability.dead_language", "category": "academic", "parameter": "language" },
            { "id": "ability.living_language", "category": "general", "parameter": "language" },
            { "id": "ability.magic_theory", "category": "arcane" },
            { "id": "ability.parma_magica", "category": "arcane" },
            { "id": "ability.penetration", "category": "arcane" },
            { "id": "ability.philosophiae", "category": "academic" },
            { "id": "ability.swim", "category": "general" }
          ]
        }"#;
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            life_stages: Some(SHIPPED_WITH_APPRENTICESHIP),
            ..crate::ruleset::RulesetSources::default()
        })
        .unwrap()
    }

    fn magus(scores: Vec<(&str, Option<&str>, u8)>) -> Entity {
        let mut entity = companion(vec![]);
        entity.type_id = Id::new("magus");
        entity.ability_scores = scores
            .into_iter()
            .map(|(ability, parameter, score)| {
                let mut a = crate::types::AbilityScore::new(Id::new(ability), score);
                a.parameter = parameter.map(crate::types::AbilityParameterValue::text);
                a
            })
            .collect();
        entity
    }

    /// The one reading of `ArMDE:2437` and `ArMDE:2451-2461`, as a checklist: every requirement
    /// in order, the score the character actually bought, and whether it is met.
    ///
    /// Both the validator and the effective-scores payload consume this, so they
    /// cannot disagree about what the Order demands.
    #[test]
    fn the_magus_minimum_checklist_reports_met_and_unmet() {
        let rs = minimum_ruleset();

        // A magus with nothing bought: seven rows, all unmet, minimums first.
        let checklist = magus_minimum_abilities(&magus(vec![]), &rs);
        let rows: Vec<(&str, u8, u8, bool, AbilityRequirementKind)> = checklist
            .iter()
            .map(|row| {
                (
                    row.ability.as_str(),
                    row.min_score,
                    row.score,
                    row.met,
                    row.requirement,
                )
            })
            .collect();
        use AbilityRequirementKind::{Recommended, Required};
        assert_eq!(
            rows,
            vec![
                ("ability.dead_language", 1, 0, false, Required),
                ("ability.magic_theory", 1, 0, false, Required),
                ("ability.parma_magica", 1, 0, false, Required),
                ("ability.artes_liberales", 1, 0, false, Recommended),
                ("ability.dead_language", 4, 0, false, Recommended),
                ("ability.magic_theory", 3, 0, false, Recommended),
                ("ability.parma_magica", 1, 0, false, Recommended),
            ]
        );

        // The Darius package: Latin 4, Magic Theory 4, Artes Liberales 3, Parma 1
        // (`ArMDE:2441`, `ArMDE:2449`) meets every row.
        let darius = magus(vec![
            ("ability.dead_language", Some("Latin"), 4),
            ("ability.magic_theory", None, 4),
            ("ability.artes_liberales", None, 3),
            ("ability.parma_magica", None, 1),
        ]);
        assert!(
            magus_minimum_abilities(&darius, &rs)
                .iter()
                .all(|row| row.met),
            "{:?}",
            magus_minimum_abilities(&darius, &rs)
        );

        // The score reported is the highest bought instance of the id, so a magus
        // with Greek 1 and Latin 4 is credited with 4.
        let polyglot = magus(vec![
            ("ability.dead_language", Some("Greek"), 1),
            ("ability.dead_language", Some("Latin"), 4),
        ]);
        let rows = magus_minimum_abilities(&polyglot, &rs);
        assert_eq!(
            rows.iter()
                .filter(|row| row.ability == Id::new("ability.dead_language"))
                .map(|row| (row.min_score, row.score, row.met))
                .collect::<Vec<_>>(),
            vec![(1, 4, true), (4, 4, true)]
        );

        // Nothing is demanded of a companion — `ArMDE:2437` is about admission to the
        // Order — and nothing at all when the ruleset ships no apprenticeship block.
        assert!(magus_minimum_abilities(&companion(vec![]), &rs).is_empty());
        assert!(magus_minimum_abilities(&magus(vec![]), &rate_ruleset()).is_empty());
    }

    /// **Documented approximation.** "Latin 1" is matched by Ability id alone, so a
    /// magus whose only dead language is Greek satisfies it. The rules model Latin as
    /// one *value* of the parameterized dead-language Ability, and an instance value
    /// is free-text player input with no localization path — a German player types
    /// "Latein" — so an instance match would fail for every non-English user, which is
    /// worse than under-enforcing. `AbilityRequirement::parameter` is the tightening a
    /// future language registry would fill in.
    ///
    /// Asserted so this can never become accidental. See `RULES.md`.
    #[test]
    fn latin_is_matched_by_ability_id_not_by_instance() {
        let rs = minimum_ruleset();
        let greek = magus(vec![("ability.dead_language", Some("Greek"), 1)]);
        let row = magus_minimum_abilities(&greek, &rs)
            .into_iter()
            .find(|row| {
                row.ability == Id::new("ability.dead_language")
                    && row.requirement == AbilityRequirementKind::Required
            })
            .expect("the Latin minimum is on the checklist");
        assert!(row.met, "any dead language satisfies it: {row:?}");
        assert!(
            row.parameter.is_none(),
            "the shipped requirement names no instance: {row:?}"
        );
    }

    /// The score tested is the **bought** one. "Magi must have the following minimum
    /// Abilities … Characters with lower scores would not be admitted to the Order"
    /// (`ArMDE:2437`) cannot mean the effective score: `effective_ability_score` returns 2
    /// for a magus with a Puissant Parma Magica and no Parma row at all, and a
    /// Virtue's +2 to *use* is not training the Order can examine.
    #[test]
    fn puissant_parma_magica_does_not_admit_a_magus_to_the_order() {
        let rs = minimum_ruleset();
        let mut entity = magus(vec![]);
        entity.selections = vec![Selection::with_params(
            Id::new("virtue.puissant_ability"),
            std::collections::BTreeMap::from([(
                "ability".to_string(),
                Id::new("ability.parma_magica"),
            )]),
        )];

        // The effective score is 2 …
        assert_eq!(
            crate::effective::effective_ability_score(
                &entity,
                &rs,
                &Id::new("ability.parma_magica"),
                None
            ),
            2
        );
        // … and the Order is unimpressed.
        let row = magus_minimum_abilities(&entity, &rs)
            .into_iter()
            .find(|row| {
                row.ability == Id::new("ability.parma_magica")
                    && row.requirement == AbilityRequirementKind::Required
            })
            .expect("the Parma minimum is on the checklist");
        assert_eq!(row.score, 0);
        assert!(!row.met, "{row:?}");
    }

    /// The chosen Sample Childhood package is recorded on the character, so it
    /// survives a save/load round-trip; it is omitted from JSON when unset, so a
    /// save written before the packages existed stays byte-compatible; and being
    /// additive it bumps no schema version of its own — exactly the
    /// `warping_choices` precedent.
    #[test]
    fn the_chosen_childhood_package_roundtrips_and_is_schema_stable() {
        let mut entity = companion(vec![]);
        // A fresh entity has no plan at all, so certainly no package.
        let json = serde_json::to_string(&entity).unwrap();
        assert!(!json.contains("childhood_package"), "{json}");

        // A plan whose package is unset omits the key rather than writing null.
        entity.life_stages = Some(LifeStagePlan::default());
        entity.normalize();
        let json = serde_json::to_string(&entity).unwrap();
        assert!(!json.contains("childhood_package"), "{json}");

        entity.life_stages = Some(LifeStagePlan {
            native_language: Some("German".into()),
            childhood_package: Some(Id::new("childhood.athletic")),
            ..LifeStagePlan::default()
        });
        entity.normalize();
        let json = serde_json::to_string_pretty(&entity).unwrap();
        assert!(
            json.contains(r#""childhood_package": "childhood.athletic""#),
            "{json}"
        );
        assert!(json.contains(r#""schema_version": 21"#), "{json}");

        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, back);
        assert_eq!(back.schema_version, SCHEMA_VERSION);
        assert_eq!(
            back.life_stages
                .as_ref()
                .and_then(|plan| plan.childhood_package.as_ref()),
            Some(&Id::new("childhood.athletic"))
        );
    }

    /// The three choices life after the Gauntlet needs are stored on the plan, and
    /// nothing else is: `age = childhood + later life + apprenticeship +
    /// post-Gauntlet` is one equation in two unknowns, so exactly one of them —
    /// the Gauntlet age — is recorded, and the years as a magus follow from it.
    ///
    /// All three are additive. Absent, they add no key at all, so a save written
    /// before they existed is byte-identical and needs no schema bump. This is also
    /// what keeps the **default** Gauntlet age out of the save: it is resolved when
    /// the budget is computed and never written back, so a plan the player left blank
    /// serializes to no `gauntlet_age` key — which is why nothing marks the entity
    /// dirty and no round trip can silently acquire a 25.
    #[test]
    fn the_post_gauntlet_choices_roundtrip_and_are_schema_stable() {
        let nothing_chosen = LifeStagePlan {
            native_language: Some("German".into()),
            childhood_package: Some(Id::new("childhood.athletic")),
            ..LifeStagePlan::default()
        };
        assert_eq!(
            serde_json::to_string(&nothing_chosen).unwrap(),
            r#"{"native_language":"German","childhood_package":"childhood.athletic"}"#
        );

        let out_of_apprenticeship = LifeStagePlan {
            gauntlet_age: Some(25),
            post_gauntlet_lab_seasons: 12,
            post_gauntlet_spell_levels: 300,
            ..nothing_chosen
        };
        let mut entity = companion(vec![]);
        entity.age = Some(60);
        entity.life_stages = Some(out_of_apprenticeship.clone());
        entity.normalize();
        let json = serde_json::to_string_pretty(&entity).unwrap();
        assert!(json.contains(r#""gauntlet_age": 25"#), "{json}");
        assert!(
            json.contains(r#""post_gauntlet_lab_seasons": 12"#),
            "{json}"
        );
        assert!(
            json.contains(r#""post_gauntlet_spell_levels": 300"#),
            "{json}"
        );
        // The post-Gauntlet fields are additive and bumped nothing of their own;
        // the literal is here so a bump has to be a conscious edit (15 came from
        // the widened aging log, 16 from the funding discriminator, 17 from the
        // per-document saga year, 18 from CV4's ability-parameter widening, 19
        // from C5a's multi-valued parameter type, 20 from F1's
        // `EquipmentSlot::loadout` move, and 21 from X9b's `virtue.rard` ->
        // `virtue.bard` id rename, not from this plan).
        assert_eq!(SCHEMA_VERSION, 21);
        assert!(json.contains(r#""schema_version": 21"#), "{json}");

        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(back.life_stages, Some(out_of_apprenticeship));
    }

    /// Wealthy multiplies out across every year of later life, not once.
    #[test]
    fn a_raised_rate_applies_to_every_later_life_year() {
        let rs = rate_ruleset();
        let mut entity = companion(vec!["virtue.wealthy"]);
        entity.age = Some(25);
        entity.life_stages = Some(LifeStagePlan::default());

        let budget = rules().budget(&entity, &rs).expect("budget");
        assert_eq!(budget.later_life_xp, 400);
        assert_eq!(budget.total(), 520);
    }

    /// No plan means no derived budget: the character is in direct entry, where
    /// `Entity::xp_pool` is the authority.
    #[test]
    fn there_is_no_budget_without_a_plan() {
        let rs = rate_ruleset();
        let mut entity = companion(vec![]);
        entity.age = Some(25);
        assert!(rules().budget(&entity, &rs).is_none(), "no plan");
    }

    /// **The shape schema 16 exists to allow.** Before the funding mode was stored,
    /// a plan's presence *was* the mode, so this case could not be constructed: a
    /// character funded from its typed [`Entity::xp_pool`] that nonetheless keeps a
    /// life-stage plan on file. The plan is inert — it earns nothing — and the pool
    /// is the authority, so no budget is derived from it.
    ///
    /// Reading the plan here instead would double-fund the character the moment the
    /// player switched to pool funding without discarding the work already typed
    /// into the stages, which is precisely what #29's fix stopped destroying.
    #[test]
    fn budget_is_none_for_pool_funding_even_with_a_stored_plan() {
        let rs = rate_ruleset();
        let mut entity = companion(vec![]);
        entity.age = Some(25);
        entity.life_stages = Some(LifeStagePlan {
            native_language: Some("German".into()),
            ..LifeStagePlan::default()
        });
        entity.ability_funding = AbilityFunding::Pool;
        assert!(
            rules().budget(&entity, &rs).is_none(),
            "a pool-funded character earns nothing from a plan it is not funded by"
        );

        // The very same plan, funded from the stages, does earn its budget.
        entity.ability_funding = AbilityFunding::LifeStages;
        assert!(rules().budget(&entity, &rs).is_some());
    }

    /// Childhood is granted "in the first five years of life" unconditionally
    /// (ArMDE:2378) — only later life counts years up to an age. So a plan
    /// whose age is not yet typed still earns both childhood blocks, and merely
    /// lives no later-life year. Returning nothing instead would leave the
    /// childhood pools at 0 and report every childhood row as unfunded, blaming
    /// the player for rows the app itself wrote; the missing age is reported on its
    /// own (`life_stage_age_unset`).
    #[test]
    fn an_unset_age_still_earns_the_childhood_blocks() {
        let rs = rate_ruleset();
        let mut entity = companion(vec![]);
        entity.life_stages = Some(LifeStagePlan::default());
        entity.age = None;

        let budget = rules()
            .budget(&entity, &rs)
            .expect("childhood does not depend on an age");
        assert_eq!(budget.childhood_native_xp, 75);
        assert_eq!(budget.childhood_spread_xp, 45);
        assert_eq!(budget.later_life_years, 0);
        assert_eq!(budget.later_life_rate, 15);
        assert_eq!(budget.later_life_xp, 0);
        assert_eq!(budget.total(), 120);
    }

    // --- `truncated_apprentice_years_completed`'s own paired-gate check -----
    //
    // F1's gate is "BOTH halves on the SAME item, naming the IDENTICAL
    // param" — every shipped item that carries either effect always pairs
    // them on the same param, so the mismatch branch
    // (`if !gate_resolved { return None; }`) is real but untested with real
    // data. This fixture deliberately declares TWO distinct parameters on one
    // item, one per effect, so `TruncatedApprenticeshipXp`'s `param` and
    // `ConfersHermeticTrainingIf`'s `param` disagree.

    const MISMATCHED_GATE_ITEMS: &str = r#"[
      { "id": "flaw.test_mismatched_gate", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "major", "categories": ["story"], "entity_kinds": ["character"],
        "parameters": [
          { "key": "years_completed", "type": { "number": { "min": 1, "max": 14 } }, "domain": "number" },
          { "key": "other_gate", "type": { "number": { "min": 1, "max": 14 } }, "domain": "number" }
        ],
        "effects": [
          { "type": "confers_hermetic_training_if", "param": "other_gate" },
          { "type": "truncated_apprenticeship_xp", "param": "years_completed" }
        ] },
      { "id": "flaw.test_filler_personality", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
    ]"#;
    const MISMATCHED_GATE_TYPES: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["story", "personality"], "creation_phases": [] }
    ]"#;
    /// Minimal apprenticeship block — `years: 15` matches both fixture
    /// items' `years_completed` bound (`max: 14`); empty ability lists
    /// sidestep the unrelated engine-required-abilities check entirely
    /// (mirrors `effective/hermetic_training.rs::TRUNCATED_LIFE_STAGES`).
    const MISMATCHED_GATE_LIFE_STAGES: &str = r#"{
      "apprenticeship": { "default_gauntlet_age": 25, "years": 15, "xp": 240,
                           "minimum_abilities": [], "recommended_abilities": [], "recommended_xp": 0,
                           "truncated_xp_per_year": 16, "truncated_spell_levels_per_year": 8 },
      "childhood": { "years": 5, "native_language_ability": "ability.living_language",
                     "native_language_xp": 75, "spread_xp": 45, "spread_abilities": [] },
      "later_life": { "xp_per_year": 15 }
    }"#;
    const MISMATCHED_GATE_ABILITIES: &str = r#"{
      "advancement": [{ "score": 1, "total_xp": 5 }],
      "abilities": [{ "id": "ability.living_language", "category": "general", "parameter": "language" }]
    }"#;

    fn mismatched_gate_ruleset() -> Ruleset {
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "test",
            version: "1",
            point_items: MISMATCHED_GATE_ITEMS,
            type_profiles: MISMATCHED_GATE_TYPES,
            abilities: Some(MISMATCHED_GATE_ABILITIES),
            life_stages: Some(MISMATCHED_GATE_LIFE_STAGES),
            ..crate::ruleset::RulesetSources::default()
        })
        .unwrap()
    }

    /// The positive control: the SAME param on both effects resolves normally,
    /// so `truncated_apprentice_years_completed` reads the filled-in value.
    #[test]
    fn truncated_apprentice_years_completed_reads_the_paired_params_value() {
        let mut entity = companion(vec![]);
        entity.selections = vec![Selection::with_params(
            Id::new("flaw.test_truncated_apprentice_paired"),
            std::collections::BTreeMap::from([("years_completed".into(), Id::new("7"))]),
        )];
        let items = r#"[
          { "id": "flaw.test_truncated_apprentice_paired", "kind": "flaw",
            "classification": "creation_effect", "magnitude": "major", "categories": ["story"],
            "entity_kinds": ["character"],
            "parameters": [{ "key": "years_completed", "type": { "number": { "min": 1, "max": 14 } }, "domain": "number" }],
            "effects": [
              { "type": "confers_hermetic_training_if", "param": "years_completed" },
              { "type": "truncated_apprenticeship_xp", "param": "years_completed" }
            ] },
          { "id": "flaw.test_filler_personality", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
        ]"#;
        let ruleset = Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: MISMATCHED_GATE_TYPES,
            abilities: Some(MISMATCHED_GATE_ABILITIES),
            life_stages: Some(MISMATCHED_GATE_LIFE_STAGES),
            ..crate::ruleset::RulesetSources::default()
        })
        .unwrap();
        assert_eq!(
            truncated_apprentice_years_completed(&entity, &ruleset),
            Some(7),
            "a paired gate on the SAME param must read the selection's own value"
        );
    }

    /// F1's mismatch branch: `TruncatedApprenticeshipXp`'s param
    /// (`years_completed`, filled in) and `ConfersHermeticTrainingIf`'s param
    /// (`other_gate`, a DIFFERENT key on the same item) never agree, so the
    /// gate never resolves — the function must return `None`, never guess at
    /// the unrelated value, even though `years_completed` itself IS answered.
    #[test]
    fn truncated_apprentice_years_completed_is_none_when_the_gate_param_differs() {
        let mut entity = companion(vec![]);
        entity.selections = vec![Selection::with_params(
            Id::new("flaw.test_mismatched_gate"),
            std::collections::BTreeMap::from([("years_completed".into(), Id::new("7"))]),
        )];
        let ruleset = mismatched_gate_ruleset();
        assert_eq!(
            truncated_apprentice_years_completed(&entity, &ruleset),
            None,
            "a TruncatedApprenticeshipXp whose sibling ConfersHermeticTrainingIf names a \
             DIFFERENT param must never resolve, regardless of its own param's value"
        );
    }
}
