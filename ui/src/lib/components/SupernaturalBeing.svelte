<script lang="ts">
  import { store } from '../state.svelte';
  import { REALMS, type Realm } from '../types';
  import LevelRemoveField from './LevelRemoveField.svelte';

  const might = $derived(store.entity.might ?? null);
  const powers = $derived(store.entity.powers ?? []);
  // Effective Might, power-levels budget and the Might-based MR are all
  // engine-authoritative — never recomputed here.
  const effectiveMight = $derived(store.effective?.might ?? null);
  const powerBudget = $derived(store.effective?.power_levels_budget ?? 0);
  const powerUsed = $derived(store.effective?.power_levels_used ?? 0);
  // A Might-being's Magic Resistance is a flat blanket = Might Score (same on
  // every Form); read the first line's total as the representative value.
  const mightMr = $derived(store.derived?.magic_resistance?.[0]?.total ?? null);

  function num(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value);
  }
</script>

<section class="panel supernatural-being">
  {#if store.ruleset}
    <div class="detail-section">
      <h3 class="detail-label">{store.t('supernatural-might-label')}</h3>
      {#if might}
        <label class="field inline">
          <span>{store.t('might-realm-label')}</span>
          <select
            value={might.realm}
            onchange={(e) =>
              store.setMightRealm((e.currentTarget as HTMLSelectElement).value as Realm)}
            data-testid="might-realm"
          >
            {#each REALMS as realm (realm)}
              <option value={realm}>{store.t(`realm-${realm}`)}</option>
            {/each}
          </select>
        </label>
        <label class="field inline">
          <span>{store.t('might-score-label')}</span>
          <input
            type="number"
            min="0"
            max="255"
            value={might.score}
            oninput={(e) => store.setMightScore(num(e))}
            data-testid="might-score"
          />
        </label>
        <button type="button" onclick={() => store.clearMight()} data-testid="might-clear">
          {store.t('might-clear')}
        </button>
      {:else}
        <p class="empty">{store.t('might-empty')}</p>
        <button
          type="button"
          onclick={() => store.setMightRealm('infernal')}
          data-testid="might-add"
        >
          {store.t('might-add')}
        </button>
      {/if}
      {#if effectiveMight}
        <p class="budget-readout" data-testid="might-effective">
          {store.t('might-effective', {
            realm: store.t(`realm-${effectiveMight.realm}`),
            score: String(effectiveMight.score),
          })}
        </p>
        {#if mightMr != null}
          <p class="budget-readout" data-testid="might-mr">
            {store.t('might-mr', { total: String(mightMr) })}
          </p>
        {/if}
      {/if}
    </div>

    <div class="detail-section">
      <h3 class="detail-label">{store.t('supernatural-powers-label')}</h3>
      <p class="budget-readout" data-testid="power-levels-used">
        {store.t('power-levels-used', { used: String(powerUsed), budget: String(powerBudget) })}
      </p>
      <ul class="power-list" data-testid="power-list">
        {#each powers as power, i (i)}
          <li>
            <input
              class="power-name"
              placeholder={store.t('power-name-placeholder')}
              aria-label={store.t('power-name-placeholder')}
              value={power.name}
              oninput={(e) => store.setPowerName(i, (e.currentTarget as HTMLInputElement).value)}
              data-testid="power-name-{i}"
            />
            <LevelRemoveField
              levelLabel={store.t('power-level-label')}
              levelValue={power.level}
              levelMin={0}
              levelMax={65535}
              levelTestid="power-level-{i}"
              onLevelInput={(value) => store.setPowerLevel(i, value)}
              removeLabel={store.t('remove-item', { name: power.name })}
              removeTestid="power-remove-{i}"
              onRemove={() => store.removePowerAt(i)}
            />
          </li>
        {:else}
          <li class="empty">{store.t('powers-empty')}</li>
        {/each}
      </ul>
      <button type="button" onclick={() => store.addPower()} data-testid="power-add">
        {store.t('power-add')}
      </button>
    </div>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
