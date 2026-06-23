// Central application state (Svelte 5 runes). A single instance is shared
// across components. It owns the entity being edited, the loaded ruleset, the
// validation result, and the active language/mode — and drives the live
// validation loop with debouncing plus a sequence guard against stale results.

import { buildBundle, translate, type Lang, type TranslateArgs } from './i18n';
import * as ipc from './ipc';
import type {
  AppError,
  Characteristic,
  Entity,
  LocalizedRuleset,
  ValidationMode,
  ValidationResult,
} from './types';

const VALIDATE_DEBOUNCE_MS = 150;
const SCHEMA_VERSION = 2;

function newEntity(rulesetId: string, version: string): Entity {
  return {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: rulesetId, version },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Record<Characteristic, number>,
    ability_scores: [],
    unspent_xp: 0,
  };
}

class AppStore {
  lang = $state<Lang>('en');
  ruleset = $state<LocalizedRuleset | null>(null);
  entity = $state<Entity>(newEntity('', ''));
  mode = $state<ValidationMode>('enforced');
  result = $state<ValidationResult | null>(null);
  error = $state<AppError | null>(null);
  loading = $state(false);

  #bundle = $derived(buildBundle(this.lang));
  #timer: ReturnType<typeof setTimeout> | undefined;
  #seq = 0;

  /** Translate a UI-chrome key. Bound so it can be passed to components. */
  t = (key: string, args?: TranslateArgs): string => translate(this.#bundle, key, args);

  /** Load the ruleset for the current language and validate the initial entity. */
  async init(): Promise<void> {
    await this.#reloadRuleset(true);
  }

  async setLang(lang: Lang): Promise<void> {
    if (lang === this.lang) return;
    this.lang = lang;
    // Rules display text is per-language, so reload the ruleset too.
    await this.#reloadRuleset(false);
  }

  async setMode(mode: ValidationMode): Promise<void> {
    this.mode = mode;
    await this.revalidate();
  }

  addSelection(ref: string): void {
    if (this.entity.selections.some((s) => s.ref === ref)) return;
    this.entity.selections.push({ ref });
    this.#scheduleValidate();
  }

  removeSelection(ref: string): void {
    this.entity.selections = this.entity.selections.filter((s) => s.ref !== ref);
    this.#scheduleValidate();
  }

  setParam(ref: string, key: string, value: string): void {
    const selection = this.entity.selections.find((s) => s.ref === ref);
    if (!selection) return;
    selection.params = { ...(selection.params ?? {}), [key]: value };
    this.#scheduleValidate();
  }

  /** Set or clear a Characteristic score (score 0 removes the explicit entry). */
  setCharacteristic(characteristic: Characteristic, score: number): void {
    const chars = { ...(this.entity.characteristics ?? {}) } as Record<Characteristic, number>;
    if (score === 0) {
      delete chars[characteristic];
    } else {
      chars[characteristic] = score;
    }
    this.entity.characteristics = chars;
    this.#scheduleValidate();
  }

  /**
   * Set an ability's whole bought score and specialty in the direct-entry UI.
   * One entry per ability id (a score of 0 removes it). The engine's data model
   * also supports multiple entries per ability with distinct specialties, but the
   * direct-entry UI keeps one row per catalogue ability.
   */
  setAbility(ability: string, score: number, specialty?: string): void {
    const spec = specialty?.trim() ? specialty.trim() : undefined;
    const rest = (this.entity.ability_scores ?? []).filter((a) => a.ability !== ability);
    this.entity.ability_scores = score > 0 ? [...rest, { ability, score, specialty: spec }] : rest;
    this.#scheduleValidate();
  }

  removeAbility(ability: string): void {
    this.entity.ability_scores = (this.entity.ability_scores ?? []).filter(
      (a) => a.ability !== ability,
    );
    this.#scheduleValidate();
  }

  setUnspentXp(xp: number): void {
    this.entity.unspent_xp = Number.isFinite(xp) && xp > 0 ? Math.floor(xp) : 0;
    this.#scheduleValidate();
  }

  async save(): Promise<void> {
    this.error = null;
    try {
      await ipc.saveEntity($state.snapshot(this.entity));
    } catch (e) {
      this.error = e as AppError;
    }
  }

  async load(): Promise<void> {
    this.error = null;
    try {
      const loaded = await ipc.loadEntity();
      if (loaded) {
        this.entity = loaded;
        await this.revalidate();
      }
    } catch (e) {
      this.error = e as AppError;
    }
  }

  /** Validate now, ignoring any in-flight response that finishes out of order. */
  async revalidate(): Promise<void> {
    if (!this.ruleset) return;
    const seq = ++this.#seq;
    try {
      const result = await ipc.validateEntity($state.snapshot(this.entity), this.mode);
      if (seq === this.#seq) this.result = result;
    } catch (e) {
      this.error = e as AppError;
    }
  }

  #scheduleValidate(): void {
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.revalidate(), VALIDATE_DEBOUNCE_MS);
  }

  async #reloadRuleset(resetEntity: boolean): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const localized = await ipc.loadRuleset(this.lang);
      this.ruleset = localized;
      const { id, version } = localized.ruleset;
      if (resetEntity) {
        this.entity = newEntity(id, version);
      } else {
        this.entity.ruleset = { id, version };
      }
      await this.revalidate();
    } catch (e) {
      this.error = e as AppError;
    } finally {
      this.loading = false;
    }
  }
}

export const store = new AppStore();
