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
  import { commitStored } from '../actions';

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

  // manual-testing-findings-2026-11-15 #9 (F9): `TalismanPanel`'s attunement bonus
  // is the one host that passes a SIGNED range (-128..127); a `type="number"`
  // input reads an incomplete entry — a bare "-" on the way to "-4" — back as the
  // empty string, so forwarding it unconditionally wrote 0 (the host's own
  // `clampInt` reads a non-finite value as 0), and the plain `value={levelValue}`
  // binding snapped the field back before the digit after the sign could ever be
  // typed. The gate is generic — `levelMin`/`levelMax` are already passed by every
  // host — so it is a no-op for the four unsigned (0..65535) call sites.
  function onLevelFieldInput(event: Event): void {
    const raw = (event.currentTarget as HTMLInputElement).value;
    if (raw === '') return;
    const typed = Number(raw);
    if (!Number.isFinite(typed) || typed < levelMin || typed > levelMax) return;
    onLevelInput(typed);
  }

  // Commit (blur or Enter): forward the raw text unconditionally, so the host's
  // own clamp (`clampInt`) still has the final word, exactly as it already does
  // for every keystroke that stayed in bounds.
  function commitLevel(raw: string): void {
    onLevelInput(Number(raw));
  }
</script>

<label class="field inline">
  <span>{levelLabel}</span>
  <input
    type="number"
    min={levelMin}
    max={levelMax}
    value={levelValue}
    oninput={onLevelFieldInput}
    use:commitStored={{ read: () => levelValue, commit: commitLevel }}
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
