<script lang="ts">
  import { store } from '../state.svelte';
  import { I32_MAX, I32_MIN } from '../derive';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Magus-only
  // (the parent mounts this only inside its own `{#if d.is_magus}` block), and
  // otherwise fully self-sufficient off the store — mirroring how
  // `FamiliarPanel`/`TalismanPanel` mount beside `MagicPossessions` with no
  // props: nothing here comes from the parent's own `derived` read-out.
  const aura = $derived(store.entity.aura ?? 0);

  // The aura's rules-legal range comes from the engine (`Ruleset.aura_modifier_min/
  // max`, round 3 Task 3), never a hardcoded -50/10 — this panel and the Magic
  // Items tab's input read the same engine-surfaced bound so the two entry points
  // cannot disagree. The i32-extreme fallback below is a defensive "no constraint"
  // sentinel for a ruleset payload predating this field, not a restatement of the
  // rule itself (this input only renders once `store.ruleset` is loaded, so the
  // fallback is not expected to be exercised in practice).
  const auraMin = $derived(store.ruleset?.ruleset.aura_modifier_min ?? I32_MIN);
  const auraMax = $derived(store.ruleset?.ruleset.aura_modifier_max ?? I32_MAX);
  // A value outside the engine's bound is not rejected here — `Entity.normalize()`
  // silently clamps it on save (Ars Magica - Definitive Edition (Core Rules).md:17390,
  // :17404-17409) — so this warns the player instead of letting the number change
  // out from under them with no explanation.
  const auraOutOfRange = $derived(aura < auraMin || aura > auraMax);

  function onAura(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    store.setAura(raw === '' ? null : Number(raw));
  }
</script>

<div class="detail-section">
  <label class="field">
    <span>{store.t('derived-aura-label')}</span>
    <!-- Same signed field as the Magic Items tab's aura input; the bounds
         come from the engine (Ruleset.aura_modifier_min/max) so the two entry
         points cannot disagree. -->
    <input
      type="number"
      min={auraMin}
      max={auraMax}
      value={aura}
      aria-describedby={auraOutOfRange ? 'derived-aura-out-of-range' : undefined}
      oninput={onAura}
      data-testid="derived-aura-input"
    />
  </label>
  {#if auraOutOfRange}
    <!-- role="status" (polite): announced once the value goes out of range
         without interrupting the keystroke the player is mid-typing, the
         same politeness BalanceBar/XpBar use for their own live totals. -->
    <p
      class="hint"
      id="derived-aura-out-of-range"
      role="status"
      data-testid="derived-aura-out-of-range"
    >
      {store.t('derived-aura-out-of-range', { min: String(auraMin), max: String(auraMax) })}
    </p>
  {/if}
</div>

<style>
  /* `.detail-section` and `.field` are shared globals in app.css; `.hint`
     follows the project's per-component convention (see `TalismanPanel`,
     `FamiliarPanel`, `MagicPossessions`). */
  .hint {
    opacity: 0.7;
    font-size: 0.85em;
    font-style: italic;
  }
</style>
