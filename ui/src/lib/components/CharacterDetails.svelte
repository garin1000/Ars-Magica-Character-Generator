<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned } from '../derive';
  import { CHARACTERISTICS, type Characteristic } from '../types';

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
  const traits = $derived(store.entity.personality_traits ?? []);
  const reputations = $derived(store.entity.reputations ?? []);
  // Reputation input is offered only for the kinds a V/F grants (Core:2514).
  const grants = $derived(store.effective?.reputation_grants ?? []);
  const agingPoints = $derived(store.entity.aging_points ?? {});
  const twilightScars = $derived(store.entity.twilight_scars ?? []);
  const warpingEffect = $derived(store.entity.warping_effect ?? '');
  const decrepitudeEffect = $derived(store.entity.decrepitude_effect ?? '');
  const agingLog = $derived(store.entity.aging_log ?? []);

  function onAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAge(raw === '' ? null : Number(raw));
  }

  function onApparentAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setApparentAge(raw === '' ? null : Number(raw));
  }

  function onBirthYear(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setBirthYear(raw === '' ? null : Number(raw));
  }

  function repKindLabel(kind: string): string {
    return store.t(`reputation-type-${kind}`);
  }

  function charLabel(characteristic: Characteristic): string {
    return store.t(`characteristic-${characteristic}`);
  }

  function numValue(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value || 0);
  }
</script>

<section class="panel character-details">
  {#if store.ruleset}
    <div class="detail-section">
      <h3 class="detail-label">{store.t('identity-label')}</h3>
      <label class="field">
        <span>{store.t('identity-concept')}</span>
        <textarea
          class="concept-input"
          rows="3"
          value={store.entity.concept ?? ''}
          oninput={(e) =>
            store.setIdentity('concept', (e.currentTarget as HTMLTextAreaElement).value)}
          placeholder={store.t('identity-concept-placeholder')}
          data-testid="identity-concept"
        ></textarea>
      </label>
      <label class="field">
        <span>{store.t('identity-gender')}</span>
        <input
          value={store.entity.gender ?? ''}
          oninput={(e) => store.setIdentity('gender', (e.currentTarget as HTMLInputElement).value)}
          data-testid="identity-gender"
        />
      </label>
      <label class="field">
        <span>{store.t('identity-birth-year')}</span>
        <input
          type="number"
          value={store.entity.birth_year ?? ''}
          oninput={onBirthYear}
          data-testid="identity-birth-year"
        />
      </label>
      <label class="field">
        <span>{store.t('identity-sigil')}</span>
        <input
          value={store.entity.sigil ?? ''}
          oninput={(e) => store.setIdentity('sigil', (e.currentTarget as HTMLInputElement).value)}
          data-testid="identity-sigil"
        />
      </label>
      <label class="field">
        <span>{store.t('identity-covenant')}</span>
        <input
          value={store.entity.covenant_name ?? ''}
          oninput={(e) =>
            store.setIdentity('covenant_name', (e.currentTarget as HTMLInputElement).value)}
          data-testid="identity-covenant"
        />
      </label>
      <label class="field">
        <span>{store.t('identity-parens')}</span>
        <input
          value={store.entity.parens ?? ''}
          oninput={(e) => store.setIdentity('parens', (e.currentTarget as HTMLInputElement).value)}
          data-testid="identity-parens"
        />
      </label>
    </div>

    <div class="detail-field">
      <label class="field">
        <span>{store.t('age-label')}</span>
        <input type="number" min="1" value={age ?? ''} oninput={onAge} data-testid="age-input" />
      </label>
      <label class="field">
        <span>{store.t('apparent-age-label')}</span>
        <input
          type="number"
          min="1"
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
          value={storedWarpingPoints}
          oninput={(e) => store.setWarpingPoints(numValue(e))}
          data-testid="warping-points-input"
        />
      </label>
    </div>

    <div class="detail-field">
      <label class="field">
        <span>{store.t('warping-effect-label')}</span>
        <input
          type="text"
          value={warpingEffect}
          oninput={(e) => store.setWarpingEffect((e.currentTarget as HTMLInputElement).value)}
          data-testid="warping-effect-input"
        />
      </label>
    </div>

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
              aria-label={store.t('aging-log-year-label')}
              value={entry.year}
              oninput={(e) => store.setAgingLogEntryYear(i, numValue(e))}
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

    <div class="detail-section">
      <h3 class="detail-label">{store.t('personality-label')}</h3>
      <ul class="trait-list" data-testid="personality-list">
        {#each traits as trait, i (i)}
          <li>
            <input
              class="trait-name"
              placeholder={store.t('personality-name-placeholder')}
              value={trait.name}
              oninput={(e) =>
                store.setPersonalityTraitName(i, (e.currentTarget as HTMLInputElement).value)}
              data-testid="personality-name-{i}"
            />
            <span class="spinner">
              <button
                type="button"
                class="icon-btn"
                aria-label={store.t('characteristic-decrement')}
                onclick={() => store.setPersonalityTraitValue(i, trait.value - 1)}
                data-testid="personality-dec-{i}"
              >
                -
              </button>
              <span class="spinner-value" data-testid="personality-value-{i}">
                {formatSigned(trait.value)}
              </span>
              <button
                type="button"
                class="icon-btn"
                aria-label={store.t('characteristic-increment')}
                onclick={() => store.setPersonalityTraitValue(i, trait.value + 1)}
                data-testid="personality-inc-{i}"
              >
                +
              </button>
            </span>
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('spell-remove')}
              onclick={() => store.removePersonalityTraitAt(i)}
              data-testid="personality-remove-{i}"
            >
              ×
            </button>
          </li>
        {:else}
          <li class="empty">{store.t('personality-empty')}</li>
        {/each}
      </ul>
      <button
        type="button"
        onclick={() => store.addPersonalityTrait()}
        data-testid="personality-add"
      >
        {store.t('personality-add')}
      </button>
    </div>

    <div class="detail-section">
      <h3 class="detail-label">{store.t('reputations-label')}</h3>
      <ul class="reputation-list" data-testid="reputation-list">
        {#each reputations as reputation, i (i)}
          <li>
            <span class="reputation-tag">
              {repKindLabel(reputation.kind)}
              {reputation.score}
            </span>
            <input
              class="reputation-content"
              placeholder={store.t('reputation-content-placeholder')}
              value={reputation.content}
              oninput={(e) =>
                store.setReputationContent(i, (e.currentTarget as HTMLInputElement).value)}
              data-testid="reputation-content-{i}"
            />
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('spell-remove')}
              onclick={() => store.removeReputationAt(i)}
              data-testid="reputation-remove-{i}"
            >
              ×
            </button>
          </li>
        {/each}
      </ul>
      {#if grants.length === 0}
        <p class="empty" data-testid="reputation-empty">{store.t('reputation-empty')}</p>
      {:else}
        {#each grants as grant, gi (gi)}
          <button
            type="button"
            onclick={() => store.addReputation(grant.kind, grant.score)}
            data-testid="reputation-add-{grant.kind}"
          >
            {store.t('reputation-add', {
              kind: repKindLabel(grant.kind),
              score: String(grant.score),
            })}
          </button>
        {/each}
      {/if}
    </div>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
