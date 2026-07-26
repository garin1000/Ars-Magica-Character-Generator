<script lang="ts">
  // Read-only play-stat read-out (M5/5i). Renders the numbers the `derived_totals`
  // command computes; it performs NO mechanics in JS. Every label goes through a
  // Fluent `derived-*` key; catalogue ids resolve to their localized display name.
  import { store } from '../state.svelte';
  import { formatSigned, spellDisplayName } from '../derive';
  import type { Addend, PenetrationLine } from '../types';

  const d = $derived(store.derived);
  const aura = $derived(store.entity.aura ?? 0);

  // Lab/Casting Total picker. The Technique and Form option lists are derived
  // from the engine's own combination tables (never a hardcoded Art list), so
  // the picker stays in sync with whatever Arts the ruleset defines. The user's
  // pick falls back to the first available Art until they choose. The selection
  // lives on the store's `filters` so it survives this panel unmounting on a tab
  // switch; it is view state, never part of the saved entity.
  function distinct(values: string[]): string[] {
    return [...new Set(values)];
  }

  const techniques = $derived(distinct((d?.lab_totals ?? []).map((c) => c.technique)));
  const forms = $derived(distinct((d?.lab_totals ?? []).map((c) => c.form)));
  const technique = $derived(store.filters.derivedArtPicker.technique || techniques[0] || '');
  const form = $derived(store.filters.derivedArtPicker.form || forms[0] || '');

  const labCell = $derived(
    (d?.lab_totals ?? []).find((c) => c.technique === technique && c.form === form) ?? null,
  );
  const castCell = $derived(
    (d?.casting_totals ?? []).find((c) => c.technique === technique && c.form === form) ?? null,
  );

  // Localized display name for a catalogue id (Art, spell, weapon…), id as fallback.
  function name(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  // A penetration line's spell label, interpolating the chosen target Form of a
  // parametrized meta-magic Vim spell so two instances of one spell id read
  // distinctly (e.g. "Wizard's Boost (Ignem)" vs "(Aquam)"). The parens belong
  // to the name template, so the hint is the plain param label.
  function penetrationLabel(line: PenetrationLine): string {
    const rs = store.ruleset;
    if (!rs) return line.spell;
    return spellDisplayName(rs, line.spell, line.parameter, (key) => store.t(`param-label-${key}`));
  }

  // A labelled addend's display name (stable slug → Fluent).
  function addendLabel(a: Addend): string {
    return store.t(`derived-addend-${a.label}`);
  }

  function breakdown(addends: Addend[]): string {
    return addends.map((a) => `${addendLabel(a)} ${formatSigned(a.value)}`).join(', ');
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

      <!-- Lab & Casting Totals: on-demand for one chosen Technique × Form. The
           engine still computes every combination; we look up the single picked
           entry instead of rendering the exhaustive tables. -->
      <div class="detail-section">
        <h3 class="detail-label">{store.t('derived-section-lab-casting')}</h3>
        <div class="art-picker">
          <label class="field inline">
            <span>{store.t('derived-picker-technique')}</span>
            <select
              bind:value={store.filters.derivedArtPicker.technique}
              data-testid="derived-technique-select"
            >
              {#each techniques as t (t)}
                <option value={t}>{name(t)}</option>
              {/each}
            </select>
          </label>
          <label class="field inline">
            <span>{store.t('derived-picker-form')}</span>
            <select
              bind:value={store.filters.derivedArtPicker.form}
              data-testid="derived-form-select"
            >
              {#each forms as f (f)}
                <option value={f}>{name(f)}</option>
              {/each}
            </select>
          </label>
        </div>

        {#if labCell}
          <dl class="derived-grid" data-testid="derived-lab-total">
            <dt title={breakdown(labCell.addends)}>{store.t('derived-lab-total')}</dt>
            <dd>{labCell.total}{labCell.deficient ? ' ' + store.t('derived-deficient') : ''}</dd>
            {#if labCell.within_focus != null}
              <dt class="focus">{store.t('derived-within-focus')}</dt>
              <dd class="focus">{labCell.within_focus}</dd>
            {/if}
          </dl>
        {/if}

        {#if castCell}
          <div class="table-scroll">
            <table class="derived-table casting" data-testid="derived-casting-total">
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
                <tr>
                  <th title={breakdown(castCell.addends)}
                    >{store.t('derived-section-casting')}{castCell.deficient
                      ? ' ' + store.t('derived-deficient')
                      : ''}</th
                  >
                  <td>{castCell.formulaic}</td>
                  <td>{castCell.ritual}</td>
                  <td>{castCell.spontaneous_fatiguing}</td>
                  <td>{castCell.spontaneous_non_fatiguing}</td>
                </tr>
                {#if castCell.within_focus}
                  <tr class="focus">
                    <th>{store.t('derived-within-focus')}</th>
                    <td>{castCell.within_focus.formulaic}</td>
                    <td>{castCell.within_focus.ritual}</td>
                    <td>{castCell.within_focus.spontaneous_fatiguing}</td>
                    <td>{castCell.within_focus.spontaneous_non_fatiguing}</td>
                  </tr>
                {/if}
                <tr class="non-standard">
                  <th
                    >{store.t('derived-cast-non-standard')}{castCell.non_standard.deft_form
                      ? ' ' + store.t('derived-deft-form')
                      : ''}</th
                  >
                  <td>{store.t('derived-cast-silent')}: {castCell.non_standard.silent}</td>
                  <td>{store.t('derived-cast-still')}: {castCell.non_standard.still}</td>
                  <td colspan="2"
                    >{store.t('derived-cast-silent-still')}: {castCell.non_standard
                      .silent_and_still}</td
                  >
                </tr>
              </tbody>
            </table>
          </div>
        {/if}
      </div>

      <!-- Penetration (per known spell) -->
      {#if d.penetration.length > 0}
        <div class="detail-section">
          <h3 class="detail-label">{store.t('derived-section-penetration')}</h3>
          <ul class="derived-list" data-testid="derived-penetration">
            {#each d.penetration as p (`${p.spell}:${p.parameter ?? ''}`)}
              <li>
                <span>{penetrationLabel(p)} ({store.t('derived-level')} {p.level})</span>
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
            {store.t(`derived-longevity-${d.longevity.source}`)}: -{d.longevity.bonus}
            {#if d.longevity.hint}
              ({store.t('derived-lab-total')} {d.longevity.hint.lab_total})
            {/if}
            {#if d.longevity.bronze_cord > 0}
              · {store.t('derived-addend-bronze_cord')} {formatSigned(d.longevity.bronze_cord)}
            {/if}
          </p>
        </div>
      {/if}

      <!-- Masterpiece (lesser enchanted item cap) -->
      {#if d.masterpiece}
        <div class="detail-section">
          <h3 class="detail-label">{store.t('derived-section-masterpiece')}</h3>
          <p data-testid="derived-masterpiece">
            {store.t('derived-masterpiece-cap')}: {d.masterpiece.cap}
            ({store.t('derived-lab-total')}
            {d.masterpiece.lab_total} ·
            {name(d.masterpiece.technique)} / {name(d.masterpiece.form)})
          </p>
          <p class="hint">{store.t('derived-masterpiece-note')}</p>
        </div>
      {/if}
    {/if}

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
      <ul class="derived-list wound-list" data-testid="derived-wounds">
        {#each d.wounds as w (w.level)}
          <li>
            <span class="wound-level">{store.t(`derived-wound-${w.level}`)}</span>
            <span class="value">{w.min}{w.max != null ? `–${w.max}` : '+'}</span>
            <span class="focus">{w.penalty != null ? w.penalty : ''}</span>
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
              {#if m.amount !== 0}<span class="value">{formatSigned(m.amount)}</span>{/if}
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
    /* Width tokens (defined once on :root in app.css) are the single source of
       truth for how wide every derived read-out is allowed to grow. Narrow cap
       for the two-column lists/grids; wider cap for the multi-column tables. */
    max-width: var(--readout-max-width);
  }
  .derived-grid dt {
    font-weight: 600;
  }
  .derived-grid dd {
    margin: 0;
  }
  /* Technique/Form picker row for the on-demand Lab & Casting Totals. */
  .art-picker {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }
  .table-scroll {
    overflow-x: auto;
  }
  .derived-table {
    border-collapse: collapse;
    width: 100%;
    max-width: var(--readout-table-max-width);
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
    /* Shared narrow read-out width (single source of truth on :root). */
    max-width: var(--readout-max-width);
  }
  .derived-list li {
    display: flex;
    gap: 0.75rem;
    justify-content: space-between;
  }
  .derived-list .value {
    font-weight: 600;
  }
  /* Wound levels: a fixed three-column grid (level | number range | penalty).
     Width comes from the shared `.derived-list` token; the grid keeps the range
     and penalty aligned in centered columns instead of splaying apart. */
  .wound-list li {
    display: grid;
    grid-template-columns: 1fr 6rem 4rem;
    align-items: baseline;
    gap: 0.75rem;
    justify-content: initial;
  }
  .wound-list .value,
  .wound-list .focus {
    text-align: center;
    font-variant-numeric: tabular-nums;
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
  .hint {
    opacity: 0.7;
    font-size: 0.85em;
    font-style: italic;
  }
</style>
