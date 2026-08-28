# Guided Creation — Implementation Plan

Companion to `docs/guided-creation-review-2026-08.md`. Written 2026-08-25 against
working tree `721224a`.

---

## 1. Purpose & scope

This plan sequences the fix work for the 31 numbered findings collected in
**`docs/guided-creation-review-2026-08.md`** — the authoritative issue source. That
document holds the symptoms, verified `file:line` anchors, root causes, rules
citations and the user's **DECIDED** resolutions; this document holds only what a
*plan* adds: dependency ordering, slice boundaries, TDD entry points, i18n and
schema impact, and the gating protocol. Nothing here re-argues a DECIDED item. Where
this plan and the review doc disagree on a fact, the review doc wins unless this
document explicitly records a verification that superseded it (see §8, "Review-doc
corrections").

**Plus one issue the review doc does not contain: #32.** Raised by the user on
2026-08-26 while answering D1 — the rules require **Latin** specifically, while the
engine requires any Dead Language. The review doc is deliberately **not** edited to add
it; it stays the historical record of the 2026-08-25 walkthrough. So **#32 lives here,
in §2 and Slice 7, and nowhere else** — a future session reconciling the two documents
should expect that asymmetry rather than treat it as drift.

**All three open decisions (D1, D2, D3) were answered on 2026-08-26** and are recorded
in §2 with their reasoning. No slice is blocked on a user decision any more.

**Anchor spot-check.** 16 of the review doc's cited locations were re-read against
the working tree while writing this plan (`app.css:540`, `:682-686`, `:705-712`,
`:1037-1040`, `:1522-1537`, `:1577-1580`, `:337-353`; `AbilityTab.svelte:201-217`;
`ParameterPicker.svelte:154-217`; `state.svelte.ts:420`, `:740-773`, `:1924-1929`;
`wizard-navigation.svelte.ts:31-45`, `:112-117`; `WizardStep.svelte:38-57`;
`SpellBudgetBar.svelte:65-90`; `ValidationPanel.svelte:42`, `:47`, `:50`;
`validation/mod.rs:126-176`; `life_stage.rs:453-494`;
`character_types.json:10-14`, `:44-48`, `:26-34`, `:95-106`;
`AgingRecordPanel.svelte:96-106`; `VirtueFlawTab.svelte:153-165`;
`LifeStagePanel.svelte:126`, `:133`; `derive.ts:925-961`, `:1085`, `:1149-1169`;
Core Rules `:2213-2216`, `:2433-2437`, `:2816-2862`, `:597`). **Zero drift found.**
Two clarifications are recorded in §8.

---

## 2. Decisions — all three answered

D1, D2 and D3 were open when this plan was first written. **All three were answered by
the user on 2026-08-26 and are now settled**; they are recorded here with their
reasoning because the reasoning constrains the implementation. Nothing below is open.
D1's answer also surfaced a **new issue, #32**, which the review doc does not contain.

### D1 — #13: what an unfilled parameterized-ability label reads as → **ANSWERED**

**DECIDED: option (d) below.** The originally-recommended option (a) was **rejected by
the user, correctly** — see the analysis, because it is a trap worth not re-entering.

**The original question.** `rules/i18n/en/abilities.json:23` names
`ability.dead_language` as the template `"{language} (Dead Language)"`. With no
instance held, `paramHint` (`ui/src/lib/derive.ts:57`) fills `{language}` by wrapping
the `param-label-language` string — which is plainly `Language`
(`locales/en/main.ftl:772`) — in `param-hint = ({ $label })` (`en:764`), yielding
`(Language)` and so the doubled *"(Language) (Dead Language) 1 is not met"*. **The
parentheses live in `param-hint`, not in the label key** — worth knowing, because the
fix site is the hint-wrapping decision, not the label's text. The Core Rules state the
requirement plainly as *"Latin 1"* (Core Rules `:2437`). What should the shared
label path emit for an unfilled instance?

| Option | Renders | Cost | Notes |
|---|---|---|---|
| **(a)** Suppress the placeholder when unfilled | `(Dead Language) 1` | Small — one change in the shared `displayName`/`abilityDisplayName` path (`derive.ts:639`), fixes `MagusMinimumAbilities.svelte:49-56` **and** the `issue-magus_minimum_ability` message together | No data change, no new keys |
| **(b)** Keep the hint, drop the template literal | `(Language) 1` | Small | Worse: loses which Ability it is |
| **(c)** Give the requirement an exemplar in the ruleset | `Latin 1` | Larger — new optional field on `life_stages.json` `minimum_abilities` entries + an i18n'd label so no translatable string enters `rules/core/` | Matches the source's own wording (`:2437`) and is authorable from source |

**Why (a) is wrong — do not revisit it.** Suppressing the placeholder breaks every
template where the token is the head of the name or sits mid-phrase, and that is the
overwhelming majority. Verified against `rules/i18n/en/virtues_flaws.json`:

| Template | Under (a) | Today, with the hint |
|---|---|---|
| `Puissant {ability}` | `Puissant` | `Puissant (Ability)` ✓ |
| `Affinity with {ability}` | `Affinity with` | `Affinity with (Ability)` ✓ |
| `Great {characteristic}` | `Great` | `Great (Characteristic)` ✓ |
| `Ways Of The {land}` | `Ways Of The` | `Ways Of The (Land)` ✓ |
| `Necessary {realm} Aura for {ability}` | `Necessary Aura for` | correct today ✓ |

There are ~36 templated V/F names in this class, and German is worse — dangling
inflected adjectives with no noun to agree with. The hint mechanism is **right** for
all of them.

**The bug's real scope is two entries.** Grepping every i18n name template for one
containing **both** a `{token}` and a parenthetical literal — the only shape that
doubles — returns exactly two, both abilities, both locales:

- `ability.dead_language` — `"{language} (Dead Language)"` / `"{language} (Tote Sprache)"`
- `ability.living_language` — `"{language} (Living Language)"` / `"{language} (Lebende Sprache)"`

So #13 is not a defect in the shared label path's default behaviour. It is two
catalogue entries whose literal already carries the generic qualifier the hint also
supplies.

**DECIDED — option (d): an optional explicit unfilled form in the i18n layer.**

```json
"ability.dead_language": { "name": "{language} (Dead Language)", "name_unfilled": "Dead Language", … }
```

`displayName` uses `name_unfilled` when no instance is held; otherwise it falls back to
today's hint substitution, unchanged. Renders `Dead Language 1 is not met` in the
minimums row **and** in the `issue-magus_minimum_ability` message, since both go
through the one path — which is what the review doc's #13 requires ("the fix belongs in
the shared label path, not the component").

Why (d) over the alternatives:

- Translatable text stays in `rules/i18n/<lang>/`, so it never enters `rules/core/` —
  this is what sank the original option (c) (see §8 item 5).
- Per-language, so German gets `Tote Sprache` with correct inflection.
- **Opt-in**: two entries × two locales. The other ~36 templates are untouched, so the
  behaviour (a) would have broken is preserved by construction.
- One `Option<String>` field on the i18n entry struct and one branch in `displayName`.
  No mechanic, no schema bump, no provenance question — it is a label.

**Rejected, and worth naming so it is not proposed as a simplification:** dropping the
parenthetical from those two templates (`"name": "{language}"`). It removes the
doubling, but makes the *filled* form worse — `Latin` no longer says which Ability it
is, and the two entries become indistinguishable in the catalogue list.

### D1 follow-on — #32: the rules require **Latin**, the engine requires any Dead Language

**A new issue, not in the review doc.** Raised by the user while answering D1, verified
against the source, and folded into Slice 7.

**The gap.** Core Rules `:2437` (verified, read in full) says: *"Magi must have the
following minimum Abilities: Parma Magica 1, Magic Theory 1, **Latin** 1. Characters
with lower scores would not be admitted to the Order. A character without a **Latin**
score of least 4 and an Artes Liberales score of at least 1 is unable to read the books
of the Order. … A character with a **Latin** score of less than 5 cannot write books."*
The passage names Latin specifically, three times. The ruleset encodes it as
`ability.dead_language`:

- `rules/core/life_stages.json:4` — `minimum_abilities`, `dead_language` ≥ 1 (rules: Latin 1)
- `rules/core/life_stages.json:10` — `recommended_abilities`, `dead_language` ≥ 4 (rules: Latin 4)
- `rules/core/abilities.json:32` — `scholarly_language`, `dead_language` ≥ 3

So a magus with Ancient Greek 1 and no Latin passes the engine's check but would not be
admitted to the Order. The Latin-4 case is starker: the Order's books *are* in Latin, so
no other dead language substitutes at all.

**Why the substitution exists, and why it is structural rather than lazy.**
`ability.dead_language` declares `parameter: "language"` (`rules/core/abilities.json:55`)
— a free-text instance label, **not** a `ref` into a catalogue. There is no
`language.latin` id for a requirement to point at, and `Latin` / `Latein` is
translatable text, so it cannot be written into `rules/core/` either.

**DECIDED — keep the widened check; fix the honesty. The catalogue option is rejected
permanently, not deferred.**

The tempting fix is to give `language` a real catalogue (`language.latin`,
`language.greek`, …) so the requirement can name Latin by id. **The user rejected this
as non-viable at any later stage, and the reasoning is decisive: the rules contain no
comprehensive list of languages, and whether a given language exists — and whether it
is dead or living — is a troupe's decision.** Authoring such a catalogue would mean
inventing rules data, which the provenance rule in `CLAUDE.md` prohibits outright. So
**free text is the correct model for the `language` parameter, not a shortcoming**, and
this must be recorded in `RULES.md` so a future session does not "improve" it into a
catalogue. It also follows that the requirement can never be narrowed to Latin
mechanically: matching a user-typed string against a localized name would be
locale-dependent and would break on `latin`, `Lateinisch`, or any troupe's own spelling.

**Implementation shape.** The enforced check stays "any Dead Language ≥ N". The rules'
own exemplar is surfaced as a label, and — because three separate rules-data sites mean
Latin — it belongs in the rules layer rather than hardcoded into a Fluent message:

- `rules/core/` gains a **language-neutral slug** on the requirement, e.g.
  `"exemplar": "latin"`. This is one named example the rules themselves name at
  `:2437`; it is **not** an enumeration of languages, and it introduces no catalogue.
- `rules/i18n/<lang>/` maps that slug to `Latin` / `Latein`, keeping translatable text
  out of the mechanics file.
- The minimums row and the validation message name the exemplar alongside the
  requirement, so the player sees what the rules actually demand while the engine keeps
  enforcing what it can defensibly enforce.
- `RULES.md` records: the verified `:2437` citation, that the check is a **deliberate
  widening** of "Latin 1", that the widening exists because languages are troupe-defined
  free text, and that it must not be narrowed.

Referential integrity: the exemplar slug is a label key, not a `ref`, so it is exempt
from the `has`/`incompatible_with` resolution rules — state that explicitly where the
loader's integrity check is documented, or the next audit will flag it.

### #33 — a grog cannot set Personality Traits in the wizard, and the rules say it most needs to — **NEEDS A DECISION**

**Surfaced by Slice 3, verified from source, not in the review doc.** Slice 3's
tab-mirrors-phases test exposed it: the editor gives **every** type a Personality &
Reputations tab, but the **grog** profile does not declare the
`personality_reputations` phase, so a grog built in the guided wizard never reaches
that surface at all.

**The rules say grogs need it more than anyone.** Core Rules
`Ars Magica - Definitive Edition (Core Rules).md:1073-1075` (verified, read in full):

> *"For major characters, such as magi and companions, they are normally nothing more
> than an aide memoire … For grogs, they are more significant. As grogs are often
> shared between players, or at least played rarely, the numbers attached to
> Personality Traits can be used as a concrete guide to playing the character. …
> 'Loyal' is a particularly important Trait, as it reflects the grog's attachment to
> the covenant, while 'Brave' is just as important for warrior grogs. A third Trait
> should be something distinctive about that grog."*

So the omission is backwards: the one type whose Personality Traits the rules treat as
mechanically load-bearing is the one type whose guided flow cannot record them. The
`:1165` character-sheet listing also names Personality Traits unconditionally.

**Why this is a decision and not a bug fix.** Adding `"personality_reputations"` to
grog's `creation_phases` is a one-line data change — the phase, its component, its
guidance key, its Fluent strings and the e2e walk's filler all already exist, so the
ripple is small (the grog rail gains a step; `grog-wizard.e2e.js`'s rail assertion
updates). But it **lengthens the grog flow**, which is a product call the user should
make rather than one to slip in under a slice about editor tabs. It is also not one of
the 31 reviewed issues.

**Recommendation: add it.** The rules are unambiguous, the machinery exists, and the
alternative — leaving the editor tab as the only way to set a grog's traits — means the
guided flow silently omits the thing the source calls out for that type.

**Interim state, shipped in Slice 3:** the Personality & Reputations tab is **ungated**,
so a grog *can* set its traits in the editor. Slice 3's mirror test therefore tolerates
a tab **superset** of the phase list rather than demanding an exact match — deliberately,
and documented in the test, so this gap does not masquerade as a passing invariant.

### D2 — #18: how the spell-levels pair is made to close → **ANSWERED: option (a)**

`SpellBudgetBar.svelte:68-90` shows `baseUsed / base`, verified as
`derive.ts:940-961`: `baseUsed = used - bonusUsed + penalty` (includes
post-Gauntlet-funded levels) while `base = budget - bonus - lifeStage` (excludes
them). At `base=120, lifeStage=30, used=150` the readout is `150 / [120]` with
`Available: 0`.

- **(a)** Denominator becomes `base + lifeStage` → displays `150 / 150`; the
  bracketed input still edits `spell_levels_override` (the base) alone. Requires the
  input's displayed value and the denominator to be visibly different things, or the
  input moves out of the denominator slot.
- **(b)** Charge post-Gauntlet levels to their own used/amount chip, alongside the
  existing bonus chip, so `baseUsed` drops to 120 and `120 / [120]` closes.

**DECIDED — option (a).** The denominator becomes `base + lifeStage`, so the readout is
`150 / 150`; the bracketed input continues to edit only `spell_levels_override` (the
base). Rationale: less disruptive, and it keeps one "levels spent" figure rather than
splitting the spend across two chips the way the V/F bonus already is. (b) would be more
internally consistent with the bonus chip but changes the meaning of the primary number.

**Consequences for the implementation, both load-bearing:**

1. The bracketed input's *value* (the base, 120) and the denominator beside it
   (`base + lifeStage`, 150) are now **visibly different numbers**. The input therefore
   cannot remain the denominator slot — either it moves out of that position, or the
   denominator is rendered separately from it. Decide this in Slice 8 and state which in
   the commit message; a layout in which `[120]` sits where `150` is expected would
   reintroduce #18 in a new form.
2. **This settles #19's shape too.** In the wizard the base is read-only (#19's
   DECIDED), so the slot renders as plain text; in the editor it renders as the input.
   The `readonlyBase` prop is designed against option (a) — the wizard shows
   `150 / 150` with no editable field anywhere in the pair.

### D3 — #25's open sub-questions → **ANSWERED, and reshaped by a future feature**

#25's *core* was already DECIDED (an app-level current-year setting, default **1220**,
Core Rules `:597` / `:364` / `:440`; age ↔ birth year as two views of one fact on
Concept; no schema bump because both values are already stored).

**The decisive new constraint.** The user stated the intent to build **saga management**
later, in which the current year *progresses*. That changes two of the three answers, so
the reasoning is recorded here rather than just the conclusions — the shape of this
field is what makes the future feature cheap or expensive.

1. **Persistence → PERSIST IT, as saga state.** *(This reverses the original
   recommendation of a session-scoped value.)* A value that resets to 1220 on every
   launch would be actively wrong once the year advances. It is a **saga year**, not a
   UI preference: name it that, and store it in a small app-settings file **owned by
   `arm-app`**. Engine purity is absolute here — the IO lives in `arm-app`, never in
   `arm-rules`. Do **not** put it on the entity: it is saga-scoped, shared across the
   characters in a saga, so an entity field would duplicate it per character and
   immediately disagree with itself. And do not put it in the ruleset: it is a saga
   fact, not a rule.
2. **Underflow guard → clamp to 0 and emit an advisory finding.** *(Unchanged, but now
   load-bearing rather than defensive.)* `birth_year` is `i32` and `age` is `u32`, so
   `current_year - birth_year` can underflow. With an advancing saga year, characters
   created at different points make impossible pairs **more** likely, not less. A new
   finding code means a new `issue-<code>` in both locales and a `RULES.md` entry.
3. **Linkage → the saga year must NOT rewrite stored age.** *(This is the answer that
   changed most, and the most important one to get right.)* Three values exist and only
   two can be authoritative. The rule:
   - `age` and `birth_year` are **the stored pair**; the saga year is a **reference for
     derivation only**.
   - Editing **age** recomputes `birth_year`. Editing **birth_year** recomputes `age`.
     Both are continuously linked while on screen, per the DECIDED "two views of one
     fact" wording. Both dirty the document — they are entity edits.
   - Editing the **saga year** recomputes **nothing**. Both stored values stay as they
     are; the new year only governs what gets derived the next time the user types in
     age or birth year. It does **not** dirty the document.

   **Why the saga year must not silently advance ages:** #25's DECIDED text already
   settled this — *"a character built in a 1220 saga and opened under a 1230 setting
   keeps both stored values"*. Saga *progression*, where everyone ages, is a separate
   **explicit** action, and it cannot be a side effect of editing a setting because
   advancing a character by N years requires aging rolls, Living Conditions and any
   Longevity Ritual applied per year — that is the aging-for-loaded-characters backlog
   item, not a subtraction. A silent recompute would fabricate ages that skipped their
   aging rolls, which is exactly the bug class the aging engine exists to prevent.

   So saga management does not invalidate #25; it means the field should be **shaped as
   saga state from the start** rather than retrofitted from a UI preference.

### Answered by default (no user input required)

- **#15** — the review doc names option (a) as the default unless told otherwise.
  This plan takes (a) and puts #15 out of scope (§3). If the user later wants (b) — a
  ruleset-level editable `apprenticeship.years`, honestly labelled a house rule — it
  is a small data + UI slice, but it is *not* in this plan.
- **#30** — the review doc flags one possible preference ("Review-only warnings").
  This plan implements phase-owned warnings, which surface on both the owning step
  and Review; if the user wants Review-only, say so before Slice 11.

---

## 3. Not in scope

| Item | Why |
|---|---|
| **#15 — editable apprenticeship duration** | **Source-blocked.** The Core Rules price apprenticeship as a flat lump — *"The fifteen years of apprenticeship give the character 240 experience points, and 120 levels of spells"* (`Ars Magica - Definitive Edition (Core Rules).md:2435`, verified) — and there is **no** XP-per-apprenticeship-year figure anywhere in `rules/source/en/`. `:11018` acknowledges duration variation without pricing it. A per-character duration with scaled XP requires inventing 240 ÷ 15 = 16 XP/yr, which the rules-provenance rule in `CLAUDE.md` prohibits. **What would unblock it:** a passage in an English source book under `rules/source/en/` that states an experience rate per year of apprenticeship (or an explicit XP total for a non-15-year apprenticeship). Until such a source is added, the value stays the ruleset constant at `rules/core/life_stages.json:16`, read at `crates/arm-rules/src/life_stage.rs:456`. The only in-plan concession: Slice 8 may make it *visible* that 15 is a ruleset constant rather than an oversight (a hint string, no mechanic, no new rule). |
| **Backlog — aging for loaded characters** | Explicitly out of the review's fix list; "details to be discussed". #25 (Slice 12) is its groundwork — the current-year field it will need — so nothing here forecloses it. |
| **Non-issue: the 10 blank aging-log rows** | Verified deliberate: user-added via `AgingRecordPanel.svelte:132`, not generated. Slice 6 bounds the log's *height* (#20's DECIDED full-width row with its own scrollport); it does not change row creation. |
| **Non-issue: native language in the life-stage panel** | Load-bearing, not decoration: the engine needs the name to instantiate the right parameterized `ability.living_language`, to enforce the "other than the character's native language" clause (`childhood.rs:997`) and because every package lists `Native Language 5`. Its *visual* home is folded into #11 (Slice 2), which moves it to the new `experience` step; nothing is removed. |
| **Non-issue: wizard/editor component sharing** | Verified. It is the premise the plan builds on, not a defect. #19 introduces the first deliberate behavioral divergence, via an explicit prop, and updates the comment that states the reuse rule (`WizardStep.svelte:38-43`). |
| **#19's noted-but-out-of-scope sibling** | The XP pool's editable total on the Abilities step is the same shape of field as the spell-levels base. The review doc leaves it as-is and merely notes the inconsistency; this plan does the same. Do not "fix" it opportunistically in Slice 8. |
| **A `language.*` catalogue (for #32) — REJECTED PERMANENTLY, not deferred** | The obvious way to make "Latin 1" enforceable is a language catalogue so the requirement can name Latin by id. **It cannot be built.** The rules publish no comprehensive list of languages, and whether a given language exists — and whether it is dead or living — is a **troupe's decision**. Authoring such a catalogue means inventing rules data, which the provenance rule prohibits outright. So free text is the *correct* model for the `language` parameter, not a shortcoming, and the "any Dead Language ≥ N" check can never be narrowed to Latin mechanically (string-matching a user-typed value against a localized name would be locale-dependent and break on `latin`, `Lateinisch`, or a troupe's own spelling). **Record this in `RULES.md`** so a future session does not propose it as an improvement. Slice 7 instead surfaces the rules' own exemplar as a label. |
| **Saga progression (advancing characters year by year)** | D3 established the saga year as persisted state, which is the groundwork — but *advancing* it must not silently recompute ages, because adding N years to a character requires aging rolls, Living Conditions and any Longevity Ritual applied per year. That is the aging-for-loaded-characters backlog item, and it is a feature, not a subtraction. Slice 12 deliberately makes the saga year inert with respect to stored values. |

---

## 4. How to execute this plan

### 4a. Agentic-implementation protocol

Implementation is **delegated to subagents** — one per slice, or one per coherent
sub-step of a large slice (Slice 2 and Slice 4 will each want two or three). The
session that runs this plan is the **orchestrator** and does the following, in order,
for every slice:

1. **Delegate the TDD work.** Launch an implementation subagent via the Agent tool.
   The prompt must contain:
   - the slice's **full text** from §5 (issues covered, files, anchors, TDD steps,
     i18n keys, acceptance criteria) — do not paraphrase it;
   - the relevant `file:line` anchors, spelled as absolute paths;
   - the explicit instruction that **the failing test comes first and must be *seen*
     failing** — run the test, paste the RED output, only then write implementation.
     TDD in this repo is strictly test-first, including for pure helpers;
   - the vitest project the frontend test belongs to (`ssr` vs `client`) and why;
   - the **LIVE command-hygiene block** (see 4c), pasted verbatim;
   - the standing rules: never write files through the shell (no `>`, `>>`, `tee`,
     `sed -i`, heredocs) — **Edit/Write only**; read files with **Read/Grep/Glob**,
     never `cat`/`head`/`tail`/`sed -n` pointed at a path; all scratch and log
     artifacts under the repo-local **`tmp/`**, never `/tmp`; leave changes in the
     working tree and **never** run `git add`/`git commit` — committing is the
     orchestrator's call.
2. **Run the gate yourself, first-hand.** After the subagent reports, the
   orchestrator runs the full gate (4b) directly and reads the output. **A subagent's
   claim that the gate passed is never sufficient** — another session may share this
   repo, so only output the orchestrator observed counts.
3. **Do the manual verification note.** Several of these are layout-shift bugs no
   unit test catches. Launch the app (`./arm-char-gen.sh`, or `cargo tauri dev`) and
   perform the slice's stated click-path before calling the slice done.
4. **Commit per slice, directly on `main`.** This repo's workflow commits to `main`;
   no branch and no PR unless the user asks. Message: what changed and which issue
   numbers it closes, ending with the trailer
   `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`.
5. **Keep the docs current in the same slice.** `crates/arm-rules/RULES.md` for any
   new or changed mechanic (verbatim excerpt → source `<book basename>.md:<lines>` →
   implementing function/file, plus the JSON value where a rule number lives).
   `README.md` whenever a change alters what the project does, its status, the stack,
   or the build/run commands.
6. **Never batch two slices into one commit.** The dependency chain is the point; a
   combined commit makes a bisect useless if a layout change turns out wrong.

### 4b. The standard gate

Every command below must pass before any commit or "done" claim.

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cd ui && npm run test:unit && npm run lint && npm run format:check && cd ..
cargo tauri build --no-bundle
```

**The last command is non-negotiable and authoritative.** It is the only step that
runs `svelte-check` (via `beforeBuildCommand`) and builds the production app, so
frontend type errors and production-only breakage slip past every other gate.
`cargo test`/`clippy` do not type-check the frontend, and `npm run test:unit`
(vitest) does not either.

**E2E is required for slices that change app runtime behavior, save/load, or the
window/app lifecycle:**

```bash
cd /home/norbert/Rolle/arm-char-gen/ui && npm run test:e2e
```

**The operative rule is broader than "runtime behavior":** run e2e for **any slice
whose diff touches a component, testid, DOM structure or `.ftl` string that an
existing spec references.** Enumerating slices is a convenience, not the rule — check
`ui/e2e/` before deciding a slice is exempt. Deferring a spec breakage to the final
commit destroys the per-slice bisectability that §4a.6 exists to protect.

Mandatory for:

- **Slice 2** — the phase list is read by `ui/e2e/wizard-walk.js`, which derives phases
  from `rules/core/character_types.json` and fails loudly if a phase has no filler
  (`FILLERS` at `wizard-walk.js:139`, dispatch at `:241-245`).
- **Slice 3** — tab ids and `aria-controls`; the specs that select a tab by name/id.
- **Slice 4** — schema + migration + save/load.
- **Slice 5** — #31: a new entry point, persisted state, and a dirty-flag interaction
  with the mandatory unsaved-changes guard.
- **Slice 6** — restructures the aging DOM (grid, full-width log, `decrepitude_effect`
  becoming a `<textarea>`) that `aging.e2e.js`, `aging-crisis.e2e.js` and
  `longevity-ritual.e2e.js` select against. The control-type change alone can break a
  spec that sets a value.
- **Slice 8** — renames budget-bar chips and `.ftl` strings that
  `magus-apprenticeship.e2e.js`, `magus-post-gauntlet.e2e.js` and `i18n-german.e2e.js`
  plausibly assert on; the sticky-bar change also alters scroll behaviour the specs
  drive.
- **Slice 10** — regroups the V/F list that `repeat-virtues.e2e.js` and
  `vf-effects.e2e.js` walk, and changes the validation row markup.
- **Slice 7** — *conditionally*: a text input becomes a `<select>`, so run e2e if any
  spec fills a `form`/`technique`-domain parameter. **Grep `ui/e2e/` first** and record
  the answer in the commit message rather than assuming.
- **Slice 12** — **now mandatory.** D3.1 landed persistence, so this slice adds an
  `arm-app` command and a settings file read at launch: a new IPC surface and a new
  startup path, neither of which any unit test exercises end to end. Also run
  `npm run test:e2e:portable` here — the settings path must resolve in the portable
  layout too, which is the one place `BaseDirectory::Resource` does not point where you
  expect (see `project_portable_resource_resolution` and `pick_rules_dir`).

Run `npm run test:e2e:portable` for any slice that touches resource or path resolution
(`crates/arm-app/src/commands.rs`, `pick_rules_dir`). **Slice 12 does** — D3.1's
saga-year settings file is a new path resolved at launch, and the portable layout is
exactly where `BaseDirectory::Resource` points somewhere that does not exist. No other
slice is expected to; if one does, add it to that slice's gate.

**E2E is runnable on this machine and must never be skipped for want of a display.**
With `DISPLAY` unset, `@wdio/local-runner` wraps the worker in `xvfb-run` and the run
is headless; with `DISPLAY` set it runs headful. Same command either way. **Never**
add an `xvfb-run` wrapper or an env prelude of your own. Logs land in
`tmp/e2e-logs/` via wdio's `outputDir` — read them with Read rather than re-running.

### 4c. The command-hygiene block to paste into every subagent prompt

Background agents cannot raise an interactive approval prompt: a command with a
non-allowlisted stage is **auto-DENIED**, not queued. So these rules must be inline
in every subagent prompt. **Re-read `.claude/skills/full-review/SKILL.md` lines
17-203 before pasting, in case it has changed** — the text below was copied verbatim
from that file at commit `721224a`. Substitute `<repo>` with
`/home/norbert/Rolle/arm-char-gen`.

```
### Command hygiene (ALL agents — required, paste into every agent prompt)

The project's `.claude/settings.local.json` pre-approves a set of command *prefixes* and
pre-sets `PATH` (cargo + node/npm are already on it). Background agents cannot raise
interactive approval prompts, so any command containing a stage that does not match an
allowlisted prefix is **auto-DENIED** (not queued for approval).

**Pipes and chains are fine** — the checker splits a command on `|`, `&&`, and `;` and
requires **every stage to match an allowlisted prefix**. So compose freely with pipelines.

**But matching a prefix is necessary, not sufficient.** Understand this before you start,
because misreading it is what sends agents hunting for workarounds. An allowlisted prefix
only clears the *first* of several independent checks; any later one can still deny the
command:

1. **Prefix match** — the stage must begin with an allowlisted prefix. `git -C <path> diff`
   fails, because it does not begin with `git diff`.
2. **Per-command flag whitelist** (`safeFlags`) — every known command has a table of
   permitted flags, git subcommands included. An unlisted flag is denied.
3. **Argument path containment** — commands classed as file *reads* (`grep`, `rg`, `cat`,
   `head`, `tail`, `od`, `jq`, `git`, …) have their positional file arguments extracted and
   checked against the working directory. Anything outside it is denied with *"Path is
   outside allowed working directories"* — `/dev/null` included.
4. **Special guards**, each with its own reason code: `cd-git-compound` (a `cd` and a `git`
   in one command), `multi-cd`, `cd-compound-redirect`, `shell-expansion`,
   `process-substitution`, `sed-dangerous`, `shell-operators`, `too-complex`.

So a command that looks entirely legitimate can still be denied, and that is **not**
evidence that the permission system is broken or that you need a cleverer spelling. Work
out which layer refused it and satisfy all four — or use the native Read/Grep/Glob tools,
which bypass this machinery completely. To stay inside it, every agent MUST:

- **A denial is information, not an obstacle to route around.** This is the governing
  rule; the specific bans below are only its worked examples. When a command is denied,
  do NOT construct a different spelling of the same thing — reaching for `perl` because
  `grep` returned nothing, or `git -C <path>` because a `cd` was refused, turns one
  blocked command into a second blocked command and burns the run. Stop and pick one of:
  (a) the native **Read/Grep/Glob** tools, which need no approval inside the repo and are
  usually the better answer anyway; (b) a genuinely different, allowlisted approach; or
  (c) report the blocker in your findings and move on. A refusal often means your premise
  is wrong — the `perl` incident began with a `grep` that was silently skipping a corrupt
  file, and no amount of tool-swapping would have revealed that.
- **Every stage must be an allowlisted command.** Allowed filter/util tools you can pipe
  through: `jq`, `grep`, `rg`, `sed`, `awk`, `sort`, `uniq`, `cut`, `tr`, `wc`, `head`,
  `tail`, `cat`, `diff`, `cmp`, `od`, `xxd`, `file`, `find`, `ls`, `echo`, `mkdir`.
  Build tools: `cargo …` (any
  subcommand), `npm run <script>`, `npx prettier/eslint/svelte-check/vitest/wdio` — `npx
  eslint <file>` is the way to lint a single file, since `npm run lint` is whole-tree.
  Read-only
  git: `git status`, `git diff`, `git log`, `git show`, `git check-ignore` — use these
  to inspect the working diff. A pipeline like `cargo tarpaulin … | tail -1` or
  `jq '.files | length' tmp/tarpaulin-report.json` auto-approves because each stage is
  allowlisted.
- **Never mutate git state.** `git push`, `checkout`, `reset`, `rebase`, `stash`, and
  `clean` are NOT allowlisted and are auto-denied, deliberately — they move refs or
  discard work. `git add`/`git commit` are allowlisted for the main session but a
  reviewer or fixer agent must NOT use them: committing is the orchestrator's call
  after the gates pass, never a subagent's. Leave your changes in the working tree.
- **An empty `grep` result does NOT mean "not present".** `grep` here is not GNU grep:
  Claude Code injects a shell function that re-execs its bundled **ugrep** as
  `grep -G --ignore-files --hidden -I …`. **`-I` is hardcoded**, so a file containing a
  NUL byte is skipped *silently* — no match, no warning, exit 0. If a search for
  something you are confident exists comes back empty, re-run it with **`grep -a`**
  (overrides `-I`) before concluding the symbol is missing or that your tools are
  broken. Also, ugrep's regex dialect is not GNU's — a bracket expression with hex
  ranges may fail with `ugrep: error at position N`; fall back to `rg -a` (NOT
  `command grep` — only `command -v` is allowlisted, so that is denied). The native
  Grep tool is ripgrep and skips binary files too.
- **Inspect bytes with `od`/`xxd`/`cat -v`/`cmp`/`file` — all allowlisted.** If a file
  looks corrupt or a search behaves impossibly, that is the toolkit. Escalating to an
  interpreter to hunt control characters is never justified and will be denied.
- **Parse JSON with `jq`, never an interpreter.** Do NOT shell out through `python`,
  `python3`, `perl`, `ruby`, `node -e`, `bash -c`, or `sh -c` — an interpreter is an
  arbitrary-code escape hatch that defeats the allowlist, and it is not allowlisted anyway.
  For the tarpaulin coverage %, read it directly: `cargo tarpaulin` prints a
  `XX.XX% coverage, N/M lines covered` summary line to stdout during the same run that
  writes the JSON — capture it (e.g. `… | tail -3`) rather than post-processing the
  1.5 MB report. If you truly need a field from the JSON, use `jq`.
- **Avoid arbitrary command executors when a direct form exists.** `xargs`, `find -exec`,
  `bash -c`, `sh -c`, and interpreter `-c`/`-e` flags run a command the allowlist cannot
  vet, so they are intentionally NOT allowlisted and will prompt/deny. Almost always there
  is a plain equivalent: prefer a shell glob or the tool's own file arguments over piping
  into an executor — e.g. `wc -l crates/arm-rules/src/*.rs` instead of
  `find … | xargs wc -l`, and `grep -r PATTERN <dir>` instead of `find … -exec grep …`.
  Only reach for an executor if there is genuinely no allowlisted alternative.
- **Avoid `$`-expansion syntax in commands.** The permission analyzer flags any `$(…)`,
  backticks, `$'…'` (ANSI-C quoting), and even `$VAR`/`$0`/`$1` field refs as unverifiable
  and prompts **regardless of the allowlist** — you cannot allowlist past them. Use plain
  forms: rely on `sort`'s default whitespace (space/tab) field-splitting, e.g.
  `… | sort -k2 -nr`, instead of `sort -t$'\t' …`. **A regex end-of-line anchor `$`
  counts as well** — the analyzer cannot distinguish an anchor from an expansion, so an
  otherwise fully allowlisted `grep -n "^## Heading$" file.md` is denied. Drop the anchor
  and tolerate the extra hits (`grep -n "^## Heading"`), or use the native Grep tool,
  which takes a real regex and needs no approval.
- **Scan code with `grep`/`rg`, not `awk`/`sed`.** `awk`/`sed` programs are built around
  `$1`/`$0`/`$` field references, which trip the `$`-expansion guard above and prompt even
  though `awk`/`sed` are allowlisted. To find items, use `grep`/`rg` with an extended
  regex — e.g. `grep -rnE '^\s*pub (fn|struct|enum|const|mod|trait)' src/` — or the native
  Grep tool. Reserve `awk`/`sed` for the rare transform with no `$` in it.
- **No shell control-flow — `for`, `while`, `if`, `case`.** Loops and conditionals cannot
  be decomposed into allowlisted prefixes, so the analyzer always prompts on them (and they
  usually carry `$var` too). Iterate with a glob or the tool's own multi-file arguments
  (`grep -nE … src/*.rs`, `cargo test -p arm-rules`), or use the native Read/Grep tools —
  never a `for f in …; do …; done` loop.
- **No brace groups or subshells — `{ … ; }` and `( … )`.** Compound commands built with
  `|`, `&&`, `||`, and `;` ARE allowed: the analyzer splits on those separators and vets
  each stage against the allowlist, so `cargo tarpaulin … | tail -3` and
  `cd <repo>/ui && npm run check` auto-approve. Grouping constructs do not decompose that
  way — the first token the analyzer sees is a bare `{` or `(`, which matches no
  allowlisted prefix, so the whole line is denied. Write the stages out flat with `&&`/`;`
  instead of wrapping them, and never use `{ … } > file` (redirects are banned anyway —
  use the Write tool). Brace *expansion* in an argument (`src/*.{rs,toml}`) is likewise
  out: use a plain glob or repeat the argument.
- **Never prefix a repo-root command with `cd <repo> &&`.** You already start in
  the repository root (`<repo>`), so that `cd` is dead weight — and it makes the
  analyzer prompt even though both stages are allowlisted, because relocating the shell
  changes what the next stage operates on. Observed repeatedly: `git log --oneline …`
  and `git diff … -- crates/…` run silently on their own, but
  `cd <repo> && git diff …` raises a dialog every time. Just
  issue the command with repo-relative paths (`git diff b49d49b..HEAD -- crates/…`,
  `sed -n '1,40p' crates/arm-rules/src/derived.rs`). The **only** place a `cd` is
  warranted is the UI gate, which genuinely must run inside `ui/` — see below.
  The precise rule (`bashMissKind: "cd-git-compound"`): a compound holding **both a `cd`
  and a `git`** is denied *regardless* of the allowlist and *regardless* of the path — it
  is not a containment check, so pointing at the repo you are already in does not help.
  Rationale: after a `cd`, git resolves its repository from the new directory, whose
  hooks are arbitrary executables. Two neighbours: **multiple `cd`s** in one command are
  denied (`multi-cd`), and a **`cd` plus an output redirect** is denied
  (`cd-compound-redirect`). A `cd` with a **non-git** command is unaffected, which is why
  the UI gate auto-approves.
  **`git -C <path> …` is NOT the way around this.** The allowlist grants the prefixes
  `git diff`, `git log`, `git show`, … and `git -C <path> diff` does not start with
  `git diff`, so no rule matches and it is denied just the same. There is no workaround
  to find here, because none is needed: you start in the repo root, so write the command
  bare — `git diff --stat -- crates/`, `git log --oneline b49d49b..HEAD`.
- **Every file argument must be inside the repo — including `/dev/null`.** File-reading
  commands (`grep`, `rg`, `cat`, `head`, `tail`, `od`, `jq`, `git`, …) have their
  positional file arguments extracted and containment-checked against the working
  directory; a path outside it is refused with *"Path is outside allowed working
  directories"* however well allowlisted the command is. `/dev/null` is exempt only as a
  redirect target, never as a file to read — `grep -n "PLAN.md" -m2 /dev/null` is denied.
  Never pass a placeholder file argument; just omit it.
- **Write artifacts with the Write tool, not shell redirects.** Redirects (`>`, `>>`,
  `tee`) are not allowlisted. To create `tmp/review-findings.json` or any file, use the
  Write/Edit tools (they work in-repo and under `tmp/` without approval). This includes
  parking a large diff in the scratchpad: `git diff … > …/derived.diff` is denied — pipe
  it through `head`/`grep`, or read the file with the Read tool instead.
- **Keep the output of an expensive run — via the tool's own log option, in `tmp/`.**
  Never throw away a full e2e or release-build log and re-run the suite to recover a
  line. Redirects are denied for you, so do not reach for `>`; use the option the tool
  already provides. For e2e that is wdio's `outputDir`, wired to `tmp/e2e-logs/` in
  `ui/e2e/wdio.conf.js` — a plain `cd <repo>/ui && npm run test:e2e` leaves the
  complete launcher/worker logs there, which you then inspect with the **Read** tool.
  If a tool offers no log option and the output is too long to keep in context, trim it
  in the same pipeline (`… | tail -40`) rather than re-running later.
- **All scratch output goes under the repo-local `tmp/`, never the system `/tmp`.**
  `tmp/` is gitignored and is where coverage reports, findings JSON, and staged builds
  belong. A `/tmp/...` path fails the argument-containment check, so every subsequent
  read of it is denied — you would be creating a file you cannot then look at. (The e2e
  specs' own `os.tmpdir()` fixture is a deliberate exception in committed code; do not
  imitate it for your own artifacts.)
- **To read a file, use the Read tool — not `cat`, `head`, `tail`, or `sed -n`.** Those
  are allowlisted as *pipeline filters for a command's stdout* (`cargo tarpaulin … |
  tail -3`), and that is the only thing they are for here. Pointing them at a path is a
  denial risk for no benefit: `Read` (with `offset`/`limit` for a slice) and
  `Grep`/`Glob` need no approval anywhere in the repo or under `tmp/`, return line
  numbers you can cite in a finding, and keep the harness's file-state tracking intact
  so a later Edit cannot silently clobber. `cat <file> | tail -50` is the specific
  anti-pattern: it burns a Bash call and throws away the 90% of the file you will ask
  for next.
- **Never prepend environment setup.** Do NOT add `source ~/.cargo/env`, `export PATH=…`,
  or `nvm use` — `PATH` is already configured via the settings `env`, so `cargo`, `npm`,
  and `node` resolve directly. (This supersedes any "source cargo/nvm first" note in older
  run recipes.)
- **Use the allowlisted build-command forms:** `cargo test/clippy/fmt/tarpaulin/tauri …`
  and `npm run <script>`. NEVER bare `npm`/`node`, `npm install`, `npm ci`, or
  **`npm --prefix <dir> run …`** — none of those match `npm run:*`, so they are denied.
- **Run the UI gate as one compound call:** `cd <repo>/ui && npm run <script>` — e.g.
  `cd <repo>/ui && npm run check`, where `<repo>` is the repository root spelled out
  as an absolute path. Both stages are allowlisted
  (`cd:*` + `npm run:*`) so the whole line auto-approves, and using the absolute path makes
  it independent of the current working directory. Do NOT rely on a bare `cd ui` from an
  earlier call persisting, and do NOT use `npm --prefix ui run …` (not allowlisted). Run
  `npm run check`, `npm run test:unit`, `npm run lint`, `npm run format:check` this way.
- File reads inside the repo and under the session scratchpad/`tmp/` need no approval —
  use the **Read** tool for them (see the `cat`/`head`/`tail` rule above); do not read
  unrelated out-of-repo paths.
```

### 4d. Standing rules every slice inherits

- **i18n.** Every new or changed user-facing string needs a key in **both**
  `locales/en/main.ftl` and `locales/de/main.ftl` — `ui/src/lib/i18n.test.ts:68`
  enforces full key parity and will fail loudly otherwise. German rules terminology
  must come from `rules/source/de/translation-tables/` first; if the term is not
  there, take it from the German rulebook at the **mirrored line number** of the
  English source (the German files mirror English line-by-line) and record which
  source supplied it. **Never invent a German rules term.** A raw ID, slug or enum
  value must never be rendered as a label — always map through a Fluent key
  (`phase-<id>`, `category-<id>`, `param-label-<key>`, `issue-<code>`).
- **Negative signs** are the ASCII hyphen `-` (U+002D) via `formatSigned` in
  `ui/src/lib/derive.ts` — never U+2212.
- **Provenance.** Any new or changed mechanic gets a source comment at the
  implementation site citing `<book basename>.md:<inclusive line range>`, with the
  range **verified against the actual file before committing**, plus a matching
  `crates/arm-rules/RULES.md` entry in the same slice. Never cite a bare line number.
- **Architecture invariants** (from `CLAUDE.md`): engine purity (`arm-rules` has no
  `tauri`, no filesystem, no UI); catalogue size is data, never code — no test may
  assert a catalogue total; one evaluation path (the engine always computes
  validation; `ValidationMode` governs enforcement at the caller); entity-generic
  design; two-file `rules/core/` + `rules/i18n/<lang>/` separation joined by stable
  slug IDs; canonical serialization (`BTreeMap`, sort by `id`/`ref`); saves store
  choices, not resolved values.
- **Shared-component blast radius.** Nine of the 31 issues (and #32, which is shared
  *rules data*, so it affects both surfaces too) live in components the
  wizard shares with direct-entry edit mode and are therefore **edit-mode bugs too**:
  **#3, #4, #5, #6, #8, #9, #13, #17, #26**. Only #1, #2, #7, #10, #14, #16, #19,
  #24, #31 are wizard-specific. Any slice touching a shared component must plan test
  coverage for **both** surfaces — the wizard mount (`WizardStep.svelte`) *and* the
  editor mount (`App.svelte` / `CharacterDetails.svelte`) — and say so in its
  acceptance criteria.
- **The unsaved-changes guard is a mandatory product behavior.** `AppStore.dirty`
  (`ui/src/lib/state.svelte.ts`), the `update_close_guard` IPC mirror (an `$effect`,
  hence `*.client.test.ts`), and the Rust `on_window_event` /
  `RunEvent::ExitRequested` handlers in `crates/arm-app/src/main.rs` must keep
  working on every quit path including macOS Cmd+Q. The dirty-tracking tests in
  `ui/src/lib/state.svelte.test.ts` and the `window-close-bridge-clean` /
  `window-close-bridge-dirty` e2e specs must stay green in every slice, and Slice 5
  (#31) must extend them.
- **Frontend test project choice.** Default to **`ssr`** (`*.test.ts`, node env,
  `render` from `svelte/server`). Use **`client`** (`*.client.test.ts`, happy-dom,
  `mount` + `flushSync` + `unmount`) **only** when the thing under test cannot be
  observed without a live component instance: an `$effect` body running, lifecycle
  hooks, focus management, event listeners, `bind:` two-way updates, or an assertion
  on live DOM state rather than rendered markup. SSR never executes an `$effect`, and
  an assertion placed after an effect that never fires still reports green — so
  guessing wrong here produces a silently useless test.

---

## 5. Slices

Dependency graph (→ = "must land before"):

```
S1 (#23,#3) → S2 (#1,#11) → S3 (#28) → S4 (#29 + schema 16) → S5 (#31)
     ↓             ↓            ↓
    S6 (#20,#21,#22,#26)      S12 (#24,#25)
     
S7 (#4,#5,#13)   — independent of S1-S6, sequenced after S6
S8 (#14,#18,#16,#19)  — after S2
S9 (#2,#17,#27)  — after S1
S10 (#6,#8,#9,#10)    — after S1
S11 (#7,#12,#30)      — after S2
```

**The diagram is a sketch; this table is authoritative.** Each slice's own "Depends on"
line is the contract.

| Slice | Issues | Depends on | Notes |
|---|---|---|---|
| S1 | #23, #3 | — | Must be first: changes every block's height. |
| S2 | #1, #11 | S1 | **Critical path.** Closed enum + 4 data profiles + locales + e2e fillers + validation ownership map. |
| S3 | #28 | S2 | Also lands the editor's Experience tab that S2's temporary mount stands in for. |
| S4 | #29 + schema 16 | S2 | **Critical path.** Engine + schema + migration + an `Option`-check sweep the compiler will not find. |
| S5 | #31 | S4, S2 | |
| S6 | #20, #21, #22, #26 | S1, S3 | S1 for the `minmax` floor; S3 for the Aging tab. |
| S7 | #4, #5, #13, **#32** | — | Structurally independent; sequenced here. D1 answered; #32 is new (§2). |
| S8 | #14, #18, #16, #19 | S2 | D2 answered: option (a). |
| S9 | #2, #17, #27 | S1 | |
| S10 | #6, #8, #9, #10 | S1 | Before S11, which adds warnings to the noise #10 reduces. |
| S11 | #7, #12, #30 | S2, S10 | |
| S12 | #24, #25 | S2, S3 | D3 answered; now also adds an `arm-app` settings surface, so e2e + portable e2e. |

**Critical path: S1 → S2 → S3 → S4 → S5.** Everything else hangs off it. **S2 and S4
are the two ripple-heavy slices** and are the ones most likely to exceed a single
subagent's useful scope — §4a already permits splitting a slice across two or three
subagents, and these two are where that is expected rather than optional. Suggested
splits: S2 into (a) the Rust enum + `character_types.json` + validation ownership, (b)
the frontend `STEPS`/`GUIDANCE_ARGS`/`ExperienceStep` + locales, (c) the e2e fillers and
rail assertions; S4 into (a) the schema fields + migration + Rust tests, (b) the
`life_stages`-presence sweep, (c) the frontend `setAbilityFunding` rewrite. **No time
estimates are given deliberately** — sequence and dependency are what govern here, and
the per-slice gate is the real pacing mechanism.

---

### Slice 1 — Base typography reset and a stable-height validation panel

- **Issues covered** #23, #3
- **Depends on** nothing. **Blocks** S2 (space budget), S6 (`minmax` floor), S9, S10.
- **Why this slice exists** One root cause. `ui/src/app.css` has **no base rule for
  `p` or headings** — it zeroes paragraph margins ad hoc in ~20 individual rules — so
  any unstyled `<p>`/`<h3>` keeps the UA `margin: 1em 0`. In flex columns margins do
  **not** collapse, so the UA margin *adds* to the container `gap`: between two
  single-line paragraphs inside `.detail-section` (`gap: 0.4rem`, `app.css:1083-1087`)
  the real spacing is 1em + 0.4rem + 1em ≈ **2.4rem** where 0.4rem was intended — ~6×,
  repeated per paragraph and heading. That is #23 (the aging step's whitespace) and
  #3 (the validation footer being *taller when empty* than when showing a violation:
  the empty-state `<p class="muted">` at `ValidationPanel.svelte:42` carries ≈3em of
  UA margin, while one `.issue` `<li>` inside the globally margin-reset `ul`
  (`app.css:682-686`) is ≈1.75rem). **This must be first**: it changes every block's
  height, and both S2's space budget and S6's `minmax` floor must be measured against
  real heights, not inflated ones.
- **Files to touch**
  - `ui/src/app.css` — add the base `p` / `h1`-`h6` margin reset; then audit the ~20
    existing rules that currently compensate for the UA default (start from
    `:532` `.wizard-review-outstanding`, `:556` `.region-title`, `:670` `.category`,
    `:1083-1087` `.detail-section`, `:1560-1564` `.validation-docked h2`) and remove
    the ones that are now redundant rather than layering the reset on top of them.
  - `ui/src/app.css:540` `.validation-bar`, `:1566-1569` `.validation-docked
    .issue-list`, `:1522-1527` `.issue` — give the panel a `min-height` sized for the
    "No issues" line so both states start from the same box.
  - `ui/src/lib/components/ValidationPanel.svelte:36-60` — only if the min-height
    needs a wrapper; prefer a pure CSS change.
- **TDD steps (strict red→green)**
  1. **RED** — `ui/src/lib/components/ValidationPanel.test.ts` (project **`ssr`**):
     add `renders the empty state and a single issue in a box of the same height`.
     CSS height is not observable in SSR, so assert the *structural* invariant that
     makes equal height possible instead: the empty state and the issue list are both
     rendered inside the same `min-height`-carrying element, i.e. a new stable wrapper
     element/class (e.g. `data-testid="validation-body"`) is present in **both**
     branches. This fails because no such wrapper exists today. Then make it pass.
  2. **RED** — a CSS-contract test. `app.css` is not unit-testable by rendering, so
     assert on the stylesheet text: add a new `ui/src/app.css.test.ts` (project
     **`ssr`**) with a test `app.css declares a base margin reset for p and headings`
     asserting a base `p,` selector block setting `margin: 0`. It fails now. This is the
     only mechanical guard against the reset being deleted by a later "cleanup".
     **Read the file with `readFileSync`, NOT an `import … from './app.css?raw'`.**
     This plan originally said `?raw` and that was **wrong**: vitest stubs CSS modules
     to an empty string and its check keys on the `.css` **extension regardless of the
     query**, so `?raw` yields `''` and every assertion passes vacuously — the exact
     failure mode the test exists to prevent. `ValidationPanel.test.ts` already used
     `readFileSync` for the same reason; follow it:
     `readFileSync(fileURLToPath(new URL('./app.css', import.meta.url)), 'utf-8')`.
     **Anchor element-level selectors at start-of-line** (`/^p,\s*h1,…/m`) so a
     descendant variant like `.panel p` cannot satisfy the assertion.
     **This applies to every later slice that extends this test** — S6 (grid, not
     multi-column; the aging log spanning the row), S9 (`.mastery-abilities`
     `flex-basis`, `.char-panel` centering) and S10 (`.issue` typography, the scoped
     `.error` rule). Verified in practice while implementing S1.
  3. Green: add the reset, then delete the now-redundant compensating rules **one at
     a time**, re-running `npm run test:unit` after each.
- **i18n** none.
- **Schema/migration** none.
- **Acceptance criteria**
  - No unstyled `<p>` or heading in the app carries a non-zero UA margin.
  - The validation panel occupies the same height with zero issues as with one.
  - No existing frontend test regresses; if a test asserted a spacing-derived value,
    it is updated deliberately with a comment saying why.
  - The full gate passes, `cargo tauri build --no-bundle` included.
- **Manual verification** — **this slice needs its own app-wide visual pass, not a
  drive-by.** Walk every editor tab (Details, Abilities, Arts, Spells, Possessions,
  Equipment, Totals, Personality/Aging as they exist pre-S3) and every wizard step for
  a magus and a grog, at a wide window and at ~800px height. Specifically: the aging
  step should lose the bulk of its vertical whitespace; the validation footer must not
  jump when the last issue clears (tick and untick a Living Condition to make one
  appear and disappear); check no heading has *collapsed* into its following text.

---

### Slice 2 — Phase-list surgery: drop `type`, split `abilities` into `experience` + `abilities`

- **Issues covered** #1, #11 *(DECIDED)*; unblocks #12, #16, #24, #28, #31
- **Depends on** S1. **Blocks** S3, S4, S8, S11, S12.
- **Why this slice exists** Both issues edit the same closed enum, the same four
  `creation_phases` arrays, and the same fan-out of phase-keyed tables. Doing them
  separately means two ripples through `CreationPhase`, two locale-parity churns and
  two e2e rail-assertion updates — and `CreationPhase::ALL` is declared as a
  **fixed-length array** (`crates/arm-rules/src/types.rs:1407`,
  `pub const ALL: [CreationPhase; 12]`), so batching keeps the length at 12 (remove
  `Type`, add `Experience`) instead of moving it 12→11→12. The review doc also notes
  #31's slug lookup must tolerate both changes, so settling them together means #31
  (S5) only ever sees the final list.
  - **#1**: the `type` step has no input at all — `StartScreen.svelte:73-82` enters
    the wizard *per type* via `store.startWizard(id)` and the type is immutable
    afterwards. Its only unique content is the V/F budget numbers and the Gift policy
    line; relocate those.
  - **#11**: three concerns share one bounded flex column — `LifeStagePanel` (≈620px:
    funding model, age, gauntlet, lab seasons, spell levels, native language,
    childhood picker + preview + slot inputs, six explanatory paragraphs),
    `MagusMinimumAbilities` (≈200px), then `.region-row` gets the remainder and hits
    its 12rem floor. Both preambles **must** be auto-height siblings of `.region-row`
    or the row collapses (verified: `AbilityTab.svelte:201-206`, `:209-215`, and the
    `.region-row` comment at `app.css:571-581` records this exact bug as the reason
    the floor exists).
- **Files to touch** (the `#11` fix-pass checklist, verified)
  - `crates/arm-rules/src/types.rs:1367-1399` — `CreationPhase`: remove `Type`, add
    `Experience`; `:1401-1420` `ALL` stays `[CreationPhase; 12]`.
  - `rules/core/character_types.json` — all four profiles' `creation_phases`:
    companion `:26-34`, grog `:58-65`, magus `:95-106`, mythic companion `:134-143`.
    Remove `"type"`; insert `"experience"` before `"abilities"` **only where
    life-stage funding applies** (see open question below).
  - `ui/src/lib/types.ts` — the `CreationPhase` TS union.
  - `ui/src/lib/components/WizardStep.svelte:44-57` — `STEPS`; delete the `type`
    entry, add `experience: { component: ExperienceStep, scroll: true }`. The
    `satisfies Record<CreationPhase, StepDef>` makes a missing entry a type error.
    Also update the reuse-rule comment at `:38-43` if this slice's split changes it.
  - `ui/src/lib/derive.ts:1149-1169` — `GUIDANCE_ARGS`; same `satisfies` guard.
  - `locales/en/main.ftl:37-48` and `locales/de/main.ftl` (mirrored block) —
    `phase-type` → `phase-experience`; likewise `wizard-guidance-<id>`.
    `locales/en/main.ftl:96-100` (`phase-type-explainer`, `phase-type-budget`,
    `phase-type-gift-*`) get **relocated**, not deleted (#1's two surviving facts).
  - The validation phase-ownership map `issuesForPhase` reads
    (`ui/src/lib/derive.ts:1060`) and the finding-code contract table
    (`crates/arm-rules/src/validation/mod.rs:118-180`): life-stage/funding codes
    (`life_stage_*`, `restricted_xp_unspent`, `childhood_*`) move from the `abilities`
    phase to `experience`; `magus_minimum_ability` / `magus_recommended_ability`,
    `not_enough_xp`, `duplicate_ability`, `ability_*` stay on `abilities`.
  - `ui/src/lib/components/TypeStep.svelte` + `TypeStep.test.ts` — deleted; their
    content relocated (see below).
  - **New** `ui/src/lib/components/ExperienceStep.svelte` — mounts `LifeStagePanel`
    (moved out of `AbilityTab.svelte:207`) and, per #11's DECIDED, the childhood
    picker/preview.
  - `ui/src/lib/components/AbilityTab.svelte:201-217` — drop the `LifeStagePanel`
    mount; keep `MagusMinimumAbilities` (S11 collapses it) and `.region-row`. **Then
    re-measure and decide whether `app.css:571-581` (`min-height: 12rem`) and
    `:614-618` (`.list-scroll`) floors are still needed** — they are prior
    symptom-patches for exactly this bug and should be **removed, not layered on**, if
    the split makes them unnecessary.
  - `ui/src/App.svelte:316` — **MANDATORY, do not skip: a temporary editor mount.**
    `AbilityTab` is mounted here as the **editor's** Abilities tab, and its own comment
    (`AbilityTab.svelte:203-206`) states the sharing is deliberate: *"This tab is
    mounted both as the editor's Abilities tab and as the wizard's `abilities` step, so
    one mount puts the panel in both flows; that is the point, since a character built
    in the wizard has to stay editable in the editor."* Removing `<LifeStagePanel />`
    from `AbilityTab` therefore removes **the editor's only surface** for choosing the
    funding mode and editing the life-stage plan (age, Gauntlet age, lab seasons,
    native language, childhood package) — and the editor does not get its replacement
    until S3 creates the Experience tab. Since §4a.6 forbids batching slices into one
    commit, that would leave a committed slice window in which direct-entry mode cannot
    edit any of it. **Existing e2e will not catch this**: `life-stage-childhood.e2e.js`
    and `magus-apprenticeship.e2e.js` drive the *wizard*, not the editor. So S2 mounts
    `LifeStagePanel` directly on the editor's Abilities tab in `App.svelte` as an
    explicitly temporary bridge, with a comment naming S3 as the slice that relocates
    it to the Experience tab. (The alternative — deferring the `AbilityTab` removal to
    S3 — is acceptable if the implementer prefers it, but then S2's abilities-step
    acceptance criterion about the lists filling the height cannot be met in S2 and
    must move to S3 too. Pick one and say which in the commit message.)
  - `ui/e2e/wizard-walk.js:139` `FILLERS` — add an `experience` filler; remove the
    `type` one. Dispatch at `:241-245` throws with the known-filler list if missing,
    so a forgotten filler fails loudly.
  - `ui/e2e/helpers.js` (`wizardRailPhases`, `currentWizardPhase`) and the four
    per-type wizard specs' rail assertions.
- **Sub-question — RESOLVED from data: all four profiles get `experience`.**
  The question was whether every profile gets the phase or only those where life-stage
  funding applies. Verified in the source:
  - `LifeStagePanel.svelte:66` gates the whole panel on `{#if rules}` —
    `store.ruleset?.ruleset.life_stages` (`:15`), i.e. **the ruleset shipping
    life-stage rules**. It does *not* gate on character type.
  - The funding-mode chooser (`pool` / `life_stages`, `:48-50`, `:68-93`) therefore
    renders for **every** character type. Only the detailed plan fields are behind
    `{#if guided}` (`:95`), and only gauntlet age / lab seasons / spell levels are
    behind `isMagus` (`:109`, `:136`, `:190`).
  - Childhood and later life apply to every character, not just magi — apprenticeship
    is the magus-only block.

  So a profile's `experience` step is **never empty**: at minimum it carries the
  funding choice, which today lives on the `abilities` step for all four types.
  Omitting the phase for non-magi would leave a grog with **no way to choose its
  funding mode at all** — a functional regression, not a tidier flow. Give all four
  profiles the phase.

  The magus step count goes 11 → 10 (#1) → 11 (#11 split), where 11 = 10 declared
  phases + the wizard's synthetic `review`.
- **TDD steps (strict red→green)**
  1. **RED (Rust)** — `crates/arm-rules/src/types.rs` tests: `creation_phase_all_has_no_type_phase` and
     `creation_phase_experience_serializes_as_experience` asserting
     `serde_json::to_string(&CreationPhase::Experience)` is `"\"experience\""` and
     that `ALL` contains no `Type`. Fails to compile → that *is* the red.
  2. **RED (Rust)** — `crates/arm-rules/tests/core_type_conformance.rs`: every
     profile's `creation_phases` parses and none contains `type`. Structural, never a
     count of phases.
  3. **RED (frontend, `ssr`)** — `ui/src/lib/components/WizardStep.test.ts`:
     `mounts the ExperienceStep for the experience phase`, and
     `has no step for a removed type phase`.
  4. **RED (frontend, `ssr`)** — a new
     `ui/src/lib/components/ExperienceStep.test.ts`: renders the life-stage panel and
     the childhood picker; `AbilityTab.test.ts` gets the mirror-image assertion that
     it **no longer** renders the life-stage panel while still rendering the
     Available/Selected regions.
  5. **RED (frontend, `ssr`)** — `ui/src/lib/i18n.test.ts`: `phase-experience` and
     `wizard-guidance-experience` exist in both locales and `phase-type` is gone
     (the existing parity test at `:68` covers the both-locales half).
  6. **RED (frontend, `ssr`)** — `ui/src/lib/derive.test.ts`: `issuesForPhase` routes
     a `life_stage_age_unset` finding to `experience`, not `abilities`.
  7. Green, then e2e: `wizard-walk.js` filler + rail assertions.
- **i18n keys** — en + de. **Added**: `phase-experience`,
  `wizard-guidance-experience`. **Relocated** (#1): `phase-type-explainer`,
  `phase-type-budget`, `phase-type-gift-required|forbidden|optional` are re-keyed to
  wherever they land (banner or head of the following step) — rename the keys to match
  their new home so no key claims a phase that no longer exists. **Removed**:
  `phase-type`, `wizard-guidance-type`. German: `phase-experience` needs a rules term
  — "experience" is `Erfahrung` / `Erfahrungspunkte`; check
  `rules/source/de/translation-tables/grundbegriffe.md` for the sanctioned form before
  writing it, and confirm against the German rulebook's own life-stage headings
  (`Ars Magica Definitive Edition Basisregeln.md:2390` "Späteres Leben" is the
  mirrored line of the English `:2390`).
- **Schema/migration** none *in this slice* — but note this changes the set of legal
  `CreationPhase` slugs, which S4/S5 will persist. Recording that dependency here is
  the whole reason S4 comes after S2.
- **Acceptance criteria**
  - A magus wizard run shows 11 rail steps: concept, characteristics, house,
    virtues & flaws, **experience**, abilities, arts, spells, personality &
    reputations, aging, review. No "Character type" step.
  - On the `abilities` step, the Available and Selected lists are the only content and
    fill the step's height; at an 800px window height both lists are visible and their
    controls clickable.
  - The V/F budget numbers and the Gift policy line are still reachable somewhere in
    the flow (their new home is stated in the commit message).
  - **The editor can still choose the funding mode and edit the whole life-stage plan**
    at the end of this slice, via the temporary `App.svelte` mount. Assert it in a test
    (`ui/src/App.test.ts`: `the editor still exposes the funding panel`), not by eye —
    this is the regression the temporary mount exists to prevent, and no existing e2e
    spec covers the editor's funding surface.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean — the closed enum
    means the compiler has enumerated every `match` site.
  - `npm run test:e2e` green, including all four per-type wizard walks.
- **Manual verification** Start a magus wizard. Confirm the rail. On the new
  Experience step, set funding to life stages, fill age/gauntlet/lab seasons/native
  language, pick a childhood package and fill its slots — nothing should be clipped or
  need an inner scroll to reach. Advance to Abilities: both lists full height, add and
  remove several abilities. Repeat at ~800px window height. Then do the same for a
  grog (no house/arts/spells) to confirm the gating.

---

### Slice 3 — Editor tabs mirror the wizard's phases

- **Issues covered** #28 *(DECIDED)*
- **Depends on** S2 — **strictly**. #28 mirrors the phase list, and S2 is the slice
  still changing it; mirroring first would mean mirroring twice. **Blocks** S12 (#24's
  age-cap-note home), S6 (the new Aging tab is where the grid lands in the editor).
- **Why this exists** The editor's tab structure and the wizard's phase list have
  drifted, and the crowding in #11 and the divergence in #27 are symptoms. Making them
  mirror each other is the structural fix.
- **Target mapping** (from the DECIDED table)

  | Wizard phase | Editor tab |
  |---|---|
  | concept | Details |
  | experience *(new, S2)* | **Experience** *(new)* |
  | abilities / arts / spells | Abilities / Arts / Spells |
  | personality_reputations | **Personality & Reputations** *(new, split out of Details)* |
  | aging | **Aging** *(new, split out of Details)* |
  | — | Possessions, Equipment, Totals *(no phase)* |

- **Files to touch**
  - `ui/src/App.svelte` — the tab list, tab order, `aria-controls` / `role="tabpanel"`
    ids, and the per-character-type gating each new tab needs (Experience only where
    life-stage funding applies; Aging wherever the aging subsystem is live).
  - `ui/src/lib/components/CharacterDetails.svelte:85-134` — Details keeps identity +
    age (+ Warping/Twilight, which have no wizard phase); Personality/Reputations and
    the aging cluster move out.
  - `ui/src/lib/components/AgingStep.svelte:15-25` and `AgingPanel.svelte:17-22` —
    the simplification #28 unlocks: `LongevityPanel` is currently mounted by
    `AgingStep` rather than inside `AgingPanel`, specifically to avoid giving a magus
    two on-screen homes for one ritual (its editor home is the magus-gated Possessions
    tab). With a real Aging tab it moves **into** `AgingPanel` and off Possessions,
    deleting the special case — and the comment explaining the special case goes with
    it.
  - `ui/src/app.css:1037-1065` — the `.character-details` multi-column block now
    describes less content; leave the `columns: 2` alone here, S6 replaces it.
  - `ui/e2e/helpers.js` + any spec selecting a tab by name/id
    (`tab-area.e2e.js`, `character-fields.e2e.js`, `aging.e2e.js`,
    `longevity-ritual.e2e.js`, `totals-tab.e2e.js`).
- **TDD steps**
  1. **RED (`ssr`)** — `ui/src/App.test.ts` (create if absent, else the nearest
     existing App-level test): `exposes an Experience tab for a life-stage-funded
     type`, `exposes separate Personality and Aging tabs`, and `does not offer an
     Aging tab for a type with no aging subsystem`. Assert on rendered tab markup and
     `aria-controls` wiring.
  2. **RED (`ssr`)** — `ui/src/lib/components/AgingPanel.test.ts` (new):
     `renders the Longevity panel`, plus `MagicPossessions.test.ts` gains
     `no longer renders the Longevity panel`.
  3. **RED (`ssr`)** — a tab-list-mirrors-phases test. **Source the phase list from the
     TypeScript side**, not from Rust: a vitest test cannot enumerate
     `CreationPhase::ALL`. Read the `CreationPhase` TS union in `ui/src/lib/types.ts`
     (or the loaded profile's `creation_phases` from a test ruleset fixture, which is
     the better choice since it is the same data the editor gates on), and assert the
     mapping table above holds for each phase that has an editor tab. Keep it
     structural (a known phase maps to a known tab), never a count. If Rust-side
     coverage of the same invariant is wanted, that is a separate `arm-rules` test.
  4. Green, then update the e2e selectors.
- **i18n keys** en + de: `tab-experience`, **`tab-personality-reputations`**,
  `tab-aging`. **Hyphens, not underscores** — verified against the shipped keys
  (`locales/en/main.ftl:196-206`, `:950`): `tab-virtues-flaws`,
  `tab-house-specialisation`, `tab-mythic-type`. The phase *slugs* use underscores
  (`personality_reputations`) but the `tab-*` Fluent keys do not, so the two do not
  spell alike and the mapping must not be generated by string-substituting one into
  the other.
  German: reuse the exact wording already used for the corresponding `phase-*` keys so
  a tab and its wizard step read identically; `Aging` is `Altern` in the existing
  `phase-aging` / `aging-label` strings — do not coin a second term.
- **Schema/migration** none. Tab selection is UI state, not saved.
- **Acceptance criteria**
  - Every wizard phase with an editor counterpart has exactly one tab, in phase order.
  - `LongevityPanel` appears on the Aging tab and nowhere else; the Possessions
    special case and its explanatory comment are gone.
  - **Experience tab gating — CORRECTED by Slice 2's finding.** This criterion
    originally read "no empty Experience tab for a type that cannot use life-stage
    funding". That premise is **false**: `LifeStagePanel.svelte:66` gates on
    `{#if rules}` — the ruleset shipping life-stage rules — **not** on character type,
    so every type can choose its funding mode. Slice 2 gave all four profiles the
    `experience` phase for exactly this reason. So the Experience **tab** is likewise
    gated on the ruleset shipping life-stage rules, not on the type, and it appears for
    all four types. Gate it on the same condition the panel uses, so the tab and its
    content can never disagree; a tab whose panel self-gates to empty is the failure
    mode to avoid, and reading the same flag in both places is what prevents it.
  - The Aging tab is gated wherever the aging subsystem is live (unchanged).
  - `aria-controls`/`tabpanel` ids resolve (no dangling references) — assert in the
    test, not by eye.
- **Manual verification** In the editor (direct entry), for each of the four
  character types: cycle every tab, confirm no tab is empty, confirm Details no longer
  carries Personality/Aging, confirm the Longevity ritual has exactly one home.
  Keyboard-navigate the tab list with arrow keys to confirm the ARIA wiring.

---

### Slice 4 — Schema 16: explicit funding discriminator, plus the wizard-progress field

- **Issues covered** #29 *(DECIDED)*; adds #31's field (behavior in S5)
- **Depends on** S2 (the final `CreationPhase` slug set). **Blocks** S5.
- **Why this exists and why the bump is batched** #29 is the largest item on the list
  — engine + schema + UI. #31 also needs a new persisted field. The review doc's
  sequencing note asks whether the two implied `SCHEMA_VERSION` bumps batch: **they
  do, and they must.** Two sequential bumps would mean two migrations, two
  round-trip-test updates, and a window in which a save written at 16 is unreadable by
  a build at 17. This slice performs **one** bump, 15 → 16, adding **both** fields;
  S5 then only adds *behavior* against a field that already exists and already
  migrates.
- **The bug (#29)** `state.svelte.ts:761-773` `setAbilityFunding` destroys typed input
  in both directions, unconfirmed and unrecoverable: pool → life stages sets
  `this.entity.xp_pool = 0` (a hand-typed total is zeroed); life stages → pool does
  `delete this.entity.life_stages`, taking the whole plan (native language, Gauntlet
  age, lab seasons, spell levels, childhood package) plus `childhoodDraft`,
  `childhoodRejections` and the aging draft. A round trip loses everything typed on
  either side. The unsaved-changes guard governs quitting, not destructive in-app
  edits. The code's own doc comment (`:740-759`) says the destruction is intended
  while simultaneously arguing that bought `ability_scores` must survive because
  losing those would be worse — so the principle is already "don't discard the
  player's work"; the plan and pool just were not held to it.
- **The structural blocker** `abilityFunding = $derived(this.entity.life_stages ?
  'life_stages' : 'pool')` (`state.svelte.ts:420`, verified) — the **presence** of the
  plan *is* the mode. Nothing can stop destroying the plan until the mode is stored
  separately.
- **Files to touch**
  - `crates/arm-rules/src/types.rs:2808` — `SCHEMA_VERSION: u32 = 15` → `16`.
  - `crates/arm-rules/src/types.rs` `Entity` — two new fields:
    `ability_funding: AbilityFunding` (new enum, serde-defaulted so old saves parse)
    and `wizard_furthest_phase: Option<String>` (skip-if-none). Both need
    `///` docs. **Document `wizard_furthest_phase` as UI/document state** that lives
    on an engine-defined entity so nobody wires validation to it.
    **Why `Option<String>` and not `Option<CreationPhase>`** — this matters, and the
    obvious typing is wrong. `CreationPhase` is a closed `Deserialize` enum with
    `#[serde(rename_all = "snake_case")]` (`crates/arm-rules/src/types.rs:1365-1367`),
    so an unrecognised slug is a **serde error**, and §6's rule that a value which will
    not deserialize fails the *whole load* would turn any unknown slug into an
    unopenable file. That directly contradicts S5's required behaviour (a save carrying
    `"type"` — a slug **this plan removes in S2** — must fall back to the ungated
    branch, not error) and #31's DECIDED intent that a slug absent from the profile's
    phase list is treated as absent. It is also a forward-compatibility trap: every
    future phase rename would make newer saves unopenable in older builds. So store the
    raw slug and resolve it **leniently at restore**, against the loaded profile's
    `creation_phases` — which S5 must do anyway, since a slug can be a *valid*
    `CreationPhase` and still not be declared by this character's profile. (A lenient
    custom deserializer mapping unknown → `None` is an acceptable alternative, but it
    hides the distinction between "no progress recorded" and "progress recorded in a
    vocabulary we no longer speak" and buys nothing here.)
  - `crates/arm-rules/src/types.rs:3034-3080` `load_entity_migrating` — the migration.
    Follow the established pattern: **dispatch on legacy-key presence / field absence,
    never on the recorded version** (a hand-edited save may carry any
    `schema_version`), stamp `SCHEMA_VERSION` when a fold happens, and fail the whole
    load on a value that will not deserialize. Migration rule: `ability_funding`
    absent → infer from `life_stages` presence (today's rule, applied once at load).
  - `crates/arm-rules/src/life_stage.rs:453-454` — `entity.life_stages.as_ref()?`,
    where `None` currently means "pool". This and every other site inferring the mode
    from plan presence must read the field instead. **The closed-enum discipline will
    not catch these** — they are `Option` checks, not `match`es — so this needs a
    deliberate `grep` for `life_stages` across `crates/arm-rules/src/` and
    `ui/src/lib/`, not just a compile. Known sites to sweep: `life_stage.rs:453`, the
    `not_enough_xp` and restricted-pool paths in
    `crates/arm-rules/src/validation/life_stage.rs` (`restricted_xp_unspent` at
    `:647-648`) and `validation/scores.rs`, `completeness.rs`, and `export.rs`.
  - `crates/arm-rules/src/validation/mod.rs:376`
    (`CODE_LIFE_STAGE_XP_POOL_CONFLICT`) and its emitter — the
    `life_stage_xp_pool_conflict` finding asserts a mutual exclusivity that no longer
    holds. **Remove it** (and its `issue-life_stage_xp_pool_conflict` strings in both
    locales, and its row in the contract table at `mod.rs:144`). If any test asserts
    it fires, invert the test.
  - `ui/src/lib/types.ts` — mirror both fields and the `AbilityFunding` enum.
  - `ui/src/lib/state.svelte.ts:420` — read the field instead of deriving it;
    `:761-773` — `setAbilityFunding` becomes a **pure mode set**: no deletion, no
    zeroing. **Keep pruning drafts** (`childhoodDraft`, `childhoodRejections`, aging
    draft) per the existing blanket rule that an un-submitted draft never outlives a
    change to how the document is built (`:756-759`); only the plan and pool stop
    being destroyed. Rewrite the `:736-759` doc comment — it currently documents the
    destruction as intentional.
  - `crates/arm-rules/RULES.md` — record the **deliberate departure from the
    sparse-save principle**: the save now keeps data the active mode ignores, so
    nobody "fixes" it back. Record the **retired invariant** ("the engine makes a plan
    and a typed pool mutually exclusive") explicitly. Record
    `wizard_furthest_phase` as UI/document state with no validation attached.
  - `examples/` — any sample save whose shape the new field affects; keep canonical
    sorted serialization.
- **TDD steps (strict red→green)**
  1. **RED (Rust)** — `crates/arm-rules/src/types.rs` tests:
     `schema_version_is_16`. **There are TWO existing `assert_eq!(SCHEMA_VERSION, 15)`
     guards, not one** — `crates/arm-rules/src/life_stage.rs:1515` **and**
     `crates/arm-rules/src/types.rs:5366` (verified 2026-08-28; the plan originally
     named only the first). Both must fail, then both be updated deliberately. Grep for
     `SCHEMA_VERSION` before assuming the set is complete; a guard left at 15 that
     someone "fixes" by weakening the assertion is worse than no guard.
     Note also that the anchors in this slice have drifted a few lines as Slices 2-3
     landed: `SCHEMA_VERSION` is now `types.rs:2812` (not `:2808`) and
     `load_entity_migrating` is `types.rs:3038` (not `:3034`). Re-verify before citing. 
     `entity_without_ability_funding_migrates_from_life_stages_presence` (a save with
     `life_stages` and no `ability_funding` loads as life-stage funding);
     `entity_without_ability_funding_or_plan_migrates_to_pool`;
     `wizard_furthest_phase_is_omitted_when_none` (byte-level: the key is absent from
     the serialized JSON); and — **required, this is the test that pins the
     `Option<String>` decision** —
     `a_save_with_an_unknown_wizard_phase_slug_still_loads`, loading an entity whose
     `wizard_furthest_phase` is `"type"` (or `"not_a_phase"`) and asserting the load
     **succeeds**. With `Option<CreationPhase>` this test cannot pass, which is exactly
     why it belongs here rather than in S5.
  2. **RED (Rust)** — `crates/arm-rules/tests/roundtrip_proptest.rs`: both fields
     survive a round trip. Existing proptest should surface this once the fields exist.
  3. **RED (Rust)** — `crates/arm-rules/src/life_stage.rs`:
     `budget_is_none_for_pool_funding_even_with_a_stored_plan` — the case that could
     not exist before this slice and is the whole point of it.
  4. **RED (Rust)** — `crates/arm-rules/src/validation/`:
     `no_conflict_finding_when_a_plan_and_a_typed_pool_coexist` (the inverted
     assertion).
  5. **RED (frontend, `ssr`)** — `ui/src/lib/state.svelte.test.ts`:
     `setAbilityFunding preserves a typed xp_pool when switching to life stages`,
     `setAbilityFunding preserves the life-stage plan when switching to pool`,
     `setAbilityFunding still clears the childhood draft when leaving life stages`,
     and a **round-trip** test: pool → life_stages → pool leaves both the typed pool
     and the plan intact.
  6. Green. Then the deliberate `grep` sweep for `life_stages`-presence checks, each
     one converted with a test naming the behavior it fixes.
- **i18n keys** **Removed**: `issue-life_stage_xp_pool_conflict` (en + de). No new
  user-facing strings expected; if `setAbilityFunding` grows a confirmation or a note
  it needs keys in both locales.
- **Schema/migration** **This slice owns the single bump, 15 → 16.** See §6.
- **Acceptance criteria**
  - Switching funding mode in either direction loses **no** typed entity data; only
    un-submitted drafts are pruned.
  - A schema-15 save with `life_stages` loads at 16 in life-stage mode; one without
    loads in pool mode; neither reports a spurious finding.
  - `life_stage_xp_pool_conflict` no longer exists anywhere — code, contract table, or
    locale.
  - No site in `crates/arm-rules/src/` or `ui/src/lib/` infers the funding mode from
    `life_stages` presence. Verify by grep and state the result in the commit message.
  - `crates/arm-rules/RULES.md` records the sparse-save departure and the retired
    invariant.
  - `npm run test:e2e` green — save/load is exercised.
- **Manual verification** Build a life-stage-funded magus: set age, gauntlet age, lab
  seasons, native language, a childhood package with filled slots. Switch to pool
  funding, type a pool total, switch back to life stages — every previously entered
  value must still be there, and the pool total must still be there after switching
  back to pool. Save, quit, reopen the file: same. Then open a save written by the
  pre-slice build (`examples/`) and confirm it lands in the right mode.

---

### Slice 5 — Open a saved character into the guided wizard, with persisted progress

- **Issues covered** #31 *(DECIDED)*
- **Depends on** S4 (the field and its migration already exist) and S2 (the phase slug
  set is final). **Blocks** nothing.
- **Why this exists** Wizard progress is not persisted at all today — *"moving through
  the flow is not an edit, so it never dirties the document and a save records no
  progress through it"* (`wizard-navigation.svelte.ts:31-45`, verified). A file saved
  on step 6 is therefore indistinguishable from one built in the editor, which means
  implementing the mid-wizard resume case **is** implementing the general case. It is
  cheap because `startWizard` is only three things
  (`state.svelte.ts:1924-1929`, verified: `#instantiateCharacter`, `view = 'wizard'`,
  `#resetWizardNav`) and only the first is new-character-specific.
- **DECIDED restore behavior**
  - **Field present** → wizard-saved. Restore as `furthest`, land there, and **clamp
    as during the original run** (a forward jump stops at the first blocking phase —
    `firstBlockedPhaseIndex`, `derive.ts:1113`).
  - **Field absent, or the stored slug is not in the current profile's phase list** →
    manually created or migrated. **All steps reachable, exempt from the clamp**,
    findings shown but never gating. (The slug-not-in-list branch is exactly what S2's
    phase changes make reachable — a save carrying `"type"` or a pre-split
    `"abilities"` must degrade gracefully, which is #1's stated interaction.)
- **Why a slug, not an index** An index resolves against `creation_phases`, which is
  data and which S2 just changed twice. A slug survives both and matches the project
  rule that saves store choices, not resolved values, with slug-style non-positional
  IDs.
- **Files to touch**
  - `ui/src/lib/wizard-navigation.svelte.ts:38-45` (`step`, `furthest`), `:112-117`
    (`next()` — the **only** place `furthest` rises; `back()` and `goTo()` never touch
    it, which is why persisting it keeps rail browsing free) and the clamp path.
  - `ui/src/lib/state.svelte.ts:1924-1929` — a second entry point that skips
    instantiation: `view = 'wizard'` after `open()`, restoring nav from
    `entity.wizard_furthest_phase`. Plus a **guard so the action is not offered for a
    `type_id` the loaded ruleset has no profile for**.
  - `ui/src/lib/state.svelte.ts` dirty-flag path — `next()` now writes
    `wizard_furthest_phase`, so it dirties the document **even on an unedited step** (a
    step can be legally empty; `canAdvance` gates on errors only,
    `wizard-navigation.svelte.ts:78-80`). Defensible, but deliberate.
  - `ui/src/lib/components/StartScreen.svelte` and/or `SaveLoadBar.svelte` — the entry
    point UI.
  - `crates/arm-rules/RULES.md` — already covered by S4's entry; extend it with the
    restore semantics if the behavior needs recording.
- **TDD steps**
  1. **RED (`ssr`)** — `ui/src/lib/wizard-navigation.svelte.test.ts` (or the existing
     home of nav tests): `restores furthest from a stored phase slug`,
     `ignores a stored slug the profile does not declare`,
     `an absent slug leaves every step reachable and ungated`,
     `next() raises furthest and back() does not lower it`.
  2. **RED (`ssr`)** — `ui/src/lib/state.svelte.test.ts`:
     `opening a saved character into the wizard does not instantiate a blank one`,
     `does not offer the wizard for a type_id with no profile in the loaded ruleset`.
  3. **RED (`ssr`)** — `ui/src/lib/state.svelte.test.ts`, the **unsaved-changes guard
     case the review doc explicitly asks for**: `advancing a wizard step dirties the
     document`, and its complement `rail navigation does not dirty the document`.
     These are the mandatory-product-behavior tests; they must be added, not just kept
     green.
  4. **RED (`client`)** — if and only if the `update_close_guard` `$effect` mirror
     needs a new assertion: extend the existing client test that covers it. **Must be
     `client`** because SSR never executes an `$effect` body, so an assertion after it
     would report green while the effect never ran.
  5. Green. Then e2e: a spec that saves mid-wizard, reopens the file into the wizard,
     and lands on the stored phase.
- **i18n keys** en + de: the entry-point label (e.g.
  `open-into-wizard` / `start-wizard-from-file`) and, if the type-profile guard needs
  to explain itself, a disabled-reason or error string. German from the existing
  wizard vocabulary in `locales/de/main.ftl` — do not coin a new term for "wizard".
- **Schema/migration** None — S4 already added and migrated the field.
- **Acceptance criteria**
  - A character saved on wizard step 6 reopens into the wizard on step 6, with forward
    jumps clamped exactly as during the original run.
  - A character built in the editor (no stored slug) opens into the wizard with every
    step reachable and no gating.
  - A save whose stored slug is `"type"` (removed in S2) or otherwise unknown falls
    back to the ungated branch rather than erroring.
  - Rail clicks do not dirty the document; a Next click does.
  - The unsaved-changes guard still prompts on close **and** on quit, including Cmd+Q;
    `window-close-bridge-clean` and `window-close-bridge-dirty` e2e specs green.
  - `npm run test:e2e` green — **mandatory** for this slice (new entry point,
    persisted state, lifecycle interaction).
- **Manual verification** Start a magus wizard, advance to step 6, save. Quit (confirm
  the guard prompts if there are unsaved edits). Relaunch, open the file into the
  wizard: it should land on step 6 and refuse to jump past the first blocking phase.
  Separately: build a character in the editor, save, open it into the wizard, and
  confirm every rail step is clickable. Then click around the rail without editing and
  confirm the title bar shows no unsaved-changes marker; click Next once and confirm it
  does.

---

### Slice 6 — The aging step: grid layout, redundant line, missing formula term, wrong control

- **Issues covered** #20 *(DECIDED)*, #21, #22, #26
- **Depends on** S1 (**strictly** — set the `minmax` floor against real block heights,
  not heights inflated by the missing margin reset) and S3 (the Aging tab is where the
  editor mount now lives). **Blocks** nothing.
- **Why this slice exists** All four are the aging surface, and #20's re-measurement
  step wants the other three's height changes already in place before the floor is
  chosen.
  - **#20**: `.character-details { columns: 2 }` (`app.css:1037-1040`, verified) is CSS
    **multi-column**, which *flows* content between columns, so any height change shifts
    the column break and blocks migrate — a whole-panel reflow on a single checkbox.
    Evidence in the current build: `AgingRecordPanel` is **split across the break**,
    with "Apparent age" at the bottom of the left column and its own "Aging / Aging
    points per Characteristic" section at the top of the right. **Key fact:** CSS
    **grid** auto-placement is order-stable under height changes — each item owns its
    cell, so a block growing changes only row heights. Multi-column is not.
  - **#21**: `LivingConditionsPicker.svelte:96-106` — `living-conditions-none` and
    `living-conditions-total` are not alternatives; with nothing ticked you get *"No
    Living Conditions chosen: the character counts as an average peasant (0)"*
    immediately followed by *"Living Conditions modifier: +0"*.
  - **#22**: `locales/en/main.ftl:497` `aging-total-formula` names exactly three terms,
    but `AgingSchedulePanel.svelte:32` fills `fixed` from `aging.fixed_total`, the
    engine's sum of **all** of them including the V/F aging-roll modifiers the book's
    three lines do not name. Hence the visible contradiction *"+4 (age) 0 (living
    conditions) 0 (Longevity Ritual) = stress die +3"* for a character with Faerie
    Blood (−1). Proof it is an oversight: the sibling `aging-total-parts`
    (`en:525`, verified) **does** carry a `{ $traits } (Virtues and Flaws)` term.
  - **#26**: `AgingRecordPanel.svelte:96-106` (verified) uses `<input type="text">`
    for `decrepitude_effect`, which `RULES.md:1243-1246` documents as *"free-text
    overall aging/decrepitude narrative"* sourced to Core Rules `:16563-16577` — a
    cumulative description that grows over a character's life. Its analogue
    `warping_effect` is already `<textarea rows="3">`
    (`CharacterDetails.svelte:128-134`).
- **Files to touch**
  - `ui/src/app.css:1037-1065` — replace `columns: 2` with
    `display: grid; grid-template-columns: repeat(auto-fit, minmax(~22rem, 1fr));
    align-items: start`, and **replace** (not keep) the existing 720px `columns: 1`
    media query — `auto-fit` gives natural 1/2/3-column response. The
    `display: contents` wrappers at `:1056-1059` keep working: children become grid
    items exactly as they became column items. Re-target the
    `break-inside: avoid; margin-bottom` rules at `:1042-1045` and `:1061-1065` to
    grid semantics (`break-inside` is meaningless in grid; the margin may be replaced
    by `gap`).
    **`.character-details` is carried by THREE components, not one.** Verified:
    `AgingStep.svelte:30`, `CharacterDetails.svelte:83` (the **editor's Details tab**)
    and `PersonalityReputationsStep.svelte:9`. #20 was reported against the aging step
    only, but the class is shared, so this change restyles all three. That is almost
    certainly *desirable* — order-stability is a win everywhere, and cross-cutting
    theme 1 argues for it — but it must be **deliberate and verified**, not a silent
    side effect. Two options: scope the grid to a dedicated aging container class
    (narrower, but leaves the other two on the reflowing layout), or keep the shared
    class and verify all three surfaces. **Prefer the shared class**, and extend the
    acceptance criteria and manual pass accordingly.
    **Two comments reason about multi-column and become wrong.**
    `CharacterDetails.svelte:99` explains a source-order arrangement chosen so *"the
    `.character-details` multi-column flow lands them together"*, and
    `AgingStep.svelte:27` opens *"`.character-details` is a CSS multi-column flow in
    which source order governs …"*. Under grid auto-placement source order still
    governs *placement order*, but "lands them together in a column" no longer follows.
    Re-read both comments, re-check whether the source ordering they justify is still
    the right ordering, and rewrite them — a stale comment explaining a layout model
    the file no longer uses is worse than none.
  - Same block — the **aging log gets `grid-column: 1 / -1`**: its own full-width row
    with a bounded `max-height` and its own scrollport. It is the only
    unbounded-growth block, and the narrow column is also truncating its fields
    ("Describe the aging roll's e…").
  - `ui/src/lib/components/AgingStep.svelte` → `AgingPanel.svelte:17-22`; the blocks
    that become grid items: `AgeFields`, `AgingSchedulePanel`,
    `LivingConditionsPicker`, `AgingRollCalculator`, `AgingRecordPanel` (several
    items), `LongevityPanel`.
  - `ui/src/lib/components/LivingConditionsPicker.svelte:96-106` — drop the `none`
    line; if the "average peasant" framing is worth keeping, move it into the
    always-present `living-conditions-hint` (one stable line replaces two-then-one,
    which also removes a height change and so serves the cross-cutting theme).
  - `locales/en/main.ftl:497` + `locales/de/main.ftl:500` — add the traits term to
    `aging-total-formula`, hidden or shown-as-0 when zero, matching the other terms.
    Copy the term's wording from the sibling `aging-total-parts`
    (`en:525` `{ $traits } (Virtues and Flaws)` / `de:528`
    `{ $traits } (Tugenden und Fehler)`) so the two strings agree.
  - `ui/src/lib/components/AgingSchedulePanel.svelte:32` — pass `traits`.
  - `ui/src/lib/components/AgingRecordPanel.svelte:96-106` —
    `<textarea rows="3">`; `setDecrepitudeEffect` is unchanged, only the event cast
    becomes `HTMLTextAreaElement`. (The review doc checked the remaining single-line
    free-text fields — aging-log `effect`, twilight scars, ability specialty,
    parameter values — and confirmed they are correctly single-line. Do not change
    them.)
- **TDD steps**
  1. **RED (`ssr`)** — `ui/src/lib/components/LivingConditionsPicker.test.ts`:
     `renders exactly one summary line when nothing is chosen` (asserting the
     `living-conditions-none` testid is absent and `living-conditions-total` present).
  2. **RED (`ssr`)** — `ui/src/lib/components/AgingSchedulePanel.test.ts`:
     `the total formula names the Virtue/Flaw term and its terms sum to the stated
     total`. Use a character with a −1 aging modifier so the arithmetic is checkable
     in the assertion, not just the presence of a placeholder.
  3. **RED (`ssr`)** — `ui/src/lib/i18n.test.ts`: `aging-total-formula` carries a
     `$traits` argument in **both** locales.
  4. **RED (`ssr`)** — `ui/src/lib/components/AgingRecordPanel.test.ts`:
     `renders the decrepitude effect as a textarea` (assert the rendered tag), plus
     `typing into the decrepitude textarea updates the entity` to prove the cast
     change did not break the handler.
  5. **RED (CSS contract, `ssr`)** — extend the stylesheet-text test added in S1:
     `the character-details layout uses grid, not multi-column` and
     `the aging log spans the full grid row`. This is the only mechanical guard against
     a future "simplification" back to `columns`.
  6. Green. **Then re-measure** (per #20's DECIDED sequence: S1 first, then measure,
     then set the floor) and pick the `minmax` floor against real block widths.
- **i18n keys** en + de. **Changed**: `aging-total-formula` (gains `$traits`).
  **Removed**: `living-conditions-none` (both locales) — or, if the "average peasant"
  framing is kept, its text is folded into `living-conditions-hint` in both locales.
  German wording for the traits term is already established at
  `locales/de/main.ftl:528` (`Tugenden und Fehler`) — reuse it verbatim, do not
  re-translate.
- **Schema/migration** none. `decrepitude_effect` is the same `String` field; only its
  control changes.
- **Provenance** #22 changes a *displayed* rules readout, so the traits term needs the
  source citation for V/F aging-roll modifiers at the `AgingSchedulePanel` fill site
  and a `RULES.md` cross-reference to the existing `aging.fixed_total` entry. #26 needs
  no new mechanic (`RULES.md:1243-1246` already covers the field); note the control
  change in the same entry only if RULES.md describes the control.
- **Acceptance criteria**
  - Ticking a Living Condition, adding an aging-log row, or toggling a Longevity
    Ritual changes **no other block's position** — only row heights.
  - `AgingRecordPanel` is never split across a column boundary.
  - The aging log occupies a full-width row with its own bounded scrollport, and its
    "effect" field is wide enough not to truncate.
  - The aging total formula's named terms sum to the stated total for a character with
    a V/F aging modifier.
  - The layout responds 1 / 2 / 3 columns by available width with no media query
    breakpoint list.
  - Both mounts covered: the wizard's Aging step **and** the editor's new Aging tab.
  - **All three `.character-details` surfaces verified**, not just the aging ones: the
    editor's **Details** tab (`CharacterDetails.svelte:83`) and the wizard's
    **Personality & Reputations** step (`PersonalityReputationsStep.svelte:9`) lay out
    correctly under grid, with nothing overlapping, orphaned or wider than its cell.
  - The two multi-column comments (`CharacterDetails.svelte:99`, `AgingStep.svelte:27`)
    describe the layout the code actually uses.
- **Manual verification** On the wizard's Aging step (magus, with Faerie Blood or any
  aging-modifier Virtue): tick and untick a Living Condition and watch that no block
  jumps to another column. Add five aging-log rows and confirm the log scrolls in place
  rather than pushing everything. Type a long decrepitude description and confirm the
  textarea grows to 3 rows and scrolls. Read the total formula and check the arithmetic
  by hand. Resize the window from narrow to wide and confirm 1 → 2 → 3 columns. Repeat
  on the editor's Aging tab. **Then the two collateral surfaces**: the editor's Details
  tab and the wizard's Personality & Reputations step both carry `.character-details`
  and are relaid out by this slice — walk each at narrow, medium and wide widths and
  confirm nothing regressed.

---

### Slice 7 — The parameter-domain cluster: `form`/`technique`/`item` pickers, mis-declared domains, unfilled-instance labels

- **Issues covered** #4 (**high**), #5, #13, **#32** (new — see D1 follow-on in §2).
  D1 is **answered**: #13 is implemented as option (d), an optional `name_unfilled` in
  the i18n layer. #32 is implemented as the widened-check-plus-exemplar shape.
- **Depends on** nothing structural; sequenced here. **Blocks** nothing.
- **Why this slice exists** One chain: the picker has no branch for the Art-subtype
  domains (#4), five catalogue items declare the wrong domain so even a correct branch
  would offer the wrong choices (#5), and the label path doubles a placeholder when an
  instance is unfilled (#13). All three are **shared-component / shared-data** issues
  and therefore edit-mode bugs too.
  - **#4** (verified): `ParameterPicker.svelte:154-217` has branches for
    `characteristic` (`:154`), `ability` (`:171`) and `art` (`:191`) only; everything
    else falls to the `<input type="text">` at `:209-216`. `ParameterDomain` has
    **seven** variants (`crates/arm-rules/src/types.rs:319-342`, mirrored
    `ui/src/lib/types.ts:35-42`): `ability, art, technique, form, characteristic,
    item, text` — and only `text` is *meant* to be an input; its doc comment says so.
    Not cosmetic: `Form`/`Technique` are validated as Art ids **plus** the right
    `ArtType`, raising `unknown_param_value` otherwise (`types.rs:325-332`), so the
    only strings that pass are internal slugs like `art.ignem`. Typing "Ignem" or a
    German label fails — and rendering the slug at all breaks the CLAUDE.md rule that
    a raw ID must never be user-facing.
  - **Affected catalogue items, exhaustive over `rules/core/`**: `virtue.deft_form`
    (`domain: form`, `virtues_flaws.json:3741`) — the reported case;
    `flaw.deficient_form` (`:600`); `flaw.deficient_technique` (`:611`). The `item`
    domain is in the enum but used by no catalogue entry today — **latent only**, so
    add the branch (the enum is exhaustive) but do not invent catalogue data for it.
  - **The proven control to reuse**: `SpellTab.svelte:391-407` already renders a
    Forms-only `<select>` for the meta-magic Vim spells (also `domain: form`) and gets
    it right. Extract or mirror it; do not write a third Art picker.
  - **#5**: five V/F declare a Form-only parameter as `domain: "art"`, which the
    engine cannot catch because `art` accepts either Art type:
    `flaw.form_monstrosity` (`virtues_flaws.json:1147`),
    `flaw.hunger_for_form_magic` (`:1409`), `virtue.extractor_of_form_vis`,
    `virtue.imbued_with_the_spirit_of_form`, `virtue.master_of_form_creatures` (the
    last is even named in `ParameterPicker.svelte:193-194` as declaring `form` under
    the `art` domain). **Verify each item's cited line range in its source book
    first**; where the rules restrict it to a Form, change to `"domain": "form"` — a
    one-word data change that #4's new branch then renders correctly.
  - **#13**: `MagusMinimumAbilities.svelte:49-56` calls
    `abilityDisplayName(…, instanceOf(row), paramHint(store.t))`; with no instance
    held, `instanceOf` (`:36-46`) returns `null` and the label falls back to the
    parenthetical hint, stacked on the ability's own parenthesized template name —
    `rules/i18n/en/abilities.json:23` is `"{language} (Dead Language)"` and
    `param-label-language` is `"(Language)"` (`locales/en/main.ftl:772`). The same
    doubling appears in the **validation message**, so the fix belongs in the shared
    label path (`ui/src/lib/derive.ts:57` `paramHint`, `:639` `abilityDisplayName`,
    and `displayName`), not the component.
- **Files to touch**
  `ui/src/lib/components/ParameterPicker.svelte:154-217`;
  `ui/src/lib/components/SpellTab.svelte:391-407` (extract the Forms select, or its
  option-building helper, into a shared place); `ui/src/lib/derive.ts:57`, `:639`;
  `crates/arm-rules/RULES.md` (the #5 domain corrections are rules-data changes and
  need their source ranges recorded).

  **#4 is code-only. #5 is the data change — and only five entries change.** Keep these
  strictly apart; conflating them invites a subagent into "fixing" entries that are
  already right. Verified state of `rules/core/virtues_flaws.json`:

  | Line | Entry | `key` | `domain` | Action |
  |---|---|---|---|---|
  | `:600` | `flaw.deficient_form` | `form` | `form` | **Already correct — do not touch.** #4's new branch is what makes it render. |
  | `:611` | `flaw.deficient_technique` | `technique` | `technique` | **Already correct — do not touch.** |
  | `:3741` | `virtue.deft_form` | `form` | `form` | **Already correct — do not touch.** This is the *reported* case, and the bug is entirely in the picker. |
  | `:1147` | `flaw.form_monstrosity` | `form` | `art` | **#5: change to `form`** if the source restricts it. |
  | `:1409` | `flaw.hunger_for_form_magic` | `form` | `art` | **#5: change to `form`** if the source restricts it. |
  | `:3970` | (Form-parameter V/F) | `form` | `art` | **#5: change to `form`** if the source restricts it. |
  | `:4481` | (Form-parameter V/F) | `form` | `art` | **#5: change to `form`** if the source restricts it. |
  | `:5092` | (Form-parameter V/F) | `form` | `art` | **#5: change to `form`** if the source restricts it. |
  | `:3274`, `:5533` | `key: "art"`, `domain: "art"` | `art` | `art` | **Correct by design — either Art type is legal. Do not touch.** |

  Locate the five by **id**, not by line (a preceding edit shifts them), then re-verify
  the line before citing it. The three unnumbered ids from the review doc
  (`virtue.extractor_of_form_vis`, `virtue.imbued_with_the_spirit_of_form`,
  `virtue.master_of_form_creatures`) are the `:3970` / `:4481` / `:5092` rows — confirm
  the id↔line pairing yourself rather than trusting this ordering. And **verify each
  item's cited source range in its own book before changing its domain**: the change is
  only justified where the rules actually restrict the parameter to a Form.
- **TDD steps**
  1. **RED (`ssr`)** — `ui/src/lib/components/ParameterPicker.test.ts` (create if
     absent): `renders a Forms-only select for a form-domain parameter`,
     `renders a Techniques-only select for a technique-domain parameter`,
     `renders a select, not a text input, for an item-domain parameter`, and
     `renders a text input only for the text domain`. Assert on **localized option
     labels**, not slugs — that is simultaneously the #4 fix and the
     no-raw-IDs invariant.
  2. **RED (Rust)** — `crates/arm-rules/tests/data_integrity.rs`:
     `virtue_deft_form_declares_the_form_domain` and a companion for each of the five
     #5 items. Structural per-item assertions, never a count.
  3. **RED (`ssr`)** — `ui/src/lib/derive.test.ts` for #13, option (d):
     `an unfilled dead language renders its name_unfilled form` asserting exactly
     `Dead Language` (and `Tote Sprache` under the German ruleset), **plus the
     regression guard that matters more** —
     `an unfilled template without name_unfilled still renders the param hint`,
     asserting `Puissant (Ability)` for `virtue.puissant_ability`. That second test is
     what stops a later "simplification" back to blanket suppression; write it even
     though it passes today, and say in a comment why it exists.
  3b. **RED (Rust)** — the i18n entry gains `name_unfilled: Option<String>`: a
     deserialization test that an entry **without** the field still parses (serde
     default) and one with it round-trips. Catalogue-size invariant applies — assert on
     two known ids, never on a count.
  3c. **RED (Rust)** — #32: `crates/arm-rules/tests/data_integrity.rs`:
     `the magus minimum dead-language requirement names its exemplar`, asserting the
     `exemplar` slug is present on the three sites (`life_stages.json:4`, `:10`,
     `abilities.json:32`) and resolves to an i18n entry in **both** locales. Plus
     `an exemplar slug is not treated as a referential-integrity ref` — the loader must
     not try to resolve it as a catalogue id.
  3d. **RED (`ssr`)** — #32 in the UI: the minimums row and the
     `issue-magus_minimum_ability` message both name the exemplar, so a magus sees that
     the rules mean Latin while the check stays "any Dead Language".
  4. **RED (`ssr`)** — `ui/src/lib/components/MagusMinimumAbilities.test.ts` and the
     validation-message path: the same label appears correctly in **both** surfaces.
  5. **RED (`ssr`)** — the edit-mode surface: `VirtueFlawTab` (or wherever the editor
     mounts `ParameterPicker`) renders the new selects too. **Shared component —
     both surfaces must be covered.**
- **i18n keys** en + de. `param-label-form` already exists (`en:778`); check
  `param-label-technique` (`en:768`) and `param-label-item`. If `item` needs a label,
  add it to both locales — German from
  `rules/source/de/translation-tables/` (`Gegenstand` is the likely table term;
  **look it up, do not assume**).
  **Rules-layer text, not `.ftl`** — #13's option (d) and #32's exemplar both add
  entries to `rules/i18n/en/` **and** `rules/i18n/de/`, not to the Fluent files, because
  they are rules text rather than UI chrome: `name_unfilled` for
  `ability.dead_language` (`Dead Language` / `Tote Sprache`) and
  `ability.living_language` (`Living Language` / `Lebende Sprache`), and the exemplar
  label for `latin` (`Latin` / `Latein`). The German forms are the ones already in
  `rules/i18n/de/abilities.json:23` and `:48` — **lift them from the existing templates,
  do not re-translate.** `Latein` is ordinary German for the language; check
  `rules/source/de/translation-tables/` for a sanctioned form first anyway, per the
  standing rule.
- **Schema/migration** None. A `domain` change is a *ruleset* change, not an entity
  change: saves store the chosen parameter value, and an existing save whose stored
  value is an Art slug remains valid under `form` as long as it is a Form. **Check
  each of the five for values that were legal under `art` but illegal under `form`**
  (a Technique) and say in the commit message what happens to such a save — the engine
  will report `unknown_param_value`, which is the correct, visible outcome.
- **Acceptance criteria**
  - Every `ParameterDomain` variant except `text` renders a select; `text` renders an
    input. No branch falls through to a text input by accident.
  - No slug is ever displayed as a parameter option label, in either locale.
  - Deft (Form) can be filled by picking a localized Form name, in English and German.
  - Each of the five #5 items offers only Forms, with the change justified by a
    verified source line range recorded in `RULES.md`.
  - The magus-minimums row and the `issue-magus_minimum_ability` message read
    identically and without a doubled placeholder.
  - `Puissant (Ability)`, `Affinity with (Ability)`, `Great (Characteristic)` and
    `Ways Of The (Land)` **still render with their param hint** — option (d) is opt-in
    and must not have regressed the ~36 templates that were already correct.
  - Both surfaces name the rules' exemplar: a magus is told the minimum means **Latin**,
    in English and German, while the enforced check remains "any Dead Language ≥ N".
  - `crates/arm-rules/RULES.md` records, for #32: the verified `:2437` citation; that
    the check is a **deliberate widening** of "Latin 1"; that the widening is permanent
    because languages are troupe-defined free text and the rules publish no language
    list, so a `language.*` catalogue would require inventing rules data; and that the
    exemplar slug is a label key, **not** a referential-integrity `ref`.
- **Manual verification** In the editor, add Deft (Form), Deficient Form and
  Deficient Technique and fill each from its select; switch the UI language to German
  and confirm the options are German labels. Add Form Monstrosity and confirm only
  Forms are offered. Then look at a fresh magus's Minimum Abilities list and confirm
  the Latin/Dead Language row reads sensibly, and that the corresponding validation
  message matches it word for word.

---

### Slice 8 — The budget-bar cluster: chronological XP pools, a closing spell-levels pair, sticky bars, read-only base

- **Issues covered** #14 *(DECIDED)*, #18 *(D2 **answered**: option (a))*,
  #16 (**high**), #19 *(DECIDED)*
- **Depends on** S2 (#16's interaction with the shortened abilities step; #14's chips
  now live on the Experience step). **Blocks** nothing.
- **Why this slice exists** All four are the budget bars you edit against, and the
  review doc pairs them explicitly: #14's taller multi-line bar changes what sticky
  (#16) costs, and #18's chosen shape decides the slot #19 makes read-only.
  - **#14** (medium-high — "the pools were unreadable"): `XpBar.svelte` + the
    `xp-pool-*` keys show two chips both labelled "Later life" (`Later life: 5 × 15 =
    75 XP` the derivation, `Later life (Abilities only): 0 / 75` the restricted pool)
    in an order that made "Later life" read as *post*-Gauntlet. **Which pool is which**
    (verified): "Later life (Abilities only)" is the early-childhood-end →
    apprenticeship-start pool — source Core Rules `:2214`, *"Later Life. 15 experience
    points per year (until apprenticeship for magi)"*, implemented
    `life_stage.rs:527-550`; Gauntlet 25 − childhood 5 − apprenticeship 15 = 5 years ×
    15 = 75, matching the Darius example at `:2402`. "Abilities only" because a magus
    cannot spend pre-apprenticeship experience on Arts and the two share one pool.
    **DECIDED**: (1) chronological order per `:2213-2216` / `:2364` — Early childhood →
    Later life → Apprenticeship → After the Gauntlet; (2) **one chip per block**,
    merging each block's derivation figure with its restricted-pool spent/total so no
    label appears twice; (3) **"Later life (ages N-M)"**, keeping the rules term and
    adding the per-character span; (4) **"After the Gauntlet"** replaces "As a magus",
    tying the label to the `Gauntlet age` field that drives it (source `:2216`);
    (5) native language + the 45-point spread group under one **Early childhood**
    heading, per `:2378`. Target shape:
    ```
    XP pool 0 / [380]   Available: 380
    Early childhood — Native language 0 / 75 · Other 0 / 45
    Later life (ages 5-10): 5 × 15 = 75 XP — 0 / 75
    Apprenticeship: 15 years = 240 XP
    After the Gauntlet: 10 × 30 - 130 for lab work = 170 points, 140 XP
    ```
    **Good news, verified:** `later_life_years` is already on `LifeStageBudget`
    (`life_stage.rs:482`) **and** already mirrored in `ui/src/lib/types.ts:286` — so
    the per-character span needs no new engine surface, contrary to the review doc's
    "may need surfacing". The span's start is `childhood.years` (5,
    `rules/core/life_stages.json:19`) and its end is `start + later_life_years`.
  - **#16** (**high** — the user had to "change them blindfold"): `XpBar` is mounted by
    the step (`WizardStep.svelte:49`, `STEPS.abilities.bar`) as an ordinary flow child
    of `.vf-tab`, and the scrollport is the ancestor
    `.tab-content { overflow-y: auto }` (`app.css:337-349`, verified). Fix:
    `position: sticky; top: 0` inside that scrollport, with a background so it does not
    overlay rows transparently. **Same treatment for `BalanceBar` (V/F) and
    `SpellBudgetBar` (spells)** — all three are budgets you edit against and all three
    are mounted the same way. **Needed on the editor tabs too.** S2's split shortens the
    abilities step so the outer scroll may stop triggering *there*, but sticky is still
    the robust fix and still needed for V/F, spells and edit mode.
  - **#18** — **D2 answered: option (a).** The denominator becomes `base + lifeStage`,
    so `base=120, lifeStage=30, used=150` displays `150 / 150` with `Available: 0`, and
    the pair closes. The bracketed input keeps editing only `spell_levels_override` (the
    base) — which means **the input's value (120) and the denominator (150) are now
    different numbers, so the input cannot stay in the denominator slot.** Move it out,
    or render the denominator separately from it; a layout showing `[120]` where `150`
    belongs would reintroduce #18 in a new form. Say which you chose in the commit
    message. Separately, the comment at `SpellBudgetBar.svelte:65-67` claiming the
    figures *"always close against the bracketed base"* is **false whenever
    `lifeStage != 0`** and must be corrected; `derive.ts:932-933` is the honest version
    and should be the one the comment points at.
  - **#19** (DECIDED): the spell-levels base is **read-only when mounted by the
    wizard, editable when mounted by the editor**. Implement by passing an explicit
    prop (e.g. `readonlyBase`) via `WizardStep.svelte`'s **existing `barProps` seam**
    (`WizardStep.svelte:30-31`, `:50` already uses it for `XpBar`'s prefix). **Do not
    sniff the flow from the store** — that is cross-cutting theme 3. Rationale: the 120
    is a fixed rules grant (`:2215`), and every legitimate in-rules variation already
    arrives elsewhere (Skilled/Weak Parens as the `bonus` chip, post-Gauntlet levels as
    the `lifeStage` chip); precedent in the same component is the post-Gauntlet input
    being deliberately absent because *"the Abilities step owns the choice"* (`:23-28`).
    **Consequence to document**: this is the first behavioral divergence between the
    two mounts, so the reuse-rule comment at `WizardStep.svelte:38-43` — *"the wizard
    reuses the direct-entry components as they are, so its steps and the editor's tabs
    can never drift apart"* — must be updated, or the prop will read as a violation.
- **Files to touch** `ui/src/lib/components/XpBar.svelte`;
  `ui/src/lib/components/SpellBudgetBar.svelte:55-99`;
  `ui/src/lib/components/BalanceBar.svelte`;
  `ui/src/lib/components/WizardStep.svelte:30-31`, `:38-43`, `:44-57`, `:86-88`;
  `ui/src/lib/derive.ts:915-961` (`spellLevelAllocation`, incl. the corrected doc
  comment); `ui/src/app.css` (sticky rules; the `.xp-summary` / bar classes);
  `locales/en/main.ftl` + `locales/de/main.ftl` (`xp-pool-*`,
  `spell-levels-post-gauntlet` at `en:387` / `de:390`, `life-stage-post-gauntlet` at
  `de:267`).
- **TDD steps**
  1. **RED (`ssr`)** — `ui/src/lib/components/XpBar.test.ts`:
     `renders the life-stage blocks in chronological order`,
     `renders exactly one chip per life-stage block`,
     `labels later life with the character's own age span`. The order test should read
     the rendered chip labels in document order and compare to the expected sequence.
  2. **RED (`ssr`)** — `ui/src/lib/i18n.test.ts`: the renamed/added `xp-pool-*` keys
     exist in both locales and the retired ones are gone.
  3. **RED (`ssr`)** — `ui/src/lib/derive.test.ts`:
     `the displayed spell-levels pair closes when post-Gauntlet levels are funded` —
     assert numerator `150` and denominator `150` for
     `base=120, lifeStage=30, used=150`, and that `available` is `0`. Add
     `the denominator equals the base when no post-Gauntlet levels are funded`
     (`lifeStage=0` → `120 / 120`) so option (a) is pinned in both directions.
  4. **RED (`ssr`)** — `ui/src/lib/components/SpellBudgetBar.test.ts`:
     `renders the base read-only when readonlyBase is set` and
     `renders the base editable by default` (the editor mount).
  5. **RED (`ssr`)** — `ui/src/lib/components/WizardStep.test.ts`:
     `passes readonlyBase to the spells bar`.
  6. **RED (CSS contract, `ssr`)** — extend the stylesheet test: the three bar classes
     declare `position: sticky` with a non-transparent background. A `client` test
     could observe computed style, but happy-dom does not do layout, so the
     stylesheet-text contract is the honest guard; sticky behaviour itself is verified
     manually and by e2e scroll assertions if practical.
  7. Green.
- **i18n keys** en + de, several. **Renamed/merged** per #14: the `xp-pool-*` family
  becomes one key per block (early childhood with its two sub-figures, later life with
  its span, apprenticeship, after the Gauntlet). **Renamed**: `As a magus` → `After the
  Gauntlet`. **Do not work from an enumerated key list — grep both `.ftl` files for
  `As a magus` / `Als Magus` and decide each hit on its merits.** The full picture,
  verified:
  - **Rename these two — they are the label #14 replaces.**
    `life-stage-post-gauntlet` (`en:254` *"As a magus: …"*, `de:267` *"Als Magus: …"*)
    and `spell-levels-post-gauntlet` (`en:387`, `de:390`). The review doc names only
    the second.
  - **Leave these alone — "years as a magus" is descriptive prose, not the label**, and
    reads correctly: `life-stage-gauntlet-age-hint` (`en:272`/`de:287`),
    `life-stage-lab-seasons-hint` (`en:274`/`de:289`),
    `life-stage-post-gauntlet-summary` (`en:277`/`de:292`),
    `issue-life_stage_gauntlet_age_after_age` (`en:852`/`de:859`),
    `issue-life_stage_lab_seasons_out_of_range` (`en:853`/`de:860`),
    `issue-life_stage_spell_level_split_exceeds_points` (`en:854`/`de:861`), plus the
    comments at `en:248`, `en:262`, `de:261`, `de:392`. Renaming these would produce
    contorted German and English for no gain — #14's decision is about the *chip
    label*, which is what tied the term to the `Gauntlet age` field.
  **German terms — verified provenance, and a correction to the review doc:** "Later
  Life" is **not** in `rules/source/de/translation-tables/`. Its German form comes from
  the German rulebook at the mirrored line: `Ars Magica Definitive Edition
  Basisregeln.md:2214` and the heading at `:2390` give **"Späteres Leben"**, which
  `locales/de/main.ftl:239` already uses — reuse it, do not re-translate. "Gauntlet" **is**
  in the tables: `rules/source/de/translation-tables/grundbegriffe.md:57` gives
  **"Lehrlingsprüfung"**, already used at `locales/de/main.ftl:286`. "Apprentice" →
  "Lehrling" (`grundbegriffe.md:15`). "Early childhood" is already
  `Frühe Kindheit` (`de:238`). So **every** German term this slice needs already exists
  in the file or in a cited source — nothing may be invented.
- **Schema/migration** none.
- **Provenance** #14's chip layout is a presentation change over existing engine
  numbers, but the *labels* now assert a chronology, so cite Core Rules
  `:2213-2216` (the ordered creation summary), `:2364`/`:2378` (the childhood
  grouping), `:2402` (the Darius worked example that validates the 5 × 15 = 75) and
  `:2216` (the post-Gauntlet label) at the implementation site, and add/extend the
  `RULES.md` entries for the life-stage blocks accordingly. All four ranges were
  verified while writing this plan except `:2364`/`:2378`/`:2402` — **re-verify those
  three before committing.**
- **Acceptance criteria**
  - No label appears twice in the XP bar; blocks read chronologically; later life shows
    the character's own age span.
  - The spell-levels pair closes for a magus with post-Gauntlet-funded levels.
  - All three bars stay visible while their step's content scrolls, in the wizard **and**
    in the editor.
  - The spell-levels base is read-only in the wizard and editable in the editor, driven
    by an explicit prop and not by a store lookup.
  - `WizardStep.svelte:38-43`'s reuse-rule comment reflects the new divergence.
- **Manual verification** Magus, life-stage funding, Gauntlet age 25, age 35. On the
  Experience step read the XP bar top to bottom and confirm it reads as a chronology
  with no repeated label and the correct age span. On the Abilities step, add abilities
  until the list scrolls and confirm the XP bar stays pinned with a readable
  background; do the same on V/F and Spells, and on the corresponding editor tabs. On
  the wizard's Spells step confirm the base is not editable and that the displayed pair
  closes; on the editor's Spells tab confirm it is editable.

---

### Slice 9 — Elements that enter or leave the flow, and one mount that centers differently

- **Issues covered** #2 (medium-high, causes mis-clicks), #17, #27
- **Depends on** S1. **Blocks** nothing.
- **Why this slice exists** Cross-cutting theme 1 — *an element enters or leaves the
  flow on first interaction, moving a control the user is actively clicking* — plus the
  one remaining mount divergence. **Rule to adopt and apply here: reserve the space
  (toggle visibility) or take the element out of flow. Never unmount something above an
  input surface.**
  - **#2**: `WizardShell.svelte:78-82` conditionally *mounts* the hint `<p>` via
    `{#if store.wizardPhaseIncomplete}`. The flag is the engine's
    `completeness.incomplete_phases` (`derive.ts:1085`, verified), which flips on the
    very **first recorded value** — so the first `+` click is guaranteed to collapse
    the paragraph and move everything below it. Reproduced on step 3 (characteristics,
    worst — repeated stepper clicks), step 7 (arts) and step 8 (spells, least harmful).
    General case: any step whose first edit flips completeness.
  - **#17**: `.spell-mastery-block` (`app.css:992-999`) is a `flex-direction: column`
    with `flex: 0 1 auto`, so its width is its widest child. At mastery 1 the
    `.mastery-abilities` row appears (`SpellTab.svelte:480`) carrying a label plus the
    "Add special ability" `<select>` — much wider. The block widens, `.item-name`
    (which takes the row's free width, `app.css:982`) gives ground, and the spinner is
    **repositioned horizontally** on a repeated-click control. Fix: give
    `.mastery-abilities` its own full-width wrap line (`flex-basis: 100%`) so it sits
    below without widening the block. **A `min-width` on the block would permanently
    indent every unmastered row** — do not do that.
  - **#27**: `.tab-content` is `display: flex; flex-direction: column; align-items:
    center` (`app.css:351-353`, verified). The editor mounts `CharacteristicPicker` as
    a **direct child** (`App.svelte:306-307`), so its `<section class="panel
    char-panel">` is centered; the wizard always wraps the step body in
    `<div class="vf-tab">` (`WizardStep.svelte:78`), which is `width: 100%` and
    stretches its children. Fix: make the panel center **itself** (`margin-inline:
    auto` plus its existing max-width) so both mounts agree by construction.
    Conditionally dropping the wrapper also works but is more fragile — the wrapper
    reproduces the editor's height chain (`WizardStep.svelte:73-77`), without which
    `.region-row`'s Available/Selected columns collapse, and it also supplies the
    `gap: 1rem` between guidance, bar and body.
- **Files to touch** `ui/src/lib/components/WizardShell.svelte:78-82`;
  `ui/src/app.css` (a visibility-toggle utility for #2; `:982`, `:992-999`,
  `:1001-1007` for #17; the `.char-panel` centering for #27);
  `ui/src/lib/components/SpellTab.svelte:429-530`.
- **TDD steps**
  1. **RED (`ssr`)** — `ui/src/lib/components/WizardShell.test.ts`:
     `keeps the incomplete hint in the flow when the phase becomes complete` —
     assert the element is **present in both states**, differing only by a
     visibility/`aria-hidden` attribute or class. This is the assertion that fails
     today because the element is unmounted.
  2. **RED (`ssr`)** — `ui/src/lib/components/SpellTab.test.ts`:
     `the mastery abilities row is a full-width wrap line`, asserting the class/style
     hook rather than a measured width (SSR does no layout).
  3. **RED (CSS contract, `ssr`)** — extend the stylesheet test: `.mastery-abilities`
     declares `flex-basis: 100%` and `.spell-mastery-block` declares **no**
     `min-width`; `.char-panel` declares `margin-inline: auto`.
  4. **RED (`ssr`)** — a two-mount test for #27: the characteristics panel carries the
     self-centering class in the wizard mount **and** in the editor mount. **Shared
     component — both surfaces.**
  5. Green.
- **i18n keys** none — #2 keeps `wizard-step-incomplete-hint` (`en:66`), it only stops
  unmounting. If the hidden state needs an accessible-hidden treatment, use
  `aria-hidden` / the existing `.sr-only` utility (`app.css:1449`), not a new string.
- **Schema/migration** none.
- **Acceptance criteria**
  - Nothing above an input surface unmounts on first interaction anywhere in the
    wizard; the incomplete hint's space is reserved.
  - The mastery spinner does not move horizontally when mastery goes 0 → 1, and
    unmastered rows are not indented.
  - The characteristics panel is positioned identically in the wizard and the editor.
- **Manual verification** Wizard step 3 (characteristics): click a `+` and watch that
  nothing below moves — then click `+` five times in a row without repositioning the
  mouse and confirm every click lands on the same control. Repeat on Arts and Spells.
  On the Spells tab, raise one spell's mastery 0 → 1 and confirm the spinner stays put
  while the abilities row appears below. Compare the characteristics panel side by side
  between the wizard step and the editor tab.

---

### Slice 10 — Lists and severity: separators, granted V/F placement, severity typography, rail noise

- **Issues covered** #6, #8, #9 *(DECIDED)*, #10
- **Depends on** S1. **Blocks** nothing.
- **Why this slice exists** Four list/severity presentation bugs in the same two
  files, one of which (#9) fixes another (#8) for free in the V/F case.
  - **#6**: `<li class="issue {issue.severity}">` (`ValidationPanel.svelte:50`) emits
    the bare class `error`, so the standalone `.error { color: var(--error);
    font-size: 0.85rem }` rule (`app.css:1577-1580`, written for banner text like
    `StartScreen.svelte:34`) **also matches the row**. There is no `.warning`
    counterpart, and `.issue` (`:1522-1527`) never sets `font-size`/`color`, so
    `.error` wins **by default, not by specificity** — error rows render smaller and
    red-tinted while warnings do not. Fix: set font-size and colour explicitly on
    `.issue` so neither severity inherits from elsewhere, keeping the colour coding in
    the left border + background tint + uppercase badge as designed
    (`:1529-1555`). **Also scope or rename the global `.error` rule** — a bare
    one-word class carrying typography will keep colliding.
  - **#8**: `.selection-list li { border-bottom }` + `.selection-list li:last-child {
    border-bottom: none }` (`app.css:703-712`, verified). `:last-child` is **scoped per
    `<ul>`**, and `SelectionList.svelte:79` opens a new `<ul>` per group. Headed groups
    hide it (the next `<h3 class="category">` draws its own boundary); a **header-less**
    group does not. Affected: the V/F granted group (fixed for free by #9), plus
    `SpellTab.svelte:82` and `EquipmentTab.svelte:76`, which both emit conditionally
    header-less groups and keep the bug. `AbilityTab.svelte:148` always sets a header —
    unaffected. Fix: draw the separator as a `border-top` on rows after the first, or
    on the `<ul>` boundary, instead of per-list `:last-child`.
  - **#9** (DECIDED, actively misleading): `VirtueFlawTab.svelte:153-165` (verified)
    pushes granted V/F into a header-less group, so a `Hermetic`-badged granted Virtue
    (Faerie Magic) appeared beneath the **SUPERNATURAL** heading — any granted V/F
    inherits whatever category happened to sort last. **DECIDED: merge granted V/F into
    the normal category-grouped list, ordered like any other row. No separate group.**
    Grouping must key off each granted item's **own** category. Keep the row union
    (`{kind:'bought'|'granted'}`) — granted rows have no entity index and no remove
    button, so the "Granted" marker at `:340-343` stays the only distinction, matching
    how the in-list `Required` rows already behave. Needs a combined grouping path:
    `groupSelectionsByCategory` currently takes only bought selections, and the
    within-group localized-name sort must cover granted rows. **Watch the key
    collision the existing comment at `:158-162` documents**: the engine concatenates
    House, Mythic Companion, `grants_selection` and warping grants **without dedup**
    (`effective.rs` `entity_grants`), so one `ref` can be granted twice and a duplicate
    `{#each}` key throws `each_key_duplicate` — in production too — aborting the whole
    tab's render. The merged path must keep an index in the key.
  - **#10** (requested change): drop the "not started" text from the rail step names
    visually. `WizardShell.svelte:55-59`, `wizard-step-incomplete-label`
    (`en:64`), `app.css:477-482`. **Keep the span for assistive tech** via the existing
    `.sr-only` utility (`app.css:1449`): it sits *inside* the button deliberately so
    the marker is part of its accessible name (`:52-54`), and deleting it outright
    would leave `data-incomplete` as a style hook only, risking a colour-only channel
    (WCAG 1.4.1). Note the flag is not "unopened" — it is
    `completeness.incomplete_phases`, so on a fresh character **every** step carries it
    at once. This does not resolve the on-step hint (#2, Slice 9) or the Review step's
    `wizard-review-incomplete` list; both stay.
- **Files to touch** `ui/src/app.css:703-712`, `:1522-1537`, `:1577-1580`, `:477-482`;
  `ui/src/lib/components/ValidationPanel.svelte:50`;
  `ui/src/lib/components/VirtueFlawTab.svelte:120-170`, `:340-343`;
  `ui/src/lib/components/SelectionList.svelte:79`;
  `ui/src/lib/components/WizardShell.svelte:52-59`;
  `ui/src/lib/derive.ts` (`groupSelectionsByCategory`);
  `ui/src/lib/components/StartScreen.svelte:34` (if the global `.error` rule is
  renamed).
- **TDD steps**
  1. **RED (`ssr`)** — `ui/src/lib/derive.test.ts`:
     `groupSelectionsByCategory places a granted selection in its own category`, and
     `sorts granted and bought rows together by localized name`. Pure helper — still
     test-first.
  2. **RED (`ssr`)** — `ui/src/lib/components/VirtueFlawTab.test.ts` (create if
     absent): `renders no separate granted group`,
     `renders a granted Hermetic virtue under the Hermetic heading`,
     `keys granted rows uniquely when the same ref is granted twice` (the
     `each_key_duplicate` regression guard).
  3. **RED (CSS contract, `ssr`)** — `.issue` declares its own `font-size` and
     `color`; the bare `.error` typography rule is scoped or renamed.
  4. **RED (`ssr`)** — `ui/src/lib/components/ValidationPanel.test.ts`:
     `error and warning rows carry the same typographic classes`.
  5. **RED (`ssr`)** — `ui/src/lib/components/WizardShell.test.ts`:
     `the incomplete marker is present but visually hidden` — assert the `.sr-only`
     class (or equivalent) is on the span and the span still exists.
  6. **RED (`ssr`)** — `SpellTab.test.ts` / `EquipmentTab.test.ts`: a header-less
     group's last row is separated from the next group.
  7. Green.
- **i18n keys** none added. `wizard-step-incomplete-label` (`en:64`, de mirror) is
  **kept** — it is now screen-reader-only, not deleted.
- **Schema/migration** none.
- **Acceptance criteria**
  - Granted V/F appear under their **own** category heading, inline with bought rows,
    with the "Granted" marker as their only distinction and no separate group.
  - No `each_key_duplicate` under a doubly-granted ref (covered by a test).
  - Error and warning rows are the same size and colour; severity is carried by border,
    tint and badge only.
  - Header-less groups in Spells and Equipment show a separator at their boundary.
  - The rail shows no "not started" text but a screen reader still announces it.
- **Manual verification** Build a magus with a House that grants a Virtue (e.g. one
  granting Faerie Magic) and confirm it appears under Hermetic, sorted with the bought
  rows. Force one error and one warning at once and compare the two rows' size and
  colour. Look at the Spells and Equipment tabs with a conditionally header-less group
  present. Check the rail is clean, then verify with a screen reader (or by inspecting
  the accessible name) that the marker is still announced.

---

### Slice 11 — Guidance and findings: per-type V/F advice, collapsed magus minimums, unspent-budget warnings

- **Issues covered** #7, #12 *(DECIDED)*, #30
- **Depends on** S2 (guidance keys and the phase-ownership map changed there;
  `MagusMinimumAbilities` now lives on the post-split `abilities` step). **Blocks**
  nothing.
- **Why this slice exists** All three are about what the guided flow *tells* the user,
  and all three draw their numbers from data already in the ruleset or the engine.
  - **#7**: `wizard-guidance-virtues_flaws` (`en:82`, verified) states only the point
    budget; `derive.ts:1154-1160` passes only `virtues`/`flaws`. Guided mode's purpose
    is to say what the rules advise. **Nothing new to author in the rules layer** — the
    profile already carries these as `flaw_category_caps` with a `hard` flag separating
    *may not* from *should not* (verified: `character_types.json:10-14` companion,
    `:44-48` grog, `:75-82` magus incl. `virtue_category_caps`, `:117-121` mythic
    companion), and the magus Hermetic-Flaw guideline is already the
    `missing_hermetic_flaw` warning (`RULES.md:227`, `:1467`). **Generate the sentence
    from the profile** the way the budget numbers already are. Verified source
    (`:2816-2862`): grog — no Story Flaws, not more than one Personality Flaw;
    companion — ≤1 Story, ≤2 Personality; mythic companion — as companion; magus —
    **should take at least one Hermetic Flaw**, ≤1 Story, ≤2 Personality; plus the
    general `:2818` (≤1 Story Flaw) and `:2820` (≤2 Personality, ≤1 Major
    Personality). **Correction the review doc already records and this plan repeats
    because it is easy to get backwards:** there is **no** "at least one Story Flaw"
    rule. Story Flaws have a recommended *ceiling* of one and no minimum; the only "at
    least one" in the rules is the magus's **Hermetic** Flaw.
  - **#12** (DECIDED): `MagusMinimumAbilities.svelte` duplicates the Validation panel —
    its own comment (`:15-18`, verified) confirms the rows come from the same engine
    findings. **DECIDED: collapse to the `magus-minimums-summary` line ("N of M still
    unmet"), expandable on demand.** Validation stays the authoritative surface.
    **Also fix while in there**: the summary counts **all** rows
    (`:23` `rows.filter(...)`, `:66` `total: String(rows.length)`) but renders under the
    *Minimum Abilities* heading above only the `required` ones (`:68-74`) — so it reads
    "7 of 7" for a list of 3. Verified: `rules/core/life_stages.json:3-13` declares 3
    minimum + 4 recommended abilities.
  - **#30**: verified against the finding-code contract table
    (`crates/arm-rules/src/validation/mod.rs:126-176`) — overspending is an error on
    both budgets (`not_enough_xp` `:139`, `over_spell_levels` `:176`) but underspending
    is warned only for characteristics (`characteristic_points_unspent` `:136`) and for
    life-stage **restricted** blocks (`restricted_xp_unspent` `:141`). Leaving 200
    general XP or 60 spell levels unspent produces **silence** while one unspent
    characteristic point produces a warning. Fix: **two new warning codes**, owned by
    the `abilities` and `spells` phases. Because `ValidationPanel` renders unfiltered on
    the terminal step (`WizardShell.svelte:91`), phase-owned warnings appear **both** on
    their own step and on Review — which is desirable and satisfies the "in the review
    step" request and more.
    **Design details, all load-bearing:** count only the **general** remainder, not the
    restricted blocks (those already have `restricted_xp_unspent`, and summing them
    again double-reports the same points on a life-stage character);
    **wording must stay factual** — `restricted_xp_unspent` says "will be wasted",
    which is backed for childhood, but the Core Rules contain **no** equivalent
    statement for apprenticeship XP or the 120 spell levels (searched during the
    review), so these messages must say "N of M unspent" and stop there rather than
    asserting a rule the source does not make; they are **warnings**, so `canFinish`
    (errors only, `wizard-navigation.svelte.ts:90-92`) is unaffected; they will fire
    from the start on a fresh character exactly as `characteristic_points_unspent`
    already does — consistent, but it adds to the noise #10 addresses, which is why #10
    lands first.
- **Files to touch**
  - `ui/src/lib/derive.ts:1149-1169` (`GUIDANCE_ARGS.virtues_flaws` gains the cap
    figures) and the guidance-sentence builder.
  - `locales/en/main.ftl:82` + `locales/de/main.ftl` mirror —
    `wizard-guidance-virtues_flaws` gains the per-type advice, driven by args so no
    rules number is frozen into a translated string.
  - `ui/src/lib/components/MagusMinimumAbilities.svelte:20-28`, `:59-80` — collapse to
    the summary with a disclosure; fix the summary's denominator to match what it
    heads.
  - `crates/arm-rules/src/validation/mod.rs` — two new `CODE_*` constants (alongside
    `:325` `CODE_NOT_ENOUGH_XP`, `:361` `CODE_RESTRICTED_XP_UNSPENT`, `:494`
    `CODE_OVER_SPELL_LEVELS`) and two new rows in the contract table at `:126-176`.
  - `crates/arm-rules/src/validation/scores.rs` (or wherever `not_enough_xp` is
    emitted) and the spells validator — the emitters.
  - `locales/en/main.ftl` + `locales/de/main.ftl` — `issue-<newcode>` × 2.
  - `crates/arm-rules/RULES.md` — the two new codes; and note explicitly that they
    assert **no** rule about waste, because the source makes none.
- **TDD steps**
  1. **RED (Rust)** — the two new warnings:
     `warns_when_general_xp_is_left_unspent`,
     `does_not_double_report_restricted_blocks_as_general_unspent`,
     `warns_when_spell_levels_are_left_unspent`,
     `unspent_warnings_do_not_block_finishing` (severity is warning). Put them beside
     the existing `not_enough_xp` tests in `validation/mod.rs` (around `:4590-4840`).
  2. **RED (Rust)** — the contract-table/consistency test that every emitted code has
     a table row (if one exists; if not, this slice is the place to add it — it is the
     cheapest possible guard for a table that is documentation-by-convention).
  3. **RED (`ssr`)** — `ui/src/lib/i18n.test.ts`: both new `issue-<code>` keys exist
     in both locales.
  4. **RED (`ssr`)** — `ui/src/lib/derive.test.ts`: the new codes route to the
     `abilities` and `spells` phases respectively via `issuesForPhase`.
  5. **RED (`ssr`)** — `ui/src/lib/components/MagusMinimumAbilities.test.ts`:
     `renders only the summary line by default`,
     `expands to the full checklist on demand`,
     `the summary counts only the rows it heads`. The third fails today with the "7 of
     7 for a list of 3" bug. If the disclosure is implemented with an `$effect` or
     focus management, add a **`client`** test for that part and say why.
  6. **RED (`ssr`)** — `ui/src/lib/derive.test.ts` for #7:
     `the virtues and flaws guidance states the type's Story and Personality Flaw
     caps` and `states the magus Hermetic Flaw recommendation`, asserting the args
     built from the profile rather than the rendered sentence where possible.
  7. Green.
- **i18n keys** en + de. **Changed**: `wizard-guidance-virtues_flaws` (gains args and
  clauses). **Added**: two `issue-<code>` messages; a disclosure label for #12
  (e.g. `magus-minimums-expand` / `magus-minimums-collapse`). German: "Story Flaw" and
  "Personality Flaw" are established V/F category terms — take them from
  `rules/source/de/translation-tables/tugenden-fehler.md` and from the existing
  `category-*` keys in `locales/de/main.ftl`; the existing
  `issue-missing_hermetic_flaw` string is the model for the Hermetic-Flaw clause.
  **Do not invent**, and remember `n/a` → `n/v` and the standalone-label
  uninflected-adjective convention.
- **Schema/migration** none — new findings are computed, never stored.
- **Provenance** #7's sentence and #30's two codes both need source citations at the
  implementation site with **verified** ranges. Verified while writing this plan:
  `:2816` (one Social Status), `:2818` (Story Flaw ceiling), `:2820` (Personality Flaw
  ceiling and Major cap), `:2822-2831` grog, `:2832-2840` companion, `:2842-2851`
  mythic companion, `:2853-2862` magus incl. `:2860` "should take at least one Hermetic
  Flaw". For #30, cite the *budget* sources (`:2215` for the 240 XP / 120 levels,
  `:2216` for the post-Gauntlet points) and state in `RULES.md` that the **absence** of
  a waste rule is why the message is purely factual.
- **Acceptance criteria**
  - Each character type's V/F guidance states its own Story and Personality Flaw
    guidance, generated from `flaw_category_caps`, with no rules number hardcoded in a
    locale string; the magus's also states the Hermetic-Flaw recommendation.
  - No guidance string claims a Story Flaw minimum.
  - The magus minimums surface is one summary line by default, expandable, and its
    count matches the rows it heads.
  - Leaving general XP or spell levels unspent produces a **warning** on the owning
    step and on Review; overspending still produces an error; Finish is unaffected.
  - The unspent warning does not double-count restricted life-stage blocks.
- **Manual verification** For each of the four types, read the V/F step's guidance and
  check it against Core Rules `:2822-2862`. As a magus, leave 200 XP and 60 spell
  levels unspent and confirm one warning each appears on Abilities, on Spells and on
  Review, that Finish is still available, and that a life-stage-funded magus does not
  see the same points reported twice. Expand and collapse the minimums summary.

---

### Slice 12 — Age's canonical home, and the current-year editing aid

- **Issues covered** #24 *(DECIDED)*, #25 *(DECIDED, and D3's three sub-questions
  **answered** — see §2; note D3.1 and D3.3 changed from the original recommendations
  because saga management is a planned future feature)*
- **Depends on** S2 (the `age-cap-note`'s home depends on the split) and S3 (the
  editor's Details tab is settled there). **Blocks** nothing.
- **Why this slice exists** Both are about where the one age value is edited from.
  - **#24** (DECIDED) — verified facts: the Concept step mounts `IdentityFields`, which
    has **birth year** but deliberately no age (`IdentityFields.svelte:12`). The
    **editor** mounts `IdentityFields` at `CharacterDetails.svelte:85` and `AgeFields`
    immediately after at `:87` — so in edit mode age already sits beside identity. The
    life-stage panel has a *separate* input (`LifeStagePanel.svelte:117-127`,
    `life-stage-age-input`, verified at `:126`), rendered only under life-stage
    funding. The aging step has `AgeFields` (`age-input`), added because *"under flat
    (pool) funding there is no age field anywhere in the wizard"*
    (`AgeFields.svelte:14-19`). All three write the one `entity.age` — duplicated
    *surface*, never duplicated data.
    **DECIDED: Concept owns it.** Mount `AgeFields` on the Concept step beside
    `IdentityFields`, mirroring `CharacterDetails.svelte:85-87`. The Experience step
    (post-S2) shows the age **read-only** next to the still-editable Gauntlet age
    (*"a magus carries two ages, not one"*, `LifeStagePanel.svelte:19-20`). The aging
    step **drops** its copy.
    **Notes that are requirements:** removing the aging step's copy is only safe
    because Concept now supplies one under **both** funding models — so add an explicit
    test that a **pool-funded** character has exactly one age input. `AgeFields` needs
    a read-only mode via the **same explicit-prop seam as #19**, not a store sniff. The
    `age-cap-note` ("Max Ability score: 6") renders in **both** `AgeFields.svelte:33-34`
    and `LifeStagePanel.svelte:130-133` (verified) — **decide its home once**, and it is
    most useful beside the ability lists, i.e. the post-S2 `abilities` step. Update
    tests asserting `age-input` on the aging step; `LifeStagePanel.test.ts:322` stays
    valid. **The editor is unaffected** — it already has this layout.
  - **#25** (DECIDED core) — today the relationship runs one way only: the engine
    computes calendar year as `birth_year + age` (`crates/arm-rules/src/aging.rs:148`,
    `:158-165`), and there is **no** current-year or saga-start field anywhere in the
    entity or ruleset. DECIDED: an **app-level current-year setting, default 1220**
    (source: Core Rules `:597`, verified; also `:364`, `:440`), with age ↔ birth year as
    two views of one fact on the Concept step — edit either, the other follows.
    **Key simplification, and why this slice carries no schema risk:** `entity.age` and
    `entity.birth_year` are *both* already stored, so the setting is a pure **editing
    aid**, not new entity state. **No `SCHEMA_VERSION` bump, no migration, no
    save-portability problem.** A character built in a 1220 saga and opened under a 1230
    setting keeps both stored values; the setting only governs what the derivation fills
    in while typing. The approximation is already baked in and consistent: `birth_year +
    age` ignores birthdays within the year, exactly as `saga_year − birth_year` would.
    **D3's three sub-questions are answered (§2), and two of the answers changed
    because the user intends to build saga management later:**
    - **It is a persisted *saga year*, not a session-scoped UI preference.** Stored in a
      small app-settings file **owned by `arm-app`** — new command, new IO surface.
      **Engine purity is absolute: no IO in `arm-rules`.** Not on the entity (it is
      saga-scoped and shared across characters, so per-character copies would disagree)
      and not in the ruleset (it is a saga fact, not a rule).
    - **A current year before the birth year clamps the derived age to 0 and emits an
      advisory finding** — a new code, so `issue-<code>` in both locales plus a
      `RULES.md` entry.
    - **The saga year recomputes nothing.** `age` and `birth_year` are the stored pair
      and are continuously linked (edit either, the other follows, using the saga year
      as reference); both dirty the document. Editing the **saga year alone leaves both
      stored values untouched and does not dirty the document** — it only governs what
      is derived the next time the user types in age or birth year.
      **Why:** #25's DECIDED text already settled it (*"a character built in a 1220 saga
      and opened under a 1230 setting keeps both stored values"*), and a silent
      recompute would fabricate ages that skipped their aging rolls — advancing a
      character by N years requires aging rolls, Living Conditions and any Longevity
      Ritual applied per year. Saga *progression* is therefore a separate explicit
      action and is the aging-for-loaded-characters backlog item, **not** part of this
      slice.
- **Files to touch** `ui/src/lib/components/IdentityFields.svelte:12`;
  `ui/src/lib/components/AgeFields.svelte:14-19`, `:33-34`;
  `ui/src/lib/components/LifeStagePanel.svelte:117-133`;
  the new `ExperienceStep.svelte` (from S2);
  `ui/src/lib/components/AgingStep.svelte`;
  `ui/src/lib/components/WizardStep.svelte:44-57` (`concept` step composition — it is
  currently `{ component: IdentityFields }`, so Concept needs a composite step
  component the way `PersonalityReputationsStep` is one);
  `ui/src/lib/state.svelte.ts` (the saga-year setting and the age ↔ birth-year
  derivation); `ui/src/lib/components/AgeFields.test.ts`,
  `LifeStagePanel.test.ts:322`, `AgingStep.test.ts`.
  **Plus the persistence surface (new, from D3.1)**: `crates/arm-app/src/commands.rs`
  (read/write the saga year) and `crates/arm-app/src/main.rs` (command registration),
  with the settings file resolved the same way the app already resolves its own paths —
  **check how `pick_rules_dir` handles the portable layout and follow that precedent**,
  because a settings path that works in `target/release` and fails in a portable build
  is exactly the class of bug `project_portable_resource_resolution` records. A missing
  or unreadable settings file must fall back to the 1220 default silently, never fail
  the launch. `crates/arm-rules/` gets **nothing** — engine purity.
- **TDD steps**
  1. **RED (`ssr`)** — `ui/src/lib/components/WizardStep.test.ts` (or a new
     `ConceptStep.test.ts`): `the concept step renders both identity fields and the age
     field`.
  2. **RED (`ssr`)** — the explicit one-input test #24 demands:
     `a pool-funded character has exactly one age input in the wizard` — count
     `age-input` + `life-stage-age-input` across the rendered steps and assert 1. Then
     the life-stage counterpart: `a life-stage-funded character has one editable age and
     one read-only display`.
  3. **RED (`ssr`)** — `AgeFields.test.ts`: `renders read-only when the readonly prop
     is set`, driven by a prop, not the store.
  4. **RED (`ssr`)** — `the age cap note renders exactly once` (in its decided home).
  5. **RED (`ssr`)** — `ui/src/lib/state.svelte.test.ts` for #25, per D3:
     `setting the birth year derives the age from the saga year`,
     `setting the age derives the birth year`,
     `a saga year before the birth year clamps the age to zero and warns` (D3.2),
     and — **the test that pins D3.3, and the one most likely to be got wrong** —
     `changing the saga year leaves age and birth year untouched` plus
     `changing the saga year does not dirty the document`, against
     `editing the age dirties the document`.
  5b. **RED (Rust)** — `crates/arm-app`: the saga-year settings command round-trips a
     value, and **a missing or unreadable settings file yields the 1220 default rather
     than an error**. Keep `arm-rules` out of it entirely — if a test needs to live in
     `arm-rules` to pass, the persistence has leaked into the engine.
  6. **RED (`client`)** — required if the saga-year setting is loaded through an
     `$effect` (likely, since it arrives over IPC). SSR never runs an `$effect` body, so
     an `ssr` assertion after it would report green with the effect never firing. State
     the reason in the test file's header.
  7. Green.
- **i18n keys** en + de: a label and hint for the **saga year** setting, and **one
  `issue-<code>`** for D3.2's clamp advisory (it is now decided, not conditional).
  German: "year" is ordinary language, but check
  `rules/source/de/translation-tables/grundbegriffe.md` for a sanctioned saga/year term
  before writing one, and reuse the existing `birth-year-*` and `age-*` key wording in
  `locales/de/main.ftl` for consistency. Name the key for what it is —
  `saga-year-*`, not `current-year-*` — so the future saga-management feature inherits
  the vocabulary instead of renaming it.
- **Schema/migration** **No `SCHEMA_VERSION` change** — this is #25's key
  simplification, and it survives D3.1's decision to persist. The saga year goes in an
  *app settings* file owned by `arm-app`, which is **not** the entity schema: no bump, no
  migration, no save-portability question. Engine purity applies — the persistence lives
  in `arm-app`, never in `arm-rules`. **If this slice finds itself wanting a bump, stop**:
  the value is saga-scoped, not character-scoped, and putting it on the entity is the
  wrong answer (see §6).
- **Provenance** The 1220 default is a rules fact: cite
  `Ars Magica - Definitive Edition (Core Rules).md:597` (verified: *"That domination
  persists until the present day, 1220"*), with `:364` and `:440` as corroboration, at
  the implementation site, and add a `RULES.md` entry recording that the default year is
  a rules value and the setting itself is an editing aid with no mechanical effect.
- **Acceptance criteria**
  - Exactly one editable age input exists in the wizard for every character type and
    both funding models; the Experience step shows the age read-only beside the
    editable Gauntlet age.
  - The `age-cap-note` renders in exactly one place.
  - Editing birth year updates age and vice versa, with no underflow for an impossible
    pair — the age clamps to 0 and an advisory finding says why.
  - **Changing the saga year changes no stored value and does not dirty the document**;
    it only affects what is derived on the next age / birth-year edit.
  - The saga year survives quitting and relaunching the app, and a missing or unreadable
    settings file falls back to 1220 without failing the launch.
  - No `SCHEMA_VERSION` change; a save written before this slice loads identically.
  - `arm-rules` gained no dependency on the filesystem.
  - The editor's Details tab is unchanged.
- **Manual verification** Wizard, grog (pool funding): Concept step has an age field;
  the aging step no longer has one; changing the age on Concept is reflected on the
  aging step's readouts. Wizard, magus (life-stage funding): Concept age is editable,
  Experience shows it read-only beside an editable Gauntlet age, and the two do not
  fight. Type a birth year and watch the age follow, then the reverse. Set the saga year
  earlier than the birth year and confirm the age clamps to 0 with an advisory finding
  rather than underflowing. **Then the D3.3 check that matters:** with a consistent
  character (age 30, birth year 1190, saga year 1220), change the saga year to 1230 and
  confirm **neither** age nor birth year moved and the document is **not** marked dirty.
  Quit and relaunch: the saga year is still 1230. Open a pre-slice save and confirm
  nothing changed.

---

## 6. Schema-version plan

**One bump: 15 → 16, owned entirely by Slice 4.**

The review doc notes two implied bumps (#29's funding discriminator, #31's wizard
progress field). They are batched into one because two sequential bumps mean two
migrations to write and test, and a window where a save written at 16 cannot be read by
a build at 17.

- **Current state (verified)** `pub const SCHEMA_VERSION: u32 = 15;` at
  `crates/arm-rules/src/types.rs:2808`. A `assert_eq!(SCHEMA_VERSION, 15)` guard lives
  at `crates/arm-rules/src/life_stage.rs:1515` and must be updated deliberately, as
  part of the RED step, not incidentally.
- **Fields added at 16**
  1. `Entity::ability_funding: AbilityFunding` — a new closed enum (`Pool` /
     `LifeStages`), **serde-defaulted** so an old save parses without the key.
  2. `Entity::wizard_furthest_phase: Option<String>` — **skip-if-none**, so a
     character that never entered the wizard writes no key and round-trips byte-stably.
     Documented as **UI/document state on an engine-defined entity**, with no
     validation wired to it. **Deliberately a raw slug, not `Option<CreationPhase>`** —
     the closed enum's `Deserialize` rejects an unknown slug, and combined with the
     fail-the-whole-load rule below that would make a save carrying a since-removed
     phase (`"type"`, dropped in S2) unopenable, contradicting S5's required fallback.
     The slug is resolved leniently at restore against the profile's `creation_phases`;
     unresolvable → treated as absent. See Slice 4 for the full argument and the test
     that pins it.
- **Migration shape** In `load_entity_migrating`
  (`crates/arm-rules/src/types.rs:3034-3080`), following the established pattern
  exactly:
  - **Dispatch on field absence, never on the recorded `schema_version`** — a
    hand-edited save may carry any version alongside either shape. This is the rule the
    existing aging and talisman folds follow (`:3025-3028`).
  - `ability_funding` absent → infer `LifeStages` if `life_stages` is present, else
    `Pool`. That is today's rule (`state.svelte.ts:420`), applied **once** at load
    instead of on every read.
  - `wizard_furthest_phase` absent → stays `None`, which S5 reads as "manually
    created": all steps reachable, exempt from the clamp.
  - A stored slug that is **not** in the loaded profile's phase list is **not** a
    migration failure — S5's restore logic treats it as the `None` branch. This is what
    makes S2's phase removal/insertion safe for existing saves.
  - Stamp `SCHEMA_VERSION` only when a fold actually happened; a value that will not
    deserialize fails the **whole load**, because a silently empty fold would still get
    the version stamped and the next save would drop the data permanently.
- **What must be updated in the same slice** the `assert_eq!(SCHEMA_VERSION, 15)` guard;
  `crates/arm-rules/tests/roundtrip_proptest.rs`; any `examples/` fixture whose
  canonical serialization changes (keys stay sorted; the new optional key is omitted
  when `None`); `ui/src/lib/types.ts`; `crates/arm-rules/RULES.md` (the sparse-save
  departure and the retired mutual-exclusivity invariant).
- **Slices that must NOT bump the schema** every other slice. In particular **#25
  (Slice 12) needs no bump** — `age` and `birth_year` are both already stored, so the
  current-year setting is a pure editing aid. If a slice finds itself wanting a bump,
  stop and re-read this section; the answer is almost certainly that the state belongs
  in the frontend store or in an `arm-app` settings file, not on the entity.

---

## 7. Risks and traps

**CSS facts this plan depends on** (established during the review; do not re-derive or
"simplify" them away):

- **CSS multi-column *flows* content between columns.** Any height change shifts the
  column break and blocks **migrate to another column**. That is inherent to `columns`,
  not a bug in any picker. Current proof in the tree: `AgingRecordPanel` is split across
  the break, with "Apparent age" at the bottom of the left column and its own
  "Aging / Aging points per Characteristic" section at the top of the right.
- **CSS grid auto-placement is order-stable under height changes.** Each grid item owns
  its cell, so a block growing changes only row heights. This is precisely why #20's
  DECIDED fix is grid and not a tweak to `columns`.
- **`display: contents` wrappers survive the switch.** Children became column items;
  they become grid items the same way. `app.css:1056-1059` keeps working, but the
  `break-inside: avoid` companions at `:1042-1045` / `:1061-1065` become meaningless
  under grid and must be re-expressed (`gap`, not `margin-bottom`, is usually the
  grid-native form).
- **Flex gaps do not collapse margins.** `gap` and an adjacent UA margin **add**. This
  is the whole of #23: 1em + 0.4rem + 1em ≈ 2.4rem where 0.4rem was designed. It is
  also why fixing the margin reset (S1) must precede measuring anything (S2's space
  budget, S6's `minmax` floor).
- **`:last-child` is scoped per parent.** `SelectionList.svelte:79` opens a new `<ul>`
  per group, so `app.css:709-712`'s `:last-child` separator suppression fires once per
  group, not once per list. That is #8, and any fix built on `:last-child` will have the
  same bug.

**Prior symptom-patches that must be REMOVED, not layered on:**

- `ui/src/app.css:571-581` — `.region-row { min-height: 12rem }`. Its own comment
  records the exact bug #11 fixes: *"on the magus's Abilities step — the funding panel
  plus the Hermetic-minimums checklist — the row measured 0 in an 800px window, so both
  lists vanished and their controls were not even clickable."* After S2's split,
  **re-measure and remove it if it is no longer load-bearing.** Adding to it would be
  patching a patch.
- `ui/src/app.css:614-618` — `.list-scroll` (~3 rows). Same review, same slice.
- `ui/src/app.css:1067-1070` — the 720px `columns: 1` media query. S6 **replaces** it
  with `auto-fit` response; keeping both would fight.
- `ui/src/app.css:1577-1580` — the bare global `.error` typography rule. It is the
  cause of #6 and will keep colliding with any one-word severity class. Scope or rename
  it; do not add a `.warning` twin next to it and call it done.
- The ~20 ad-hoc paragraph-margin zeroings in `app.css`. S1 replaces them with one base
  rule; leaving them means the reset is invisible and the next person re-adds a 21st.

**Structural traps:**

- **`abilityFunding` is derived from plan presence (#29's blocker).**
  `state.svelte.ts:420` verified. Until S4 stores the mode explicitly, nothing can stop
  `setAbilityFunding` from destroying the plan, because deleting the plan *is* how the
  mode is recorded. And the sweep afterwards is **`Option` checks, not `match`es** — the
  closed-enum discipline will **not** produce compile errors for them. Known starting
  points: `life_stage.rs:453` (`entity.life_stages.as_ref()?`, where `None` means
  "pool"), `validation/life_stage.rs:647-648`, the `not_enough_xp` path,
  `completeness.rs`, `export.rs`. This needs a deliberate grep for `life_stages`, and
  the commit message should state what was found and converted.
- **`each_key_duplicate` in the V/F list (#9).** The engine concatenates House, Mythic
  Companion, `grants_selection` and warping grants **without dedup**, so one `ref` can
  be granted twice. A duplicate `{#each}` key throws in production and **aborts the
  whole tab's render** — the existing code carries an explicit comment about this
  (`VirtueFlawTab.svelte:158-162`) and the merged grouping path must keep an index in
  the key. Same hazard in `ParameterPicker.svelte:181-185` for unset parameterized
  abilities.
- **`CreationPhase::ALL` is a fixed-length array** (`types.rs:1407`,
  `[CreationPhase; 12]`). S2's batched remove-one/add-one keeps it at 12; doing #1 and
  #11 in separate slices would move it twice for no benefit.
- **`ui/e2e/wizard-walk.js` derives phases from `rules/core/character_types.json`** and
  throws with the known-filler list if a phase has no filler (`:139`, `:241-245`). A
  phase change without a filler fails e2e loudly — which is good, but it means S2 is not
  done until the filler exists.
- **The finding-code contract table at `validation/mod.rs:118-180` is documentation by
  convention.** Adding a code (S11) or removing one (S4's
  `life_stage_xp_pool_conflict`) without updating the table leaves the table lying. If
  no test enforces table/emitter agreement, S11 is the cheapest place to add one.
- **Locale key parity is enforced** (`ui/src/lib/i18n.test.ts:68`), so a forgotten
  German key fails the unit suite — but **nothing enforces German *correctness***. That
  is entirely on the author, and the rule is: translation tables first, then the German
  rulebook at the mirrored line number, never invention.
- **`cargo tauri build --no-bundle` is the only gate that type-checks the frontend.**
  Every slice here touches Svelte/TS. A slice that passes `test:unit`, `lint` and
  `format:check` can still fail the build; treat the build as part of the slice, not as
  a formality after it.

---

## 8. Review-doc corrections and clarifications

Recorded so a future session does not chase them:

1. **#14's German-terminology note is slightly wrong in its pointer.** The doc says the
   DE term for "Later Life" *"must come from `rules/source/de/translation-tables/`"*. It
   is **not in the tables** (searched). Its verified provenance is the German rulebook
   at the line mirroring the English source: `Ars Magica Definitive Edition
   Basisregeln.md:2214` and the heading at `:2390` give **"Späteres Leben"**, which
   `locales/de/main.ftl:239` already uses. "Gauntlet" **is** in the tables
   (`grundbegriffe.md:57` → *Lehrlingsprüfung*), as is "Apprentice"
   (`grundbegriffe.md:15` → *Lehrling*). So the rule stands — never invent — but the
   lookup order is: tables first, **then** the mirrored German rulebook line, and only a
   term found in neither is a genuine open question.
2. **#14's "the span may need surfacing to the UI" is already done.**
   `later_life_years` is on `LifeStageBudget` (`life_stage.rs:482`) **and** already
   mirrored in `ui/src/lib/types.ts:286`. No engine change is needed for "Later life
   (ages N-M)"; the span's start is `childhood.years` (`rules/core/life_stages.json:19`).
3. **#30's anchor `validation/mod.rs:126-176` is the finding-code *contract table*, a
   doc comment — not the emitting code.** The code constants are at `mod.rs:325`,
   `:361`, `:376`, `:494`; the emitters live in `validation/` submodules
   (`life_stage.rs:647-648` for `restricted_xp_unspent`, and the scores/spells
   validators). Adding a code therefore touches three places: the constant, the table
   row, and the emitter — plus both locales.
4. **#12's "7 of 7 for a list of 3" is confirmed from data.**
   `rules/core/life_stages.json:3-13` declares **3** `minimum_abilities` and **4**
   `recommended_abilities`; `MagusMinimumAbilities.svelte:66` passes
   `total: String(rows.length)` (all 7) while `:68-74` renders only the 3 required rows
   under the *Minimum Abilities* heading.
5. **#13's option (c) has a data-kind complication the doc does not mention.** The
   requirement's parameter would be a *text*-domain value ("Latin"), and putting a
   translatable string in `rules/core/life_stages.json` violates the
   mechanics-files-carry-no-translatable-strings rule. Option (c) therefore needs an
   id + `rules/i18n/<lang>/` entry. **Superseded by D1's answer** (§2): #13 is fixed by
   option (d) — an optional `name_unfilled` in the i18n layer — and the "name Latin"
   half of (c) became **#32**, resolved with a language-neutral `exemplar` slug in
   `rules/core/` plus its label in `rules/i18n/<lang>/`, which is exactly the
   id-plus-i18n shape this item identified as the only compliant one.
6. **No anchor drift found.** All 16 sampled `file:line` references were correct at
   `721224a`. The review doc's locations can be trusted; re-verify only the ones a
   preceding slice has already moved. An independent review pass re-checked ~45 anchors
   across Rust, Svelte, JSON, both `.ftl` files and both rulebooks and found none wrong.

### 8b. Corrections applied to this plan after its own review

This plan was reviewed before first presentation. Nine findings were raised; all were
verified against the tree and folded in above. The two that were **blocking**:

1. **S2 would have regressed the editor for one commit.** `AbilityTab` is mounted at
   `ui/src/App.svelte:316` as the *editor's* Abilities tab, and its own comment
   (`AbilityTab.svelte:203-206`) says the sharing is deliberate. Dropping
   `<LifeStagePanel />` there — with the editor's replacement not arriving until S3, and
   §4a.6 forbidding batched commits — would have left direct entry with no way to choose
   the funding mode or edit the life-stage plan, uncaught by e2e (the life-stage specs
   drive the wizard). S2 now carries a mandatory temporary editor mount and an
   acceptance criterion asserting it.
2. **`wizard_furthest_phase` could not have been `Option<CreationPhase>`.** The closed
   enum's `Deserialize` (`crates/arm-rules/src/types.rs:1365-1367`) rejects an unknown
   slug, and §6's fail-the-whole-load rule would then have made any save carrying
   `"type"` — a slug this plan itself removes in S2 — unopenable, contradicting S5's own
   acceptance criterion. Now `Option<String>`, resolved leniently at restore, with a
   dedicated RED test in S4.

The seven non-blocking findings, all applied: the e2e-mandatory list was too narrow
(S6/S8/S10 added, S7 conditional, and the *rule* restated so it does not depend on the
enumeration); `.character-details` is carried by three components so S6 restyles the
editor's Details tab and the Personality step too (plus two comments that reason about
multi-column and go stale); the "As a magus" rename covers two label keys and must
*not* touch six prose strings; Slice 7's data list mixed already-correct domain
declarations in with the five that need changing; D1 misplaced the parentheses (they are
in `param-hint`, not `param-label-language`); Slice 3's mirror test cannot enumerate a
Rust const from vitest; and the plan lacked a critical-path statement.

---

## 9. Final checklist

The plan is done when **all** of the following are true.

**Coverage**

- [ ] Every one of the 31 numbered issues **plus #32** is either implemented, or
      explicitly out of scope in §3 with a reason (#15 only), or a confirmed
      non-issue (§3).
- [ ] #32 is implemented as the widened-check-plus-exemplar shape, and `RULES.md`
      records why a `language.*` catalogue can never be built.
- [ ] All 11 DECIDED items are implemented as decided, with no alternative substituted:
      #9, #11, #12, #14, #19, #20, #24, #25, #28, #29, #31.
- [ ] D1, D2 and D3 are implemented **as answered in §2** — option (d) for #13,
      option (a) for #18, and D3's three answers including the two that changed
      (persisted saga year; the saga year recomputes nothing). No slice quietly
      substituted the plan's original recommendation for the user's answer.
- [ ] #15 remains unimplemented, and the reason (no per-year apprenticeship XP rate in
      `rules/source/en/`) is recorded where a future session will find it.

**Quality gates**

- [ ] The full gate (§4b) passes on the final commit, `cargo tauri build --no-bundle`
      included.
- [ ] `npm run test:e2e` passes on the final commit, and passed on each of Slices 2, 3,
      4, 5 (and 12 if D3 added persistence).
- [ ] No test asserts an exact catalogue total anywhere.
- [ ] Every new test was **seen failing first** — the red output is in the slice's
      subagent transcript, and the gate output for each slice was observed
      **first-hand** by the orchestrator, never taken from a subagent's summary.

**Invariants**

- [ ] `arm-rules` still has no dependency on `tauri`, the filesystem, or the UI.
- [ ] No user-facing string is hardcoded in Rust or Svelte; no raw ID, slug or enum
      value is rendered as a label.
- [ ] Every new/changed key exists in **both** `locales/en/main.ftl` and
      `locales/de/main.ftl`; `i18n.test.ts` parity passes; every German rules term
      traces to `rules/source/de/translation-tables/` or to a cited German rulebook
      line.
- [ ] Every displayed negative sign is the ASCII hyphen U+002D, via `formatSigned`.
- [ ] Exactly **one** `SCHEMA_VERSION` bump (15 → 16), owned by Slice 4, with a
      migration that dispatches on field absence and not on the recorded version.
- [ ] JSON output stays canonically sorted; `examples/` round-trips byte-stably.
- [ ] Saves store choices, not resolved values — `wizard_furthest_phase` is a slug, not
      an index.

**Documentation**

- [ ] Every new or changed mechanic carries a source citation
      (`<book basename>.md:<lines>`) **verified against the file**, and a matching
      `crates/arm-rules/RULES.md` entry.
- [ ] `RULES.md` records: the deliberate sparse-save departure (#29), the retired
      "plan and pool are mutually exclusive" invariant (#29),
      `wizard_furthest_phase` as UI/document state with no validation attached (#31),
      the two new unspent-budget codes and the explicit note that they assert **no**
      waste rule because the source makes none (#30), and the 1220 default year as a
      rules value backing a non-mechanical editing aid (#25).
- [ ] `README.md` is current with respect to what the project does and how it is built
      and run.
- [ ] `docs/guided-creation-review-2026-08.md` is left as the historical record — it is
      not edited to match the implementation.

**Mandatory product behavior**

- [ ] The unsaved-changes guard prompts before discarding on **every** platform and
      **every** quit path, macOS Cmd+Q included; the dirty-tracking tests in
      `ui/src/lib/state.svelte.test.ts` pass, the `update_close_guard` `$effect` mirror
      is still covered by a **client** test, and both
      `window-close-bridge-clean.e2e.js` and `window-close-bridge-dirty.e2e.js` pass.
- [ ] Slice 5's added guard cases (a Next click dirties the document; rail navigation
      does not) exist and pass.

**Process**

- [ ] One commit per slice, on `main`, each with the
      `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>` trailer.
- [ ] Each slice's manual verification note was actually performed in the running app,
      not inferred from green tests — several of these bugs are layout shifts that no
      unit test can catch.
