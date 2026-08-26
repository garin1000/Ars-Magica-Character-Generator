<script lang="ts">
  import { store } from '../state.svelte';

  // The character type is chosen once, when the character is created, so the
  // banner displays it and never offers to change it. `translate()` echoes an
  // unknown key back, so a save naming a type the loaded ruleset has no profile
  // for gets its own key instead of rendering the raw slug. (The engine reports
  // that save's `unknown_type` separately, so the real problem is still shown.)
  const profile = $derived(store.ruleset?.ruleset.type_profiles[store.entity.type_id]);
  const typeName = $derived(
    profile ? store.t(`type-${store.entity.type_id}`) : store.t('type-unknown'),
  );

  // Gated on the profile's Gift policy, never on the type id, so a new Gifted type
  // gets the right line as data.
  const giftPolicy = $derived(profile?.gift_policy);
</script>

<!-- Shared chrome above both the editor and the guided wizard: the character's
     immutable type plus its name and description. The wizard edits the same
     entity, so the banner it shows is the same one. -->
<section class="char-banner">
  <!-- The type, plus what it commits the character to. Slice 2 (#1) deleted the
       read-only `type` wizard step, whose only unique content was the Virtue/Flaw
       budget numbers and the Gift policy line; they live here now, on chrome both
       the editor and the wizard already show, so they stay reachable from every step
       instead of from one step nobody could act on. Every number is read from the
       loaded profile, so nothing here restates a rules value.
       They share the type's own row rather than adding one of their own. That is not
       cosmetic: this banner sits above every tab and every wizard step, so one extra
       line of permanent chrome comes straight out of the height the surfaces below
       need — and it measurably did, clipping the Virtues & Flaws picker by ~16px in
       an 800px window with no scrollbar to recover it. Wrapping, so a narrow window
       still costs at most a line, and only then. -->
  <p class="char-type" data-testid="character-type">
    <span class="char-type-label">{store.t('type-label')}</span>
    <span class="char-type-value">{typeName}</span>
    <span data-testid="character-type-explainer">{store.t('character-type-explainer')}</span>
    {#if profile}
      <span data-testid="character-type-budget">
        {store.t('character-type-budget', {
          virtues: String(profile.budget.virtue_points),
          flaws: String(profile.budget.flaw_points),
        })}
      </span>
      {#if giftPolicy === 'required'}
        <span data-testid="character-type-gift-required"
          >{store.t('character-type-gift-required')}</span
        >
      {:else if giftPolicy === 'forbidden'}
        <span data-testid="character-type-gift-forbidden"
          >{store.t('character-type-gift-forbidden')}</span
        >
      {:else if giftPolicy === 'allowed'}
        <span data-testid="character-type-gift-optional"
          >{store.t('character-type-gift-optional')}</span
        >
      {/if}
    {/if}
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
