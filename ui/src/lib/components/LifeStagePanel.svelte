<script lang="ts">
  import { store, type AbilityFunding } from '../state.svelte';
  import ChildhoodPackagePicker from './ChildhoodPackagePicker.svelte';

  // Where the character's experience comes from: one total the player enters, or
  // the experience its life stages earn.
  //
  // "Abilities ... for grogs and companions they are acquired in two blocks: early
  // childhood, and later life. For magi, there are two more periods to consider:
  // apprenticeship, and life as a magus after that."
  // Source: Ars Magica - Definitive Edition (Core Rules).md:2364

  // A ruleset shipping no life-stage rules cannot fund Abilities that way, so the
  // whole panel is absent rather than offered as a dead control.
  const rules = $derived(store.ruleset?.ruleset.life_stages ?? null);
  const funding = $derived(store.abilityFunding);
  const guided = $derived(funding === 'life_stages');
  const typeProfile = $derived(store.ruleset?.ruleset.type_profiles[store.entity.type_id] ?? null);
  // A magus carries two ages, not one: the age it is now, and the age its Gauntlet
  // came at. So the age means something different for a magus than for anyone else,
  // which the note below says in words.
  const isMagus = $derived(typeProfile?.is_magus ?? false);
  const age = $derived(store.entity.age ?? null);
  // The engine's age→max-Ability-score cap, echoed read-only beside the age.
  const ageCap = $derived(store.effective?.age_ability_cap ?? null);
  const nativeLanguage = $derived(store.entity.life_stages?.native_language ?? '');

  // The years after the Gauntlet are worth what the DATA says (30 points a year, 10
  // a charged lab season), so a ruleset shipping no such block grants nothing and the
  // three fields would be dead controls — hence the gate.
  const postGauntlet = $derived(rules?.post_apprenticeship ?? null);
  const showPostGauntlet = $derived(isMagus && postGauntlet != null);
  const plan = $derived(store.entity.life_stages ?? null);
  // The engine's own figures for those years; null until an age makes a budget.
  const budget = $derived(store.effective?.life_stage ?? null);

  /** A stored optional count as an input value: absent reads as an empty field. */
  function fieldValue(count: number | undefined): string {
    return count == null ? '' : String(count);
  }

  /** The number typed into a post-Gauntlet field, with a blank field meaning `null`. */
  function count(event: Event): number | null {
    const raw = (event.currentTarget as HTMLInputElement).value;
    return raw === '' ? null : Number(raw);
  }

  // The two funding modes, in the order they are offered. A fixed taxonomy (the
  // store's `AbilityFunding` union), not catalogue data.
  const options: AbilityFunding[] = ['pool', 'life_stages'];

  function hintId(option: AbilityFunding): string {
    return `ability-funding-${option}-hint`;
  }

  function onAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAge(raw === '' ? null : Number(raw));
  }

  function onNativeLanguage(event: Event) {
    store.setNativeLanguage((event.currentTarget as HTMLInputElement).value);
  }
</script>

{#if rules}
  <div class="life-stage-panel" data-testid="life-stage-panel">
    <fieldset class="ability-funding">
      <legend>{store.t('ability-funding-label')}</legend>
      {#each options as option (option)}
        <div class="funding-option">
          <label class="field inline">
            <input
              type="radio"
              name="ability-funding"
              value={option}
              checked={funding === option}
              aria-describedby={hintId(option)}
              onchange={() => store.setAbilityFunding(option)}
              data-testid="ability-funding-{option}"
            />
            <span>{store.t(`ability-funding-${option}`)}</span>
          </label>
          <span
            class="funding-hint"
            id={hintId(option)}
            data-testid="ability-funding-{option}-hint"
          >
            {store.t(`ability-funding-${option}-hint`)}
          </span>
        </div>
      {/each}
    </fieldset>

    {#if guided}
      <!-- Age is edited here as well as on the Details tab. Both surfaces read and
           write the one `entity.age`, so they cannot diverge — and the guided flow
           needs it, because later life's experience is (age - childhood years) × rate. -->
      {#if isMagus}
        <!-- Announced, and placed above the two age inputs it explains: a magus has an
             age AND a Gauntlet age, and neither field can say for itself which is
             which or what the years between them are worth. -->
        <p class="gauntlet-note" role="status" data-testid="life-stage-gauntlet-note">
          {store.t('life-stage-gauntlet-note')}
        </p>
      {/if}
      <div class="life-stage-fields">
        <label class="field inline">
          <span>{store.t('life-stage-age-label')}</span>
          <input
            type="number"
            min="1"
            max="4294967295"
            value={age ?? ''}
            oninput={onAge}
            data-testid="life-stage-age-input"
          />
        </label>
        {#if ageCap != null}
          <!-- Announced: it changes in response to the age edit beside it. Reuses the
               Details tab's own key, so one rule keeps one wording. -->
          <span class="age-cap" role="status" data-testid="life-stage-age-cap">
            {store.t('age-cap-note', { cap: String(ageCap) })}
          </span>
        {/if}
        {#if showPostGauntlet}
          <!-- In the same wrapping row as the age, deliberately: the panel is an
               auto-height sibling of the `flex: 1` region row below it, so every
               full-width row added here comes straight out of the Available/Selected
               lists' height (see `.region-row` in app.css). -->
          <label class="field inline">
            <span>{store.t('life-stage-gauntlet-age-label')}</span>
            <input
              type="number"
              min="1"
              max="4294967295"
              placeholder={age == null ? '' : String(age)}
              value={fieldValue(plan?.gauntlet_age)}
              aria-describedby="life-stage-gauntlet-age-hint"
              oninput={(event) => store.setGauntletAge(count(event))}
              data-testid="life-stage-gauntlet-age-input"
            />
          </label>
          <label class="field inline">
            <span>{store.t('life-stage-lab-seasons-label')}</span>
            <input
              type="number"
              min="0"
              max="4294967295"
              value={fieldValue(plan?.post_gauntlet_lab_seasons)}
              aria-describedby="life-stage-lab-seasons-hint"
              oninput={(event) => store.setPostGauntletLabSeasons(count(event))}
              data-testid="life-stage-lab-seasons-input"
            />
          </label>
          <label class="field inline">
            <span>{store.t('life-stage-spell-levels-label')}</span>
            <input
              type="number"
              min="0"
              max="4294967295"
              value={fieldValue(plan?.post_gauntlet_spell_levels)}
              aria-describedby="life-stage-spell-levels-hint"
              oninput={(event) => store.setPostGauntletSpellLevels(count(event))}
              data-testid="life-stage-spell-levels-input"
            />
          </label>
        {/if}
        <label class="field inline">
          <span>{store.t('native-language-label')}</span>
          <input
            type="text"
            placeholder={store.t('native-language-placeholder')}
            value={nativeLanguage}
            oninput={onNativeLanguage}
            data-testid="native-language-input"
          />
        </label>
      </div>
      {#if showPostGauntlet}
        <!-- The three hints share one wrapping row rather than sitting under their
             fields: three stacked hint lines would cost the region row below three
             more lines of height. -->
        <div class="life-stage-hints">
          {#each ['gauntlet-age', 'lab-seasons', 'spell-levels'] as field (field)}
            <span
              class="funding-hint"
              id="life-stage-{field}-hint"
              data-testid="life-stage-{field}-hint"
            >
              {store.t(`life-stage-${field}-hint`)}
            </span>
          {/each}
        </div>
        {#if budget}
          <!-- Announced: the numbers arrive in response to an edit above (either age,
               the lab seasons or the split), so their change must reach a screen
               reader. Every figure is the engine's own — nothing is recomputed here. -->
          <span
            class="post-gauntlet-summary"
            role="status"
            data-testid="life-stage-post-gauntlet-summary"
          >
            {store.t('life-stage-post-gauntlet-summary', {
              years: String(budget.post_gauntlet_years),
              points: String(budget.post_gauntlet_points),
              xp: String(budget.post_gauntlet_xp),
              levels: String(budget.post_gauntlet_spell_levels),
            })}
          </span>
        {/if}
      {/if}
      <!-- Childhood comes after the native language it is measured against: a
           package's native entry is bought in that language, and its own childhood
           language must differ from it. -->
      <ChildhoodPackagePicker />
    {/if}
  </div>
{/if}
