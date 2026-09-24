//! Encumbrance, Combat (weapon lines), Soak, and Fatigue & Wounds — the
//! non-magic play-stat totals every character type gets (Wave 6, `derived.rs`
//! module split; extracted verbatim from `derived.rs`, no logic changed).
//!
//! `encumbrance` is defined here (Viktor's V5 "Combat" grouping) even though
//! [`super::casting`] also calls it (a Casting Score subtracts Encumbrance) —
//! it is already `pub fn`, so the cross-module call is a plain `use
//! super::combat::encumbrance;`, no visibility change.

use super::*;

// --- Encumbrance -----------------------------------------------------------

/// The Encumbrance read-out: total Load, Burden, and the Encumbrance penalty.
///
/// Burden comes from the Load table (ArMDE:17103-17123); Encumbrance =
/// max(0, Burden − max(0, Strength)).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncumbranceTotal {
    /// Total Load of all carried equipment.
    pub load: u32,
    /// Burden derived from the Load table.
    pub burden: i32,
    /// The Encumbrance penalty (≥ 0).
    pub total: i32,
}

/// The Load-table thresholds: index = Burden, value = Load at which that Burden
/// begins (ArMDE:17103-17123).
const LOAD_TABLE: [u32; 11] = [0, 1, 3, 6, 10, 15, 21, 28, 36, 45, 55];

/// Burden for a total Load: the highest table index whose threshold is ≤ load.
fn burden_for_load(load: u32) -> i32 {
    let mut burden = 0;
    for (b, threshold) in LOAD_TABLE.iter().enumerate() {
        if load >= *threshold {
            burden = b as i32;
        }
    }
    burden
}

/// The character's Encumbrance, from **equipped** gear only — a stowed spare
/// weapon costs nothing.
///
/// This is a deliberate departure from the rulebook's own practice, not an
/// attempt to copy it. The rule says to total "the Load that a character is
/// carrying" (ArMDE:17107) and never defines carried-but-stowed, and the worked
/// templates show the book resolving that silence the *other* way: every
/// statblock applies one Encumbrance to every Combat row, computed from the whole
/// carried kit. Four of them prove it, because a row's Initiative exposes the
/// Encumbrance behind it (ArMDE:16658) and their unarmed rows still pay for the
/// weapon not being used — the Standard Soldier's Fist at ArMDE:1354 implies 3,
/// the Grizzled Veteran's Kick at ArMDE:1249 implies 4, the Tough Guy's Fist at
/// ArMDE:1390 implies 3, the Specialist's Fist at ArMDE:1318 implies 2, each the
/// full-loadout figure rather than the weapon-less one.
///
/// Equipped-only was chosen anyway, as the sounder model: a sheathed sword should
/// not slow its owner down, and under the whole-kit reading merely recording gear
/// a character owns costs them Encumbrance and Initiative. It also happens to
/// reproduce the Knight (ArMDE:1447-1486), whose printed "Encumbrance: 2 (3)"
/// (ArMDE:1484) omits his great sword's Load 2 and so contradicts both his own
/// Equipment line and the method above.
///
/// Pinned by `derived.rs::only_equipped_gear_counts_toward_load` and by the
/// Knight's entry in `tests/book_templates.rs`.
///
/// Source: ArMDE:17103-17123.
pub fn encumbrance(entity: &Entity, ruleset: &Ruleset) -> EncumbranceTotal {
    let load: u32 = entity
        .equipment
        .iter()
        .filter(|slot| slot.equipped)
        .map(|slot| equipment_load(ruleset, &slot.item))
        .sum();
    let burden = burden_for_load(load);
    let strength = characteristic(entity, ruleset, Characteristic::Str);
    let total = (burden - strength.max(0)).max(0);
    EncumbranceTotal {
        load,
        burden,
        total,
    }
}

/// The Load of one equipment id (weapon, shield, or armor); 0 if unknown.
fn equipment_load(ruleset: &Ruleset, id: &Id) -> u32 {
    if let Some(w) = ruleset.weapon(id) {
        u32::from(w.load)
    } else if let Some(s) = ruleset.shield(id) {
        u32::from(s.load)
    } else if let Some(a) = ruleset.armor_item(id) {
        u32::from(a.load)
    } else {
        0
    }
}

// --- Combat ----------------------------------------------------------------

/// One way of wielding an equipped weapon. A one-handed weapon carried alongside a
/// shield yields **two** lines — one with every equipped shield's modifiers combined
/// in (ArMDE:16656) and one bare — because both are legal choices in play. A
/// two-handed weapon receives no shield modifiers (ArMDE:7494) and so yields a single
/// line. Attack / Damage are `None` for a weapon that lacks them (Dodge). Initiative
/// is always reduced by Encumbrance (ArMDE:16658); Attack and Defense are reduced only
/// when the Encumbrance is **not** largely due to weapons and armor (ArMDE:17105),
/// which no modelled character can be — see the invariant noted at the
/// `atk_def_enc` binding in [`combat_totals`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatLine {
    /// The weapon id.
    pub weapon: Id,
    /// Every equipped shield whose modifiers this line folded in, in equipment order.
    /// Empty on a bare line, on a two-handed weapon's line, and when no shield is
    /// equipped — so the renderers label the line by the weapon alone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shields: Vec<Id>,
    /// The combat Ability id the weapon uses.
    pub ability: Id,
    /// Initiative total (Qik + weapon/shield Init − Encumbrance + CombatMod).
    pub initiative: i32,
    /// Attack total; `None` if the weapon has no attack (Dodge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack: Option<i32>,
    /// Defense total.
    pub defense: i32,
    /// Damage total; `None` if the weapon has no damage (Dodge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<i32>,
    /// The weapon's Range in paces (missile / thrown); `None` for melee.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<u16>,
}

/// Combat lines: **one or two** per equipped weapon. With a shield equipped, a
/// one-handed weapon yields a with-shield line (every equipped shield's Init/Atk/Def
/// modifiers added — Source: ArMDE:16656)
/// followed by a bare line, because the fighter may drop the shield at will and a
/// Single Weapon specialty "covers using that weapon with any shield or none"
/// (Source: ArMDE:7746). With-shield first
/// mirrors the book's own statblocks, which list the weapon-and-shield lines ahead of
/// the rest (Source: ArMDE:1467-1472). The
/// two lines differ ONLY by the shield modifiers — specialization and Encumbrance are
/// shield-independent. Source: ArMDE:16658-16670, :16656, :7746, :17105.
pub fn combat_totals(entity: &Entity, ruleset: &Ruleset) -> Vec<CombatLine> {
    let mods = in_play_mods(entity, ruleset);
    let quickness = characteristic(entity, ruleset, Characteristic::Qik);
    let dexterity = characteristic(entity, ruleset, Characteristic::Dex);
    let strength = characteristic(entity, ruleset, Characteristic::Str);
    let enc = encumbrance(entity, ruleset).total;

    // Every equipped shield's modifiers sum into one combined pseudo-shield.
    // Source: ArMDE:16656
    let shield_ids: Vec<Id> = entity
        .equipment
        .iter()
        .filter(|s| s.equipped && ruleset.shield(&s.item).is_some())
        .map(|s| s.item.clone())
        .collect();
    let (shield_init, shield_attack, shield_defense) = shield_ids
        .iter()
        .filter_map(|id| ruleset.shield(id))
        .fold((0i32, 0i32, 0i32), |(i, a, d), sh| {
            (
                i + i32::from(sh.init_mod),
                a + i32::from(sh.attack_mod),
                d + i32::from(sh.defense_mod),
            )
        });

    // The combat-total modifier for one stat on one weapon's lines: every unscoped
    // modifier, plus any delta an item scoped to this weapon (Lame's -3 on Dodge,
    // ArMDE:6332). A weapon nobody scoped a figure to sees the unscoped sum alone.
    let cm = |stat: CombatStat, weapon: &Id| {
        mods.combat_mods.get(&stat).copied().unwrap_or(0)
            + mods
                .weapon_combat_mods
                .get(weapon)
                .and_then(|per_stat| per_stat.get(&stat))
                .copied()
                .unwrap_or(0)
    };

    // Attack/Defense take the Encumbrance penalty only when the Encumbrance is NOT
    // "largely due to weapons and armor"; Initiative always takes it
    // (ArMDE:17105, :16658).
    //
    // That waiver is unconditional here, because **all** modelled Load is combat
    // gear by construction: `EquipmentCatalogue` holds exactly `weapons`, `shields`
    // and `armor` (`equipment.rs`), `load` is declared on exactly those three
    // structs, and `equipment_load` returns 0 for anything it cannot resolve. The
    // rules agree — Load is "listed in the Armor and Weapons tables" (ArMDE:17107),
    // so non-combat carried gear has no Load to contribute in the first place.
    //
    // This was previously written as a live majority test over combat Load vs total
    // Load. Both sums ranged over the same items, so the predicate was identically
    // `2n >= n` and this term was always 0 — a decision in shape only, with a test
    // that could only ever exercise the one reachable answer. Revisit if a
    // non-combat load-bearing item is ever modelled: that means a new catalogue
    // collection and a new `equipment_load` arm, which lands on this comment.
    let atk_def_enc = 0;

    let mut out = Vec::new();
    for slot in entity.equipment.iter().filter(|s| s.equipped) {
        let Some(weapon) = ruleset.weapon(&slot.item) else {
            continue;
        };
        // Ability specialization (+1) applies to Attack and Defense only, when the
        // slot is flagged and the weapon's Ability carries a specialty aligned to
        // this weapon (ArMDE:7122, :7139). It acts as if the score were one higher.
        // A specialty "covers using that weapon with any shield or none", so it is
        // shield-independent and identical on both lines.
        // Source: ArMDE:7746
        let spec_bonus = i32::from(specialization_bonus(entity, ruleset, slot, weapon));
        let combat_ability =
            effective_ability_score(entity, ruleset, &weapon.ability, None) + spec_bonus;
        // One way of wielding this weapon. The shield modifiers are the only part that
        // differs between the with-shield and the bare line; Encumbrance and the
        // specialization bonus are shield-independent.
        let wielding =
            |shields: Vec<Id>, sh_init: i32, sh_attack: i32, sh_defense: i32| CombatLine {
                weapon: slot.item.clone(),
                shields,
                ability: weapon.ability.clone(),
                initiative: quickness + i32::from(weapon.init_mod) + sh_init - enc
                    + cm(CombatStat::Initiative, &slot.item),
                attack: weapon.attack_mod.map(|m| {
                    dexterity + combat_ability + i32::from(m) + sh_attack - atk_def_enc
                        + cm(CombatStat::Attack, &slot.item)
                }),
                defense: quickness + combat_ability + i32::from(weapon.defense_mod) + sh_defense
                    - atk_def_enc
                    + cm(CombatStat::Defense, &slot.item),
                damage: weapon
                    .damage_mod
                    .map(|m| strength + i32::from(m) + cm(CombatStat::Damage, &slot.item)),
                range: weapon.range,
            };
        let bare = || wielding(Vec::new(), 0, 0, 0);
        // A two-handed weapon cannot be paired with a shield, so it takes none of
        // the combined shield modifiers and offers no choice to print.
        // Source: ArMDE:7494
        if weapon.two_handed || shield_ids.is_empty() {
            out.push(bare());
            continue;
        }
        // Both ways of wielding a one-handed weapon, with the shield first — the order
        // the book's own statblocks use.
        // Source: ArMDE:1467-1472
        out.push(wielding(
            shield_ids.clone(),
            shield_init,
            shield_attack,
            shield_defense,
        ));
        out.push(bare());
    }
    out
}

/// The Ability-specialization bonus for one equipped weapon slot: +1 when the
/// slot is flagged `specialization_applies` AND the entity holds a non-empty
/// specialty on the weapon's combat Ability (so the toggle is not a dead switch).
/// The specialty is a per-weapon alignment the player asserts — the engine never
/// matches specialty text to weapon names — so a flagged slot with a real specialty
/// grants the bonus. Source: ArMDE:7122 (Single Weapon longsword example),
/// :7139 ("Add +1 when using an Ability's specialization").
fn specialization_bonus(
    entity: &Entity,
    _ruleset: &Ruleset,
    slot: &crate::types::EquipmentSlot,
    weapon: &crate::equipment::Weapon,
) -> u8 {
    if !slot.specialization_applies {
        return 0;
    }
    let has_specialty = entity.ability_scores.iter().any(|a| {
        a.ability == weapon.ability && a.specialty.as_deref().is_some_and(|s| !s.trim().is_empty())
    });
    u8::from(has_specialty)
}

// --- Soak ------------------------------------------------------------------

/// The Soak read-out.
///
/// Soak = Stamina + Armor Protection + SoakMod (Tough +3) + Bronze cord
/// (ArMDE:16666, :10840-10844). The
/// magus Form bonus is situational and shown as an entered addend of 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoakTotal {
    /// The labelled breakdown (stamina, armor, soak_mod, bronze_cord, form_bonus).
    pub addends: Vec<Addend>,
    /// The Soak total.
    pub total: i32,
}

/// The character's Soak. Source: ArMDE:16666, :5145-5147 (Tough), :10840-10844
/// (Bronze cord). The cord addend goes through [`bronze_cord_bonus`] and thus
/// [`cord_score`], so it can never exceed the +5 maximum (ArMDE:10836) or
/// disagree with the other cord read-outs.
pub fn soak(entity: &Entity, ruleset: &Ruleset) -> SoakTotal {
    let mods = in_play_mods(entity, ruleset);
    let stamina = characteristic(entity, ruleset, Characteristic::Sta);
    let armor: i32 = entity
        .equipment
        .iter()
        .filter(|s| s.equipped)
        .filter_map(|s| ruleset.armor_item(&s.item))
        .map(|a| i32::from(a.protection))
        .sum();
    let bronze = bronze_cord_bonus(entity);
    let addends = vec![
        Addend::new("stamina", stamina),
        Addend::new("armor", armor),
        Addend::new("soak_mod", mods.soak_mod),
        Addend::new("bronze_cord", bronze),
        Addend::new("form_bonus", 0),
    ];
    let total = sum(&addends);
    SoakTotal { addends, total }
}

// --- Fatigue & Wounds ------------------------------------------------------

/// The five penalty-bearing Fatigue levels. A fixed rules taxonomy, rendered via
/// Fluent, never as a raw slug. Unconscious is a game state with no action penalty
/// and is intentionally not a variant here. Source: ArMDE:17127-17129.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FatigueTier {
    /// No fatigue; no penalty.
    Fresh,
    /// One level lost; no penalty.
    Winded,
    /// Weary: −1 to all actions.
    Weary,
    /// Tired: −3 to all actions.
    Tired,
    /// Dazed: −5 to all actions.
    Dazed,
}

impl std::fmt::Display for FatigueTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            FatigueTier::Fresh => "fresh",
            FatigueTier::Winded => "winded",
            FatigueTier::Weary => "weary",
            FatigueTier::Tired => "tired",
            FatigueTier::Dazed => "dazed",
        })
    }
}

/// A Fatigue level and the penalty it imposes on all actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FatigueLevel {
    /// The Fatigue tier this row reports; serializes to its stable slug
    /// (`"fresh"`, `"winded"`, `"weary"`, `"tired"`, `"dazed"`), mapped through
    /// Fluent. Unconscious is a game state and is intentionally not surfaced here.
    pub level: FatigueTier,
    /// The penalty applied at this level (≤ 0), after any HealthMod fatigue delta.
    pub penalty: i32,
}

/// The five penalty-bearing Fatigue levels and their penalties, adjusted by
/// HealthMod fatigue deltas (a positive delta reduces the penalty magnitude).
/// Unconscious is omitted (it is a state, not an action penalty). Source:
/// ArMDE:17127-17129.
pub fn fatigue_levels(entity: &Entity, ruleset: &Ruleset) -> Vec<FatigueLevel> {
    let mods = in_play_mods(entity, ruleset);
    let delta = mods
        .health_mods
        .get(&HealthTrack::FatiguePenalty)
        .copied()
        .unwrap_or(0);
    // (id, base penalty). Fresh/Winded are penalty-free; Unconscious is its own
    // penalty (no numeric). ArMDE:17127-17129 gives Weary −1, Tired −3, Dazed −5.
    [
        (FatigueTier::Fresh, 0),
        (FatigueTier::Winded, 0),
        (FatigueTier::Weary, -1),
        (FatigueTier::Tired, -3),
        (FatigueTier::Dazed, -5),
    ]
    .into_iter()
    .map(|(level, base)| FatigueLevel {
        level,
        // A positive delta reduces magnitude; never flip a penalty positive.
        penalty: (base + delta).min(0),
    })
    .collect()
}

/// The five wound bands, in ascending severity. A fixed rules taxonomy, rendered
/// via Fluent, never as a raw slug. Source: ArMDE:17167-17191.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WoundBand {
    /// Light wound: −1 per wound.
    Light,
    /// Medium wound: −3 per wound.
    Medium,
    /// Heavy wound: −5 per wound.
    Heavy,
    /// Incapacitating wound (special; no numeric per-wound penalty).
    Incapacitating,
    /// Dead (special; open-ended top band).
    Dead,
}

impl std::fmt::Display for WoundBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            WoundBand::Light => "light",
            WoundBand::Medium => "medium",
            WoundBand::Heavy => "heavy",
            WoundBand::Incapacitating => "incapacitating",
            WoundBand::Dead => "dead",
        })
    }
}

/// One wound band's inclusive damage range and its per-wound penalty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WoundRange {
    /// The wound band; serializes to its stable slug (`"light"`, `"medium"`,
    /// `"heavy"`, `"incapacitating"`, `"dead"`), mapped through Fluent.
    pub level: WoundBand,
    /// Lowest damage-total value in this band.
    pub min: i32,
    /// Highest damage-total value in this band; `None` for the open-ended Dead band.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
    /// The per-wound penalty (≤ 0); `None` for Incapacitating / Dead (special).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub penalty: Option<i32>,
}

/// The Size-indexed wound ranges, with wound penalties adjusted by any HealthMod
/// wound delta. The band unit is `u = max(1, Size + 5)`; Light 1..u, Medium
/// u+1..2u, Heavy 2u+1..3u, Incapacitating 3u+1..4u, Dead 4u+1.. — so each +1 Size
/// widens every band (ArMDE:17167-17191). Uses the character's derived Size.
pub fn wound_ranges(entity: &Entity, ruleset: &Ruleset) -> Vec<WoundRange> {
    let mods = in_play_mods(entity, ruleset);
    let delta = mods
        .health_mods
        .get(&HealthTrack::WoundPenalty)
        .copied()
        .unwrap_or(0);
    let size = crate::effective::size(entity, ruleset);
    let u = (size + 5).max(1);
    let pen = |base: i32| (base + delta).min(0);
    vec![
        WoundRange {
            level: WoundBand::Light,
            min: 1,
            max: Some(u),
            penalty: Some(pen(-1)),
        },
        WoundRange {
            level: WoundBand::Medium,
            min: u + 1,
            max: Some(2 * u),
            penalty: Some(pen(-3)),
        },
        WoundRange {
            level: WoundBand::Heavy,
            min: 2 * u + 1,
            max: Some(3 * u),
            penalty: Some(pen(-5)),
        },
        WoundRange {
            level: WoundBand::Incapacitating,
            min: 3 * u + 1,
            max: Some(4 * u),
            penalty: None,
        },
        WoundRange {
            level: WoundBand::Dead,
            min: 4 * u + 1,
            max: None,
            penalty: None,
        },
    ]
}
