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

/// The within-Potent-Magic-field counterparts of a [`CastingTotal`]'s four cast
/// types (D79, `docs/vf-audit/decisions.md`). Unlike [`CastingWithinFocus`],
/// there is no doubled Art — Potent Magic is a flat +6/+3 bonus, not a
/// doubling — so this struct carries no `focus_art`-equivalent field. Gated
/// on holding any Potent Magic Virtue (`InPlayMods::has_potent_magic`),
/// independent of [`CastingWithinFocus`]'s `has_focus` gate — a spell may be
/// within a Magical Focus, a Potent Magic field, both, or neither.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastingWithinPotentField {
    /// Within-potent-field formulaic total.
    pub formulaic: i32,
    /// Within-potent-field ritual total.
    pub ritual: i32,
    /// Within-potent-field fatiguing-spontaneous total.
    pub spontaneous_fatiguing: i32,
    /// Within-potent-field non-fatiguing-spontaneous total.
    pub spontaneous_non_fatiguing: i32,
}

/// The non-standard-casting variants of a cell's **Formulaic** Casting Total:
/// casting with no voice ("silent") and/or no gestures ("still"). The Words and
/// Gestures penalties apply to Formulaic and Spontaneous casting, never to Ritual
/// (ArMDE:9236); these variants adjust the Formulaic total. Quiet Magic reduces the
/// no-voice penalty, Subtle Magic the no-gesture penalty, and Deft Form waives
/// both for spells in its Form; each residual penalty clamps at 0. Source:
/// ArMDE:9236-9245, :4822-4826, :5073-5076, :3645-3648.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonStandardCasting {
    /// Residual no-voice penalty (≤ 0) after Quiet Magic / Deft Form.
    pub voice_penalty: i32,
    /// Residual no-gesture penalty (≤ 0) after Subtle Magic / Deft Form.
    pub gesture_penalty: i32,
    /// Formulaic total cast with no voice. The penalty is a **Casting-Score**
    /// term (ArMDE:9236), so it is summed with the others *before* any
    /// Deficient-Art halving: `halve(score + voice_penalty)`, which for a
    /// non-deficient magus is just `formulaic + voice_penalty`.
    pub silent: i32,
    /// Formulaic total cast with no gestures — same ordering as [`Self::silent`].
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
/// Deficient Art halves the totals. Source: ArMDE:9089 (Casting Score), :9103-9145
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
    /// The flat `CastingTotalMod` reaching each cast type, one labelled addend
    /// per scope (`casting_mod_formulaic`, `casting_mod_ritual`,
    /// `casting_mod_spontaneous`).
    ///
    /// Separate from [`Self::addends`] rather than folded into it because the
    /// modifier is **per scope** while `addends` is the one breakdown shared by
    /// all four cast types: Method Caster's +3 reaches Formulaic and Ritual but
    /// not Spontaneous, so a single shared `casting_mod` addend would be wrong
    /// for one column whichever value it carried. Same split as
    /// [`Self::ritual_addends`], for the same reason.
    ///
    /// Each cast type's figure is therefore `sum(addends)` + its own entry here
    /// (+ `ritual_addends` for Ritual), *before* the Magical-Focus double and
    /// the Deficient-Art halving, which are transforms of that sum rather than
    /// addends. Pinned by
    /// `derived.rs::every_shipped_breakdown_accounts_for_the_total_it_explains`.
    pub casting_mod_addends: Vec<Addend>,
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
    /// The within-Potent-Magic-field variants; `None` when the magus holds no
    /// Potent Magic Virtue. Independent of [`Self::within_focus`] — gates the
    /// UI's second per-spell toggle (D79) rather than sharing the Magical
    /// Focus one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_potent_field: Option<CastingWithinPotentField>,
    /// The non-standard-casting (silent / still) variants of the Formulaic total.
    pub non_standard: NonStandardCasting,
    /// Whether a Deficient Art halved these totals.
    pub deficient: bool,
    /// Whether this cell's `(Technique, Form)` pair is one of a held Incompatible
    /// Arts Flaw's two combinations (D81.8, `docs/vf-audit/decisions.md`;
    /// ArMDE:6290-6292). A requisite spell may still be unusable through this
    /// cell without the cell itself being flagged — see
    /// `effective/spell.rs::spell_touches_barred_combination`, which the
    /// per-spell figure reads instead.
    pub unusable: bool,
}

/// The addends of a Casting Total that belong to the **character** rather than to
/// a `(Technique, Form)` cell or a spell, so they are computed once per read-out
/// and carried down. Deriving them per cell made `penetration` re-walk the whole
/// equipment list once or twice for every known spell to arrive at the same two
/// numbers.
#[derive(Clone, Copy)]
struct CastingBase {
    stamina: i32,
    /// The Encumbrance **total**, subtracted (so it is stored positive).
    encumbrance: i32,
}

impl CastingBase {
    fn of(entity: &Entity, ruleset: &Ruleset) -> Self {
        Self {
            stamina: characteristic(entity, ruleset, Characteristic::Sta),
            encumbrance: encumbrance(entity, ruleset).total,
        }
    }
}

/// The lesser-of-requisite-and-primary fold for one Art class (ArMDE:12309,
/// `### Requisites`): the effective score used is the lowest of `primary_score`
/// and every requisite in `requisites` whose own `ArtType` matches `art_type` —
/// a requisite of the other class (e.g. a Technique requisite, while folding
/// the Form side) is skipped, and several requisites of the same class fold to
/// the lowest of the whole group (ArMDE:12311). Every side is read through
/// [`effective_art_score`], so a Puissant Art bonus is included before the
/// comparison on whichever side carries it (ArMDE:4820) — no separate
/// case needed, since "use the lesser of the two bonus-inclusive scores"
/// already produces exactly that rule's worked example.
///
/// `elemental_forms` is `Some` only for an entity holding Elemental Magic
/// (`InPlayMods::elemental_forms`). When both `primary_id` and a candidate
/// requisite are members of that set, the requisite is skipped outright
/// (ArMDE:3737: "you use the primary Form to calculate totals, even if the
/// requisite is lower") — Elemental Magic pools only Forms, so this can only
/// ever matter when `art_type` is [`ArtType::Form`]; a Technique is never a
/// member of `elemental_forms`, so the check is simply never true for the
/// Technique side.
///
/// Deliberately does **not** reach Magic Resistance, vis-boosting, or any
/// other "effects that affect spells based on their Arts" (ArMDE:12313) —
/// those rely on the primary Arts only and call neither this function nor
/// [`formulaic_casting_score`].
///
/// `pub(crate)`: `effective/spell.rs::spell_level_cap` (X11b) reuses this
/// same fold for the per-spell creation-time level cap — `ArMDE:2465`'s own
/// closing sentence calls that cap "the appropriate Lab Total", so the
/// identical lesser-of-requisite-and-primary rule governs it too. One fold,
/// shared by both call sites, rather than a second copy that could disagree.
pub(crate) fn fold_requisite(
    entity: &Entity,
    ruleset: &Ruleset,
    art_type: ArtType,
    primary_id: &Id,
    primary_score: i32,
    requisites: &[Id],
    elemental_forms: Option<&BTreeSet<Id>>,
) -> i32 {
    requisites.iter().fold(primary_score, |score, req| {
        let Some(req_art) = ruleset.art(req) else {
            // Referential integrity (`ruleset/integrity.rs::validate_spell_refs`)
            // guarantees every requisite resolves in a loaded ruleset; this arm
            // only protects a hand-edited save against a dangling id.
            return score;
        };
        if req_art.art_type != art_type {
            return score;
        }
        if let Some(forms) = elemental_forms
            && forms.contains(primary_id)
            && forms.contains(req)
        {
            return score;
        }
        score.min(effective_art_score(entity, ruleset, req))
    })
}

/// The within-focus/within-potent-field toggles [`formulaic_casting_score`]
/// applies, bundled into one struct purely to keep that function's parameter
/// count under clippy's `too_many_arguments` threshold now that it also takes
/// `requisites` — `cell`'s two Arts and this pair are otherwise unrelated, so a
/// tuple would read as arbitrary; this groups exactly the two toggles that
/// vary per call site (D79: independent of each other, since a Magical Focus
/// and a Potent Magic field are two free-text themes that need not coincide).
#[derive(Clone, Copy)]
struct CastingVariant {
    /// Doubles the folded lower Art (ArMDE:4403).
    focus: bool,
    /// Adds the Potent Magic field bonus (D79).
    potent: bool,
}

/// The formulaic casting score of one `(Technique, Form)` cell, without the die,
/// including the requisite fold (`requisites`, empty for the grid — a cell has
/// no specific spell to fold against), the focus double when `variant.focus` is
/// set, the Potent Magic bonus when `variant.potent` is set (D79 — independent
/// of focus, since the two free-text themes need not coincide), and the
/// Deficient-Art halving. Shared by per-spell penetration and by
/// [`spell_casting_total`].
fn formulaic_casting_score(
    entity: &Entity,
    ruleset: &Ruleset,
    mods: &InPlayMods,
    cell: (&Id, &Id),
    base: CastingBase,
    requisites: &[Id],
    variant: CastingVariant,
) -> i32 {
    let (technique, form) = cell;
    let te = fold_requisite(
        entity,
        ruleset,
        ArtType::Technique,
        technique,
        effective_art_score(entity, ruleset, technique),
        requisites,
        None,
    );
    let fo = fold_requisite(
        entity,
        ruleset,
        ArtType::Form,
        form,
        effective_art_score(entity, ruleset, form),
        requisites,
        mods.elemental_forms.as_ref(),
    );
    let mut score = saturating_i32_sum([
        te,
        fo,
        base.stamina,
        -base.encumbrance,
        entity.aura,
        mods.casting_mod_for(CastType::Formulaic),
    ]);
    if variant.focus {
        score = saturating_i32_sum([score, te.min(fo)]);
    }
    if variant.potent {
        score = saturating_i32_sum([score, mods.potent_casting_mod_for(CastType::Formulaic)]);
    }
    if mods.deficient(technique, form, requisites) {
        score = halve(score);
    }
    score
}

/// Casting Totals for every `(Technique, Form)` pair. Source: ArMDE:9089-9145.
pub fn casting_totals(entity: &Entity, ruleset: &Ruleset) -> Vec<CastingTotal> {
    let mods = in_play_mods(entity, ruleset);
    let CastingBase {
        stamina,
        encumbrance: enc,
    } = CastingBase::of(entity, ruleset);
    let aura = entity.aura;
    let artes_liberales = ability(entity, ruleset, ID_ARTES_LIBERALES);
    let philosophiae = ability(entity, ruleset, ID_PHILOSOPHIAE);
    let weak_spont = mods.halvings.contains(&HalvableTotal::SpontaneousCasting);
    // D81.8: the Flaw's barred `(Technique, Form)` pairs, computed once for
    // the whole grid rather than per cell.
    let barred = crate::effective::barred_combinations(
        &crate::effective::selections_for_effects(entity, ruleset),
        ruleset,
    );
    let mut out = Vec::new();
    for technique in ruleset.art_ids_of(ArtType::Technique) {
        let te = effective_art_score(entity, ruleset, &technique);
        for form in ruleset.art_ids_of(ArtType::Form) {
            let fo = effective_art_score(entity, ruleset, &form);
            // No specific spell at this grid cell (it may host several, each
            // with different or no requisites), so the fold is requisite-free
            // here — same reasoning as the grid never calling `fold_requisite`
            // at all, only `effective_art_score` directly, above.
            let deficient = mods.deficient(&technique, &form, &[]);
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
            // Read once and used both for the breakdown and for the arithmetic,
            // so the two cannot drift: a breakdown that computes its own copy of
            // a term is a breakdown that can disagree with the total it explains.
            let formulaic_mod = mods.casting_mod_for(CastType::Formulaic);
            let ritual_mod = mods.casting_mod_for(CastType::Ritual);
            let spontaneous_mod = mods.casting_mod_for(CastType::Spontaneous);
            // D79: Potent Magic's bonus, folded SEPARATELY from the three
            // above — it applies only within the maga's own Potent Magic
            // field, never unconditionally, so it must never reach `common`,
            // `within_focus`, or the labelled `casting_mod_addends` breakdown
            // below (which explains only the unconditional figures).
            let potent_formulaic_mod = mods.potent_casting_mod_for(CastType::Formulaic);
            let potent_ritual_mod = mods.potent_casting_mod_for(CastType::Ritual);
            let potent_spontaneous_mod = mods.potent_casting_mod_for(CastType::Spontaneous);
            // One entry per scope, always present — including at 0, exactly as
            // the sibling `lab.rs::lab_totals` always carries its `lab_mod`
            // addend. A breakdown that silently omits a zero term cannot be
            // told apart from one that omits a term it should have had.
            let casting_mod_addends = vec![
                Addend::new("casting_mod_formulaic", formulaic_mod),
                Addend::new("casting_mod_ritual", ritual_mod),
                Addend::new("casting_mod_spontaneous", spontaneous_mod),
            ];
            let common = sum(&addends);
            let focus_art = te.min(fo);

            // `extra` is a further **Casting-Score** term, summed with the rest
            // *before* `post` applies the Deficient-Art halving — which is where
            // every Casting-Score term belongs. The Words/Gestures penalties are
            // its only caller: ArMDE:9236 classifies them as "a penalty to the
            // casting score", and ArMDE:5911 halves the total that score feeds,
            // so `halve(score + penalty)` is the order on either reading.
            // Passing them to `variant` rather than adding them to its result is
            // what keeps that true — see
            // `derived.rs::non_standard_penalties_are_inside_the_deficient_halving`.
            let variant = |focused: bool, potent: bool, extra: i32| -> CastingScores {
                let focus_add = if focused { focus_art } else { 0 };
                // D79: the potent addends are 0 unless `potent` is set — base
                // (`variant(false, false, _)`) and within-focus
                // (`variant(true, false, _)`) never pick these up; only
                // `within_potent_field` (`variant(false, true, 0)`) does.
                let potent_formulaic = if potent { potent_formulaic_mod } else { 0 };
                let potent_ritual = if potent { potent_ritual_mod } else { 0 };
                let potent_spontaneous = if potent { potent_spontaneous_mod } else { 0 };
                // `common` is already saturated by `sum`, so every fold onto it
                // goes through the same helper: a plain `+` aborts the process
                // under `overflow-checks = true`. See
                // `derived.rs::saturating_i32_sum`.
                let formulaic = post(
                    saturating_i32_sum([common, focus_add, formulaic_mod, potent_formulaic, extra]),
                    deficient,
                );
                let ritual = post(
                    saturating_i32_sum([
                        common,
                        focus_add,
                        sum(&ritual_addends),
                        ritual_mod,
                        potent_ritual,
                        extra,
                    ]),
                    deficient,
                );
                let spont_base = post(
                    saturating_i32_sum([
                        common,
                        focus_add,
                        spontaneous_mod,
                        potent_spontaneous,
                        extra,
                    ]),
                    deficient,
                );
                // Rounded down, like every other division the rules leave
                // undirected. Source: ArMDE:547; see `derived.rs::halve`.
                let spontaneous_non_fatiguing = spont_base.div_euclid(5);
                CastingScores {
                    formulaic,
                    ritual,
                    // Weak Spontaneous Magic (ArMDE:7084-7086): "You may not exert yourself
                    // when casting spontaneous magic, so you always divide
                    // your Casting Score by five." This does not add a second
                    // halving on top of the normal ÷2 fatiguing rate (that
                    // would produce ÷4, a rate the rules never state) — it
                    // removes the fatiguing (exert-yourself) option outright,
                    // leaving only the ÷5 rate. `CastingScores` has no "this
                    // option does not exist" representation, so the fatiguing
                    // slot reports the same figure as the non-fatiguing one
                    // rather than a distinct, made-up divisor.
                    spontaneous_fatiguing: if weak_spont {
                        spontaneous_non_fatiguing
                    } else {
                        halve(spont_base)
                    },
                    spontaneous_non_fatiguing,
                }
            };

            let base = variant(false, false, 0);
            let voice_penalty = mods.residual_voice_penalty(&form);
            let gesture_penalty = mods.residual_gesture_penalty(&form);
            // These three penalties are always `<= 0`, so they cannot push a
            // saturated `i32::MAX` higher — but they *can* push a saturated
            // `i32::MIN` lower, which is the same abort at the other end of the
            // range. The sign is not a guard in either direction; the helper is,
            // and `variant` folds them in through the same `saturating_i32_sum`
            // the other Casting-Score terms go through.
            let non_standard = NonStandardCasting {
                voice_penalty,
                gesture_penalty,
                silent: variant(false, false, voice_penalty).formulaic,
                still: variant(false, false, gesture_penalty).formulaic,
                silent_and_still: variant(
                    false,
                    false,
                    saturating_i32_sum([voice_penalty, gesture_penalty]),
                )
                .formulaic,
                deft_form: mods.deft_forms.contains(&form),
            };
            let within_focus = mods.has_focus.then(|| {
                let f = variant(true, false, 0);
                CastingWithinFocus {
                    focus_art,
                    formulaic: f.formulaic,
                    ritual: f.ritual,
                    spontaneous_fatiguing: f.spontaneous_fatiguing,
                    spontaneous_non_fatiguing: f.spontaneous_non_fatiguing,
                }
            });
            let within_potent_field = mods.has_potent_magic.then(|| {
                let p = variant(false, true, 0);
                CastingWithinPotentField {
                    formulaic: p.formulaic,
                    ritual: p.ritual,
                    spontaneous_fatiguing: p.spontaneous_fatiguing,
                    spontaneous_non_fatiguing: p.spontaneous_non_fatiguing,
                }
            });

            out.push(CastingTotal {
                technique: technique.clone(),
                form: form.clone(),
                addends,
                ritual_addends,
                casting_mod_addends,
                formulaic: base.formulaic,
                ritual: base.ritual,
                spontaneous_fatiguing: base.spontaneous_fatiguing,
                spontaneous_non_fatiguing: base.spontaneous_non_fatiguing,
                within_focus,
                within_potent_field,
                non_standard,
                deficient,
                unusable: barred.contains(&(technique.clone(), form.clone())),
            });
        }
    }
    out
}

/// The Casting Total the book prints beside one known spell (ArMDE:1179's
/// "TeFo X/+Y"; formulaic only — Ritual/Spontaneous stay grid-only).
/// Independently applies [`SpellSelection::within_focus`] (X10c, Magical
/// Focus doubling) and [`SpellSelection::within_potent_field`] (D79, Potent
/// Magic's flat bonus) via [`formulaic_casting_score`] — a spell marked under
/// both gets the doubling AND the bonus together, since the two free-text
/// themes need not coincide. Either marker with no matching Virtue held falls
/// back to no contribution from that marker, silently: reachable only via a
/// hand-edited save, since the UI offers each toggle only when the character
/// holds the matching Virtue.
///
/// Computed directly via [`formulaic_casting_score`] rather than selecting
/// from a [`casting_totals`] cell, because the grid only carries three
/// figures (base, within-focus, within-potent-field) while a spell marked
/// under both markers needs a fourth combination the grid does not
/// materialize.
pub fn spell_casting_total(
    chosen: &SpellSelection,
    entity: &Entity,
    ruleset: &Ruleset,
) -> Option<i32> {
    let spell = ruleset.spell(&chosen.spell)?;
    let mods = in_play_mods(entity, ruleset);
    let base = CastingBase::of(entity, ruleset);
    Some(formulaic_casting_score(
        entity,
        ruleset,
        &mods,
        (&spell.technique, &spell.form),
        base,
        &spell.requisites,
        CastingVariant {
            focus: chosen.within_focus,
            potent: chosen.within_potent_field,
        },
    ))
}

/// Applies the Deficient-Art halving to a casting score. Weak Spontaneous
/// Magic is handled separately (it fixes the spontaneous *divisor* to 5
/// rather than halving an already-computed score — see the `spontaneous_fatiguing`
/// field comment in [`casting_totals`]'s `variant` closure). Source: ArMDE:5909-5915.
fn post(score: i32, deficient: bool) -> i32 {
    if deficient { halve(score) } else { score }
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
/// (ArMDE:9159-9161). Weak Magic
/// halves the Penetration Total *after* subtracting the level
/// (ArMDE:7064-7067).
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

/// Per-known-spell Penetration Totals. Source: ArMDE:9159-9161, :7064-7067.
pub fn penetration(entity: &Entity, ruleset: &Ruleset) -> Vec<PenetrationLine> {
    let mods = in_play_mods(entity, ruleset);
    let pen_ability = ability(entity, ruleset, ID_PENETRATION);
    let weak_magic = mods.halvings.contains(&HalvableTotal::Penetration);
    // Character-level, not spell-level: hoisted so a magus with a long spell list
    // does not re-walk his equipment for every line.
    let base = CastingBase::of(entity, ruleset);
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
        // D79: `potent: false` at both call sites — Penetration has no
        // Potent-Magic-field figure of its own (only `within_focus`, gated on
        // holding a Focus, predates D79 and is out of this ruling's scope).
        // This still FIXES a side-effect bug: before D79, `casting_mod_for`
        // folded Potent Magic's bonus unconditionally, so Penetration
        // silently included it even for a character with no Focus at all;
        // now that fold excludes Potent Magic entirely, so Penetration no
        // longer leaks it either.
        let base_casting = formulaic_casting_score(
            entity,
            ruleset,
            &mods,
            (&spell.technique, &spell.form),
            base,
            &spell.requisites,
            CastingVariant {
                focus: false,
                potent: false,
            },
        );
        let within_focus = mods.has_focus.then(|| {
            let focus_casting = formulaic_casting_score(
                entity,
                ruleset,
                &mods,
                (&spell.technique, &spell.form),
                base,
                &spell.requisites,
                CastingVariant {
                    focus: true,
                    potent: false,
                },
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
/// A magus's Magic Resistance = Form + 5 × Parma Magica (ArMDE:9390-9398). A
/// supernatural being uses
/// its **Might Score** as a blanket resistance instead of Parma — the two do
/// not stack; the higher is the base (RoP:M:1472; ArMDE:2627), and
/// the Form bonus is compatible with either. Limited Magic Resistance drops one
/// named Form's bonus; Flawed Parma Magica halves the Parma contribution
/// against one named Form. Both are per-Form, never blanket.
///
/// A True Faith Score adds a **third, wholly independent** source: "a True
/// Faith Score gains Magic Resistance equal to this score multiplied by ten"
/// (ArMDE:17611). Unlike Might, no passage states the Form bonus is compatible
/// with it, so it competes with the Form's **entire** total, not just the
/// might/parma base — "these totals do not stack … you simply use the higher
/// total" (ArMDE:2627). Because it is a separate source, it is unaffected by a
/// Flaw that reduces only the Parma/Form side (Flawed Parma Magica, Limited
/// Magic Resistance) and so can override that Flaw's reduction on the very
/// Form it names (D37, F-329).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MagicResistance {
    /// The Form Art id.
    pub form: Id,
    /// The labelled breakdown (form, and one of parma/might).
    pub addends: Vec<Addend>,
    /// The Magic Resistance total.
    pub total: i32,
}

/// Per-Form Magic Resistance. Source: ArMDE:9390-9398, :6142-6145, :6346-6349;
/// RoP:M:1472 (Might grants MR = Might
/// Score, not stacking with Parma).
pub fn magic_resistance(entity: &Entity, ruleset: &Ruleset) -> Vec<MagicResistance> {
    let mods = in_play_mods(entity, ruleset);
    let parma = ability(entity, ruleset, ID_PARMA_MAGICA);
    let parma_mr = 5 * parma;
    // A Might-being's blanket resistance = its effective Might Score. Might and
    // Parma do not stack; the higher is the base (RoP:M:1472, ArMDE:2627).
    let might = crate::effective::effective_might(entity, ruleset)
        .map(|m| i32::from(m.score))
        .unwrap_or(0);
    // True Faith's blanket floor (ArMDE:17611), compared against each Form's
    // WHOLE total below, never added to it (ArMDE:2627). `true_faith` already
    // sums every `Effect::TrueFaithGrant`, so this is a consumer change only.
    let true_faith_floor = i32::from(crate::effective::true_faith(entity, ruleset)) * 10;
    let mut out = Vec::new();
    for form in ruleset.art_ids_of(ArtType::Form) {
        let fo = effective_art_score(entity, ruleset, &form);
        // Limited Magic Resistance drops the bonus of the ONE Form its copy names,
        // and leaves every other Form's alone. Source: ArMDE:6346-6349.
        let form_bonus = if mods.no_form_bonus_forms.contains(&form) {
            0
        } else {
            fo
        };
        // Flawed Parma Magica halves the PARMA contribution against the Form its
        // copy names, and nothing else: the Flaw's subject is the Parma, while
        // :9396 puts the rest of the resistance on the Form scores, so it cannot
        // touch `form_bonus`. Source: ArMDE:6142-6145, :9396.
        let parma_for_form = if mods.halved_parma_forms.contains(&form) {
            halve(parma_mr)
        } else {
            parma_mr
        };
        // Might and Parma do not stack; the higher is the base (RoP:M:1472,
        // ArMDE:2627). The comparison uses this Form's own Parma figure, so a
        // defective Parma lowers only the Parma side of it — a Might base is a
        // being's own resistance, not a Parma contribution, and is never halved.
        let base_addend = if might > parma_for_form {
            Addend::new("might", might)
        } else {
            Addend::new("parma", parma_for_form)
        };
        let addends = vec![Addend::new("form", form_bonus), base_addend];
        let ordinary_total = sum(&addends);
        // True Faith competes with (never adds to) this Form's whole total; a
        // higher floor replaces the breakdown entirely, since it is a single
        // blanket source rather than a Form-plus-base composite.
        let (addends, total) = if true_faith_floor > ordinary_total {
            (
                vec![Addend::new("true_faith", true_faith_floor)],
                true_faith_floor,
            )
        } else {
            (addends, ordinary_total)
        };
        // X6a/e1-e2: Commanding Aura's flat bonus competes with (never adds
        // to) the ordinary/True-Faith floor above, via the same `max()`
        // idiom — the "relic absent" branch of the book's composition rule
        // (ArMDE:3583, :17653, :2627). The "relic present" branch (adds
        // instead of competing) is X7b-d's F-256 and not yet wired: `relic_mr`
        // is a permanent 0 until that slice lands, so this is always the
        // relic-absent path today.
        let (addends, total) = if mods.aura_bonus > total {
            (
                vec![Addend::new("aura_bonus", mods.aura_bonus)],
                mods.aura_bonus,
            )
        } else {
            (addends, total)
        };
        out.push(MagicResistance {
            form,
            addends,
            total,
        });
    }
    out
}
