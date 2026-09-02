<script lang="ts">
  // The "labeled level number field + remove button" tail shared by every
  // "named item + numeric level + remove" row: `MagicPossessions` (devices),
  // `TalismanPanel` (attunement bonus, effect level), `SupernaturalBeing` and
  // `FamiliarPanel` (powers) — V30, full-audit round. Scoped to exactly this
  // pair rather than the whole row: the row's own name `<input>` differs across
  // hosts in class and (for two of the five) a `flex: 1` layout rule declared in
  // the host's own scoped `<style>` block, which a shared child component's
  // markup cannot pick up (Svelte style scoping does not reach into a child
  // component's own template). This field and the button use only classes
  // already global in `app.css` (`.field.inline`, `.icon-btn`), so factoring
  // exactly this subset carries no such risk.
  let {
    levelLabel,
    levelValue,
    levelMin,
    levelMax,
    levelTestid,
    onLevelInput,
    removeLabel,
    removeTestid,
    onRemove,
  }: {
    levelLabel: string;
    levelValue: number;
    levelMin: number;
    levelMax: number;
    levelTestid: string;
    onLevelInput: (value: number) => void;
    removeLabel: string;
    removeTestid: string;
    onRemove: () => void;
  } = $props();
</script>

<label class="field inline">
  <span>{levelLabel}</span>
  <input
    type="number"
    min={levelMin}
    max={levelMax}
    value={levelValue}
    oninput={(e) => onLevelInput(Number((e.currentTarget as HTMLInputElement).value))}
    data-testid={levelTestid}
  />
</label>
<button
  type="button"
  class="icon-btn"
  aria-label={removeLabel}
  onclick={onRemove}
  data-testid={removeTestid}
>
  ×
</button>
