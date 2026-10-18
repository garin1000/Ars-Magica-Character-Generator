//! G2: the life timeline — a character's life as chronological spans, each with
//! what its years earned and how much of that the purchases fill. Display only:
//! derived from the life-stage budget and an already-solved `XpAllocation`.
//!
//! ArMDE:2434: "For grogs and companions they are acquired in two blocks: early
//! childhood, and later life. For magi, there are two more periods to consider:
//! apprenticeship, and life as a magus after that."
//! ArMDE:2505: "The fifteen years of apprenticeship give the character 240
//! experience points, and 120 levels of spells."
//! ArMDE:2541: "For every year, the magus gets 30 points. Each point can be an
//! experience point in an Art or Ability or one level of spell."
//! ArMDE:2552: "For each season that your magus spends working on a lab project,
//! the character loses 10 points from the yearly 30 experience points, to a
//! minimum of 0 if three or four seasons are spent on lab work."
//! ArMDE:2556, :2558 (Darius): eight years of study "gives him 240 experience
//! points", then a year in the laboratory, "so he gains no experience points in
//! that year".
//! ArMDE:4921 (Redcap): "You are trained in a similar manner to magi […] You have
//! spent fifteen years as an apprentice, and gained a total of 300 experience
//! points in those fifteen years. […] In addition, you have the Well-Traveled
//! Virtue (page 116) at no cost."
//! ArMDE:5716 (Abandoned Apprentice): "The character was abandoned by his master
//! midway through his Hermetic training."

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::{
    EntryCapacity, FixedSection, LabYears, LifeSection, LifeStageBlock, LifeStagePlan,
    LifeTimeline, NativeLanguage, TimelineEntry, TimelineEntryKind, XpAllocation, XpPoolOrigin,
    checked_xp_allocation, life_timeline,
};
use std::collections::BTreeMap;

fn shipped_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: Some(include_str!(
            "../../../rules/core/spell_mastery_abilities.json"
        )),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("shipped core ruleset loads")
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A character of `type_id` funded from its life stages, with life sections
/// switched on and a native language set, so childhood's two pools both exist.
fn character(type_id: &str, age: Option<u32>, selections: Vec<Selection>) -> Entity {
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.ability_funding = AbilityFunding::LifeStages;
    entity.age = age;
    entity.selections = selections;
    entity.life_stages = Some(LifeStagePlan {
        native_language: Some(NativeLanguage::from("Frankish")),
        life_sections_enabled: true,
        ..LifeStagePlan::default()
    });
    entity
}

fn plan(entity: &mut Entity) -> &mut LifeStagePlan {
    entity.life_stages.as_mut().expect("the fixture has a plan")
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn sel_with(id: &str, params: &[(&str, &str)]) -> Selection {
    Selection::with_params(
        Id::new(id),
        params
            .iter()
            .map(|&(key, value)| (key.to_string(), Id::new(value)))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn section(name: &str, years: u32) -> LifeSection {
    LifeSection {
        name: name.to_string(),
        years,
        lab_years: LabYears::default(),
    }
}

/// An Ability row at score 0 carrying `xp` banked experience: a spend of exactly
/// `xp`, so the fill numbers stay readable.
fn ability_xp(ability: &str, xp: u32) -> AbilityScore {
    let mut row = AbilityScore::new(Id::new(ability), 0);
    row.banked_xp = xp;
    row
}

/// An Art row at score 0 carrying `xp` banked experience (general pool only).
fn art_xp(art: &str, xp: u32) -> ArtScore {
    let mut row = ArtScore::new(Id::new(art), 0);
    row.banked_xp = xp;
    row
}

fn spells(ids: &[&str]) -> Vec<SpellSelection> {
    ids.iter()
        .map(|&id| SpellSelection::new(Id::new(id)))
        .collect()
}

fn allocation(entity: &Entity, ruleset: &Ruleset) -> XpAllocation {
    checked_xp_allocation(entity, ruleset).expect("the fixture is within the solve bound")
}

fn timeline(entity: &Entity, ruleset: &Ruleset) -> LifeTimeline {
    life_timeline(entity, ruleset, &allocation(entity, ruleset)).expect("a timeline")
}

fn fixed(section: FixedSection) -> TimelineEntryKind {
    TimelineEntryKind::Fixed { section }
}

fn named(index: usize) -> TimelineEntryKind {
    TimelineEntryKind::Named { index }
}

fn xp_only(xp: u32) -> EntryCapacity {
    EntryCapacity::Separate {
        xp,
        spell_levels: 0,
    }
}

fn points(points: u32) -> EntryCapacity {
    EntryCapacity::Shared { points }
}

fn entry(
    kind: TimelineEntryKind,
    (start_age, end_age): (u32, u32),
    capacity: EntryCapacity,
    filled_xp: u32,
    filled_spell_levels: u32,
) -> TimelineEntry {
    TimelineEntry {
        kind,
        start_age,
        end_age,
        years: end_age - start_age,
        capacity,
        filled_xp,
        filled_spell_levels,
    }
}

/// "75 experience points in their native language […] and 45 experience points
/// to divide" (ArMDE:2448) — childhood as one fixed span.
fn childhood(filled_xp: u32) -> TimelineEntry {
    entry(
        fixed(FixedSection::Childhood),
        (0, 5),
        xp_only(120),
        filled_xp,
        0,
    )
}

fn capacity_total(capacity: EntryCapacity) -> u32 {
    match capacity {
        EntryCapacity::Separate { xp, .. } => xp,
        EntryCapacity::Shared { points } => points,
    }
}

// ---------------------------------------------------------------------------
// Grog / companion: Childhood → named later-life sections → Remaining
// ---------------------------------------------------------------------------

/// A companion of 35 splits its 30 later-life years into three named sections
/// and leaves 10 as Remaining. Each span earns its years × 15 (ArMDE:2462), and
/// the general experience is poured into them oldest first, capped.
#[test]
fn a_companion_pours_later_life_xp_into_its_sections_oldest_first() {
    let rs = shipped_ruleset();
    let mut companion = character("companion", Some(35), vec![]);
    plan(&mut companion).life_sections = vec![
        section("Squire", 5),
        section("Crusade", 10),
        section("Monastery", 5),
    ];
    companion.ability_scores = vec![
        // Childhood's spread funds Athletics (ArMDE:2448).
        ability_xp("ability.athletics", 30),
        // Ride is not on the childhood list, so later life pays.
        ability_xp("ability.ride", 200),
    ];
    let solved = allocation(&companion, &rs);
    assert_eq!(solved.general_used, 200, "precondition: Ride is general");

    let timeline = life_timeline(&companion, &rs, &solved).expect("a timeline");

    assert_eq!(
        timeline.entries,
        vec![
            childhood(30),
            entry(named(0), (5, 10), xp_only(75), 75, 0),
            entry(named(1), (10, 20), xp_only(150), 125, 0),
            entry(named(2), (20, 25), xp_only(75), 0, 0),
            entry(TimelineEntryKind::Remaining, (25, 35), xp_only(150), 0, 0),
        ]
    );
    assert_eq!(timeline.overrun_years, 0);
    assert!(timeline.outside.is_empty(), "{:?}", timeline.outside);

    let budget = rs
        .life_stages()
        .and_then(|rules| rules.budget(&companion, &rs))
        .expect("a budget");
    let later_life: u32 = timeline.entries[1..]
        .iter()
        .map(|entry| capacity_total(entry.capacity))
        .sum();
    assert_eq!(later_life, budget.later_life_xp);
}

// ---------------------------------------------------------------------------
// Magus: Childhood → LaterLife → Apprenticeship → post-Gauntlet sections
// ---------------------------------------------------------------------------

/// Darius (ArMDE:2556, :2558): gauntleted at 25, eight years of study worth 240
/// points, then a year of four lab seasons worth nothing. 40 of the points are
/// taken as levels of spells.
fn darius(sections: Vec<LifeSection>, plan_lab_years: LabYears) -> Entity {
    let mut magus = character("magus", Some(34), vec![]);
    let plan = plan(&mut magus);
    plan.life_sections = sections;
    plan.post_gauntlet_lab_years = plan_lab_years;
    plan.post_gauntlet_spell_levels = 40;
    magus.ability_scores = vec![ability_xp("ability.ride", 40)];
    magus.art_scores = vec![art_xp("art.creo", 340)];
    // 40 + 35 + 25 + 25 + 20 + 15 = 160 levels.
    magus.spells = spells(&[
        "spell.black_whisper",
        "spell.ball_of_abysmal_flame",
        "spell.bane_of_the_decrepit_body",
        "spell.blessing_of_childlike_bliss",
        "spell.beast_of_minuscule_proportions",
        "spell.beast_of_outlandish_size",
    ]);
    magus
}

fn one_four_season_lab_year() -> LabYears {
    LabYears {
        four_seasons: 1,
        ..LabYears::default()
    }
}

/// The magus shape: later life ends where the apprenticeship begins (ArMDE:2284),
/// the apprenticeship holds 240 experience and 120 levels as two numbers
/// (ArMDE:2505), and each post-Gauntlet section earns years × 30 less its OWN lab
/// seasons (ArMDE:2541, :2552). Experience is poured first, oldest first; the
/// levels of spells then take the apprenticeship's 120 and the room the
/// experience left in the post-Gauntlet points.
#[test]
fn a_magus_timeline_runs_childhood_later_life_apprenticeship_then_sections() {
    let rs = shipped_ruleset();
    let mut lab = section("Lab", 1);
    lab.lab_years = one_four_season_lab_year();
    let magus = darius(vec![section("Study", 8), lab], LabYears::default());
    let solved = allocation(&magus, &rs);
    assert_eq!(solved.general_used, 340, "precondition: Creo is general");

    let timeline = life_timeline(&magus, &rs, &solved).expect("a timeline");

    assert_eq!(
        timeline.entries,
        vec![
            childhood(0),
            entry(fixed(FixedSection::LaterLife), (5, 10), xp_only(75), 40, 0),
            entry(
                fixed(FixedSection::Apprenticeship),
                (10, 25),
                EntryCapacity::Separate {
                    xp: 240,
                    spell_levels: 120,
                },
                240,
                120,
            ),
            entry(named(0), (25, 33), points(240), 100, 40),
            entry(named(1), (33, 34), points(0), 0, 0),
            entry(TimelineEntryKind::Remaining, (34, 34), points(0), 0, 0),
        ]
    );
    assert_eq!(timeline.overrun_years, 0);
}

/// Splitting Darius's nine post-Gauntlet years into sections moves no point: the
/// sections' capacities sum to the same 240 the unsplit years earn.
#[test]
fn darius_split_into_two_sections_earns_the_unsplit_points() {
    let rs = shipped_ruleset();
    let mut lab = section("Lab", 1);
    lab.lab_years = one_four_season_lab_year();
    let split = darius(vec![section("Study", 8), lab], LabYears::default());
    let unsplit = darius(vec![], one_four_season_lab_year());

    let post_gauntlet_points = |magus: &Entity| -> u32 {
        timeline(magus, &rs)
            .entries
            .iter()
            .filter(|entry| matches!(entry.capacity, EntryCapacity::Shared { .. }))
            .map(|entry| capacity_total(entry.capacity))
            .sum()
    };

    let budget = rs
        .life_stages()
        .and_then(|rules| rules.budget(&unsplit, &rs))
        .expect("a budget");
    assert_eq!(budget.post_gauntlet_points, 240, "precondition");
    assert_eq!(post_gauntlet_points(&unsplit), 240);
    assert_eq!(post_gauntlet_points(&split), 240);
}

/// Skilled Parens: "You gain an additional 60 experience points and 30 spell
/// levels during apprenticeship" (ArMDE:5039) — counted in the Apprenticeship
/// span (DECIDED 1).
#[test]
fn skilled_parens_widens_the_apprenticeship_span() {
    let rs = shipped_ruleset();
    let magus = character("magus", Some(25), vec![sel("virtue.skilled_parens")]);

    let timeline = timeline(&magus, &rs);

    let apprenticeship = timeline
        .entries
        .iter()
        .find(|entry| entry.kind == fixed(FixedSection::Apprenticeship))
        .expect("an apprenticeship span");
    assert_eq!(
        apprenticeship.capacity,
        EntryCapacity::Separate {
            xp: 300,
            spell_levels: 150,
        }
    );
}

/// A spell-levels override replaces the profile's 120 and nothing else: the
/// Apprenticeship span's levels are the effective base plus Parens (DECIDED 3).
#[test]
fn a_spell_levels_override_sets_the_apprenticeship_spell_capacity() {
    let rs = shipped_ruleset();
    let mut magus = character("magus", Some(25), vec![sel("virtue.skilled_parens")]);
    magus.spell_levels_override = Some(80);

    let timeline = timeline(&magus, &rs);

    let apprenticeship = timeline
        .entries
        .iter()
        .find(|entry| entry.kind == fixed(FixedSection::Apprenticeship))
        .expect("an apprenticeship span");
    assert_eq!(
        apprenticeship.capacity,
        EntryCapacity::Separate {
            xp: 300,
            spell_levels: 110,
        }
    );
}

// ---------------------------------------------------------------------------
// Redcap: the magus shape, anchored at the default Gauntlet age (25)
// ---------------------------------------------------------------------------

/// A Redcap of 30 (ArMDE:4921): later life 5–10, the fifteen apprentice years
/// 10–25 showing their own restricted pool, then the named sections from 25.
/// General experience fills the fixed later life first, skips the
/// apprenticeship, then the sections. Well-Traveled's grant is no span's.
#[test]
fn a_redcap_has_the_magus_shape_anchored_at_twenty_five() {
    let rs = shipped_ruleset();
    let mut redcap = character("companion", Some(30), vec![sel("virtue.redcap")]);
    plan(&mut redcap).life_sections = vec![section("Road", 3)];
    // The Redcap pool takes 300 of it (it covers every category), general 100.
    redcap.ability_scores = vec![ability_xp("ability.ride", 400)];
    let solved = allocation(&redcap, &rs);
    assert_eq!(solved.general_used, 100, "precondition");

    let timeline = life_timeline(&redcap, &rs, &solved).expect("a timeline");

    assert_eq!(
        timeline.entries,
        vec![
            childhood(0),
            entry(fixed(FixedSection::LaterLife), (5, 10), xp_only(75), 75, 0),
            entry(
                fixed(FixedSection::Apprenticeship),
                (10, 25),
                xp_only(300),
                300,
                0
            ),
            entry(named(0), (25, 28), xp_only(45), 25, 0),
            entry(TimelineEntryKind::Remaining, (28, 30), xp_only(30), 0, 0),
        ]
    );
    let budget = rs
        .life_stages()
        .and_then(|rules| rules.budget(&redcap, &rs))
        .expect("a budget");
    assert_eq!(
        75 + 45 + 30,
        budget.later_life_xp,
        "the fixed later life and the open span are exactly the later-life XP"
    );
    assert_eq!(timeline.outside.len(), 1, "{:?}", timeline.outside);
    assert_eq!(timeline.outside[0].item, Id::new("virtue.well_traveled"));
    assert_eq!(timeline.outside[0].amount, 50);
    assert_eq!(timeline.outside[0].used, 0);
}

/// A Redcap of 22 has not reached 25: the apprenticeship ends at 22 and starts
/// fifteen years earlier, so later life runs 5–7 and the open span is empty.
#[test]
fn a_redcap_younger_than_twenty_five_is_clamped_to_its_age() {
    let rs = shipped_ruleset();
    let mut redcap = character("companion", Some(22), vec![sel("virtue.redcap")]);
    plan(&mut redcap).life_sections = vec![section("Road", 2)];

    let timeline = timeline(&redcap, &rs);

    assert_eq!(
        timeline.entries,
        vec![
            childhood(0),
            entry(fixed(FixedSection::LaterLife), (5, 7), xp_only(30), 0, 0),
            entry(
                fixed(FixedSection::Apprenticeship),
                (7, 22),
                xp_only(300),
                0,
                0
            ),
            entry(named(0), (22, 22), xp_only(0), 0, 0),
            entry(TimelineEntryKind::Remaining, (22, 22), xp_only(0), 0, 0),
        ]
    );
    assert_eq!(timeline.overrun_years, 2);
}

// ---------------------------------------------------------------------------
// Abandoned Apprentice: pre-span → truncated training → post-span sections
// ---------------------------------------------------------------------------

/// An Abandoned Apprentice of 30 who completed 5 years (ArMDE:5716, D3/D64):
/// the training would have started at 25 − 15 = 10, so later life runs 5–10,
/// the truncated training 10–15 (16 XP and 8 levels a year, D56), and the
/// post-span sections from 15 at the later-life rate.
#[test]
fn an_abandoned_apprentice_has_a_pre_span_truncated_training_and_post_span() {
    let rs = shipped_ruleset();
    let mut apprentice = character(
        "companion",
        Some(30),
        vec![
            sel("virtue.the_gift"),
            sel_with("flaw.abandoned_apprentice", &[("years_completed", "5")]),
        ],
    );
    plan(&mut apprentice).life_sections = vec![section("Wandering", 10)];
    apprentice.ability_scores = vec![ability_xp("ability.ride", 50)];
    apprentice.art_scores = vec![art_xp("art.creo", 150)];
    apprentice.spells = spells(&["spell.ball_of_abysmal_flame"]);
    let solved = allocation(&apprentice, &rs);
    assert_eq!(solved.general_used, 150, "precondition: Creo is general");

    let timeline = life_timeline(&apprentice, &rs, &solved).expect("a timeline");

    assert_eq!(
        timeline.entries,
        vec![
            childhood(0),
            entry(fixed(FixedSection::LaterLife), (5, 10), xp_only(75), 50, 0),
            entry(
                fixed(FixedSection::TruncatedTraining),
                (10, 15),
                EntryCapacity::Separate {
                    xp: 80,
                    spell_levels: 40,
                },
                80,
                35,
            ),
            entry(named(0), (15, 25), xp_only(150), 70, 0),
            entry(TimelineEntryKind::Remaining, (25, 30), xp_only(75), 0, 0),
        ]
    );
}

/// An Abandoned Apprentice of 12 who completed 5 years has not lived past the
/// training's hypothetical end at 15. Like a young Redcap, the training is
/// anchored at the age (7–12) and later life shrinks to 5–7; the open span is
/// empty and the named years are overrun. The capacities stay what the budget
/// grants — the timeline moves ages, never experience.
#[test]
fn an_abandoned_apprentice_younger_than_the_training_end_is_clamped_to_its_age() {
    let rs = shipped_ruleset();
    let mut apprentice = character(
        "companion",
        Some(12),
        vec![
            sel("virtue.the_gift"),
            sel_with("flaw.abandoned_apprentice", &[("years_completed", "5")]),
        ],
    );
    plan(&mut apprentice).life_sections = vec![section("Wandering", 3)];

    let timeline = timeline(&apprentice, &rs);

    assert_eq!(
        timeline.entries,
        vec![
            childhood(0),
            entry(fixed(FixedSection::LaterLife), (5, 7), xp_only(75), 0, 0),
            entry(
                fixed(FixedSection::TruncatedTraining),
                (7, 12),
                EntryCapacity::Separate {
                    xp: 80,
                    spell_levels: 40,
                },
                0,
                0,
            ),
            entry(named(0), (12, 12), xp_only(0), 0, 0),
            entry(TimelineEntryKind::Remaining, (12, 12), xp_only(0), 0, 0),
        ]
    );
    assert_eq!(timeline.overrun_years, 3);
}

/// Skilled Parens is open to an Abandoned Apprentice (Hermetic, prerequisite
/// "hermetically trained", which the Flaw confers). Its 60 experience and 30
/// levels "during apprenticeship" (ArMDE:5039) land in the truncated training.
#[test]
fn skilled_parens_widens_an_abandoned_apprentices_truncated_training() {
    let rs = shipped_ruleset();
    let apprentice = character(
        "companion",
        Some(30),
        vec![
            sel("virtue.the_gift"),
            sel_with("flaw.abandoned_apprentice", &[("years_completed", "5")]),
            sel("virtue.skilled_parens"),
        ],
    );

    let timeline = timeline(&apprentice, &rs);

    let training = timeline
        .entries
        .iter()
        .find(|entry| entry.kind == fixed(FixedSection::TruncatedTraining))
        .expect("a truncated-training span");
    assert_eq!(
        training.capacity,
        EntryCapacity::Separate {
            xp: 140,
            spell_levels: 70,
        }
    );
}

// ---------------------------------------------------------------------------
// Childhood replacement
// ---------------------------------------------------------------------------

/// Feral Upbringing replaces both childhood pools with one of its own
/// (ArMDE:6185, D40/D2): the Childhood span shows that pool's amount and use.
#[test]
fn a_childhood_replacement_fills_the_childhood_span() {
    let rs = shipped_ruleset();
    let mut feral = character("companion", Some(25), vec![sel("flaw.feral_upbringing")]);
    // Hunt is on Feral Upbringing's list, not on the ordinary childhood list.
    feral.ability_scores = vec![ability_xp("ability.hunt", 100)];
    let solved = allocation(&feral, &rs);
    let childhood_pools: Vec<(u32, u32)> = solved
        .restricted
        .iter()
        .filter(|pool| {
            matches!(
                pool.origin,
                XpPoolOrigin::LifeStage {
                    block: LifeStageBlock::ChildhoodSpread
                        | LifeStageBlock::ChildhoodNativeLanguage
                }
            )
        })
        .map(|pool| (pool.amount, pool.used))
        .collect();
    assert_eq!(childhood_pools, vec![(120, 100)], "precondition: one pool");

    let timeline = life_timeline(&feral, &rs, &solved).expect("a timeline");

    assert_eq!(timeline.entries.first(), Some(&childhood(100)));
}

/// "In the first five years of life, characters gain 75 experience points in
/// their native language […] and 45 experience points to divide" (ArMDE:2448) —
/// whether or not the language has been picked yet. Without one the solve
/// builds no native-language pool, but the span still earned all 120.
#[test]
fn childhood_earns_its_full_experience_before_a_native_language_is_picked() {
    let rs = shipped_ruleset();
    let mut companion = character("companion", Some(25), vec![]);
    plan(&mut companion).native_language = None;
    companion.ability_scores = vec![ability_xp("ability.athletics", 30)];

    let timeline = timeline(&companion, &rs);

    assert_eq!(timeline.entries.first(), Some(&childhood(30)));
}

// ---------------------------------------------------------------------------
// Earmarks and lumps
// ---------------------------------------------------------------------------

/// Church Upbringing's 25 are "from the normal budget" — carved out of the
/// years — so they fill the timeline; Educated's 50 are a lump on top, shown
/// outside it.
#[test]
fn an_earmark_fills_the_timeline_and_a_lump_stays_outside() {
    let rs = shipped_ruleset();
    let mut companion = character(
        "companion",
        Some(25),
        vec![sel("flaw.church_upbringing"), sel("virtue.educated")],
    );
    plan(&mut companion).life_sections = vec![section("Youth", 10)];
    companion.ability_scores = vec![
        ability_xp("ability.music", 25),
        ability_xp("ability.artes_liberales", 50),
        ability_xp("ability.ride", 100),
    ];
    let solved = allocation(&companion, &rs);
    assert_eq!(solved.general_used, 100, "precondition");

    let timeline = life_timeline(&companion, &rs, &solved).expect("a timeline");

    assert_eq!(
        timeline.entries,
        vec![
            childhood(0),
            entry(named(0), (5, 15), xp_only(150), 125, 0),
            entry(TimelineEntryKind::Remaining, (15, 25), xp_only(150), 0, 0),
        ]
    );
    assert_eq!(timeline.outside.len(), 1, "{:?}", timeline.outside);
    assert_eq!(timeline.outside[0].item, Id::new("virtue.educated"));
    assert_eq!(timeline.outside[0].amount, 50);
    assert_eq!(timeline.outside[0].used, 50);
}

// ---------------------------------------------------------------------------
// Overrun, unset age, switched off, pool funding
// ---------------------------------------------------------------------------

/// Sections claiming more years than the open span holds are clipped to the
/// years lived; the excess is a note, and Remaining is empty.
#[test]
fn sections_beyond_the_open_span_are_clipped_and_noted() {
    let rs = shipped_ruleset();
    let mut companion = character("companion", Some(20), vec![]);
    plan(&mut companion).life_sections = vec![section("A", 10), section("B", 10)];

    let timeline = timeline(&companion, &rs);

    assert_eq!(
        timeline.entries,
        vec![
            childhood(0),
            entry(named(0), (5, 15), xp_only(150), 0, 0),
            entry(named(1), (15, 20), xp_only(75), 0, 0),
            entry(TimelineEntryKind::Remaining, (20, 20), xp_only(0), 0, 0),
        ]
    );
    assert_eq!(timeline.overrun_years, 5);
}

/// With no age there is no open span: childhood still stands (ArMDE:2448), and
/// every named year is overrun.
#[test]
fn an_unset_age_overruns_every_named_year() {
    let rs = shipped_ruleset();
    let mut companion = character("companion", None, vec![]);
    plan(&mut companion).life_sections = vec![section("A", 3), section("B", 4)];

    let timeline = timeline(&companion, &rs);

    assert_eq!(
        timeline.entries,
        vec![
            childhood(0),
            entry(named(0), (5, 5), xp_only(0), 0, 0),
            entry(named(1), (5, 5), xp_only(0), 0, 0),
            entry(TimelineEntryKind::Remaining, (5, 5), xp_only(0), 0, 0),
        ]
    );
    assert_eq!(timeline.overrun_years, 7);
}

#[test]
fn switched_off_life_sections_draw_no_timeline() {
    let rs = shipped_ruleset();
    let mut companion = character("companion", Some(35), vec![]);
    plan(&mut companion).life_sections = vec![section("Squire", 5)];
    plan(&mut companion).life_sections_enabled = false;

    assert_eq!(
        life_timeline(&companion, &rs, &allocation(&companion, &rs)),
        None
    );
}

#[test]
fn pool_funding_draws_no_timeline() {
    let rs = shipped_ruleset();
    let mut companion = character("companion", Some(35), vec![]);
    plan(&mut companion).life_sections = vec![section("Squire", 5)];
    companion.ability_funding = AbilityFunding::Pool;

    assert_eq!(
        life_timeline(&companion, &rs, &allocation(&companion, &rs)),
        None
    );
}

#[test]
fn a_character_without_a_plan_draws_no_timeline() {
    let rs = shipped_ruleset();
    let mut companion = character("companion", Some(35), vec![]);
    companion.life_stages = None;

    assert_eq!(
        life_timeline(&companion, &rs, &allocation(&companion, &rs)),
        None
    );
}

// ---------------------------------------------------------------------------
// Hostile input
// ---------------------------------------------------------------------------

/// A crafted save: ten thousand sections of `u32::MAX` years, each with
/// `u32::MAX` four-season lab years. Saturates; never panics.
#[test]
fn ten_thousand_sections_of_u32_max_years_saturate_without_panicking() {
    let rs = shipped_ruleset();
    let mut magus = character("magus", Some(60), vec![]);
    let mut hostile = section("x", u32::MAX);
    hostile.lab_years = LabYears {
        four_seasons: u32::MAX,
        ..LabYears::default()
    };
    plan(&mut magus).life_sections = vec![hostile; 10_000];

    let timeline = timeline(&magus, &rs);

    assert_eq!(timeline.overrun_years, u32::MAX);
    assert_eq!(timeline.entries.len(), 3 + 10_000 + 1);
    let last = timeline.entries.last().expect("a Remaining entry");
    assert_eq!(last.kind, TimelineEntryKind::Remaining);
    assert_eq!(last.years, 0);
    assert!(
        timeline.entries.iter().all(|entry| entry.end_age <= 60),
        "no span runs past the character's age"
    );
}
