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
//! Its *value* additionally caps the `O(n^3)` solve's worst accepted runtime
//! (audit finding Klaus F4) — see the constant's own docs.

use super::*;

/// The experience charged against a pool for a bought score whose advancement
/// table cost is `table_xp`, under an optional Affinity multiplier.
///
/// Affinity (Ability/Art) says creation XP "counts as" `num/den` of itself
/// (3/2, rounded up): so the points actually charged to reach a fixed table cost
/// `T` are the smallest `c` with `ceil(c·num/den) ≥ T`. For `T > 0` that is
/// `floor(den·(T−1)/num) + 1`, **not** the simpler-looking `ceil(T·den/num)`.
/// The two formulas disagree exactly when `T·den mod num` falls strictly
/// between `0` and `den`, and overcharge by one XP in that case — row 47 /
/// V/F-audit F-547. They agree on the rulebook's own worked example (Perdo 10,
/// Art table T=55, 3/2: both give `floor(2·54/3)+1 = ceil(55·2/3) = 37`), which
/// is why the bug survived undetected there; they disagree on the Specialist
/// grog template's Single Weapon 7 (Ability table T=140, 3/2: the correct
/// charge is `floor(2·139/3)+1 = 93`, while `ceil(140·2/3) = 94` overcharged
/// the player by one — `docs/book-template-conformance.md` § S2).
///
/// A degenerate ratio charges the **full** cost. Both halves are rejected at load
/// (`ruleset/integrity.rs::validate_item_ratios`), so this arm is
/// defence in depth for a ruleset that somehow reached the engine unvalidated:
/// `den == 0` would otherwise make every score under the Affinity free, which is
/// the one direction that must never fail open.
///
/// Source: ArMDE:3372-3378, worked
/// example `ArMDE:2443`.
pub(crate) fn charged_cost(table_xp: u32, affinity: Option<(u8, u8)>) -> u32 {
    match affinity {
        Some((num, den)) if num != 0 && den != 0 && table_xp != 0 => {
            (table_xp - 1).saturating_mul(u32::from(den)) / u32::from(num) + 1
        }
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
/// an Ability carries an Affinity (which raises its age cap by +2, ArMDE:3374).
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
                } if selection
                    .params
                    .get(param)
                    .and_then(SelectionParamValue::as_single)
                    == Some(ability) =>
                {
                    let matches = match instance_key {
                        None => true,
                        Some(key) => {
                            selection
                                .params
                                .get(key)
                                .and_then(SelectionParamValue::as_single)
                                .map(Id::as_str)
                                == parameter
                        }
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
                } if selection
                    .params
                    .get(param)
                    .and_then(SelectionParamValue::as_single)
                    == Some(art) =>
                {
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
/// Abilities" (ArMDE:2435). Later life buys "any **Abilities**" (`ArMDE:2214`,
/// `ArMDE:2392`) and, for a magus, ends where apprenticeship begins — so it is a
/// restricted pool of its own, listed here. For a grog or companion later life is
/// still the general pool; the enum names the blocks that *can* be restricted, and
/// which pools a character actually gets is decided in [`xp_allocation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifeStageBlock {
    /// Childhood's native-language experience: spendable only on the native
    /// language instance (ArMDE:2378).
    ChildhoodNativeLanguage,
    /// Childhood's restricted spread: spendable only on the childhood Ability list,
    /// and never on the native language (`ArMDE:2378`).
    ChildhoodSpread,
    /// Later life: for a magus, the years before apprenticeship, spendable on
    /// Abilities alone and never on an Art (`ArMDE:2214`, `ArMDE:2392`).
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
///
/// The `life_stages` check here genuinely asks "is there a plan to read a language
/// off", **not** "is this character life-stage funded". The funding mode is decided
/// once, upstream: this is only called inside the `if let Some((rules, budget))`
/// branch below, and `LifeStageRules::budget` already returns `None` under
/// [`crate::AbilityFunding::Pool`].
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

/// One Ability instance a selection's effects authorize the character to buy —
/// the resolved form [`ability_authorizations`]'s fold produces from an
/// [`AbilityRef`]. `instance: None` authorizes every instance of `ability`
/// (Second Sight, or a category-wide [`Effect::AbilityAuthorization`] entry);
/// `Some(x)` authorizes only the instance whose bought
/// [`crate::types::AbilityScore::parameter`] equals `x` (Covenant Upbringing's
/// Latin proxy — this is F-349/F-16x's actual fix: without the instance,
/// authorizing `ability.dead_language` at all would also authorize Ancient
/// Greek).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AuthorizedAbility {
    pub(crate) ability: Id,
    pub(crate) instance: Option<String>,
}

impl AuthorizedAbility {
    /// Whether this authorization covers the bought `(ability, parameter)`
    /// instance: the ability id matches, and either this entry authorizes any
    /// instance (`instance: None`) or the bought instance is the one named.
    fn covers(&self, ability: &Id, parameter: Option<&str>) -> bool {
        self.ability == *ability
            && (self.instance.is_none() || self.instance.as_deref() == parameter)
    }
}

/// Whether `authorized` contains an entry covering the bought `(ability,
/// parameter)` instance — the instance-aware membership test
/// [`crate::validation::authorization::validate_ability_authorization`] uses in
/// place of a bare `.contains(&id)`.
pub(crate) fn authorizes_instance(
    authorized: &BTreeSet<AuthorizedAbility>,
    ability: &Id,
    parameter: Option<&str>,
) -> bool {
    authorized.iter().any(|a| a.covers(ability, parameter))
}

/// The Abilities and categories the character's selections permit.
///
/// A Virtue grants access three ways, and all three count: an explicit
/// [`Effect::AbilityAuthorization`], any [`Effect::RestrictedAbilityXp`] pool —
/// experience earmarked for a category is evidence the category is permitted, which
/// is what makes Warrior (Martial XP) and Arcane Lore (Arcane XP) work without
/// further data — or an [`Effect::AbilityBonusGated`] target: a competence bonus
/// tied to a specific Ability instance is itself permission to own it, since the
/// bonus could never apply to an Ability the character may not buy (Student of
/// (Realm)'s "even if you cannot learn other Arcane Abilities", ArMDE:5054).
///
/// [`Effect::AbilityAuthorization`] and [`Effect::AbilityBonusGated`] entries may
/// be gated on the OWNING selection's own parameter (D14/W2): an entry whose
/// [`ParamGate`] does not hold contributes nothing, by construction — there is no
/// "include but mark restricted" state to forget (see
/// `docs/vf-audit/design-c0-parameter-model.md` § 3/§ 4).
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
) -> (BTreeSet<AuthorizedAbility>, BTreeSet<AbilityCategory>) {
    let mut abilities = BTreeSet::new();
    let mut categories = BTreeSet::new();
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            match effect {
                // Experience earmarked for a category or Ability is itself
                // permission to learn it — otherwise the grant could never be
                // spent. A fixed id/category list (never gated, never
                // instance-scoped): every named entry always counts.
                Effect::RestrictedAbilityXp {
                    abilities: ids,
                    categories: cats,
                    ..
                } => {
                    abilities.extend(ids.iter().map(|ability| AuthorizedAbility {
                        ability: ability.clone(),
                        instance: None,
                    }));
                    categories.extend(cats.iter().copied());
                }
                // D35's parameter-scaled sibling: an earmark is itself
                // permission, exactly like `RestrictedAbilityXp` above — but
                // `abilities` here is `Vec<AbilityRef>`, so each entry is
                // resolved (and, in principle, gated) against THIS selection
                // the same way `AbilityAuthorization`'s own list is below.
                Effect::ScaledRestrictedAbilityXp {
                    abilities: refs,
                    categories: cats,
                    ..
                } => {
                    for a in refs {
                        if a.active_for(selection) {
                            abilities.insert(AuthorizedAbility {
                                ability: a.ability().clone(),
                                instance: a.resolved_instance(selection),
                            });
                        }
                    }
                    categories.extend(cats.iter().copied());
                }
                // The gated carrier (D14/W2): only the entries whose gate holds
                // for THIS selection contribute — F-349's fix (see
                // `docs/vf-audit/design-c0-parameter-model.md` § 3/§ 4).
                Effect::AbilityAuthorization {
                    abilities: refs,
                    categories: cat_refs,
                } => {
                    for a in refs {
                        if a.active_for(selection) {
                            abilities.insert(AuthorizedAbility {
                                ability: a.ability().clone(),
                                instance: a.resolved_instance(selection),
                            });
                        }
                    }
                    for c in cat_refs {
                        if c.active_for(selection) {
                            categories.insert(c.category());
                        }
                    }
                }
                // A free score in an Ability is permission to have it, since the
                // Virtue confers the Ability outright.
                Effect::AbilityScoreGrant { ability, .. } => {
                    abilities.insert(AuthorizedAbility {
                        ability: ability.clone(),
                        instance: None,
                    });
                }
                // The gated-bonus carrier (Student of (Realm)'s +2 Lore, row
                // 50(a)): same gate fold as `AbilityAuthorization`, read off
                // this effect's own target list instead.
                Effect::AbilityBonusGated { targets, .. } => {
                    for t in targets {
                        if t.active_for(selection) {
                            abilities.insert(AuthorizedAbility {
                                ability: t.ability().clone(),
                                instance: t.resolved_instance(selection),
                            });
                        }
                    }
                }
                // Exhaustive so adding an Effect variant is a compile error here,
                // not a silently-ignored authorization gap (V55). Every listed
                // variant is a deliberate no-op for *ownership permission*:
                // Ability/Art/Characteristic bonuses and Affinities (including
                // the group form) target or discount something already legally
                // owned rather than granting the right to own it (Puissant
                // Ability: ArMDE:4814-4816);
                // the XP/budget/derived-stat grants (spell levels, general XP,
                // later-life rate, confidence, mastery, item levels, True
                // Faith/Warping/Might/Power/Reputation, size, characteristic
                // points) fund or size something, never authorize an Ability;
                // `GrantsSelection` is already expanded by
                // `selections_for_effects` before this loop runs, so its target
                // item's own effects are picked up on their own iteration, not
                // here; and the M5/5b in-play effects (casting/lab/combat/health/
                // soak/magic-resistance/aging/advancement/casting-style/roll
                // modifiers, Elemental Magic) modify a derived total computed
                // over Abilities/Arts already owned, never ownership itself.
                Effect::AbilityBonus { .. }
                | Effect::CharacteristicScoreDeltaParam { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::GroupAffinityCost { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::LaterLifeXpRate { .. }
                | Effect::LocalityAbilityCapFraction { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::SpellMasteryXp { .. }
                | Effect::GrantsSpellMastery { .. }
                | Effect::GrantsSelection { .. }
                | Effect::ItemLevelBudget { .. }
                | Effect::MasterpieceItem
                | Effect::TrueFaithGrant { .. }
                | Effect::WarpingGrant { .. }
                | Effect::SizeDelta { .. }
                | Effect::CharacteristicScoreDelta { .. }
                | Effect::GrantsReputation { .. }
                | Effect::MightGrant { .. }
                | Effect::PowerLevels { .. }
                | Effect::FocusPoints { .. }
                | Effect::MagicalFocus { .. }
                | Effect::CastingTotalMod { .. }
                | Effect::LabTotalMod { .. }
                | Effect::HalvesSpellCapBeyondTouch
                | Effect::DeficientArt { .. }
                | Effect::MagicTotalHalving { .. }
                | Effect::SoakMod { .. }
                | Effect::CombatMod { .. }
                | Effect::HealthMod { .. }
                | Effect::MagicResistanceMod { .. }
                | Effect::AgingMod { .. }
                | Effect::AdvancementMod { .. }
                | Effect::SpecialCastingMod { .. }
                | Effect::AbilityRollMod { .. }
                | Effect::ElementalMagic { .. }
                | Effect::ForbidsAbilitySpecialties
                | Effect::ForbidsRitualCasting
                // A cap waiver raises a ceiling, not a permission to own an
                // Ability in the first place.
                | Effect::WaivesAbilityAgeCap
                // Grants Hermetic training as a fact, not permission to own a
                // specific Ability or category — Arcane authorization for a
                // trained non-magus is a profile/entity-level gate in
                // `validation/authorization.rs`, not a per-effect grant here.
                | Effect::ConfersHermeticTraining => {}
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
/// The value bounds **CPU** as well as memory (audit finding Klaus F4), which is
/// what sets it this low: the solve is `O(n^3)`, so the cost of the worst save
/// that is still *accepted* scales with the cube of this number. Measured on a
/// spends-heavy entity (the expensive shape — see `rs_with_dead_pools`), release
/// build: 60 ms at 512 spends, 196 ms at 768, 454 ms at 1024, and ~3.6 s
/// extrapolated at the old 2048, against ~21 s / ~170 s for the same two points
/// in a debug build. Since `validate_xp_pool`, `effective_scores` and
/// `export/sections.rs` each re-run the solve, a debounced `refresh()` pays it
/// about three times over — so 2048 meant a multi-second freeze per keystroke on
/// a save that was accepted rather than rejected, and 1024 keeps that worst case
/// comfortably sub-second.
///
/// The lower edge is the binding constraint, and it is a *character* size, not a
/// catalogue size: roughly 211 Ability scores (71 distinct plus twenty instances
/// of each parameterized one), 15 Art scores, 560 mastered spells and 64
/// restricted pools come to 853 nodes for a maximal-but-legal character, so this
/// is the smallest power of two that cannot reject one.
/// `the_solve_bound_admits_a_maximal_legal_character` pins that arithmetic and
/// fails if the value is ever lowered into it.
///
/// `pub(crate)` (not otherwise used outside this module) so
/// [`crate::validation::magus::validate_xp_pool`] can compare against the same
/// constant `xp_allocation` enforces, rather than restating the number: the
/// validation layer rejects a hostile save with a structured issue *before*
/// ever calling `xp_allocation`, and this `assert!` remains the unbypassable
/// backstop for any caller that skips validation.
pub(crate) const MAX_XP_SOLVE_NODES: usize = 1024;

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
        // Source: ArMDE:2639.
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
    // Spell Mastery is an Ability (ArMDE:9516, :7143) bought from the
    // Ability advancement table (ArMDE:15952, :15956-15979). Flawless Magic auto-masters
    // every spell at a free floor (charge only above it, like a granted Supernatural
    // floor) AND doubles all mastery Advancement Totals (an Affinity that halves the
    // charge). The mastery pool (Mastered Spells) — not the ability-restricted pools
    // — plus the general pool fund it.
    // Source: ArMDE:3887-3889, :4471-4474.
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

/// One [`FlowPool`] per [`Effect::RestrictedAbilityXp`] or
/// [`Effect::ScaledRestrictedAbilityXp`] grant among the entity's selections
/// (Educated, Warrior, Privileged Upbringing, Simple Student, …) — pool kind 1
/// of 5 [`build_flow_pools`] assembles.
fn restricted_ability_xp_pools(entity: &Entity, ruleset: &Ruleset) -> Vec<FlowPool> {
    let mut flow_pools = Vec::new();
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            match effect {
                Effect::RestrictedAbilityXp {
                    amount,
                    abilities,
                    categories,
                } => {
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
                // D35: the grant's `amount` is `per_unit` times the value the
                // selection's own `param` (a `Number`-domain parameter) names
                // — no pool at all until a legal value is filled in, matching
                // `missing_param`'s "a choice not yet made" reading elsewhere.
                // `abilities` is `Vec<AbilityRef>` (Simple Student's Latin
                // instance restriction, § 1 of the design note), so every
                // active entry is resolved into an `AbilityInstanceRef` and
                // put in `instances` — a bare (any-instance) entry there is
                // `parameter: None`, which `AbilityInstanceRef::matches`
                // already treats as "any instance", so an unscoped ability
                // and an instance-scoped one coexist in the same list with no
                // special-casing (the same reasoning covenant_upbringing's
                // literal-instance form already established for
                // `AbilityAuthorization`). `categories` stays a plain
                // eligibility list; a future entry combining `categories` with
                // an instance-scoped ability in ONE grant would need D48's
                // union fix (C4) to see both — not needed by Simple Student,
                // this effect's only known consumer, and out of this slice's
                // scope.
                Effect::ScaledRestrictedAbilityXp {
                    param,
                    per_unit,
                    abilities,
                    categories,
                } => {
                    let Some(units) = selection
                        .params
                        .get(param)
                        .and_then(SelectionParamValue::as_single)
                        .and_then(|v| v.as_str().parse::<u32>().ok())
                    else {
                        continue;
                    };
                    flow_pools.push(FlowPool {
                        amount: per_unit.saturating_mul(units),
                        eligibility: PoolEligibility::Ability {
                            abilities: Vec::new(),
                            categories: categories.clone(),
                            instances: abilities
                                .iter()
                                .filter(|a| a.active_for(selection))
                                .map(|a| AbilityInstanceRef {
                                    ability: a.ability().clone(),
                                    parameter: a.resolved_instance(selection),
                                })
                                .collect(),
                            exclude: Vec::new(),
                        },
                        origin: XpPoolOrigin::Item {
                            item: selection.item_ref.clone(),
                        },
                    });
                }
                // Every other effect grants nothing to a restricted Ability-XP
                // pool.
                _ => {}
            }
        }
    }
    flow_pools
}

/// Childhood's native-language pool — pool kind 2 of 5 [`build_flow_pools`]
/// assembles. `None` when no native language is set (no pool is created for a
/// language nobody picked). Source: ArMDE:2378.
fn childhood_native_language_pool(
    entity: &Entity,
    rules: &crate::life_stage::LifeStageRules,
    budget: &crate::life_stage::LifeStageBudget,
) -> Option<FlowPool> {
    let native = native_language_instance(entity, rules)?;
    Some(FlowPool {
        amount: budget.childhood_native_xp,
        eligibility: PoolEligibility::Ability {
            abilities: Vec::new(),
            categories: Vec::new(),
            instances: vec![native],
            exclude: Vec::new(),
        },
        origin: XpPoolOrigin::LifeStage {
            block: LifeStageBlock::ChildhoodNativeLanguage,
        },
    })
}

/// Childhood's restricted-spread pool — pool kind 3 of 5 [`build_flow_pools`]
/// assembles. Always present alongside a life-stage budget (unlike the native
/// pool, which needs a language actually set). Source: ArMDE:2378.
fn childhood_spread_pool(
    entity: &Entity,
    rules: &crate::life_stage::LifeStageRules,
    budget: &crate::life_stage::LifeStageBudget,
) -> FlowPool {
    let native = native_language_instance(entity, rules);
    FlowPool {
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
    }
}

/// A magus's later-life pool — pool kind 4 of 5 [`build_flow_pools`] assembles:
/// the years between childhood and being taken as an apprentice, which buy "any
/// Abilities" (`ArMDE:2214`) and never an Art, and not an Arcane, Academic or Martial
/// Ability either — "magi can only spend experience points on Arcane, Academic
/// and Martial Abilities before apprenticeship if they have a Virtue which
/// allows them to do so" (`ArMDE:2435`). A Virtue that does allow it (Covenant
/// Upbringing, Educated, Warrior) widens the pool through the same
/// authorizations the ownership check reads, so the two cannot disagree.
///
/// Supernatural stays in the set and legalizes nothing: access to each
/// Supernatural Ability is granted per Ability, which
/// `validate_supernatural_abilities` enforces for magi too — so an
/// unauthorized one is already an error and funding it here changes nothing.
///
/// `None` when `budget.later_life_xp == 0` — a grog or companion never calls
/// this at all (later life is their general pool, `ArMDE:2392`, and the categories
/// are gated by an error on the character instead; a magus's category gate is
/// waived whole-character, `ArMDE:7151`, so this pool is the only place the "before
/// apprenticeship" half can live).
fn magus_later_life_pool(
    entity: &Entity,
    ruleset: &Ruleset,
    budget: &crate::life_stage::LifeStageBudget,
) -> Option<FlowPool> {
    if budget.later_life_xp == 0 {
        return None;
    }
    let (abilities, authorized_categories) = ability_authorizations(entity, ruleset);
    let gated = ruleset.categories_requiring_virtue();
    Some(FlowPool {
        amount: budget.later_life_xp,
        eligibility: PoolEligibility::Ability {
            // Funding stays id-scoped (any instance) here — narrowing a POOL to
            // one instance is D48/C4's `instances` field, not this slice's job;
            // C1 only tightens OWNERSHIP (`validate_ability_authorization`),
            // which is the actual F-349 defect.
            abilities: abilities.iter().map(|a| a.ability.clone()).collect(),
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
    })
}

/// The Spell-Mastery pool (Mastered Spells, summed) — pool kind 5 of 5
/// [`build_flow_pools`] assembles. Flow-only: never surfaced in `restricted`,
/// which the UI reserves for ability-XP grants, so its origin is nominal.
/// `None` when the entity holds no Spell-Mastery grant.
fn spell_mastery_flow_pool(entity: &Entity, ruleset: &Ruleset) -> Option<FlowPool> {
    let amount = spell_mastery_xp(entity, ruleset);
    (amount > 0).then_some(FlowPool {
        amount,
        eligibility: PoolEligibility::Mastery,
        origin: XpPoolOrigin::LifeStage {
            block: LifeStageBlock::ChildhoodSpread,
        },
    })
}

/// Builds every restricted [`FlowPool`] the entity's selections and life stages
/// grant — the non-general supply side of [`xp_allocation`]'s flow solve.
/// Extracted from `xp_allocation` (pure code motion, no behavior change): the
/// general pool is computed separately (it needs [`general_xp_bonus`] and the
/// life-stage/magus flags again, recomputed there — both are cheap, pure lookups
/// with no side effects, so recomputing costs nothing and keeps this function
/// independent of the graph-solve locals).
///
/// V57: assembles the five independent pool kinds above, in the fixed order
/// the UI's XP bar and the flow-solve node indices both depend on
/// (`restricted_xp_pools`'s output order is observable). Which block is the
/// general pool depends on whether the character is Hermetically trained
/// (`is_hermetically_trained`, D56/A0 — the profile flag unioned with any
/// selection carrying `Effect::ConfersHermeticTraining`), so the fact is read
/// once here — never a type id.
fn build_flow_pools(entity: &Entity, ruleset: &Ruleset) -> Vec<FlowPool> {
    let mut flow_pools = restricted_ability_xp_pools(entity, ruleset);

    let life_stage_budget = ruleset
        .life_stages()
        .and_then(|rules| rules.budget(entity, ruleset).map(|budget| (rules, budget)));
    if let Some((rules, budget)) = &life_stage_budget {
        flow_pools.extend(childhood_native_language_pool(entity, rules, budget));
        flow_pools.push(childhood_spread_pool(entity, rules, budget));
        let trained = crate::effective::is_hermetically_trained(
            entity,
            ruleset,
            ruleset.profile(&entity.type_id),
        );
        if trained {
            flow_pools.extend(magus_later_life_pool(entity, ruleset, budget));
        }
    }

    flow_pools.extend(spell_mastery_flow_pool(entity, ruleset));
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

    let (general_pool, general_bonus) = general_pool_and_bonus(entity, ruleset);

    let (layout, mut cap) = build_capacity_matrix(&flow_pools, &spends);
    let (max_flow, general_used) = two_phase_max_flow(&layout, general_pool, &mut cap);
    let restricted = assemble_restricted_pools(&layout, &flow_pools, &cap);

    XpAllocation {
        total_demand,
        max_flow,
        general_pool,
        general_bonus,
        general_used,
        restricted,
    }
}

/// The general pool size (spendable on any Ability, Art, or Mastery) and the
/// signed [`Effect::GeneralXp`] contribution folded into it — sub-check 1 of
/// [`xp_allocation`], extracted (pure code motion, no behavior change) so the
/// rules citation this needs stays with the arithmetic it explains rather than
/// interleaved with the flow-graph sub-checks below.
///
/// The general pool funds anything, so it is the block whose experience the rules
/// let buy Arts as well as Abilities. For a **magus** that is apprenticeship —
/// "These experience points can be spent on Arts or Abilities" (`ArMDE:2435`) — with
/// later life a restricted, Abilities-only pool ([`magus_later_life_pool`]). For a
/// grog or companion there is no apprenticeship and later life is itself
/// unrestricted (`ArMDE:2392`), so it is the general pool. A directly-entered character
/// uses the typed `xp_pool`. Decided here, once: Skilled/Weak Parens (and any
/// GeneralXp effect) then adjust it — "an additional 60 experience points … during
/// apprenticeship" (`ArMDE:4966`) — and a net-negative grant clamps at 0 rather than
/// underflowing.
///
/// A magus's years past its Gauntlet join that same general pool rather than
/// forming a block of their own: "Divide 30 points per year between experience
/// points in Arts, experience points in Abilities, and levels of spells"
/// (`ArMDE:2216`), "Each point can be an experience point in an Art or Ability or one
/// level of spell" (`ArMDE:2471`) — Arts included, which is precisely what makes a pool
/// general. The Academic/Arcane/Martial gate does not narrow them either: `ArMDE:2435`
/// restricts only what a magus may buy "**before** apprenticeship", and "Magi
/// without a specific Virtue may only buy Academic Abilities during or after
/// apprenticeship" (`ArMDE:7151`) says the years after it are on the permitted side.
/// So there is no restricted pool and no life-stage block to add — the block that
/// funds anything is the general pool and needs no slug.
/// Source: ArMDE:2216, :2435, :2471, :7151.
///
/// Recomputed here rather than threaded out of `build_flow_pools`: both are
/// cheap, pure, side-effect-free lookups (a profile map lookup; a life-stage
/// budget derivation), so recomputing costs nothing and keeps that function's
/// return type a plain `Vec<FlowPool>` independent of this one's locals.
fn general_pool_and_bonus(entity: &Entity, ruleset: &Ruleset) -> (u32, i64) {
    let trained = crate::effective::is_hermetically_trained(
        entity,
        ruleset,
        ruleset.profile(&entity.type_id),
    );
    let life_stage_budget = ruleset
        .life_stages()
        .and_then(|rules| rules.budget(entity, ruleset).map(|budget| (rules, budget)));
    let base_general = match &life_stage_budget {
        Some((_, budget)) if trained => budget
            .apprenticeship_xp
            .saturating_add(budget.post_gauntlet_xp),
        Some((_, budget)) => budget.later_life_xp,
        None => entity.xp_pool,
    };
    let general_bonus = general_xp_bonus(entity, ruleset);
    let general_pool = clamp_to_u32(i64::from(base_general) + general_bonus);
    (general_pool, general_bonus)
}

/// The flow graph's node-index scheme: source(0) → sink(1); general(2) and
/// restricted pools (3..3+R) are pool nodes; spends follow. A tiny bundle so
/// [`build_capacity_matrix`], [`two_phase_max_flow`], and
/// [`assemble_restricted_pools`] agree on the same layout without re-deriving
/// `pool_node`/`spend_node` three times.
struct FlowGraphLayout {
    r: usize,
    s: usize,
}

impl FlowGraphLayout {
    const SOURCE: usize = 0;
    const SINK: usize = 1;
    const GENERAL: usize = 2;

    fn n(&self) -> usize {
        3 + self.r + self.s
    }

    fn pool_node(&self, i: usize) -> usize {
        3 + i
    }

    fn spend_node(&self, j: usize) -> usize {
        3 + self.r + j
    }
}

/// Builds the flow graph's initial residual-capacity matrix — sub-check 2 of
/// [`xp_allocation`]. `cap[u][v]` is the capacity of edge `u → v`: every pool
/// from the source, every spend to the sink, the general pool to every spend,
/// and each restricted pool to the spends it [`pool_covers`].
///
/// # Panics
/// Panics if the graph would need more than [`MAX_XP_SOLVE_NODES`] nodes.
/// `r` and `s` both grow 1:1 with save-controlled `Vec`s (`entity.selections`
/// for `r`; `entity.ability_scores`/`art_scores`/`spells` for `s` — see
/// `types.rs::Entity::selections`, `types.rs::Entity::ability_scores`,
/// `types.rs::Entity::art_scores` and `types.rs::Entity::spells`), and the
/// matrix below is `n * n` `u32`s.
/// With no bound, a crafted save with tens of thousands of entries forces a
/// multi-gigabyte single allocation on a plain File → Open, aborting the whole
/// process (`handle_alloc_error`) with no dialog and no diagnostic — an
/// uncontrolled-resource-consumption DoS (CWE-400/789). `MAX_XP_SOLVE_NODES`
/// bounds a character's own plausible selections, never the ruleset's
/// catalogue size (which the engine must never assume — see CLAUDE.md's
/// "Catalogue size is data, never code"): even an implausibly long-lived
/// archmage buys at most a few hundred distinct Ability/Art scores and masters
/// at most a few hundred spells, so this gives roughly an order of magnitude of
/// headroom above that while keeping the matrix under ~64 MB.
///
/// The bound protects CPU time as well as memory (audit finding Klaus F4).
/// `max_flow`'s BFS is `O(n)` per dequeued node regardless of real edge
/// count, and this graph's shape needs roughly one augmenting BFS per spend,
/// i.e. `O(n)` BFS calls — `O(n^3)` overall. So an entity whose *spends*
/// (not `flow_pools`) make up most of `n` is the expensive shape, and the
/// cost of the worst such save that is still accepted scales with the cube
/// of [`MAX_XP_SOLVE_NODES`]. Lowering that constant from 2048 to 1024 cut it
/// by 8x — from a multi-second freeze per debounced `refresh()` to
/// comfortably sub-second — which is why the constant is set from the
/// largest *character* that must not be rejected rather than from the
/// largest matrix that fits in memory. The `O(n^3)` algorithm itself is
/// unchanged and is tracked in `docs/open-todos.md`; no test in this suite
/// exercises the expensive shape at the full bound, since that would make
/// the suite pay the very cost the bound exists to cap (see the `tests`
/// module below).
///
/// This function is private to the module, reachable only through
/// [`xp_allocation`], which is itself `pub(crate)`, not `pub` (audit finding
/// K1, round 2): an earlier version of this comment claimed the check here
/// was "unbypassable" and cited `validate_xp_pool` as `xp_allocation`'s "only
/// call site anywhere" — both false. `xp_allocation` had three more callers
/// with no bound check at all (`ruleset_io::xp_fields`, `export/sections.rs`,
/// and `restricted_xp_pools` below), each reachable from a plain File → Open
/// or File → Export Markdown of a hostile save, and each hit this very
/// `assert!` instead of the friendly rejection the check was supposed to
/// guarantee. [`checked_xp_allocation`] is now the only route to this
/// function reachable from outside this crate — the compiler enforces that,
/// not a comment — and every in-crate caller (`validate_xp_pool`,
/// `restricted_xp_pools`) goes through it too. The `assert!` below remains
/// only as an internal invariant for any future in-crate caller that
/// bypasses the checked wrapper; it can no longer be reached by an
/// untrusted `Entity` loaded from disk.
fn build_capacity_matrix(
    flow_pools: &[FlowPool],
    spends: &[Spend],
) -> (FlowGraphLayout, Vec<Vec<u32>>) {
    let layout = FlowGraphLayout {
        r: flow_pools.len(),
        s: spends.len(),
    };
    let n = layout.n();
    assert!(
        n <= MAX_XP_SOLVE_NODES,
        "xp_allocation: entity selections exceed the safety bound of \
         {MAX_XP_SOLVE_NODES} flow-solve nodes ({n} needed: {} spends + {} flow \
         pools) — refusing to build the {n}x{n} matrix; this indicates a malformed \
         or hostile save, not a legal character",
        layout.s,
        layout.r,
    );

    let mut cap = vec![vec![0u32; n]; n];
    for (i, pool) in flow_pools.iter().enumerate() {
        cap[FlowGraphLayout::SOURCE][layout.pool_node(i)] = pool.amount;
    }
    for (j, spend) in spends.iter().enumerate() {
        cap[layout.spend_node(j)][FlowGraphLayout::SINK] = spend.cost;
        // The general pool can fund any spend.
        cap[FlowGraphLayout::GENERAL][layout.spend_node(j)] = spend.cost;
        for (i, pool) in flow_pools.iter().enumerate() {
            if pool_covers(&pool.eligibility, spend) {
                cap[layout.pool_node(i)][layout.spend_node(j)] = spend.cost;
            }
        }
    }
    (layout, cap)
}

/// Runs the two-phase max-flow solve on `cap` in place — sub-check 3 of
/// [`xp_allocation`]. So a spend the restricted pools *can* cover drains them
/// before the general pool (Educated/Warrior/Privileged and Mastered-Spells XP
/// is free-but-earmarked; the general pool must stay available and no
/// restricted XP wasted while eligible spends exist):
/// - Phase 1: restricted-only max flow — the source→general edge stays closed.
/// - Phase 2: open the source→general edge and continue Edmonds-Karp on the
///   same residuals. The sum is the true max flow with restricted usage
///   maximized, i.e. minimum general used.
///
/// Returns the total max flow and how much of `general_pool` the solve used
/// (read off the source→general residual `cap` is left holding).
fn two_phase_max_flow(
    layout: &FlowGraphLayout,
    general_pool: u32,
    cap: &mut [Vec<u32>],
) -> (u32, u32) {
    let n = layout.n();
    let restricted_flow = max_flow(n, FlowGraphLayout::SOURCE, FlowGraphLayout::SINK, cap);
    cap[FlowGraphLayout::SOURCE][FlowGraphLayout::GENERAL] = general_pool;
    let total = restricted_flow + max_flow(n, FlowGraphLayout::SOURCE, FlowGraphLayout::SINK, cap);
    let general_used = general_pool - cap[FlowGraphLayout::SOURCE][FlowGraphLayout::GENERAL];
    (total, general_used)
}

/// Reads the solved `cap` residuals back into the per-pool `used` figures the
/// frontend's XP bar shows — sub-check 4 of [`xp_allocation`]. Surfaces only
/// the ability-XP pools (the mastery pool is accounted for in `max_flow`
/// alone, never shown as its own bar).
fn assemble_restricted_pools(
    layout: &FlowGraphLayout,
    flow_pools: &[FlowPool],
    cap: &[Vec<u32>],
) -> Vec<RestrictedXpPool> {
    let mut restricted = Vec::new();
    for (i, pool) in flow_pools.iter().enumerate() {
        if let PoolEligibility::Ability {
            abilities,
            categories,
            ..
        } = &pool.eligibility
        {
            restricted.push(RestrictedXpPool {
                amount: pool.amount,
                used: pool.amount - cap[FlowGraphLayout::SOURCE][layout.pool_node(i)],
                origin: pool.origin.clone(),
                abilities: abilities.clone(),
                categories: categories.clone(),
            });
        }
    }
    restricted
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
    /// runtime (the `MAX_XP_SOLVE_NODES` docs carry the measured figures for
    /// the spends-heavy shape, which is the one the bound is set from).
    fn rs_with_dead_pools(pool_count: usize) -> Ruleset {
        // The engine requires at least one V/F category-tagged "personality"
        // (`ENGINE_REQUIRED_CATEGORY_PERSONALITY`) once a catalogue is shipped
        // at all; unrelated to what this fixture is testing, but needed for
        // `Ruleset::from_sources` to pass integrity.
        let mut items = String::from(
            r#"[{"id":"flaw.filler","kind":"flaw","classification":"narrative","magnitude":"minor","categories":["personality"],"entity_kinds":["character"]}"#,
        );
        for i in 0..pool_count {
            items.push(',');
            items.push_str(&format!(
                r#"{{"id":"virtue.dead_pool_{i}","kind":"virtue","classification":"narrative","magnitude":"minor","categories":["general"],"effects":[{{"type":"restricted_ability_xp","amount":10,"abilities":["ability.artes_liberales"]}}]}}"#
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

    /// D56/A0 (§ 4 row 8, § 5 sub-slice 3): a companion (untrained profile)
    /// carrying a **test-only fixture** selection with
    /// `Effect::ConfersHermeticTraining`. Test-fixture-only per the design
    /// note: `flaw.abandoned_apprentice` is not edited until D3, so this
    /// proves only that the branch is now *capable* of selecting the
    /// apprenticeship shape — not that D3's truncated funding exists yet
    /// (`apprenticeship_of` stays profile-only, so `apprenticeship_xp`/
    /// `post_gauntlet_xp` are still 0 for a non-magus profile either way).
    const LIFE_STAGE_ITEMS: &str = r#"[
      { "id": "flaw.test_confers_training", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "major", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [{ "type": "confers_hermetic_training" }] },
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"] }
    ]"#;
    const LIFE_STAGE_ABILITIES: &str = r#"{
      "advancement": [{ "score": 1, "total_xp": 5 }],
      "abilities": [
        { "id": "ability.artes_liberales", "category": "general" },
        { "id": "ability.living_language", "category": "general", "parameter": "language" }
      ]
    }"#;
    const LIFE_STAGES: &str = r#"{
      "childhood": { "years": 5, "native_language_ability": "ability.living_language",
                     "native_language_xp": 75, "spread_xp": 45, "spread_abilities": [] },
      "later_life": { "xp_per_year": 15 }
    }"#;

    fn rs_with_life_stages() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: LIFE_STAGE_ITEMS,
            type_profiles: TYPES,
            abilities: Some(LIFE_STAGE_ABILITIES),
            life_stages: Some(LIFE_STAGES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn companion_with_life_stages(age: u32, trained: bool) -> Entity {
        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        e.ability_funding = crate::types::AbilityFunding::LifeStages;
        e.age = Some(age);
        e.life_stages = Some(crate::life_stage::LifeStagePlan::default());
        if trained {
            e.selections = vec![Selection::new(Id::new("flaw.test_confers_training"))];
        }
        e
    }

    #[test]
    fn apprenticeship_shaped_test_fixture_selects_the_apprenticeship_xp_branch_when_trained_off_profile()
     {
        let rs = rs_with_life_stages();

        // Untrained: later life (15/yr × 15 yrs = 225) is the GENERAL pool, and
        // there is no restricted LaterLife pool at all — today's status quo.
        let untrained = companion_with_life_stages(20, false);
        let untrained_alloc = checked_xp_allocation(&untrained, &rs).unwrap();
        assert_eq!(untrained_alloc.general_pool, 225);
        assert!(
            !untrained_alloc.restricted.iter().any(|p| matches!(
                p.origin,
                XpPoolOrigin::LifeStage {
                    block: LifeStageBlock::LaterLife
                }
            )),
            "{:?}",
            untrained_alloc.restricted
        );

        // Trained-by-selection: the general pool selects the apprenticeship
        // shape (apprenticeship_xp + post_gauntlet_xp — 0 + 0 today, since
        // D3 has not yet funded it), and the SAME 225 that used to be general
        // now surfaces as a restricted, Abilities-only LaterLife pool instead.
        let trained = companion_with_life_stages(20, true);
        let trained_alloc = checked_xp_allocation(&trained, &rs).unwrap();
        assert_eq!(trained_alloc.general_pool, 0);
        let later_life_pool = trained_alloc.restricted.iter().find(|p| {
            matches!(
                p.origin,
                XpPoolOrigin::LifeStage {
                    block: LifeStageBlock::LaterLife
                }
            )
        });
        assert_eq!(later_life_pool.map(|p| p.amount), Some(225));
    }

    /// Klaus F4 (round-1 MINOR), upper edge: the bound must refuse a spend
    /// count no legal character reaches, so an oversized save hits the
    /// friendly rejection *before* the solve rather than being accepted into
    /// it. The solve is `O(n^3)` and the UI re-pays it on every debounced
    /// `refresh()`, so "accepted, then runs for seconds" is the defect — an
    /// `Err` here is what keeps the worst accepted case sub-second.
    ///
    /// 1200 spends is above every term of the maximal-character arithmetic
    /// [`the_solve_bound_admits_a_maximal_legal_character`] pins, so no real
    /// character is refused by this.
    #[test]
    fn the_solve_bound_refuses_a_spend_count_no_legal_character_reaches() {
        let rs = rs();
        let oversized = companion_with_scores(1200);

        let err = checked_xp_allocation(&oversized, &rs).unwrap_err();

        assert_eq!(err.spends, 1200);
        assert_eq!(err.limit, MAX_XP_SOLVE_NODES);
    }

    /// Klaus F4, lower edge — the guard that stops the bound above from being
    /// lowered into a legal character. [`MAX_XP_SOLVE_NODES`] bounds one
    /// entity's own collections, never the ruleset's catalogue (CLAUDE.md,
    /// "Catalogue size is data, never code"), so every term below is a
    /// *character* collection size, each already an order of magnitude past
    /// any real one. The shipped catalogue is a sanity input, not the
    /// derivation: a bigger catalogue does not make a character bigger.
    #[test]
    fn the_solve_bound_admits_a_maximal_legal_character() {
        // 71 distinct non-parameterized Abilities, plus 20 instances of each
        // of the 7 parameterized ones (Area Lore, Craft, Language, …) — a
        // character with twenty Crafts is already beyond implausible.
        let ability_scores = 71 + 7 * 20;
        // One score per Art: 5 Techniques + 10 Forms.
        let art_scores = 15;
        // Only a spell with mastery >= 1 becomes a spend. An archmage who
        // mastered every spell in print (the shipped catalogue is ~360) and
        // invented two hundred more.
        let mastered_spells = 560;
        // One pool per restricted-XP grant held, plus four life-stage pools.
        // A real character has fewer than ten.
        let flow_pools = 64;

        let maximal = companion_with_scores(ability_scores + art_scores + mastered_spells);
        let nodes = xp_solve_scale(&maximal, &rs()).nodes() + flow_pools;

        assert!(
            nodes <= MAX_XP_SOLVE_NODES,
            "a maximal legal character needs {nodes} nodes but the bound is \
             {MAX_XP_SOLVE_NODES} — lowering it this far rejects a character \
             the app must be able to open",
        );
    }

    /// `restricted_xp_pools` is one of K1's four unguarded call sites — it must
    /// degrade to an empty list over the bound, never panic.
    #[test]
    fn restricted_xp_pools_degrades_to_empty_over_the_solve_bound() {
        let rs = rs();
        let past_bound = companion_with_scores(MAX_XP_SOLVE_NODES - 3 + 1);
        assert_eq!(restricted_xp_pools(&past_bound, &rs), Vec::new());
    }

    /// V55: `ability_authorizations` used to end in a bare `_ => {}` wildcard —
    /// the only non-exhaustive `Effect` match in `effective/`. Characterizes the
    /// behaviour the explicit match must preserve: only `RestrictedAbilityXp`,
    /// `AbilityAuthorization`, `AbilityScoreGrant`, and `AbilityBonusGated`
    /// (C1) contribute a permission; every other effect is a no-op.
    /// `AbilityBonus` (Puissant Ability) stands in
    /// for the rest — it names a target ability via `params[param]` but, per the
    /// rules text (ArMDE:4814-4816), grants no permission to own that ability,
    /// only a bonus once it is already legally held.
    #[test]
    fn ability_authorizations_reads_only_the_three_permission_granting_effects() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "virtue.warrior", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "restricted_ability_xp", "amount": 50, "categories": ["martial"] }] },
          { "id": "virtue.covenant_upbringing", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "ability_authorization", "abilities": ["ability.dead_language"] }] },
          { "id": "virtue.second_sight", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "ability_score_grant", "ability": "ability.second_sight", "amount": 1 }] },
          { "id": "virtue.puissant_ability", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
            "effects": [{ "type": "ability_bonus", "param": "ability", "amount": 2 }] }
        ]"#;
        let abilities_json = r#"{
          "advancement": [{ "score": 1, "total_xp": 5 }],
          "abilities": [
            { "id": "ability.artes_liberales", "category": "general" },
            { "id": "ability.dead_language", "category": "academic", "parameter": "language" },
            { "id": "ability.second_sight", "category": "supernatural" }
          ]
        }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: TYPES,
            abilities: Some(abilities_json),
            ..RulesetSources::default()
        })
        .unwrap();

        let mut e = companion_with_scores(0);
        e.selections = vec![
            Selection::new(Id::new("virtue.warrior")),
            Selection::new(Id::new("virtue.covenant_upbringing")),
            Selection::new(Id::new("virtue.second_sight")),
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.single_weapon"))]),
            ),
        ];

        let (abilities, categories) = ability_authorizations(&e, &rs);
        assert_eq!(categories, BTreeSet::from([AbilityCategory::Martial]));
        assert_eq!(
            abilities,
            BTreeSet::from([
                AuthorizedAbility {
                    ability: Id::new("ability.dead_language"),
                    instance: None,
                },
                AuthorizedAbility {
                    ability: Id::new("ability.second_sight"),
                    instance: None,
                },
            ])
        );
        // Puissant Ability names ability.single_weapon via `params[param]` but
        // must not appear: AbilityBonus grants no ownership permission.
        assert!(
            !abilities
                .iter()
                .any(|a| a.ability == Id::new("ability.single_weapon"))
        );
    }
}
