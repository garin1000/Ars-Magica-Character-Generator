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
  // A magus earns experience in four periods, and later life stops at apprenticeship
  // — which slice 6b4 adds. Until then the engine refuses the combination
  // (`life_stage_magus_guided_unsupported`), so the UI must not offer it: the option
  // is disabled with its reason spoken through `aria-describedby`, never greyed alone.
  const isMagus = $derived(typeProfile?.is_magus ?? false);
  const age = $derived(store.entity.age ?? null);
  // The engine's age→max-Ability-score cap, echoed read-only beside the age.
  const ageCap = $derived(store.effective?.age_ability_cap ?? null);
  const nativeLanguage = $derived(store.entity.life_stages?.native_language ?? '');

  // The two funding modes, in the order they are offered. A fixed taxonomy (the
  // store's `AbilityFunding` union), not catalogue data.
  const options: AbilityFunding[] = ['pool', 'life_stages'];

  const MAGUS_REASON_ID = 'ability-funding-magus-reason';

  function hintId(option: AbilityFunding): string {
    return `ability-funding-${option}-hint`;
  }

  function optionDisabled(option: AbilityFunding): boolean {
    return option === 'life_stages' && isMagus;
  }

  /** The hint a radio describes itself with, plus the reason when it is disabled. */
  function describedBy(option: AbilityFunding): string {
    return optionDisabled(option) ? `${hintId(option)} ${MAGUS_REASON_ID}` : hintId(option);
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
              disabled={optionDisabled(option)}
              aria-describedby={describedBy(option)}
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
          {#if optionDisabled(option)}
            <span class="funding-reason" id={MAGUS_REASON_ID} data-testid={MAGUS_REASON_ID}>
              {store.t('ability-funding-magus-reason')}
            </span>
          {/if}
        </div>
      {/each}
    </fieldset>

    {#if guided}
      <!-- Age is edited here as well as on the Details tab. Both surfaces read and
           write the one `entity.age`, so they cannot diverge — and the guided flow
           needs it, because later life's experience is (age - childhood years) × rate. -->
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
      <!-- Childhood comes after the native language it is measured against: a
           package's native entry is bought in that language, and its own childhood
           language must differ from it. -->
      <ChildhoodPackagePicker />
    {/if}
  </div>
{/if}
