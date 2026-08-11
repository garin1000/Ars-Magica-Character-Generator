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
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChildhoodRules {
    /// Years childhood covers ("the first five years of life").
    pub years: u32,
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
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LaterLifeRules {
    /// Experience per year of later life ("15 experience points per year").
    pub xp_per_year: u32,
}

impl LifeStageRules {
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
    use crate::types::{Entity, EntityKind, Id, RulesetRef, Selection};

    /// The shipped file's shape, so a rename or a retype fails here.
    const SHIPPED: &str = r#"{
      "childhood": {
        "years": 5,
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
}
