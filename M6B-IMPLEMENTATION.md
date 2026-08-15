# Milestone 6b — Implementation Tracker

Sequenced execution plan for 6b ("Guided creation wizard"). `PLAN.md:588-618` holds
the high-level 6b scope checkboxes; this file is the session-resumable execution
tracker with per-slice tasks, decisions, and verified source citations. Tick a box
here and the matching `PLAN.md` box in the same commit as green code.

6a is done (`PLAN.md:562-586`): the app opens on `StartScreen.svelte` and a character's
type is fixed at creation via `store.createCharacter(typeId)`. 6b1b replaced that
screen's hardcoded-disabled wizard button with one guided entry per character type.

**Status: 6b1a, 6b1b, 6b2, 6b3 (both halves), 6b4 and 6b5 (both halves) are done.**
Next is **6b6** — the aging tables, the aging total and outcome resolution, which is
also where `life_stage_aging_rolls_pending` (shipped in 6b5a against `CreationPhase::Review`
for want of anywhere better) moves onto its own `Aging` phase and its threshold moves
out of Rust into `rules/core/aging.json`. The `PLAN.md` 6b3 boxes are ticked as of
6b3b; 6b4 ticked the first half of the magus life-stage story (apprenticeship) and 6b5
ticks the second (the years after the Gauntlet), so both boxes are now closed.

6b2 shipped as five commits: the life-stage rules as data; the missing Poor Major
Flaw plus the Wealthy/Poor rate effect; `Entity::life_stages` and the derived
budget; the childhood blocks as instance-restricted pools in the existing solver;
and the surfacing. Two deliberate deviations from the plan above:

- **`life_stages.json` carries only childhood and later life.** The apprenticeship
  and post-apprenticeship numbers move to 6b4/6b5, with the engines that read them —
  shipping inert unvalidated data is the very thing the M3 deferral note warned
  against.
- **`ChildhoodRules.native_language_ability` was added** (not in the plan): the
  native-language block has to name the ability it buys, and a hardcoded
  `ability.living_language` in Rust would have been an engine-required slug. It is
  data, and load-time integrity requires it to resolve *and* be parameterized.

**Both tails are done as well**, in two further commits:

- **6b2b** — Academic/Arcane/Martial Abilities require a permitting Virtue (`:2315`,
  `:2392`), which was previously enforced for Supernatural only. The gated categories
  are data; a `restricted_ability_xp` pool counts as permission, so Educated, Warrior,
  Arcane Lore and Privileged Upbringing needed no new data (only Covenant Upbringing
  did, for its Latin at `:5867`); Supernatural keeps its stricter per-Ability rule; and
  magi are exempt per `:7151`/`:2435`. The Latin-3 clause (`:7151`) is a warning,
  since the passage hedges it twice.
- **6b2c** — Foreign Upbringing halves the creation cap on locality-dependent
  Abilities (`:6160`), via a `locality_dependent` flag on the Ability and a cap
  fraction on the Flaw. The passage's trailing "some social Abilities" is left
  unflagged on purpose: which ones a saga counts is a data decision, and the engine
  enforces the flag it is given rather than guessing.

One thing 6b2b models only **partly**, and should be recorded rather than forgotten:
`:7151`'s finer "Magi without a specific Virtue may only buy Academic Abilities
during or after apprenticeship". A bought score still carries no per-stage attribution,
so the exemption remains whole-character for a magus on the flat pool. 6b4 covers the
*guided* magus, where the stage is implicit in the pool: later life is a restricted
pool that excludes Academic/Arcane/Martial unless a Virtue permits them, while
apprenticeship's own 240 may buy them freely (`:2435`).

## Required gate (end of every slice, before any "done" claim)

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
cd ui && npm run test:unit && npm run lint && npm run format:check && cd ..
cargo tauri build --no-bundle          # authoritative: type-checks UI + builds release
cd ui && npm run test:e2e              # green against the real release binary
```

TDD mandatory (red → green → refactor). Cite rules by source basename + line range at
the implementation site; update `crates/arm-rules/RULES.md` in the same change. Keep
canonical key/array sorting. German labels must match
`rules/source/de/translation-tables/`.

## Settled decisions

| Question | Decision |
|---|---|
| Phase attribution for gating | **Engine-side**: a required `phase` field on `ValidationIssue`, set at each of the 85 emit sites |
| Phase vocabulary | A Rust enum `CreationPhase` with `ALL` + `Display`; `creation_phases` is typed, so serde validates it at load |
| Die rolls (aging, crisis) | **User types the result.** `arm-rules` gains no RNG and stays deterministic |
| Wizard placement | **Third view** (`store.view === 'wizard'`) over the same real entity; Finish lands in the editor; tabs hidden while it is up |
| Navigation | Back **and** forward: back always allowed, forward via Next (gated) or a rail click to any already-visited phase |
| Poor Major Flaw | Missing from the catalogue; **added and tested in 6b2**, where its rate effect lands |

## Slice map

| Slice | Contents | Depends on |
|---|---|---|
| **6b1a** ✅ | Phase vocabulary + issue attribution across all 85 emit sites; Fluent keys; TS mirror. UI unchanged | 6a |
| **6b1b** ✅ | Wizard shell: view, rail, back/forward nav, per-phase gating, existing components mounted as steps | 6b1a |
| **6b2** ✅ | Life-stage XP engine (childhood 75+45, later life 15/20/10, pools into the existing solver) **+ the missing Poor Major Flaw** | 6b1a |
| **6b3a** ✅ | Sample Childhood packages: catalogue + load-time integrity + applicator + IPC command (no UI) | 6b2 |
| **6b3b** ✅ | The "sophisticated" Abilities step: funding-mode toggle, life-stage panel, package picker, XP-bar guided branch, e2e | 6b3a |
| **6b4** ✅ | Magus apprenticeship (240 xp / 120 spell levels / hard minimums) | 6b2 |
| **6b5** ✅ | Post-Gauntlet accrual (30 pts/year, lab-season deduction, xp↔spell-level split) | 6b4 |
| **6b6** | Aging tables + aging total + outcome resolution | 6b1a |
| **6b7** | Crisis, Decrepitude levels, per-year write-back | 6b6 |
| **6b8** | Per-type flow completion, completeness indicators, guided copy, milestone gate | 6b2-6b7 |

6b1 was one slice until review: ~24 TDD steps spanning 85 emit sites, a contract-table
rewrite, two new scanner tests, three extractions, five new components and a new e2e
spec is too much for one gate-closing slice. Split at the IPC boundary — 6b1a is
independently green with an unchanged UI that merely carries the new field.

---

## Slice 6b1a — Phase vocabulary and issue attribution ✅

### Design

**The vocabulary** is a Rust enum in `crates/arm-rules/src/types.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreationPhase { Concept, Type, Characteristics, VirtuesFlaws, Abilities,
                         Arts, Spells, HouseSpecialisation, MythicType,
                         PersonalityReputations, Review }
impl CreationPhase { pub const ALL: [CreationPhase; 11] = [ /* … */ ]; }
impl fmt::Display for CreationPhase { /* the serde slug */ }
```

A fixed taxonomy the rules define, so an enum per CLAUDE.md — surfaced as `ALL` (like
`AbilityCategory::ALL`) so nothing re-hardcodes its members. It lives in `types.rs`,
not `validation/`, because both `ValidationIssue` and
`EntityTypeProfile.creation_phases` consume it.

`EntityTypeProfile.creation_phases` (`types.rs:1537`, a required `Vec<String>`) becomes
`Vec<CreationPhase>`, which makes serde itself the load-time validator: an unknown
phase string fails the ruleset load with the bad variant named, and the wizard can never
meet a phase it has no step for. JSON is byte-identical (same snake_case slugs), so
golden and round-trip tests are unaffected, and `normalize()` still leaves the order
alone.

Two covenant fixtures declare `"boons_hooks"` (`types.rs:3109`, `:3271`), a phase in no
shipped profile and implemented nowhere. Replace it with **two real phases in
non-canonical order** — `["type", "concept"]` — because the assertion at `:3122` exists
to prove `normalize()` does *not* sort this vector, and a single-element list would make
that vacuous. Comment it as a placeholder for M8's covenant phases.

**`Review` is the home for issues no phase owns.** `unknown_equipment`,
`equipment_min_strength`, `shield_with_two_handed_weapon`, `over_item_level`,
`over_power_levels`, `might_realm_mismatch`, `warping_*`, `excessive_aging_reduction`
and `unknown_type` belong to no declared phase, because there is no equipment /
possessions / warping / aging phase. A named terminal `Review` variant, rather than
`Option<CreationPhase>`, makes "only fixable in the finished character" an explicit,
testable claim and gives the wizard a step to hang it on. Profiles may not declare it
(integrity error); the wizard appends it in 6b1b. The same bucket covers issues in
phases a profile does not declare (a grog has no `arts` step): the Review step is
unfiltered and Finish gates on *all* errors.

**Attribution is per emit site, not per code.** `missing_param`,
`unknown_param_value`, `prereq_not_met`, `duplicate_selection` and `unknown_ref` are
emitted from several modules over different subject kinds, so a `code → phase` table
would be wrong for them. `phase` becomes the **third constructor argument** —
`ValidationIssue::error(code, phase, args, context)` (`validation/mod.rs:415/430/435`) —
and the compiler walks all 85 sites (`scores.rs` 18, `magus.rs` 21, `selections.rs` 13,
`mod.rs` 8, `caps.rs` 6, `warping.rs` 6, `balance.rs`/`equipment.rs`/`might.rs`/
`prereq.rs` 3 each, `aging.rs` 1 — census verified).

Rule at each site: **the phase whose input surface owns the offending value**, not the
module that detected it. Notable calls: `missing_hermetic_flaw` and
`mythic_required_trait_missing` are emitted in `magus.rs` but fixed on the V/F step →
`virtues_flaws`; `supernatural_ability_requires_virtue` → `abilities` (the stored
offending value is the Ability); `not_enough_xp` → `abilities` (where the `XpBar` lives,
though the pool spans Arts too); all dynamic category-cap codes from `caps.rs:98-113` →
`virtues_flaws`.

`phase` is required in the JSON, no `skip_serializing_if`. `ValidationResult` is a
transient IPC payload, never a git-tracked save, so this costs no migration and
`SCHEMA_VERSION` stays 14. Blast radius outside `validation/` is small — nothing else
constructs a `ValidationIssue` (only a re-export at `lib.rs:81` and `CODE_*` reads) and
no proptest or golden fixture serializes one — but two spots break and must be fixed
here: the deserialization test at `validation/mod.rs:2537`, which parses an issue
literal without the new key, and the four TS issue literals in
`ui/src/lib/derive.test.ts`.

### Tasks

- [x] **1. `CreationPhase` round-trips.** RED in `types.rs`: `VirtuesFlaws` serializes
      to `"virtues_flaws"`, `ALL.len() == 11`, `Display` equals the serde slug for every
      member. GREEN: the enum + `ALL` + `Display`; re-export from `lib.rs`.
- [x] **2. Typed profile phases.** RED: a profile with `"creation_phases":
      ["not_a_phase"]` fails to parse; update `:3122` to the `["type", "concept"]`
      fixture, keeping it an order assertion. GREEN: retype the field; fix both
      fixtures.
- [x] **3. Load-time integrity** in `ruleset.rs`, beside the duplicate checks at
      `:686-731`: a profile declaring `review` is an error; a repeated phase is an
      error. Decide and document the stance on an **empty** `creation_phases` (several
      fixtures ship `[]`, and such a profile would get a review-only wizard).
- [x] **4. Every module attributes its issues.** RED: table-driven test in
      `validation/mod.rs` asserting `(code, phase)` for ≥1 representative per submodule
      — `over_budget_virtues → virtues_flaws`, `characteristic_overspent →
      characteristics`, `ability_above_age_cap → abilities`, `art_score_out_of_range →
      arts`, `over_spell_levels → spells`, `house_choice_unresolved →
      house_specialisation`, `mythic_choice_unresolved → mythic_type`,
      `personality_trait_out_of_range → personality_reputations`,
      `too_many_personality_flaws → virtues_flaws`, `prereq_not_met → virtues_flaws`,
      `unknown_equipment` / `over_power_levels` / `warping_fill_excess` /
      `excessive_aging_reduction` / `unknown_type` → `review`. GREEN: the field, the
      three constructors, all 85 sites plus the deserialization test at `:2537`.
      **Commit one submodule per commit.**
- [x] **5. Contract table carries the phase.** Extend
      `every_issue_code_const_is_documented_in_the_contract_table` (`mod.rs:1979`) to
      require a 4-column row `| code | severity | phase | args |`, assert every phase
      token is a slug from `CreationPhase::ALL` (derived, never hardcoded), and assert
      in reverse that every `ALL` slug appears at least once. Rewrite the table at
      `mod.rs:77-168`; the `†` footnote states the dynamic cap codes are
      `virtues_flaws`.
- [x] **6. No emit site drifts.** New source-scanning test (same idiom as the existing
      `const CODE_` scanner): for every `ValidationIssue::(error|warning|new)(` call,
      extract the `CODE_*` and `CreationPhase::*` tokens and assert the table row lists
      that phase; the two runtime-`&code` sites (`caps.rs:110`, `:112`) are exempt from
      the code half but must still carry a literal phase. Assert `sites >= 80` so a
      broken scanner cannot look green.
- [x] **7. Fluent coverage.** New test in `crates/arm-app/tests/commands.rs`, mirroring
      `every_export_label_key_has_a_fluent_key_in_each_locale` (`:1010`), iterating
      `CreationPhase::ALL` and asserting `phase-<slug>` in both locales. Add the 11 keys.
- [x] **8. TS mirror** (`ui/src/lib/types.ts`): `CreationPhase` union next to
      `IssueSeverity`; `phase: CreationPhase` on `ValidationIssue` (`:1123`);
      `creation_phases: CreationPhase[]` (`:596`); fix the four `derive.test.ts`
      literals. Add the Rust-side test that every `ALL` slug appears in `types.ts` — the
      one real drift risk now that phases gate.
- [x] **9. Full gate.** No behavior change, so e2e should pass untouched — run it anyway,
      since the IPC payload shape changed. (All 25 specs green against the real
      binary; the whole gate passed.)

---

## Slice 6b1b — The wizard shell ✅

### Design

**Gating, in one rule:** *Next is blocked iff the current phase has an `error`-severity
issue; Finish is blocked iff any error remains anywhere.* `apply_mode` runs in the app
layer (`crates/arm-app/src/ruleset_io.rs:454`), so Advisory (errors→warnings) and Silent
(issues cleared) degrade to "never blocks" for free. The `ModeToggle` **is** the escape
hatch — no "finish anyway" button. The UI copy must say plainly that in Advisory or
Silent mode nothing is gated and Finish is unconditional.

**Known limitation, stated here and not hidden until 6b8: legal ≠ complete.** The gate
catches only *errors*, so a merely empty phase sails through — `house_unset` is a
**warning**, so a magus can Next past `house_specialisation` and Finish with no House,
and a character with zero Abilities bought is perfectly legal. The Review step's copy
says so; the completeness indicator that fixes it is 6b8.

**Store shape** (`ui/src/lib/state.svelte.ts`):

```
view = $state<'start' | 'editor' | 'wizard'>('start')   // retype at :230
wizardStep     = $state(0)   // index into wizardPhases
wizardFurthest = $state(0)   // high-water mark; only wizardNext raises it
wizardPhases     = $derived.by(…)  // [...profile.creation_phases, 'review']; [] with no profile
wizardPhase      = $derived(this.wizardPhases[this.wizardStep] ?? 'review')
wizardCanAdvance = $derived(!phaseHasBlockingIssue(issues, this.wizardPhase))
wizardCanFinish  = $derived(!issues.some((i) => i.severity === 'error'))

async startWizard(typeId)   // #instantiateCharacter → view='wizard' → #resetWizardNav → revalidate
wizardNext()                // gated by wizardCanAdvance; raises wizardFurthest
wizardBack()                // never gated; never lowers wizardFurthest
wizardGoTo(i)               // i <= wizardFurthest; a forward jump clamps to the first blocked phase
finishWizard()              // gated by wizardCanFinish; view='editor'; nav reset; entity untouched
#instantiateCharacter(typeId)  // extracted from createCharacter (:1514)
#resetWizardNav()              // called by startWizard/finishWizard/newDocument/open
```

- `createCharacter` keeps its behavior; its body moves to `#instantiateCharacter` so one
  place seeds mandatory traits, clears path/filters/results, and re-seeds
  `#savedSnapshot` (`:255`).
- **`revalidate()` needs no change**: its early return tests `view === 'start'`
  (`:1556`), so `'wizard'` already falls through to real validation. Comment it so
  nobody "tidies" it into `!== 'editor'`.
- **The forward clamp must include the departure phase.** Advance to step 5, go back to
  2, break 2: Next is blocked by the current-phase gate, but `wizardGoTo(5)` is
  permitted by the high-water mark — so `firstBlockedPhaseIndex(phases, issues, from,
  to)` must scan **inclusive of `from`**, or the rail becomes a way around the Next
  gate. Pin that in the helper spec, not just a store test.
- Nav state is not in `#snapshot()`, so navigating never dirties. `startWizard` re-seeds
  the baseline (a fresh wizard character is clean); the first edit dirties it; the
  `closeGuardPayload()` mirror in `App.svelte` sits outside the view branch and keeps
  working — quitting mid-wizard prompts exactly as the editor does, so the mandatory
  unsaved-changes guard is untouched.
- Ctrl+S/Shift+S **work in the wizard** (a wizard character is a real character and the
  close guard promises Save is reachable): the gate at `App.svelte:129` flips from
  `store.view !== 'editor'` to `store.view === 'start'`. Ctrl+N unchanged (discard guard
  → start screen). Ctrl+O unchanged and always lands in the **editor** — a save records
  no wizard progress.

**Component tree:**

```
App.svelte
├─ header (LanguageSelector always; doc-status + ModeToggle + SaveLoadBar when view !== 'start')
├─ view==='start' → StartScreen        (per-type create row + per-type wizard row)
└─ else
   ├─ CharacterBanner                 (extracted from App.svelte:200-222; shared by both views)
   ├─ view==='wizard' → WizardShell
   │     ├─ nav.wizard-rail           (aria-label; button per phase; disabled past furthest; aria-current)
   │     ├─ p.wizard-progress         ("Step 1 of N", localized)
   │     ├─ main.tab-content.wizard-body → WizardStep {phase}
   │     ├─ footer.validation-bar → ValidationPanel docked phase={…}   (unfiltered on review)
   │     └─ nav.wizard-nav            (wizard-back | wizard-next ⟷ wizard-finish on review)
   └─ else → tabbar + tab-content + validation-bar  (unchanged)
```

Two reuse constraints shape this. The step body reproduces the existing wrapper chain
verbatim — `<div class="vf-tab">[bar][<div class="tab-scroll">]<Component/>` — so the
height-bounded flex chain (`.tab-content` `app.css:337`, `.vf-tab` `:358`, `.region-row`
`:414`) keeps working with no new layout rules. **`.tab-content`'s `flex:1; min-height:0`
only binds if `WizardShell`'s own root is itself a bounded flex column passing height
down from `#app`** — the one thing the new CSS block must provide, or the
Available/Selected lists collapse. And the **phase title lives only in the rail**, never
as a step heading, because every region component hardcodes its own
`<h2 class="region-title">`; a step heading would nest h2s.

Phase → component table, declared `satisfies Record<CreationPhase, WizardStepDef>` so a
new phase is a `svelte-check` error:

| phase | component | bar | `.tab-scroll` |
|---|---|---|---|
| `concept` | `IdentityFields` (extracted) | — | yes |
| `type` | `TypeStep` (new) | — | yes |
| `characteristics` | `CharacteristicPicker` | — | no |
| `virtues_flaws` | `VirtueFlawTab` | `BalanceBar` | no |
| `abilities` | `AbilityTab` | `XpBar` | no |
| `arts` | `ArtGrid` | `XpBar prefix="art-"` | no |
| `spells` | `SpellTab` | — (owns `SpellBudgetBar`) | no |
| `house_specialisation` | `HouseSelector` | — | yes |
| `mythic_type` | `MythicCompanionTypeSelector` | — | no |
| `personality_reputations` | `PersonalityReputationsStep` (new, composes two extractions) | — | yes |
| `review` | `WizardReview` (new) | — | yes |

Closing the three phases with no component (ranges verified; each extraction leaves the
enclosing `<section>` / `{#if store.ruleset}` wrapper behind):

- **`concept`** — two pure extractions, no invented inputs. `CharacterBanner.svelte`
  from `App.svelte:200-222` (type label + name + description) and
  `IdentityFields.svelte` from `CharacterDetails.svelte:114-172` (concept, gender, birth
  year, sigil, covenant, parens). `CharacterDetails` mounts the extraction in place.
  Testids unchanged → zero e2e migration.
- **`type`** — `TypeStep.svelte`, a read-only confirmation of what the immutable type
  commits the character to: localized `type-<id>` label, the profile's V/F budget
  numbers, one Gift-policy line. Gated on profile data, never `if type === 'magus'`.
- **`personality_reputations`** — extract `PersonalityTraits.svelte`
  (`CharacterDetails.svelte:442-499`) and `Reputations.svelte` (`:501-546`), both mounted
  in place, composed by a thin `PersonalityReputationsStep`. Their helpers (`traits`
  `:37`, `grants` `:40`, `repKindLabel` `:99`) are used only inside those blocks, so the
  moves are clean. The only way to avoid dragging aging/warping/twilight surfaces into
  the step.

**Do not touch in 6b1b:** `CharacterDetails`'s age/aging/warping/twilight/decrepitude/
confidence blocks; `EquipmentTab`, `MagicPossessions`, `SupernaturalBeing`,
`DerivedTotalsPanel` (no phase maps to them; Review announces they live in the editor);
`SpellTab`'s self-owned budget bar; and `store.filters.vf` being shared between views
(same character, never on screen at once — document it so nobody "fixes" it).

### Tasks

- [x] **10. Pure helpers** in `derive.test.ts` → `derive.ts`: `wizardPhases(profile)`
      (profile order + `review` appended once; magus keeps `house_specialisation` before
      `virtues_flaws`), `issuesForPhase`, `phaseHasBlockingIssue` (errors only),
      `firstBlockedPhaseIndex(phases, issues, from, to)` **with an explicit case proving
      the range includes `from`**.
- [x] **11. Store: entry.** `startWizard('magus')` → `view === 'wizard'`, mandatory
      traits seeded, path/filters/results cleared, `dirty === false`, nav at 0/0, and
      `ipc.validateEntity` called once (proving no early return).
- [x] **12. Store: navigation.** `wizardNext` advances and raises `wizardFurthest`; no-op
      while the phase has an error; `wizardBack` always works, never lowers the mark,
      no-op at 0; `wizardGoTo(furthest)` works, `wizardGoTo(furthest + 1)` does not; a
      forward jump clamps both when a *strictly intermediate* phase is broken and when
      the **departure** phase itself is broken.
- [x] **13. Store: finish and exits.** `finishWizard()` → `view === 'editor'`, entity
      byte-identical, `dirty` unchanged; no-op while `wizardCanFinish` is false;
      navigating never flips `dirty`; an edit in the wizard flips `dirty` **and**
      `closeGuardPayload().dirty`; `newDocument()` → `'start'` with nav reset; `open()` →
      `'editor'` with nav reset.
- [x] **14. Extractions** (pure moves, no new specs — `App.test.ts`,
      `character-fields.e2e.js` and `validation-errors.e2e.js` are the regression gate):
      `CharacterBanner.svelte` ← `App.svelte:200-222` + the `typeName` derivation at
      `:55-59`; `IdentityFields.svelte` ← `CharacterDetails.svelte:114-172`;
      `PersonalityTraits.svelte` ← `:442-499` and `Reputations.svelte` ← `:501-546`.
- [x] **15. `ValidationPanel` phase prop:** renders only that phase's issues when set,
      everything when absent (today's behavior), `no-issues` when filtered empty.
- [x] **16. `WizardStep.svelte`** — the phase→component table; per-phase test that the
      expected component's testid appears and the step body adds no `<h2>` of its own.
- [x] **17. `TypeStep.svelte`** (localized type label, budget numbers from
      `profile.budget`, exactly one Gift-policy line) and **`WizardReview.svelte`**
      (unfiltered panel, clean-state message, the "Equipment / Magic Items / Aging /
      Totals live in the editor" hint, and the legal-≠-complete caveat).
- [x] **18. `WizardShell.svelte`** — rail button per phase in order
      (`data-testid="wizard-step-{phase}"`), `aria-current="step"` on the current one,
      `disabled` past `wizardFurthest`, and a blocked phase **announced to assistive
      tech**, not just styled (`data-blocked` is a CSS hook; also attach
      `wizard-step-blocked-label` via `aria-label`/`aria-describedby`, and give the rail
      `<nav>` an accessible name). `wizard-back` disabled at step 0; `wizard-next`
      disabled when blocked; on `review` `wizard-next` absent and `wizard-finish`
      present-but-disabled while errors remain; localized `wizard-step-progress`. New CSS
      block including the bounded-flex root.
- [x] **19. `App.svelte`** three-way view branch; the **two** `view === 'editor'` header
      guards (`:173` doc-status, `:190` ModeToggle + SaveLoadBar) flip to
      `view !== 'start'`; the Ctrl+S gate at `:129` flips to `view === 'start'`; the
      app-level validation footer moves inside the editor branch.
- [x] **20. `StartScreen.svelte`** — one enabled `start-wizard-{id}` per profile,
      mirroring the create row, labelled through `type-<id>`, disabled while
      `loading`/`busy`; rewrite `start-wizard-hint`; delete the unused `start-wizard` key
      from both locales; cover the zero-profiles case (no buttons, error banner only).
- [x] **21. E2E.** `ui/e2e/helpers.js:64` — `returnToStartScreen()` waits for
      `START_SCREEN || TAB_BAR`; a wizard view has neither, so it would fail at
      `BOOT_TIMEOUT`. Add the wizard rail to the probe and add `startWizard(type)`. New
      `ui/e2e/specs/wizard.e2e.js` (magus); update the `start-wizard`-disabled assertion
      at `app-entry.e2e.js:50-51`.
- [x] **22. Docs + full gate.** `PLAN.md` 6b checkboxes + focus note; `README.md` (a real
      third input mode now exists). `RULES.md` needs nothing — 6b1 ships no new mechanic.

### New Fluent keys (both `locales/en/main.ftl` and `locales/de/main.ftl`)

- **Phase labels** (6b1a), keyed by the engine slug like `type-<id>` / `category-<id>`
  and enforced by task 7: `phase-concept`, `phase-type`, `phase-characteristics`,
  `phase-virtues_flaws`, `phase-abilities`, `phase-arts`, `phase-spells`,
  `phase-house_specialisation`, `phase-mythic_type`, `phase-personality_reputations`,
  `phase-review`.
- **Chrome** (6b1b): `wizard-title`, `wizard-rail-label`, `wizard-step-progress`
  (`{$current}`, `{$total}`), `wizard-back`, `wizard-next`, `wizard-finish`,
  `wizard-blocked-hint`, `wizard-step-blocked-label`, `wizard-review-title`,
  `wizard-review-clean`, `wizard-review-hint`, `wizard-review-incomplete-caveat`,
  `wizard-unchecked-hint`.
- **Type step**: `phase-type-explainer`, `phase-type-budget` (`{$virtues}`, `{$flaws}`),
  `phase-type-gift-required`, `phase-type-gift-forbidden`, `phase-type-gift-optional`.
- **Start screen**: new `start-wizard-choose-hint`; rewritten `start-wizard-hint`;
  **removed** `start-wizard`.

German must follow the translation tables for every rules term (Tugenden/Schwächen,
Eigenschaften, Fertigkeiten, Künste, Zauber, Haus, Ruf, Persönlichkeitsmerkmale) — the
phase labels are exactly where a table mismatch shows. The rail must re-render correctly
across `store.setLang`, which reloads the ruleset without resetting the dirty baseline;
a language switch mid-wizard must keep `wizardStep`.

### E2E spec — `wizard.e2e.js`

1. `start-wizard-magus` enters the wizard: `wizard-rail` present, `role="tablist"`
   absent, `character-type` shows the localized magus label.
2. Rail order equals the magus profile's declared order — the one assertion that
   `house_specialisation` precedes `virtues_flaws`.
3. `wizard-back` disabled at step 0; `wizard-next` advances; `aria-current` follows.
4. On `virtues_flaws`, take a Major Virtue with no Flaws → `wizard-next` disabled, that
   rail item marked blocked, the docked panel shows `unbalanced_virtues` and **not** an
   issue from another phase (proves the filter).
5. Switch to Advisory → Next enabled with the illegal state standing; back to Enforced →
   blocked again.
6. Remove the Virtue → unblocked; advance to `review`: `wizard-next` gone,
   `wizard-finish` present.
7. Back two steps, then rail-click forward to the furthest visited step → lands there.
8. `wizard-finish` → `role="tablist"` present, `wizard-rail` gone, and the name typed in
   the wizard is still in `identity-name`.
9. `save-button` and `mode-select` exist while the wizard is up.

Only two existing e2e files change (`helpers.js`, `app-entry.e2e.js`); the other 24 go
through the untouched `startCharacter`.

---

## Slice 6b3a — Sample Childhood packages: data, engine, IPC ✅

6b3 was split at the IPC boundary, exactly as 6b1 was: **6b3a** is the engine, the
catalogue and the command — independently green with an unchanged UI — and **6b3b** is
the frontend (funding-mode toggle, life-stage panel, package picker, the XP bar's guided
branch, e2e). The `PLAN.md` 6b3 boxes tick in 6b3b, not here: 6b3 is only done when a
player can actually take a package.

### What shipped (17 commits)

- **Types and pricing.** `crates/arm-rules/src/childhood.rs` — `ChildhoodPackage` /
  `ChildhoodEntry`, `native_entry`/`spread_entries`, and `spread_xp`/`native_xp` pricing a
  package against `AdvancementTable::xp_for_score` (`86ff068`, `6dc7fd6`). Registered on
  the `Ruleset` from its own source (`85f8225`).
- **Load-time integrity.** Twelve rules in `Ruleset::validate_childhood_packages`
  (`c949216`), the 45/75 re-pricing among them — the trust gate on a hand-transcribed
  catalogue. The 6b2 childhood ref checks moved so `from_serialized` enforces them too,
  i.e. a cached ruleset is trusted no further than a freshly parsed one (`74f746a`).
- **Storage.** `LifeStagePlan.childhood_package` — the taken package's id, additive, so
  `SCHEMA_VERSION` stays 14 (`a277096`).
- **The applicator.** `apply_package` / `apply_childhood_package`: a monotone raise keyed
  by `(ability, parameter)`, slot values resolved into the rows' own `parameter`, every
  rejection collected in one pass (`171258d`, `02b9414`, `33dbcde`).
- **Localization seam.** `validation::childhood_rejection_issues` maps the rejections onto
  four codes — `childhood_slot_unfilled`, `childhood_slot_is_native_language`,
  `childhood_slot_duplicate_value`, `childhood_package_unknown` — with Fluent keys in both
  locales (`096e786`).
- **Data.** `rules/core/childhoods.json` plus `rules/i18n/{en,de}/childhoods.json`: the
  five packages of `:2384-2388`, one `source` line each; German names read off the German
  mirror at the same lines (`65e935d`). Read by the app loader with the "an empty file
  means the ruleset ships none" idiom `life_stages.json` uses (`be9dbee`), and applied
  over IPC by the `apply_childhood_package` command returning `ChildhoodApplication`
  (`0fed0af`).
- **Four latent 6b2 defects, fixed here** (`f3adc40`, `23f12c9`, `8d7ecdf`, `ab529d8`) —
  see below.

### Deliberate deviations from the 6b3 design below

- **Only the package id is stored — no `childhood_slots`.** *Hard problems, decided* and
  the 6b3 slice text both proposed storing the slot answers on the plan. They are not:
  a slot value **is** the Ability row's `parameter`, so storing it twice creates two
  representations of one fact that can disagree (edit the row, and the stored slot lies).
  `LifeStagePlan` therefore carries `native_language` and `childhood_package` only.
- **The stored package narrows nothing.** The design had the taken package restrict the
  45-xp spread pool's eligibility. It does not: eligibility stays the closed eleven-ability
  list of `:2378`, because no passage forbids the other eight once a package is taken and
  `:2382` explicitly invites adjusting one. `xp_allocation` never reads
  `childhood_package`; the pool is identical whether a package was taken or the points
  were divided by hand. The stored id is a **for-the-record annotation** — `validate()`
  checks only that it resolves (`childhood_package_unknown`), never that the rows still
  match it.
- **Three of the four codes are command-input rejections, not `validate()` findings.**
  Applying a package is all-or-nothing, so no stored character can *hold* an unfilled or
  colliding slot; `childhood_slot_*` therefore come only from
  `childhood_rejection_issues`, describing the form the player just submitted. Only
  `childhood_package_unknown` is a real `validate()` error — precisely because that id is
  the one thing persisted. (The emit sites still live in `validation/`, so the
  contract-table and phase scanners keep seeing every `(code, phase)` pair.)
- **A magus life-stage plan is refused, not costed.** `:2364` grants a magus **four**
  periods; this engine models two, and `later_life_years` would swallow apprenticeship and
  life as a magus both — 825 points for a 60-year-old, spendable on Arts as well through
  the shared pool. `validate_life_stage_plan` emits
  `life_stage_magus_guided_unsupported` (error, `abilities`) whenever a profile with
  `is_magus` carries a plan (`ab529d8`). A magus keeps the directly-entered pool, which is
  fully functional; magus life stages remain 6b4/6b5.
  **Superseded by 6b4** — the code const, the contract row, the emit site and both
  Fluent keys are gone. A guided magus is funded by its apprenticeship, its later life
  stops at the Gauntlet, and the age entered is the Gauntlet age; only the years *after*
  the Gauntlet were still unmodelled.
  **And superseded again by 6b5**, which models those years: the panel now asks for the
  age *and* the Gauntlet age, `LifeStagePlan::gauntlet_age` records the second, and the
  span between them earns 30 points a year (`:2216`, `:2471`). All four of `:2364`'s
  periods are costed, so nothing about a guided magus is refused or deferred any more.
- **Three further latent 6b2 defects were fixed in passing**, since 6b3 depends on all
  three being right: the native-language check now matches the childhood's
  `native_language_ability` rather than any parameterized Ability whose parameter happens
  to equal the language (`f3adc40`); an unset age is reported as
  `life_stage_age_unset` instead of silently withholding the childhood budget and blaming
  every childhood row as unfunded (`23f12c9`); and `restricted_xp_unspent` now names the
  pool it is about via `RestrictedXpPool::origin` (`origin_kind` + `origin`), so a
  life-stage character no longer gets two warnings that differ only in their numbers
  (`8d7ecdf`).

### Deferred to 6b3b (all shipped — see the next section)

The Abilities step's two funding modes (flat pool as today, guided life-stage flow), the
life-stage panel (age, native language, the two block read-outs), the package picker with
a field per slot calling `apply_childhood_package`, the XP bar's guided branch, and the
e2e spec. Provenance for all of the above: **Sample Childhood packages (M6/6b3a)** in
`crates/arm-rules/RULES.md`.

---

## Slice 6b3b — The guided Abilities step ✅

The frontend half, and the one that makes 6b3 real: before it, the catalogue, the
applicator and the command shipped with no surface calling them. No engine change was
needed — 6b3a's command was the whole contract — so this slice is UI, state and
localization only, plus the spec that drives the flow through the real binary.

### What shipped (14 commits, `f4d5280..f812644` plus `c1772f9`)

- **The types mirror** (`f4d5280`). `LifeStagePlan`, `LifeStageBudget`,
  `ChildhoodPackage`/`ChildhoodEntry` and `ChildhoodApplication` in `ui/src/lib/types.ts`,
  pinned against the Rust shapes by `crates/arm-app/tests/commands.rs`.
- **The derived funding mode** (`58e84ec`). `store.abilityFunding` is
  `entity.life_stages ? 'life_stages' : 'pool'`, and `setAbilityFunding` is the only way
  across: entering adds an empty plan and zeroes `xp_pool` (the engine forbids both at
  once), leaving deletes the plan key outright.
- **The plan's own fields** (`a5eb2b3`, `bbaeeae`). `setNativeLanguage` writes
  `plan.native_language` (blank deletes the key, and it is a no-op without a plan, so a
  flat-mode surface cannot conjure one); the chrome's Fluent keys land in both locales.
- **The XP bar's guided branch** (`e009889`). Under a plan the editable pool input gives
  way to a read-only `xp-pool-total`, with the later-life line
  (`years × rate = xp`) beside it; the restricted-pool read-outs name their block
  through `xp-pool-childhood_*` rather than the slug.
- **The panel** (`b0d75ba`, `1412efd`). `LifeStagePanel.svelte` — the funding radio, the
  age input with the engine's age→cap echoed beside it, and the native language — mounted
  by `AbilityTab` above the Ability rows, so the wizard step and the editor tab get it
  from one place.
- **The draft, the picker and the application** (`b95cd9c`, `8574573`, `9c0a930`,
  `89d466f`, `85e94d5`). `store.childhoodDraft` + `childhoodRejections`,
  `applyChildhoodPackage` over IPC, the `childhoodSlots` / `childhoodEntryPreview` /
  `childhoodSlotFault` / `restrictedPoolLabel` helpers in `derive.ts`, and
  `ChildhoodPackagePicker.svelte`.
- **The 27th e2e spec** (`ea7dae7`, `f812644`).
  `ui/e2e/specs/life-stage-childhood.e2e.js` drives the whole narrative on a companion —
  switch the funding source, type an age and a native language, draft Traveling Childhood,
  watch the two blocks fill, re-take it idempotently, switch away and back, save/load, and
  finish on a magus being refused. The wizard's phase-advance driver moved into
  `ui/e2e/helpers.js` (`advanceWizardTo`) rather than being copied.
- **Pruning the draft on the way out** (`c1772f9`) — see the decisions below.

### Decisions of record

- **The funding mode is derived, never a second flag.** A plan on the entity *is* guided
  funding, so there is nothing to reconcile on load and no way for a flag and a plan to
  disagree. The save shape is unchanged, and `SCHEMA_VERSION` stays 14.
- **The draft is UI state; the taken package is the record.** `store.childhoodDraft`
  (package id + slot answers) lives beside `filters`: never on the entity, so drafting
  cannot dirty the document and nothing about a half-filled form is ever saved. What
  persists is `life_stages.childhood_package`, written by the engine's applicator. The
  select therefore starts **unselected even when a package is recorded** — a recorded
  package is history, not a draft, and prefilling it would show empty-slot faults for a
  form the player never opened (the slot answers are the Ability rows' `parameter`s and
  are deliberately not stored, so they could not be restored anyway).
- **Leaving guided funding prunes the draft** (`c1772f9`). Leaving deletes the plan, and
  the plan is what a draft is *for*: a survivor would prefill a package for a future plan
  that starts from nothing, with stale slot faults for a decision nobody has made.
  Deliberately asymmetric — *entering* guided mode keeps an in-progress draft, since
  toggling the radio back and forth would otherwise destroy typed slot values.
- **The guided radio is disabled for a magus, in engine and UI.** The engine's
  `life_stage_magus_guided_unsupported` is the backstop for a hand-edited save; the UI
  reads `type_profile.is_magus` and disables the option with its reason spoken through
  `aria-describedby` (`ability-funding-magus-reason`), never greyed out alone. Both lift
  in 6b4.
- **The panel carries a second age input, bound to the same `entity.age`.** Age already
  lives on the Details tab, but later life is (age − childhood years) × rate, so the
  guided flow cannot ask the player to leave for it. Both surfaces read and write the one
  field, so they cannot diverge.
- **Rejections stay out of `result`.** `childhoodRejections` is its own store field: a
  rejection describes the *form just submitted*, not the character (which a rejection
  leaves untouched), and it is retired by the next apply and by every draft edit.

### Two findings the e2e spec pinned

- **The 75-point childhood block only forms once the native language is named.** It is an
  instance-restricted pool over the chosen language, so before the language exists there
  is no instance to restrict it to and only the 45-point spread is on screen. The slice
  design expected both blocks as soon as guided funding was picked; the spec asserts the
  real order instead.
- **Leaving guided mode deletes the recorded package with the plan.** The plan *is* the
  record (`life_stages.childhood_package`), so switching to the flat pool and back leaves
  no childhood taken — the bought Ability rows stand, as ordinary rows, and the now
  unfunded spend surfaces as `not_enough_xp`. That is the same "deleting the plan prunes
  its contents" rule as `native_language`, not a special case.

---

## Slice 6b4 — The magus's apprenticeship ✅

The period 6b2 left out. Apprenticeship is now the magus's third life-stage block, and
modelling it is what lifts 6b3a's refusal of a guided magus: the option is live, the
240 points fund Arts and Abilities alike, later life stops at the Gauntlet, and every
magus — guided or flat — is held to the minimum Abilities the Order admits.

### What shipped

**Engine (20 commits, `2c07793..3c0eeeb`).** `ApprenticeshipRules` (years, xp, the
minimum and recommended Ability lists, `recommended_xp`) as an optional third block of
`rules/core/life_stages.json`, re-priced and ref-resolved at load; `LifeStageBudget`
gains `apprenticeship_years`/`apprenticeship_xp` and `total()` counts them;
`later_life_years = age − childhood − apprenticeship` for a magus (`:2214`), so the age
entered is the Gauntlet age; `life_stage_age_before_gauntlet` (error) for a magus under
20; the pool restructuring below; `magus_minimum_abilities` as the single reading of
`:2437`/`:2451-2461`, feeding both `EffectiveScores` and the two new findings
(`magus_minimum_ability` error, `magus_recommended_ability` warning, phase `abilities`);
and the deletion of `life_stage_magus_guided_unsupported` — code const, contract row,
emit site and both Fluent keys.

**Frontend (12 commits, `e1bd0cd..8da4862`).**

- **The types mirror** (`e1bd0cd`). `EffectiveScores.xp_general_pool` and
  `magus_minimum_abilities`, plus `MagusMinimumAbility`/`AbilityRequirementKind`. The
  Rust-side drift test now serializes a populated `MagusMinimumAbility` into its
  `mirrored_keys` pass, so a rename of `met` or `requirement` fails in Rust rather than
  leaving `types.ts` compiling against a key the engine no longer sends.
- **The locale strings** (`573c73b`). `life-stage-apprenticeship`,
  `life-stage-gauntlet-note`, and the six checklist keys, in both locales. German from
  the rulebook mirror and the tables: **Lehrlingszeit** (`Basisregeln.md:2433`),
  **Lehrlingsprüfung** (`translation-tables/grundbegriffe.md:57`, `Basisregeln.md:2449`),
  **Mindestfertigkeiten** (`:2437`), **Empfohlene Mindestfertigkeiten** (`:2451`).
- **The panel** (`01e2b80`). `LifeStagePanel` drops the disabled attribute, the reason
  node and `ability-funding-magus-reason` (key and comment blocks with it), and gains a
  `role="status"` Gauntlet note above the age input it qualifies.
- **The XP bar** (`c76f166`). Under a plan the pool is the engine's `xp_general_pool`
  rather than a re-derived later life, and an apprenticeship line sits above the
  later-life one — gated on `apprenticeship_xp > 0`, so there is zero magus branching in
  the component and the non-magus bar is untouched.
- **The checklist** (`abf0080`, `fd09b0b`). `MagusMinimumAbilities.svelte`, mounted by
  `AbilityTab` as a root-level sibling between the funding panel and `.region-row`.
- **E2E** (`a175bdc`, `39bbc49`, `8da4862`). `satisfyMagusMinimums()` in
  `ui/e2e/helpers.js`; the childhood spec's closing test inverted; and the 28th spec,
  `ui/e2e/specs/magus-apprenticeship.e2e.js`.

### Decisions of record

- **The general pool is `EffectiveScores.xp_general_pool`, not a budget field.** A
  `general_xp` on `LifeStageBudget` would carry the block's *base* (240) while the solve
  funds base + `general_xp_bonus` (300 with Skilled Parens), so the bar would show a
  pool the engine does not use. The budget keeps `apprenticeship_xp` as the block, and
  the resolved pool crosses the boundary once, engine-authoritative.
- **Which block is the general pool depends on the character.** For a magus it is
  apprenticeship — "These experience points can be spent on Arts or Abilities"
  (`:2435`) — and later life becomes a restricted, Abilities-only pool that excludes
  Academic/Arcane/Martial unless a Virtue permits them. For everyone else nothing
  changed: later life is still the general pool and never appears as a restricted row.
  The companion assertion in `life-stage-childhood.e2e.js` is annotated as the lock on
  that.
- **Latin is matched by Ability id.** `ability.dead_language >= 1` satisfies `:2437`, so
  a magus whose only dead language is Greek passes. This **supersedes** this tracker's
  own 6b4 sketch (`Slice contents`, "matched on the Latin *instance*"): an instance
  value is free-text player input with no localization path — a German player types
  "Latein" — so an instance match would fail for every non-English user, which is worse
  than under-enforcing. `AbilityRequirement.parameter` is the field a future language
  registry fills to tighten it, in data rather than code.
- **The score examined is the BOUGHT one.** `effective_ability_score` returns 2 for a
  magus with a Puissant Parma Magica and no Parma row at all, and a Virtue's +2 to *use*
  is not training the Order can examine.
- **No second spell-level budget.** `:2435`'s "120 levels of spells" is already the
  magus profile's `spell_levels` in `rules/core/character_types.json`, with
  `spell_levels_base` its single selector. The two numbers of one sentence live in two
  files on purpose; `ApprenticeshipRules` carries only the experience.
- **One age finding, not two.** A magus under 20 gets `life_stage_age_before_gauntlet`
  alone — `life_stage_age_before_childhood` would pile a second, less specific sentence
  onto the same field.
- **No `apprenticeship_start_age`.** Apprenticeship is fifteen *fixed* years ending at
  the age entered, so a start age is derivable and stores nothing new. It only becomes a
  question in 6b5, where the years after the Gauntlet start being counted. **Still true
  after 6b5**, which stores `gauntlet_age` — the *end* of apprenticeship — for exactly
  the same reason.
- **Guided funding builds a magus AT its Gauntlet — the limit 6b5 lifts.** An
  experienced magus's Arts come from the post-Gauntlet 30 points a year (`:2216`,
  `:2471`), which is not modelled, so entering 60 would silently earn a magus of 60 the
  same 240. The `life-stage-gauntlet-note` says both halves in words: what the age means,
  and that an older magus should use the flat experience pool for now.
  **Superseded by 6b5.** The age and the Gauntlet age are now two fields, the years
  between them are costed, and the note no longer sends anyone to the flat pool — the
  e2e spec asserts the sentence does *not* contain "experience pool". A guided magus of
  60 gauntleted at 25 is a first-class character.
- **The checklist resolves the instance from the entity.** `MagusMinimumAbility.parameter`
  is `None` throughout the shipped data (see above), so a dead-language row would read
  only "(Language) (Dead Language)". The component fills the token from the character's
  own highest-scoring matching row — the very row the engine's `score` came from — so it
  reads "Latin (Dead Language)". Display only; nothing is re-derived.
- **The recommended package's 90 is an argument, not a literal.** `magus-recommended-hint`
  takes `{ $xp }` from `life_stages.apprenticeship.recommended_xp`, so the rules number
  stays in the rules file rather than being transcribed into two `.ftl` files.
- **Each checklist row is a whole sentence per status.** `magus-minimum-met` /
  `magus-minimum-unmet` rather than one template plus a bolted-on "met"/"unmet" word:
  natural German (`uebersetzungsregeln.md`) and a status no screen reader can miss.
  `data-met` only mirrors it, and the summary is the single `role="status"` region —
  a live region per row would announce seven sentences on every keystroke.

### One finding the e2e spec pinned

- **`.region-row` collapsed to nothing under the checklist.** It is `flex: 1` with
  `min-height: 0`, so it gives up all of its height to the auto-height siblings above it:
  on the magus's Abilities step (funding panel 112px + checklist 224px) it measured **0**
  in an 800×1100 window, and the Available/Selected lists were not merely cramped but
  gone — their spinners reported "element not interactable" to WebDriver. Fixed with a
  `min-height: 12rem` floor (`12ba515`), which turns the collapse into a scroll of the
  enclosing `.tab-content` (`overflow-y: auto` for exactly this reason). The unit test's
  depth assertion could not have caught it: the checklist *is* a correct sibling, and the
  fault was the height budget, not the DOM shape.

Two smaller corrections, both in the specs rather than the app: a guided magus reads 240
**before any age is typed** (apprenticeship is a fixed block), so a wait on the pool
total races the round-trip — wait on later life's own figure; and the save/reload
round-trip has to come **after** Finish, because loading a file lands in the editor and
there is no wizard left to advance.

### Two deliberate non-changes

- **The Markdown export labels the later-life pool by its eligibility list.**
  `crates/arm-rules/src/export.rs:704-743` renders `xp.general_pool` — so a guided magus's
  sheet correctly reads `used / 240` — but labels every restricted pool by what it may
  buy, never by where it came from, so later life prints as a long category enumeration.
  Numbers right, label poor. Labelling by origin (the XP bar already does it, through
  `xp-pool-<block>`) is a **6b8** follow-up.
- **Flat mode ignores `general_xp_bonus`.** Without a plan the bar's pool is
  `entity.xp_pool`, so a flat magus with Skilled Parens spends the extra 60 the engine
  grants and shows a negative Available with no error. Pre-existing (the bonus predates
  this slice) and deliberately not folded in here, because the flat pool is the number
  the player typed and the e2e suite drives exact arithmetic against it — also **6b8**.

---

## Slice 6b5 — Post-Gauntlet accrual ✅

The fourth and last of the periods `:2364` names, and the one that makes the guided
magus general rather than a snapshot: 6b4 built a magus standing *at* its Gauntlet, and
this slice lets it have lived on. "For every year, the magus gets 30 points. Each point
can be an experience point in an Art or Ability or one level of spell"
(`Ars Magica - Definitive Edition (Core Rules).md:2471`), less "10 points from the yearly
30" for each season of lab work, "to a minimum of 0 if three or four seasons are spent"
(`:2482`), all under `:2216`'s "**Hermetic Magi Only (Optional):** Years after
apprenticeship". Full provenance — data, arithmetic, findings and every rejected
alternative — is `#### Life as a magus after the Gauntlet — 30 points per year (M6/6b5)`
in `crates/arm-rules/RULES.md`; this section records the slice, not the rule.

Split at the IPC boundary exactly as 6b1 and 6b3 were: **6b5a** is independently green
with an unchanged UI that merely carries the new fields, **6b5b** is the surface that
uses them.

### What shipped

**Engine — 6b5a (14 commits, `6cad5e0..bb35a5d`).** `PostApprenticeshipRules`
(`points_per_year` 30, `lab_season_cost` 10, `max_charged_lab_seasons_per_year` 3) as an
optional fourth block of `rules/core/life_stages.json`, with
`Ruleset::validate_post_apprenticeship_rules` refusing a transcription whose
`lab_season_cost × max_charged` does not equal `points_per_year` and a load-time gate
requiring the block wherever an `is_magus` profile and life-stage rules meet;
`LifeStagePlan` gains `gauntlet_age`, `post_gauntlet_lab_seasons` and
`post_gauntlet_spell_levels`, `LifeStageBudget` the five derived figures
(`gauntlet_age`, `post_gauntlet_years`/`_points`/`_spell_levels`/`_xp`); `budget()`
re-derived around the Gauntlet age rather than the character's; `xp_allocation`'s
`base_general` becomes `apprenticeship_xp + post_gauntlet_xp`; `spell_levels_budget`
becomes `base + bonus + life_stage_spell_levels`; three new errors plus
`life_stage_aging_rolls_pending`, and `life_stage_age_before_gauntlet` retargeted onto
the resolved Gauntlet age; `EffectiveScores` splits the spell-levels budget into
`spell_levels_profile_base`/`_bonus`/`_life_stage`; and RULES.md rewritten so the
life-stage provenance describes all four periods rather than two.

**Frontend — 6b5b (6 commits, `624609e..302a54d`).**

- **The store setters** (`624609e`). `setGauntletAge`, `setPostGauntletLabSeasons` and
  `setPostGauntletSpellLevels` write the three plan fields. Each is a no-op without a
  plan (so a flat-mode surface cannot conjure one) and each deletes its key when blank or
  zero, so a magus standing at its Gauntlet still saves nothing but its native language.
- **The panel** (`629cb63`). `LifeStagePanel.svelte` gains the Gauntlet age, the lab
  seasons and the spell-level split, plus a `life-stage-post-gauntlet-summary` read-out
  of the engine's own figures (`{years} years as a magus: {points} points = {xp} XP +
  {levels} levels of spells`). Gated on `is_magus && rules.post_apprenticeship != null` —
  a ruleset shipping no such block would otherwise show three dead controls. The three
  fields join the age and the native language on **one wrapping row** (`app.css`), because
  the panel is an auto-height sibling of the `flex: 1` `.region-row` and every full-width
  row is height taken straight off the Available/Selected lists — the lesson 6b4's
  collapse taught.
- **The XP bar** (`92203d0`). A `life-stage-post-gauntlet` line, `years × rate - lab for
  lab work = points, xp`. The rate is read out of
  `ruleset.life_stages.post_apprenticeship.points_per_year` (never a literal 30) and the
  lab deduction is `years × rate − points`, derived back from the engine's own figure
  rather than recomputed from the season count. Gated on `post_gauntlet_years > 0`, not on
  the type, so there is no magus branch in the component.
- **The spell bar** (`b71bf23`, `ddb6527`). `spellLevelAllocation` takes a fourth term
  and subtracts it: `base = budget − bonus − lifeStage`, so a 35-year magus that took 300
  levels reports the profile's 120 as its editable base instead of 420. The levels are
  already earned, as unconditional as the base, so they sit on the base's side of the
  split — they raise Available rather than forming a pool, and a Weak Parens penalty is
  still charged to the base. `SpellBudgetBar` lists them as a muted
  `spell-levels-post-gauntlet` row.
- **The 29th e2e spec** (`302a54d`). `ui/e2e/specs/magus-post-gauntlet.e2e.js` drives one
  magus of 60 gauntleted at 25 end to end against the real binary: later life 5 × 15 = 75
  (the 6b4 regression lock), 35 × 30 = 1050 points, 10 charged seasons = 950, a 300-level
  split leaving 650 XP, a general pool of 240 + 650 = 890 spent by both the Abilities and
  the Arts step, a spell budget of 120 + 300 = 420, each of the three errors raised and
  cleared with `Next` blocking and unblocking, the aging warning leaving Finish live, and
  a save asserting **exactly** the four stored keys.

### Decisions of record

- **One stored choice: `LifeStagePlan::gauntlet_age`.** `age = childhood + later life +
  apprenticeship + post-Gauntlet` is one equation in two unknowns, so exactly one has to
  be recorded. **Absent means the magus stands at its Gauntlet** — the Gauntlet age is
  then `Entity::age` itself, which is precisely what 6b4 computed, so every pre-6b5 save
  reads identically, the field is purely additive and `SCHEMA_VERSION` stays **14**.
- **`post_gauntlet_years` was rejected as the stored value.** With the years stored
  instead, raising a magus's age would stretch its *childhood-to-apprenticeship* span —
  the years before it was taken as an apprentice — rather than its life as a magus, which
  is the opposite of what raising the age means. `later_life_years(stop_age, …)` is
  therefore fed the Gauntlet age, which for anyone serving no apprenticeship is the age.
- **Lab seasons are one total of *charged* seasons, not a per-year list.** The deduction
  stops at the third season of a year (`:2482` reaches 0 there, so the fourth is free), so
  every legal per-year distribution totals at most `3 × years`, every total in that range
  is realizable, and all of them cost the same — one number is lossless and the per-year
  clamp collapses into one range check. It has to count *charged* seasons: sixteen seasons
  actually worked cost 0 across four years but 30 across five, and a single total of
  worked seasons could not tell those apart.
- **The post-Gauntlet experience is the general pool — no new `LifeStageBlock`.**
  `:2216`/`:2471` let each point buy an Art or any Ability, and only the general pool may
  fund an Art (`pool_covers` is false for every `(Ability pool, Art spend)` pair); `:7151`
  puts the years after apprenticeship explicitly on the permitted side of the
  Academic/Arcane/Martial gate. So **no** new `LifeStageBlock` variant, **no**
  `RestrictedXpPool`, **no** `xp-pool-<slug>` Fluent key — the single biggest
  simplification in the slice. Later life stays the restricted, Abilities-only pool
  however many years the magus has lived since
  (`post_gauntlet_years_leave_later_life_restricted`).
- **The spell levels are additive to the profile's 120, not a second budget.** Folded in
  inside `spell_levels_budget` (`base + bonus + life_stage`), which is the single selector
  both `validate_spells` and the `EffectiveScores` payload call — so `validate_spells`
  needed **no change of its own**, and a change there would have meant the term was in the
  wrong place. The payload carries the three parts separately, identity
  `base + bonus + life_stage == spell_levels_budget`, so the bar labels each rather than
  showing an unexplained total.
- **The split is chosen on the Abilities step, and its finding is filed there.** One
  number defines both the experience pool and the spell budget, and the magus phase order
  is `… abilities, arts, spells`, so editing it from the Spells step would retroactively
  shrink a pool already spent two steps earlier — hence no input there, only a read-out,
  and hence `life_stage_spell_level_split_exceeds_points` under `abilities` despite
  feeding `spell_levels_budget`. Filing it under `spells` would let the wizard walk past
  the only surface that can correct it.
- **The clamps in `budget()` stay, and the validators name what they absorb.**
  `ValidationMode::Advisory` downgrades every issue and `Silent` drops it, so neither
  blocks a save and the arithmetic has to stay sane whatever a file holds — which is what
  `min`/`saturating_sub` guarantee. The price is that a wrong number *vanishes* rather
  than failing, so each of the three findings names the value the clamp swallowed: the two
  together are honest.
- **The Gauntlet age is resolved only for a character that serves an apprenticeship.**
  `:2216` is "Hermetic Magi Only", so `budget()` reads the stored age through
  `apprenticeship.and(plan.gauntlet_age)`. Gating on the *points* being zero instead would
  let a hand-edited companion plan carrying `gauntlet_age: 25` at age 60 silently lose 35
  later-life years (525 experience points).
- **`life_stage_age_before_gauntlet` was retargeted, not duplicated.**
  `minimum_gauntlet_age()` (childhood + apprenticeship) is now compared against
  `LifeStageBudget::gauntlet_age` — read off the budget rather than re-derived, so the
  finding and the arithmetic cannot drift. A magus of 60 gauntleted at 12 never served its
  fifteen years either, and only the Gauntlet age sees that.
- **`life_stage_aging_rolls_pending` fires for every character over 35**, not only a magus
  with a plan: "a character over the age of 35 must make aging rolls before the game
  begins" (`:2232`, `:16565`) is about the character, and the post-Gauntlet years merely
  make the case routine. Strictly over — aging begins "in the Winter after they turn 35",
  so 35 owes nothing and 36 owes the first roll. It is a **warning** on phase `review`,
  so Finish stays live. The threshold was a cited Rust constant
  (`AGING_ROLLS_START_AGE`) until 6b6a created `rules/core/aging.json`; it now reads
  `AgingRules::first_roll_age()`, and the constant is gone. The `review` phase is still a
  placeholder and says so at the emit site: the finding moves onto the new `Aging` phase
  when 6b6b adds the variant.

### Findings the e2e spec pinned

- **The Review step renders the validation panel twice.** `WizardReview.svelte` mounts an
  unscoped `<ValidationPanel />` in its body and `WizardShell.svelte` the docked one in
  the footer, and both carry `data-testid="issue-list"` — so on Review, and only there,
  every finding is on screen twice. By design (the body panel reports the whole character,
  the docked one the step), but a trap for any future spec that counts issues on Review:
  this one counts the aging warning loosely (`> 0`) and keeps its exact counts to the
  earlier steps, where the docked panel stands alone.
- **A guided magus reads its post-Gauntlet figures only once an age *and* a Gauntlet age
  exist.** `post_gauntlet_years` is `age − gauntlet_age`, and an absent Gauntlet age reads
  as the age, so the summary shows nothing until both are typed — the spec waits on
  `35 years` rather than asserting immediately after the age.

### Two deliberate non-changes

- **The Markdown export still renders only `xp.general_pool`.** `export.rs:704-743` prints
  the pool as one `used / total`, so a post-Gauntlet magus's sheet reads `used / 890` —
  right numbers, no breakdown of which block contributed what. This is the same labelling
  gap 6b4 recorded (restricted pools named by their eligibility list rather than their
  origin) and it stays a **6b8** follow-up; the XP bar already labels by origin, the sheet
  does not.
- **`spell_levels_override` and the split coexist.** The override still replaces the
  **profile base** alone, with the V/F bonus and the post-Gauntlet levels additive on top.
  Deliberately *not* made exclusive with a plan the way `xp_pool` is
  (`life_stage_xp_pool_conflict`): the override is the flat flow's escape hatch, and the
  two answer different questions.

---

## Slices 6b2-6b8 — design notes

### What already exists (verified in code — these engines add less than PLAN.md implies)

Already shipping and **not** to be re-implemented: the **advancement table** as data
(`rules/core/abilities.json` → `advancement`; `ability.rs:148` `AdvancementTable`); the
**age→max-score band table** as data (`abilities.json:24` `age_ability_caps`; enforced at
`validation/scores.rs:292-322` with Affinity's +2 already applied); the **shared
Ability+Art xp pool** as an Edmonds-Karp max-flow over a general pool plus one pool per
`Effect::RestrictedAbilityXp` (`effective.rs:924` `xp_allocation`, `PoolEligibility`
`:877`); the **120 spell-level budget** and the **Te+Fo+Int+MT+3 spell cap**
(`effective.rs:1445-1502`); **Decrepitude** rising up the advancement table and the
`|score|+1` aging-drop derivation with its shrinking threshold (`effective.rs:2077`,
`:2103`); `aging_points`, `apparent_age`, `longevity_ritual`, `aging_log` as
direct-entry fields.

So the engines add **budget provision** (new pools feeding the existing solver),
**package application**, and **the aging transformation** — plus five genuinely missing
rules: the 2437 minimums, the 2392/2394 rates, the 2315 authorization gate, the
aging/crisis tables, and Foreign Upbringing's halved cap.

**Corrected source citations** (verified against the file; `PLAN.md:606-609` has one
wrong): apprenticeship is **2433-2437**, not 2433-2435 — the Parma Magica 1 / Magic
Theory 1 / Latin 1 minimums are at **2437**, outside the cited range. Also uncited but
load-bearing: the advancement table **2404-2431**, the spell cap **2465**, the childhood
11-ability closed list **2378**, the base 15 xp/year at **2392** with Wealthy/Poor and
"only companions" at **2394**, and the recommended minimums **2451-2461**. The
quick-reference reprints at **23575-23617** and **~23620-23648** must never be cited in
`source` fields; RULES.md gets a "duplicate reprint, keep in sync" note per table.

### The missing Poor Major Flaw (6b2)

**Verified gap.** `rules/core/virtues_flaws.json` has no `flaw.poor` — only
`flaw.poor_characteristic`, `flaw.poor_concentration`, `flaw.poor_living_conditions` and
friends (`:2126ff`) — and no item cites Core Rules 6594-6596, where the flaw lives:

> `#### Poor` / `*Major, General*` / "You are a poor member of your social class. You
> must work three seasons per year in order to make ends meet… In particular, this Flaw
> is not available to magi." — `Ars Magica - Definitive Edition (Core Rules).md:6594-6596`

A real hole in a catalogue M5 declared complete (the extraction plausibly dropped it
because "Poor" prefixes six other flaw names). Fixed **in 6b2** rather than earlier,
because the flaw's whole mechanical content is the 10-xp/year rate, and shipping the item
before its `LaterLifeXpRate` effect exists would add a flaw that silently does nothing.

All of this shipped in 6b2; the boxes below record it.

- [x] RED: the ruleset contains `flaw.poor` with `magnitude: major`, `category: general`
      and a `later_life_xp_rate` effect of 10 — structural, never a catalogue total.
      (`wealthy_and_poor_ship_with_their_rates_and_eligibility`, `tests/data_integrity.rs`.)
- [x] GREEN (data): add the item to `rules/core/virtues_flaws.json` in canonical
      (id-sorted) position with `source: { file: "Ars Magica - Definitive Edition (Core
      Rules).md", lines: [6594, 6596] }`, mirroring `flaw.poor_living_conditions`
      (`:2179-2187`).
- [x] GREEN (i18n): `rules/i18n/en/virtues_flaws.json` name "Poor" + summary from the
      source; `rules/i18n/de/virtues_flaws.json` name **"Arm"** — the translation table's
      value (`rules/source/de/translation-tables/tugenden-fehler.md:445`, "Konsistent im
      Text verwendet"), corroborated by the German source at the same mirrored lines
      6594-6596 (`#### Arm` / `*Groß, Allgemein*`).
- [x] RED → GREEN (rate): Poor gives 10 xp/year, Wealthy 20, neither 15 (2392/2394).
      (`LifeStageRules::later_life_rate`, `life_stage.rs`.)
- [x] RED → GREEN (eligibility): 2394 says "only companions can take this Virtue or
      Flaw" and 6596 repeats "not available to magi". Add both `flaw.poor` and
      `virtue.wealthy` to `forbidden_traits` on the **magus** *and* **mythic_companion**
      profiles — grog is covered incidentally by `max_major_virtues/flaws: 0`, but
      mythic_companion has `max_major_*: null` (`character_types.json:110ff`) and would
      otherwise take both legally. Record the 2394-vs-5237/6596 scope reading in
      RULES.md.
- [x] RULES.md provenance entry: the verbatim 6594-6596 excerpt, the
      `later_life_xp_rate: 10` data value, the eligibility ruling. (**Wealthy / Poor —
      the rate, and who may take them**, `crates/arm-rules/RULES.md`.)

`virtue.wealthy` already exists (`virtues_flaws.json:6294`) but is `classification:
narrative` with no effects, so it gains the same treatment in the same commit.

### Table decisions (data vs Rust taxonomy)

| Table | Verdict |
|---|---|
| Age→max score, advancement | **Data — already is.** No change |
| Later-life rates 15/20/10 | **Data on the V/F items** via a new `Effect::LaterLifeXpRate { amount }`; base 15 in `rules/core/life_stages.json` |
| Living Conditions (16581-16594) | **Data** in `rules/core/aging.json` + names in `rules/i18n/<lang>/aging.json` — an open catalogue of situations; the engine branches only on `modifier`/`cumulative` |
| Aging Roll table (16597-16611) | **Split**: rows are data, `AgingOutcome` is a Rust enum — the engine genuinely branches ("who picks the Characteristic", "does a Crisis follow"), so a new kind must be a compile error. Two arrays, because a roll of 15 both ages you a year *and* costs Sta a point |
| Crisis table (16621-16632) | **Split**: rows are data, `CrisisSeverity` is a Rust enum; ease factors and CrCo levels are numbers |
| Decrepitude levels | **No new table** — reuse `AdvancementTable::xp_for_score` |
| Sample Childhoods | **Data** in `rules/core/childhoods.json` + i18n names |

### Hard problems, decided

- **The 45-xp childhood spread without per-ability xp storage.** No ledger. The 45 xp is
  a **funding constraint** — a restricted pool whose eligibility is the closed
  11-ability list — so life-stage pools feed the *existing* `xp_allocation` max-flow
  solver. An Area Lore part-funded by childhood and part by later life is then solved
  globally, `restricted_xp_unspent` already warns about wasted childhood xp, and
  Affinity's discount already applies. One extension: `PoolEligibility::Ability` matches
  by ability **id**, but childhood needs **instance** granularity (the 75-xp pool covers
  only the *native* `living_language` instance; the 45-xp pool must exclude it) → add
  `instances` / `exclude` vectors, which V/F pools leave empty and behave
  bit-identically (regression-locked).
- **Parameterized package entries** ("Area A Lore", "Area B Lore", "Living Language 1").
  Each entry carries a `slot` — a stable choice key, mirroring `house_choices` /
  `warping_choices` — and player values land in `LifeStagePlan::childhood_slots`.
  (**Superseded in 6b3a:** slot values are *not* stored on the plan; they are the Ability
  rows' own `parameter`, and a second copy could only diverge. See the 6b3a section.)
  Load-time integrity is therefore **not** a plain slug lookup: refs resolve; a
  parameterized ability must carry a slot and a plain one must not; slots are unique per
  package; exactly one entry is `native: true` and it must be the language-parameterized
  ability; and **the arithmetic is verified** — each package's non-native entries must
  price to exactly 45 xp off the advancement table and the native entry to 75 (all five
  shipped packages check out). That test is the trust gate on transcribed data.
- **Stored audit trail vs computation aid.** Store the *choices*, derive every number,
  per "saves store choices, not resolved values". New
  `Entity::life_stages: Option<LifeStagePlan>` holding `native_language`,
  `childhood_package`, `childhood_slots`, `apprenticeship_start_age`,
  `post_gauntlet_spell_levels`, `post_gauntlet_lab_seasons`. (As shipped: 6b2 added
  `native_language`, 6b3a `childhood_package`, and **no** `childhood_slots` — see the
  6b3a section; the apprenticeship/post-Gauntlet fields land with 6b4/6b5.) When
  `life_stages` is
  `Some`, the derived pools are authoritative and `xp_pool` must be 0 — a non-zero value
  alongside a plan is an error (`life_stage_xp_pool_conflict`), never a silent
  double-count. Rejected: making `xp_pool` an `Option` override, which forces a
  migration and touches every consumer for no gain.
- **`SCHEMA_VERSION` stays 14.** Every new field is additive `#[serde(default,
  skip_serializing_if)]`, exactly like `mastery_abilities`, `warping_choices` and
  `LongevityRitual::focus` before it. No migration code.
- **Affinity vs whole-score storage: nothing changes.** `charged_cost` already charges
  `⌈cost·2/3⌉` and the +2 cap bump is already in `validate_abilities`. Add two regression
  tests, no code: a childhood-funded Affinity ability draws the reduced amount from the
  45-pool, and Puissant is charged **nothing** and never counted against the age cap.
- **Apprenticeship length is fixed at 15 years**; what varies is the *start age*, so the
  Gauntlet age is `start + 15`, never a constant 25.
- **The fungible post-Gauntlet "point" (2471).** Do not make spell levels a flow
  commodity. The player's xp↔spell-level split is a genuine choice the rules ask for, so
  store it and feed the two existing budgets. The rejected alternative (a
  `SpendKind::SpellLevel` in the flow graph) changes `over_spell_levels`'s semantics and
  the UI's bar for zero user-visible gain — record the rejection in RULES.md so it is not
  re-litigated.

### Slice contents

**6b2 — Life-stage XP engine (non-magus) + the Poor flaw.** New
`crates/arm-rules/src/life_stage.rs` (budget, pools, native-language instance, per-year
rate, year counts); new `rules/core/life_stages.json` (childhood 75/45 + the 11-ability
list; later-life 15; apprenticeship 240/120/15 + minimum and recommended ability lists;
post-apprenticeship 30 and the 10-per-lab-season cost) — language-neutral, so no i18n
file, with provenance in RULES.md; `Effect::LaterLifeXpRate` (a new `Effect` variant
cascades through ~12 exhaustive matches — that compile error *is* the design working);
the `PoolEligibility` instance/exclude extension; `RestrictedXpPool::origin` so the XP
bar can label "Early Childhood (45)" through Fluent instead of guessing from the ability
list. Plus the Poor Major Flaw work above. New codes (all phase `abilities`):
`life_stage_xp_pool_conflict`, `life_stage_age_before_childhood`,
`life_stage_native_language_unset` (errors), `life_stage_native_language_missing_score`
(warning). Two severable tails: the **2315/2390 authorization gate** for
Academic/Arcane/Martial Abilities (waived when `profile.is_magus`, per 2435/7151), and
**Foreign Upbringing**'s halved locality-dependent cap (a new `locality_dependent` data
flag on abilities).

**6b3 — Sample Childhood packages.** (Split into 6b3a/6b3b; **both have shipped** — read
the **Slice 6b3a** and **Slice 6b3b** sections above for what the design below actually
became. Notably the funding mode is *not* UI state as sketched here: it is derived from
the plan on the entity, and only the drafted package is UI state.)
`childhood.rs` (`ChildhoodPackage`, `apply_package`,
`package_cost`), `rules/core/childhoods.json` (five packages, one `source` each at
2384-2388) + `rules/i18n/{en,de}/childhoods.json`, the five integrity rules above, and an
`apply_childhood_package` command. The Abilities step gains its two modes — flat
allocation as today, and the guided life-stage flow — with the mode as UI state, never
saved. New codes (phase `abilities`): `childhood_package_unknown`,
`childhood_slot_unfilled`, `childhood_slot_duplicate_value` (two Area Lore slots resolved
to the same area would collapse into a duplicate row),
`childhood_slot_is_native_language` (2378: a Living Language *other than* the native one).

**6b4 — Magus apprenticeship.** `gauntlet_age`, the 240-xp apprenticeship pool, and
`validate_magus_minimum_abilities` reading the minimum/recommended lists from data (zero
slugs in Rust), matched on the Latin *instance* of `ability.dead_language` rather than any
dead language. `magus_minimum_ability` is an **error** (2437: "would not be admitted to
the Order"); `magus_recommended_ability` is a warning (2451-2461's 90-xp list).
`EffectiveScores.magus_minimum_abilities` lets the UI render a checklist.

**6b5 — Post-Gauntlet accrual.** (**Shipped** — read the **Slice 6b5** section above for
what this became. The arithmetic held; two details did not.) `years × 30 − Σ min(seasons,
3) × 10` clamped at 0 per year (2482: minimum 0 at "three or four seasons", so the fourth
is free), plus the stored xp↔spell-level split. New codes:
`life_stage_spell_level_split_exceeds_points` (error, `spells`),
`life_stage_lab_seasons_out_of_range` (error), and `life_stage_aging_rolls_pending`
(warning) — the seam 6b6/6b7 fill, from 2494/2496 and 2232's "must make aging rolls
before the game begins".

Two corrections to the above. **The split finding shipped under `abilities`, not
`spells`**: the number is typed on the Abilities step, and the magus phase order is
`… abilities, arts, spells`, so `spells` would have walked the wizard past the only
surface that can correct it (M6/6b1a's rule — a finding belongs to the phase whose input
surface owns the value). And a **fourth** code was needed that this sketch did not
anticipate, `life_stage_gauntlet_age_after_age` (error, `abilities`): storing a Gauntlet
age at all makes a Gauntlet in the character's future expressible, and `budget()` clamps
it, so without the finding a magus of 25 gauntleted at 40 would lose the years between
in silence. The per-year clamp also did not survive as written — the seasons are stored
as **one total of charged seasons**, so `Σ min(seasons, 3)` collapses to a single
`min(seasons, 3 × years)` (see the decisions above).

**6b6 — Aging tables and totals.** New `crates/arm-rules/src/aging.rs`: `aging_total` =
`die + ⌈age/10⌉ − living conditions − longevity bonus − aging-roll modifiers`, with `die`
**user-entered** and the doc comment saying so; 16575's pre-35 "treat 10+ as 9" clamp
applied to the die and reported as `capped_by_longevity` so the UI can explain the
number; 16577's "actual, not apparent, age" as a named test; `resolve_outcome`;
`aging_schedule`, whose first roll is at **36** (16565: "the Winter *after* they turn
35") — the one off-by-one in the engine, so it gets a named, commented test. New
`rules/core/aging.json` + `rules/i18n/{en,de}/aging.json`. This slice also adds `"aging"`
to `creation_phases` on all four profiles (and therefore an `Aging` variant plus its
Fluent keys), which 6b1's gating then consumes for free.

**6b7 — Crisis, Decrepitude levels, write-back.** `crisis_total` = `simple die +
⌈age/10⌉ + Decrepitude`, with 16636 ("Virtues that affect aging rolls do not affect
crisis survival rolls") as a named regression test — the rule most likely to be silently
broken later. `resolve_year` is the **only writer**: it adds aging points, bumps apparent
age, appends a log entry, raises Decrepitude **before** the crisis roll (16619), and
spends the Longevity Ritual on a crisis (16571) as a surfaced note rather than a silent
deletion. It never kills the character and never rolls a die. `AgingLogEntry` widens
additively (roll, total, conditions, points, crisis roll and severity) while
`aging_points` stays authoritative; divergence is a warning. Two new commands:
`aging_year`, and `aging_schedule` riding on `EffectiveScores`.

**6b8 — Per-type flow completion and the milestone gate.** Walk grog, companion, mythic
companion and magus end to end and close what the composed flow reveals — above all the
**completeness indicator** deferred from 6b1b, so a legal-but-empty phase is visibly
incomplete even though it does not block. Plus per-step guided copy sourced from the
rules passage, and the small ownership inconsistencies 6b1b deliberately froze (pull
`SpellBudgetBar` out of `SpellTab`; decide whether `store.filters.vf` should be
per-view). "Skip where allowed" happens here **only if** the data by then marks a phase
optional — nothing does today. One e2e spec per type asserting a complete legal character
from start screen to Finish with zero errors, plus a portable-layout smoke check, and
reconciliation of `PLAN.md`, `README.md` and `RULES.md`.

### Out of scope for M6 entirely

In-play advancement (seasons, books, teachers, vis); **any RNG** — `arm-rules` gains no
`rand` dependency; crisis *survival* outcome, the doctor's Medicine roll, and death (the
engine computes the total, Ease Factor and required CrCo level, then stops — no auto-kill
at Decrepitude 5); Twilight episodes; in-play aging after creation (M6 only covers 2232's
pre-play catch-up); deriving Living Conditions from a covenant entity (M8); lab-project
simulation beyond 2482's point deduction; child-character Characteristic/Size penalties
(2246-2258, a direct-entry concern); flow-graph unification of spell levels and xp
(rejected above); and any general "raises the age cap" mechanism beyond Affinity +2 and
Foreign Upbringing.
