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

  // What a cell with no value reads (Attack/Damage/Range are optional per line).
  // A Fluent key, not the literal em dash it used to be: a screen reader
  // announces a bare dash as nothing, so N/A was indistinguishable from a cell
  // that had failed to render.
  const notApplicable = $derived(store.t('derived-not-applicable'));
</script>

<!-- Combat lines -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('derived-section-combat')}</h3>
  {#if d.combat.length > 0}
    <div class="table-scroll">
      <table class="derived-table combat" data-testid="derived-combat">
        <!-- `scope` on both axes (Sabine 9, round-1 audit): this table has a
             header ROW and a header COLUMN, and with two header axes the
             cell→header association is not inferable, so a screen reader reads
             the body as bare numbers. The empty corner cell is scoped too — it
             heads the weapon-name column. -->
        <thead>
          <tr>
            <th scope="col"></th>
            <th scope="col">{store.t('derived-combat-init')}</th>
            <th scope="col">{store.t('derived-combat-attack')}</th>
            <th scope="col">{store.t('derived-combat-defense')}</th>
            <th scope="col">{store.t('derived-combat-damage')}</th>
            <th scope="col">{store.t('derived-range')}</th>
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
              <th scope="row">
                {combatRowLabel(
                  name(line.weapon),
                  (line.shields ?? []).map(name),
                  store.t('derived-combat-shield-joiner'),
                )}
              </th>
              <td>{line.initiative}</td>
              <td>{line.attack ?? notApplicable}</td>
              <td>{line.defense}</td>
              <td>{line.damage ?? notApplicable}</td>
              <td>{line.range ?? notApplicable}</td>
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
