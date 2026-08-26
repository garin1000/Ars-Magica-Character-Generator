<script lang="ts">
  import LifeStagePanel from './LifeStagePanel.svelte';
</script>

<!-- Where the character's experience comes from, and — under guided funding — the
     plan it is priced from: the age, a magus's Gauntlet age and post-Gauntlet
     seasons and spell-level split, the native language, and the sample childhood
     with its slot inputs and preview
     (Ars Magica - Definitive Edition (Core Rules).md:2364, :2213-2216).

     Its own step rather than a preamble on the Abilities step (guided-creation
     review #11): three concerns shared one bounded flex column there — this panel
     (≈620px), the Hermetic-minimums checklist, and only then the Available/Selected
     row, which got whatever was left and hit its floor. Both preambles were
     auto-height siblings, so in an 800px window the row measured 0 and both ability
     lists disappeared. Splitting the phase gives each concern its own height.

     A thin wrapper on purpose: the panel is the same component the editor mounts,
     so the wizard's step and the editor's surface can never drift apart. The
     childhood picker is mounted by the panel itself, after the native language it
     is measured against. -->
<LifeStagePanel />
