<script lang="ts">
  import { store } from '../state.svelte';

  // The update check's consent as a Settings checkbox (M6.6, decision 3). Turning
  // it on writes the consent and THEN checks, once per session; turning it off
  // writes the refusal. A failed write reverts the box and reaches the error
  // banner (see `UpdateCheck`).
  const check = store.updateCheck;

  function onChange(event: Event) {
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    void (enabled ? check.enable() : check.disable());
  }
</script>

<label class="checkbox inline">
  <input
    type="checkbox"
    checked={check.enabled}
    onchange={onChange}
    data-testid="update-check-toggle"
  />
  <span>{store.t('settings-update-check')}</span>
</label>
