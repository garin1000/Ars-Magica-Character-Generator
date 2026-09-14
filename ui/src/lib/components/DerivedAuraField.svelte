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
  // What the hint reports is the CLAMP, not the stored value's range — Gerda #1 /
  // Sabine #2, full-audit round 2. A stored aura can no longer be out of range at
  // all: `state.svelte.ts::AppStore.setAura` clamps what is typed, and
  // `migration.rs::load_entity_migrating` clamps what is loaded (ArMDE:17390,
  // :17404-17409), which between them is every route into the store. Testing the
  // stored value therefore asked a question with only one answer, leaving the
  // field free to overrule the player in complete silence. It reports the event
  // that does happen instead: the entry the store rewrote.
  let clampedEntry = $state(false);

  function onAura(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    const requested = raw === '' ? null : Number(raw);
    store.setAura(requested);
    // A cleared field is not the player being overruled — it is no entry at all,
    // which the store reads as 0.
    clampedEntry = requested !== null && requested !== store.entity.aura;
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
      aria-describedby={clampedEntry ? 'derived-aura-out-of-range' : undefined}
      oninput={onAura}
      data-testid="derived-aura-input"
    />
  </label>
  {#if clampedEntry}
    <!-- role="status" (polite): announced once the entry is rewritten, without
         interrupting the keystroke the player is mid-typing, the same
         politeness BalanceBar/XpBar use for their own live totals. -->
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
