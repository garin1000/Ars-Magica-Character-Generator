# Manual testing findings — 2026-11-15

Norbert's manual test of 0.5.0 on Windows (portable zip, `main` at `df634561`). The first pass ran on a stale 0.5.0-beta exe; F4 and F6 came from that pass and are withdrawn. Fixed in the 2026-11-15/16 fix round (plan reviewed by Fable; decisions D93.1–D93.3); a retest on Windows is pending.

Retest of the 2026-10-18 findings: all passed on 0.5.0 except #15 (F20) and the alignment rule of #5c/#7 (superseded by F12).

| # | Area | Finding | Status |
|---|---|---|---|
| 1 | Window title | Show the open file's name in the window title, with "*" while there are unsaved changes. **Root cause:** `setTitle`/`setFullscreen` were denied by the `core:default` capability and the error swallowed (Window > Fullscreen was broken too). Now Rust commands; title "`<character> (<file>)`" (Norbert). | done `765a3d44` |
| 2 | Menu, DE | The German menu shows shortcut labels as "Ctrl", not "Strg". muda hardcodes the English names on Windows; the chord is now folded into the label with localized key names (Windows only; muda's predefined Edit items keep "Ctrl"). | done `765a3d44` |
| 3 | V/F: realm association | The Realm dropdown on a Supernatural V/F row (D42) is a bare select with no explanation, takes much space, and is misaligned. Left-align it with the text field above. | done `73924278` |
| 4 | — | Withdrawn (stale beta exe). | withdrawn |
| 5 | Unsaved-changes dialog | The default button is Discard; it must be Cancel (Enter must never discard work). rfd hardcodes the first button as default (Windows, macOS) and sets none on GTK; new native dialogs per platform (open-todos row 62: return to rfd once fixed). Manual Windows check pending. | done `894d5e54` |
| 6 | — | Withdrawn (stale beta exe). | withdrawn |
| 7 | V/F picker | A grog can pick Redcap (Major, `ArMDE:4915-4916`) in Enforced mode, though grogs "may not take Major Virtues or Flaws" (`ArMDE:2898`). The add-verdict now refuses hard-cap breaches. | done `b9b8efa2` |
| 8 | V/F picker | Brutal Artist is not greyed while the House is unset. **Decided (Norbert, 2026-11-15):** greyed out whenever the House is not Jerbiton, including "House not set" (D93.2). | done `2694b7d9` |
| 9 | Numeric inputs | Gauntlet age and other number fields are fiddly to type: per-keystroke checks block normal typing. Ten components moved to commit-time clamping; Gauntlet age itself awaits a concrete repro. | done `365a17b6` |
| 10 | Life stages | With Age unset, nothing shows next to "Age"; show "unknown" or similar. | done `2ad3c47c` |
| 11 | Life stages | "Age" and "Gauntlet age" are not on one baseline. | done `73924278` |
| 12 | Life stages | The lab-years box must share the row's text baseline, not be centred. Supersedes #5c/#7. | done `73924278` |
| 13 | Life stages | With Age unset, life stages don't work well. Show an inline notice in the Life-stages area asking to set the age first. | done `2ad3c47c` |
| 14 | Life sections | The "sections exceed the age" error text should be bold and red. | done `10febd3f` |
| 15 | Life sections | A section's lab years exceeding its years must be marked inline in the section row. | done `10febd3f` |
| 16 | Spells tab | Show the life timeline on the Spells tab too (`ArMDE:2541`). | done `cf9f2cef` |
| 17 | XP bar | The apprenticeship bar mixed its 120 spell levels into the XP fill (`ArMDE:2505`). Engine verified: never pooled. The bar now fills from XP alone. | done `0cde263c` |
| 18 | Abilities | With two (Area) Lore entries, one with Area empty, both rows were marked red. | done `a567b4e1` |
| 19 | Spells tab | "Within focus" label and the Form selector not on the spell name's baseline. | done `73924278` |
| 20 | Aging tab | Aging-point distribution inputs showed only spinner arrows on Windows (a scoped `width: 4ch`). | done `07c4fa04` |
| 21 | Aging tab | Columns left-aligned in a full-width panel. Norbert chose one panel per column, three equal widths filling the row. | done `73924278` |
| 22 | Aging tab | The Aging-point distribution accepted more than owed ("5 of 1 placed"). | done `07c4fa04` |
| 23 | Life timeline | "Well-Traveled: 50 / 50 XP" was printed under Early childhood. Bonus pools now have their own row ("Bonus XP:"). | done `0cde263c` |
| 24 | Wizard | No life timeline on the wizard's Abilities step. Now on Abilities, Arts and Spells. | done `cf9f2cef` |
| 25 | Ability picker | In Enforced mode, gated categories the character cannot fund were not greyed out. Engine verdict shared with the validator. | done `676f89fb` |
| 26 | Reputations | The label and the text input are not on one baseline. | done `73924278` |
| 27 | V/F picker | Vengeful Powers must not be listed under Hermetic for non-magi (D93.3). | done `fa89c313` |
| 28 | **Rules: XP flow (HIGH)** | The native language was funded from Well-Traveled before the native-language pool (`ArMDE:2448`, `ArMDE:5314`). Restricted pools now drain narrowest first; a sweep over all 49 overlapping pool pairs found and fixed a second case (Ineslemen vs Arcane Lore) (D93.1). | done `e08b0e74` |
| 29 | Childhood | "Take this childhood" stayed visible after the childhood was taken. | done `8120ecee` |
| 30 | Markdown export | Combat values carry an explicit "+" on positive values (also in the in-app Combat section). | done `e826a2fe` |

## Retest 2026-11-17/18

All 27 steps passed (`tmp/tryout-0.5.0-fixes.md`). R1, R2, R3 and R7 were deferred until after 0.5.0 (open-todos rows 64–67). Norbert put three into 0.5.0:

| # | Area | Finding | Status |
|---|---|---|---|
| R4 | Wizard | Unspent restricted XP warned on the Experience step, where nothing is spent. Now on Abilities; the native-language missing-score warning shows on both steps. The editor shows all issues on every tab, so it is unaffected. | done `54637d55` |
| R5 | Life timeline, XP bar | Bonus pools leave the timeline (reverses #23's "Bonus XP:" row). The XP bar names them by Virtue, with the Ability list as tooltip. The Markdown export keeps them. | done `20e5fd04` |
| R6 | Spells | Regression from #19: the Form select was clipped on Windows. `padding-block: 0` inside the same 1.9rem box. | done `20e5fd04` |
