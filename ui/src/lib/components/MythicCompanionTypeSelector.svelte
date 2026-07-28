<script lang="ts">
  import { store } from '../state.svelte';
  import { eligibleForConstraint as eligibleItems, grantItemLabel, sameSelection } from '../derive';
  import ParameterPicker from './ParameterPicker.svelte';
  import { tooltip, type TooltipContent } from '../actions';
  import type {
    GrantConstraint,
    MythicCompanionType,
    PointItem,
    RequiredFlaw,
    Selection,
  } from '../types';

  // Mythic Companion types are data from the ruleset; names/descriptions come
  // from the rules i18n map (keyed by type id, like Houses), never Fluent chrome
  // or the raw slug. Sorted by localized name.
  const types = $derived.by((): MythicCompanionType[] => {
    const rs = store.ruleset;
    if (!rs) return [];
    return Object.values(rs.ruleset.mythic_companion_types ?? {}).sort((a, b) =>
      typeName(a.id).localeCompare(typeName(b.id)),
    );
  });

  const selected = $derived(
    store.entity.mythic_type
      ? store.ruleset?.ruleset.mythic_companion_types?.[store.entity.mythic_type]
      : undefined,
  );

  function typeName(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  function typeTip(id: string): TooltipContent {
    return { text: store.ruleset?.i18n[id]?.description ?? undefined };
  }

  // Localized name of a granted/option/required item, filling any `{param}` token
  // (same helper HouseSelector uses).
  function label(ref: string, params: Record<string, string> = {}): string {
    const rs = store.ruleset;
    if (!rs) return ref;
    return grantItemLabel(rs, ref, store.t, params);
  }

  // Items a constraint admits — the shared filter, mirroring the engine's
  // `open_pick_satisfies`. Serves both the required-Flaw substitute menu and an
  // `open` free-Virtue grant.
  function eligibleForConstraint(c: GrantConstraint): PointItem[] {
    const rs = store.ruleset;
    return rs ? eligibleItems(rs, c) : [];
  }

  // Index of the currently-picked option for a `choice` grant (−1 if none).
  function pickedIndex(choiceKey: string, options: Selection[]): number {
    const pick = store.entity.mythic_choices?.[choiceKey];
    return pick ? options.findIndex((o) => sameSelection(o, pick)) : -1;
  }

  // The whole Selection picked for an `open` grant (undefined while unchosen), so
  // a parameterized pick's params are in reach for the ParameterPicker.
  function openPick(choiceKey: string): Selection | undefined {
    return store.entity.mythic_choices?.[choiceKey];
  }

  // The required-Flaw ref currently satisfying a slot: the eligible flaw present
  // in the bought selections, else the rules default (pre-filled).
  function currentRequiredFlaw(flaw: RequiredFlaw): string {
    const eligible = new Set(eligibleForConstraint(flaw.constraint).map((it) => it.id));
    const chosen = store.entity.selections.find((s) => eligible.has(s.ref));
    return chosen?.ref ?? flaw.default.ref;
  }

  function onType(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    void store.setMythicType(value === '' ? null : value);
  }

  function onChoice(choiceKey: string, options: Selection[], event: Event) {
    const raw = (event.currentTarget as HTMLSelectElement).value;
    if (raw === '') return;
    const option = options[Number(raw)];
    if (option) store.setMythicChoice(choiceKey, option);
  }

  function onOpen(choiceKey: string, event: Event) {
    const ref = (event.currentTarget as HTMLSelectElement).value;
    // A bare `{ref}` resets any params the previous pick carried.
    if (ref) store.setMythicChoice(choiceKey, { ref });
  }

  function onFlawSwap(previousRef: string, event: Event) {
    const nextRef = (event.currentTarget as HTMLSelectElement).value;
    if (nextRef) void store.setMythicRequiredFlaw(previousRef, nextRef);
  }
</script>

<section class="panel mythic-type-selector" data-testid="mythic-type-selector">
  {#if store.ruleset}
    <label class="field">
      <span>{store.t('mythic-type-label')}</span>
      <select
        value={store.entity.mythic_type ?? ''}
        onchange={onType}
        data-testid="mythic-type-select"
      >
        <option value="">{store.t('mythic-type-none')}</option>
        {#each types as type (type.id)}
          <option value={type.id}>{typeName(type.id)}</option>
        {/each}
      </select>
    </label>

    {#if selected}
      <p class="mythic-type-description" use:tooltip={typeTip(selected.id)}>
        {store.ruleset.i18n[selected.id]?.description ?? ''}
      </p>

      <ul class="mythic-grants">
        {#each selected.grants ?? [] as grant, g (g)}
          <li class="mythic-grant">
            {#if grant.kind === 'fixed'}
              <span class="mythic-granted-label">{store.t('mythic-granted-label')}</span>
              <span class="item-name" data-testid="mythic-granted-{grant.item}">
                {label(grant.item, grant.params)}
              </span>
            {:else if grant.kind === 'choice'}
              <span class="mythic-granted-label">{store.t('mythic-granted-label')}</span>
              <select
                value={String(pickedIndex(grant.choice_key, grant.options))}
                onchange={(e) => onChoice(grant.choice_key, grant.options, e)}
                data-testid="mythic-choice-{grant.choice_key}"
              >
                <option value="-1">{store.t('mythic-choose-prompt')}</option>
                {#each grant.options as option, i (i)}
                  <option value={String(i)}>{label(option.ref, option.params)}</option>
                {/each}
              </select>
            {:else}
              <!-- An `open` free-Virtue grant: the player picks anything the
                   constraint admits (the engine validates the same way it does for
                   a House open grant). No shipped Mythic type defines one today,
                   but the engine's Grant set does, so the UI covers it. -->
              {@const pick = openPick(grant.choice_key)}
              <span class="mythic-granted-label">{store.t('mythic-granted-label')}</span>
              <select
                value={pick?.ref ?? ''}
                onchange={(e) => onOpen(grant.choice_key, e)}
                data-testid="mythic-open-{grant.choice_key}"
              >
                <option value="">{store.t('mythic-choose-prompt')}</option>
                {#each eligibleForConstraint(grant.constraint) as item (item.id)}
                  <option value={item.id}>{label(item.id)}</option>
                {/each}
              </select>
              {#if pick}
                {@const parameters = store.ruleset.ruleset.point_items[pick.ref]?.parameters ?? []}
                {#if parameters.length > 0}
                  <ParameterPicker
                    selection={pick}
                    params={parameters}
                    idSuffix={grant.choice_key}
                    commit={(next) => store.setMythicChoice(grant.choice_key, next)}
                  />
                {/if}
              {/if}
            {/if}
          </li>
        {/each}
      </ul>

      {#if (selected.required_flaws ?? []).length > 0}
        <ul class="mythic-required-flaws">
          {#each selected.required_flaws ?? [] as flaw, f (f)}
            {@const current = currentRequiredFlaw(flaw)}
            <li class="mythic-required-flaw">
              <span class="mythic-required-flaw-label">{store.t('mythic-required-flaw-label')}</span
              >
              <select
                value={current}
                onchange={(e) => onFlawSwap(current, e)}
                data-testid="mythic-required-flaw-{flaw.default.ref}"
              >
                {#each eligibleForConstraint(flaw.constraint) as item (item.id)}
                  <option value={item.id}>{label(item.id)}</option>
                {/each}
              </select>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
