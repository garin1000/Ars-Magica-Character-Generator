//! The core character-sheet section writers: title, identity, characteristics,
//! Virtues/Flaws, Abilities, Arts, spells, equipment, combat, Soak,
//! Encumbrance, and the Fatigue/Wound tracks. Split out of `export.rs`; see
//! `export/resolve.rs` for the label/name resolution primitives these lean on
//! and `export/magic.rs` for the remaining (traits/magic/annotations) writers.

use super::*;

impl<'a> Doc<'a> {
    /// The `# ` title and the subtitle line (character type, House, ages).
    pub(super) fn write_title(&self, out: &mut String) {
        let title = if self.entity.name.trim().is_empty() {
            self.label("export-untitled")
        } else {
            escape_cell(&self.entity.name)
        };
        out.push_str("# ");
        out.push_str(&title);
        out.push_str("\n\n");

        let mut parts: Vec<String> = Vec::new();
        // The type label is composed from the profile id — catalogue data, so it is
        // the one key family LABEL_KEYS does not enumerate (see its docs).
        parts.push(self.label(&format!("type-{}", self.entity.type_id)));
        if let Some(house) = &self.entity.house {
            parts.push(pair(&self.label("house-label"), &self.name(house)));
        }
        if let Some(age) = self.entity.age {
            parts.push(pair(&self.label("age-label"), &age.to_string()));
        }
        if let Some(age) = self.entity.apparent_age {
            parts.push(pair(&self.label("apparent-age-label"), &age.to_string()));
        }
        out.push('*');
        out.push_str(&parts.join(SUBTITLE_SEPARATOR));
        out.push_str("*\n\n");
    }

    /// The free-text identity fields, each omitted when empty.
    pub(super) fn write_identity(&self, out: &mut String) {
        let e = self.entity;
        let mut body = String::new();
        for (key, value) in [
            ("identity-description", e.description.as_str()),
            ("identity-concept", e.concept.as_str()),
            ("identity-gender", e.gender.as_str()),
            ("identity-sigil", e.sigil.as_str()),
            ("identity-covenant", e.covenant_name.as_str()),
            ("identity-parens", e.parens.as_str()),
        ] {
            if !value.trim().is_empty() {
                self.labelled(&mut body, key, &escape_cell(value));
            }
        }
        if let Some(year) = e.birth_year {
            self.labelled(&mut body, "identity-birth-year", &year.to_string());
        }
        if body.is_empty() {
            return;
        }
        self.section(out, 2, "identity-label");
        out.push_str(&body);
        out.push('\n');
    }

    /// The bought Characteristic scores, their effective values where aging or a
    /// Virtue moves them, and the sheet's free-text descriptions.
    ///
    /// The Effective cell is [`effective_characteristic_after_aging`] — the same
    /// single source of truth every derived total reads, so an aged character's sheet
    /// cannot show a score its own Soak and Combat lines contradict. It is printed
    /// only when it differs from the bought score.
    ///
    /// Every Characteristic in the taxonomy gets a row: a score of 0 is stored as an
    /// *absent* key, so skipping the unstored ones would silently drop a
    /// Characteristic the user did set — to 0 — from the sheet. Whether the section
    /// exists at all is still pure emptiness (no score and no description anywhere),
    /// which is what keeps a covenant's sheet free of it without an `EntityKind` gate.
    pub(super) fn write_characteristics(&self, out: &mut String) {
        let e = self.entity;
        let described = e
            .characteristic_descriptions
            .values()
            .any(|text| !text.trim().is_empty());
        if e.characteristics.is_empty() && !described {
            return;
        }
        let mut rows: Vec<Vec<String>> = Vec::new();
        for c in Characteristic::ALL {
            let description = e
                .characteristic_descriptions
                .get(&c)
                .map(String::as_str)
                .unwrap_or_default();
            let bought = i32::from(e.characteristics.get(&c).copied().unwrap_or(0));
            let effective = effective_characteristic_after_aging(e, self.rules(), c);
            rows.push(vec![
                self.label(&format!("characteristic-{c}")),
                signed(bought),
                if effective == bought {
                    String::new()
                } else {
                    signed(effective)
                },
                escape_cell(description),
            ]);
        }
        self.section(out, 2, "characteristics-title");
        table(
            out,
            &[
                self.label("identity-name"),
                self.label("ability-score-label"),
                self.label("export-col-effective"),
                self.label("characteristic-description-label"),
            ],
            &rows,
        );
    }

    /// The Virtues/Flaws (Boons/Hooks for a covenant) the entity holds, grouped by
    /// the catalogue item's kind, plus the point-balance read-out.
    ///
    /// Each kind section holds two tables: the **point-bought** rows from
    /// `entity.selections`, then — under a marked sub-heading, and only when there
    /// are any — the **granted** rows from [`entity_grants`] (a magus's free House
    /// Virtue, a mythic-companion type's grants, an owed Warping pick, a Virtue that
    /// grants another). Both belong on the sheet, but only the bought rows are
    /// budgeted, so the balance line stays on [`compute_balance`], which counts
    /// `selections` alone — the two tables are what keeps the granted items visible
    /// without letting them move a number.
    ///
    /// A selection whose id is absent from the catalogue has no kind to file it under
    /// and is not printed; [`crate::validation::validate`] reports it as
    /// `unknown_ref`.
    pub(super) fn write_virtues_flaws(&self, out: &mut String) {
        let granted = entity_grants(self.entity, self.rules());
        let mut body = String::new();
        for (kind, heading_key) in ITEM_KIND_HEADINGS {
            let bought = self.item_rows(&self.entity.selections, kind);
            let granted_rows = self.item_rows(&granted, kind);
            if bought.is_empty() && granted_rows.is_empty() {
                continue;
            }
            self.section(&mut body, 3, heading_key);
            let headers = [
                self.label("identity-name"),
                self.label("export-col-type"),
                self.label("export-col-magnitude"),
            ];
            table(&mut body, &headers, &bought);
            if granted_rows.is_empty() {
                continue;
            }
            self.section(&mut body, 4, "export-granted");
            table(&mut body, &headers, &granted_rows);
        }
        if body.is_empty() {
            return;
        }
        self.section(out, 2, "tab-virtues-flaws");
        out.push_str(&body);
        let balance = compute_balance(self.entity, self.rules());
        let ceilings = effective_point_ceilings(self.entity, self.rules());
        for (key, used, ceiling) in [
            (
                "items-virtues-title",
                balance.virtue_points,
                ceilings.as_ref().map(|c| c.virtue_ceiling),
            ),
            (
                "items-flaws-title",
                balance.flaw_points,
                ceilings.as_ref().map(|c| c.flaw_ceiling),
            ),
        ] {
            let value = match ceiling {
                Some(max) => format!("{used} / {max}"),
                None => used.to_string(),
            };
            self.labelled(out, key, &value);
        }
        out.push('\n');
    }

    /// The name, type and magnitude rows for those `selections` whose catalogue item
    /// is of `kind`, in the list's own order. Shared by the bought and the granted
    /// table so a granted item is rendered exactly like a bought one — only its table
    /// differs.
    ///
    /// The "type" cell is the item's `category`, localized through `category-<id>` —
    /// the same key the in-app badge uses. Categories are catalogue *data*, so that
    /// family is not enumerated in [`LABEL_KEYS`] (see its docs).
    fn item_rows(&self, selections: &[Selection], kind: ItemKind) -> Vec<Vec<String>> {
        selections
            .iter()
            .filter_map(|selection| {
                let item = self.rules().item(&selection.item_ref)?;
                if item.kind != kind {
                    return None;
                }
                let values = self.param_display_values(&selection.params);
                Some(vec![
                    self.parameterized_name(&selection.item_ref, &values),
                    self.label(&format!("category-{}", item.category)),
                    self.label(&format!("magnitude-{}", item.magnitude)),
                ])
            })
            .collect()
    }

    /// The bought Abilities — each instance a row of its own — then the Abilities a
    /// Virtue granted without any purchase, and the experience pools that funded the
    /// bought ones (shared with the Arts, as the rules give one pool).
    pub(super) fn write_abilities(&self, out: &mut String) {
        let e = self.entity;
        let mut rows: Vec<Vec<String>> = Vec::new();
        for bought in &e.ability_scores {
            let mut values: BTreeMap<String, String> = BTreeMap::new();
            if let Some(parameter) = &bought.parameter {
                let key = self
                    .rules()
                    .ability(&bought.ability)
                    .and_then(|a| a.parameter.clone())
                    .unwrap_or_else(|| UNKNOWN_PARAM_KEY.to_string());
                values.insert(key, self.param_value(parameter));
            }
            let score = i32::from(bought.score);
            let effective = effective_ability_score(
                e,
                self.rules(),
                &bought.ability,
                bought.parameter.as_deref(),
            );
            rows.push(vec![
                self.parameterized_name(&bought.ability, &values),
                escape_cell(bought.specialty.as_deref().unwrap_or_default()),
                score.to_string(),
                if effective == score {
                    String::new()
                } else {
                    effective.to_string()
                },
            ]);
        }
        rows.extend(self.granted_ability_rows());
        let xp = xp_allocation(e, self.rules());
        let has_xp = xp.general_pool > 0 || xp.general_used > 0 || !xp.restricted.is_empty();
        if rows.is_empty() && !has_xp {
            return;
        }
        self.section(out, 2, "abilities-title");
        let headers = [
            self.label("identity-name"),
            self.label("ability-specialty-label"),
            self.label("ability-score-label"),
            self.label("export-col-effective"),
        ];
        table(out, &headers, &rows);
        if xp.general_pool > 0 || xp.general_used > 0 {
            let spent = format!("{} / {}", xp.general_used, xp.general_pool);
            self.labelled(out, "xp-pool", &spent);
            out.push('\n');
        }
        if xp.restricted.is_empty() {
            return;
        }
        self.section(out, 3, "export-xp-restricted");
        for pool in &xp.restricted {
            let drawn = format!("{} / {}", pool.used, pool.amount);
            field(out, &self.restricted_pool_label(pool), &drawn);
        }
        out.push('\n');
    }

    /// What to call one restricted pool — the same rule the in-app XP bar follows
    /// (`restrictedPoolLabel` in `ui/src/lib/derive.ts`), so a budget reads the same
    /// on screen and on the sheet.
    ///
    /// A **life-stage block** is named for where the experience came from, never for
    /// what it may buy: both childhood blocks share the childhood Ability list, so
    /// eligibility cannot even tell them apart, and the native-language block is
    /// restricted to one *instance* — it lists no Ability and no category at all, so
    /// an eligibility label for it is the empty string. Its `xp-pool-<block>` key is
    /// declared in [`LABEL_KEYS`], the block enum being a fixed taxonomy.
    ///
    /// A **Virtue's grant** keeps its eligibility list: the item's own name says
    /// nothing about what its points buy, which is exactly where Educated and
    /// Warrior differ. An eligible Ability may be parameterized (Dead Language is
    /// "{language} (Dead Language)"), and the pool names the Ability rather than one
    /// instance of it, so the placeholder keeps its slot label instead of reaching
    /// the reader raw.
    fn restricted_pool_label(&self, pool: &RestrictedXpPool) -> String {
        if let XpPoolOrigin::LifeStage { block } = &pool.origin {
            return self.label(&format!("xp-pool-{block}"));
        }
        let mut eligibility: Vec<String> = pool
            .abilities
            .iter()
            .map(|id| self.parameterized_name(id, &BTreeMap::new()))
            .collect();
        eligibility.extend(
            pool.categories
                .iter()
                .map(|c| self.label(&format!("ability-category-{c}"))),
        );
        eligibility.join(&self.list_separator())
    }

    /// A row for every Ability a Virtue seeded with a free starting score
    /// ([`ability_score_floors`], e.g. Second Sight 1) that the character has *not*
    /// also bought. Without these the sheet omits an Ability the character can use,
    /// since a granted score is stored nowhere in `ability_scores`.
    ///
    /// A grant fixes its target by id and reaches only the parameter-less instance
    /// (see [`crate::effective::granted_ability_floor`]), so a bought row cancels the
    /// floor row exactly when it names the same Ability with no parameter — the same
    /// instance match [`effective_ability_score`] makes. The bought column reads 0,
    /// the convention every unbought score uses, and the effective column comes from
    /// [`effective_ability_score`], so a Puissant bonus on a granted Ability shows.
    /// Order is the floors' own (ability id), after the bought rows.
    fn granted_ability_rows(&self) -> Vec<Vec<String>> {
        let e = self.entity;
        ability_score_floors(e, self.rules())
            .into_iter()
            .filter(|floor| {
                !e.ability_scores
                    .iter()
                    .any(|bought| bought.ability == floor.ability && bought.parameter.is_none())
            })
            .map(|floor| {
                let effective = effective_ability_score(e, self.rules(), &floor.ability, None);
                vec![
                    self.parameterized_name(&floor.ability, &BTreeMap::new()),
                    String::new(),
                    "0".to_string(),
                    if effective == 0 {
                        String::new()
                    } else {
                        effective.to_string()
                    },
                ]
            })
            .collect()
    }

    /// The Hermetic Arts, split into Techniques and Forms.
    ///
    /// The **catalogue** is the row set, in id order within each class: an Art scored
    /// 0 is stored as no score at all, so listing only the stored Arts would drop
    /// every unscored Art — and, for a magus who scored no Form, the entire Forms
    /// table — from a sheet whose reader needs the whole grid. Which Arts exist is
    /// data, so the section's size follows `arts.json` with no code change.
    ///
    /// The section itself is still gated on emptiness: it appears once the entity
    /// scores an Art the catalogue holds. An Art id absent from the catalogue has no
    /// Technique/Form class to file it under and can be scored by no engine read-out
    /// either ([`crate::validation::validate`] reports it as `unknown_art`), so it
    /// neither prints nor opens the section.
    pub(super) fn write_arts(&self, out: &mut String) {
        let e = self.entity;
        let scores_a_known_art = e
            .art_scores
            .iter()
            .any(|score| self.rules().art(&score.art).is_some());
        if !scores_a_known_art {
            return;
        }
        self.section(out, 2, "tab-arts");
        for art_type in ArtType::ALL {
            let rows: Vec<Vec<String>> = self
                .rules()
                .art_ids_of(art_type)
                .iter()
                .map(|art| {
                    let score = e
                        .art_scores
                        .iter()
                        .find(|stored| &stored.art == art)
                        .map(|stored| i32::from(stored.score))
                        .unwrap_or(0);
                    let effective = effective_art_score(e, self.rules(), art);
                    vec![
                        escape_cell(&self.name(art)),
                        score.to_string(),
                        if effective == score {
                            String::new()
                        } else {
                            effective.to_string()
                        },
                    ]
                })
                .collect();
            if rows.is_empty() {
                continue;
            }
            self.section(out, 3, &format!("art-type-{art_type}"));
            let headers = [
                self.label("identity-name"),
                self.label("ability-score-label"),
                self.label("export-col-effective"),
            ];
            table(out, &headers, &rows);
        }
    }

    /// The spells the character knows, each with its short Art-and-level code and its
    /// Spell Mastery.
    pub(super) fn write_spells(&self, out: &mut String) {
        let e = self.entity;
        let mut rows: Vec<Vec<String>> = Vec::new();
        for chosen in &e.spells {
            let catalogue = self.rules().spell(&chosen.spell);
            let mut values: BTreeMap<String, String> = BTreeMap::new();
            if let Some(parameter) = &chosen.parameter {
                let key = catalogue
                    .and_then(|s| s.parameters.first())
                    .map(|p| p.key.clone())
                    .unwrap_or_else(|| UNKNOWN_PARAM_KEY.to_string());
                values.insert(key, self.param_value(parameter));
            }
            let mastery = effective_spell_mastery(chosen, e, self.rules());
            let abilities: Vec<String> = chosen
                .mastery_abilities
                .iter()
                .map(|id| escape_cell(&self.name(id)))
                .collect();
            rows.push(vec![
                self.parameterized_name(&chosen.spell, &values),
                self.spell_code(chosen),
                if mastery == 0 {
                    String::new()
                } else {
                    mastery.to_string()
                },
                abilities.join(&self.list_separator()),
            ]);
        }
        if rows.is_empty() {
            return;
        }
        self.section(out, 2, "tab-spells");
        let headers = [
            self.label("identity-name"),
            self.label("export-col-spell-code"),
            self.label("spell-mastery-label"),
            self.label("spell-mastery-abilities-label"),
        ];
        table(out, &headers, &rows);
    }

    /// A spell's Arts and level as the one short cell the rulebook and the app both
    /// use: the Technique and Form abbreviations followed by the resolved level, with
    /// no separator (`CrIg20`).
    ///
    /// An unresolved General level keeps its localized marker a space apart, since
    /// `CrIgGeneral` would not read as one figure. A spell no catalogue holds has no
    /// Arts to abbreviate, and a bare level would read as a code, so its cell is empty
    /// — the row still names the spell the character claims.
    fn spell_code(&self, chosen: &SpellSelection) -> String {
        let Some(spell) = self.rules().spell(&chosen.spell) else {
            return String::new();
        };
        let level = match resolved_spell_level(chosen, self.rules()) {
            Some(level) => level.to_string(),
            None => format!(" {}", self.label("spell-level-general")),
        };
        format!(
            "{}{}{level}",
            self.art_code(&spell.technique),
            self.art_code(&spell.form)
        )
    }

    /// Which catalogue holds `id`, or `None` when no catalogue does.
    fn carried_class(&self, id: &Id) -> Option<Carried> {
        if self.rules().weapon(id).is_some() {
            Some(Carried::Weapon)
        } else if self.rules().shield(id).is_some() {
            Some(Carried::Shield)
        } else if self.rules().armor_item(id).is_some() {
            Some(Carried::Armor)
        } else {
            None
        }
    }

    /// The carried equipment, grouped weapons / shields / armor with the equipped
    /// marker. An id no catalogue holds is not printed;
    /// [`crate::validation::validate`] reports it as `unknown_equipment`.
    pub(super) fn write_equipment(&self, out: &mut String) {
        let mut body = String::new();
        for (class, heading_key) in EQUIPMENT_GROUPS {
            let rows: Vec<Vec<String>> = self
                .entity
                .equipment
                .iter()
                .filter(|slot| self.carried_class(&slot.item) == Some(class))
                .map(|slot| {
                    vec![
                        escape_cell(&self.name(&slot.item)),
                        self.label(if slot.equipped {
                            "export-yes"
                        } else {
                            "export-no"
                        }),
                    ]
                })
                .collect();
            if rows.is_empty() {
                continue;
            }
            self.section(&mut body, 3, heading_key);
            let headers = [
                self.label("identity-name"),
                self.label("equipment-equipped-label"),
            ];
            table(&mut body, &headers, &rows);
        }
        if body.is_empty() {
            return;
        }
        self.section(out, 2, "tab-equipment");
        out.push_str(&body);
    }

    /// One or two combat lines per equipped weapon — a line carrying shield modifiers
    /// names the shields alongside the weapon. Attack, Damage and Range are blank for
    /// a weapon that has none (Dodge is attack- and damage-less; melee has no Range).
    pub(super) fn write_combat(&self, out: &mut String) {
        let lines = combat_totals(self.entity, self.rules());
        if lines.is_empty() {
            return;
        }
        let rows: Vec<Vec<String>> = lines
            .iter()
            .map(|line| {
                vec![
                    escape_cell(&self.combat_line_name(line)),
                    escape_cell(&self.name(&line.ability)),
                    line.initiative.to_string(),
                    optional_number(line.attack),
                    line.defense.to_string(),
                    optional_number(line.damage),
                    line.range.map(|r| r.to_string()).unwrap_or_default(),
                ]
            })
            .collect();
        self.section(out, 2, "derived-section-combat");
        let headers = [
            self.label("identity-name"),
            self.label("param-label-ability"),
            self.label("derived-combat-init"),
            self.label("derived-combat-attack"),
            self.label("derived-combat-defense"),
            self.label("derived-combat-damage"),
            self.label("derived-range"),
        ];
        table(out, &headers, &rows);
    }

    /// A combat line's name: the weapon alone on a bare line, or the weapon joined to
    /// every shield whose modifiers it folded in ("Long Sword & Round Shield"). The
    /// joiner is a localized label; the spaces around it are composed here, since a
    /// Fluent value cannot begin or end with one.
    fn combat_line_name(&self, line: &CombatLine) -> String {
        if line.shields.is_empty() {
            return self.name(&line.weapon);
        }
        let joiner = format!(" {} ", self.label("derived-combat-shield-joiner"));
        let mut parts = vec![self.name(&line.weapon)];
        parts.extend(line.shields.iter().map(|shield| self.name(shield)));
        parts.join(&joiner)
    }

    /// The Soak breakdown: every labelled addend, then the total.
    pub(super) fn write_soak(&self, out: &mut String) {
        let soak_total = soak(self.entity, self.rules());
        if soak_total.total == 0 && soak_total.addends.iter().all(|a| a.value == 0) {
            return;
        }
        self.section(out, 2, "derived-section-soak");
        for addend in &soak_total.addends {
            let key = format!("derived-addend-{}", addend.label);
            self.labelled(out, &key, &signed(addend.value));
        }
        self.labelled(out, "export-col-total", &signed(soak_total.total));
        out.push('\n');
    }

    /// Carried Load, the Burden it produces, and the Encumbrance penalty.
    pub(super) fn write_encumbrance(&self, out: &mut String) {
        let enc = encumbrance(self.entity, self.rules());
        if enc.load == 0 && enc.burden == 0 && enc.total == 0 {
            return;
        }
        self.section(out, 2, "derived-section-encumbrance");
        self.labelled(out, "derived-load", &enc.load.to_string());
        self.labelled(out, "derived-burden", &enc.burden.to_string());
        self.labelled(out, "export-col-total", &enc.total.to_string());
        out.push('\n');
    }

    /// The Fatigue and Wound tracks.
    ///
    /// The one pair of sections that emptiness cannot govern: both are constants of a
    /// creature's body (five Fatigue levels, five wound bands widened by Size), never
    /// a collection that can be empty. They are therefore gated on the entity being a
    /// character — a covenant has no body to fatigue or wound.
    pub(super) fn write_health_tracks(&self, out: &mut String) {
        if self.entity.entity_kind != EntityKind::Character {
            return;
        }
        let fatigue: Vec<Vec<String>> = fatigue_levels(self.entity, self.rules())
            .iter()
            .map(|level| {
                vec![
                    self.label(&format!("derived-fatigue-{}", level.level)),
                    level.penalty.to_string(),
                ]
            })
            .collect();
        self.section(out, 2, "derived-section-fatigue");
        let fatigue_headers = [
            self.label("identity-name"),
            self.label("export-col-penalty"),
        ];
        table(out, &fatigue_headers, &fatigue);

        let wounds: Vec<Vec<String>> = wound_ranges(self.entity, self.rules())
            .iter()
            .map(|band| {
                let span = match band.max {
                    Some(max) => format!("{}{RANGE_DASH}{max}", band.min),
                    None => format!("{}+", band.min),
                };
                vec![
                    self.label(&format!("derived-wound-{}", band.level)),
                    span,
                    band.penalty.map(|p| p.to_string()).unwrap_or_default(),
                ]
            })
            .collect();
        self.section(out, 2, "derived-section-wounds");
        let wound_headers = [
            self.label("identity-name"),
            self.label("derived-range"),
            self.label("export-col-penalty"),
        ];
        table(out, &wound_headers, &wounds);
    }
}
