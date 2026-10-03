//! Save-format schema versioning and backward-compatible load migrations.
//!
//! Owns [`SCHEMA_VERSION`], the [`LoadedEntity`] outcome, and
//! [`load_entity_migrating`] — the one read path a save file enters the engine
//! through — together with the folds it applies. The folds come in two kinds, and
//! each one's doc comment says which it is: **version-driven** folds stamp
//! [`SCHEMA_VERSION`] because a shape moved, while **value-driven** folds are
//! idempotent, have no version signal at all, and stamp nothing.

use std::collections::BTreeMap;

use crate::characteristics::Characteristic;
use crate::effective::{effective_characteristic_score, minimal_aging_points_for_drops};
use crate::ruleset::Ruleset;
use crate::types::{
    AURA_MODIFIER_MAX, AURA_MODIFIER_MIN, AbilityFunding, AbilityParameterValue, Entity, Id,
    Selection, SelectionParamValue, Talisman, TalismanAttunement,
};

/// Current save-format schema version.
///
/// Bumped 9 → 10 when the manual `aging_reductions` map was removed: aging-forced
/// Characteristic drops are now DERIVED from `aging_points`. Old saves are
/// migrated by [`load_entity_migrating`], which folds any legacy `aging_reductions`
/// into `aging_points`.
///
/// Bumped 10 → 11 when the aging/warping annotation fields (`apparent_age`,
/// `warping_effect`, `decrepitude_effect`, `aging_log`) were added. These are
/// purely additive `serde(default)` fields, so old saves load unchanged with no
/// migration code.
///
/// Bumped 11 → 12 when the optional per-character `spell_levels_override` field
/// was added (Issue 11). Purely additive `serde(default)`, so old saves load
/// unchanged with no migration code.
///
/// Bumped 12 → 13 when the optional `parameter` field was added to
/// [`SpellSelection`](crate::types::SpellSelection) (parametrized meta-magic Vim
/// spells like Wizard's Boost, Issue 8/10). Purely additive `serde(default)`, so
/// old saves load unchanged with no migration code.
///
/// The `SpellSelection::mastery_abilities` field and the `warping_choices` map
/// (the off-budget owed-warping V/F fills, Issue E) were both later additive
/// `serde(default)` additions that did NOT bump the version: an old save omits
/// the key and deserializes to the empty default; a new save with the default
/// omits it on write, so version 13 saves remain byte-compatible both ways. The
/// same applies to [`LongevityRitual::focus`](crate::types::LongevityRitual::focus)
/// (M5.5a): retaining
/// `bonus: Option<i8>` made the accompanying semantic change — the self-made
/// bonus is now *entered* rather than derived — self-migrating, so a pre-5.5a
/// self-made ritual simply loads as "bonus not entered" and the hint beside the
/// input suggests the number the old build derived. A pre-5.5a *external* ritual
/// that was added but never filled in stored a seeded 0 and now reads as a
/// deliberate 0. Neither is silently repaired: guessing would re-invent the live
/// derivation 5.5a exists to delete. The [`Familiar`](crate::types::Familiar)
/// statblock fields (M5.5c —
/// `animal`, `might`, `characteristics`, `size`, `personality_traits`, `powers`)
/// are the same kind of addition: a pre-5.5c familiar carried only a name and the
/// three cords, omits every new key, and writes byte-identical JSON, so the shape
/// grew without a bump.
///
/// Bumped 13 → 14 when the flat `talisman_attunements` list was replaced by
/// [`Entity::talisman`], the magus's talisman as an *item* (identity +
/// attunements + instilled effects). This is a field **move**, not an addition,
/// so [`load_entity_migrating`] folds any legacy list into the new
/// `talisman.attunements`. The fold neither infers nor discards: the attunements are
/// carried over verbatim, the identity/effects the old shape never stored stay empty
/// rather than being invented, and a legacy list that cannot deserialize fails the
/// load instead of quietly folding in nothing. So unlike the `aging_reductions`
/// migration — which *infers* a point total — it needs no [`LoadedEntity`] notice
/// flag.
///
/// Bumped 14 → 15 when [`AgingLogEntry`](crate::types::AgingLogEntry) widened from
/// `{ year, effect }` into the
/// full record of a resolved aging year (`age`, `die`, `total`,
/// `living_conditions`, `points`, `apparent_age_increased`, `crisis`) and `year`
/// became `Option<i32>`. Every added field is `serde(default,
/// skip_serializing_if)` and serde reads a bare `year` number into `Some`, so a
/// schema-14 save loads unchanged and **no migration code exists**. What is not
/// backward-compatible is the *forward* direction: a schema-15 save may omit
/// `year` entirely (a character with no `birth_year` has no calendar year to
/// write), which a schema-14 reader rejects — and a version number is exactly how
/// an older build learns not to try.
///
/// Bumped 15 → 16 for **two** fields at once — [`Entity::ability_funding`] (the
/// explicit funding discriminator) and [`Entity::wizard_furthest_phase`] (the
/// guided-wizard progress slug). Batching them is deliberate: two sequential bumps
/// would mean two migrations, two round-trip-test updates, and a window in which a
/// save written at 16 is unreadable by a build at 17.
///
/// `ability_funding` earns the bump on its own. It is the **only** field on `Entity`
/// that is written even when it holds its default, because
/// [`load_entity_migrating`] dispatches on its absence to infer the mode a pre-16
/// save recorded only implicitly (a `life_stages` plan meant life-stage funding). So
/// the forward direction breaks in the way a bump exists to announce: a schema-16
/// save may carry both a plan and a nonzero `xp_pool`, a combination a schema-15
/// reader reports as `life_stage_xp_pool_conflict` — a finding this bump retires.
/// `wizard_furthest_phase` alone would have been purely additive
/// (`skip_serializing_if`, so byte-identical when unset) and earned nothing.
///
/// Bumped 16 → 17 for [`Entity::saga_year`] (C8). The saga year used to live in the
/// app's machine-global `settings.json`, which meant a storyguide running a 1220
/// Rhine saga and a 1197 Iberia saga had **one** number that was wrong for one of
/// them — every derived age, and the `saga_year_before_birth_year` advisory with it.
/// It is a property of the saga, so it belongs to the document that was built for
/// one; `settings.json` keeps only a default for *new* documents.
///
/// The bump is earned in both directions. Backwards: a pre-17 save has no
/// `saga_year` key, so [`load_entity_migrating`] fills it from the default its
/// caller hands in and stamps the version — the same dispatch-on-absence rule
/// `ability_funding` uses, and for the same reason (`serde(default)` would make
/// "absent" and "really 1220" indistinguishable). Forwards: a schema-16 reader
/// ignores the new key and goes on resolving ages against whatever its own settings
/// file happens to say, which is precisely the defect this retires — and a version
/// number is how that reader learns not to try.
///
/// [`Entity::saga_year`] is therefore the **second** field written even when it
/// holds its default ([`Entity::ability_funding`] is the first), because the
/// absence of the key is load-bearing.
///
/// The migration takes the default as an *argument* rather than reading one. This
/// crate has no filesystem and no knowledge of `settings.json` (the engine-purity
/// invariant), and the value an old save should inherit is the user's configured
/// default, not a constant. [`crate::DEFAULT_SAGA_YEAR`] stays the engine's own
/// fallback of last resort: what [`Entity::new`] starts at, and what
/// `serde(default)` fills into a hand-edited schema-17 save that omits the key.
///
/// Bumped 17 → 18 for CV4 (`docs/vf-audit/design-cv-catalogued-values.md`):
/// [`crate::types::AbilityScore::parameter`] widens from a bare `Option<String>`
/// to `Option<AbilityParameterValue>` (D14 fix) — a Literal instance's match no
/// longer depends on exact-string equality in a single language. The bump is
/// earned by a genuine shape move, exactly like 13 → 14's talisman field: a
/// pre-18 save's bare-string `"parameter": "Latin"` is not a shape the new
/// typed enum accepts on its own, so [`wrap_legacy_ability_parameters`]
/// rewraps it into `{"text": "Latin"}` before the typed parse — run
/// unconditionally, regardless of the claimed `schema_version` (§ 5.1/§ 5.2).
/// [`fold_catalogue_matching`] then upgrades any resulting `Text` value that
/// spells out a catalogue entry's name (either locale) into `Catalogued { id
/// }` (§ 5.3) — value-driven and idempotent, so it runs on every load, not
/// just a migrating one, and stamps no version of its own; the version bump
/// belongs to the wire-shape change alone.
///
/// Bumped 18 → 19 for C5a (`docs/vf-audit/design-c0-parameter-model.md` § 8):
/// `ParamType::MultiRef` and `ParameterDomain::Spell` land, and selection-parameter
/// validation gains a shape check (`CODE_PARAM_WRONG_SHAPE`) between a
/// `multi_ref`-declared key and the `Single`/`Multi` value actually stored under
/// it. This is a **pure version marker** — no fold runs and none is owed: unlike
/// 13 → 14's talisman move, no wire shape changes here at all.
/// [`crate::types::SelectionParamValue::Multi`] was already parseable with no
/// bump owed since C0b (wire-compatible: a bare-string value still deserializes
/// as `Single`), and `Multi`'s canonical (sorted, deduplicated) form falls out
/// of `BTreeSet<Id>` for free on deserialize — so there is no shape for a fold
/// to repair and none is added for this bump. The bump is earned purely by the
/// FORWARD direction, on the same precedent as 14 → 15 and 16 → 17: an older
/// (pre-19) build has no `CODE_PARAM_WRONG_SHAPE` check and no `multi_ref`
/// type at all, so it would silently accept — or silently mis-read — a shape a
/// v19-aware validator correctly rejects. Refusing the load outright
/// (`entity.schema_version > SCHEMA_VERSION`) is exactly how that older build
/// learns not to try, rather than misvalidating in silence.
///
/// Bumped 19 → 20 for F1 (K5,
/// `docs/vf-audit/design-f0-book-template-engine.md` § 6): [`EquipmentSlot::
/// equipped`](crate::types::EquipmentSlot) (a `bool`) is replaced by
/// [`EquipmentSlot::loadout`](crate::types::EquipmentSlot::loadout) (a
/// [`LoadoutState`](crate::types::LoadoutState) three-state enum), because a
/// boolean cannot express "yields a Combat row but contributes no Load" — the
/// book's own Knight template wants exactly that for his carried spare great
/// sword. This is a genuine per-element shape move, not an addition, so
/// [`load_entity_migrating`] folds any legacy `equipped: true`/`false` into
/// `loadout: "wielded"`/`"stowed"` before the typed parse, the same
/// dispatch-on-absence idiom as every fold above, generalized one level down
/// (§ 6's own note): there is no single top-level boolean here, so the fold
/// captures — before the loop — whether ANY equipment element still lacks
/// `loadout`, and stamps [`SCHEMA_VERSION`] once afterward on that flag, rather
/// than per element.
///
/// Bumped 20 → 21 for X9b (F-16, `docs/vf-audit/corrections.md`): the point
/// item `virtue.rard` is renamed to `virtue.bard` — the English name at
/// ArMDE:3476 is the scanno "Rard", but the book's own index (ArMDE:24289) and
/// the German translation table (`tugenden-fehler.md:243`) both read "Bard",
/// and the **id**, not only the displayed name, was wrong. `source.anchor`
/// stays `"rard"`: the heading itself is still misspelled `#### Rard`, and
/// Guard A (`rules_source_provenance.rs`) compares the anchor against the
/// heading, not the corrected name — only the id and the English name track
/// the fix. This is a genuine value move (an id, not a shape), so
/// [`fold_legacy_bard_id`] renames every occurrence it finds — the bought
/// `selections` list, the three resolved-pick maps (`house_choices`,
/// `mythic_choices`, `warping_choices`), every selection `params` value
/// (`Single` or a member of `Multi`), and an `AbilityScore.parameter`'s
/// `Linked.item` — and the version is stamped only when something was
/// actually found, the same dispatch-on-presence rule every fold above uses.
/// No shipped data holds the old id in any of those places except a bought
/// `selections` entry, but a hand-edited save is this project's declared
/// hostile-input surface (`CLAUDE.md` → "This is a DESKTOP APPLICATION"), so
/// every place an item id can hide is swept, not only the one the shipped
/// catalogue happens to use.
pub const SCHEMA_VERSION: u32 = 21;

/// `(ability, original text, resolved catalogue id)` — see
/// [`LoadedEntity::migrated_catalogued_parameters`]. A named alias rather than
/// the bare tuple keeps [`fold_catalogue_matching`]'s signature readable
/// (clippy's `type_complexity`).
type MigratedCatalogueParameters = Vec<(Id, String, Id)>;
/// `(ability, text)` — see [`LoadedEntity::unresolved_catalogued_parameters`].
type UnresolvedCatalogueParameters = Vec<(Id, String)>;
/// `(ability, target item, param)` — see [`LoadedEntity::dangling_links`] and
/// [`LoadedEntity::ambiguous_links`]. The same shape serves both: a dangling
/// link (zero effective occurrences of `item`) and an ambiguous one (more than
/// one, design § 4.1) are reported identically, just filed under different
/// fields.
type LinkFoldReport = Vec<(Id, Id, String)>;

/// The outcome of loading an entity save, including any schema migration applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedEntity {
    /// The entity, migrated to the current [`SCHEMA_VERSION`].
    pub entity: Entity,
    /// Characteristics whose legacy `aging_reductions` drops were folded into
    /// `aging_points` during migration (empty when nothing was migrated, sorted).
    /// The caller surfaces a localized notice; the engine holds no user-facing
    /// string.
    pub migrated_aging_characteristics: Vec<Characteristic>,
    /// Parameterized Ability rows whose free-text value was recognized as a
    /// catalogue entry's name (either locale) and folded from `Text` into
    /// `Catalogued` — `(ability, original text, resolved catalogue id)`, in the
    /// order encountered (design § 5.3/§ 5.5). Empty when nothing matched.
    pub migrated_catalogued_parameters: MigratedCatalogueParameters,
    /// Parameterized Ability rows whose free-text value did NOT match any
    /// catalogue entry's name in either locale, and so stayed `Text` unchanged —
    /// `(ability, text)` (design § 5.3/§ 5.5: "never guessed"). Empty when
    /// nothing was left unresolved (including when the ability is not
    /// catalogued, in which case this fold does not apply to it at all). The
    /// caller surfaces this as a localized notice, exactly like
    /// [`Self::migrated_aging_characteristics`] — the player is told a value
    /// went unrecognized rather than left to discover a silently broken
    /// authorization later.
    pub unresolved_catalogued_parameters: UnresolvedCatalogueParameters,
    /// Parameterized Ability rows whose `Linked` value named a declaring item
    /// held ZERO times among effective (bought ∪ granted) selections — the
    /// Virtue was removed, never held, or the save is hand-edited — folded to
    /// `Text` (design § 5.4). `(ability, target item, param)`. Empty when
    /// nothing was dangling. The caller surfaces this as a localized notice,
    /// exactly like [`Self::unresolved_catalogued_parameters`] — a silently
    /// lost authorization is the failure mode this exists to prevent.
    pub dangling_links: LinkFoldReport,
    /// Parameterized Ability rows whose `Linked` value named a declaring item
    /// held MORE THAN ONCE among effective selections (design § 4.1's
    /// ambiguity guard) — never guessed, folded to `Text` holding the
    /// provenance-based fallback value. `(ability, target item, param)`. Empty
    /// when nothing was ambiguous — today, always empty in practice, since
    /// § 4.2 pins that no shipped grant ever duplicates a Bound/Link-declaring
    /// item; reachable only via a hand-edited or direct-unchecked save.
    pub ambiguous_links: LinkFoldReport,
}

/// Folds a legacy (schema ≤ 13) `talisman_attunements` value into
/// [`Entity::talisman`].
///
/// Two shapes are ambiguous and are decided here:
/// - An **empty** legacy list creates no talisman. A magus with no attunements had
///   no talisman recorded either, and a phantom empty item would show up as an
///   owned talisman in the UI. (An app-written save can never contain one —
///   `skip_serializing_if = "Vec::is_empty"` omits the key — so this only covers a
///   hand-edited file. The version is still bumped by the caller, because the
///   key's presence proves the old shape; the `aging_reductions` fold does the
///   same.)
/// - A save carrying **both** keys keeps the new `talisman` and drops the legacy
///   list unmerged — but only when the new shape already carries **attunements**.
///   `attunements` is the one field the legacy list can migrate into, so it is the
///   only field whose contents can make merging duplicate anything: attunements a
///   player already moved across by hand. The legacy list is then not even parsed —
///   it is discarded by that decision, so its shape cannot matter.
///
///   A talisman with **no** attunements does not win, however much its other fields
///   hold. `"talisman": {}` states nothing at all (every [`Talisman`] field is
///   `skip_serializing_if`, so an untouched talisman the app itself wrote serializes
///   as exactly `{}`, and the key's mere presence is no evidence the player moved
///   anything), and `{"description": "An ash staff"}` or an effects-only talisman
///   says nothing about *attunements* either. In all of those the list is folded into
///   the existing talisman — filling its `attunements` while its own fields are left
///   as written. Letting them take precedence would drop the list unparsed while
///   [`load_entity_migrating`] still stamps [`SCHEMA_VERSION`] — the same permanent
///   loss the error path below exists to prevent.
///
/// A legacy list that **cannot** deserialize (a `bonus` outside `i8`, a bonus
/// written as a JSON string, `null` in place of the list) is an error, propagated to
/// [`load_entity_migrating`]'s caller. Swallowing it would fold in nothing while the
/// caller still stamps the current [`SCHEMA_VERSION`], so the load would report
/// success and the next save would rewrite the file without the legacy key —
/// destroying the attunements. Failing the load leaves the file untouched.
fn fold_legacy_talisman(
    entity: &mut Entity,
    legacy: serde_json::Value,
) -> Result<(), serde_json::Error> {
    let new_shape_carries_attunements = entity
        .talisman
        .as_ref()
        .is_some_and(|talisman| !talisman.attunements.is_empty());
    if new_shape_carries_attunements {
        return Ok(());
    }
    let attunements: Vec<TalismanAttunement> = serde_json::from_value(legacy)?;
    if attunements.is_empty() {
        return Ok(());
    }
    // Fold into the existing talisman so a description-only or effects-only one keeps
    // its own data; only `attunements` (empty, per the gate above) is filled in.
    entity
        .talisman
        .get_or_insert_with(Talisman::default)
        .attunements = attunements;
    Ok(())
}

/// The three items whose `being` parameter changed from a free-text domain to an
/// enumerated one, so a save written before that change holds a typed label where
/// a `being.*` id now belongs.
///
/// `virtue.alluring_to_beings` and `flaw.magical_being_companion` carry a `being`
/// parameter too and are deliberately **absent**: both are still `text` domains
/// (their value is the player's own description), so folding them would rewrite a
/// perfectly legal value into an id their domain never declared.
const REFOLDED_BEING_ITEMS: &[&str] = &[
    "flaw.offensive_to_beings",
    "flaw.unbearable_to_beings",
    "virtue.inoffensive_to_beings",
];

/// The parameter key [`REFOLDED_BEING_ITEMS`] all share.
const BEING_PARAM_KEY: &str = "being";

/// The labels a pre-enumeration save can hold in a `being` slot, mapped to the id
/// each one names: six classes of being × the two shipped languages, plus the three
/// German **dative** forms the rulebook itself prints.
///
/// **These strings are matched, never rendered**, so they are not a breach of the
/// no-hardcoded-user-facing-strings rule: nothing here reaches a user, and the
/// localized labels the UI *does* render still live only in
/// `rules/i18n/<lang>/virtues_flaws.json`.
///
/// The table is frozen on purpose rather than read from those files. Two reasons:
/// [`load_entity_migrating`] holds neither a [`Ruleset`](crate::ruleset::Ruleset)
/// nor an i18n map, so a live lookup is impossible without an API change; and it
/// would be wrong even then, because editing a label later would silently change
/// what a decade-old save migrates *to*. What a v0.2.x app printed is history, and
/// history does not move.
///
/// Sources, all three read at implementation time. The i18n files supply the labels
/// the picker showed:
/// `rules/i18n/en/virtues_flaws.json` and `rules/i18n/de/virtues_flaws.json`, both
/// at the `being.*` entries. The rulebook supplies the wording a player copying
/// from the page would have typed:
/// ArMDE:4135, :6526, :6893, and the
/// German mirror at the same line numbers. The English lists and the German lists
/// at `ArMDE:6526` / `ArMDE:6893` fold onto the i18n labels exactly; the German `ArMDE:4135` list
/// is in the dative, which contributes the three extra keys below. No synonym
/// beyond what those print is invented — an unrecognised value is left exactly as
/// typed.
///
/// Every key is distinct under [`fold_being_label`], and no key is ever equal to a
/// `being.*` id, which is what makes the fold both unambiguous and idempotent.
const LEGACY_BEING_LABELS: &[(&str, &str)] = &[
    // rules/i18n/en/virtues_flaws.json
    ("Animals", "being.animals"),
    ("Demons", "being.demons"),
    ("Divine Beings", "being.divine"),
    ("Faeries", "being.faeries"),
    ("Magical Creatures", "being.magical_creatures"),
    ("Mundane Humans", "being.mundane_humans"),
    // rules/i18n/de/virtues_flaws.json
    ("Tiere", "being.animals"),
    ("Dämonen", "being.demons"),
    ("Göttliche Wesen", "being.divine"),
    ("Feen", "being.faeries"),
    ("Magische Kreaturen", "being.magical_creatures"),
    ("Sterbliche Menschen", "being.mundane_humans"),
    // The German **dative** forms, printed by the Inoffensive entry at
    // `ArMDE:4135` — "…von Wesen verbunden: Tieren, göttlichen Wesen, Feen, Dämonen
    // oder magischen Kreaturen." A player filling the old free-text box while
    // reading that page copied the inflected form off it, so these are recovered
    // too. Transcribed, not declined: "Feen" and "Dämonen" are identical in the
    // dative and are already above, and the fourth class has no dative form here
    // to take — `sterbliche Menschen` appears only in the two entries that print
    // the nominative (`ArMDE:6526`, `ArMDE:6893`), so no "sterblichen Menschen" key is
    // invented.
    ("Tieren", "being.animals"),
    ("göttlichen Wesen", "being.divine"),
    ("magischen Kreaturen", "being.magical_creatures"),
];

/// Folds a typed `being` label for comparison: lowercased and stripped of all
/// whitespace, so "  Divine  beings " and "Divine Beings" are one value. The old
/// slot was a free-text box, and case and spacing were never part of the choice.
fn fold_being_label(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Rewrites a legacy typed `being` label into the `being.*` id it names, in place.
///
/// A no-op for anything else: another item, a missing `being` key, a value already
/// holding an id, or a label the frozen table does not recognise. That last case is
/// the deliberate one — the player re-picks it once, and nothing is guessed on
/// their behalf.
fn fold_legacy_being_param(selection: &mut Selection) {
    if !REFOLDED_BEING_ITEMS.contains(&selection.item_ref.as_str()) {
        return;
    }
    let Some(typed) = selection
        .params
        .get(BEING_PARAM_KEY)
        .and_then(SelectionParamValue::as_single)
    else {
        return;
    };
    let folded = fold_being_label(typed.as_str());
    let Some((_, id)) = LEGACY_BEING_LABELS
        .iter()
        .find(|(label, _)| fold_being_label(label) == folded)
    else {
        return;
    };
    selection.params.insert(
        BEING_PARAM_KEY.to_string(),
        SelectionParamValue::Single(Id::new(*id)),
    );
}

/// Applies [`fold_legacy_being_param`] everywhere a save can hold a [`Selection`]:
/// the bought list plus the three resolved-pick maps, since a House choice, a
/// mythic-type pick or a Warping fill can name one of these items too.
fn fold_legacy_being_params(entity: &mut Entity) {
    for selection in &mut entity.selections {
        fold_legacy_being_param(selection);
    }
    for choices in [
        &mut entity.house_choices,
        &mut entity.mythic_choices,
        &mut entity.warping_choices,
    ] {
        for selection in choices.values_mut() {
            fold_legacy_being_param(selection);
        }
    }
}

/// X9b (F-16): the point-item id [`fold_legacy_bard_id`] renames away from.
/// See [`SCHEMA_VERSION`]'s 20 → 21 paragraph for why.
const LEGACY_BARD_ID: &str = "virtue.rard";
/// What [`LEGACY_BARD_ID`] renames to.
const RENAMED_BARD_ID: &str = "virtue.bard";

/// Renames a single parameter value from [`LEGACY_BARD_ID`] to
/// [`RENAMED_BARD_ID`] in place, wherever it appears — the one bound [`Id`] of
/// a `Single` value, or any member of a `Multi` set — reporting whether
/// anything changed. A parameter value is compared byte-for-byte wherever a
/// selection's identity or a grant's `options.contains(pick)` is decided
/// (`trim_selection_params`'s doc comment), so a stale id surviving under ANY
/// parameter key — not only the `item_ref` the parameter is declared
/// against — would silently stop matching once the catalogue id moves.
fn fold_legacy_bard_param_value(value: &mut SelectionParamValue) -> bool {
    match value {
        SelectionParamValue::Single(id) if id.as_str() == LEGACY_BARD_ID => {
            *id = Id::new(RENAMED_BARD_ID);
            true
        }
        SelectionParamValue::Single(_) => false,
        SelectionParamValue::Multi(ids) => {
            if !ids.remove(&Id::new(LEGACY_BARD_ID)) {
                return false;
            }
            ids.insert(Id::new(RENAMED_BARD_ID));
            true
        }
    }
}

/// Renames `selection.item_ref` and every one of `selection.params`'s values
/// from [`LEGACY_BARD_ID`] to [`RENAMED_BARD_ID`], reporting whether anything
/// changed.
fn fold_legacy_bard_selection(selection: &mut Selection) -> bool {
    let mut found = false;
    if selection.item_ref.as_str() == LEGACY_BARD_ID {
        selection.item_ref = Id::new(RENAMED_BARD_ID);
        found = true;
    }
    for value in selection.params.values_mut() {
        found |= fold_legacy_bard_param_value(value);
    }
    found
}

/// X9b (F-16): renames every occurrence of [`LEGACY_BARD_ID`] on `entity` to
/// [`RENAMED_BARD_ID`] — the bought `selections` list, the three
/// resolved-pick maps (`house_choices`, `mythic_choices`, `warping_choices`,
/// the same containers [`fold_legacy_being_params`] and
/// [`trim_all_selection_params`] already walk for the same structural
/// reason), and every `AbilityScore.parameter`'s
/// [`AbilityParameterValue::Linked`] `item` — the one place a declaring item's
/// id can hide outside a [`Selection`] (CV5). Returns whether ANY occurrence
/// was found, so the caller stamps [`SCHEMA_VERSION`] only when something
/// actually moved, exactly like [`fold_legacy_equipment_loadout`]'s own
/// dispatch-on-presence flag.
fn fold_legacy_bard_id(entity: &mut Entity) -> bool {
    let mut found = false;
    for selection in &mut entity.selections {
        found |= fold_legacy_bard_selection(selection);
    }
    for choices in [
        &mut entity.house_choices,
        &mut entity.mythic_choices,
        &mut entity.warping_choices,
    ] {
        for selection in choices.values_mut() {
            found |= fold_legacy_bard_selection(selection);
        }
    }
    for score in &mut entity.ability_scores {
        if let Some(AbilityParameterValue::Linked { item, .. }) = &mut score.parameter
            && item.as_str() == LEGACY_BARD_ID
        {
            *item = Id::new(RENAMED_BARD_ID);
            found = true;
        }
    }
    found
}

/// Trims surrounding whitespace off every parameter value of one selection.
///
/// Parameter values establish a selection's **identity** in four places that all
/// compare them byte-for-byte: the duplicate-selection key (the whole params map),
/// a grant's `options.contains(pick)`, a mythic type's required-Virtue check, and
/// [`Entity::normalize`]'s sort. So "Wolf Shape ", "Wolf Shape" and " Wolf Shape"
/// were three distinct targets, and the per-power cap counted them separately.
///
/// Every domain is trimmed, not just
/// [`ParameterDomain::Text`](crate::types::ParameterDomain::Text) — this function has
/// no ruleset to ask, and no domain has a legal value with an edge of whitespace:
/// an id never does, and a `text` value's padding was never part of the player's
/// choice. Case is deliberately **left alone**: the rules ask for no case-folding,
/// and two powers a player capitalised differently are their own business.
///
/// A value that trims to nothing is kept as the empty string rather than deleted,
/// so the player sees `missing_param` naming the box to fill — the same issue an
/// absent key raises, which is what makes a blank power and an unrecorded one read
/// identically.
fn trim_selection_params(selection: &mut Selection) {
    // `Multi` values are left untouched: no producer exists yet (C5a/C5b's
    // job), so there is no padded free-text shape to trim there today.
    let padded: Vec<String> = selection
        .params
        .iter()
        .filter_map(|(key, value)| {
            let single = value.as_single()?;
            (single.as_str().trim() != single.as_str()).then(|| key.clone())
        })
        .collect();
    for key in padded {
        let Some(single) = selection.params[&key].as_single() else {
            continue;
        };
        let trimmed = SelectionParamValue::Single(Id::new(single.as_str().trim()));
        selection.params.insert(key, trimmed);
    }
}

/// Applies [`trim_selection_params`] everywhere a save can hold a [`Selection`]:
/// the bought list plus the three resolved-pick maps, exactly as
/// [`fold_legacy_being_params`] does — a House choice, a mythic-type pick or a
/// Warping fill carries params of its own, and a padded one falls off its own menu
/// (`options.contains(pick)` is full [`Selection`] equality).
fn trim_all_selection_params(entity: &mut Entity) {
    for selection in &mut entity.selections {
        trim_selection_params(selection);
    }
    for choices in [
        &mut entity.house_choices,
        &mut entity.mythic_choices,
        &mut entity.warping_choices,
    ] {
        for selection in choices.values_mut() {
            trim_selection_params(selection);
        }
    }
}

/// Rewrites every legacy bare-string `ability_scores[].parameter` into the new
/// tagged shape `{"text": <string>}`, before the typed parse. A bare string
/// cannot self-report "id" vs "text" the way [`SelectionParamValue`]'s
/// scalar/array split already could, so — unlike that field — this one is not
/// wire-compatible without a rewrite (design § 5.1).
///
/// Runs regardless of the claimed `schema_version` — a hand-edited save can
/// claim any version alongside a legacy bare-string `parameter`, and the shape
/// is rewrapped either way (design § 5.2). Idempotent: a `parameter` that is
/// already an object (any schema-18+ save, or one this same pre-pass already
/// rewrote) is untouched; only a JSON *string* value is rewrapped.
///
/// Untrusted input: a linear walk over `ability_scores`, an array already
/// bounded by the file the player opened — no recursion, no
/// attacker-controlled loop count beyond "one iteration per ability the save
/// already lists" (design § 5.2).
fn wrap_legacy_ability_parameters(value: &mut serde_json::Value) {
    let Some(scores) = value
        .get_mut("ability_scores")
        .and_then(|v| v.as_array_mut())
    else {
        return;
    };
    for score in scores {
        let Some(obj) = score.as_object_mut() else {
            continue;
        };
        if let Some(serde_json::Value::String(text)) = obj.get("parameter").cloned() {
            obj.insert("parameter".into(), serde_json::json!({ "text": text }));
        }
    }
}

/// Rewrites every legacy `equipment[].equipped: bool` into `loadout:
/// LoadoutState`, before the typed parse (K5, F1 —
/// `docs/vf-audit/design-f0-book-template-engine.md` § 6). Unlike the four
/// single top-level-boolean folds this function's caller also runs
/// (`aging_reductions`, `talisman_attunements`, `ability_funding`,
/// `saga_year`), the legacy shape here lives per equipment-array ELEMENT, so
/// this fold generalizes the same dispatch-on-absence idiom one level down:
/// each element is folded independently.
///
/// Returns whether ANY element still lacked `loadout` before the fold ran —
/// the caller stamps [`SCHEMA_VERSION`] on that flag, exactly once, after
/// deserializing: a genuine per-element shape move earns the bump, the same
/// criterion every other meaning-changing fold above uses.
///
/// Dispatch is on `loadout`'s presence, never `equipped`'s or the claimed
/// `schema_version`: an element that already carries `loadout` is left
/// untouched (idempotent, matching every sibling fold), even if it also
/// carries a stray `equipped` key — a hand-edited or partially migrated save
/// may carry either shape, or both, at any claimed version.
fn fold_legacy_equipment_loadout(value: &mut serde_json::Value) -> bool {
    let equipment_loadout_absent = value
        .get("equipment")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .any(|slot| slot.get("loadout").is_none());

    if let Some(equipment) = value.get_mut("equipment").and_then(|v| v.as_array_mut()) {
        for slot in equipment {
            let Some(obj) = slot.as_object_mut() else {
                continue;
            };
            if obj.contains_key("loadout") {
                continue;
            }
            let was_equipped = obj
                .get("equipped")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            obj.remove("equipped");
            obj.insert(
                "loadout".into(),
                serde_json::json!(if was_equipped { "wielded" } else { "stowed" }),
            );
        }
    }

    equipment_loadout_absent
}

/// The catalogue-matching fold (design § 5.3): upgrades a parameterized
/// Ability's `Text { text }` value to `Catalogued { id }` where `text`
/// case-insensitively, trimmed-ly spells out one of the ability's catalogue's
/// values' names, in either locale — never guessed, so an unmatched or
/// ambiguous value stays `Text` unchanged and is reported.
///
/// A no-op for any `AbilityScore` whose value is not `Text` (`Catalogued` and
/// `Linked` pass through untouched — idempotent, and safe to run on every
/// load, not just a migrating one) or whose ability is not catalogued at all
/// (`ability.craft`, `ability.area_lore`, `ability.mystery_cult_lore` stay
/// free text by design, D9).
///
/// "Exactly one match" is well-defined because ruleset load already rejects a
/// catalogue whose values collide under this same trimmed/case-folded name
/// comparison (§ 2.2) — so at most one catalogue value can match a given
/// folded text within one catalogue.
fn fold_catalogue_matching(
    entity: &mut Entity,
    ruleset: &Ruleset,
    catalogue_names: &BTreeMap<Id, Vec<String>>,
) -> (MigratedCatalogueParameters, UnresolvedCatalogueParameters) {
    let mut migrated = Vec::new();
    let mut unresolved = Vec::new();
    for score in &mut entity.ability_scores {
        let Some(AbilityParameterValue::Text { text }) = &score.parameter else {
            continue;
        };
        let Some(ability) = ruleset.ability(&score.ability) else {
            continue;
        };
        if !ability.catalogued {
            continue;
        }
        let Some(key) = ability.parameter.as_deref() else {
            continue;
        };
        let catalogue_id = Id::new(format!("catalogue.{key}"));
        let Some(catalogue) = ruleset.parameter_catalogues().get(&catalogue_id) else {
            continue;
        };
        let folded_text = crate::catalogue::fold_name(text);
        let matched = catalogue.values.iter().find(|value| {
            catalogue_names
                .get(&value.id)
                .into_iter()
                .flatten()
                .any(|name| crate::catalogue::fold_name(name) == folded_text)
        });
        match matched {
            Some(value) => {
                migrated.push((score.ability.clone(), text.clone(), value.id.clone()));
                score.parameter = Some(AbilityParameterValue::Catalogued {
                    id: value.id.clone(),
                });
            }
            None => {
                unresolved.push((score.ability.clone(), text.clone()));
            }
        }
    }
    (migrated, unresolved)
}

/// Design § 5.4: walks every `AbilityScore` whose `parameter` is
/// [`AbilityParameterValue::Linked`] and resolves it via
/// [`crate::effective::resolve_link`] (§ 4.1's ambiguity guard) against the
/// entity's effective selections. A dangling link (zero occurrences) or one
/// whose target's parameter was declared but never filled in is folded to
/// `Text` holding that value (empty when none); an ambiguous link (more than
/// one occurrence) is folded to `Text` holding the provenance-based fallback.
/// Runs unconditionally on every load, not gated on `schema_version < 18` — a
/// dangling or ambiguous link can arise on an up-to-date save just as easily
/// (a later ruleset revision removed the item, or a duplicate selection
/// exists).
///
fn fold_dangling_and_ambiguous_links(
    entity: &mut Entity,
    ruleset: &Ruleset,
) -> (LinkFoldReport, LinkFoldReport) {
    let targets: Vec<(usize, Id, Id, String)> = entity
        .ability_scores
        .iter()
        .enumerate()
        .filter_map(|(index, score)| match &score.parameter {
            Some(AbilityParameterValue::Linked { item, param }) => {
                Some((index, score.ability.clone(), item.clone(), param.clone()))
            }
            _ => None,
        })
        .collect();

    let mut dangling = Vec::new();
    let mut ambiguous = Vec::new();
    for (index, ability, item, param) in targets {
        let resolution = crate::effective::resolve_link(entity, ruleset, &item, &param);
        let (report, fold_value): (Option<&mut LinkFoldReport>, Option<String>) = match resolution {
            crate::effective::LinkResolution::Resolved(Some(_)) => (None, None),
            crate::effective::LinkResolution::Resolved(None)
            | crate::effective::LinkResolution::Dangling => (Some(&mut dangling), None),
            crate::effective::LinkResolution::Ambiguous(fallback) => {
                (Some(&mut ambiguous), fallback)
            }
        };
        if let Some(report) = report {
            report.push((ability, item, param));
            entity.ability_scores[index].parameter =
                Some(AbilityParameterValue::text(fold_value.unwrap_or_default()));
        }
    }
    (dangling, ambiguous)
}

/// Deserializes an entity from JSON, applying backward-compatible save
/// migrations, and reports what was migrated.
///
/// Saves at schema ≤ 9 carried a manual `aging_reductions` map of completed
/// Characteristic drops. The current model derives those drops from
/// `aging_points` (ArMDE:16579), so any legacy reductions are folded into
/// `aging_points` as the minimal point total that reproduces the same number of
/// drops — counted from the actual score, free deltas included, by the same helper
/// the live `effective/warping.rs::aging_drops` uses. Because every aging point counts toward Decrepitude — including those
/// "lost" to a drop — this fold also corrects the old model's Decrepitude
/// under-count.
///
/// Saves at schema ≤ 13 carried a flat `talisman_attunements` list. Schema 14
/// models the talisman as an item ([`Talisman`]), so any legacy list is folded
/// into `talisman.attunements`. See [`fold_legacy_talisman`] for the ambiguous
/// shapes it has to decide.
///
/// Saves at schema ≤ 15 carried no `ability_funding` key, because the funding mode
/// was not stored at all — it *was* the presence of `life_stages`. That inference is
/// applied here, **once at load** rather than on every read: a plan present means
/// [`AbilityFunding::LifeStages`], no plan means [`AbilityFunding::Pool`]. From 16 on
/// the key is always written (see [`Entity::ability_funding`]), so its absence is
/// exactly the pre-16 signal and nothing else. `wizard_furthest_phase` needs no fold:
/// absent stays `None`, which its reader treats as "no wizard progress recorded".
///
/// Saves written before the three `being` parameters became enumerated domains hold
/// a **typed label** where a `being.*` id now belongs, so every such save reported
/// `unknown_param_value` on open. Those labels are folded back onto their ids by
/// [`fold_legacy_being_params`]. This one is unlike every fold above it: it has **no
/// version signal at all**, because the closed lists were a *ruleset* change, not a
/// save-format change — nothing in a save distinguishes the two eras, and no
/// `SCHEMA_VERSION` bump could. So the fold is **value-driven and idempotent**: it
/// rewrites the twelve labels it recognises and leaves everything else, an id
/// included, exactly as written. For the same reason it does **not** stamp
/// `SCHEMA_VERSION` — no schema moved, and stamping would rewrite the version of a
/// save that was already current.
///
/// Every parameter value is also **trimmed** here, by
/// [`trim_all_selection_params`]. Nothing trimmed a value arriving from a save
/// file, and parameter values are compared byte-for-byte wherever a selection's
/// identity is decided, so a padded descriptor was a target of its own. This too is
/// value-driven, idempotent and version-free, and it belongs at **load** rather than
/// in [`Entity::normalize`]: normalize runs on every *save* and sorts selections by
/// `(ref, params)`, so trimming there would silently reorder the rows of a file the
/// player had merely opened — against the zero-noise-diff convention. Trimming on
/// load lets the reordering settle once, at open.
///
/// What it deliberately does not do: Folk Magic's `category` and the three per-power
/// Flaws' `power` were never *stored* in the old shape, so there is nothing to
/// migrate from. They stay a visible `missing_param` naming the item and key rather
/// than a fabricated placeholder — inventing one would invent a character's rules
/// choices. Likewise a genuine `too_many_selections` (an over-cap Puissant Art the
/// engine was previously blind to) is a rules finding, not a format problem, and
/// survives untouched.
///
/// It also clamps [`Entity::aura`] to
/// [`AURA_MODIFIER_MIN`]..=[`AURA_MODIFIER_MAX`]. That field is the only unbounded
/// signed one on [`Entity`], and [`Entity::normalize`] — which clamps it on every
/// *save* — never runs on the read path, so a value straight from a file otherwise
/// reaches the derived layer untouched and saturates every casting, lab and
/// penetration total into a meaningless number. Like the two folds above this is
/// value-driven, idempotent and version-free, so it stamps no [`SCHEMA_VERSION`]:
/// the document's *shape* is current, only one of its values was out of range, and
/// a legal aura round-trips byte-identically.
///
/// Dispatch is on legacy-key *presence*, never on the recorded version: a
/// hand-edited save may carry any `schema_version` alongside either shape. Plain
/// `serde` deserialization still works for current saves; this wrapper only adds
/// the folds and a migration report.
///
/// Either legacy value failing to deserialize fails the **whole load**, exactly as a
/// malformed current field does. A fold that quietly yielded nothing would still get
/// the version stamped, so the load would look successful and the next save would
/// drop the legacy key — losing the data permanently.
///
/// `ruleset`/`catalogue_names` are new as of the 17 → 18 bump (design § 5.6):
/// the catalogue-matching fold above needs to know which abilities are
/// catalogued and what their catalogue values are named in each locale, and
/// this crate has no filesystem and cannot load its own rules data — so the
/// caller (which already owns ruleset loading/caching for every other command)
/// hands both in. `catalogue_names` covers **both** shipped locales at once,
/// independent of whichever language the UI happens to be showing, because
/// recognizing a value must not depend on which language the player was using
/// when they typed it.
pub fn load_entity_migrating(
    json: &str,
    default_saga_year: i32,
    ruleset: &Ruleset,
    catalogue_names: &BTreeMap<Id, Vec<String>>,
) -> Result<LoadedEntity, serde_json::Error> {
    let mut value: serde_json::Value = serde_json::from_str(json)?;
    let legacy = value
        .as_object_mut()
        .and_then(|obj| obj.remove("aging_reductions"));
    let legacy_attunements = value
        .as_object_mut()
        .and_then(|obj| obj.remove("talisman_attunements"));
    wrap_legacy_ability_parameters(&mut value);
    let equipment_loadout_absent = fold_legacy_equipment_loadout(&mut value);
    // Dispatch on the key's absence, never on the recorded `schema_version`: a
    // hand-edited save may carry any version alongside either shape. Read before the
    // deserialization below, because `serde(default)` would make the two
    // indistinguishable afterwards.
    let funding_absent = !value
        .as_object()
        .is_some_and(|obj| obj.contains_key("ability_funding"));
    // Same dispatch-on-absence rule, for the same reason: `serde(default)` fills the
    // engine's own 1220 in, which is indistinguishable afterwards from a save that
    // really says 1220. Read before deserializing.
    let saga_year_absent = !value
        .as_object()
        .is_some_and(|obj| obj.contains_key("saga_year"));
    let mut entity: Entity = serde_json::from_value(value)?;

    // A save from a build that does not exist yet is REFUSED, not migrated. Every
    // fold below was written against a shape this build knows; running them over a
    // newer one would stamp the current version onto a document whose unknown keys
    // serde has already dropped, and the next save would make that loss permanent.
    // Refusing leaves the file exactly as the user left it. This is the one place
    // the recorded version is read as a decision rather than as data — a hand-edited
    // *legacy* version still decides nothing, because the folds dispatch on the
    // legacy keys themselves.
    if entity.schema_version > SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "save schema_version {} is newer than this build's {SCHEMA_VERSION}",
            entity.schema_version
        )));
    }

    // All three folds below are value-driven and idempotent, and deliberately stamp
    // no version — see the `being`, whitespace and aura paragraphs above. The trim
    // runs first, so the being fold sees canonical values.
    trim_all_selection_params(&mut entity);
    fold_legacy_being_params(&mut entity);
    // `aura` is the only unbounded signed field on `Entity`, and `Entity::normalize`
    // runs on save, never here — so without this an out-of-range value from a file
    // reaches the whole derived layer, where it saturates every casting/lab/
    // penetration total into a meaningless read-out.
    entity.aura = entity.aura.clamp(AURA_MODIFIER_MIN, AURA_MODIFIER_MAX);

    if let Some(legacy_attunements) = legacy_attunements {
        fold_legacy_talisman(&mut entity, legacy_attunements)?;
        entity.schema_version = SCHEMA_VERSION;
    }

    if funding_absent {
        // The pre-16 rule, applied once: the plan's presence WAS the mode. A value
        // that will not deserialize has already failed the whole load above, so this
        // fold cannot silently yield nothing while the version gets stamped.
        entity.ability_funding = if entity.life_stages.is_some() {
            AbilityFunding::LifeStages
        } else {
            AbilityFunding::Pool
        };
        entity.schema_version = SCHEMA_VERSION;
    }

    if equipment_loadout_absent {
        // K5 (F1): at least one equipment element still carried the legacy
        // `equipped: bool` shape (or omitted both keys) before
        // `fold_legacy_equipment_loadout` ran above — a genuine per-element shape
        // move, so the version is stamped, same as every other meaning-changing
        // fold in this function.
        entity.schema_version = SCHEMA_VERSION;
    }

    if saga_year_absent {
        // Pre-17: the saga year lived in `settings.json`, machine-globally. The value
        // the document inherits is therefore the one the user has configured — handed
        // in, because this crate may not read a settings file (engine purity).
        entity.saga_year = default_saga_year;
        entity.schema_version = SCHEMA_VERSION;
    }

    // X9b (F-16): dispatch is on finding the OLD id itself, not a legacy key's
    // presence — there is no single top-level marker the way `talisman_attunements`
    // or `ability_funding`'s absence is, so `fold_legacy_bard_id` reports whether it
    // renamed anything, the same per-element flag `fold_legacy_equipment_loadout`
    // returns. Runs AFTER the trim/being folds above (so it sees canonical values)
    // and BEFORE `fold_dangling_and_ambiguous_links` below, so a renamed
    // `Linked.item` resolves against the also-renamed `selections` in the same pass.
    if fold_legacy_bard_id(&mut entity) {
        entity.schema_version = SCHEMA_VERSION;
    }

    let mut migrated_aging_characteristics = Vec::new();
    if let Some(legacy) = legacy {
        // The save predates schema 10 (manual `aging_reductions`); fold it in and
        // bump the version. Current saves (no legacy field) are left untouched so
        // a load is a faithful, byte-stable round trip. A map that cannot
        // deserialize fails the load for the same reason as the talisman list: a
        // silently empty fold would still bump the version, and the next save would
        // drop the key.
        let reductions: BTreeMap<Characteristic, u8> = serde_json::from_value(legacy)?;
        for (characteristic, drops) in reductions {
            if drops == 0 {
                continue;
            }
            // Ruling F-A: the drops are reconstructed against the actual score
            // (free deltas included), through the same cost sequence the live
            // `effective/warping.rs::aging_drops` walks. Source: ArMDE:16579.
            let actual = effective_characteristic_score(&entity, ruleset, characteristic);
            let add = minimal_aging_points_for_drops(actual, u32::from(drops));
            let slot = entity.aging_points.entry(characteristic).or_insert(0);
            *slot = slot.saturating_add(u8::try_from(add).unwrap_or(u8::MAX));
            migrated_aging_characteristics.push(characteristic);
        }
        entity.schema_version = SCHEMA_VERSION;
    }

    migrated_aging_characteristics.sort();
    let (migrated_catalogued_parameters, unresolved_catalogued_parameters) =
        fold_catalogue_matching(&mut entity, ruleset, catalogue_names);
    let (dangling_links, ambiguous_links) = fold_dangling_and_ambiguous_links(&mut entity, ruleset);
    Ok(LoadedEntity {
        entity,
        migrated_aging_characteristics,
        migrated_catalogued_parameters,
        unresolved_catalogued_parameters,
        dangling_links,
        ambiguous_links,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ruleset::RulesetSources;
    use crate::types::{EntityKind, LoadoutState, RulesetRef};
    use crate::validation::DEFAULT_SAGA_YEAR;
    use pretty_assertions::assert_eq;

    /// A ruleset carrying no catalogued abilities and no catalogue values —
    /// every test in this module exercises a migration fold unrelated to
    /// `AbilityScore.parameter` (aging, talisman, funding, saga year), so
    /// [`fold_catalogue_matching`] is a no-op against it (no catalogued
    /// ability to fold).
    fn empty_ruleset() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            point_items: "[]",
            type_profiles: "[]",
            ..RulesetSources::default()
        })
        .expect("an empty ruleset loads")
    }

    /// The matching, empty `catalogue_names` map (see [`empty_ruleset`]).
    fn empty_catalogue_names() -> BTreeMap<Id, Vec<String>> {
        BTreeMap::new()
    }

    /// A legacy save carrying `aging_reductions` migrates: the completed drops are
    /// folded into `aging_points` as the minimal total reproducing them, the field
    /// is dropped, the schema is bumped, and the migration is reported.
    #[test]
    fn legacy_aging_reductions_migrate_into_aging_points() {
        // Com +2 with one completed aging drop under the old manual model.
        let old = r#"{
          "schema_version": 9,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "characteristics": { "com": 2 },
          "aging_reductions": { "com": 1 }
        }"#;
        let loaded = load_entity_migrating(
            old,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(
            loaded.migrated_aging_characteristics,
            vec![Characteristic::Com]
        );
        // Minimal points reproducing one drop on a +2 score = |2| + 1 = 3.
        assert_eq!(
            loaded
                .entity
                .aging_points
                .get(&Characteristic::Com)
                .copied(),
            Some(3)
        );
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
        // The bought score is untouched (point-buy stays valid).
        assert_eq!(
            loaded
                .entity
                .characteristics
                .get(&Characteristic::Com)
                .copied(),
            Some(2)
        );
        // Re-serializing carries no `aging_reductions` field.
        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("aging_reductions"));
    }

    /// Ruling F-A, in lockstep: the legacy fold reconstructs the drops against the
    /// same threshold the live [`crate::effective::aging_drops`] uses — the actual
    /// score, free delta included (ArMDE:16579). Great (Stamina) twice over a bought
    /// +3 is an actual +5, so one legacy drop needs |5| + 1 = 6 points, and the
    /// live computation reads exactly one drop back from them (and none from 5).
    #[test]
    fn legacy_aging_fold_agrees_with_the_live_drops_under_a_great_characteristic() {
        let ruleset = Ruleset::from_sources(RulesetSources {
            point_items: r#"[{
              "id": "virtue.great_characteristic",
              "kind": "virtue",
              "classification": "narrative",
              "magnitude": "minor",
              "categories": ["general"],
              "entity_kinds": ["character"],
              "parameters": [{ "key": "characteristic", "type": "ref", "domain": "characteristic" }],
              "effects": [
                { "type": "characteristic_score_delta_param", "param": "characteristic", "amount": 1 }
              ],
              "max_per_target": 2
            },
            { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
              "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"] }
            ]"#,
            type_profiles: "[]",
            ..RulesetSources::default()
        })
        .expect("a Great Characteristic ruleset loads");
        let old = r#"{
          "schema_version": 9,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "selections": [
            { "ref": "virtue.great_characteristic", "params": { "characteristic": "characteristic.sta" } },
            { "ref": "virtue.great_characteristic", "params": { "characteristic": "characteristic.sta" } }
          ],
          "characteristics": { "sta": 3 },
          "aging_reductions": { "sta": 1 }
        }"#;
        let loaded =
            load_entity_migrating(old, DEFAULT_SAGA_YEAR, &ruleset, &empty_catalogue_names())
                .unwrap();
        let mut entity = loaded.entity;
        assert_eq!(
            crate::effective::effective_characteristic_score(
                &entity,
                &ruleset,
                Characteristic::Sta
            ),
            5,
            "the fixture's actual Stamina is +5"
        );
        assert_eq!(
            entity.aging_points.get(&Characteristic::Sta).copied(),
            Some(6)
        );
        assert_eq!(
            crate::effective::aging_drops(&entity, &ruleset, Characteristic::Sta),
            1
        );
        // Minimal: one point fewer forces no drop at all.
        entity.aging_points.insert(Characteristic::Sta, 5);
        assert_eq!(
            crate::effective::aging_drops(&entity, &ruleset, Characteristic::Sta),
            0
        );
    }

    /// A current save without `aging_reductions` migrates to a no-op report.
    #[test]
    fn current_save_migrates_without_aging_changes() {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.aging_points.insert(Characteristic::Sta, 4);
        let json = serde_json::to_string(&entity).unwrap();
        let loaded = load_entity_migrating(
            &json,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert!(loaded.migrated_aging_characteristics.is_empty());
        assert_eq!(
            loaded
                .entity
                .aging_points
                .get(&Characteristic::Sta)
                .copied(),
            Some(4)
        );
    }

    /// C8: the saga year is entity data from schema 17 on, and a save written
    /// before it inherits the default its CALLER supplies — the user's configured
    /// default for new documents, which this crate cannot read for itself.
    #[test]
    fn a_pre_17_save_takes_the_default_saga_year_it_is_handed() {
        let old = r#"{
          "schema_version": 16,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "age": 30,
          "birth_year": 1167
        }"#;
        // An Iberia saga, deliberately NOT the engine's 1220: what an old save
        // inherits is the year the user configured, not a constant.
        let loaded =
            load_entity_migrating(old, 1197, &empty_ruleset(), &empty_catalogue_names()).unwrap();
        assert_eq!(loaded.entity.saga_year, 1197);
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// C8 / round-trip fidelity, which CLAUDE.md rates top severity. A schema-16
    /// save migrates, re-saves and reloads **without the migrated year moving** —
    /// and the second load is handed a *different* default, so a `saga_year` that
    /// failed to reach the file would be caught rather than re-invented.
    ///
    /// Green on arrival: the migration and the always-write serialization that make
    /// it hold were both written for the test above. It is kept because it states
    /// the property that would break silently — a `skip_serializing_if` added later
    /// to `saga_year` would leave every other test passing and this one failing.
    #[test]
    fn a_migrated_saga_year_survives_a_save_and_reload_unchanged() {
        let old = r#"{
          "schema_version": 16,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "name": "Fiona",
          "age": 30,
          "birth_year": 1167
        }"#;
        let migrated = load_entity_migrating(old, 1197, &empty_ruleset(), &empty_catalogue_names())
            .unwrap()
            .entity;
        assert_eq!(migrated.saga_year, 1197);

        let saved = serde_json::to_string_pretty(&migrated).unwrap();
        assert!(saved.contains(r#""saga_year": 1197"#), "{saved}");

        // A different default: 1197 may now only come from the file.
        let reloaded =
            load_entity_migrating(&saved, 1000, &empty_ruleset(), &empty_catalogue_names())
                .unwrap()
                .entity;
        assert_eq!(reloaded.saga_year, 1197);
        assert_eq!(reloaded, migrated, "nothing was lost or invented");

        // And the second write is byte-identical to the first.
        assert_eq!(serde_json::to_string_pretty(&reloaded).unwrap(), saved);
    }

    /// C8 / Trap 3. The save file is this project's declared hostile-input surface,
    /// and `schema_version` is the attacker-controlled field in it. A save claiming
    /// a schema this build has never seen is **refused**, cleanly, rather than run
    /// through migrations written for an older shape: half-migrating it would stamp
    /// the current version onto a document whose unknown keys serde had already
    /// dropped, and the next save would make that permanent.
    ///
    /// No panic, no hang, no allocation: `u32::MAX` takes the same path as 18.
    #[test]
    fn a_save_from_a_future_schema_is_refused_rather_than_half_migrated() {
        for claimed in [SCHEMA_VERSION + 1, 999, u32::MAX] {
            let future = format!(
                r#"{{
                  "schema_version": {claimed},
                  "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
                  "entity_kind": "character",
                  "type_id": "companion",
                  "ability_funding": "pool",
                  "saga_year": 1197
                }}"#
            );
            let error = load_entity_migrating(
                &future,
                DEFAULT_SAGA_YEAR,
                &empty_ruleset(),
                &empty_catalogue_names(),
            )
            .expect_err("a save from the future must not be opened");
            let message = error.to_string();
            assert!(
                message.contains(&claimed.to_string()),
                "the refusal must name the version it read: {message}"
            );
        }

        // The boundary itself is fine: this build's own version opens.
        let current = format!(
            r#"{{
              "schema_version": {},
              "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
              "entity_kind": "character",
              "type_id": "companion",
              "ability_funding": "pool",
              "saga_year": 1197
            }}"#,
            SCHEMA_VERSION
        );
        let loaded = load_entity_migrating(
            &current,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect("this build's own version opens");
        assert_eq!(loaded.entity.saga_year, 1197);
    }

    /// Cluster A, root cause. `aura` is the only unbounded signed field on
    /// [`Entity`], and [`Entity::normalize`] — the one thing that clamps it — runs
    /// on **save**, never on load. So an out-of-range value from a file reached the
    /// whole derived layer untouched, where it both aborted the process on an
    /// unguarded `+` and, once that was guarded, produced a saturated and
    /// meaningless read-out on every casting, lab and penetration line.
    ///
    /// Clamping belongs here beside [`trim_all_selection_params`] and
    /// [`fold_legacy_being_params`]: value-driven, idempotent and version-free, so
    /// like those two it stamps **no** [`SCHEMA_VERSION`] — the document's shape is
    /// current, only one of its values was out of range.
    #[test]
    fn an_out_of_range_aura_is_clamped_on_load() {
        for (stored, expected) in [
            (i32::MAX, AURA_MODIFIER_MAX),
            (i32::MIN, AURA_MODIFIER_MIN),
            (AURA_MODIFIER_MAX + 1, AURA_MODIFIER_MAX),
            (AURA_MODIFIER_MIN - 1, AURA_MODIFIER_MIN),
        ] {
            let json = format!(
                r#"{{
                  "schema_version": {SCHEMA_VERSION},
                  "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
                  "entity_kind": "character",
                  "type_id": "magus",
                  "ability_funding": "pool",
                  "saga_year": 1197,
                  "aura": {stored}
                }}"#
            );
            let loaded = load_entity_migrating(
                &json,
                DEFAULT_SAGA_YEAR,
                &empty_ruleset(),
                &empty_catalogue_names(),
            )
            .expect("an out-of-range aura is clamped, not refused");
            assert_eq!(
                loaded.entity.aura, expected,
                "aura {stored} must load clamped to {expected}"
            );
            assert_eq!(
                loaded.entity.schema_version, SCHEMA_VERSION,
                "a value-driven clamp stamps no new version"
            );
        }
    }

    /// The other half of the clamp: a legal aura is data, not something to
    /// normalize. It must survive the load untouched and re-serialize
    /// byte-identically, or opening a file would silently rewrite it.
    #[test]
    fn a_legal_aura_round_trips_unchanged() {
        let json = format!(
            r#"{{
              "schema_version": {SCHEMA_VERSION},
              "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
              "entity_kind": "character",
              "type_id": "magus",
              "ability_funding": "pool",
              "saga_year": 1197,
              "aura": 5
            }}"#
        );
        let loaded = load_entity_migrating(
            &json,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        assert_eq!(loaded.aura, 5);

        let saved = serde_json::to_string_pretty(&loaded).unwrap();
        assert!(saved.contains(r#""aura": 5"#), "{saved}");
        let reloaded = load_entity_migrating(
            &saved,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        assert_eq!(reloaded, loaded, "nothing was lost or invented");
        assert_eq!(serde_json::to_string_pretty(&reloaded).unwrap(), saved);
    }

    /// Both boundary values are legal and must not be moved: the clamp is
    /// inclusive, so `AURA_MODIFIER_MIN` (a Dominion aura at its worst) and
    /// `AURA_MODIFIER_MAX` load exactly as stored.
    #[test]
    fn the_aura_clamp_boundaries_are_inclusive() {
        for legal in [AURA_MODIFIER_MIN, AURA_MODIFIER_MAX, 0] {
            let json = format!(
                r#"{{
                  "schema_version": {SCHEMA_VERSION},
                  "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
                  "entity_kind": "character",
                  "type_id": "magus",
                  "ability_funding": "pool",
                  "saga_year": 1197,
                  "aura": {legal}
                }}"#
            );
            let loaded = load_entity_migrating(
                &json,
                DEFAULT_SAGA_YEAR,
                &empty_ruleset(),
                &empty_catalogue_names(),
            )
            .unwrap();
            assert_eq!(loaded.entity.aura, legal, "a legal aura is left alone");
        }
    }

    /// A legacy save carrying the flat `talisman_attunements` list migrates into
    /// `Entity.talisman`: the attunements are carried over verbatim (nothing is
    /// inferred, unlike the aging fold), the legacy key is dropped, and the schema
    /// is bumped to 14. A list that cannot deserialize instead fails the load — see
    /// `legacy_talisman_attunements_that_cannot_deserialize_fail_the_load`.
    #[test]
    fn legacy_talisman_attunements_migrate_into_talisman() {
        let old = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": [
            { "description": "Projecting bolts and missiles", "bonus": 3 },
            { "description": "Controlling things at a distance", "bonus": 4 }
          ]
        }"#;
        let loaded = load_entity_migrating(
            old,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        let talisman = loaded
            .entity
            .talisman
            .as_ref()
            .expect("legacy attunements become a talisman");
        assert_eq!(talisman.attunements.len(), 2);
        assert_eq!(talisman.attunements[0].bonus, 3);
        // Only the attunements are known; the item's identity and instilled
        // effects were never stored, so they stay empty rather than invented.
        assert_eq!(talisman.description, "");
        assert!(talisman.effects.is_empty());
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
        // Re-serializing carries no legacy key.
        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("talisman_attunements"), "{json}");
    }

    /// An empty legacy list is not a talisman: no phantom item is created. The
    /// schema is still bumped, because the key's presence proves the save was
    /// written in the old shape (the `aging_reductions` precedent). App-written
    /// saves can never carry an empty list (`skip_serializing_if`), so this only
    /// covers a hand-edited file.
    #[test]
    fn legacy_empty_talisman_attunements_migrate_to_no_talisman() {
        let old = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": []
        }"#;
        let loaded = load_entity_migrating(
            old,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert!(loaded.entity.talisman.is_none(), "no phantom talisman");
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// A hand-edited save carrying BOTH shapes, where the new `talisman` already has
    /// `attunements`: the new one wins and the legacy list is dropped unmerged.
    /// Merging would silently duplicate attunements the player already moved across by
    /// hand — which is why populated `attunements`, and only that, is what the early
    /// return gates on (see
    /// `legacy_attunements_fold_into_a_talisman_that_has_only_a_description`).
    ///
    /// The legacy row here is deliberately one that **cannot** deserialize (`bonus`
    /// far outside `i8`), which pins the *order of operations* too: the new shape's
    /// presence is decided before the legacy value is parsed, so its shape genuinely
    /// cannot matter. Parsing first would make this save fail to load — a regression
    /// from a load that succeeds today — and this `unwrap` would catch it.
    #[test]
    fn hand_edited_save_with_both_talisman_shapes_keeps_the_new_one() {
        let both = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman": {
            "description": "An ash staff",
            "attunements": [{ "description": "Warding", "bonus": 5 }]
          },
          "talisman_attunements": [{ "description": "Stale", "bonus": 3000 }]
        }"#;
        let loaded = load_entity_migrating(
            both,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        let talisman = loaded.entity.talisman.as_ref().expect("talisman kept");
        assert_eq!(talisman.description, "An ash staff");
        assert_eq!(talisman.attunements.len(), 1, "legacy row not merged in");
        assert_eq!(talisman.attunements[0].description, "Warding");
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// A hand-edited save carrying an **empty** `"talisman": {}` beside a legacy
    /// list must still fold the attunements in. Every `Talisman` field is
    /// `skip_serializing_if`, so an app-written talisman the user never filled in
    /// serializes as exactly `{}` — the presence of the key therefore proves
    /// nothing, and treating it as "the new shape wins" would drop the legacy list
    /// unparsed while the load still stamps `SCHEMA_VERSION`. The next save would
    /// then rewrite the file without the legacy key: the same permanent silent loss
    /// the error path is built to prevent, reached through the both-keys path.
    ///
    /// The both-keys early return therefore gates on the new shape already carrying
    /// *attunements*, not merely existing — see
    /// `hand_edited_save_with_both_talisman_shapes_keeps_the_new_one` for the
    /// populated case, which still wins.
    #[test]
    fn legacy_attunements_fold_into_an_empty_new_talisman() {
        let both = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman": {},
          "talisman_attunements": [{ "description": "Warding", "bonus": 5 }]
        }"#;
        let loaded = load_entity_migrating(
            both,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        let talisman = loaded
            .entity
            .talisman
            .as_ref()
            .expect("an empty talisman plus a legacy list keeps the attunements");
        assert_eq!(
            talisman.attunements.len(),
            1,
            "the legacy list is folded into the empty talisman, not dropped"
        );
        assert_eq!(talisman.attunements[0].description, "Warding");
        assert_eq!(talisman.attunements[0].bonus, 5);
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("talisman_attunements"), "{json}");
    }

    /// A new-shape talisman that carries data in a field the fold cannot touch — here
    /// only a `description`, no attunements — must **still** take the legacy list.
    /// `attunements` is the sole field the legacy `talisman_attunements` key can
    /// migrate into, so it is the only field whose contents can make merging
    /// duplicate anything. Gating on the whole `Talisman` differing from
    /// `Talisman::default()` would drop this list unparsed while the load still
    /// stamps `SCHEMA_VERSION`, and the next save would rewrite the file without the
    /// legacy key: permanent silent loss.
    ///
    /// The fold must also keep the talisman's own data — the description survives
    /// alongside the migrated attunement, because the list is folded *into* the
    /// existing item rather than replacing it.
    #[test]
    fn legacy_attunements_fold_into_a_talisman_that_has_only_a_description() {
        let both = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman": { "description": "An ash staff" },
          "talisman_attunements": [{ "description": "Warding", "bonus": 5 }]
        }"#;
        let loaded = load_entity_migrating(
            both,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        let talisman = loaded
            .entity
            .talisman
            .as_ref()
            .expect("a described talisman plus a legacy list keeps both");
        assert_eq!(
            talisman.description, "An ash staff",
            "the talisman keeps its own data; the fold does not replace the item"
        );
        assert_eq!(
            talisman.attunements.len(),
            1,
            "the legacy list is folded in, not dropped: `attunements` was empty"
        );
        assert_eq!(talisman.attunements[0].description, "Warding");
        assert_eq!(talisman.attunements[0].bonus, 5);
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("talisman_attunements"), "{json}");
    }

    /// An empty `"talisman": {}` beside an **empty** legacy list stays no-talisman:
    /// the fold invents nothing, so nothing turns the empty item into a filled one.
    /// (An empty legacy list alone is pinned by
    /// `legacy_empty_talisman_attunements_migrate_to_no_talisman`; here the empty
    /// new-shape key is preserved as written, since the fold has nothing to add.)
    #[test]
    fn an_empty_talisman_beside_an_empty_legacy_list_gains_nothing() {
        let both = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman": {},
          "talisman_attunements": []
        }"#;
        let loaded = load_entity_migrating(
            both,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(
            loaded.entity.talisman,
            Some(Talisman::default()),
            "the empty talisman is kept as written; nothing is invented"
        );
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// A legacy `talisman_attunements` list that cannot deserialize fails the load
    /// **loudly**. Silently folding it into an empty list would create no talisman
    /// while the caller still stamps the current `SCHEMA_VERSION`, so the next save
    /// would rewrite the file without the legacy key and the attunements would be
    /// gone for good. A failed load leaves the file on disk untouched.
    ///
    /// Each case asserts **what** the error says and pairs the malformed fixture with
    /// a byte-identical **positive control** whose one offending value is corrected.
    /// Together they attribute the failure to the legacy row itself: a bare
    /// `is_err()` would stay green if the surrounding document had merely stopped
    /// parsing for an unrelated reason (a newly required `Entity` field, a renamed
    /// `ruleset` shape) while the fold quietly reverted to `unwrap_or_default()`.
    #[test]
    fn legacy_talisman_attunements_that_cannot_deserialize_fail_the_load() {
        // `bonus` is out of `i8` range, so `TalismanAttunement` cannot deserialize.
        let broken = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": [
            { "description": "Projecting bolts and missiles", "bonus": 3000 }
          ]
        }"#;
        let err = load_entity_migrating(
            broken,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect_err("a malformed legacy attunement list must not load as an empty talisman")
        .to_string();
        assert!(err.contains("3000") && err.contains("i8"), "{err}");
        // Positive control: the same document with the bonus in range loads, so the
        // failure above is the legacy row and nothing else.
        let fixed = broken.replace("3000", "3");
        let loaded = load_entity_migrating(
            &fixed,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect("the in-range twin loads");
        assert_eq!(
            loaded
                .entity
                .talisman
                .expect("folded into a talisman")
                .attunements[0]
                .bonus,
            3
        );

        // The same for a bonus written as a JSON string, and for a null list.
        let stringly = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": [{ "description": "Warding", "bonus": "5" }]
        }"#;
        let err = load_entity_migrating(
            stringly,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect_err("a stringly-typed bonus must not load")
        .to_string();
        assert!(err.contains("i8"), "{err}");
        let fixed = stringly.replace("\"5\"", "5");
        load_entity_migrating(
            &fixed,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect("the numeric twin loads");

        let nulled = r#"{
          "schema_version": 13,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "talisman_attunements": null
        }"#;
        let err = load_entity_migrating(
            nulled,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect_err("a null legacy list must not load")
        .to_string();
        assert!(err.contains("null") && err.contains("sequence"), "{err}");
        let fixed = nulled.replace("null", "[]");
        load_entity_migrating(
            &fixed,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect("the empty-list twin loads");
    }

    /// The same guarantee for the older `aging_reductions` fold: a legacy map that
    /// cannot deserialize (an out-of-`u8` drop count, an unknown Characteristic key)
    /// fails the load instead of folding in nothing and bumping the version.
    ///
    /// As above, each case checks the error text and is paired with a corrected
    /// positive control, so the failure is provably the legacy map's and not the
    /// surrounding document's.
    #[test]
    fn legacy_aging_reductions_that_cannot_deserialize_fail_the_load() {
        let out_of_range = r#"{
          "schema_version": 9,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "characteristics": { "com": 2 },
          "aging_reductions": { "com": 300 }
        }"#;
        let err = load_entity_migrating(
            out_of_range,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect_err("an out-of-u8 drop count must not load")
        .to_string();
        assert!(err.contains("300") && err.contains("u8"), "{err}");
        let fixed = out_of_range.replace("300", "1");
        let loaded = load_entity_migrating(
            &fixed,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect("the in-range twin loads");
        assert_eq!(
            loaded.migrated_aging_characteristics,
            vec![Characteristic::Com]
        );

        let unknown_key = r#"{
          "schema_version": 9,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "aging_reductions": { "cun": 1 }
        }"#;
        let err = load_entity_migrating(
            unknown_key,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect_err("an unknown Characteristic key must not load")
        .to_string();
        assert!(err.contains("cun"), "{err}");
        // Positive control: `int` is a real Characteristic, so the twin loads — the
        // Creature Format's Cunning score is deliberately not a variant (see
        // `Familiar::characteristics`).
        let fixed = unknown_key.replace("cun", "int");
        let loaded = load_entity_migrating(
            &fixed,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .expect("the known-key twin loads");
        assert_eq!(
            loaded.migrated_aging_characteristics,
            vec![Characteristic::Int]
        );
    }

    /// The 14 → 15 bump shipped **no migration code** of its own: a schema-14 save is
    /// already a valid schema-15 document, because serde reads a bare `year` number
    /// into `Some` and every widened field defaults. Only the *forward* direction
    /// broke — a schema-15 save may omit `year` entirely, which a schema-14 reader
    /// rejects — and that is what earned the bump. This test pins that the aging log
    /// still loads verbatim.
    ///
    /// **Schema 16 does have a fold**, and it is what now stamps this save: the file
    /// carries no `ability_funding` key, so the funding mode is inferred (no plan →
    /// `Pool`) and the version is stamped, exactly as the `aging_reductions` and
    /// talisman folds do. The aging log itself is still untouched.
    #[test]
    fn a_schema_fourteen_save_loads_without_migration() {
        assert_eq!(SCHEMA_VERSION, 21);
        let schema_14 = r#"{
          "schema_version": 14,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "magus",
          "apparent_age": 45,
          "aging_log": [{ "year": 1220, "effect": "Lost a point of Stamina" }]
        }"#;
        let loaded = load_entity_migrating(
            schema_14,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert!(
            loaded.migrated_aging_characteristics.is_empty(),
            "the bump migrates nothing"
        );
        assert_eq!(loaded.entity.aging_log.len(), 1);
        assert_eq!(loaded.entity.aging_log[0].year, Some(1220));
        assert_eq!(loaded.entity.aging_log[0].effect, "Lost a point of Stamina");
        assert_eq!(loaded.entity.aging_log[0].age, None);
        assert_eq!(loaded.entity.apparent_age, Some(45));
        // Nothing in the aging log was migrated, but the missing `ability_funding` key
        // is schema 16's fold, so the version IS stamped. The mode it infers is `Pool`:
        // no `life_stages` plan means the typed pool was the authority.
        assert_eq!(loaded.entity.ability_funding, AbilityFunding::Pool);
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// The current-version pin, advanced at every bump (most recently 20 → 21 for
    /// X9b's `virtue.rard` -> `virtue.bard` id rename — see [`SCHEMA_VERSION`]'s own
    /// doc comment for why that bump owns a fold, unlike C5a's 18 → 19 pure marker).
    #[test]
    fn schema_version_is_21() {
        assert_eq!(SCHEMA_VERSION, 21);
    }

    /// C5a's bump is a **pure version marker**: no shape moved, so no fold
    /// dispatches on anything for it. A save already at schema 18, with nothing
    /// any OTHER fold above would touch either, therefore loads completely
    /// unchanged — its recorded `schema_version` stays exactly what the file
    /// said (18) rather than being bumped in memory, precisely like every other
    /// marker-only bump above (10 → 11, 11 → 12, 12 → 13): it becomes 19 only on
    /// the next save, via `arm-app`'s unconditional stamp
    /// (`ruleset_io.rs::save_entity_to_path`).
    #[test]
    fn a_clean_schema_18_save_loads_unchanged_under_the_19_build() {
        let json = r#"{
          "schema_version": 18,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "xp_pool": 240
        }"#;
        let loaded = load_entity_migrating(
            json,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(
            loaded.entity.schema_version, 18,
            "no fold exists for 18 -> 19 (a pure version marker), so a save with \
             nothing else to migrate keeps its recorded version until the next save"
        );
        assert_eq!(loaded.entity.xp_pool, 240);
        assert_eq!(loaded.entity.saga_year, 1220);
    }

    /// A save written before the funding discriminator existed carries a
    /// `life_stages` plan and no `ability_funding` key. The mode used to *be* the
    /// plan's presence, so the fold applies that rule once at load: a plan means
    /// life-stage funding.
    #[test]
    fn entity_without_ability_funding_migrates_from_life_stages_presence() {
        let old = r#"{
          "schema_version": 15,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "age": 25,
          "life_stages": { "native_language": "German" }
        }"#;
        let loaded = load_entity_migrating(
            old,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(loaded.entity.ability_funding, AbilityFunding::LifeStages);
        assert!(loaded.entity.life_stages.is_some(), "the plan is kept");
        // A fold happened, so the version is stamped.
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// The other half of the same rule: no plan means the typed pool was the
    /// authority, so the mode folds to `Pool`.
    #[test]
    fn entity_without_ability_funding_or_plan_migrates_to_pool() {
        let old = r#"{
          "schema_version": 15,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "xp_pool": 240
        }"#;
        let loaded = load_entity_migrating(
            old,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(loaded.entity.ability_funding, AbilityFunding::Pool);
        assert_eq!(loaded.entity.xp_pool, 240, "the typed pool is kept");
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }

    /// A hand-written save in exactly the shape v0.2.x wrote (`schema_version` 14,
    /// no `ability_funding` key), holding every case the `being` fold has to decide:
    ///
    /// - `flaw.offensive_to_beings` — a typed **English** label;
    /// - `flaw.unbearable_to_beings` — a typed **German** label, with stray
    ///   whitespace and the wrong case, since a text box accepted anything;
    /// - `virtue.inoffensive_to_beings` — a value that is **already** an id, i.e. a
    ///   save someone re-picked by hand, or the second load of a migrated one;
    /// - `virtue.alluring_to_beings` — an English *being* label on an item whose
    ///   `being` parameter is **still** free text today, so the fold must not touch
    ///   it however well the value matches;
    /// - `flaw.slow_power` — paramless, because v0.2.x declared no `power`;
    /// - `virtue.folk_magic` — paramless, because v0.2.x declared no `category`.
    ///
    /// Deliberately not round-tripped through `Entity`: a save the current code
    /// wrote could never carry the legacy shapes this fixture exists to exercise.
    const V0_2_X_SAVE: &str = r#"{
      "schema_version": 14,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "magus",
      "name": "Iohannes filius Bonisagi",
      "xp_pool": 240,
      "selections": [
        { "ref": "flaw.offensive_to_beings", "params": { "being": "Mundane Humans" } },
        { "ref": "flaw.unbearable_to_beings", "params": { "being": "  dämonen " } },
        { "ref": "virtue.inoffensive_to_beings", "params": { "being": "being.faeries" } },
        { "ref": "virtue.alluring_to_beings", "params": { "being": "Faeries" } },
        { "ref": "flaw.slow_power" },
        { "ref": "virtue.folk_magic" }
      ]
    }"#;

    /// The `being` value the loaded fixture holds for `item_ref`.
    fn being_param(entity: &Entity, item_ref: &str) -> String {
        entity
            .selections
            .iter()
            .find(|selection| selection.item_ref.as_str() == item_ref)
            .and_then(|selection| selection.params.get("being"))
            .and_then(SelectionParamValue::as_single)
            .map(|value| value.as_str().to_string())
            .unwrap_or_else(|| panic!("{item_ref} must be in the fixture with a `being` param"))
    }

    /// The three items whose `being` parameter became an enumerated domain carry a
    /// typed label in every v0.2.x save. The fold maps the labels the app and the
    /// rulebook printed — in **both** shipped languages — onto the `being.*` ids,
    /// case- and whitespace-insensitively.
    #[test]
    fn a_v0_2_x_save_migrates_its_typed_being_values_in_both_languages() {
        let loaded = load_entity_migrating(
            V0_2_X_SAVE,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        let entity = &loaded.entity;

        assert_eq!(
            being_param(entity, "flaw.offensive_to_beings"),
            "being.mundane_humans"
        );
        assert_eq!(
            being_param(entity, "flaw.unbearable_to_beings"),
            "being.demons",
            "a German label, mis-cased and padded, is still the same choice"
        );
    }

    /// German `ArMDE:4135` — the Inoffensive entry — prints its list of classes in the
    /// **dative** ("Tieren, göttlichen Wesen, … magischen Kreaturen"), so a player
    /// copying the class straight off the page typed an inflected form. Those three
    /// forms are printed by the source, not invented, and all three name a class
    /// `virtue.inoffensive_to_beings` actually declares — which is why this test
    /// exercises them on exactly that item.
    #[test]
    fn the_german_dative_forms_the_rulebook_prints_fold_too() {
        for (typed, expected) in [
            ("Tieren", "being.animals"),
            ("göttlichen Wesen", "being.divine"),
            // Mis-cased and padded, so the dative keys go through the same
            // case/whitespace folding as every other key.
            ("  MAGISCHEN   KREATUREN ", "being.magical_creatures"),
        ] {
            let json = format!(
                r#"{{
                  "schema_version": 14,
                  "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
                  "entity_kind": "character",
                  "type_id": "magus",
                  "selections": [
                    {{ "ref": "virtue.inoffensive_to_beings", "params": {{ "being": "{typed}" }} }}
                  ]
                }}"#
            );
            let loaded = load_entity_migrating(
                &json,
                DEFAULT_SAGA_YEAR,
                &empty_ruleset(),
                &empty_catalogue_names(),
            )
            .unwrap();
            assert_eq!(
                being_param(&loaded.entity, "virtue.inoffensive_to_beings"),
                expected,
                "the dative form '{typed}' printed at Core Rules (de):4135 must fold"
            );
        }
    }

    /// The property the whole fold rests on, asserted rather than eyeballed: no two
    /// frozen labels collapse onto one another under [`fold_being_label`], and no
    /// label is ever equal to a `being.*` id. The first makes the mapping
    /// unambiguous; the second makes it idempotent, since an already-migrated value
    /// can then never be a key.
    #[test]
    fn every_frozen_being_label_is_distinct_and_is_never_an_id() {
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (label, id) in LEGACY_BEING_LABELS {
            let folded = fold_being_label(label);
            if let Some(previous) = seen.insert(folded.clone(), label) {
                panic!("'{label}' and '{previous}' fold onto the same key '{folded}'");
            }
            assert!(
                !label.starts_with("being."),
                "'{label}' is an id, not a label, so the fold would stop being idempotent"
            );
            assert!(
                fold_being_label(id) != folded,
                "'{label}' folds onto its own id '{id}'"
            );
        }
        assert_eq!(
            seen.len(),
            LEGACY_BEING_LABELS.len(),
            "every frozen label must survive folding as its own key"
        );
    }

    /// A value that is already an id is not in the legacy map, so it is left exactly
    /// as written — which is also what makes the fold idempotent.
    #[test]
    fn an_already_migrated_being_value_is_left_alone() {
        let loaded = load_entity_migrating(
            V0_2_X_SAVE,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(
            being_param(&loaded.entity, "virtue.inoffensive_to_beings"),
            "being.faeries"
        );
    }

    /// `virtue.alluring_to_beings` and `flaw.magical_being_companion` also carry a
    /// `being` parameter, and both are **still** `text` domains — their value is a
    /// player's free description, not one of the closed lists. Folding them would
    /// silently rewrite a legal value into an id the item's domain never declared,
    /// so the fold is keyed on the three items that actually changed.
    #[test]
    fn a_being_param_that_is_still_free_text_is_never_folded() {
        let loaded = load_entity_migrating(
            V0_2_X_SAVE,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(
            being_param(&loaded.entity, "virtue.alluring_to_beings"),
            "Faeries",
            "a text-domain being value stays exactly as the player typed it"
        );
    }

    /// A typed value the frozen map does not recognise is left exactly as written,
    /// on every one of the three folded items. The player re-picks it once; nothing
    /// is guessed on their behalf.
    #[test]
    fn an_unrecognised_being_value_is_left_exactly_as_typed() {
        for item_ref in [
            "flaw.offensive_to_beings",
            "flaw.unbearable_to_beings",
            "virtue.inoffensive_to_beings",
        ] {
            let json = format!(
                r#"{{
                  "schema_version": 14,
                  "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
                  "entity_kind": "character",
                  "type_id": "magus",
                  "selections": [
                    {{ "ref": "{item_ref}", "params": {{ "being": "dragons" }} }}
                  ]
                }}"#
            );
            let loaded = load_entity_migrating(
                &json,
                DEFAULT_SAGA_YEAR,
                &empty_ruleset(),
                &empty_catalogue_names(),
            )
            .unwrap();
            assert_eq!(being_param(&loaded.entity, item_ref), "dragons");
        }
    }

    /// The two choices v0.2.x never stored stay unfilled: `flaw.slow_power` has no
    /// `power` and `virtue.folk_magic` no `category`. There is nothing to migrate
    /// *from*, so inventing a placeholder would invent a rules choice — the visible
    /// `missing_param` is the correct outcome and the player supplies it once.
    #[test]
    fn a_choice_the_old_save_never_stored_is_not_invented() {
        let loaded = load_entity_migrating(
            V0_2_X_SAVE,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        for item_ref in ["flaw.slow_power", "virtue.folk_magic"] {
            let selection = loaded
                .entity
                .selections
                .iter()
                .find(|selection| selection.item_ref.as_str() == item_ref)
                .expect("the fixture holds it");
            assert!(
                selection.params.is_empty(),
                "{item_ref} must stay unfilled, not carry a made-up value: {:?}",
                selection.params
            );
        }
    }

    /// No `schema_version` distinguishes a v0.2.x save from a current one — the
    /// `being` lists were a **ruleset** change, not a save-format one — so the fold
    /// is value-driven and must be idempotent. Loading a migrated save again changes
    /// nothing.
    #[test]
    fn migrating_a_v0_2_x_save_twice_changes_nothing() {
        let once = load_entity_migrating(
            V0_2_X_SAVE,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        let json = serde_json::to_string(&once).unwrap();
        let twice = load_entity_migrating(
            &json,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        assert_eq!(once, twice);
    }

    /// A migrated save must not churn on every open: after the fold and
    /// `normalize()`, a save→load→save cycle is byte-identical. (`normalize` sorts
    /// selections by `(ref, params)`, so a fold that changed a param value could in
    /// principle reorder rows — this pins that the reordering settles at the first
    /// save rather than repeating.)
    #[test]
    fn a_migrated_save_is_byte_stable_across_a_save_load_save_cycle() {
        let mut first = load_entity_migrating(
            V0_2_X_SAVE,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        first.normalize();
        let first_bytes = serde_json::to_string_pretty(&first).unwrap();

        let mut second = load_entity_migrating(
            &first_bytes,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        second.normalize();
        let second_bytes = serde_json::to_string_pretty(&second).unwrap();

        assert_eq!(first_bytes, second_bytes);
    }

    /// A save carrying padded parameter values, in every place a [`Selection`] can
    /// live. Written by hand for the same reason [`V0_2_X_SAVE`] is: the current code
    /// trims on load, so it can no longer *produce* the shape under test.
    ///
    /// `flaw.lesser_power` and `virtue.minor_magical_focus` are the case row 10 is
    /// about — a padded free-text descriptor is the same choice as an unpadded one.
    /// `virtue.puissant_ability` shows that a padded *id* is recovered too, and
    /// `flaw.slow_power` holds a whitespace-only value, i.e. a box the player left
    /// blank.
    const PADDED_PARAMS_SAVE: &str = r#"{
      "schema_version": 16,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "magus",
      "ability_funding": "pool",
      "selections": [
        { "ref": "flaw.lesser_power", "params": { "power": "  Wolf Shape " } },
        { "ref": "virtue.academic_concentration", "params": { "subject": "\tTheology\n" } },
        { "ref": "virtue.puissant_ability", "params": { "ability": " ability.awareness " } },
        { "ref": "flaw.slow_power", "params": { "power": "   " } }
      ],
      "house_choices": {
        "virtue.bjornaer_heartbeast": { "ref": "virtue.minor_magical_focus", "params": { "focus": " Wolf Shape " } }
      },
      "mythic_choices": {
        "virtue.mythic_blood": { "ref": "virtue.puissant_art", "params": { "art": "art.ignem " } }
      },
      "warping_choices": {
        "0": { "ref": "flaw.magical_air", "params": { "being": " Faeries" } }
      }
    }"#;

    /// The value `item_ref`'s `key` holds in the loaded fixture's bought list.
    fn param_of(entity: &Entity, item_ref: &str, key: &str) -> String {
        entity
            .selections
            .iter()
            .find(|selection| selection.item_ref.as_str() == item_ref)
            .and_then(|selection| selection.params.get(key))
            .and_then(SelectionParamValue::as_single)
            .map(|value| value.as_str().to_string())
            .unwrap_or_else(|| panic!("{item_ref} must be in the fixture with a `{key}` param"))
    }

    /// Row 10: parameter values are compared byte-for-byte everywhere that matters —
    /// the duplicate-selection key, a grant's `options.contains(pick)`, a mythic
    /// type's required-Virtue check, `normalize()`'s sort — so "Wolf Shape " and
    /// "Wolf Shape" were two different powers. Nothing trimmed a value arriving from
    /// a save file, so the fix lands **at load**: every param value in every place a
    /// selection can live is trimmed before the entity reaches anyone.
    #[test]
    fn load_trims_the_whitespace_around_every_param_value() {
        let entity = load_entity_migrating(
            PADDED_PARAMS_SAVE,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;

        assert_eq!(
            param_of(&entity, "flaw.lesser_power", "power"),
            "Wolf Shape"
        );
        assert_eq!(
            param_of(&entity, "virtue.academic_concentration", "subject"),
            "Theology",
            "tabs and newlines are whitespace too"
        );
        assert_eq!(
            param_of(&entity, "virtue.puissant_ability", "ability"),
            "ability.awareness",
            "a padded id resolves against its catalogue again"
        );
        assert_eq!(
            param_of(&entity, "flaw.slow_power", "power"),
            "",
            "a blank box stays blank — and is reported as `missing_param`, not filled in"
        );

        for (map, key, param, expected) in [
            (
                &entity.house_choices,
                "virtue.bjornaer_heartbeast",
                "focus",
                "Wolf Shape",
            ),
            (
                &entity.mythic_choices,
                "virtue.mythic_blood",
                "art",
                "art.ignem",
            ),
            (&entity.warping_choices, "0", "being", "Faeries"),
        ] {
            assert_eq!(
                map.get(key)
                    .and_then(|selection| selection.params.get(param))
                    .and_then(SelectionParamValue::as_single)
                    .map(|value| value.as_str()),
                Some(expected),
                "a resolved pick's params are trimmed too ({key})"
            );
        }
    }

    /// The trim has no version signal either (a padded value is legal JSON at any
    /// `schema_version`), so it is value-driven like the `being` fold: a second load
    /// changes nothing, and a save→load→save cycle is byte-identical. Trimming
    /// reorders `normalize()`'s selection sort, so this pins that the reordering
    /// settles at the first save instead of churning the file on every open.
    #[test]
    fn trimming_params_at_load_is_idempotent_and_byte_stable() {
        let mut first = load_entity_migrating(
            PADDED_PARAMS_SAVE,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        first.normalize();
        let first_bytes = serde_json::to_string_pretty(&first).unwrap();

        let mut second = load_entity_migrating(
            &first_bytes,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        second.normalize();
        let second_bytes = serde_json::to_string_pretty(&second).unwrap();

        assert_eq!(first, second);
        assert_eq!(first_bytes, second_bytes);
    }

    // --- F1/K5: `EquipmentSlot::equipped` (bool) -> `EquipmentSlot::loadout`
    // (`LoadoutState`) — design-f0-book-template-engine.md § 6. RED CHECKPOINT:
    // the fold itself is not implemented yet (green phase); these pin the fold's
    // eventual contract, per § 6's own four-fixture enumeration.

    /// The core red (§ 6 item 2): a legacy `"equipped": true` must become
    /// `loadout: Wielded`. Before the fold exists, `loadout` is filled only by its
    /// own `#[serde(default)]` (`Stowed`) and the legacy key is silently dropped
    /// (no `deny_unknown_fields`) — so this assertion fails until the fold
    /// translates it.
    #[test]
    fn legacy_equipped_true_migrates_to_wielded_loadout() {
        let old = r#"{
          "schema_version": 19,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "equipment": [
            { "item": "weapon.sword_long", "equipped": true }
          ]
        }"#;
        let loaded = load_entity_migrating(
            old,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(loaded.entity.equipment.len(), 1);
        assert_eq!(loaded.entity.equipment[0].loadout, LoadoutState::Wielded);
        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);

        // The legacy key never survives the fold into a re-saved document.
        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("equipped"), "got {json}");
    }

    /// Control case (§ 6): `"equipped": false`, or the key absent entirely,
    /// already deserializes to `Stowed` via `loadout`'s own `#[serde(default)]` —
    /// no fold is needed for the VALUE to come out right. The note's own words
    /// (§ 6, "What an old save becomes"): "every equipped-`true` slot →
    /// `loadout: "wielded"` (identical derived figures); every
    /// equipped-`false`-or-absent slot → `loadout: "stowed"` (identical derived
    /// figures — the post-K2 behavior)." Already green today for the VALUE;
    /// kept so a future refactor of the fold cannot flip it. The re-serialized
    /// document must also carry no stray `equipped` key, matching the same
    /// requirement pinned for the `true` case in
    /// [`legacy_equipped_true_migrates_to_wielded_loadout`].
    #[test]
    fn legacy_equipped_false_or_absent_already_defaults_to_stowed() {
        for equipped_json in ["\"equipped\": false,", ""] {
            let old = format!(
                r#"{{
                  "schema_version": 19,
                  "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
                  "entity_kind": "character",
                  "type_id": "companion",
                  "ability_funding": "pool",
                  "saga_year": 1220,
                  "equipment": [
                    {{ "item": "weapon.sword_long", {equipped_json} "specialization_applies": false }}
                  ]
                }}"#
            );
            let loaded = load_entity_migrating(
                &old,
                DEFAULT_SAGA_YEAR,
                &empty_ruleset(),
                &empty_catalogue_names(),
            )
            .unwrap();
            assert_eq!(
                loaded.entity.equipment[0].loadout,
                LoadoutState::Stowed,
                "input {old}"
            );
            let json = serde_json::to_string(&loaded.entity).unwrap();
            assert!(!json.contains("equipped"), "got {json}");
        }
    }

    /// Control case (§ 6, third fixture): a document already at the current shape
    /// (`loadout` present) is untouched — the eventual fold's
    /// `obj.contains_key("loadout")` short-circuit, matching every sibling fold's
    /// own idempotence. Already green today (plain deserialization, no fold
    /// involved).
    #[test]
    fn a_document_already_carrying_loadout_is_left_untouched() {
        let current = r#"{
          "schema_version": 20,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "equipment": [
            { "item": "weapon.sword_great", "loadout": "carried" }
          ]
        }"#;
        let loaded = load_entity_migrating(
            current,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(loaded.entity.equipment[0].loadout, LoadoutState::Carried);
        assert_eq!(loaded.entity.schema_version, 20);
    }

    /// The "both keys present" case (§ 6, fourth fixture): a hand-edited or
    /// partially migrated save carrying BOTH `equipped` and `loadout`. The eventual
    /// fold dispatches on `loadout`'s presence, never `equipped`'s, so this is a
    /// no-op: `loadout` keeps its own value, unaffected by the stray legacy key,
    /// which is dropped on the next save. Already green today (plain
    /// deserialization already reads `loadout` and silently drops the unrecognized
    /// `equipped`).
    #[test]
    fn a_document_carrying_both_legacy_and_current_keys_keeps_the_current_value() {
        let hand_edited = r#"{
          "schema_version": 20,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "equipment": [
            { "item": "weapon.sword_great", "equipped": true, "loadout": "carried" }
          ]
        }"#;
        let loaded = load_entity_migrating(
            hand_edited,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(loaded.entity.equipment[0].loadout, LoadoutState::Carried);

        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("\"equipped\""), "got {json}");
    }

    /// The crafted/untrusted-`schema_version` case: a save dishonestly claims the
    /// CURRENT version (implying "nothing to migrate") while still carrying the
    /// legacy `equipped` shape. Dispatch is on the legacy key's absence, never on
    /// the recorded version — this project's own governing rule for every fold —
    /// so a claimed-current version must not skip the equipment fold either. Red
    /// for the same reason as [`legacy_equipped_true_migrates_to_wielded_loadout`]:
    /// nothing yet reads the legacy key regardless of the claimed version.
    #[test]
    fn a_crafted_schema_version_claiming_current_does_not_skip_the_equipment_fold() {
        let dishonest = format!(
            r#"{{
              "schema_version": {SCHEMA_VERSION},
              "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
              "entity_kind": "character",
              "type_id": "companion",
              "ability_funding": "pool",
              "saga_year": 1220,
              "equipment": [
                {{ "item": "weapon.sword_long", "equipped": true }}
              ]
            }}"#
        );
        let loaded = load_entity_migrating(
            &dishonest,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert_eq!(loaded.entity.equipment[0].loadout, LoadoutState::Wielded);
    }

    /// A save already at schema 20, in the new per-element `loadout` shape,
    /// round-trips byte-identically across a load/save/load cycle — the same
    /// property [`a_migrated_save_is_byte_stable_across_a_save_load_save_cycle`]
    /// pins for the pre-14 talisman shape, restated for K5's per-element fold.
    #[test]
    fn a_v20_equipment_save_round_trips_byte_identically() {
        let current = r#"{
          "schema_version": 20,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "equipment": [
            { "item": "weapon.sword_great", "loadout": "carried" },
            { "item": "armor.chain_mail_full", "loadout": "wielded" }
          ]
        }"#;
        let mut first = load_entity_migrating(
            current,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        first.normalize();
        let first_bytes = serde_json::to_string_pretty(&first).unwrap();

        let mut second = load_entity_migrating(
            &first_bytes,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap()
        .entity;
        second.normalize();
        let second_bytes = serde_json::to_string_pretty(&second).unwrap();

        assert_eq!(first_bytes, second_bytes);
    }

    // --- F2/K3: `Entity.mounted` (design-f0-book-template-engine.md § 6) —
    // purely additive, `serde(default)` + `skip_serializing_if`, no fold, no
    // SCHEMA_VERSION bump. These are control cases (already green as soon as
    // the stub field carries the right serde attributes): unlike F1's
    // `loadout` rename, nothing here needs translating from a legacy shape.

    /// A v20 save written before `mounted` existed lacks the key entirely; it
    /// must default to `false`, and the round trip must not introduce the key
    /// (`skip_serializing_if` on the false default).
    #[test]
    fn absent_mounted_defaults_to_false_and_round_trips_without_the_key() {
        let old = r#"{
          "schema_version": 20,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220
        }"#;
        let loaded = load_entity_migrating(
            old,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert!(!loaded.entity.mounted);
        // Literal 20, not `SCHEMA_VERSION`: the claim this test pins is that a v20
        // save lacking `mounted` stays at v20 (no bump owed for a purely additive
        // field), which is a fact about the fixture's OWN claimed version, not about
        // whatever the live constant happens to be — X9b's later, unrelated bump
        // must not make this assertion drift with it.
        assert_eq!(loaded.entity.schema_version, 20);

        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(!json.contains("mounted"), "got {json}");
    }

    /// `mounted: true` survives a load/save cycle and the key is written back.
    #[test]
    fn mounted_true_round_trips_and_serializes() {
        let current = r#"{
          "schema_version": 20,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "mounted": true
        }"#;
        let loaded = load_entity_migrating(
            current,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert!(loaded.entity.mounted);

        let json = serde_json::to_string(&loaded.entity).unwrap();
        assert!(json.contains(r#""mounted":true"#), "got {json}");

        let reloaded = load_entity_migrating(
            &json,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();
        assert!(reloaded.entity.mounted);
    }

    // --- X9b (F-16): `fold_legacy_bard_id` ------------------------------------

    /// The open gap `tmp/x9b-handover.md` flagged: a stray `virtue.rard` value
    /// hiding under a selection's OWN `params` map (not its `item_ref`) is swept
    /// too, both a `Single` value and every member of a `Multi` set. No shipped
    /// parameter can legally hold `virtue.rard` today — none of the three
    /// `domain: "item"` entries' `require_categories`/`allow_ids` admit it — but
    /// a hand-edited save is this project's declared hostile-input surface, so
    /// the fold sweeps every parameter value, not only the ones the shipped
    /// catalogue happens to produce.
    #[test]
    fn a_bard_id_hiding_in_a_selection_param_value_migrates_too() {
        let old = r#"{
          "schema_version": 20,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "selections": [
            { "ref": "virtue.craft_guild_training", "params": { "hermetic_flaw": "virtue.rard" } },
            { "ref": "virtue.puissant_ability", "params": { "ability": ["virtue.rard", "ability.awareness"] } }
          ]
        }"#;
        let loaded = load_entity_migrating(
            old,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap();

        let single = loaded.entity.selections[0]
            .params
            .get("hermetic_flaw")
            .and_then(SelectionParamValue::as_single)
            .cloned();
        assert_eq!(single, Some(Id::new("virtue.bard")));

        let SelectionParamValue::Multi(multi) = &loaded.entity.selections[1].params["ability"]
        else {
            panic!("expected a Multi value");
        };
        assert!(multi.contains(&Id::new("virtue.bard")));
        assert!(!multi.contains(&Id::new("virtue.rard")));
        assert!(multi.contains(&Id::new("ability.awareness")));

        assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
    }
}
