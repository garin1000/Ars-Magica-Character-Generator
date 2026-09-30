//! X3a (`tmp/x3-scope.md`, `docs/vf-audit/decisions.md` D12/D24/D56/D68) — the
//! *trained* gate on Hermetic Virtues and Flaws.
//!
//! D12: every `hermetic`-category entry is either **intrinsic** (operates on
//! The Gift itself — the three entries that already gate on
//! `virtue.the_gift`) or **trained** (operates on Techniques, Forms, spells,
//! Casting/Lab Totals, Parma Magica, Arcane Connections, certámen, or
//! Twilight — all trained-only per D68.1, which collapses D12 to "every
//! Hermetic entry except the three Gift ones requires Hermetic training").
//! D24.6/D12.6's acceptance criterion: **no Gifted non-magus computes against
//! a magus budget.**
//!
//! D68 settles the shape: (1) all 15 previously-ambiguous entries are
//! trained; (2) the gate is `order_member` where the passage names the
//! Order, a House, or the Gauntlet, and `hermetically_trained` otherwise;
//! (3) training is stated twice — the `trained` flag plus a `prerequisites`
//! gate — with a guard that they agree; no engine change.
//!
//! **X3a's data pass has landed** (the 55 Hermetic Virtues, `tmp/x3-scope.md`
//! § 2, plus the workspace-wide guards). **X3b's Phase 1 has landed its own
//! 33 Hermetic Flaws — `flaw.bound_casting_tools` through
//! `flaw.necessary_condition` — into `X3B_FLAWS`, RED until X3b's Phase 2 sets
//! their data.** **X3c's Phase 1 (this file) has now landed its own 31
//! Flaws — `flaw.painful_magic` through `flaw.weird_magic` — into
//! `X3C_FLAWS`, RED until X3c's Phase 2 sets their data.**
//! `PENDING_HERMETIC_FLAWS` is now empty (X3c's own 31 ids all moved into
//! `X3C_FLAWS`) but kept, not deleted, for the same reason
//! `PENDING_HERMETIC_MAGUS_SPELLING` is kept empty: it is still referenced by
//! name (the `flaw.weak_parens`/`flaw.restriction` behavioral tests below,
//! and the whole-catalogue guard's bucket list), and an empty-but-named list
//! is the honest shape if a future hermetic entry is ever added mid-catalogue
//! and needs staging. `PENDING_HERMETIC_MAGUS_SPELLING` is likewise still
//! empty — X3b's single occupant, `flaw.deficient_technique`, moved to
//! `X3B_FLAWS` and its normalization is that slice's own RED. See
//! `tmp/x3a-verdicts.md` / `tmp/x3b-verdicts.md` / `tmp/x3c-verdicts.md` for
//! the full per-entry citation and `tmp/x3a-handover.md` / `tmp/x3b-handover.md`
//! / `tmp/x3c-handover.md` for what each slice's Phase 2 must do.
//!
//! Per CLAUDE.md's catalogue-size invariant, the whole-catalogue guard
//! (`hermetic_catalogue_is_fully_classified_by_x3a_or_pending`) never asserts
//! a total count — every hermetic entry must land in exactly one of five
//! named buckets (the three Gift entries / X3a's 55 / X3b's 33 / X3c's 31 /
//! the now-always-empty pending list), and an entry in none of them fails
//! loudly rather than being silently skipped.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationResult, validate};
use arm_rules::{XpPoolOrigin, checked_xp_allocation, compute_balance, restricted_xp_pools};

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        houses: Some(SHIPPED_HOUSES),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .unwrap()
}

/// Same shape as `x1_authorization_family.rs::entity` / `data_integrity.rs::entity` —
/// duplicated since integration test binaries cannot share private helpers.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn issue_codes(result: &ValidationResult) -> Vec<&str> {
    result.issues.iter().map(|i| i.code.as_str()).collect()
}

/// A Gifted, non-magus companion — `virtue.the_gift` alone, nothing else.
/// `xp_pool` is set generously so a shortfall can only come from the
/// prereq/budget behavior under test, never from ordinary insufficient XP.
fn gifted_companion(extra: Vec<Selection>) -> Entity {
    let mut selections = vec![Selection::new(Id::new("virtue.the_gift"))];
    selections.extend(extra);
    let mut e = entity("companion", selections);
    e.xp_pool = 1000;
    e
}

// ---------------------------------------------------------------------------
// Prereq-tree helpers: a "gate" is a HermeticallyTrained or OrderMember leaf,
// at the top level or nested one level under `All` (the only compound shape
// the shipped data uses for this — see `flaw.primogeniture_lineage`,
// `all[order_member, house(house.verditius)]`).
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Gate {
    Trained,
    Order,
}

fn contains_gate(prereq: &Prereq, gate: Gate) -> bool {
    match (prereq, gate) {
        (Prereq::HermeticallyTrained, Gate::Trained) => true,
        (Prereq::OrderMember, Gate::Order) => true,
        (Prereq::All(children), _) => children.iter().any(|c| contains_gate(c, gate)),
        _ => false,
    }
}

fn contains_any_gate(prereq: &Prereq) -> bool {
    contains_gate(prereq, Gate::Trained) || contains_gate(prereq, Gate::Order)
}

fn contains_house(prereq: &Prereq, house: &Id) -> bool {
    match prereq {
        Prereq::House(h) => h == house,
        Prereq::All(children) => children.iter().any(|c| contains_house(c, house)),
        _ => false,
    }
}

/// D12.3: the one shipped `Has(virtue.hermetic_magus)` spelling (`flaw.deficient_technique`)
/// must normalize to `HermeticallyTrained` — checked recursively since a future
/// author might bury it under `All`/`Any`/`Nor`.
fn contains_has_hermetic_magus(prereq: &Prereq) -> bool {
    match prereq {
        Prereq::Has(id) => id.as_str() == "virtue.hermetic_magus",
        Prereq::All(children) | Prereq::Any(children) | Prereq::Nor(children) => {
            children.iter().any(contains_has_hermetic_magus)
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Group A — D12.6/D24.6 behavioral acceptance: "no Gifted non-magus computes
// against a magus budget." Each test's shape is the OR the scope doc's own
// Test 3 states (`tmp/x3-scope.md` § 2): a Gifted companion holding a trained
// Hermetic entry either gets `prereq_not_met`, or the entry's budget effect is
// absent — never both branches false. This is achievable with NO engine
// change (D68.3): `general_xp_bonus`/`restricted_xp_pools`/`compute_balance`
// keep folding every selection's effect unconditionally (by design — the
// engine always computes, CLAUDE.md's "one evaluation path"), so the ONLY
// lever Phase 2's data-only fix has is the first branch. Today, with no gate
// on any entry, the first branch never fires and the second is the live
// defect this file documents — so each test is RED for the right reason.
// ---------------------------------------------------------------------------

/// `virtue.skilled_parens` (X3a scope — must go GREEN once Phase 2 lands).
#[test]
fn d12_6_gifted_companion_skilled_parens_refused_or_neutral() {
    let rs = load_ruleset();
    let skilled = Id::new("virtue.skilled_parens");
    let e = gifted_companion(vec![Selection::new(skilled.clone())]);
    let result = validate(&e, &rs);
    let refused = result
        .issues
        .iter()
        .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&skilled));
    let general_bonus = checked_xp_allocation(&e, &rs).unwrap().general_bonus;
    assert!(
        refused || general_bonus == 0,
        "a Gifted non-magus companion holding Skilled Parens must either be refused \
         (prereq_not_met) or gain none of its +60 general XP — issues: {:?}, general_bonus: {}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>(),
        general_bonus
    );
}

/// `virtue.clan_ilfetu` (X3a scope — must go GREEN once Phase 2 lands).
#[test]
fn d12_6_gifted_companion_clan_ilfetu_refused_or_pool_absent() {
    let rs = load_ruleset();
    let ilfetu = Id::new("virtue.clan_ilfetu");
    let e = gifted_companion(vec![Selection::new(ilfetu.clone())]);
    let result = validate(&e, &rs);
    let refused = result
        .issues
        .iter()
        .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&ilfetu));
    let pool_present = restricted_xp_pools(&e, &rs).iter().any(|p| {
        p.origin
            == XpPoolOrigin::Item {
                item: ilfetu.clone(),
            }
    });
    assert!(
        refused || !pool_present,
        "a Gifted non-magus companion holding Clan Ilfetu must either be refused \
         (prereq_not_met) or gain none of its 50-point restricted pool — issues: {:?}, \
         pool_present: {pool_present}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

/// `flaw.weak_parens` (X3c scope; kept here because it is D12.6's own worked
/// example, ArMDE:7072-7074). While `flaw.weak_parens` is on
/// `PENDING_HERMETIC_FLAWS`, this asserts the live defect persists (the guard
/// can only shrink the pending list, never widen it); once X3c gates it, the
/// pending row is removed and this asserts the fix instead.
#[test]
fn d12_6_gifted_companion_weak_parens_refused_or_neutral() {
    let rs = load_ruleset();
    let weak = Id::new("flaw.weak_parens");
    let e = gifted_companion(vec![Selection::new(weak.clone())]);
    let result = validate(&e, &rs);
    let refused = result
        .issues
        .iter()
        .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&weak));
    let general_bonus = checked_xp_allocation(&e, &rs).unwrap().general_bonus;
    if PENDING_HERMETIC_FLAWS.contains(&weak.as_str()) {
        assert!(
            !refused && general_bonus != 0,
            "flaw.weak_parens is on PENDING_HERMETIC_FLAWS (X3c) but no longer shows the live \
             defect (refused: {refused}, general_bonus: {general_bonus}) — remove it from the \
             pending list and update this test to assert the fix"
        );
    } else {
        assert!(
            refused || general_bonus == 0,
            "a Gifted non-magus companion holding Weak Parens must either be refused \
             (prereq_not_met) or lose none of its -60 general XP — issues: {:?}, general_bonus: {}",
            result.issues.iter().map(|i| &i.code).collect::<Vec<_>>(),
            general_bonus
        );
    }
}

/// `flaw.restriction` (X3c scope — Major Hermetic Flaw, ArMDE:6691-6694, no
/// prerequisite shipped today): "Flaw points from a trained Flaw". While
/// `flaw.restriction` is on `PENDING_HERMETIC_FLAWS`, this asserts the live
/// defect persists; once X3c gates it, the pending row is removed and this
/// asserts the fix instead. (This comment previously said "X3b scope" —
/// mislabeled; `flaw.restriction` was always in X3c's alphabetical span,
/// `flaw.painful_magic`…`flaw.weird_magic`.)
#[test]
fn d12_6_gifted_companion_trained_flaw_banks_no_points_or_refused() {
    let rs = load_ruleset();
    let restriction = Id::new("flaw.restriction");
    let baseline = gifted_companion(vec![]);
    let baseline_flaw_points = compute_balance(&baseline, &rs).flaw_points;

    let e = gifted_companion(vec![Selection::new(restriction.clone())]);
    let result = validate(&e, &rs);
    let refused = result
        .issues
        .iter()
        .any(|i| i.code == "prereq_not_met" && i.context.as_ref() == Some(&restriction));
    let flaw_points = compute_balance(&e, &rs).flaw_points;
    if PENDING_HERMETIC_FLAWS.contains(&restriction.as_str()) {
        assert!(
            !refused && flaw_points != baseline_flaw_points,
            "flaw.restriction is on PENDING_HERMETIC_FLAWS (X3c) but no longer shows the live \
             defect (refused: {refused}, flaw_points: {flaw_points}, baseline: \
             {baseline_flaw_points}) — remove it from the pending list and update this test to \
             assert the fix"
        );
    } else {
        assert!(
            refused || flaw_points == baseline_flaw_points,
            "a Gifted non-magus companion holding a trained Major Hermetic Flaw (Restriction) \
             must either be refused (prereq_not_met) or bank none of its Flaw points — issues: \
             {:?}, flaw_points: {flaw_points} (baseline {baseline_flaw_points})",
            result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
        );
    }
}

/// The grog half of D12.6's acceptance criterion. Unlike the companion cases
/// above, this is NOT expected to be red: D56 already documents the grog
/// profile blocking The Gift four ways over
/// (`forbidden_traits: ["virtue.the_gift"]`, `forbidden_categories: ["hermetic"]`,
/// `gift_policy: "forbidden"`, `max_major_flaws`/category caps). A forced
/// selection of both is refused today by that existing, independent
/// machinery — this test is a completeness/regression check, not a new red.
#[test]
fn d12_6_gifted_grog_hermetic_entry_is_refused() {
    let rs = load_ruleset();
    let gift = Id::new("virtue.the_gift");
    let restriction = Id::new("flaw.restriction");
    let mut e = entity(
        "grog",
        vec![
            Selection::new(gift.clone()),
            Selection::new(restriction.clone()),
        ],
    );
    e.xp_pool = 1000;
    let result = validate(&e, &rs);
    let codes = issue_codes(&result);
    assert!(
        codes.contains(&"forbidden_trait") || codes.contains(&"category_not_permitted"),
        "a Gifted grog holding a Hermetic entry must be refused (forbidden_trait on the Gift, \
         and/or category_not_permitted on the Hermetic entry) — issues: {codes:?}"
    );
}

// ---------------------------------------------------------------------------
// Group B — D68.3 structural guards, workspace-wide (not X3a-scoped).
// ---------------------------------------------------------------------------

/// D68.3's core invariant, scoped to `hermetic`-category items — D12's own
/// classification domain ("every hermetic entry…"). A non-Hermetic-category
/// item may legitimately carry an `order_member`/`hermetically_trained` leaf
/// for an unrelated reason with no `trained` flag of its own: shipped
/// `flaw.primogeniture_lineage` is `categories: ["story"]` with only
/// `index_categories: ["hermetic"]` (provenance, not membership — see
/// `PointItem::index_categories`'s own doc comment) and carries
/// `all[order_member, house(house.verditius)]` to gate a Diedne-secret
/// storyline, nothing to do with D12's intrinsic/trained split. Scoping this
/// invariant to `categories` (not `index_categories`) keeps that entry out of
/// scope, matching D68.3's own wording: `trained` and its gate agree for
/// every `hermetic` entry. True vacuously today (both sides false
/// everywhere); must stay true once Phase 2 sets both together.
#[test]
fn trained_flag_and_gate_always_agree() {
    let rs = load_ruleset();
    for item in rs.items_by_category("hermetic") {
        let gated = item.prerequisites.as_ref().is_some_and(contains_any_gate);
        assert_eq!(
            item.trained, gated,
            "{}: `trained` ({}) and a hermetically_trained/order_member prerequisite \
             leaf ({gated}) must agree",
            item.id, item.trained
        );
    }
}

/// D12.3: `flaw.deficient_technique` is the one shipped entry using the
/// retired `Has(virtue.hermetic_magus)` spelling. X3b (this file, Phase 1)
/// moves it into `X3B_FLAWS` — normalizing the spelling to
/// `HermeticallyTrained` alongside setting `trained: true` is exactly the
/// data change X3b's Phase 2 owes, so this list is empty now and this test is
/// unconditionally RED on `flaw.deficient_technique` until that data lands.
/// Kept as a named list (not deleted outright) so a future slice that ever
/// needs to stage a spelling fix has the same honest-pending shape to reuse.
const PENDING_HERMETIC_MAGUS_SPELLING: &[&str] = &[];

/// D12.3: no prerequisite anywhere in the catalogue may use the retired
/// `Has(virtue.hermetic_magus)` spelling, except the entries on
/// `PENDING_HERMETIC_MAGUS_SPELLING` (empty as of X3b), which must still show
/// the old spelling — the guard can only shrink that list, never widen it.
#[test]
fn no_prerequisite_uses_has_hermetic_magus_spelling() {
    let rs = load_ruleset();
    for item in rs.items() {
        let uses_old_spelling = item
            .prerequisites
            .as_ref()
            .is_some_and(contains_has_hermetic_magus);
        if PENDING_HERMETIC_MAGUS_SPELLING.contains(&item.id.as_str()) {
            assert!(
                uses_old_spelling,
                "{}: is on PENDING_HERMETIC_MAGUS_SPELLING (X3b) but no longer uses \
                 Has(virtue.hermetic_magus) — remove it from the pending list",
                item.id
            );
        } else {
            assert!(
                !uses_old_spelling,
                "{}: prerequisites must not use Has(virtue.hermetic_magus) — normalize to \
                 HermeticallyTrained (D12.3/D56)",
                item.id
            );
        }
    }

    for id in PENDING_HERMETIC_MAGUS_SPELLING {
        rs.item(&Id::new(*id)).unwrap_or_else(|| {
            panic!("PENDING_HERMETIC_MAGUS_SPELLING: {id} must exist in the shipped catalogue")
        });
    }
}

/// The three intrinsic Gift entries operate on The Gift itself and are the
/// one Hermetic-category exception D12 draws: neither `trained` nor any gate.
const GIFT_ENTRIES: &[&str] = &[
    "flaw.blatant_gift",
    "virtue.gentle_gift",
    "flaw.suppressed_gift",
];

/// X3a's 55 Hermetic Virtues (`tmp/x3-scope.md` § 2 — all Hermetic-category
/// Virtues except `virtue.gentle_gift`). Each entry: the expected gate kind,
/// with the passage wording that decides it (D68.2: `order_member` where the
/// text names the Order, a House, or the Gauntlet; `hermetically_trained`
/// otherwise). Sorted by rulebook line (`tmp/x3-scope.md`'s own order).
///
/// The 47 `Gate::Trained` entries below operate on Techniques/Forms/spells/
/// Casting or Lab Totals/Parma/certámen/Twilight — D12's own criterion — with
/// no Order/House/Gauntlet audience language; a per-id citation for all 47
/// would just repeat that same fact 47 times, so only the entries whose
/// passage is genuinely borderline carry an inline note explaining why they
/// stayed `Trained` despite mentioning one of those words in passing.
const X3A_VIRTUES: &[(&str, Gate)] = &[
    ("virtue.adept_laboratory_student", Gate::Trained), // ArMDE:3368-3371, Lab Total
    ("virtue.affinity_art", Gate::Trained),             // ArMDE:3376-3378, an Art
    ("virtue.atlantean_magic", Gate::Trained), // ArMDE:3444-3469, Hermetic tradition/spells
    ("virtue.boosted_magic", Gate::Trained),   // ArMDE:3523-3528, spell levels
    ("virtue.cautious_sorcerer", Gate::Trained), // ArMDE:3555-3558, Casting Total/botch
    (
        "virtue.clan_ilfetu",
        Gate::Order,
        // ArMDE:3565: "a member of Clan Ilfetu within House Bjornaer" — folded
        // House gate (F-35); see `x3a_folded_house_gates_carry_their_house_leaf`.
    ),
    ("virtue.cyclic_magic_positive", Gate::Trained), // ArMDE:3635-3638, spellcasting
    ("virtue.deft_form", Gate::Trained),             // ArMDE:3645-3648, a Form
    (
        "virtue.diedne_magic",
        Gate::Order,
        // ArMDE:3675-3682: "hidden from the Order" — D68.12 settles the
        // borderline reading raised in tmp/x3a-verdicts.md: gates on
        // order_member as well as training.
    ),
    ("virtue.elemental_magic", Gate::Trained), // ArMDE:3731-3738, Forms/spontaneous magic
    ("virtue.enduring_magic", Gate::Trained),  // ArMDE:3755-3758, spell duration/Casting
    ("virtue.the_enigma", Gate::Order),        // ArMDE:3759-3761: "a member of House Criamon"
    (
        "virtue.exotic_casting",
        Gate::Trained,
        // ArMDE:3775-3778 mentions "Magic Theory within the Order" only as
        // background contrast (non-Hermetic casting methods vs. standard
        // Hermetic ones), not an eligibility clause. BORDERLINE — see report.
    ),
    ("virtue.extractor_of_form_vis", Gate::Trained), // ArMDE:3779-3782, lab vis extraction
    ("virtue.faerie_magic", Gate::Order), // ArMDE:3825-3827: "a member of House Merinita"
    ("virtue.faerie_raised_magic", Gate::Trained), // ArMDE:3829-3842, Hermetic magic style
    ("virtue.fast_caster", Gate::Trained), // ArMDE:3865-3868, Casting Total speed
    ("virtue.flawless_magic", Gate::Trained), // ArMDE:3887-3890, botch on spells
    ("virtue.flexible_formulaic_magic", Gate::Trained), // ArMDE:3891-3894, formulaic spells
    ("virtue.free_study", Gate::Trained), // ArMDE:3937-3940, D68.1's raw-vis group
    (
        "virtue.gorgiastic",
        Gate::Order,
        // ArMDE:3979-3982: left House Criamon but still in the Order —
        // D68.12 settles the borderline reading raised in
        // tmp/x3a-verdicts.md: gates on order_member as well as training.
    ),
    ("virtue.guest_of_house_criamon", Gate::Order), // ArMDE:4037-4040: "Magi... members of House Criamon"
    ("virtue.harnessed_magic", Gate::Trained),      // ArMDE:4053-4058, spontaneous magic
    ("virtue.heartbeast", Gate::Order),             // ArMDE:4059-4061: "a member of House Bjornaer"
    ("virtue.hermetic_prestige", Gate::Order), // ArMDE:4071-4073: "a Reputation... within the Order"
    ("virtue.imbued_with_the_spirit_of_form", Gate::Trained), // ArMDE:4085-4094, a Form
    ("virtue.inventive_genius", Gate::Trained), // ArMDE:4151-4154, spell invention
    (
        "virtue.leper_magus",
        Gate::Order,
        // ArMDE:4249-4252: "only available to magi trained in House Tytalus"
        // — folded House gate (F-136); see
        // `x3a_folded_house_gates_carry_their_house_leaf`.
    ),
    ("virtue.life_boost", Gate::Trained), // ArMDE:4295-4298, Casting Total
    ("virtue.life_linked_spontaneous_magic", Gate::Trained), // ArMDE:4299-4306, spontaneous magic
    ("virtue.magical_memory", Gate::Trained), // ArMDE:4355-4358, spell memorization
    ("virtue.major_magical_focus", Gate::Trained), // ArMDE:4399-4422, Casting Total
    ("virtue.mastered_spells", Gate::Trained), // ArMDE:4471-4475, Spell Mastery
    (
        "virtue.masterpiece",
        Gate::Order,
        // ArMDE:4476-4479: passing "his Gauntlet" — D68.12 settles the
        // borderline reading raised in tmp/x3a-verdicts.md: gates on
        // order_member as well as training.
    ),
    (
        "virtue.mercurian_magic",
        Gate::Trained,
        // ArMDE:4514-4523 mentions lineage predating "the Order of Hermes" as
        // background, not an eligibility clause.
    ),
    ("virtue.method_caster", Gate::Trained), // ArMDE:4524-4527, Casting Total
    ("virtue.minor_magical_focus", Gate::Trained), // ArMDE:4536-4538 (range defect noted for X9c), Casting Total
    ("virtue.mystical_choreography", Gate::Trained), // ArMDE:4567-4572, spellcasting
    (
        "virtue.mythic_blood",
        Gate::Trained,
        // ArMDE:4573-4589 (D68.1's own group): "not particularly uncommon in
        // the Order of Hermes" is a rarity remark, not an eligibility clause.
    ),
    ("virtue.performance_magic", Gate::Trained), // ArMDE:4642-4709, spellcasting
    ("virtue.personal_vis_source", Gate::Trained), // ArMDE:4728-4731, D68.1's raw-vis group
    ("virtue.potent_magic_major", Gate::Trained), // ArMDE:4740-4781, spell effect
    ("virtue.potent_magic_minor", Gate::Trained), // ArMDE:4740-4781, spell effect
    ("virtue.puissant_art", Gate::Trained),      // ArMDE:4818-4820, an Art
    ("virtue.quiet_magic", Gate::Trained),       // ArMDE:4822-4827, spellcasting
    ("virtue.secondary_insight", Gate::Trained), // ArMDE:4892-4895, Arts
    ("virtue.side_effect", Gate::Trained),       // ArMDE:4954-4957, spell effect
    ("virtue.skilled_parens", Gate::Trained), // ArMDE:4964-4966 — D24: a parens exists only via apprenticeship
    ("virtue.special_circumstances", Gate::Trained), // ArMDE:4998-5001, D68.1's Magic Resistance group
    ("virtue.spell_improvisation", Gate::Trained),   // ArMDE:5002-5005, spontaneous magic
    ("virtue.study_bonus", Gate::Trained),           // ArMDE:5056-5072, lab study
    ("virtue.subtle_magic", Gate::Trained),          // ArMDE:5073-5076, spellcasting
    ("virtue.tethered_magic", Gate::Trained),        // ArMDE:5141-5144, spellcasting
    ("virtue.verditius_magic", Gate::Order), // ArMDE:5215-5217: "a member of House Verditius"
    ("virtue.withstand_casting", Gate::Trained), // ArMDE:5261-5282, Casting Total
];

/// X3b's 33 Hermetic Flaws (`tmp/x3-scope.md` § 2:
/// `flaw.bound_casting_tools`…`flaw.necessary_condition`, alphabetical). Same
/// shape as `X3A_VIRTUES`: the expected gate kind, with the passage wording
/// that decides it (D68.2: `order_member` where the text names the Order, a
/// House, or the Gauntlet; `hermetically_trained` otherwise). RED until
/// X3b's Phase 2 sets the data.
///
/// Most entries operate on Techniques/Forms/spells/Casting or Lab
/// Totals/Parma/Spell Mastery — D12's own criterion — with no Order/House/
/// Gauntlet audience language; a per-id citation for each would just repeat
/// that same fact, so only the genuinely borderline or House-gated entries
/// carry an inline note.
const X3B_FLAWS: &[(&str, Gate)] = &[
    (
        "flaw.bound_casting_tools",
        Gate::Trained,
        // ArMDE:5723-5726 mentions "House Verditius" only to explain that
        // casting tools are a Verditius-specific mechanic (ArMDE:767,
        // :10090) — background, not an eligibility clause, unlike Consumed
        // Casting Tools' explicit "may only be taken by Verditius magi"
        // three entries later (ArMDE:5841). BORDERLINE — see report.
    ),
    ("flaw.bound_magic", Gate::Trained), // ArMDE:5727-5730, spells/magic items
    (
        "flaw.brutal_artist",
        Gate::Order,
        // ArMDE:5757-5760: "only available to magi of House Jerbiton" —
        // folded House gate (F-359); see
        // `x3b_folded_house_gates_carry_their_house_leaf`.
    ),
    ("flaw.careless_sorcerer", Gate::Trained), // ArMDE:5769-5772, botch dice when casting spells
    ("flaw.ceremonial_spontaneous_magic", Gate::Trained), // ArMDE:5781-5784, Spontaneous magic
    ("flaw.chaotic_magic", Gate::Trained),     // ArMDE:5785-5788, spontaneous spell effect
    ("flaw.clumsy_magic", Gate::Trained),      // ArMDE:5801-5804, aiming rolls/Finesse
    (
        "flaw.consumed_casting_tools",
        Gate::Order,
        // ArMDE:5839-5842: "This Flaw may only be taken by Verditius magi" —
        // folded House gate (F-390); see
        // `x3b_folded_house_gates_carry_their_house_leaf`.
    ),
    ("flaw.corrupted_arts", Gate::Trained), // ArMDE:5853-5858, an Art's Casting Total
    ("flaw.corrupted_spells", Gate::Trained), // ArMDE:5859-5864, formulaic spells
    ("flaw.creative_block", Gate::Trained), // ArMDE:5873-5876, Lab Total
    ("flaw.cyclic_magic_negative", Gate::Trained), // ArMDE:5893-5896, Lab Total/Casting Score
    ("flaw.deficient_form", Gate::Trained), // ArMDE:5909-5912, a Form (D12's own worked example)
    (
        "flaw.deficient_technique",
        Gate::Trained,
        // ArMDE:5913-5915, a Technique. D12.3/D56: normalizes the retired
        // Has(virtue.hermetic_magus) spelling to HermeticallyTrained — see
        // `no_prerequisite_uses_has_hermetic_magus_spelling`.
    ),
    ("flaw.deleterious_circumstances", Gate::Trained), // ArMDE:5917-5920, magic totals (D68.1)
    ("flaw.difficult_longevity_ritual", Gate::Trained), // ArMDE:5962-5965, Lab Total (D68.1)
    ("flaw.difficult_spontaneous_magic", Gate::Trained), // ArMDE:5966-5971, Spontaneous magic
    ("flaw.disjointed_magic", Gate::Trained), // ArMDE:5972-5975, spells/Techniques and Forms
    ("flaw.disorientating_magic", Gate::Trained), // ArMDE:5984-5987, after casting a spell
    ("flaw.environmental_magic_condition", Gate::Trained), // ArMDE:6020-6023, Casting/Lab Totals
    ("flaw.exciting_experimentation", Gate::Trained), // ArMDE:6040-6043, lab experimentation tables
    ("flaw.fettered_magic", Gate::Trained), // ArMDE:6114-6117, spells/magic items as Arcane Connections
    ("flaw.flawed_parma_magica", Gate::Trained), // ArMDE:6142-6145, Parma Magica
    ("flaw.harmless_magic", Gate::Trained), // ArMDE:6230-6235, Perdo spells
    (
        "flaw.hedge_wizard",
        Gate::Order,
        // ArMDE:6240-6243: "a negative Reputation within the Order of
        // Hermes... even though you are a member of the Order" — an
        // eligibility/membership fact (D68.2), not flavor; same shape as
        // `virtue.hermetic_prestige` (X3a).
    ),
    ("flaw.incompatible_arts", Gate::Trained), // ArMDE:6290-6293, Techniques and Forms
    ("flaw.inconstant_magic", Gate::Trained),  // ArMDE:6298-6301, spellcasting/Finesse
    (
        "flaw.infamous_master",
        Gate::Order,
        // ArMDE:6314-6317: "don't deserve to be a member of the Order... a
        // bad Reputation... among magi" — same shape as Hedge Wizard.
    ),
    ("flaw.limited_magic_resistance", Gate::Trained), // ArMDE:6346-6349, Magic Resistance/Parma
    ("flaw.loose_magic", Gate::Trained),              // ArMDE:6354-6357, Spell Mastery
    ("flaw.magic_addiction", Gate::Trained),          // ArMDE:6378-6381, spellcasting
    (
        "flaw.monastic_vows_hermetic",
        Gate::Trained,
        // ArMDE:6450-6453 mentions "your order" (ArMDE:6452) — a religious
        // order (monastic vows), NOT the Order of Hermes; operates on vis
        // ownership and magic devices.
    ),
    ("flaw.necessary_condition", Gate::Trained), // ArMDE:6476-6479, casting spells
];

/// X3c's 31 Hermetic Flaws (`tmp/x3-scope.md` § 2: `flaw.painful_magic`…
/// `flaw.weird_magic`, alphabetical). Same shape as `X3B_FLAWS`: the expected
/// gate kind, with the passage wording that decides it (D68.2: `order_member`
/// where the text names the Order, a House, or the Gauntlet;
/// `hermetically_trained` otherwise). RED until X3c's Phase 2 sets the data.
///
/// 30 of the 31 operate on Techniques/Forms/spells/Casting or Lab
/// Totals/Parma/Twilight/raw vis — D12's own criterion — with no Order/House/
/// Gauntlet audience language; a per-id citation for each would just repeat
/// that same fact, so only the one House-gated entry carries an inline note.
/// `flaw.vulnerable_casting`'s "Magi of the Mercurian cults" mention is
/// background flavor, not an eligibility clause — the same reading D68.12
/// already gave `virtue.mercurian_magic`'s near-identical wording in X3a — so
/// it is not listed on the STOP-list below.
const X3C_FLAWS: &[(&str, Gate)] = &[
    ("flaw.painful_magic", Gate::Trained), // ArMDE:6574-6577, casting-fatigue penalty
    ("flaw.poor_formulaic_magic", Gate::Trained), // ArMDE:6610-6613, Formulaic spell rolls
    ("flaw.restriction", Gate::Trained),   // ArMDE:6691-6694, casting spells (Q-X4-1's own example)
    ("flaw.rigid_magic", Gate::Trained),   // ArMDE:6695-6698, vis when casting/Ritual magic
    ("flaw.short_lived_magic", Gate::Trained), // ArMDE:6729-6732, spell duration
    ("flaw.short_ranged_magic", Gate::Trained), // ArMDE:6737-6740, Casting/Lab Totals
    ("flaw.slow_caster", Gate::Trained),   // ArMDE:6755-6758, spellcasting time
    (
        "flaw.spontaneous_casting_tools",
        Gate::Order,
        // ArMDE:6779-6782: "This Flaw can only be taken by Verditius magi" —
        // folded House gate (F-518); see
        // `x3c_folded_house_gate_carries_its_house_leaf`.
    ),
    ("flaw.stockade_parma_magica", Gate::Trained), // ArMDE:6787-6790, Parma Magica
    ("flaw.study_requirement", Gate::Trained),     // ArMDE:6795-6798, studying an Art
    ("flaw.susceptibility_to_divine_power", Gate::Trained), // ArMDE:6815-6818, D68.1's Magic Resistance group
    ("flaw.susceptibility_to_faerie_power", Gate::Trained), // ArMDE:6819-6822, Magic Resistance/Parma
    ("flaw.susceptibility_to_infernal_power", Gate::Trained), // ArMDE:6823-6826, Magic Resistance
    ("flaw.the_constant_expression", Gate::Trained), // ArMDE:5821-5838, Spontaneous magic/Lab safety
    ("flaw.twilight_prone", Gate::Trained),          // ArMDE:6879-6882, Twilight
    (
        "flaw.unimaginative_learner",
        Gate::Trained,
        // ArMDE:6915-6918, D68.1's raw-vis group (studying from raw vis)
    ),
    ("flaw.unnatural_magic", Gate::Trained), // ArMDE:6931-6934, Creo rituals/vis extraction
    ("flaw.unpredictable_magic", Gate::Trained), // ArMDE:6935-6938, casting magic/botch dice
    ("flaw.unstructured_caster", Gate::Trained), // ArMDE:6947-6950, Formulaic/Ritual spells
    (
        "flaw.vulnerable_casting",
        Gate::Trained,
        // ArMDE:6993-7004 mentions "Magi of the Mercurian cults" only as
        // background (who commonly has the Flaw), not an eligibility clause —
        // same reading as `virtue.mercurian_magic` (X3a, D68.12).
    ),
    ("flaw.vulnerable_magic", Gate::Trained), // ArMDE:7005-7010, dispelling active spells
    ("flaw.vulnerable_to_folk_tradition", Gate::Trained), // ArMDE:7011-7014, Magic Resistance/Penetration
    ("flaw.warped_magic", Gate::Trained),                 // ArMDE:7023-7026, spell side effect
    (
        "flaw.waster_of_vis",
        Gate::Trained,
        // ArMDE:7052-7055, D68.1's raw-vis group (using raw vis/Ritual casting)
    ),
    ("flaw.weak_enchanter", Gate::Trained), // ArMDE:7060-7063, Lab Total (enchanted items)
    ("flaw.weak_magic", Gate::Trained),     // ArMDE:7064-7067, Penetration Total
    ("flaw.weak_magic_resistance", Gate::Trained), // ArMDE:7068-7071, Magic Resistance/Penetration
    (
        "flaw.weak_parens",
        Gate::Trained,
        // ArMDE:7072-7074 — D24: a parens exists only via apprenticeship;
        // D12.6's own worked example.
    ),
    ("flaw.weak_scholar", Gate::Trained), // ArMDE:7080-7083, Lab Totals
    ("flaw.weak_spontaneous_magic", Gate::Trained), // ArMDE:7084-7089, Casting Score/spontaneous magic
    ("flaw.weird_magic", Gate::Trained),            // ArMDE:7098-7101, botch dice on spellcasting
];

/// X3a/X3b/X3c's data pass now covers all 122 `hermetic`-category entries
/// (`tmp/x3-scope.md` § 1's own count: 3 Gift + 55 Virtues + 33 + 31 Flaws),
/// so no hermetic entry should ever need to be staged here again. Kept empty
/// rather than deleted, for the same reason `PENDING_HERMETIC_MAGUS_SPELLING`
/// is kept empty: both the `flaw.weak_parens`/`flaw.restriction` behavioral
/// tests below and the whole-catalogue guard's bucket list still reference it
/// by name, and an empty-but-named list is the honest shape to reuse if a
/// future hermetic entry is ever added mid-catalogue and needs staging before
/// its own gate lands.
// `flaw.vengeful_powers` (X6b, design-x6-parameters.md): its passage lets it
// be "taken as a Hermetic Flaw" OR the ordinary Story reading (ArMDE:6975),
// recorded by a `taken_as` Category parameter — but `trained`/`prerequisites`
// gate the WHOLE item unconditionally, with no per-value conditional
// prerequisite mechanism in the engine, so gating it here would wrongly
// block the Story reading for a non-magus. Left pending rather than gated
// wrong; a real per-value Prereq gate is an engine change outside this
// data-only slice.
const PENDING_HERMETIC_FLAWS: &[&str] = &["flaw.vengeful_powers"];

/// D68.3's per-entry shape, table-driven, for X3a's 55 Virtues. RED now for
/// every entry: nothing in the shipped catalogue carries `trained: true` yet
/// (`tmp/x3-scope.md`: "`PointItem::trained`... is `false` everywhere").
#[test]
fn x3a_virtues_carry_trained_and_the_expected_gate() {
    let rs = load_ruleset();
    for (id, gate) in X3A_VIRTUES {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must exist in the shipped catalogue"));
        assert!(item.trained, "{id}: must carry trained: true (D12/D68.1)");
        let has_gate = item
            .prerequisites
            .as_ref()
            .is_some_and(|p| contains_gate(p, *gate));
        assert!(
            has_gate,
            "{id}: must carry a {gate:?} prerequisite leaf (D68.2), got {:?}",
            item.prerequisites
        );
    }
}

/// The two X3a Virtues among the five House gates X5's findings state on
/// Hermetic entries, folded into X3 (F-35, F-136): each needs its own House
/// leaf ALONGSIDE the `order_member` gate, in the established
/// `all[order_member, house(x)]` shape (`flaw.primogeniture_lineage`'s
/// existing pattern).
#[test]
fn x3a_folded_house_gates_carry_their_house_leaf() {
    let rs = load_ruleset();
    for (id, house) in [
        ("virtue.clan_ilfetu", "house.bjornaer"), // ArMDE:3565 "within House Bjornaer"
        ("virtue.leper_magus", "house.tytalus"),  // ArMDE:4251 "House Tytalus"
    ] {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must exist"));
        let has_house = item
            .prerequisites
            .as_ref()
            .is_some_and(|p| contains_house(p, &Id::new(house)));
        assert!(
            has_house,
            "{id}: must carry a House({house}) leaf alongside order_member, got {:?}",
            item.prerequisites
        );
    }
}

/// D68.3's per-entry shape, table-driven, for X3b's 33 Flaws. RED now for
/// every entry: nothing in the shipped catalogue carries `trained: true` for
/// these yet (X3a's Phase 2 only touched the 55 Virtues).
#[test]
fn x3b_flaws_carry_trained_and_the_expected_gate() {
    let rs = load_ruleset();
    for (id, gate) in X3B_FLAWS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must exist in the shipped catalogue"));
        assert!(item.trained, "{id}: must carry trained: true (D12/D68.1)");
        let has_gate = item
            .prerequisites
            .as_ref()
            .is_some_and(|p| contains_gate(p, *gate));
        assert!(
            has_gate,
            "{id}: must carry a {gate:?} prerequisite leaf (D68.2), got {:?}",
            item.prerequisites
        );
    }
}

/// The two X3b Flaws among the five House gates X5's findings state on
/// Hermetic entries, folded into X3 (F-359, F-390): each needs its own House
/// leaf ALONGSIDE the `order_member` gate, in the established
/// `all[order_member, house(x)]` shape.
#[test]
fn x3b_folded_house_gates_carry_their_house_leaf() {
    let rs = load_ruleset();
    for (id, house) in [
        ("flaw.brutal_artist", "house.jerbiton"), // ArMDE:5759 "magi of House Jerbiton"
        ("flaw.consumed_casting_tools", "house.verditius"), // ArMDE:5841 "Verditius magi"
    ] {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must exist"));
        let has_house = item
            .prerequisites
            .as_ref()
            .is_some_and(|p| contains_house(p, &Id::new(house)));
        assert!(
            has_house,
            "{id}: must carry a House({house}) leaf alongside order_member, got {:?}",
            item.prerequisites
        );
    }
}

/// D68.3's per-entry shape, table-driven, for X3c's 31 Flaws. RED now for
/// every entry: nothing in the shipped catalogue carries `trained: true` for
/// these yet (X3a/X3b's Phase 2 only touched the 55 Virtues and 33 Flaws).
#[test]
fn x3c_flaws_carry_trained_and_the_expected_gate() {
    let rs = load_ruleset();
    for (id, gate) in X3C_FLAWS {
        let item = rs
            .item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must exist in the shipped catalogue"));
        assert!(item.trained, "{id}: must carry trained: true (D12/D68.1)");
        let has_gate = item
            .prerequisites
            .as_ref()
            .is_some_and(|p| contains_gate(p, *gate));
        assert!(
            has_gate,
            "{id}: must carry a {gate:?} prerequisite leaf (D68.2), got {:?}",
            item.prerequisites
        );
    }
}

/// The one X3c Flaw among the five House gates X5's findings state on
/// Hermetic entries, folded into X3 (F-518): it needs its own House leaf
/// ALONGSIDE the `order_member` gate, in the established
/// `all[order_member, house(x)]` shape.
#[test]
fn x3c_folded_house_gate_carries_its_house_leaf() {
    let rs = load_ruleset();
    let id = "flaw.spontaneous_casting_tools";
    let house = "house.verditius"; // ArMDE:6781 "may only be taken by Verditius magi"
    let item = rs
        .item(&Id::new(id))
        .unwrap_or_else(|| panic!("{id} must exist"));
    let has_house = item
        .prerequisites
        .as_ref()
        .is_some_and(|p| contains_house(p, &Id::new(house)));
    assert!(
        has_house,
        "{id}: must carry a House({house}) leaf alongside order_member, got {:?}",
        item.prerequisites
    );
}

/// D68.3's whole-catalogue classification: every `hermetic`-category entry
/// lands in exactly one of five buckets — the three Gift entries (neither
/// trained nor gated), X3a's 55 Virtues, X3b's 33 Flaws, X3c's 31 Flaws (all
/// three Flaw/Virtue buckets fully compliant — RED today until each slice's
/// Phase 2 lands), or the now-always-empty pending list (kept honest: any
/// future id staged there must still be non-compliant). An entry in none of
/// the five is a data-shape the test suite has never seen and fails loudly
/// rather than silently passing (CLAUDE.md's catalogue-size invariant: no
/// total is ever asserted here).
#[test]
fn hermetic_catalogue_is_fully_classified_by_x3a_or_pending() {
    let rs = load_ruleset();
    let x3a_ids: std::collections::BTreeSet<&str> = X3A_VIRTUES.iter().map(|(id, _)| *id).collect();
    let x3b_ids: std::collections::BTreeSet<&str> = X3B_FLAWS.iter().map(|(id, _)| *id).collect();
    let x3c_ids: std::collections::BTreeSet<&str> = X3C_FLAWS.iter().map(|(id, _)| *id).collect();
    let pending_ids: std::collections::BTreeSet<&str> =
        PENDING_HERMETIC_FLAWS.iter().copied().collect();

    for item in rs.items_by_category("hermetic") {
        let id = item.id.as_str();
        if GIFT_ENTRIES.contains(&id) {
            assert!(
                !item.trained && !item.prerequisites.as_ref().is_some_and(contains_any_gate),
                "{id}: one of the three intrinsic Gift entries must carry neither \
                 trained nor a gate"
            );
        } else if let Some((_, gate)) = X3A_VIRTUES.iter().find(|(vid, _)| *vid == id) {
            assert!(item.trained, "{id}: X3a virtue must carry trained: true");
            assert!(
                item.prerequisites
                    .as_ref()
                    .is_some_and(|p| contains_gate(p, *gate)),
                "{id}: X3a virtue must carry its expected gate, got {:?}",
                item.prerequisites
            );
        } else if let Some((_, gate)) = X3B_FLAWS.iter().find(|(fid, _)| *fid == id) {
            assert!(item.trained, "{id}: X3b flaw must carry trained: true");
            assert!(
                item.prerequisites
                    .as_ref()
                    .is_some_and(|p| contains_gate(p, *gate)),
                "{id}: X3b flaw must carry its expected gate, got {:?}",
                item.prerequisites
            );
        } else if let Some((_, gate)) = X3C_FLAWS.iter().find(|(fid, _)| *fid == id) {
            assert!(item.trained, "{id}: X3c flaw must carry trained: true");
            assert!(
                item.prerequisites
                    .as_ref()
                    .is_some_and(|p| contains_gate(p, *gate)),
                "{id}: X3c flaw must carry its expected gate, got {:?}",
                item.prerequisites
            );
        } else if pending_ids.contains(id) {
            let compliant =
                item.trained && item.prerequisites.as_ref().is_some_and(contains_any_gate);
            assert!(
                !compliant,
                "{id}: appears on the pending list but is already trained+gated — remove it \
                 from PENDING_HERMETIC_FLAWS"
            );
        } else {
            panic!(
                "{id}: unclassified hermetic-category entry — not one of the three Gift \
                 entries, not in X3A_VIRTUES, not in X3B_FLAWS, not in X3C_FLAWS, not in \
                 PENDING_HERMETIC_FLAWS. Classify it (this guards the catalogue-size \
                 invariant: a newly added hermetic entry must not be silently skipped)."
            );
        }
    }

    // Sanity: the constant buckets above account for every Virtue/Flaw named
    // in X3A_VIRTUES/X3B_FLAWS/X3C_FLAWS/PENDING_HERMETIC_FLAWS — a typo'd id
    // in any list would otherwise just never match anything in the loop above
    // and silently pass.
    for id in x3a_ids {
        rs.item(&Id::new(id))
            .unwrap_or_else(|| panic!("X3A_VIRTUES: {id} must exist in the shipped catalogue"));
    }
    for id in x3b_ids {
        rs.item(&Id::new(id))
            .unwrap_or_else(|| panic!("X3B_FLAWS: {id} must exist in the shipped catalogue"));
    }
    for id in x3c_ids {
        rs.item(&Id::new(id))
            .unwrap_or_else(|| panic!("X3C_FLAWS: {id} must exist in the shipped catalogue"));
    }
    for id in pending_ids {
        rs.item(&Id::new(id)).unwrap_or_else(|| {
            panic!("PENDING_HERMETIC_FLAWS: {id} must exist in the shipped catalogue")
        });
    }
}
