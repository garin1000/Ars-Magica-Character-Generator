# Open to-dos

Items waiting on a decision, a visual check, or a follow-up pass. Kept here so
they survive a session ending. Findings that are merely *implemented* live in
their own findings document; this list is what is still owed.

**Surface this list when a release or a git tag is being prepared** — none of
these should be tagged over silently.

| # | Item | Waiting on | Raised |
|---|---|---|---|
| 4 | **`flaw.false_power` cannot be made repeatable as data.** It repeats "in each subsequent instance as a Minor Flaw rather than a Major one" (:6096), and magnitude belongs to the catalogue entry, not the selection, so every copy would cost 3 points instead of 3-then-1. Left non-repeatable. The clean fix is a second entry plus its EN/DE text, the way `virtue.amorphous_major` / `_minor` already works. Part of finding 34. | decision | 2026-09-05 |
| 5 | **Demonic Might and Demonic Powers cap at "no more than half the character's total Virtues"** (:3665, :3669). That is a whole-build ratio and the engine has only absolute caps, so it is unenforced and left to the troupe. Recorded in RULES.md; raise it here in case you want it modelled. Part of finding 34. | decision | 2026-09-05 |
| 6 | **A data-declared enumerated parameter domain would fix `virtue.folk_magic`** (finding 37 — four spell categories, a closed list at :3909-3917) and would also turn the free-text `being` slots on Inoffensive to (Beings), Offensive to (Beings) and Unbearable to (Beings) into real dropdowns, since all three enumerate a closed being list today stored as free text. `flaw.fish_out_of_water_terrain` is **not** a candidate: its terrain list ends "…, etc." (:6130), so free text is what the book means there. `ParameterDomain` (`crates/arm-rules/src/types.rs:424-447`) has no enumerated-domain variant today; building one is the prerequisite. | decision | 2026-09-06 |
| 7 | **The per-power cap has no per-power target.** `flaw.slow_power` (:6761), `virtue.variable_power` (:5205) and `flaw.restricted_power` (:6689, finding 36) all cap "per power the character possesses", but no selection records *which* power, so nothing stops two copies naming the same one. | decision | 2026-09-06 |
| 8 | **GitHub issue #3 is fixed but not yet answered or closed on GitHub.** Finding 34 fixed the repeatable-Virtues bug it reported (Improved Characteristics and 24 others); the issue itself still needs a reply and closing. | reply + close on GitHub | 2026-09-06 |

## Done since this list was started

- Dual-category Virtues/Flaws now appear under every category heading in the
  Available list (the rulebook indexes each of them twice); "primary" survives
  only as a tie-break where a surface structurally holds one value.
- `Mythic Companion` became a real category, which stopped a grog taking Devil
  Child.
- The 11.25px tab labels were checked in the running app and read well
  (2026-09-05), so the German labels stay as they are.
- Spirit Votary's +7 Flaw points turned out to be the standard Mythic Companion
  arithmetic — ten points of Flaws at 2:1, minus the 3 that Pagan spends funding
  the 6 points of required Virtues. Core does state it, just not as a number;
  RULES.md carries the derivation now (2026-09-06).
- The total-copies cap the duplicate check was missing is now data (`max_total`
  beside `max_per_target`), closing Inoffensive/Offensive/Unbearable to
  (Beings), Fish out of Water, and Affinity/Puissant Art at the ceilings their
  own descriptors state — finding 35, fixed 2026-09-06.
