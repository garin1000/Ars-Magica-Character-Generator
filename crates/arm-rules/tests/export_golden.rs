//! Golden-fixture test for the Markdown character export (M5.6a).
//!
//! Renders a fully-populated magus against the **real shipped ruleset** — the same
//! `rules/core` + `rules/i18n/en` files the app loads — and compares the result byte
//! for byte with a checked-in fixture. So this covers what the unit tests in
//! `export.rs` cannot: that the formatter still produces a sane whole document when
//! the catalogue is the real one, and that the document does not drift unnoticed.
//!
//! The label map is **synthetic** (`key → key`), never parsed out of
//! `locales/en/main.ftl`: the engine links no Fluent parser, and a fixture built from
//! real English wording would churn on every copy tweak in the UI. What the fixture
//! pins is the document's *structure* and its *numbers*; the wording is 5.6c's
//! business.

use arm_rules::export::{LABEL_KEYS, character_markdown};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::{
    Characteristic, CrisisSeverity, LabYears, LifeSection, LifeStagePlan, NativeLanguage,
    parse_catalogue_names,
};
use std::collections::{BTreeMap, BTreeSet};

/// The whole shipped ruleset, localized with the shipped English rules text.
fn shipped_ruleset() -> LocalizedRuleset {
    let ruleset = Ruleset::from_sources(RulesetSources {
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
    .expect("the shipped ruleset loads");
    LocalizedRuleset::from_merged(
        ruleset,
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
    .expect("the shipped English rules text loads")
}

/// Every declared chrome key resolved to itself, plus the three catalogue-derived
/// families `export.rs`'s own docs name as deliberately excluded from
/// [`LABEL_KEYS`] (`type-`, `param-label-`, `category-`) — mirroring the
/// frontend's `composedExportLabelKeys` (`ui/src/lib/state.svelte.ts`), which
/// assembles this exact union before every real export. See the module docs for
/// why the fixture is deliberately not localized.
fn synthetic_labels() -> BTreeMap<String, String> {
    let rs = shipped_ruleset();
    let mut keys: BTreeSet<String> = LABEL_KEYS.iter().map(|k| k.to_string()).collect();
    for profile in rs.ruleset.profiles() {
        keys.insert(format!("type-{}", profile.id));
    }
    for item in rs.ruleset.items() {
        for category in &item.categories {
            keys.insert(format!("category-{category}"));
        }
        for param in &item.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    for ability in rs.ruleset.abilities() {
        if let Some(parameter) = &ability.parameter {
            keys.insert(format!("param-label-{parameter}"));
        }
    }
    for spell in rs.ruleset.spells() {
        for param in &spell.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    keys.into_iter().map(|k| (k.clone(), k)).collect()
}

/// A magus built entirely from **real** catalogue ids, touching every section the
/// formatter can emit.
fn golden_magus() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.name = "Marcus of Bonisagus".to_string();
    e.description = "A bookish theorist".to_string();
    e.concept = "Seeker after lost Hermetic lore".to_string();
    e.gender = "male".to_string();
    e.birth_year = Some(1194);
    e.sigil = "the smell of old parchment".to_string();
    e.covenant_name = "Semita Errabunda".to_string();
    e.parens = "Vittoria of Bonisagus".to_string();
    e.house = Some(Id::new("house.bonisagus"));
    e.age = Some(35);
    e.apparent_age = Some(30);
    e.aura = 3;
    for (characteristic, score) in [
        (Characteristic::Int, 3),
        (Characteristic::Per, 1),
        (Characteristic::Str, 1),
        (Characteristic::Sta, 2),
        (Characteristic::Pre, -1),
        (Characteristic::Com, 1),
        (Characteristic::Dex, 1),
        (Characteristic::Qik, 1),
    ] {
        e.characteristics.insert(characteristic, score);
    }
    e.characteristic_descriptions
        .insert(Characteristic::Int, "quick-witted".to_string());
    e.selections = vec![
        Selection::new(Id::new("virtue.the_gift")),
        Selection::new(Id::new("virtue.hermetic_magus")),
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.magic_theory"))]),
        ),
        Selection::with_params(
            Id::new("virtue.minor_magical_focus"),
            BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
        ),
        Selection::new(Id::new("virtue.warrior")),
        Selection::new(Id::new("flaw.blatant_gift")),
    ];
    // House Bonisagus grants a free Puissant Ability; the pick is Intrigue, distinct
    // from the point-bought Puissant Magic Theory above. Off-budget, so the sheet
    // lists it without it moving the balance.
    e.house_choices = BTreeMap::from([(
        "bonisagus_puissant".to_string(),
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.intrigue"))]),
        ),
    )]);
    e.xp_pool = 240;
    e.ability_scores = vec![
        {
            let mut a = AbilityScore::new(Id::new("ability.area_lore"), 2);
            a.specialty = Some("legends".to_string());
            a.parameter = Some(AbilityParameterValue::text("Provence"));
            a
        },
        AbilityScore::new(Id::new("ability.artes_liberales"), 1),
        {
            let mut a = AbilityScore::new(Id::new("ability.awareness"), 2);
            a.specialty = Some("searching".to_string());
            a
        },
        AbilityScore::new(Id::new("ability.magic_theory"), 4),
        AbilityScore::new(Id::new("ability.parma_magica"), 2),
        {
            let mut a = AbilityScore::new(Id::new("ability.single_weapon"), 4);
            a.specialty = Some("long sword".to_string());
            a
        },
    ];
    e.art_scores = vec![
        // Scores chosen so the whole spend fits the 240-point pool: 232 from the
        // general pool (Abilities 100 + Arts 117 + Spell Mastery 15) plus Single
        // Weapon's 50 from Warrior's Martial-only pool.
        ArtScore::new(Id::new("art.creo"), 8),
        ArtScore::new(Id::new("art.rego"), 5),
        ArtScore::new(Id::new("art.corpus"), 5),
        ArtScore::new(Id::new("art.ignem"), 8),
        ArtScore::new(Id::new("art.vim"), 5),
    ];
    e.spells = vec![{
        let mut s = SpellSelection::new(Id::new("spell.pilum_of_fire"));
        s.mastery = Some(2);
        s.mastery_abilities = vec![Id::new("spell_mastery_ability.penetration")];
        s
    }];
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("weapon.sword_long"),
            loadout: LoadoutState::Wielded,
            specialization_applies: true,
        },
        EquipmentSlot {
            item: Id::new("shield.round"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("armor.leather_scale_partial"),
            loadout: LoadoutState::Stowed,
            specialization_applies: false,
        },
    ];
    e.personality_traits = vec![
        PersonalityTrait {
            name: "Curious".to_string(),
            value: 3,
        },
        PersonalityTrait {
            name: "Brash".to_string(),
            value: -2,
        },
    ];
    e.reputations = vec![Reputation {
        kind: ReputationType::Hermetic,
        score: 2,
        content: "a promising theoretician".to_string(),
    }];
    e.devices = vec![EnchantedDevice {
        name: "Ring of Seeing".to_string(),
        level: 15,
    }];
    e.talisman = Some(Talisman {
        description: "an ash staff shod with silver".to_string(),
        attunements: vec![TalismanAttunement {
            description: "to ward off flame".to_string(),
            bonus: 3,
        }],
        effects: vec![TalismanEffect {
            name: "Lamp Without Flame".to_string(),
            level: 10,
        }],
    });
    e.longevity_ritual = Some(LongevityRitual {
        source: LongevitySource::SelfMade,
        bonus: Some(7),
        focus: "a draught of gold and silver".to_string(),
    });
    e.familiar = Some(Familiar {
        name: "Corvus".to_string(),
        animal: "a raven".to_string(),
        might: Some(MightScore {
            realm: Realm::Magic,
            score: 10,
        }),
        characteristics: BTreeMap::from([(Characteristic::Int, 2), (Characteristic::Qik, 4)]),
        size: -4,
        personality_traits: vec![PersonalityTrait {
            name: "Loyal".to_string(),
            value: 3,
        }],
        cord_gold: 2,
        cord_silver: 1,
        cord_bronze: 0,
        powers: vec![SupernaturalPower {
            name: "Wings of the Storm".to_string(),
            level: 20,
            penetration: 0,
        }],
    });
    e.warping_points = 6;
    e.warping_effect = "his shadow lags a heartbeat behind".to_string();
    e.twilight_scars = vec![TwilightScar {
        description: "his eyes reflect no candlelight".to_string(),
    }];
    e.aging_points = BTreeMap::from([(Characteristic::Pre, 5)]);
    e.decrepitude_effect = "a persistent cough each winter".to_string();
    // Two cumulative rows off the shipped table, so the fixture pins that a stored
    // choice reaches the sheet through the rules i18n rather than as its slug.
    e.living_conditions = BTreeSet::from([
        Id::new("living_condition.work_in_a_mine"),
        Id::new("living_condition.live_in_a_leper_colony"),
    ]);
    // One hand-written year and two the engine resolved, so the fixture covers the
    // free-text entry, the widened one carrying its die and total, and both states a
    // Crisis can be in: demanded and unrolled (1229), and resolved against the
    // Crisis Table (1230, ArMDE:16698-16709).
    e.aging_log = vec![
        AgingLogEntry {
            year: Some(1220),
            effect: "an apparent aging crisis, weathered".to_string(),
            ..AgingLogEntry::default()
        },
        AgingLogEntry {
            year: Some(1229),
            age: Some(35),
            die: Some(9),
            total: Some(13),
            living_conditions: BTreeSet::from([Id::new("living_condition.work_in_a_mine")]),
            points: BTreeMap::from([(Characteristic::Pre, 5)]),
            crisis: true,
            ..AgingLogEntry::default()
        },
        AgingLogEntry {
            year: Some(1230),
            age: Some(36),
            die: Some(9),
            total: Some(13),
            living_conditions: BTreeSet::from([Id::new("living_condition.work_in_a_mine")]),
            crisis: true,
            crisis_die: Some(10),
            crisis_total: Some(15),
            crisis_row: Some(Id::new("crisis.minor_illness")),
            crisis_severity: Some(CrisisSeverity::Minor),
            ..AgingLogEntry::default()
        },
    ];
    e.normalize();
    e
}

#[test]
fn a_fully_populated_magus_matches_the_golden_document() {
    let rendered = character_markdown(&golden_magus(), &shipped_ruleset(), &synthetic_labels())
        .expect("synthetic_labels resolves every chrome key and every id is real");
    let expected = include_str!("fixtures/magus_export.md");
    assert_eq!(
        rendered, expected,
        "the rendered document drifted from tests/fixtures/magus_export.md"
    );
}

/// A Virtue/Flaw whose descriptor names two categories renders BOTH in the
/// "Type" cell, in the descriptor's order and joined with the same localized list
/// separator the rest of the document uses. Rendering only the primary would hide
/// that Visions is a Supernatural Flaw as well as a Story one.
///
/// Source: ArMDE:7056-7057
/// (*Minor, Story, Supernatural*).
#[test]
fn the_type_cell_lists_every_category_the_descriptor_names() {
    let ruleset = shipped_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.selections = vec![Selection::new(Id::new("flaw.visions"))];

    // Readable stand-ins for the three keys this row composes, so the assertion
    // reads as the sheet does. Everything else stays key-as-label.
    let mut labels = synthetic_labels();
    labels.insert("category-story".to_string(), "Story".to_string());
    labels.insert(
        "category-supernatural".to_string(),
        "Supernatural".to_string(),
    );
    labels.insert("restricted-xp-list-separator".to_string(), ",".to_string());

    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("synthetic_labels resolves every chrome key and every id is real");
    assert!(
        rendered.contains("| Story, Supernatural |"),
        "the Type cell must list both categories, primary first; got:\n{rendered}"
    );
}

/// D42/D70/D74: a Supernatural Virtue/Flaw's resolved realm rides the
/// existing text cell — a new cell would change the table shape (and hence
/// the byte-stability of every OTHER row) just because one row needs it.
/// Faerie Blood is `Fixed { realm: Faerie }` in the shipped data, so the
/// concept realm set here must be ignored.
#[test]
fn a_fixed_supernatural_entrys_realm_rides_the_text_cell() {
    let ruleset = shipped_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.concept_realm = Some(Realm::Infernal);
    entity.selections = vec![Selection::with_params(
        Id::new("virtue.faerie_blood"),
        BTreeMap::from([("heritage".to_string(), Id::new("heritage.sidhe"))]),
    )];

    let mut labels = synthetic_labels();
    labels.insert("export-vf-realm-label".to_string(), "Realm".to_string());
    labels.insert("realm-faerie".to_string(), "Faerie".to_string());

    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("synthetic_labels resolves every chrome key and every id is real");
    assert!(
        rendered.contains("Realm: Faerie"),
        "the Fixed realm must render regardless of the concept realm; got:\n{rendered}"
    );
}

/// A non-Supernatural entry's text cell is untouched — no "Realm:" line, no
/// trailing parenthesis — so an entity holding none of these renders exactly
/// as it did before this feature existed (already locked end-to-end by
/// `a_fully_populated_magus_matches_the_golden_document`; this is the
/// targeted, single-row version of the same claim).
#[test]
fn a_non_supernatural_entrys_text_cell_gets_no_realm_line() {
    let ruleset = shipped_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.selections = vec![Selection::new(Id::new("virtue.warrior"))];

    let labels = synthetic_labels();
    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("synthetic_labels resolves every chrome key and every id is real");
    assert!(
        !rendered.contains("export-vf-realm-label") && !rendered.contains("Realm:"),
        "a non-Supernatural row must carry no realm text; got:\n{rendered}"
    );
}

/// Row 55 (`docs/open-todos.md`; D74.4) — `write_virtues_flaws` renders the
/// GRANTED table through the exact same [`item_rows`] helper as the bought
/// one, so the realm line already appeared here before row 55 — just the
/// WRONG realm. Faerie Doctor (ArMDE:2736-2772, buyable standalone: "A
/// character can be a faerie doctor without being a Mythic Companion") grants
/// Dowsing for free; Dowsing has no `realm_association` of its own and Faerie
/// Doctor's OWN category is `mythic_companion`, not `supernatural`, so
/// (unlike a Strong-Faerie-Blood fixture) the bought row contributes no
/// "Realm:" text of its own — only the granted Dowsing row's resolution is
/// under test here. `virtue.faerie_doctor`'s `grants_selection` effect now
/// states `realm: "faerie"` (ArMDE:2709), stamped onto the granted Dowsing by
/// `effective::vf_granted_selections`, so it resolves Faerie instead of
/// falling through the plain chain to Magic.
#[test]
fn a_granted_supernatural_entrys_realm_rides_the_granted_table_text_cell() {
    let ruleset = shipped_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.selections = vec![Selection::new(Id::new("virtue.faerie_doctor"))];

    let mut labels = synthetic_labels();
    labels.insert("export-vf-realm-label".to_string(), "Realm".to_string());
    labels.insert("realm-faerie".to_string(), "Faerie".to_string());
    labels.insert("realm-magic".to_string(), "Magic".to_string());

    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("synthetic_labels resolves every chrome key and every id is real");
    assert!(
        rendered.contains("Realm: Faerie"),
        "ArMDE:2709 — the granted Dowsing must show Faerie, not the Magic fallback; \
         got:\n{rendered}"
    );
}

#[test]
fn rendering_the_same_magus_twice_is_byte_identical() {
    let ruleset = shipped_ruleset();
    let entity = golden_magus();
    let labels = synthetic_labels();
    assert_eq!(
        character_markdown(&entity, &ruleset, &labels),
        character_markdown(&entity, &ruleset, &labels)
    );
}

// --- CV8 (design-cv-catalogued-values.md § 6.4): display/export parity -----
//
// CV7 already made `export/resolve.rs::Doc::ability_param_value` the single
// resolver for a bought Ability's stored `AbilityParameterValue`; what CV8 owes
// is proof, not new production code — the design row's own words: "already
// benefits from CV7's `LocalizedRuleset.i18n` merge, but its own
// humanize-fallback code path is unchanged". These are the "one Rust test"
// § 6.4 promises for the resolver (the TS twin is `AbilityTab.test.ts`'s
// "shows a catalogued value by its localized name, never the raw id").

/// The whole shipped ruleset, localized with the shipped German rules text —
/// the DE counterpart of `shipped_ruleset()`, needed to prove a `Catalogued`
/// value resolves through REAL per-language i18n rather than the id's-own-
/// final-segment fallback (which happens to read "Latin" in English too, so an
/// English-only test cannot tell the two apart).
fn shipped_ruleset_de() -> LocalizedRuleset {
    let ruleset = Ruleset::from_sources(RulesetSources {
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
    .expect("the shipped ruleset loads");
    LocalizedRuleset::from_merged(
        ruleset,
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
    .expect("the shipped German rules text loads")
}

/// Merges a catalogue's per-language display names into `localized.i18n`,
/// mirroring `arm-app::ruleset_io::merge_catalogue_display_names` exactly —
/// that function lives in the `arm-app` crate, which this pure-engine test
/// cannot depend on, so this reproduces its effect on the SAME public map
/// (`LocalizedRuleset.i18n`) via the SAME public parser
/// (`arm_rules::parse_catalogue_names`), so `Catalogued` is resolved the same
/// way the real app loads it, not just through its defense-in-depth fallback.
fn merge_catalogue_names(localized: &mut LocalizedRuleset, catalogue_json: &str) {
    for (id, name) in parse_catalogue_names(catalogue_json).expect("catalogue names parse") {
        localized.i18n.insert(
            id,
            I18nEntry {
                name,
                name_unfilled: None,
                summary: None,
                description: None,
                abbreviation: None,
                specialties: Vec::new(),
            },
        );
    }
}

/// A companion holding all three `AbilityParameterValue` shapes at once:
/// `Catalogued` (Dead Language: Latin), `Linked` (Organization Lore following
/// Craft Guild Training's current guild), and `Text` (Area Lore: Provence).
fn entity_with_every_parameter_kind() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".to_string(), Id::new("Smiths' Guild of Verdi"))]),
    )];
    e.ability_scores = vec![
        {
            let mut a = AbilityScore::new(Id::new("ability.dead_language"), 4);
            a.parameter = Some(AbilityParameterValue::Catalogued {
                id: Id::new("language.latin"),
            });
            a
        },
        {
            let mut a = AbilityScore::new(Id::new("ability.organization_lore"), 2);
            a.parameter = Some(AbilityParameterValue::Linked {
                item: Id::new("virtue.craft_guild_training"),
                param: "guild".to_string(),
            });
            a
        },
        {
            let mut a = AbilityScore::new(Id::new("ability.area_lore"), 1);
            a.parameter = Some(AbilityParameterValue::text("Provence"));
            a
        },
    ];
    e.normalize();
    e
}

/// No surface may print a raw catalogue id (`language.latin`) or the bare
/// `(item, param)` pair (`virtue.craft_guild_training`/`guild`) as visible
/// text — only the resolved/localized name design § 6.4 requires. A search
/// for the catalogue-prefixed id form covers every catalogue family this
/// design introduces, not only the one this fixture happens to exercise.
fn assert_no_raw_parameter_id(rendered: &str) {
    for prefix in ["language.", "organization.", "profession."] {
        assert!(
            !rendered.contains(prefix),
            "export must never print a raw catalogue id (prefix {prefix:?} found):\n{rendered}"
        );
    }
    assert!(
        !rendered.contains("virtue.craft_guild_training"),
        "export must never print a raw (item, param) link pair as text:\n{rendered}"
    );
}

#[test]
fn a_catalogued_a_linked_and_a_text_ability_parameter_render_localized_in_english() {
    let mut ruleset = shipped_ruleset();
    merge_catalogue_names(
        &mut ruleset,
        include_str!("../../../rules/i18n/en/parameter_catalogue.json"),
    );
    let entity = entity_with_every_parameter_kind();
    let rendered = character_markdown(&entity, &ruleset, &synthetic_labels())
        .expect("every id in the fixture is real");

    assert!(
        rendered.contains("Latin (Dead Language)"),
        "a Catalogued value must show its EN localized name:\n{rendered}"
    );
    assert!(
        rendered.contains("Smiths' Guild of Verdi Lore"),
        "a Linked value must show the source Virtue's CURRENT resolved text:\n{rendered}"
    );
    assert!(
        rendered.contains("Provence Lore"),
        "a Text value must pass through unchanged:\n{rendered}"
    );
    assert_no_raw_parameter_id(&rendered);
}

#[test]
fn a_catalogued_a_linked_and_a_text_ability_parameter_render_localized_in_german() {
    let mut ruleset = shipped_ruleset_de();
    merge_catalogue_names(
        &mut ruleset,
        include_str!("../../../rules/i18n/de/parameter_catalogue.json"),
    );
    let entity = entity_with_every_parameter_kind();
    let rendered = character_markdown(&entity, &ruleset, &synthetic_labels())
        .expect("every id in the fixture is real");

    assert!(
        rendered.contains("Latein (Tote Sprache)"),
        "a Catalogued value must show its DE localized name — proving real \
         i18n resolution, not the EN-shaped structural id fallback which also \
         happens to read \"Latin\":\n{rendered}"
    );
    assert!(
        rendered.contains("Smiths' Guild of Verdi-Kunde"),
        "a Linked value must show the source Virtue's CURRENT resolved text, \
         DE-templated:\n{rendered}"
    );
    assert!(
        rendered.contains("Provence-Kunde"),
        "a Text value must pass through unchanged, DE-templated:\n{rendered}"
    );
    assert_no_raw_parameter_id(&rendered);
}

/// The Dangling case (design § 4.1, `LinkResolution::Dangling`): a `Linked`
/// parameter whose target item is not held at all — unlike
/// `entity_with_every_parameter_kind`, which always holds
/// `virtue.craft_guild_training` and so only ever exercises `Resolved` —
/// must still render (never panic or fail the export) and never leak the
/// raw `(item, param)` pair. `export/resolve.rs::Doc::ability_param_value`'s
/// `LinkResolution::Dangling` arm was otherwise never reached by any test.
#[test]
fn a_dangling_linked_parameter_renders_without_the_raw_link_pair() {
    let ruleset = shipped_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.ability_scores = vec![{
        let mut a = AbilityScore::new(Id::new("ability.organization_lore"), 2);
        a.parameter = Some(AbilityParameterValue::Linked {
            item: Id::new("virtue.craft_guild_training"),
            param: "guild".to_string(),
        });
        a
    }];
    entity.normalize();

    let rendered = character_markdown(&entity, &ruleset, &synthetic_labels())
        .expect("every id in the fixture is real, even though the linked item is not held");

    assert!(
        rendered.contains("Lore"),
        "a dangling Linked parameter must still render the ability row:\n{rendered}"
    );
    assert_no_raw_parameter_id(&rendered);
}

/// The Ambiguous case (design § 4.1, `LinkResolution::Ambiguous`): TWO bought
/// copies of the Linked target leave no single value to follow — resolution
/// must fall back to the same deterministic empty text as Dangling (never
/// guess, never panic, never leak the raw pair), not show either copy's
/// value as if it were the only one. `export/resolve.rs::Doc::ability_param_value`'s
/// `LinkResolution::Ambiguous` arm was otherwise never reached by any test.
#[test]
fn an_ambiguous_linked_parameter_renders_without_the_raw_link_pair() {
    let ruleset = shipped_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.selections = vec![
        Selection::with_params(
            Id::new("virtue.craft_guild_training"),
            BTreeMap::from([("guild".to_string(), Id::new("Smiths' Guild of Verdi"))]),
        ),
        Selection::with_params(
            Id::new("virtue.craft_guild_training"),
            BTreeMap::from([("guild".to_string(), Id::new("Weavers' Guild of Londres"))]),
        ),
    ];
    entity.ability_scores = vec![{
        let mut a = AbilityScore::new(Id::new("ability.organization_lore"), 2);
        a.parameter = Some(AbilityParameterValue::Linked {
            item: Id::new("virtue.craft_guild_training"),
            param: "guild".to_string(),
        });
        a
    }];
    entity.normalize();

    let rendered = character_markdown(&entity, &ruleset, &synthetic_labels())
        .expect("every id in the fixture is real, even though which copy to follow is ambiguous");

    assert!(
        rendered.contains("Lore"),
        "an ambiguous Linked parameter must still render the ability row:\n{rendered}"
    );
    // Both held copies' OWN name rows legitimately show their own guild
    // ("Craft Guild Training (Smiths' Guild of Verdi)") — that is a
    // different, correct rendering path (the virtue's own parameterized
    // name), not the ambiguous Ability's resolution. What must never appear
    // is the Ability guessing either guild as ITS OWN resolved value.
    assert!(
        !rendered.contains("Smiths' Guild of Verdi Lore")
            && !rendered.contains("Weavers' Guild of Londres Lore"),
        "an ambiguous Linked parameter must never guess either copy's value for the Ability row:\n{rendered}"
    );
    assert_no_raw_parameter_id(&rendered);
}

/// The defense-in-depth fallback (design § 6.4, `export/resolve.rs`'s doc
/// comment on `ability_param_value`): a `Catalogued` value whose id the
/// loaded ruleset's i18n does NOT merge a display name for — a hand-built
/// fixture that never ran catalogue-name merging, per that doc comment's own
/// example — falls back to a structural, readable label derived from the
/// id's own final segment (`humanize_catalogue_id`) rather than printing the
/// raw slug. Every existing `Catalogued` test merges real catalogue names
/// first, so this fallback path (`export/resolve.rs::ability_param_value`'s
/// `Catalogued` arm, falling through to `resolve.rs::humanize_catalogue_id`)
/// was never reached by any test.
#[test]
fn an_unmerged_catalogued_parameter_falls_back_to_a_humanized_label() {
    // Deliberately NOT merge_catalogue_names()'d: this id exists nowhere in
    // the loaded ruleset's i18n, exactly the "hand-built fixture" case the
    // fallback exists for.
    let ruleset = shipped_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.ability_scores = vec![{
        let mut a = AbilityScore::new(Id::new("ability.dead_language"), 3);
        a.parameter = Some(AbilityParameterValue::Catalogued {
            id: Id::new("language.nonexistent_test_language"),
        });
        a
    }];
    entity.normalize();

    let rendered = character_markdown(&entity, &ruleset, &synthetic_labels()).expect(
        "every id in the fixture is real, even though the catalogued value has no merged name",
    );

    assert!(
        rendered.contains("Nonexistent Test Language (Dead Language)"),
        "an unmerged Catalogued value must fall back to its humanized final segment:\n{rendered}"
    );
    assert!(
        !rendered.contains("language.nonexistent_test_language"),
        "no raw catalogue id, even on the fallback path:\n{rendered}"
    );
}

// --- G4: the life sections' table ---------------------------------------------
//
// Unlike the whole-document fixture above, these goldens pin the Life table's
// real wording in both locales: its chrome is new, and a German column header
// left in English is exactly what a key-as-label map cannot catch. Only the Life
// section is compared, so copy tweaks elsewhere in the bundles cannot churn them.

/// [`synthetic_labels`] with every argument-free one-line message of `ftl` that
/// [`LABEL_KEYS`] declares resolved to its real text — the subset the app's own
/// Fluent bundle would hand the exporter.
fn locale_labels(ftl: &str) -> BTreeMap<String, String> {
    let mut labels = synthetic_labels();
    for line in ftl.lines() {
        let Some((key, value)) = line.split_once(" = ") else {
            continue;
        };
        if LABEL_KEYS.contains(&key) && !value.contains('{') {
            labels.insert(key.to_string(), value.trim().to_string());
        }
    }
    labels
}

fn english_labels() -> BTreeMap<String, String> {
    locale_labels(include_str!("../../../locales/en/main.ftl"))
}

fn german_labels() -> BTreeMap<String, String> {
    locale_labels(include_str!("../../../locales/de/main.ftl"))
}

/// The `## heading` section of `doc`, up to the next level-2 heading; empty when
/// the document has no such section.
fn h2_section(doc: &str, heading: &str) -> String {
    let Some(start) = doc.find(&format!("\n## {heading}\n")) else {
        return String::new();
    };
    let rest = &doc[start + 1..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |at| at + 4);
    rest[..end].to_string()
}

/// A character of `type_id` funded from its life stages, with the life sections
/// switched on.
fn life_character(type_id: &str, age: u32) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.ability_funding = AbilityFunding::LifeStages;
    e.age = Some(age);
    e.life_stages = Some(LifeStagePlan {
        native_language: Some(NativeLanguage::from("Frankish")),
        life_sections_enabled: true,
        ..LifeStagePlan::default()
    });
    e
}

fn life_plan(e: &mut Entity) -> &mut LifeStagePlan {
    e.life_stages.as_mut().expect("the fixture has a plan")
}

fn life_section(name: &str, years: u32, lab_years: LabYears) -> LifeSection {
    LifeSection {
        name: name.to_string(),
        years,
        lab_years,
    }
}

/// An Ability at score 0 with `xp` banked: a spend of exactly `xp`.
fn banked_ability(ability: &str, xp: u32) -> AbilityScore {
    let mut row = AbilityScore::new(Id::new(ability), 0);
    row.banked_xp = xp;
    row
}

/// A companion of 30 whose three sections claim 35 of its 25 later-life years:
/// one named with a `|`, one left unnamed, one clipped by the 10-year overrun.
/// Warrior's 50 Martial experience is drawn from no span (an outside lump).
fn life_companion() -> Entity {
    let mut e = life_character("companion", 30);
    e.selections = vec![Selection::new(Id::new("virtue.warrior"))];
    life_plan(&mut e).life_sections = vec![
        life_section("Squire | page", 5, LabYears::default()),
        life_section("", 5, LabYears::default()),
        life_section("Crusade", 25, LabYears::default()),
    ];
    e.ability_scores = vec![
        banked_ability("ability.athletics", 30),
        banked_ability("ability.ride", 150),
        banked_ability("ability.single_weapon", 50),
    ];
    e.normalize();
    e
}

/// Darius's shape (ArMDE:2554, :2556) at 36: eight years of study, a year of
/// four lab seasons in a section named with a `|`, and two Remaining years with
/// one and two lab seasons — 240 + 0 + 30 post-Gauntlet points (ArMDE:2539,
/// :2550). 40 of them are levels of spells.
fn life_magus() -> Entity {
    let mut e = life_character("magus", 36);
    let plan = life_plan(&mut e);
    plan.life_sections = vec![
        life_section("Study", 8, LabYears::default()),
        life_section(
            "Lab | Pisa",
            1,
            LabYears {
                four_seasons: 1,
                ..LabYears::default()
            },
        ),
    ];
    plan.post_gauntlet_lab_years = LabYears {
        one_season: 1,
        two_seasons: 1,
        ..LabYears::default()
    };
    plan.post_gauntlet_spell_levels = 40;
    e.ability_scores = vec![banked_ability("ability.ride", 40)];
    e.art_scores = vec![{
        let mut row = ArtScore::new(Id::new("art.creo"), 0);
        row.banked_xp = 340;
        row
    }];
    // 40 + 35 + 25 + 25 + 20 + 15 = 160 levels.
    e.spells = [
        "spell.black_whisper",
        "spell.ball_of_abysmal_flame",
        "spell.bane_of_the_decrepit_body",
        "spell.blessing_of_childlike_bliss",
        "spell.beast_of_minuscule_proportions",
        "spell.beast_of_outlandish_size",
    ]
    .iter()
    .map(|&id| SpellSelection::new(Id::new(id)))
    .collect();
    e.normalize();
    e
}

fn life_table(
    entity: &Entity,
    ruleset: &LocalizedRuleset,
    labels: &BTreeMap<String, String>,
) -> String {
    let rendered = character_markdown(entity, ruleset, labels)
        .expect("every chrome key is declared and every id is real");
    let heading = labels
        .get("life-timeline-label")
        .map_or("life-timeline-label", String::as_str);
    h2_section(&rendered, heading)
}

#[test]
fn a_companions_life_table_matches_the_english_golden() {
    let expected = "\
## Life timeline

| Section | Ages | Years | Earned | Filled |
| --- | --- | --- | --- | --- |
| Early childhood | 0–5 | 5 | 120 XP | 30 XP |
| Squire \\| page | 5–10 | 5 | 75 XP | 75 XP |
| Section 2 | 10–15 | 5 | 75 XP | 75 XP |
| Crusade | 15–30 | 15 | 225 XP | 0 XP |
| Remaining | 30–30 | 0 | 0 XP | 0 XP |

- **Years past the character's age**: 10
- **Experience beyond the years**: Warrior 50 / 50 XP

";
    assert_eq!(
        life_table(&life_companion(), &shipped_ruleset(), &english_labels()),
        expected
    );
}

#[test]
fn a_companions_life_table_matches_the_german_golden() {
    let expected = "\
## Lebenslauf

| Abschnitt | Alter | Jahre | Erworben | Genutzt |
| --- | --- | --- | --- | --- |
| Frühe Kindheit | 0–5 | 5 | 120 EP | 30 EP |
| Squire \\| page | 5–10 | 5 | 75 EP | 75 EP |
| Abschnitt 2 | 10–15 | 5 | 75 EP | 75 EP |
| Crusade | 15–30 | 15 | 225 EP | 0 EP |
| Übrige Jahre | 30–30 | 0 | 0 EP | 0 EP |

- **Jahre über das Alter des Charakters hinaus**: 10
- **Erfahrung außerhalb der Jahre**: Krieger 50 / 50 EP

";
    assert_eq!(
        life_table(&life_companion(), &shipped_ruleset_de(), &german_labels()),
        expected
    );
}

/// A magus's table adds the lab-years column: each post-Gauntlet span's years
/// counted by their lab seasons, the fixed spans blank.
#[test]
fn a_magus_life_table_matches_the_english_golden() {
    let expected = "\
## Life timeline

| Section | Ages | Years | Earned | Filled | Years of lab work |
| --- | --- | --- | --- | --- | --- |
| Early childhood | 0–5 | 5 | 120 XP | 0 XP |  |
| Later life | 5–10 | 5 | 75 XP | 40 XP |  |
| Apprenticeship | 10–25 | 15 | 240 XP, 120 levels of spells | 240 XP, 120 levels of spells |  |
| Study | 25–33 | 8 | 240 points | 100 XP, 40 levels of spells |  |
| Lab \\| Pisa | 33–34 | 1 | 0 points | 0 XP, 0 levels of spells | 1 × 4 seasons |
| Remaining | 34–36 | 2 | 30 points | 0 XP, 0 levels of spells | 1 × 1 season, 1 × 2 seasons |

";
    assert_eq!(
        life_table(&life_magus(), &shipped_ruleset(), &english_labels()),
        expected
    );
}

#[test]
fn a_magus_life_table_matches_the_german_golden() {
    let expected = "\
## Lebenslauf

| Abschnitt | Alter | Jahre | Erworben | Genutzt | Jahre mit Laborarbeit |
| --- | --- | --- | --- | --- | --- |
| Frühe Kindheit | 0–5 | 5 | 120 EP | 0 EP |  |
| Späteres Leben | 5–10 | 5 | 75 EP | 40 EP |  |
| Lehrlingszeit | 10–25 | 15 | 240 EP, 120 Zauberstufen | 240 EP, 120 Zauberstufen |  |
| Study | 25–33 | 8 | 240 Punkte | 100 EP, 40 Zauberstufen |  |
| Lab \\| Pisa | 33–34 | 1 | 0 Punkte | 0 EP, 0 Zauberstufen | 1 × 4 Quartale |
| Übrige Jahre | 34–36 | 2 | 30 Punkte | 0 EP, 0 Zauberstufen | 1 × 1 Quartal, 1 × 2 Quartale |

";
    assert_eq!(
        life_table(&life_magus(), &shipped_ruleset_de(), &german_labels()),
        expected
    );
}

/// A `|` in a section's name is escaped like every other free text on the sheet,
/// so the row keeps exactly its six cells.
#[test]
fn a_pipe_in_a_section_name_cannot_split_its_cell() {
    let table = life_table(&life_magus(), &shipped_ruleset(), &english_labels());
    let row = table
        .lines()
        .find(|line| line.contains("Pisa"))
        .unwrap_or_default();
    assert!(row.starts_with("| Lab \\| Pisa | 33–34 |"), "{table}");
    assert_eq!(row.replace("\\|", "").matches('|').count(), 7, "{row}");
}

/// Switched off, the sections draw no table, and the sheet is byte-identical to
/// the same character's with the sections folded away — the shape every export
/// had before the sections existed.
#[test]
fn switched_off_life_sections_leave_the_export_byte_identical() {
    let ruleset = shipped_ruleset();
    let labels = english_labels();
    let enabled = life_magus();
    let mut disabled = enabled.clone();
    life_plan(&mut disabled).life_sections_enabled = false;
    let mut folded = enabled.clone();
    life_plan(&mut folded).disable_life_sections();
    folded.life_stages.as_mut().expect("plan").life_sections = Vec::new();

    let render = |e: &Entity| character_markdown(e, &ruleset, &labels).expect("renders");
    assert_ne!(
        render(&enabled),
        render(&disabled),
        "switched on, the sections must add their table"
    );
    assert_eq!(life_table(&disabled, &ruleset, &labels), "");
    assert_eq!(render(&disabled), render(&folded));
}
