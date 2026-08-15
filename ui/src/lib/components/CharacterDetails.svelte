<script lang="ts">
  import { store } from '../state.svelte';
  import { eligibleForConstraint, grantItemLabel, groupWarpingOwedGrants } from '../derive';
  import ParameterPicker from './ParameterPicker.svelte';
  import IdentityFields from './IdentityFields.svelte';
  import PersonalityTraits from './PersonalityTraits.svelte';
  import Reputations from './Reputations.svelte';
  import {
    CHARACTERISTICS,
    type Characteristic,
    type GrantConstraint,
    type PointItem,
    type Selection,
  } from '../types';

  const age = $derived(store.entity.age ?? null);
  const apparentAge = $derived(store.entity.apparent_age ?? null);
  const ageCap = $derived(store.effective?.age_ability_cap ?? null);
  // Confidence is derived (type default + V/F); grogs have none (0/0) → hidden.
  const confScore = $derived(store.effective?.confidence_score ?? 0);
  const confPoints = $derived(store.effective?.confidence_points ?? 0);
  const showConfidence = $derived(confScore > 0 || confPoints > 0);
  // Warping is derived by the engine: stored Warping Points + any granted by V/F,
  // with the score inverted from the advancement curve. Hidden when 0/0.
  const warpScore = $derived(store.effective?.warping_score ?? 0);
  const warpPoints = $derived(store.effective?.warping_points ?? 0);
  const showWarping = $derived(warpScore > 0 || warpPoints > 0);
  const storedWarpingPoints = $derived(store.entity.warping_points ?? 0);
  // Decrepitude is derived by the engine from the sum of aging points; hidden at 0.
  const decrepitude = $derived(store.effective?.decrepitude_score ?? 0);
  // True Faith is derived from V/F (True Faith → 1); hidden when 0.
  const trueFaith = $derived(store.effective?.true_faith_score ?? 0);
  // Starting enchanted-device level budget (Magic Items/Redcap); hidden when 0.
  const itemLevels = $derived(store.effective?.item_level_budget ?? 0);
  const agingPoints = $derived(store.entity.aging_points ?? {});
  const twilightScars = $derived(store.entity.twilight_scars ?? []);
  const warpingEffect = $derived(store.entity.warping_effect ?? '');
  const decrepitudeEffect = $derived(store.entity.decrepitude_effect ?? '');
  const agingLog = $derived(store.entity.aging_log ?? []);

  // Off-budget Virtues/Flaws owed from the Warping Score (Core:16547-16561). The
  // engine surfaces the per-kind counts and one OPEN grant (choice_key +
  // constraint) per owed slot; it returns an empty list for magi (exempt —
  // Twilight instead), so the section simply never renders for them.
  const warpingOwedGrants = $derived(store.effective?.warping_owed_grants ?? []);
  // The owed slots bucketed by what each expects, so every <select> can say what
  // it wants instead of standing in an unlabelled row.
  const warpingSlotGroups = $derived(groupWarpingOwedGrants(warpingOwedGrants));

  // Localized name of an owed-fill candidate, with any `{param}` token filled: an
  // unchosen parameter shows its localized hint ("(Form)"), a chosen ref resolves
  // to its own name — never a raw brace or slug.
  function warpingItemName(ref: string, params: Record<string, string> = {}): string {
    const rs = store.ruleset;
    if (!rs) return ref;
    return grantItemLabel(rs, ref, store.t, params);
  }

  // Point items an owed slot admits: the constraint's kind/magnitude/category,
  // AND never an item that itself grants Warping (the recursion guard — mirrors
  // the engine's ineligibility rule).
  function eligibleForWarping(c: GrantConstraint): PointItem[] {
    const rs = store.ruleset;
    return rs ? eligibleForConstraint(rs, c, { excludeWarpingSources: true }) : [];
  }

  // The Selection filling one owed slot (undefined while unchosen). The whole
  // Selection, not just its ref, so a parameterized fill's params are in reach.
  function warpingPick(choiceKey: string): Selection | undefined {
    return store.entity.warping_choices?.[choiceKey];
  }

  function onWarpingChoice(choiceKey: string, event: Event) {
    const ref = (event.currentTarget as HTMLSelectElement).value;
    store.setWarpingChoice(choiceKey, ref ? { ref } : null);
  }

  function onAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAge(raw === '' ? null : Number(raw));
  }

  function onApparentAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setApparentAge(raw === '' ? null : Number(raw));
  }

  function charLabel(characteristic: Characteristic): string {
    return store.t(`characteristic-${characteristic}`);
  }

  function numValue(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value || 0);
  }

  /** An emptied field reads as "not given", not as 0 — same as {@link onAge}. */
  function optionalNumValue(event: Event): number | null {
    const raw = (event.currentTarget as HTMLInputElement).value;
    return raw === '' ? null : Number(raw);
  }
</script>

<section class="panel character-details">
  {#if store.ruleset}
    <IdentityFields />

    <div class="detail-field">
      <label class="field">
        <span>{store.t('age-label')}</span>
        <input
          type="number"
          min="1"
          max="4294967295"
          value={age ?? ''}
          oninput={onAge}
          data-testid="age-input"
        />
      </label>
      <label class="field">
        <span>{store.t('apparent-age-label')}</span>
        <input
          type="number"
          min="1"
          max="4294967295"
          value={apparentAge ?? ''}
          oninput={onApparentAge}
          data-testid="apparent-age-input"
        />
      </label>
      {#if ageCap != null}
        <span class="age-cap" data-testid="age-cap-note">
          {store.t('age-cap-note', { cap: String(ageCap) })}
        </span>
      {/if}
    </div>

    {#if showConfidence}
      <div class="detail-field">
        <span class="detail-label">{store.t('confidence-label')}</span>
        <span data-testid="confidence-readout">
          {store.t('confidence-readout', { score: String(confScore), points: String(confPoints) })}
        </span>
      </div>
    {/if}

    <!-- Warping & Twilight cluster: kept consecutive in source order so the
         .character-details multi-column flow lands them together. Source order
         governs column flow; CSS `order` has no effect on multi-column children,
         so adjacency is achieved by ordering the DOM, not by styling. -->
    {#if showWarping}
      <div class="detail-field">
        <span class="detail-label">{store.t('warping-label')}</span>
        <span data-testid="warping-readout">
          {store.t('warping-readout', { score: String(warpScore), points: String(warpPoints) })}
        </span>
      </div>
    {/if}

    <div class="detail-field">
      <label class="field">
        <span>{store.t('warping-points-label')}</span>
        <input
          type="number"
          min="0"
          max="4294967295"
          value={storedWarpingPoints}
          oninput={(e) => store.setWarpingPoints(numValue(e))}
          data-testid="warping-points-input"
        />
      </label>
    </div>

    <div class="detail-field">
      <label class="field">
        <span>{store.t('warping-effect-label')}</span>
        <textarea
          class="warping-effect"
          rows="3"
          value={warpingEffect}
          oninput={(e) => store.setWarpingEffect((e.currentTarget as HTMLTextAreaElement).value)}
          data-testid="warping-effect-input"
        ></textarea>
      </label>
    </div>

    <!-- Owed Warping Virtues & Flaws (Core:16547-16561). Non-magi only: the engine
         returns no owed grants for magi (Twilight instead), so this never renders
         for them. The slots are grouped by what they expect (Minor Flaw,
         supernatural Minor Virtue, Major Flaw) and each one labelled, so a row of
         otherwise identical <select>s is readable; the rules name no eligible
         list, so each picker offers everything the constraint admits. -->
    {#if warpingOwedGrants.length > 0}
      <div class="detail-section" data-testid="warping-owed">
        <h3 class="detail-label">{store.t('warping-owed-label')}</h3>
        <p class="warping-owed-hint">{store.t('warping-owed-hint')}</p>
        {#each warpingSlotGroups as group (group.labelKey)}
          <div class="warping-owed-group" data-testid="warping-owed-group-{group.labelKey}">
            <h4 class="detail-label">
              {store.t(group.countKey, { count: group.grants.length })}
            </h4>
            <ul class="warping-owed-pickers">
              {#each group.grants as slot (slot.choice_key)}
                {@const pick = warpingPick(slot.choice_key)}
                <li>
                  <label class="field">
                    <span>{store.t(group.labelKey)}</span>
                    <select
                      value={pick?.ref ?? ''}
                      onchange={(e) => onWarpingChoice(slot.choice_key, e)}
                      data-testid="warping-fill-{slot.choice_key}"
                    >
                      <option value="">{store.t('warping-choose-prompt')}</option>
                      {#each eligibleForWarping(slot.constraint) as item (item.id)}
                        <option value={item.id}>{warpingItemName(item.id)}</option>
                      {/each}
                    </select>
                  </label>
                  <!-- A parameterized fill ("Master of (Form) Creatures") needs its
                       target chosen too, or the engine reports the parameter
                       missing. The pick carries its own params, keyed by slot. -->
                  {#if pick}
                    {@const parameters =
                      store.ruleset?.ruleset.point_items[pick.ref]?.parameters ?? []}
                    {#if parameters.length > 0}
                      <ParameterPicker
                        selection={pick}
                        params={parameters}
                        idSuffix={slot.choice_key}
                        commit={(next) => store.setWarpingChoice(slot.choice_key, next)}
                      />
                    {/if}
                  {/if}
                </li>
              {/each}
            </ul>
          </div>
        {/each}
      </div>
    {/if}

    <div class="detail-section">
      <h3 class="detail-label">{store.t('twilight-scars-label')}</h3>
      <ul class="twilight-list" data-testid="twilight-scars-list">
        {#each twilightScars as scar, i (i)}
          <li>
            <input
              class="twilight-desc"
              placeholder={store.t('twilight-scar-placeholder')}
              value={scar.description}
              oninput={(e) =>
                store.setTwilightScarDescription(i, (e.currentTarget as HTMLInputElement).value)}
              data-testid="twilight-scar-{i}"
            />
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('spell-remove')}
              onclick={() => store.removeTwilightScarAt(i)}
              data-testid="twilight-scar-remove-{i}"
            >
              ×
            </button>
          </li>
        {:else}
          <li class="empty">{store.t('twilight-scars-empty')}</li>
        {/each}
      </ul>
      <button type="button" onclick={() => store.addTwilightScar()} data-testid="twilight-scar-add">
        {store.t('twilight-scar-add')}
      </button>
    </div>

    {#if trueFaith > 0}
      <div class="detail-field">
        <span class="detail-label">{store.t('true-faith-label')}</span>
        <span data-testid="true-faith-readout">
          {store.t('true-faith-readout', { score: String(trueFaith) })}
        </span>
      </div>
    {/if}

    {#if itemLevels > 0}
      <div class="detail-field">
        <span class="detail-label">{store.t('item-levels-label')}</span>
        <span data-testid="item-levels-readout">
          {store.t('item-levels-readout', { levels: String(itemLevels) })}
        </span>
      </div>
    {/if}

    {#if decrepitude > 0}
      <div class="detail-field">
        <span class="detail-label">{store.t('decrepitude-label')}</span>
        <span data-testid="decrepitude-readout">
          {store.t('decrepitude-readout', { score: String(decrepitude) })}
        </span>
      </div>
    {/if}

    <div class="detail-field">
      <label class="field">
        <span>{store.t('decrepitude-effect-label')}</span>
        <input
          type="text"
          value={decrepitudeEffect}
          oninput={(e) => store.setDecrepitudeEffect((e.currentTarget as HTMLInputElement).value)}
          data-testid="decrepitude-effect-input"
        />
      </label>
    </div>

    <div class="detail-section">
      <h3 class="detail-label">{store.t('aging-label')}</h3>
      <p class="detail-label">{store.t('aging-points-heading')}</p>
      <ul class="aging-list" data-testid="aging-points-list">
        {#each CHARACTERISTICS as characteristic (characteristic)}
          <li>
            <span class="char-name">{charLabel(characteristic)}</span>
            <input
              type="number"
              min="0"
              max="255"
              value={agingPoints[characteristic] ?? 0}
              oninput={(e) => store.setAgingPoints(characteristic, numValue(e))}
              data-testid="aging-points-{characteristic}"
            />
          </li>
        {/each}
      </ul>
      <p class="detail-label" data-testid="aging-points-note">
        {store.t('aging-points-note')}
      </p>

      <p class="detail-label">{store.t('aging-log-heading')}</p>
      <ul class="twilight-list" data-testid="aging-log-list">
        {#each agingLog as entry, i (i)}
          <li>
            <input
              type="number"
              class="aging-log-year"
              min="-2147483648"
              max="2147483647"
              aria-label={store.t('aging-log-year-label')}
              value={entry.year ?? ''}
              oninput={(e) => store.setAgingLogEntryYear(i, optionalNumValue(e))}
              data-testid="aging-log-year-{i}"
            />
            <input
              class="twilight-desc"
              placeholder={store.t('aging-log-effect-placeholder')}
              value={entry.effect}
              oninput={(e) =>
                store.setAgingLogEntryEffect(i, (e.currentTarget as HTMLInputElement).value)}
              data-testid="aging-log-effect-{i}"
            />
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('spell-remove')}
              onclick={() => store.removeAgingLogEntryAt(i)}
              data-testid="aging-log-remove-{i}"
            >
              ×
            </button>
          </li>
        {:else}
          <li class="empty">{store.t('aging-log-empty')}</li>
        {/each}
      </ul>
      <button type="button" onclick={() => store.addAgingLogEntry()} data-testid="aging-log-add">
        {store.t('aging-log-add')}
      </button>
    </div>

    <PersonalityTraits />
    <Reputations />
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
