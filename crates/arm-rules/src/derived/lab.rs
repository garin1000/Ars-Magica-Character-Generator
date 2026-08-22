//! Lab Totals, the Longevity Ritual read-out, and the Masterpiece lesser-item
//! cap (Wave 6, `derived.rs` module split; extracted verbatim from
//! `derived.rs`, no logic changed).

use super::*;

// --- Lab totals (per Technique × Form) -------------------------------------

/// A Lab Total for one `(Technique, Form)` cell of the 5×10 grid.
///
/// Lab Total = Int + Magic Theory + Technique + Form + Aura + flat LabTotalMod;
/// within a Magical Focus the lower applicable Art is added again; a Deficient Art
/// halves the whole cell. Source: Ars Magica - Definitive Edition (Core
/// Rules).md:10276-10278 (Lab Total shape), :4151-4154
/// (Inventive Genius, the flat `LabTotalMod`), :4399-4422 (focus doubling),
/// :5909-5915 (Deficient halving).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabTotal {
    /// Technique Art id of this cell.
    pub technique: Id,
    /// Form Art id of this cell.
    pub form: Id,
    /// The labelled base breakdown (int, magic_theory, technique, form, aura, lab_mod).
    pub addends: Vec<Addend>,
    /// The base Lab Total (after any Deficient-Art halving), no focus.
    pub total: i32,
    /// The within-focus Lab Total (base + lower Art, then halving); `None` when the
    /// magus holds no Magical Focus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_focus: Option<i32>,
    /// Whether a Deficient Art halved this cell.
    pub deficient: bool,
    /// `total`, halved again for Weak Enchanter (Ars Magica - Definitive
    /// Edition (Core Rules).md:7060-7063: "Halve your Lab Total whenever you
    /// create or investigate an enchanted item. If you have a Deficiency that
    /// counts as part of the Lab Total, apply the Deficiency first and then
    /// halve the remaining total"). Equal to `total` for anyone without the
    /// Flaw. This is the figure to use when creating or investigating an
    /// enchanted item; `total`/`within_focus` remain the ordinary Lab Total
    /// for every other lab activity (spell invention, etc.), which Weak
    /// Enchanter does not touch.
    pub enchanting: i32,
}

/// Lab Totals for every `(Technique, Form)` pair — the 5×10 grid. Source:
/// Ars Magica - Definitive Edition (Core Rules).md:10276-10278.
pub fn lab_totals(entity: &Entity, ruleset: &Ruleset) -> Vec<LabTotal> {
    let mods = in_play_mods(entity, ruleset);
    let intelligence = characteristic(entity, ruleset, Characteristic::Int);
    let magic_theory = ability(entity, ruleset, ID_MAGIC_THEORY);
    let aura = entity.aura;
    let mut out = Vec::new();
    for technique in ruleset.art_ids_of(ArtType::Technique) {
        let te = effective_art_score(entity, ruleset, &technique);
        for form in ruleset.art_ids_of(ArtType::Form) {
            let fo = effective_art_score(entity, ruleset, &form);
            let addends = vec![
                Addend::new("intelligence", intelligence),
                Addend::new("magic_theory", magic_theory),
                Addend::new("technique", te),
                Addend::new("form", fo),
                Addend::new("aura", aura),
                Addend::new("lab_mod", mods.lab_mod),
            ];
            let deficient = mods.deficient(&technique, &form);
            let base = sum(&addends);
            let total = if deficient { halve(base) } else { base };
            let within_focus = mods.has_focus.then(|| {
                let focused = base + te.min(fo);
                if deficient { halve(focused) } else { focused }
            });
            // Weak Enchanter: Deficiency (already folded into `total`) first,
            // then this halving on top — the order the Flaw's text specifies.
            let enchanting = if mods.halvings.contains(&HalvableTotal::LabEnchanting) {
                halve(total)
            } else {
                total
            };
            out.push(LabTotal {
                technique: technique.clone(),
                form: form.clone(),
                addends,
                total,
                within_focus,
                deficient,
                enchanting,
            });
        }
    }
    out
}

// --- Longevity -------------------------------------------------------------

/// What a Longevity Ritual made *today* would be worth — a suggestion, never the
/// stored value.
///
/// "+1 bonus for every five points or fraction of Creo Corpus Lab Total"
/// (Ars Magica - Definitive Edition (Core Rules).md:10662). Shown beside the entered-bonus input so a player who is creating
/// the ritual now (or reinventing it after an aging crisis, Ars Magica - Definitive Edition (Core Rules).md:10668, :10670) can
/// read off the number the rules give them. It moves whenever Creo, Corpus,
/// Intelligence, Magic Theory or the aura move — which is exactly why the *stored*
/// bonus must not be derived from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongevityHint {
    /// Today's Creo Corpus Lab Total, after any halvings.
    pub lab_total: i32,
    /// The bonus that Lab Total would buy: `ceil(lab_total / 5)`, floored at 0.
    pub suggested_bonus: i32,
    /// Whether a Deficient Art or Difficult Longevity Ritual halved `lab_total`.
    pub halved: bool,
}

/// The Longevity Ritual aging bonus read-out.
///
/// `bonus` is what the player entered, for **both** sources — the ritual is a past
/// event whose bonus was fixed by the Lab Total of the season it was made
/// (Ars Magica - Definitive Edition (Core Rules).md:10662, :10670), so nothing here is derived. `entered` distinguishes an
/// unfilled field from a deliberate 0. `hint` carries the live suggestion for a
/// self-made ritual only. The Bronze cord adds "to rolls to resist aging"
/// (Ars Magica - Definitive Edition (Core Rules).md:10844) and is noted separately, since it is not part of the ritual; it goes
/// through [`bronze_cord_bonus`], so it can never exceed the +5 maximum (Ars Magica - Definitive Edition (Core Rules).md:10836)
/// or disagree with the Soak and cord-cost read-outs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongevityBonus {
    /// Whether the ritual is self-made or external.
    pub source: LongevitySource,
    /// The stored aging bonus (magnitude; applied as a negative to aging rolls).
    pub bonus: i32,
    /// Whether a bonus was actually entered; `false` ⇒ `bonus` is a placeholder 0.
    pub entered: bool,
    /// The Bronze-cord bonus, noted here and **not** summed into `bonus`.
    ///
    /// The cord applies "to rolls to resist aging" (Ars Magica - Definitive Edition (Core Rules).md:10844), and the roll that
    /// referent names is the **crisis survival** roll — an aging roll itself is not
    /// passed or failed, and Ars Magica - Definitive Edition (Core Rules).md:16636 keeps the two roll families apart. So this
    /// line is informational on the ritual panel; the cord reaches a total through
    /// [`bronze_cord_bonus`] on the crisis-survival read-out, not here.
    pub bronze_cord: i32,
    /// What a ritual made today would be worth; `None` for an external ritual.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<LongevityHint>,
}

/// The Longevity Ritual read-out, or `None` when the magus has no ritual. Source:
/// Ars Magica - Definitive Edition (Core Rules).md:10662 (formula), :10668 + :10670 (the bonus is fixed at creation and only a
/// reinvention takes advantage of raised Arts), :10844 (Bronze cord).
pub fn longevity_bonus(entity: &Entity, ruleset: &Ruleset) -> Option<LongevityBonus> {
    let ritual = entity.longevity_ritual.as_ref()?;
    let bronze = bronze_cord_bonus(entity);
    // A hint only makes sense for a ritual this magus makes: an external one came
    // from another magus's Lab Total, which this sheet does not know (Ars Magica - Definitive Edition (Core Rules).md:10672).
    let hint = match ritual.source {
        LongevitySource::SelfMade => {
            let (lab_total, halved) = creo_corpus_lab_total(entity, ruleset);
            Some(LongevityHint {
                lab_total,
                suggested_bonus: suggested_longevity_bonus(lab_total),
                halved,
            })
        }
        LongevitySource::External => None,
    };
    Some(LongevityBonus {
        source: ritual.source,
        bonus: i32::from(ritual.bonus.unwrap_or(0)),
        entered: ritual.bonus.is_some(),
        bronze_cord: bronze,
        hint,
    })
}

/// The Creo Corpus Lab Total and whether it was halved.
///
/// "Your basic Lab Total is: Technique + Form + Intelligence + Magic Theory + Aura
/// Modifier" (Ars Magica - Definitive Edition (Core Rules).md:10276-10278) plus any flat Lab-Total modifier. The Aura Modifier
/// is a plain addend with no floor and no gate: a zero aura is simply "the absence
/// of aura, so powers used there function without hindrance" (Ars Magica - Definitive Edition (Core Rules).md:17658).
///
/// Two halvings can apply. A Deficient Creo or Corpus halves "almost all totals
/// (including … Lab Totals) to which a particular Form is added" (Ars Magica - Definitive Edition (Core Rules).md:5909-5915),
/// and Difficult Longevity Ritual makes anyone "creating a Longevity Ritual for you
/// … halve their Lab Total" (Ars Magica - Definitive Edition (Core Rules).md:5962-5964). **That the two compound is an
/// inference**: each Flaw halves the Lab Total and neither carves out the other, but
/// no passage states the interaction. The order is immaterial — [`halve`] truncates
/// toward zero — so it is fixed here as base → Deficient → Difficult.
///
/// **Sibling formula:** [`lab_totals`] builds the same addend list for every
/// `(Technique, Form)` cell. This is deliberately not that grid's Creo/Corpus cell:
/// the whole 5 × 10 grid would be built to answer a one-cell question, and the two
/// figures legitimately differ — no focus figure applies to a Longevity Ritual, and
/// the `LabLongevity` halving applies to nothing else. Keep the shared addend list
/// (Int + Magic Theory + Technique + Form + aura + `lab_mod`) in step across both.
fn creo_corpus_lab_total(entity: &Entity, ruleset: &Ruleset) -> (i32, bool) {
    let mods = in_play_mods(entity, ruleset);
    let base = saturating_i32_sum([
        characteristic(entity, ruleset, Characteristic::Int),
        ability(entity, ruleset, ID_MAGIC_THEORY),
        art(entity, ruleset, ID_CREO),
        art(entity, ruleset, ID_CORPUS),
        entity.aura,
        mods.lab_mod,
    ]);
    let deficient = mods.deficient(&Id::new(ID_CREO), &Id::new(ID_CORPUS));
    let difficult = mods.halvings.contains(&HalvableTotal::LabLongevity);
    let mut total = base;
    if deficient {
        total = halve(total);
    }
    if difficult {
        total = halve(total);
    }
    (total, deficient || difficult)
}

/// The bonus a Creo Corpus Lab Total buys: "+1 bonus for every five points or
/// fraction" (Ars Magica - Definitive Edition (Core Rules).md:10662), i.e. `ceil(lab_total / 5)`. A non-positive Lab Total buys
/// nothing — there is no fraction of five points below one point.
fn suggested_longevity_bonus(lab_total: i32) -> i32 {
    if lab_total <= 0 {
        return 0;
    }
    // `i32::div_ceil` is still unstable, so round up in unsigned space.
    i32::try_from((lab_total as u32).div_ceil(5)).unwrap_or(i32::MAX)
}

// --- Masterpiece (lesser enchanted item cap) -------------------------------

/// The Masterpiece read-out: the best-Lab-Total lesser enchanted item cap.
///
/// The Masterpiece Virtue lets the magus keep one *lesser enchanted item* he
/// designed "based on his Lab Totals at character generation, following the
/// regular rules for construction of such a device" (Ars Magica - Definitive
/// Edition (Core Rules).md:4476-4479). The
/// regular lesser-enchantment rule caps a single-season instillation at
/// `Lab Total ≥ 2 × effect level`, i.e. the effect level may not exceed
/// `Lab Total ÷ 2` (Ars Magica - Definitive Edition (Core Rules).md:10410). Vis costs are ignored (the parens provided
/// them), so the only bound the engine can honestly compute is that Lab-Total
/// cap. The best base `(Technique, Form)` Lab Total is used — the magus is free
/// to pick the Technique/Form that maximises it — without any Magical-Focus
/// doubling (a focus applies only to items within its narrow field).
///
/// This is **read-only guidance**: the engine does not auto-create a device or
/// spend an item-level budget. The player still enters the actual item under
/// Magic Items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasterpieceCap {
    /// The Technique Art of the best Lab Total.
    pub technique: Id,
    /// The Form Art of the best Lab Total.
    pub form: Id,
    /// The best base `(Technique, Form)` Lab Total (no focus doubling).
    pub lab_total: i32,
    /// The maximum lesser-enchantment effect level: `lab_total ÷ 2` (Ars Magica -
    /// Definitive Edition (Core Rules).md:10410).
    pub cap: i32,
}

/// The Masterpiece lesser-item cap, or `None` when the magus lacks the Virtue.
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4476-4479 (Virtue),
/// :10410 (lesser-enchantment cap).
pub fn masterpiece_item_cap(entity: &Entity, ruleset: &Ruleset) -> Option<MasterpieceCap> {
    if !in_play_mods(entity, ruleset).has_masterpiece {
        return None;
    }
    // Best base Lab Total across the grid; the magus picks the Te/Fo that maxes it.
    lab_totals(entity, ruleset)
        .into_iter()
        .max_by_key(|lt| lt.total)
        .map(|lt| MasterpieceCap {
            technique: lt.technique,
            form: lt.form,
            lab_total: lt.total,
            cap: halve(lt.total),
        })
}
