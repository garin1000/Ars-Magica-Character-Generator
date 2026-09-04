<script lang="ts">
  import { store } from '../state.svelte';
  import { eligibleForConstraint, grantItemLabel, sameSelection } from '../derive';
  import ParameterPicker from './ParameterPicker.svelte';
  import { tooltip, type TooltipContent } from '../actions';
  import type { GrantConstraint, House, PointItem, Selection } from '../types';

  // Houses are data from the ruleset. Display names/descriptions come from the
  // rules i18n map (keyed by house id, like Arts), never a Fluent chrome key or
  // the raw slug. Sorted by localized name so the list reads alphabetically in
  // the active language.
  const houses = $derived.by((): House[] => {
    const rs = store.ruleset;
    if (!rs) return [];
    return Object.values(rs.ruleset.houses ?? {}).sort((a, b) =>
      houseName(a.id).localeCompare(houseName(b.id)),
    );
  });

  const selected = $derived(
    store.entity.house ? store.ruleset?.ruleset.houses?.[store.entity.house] : undefined,
  );

  function houseName(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  function houseTip(id: string): TooltipContent {
    return { text: store.ruleset?.i18n[id]?.description ?? undefined };
  }

  // Localized name of a granted/option item, filling any `{param}` token: an
  // unfilled slot shows a localized hint ("(Ability)"), a ref value resolves to
  // its own localized name ("Puissant Ignem" rather than "Puissant art.ignem").
  function label(ref: string, params: Record<string, string> = {}): string {
    const rs = store.ruleset;
    if (!rs) return ref;
    return grantItemLabel(rs, ref, store.t, params);
  }

  // Point items an open grant admits — the shared constraint filter, mirroring
  // the engine's `validate_house` check so the picker offers only legal choices.
  // The character's House goes in too: a Virtue that confers a different House
  // (Heartbeast makes you a Bjornaer) must never appear on another House's menu.
  function eligibleForOpen(c: GrantConstraint): PointItem[] {
    const rs = store.ruleset;
    return rs ? eligibleForConstraint(rs, c, store.entity.house ?? null) : [];
  }

  // Index of the currently-picked option for a choice grant (−1 if none), so the
  // <select> reflects the stored pick.
  function pickedIndex(choiceKey: string, options: Selection[]): number {
    const pick = store.entity.house_choices?.[choiceKey];
    return pick ? options.findIndex((o) => sameSelection(o, pick)) : -1;
  }

  // The whole Selection picked for an open grant (undefined while unchosen), so a
  // parameterized pick's params are in reach for the ParameterPicker.
  function openPick(choiceKey: string): Selection | undefined {
    return store.entity.house_choices?.[choiceKey];
  }

  function onHouse(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    void store.setHouse(value === '' ? null : value);
  }

  function onChoice(choiceKey: string, options: Selection[], event: Event) {
    const raw = (event.currentTarget as HTMLSelectElement).value;
    if (raw === '') return;
    const option = options[Number(raw)];
    if (option) store.setHouseChoice(choiceKey, option);
  }

  function onOpen(choiceKey: string, event: Event) {
    const ref = (event.currentTarget as HTMLSelectElement).value;
    if (ref) store.setHouseChoice(choiceKey, { ref });
  }
</script>

<section class="house-selector" data-testid="house-selector">
  {#if store.ruleset}
    <div class="region-row">
      <section class="region region-source">
        <h2 class="region-title">{store.t('house-label')}</h2>
        <div class="panel">
          <label class="field">
            <span>{store.t('house-label')}</span>
            <select value={store.entity.house ?? ''} onchange={onHouse} data-testid="house-select">
              <option value="">{store.t('house-none')}</option>
              {#each houses as house (house.id)}
                <option value={house.id}>{houseName(house.id)}</option>
              {/each}
            </select>
          </label>
        </div>
      </section>

      <section class="region region-selected">
        <h2 class="region-title">{store.t('house-grants-title')}</h2>
        <div class="selected-frame">
          <!-- The frame carries the border and its padding; this inner box does the
               scrolling, so the padding stays a gap the content cannot scroll into. -->
          <div class="selected-scroll">
            {#if selected}
              <p class="house-description" use:tooltip={houseTip(selected.id)}>
                {store.ruleset.i18n[selected.id]?.description ?? ''}
              </p>

              <ul class="house-grants">
                {#each selected.grants ?? [] as grant, g (g)}
                  <li class="house-grant">
                    {#if grant.kind === 'fixed'}
                      <span class="house-granted-label">{store.t('house-granted-label')}</span>
                      <span class="item-name" data-testid="house-granted-{grant.item}">
                        {label(grant.item, grant.params)}
                      </span>
                    {:else if grant.kind === 'choice'}
                      <span class="house-granted-label" id="house-choice-label-{grant.choice_key}"
                        >{store.t('house-granted-label')}</span
                      >
                      <select
                        value={String(pickedIndex(grant.choice_key, grant.options))}
                        onchange={(e) => onChoice(grant.choice_key, grant.options, e)}
                        aria-labelledby="house-choice-label-{grant.choice_key}"
                        data-testid="house-choice-{grant.choice_key}"
                      >
                        <option value="-1">{store.t('house-choose-prompt')}</option>
                        {#each grant.options as option, i (i)}
                          <option value={String(i)}>{label(option.ref, option.params)}</option>
                        {/each}
                      </select>
                    {:else}
                      {@const pick = openPick(grant.choice_key)}
                      <span class="house-granted-label" id="house-open-label-{grant.choice_key}"
                        >{store.t('house-granted-label')}</span
                      >
                      <select
                        value={pick?.ref ?? ''}
                        onchange={(e) => onOpen(grant.choice_key, e)}
                        aria-labelledby="house-open-label-{grant.choice_key}"
                        data-testid="house-open-{grant.choice_key}"
                      >
                        <option value="">{store.t('house-choose-prompt')}</option>
                        {#each eligibleForOpen(grant.constraint) as item (item.id)}
                          <option value={item.id}>{label(item.id)}</option>
                        {/each}
                      </select>
                      <!-- A parameterized pick ("Puissant (Art)") needs its target
                           chosen too, or the engine reports the parameter missing. -->
                      {#if pick}
                        {@const parameters =
                          store.ruleset.ruleset.point_items[pick.ref]?.parameters ?? []}
                        {#if parameters.length > 0}
                          <ParameterPicker
                            selection={pick}
                            params={parameters}
                            idSuffix={grant.choice_key}
                            commit={(next) => store.setHouseChoice(grant.choice_key, next)}
                          />
                        {/if}
                      {/if}
                    {/if}
                  </li>
                {/each}
              </ul>
            {:else}
              <p class="empty">{store.t('house-none-selected')}</p>
            {/if}
          </div>
        </div>
      </section>
    </div>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>

<style>
  .house-description {
    margin: 0 0 0.75rem;
    line-height: 1.4;
  }

  .house-grants {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .house-grant {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .house-granted-label {
    font-size: var(--font-chrome);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
  }

  .empty {
    color: var(--muted);
  }
</style>
