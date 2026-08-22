//! Experience allocation: Affinity cost reduction, restricted XP pools, and the
//! max-flow solve that funds Ability/Art/Spell-Mastery spends across the
//! general pool and every restricted pool. Split out of `effective.rs`
//! (Viktor's V4 architecture finding — 93 free functions across 7 unrelated
//! domains in one file); pure code motion, no behavior change.
//!
//! `MAX_XP_SOLVE_NODES` is a security fix (audit finding K3): an unbounded
//! flow-solve matrix over a crafted save's selections could force a
//! multi-gigabyte single allocation and abort the process on File → Open.
//! [`checked_xp_allocation`] is the guarded, `pub` entry point every caller
//! outside this module should use; the raw [`xp_allocation`] is `pub(crate)`
//! and keeps only an internal `assert!` backstop (audit finding K1, round 2:
//! the guard used to live solely in that `assert!` and in one caller's
//! discipline, and three other call sites grew directly against the
//! unguarded function — see [`checked_xp_allocation`]'s docs for the fix).

use super::*;

/// The experience charged against a pool for a bought score whose advancement
/// table cost is `table_xp`, under an optional Affinity multiplier.
///
/// Affinity (Ability/Art) says creation XP "counts as" `num/den` of itself
/// (3/2, rounded up): so the points actually charged to reach a fixed table cost
/// `T` are the smallest `c` with `ceil(c·num/den) ≥ T`, which is
/// `ceil(T·den/num)`. The worked example (Perdo 10, Art table T=55, 3/2):
/// `ceil(55·2/3) = ceil(36.67) = 37`, which the rules say counts as 56 ≥ 55.
/// Integer-only so the engine stays exact.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3372-3378, worked
/// example `:2443`.
pub(crate) fn charged_cost(table_xp: u32, affinity: Option<(u8, u8)>) -> u32 {
    match affinity {
        Some((num, den)) if num != 0 => table_xp
            .saturating_mul(u32::from(den))
            .div_ceil(u32::from(num)),
        _ => table_xp,
    }
}

/// Of several Affinity multipliers on one target, the one giving the greatest
/// cost reduction. Affinities do not stack, so the single most generous wins.
/// A score "counts as `num/den` of itself", charged `table·den/num`, so a larger
/// `num/den` is cheaper — the most generous is `max(num/den)`. Compares
/// `n1/d1` vs `n2/d2` as `n1·d2` vs `n2·d1` to stay in integer arithmetic.
///
/// `pub(crate)` (not just module-private) because
/// [`spell_mastery_advancement_affinity`](crate::effective::spell_mastery_advancement_affinity)
/// (the `spell` domain) needs the same reduction for Flawless Magic's mastery
/// Affinity.
pub(crate) fn best_affinity(multipliers: impl Iterator<Item = (u8, u8)>) -> Option<(u8, u8)> {
    multipliers.reduce(|a, b| {
        let (an, ad) = (u32::from(a.0), u32::from(a.1));
        let (bn, bd) = (u32::from(b.0), u32::from(b.1));
        if an * bd >= bn * ad { a } else { b }
    })
}

/// The Affinity multiplier applying to one ability instance, if any
/// ([`Effect::AffinityAbilityCost`] targeting it). Matches the instance exactly,
/// like [`ability_bonus`]. `pub(crate)` so the age-cap validator can read whether
/// an Ability carries an Affinity (which raises its age cap by +2, Ars Magica - Definitive Edition (Core Rules).md:3374).
pub(crate) fn ability_affinity(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> Option<(u8, u8)> {
    let instance_key = ruleset
        .abilities
        .get(ability)
        .and_then(|a| a.parameter.as_deref());
    let selections = selections_for_effects(entity, ruleset);
    let found = selections.iter().flat_map(|selection| {
        let item = ruleset.point_items.get(&selection.item_ref);
        item.into_iter()
            .flat_map(|item| &item.effects)
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-ignored cost reduction.
            .filter_map(move |effect| match effect {
                Effect::AffinityAbilityCost {
                    param,
                    counts_as_num,
                    counts_as_den,
                } if selection.params.get(param) == Some(ability) => {
                    let matches = match instance_key {
                        None => true,
                        Some(key) => selection.params.get(key).map(Id::as_str) == parameter,
                    };
                    matches.then_some((*counts_as_num, *counts_as_den))
                }
                // A group Affinity (Linguist) covers a fixed set of ability ids,
                // any instance — so it matches by id regardless of `parameter`.
                Effect::GroupAffinityCost {
                    abilities,
                    counts_as_num,
                    counts_as_den,
                } if abilities.contains(ability) => Some((*counts_as_num, *counts_as_den)),
                // Not an Affinity for this ability instance; no reduction here.
                irrelevant_effect_variants!() => None,
            })
    });
    best_affinity(found)
}

/// The Affinity multiplier applying to one Art, if any
/// ([`Effect::AffinityArtCost`] targeting it). Arts are matched by id alone.
fn art_affinity(entity: &Entity, ruleset: &Ruleset, art: &Id) -> Option<(u8, u8)> {
    let selections = selections_for_effects(entity, ruleset);
    let found = selections.iter().flat_map(|selection| {
        let item = ruleset.point_items.get(&selection.item_ref);
        item.into_iter()
            .flat_map(|item| &item.effects)
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-ignored cost reduction.
            .filter_map(move |effect| match effect {
                Effect::AffinityArtCost {
                    param,
                    counts_as_num,
                    counts_as_den,
                } if selection.params.get(param) == Some(art) => {
                    Some((*counts_as_num, *counts_as_den))
                }
                // Not an Affinity for this Art; no reduction here.
                irrelevant_effect_variants!() => None,
            })
    });
    best_affinity(found)
}

/// A restricted experience pool granted by a virtue, with how much of it the
/// character's eligible spends actually consume (from the allocation). Serializes
/// for the frontend so the XP bar can show each pool's `used`/`amount`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestrictedXpPool {
    /// Points granted to this pool.
    pub amount: u32,
    /// Points the allocation draws from this pool (≤ `amount`; remainder wasted).
    pub used: u32,
    /// Eligible ability ids (empty when eligibility is purely by category).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub abilities: Vec<Id>,
    /// Eligible ability categories (empty when eligibility is purely by id).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<AbilityCategory>,
    /// Where the pool came from, so the UI can label it. Without this the XP bar
    /// would have to infer a name from the ability list, which cannot distinguish
    /// childhood's two blocks (both list childhood Abilities) and would read as a
    /// V/F grant.
    pub origin: XpPoolOrigin,
}

/// Where a restricted XP pool came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum XpPoolOrigin {
    /// A Virtue/Flaw grant (Educated, Warrior, Privileged Upbringing, …). The UI
    /// labels it with the item's own localized name.
    Item {
        /// The granting item.
        item: Id,
    },
    /// A block of life-stage experience. Labelled through a Fluent key on the
    /// block, since a life stage is not an item and has no i18n entry.
    LifeStage {
        /// Which block.
        block: LifeStageBlock,
    },
}

/// A block of life-stage experience that funds purchases on its own terms.
///
/// A fixed taxonomy (the rules grant exactly these), so an enum: adding a block is
/// a compile error until the UI labels it.
///
/// **Apprenticeship is absent, and later life is present.** Whichever block funds
/// anything the character may learn is the *general* pool and needs no slug: for a
/// magus that is apprenticeship, whose experience "can be spent on Arts or
/// Abilities" (Ars Magica - Definitive Edition (Core Rules).md:2435). Later life buys "any **Abilities**" (`:2214`,
/// `:2392`) and, for a magus, ends where apprenticeship begins — so it is a
/// restricted pool of its own, listed here. For a grog or companion later life is
/// still the general pool; the enum names the blocks that *can* be restricted, and
/// which pools a character actually gets is decided in [`xp_allocation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifeStageBlock {
    /// Childhood's native-language experience: spendable only on the native
    /// language instance (Ars Magica - Definitive Edition (Core Rules).md:2378).
    ChildhoodNativeLanguage,
    /// Childhood's restricted spread: spendable only on the childhood Ability list,
    /// and never on the native language (`:2378`).
    ChildhoodSpread,
    /// Later life: for a magus, the years before apprenticeship, spendable on
    /// Abilities alone and never on an Art (`:2214`, `:2392`).
    LaterLife,
}

impl LifeStageBlock {
    /// Every block, the single source of the set (the UI's labels are checked
    /// against it).
    pub const ALL: [LifeStageBlock; 3] = [
        LifeStageBlock::ChildhoodNativeLanguage,
        LifeStageBlock::ChildhoodSpread,
        LifeStageBlock::LaterLife,
    ];
}

impl fmt::Display for LifeStageBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LifeStageBlock::ChildhoodNativeLanguage => "childhood_native_language",
            LifeStageBlock::ChildhoodSpread => "childhood_spread",
            LifeStageBlock::LaterLife => "later_life",
        })
    }
}

/// One instance of an ability: the id plus, for a parameterized ability, the
/// instance value ("Living Language (German)"). Childhood's blocks need this
/// granularity — the 75 points buy the native language and the 45 may buy any
/// OTHER Living Language — which an id alone cannot express.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityInstanceRef {
    /// The ability.
    pub ability: Id,
    /// The instance value, for a parameterized ability. `None` matches the ability
    /// whatever its instance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
}

impl AbilityInstanceRef {
    /// Whether this ref names the given bought instance.
    fn matches(&self, ability: &Id, parameter: Option<&str>) -> bool {
        self.ability == *ability
            && (self.parameter.is_none() || self.parameter.as_deref() == parameter)
    }
}

/// The result of allocating Ability + Art spends across the general experience
/// pool and any restricted pools (Educated/Warrior/Privileged). Computed by a
/// max-flow feasibility solve; `total_demand > max_flow` means the spends cannot
/// all be funded (overspend by `total_demand - max_flow`).
///
/// Serializes like its sibling result types (`RestrictedXpPool`, `AbilityBonus`,
/// `Balance`, …) so a Tauri command can hand the full allocation to the frontend
/// directly — including `max_flow`/`general_pool`, which let the UI surface the
/// overspend delta — rather than reshaping a subset of fields at the IPC edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XpAllocation {
    /// Sum of every spend's charged cost (post-Affinity).
    pub total_demand: u32,
    /// Maximum demand that can be funded. Equals `total_demand` iff legal.
    pub max_flow: u32,
    /// The general pool size: the block's base (`Entity::xp_pool`, or the life-stage
    /// block that may fund anything) **plus** [`XpAllocation::general_bonus`].
    pub general_pool: u32,
    /// The signed [`Effect::GeneralXp`] contribution folded into `general_pool`
    /// (Skilled Parens +60, Weak Parens -60), reported on its own so a bar can name
    /// it beside the base rather than leaving the two numbers unexplained.
    pub general_bonus: i64,
    /// Points drawn from the general pool by the allocation.
    pub general_used: u32,
    /// The restricted pools with their consumed amounts.
    pub restricted: Vec<RestrictedXpPool>,
}

/// One bought score's funding demand for the flow solve, tagged with what it buys
/// so the kind-specific restricted pools know whether they may fund it.
struct Spend {
    cost: u32,
    kind: SpendKind,
}

/// What a [`Spend`] buys — decides which restricted pools may fund it (the general
/// pool always can). Ability spends draw RestrictedAbilityXp pools; Mastery spends
/// draw SpellMasteryXp pools; the two never cross, and Arts have no restricted pool.
enum SpendKind {
    /// An Ability score, eligible for RestrictedAbilityXp and life-stage pools. The
    /// instance value travels with it because childhood's blocks are
    /// instance-restricted (the native language against every other one).
    Ability {
        ability: Id,
        category: AbilityCategory,
        parameter: Option<String>,
    },
    /// An Art score — funded from the general pool only.
    Art,
    /// A per-spell Spell Mastery Ability, eligible for SpellMasteryXp pools only.
    Mastery,
}

/// A restricted pool's funding scope for the flow solve.
enum PoolEligibility {
    /// An ability-XP grant (Educated/Warrior/Privileged) or a life-stage block:
    /// funds an Ability whose id, category or instance is listed, minus anything
    /// `exclude` names. Never Arts, never Mastery.
    Ability {
        abilities: Vec<Id>,
        categories: Vec<AbilityCategory>,
        /// Specific instances this pool funds. When non-empty it is the ONLY test —
        /// childhood's native-language block funds one instance and nothing else,
        /// which `abilities` (id-only) cannot express.
        instances: Vec<AbilityInstanceRef>,
        /// Instances this pool never funds, even when `abilities`/`categories`
        /// would cover them: childhood's spread excludes the native language.
        exclude: Vec<AbilityInstanceRef>,
    },
    /// A Spell-Mastery grant (Mastered Spells): funds only Spell Mastery spends.
    Mastery,
}

/// One restricted pool in the flow graph: its capacity, what it may fund, and
/// where it came from (carried through to the surfaced pool so the UI can name it).
struct FlowPool {
    amount: u32,
    eligibility: PoolEligibility,
    origin: XpPoolOrigin,
}

/// The instance childhood's native-language experience may be spent on: the
/// ability the rules data names for it, at the language the plan chose. `None`
/// when either is unset — the validator reports an unset language
/// (`life_stage_native_language_unset`), and no pool is created for a language
/// nobody picked.
fn native_language_instance(
    entity: &Entity,
    rules: &crate::life_stage::LifeStageRules,
) -> Option<AbilityInstanceRef> {
    let language = entity.life_stages.as_ref()?.native_language.clone()?;
    Some(AbilityInstanceRef {
        ability: rules.childhood.native_language_ability.clone(),
        parameter: Some(language),
    })
}

/// The Abilities and categories the character's selections permit.
///
/// A Virtue grants access two ways, and both count: an explicit
/// [`Effect::AbilityAuthorization`], or any [`Effect::RestrictedAbilityXp`] pool —
/// experience earmarked for a category is evidence the category is permitted, which
/// is what makes Warrior (Martial XP) and Arcane Lore (Arcane XP) work without
/// further data.
///
/// It lives here rather than in `validation/` because both readers need it and the
/// layering only runs one way: `validation` already depends on `effective`
/// ([`validate_ability_authorization`](crate::validation) calls
/// [`selections_for_effects`]), so `effective` calling back into `validation` would
/// invert it. The two readers are that validator, which gates *owning* a gated
/// Ability, and [`xp_allocation`], which decides which of a magus's blocks may
/// *fund* one.
pub(crate) fn ability_authorizations(
    entity: &Entity,
    ruleset: &Ruleset,
) -> (BTreeSet<Id>, BTreeSet<AbilityCategory>) {
    let mut abilities = BTreeSet::new();
    let mut categories = BTreeSet::new();
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            match effect {
                // Experience earmarked for a category or Ability is itself
                // permission to learn it — otherwise the grant could never be spent.
                Effect::RestrictedAbilityXp {
                    abilities: ids,
                    categories: cats,
                    ..
                }
                | Effect::AbilityAuthorization {
                    abilities: ids,
                    categories: cats,
                } => {
                    abilities.extend(ids.iter().cloned());
                    categories.extend(cats.iter().copied());
                }
                // A free score in an Ability is permission to have it, since the
                // Virtue confers the Ability outright.
                Effect::AbilityScoreGrant { ability, .. } => {
                    abilities.insert(ability.clone());
                }
                _ => {}
            }
        }
    }
    (abilities, categories)
}

/// Whether a restricted pool may fund a spend. Ability pools cover only Ability
/// spends they list (by id or category); Mastery pools cover only Mastery spends.
/// No pool covers an Art (general pool only), and the two pool kinds never cross.
fn pool_covers(eligibility: &PoolEligibility, spend: &Spend) -> bool {
    match (eligibility, &spend.kind) {
        (
            PoolEligibility::Ability {
                abilities,
                categories,
                instances,
                exclude,
            },
            SpendKind::Ability {
                ability,
                category,
                parameter,
            },
        ) => {
            let parameter = parameter.as_deref();
            if exclude.iter().any(|e| e.matches(ability, parameter)) {
                return false;
            }
            if !instances.is_empty() {
                // An instance list is exhaustive for this pool, not additive: the
                // native-language block funds exactly its one instance.
                return instances.iter().any(|i| i.matches(ability, parameter));
            }
            abilities.contains(ability) || categories.contains(category)
        }
        (PoolEligibility::Mastery, SpendKind::Mastery) => true,
        // Every remaining combination is explicitly uncovered, so a new
        // PoolEligibility or SpendKind variant forces a decision here rather than
        // silently defaulting to false: Ability pools never fund Arts or Mastery,
        // and Mastery pools never fund Abilities or Arts.
        (PoolEligibility::Ability { .. }, SpendKind::Art)
        | (PoolEligibility::Ability { .. }, SpendKind::Mastery)
        | (PoolEligibility::Mastery, SpendKind::Ability { .. })
        | (PoolEligibility::Mastery, SpendKind::Art) => false,
    }
}

/// Hard ceiling on the flow-solve node count (`3 + flow_pools.len() +
/// spends.len()`) [`xp_allocation`] will build a dense residual matrix for. See
/// the `assert!` at its call site for the full rationale; in short, this bounds
/// an entity's own plausible selections — never the ruleset's catalogue size —
/// to keep the `n * n` matrix and the Edmonds-Karp solve cheap regardless of how
/// large a crafted save's `ability_scores`/`art_scores`/`spells`/`selections`
/// arrays are.
///
/// `pub(crate)` (not otherwise used outside this module) so
/// [`crate::validation::magus::validate_xp_pool`] can compare against the same
/// constant `xp_allocation` enforces, rather than restating the number: the
/// validation layer rejects a hostile save with a structured issue *before*
/// ever calling `xp_allocation`, and this `assert!` remains the unbypassable
/// backstop for any caller that skips validation.
pub(crate) const MAX_XP_SOLVE_NODES: usize = 2048;

/// The counts behind the flow-solve node total [`xp_allocation`] would need for
/// this entity, without building its `n x n` matrix — `build_spends` and
/// `build_flow_pools` are both linear in the entity's own selections, so
/// computing this is cheap even for a pathological save. Lets the validation
/// layer reject an entity that would exceed [`MAX_XP_SOLVE_NODES`] with a
/// structured issue naming the offending counts, instead of ever reaching the
/// solver's `assert!`.
pub(crate) struct XpSolveScale {
    /// `spends.len()` — bought Ability/Art scores plus mastered spells.
    pub spends: usize,
    /// `flow_pools.len()` — restricted XP pools (grants and life-stage blocks).
    pub flow_pools: usize,
}

impl XpSolveScale {
    /// The `n` [`xp_allocation`] would build its matrix at: `3 + flow_pools +
    /// spends` (source, sink, general pool, then one node per pool/spend).
    pub(crate) fn nodes(&self) -> usize {
        3 + self.flow_pools + self.spends
    }
}

/// Computes [`XpSolveScale`] for `entity` — see its docs.
pub(crate) fn xp_solve_scale(entity: &Entity, ruleset: &Ruleset) -> XpSolveScale {
    XpSolveScale {
        spends: build_spends(entity, ruleset).len(),
        flow_pools: build_flow_pools(entity, ruleset).len(),
    }
}

/// Builds one [`Spend`] per bought Ability score, Art score, and per-spell Spell
/// Mastery score — the demand side of [`xp_allocation`]'s flow solve. Extracted
/// from `xp_allocation` (pure code motion, no behavior change): Abilities, Arts,
/// and Spell Mastery are independently priced and this needs none of
/// [`build_flow_pools`]'s or the graph-solve's locals.
fn build_spends(entity: &Entity, ruleset: &Ruleset) -> Vec<Spend> {
    // Spends: abilities (Affinity-reduced, with category for eligibility) + arts +
    // per-spell Spell Mastery Abilities.
    let mut spends: Vec<Spend> = Vec::new();
    for a in &entity.ability_scores {
        let Some(table) = ruleset.advancement.xp_for_score(a.score) else {
            continue;
        };
        // A Virtue-granted Supernatural-Ability floor (e.g. Second Sight 1) is
        // free: the player "will not need to spend experience points for the
        // first point". So only the score above the granted floor is charged —
        // the floor's own table cost is subtracted before Affinity is applied.
        // Source: Ars Magica - Definitive Edition (Core Rules).md:2639.
        let floor = granted_ability_floor(entity, ruleset, &a.ability, a.parameter.as_deref());
        let floor_table = u8::try_from(floor)
            .ok()
            .filter(|f| *f > 0)
            .and_then(|f| ruleset.advancement.xp_for_score(f))
            .unwrap_or(0);
        let payable = table.saturating_sub(floor_table);
        let cost = charged_cost(
            payable,
            ability_affinity(entity, ruleset, &a.ability, a.parameter.as_deref()),
        );
        // A catalogue-known ability carries its category (for restricted-pool
        // eligibility); an unknown one funds from the general pool only, like an Art.
        let kind = ruleset
            .abilities
            .get(&a.ability)
            .map(|def| SpendKind::Ability {
                ability: a.ability.clone(),
                category: def.category,
                parameter: a.parameter.clone(),
            })
            .unwrap_or(SpendKind::Art);
        spends.push(Spend { cost, kind });
    }
    for a in &entity.art_scores {
        let Some(table) = ruleset.art_advancement.xp_for_score(a.score) else {
            continue;
        };
        let cost = charged_cost(table, art_affinity(entity, ruleset, &a.art));
        spends.push(Spend {
            cost,
            kind: SpendKind::Art,
        });
    }
    // Spell Mastery is an Ability (Ars Magica - Definitive Edition (Core Rules).md:9516, :7143) bought from the
    // Ability advancement table (:15952, :15956-15979). Flawless Magic auto-masters
    // every spell at a free floor (charge only above it, like a granted Supernatural
    // floor) AND doubles all mastery Advancement Totals (an Affinity that halves the
    // charge). The mastery pool (Mastered Spells) — not the ability-restricted pools
    // — plus the general pool fund it.
    // Source: Ars Magica - Definitive Edition (Core Rules).md:3887-3889, :4471-4474.
    let mastery_floor = spell_mastery_floor(entity, ruleset);
    let mastery_floor_table = if mastery_floor > 0 {
        ruleset.advancement.xp_for_score(mastery_floor).unwrap_or(0)
    } else {
        0
    };
    let mastery_affinity = spell_mastery_advancement_affinity(entity, ruleset);
    for spell in &entity.spells {
        let bought = spell.mastery.unwrap_or(0);
        if bought == 0 {
            continue;
        }
        let Some(table) = ruleset.advancement.xp_for_score(bought) else {
            continue;
        };
        let payable = table.saturating_sub(mastery_floor_table);
        let cost = charged_cost(payable, mastery_affinity);
        if cost == 0 {
            continue;
        }
        spends.push(Spend {
            cost,
            kind: SpendKind::Mastery,
        });
    }
    spends
}

/// Builds every restricted [`FlowPool`] the entity's selections and life stages
/// grant — the non-general supply side of [`xp_allocation`]'s flow solve.
/// Extracted from `xp_allocation` (pure code motion, no behavior change): the
/// general pool is computed separately (it needs [`general_xp_bonus`] and the
/// life-stage/magus flags again, recomputed there — both are cheap, pure lookups
/// with no side effects, so recomputing costs nothing and keeps this function
/// independent of the graph-solve locals).
fn build_flow_pools(entity: &Entity, ruleset: &Ruleset) -> Vec<FlowPool> {
    // Restricted pools: one per RestrictedAbilityXp effect (Educated/Warrior/…),
    // plus a single Spell-Mastery pool (Mastered Spells, summed). The mastery pool
    // is flow-only — it is not surfaced in `restricted`, which the UI reserves for
    // ability-XP grants.
    let mut flow_pools: Vec<FlowPool> = Vec::new();
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::RestrictedAbilityXp {
                amount,
                abilities,
                categories,
            } = effect
            {
                flow_pools.push(FlowPool {
                    amount: *amount,
                    eligibility: PoolEligibility::Ability {
                        abilities: abilities.clone(),
                        categories: categories.clone(),
                        // A V/F grant is id/category-scoped, never instance-scoped.
                        instances: Vec::new(),
                        exclude: Vec::new(),
                    },
                    origin: XpPoolOrigin::Item {
                        item: selection.item_ref.clone(),
                    },
                });
            }
        }
    }
    // Childhood's two blocks, for a character built through its life stages. Both
    // are restricted pools rather than budget added to the general one, because
    // each may buy only its own things: the 75 the native language, the 45 the
    // childhood list minus that language.
    // Source: Ars Magica - Definitive Edition (Core Rules).md:2378.
    // Which block is the general pool depends on whether the character serves an
    // apprenticeship, so the flag is read once here — off the profile, never a type id.
    let is_magus = ruleset
        .profile(&entity.type_id)
        .is_some_and(|profile| profile.is_magus);
    let life_stage_budget = ruleset
        .life_stages()
        .and_then(|rules| rules.budget(entity, ruleset).map(|budget| (rules, budget)));
    if let Some((rules, budget)) = &life_stage_budget {
        let native = native_language_instance(entity, rules);
        if let Some(native) = &native {
            flow_pools.push(FlowPool {
                amount: budget.childhood_native_xp,
                eligibility: PoolEligibility::Ability {
                    abilities: Vec::new(),
                    categories: Vec::new(),
                    instances: vec![native.clone()],
                    exclude: Vec::new(),
                },
                origin: XpPoolOrigin::LifeStage {
                    block: LifeStageBlock::ChildhoodNativeLanguage,
                },
            });
        }
        flow_pools.push(FlowPool {
            amount: budget.childhood_spread_xp,
            eligibility: PoolEligibility::Ability {
                abilities: rules.childhood.spread_abilities.iter().cloned().collect(),
                categories: Vec::new(),
                instances: Vec::new(),
                // "Living Language (other than the character's native language)":
                // the spread may buy a second language, never the native one.
                exclude: native.into_iter().collect(),
            },
            origin: XpPoolOrigin::LifeStage {
                block: LifeStageBlock::ChildhoodSpread,
            },
        });
        // A magus's later life is a restricted pool of its own: the years between
        // childhood and being taken as an apprentice, which buy "any Abilities"
        // (`:2214`) and never an Art, and not an Arcane, Academic or Martial Ability
        // either — "magi can only spend experience points on Arcane, Academic and
        // Martial Abilities before apprenticeship if they have a Virtue which allows
        // them to do so" (`:2435`). A Virtue that does allow it (Covenant Upbringing,
        // Educated, Warrior) widens the pool through the same authorizations the
        // ownership check reads, so the two cannot disagree.
        //
        // Supernatural stays in the set and legalizes nothing: access to each
        // Supernatural Ability is granted per Ability, which
        // `validate_supernatural_abilities` enforces for magi too — so an
        // unauthorized one is already an error and funding it here changes nothing.
        //
        // For a grog or companion no such pool is pushed: later life is their general
        // pool (`:2392`), and the categories are gated by an error on the character
        // instead. A magus's category gate is waived whole-character (`:7151`), so the
        // pool is the only place the "before apprenticeship" half can live.
        if is_magus && budget.later_life_xp > 0 {
            let (abilities, authorized_categories) = ability_authorizations(entity, ruleset);
            let gated = ruleset.categories_requiring_virtue();
            flow_pools.push(FlowPool {
                amount: budget.later_life_xp,
                eligibility: PoolEligibility::Ability {
                    abilities: abilities.into_iter().collect(),
                    categories: AbilityCategory::ALL
                        .into_iter()
                        .filter(|category| {
                            !gated.contains(category) || authorized_categories.contains(category)
                        })
                        .collect(),
                    instances: Vec::new(),
                    exclude: Vec::new(),
                },
                origin: XpPoolOrigin::LifeStage {
                    block: LifeStageBlock::LaterLife,
                },
            });
        }
    }
    let mastery_pool = spell_mastery_xp(entity, ruleset);
    if mastery_pool > 0 {
        flow_pools.push(FlowPool {
            amount: mastery_pool,
            eligibility: PoolEligibility::Mastery,
            // Never surfaced (see `restricted` below), so its origin is nominal.
            origin: XpPoolOrigin::LifeStage {
                block: LifeStageBlock::ChildhoodSpread,
            },
        });
    }
    flow_pools
}

/// Why [`checked_xp_allocation`] refused to run the flow solve: the entity's
/// own selections would need more flow-solve nodes than [`MAX_XP_SOLVE_NODES`]
/// allows. Carries the same counts `CODE_XP_SOLVE_BOUND_EXCEEDED` names, so a
/// caller can render a fallback or a diagnostic without recomputing
/// [`XpSolveScale`] itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XpSolveBoundExceeded {
    /// The flow-solve node count this entity would need.
    pub nodes: usize,
    /// The bound it exceeded ([`MAX_XP_SOLVE_NODES`]).
    pub limit: usize,
    /// `spends.len()` behind `nodes`.
    pub spends: usize,
    /// `flow_pools.len()` behind `nodes`.
    pub flow_pools: usize,
}

/// The only way to run the flow solve on an **untrusted** `Entity` — a
/// `.armc`/`.armcov` save that may have been hand-edited or come from
/// anywhere. Checks [`XpSolveScale::nodes`] against [`MAX_XP_SOLVE_NODES`]
/// *before* calling [`xp_allocation`] and returns [`XpSolveBoundExceeded`]
/// instead of ever building the oversized matrix.
///
/// This is the fix for audit finding K1 (round 2): [`xp_allocation`] itself
/// used to be `pub`, and three call sites (`arm-app`'s `ruleset_io::xp_fields`,
/// `export/sections.rs`, and [`restricted_xp_pools`] below) called it directly
/// with no bound check, reaching its internal `assert!` — a panic, on a plain
/// File → Open or File → Export Markdown of a hostile save — instead of the
/// friendly `CODE_XP_SOLVE_BOUND_EXCEEDED` the validation layer produces.
/// `xp_allocation` is now `pub(crate)`, so this function is the *only* route
/// to it reachable from another crate: the compiler, not caller discipline,
/// keeps a future `arm-app` call site from regrowing the gap.
/// [`crate::validation::magus::validate_xp_pool`] uses this too, folding its
/// own rejection into the one call rather than recomputing the scale
/// separately.
pub fn checked_xp_allocation(
    entity: &Entity,
    ruleset: &Ruleset,
) -> Result<XpAllocation, XpSolveBoundExceeded> {
    let scale = xp_solve_scale(entity, ruleset);
    let nodes = scale.nodes();
    if nodes > MAX_XP_SOLVE_NODES {
        return Err(XpSolveBoundExceeded {
            nodes,
            limit: MAX_XP_SOLVE_NODES,
            spends: scale.spends,
            flow_pools: scale.flow_pools,
        });
    }
    Ok(xp_allocation(entity, ruleset))
}

/// Allocates the entity's Ability + Art spends across the general pool and every
/// restricted pool, by max-flow feasibility. The general pool funds any spend;
/// each restricted pool funds only its eligible Abilities; overlapping
/// eligibility is resolved globally (greedy assignment would strand capacity).
/// A score the advancement table cannot price contributes 0 (already flagged by
/// `validate_abilities`/`validate_arts`).
///
/// `pub(crate)`, not `pub`: the raw solver trusts its caller to have already
/// checked the flow-solve node count. [`checked_xp_allocation`] is the
/// pre-checked entry point every caller outside this module should use —
/// see its docs for why (audit finding K1).
///
/// # Panics
/// Panics if the entity's own selections are so numerous that the flow-solve
/// matrix would exceed [`MAX_XP_SOLVE_NODES`] — see the `assert!` inside for why
/// this is a deliberate safety bound, not a game-rule limit. Kept as an
/// internal invariant now that this function is `pub(crate)`: every route
/// reachable from outside this crate goes through [`checked_xp_allocation`],
/// which pre-checks the identical bound, so an untrusted `Entity` can no
/// longer drive `n` past the limit and reach this `assert!` at all. It
/// remains as a hard stop for any future in-crate caller that bypasses the
/// checked wrapper.
pub(crate) fn xp_allocation(entity: &Entity, ruleset: &Ruleset) -> XpAllocation {
    let spends = build_spends(entity, ruleset);
    let flow_pools = build_flow_pools(entity, ruleset);

    let total_demand: u32 = spends.iter().map(|s| s.cost).sum();
    // The general pool funds anything, so it is the block whose experience the rules
    // let buy Arts as well as Abilities. For a **magus** that is apprenticeship —
    // "These experience points can be spent on Arts or Abilities" (`:2435`) — with
    // later life a restricted, Abilities-only pool above. For a grog or companion
    // there is no apprenticeship and later life is itself unrestricted (`:2392`), so
    // it is the general pool. A directly-entered character uses the typed `xp_pool`.
    // Decided here, once: Skilled/Weak Parens (and any GeneralXp effect) then adjust
    // it — "an additional 60 experience points … during apprenticeship" (`:4966`) —
    // and a net-negative grant clamps at 0 rather than underflowing.
    //
    // A magus's years past its Gauntlet join that same general pool rather than
    // forming a block of their own: "Divide 30 points per year between experience
    // points in Arts, experience points in Abilities, and levels of spells"
    // (`:2216`), "Each point can be an experience point in an Art or Ability or one
    // level of spell" (`:2471`) — Arts included, which is precisely what makes a pool
    // general. The Academic/Arcane/Martial gate does not narrow them either: `:2435`
    // restricts only what a magus may buy "**before** apprenticeship", and "Magi
    // without a specific Virtue may only buy Academic Abilities during or after
    // apprenticeship" (`:7151`) says the years after it are on the permitted side.
    // So there is no restricted pool and no life-stage block to add — the block that
    // funds anything is the general pool and needs no slug.
    // Source: Ars Magica - Definitive Edition (Core Rules).md:2216, :2435, :2471, :7151.
    //
    // Recomputed here rather than threaded out of `build_flow_pools`: both are
    // cheap, pure, side-effect-free lookups (a profile map lookup; a life-stage
    // budget derivation), so recomputing costs nothing and keeps that function's
    // return type a plain `Vec<FlowPool>` independent of this one's locals.
    let is_magus = ruleset
        .profile(&entity.type_id)
        .is_some_and(|profile| profile.is_magus);
    let life_stage_budget = ruleset
        .life_stages()
        .and_then(|rules| rules.budget(entity, ruleset).map(|budget| (rules, budget)));
    let base_general = match &life_stage_budget {
        Some((_, budget)) if is_magus => budget
            .apprenticeship_xp
            .saturating_add(budget.post_gauntlet_xp),
        Some((_, budget)) => budget.later_life_xp,
        None => entity.xp_pool,
    };
    let general_bonus = general_xp_bonus(entity, ruleset);
    let general_pool = clamp_to_u32(i64::from(base_general) + general_bonus);

    // Flow graph: source(0) → sink(1); general(2) and restricted pools
    // (3..3+R) are pool nodes; spends follow. cap is the residual matrix.
    let r = flow_pools.len();
    let s = spends.len();
    let n = 3 + r + s;
    // `r` and `s` both grow 1:1 with save-controlled `Vec`s (`entity.selections`
    // for `r`; `entity.ability_scores`/`art_scores`/`spells` for `s` — see
    // `types.rs:2482` (selections), `:2492` (ability_scores), `:2521`
    // (art_scores), `:2526` (spells)), and the matrix below is `n * n` `u32`s.
    // With no bound, a crafted save with tens of thousands of entries forces a
    // multi-gigabyte single allocation on a plain File → Open, aborting the whole
    // process (`handle_alloc_error`) with no dialog and no diagnostic — an
    // uncontrolled-resource-consumption DoS (CWE-400/789). `MAX_XP_SOLVE_NODES`
    // bounds a character's own plausible selections, never the ruleset's
    // catalogue size (which the engine must never assume — see CLAUDE.md's
    // "Catalogue size is data, never code"): even an implausibly long-lived
    // archmage buys at most a few hundred distinct Ability/Art scores and masters
    // at most a few hundred spells, so this gives roughly an order of magnitude of
    // headroom above that while keeping the matrix under ~64 MB.
    //
    // The bound protects memory, not CPU time: a legal entity whose *spends*
    // (not `flow_pools`) make up most of `n` near the 2048 ceiling measurably
    // takes on the order of a minute in a debug build (`max_flow`'s BFS is
    // `O(n)` per dequeued node regardless of real edge count, and this graph's
    // shape needs roughly one augmenting BFS per spend, i.e. `O(n)` BFS calls —
    // `O(n^3)` overall — found empirically while writing this fix's own
    // regression test, not previously measured). No test in this suite
    // exercises that shape at the full bound for exactly this reason (see the
    // `tests` module below); flagged in the round-2 K1 fix report as a
    // follow-up rather than fixed here, since a spend-heavy save at this scale
    // is a legal-but-slow character, not a memory-safety regression.
    //
    // This function is `pub(crate)`, not `pub` (audit finding K1, round 2): an
    // earlier version of this comment claimed the check here was "unbypassable"
    // and cited `validate_xp_pool` as this function's "only call site anywhere"
    // — both false. `xp_allocation` had three more callers with no bound check
    // at all (`ruleset_io::xp_fields`, `export/sections.rs`, and
    // `restricted_xp_pools` below), each reachable from a plain File → Open or
    // File → Export Markdown of a hostile save, and each hit this very
    // `assert!` instead of the friendly rejection the check was supposed to
    // guarantee. [`checked_xp_allocation`] is now the only route to this
    // function reachable from outside this crate — the compiler enforces that,
    // not a comment — and every in-crate caller (`validate_xp_pool`,
    // `restricted_xp_pools`) goes through it too. The `assert!` below remains
    // only as an internal invariant for any future in-crate caller that
    // bypasses the checked wrapper; it can no longer be reached by an
    // untrusted `Entity` loaded from disk.
    assert!(
        n <= MAX_XP_SOLVE_NODES,
        "xp_allocation: entity selections exceed the safety bound of \
         {MAX_XP_SOLVE_NODES} flow-solve nodes ({n} needed: {s} spends + {r} flow \
         pools) — refusing to build the {n}x{n} matrix; this indicates a malformed \
         or hostile save, not a legal character"
    );
    let general_node = 2;
    let pool_node = |i: usize| 3 + i;
    let spend_node = |j: usize| 3 + r + j;
    let (source, sink) = (0usize, 1usize);

    let mut cap = vec![vec![0u32; n]; n];
    for (i, pool) in flow_pools.iter().enumerate() {
        cap[source][pool_node(i)] = pool.amount;
    }
    for (j, spend) in spends.iter().enumerate() {
        cap[spend_node(j)][sink] = spend.cost;
        // The general pool can fund any spend.
        cap[general_node][spend_node(j)] = spend.cost;
        for (i, pool) in flow_pools.iter().enumerate() {
            if pool_covers(&pool.eligibility, spend) {
                cap[pool_node(i)][spend_node(j)] = spend.cost;
            }
        }
    }

    // Two-phase fill on the shared residual matrix, so a spend the restricted
    // pools *can* cover drains them before the general pool (Educated/Warrior/
    // Privileged and Mastered-Spells XP is free-but-earmarked; the general pool
    // must stay available and no restricted XP wasted while eligible spends exist).
    // Phase 1: restricted-only max flow — the source→general edge stays closed.
    let restricted_flow = max_flow(n, source, sink, &mut cap);
    // Phase 2: open the source→general edge and continue Edmonds-Karp on the
    // same residuals. The sum is the true max flow with restricted usage
    // maximized, i.e. minimum general used.
    cap[source][general_node] = general_pool;
    let max_flow = restricted_flow + max_flow(n, source, sink, &mut cap);

    // Residual on source→pool tells how much each pool funded.
    let general_used = general_pool - cap[source][general_node];
    // Surface only the ability-XP pools (the mastery pool is accounted separately).
    let mut restricted: Vec<RestrictedXpPool> = Vec::new();
    for (i, pool) in flow_pools.iter().enumerate() {
        if let PoolEligibility::Ability {
            abilities,
            categories,
            ..
        } = &pool.eligibility
        {
            restricted.push(RestrictedXpPool {
                amount: pool.amount,
                used: pool.amount - cap[source][pool_node(i)],
                origin: pool.origin.clone(),
                abilities: abilities.clone(),
                categories: categories.clone(),
            });
        }
    }

    XpAllocation {
        total_demand,
        max_flow,
        general_pool,
        general_bonus,
        general_used,
        restricted,
    }
}

/// Edmonds-Karp max flow on a residual capacity matrix (BFS augmenting paths).
/// The graph is tiny (a few pools + a few dozen spends), so the simple matrix
/// form is more than fast enough.
fn max_flow(n: usize, source: usize, sink: usize, cap: &mut [Vec<u32>]) -> u32 {
    let mut total = 0;
    loop {
        let mut parent = vec![usize::MAX; n];
        parent[source] = source;
        let mut queue = VecDeque::new();
        queue.push_back(source);
        while let Some(u) = queue.pop_front() {
            for v in 0..n {
                if parent[v] == usize::MAX && cap[u][v] > 0 {
                    parent[v] = u;
                    queue.push_back(v);
                }
            }
        }
        if parent[sink] == usize::MAX {
            return total;
        }
        // Bottleneck along the found path.
        let mut bottleneck = u32::MAX;
        let mut v = sink;
        while v != source {
            let u = parent[v];
            bottleneck = bottleneck.min(cap[u][v]);
            v = u;
        }
        // Augment.
        let mut v = sink;
        while v != source {
            let u = parent[v];
            cap[u][v] -= bottleneck;
            cap[v][u] += bottleneck;
            v = u;
        }
        total += bottleneck;
    }
}

/// The restricted XP pools an entity holds, with their consumed amounts (for the
/// frontend XP bar). Convenience wrapper over [`checked_xp_allocation`]; an
/// entity over the solve bound degrades to an empty list rather than
/// panicking — the caller that actually needs to tell the user why is
/// `validate_xp_pool`, which runs the same check and reports
/// `CODE_XP_SOLVE_BOUND_EXCEEDED`.
pub fn restricted_xp_pools(entity: &Entity, ruleset: &Ruleset) -> Vec<RestrictedXpPool> {
    checked_xp_allocation(entity, ruleset)
        .map(|allocation| allocation.restricted)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ruleset::RulesetSources;
    use crate::types::{AbilityScore, EntityKind, RulesetRef};

    const ITEMS: &str = "[]";
    const TYPES: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general"], "creation_phases": [] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [{ "score": 1, "total_xp": 5 }],
      "abilities": [{ "id": "ability.artes_liberales", "category": "general" }]
    }"#;

    fn rs() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn companion_with_scores(count: usize) -> Entity {
        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        e.xp_pool = 1_000_000;
        e.ability_scores = (0..count)
            .map(|_| AbilityScore {
                ability: Id::new("ability.artes_liberales"),
                parameter: None,
                score: 1,
                specialty: None,
            })
            .collect();
        e
    }

    /// A ruleset carrying `pool_count` distinct restricted-XP-granting virtues,
    /// all eligible for the one real ability the fixture's entities buy — used
    /// to put node mass on `flow_pools` rather than `spends`. This matters for
    /// runtime, not just node count: `max_flow`'s BFS cost is `O(n)` per
    /// dequeued node regardless of real edge count, and the number of BFS
    /// calls the solve needs tracks `spends` (the sink-side bottleneck), not
    /// `flow_pools` — so a fixture with thousands of pools and a handful of
    /// spends reaches the same `n` as a spends-heavy one in a fraction of the
    /// runtime (see the comment on the `assert!` above, which measured a
    /// spends-heavy construction at this scale taking on the order of a
    /// minute in a debug build).
    fn rs_with_dead_pools(pool_count: usize) -> Ruleset {
        // The engine requires at least one V/F category-tagged "personality"
        // (`ENGINE_REQUIRED_CATEGORY_PERSONALITY`) once a catalogue is shipped
        // at all; unrelated to what this fixture is testing, but needed for
        // `Ruleset::from_sources` to pass integrity.
        let mut items = String::from(
            r#"[{"id":"flaw.filler","kind":"flaw","classification":"narrative","magnitude":"minor","category":"personality","entity_kinds":["character"]}"#,
        );
        for i in 0..pool_count {
            items.push(',');
            items.push_str(&format!(
                r#"{{"id":"virtue.dead_pool_{i}","kind":"virtue","classification":"narrative","magnitude":"minor","category":"general","effects":[{{"type":"restricted_ability_xp","amount":10,"abilities":["ability.artes_liberales"]}}]}}"#
            ));
        }
        items.push(']');
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: &items,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// A companion holding one selection per dead pool plus `score_count` real
    /// bought scores — paired with [`rs_with_dead_pools`].
    fn companion_with_pools_and_scores(pool_count: usize, score_count: usize) -> Entity {
        let mut e = companion_with_scores(score_count);
        e.selections = (0..pool_count)
            .map(|i| Selection::new(Id::new(format!("virtue.dead_pool_{i}"))))
            .collect();
        e
    }

    /// K1 (round-2 CRITICAL): `checked_xp_allocation` is the only entry point to
    /// the flow solve safe to call on an untrusted `Entity`. Exactly at the
    /// bound (`nodes == MAX_XP_SOLVE_NODES`) it must still compute, and return
    /// the same figures the raw solver would.
    #[test]
    fn checked_xp_allocation_computes_at_the_bound() {
        let pools = MAX_XP_SOLVE_NODES - 3 - 5;
        let rs = rs_with_dead_pools(pools);
        let at_bound = companion_with_pools_and_scores(pools, 5);
        assert_eq!(xp_solve_scale(&at_bound, &rs).nodes(), MAX_XP_SOLVE_NODES);

        let allocation = checked_xp_allocation(&at_bound, &rs).unwrap();
        assert_eq!(allocation, xp_allocation(&at_bound, &rs));
    }

    /// One node past the bound must return `Err` — naming the same counts
    /// `CODE_XP_SOLVE_BOUND_EXCEEDED` does — rather than ever building the
    /// oversized matrix or panicking. This path never reaches the solver at
    /// all (the check runs first), so it stays fast regardless of how the
    /// node mass is constructed; a plain spends-heavy fixture is simplest.
    #[test]
    fn checked_xp_allocation_refuses_one_past_the_bound() {
        let rs = rs();
        let past_bound = companion_with_scores(MAX_XP_SOLVE_NODES - 3 + 1);
        let err = checked_xp_allocation(&past_bound, &rs).unwrap_err();
        assert_eq!(err.nodes, MAX_XP_SOLVE_NODES + 1);
        assert_eq!(err.limit, MAX_XP_SOLVE_NODES);
        assert_eq!(err.spends, MAX_XP_SOLVE_NODES - 2);
        assert_eq!(err.flow_pools, 0);
    }

    /// `restricted_xp_pools` is one of K1's four unguarded call sites — it must
    /// degrade to an empty list over the bound, never panic.
    #[test]
    fn restricted_xp_pools_degrades_to_empty_over_the_solve_bound() {
        let rs = rs();
        let past_bound = companion_with_scores(MAX_XP_SOLVE_NODES - 3 + 1);
        assert_eq!(restricted_xp_pools(&past_bound, &rs), Vec::new());
    }
}
