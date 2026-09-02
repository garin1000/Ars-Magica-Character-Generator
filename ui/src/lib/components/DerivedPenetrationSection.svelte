<script lang="ts">
  import { store } from '../state.svelte';
  import { spellDisplayName } from '../derive';
  import type { DerivedTotals, PenetrationLine } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Magus-only
  // (the parent mounts this only inside its own `{#if d.is_magus}` block).
  // Self-gated on `d.penetration.length > 0`, matching the original inline
  // `{#if}` exactly, so the parent needs no per-section knowledge of which
  // sections can render empty.

  let { d }: { d: DerivedTotals } = $props();

  // A penetration line's spell label, interpolating the chosen target Form of a
  // parametrized meta-magic Vim spell so two instances of one spell id read
  // distinctly (e.g. "Wizard's Boost (Ignem)" vs "(Aquam)"). The parens belong
  // to the name template, so the hint is the plain param label.
  function penetrationLabel(line: PenetrationLine): string {
    const rs = store.ruleset;
    if (!rs) return line.spell;
    return spellDisplayName(rs, line.spell, line.parameter, (key) => store.t(`param-label-${key}`));
  }
</script>

{#if d.penetration.length > 0}
  <div class="detail-section">
    <h3 class="detail-label">{store.t('derived-section-penetration')}</h3>
    <ul class="derived-list" data-testid="derived-penetration">
      <!-- Deliberately UNKEYED. These are read-only engine output rows in a
           stable order with no identity to preserve, and no expression built
           from the line's own fields is unique: the same General spell known
           at two levels repeats (spell, parameter), and a duplicate key makes
           Svelte throw `each_key_duplicate`, which aborts the whole panel's
           render (the tab then appears dead). An unkeyed block cannot collide. -->
      {#each d.penetration as p}
        <li>
          <span>{penetrationLabel(p)} ({store.t('derived-level')} {p.level})</span>
          <span class="value"
            >{p.total}{p.weak_magic ? ' ' + store.t('derived-weak-magic') : ''}</span
          >
          {#if p.within_focus != null}
            <span class="derived-focus">{store.t('derived-within-focus')}: {p.within_focus}</span>
          {/if}
        </li>
      {/each}
    </ul>
  </div>
{/if}
