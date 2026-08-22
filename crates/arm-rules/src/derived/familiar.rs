//! Talisman enchantment capacity and the familiar bonding read-outs (Wave 6,
//! `derived.rs` module split; extracted verbatim from `derived.rs`, no logic
//! changed).
//!
//! `CORD_COST_TABLE`/`cord_score`/`bronze_cord_bonus` stay in the **parent**
//! `derived.rs`, not here, even though they are conceptually "familiar"
//! numbers: [`super::combat::soak`] and [`super::lab::longevity_bonus`] both
//! need `bronze_cord_bonus`, so keeping the three in the parent means every
//! domain module (including this one) sees them for free via `use
//! super::*;`, with no visibility widening in any direction.

use super::lab::lab_totals;
use super::*;

// --- Talisman (enchantment capacity) ---------------------------------------

/// The talisman's enchantment-capacity read-out, in pawns of Vim vis.
///
/// "The capacity of a talisman is independent of its shape and material, and
/// instead depends on the power of the magus to whom it is attuned. The maximum
/// number of pawns of Vim vis that may be used to prepare a talisman is equal to
/// the sum of the magus's highest Technique and highest Form" (Ars Magica - Definitive Edition (Core Rules).md:10619).
///
/// The two contributing Arts and their scores are surfaced alongside the sum so
/// the UI can show the whole derivation without doing arithmetic in JS.
///
/// This is **read-only guidance**, like [`MasterpieceCap`]: no `ValidationIssue`
/// is ever raised from it. Vis costs are out of scope — the model holds no vis
/// stock, so the engine cannot know how much of the capacity is actually opened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TalismanCapacity {
    /// The magus's highest Technique.
    pub technique: Id,
    /// The magus's highest Form.
    pub form: Id,
    /// That Technique's effective score.
    pub technique_score: i32,
    /// That Form's effective score.
    pub form_score: i32,
    /// The capacity: `technique_score + form_score`, in pawns of Vim vis.
    pub pawns: i32,
}

/// The Art of `art_type` with the highest **effective** score, and that score.
/// Ties go to the alphabetically first id, because [`Ruleset::art_ids_of`] is
/// sorted and the fold keeps the first strict maximum — so the read-out never
/// flickers between equal Arts. `None` only when the ruleset defines no Art of
/// that class.
fn highest_art(entity: &Entity, ruleset: &Ruleset, art_type: ArtType) -> Option<(Id, i32)> {
    ruleset
        .art_ids_of(art_type)
        .into_iter()
        .map(|id| {
            let score = crate::effective::effective_art_score(entity, ruleset, &id);
            (id, score)
        })
        .reduce(|best, current| if current.1 > best.1 { current } else { best })
}

/// The magus's talisman capacity, or `None` when he has no talisman.
///
/// Uses the per-Art maxima of **effective** scores (Puissant Art and the like
/// folded in by [`crate::effective::effective_art_score`]), *not* the best
/// [`lab_totals`] pair: the rule reads the magus's Art scores directly, and a
/// Deficient Art halves *totals*, never the score. A magus who has bought no Arts
/// still gets a read-out, at 0 pawns.
///
/// **Non-goal — instilled effects are charged against no budget.** A talisman's
/// [`crate::types::TalismanEffect`] levels are deliberately NOT added to
/// `item_level_used`, so they can never overrun `item_level_budget`. That budget
/// exists only because of two Redcap-only Virtues: Magic Items requires "You must
/// be a Redcap to take this Virtue" (Ars Magica - Definitive Edition (Core Rules).md:4347-4349, the requirement on `:4349`),
/// and the Redcap Social Status itself grants the fifty starting levels
/// (`:4842-4850`) while stating "You may not take The Gift" (`:4850`). A talisman
/// can only be attuned by a magus, so the budget can never fund one, and charging
/// against it would invent a limit the rules do not impose. The talisman's own
/// limit is this vis capacity, which the model cannot enforce (it holds no vis
/// stock) and therefore only reports.
///
/// Source: `Ars Magica - Definitive Edition (Core Rules).md:10619`.
pub fn talisman_capacity(entity: &Entity, ruleset: &Ruleset) -> Option<TalismanCapacity> {
    entity.talisman.as_ref()?;
    let (technique, technique_score) = highest_art(entity, ruleset, ArtType::Technique)?;
    let (form, form_score) = highest_art(entity, ruleset, ArtType::Form)?;
    Some(TalismanCapacity {
        technique,
        form,
        technique_score,
        form_score,
        pawns: technique_score.saturating_add(form_score),
    })
}

// --- Familiar (bonding read-outs) ------------------------------------------

/// The **total** Lab-Total points the three cords cost (Ars Magica - Definitive Edition (Core Rules).md:10836).
///
/// The score indexing [`super::CORD_COST_TABLE`] goes through
/// [`super::cord_score`], so it is **clamped** to the +5 maximum the same line
/// sets. That clamp is load-bearing, not defensive noise: the cord fields are
/// `u8`, so a hand-edited or legacy save can carry any value up to 255, and a
/// raw index would panic inside the `derived_totals` command and take the
/// whole read-out panel down with it. (Input bounds in the UI are a separate,
/// non-durable layer — the save file is reachable without them.)
pub fn cord_points_spent(familiar: &Familiar) -> u32 {
    let cost = |score: u8| CORD_COST_TABLE[usize::from(cord_score(score))];
    cost(familiar.cord_gold) + cost(familiar.cord_silver) + cost(familiar.cord_bronze)
}

/// The level of the bonding enchantment: the familiar's Magic Might + 25 + 5 × Size.
///
/// "The level for the enchantment is equal to 25 plus the familiar's Magic Might
/// plus 5 times its Size. If the familiar has negative Size, this reduces the level
/// for the enchantment" (Ars Magica - Definitive Edition (Core Rules).md:10824), restated as
/// "**FAMILIAR BONDING LEVEL: Familiar's Magic Might + 25 + (5 x Size)**"
/// (`:10828`).
///
/// A familiar with no entered Might contributes 0 rather than suppressing the
/// read-out — the panel says so instead.
pub fn familiar_binding_level(familiar: &Familiar) -> i32 {
    let might = familiar.might.map(|m| i32::from(m.score)).unwrap_or(0);
    25 + might + 5 * i32::from(familiar.size)
}

/// The total level of the powers invested in the familiar bond.
///
/// Informational only: "there is no limit to the number of powers which may be
/// invested in a familiar" (Ars Magica - Definitive Edition (Core Rules).md:10866), so unlike a being's own
/// [`crate::effective::powers_used`] this sum is compared against no budget and can
/// raise no issue.
pub fn familiar_invested_power_levels(familiar: &Familiar) -> u32 {
    familiar.powers.iter().map(|p| u32::from(p.level)).sum()
}

/// The magus's side of the bonding season: the Lab Total he can bring to it, and
/// how it compares with what the bond needs.
///
/// The bonding Lab Total is the ordinary Lab Total shape — "any appropriate
/// Technique + any appropriate Form + Int + Magic Theory + Aura Modifier"
/// (Ars Magica - Definitive Edition (Core Rules).md:10818), restated as **FAMILIAR BONDING LAB TOTAL** (`:10826`) — so
/// [`lab_totals`] is reused and the best `(Technique, Form)` cell taken, exactly as
/// [`masterpiece_item_cap`] does. Which Arts are *appropriate* to a given beast is a
/// troupe judgment (`:10818` spells out the correspondences in prose), and
/// "Any magus should be able to find an animal that he can bind with his best
/// Technique and Form" (`:10822`) — so the best cell is the honest figure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamiliarBinding {
    /// The Technique Art of the best Lab Total.
    pub technique: Id,
    /// The Form Art of the best Lab Total.
    pub form: Id,
    /// The best base `(Technique, Form)` Lab Total.
    pub lab_total: i32,
    /// The same cell's within-focus Lab Total; `None` when the magus holds no
    /// Magical Focus. Unlike Masterpiece, `:10818` explicitly allows a focus here
    /// ("Puissant Arts and foci may apply to this"), but whether *this* familiar
    /// falls inside the focus's narrow field is a troupe judgment the engine cannot
    /// evaluate — so it is surfaced as a separate, conditional figure the UI labels
    /// as such. (Puissant Arts need no separate figure: `effective_art_score`
    /// already folds them in.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lab_total_within_focus: Option<i32>,
    /// Whether the base Lab Total reaches the binding level: "A magus can only bind
    /// a familiar if his Lab Total equals or exceeds this level" (`:10824`).
    pub lab_total_reaches_level: bool,
    /// Whether the cords bought fit in that Lab Total: "The total cost of the cords
    /// you buy cannot exceed the magus's Lab Total" (`:10836`).
    pub cord_points_within_lab_total: bool,
}

/// The familiar read-out: the bonding numbers a player can check by hand.
///
/// **Read-only guidance**, like [`MasterpieceCap`] and [`TalismanCapacity`]: no
/// `ValidationIssue` is ever raised from any of it, and a
/// `fully_populated_familiar_raises_no_issues` test in `validation` pins that.
/// Whether the bonding season is legal depends on judgments the engine cannot make
/// (which Arts suit the beast, whether a focus applies) and on vis, which the model
/// does not hold — so the engine reports rather than enforces. Vis costs (`:10830`,
/// `:10882`) are out of scope for the same reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamiliarReadout {
    /// The level of the bonding enchantment (`:10828`).
    pub binding_level: i32,
    /// The Lab-Total points the three cords cost in total (`:10836`).
    pub cord_points_spent: u32,
    /// The total level of the bond-invested powers (`:10866`; no budget).
    pub invested_power_levels: u32,
    /// The magus's best bonding Lab Total and how it compares.
    pub binding: FamiliarBinding,
}

/// The magus's familiar read-out, or `None` when he has no familiar.
/// Source: `Ars Magica - Definitive Edition (Core Rules).md:10818`, `:10822`,
/// `:10824`, `:10826`, `:10828`, `:10836`, `:10866`.
pub fn familiar_readout(entity: &Entity, ruleset: &Ruleset) -> Option<FamiliarReadout> {
    let familiar = entity.familiar.as_ref()?;
    let binding_level = familiar_binding_level(familiar);
    let cord_points = cord_points_spent(familiar);
    // Best base Lab Total across the grid; the magus picks the Te/Fo that maxes it.
    let best = lab_totals(entity, ruleset)
        .into_iter()
        .max_by_key(|lt| lt.total)?;
    Some(FamiliarReadout {
        binding_level,
        cord_points_spent: cord_points,
        invested_power_levels: familiar_invested_power_levels(familiar),
        binding: FamiliarBinding {
            technique: best.technique,
            form: best.form,
            lab_total: best.total,
            lab_total_within_focus: best.within_focus,
            lab_total_reaches_level: best.total >= binding_level,
            cord_points_within_lab_total: i64::from(cord_points) <= i64::from(best.total),
        },
    })
}
