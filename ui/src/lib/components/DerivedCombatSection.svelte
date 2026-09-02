<script lang="ts">
  import { store } from '../state.svelte';
  import { combatRowLabel } from '../derive';
  import type { DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Always
  // rendered (not magus-only), matching the original.

  let { d }: { d: DerivedTotals } = $props();

  // Localized display name for a catalogue id (weapon/shield here), id as
  // fallback.
  function name(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }
</script>

<!-- Combat lines -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('derived-section-combat')}</h3>
  {#if d.combat.length > 0}
    <div class="table-scroll">
      <table class="derived-table combat" data-testid="derived-combat">
        <thead>
          <tr>
            <th></th>
            <th>{store.t('derived-combat-init')}</th>
            <th>{store.t('derived-combat-attack')}</th>
            <th>{store.t('derived-combat-defense')}</th>
            <th>{store.t('derived-combat-damage')}</th>
            <th>{store.t('derived-range')}</th>
          </tr>
        </thead>
        <tbody>
          <!-- Deliberately UNKEYED, same reason as the Penetration list:
               `combat_totals` emits one line per equipped slot, so carrying the
               same weapon twice repeats `line.weapon` — and with a shield equipped
               a single one-handed weapon repeats it too, since it yields a
               with-shield line and a bare one. Keying by it threw
               `each_key_duplicate` and killed this whole tab. -->
          {#each d.combat as line}
            <tr>
              <th>
                {combatRowLabel(
                  name(line.weapon),
                  (line.shields ?? []).map(name),
                  store.t('derived-combat-shield-joiner'),
                )}
              </th>
              <td>{line.initiative}</td>
              <td>{line.attack ?? '—'}</td>
              <td>{line.defense}</td>
              <td>{line.damage ?? '—'}</td>
              <td>{line.range ?? '—'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <p class="empty">{store.t('derived-combat-empty')}</p>
  {/if}
</div>

<style>
  /* `.detail-section` is a shared global in app.css; `.empty` follows the
     project's per-component convention (see `TalismanPanel`, `FamiliarPanel`,
     `MagicPossessions`). */
  .empty {
    opacity: 0.7;
  }
</style>
