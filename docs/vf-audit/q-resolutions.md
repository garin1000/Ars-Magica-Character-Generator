# V/F audit — resolutions for the 31 evidence-settleable questions

Answers to the questions marked **R** (rules read), **C** (code read) or a mix in
`corrections.md` § 6 — i.e. the ones that do **not** need Norbert's ruling. Each
was re-derived from `rules/source/en/`, `rules/source/de/`, the shipped
`rules/core/` + `rules/i18n/` data and, where the question is C-flavoured, the
consuming code. `decisions.md` D1-D31 are binding and are cited where they
decide or constrain an answer.

**Nothing here is implemented.** Every entry states what it obliges so a Phase 2
slice can pick it up.

Written incrementally; the file is valid at every point.

---

## Independent verification and reconciliation

Every answer below was re-derived **independently** by a second pass that could not
read this file and worked only from `decisions.md`, `corrections.md` § 6, the batch
records and the sources. The two derivations agreed on **24 of 31**. The seven
divergences are resolved here, each against evidence I re-opened myself; where a
section below is superseded, this section governs and the section carries a
pointer.

**Both passes independently reached the same verdict on the two largest findings**
— Q-116 (the combat-scores reading, *and* its reduction from six entries to two)
and Q-109 (Berserk's flat fold is the defect, not Fury's silence). That
convergence is worth more than either pass alone.

### The seven divergences

**1. Q-81 — I was wrong; the data is right and the label is the defect.** I
concluded all three `casting_fatigue` amounts were inverted. The second pass
pointed at `derived/combat.rs::fatigue_levels`, which I had not read: *"A positive
delta reduces magnitude; never flip a penalty positive"*, `penalty: (base +
delta).min(0)`. I then took the census myself — **twelve `health_mod` rows across
ten entries** — and **every one** obeys *positive = better for the character*,
including the two computed penalty tracks where the arithmetic proves it
(`enduring_constitution` +1 / `low_tolerance` −1 on tracks literally named
*penalty*). So the convention is deliberate and catalogue-wide; the doc comment on
`types.rs::HealthTrack::CastingFatigue` is the single outlier.

**What survives from my section, and it is the part that matters:** both passes
independently found that **the user reads a backwards number today**. The Fluent
label `Casting fatigue` / `Zauber-Erschöpfung` names a *cost*, so a Withstand
Casting magus — a Virtue — is shown *"Casting fatigue: +1"*. **Revised remedy:**
fix the doc comment and rename the label's value in both locales (e.g. to a
resistance/benefit wording), and **leave the three amounts alone**. That is
cheaper, touches no shipped data, and preserves the family consistency the census
just proved. My "flip three amounts" is the rejected alternative.

**F-352 is unaffected and confirmed by both passes** (ArMDE:7003 / :5269 state the
Vulnerable/Withstand exclusion verbatim on both sides). The second pass adds a
third point I missed: both passages explicitly permit repeats (*"Vulnerable
Casting (2)"*, *"Withstand Casting (3)"*), so both entries are in D10's 42 and need
an explicit `max_total`.

**2. Q-39 — my formula is right, my impact table was wrong.** `charged_cost`
floors where the book ceils; that stands, and both passes derived the same
correction `(T − 1)·den/num + 1`. But I tabulated by *spend* `S`, where the engine
is exercised by *table cost* `T`, and the two give different answers about **who is
affected**. Re-derived against the shipped tables:

- **`virtue.linguist` (5/4) is clean at every reachable value.** Every Ability cost
  in `abilities.json` is a multiple of 5 (5, 15, 30, 50, 75, 105, 140, 180, 225,
  275), and at `T = 5k` both formulas give `4k`. So Q-39 **as asked** — do the two
  roundings agree for Linguist? — answers **yes**.
- **`virtue.affinity_ability` (3/2) is overcharged by 1 XP at Ability scores 1, 4,
  7, 10, 13, 16, 19** — the costs `≡ 2 (mod 3)`. Worked: Ability 4 costs 50; the
  engine charges `ceil(100/3) = 34`, where 33 suffices because
  `ceil(33·3/2) = 50`.
- **Arts are clean** — triangular costs are never `≡ 2 (mod 3)` — and **Flawless
  Magic's 2/1 is clean**, which both passes had.

**Revised:** the defect is real and is a code fix, but it is on Linguist's
*sibling*, not on Linguist. The red that matters is Ability 4 with
`virtue.affinity_ability`, plus ArMDE:2443 as the regression guard.

**3. Q-50 — both passes agree Nephilim is correct, and the second pass found a
`high` defect on its two siblings that neither Q-50 nor any finding covers.** I
verified it myself rather than relaying it. ArMDE:2638: *"you may take up to **ten
points of Flaws** … Most Mythic Companion Virtues require you to take some
particular Virtues and Flaws, **these count against your maximum** of 20 points of
Virtues and 10 points of Flaws."* ArMDE:2664 (Devil Child): the compulsory Flaw is
Tragic Life (Major, 3 points), and *"may take an **additional seven** points of
Flaws"* — 3 + 7 = **10**, exactly :2638's maximum. So the seven is the **remainder
inside** the ten, not a bonus on top of it.

`validation/balance.rs::effective_budget` computes `flaw_ceiling = base +
bonus_flaw`, so `bonus_flaw_points: 7` gives `mythic_type.devil_child` a ceiling of
**17** Flaw points and **34** Virtue points against the book's 10 and 20.
**`mythic_type.spirit_votary` carries the same 7** and the second pass cites
RoP:M:5486 for the identical structure — I did **not** open that line, so that half
is relayed, not verified. `bonus_free_virtue_points: 3` on Devil Child is a direct
transcription of *"three more points of Virtues at no cost"* and **stands**.

**New finding, `high`:** drop `bonus_flaw_points: 7` from `mythic_type.devil_child`
and (subject to checking RoP:M:5486) from `mythic_type.spirit_votary`. Two of the
four Mythic Companion types are over-budgeted by 7 Flaw / 14 Virtue points.
ArMDE:2638's own "21 … 20" is a book erratum for D25's note (the free Minor Virtue
is the 21st point).

**4. Q-25 — the second pass found the passage I missed, and its classification is
better.** I left `virtue.guest_of_house_criamon` as `narrative` on D22's
`flaw.seeker` precedent. ArMDE:2264 defeats that: *"**Membership in a House grants a
particular benefit at character creation** … A magus can only be a member of one
House."* Membership and the creation benefit are one thing, and ArMDE:4039
**decouples** them — politically Criamon, created under another House's rules.
`Entity` has one `house` field and structurally cannot hold that, so under **D3**
it is `uncomputed_rule` with the rule written out, never `narrative`.

**Revised:** `narrative` → `uncomputed_rule`, plus the description in both locales.
My two prerequisites (`Prereq::IsMagus` under D12; `Prereq::Nor([House(criamon)])`
for *"any **other** House"*) stand and are complementary — the second pass did not
raise them.

**5. Q-115 — the second pass is right that a warning is owed; I was too strict.** I
read *"which normally means that they must be magi of House Criamon"* as a gloss
stating no rule, and concluded nothing is owed. But **D16 closed Q-139 on the
identical wording** — ArMDE:6957's *"is **generally** restricted to magi of House
Verditius"* → warning — with the standing instruction to *"apply this wherever the
book hedges, rather than escalating each instance"*. ArMDE:6320 is that shape.

**Revised:** one validation rule — holds `flaw.inscribed_shadow` and is not House
Criamon → **warning**, never an error; reachability per D2, so the check must see
*granted* Flaws. The absolute half (stigmata, unmodelled) stays text, which is
what ships. The second pass's sizing sweep is also better than mine: widening the
terms to include *"generally/normally restricted"* returns **5** hits, of which
**2** are restrictions on an entry — ArMDE:6320 and ArMDE:6957 — so the population
is two, not one. My residual note stands and is now live rather than latent:
**`Prereq` has no warning severity**, so this rule needs a carrier that does not
exist at the entry level.

**6. Q-103 — the second pass measured what I only asserted, and the numbers change
the shape.** By `jq`: `special` contains **exactly one** Virtue (`virtue.the_gift`),
so that arm names precisely the entry ArMDE:6082 names; `supernatural` = 77, the
stated class; `hermetic` = **56**, where the book names **one**
(`virtue.diedne_magic`). So **two of the three arms are already exact** and the
over-permission is confined to one arm and is **55 entries**. It also found
ArMDE:6096 as the warrant for `forbid_tainted: true` (*"cannot apply to
Supernatural Virtues that are affiliated to the Infernal realm in the first
place"*), which I had not sourced.

**Revised:** my "the whole shape needs D23's predicate plus a new data property"
overstates it. The cheap exact fix is an id-whitelist on the `hermetic` arm naming
`virtue.diedne_magic` — one optional `ParameterDef` field, landing beside D14's
parameter work. **Against that:** ArMDE:6082 says *"**like** Faerie Blood, Diedne
Magic, or even The Gift"*, and *"like"* is open, so a whitelist of one is closed
where the book is open. Both passes agree the status quo (56) is wrong. **This
becomes a narrow cost call rather than a settled remedy** — see the revised
decision list below.

**7. Q-29 / Q-38 — the two passes reached opposite answers and neither has decisive
evidence. This is the one I am flagging rather than settling.** I read ArMDE:4277's
bare *"See Greater Immunity, page 83"* as carrying the whole entry, including
:4015's repeat clause, because the pointer must carry the substantive rules or the
entry is unplayable. The second pass leans **once**, on two grounds I had not
weighed: ArMDE:2814 requires a repeat be allowed *"**only if the description
explicitly allows it**"*, and a bare pointer is not explicit; and the Lesser/Greater
**Power** pair is a counter-precedent — ArMDE:4283 has `virtue.lesser_power` state
its own repeatability (*"This Virtue may be taken more than once, and the levels
added together"*) rather than inherit Greater Power's.

I verified ArMDE:4283 and it says what the second pass says. The parallel is not
exact — Lesser Power's body is self-contained and needs no pointer, Lesser
Immunity's is one sentence plus a pointer — but it is real evidence and it cuts
against me.

**Resolution: D10 breaks the tie toward *once*.** D10's whole point is that absent
means once and a repeat must be **declared**; reading a repeat into a bare pointer
is exactly the inference D10 was written to stop. Adopt **once** for
`virtue.lesser_immunity`, record the reasoning in `RULES.md` so the pointer is not
re-litigated, and note it as the audit's one genuinely arguable reading.
**Both passes agree on the part that matters:** `virtue.greater_immunity`'s
`max_per_target: 255` with **no parameter** licenses 255 *identical* copies where
ArMDE:4015 says *"a different immunity each time"*, so F-141's parameter is the
precondition either way.

### One of the second pass's findings does **not** survive

It reported, at high confidence, that *"all 21 Supernatural Abilities are buyable
by any character with ordinary XP and no Virtue"*, because `supernatural` is absent
from `abilities.json`'s `categories_requiring_virtue`. **That is false.** The gate
exists and is per-Ability rather than per-category, exactly as
`validation/authorization.rs`'s doc comment states: `validate_supernatural_abilities`
(`validation/scores.rs`) collects every bought Supernatural Ability not covered by
an `ability_score_grant` floor, skips the free Gift slots, and raises
`CODE_SUPERNATURAL_ABILITY_REQUIRES_VIRTUE` on the remainder — and it **is** called,
from `validation/mod.rs`. Reading the absent category without reading the validator
that replaces it produced a false alarm on the app's most load-bearing gate.

**Consequently Q-41's verdict stands as written below**, against the second pass's
recommendation to drop `supernatural` from `virtue.lone_redcap`'s pool: the list is
all five categories, i.e. the *absence* of a funding restriction, which is what
ArMDE:4848's second sentence states, and the permission half is enforced elsewhere.

### Corroborations worth keeping from the second pass

- **Q-116** — the German preserves the distinction exactly: DE:6332 *Kampfwerte*
  against DE:16656's *"fünf **Kampfwerte**"*, and DE:6262 *Kampfwürfe*. That is
  independent confirmation that *scores* and *rolls* are two terms, not one. It
  also names the test that must go red first:
  `data_integrity.rs::the_combat_roll_flaws_penalize_the_combat_ability_totals`,
  which asserts none of them touches Initiative.
- **Q-70** — a decisive argument I missed: `flaw.false_power`'s parameter carries
  `forbid_tainted: true`, so marking `virtue.sense_passions` tainted would make
  **False Power (Sense Passions) unselectable** — the very arm ArMDE:7737 names
  first. A boolean cannot hold the passage's disjunction.
- **Q-78** — a second boundary: ArMDE:16636, *"Virtues that affect aging rolls do
  not affect crisis survival rolls"*, so the rule must **not** be routed through
  `AgingEffect::CrisisSurvival`. And ArMDE:16619 confirms the Decrepitude gain
  happens before the table roll and is not voided.
- **Q-117** — `jq` confirms `flaw.horrifying_appearance_snake_legs` is
  `supernatural` + `narrative`, i.e. one of **D8's 48** by D8's own enumeration
  rule, so the reclassification is already ruled and the exemption row already
  contradicts it. (D8's third support covers only the 33 Virtues in that cohort;
  supports 1 and 2 cover all 48.)
- **Q-68** — `RULES.md`'s residual **clamp at 0** already implements exactly the
  reading both passes reached, so nothing mechanical is owed; only `RULES.md`'s
  justification needs re-founding on ArMDE:5075 / :4824.
- **Q-42** — three further unencoded absolutes on `virtue.redcap`, which no
  question raises and which compound D17: ArMDE:4848 *"you have the **Well-Traveled
  Virtue** at no cost"* (Lone Redcap ships that grant; Redcap ships none), and
  ArMDE:4850 *"you **cannot** take the Wealthy Virtue or Poor Flaw"* and *"You
  **may not** take The Gift"*. All three are expressible today and all three are
  absent. The second pass also argues the Redcap/Lone Redcap pair can be encoded as
  a plain `incompatible_with` without ArMDE:2816's machinery, since ArMDE:4325's
  compatibility note is scoped to *mundane* statuses and ArMDE:4844 makes Redcap
  non-mundane — that is a fair reading, though it still routes through :2816's
  default and so remains entangled with Q-132.
- **Q-138** — a materially different reading of ArMDE:6148 that would change what
  D23 does. See the revised note below.

### What changed in the decision list

| Q | Was | Now | Why |
|---|---|---|---|
| Q-25 | `narrative` stands | **`uncomputed_rule`** | ArMDE:2264 + D3 |
| Q-29 / Q-38 | pointer transfers → repeatable | **once**, flagged as arguable | ArMDE:2814 "explicitly" + ArMDE:4283 precedent; D10 breaks the tie |
| Q-39 | Linguist diverges | **Linguist is clean**; `virtue.affinity_ability` overcharged at 7 scores | re-derived against the shipped cost tables |
| Q-41 | list is correct | **unchanged** | the second pass's counter-argument rests on a false premise |
| Q-50 | nothing owed | nothing owed **on Nephilim**; new `high` defect on its two siblings | ArMDE:2638 + :2664 |
| Q-81 | flip three amounts | **fix the doc comment + the Fluent label** | 12-row census proves the data convention deliberate |
| Q-103 | blocked on D23 + new data property | **narrow cost call**: one-id whitelist vs D23's predicate | `special` = 1, `hermetic` = 56, over-permission = 55 |
| Q-115 | nothing owed | **a warning is owed** | D16 closed Q-139 on identical wording |

### Three narrow decisions for Norbert

Neither pass could settle these from the source, and both agree they are cost or
policy calls rather than readings. Each is one sentence.

1. **Q-103** — *"Tighten `flaw.false_power`'s `hermetic` arm from all 56 Hermetic
   Virtues to the one the book names: add a one-id whitelist to `ParameterDef`
   (exact for today's catalogue, closed where ArMDE:6082's "like" is open, ~1
   field, lands beside D14), route it through D23's predicate (open, but needs a
   property the catalogue does not carry), or keep 56?"*
2. **Q-138** — *"Is ArMDE:6148's 'Any Flaw that is only appropriate to Hermetic
   Magic cannot be taken with this Flaw' a constraint on the Major Hermetic Flaw
   that Flawed Powers **imports**, or a plain incompatibility with Flaws the
   character may hold?"* **D23 assumed the second**, and the second pass makes a
   strong case for the first: the two named examples (Deficient Technique,
   Unstructured Caster) are exactly the Hermetic Flaws that cannot be *re-targeted*
   at Supernatural Virtues, and the incompatibility reading would forbid a magus
   with Flawed Powers from holding Deficient Technique as an ordinary, unrelated
   Flaw — which ArMDE:6148's own *"rather than to her Hermetic magic **(if any)**"*
   shows the passage contemplates. The grammar (*"taken **with** this Flaw"*)
   favours D23's reading; the semantics favour the other. **This is worth putting
   because it changes what D23 builds**, not only where it is pointed.
3. **Q-67** — *"`virtue.simple_student` grants 30 XP per finished year with no year
   count stated: ship no pool and put the rule in `description` (my recommendation
   and, I think, already forced by D3), or ship 60 (the bound before the character
   becomes a Baccalaureus, whose 90 = 3 × 30 is derived the same way)?"* Both passes
   agree the number is not derivable; we differ only on whether D3 already
   forecloses inventing one. The second pass leans 60.

---

### Q-16 — `virtue.dust_devil`: how much of Skinchanger does "this variant" inherit?

**Verdict.** Reading B. The +3 Soak does **not** transfer, because the clause
that carries it has no operand in a dust devil; the *item* rules do transfer,
because the entry explicitly substitutes the item rather than dropping it. The
shipped data (`uncomputed_rule`, no effects, own passage in both locales) is
correct and no `soak_mod` is owed.

**Evidence.** ArMDE:4974, the Skinchanger clause at issue, verbatim: *"Skinchangers
may transform into any non-magical **animal** between Size -10 (robin) and Size +2
(bear). The character has the normal physical characteristics of **the animal**,
except that +3 is added to the character's Soak score (in animal form only)."*
Both halves are quantified over *animals*. ArMDE:3709 replaces the target with
*"a tamzawit, or dust devil, **a small whirlwind carrying dust**"* — which is not
an animal, is not in the Size -10…+2 animal band the same sentence defines, and
has no "normal physical characteristics of the animal" to inherit. The Soak
clause is not overridden; it simply has nothing to attach to.

The item half goes the other way. ArMDE:3709: *"**Rather than a cloak or animal
skin**, the focus object of this Virtue is a glass amulet…"* — the entry names
Skinchanger's item slot and swaps its content, which presupposes the slot and its
rules (stolen → Arcane Connection; destroyed → remade over a season, ArMDE:4974).
It overrides only what it contradicts: *"can assume the form at any time"*
displaces the one-round-and-physical-contact clause.

The Dove variant is consistent with this and is not, as B02 feared, merely one
data point. ArMDE:4978 declares itself *"a more powerful variant of the Minor
Virtue above"* and at ArMDE:4984 restates the Soak (*"save that her Soak is +3
higher than usual"*) — and it has to, because :4984 also replaces "the normal
physical characteristics of the animal" with *"the statistics of a common dove"*.
Having overwritten the sentence that carried the Soak, it must restate the Soak.
Dust Devil overwrites the same sentence and does **not** restate it, which under
the book's own drafting habit is the absence of the bonus, not an omission.

**What it obliges.** Nothing. `virtue.dust_devil` stays `uncomputed_rule` with no
effects. Q-16 closes without touching data, and F-62's cross-reference-following
instruction is satisfied for this entry.

**Status.** SETTLED

---

### Q-18 — `virtue.gentle_gift`: does the book state the `has virtue.the_gift` prerequisite?

**Verdict.** Yes — not in the entry's own body, but in a general rule that binds
it twice over. The shipped prerequisite is **correct**, and Reading B (over-strict)
is refuted.

**Evidence.** `virtue.gentle_gift` ships `categories: ["hermetic"]`. ArMDE:2880,
the `### Hermetic` type definition, verbatim: *"**Only characters with The Gift
can take these Virtues and Flaws**, and some are only applicable to Hermetic magi
who have already completed their training."* ArMDE:2840 says the same from the
character-type side: *"You may not take Hermetic Virtues and Flaws, **unless you
have The Gift** (this would be highly unusual)."* Both are absolute in D16's
sense ("only", "may not"), so the gate is a hard error, which is what ships.

**The routing sentences do not say what Reading B needs them to.** ArMDE:4139,
inside `#### Inoffensive to (Beings)`: *"You may not take this Virtue more than
once; characters who are Inoffensive to more than one type of being should take
Gentle Gift instead. UnGifted characters may take **this** Virtue only if they
have the Flaw Magical Air."* The unGifted clause restricts *Inoffensive*, not
Gentle Gift, and it is a separate sentence from the routing clause — nothing
joins them into "an unGifted character may take Gentle Gift". ArMDE:4141 —
*"Inoffensive to Mundane Humans is not available as a Minor Virtue; take Gentle
Gift instead"* — points at a target ArMDE:4135 does not even list among
Inoffensive's five classes (*"animals, divine beings, faeries, demons, or magical
creatures"*), so it is a redirection for a Virtue that does not exist, not a
statement about who may hold Gentle Gift. Both routing clauses use *"should"* /
*"take … instead"*, which D16's table maps to advice; ArMDE:2880's *"only"* is the
absolute, and an absolute beats advice.

**Corroborated by an existing ruling.** D12 names *exactly* the three entries
that gate on `virtue.the_gift` — `flaw.blatant_gift`, `virtue.gentle_gift`,
`flaw.suppressed_gift` — as the **intrinsic** class and says *"the catalogue
already had the right instinct here"*. Q-18 is that instinct's warrant.

**What it obliges.** Nothing on the prerequisite. Two side effects worth
recording: (a) `virtue.gentle_gift` is D12's worked **intrinsic** case, so D12's
classification pass must not hand it an `IsMagus` gate — The Gift alone is the
right gate; (b) ArMDE:2880 is the general warrant for every `hermetic` entry's
Gift requirement, and no entry currently cites it, so D12's pass should cite it
once rather than re-deriving it 122 times. F-81's reclassification is unaffected
and stands.

**Status.** SETTLED

---

### Q-20 — `virtue.fidai`: does "choose the Social Status he is pretending to have" permit a second Social Status?

**Verdict.** No. The pretended status is fiction and takes no second Social
Status Virtue. But the sentence **is** a stated choice, so under D9 it obliges a
parameter — which is the finding B03 was reaching for and did not name.

**Evidence.** ArMDE:2816 states the escape hatch and the exact test it applies:
*"All characters must take one Social Status, and may only take more than one if
the descriptions of the Virtues or Flaws **explicitly note that they are
compatible**."* The book has a settled idiom for meeting that test, and it is the
word *compatible* — ArMDE:4325, `virtue.lone_redcap`: *"This social status is
**compatible with** any other mundane Social Status Virtue that would reasonably
allow you to do your job as a Redcap, such as Merchant or Mendicant Friar."*
ArMDE:3881 does not use it: *"Such a character should have a Story Flaw
representing his mission, and choose the Social Status he is **pretending** to
have."* Nothing is noted as compatible, and *pretending to have* is the negation
of *having*. The German is line-parallel and equally unambiguous (DE:3881: *"und
den Sozialen Status wählen, den er **vorgibt** zu haben"* — *vorgeben* is to
feign, not to hold).

The mechanical test settles it too: a Social Status Virtue costs or grants points
and confers benefits (Fida'i's own is *"Fida'i may take Martial Abilities at
character creation"*, ArMDE:3879). A pretended status confers none of that — the
player is choosing a disguise, not buying a Virtue.

**What it obliges.** Two things, neither of them a second Social Status.

1. **A parameter, per D9.** ArMDE:3881 tells the player to choose, and D9 rules
   that *every* stated choice is recorded. The choice is over the Social Status
   catalogue, so this is a `ref` parameter into the `social_status` category, not
   free text. D9's § 3.7 list does **not** carry `virtue.fidai` (checked: the only
   `corrections.md` row naming it is F-68), so this is a new instance for the
   catalogue-wide re-derivation D9 requires.
2. **Nothing on `incompatible_with` or on ArMDE:2816.** Fida'i is an ordinary
   single Social Status and needs no special treatment when Q-07/Q-102/F-427's
   machinery lands.

F-68 (the missing `ability_authorization` for Martial) is independent and stands.

**Status.** SETTLED

---

### Q-25 — `virtue.guest_of_house_criamon`: is "politically Criamon, created by another House's rules" a mechanical rule?

> **REVISED — see § Reconciliation, divergence 4.** ArMDE:2264 makes this
> `uncomputed_rule`, not `narrative`. The two prerequisites below stand.

**Verdict.** The *membership* half is not mechanical and the `narrative`
classification stands. But the passage states **two eligibility facts** the data
does not encode, and both are expressible with existing `Prereq` variants.

**Evidence.** ArMDE:4039, the entry's whole body: *"**Magi** with this Virtue are,
politically, members of House Criamon, but may be created using the rules for
**any other House**. Guests are offered membership, which Criamon see as a
political formality, for many reasons. The troupe and player should determine why
the character found it necessary to find sanctuary with this House."*

What happens when a player does this today: he picks any House — **including
Criamon** — and takes the Virtue. If he picks Criamon he receives
`house.criamon`'s grant (`virtue.the_enigma`, `rules/core/houses.json`) *and*
claims guest status, which is the one configuration the words "any **other**
House" exclude. Nothing objects. And a **grog or companion** may take it too:
`categories: ["hermetic"]` is not a magus gate (D24 establishes this precisely),
so a Gifted companion can hold a Virtue whose first word is *Magi*.

Political House membership has no modelled consequence — the app computes no
Tribunal politics — so the classification is right: the *mechanics* come from the
created-under House, which is exactly what an effect-less entry produces.

**What it obliges.**

1. **`Prereq::IsMagus`.** House membership exists only through Hermetic training,
   so this is **trained** under D12's criterion and falls inside D12's
   classification pass over the 122 `hermetic` entries. Not a separate slice.
2. **`Prereq::Nor([House("house.criamon")])`.** "Any other House" is stated, not
   inferred — the word *other* is in the text — and `Prereq::House` + `Prereq::Nor`
   already exist, so no machinery is needed. Note the D2/F-466 trap: a House is
   *assigned*, not *bought*, so this must be a `Prereq` and not an
   `incompatible_with` row, which reads bought selections only.
3. **Classification unchanged.** D22 set the precedent on `flaw.seeker`: *"the
   prerequisite is an eligibility fact rather than a rule the entry states"*, so
   it stays `narrative`. Under D5 nothing is owed in `description` either — there
   is no uncomputed mechanical clause left once the two gates are data.

**Status.** SETTLED

---

### Q-29 / Q-38 — do Greater and Lesser Immunity disagree about repeatability, and does the repeat clause travel through the cross-reference?

> **REVISED — see § Reconciliation, divergence 7.** The two derivations reached
> opposite answers; D10 breaks the tie toward **once** for Lesser Immunity. This
> is the audit's one genuinely arguable reading. The data defect below stands
> either way.

Answered together: Q-29 asks which of the two passages is right and Q-38 asks
whether the pointer carries the clause. They are one question.

**Verdict.** They do **not** disagree. ArMDE:4277's *"See Greater Immunity"* is an
unrestricted pointer to the whole entry, so the repeat clause transfers and
**both** are repeatable, each with a different immunity. The shipped data is
wrong on both entries, in opposite directions.

**Evidence.** ArMDE:4275-4277 is `#### Lesser Immunity`'s entire body: *"You are
immune to some hazard which is either rare, or not deadly, or both. See Greater
Immunity, page 83."* One sentence of content plus a bare pointer — it does not
restate the immunity rules, the mundane-and-magical rule, the deprivation
worked example or the aging exclusion, so the pointer is doing **all** the work
beyond "rare, or not deadly, or both". A pointer that must carry ArMDE:4011's
*"This immunity applies to mundane and magical versions of the thing"* in order
for the entry to be playable at all cannot be read as stopping four lines short
of ArMDE:4015's *"You may take this Virtue more than once, with a different
immunity each time."* There is no scope marker, and the alternative reading has
to supply one the book does not write.

Two further supports. **(a)** The pointer's page target is the Greater Immunity
entry as a whole, and ArMDE:4015 is inside it. **(b)** The rule the pointer would
have to *not* carry is the one ArMDE:2814 makes the catalogue-wide default
against: *"A Virtue or Flaw may be taken more than once **only if the description
explicitly allows it**."* Lesser Immunity's description is, by its own
construction, Greater Immunity's — so under D10's inverted default the pointer is
the only thing that can license a repeat, and reading it narrowly would make
Lesser Immunity the one Virtue whose description is deliberately incorporated by
reference *except* for the sentence that matters.

**What the data says, and why both entries are wrong.** `virtue.greater_immunity`
ships `max_per_target: 255` and no parameter; `virtue.lesser_immunity` ships
neither. So today the Major one permits 255 **identical** copies (the book says
*"a different immunity each time"*) and the Minor one permits exactly one
(the book permits many). Neither matches.

**What it obliges.**

1. **F-141's missing parameter is the precondition for both**, exactly as B05
   said. Without a `text` (or `enumerated`) parameter naming the hazard, "a
   different immunity each time" is unrepresentable, and `max_per_target` has no
   target to count.
2. **Both entries then take the same shape**: parameter = the hazard;
   `max_per_target: 1` (a different one each time); `max_total` explicitly > 1,
   declared per D10 because under the inverted default absent means once. This is
   one of the 42 entries D10's pass must read, and it is one that **does**
   legitimately repeat.
3. **F-94 is confirmed and Q-29 stops being a contradiction.** The two passages
   never disagreed; the *data* did.

**Status.** SETTLED (both)

---

### Q-33 — the book gives Hermetic Prestige two different levels; the data picks one

**Verdict.** The data is right: `score: 4` follows the entry's own descriptor.
The level-3 figure is in a **worked example**, and under D25's precedent the
substantive statement beats the secondary listing. The disagreement is a known
source erratum and belongs in `RULES.md` with the others.

**Evidence.** ArMDE:4073, `#### Hermetic Prestige`'s whole body, ends: *"You gain
a Reputation of **level 4** within the Order."* ArMDE:2518, inside a block-quoted
`> #### Example: Darius of Flambeau` in the Reputations section: *"Darius does
have a Reputation, thanks to his Hermetic Prestige Virtue. It's a reputation with
Hermetic magi, and it has a **level of 3**. Niall picks 'Dedicated Hoplite' as the
content."* Verified: :2518 is inside the example's `>` quote block, and :4073 is
the entry's own body line. Shipped:
`{"type":"grants_reputation","kind":"hermetic","score":4}` — correct on both
`kind` (:2518's *"with Hermetic magi"* and :4073's *"within the Order"* agree) and
`score`.

**What it obliges, and this is the part Q-33 did not anticipate.**

1. **A `RULES.md` erratum row**, in the same note D25 opened for
   `flaw.weak_personality`: entry, both citations (ArMDE:4073 descriptor,
   ArMDE:2518 worked example), side taken. D25's shape is descriptor-vs-**index**;
   this is descriptor-vs-**worked example**, a second shape of the same defect, so
   the note should be worded to hold both rather than being Weak-Personality
   specific.
2. **D11 makes this user-visible, which it was not before.** D11 rules the
   granted `score` **exact and enforced**. So a player who types the book's own
   printed example character — Darius, Hermetic Prestige, Reputation level 3 —
   gets a validation error, and the error will look like an app bug because the
   book is open beside him. This is not grounds to add `max_score: 3..4`: D11
   surveyed all 31 granting passages and found *exactly one* stating a range
   (`flaw.outsider_*`, ArMDE:6554), and a worked example is not a range. But the
   erratum note must say so explicitly, or the next reader will "fix" it by
   widening the bound.
3. **No data change.**

**Status.** SETTLED

---

### Q-39 — `virtue.linguist`: the book rounds the XP up, the engine rounds the cost up. Do they agree?

> **REVISED — see § Reconciliation, divergence 2.** The formula below is right,
> but the impact table is not: it is keyed by *spend*, where the engine is
> exercised by *table cost*. **`virtue.linguist` is clean at every reachable
> value**; the live defect is `virtue.affinity_ability` at Ability scores 1, 4, 7,
> 10, 13, 16, 19.

**Verdict.** **No, they do not agree — and the divergence is not Linguist's.** The
engine *floors* exactly where the book *ceils*, on every fractional Affinity in
the catalogue. B05's reason for treating Linguist as a special case is false:
ArMDE:3374 words Affinity in the *same* XP-gain construction as ArMDE:4317, and
ArMDE:2443's worked example does not adjudicate between the two roundings.

**Evidence — the two passages are the same sentence.**

ArMDE:3374, `#### Affinity with Ability`: *"All Advancement Totals for one Ability
are **increased by half, rounded up**, as are **any experience points you put in**
that Ability at character creation."*

ArMDE:4317, `#### Linguist`: *"All Advancement Totals for any Language are
**increased by a quarter, rounded up**, as are **any experience points you put
into** any language at character generation."*

Word for word the same construction with a different fraction. So the premise
that *"Affinity is worded as a cost reduction … Linguist is worded the other way
round"* does not survive reading ArMDE:3374. Both are worded as a **gain**, and
neither is worded as a cost reduction. (ArMDE:3378, `#### Affinity with Art`, is
the same again.)

**Evidence — the arithmetic, stated exactly.** `effective/xp.rs::charged_cost`
computes `charged(T) = ceil(T · den / num)` for an Affinity `num/den`. A spend of
`S` therefore affords the largest `T` with `ceil(T·den/num) ≤ S`; since `S` is an
integer that is `T·den/num ≤ S`, i.e. **`T = floor(S · num/den)`**. The book
computes the effective points as **`E = ceil(S · num/den)`**. The two differ by
exactly 1 whenever `S · num/den` is not an integer, and by 0 otherwise — the
engine always takes the low side. Worked:

| Virtue | num/den | Spend `S` | Book `ceil(S·n/d)` | Engine `floor(S·n/d)` |
|---|---|---|---|---|
| Linguist | 5/4 | 10 | **13** | **12** |
| Linguist | 5/4 | 12 | 15 | 15 |
| Affinity with Art | 3/2 | 5 | **8** | **7** |
| Affinity with Art | 3/2 | 37 | **56** | **55** |

**Evidence — why ArMDE:2443 cannot be cited as endorsement.** The example reads:
*"He spends 37 points on Perdo, which his affinity turns into 56 points, so that
he has **Perdo 10 (1)**"*. Perdo 10 costs 55 on the triangular Art curve, so the
book's 56 buys score 10 **with one point left over** — which is precisely the
figure in the parenthesis. The engine charges `ceil(55·2/3) = 37` for score 10 and
leaves nothing over. Both reach *Perdo 10 for 37 spent*, so the example is
consistent with both roundings and discriminates between neither; the single
point of difference is exactly the leftover that `RULES.md` already records the
model as not representing. A4's citation of it as warrant for cost-side `ceil` is
therefore over-read.

**What it obliges.**

1. **A correction in `effective/xp.rs::charged_cost`, not in `virtue.linguist`.**
   The charge that inverts the book's `E(S) = ceil(S·num/den)` is the least `S`
   with `E(S) ≥ T`, i.e. **`charged(T) = (T - 1) · den / num + 1`** in integer
   floor division, for `T ≥ 1` (`T = 0` charges 0). Checks: `T=55, 3/2` → 37 ✓
   (matches ArMDE:2443); `T=13, 5/4` → 10 ✓ (matches the book's 10-point
   Linguist); `T=56, 3/2` → 37 ✓ (the point the current formula loses).
2. **Scope is every fractional Affinity, not one entry.** It touches
   `affinity_ability_cost`, `affinity_art_cost`, `group_affinity_cost` and
   Flawless Magic's mastery Affinity. Whole-number ratios (Flawless Magic's 2/1)
   are provably unaffected — the two formulas coincide when `den = 1` — so the
   observable change is confined to the 3/2 and 5/4 carriers.
3. **It raises caps and buying power**, so unlike D10 and D28 it invalidates no
   save; a character legal today stays legal.
4. **A finding is owed.** Q-39 is filed as a question against `virtue.linguist`
   and no `corrections.md` row rates the general defect. It is *wrong rules
   output*, which `CLAUDE.md` rates in the top class, though the magnitude is one
   experience point.

**Status.** SETTLED

---

### Q-41 — `virtue.lone_redcap`: is `supernatural` in the 300-point pool sourced?

> **CONFIRMED against a challenge — see § Reconciliation.** The second pass
> recommended dropping `supernatural`, on the premise that Supernatural Abilities
> are ungated. They are not: `validation/scores.rs::validate_supernatural_abilities`
> gates them per Ability and is called from `validation/mod.rs`. This verdict
> stands.

**Verdict.** The word is in neither passage — but the shipped list is
nevertheless **correct**, and removing `supernatural` would impose a restriction
the book does not state. B05's stated worry is answered by the engine itself.

**Evidence — what the passages restrict.** ArMDE:4848 (`virtue.redcap`, which
`virtue.lone_redcap` inherits through ArMDE:4321's *"You **still** begin with 300
experience points"*): *"You are trained in a similar manner to magi, and **may
take** Academic. Arcane, and Martial Abilities during character generation. You
have spent fifteen years as an apprentice, and gained a total of 300 experience
points in those fifteen years."* Two separate sentences: the first is a
**permission** over three gated categories, the second a **quantity** with no
category restriction attached to it at all. ArMDE:4321 restricts nothing either.
So the book places **no** limit on what the 300 may buy.

**Evidence — what the data says.** The shipped list is
`["academic","arcane","general","martial","supernatural"]`, which is *all five*
members of `AbilityCategory::ALL` (`ability.rs`, verified: General, Academic,
Arcane, Martial, Supernatural). An all-five list is not a restriction; it is the
absence of one — which is exactly what ArMDE:4848's second sentence states. The
three categories the *first* sentence names do double duty, because a restricted
pool implies permission for what it funds (`types.rs`, the note D13 quotes).
`general` and `supernatural` carry only the funding half and authorize nothing.

**Evidence — why the funding half is harmless.** `supernatural` is not in
`categories_requiring_virtue` (`rules/core/abilities.json`, which lists
`["academic","arcane","martial"]`), so
`validation/authorization.rs::validate_ability_authorization` never consults it;
Supernatural access is granted **per Ability** and enforced separately by
`validate_supernatural_abilities`. The engine's own doc comment at
`effective/xp.rs::magus_later_life_pool` states the consequence in as many words:
*"Supernatural stays in the set and legalizes nothing … an unauthorized one is
already an error and funding it here changes nothing."* So a Lone Redcap can
spend apprenticeship XP on a Supernatural Ability **only** if a Virtue already
granted him that Ability — which is the rules-as-played outcome B05 called
defensible, reached without stretching the text.

**What it obliges.** Nothing. Q-41 closes clean; do not remove `supernatural`.
Two records are worth leaving so this is not re-opened: the list is "all five",
not "three plus two extras"; and the same reasoning covers
`virtue.mentored_by_demons`, the only other entry on the identical list.
It does **not** cover `virtue.privileged_upbringing`, whose
`["general","academic","martial"]` is a genuinely *narrower* list and so is a
funding restriction that needs its own passage — that is Q-59, untouched here.

**Status.** SETTLED

---

### Q-42 — nothing encodes that `virtue.redcap` and `virtue.lone_redcap` are alternatives

**Verdict.** The book does make them mutually exclusive, but not by any sentence
in either entry — it does so through ArMDE:2816, which this repo models nowhere.
So the *rule* is settled and the *remedy* is blocked: this is one more instance of
Q-132/F-427's missing machinery, not a pair-specific `incompatible_with` row.

**Evidence.** ArMDE:2816: *"All characters must take one Social Status, and **may
only take more than one if** the descriptions of the Virtues or Flaws explicitly
note that they are compatible."* Both entries are `categories: ["social_status"]`.
ArMDE:4325 is `virtue.lone_redcap`'s compatibility note and it is **scoped**:
*"This social status is compatible with any other **mundane** Social Status
Virtue that would reasonably allow you to do your job as a Redcap, such as
Merchant or Mendicant Friar."* `virtue.redcap` is not mundane — ArMDE:4844: *"you
are a full member of the Order of Hermes and of House Mercere"* — so it falls
outside the note, and :2816's default (only one) governs. `virtue.redcap`'s own
passage offers no note in the other direction.

The pair is also incoherent on its face — ArMDE:4321 is *"You are a Redcap **who
does not maintain ties to a Mercer House**"* against ArMDE:4844's *"full member …
of House Mercere"*, and ArMDE:4848 gives Redcap a free Longevity Ritual that
ArMDE:4321 denies. But that is entailment, not a stated exclusion, and Q-19's
open policy question is precisely whether entailment licenses an
`incompatible_with` row. So it cannot carry the verdict on its own.

**What it obliges.**

1. **No pair-specific row.** Singling out this pair would encode a specific rule
   derived from a general one nobody has modelled, which is the failure mode
   `7f5605a` was reverted for. B05's instinct was right.
2. **File it against F-427 / Q-07 / Q-102 / Q-132** as a worked instance: when
   the category machinery lands, "at most one `social_status`, unless a
   compatibility note says otherwise" resolves this pair with no entry-level data.
3. **ArMDE:4325 is the one place in the catalogue that exercises the escape
   hatch** (checked against Q-20's survey of the idiom: the marker is the word
   *compatible*). Whatever shape :2816 takes must be able to express a
   **scoped** exception — "compatible with any other *mundane* Social Status" —
   not merely a boolean "may stack". That is a design constraint Q-132's ruling
   does not currently record, and it is the reason this row is worth carrying
   forward rather than closing as a duplicate.

**Status.** SETTLED on the rules question; the **remedy** is BLOCKED on Q-132 /
F-427.

---

### Q-53 — the book names six Ability types; `AbilityCategory` has five

**Verdict.** The five-member enum is **correct**, and making Spell Mastery a
sixth member would actively break a rule the book states. The divergence is
deliberate; the only defect is documentary, and it is smaller than Q-53 supposed
because `RULES.md` already records half of it.

**Evidence.** ArMDE:7143: *"There are **six** types of Ability: General Abilities,
Academic Abilities, Arcane Abilities, Martial Abilities, **Spell Mastery
Abilities**, and Supernatural Abilities."* `ability.rs::AbilityCategory` has five
(General, Academic, Arcane, Martial, Supernatural) and `AbilityCategory::ALL` is
`[AbilityCategory; 5]`.

**Why six would be wrong.** ArMDE:9516: *"Spell mastery Abilities are their own
category, and **Virtues that give characters access to other categories of Ability
do not cover spell mastery Abilities**. They may only be learned by characters who
use Hermetic magic to cast spells."* Every mechanism in the engine that ranges
over `AbilityCategory` would then sweep Spell Mastery in — most sharply
`effective/xp.rs::magus_later_life_pool`, which builds its eligibility from
`AbilityCategory::ALL` filtered by the gated set: a sixth member would fund Spell
Mastery from pre-apprenticeship later-life XP, which is exactly what :9516
forbids and what `PoolEligibility::Mastery` exists to prevent. ArMDE:9518 adds a
second, structural reason: *"For every possible Hermetic spell, there is a
corresponding Ability"* — an open-ended set no catalogue can enumerate, so Spell
Mastery cannot be a category *of catalogue entries* at all. It is modelled per
spell (`SpellSelection.mastery`), which is the shape the book describes.

**What `RULES.md` already has.** `RULES.md`'s *Mastered Spells / Flawless Magic*
section cites both ArMDE:9516 and ArMDE:7143, names Spell Mastery *"one of the six
Ability types"*, and records the per-spell modelling and its pricing. So the
divergence is *not* undocumented, contrary to Q-53's premise — what is missing is
the statement at the **enum**, where a reader counting members will notice.

**What it obliges.** A doc comment on `ability.rs::AbilityCategory` recording that
the sixth book type is deliberately absent, citing ArMDE:7143 for the six and
ArMDE:9516 for why the sixth must not be a member, and pointing at
`PoolEligibility::Mastery`. No data change, no behaviour change, no new finding
beyond that. Q-53's own conclusion — *"raised so the divergence is on the record"*
— is correct; this makes the record load-bearing.

**Status.** SETTLED

---

### Q-79 — the book files two Flaws under a magnitude their own descriptors contradict

**Verdict.** **D25 already decides this**: the descriptor wins, the shipped
`magnitude: "minor"` stands on both entries, and the disagreement is recorded as
a known source erratum. Q-79 is D25's obligation 2 — *"establish whether it is the
only one"* — answered for B10's span: it is not, and here are two more.

**Evidence, verified line by line.** The index heading `### Supernatural, Major`
sits at ArMDE:5387; `[Bound to (Role) Role]` is at ArMDE:5392 and `[Broken
Vessel]` at ArMDE:5393, both inside it. The `### Supernatural, Minor` list begins
at ArMDE:5534 and contains neither. The descriptors disagree: ArMDE:5736 reads
*Minor, Supernatural* and ArMDE:5754 reads *Minor, Supernatural*. Shipped:
`magnitude: "minor"` on both.

**Why the descriptor wins here specifically.** D25's reasoning is that the
descriptor is the entry's own substantive statement while the index is a link
line, and D25's obligation 3 adds that *"link hygiene is unreliable — the anchor
sweep found 134 link targets resolving to no heading"*. Both ArMDE:5392 and :5393
are literally `[Name](#anchor)<br>` link lines. The content test points the same
way as it did for Weak Personality: nothing in either passage is Major-sized —
ArMDE:5755, `Broken Vessel`, is a 5-experience-point loss on a zero and a
one-level loss on a botch, which is a Minor-scale mechanic.

**What it obliges.**

1. **No data change.** Both stay `minor`.
2. **Two more rows in D25's `RULES.md` errata note**, with both citations each
   (ArMDE:5736 vs :5387, :5392; ArMDE:5754 vs :5387, :5393). D25 opened that note
   for a *category* disagreement; these are *magnitude* disagreements, so the note
   must be worded to hold both axes rather than being category-specific.
3. **D25's sweep is now known to find more than one.** B19 found one in 25
   entries, B10 found two in its own span, and B16 found a third (Q-125, below) —
   so the one-off sweep D25 requires will return a list, not a single row, and
   should be budgeted as such.

**Status.** SETTLED (by D25; recorded here because Q-79 predates it and its row
still reads "open")

---

### Q-81 — which `casting_fatigue` sign convention is intended?

> **SUPERSEDED — see § Reconciliation, divergence 1.** This verdict is
> **withdrawn**. A 12-row census shows *positive = better* is deliberate and
> catalogue-wide, proved arithmetically by `derived/combat.rs::fatigue_levels`.
> The data stays; the **doc comment and the Fluent label** are the defects. The
> user-facing symptom below is real and is what both passes found independently;
> only the remedy changes. Points 2 and 3 stand.

**Verdict (withdrawn).** The **doc comment is right and all three shipped amounts
are wrong.**
This is not a convention to pick: the book states the quantity and states both
directions in words, and the track is already named after that quantity. Three
signs flip; the comment and both labels stay.

**Evidence — the book names the quantity and the direction.**

ArMDE:6995, `flaw.vulnerable_casting`: *"whenever she is about to lose Fatigue
from casting a spell, she loses **1 more** Fatigue level than normal."*

ArMDE:5263, `virtue.withstand_casting`: *"whenever he is about to lose Fatigue
from casting a spell, he loses **1 less** Fatigue level than normal, with a
minimum of 1 Fatigue level."*

ArMDE:6576, `flaw.painful_magic`: *"Casting spells causes you to suffer the
equivalent of **one Fatigue level** in pain for each spell you cast."*

The quantity the book modifies is **Fatigue levels lost per spell cast** — "more"
for the Flaw, "less" for the Virtue. `types.rs::HealthTrack::CastingFatigue` is
named after exactly that quantity (*"Fatigue levels lost per spell cast"*) and its
doc comment gives the matching signs (Vulnerable Casting +1, Withstand Casting
-1). The shipped data gives `virtue.withstand_casting` **+1**,
`flaw.vulnerable_casting` **-1**, `flaw.painful_magic` **-1** — inverted on all
three.

**Evidence — why the "positive = better" reading is the mistake, and how it got
in.** Its two sibling tracks are **rolls**, where higher genuinely is better:
`HealthTrack::FatigueRoll` (Long-Winded +3, Obese -3) and `HealthTrack::Recovery`
(Rapid Convalescence +3, Fragile Constitution -3). `CastingFatigue` is the one
member of the family that is a **count of levels lost**, not a roll, so the
family-wide convention inverts it. That is a plausible generalisation and it is
still wrong.

**Evidence — the user reads the wrong number.** The track is surfaced-only:
`derived.rs::surfaced_modifiers` pushes it as a `ModifierFamily::HealthRoll` row
with `detail = "casting_fatigue"`, rendered under the Fluent label
`Casting fatigue` / `Zauber-Erschöpfung`. Those labels name the *quantity of
fatigue*, not a bonus, so today a **Withstand Casting** magus — a Virtue — is
shown *"Casting fatigue: +1"*, i.e. that he takes an extra level. That is wrong
rules output on a sheet, which `CLAUDE.md` rates in the top class.

**What it obliges.**

1. **F-351 is unblocked and confirmed.** Flip three amounts in
   `rules/core/virtues_flaws.json`: `virtue.withstand_casting` → -1,
   `flaw.vulnerable_casting` → +1, `flaw.painful_magic` → +1. The doc comment and
   both `.ftl` labels are correct and must **not** change.
2. **A second, independent defect on the same entries, found in the same read.**
   ArMDE:7003 (`flaw.vulnerable_casting`) and ArMDE:5269
   (`virtue.withstand_casting`) each state the pair's exclusion verbatim: *"You
   may **not start your character** with both Vulnerable Casting and Withstand
   Casting."* It is stated on **both** sides and scoped to character creation,
   which is precisely what `incompatible_with` expresses. This is not an
   inference and needs no ruling — it confirms **F-352** outright.
3. **A third, recorded rather than rated.** `flaw.painful_magic` is not a modifier
   to fatigue lost from *casting* at all — ArMDE:6576 gives a pain level *"for
   each spell you cast"*, including spells cast with no Fatigue loss, recovered
   separately (*"You recover these 'pain levels' just like Fatigue levels"*). It
   therefore sums into `casting_fatigue` alongside two entries measuring a
   different thing. Flipping its sign makes it right-signed but still mis-tracked;
   flag it for the D20 text pass rather than inventing a fourth track.

**Status.** SETTLED

---

### Q-89 — ArMDE:5911 says "Technique" inside *Deficient Form*'s own passage: misprint or not?

**Verdict.** It is a **misprint in the book**, not in this repo — and the German
carries it too, so it is in the printed source on both sides and must not be
"corrected" in `rules/source/`. D26 does not reach it; D25 does.

**Evidence.** ArMDE:5911, `#### Deficient Form`, whole body: *"Almost all totals
(including Casting Totals and Lab Totals, but excluding Magic Resistance) to
which a particular **Form** is added are halved. Advancement Totals are not
halved. Experience points required are based on the actual value of the
**Technique**, before halving."* ArMDE:5915, `#### Deficient Technique`, four
lines later: *"All totals, including Lab and Casting totals, including a
particular **Technique** are halved. Advancement Totals are not halved.
Experience points required are based on the actual value of the **Technique**,
before halving."* The final sentence is byte-identical in both entries, and it is
correct in the second. That is a copy-paste from the neighbouring entry.

Three things settle it as a misprint rather than a rule. The entry's own subject
is *"a particular Form"*; read literally the sentence governs an object the Flaw
does not touch, so it would be vacuous for a character with no deficient
Technique; and `effective/xp.rs` never consults `effective/art.rs::deficient_arts`
at all, so XP is already costed off the unhalved score for **both** Arts — the
rule the sentence means to state is satisfied by construction either way.

**The German is the decisive part, and it goes the other way from Q-82's case.**
DE:5911 reads *"Die benötigten Erfahrungspunkte richten sich nach dem
tatsächlichen Wert der **Technik** vor der Halbierung"* — the same error, in the
same place. So it is **not** an OCR slip in the English extraction (D26's scanno
carve-out needs the error to be the OCR's, and `Technique`/`Form` are not an OCR
confusion), and it is not a translation slip. Two independent renderings
reproduce it, which locates it in the printed book.

**What it obliges.**

1. **No change to `rules/source/en/`, `rules/source/de/`, or the shipped strings.**
   The `scheitest` precedent governs: a faithful copy of a real error in the
   source. D26 is explicitly limited to scannos and to a string that contradicts
   itself; this string does neither.
2. **One more row in D25's `RULES.md` errata note** — entry, both citations
   (ArMDE:5911's "Technique" against ArMDE:5909's "Deficient **Form**" heading and
   :5911's own "a particular **Form**"), and the side taken: the sentence means
   *Form*, the text is left as printed, and nothing computes off it.
3. **No mechanical consequence and no finding beyond the note.** F-400
   (`flaw.deficient_form`'s missing `max_total`) is independent and stands.

**Status.** SETTLED

---

### Q-103 — `flaw.false_power`'s `require_categories` admits any Hermetic or Special Virtue where the book says "Supernatural Virtues"

> **REVISED — see § Reconciliation, divergence 6.** Measured: `special` contains
> **one** Virtue and `supernatural` is the stated class, so **two of the three arms
> are already exact**; the over-permission is one arm and **55 entries**. That makes
> a one-id whitelist a cheap exact fix, and reduces this to a narrow cost call
> rather than a remedy blocked on D23.

**Verdict.** Neither of B13's two options is what the book states. ArMDE:6082
states a **predicate**, and signals its own open-endedness with *"like"* — so the
three examples neither close a whitelist nor open two categories. The shipped
three-category list is a measurable over-permission, and the correct shape is
D23's predicate machinery applied on the **inclusion** side.

**Evidence.** ArMDE:6082, verbatim: *"**One of the character's Supernatural
Virtues** is associated with the Infernal realm … This Flaw can apply to
Supernatural Virtues that define the character's background, **like** Faerie
Blood, Diedne Magic, **or even** The Gift."* The head noun is *Supernatural
Virtues*; *"like … or even"* marks the three as illustrations of breadth, not as
the set. So:

- **The whitelist reading fails** because *"like"* is explicitly open.
- **The category reading fails** because it is reverse-engineered from the
  examples' catalogue categories rather than from the noun. Verified: shipped
  `require_categories: ["hermetic","special","supernatural"]`, and
  `virtue.faerie_blood` is `supernatural`, `virtue.diedne_magic` is `hermetic`,
  `virtue.the_gift` is `special` — one category per example, which is the
  signature of a set inferred from three points.

**The over-permission is real, not theoretical.** Admitting the whole `hermetic`
category admits every Hermetic Virtue — Cautious Sorcerer, Free Study, Method
Caster — none of which grants a supernatural power and none of which the passage
contemplates. The parameter's `require_possessed: true` narrows it to Virtues the
character holds, so any magus with an ordinary Hermetic Virtue can name it as his
False Power. `RULES.md` records the three-category list as a deliberate reading;
that record is where the correction belongs.

**What it obliges.**

1. **D23's ruling generalises to this, and should be written to.** D23 added
   predicate-valued **exclusions** for ArMDE:6925 and ArMDE:6148. Q-103 is the
   same missing mechanism on the **inclusion** side of a parameter's domain
   constraint. Designing the exclusion predicate without it would produce a second
   incompatible spelling — exactly what D21 and D13 warn about. **Add it to D23's
   design scope rather than filing it separately.**
2. **The predicate itself is not derivable from the data today.** "A Virtue that
   grants a supernatural power" has no flag: the catalogue carries no realm
   association at top level (`jq` over `rules/core/virtues_flaws.json`: **zero**
   entries have a `realm` key), and `categories: ["supernatural"]` is the wrong
   set by the passage's own examples. So unlike Q-137's predicate, this one needs
   a **new data property** as well as the mechanism. That is the cost to state
   when D23 is scoped.
3. **Until then the current list stands as a documented approximation**, but
   `RULES.md`'s record of it must say it is an approximation with a known
   over-permission — the same rewrite D14 required of `RULES.md`'s Covenant
   Upbringing entry rather than leaving it blessing the looseness.

**Status.** SETTLED on the rules question. Remedy **BLOCKED** on D23's design
(and on a new data property the predicate needs).

---

### Q-109 — `flaw.fury` and `virtue.berserk` state the same condition and are encoded two contradictory ways

**Verdict.** They converge on **Fury's** side. A condition that can never be
resolved at creation time must not be folded flat into a printed total — which is
D4's ruling, for D4's own reason — so `virtue.berserk`'s three effects are a
defect, `flaw.fury`'s `uncomputed_rule` is correct, and **F-20 is confirmed**.

**Evidence — the two passages state the same shape.** ArMDE:3502,
`virtue.berserk`: *"**While berserk**, you get +2 to Attack and Soak scores, but
suffer a -2 penalty to Defense."* ArMDE:6196, `flaw.fury`: *"**While enraged** you
get +3 to Damage, but -1 on all other scores and rolls."* One condition shape, and
neither condition is a character-generation fact — going berserk requires a stress
die of 9+ after taking or dealing a wound (ArMDE:3502), and flying into a rage
requires failing a 9+ stress roll on a provoking event (ArMDE:6196).

**Evidence — the engine folds one flat.** Shipped `virtue.berserk` carries
`combat_mod {attack, +2}`, `combat_mod {defense, -2}`, `soak_mod {+2}`, and
`derived.rs::in_play_mods` sums them unconditionally into
`derived/combat.rs::combat_totals` and the Soak line. So an un-enraged Berserk
character's sheet prints +2 Attack and +2 Soak he does not have — and a -2 Defense
he does not have either, so the error is not even uniformly generous.
`flaw.fury` ships `uncomputed_rule` with no effects.

**Why D4 decides it and D1 does not.** D4 ruled *"a Lab Total does not change
unconditionally"* and required each of nine conditional modifiers to be resolved
statically from its passage, excluding entirely the ones whose condition creation
cannot fix (the season-dependent Cyclic Magic pair). Berserk's condition is of
that kind. D1 went the other way for `spell_level_cap` on an explicitly narrow
ground — *"this is only a **cap** … not a number printed as a result"* — and D1's
own scope paragraph says it *"governs `spell_level_cap` and nothing else"*. A
combat total **is** a number printed as a result, on the sheet the player hands
round the table. So D4's strictness applies and D1's generosity does not.

**What it obliges.**

1. **Delete `virtue.berserk`'s three effects** and move it to `uncomputed_rule`
   with the conditional bonuses written into `description` in both locales
   (D5/D20). This is D15's shape for `flaw.corrupted_spells` and needs the same
   treatment: **its own test**, because a Soak total and two combat totals
   silently change for every Berserk character.
2. **`flaw.fury` stays `uncomputed_rule` and needs no change** — and it could not
   be computed anyway: *"-1 on all other **scores and rolls**"* is broader than any
   effect vocabulary in the engine, broader even than Q-116's five combat scores.
3. **A third clause of Berserk is unencoded and was not part of Q-109.**
   ArMDE:3502: *"You **automatically gain** the Personality Trait Angry +2 (or
   more, at your option)."* Nothing in the entry grants it. That is the same family
   as Q-110's defect below — a V/F that mandates a Personality Trait at a stated
   value, with no machinery to express it — and the two should be designed
   together.
4. **F-20 moves from *live* to *confirmed*,** and Q-109 stops blocking
   `flaw.fury`'s verdict: Fury is **clean**.

**Status.** SETTLED

---

### Q-110 — ArMDE:6124's two +4 traits contradict ArMDE:2502's ±3 for a Minor Personality Flaw

**Verdict.** **There is no contradiction.** The book says in terms that the ±3
range has exceptions, and breaks it itself one clause after stating it. The defect
is in `validation/scores.rs::validate_personality_traits`, which enforces a
**hedged** guideline as a hard error — which D16 already forbids — and which
hardcodes the only exception shape it knows. B13's option (a) is right; (b) and
(c) are both excluded.

**Evidence — the book states the exceptions explicitly.** ArMDE:1075, in the
Personality Traits rules section that ArMDE:2502 itself cross-references (*"These
are your character's Personality Traits (see page 28)"*), verbatim:

> "They can be positive or negative, and **normally range between +3 and -3,
> although there are exceptions**. A Minor Personality Flaw (see later) would
> **normally** be matched by a Personality Trait of +3 or -3, while a Major
> Personality Flaw … **would justify** a Personality Trait of +6 or -6."

*"normally … although there are exceptions"*, *"would normally"*, *"would
justify"* — three hedges in two sentences. And ArMDE:2502 breaks its own range one
clause after stating it: *"attach a value between -3 and +3 to each … a Major
Personality Flaw **should have** a Personality Trait of +6 or -6."* A range the
same sentence exceeds is a default, not a bound.

ArMDE:6124, `flaw.fickle_nature` (*Minor, Personality*), by contrast **commands**:
*"**Select** a Personality Trait at +4, and its opposite at +4."*

**D16's table applies directly.** Hedged (*"normally"*, *"would justify"*) →
**warning, never an error**. Absolute (*"Select"*) → the thing the engine must
permit. Today it is exactly the other way round.

**Evidence — what the validator actually does.**
`validation/scores.rs::validate_personality_traits` counts the entity's **Major**
Personality Flaws, allows that many traits above ±3, caps everything at ±6, and
emits `CODE_PERSONALITY_TRAIT_OUT_OF_RANGE` as an **error** otherwise. Fickle
Nature is Minor, so its two mandated +4 traits raise **two hard errors** — the app
refuses to build the character its own shipped description instructs the player to
build. Its doc comment cites ArMDE:2500-2503 only and does not know ArMDE:1075 or
ArMDE:6124 exist.

**Why (b) and (c) are excluded, not merely disfavoured.** (b) "record the conflict
and leave the validator alone" leaves the app rejecting a legal character, which
is the class of error D16 rejected a `forbidden_traits` row for. (c) "read the +4
as a typo for ±3" invents a number the book does not state — the `7f5605a`
aging-floor mistake by name.

**What it obliges.**

1. **The ±3/±6 check becomes a warning** (D16), and the ±6 hard cap goes with it —
   ArMDE:1075 bounds nothing absolutely.
2. **A V/F must be able to mandate N traits at value M.** B13 named the missing
   field correctly; nothing in `PointItem` expresses it. The family is larger than
   one entry — a `grep` for "Personality Trait" over the V/F block
   (ArMDE:3300-7130) finds **at least five** carriers with a stated value:
   ArMDE:3502 (`virtue.berserk`, Angry +2), ArMDE:4375 (Loyal 0), ArMDE:6124
   (`flaw.fickle_nature`, two at +4), ArMDE:6903 (Uncertain Faith +1) and
   ArMDE:7078 (`flaw.weak_personality`, *all* traits constrained to +1…-1). None is
   encoded.
3. **It must express both directions.** ArMDE:7078 **narrows** the range where the
   others widen it, so a "grants N traits at M" field alone is insufficient — the
   same observation F-542 makes from the other side, and another reason to design
   it with D21's Effect-side twin rather than separately.
4. **The trait *names* are a D9 choice.** ArMDE:6124 lists the options (*"Happy
   and Sad, Energetic and Lazy, Confident and Diffident, or Proud and Humble"*), so
   under D9 they are `enumerated`, not free text.
5. **F-440 is unblocked**: it is a real defect, and its fix is the validator, not
   the data.

**Status.** SETTLED

---

### Q-116 — the book defines "combat scores"; the repo's reading excludes three of the five

**Verdict.** **ArMDE:16656's definition governs, and `RULES.md`'s narrower reading
is wrong.** Both entries are under-modelled. But the scope is **two entries, not
six** — B14's predicted generalisation does not survive reading the other four
passages.

**Evidence — the defined term.** ArMDE:16656, under the heading `### Combat
Scores`: *"Characters have **five combat scores: Initiative, Attack, Defense,
Damage, and Soak.**"* Both entries use the book's own vocabulary:

- ArMDE:6332, `flaw.lame`: *"-3 on Dodge, and -1 on other **combat scores**."*
- ArMDE:6262, `flaw.hobbled`: *"Her Dodge and other **combat rolls** are penalized
  by -6."*

**Evidence — the book distinguishes *scores* from *rolls*, and `RULES.md` does
not.** ArMDE:22809: *"**All combat rolls use stress dice.**"* Of the five formulas
at ArMDE:16658-16666, exactly three end in `+ Stress Die` — Initiative, Attack,
Defense — while DAMAGE TOTAL (Strength + Weapon Damage Modifier + Attack
Advantage) and SOAK TOTAL (Stamina + Armor Protection) do not. So the book's two
phrases have **different extensions**, and the two entries are not saying the same
thing:

| Entry | Phrase | Reaches | Shipped `combat_mod` |
|---|---|---|---|
| `flaw.lame` | "other combat **scores**" | Initiative, Attack, Defense, Damage, Soak | attack, defense (+ dodge-scoped defense) |
| `flaw.hobbled` | "other combat **rolls**" | Initiative, Attack, Defense | attack, defense |

Any reading that collapses the two phrases into one — which is what `RULES.md`
does, treating *"combat rolls / combat scores"* as a single phrase needing a
single decision — cannot produce that difference.

**Evidence — the term is used as a collective elsewhere, never as "Attack and
Defense".** Sweeping the English core book for the phrase returns five further
sites, all meaning the whole set: ArMDE:16363 (*"the animals must all have Combat
Scores that each fall within a 5-point range"*), ArMDE:18677 (*"he has no physical
statistics or combat scores"*), ArMDE:19919 (*"It uses the combat scores listed
above"*), plus ArMDE:16654's heading and ArMDE:16656 itself.

**What `RULES.md` actually argues, and where it fails.** Contrary to B14's
account, `RULES.md` **does** cite ArMDE:16656 and quote its five names — it saw
the definition and decided against it on 2026-09-14, closing row 37 of
`docs/open-todos.md`. Its argument is that Attack and Defense are the two formulas
carrying a **Combat Ability**, so *"a penalty on 'combat' as a skill at fighting
lands on exactly Attack and Defense"*. The substitution is the failure: the
entries do not say "combat skill", they say *combat scores* and *combat rolls*,
and both are terms the book defines elsewhere. Deriving a defined term's extension
from which formulas mention a Combat Ability answers a different question.

**Scope — corrected from six entries to two.** I read the other four the batch
predicted would follow, and none uses either phrase:

- ArMDE:6436, `flaw.missing_eye` — *"-3 on **Attack rolls** for missiles … -1 on
  **Attack rolls**"*; names the total outright. (Separately: the -3 missile/
  targeting half is unmodelled, which is its own defect and not Q-116's.)
- ArMDE:6608, `flaw.poor_eyesight` — *"Rolls involving sight, including rolls to
  **attack and defend**"*; names both outright.
- ArMDE:6580, `flaw.palsied_hands` — *"All rolls involving holding or wielding an
  object … including weapon skills"*; a different predicate entirely.
- ArMDE:6440, `flaw.missing_hand` — *"activities normally requiring both hands are
  at a penalty of **-3 or greater**"*; hedged, and not in combat vocabulary.

So `RULES.md`'s reading is defensible for those four on their own words and is
untouched by this. **Q-116 is a two-entry decision.**

**What it obliges.**

1. **`flaw.lame` gains `combat_mod -1` on `initiative` and `damage`, and
   `soak_mod -1`.** The existing `combat_mod -3 defense weapon:weapon.dodge` row is
   correct and stays (verified: `derived.rs::in_play_mods` stores it as the delta
   `-3 - (-1) = -2` so the Dodge line reads -3, which is what *"other"* requires).
2. **`flaw.hobbled` gains `combat_mod -6` on `initiative`.** Not Damage, not Soak —
   it says *rolls*.
3. **No engine change.** `types.rs::CombatStat` already has Initiative, Attack,
   Defense and Damage; Soak rides `soak_mod`, which `virtue.tough` and `flaw.frail`
   already use.
4. **`RULES.md`'s "What 'combat rolls' / 'combat scores' was read to mean" block is
   rewritten, not amended** — both its conclusion and its stated warrant stop being
   true, and row 37 of `docs/open-todos.md` reopens. Cite ArMDE:22809 there, since
   it is the line that makes the two phrases differ.
5. **Both entries stop being unrated.** They are **defective**, at `high`: wrong
   numbers on a printed sheet.

**Status.** SETTLED

---

### Q-117 — two in-repo authorities give opposite readings to "the passage states the *absence* of a rule"

**Verdict.** `RULES.md`'s principle survives and the `NO_RULE_DESPITE_TOKEN`
exemption does not. **D20 decides it**, and D20 post-dates both authorities: a rule
the book states must reach the player somehow, and this one reaches him nowhere.

**Evidence — the two passages are the same shape.** ArMDE:6324, `flaw.jinxed`:
*"He is not personally the cause of the bad luck, and so **need not roll any extra
botch dice or suffer any penalty to die rolls**."* ArMDE:6266,
`flaw.horrifying_appearance_snake_legs`: *"You have no legs, and instead your hips
give rise to two or more snake-like tails … **Your movement is not hindered under
most circumstances.** You can hide this deformity under clothing, but you cannot
move without revealing it, leaving you feigning being crippled as well."* Both
sentences exist to forestall an inference the rest of the entry invites — that a
bad-luck magnet rolls extra botch dice; that a legless character moves like
`flaw.lame` or `flaw.hobbled` (ArMDE:6332, :6262), which do exactly that. A
sentence written to stop a player applying a penalty is a rule the player needs.

**Evidence — D20's test, applied.** D20: *"a rule the book states must reach the
player somehow — as a computed number, or as written text. If the engine cannot
compute it, text is the only route, and **if neither happens the app has silently
dropped a rule**."* `flaw.horrifying_appearance_snake_legs` ships `narrative`, with
**no `description` in either locale**, a `summary` stopping at sentence one, and an
explicit screen exemption. Neither route happens. That is D20's silent drop, stated
as plainly as the ruling could ask.

**Evidence — D3's precedent is the same one.** D3: *"`narrative` remains a claim
that the book states nothing mechanical, and **nothing about engine capability can
make that claim true**."* The exemption's written reading — *"explicitly declines
to impose a penalty. Nothing rolls, nothing is capped, nothing is modified"* — is
an argument from what the engine computes, which D3 forbids as grounds for
`narrative`, and D8 then generalised (*"a roll-free supernatural capability is a
rule"*). A roll-free *non*-penalty is the mirror of D8's case and falls the same
way. Note the entry is `supernatural`-categorised, so it is plausibly one of
**D8's own 48** already — check that list before filing it as a separate move.

**What it obliges.**

1. **`flaw.horrifying_appearance_snake_legs` moves `narrative` →
   `uncomputed_rule`**, with the passage in `description` in **both** locales (its
   German, DE:6266, is already complete and line-parallel).
2. **The `NO_RULE_DESPITE_TOKEN` row is deleted, not reworded.** B14 is right that
   the module asserts every row still fails the screen, so a reworded row would
   assert the opposite of the ruling.
3. **`RULES.md`'s sentence stands as written** and does not need re-founding on
   Jinxed's half-the-time clause. B14's observation that Jinxed *also* carries a
   positive rule is true and is now belt-and-braces rather than load-bearing.
4. **It lands in D19 / § 2.1's queue, not before it.** The new `description` is a
   pure non-penalty statement carrying no signed number and no botch term, so it
   depends on a `MECHANICAL_PHRASES` family that does not yet exist — the same
   blocker as D8's 48. Add a "declines to impose a penalty" family to the list
   § 2.1b is assembling; *"nicht behindert"* and *"muss … weder … noch"* are the
   German forms, and both are **discontinuous**, which is exactly why D19 ruled for
   regex.

**Status.** SETTLED (by D20 + D3, applied)

---

### Q-125 — the rulebook contradicts itself about `flaw.prohibition`'s category

**Verdict.** **D25 decides it and the shipped `["supernatural"]` is right**, by two
independent margins: the descriptor and the index agree with each other, and the
dissenting mention is an aside inside a *different* entry. Recorded as an erratum,
no data change.

**Evidence, verified line by line.** ArMDE:6639, `#### Prohibition`'s own
descriptor: *Minor, **Supernatural***. ArMDE:5552 is `[Prohibition](#prohibition)`
inside the index list headed `### Supernatural, Minor` (heading at ArMDE:5534) — so
the index **agrees** with the descriptor here, unlike Q-79 and unlike D25's own
case. The dissent is ArMDE:6719, inside `#### Servant of the (Land)`: *"while it
remains incomplete the character has the **Minor Personality Flaw: Prohibition**,
but this does not count toward the character's total number of Virtues and
Flaws."*

So it is two sources against one, and the one is a passing cross-reference in
another entry — weaker still than D25's index line, which at least purports to
classify.

**And the mechanical test points the same way, which D25's own case did not have.**
If Prohibition were `personality`, the grant at ArMDE:6719 would consume one of the
character's Personality-Flaw slots (ArMDE:2820: *"A character may not have more
than one Major Personality Flaw … should normally not have more than two
Personality Flaws in total"*) and would feed
`validation/scores.rs::validate_personality_traits`' Major-Flaw budget — while
:6719's own words are that the grant *"does not count toward the character's total
number of Virtues and Flaws"*. The Supernatural reading is the one that keeps the
grant free, as the granting sentence says it must be.

**What it obliges.**

1. **No data change.** `categories: ["supernatural"]` stands.
2. **One more row in D25's `RULES.md` errata note**, with all three citations
   (ArMDE:6639 descriptor, ArMDE:5534 and :5552 index, ArMDE:6719 dissent) and the
   side taken — plus the mechanical argument above, which is the reason the dissent
   cannot simply be adopted.
3. **F-498 is unaffected and stands** — `flaw.servant_of_the_land` still owes the
   `grants_selection` naming `flaw.prohibition`, and it names the right entry
   whichever category that entry carries.

**Status.** SETTLED (by D25)

---

### Q-50 — `mythic_type.nephilim` carries neither of ArMDE:2731's two point adjustments

> **CONFIRMED, and extended — see § Reconciliation, divergence 3.** Both passes
> agree Nephilim is correct. The second pass then found a **new `high` defect on
> its two siblings**: ArMDE:2638 and ArMDE:2664 show `bonus_flaw_points: 7` on
> `mythic_type.devil_child` (and `mythic_type.spirit_votary`) over-budgets them by
> 7 Flaw / 14 Virtue points. No existing finding covers it.

**Verdict.** It should not. ArMDE:2731 states **no bonus at all** — it is the base
Mythic Companion budget with the required Virtues' cost worked out for the reader.
`bonus_flaw_points: null` and `bonus_free_virtue_points: null` are **correct**, and
adding either would over-fund the type.

**Evidence — the arithmetic closes exactly.** ArMDE:2731, verbatim: *"Nephilim
must take **five points of Flaws to pay for these virtues** and may take an
**additional five points of Flaws, which grants a further ten points of
Virtues**."*

The required list is ArMDE:2723-2730. Priced from the shipped magnitudes
(free = 0, minor = 1, major = 3):

| Required Virtue | Magnitude | Points |
|---|---|---|
| `virtue.nephilim` (granted) | free | 0 |
| `virtue.strong_angelic_heritage` (granted, "free with Nephilim") | minor | 0 — a `grants` row, and grants are budget-exempt |
| `virtue.blood_of_the_nephilim` | major | 3 |
| `virtue.greater_immunity` | major | 3 |
| `virtue.great_characteristic` (Sta) | minor | 1 |
| `virtue.great_characteristic` (Str) | minor | 1 |
| `virtue.improved_characteristics` | minor | 1 |
| `virtue.sense_holiness_and_unholiness` | minor | 1 |
| **Total bought** | | **10** |

Ten Virtue points at the mythic-companion rate of 2 is exactly **five points of
Flaws** — clause 1, precisely. Clause 2 then adds five more Flaw points for ten
more Virtue points, giving **10 Flaw / 20 Virtue** in total, which is
`character_types.json`'s `mythic_companion` budget verbatim
(`{"virtue_points": 20, "flaw_points": 10, "virtue_points_per_flaw_point": 2}`)
and ArMDE:2844's *"up to 10 points of Flaws, and 2 points of Virtues for every 1
point of Flaws, for a maximum of 20 points of Virtues."*

**So :2731 is a worked example, not an adjustment.** It tells the player how his
standard budget is consumed: half of it is spoken for by the required list, and
the other half is his. The two siblings are genuinely different — `devil_child`'s
`+7 F / +3 free V` comes from ArMDE:2664 and `spirit_votary`'s `+7 F` from
RoP:M:5486, both of which state an increase in so many words.

**What it obliges.** Nothing. Record the arithmetic in `RULES.md` beside the
mythic-type budgets so the absence of the two fields reads as verified rather than
as an omission — an undocumented correct-looking value is indistinguishable from
an unexamined one (D25's closing precedent).

**Status.** SETTLED

---

### Q-55 — `virtue.physician_of_salerno`: is the granted Reputation Local or Academic?

**Verdict.** **Neither is stated, and `local` is positively wrong.** The passage
names no audience, so under D11's own worked remedy the pin must go and the grant
ships the wildcard. This is F-450's defect on a second entry.

**Evidence — the passage names no audience, and rules one out.** ArMDE:4734:
*"Not only does he carry **the reputation of the school with him** (granting a
Reputation of Physician of Salerno 2), but he has also learned some unique medical
procedures…"* A `Local` Reputation is scoped to a place; this one is explicitly
**portable** — he carries it with him. So the shipped
`{"type":"grants_reputation","kind":"local","score":2}` asserts the one type the
sentence excludes.

**Evidence — D11 has already ruled the remedy for exactly this shape.** D11: *"stop
hardcoding an audience the passage leaves open (F-450 `flaw.infamous` pins `kind:
"local"` where ArMDE:6312 names none, while its twin `virtue.famous` correctly
ships the wildcard)"*, and *"`GrantsReputation.kind: Option<..>` with `None` means
player-chosen"*. ArMDE:4734 names none. Same defect, same fix.

**Why not simply `academic`.** It is the better single guess — the entry's own
closing sentence is *"To take this Virtue, you must be able to take **Academic
Abilities**"*, the school is *"the pre-eminent source of medical learning"*, and
`types.rs`'s `ReputationType::Academic` doc comment names the scholastic
Social-Status Virtues as its company. But a physician's reputation among patients
is not academic, the repo's own curated table hedges (`reputationen.md:82` and
`:131` both read *Lokal / Akademisch*), and asserting a type the passage does not
state is the same error as asserting `local`. **The wildcard is the faithful
encoding**; `academic` would be a guess dressed as data.

**What it obliges.**

1. **Drop `"kind": "local"`** from `virtue.physician_of_salerno`'s
   `grants_reputation`, leaving the player-chosen wildcard, and note the entry
   alongside F-450 so the two are fixed in one pass.
2. **D11's score enforcement is unaffected** — `score: 2` is stated outright and
   stays exact.
3. **The table rows are not wrong and need no D6 correction.** *Lokal /
   Akademisch* is an accurate report that the book leaves it open; it is a
   terminology table correctly declining to make a factual claim.

**Status.** SETTLED

---

### Q-57 — Potent Magic: how should the forced magnitude-variant incompatibility be lifted?

**Verdict.** **Neither of B07's two ways out.** Both are workarounds for a guard
that infers a *rules fact* from an *id naming convention*, which breaks
`CLAUDE.md`'s separability invariant. The fix is D10's shape: keep the default,
declare the exception in **data**.

**Evidence — the book is explicit and the data currently contradicts it.**
ArMDE:4742: *"…**a maga may have more than one area of Potent Magic**, although
only one Potent Magic Virtue applies to any single activity."* DE:4742 is clean and
says the same (*"kann eine Maga jedoch mehr als einen Bereich Potenter Magie
besitzen"*). The shipped data declares
`virtue.potent_magic_major.incompatible_with = ["virtue.potent_magic_minor"]` and
the mirror — a value the book denies (F-226).

**Evidence — why it is there.**
`ruleset/integrity.rs::validate_magnitude_variant_exclusivity` iterates every
`point_item`, derives a sibling id from the **id string** (`minor_variant_sibling`),
and pushes a **load error** unless both declare each other incompatible. So the
wrong value is not an authoring slip; the guard compels it, and the whole ruleset
fails to load without it.

**Why that is the defect.** `CLAUDE.md`: *"Abilities and V/F are added by editing
`rules/core/*.json` + `rules/i18n/<lang>/*.json` with **zero code changes** — this
is what keeps the executable and the rules separable."* A guard that reads a rules
fact off an id spelling makes a legitimately-compatible `_major`/`_minor` pair
unaddable without touching Rust. The invariant is already violated in practice:
the catalogue today ships a value the rulebook contradicts, because the code
required it.

**Why B07's two options are both wrong.**

- **Exempt the pair by name in the guard** puts a catalogue id in Rust, which is
  the same invariant violation in a smaller font, and the next compatible pair
  needs a second code change.
- **Rename the ids** (`_wide`/`_narrow`) breaks every save holding one —
  `Selection.item_ref` is the id — so it needs a `SCHEMA_VERSION` bump and a
  migration, to buy nothing but invisibility from a guard. It also discards the
  book's own words: ArMDE:4741's descriptor is *"Minor or Major, Hermetic"*, so
  the magnitude names are the rulebook's.

**What it obliges.**

1. **An optional data opt-out, exactly D10's shape.** The default stays "a
   `_major`/`_minor` pair is mutually exclusive" — true for every other pair in
   the catalogue, and warranted for Magical Focus by ArMDE:4405 and :4542 — and a
   pair the book permits together declares so. One optional field, with
   `skip_serializing_if`, so no other entry's JSON changes; the same shape D11
   chose for `max_score` and D13 for `from_normal_budget`.
2. **Then delete the two `incompatible_with` rows**, which closes F-226.
3. **D10 has a second claim on this pair.** ArMDE:4742's *"more than one area"*
   means both entries legitimately repeat, so each needs an explicit `max_total`
   under D10's inverted default, and a parameter recording the **area** under D9 —
   without which "more than one area" is unrepresentable and the repeats are
   indistinguishable.
4. **Incidental, and nobody asked:** the English at ArMDE:4742 contains a
   **doubled clause** — *"compatible with a Magical Focus, **unlike a Magical
   Focus, unlike a Magical Focus,** a maga may have more than one area"*. DE:4742
   has it once. So the duplication is an extraction artefact, not the book; it is
   D26's decision shape (correct in `rules/source/en/`), and it is **not** covered
   by D26's closed scope, which named only ArMDE:7054 and ArMDE:3502.

**Status.** SETTLED

---

### Q-62 — `virtue.powerful_relic`: is the relic's one power charged against the power-levels budget?

**Verdict.** No — and it must not be recorded in `Entity::powers` at all. A relic
is an **item the character owns**, not a power the character has, and the book
states no level for its power, so there is nothing any budget could charge.
`virtue.powerful_relic` must **not** gain a `power_levels` grant.

**Evidence — the passage says "own" and "item" and gives no number.** ArMDE:4784:
*"**You own** an unusually powerful relic with a True Faith score of 3. The relic
also has **one power**, which should be **agreed upon with the storyguide** (see
Relics, page 419). As with the Minor General Virtue Relic, **the item** may be
built into any other item that you possess, like a sword or a pendant."* The
subject throughout is an object the character possesses. And the power's content is
a troupe agreement with no magnitude — B07 followed the page-419 cross-reference
and found no level stated, which I have not re-derived and am relying on.

**Evidence — recording it in `Entity::powers` produces a false error.**
`effective::powers_used` charges every `Entity::powers` row against
`power_levels_budget`, which this Virtue leaves at 0, so a player who writes the
relic's power down gets `over_power_levels` — the app refusing a character the book
describes. That is the same misfit F-249 reports for `virtue.ripper`, and it is why
the answer matters.

**Why a `power_levels` grant is the wrong repair.** A budget models the player
*designing* powers to a level allowance. Here there is exactly one power, its
content is agreed with the storyguide, and no level exists to spend — so any number
put in the grant would be invented, which is the `7f5605a` error. The neighbouring
`item_level_budget` is equally wrong: it counts **levels of enchantment effect**,
and a relic's power is Divine, not a Hermetic enchantment.

**What it obliges.**

1. **No new effect on `virtue.powerful_relic`.** Its `true_faith_grant: 3` is
   correct and complete for what the engine computes (ArMDE:4784's *"True Faith
   score of 3"*, and `virtue.relic`'s `1` at ArMDE:4854 matches the same way).
2. **The relic's power reaches the player as text**, per D5/D20 — it is a rule the
   engine cannot compute, so the description in both locales is the only route,
   together with ArMDE:4786's impiety clause (*"your relic will cease to function
   until suitable penance is made"*), which is equally uncomputed.
3. **The guidance is general and worth writing down once:** a power belonging to an
   **item the character owns** does not enter `Entity::powers`. Stating that beside
   `powers_used` would have prevented this question and would also settle the same
   worry for `virtue.relic` and `virtue.infernal_heirloom`.
4. **F-249 (`virtue.ripper`) is a different case and is not settled here** — there
   the two powers are the *character's* and their levels **are** stated (PeAn(He)
   25 and PeAn 45), so the budget question is real. Q-58 still needs its answer.

**Status.** SETTLED

---

### Q-67 — what amount should `virtue.simple_student`'s restricted XP pool carry?

**Verdict.** **No single amount exists.** ArMDE:4960 states a *rate*, not a total,
and the multiplicand is a player choice the engine does not record. So the honest
classification today is `uncomputed_rule` with the rule written out — not
`creation_effect` with no effects, which is what ships and which silently gives the
character **nothing**.

**Evidence.** ArMDE:4960, verbatim: *"He is typically between 14 and 16 years old
and somewhere along his university program. He receives **30 experience points per
finished year** that he can apply to Latin or Artes Liberales. If he has finished
his second year of studies, he is in the liminal position of either applying for
work or continuing his education."*

The grant is `30 × (finished years)`. The passage fixes neither the number of
finished years nor a maximum — *"somewhere along his university program"* — and the
second-year sentence is a story observation, not a cap. So there is no constant to
put in `RestrictedAbilityXp.amount`, and choosing 30 or 60 would be inventing a
number the book does not state (`7f5605a`).

**Evidence — what ships.** `virtue.simple_student` is `classification:
"creation_effect"` with **no `effects` array at all**. So the entry claims to
compute something and computes nothing: a Simple Student receives zero of the
experience points the book grants him, and nothing in either locale tells him why.
It is one of the five effect-less `creation_effect` entries `README.md` flagged on
day one; D15 resolved another (`flaw.corrupted_arts`).

**What it obliges.**

1. **Reclassify to `uncomputed_rule`** and write ArMDE:4960's rule into
   `description` in both locales — D3's precedent exactly (*"an engine that
   structurally cannot express a rule is grounds for `uncomputed_rule` with the
   rule written out"*), reinforced by D20's RAW-fidelity test, which this entry
   currently fails outright.
2. **The proper model is a numeric parameter and a scaled grant**, and neither
   exists: `ParameterDef` has no numeric type (D9's survey found every parameter
   is single-valued `"type": "ref"`), and `RestrictedAbilityXp.amount` is a
   constant with no way to read a parameter. That is **new machinery**, and it is
   *not* D9 part 3's multi-valued type — it is a third parameter kind. Worth
   designing alongside D9 part 3 rather than after it, for D13's stated reason.
3. **A second defect fell out of the same read, and it belongs to D16/Q-05.**
   ArMDE:4960: *"**Female characters can only take this Virtue if they are studying
   to be physicians at Salerno**, although the Paid Rights Virtue would allow them
   to take this Virtue elsewhere."* D16 re-counted the sex-restriction family as
   *"20 entries … plus **one** female-only exception (`virtue.baccalaureus`,
   ArMDE:3474)"*. This is a **second** female-only exception, and the count is
   therefore short. Under D16 it is `uncomputed_rule` + `description` in both
   locales, which the reclassification above already delivers.

**Status.** SETTLED on the rules read. The *exact* model is **BLOCKED** on a
numeric parameter type; the interim classification is not blocked and should land.

---

### Q-68 — does Subtle Magic's "no benefits from normal gestures" add anything the table does not already price at zero?

**Verdict.** **"Normal gestures" means Bold, which is 0, so the clause adds no
rule.** The book says so in the sentence that introduces the table, and the
Words-side twin confirms the reading independently. F-301 is right to stand on the
exaggerated half alone.

**Evidence — the book defines "normal" for this table.** ArMDE:9236, the paragraph
immediately above the Words and Gestures table: *"Spells are **normally** cast with
a **firm voice and bold gestures**. However, the caster may choose to be more or
less subtle."* The table at ArMDE:9240-9245 then prices Bold at **0** and Firm at
**0**, with Exaggerated +1, Subtle −2, None −5 on the Gestures side. So *normal
gestures* = Bold = 0.

`virtue.subtle_magic`, ArMDE:5075: *"You may cast spells without using gestures at
no penalty. You gain no benefits from using **normal** gestures but gain the normal
benefit for exaggerated gestures."* Read against :9236 the middle clause restates a
zero.

**Evidence — the twin settles it independently.** `virtue.quiet_magic`, ArMDE:4824:
*"You can cast spells using only a soft voice at no penalty, and at only a −5
penalty if you do not speak at all. You gain no benefits from **using your voice
normally** but gain the normal benefit for using a booming voice."* Same
construction on the Words column, where *"using your voice normally"* is
unambiguously :9236's **Firm**, also 0. Two entries, one idiom, one referent — and
`virtue.quiet_magic` cannot be read as pointing at its own name's row, because
there is no "Quiet Magic" row, only Quiet at −5 which the entry explicitly waives.

**Why the German hint does not carry.** DE:5075's *"Du profitierst **nicht mehr**
von normalen Gesten"* reads as *no longer*, which suggested a change of state. But
`CLAUDE.md` makes English the source of truth for rules, DE is a rendering of it,
and the English carries no such word. The German is a translator's intensifier, not
evidence about the rule.

**What the clause is actually doing**, worth writing into the description rather
than dropping: having zeroed the *None* row, it blocks the inference that the whole
column shifts — Bold stays 0 and Exaggerated stays +1 relative to the unchanged
scale, rather than becoming +5 and +6.

**What it obliges.**

1. **Nothing computed changes.** Both entries carry surfaced-only markers
   (`special_casting_mod: subtle_gestures` / the `voice_reduction` constant) with
   no number, so they sit inside D20's cohort and owe their rule as **text** in
   both locales regardless.
2. **F-301 shrinks as B08 expected** — the informative half is the exaggerated-
   gestures clause plus the waived *None* penalty.
3. **F-239 (`virtue.quiet_magic`) takes the same resolution**, as B08 predicted;
   the two should be written in one pass so the parallel wording stays parallel.

**Status.** SETTLED

---

### Q-70 — should `virtue.sense_passions` carry `tainted: true`?

**Verdict.** **No.** `tainted` is exactly the descriptor tag, and the catalogue
follows that rule with **zero exceptions** — measured, not assumed. ArMDE:4931
carries no tag, so neither should the entry.

**Evidence — the rule, and the census that makes it a rule rather than a habit.**
`engine-semantics.md` § B9 defines the field as the descriptor tag, and ArMDE:3000
defines the tag's meaning (*"Tainted Virtues and Flaws are associated with the
Infernal realm"*). Counting both sides:

- Descriptor lines in the English core book matching `*…, Tainted*`: **21**.
- Catalogue entries with `tainted: true`: **23**.

The gap of two is fully accounted for by two descriptors that define **two ids**
each: ArMDE:3411 *"Major **or** Minor, Supernatural, Tainted"* →
`virtue.amorphous_major` + `virtue.amorphous_minor` (both cite :3410-3413), and
ArMDE:6081 → `flaw.false_power` + `flaw.false_power_minor` (both cite :6080-6097).
**So the mapping is exact: every `tainted: true` id traces to a descriptor carrying
the tag, and no entry carries the flag without one.** `virtue.sense_passions`
(ArMDE:4931, *Major, Supernatural*; DE:4931, *Groß, Übernatürlich*) is not in that
set.

**Why the page-170 cross-reference does not override it.** ArMDE:7737: *"Sense
Passions is **either a false power** (see the False Power Flaw), **or** is
associated with the Infernal."* The association is **disjunctive** — one of the two
arms is that the power is *not* actually Infernal but merely appears so — so it is
not the unconditional realm association the tag asserts. ArMDE:3000 states Tainted ⇒
Infernal and never the converse, so an Infernal association does not entail the
tag. And the one consumer, `validate_tainted_cap`, would then count a Virtue toward
a cap the book's descriptor declines to apply to it.

**What it obliges.** Nothing on the entry. Two things worth recording: the census
above, in `RULES.md`, so `tainted`'s exact correspondence to the descriptor is a
*verified* invariant rather than an assumption (it is also a candidate for a cheap
data-integrity guard, since both sides are mechanically derivable); and B08's
observation that the realm fact **does** reach the user through
`abilities.json`'s description in both locales, so nothing is dropped by declining
the flag.

**Status.** SETTLED

---

### Q-78 — does `virtue.unaging`'s non-fatal-crisis clause have a home in the M6/6b7 crisis engine?

**Verdict.** **Yes, and it is a small, well-scoped change.** B09 looked one level
too low: the distinction the passage draws is already the crisis table's
**outcome type**, not its `CrisisSeverity` rank, and `CrisisOutcome` has exactly
the two variants the rule needs.

**Evidence — the passage draws a two-way split.** ArMDE:5189: *"**If a crisis is
not potentially fatal, you suffer no ill-effects.** You may die from terminal and
potentially fatal crises, according to the normal rules."*

**Evidence — the table already draws the same split.** `aging.rs::CrisisOutcome`
has exactly two variants, and their own doc comments state the distinction:

- `Bedridden` — *"'Bedridden for a week' (ArMDE:16626) and 'Bedridden for a month.'
  (ArMDE:16627) — **no roll, no spell, nothing but time**."* Rows
  `crisis.bedridden_week` (≤ 8) and `crisis.bedridden_month` (9-14).
- `Illness { severity, ease_factor, ritual_level }` — *"an illness the character
  **must survive**"*, ArMDE:16628-16632. Rows `crisis.minor_illness` (15) through
  the Terminal row.

*Not potentially fatal* is `Bedridden`, definitionally: no roll is made, so no
death is possible. *Potentially fatal* is every `Illness` row, each of which demands
a Stamina stress roll or a Creo Corpus Ritual to survive, up to the Terminal row
that offers no roll at all. The mapping is total and needs no judgement.

**Why B09 could not see it.** It reached for `crisis_survival` (a *modifier* to the
survival roll, which cannot express "no effect") and for `CrisisSeverity`, which
only ranks the five **illness** rows and therefore cannot distinguish a bedridden
crisis from an illness at all. The rule lives one level up.

**What it obliges.**

1. **One narrow rule in the crisis resolver:** a carrier of `virtue.unaging` who
   rolls a `CrisisOutcome::Bedridden` row suffers **no ill-effect**; every
   `Illness` row resolves unchanged. No new `AgingEffect` kind, no new table
   column, no change to `CrisisSeverity`.
2. **Scope it tightly — the clause voids the crisis's *outcome*, nothing else.**
   ArMDE:5189's neighbouring sentences keep the rest of the bookkeeping intact:
   *"your aging points do not decrease your Characteristics, **only building up to
   give you Decrepitude points**"* and *"You are not enfeebled when you reach four
   Decrepitude points, but you **die as normal when you reach five**."* Decrepitude
   still accrues; only the bedridden time is voided.
3. **F-332's text obligation still stands** and is not replaced by this — D5/D20
   want the whole clause in `description` in both locales whether or not it is
   computed, and the Decrepitude-4 enfeeblement waiver in the same passage is a
   second uncomputed clause.

**Status.** SETTLED

---

### Q-112 — does `virtue.lone_redcap` satisfy `flaw.hermetic_patron`'s "a Redcap"?

**Verdict.** **Yes** — `virtue.lone_redcap`'s own passage opens by saying so. But
Q-112's stated aim, *"apply one answer to both entries rather than leaving two
Redcap prerequisites written differently"*, is the wrong aim: the source says the
two prerequisites **must** differ, and `virtue.magic_items` naming
`virtue.redcap` alone is correct.

**Evidence — a Lone Redcap is a Redcap.** ArMDE:4321's first six words: *"**You are
a Redcap** who does not maintain ties to a Mercer House."* ArMDE:6250,
`flaw.hermetic_patron`, whole restriction: *"You must be a Redcap or magus to take
this Flaw."* Nothing in ArMDE:6248-6255 narrows "Redcap", and ArMDE:6252 uses the
word again unqualified for the patron himself (*"an older magus or a more
established **Redcap**"*). So the Flaw admits him, and **F-448's**
`Any([Has(virtue.redcap), Has(virtue.lone_redcap), IsMagus])` is right.

**Evidence — and `virtue.magic_items` is the deliberate exclusion B14 could not
confirm.** ArMDE:4349: *"You begin with **25 more starting levels of magic items**
than you would otherwise … **You must be a Redcap to take this Virtue.**"* Its
warrant is not the phrase but the other entry's own text: ArMDE:4321 says a Lone
Redcap *"does not maintain ties to a Mercer House, and thus **do not receive magic
items** or Longevity Rituals."* A character who receives no magic items cannot
begin with 25 more levels of them. So the shipped
`prerequisites: {has: virtue.redcap}` on `virtue.magic_items` is **correct**, and
the two prerequisites differ because the rules differ — not because nobody
revisited one.

**What it obliges.**

1. **F-448's fix is unblocked** and should be written as proposed, with
   `Prereq::IsMagus` for the magus arm (D12's normalisation) rather than
   `Has(virtue.hermetic_magus)`.
2. **Do not widen `virtue.magic_items`.** Add a short note at that entry — or in
   `RULES.md` — citing ArMDE:4321 for *why* it names the Major Redcap alone, so the
   next reader does not "harmonise" the two.
3. **A stronger form is arguably owed there and is flagged, not asserted:**
   ArMDE:4321 denies the Lone Redcap magic items outright, which reads like an
   incompatibility rather than merely an unmet prerequisite. Today a character
   holding both would fail `prereq_not_met` anyway, so nothing is reachable; it is
   a wording question for the same slice.
4. **D17 is untouched.** This says nothing about the 300-point question, which D17
   settled independently.

**Status.** SETTLED

---

### Q-115 — is `flaw.inscribed_shadow`'s House Criamon restriction a prerequisite to encode?

> **REVISED — see § Reconciliation, divergence 5.** A **warning is owed** after
> all: D16 closed Q-139 on ArMDE:6957's identical *"generally restricted to magi of
> House Verditius"* wording. The "no hard prerequisite" half below stands; the "no
> warning" half does not.

**Verdict (partly withdrawn).** **No hard prerequisite** — and, as originally
written here but now corrected above, no warning either. The sentence imposes exactly one
restriction (stigmata), which the engine does not model and which the entry already
carries as text; the House clause is a hedged *gloss* on that restriction, not a
second rule. The shipped state is correct and B14's instinct was right.

**Evidence — parse the sentence.** ArMDE:6320, final sentence: *"**Only characters
with stigmata may have this Flaw**, which **normally means** that they must be magi
of House Criamon."* DE:6320 is line-parallel and identical in structure (*"Nur
Charaktere mit Stigmata dürfen diesen Fehler haben, was **normalerweise** bedeutet,
dass sie Magi des Hauses Criamon sein müssen"*). The main clause is absolute
(*"Only … may"*); the relative clause is the author explaining what the absolute
usually implies. It states no restriction of its own.

**Why that settles both halves.** The **absolute** gate is *stigmata*, which the
engine models nowhere — D3's precedent puts that in `uncomputed_rule` with the rule
written out, which is exactly what `flaw.inscribed_shadow` ships (`uncomputed_rule`,
full passage in both locales). The **gloss** is hedged, and D16's table maps hedged
to a warning — but there is nothing to warn *about*: a warning here would enforce
the author's observation about the typical case, which is stricter than the one
restriction the book imposes. A hard `Prereq::House` would be stricter still, and
would forbid the non-Criamon stigmatic the sentence's own *"normally"* leaves room
for.

**The sizing sweep B14 asked for, done.** Grepping the English V/F block
(ArMDE:3300-7130) for `normally means` / `usually means` / `which normally` /
`which usually` returns **exactly one** hit: ArMDE:6320 itself. So this idiom is a
population of one and generalises to nothing — there is no family here needing a
rule.

**What it obliges.** Nothing. `flaw.inscribed_shadow` moves from *escalated* to
**checked and clean**. One thing is worth recording for the next hedged case that
*does* restrict: **`Prereq` has no warning severity** — `prereq_not_met` is an
error — so D16's hedged→warning rule currently has no carrier at the *entry* level,
only at the profile level where D16 was applied. That gap is real, it is just not
reachable from this entry.

**Status.** SETTLED

---

### Q-138 — ArMDE:6148 forbids a *class* of Flaws, and it sits on another batch's entry

> **REVISED — see § Reconciliation, divergence-list note and decision 2.** The
> second pass reads ArMDE:6148's clause as a constraint on the Flaw that Flawed
> Powers **imports**, not as an incompatibility — which would change what D23
> builds. That reading is strong and is now a narrow question for Norbert. The
> magnitude-qualifier constraint below stands under either reading.

**Verdict.** **D23 settles the mechanism and D12 supplies the predicate**, exactly
as D23 records. What is left for this pass is to name the entry, confirm it is
rated, and surface a design constraint neither D21 nor D23 currently carries: the
predicate must be able to name a **magnitude**, not only a category.

**Evidence — the entry and the clause.** ArMDE:6146-6148, `#### Flawed Powers`
(*Minor, Supernatural*), which is B13's span: *"The character must have **at least
one Major Supernatural Virtue** to take this Flaw. … **Any Flaw that is only
appropriate to Hermetic Magic (for example, Deficient Technique or Unstructured
Caster) cannot be taken with this Flaw.** Note that the restriction on the
supernatural power is not a separate Major Flaw…"* The entry ships
`classification: "uncomputed_rule"` with the whole passage in `description` in both
locales, and `RULES.md` records **both** restrictions as selection prerequisites
the engine does not express.

**What D23 already decided, so it is not re-litigated.** Predicate-valued
exclusions are ruled in; Q-138's predicate is D12's **trained** flag (ArMDE:6148's
two worked examples, `flaw.deficient_technique` and `flaw.unstructured_caster`, are
both squarely trained under D12's criterion); and the work is therefore blocked on
D12's classification pass over the 122 `hermetic` entries, not on D23 itself.
B18's reason for calling it harder than Q-137 — that `categories: ["hermetic"]`
includes Flaws that are not *only* Hermetic — is exactly the gap D12's pass closes.

**The entry is rated.** D23 notes Q-138 *"sits on a B13 entry and is carried by no
finding in B13, B17 or the index"*. That is right for the **exclusion clause**, but
the entry itself is not unrated: B13 checked it and passed it, resting on
`RULES.md`'s record that both clauses are inexpressible. So the residue is narrow
and precise: **when D23 lands, that `RULES.md` record stops being true for the
exclusion half and must be rewritten**, the same way D14 required of the Covenant
Upbringing record.

**The new constraint — and this is the part worth carrying.** The **first**
restriction in the same sentence is *"at least one **Major** Supernatural
Virtue"*. That is D21's category-ranging `Prereq` **plus a magnitude filter plus a
minimum count**, and D21's ruling records none of the three together — its worked
case, `flaw.rector`'s "a Social Status Virtue", needs neither magnitude nor count.
Three independent sites want the magnitude qualifier:

| Site | The clause | Needs |
|---|---|---|
| `flaw.flawed_powers`, ArMDE:6148 | "at least one **Major** Supernatural Virtue" | category + magnitude + count ≥ 1 |
| `flaw.flawed_powers`, ArMDE:6148 | "a **Major** Hermetic Flaw (commonly Restriction or Necessary Condition)" | the same, on a parameter (Q-104) |
| `flaw.false_power`, ArMDE:6082 | the Supernatural-Virtue predicate | Q-103, inclusion side |

**What it obliges.**

1. **Fold the magnitude qualifier into D21 + D23's shared design**, so the three
   sites above are served by one mechanism. D21's own argument — *"decide once; the
   alternative is four mechanisms for one idea"* — applies with one more axis.
2. **Rewrite `RULES.md`'s "inexpressible" record for `flaw.flawed_powers`** when
   the mechanism lands, rather than leaving it asserting a limitation that no
   longer holds.
3. **No data change now, and no new finding against the entry.** B13's verdict
   stands; Q-104's asymmetry (no parameter recording *which* Major Hermetic Flaw
   applies) is a separate, still-open question.

**Status.** SETTLED on the rules question and on the entry's status. Remedy
**BLOCKED** on D12's classification pass, then D23 (with the magnitude qualifier
added to its scope).

---

## Summary

**Reconciled against the independent second derivation** (§ Reconciliation above);
this table reflects the resolved positions, not the first-pass ones.

All 31 are settled on the **question asked**. Three carry a narrow follow-on
decision for Norbert (Q-103, Q-138, Q-67) — none of them the question the batch
raised, all of them cost or policy calls the source cannot settle. Five more carry
a remedy blocked on machinery another decision already owns.

| Q | Verdict | Status | Findings / rulings touched |
|---|---|---|---|
| Q-16 | no-inherit | SETTLED | F-62 (cross-reference satisfied); no data change |
| Q-18 | correct | SETTLED | F-81 unaffected; D12 (Gentle Gift is the *intrinsic* model) |
| Q-20 | no | SETTLED | F-68 unaffected; **new D9 instance** (pretended-status parameter) |
| Q-25 | uncomputed_rule | SETTLED (revised) | ArMDE:2264 + D3; D12 (`IsMagus`), D2/F-466 (`Prereq::Nor`) |
| Q-29 | agree | SETTLED (revised, **arguable**) | F-94 confirmed; blocked-behind F-141; D10 breaks the tie toward *once* |
| Q-33 | four | SETTLED | D25 errata note; D11 (enforcement makes the book's own example invalid) |
| Q-38 | **once** | SETTLED (revised, **arguable**) | F-140, F-141; D10; ArMDE:2814 + the Lesser Power precedent |
| Q-39 | Linguist clean | SETTLED (revised) | **new defect**: `effective/xp.rs::charged_cost` overcharges `virtue.affinity_ability` at 7 Ability scores |
| Q-41 | correct | SETTLED (**challenged and confirmed**) | none — do not remove `supernatural` |
| Q-42 | exclusive | SETTLED (remedy blocked) | F-427, Q-07, Q-102, **Q-132** — and a new constraint: ArMDE:2816 needs a *scoped* exception |
| Q-50 | correct | SETTLED (+ **new `high` defect**) | none on Nephilim; drop `bonus_flaw_points: 7` from `devil_child` and `spirit_votary` |
| Q-53 | five | SETTLED | doc comment on `ability.rs::AbilityCategory`; no data change |
| Q-55 | neither | SETTLED | **F-450's defect on a second entry**; D11 |
| Q-57 | neither | SETTLED | F-226; D10's shape; **new**: doubled clause in the English source at ArMDE:4742 |
| Q-62 | nowhere | SETTLED | F-249 distinguished (Ripper is *not* settled by this) |
| Q-67 | rate | SETTLED (exact model blocked) | effect-less `creation_effect`; D3/D20; **D16/Q-05's count is short by one** |
| Q-68 | bold | SETTLED | F-301 shrinks; F-239 takes the same answer |
| Q-70 | no | SETTLED | none — census: 21 descriptors ↔ 23 ids, exact |
| Q-78 | yes | SETTLED | F-332 (text half stands); small scoped change on `CrisisOutcome` |
| Q-79 | descriptor | SETTLED (D25) | D25's errata note + its "is it the only one" sweep |
| Q-81 | label, not data | SETTLED (revised) | **F-351**: fix the doc comment + the Fluent label, data untouched; **F-352 confirmed outright**; both entries join D10's 42 |
| Q-89 | misprint | SETTLED | D25 errata note; F-400 unaffected; `scheitest` governs, D26 does not |
| Q-103 | over-permits by 55 | SETTLED; **remedy = narrow decision** | one-id whitelist vs D23's predicate; `RULES.md`'s approximation record must be rewritten |
| Q-109 | Fury's side | SETTLED | **F-20 confirmed**; `flaw.fury` now **clean**; D4 over D1 |
| Q-110 | no conflict | SETTLED | **F-440 unblocked**; D16; five-entry mandated-trait family; F-542 |
| Q-112 | yes | SETTLED | **F-448 unblocked**; `virtue.magic_items` confirmed *correctly* narrow |
| Q-115 | hedged → warning | SETTLED (revised) | `flaw.inscribed_shadow` → checked; **a warning is owed** (D16/Q-139); `Prereq` has no warning severity |
| Q-116 | five scores | SETTLED | **`flaw.lame` + `flaw.hobbled` defective at `high`**; `RULES.md` block rewritten; scope **2**, not 6 |
| Q-117 | is a rule | SETTLED (D20 + D3) | `NO_RULE_DESPITE_TOKEN` row deleted; D19/§ 2.1b queue |
| Q-125 | supernatural | SETTLED (D25) | D25 errata note; F-498 unaffected |

### The six whose *remedy* waits on machinery

| Q | Blocked on |
|---|---|
| Q-42 | Q-132 / F-427 — the category mechanism, plus a **scoped** exception |
| Q-67 | a **numeric** parameter type + a parameter-scaled `RestrictedAbilityXp` (interim: `uncomputed_rule` + text, not blocked) |
| Q-138 | D12's classification pass, then D23 + a **magnitude** qualifier |
| Q-29 / Q-38 | F-141's parameter, then D10's explicit declaration |
| Q-110 | a "mandates N traits at value M" field, designed with D21's Effect-side twin |
| Q-115 | a **warning severity for `Prereq`**, which does not exist at the entry level |

### Things nobody asked about, found on the way

1. **Two of the four Mythic Companion types are over-budgeted by 7 Flaw / 14
   Virtue points** — `bonus_flaw_points: 7` on `mythic_type.devil_child` and
   `mythic_type.spirit_votary` reads ArMDE:2664's *"an **additional** seven points
   of Flaws"* as a bonus on top of ArMDE:2638's ten, when 3 (the compulsory Major
   Flaw) + 7 **is** the ten. Verified for Devil Child; Spirit Votary's RoP:M:5486
   citation is relayed, not verified. **The largest single defect this pass found,
   and no existing finding covers it.**
2. **`charged_cost` floors where the book ceils** (Q-39) — correct charge
   `(T - 1) · den / num + 1`. Live on `virtue.affinity_ability` at Ability scores
   1, 4, 7, 10, 13, 16, 19; `virtue.linguist` and every Art are provably clean.
3. **The `casting_fatigue` read-out prints backwards** (Q-81) — a *Virtue* shows
   *"Casting fatigue: +1"*. The data convention is right; the label and the doc
   comment are wrong. And the Vulnerable/Withstand mutual exclusion is stated
   verbatim on **both** sides, confirming F-352 with no inference.
4. **Three unencoded absolutes on `virtue.redcap`** — ArMDE:4848's free
   Well-Traveled Virtue (which Lone Redcap grants and Redcap does not), and
   ArMDE:4850's *"cannot take the Wealthy Virtue or Poor Flaw"* and *"may not take
   The Gift"*. All expressible today; all absent. Compounds D17's finding that the
   entry encodes its 300 XP not at all.
5. **`virtue.berserk` grants a Personality Trait (Angry +2) that nothing encodes**
   (Q-109), and it is one of at least **five** V/F in the block that mandate a
   trait at a stated value (Q-110).
6. **A second female-only Social Status exception** (`virtue.simple_student`,
   ArMDE:4960), so D16's "20 male-only plus one female-only" count is short.
7. **A doubled clause in the English source at ArMDE:4742** (*"unlike a Magical
   Focus, unlike a Magical Focus"*), clean in DE:4742 — an extraction artefact
   outside D26's closed scope.
8. **`Prereq` has no warning severity** (Q-115), so D16's hedged→warning rule has
   no carrier at the entry level. Q-115 makes this **live**, not latent.
9. **`tainted` maps exactly onto the descriptor tag** (Q-70: 21 descriptor lines ↔
   23 ids, the gap fully explained by two "Major or Minor" lines). Both sides are
   mechanically derivable, so this is a cheap data-integrity guard nobody has
   written.
10. **Marking `virtue.sense_passions` tainted would make `flaw.false_power`
    (Sense Passions) unselectable**, because the parameter carries
    `forbid_tainted: true` — the arm ArMDE:7737 names *first*. A boolean cannot
    hold that passage's disjunction (found by the second pass).
11. **`flaw.deficient_form`'s name is the flat `"Deficient Form"` despite carrying
    a `form` parameter**, where its twin is `"Deficient {technique}"` — so two
    Deficient Form selections render identically in both locales. Found by the
    second pass alongside Q-89; **verified here** — EN `"Deficient Form"` vs
    `"Deficient {technique}"`, DE `"Defizitäre Form"` vs
    `"Defizitäre {technique}"`.
