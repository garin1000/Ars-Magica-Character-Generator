<script lang="ts">
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';
  import { I32_MAX, I32_MIN } from '../derive';

  const sagaYear = $derived(store.entity.saga_year);

  function onSagaYear(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    if (raw === '') return;
    store.setSagaYear(Number(raw));
  }
</script>

<!-- The year THIS character's saga stands in (guided-creation-review-2026-08 #25;
     moved onto the document by C8).

     It was a machine-global app setting until C8, and that was wrong: a storyguide
     running a 1220 Rhine saga and a 1197 Iberia saga had one number that was correct
     for one of them, so opening a character from the other reported the wrong age and
     could raise a spurious "not born yet" advisory. It is a property of the saga, not
     of the person at the keyboard, so it travels in the save. What the settings dialog
     keeps is only the year a NEW document starts at (`DefaultSagaYearField`).

     It sits beside the age and the birth year because that is where it does its work:
     those two are two views of one fact and this is the year they are measured
     against. Changing it rewrites NEITHER of them — advancing a saga means aging
     rolls, Living Conditions and any Longevity Ritual applied per year, not a
     subtraction (D3.3) — but it does dirty the document, because it is now part of
     what an unsaved file would lose. -->
<div class="detail-field">
  <label class="field">
    <span>{store.t('saga-year-label')}</span>
    <input
      type="number"
      min={I32_MIN}
      max={I32_MAX}
      value={sagaYear}
      oninput={onSagaYear}
      use:commitStored={{ read: () => sagaYear }}
      data-testid="saga-year-input"
    />
  </label>
</div>
