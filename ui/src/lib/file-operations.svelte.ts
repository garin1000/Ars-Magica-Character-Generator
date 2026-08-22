// Save/Save As/Export-Markdown and the New/Open discard-confirmation prompt.
// Extracted out of `AppStore` (VA6). `Open` and `New Document` stay on
// `AppStore` itself (VA6's own fix text notes the split "does not need to
// happen in one pass"): both reset several axes at once — the entity, the
// wizard rail, the childhood/aging drafts, the view — that belong to other
// modules or to `AppStore` directly, so folding them in here would mean this
// module reaching into every other module's state instead of `AppStore`
// composing them. What DOES move is the narrower "write bytes / prompt for a
// destination" mechanics those two orchestrators call into: the busy flag,
// the discard prompt, and the three commands that actually talk to `ipc`.

import * as ipc from './ipc';
import type { AppError, Entity, LocalizedRuleset } from './types';
import type { TranslateArgs } from './i18n';

/** File-name portion of a save path (handles both `/` and `\` separators). */
function fileNameOf(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

/**
 * The export label keys the engine composes from catalogue *data* and therefore
 * cannot list in `LABEL_KEYS` (see `crates/arm-rules/src/export.rs`): the
 * character-type subtitle `type-<profile id>`, the slot label
 * `param-label-<parameter key>` printed where a parameterized item has no chosen
 * value, and `category-<item category>` — the Type cell of an exported Virtue/Flaw
 * row, the same key the in-app badge renders. All three are read off the loaded
 * ruleset, so a new profile, parameterized item or category ships its label with
 * zero code changes.
 */
function composedExportLabelKeys(localized: LocalizedRuleset | null): string[] {
  if (!localized) return [];
  const keys = Object.keys(localized.ruleset.type_profiles ?? {}).map((id) => `type-${id}`);
  for (const key of parameterKeys(localized)) keys.push(`param-label-${key}`);
  for (const category of itemCategories(localized)) keys.push(`category-${category}`);
  return keys;
}

/**
 * Every distinct `category` the point-item catalogue uses. Names the
 * `category-<id>` label the exported Virtue/Flaw tables print in their Type column,
 * so the set follows the catalogue rather than a hardcoded list of categories.
 */
function itemCategories(localized: LocalizedRuleset): Set<string> {
  const categories = new Set<string>();
  for (const item of Object.values(localized.ruleset.point_items ?? {})) {
    if (item.category) categories.add(item.category);
  }
  return categories;
}

/**
 * Every parameter key the ruleset declares, across the three parameterized
 * catalogues (Virtues/Flaws, Abilities, spells). A key names both the
 * `{placeholder}` in the item's localized name and its `param-label-<key>` label,
 * which is what the exporter prints for an unfilled slot.
 */
function parameterKeys(localized: LocalizedRuleset): Set<string> {
  const rules = localized.ruleset;
  const keys = new Set<string>();
  for (const item of Object.values(rules.point_items ?? {})) {
    for (const param of item.parameters ?? []) keys.add(param.key);
  }
  for (const ability of Object.values(rules.abilities ?? {})) {
    if (ability.parameter) keys.add(ability.parameter);
  }
  for (const spell of Object.values(rules.spells ?? {})) {
    for (const param of spell.parameters ?? []) keys.add(param.key);
  }
  return keys;
}

/** The slice of `AppStore` file operations need, as live accessors so writes
 * land on the host's *current* state. `markSaved` is the one and only path
 * that updates `AppStore`'s close-guard baseline (`#savedSnapshot`) on a
 * successful write — the callback is defined in `AppStore`'s own body, so the
 * baseline keeps exactly one owner even though the *call* originates here. */
export interface FileOperationsHost {
  entity: () => Entity;
  /** Canonical serialized form of the current entity (see `AppStore.#snapshot`),
   * captured *before* the write's `await` so edits made while a dialog is open
   * are not folded into the baseline the write commits. */
  snapshot: () => string;
  /** Commit `snapshot` as the new close-guard baseline. */
  markSaved: (snapshot: string) => void;
  setError: (error: AppError | null) => void;
  ruleset: () => LocalizedRuleset | null;
  t: (key: string, args?: TranslateArgs) => string;
}

export class FileOperations {
  #host: FileOperationsHost;

  constructor(host: FileOperationsHost) {
    this.#host = host;
  }

  // Absolute path of the document's current file (from the last Open or the last
  // Save As / first Save). `null` for a never-saved document, so Save behaves as
  // Save As. Drives the window title too.
  currentPath = $state<string | null>(null);

  /** File name of the current document, or `null` when it has never been saved. */
  get currentFileName(): string | null {
    return this.currentPath ? fileNameOf(this.currentPath) : null;
  }

  // A Save/Save As/Open is running. A second one is a no-op until it finishes, so
  // a stray double click or shortcut can't stack native dialogs or races. Public
  // (not a private-field-plus-derived-getter pair): `AppStore.open()` also needs
  // to raise/lower it around its own `ipc.loadEntity()` call, since Open is not
  // one of this module's own commands.
  busy = $state(false);

  /** Whether the New/Open discard-confirmation prompt is currently shown. */
  discardPromptOpen = $state(false);
  // Resolver for the in-flight discard prompt (`true` = discard and proceed).
  #discardResolve: ((discard: boolean) => void) | null = null;

  /**
   * Show the discard-changes prompt and resolve once the user answers via
   * {@link resolveDiscardPrompt}. Resolves `true` to discard and proceed, `false`
   * to cancel. The UI renders a modal keyed off {@link discardPromptOpen}.
   */
  confirmDiscard(): Promise<boolean> {
    return new Promise((resolve) => {
      this.#discardResolve = resolve;
      this.discardPromptOpen = true;
    });
  }

  /** Answer the open discard prompt (called by the modal's buttons). */
  resolveDiscardPrompt(discard: boolean): void {
    this.discardPromptOpen = false;
    const resolve = this.#discardResolve;
    this.#discardResolve = null;
    resolve?.(discard);
  }

  /**
   * Save to the current file. A never-saved document (no {@link currentPath})
   * falls back to {@link saveAs} so the user picks a destination; otherwise it
   * writes straight to the tracked file with no prompt (standard document-app
   * behavior). No-op while another file operation is in flight.
   */
  async save(): Promise<void> {
    if (this.busy) return;
    if (this.currentPath === null) {
      await this.saveAs();
      return;
    }
    await this.#writeTo(this.currentPath);
  }

  /**
   * Always prompt for a destination and, on success, adopt it as the current
   * file. A cancelled prompt (null path) leaves the current file untouched and
   * the document dirty. No-op while another file operation is in flight.
   */
  async saveAs(): Promise<void> {
    if (this.busy) return;
    await this.#writeTo(null);
  }

  /**
   * Shared write path. `path === null` prompts (Save As / first Save); a concrete
   * path writes directly. On success clears dirty and records the written path as
   * the current file. The baseline is captured BEFORE awaiting, so edits made
   * while a dialog is open stay marked dirty.
   */
  async #writeTo(path: string | null): Promise<void> {
    this.busy = true;
    this.#host.setError(null);
    const snapshot = this.#host.snapshot();
    try {
      const written = await ipc.saveEntity($state.snapshot(this.#host.entity()), path);
      // A null return means the dialog was cancelled — nothing was written.
      if (written !== null) {
        this.currentPath = written;
        this.#host.markSaved(snapshot);
      }
    } catch (e) {
      this.#host.setError(e as AppError);
    } finally {
      this.busy = false;
    }
  }

  /**
   * Export the entity as a Markdown character sheet, prompting for a destination.
   * The current file is passed along so the prompt can default to its name and
   * directory (`gerhard.armc` -> `gerhard.md`).
   *
   * Deliberately **not** a save: the document keeps its current file, its dirty
   * flag and its saved baseline, so exporting a work in progress neither silences
   * the unsaved-changes guard nor retargets the next Save. It shares the
   * in-flight guard with the file operations so a stray second click cannot stack
   * two native dialogs.
   */
  async exportMarkdown(): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    this.#host.setError(null);
    try {
      await ipc.exportMarkdown(
        $state.snapshot(this.#host.entity()),
        await this.#exportLabels(),
        null,
        this.currentPath,
      );
    } catch (e) {
      this.#host.setError(e as AppError);
    } finally {
      this.busy = false;
    }
  }

  /**
   * The localized document chrome the exporter prints: every key the engine names,
   * plus the families it composes from catalogue data (see
   * {@link composedExportLabelKeys}), each resolved against the active bundle. The
   * engine hardcodes no user-facing string, so a key it never receives would print
   * as its own slug.
   */
  async #exportLabels(): Promise<Record<string, string>> {
    const keys = new Set(await ipc.exportLabelKeys());
    for (const key of composedExportLabelKeys(this.#host.ruleset())) keys.add(key);
    const labels: Record<string, string> = {};
    for (const key of keys) labels[key] = this.#host.t(key);
    return labels;
  }
}
