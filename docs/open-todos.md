# Open to-dos

Items waiting on a decision, a visual check, or a follow-up pass. Kept here so
they survive a session ending. Findings that are merely *implemented* live in
their own findings document; this list is what is still owed.

**Surface this list when a release or a git tag is being prepared** — none of
these should be tagged over silently.

| # | Item | Waiting on | Raised |
|---|---|---|---|
| 1 | `virtue.spirit_votary`'s **+7 Flaw points** in `rules/core/mythic_companion_types.json` cites core `:2741-2764`, but those lines never state the number; only *RoP: Magic* `:5486` does. Either the citation is wrong or the value is unsourced. English core is the source of truth for values, so this needs a decision, not a silent fix. See finding 28 of `manual-testing-findings-2026-09-03.md`. | Norbert | 2026-09-03 |
| 2 | **Tab label type is now 11.25px** (`0.75rem`) — the largest size at which all thirteen German magus tabs fit the default window without ellipsis. Needs a look in the running app. If it reads too small, the alternative is shortening the German labels ("Persönlichkeit & Reputationen" is 29 characters) rather than shrinking further. | Norbert | 2026-09-04 |
| 3 | **Dual-category Virtues/Flaws must appear under *every* category heading in the Available list**, not only the first-listed one. The rulebook's own index lists each of them twice (Sufi under Supernatural :3179 *and* Social Status :3230; Suppressed Gift :5301/:5369; Raised from the Dead :5365/:5399; Visions :5517/:5561), so "primary" was a grouping convenience with no rules backing. The Selected list stays one row per selection — its remove buttons are index-addressed. | in progress | 2026-09-04 |
| 4 | **Finding 10 reopened.** `Mythic Companion` *is* a category in the rules' own taxonomy: `### Mythic Companion, Free` (:3329-3334) lists Devil Child, Faerie Doctor, Nephilim and Spirit Votary, and none of them appears under `Social Status, Free` (:3336). Their descriptor `*Free, Mythic Companion*` puts it in the category slot, which `Tainted` never occupies. They are currently stored as `social_status`. | in progress | 2026-09-04 |
