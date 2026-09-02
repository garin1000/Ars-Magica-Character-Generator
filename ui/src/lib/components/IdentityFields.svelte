<script lang="ts">
  import { store } from '../state.svelte';
  import { I32_MAX, I32_MIN } from '../derive';

  function onBirthYear(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setBirthYear(raw === '' ? null : Number(raw));
  }
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
      min={I32_MIN}
      max={I32_MAX}
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
