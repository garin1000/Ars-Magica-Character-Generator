# CV — catalogued parameter values

**Slice.** Fixes the D14 literal-instance defect (found 2026-09-27): an
`AbilityRef`'s `instance: { "literal": … }` is compared by exact string equality
against a player-typed free-text `AbilityScore.parameter`, so a correct answer in
the wrong case or language ("Latein" for a German player) silently fails to
authorize or fund. Design only, no code. **Revision 4** (agreed with Norbert;
his decisions are marked **[decided]**). This revision is **fully
self-contained** — a prior "revision 2"/"revision 3" was never committed as a
separate artifact, so nothing here says "unchanged from revision N"; every
section that exists is written out in full. Addresses the architect's
"approve after fixes": an ambiguity hole in Link resolution, the migration's
undocumented new dependency on a loaded ruleset, a rationale gap, missing
content from the "unchanged" shorthand, a cross-locale catalogue-collision
rule, and two small process items.

---

## 0. What already exists (read before designing on top of it)

| Piece | Where | Shape |
|---|---|---|
| `Ability.parameter` | `ability.rs::Ability` | `Option<String>` — the parameter's *key* (`"language"`, `"profession"`, `"organization"`, `"craft"`, `"area"`, `"mystery_cult"`), used only to pick the `param-label-<key>` Fluent label |
| `AbilityScore.parameter` | `types.rs::AbilityScore` | `Option<String>` — the player's **free-text value** for that key; part of the score's identity (`(ability, parameter)`) |
| `AbilityRef::Scoped.instance` | `types.rs::AbilityRef` | `Option<ParamValue>`; `ParamValue::Literal { literal: String }` (fixed value, D14) or `ParamValue::Bound { param }` (reads the SAME declaring item's OWN parameter at evaluation time) |
| Matching | `effective/ability.rs:95`, `effective/xp.rs:399` | `target.resolved_instance(selection).as_deref() == parameter` — plain `==` on two strings |
| `virtue.forge_companion` | `rules/core/virtues_flaws.json:4301-4311` | **Already the Bound precedent this design generalizes**: `parameters: [{key:"craft", type:"ref", domain:"text"}]`, pool instance `{"ability":"ability.craft","instance":{"param":"craft"}}` — the character's OWN typed value on this Virtue, not a fixed word |
| `ParameterDomain::Enumerated` | `types.rs::ParameterDomain` | closed list the item itself declares; **no fallback** |
| `ParameterDomain::Text` | `types.rs::ParameterDomain` | fully open, never resolved against any registry |
| `SelectionParamValue` | `types.rs` | `Single(Id) \| Multi(BTreeSet<Id>)` — shape-discriminated (scalar vs array); resolves by **fallback lookup at read time** (§ 3 explains why `AbilityParameterValue` does not copy this idiom) |
| `load_entity_migrating`'s pre-typed-parse key removal | `migration.rs:478-501` | `aging_reductions`/`talisman_attunements` are `obj.remove(...)`'d from the raw `serde_json::Value` before `serde_json::from_value` — the precedent CV's raw pre-pass (§ 5.1) builds on |
| `ruleset_guard`/`require_loaded`/`AppError::NotLoaded` | `crates/arm-app/src/commands.rs:234-254` | The existing two-step pattern every ruleset-needing command already uses: lock the cached `Option<LocalizedRuleset>` for reading, then turn "not loaded yet" into the one `AppError::NotLoaded` every command reports it with. `load_entity` (the "open a save" command, `commands.rs:904`) does **not** call this pair today, because `load_entity_migrating` has never needed a ruleset. CV4 (§ 5.6) makes it the second caller. |
| `validate_duplicate_selections` / `validate_total_selection_cap` | `validation/selections.rs:269-345` | Both raise a `ValidationIssue::error` (`CODE_DUPLICATE_SELECTION` / `CODE_TOO_MANY_SELECTIONS`) when an item's effective (bought+granted) count exceeds its cap — but a `ValidationIssue` is only **enforced** in `ValidationMode::Enforced`; Advisory shows it as a non-blocking warning and Silent suppresses it. So a `max_total: 1` item held **twice** (once bought, once granted) is a state the app can and does render in Advisory/Silent modes and in direct-unchecked entry — it is illegal, but not unreachable. § 1's Link design must not assume validation already rules this out. |
| D2 | `docs/vf-audit/decisions.md` | validators must read **effective selections — bought and granted** (union of `entity.selections` with `house::granted_selections`, `mythic_companion::granted_selections`, `warping::warping_granted_selections`), not bought-only |
| D10 | `docs/vf-audit/decisions.md` | `max_total`'s default is now `1` ("once means once"); a repeatable item declares the ceiling explicitly |
| D48 | `docs/vf-audit/decisions.md` | a restricted pool's `instances` is part of a **union** test; also the decision that first flagged `virtue.forge_companion`'s craft-scoping (F-74) — already fixed with the Bound precedent above |
| `no_gated_authorization_item_is_ever_granted`-shaped test | `design-c0-parameter-model.md` § 3 | C0's own precedent for exactly this class of risk: a gated-authorization item's exclusivity proof holds "within one `Selection`" but not "across a bought-plus-granted pair of the same item", which is *latent, not live* today because nothing grants `wise_one`/`custos`/`templar_specialist`/`student_of_realm`. C0 owes a data-integrity test pinning that absence rather than fixing the gate-fold itself. CV's § 7 adds the same shape of test for Bound/Link-declaring items. |

---

## 1. Scope

### 1.1 Existing literal instances — universal values only

**[decided] "Organization Lore: Guild" is not a catalogue value — removed.**
Verified against source: ArMDE:3615 (Craft Guild Training) — *"These must be
spent on any Craft or Profession Abilities, Bargain, or Organization Lore:
Guild"* — names **the character's own guild**, which is local and specific
(there is no single, universal "Guild" any two characters share, unlike "House
Bjornaer" or "the Order of Hermes", which name one specific real organization
every instance refers to identically). The same shape recurs at two more
sites, verified:

- **ArMDE:3729** (Educated (Vernacular), not yet in the catalogue — § 3.2a):
  *"…the Organization Lore of the character's company, Profession Merchant, or
  the language of trade in the company's region…"* — "the character's company"
  is exactly the same character-specific pattern.
- **ArMDE:3927** (Forge-Companion): *"…raise the particular Crafts **her master**
  practices"* — already fixed (§ 0's table): `virtue.forge_companion` declares
  its own `craft` parameter and its pool instance is `Bound` to it, not a
  literal. This is the existing precedent § 1.1a generalizes.

Grepped from `rules/core/virtues_flaws.json` (now **12** sites, **10** distinct
universal values, after removing Guild):

| Ability | `parameter` key | Value | Citing item(s) | Source |
|---|---|---|---|---|
| `ability.dead_language` | `language` | Latin | `flaw.covenant_upbringing`, `virtue.baccalaureus`, `virtue.custos`, `virtue.educated`, `virtue.falconer`, `virtue.hermetic_experience`, `virtue.marshal`, `virtue.master_of_kennels`, `virtue.simple_student` | ArMDE:3713 |
| `ability.dead_language` | `language` | Gothic | `virtue.clan_ilfetu` | ArMDE:3565 |
| `ability.dead_language` | `language` | Hebrew | `virtue.rosh_beth_din` | ArMDE:4878-4883 |
| `ability.profession` | `profession` | Falconer | `virtue.falconer` | ArMDE:3847-3852 |
| `ability.profession` | `profession` | Marshal | `virtue.marshal` | ArMDE:4453 |
| `ability.profession` | `profession` | Storyteller | `virtue.master_bard`, `virtue.senior_bard` | ArMDE:4461, :4904-4909 |
| `ability.profession` | `profession` | Poet | `virtue.master_bard`, `virtue.senior_bard` | ArMDE:4461, :4904-4909 |
| `ability.profession` | `profession` | Master of Kennels | `virtue.master_of_kennels` | ArMDE:4467-4470 |
| `ability.organization_lore` | `organization` | House Bjornaer | `virtue.clan_ilfetu` | ArMDE:3565 |
| `ability.organization_lore` | `organization` | Order of Hermes | `virtue.hermetic_experience` | ArMDE:4063-4065 |

No `ability.craft`, `ability.area_lore`, or `ability.mystery_cult_lore` literal
exists today; those stay pure `Text` (D9).

### 1.1a Bound-shaped "literals": the fix, item by item

| Item | Today | Fix | Slice |
|---|---|---|---|
| `virtue.craft_guild_training` | `instance: {"literal": "guild"}` (defect — "Guild" is not universal) | Add `parameters: [{"key":"guild","type":"ref","domain":"text"}]`; change instance to `{"param": "guild"}` | CV3 |
| `virtue.forge_companion` | Already `{"param": "craft"}` with its own `craft` parameter | No change — the shipped precedent | — |
| `virtue.educated_vernacular` | **Does not exist yet** (§ 3.2a, one of the four missing `Educated (…)` entries) | **Obligation recorded for slice X1** (the Educated-family slice), not CV's: when authored, it must declare `parameters: [{"key":"company","type":"ref","domain":"text"}]` and its `ability.organization_lore` pool instance must be `{"param": "company"}`, never a literal | X1 (not CV) |

Each of these three items needs a `guild`/`craft`/`company` Fluent
`param-label-<key>` in both locales (`param-label-craft` already exists for
Forge-Companion; `param-label-guild` is new, mechanical).

### 1.2 Rulebook-named values not yet wired as literals

Per Norbert's instruction to look ahead rather than re-open this file at the
next Educated-family slice (`corrections.md` § 3.2a): the four missing
`Educated (…)` Virtues name **four more languages**, and Educated (Vernacular)
also names a fixed, universal profession — before any of those entries exist.

| Value | Named by | Source | Kind |
|---|---|---|---|
| Arabic | Educated (Islamic)'s list; Educated (Hebrew)'s Iberia/East clause | ArMDE:3721, :3725 | language (universal) |
| Persian | Educated (Islamic) | ArMDE:3721 | language (universal) |
| Greek | Educated (Islamic); Educated (Vernacular)'s "usually Latin, Greek, or Arabic" | ArMDE:3721, :3729 | language (universal) |
| Aramaic | Educated (Hebrew) | ArMDE:3725 | language (universal) |
| Merchant | Educated (Vernacular): "Profession Merchant" | ArMDE:3729 | profession (universal — the profession's own name, exactly like Falconer/Marshal) |

**Educated (Vernacular)'s two OTHER clauses are not catalogue candidates**: "the
Organization Lore of the character's company" is § 1.1a's `company` Bound
parameter (character-specific, not universal), and "the language of trade …
*usually* Latin, Greek, or Arabic" names no fixed value at all (precedent only,
the Virtue's own choice stays open even once the catalogue knows those three
ids).

**Open point (10.1) — recommendation stands: include the five universal values
above now.** Decide before CV1.

### 1.3 Final catalogue

| Catalogue | id | Values (id — EN name) |
|---|---|---|
| dead languages | `catalogue.language_dead` | `language.latin` — Latin; `language.gothic` — Gothic; `language.hebrew` — Hebrew |
| living languages | `catalogue.language_living` | `language.arabic` — Arabic; `language.persian` — Persian; `language.greek` — Greek; `language.aramaic` — Aramaic |
| `profession` | `catalogue.profession` | `profession.falconer` — Falconer; `profession.marshal` — Marshal; `profession.storyteller` — Storyteller; `profession.poet` — Poet; `profession.master_of_kennels` — Master of Kennels; `profession.merchant` — Merchant |
| `organization` | `catalogue.organization` | `organization.house_bjornaer` — House Bjornaer; `organization.order_of_hermes` — Order of Hermes |

**L1a (2026-10-03, try-out finding 6, amends D59): the language catalogue is
split.** Revision 4 had `ability.dead_language` and `ability.living_language`
**share** one `catalogue.language`, which offered Arabic and Greek as Dead
Languages against ArMDE:7432 ("Arabic, Greek and Hebrew fill similar
functions, although of these only Hebrew is a dead language"). Norbert's
decision: Dead = Latin, Hebrew, Gothic (ArMDE:3565, "Gothic, the dead language
that the House uses"); Living = Arabic, Greek, Persian, Aramaic. Each value
sits in exactly one list; value ids and their i18n names are unchanged.
Both Abilities keep the parameter key `language` (it picks the
`param-label-language` Fluent label) and name their catalogue through the new
`catalogue` field (§ 2.2). The pool instances that cite a now-living value
(Educated (Islamic)'s Arabic/Greek/Persian, ArMDE:3721; Educated (Hebrew)'s
Aramaic, ArMDE:3725) move from `ability.dead_language` to
`ability.living_language`, and Turb Trained's `language` choice (Q-X6-2, "the
catalogue's dead languages", ArMDE:5181) narrows to the dead list. Saves
holding a living value under Dead Language
are moved on load by the separate migration slice L1b.

`ability.profession` uses `profession`; `ability.organization_lore` uses
`organization`. `ability.craft`, `ability.area_lore`,
`ability.mystery_cult_lore` stay uncatalogued.

---

## 2. Data model

### 2.1 New rules file: `rules/core/parameter_catalogues.json`

One entry per catalogue, sorted by `id` (canonical serialization):

```jsonc
{
  "catalogues": [
    {
      "id": "catalogue.language",
      "values": [
        { "id": "language.aramaic", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3723, 3725] } },
        { "id": "language.arabic",  "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3719, 3721] } },
        { "id": "language.gothic",  "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3563, 3565] } },
        { "id": "language.greek",   "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3719, 3721] } },
        { "id": "language.hebrew",  "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3723, 3725] } },
        { "id": "language.latin",   "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3711, 3713] } },
        { "id": "language.persian", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3719, 3721] } }
      ]
    },
    {
      "id": "catalogue.profession",
      "values": [
        { "id": "profession.falconer", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3847, 3852] } },
        { "id": "profession.marshal", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4449, 4453] } },
        { "id": "profession.master_of_kennels", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4467, 4470] } },
        { "id": "profession.merchant", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3727, 3729] } },
        { "id": "profession.poet", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4456, 4461] } },
        { "id": "profession.storyteller", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4456, 4461] } }
      ]
    },
    {
      "id": "catalogue.organization",
      "values": [
        { "id": "organization.house_bjornaer", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3563, 3565] } },
        { "id": "organization.order_of_hermes", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4063, 4065] } }
      ]
    }
  ]
}
```

Arrays sorted by `id`; the `source` field reuses `types.rs::SourceRef`
byte-for-byte (JSON carries no comments, so this — not a Rust comment — is the
provenance home, exactly as `Ability.source` already does it).

### 2.2 `Ability` gains one field

```rust
pub struct Ability {
    // …unchanged…
    pub parameter: Option<String>,          // unchanged: the label key
    #[serde(default, skip_serializing_if = "is_false")]
    pub catalogued: bool,                   // NEW
}
```

`catalogued: true` means: "this ability's parameter values are looked up in the
`rules/core/parameter_catalogues.json` catalogue whose id is `catalogue.<parameter
key>`" — reusing the existing key string rather than a second id field, since
the two are always in lockstep. **L1a amends this:** they stopped being in
lockstep once Dead and Living Language, which share the label key `language`,
needed different lists (§ 1.3). An Ability now names its catalogue with an
optional `catalogue: Option<Id>` field (`"catalogue.language_dead"`); when
absent, the id still falls back to `catalogue.<parameter key>`. Every reader
(picker options, load integrity, the § 5.3 fold) resolves it the same way.
`false` (default) means the parameter stays
free text with no catalogue — unchanged for `craft`/`area`/`mystery_cult`, and
also the answer for the two Ability-*requirement* mechanisms in § 8, which stay
wide by design.

Load-time integrity (`ruleset::integrity`), fails loudly with ids:

- `catalogued: true` requires `parameter: Some(_)` (cannot catalogue a
  non-parameterized ability).
- A `catalogue.<key>` matching the ability's `parameter` must exist (L1a: the
  catalogue the ability's `catalogue` field names, when it declares one).
- Every catalogue's `values` non-empty, sorted by `id`, unique **by id**
  within its own catalogue.
- **[decided, MEDIUM, new]** Within one catalogue, **no two values may collide
  under trimmed, case-folded comparison across the union of their EN and DE
  display names.** Two distinct ids with the same-looking name would make § 5.2's
  and § 4 rule 2's name-matching genuinely ambiguous ("Latin" cannot mean two
  different catalogue entries) — checked at ruleset load by folding every
  `(en_name, de_name)` pair per value, trimmed and lower-cased, into one set per
  catalogue and rejecting a second value that produces a name already in the
  set, citing both colliding ids.
- Every catalogue value's `id` must have an i18n `name` in **both** `en` and
  `de` (§ 2.3) — missing either fails load with the offending id.
- **[decided, MAJOR, new — § 7 restates this in the Integrity section]** A
  `Bound`-instance-declaring item must be `max_total <= 1`.

### 2.3 i18n: `rules/i18n/{en,de}/parameter_catalogue.json`

```jsonc
// en
{ "names": [
  { "id": "language.latin", "name": "Latin" },
  { "id": "language.gothic", "name": "Gothic" },
  { "id": "language.hebrew", "name": "Hebrew" },
  { "id": "language.arabic", "name": "Arabic" },
  { "id": "language.persian", "name": "Persian" },
  { "id": "language.greek", "name": "Greek" },
  { "id": "language.aramaic", "name": "Aramaic" },
  { "id": "profession.falconer", "name": "Falconer" },
  { "id": "profession.marshal", "name": "Marshal" },
  { "id": "profession.storyteller", "name": "Storyteller" },
  { "id": "profession.poet", "name": "Poet" },
  { "id": "profession.master_of_kennels", "name": "Master of Kennels" },
  { "id": "profession.merchant", "name": "Merchant" },
  { "id": "organization.house_bjornaer", "name": "House Bjornaer" },
  { "id": "organization.order_of_hermes", "name": "Order of Hermes" }
] }
```

Same shape, German names (§ 9). Rules text (a value's display name) lives in
`rules/i18n/`, never in Fluent — the picker's *chrome* is Fluent; the catalogue
*entries* are rules-i18n, exactly like every other rules-text/UI-chrome split
in this codebase.

**Migration also needs these names in a shape display does not: both locales at
once, regardless of the UI's active language** (§ 5.4) — noted here because it
constrains this file's loader, not just the picker's.

---

## 3. Saved-value shape — three kinds now

`AbilityScore.parameter` is `Option<AbilityParameterValue>`:

```rust
#[serde(untagged, deny_unknown_fields)]
pub enum AbilityParameterValue {
    /// A chosen catalogue entry (§ 1.3): matches a `Literal` by id, exactly.
    Catalogued { id: Id },
    /// [decided, NEW] Follows a Virtue/Flaw selection's own parameter live:
    /// "the guild I'm already a member of via Craft Guild Training", rather
    /// than a second, independently-typed copy of the same fact. Resolves
    /// against the CURRENT value of `item`'s own `param` at read time — so
    /// renaming the guild on the Virtue renames every linked Ability row too.
    Linked { item: Id, param: String },
    /// Free text: no catalogue, or the player picked "Other…", or a link
    /// target vanished/became ambiguous and was converted here (§ 5.4/5.5).
    Text { text: String },
}
```

Wire examples: `{"id": "language.latin"}`, `{"item": "virtue.craft_guild_training",
"param": "guild"}`, `{"text": "Klingon"}` — three disjoint field-name sets, so
`#[serde(untagged)]` discriminates unambiguously, exactly as `ParamValue`
already does for its own two variants.

### 3.1 Why two idioms, not one — `SelectionParamValue` vs `AbilityParameterValue`

**[decided, MAJOR, addressed]** `SelectionParamValue` (`Single(Id) \|
Multi(BTreeSet<Id>)`) and `AbilityParameterValue` look like they could be "the
same kind of thing, twice" — both are enums naming a stored parameter value.
They are not, and must not be unified later:

- **`SelectionParamValue` resolves by fallback lookup at read time.** Its two
  variants both name **the same kind of content** (one `Id` or several) and
  every consumer that wants "the one value" calls `.as_single()`, which
  degrades a `Multi` to `None` — there is no semantic difference in what a
  `Single` *means* versus a `Multi`, only in how many there are. Widening it
  again (a third variant) costs nothing conceptually: it is still "some Ids,"
  read through the same fallback.
- **`AbilityParameterValue`'s three variants mean three different things**, not
  three counts of the same thing: a catalogue reference, a *live pointer* into
  another selection's own data, and inert text. Collapsing them into "resolve
  to a string, however you get there" — the tempting unification, since § 3's
  own "match key"/"display value" split already reduces all three to a string
  for those two purposes — would **erase exactly the fact Link exists to
  preserve**: that a `Linked` value is not a string, it is a *relationship*
  that must survive a rename on the other end, be found and reported when its
  target disappears (§ 5.4), and be surfaced to the player as "this follows a
  Virtue" (§ 6.4) rather than presented as if it had been typed. A
  `SelectionParamValue`-style fallback-to-string read would let all of that
  quietly stop being true the first time someone "simplified" the read path.

So the two types are deliberately unrelated code, not a missed opportunity to
share one enum, and future work must not merge them.

### 3.2 Two different resolutions, kept distinct on purpose

- **Match key** (what a `Literal`/`Bound` pool test compares against, § 4):
  `Catalogued` → the id itself; `Linked` → the resolved current text of the
  target's own parameter (or "unresolvable", § 4); `Text` → the text itself.
- **Display value** (what the sheet/export/tooltip shows, § 6.4): `Catalogued`
  → the localized catalogue name; `Linked` → the resolved current text (same
  resolution as the match key — a link never has a separate "display name", it
  always shows what the Virtue currently says, or its ambiguity fallback, §
  4); `Text` → the text itself.

Both go through **one** engine function each (§ 6.4 names the display one; the
match-key one is internal to `effective/`), never duplicated at a call site —
this is what makes "one Ability instance visible everywhere shows one
consistent value" enforceable rather than aspirational.

**[decided] Applies uniformly to every parameterized Ability.** Adding `Linked`
does not reopen this: it is still one Rust field, one type, for every
`AbilityScore.parameter` regardless of ability.

**Wire-compatibility.** Adding `Linked` to an already-untagged enum is **purely
additive** for existing schema-18 data — an old `{id}`/`{text}` value still
parses exactly as before. § 5.1's raw pre-pass (bare string → `{text:…}`) is
unaffected: no pre-CV save could ever contain a `{item, param}` shape, because
the shape did not exist before CV. No separate version bump for this; it lands
inside CV's existing 17 → 18 (§ 5.7).

### 3.3 Hostile input: extra/conflicting keys — [decided, LOW, addressed]

A value carrying keys from more than one variant, e.g. `{"id": "language.latin",
"item": "virtue.craft_guild_training", "param": "guild"}`, is not a shape any
writer of this format ever produces — only a hand-edited or adversarial save
could contain it. Serde's default untagged behaviour would try each variant in
declared order and **accept the first structural match while silently
discarding the unrecognised extra fields** — here, matching `Catalogued { id }`
and silently dropping `item`/`param` — which is exactly the kind of quiet
misinterpretation CLAUDE.md's "fail loudly" convention exists to prevent.

**Fix, cheap: `#[serde(deny_unknown_fields)]` on the enum** (shown in § 3's
definition above). With it, `Catalogued`'s deserialize attempt fails because
`item`/`param` are unrecognised, `Linked`'s attempt fails because `id` is
unrecognised, `Text`'s attempt fails because `id`/`item`/`param` are all
unrecognised — every variant rejects the value, so the untagged enum as a whole
fails with "data did not match any variant", and the **whole entity load
fails** rather than one field being silently misread. This is the same
severity class as a load failing today over a malformed `bonus` in
`fold_legacy_talisman` (`migration.rs`): failing the load leaves the file
untouched, which is strictly safer than guessing.

**Test obligation (CV4, § 5.6's table):** a fixture with the three-key value
above fails to load with a clear "no matching variant" error, not a silently
wrong `Catalogued { id: "language.latin" }`.

---

## 4. Matching semantics — two rules, plus an ambiguity guard

**A `Literal` instance** (`AbilityRef::Scoped { instance: Some(ParamValue::Literal
{ literal: <Id> }) }`) is satisfied **only** by `Catalogued { id }` with
`id == literal`. `Linked` and `Text` never satisfy a `Literal`, full stop — this
is exactly why "Guild" had to leave the catalogue rather than stay as a
permissive literal: a literal can never mean "whichever one is yours".

**A `Bound` instance** (`AbilityRef::Scoped { instance: Some(ParamValue::Bound
{ param }) }`, e.g. Craft Guild Training's own pool covering
`ability.organization_lore` bound to its own `guild` parameter) is satisfied by
resolving the Bound source **first** (§ 4.1), then trying, in order:

1. **Structural link match** — the character's `AbilityScore.parameter` is
   `Linked { item, param }` naming the SAME declaring item and the SAME
   parameter key the pool's `Bound` reads from. No string comparison at all;
   true by construction, and immune to the source value ever changing (rename
   the guild, the match still holds).
2. **Content match, case-insensitive and trimmed** — the character's
   `AbilityScore.parameter` is `Text { text }` (typed by hand instead of
   linked) or `Catalogued { id }` (its localized name, in either locale, per §
   5.4's shared name-matching logic), and that content equals the Bound
   source's own current parameter value, folded and trimmed. **A player who
   typed the guild's name by hand instead of linking it is not punished** —
   this is the whole point of trying (2) at all.

Rule 2 deliberately reuses § 5.4's own catalogue-name-matching logic (one place
that knows "does this text spell out this catalogue value's name in either
locale", used by both the migration fold and this live check) rather than a
second, parallel implementation.

### 4.1 [decided, BLOCKER, addressed] Resolving the Bound source — the ambiguity guard

**The gap the architect found.** `(item_ref, param)` is only unambiguous if
`item_ref` names **exactly one** effective (bought ∪ granted, D2) selection.
§ 0 already records that `validate_duplicate_selections`/
`validate_total_selection_cap` flag a second copy as an *error*, not as
*impossible* — Advisory mode, Silent mode, and direct-unchecked entry all let
the character render anyway. A `max_total: 1` item held once bought and once
granted is therefore a **reachable** state, and "read `item_ref`'s own
parameter" has no defined answer when there are two.

**Rule: a Bound source (or a `Linked` target — the same resolution, § 4's rule
1) that resolves to more than one effective occurrence refuses to resolve.**
Concretely, resolving `(item_ref, param)` against effective selections:

- **Zero occurrences** → unresolvable, dangling (§ 5.5's fold territory if this
  is being read from a stored `Linked` value; for a `Bound` pool source it
  simply means the pool authorizes nothing right now).
- **Exactly one occurrence** → resolves normally to that occurrence's `param`
  value (or "unset" if the parameter was declared but never filled in).
- **More than one occurrence** → **ambiguous.** Never resolved by picking one
  via iteration order (`Vec` position is an implementation accident, not a
  rule). The resolver returns a distinct `Ambiguous` outcome, which two
  different consumers handle differently, deliberately:
  - **Matching (§ 4 rules 1/2, § 4.1 itself):** `Ambiguous` satisfies
    **nothing**. An ambiguous Bound source authorizes no instance, and an
    ambiguous Link target satisfies no pool test — the same "under-authorize
    rather than guess" bias a dangling link already has.
  - **Display (§ 6.4):** shows a **deterministic, provenance-based** fallback
    text rather than a blank field — "bought beats granted" (if exactly one of
    the effective occurrences is the bought copy, its value displays,
    labelled as uncertain); if the ambiguity is between two *granted* copies
    (today: unreachable, § 4.2), the display falls back to empty text. This is
    "last known text" in the sense the architect's wording asks for: a
    stable, rule-derived value, not a memoized history the type does not
    store and not an iteration-order accident.
  - **Both cases raise the same Fluent-keyed issue** (`issue-ambiguous_bound_parameter`,
    both locales), naming the item and the ability, so the player is told to
    resolve the duplicate rather than left guessing why funding or a link
    stopped working.

### 4.2 Mitigation: pin that this is latent, not live — mirrors C0 § 3

**[decided, add the C0 § 3 mitigation]** Exactly as C0 § 3 records for the
gated-authorization exclusivity proof ("not live today: no entry in the
catalogue grants `wise_one`/`custos`/…") — a data-integrity test,
`no_bound_or_link_declaring_item_is_ever_granted`, scans every
`GrantsSelection` target, every House grant, every Mythic-type grant, and
every Warping-fill grant, and fails if any of them names `virtue.craft_guild_training`,
`virtue.forge_companion`, or (once it exists) `virtue.educated_vernacular` —
the three items §1.1a's Bound wiring touches. Today it passes vacuously (none
of the three is granted anywhere), which is exactly the reassurance C0's
equivalent test gives: the ambiguity guard (§ 4.1) is real, tested,
correct-by-construction defense-in-depth, but the "bought + granted" scenario
it defends against is not reachable through any *shipped* data path yet. The
moment a future House or Mythic type grants one of these three items, this
test fails loudly and names the offending grant, rather than the ambiguity
guard silently becoming live with no one told.

**Red test obligation (CV5, § 5.6's table):** build a character holding
`virtue.craft_guild_training` **both** bought (with `guild = "Smiths' Guild of
Verdi"`) **and** — bypassing normal grant rules, exactly as a hand-edited or
direct-unchecked save could — granted a second time with `guild = "Different
Guild"`; a `Linked` Organization Lore into it must resolve `Ambiguous` and
raise the Fluent issue, never silently pick either value.

---

## 5. Migration

### 5.1 The blocker, and the fix

**Problem.** `load_entity_migrating` deserializes the whole `Entity` in one
typed `serde_json::from_value` pass. Once `AbilityScore.parameter` is
`Option<AbilityParameterValue>` (object-shaped), a pre-CV save's bare
`"parameter": "Latin"` fails that ONE typed pass outright — not per-field, the
**whole entity** fails to load. This is not limited to the 3 catalogued
parameters: § 3.2 settled that the type change is uniform, so every
parameterized-Ability save (Craft, Area Lore, Mystery Cult Lore included) is
affected. Grepped: **26** fixtures/examples under
`crates/arm-rules/tests/fixtures/` and `examples/` carry at least one
bare-string `"parameter"` value today.

**[decided] Fix: a raw-`serde_json::Value` pre-pass, before the typed parse**,
extending the precedent § 0 already names (`aging_reductions`/
`talisman_attunements` key removal) — except this one rewrites a **nested**
value in place rather than removing a top-level key, because the key
(`parameter`) is not moving, only its value's shape:

```rust
/// Rewrites every legacy bare-string `ability_scores[].parameter` into the new
/// tagged shape `{"text": <string>}`, before the typed parse. A bare string
/// cannot self-report "id" vs "text" the way `SelectionParamValue`'s scalar/array
/// split already could, so — unlike that field — this one is not wire-compatible
/// without a rewrite.
///
/// Idempotent: a `parameter` that is already an object (any schema-18+ save, or
/// one this same pre-pass already rewrote) is untouched; only a JSON *string*
/// value is rewrapped.
///
/// Untrusted input: a linear walk over `ability_scores`, an array already
/// bounded by the file the player opened — no recursion, no attacker-controlled
/// loop count beyond "one iteration per ability the save already lists".
fn wrap_legacy_ability_parameters(value: &mut serde_json::Value) {
    let Some(scores) = value.get_mut("ability_scores").and_then(|v| v.as_array_mut()) else {
        return;
    };
    for score in scores {
        let Some(obj) = score.as_object_mut() else { continue };
        if let Some(serde_json::Value::String(text)) = obj.get("parameter").cloned() {
            obj.insert("parameter".into(), serde_json::json!({ "text": text }));
        }
    }
}
```

Called immediately before `serde_json::from_value(value)`, alongside the
existing `aging_reductions`/`talisman_attunements` extraction.

**Why not a migration-only bare-string variant on the type itself.** That would
make `AbilityParameterValue` permanently accept three wire shapes (legacy bare
string, `{id}`, `{text}`) rather than two, forever — every future reader
(including a fresh save this build writes) would carry the legacy shape's
ambiguity in its own type definition, and it would also complicate § 3.3's
`deny_unknown_fields` hostile-input rejection (a bare string would need its own
carve-out from that check). The pre-pass keeps the permanent type exactly the
three shapes § 3 declares and confines legacy-shape knowledge to
`migration.rs`, matching where every other fold's complexity already lives.

### 5.2 Idempotence and untrusted input (pre-pass)

**Idempotence.** The pre-pass only rewrites bare-string shapes — a no-op on
anything already tagged (an object, including a schema-18+ `Linked` or
`Catalogued` value it has never seen before and does not need to touch).
Running it twice on the same value produces the same output as running it
once.

**Untrusted input.** `schema_version` is player-controlled (a hand-edited
`.armc`). The pre-pass performs only a linear walk over `ability_scores`, an
array whose length is already bounded by the file the player opened — no
recursion, no attacker-controlled loop count, no allocation proportional to
anything but the save's own declared ability count. Safe under CLAUDE.md's
untrusted-`schema_version` bar regardless of what a crafted save's
`schema_version` claims (the pre-pass does not even read that field).

### 5.3 The catalogue-matching fold (after the typed parse)

Once the typed `Entity` exists (guaranteed structurally valid by § 5.1), a
second, separate fold upgrades `Text { text }` → `Catalogued { id }` where it
matches:

1. Trim the text (already the entity-normalize convention).
2. Case-insensitively match it against every catalogue value's name, **in
   either English or German** — for the catalogue named by this ability's
   `parameter` key, only when `Ability.catalogued` is true for this ability.
3. Exactly one match → `Catalogued { id }`. (§ 2.2's new cross-locale
   collision rule is what guarantees "exactly one match" is well-defined —
   without it, two catalogue values could share a folded name and this step
   would be genuinely ambiguous rather than merely unlikely to collide.)
4. No match → stays `Text { text: <trimmed> }`, unchanged in meaning. **Never
   guessed.**
5. An uncatalogued ability's value, or a value already `Catalogued`/`Linked`,
   is a no-op for this fold.

**[L3, try-out findings 13/14, 2026-10-04] The fold normalises on every load;
it reports only on the migration.** Steps 3 and 4 run on every load, so typed
text naming a catalogue entry always becomes its id. The two reports (§ 5.5's
`migrated_catalogued_parameters` and `unresolved_catalogued_parameters`) are
kept only when the file's **raw** `schema_version` is below 18. "Raw" is the
file's own claim, read before any other fold stamps the current version. A save
at 18 or later was written with the catalogue picker available, so its
leftover text ("Native language", "Gaelic") is a deliberate "Other…" choice. It
never warns, and its normalisation is silent. Before L3, the notices fired on
every open.

**Both-locales-at-once is a new requirement on the loader, not just an
implementation nicety.** `LocalizedRuleset` (the app's existing i18n carrier)
holds **one active language's** display text at a time, merged with a
fallback — by design, for display. Migration is the opposite operation:
recognizing a value regardless of which language the player was using *when
they typed it*, independent of which language the UI happens to be showing
*now*. So this fold does not read through `LocalizedRuleset`; the caller
(`arm-app`) loads **both** shipped `rules/i18n/{en,de}/parameter_catalogue.json`
files directly, independent of the active UI language, and hands the combined
id→names index in alongside the language-neutral `Ruleset` (§ 5.6).

### 5.4 Dangling- and ambiguous-link fold (runs on every load, no version signal)

After § 5.3's catalogue-matching fold, a further fold — **value-driven,
idempotent, stamps no version**, exactly like `trim_all_selection_params` —
walks every `AbilityScore` whose `parameter` is `Linked { item, param }` and
resolves it via § 4.1's resolver against the character's effective selections:

- **Resolves to exactly one occurrence with a set value** → the link is live;
  nothing changes.
- **Resolves to zero occurrences, or one occurrence with the parameter unset**
  → dangling. Converted to `Text { text: <the value if the target was found
  but unset — "" — or "" if the item itself is absent> }`.
- **Resolves to `Ambiguous` (§ 4.1)** → converted to `Text { text: <the
  provenance-based fallback text from § 4.1, or "" if none applies> }`.

Both dangling and ambiguous conversions are **reported** via the
`LoadedEntity`/Fluent mechanism (§ 5.5) — a dangling or ambiguous link is
exactly as load-bearing to surface as an unmatched free-text value, for the
same reason (silent authorization loss).

This fold runs unconditionally on every load, not gated on `schema_version <
18` — a dangling or ambiguous link can arise on an up-to-date save just as
easily (the player edited the file, a later ruleset revision removed the item,
or — per § 4.1 — a duplicate selection exists), so version-gating it would be
wrong.

### 5.5 Engine-owned unlink operation, and reporting

**Unlink operation.** One function, e.g. `fn unlink_ability_parameters(entity:
&mut Entity, ruleset: &Ruleset, removed_item: &Id) -> Vec<Id>` (returns which
Abilities were converted, for the caller's notice), called by the UI's
Virtue/Flaw removal flow **before** the selection is actually spliced out (so
"last resolvable value" is still readable). Converts every `AbilityScore`
`Linked` to `removed_item` into `Text { text: <value just before removal> }`.
**The UI must call this, not reimplement it** — it is the same conversion §
5.4 uses for a load-time dangling link, so there is exactly one place that
decides what "losing your link target" means.

**[implemented, CV5] Shipped engine-only; the UI call site is CV7's, not
CV5's** (§ 10's slice table already scoped it that way). `unlink_ability_parameters`
is a real, tested function in `effective.rs` as of CV5 — but nothing in
`ui/src` calls it yet, because CV5 ships no UI at all. CV7's own row ("removal
flow calls § 5.5") is where the Virtue/Flaw removal handler must call this
function **before** applying the removal, exactly as documented above. Since §
7's integrity rule pins every Bound/Link-declaring item to `max_total <= 1`
and § 4.2 pins that none is ever granted, a link's target is always a bought,
once-only selection — so there are exactly **two** ways a link's target can
actually disappear, both of them CV7's to wire: the player removes that
Virtue/Flaw selection outright, or clears its own parameter (the guild/craft
text) back to empty. There is no third path (no grant can ever remove itself
out from under a link).

**Reporting.** `LoadedEntity` gains fields mirroring the existing
`migrated_aging_characteristics` precedent:

```rust
pub struct LoadedEntity {
    // …
    pub migrated_catalogued_parameters: Vec<(Id, String, Id)>,     // ability, original text, resolved id
    pub unresolved_catalogued_parameters: Vec<(Id, String)>,       // ability, original text — unchanged
    pub dangling_links: Vec<(Id, Id, String)>,                     // ability, target item, param
    pub ambiguous_links: Vec<(Id, Id, String)>,                    // ability, target item, param
}
```

The caller surfaces **all four** as a localized, Fluent-keyed notice on load —
not an `eprintln!` alone (the aging-migration precedent's own doc comment
calls out exactly this failure mode, Viktor #4: "the report is returned, not
swallowed"). `unresolved_catalogued_parameters`/`dangling_links`/
`ambiguous_links` are the cases where a previously-working-by-luck character
(e.g. a stray trailing space that used to `==`-match by accident, or a
duplicate selection nobody noticed) could silently lose an authorization —
these notices are what keep that from being silent.

**[L3] The two catalogue notices are one-time migration notices.**
`migrated_catalogued_parameters` and `unresolved_catalogued_parameters` are
filled only for a file whose raw `schema_version` is below 18 (§ 5.3's L3 note).
Once that file is saved, it is stamped with the current version and reopens
without either notice. `dangling_links` and `ambiguous_links` stay ungated (§
5.4). Both notices are worded count-neutrally in EN and DE, because `$items`
arrives as one pre-joined string and no `[one]` selector can see the count.

### 5.6 [decided, MAJOR, addressed] Per-call-site impact — `load_entity_migrating`'s new dependency

**`load_entity_migrating` has never taken a `Ruleset` or an i18n map — its
whole contract has been "raw JSON text plus a default saga year in, a migrated
`Entity` out."** CV4 breaks that, because § 5.3's catalogue-matching fold
cannot run without knowing which abilities are catalogued and what their
catalogue values are named in each locale. New signature:

```rust
pub fn load_entity_migrating(
    json: &str,
    default_saga_year: i32,
    ruleset: &Ruleset,
    catalogue_names: &BTreeMap<Id, Vec<String>>, // catalogue value id -> every known display name, all locales
) -> Result<LoadedEntity, serde_json::Error>
```

`catalogue_names` is built once, by `arm-app`, from **both** shipped
`rules/i18n/{en,de}/parameter_catalogue.json` files (§ 5.3) — not from the
single active-language `LocalizedRuleset`. The natural place to build it is
`commands.rs::load_ruleset`, the existing command that already loads the
ruleset and its i18n at startup/language-switch: it gains a second, small,
locale-independent load (both catalogue files, always, regardless of the
requested UI language) and caches the result in `AppState` alongside the
ruleset, e.g. `catalogue_names: RwLock<Option<BTreeMap<Id, Vec<String>>>>`.

**Every call site that breaks, or must be newly guarded, by file:**

| File | Call sites / construction sites | Verdict | What changes |
|---|---|---|---|
| `crates/arm-rules/src/migration.rs` | Owns `load_entity_migrating` itself | **Rewrite** | Signature above; § 5.1's pre-pass call, § 5.3's fold, § 5.4's fold all wired in; doc comment rewritten (below) |
| `crates/arm-rules/src/types.rs` | `AbilityScore` struct def (`:3525`) + 3 test-fixture constructions (`:5909,7381,7387`, all `parameter: None`) | **Type change, no fixture edits** | The struct's field type changes; every fixture here already uses `None`, unaffected by what `Some` wraps |
| `crates/arm-rules/src/derived.rs` | ~27 `AbilityScore { … }` test fixtures, of which **1** sets `parameter: Some(...)` | **1 site edited** | That one site's `Some("…".to_string())`-shaped value becomes `Some(AbilityParameterValue::Text{text:"…".into()})` (or a test-only constructor helper, § 5.6a); the other ~26 (`parameter: None`) are untouched |
| `crates/arm-rules/src/effective.rs` | ~17 fixtures, **6** set `Some(...)` | **6 sites edited** | Same pattern |
| `crates/arm-rules/src/export.rs` | ~12 fixtures, **6** set `Some(...)` | **6 sites edited** | Same pattern — distinct from `export/sections.rs` (below), which is the real display logic, not fixtures |
| `crates/arm-rules/src/export/sections.rs` | 3 read sites (`:227,240,353`) | **Rewrite (semantic)** | Already covered by § 6.4 — resolves `Catalogued`/`Linked`/`Text` to a display string instead of reading a bare `Option<&str>` |
| `crates/arm-rules/src/effective/ability.rs`, `effective/xp.rs` | Matching logic (§ 0's table) | **Rewrite (semantic)** | § 4's two-rule/ambiguity-guard matching replaces the plain `==` |
| `crates/arm-rules/src/validation/{scores,prereq,authorization,selections,life_stage}.rs` | Comparison/read sites | **Rewrite (semantic)** | Each site's `parameter.as_deref()`-shaped read becomes a call into the shared resolution/matching helpers |
| `crates/arm-rules/src/childhood.rs` | `raise_score`'s `parameter: Option<String>` argument (prod, `:363`) + 1 test-fixture `Some`-shaped helper (`:752-758`, currently `None` only) | **Signature change** | `raise_score` takes `Option<AbilityParameterValue>`; its one caller (the native-language pool builder) passes `Text{text: <chosen language>}`, since a child's native language is genuinely free text, not catalogued, at that phase |
| `crates/arm-rules/src/life_stage.rs` | `AbilityRequirement.parameter` (unaffected, § 8) + 1 test-helper fixture (`:1491`, `Some`-shaped) | **1 site edited, plus § 8's doc-comment fix** | The test helper's `parameter.map(str::to_string)` becomes a small wrapper producing `Text{text:…}` |
| `crates/arm-rules/tests/data_integrity.rs` | **4** `load_entity_migrating` calls (v0.2.x legacy-save compatibility tests, `:8538,9642,9666,9706`) + **4** `Some`-shaped `AbilityScore` fixtures + the § 3.3 hostile-input test lands here | **Signature-adapted + new test** | Each call gains `&ruleset, &catalogue_names` built from a small in-test fixture ruleset (these tests already build minimal rulesets); this file is also where the "no pre-CV save fails to load" regression proof belongs, since it already exists to prove exactly that class of guarantee |
| `crates/arm-rules/tests/book_templates.rs` | **1** `load_entity_migrating` call (`:58`), exercised once per book template — the 17-fixture sweep (§ 5.1) rides on this file | **Signature-adapted** | Already loads the real shipped ruleset to build templates against, so passing it through is a straight plumbing change, not a new fixture |
| `crates/arm-rules/tests/selection_param_value.rs` | **2** `load_entity_migrating` calls (`:132,138`) | **Signature-adapted** | Round-trip proof for `SelectionParamValue` — unrelated in subject matter, but breaks to compile the moment the signature changes |
| `crates/arm-rules/tests/save_invariant.rs` | **1** `Some`-shaped fixture | **1 site edited** | — |
| `crates/arm-rules/tests/export_golden.rs` | **1** `Some`-shaped fixture; this file also pins a **byte-for-byte** golden Markdown export | **1 site edited + golden-file check** | Once § 6.4's display resolution is live, a catalogued value must render the **same English text** it always has ("Latin"), so the golden fixture is expected to stay byte-identical — verify this explicitly rather than assume it, since it is the one place a silent wording drift would be caught |
| `crates/arm-rules/tests/core_type_conformance.rs` | **1** `Some`-shaped fixture (part of the "every core type validates clean" regression guard) | **1 site edited** | — |
| `crates/arm-rules/tests/d48_instance_pools.rs` | **2** `Some`-shaped fixtures — this file directly tests D48's instance/eligibility union (Marshal, Master of Kennels, Master Bard-shaped pools), i.e. exactly the mechanism CV's defect lives in | **2 sites edited + new assertions** | Gains the case-insensitive/locale-independent proof this whole slice exists for, plus (once CV5 lands) the Bound/Craft-Guild-Training case |
| `crates/arm-rules/tests/roundtrip_proptest.rs` | **1** `AbilityScore` construction inside a property-based generator | **Generator updated** | The `Arbitrary`-style (or hand-rolled) generator for `AbilityScore.parameter` must produce all three `AbilityParameterValue` shapes, not just `None`/a bare string, or the property test silently stops exercising two-thirds of the new type |
| `crates/arm-app/src/ruleset_io.rs` | `load_entity_from_path`'s 1 call site (`:732`) | **Rewrite** | Gains `ruleset: &Ruleset, catalogue_names: &BTreeMap<Id, Vec<String>>` parameters; doc comment rewritten (below) |
| `crates/arm-app/src/commands.rs` | `load_entity`, the Tauri "open a file" command (`:904-940`) | **New guard, MAJOR** | **Must call `ruleset_guard`/`require_loaded` before calling `load_entity_from_path`**, exactly like every other ruleset-needing command — today it is the one file-reading command that does not, because it never needed to. A file-open attempted before any ruleset has loaded now fails with the existing `AppError::NotLoaded`, not a panic or a confusing deserialize error. This adds no *new* failure mode: the app already cannot validate, display, or do anything else with an opened character without a loaded ruleset, and `commands.rs::load_ruleset` already runs at startup before any "open" action is reachable from the UI |

**Doc-comment rewrites required (both currently state, accurately until CV4,
that the function needs nothing but raw text and a default):**

- `migration.rs::load_entity_migrating`'s own doc comment gains a paragraph
  stating the new `ruleset`/`catalogue_names` dependency and why (§ 5.3).
- `ruleset_io.rs::load_entity_from_path`'s doc comment — which currently reads
  "The engine cannot read that file — it has no filesystem at all — so this
  crate, which owns the settings, hands it in" (about `default_saga_year`) —
  gains the same reasoning for the ruleset: `arm-rules` has no filesystem and
  cannot load its own rules, so this crate, which already owns ruleset loading
  and caching for every other command, hands both new arguments in too.

**Lands entirely in CV4** (§ 10's slice table) — not split across CV4/CV5,
because `load_entity_migrating`'s signature can only change once and every
caller must update in the same commit to compile.

### 5.6a A shared test constructor, to keep the ripple small

Given § 5.6's table shows the overwhelming majority of `AbilityScore { …
}` fixtures use `parameter: None` (unaffected by the type change at all) and
only 23 production-test sites across 9 files ever set `Some(...)`, a small
test-only helper — e.g. `AbilityParameterValue::text(s: impl Into<String>) ->
Self` returning `Text { text: s.into() }` — turns each of those 23 sites into
a one-token change (`Some("Latin".to_string())` → `Some(AbilityParameterValue::text("Latin"))`)
rather than a hand-written struct literal each time. Not load-bearing design,
but worth stating so CV4's implementer does not treat this as 23 independent
judgment calls.

### 5.7 Bump — [locked]

`SCHEMA_VERSION` 17 → 18 (own bump; "never two in one slice"). CV lands before
C5a: CV bumps 17 → 18, C5a bumps 18 → 19 (`design-c0-parameter-model.md` § 10
corrected to match). Adding `Linked` does not add a second bump (§ 3.2).

**Tests — every fixture that stores a currently-free-text catalogued value.**
`examples/magus_sample.json` plus all 16
`crates/arm-rules/tests/fixtures/book_templates/*.json` (17 files; several
store Profession or Organization Lore values, not only Latin) each get a
round-trip migration assertion — including `companion_witch.json`'s lowercase
`"latin"` (the C4 stopgap, **reverted to `"Latin"`** in CV3), which must still
resolve to `language.latin`. Plus, across the wider 26-fixture set (§ 5.1): an
adversarial fixture with an unresolvable value (`"Klingon"`) staying `Text`; a
mixed-case DE input (`"LATEIN"`) resolving via the German name; an
uncatalogued `craft`/`area` value correctly wrapped to `Text` and left alone;
a fixture already on schema 18 proving idempotence end-to-end (pre-pass
no-ops, folds no-op); the § 3.3 hostile-input three-key fixture, failing load
with a clear error; and (CV5) the § 4.2 bought-plus-granted ambiguity fixture.

---

## 6. UI

### 6.1 Picker

`AbilityTab.svelte`'s free-text `<input>` at the ability-parameter site
(today: `placeholder={store.t(`param-label-${key}`)}`, `value={entry.parameter
?? ''}`) becomes, for a `catalogued: true` **or** link-eligible ability, a
`<select>` combo box (§ 6.3) fed entirely by the engine's
`AbilityParameterOptions`: the catalogue's localized names, any current
link targets, and a trailing "Other…" option. Choosing "Other…" reveals the
existing free-text `<input>`. This reuses `ParameterPicker.svelte`'s existing
`enumerated`-branch pattern (dropdown + value list) as the closest sibling,
extended with the free-text escape that branch does not have and the
link-target branch neither has nor needs (V/F parameters are never
link-eligible themselves — only an *Ability's* parameter can point at one).

### 6.2 `sameParam` — the structural-equality helper, extended with the link kind

`AbilityScore.parameter`'s new shape breaks every UI site that compares it with
`===`/`??`-to-plain-string, silently: an object is never `===` a string or
`null`-coalesced the way a string was, so `bonusOf`/`floorOf`/`settledScoreOf`
(§ 6.6's table) would all resolve to "never matches" once the type lands — a
correct-looking diff that quietly zeroes every parameterized-ability bonus and
floor in the UI.

```ts
type AbilityParamValue = { id: string } | { item: string; param: string } | { text: string };

// `resolvedLinks` is engine-derived (`store.effective`): for every distinct
// (item, param) pair currently linked FROM some AbilityScore on this
// character, its CURRENT resolved text per § 4.1's resolver — including the
// "" fallback for a dangling/ambiguous link, computed once per derive() pass,
// never recomputed by the picker or by this helper.
//
// [decided, CV7 implementation correction, 2026-09-27 — supersedes the
// paragraph this replaces] A bonus/floor target (`AbilityBonus.parameter`) is
// NOT always id-shaped: it names whichever instance ITS OWN picker pointed
// at (`ParameterPicker.svelte`'s ability-target dropdown), which for an
// UNCATALOGUED ability (Puissant (Area) Lore targeting "Brandenburg") is
// genuine free text, not a catalogue id. Tagging a bare string as kind "id"
// unconditionally (this note's original text) broke exactly that case —
// found via a real e2e regression, `companion-editor.e2e.js`'s "Puissant
// Ability targets one ability instance" — because it then refused to match
// the bought `Text{"Brandenburg"}` row the bonus is about.
//
// The fix: a bare string is its OWN, deliberately promiscuous "raw" kind,
// matching by value alone against EITHER a `Catalogued` or a `Text` value.
// The anti-pathological guard survives only where it is actually meaningful
// — comparing two ACTUAL `AbilityParamValue`s against each other (as
// `settledScoreOf` does, two bought rows) — never between a bare engine
// string and a typed value.
function normalizeParam(
  p: AbilityParamValue | string | null | undefined,
  resolvedLinks: Record<string, string>,
): { kind: 'id' | 'text' | 'raw'; value: string } | null {
  if (p == null) return null;
  if (typeof p === 'string') return { kind: 'raw', value: p };
  if ('id' in p) return { kind: 'id', value: p.id };
  if ('item' in p) return { kind: 'text', value: resolvedLinks[`${p.item}\u0000${p.param}`] ?? '' };
  return { kind: 'text', value: p.text };
}

export function sameParam(
  a: AbilityParamValue | string | null | undefined,
  b: AbilityParamValue | string | null | undefined,
  resolvedLinks: Record<string, string>,
): boolean {
  const na = normalizeParam(a, resolvedLinks);
  const nb = normalizeParam(b, resolvedLinks);
  if (na === null || nb === null) return na === nb;
  if (na.kind === 'raw' || nb.kind === 'raw') return na.value === nb.value;
  return na.kind === nb.kind && na.value === nb.value;
}
```

**Deliberately no case-folding here.** § 4 rule 2's case-insensitive/trimmed
compare is a **Rust-engine, pool-satisfaction** rule (comparing independently
*typed* text against a Virtue's value); `sameParam` is a **UI, same-entity**
comparison (a bonus/floor target against the row that produced it, or two rows
against each other) where both sides are either already-resolved or literally
the same stored value — importing fuzz here would let two visibly-different
rows report as "the same bonus", which is a display bug in the other
direction. Kept separate on purpose; do not merge them later "for
consistency".

### 6.3 Combo box — engine-built, and why it bundles two sources

**[decided] The UI must not derive the option list itself.** The engine
computes, per catalogued-or-linkable ability and attached to the derived
payload (alongside `ability_bonuses`/`ability_score_floors`/
`magus_minimum_abilities` — the existing precedents for "engine ships a
per-character derived list, UI only renders it"):

```
AbilityParameterOptions {
  ability: Id,
  catalogued: Vec<Id>,        // this ability's catalogue values, in catalogue order (empty if uncatalogued)
  linked: Vec<{ item: Id, param: String }>,  // every item the CHARACTER holds (bought ∪ granted, D2) that:
                                              //   - declares `param` as its own parameter,
                                              //   - has a `Bound{param}` AbilityRef targeting THIS ability
                                              //     in a currently-active effect (respecting any `ParamGate`),
                                              //   - resolves unambiguously (§ 4.1 — an ambiguous source is not
                                              //     offered as a link target; offering it would let the player
                                              //     link to a value that cannot itself resolve),
                                              //   - and is itself `max_total <= 1` (§ 7)
}
```

plus, unconditionally, the free-text "Other…" escape.

**[decided, MINOR, addressed] Why one struct bundles both lists rather than two
separate derived outputs.** Both `catalogued` and `linked` populate the SAME
combo box for the SAME ability, in one visible order (catalogue values first,
then link targets, then "Other…" — a single, stable ordering the player learns
once). If the engine shipped two independent lists, the UI would own deciding
how to interleave or order them relative to each other, which is exactly the
kind of decision § 6.3's opening rule ("the UI must not derive the option list
itself") exists to keep out of the frontend. Bundling them in one struct makes
the **engine** the one and only source of the combo box's ordering, matching
every other "the engine, not the UI, decides display order" precedent in this
codebase (e.g. `AbilityCategory::ALL`, `Ruleset.ability_category_order`).

The Svelte picker renders `catalogued` (localized names) and `linked` (each
shown via § 6.4's resolution — "linked to «Craft Guild Training»: Smiths'
Guild of Verdi", not a raw id/param pair) and offers "Other…" last. Choosing
any option simply writes the corresponding `AbilityParameterValue`; **choosing
a different option always unlinks** — there is no "keep both" state, matching
§ 4's model where exactly one shape is stored at a time.

### 6.4 Display

**One engine function resolves a link (or a catalogued id) to its display
value**, and every reader — sheet, Markdown export (`export/sections.rs`),
tooltips, validation/issue messages, `MagusMinimumAbilities`'s `instanceOf`
(§ 6.6's table) — calls it; no reader re-derives a link's current value on its
own. Parity tests cover it directly (one Rust test, one TS test, same
fixture). **No raw id, and no raw `(item, param)` pair, is ever rendered.**

The UI additionally shows, beside a `Linked` value, that it **follows the
Virtue** — a small inline indicator (e.g. a chip naming the source item),
distinguishing "this Organization Lore reads whatever Craft Guild Training
currently says" from a plain typed value that happens to currently agree with
it. An **ambiguous** link (§ 4.1) is shown with a distinct, visibly different
indicator (not the same chip) plus the Fluent-keyed issue text, so "linked and
fine" is never visually confused with "linked and broken".

### 6.5 TS mirror + parity

`ui/src/lib/types.ts`'s `AbilityScore.parameter?: string | null` becomes
`{ id: string } | { item: string; param: string } | { text: string } | null`.
Existing Rust/TS parity tests gain a case for this type; a new
`rules/i18n/*/parameter_catalogue.json` schema test confirms every catalogue
id has a name in both locales; a new parity test confirms `AbilityParameterOptions`'s
shape matches its Rust source.

### 6.6 Every `.parameter` comparison site in `ui/src`, classified

Grepped exhaustively; sites split into **unaffected** (a different mechanism
entirely, unchanged by CV) and **in scope** (reads or compares
`AbilityScore.parameter` and must route through `sameParam` or the catalogue/
link resolution).

**Unaffected — different field, different mechanism, no change:**

| Site | What it actually is |
|---|---|
| `ability-workflow.svelte.ts:43`, `file-operations.svelte.ts:71`, `AbilityTab.svelte:70,188`, `derive.ts:362,1333`, `selection-workflow.svelte.ts:115`, `ParameterPicker.svelte:154,162,174,370` | `Ruleset.abilities[id].parameter` — the label **key** (`"language"`), not a value; untouched |
| `spell-workflow.svelte.ts:54`, `SpellTab.svelte:93,200,397`, `derive.ts:1282,2297,2544`, `DerivedPenetrationSection.svelte:21` | `SpellSelection.parameter` — the Art-target of a meta-magic Vim spell; a wholly different mechanism, out of CV's scope |
| `CharacterDetails.svelte:200`, `VirtueFlawTab.svelte:86,387-388`, `HouseSelector.svelte:185`, `MythicCompanionTypeSelector.svelte:210`, `file-operations.svelte.ts:68,74`, `derive.ts:645,852` | `PointItem.parameters` (V/F `ParameterDef` list) and `ParameterPicker`'s own domain branches — D14/D9's territory, untouched by CV |

**In scope — reads or compares `AbilityScore.parameter`, must change:**

| Site | Current shape | Fix |
|---|---|---|
| `AbilityTab.svelte:201` (`bonusOf`) | `(b.parameter ?? null) === (parameter ?? null)` | `sameParam(b.parameter, parameter, resolvedLinks)` |
| `AbilityTab.svelte:208-211` (`floorOf`) | `if (parameter != null) return 0;` | Structurally unaffected (only tests null-ness) — but the caller now passes an `AbilityParamValue`, so the parameter type annotation changes even though the body doesn't |
| `AbilityTab.svelte:214-220` (`effectiveOf`) | passes `parameter` through to both above | Type annotation only |
| `AbilityTab.svelte:228-235` (`settledScoreOf`) | `(a.parameter ?? null) === (entry.parameter ?? null)` | `sameParam(a.parameter, entry.parameter, resolvedLinks)` — **both** sides are `AbilityScore.parameter`, so this is the pure `Text`-vs-`Text`/`Linked`-vs-`Linked` case `sameParam` also has to get right |
| `AbilityTab.svelte:191-196,332,336,342,389` (`selectedName`) | passes `entry.parameter` (now an object) to `abilityDisplayName` | `abilityDisplayName` must resolve `Catalogued`/`Linked` through § 6.4's one resolution function and show `Text` verbatim |
| `AbilityTab.svelte:403` | `value={entry.parameter ?? ''}` free-text input | Replaced by § 6.1/6.3's combo box |
| `MagusMinimumAbilities.svelte:37-47` (`instanceOf`) | builds `{score, parameter: string \| null}` from `bought.parameter` (an `AbilityScore`) and returns it for direct display | `bought.parameter` is now `AbilityParamValue \| null`; `instanceOf`'s return type and its caller (`requirementAbilityLabel`) must resolve a `Catalogued`/`Linked` value to its display name (§ 6.4) rather than pass the raw value through — note `AbilityRequirement.parameter` **itself** stays `Option<String>`/unaffected (§ 8): only the *bought* `AbilityScore` side of this comparison changes shape |
| `derive.ts:2247-2263` (`unboughtModifiedAbilities`) | `bonus.parameter == null` | `AbilityBonus.parameter` is engine-derived, stays a plain string (§ 6.2's rationale) — only a null-check, unaffected in practice, but confirm with a test since it sits beside in-scope code |

Ability-only fixtures/components not covered by the grep (e.g. any future test
mock building an `AbilityScore` literal) are covered by the TS type change
itself: `parameter: 'Latin'` no longer type-checks, so `npm run check`
surfaces every remaining site the grep might have missed — this is the
backstop, not a substitute for the table above.

---

## 7. Integrity at load

- Every `AbilityRef.instance`'s literal (now an `Id`) must resolve inside the
  target ability's catalogue (its `catalogue` field, else the one its
  `parameter` key names; L1a) — fail loudly with
  the offending literal id and the ability id (mirrors the existing
  `validate_gated_ability_refs` pattern for `ParamValue::Bound`).
- Catalogue ids unique within their own catalogue; catalogues and values
  sorted by id.
- **[decided, MEDIUM, new] Within one catalogue, no two values collide under
  trimmed, case-folded comparison across the union of their EN and DE names**
  (§ 2.2 states the rule; this bullet is its Integrity-section restatement).
- Every catalogue id has an `en` and `de` i18n name.
- `catalogued: true` requires its catalogue (the `catalogue` field, else
  `catalogue.<parameter>`; L1a) to exist, and
  vice versa a catalogue naming no ability's parameter key is dead data.
- **[decided, BLOCKER-mitigating, new] A `Bound`-instance-declaring item must
  be `max_total <= 1`.** `Selection` carries no stable per-copy id, so
  `(item_ref, param)` is the *only* handle a Link or a Bound pool source can
  name — if the declaring item allowed two copies (`max_total > 1`), each with
  its own `guild`/`craft`/`company` value, `(item_ref, param)` would be
  ambiguous between them by construction, not merely at runtime. D10 already
  defaults `max_total` to `1`, so this is a no-op for every item shipped today
  (only an item that explicitly widens the default AND also declares a Bound
  instance would ever trip it) — checked at ruleset load, failing loudly with
  the item id and the param key. **This bounds authored data, not saves** —
  § 4.1's ambiguity guard is the *separate*, still-necessary defense against a
  same-`max_total`-item held twice via bought+granted, which no ruleset-level
  check can rule out (that is an entity-level, not ruleset-level, fact).
- `no_bound_or_link_declaring_item_is_ever_granted` (§ 4.2) — a data-integrity
  test, not a load-time rejection, pinning that the ambiguity guard's scenario
  is latent rather than live.
- Every catalogue-name-matching function (§ 4 rule 2, § 5.3) is the SAME
  function, not two independent implementations that could disagree.
- § 3.3's `deny_unknown_fields` rejects a value naming keys from more than one
  `AbilityParameterValue` variant.

---

## 8. Deliberately unaffected: the two regional-language requirement mechanisms

Two existing mechanisms compare an `AbilityScore` against a *requirement*, not
a *literal*, and both are correct to stay instance-blind — CV must not touch
their behavior, only correct one doc comment that is about to become
factually stale.

- `ruleset.rs::ScholarlyLanguageRequirement` (`validate_academic_language`,
  `validation/authorization.rs:120`) — "learning an Academic Knowledge normally
  requires a Latin, Greek, Hebrew, or Arabic score of at least 3, **depending on
  the region of Europe you are from**" (ArMDE:7151). The requirement is
  deliberately **any** qualifying language, by rule — narrowing it to one
  specific catalogued instance would misimplement ArMDE:7151, not fix
  anything.
- `life_stage.rs::AbilityRequirement` (the Order's minimum-Ability checklist,
  ArMDE:2437, "Latin 1") — same shape, `parameter: Option<String>`, shipped
  data-wide as `None` throughout, matched by Ability id alone.

**Both stay wide by design; this does not change.** What changes is one
sentence: `AbilityRequirement`'s doc comment currently reads *"languages are
troupe-defined free text and the rulebook publishes no language list, so there
is no catalogue for a requirement to point at and never will be."* CV creates
exactly the catalogue that sentence says will never exist — for a
**different** purpose (matching a rules-authored literal, not narrowing a wide
regional requirement). **One-line correction, landed in the same slice that
lands the catalogue (CV1, § 10):**

> ~~so there is no catalogue for a requirement to point at and never will
> be~~ → "so this requirement stays instance-blind by design even after CV
> (`docs/vf-audit/design-cv-catalogued-values.md`) adds a `language` catalogue
> for literal instances elsewhere — narrowing to one instance here would
> misstate ArMDE:7151/:2437's *any-qualifying-language* rule, not merely widen
> an implementation gap."

No code change, no test change beyond CV1 — a doc comment fix, tracked so
CV1's PR description doesn't have to explain the surviving "no catalogue"
sentence to a future reader who can see the catalogue right there in the same
crate.

---

## 9. German names

No dedicated language/profession/organization glossary table exists in
`rules/source/de/translation-tables/` (checked: `islamische-begriffe.md` and
`juedische-begriffe.md` are Arabic/Hebrew *cultural-term* glossaries, not
language names). Per CLAUDE.md's precedence (tier 3: thematic table before
`tugenden-fehler.md` — no table applies to a bare language/profession name —
falling through to the DE rulebook), names are taken from
`rules/source/de/Ars Magica Definitive Edition Basisregeln.md`, **at the
line-parallel position of the item that names the value** (not a
coincidentally-matching passage elsewhere — Storyteller/Poet are cited at
Master Bard's/Senior Bard's own line positions, not Educated (Bardic)'s, even
though all three passages happen to name the same two professions):

| id | German name | DE source line | Corresponding EN line (from § 1.1/1.2) |
|---|---|---|---|
| `language.latin` | Latein | 3713 | 3713 |
| `language.gothic` | Gotisch | 3565 | 3565 |
| `language.hebrew` | Hebräisch | 3725 | 3725 |
| `language.arabic` | Arabisch | 3721 | 3721 |
| `language.persian` | Persisch | 3721 | 3721 |
| `language.greek` | Griechisch | 3721 | 3721 |
| `language.aramaic` | Aramäisch | 3725 | 3725 |
| `profession.falconer` | Falkner | 3849 | 3847-3852 |
| `profession.marshal` | Marschall | 4453 | 4453 |
| `profession.storyteller` | Geschichtenerzähler | **4461** (Master Bard) and **4908** (Senior Bard) | 4461, :4904-4909 |
| `profession.poet` | Dichter | **4461** (Master Bard) and **4908** (Senior Bard) | 4461, :4904-4909 |
| `profession.master_of_kennels` | Meister der Hundezwinger | 4469 | 4467-4470 |
| `profession.merchant` | Kaufmann | 3729 | 3729 |
| `organization.house_bjornaer` | Haus Bjornaer | 3565 (compound "Haus-Bjornaer-Kunde") | 3565 |
| `organization.order_of_hermes` | Orden des Hermes | `grundbegriffe.md:85` (table, thematic — takes precedence for this one over the rulebook's compound "Ordenskunde") | 4063-4065 |

Verified: DE line 4461 reads *"…muss mindestens 5 in Beruf: Geschichtenerzähler
oder Dichter haben…"* (Master Bard); DE line 4908 reads *"…erhält 90
zusätzliche Erfahrungspunkte für Gedächtniskunst, Beruf: Geschichtenerzähler,
Beruf: Dichter…"* (Senior Bard); DE line 3729 reads *"…Beruf: Kaufmann…"*
(Educated (Vernacular)) — all at the same line position as their EN
counterparts, confirming the mirror.

`organization.order_of_hermes` is the one entry where a table **does** apply
and wins over the rulebook's compound word, per precedence tiers 2/3 — the
table names the *organization*, the rulebook's compound names the *Ability
instance*, and the catalogue stores the organization's name.

`organization.guild`/"Zunft" is **removed** (§ 1.1) — Guild is no longer a
catalogue value. The word survives only as the new Fluent field label:
`param-label-guild` (en: "Guild"; de: "Zunft") for Craft Guild Training's own
text parameter — a field *label*, not a catalogue value, which is the correct
place for it now.

---

## 10. Slice breakdown

Each row's first test must be written and observed **red** before any
implementation line, per the red-checkpoint protocol.

| # | Slice | Size | First failing test | Notes |
|---|---|---|---|---|
| CV1 | `rules/core/parameter_catalogues.json` + `rules/i18n/{en,de}/parameter_catalogue.json`, load + integrity (including § 2.2's cross-locale collision rule) + § 8's doc-comment fix | **S** | loading a catalogue with a duplicate id, or with two values whose EN/DE names collide case-insensitively, fails with the offending id(s) | Content per § 1.3 (2-value organization catalogue). `load_parameter_catalogues`/`load_catalogue_names` ship as standalone functions, **not yet wired into `Ruleset`/`RulesetSources`** — that wiring is CV2's own first red, not CV1's (avoids rippling the ~10 call sites that build `RulesetSources` as an exhaustive literal for a field CV1 does not need). |
| CV2 | `Ability.catalogued` field, and wiring `Catalogue` into `Ruleset`/`RulesetSources` so `validate_integrity` can resolve a `catalogue.<key>` against it | **S** | `catalogued: true` with no `parameter` fails to load | |
| CV3 | 12 literal sites → catalogue ids; revert `companion_witch.json` stopgap; `virtue.craft_guild_training` gains `guild` parameter + Bound instance (§ 1.1a); ruleset integrity gains the `max_total <= 1` Bound-declaring-item check (§ 7) | **S/M** | a `ParamValue::Literal` resolving in no catalogue fails load; a Bound-declaring item with `max_total > 1` fails load with its id | Rules-data-only; no bump |
| CV4 | `AbilityScore.parameter: Option<AbilityParameterValue>` (all 3 variants land here per § 3.2's uniformity and § 3's wire-compatibility argument, though only `Catalogued`/`Text` are reachable through the UI until CV5-7) + § 3.3's `deny_unknown_fields` hostile-input rejection + every read/compare site in § 5.6's table + § 4's Literal-only matching (Bound/Link matching is CV5) + § 5.1's raw pre-pass + § 5.3's catalogue-matching fold + § 5.6's `load_entity_migrating`/`ruleset_io`/`commands.rs` signature and guard changes + `SCHEMA_VERSION` 17→18 + `LoadedEntity`/Fluent report | **L** | `examples/magus_sample.json` loads at schema 17 (with a real `&Ruleset` and `&catalogue_names` fixture) and resolves to `Catalogued{"language.latin"}` — not a fresh hand-written value | Ships the original D14 fix standalone — CV5-8 are a pure follow-on enhancement, separately shippable in sequence. `AbilityParameterValue::Linked` the *type* can land here (it costs nothing extra once the enum exists), but no code produces or resolves one until CV5. **Must also revert `crates/arm-rules/tests/fixtures/book_templates/companion_witch.json`'s Dead Language score from the interim id `"language.latin"` (CV3) back to the human-typed `"Latin"`** — CV3 left it as the id only because a `Literal` still matches `AbilityScore.parameter` by plain string equality pre-CV4 (`AbilityScore.parameter` was still `Option<String>`); once § 5.3's name-matching fold exists, "Latin" is the value CV4's own § 5.7 test-obligation list already expects, and the id would no longer be the player-typed shape this slice's migration fold is supposed to prove itself against. |
| CV5 | § 4's Bound/Link matching (structural-or-content) + § 4.1's ambiguity guard + § 4.2's `no_bound_or_link_declaring_item_is_ever_granted` mitigation test + § 5.4's dangling/ambiguous-link fold + § 5.5's unlink operation and reporting fields | **L** | a character `Linked` to Craft Guild Training's `guild` satisfies its own pool; removing that Virtue converts the link to `Text` with the last value; a bought-plus-granted duplicate with a link into it resolves `Ambiguous` and raises the Fluent issue, never picking either value | Engine-only; no UI yet — an inert-but-correct intermediate state |
| CV6 | `AbilityParameterOptions` derived output (§ 6.3), excluding ambiguous sources from `linked` | **M** | a character holding Craft Guild Training with `guild` set sees it in `organization_lore`'s option list; an ambiguous source is not offered | Engine, no UI |
| CV7 | Picker consumes § 6.3's options (§ 6.1); "follows the Virtue" / unresolved-link indicators (§ 6.4); unlink-on-reselect; removal AND clear-to-empty flows call § 5.5; **pulled forward from CV8**: `sameParam`/`normalizeParam`/`resolvedLinksFrom` (§ 6.2), `LinkTarget.resolved` (a carried-through `AuthorizedAbility::instance`, not a new resolution), and real localized catalogue names merged into `LocalizedRuleset.i18n` at load (§ 2.3/§ 6.4) | **M/L** | `AbilityTab.client.test.ts`: selecting a linked option writes `{item,param}`; removing the source Virtue converts it live; `sameParam` unit tests; `AbilityTab.test.ts`: a catalogued value shows its localized name, never the raw id as visible text | `.client.test.ts`. **[decided, 2026-09-27]** A live picker cannot ship without the three pulled-forward pieces: it would otherwise render a raw/humanized-guess id (violating "never render a raw id as a label") and silently break bonus/floor matching the instant it writes a `Linked` value — CV8's own boundary assumed an inert interim state CV7 no longer leaves behind. `AbilityParameterOptions.linked[].resolved` reuses `ability_authorizations`' already-computed value; no new engine resolution pass. |
| CV8 | Remaining display/export parity (§ 6.4, § 6.6's full site table) not already covered by CV7's `abilityParamDisplay`/`sameParam` rewiring — the `export/resolve.rs` fallback chain (already benefits from CV7's `LocalizedRuleset.i18n` merge, but its own humanize-fallback code path is unchanged) + `export_golden.rs`'s byte-identical fixture check + any `.parameter` comparison site § 6.6 lists that CV7's own `AbilityTab.svelte`/`ParameterPicker.svelte`/`MagusMinimumAbilities.svelte` updates did not reach | **S** | export never prints a raw id or `(item,param)` pair; `export_golden.rs`'s fixture stays byte-identical | Narrower than originally scoped — CV7 closed most of "no raw id ever rendered" for this surface already |

Cross-cutting: `crates/arm-rules/tests/rulebook_citations.rs` and
`source_citations.rs` gates apply to every new comment/citation CV1–CV8 add.

---

## 11. Risks and open points for Norbert

**Both decided by Norbert, 2026-09-27.** Nothing remains open.

1. **§ 1.2: catalogue the five universal Educated-family values (four
   languages plus Merchant) now.** They land in CV1.
2. **§ 4: the hint is conditional.** It shows only when the free-text choice
   actually costs the player something, i.e. a literal or bound instance on one
   of the character's own items targets this Ability and is unmet. Otherwise no
   hint is shown, so Craft and Area Lore rows carry no permanent noise. The
   engine decides it and surfaces it with the Ability's options (§ 6.3); the UI
   does not query the ruleset itself. This replaces the note's earlier "static,
   always shown" recommendation.

---

## References read

`CLAUDE.md`; `crates/arm-rules/src/types.rs` (`ParamValue`, `ParamGate`,
`AbilityRef`, `ParameterDomain`, `ParameterDef`, `SelectionParamValue`,
`AbilityScore`, and the fixtures at `:5909,7381,7387`);
`crates/arm-rules/src/ability.rs` (`Ability`); `crates/arm-rules/src/migration.rs`
(`load_entity_migrating`, the `aging_reductions`/`talisman_attunements`
pre-pass precedent); `crates/arm-rules/src/life_stage.rs` (`AbilityRequirement`,
the `:1491` test fixture); `crates/arm-rules/src/ruleset.rs` (`Ruleset`,
`LocalizedRuleset`, `ScholarlyLanguageRequirement`);
`crates/arm-rules/src/validation/authorization.rs` (`validate_academic_language`);
`crates/arm-rules/src/validation/selections.rs` (`validate_duplicate_selections`,
`validate_total_selection_cap`); `crates/arm-rules/src/childhood.rs`
(`raise_score`); `crates/arm-rules/src/effective/ability.rs`, `effective/xp.rs`;
`crates/arm-rules/src/derived.rs`, `effective.rs`, `export.rs`,
`export/sections.rs`, `completeness.rs` (fixture/read-site counts);
`crates/arm-rules/tests/{data_integrity,book_templates,selection_param_value,
save_invariant,export_golden,core_type_conformance,d48_instance_pools,
roundtrip_proptest}.rs`; `crates/arm-app/src/ruleset_io.rs`, `commands.rs`
(`ruleset_guard`, `require_loaded`, `AppError::NotLoaded`, `load_entity`,
`load_ruleset`); `rules/core/abilities.json`, `virtues_flaws.json`
(`virtue.craft_guild_training:3748-3757`, `virtue.forge_companion:4301-4311`);
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md`
(3560-3730, 3610-3617, 3922-3928, 4062-4065, 4449-4470); `rules/source/de/Ars
Magica Definitive Edition Basisregeln.md` (matching regions, lines 3565, 3615,
3721, 3725, 3729, 4453, 4461, 4469, 4908 specifically verified);
`rules/source/de/translation-tables/{grundbegriffe, orden-tribunale,
reputationen, islamische-begriffe, juedische-begriffe}.md`;
`docs/vf-audit/decisions.md` (D2, D9, D10, D14, D34, D35, D48);
`docs/vf-audit/corrections.md` § 3.2a; `docs/vf-audit/design-c0-parameter-model.md`
(§ 3's `no_gated_authorization_item_is_ever_granted` precedent, § 10's bump
table, both amended in this review); `examples/magus_sample.json`;
`ui/src/lib/components/{AbilityTab,ParameterPicker,MagusMinimumAbilities}.svelte`;
`ui/src/lib/derive.ts`; `ui/src/lib/types.ts`.
