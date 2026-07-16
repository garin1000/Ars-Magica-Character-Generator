//! Equipment: the catalogue of weapons, shields, and armor a character may carry.
//!
//! This slice (M5/5h) defines the language-neutral **catalogue** and stores the
//! character's equipment choices (see [`crate::types::EquipmentSlot`]); it does
//! **not** compute combat totals, Soak, or Encumbrance — that lands in the
//! derived-totals slice (5i), which consumes these rows. The data shape here is
//! therefore deliberately rich enough for 5i to compute Init/Atk/Def/Dam, Soak,
//! and Encumbrance without re-reading the rulebook.
//!
//! Three record types, mirroring the three source tables:
//! - [`Weapon`] — the Melee and Missile weapon tables. A weapon names the combat
//!   [`Ability`](crate::ability::Ability) it uses (Brawl, Single Weapon, Great
//!   Weapon, Thrown Weapon, Bows) and carries its Init/Atk/Def/Dam modifiers, a
//!   minimum-Strength requirement, a Load, and (missiles only) a Range.
//! - [`Shield`] — shields are their own table because a shield's modifiers **add
//!   to** the wielded weapon's line (Core Rules.md:16656); 5i pairs a wielded
//!   weapon with a wielded shield and sums them.
//! - [`Armor`] — the Armor table, split into one row per material *and coverage*
//!   (partial / full), each with a Protection (Soak) bonus and a Load.
//!
//! `n/a` cells: the Dodge row has no Attack and no Damage; Dodge/Fist/Kick (body
//! attacks) have no minimum-Strength column. These are modeled as `Option` set to
//! `None`, so 5i can tell "no such stat" from a real 0 modifier (Fist's Attack is a
//! genuine +0, distinct from Dodge's absent one).
//!
//! Source: Ars Magica - Definitive Edition (Core Rules).md — Armor Table
//! :16944-16949, Melee Weapon Statistics :16959-16986, Missile Weapon Statistics
//! :17005-17011. Display names live in `rules/i18n/<lang>/equipment.json`, keyed
//! by `id`.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::types::{Id, SourceRef};

/// Which weapon table a [`Weapon`] comes from, and how it reaches its target.
/// A fixed rules taxonomy, so an enum; its label lives in Fluent
/// (`weapon-kind-<scalar>`), never rendered as the raw slug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponKind {
    /// A hand-to-hand weapon (Melee Weapon Statistics table).
    Melee,
    /// A launched missile weapon — a bow or sling that stays in hand (Bows).
    Missile,
    /// A hurled weapon (Thrown Weapon), which also carries a Range.
    Thrown,
}

impl WeaponKind {
    /// Every kind, for exhaustive iteration in the module's guard test
    /// (serde-scalar/Display agreement). No `Ruleset` field surfaces a weapon-kind
    /// ordering — nothing consumes one — so unlike `ability_category_order` /
    /// `art_type_order` this is a test-only taxonomy enumeration, not a
    /// serialized-ordering source.
    pub const ALL: [WeaponKind; 3] = [WeaponKind::Melee, WeaponKind::Missile, WeaponKind::Thrown];
}

impl fmt::Display for WeaponKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            WeaponKind::Melee => "melee",
            WeaponKind::Missile => "missile",
            WeaponKind::Thrown => "thrown",
        })
    }
}

/// A single weapon in the catalogue. Its display name lives in `rules/i18n`, keyed
/// by `id`.
///
/// All modifiers are signed and applied on top of the wielder's Characteristic +
/// Ability in the combat formulas (5i). `attack_mod` and `damage_mod` are
/// `Option` because the Dodge row has neither (`n/a`); every other row carries a
/// real value (Fist's `+0` is distinct from Dodge's absent stat). `min_strength`
/// is `Option` because the body attacks (Dodge/Fist/Kick) have no `Str` column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    /// Slug-style id, e.g. `weapon.long_sword`.
    pub id: Id,
    /// Which table this weapon comes from (melee / missile / thrown).
    pub kind: WeaponKind,
    /// The Weapon Initiative Modifier.
    pub init_mod: i8,
    /// The Weapon Attack Modifier; `None` for Dodge (no attack).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_mod: Option<i8>,
    /// The Weapon Defense Modifier.
    pub defense_mod: i8,
    /// The Weapon Damage Modifier; `None` for Dodge (no damage).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_mod: Option<i8>,
    /// The minimum Strength score needed to wield the weapon; `None` for body
    /// attacks (Dodge/Fist/Kick) which have no requirement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_strength: Option<i8>,
    /// The weapon's contribution to Encumbrance Load.
    pub load: u8,
    /// The range increment in paces (missile / thrown weapons only); `None` for
    /// melee weapons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<u16>,
    /// The combat Ability this weapon uses (e.g. `ability.single_weapon`,
    /// `ability.brawl`, `ability.bows`).
    pub ability: Id,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// A shield in the catalogue. Its display name lives in `rules/i18n`, keyed by
/// `id`.
///
/// A shield is modeled separately from weapons because a shield's modifiers **add
/// to** the wielded weapon's combat line (Core Rules.md:16656): 5i combines the
/// wielded weapon and shield rows. Shields carry their own minimum-Strength
/// requirement, checked independently of the weapon's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shield {
    /// Slug-style id, e.g. `shield.heater`.
    pub id: Id,
    /// The shield's Initiative Modifier (added to the weapon line).
    pub init_mod: i8,
    /// The shield's Attack Modifier (added to the weapon line).
    pub attack_mod: i8,
    /// The shield's Defense Modifier — a shield's defining benefit.
    pub defense_mod: i8,
    /// The shield's contribution to Encumbrance Load.
    pub load: u8,
    /// The minimum Strength score needed to use the shield (met separately from
    /// the weapon's requirement, Core Rules.md:16993).
    pub min_strength: i8,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// A suit of armor in the catalogue (one row per material *and* coverage —
/// partial vs full). Its display name lives in `rules/i18n`, keyed by `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Armor {
    /// Slug-style id, e.g. `armor.chain_mail_full`.
    pub id: Id,
    /// The Soak (Protection) bonus the armor grants.
    pub protection: u8,
    /// The armor's contribution to Encumbrance Load.
    pub load: u8,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// The on-disk shape of `rules/core/equipment.json`. Internal deserialize-only
/// wrapper (`pub(crate)`): parsed by [`crate::ruleset`] at load, never part of the
/// crate's public API. Matches the sibling arts/spells/houses file wrappers.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub(crate) struct EquipmentFile {
    /// The weapon catalogue (melee + missile tables).
    #[serde(default)]
    pub weapons: Vec<Weapon>,
    /// The shield catalogue.
    #[serde(default)]
    pub shields: Vec<Shield>,
    /// The armor catalogue (per material and coverage).
    #[serde(default)]
    pub armor: Vec<Armor>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn weapon_kind_display_matches_serde_scalar() {
        for kind in WeaponKind::ALL {
            let scalar = serde_json::to_value(kind).unwrap();
            assert_eq!(scalar.as_str().unwrap(), kind.to_string());
        }
    }

    /// A melee weapon round-trips; its Range is absent and Attack/Damage present.
    #[test]
    fn melee_weapon_roundtrips() {
        let json = r#"{
          "id": "weapon.long_sword",
          "kind": "melee",
          "init_mod": 2,
          "attack_mod": 4,
          "defense_mod": 1,
          "damage_mod": 6,
          "min_strength": 0,
          "load": 1,
          "ability": "ability.single_weapon",
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [16974, 16974] }
        }"#;
        let w: Weapon = serde_json::from_str(json).unwrap();
        assert_eq!(w.id, Id::new("weapon.long_sword"));
        assert_eq!(w.kind, WeaponKind::Melee);
        assert_eq!(w.attack_mod, Some(4));
        assert_eq!(w.damage_mod, Some(6));
        assert_eq!(w.range, None);
        assert_eq!(w.ability, Id::new("ability.single_weapon"));
        let back = serde_json::to_string(&w).unwrap();
        assert_eq!(serde_json::from_str::<Weapon>(&back).unwrap(), w);
    }

    /// Dodge has no Attack, no Damage, and no min-Strength: all three are `None`
    /// and serialize away (skip-if-none), distinct from a real 0.
    #[test]
    fn dodge_omits_na_columns() {
        let w = Weapon {
            id: Id::new("weapon.dodge"),
            kind: WeaponKind::Melee,
            init_mod: 0,
            attack_mod: None,
            defense_mod: 0,
            damage_mod: None,
            min_strength: None,
            load: 0,
            range: None,
            ability: Id::new("ability.brawl"),
            source: None,
        };
        let out = serde_json::to_string(&w).unwrap();
        assert!(!out.contains("attack_mod"), "n/a attack skipped: {out}");
        assert!(!out.contains("damage_mod"), "n/a damage skipped: {out}");
        assert!(!out.contains("min_strength"), "n/a strength skipped: {out}");
        assert!(!out.contains("range"), "melee range skipped: {out}");
        assert_eq!(serde_json::from_str::<Weapon>(&out).unwrap(), w);
    }

    /// A missile weapon carries a Range; a thrown weapon uses the Thrown ability.
    #[test]
    fn missile_and_thrown_weapons_load() {
        let json = r#"{
          "weapons": [
            { "id": "weapon.bow_long", "kind": "missile", "init_mod": -2, "attack_mod": 4,
              "defense_mod": 0, "damage_mod": 8, "min_strength": 2, "load": 2, "range": 30,
              "ability": "ability.bows" },
            { "id": "weapon.javelin", "kind": "thrown", "init_mod": 0, "attack_mod": 2,
              "defense_mod": 0, "damage_mod": 5, "min_strength": 0, "load": 1, "range": 10,
              "ability": "ability.thrown_weapon" }
          ]
        }"#;
        let file: EquipmentFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.weapons.len(), 2);
        assert_eq!(file.weapons[0].range, Some(30));
        assert_eq!(file.weapons[1].kind, WeaponKind::Thrown);
    }

    /// A shield and an armor row round-trip.
    #[test]
    fn shield_and_armor_roundtrip() {
        let json = r#"{
          "shields": [
            { "id": "shield.heater", "init_mod": 0, "attack_mod": 0, "defense_mod": 3,
              "load": 2, "min_strength": 0 }
          ],
          "armor": [
            { "id": "armor.chain_mail_full", "protection": 9, "load": 6 }
          ]
        }"#;
        let file: EquipmentFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.shields[0].defense_mod, 3);
        assert_eq!(file.armor[0].protection, 9);
        let s_back = serde_json::to_string(&file.shields[0]).unwrap();
        assert_eq!(
            serde_json::from_str::<Shield>(&s_back).unwrap(),
            file.shields[0]
        );
        let a_back = serde_json::to_string(&file.armor[0]).unwrap();
        assert_eq!(
            serde_json::from_str::<Armor>(&a_back).unwrap(),
            file.armor[0]
        );
    }
}
