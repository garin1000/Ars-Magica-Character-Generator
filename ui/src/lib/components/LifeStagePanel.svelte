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
  // A magus is built through its life stages too, but only AT its Gauntlet: its
  // apprenticeship is the fifteen years ending at the age entered, and life as a magus
  // after that is not counted yet (6b5). So the age means something different for a
  // magus than for anyone else, which the note below says in words.
  const isMagus = $derived(typeProfile?.is_magus ?? false);
  const age = $derived(store.entity.age ?? null);
  // The engine's age→max-Ability-score cap, echoed read-only beside the age.
  const ageCap = $derived(store.effective?.age_ability_cap ?? null);
  const nativeLanguage = $derived(store.entity.life_stages?.native_language ?? '');

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
        <!-- Announced, and placed above the age input it constrains: for a magus the
             age entered is the Gauntlet age, and the years lived as a magus after it
             are not counted yet — so the field cannot be read at face value. -->
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
