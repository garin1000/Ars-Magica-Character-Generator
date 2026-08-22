//! Hermetic spells: the catalogue of formulaic/ritual spells a magus may know at
//! character creation.
//!
//! A spell is defined by its Technique + Form (both [`Art`](crate::art::Art) ids),
//! its Level, and any Art requisites. Level is either a fixed multiple of five or
//! **General** — a General spell is learned at a per-character chosen level, and
//! two General versions at different levels are different spells
//! (Ars Magica - Definitive Edition (Core Rules).md:12349-12353). The character's
//! chosen spells and (for General
//! spells) their learned levels live on the [`Entity`](crate::types::Entity) as
//! [`SpellSelection`](crate::types::SpellSelection)s; the catalogue here is
//! language-neutral mechanics, with names/descriptions in `rules/i18n`.
//!
//! Spells consume the magus's *spell-levels budget* (120 out of apprenticeship,
//! Ars Magica - Definitive Edition (Core Rules).md:2215-2216, 2435, plus whatever
//! the magus took as levels out of its
//! years past the Gauntlet, `:2471`), a currency distinct from the shared
//! Ability/Art XP pool. Enforcement lives in [`crate::validation`].

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::types::{Id, ParameterDef, SourceRef};

/// A spell's Range — how far the target may be from the caster. Ordered least- to
/// most-difficult, matching the RDT chart. Its label lives in Fluent
/// (`spell-range-<scalar>`), never rendered as the raw slug.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:12001-12027.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellRange {
    /// Affects only the caster.
    Personal,
    /// The caster or anything he touches.
    Touch,
    /// A target the caster has eye contact with (same difficulty as Touch).
    Eye,
    /// Anything the caster's voice carries to.
    Voice,
    /// Anything the caster can see.
    Sight,
    /// Anything the caster has an Arcane Connection to.
    ArcaneConnection,
}

impl fmt::Display for SpellRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SpellRange::Personal => "personal",
            SpellRange::Touch => "touch",
            SpellRange::Eye => "eye",
            SpellRange::Voice => "voice",
            SpellRange::Sight => "sight",
            SpellRange::ArcaneConnection => "arcane_connection",
        })
    }
}

/// A spell's Duration — how long the effect lasts. Year Duration forces a Ritual
/// (Ars Magica - Definitive Edition (Core Rules).md:12055).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:12001-12055.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellDuration {
    /// Lasts a moment then dissipates.
    Momentary,
    /// Lasts while the caster concentrates (same difficulty as Diameter).
    Concentration,
    /// Lasts about two minutes.
    Diameter,
    /// Lasts until the sun next rises or sets.
    Sun,
    /// Lasts until the target leaves, or the ring is broken (as Sun difficulty).
    Ring,
    /// Lasts until both new and full moon have set.
    Moon,
    /// Lasts until the fourth season-boundary; must be Ritual.
    Year,
}

impl fmt::Display for SpellDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SpellDuration::Momentary => "momentary",
            SpellDuration::Concentration => "concentration",
            SpellDuration::Diameter => "diameter",
            SpellDuration::Sun => "sun",
            SpellDuration::Ring => "ring",
            SpellDuration::Moon => "moon",
            SpellDuration::Year => "year",
        })
    }
}

/// A spell's Target — what the effect can affect: objects (Individual, Part,
/// Group), containers (Circle, Room, Structure, Boundary), and magical senses
/// (Taste, Touch, Smell, Hearing, Vision). Boundary forces a Ritual
/// (Ars Magica - Definitive Edition (Core Rules).md:12077); Vision, though
/// equally difficult, does not
/// (Ars Magica - Definitive Edition (Core Rules).md:12099).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:12001-12099.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellTarget {
    /// A single discrete thing (object Target).
    Individual,
    /// Everything within a drawn circle (container Target, as Individual level).
    Circle,
    /// A part of a discrete thing (object Target).
    Part,
    /// A group of close-together things (object Target).
    Group,
    /// Everything within a chamber (container Target, as Group level).
    Room,
    /// Everything within a single structure (container Target).
    Structure,
    /// Everything within a natural/man-made boundary (container Target); Ritual.
    Boundary,
    /// Magical sense via taste (as Individual level).
    Taste,
    /// Magical sense via touch (as Part level).
    Touch,
    /// Magical sense via smell (as Group level).
    Smell,
    /// Magical sense via hearing (as Structure level).
    Hearing,
    /// Magical sense via sight (as Boundary level, but no Ritual required).
    Vision,
}

impl fmt::Display for SpellTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SpellTarget::Individual => "individual",
            SpellTarget::Circle => "circle",
            SpellTarget::Part => "part",
            SpellTarget::Group => "group",
            SpellTarget::Room => "room",
            SpellTarget::Structure => "structure",
            SpellTarget::Boundary => "boundary",
            SpellTarget::Taste => "taste",
            SpellTarget::Touch => "touch",
            SpellTarget::Smell => "smell",
            SpellTarget::Hearing => "hearing",
            SpellTarget::Vision => "vision",
        })
    }
}

/// Minimum level a Ritual spell may be learned/cast at, even if the guideline
/// calculation would produce a lower level. Surfaced to the frontend through
/// [`crate::ruleset::Ruleset`]'s `ritual_min_level` field (see
/// `derived_magnitude_points` in `ruleset.rs` for the sibling pattern this
/// follows) so the UI never re-hardcodes the floor.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:12293 ("Ritual
/// spells are always at least level 20, even if the level calculation would
/// make them lower.").
pub const RITUAL_MIN_LEVEL: u8 = 20;

/// A single spell in the catalogue. Its display name and description live in
/// `rules/i18n`, keyed by `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spell {
    /// Slug-style id, e.g. `spell.pilum_of_fire`.
    pub id: Id,
    /// The spell's Technique (a Technique-class Art id, e.g. `art.creo`).
    pub technique: Id,
    /// The spell's Form (a Form-class Art id, e.g. `art.ignem`).
    pub form: Id,
    /// Fixed level (`Some`, a multiple of five) or **General** (`None`), where a
    /// General spell is learned at a per-character chosen level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    /// Art requisites, if any (each a catalogue Art id). Display detail for M4;
    /// not factored into the per-spell level cap.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requisites: Vec<Id>,
    /// Whether this spell is a Ritual: longer to cast, requires vis, floored at
    /// [`RITUAL_MIN_LEVEL`]
    /// (Ars Magica - Definitive Edition (Core Rules).md:12293).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ritual: bool,
    /// The spell's Range. `None` in the seed data until the full catalogue (5d)
    /// populates it; RDT-dependent legality is checked only when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<SpellRange>,
    /// The spell's Duration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<SpellDuration>,
    /// The spell's Target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<SpellTarget>,
    /// True when a Momentary Creo spell creates a lasting thing — which, per
    /// Ars Magica - Definitive Edition (Core Rules).md:12039/:12115, forces the
    /// spell to be a Ritual.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub creates_lasting: bool,
    /// Selection parameters this spell requires — mirrors [`ParameterDef`] on a
    /// virtue/flaw. A meta-magic Vim spell whose name carries a target `(Form)`
    /// (e.g. Wizard's Boost) declares a single `form`-domain parameter here; the
    /// chosen value is **display + identity only** and does NOT change the spell's
    /// own Technique/Form — those stay the catalogue Vim Arts. Empty for ordinary
    /// spells, so existing catalogue entries are unaffected.
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:15791-15794
    /// ("There are ten versions of this spell, one for each Hermetic Form").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ParameterDef>,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// The on-disk shape of `rules/core/spells.json`. Internal deserialize-only
/// wrapper (`pub(crate)`): parsed by [`crate::ruleset`] at load, never part of the
/// crate's public API. Matches the sibling arts/houses file wrappers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct SpellsFile {
    /// The spell catalogue.
    #[serde(default)]
    pub spells: Vec<Spell>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fixed_spell_roundtrips() {
        let json = r#"{
          "id": "spell.pilum_of_fire",
          "technique": "art.creo",
          "form": "art.ignem",
          "level": 20,
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [14250, 14253] }
        }"#;
        let spell: Spell = serde_json::from_str(json).unwrap();
        assert_eq!(spell.id, Id::new("spell.pilum_of_fire"));
        assert_eq!(spell.technique, Id::new("art.creo"));
        assert_eq!(spell.form, Id::new("art.ignem"));
        assert_eq!(spell.level, Some(20));
        assert!(spell.requisites.is_empty());
        let back = serde_json::to_string(&spell).unwrap();
        assert_eq!(serde_json::from_str::<Spell>(&back).unwrap(), spell);
    }

    /// A General spell omits `level`; a spell may carry requisites.
    #[test]
    fn general_and_requisite_spells_load() {
        let json = r#"{
          "spells": [
            { "id": "spell.aegis_of_the_hearth", "technique": "art.rego", "form": "art.vim" },
            { "id": "spell.coat_of_flame", "technique": "art.creo", "form": "art.ignem",
              "level": 25, "requisites": ["art.rego"] }
          ]
        }"#;
        let file: SpellsFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.spells.len(), 2);
        assert_eq!(file.spells[0].level, None); // General
        assert_eq!(file.spells[1].requisites, vec![Id::new("art.rego")]);
    }

    /// A General spell serializes without a `level` key (skip-if-none).
    #[test]
    fn general_spell_omits_level_key() {
        let spell = Spell {
            id: Id::new("spell.aegis_of_the_hearth"),
            technique: Id::new("art.rego"),
            form: Id::new("art.vim"),
            level: None,
            requisites: Vec::new(),
            ritual: false,
            range: None,
            duration: None,
            target: None,
            creates_lasting: false,
            parameters: Vec::new(),
            source: None,
        };
        let out = serde_json::to_string(&spell).unwrap();
        assert!(
            !out.contains("level"),
            "General spell must omit level: {out}"
        );
        assert!(
            !out.contains("requisites"),
            "empty requisites skipped: {out}"
        );
        // Model-completeness (5d): boolean flags default false and are skipped;
        // absent Range/Duration/Target stay None.
        assert!(
            !out.contains("ritual"),
            "default-false ritual skipped: {out}"
        );
        assert!(
            !out.contains("creates_lasting"),
            "default-false creates_lasting skipped: {out}"
        );
    }

    /// A spell carrying full Range/Duration/Target + ritual flag round-trips.
    /// Source (RDT chart): Ars Magica - Definitive Edition (Core Rules).md:12001-12009.
    #[test]
    fn spell_with_rdt_and_ritual_roundtrips() {
        let json = r#"{
          "id": "spell.aegis_of_the_hearth",
          "technique": "art.rego",
          "form": "art.vim",
          "ritual": true,
          "range": "touch",
          "duration": "year",
          "target": "boundary"
        }"#;
        let spell: Spell = serde_json::from_str(json).unwrap();
        assert!(spell.ritual);
        assert_eq!(spell.range, Some(SpellRange::Touch));
        assert_eq!(spell.duration, Some(SpellDuration::Year));
        assert_eq!(spell.target, Some(SpellTarget::Boundary));
        let back = serde_json::to_string(&spell).unwrap();
        assert_eq!(serde_json::from_str::<Spell>(&back).unwrap(), spell);
    }

    /// `arcane_connection` is the only multi-word RDT scalar; guard its casing.
    #[test]
    fn arcane_connection_range_scalar() {
        assert_eq!(
            serde_json::to_value(SpellRange::ArcaneConnection).unwrap(),
            serde_json::json!("arcane_connection")
        );
    }

    /// Every RDT `Display` impl renders each variant as its snake_case slug. The
    /// per-variant lists are exhaustive, so a variant added without a `Display`
    /// arm is a compile error in the `impl`, and a mis-wired slug fails here.
    #[test]
    fn rdt_display_renders_every_variant_as_slug() {
        let ranges = [
            (SpellRange::Personal, "personal"),
            (SpellRange::Touch, "touch"),
            (SpellRange::Eye, "eye"),
            (SpellRange::Voice, "voice"),
            (SpellRange::Sight, "sight"),
            (SpellRange::ArcaneConnection, "arcane_connection"),
        ];
        for (variant, slug) in ranges {
            assert_eq!(variant.to_string(), slug);
        }

        let durations = [
            (SpellDuration::Momentary, "momentary"),
            (SpellDuration::Concentration, "concentration"),
            (SpellDuration::Diameter, "diameter"),
            (SpellDuration::Sun, "sun"),
            (SpellDuration::Ring, "ring"),
            (SpellDuration::Moon, "moon"),
            (SpellDuration::Year, "year"),
        ];
        for (variant, slug) in durations {
            assert_eq!(variant.to_string(), slug);
        }

        let targets = [
            (SpellTarget::Individual, "individual"),
            (SpellTarget::Circle, "circle"),
            (SpellTarget::Part, "part"),
            (SpellTarget::Group, "group"),
            (SpellTarget::Room, "room"),
            (SpellTarget::Structure, "structure"),
            (SpellTarget::Boundary, "boundary"),
            (SpellTarget::Taste, "taste"),
            (SpellTarget::Touch, "touch"),
            (SpellTarget::Smell, "smell"),
            (SpellTarget::Hearing, "hearing"),
            (SpellTarget::Vision, "vision"),
        ];
        for (variant, slug) in targets {
            assert_eq!(variant.to_string(), slug);
        }

        // The Display slug must match the serde scalar for every variant.
        for (variant, slug) in ranges {
            assert_eq!(serde_json::to_value(variant).unwrap(), slug);
        }
        for (variant, slug) in durations {
            assert_eq!(serde_json::to_value(variant).unwrap(), slug);
        }
        for (variant, slug) in targets {
            assert_eq!(serde_json::to_value(variant).unwrap(), slug);
        }
    }
}
