# Manual-testing findings — 2026-09-09

Findings from a manual test session across the whole project. This document is
a **collection pass**: findings are recorded, not fixed. Nothing is implemented
from here until the list is closed and an implementation order is agreed.

Each entry records the symptom as observed, the root cause once located in the
code, the rules citation where the finding is a rules question, and the decision
taken. Rules citations are `file basename:line` into `rules/source/en/`; the
mechanics behind them live in `crates/arm-rules/RULES.md`, which is the
traceability map and is not restated here.

Status values: **open**, **done**, **closed** (deliberately no change).

**Numbering continues from the previous session.** `docs/manual-testing-findings-2026-09-03.md`
ran to finding 37, so this document starts at **38** — finding numbers are unique
across both documents, and a reference to "finding 34" means exactly one thing.

**Concurrency note.** Another session is working in this repository while this
pass runs. This document is append-only and touches no code, no rules data and
no other document; `docs/open-todos.md` is *not* edited from here — rows are
proposed in a finding and carried over once the collection pass closes.

---

## Progress log

| Date | What landed | Gate |
|---|---|---|
| 2026-09-09 | Collection document opened; awaiting findings | n/a (docs only) |
| 2026-09-09 | Findings 38 (a/b/c) and 39 recorded from the guided-creation Mythic Companion pass | n/a (docs only) |

---

## Findings

### 38 — A Mythic Companion's type package is advisory where a magus's House package is binding

**Status:** open

**Observed.** In guided creation for a Mythic Companion, choosing a type — the
report used Nephilim — seeds the required Virtues, but the editor then treats
them as ordinary purchases: the Greater Characteristic rows let the user change
which Characteristic they apply to, and the Supernatural Virtues can be
deselected outright. RAW gives no such latitude. Two neighbouring surfaces show
the same class of defect (38b, 38c). The overall complaint: **Mythic Companion is
handled completely differently from Hermetic Magus**, and the magus treatment is
the correct one.

**Rules.** *Ars Magica - Definitive Edition (Core Rules).md*:2720-2730 —
"**Required Virtues:** All Nephilim must take the following Virtues", then a
closed list: Nephilim (free Mythic Companion Virtue), Blood of the Nephilim,
Greater Immunity: Disease, Great Stamina, Great Strength, Improved
Characteristics, Sense Holiness and Unholiness, Strong Angelic Heritage (free
with Nephilim). "Must take" with a named list is not a menu — the Characteristics
are named (*Stamina*, *Strength*), so they are not the player's to re-point, and
the Supernatural entries are not the player's to drop. Note the contrast the
rules themselves draw: the **substitute** allowance is worded only for required
*Flaws* ("or a suitable substitute agreed with the troupe", :2660, :2689, :2754),
never for the required Virtues.

**Root cause — 38a: required Virtues are seeded as ordinary budgeted selections.**
The data is right. `rules/core/mythic_companion_types.json:53-67` models
`mythic_type.nephilim` correctly: `grants` carries the two point-free Virtues
(`virtue.nephilim`, `virtue.strong_angelic_heritage`), and `required_virtues`
carries the six budgeted ones, each as a full `Selection` — including
`virtue.great_characteristic` twice with `characteristic.sta` and
`characteristic.str` params, exactly as the rules name them.

The divergence is in how the two layers *hold* that package:

| | Magus / House | Mythic Companion / type |
|---|---|---|
| Point-free items | `granted_selections`, engine-derived, never in `entity.selections` | same — `grants` (correct) |
| Mandatory budgeted items | *(House has none)* | `required_virtues`, **copied into `entity.selections`** |
| Editable by the user | no | **yes** |
| Enforcement | grant machinery | `mythic_required_trait_missing`, a **warning** |

`AppStore.setMythicType` (`ui/src/lib/state.svelte.ts:571-591`, via `#mythicPackage`
at :623-628) pushes the required package into `entity.selections` as plain rows —
the doc comment at :564-566 states the intent, "as ordinary budgeted selections".
Once there, they are indistinguishable from a free purchase: `VirtueFlawTab.svelte`
renders them with the normal remove control and the normal `ParameterPicker`, so
the Characteristic is re-choosable and the row is removable.

The engine does notice, but only softly.
`validate_mythic_type` (`crates/arm-rules/src/validation/magus.rs:355-370`) matches
each `required_virtues` entry against `entity.selections` by **full Selection
(ref + params)** and, on a miss, raises `CODE_MYTHIC_REQUIRED_TRAIT_MISSING` as a
**warning** on the `virtues_flaws` phase — deliberately non-blocking, per the
comment at :274-280, so Enforced mode "never hard-blocks a legal-with-substitute
build". That reasoning is sound for required *Flaws*, which the rules do let the
troupe swap; it is carried over to required *Virtues*, which they do not. So
re-pointing Great Strength to, say, Great Presence produces a warning, not an
error, and deselecting Blood of the Nephilim does the same.

**Decision needed** — which of these is the intended shape:

1. **Lock the rows** (closest to the magus treatment, and to what the report
   expects): render required-Virtue rows read-only — no remove control, no
   parameter picker — flagged as owed to the type. Their point cost still counts
   against the budget, so they cannot simply move to `granted_selections`; this
   needs a "budgeted but not editable" row state that does not exist yet.
2. **Keep them editable, raise the severity**: leave seeding as-is but make a
   missing/re-parameterized required *Virtue* an **error** rather than a warning,
   splitting the code so required *Flaws* keep the substitute allowance.

Option 1 matches the reported expectation and the magus precedent; option 2 is
much the smaller change. Either way the required-Flaw path must keep its
substitute allowance intact (`setMythicRequiredFlaw`, `state.svelte.ts:612-620`).

**Root cause — 38b: the Available list is not filtered by the character type at all.**
Reported as: building a non-mythic character still offers the Mythic Companion
Virtues (Nephilim and its siblings), and they can be selected, which then raises
an error — they should be invisible or greyed instead.

The data gate exists and is correct. `mythic_companion` is a real category on
those four Virtues (`rules/core/virtues_flaws.json:5333` for `virtue.nephilim`),
and it appears in exactly one profile's `permitted_categories` —
`rules/core/character_types.json:124-131`, the `mythic_companion` profile. The
engine enforces it: `category_not_permitted` / `forbidden_category`, both
**errors** on the `virtues_flaws` phase
(`crates/arm-rules/src/validation/mod.rs:141-142`, :335-337). That is precisely
the error the report saw.

The UI never consults the gate. `VirtueFlawTab.svelte` builds its sections from
`groupByCategory(rs, kindsFor(side))` (:67) — the whole catalogue, unfiltered by
profile — and its `disabled` predicate (:295-296) covers only three things:
already-selected, `atCap` (`max_total`), and `blocked` (incompatibilities). It
tests neither `permitted_categories` nor `forbidden_categories`. Confirmed
project-wide: outside the type declarations in `ui/src/lib/types.ts:867-868`,
**no `.svelte`/`.ts` file in `ui/src` reads either field**. So every profile is
offered the entire catalogue and learns what was illegal only after buying it.

**Root cause — 38c: The Gift, same mechanism, different gate.**
`virtue.the_gift` carries `categories: ["special"]`
(`rules/core/virtues_flaws.json:6181`), and `special` is used by **no other item**
and listed in **no profile's `permitted_categories`** — not even the magus's
(`character_types.json:85`). The Gift is instead routed through
`gift_policy`/`gift_id`/`gift_categories` (:93-95 magus `required`, :139-141
mythic companion `forbidden`; grog and companion likewise `forbidden` at :23 and
:55), enforced as `gift_required` / `gift_forbidden`
(`validation/mod.rs:149`, :349-351). The single UI read of any of this is
`derive.ts:754`, and it only handles the `required` direction (seeding the
magus's Gift). Nothing reads `gift_policy === 'forbidden'` to suppress the row,
so The Gift is offered to a grog exactly as Nephilim is.

**Shared remedy.** 38b and 38c are one missing filter with two data sources.
A single profile-aware predicate over the Available list — permitted/forbidden
categories plus gift policy — closes both. Whether the row is **hidden** or
**greyed with a reason** is the open call; the report accepts either. Note the
tab already has the vocabulary for the greyed form: `vf-blocked-incompatible`
and `vf-blocked-max-total` render a "why" line above a disabled row
(`VirtueFlawTab.svelte:121-138`), so a third blocked-reason key would follow the
established pattern. Greying is also the better fit for guided creation, where a
silently absent Virtue teaches the user nothing. Both new reason strings need
EN + DE Fluent keys.

---

### 39 — Nephilim's Greater Immunity is not pinned to Disease

**Status:** open

Raised as a by-product of 38, not from a separate observation.

:2725 requires "**Greater Immunity: Disease** (Major, Supernatural)" — a named
target, not a free choice. `virtue.greater_immunity`
(`rules/core/virtues_flaws.json:4371-4378`) carries **no `parameters`** and
`max_per_target: 255`, and the Nephilim requirement references it bare —
`{ "ref": "virtue.greater_immunity" }`
(`mythic_companion_types.json:60`), with no `params`. So what the character is
immune *to* is unrecorded anywhere in the model: a Nephilim satisfies the
requirement with an unspecified Greater Immunity, and nothing can state it is to
Disease.

Contrast the sibling entries in the same package, which *are* pinned:
`virtue.great_characteristic` carries `characteristic.sta` / `characteristic.str`
params, and Devil Child's Puissant Guile carries `ability.guile`
(`mythic_companion_types.json:18`). Greater Immunity is the one required Virtue
in the file whose named target is dropped.

Fixing it means giving `virtue.greater_immunity` a target parameter, which is a
catalogue-wide question rather than a Nephilim one — the free-text vs enumerated
choice is the same one open-todos row 9 records for the per-power targets, and an
enumerated domain now exists (`ParameterDomain::Enumerated`, `b86889c`). Save
impact: an existing character holding a paramless Greater Immunity would start
reporting `missing_param`, the same break `2380322` accepted for the three
`power` items.
