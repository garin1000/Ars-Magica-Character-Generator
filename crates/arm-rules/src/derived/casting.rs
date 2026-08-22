//! Casting Totals, Penetration, and Magic Resistance — the spellcasting-facing
//! play-stat totals (Wave 6, `derived.rs` module split; extracted verbatim
//! from `derived.rs`, no logic changed).
//!
//! Needs [`super::combat::encumbrance`] (a Casting Score subtracts
//! Encumbrance) via an explicit cross-module `use`, since `encumbrance` moved
//! into the sibling `combat` module — it was already `pub fn`, so this is a
//! plain import, no visibility change.

use super::combat::encumbrance;
use super::*;

// --- Casting totals (per Technique × Form) ---------------------------------

/// The within-focus counterparts of a [`CastingTotal`]'s four cast types (the
/// lower applicable Art added again before halving).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastingWithinFocus {
    /// The doubled lower Art score added within the focus.
    pub focus_art: i32,
    /// Within-focus formulaic total.
    pub formulaic: i32,
    /// Within-focus ritual total.
    pub ritual: i32,
    /// Within-focus fatiguing-spontaneous total.
    pub spontaneous_fatiguing: i32,
    /// Within-focus non-fatiguing-spontaneous total.
    pub spontaneous_non_fatiguing: i32,
}

/// The non-standard-casting variants of a cell's **Formulaic** Casting Total:
/// casting with no voice ("silent") and/or no gestures ("still"). The Words and
/// Gestures penalties apply to Formulaic and Spontaneous casting, never to Ritual
/// (Ars Magica - Definitive Edition (Core Rules).md:9236); these variants adjust the Formulaic total. Quiet Magic reduces the
/// no-voice penalty, Subtle Magic the no-gesture penalty, and Deft Form waives
/// both for spells in its Form; each residual penalty clamps at 0. Source:
/// Ars Magica - Definitive Edition (Core Rules).md:9236-9245, :4822-4826, :5073-5076, :3645-3648.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonStandardCasting {
    /// Residual no-voice penalty (≤ 0) after Quiet Magic / Deft Form.
    pub voice_penalty: i32,
    /// Residual no-gesture penalty (≤ 0) after Subtle Magic / Deft Form.
    pub gesture_penalty: i32,
    /// Formulaic total cast with no voice: `formulaic + voice_penalty`.
    pub silent: i32,
    /// Formulaic total cast with no gestures: `formulaic + gesture_penalty`.
    pub still: i32,
    /// Formulaic total cast with neither voice nor gestures.
    pub silent_and_still: i32,
    /// Whether Deft Form applies to this cell's Form (both penalties waived).
    pub deft_form: bool,
}

/// The Casting Total for one `(Technique, Form)` cell, split into the four cast
/// types from one shared breakdown.
///
/// Casting Score = Technique + Form + Stamina − Encumbrance + Aura + flat
/// CastingTotalMod (per scope). Formulaic = the score; Ritual = the score +
/// Artes Liberales + Philosophiae; fatiguing Spontaneous = ÷2; non-fatiguing
/// Spontaneous = ÷5. Within a Magical Focus the lower Art is added again; a
/// Deficient Art halves the totals. Source: Ars Magica - Definitive Edition
/// (Core Rules).md:9089 (Casting Score), :9103-9145
/// (cast types), :4524-4527 (Method Caster), :4399-4422 (focus), :5909-5915
/// (Deficient halving).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastingTotal {
    /// Technique Art id of this cell.
    pub technique: Id,
    /// Form Art id of this cell.
    pub form: Id,
    /// The shared Casting-Score breakdown (technique, form, stamina, encumbrance, aura).
    pub addends: Vec<Addend>,
    /// The ritual-only extra addends (artes_liberales, philosophiae).
    pub ritual_addends: Vec<Addend>,
    /// Formulaic Casting Total.
    pub formulaic: i32,
    /// Ritual Casting Total (+ Artes Liberales + Philosophiae).
    pub ritual: i32,
    /// Fatiguing spontaneous total (÷2).
    pub spontaneous_fatiguing: i32,
    /// Non-fatiguing spontaneous total (÷5).
    pub spontaneous_non_fatiguing: i32,
    /// The within-focus variants; `None` when the magus holds no Magical Focus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_focus: Option<CastingWithinFocus>,
    /// The non-standard-casting (silent / still) variants of the Formulaic total.
    pub non_standard: NonStandardCasting,
    /// Whether a Deficient Art halved these totals.
    pub deficient: bool,
}

/// The formulaic casting score of one `(Technique, Form)` cell, without the die,
/// including the focus double when `focus` is set and the Deficient-Art halving.
/// Shared by the grid and by per-spell penetration.
fn formulaic_casting_score(
    entity: &Entity,
    ruleset: &Ruleset,
    mods: &InPlayMods,
    technique: &Id,
    form: &Id,
    focus: bool,
) -> i32 {
    let te = effective_art_score(entity, ruleset, technique);
    let fo = effective_art_score(entity, ruleset, form);
    let stamina = characteristic(entity, ruleset, Characteristic::Sta);
    let enc = encumbrance(entity, ruleset).total;
    let mut score = saturating_i32_sum([
        te,
        fo,
        stamina,
        -enc,
        entity.aura,
        mods.casting_mod_for(CastType::Formulaic),
    ]);
    if focus {
        score = saturating_i32_sum([score, te.min(fo)]);
    }
    if mods.deficient(technique, form) {
        score = halve(score);
    }
    score
}

/// Casting Totals for every `(Technique, Form)` pair. Source: Ars Magica -
/// Definitive Edition (Core Rules).md:9089-9145.
pub fn casting_totals(entity: &Entity, ruleset: &Ruleset) -> Vec<CastingTotal> {
    let mods = in_play_mods(entity, ruleset);
    let stamina = characteristic(entity, ruleset, Characteristic::Sta);
    let enc = encumbrance(entity, ruleset).total;
    let aura = entity.aura;
    let artes_liberales = ability(entity, ruleset, ID_ARTES_LIBERALES);
    let philosophiae = ability(entity, ruleset, ID_PHILOSOPHIAE);
    let weak_spont = mods.halvings.contains(&HalvableTotal::SpontaneousCasting);
    let mut out = Vec::new();
    for technique in ruleset.art_ids_of(ArtType::Technique) {
        let te = effective_art_score(entity, ruleset, &technique);
        for form in ruleset.art_ids_of(ArtType::Form) {
            let fo = effective_art_score(entity, ruleset, &form);
            let deficient = mods.deficient(&technique, &form);
            let addends = vec![
                Addend::new("technique", te),
                Addend::new("form", fo),
                Addend::new("stamina", stamina),
                Addend::new("encumbrance", -enc),
                Addend::new("aura", aura),
            ];
            let ritual_addends = vec![
                Addend::new("artes_liberales", artes_liberales),
                Addend::new("philosophiae", philosophiae),
            ];
            let common = sum(&addends);
            let focus_art = te.min(fo);

            let variant = |focused: bool| -> CastingScores {
                let focus_add = if focused { focus_art } else { 0 };
                let formulaic = post(
                    common + focus_add + mods.casting_mod_for(CastType::Formulaic),
                    deficient,
                    false,
                );
                let ritual = post(
                    common
                        + focus_add
                        + sum(&ritual_addends)
                        + mods.casting_mod_for(CastType::Ritual),
                    deficient,
                    false,
                );
                let spont_base = post(
                    common + focus_add + mods.casting_mod_for(CastType::Spontaneous),
                    deficient,
                    weak_spont,
                );
                CastingScores {
                    formulaic,
                    ritual,
                    spontaneous_fatiguing: halve(spont_base),
                    spontaneous_non_fatiguing: spont_base / 5,
                }
            };

            let base = variant(false);
            let voice_penalty = mods.residual_voice_penalty(&form);
            let gesture_penalty = mods.residual_gesture_penalty(&form);
            let non_standard = NonStandardCasting {
                voice_penalty,
                gesture_penalty,
                silent: base.formulaic + voice_penalty,
                still: base.formulaic + gesture_penalty,
                silent_and_still: base.formulaic + voice_penalty + gesture_penalty,
                deft_form: mods.deft_forms.contains(&form),
            };
            let within_focus = mods.has_focus.then(|| {
                let f = variant(true);
                CastingWithinFocus {
                    focus_art,
                    formulaic: f.formulaic,
                    ritual: f.ritual,
                    spontaneous_fatiguing: f.spontaneous_fatiguing,
                    spontaneous_non_fatiguing: f.spontaneous_non_fatiguing,
                }
            });

            out.push(CastingTotal {
                technique: technique.clone(),
                form: form.clone(),
                addends,
                ritual_addends,
                formulaic: base.formulaic,
                ritual: base.ritual,
                spontaneous_fatiguing: base.spontaneous_fatiguing,
                spontaneous_non_fatiguing: base.spontaneous_non_fatiguing,
                within_focus,
                non_standard,
                deficient,
            });
        }
    }
    out
}

/// Applies the Deficient-Art halving and (for spontaneous) the Weak-Spontaneous
/// halving to a casting score, in that order. Source: Ars Magica - Definitive
/// Edition (Core Rules).md:5909-5915, :7060-7063.
fn post(score: i32, deficient: bool, weak_spont: bool) -> i32 {
    let mut s = score;
    if deficient {
        s = halve(s);
    }
    if weak_spont {
        s = halve(s);
    }
    s
}

struct CastingScores {
    formulaic: i32,
    ritual: i32,
    spontaneous_fatiguing: i32,
    spontaneous_non_fatiguing: i32,
}

// --- Penetration (per known spell) -----------------------------------------

/// A Penetration line for one known spell.
///
/// Penetration Total = Casting Total − Spell Level + Penetration Ability score
/// (Ars Magica - Definitive Edition (Core Rules).md:9159-9161). Weak Magic
/// halves the Penetration Total *after* subtracting the level
/// (Ars Magica - Definitive Edition (Core Rules).md:7064-7067).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PenetrationLine {
    /// The known spell's id.
    pub spell: Id,
    /// The chosen parameter of a parameterized spell (the target `(Form)` of a
    /// meta-magic Vim spell), disambiguating two instances of one spell id that
    /// differ only by parameter. `None` for ordinary, unparameterized spells.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// The spell's (resolved) level.
    pub level: u32,
    /// The formulaic Casting Total used (base, no focus).
    pub casting_total: i32,
    /// The Penetration Ability score contributing to the bonus.
    pub penetration_ability: i32,
    /// The Penetration Total (base, no focus).
    pub total: i32,
    /// The within-focus Penetration Total; `None` when no Magical Focus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_focus: Option<i32>,
    /// Whether Weak Magic halved the total.
    pub weak_magic: bool,
}

/// Per-known-spell Penetration Totals. Source: Ars Magica - Definitive Edition
/// (Core Rules).md:9159-9161, :7064-7067.
pub fn penetration(entity: &Entity, ruleset: &Ruleset) -> Vec<PenetrationLine> {
    let mods = in_play_mods(entity, ruleset);
    let pen_ability = ability(entity, ruleset, ID_PENETRATION);
    let weak_magic = mods.halvings.contains(&HalvableTotal::Penetration);
    let mut out = Vec::new();
    for sel in &entity.spells {
        let Some(spell) = ruleset.spell(&sel.spell) else {
            continue;
        };
        let Some(level) = resolved_spell_level(sel, ruleset) else {
            continue;
        };
        let level_i = i32::try_from(level).unwrap_or(i32::MAX);
        let pen = |casting: i32| -> i32 {
            let raw = saturating_i32_sum([casting, -level_i, pen_ability]);
            if weak_magic { halve(raw) } else { raw }
        };
        let base_casting =
            formulaic_casting_score(entity, ruleset, &mods, &spell.technique, &spell.form, false);
        let within_focus = mods.has_focus.then(|| {
            let focus_casting = formulaic_casting_score(
                entity,
                ruleset,
                &mods,
                &spell.technique,
                &spell.form,
                true,
            );
            pen(focus_casting)
        });
        out.push(PenetrationLine {
            spell: sel.spell.clone(),
            parameter: sel.parameter.clone(),
            level,
            casting_total: base_casting,
            penetration_ability: pen_ability,
            total: pen(base_casting),
            within_focus,
            weak_magic,
        });
    }
    out
}

// --- Magic Resistance (per Form) -------------------------------------------

/// A per-Form Magic Resistance line.
///
/// A magus's Magic Resistance = Form + 5 × Parma Magica (Ars Magica -
/// Definitive Edition (Core Rules).md:9390-9398). A supernatural being uses
/// its **Might Score** as a blanket resistance instead of Parma — the two do
/// not stack; the higher is the base (Ars Magica 5e - Realms of Power -
/// Magic.md:1472; Ars Magica - Definitive Edition (Core Rules).md:2627), and
/// the Form bonus is compatible with either. Limited Magic Resistance drops
/// the Form bonus; Flawed Parma / Weak Magic Resistance halve it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MagicResistance {
    /// The Form Art id.
    pub form: Id,
    /// The labelled breakdown (form, and one of parma/might).
    pub addends: Vec<Addend>,
    /// The Magic Resistance total.
    pub total: i32,
}

/// Per-Form Magic Resistance. Source: Ars Magica - Definitive Edition (Core
/// Rules).md:9390-9398, :6142-6145, :6346-6349;
/// Ars Magica 5e - Realms of Power - Magic.md:1472 (Might grants MR = Might
/// Score, not stacking with Parma).
pub fn magic_resistance(entity: &Entity, ruleset: &Ruleset) -> Vec<MagicResistance> {
    let mods = in_play_mods(entity, ruleset);
    let parma = ability(entity, ruleset, ID_PARMA_MAGICA);
    let parma_mr = 5 * parma;
    // A Might-being's blanket resistance = its effective Might Score. Might and
    // Parma do not stack; the higher is the base (Ars Magica 5e - Realms of Power - Magic.md:1472, Ars Magica - Definitive Edition (Core Rules).md:2627).
    let might = crate::effective::effective_might(entity, ruleset)
        .map(|m| i32::from(m.score))
        .unwrap_or(0);
    let no_form = mods.mr_mods.contains(&MagicResistanceEffect::NoFormBonus);
    let halved = mods.halvings.contains(&HalvableTotal::MagicResistance);
    let mut out = Vec::new();
    for form in ruleset.art_ids_of(ArtType::Form) {
        let fo = effective_art_score(entity, ruleset, &form);
        let form_bonus = if no_form { 0 } else { fo };
        let base_addend = if might > parma_mr {
            Addend::new("might", might)
        } else {
            Addend::new("parma", parma_mr)
        };
        let addends = vec![Addend::new("form", form_bonus), base_addend];
        let mut total = sum(&addends);
        if halved {
            total = halve(total);
        }
        out.push(MagicResistance {
            form,
            addends,
            total,
        });
    }
    out
}
