<script lang="ts">
  import { store } from '../state.svelte';

  // The character type was fixed when the character was created and can never
  // change, so this step reads rather than asks: it says what the type commits the
  // character to, which is what the rest of the flow then obeys.
  const profile = $derived(store.ruleset?.ruleset.type_profiles[store.entity.type_id]);

  // `translate()` echoes an unknown key back, so a save naming a type the loaded
  // ruleset has no profile for gets `type-unknown` rather than its raw slug.
  const typeName = $derived(
    profile ? store.t(`type-${store.entity.type_id}`) : store.t('type-unknown'),
  );

  // Gated on the profile's Gift policy, never on the type id, so a new Gifted type
  // gets the right line as data.
  const giftPolicy = $derived(profile?.gift_policy);
</script>

<section class="panel type-step">
  <h3 class="detail-label" data-testid="type-step-name">{typeName}</h3>
  <p>{store.t('phase-type-explainer')}</p>
  {#if profile}
    <p data-testid="type-step-budget">
      {store.t('phase-type-budget', {
        virtues: String(profile.budget.virtue_points),
        flaws: String(profile.budget.flaw_points),
      })}
    </p>
    {#if giftPolicy === 'required'}
      <p data-testid="type-step-gift-required">{store.t('phase-type-gift-required')}</p>
    {:else if giftPolicy === 'forbidden'}
      <p data-testid="type-step-gift-forbidden">{store.t('phase-type-gift-forbidden')}</p>
    {:else if giftPolicy === 'allowed'}
      <p data-testid="type-step-gift-optional">{store.t('phase-type-gift-optional')}</p>
    {/if}
  {/if}
</section>
