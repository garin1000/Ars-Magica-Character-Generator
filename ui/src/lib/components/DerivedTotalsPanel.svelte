<script lang="ts">
  // Read-only play-stat read-out (M5/5i). Renders the numbers the `derived_totals`
  // command computes; it performs NO mechanics in JS. Every label goes through a
  // Fluent `derived-*` key; catalogue ids resolve to their localized display name.
  import { store } from '../state.svelte';
  import type { Addend } from '../types';

  const d = $derived(store.derived);
  const aura = $derived(store.entity.aura ?? 0);

  // Localized display name for a catalogue id (Art, spell, weapon…), id as fallback.
  function name(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  // A labelled addend's display name (stable slug → Fluent).
  function addendLabel(a: Addend): string {
    return store.t(`derived-addend-${a.label}`);
  }

  // Signed rendering for a breakdown addend.
  function signed(n: number): string {
    return n >= 0 ? `+${n}` : `${n}`;
  }

  function breakdown(addends: Addend[]): string {
    return addends.map((a) => `${addendLabel(a)} ${signed(a.value)}`).join(', ');
  }

  function onAura(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    store.setAura(raw === '' ? null : Number(raw));
  }

  // Surfaced-modifier detail: enum scalars go through Fluent; free-text
  // (ability-roll subject) is shown as entered.
  function detailLabel(family: string, detail: string): string {
    if (family === 'ability_roll') return detail;
    return store.t(`derived-detail-${detail}`);
  }
</script>

<section class="panel derived-panel" data-testid="derived-panel">
  {#if !store.ruleset || !d}
    <p>{store.t('loading')}</p>
  {:else}
    <!-- Identity / summary -->
    <div class="detail-section">
      <h3 class="detail-label">{store.t('derived-section-summary')}</h3>
      <dl class="derived-grid">
        <dt>{store.t('derived-size')}</dt>
        <dd data-testid="derived-size">{d.size}</dd>
        <dt>{store.t('derived-section-decrepitude')}</dt>
        <dd data-testid="derived-decrepitude">{d.decrepitude_score}</dd>
        <dt>{store.t('derived-section-warping')}</dt>
        <dd data-testid="derived-warping">{d.warping_score} ({d.warping_points})</dd>
      </dl>
    </div>

    {#if d.is_magus}
      <div class="detail-section">
        <label class="field">
          <span>{store.t('derived-aura-label')}</span>
          <input type="number" value={aura} oninput={onAura} data-testid="derived-aura-input" />
        </label>
      </div>

      <!-- Lab Totals (5×10 grid) -->
      <div class="detail-section">
        <h3 class="detail-label">{store.t('derived-section-lab')}</h3>
        <div class="table-scroll">
          <table class="derived-table" data-testid="derived-lab-grid">
            <tbody>
              {#each d.lab_totals as cell (cell.technique + cell.form)}
                <tr>
                  <th>{name(cell.technique)} / {name(cell.form)}</th>
                  <td>{cell.total}{cell.deficient ? ' ' + store.t('derived-deficient') : ''}</td>
                  {#if cell.within_focus != null}
                    <td class="focus">{store.t('derived-within-focus')}: {cell.within_focus}</td>
                  {/if}
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>

      <!-- Casting Totals: four cast types per (Te, Fo) -->
      <div class="detail-section">
        <h3 class="detail-label">{store.t('derived-section-casting')}</h3>
        <div class="table-scroll">
          <table class="derived-table" data-testid="derived-casting-grid">
            <thead>
              <tr>
                <th></th>
                <th>{store.t('derived-cast-formulaic')}</th>
                <th>{store.t('derived-cast-ritual')}</th>
                <th>{store.t('derived-cast-spont-fatiguing')}</th>
                <th>{store.t('derived-cast-spont-non-fatiguing')}</th>
              </tr>
            </thead>
            <tbody>
              {#each d.casting_totals as cell (cell.technique + cell.form)}
                <tr>
                  <th title={breakdown(cell.addends)}
                    >{name(cell.technique)} / {name(cell.form)}{cell.deficient
                      ? ' ' + store.t('derived-deficient')
                      : ''}</th
                  >
                  <td>{cell.formulaic}</td>
                  <td>{cell.ritual}</td>
                  <td>{cell.spontaneous_fatiguing}</td>
                  <td>{cell.spontaneous_non_fatiguing}</td>
                </tr>
                {#if cell.within_focus}
                  <tr class="focus">
                    <th>{store.t('derived-within-focus')}</th>
                    <td>{cell.within_focus.formulaic}</td>
                    <td>{cell.within_focus.ritual}</td>
                    <td>{cell.within_focus.spontaneous_fatiguing}</td>
                    <td>{cell.within_focus.spontaneous_non_fatiguing}</td>
                  </tr>
                {/if}
                <tr class="non-standard">
                  <th
                    >{store.t('derived-cast-non-standard')}{cell.non_standard.deft_form
                      ? ' ' + store.t('derived-deft-form')
                      : ''}</th
                  >
                  <td>{store.t('derived-cast-silent')}: {cell.non_standard.silent}</td>
                  <td>{store.t('derived-cast-still')}: {cell.non_standard.still}</td>
                  <td colspan="2"
                    >{store.t('derived-cast-silent-still')}: {cell.non_standard
                      .silent_and_still}</td
                  >
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>

      <!-- Penetration (per known spell) -->
      {#if d.penetration.length > 0}
        <div class="detail-section">
          <h3 class="detail-label">{store.t('derived-section-penetration')}</h3>
          <ul class="derived-list" data-testid="derived-penetration">
            {#each d.penetration as p (p.spell)}
              <li>
                <span>{name(p.spell)} ({store.t('derived-level')} {p.level})</span>
                <span class="value"
                  >{p.total}{p.weak_magic ? ' ' + store.t('derived-weak-magic') : ''}</span
                >
                {#if p.within_focus != null}
                  <span class="focus">{store.t('derived-within-focus')}: {p.within_focus}</span>
                {/if}
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      <!-- Magic Resistance (per Form) -->
      <div class="detail-section">
        <h3 class="detail-label">{store.t('derived-section-magic-resistance')}</h3>
        <ul class="derived-list" data-testid="derived-magic-resistance">
          {#each d.magic_resistance as mr (mr.form)}
            <li>
              <span>{name(mr.form)}</span>
              <span class="value" title={breakdown(mr.addends)}>{mr.total}</span>
            </li>
          {/each}
        </ul>
      </div>

      <!-- Longevity -->
      {#if d.longevity}
        <div class="detail-section">
          <h3 class="detail-label">{store.t('derived-section-longevity')}</h3>
          <p data-testid="derived-longevity">
            {store.t(`derived-longevity-${d.longevity.source}`)}: −{d.longevity.bonus}
            {#if d.longevity.lab_total != null}
              ({store.t('derived-lab-total')} {d.longevity.lab_total})
            {/if}
            {#if d.longevity.bronze_cord > 0}
              · {store.t('derived-addend-bronze_cord')} {signed(d.longevity.bronze_cord)}
            {/if}
          </p>
        </div>
      {/if}
    {/if}

    <!-- Combat lines -->
    <div class="detail-section">
      <h3 class="detail-label">{store.t('derived-section-combat')}</h3>
      {#if d.combat.length > 0}
        <div class="table-scroll">
          <table class="derived-table" data-testid="derived-combat">
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
              {#each d.combat as line (line.weapon)}
                <tr>
                  <th>{name(line.weapon)}</th>
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

    <!-- Soak & Encumbrance -->
    <div class="detail-section">
      <dl class="derived-grid">
        <dt title={breakdown(d.soak.addends)}>{store.t('derived-section-soak')}</dt>
        <dd data-testid="derived-soak">{d.soak.total}</dd>
        <dt>{store.t('derived-section-encumbrance')}</dt>
        <dd data-testid="derived-encumbrance">
          {d.encumbrance.total} ({store.t('derived-load')}
          {d.encumbrance.load}, {store.t('derived-burden')}
          {d.encumbrance.burden})
        </dd>
      </dl>
    </div>

    <!-- Fatigue -->
    <div class="detail-section">
      <h3 class="detail-label">{store.t('derived-section-fatigue')}</h3>
      <ul class="derived-inline" data-testid="derived-fatigue">
        {#each d.fatigue as f (f.level)}
          <li>{store.t(`derived-fatigue-${f.level}`)}: {f.penalty}</li>
        {/each}
      </ul>
    </div>

    <!-- Wound ranges -->
    <div class="detail-section">
      <h3 class="detail-label">{store.t('derived-section-wounds')}</h3>
      <ul class="derived-list" data-testid="derived-wounds">
        {#each d.wounds as w (w.level)}
          <li>
            <span>{store.t(`derived-wound-${w.level}`)}</span>
            <span class="value">{w.min}{w.max != null ? `–${w.max}` : '+'}</span>
            {#if w.penalty != null}<span class="focus">{w.penalty}</span>{/if}
          </li>
        {/each}
      </ul>
    </div>

    <!-- Surfaced-only modifiers -->
    {#if d.surfaced_modifiers.length > 0}
      <div class="detail-section">
        <h3 class="detail-label">{store.t('derived-section-surfaced')}</h3>
        <ul class="derived-list" data-testid="derived-surfaced">
          {#each d.surfaced_modifiers as m, i (m.family + m.detail + i)}
            <li>
              <span
                >{store.t(`derived-surfaced-${m.family}`)}: {detailLabel(m.family, m.detail)}</span
              >
              {#if m.amount !== 0}<span class="value">{signed(m.amount)}</span>{/if}
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  {/if}
</section>

<style>
  .derived-grid {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.25rem 1rem;
    margin: 0;
  }
  .derived-grid dt {
    font-weight: 600;
  }
  .derived-grid dd {
    margin: 0;
  }
  .table-scroll {
    overflow-x: auto;
  }
  .derived-table {
    border-collapse: collapse;
    width: 100%;
  }
  .derived-table th,
  .derived-table td {
    padding: 0.15rem 0.5rem;
    text-align: right;
    white-space: nowrap;
  }
  .derived-table th:first-child {
    text-align: left;
  }
  .derived-table tr.focus td,
  .derived-table tr.focus th,
  .focus {
    opacity: 0.75;
    font-style: italic;
  }
  .derived-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .derived-list li {
    display: flex;
    gap: 0.75rem;
    justify-content: space-between;
  }
  .derived-list .value {
    font-weight: 600;
  }
  .derived-inline {
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin: 0;
    padding: 0;
  }
  .empty {
    opacity: 0.7;
  }
</style>
