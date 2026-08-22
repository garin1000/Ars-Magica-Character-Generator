use arm_rules::AbilityCategory;
use arm_rules::Characteristic;
use arm_rules::aging::{
    AgingOutcome, AgingPointAward, AgingPointTarget, AgingTotal, CrisisAllowance, CrisisModifier,
    CrisisModifierSource, CrisisOutcome, CrisisSeverity, CrisisSurvival,
};
use arm_rules::effective_art_score;
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{compute_balance, validate};
use arm_rules::{AgingRowEffect, AgingRules};
use std::collections::BTreeMap;

fn load_ruleset() -> Ruleset {
    let items = include_str!("../../../rules/core/virtues_flaws.json");
    let types = include_str!("../../../rules/core/character_types.json");
    let abilities = include_str!("../../../rules/core/abilities.json");
    let characteristics = include_str!("../../../rules/core/characteristics.json");
    Ruleset::from_core_json(
        "arm5-core",
        "2024.1",
        items,
        types,
        abilities,
        Some(characteristics),
    )
    .unwrap()
}

/// The full shipped ruleset including Arts and the spell catalogue — the spell
/// tests need Arts loaded so each spell's Technique/Form resolves.
fn load_ruleset_with_spells() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: None,
        mythic_types: None,
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: None,
        equipment: None,
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: None,
    })
    .unwrap()
}

/// The full shipped ruleset including the Spell Mastery special-ability
/// catalogue.
fn load_ruleset_with_mastery_abilities() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: None,
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: Some(include_str!(
            "../../../rules/core/spell_mastery_abilities.json"
        )),
        equipment: None,
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: None,
    })
    .unwrap()
}

/// The full shipped ruleset including the equipment catalogue — the equipment
/// tests need Abilities loaded so each weapon's combat Ability resolves.
fn load_ruleset_with_equipment() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: None,
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: None,
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: None,
    })
    .unwrap()
}

/// Builds a character entity of `type_id` with the given selections, at the
/// current schema version and empty trait data.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

#[test]
fn shipped_data_passes_integrity_check() {
    let rs = load_ruleset();
    // Catalogue size is data, not code: assert key items are present, never an
    // exact V/F total (which would break when any item is added to the JSON).
    assert!(
        rs.item(&Id::new("virtue.the_gift")).is_some(),
        "virtue.the_gift must be present"
    );
    // All four character-type profiles must load.
    assert!(rs.profile(&Id::new("companion")).is_some());
    assert!(rs.profile(&Id::new("grog")).is_some());
    assert!(rs.profile(&Id::new("magus")).is_some());
    assert!(rs.profile(&Id::new("mythic_companion")).is_some());
    // The magus is the only seeded Hermetic type.
    assert!(
        rs.profile(&Id::new("magus")).unwrap().is_magus,
        "magus profile must carry is_magus"
    );
    // Mythic Companions convert each Flaw point into two Virtue points.
    assert_eq!(
        rs.profile(&Id::new("mythic_companion"))
            .unwrap()
            .budget
            .virtue_points_per_flaw_point,
        2,
        "mythic companion funds virtues at 2:1"
    );
}

/// The two Virtues/Flaws that change the later-life experience rate, and the
/// eligibility the same rules line puts on them.
///
/// Poor was absent from the shipped catalogue entirely until this milestone (the
/// extraction plausibly dropped it because six other flaw names begin with
/// "Poor"), so this test is as much a guard against losing it again as a check on
/// its numbers.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2394 ("Characters with
/// the Wealthy Virtue get 20 experience points per year, while characters with the
/// Poor Flaw get 10 … Note that only companions can take this Virtue or Flaw"),
/// `:5235-5238` (Wealthy), `:6594-6596` (Poor: "this Flaw is not available to
/// magi").
#[test]
fn wealthy_and_poor_ship_with_their_rates_and_eligibility() {
    let rs = load_ruleset();

    let rate_of = |id: &str| -> u32 {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} ships"));
        item.effects
            .iter()
            .find_map(|e| match e {
                Effect::LaterLifeXpRate { amount } => Some(*amount),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{id} carries a later-life rate"))
    };
    assert_eq!(rate_of("virtue.wealthy"), 20);
    assert_eq!(rate_of("flaw.poor"), 10);

    // Both are Major, per their own entries.
    for id in ["virtue.wealthy", "flaw.poor"] {
        assert_eq!(
            rs.item(&Id::new(id)).unwrap().magnitude,
            Magnitude::Major,
            "{id} is a Major Virtue/Flaw"
        );
    }

    // "Only companions can take this Virtue or Flaw." A grog is covered
    // incidentally (its profile allows no Major V/F at all), so the two profiles
    // that would otherwise permit them must forbid them outright.
    for type_id in ["magus", "mythic_companion"] {
        let profile = rs.profile(&Id::new(type_id)).expect("profile ships");
        for id in ["virtue.wealthy", "flaw.poor"] {
            assert!(
                profile.forbidden_traits.contains(&Id::new(id)),
                "{type_id} must forbid {id} (Ars Magica - Definitive Edition (Core Rules).md:2394)"
            );
        }
    }
    // The grog case, stated so the incidental cover is deliberate rather than luck.
    assert_eq!(
        rs.profile(&Id::new("grog")).unwrap().budget.max_major_flaws,
        Some(0),
        "a grog takes no Major Flaw, so Poor is out of reach without a forbid"
    );
}

/// Foreign Upbringing halves the creation cap on locality-dependent Abilities, so
/// the Flaw must carry the fraction and the three Ability families the passage names
/// must carry the flag.
///
/// > The maximum scores at character creation for locality-dependent Abilities like
/// > Language, Area Lore, or Organization Lore, as well as some social Abilities, are
/// > half (round up) that which his age normally allows.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:6160. The trailing "some
/// social Abilities" is deliberately unflagged — see RULES.md.
#[test]
fn foreign_upbringing_halves_the_locality_dependent_abilities() {
    let rs = load_ruleset();

    let flaw = rs
        .item(&Id::new("flaw.foreign_upbringing"))
        .expect("flaw.foreign_upbringing ships");
    let fraction = flaw
        .effects
        .iter()
        .find_map(|e| match e {
            Effect::LocalityAbilityCapFraction { num, den } => Some((*num, *den)),
            _ => None,
        })
        .expect("it carries the cap fraction");
    assert_eq!(fraction, (1, 2), "half");

    for id in [
        "ability.living_language",
        "ability.dead_language",
        "ability.area_lore",
        "ability.organization_lore",
    ] {
        assert!(
            rs.ability(&Id::new(id))
                .expect("ability ships")
                .locality_dependent,
            "{id} is locality-dependent (Ars Magica - Definitive Edition (Core Rules).md:6160)"
        );
    }
    // A plainly non-local Ability is not flagged, so the fraction has a real edge.
    assert!(
        !rs.ability(&Id::new("ability.brawl"))
            .unwrap()
            .locality_dependent
    );
}

#[test]
fn shipped_abilities_and_characteristics_load() {
    let rs = load_ruleset();
    // Catalogue size is data, not code: assert the items the engine relies
    // on are present and read correctly, never an exact ability total. `main`
    // ships the full 78-ability Core Rules catalogue; the check stays
    // total-agnostic so a future catalogue edit needs no test change — see
    // crates/arm-rules/RULES.md.
    assert!(rs.ability(&Id::new("ability.awareness")).is_some());
    // The whole childhood restricted list ships (Core Rules 2378).
    for id in [
        "ability.area_lore",
        "ability.athletics",
        "ability.awareness",
        "ability.brawl",
        "ability.charm",
        "ability.folk_ken",
        "ability.guile",
        "ability.living_language",
        "ability.stealth",
        "ability.survival",
        "ability.swim",
    ] {
        assert!(rs.ability(&Id::new(id)).is_some(), "missing {id}");
    }

    // Supernatural Abilities ship and carry the supernatural category.
    for id in [
        "ability.second_sight",
        "ability.premonitions",
        "ability.animal_ken",
    ] {
        let ability = rs
            .ability(&Id::new(id))
            .expect("supernatural ability present");
        assert_eq!(
            ability.category,
            AbilityCategory::Supernatural,
            "{id} must be supernatural"
        );
    }

    // The `*` marker is the per-ability `requires_training` flag (cannot be used
    // untrained), NOT the supernatural category: it spans General/Academic/Arcane
    // too. Source: Ars Magica - Definitive Edition (Core Rules).md:4157
    // (Jack of All Trades) — heading asterisks set it.
    for (id, expected) in [
        ("ability.artes_liberales", true), // Academic, asterisked
        ("ability.magic_theory", true),    // Arcane, asterisked
        ("ability.area_lore", true),       // General, asterisked
        ("ability.second_sight", true),    // Supernatural, asterisked
        ("ability.awareness", false),      // General, usable untrained
        ("ability.penetration", false),    // Arcane but explicitly not asterisked
    ] {
        let ability = rs.ability(&Id::new(id)).expect("ability present");
        assert_eq!(
            ability.requires_training, expected,
            "{id} requires_training must be {expected}"
        );
    }

    let chars = rs
        .characteristic_rules()
        .expect("characteristic rules present");
    assert_eq!(chars.start_points, 7);
    // The cost table spans the absolute ±5 range (Great/Poor headroom)...
    assert_eq!(chars.min_score(), Some(-5));
    assert_eq!(chars.max_score(), Some(5));
    // ...while the no-virtue base limits are ±3.
    assert_eq!(chars.base_max_score(), Some(3));
    assert_eq!(chars.base_min_score(), Some(-3));
    assert_eq!(chars.effective_max_score(), Some(5));
    assert_eq!(chars.effective_min_score(), Some(-5));

    // Advancement table is triangular: score 5 costs 75 xp total.
    assert_eq!(rs.advancement().xp_for_score(5), Some(75));
    assert_eq!(rs.advancement().xp_to_raise(5), Some(25));
}

#[test]
fn english_i18n_covers_all_abilities() {
    let rs = load_ruleset();
    let i18n_en = include_str!("../../../rules/i18n/en/abilities.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for ability in rs.abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "English i18n missing ability '{}'",
            ability.id
        );
    }
}

#[test]
fn german_i18n_covers_all_abilities() {
    let rs = load_ruleset();
    let i18n_de = include_str!("../../../rules/i18n/de/abilities.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for ability in rs.abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "German i18n missing ability '{}'",
            ability.id
        );
    }
}

/// The shipped Sample Childhood catalogue, read as text so the tests below can
/// check the *file's* own canonical order as well as what the engine parses out
/// of it.
const SHIPPED_CHILDHOODS: &str = include_str!("../../../rules/core/childhoods.json");

/// Every shipped Sample Childhood package spends exactly the two childhood blocks
/// it is a shortcut for: 45 experience points across the spread and 75 in the
/// native language.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2378 (the two blocks),
/// `:2384-2388` (the five packages), priced off the "ABILITY To Buy" column at
/// `:2406-2427`.
///
/// The two figures are deliberately **literals**. `Ruleset::validate_childhood_packages`
/// already prices every package at load, but against `rules/core/life_stages.json`
/// — so an edit that lowered the block and a package together would pass the load
/// in silence. These literals are the outside witness that keeps the transcription
/// honest: they come from the rulebook line, not from another JSON file.
#[test]
fn every_shipped_childhood_package_prices_to_45_and_75() {
    let rs = load_full_ruleset();
    assert!(
        rs.childhoods().next().is_some(),
        "the shipped ruleset must offer Sample Childhood packages"
    );
    for package in rs.childhoods() {
        assert_eq!(
            package.spread_xp(rs.advancement()),
            Some(45),
            "package '{}' must spread exactly 45 experience points",
            package.id
        );
        assert_eq!(
            package.native_xp(rs.advancement()),
            Some(75),
            "package '{}' must spend exactly 75 on its native language",
            package.id
        );
    }
}

/// The shipped apprenticeship block carries the rulebook's own numbers.
///
/// > The fifteen years of apprenticeship give the character 240 experience points
/// > … Magi must have the following minimum Abilities: Parma Magica 1, Magic
/// > Theory 1, Latin 1.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2435 (the years and the
/// experience), `:2437` (the three minimums), `:2451-2461` (the four recommended
/// Abilities and their "Total Cost: 90 experience points").
///
/// Every figure is a **literal off the rulebook line**, for the same reason the
/// childhood packages are priced with literals above: the load-time check prices the
/// recommended list against `recommended_xp` in the same file, so an edit that moved
/// both together would pass in silence. These literals are the outside witness.
#[test]
fn shipped_apprenticeship_carries_the_2435_and_2437_numbers() {
    let rs = load_full_ruleset();
    let apprenticeship = rs
        .life_stages()
        .and_then(|rules| rules.apprenticeship.as_ref())
        .expect("the shipped life stages declare an apprenticeship");

    assert_eq!(apprenticeship.years, 15, "\"The fifteen years\" (:2435)");
    assert_eq!(apprenticeship.xp, 240, "\"240 experience points\" (:2435)");

    // "Parma Magica 1, Magic Theory 1, Latin 1" (:2437). Latin is one instance of
    // the parameterized dead-language Ability, matched by id (see RULES.md), so no
    // requirement names a parameter.
    let stated: Vec<(&str, u8, Option<&str>)> = apprenticeship
        .minimum_abilities
        .iter()
        .map(|r| (r.ability.as_str(), r.min_score, r.parameter.as_deref()))
        .collect();
    assert_eq!(
        stated,
        vec![
            ("ability.dead_language", 1, None),
            ("ability.magic_theory", 1, None),
            ("ability.parma_magica", 1, None),
        ]
    );

    // "Artes Liberales 1 / Latin 4 / Magic Theory 3 / Parma Magica 1" (:2453-2459).
    let recommended: Vec<(&str, u8, Option<&str>)> = apprenticeship
        .recommended_abilities
        .iter()
        .map(|r| (r.ability.as_str(), r.min_score, r.parameter.as_deref()))
        .collect();
    assert_eq!(
        recommended,
        vec![
            ("ability.artes_liberales", 1, None),
            ("ability.dead_language", 4, None),
            ("ability.magic_theory", 3, None),
            ("ability.parma_magica", 1, None),
        ]
    );
    assert_eq!(
        apprenticeship.recommended_xp, 90,
        "\"Total Cost: 90 experience points\" (:2461)"
    );
}

/// Two known packages load with their entries and provenance intact — never a
/// package total, which is data (a ruleset may ship any number of packages).
/// Athletic is the plain shape, Traveling the one that exercises every feature at
/// once: two instances of one parameterized Ability and a spread language beside
/// the native one.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2384 (Athletic),
/// `:2388` (Traveling).
#[test]
fn known_childhood_packages_ship_with_their_entries_and_provenance() {
    let rs = load_full_ruleset();

    let athletic = rs
        .childhood(&Id::new("childhood.athletic"))
        .expect("childhood.athletic ships");
    let entries: Vec<(&str, Option<&str>, u8, bool)> = athletic
        .entries
        .iter()
        .map(|e| (e.ability.as_str(), e.slot.as_deref(), e.score, e.native))
        .collect();
    assert_eq!(
        entries,
        vec![
            ("ability.athletics", None, 2, false),
            ("ability.brawl", None, 2, false),
            ("ability.living_language", None, 5, true),
            ("ability.swim", None, 2, false),
        ],
        "Athletics 2, Brawl 2, Native Language 5, Swim 2 (Ars Magica - Definitive Edition (Core Rules).md:2384)"
    );
    let source = athletic
        .source
        .as_ref()
        .expect("Athletic carries provenance");
    assert_eq!(
        source.file,
        "Ars Magica - Definitive Edition (Core Rules).md"
    );
    assert_eq!((source.lines.start, source.lines.end), (2384, 2384));

    let traveling = rs
        .childhood(&Id::new("childhood.traveling"))
        .expect("childhood.traveling ships");
    let slots: Vec<(&str, &str)> = traveling
        .slots()
        .map(|(slot, ability)| (slot, ability.as_str()))
        .collect();
    assert_eq!(
        slots,
        vec![
            ("area_a", "ability.area_lore"),
            ("area_b", "ability.area_lore"),
            ("language", "ability.living_language"),
        ],
        "Area A Lore, Area B Lore and the spread Living Language are asked for \
         (Ars Magica - Definitive Edition (Core Rules).md:2388)"
    );
    // The native language is never a slot: it is chosen once per character.
    let native = traveling.native_entry().expect("a native-language entry");
    assert_eq!(native.ability, Id::new("ability.living_language"));
    assert_eq!(native.score, 5);
    assert!(native.slot.is_none());
    assert_eq!(
        traveling
            .source
            .as_ref()
            .map(|s| (s.lines.start, s.lines.end)),
        Some((2388, 2388))
    );
}

/// The shipped file is canonically ordered: packages by id, each package's entries
/// by `(ability, slot)`.
///
/// This has to be checked here because nothing else can. `ChildhoodEntry` order is
/// deliberately *preserved* on load — it is the order a UI asks for slot values in
/// — so a mis-sorted file parses and validates perfectly happily, and the only
/// symptom would be a noisy git diff the next time the catalogue is regenerated.
#[test]
fn the_shipped_childhoods_file_is_canonically_ordered() {
    let file: serde_json::Value =
        serde_json::from_str(SHIPPED_CHILDHOODS).expect("the shipped catalogue is valid JSON");
    let packages = file["packages"]
        .as_array()
        .expect("the file carries a package list");

    let ids: Vec<&str> = packages
        .iter()
        .map(|p| p["id"].as_str().expect("every package has an id"))
        .collect();
    let mut id_sorted = ids.clone();
    id_sorted.sort_unstable();
    assert_eq!(ids, id_sorted, "packages must be id-sorted");

    for package in packages {
        // An absent slot sorts before any present one, which is what puts the
        // native-language entry ahead of Traveling's slotted second language.
        let keys: Vec<(&str, &str)> = package["entries"]
            .as_array()
            .expect("every package has entries")
            .iter()
            .map(|e| {
                (
                    e["ability"].as_str().expect("every entry names an ability"),
                    e["slot"].as_str().unwrap_or(""),
                )
            })
            .collect();
        let mut key_sorted = keys.clone();
        key_sorted.sort_unstable();
        assert_eq!(
            keys, key_sorted,
            "entries of '{}' must be sorted by (ability, slot)",
            package["id"]
        );
    }
}

#[test]
fn english_i18n_covers_all_childhood_packages() {
    let rs = load_full_ruleset();
    let i18n_en = include_str!("../../../rules/i18n/en/childhoods.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for package in rs.childhoods() {
        assert!(
            loc.display_name(&package.id).is_some(),
            "English i18n missing childhood package '{}'",
            package.id
        );
    }
}

#[test]
fn german_i18n_covers_all_childhood_packages() {
    let rs = load_full_ruleset();
    let i18n_de = include_str!("../../../rules/i18n/de/childhoods.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for package in rs.childhoods() {
        assert!(
            loc.display_name(&package.id).is_some(),
            "German i18n missing childhood package '{}'",
            package.id
        );
    }
}

#[test]
fn english_i18n_covers_all_spells() {
    let rs = load_ruleset_with_spells();
    let i18n_en = include_str!("../../../rules/i18n/en/spells.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for spell in rs.spells() {
        // A spell tooltip renders the description, so a missing English
        // description leaves an empty tooltip — assert both name and description
        // cover every spell id (fallback-free: this is the raw en file).
        assert!(
            loc.display_name(&spell.id).is_some(),
            "English i18n missing spell name '{}'",
            spell.id
        );
        assert!(
            loc.description(&spell.id).is_some(),
            "English i18n missing spell description '{}'",
            spell.id
        );
    }
}

#[test]
fn german_i18n_covers_all_spells() {
    let rs = load_ruleset_with_spells();
    let i18n_de = include_str!("../../../rules/i18n/de/spells.json");
    // Raw German file (fallback-free `new`): a missing German description here is
    // a real gap. The app layer falls back to English at load, but this gate
    // asserts the shipped German data itself covers every spell — the check that
    // was name-only before and let empty German tooltips ship.
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for spell in rs.spells() {
        assert!(
            loc.display_name(&spell.id).is_some(),
            "German i18n missing spell name '{}'",
            spell.id
        );
        assert!(
            loc.description(&spell.id).is_some(),
            "German i18n missing spell description '{}'",
            spell.id
        );
    }
}

/// The four meta-magic Vim spells whose target `(Form)` is a selection each
/// declare a single `form`-domain parameter, keep their catalogue Vim
/// Technique/Form (the parameter is display + identity only), and the whole
/// shipped catalogue still passes load-time referential integrity.
/// Source: Ars Magica - Definitive Edition (Core Rules).md:15776-15779,
/// :15791-15794, :15801-15804, :15843-15846.
#[test]
fn parametrized_vim_spells_declare_a_form_parameter() {
    let rs = load_ruleset_with_spells();
    for id in [
        "spell.mirror_of_opposition_form",
        "spell.unravelling_the_fabric_of_form",
        "spell.wizards_boost_form",
        "spell.wizards_reach_form",
    ] {
        let spell = rs
            .spell(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} present"));
        assert_eq!(spell.parameters.len(), 1, "{id} has one parameter");
        let def = &spell.parameters[0];
        assert_eq!(def.key, "form", "{id} parameter key is 'form'");
        assert_eq!(def.domain, ParameterDomain::Form, "{id} domain is Form");
        // The spell's own Technique/Form stay the catalogue Vim Arts.
        assert_eq!(spell.form, Id::new("art.vim"), "{id} form stays Vim");
    }
}

#[test]
fn shipped_equipment_loads_and_exposes_accessors() {
    let rs = load_ruleset_with_equipment();
    // Catalogue size is data, not code: assert representative items are present
    // and read correctly, never exact totals.
    let sword = rs
        .weapon(&Id::new("weapon.sword_long"))
        .expect("weapon.sword_long present");
    assert_eq!(sword.attack_mod, Some(4));
    assert_eq!(sword.damage_mod, Some(6));
    assert_eq!(sword.ability, Id::new("ability.single_weapon"));
    assert!(sword.range.is_none(), "melee weapon has no range");
    // A missile weapon carries a Range and uses Bows.
    let bow = rs
        .weapon(&Id::new("weapon.bow_long"))
        .expect("weapon.bow_long present");
    assert_eq!(bow.range, Some(30));
    assert_eq!(bow.ability, Id::new("ability.bows"));
    // Dodge has no attack/damage/min-Strength (n/a cells).
    let dodge = rs.weapon(&Id::new("weapon.dodge")).expect("dodge present");
    assert_eq!(dodge.attack_mod, None);
    assert_eq!(dodge.min_strength, None);
    // Shields and armor resolve through their own accessors.
    assert_eq!(rs.shield(&Id::new("shield.heater")).unwrap().defense_mod, 3);
    assert_eq!(
        rs.armor_item(&Id::new("armor.chain_mail_full"))
            .unwrap()
            .protection,
        9
    );
    assert!(rs.weapon_count() > 0 && rs.shield_count() > 0 && rs.armor_count() > 0);
}

/// A weapon whose combat `ability` names something that is neither Martial nor
/// Brawl is rejected at load (referential-integrity trust gate).
#[test]
fn weapon_with_non_combat_ability_rejected_at_load() {
    let equipment = r#"{ "weapons": [
      { "id": "weapon.bad", "kind": "melee", "init_mod": 0, "defense_mod": 0,
        "load": 1, "ability": "ability.awareness" }
    ] }"#;
    let err = Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: None,
        houses: None,
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: None,
        equipment: Some(equipment),
        characteristics: None,
        life_stages: None,
        childhoods: None,
        aging: None,
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("not a combat Ability"),
        "expected combat-ability rejection, got: {err}"
    );
}

/// A weapon naming a wholly unknown ability id is also rejected at load.
#[test]
fn weapon_with_unknown_ability_rejected_at_load() {
    let equipment = r#"{ "weapons": [
      { "id": "weapon.bad", "kind": "melee", "init_mod": 0, "defense_mod": 0,
        "load": 1, "ability": "ability.nonexistent" }
    ] }"#;
    let err = Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: None,
        houses: None,
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: None,
        equipment: Some(equipment),
        characteristics: None,
        life_stages: None,
        childhoods: None,
        aging: None,
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("unknown ability"),
        "expected unknown-ability rejection, got: {err}"
    );
}

#[test]
fn english_i18n_covers_all_mastery_abilities() {
    let rs = load_ruleset_with_mastery_abilities();
    let i18n_en = include_str!("../../../rules/i18n/en/spell_mastery_abilities.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for ability in rs.spell_mastery_abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "English i18n missing mastery-ability name '{}'",
            ability.id
        );
        assert!(
            loc.description(&ability.id).is_some(),
            "English i18n missing mastery-ability description '{}'",
            ability.id
        );
    }
}

#[test]
fn german_i18n_covers_all_mastery_abilities() {
    let rs = load_ruleset_with_mastery_abilities();
    let i18n_de = include_str!("../../../rules/i18n/de/spell_mastery_abilities.json");
    // Raw German file (fallback-free `new`): a missing German entry here is a real
    // gap, mirroring the spell coverage gate.
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for ability in rs.spell_mastery_abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "German i18n missing mastery-ability name '{}'",
            ability.id
        );
        assert!(
            loc.description(&ability.id).is_some(),
            "German i18n missing mastery-ability description '{}'",
            ability.id
        );
    }
}

#[test]
fn english_i18n_covers_all_equipment() {
    let rs = load_ruleset_with_equipment();
    let i18n_en = include_str!("../../../rules/i18n/en/equipment.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for id in rs
        .weapons()
        .map(|w| &w.id)
        .chain(rs.shields().map(|s| &s.id))
        .chain(rs.armor().map(|a| &a.id))
    {
        assert!(
            loc.display_name(id).is_some(),
            "English i18n missing equipment '{id}'"
        );
    }
}

#[test]
fn german_i18n_covers_all_equipment() {
    let rs = load_ruleset_with_equipment();
    let i18n_de = include_str!("../../../rules/i18n/de/equipment.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for id in rs
        .weapons()
        .map(|w| &w.id)
        .chain(rs.shields().map(|s| &s.id))
        .chain(rs.armor().map(|a| &a.id))
    {
        assert!(
            loc.display_name(id).is_some(),
            "German i18n missing equipment '{id}'"
        );
    }
}

/// An [`EquipmentSlot`] referencing an unknown catalogue id errors in validate;
/// an over-heavy equipped weapon warns (advisory, non-blocking); normalize sorts.
#[test]
fn validate_equipment_unknown_ref_and_min_strength() {
    let rs = load_ruleset_with_equipment();
    let mut e = entity("grog", vec![]);
    // Unknown id → error.
    e.equipment = vec![EquipmentSlot {
        item: Id::new("weapon.nonexistent"),
        equipped: true,
        specialization_applies: false,
    }];
    let result = validate(&e, &rs);
    assert!(
        result.issues.iter().any(|i| i.code == "unknown_equipment"),
        "unknown equipment id must error"
    );

    // A Warhammer (min-Strength +2) equipped by a Strength −1 grog warns, not errors.
    e.characteristics.insert(Characteristic::Str, -1);
    e.equipment = vec![EquipmentSlot {
        item: Id::new("weapon.warhammer"),
        equipped: true,
        specialization_applies: false,
    }];
    let result = validate(&e, &rs);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "equipment_min_strength"
                && i.severity == arm_rules::IssueSeverity::Warning),
        "over-heavy equipped weapon must warn"
    );
    assert!(
        !result.issues.iter().any(|i| i.code == "unknown_equipment"),
        "a known weapon must not report unknown_equipment"
    );
}

/// Exercises the shield and armor arms of `validate_equipment`. An equipped shield
/// whose min-Strength exceeds the wielder's Strength raises the same advisory
/// `equipment_min_strength` warning as a weapon (shield arm), while armor — which
/// carries no min-Strength requirement — never warns however weak the wearer (armor
/// arm's `None`). Ars Magica - Definitive Edition (Core Rules).md:16997.
#[test]
fn validate_equipment_shield_warns_and_armor_never_warns() {
    let rs = load_ruleset_with_equipment();
    let mut e = entity("grog", vec![]);

    // A Heater shield (min-Strength 0) equipped by a Strength −1 grog warns.
    e.characteristics.insert(Characteristic::Str, -1);
    e.equipment = vec![EquipmentSlot {
        item: Id::new("shield.heater"),
        equipped: true,
        specialization_applies: false,
    }];
    let result = validate(&e, &rs);
    let warnings: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code == "equipment_min_strength")
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "exactly one shield min-Strength advisory"
    );
    let warning = warnings[0];
    assert_eq!(warning.severity, arm_rules::IssueSeverity::Warning);
    assert_eq!(
        warning.args.get("item").map(String::as_str),
        Some("shield.heater")
    );
    assert_eq!(warning.args.get("required").map(String::as_str), Some("0"));
    assert_eq!(warning.args.get("strength").map(String::as_str), Some("-1"));
    assert_eq!(warning.context.as_ref(), Some(&Id::new("shield.heater")));

    // Full chain mail equipped by a much weaker grog never warns: armor has no
    // min-Strength requirement (armor arm returns `None`), so no advisory is raised
    // and a known armor id must not be reported as unknown.
    e.characteristics.insert(Characteristic::Str, -5);
    e.equipment = vec![EquipmentSlot {
        item: Id::new("armor.chain_mail_full"),
        equipped: true,
        specialization_applies: false,
    }];
    let result = validate(&e, &rs);
    assert!(
        !result
            .issues
            .iter()
            .any(|i| i.code == "equipment_min_strength"),
        "armor carries no min-Strength requirement, so it must never warn"
    );
    assert!(
        !result.issues.iter().any(|i| i.code == "unknown_equipment"),
        "a known armor id must not report unknown_equipment"
    );
}

/// Issue B: equipping a shield alongside ONLY two-handed weapon(s) raises the
/// advisory `shield_with_two_handed_weapon` warning (the shield's modifiers are
/// dropped, which looks like a bug otherwise). A one-handed weapon in the mix
/// clears the advisory, since the shield is usable with it. Non-blocking.
/// Ars Magica - Definitive Edition (Core Rules).md:7494, :17063, :16975.
#[test]
fn shield_with_only_two_handed_weapons_warns() {
    let rs = load_ruleset_with_equipment();
    let mut e = entity("grog", vec![]);
    e.characteristics.insert(Characteristic::Str, 3);

    // A great sword (two-handed) + a shield → advisory warning.
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("weapon.sword_great"),
            equipped: true,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("shield.heater"),
            equipped: true,
            specialization_applies: false,
        },
    ];
    let result = validate(&e, &rs);
    let warning = result
        .issues
        .iter()
        .find(|i| i.code == "shield_with_two_handed_weapon")
        .expect("shield + two-handed-only must warn");
    assert_eq!(warning.severity, arm_rules::IssueSeverity::Warning);

    // Add a one-handed weapon: the shield is now usable, so the advisory clears.
    e.equipment.push(EquipmentSlot {
        item: Id::new("weapon.sword_long"),
        equipped: true,
        specialization_applies: false,
    });
    let result = validate(&e, &rs);
    assert!(
        !result
            .issues
            .iter()
            .any(|i| i.code == "shield_with_two_handed_weapon"),
        "a one-handed weapon makes the shield usable — no advisory"
    );
}

/// Issue C: `specialization_applies` is a canonical, additive field — it survives
/// a JSON round-trip and participates in the derived `Ord`, so `normalize` sorts
/// deterministically on it and it serializes only when true (skip-when-false).
#[test]
fn specialization_applies_round_trips_and_sorts() {
    // Skip-when-false keeps the JSON noise-free; true is written.
    let off = EquipmentSlot {
        item: Id::new("weapon.sword_long"),
        equipped: true,
        specialization_applies: false,
    };
    let off_json = serde_json::to_string(&off).unwrap();
    assert!(
        !off_json.contains("specialization_applies"),
        "false is skipped: {off_json}"
    );
    let on = EquipmentSlot {
        specialization_applies: true,
        ..off.clone()
    };
    let on_json = serde_json::to_string(&on).unwrap();
    assert!(on_json.contains("specialization_applies"));
    assert_eq!(serde_json::from_str::<EquipmentSlot>(&on_json).unwrap(), on);

    // The field joins the derived Ord: two otherwise-identical slots order with
    // `false` before `true`, so normalize is deterministic.
    let mut e = entity("grog", vec![]);
    e.equipment = vec![on.clone(), off.clone()];
    e.normalize();
    assert!(!e.equipment[0].specialization_applies);
    assert!(e.equipment[1].specialization_applies);
}

#[test]
fn normalize_sorts_equipment() {
    let mut e = entity("grog", vec![]);
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("weapon.warhammer"),
            equipped: false,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("armor.chain_mail_full"),
            equipped: true,
            specialization_applies: false,
        },
    ];
    e.normalize();
    assert_eq!(e.equipment[0].item, Id::new("armor.chain_mail_full"));
    assert_eq!(e.equipment[1].item, Id::new("weapon.warhammer"));
}

/// The shipped Skilled Parens raises *both* the spell-levels budget (+30) and the
/// general apprenticeship XP pool (+60), proving its two-effect package is wired
/// end-to-end against real data (Ars Magica - Definitive Edition (Core Rules).md:4964-4966).
#[test]
fn shipped_skilled_parens_raises_both_budgets() {
    let rs = load_ruleset_with_spells();
    let mut e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.skilled_parens"))],
    );
    e.xp_pool = 240;
    assert_eq!(arm_rules::spell_levels_budget(120, &e, &rs), 150);
    assert_eq!(
        arm_rules::checked_xp_allocation(&e, &rs)
            .unwrap()
            .general_pool,
        300
    );
}

/// The shipped Elemental Magic (Ars Magica - Definitive Edition (Core Rules).md:3731-3737) redistributes Art-XP over the four
/// elemental Forms against the real Arts catalogue: each Form gains half (rounded
/// up) of every other Form's table-XP. With Ignem/Auram/Terram at score 6 (21 XP)
/// and Aquam at score 4 (10 XP), the boosted effective scores are 9/9/9 and 8,
/// while a non-elemental Form (Corpus) at score 6 is untouched.
#[test]
fn shipped_elemental_magic_redistributes_art_xp() {
    let rs = load_ruleset_with_spells();
    let mut e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.elemental_magic"))],
    );
    let row = |art: &str, score: u8| ArtScore {
        art: Id::new(art),
        score,
    };
    e.art_scores = vec![
        row("art.aquam", 4),
        row("art.auram", 6),
        row("art.ignem", 6),
        row("art.terram", 6),
        row("art.corpus", 6),
    ];
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.aquam")), 8);
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.auram")), 9);
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.ignem")), 9);
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.terram")), 9);
    // A non-elemental Form is never touched by the redistribution.
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.corpus")), 6);
}

/// The shipped Weak Parens lowers both budgets (Ars Magica - Definitive Edition (Core Rules).md:7072-7074).
#[test]
fn shipped_weak_parens_lowers_both_budgets() {
    let rs = load_ruleset_with_spells();
    let mut e = entity("magus", vec![Selection::new(Id::new("flaw.weak_parens"))]);
    e.xp_pool = 240;
    assert_eq!(arm_rules::spell_levels_budget(120, &e, &rs), 90);
    assert_eq!(
        arm_rules::checked_xp_allocation(&e, &rs)
            .unwrap()
            .general_pool,
        180
    );
}

#[test]
fn fully_specified_companion_validates() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );
    // Int +2 (3) + Per +1 (1) + Sta -1 (-1) + others 0 = 3 <= 7 (under -> warning only).
    e.characteristics = BTreeMap::from([
        (Characteristic::Int, 2),
        (Characteristic::Per, 1),
        (Characteristic::Sta, -1),
    ]);
    e.ability_scores = vec![
        AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2,
            specialty: Some("searching".into()),
            parameter: None,
        },
        AbilityScore {
            ability: Id::new("ability.living_language"),
            score: 5,
            specialty: None,
            parameter: Some("German".into()),
        },
    ];
    // Awareness 2 (15 xp) + Living Language 5 (75 xp) = 90 spent; give a pool that
    // covers it (banking the rest).
    e.xp_pool = 120;

    let result = validate(&e, &rs);
    assert!(
        result.is_valid(),
        "fully-specified companion should validate: {:?}",
        result.issues
    );
}

/// Acceptance criterion for M5 slice 5a: every shipped Virtue/Flaw carries a
/// `classification`. The field is required (no serde default), so an unclassified
/// entry would already fail `load_ruleset()`; this test additionally asserts the
/// catalogue is non-trivial and that all three classes are actually used, so the
/// classification pass can never silently collapse to a single bucket.
#[test]
fn every_vf_is_classified() {
    let rs = load_ruleset();
    let mut narrative = 0usize;
    let mut creation = 0usize;
    let mut in_play = 0usize;
    for item in rs.items() {
        match item.classification {
            Classification::Narrative => narrative += 1,
            Classification::CreationEffect => creation += 1,
            Classification::InPlayEffect => in_play += 1,
        }
    }
    // Catalogue size is data, not code: assert only that the catalogue is large
    // and every class is represented, never exact per-class totals.
    assert!(
        narrative + creation + in_play > 600,
        "expected the full V/F catalogue to load"
    );
    assert!(narrative > 0, "some V/F must be narrative");
    assert!(creation > 0, "some V/F must be creation_effect");
    assert!(in_play > 0, "some V/F must be in_play_effect");
    // An entry carrying `effects` is mechanical, never narrative: it either
    // changes a creation number (creation_effect) or modifies an in-play/derived
    // total (in_play_effect, M5/5b). Narrative items are never given an effect.
    for item in rs.items() {
        if !item.effects.is_empty() {
            assert_ne!(
                item.classification,
                Classification::Narrative,
                "{} carries effects so must not be narrative",
                item.id
            );
        }
    }
    // M5/5b acceptance: every in_play_effect V/F is wired to at least one
    // derived-total Effect variant (the 93-item in-play audit is fully wired).
    for item in rs.items() {
        if item.classification == Classification::InPlayEffect {
            assert!(
                !item.effects.is_empty(),
                "{} is in_play_effect but carries no effect",
                item.id
            );
        }
    }
}

#[test]
fn english_i18n_covers_all_items() {
    let rs = load_ruleset();
    let i18n_en = include_str!("../../../rules/i18n/en/virtues_flaws.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();

    for item in rs.items() {
        assert!(
            loc.display_name(&item.id).is_some(),
            "English i18n missing entry for '{}'",
            item.id
        );
    }
}

#[test]
fn german_i18n_covers_all_items() {
    let rs = load_ruleset();
    let i18n_de = include_str!("../../../rules/i18n/de/virtues_flaws.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();

    for item in rs.items() {
        assert!(
            loc.display_name(&item.id).is_some(),
            "German i18n missing entry for '{}'",
            item.id
        );
    }
}

#[test]
fn companion_balanced_entity_validates() {
    let rs = load_ruleset();

    let entity = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.keen_vision")),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );

    let result = validate(&entity, &rs);
    assert!(
        result.is_valid(),
        "balanced companion should validate: {:?}",
        result.issues
    );

    let balance = compute_balance(&entity, &rs);
    assert_eq!(balance.virtue_points, 1);
    assert_eq!(balance.flaw_points, 1);
}

#[test]
fn save_load_roundtrip_with_canonical_output() {
    let mut entity = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );
    // Canonical output requires normalize(): Serialize no longer auto-sorts.
    entity.normalize();

    let json1 = serde_json::to_string_pretty(&entity).unwrap();
    let roundtripped: Entity = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&roundtripped).unwrap();

    assert_eq!(json1, json2, "canonical serialization should be stable");
    assert_eq!(entity, roundtripped);
}

#[test]
fn grog_type_restricts_major_virtues() {
    let rs = load_ruleset();

    // One minor virtue funded by one minor flaw, so the points balance and
    // the test isolates the Major-virtue restriction.
    let entity = entity(
        "grog",
        vec![
            Selection::new(Id::new("virtue.keen_vision")),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );

    let result = validate(&entity, &rs);
    assert!(
        result.is_valid(),
        "grog with one balanced minor virtue should be valid: {:?}",
        result.issues
    );
}

#[test]
fn grog_over_budget() {
    let rs = load_ruleset();

    let entity = entity(
        "grog",
        vec![
            Selection::new(Id::new("virtue.keen_vision")),
            Selection::new(Id::new("virtue.large")),
            Selection::new(Id::new("virtue.tough")),
            Selection::new(Id::new("virtue.puissant_ability")),
        ],
    );

    let result = validate(&entity, &rs);
    let codes: Vec<&str> = result.errors().map(|i| i.code.as_str()).collect();
    assert!(
        codes.contains(&"over_budget_virtues"),
        "grog over budget: {codes:?}"
    );
}

#[test]
fn shipped_score_effects_apply() {
    use arm_rules::{characteristic_cap, characteristic_floor, effective_ability_score};
    let rs = load_ruleset();

    // Puissant Ability (+2 bonus) and Great/Poor Characteristic (limit shifts)
    // from the shipped data.
    let mut e = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::with_params(
                Id::new("virtue.great_characteristic"),
                BTreeMap::from([("characteristic".into(), Characteristic::Str.id())]),
            ),
            Selection::with_params(
                Id::new("flaw.poor_characteristic"),
                BTreeMap::from([("characteristic".into(), Characteristic::Qik.id())]),
            ),
        ],
    );
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.awareness"),
        score: 2,
        specialty: None,
        parameter: None,
    }];
    e.characteristics = BTreeMap::from([(Characteristic::Str, 3), (Characteristic::Qik, -3)]);

    // Puissant adds to the effective ability score.
    assert_eq!(
        effective_ability_score(&e, &rs, &Id::new("ability.awareness"), None),
        4,
        "Awareness 2 + Puissant +2"
    );
    // Great raises Strength's buy cap (no free point); Poor lowers Quickness's
    // buy floor. Untargeted characteristics keep the base ±3 limits.
    assert_eq!(
        characteristic_cap(&e, &rs, Characteristic::Str),
        4,
        "Great raises the Strength cap to +4"
    );
    assert_eq!(
        characteristic_floor(&e, &rs, Characteristic::Qik),
        -4,
        "Poor lowers the Quickness floor to -4"
    );
    assert_eq!(characteristic_cap(&e, &rs, Characteristic::Int), 3);
    assert_eq!(characteristic_floor(&e, &rs, Characteristic::Int), -3);
}

// --- M5 slice 5b: in-play effect variants ---

/// Helper: the set of issue codes `validate` emits for an entity.
fn issue_codes(entity: &Entity, rs: &Ruleset) -> Vec<String> {
    validate(entity, rs)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

/// A magus may hold at most one Magical Focus (Ars Magica - Definitive Edition (Core Rules).md:4542): two Minor
/// Foci (distinct descriptors, so not a duplicate selection) trip the
/// `multiple_magical_foci` rule, which counts the `MagicalFocus` effect rather
/// than relying on pairwise incompatibility (which cannot catch two Minors).
#[test]
fn two_magical_foci_are_rejected() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![
            Selection::with_params(
                Id::new("virtue.minor_magical_focus"),
                BTreeMap::from([("focus".into(), Id::new("necromancy"))]),
            ),
            Selection::with_params(
                Id::new("virtue.minor_magical_focus"),
                BTreeMap::from([("focus".into(), Id::new("weather"))]),
            ),
        ],
    );
    assert!(
        issue_codes(&e, &rs).contains(&"multiple_magical_foci".to_string()),
        "two foci must be rejected"
    );
}

/// A single Magical Focus is legal — the one-focus rule does not fire.
#[test]
fn one_magical_focus_is_allowed() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("virtue.major_magical_focus"),
            BTreeMap::from([("focus".into(), Id::new("necromancy"))]),
        )],
    );
    assert!(
        !issue_codes(&e, &rs).contains(&"multiple_magical_foci".to_string()),
        "one focus must be allowed"
    );
}

/// Deficient Form's parameter is Form-domain, so targeting a Technique (art.creo)
/// fails parameter resolution — the art-class restriction the slice requires.
#[test]
fn deficient_form_cannot_target_a_technique() {
    let rs = load_ruleset_with_spells();
    let e = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.creo"))]),
        )],
    );
    assert!(
        issue_codes(&e, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Form targeting a Technique must be rejected"
    );
    // A Form target (art.ignem) resolves cleanly.
    let ok = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.ignem"))]),
        )],
    );
    assert!(
        !issue_codes(&ok, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Form targeting a Form must be accepted"
    );
}

/// Deficient Technique's parameter is Technique-domain, so targeting a Form
/// (art.ignem) fails; a Technique (art.creo) resolves.
#[test]
fn deficient_technique_cannot_target_a_form() {
    let rs = load_ruleset_with_spells();
    let bad = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("technique".into(), Id::new("art.ignem"))]),
        )],
    );
    assert!(
        issue_codes(&bad, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Technique targeting a Form must be rejected"
    );
    let ok = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("technique".into(), Id::new("art.creo"))]),
        )],
    );
    assert!(
        !issue_codes(&ok, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Technique targeting a Technique must be accepted"
    );
}

/// In-play effects (Tough/Soak, Method Caster, Enduring Constitution, a Magical
/// Focus, Deficient Form) never perturb creation-legality totals: adding them
/// leaves the XP allocation, characteristic caps, and effective ability scores
/// exactly as they were. They cost/grant only the point-balance their magnitude
/// dictates, computed elsewhere.
#[test]
fn in_play_effects_do_not_perturb_creation_totals() {
    use arm_rules::{characteristic_cap, checked_xp_allocation, effective_ability_score};
    let rs = load_ruleset_with_spells();

    let mut base = entity("magus", vec![]);
    base.xp_pool = 15;
    base.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.awareness"),
        score: 3,
        specialty: None,
        parameter: None,
    }];
    base.characteristics = BTreeMap::from([(Characteristic::Int, 2)]);

    let mut with_effects = base.clone();
    with_effects.selections = vec![
        Selection::new(Id::new("virtue.tough")),
        Selection::new(Id::new("virtue.method_caster")),
        Selection::new(Id::new("virtue.enduring_constitution")),
        Selection::with_params(
            Id::new("virtue.major_magical_focus"),
            BTreeMap::from([("focus".into(), Id::new("necromancy"))]),
        ),
        Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.ignem"))]),
        ),
    ];

    assert_eq!(
        checked_xp_allocation(&base, &rs).unwrap().total_demand,
        checked_xp_allocation(&with_effects, &rs)
            .unwrap()
            .total_demand,
        "in-play effects must not change XP demand"
    );
    assert_eq!(
        characteristic_cap(&base, &rs, Characteristic::Int),
        characteristic_cap(&with_effects, &rs, Characteristic::Int),
        "in-play effects must not change characteristic caps"
    );
    assert_eq!(
        effective_ability_score(&base, &rs, &Id::new("ability.awareness"), None),
        effective_ability_score(&with_effects, &rs, &Id::new("ability.awareness"), None),
        "in-play effects must not change effective ability scores"
    );
}

// --- M5 slice 5a-wire: creation-effect wiring on the shipped catalogue ---

/// Helper: does an entity's reputation set raise `reputation_not_granted`?
fn reputation_ungranted(entity: &Entity, rs: &Ruleset) -> bool {
    issue_codes(entity, rs).contains(&"reputation_not_granted".to_string())
}

/// Builds a companion holding a single virtue/flaw (no params) plus one
/// player-declared Reputation of `kind`/`score`, to check the grant authorizes it.
fn companion_with_reputation(item: &str, kind: ReputationType, score: u8) -> Entity {
    let mut e = entity("companion", vec![Selection::new(Id::new(item))]);
    e.reputations = vec![Reputation {
        kind,
        score,
        content: "test".into(),
    }];
    e
}

#[test]
fn shipped_reputation_granters_authorize_their_kind() {
    let rs = load_ruleset();
    // Hermetic Prestige → a Hermetic Reputation at 4 (Ars Magica - Definitive Edition (Core Rules).md:4071-4073).
    let hp = companion_with_reputation("virtue.hermetic_prestige", ReputationType::Hermetic, 4);
    assert!(
        !reputation_ungranted(&hp, &rs),
        "Hermetic Prestige grants Hermetic"
    );
    // Baccalaureus → an Academic Reputation (Ars Magica - Definitive Edition (Core Rules).md:3472).
    let bac = companion_with_reputation("virtue.baccalaureus", ReputationType::Academic, 1);
    assert!(
        !reputation_ungranted(&bac, &rs),
        "Baccalaureus grants Academic"
    );
    // A Local reputation is NOT authorized by Hermetic Prestige alone.
    let wrong = companion_with_reputation("virtue.hermetic_prestige", ReputationType::Local, 4);
    assert!(
        reputation_ungranted(&wrong, &rs),
        "Hermetic Prestige does not grant Local"
    );
}

#[test]
fn shipped_famous_authorizes_any_reputation_kind() {
    let rs = load_ruleset();
    // Famous (Ars Magica - Definitive Edition (Core Rules).md:3861-3863): player chooses the type — any single type is legal.
    for kind in ReputationType::ALL {
        let e = companion_with_reputation("virtue.famous", kind, 4);
        assert!(
            !reputation_ungranted(&e, &rs),
            "Famous authorizes a {kind} Reputation",
        );
    }
    // But only ONE: two reputations exceed the single wildcard grant.
    let mut two = companion_with_reputation("virtue.famous", ReputationType::Local, 4);
    two.reputations.push(Reputation {
        kind: ReputationType::Hermetic,
        score: 4,
        content: "second".into(),
    });
    assert!(
        reputation_ungranted(&two, &rs),
        "Famous grants only one Reputation"
    );
}

#[test]
fn shipped_supernatural_virtues_grant_starting_score() {
    use arm_rules::effective_ability_score;
    let rs = load_ruleset();
    for (item, ability) in [
        ("virtue.animal_ken", "ability.animal_ken"),
        ("virtue.shapeshifter", "ability.shapeshifter"),
        ("virtue.enchanting_ability", "ability.enchanting"),
        ("virtue.wilderness_sense", "ability.wilderness_sense"),
    ] {
        let e = entity("companion", vec![Selection::new(Id::new(item))]);
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new(ability), None),
            1,
            "{item} grants {ability} at 1",
        );
    }
    // Strong Faerie Blood grants the Second Sight *Virtue* for free (Ars Magica - Definitive Edition (Core Rules).md:5038),
    // which in turn floors Second Sight at 1.
    let sfb = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.strong_faerie_blood"))],
    );
    assert_eq!(
        effective_ability_score(&sfb, &rs, &Id::new("ability.second_sight"), None),
        1,
        "Strong Faerie Blood grants Second Sight via a nested Virtue grant",
    );
}

#[test]
fn shipped_xp_granters_add_restricted_pool() {
    use arm_rules::restricted_xp_pools;
    let rs = load_ruleset();
    // Arcane Lore → +50 XP restricted to Arcane abilities (Ars Magica - Definitive Edition (Core Rules).md:3432).
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.arcane_lore"))],
    );
    let pools = restricted_xp_pools(&e, &rs);
    assert!(
        pools
            .iter()
            .any(|p| p.amount == 50 && p.categories.contains(&AbilityCategory::Arcane)),
        "Arcane Lore grants a 50-xp Arcane-restricted pool",
    );
    // Feral Upbringing → 120 XP on a fixed ability list (Ars Magica - Definitive Edition (Core Rules).md:6112).
    let fu = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.feral_upbringing"))],
    );
    assert!(
        restricted_xp_pools(&fu, &rs)
            .iter()
            .any(|p| p.amount == 120),
        "Feral Upbringing grants a 120-xp restricted pool",
    );
}

#[test]
fn shipped_confidence_true_faith_and_size_granters() {
    use arm_rules::{Confidence, confidence, size, true_faith};
    let rs = load_ruleset();
    // Ferocity → +1 Confidence Score / +3 Points (Ars Magica - Definitive Edition (Core Rules).md:3875) over the base.
    let fer = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.ferocity"))],
    );
    assert_eq!(
        confidence(1, 3, &fer, &rs),
        Confidence {
            score: 2,
            points: 6
        },
        "Ferocity adds 1/3"
    );
    // Low Self-Esteem → removes the standard 1/3 Confidence (Ars Magica - Definitive Edition (Core Rules).md:6364).
    let lse = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.low_self_esteem"))],
    );
    assert_eq!(
        confidence(1, 3, &lse, &rs),
        Confidence {
            score: 0,
            points: 0
        },
        "Low Self-Esteem zeroes Confidence"
    );
    // Relic → True Faith 1 (Ars Magica - Definitive Edition (Core Rules).md:4854); Powerful Relic → 3 (Ars Magica - Definitive Edition (Core Rules).md:4783).
    let relic = entity("companion", vec![Selection::new(Id::new("virtue.relic"))]);
    assert_eq!(true_faith(&relic, &rs), 1);
    let prelic = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.powerful_relic"))],
    );
    assert_eq!(true_faith(&prelic, &rs), 3);
    // Blood of the Nephilim → Size +1 (Divine:1945).
    let bon = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.blood_of_the_nephilim"))],
    );
    assert_eq!(
        size(&bon, &rs),
        1,
        "Blood of the Nephilim raises Size to +1"
    );
}

/// The full shipped ruleset — every catalogue loaded, exactly as the app ships
/// it (`ruleset_io.rs`). This is the production data the derived-totals end-to-end
/// test runs against.
fn load_full_ruleset() -> Ruleset {
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
        spell_mastery_abilities: None,
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: Some(SHIPPED_CHILDHOODS),
        aging: Some(SHIPPED_AGING),
    })
    .unwrap()
}

/// End-to-end proof of the M5 headline capability: a fully-specified magus built
/// on the **real shipped ruleset** is fully *computable* in direct entry. Wires
/// together the whole 5b→5i chain — aura, a Major Magical Focus, Method Caster,
/// Tough, Enduring Constitution, a weapon + shield + armor, a familiar with a
/// Bronze cord, a self-made Longevity Ritual, and stored warping / aging points —
/// then calls `derived_totals` and asserts every play-stat block is populated and
/// internally self-consistent. Numbers are pinned to the shipped catalogue values
/// (verified against `rules/core/*.json`).
#[test]
fn full_magus_derived_totals_are_populated_and_consistent() {
    let rs = load_full_ruleset();
    let mut e = entity(
        "magus",
        vec![
            Selection::new(Id::new("virtue.the_gift")),
            Selection::new(Id::new("virtue.hermetic_magus")),
            Selection::new(Id::new("virtue.method_caster")),
            Selection::new(Id::new("virtue.tough")),
            Selection::new(Id::new("virtue.enduring_constitution")),
            // A Major Magical Focus with a free-text descriptor ("fire").
            Selection::with_params(
                Id::new("virtue.major_magical_focus"),
                BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
            ),
        ],
    );

    // Characteristics.
    for (c, v) in [
        (Characteristic::Int, 3),
        (Characteristic::Sta, 2),
        (Characteristic::Str, 2),
        (Characteristic::Qik, 1),
        (Characteristic::Dex, 2),
    ] {
        e.characteristics.insert(c, v);
    }

    // Abilities the totals read (Magic Theory, Parma, Penetration, Single Weapon).
    let ab = |id: &str, score: u8| AbilityScore {
        ability: Id::new(id),
        parameter: None,
        specialty: None,
        score,
    };
    e.ability_scores = vec![
        ab("ability.magic_theory", 4),
        ab("ability.parma_magica", 3),
        ab("ability.penetration", 2),
        ab("ability.single_weapon", 4),
    ];

    // Arts: Creo 10, Ignem 8, Corpus 12.
    let art = |id: &str, score: u8| ArtScore {
        art: Id::new(id),
        score,
    };
    e.art_scores = vec![
        art("art.creo", 10),
        art("art.ignem", 8),
        art("art.corpus", 12),
    ];

    e.aura = 3;
    e.spells = vec![SpellSelection {
        spell: Id::new("spell.blade_of_the_virulent_flame"),
        level: None,
        mastery: None,
        parameter: None,
        mastery_abilities: Vec::new(),
    }];
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("weapon.axe"),
            equipped: true,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("shield.round"),
            equipped: true,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("armor.chain_mail_partial"),
            equipped: true,
            specialization_applies: false,
        },
    ];
    // A full familiar statblock: a raven (Size -4, Ars Magica - Definitive Edition (Core Rules).md:17829-17856) with Magic
    // Might 10, human intelligence at Int -3 (Ars Magica - Definitive Edition (Core Rules).md:10854), the bond's Loyal
    // (partner) +3 entered by hand, and one power invested in the bond.
    e.familiar = Some(Familiar {
        name: "Corvus".to_string(),
        animal: "raven".to_string(),
        might: Some(MightScore {
            realm: Realm::Magic,
            score: 10,
        }),
        characteristics: BTreeMap::from([(Characteristic::Int, -3), (Characteristic::Qik, 4)]),
        size: -4,
        personality_traits: vec![PersonalityTrait {
            name: "Loyal (Marcus)".to_string(),
            value: 3,
        }],
        cord_gold: 1,
        cord_silver: 1,
        cord_bronze: 2,
        powers: vec![SupernaturalPower {
            name: "Mental communication".to_string(),
            level: 20,
        }],
    });
    e.talisman = Some(Talisman {
        description: "An ash staff shod with silver".to_string(),
        attunements: vec![TalismanAttunement {
            description: "Controlling things at a distance".to_string(),
            bonus: 4,
        }],
        effects: vec![TalismanEffect {
            name: "Wielding the Invisible Sling".to_string(),
            level: 15,
        }],
    });
    e.longevity_ritual = Some(LongevityRitual {
        source: LongevitySource::SelfMade,
        bonus: Some(6),
        focus: "A tincture of gold sipped each Midsummer".to_string(),
    });
    e.warping_points = 15;
    // Decrepitude is the SUM of aging points across Characteristics; the drops they
    // force now lower the effective Characteristic (derived from aging_points), so
    // they are placed in Per/Com — which no asserted magic/combat total reads — to
    // exercise Decrepitude without perturbing those totals. 10 + 7 = 17 → Decrepitude 2.
    e.aging_points = BTreeMap::from([(Characteristic::Per, 10), (Characteristic::Com, 7)]);

    let d = arm_rules::derived_totals(&e, &rs);

    // --- Magic totals present (magus) ------------------------------------
    assert!(d.is_magus, "magus profile drives magic totals");
    assert!(!d.lab_totals.is_empty(), "lab totals populated");
    assert!(!d.casting_totals.is_empty(), "casting totals populated");

    // Lab Total (Creo, Corpus) = Int 3 + Magic Theory 4 + Creo 10 + Corpus 12 + Aura 3 = 32.
    let cr_co = d
        .lab_totals
        .iter()
        .find(|l| l.technique.as_str() == "art.creo" && l.form.as_str() == "art.corpus")
        .expect("Creo/Corpus lab cell present");
    assert_eq!(cr_co.total, 32, "Creo+Corpus Lab Total");

    // Casting (Creo, Ignem) formulaic = 10 + 8 + Sta 2 − Enc 1 + Aura 3 + Method Caster 3 = 25.
    let cr_ig = d
        .casting_totals
        .iter()
        .find(|c| c.technique.as_str() == "art.creo" && c.form.as_str() == "art.ignem")
        .expect("Creo/Ignem casting cell present");
    assert_eq!(cr_ig.formulaic, 25, "Creo+Ignem formulaic casting total");
    // Within the focus, the lower applicable Art (Ignem 8) is added again → 33.
    let wf = cr_ig.within_focus.as_ref().expect("focus present on cell");
    assert_eq!(
        wf.formulaic, 33,
        "within-focus adds the lower Art (Ignem 8)"
    );

    // Encumbrance: Load 7 (axe 1 + round shield 2 + partial chain 4) → Burden 3; Str 2 → 1.
    assert_eq!(d.encumbrance.total, 1, "Encumbrance = Burden 3 − Str 2");

    // Per-Form Magic Resistance (Ignem) = Form 8 + 5 × Parma 3 = 23.
    let mr_ig = d
        .magic_resistance
        .iter()
        .find(|m| m.form.as_str() == "art.ignem")
        .expect("Ignem magic resistance present");
    assert_eq!(mr_ig.total, 23, "Ignem MR = Form + 5×Parma");

    // Penetration for the known spell = casting total − level 15 + Penetration 2.
    let pen = d
        .penetration
        .iter()
        .find(|p| p.spell.as_str() == "spell.blade_of_the_virulent_flame")
        .expect("penetration line for the known spell");
    assert_eq!(
        pen.total,
        cr_ig.formulaic - 15 + 2,
        "penetration self-consistent"
    );
    assert_eq!(pen.total, 12);

    // Combat line for the axe, with the round shield's mods combined in.
    let axe = d
        .combat
        .iter()
        .find(|c| c.weapon.as_str() == "weapon.axe")
        .expect("axe combat line present");
    assert_eq!(axe.initiative, 1, "Init = Qik 1 + wpn 1 + shield 0 − Enc 1");
    assert_eq!(
        axe.attack,
        Some(10),
        "Attack = Dex 2 + Ability 4 + wpn 4 + shield 0"
    );
    assert_eq!(
        axe.defense, 7,
        "Defense = Qik 1 + Ability 4 + wpn 0 + shield 2"
    );
    assert_eq!(axe.damage, Some(8), "Damage = Str 2 + wpn 6");

    // Soak = Sta 2 + Armor 6 + Tough 3 + Bronze cord 2 = 13.
    assert_eq!(d.soak.total, 13, "Soak = Sta + Armor + Tough + Bronze cord");

    // Fatigue and wound tracks populated; Enduring Constitution eases the wound penalty.
    assert!(!d.fatigue.is_empty(), "fatigue levels populated");
    assert!(!d.wounds.is_empty(), "wound ranges populated");

    // Longevity: the *entered* bonus is reported verbatim (6), not the 7 that
    // today's Creo Corpus Lab Total of 32 would suggest — the ritual is a frozen
    // past event. The hint carries the suggestion; the bronze cord is surfaced.
    let lon = d
        .longevity
        .expect("longevity present for a magus with a ritual");
    assert_eq!(lon.bonus, 6, "the entered bonus, verbatim");
    assert!(lon.entered);
    assert_eq!(lon.bronze_cord, 2);
    let hint = lon.hint.expect("self-made rituals get a hint");
    assert_eq!(hint.lab_total, 32, "CrCo Lab Total on the real ruleset");
    assert_eq!(hint.suggested_bonus, 7, "ceil(32/5)");
    assert!(!hint.halved);

    // Talisman capacity on the real ruleset = highest Technique (Creo 10) + highest
    // Form (Corpus 12) = 22 pawns of Vim vis (Ars Magica - Definitive Edition (Core Rules).md:10619). Ignem 8 loses to Corpus.
    let capacity = d
        .talisman_capacity
        .expect("capacity present for a magus with a talisman");
    assert_eq!(capacity.technique.as_str(), "art.creo");
    assert_eq!(capacity.form.as_str(), "art.corpus");
    assert_eq!(capacity.technique_score, 10);
    assert_eq!(capacity.form_score, 12);
    assert_eq!(capacity.pawns, 22);

    // Non-goal guard: the level-15 instilled effect is charged against NOTHING. The
    // item-level budget belongs to the Redcap-only Virtues (Ars Magica - Definitive Edition (Core Rules).md:4347-4349,
    // :4842-4850), and a Redcap "may not take The Gift" (:4850), so it can never
    // fund a magus's talisman. This magus has no such Virtue, so both figures are 0
    // even though he owns a talisman holding 15 levels of effect.
    assert_eq!(
        arm_rules::item_level_used(&e),
        0,
        "talisman effects must not be swept into item_level_used"
    );
    assert_eq!(arm_rules::item_level_budget(&e, &rs), 0);
    assert!(
        validate(&e, &rs)
            .issues
            .iter()
            .all(|i| i.code != "over_item_level"),
        "a talisman effect must raise no item-level issue"
    );

    // Familiar bonding read-out on the real ruleset. Binding level = Magic Might 10
    // + 25 + 5 × Size(-4) = 15 (Ars Magica - Definitive Edition (Core Rules).md:10824, :10828) — the negative Size takes 20
    // points off. Cords 1/1/2 cost 5 + 5 + 15 = 25 off the curve (:10836), which
    // fits inside the best bonding Lab Total. Invested powers total 20 levels and
    // are charged against nothing (:10866).
    let fam = d
        .familiar
        .expect("read-out present for a magus with a familiar");
    assert_eq!(fam.binding_level, 15, "Might 10 + 25 + 5 x -4");
    assert_eq!(fam.cord_points_spent, 25, "5 + 5 + 15 off the cord curve");
    assert_eq!(fam.invested_power_levels, 20);
    // The best (Te,Fo) cell is the same Creo/Corpus pair the capacity names, so the
    // bonding Lab Total matches the Creo Corpus Lab Total the longevity hint used.
    assert_eq!(fam.binding.technique.as_str(), "art.creo");
    assert_eq!(fam.binding.form.as_str(), "art.corpus");
    assert_eq!(fam.binding.lab_total, 32);
    // This magus holds a Major Magical Focus, and :10818 lets a focus apply to the
    // bonding Lab Total — so the conditional figure is present (base + lower Art).
    assert_eq!(fam.binding.lab_total_within_focus, Some(42));
    assert!(fam.binding.lab_total_reaches_level, "32 >= 15");
    assert!(fam.binding.cord_points_within_lab_total, "25 <= 32");

    // Guidance-only guard: nothing the familiar carries raises an issue, not even
    // its Faerie-capable Might, its own Characteristics, or 20 levels of invested
    // power. Compare against the same magus with no familiar at all.
    let mut without = e.clone();
    without.familiar = None;
    let codes = |x: &Entity| -> Vec<String> {
        validate(x, &rs)
            .issues
            .iter()
            .map(|i| i.code.clone())
            .collect()
    };
    assert_eq!(
        codes(&e),
        codes(&without),
        "no familiar read-out may produce a ValidationIssue"
    );

    // Warping Score 2 from 15 stored points; Decrepitude 2 from 17 aging points.
    assert_eq!(d.warping_points, 15);
    assert_eq!(d.warping_score, 2, "15 points → Warping Score 2");
    assert_eq!(d.decrepitude_score, 2, "17 aging points → Decrepitude 2");
}

/// A magus with a Creo Corpus Lab Total of 35, built on the **real shipped
/// ruleset**, gets the book's own worked example back: "Longevity Ritual: Lab Total
/// 35, +7 aging bonus".
/// Source: `Ars Magica - Definitive Edition (Core Rules).md:2573` (the sheet line),
/// `:2488` (the same magus's lab season), `:10662` (the formula).
#[test]
fn longevity_hint_reproduces_the_books_lab_total_35_example() {
    let rs = load_full_ruleset();
    let mut e = entity(
        "magus",
        vec![
            Selection::new(Id::new("virtue.the_gift")),
            Selection::new(Id::new("virtue.hermetic_magus")),
        ],
    );
    // Int 3 + Magic Theory 4 + Creo 10 + Corpus 13 + Aura 5 = 35.
    e.characteristics.insert(Characteristic::Int, 3);
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.magic_theory"),
        parameter: None,
        specialty: None,
        score: 4,
    }];
    e.art_scores = vec![
        ArtScore {
            art: Id::new("art.creo"),
            score: 10,
        },
        ArtScore {
            art: Id::new("art.corpus"),
            score: 13,
        },
    ];
    e.aura = 5;
    e.longevity_ritual = Some(LongevityRitual {
        source: LongevitySource::SelfMade,
        bonus: None,
        focus: String::new(),
    });

    let lon = arm_rules::derived_totals(&e, &rs)
        .longevity
        .expect("longevity present");
    assert!(!lon.entered, "nothing entered yet");
    assert_eq!(lon.bonus, 0, "placeholder, not a claim");
    let hint = lon.hint.expect("self-made rituals get a hint");
    assert_eq!(hint.lab_total, 35);
    assert_eq!(hint.suggested_bonus, 7);

    // A zero aura does not suppress the suggestion — the Aura Modifier is a plain
    // addend, and no aura simply means no hindrance (Ars Magica - Definitive Edition (Core Rules).md:10276-10278, :17658).
    // The removed `aura != 0` gate, guarded on the real ruleset.
    e.aura = 0;
    let hint = arm_rules::derived_totals(&e, &rs)
        .longevity
        .expect("longevity present")
        .hint
        .expect("a zero aura still gets a hint");
    assert_eq!(hint.lab_total, 30);
    assert_eq!(hint.suggested_bonus, 6, "ceil(30/5), not 0");

    // Difficult Longevity Ritual halves it on the shipped catalogue too
    // (Ars Magica - Definitive Edition (Core Rules).md:5962-5964): 30 → 15 → ceil(15/5) = 3.
    e.selections
        .push(Selection::new(Id::new("flaw.difficult_longevity_ritual")));
    let hint = arm_rules::derived_totals(&e, &rs)
        .longevity
        .expect("longevity present")
        .hint
        .expect("has a hint");
    assert_eq!(hint.lab_total, 15);
    assert_eq!(hint.suggested_bonus, 3);
    assert!(hint.halved, "the Flaw's halving is flagged for the UI");
}

/// The shipped Aging tables, read as text. `load_full_ruleset` feeds these bytes
/// to the loader exactly as `ruleset_io.rs` does, so the row-by-row tests below
/// read the tables back off the ruleset rather than deserializing them
/// themselves.
const SHIPPED_AGING: &str = include_str!("../../../rules/core/aging.json");
const SHIPPED_AGING_EN: &str = include_str!("../../../rules/i18n/en/aging.json");
const SHIPPED_AGING_DE: &str = include_str!("../../../rules/i18n/de/aging.json");

fn shipped_aging_rules() -> AgingRules {
    load_full_ruleset()
        .aging()
        .expect("the shipped ruleset carries the aging tables")
        .clone()
}

/// The whole of `## Aging`'s two tables, transcribed row by row: the scalars of
/// `:16565`-`:16577`, the ten Living Conditions of `:16583-16592` (five of them
/// asterisked, i.e. cumulative — `:16594`) and the eleven Aging Roll outcomes of
/// `:16601-16611`.
///
/// The row values are deliberately **literals** here: nothing else in the engine
/// can witness a mis-transcribed modifier or a swapped Characteristic, since the
/// JSON is the only place those numbers live.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16563-16615.
#[test]
fn shipped_aging_table_carries_the_16583_to_16611_rows() {
    let rules = shipped_aging_rules();

    // Stated outright rather than left to `load_full_ruleset`'s unwrap: the
    // shipped tables clear every load-time gate of `validate_aging_rules` —
    // contiguous rows up to an open-ended top one, a clamp below the first
    // aging-point row (:16575), and no duplicate Living Condition id.
    load_full_ruleset()
        .validate_integrity()
        .expect("the shipped aging tables pass every load-time gate");

    // "Characters begin aging in the Winter after they turn 35" (:16565), the
    // "age/10 (round up)" term (:16567) and the apparent-aging threshold, which
    // is a question asked of every total rather than a row: "2 or less — No
    // apparent aging" / "3 or more — Apparent age increases by one year"
    // (:16599-16600, :16577).
    assert_eq!(rules.start_age, 35);
    assert_eq!(rules.age_divisor, 10);
    assert_eq!(rules.apparent_age_increase_min, 3);
    // "treats all rolls of 10 or more as rolls of 9 until he reaches the age of
    // 35" (:16575).
    let clamp = rules
        .longevity_clamp
        .clone()
        .expect("the :16575 clamp ships");
    assert_eq!(clamp.max_total, 9);
    assert_eq!(clamp.until_age, 35);

    let conditions: Vec<(&str, i8)> = rules
        .living_conditions
        .iter()
        .map(|row| (row.id.as_str(), row.modifier))
        .collect();
    assert_eq!(
        conditions,
        vec![
            ("living_condition.average_peasant", 0),
            ("living_condition.leper", -2),
            ("living_condition.live_in_a_leper_colony", -1),
            (
                "living_condition.poor_or_unhealthy_location_typical_town",
                -2
            ),
            (
                "living_condition.typical_spring_or_winter_covenant_magus",
                1
            ),
            (
                "living_condition.typical_summer_or_autumn_covenant_magus",
                2
            ),
            (
                "living_condition.typical_summer_or_autumn_covenant_mundane",
                1
            ),
            ("living_condition.wealthy_or_healthy_location", 2),
            ("living_condition.work_in_a_bad_air_trade", -1),
            ("living_condition.work_in_a_mine", -1),
        ],
        "the ten rows of :16583-16592, id-sorted"
    );

    // "\\* Modifiers marked with an asterisk are cumulative with each other."
    // (:16594) — FIVE rows carry it, the three occupational -1s and both -2s.
    let cumulative: Vec<&str> = rules
        .living_conditions
        .iter()
        .filter(|row| row.cumulative)
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(
        cumulative,
        vec![
            "living_condition.leper",
            "living_condition.live_in_a_leper_colony",
            "living_condition.poor_or_unhealthy_location_typical_town",
            "living_condition.work_in_a_bad_air_trade",
            "living_condition.work_in_a_mine",
        ],
        "exactly the five asterisked rows :16588-16592 are cumulative"
    );

    // Every row points at its own line of the table, so a re-transcription can be
    // checked against the source one row at a time.
    let condition_lines: Vec<(&str, u32, u32)> = rules
        .living_conditions
        .iter()
        .map(|row| {
            let source = row.source.as_ref().expect("every row carries provenance");
            assert_eq!(
                source.file,
                "Ars Magica - Definitive Edition (Core Rules).md"
            );
            (row.id.as_str(), source.lines.start, source.lines.end)
        })
        .collect();
    assert_eq!(
        condition_lines,
        vec![
            ("living_condition.average_peasant", 16587, 16587),
            ("living_condition.leper", 16592, 16592),
            ("living_condition.live_in_a_leper_colony", 16588, 16588),
            (
                "living_condition.poor_or_unhealthy_location_typical_town",
                16591,
                16591
            ),
            (
                "living_condition.typical_spring_or_winter_covenant_magus",
                16586,
                16586
            ),
            (
                "living_condition.typical_summer_or_autumn_covenant_magus",
                16584,
                16584
            ),
            (
                "living_condition.typical_summer_or_autumn_covenant_mundane",
                16585,
                16585
            ),
            ("living_condition.wealthy_or_healthy_location", 16583, 16583),
            ("living_condition.work_in_a_bad_air_trade", 16589, 16589),
            ("living_condition.work_in_a_mine", 16590, 16590),
        ]
    );

    // The Aging Roll table (:16601-16611): eleven effect rows, ascending, the
    // last one open-ended ("22+").
    let any = |points| AgingRowEffect::AnyCharacteristic { points };
    let named = |characteristics: Vec<Characteristic>| AgingRowEffect::NamedCharacteristics {
        points: 1,
        characteristics,
    };
    let crisis = AgingRowEffect::NextDecrepitudeLevelAndCrisis;
    let outcomes: Vec<(i32, Option<i32>, &AgingRowEffect, u32)> = rules
        .outcomes
        .iter()
        .map(|row| {
            let source = row.source.as_ref().expect("every row carries provenance");
            assert_eq!(
                source.file,
                "Ars Magica - Definitive Edition (Core Rules).md"
            );
            assert_eq!(
                source.lines.start, source.lines.end,
                "a table row spans one line"
            );
            (row.min, row.max, &row.effect, source.lines.start)
        })
        .collect();
    assert_eq!(
        outcomes,
        vec![
            (10, Some(12), &any(1), 16601),
            (13, Some(13), &crisis, 16602),
            (14, Some(14), &named(vec![Characteristic::Qik]), 16603),
            (15, Some(15), &named(vec![Characteristic::Sta]), 16604),
            (16, Some(16), &named(vec![Characteristic::Per]), 16605),
            // "1 Aging Point in Prs" (:16606) — the table's abbreviation for
            // Presence, which the engine spells `pre`.
            (17, Some(17), &named(vec![Characteristic::Pre]), 16606),
            (
                18,
                Some(18),
                &named(vec![Characteristic::Str, Characteristic::Sta]),
                16607
            ),
            (
                19,
                Some(19),
                &named(vec![Characteristic::Dex, Characteristic::Qik]),
                16608
            ),
            (
                20,
                Some(20),
                &named(vec![Characteristic::Com, Characteristic::Pre]),
                16609
            ),
            (
                21,
                Some(21),
                &named(vec![Characteristic::Int, Characteristic::Per]),
                16610
            ),
            (22, None, &crisis, 16611),
        ]
    );

    // The table's structural signature: over the eight rows that name
    // Characteristics (:16603-16610), the four "physical/social pairs" halves
    // Qik, Sta, Per and Prs each appear twice — once alone, once paired — and
    // Str, Dex, Com and Int exactly once each. A row transcribed with the wrong
    // Characteristic breaks this even if every band still looks plausible.
    let mut tally: BTreeMap<Characteristic, usize> = BTreeMap::new();
    for row in rules.outcomes.iter().filter(|r| (14..=21).contains(&r.min)) {
        let AgingRowEffect::NamedCharacteristics {
            characteristics, ..
        } = &row.effect
        else {
            panic!("rows 14-21 all name their Characteristics: {row:?}");
        };
        for c in characteristics {
            *tally.entry(*c).or_default() += 1;
        }
    }
    assert_eq!(
        tally,
        BTreeMap::from([
            (Characteristic::Int, 1),
            (Characteristic::Per, 2),
            (Characteristic::Pre, 2),
            (Characteristic::Com, 1),
            (Characteristic::Str, 1),
            (Characteristic::Sta, 2),
            (Characteristic::Dex, 1),
            (Characteristic::Qik, 2),
        ])
    );
}

/// One row of the Crisis Table as the transcription below compares it: the row
/// id, its inclusive total range, the outcome it names, and the source line it
/// was taken from. Named rather than written inline because the tuple is what the
/// whole transcription is asserted against, and an anonymous five-field tuple in
/// the assertion says nothing about which field is which.
type CrisisTableRow<'a> = (&'a str, Option<i32>, Option<i32>, &'a CrisisOutcome, u32);

/// The Crisis Table transcribed row by row (`:16626-16632`), together with the
/// Simple Die it is rolled on (`:474`), the attending doctor (`:16634`) and the
/// two Decrepitude thresholds of `:16617`.
///
/// The values are deliberately **literals**, for the same reason the Aging Roll
/// transcription above uses them: the shipped JSON is the only place an Ease
/// Factor or a Ritual level lives, so nothing else in the engine can witness a
/// mis-transcribed one.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:474, :16617-16634.
#[test]
fn shipped_crisis_table_carries_the_16626_to_16632_rows() {
    let rules = shipped_aging_rules();
    let crisis = rules.crisis.clone().expect("the shipped crisis table");

    // Stated outright rather than left to `load_full_ruleset`'s unwrap: the
    // shipped Crisis Table clears every load-time gate of `validate_crisis_rules`
    // — rows tiling contiguously between an open-below and an open-above end, an
    // illness ladder whose severity, Ritual level and Ease Factor all climb
    // together (:16638), an attendant Ability that resolves, and a frail
    // Decrepitude score below the fatal one (:16617).
    load_full_ruleset()
        .validate_integrity()
        .expect("the shipped crisis table passes every load-time gate");

    // "Characters with a Decrepitude score of 4 are extremely frail, and must
    // roll on the Crisis Table … Characters with a Decrepitude score of 5 are
    // bedridden and will die within a few months at most." (:16617)
    assert_eq!(rules.frail_decrepitude_score, Some(4));
    assert_eq!(rules.fatal_decrepitude_score, Some(5));

    // "Roll a ten-sided die. Each number counts for its value, except that a zero
    // counts as ten." (:474) — the CRISIS TOTAL's Simple die (:16621).
    let die = crisis.die.clone().expect("the Simple Die of :474 ships");
    assert_eq!((die.min, die.max), (1, 10));
    let die_source = die.source.expect("the die carries provenance");
    assert_eq!(
        die_source.file,
        "Ars Magica - Definitive Edition (Core Rules).md"
    );
    assert_eq!((die_source.lines.start, die_source.lines.end), (474, 474));

    // "An Int + Medicine roll against an Ease Factor of 6 allows the character to
    // add the attendant's Medicine score to the roll to survive the crisis. …
    // if the doctor botches the character must subtract 3 from the survival
    // roll." (:16634) — the penalty is stored signed, as the roll takes it.
    let attendant = crisis
        .attendant
        .clone()
        .expect("the attendant of :16634 ships");
    assert_eq!(attendant.ability.as_str(), "ability.medicine");
    assert_eq!(attendant.characteristic, Characteristic::Int);
    assert_eq!(attendant.ease_factor, 6);
    assert_eq!(attendant.botch_penalty, -3);
    let attendant_source = attendant.source.expect("the attendant carries provenance");
    assert_eq!(
        attendant_source.file,
        "Ars Magica - Definitive Edition (Core Rules).md"
    );
    assert_eq!(
        (attendant_source.lines.start, attendant_source.lines.end),
        (16634, 16634)
    );

    let bedridden = CrisisOutcome::Bedridden;
    let illness = |severity, ease_factor, ritual_level| CrisisOutcome::Illness {
        severity,
        ease_factor,
        ritual_level,
    };
    let rows: Vec<CrisisTableRow<'_>> = crisis
        .rows
        .iter()
        .map(|row| {
            let source = row.source.as_ref().expect("every row carries provenance");
            assert_eq!(
                source.file,
                "Ars Magica - Definitive Edition (Core Rules).md"
            );
            assert_eq!(
                source.lines.start, source.lines.end,
                "a table row spans one line"
            );
            (
                row.id.as_str(),
                row.min,
                row.max,
                &row.outcome,
                source.lines.start,
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            // "8 or less — Bedridden for a week" (:16626): open below, so no
            // `min` at all rather than a very small one.
            ("crisis.bedridden_week", None, Some(8), &bedridden, 16626),
            (
                "crisis.bedridden_month",
                Some(9),
                Some(14),
                &bedridden,
                16627
            ),
            (
                "crisis.minor_illness",
                Some(15),
                Some(15),
                &illness(CrisisSeverity::Minor, Some(3), 20),
                16628
            ),
            (
                "crisis.serious_illness",
                Some(16),
                Some(16),
                &illness(CrisisSeverity::Serious, Some(6), 25),
                16629
            ),
            (
                "crisis.major_illness",
                Some(17),
                Some(17),
                &illness(CrisisSeverity::Major, Some(9), 30),
                16630
            ),
            (
                "crisis.critical_illness",
                Some(18),
                Some(18),
                &illness(CrisisSeverity::Critical, Some(12), 35),
                16631
            ),
            // "19+ — **Terminal illness**. CrCo40 required to survive."
            // (:16632): open above, and no Stamina roll at all — hence no Ease
            // Factor rather than an unbeatable one.
            (
                "crisis.terminal_illness",
                Some(19),
                None,
                &illness(CrisisSeverity::Terminal, None, 40),
                16632
            ),
        ]
    );

    // The table's structural signature. `crisis.rows` is the ONE array in
    // `rules/core` that ships in band order rather than id order (see RULES.md):
    // the rows ascend by the totals they cover, open below at the top of the
    // table and open above at the bottom. An id-alphabetical sort would leave
    // every value above intact and still break this.
    let first = crisis.rows.first().expect("the crisis table has rows");
    let last = crisis.rows.last().expect("the crisis table has rows");
    assert!(
        first.min.is_none(),
        "the first row is open below: \"8 or less\" (:16626)"
    );
    assert!(
        last.max.is_none(),
        "the last row is open above: \"19+\" (:16632)"
    );
    for pair in crisis.rows.windows(2) {
        let ceiling = pair[0].max.expect("only the last row is open above");
        let floor = pair[1].min.expect("only the first row is open below");
        assert!(
            ceiling < floor,
            "the rows ascend by band: '{}' ends at {ceiling}, '{}' starts at {floor}",
            pair[0].id,
            pair[1].id
        );
    }

    // "The level of spell required depends on the severity of the crisis, as
    // noted on the table." (:16638) — so severity is a ladder that climbs with
    // the band, and the Ritual level climbs with it.
    let illnesses: Vec<(CrisisSeverity, u32)> = crisis
        .rows
        .iter()
        .filter_map(|row| match &row.outcome {
            CrisisOutcome::Illness {
                severity,
                ritual_level,
                ..
            } => Some((*severity, *ritual_level)),
            CrisisOutcome::Bedridden => None,
        })
        .collect();
    assert!(
        illnesses
            .windows(2)
            .all(|pair| pair[0].0 < pair[1].0 && pair[0].1 < pair[1].1),
        "the illness rows climb in severity and Ritual level with the band: {illnesses:?}"
    );
}

/// The AGING TOTAL of a 40-year-old carrying the named shipped items, rolling a
/// 6: `6 + ceil(40/10)` = 10 before any modifier.
fn shipped_aging_total(items: &[&str]) -> AgingTotal {
    let rs = load_full_ruleset();
    let e = entity(
        "companion",
        items
            .iter()
            .map(|id| Selection::new(Id::new(*id)))
            .collect(),
    );
    arm_rules::aging::aging_total(&e, &rs, 40, 6).expect("the shipped ruleset carries aging rules")
}

/// Faerie Blood: "You are resistant to aging, and get -1 to all aging rolls."
/// (Ars Magica - Definitive Edition (Core Rules).md:3801) — the shipped `aging_roll -1` is ADDED with its stored sign, so
/// the total drops by one.
#[test]
fn faerie_blood_lowers_the_aging_total_by_one() {
    assert_eq!(
        shipped_aging_total(&[]).total,
        10,
        "the unmodified baseline"
    );

    let faerie = shipped_aging_total(&["virtue.faerie_blood"]);
    assert_eq!(faerie.trait_modifier, -1);
    assert_eq!(faerie.total, 9);
}

/// Strong Faerie Blood: "You start making aging rolls at the age of fifty, rather
/// than the normal 35, and get -3 to Aging Rolls, cumulative with any other
/// bonuses." (Ars Magica - Definitive Edition (Core Rules).md:5036)
///
/// Only the -3 half is implemented. The start-at-fifty half needs a per-trait
/// override of [`arm_rules::AgingRules::start_age`] — machinery no other shipped
/// item asks for — so this character is still scheduled from 36, which is a known
/// gap rather than a reading of the text.
#[test]
fn strong_faerie_blood_lowers_the_aging_total_by_three() {
    let strong = shipped_aging_total(&["virtue.strong_faerie_blood"]);
    assert_eq!(strong.trait_modifier, -3);
    assert_eq!(strong.total, 7);
}

/// The `aging_mod` kinds an item ships, sorted so the assertion does not depend
/// on the order the effects happen to sit in the file.
fn aging_kinds(id: &str) -> Vec<(AgingEffect, i8)> {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new(id))
        .unwrap_or_else(|| panic!("the shipped catalogue carries '{id}'"));
    let mut kinds: Vec<(AgingEffect, i8)> = item
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::AgingMod { kind, amount } => Some((*kind, *amount)),
            _ => None,
        })
        .collect();
    kinds.sort_unstable();
    kinds
}

/// Mild Aging states **two** mechanics in one sentence, and they go to two
/// different places:
///
/// > "The character's aging rolls benefit from a +1 bonus to the Living
/// > Conditions Modifier, in addition to whatever his social standing normally
/// > offers him. Furthermore, he receives a +3 bonus to rolls to survive an aging
/// > crisis." (Ars Magica - Definitive Edition (Core Rules).md:4530)
///
/// The +1 is a Living Conditions term of the AGING TOTAL; the +3 belongs to the
/// crisis *survival* roll, which `:16636` otherwise walls off from aging-roll
/// modifiers entirely. Only the first half shipped until 6b7, so the Virtue read
/// as half a rule. Both halves now ship, and this test is the witness that a
/// later sweep does not drop one again.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4530, :16636.
#[test]
fn mild_aging_carries_both_halves_of_4530() {
    assert_eq!(
        aging_kinds("virtue.mild_aging"),
        vec![
            (AgingEffect::LivingConditions, 1),
            (AgingEffect::CrisisSurvival, 3),
        ],
    );
}

/// The [`CrisisOutcome`] of one named row of the **shipped** Crisis Table — what
/// a look-up would hand `crisis_survival`, fetched by id so the test does not
/// have to invent a total to reach the row.
fn shipped_crisis_outcome(id: &str) -> CrisisOutcome {
    shipped_aging_rules()
        .crisis
        .as_ref()
        .expect("the shipped crisis table")
        .rows
        .iter()
        .find(|row| row.id.as_str() == id)
        .unwrap_or_else(|| panic!("the shipped Crisis Table carries '{id}'"))
        .outcome
        .clone()
}

/// The crisis-survival read-out for a companion carrying the named shipped
/// items, against the shipped Minor illness row (`:16628`).
fn shipped_crisis_survival(items: &[&str]) -> CrisisSurvival {
    let rs = load_full_ruleset();
    let e = entity(
        "companion",
        items
            .iter()
            .map(|id| Selection::new(Id::new(*id)))
            .collect(),
    );
    arm_rules::aging::crisis_survival(&e, &rs, &shipped_crisis_outcome("crisis.minor_illness"))
        .expect("an illness is a roll to describe")
}

/// **"Virtues that affect aging rolls do not affect crisis survival rolls."**
/// (Ars Magica - Definitive Edition (Core Rules).md:16636) — the single load-bearing sentence of the survival read-out,
/// locked against the shipped catalogue rather than a fixture.
///
/// Three shipped items carry the two kinds the aging roll takes, and all three
/// are witnessed moving the AGING TOTAL and then contributing **nothing** to the
/// survival roll:
///
/// - Faerie Blood, `aging_roll -1` (`:3801`)
/// - Strong Faerie Blood, `aging_roll -3` (`:5036`)
/// - Poor Living Conditions, `living_conditions -1` (`:6620`)
///
/// Mild Aging is the proof case, because `:4530` grants both sides in one
/// sentence: "The character's aging rolls benefit from a +1 bonus to the Living
/// Conditions Modifier … Furthermore, he receives a +3 bonus to rolls to survive
/// an aging crisis." The +1 stays out of the crisis; the +3 goes in, alone.
///
/// An implementation that simply summed every `aging_mod` amount would read -5
/// on the first character and -1 on the second, so this test is not satisfiable
/// by accident.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3801, :4530, :5036,
/// :6620, :16636.
#[test]
fn virtues_that_modify_aging_rolls_do_not_affect_crisis_survival_rolls() {
    let aging_movers = [
        "virtue.faerie_blood",
        "virtue.strong_faerie_blood",
        "flaw.poor_living_conditions",
    ];

    // They demonstrably move the AGING TOTAL: -1 and -3 on the trait modifier,
    // -1 on the Living Conditions term.
    let aging = shipped_aging_total(&aging_movers);
    assert_eq!(aging.trait_modifier, -4);
    assert_eq!(aging.living_conditions.total, -1);

    // …and contribute nothing at all to the survival roll.
    let survival = shipped_crisis_survival(&aging_movers);
    assert_eq!(survival.modifiers, vec![]);
    assert_eq!(survival.modifier_total, 0);

    // Mild Aging's +3 is a grant to *this* roll by name, so it does arrive — and
    // it arrives alone, itemized under the id the UI resolves to a label.
    let mut with_mild = aging_movers.to_vec();
    with_mild.push("virtue.mild_aging");
    let survival = shipped_crisis_survival(&with_mild);
    assert_eq!(
        survival.modifiers,
        vec![CrisisModifier {
            source: CrisisModifierSource::Trait {
                item: Id::new("virtue.mild_aging"),
            },
            amount: 3,
        }],
        "only :4530's crisis-survival half crosses the :16636 wall"
    );
    assert_eq!(survival.modifier_total, 3);

    // The Living Conditions half is not lost, merely elsewhere: -1 from the Flaw
    // and +1 from Mild Aging cancel on the roll that takes them.
    assert_eq!(shipped_aging_total(&with_mild).living_conditions.total, 0);
}

/// "19+ — **Terminal illness**. CrCo40 required to survive." (Ars Magica - Definitive Edition (Core Rules).md:16632) — the
/// one row that offers no Stamina roll at all. The read-out reports the Ritual
/// that resolves it (`:16638`) and **no** Ease Factor, rather than an unbeatable
/// one; and the Minor row beside it shows the ordinary shape, Ease Factor 3 and
/// CrCo20 (`:16628`).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16628, :16632, :16638.
#[test]
fn the_terminal_row_reports_a_ritual_level_and_no_ease_factor() {
    let rs = load_full_ruleset();
    let e = entity("companion", vec![]);

    let terminal = arm_rules::aging::crisis_survival(
        &e,
        &rs,
        &shipped_crisis_outcome("crisis.terminal_illness"),
    )
    .expect("Terminal illness is still a crisis to describe");
    assert_eq!(
        terminal.ease_factor, None,
        "no Stamina roll is offered at 19+"
    );
    assert_eq!(terminal.ritual_level, 40);

    let minor =
        arm_rules::aging::crisis_survival(&e, &rs, &shipped_crisis_outcome("crisis.minor_illness"))
            .expect("Minor illness is survivable");
    assert_eq!(minor.ease_factor, Some(3));
    assert_eq!(minor.ritual_level, 20);
}

/// "An Int + Medicine roll against an Ease Factor of 6 allows the character to
/// add the attendant's Medicine score to the roll to survive the crisis. Only
/// one doctor may usefully attend a patient, and if the doctor botches the
/// character must subtract 3 from the survival roll." (Ars Magica - Definitive Edition (Core Rules).md:16634)
///
/// The doctor is reported as an **allowance** — what the rules permit — and not
/// as a modifier, because the app has no attendant to score: the Medicine score
/// belongs to another character entirely. Every value comes off the ruleset, and
/// the botch penalty keeps the file's one sign convention (stored signed, added).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16634.
#[test]
fn the_attending_doctor_is_reported_as_an_allowance() {
    let survival = shipped_crisis_survival(&[]);
    assert_eq!(
        survival.allowances,
        vec![CrisisAllowance::Attendant {
            ability: Id::new("ability.medicine"),
            characteristic: Characteristic::Int,
            ease_factor: 6,
            botch_penalty: -3,
        }],
        "one doctor, with the ruleset's own numbers"
    );
    // An allowance is never a term of the total the character brings.
    assert_eq!(survival.modifier_total, 0);
}

/// One Crisis walked end to end against the **shipped** table, through the
/// crate's public surface: a die and a year in, and the CRISIS TOTAL
/// (Ars Magica - Definitive Edition (Core Rules).md:16621), the row it lands on (`:16624-16632`), what that row costs and
/// what surviving it would take (`:16628-16638`) out.
///
/// The fixture tests in `aging.rs` prove the composition; this proves it against
/// the real `rules/core/aging.json`, where the attendant of `:16634` actually
/// ships — so the doctor reaches a caller through the composed read-out and not
/// only through a hand-built outcome.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16621-16638.
#[test]
fn the_shipped_crisis_table_answers_a_total_end_to_end() {
    let rs = load_full_ruleset();
    let mut e = entity("companion", vec![]);
    e.age = Some(40);
    e.aging_points.insert(Characteristic::Sta, 15);

    // `9 + ⌈36/10⌉ + 2 = 15` — Minor illness, Ease Factor 3, CrCo20 (:16628).
    let preview = arm_rules::crisis_preview(&e, &rs, 36, 9).expect("the shipped table");
    assert_eq!(preview.total.age_modifier, 4);
    assert_eq!(preview.total.decrepitude_score, 2);
    assert_eq!(preview.total.total, 15);
    assert_eq!(preview.row, Id::new("crisis.minor_illness"));
    assert_eq!(
        preview.outcome,
        CrisisOutcome::Illness {
            severity: CrisisSeverity::Minor,
            ease_factor: Some(3),
            ritual_level: 20,
        }
    );
    let survival = preview.survival.expect("an illness is survivable");
    assert_eq!(survival.ease_factor, Some(3));
    assert_eq!(survival.ritual_level, 20);
    assert_eq!(
        survival.allowances,
        vec![CrisisAllowance::Attendant {
            ability: Id::new("ability.medicine"),
            characteristic: Characteristic::Int,
            ease_factor: 6,
            botch_penalty: -3,
        }],
        "the doctor of :16634 reaches the composed read-out too"
    );

    // "8 or less — Bedridden for a week" (:16626) is time, not a roll.
    let unaged = entity("companion", vec![]);
    let bedridden = arm_rules::crisis_preview(&unaged, &rs, 36, 4).expect("the shipped table");
    assert_eq!(bedridden.total.total, 8);
    assert_eq!(bedridden.row, Id::new("crisis.bedridden_week"));
    assert_eq!(bedridden.outcome, CrisisOutcome::Bedridden);
    assert!(bedridden.survival.is_none());

    // The shipped table's own bands, read by the look-up rather than by index.
    let landings: Vec<&str> = [8, 9, 14, 15, 16, 17, 18, 19, 99]
        .into_iter()
        .map(|total| {
            arm_rules::resolve_crisis_row(&rs, total)
                .unwrap_or_else(|| panic!("the shipped table covers {total}"))
                .id
                .as_str()
        })
        .collect();
    assert_eq!(
        landings,
        vec![
            "crisis.bedridden_week",
            "crisis.bedridden_month",
            "crisis.bedridden_month",
            "crisis.minor_illness",
            "crisis.serious_illness",
            "crisis.major_illness",
            "crisis.critical_illness",
            "crisis.terminal_illness",
            "crisis.terminal_illness",
        ]
    );
}

/// One Crisis **written into a character** against the shipped tables, and taken
/// back off again.
///
/// A 40-year-old companion rolls a 9: `9 + ⌈40/10⌉ = 13`, the row of `:16602` that
/// reaches the next level in Decrepitude and sends him to the Crisis Table. Five
/// Aging Points is what the shipped advancement curve prices Decrepitude 1 at, and
/// `:16619`'s "increase the character's Decrepitude first" is visible in the CRISIS
/// TOTAL: `10 + 4 + 1 = 15`, the **1** being the score this very year raised. That
/// lands on the shipped minor illness — Ease Factor 3, CrCo20, and the doctor of
/// `:16634`, which only the real `rules/core/aging.json` ships.
///
/// The fixture tests in `aging.rs` prove the leg; this proves it against the
/// catalogue the app actually loads, and that the year still comes back off byte
/// for byte.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16602, :16619, :16621,
/// :16628, :16634.
#[test]
fn a_shipped_crisis_year_is_written_into_the_character_and_reverts_exactly() {
    let rs = load_full_ruleset();
    let mut e = entity("companion", vec![]);
    e.age = Some(40);
    let before = serde_json::to_string(&e).expect("a character serializes");

    let request = arm_rules::AgingYearRequest {
        age: 40,
        die: 9,
        distribution: BTreeMap::from([(Characteristic::Sta, 5)]),
        crisis_die: Some(10),
    };
    let resolved = arm_rules::resolve_year(&e, &rs, &request).expect("a shipped crisis year");
    assert_eq!(resolved.total.total, 13);
    assert!(resolved.outcome.crisis);

    let crisis = resolved.crisis.as_ref().expect("the player rolled it");
    assert_eq!(
        crisis.total.decrepitude_score, 1,
        "the score this year raised, not the 0 he started it with"
    );
    assert_eq!(crisis.total.total, 15);
    assert_eq!(crisis.row, Id::new("crisis.minor_illness"));
    let survival = crisis.survival.as_ref().expect("an illness is survivable");
    assert_eq!(survival.ease_factor, Some(3));
    assert_eq!(survival.ritual_level, 20);
    assert_eq!(
        survival.allowances,
        vec![CrisisAllowance::Attendant {
            ability: Id::new("ability.medicine"),
            characteristic: Characteristic::Int,
            ease_factor: 6,
            botch_penalty: -3,
        }],
        "the doctor of :16634 reaches the write-back too"
    );

    // The year records it, and the character is alive and holding exactly the
    // points the aging row awarded.
    let entry = &resolved.entity.aging_log[0];
    assert_eq!(entry.crisis_die, Some(10));
    assert_eq!(entry.crisis_total, Some(15));
    assert_eq!(entry.crisis_row, Some(Id::new("crisis.minor_illness")));
    assert_eq!(entry.crisis_severity, Some(CrisisSeverity::Minor));
    assert_eq!(
        resolved.entity.aging_points,
        BTreeMap::from([(Characteristic::Sta, 5)])
    );

    let reverted = arm_rules::revert_year(&resolved.entity, &rs, 40).expect("comes back off");
    assert_eq!(
        serde_json::to_string(&reverted).expect("a character serializes"),
        before
    );
}

/// Leprosy likewise states two mechanics at once:
///
/// > "A leper has a permanent -2 modifier to her Living Condition …, and whenever
/// > she undergoes an Aging Crisis (page 392) the leper sustains a Heavy Wound in
/// > addition to any other result." (Ars Magica - Definitive Edition (Core Rules).md:6340)
///
/// The Heavy Wound is a *consequence*, not a number, so it ships as a marker with
/// amount 0 — the `crisis_heavy_wound` kind exists precisely so the shipped 0 is
/// not mistaken for an unfilled modifier.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:6340.
#[test]
fn leprosy_carries_its_crisis_wound_beside_its_living_conditions_penalty() {
    assert_eq!(
        aging_kinds("flaw.leprosy"),
        vec![
            (AgingEffect::LivingConditions, -2),
            (AgingEffect::CrisisHeavyWound, 0),
        ],
    );
}

/// > "Virtues that affect aging rolls do not affect crisis survival rolls."
/// > (Ars Magica - Definitive Edition (Core Rules).md:16636)
///
/// This is that sentence's **converse**, which Mild Aging is the first shipped
/// item to make expressible: a modifier granted specifically to the crisis
/// survival roll is not an aging-roll modifier either, so nothing of the +3 may
/// reach the AGING TOTAL. Mild Aging moves the Living Conditions term by +1 and
/// nothing else — the trait modifier stays 0, and the total drops by exactly one,
/// because the total *subtracts* the Living Conditions Modifier (`:16571`).
///
/// (Step 7 pins the other direction, that `aging_roll` modifiers stay out of the
/// survival roll.)
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4530, :16636.
#[test]
fn a_crisis_survival_modifier_never_reaches_the_aging_total() {
    let baseline = shipped_aging_total(&[]);
    let mild = shipped_aging_total(&["virtue.mild_aging"]);

    assert_eq!(
        mild.living_conditions.from_traits,
        baseline.living_conditions.from_traits + 1,
        "the +1 half is a Living Conditions term"
    );
    assert_eq!(
        mild.trait_modifier, 0,
        "the +3 is not an aging-roll modifier"
    );
    assert_eq!(mild.longevity_bonus, baseline.longevity_bonus);
    assert_eq!(mild.age_modifier, baseline.age_modifier);
    assert_eq!(
        mild.total,
        baseline.total - 1,
        "only the Living Conditions half moves the total"
    );

    // The +3 is nowhere in the total, but it is still surfaced for the player,
    // labelled by its own kind rather than folded into an aging-roll figure.
    let rs = load_full_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.mild_aging"))],
    );
    let surfaced: Vec<(String, i32)> = arm_rules::derived::surfaced_modifiers(&e, &rs)
        .into_iter()
        .filter(|m| m.family == arm_rules::derived::ModifierFamily::Aging)
        .map(|m| (m.detail, m.amount))
        .collect();
    assert!(
        surfaced.contains(&("crisis_survival".to_string(), 3)),
        "the crisis bonus stays visible: {surfaced:?}"
    );
}

/// `flaw.age_quickly` and `flaw.baneful_circumstances` both ship an `aging_roll`
/// modifier of **0**, and that 0 is deliberate — not an unfilled field waiting to
/// be "fixed" into a number.
///
/// Age Quickly doubles the *rate*: "your effective age … increases two years for
/// every year that passes, and you make two aging rolls each year" (Ars Magica - Definitive Edition (Core Rules).md:5661).
/// Baneful Circumstances adds a *conditional extra roll*: "he must make an
/// additional Aging roll even if he is normally immune to aging" (Ars Magica - Definitive Edition (Core Rules).md:5689).
/// Both are schedule rules — how many rolls, at what effective age — and neither
/// shifts the total of any one roll. The engine does not implement either
/// schedule yet, so each Flaw contributes nothing to the arithmetic while staying
/// visible in the surfaced-modifier read-out, where a player can act on it.
#[test]
fn age_quickly_contributes_nothing_to_the_total_and_stays_surfaced() {
    let items = ["flaw.age_quickly", "flaw.baneful_circumstances"];
    let both = shipped_aging_total(&items);
    assert_eq!(
        both.trait_modifier, 0,
        "neither Flaw shifts one roll's total"
    );
    assert_eq!(both.total, shipped_aging_total(&[]).total);

    // Still surfaced, so the player sees the mechanics the engine cannot apply.
    let rs = load_full_ruleset();
    let e = entity(
        "companion",
        items
            .iter()
            .map(|id| Selection::new(Id::new(*id)))
            .collect(),
    );
    let surfaced: Vec<(String, i32)> = arm_rules::derived::surfaced_modifiers(&e, &rs)
        .into_iter()
        .filter(|m| m.family == arm_rules::derived::ModifierFamily::Aging)
        .map(|m| (m.detail, m.amount))
        .collect();
    assert_eq!(
        surfaced,
        vec![("aging_roll".to_string(), 0), ("aging_roll".to_string(), 0)],
        "both Flaws stay listed for the player"
    );
}

/// The shipped table's own reading of `total`, for a companion who has already
/// accrued `accrued` Aging Points (parked in Str — Decrepitude counts the
/// character's whole bank, whichever Characteristics hold it, Ars Magica - Definitive Edition (Core Rules).md:16617).
fn shipped_aging_outcome(total: i32, accrued: u8) -> AgingOutcome {
    let rs = load_full_ruleset();
    let mut e = entity("companion", vec![]);
    if accrued > 0 {
        e.aging_points.insert(Characteristic::Str, accrued);
    }
    arm_rules::aging::resolve_outcome(&e, &rs, total)
        .expect("the shipped ruleset carries aging rules")
}

/// The shipped table resolved row by row, against the shipped advancement curve:
/// the apparent-aging threshold of `:16599-16600`, the "any Characteristic" band
/// of `:16601`, the named rows of `:16603-16610`, and both Decrepitude-and-Crisis
/// rows (`:16602`, `:16611`).
///
/// The unit fixture in `aging.rs` transcribes these rows by hand; only this test
/// witnesses the ones the app actually ships — and only here does the derived
/// Decrepitude count meet the real advancement curve.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16599-16617.
#[test]
fn the_shipped_aging_table_resolves_each_row_of_16599_to_16611() {
    // "2 or less — No apparent aging" / "3 or more — Apparent age increases by
    // one year": one threshold asked of every total, not a pair of rows.
    let two = shipped_aging_outcome(2, 0);
    assert!(!two.apparent_age_increases);
    assert!(two.awards.is_empty());
    assert!(!two.crisis);
    let nine = shipped_aging_outcome(9, 0);
    assert!(nine.apparent_age_increases);
    assert!(
        nine.awards.is_empty(),
        "the appearance ages below the first row that costs anything"
    );

    // "10–12 — 1 Aging Point in any Characteristic" (:16601), the player placing
    // it (:16615).
    let eleven = shipped_aging_outcome(11, 0);
    assert_eq!(
        eleven.awards,
        vec![AgingPointAward {
            target: AgingPointTarget::PlayerChoice,
            points: Some(1),
        }]
    );
    assert!(eleven.apparent_age_increases);
    assert!(!eleven.crisis);

    // The named rows, including the one the book spells "Prs" (:16606) and a
    // two-Characteristic row where EACH name takes a point of its own (:16608).
    let named = |total: i32| -> Vec<AgingPointAward> { shipped_aging_outcome(total, 0).awards };
    let one_point = |characteristic| AgingPointAward {
        target: AgingPointTarget::Named(characteristic),
        points: Some(1),
    };
    assert_eq!(named(14), vec![one_point(Characteristic::Qik)]);
    assert_eq!(named(17), vec![one_point(Characteristic::Pre)]);
    assert_eq!(
        named(19),
        vec![
            one_point(Characteristic::Dex),
            one_point(Characteristic::Qik),
        ]
    );
    assert!(!shipped_aging_outcome(21, 0).crisis, "only 13 and 22+ do");

    // "Gain sufficient Aging Points … to reach the next level in Decrepitude, and
    // Crisis" (:16602, :16611). The count comes off the shipped curve, so the
    // expectation is computed from it rather than written out.
    let rs = load_full_ruleset();
    let to_first_level = rs
        .advancement()
        .xp_for_score(1)
        .expect("the shipped curve prices Decrepitude 1");
    let accrued = 3;
    let owed = vec![AgingPointAward {
        target: AgingPointTarget::NextDecrepitudeLevel,
        points: Some(to_first_level - u32::from(accrued)),
    }];
    for total in [13, 22, 40] {
        let outcome = shipped_aging_outcome(total, accrued);
        assert_eq!(outcome.awards, owed, "total {total}");
        assert!(
            outcome.crisis,
            "total {total} sends him to the Crisis Table"
        );
        assert!(outcome.apparent_age_increases, "total {total}");
    }
}

/// Every Living Condition has a display name in both shipped languages, and no
/// name smuggles the table's cumulative-marker asterisk into the UI — the
/// `cumulative` flag carries that, and a raw `*` in a label would render as one.
#[test]
fn english_and_german_i18n_cover_all_living_conditions() {
    let rules = shipped_aging_rules();
    let en: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(SHIPPED_AGING_EN).expect("the English aging i18n is valid JSON");
    let de: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(SHIPPED_AGING_DE).expect("the German aging i18n is valid JSON");

    for row in &rules.living_conditions {
        for (lang, texts) in [("en", &en), ("de", &de)] {
            let name = texts
                .get(row.id.as_str())
                .and_then(|entry| entry.get("name"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("{lang} i18n missing living condition '{}'", row.id));
            assert!(!name.is_empty(), "{lang} name for '{}' is empty", row.id);
            assert!(
                !name.contains('*'),
                "{lang} name for '{}' carries the cumulative asterisk: {name}",
                row.id
            );
        }
    }
}

/// Every Crisis Table row has a display name in both shipped languages.
///
/// The German names are pinned as literals for the two rows that are a false
/// friend in the other direction: German *Schwere* is **Major** (:16630) and
/// *Ernste* is **Serious** (:16629), which is the opposite of what the English
/// cognate suggests. `alterung-twilight.md:82-92` agrees with the rulebook body.
#[test]
fn english_and_german_i18n_cover_all_crisis_rows() {
    let rules = shipped_aging_rules();
    let crisis = rules.crisis.clone().expect("the shipped crisis table");
    let en: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(SHIPPED_AGING_EN).expect("the English aging i18n is valid JSON");
    let de: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(SHIPPED_AGING_DE).expect("the German aging i18n is valid JSON");

    let name = |texts: &BTreeMap<String, serde_json::Value>, id: &str, lang: &str| -> String {
        texts
            .get(id)
            .and_then(|entry| entry.get("name"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{lang} i18n missing crisis row '{id}'"))
            .to_owned()
    };

    for row in &crisis.rows {
        for (lang, texts) in [("en", &en), ("de", &de)] {
            let text = name(texts, row.id.as_str(), lang);
            assert!(!text.is_empty(), "{lang} name for '{}' is empty", row.id);
        }
    }

    assert_eq!(
        name(&de, "crisis.serious_illness", "de"),
        "Ernste Erkrankung"
    );
    assert_eq!(
        name(&de, "crisis.major_illness", "de"),
        "Schwere Erkrankung"
    );
}

/// The three shipped items that suspend some part of aging tag the **two
/// independent facts** separately, because the sources state them separately:
///
/// - Unaging — "your aging points do not decrease your Characteristics, only
///   building up to give you Decrepitude points … You may choose your apparent
///   age freely" (Ars Magica - Definitive Edition (Core Rules).md:5189): both facts.
/// - Bound to (Role) — "This Flaw also includes the effects of the Unaging
///   Virtue, **but** the character's apparent age advances in line with their
///   physical age" (Ars Magica - Definitive Edition (Core Rules).md:5743): the Characteristic immunity only. That *but* is
///   what proves the two are separable at all.
/// - Bee King — "Bee Kings do not appear to age after reaching maturity"
///   (Ars Magica - Definitive Edition (Core Rules).md:3488): the appearance only, and nothing about Characteristics.
///
/// All three shipped `no_aging` alone before the tags came apart, which made the
/// Bee King's entry simply wrong. This test is the outside witness that keeps the
/// retag from silently regressing to one tag again.
#[test]
fn the_three_aging_immunities_ship_their_two_facts_separately() {
    let rs = load_ruleset();
    let tagged = |id: &str| -> Vec<AgingEffect> {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("the shipped catalogue carries '{id}'"));
        let mut kinds: Vec<AgingEffect> = item
            .effects
            .iter()
            .filter_map(|effect| match effect {
                Effect::AgingMod { kind, .. } => Some(kind),
                _ => None,
            })
            .copied()
            .collect();
        kinds.sort_unstable();
        kinds
    };

    assert_eq!(
        tagged("virtue.unaging"),
        vec![AgingEffect::NoAging, AgingEffect::NoApparentAging],
        "Unaging states both facts (Ars Magica - Definitive Edition (Core Rules).md:5189)"
    );
    assert_eq!(
        tagged("flaw.bound_to_role_role"),
        vec![AgingEffect::NoAging],
        "Bound to (Role) keeps ageing in appearance (Ars Magica - Definitive Edition (Core Rules).md:5743)"
    );
    assert_eq!(
        tagged("virtue.bee_king"),
        vec![AgingEffect::NoApparentAging],
        "a Bee King only stops looking older (Ars Magica - Definitive Edition (Core Rules).md:3488)"
    );
}

/// Issue F (selection-level): a magus selecting BOTH magnitude variants of the
/// same Virtue — here the prefix pair Major / Minor Magical Focus — must raise
/// the incompatibility issue. Behavioral assertion (no catalogue counts).
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4405.
#[test]
fn both_magical_focus_variants_are_incompatible() {
    let rs = load_ruleset();
    let focus = |name: &str| {
        Selection::with_params(
            Id::new(name),
            BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
        )
    };
    let e = entity(
        "magus",
        vec![
            focus("virtue.major_magical_focus"),
            focus("virtue.minor_magical_focus"),
        ],
    );
    let result = validate(&e, &rs);
    let flagged = result.issues.iter().any(|i| {
        i.code == arm_rules::validation::ValidationIssue::CODE_INCOMPATIBLE
            && [i.args.get("item"), i.args.get("other")]
                .iter()
                .filter_map(|a| a.map(String::as_str))
                .any(|a| a == "virtue.major_magical_focus")
    });
    assert!(
        flagged,
        "selecting both Magical Focus variants must be flagged incompatible, got: {:?}",
        result.issues
    );
}
