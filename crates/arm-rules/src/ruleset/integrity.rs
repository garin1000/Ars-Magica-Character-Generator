//! Referential-integrity validation: [`Ruleset::validate_integrity`] and its
//! private `validate_*` helpers. Split out of `ruleset.rs`; see `ruleset.rs`
//! for the [`Ruleset`] struct itself and `ruleset/parse.rs` for construction.
//!
//! [`validate_source_range`] is the GC2 fix: the "source line range is valid"
//! check used to be copy-pasted six lines at a time across nine call sites
//! (one per record kind). Each site now passes its own `subject` string —
//! the exact prefix it always used — so every site's error message is
//! byte-identical to before; only the duplicated check itself is shared.
//!
//! [`Ruleset::validate_integrity`], [`Ruleset::validate_childhood_packages`],
//! and [`Ruleset::validate_aging_rules`] were three of GC3's five over-long
//! functions; each is now a short orchestrator over named steps extracted
//! below it, with no change to the checks themselves or the order errors
//! accumulate in. [`Ruleset::validate_effect_refs`] (the fourth) is left
//! whole: it is one exhaustive `match` over [`Effect`] with no repeated
//! logic to factor out, and forcing an arbitrary split would cut across a
//! single enum match rather than along a real seam — see the doc comment on
//! it for the considered-and-rejected note.

use super::*;

impl Ruleset {
    /// Checks referential integrity: prerequisite refs, incompatibility symmetry,
    /// type profile trait refs, parameter domain refs, and source line ranges.
    pub fn validate_integrity(&self) -> Result<(), IntegrityError> {
        let mut errors = Vec::new();

        self.validate_point_items(&mut errors);
        self.validate_incompatibility_symmetry(&mut errors);
        self.validate_magnitude_variant_exclusivity(&mut errors);

        // The advancement table must have unique scores and non-decreasing
        // total_xp, or xp_to_raise's step subtraction would underflow later.
        errors.extend(self.advancement.validation_errors());
        // Same invariant for the Art advancement table.
        errors.extend(self.art_advancement.validation_errors());

        self.validate_childhood_refs(&mut errors);
        self.validate_apprenticeship_refs(&mut errors);
        self.validate_post_apprenticeship_rules(&mut errors);
        self.validate_aging_rules(&mut errors);
        self.validate_childhood_packages(&mut errors);

        self.validate_type_profile_refs(&mut errors);

        for house in self.houses.values() {
            self.validate_house_refs(house, &mut errors);
        }

        for mtype in self.mythic_companion_types.values() {
            self.validate_mythic_type_refs(mtype, &mut errors);
        }

        for spell in self.spells.values() {
            self.validate_spell_refs(spell, &mut errors);
        }

        for ability in self.spell_mastery_abilities.values() {
            validate_source_range(
                &ability.source,
                &format!("spell mastery ability '{}'", ability.id),
                &mut errors,
            );
        }

        for weapon in self.weapons.values() {
            self.validate_weapon_refs(weapon, &mut errors);
        }
        for shield in self.shields.values() {
            validate_source_range(
                &shield.source,
                &format!("shield '{}'", shield.id),
                &mut errors,
            );
        }
        for armor in self.armor.values() {
            validate_source_range(&armor.source, &format!("armor '{}'", armor.id), &mut errors);
        }

        self.validate_engine_required_roles(&mut errors);
        self.validate_engine_required_categories(&mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(IntegrityError::new(errors))
        }
    }

    /// Validates every point item's prerequisite refs, `incompatible_with`
    /// symmetric-target existence, effect refs, and source line range. Split
    /// out of [`Self::validate_integrity`] (GC3) as its own named step.
    fn validate_point_items(&self, errors: &mut Vec<String>) {
        for (id, item) in &self.point_items {
            if let Some(ref prereq) = item.prerequisites {
                self.validate_prereq_refs(prereq, id, errors);
            }

            for incompat_id in &item.incompatible_with {
                if !self.point_items.contains_key(incompat_id) {
                    errors.push(format!(
                        "{id}: incompatible_with references unknown ID '{incompat_id}'"
                    ));
                }
            }

            // Parameter domains are validated at parse time by the
            // ParameterDomain enum; concrete param VALUES are resolved per
            // selection in validation::validate_parameters.
            self.validate_effect_refs(item, id, errors);

            validate_source_range(&item.source, &format!("{id}"), errors);
        }
    }

    /// Validates each type profile's `required_traits`/`forbidden_traits`/
    /// `gift_id` references against the point-item catalogue. Split out of
    /// [`Self::validate_integrity`] (GC3) as its own named step.
    fn validate_type_profile_refs(&self, errors: &mut Vec<String>) {
        for (type_id, profile) in &self.type_profiles {
            for trait_id in &profile.required_traits {
                if !self.point_items.contains_key(trait_id) {
                    errors.push(format!(
                        "type profile '{type_id}': required_trait references unknown ID '{trait_id}'"
                    ));
                }
            }
            for trait_id in &profile.forbidden_traits {
                if !self.point_items.contains_key(trait_id) {
                    errors.push(format!(
                        "type profile '{type_id}': forbidden_trait references unknown ID '{trait_id}'"
                    ));
                }
            }
            if let Some(ref gift_id) = profile.gift_id
                && !self.point_items.contains_key(gift_id)
            {
                errors.push(format!(
                    "type profile '{type_id}': gift_id references unknown ID '{gift_id}'"
                ));
            }
            // Intentionally unchecked: the profile's category-typed fields
            // (`permitted_categories`, `forbidden_categories`, `gift_categories`,
            // and the budget's `flaw_category_caps`) are NOT validated against the
            // set of categories carried by point items. Categories are an open,
            // forward-declared namespace: the `point_items` catalogue is extracted
            // incrementally from the rules source, so a profile legitimately names
            // a category (e.g. `personality`, `story`, `supernatural`) before any
            // item in that category has been extracted yet — the shipped
            // `rules/core` data does exactly this. Requiring a backing item would
            // reject valid data, so this referential check is deliberately omitted
            // (tracked here rather than left silent).
        }
    }

    /// Checks that every engine-required Hermetic role ([`ENGINE_REQUIRED_ABILITIES`],
    /// [`ENGINE_REQUIRED_ARTS`]) resolves, failing loudly with the offending id.
    ///
    /// Gated on the ruleset declaring a magus profile — the condition under which
    /// the engine dereferences these ids while computing a magus's play-stats. A
    /// non-magus fixture is exempt entirely. The two role families are then gated
    /// separately on the catalogue each one lives in, mirroring
    /// [`Self::validate_engine_required_categories`]: the required **abilities**
    /// (e.g. Parma Magica, which `magic_resistance()` looks up by slug and
    /// silently treats as 0 if absent) are enforced whenever the ruleset ships an
    /// abilities catalogue, independent of Arts; the required **arts** are
    /// enforced only once an Arts catalogue is shipped. A magus fixture that ships
    /// neither catalogue is exempt from the corresponding family. This keeps the
    /// guard from rejecting valid partial rulesets while still catching a real
    /// Hermetic ruleset that renamed or dropped one of the roles — in particular a
    /// magus ruleset that ships abilities but no Arts is no longer waved through.
    fn validate_engine_required_roles(&self, errors: &mut Vec<String>) {
        let has_magus = self.type_profiles.values().any(|p| p.is_magus);
        if !has_magus {
            return;
        }
        // The engine-required abilities are dereferenced for a magus (e.g.
        // `magic_resistance()` looks up Parma Magica by slug and yields 0 if
        // absent) whenever the ruleset ships an abilities catalogue — the exact
        // condition under which one of them going missing is a real defect. This
        // is gated independently of the Arts catalogue (a magus ruleset can ship
        // abilities without Arts) and, mirroring the personality-category check,
        // exempts a fixture that ships no abilities catalogue at all.
        if !self.abilities.is_empty() {
            for required in ENGINE_REQUIRED_ABILITIES {
                let id = Id::new(required);
                if !self.abilities.contains_key(&id) {
                    errors.push(format!(
                        "engine-required ability '{id}' is missing from the catalogue"
                    ));
                }
            }
        }
        // The engine-required Arts are only dereferenced once the ruleset ships an
        // Arts catalogue; a magus ruleset with no Arts at all needs none of them.
        if self.arts.is_empty() {
            return;
        }
        for required in ENGINE_REQUIRED_ARTS {
            let id = Id::new(required);
            if !self.arts.contains_key(&id) {
                errors.push(format!(
                    "engine-required art '{id}' is missing from the catalogue"
                ));
            }
        }
    }

    /// Checks that a ruleset shipping a V/F catalogue carries the engine-required
    /// personality category ([`ENGINE_REQUIRED_CATEGORY_PERSONALITY`]), failing
    /// loudly with the category name if no point item declares it.
    ///
    /// Gated on the ruleset shipping any point items — the exact condition under
    /// which the engine's Major-Personality-Flaw rule (`validation/scores.rs`)
    /// dereferences the category. An empty V/F catalogue (a minimal or non-standard
    /// fixture) ships no Personality Flaws and needs none of it, so it is exempt;
    /// this mirrors how the Hermetic-role check is gated on the ruleset actually
    /// shipping the relevant catalogue.
    fn validate_engine_required_categories(&self, errors: &mut Vec<String>) {
        if self.point_items.is_empty() {
            return;
        }
        let has_personality = self
            .point_items
            .values()
            .any(|item| item.category == ENGINE_REQUIRED_CATEGORY_PERSONALITY);
        if !has_personality {
            errors.push(format!(
                "engine-required V/F category '{ENGINE_REQUIRED_CATEGORY_PERSONALITY}' \
                 is missing from the catalogue"
            ));
        }
    }

    /// Validates the childhood block's own ability references: every ability on
    /// the spread list must resolve, and so must the native-language ability —
    /// which must additionally be parameterized, since "the character's native
    /// language" is one instance among many and a plain Ability could not tell
    /// German from any other language.
    ///
    /// A typo here would silently shrink the list the guided flow offers instead
    /// of failing the load, which is why it is a load-time referential check like
    /// every other ref in the rules data. It runs from
    /// [`Ruleset::validate_integrity`] rather than [`Ruleset::from_sources`] so a
    /// cached ruleset returning through [`Ruleset::from_serialized`] — the
    /// documented integrity gate — is held to exactly the same standard.
    fn validate_childhood_refs(&self, errors: &mut Vec<String>) {
        let Some(rules) = &self.life_stages else {
            return;
        };
        for ability in &rules.childhood.spread_abilities {
            if !self.abilities.contains_key(ability) {
                errors.push(format!(
                    "life-stage childhood spread names unknown ability '{ability}'"
                ));
            }
        }
        let native = &rules.childhood.native_language_ability;
        match self.abilities.get(native) {
            None => errors.push(format!(
                "life-stage childhood names unknown native-language ability '{native}'"
            )),
            Some(ability) if ability.parameter.is_none() => errors.push(format!(
                "life-stage childhood native-language ability '{native}' takes no parameter, so it cannot name one language"
            )),
            Some(_) => {}
        }
    }

    /// Validates the apprenticeship block's own Ability references: every
    /// requirement — the minimums of `:2437` and the recommendations of
    /// `:2451-2461` — must name an Ability the catalogue knows, and a requirement
    /// that narrows itself to an *instance* must name a parameterized Ability, since
    /// a plain one has no instance to be.
    ///
    /// A typo here would silently drop a requirement no magus is then held to, which
    /// is why it is a load-time referential check like every other ref in the rules
    /// data. It runs from [`Ruleset::validate_integrity`] beside
    /// [`Self::validate_childhood_refs`], so a cached ruleset returning through
    /// [`Ruleset::from_serialized`] is held to the same standard.
    fn validate_apprenticeship_refs(&self, errors: &mut Vec<String>) {
        let Some(life_stages) = self.life_stages.as_ref() else {
            return;
        };
        let Some(apprenticeship) = life_stages.apprenticeship.as_ref() else {
            // A ruleset declaring Hermetic magi must declare their apprenticeship
            // too: a magus's later life runs only "until apprenticeship" (`:2214`,
            // `:2364`), so without the block the engine would cost a magus exactly as
            // it costs a companion — every year to its age, funding Arts out of a
            // child's experience. Gated on an `is_magus` profile, like
            // `validate_engine_required_roles`, because that is the condition under
            // which the missing block is a real defect.
            if self.type_profiles.values().any(|profile| profile.is_magus) {
                errors.push(
                    "life-stage rules ship no apprenticeship block, but the ruleset \
                     declares a magus type, whose later life ends at apprenticeship"
                        .to_string(),
                );
            }
            return;
        };
        let requirements = apprenticeship
            .minimum_abilities
            .iter()
            .chain(&apprenticeship.recommended_abilities);
        for requirement in requirements {
            let id = &requirement.ability;
            match self.abilities.get(id) {
                None => errors.push(format!(
                    "apprenticeship names unknown ability '{id}' as a requirement"
                )),
                Some(ability) if requirement.parameter.is_some() && ability.parameter.is_none() => {
                    errors.push(format!(
                        "apprenticeship requirement for '{id}' names an instance, \
                         but the ability takes no parameter"
                    ));
                }
                Some(_) => {}
            }
        }

        // The recommended set states its own total — "Total Cost: 90 experience
        // points" (Ars Magica - Definitive Edition (Core Rules).md:2461) — so the list must price to it off the
        // advancement table. The trust gate on transcribed data: a mistyped score
        // fails the load rather than shipping a recommendation the rulebook never
        // costed. The *minimum* set carries no total in the source (`:2437`), so it
        // is deliberately not priced — the engine would only be checking itself.
        let mut total = Some(0u32);
        for requirement in &apprenticeship.recommended_abilities {
            match self.advancement.xp_for_score(requirement.min_score) {
                // An unpriceable score is reported as itself, and the sum below then
                // stays silent rather than blaming a total it could not compute.
                None => {
                    total = None;
                    errors.push(format!(
                        "apprenticeship recommends '{}' at score {}, \
                         which the advancement table does not price",
                        requirement.ability, requirement.min_score
                    ));
                }
                Some(xp) => total = total.map(|sum| sum.saturating_add(xp)),
            }
        }
        if let Some(sum) = total
            && sum != apprenticeship.recommended_xp
        {
            errors.push(format!(
                "apprenticeship recommended abilities price to {sum} experience, \
                 not the stated {}",
                apprenticeship.recommended_xp
            ));
        }
    }

    /// Validates the years a magus lives after its Gauntlet: that a season of lab
    /// work costs something, that the charged seasons exhaust the year exactly, and
    /// that a ruleset declaring magi ships the block at all.
    ///
    /// The multiplication is **not** a sanity check, it is the rule:
    ///
    /// > For each season that your magus spends working on a lab project, the
    /// > character loses 10 points from the yearly 30 experience points, to a
    /// > minimum of 0 if three or four seasons are spent on lab work.
    ///
    /// The deduction *reaches* zero at three seasons, so three seasons at 10 must
    /// cancel the yearly 30 exactly — a year that overshoots would take points it
    /// never granted, and one that falls short would pay a magus for a year spent
    /// entirely in the lab. That makes the identity the **trust gate on three
    /// hand-transcribed numbers**, the same idiom as re-pricing the apprenticeship's
    /// `recommended_xp` against the advancement table.
    ///
    /// Runs from [`Ruleset::validate_integrity`] beside
    /// [`Self::validate_apprenticeship_refs`], so a cached ruleset returning through
    /// [`Ruleset::from_serialized`] is held to the same standard.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2471, :2482.
    fn validate_post_apprenticeship_rules(&self, errors: &mut Vec<String>) {
        let Some(life_stages) = self.life_stages.as_ref() else {
            return;
        };
        let Some(post) = life_stages.post_apprenticeship.as_ref() else {
            // A ruleset declaring Hermetic magi must declare their years after the
            // Gauntlet too. The apprenticeship block already ends a magus's later
            // life at its Gauntlet age (`:2364`); with nothing granted for the years
            // after it, a magus would simply lose them. Gated on an `is_magus`
            // profile, exactly like the apprenticeship block above.
            if self.type_profiles.values().any(|profile| profile.is_magus) {
                errors.push(
                    "life-stage rules ship no post-apprenticeship block, but the ruleset \
                     declares a magus type, whose years after the Gauntlet would then \
                     earn nothing"
                        .to_string(),
                );
            }
            return;
        };
        if post.lab_season_cost == 0 {
            errors.push(
                "post-apprenticeship lab_season_cost is 0, so no season of lab work \
                 would ever cost a magus anything"
                    .to_string(),
            );
        }
        let charged = post
            .lab_season_cost
            .saturating_mul(post.max_charged_lab_seasons_per_year);
        if charged != post.points_per_year {
            errors.push(format!(
                "post-apprenticeship lab seasons cost {} × {} = {charged} points, \
                 not the {} a year grants, so the deduction never lands on 0",
                post.lab_season_cost, post.max_charged_lab_seasons_per_year, post.points_per_year
            ));
        }
    }

    /// Validates the aging tables: that no Living Condition id is used twice, that
    /// the Aging Roll table tiles every total it will be asked about, that each of
    /// its rows actually costs something, and that a Longevity Ritual clamp does
    /// what the rulebook says it is *for*.
    ///
    /// The tiling check ([`Self::validate_aging_outcome_tiling`]) and the two
    /// bound checks ([`Self::validate_aging_outcome_bounds`]) are split out as
    /// named steps (GC3); this orchestrator otherwise runs unchanged.
    ///
    /// Runs from [`Ruleset::validate_integrity`] beside
    /// [`Self::validate_post_apprenticeship_rules`], so a cached ruleset returning
    /// through [`Ruleset::from_serialized`] is held to the same standard. An
    /// absent aging block stands the whole subsystem down and is not an error.
    ///
    /// **Considered and rejected** (recorded so they are not re-litigated):
    /// - *"exactly one Living Condition with `modifier == 0` and
    ///   `cumulative == false`"* — the baseline row is a UI affordance, not a
    ///   rule: an empty set of conditions already contributes 0. It would catch no
    ///   transcription error worth catching.
    /// - *`start_age == longevity_clamp.until_age`* — two numbers from two
    ///   different sentences (`:16565` and `:16575`) that happen to coincide.
    ///   Asserting equality would invent a relationship the rules never state.
    /// - *requiring the block whenever the ruleset declares characters* — an
    ///   absent `Option` stands the subsystem down, which is the house position
    ///   stated on [`Ruleset::aging`].
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16567, :16575,
    /// :16577, :16597-16611.
    fn validate_aging_rules(&self, errors: &mut Vec<String>) {
        let Some(aging) = self.aging.as_ref() else {
            return;
        };

        // Living Conditions are a catalogue like any other, so their ids join the
        // duplicate sweep — but from here rather than from `from_sources`, because
        // they are the one swept catalogue the [`Ruleset`] keeps as a `Vec`
        // instead of collapsing into a `BTreeMap`. A duplicate therefore survives
        // serialization, and only a check on this path catches it when a cached
        // ruleset comes back through [`Ruleset::from_serialized`].
        collect_duplicates(
            aging.living_conditions.iter().map(|c| &c.id),
            "living condition",
            errors,
        );

        // The age term is "age/10 (round up)" (`:16567`); a divisor of 0 has no
        // rounding-up to do, it has a division by zero.
        if aging.age_divisor == 0 {
            errors.push(
                "aging age_divisor is 0, but the aging total adds the character's age \
                 divided by it, rounded up"
                    .to_string(),
            );
        }

        let Some(first) = aging.outcomes.first() else {
            errors.push(
                "aging rules ship no outcome rows, so no aging total would ever have a result"
                    .to_string(),
            );
            return;
        };

        let last_index = aging.outcomes.len() - 1;
        self.validate_aging_outcome_tiling(&aging.outcomes, errors);
        self.validate_aging_outcome_bounds(aging, first, last_index, errors);

        self.validate_crisis_rules(errors);
    }

    /// Per-row ordering for the Aging Roll table: rows must ascend by `min`,
    /// consecutive rows must not gap or overlap, and only the last row may
    /// leave its `max` open. Mirrors [`Self::validate_crisis_tiling`]'s job for
    /// the Crisis Table. Split out of [`Self::validate_aging_rules`] (GC3) as
    /// its own named step.
    fn validate_aging_outcome_tiling(&self, outcomes: &[AgingRow], errors: &mut Vec<String>) {
        let last_index = outcomes.len() - 1;
        for (index, row) in outcomes.iter().enumerate() {
            self.validate_aging_row_effect(row.min, &row.effect, errors);

            match row.max {
                None if index != last_index => {
                    errors.push(format!(
                        "aging outcome row starting at {} has no upper bound but is not the \
                         last row: only the open-ended top row may omit its maximum",
                        row.min
                    ));
                    continue;
                }
                Some(max) if max < row.min => {
                    errors.push(format!(
                        "aging outcome row spans {} to {max}, whose upper bound is below its \
                         lower bound, so it covers no total at all",
                        row.min
                    ));
                    continue;
                }
                _ => {}
            }

            let Some(next) = outcomes.get(index + 1) else {
                break;
            };
            if next.min <= row.min {
                errors.push(format!(
                    "aging outcome rows are not in ascending order: the row starting at {} \
                     is followed by one starting at {}",
                    row.min, next.min
                ));
                continue;
            }
            // Unreachable for the open-ended row, which the match above already
            // established is the last one.
            let Some(max) = row.max else { continue };
            if next.min > max.saturating_add(1) {
                errors.push(format!(
                    "aging outcome rows leave a gap: the row ending at {max} is followed by \
                     one starting at {}, so totals {} to {} land on no row",
                    next.min,
                    max.saturating_add(1),
                    next.min.saturating_sub(1)
                ));
            } else if next.min <= max {
                errors.push(format!(
                    "aging outcome rows overlap: the row ending at {max} is followed by one \
                     starting at {}, so totals {} to {max} land on two rows",
                    next.min, next.min
                ));
            }
        }
    }

    /// The three Aging Roll bound checks that are not about tiling: the top row
    /// must be open-ended ("22+", `:16611`), the low-roll "no older" exception
    /// (`:16577`) must not sit above the first Aging-Point row, and a Longevity
    /// Ritual clamp (`:16575`) must clear that same row for its stated purpose to
    /// hold. Split out of [`Self::validate_aging_rules`] (GC3) as its own named
    /// step.
    fn validate_aging_outcome_bounds(
        &self,
        aging: &AgingRules,
        first: &AgingRow,
        last_index: usize,
        errors: &mut Vec<String>,
    ) {
        // "22+" (`:16611`) catches every total above the table; a bounded top row
        // would let a high total fall off the end with no result at all.
        if let Some(max) = aging.outcomes[last_index].max {
            errors.push(format!(
                "the highest aging outcome row ends at {max} rather than being open-ended, \
                 so any higher total would land on no row"
            ));
        }

        // "Particularly low rolls on the table mean that the character appears no
        // older. Otherwise, the character's apparent age increases by one year"
        // (`:16577`) — low rolls being the *only* exception, no row that costs
        // Aging Points may sit below the threshold.
        if aging.apparent_age_increase_min > first.min {
            errors.push(format!(
                "aging apparent_age_increase_min is {}, above the first aging-point row at \
                 {}: a total could then cost Aging Points while leaving the character \
                 looking no older, though ':16577' makes particularly low rolls the only \
                 exception",
                aging.apparent_age_increase_min, first.min
            ));
        }

        // The trust gate: the clamp's stated purpose holds only while the clamped
        // total stays below the first row that costs anything.
        if let Some(clamp) = aging.longevity_clamp.as_ref()
            && clamp.max_total >= first.min
        {
            errors.push(format!(
                "a Longevity Ritual clamps aging totals to {}, which does not clear the \
                 first aging-point row at {}: the clamp exists so its bearer 'is at no risk \
                 of actually aging before any other characters' \
                 (Ars Magica - Definitive Edition (Core Rules).md:16575), which holds only \
                 while the clamped total stays below that row",
                clamp.max_total, first.min
            ));
        }
    }

    /// Validates the Crisis Table: that no row id is used twice, that the rows
    /// tile every crisis total between the table's two open ends, that the illness
    /// rows climb as one ladder, that the attending doctor's Ability resolves, and
    /// that the two Decrepitude thresholds of `:16617` sit in the right order.
    ///
    /// The tiling check is **contiguity**, like the Aging Roll table's, and for
    /// the same reason — but with a second open end. "8 or less" (`:16626`) has no
    /// lower bound and "19+" (`:16632`) no upper one, so the first row and only the
    /// first may omit its minimum, the last and only the last its maximum, and
    /// between them no total may land on two rows or on none.
    ///
    /// The ladder check is licensed by one sentence: "The level of spell required
    /// depends on the severity of the crisis, as noted on the table." (`:16638`)
    /// That makes severity a **rank** rather than a label, and makes the required
    /// Ritual level a function of it — so severity and Ritual level must both climb
    /// strictly down the illness rows, or the stated dependency does not hold. The
    /// Ease Factor rides along on the same argument: a more severe crisis that were
    /// easier to survive would invert the ladder the sentence names. All three are
    /// read off the **illness rows only** — a bedridden row has no severity to
    /// compare — with the bedridden rows pinned to the front of the table instead.
    ///
    /// Runs from [`Self::validate_aging_rules`], so a cached ruleset returning
    /// through [`Ruleset::from_serialized`] is held to the same standard. An absent
    /// `crisis` block stands the crisis subsystem down and is not an error, which
    /// is the house position stated on [`Ruleset::aging`].
    ///
    /// **Considered and rejected** (recorded so they are not re-litigated):
    /// - *the `+3` Ease-Factor and `+5` Ritual-level steps* — the shipped columns
    ///   do step by exactly 3 and 5 (`:16628-16632`), but the rulebook never states
    ///   that relation, and gating it would refuse a legitimate house table. Slice
    ///   6b6 rejected `start_age == longevity_clamp.until_age` on the same ground:
    ///   a gate must not invent a relationship the rules do not state.
    /// - *`ritual_level` == the Creo Corpus guideline + 5* — the guidelines at
    ///   `:13372-13376` price a minor/serious/major/critical/terminal aging crisis
    ///   at 15/20/25/30/35, exactly 5 below the table's 20/25/30/35/40, which is the
    ///   `+1` Touch magnitude of a Ritual cast on someone else. But the guidelines
    ///   are not loaded data and the `+5` is an inference, not a stated rule. It
    ///   belongs in `RULES.md` as provenance — so nobody "fixes" one table against
    ///   the other — not in a gate.
    /// - *any literal band boundary* (e.g. "the table must cover 15..=19") — that
    ///   hardcodes the shipped numbers in Rust, which is the catalogue-size-is-data
    ///   violation. Transcription is already pinned by
    ///   `shipped_crisis_table_carries_the_16626_to_16632_rows` (`data_integrity.rs`).
    /// - *"exactly five illness rows and two bedridden rows"* — catalogue size is
    ///   data, never code.
    /// - *"every [`CrisisSeverity`] variant is used by the shipped rows"* — a house
    ///   ruleset that omits Terminal would fail to load for no good reason.
    /// - *"a `crisis` block is required whenever `outcomes` contains
    ///   `next_decrepitude_level_and_crisis`"* — an absent `Option` stands the
    ///   subsystem down, and the player learns about it from a refusal at command
    ///   time rather than from a ruleset that will not load at all.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16617,
    /// :16624-16634, :16638.
    fn validate_crisis_rules(&self, errors: &mut Vec<String>) {
        let Some(aging) = self.aging.as_ref() else {
            return;
        };
        let Some(crisis) = aging.crisis.as_ref() else {
            return;
        };

        // The rows stay a `Vec` inside [`CrisisRules`], exactly like the Living
        // Conditions above, so a duplicate id survives serialization and only a
        // check on this path catches it on the way back in.
        collect_duplicates(crisis.rows.iter().map(|row| &row.id), "crisis row", errors);

        let Some(first) = crisis.rows.first() else {
            errors.push(
                "crisis rules ship no crisis rows, so no crisis total would ever have a result"
                    .to_string(),
            );
            return;
        };
        let last_index = crisis.rows.len() - 1;
        let last = &crisis.rows[last_index];

        // "8 or less" (`:16626`) opens the table below and "19+" (`:16632`) closes
        // it above. A bounded end would let a total fall off the table with no
        // result at all.
        if let Some(min) = first.min {
            errors.push(format!(
                "the first crisis row '{}' starts at {min} rather than being open below, \
                 so any lower total would land on no row",
                first.id
            ));
        }
        if let Some(max) = last.max {
            errors.push(format!(
                "the last crisis row '{}' ends at {max} rather than being open above, \
                 so any higher total would land on no row",
                last.id
            ));
        }

        for (index, row) in crisis.rows.iter().enumerate() {
            if index != 0 && row.min.is_none() {
                errors.push(format!(
                    "crisis row '{}' has no lower bound but is not the first row: only the \
                     row open below may omit its minimum, or every total beneath it would \
                     land on two rows",
                    row.id
                ));
            }
            if index != last_index && row.max.is_none() {
                errors.push(format!(
                    "crisis row '{}' has no upper bound but is not the last row: only the \
                     row open above may omit its maximum, or nothing after it would ever \
                     be reached",
                    row.id
                ));
            }
            if let (Some(min), Some(max)) = (row.min, row.max)
                && max < min
            {
                errors.push(format!(
                    "crisis row '{}' spans {min} to {max}, whose upper bound is below its \
                     lower bound, so it covers no total at all",
                    row.id
                ));
            }
        }

        self.validate_crisis_tiling(&crisis.rows, errors);
        self.validate_crisis_ladder(&crisis.rows, errors);

        // "An Int + Medicine roll against an Ease Factor of 6 allows the character
        // to add the attendant's Medicine score" (`:16634`) names the Ability by
        // id, so a typo would leave the survival read-out quietly finding no score
        // to add — a referential-integrity failure like every other ref in the
        // rules data.
        if let Some(attendant) = crisis.attendant.as_ref()
            && !self.abilities.contains_key(&attendant.ability)
        {
            errors.push(format!(
                "crisis attendant names unknown ability '{}'",
                attendant.ability
            ));
        }

        // "Characters with a Decrepitude score of 4 are extremely frail … Characters
        // with a Decrepitude score of 5 are bedridden and will die" (`:16617`) — two
        // thresholds on one ascending track, so frailty must be reached first. The
        // pair rides with the crisis block because the crisis subsystem is the only
        // thing that reads it.
        if let (Some(frail), Some(fatal)) =
            (aging.frail_decrepitude_score, aging.fatal_decrepitude_score)
            && frail >= fatal
        {
            errors.push(format!(
                "aging frail_decrepitude_score is {frail} and fatal_decrepitude_score is \
                 {fatal}, so a character would be dead before he ever turned frail \
                 (Ars Magica - Definitive Edition (Core Rules).md:16617)"
            ));
        }
    }

    /// The Crisis Table answers every crisis total, so consecutive rows must abut
    /// exactly: `next.min == this.max + 1`. A gap leaves a total with no row, an
    /// overlap gives it two.
    ///
    /// Rows with an open end are skipped here — whether they are allowed to have
    /// one is [`Self::validate_crisis_rules`]'s question, not this one's.
    fn validate_crisis_tiling(&self, rows: &[CrisisRow], errors: &mut Vec<String>) {
        for (index, row) in rows.iter().enumerate() {
            let (Some(next), Some(max)) = (rows.get(index + 1), row.max) else {
                continue;
            };
            let Some(next_min) = next.min else {
                continue;
            };
            if next_min > max.saturating_add(1) {
                errors.push(format!(
                    "crisis rows leave a gap: the row '{}' ending at {max} is followed by \
                     '{}' starting at {next_min}, so totals {} to {} land on no row",
                    row.id,
                    next.id,
                    max.saturating_add(1),
                    next_min.saturating_sub(1)
                ));
            } else if next_min <= max {
                errors.push(format!(
                    "crisis rows overlap: the row '{}' ending at {max} is followed by '{}' \
                     starting at {next_min}, so totals {next_min} to {max} land on two rows",
                    row.id, next.id
                ));
            }
        }
    }

    /// The illness ladder of `:16638`: severity, required Ritual level and Ease
    /// Factor all climb together down the illness rows, and the bedridden rows —
    /// which carry no severity at all — sit in front of every illness.
    ///
    /// Only the **last** illness row may omit its Ease Factor: "**Terminal
    /// illness**. CrCo40 required to survive." (`:16632`) offers no Stamina roll,
    /// and a milder row that offered none either would be unsurvivable without
    /// magic while a worse one was not.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16626-16632, :16638.
    fn validate_crisis_ladder(&self, rows: &[CrisisRow], errors: &mut Vec<String>) {
        let mut illnesses: Vec<(&Id, CrisisSeverity, Option<i32>, u32)> = Vec::new();
        let mut first_illness: Option<&Id> = None;
        for row in rows {
            match &row.outcome {
                CrisisOutcome::Bedridden => {
                    if let Some(illness) = first_illness {
                        errors.push(format!(
                            "crisis row '{}' is bedridden but follows the illness row \
                             '{illness}': the table's bedridden results are its mildest \
                             (Ars Magica - Definitive Edition (Core Rules).md:16626-16627) \
                             and must come before every illness",
                            row.id
                        ));
                    }
                }
                CrisisOutcome::Illness {
                    severity,
                    ease_factor,
                    ritual_level,
                } => {
                    if first_illness.is_none() {
                        first_illness = Some(&row.id);
                    }
                    illnesses.push((&row.id, *severity, *ease_factor, *ritual_level));
                }
            }
        }

        let last_illness = illnesses.len().saturating_sub(1);
        for (index, (id, severity, ease_factor, ritual_level)) in illnesses.iter().enumerate() {
            if ease_factor.is_none() && index != last_illness {
                errors.push(format!(
                    "crisis illness row '{id}' offers no Ease Factor but is not the most \
                     severe illness on the table: only the last illness row may forgo the \
                     Stamina roll (Ars Magica - Definitive Edition (Core Rules).md:16632)"
                ));
            }

            let Some((next_id, next_severity, next_ease_factor, next_ritual_level)) =
                illnesses.get(index + 1)
            else {
                continue;
            };
            if next_severity <= severity {
                errors.push(format!(
                    "crisis illness rows do not climb in severity: '{id}' ({severity:?}) is \
                     followed by '{next_id}' ({next_severity:?}), though 'the level of spell \
                     required depends on the severity of the crisis, as noted on the table' \
                     (Ars Magica - Definitive Edition (Core Rules).md:16638)"
                ));
            }
            if next_ritual_level <= ritual_level {
                errors.push(format!(
                    "crisis illness rows do not climb in required Ritual level: '{id}' \
                     requires CrCo{ritual_level} and is followed by '{next_id}' requiring \
                     CrCo{next_ritual_level}, though 'the level of spell required depends on \
                     the severity of the crisis' \
                     (Ars Magica - Definitive Edition (Core Rules).md:16638)"
                ));
            }
            if let (Some(ease), Some(next_ease)) = (ease_factor, next_ease_factor)
                && next_ease <= ease
            {
                errors.push(format!(
                    "crisis illness rows do not climb in Ease Factor: '{id}' rolls against \
                     {ease} and is followed by '{next_id}' rolling against {next_ease}, \
                     though a more severe crisis cannot be easier to survive \
                     (Ars Magica - Definitive Edition (Core Rules).md:16638)"
                ));
            }
        }
    }

    /// Per-row sanity for one Aging Roll outcome: a row that awards no Aging
    /// Points is a no-op that reads as a rule, and a row that names
    /// Characteristics must name at least one, each of them once — "1 Aging Point
    /// in Str and Sta" (Ars Magica - Definitive Edition (Core Rules).md:16607) gives *each* named Characteristic a
    /// point, so a repeat would silently double it.
    fn validate_aging_row_effect(
        &self,
        min: i32,
        effect: &AgingRowEffect,
        errors: &mut Vec<String>,
    ) {
        match effect {
            AgingRowEffect::AnyCharacteristic { points } => {
                if *points == 0 {
                    errors.push(format!(
                        "aging outcome row starting at {min} awards 0 Aging Points, \
                         so landing on it would cost nothing"
                    ));
                }
            }
            AgingRowEffect::NamedCharacteristics {
                points,
                characteristics,
            } => {
                if *points == 0 {
                    errors.push(format!(
                        "aging outcome row starting at {min} awards 0 Aging Points, \
                         so landing on it would cost nothing"
                    ));
                }
                if characteristics.is_empty() {
                    errors.push(format!(
                        "aging outcome row starting at {min} names no Characteristic \
                         to take its Aging Points"
                    ));
                }
                let mut seen = BTreeSet::new();
                for characteristic in characteristics {
                    if !seen.insert(characteristic) {
                        errors.push(format!(
                            "aging outcome row starting at {min} names Characteristic \
                             '{characteristic}' twice, which would silently double its \
                             Aging Points"
                        ));
                    }
                }
            }
            // A crisis row costs whatever it takes to reach the next Decrepitude
            // level, so there is no number of its own to check.
            AgingRowEffect::NextDecrepitudeLevelAndCrisis => {}
        }
    }

    /// Validates the Sample Childhood packages against the abilities catalogue,
    /// the advancement table, and the childhood blocks they are a shortcut for.
    ///
    /// This is the **trust gate on transcribed rulebook data**. A package is one
    /// way of spending the two childhood blocks
    /// ("75 experience points in their native language … and 45 experience points
    /// to divide between …", Ars Magica - Definitive Edition (Core Rules).md:2378), so its entries must price to
    /// exactly those blocks: a mistyped score fails the load rather than shipping
    /// a package that quietly costs 40 or 50 experience points. It also makes
    /// [`ChildhoodPackage::native_entry`]'s "at most one native entry" a real
    /// guarantee rather than a hope.
    ///
    /// Errors accumulate — a broken file reports every problem at once. Per-package
    /// entry checks ([`Self::validate_childhood_package_entries`]) and pricing
    /// checks against the childhood block ([`Self::validate_childhood_package_pricing`])
    /// are split out as named steps (GC3); this orchestrator otherwise runs
    /// unchanged.
    fn validate_childhood_packages(&self, errors: &mut Vec<String>) {
        if self.childhoods.is_empty() {
            return;
        }
        // Without the blocks there is nothing to price a package against, so the
        // pricing rules below stay silent rather than blaming each package for
        // the missing file.
        let childhood = match self.life_stages {
            Some(ref rules) => Some(&rules.childhood),
            None => {
                errors.push(
                    "childhood packages are shipped without life-stage rules, \
                     so their experience cannot be priced"
                        .to_string(),
                );
                None
            }
        };

        for package in self.childhoods.values() {
            self.validate_childhood_package_entries(package, childhood, errors);
            if let Some(childhood) = childhood {
                self.validate_childhood_package_pricing(package, childhood, errors);
            }

            validate_source_range(
                &package.source,
                &format!("childhood package '{}'", package.id),
                errors,
            );
        }
    }

    /// Validates one package's entries: every ability must resolve, a
    /// parameterized ability needs a slot key (unless it is the native entry),
    /// a plain ability must NOT name a slot, two entries of one parameterized
    /// ability must use distinct slots, exactly one entry must be native, every
    /// entry's score must be priceable, and — when the childhood block is known
    /// — every non-native entry's ability must be on the spread list. Split out
    /// of [`Self::validate_childhood_packages`] (GC3) as its own named step.
    fn validate_childhood_package_entries(
        &self,
        package: &ChildhoodPackage,
        childhood: Option<&ChildhoodRules>,
        errors: &mut Vec<String>,
    ) {
        let id = &package.id;
        let mut slots_seen: BTreeSet<&str> = BTreeSet::new();
        let mut native_entries = 0usize;

        for entry in &package.entries {
            let ability = &entry.ability;
            match self.abilities.get(ability) {
                None => errors.push(format!(
                    "childhood package '{id}' names unknown ability '{ability}'"
                )),
                // A parameterized ability needs a slot key for the player's
                // answer to arrive under; the native language is the
                // exception, being chosen once for the character as a whole.
                Some(known) if known.parameter.is_some() => {
                    if entry.slot.is_none() && !entry.native {
                        errors.push(format!(
                            "childhood package '{id}' entry for parameterized ability \
                             '{ability}' names no slot, so its value could never be asked for"
                        ));
                    }
                }
                Some(_) => {
                    if let Some(slot) = entry.slot.as_deref() {
                        errors.push(format!(
                            "childhood package '{id}' entry for plain ability '{ability}' \
                             names slot '{slot}', which it has no parameter to fill"
                        ));
                    }
                }
            }

            // Two entries of one parameterized ability are told apart by slot
            // alone, so a repeated key would collapse them into one row.
            if let Some(slot) = entry.slot.as_deref()
                && !slots_seen.insert(slot)
            {
                errors.push(format!("childhood package '{id}' repeats slot '{slot}'"));
            }

            if entry.native {
                native_entries += 1;
            }

            // An unpriceable score is reported here, per entry; the two sum
            // checks below then stay silent rather than blaming the total.
            if self.advancement.xp_for_score(entry.score).is_none() {
                errors.push(format!(
                    "childhood package '{id}' entry for '{ability}' has score {}, \
                     which the advancement table does not price",
                    entry.score
                ));
            }

            if let Some(childhood) = childhood
                && !entry.native
                && !childhood.spread_abilities.contains(ability)
            {
                errors.push(format!(
                    "childhood package '{id}' names ability '{ability}', \
                     which the childhood spread cannot buy"
                ));
            }
        }

        if native_entries != 1 {
            errors.push(format!(
                "childhood package '{id}' has {native_entries} native entries, \
                 but exactly one is required"
            ));
        }
    }

    /// Validates one package's pricing against the childhood block it is a
    /// shortcut for: the native entry must name the childhood's own
    /// native-language ability, and the spread/native entries must each price to
    /// the childhood's stated experience totals. Split out of
    /// [`Self::validate_childhood_packages`] (GC3) as its own named step; only
    /// called when the childhood block is known (an absent one already reported
    /// its own error in the orchestrator).
    fn validate_childhood_package_pricing(
        &self,
        package: &ChildhoodPackage,
        childhood: &ChildhoodRules,
        errors: &mut Vec<String>,
    ) {
        let id = &package.id;
        if let Some(native) = package.native_entry()
            && native.ability != childhood.native_language_ability
        {
            errors.push(format!(
                "childhood package '{id}' native entry names ability '{}', not the \
                 childhood's native-language ability '{}'",
                native.ability, childhood.native_language_ability
            ));
        }
        if let Some(sum) = package.spread_xp(&self.advancement)
            && sum != childhood.spread_xp
        {
            errors.push(format!(
                "childhood package '{id}' spread entries price to {sum} experience, \
                 not the childhood spread's {}",
                childhood.spread_xp
            ));
        }
        if let Some(sum) = package.native_xp(&self.advancement)
            && sum != childhood.native_language_xp
        {
            errors.push(format!(
                "childhood package '{id}' native entry prices to {sum} experience, \
                 not the childhood's {}",
                childhood.native_language_xp
            ));
        }
    }

    /// Validates a House record: every Virtue/Flaw it can grant must resolve to a
    /// known point item, and its source line range (if any) must be well-formed.
    ///
    /// A `Fixed` grant and each `Choice` option name a concrete point-item id, so
    /// those are integrity-checked here. An `Open` grant carries no item id (the
    /// player picks one at runtime, validated against its `GrantConstraint` in
    /// [`crate::validation`]), so there is nothing to resolve at load. Grant
    /// `params` values (the `ability`/`art` a Puissant targets) are deliberately
    /// NOT registry-checked, mirroring the forward-declared Ability/Art-domain
    /// policy on parameter values elsewhere.
    fn validate_house_refs(&self, house: &House, errors: &mut Vec<String>) {
        let id = &house.id;
        self.validate_grant_refs("house", id, &house.grants, errors);
        validate_source_range(&house.source, &format!("house '{id}'"), errors);
    }

    /// Validates that every concrete item id a grant list names resolves. A
    /// `Fixed` grant and each `Choice` option name a point-item id; an `Open`
    /// grant carries none (checked at runtime against its `GrantConstraint`).
    /// Shared by House and Mythic-Companion-type integrity checks; `kind` and
    /// `owner` label the offending record in the error message. Grant `params`
    /// values are deliberately NOT registry-checked, mirroring the
    /// forward-declared Ability/Art-domain policy on parameter values elsewhere.
    fn validate_grant_refs(
        &self,
        kind: &str,
        owner: &Id,
        grants: &[Grant],
        errors: &mut Vec<String>,
    ) {
        for grant in grants {
            match grant {
                Grant::Fixed { item, .. } => {
                    if !self.point_items.contains_key(item) {
                        errors.push(format!(
                            "{kind} '{owner}': fixed grant references unknown item '{item}'"
                        ));
                    }
                }
                Grant::Choice { options, .. } => {
                    for option in options {
                        if !self.point_items.contains_key(&option.item_ref) {
                            errors.push(format!(
                                "{kind} '{owner}': choice option references unknown item '{}'",
                                option.item_ref
                            ));
                        }
                    }
                }
                // Open grants resolve to a player pick at runtime — nothing here.
                Grant::Open { .. } => {}
            }
        }
    }

    /// Validates a Mythic Companion type: every grant item, every required Virtue,
    /// and every required Flaw's default must resolve to a known point item, and
    /// its source line range (if any) must be well-formed. This is the load-time
    /// trust gate that a type's package can only ship once all its items are
    /// seeded. Required-flaw substitute *constraints* name categories (an open,
    /// forward-declared namespace), so they are not registry-checked — mirroring
    /// the profile category-field policy above.
    fn validate_mythic_type_refs(&self, mtype: &MythicCompanionType, errors: &mut Vec<String>) {
        let id = &mtype.id;
        self.validate_grant_refs("mythic companion type", id, &mtype.grants, errors);
        for req in &mtype.required_virtues {
            if !self.point_items.contains_key(&req.item_ref) {
                errors.push(format!(
                    "mythic companion type '{id}': required virtue references unknown item '{}'",
                    req.item_ref
                ));
            }
        }
        for flaw in &mtype.required_flaws {
            if !self.point_items.contains_key(&flaw.default.item_ref) {
                errors.push(format!(
                    "mythic companion type '{id}': required flaw default references unknown item '{}'",
                    flaw.default.item_ref
                ));
            }
        }
        validate_source_range(
            &mtype.source,
            &format!("mythic companion type '{id}'"),
            errors,
        );
    }

    /// Validates a spell: its Technique must resolve to a Technique-class Art, its
    /// Form to a Form-class Art, every requisite to a known Art, and its source
    /// line range (if any) must be well-formed. This is the load-time trust gate
    /// that a spell can only ship once its Arts exist.
    fn validate_spell_refs(&self, spell: &Spell, errors: &mut Vec<String>) {
        let id = &spell.id;
        match self.arts.get(&spell.technique) {
            None => errors.push(format!(
                "spell '{id}': technique references unknown art '{}'",
                spell.technique
            )),
            Some(art) if art.art_type != ArtType::Technique => errors.push(format!(
                "spell '{id}': technique '{}' is not a Technique-class Art",
                spell.technique
            )),
            Some(_) => {}
        }
        match self.arts.get(&spell.form) {
            None => errors.push(format!(
                "spell '{id}': form references unknown art '{}'",
                spell.form
            )),
            Some(art) if art.art_type != ArtType::Form => errors.push(format!(
                "spell '{id}': form '{}' is not a Form-class Art",
                spell.form
            )),
            Some(_) => {}
        }
        for req in &spell.requisites {
            if !self.arts.contains_key(req) {
                errors.push(format!(
                    "spell '{id}': requisite references unknown art '{req}'"
                ));
            }
        }
        // Ritual creation-legality (Ars Magica - Definitive Edition (Core Rules).md:
        // 12279-12295, :12055, :12077, :12039/:12115). Rituals are floored at
        // RITUAL_MIN_LEVEL; Formulaic/Spontaneous spells are capped at level 50;
        // Year duration and Boundary target each force a Ritual; a Momentary Creo
        // spell that creates a lasting thing must be a Ritual. Vision, though
        // Boundary-level in difficulty, does NOT (:12099).
        if let Some(level) = spell.level {
            if spell.ritual && level < RITUAL_MIN_LEVEL {
                errors.push(format!(
                    "spell '{id}': a ritual spell must be at least level {RITUAL_MIN_LEVEL} (has {level})"
                ));
            }
            if !spell.ritual && level > 50 {
                errors.push(format!(
                    "spell '{id}': a non-ritual spell may not exceed level 50 (has {level})"
                ));
            }
        }
        if !spell.ritual {
            if spell.duration == Some(SpellDuration::Year) {
                errors.push(format!(
                    "spell '{id}': Year duration requires the spell to be a ritual"
                ));
            }
            if spell.target == Some(SpellTarget::Boundary) {
                errors.push(format!(
                    "spell '{id}': Boundary target requires the spell to be a ritual"
                ));
            }
            if spell.duration == Some(SpellDuration::Momentary)
                && spell.technique == Id::new(ID_CREO)
                && spell.creates_lasting
            {
                errors.push(format!(
                    "spell '{id}': a Momentary Creo spell that creates a lasting effect must be a ritual"
                ));
            }
        }
        validate_source_range(&spell.source, &format!("spell '{id}'"), errors);
    }

    /// Validates a weapon: its combat `ability` must resolve to a known Ability and
    /// be a combat-appropriate one (a Martial Ability, or an Ability flagged
    /// `combat_ability` in data — Brawl, which the rules categorize as General but
    /// which is the combat Ability for unarmed and improvised weapons); and its
    /// source line range (if any) must be well-formed. This is the load-time trust
    /// gate that a weapon can only ship once its combat Ability exists. Source: Ars
    /// Magica - Definitive Edition (Core Rules).md:16992 (the "Ability" column names
    /// the Weapon Ability needed to use the weapon).
    fn validate_weapon_refs(&self, weapon: &Weapon, errors: &mut Vec<String>) {
        let id = &weapon.id;
        // The non-Martial combat Abilities (Brawl) are marked in data by the
        // `combat_ability` flag rather than a hardcoded slug, so a ruleset that
        // slugs unarmed combat differently just sets the flag.
        match self.abilities.get(&weapon.ability) {
            None => errors.push(format!(
                "weapon '{id}': ability references unknown ability '{}'",
                weapon.ability
            )),
            Some(ability)
                if ability.category != crate::ability::AbilityCategory::Martial
                    && !ability.combat_ability =>
            {
                errors.push(format!(
                    "weapon '{id}': ability '{}' is not a combat Ability (must be Martial or Brawl)",
                    weapon.ability
                ))
            }
            Some(_) => {}
        }
        validate_source_range(&weapon.source, &format!("weapon '{id}'"), errors);
    }

    /// Recursively validates that prerequisite refs resolve to known registries:
    /// [`Prereq::Has`] against point items, [`Prereq::AbilityMin`] against the
    /// ability catalogue, [`Prereq::ArtMin`] against the Art catalogue, and
    /// [`Prereq::House`] against the House registry. `IsMagus` carries no
    /// reference at all, so there is nothing to check for it.
    fn validate_prereq_refs(&self, prereq: &Prereq, context_id: &Id, errors: &mut Vec<String>) {
        match prereq {
            Prereq::All(children) | Prereq::Any(children) | Prereq::Nor(children) => {
                for child in children {
                    self.validate_prereq_refs(child, context_id, errors);
                }
            }
            Prereq::Has(ref_id) => {
                if !self.point_items.contains_key(ref_id) {
                    errors.push(format!(
                        "{context_id}: prerequisite references unknown ID '{ref_id}'"
                    ));
                }
            }
            Prereq::AbilityMin { ability, .. } => {
                if !self.abilities.contains_key(ability) {
                    errors.push(format!(
                        "{context_id}: prerequisite references unknown ability '{ability}'"
                    ));
                }
            }
            Prereq::ArtMin { art, .. } => {
                if !self.arts.contains_key(art) {
                    errors.push(format!(
                        "{context_id}: prerequisite references unknown art '{art}'"
                    ));
                }
            }
            Prereq::House(ref_id) => {
                if !self.houses.contains_key(ref_id) {
                    errors.push(format!(
                        "{context_id}: prerequisite references unknown house '{ref_id}'"
                    ));
                }
            }
            // IsMagus carries no reference at all, so there is nothing to check.
            Prereq::IsMagus => {}
        }
    }

    /// Fails if `ability` does not resolve to a known Ability, naming the effect
    /// `kind` in the message. Shared by the fixed-ability-target effects.
    fn validate_ability_ref(&self, ability: &Id, kind: &str, id: &Id, errors: &mut Vec<String>) {
        if !self.abilities.contains_key(ability) {
            errors.push(format!(
                "{id}: effect '{kind}' references unknown ability '{ability}'"
            ));
        }
    }

    /// Fails for every id in a fixed ability list that does not resolve
    /// (`restricted_ability_xp`, `group_affinity_cost`).
    fn validate_ability_list_effect<'a>(
        &self,
        abilities: impl IntoIterator<Item = &'a Id>,
        kind: &str,
        id: &Id,
        errors: &mut Vec<String>,
    ) {
        for ability in abilities {
            self.validate_ability_ref(ability, kind, id, errors);
        }
    }

    /// Fails for every id in a fixed point-item list that does not resolve
    /// (`grants_selection`).
    fn validate_item_list_effect<'a>(
        &self,
        items: impl IntoIterator<Item = &'a Id>,
        kind: &str,
        id: &Id,
        errors: &mut Vec<String>,
    ) {
        for granted in items {
            if !self.point_items.contains_key(granted) {
                errors.push(format!(
                    "{id}: effect '{kind}' references unknown item '{granted}'"
                ));
            }
        }
    }

    /// Fails for every Form id in a fixed Art list that does not resolve
    /// (`elemental_magic`). Only checked when the Arts catalogue is loaded (the
    /// point-items file loads even in the Arts-less `from_core_json` path; the full
    /// app + spell paths load Arts and do enforce this), exactly like
    /// `validate_spell_refs`.
    fn validate_art_list_effect<'a>(
        &self,
        forms: impl IntoIterator<Item = &'a Id>,
        kind: &str,
        id: &Id,
        errors: &mut Vec<String>,
    ) {
        if self.arts.is_empty() {
            return;
        }
        for form in forms {
            if !self.arts.contains_key(form) {
                errors.push(format!(
                    "{id}: effect '{kind}' references unknown art '{form}'"
                ));
            }
        }
    }

    /// Validates a `deficient_art` effect: its declared parameter must exist and
    /// carry a Technique- or Form-domain (either fixes the class).
    fn validate_deficient_art_effect(
        &self,
        item: &PointItem,
        param: &str,
        id: &Id,
        errors: &mut Vec<String>,
    ) {
        match item.parameters.iter().find(|p| p.key.as_str() == param) {
            None => errors.push(format!(
                "{id}: effect 'deficient_art' references unknown parameter '{param}'"
            )),
            Some(def)
                if def.domain != ParameterDomain::Technique
                    && def.domain != ParameterDomain::Form =>
            {
                errors.push(format!(
                    "{id}: effect 'deficient_art' parameter '{param}' has domain '{}', expected 'technique' or 'form'",
                    def.domain
                ))
            }
            Some(_) => {}
        }
    }

    /// Validates that every [`Effect`] names a declared parameter whose domain
    /// matches the effect kind (`ability_bonus` → an `ability`-domain param,
    /// `characteristic_limit` → a `characteristic`-domain param). Effects resolve
    /// the target through that parameter, so a missing key or domain mismatch
    /// would silently never apply — fail loudly at load instead. Effects that carry
    /// a directly-stored ref instead of a parameter are validated inline via the
    /// small `validate_*_effect` helpers.
    ///
    /// **GC3 considered and rejected:** this is one of the five functions the
    /// audit flagged as over-long (147 lines), but it is a single exhaustive
    /// `match` over [`Effect`] with no repeated logic — each arm is already a
    /// short, self-contained, well-named case, and the ~20 param-less variants
    /// that fall straight to `continue` are enumerated because the match must
    /// be exhaustive, not because the code is tangled. Splitting it would cut
    /// across one enum match into arbitrary pieces rather than along a real
    /// seam, trading one readable function for several that only make sense
    /// read together — the opposite of GC3's goal. Left whole.
    fn validate_effect_refs(&self, item: &PointItem, id: &Id, errors: &mut Vec<String>) {
        for effect in &item.effects {
            let (param, expected, kind) = match effect {
                Effect::AbilityBonus { param, .. } => {
                    (param, ParameterDomain::Ability, "ability_bonus")
                }
                Effect::CharacteristicLimit { param, .. } => (
                    param,
                    ParameterDomain::Characteristic,
                    "characteristic_limit",
                ),
                Effect::ArtBonus { param, .. } => (param, ParameterDomain::Art, "art_bonus"),
                Effect::AffinityAbilityCost { param, .. } => {
                    (param, ParameterDomain::Ability, "affinity_ability_cost")
                }
                Effect::AffinityArtCost { param, .. } => {
                    (param, ParameterDomain::Art, "affinity_art_cost")
                }
                // Magical Focus / Academic Concentration name a free-text
                // descriptor the player types (a sub-Art focus, a study field).
                Effect::MagicalFocus { param, .. } => {
                    (param, ParameterDomain::Text, "magical_focus")
                }
                Effect::AbilityRollMod { param, .. } => {
                    (param, ParameterDomain::Text, "ability_roll_mod")
                }
                // Deficient Art targets a Technique OR a Form; the declared
                // param's domain (technique/form) is what fixes the class.
                Effect::DeficientArt { param } => {
                    self.validate_deficient_art_effect(item, param, id, errors);
                    continue;
                }
                // Fixed target: validate the directly-stored ability id resolves.
                Effect::AbilityScoreGrant { ability, .. } => {
                    self.validate_ability_ref(ability, "ability_score_grant", id, errors);
                    continue;
                }
                // Fixed eligibility list: validate each named ability id resolves,
                // like AbilityScoreGrant.ability and AbilityMin. The eligible
                // categories are a loose namespace matched at eval, not a registry,
                // so they are not checked here.
                Effect::RestrictedAbilityXp { abilities, .. } => {
                    self.validate_ability_list_effect(
                        abilities,
                        "restricted_ability_xp",
                        id,
                        errors,
                    );
                    continue;
                }
                // Fixed group of abilities the Affinity covers (Linguist).
                Effect::GroupAffinityCost { abilities, .. } => {
                    self.validate_ability_list_effect(abilities, "group_affinity_cost", id, errors);
                    continue;
                }
                // Fixed nested grant: every granted id must resolve to a point item
                // (a Virtue/Flaw), like a House grant's `item`.
                Effect::GrantsSelection { items } => {
                    self.validate_item_list_effect(items, "grants_selection", id, errors);
                    continue;
                }
                // Fixed target set: every elemental Form id the redistribution pools
                // over must resolve to a known Art.
                Effect::ElementalMagic { forms } => {
                    self.validate_art_list_effect(forms, "elemental_magic", id, errors);
                    continue;
                }
                // Fixed target: validate the directly-stored characteristic id
                // resolves to one of the eight Characteristics.
                Effect::CharacteristicScoreDelta { characteristic, .. } => {
                    if crate::characteristics::Characteristic::from_id(characteristic).is_none() {
                        errors.push(format!(
                            "{id}: effect 'characteristic_score_delta' references unknown characteristic '{characteristic}'"
                        ));
                    }
                    continue;
                }
                // The Form-scoped `deft_form` quirk names a Form via its param,
                // resolved against the selection's params exactly as
                // `deficient_art` resolves its Art (see the SpecialCastingMod doc
                // in types.rs); the declared param must exist and carry the Form
                // domain. A missing key or non-Form domain would make the waiver
                // silently never apply in derived.rs::in_play_mods.
                Effect::SpecialCastingMod {
                    kind: SpecialCasting::DeftForm,
                    param: Some(p),
                } => (p, ParameterDomain::Form, "special_casting_mod"),
                // deft_form REQUIRES a param naming the affected Form:
                // derived.rs::in_play_mods guards on `param.as_ref()`, so a
                // param-less deft_form would silently never apply its waiver.
                // Fail loudly rather than fall into the param-less catch-all.
                Effect::SpecialCastingMod {
                    kind: SpecialCasting::DeftForm,
                    param: None,
                } => {
                    errors.push(format!(
                        "{id}: effect 'special_casting_mod' kind 'deft_form' requires a param naming the affected Form"
                    ));
                    continue;
                }
                // No parameter or ref to resolve: the grant is intrinsic. The
                // param-less / non-`deft_form` SpecialCasting quirks fall here.
                Effect::SpellMasteryXp { .. }
                | Effect::GrantsSpellMastery { .. }
                | Effect::ItemLevelBudget { .. }
                | Effect::MasterpieceItem
                | Effect::TrueFaithGrant { .. }
                | Effect::WarpingGrant { .. }
                | Effect::SizeDelta { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::LaterLifeXpRate { .. }
                | Effect::AbilityAuthorization { .. }
                | Effect::LocalityAbilityCapFraction { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::GrantsReputation { .. }
                | Effect::MightGrant { .. }
                | Effect::PowerLevels { .. }
                // M5/5b in-play effects with no parameter or ref to resolve:
                // consumed intrinsically by derived.rs (5i).
                | Effect::CastingTotalMod { .. }
                | Effect::LabTotalMod { .. }
                | Effect::MagicTotalHalving { .. }
                | Effect::SoakMod { .. }
                | Effect::CombatMod { .. }
                | Effect::HealthMod { .. }
                | Effect::MagicResistanceMod { .. }
                | Effect::AgingMod { .. }
                | Effect::AdvancementMod { .. }
                | Effect::SpecialCastingMod { .. } => {
                    continue;
                }
            };
            match item.parameters.iter().find(|p| &p.key == param) {
                None => errors.push(format!(
                    "{id}: effect '{kind}' references unknown parameter '{param}'"
                )),
                Some(def) if def.domain != expected => errors.push(format!(
                    "{id}: effect '{kind}' parameter '{param}' has domain '{}', expected '{expected}'",
                    def.domain
                )),
                Some(_) => {}
            }
        }
    }

    fn validate_incompatibility_symmetry(&self, errors: &mut Vec<String>) {
        for (id, item) in &self.point_items {
            for incompat_id in &item.incompatible_with {
                if let Some(other) = self.point_items.get(incompat_id)
                    && !other.incompatible_with.contains(id)
                {
                    errors.push(format!(
                        "asymmetric incompatibility: '{id}' lists '{incompat_id}' but not vice versa"
                    ));
                }
            }
        }
    }

    /// Enforces that the Major and Minor variants of the SAME Virtue/Flaw mutually
    /// exclude — a character may only take one magnitude of a given item. Variant
    /// pairs are detected by a shared stem under two naming conventions: the suffix
    /// form `<stem>_major` / `<stem>_minor` and the prefix form
    /// `major_<stem>` / `minor_<stem>` (the latter covers Major / Minor Magical
    /// Focus). Only the Major side is inspected, so each pair is reported once, and
    /// a pair is considered only when BOTH members exist — a lone `*_major` (or a
    /// `*_minor` whose `*_major` counterpart is a different, absent concept, e.g.
    /// `virtue.minor_enchantments`) is never flagged. For every detected pair, both
    /// members must list each other in `incompatible_with`; otherwise this fails
    /// loudly naming both offending ids.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4405 ("A character
    /// can have only one Magical Focus, either major or minor").
    fn validate_magnitude_variant_exclusivity(&self, errors: &mut Vec<String>) {
        for (major_id, major_item) in &self.point_items {
            let Some(minor_id) = minor_variant_sibling(major_id) else {
                continue;
            };
            let Some(minor_item) = self.point_items.get(&minor_id) else {
                continue;
            };
            if !major_item.incompatible_with.contains(&minor_id)
                || !minor_item.incompatible_with.contains(major_id)
            {
                errors.push(format!(
                    "magnitude variants '{major_id}' and '{minor_id}' must be mutually incompatible_with each other"
                ));
            }
        }
    }
}

/// Checks one record's optional source-line-range for validity, pushing a
/// `"<subject>: source line range start (S) exceeds end (E)"` error if the
/// range is malformed. `subject` is the exact prefix each of the nine former
/// call sites used to format inline (GC2's fix): most pass `"<label> '<id>'"`
/// (e.g. `"spell 'spell.foo'"`), while the point-item site passes the bare id
/// with no label — so every site's message text is byte-identical to before
/// this extraction; only the duplicated check is shared.
fn validate_source_range(source: &Option<SourceRef>, subject: &str, errors: &mut Vec<String>) {
    if let Some(source) = source
        && !source.lines.is_valid()
    {
        errors.push(format!(
            "{subject}: source line range start ({}) exceeds end ({})",
            source.lines.start, source.lines.end
        ));
    }
}
