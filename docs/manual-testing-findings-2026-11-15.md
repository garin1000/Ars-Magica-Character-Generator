# Manual testing findings — 2026-11-15

Norbert's manual test of 0.5.0 on Windows (portable zip, `main` at `df634561`). The first pass ran on a stale 0.5.0-beta exe; F4 and F6 came from that pass and are withdrawn. Collected for the next fix run; nothing here is fixed yet.

Retest of the 2026-10-18 findings: all passed on 0.5.0 except #15 (F20) and the alignment rule of #5c/#7 (superseded by F12).

| # | Area | Finding | Status |
|---|---|---|---|
| 1 | Window title | Show the open file's name in the window title, with "*" while there are unsaved changes. | open |
| 2 | Menu, DE | The German menu shows shortcut labels as "Ctrl", not "Strg". The labels are drawn by muda; check feasibility first. | open |
| 3 | V/F: realm association | The Realm dropdown on a Supernatural V/F row (D42) is a bare select with no explanation, takes much space, and is misaligned. Left-align it with the text field above. | open |
| 4 | — | Withdrawn (stale beta exe). | withdrawn |
| 5 | Unsaved-changes dialog | The default button is Discard; it must be Cancel (Enter must never discard work). | open |
| 6 | — | Withdrawn (stale beta exe). | withdrawn |
| 7 | V/F picker | A grog can pick Redcap (Major, `ArMDE:4915-4916`) in Enforced mode, though grogs "may not take Major Virtues or Flaws" (`ArMDE:2898`). The engine raises the error; the add-verdict (#2/#19) must also refuse items that break a hard magnitude cap. | open |
| 8 | V/F picker | Brutal Artist is not greyed while the House is unset (House prereq is Unknown, by design). **Decided (Norbert, 2026-11-15):** greyed out whenever the House is not Jerbiton, including "House not set". So in the add-verdict an unset House counts as not meeting a `House` prerequisite; validation of an already-held item is unchanged. | open |
| 9 | Numeric inputs | Gauntlet age and other number fields are fiddly to type: per-keystroke checks block normal typing. | open |
| 10 | Life stages | With Age unset, nothing shows next to "Age"; show "unknown" or similar. | open |
| 11 | Life stages | "Age" and "Gauntlet age" are not on one baseline. | open |
| 12 | Life stages | The lab-years box must share the row's text baseline, not be centred: without sections, "Levels of spells" lines up with "1 season … 4 seasons"; with sections, "Name" lines up with the section's season labels. Supersedes #5c/#7. | open |
| 13 | Life stages | With Age unset, life stages don't work well. Show an inline notice in the Life-stages area asking to set the age first, not only a footer issue. | open |
| 14 | Life sections | The "sections exceed the age" error text should be bold and red. | open |
| 15 | Life sections | A section's lab years exceeding its years must be marked inline in the section row, like Ability/spell errors. | open |
| 16 | Spells tab | Show the life timeline on the Spells tab too: post-Gauntlet points buy spell levels from the same pool as XP (`ArMDE:2541`). | open |
| 17 | XP bar | The XP bar shows apprenticeship's 120 spell levels as XP; they are a separate budget (`ArMDE:2505`). Fix the display, and verify the engine never pools them. | open |
| 18 | Abilities | With two (Area) Lore entries, one filled and one with Area empty, both rows are marked red; only the empty one should be. | open |
| 19 | Spells tab | In a spell row, the "Within focus" label and the Form selector (e.g. Unravelling the Fabric of (Form)) are not on the spell name's baseline. | open |
| 20 | Aging tab | #15 not fixed on Windows: the Aging-point distribution inputs still show only spinner arrows, no number. | open |
| 21 | Aging tab | The three columns sit left-aligned while the bright panel spans the full width. Either one panel per column, all equal width filling the row, or one panel sized to the columns and centred. | open |
| 22 | Aging tab | The Aging-point distribution accepts more than owed ("5 of 1 placed"); Apply stays disabled, but each input should cap at the points still unplaced. | open |
| 23 | Life timeline | "Well-Traveled: 50 / 50 XP" is printed under Early childhood, as if it belonged to that stage. Show bonus pools apart from the stages. | open |
| 24 | Wizard | No life timeline on the wizard's Abilities step (#10 covered only the editor tabs). | open |
| 25 | Ability picker | In Enforced mode, gated categories (Arcane, Academic, Martial, Supernatural) the character cannot fund are not greyed out. Same treatment as the V/F picker. | open |
| 26 | Reputations | The label ("Academic level 2 from Failed Student") and the text input are not on one baseline. | open |
| 27 | V/F picker | Vengeful Powers (a Story Flaw that "may be taken as a Hermetic Flaw", `ArMDE:7048`) must not be listed under Hermetic for non-magi. Decided by Norbert. | open |
| 28 | **Rules: XP flow (HIGH)** | The native language is funded from Well-Traveled before the native-language pool: Athletic Childhood's German 5 (75 XP) took 25 from native language and 50 from Well-Traveled (Living Language is on its list, `ArMDE:5314`), wasting 50 native-language XP (`ArMDE:2448`) and consuming Well-Traveled. The narrowest restricted pool must drain first; same tie-break class as 2026-10-18 #11 (`effective/xp.rs::two_phase_max_flow`). Check every bonus-pool overlap. | open |
| 29 | Childhood | "Take this childhood" stays visible after the childhood was taken. | open |
| 30 | Markdown export | Combat values should carry an explicit "+" on positive values, like the characteristics (`formatSigned`). | open |
