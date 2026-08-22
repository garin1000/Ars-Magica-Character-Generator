//! The remaining section writers: Personality Traits, Reputations,
//! Confidence, supernatural Might/powers, magic possessions (devices,
//! Longevity Ritual, talisman, familiar), and the annotation block (Warping,
//! Twilight Scars, Decrepitude, Living Conditions, the aging log). Split out
//! of `export.rs`; see `export/resolve.rs` for the label/name resolution
//! primitives these lean on and `export/sections.rs` for the core
//! character-sheet writers.

use super::*;

impl<'a> Doc<'a> {
    /// The named Personality Traits and their signed values.
    pub(super) fn write_personality_traits(&self, out: &mut String) {
        if self.entity.personality_traits.is_empty() {
            return;
        }
        self.section(out, 2, "personality-label");
        write_traits(out, &self.entity.personality_traits);
    }

    /// The starting Reputations: audience, level, and what the Reputation is for.
    pub(super) fn write_reputations(&self, out: &mut String) {
        if self.entity.reputations.is_empty() {
            return;
        }
        self.section(out, 2, "reputations-label");
        for reputation in &self.entity.reputations {
            let audience = self.label(&format!("reputation-type-{}", reputation.kind));
            let heading = format!("{audience} {}", reputation.score);
            field(out, &heading, &escape_cell(&reputation.content));
        }
        out.push('\n');
    }

    /// The derived Confidence Score and Points (a grog has neither, so the section
    /// disappears for one).
    pub(super) fn write_confidence(&self, out: &mut String) {
        let Some(profile) = self.rules().profile(&self.entity.type_id) else {
            return;
        };
        let confidence = confidence(
            profile.confidence_score,
            profile.confidence_points,
            self.entity,
            self.rules(),
        );
        if confidence.score == 0 && confidence.points == 0 {
            return;
        }
        self.section(out, 2, "confidence-label");
        self.labelled(out, "ability-score-label", &confidence.score.to_string());
        self.labelled(out, "export-col-points", &confidence.points.to_string());
        out.push('\n');
    }

    /// A supernatural being's effective Might and the powers it holds.
    pub(super) fn write_supernatural(&self, out: &mut String) {
        let might = effective_might(self.entity, self.rules());
        let powers = self.leveled_rows(&self.entity.powers, "power-level-label");
        if might.is_none() && powers.is_empty() {
            return;
        }
        self.section(out, 2, "tab-supernatural");
        if let Some(might) = might {
            self.labelled(out, "supernatural-might-label", &self.realm_score(&might));
            out.push('\n');
        }
        if powers.is_empty() {
            return;
        }
        self.section(out, 3, "supernatural-powers-label");
        table(out, &powers.headers, &powers.rows);
    }

    /// The magus's magical possessions: the assumed aura, enchanted devices, the
    /// Longevity Ritual as stored, the talisman, and the familiar's statblock.
    pub(super) fn write_magic_items(&self, out: &mut String) {
        let e = self.entity;
        let devices = self.leveled_rows(&e.devices, "device-level-label");
        let mut body = String::new();
        if !devices.is_empty() {
            self.section(&mut body, 3, "possessions-devices-label");
            table(&mut body, &devices.headers, &devices.rows);
        }
        self.write_longevity(&mut body);
        self.write_talisman(&mut body);
        self.write_familiar(&mut body);
        if e.aura == 0 && body.is_empty() {
            return;
        }
        self.section(out, 2, "tab-possessions");
        if e.aura != 0 {
            self.labelled(out, "aura-label", &signed(e.aura));
            out.push('\n');
        }
        out.push_str(&body);
    }

    /// The stored Longevity Ritual: where it came from, the entered aging bonus (or a
    /// marker when it was never entered), and its culminating focus.
    fn write_longevity(&self, out: &mut String) {
        let Some(ritual) = &self.entity.longevity_ritual else {
            return;
        };
        self.section(out, 3, "longevity-label");
        let source = self.label(&format!("longevity-source-{}", ritual.source));
        self.labelled(out, "longevity-source-label", &source);
        let bonus = match ritual.bonus {
            Some(bonus) => signed(i32::from(bonus)),
            None => self.label("longevity-not-entered"),
        };
        self.labelled(out, "longevity-bonus-label", &bonus);
        if !ritual.focus.trim().is_empty() {
            self.labelled(out, "longevity-focus-label", &escape_cell(&ritual.focus));
        }
        out.push('\n');
    }

    /// The talisman: its shape-and-material identity, its attunements, and the
    /// effects instilled in it.
    fn write_talisman(&self, out: &mut String) {
        let Some(talisman) = &self.entity.talisman else {
            return;
        };
        let attunements: Vec<Vec<String>> = talisman
            .attunements
            .iter()
            .map(|a| vec![escape_cell(&a.description), signed(i32::from(a.bonus))])
            .collect();
        let effects = self.leveled_rows(&talisman.effects, "talisman-effect-level-label");
        if talisman.description.trim().is_empty() && attunements.is_empty() && effects.is_empty() {
            return;
        }
        self.section(out, 3, "talisman-label");
        if !talisman.description.trim().is_empty() {
            let identity = escape_cell(&talisman.description);
            self.labelled(out, "talisman-description-label", &identity);
            out.push('\n');
        }
        if !attunements.is_empty() {
            self.section(out, 4, "talisman-attunements-label");
            let headers = [
                self.label("identity-name"),
                self.label("talisman-bonus-label"),
            ];
            table(out, &headers, &attunements);
        }
        if !effects.is_empty() {
            self.section(out, 4, "talisman-effects-label");
            table(out, &effects.headers, &effects.rows);
        }
    }

    /// The familiar's own statblock, in the rulebook's Creature Format order: the
    /// beast, its Magic Might, Characteristics, Size, Personality Traits, the three
    /// bond cords, and the powers invested in the bond. Everything here is the
    /// familiar's own, never the magus's.
    fn write_familiar(&self, out: &mut String) {
        let Some(familiar) = &self.entity.familiar else {
            return;
        };
        self.section(out, 3, "familiar-label");
        if !familiar.name.trim().is_empty() {
            self.labelled(out, "identity-name", &escape_cell(&familiar.name));
        }
        if !familiar.animal.trim().is_empty() {
            self.labelled(out, "familiar-animal-label", &escape_cell(&familiar.animal));
        }
        if let Some(might) = familiar.might {
            self.labelled(out, "familiar-might-label", &self.realm_score(&might));
        }
        if familiar.size != 0 {
            let size = signed(i32::from(familiar.size));
            self.labelled(out, "familiar-size-label", &size);
        }
        for (key, score) in [
            ("familiar-cord-gold", familiar.cord_gold),
            ("familiar-cord-silver", familiar.cord_silver),
            ("familiar-cord-bronze", familiar.cord_bronze),
        ] {
            if score != 0 {
                self.labelled(out, key, &signed(i32::from(score)));
            }
        }
        out.push('\n');
        let characteristics: Vec<Vec<String>> = Characteristic::ALL
            .into_iter()
            .filter_map(|c| {
                let score = familiar.characteristics.get(&c)?;
                Some(vec![
                    self.label(&format!("characteristic-{c}")),
                    signed(i32::from(*score)),
                ])
            })
            .collect();
        if !characteristics.is_empty() {
            self.section(out, 4, "characteristics-title");
            let headers = [
                self.label("identity-name"),
                self.label("ability-score-label"),
            ];
            table(out, &headers, &characteristics);
        }
        if !familiar.personality_traits.is_empty() {
            self.section(out, 4, "personality-label");
            write_traits(out, &familiar.personality_traits);
        }
        let powers = self.leveled_rows(&familiar.powers, "power-level-label");
        if !powers.is_empty() {
            self.section(out, 4, "familiar-powers-label");
            table(out, &powers.headers, &powers.rows);
        }
    }

    /// The annotation block: Warping, Twilight Scars, Decrepitude, the chosen Living
    /// Conditions, the accrued aging points, and the aging log. It reads from the
    /// standing choice through the accrued state to the year-by-year history.
    pub(super) fn write_annotations(&self, out: &mut String) {
        let e = self.entity;
        let warping = warping(e, self.rules());
        let decrepitude = decrepitude_score(e, self.rules());
        let mut body = String::new();
        // Ahead of the subsections, because it is the standing choice the rest of
        // the block is the consequence of — and because a bullet sitting after a
        // `###` heading would read as part of that subsection.
        self.write_living_conditions(&mut body);
        if warping.score != 0 || warping.points != 0 || !e.warping_effect.trim().is_empty() {
            self.section(&mut body, 3, "warping-label");
            self.labelled(&mut body, "ability-score-label", &warping.score.to_string());
            let points = warping.points.to_string();
            self.labelled(&mut body, "warping-points-label", &points);
            if !e.warping_effect.trim().is_empty() {
                let effect = escape_cell(&e.warping_effect);
                self.labelled(&mut body, "warping-effect-label", &effect);
            }
            body.push('\n');
        }
        if !e.twilight_scars.is_empty() {
            self.section(&mut body, 3, "twilight-scars-label");
            for scar in &e.twilight_scars {
                body.push_str("- ");
                body.push_str(&escape_cell(&scar.description));
                body.push('\n');
            }
            body.push('\n');
        }
        if decrepitude != 0 || !e.decrepitude_effect.trim().is_empty() {
            self.section(&mut body, 3, "decrepitude-label");
            self.labelled(&mut body, "ability-score-label", &decrepitude.to_string());
            if !e.decrepitude_effect.trim().is_empty() {
                let effect = escape_cell(&e.decrepitude_effect);
                self.labelled(&mut body, "decrepitude-effect-label", &effect);
            }
            body.push('\n');
        }
        self.write_aging_points(&mut body);
        if !e.aging_log.is_empty() {
            self.section(&mut body, 3, "aging-log-heading");
            for entry in &e.aging_log {
                let recorded = self.aging_log_entry(entry);
                // A character with no birth year has no calendar year to label the
                // entry with (see `AgingLogEntry::year`), so it prints as a plain
                // bullet rather than an empty bold label.
                match entry.year {
                    Some(year) => field(&mut body, &year.to_string(), &recorded),
                    None => {
                        body.push_str("- ");
                        body.push_str(&recorded);
                        body.push('\n');
                    }
                }
            }
            body.push('\n');
        }
        if body.is_empty() {
            return;
        }
        self.section(out, 2, "aging-label");
        out.push_str(&body);
    }

    /// The chosen Living Conditions, one inline list of localized names.
    ///
    /// They are a **stored choice** ([`Entity::living_conditions`]) and a standing
    /// term of every future aging total — "AGING TOTAL: Stress die (no botch) +
    /// age/10 (round up) - Living Conditions modifier"
    /// (Ars Magica - Definitive Edition (Core Rules).md:16567-16569, table at
    /// :16581-16594) — so a sheet that dropped them would read as data loss. The
    /// resolved modifier is deliberately *not* printed: it is derived from these ids
    /// and the character's Virtues and Flaws, and the sheet records choices.
    fn write_living_conditions(&self, out: &mut String) {
        let chosen = &self.entity.living_conditions;
        if chosen.is_empty() {
            return;
        }
        let names: Vec<String> = chosen
            .iter()
            .map(|id| escape_cell(&self.name(id)))
            .collect();
        self.labelled(
            out,
            "living-conditions-label",
            &names.join(&self.list_separator()),
        );
        out.push('\n');
    }

    /// One logged year's recorded detail: its free text, then the stress die the
    /// player typed and the AGING TOTAL it produced, then the Crisis the year sent
    /// the character to — where the entry carries them.
    ///
    /// A resolved year may leave the free text empty and let the structured fields
    /// speak, and a hand-written entry carries no die at all, so each part is
    /// included only when it is there.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16569, :16621.
    fn aging_log_entry(&self, entry: &AgingLogEntry) -> String {
        let mut parts: Vec<String> = Vec::new();
        let effect = escape_cell(&entry.effect);
        if !effect.is_empty() {
            parts.push(effect);
        }
        if let Some(die) = entry.die {
            parts.push(pair(&self.label("aging-die-label"), &die.to_string()));
        }
        if let Some(total) = entry.total {
            parts.push(pair(&self.label("export-col-total"), &total.to_string()));
        }
        parts.extend(self.aging_log_crisis(entry));
        parts.join(SUBTITLE_SEPARATOR)
    }

    /// What the Crisis Table was asked of one logged year and what it answered, or
    /// that it has not been asked yet.
    ///
    /// Three states, and the sheet has to tell them apart: a year with no Crisis
    /// says nothing, a Crisis the aging row demanded that nobody has rolled says so
    /// (`crisis` set with no row — the aging roll happened whether or not the second
    /// die was thrown), and a resolved one prints its row, its severity and the
    /// Simple Die that found it beside the CRISIS TOTAL they made.
    ///
    /// The row travels as an [`Id`], so its text comes from the rules i18n and never
    /// reaches the sheet as a slug; the severity is a Rust taxonomy and goes through
    /// `crisis-severity-<slug>`, which [`LABEL_KEYS`] declares.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16619-16632.
    fn aging_log_crisis(&self, entry: &AgingLogEntry) -> Vec<String> {
        if !entry.crisis {
            return Vec::new();
        }
        let Some(row) = &entry.crisis_row else {
            return vec![self.label("aging-log-crisis-unrolled")];
        };

        let named = match entry.crisis_severity {
            Some(severity) => format!(
                "{} ({})",
                self.name(row),
                self.label(&format!("crisis-severity-{severity}"))
            ),
            // A bedridden row carries no severity: a week in bed is time, not an
            // illness, so there is no rank to print.
            None => self.name(row),
        };
        let mut parts = vec![pair(&self.label("crisis-label"), &named)];
        if let Some(die) = entry.crisis_die {
            parts.push(pair(&self.label("crisis-die-label"), &die.to_string()));
        }
        if let Some(total) = entry.crisis_total {
            parts.push(pair(&self.label("crisis-total-label"), &total.to_string()));
        }
        parts
    }

    /// The accrued aging points, one bullet per Characteristic that carries any.
    ///
    /// These are the recorded state the Decrepitude Score and the Characteristic drops
    /// are both derived from ([`crate::effective::decrepitude_score`],
    /// [`effective_characteristic_after_aging`]), so the sheet shows them: a reader who
    /// sees only the dropped score cannot tell how close the next drop is. A
    /// Characteristic with no points contributes nothing and is skipped, which also
    /// keeps the whole `aging-label` block out of an unaged character's sheet.
    fn write_aging_points(&self, out: &mut String) {
        let accrued: Vec<(Characteristic, u8)> = Characteristic::ALL
            .into_iter()
            .filter_map(|c| {
                let points = self.entity.aging_points.get(&c).copied().unwrap_or(0);
                (points != 0).then_some((c, points))
            })
            .collect();
        if accrued.is_empty() {
            return;
        }
        self.section(out, 3, "aging-points-heading");
        for (c, points) in accrued {
            self.labelled(out, &format!("characteristic-{c}"), &points.to_string());
        }
        out.push('\n');
    }

    /// A name-and-level table for the three list types that share that shape:
    /// enchanted devices, supernatural powers, and instilled talisman effects. Each
    /// names its own "Level" key, since each is a different quantity.
    fn leveled_rows<T: Leveled>(&self, items: &[T], level_key: &str) -> LeveledTable {
        LeveledTable {
            headers: [self.label("identity-name"), self.label(level_key)],
            rows: items
                .iter()
                .map(|item| vec![escape_cell(item.leveled_name()), item.level().to_string()])
                .collect(),
        }
    }
}
