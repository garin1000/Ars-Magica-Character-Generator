//! X6c (`tmp/x6-scope.md` §1 "L — label choices"; `tmp/x6c-verdicts.md`) —
//! the 26 label-only parameters plus the 3 text→enumerated retypes. Unlike
//! X6b's 16 "M" parameters, none of these drive a computed rule: D9 still
//! requires the choice to be recorded (for the sheet/export), so every entry
//! gets a required parameter, and the 3 retypes stop accepting free text
//! (Q-X6-4, D70: no migration, no schema bump).
//!
//! One behavioral test per entry, against the SHIPPED `rules/core/*.json`
//! (never a minimal in-test fixture) — same convention as
//! `x6b_parameter_data.rs`. Every test here is expected RED until Phase 2
//! lands the data; see `tmp/x6c-handover.md` for the verbatim run.
//!
//! **D80** (`docs/vf-audit/decisions.md`) settled the file's three open
//! questions after the first red checkpoint: `flaw.rector` takes NO
//! parameter at all (the required university Social Status Virtue already
//! decides faculty vs nation, ArMDE:6673); `flaw.demonic_familiar`'s role is
//! free TEXT, not enumerated (ArMDE:5930's list is only examples); and
//! `virtue.fidai`/`virtue.lasiq` get an OPTIONAL "cover social status" —
//! an [`ParameterDomain::Item`] reference narrowed to the `social_status`
//! category (ArMDE:4235: "pretending to have some other social status"),
//! via the new `ParameterDef::required: bool` field
//! (`d80_optional_parameter.rs` covers the field itself; red-first, already
//! green before this file's own data lands).

use std::collections::BTreeMap;

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(SHIPPED_HOUSES),
        mythic_types: Some(include_str!(
            "../../../rules/core/mythic_companion_types.json"
        )),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .unwrap()
}

/// Same shape as every other X-series integration test file's `entity`
/// helper (e.g. `x6b_parameter_data.rs`) — duplicated since integration test
/// binaries cannot share private helpers.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn sel_with(id: &str, params: BTreeMap<String, Id>) -> Selection {
    Selection::with_params(Id::new(id), params)
}

fn param(key: &str, value: &str) -> BTreeMap<String, Id> {
    BTreeMap::from([(key.to_string(), Id::new(value))])
}

fn issue_codes(entity: &Entity, ruleset: &Ruleset) -> Vec<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

/// Shared assertion for the 19 plain-text, required, label-only parameters:
/// omitting the key reports `missing_param`; filling it with arbitrary
/// non-empty text resolves cleanly (no `missing_param`, no
/// `unknown_param_value` — there is no catalogue to reject against).
fn assert_required_text_param(item_id: &str, key: &str, sample_value: &str) {
    let rs = load_ruleset();

    let missing = entity("companion", vec![sel(item_id)]);
    assert!(
        issue_codes(&missing, &rs)
            .iter()
            .any(|c| c == "missing_param"),
        "{item_id}: an old save with no `{key}` must report missing_param (D9/D70 — \
         every recorded choice is a required parameter)"
    );

    let filled = entity(
        "companion",
        vec![sel_with(item_id, param(key, sample_value))],
    );
    let codes = issue_codes(&filled, &rs);
    assert!(
        !codes
            .iter()
            .any(|c| c == "missing_param" || c == "unknown_param_value"),
        "{item_id}: a filled `{key}` must resolve cleanly, got {codes:?}"
    );
}

// --- Text-only label parameters (17), plus D80.3's two optional cover-status ---

#[test]
fn greater_purifying_touch_requires_a_disease() {
    assert_required_text_param(
        "virtue.greater_purifying_touch",
        "disease",
        "the falling sickness",
    );
}

#[test]
fn lesser_purifying_touch_requires_an_illness() {
    assert_required_text_param("virtue.lesser_purifying_touch", "illness", "the ague");
}

#[test]
fn lesser_immunity_requires_a_hazard() {
    assert_required_text_param("virtue.lesser_immunity", "hazard", "drowning");
}

#[test]
fn troupe_upbringing_requires_an_area() {
    assert_required_text_param(
        "virtue.troupe_upbringing",
        "area",
        "tumbling and acrobatics",
    );
}

#[test]
fn focus_power_requires_a_focus() {
    assert_required_text_param("virtue.focus_power", "focus", "finding lost things");
}

/// Every issue code that judges an item's copies against each other: the
/// whole-tuple repeat (`max_per_target`), the total (`max_total`), and the
/// one-key repeat (`max_per_value`). Try-out finding 25 slipped past a guard
/// that checked only the first of them.
const REPEAT_CODES: [&str; 3] = [
    "duplicate_selection",
    "too_many_selections",
    "too_many_for_param_value",
];

fn repeat_findings(codes: &[String]) -> Vec<&String> {
    codes
        .iter()
        .filter(|c| REPEAT_CODES.contains(&c.as_str()))
        .collect()
}

/// ArMDE:3903: "This Virtue may be taken more than once, and the points
/// gained may be combined", so a second copy for the SAME focus is legal
/// (D81.9: copies may share a scope). No repeat check of ANY code may object —
/// finding 25 was `too_many_for_param_value`, which the old form of this guard
/// (checking `duplicate_selection` alone) never looked for.
#[test]
fn focus_power_may_be_taken_twice_for_the_same_focus() {
    let rs = load_ruleset();
    let twice = entity(
        "companion",
        vec![
            sel_with("virtue.focus_power", param("focus", "finding lost things")),
            sel_with("virtue.focus_power", param("focus", "finding lost things")),
        ],
    );
    let codes = issue_codes(&twice, &rs);
    assert!(
        repeat_findings(&codes).is_empty(),
        "two Focus Power copies on one focus combine their points (ArMDE:3903), got {codes:?}"
    );
}

/// The same two copies spelled with different case and padding are still one
/// focus (free text compares case- and whitespace-insensitively), and still
/// legal to hold twice.
#[test]
fn focus_power_twice_for_one_focus_spelled_differently_is_still_legal() {
    let rs = load_ruleset();
    let twice = entity(
        "companion",
        vec![
            sel_with("virtue.focus_power", param("focus", "Fire")),
            sel_with("virtue.focus_power", param("focus", " fire  ")),
        ],
    );
    let codes = issue_codes(&twice, &rs);
    assert!(
        repeat_findings(&codes).is_empty(),
        "\"Fire\" and \" fire  \" are one focus, which may be taken twice, got {codes:?}"
    );
}

#[test]
fn baneful_circumstances_requires_a_circumstance() {
    assert_required_text_param(
        "flaw.baneful_circumstances",
        "circumstance",
        "touching the ground",
    );
}

#[test]
fn deleterious_circumstances_requires_a_circumstance() {
    assert_required_text_param("flaw.deleterious_circumstances", "circumstance", "wet");
}

#[test]
fn environmental_magic_condition_requires_a_condition() {
    assert_required_text_param(
        "flaw.environmental_magic_condition",
        "condition",
        "whenever he is inside",
    );
}

#[test]
fn environmental_sensitivity_requires_a_feature() {
    assert_required_text_param("flaw.environmental_sensitivity", "feature", "salt water");
}

#[test]
fn restriction_requires_a_condition() {
    assert_required_text_param("flaw.restriction", "condition", "no beard");
}

#[test]
fn necessary_condition_requires_an_action() {
    assert_required_text_param(
        "flaw.necessary_condition",
        "action",
        "spinning around three times",
    );
}

#[test]
fn supernatural_nuisance_requires_a_kind() {
    assert_required_text_param("flaw.supernatural_nuisance", "kind", "ghosts");
}

#[test]
fn poor_memory_requires_a_kind() {
    assert_required_text_param("flaw.poor_memory", "kind", "names");
}

#[test]
fn lycanthrope_requires_a_predator() {
    assert_required_text_param("flaw.lycanthrope", "predator", "wolf");
}

#[test]
fn paid_rights_requires_a_right() {
    assert_required_text_param("virtue.paid_rights", "right", "hold a fief in her own name");
}

/// D80.3: Fida'i/Lasiq's "cover social status" is the ONE exception to
/// D70's "every new parameter is required" — it applies only while the
/// character is away from home on a mission (ArMDE:4235), so it must NEVER
/// report `missing_param`. Modeled as an [`ParameterDomain::Item`]
/// reference narrowed to the `social_status` category (ArMDE:3881, :4235:
/// "pretending to have some other social status, which you should choose"),
/// not free text — the book names an actual Social Status Virtue/Flaw, and
/// `require_categories` already narrows exactly that without inventing a
/// new mechanism. Exercises the new `ParameterDef::required: bool` field
/// (`d80_optional_parameter.rs`), already green before this data lands.
fn assert_optional_cover_status_param(item_id: &str, key: &str) {
    let rs = load_ruleset();

    let bare = entity("companion", vec![sel(item_id)]);
    assert!(
        !issue_codes(&bare, &rs).iter().any(|c| c == "missing_param"),
        "{item_id}: D80.3 — an unfilled `{key}` must NEVER report missing_param"
    );

    let social_status = entity(
        "companion",
        vec![sel_with(item_id, param(key, "virtue.journeyman"))],
    );
    let codes = issue_codes(&social_status, &rs);
    assert!(
        !codes
            .iter()
            .any(|c| c == "missing_param" || c == "unknown_param_value"),
        "{item_id}: a Social Status item named for `{key}` must resolve cleanly, got {codes:?}"
    );

    let not_social_status = entity(
        "companion",
        vec![sel_with(item_id, param(key, "virtue.focus_power"))],
    );
    assert!(
        issue_codes(&not_social_status, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "{item_id}: `{key}` must be narrowed to the social_status category — \
         virtue.focus_power (supernatural) must not resolve"
    );
}

#[test]
fn fidai_has_an_optional_cover_social_status() {
    assert_optional_cover_status_param("virtue.fidai", "cover");
}

#[test]
fn lasiq_has_an_optional_cover_social_status() {
    assert_optional_cover_status_param("virtue.lasiq", "cover");
}

#[test]
fn templar_office_holder_requires_a_position() {
    assert_required_text_param("virtue.templar_office_holder", "position", "marshal");
}

/// Unlike the other 18, Curse of Slander ALREADY carries a `taken_as`
/// category parameter (`rules/core/virtues_flaws.json:604-611`); X6c adds a
/// second, independent `section` text parameter alongside it — the existing
/// `taken_as` must keep resolving once `section` is added.
#[test]
fn curse_of_slander_requires_a_section_and_keeps_its_taken_as() {
    let rs = load_ruleset();

    let missing_section = entity(
        "companion",
        vec![sel_with(
            "flaw.curse_of_slander",
            param("taken_as", "general"),
        )],
    );
    assert!(
        issue_codes(&missing_section, &rs)
            .iter()
            .any(|c| c == "missing_param"),
        "ArMDE:5882: the slander is centered on one specific section of mundane \
         society — an old save naming only `taken_as` must now also report \
         missing_param for `section`"
    );

    let both = entity(
        "companion",
        vec![sel_with(
            "flaw.curse_of_slander",
            BTreeMap::from([
                ("taken_as".to_string(), Id::new("general")),
                ("section".to_string(), Id::new("the nobility")),
            ]),
        )],
    );
    let codes = issue_codes(&both, &rs);
    assert!(
        !codes
            .iter()
            .any(|c| c == "missing_param" || c == "unknown_param_value"),
        "both `taken_as` and `section` filled must resolve cleanly, got {codes:?}"
    );
}

// --- Enumerated label parameters (4) ----------------------------------------
// (D80 moves flaw.rector to "no parameter" and flaw.demonic_familiar to the
// text-only group above; both tests for them sit just after this section,
// next to cyclic_magic_positive, where they were originally written.)

/// ArMDE:4253-4274: the sidebar names exactly four examples, but the body
/// text says only that the effect "should be comparable to other Minor
/// Virtues" — the four are illustrative, not exhaustive, so this is E+custom.
#[test]
fn lesser_benediction_resolves_its_four_examples_and_a_custom_branch() {
    let rs = load_ruleset();

    for value in [
        "benediction.gift_of_the_gab",
        "benediction.green_fingers",
        "benediction.pricking_thumbs",
        "benediction.unusually_fecund",
        "benediction.custom",
    ] {
        let e = entity(
            "companion",
            vec![sel_with(
                "virtue.lesser_benediction",
                param("benediction", value),
            )],
        );
        let codes = issue_codes(&e, &rs);
        assert!(
            !codes.iter().any(|c| c == "unknown_param_value"),
            "{value} must resolve against virtue.lesser_benediction's enumerated \
             `benediction` parameter, got {codes:?}"
        );
    }

    let bogus = entity(
        "companion",
        vec![sel_with(
            "virtue.lesser_benediction",
            param("benediction", "benediction.second_sight"),
        )],
    );
    assert!(
        issue_codes(&bogus, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "an unlisted benediction must not silently resolve"
    );

    let missing = entity("companion", vec![sel("virtue.lesser_benediction")]);
    assert!(
        issue_codes(&missing, &rs)
            .iter()
            .any(|c| c == "missing_param"),
        "an old save with no `benediction` must report missing_param"
    );
}

/// ArMDE:4109: "A player who selects this Virtue... needs to select which
/// form of the Virtue his character has" — exactly two forms are described
/// (the forgettable-average character, and the one who distracts with a
/// striking prop), no third option or troupe discretion is offered, so this
/// is closed Enumerated, not E+custom.
#[test]
fn indescribable_face_resolves_exactly_two_forms() {
    let rs = load_ruleset();

    for value in ["form.forgettable", "form.distracting_prop"] {
        let e = entity(
            "companion",
            vec![sel_with("virtue.indescribable_face", param("form", value))],
        );
        assert!(
            !issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "unknown_param_value"),
            "{value} must resolve against virtue.indescribable_face's `form` parameter"
        );
    }

    let bogus = entity(
        "companion",
        vec![sel_with(
            "virtue.indescribable_face",
            param("form", "form.invisible"),
        )],
    );
    assert!(
        issue_codes(&bogus, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "ArMDE:4107-4114 describes exactly two forms; a third must not resolve"
    );
}

/// ArMDE:3380-3383: "'Alim ... may be a minor official ... or ... a major
/// figure" — the named examples (mu'adhdhin, imam, mufti, qadi) are
/// introduced with "such as", so only the two-way rank axis is closed.
#[test]
fn alim_resolves_minor_official_and_major_figure_only() {
    let rs = load_ruleset();

    for value in ["rank.minor_official", "rank.major_figure"] {
        let e = entity(
            "companion",
            vec![sel_with("virtue.alim", param("rank", value))],
        );
        assert!(
            !issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "unknown_param_value"),
            "{value} must resolve against virtue.alim's `rank` parameter"
        );
    }

    let missing = entity("companion", vec![sel("virtue.alim")]);
    assert!(
        issue_codes(&missing, &rs)
            .iter()
            .any(|c| c == "missing_param"),
        "an old save with no `rank` must report missing_param"
    );
}

/// D80.1 (drops the earlier red): ArMDE:6673 — "the representative leader
/// of his faculty or nation at a university, depending on whether he is a
/// master or a student. ... The character must have a Social Status Virtue
/// dictating his place within the university." That required prerequisite
/// Virtue already decides faculty vs nation, so Rector/Proctor takes NO
/// parameter of its own — verify-only, pinning that this stays true. D67:
/// its `prerequisites` constraint is itself engine-enforced, so
/// `creation_effect` (unchanged) is still the right classification with no
/// parameter at all.
#[test]
fn rector_takes_no_parameter_and_its_description_states_the_rule() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.rector"))
        .expect("flaw.rector must ship");
    assert!(
        item.parameters.is_empty(),
        "D80.1: Rector/Proctor takes no parameter — the required Social \
         Status Virtue prerequisite already decides faculty vs nation \
         (ArMDE:6673), got {:?}",
        item.parameters
    );

    let en: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/i18n/en/virtues_flaws.json")).unwrap();
    let de: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/i18n/de/virtues_flaws.json")).unwrap();
    for (lang, doc) in [("en", &en), ("de", &de)] {
        let text = doc["flaw.rector"]["summary"]
            .as_str()
            .or_else(|| doc["flaw.rector"]["description"].as_str())
            .unwrap_or("");
        let states_master_or_student =
            text.to_lowercase().contains("student") || text.contains("Student");
        assert!(
            states_master_or_student,
            "{lang}/flaw.rector must state ArMDE:6673's master-or-student rule; \
             text was: {text:?}"
        );
    }
}

/// D80.2 (replaces the earlier red): "Demonic Familiar's role is free text.
/// The book's list is only examples" (ArMDE:5930: "at the storyguide's
/// discretion... SUCH AS a warder, teacher, or paramour" — open-ended, not a
/// closed enumeration). Folded into the same required-text shape every other
/// label-only entry uses.
#[test]
fn demonic_familiar_requires_a_role_description() {
    assert_required_text_param("flaw.demonic_familiar", "role", "a warder");
}

/// ArMDE:3635-3638: "attuned to some cycle of nature (solar, lunar, or
/// seasonal, for example)" — mirrors `flaw.cyclic_magic_negative`'s own
/// shipped `cycle` parameter (ArMDE:5893-5896, `rules/core/virtues_flaws.json`),
/// which already resolves the same three values; the Virtue currently ships
/// with NO parameter at all.
#[test]
fn cyclic_magic_positive_resolves_the_same_three_cycles_as_its_flaw_twin() {
    let rs = load_ruleset();

    for value in ["cycle.solar", "cycle.lunar", "cycle.seasonal"] {
        let e = entity(
            "companion",
            vec![sel_with(
                "virtue.cyclic_magic_positive",
                param("cycle", value),
            )],
        );
        assert!(
            !issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "unknown_param_value"),
            "{value} must resolve against virtue.cyclic_magic_positive's `cycle` \
             parameter, exactly as it already does for flaw.cyclic_magic_negative"
        );
    }

    let missing = entity("companion", vec![sel("virtue.cyclic_magic_positive")]);
    assert!(
        issue_codes(&missing, &rs)
            .iter()
            .any(|c| c == "missing_param"),
        "an old save with no `cycle` must report missing_param"
    );
}

/// Verify-only (`tmp/x6c-verdicts.md`: "already done at HEAD") — NOT part of
/// the red count: `flaw.cyclic_magic_negative`'s `cycle` parameter already
/// ships. This pins that it stays green across X6c's data changes elsewhere
/// in the same file.
#[test]
fn cyclic_magic_negative_cycle_parameter_already_resolves() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![sel_with(
            "flaw.cyclic_magic_negative",
            param("cycle", "cycle.seasonal"),
        )],
    );
    assert!(
        !issue_codes(&e, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "flaw.cyclic_magic_negative's cycle parameter was already shipped before X6c"
    );
}

// --- Retypes: text → enumerated (3) -----------------------------------------

/// F-03: ArMDE:3362-3367, :7310 — "one of the seven subjects of Artes
/// Liberales" (Trivium: Grammar, Logic, Rhetoric; Quadrivium: Arithmetic,
/// Geometry, Astronomy, Music). Today `subject` is `domain: text`
/// (`rules/core/virtues_flaws.json:3685`).
///
/// **Previously BLOCKED** (`tmp/x6c-handover.md`'s "A discovered, out-of-scope
/// blocker" section): the SAME parameter is read by
/// `Effect::AbilityRollModParam { param: "subject", .. }`
/// (`types.rs::Effect::AbilityRollModParam`), whose declared parameter
/// `ruleset::integrity.rs::validate_effect_refs` hardcoded to
/// `ParameterDomain::Text` — so a ruleset giving `subject` `domain: enumerated`
/// failed to LOAD at all. `tmp/ac-handover.md`'s design note resolves this:
/// `ruleset::integrity.rs` accepts Text OR Enumerated for this effect's
/// parameter (see `x6c_academic_concentration_domain.rs`'s two reds), and the
/// UI resolves `SurfacedModifier::detail` through the SAME generic
/// rules-i18n-id-or-verbatim lookup every other selection parameter value
/// already uses (`derive.ts::selectionParamLabel`), rather than rendering it
/// unconditionally raw — so a free-text subject still shows verbatim and an
/// enumerated one resolves through its rules-i18n name, never as a bare slug
/// (`DerivedSurfacedModifiersSection.test.ts` covers this UI side). The other
/// two retypes below have no such consumer (confirmed: no effect anywhere
/// reads `faculty` or `being` by param key) and were unaffected by the
/// blocker.
#[test]
fn academic_concentration_subject_retypes_to_the_seven_liberal_arts() {
    let rs = load_ruleset();

    for value in [
        "subject.grammar",
        "subject.logic",
        "subject.rhetoric",
        "subject.arithmetic",
        "subject.geometry",
        "subject.astronomy",
        "subject.music",
    ] {
        let e = entity(
            "companion",
            vec![sel_with(
                "virtue.academic_concentration_subject",
                param("subject", value),
            )],
        );
        assert!(
            !issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "unknown_param_value"),
            "{value} must resolve as one of the seven Artes Liberales subjects"
        );
    }

    let old_free_text = entity(
        "companion",
        vec![sel_with(
            "virtue.academic_concentration_subject",
            param("subject", "Logic"),
        )],
    );
    assert!(
        issue_codes(&old_free_text, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "Q-X6-4/D70: an old save's free-text value ('Logic', not 'subject.logic') \
         must NOT be migrated — it reports unknown_param_value"
    );
}

/// F-07: ArMDE:3388-3395 — "one of three classes of beings: mundane animals,
/// faeries, or magical beings." `being` is `domain: text`
/// (`rules/core/virtues_flaws.json:3745`); retyped to enumerated, reusing the
/// SAME `being.animals`/`being.faeries`/`being.magical_creatures` values
/// already shipped for `flaw.offensive_to_beings` et al. — zero new i18n.
#[test]
fn alluring_to_beings_retypes_to_its_three_named_classes() {
    let rs = load_ruleset();

    for value in ["being.animals", "being.faeries", "being.magical_creatures"] {
        let e = entity(
            "companion",
            vec![sel_with("virtue.alluring_to_beings", param("being", value))],
        );
        assert!(
            !issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "unknown_param_value"),
            "{value} must resolve against virtue.alluring_to_beings's retyped \
             `being` parameter"
        );
    }

    // being.demons/being.divine/being.mundane_humans are used by sibling
    // entries (offensive_to_beings et al.) but are NOT among the three
    // classes ArMDE:3390 names for Alluring to (Beings).
    let wrong_sibling_value = entity(
        "companion",
        vec![sel_with(
            "virtue.alluring_to_beings",
            param("being", "being.demons"),
        )],
    );
    assert!(
        issue_codes(&wrong_sibling_value, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "ArMDE:3390 names only animals/faeries/magical beings; a sibling \
         entry's being.demons must not resolve here"
    );

    let old_free_text = entity(
        "companion",
        vec![sel_with(
            "virtue.alluring_to_beings",
            param("being", "faeries"),
        )],
    );
    assert!(
        issue_codes(&old_free_text, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "Q-X6-4/D70: an old save's free-text value ('faeries', not \
         'being.faeries') must NOT be migrated"
    );
}

/// F-57: ArMDE:3683-3692 — "graduated from one of the higher faculties of a
/// university, in medicine, civil or canon law, or theology." `faculty` is
/// `domain: text` (`rules/core/virtues_flaws.json:4348`); retyped to
/// enumerated over exactly those three, mirroring the shipped
/// `ability.civil_and_canon_law`/`ability.medicine`/`ability.theology_*`
/// academic abilities the book ties the faculty to.
#[test]
fn doctor_in_faculty_retypes_to_medicine_law_or_theology() {
    let rs = load_ruleset();

    for value in ["faculty.medicine", "faculty.law", "faculty.theology"] {
        let e = entity(
            "magus",
            vec![
                sel("virtue.the_gift"),
                sel("virtue.hermetic_magus"),
                sel_with("virtue.doctor_in_faculty", param("faculty", value)),
            ],
        );
        assert!(
            !issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "unknown_param_value"),
            "{value} must resolve against virtue.doctor_in_faculty's retyped \
             `faculty` parameter"
        );
    }

    let old_free_text = entity(
        "magus",
        vec![
            sel("virtue.the_gift"),
            sel("virtue.hermetic_magus"),
            sel_with(
                "virtue.doctor_in_faculty",
                param("faculty", "Theology: Christian"),
            ),
        ],
    );
    assert!(
        issue_codes(&old_free_text, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "Q-X6-4/D70: an old save's free-text value must NOT be migrated to \
         'faculty.theology'"
    );
}

// --- Fluent coverage: every NEW param-label key, both locales ---------------

/// Keys already shipped for an existing entry (`being`, `faculty`, `subject`,
/// `hazard`, `form`, `area`, `focus`, `cycle`, `circumstance`, `condition`,
/// `feature`, `role`, `rank`) are reused here and need no new Fluent work —
/// only genuinely new keys are checked. D80 drops `flaw.rector` entirely (no
/// parameter at all) and reuses the already-shipped `role` key, unchanged,
/// as `flaw.demonic_familiar`'s free-text key; `pretended_status` is
/// replaced by `cover` (D80.3's optional item-reference key).
#[test]
fn new_x6c_parameter_keys_have_fluent_labels_in_both_locales() {
    let en_ftl = include_str!("../../../locales/en/main.ftl");
    let de_ftl = include_str!("../../../locales/de/main.ftl");

    let new_keys = [
        "disease",
        "illness",
        "benediction",
        "action",
        "kind",
        "section",
        "predator",
        "right",
        "cover",
        "position",
    ];

    for key in new_keys {
        let needle = format!("param-label-{key} =");
        assert!(
            en_ftl.contains(&needle),
            "locales/en/main.ftl is missing `{needle}` (X6c label parameter)"
        );
        assert!(
            de_ftl.contains(&needle),
            "locales/de/main.ftl is missing `{needle}` (X6c label parameter)"
        );
    }
}

// --- Rules-i18n coverage: every NEW enumerated value, both locales ----------

fn rules_i18n_has_name(doc: &serde_json::Value, id: &str) -> bool {
    doc.get(id)
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.trim().is_empty())
}

#[test]
fn new_x6c_enumerated_values_have_rules_i18n_names_in_both_locales() {
    let en: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/i18n/en/virtues_flaws.json")).unwrap();
    let de: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/i18n/de/virtues_flaws.json")).unwrap();

    let new_values = [
        // academic_concentration_subject's retype (F-03): the seven Artes
        // Liberales subjects (Trivium + Quadrivium), ArMDE:3362-3367, :7310.
        "subject.grammar",
        "subject.logic",
        "subject.rhetoric",
        "subject.arithmetic",
        "subject.geometry",
        "subject.astronomy",
        "subject.music",
        // doctor_in_faculty's retype
        "faculty.medicine",
        "faculty.law",
        "faculty.theology",
        // lesser_benediction
        "benediction.gift_of_the_gab",
        "benediction.green_fingers",
        "benediction.pricking_thumbs",
        "benediction.unusually_fecund",
        "benediction.custom",
        // indescribable_face
        "form.forgettable",
        "form.distracting_prop",
        // alim
        "rank.minor_official",
        "rank.major_figure",
        // D80: flaw.rector takes no parameter, and flaw.demonic_familiar's
        // `role` is free text (ArMDE:5930) — neither has enumerated values.
    ];

    for id in new_values {
        assert!(
            rules_i18n_has_name(&en, id),
            "rules/i18n/en/virtues_flaws.json is missing a `name` for `{id}`"
        );
        assert!(
            rules_i18n_has_name(&de, id),
            "rules/i18n/de/virtues_flaws.json is missing a `name` for `{id}`"
        );
    }

    // alluring_to_beings's retype reuses being.animals/being.faeries/
    // being.magical_creatures, which already carry names in both locales —
    // pinned here so a future rename of those shared values is caught too.
    for id in ["being.animals", "being.faeries", "being.magical_creatures"] {
        assert!(rules_i18n_has_name(&en, id));
        assert!(rules_i18n_has_name(&de, id));
    }
}
