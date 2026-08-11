<script lang="ts">
  import { store } from '../state.svelte';

  // The character type is chosen once, when the character is created, so the
  // banner displays it and never offers to change it. `translate()` echoes an
  // unknown key back, so a save naming a type the loaded ruleset has no profile
  // for gets its own key instead of rendering the raw slug. (The engine reports
  // that save's `unknown_type` separately, so the real problem is still shown.)
  const typeName = $derived(
    store.ruleset?.ruleset.type_profiles[store.entity.type_id]
      ? store.t(`type-${store.entity.type_id}`)
      : store.t('type-unknown'),
  );
</script>

<!-- Shared chrome above both the editor and the guided wizard: the character's
     immutable type plus its name and description. The wizard edits the same
     entity, so the banner it shows is the same one. -->
<section class="char-banner">
  <p class="char-type" data-testid="character-type">
    <span class="char-type-label">{store.t('type-label')}</span>
    <span class="char-type-value">{typeName}</span>
  </p>
  <input
    class="name-input"
    value={store.entity.name ?? ''}
    oninput={(e) => store.setIdentity('name', (e.currentTarget as HTMLInputElement).value)}
    placeholder={store.t('identity-name-placeholder')}
    aria-label={store.t('identity-name')}
    data-testid="identity-name"
  />
  <input
    class="desc-input"
    value={store.entity.description ?? ''}
    oninput={(e) => store.setIdentity('description', (e.currentTarget as HTMLInputElement).value)}
    placeholder={store.t('identity-description-placeholder')}
    aria-label={store.t('identity-description')}
    data-testid="identity-description"
  />
</section>
