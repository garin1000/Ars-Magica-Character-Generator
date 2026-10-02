//! X5a (`tmp/x5-verdicts.md`; `docs/vf-audit/corrections.md` § 3.6;
//! `docs/vf-audit/decisions.md` D16, D38, D41, D51, D56, D68.2/.9/.10/.11) —
//! prerequisite and eligibility gates: the character-type-audience family
//! (`IsGrog`/`IsCompanion`), the three Order-audience Story Flaws (Tormenting
//! Master, Vendetta, Hermetic Patron), the two vacuous `has_category` leaves
//! (Rector, Male Guild Sponsor), D51's four worked instances (Mercurian
//! Magic, Leper Magus, Mythic Blood, A Deal with the Devil), and the two
//! Educated (Hebrew) prerequisites (Shamash, Sofer).
//!
//! GREEN, phase 2 landed. See `tmp/x5-verdicts.md` for the full per-entry
//! citation table and `tmp/x5-handover.md` for what phase 2 changed: the new
//! `Prereq::IsGrog` variant + `EntityTypeProfile::is_grog` (every match site,
//! plus the UI mirror and its parity test), the new open-V/F-grant machinery
//! (Mythic Blood's hereditary Minor Personality Flaw), and the data for every
//! entry below.
//!
//! `Prereq::IsGrog` is exercised only through `validate()` against the shipped
//! ruleset — data behaviour, never the enum directly — exactly as the brief
//! required.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationResult, validate};

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

/// Same shape as `x3_trained_gate.rs::entity` / `x4_incompatibilities.rs::entity`
/// — duplicated since integration test binaries cannot share private helpers.
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

fn sel_with_param(id: &str, key: &str, value: &str) -> Selection {
    let mut s = Selection::new(Id::new(id));
    s.params
        .insert(key.to_string(), SelectionParamValue::Single(Id::new(value)));
    s
}

fn issue_codes(result: &ValidationResult) -> Vec<&str> {
    result.issues.iter().map(|i| i.code.as_str()).collect()
}

// ---------------------------------------------------------------------------
// Group A — the character-type-audience family (D38, D68.9). F-24 needs only
// the already-built `IsCompanion`; the other three need the new `IsGrog`,
// which is why they are tested through `validate()`'s current (gap) behaviour
// rather than by constructing `Prereq::IsGrog` in test code.
// ---------------------------------------------------------------------------

/// D68.9: "Grogs may not take this Virtue" (ArMDE:5139). Live gap today — no
/// `prerequisites` on `virtue.temporal_influence` at all.
#[test]
fn grog_is_refused_temporal_influence() {
    let rs = load_ruleset();
    let e = entity("grog", vec![sel("virtue.temporal_influence")]);
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"prereq_not_met"),
        "ArMDE:5139 'Grogs may not take this Virtue' — a grog taking Temporal \
         Influence must be refused, got: {:?}",
        issue_codes(&result)
    );
}

/// D68.9: "This Flaw may only be taken by grogs" (ArMDE:5747). A companion
/// must be refused; a grog stays legal.
#[test]
fn companion_is_refused_bound_to_role() {
    let rs = load_ruleset();
    let companion = entity(
        "companion",
        vec![sel_with_param(
            "flaw.bound_to_role_role",
            "role",
            "watchtower",
        )],
    );
    let companion_result = validate(&companion, &rs);
    assert!(
        issue_codes(&companion_result).contains(&"prereq_not_met"),
        "ArMDE:5747 'may only be taken by grogs' — a companion taking Bound \
         to Role must be refused, got: {:?}",
        issue_codes(&companion_result)
    );

    let grog = entity(
        "grog",
        vec![sel_with_param(
            "flaw.bound_to_role_role",
            "role",
            "watchtower",
        )],
    );
    let grog_result = validate(&grog, &rs);
    assert!(
        !issue_codes(&grog_result).contains(&"prereq_not_met"),
        "a grog taking Bound to Role must stay legal, got: {:?}",
        issue_codes(&grog_result)
    );
}

/// D68.9: "Grogs may not take this Flaw" (ArMDE:6548).
#[test]
fn grog_is_refused_outlaw_leader() {
    let rs = load_ruleset();
    let e = entity("grog", vec![sel("flaw.outlaw_leader")]);
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"prereq_not_met"),
        "ArMDE:6548 'Grogs may not take this Flaw' — a grog taking Outlaw \
         Leader must be refused, got: {:?}",
        issue_codes(&result)
    );
}

/// F-24/D38: "Magi and Grogs may not take this Virtue" (ArMDE:3517) — only
/// companions (and mythic companions) may. This is exactly `IsCompanion`,
/// already built (E1) — the gap here is purely that `virtue.blood_of_the_nephilim`
/// carries no `prerequisites` yet.
#[test]
fn blood_of_the_nephilim_is_refused_to_magus_and_grog_but_legal_for_companion() {
    let rs = load_ruleset();

    let magus = entity("magus", vec![sel("virtue.blood_of_the_nephilim")]);
    let magus_result = validate(&magus, &rs);
    assert!(
        issue_codes(&magus_result).contains(&"prereq_not_met"),
        "ArMDE:3517 'Magi... may not take this Virtue' — a magus must be \
         refused, got: {:?}",
        issue_codes(&magus_result)
    );

    let grog = entity("grog", vec![sel("virtue.blood_of_the_nephilim")]);
    let grog_result = validate(&grog, &rs);
    assert!(
        issue_codes(&grog_result).contains(&"prereq_not_met"),
        "ArMDE:3517 '...and Grogs may not take this Virtue' — a grog must be \
         refused, got: {:?}",
        issue_codes(&grog_result)
    );

    let companion = entity("companion", vec![sel("virtue.blood_of_the_nephilim")]);
    let companion_result = validate(&companion, &rs);
    assert!(
        !issue_codes(&companion_result).contains(&"prereq_not_met"),
        "a companion taking Blood of the Nephilim must stay legal, got: {:?}",
        issue_codes(&companion_result)
    );
}

// ---------------------------------------------------------------------------
// Group B — the three Order-audience Story Flaws (D68.2: `order_member`
// where the passage names the Order, a House, or the Gauntlet).
// ---------------------------------------------------------------------------

/// F-532: "only applicable to magi" (ArMDE:6851-6854, the Gauntlet is named
/// one sentence earlier) → `Prereq::OrderMember`.
#[test]
fn tormenting_master_requires_order_membership() {
    let rs = load_ruleset();
    let companion = entity("companion", vec![sel("flaw.tormenting_master")]);
    let companion_result = validate(&companion, &rs);
    assert!(
        issue_codes(&companion_result).contains(&"prereq_not_met"),
        "ArMDE:6851-6854 'only applicable to magi' — a companion must be \
         refused, got: {:?}",
        issue_codes(&companion_result)
    );

    let magus = entity("magus", vec![sel("flaw.tormenting_master")]);
    let magus_result = validate(&magus, &rs);
    assert!(
        !issue_codes(&magus_result).contains(&"prereq_not_met"),
        "a magus taking Tormenting Master must stay legal, got: {:?}",
        issue_codes(&magus_result)
    );
}

/// F-533: the House half (hedged, "generally restricted... to magi of House
/// Verditius") is ALREADY an `advisory_prerequisites` — SANITY, must not
/// regress. The magus half is unhedged and still missing its
/// `Prereq::OrderMember` `prerequisites` gate.
#[test]
fn vendetta_house_half_stays_advisory_and_magus_half_becomes_a_hard_gate() {
    let rs = load_ruleset();

    // SANITY: the House hedge is already a warning, not a hard block.
    let non_verditius_magus = entity("magus", vec![sel("flaw.vendetta")]);
    let result = validate(&non_verditius_magus, &rs);
    assert!(
        !issue_codes(&result).contains(&"prereq_not_met"),
        "the House Verditius half is hedged (D16) — must never be a hard \
         block, got: {:?}",
        issue_codes(&result)
    );

    // RED: the unhedged magus half has no gate yet.
    let companion = entity("companion", vec![sel("flaw.vendetta")]);
    let companion_result = validate(&companion, &rs);
    assert!(
        issue_codes(&companion_result).contains(&"prereq_not_met"),
        "ArMDE:6955-6958 restricts Vendetta to magi (unhedged) — a companion \
         must be refused, got: {:?}",
        issue_codes(&companion_result)
    );
}

/// F-448: "You must be a Redcap or magus to take this Flaw" (ArMDE:6250) —
/// `any[has(virtue.redcap), has(virtue.lone_redcap), order_member]`.
#[test]
fn hermetic_patron_requires_redcap_or_magus() {
    let rs = load_ruleset();

    let bare_companion = entity("companion", vec![sel("flaw.hermetic_patron")]);
    let bare_result = validate(&bare_companion, &rs);
    assert!(
        issue_codes(&bare_result).contains(&"prereq_not_met"),
        "ArMDE:6250 'You must be a Redcap or magus' — a companion with \
         neither must be refused, got: {:?}",
        issue_codes(&bare_result)
    );

    let redcap_companion = entity(
        "companion",
        vec![sel("flaw.hermetic_patron"), sel("virtue.redcap")],
    );
    let redcap_result = validate(&redcap_companion, &rs);
    assert!(
        !issue_codes(&redcap_result).contains(&"prereq_not_met"),
        "a Redcap taking Hermetic Patron must stay legal, got: {:?}",
        issue_codes(&redcap_result)
    );

    let magus = entity("magus", vec![sel("flaw.hermetic_patron")]);
    let magus_result = validate(&magus, &rs);
    assert!(
        !issue_codes(&magus_result).contains(&"prereq_not_met"),
        "a magus taking Hermetic Patron must stay legal, got: {:?}",
        issue_codes(&magus_result)
    );
}

// ---------------------------------------------------------------------------
// Group C — the two vacuous `has_category social_status` leaves (Q-X5-2):
// D41 already requires SOME Social Status universally, so the generic
// category leaf is satisfied by ANY status and gates nothing extra.
// ---------------------------------------------------------------------------

/// F-502: "a Social Status Virtue dictating his place within the university"
/// (ArMDE:6673) — a non-university Social Status must not satisfy this.
#[test]
fn rector_without_a_university_status_is_refused_and_with_one_is_legal() {
    let rs = load_ruleset();

    let non_university = entity(
        "companion",
        vec![sel("flaw.rector"), sel("virtue.merchant")],
    );
    let non_university_result = validate(&non_university, &rs);
    assert!(
        issue_codes(&non_university_result).contains(&"prereq_not_met"),
        "ArMDE:6673 names a status 'within the university' specifically — a \
         non-university Social Status (Merchant) must not satisfy it, got: {:?}",
        issue_codes(&non_university_result)
    );

    let university = entity("companion", vec![sel("flaw.rector"), sel("virtue.beadle")]);
    let university_result = validate(&university, &rs);
    assert!(
        !issue_codes(&university_result).contains(&"prereq_not_met"),
        "Beadle (ArMDE:3480-3483, 'employed by a university') must satisfy \
         Rector's requirement, got: {:?}",
        issue_codes(&university_result)
    );
}

/// D41: "must select a separate guild Social Status Virtue" (ArMDE:4441) — a
/// non-guild Social Status must not satisfy this.
#[test]
fn male_guild_sponsor_without_a_guild_status_is_refused_and_with_one_is_legal() {
    let rs = load_ruleset();

    let non_guild = entity(
        "companion",
        vec![sel("virtue.male_guild_sponsor"), sel("virtue.merchant")],
    );
    let non_guild_result = validate(&non_guild, &rs);
    assert!(
        issue_codes(&non_guild_result).contains(&"prereq_not_met"),
        "ArMDE:4441 names a SEPARATE GUILD Social Status specifically — a \
         non-guild Social Status (Merchant) must not satisfy it, got: {:?}",
        issue_codes(&non_guild_result)
    );

    let guild = entity(
        "companion",
        vec![
            sel("virtue.male_guild_sponsor"),
            sel("virtue.guild_apprentice"),
        ],
    );
    let guild_result = validate(&guild, &rs);
    assert!(
        !issue_codes(&guild_result).contains(&"prereq_not_met"),
        "Guild Apprentice must satisfy Male Guild Sponsor's requirement, \
         got: {:?}",
        issue_codes(&guild_result)
    );
}

// ---------------------------------------------------------------------------
// Group D — D51's four worked instances.
// ---------------------------------------------------------------------------

/// D51 row 1 ("also have" → prerequisite): ArMDE:4522 "All known members of
/// the Mercurian lineage also have the Minor Flaw Ceremonial Spontaneous
/// Magic."
#[test]
fn mercurian_magic_requires_ceremonial_spontaneous_magic() {
    let rs = load_ruleset();

    let without_flaw = entity("magus", vec![sel("virtue.mercurian_magic")]);
    let without_result = validate(&without_flaw, &rs);
    assert!(
        issue_codes(&without_result).contains(&"prereq_not_met"),
        "ArMDE:4522 — Mercurian Magic without Ceremonial Spontaneous Magic \
         must be refused, got: {:?}",
        issue_codes(&without_result)
    );

    let with_flaw = entity(
        "magus",
        vec![
            sel("virtue.mercurian_magic"),
            sel("flaw.ceremonial_spontaneous_magic"),
        ],
    );
    let with_result = validate(&with_flaw, &rs);
    assert!(
        !issue_codes(&with_result).contains(&"prereq_not_met"),
        "Mercurian Magic + Ceremonial Spontaneous Magic must stay legal, \
         got: {:?}",
        issue_codes(&with_result)
    );
}

/// D51 row 3 ("can only be bought if... also has the Leprosy Flaw" →
/// prerequisite): ArMDE:4249-4252. The House Tytalus gate is ALREADY landed
/// (X3) — SANITY, must not regress.
#[test]
fn leper_magus_requires_leprosy() {
    let rs = load_ruleset();

    // SANITY: the already-shipped House gate. `virtue.the_gift` +
    // `virtue.hermetic_magus` satisfy the magus profile's OWN required trait
    // so its unmet prerequisite (the Gift) does not pollute the codes below
    // with unrelated `prereq_not_met` noise.
    let mut wrong_house = entity(
        "magus",
        vec![
            sel("virtue.leper_magus"),
            sel("flaw.leprosy"),
            sel("virtue.the_gift"),
            sel("virtue.hermetic_magus"),
        ],
    );
    wrong_house.house = Some(Id::new("house.flambeau"));
    let wrong_house_result = validate(&wrong_house, &rs);
    assert!(
        issue_codes(&wrong_house_result).contains(&"prereq_not_met"),
        "the House Tytalus gate must still hold for a different House, \
         got: {:?}",
        issue_codes(&wrong_house_result)
    );

    // RED: no Leprosy Flaw, but IS House Tytalus.
    let mut no_leprosy = entity(
        "magus",
        vec![
            sel("virtue.leper_magus"),
            sel("virtue.the_gift"),
            sel("virtue.hermetic_magus"),
        ],
    );
    no_leprosy.house = Some(Id::new("house.tytalus"));
    let no_leprosy_result = validate(&no_leprosy, &rs);
    assert!(
        issue_codes(&no_leprosy_result).contains(&"prereq_not_met"),
        "ArMDE:4251 'can only be bought if the character also has the \
         Leprosy Flaw' — Leper Magus without Leprosy must be refused, got: {:?}",
        issue_codes(&no_leprosy_result)
    );
}

/// D51 row 3's grant half ("granting the Life Boost Minor Virtue" → grant,
/// not an inlined effect): the entry must carry `grants_selection` naming
/// `virtue.life_boost`, not merely duplicate its `special_casting_mod` effect
/// inline.
#[test]
fn leper_magus_grants_life_boost_via_grants_selection() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("virtue.leper_magus"))
        .expect("virtue.leper_magus must exist in the shipped catalogue");
    let grants_life_boost = item.effects.iter().any(|e| {
        matches!(
            e,
            Effect::GrantsSelection { items, .. } if items.contains(&Id::new("virtue.life_boost"))
        )
    });
    assert!(
        grants_life_boost,
        "ArMDE:4251-4252 'granting the Life Boost Minor Virtue' — \
         virtue.leper_magus must carry a GrantsSelection naming \
         virtue.life_boost, not an inlined special_casting_mod duplicating it"
    );
}

/// D68.11: Mythic Blood's hereditary Minor Personality Flaw (ArMDE:4588,
/// "both at no extra cost") is an OPEN grant — no fixed id — that costs no
/// points but counts toward the Personality caps (ArMDE:2820).
#[test]
fn mythic_blood_open_flaw_grant_counts_toward_personality_cap() {
    let rs = load_ruleset();
    // Two bought Minor Personality Flaws already sit at the magus profile's
    // soft cap of 2 (character_types.json "magus" flaw_category_caps). A
    // THIRD, granted by Mythic Blood, must push the count over the cap and
    // raise the warning — today it raises nothing, because no open-grant
    // machinery exists to add the granted Flaw to the count at all.
    let e = entity(
        "magus",
        vec![
            sel("flaw.ambitious_minor"),
            sel("flaw.avaricious_minor"),
            sel_with_param("virtue.mythic_blood", "focus", "fire"),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"too_many_personality_flaws"),
        "ArMDE:2820/:4588 — Mythic Blood's granted hereditary Minor \
         Personality Flaw must count toward the Personality cap, got: {:?}",
        issue_codes(&result)
    );
}

/// SANITY, correcting `tmp/x3-scope.md`'s "not a question" note: ArMDE:4405
/// "only one Magical Focus... regardless of the source" is ALREADY enforced
/// today, and by the more general mechanism — `validate_magical_focus` counts
/// `Effect::MagicalFocus` across every bought+granted selection, and Mythic
/// Blood's bundled focus already carries that effect. No `incompatible_with`
/// addition is owed; adding one would be a second, redundant mechanism for
/// the same rule (the "one mechanism for two ideas" error D37 warns against).
/// This must stay green, not go red.
#[test]
fn mythic_blood_and_a_standalone_magical_focus_already_trip_the_one_focus_rule() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![
            sel_with_param("virtue.mythic_blood", "focus", "fire"),
            sel_with_param("virtue.minor_magical_focus", "focus", "water"),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"multiple_magical_foci"),
        "ArMDE:4405 'only one Magical Focus... regardless of the source' — \
         Mythic Blood + a standalone Magical Focus Virtue must already trip \
         validate_magical_focus's count>1 check, got: {:?}",
        issue_codes(&result)
    );
}

/// D51 row 4 ("includes the effects of" → effects only, never held): A Deal
/// with the Devil copies Plagued by Supernatural Entity's effects, but that
/// Flaw carries none — SANITY that this stays a no-op, confirming the
/// verdict that no engine work is owed here.
#[test]
fn plagued_by_supernatural_entity_has_no_effects_to_copy() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.plagued_by_supernatural_entity"))
        .expect("flaw.plagued_by_supernatural_entity must exist in the shipped catalogue");
    assert!(
        item.effects.is_empty(),
        "D51/tmp/x3-scope.md: A Deal with the Devil's 'includes the effects \
         of Plagued By Supernatural Entity' stays text-only because Plagued \
         has no effects to copy — if this now fails, Plagued gained effects \
         and A Deal with the Devil needs re-reading, not this test loosened"
    );
}

// ---------------------------------------------------------------------------
// Group E — F-270/F-283: add the Educated (Hebrew) prerequisite (the heading
// exists at ArMDE:3723 and the entry is already in the catalogue, X1).
// ---------------------------------------------------------------------------

#[test]
fn shamash_requires_educated_hebrew() {
    let rs = load_ruleset();

    let without = entity("companion", vec![sel("virtue.shamash")]);
    let without_result = validate(&without, &rs);
    assert!(
        issue_codes(&without_result).contains(&"prereq_not_met"),
        "ArMDE:4944 'the character must have the Educated (Hebrew) Virtue' — \
         Shamash without it must be refused, got: {:?}",
        issue_codes(&without_result)
    );

    let with = entity(
        "companion",
        vec![sel("virtue.shamash"), sel("virtue.educated_hebrew")],
    );
    let with_result = validate(&with, &rs);
    assert!(
        !issue_codes(&with_result).contains(&"prereq_not_met"),
        "Shamash + Educated (Hebrew) must stay legal, got: {:?}",
        issue_codes(&with_result)
    );
}

#[test]
fn sofer_requires_educated_hebrew() {
    let rs = load_ruleset();

    let without = entity("companion", vec![sel("virtue.sofer")]);
    let without_result = validate(&without, &rs);
    assert!(
        issue_codes(&without_result).contains(&"prereq_not_met"),
        "ArMDE:4996 'the character must have the Educated (Hebrew) Virtue' — \
         Sofer without it must be refused, got: {:?}",
        issue_codes(&without_result)
    );

    let with = entity(
        "companion",
        vec![sel("virtue.sofer"), sel("virtue.educated_hebrew")],
    );
    let with_result = validate(&with, &rs);
    assert!(
        !issue_codes(&with_result).contains(&"prereq_not_met"),
        "Sofer + Educated (Hebrew) must stay legal, got: {:?}",
        issue_codes(&with_result)
    );
}
