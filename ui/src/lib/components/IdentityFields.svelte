<script lang="ts">
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';
  import { REALMS, type Realm } from '../types';

  // The store clamps the birth year to [saga year - max_age, saga year - 1] (slice A1,
  // N3), and every prefix of an ordinary year ("1", "11", "119") lies under the lower
  // bound while digits typed in front of a year overshoot the upper one. So a
  // keystroke only reaches the store once the typed value is in bounds; anything else
  // stays in the field as typed, and is clamped when committed.
  function onBirthYearInput(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    if (raw !== '') {
      const year = Number(raw);
      if (year < store.earliestBirthYear || year > store.latestBirthYear) return;
    }
    store.setBirthYear(raw === '' ? null : Number(raw));
  }

  // Commit (blur or Enter): `commitStored` hands the typed text to the store, which
  // clamps it, then shows the stored year (N2).
  function commitBirthYear(raw: string) {
    store.setBirthYear(raw === '' ? null : Number(raw));
  }

  function onConceptRealm(event: Event) {
    const raw = (event.currentTarget as HTMLSelectElement).value;
    store.setConceptRealm(raw === '' ? null : (raw as Realm));
  }

  // manual-testing-findings #18: Sigil and Parens used to show for every
  // character. Sigil is a magus's own thing ("recurrent mark or quirk in a
  // magus's spell", ArMDE:12453) — gated on the TYPE PROFILE's own
  // `hermetically_trained` flag (never the entity-level union, which also
  // reads true for an Abandoned Apprentice trained by selection rather than
  // by profile — Sigil is narrower than that). Parens is broader: any
  // Hermetically trained character (the entity-level union,
  // `DerivedTotals.hermetically_trained`) OR a Redcap — `virtue.redcap`/
  // `virtue.lone_redcap` never set `ConfersHermeticTraining` (they fund
  // apprenticeship via `replaces_life_stage_xp` instead), so the entity-level
  // union reads false for a Redcap and the Virtue is checked directly
  // ("trained in a similar manner to magi", ArMDE:4919). Never a hardcoded
  // type id, so a future type gains either field purely by data. UI-only: a
  // value stored before the gate closed stays on the entity untouched.
  const profile = $derived(store.ruleset?.ruleset.type_profiles[store.entity.type_id]);
  const isMagusShaped = $derived(profile?.hermetically_trained ?? false);
  const isRedcap = $derived(
    (store.entity.selections ?? []).some(
      (s) => s.ref === 'virtue.redcap' || s.ref === 'virtue.lone_redcap',
    ),
  );
  const showSigil = $derived(isMagusShaped);
  const showParens = $derived((store.derived?.hermetically_trained ?? false) || isRedcap);
</script>

<!-- The character's concept and identity fields. Extracted from CharacterDetails
     so the wizard's `concept` step can mount them on their own, without dragging
     the age, aging, Warping and Twilight surfaces along. Name and description are
     not here: they live in the banner, above whichever view is up. -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('identity-label')}</h3>
  <label class="field">
    <span>{store.t('identity-concept')}</span>
    <textarea
      class="concept-input"
      rows="3"
      value={store.entity.concept ?? ''}
      oninput={(e) => store.setIdentity('concept', (e.currentTarget as HTMLTextAreaElement).value)}
      placeholder={store.t('identity-concept-placeholder')}
      data-testid="identity-concept"></textarea>
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
      min={store.earliestBirthYear}
      max={store.latestBirthYear}
      value={store.entity.birth_year ?? ''}
      oninput={onBirthYearInput}
      use:commitStored={{ read: () => store.entity.birth_year, commit: commitBirthYear }}
      data-testid="identity-birth-year"
    />
  </label>
  <label class="field">
    <span>{store.t('identity-concept-realm')}</span>
    <select
      value={store.entity.concept_realm ?? ''}
      onchange={onConceptRealm}
      data-testid="identity-concept-realm"
    >
      <option value="">{store.t('identity-concept-realm-none')}</option>
      {#each REALMS as realm (realm)}
        <option value={realm}>{store.t(`realm-${realm}`)}</option>
      {/each}
    </select>
  </label>
  {#if showSigil}
    <label class="field">
      <span>{store.t('identity-sigil')}</span>
      <input
        value={store.entity.sigil ?? ''}
        oninput={(e) => store.setIdentity('sigil', (e.currentTarget as HTMLInputElement).value)}
        data-testid="identity-sigil"
      />
    </label>
  {/if}
  <label class="field">
    <span>{store.t('identity-covenant')}</span>
    <input
      value={store.entity.covenant_name ?? ''}
      oninput={(e) =>
        store.setIdentity('covenant_name', (e.currentTarget as HTMLInputElement).value)}
      data-testid="identity-covenant"
    />
  </label>
  {#if showParens}
    <label class="field">
      <span>{store.t('identity-parens')}</span>
      <input
        value={store.entity.parens ?? ''}
        oninput={(e) => store.setIdentity('parens', (e.currentTarget as HTMLInputElement).value)}
        data-testid="identity-parens"
      />
    </label>
  {/if}
</div>
