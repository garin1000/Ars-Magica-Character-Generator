# Open to-dos

Items waiting on a decision, a visual check, or a follow-up pass. Kept here so
they survive a session ending. Findings that are merely *implemented* live in
their own findings document; this list is what is still owed.

**Surface this list when a release or a git tag is being prepared** — none of
these should be tagged over silently.

| # | Item | Waiting on | Raised |
|---|---|---|---|
| 1 | `virtue.spirit_votary`'s **+7 Flaw points** in `rules/core/mythic_companion_types.json` cites core `:2741-2764`, but those lines never state the number; only *RoP: Magic* `:5486` does. Either the citation is wrong or the value is unsourced. English core is the source of truth for values, so this needs a decision, not a silent fix. Finding 28. | Norbert | 2026-09-03 |
| 3 | **Items the rules forbid repeating can still be repeated.** The duplicate check keys on `(item, params)`, so anything carrying a target can be taken once per target however firmly the rules forbid it: Inoffensive to (Beings) :4139, Fish out of Water :6132, Offensive to (Beings) :6530, Unbearable to (Beings) :6897. Affinity with (Art) and Puissant (Art) say "twice, for two different Arts" and the app allows fifteen. Needs a cap on total copies across targets — a model change, not a data edit. Finding 35. | decision on scope | 2026-09-05 |
| 4 | **`flaw.false_power` cannot be made repeatable as data.** It repeats "in each subsequent instance as a Minor Flaw rather than a Major one" (:6096), and magnitude belongs to the catalogue entry, not the selection, so every copy would cost 3 points instead of 3-then-1. Left non-repeatable. The clean fix is a second entry plus its EN/DE text, the way `virtue.amorphous_major` / `_minor` already works. Part of finding 34. | decision | 2026-09-05 |
| 5 | **Demonic Might and Demonic Powers cap at "no more than half the character's total Virtues"** (:3665, :3669). That is a whole-build ratio and the engine has only absolute caps, so it is unenforced and left to the troupe. Recorded in RULES.md; raise it here in case you want it modelled. Part of finding 34. | decision | 2026-09-05 |

## Done since this list was started

- Dual-category Virtues/Flaws now appear under every category heading in the
  Available list (the rulebook indexes each of them twice); "primary" survives
  only as a tie-break where a surface structurally holds one value.
- `Mythic Companion` became a real category, which stopped a grog taking Devil
  Child.
- The 11.25px tab labels were checked in the running app and read well
  (2026-09-05), so the German labels stay as they are.
