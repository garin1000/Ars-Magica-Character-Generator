//! The Life table (G4): the character's life timeline ([`crate::life_timeline`])
//! as one table, drawn only while the player has the life sections switched on —
//! so a sheet without them is byte-identical to one written before they existed.
//! Split out like `export/sections.rs` and `export/magic.rs`; it computes nothing
//! of its own, it lays out what [`life_timeline`] already derived.

use super::*;
use crate::life_stage::{LabYears, LifeStagePlan};
use crate::life_timeline::{EntryCapacity, TimelineEntry, TimelineEntryKind, life_timeline};

/// Between a count of years and the lab seasons each held ("1 × 4 seasons").
/// Punctuation, not prose — no locale owns it.
const TIMES: &str = " × ";

impl<'a> Doc<'a> {
    /// The Life table, then one line for the years the sections claim past the
    /// character's age and one for the experience no span of years pays for.
    ///
    /// A magus's table gains a lab-years column: each post-Gauntlet span's years
    /// counted by the lab seasons they held, the column [`life_timeline`] charged
    /// its points by. The fixed spans leave that cell blank.
    pub(super) fn write_life(&self, out: &mut String) {
        let Some(plan) = self
            .entity
            .life_stages
            .as_ref()
            .filter(|plan| plan.life_sections_enabled)
        else {
            return;
        };
        let Ok(allocation) = checked_xp_allocation(self.entity, self.rules()) else {
            return;
        };
        let Some(timeline) = life_timeline(self.entity, self.rules(), &allocation) else {
            return;
        };
        let lab_column = timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.capacity, EntryCapacity::Shared { .. }));
        let rows: Vec<Vec<String>> = timeline
            .entries
            .iter()
            .map(|entry| self.life_row(plan, entry, lab_column))
            .collect();
        self.section(out, 2, "life-timeline-label");
        let mut headers = vec![
            self.label("export-life-col-section"),
            self.label("export-life-col-ages"),
            self.label("life-section-years-label"),
            self.label("export-life-col-earned"),
            self.label("export-life-col-filled"),
        ];
        if lab_column {
            headers.push(self.label("life-stage-lab-years-legend"));
        }
        table(out, &headers, &rows);

        let mut notes = String::new();
        if timeline.overrun_years > 0 {
            self.labelled(
                &mut notes,
                "export-life-overrun",
                &timeline.overrun_years.to_string(),
            );
        }
        if !timeline.outside.is_empty() {
            let xp = self.label("export-life-unit-xp");
            let lumps: Vec<String> = timeline
                .outside
                .iter()
                .map(|lump| {
                    format!(
                        "{} {} / {} {xp}",
                        escape_cell(&self.name(&lump.item)),
                        lump.used,
                        lump.amount
                    )
                })
                .collect();
            self.labelled(
                &mut notes,
                "life-timeline-outside-label",
                &lumps.join(&self.list_separator()),
            );
        }
        if !notes.is_empty() {
            out.push_str(&notes);
            out.push('\n');
        }
    }

    fn life_row(
        &self,
        plan: &LifeStagePlan,
        entry: &TimelineEntry,
        lab_column: bool,
    ) -> Vec<String> {
        let mut row = vec![
            self.life_span_name(plan, entry.kind),
            format!("{}{RANGE_DASH}{}", entry.start_age, entry.end_age),
            entry.years.to_string(),
            self.life_earned(entry.capacity),
            self.life_filled(entry),
        ];
        if lab_column {
            row.push(self.lab_years_cell(plan, entry));
        }
        row
    }

    /// A fixed span by its localized name; a named section by the player's name,
    /// escaped like all free text, or "Section n" when left blank.
    fn life_span_name(&self, plan: &LifeStagePlan, kind: TimelineEntryKind) -> String {
        match kind {
            TimelineEntryKind::Fixed { section } => {
                self.label(&format!("life-section-fixed-{section}"))
            }
            TimelineEntryKind::Named { index } => {
                let name = plan
                    .life_sections
                    .get(index)
                    .map_or("", |section| section.name.as_str());
                if name.trim().is_empty() {
                    format!(
                        "{} {}",
                        self.label("export-life-section-unnamed"),
                        index + 1
                    )
                } else {
                    escape_cell(name)
                }
            }
            TimelineEntryKind::Remaining => self.label("life-section-remaining"),
        }
    }

    /// What the span's years earned: experience, experience and levels of spells
    /// (an apprenticeship's two numbers), or a magus's post-Gauntlet points.
    fn life_earned(&self, capacity: EntryCapacity) -> String {
        match capacity {
            EntryCapacity::Separate {
                xp,
                spell_levels: 0,
            } => self.life_amount(xp, "export-life-unit-xp"),
            EntryCapacity::Separate { xp, spell_levels } => {
                self.xp_and_spell_levels(xp, spell_levels)
            }
            EntryCapacity::Shared { points } => self.life_amount(points, "export-life-unit-points"),
        }
    }

    /// What the character's purchases draw from the span, in the units it earned
    /// them — points being spent as experience and levels of spells both.
    fn life_filled(&self, entry: &TimelineEntry) -> String {
        match entry.capacity {
            EntryCapacity::Separate {
                spell_levels: 0, ..
            } => self.life_amount(entry.filled_xp, "export-life-unit-xp"),
            _ => self.xp_and_spell_levels(entry.filled_xp, entry.filled_spell_levels),
        }
    }

    fn xp_and_spell_levels(&self, xp: u32, spell_levels: u32) -> String {
        format!(
            "{}{}{}",
            self.life_amount(xp, "export-life-unit-xp"),
            self.list_separator(),
            self.life_amount(spell_levels, "export-life-unit-spell-levels")
        )
    }

    fn life_amount(&self, amount: u32, unit_key: &str) -> String {
        format!("{amount} {}", self.label(unit_key))
    }

    /// A post-Gauntlet span's lab years ("1 × 1 season, 1 × 2 seasons"), fewest
    /// seasons first; blank for any other span or a span without lab work.
    fn lab_years_cell(&self, plan: &LifeStagePlan, entry: &TimelineEntry) -> String {
        if !matches!(entry.capacity, EntryCapacity::Shared { .. }) {
            return String::new();
        }
        let lab_years = match entry.kind {
            TimelineEntryKind::Named { index } => plan
                .life_sections
                .get(index)
                .map(|section| section.lab_years)
                .unwrap_or_default(),
            TimelineEntryKind::Remaining => plan.post_gauntlet_lab_years,
            TimelineEntryKind::Fixed { .. } => LabYears::default(),
        };
        [
            (lab_years.one_season, "life-stage-lab-years-1-label"),
            (lab_years.two_seasons, "life-stage-lab-years-2-label"),
            (lab_years.three_seasons, "life-stage-lab-years-3-label"),
            (lab_years.four_seasons, "life-stage-lab-years-4-label"),
        ]
        .into_iter()
        .filter(|&(years, _)| years > 0)
        .map(|(years, key)| format!("{years}{TIMES}{}", self.label(key)))
        .collect::<Vec<_>>()
        .join(&self.list_separator())
    }
}
