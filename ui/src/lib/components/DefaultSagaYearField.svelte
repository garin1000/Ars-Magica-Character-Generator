<script lang="ts">
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';
  import { I32_MAX, I32_MIN } from '../derive';

  const defaultSagaYear = $derived(store.defaultSagaYear);

  function onDefaultSagaYear(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    if (raw === '') return;
    store.setDefaultSagaYear(Number(raw));
  }
</script>

<!-- The saga year a NEW document starts at (C8).

     This is the half of the old saga-year setting that really is a preference: not
     "which year is it", which is a property of the saga and now lives in each save
     (`SagaYearField`), but "which year should a character I create next be built
     for". Changing it touches no open document — a character built for another saga
     keeps its own year — and is persisted beside the language and the palette.

     The label says what it seeds, so the dialog needs no explanatory sentence under
     it; the field is the only place the distinction has to be legible. -->
<label class="field">
  <span>{store.t('settings-default-saga-year-label')}</span>
  <input
    type="number"
    min={I32_MIN}
    max={I32_MAX}
    value={defaultSagaYear}
    oninput={onDefaultSagaYear}
    use:commitStored={{ read: () => defaultSagaYear }}
    data-testid="default-saga-year-input"
  />
</label>
