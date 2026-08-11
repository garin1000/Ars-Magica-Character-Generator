//! Life-stage experience: the blocks of XP a character accumulates before play.
//!
//! A character's Abilities are bought with experience earned in blocks, not from
//! one undifferentiated bank: "For grogs and companions they are acquired in two
//! blocks: early childhood, and later life."
//! (Source: Ars Magica - Definitive Edition (Core Rules).md:2364.) This module
//! turns an age into those blocks; the blocks then become funding pools for the
//! existing allocation solve in [`crate::effective::xp_allocation`], which is why
//! nothing here charges or spends anything itself.
//!
//! See `RULES.md` for the provenance of every value the shipped
//! `rules/core/life_stages.json` carries.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::effective::selections_for_effects;
use crate::ruleset::Ruleset;
use crate::types::{Effect, Entity, Id};

/// The life-stage experience rules, loaded from `rules/core/life_stages.json`.
// No `Default`: every field is authored data with no meaningful zero (a childhood
// of no years granting no experience is not a default, it is a broken file), and
// the ruleset holds these as an `Option` for the absent case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeStageRules {
    /// The first years of life, before any chosen advancement.
    pub childhood: ChildhoodRules,
    /// Every year after childhood, up to the character's age.
    pub later_life: LaterLifeRules,
}

/// Early childhood: a fixed block of years granting a native language and a
/// restricted spread of the Abilities a child picks up in play.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2378.
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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2392.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaterLifeRules {
    /// Experience per year of later life ("15 experience points per year").
    pub xp_per_year: u32,
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
/// Its presence is also the switch between the two ways of buying Abilities: with
/// a plan the budget below is authoritative and [`Entity::xp_pool`] must be 0
/// (`life_stage_xp_pool_conflict`); without one, `xp_pool` is the authority and
/// nothing here applies.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LifeStagePlan {
    /// The language the character grew up speaking — the one the childhood's
    /// native-language experience may be spent on, and the one a second Living
    /// Language may not be ("Living Language (other than the character's native
    /// language)", Core Rules.md:2378). A `living_language` instance value, so it is
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
    /// (Ars Magica - Definitive Edition (Core Rules).md:2382) leaves a package open
    /// to adjustment after it has been taken, so a character whose scores no longer
    /// match the package is legal, not an error.
    ///
    /// `None` for a character who divided the childhood experience by hand — and for
    /// every save written before the packages existed, which is why the field is
    /// additive and needs no schema bump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub childhood_package: Option<Id>,
}

/// The experience a life-stage plan earns, split into the blocks the rules grant
/// it in. Derived — never stored (see [`LifeStagePlan`]).
///
/// The three blocks fund different things, which is the whole reason they are kept
/// apart rather than summed: the native-language points buy one language and
/// nothing else, the spread buys only the childhood Abilities, and later life buys
/// anything the character is permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeStageBudget {
    /// Experience for the native language alone (75).
    pub childhood_native_xp: u32,
    /// Experience for the childhood spread (45).
    pub childhood_spread_xp: u32,
    /// Years of later life lived (age − childhood years).
    pub later_life_years: u32,
    /// Experience earned per year of later life for this character (15/20/10).
    pub later_life_rate: u32,
    /// Experience from later life (`later_life_years × later_life_rate`).
    pub later_life_xp: u32,
}

impl LifeStageBudget {
    /// Every point the character has earned, across all three blocks.
    pub fn total(&self) -> u32 {
        self.childhood_native_xp
            .saturating_add(self.childhood_spread_xp)
            .saturating_add(self.later_life_xp)
    }
}

impl LifeStageRules {
    /// The experience this character has earned through its life stages, or `None`
    /// when it has no life-stage plan (direct entry, where [`Entity::xp_pool`] is
    /// the authority) or no age (the yearly block cannot be counted).
    pub fn budget(&self, entity: &Entity, ruleset: &Ruleset) -> Option<LifeStageBudget> {
        entity.life_stages.as_ref()?;
        let age = entity.age?;
        let later_life_years = self.later_life_years(age);
        let later_life_rate = self.later_life_rate(entity, ruleset);
        Some(LifeStageBudget {
            childhood_native_xp: self.childhood.native_language_xp,
            childhood_spread_xp: self.childhood.spread_xp,
            later_life_years,
            later_life_rate,
            later_life_xp: later_life_years.saturating_mul(later_life_rate),
        })
    }

    /// Years of later life a character of `age` has lived: every year after
    /// childhood. Childhood is a fixed block, so an age inside it yields 0 rather
    /// than a negative span (the validator reports such an age separately).
    pub fn later_life_years(&self, age: u32) -> u32 {
        age.saturating_sub(self.childhood.years)
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
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2392, :2394.
    pub fn later_life_rate(&self, entity: &Entity, ruleset: &Ruleset) -> u32 {
        // The base rate is not one of the candidates: a named rate replaces it
        // outright, so `min` is taken over the named rates only (or the base stands
        // when nothing names one). Folding the base in would make Wealthy's 20 lose
        // to it.
        let mut named: Option<u32> = None;
        for selection in selections_for_effects(entity, ruleset).iter() {
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
    use crate::types::{Entity, EntityKind, Id, RulesetRef, SCHEMA_VERSION, Selection};

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

    fn rules() -> LifeStageRules {
        serde_json::from_str(SHIPPED).expect("the shipped life-stage shape parses")
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
    /// experience are `age - childhood.years` (Core Rules.md:2378, :2392).
    #[test]
    fn later_life_years_start_after_childhood() {
        let rules = rules();
        assert_eq!(rules.later_life_years(25), 20);
        // A five-year-old has finished childhood and no more.
        assert_eq!(rules.later_life_years(5), 0);
    }

    /// An age below the childhood span is not a shorter childhood — childhood is a
    /// fixed five-year block — so it earns no later-life years at all rather than
    /// underflowing into a huge count.
    #[test]
    fn an_age_inside_childhood_earns_no_later_life_years() {
        assert_eq!(rules().later_life_years(3), 0);
    }

    /// A ruleset carrying the two rate-bearing items, so the per-year rate can be
    /// read off a character's selections rather than hardcoded.
    fn rate_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "virtue.wealthy", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "major", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "later_life_xp_rate", "amount": 20 }] },
          { "id": "flaw.poor", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "major", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "later_life_xp_rate", "amount": 10 }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"], "creation_phases": [] }
        ]"#;
        Ruleset::from_json("test", "1", items, types).unwrap()
    }

    fn companion(selections: Vec<&str>) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.selections = selections
            .into_iter()
            .map(|r| Selection::new(Id::new(r)))
            .collect();
        entity
    }

    /// Wealthy and Poor replace the yearly rate rather than adding to it: "get 20
    /// experience points per year" / "get 10 experience points per year".
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2394.
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
        });
        entity.normalize();
        let json = serde_json::to_string_pretty(&entity).unwrap();
        assert!(
            json.contains(r#""childhood_package": "childhood.athletic""#),
            "{json}"
        );
        assert!(json.contains(r#""schema_version": 14"#), "{json}");

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
    /// `Entity::xp_pool` is the authority. Nor is there one without an age, since
    /// the yearly block cannot be counted.
    #[test]
    fn there_is_no_budget_without_a_plan_or_an_age() {
        let rs = rate_ruleset();
        let mut entity = companion(vec![]);
        entity.age = Some(25);
        assert!(rules().budget(&entity, &rs).is_none(), "no plan");

        entity.life_stages = Some(LifeStagePlan::default());
        entity.age = None;
        assert!(rules().budget(&entity, &rs).is_none(), "no age");
    }
}
