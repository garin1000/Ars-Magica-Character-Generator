//! X8a (`tmp/x8a-verdicts.md`; `docs/vf-audit/corrections.md` § 3.12;
//! `decisions.md` D36, D53, D57) — German rules-text quality: the truncation
//! sub-slice, the glossary-inside-a-quotation ruling, the canonical-term-in-
//! prose ruling, and the apposition fix for free-text/mixed-gender template
//! tokens.
//!
//! Table-driven against the REAL shipped `rules/i18n/{en,de}/virtues_flaws.json`
//! so the fixture proves the shipped data actually changed, not a synthetic
//! stand-in. Full reasoning (current → target, rule applied) for every row is
//! in `tmp/x8a-verdicts.md`.
//!
//! F-46 (`virtue.cyclic_magic_positive`) is already fixed in the shipped tree,
//! so it is a guard, not a red. `virtue.enchanting_ability` is pinned in the
//! D57 table: its DE name template is the Virtue's own, separate from the
//! granted `ability.enchanting` label (`tmp/x8a-handover.md` § E).

use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::Id;

fn shipped_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        mythic_types: Some(include_str!(
            "../../../rules/core/mythic_companion_types.json"
        )),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: Some(include_str!(
            "../../../rules/core/spell_mastery_abilities.json"
        )),
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

fn localized_en() -> LocalizedRuleset {
    LocalizedRuleset::from_merged(
        shipped_ruleset(),
        &[
            include_str!("../../../rules/i18n/en/virtues_flaws.json"),
            include_str!("../../../rules/i18n/en/abilities.json"),
            include_str!("../../../rules/i18n/en/arts.json"),
            include_str!("../../../rules/i18n/en/houses.json"),
            include_str!("../../../rules/i18n/en/mythic_companion_types.json"),
            include_str!("../../../rules/i18n/en/spells.json"),
            include_str!("../../../rules/i18n/en/spell_mastery_abilities.json"),
            include_str!("../../../rules/i18n/en/equipment.json"),
            include_str!("../../../rules/i18n/en/aging.json"),
        ],
    )
    .expect("shipped English rules text loads")
}

fn localized_de() -> LocalizedRuleset {
    LocalizedRuleset::from_merged(
        shipped_ruleset(),
        &[
            include_str!("../../../rules/i18n/de/virtues_flaws.json"),
            include_str!("../../../rules/i18n/de/abilities.json"),
            include_str!("../../../rules/i18n/de/arts.json"),
            include_str!("../../../rules/i18n/de/houses.json"),
            include_str!("../../../rules/i18n/de/mythic_companion_types.json"),
            include_str!("../../../rules/i18n/de/spells.json"),
            include_str!("../../../rules/i18n/de/spell_mastery_abilities.json"),
            include_str!("../../../rules/i18n/de/equipment.json"),
            include_str!("../../../rules/i18n/de/aging.json"),
        ],
    )
    .expect("shipped German rules text loads")
}

// ---------------------------------------------------------------------------
// § 3.12 truncation sub-slice — one cause (a sentence splitter breaking on an
// abbreviation's period or a rhetorical ellipsis), several entries.
// ---------------------------------------------------------------------------

const EN_SUMMARY_TRUNCATIONS: &[(&str, &str)] = &[
    (
        "virtue.mendicant_friar",
        "You are a follower of St. Francis or St. Dominic going among the rich and poor, spreading the word of God and giving comfort to the sick, homeless, hungry, or dying.",
    ),
    (
        "flaw.exciting_experimentation",
        "Your character's experiments tend to have a flair for the ... dramatic.",
    ),
];

#[test]
fn en_summary_truncations_are_repaired() {
    let ruleset = localized_en();
    for (id, expected) in EN_SUMMARY_TRUNCATIONS {
        let id = Id::new(*id);
        let got = ruleset
            .summary(&id)
            .unwrap_or_else(|| panic!("{id} must carry an English summary"));
        assert_eq!(
            got, *expected,
            "{id}'s English summary must be the book's full first sentence (F-193/F-415)"
        );
    }
}

const DE_SUMMARY_TRUNCATIONS: &[(&str, &str)] = &[
    (
        "flaw.exciting_experimentation",
        "Die Experimente deines Charakters neigen zu einer gewissen ... dramatischen Note.",
    ),
    (
        "flaw.lycanthrope",
        "Du wurdest verflucht, dich bei Vollmond (oder ähnlichen, monatlichen astronomischen Ereignissen) in ein gefährliches Raubtier (z.B. Wolf, Luchs oder Bär) zu verwandeln.",
    ),
];

#[test]
fn de_summary_truncations_are_repaired() {
    let ruleset = localized_de();
    for (id, expected) in DE_SUMMARY_TRUNCATIONS {
        let id = Id::new(*id);
        let got = ruleset
            .summary(&id)
            .unwrap_or_else(|| panic!("{id} must carry a German summary"));
        assert_eq!(
            got, *expected,
            "{id}'s German summary must be the book's full first sentence (F-415/F-471)"
        );
    }
}

/// F-46 was already repaired in the shipped tree before this slice started.
/// Guard, not a red: proves the earlier fix is still there rather than
/// proving a new one.
#[test]
fn f46_cyclic_magic_positive_truncation_guard() {
    let ruleset = localized_de();
    let id = Id::new("virtue.cyclic_magic_positive");
    let got = ruleset.summary(&id).expect("must carry a German summary");
    assert_eq!(
        got,
        "Deine Magie ist auf einen Naturzyklus abgestimmt (z. B. solar, lunar oder saisonal) und ist daher zu bestimmten Zeiten besonders potent.",
        "F-46 was already fixed; this guards against a future regression"
    );
}

// ---------------------------------------------------------------------------
// D53 — the glossary governs a game term even inside a verbatim quotation.
// ---------------------------------------------------------------------------

#[test]
fn d53_mild_aging_uses_the_closed_compound_glossary_term() {
    let ruleset = localized_de();
    let id = Id::new("virtue.mild_aging");
    let got = ruleset.summary(&id).expect("must carry a German summary");
    assert_eq!(
        got,
        "Die Alterungswürfe des Charakters profitieren von einem +1-Bonus auf den Lebensumständemodifikator, zusätzlich zu dem, was sein sozialer Stand normalerweise bietet.",
        "translation-tables/alterung-twilight.md:21 governs the term even though the \
         sentence quotes ArMDE:4530 verbatim (D53): Lebensumständemodifikator, not the \
         rulebook's hyphenated Lebensumstände-Modifikator"
    );
}

// ---------------------------------------------------------------------------
// D36 — a canonical German term binds wherever it names a game element the
// app shows, prose included.
// ---------------------------------------------------------------------------

#[test]
fn d36_curse_throwing_and_dowsing_name_the_right_ability_and_score() {
    let de = localized_de();
    let en = localized_en();
    let cases: &[(&str, &str, &str)] = &[
        (
            "virtue.curse_throwing",
            "Heile Krankheiten und entferne Flüche, indem du sie auf eine andere Person überträgst. Verleiht die Fertigkeit Fluchschleudern auf 1.",
            "Cure diseases and remove curses by transferring them to another. Confers the Curse-Throwing Ability at 1.",
        ),
        (
            "virtue.dowsing",
            "Finde nahe Dinge mit einer Wünschelrute. Verleiht die Fertigkeit Wünschelrutengehen auf 1.",
            "Find nearby things with a dowsing rod. Confers the Dowsing Ability at 1.",
        ),
    ];
    for (id, de_expected, en_expected) in cases {
        let id = Id::new(*id);
        assert_eq!(
            de.summary(&id).expect("must carry a German summary"),
            *de_expected,
            "{id}: German must say Fertigkeit (the game term), not Fähigkeit, and must \
             carry the granted score (F-65)"
        );
        assert_eq!(
            en.summary(&id).expect("must carry an English summary"),
            *en_expected,
            "{id}: English must carry the granted score too (F-65's second defect)"
        );
    }
}

#[test]
fn d36_tough_names_soak_as_absorption_not_widerstandsfaehigkeit() {
    let ruleset = localized_de();
    let id = Id::new("virtue.tough");
    let got = ruleset.summary(&id).expect("must carry a German summary");
    assert_eq!(
        got, "+3 auf Absorption.",
        "F-320: both the glossary (kampf.md:27) and the DE rulebook itself (ArMDE:5147) \
         say Absorption(swert), never Widerstandsfähigkeit"
    );
}

#[test]
fn d36_offensive_to_beings_cross_references_the_shipped_magical_air_name() {
    let ruleset = localized_de();
    let id = Id::new("flaw.offensive_to_beings");
    let got = ruleset
        .description(&id)
        .expect("must carry a German description");
    assert!(
        !got.contains("Auftreten"),
        "F-473: must not reference Magisches/Magischem Auftreten, a name the app never \
         shows — flaw.magical_air's own shipped German name is Magische Ausstrahlung"
    );
    assert!(
        got.contains("sollten stattdessen Magische Ausstrahlung nehmen"),
        "F-473: the first occurrence (accusative, direct object of nehmen) must name the \
         Flaw's own shipped German name"
    );
    assert!(
        got.contains("Charaktere mit Magischer Ausstrahlung dürfen ihn überhaupt nicht nehmen"),
        "F-473: the second occurrence (dative after mit) must name the Flaw's own shipped \
         German name, inflected for the feminine noun Ausstrahlung"
    );
}

// ---------------------------------------------------------------------------
// D57 — a free-text or mixed-gender-catalogue placeholder stands in
// apposition after a comma, uninflected, because the app cannot know (or the
// catalogue does not uniformly share) its gender.
// ---------------------------------------------------------------------------

const D57_APPOSITION_NAMES: &[(&str, &str)] = &[
    // Named in Q-77 / the ruling text itself.
    ("virtue.voice_of_the_land", "Stimme eines Landes, {land}"),
    ("virtue.ways_of_the_land", "Wege eines Landes, {land}"),
    // Same shape, found by the "check the rest for the same shape" sweep:
    // free-text `land`/`terrain`/`faculty` params sitting after a
    // case-governing article (im/des/der).
    (
        "flaw.anchored_to_the_land",
        "Verwurzelt in einem Land, {land}",
    ),
    (
        "flaw.fish_out_of_water_terrain",
        "Fremd in einem Gelände, {terrain}",
    ),
    ("flaw.servant_of_the_land", "Diener eines Landes, {land}"),
    (
        "virtue.doctor_in_faculty",
        "Doktor einer Fakultät, {faculty}",
    ),
    // `form` is a closed catalogue, not free text, but it is a MIXED-gender
    // one: the book's own worked example at DE:4093 fills this exact template
    // with Aquam using `des`, not `der`, proving the shipped `der {form}` is
    // already wrong for at least one Form.
    (
        "virtue.extractor_of_form_vis",
        "Vis-Gewinner der Form, {form}",
    ),
    (
        "virtue.imbued_with_the_spirit_of_form",
        "Durchdrungen vom Geist der Form, {form}",
    ),
    // `medium` (`domain: "text"`, free) sits directly as the noun an adjective
    // ending agrees with ("Bezaubernde {medium}"), unlike the other six which
    // at least had an explicit class noun to convert. The canonical glossary
    // term for this entry (`tugenden-fehler.md:116`, `fertigkeiten.md:45`,
    // matching both the DE and EN book headings at ArMDE:3747) is
    // "Bezaubernde (Fertigkeit)" — so the fixed, always-feminine generic noun
    // `Fertigkeit` (never the free-text medium) is what the adjective must
    // agree with, with the medium moved to apposition after a comma.
    (
        "virtue.enchanting_ability",
        "Bezaubernde Fertigkeit, {medium}",
    ),
];

#[test]
fn d57_apposition_templates_do_not_inflect_the_free_or_mixed_gender_token() {
    let ruleset = localized_de();
    for (id, expected) in D57_APPOSITION_NAMES {
        let id = Id::new(*id);
        let got = ruleset
            .display_name(&id)
            .unwrap_or_else(|| panic!("{id} must carry a German name"));
        assert_eq!(
            got, *expected,
            "{id}'s German name template must stand the parameter in apposition after a \
             comma (D57), not mid-phrase behind an article that cannot agree with every \
             possible value"
        );
    }
}

/// Templates checked during the X8a sweep and found NOT to need the
/// apposition fix (no agreement is actually demanded, or the catalogue is
/// uniformly one gender). Guard against a future "simplification" that
/// blanket-converts every `{token}` template without re-checking the shape;
/// see `tmp/x8a-verdicts.md` § D for the per-entry reasoning.
#[test]
fn d57_unaffected_templates_guard() {
    let ruleset = localized_de();
    let cases: &[(&str, &str)] = &[
        // No article at all ("an {role}"), matches the book's own heading.
        ("flaw.bound_to_role_role", "Gebunden an {role}"),
        // Prefix compound: German compounding needs no case/gender agreement.
        ("virtue.land_regio_network", "{land}Regio-Netz"),
        // T2 / C3: the heading, then the Technique after a colon, so the
        // adjective agrees with "Technik" and never with a Latin Art name
        // (same shape as `flaw.deficient_form`'s "Defizitäre Form: {form}").
        (
            "flaw.deficient_technique",
            "Defizitäre Technik: {technique}",
        ),
        ("flaw.deficient_form", "Defizitäre Form: {form}"),
        // Closed 8-item catalogue, uniformly feminine (all 8 Characteristics
        // are grammatically feminine nouns in German).
        (
            "virtue.great_characteristic",
            "Hervorragende {characteristic}",
        ),
    ];
    for (id, expected) in cases {
        let id = Id::new(*id);
        let got = ruleset
            .display_name(&id)
            .unwrap_or_else(|| panic!("{id} must carry a German name"));
        assert_eq!(
            got, *expected,
            "{id} was classified SAFE during the X8a sweep (tmp/x8a-verdicts.md § D); \
             re-check the reasoning there before changing it"
        );
    }
}
