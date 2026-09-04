<script lang="ts">
  import { store, type AbilityFunding } from '../state.svelte';
  import { U32_MAX } from '../derive';
  import AgeFields from './AgeFields.svelte';
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
  // Read here only as the Gauntlet-age field's placeholder — the age itself is
  // edited on the Details tab / Concept step and shown here read-only through
  // `AgeFields` (Slice 12, #24). The age→Ability-score cap moved to the Abilities
  // surface, beside the lists it constrains.
  const age = $derived(store.entity.age ?? null);
  const nativeLanguage = $derived(store.entity.life_stages?.native_language ?? '');

  // The years after the Gauntlet are worth what the DATA says (30 points a year, 10
  // a charged lab season), so a ruleset shipping no such block grants nothing and the
  // three fields would be dead controls — hence the gate.
  const postGauntlet = $derived(rules?.post_apprenticeship ?? null);
  const showPostGauntlet = $derived(isMagus && postGauntlet != null);
  const plan = $derived(store.entity.life_stages ?? null);
  // What the engine reads a BLANK Gauntlet-age field as, so the placeholder states
  // it instead of contradicting it. The number is the ruleset's
  // (`apprenticeship.default_gauntlet_age` — "25 years old and just out of
  // apprenticeship"), never a constant here, and it is clamped to the character's
  // own age exactly as `LifeStageRules::budget` clamps it: a magus younger than the
  // baseline really does stand at its Gauntlet. With no baseline in the data the
  // older reading holds and the age is the answer.
  const defaultGauntletAge = $derived(rules?.apprenticeship?.default_gauntlet_age ?? null);
  const gauntletPlaceholder = $derived.by(() => {
    if (age == null) return '';
    return String(defaultGauntletAge == null ? age : Math.min(defaultGauntletAge, age));
  });
  // The engine's own figures for those years; null until an age makes a budget.
  const budget = $derived(store.effective?.life_stage ?? null);
  // A magus standing at its Gauntlet has lived no year as a magus, and BOTH the lab
  // seasons and the levels of spells are priced per year — three charged seasons a
  // year (`:2482`), 30 fungible points a year (`:2471`) — so with no years each
  // ceiling is 0 and every value either field could take is already an error. The
  // fields go read-only rather than staying open to be typed into: the state cannot
  // be entered by hand, and the note below says why it is closed.
  //
  // `readonly`, not `disabled` — a disabled control leaves the tab order and takes
  // its label out of the accessibility tree with it, so the one user who most needs
  // the explanation never lands on the field that carries it.
  //
  // Gated on the ENGINE's year count, not on the points: a points total of 0 can also
  // come from lab seasons eating the whole span, and that is a live consequence of
  // the neighbouring field which the player fixes by editing it. Only an empty span
  // is unfixable from either field — the Gauntlet age above ends it, and stays open.
  // `budget == null` is "not computed yet", never "no years".
  const noPostGauntletYears = $derived(budget != null && budget.post_gauntlet_years === 0);
  const noYearsNoteId = 'life-stage-post-gauntlet-no-years-note';

  /** A post-Gauntlet field's descriptions: its own hint, plus the read-only reason. */
  function describedBy(field: string): string {
    const hint = `life-stage-${field}-hint`;
    return noPostGauntletYears ? `${hint} ${noYearsNoteId}` : hint;
  }

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
      <!-- Slice 12 (#24) ended the duplication this panel used to carry. It had its
           own `life-stage-age-input`, a second editable copy of the one `entity.age`,
           because later life's experience is (age - childhood years) × rate and the
           panel had to be priceable. The age now has ONE canonical home — beside the
           identity it belongs with, on the editor's Details tab and the wizard's
           Concept step alike — so what is left here is `AgeFields` in `readonly`
           mode: the age is still shown, because a magus carries two ages and the
           Gauntlet age below is measured against this one, but it is no longer a
           second place to change it. -->
      {#if isMagus}
        <!-- Announced, and placed above the two age inputs it explains: a magus has an
             age AND a Gauntlet age, and neither field can say for itself which is
             which or what the years between them are worth. -->
        <p class="gauntlet-note" role="status" data-testid="life-stage-gauntlet-note">
          {store.t('life-stage-gauntlet-note')}
        </p>
      {/if}
      <div class="life-stage-fields">
        <AgeFields readonly />
        {#if showPostGauntlet}
          <!-- In the same wrapping row as the age, deliberately. On the wizard's own
               `experience` step (Slice 2) the panel has the step to itself, but it is
               still mounted above the editor's Abilities tab until Slice 3 gives that
               tab its own Experience sibling — and there it is an auto-height sibling
               of the `flex: 1` region row, so every full-width row added here comes
               straight out of the Available/Selected lists' height (see `.region-row`
               in app.css). -->
          <label class="field inline">
            <span>{store.t('life-stage-gauntlet-age-label')}</span>
            <input
              type="number"
              min="1"
              max={U32_MAX}
              placeholder={gauntletPlaceholder}
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
              max={U32_MAX}
              readonly={noPostGauntletYears}
              value={fieldValue(plan?.post_gauntlet_lab_seasons)}
              aria-describedby={describedBy('lab-seasons')}
              oninput={(event) => store.setPostGauntletLabSeasons(count(event))}
              data-testid="life-stage-lab-seasons-input"
            />
          </label>
          <label class="field inline">
            <span>{store.t('life-stage-spell-levels-label')}</span>
            <input
              type="number"
              min="0"
              max={U32_MAX}
              readonly={noPostGauntletYears}
              value={fieldValue(plan?.post_gauntlet_spell_levels)}
              aria-describedby={describedBy('spell-levels')}
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
          {#if noPostGauntletYears}
            <!-- Referenced by both read-only fields through `aria-describedby`, so it
                 is deliberately NOT a `role="status"` live region: the two together
                 would announce it twice, once on arrival and again on focus. -->
            <span class="funding-hint" id={noYearsNoteId} data-testid={noYearsNoteId}>
              {store.t('life-stage-post-gauntlet-no-years-note')}
            </span>
          {/if}
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
