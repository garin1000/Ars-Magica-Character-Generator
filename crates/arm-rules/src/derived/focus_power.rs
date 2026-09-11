//! The Focus Power read-out: the per-power figures the rulebook derives from a
//! Focus Power's maximum level of effect — its magnitude, its Initiative, and the
//! Fatigue it costs to activate.
//!
//! Display-only, like [`super::familiar`]: no `ValidationIssue` is ever raised
//! from any of it. What *is* enforced is the point pool
//! ([`crate::validation`]'s `over_focus_points`), which lives in `effective.rs`.

use super::*;

/// One Focus Power's derived figures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusPowerLine {
    /// The power's free-text name / scope, as entered.
    pub name: String,
    /// The maximum level of effect the character bought (`ArMDE:3899`).
    pub max_level: u16,
    /// The Penetration bought from the same pool (`ArMDE:3899`).
    pub penetration: u16,
    /// The maximum magnitude of the effect (`ArMDE:9097`).
    pub magnitude: u16,
    /// Initiative = Quickness − the maximum magnitude (`ArMDE:3899`).
    pub initiative: i32,
    /// Fatigue levels spent to activate the power, or `None` above level 75 where
    /// the rulebook states no cost (`ArMDE:3901`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fatigue_levels: Option<u8>,
}

/// A level's magnitude: "Spells also have a magnitude, which is equal to the level
/// divided by five, rounded up" (`ArMDE:9097`). The only sourced level→magnitude
/// rule in the book, and what `ArMDE:3899`'s "maximum magnitude of the effect"
/// resolves to.
fn magnitude_of(level: u16) -> u16 {
    level.div_ceil(5)
}

/// The Fatigue levels activating an effect of this level costs: "It costs one
/// Fatigue level to activate this power for an effect of level 25 or lower, two
/// Fatigue levels to activate it if the effect has a level of 26 to 50, and three
/// for 51 to 75" (`ArMDE:3901`).
///
/// `None` above 75: the table stops there and the book says nothing about higher
/// effects, so the honest answer is "unstated", never a fourth band invented by
/// continuing the pattern.
fn fatigue_levels_for(level: u16) -> Option<u8> {
    match level {
        0..=25 => Some(1),
        26..=50 => Some(2),
        51..=75 => Some(3),
        _ => None,
    }
}

/// One Focus Power read-out line per entered focus power, in entry order.
///
/// Initiative is "the character's Quickness – the maximum magnitude of the effect"
/// (`ArMDE:3899`), against the same aging-adjusted Quickness every other play stat
/// uses.
pub fn focus_power_lines(entity: &Entity, ruleset: &Ruleset) -> Vec<FocusPowerLine> {
    let quickness = characteristic(entity, ruleset, Characteristic::Qik);
    entity
        .focus_powers
        .iter()
        .map(|power| {
            let magnitude = magnitude_of(power.max_level);
            FocusPowerLine {
                name: power.name.clone(),
                max_level: power.max_level,
                penetration: power.penetration,
                magnitude,
                initiative: quickness - i32::from(magnitude),
                fatigue_levels: fatigue_levels_for(power.max_level),
            }
        })
        .collect()
}
