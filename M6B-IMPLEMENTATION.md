# Milestone 6b — Implementation Tracker

Sequenced execution plan for 6b ("Guided creation wizard"). `PLAN.md:588-618` holds
the high-level 6b scope checkboxes; this file is the session-resumable execution
tracker with per-slice tasks, decisions, and verified source citations. Tick a box
here and the matching `PLAN.md` box in the same commit as green code.

6a is done (`PLAN.md:562-586`): the app opens on `StartScreen.svelte` and a character's
type is fixed at creation via `store.createCharacter(typeId)`. 6b1b replaced that
screen's hardcoded-disabled wizard button with one guided entry per character type.

**Status: 6b1a, 6b1b and 6b2 are done.** Next is 6b3 — Sample Childhood packages and
the sophisticated Abilities step.

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

One thing 6b2b does **not** model, and should be recorded rather than forgotten:
`:7151`'s finer "Magi without a specific Virtue may only buy Academic Abilities
during or after apprenticeship". A bought score carries no per-stage attribution in
this engine, so the magus exemption is whole-character. 6b4 (apprenticeship) is where
that could change.

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
| **6b1a** | Phase vocabulary + issue attribution across all 85 emit sites; Fluent keys; TS mirror. UI unchanged | 6a |
| **6b1b** | Wizard shell: view, rail, back/forward nav, per-phase gating, existing components mounted as steps | 6b1a |
| **6b2** | Life-stage XP engine (childhood 75+45, later life 15/20/10, pools into the existing solver) **+ the missing Poor Major Flaw** | 6b1a |
| **6b3** | Sample Childhood packages (data + integrity + apply) and the "sophisticated" Abilities step | 6b2 |
| **6b4** | Magus apprenticeship (240 xp / 120 spell levels / hard minimums) | 6b2 |
| **6b5** | Post-Gauntlet accrual (30 pts/year, lab-season deduction, xp↔spell-level split) | 6b4 |
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

- [ ] RED: the ruleset contains `flaw.poor` with `magnitude: major`, `category: general`
      and a `later_life_xp_rate` effect of 10 — structural, never a catalogue total.
- [ ] GREEN (data): add the item to `rules/core/virtues_flaws.json` in canonical
      (id-sorted) position with `source: { file: "Ars Magica - Definitive Edition (Core
      Rules).md", lines: [6594, 6596] }`, mirroring `flaw.poor_living_conditions`
      (`:2179-2187`).
- [ ] GREEN (i18n): `rules/i18n/en/virtues_flaws.json` name "Poor" + summary from the
      source; `rules/i18n/de/virtues_flaws.json` name **"Arm"** — the translation table's
      value (`rules/source/de/translation-tables/tugenden-fehler.md:445`, "Konsistent im
      Text verwendet"), corroborated by the German source at the same mirrored lines
      6594-6596 (`#### Arm` / `*Groß, Allgemein*`).
- [ ] RED → GREEN (rate): Poor gives 10 xp/year, Wealthy 20, neither 15 (2392/2394).
- [ ] RED → GREEN (eligibility): 2394 says "only companions can take this Virtue or
      Flaw" and 6596 repeats "not available to magi". Add both `flaw.poor` and
      `virtue.wealthy` to `forbidden_traits` on the **magus** *and* **mythic_companion**
      profiles — grog is covered incidentally by `max_major_virtues/flaws: 0`, but
      mythic_companion has `max_major_*: null` (`character_types.json:110ff`) and would
      otherwise take both legally. Record the 2394-vs-5237/6596 scope reading in
      RULES.md.
- [ ] RULES.md provenance entry: the verbatim 6594-6596 excerpt, the
      `later_life_xp_rate: 10` data value, the eligibility ruling.

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
  `post_gauntlet_spell_levels`, `post_gauntlet_lab_seasons`. When `life_stages` is
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

**6b3 — Sample Childhood packages.** `childhood.rs` (`ChildhoodPackage`, `apply_package`,
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

**6b5 — Post-Gauntlet accrual.** `years × 30 − Σ min(seasons, 3) × 10` clamped at 0 per
year (2482: minimum 0 at "three or four seasons", so the fourth is free), plus the stored
xp↔spell-level split. New codes: `life_stage_spell_level_split_exceeds_points` (error,
`spells`), `life_stage_lab_seasons_out_of_range` (error), and
`life_stage_aging_rolls_pending` (warning) — the seam 6b6/6b7 fill, from 2494/2496 and
2232's "must make aging rolls before the game begins".

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
