# X6-0 — design: the 16 rule-driving parameters (D70's before-Friday scope)

Design note for `tmp/x6-scope.md`'s slice X6, applying D70. Design only, no code, no data. Reviewed next by the plan-reviewer, then the architect, before X6a starts. Scope per D70: the design note, the engine, and the 16 rule-driving ("M") parameters land before Friday 2026-10-02; the 26 label-only ("L") parameters, the three text→enumerated retypes, and D42 land after, and are **not** designed here beyond the coordination note in § 3.

---

## 0. What already exists (read before designing on top of it)

| Piece | Where | Shape |
|---|---|---|
| `ParamType` | `types.rs` | `Ref \| Number{min,max} \| MultiRef` (`BTreeSet<Id>` value) |
| `ParameterDomain` | `types.rs` | 12 variants incl. `Ability`, `Item`, `Enumerated`, `Category` (item's own `taken_as`), `Realm` (closed 4-enum, no catalogue), `Text`, `Number`, `Spell` |
| `ParameterDef` | `types.rs` | `values`, `at_most_one_of`, `max_per_value` (default 1, D10), `require_categories`/`allow_ids` (Item-domain narrowing, D34), `require_possessed`, `forbid_tainted`, `require_power`, `exclude_if: Option<ItemPredicate>` (D33), `required_if: Option<ParamGate>` (conditional REQUIREDNESS, not severity) |
| `ParamGate{param, equals}` | `types.rs` | already on `AbilityRef::Scoped`, `CategoryRef::Scoped`, `Effect::CharacteristicScoreDeltaParam`, `Effect::GrantsReputation` — the established idiom (B4/Q-51): `gate: Option<ParamGate>` added **directly to the concrete variant**, no wrapper |
| `CharacteristicDeltaCap::{AboveBase, WithinBase}` | `types.rs` | `WithinBase` clamps `bought + contribution` to the ruleset's own ±3 base cap/floor — covers every "+1, but not more than +3" / "-1, but not below -3" shape in § 2, no new clamp variant needed |
| `Effect::SoakMod{amount}`, `AbilityRollMod{ability,amount}`, `CharacteristicScoreDelta{characteristic,amount}` | `types.rs` | **fixed-target, no `gate` field yet** — the gap e1 closes |
| `Effect::MagicResistanceMod{kind,param}` | `types.rs` | `kind: MagicResistanceEffect::AuraBonus` already exists (Commanding Aura's slot) but carries **no amount and is read nowhere** in `derived/casting.rs::magic_resistance` (verified) — CA ships text only today (X2a) |
| `Effect::RestrictedAbilityXp{amount,abilities,categories,instances,from_normal_budget}` | `types.rs` | eligibility = **union** of the three lists (D48) |
| `Effect::ForbidsAbilityCategory{category: AbilityCategory}` | `types.rs` (B1, landed) | fixed-category only; consumed by a dedicated grant-aware validator already built — the e5 gap is the **parameter-relative** sibling |
| `ability_age_cap(entity, ruleset, ability, parameter)` | `effective/reputation_and_caps.rs` | **D29's single resolution point** — folds base age band, a full waiver, `LocalityAbilityCapFraction`, `AffinityAbilityCost`'s +2; both the validator and any UI read only this |
| `derived/casting.rs::magic_resistance` | verified directly | per-Form total, then a **True Faith floor** via `max()` (D37, ArMDE:2627) — no relic term, no CA term |
| `effective/might.rs::true_faith` | verified directly | sums every `Effect::TrueFaithGrant` — **this is F-256's bug**: `virtue.relic`/`virtue.powerful_relic` currently feed it too; **owned by X7b-d**, LIVE red test there |
| `CODE_MISSING_PARAM`/`UNKNOWN_PARAM_VALUE`/`TOO_MANY_FOR_PARAM_VALUE`/`RESTRICTED_XP_UNSPENT`/`INCOMPATIBLE` | `validation/mod.rs` | existing codes; Fluent keys `issue-<code>` in `locales/{en,de}/main.ftl` |
| `ability-category-<slug>` | `locales/*/main.ftl` | already shipped Fluent family for `AbilityCategory`'s 5 values — reused, not duplicated, by e5 |

---

## 1. Engine additions (X6a)

**e1 — `gate: Option<ParamGate>` on `SoakMod`, `CharacteristicScoreDelta`, `AbilityRollMod`, `MagicResistanceMod`.** Same idiom as `CharacteristicScoreDeltaParam`/`GrantsReputation`: field added directly to each concrete variant, `#[serde(default, skip_serializing_if)]`, no wrapper, no `SCHEMA_VERSION` bump (`Effect` lives in `rules/core/`). Each consumer gains one `gate.as_ref().is_none_or(|g| g.holds(selection))` filter. No new validation code — an inactive gate contributes nothing, exactly as `AbilityRef::active_for` already behaves. Also: `MagicResistanceMod` gains `amount: i32` (currently kind-only), paired with `gate` since CA is the entry that needs both.

**e2 — Commanding Aura's flat MR, and the relic coordination (F-256).** `magic_resistance()` folds `ca_bonus` = the sum of every active `MagicResistanceMod{kind: AuraBonus, amount, gate}` on top of the existing per-Form/True-Faith logic. **F-256 (relic True Faith split) is X7b-d's slice, not X6a's** — X6a must not touch `effective::true_faith`. X6a reads a new, X7b-d-owned quantity `relic_mr` (stubbed 0 until that slice lands `Effect::RelicTrueFaith`/its consumer) and applies the book's own composition rule (ArMDE:3583, :17653, :2627): relic present → total = `relic_mr + ca_bonus` (adds; Soak is unaffected by the relic, :17653's carve-out); relic absent → `ca_bonus` competes with the ordinary floor via the existing `max()`. Land the plumbing now, inert until X7b-d ships (open point 2).

**e3 — category/id narrowing for Ability-domain parameters.** `ParameterDef::require_ability_categories: BTreeSet<AbilityCategory>` (non-empty-intersection, mirrors `require_categories`, `domain: Ability` only) and `ParameterDef::forbid_ids: BTreeSet<Id>` (subtractive mirror of `allow_ids`, `domain: Ability` only). Both enforced by the existing `param_value_resolves`, raising the existing `CODE_UNKNOWN_PARAM_VALUE` — no new code. Integrity rejects either field off `Ability`.

**e4 — an exact count on `MultiRef`.** `ParameterDef::exact_count: Option<u8>`, valid only when `param_type: MultiRef`. New code `CODE_WRONG_PARAM_COUNT` ("wrong_param_count"): `issue-wrong_param_count = { $item } names { $count } values for { $key }, but exactly { $expected } are required.` (both locales). Integrity rejects the field off `MultiRef` and rejects `0`. Chosen over F-504's five single `Ref` keys because `BTreeSet<Id>` already gives distinctness for free; five single keys need a second mechanism just to refuse a repeat.

**e5 — the two XP validators.** *Ability Block:* new `Effect::ForbidsAbilityCategoryParam { param: String }`, the parameter-relative sibling of B1's fixed `ForbidsAbilityCategory` (the same "Foo"/"FooParam" convention this file already follows), consumed by the SAME grant-aware validator B1 built, gaining an arm that reads `selection.params[param]`. Needs a new `ParameterDomain::AbilityCategory`, resolving against the closed 5-member `AbilityCategory` enum exactly as `Realm` resolves against `Realm` — values `ability_category.<slug>`, labelled through the already-shipped `ability-category-<slug>` Fluent family. The E+ entry's `custom` branch names no category and gates nothing (stays text, D9). *Restricted Learning:* new `RestrictedAbilityXp::abilities_param: Option<String>` names a `MultiRef`/`Ability` parameter whose resolved set is a fourth source unioned into the pool's eligibility (D48, extended); the entry also adds `categories: [supernatural]` (ArMDE:6685 — safe, since `categories_requiring_virtue` already gates which Supernatural Abilities she may *hold*; this union only says what may be *funded*). Funding alone doesn't stop unrestricted `xp_pool` capacity buying an ineligible Ability, so a new grant-aware validator `validate_ability_xp_scope` checks every scored Ability against the union. New code `CODE_ABILITY_OUTSIDE_RESTRICTED_SCOPE`: `issue-ability_outside_restricted_scope = { $item } restricts experience to { $allowed }; { $ability } is outside that list.`

**e6 — Savantism, through D29's one resolution point.** Two new effects fold into `ability_age_cap` (§ 0), never a second check beside it: `Effect::AbilityScoreCapOverrideParam { param: String, max: u8 }` — the favored Ability caps at `max` (6) **instead of** the age-band figure (stated flatly, not age-derived); `Effect::AbilityScoreCapAllExcept { param: String, max: u8 }` — every OTHER Ability caps at `max` (3), lowering the age band. `ability_age_cap` gains one step: an override naming the asked-about ability returns `max` outright; an all-except whose target differs clamps `cap = cap.min(max)`. F-510's remainder (halved starting XP, halved Advancement Totals, +3 specialty roll) is not this slice's concern — hand-off stands, not re-decided here.

**e7 — Warped Senses' conditional incompatibility** (new; not in `tmp/x6-scope.md`'s own e1–e8 list, surfaced by this pass). New `PointItem::conditional_incompatible_with: Vec<ConditionalIncompatibility>`, `ConditionalIncompatibility { gate: ParamGate, forbids: BTreeSet<Id> }` — a per-value extension of the flat `incompatible_with`, active only when `gate` holds. Consumed by `validate_incompatibilities`, gaining one more forbidden-id source per selection — same `CODE_INCOMPATIBLE`, no new Fluent key. D58 rules this absolute (ArMDE:7031, :7033), so it is a hard error even though the -2 penalty itself stays text (D61).

---

## 2. The 16 rule-driving entries (X6b)

| Entry | Parameter | ArMDE | Effects gated |
|---|---|---|---|
| `virtue.commanding_aura` | Enumerated `rank` {pope, cardinal_legatus, legatus_missus, archbishop, king} | 3583-3591, 17651, 17653 | gated `SoakMod`+`MagicResistanceMod{AuraBonus}`: 5/25, 4/20, 3/15, 2/10, king 2/10 (wife rule stays text, D70) |
| `flaw.savantism` | `Ref`/`Ability` `favored` | 6705-6706 | `AbilityScoreCapOverrideParam{max:6}` + `AbilityScoreCapAllExcept{max:3}` (e6); +3 specialty out of scope |
| `flaw.restricted_learning` | `MultiRef`/`Ability`, `exact_count:5` | 6685 | `RestrictedAbilityXp{abilities_param, categories:[supernatural]}` + `validate_ability_xp_scope` (e5) |
| `virtue.faerie_blood` | Enumerated+custom `heritage` {bee_king, dwarf, goblin, satyr, sidhe, spinnen, undine} | 3805-3819 | Sidhe: `CharacteristicScoreDelta{presence,+1,gate,cap:WithinBase}`; Dwarf: `AbilityRollMod{craft,+1,gate}` (e1); rest text |
| `virtue.strong_faerie_blood` | Enumerated `heritage` (same 7) + `Text` quirk | 5032-5047 | same shape as Faerie Blood, keyed off its own `heritage` param (:5042, "as given in Faerie Blood") |
| `flaw.monstrous_blood` | Enumerated+custom `type` {magic_animal, magic_human, magic_spirit, magic_thing} + `Ref`/`Characteristic` (`required_if: type==magic_human`) | 6458-6466 | Magic Human: `CharacteristicScoreDeltaParam{-1,gate,cap:WithinBase}` + `GrantsReputation{3,gate}` |
| `flaw.ability_block` | new `AbilityCategory` `class` + custom `Text` | 5651-5654 | `ForbidsAbilityCategoryParam{param}` (category branch only, e5) |
| `flaw.vengeful_powers` | existing `Category` `taken_as` {story, hermetic} | 6975 | data only — category membership read by X3's trained gate / House credit |
| `virtue.potent_magic_major`/`_minor` | `Text` `field`, `max_total:255` | 4742-4746 | +3 keyed off the Magical Focus mechanism per X7b-d's D4 test — **coordinate, do not duplicate** |
| `virtue.special_circumstances` | `Text` `circumstance`, `max_total:255`, `max_per_value:1` (default) | 5000 | +3 stays **surfaced-only** (D45/D61: scene-conditional); the parameter only closes the duplicate-copy bug (F-541) |
| `virtue.performance_magic` | `Ref`/`Ability` `ability`, `require_ability_categories:{general}`, `max_total:255` | 4646-4648 | picker/validator domain via e3 |
| `virtue.magian_lineage_major` | `MultiRef`/`Ability`, `require_ability_categories:{arcane,supernatural}`, `forbid_ids:{ability.true_names}`, `exact_count:3` | 4345 | e3+e4; connected-XP rule stays text (in-play) |
| `flaw.repellent` | Enumerated+custom `feature` {natural_weapons, scales, dark_sight} | 6681 | `SoakMod{+3, gate: feature==scales}` (e1) |
| `virtue.turb_trained` | `language` (D59/CV catalogue of dead languages; **CV not landed — § 5**) | 5181 | `AbilityAuthorization` via `AbilityRef::Scoped{ability: dead_language, instance: ParamValue::Bound{param:"language"}}` (D14 shape 2) |
| `flaw.warped_senses` | Enumerated `form` + Enumerated `sense` (`required_if`) | 7029-7037 | conditional `incompatible_with` (e7); -2 stays text (D61) |

---

## 3. D70's required-parameter policy, and the schema

**Q-X6-4 settled (a):** every new parameter above is **required**. An old save missing it reports `CODE_MISSING_PARAM`; `Enforced` blocks, `Advisory` warns, `Silent` suppresses — D10's existing three-mode machinery, unchanged. **No advisory-severity flag is built** — `required_if` keeps its existing job (conditional requiredness *within* one entry) and is not repurposed for severity; x6-scope's tentative "e8" is **dropped**, since D70 answered Q-X6-4 as (a), not (b).

**No `SCHEMA_VERSION` bump.** Every field in § 1 is additive on `Effect`/`ParameterDef`/`PointItem` — ruleset JSON, never `Selection`/`Entity` (saves). The one save-shaped change anywhere in X6's plan — D42's optional concept-realm field — is X6d (after Friday), itself additive and omittable (Q-X6-5), so it needs no bump either; noted only so "after Friday" is not misread as "undesigned." **If any item in § 1 turns out not to be additive at implementation, stop and reopen this note** — D70 grants no bump license.

The three text→enumerated retypes (X6c, after Friday) are **not** migrated; old free text fails `CODE_UNKNOWN_PARAM_VALUE` (Q-X6-4, decided).

---

## 4. Test plan and sub-slice order

**X6a (engine, zero data) → X6b (the 16 entries' data + consumers).** Red-first per CLAUDE.md; each red must fail for the right reason (a not-yet-existing field/variant fails to *compile* first, then the behavior assertion must fail before any implementation).

Order inside X6a: **e1** first (every gated consumer depends on `gate` existing) → **e3, e4** (independent of each other and of e1/e2) → **e2** (needs e1's `amount`+`gate`) → **e5** (needs e3's domain and e4's count) → **e6** → **e7 last** (touches `validate_incompatibilities`, least coupled here).

| e | Representative reds |
|---|---|
| e1 | `soak_mod_inactive_when_gate_unmet`, `magic_resistance_mod_aura_bonus_applies_when_gate_met` |
| e2 | `commanding_aura_pope_grants_soak_5_and_mr_25`, `commanding_aura_and_relic_add`, `commanding_aura_competes_with_form_total_when_no_relic` |
| e3 | `performance_magic_rejects_academic_ability`, `magian_lineage_major_rejects_true_names` |
| e4 | `restricted_learning_rejects_four_abilities`, `restricted_learning_accepts_five_distinct` |
| e5 | `ability_block_martial_blocks_martial_xp`, `ability_block_custom_branch_stays_uncomputed`, `restricted_learning_blocks_ability_outside_five_and_supernatural` |
| e6 | `savantism_favored_ability_caps_at_6_regardless_of_age`, `savantism_other_abilities_cap_at_3_under_the_normal_age_cap` |
| e7 | `weak_sight_excludes_sensitive_sight_and_keen_vision`, `weak_hearing_has_no_incompatibility_with_a_sight_entry` |

X6b lands all 16 entries' JSON (both locales for every new enumerated value, looked up in the translation tables per standing rule) with each entry's own structural test plus an `unknown_param_value` negative — x6-scope § 4 already states this per-entry plan, not repeated here. Full required gate (`cargo test/clippy/fmt`, `npm run test:unit/lint/format:check`, `cargo tauri build --no-bundle`) at the end of X6a and again at the end of X6b — not per micro-slice.

---

## 5. Open points for Norbert (not decided here)

1. **CA stacking.** Can a character hold two simultaneously-active `AuraBonus` sources? Nothing in ArMDE:3583-3591 forbids it. **Recommend: sum**, the default every other additive `Effect` uses — flagged since no shipped entry exercises it today.
2. **e2's relic stub ordering.** X6a lands `relic_mr` as a permanent 0 until X7b-d supplies the real figure. **Recommend proceeding** — inert everywhere no relic exists, and blocking X6a on X7b-d's schedule serves nothing. Confirm this ordering.
3. **Special Circumstances' +3.** Recommend it stays surfaced-only (D45/D61 precedent), so X6b lands only the `Text` parameter + the duplicate-copy fix. Computing the actual bonus needs an explicit "circumstance is active" toggle — a larger, separate design.
4. **Ability Block's `custom` branch.** Recommend it stays permanently text (no closed domain to validate a free-text Ability list against, and ArMDE:5653 states no count to bound it). Confirm this isn't expected to gain its own MultiRef later.
5. **e7's severity.** D58 calls Warped Senses' incompatibilities absolute, so e7 makes the *selection* a hard error even though the -2 penalty stays text (D61). Confirm this asymmetry (computed gate, uncomputed effect) is the intended reading.
6. **Turb Trained's CV dependency.** Its `language` parameter is specified against "the catalogue's dead languages" (D70/Q-X6-2), but D59/CV has not landed (verified: no `language` parameter or catalogue-value type exists today). Recommend X6b either lands CV's minimal slice first, or ships `turb_trained` with a plain `Ability`-instance parameter now and migrates to the CV catalogue later (additive, non-bumping, per D59). Confirm which.

---

## References Loaded

`tmp/x6-scope.md`; `docs/vf-audit/decisions.md` D70, D68, D67, D66 (skimmed), D61, D58, D48, D42, D35, D34, D33, D23 (context), D14, D10, D9, D29; `docs/vf-audit/design-d0-xp-modes.md` (form/idiom reference); `docs/vf-audit/design-b0-ranging-and-predicates.md` (gate idiom, B1/B4/B5 precedents); `rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` (every ArMDE citation above verified directly); `crates/arm-rules/src/types.rs`, `effective/might.rs`, `effective/reputation_and_caps.rs`, `derived/casting.rs`, `validation/mod.rs`, `grant.rs`; `locales/en/main.ftl`; `tmp/x7bd-verdicts.md` (F-256, F-329).

### New Learnings

D29's "single resolution point" for an Ability's max score already exists (`effective/reputation_and_caps.rs::ability_age_cap`) — any future per-ability cap override folds into it, never a second check beside it. `MagicResistanceEffect::AuraBonus` exists in the enum today but is read nowhere — a kind existing in a closed enum is not evidence it is wired to a consumer; check the consumer directly. `ability-category-<slug>` Fluent keys already ship (ability filter UI) — reusable by a new `ParameterDomain::AbilityCategory` picker with no new translation work.

### Verdict

COMPLETE — design only, no code or data changed.
