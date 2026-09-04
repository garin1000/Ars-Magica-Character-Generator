//! Ruleset construction: the JSON loaders and the [`Ruleset::from_sources`]
//! pipeline. Split out of `ruleset.rs`; see `ruleset.rs` for the [`Ruleset`]
//! struct itself and `ruleset/integrity.rs` for the referential-integrity
//! checks `Self::validate_integrity` runs once a ruleset is assembled.
//!
//! [`Ruleset::from_sources`] used to be one 267-line function (GC3); it is now
//! a short pipeline over four named steps — parse
//! ([`parse_sources`]), pre-integrity checks ([`check_duplicate_ids`],
//! [`check_scholarly_language`], [`check_creation_phase_flow`]), and assembly
//! ([`assemble_ruleset`]) — with no change to the checks themselves or the
//! order errors accumulate in.

use super::*;

impl Ruleset {
    /// Parses point items and type profiles from JSON, validates referential
    /// integrity, and returns a Ruleset with no abilities.
    ///
    /// Convenience constructor over [`Ruleset::from_sources`] for the many tests
    /// that do not exercise the ability registry.
    pub fn from_json(
        id: &str,
        version: &str,
        point_items_json: &str,
        type_profiles_json: &str,
    ) -> Result<Self, RulesetError> {
        Self::from_sources(RulesetSources {
            id,
            version,
            point_items: point_items_json,
            type_profiles: type_profiles_json,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
        })
    }

    /// Like [`Ruleset::from_json`] but also loads the abilities file (catalogue +
    /// advancement table). No arts or characteristic rules.
    ///
    /// `abilities_json` is an object `{ "advancement": [...], "abilities": [...] }`;
    /// both keys default to empty, so `"{}"` is a valid empty file.
    pub fn from_json_with_abilities(
        id: &str,
        version: &str,
        point_items_json: &str,
        type_profiles_json: &str,
        abilities_json: &str,
    ) -> Result<Self, RulesetError> {
        Self::from_sources(RulesetSources {
            id,
            version,
            point_items: point_items_json,
            type_profiles: type_profiles_json,
            abilities: Some(abilities_json),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
        })
    }

    /// Parses every language-neutral core source — point items, type profiles,
    /// abilities (catalogue + advancement), and the Characteristic point-buy rules
    /// — validates referential integrity, and returns a Ruleset.
    ///
    /// `characteristics_json` is `Some` of an object `{ "start_points", "costs" }`,
    /// or `None` for a ruleset that ships no characteristic rules.
    pub fn from_core_json(
        id: &str,
        version: &str,
        point_items_json: &str,
        type_profiles_json: &str,
        abilities_json: &str,
        characteristics_json: Option<&str>,
    ) -> Result<Self, RulesetError> {
        Self::from_sources(RulesetSources {
            id,
            version,
            point_items: point_items_json,
            type_profiles: type_profiles_json,
            abilities: Some(abilities_json),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: characteristics_json,
            life_stages: None,
            childhoods: None,
            aging: None,
        })
    }

    /// Parses every language-neutral core source named in `sources`, validates
    /// referential integrity, and returns a Ruleset. This is the canonical
    /// loader (used in production by `arm-app`); [`Ruleset::from_json`],
    /// [`Ruleset::from_json_with_abilities`], and [`Ruleset::from_core_json`] are
    /// thin convenience constructors over it for tests that need only a subset of
    /// the sources.
    pub fn from_sources(sources: RulesetSources) -> Result<Self, RulesetError> {
        let RulesetSources { id, version, .. } = sources;
        let parsed = parse_sources(sources)?;

        let mut errors = check_duplicate_ids(&parsed);
        errors.extend(check_scholarly_language(&parsed.abilities_file));
        // The childhood block's own ability refs are checked in
        // `validate_childhood_refs`, called from `validate_integrity` — so a
        // cached ruleset arriving through `from_serialized` is held to the same
        // standard as a freshly parsed one.
        errors.extend(check_creation_phase_flow(&parsed.types));
        if !errors.is_empty() {
            return Err(IntegrityError::new(errors).into());
        }

        let ruleset = assemble_ruleset(id, version, parsed);
        ruleset.validate_integrity()?;
        Ok(ruleset)
    }

    /// Deserializes a previously-serialized [`Ruleset`] and RE-RUNS referential
    /// integrity validation. Use this for trusted/cached data; untrusted JSON must
    /// still go through [`Ruleset::from_json`]. Deriving `Deserialize` alone does
    /// NOT validate integrity — always reconstruct via this method.
    pub fn from_serialized(json: &str) -> Result<Self, RulesetError> {
        let mut ruleset: Ruleset = serde_json::from_str(json)?;
        // These are derived constants, not authored data: re-derive them (via
        // the same `apply_derived_fields` assemble_ruleset uses, V46) rather
        // than trusting the incoming JSON, so an older payload missing the
        // fields still yields a correct, non-empty ruleset.
        ruleset.apply_derived_fields();
        ruleset.validate_integrity()?;
        Ok(ruleset)
    }
}

/// Every language-neutral source file, parsed but not yet cross-checked or
/// indexed by id. The intermediate stage between [`parse_sources`] and
/// [`assemble_ruleset`] in the [`Ruleset::from_sources`] pipeline.
struct ParsedSources {
    items: Vec<PointItem>,
    types: Vec<EntityTypeProfile>,
    abilities_file: AbilitiesFile,
    arts_file: ArtsFile,
    houses_file: HousesFile,
    mythic_types_file: MythicCompanionTypesFile,
    spells_file: SpellsFile,
    spell_mastery_abilities_file: SpellMasteryAbilitiesFile,
    equipment_file: EquipmentFile,
    characteristic_rules: Option<CharacteristicRules>,
    life_stage_rules: Option<LifeStageRules>,
    childhoods_file: ChildhoodsFile,
    aging_rules: Option<AgingRules>,
}

/// Parses each named source string into its typed file shape. An absent
/// catalogue file (`None`) is equivalent to an empty `"{}"`, except for aging:
/// `AgingRules` has no meaningful zero (an aging table with no rows is broken,
/// not empty), so its absence stays an honest `None` instead of defaulting.
fn parse_sources(sources: RulesetSources) -> Result<ParsedSources, RulesetError> {
    let RulesetSources {
        point_items: point_items_json,
        type_profiles: type_profiles_json,
        abilities,
        arts,
        houses,
        mythic_types,
        spells,
        spell_mastery_abilities,
        equipment,
        characteristics,
        life_stages,
        childhoods,
        aging,
        ..
    } = sources;

    let items: Vec<PointItem> = serde_json::from_str(point_items_json)
        .map_err(|e| RulesetError::parse(parse_source::POINT_ITEMS, e))?;
    let types: Vec<EntityTypeProfile> = serde_json::from_str(type_profiles_json)
        .map_err(|e| RulesetError::parse(parse_source::TYPE_PROFILES, e))?;
    // An absent abilities file is equivalent to an empty `"{}"`.
    let abilities_file: AbilitiesFile = serde_json::from_str(abilities.unwrap_or("{}"))
        .map_err(|e| RulesetError::parse(parse_source::ABILITIES, e))?;
    // An absent arts file is equivalent to an empty `"{}"`.
    let arts_file: ArtsFile = serde_json::from_str(arts.unwrap_or("{}"))
        .map_err(|e| RulesetError::parse(parse_source::ARTS, e))?;
    // An absent houses file is equivalent to an empty `"{}"`.
    let houses_file: HousesFile = serde_json::from_str(houses.unwrap_or("{}"))
        .map_err(|e| RulesetError::parse(parse_source::HOUSES, e))?;
    // An absent mythic-types file is equivalent to an empty `"{}"`.
    let mythic_types_file: MythicCompanionTypesFile =
        serde_json::from_str(mythic_types.unwrap_or("{}"))
            .map_err(|e| RulesetError::parse(parse_source::MYTHIC_TYPES, e))?;
    // An absent spells file is equivalent to an empty `"{}"`.
    let spells_file: SpellsFile = serde_json::from_str(spells.unwrap_or("{}"))
        .map_err(|e| RulesetError::parse(parse_source::SPELLS, e))?;
    // An absent spell-mastery-abilities file is equivalent to an empty `"{}"`.
    let spell_mastery_abilities_file: SpellMasteryAbilitiesFile =
        serde_json::from_str(spell_mastery_abilities.unwrap_or("{}"))
            .map_err(|e| RulesetError::parse(parse_source::SPELL_MASTERY_ABILITIES, e))?;
    // An absent equipment file is equivalent to an empty `"{}"`.
    let equipment_file: EquipmentFile = serde_json::from_str(equipment.unwrap_or("{}"))
        .map_err(|e| RulesetError::parse(parse_source::EQUIPMENT, e))?;
    let characteristic_rules: Option<CharacteristicRules> = match characteristics {
        None => None,
        Some(json) => Some(
            serde_json::from_str(json)
                .map_err(|e| RulesetError::parse(parse_source::CHARACTERISTICS, e))?,
        ),
    };
    let life_stage_rules: Option<LifeStageRules> = match life_stages {
        None => None,
        Some(json) => Some(
            serde_json::from_str(json)
                .map_err(|e| RulesetError::parse(parse_source::LIFE_STAGES, e))?,
        ),
    };
    // An absent childhoods file is equivalent to an empty `"{}"`.
    let childhoods_file: ChildhoodsFile = serde_json::from_str(childhoods.unwrap_or("{}"))
        .map_err(|e| RulesetError::parse(parse_source::CHILDHOODS, e))?;
    // Unlike the catalogue files above, an absent aging file is NOT an empty
    // one: `AgingRules` has no meaningful zero (an aging table with no rows is
    // broken, not empty), so absence stays an honest `None`.
    let aging_rules: Option<AgingRules> = match aging {
        None => None,
        Some(json) => Some(
            serde_json::from_str(json).map_err(|e| RulesetError::parse(parse_source::AGING, e))?,
        ),
    };

    Ok(ParsedSources {
        items,
        types,
        abilities_file,
        arts_file,
        houses_file,
        mythic_types_file,
        spells_file,
        spell_mastery_abilities_file,
        equipment_file,
        characteristic_rules,
        life_stage_rules,
        childhoods_file,
        aging_rules,
    })
}

/// Detects duplicate IDs across each registry. Living Conditions are swept for
/// duplicates too, but from `validate_aging_rules` rather than from here:
/// unlike every catalogue below, they are not collapsed into a `BTreeMap` on
/// the way in, so a duplicate survives a round trip and `from_serialized` has
/// to catch it as well.
fn check_duplicate_ids(parsed: &ParsedSources) -> Vec<String> {
    let mut errors = Vec::new();
    collect_duplicates(
        parsed.items.iter().map(|i| &i.id),
        "point item",
        &mut errors,
    );
    collect_duplicates(
        parsed.types.iter().map(|t| &t.id),
        "type profile",
        &mut errors,
    );
    collect_duplicates(
        parsed.abilities_file.abilities.iter().map(|a| &a.id),
        "ability",
        &mut errors,
    );
    collect_duplicates(
        parsed.arts_file.arts.iter().map(|a| &a.id),
        "art",
        &mut errors,
    );
    collect_duplicates(
        parsed.houses_file.houses.iter().map(|h| &h.id),
        "house",
        &mut errors,
    );
    collect_duplicates(
        parsed.mythic_types_file.types.iter().map(|t| &t.id),
        "mythic companion type",
        &mut errors,
    );
    collect_duplicates(
        parsed.spells_file.spells.iter().map(|s| &s.id),
        "spell",
        &mut errors,
    );
    collect_duplicates(
        parsed
            .spell_mastery_abilities_file
            .abilities
            .iter()
            .map(|a| &a.id),
        "spell mastery ability",
        &mut errors,
    );
    collect_duplicates(
        parsed.equipment_file.weapons.iter().map(|w| &w.id),
        "weapon",
        &mut errors,
    );
    collect_duplicates(
        parsed.equipment_file.shields.iter().map(|s| &s.id),
        "shield",
        &mut errors,
    );
    collect_duplicates(
        parsed.equipment_file.armor.iter().map(|a| &a.id),
        "armor",
        &mut errors,
    );
    collect_duplicates(
        parsed.childhoods_file.packages.iter().map(|p| &p.id),
        "childhood package",
        &mut errors,
    );
    errors
}

/// The scholarly-language expectation names an ability, which must resolve and
/// be parameterized (a scholarly language is one instance of a dead language).
///
/// Its `exemplar` is **not** checked, and must not be: it is a label key pointing at
/// `exemplar.<slug>` in the i18n layer, not a `ref` into any catalogue. See
/// [`crate::AbilityRequirement::exemplar`] and `RULES.md`.
fn check_scholarly_language(abilities_file: &AbilitiesFile) -> Vec<String> {
    let mut errors = Vec::new();
    if let Some(requirement) = &abilities_file.scholarly_language {
        match abilities_file
            .abilities
            .iter()
            .find(|a| a.id == requirement.ability)
        {
            None => errors.push(format!(
                "scholarly-language requirement names unknown ability '{}'",
                requirement.ability
            )),
            Some(ability) if ability.parameter.is_none() => errors.push(format!(
                "scholarly-language ability '{}' takes no parameter, so it cannot name one language",
                requirement.ability
            )),
            Some(_) => {}
        }
    }
    errors
}

/// A declared creation flow must be walkable: no phase twice (the second visit's
/// Back would land where the user just was), and never `review`, which the
/// wizard appends itself as the terminal catch-all step. An empty list is legal
/// and simply means the type has no guided flow.
fn check_creation_phase_flow(types: &[EntityTypeProfile]) -> Vec<String> {
    let mut errors = Vec::new();
    for profile in types {
        let mut seen = BTreeSet::new();
        for phase in &profile.creation_phases {
            if *phase == CreationPhase::Review {
                errors.push(format!(
                    "type profile '{}' declares the synthetic '{phase}' phase, which the wizard appends itself",
                    profile.id
                ));
            } else if !seen.insert(phase) {
                errors.push(format!(
                    "type profile '{}' repeats creation phase '{phase}'",
                    profile.id
                ));
            }
        }
    }
    errors
}

/// Indexes a parsed list by id, cloning each item's id as the map key. Replaces
/// the twelve near-identical `.into_iter().map(|x| (x.id.clone(), x)).collect()`
/// conversions [`assemble_ruleset`] used to repeat inline (GC3).
fn index_by_id<T>(items: Vec<T>, id_of: impl Fn(&T) -> Id) -> BTreeMap<Id, T> {
    items.into_iter().map(|item| (id_of(&item), item)).collect()
}

/// Indexes every parsed catalogue by id and assembles the final [`Ruleset`].
/// Called only once pre-integrity checks (duplicates, scholarly language,
/// creation-phase flow) have passed with no errors.
fn assemble_ruleset(id: &str, version: &str, parsed: ParsedSources) -> Ruleset {
    let ParsedSources {
        items,
        types,
        abilities_file,
        arts_file,
        houses_file,
        mythic_types_file,
        spells_file,
        spell_mastery_abilities_file,
        equipment_file,
        characteristic_rules,
        life_stage_rules,
        childhoods_file,
        aging_rules,
    } = parsed;

    // The seven engine-derived fields below (magnitude_points through
    // aura_modifier_max) are placeholders, immediately overwritten by
    // `apply_derived_fields` — the same single derivation
    // `Ruleset::from_serialized` calls (V46), so the two construction paths
    // cannot silently diverge.
    let mut ruleset = Ruleset {
        id: Id::new(id),
        version: version.to_string(),
        point_items: index_by_id(items, |i| i.id.clone()),
        type_profiles: index_by_id(types, |t| t.id.clone()),
        abilities: index_by_id(abilities_file.abilities, |a| a.id.clone()),
        advancement: abilities_file.advancement,
        age_ability_caps: abilities_file.age_ability_caps,
        categories_requiring_virtue: abilities_file.categories_requiring_virtue,
        scholarly_language: abilities_file.scholarly_language,
        characteristic_rules,
        life_stages: life_stage_rules,
        childhoods: index_by_id(childhoods_file.packages, |p| p.id.clone()),
        aging: aging_rules,
        magnitude_points: BTreeMap::new(),
        ability_category_order: Vec::new(),
        arts: index_by_id(arts_file.arts, |a| a.id.clone()),
        art_advancement: arts_file.advancement,
        art_type_order: Vec::new(),
        reputation_type_order: Vec::new(),
        ritual_min_level: 0,
        aura_modifier_min: 0,
        aura_modifier_max: 0,
        houses: index_by_id(houses_file.houses, |h| h.id.clone()),
        mythic_companion_types: index_by_id(mythic_types_file.types, |t| t.id.clone()),
        spells: index_by_id(spells_file.spells, |s| s.id.clone()),
        spell_mastery_abilities: index_by_id(spell_mastery_abilities_file.abilities, |a| {
            a.id.clone()
        }),
        weapons: index_by_id(equipment_file.weapons, |w| w.id.clone()),
        shields: index_by_id(equipment_file.shields, |s| s.id.clone()),
        armor: index_by_id(equipment_file.armor, |a| a.id.clone()),
    };
    ruleset.apply_derived_fields();
    ruleset
}
