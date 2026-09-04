<script lang="ts">
  import { formatSigned } from '../derive';
  import { store } from '../state.svelte';
  import { CHARACTERISTICS, REALMS, type Characteristic, type Realm } from '../types';
  import ConfirmPrompt from './ConfirmPrompt.svelte';
  import Spinner from './Spinner.svelte';
  import LevelRemoveField from './LevelRemoveField.svelte';

  const familiar = $derived(store.entity.familiar ?? null);
  const might = $derived(familiar?.might ?? null);
  const characteristics = $derived(familiar?.characteristics ?? {});
  const traits = $derived(familiar?.personality_traits ?? []);
  const powers = $derived(familiar?.powers ?? []);

  function num(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value);
  }

  // S18: removing the familiar discards its whole statblock — name, Might,
  // Characteristics, personality traits, cords, powers — in one click, with
  // no undo. Gate it behind a confirmation instead of calling
  // store.removeFamiliar() directly from the button.
  let confirmRemoveOpen = $state(false);
</script>

<div class="detail-section">
  <h3 class="detail-label">{store.t('familiar-label')}</h3>
  {#if familiar}
    <label class="field inline">
      <span>{store.t('familiar-name-placeholder')}</span>
      <input
        class="familiar-name"
        placeholder={store.t('familiar-name-placeholder')}
        value={familiar.name}
        oninput={(e) => store.setFamiliarName((e.currentTarget as HTMLInputElement).value)}
        data-testid="familiar-name"
      />
    </label>
    <label class="field inline">
      <span>{store.t('familiar-animal-label')}</span>
      <input
        class="familiar-animal"
        placeholder={store.t('familiar-animal-placeholder')}
        value={familiar.animal ?? ''}
        oninput={(e) => store.setFamiliarAnimal((e.currentTarget as HTMLInputElement).value)}
        data-testid="familiar-animal"
      />
    </label>
    <!-- Size is signed and commonly NEGATIVE (a raven is -4), so `min` is the i8
         floor rather than 0, and the number input renders the ASCII hyphen-minus
         natively. The bounds match the store's clamp: the field is an i8 in the
         engine, and a value serde cannot represent would fail every IPC call. -->
    <label class="field inline">
      <span>{store.t('familiar-size-label')}</span>
      <input
        type="number"
        min="-128"
        max="127"
        value={familiar.size ?? 0}
        oninput={(e) => store.setFamiliarSize(num(e))}
        data-testid="familiar-size"
      />
    </label>

    <!-- The familiar's OWN Magic Might. No Virtue grant stacks on it, which is why
         the score carries familiar-might-score-label rather than the character's
         might-score-label ("Base Might Score"). -->
    <h4 class="detail-sublabel">{store.t('familiar-might-label')}</h4>
    {#if might}
      <label class="field inline">
        <span>{store.t('might-realm-label')}</span>
        <select
          value={might.realm}
          onchange={(e) =>
            store.setFamiliarMightRealm((e.currentTarget as HTMLSelectElement).value as Realm)}
          data-testid="familiar-might-realm"
        >
          {#each REALMS as realm (realm)}
            <option value={realm}>{store.t(`realm-${realm}`)}</option>
          {/each}
        </select>
      </label>
      <label class="field inline">
        <span>{store.t('familiar-might-score-label')}</span>
        <input
          type="number"
          min="0"
          max="255"
          value={might.score}
          oninput={(e) => store.setFamiliarMightScore(num(e))}
          data-testid="familiar-might-score"
        />
      </label>
      <button
        type="button"
        onclick={() => store.clearFamiliarMight()}
        data-testid="familiar-might-clear"
      >
        {store.t('familiar-might-clear')}
      </button>
    {:else}
      <p class="empty" data-testid="familiar-might-empty">{store.t('familiar-might-empty')}</p>
      <button
        type="button"
        onclick={() => store.setFamiliarMightRealm('magic')}
        data-testid="familiar-might-add"
      >
        {store.t('familiar-might-add')}
      </button>
    {/if}

    <!-- All eight Characteristics are offered, in the engine's canonical order, as
         plain signed inputs — these are the creature's own scores, so there is no
         point-buy, no cap and no cost table to consult. -->
    <h4 class="detail-sublabel">{store.t('characteristics-title')}</h4>
    <div class="char-grid">
      {#each CHARACTERISTICS as characteristic (characteristic)}
        <label class="field inline">
          <span>{store.t(`characteristic-${characteristic}`)}</span>
          <input
            type="number"
            min="-128"
            max="127"
            value={characteristics[characteristic as Characteristic] ?? 0}
            oninput={(e) => store.setFamiliarCharacteristic(characteristic, num(e))}
            data-testid="familiar-char-{characteristic}"
          />
        </label>
      {/each}
    </div>
    <p class="hint" data-testid="familiar-characteristics-note">
      {store.t('familiar-characteristics-note')}
    </p>

    <h4 class="detail-sublabel">{store.t('personality-label')}</h4>
    <ul class="trait-list" data-testid="familiar-personality-list">
      {#each traits as trait, i (i)}
        <li>
          <input
            class="trait-name"
            placeholder={store.t('personality-name-placeholder')}
            aria-label={store.t('personality-name-placeholder')}
            value={trait.name}
            oninput={(e) =>
              store.setFamiliarPersonalityTraitName(i, (e.currentTarget as HTMLInputElement).value)}
            data-testid="familiar-personality-name-{i}"
          />
          <Spinner
            decLabel={store.t('characteristic-decrement', { name: trait.name })}
            decTestid="familiar-personality-dec-{i}"
            onDec={() => store.setFamiliarPersonalityTraitValue(i, trait.value - 1)}
            incLabel={store.t('characteristic-increment', { name: trait.name })}
            incTestid="familiar-personality-inc-{i}"
            onInc={() => store.setFamiliarPersonalityTraitValue(i, trait.value + 1)}
          >
            {#snippet children()}
              <span class="spinner-value" data-testid="familiar-personality-value-{i}">
                {formatSigned(trait.value)}
              </span>
            {/snippet}
          </Spinner>
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('remove-item', { name: trait.name })}
            onclick={() => store.removeFamiliarPersonalityTraitAt(i)}
            data-testid="familiar-personality-remove-{i}"
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
      onclick={() => store.addFamiliarPersonalityTrait()}
      data-testid="familiar-personality-add"
    >
      {store.t('personality-add')}
    </button>

    <!-- Cord scores run 0 to +5, "+5 (the maximum)" (Core:10836) — the bound here is
         the RULE, not the u8 the field is stored in; the store clamps to the same 5. -->
    <div class="cord-row">
      {#each [['gold', 'familiar-cord-gold'], ['silver', 'familiar-cord-silver'], ['bronze', 'familiar-cord-bronze']] as [cord, key] (cord)}
        <label class="field inline">
          <span>{store.t(key)}</span>
          <input
            type="number"
            min="0"
            max="5"
            value={familiar[`cord_${cord}` as 'cord_gold' | 'cord_silver' | 'cord_bronze'] ?? 0}
            oninput={(e) => store.setFamiliarCord(cord as 'gold' | 'silver' | 'bronze', num(e))}
            data-testid="familiar-cord-{cord}"
          />
        </label>
      {/each}
    </div>

    <!-- Powers invested in the bond. Deliberately NO budget read-out: "there is no
         limit to the number of powers which may be invested in a familiar"
         (Core:10866), so a bar here would invent a limit the rules deny. The absence
         of a bar is the whole statement — manual-testing-findings #21 removed the
         sentence that also said it in words. -->
    <h4 class="detail-sublabel">{store.t('familiar-powers-label')}</h4>
    <ul class="power-list" data-testid="familiar-power-list">
      {#each powers as power, i (i)}
        <li>
          <input
            class="power-name"
            placeholder={store.t('power-name-placeholder')}
            aria-label={store.t('power-name-placeholder')}
            value={power.name}
            oninput={(e) =>
              store.setFamiliarPowerName(i, (e.currentTarget as HTMLInputElement).value)}
            data-testid="familiar-power-name-{i}"
          />
          <LevelRemoveField
            levelLabel={store.t('power-level-label')}
            levelValue={power.level}
            levelMin={0}
            levelMax={65535}
            levelTestid="familiar-power-level-{i}"
            onLevelInput={(value) => store.setFamiliarPowerLevel(i, value)}
            removeLabel={store.t('remove-item', { name: power.name })}
            removeTestid="familiar-power-remove-{i}"
            onRemove={() => store.removeFamiliarPowerAt(i)}
          />
        </li>
      {:else}
        <li class="empty">{store.t('powers-empty')}</li>
      {/each}
    </ul>
    <button type="button" onclick={() => store.addFamiliarPower()} data-testid="familiar-power-add">
      {store.t('power-add')}
    </button>

    <!-- The bond's grants are SURFACED, never auto-applied: virtue.true_friend is
         not in the catalogue, and grant.rs keys off House/mythic profiles, not off
         a familiar bond. -->
    <p class="hint" data-testid="familiar-bond-note">{store.t('familiar-bond-note')}</p>

    <button type="button" onclick={() => (confirmRemoveOpen = true)} data-testid="familiar-remove">
      {store.t('familiar-remove')}
    </button>
    <ConfirmPrompt
      open={confirmRemoveOpen}
      titleKey="familiar-remove-confirm-title"
      messageKey="familiar-remove-confirm-message"
      confirmKey="familiar-remove-confirm-confirm"
      cancelKey="familiar-remove-confirm-cancel"
      testidPrefix="familiar-remove-confirm"
      onConfirm={() => {
        confirmRemoveOpen = false;
        store.removeFamiliar();
      }}
      onCancel={() => (confirmRemoveOpen = false)}
    />
  {:else}
    <button type="button" onclick={() => store.addFamiliar()} data-testid="familiar-add">
      {store.t('familiar-add')}
    </button>
  {/if}
</div>

<style>
  /* `.detail-section` (spacing within a section), `.field.inline` (label beside its
     input), `.empty`, `.hint`, `.spinner`, `.icon-btn` and `.trait-list` are shared
     globals in app.css. */

  /* Section-level action buttons (Add / Remove familiar, Add trait / power) are
     direct children of a `.detail-section`, which is a column flex with the default
     `align-items: stretch` — so without this they stretch to the full panel width.
     Shrink them to their label. */
  .detail-section > button {
    align-self: flex-start;
  }

  /* Sub-headings within the familiar statblock (Might, Characteristics, …), one
     step down from `.detail-label`. */
  .detail-sublabel {
    margin: 0.5rem 0 0;
    font-size: 0.85rem;
    font-weight: 600;
    opacity: 0.85;
  }

  .familiar-name,
  .familiar-animal {
    flex: 1;
    min-width: 0;
  }

  /* The eight Characteristics wrap into as many columns as fit. */
  .char-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr));
    gap: 0.25rem 1rem;
  }

  /* Familiar bond cords sit in a wrapping row of inline number fields. */
  .cord-row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }
</style>
