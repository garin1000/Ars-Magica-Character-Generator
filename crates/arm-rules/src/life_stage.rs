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
    /// The magus's apprenticeship, when the ruleset ships one. Optional because
    /// `:2364` calls apprenticeship and life as a magus "two **more** periods" — a
    /// ruleset with no Hermetic magi needs neither.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apprenticeship: Option<ApprenticeshipRules>,
    /// The first years of life, before any chosen advancement.
    pub childhood: ChildhoodRules,
    /// Every year after childhood, up to the character's age.
    pub later_life: LaterLifeRules,
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
/// numbers of `:2435` live in two files on purpose; see `RULES.md`.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2433-2437.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprenticeshipRules {
    /// Abilities the Order demands of every magus: "Magi must have the following
    /// minimum Abilities: Parma Magica 1, Magic Theory 1, Latin 1. Characters with
    /// lower scores would not be admitted to the Order." (`:2437`.)
    pub minimum_abilities: Vec<AbilityRequirement>,
    /// The Abilities of `#### Hermetic Magi Recommended Minimum Abilities`
    /// (`:2451-2461`) — advice, not admission, so a shortfall is a warning.
    pub recommended_abilities: Vec<AbilityRequirement>,
    /// What [`Self::recommended_abilities`] costs off the advancement table:
    /// "Total Cost: 90 experience points" (`:2461`). Carried as data so the load
    /// can re-price the list against it — the trust gate on transcribed numbers.
    pub recommended_xp: u32,
    /// Experience the years of apprenticeship grant ("240 experience points",
    /// `:2435`), spendable on Arts or Abilities alike.
    pub xp: u32,
    /// Years apprenticeship covers ("The fifteen years of apprenticeship", `:2435`).
    pub years: u32,
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
/// localization path (a German player types "Latein"). The field exists so a future
/// language registry can tighten the match by filling one JSON field rather than
/// changing code; see `RULES.md` for the consequence (a magus with Greek 1 passes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityRequirement {
    /// The Ability the requirement is about.
    pub ability: Id,
    /// The score it must reach.
    pub min_score: u8,
    /// The instance it must be, for a parameterized Ability. `None` accepts any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
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
    /// when it has no life-stage plan — direct entry, where [`Entity::xp_pool`] is
    /// the authority.
    ///
    /// An **unset age** still yields a budget: childhood is granted "in the first
    /// five years of life" with no further condition
    /// (Ars Magica - Definitive Edition (Core Rules).md:2378), so only later life
    /// scales with an age and an ageless plan simply lives 0 later-life years.
    /// Withholding the whole budget instead would leave childhood's two restricted
    /// pools at nothing and report every childhood row as unfunded — the validator
    /// names the missing age itself (`life_stage_age_unset`).
    pub fn budget(&self, entity: &Entity, ruleset: &Ruleset) -> Option<LifeStageBudget> {
        entity.life_stages.as_ref()?;
        let apprenticeship_years = self.apprenticeship_years(entity, ruleset);
        let later_life_years = entity
            .age
            .map_or(0, |age| self.later_life_years(age, apprenticeship_years));
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
    /// childhood, minus the `apprenticeship_years` that follow it (0 for anyone who
    /// serves no apprenticeship). Both blocks are fixed spans, so an age inside them
    /// yields 0 rather than a negative one (the validator reports such an age
    /// separately).
    ///
    /// **A magus's later life ends where its apprenticeship begins.** "**Later
    /// Life.** 15 experience points per year (until apprenticeship for magi)"
    /// (Ars Magica - Definitive Edition (Core Rules).md:2214), the same four periods
    /// `:2364` sets out. Counting every year to a magus's age would over-grant, and
    /// since Abilities and Arts buy from one shared pool the surplus would fund Arts
    /// as well. The character stands at its Gauntlet, so the span this leaves is the
    /// childhood-to-apprenticeship one — five years for a magus of 25, which is
    /// exactly the Darius example's "75 experience points to spend from those five
    /// years" (`:2402`).
    ///
    /// Life as a magus *after* the Gauntlet — "30 points per year" (`:2216`, `:2471`)
    /// — is **M6/6b5** and adds nothing here.
    pub fn later_life_years(&self, age: u32, apprenticeship_years: u32) -> u32 {
        age.saturating_sub(self.childhood.years.saturating_add(apprenticeship_years))
    }

    /// Years of apprenticeship this character serves: the block's own `years` for a
    /// magus, and 0 for anyone else — a grog or companion has no apprenticeship at
    /// all, and a ruleset shipping no block declares none.
    ///
    /// Read off the type profile's `is_magus` flag, never a type id.
    fn apprenticeship_years(&self, entity: &Entity, ruleset: &Ruleset) -> u32 {
        if !ruleset
            .profile(&entity.type_id)
            .is_some_and(|profile| profile.is_magus)
        {
            return 0;
        }
        self.apprenticeship
            .as_ref()
            .map_or(0, |apprenticeship| apprenticeship.years)
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

    /// The same shape plus the magus's apprenticeship block, so the optional third
    /// period parses with every field the rules state.
    const SHIPPED_WITH_APPRENTICESHIP: &str = r#"{
      "apprenticeship": {
        "years": 15,
        "xp": 240,
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
        "recommended_xp": 90
      },
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

    /// The same rules with the magus's apprenticeship block declared.
    fn rules_with_apprenticeship() -> LifeStageRules {
        serde_json::from_str(SHIPPED_WITH_APPRENTICESHIP).expect("the apprenticeship shape parses")
    }

    /// "For magi, there are two more periods to consider: apprenticeship, and life
    /// as a magus after that." (Core Rules.md:2364.) Apprenticeship is the third
    /// block a ruleset may ship: "The fifteen years of apprenticeship give the
    /// character 240 experience points" (`:2435`), with the minimum Abilities the
    /// Order demands (`:2437`) and the recommended package priced at 90 experience
    /// points (`:2451-2461`).
    ///
    /// Optional, because `:2364` calls these "two **more** periods": a non-Hermetic
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
                    min_score: 1,
                    parameter: None,
                },
                AbilityRequirement {
                    ability: Id::new("ability.magic_theory"),
                    min_score: 1,
                    parameter: None,
                },
                AbilityRequirement {
                    ability: Id::new("ability.parma_magica"),
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

    /// A requirement may name one instance of a parameterized Ability, so the field
    /// exists — and is omitted from JSON when unset, which is what keeps the shipped
    /// file free of a `"parameter": null` on every row.
    #[test]
    fn an_ability_requirement_omits_an_unset_parameter() {
        let requirement = AbilityRequirement {
            ability: Id::new("ability.dead_language"),
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
            "permitted_categories": ["general"], "creation_phases": [] },
          { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"], "is_magus": true, "creation_phases": [] }
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

    /// A magus's later life ends where its apprenticeship begins: "**Later Life.** 15
    /// experience points per year (until apprenticeship for magi)"
    /// (Core Rules.md:2214). Apprenticeship is fifteen years (`:2435`) and the
    /// character stands at its Gauntlet, so a magus of 25 was taken as an apprentice
    /// at 10 and lived five later-life years — exactly the arithmetic of the Darius
    /// example, whose master "picks 10 as a nice, round number" and who then "has 75
    /// experience points to spend from those five years" (`:2402`).
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
    /// `Entity::xp_pool` is the authority.
    #[test]
    fn there_is_no_budget_without_a_plan() {
        let rs = rate_ruleset();
        let mut entity = companion(vec![]);
        entity.age = Some(25);
        assert!(rules().budget(&entity, &rs).is_none(), "no plan");
    }

    /// Childhood is granted "in the first five years of life" unconditionally
    /// (Core Rules.md:2378) — only later life counts years up to an age. So a plan
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
}
