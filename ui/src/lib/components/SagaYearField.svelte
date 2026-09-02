<script lang="ts">
  import { store } from '../state.svelte';
  import { I32_MAX, I32_MIN } from '../derive';

  const sagaYear = $derived(store.sagaYear);

  function onSagaYear(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    if (raw === '') return;
    store.setSagaYear(Number(raw));
  }
</script>

<!-- The year the saga stands in (guided-creation-review-2026-08 #25).
     An APP setting, not part of any character: it is a saga fact shared by every
     character in one saga, so putting it on the entity would make per-character
     copies that disagree, and putting it in the ruleset would call it a rule. It is
     persisted by `arm-app` in its own settings file.

     It sits here, beside the age and the birth year, because that is the only place
     it does anything: those two are two views of one fact and this is the year they
     are measured against. Changing it rewrites NEITHER of them and does not dirty
     the document — advancing a saga means aging rolls, Living Conditions and any
     Longevity Ritual applied per year, not a subtraction (D3.3). The hint says so.

     Absent until the setting has been read: the default is a rules value the engine
     owns, so this field never states a year of its own. -->
{#if sagaYear != null}
  <div class="detail-field">
    <label class="field">
      <span>{store.t('saga-year-label')}</span>
      <input
        type="number"
        min={I32_MIN}
        max={I32_MAX}
        value={sagaYear}
        aria-describedby="saga-year-hint"
        oninput={onSagaYear}
        data-testid="saga-year-input"
      />
    </label>
    <span class="hint" id="saga-year-hint" data-testid="saga-year-hint">
      {store.t('saga-year-hint')}
    </span>
  </div>
{/if}
