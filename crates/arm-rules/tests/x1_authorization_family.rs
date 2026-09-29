//! X1 (`docs/vf-audit/phase-2-plan.md` row X1) — data-integrity tests for the
//! authorization family: every Virtue/Flaw whose rulebook passage grants
//! permission to buy a gated (academic/arcane/martial) Ability category or a
//! specific gated Ability, but whose shipped `rules/core/virtues_flaws.json`
//! entry carries no `ability_authorization` effect (or, for four Educated
//! variants, does not exist at all).
//!
//! Phase 1 only: these tests assert the TARGET shape of the shipped data and
//! are expected to be RED until Phase 2's data pass lands. See
//! `docs/vf-audit/corrections.md` § 3.2/§ 3.2a, `docs/vf-audit/decisions.md`
//! D14/D43, `docs/open-todos.md` rows 45/48/50, and `tmp/x1-verdicts.md` for
//! the full per-entry citation and rationale — this file intentionally does
//! not re-derive that reasoning inline, to stay a plain data-shape check.
//!
//! Per CLAUDE.md's catalogue-size invariant, nothing here asserts a catalogue
//! total; every assertion is per-entry presence/shape.

use arm_rules::AbilityCategory;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationIssue, ValidationResult, validate};

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

/// Builds a character entity of `type_id` with the given selections, at the
/// current schema version and empty trait data — same shape as
/// `data_integrity.rs::entity`, duplicated here since integration test binaries
/// cannot share private helpers.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn cat(c: AbilityCategory) -> CategoryRef {
    CategoryRef::Bare(c)
}

fn ab(id: &str) -> AbilityRef {
    AbilityRef::Bare(Id::new(id))
}

/// `ability.dead_language` restricted to Latin — the D14/D59 catalogued-value
/// shape every existing carrier (`flaw.covenant_upbringing`, `virtue.custos`)
/// already uses.
fn latin() -> AbilityRef {
    AbilityRef::Scoped {
        ability: Id::new("ability.dead_language"),
        instance: Some(ParamValue::Literal {
            literal: "language.latin".into(),
        }),
        gate: None,
    }
}

fn auth(abilities: Vec<AbilityRef>, categories: Vec<CategoryRef>) -> Effect {
    Effect::AbilityAuthorization {
        abilities,
        categories,
    }
}

fn score(ability: &str, value: u8) -> AbilityScore {
    AbilityScore {
        ability: Id::new(ability),
        score: value,
        specialty: None,
        parameter: None,
    }
}

fn issue_codes(result: &ValidationResult) -> Vec<&str> {
    result.issues.iter().map(|i| i.code.as_str()).collect()
}

/// § 3.2 primary `auth` findings + the 27 `class+auth` findings, minus the four
/// entries already fixed (custos/wise_one/templar_specialist/student_of_realm —
/// verified directly against the shipped data, see `tmp/x1-verdicts.md`) and
/// minus `flaw.church_upbringing` (blocked on a D43/D13 design decision, not a
/// data edit — see `d43_pool_does_not_authorize_unlimited_general_spending`
/// below and the verdict table).
///
/// Each entry: the exact `Effect::AbilityAuthorization` the shipped
/// `effects` list must contain (in ADDITION to whatever else it already
/// carries, for entries that already have other effects).
#[test]
fn authorization_family_entries_carry_the_stated_permission() {
    let rs = load_ruleset();
    let table: Vec<(&str, Effect)> = vec![
        // --- row 48 / primary auth ---
        (
            "virtue.berserk",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.blood_of_the_nephilim",
            auth(vec![ab("ability.dominion_lore")], vec![]),
        ),
        (
            "virtue.demonic_blood",
            auth(vec![ab("ability.infernal_lore")], vec![]),
        ),
        (
            "virtue.educated",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.failed_apprentice",
            auth(
                vec![],
                vec![
                    cat(AbilityCategory::Academic),
                    cat(AbilityCategory::Arcane),
                    cat(AbilityCategory::Martial),
                ],
            ),
        ),
        (
            "virtue.fidai",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.faerie_blood",
            auth(vec![ab("ability.faerie_lore")], vec![]),
        ),
        (
            "virtue.familiarity_with_the_fae",
            auth(vec![ab("ability.faerie_lore")], vec![]),
        ),
        (
            "virtue.guild_dean",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.guild_master",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.knight",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.lasiq",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.lupus_the_wolf",
            auth(vec![ab("ability.artes_liberales"), latin()], vec![]),
        ),
        (
            "virtue.magical_blood",
            auth(vec![ab("ability.magic_lore")], vec![]),
        ),
        (
            "virtue.mamluk",
            auth(
                vec![ab("ability.theology_islam")],
                vec![cat(AbilityCategory::Martial)],
            ),
        ),
        (
            "virtue.marshal",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.master_bard",
            auth(vec![], vec![cat(AbilityCategory::Arcane)]),
        ),
        (
            "virtue.master_of_form_creatures",
            auth(vec![ab("ability.magic_lore")], vec![]),
        ),
        (
            "virtue.master_of_kennels",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.mazdean_priest",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.mendicant_friar",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.mercenary_captain",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.redcap",
            auth(
                vec![],
                vec![
                    cat(AbilityCategory::Academic),
                    cat(AbilityCategory::Arcane),
                    cat(AbilityCategory::Martial),
                ],
            ),
        ),
        (
            "virtue.rosh_beth_din",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.senior_bard",
            auth(
                vec![
                    ab("ability.dominion_lore"),
                    ab("ability.faerie_lore"),
                    ab("ability.infernal_lore"),
                    ab("ability.magic_lore"),
                ],
                vec![],
            ),
        ),
        (
            "virtue.senior_clergy",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.simple_student",
            auth(vec![ab("ability.artes_liberales"), latin()], vec![]),
        ),
        (
            "virtue.strong_faerie_blood",
            auth(vec![ab("ability.faerie_lore")], vec![]),
        ),
        (
            "virtue.town_magistrate",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.university_grammar_teacher",
            auth(vec![ab("ability.artes_liberales"), latin()], vec![]),
        ),
        (
            "virtue.venditor",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "flaw.diabolic_past",
            auth(vec![ab("ability.infernal_lore")], vec![]),
        ),
        (
            "flaw.faerie_friend",
            auth(vec![ab("ability.faerie_lore")], vec![]),
        ),
        (
            "flaw.faerie_upbringing",
            auth(vec![ab("ability.faerie_lore")], vec![]),
        ),
        (
            "flaw.failed_monk",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "flaw.monstrous_blood",
            auth(vec![ab("ability.magic_lore")], vec![]),
        ),
        (
            "flaw.outlaw",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "flaw.outlaw_leader",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "flaw.warped_by_magic",
            auth(vec![ab("ability.magic_lore")], vec![]),
        ),
        // --- class+auth (27) ---
        (
            "virtue.alim",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.almogaten",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.almogavar",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.archieunuch",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.beadle",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.brother_chaplain",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.brother_knight",
            auth(
                vec![],
                vec![
                    cat(AbilityCategory::Academic),
                    cat(AbilityCategory::Martial),
                ],
            ),
        ),
        (
            "virtue.brother_sergeant",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.bureaucrat",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.clerk",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.eunuch",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.jurist",
            auth(
                vec![
                    ab("ability.artes_liberales"),
                    ab("ability.civil_and_canon_law"),
                    latin(),
                ],
                vec![],
            ),
        ),
        (
            "virtue.notary",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.perfectus",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.prestigious_student",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.priest",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.religious",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.senior_master",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.sufi",
            auth(
                vec![
                    ab("ability.dominion_lore"),
                    ab("ability.islamic_law"),
                    ab("ability.theology_islam"),
                ],
                vec![],
            ),
        ),
        (
            "virtue.templar_administrator",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        // turb_trained: only the martial half is asserted here — the
        // dead-language half needs a new player-chosen `language` parameter
        // (the book leaves the language open, unlike custos's fixed Latin),
        // which is a Phase-2 design choice, not asserted structurally here.
        (
            "virtue.turb_trained",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "virtue.troubadour",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "flaw.branded_criminal",
            auth(vec![], vec![cat(AbilityCategory::Martial)]),
        ),
        (
            "flaw.imagined_folk_tradition_vulnerability",
            auth(vec![ab("ability.faerie_lore")], vec![]),
        ),
        (
            "flaw.magical_fascination",
            auth(
                vec![ab("ability.faerie_lore"), ab("ability.magic_lore")],
                vec![],
            ),
        ),
        (
            "flaw.pagan",
            auth(
                vec![ab("ability.faerie_lore"), ab("ability.magic_lore")],
                vec![],
            ),
        ),
    ];

    for (id, expected) in table {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must exist in the shipped catalogue"));
        assert!(
            item.effects.contains(&expected),
            "{id}: expected effects to contain {expected:?}, got {:?}",
            item.effects
        );
    }
}

/// `virtue.folk_magic` (ArMDE:3907-3920): "he may learn this Ability [the
/// chosen (Realm) Lore] at Character Creation even if he is normally unable
/// to take Arcane Abilities" — same D14-shape-2 binding `virtue.student_of_realm`
/// already uses (`realm` param → one gated Lore Ability per realm value).
#[test]
fn folk_magic_authorizes_only_the_chosen_realms_lore() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("virtue.folk_magic"))
        .expect("virtue.folk_magic must exist");
    let expected = Effect::AbilityAuthorization {
        abilities: vec![
            AbilityRef::Scoped {
                ability: Id::new("ability.dominion_lore"),
                instance: None,
                gate: Some(ParamGate {
                    param: "realm".into(),
                    equals: Id::new("realm.divine"),
                }),
            },
            AbilityRef::Scoped {
                ability: Id::new("ability.faerie_lore"),
                instance: None,
                gate: Some(ParamGate {
                    param: "realm".into(),
                    equals: Id::new("realm.faerie"),
                }),
            },
            AbilityRef::Scoped {
                ability: Id::new("ability.infernal_lore"),
                instance: None,
                gate: Some(ParamGate {
                    param: "realm".into(),
                    equals: Id::new("realm.infernal"),
                }),
            },
            AbilityRef::Scoped {
                ability: Id::new("ability.magic_lore"),
                instance: None,
                gate: Some(ParamGate {
                    param: "realm".into(),
                    equals: Id::new("realm.magic"),
                }),
            },
        ],
        categories: vec![],
    };
    assert!(
        item.effects.contains(&expected),
        "virtue.folk_magic: expected a realm-gated ability_authorization, got {:?}",
        item.effects
    );
}

/// row 50(b): `virtue.well_traveled` (ArMDE:5239-5242) ships `narrative` with
/// **no effects at all** today. Its fifty bonus XP fund only `general`
/// Abilities, so no `ability_authorization` is needed — only the missing
/// `restricted_ability_xp` pool and the reclassification to `creation_effect`
/// (D46: classification follows what is computed).
#[test]
fn well_traveled_carries_its_fifty_point_pool() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("virtue.well_traveled"))
        .expect("virtue.well_traveled must exist");
    assert_eq!(
        item.classification,
        Classification::CreationEffect,
        "virtue.well_traveled must be creation_effect once its XP pool is computed"
    );
    let expected = Effect::RestrictedAbilityXp {
        amount: 50,
        abilities: vec![
            Id::new("ability.area_lore"),
            Id::new("ability.bargain"),
            Id::new("ability.carouse"),
            Id::new("ability.charm"),
            Id::new("ability.etiquette"),
            Id::new("ability.folk_ken"),
            Id::new("ability.guile"),
            Id::new("ability.living_language"),
        ],
        categories: vec![],
        instances: vec![],
        from_normal_budget: false,
    };
    assert!(
        item.effects.contains(&expected),
        "virtue.well_traveled: expected the 50-point pool, got {:?}",
        item.effects
    );
}

/// `flaw.church_upbringing` (ArMDE:5789-5792): "The player must spend 25
/// experience points FROM THE NORMAL BUDGET on Artes Liberales, Latin, Music,
/// Organization Lore: Church, or Theology." D13's `from_normal_budget: true`
/// (already built — see `types.rs::Effect::RestrictedAbilityXp` and
/// `effective/xp.rs::general_pool_and_bonus`) is exactly the earmark shape
/// this passage needs, which the original audit (batch-11, F-387) rated
/// `uncomputed_rule`/unencodable BEFORE that field existed. `tmp/x1-verdicts.md`
/// records the residual "no other XP on Academic Abilities" bar as the same
/// D43 amount-tracking gap `privileged_upbringing`/`hermetic_experience` have —
/// not blocked here, since the earmark itself is now buildable.
#[test]
fn church_upbringing_earmarks_from_the_normal_budget() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.church_upbringing"))
        .expect("flaw.church_upbringing must exist");
    assert_eq!(
        item.classification,
        Classification::CreationEffect,
        "flaw.church_upbringing must be creation_effect now the earmark is computable"
    );
    let expected = Effect::RestrictedAbilityXp {
        amount: 25,
        abilities: vec![
            Id::new("ability.artes_liberales"),
            Id::new("ability.music"),
            Id::new("ability.theology_christian"),
        ],
        categories: vec![],
        instances: vec![
            latin(),
            AbilityRef::Scoped {
                ability: Id::new("ability.organization_lore"),
                instance: Some(ParamValue::Literal {
                    literal: "organization.church".into(),
                }),
                gate: None,
            },
        ],
        from_normal_budget: true,
    };
    assert!(
        item.effects.contains(&expected),
        "flaw.church_upbringing: expected the 25-point normal-budget earmark, got {:?}",
        item.effects
    );
}

/// § 3.2a: the four missing `#### Educated (...)` headings
/// (ArMDE:3715-3717/:3719-3721/:3723-3725/:3727-3729) are not parameter
/// variants — each is its own distinct Virtue, entirely absent from the
/// catalogue today. Phase 1 asserts only existence + shape (minor, general,
/// creation_effect) plus the authorization each variant's own passage states;
/// the exact `restricted_ability_xp` pool shape for each is Phase-2 authoring
/// (see `tmp/x1-verdicts.md`).
///
/// `virtue.educated_bardic` is excluded here — ArMDE:3715-3717 carries no
/// "may purchase Academic Abilities" sentence at all, unlike its three
/// siblings, so under D43 it gets NO `ability_authorization`: its pool alone
/// (pool-implied permission) is what legalizes owning Art of Memory, up to
/// what the pool funds. See `educated_bardic_carries_no_authorization_beyond_its_pool`.
#[test]
fn educated_family_is_complete() {
    let rs = load_ruleset();

    // Base Educated already exists; only Islamic/Hebrew/Vernacular are asserted
    // here (Bardic is its own test, see above).
    let new_variants = [
        (
            "virtue.educated_islamic",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.educated_hebrew",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
        (
            "virtue.educated_vernacular",
            auth(vec![], vec![cat(AbilityCategory::Academic)]),
        ),
    ];
    for (id, expected) in new_variants {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} must exist — F-34's missing heading (§ 3.2a)"));
        assert_eq!(item.magnitude, Magnitude::Minor, "{id}: magnitude");
        assert!(
            item.categories.iter().any(|c| c == "general"),
            "{id}: must be General like every other Educated heading"
        );
        assert_eq!(
            item.classification,
            Classification::CreationEffect,
            "{id}: classification"
        );
        assert!(
            item.effects.contains(&expected),
            "{id}: expected effects to contain {expected:?}, got {:?}",
            item.effects
        );
    }
}

/// `virtue.educated_bardic` (ArMDE:3715-3717): only a 50-point pool naming
/// Art of Memory/Profession: Storyteller/Profession: Poet/Area Lore/
/// Organization Lore — no broader "may purchase Academic Abilities" sentence.
/// D43 (coordinator correction, 2026-09-29): a pool with no separately-stated
/// permission gets pool-implied permission ONLY, never an
/// `Effect::AbilityAuthorization`. Existence must still stay legal for
/// whatever the pool itself can fund (Art of Memory up to 50 points); spending
/// beyond the pool must NOT be legalized.
#[test]
fn educated_bardic_carries_no_authorization_beyond_its_pool() {
    let rs = load_ruleset();
    // The entry does not exist yet (Phase 2 authors it) — this assertion is
    // what will need to hold once it does.
    if let Some(item) = rs.item(&Id::new("virtue.educated_bardic")) {
        assert!(
            !item
                .effects
                .iter()
                .any(|e| matches!(e, Effect::AbilityAuthorization { .. })),
            "virtue.educated_bardic: must carry NO ability_authorization (ArMDE:3715-3717 \
             states no broader permission), got {:?}",
            item.effects
        );
    } else {
        panic!(
            "virtue.educated_bardic must exist — F-34's missing heading (§ 3.2a); \
             once it exists, this test also asserts it carries no ability_authorization"
        );
    }
}

/// row 45: "`virtue.covenfolk`/`virtue.custos` additionally state 'may not
/// take the Wealthy Virtue or Poor Flaw', which `incompatible_with` already
/// expresses" (ArMDE:3611/:3631). Verified against the shipped data that
/// neither entry (nor the reverse edges on `virtue.wealthy`/`flaw.poor`)
/// carries this exclusion today.
#[test]
fn custos_and_covenfolk_exclude_wealthy_and_poor() {
    let rs = load_ruleset();
    for id in ["virtue.custos", "virtue.covenfolk"] {
        let item = rs.item(&Id::new(id)).expect("must exist");
        assert!(
            item.incompatible_with.contains(&Id::new("virtue.wealthy"))
                && item.incompatible_with.contains(&Id::new("flaw.poor")),
            "{id}: expected incompatible_with to name virtue.wealthy and flaw.poor, got {:?}",
            item.incompatible_with
        );
    }
    for id in ["virtue.wealthy", "flaw.poor"] {
        let item = rs.item(&Id::new(id)).expect("must exist");
        assert!(
            item.incompatible_with.contains(&Id::new("virtue.custos"))
                && item
                    .incompatible_with
                    .contains(&Id::new("virtue.covenfolk")),
            "{id}: symmetric edge back to custos/covenfolk missing, got {:?}",
            item.incompatible_with
        );
    }
}

// ---------------------------------------------------------------------------
// D43 — the design (coordinator correction, 2026-09-29):
//
// `effective/xp.rs::build_capacity_matrix` gives the GENERAL pool an
// unconditional edge to every spend ("the general pool can fund any spend").
// D43 narrows that: the GENERAL edge exists only for a spend the character may
// fund from general XP — an ungated ability, or one covered by EXPLICIT
// permission (an `AbilityAuthorization`, a free grant, the hermetically-
// trained exemption — whatever `validate_ability_authorization` accepts today
// OTHER than pool-implied permission). A spend only a restricted pool permits
// gets POOL edges only, so demand above the pool surfaces as the ordinary XP
// shortfall (`not_enough_xp`), never `ability_category_requires_virtue` —
// existence stays legal either way (`ability_authorizations` keeps returning
// both the explicit and the pool-implied sets for the ownership check;
// `magus_later_life_pool` must read the explicit set only).
//
// Every ruleset below mirrors a REAL shipped entry's exact effect shape (not
// an invented one), on the same precedent `validation/authorization.rs`'s own
// private unit tests use: no `life_stages` block, so `entity.xp_pool` is the
// general pool directly, set large enough (1000) that a shortfall can only be
// explained by the category gate, never by ordinary insufficient budget.
// ---------------------------------------------------------------------------

const D43_TYPES: &str = r#"[
  { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
    "permitted_categories": ["general", "personality", "social_status", "special"],
    "creation_phases": [] }
]"#;

/// (a) `virtue.privileged_upbringing` (ArMDE:4806-4808): "You may not...buy
/// Academic or Martial Abilities with your normal pool of experience points
/// unless you have another Virtue or Flaw permitting that." A 75-xp Artes
/// Liberales score (score 5) cannot be funded by the 50-point pool alone, so
/// the shortfall must be `not_enough_xp` — TODAY it is legal outright (the
/// general pool funds the remaining 25 unconditionally), which is the bug.
#[test]
fn d43_privileged_upbringing_refuses_spending_beyond_its_pool() {
    let rs = privileged_upbringing_rs();
    let mut companion = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.privileged_upbringing"))],
    );
    companion.xp_pool = 1000;
    companion
        .ability_scores
        .push(score("ability.artes_liberales", 5));

    let result = validate(&companion, &rs);
    let codes = issue_codes(&result);
    assert!(
        !codes.contains(&ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE),
        "D43: owning the Ability must stay legal (pool-implied permission covers existence) \
         — issues: {:?}",
        result.issues
    );
    assert!(
        codes.contains(&ValidationIssue::CODE_NOT_ENOUGH_XP),
        "D43: a category-scoped pool (Privileged Upbringing) must not fund spending beyond \
         its own 50-point capacity from general XP — issues: {:?}",
        result.issues
    );
}

/// (b) Positive control: an Artes Liberales score fully inside the pool's
/// 50-point capacity (score 3 = 30xp) must stay legal, today and after D43.
#[test]
fn d43_privileged_upbringing_permits_spending_within_its_pool() {
    let rs = privileged_upbringing_rs();
    let mut companion = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.privileged_upbringing"))],
    );
    companion.xp_pool = 1000;
    companion
        .ability_scores
        .push(score("ability.artes_liberales", 3));

    let result = validate(&companion, &rs);
    let codes = issue_codes(&result);
    assert!(
        !codes.contains(&ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE)
            && !codes.contains(&ValidationIssue::CODE_NOT_ENOUGH_XP),
        "a spend fully inside the pool's own capacity must be legal — issues: {:?}",
        result.issues
    );
}

/// (c) The same 75-xp spend, but the character ALSO holds an Educated-shaped
/// Virtue granting an explicit, unconditional `ability_authorization` for the
/// `academic` category. Explicit permission must open the general edge, so
/// the full 75 is fundable and nothing is refused.
#[test]
fn d43_explicit_authorization_permits_spending_beyond_the_pool() {
    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.privileged_upbringing", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [{ "type": "restricted_ability_xp", "amount": 50,
          "categories": ["general", "academic", "martial"] }] },
      { "id": "virtue.educated_shaped", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [{ "type": "ability_authorization", "categories": ["academic"] }] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
        { "score": 5, "total_xp": 75 }, { "score": 6, "total_xp": 105 },
        { "score": 7, "total_xp": 140 }, { "score": 8, "total_xp": 180 },
        { "score": 9, "total_xp": 225 }, { "score": 10, "total_xp": 275 }
      ],
      "categories_requiring_virtue": ["academic", "arcane", "martial"],
      "abilities": [ { "id": "ability.artes_liberales", "category": "academic" } ]
    }"#;
    let rs = Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: ITEMS,
        type_profiles: D43_TYPES,
        abilities: Some(ABILITIES),
        ..RulesetSources::default()
    })
    .unwrap();

    let mut companion = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.privileged_upbringing")),
            Selection::new(Id::new("virtue.educated_shaped")),
        ],
    );
    companion.xp_pool = 1000;
    companion
        .ability_scores
        .push(score("ability.artes_liberales", 5));

    let result = validate(&companion, &rs);
    let codes = issue_codes(&result);
    assert!(
        !codes.contains(&ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE)
            && !codes.contains(&ValidationIssue::CODE_NOT_ENOUGH_XP),
        "an explicit AbilityAuthorization must open the general edge for the full spend \
         — issues: {:?}",
        result.issues
    );
}

/// (d) `virtue.hermetic_experience`'s id-form pool (ArMDE:4063-4066): "You
/// cannot spend other experience points on Magic Lore or Latin unless the
/// character has another Virtue or Flaw permitting this." Same shape as (a),
/// on the id-form (not category-form) pool — a 75-xp Magic Lore score (score
/// 5) exceeds the 50-point pool.
#[test]
fn d43_hermetic_experience_refuses_spending_beyond_its_pool() {
    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.hermetic_experience", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [{ "type": "restricted_ability_xp", "amount": 50,
          "abilities": ["ability.magic_lore"],
          "instances": [
            { "ability": "ability.dead_language", "instance": { "literal": "language.latin" } },
            { "ability": "ability.organization_lore", "instance": { "literal": "organization.order_of_hermes" } }
          ] }] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
        { "score": 5, "total_xp": 75 }, { "score": 6, "total_xp": 105 },
        { "score": 7, "total_xp": 140 }, { "score": 8, "total_xp": 180 },
        { "score": 9, "total_xp": 225 }, { "score": 10, "total_xp": 275 }
      ],
      "categories_requiring_virtue": ["academic", "arcane", "martial"],
      "abilities": [
        { "id": "ability.magic_lore", "category": "arcane" },
        { "id": "ability.dead_language", "category": "academic", "parameter": "language" },
        { "id": "ability.organization_lore", "category": "general", "parameter": "organization" }
      ]
    }"#;
    let rs = Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: ITEMS,
        type_profiles: D43_TYPES,
        abilities: Some(ABILITIES),
        ..RulesetSources::default()
    })
    .unwrap();

    let mut companion = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.hermetic_experience"))],
    );
    companion.xp_pool = 1000;
    companion
        .ability_scores
        .push(score("ability.magic_lore", 5));

    let result = validate(&companion, &rs);
    let codes = issue_codes(&result);
    assert!(
        !codes.contains(&ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE),
        "D43: owning the Ability must stay legal — issues: {:?}",
        result.issues
    );
    assert!(
        codes.contains(&ValidationIssue::CODE_NOT_ENOUGH_XP),
        "D43: an id-form pool (Hermetic Experience) must not fund spending beyond its own \
         50-point capacity from general XP — issues: {:?}",
        result.issues
    );
}

/// `flaw.church_upbringing`'s "no other XP on Academic Abilities" (ArMDE:5789-
/// 5792) falls out of the same change: its 25-point earmark
/// (`from_normal_budget: true`) names Theology among its targets; a 50-xp
/// Theology: Christian score (score 4) exceeds it by 25.
#[test]
fn d43_church_upbringing_refuses_spending_beyond_its_earmark() {
    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "flaw.church_upbringing", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"],
        "effects": [{ "type": "restricted_ability_xp", "amount": 25,
          "abilities": ["ability.artes_liberales", "ability.music", "ability.theology_christian"],
          "instances": [
            { "ability": "ability.dead_language", "instance": { "literal": "language.latin" } },
            { "ability": "ability.organization_lore", "instance": { "literal": "organization.church" } }
          ],
          "from_normal_budget": true }] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
        { "score": 5, "total_xp": 75 }, { "score": 6, "total_xp": 105 },
        { "score": 7, "total_xp": 140 }, { "score": 8, "total_xp": 180 },
        { "score": 9, "total_xp": 225 }, { "score": 10, "total_xp": 275 }
      ],
      "categories_requiring_virtue": ["academic", "arcane", "martial"],
      "abilities": [
        { "id": "ability.theology_christian", "category": "academic" },
        { "id": "ability.artes_liberales", "category": "academic" },
        { "id": "ability.music", "category": "general" },
        { "id": "ability.dead_language", "category": "academic", "parameter": "language" },
        { "id": "ability.organization_lore", "category": "general", "parameter": "organization" }
      ]
    }"#;
    let rs = Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: ITEMS,
        type_profiles: D43_TYPES,
        abilities: Some(ABILITIES),
        ..RulesetSources::default()
    })
    .unwrap();

    let mut companion = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.church_upbringing"))],
    );
    companion.xp_pool = 1000;
    companion
        .ability_scores
        .push(score("ability.theology_christian", 4));

    let result = validate(&companion, &rs);
    let codes = issue_codes(&result);
    assert!(
        !codes.contains(&ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE),
        "D43: owning the Ability must stay legal — issues: {:?}",
        result.issues
    );
    assert!(
        codes.contains(&ValidationIssue::CODE_NOT_ENOUGH_XP),
        "D43: Church Upbringing's earmark must not fund spending beyond its own 25-point \
         capacity from general XP — issues: {:?}",
        result.issues
    );
}

/// `virtue.educated_bardic`'s pool (see `educated_bardic_carries_no_authorization_beyond_its_pool`)
/// funds Art of Memory up to 50 points and nothing more — no separate
/// permission sentence exists to authorize spending beyond it.
#[test]
fn d43_educated_bardic_refuses_spending_beyond_its_pool() {
    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.educated_bardic", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [{ "type": "restricted_ability_xp", "amount": 50,
          "abilities": ["ability.art_of_memory"] }] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
        { "score": 5, "total_xp": 75 }, { "score": 6, "total_xp": 105 },
        { "score": 7, "total_xp": 140 }, { "score": 8, "total_xp": 180 },
        { "score": 9, "total_xp": 225 }, { "score": 10, "total_xp": 275 }
      ],
      "categories_requiring_virtue": ["academic", "arcane", "martial"],
      "abilities": [ { "id": "ability.art_of_memory", "category": "academic" } ]
    }"#;
    let rs = Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: ITEMS,
        type_profiles: D43_TYPES,
        abilities: Some(ABILITIES),
        ..RulesetSources::default()
    })
    .unwrap();

    let mut companion = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.educated_bardic"))],
    );
    companion.xp_pool = 1000;
    companion
        .ability_scores
        .push(score("ability.art_of_memory", 5));

    let result = validate(&companion, &rs);
    let codes = issue_codes(&result);
    assert!(
        !codes.contains(&ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE),
        "D43: owning the Ability must stay legal — issues: {:?}",
        result.issues
    );
    assert!(
        codes.contains(&ValidationIssue::CODE_NOT_ENOUGH_XP),
        "D43: Educated (Bardic)'s pool must not fund spending beyond its own 50-point \
         capacity from general XP, since no broader permission is stated — issues: {:?}",
        result.issues
    );
}

/// Shared ruleset for the Privileged Upbringing cases — mirrors the REAL
/// shipped entry's exact effect shape (`amount:50,
/// categories:["general","academic","martial"]`).
fn privileged_upbringing_rs() -> Ruleset {
    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.privileged_upbringing", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [{ "type": "restricted_ability_xp", "amount": 50,
          "categories": ["general", "academic", "martial"] }] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
        { "score": 5, "total_xp": 75 }, { "score": 6, "total_xp": 105 },
        { "score": 7, "total_xp": 140 }, { "score": 8, "total_xp": 180 },
        { "score": 9, "total_xp": 225 }, { "score": 10, "total_xp": 275 }
      ],
      "categories_requiring_virtue": ["academic", "arcane", "martial"],
      "abilities": [ { "id": "ability.artes_liberales", "category": "academic" } ]
    }"#;
    Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: ITEMS,
        type_profiles: D43_TYPES,
        abilities: Some(ABILITIES),
        ..RulesetSources::default()
    })
    .unwrap()
}
