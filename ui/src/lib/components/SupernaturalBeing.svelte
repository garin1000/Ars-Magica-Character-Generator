<script lang="ts">
  import { formatSigned } from '../derive';
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';
  import { REALMS, type Realm } from '../types';
  import LevelRemoveField from './LevelRemoveField.svelte';

  const might = $derived(store.entity.might ?? null);
  const powers = $derived(store.entity.powers ?? []);
  const focusPowers = $derived(store.entity.focus_powers ?? []);
  // The Focus Power pool and its per-power derivations are engine-authoritative
  // too — this panel only reads them.
  const focusPointsBudget = $derived(store.effective?.focus_points_budget ?? 0);
  const focusPointsUsed = $derived(store.effective?.focus_points_used ?? 0);
  const focusPowerLines = $derived(store.derived?.focus_powers ?? []);
  // "Magi never have Might" (the same rule App.svelte's tab gate states), and a
  // Hermetically trained entity reaches this tab only through Focus Power — so
  // the Might block and the level-budget power list, which are a Might-being's,
  // are not offered to it. Reads the engine's own profile-OR-selection union
  // (`DerivedTotals.hermetically_trained`, D56/A0) rather than the bare
  // type-profile flag: unlike App.svelte's tab *list* (still A2's job, since
  // that needs a resolved `phases_in_force`), this is content gating inside an
  // already-open tab, and the union already reaches the frontend today.
  const isMagus = $derived(store.derived?.hermetically_trained ?? false);
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
    {#if !isMagus}
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
              use:commitStored={{ read: () => might?.score }}
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
              <!-- Levels spent one-for-one on Penetration, out of the SAME budget
                 as the level (ArMDE:4019) — the engine's
                 `power_levels_used` counts both, so the bar above already
                 charges for this and it has to be enterable. Its own control
                 rather than a third field inside LevelRemoveField, which is
                 shared with four hosts that have no Penetration. -->
              <label class="field inline">
                <span>{store.t('power-penetration-label')}</span>
                <input
                  type="number"
                  min="0"
                  max="65535"
                  value={power.penetration ?? 0}
                  oninput={(e) =>
                    store.setPowerPenetration(
                      i,
                      Number((e.currentTarget as HTMLInputElement).value),
                    )}
                  use:commitStored={{ read: () => power.penetration ?? 0 }}
                  data-testid="power-penetration-{i}"
                />
              </label>
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
    {/if}

    <!-- Focus Powers draw on a SECOND pool, not the level budget above: "This
         Virtue grants a pool of 25 points… It costs 2 points to raise the maximum
         level of effect by 1, and 1 point to raise the Penetration by 1"
         (ArMDE:3899). Their level column is a ceiling on what the character may
         create, not a level that was spent, which is why they are their own list
         with their own bar rather than rows in the one above. -->
    <div class="detail-section">
      <h3 class="detail-label">{store.t('focus-powers-label')}</h3>
      <p class="budget-readout" data-testid="focus-points-used">
        {store.t('focus-points-used', {
          used: String(focusPointsUsed),
          budget: String(focusPointsBudget),
        })}
      </p>
      <ul class="power-list" data-testid="focus-power-list">
        {#each focusPowers as power, i (i)}
          <li>
            <input
              class="power-name"
              placeholder={store.t('focus-power-name-placeholder')}
              aria-label={store.t('focus-power-name-placeholder')}
              value={power.name}
              oninput={(e) =>
                store.setFocusPowerName(i, (e.currentTarget as HTMLInputElement).value)}
              data-testid="focus-power-name-{i}"
            />
            <label class="field inline">
              <span>{store.t('power-penetration-label')}</span>
              <input
                type="number"
                min="0"
                max="65535"
                value={power.penetration ?? 0}
                oninput={(e) => store.setFocusPowerPenetration(i, num(e))}
                use:commitStored={{ read: () => power.penetration ?? 0 }}
                data-testid="focus-power-penetration-{i}"
              />
            </label>
            <LevelRemoveField
              levelLabel={store.t('focus-power-max-level-label')}
              levelValue={power.max_level}
              levelMin={0}
              levelMax={65535}
              levelTestid="focus-power-max-level-{i}"
              onLevelInput={(value) => store.setFocusPowerMaxLevel(i, value)}
              removeLabel={store.t('remove-item', { name: power.name })}
              removeTestid="focus-power-remove-{i}"
              onRemove={() => store.removeFocusPowerAt(i)}
            />
            {#if focusPowerLines[i]}
              <!-- Magnitude, Initiative and the Fatigue cost are all engine-derived
                   (ArMDE:3899, ArMDE:3901, ArMDE:9097) and only displayed here. -->
              <p class="budget-readout" data-testid="focus-power-derived-{i}">
                {store.t('focus-power-derived', {
                  magnitude: String(focusPowerLines[i].magnitude),
                  initiative: formatSigned(focusPowerLines[i].initiative),
                  fatigue:
                    focusPowerLines[i].fatigue_levels == null
                      ? store.t('focus-power-fatigue-unstated')
                      : String(focusPowerLines[i].fatigue_levels),
                })}
              </p>
            {/if}
          </li>
        {:else}
          <li class="empty">{store.t('focus-powers-empty')}</li>
        {/each}
      </ul>
      <button type="button" onclick={() => store.addFocusPower()} data-testid="focus-power-add">
        {store.t('focus-power-add')}
      </button>
    </div>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
