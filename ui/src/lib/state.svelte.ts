// Central application state (Svelte 5 runes). A single instance is shared
// across components. It owns the entity being edited, the loaded ruleset, the
// validation result, and the active language/mode — and drives the live
// validation loop with debouncing plus a sequence guard against stale results.

import { mandatoryTraitRefs, sameSelection } from './derive';
import { buildBundle, translate, type Lang, type TranslateArgs } from './i18n';
import * as ipc from './ipc';
import type { CloseGuardLabels } from './ipc';
import type {
  AppError,
  Characteristic,
  DerivedTotals,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
  LongevitySource,
  Realm,
  ReputationType,
  Selection,
  ValidationMode,
  ValidationResult,
} from './types';

const VALIDATE_DEBOUNCE_MS = 150;

/**
 * Save-format schema version written into every new entity. Exported so test
 * harnesses seed the current version instead of a literal that silently rots.
 */
export const SCHEMA_VERSION = 14;

// Inclusive ranges of the fixed-width Rust integer fields the entity's numbers
// land in. A value outside its field's range makes serde reject the whole payload
// at the Tauri boundary, which fails `validate`, `effective_scores` and
// `derived_totals` at once — leaving every read-out frozen on stale numbers that
// still look current. Mutators clamp instead, so the engine always gets a
// representable value and the panel inputs carry the matching min/max.
const I8_MIN = -128;
const I8_MAX = 127;
const U8_MAX = 255;
const U16_MAX = 65535;
const I32_MIN = -2147483648;
const I32_MAX = 2147483647;
const U32_MAX = 4294967295;

// A few fields are bounded by the RULES more tightly than by their serde width,
// and the rule is the bound that belongs at the point of entry — a value the
// engine's consumers disagree about is worse than one it rejects outright.
/** "The strength of each of these cords is rated from 0 to +5 … a score of +5
 * (the maximum)" — Source: Ars Magica - Definitive Edition (Core Rules).md:10836. */
const CORD_MAX = 5;

/** Truncate to an integer inside an inclusive range; a non-finite value becomes 0
 * (itself clamped into range), the same fallback the field's default carries. */
function clampInt(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return Math.min(Math.max(0, min), max);
  return Math.min(max, Math.max(min, Math.trunc(value)));
}

/** Live filter/search state of the Virtue/Flaw picker (one per side). */
export interface VfFilterState {
  search: string;
  magnitude: string;
  category: string;
  taintedOnly: boolean;
}

/** Live filter/search state of the Ability picker. */
export interface AbilityFilterState {
  search: string;
  category: string;
}

/** Live filter/search state of the Spell picker. */
export interface SpellFilterState {
  search: string;
  technique: string;
  form: string;
  /** Inclusive min/max level range; null = open bound (empty input). */
  levelMin: number | null;
  levelMax: number | null;
}

/** Live filter/search state of the Equipment picker (kind = weapons/shields/armor). */
export interface EquipmentFilterState {
  search: string;
  kind: string;
}

/**
 * Per-panel view state, lifted out of the panel components so it survives tab
 * switches. Each `{#if tab === …}` panel in `App.svelte` unmounts its content,
 * which would discard any component-local `$state`; holding it here (keyed by
 * panel identity) restores it when the tab returns. Mostly picker search/filter
 * state, plus the Derived-Totals Technique/Form picker selection (`derivedArtPicker`,
 * a non-filter choice). Language-neutral (search text + ids), so it is never
 * reset on a ruleset reload and never enters the saved entity.
 */
export interface PickerFilters {
  vf: Record<'virtue' | 'flaw', VfFilterState>;
  abilities: AbilityFilterState;
  spells: SpellFilterState;
  equipment: EquipmentFilterState;
  /** Derived-Totals Lab/Casting picker: the chosen Technique and Form art ids. */
  derivedArtPicker: { technique: string; form: string };
}

/** File-name portion of a save path (handles both `/` and `\` separators). */
function fileNameOf(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

/** A fresh, all-empty set of picker filters (the initial/reset state). */
export function defaultPickerFilters(): PickerFilters {
  return {
    vf: {
      virtue: { search: '', magnitude: '', category: '', taintedOnly: false },
      flaw: { search: '', magnitude: '', category: '', taintedOnly: false },
    },
    abilities: { search: '', category: '' },
    spells: { search: '', technique: '', form: '', levelMin: null, levelMax: null },
    equipment: { search: '', kind: '' },
    derivedArtPicker: { technique: '', form: '' },
  };
}

function newEntity(rulesetId: string, version: string): Entity {
  return {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: rulesetId, version },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Record<Characteristic, number>,
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    art_scores: [],
    spells: [],
    house: null,
    mythic_type: null,
    age: null,
    personality_traits: [],
    reputations: [],
  };
}

class AppStore {
  lang = $state<Lang>('en');
  ruleset = $state<LocalizedRuleset | null>(null);
  entity = $state<Entity>(newEntity('', ''));
  mode = $state<ValidationMode>('enforced');
  result = $state<ValidationResult | null>(null);
  effective = $state<EffectiveScores | null>(null);
  derived = $state<DerivedTotals | null>(null);
  error = $state<AppError | null>(null);
  loading = $state(false);
  // Per-picker filter/search state; persists across tab switches (see
  // {@link PickerFilters}). Not part of the entity, so it is never saved.
  filters = $state<PickerFilters>(defaultPickerFilters());

  // Absolute path of the document's current file (from the last Open or the last
  // Save As / first Save). `null` for a never-saved document, so Save behaves as
  // Save As. Drives the window title too.
  currentPath = $state<string | null>(null);

  /** File name of the current document, or `null` when it has never been saved. */
  currentFileName = $derived(this.currentPath ? fileNameOf(this.currentPath) : null);

  // A Save/Save As/Open is running. A second one is a no-op until it finishes, so
  // a stray double click or shortcut can't stack native dialogs or races.
  #opInFlight = $state(false);

  /** Whether a file operation (Save/Save As/Open) is running; disables the toolbar. */
  busy = $derived(this.#opInFlight);

  /** Whether the New/Open discard-confirmation prompt is currently shown. */
  discardPromptOpen = $state(false);
  // Resolver for the in-flight discard prompt (`true` = discard and proceed).
  #discardResolve: ((discard: boolean) => void) | null = null;

  // Serialized snapshot of the entity as of the last save/load — the baseline
  // the close/quit guard compares against. Seeded from the initial entity so
  // `dirty` is not spuriously true before init() runs.
  #savedSnapshot = $state<string>(this.#snapshot());

  /**
   * Whether the entity has unsaved edits. A stringify-compare against the
   * last-saved baseline: it only ever errs toward a spurious prompt (false
   * positive), never toward silently discarding work (false negative).
   */
  dirty = $derived(this.#snapshot() !== this.#savedSnapshot);

  #bundle = $derived(buildBundle(this.lang));
  #timer: ReturnType<typeof setTimeout> | undefined;
  #seq = 0;

  /** Translate a UI-chrome key. Bound so it can be passed to components. */
  t = (key: string, args?: TranslateArgs): string => translate(this.#bundle, key, args);

  /** Canonical serialized form of the current entity, for dirty comparison. */
  #snapshot(): string {
    return JSON.stringify($state.snapshot(this.entity));
  }

  /**
   * The payload the backend close/quit guard needs: whether there are unsaved
   * edits, plus the localized dialog strings (kept in the frontend so no
   * user-facing text lives in Rust).
   */
  closeGuardPayload(): { dirty: boolean; labels: CloseGuardLabels } {
    return {
      dirty: this.dirty,
      labels: {
        title: this.t('close-unsaved-title'),
        message: this.t('close-unsaved-message'),
        discard: this.t('close-unsaved-discard'),
        cancel: this.t('close-unsaved-cancel'),
      },
    };
  }

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

  /**
   * Switch the character type in place. Selections, characteristics and
   * abilities are kept; live validation then flags anything the new type's
   * budget/category rules forbid (e.g. a Hermetic flaw on a grog). The available
   * direct-entry sections (Arts/Spells/House) follow the new profile's
   * capability flags, never the type id.
   */
  async setType(typeId: string): Promise<void> {
    if (this.entity.type_id === typeId) return;
    const previousMandatory = this.#mandatoryTraitRefs(this.entity.type_id);
    this.entity.type_id = typeId;
    const nextMandatory = this.#mandatoryTraitRefs(typeId);
    // Auto-manage the type's mandatory free traits (a magus's The Gift + Hermetic
    // Magus): drop the previous type's that the new type doesn't mandate, then
    // add any the new type mandates but that aren't already selected. Both are
    // free, so this never touches the point budget; it makes a direct-entry magus
    // legal without hand-picking them. The set is data-driven (profile
    // required_traits + required gift_id), so no id is hardcoded here.
    this.entity.selections = this.entity.selections.filter(
      (s) => !(previousMandatory.has(s.ref) && !nextMandatory.has(s.ref)),
    );
    for (const ref of nextMandatory) {
      if (!this.entity.selections.some((s) => s.ref === ref)) {
        this.entity.selections.push({ ref });
      }
    }
    // A type switch is a discrete action (not rapid typing), so validate
    // immediately rather than through the debounce — mirrors setMode and avoids
    // a stale debounced result from the prior type winning the race.
    await this.revalidate();
  }

  /** The item ids the given type profile mandates (required traits + required Gift). */
  #mandatoryTraitRefs(typeId: string | undefined): Set<string> {
    const profile = typeId ? this.ruleset?.ruleset.type_profiles[typeId] : undefined;
    return mandatoryTraitRefs(profile);
  }

  /**
   * Select the Hermetic House (or clear it with `null`). A discrete action, so
   * it validates immediately like {@link setType}. Switching House drops any
   * specialisation picks whose `choice_key` the new House no longer defines, so
   * a stale pick from the previous House can't linger in the save; clearing the
   * House drops them all.
   */
  async setHouse(house: string | null): Promise<void> {
    if ((this.entity.house ?? null) === house) return;
    this.entity.house = house;
    this.entity.house_choices = this.#prunedHouseChoices(house);
    await this.revalidate();
  }

  /**
   * Set the specialisation pick for one of the current House's grants, keyed by
   * the grant's `choice_key` (a menu option for a `choice` grant, or a chosen
   * Virtue/Flaw for an `open` one). Debounced like the other picker edits.
   */
  setHouseChoice(choiceKey: string, selection: Selection): void {
    this.entity.house_choices = { ...(this.entity.house_choices ?? {}), [choiceKey]: selection };
    this.#scheduleValidate();
  }

  /** The `choice_key`s the given House's `choice`/`open` grants define. */
  #houseChoiceKeys(house: string | null): Set<string> {
    const keys = new Set<string>();
    if (!house) return keys;
    for (const grant of this.ruleset?.ruleset.houses?.[house]?.grants ?? []) {
      if (grant.kind === 'choice' || grant.kind === 'open') keys.add(grant.choice_key);
    }
    return keys;
  }

  /** Existing picks kept only where the target House still defines their key. */
  #prunedHouseChoices(house: string | null): Record<string, Selection> {
    const valid = this.#houseChoiceKeys(house);
    const kept: Record<string, Selection> = {};
    for (const [key, pick] of Object.entries(this.entity.house_choices ?? {})) {
      if (valid.has(key)) kept[key] = pick;
    }
    return kept;
  }

  /**
   * Select the Mythic Companion type (or clear it with `null`). A discrete
   * action, so it validates immediately. Switching auto-manages the type's
   * required package: it removes the previous type's seeded required Virtues/
   * Flaws that the new type doesn't require, then seeds the new type's package
   * (its required Virtues + each required Flaw's rules default) as ordinary
   * budgeted selections — so a direct-entry mythic companion starts legal, with
   * the required Flaws swappable via {@link setMythicRequiredFlaw}. The free
   * status/Minor Virtue are point-free grants derived engine-side (never in
   * `selections`); a `choice` free-Minor (Devil Child's Might/Powers) defaults to
   * its first option. Mirrors {@link setHouse} + {@link setType}.
   */
  async setMythicType(mythicType: string | null): Promise<void> {
    if ((this.entity.mythic_type ?? null) === mythicType) return;
    const previousPackage = this.#mythicPackage(this.entity.mythic_type ?? null);
    const nextPackage = this.#mythicPackage(mythicType);
    // Drop the previous type's seeded package rows the new type doesn't require.
    this.entity.selections = this.entity.selections.filter(
      (s) =>
        !(
          previousPackage.some((p) => sameSelection(p, s)) &&
          !nextPackage.some((p) => sameSelection(p, s))
        ),
    );
    this.entity.mythic_type = mythicType;
    this.entity.mythic_choices = this.#defaultedMythicChoices(mythicType);
    // Seed the new type's required package (budgeted) where not already present.
    for (const pkg of nextPackage) {
      if (!this.entity.selections.some((s) => sameSelection(s, pkg))) {
        this.entity.selections.push(pkg);
      }
    }
    await this.revalidate();
  }

  /**
   * Set a Mythic Companion type grant pick keyed by the grant's `choice_key`
   * (e.g. Devil Child's Demonic Might-or-Powers free Minor). Debounced like the
   * other picker edits. Mirrors {@link setHouseChoice}.
   */
  setMythicChoice(choiceKey: string, selection: Selection): void {
    this.entity.mythic_choices = {
      ...(this.entity.mythic_choices ?? {}),
      [choiceKey]: selection,
    };
    this.#scheduleValidate();
  }

  /**
   * Swap a required Flaw for a "suitable substitute agreed with the troupe":
   * removes the currently-selected required Flaw (`previousRef`) and adds the
   * chosen substitute (`nextRef`) as a budgeted selection. A discrete dropdown
   * action, so it validates immediately.
   */
  async setMythicRequiredFlaw(previousRef: string, nextRef: string): Promise<void> {
    if (previousRef === nextRef) return;
    const idx = this.entity.selections.findIndex((s) => s.ref === previousRef);
    if (idx >= 0) this.entity.selections.splice(idx, 1);
    if (!this.entity.selections.some((s) => s.ref === nextRef)) {
      this.entity.selections.push({ ref: nextRef });
    }
    this.entity.selections = [...this.entity.selections];
    await this.revalidate();
  }

  /** The budgeted required package (required Virtues + each Flaw's default). */
  #mythicPackage(mythicType: string | null): Selection[] {
    if (!mythicType) return [];
    const t = this.ruleset?.ruleset.mythic_companion_types?.[mythicType];
    if (!t) return [];
    return [...(t.required_virtues ?? []), ...(t.required_flaws ?? []).map((f) => f.default)];
  }

  /**
   * Mythic-type grant picks kept where the target type still defines their key,
   * with each `choice` grant defaulted to its first option so the free Minor
   * Virtue is granted without an extra step.
   */
  #defaultedMythicChoices(mythicType: string | null): Record<string, Selection> {
    const grants = mythicType
      ? (this.ruleset?.ruleset.mythic_companion_types?.[mythicType]?.grants ?? [])
      : [];
    const validKeys = new Set(
      grants.filter((g) => g.kind === 'choice' || g.kind === 'open').map((g) => g.choice_key),
    );
    const kept: Record<string, Selection> = {};
    for (const [key, pick] of Object.entries(this.entity.mythic_choices ?? {})) {
      if (validKeys.has(key)) kept[key] = pick;
    }
    for (const g of grants) {
      if (g.kind === 'choice' && !kept[g.choice_key]) kept[g.choice_key] = g.options[0];
    }
    return kept;
  }

  /**
   * Add a virtue/flaw selection. A repeatable item — one carrying parameters
   * (e.g. Great Characteristic) or with `max_per_target > 1` — can be added
   * several times, each instance choosing its own target; a plain item is added
   * once. Mirrors {@link addAbility}.
   */
  addSelection(ref: string): void {
    const item = this.ruleset?.ruleset.point_items[ref];
    const repeatable = !!item?.parameters?.length || (item?.max_per_target ?? 1) > 1;
    const present = this.entity.selections.some((s) => s.ref === ref);
    if (!repeatable && present) return;
    this.entity.selections.push({ ref });
    this.#scheduleValidate();
  }

  /** Selection edits are by row index, since a repeatable item has several rows. */
  removeSelectionAt(index: number): void {
    this.entity.selections = this.entity.selections.filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  setParamAt(index: number, key: string, value: string): void {
    this.entity.selections = this.entity.selections.map((s, i) =>
      i === index ? { ...s, params: { ...(s.params ?? {}), [key]: value } } : s,
    );
    this.#scheduleValidate();
  }

  /**
   * Point an ability-bonus selection (Puissant Ability) at a specific ability
   * *instance*. The ability id goes under the `ability` param; for a
   * parameterized ability the instance value (the area/language) goes under the
   * ability's own param key (so the bonus attaches to that one row). Switching to
   * a plain ability drops any stale instance key.
   */
  setAbilityBonusTarget(index: number, abilityId: string, parameter?: string | null): void {
    const instanceKey = this.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
    const params: Record<string, string> = { ability: abilityId };
    if (instanceKey && parameter) params[instanceKey] = parameter;
    this.entity.selections = this.entity.selections.map((s, i) =>
      i === index ? { ...s, params } : s,
    );
    this.#scheduleValidate();
  }

  /** Set or clear a Characteristic score (score 0 removes the explicit entry). */
  setCharacteristic(characteristic: Characteristic, score: number): void {
    const chars = { ...(this.entity.characteristics ?? {}) } as Record<Characteristic, number>;
    const value = clampInt(score, I8_MIN, I8_MAX);
    if (value === 0) {
      delete chars[characteristic];
    } else {
      chars[characteristic] = value;
    }
    this.entity.characteristics = chars;
    this.#scheduleValidate();
  }

  /** Set or clear a Characteristic's free-text description (the sheet's flavor). */
  setCharacteristicDescription(characteristic: Characteristic, text: string): void {
    const descriptions = { ...(this.entity.characteristic_descriptions ?? {}) };
    if (text.trim()) {
      descriptions[characteristic] = text;
    } else {
      delete descriptions[characteristic];
    }
    this.entity.characteristic_descriptions = descriptions;
    this.#scheduleValidate();
  }

  /**
   * Select an ability (like a virtue/flaw): it enters at score 0, which costs no
   * XP — the first point is bought by raising it. A parameterized ability (e.g.
   * (Area) Lore) can be added several times (each instance gets its own value); a
   * plain ability is added once.
   */
  addAbility(ability: string): void {
    const parameterized = !!this.ruleset?.ruleset.abilities?.[ability]?.parameter;
    const present = (this.entity.ability_scores ?? []).some((a) => a.ability === ability);
    if (!parameterized && present) return;
    this.entity.ability_scores = [...(this.entity.ability_scores ?? []), { ability, score: 0 }];
    this.#scheduleValidate();
  }

  /** Ability edits are by row index, since a parameterized ability has several rows. */
  removeAbilityAt(index: number): void {
    this.entity.ability_scores = (this.entity.ability_scores ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  adjustAbilityAt(index: number, delta: number, max: number): void {
    this.entity.ability_scores = (this.entity.ability_scores ?? []).map((a, i) =>
      i === index ? { ...a, score: Math.max(0, Math.min(max, a.score + delta)) } : a,
    );
    this.#scheduleValidate();
  }

  setAbilitySpecialtyAt(index: number, specialty: string): void {
    const spec = specialty.trim() ? specialty.trim() : undefined;
    this.entity.ability_scores = (this.entity.ability_scores ?? []).map((a, i) =>
      i === index ? { ...a, specialty: spec } : a,
    );
    this.#scheduleValidate();
  }

  setAbilityParameterAt(index: number, value: string): void {
    const param = value.trim() ? value.trim() : undefined;
    this.entity.ability_scores = (this.entity.ability_scores ?? []).map((a, i) =>
      i === index ? { ...a, parameter: param } : a,
    );
    this.#scheduleValidate();
  }

  setXpPool(xp: number): void {
    this.entity.xp_pool = clampInt(xp, 0, U32_MAX);
    this.#scheduleValidate();
  }

  /**
   * Adjust an Art's bought score by `delta`, clamped to [0, max]. All 15 Arts are
   * always present for a magus, so an Art is addressed by id (not a row index)
   * and upserted: the score is stored only while non-zero (score 0 is the default
   * and is dropped to keep saves sparse and canonical).
   */
  adjustArt(art: string, delta: number, max: number): void {
    const scores = this.entity.art_scores ?? [];
    const current = scores.find((a) => a.art === art)?.score ?? 0;
    const next = Math.max(0, Math.min(max, current + delta));
    if (next === 0) {
      this.entity.art_scores = scores.filter((a) => a.art !== art);
    } else if (scores.some((a) => a.art === art)) {
      this.entity.art_scores = scores.map((a) => (a.art === art ? { ...a, score: next } : a));
    } else {
      this.entity.art_scores = [...scores, { art, score: next }];
    }
    this.#scheduleValidate();
  }

  /** Point an art-bonus selection (Puissant Art) at a specific Art (by id). */
  setArtBonusTarget(index: number, artId: string): void {
    this.entity.selections = this.entity.selections.map((s, i) =>
      i === index ? { ...s, params: { art: artId } } : s,
    );
    this.#scheduleValidate();
  }

  /**
   * Add a spell to the magus's list. `level` is passed only for a General spell
   * (the chosen level); a fixed spell derives its level from the catalogue.
   * `parameter` names the target Form of a parametrized meta-magic Vim spell.
   *
   * Identity is (spell, level, parameter): the same base spell may be taken once
   * per distinct Form. A parametrized spell adds a fresh row each time (its Form
   * is chosen afterwards in the selected list, mirroring parametrized abilities),
   * so it is never blocked at add time and its source row never greys just
   * because one Form instance exists; exact (spell, level, Form) duplicates are
   * flagged by the engine's dedupe. A plain spell is added once per (level).
   */
  addSpell(spellId: string, level?: number | null, parameter?: string | null): void {
    const lvl = typeof level === 'number' ? level : undefined;
    const param = parameter ?? undefined;
    const parameterized = (this.ruleset?.ruleset.spells?.[spellId]?.parameters?.length ?? 0) > 0;
    if (!parameterized) {
      const present = (this.entity.spells ?? []).some(
        (s) =>
          s.spell === spellId &&
          (s.level ?? undefined) === lvl &&
          (s.parameter ?? undefined) === param,
      );
      if (present) return;
    }
    this.entity.spells = [
      ...(this.entity.spells ?? []),
      {
        spell: spellId,
        ...(lvl === undefined ? {} : { level: lvl }),
        ...(param === undefined ? {} : { parameter: param }),
      },
    ];
    this.#scheduleValidate();
  }

  /**
   * Set (or clear) the target Form of a parametrized spell at `index` — part of
   * the spell's identity, so distinct Forms are distinct instances. The chosen
   * value is an Art id (e.g. `art.ignem`). Mirrors {@link setAbilityParameterAt}.
   */
  setSpellParameterAt(index: number, parameter: string | null): void {
    const param = parameter && parameter.trim() ? parameter.trim() : undefined;
    this.entity.spells = (this.entity.spells ?? []).map((s, i) =>
      i === index ? { ...s, parameter: param } : s,
    );
    this.#scheduleValidate();
  }

  /**
   * Adjust the bought Spell Mastery score of the spell at `index` by `delta`,
   * clamped to [0, max]. Spent from the restricted Spell-Mastery XP pool; the
   * granted floor (Flawless Magic) is applied on top when computing the effective
   * mastery, so it is not stored here. Mirrors {@link adjustAbilityAt}.
   */
  adjustSpellMasteryAt(index: number, delta: number, max: number): void {
    this.entity.spells = (this.entity.spells ?? []).map((s, i) =>
      i === index ? { ...s, mastery: Math.max(0, Math.min(max, (s.mastery ?? 0) + delta)) } : s,
    );
    this.#scheduleValidate();
  }

  /**
   * Add a Spell Mastery special ability (a `spell_mastery_ability.*` id) to the
   * spell at `index`. A repeatable ability (Precise/Quick/Quiet Casting) may be
   * added more than once; the count cap vs. effective mastery is enforced by the
   * engine, not here. Mirrors {@link adjustSpellMasteryAt}.
   */
  addMasteryAbilityAt(index: number, abilityId: string): void {
    this.entity.spells = (this.entity.spells ?? []).map((s, i) =>
      i === index ? { ...s, mastery_abilities: [...(s.mastery_abilities ?? []), abilityId] } : s,
    );
    this.#scheduleValidate();
  }

  /**
   * Remove the mastery special ability at position `abilityIndex` within the
   * spell at `index`. Index-addressed so a repeatable ability chosen several
   * times removes exactly one instance. Mirrors {@link addMasteryAbilityAt}.
   */
  removeMasteryAbilityAt(index: number, abilityIndex: number): void {
    this.entity.spells = (this.entity.spells ?? []).map((s, i) =>
      i === index
        ? {
            ...s,
            mastery_abilities: (s.mastery_abilities ?? []).filter((_, j) => j !== abilityIndex),
          }
        : s,
    );
    this.#scheduleValidate();
  }

  /**
   * Set the level of the (General) spell at `index`. The budget/used totals are
   * engine-authoritative, so no recompute happens here. Mirrors
   * {@link adjustSpellMasteryAt}.
   */
  setSpellLevelAt(index: number, level: number): void {
    // `SpellSelection.level` is the entity's narrowest number (u8), and a General
    // spell has no level 0, so the floor is 1 — matching the input's `min`.
    const clamped = clampInt(level, 1, U8_MAX);
    this.entity.spells = (this.entity.spells ?? []).map((s, i) =>
      i === index ? { ...s, level: clamped } : s,
    );
    this.#scheduleValidate();
  }

  /** Spell edits are by row index, since a General spell can appear at several levels. */
  removeSpellAt(index: number): void {
    this.entity.spells = (this.entity.spells ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  /**
   * Set (or clear) the per-character spell-levels budget override. A non-positive
   * or non-finite value clears it (`null`), so the engine falls back to the type
   * profile's base. The budget/used totals stay engine-authoritative — no
   * recompute happens here. Mirrors {@link setAge}.
   */
  setSpellLevelsOverride(levels: number | null): void {
    this.entity.spell_levels_override =
      levels != null && Number.isFinite(levels) && levels > 0 ? clampInt(levels, 1, U32_MAX) : null;
    this.#scheduleValidate();
  }

  /** Set (or clear) the character's age; drives the age → Ability-cap check. */
  setAge(age: number | null): void {
    this.entity.age =
      age != null && Number.isFinite(age) && age > 0 ? clampInt(age, 1, U32_MAX) : null;
    this.#scheduleValidate();
  }

  /** Set (or clear) the character's apparent age (annotation; no mechanic). */
  setApparentAge(age: number | null): void {
    this.entity.apparent_age =
      age != null && Number.isFinite(age) && age > 0 ? clampInt(age, 1, U32_MAX) : null;
    this.#scheduleValidate();
  }

  addPersonalityTrait(): void {
    this.entity.personality_traits = [
      ...(this.entity.personality_traits ?? []),
      { name: '', value: 0 },
    ];
    this.#scheduleValidate();
  }

  removePersonalityTraitAt(index: number): void {
    this.entity.personality_traits = (this.entity.personality_traits ?? []).filter(
      (_, i) => i !== index,
    );
    this.#scheduleValidate();
  }

  setPersonalityTraitName(index: number, name: string): void {
    this.entity.personality_traits = (this.entity.personality_traits ?? []).map((t, i) =>
      i === index ? { ...t, name } : t,
    );
    this.#scheduleValidate();
  }

  setPersonalityTraitValue(index: number, value: number): void {
    const clamped = Math.max(-6, Math.min(6, Math.trunc(value)));
    this.entity.personality_traits = (this.entity.personality_traits ?? []).map((t, i) =>
      i === index ? { ...t, value: clamped } : t,
    );
    this.#scheduleValidate();
  }

  /** Add a Reputation from a granting V/F (kind + score come from the grant). */
  addReputation(kind: ReputationType, score: number): void {
    this.entity.reputations = [...(this.entity.reputations ?? []), { kind, score, content: '' }];
    this.#scheduleValidate();
  }

  removeReputationAt(index: number): void {
    this.entity.reputations = (this.entity.reputations ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  setReputationContent(index: number, content: string): void {
    this.entity.reputations = (this.entity.reputations ?? []).map((r, i) =>
      i === index ? { ...r, content } : r,
    );
    this.#scheduleValidate();
  }

  // --- Magic Items tab: aura, devices, familiar, talisman, longevity ---

  /** Set the realm aura modifier (signed; Divine can be a penalty). */
  setAura(aura: number | null): void {
    this.entity.aura = aura == null ? 0 : clampInt(aura, I32_MIN, I32_MAX);
    this.#scheduleValidate();
  }

  addDevice(): void {
    this.entity.devices = [...(this.entity.devices ?? []), { name: '', level: 0 }];
    this.#scheduleValidate();
  }

  removeDeviceAt(index: number): void {
    this.entity.devices = (this.entity.devices ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  setDeviceName(index: number, name: string): void {
    this.entity.devices = (this.entity.devices ?? []).map((d, i) =>
      i === index ? { ...d, name } : d,
    );
    this.#scheduleValidate();
  }

  setDeviceLevel(index: number, level: number): void {
    const clamped = clampInt(level, 0, U16_MAX);
    this.entity.devices = (this.entity.devices ?? []).map((d, i) =>
      i === index ? { ...d, level: clamped } : d,
    );
    this.#scheduleValidate();
  }

  // --- Supernatural being: Might Score + Realm, powers (Devil Child / Nephilim) ---

  /** Set the being's Might Realm (creating the base Might at score 0 if absent). */
  setMightRealm(realm: Realm): void {
    const score = this.entity.might?.score ?? 0;
    this.entity.might = { realm, score };
    this.#scheduleValidate();
  }

  /** Set the being's base Might Score (non-negative; keeps the current Realm). */
  setMightScore(score: number): void {
    const realm = this.entity.might?.realm ?? 'magic';
    this.entity.might = { realm, score: clampInt(score, 0, U8_MAX) };
    this.#scheduleValidate();
  }

  /** Remove the being's base Might (Virtue-granted Might is unaffected). */
  clearMight(): void {
    this.entity.might = null;
    this.#scheduleValidate();
  }

  addPower(): void {
    this.entity.powers = [...(this.entity.powers ?? []), { name: '', level: 0 }];
    this.#scheduleValidate();
  }

  removePowerAt(index: number): void {
    this.entity.powers = (this.entity.powers ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  setPowerName(index: number, name: string): void {
    this.entity.powers = (this.entity.powers ?? []).map((p, i) =>
      i === index ? { ...p, name } : p,
    );
    this.#scheduleValidate();
  }

  setPowerLevel(index: number, level: number): void {
    const clamped = clampInt(level, 0, U16_MAX);
    this.entity.powers = (this.entity.powers ?? []).map((p, i) =>
      i === index ? { ...p, level: clamped } : p,
    );
    this.#scheduleValidate();
  }

  // --- Familiar: the creature statblock + the three bond cords ---
  //
  // Every edit below is guarded by `if (!this.entity.familiar) return;`: the panel
  // only renders these controls once a familiar exists, but a mutator must not
  // conjure one out of a stray call. Nothing here touches the character's own
  // `might`, `powers`, `characteristics` or `personality_traits` — the familiar is
  // a separate creature.

  /** Add the magus's familiar (at most one) with the full empty statblock. */
  addFamiliar(): void {
    this.entity.familiar = {
      name: '',
      animal: '',
      might: null,
      characteristics: {},
      size: 0,
      personality_traits: [],
      cord_gold: 0,
      cord_silver: 0,
      cord_bronze: 0,
      powers: [],
    };
    this.#scheduleValidate();
  }

  removeFamiliar(): void {
    this.entity.familiar = null;
    this.#scheduleValidate();
  }

  setFamiliarName(name: string): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = { ...this.entity.familiar, name };
    this.#scheduleValidate();
  }

  /** Set the kind of beast (free text; not `species`, which is the Imaginem term). */
  setFamiliarAnimal(animal: string): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = { ...this.entity.familiar, animal };
    this.#scheduleValidate();
  }

  /** Set the creature's Size — signed and deliberately NOT clamped at 0: most
   * familiars are smaller than a human and so carry a negative Size. */
  setFamiliarSize(size: number): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = { ...this.entity.familiar, size: clampInt(size, I8_MIN, I8_MAX) };
    this.#scheduleValidate();
  }

  /** Set the familiar's own Might Realm (creating its Might at score 0 if absent). */
  setFamiliarMightRealm(realm: Realm): void {
    if (!this.entity.familiar) return;
    const score = this.entity.familiar.might?.score ?? 0;
    this.entity.familiar = { ...this.entity.familiar, might: { realm, score } };
    this.#scheduleValidate();
  }

  /** Set the familiar's own Might Score (non-negative; keeps the current Realm). */
  setFamiliarMightScore(score: number): void {
    if (!this.entity.familiar) return;
    const realm = this.entity.familiar.might?.realm ?? 'magic';
    this.entity.familiar = {
      ...this.entity.familiar,
      might: { realm, score: clampInt(score, 0, U8_MAX) },
    };
    this.#scheduleValidate();
  }

  clearFamiliarMight(): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = { ...this.entity.familiar, might: null };
    this.#scheduleValidate();
  }

  /** Set one of the familiar's Characteristics, deleting the entry at 0 (mirrors
   * `setCharacteristic`, so a default score writes no key). */
  setFamiliarCharacteristic(characteristic: Characteristic, score: number): void {
    if (!this.entity.familiar) return;
    const chars = { ...(this.entity.familiar.characteristics ?? {}) };
    const value = clampInt(score, I8_MIN, I8_MAX);
    if (value === 0) {
      delete chars[characteristic];
    } else {
      chars[characteristic] = value;
    }
    this.entity.familiar = { ...this.entity.familiar, characteristics: chars };
    this.#scheduleValidate();
  }

  /** Set one bond cord's score, bounded by the rule (0..+5), not by its u8 width:
   * the engine's cord consumers only agree on values the rules allow. */
  setFamiliarCord(cord: 'gold' | 'silver' | 'bronze', value: number): void {
    if (!this.entity.familiar) return;
    const clamped = clampInt(value, 0, CORD_MAX);
    const key = `cord_${cord}` as const;
    this.entity.familiar = { ...this.entity.familiar, [key]: clamped };
    this.#scheduleValidate();
  }

  addFamiliarPersonalityTrait(): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = {
      ...this.entity.familiar,
      personality_traits: [
        ...(this.entity.familiar.personality_traits ?? []),
        { name: '', value: 0 },
      ],
    };
    this.#scheduleValidate();
  }

  removeFamiliarPersonalityTraitAt(index: number): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = {
      ...this.entity.familiar,
      personality_traits: (this.entity.familiar.personality_traits ?? []).filter(
        (_, i) => i !== index,
      ),
    };
    this.#scheduleValidate();
  }

  setFamiliarPersonalityTraitName(index: number, name: string): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = {
      ...this.entity.familiar,
      personality_traits: (this.entity.familiar.personality_traits ?? []).map((t, i) =>
        i === index ? { ...t, name } : t,
      ),
    };
    this.#scheduleValidate();
  }

  setFamiliarPersonalityTraitValue(index: number, value: number): void {
    if (!this.entity.familiar) return;
    const clamped = Math.max(-6, Math.min(6, Math.trunc(value)));
    this.entity.familiar = {
      ...this.entity.familiar,
      personality_traits: (this.entity.familiar.personality_traits ?? []).map((t, i) =>
        i === index ? { ...t, value: clamped } : t,
      ),
    };
    this.#scheduleValidate();
  }

  addFamiliarPower(): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = {
      ...this.entity.familiar,
      powers: [...(this.entity.familiar.powers ?? []), { name: '', level: 0 }],
    };
    this.#scheduleValidate();
  }

  removeFamiliarPowerAt(index: number): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = {
      ...this.entity.familiar,
      powers: (this.entity.familiar.powers ?? []).filter((_, i) => i !== index),
    };
    this.#scheduleValidate();
  }

  setFamiliarPowerName(index: number, name: string): void {
    if (!this.entity.familiar) return;
    this.entity.familiar = {
      ...this.entity.familiar,
      powers: (this.entity.familiar.powers ?? []).map((p, i) => (i === index ? { ...p, name } : p)),
    };
    this.#scheduleValidate();
  }

  setFamiliarPowerLevel(index: number, level: number): void {
    if (!this.entity.familiar) return;
    const clamped = clampInt(level, 0, U16_MAX);
    this.entity.familiar = {
      ...this.entity.familiar,
      powers: (this.entity.familiar.powers ?? []).map((p, i) =>
        i === index ? { ...p, level: clamped } : p,
      ),
    };
    this.#scheduleValidate();
  }

  /** Add the magus's talisman (at most one) with nothing filled in yet. */
  addTalisman(): void {
    this.entity.talisman = { description: '', attunements: [], effects: [] };
    this.#scheduleValidate();
  }

  removeTalisman(): void {
    this.entity.talisman = null;
    this.#scheduleValidate();
  }

  /** Set the item's shape/material identity (not an attunement's descriptor). */
  setTalismanDescription(description: string): void {
    if (!this.entity.talisman) return;
    this.entity.talisman = { ...this.entity.talisman, description };
    this.#scheduleValidate();
  }

  addTalismanAttunement(): void {
    if (!this.entity.talisman) return;
    this.entity.talisman = {
      ...this.entity.talisman,
      attunements: [...(this.entity.talisman.attunements ?? []), { description: '', bonus: 0 }],
    };
    this.#scheduleValidate();
  }

  removeTalismanAttunementAt(index: number): void {
    if (!this.entity.talisman) return;
    this.entity.talisman = {
      ...this.entity.talisman,
      attunements: (this.entity.talisman.attunements ?? []).filter((_, i) => i !== index),
    };
    this.#scheduleValidate();
  }

  setTalismanAttunementDescription(index: number, description: string): void {
    if (!this.entity.talisman) return;
    this.entity.talisman = {
      ...this.entity.talisman,
      attunements: (this.entity.talisman.attunements ?? []).map((a, i) =>
        i === index ? { ...a, description } : a,
      ),
    };
    this.#scheduleValidate();
  }

  setTalismanAttunementBonus(index: number, bonus: number): void {
    if (!this.entity.talisman) return;
    this.entity.talisman = {
      ...this.entity.talisman,
      attunements: (this.entity.talisman.attunements ?? []).map((a, i) =>
        i === index ? { ...a, bonus: clampInt(bonus, I8_MIN, I8_MAX) } : a,
      ),
    };
    this.#scheduleValidate();
  }

  addTalismanEffect(): void {
    if (!this.entity.talisman) return;
    this.entity.talisman = {
      ...this.entity.talisman,
      effects: [...(this.entity.talisman.effects ?? []), { name: '', level: 0 }],
    };
    this.#scheduleValidate();
  }

  removeTalismanEffectAt(index: number): void {
    if (!this.entity.talisman) return;
    this.entity.talisman = {
      ...this.entity.talisman,
      effects: (this.entity.talisman.effects ?? []).filter((_, i) => i !== index),
    };
    this.#scheduleValidate();
  }

  setTalismanEffectName(index: number, name: string): void {
    if (!this.entity.talisman) return;
    this.entity.talisman = {
      ...this.entity.talisman,
      effects: (this.entity.talisman.effects ?? []).map((e, i) =>
        i === index ? { ...e, name } : e,
      ),
    };
    this.#scheduleValidate();
  }

  setTalismanEffectLevel(index: number, level: number): void {
    if (!this.entity.talisman) return;
    const clamped = clampInt(level, 0, U16_MAX);
    this.entity.talisman = {
      ...this.entity.talisman,
      effects: (this.entity.talisman.effects ?? []).map((e, i) =>
        i === index ? { ...e, level: clamped } : e,
      ),
    };
    this.#scheduleValidate();
  }

  /** Add a Longevity Ritual with nothing entered yet (null bonus ≠ a claimed 0). */
  addLongevityRitual(source: LongevitySource): void {
    this.entity.longevity_ritual = { source, bonus: null, focus: '' };
    this.#scheduleValidate();
  }

  removeLongevityRitual(): void {
    this.entity.longevity_ritual = null;
    this.#scheduleValidate();
  }

  /**
   * Switch the ritual source. The entered bonus is *preserved*: it describes the
   * ritual the magus actually has, and who made it does not change its value.
   */
  setLongevitySource(source: LongevitySource): void {
    if (!this.entity.longevity_ritual) return;
    this.entity.longevity_ritual = { ...this.entity.longevity_ritual, source };
    this.#scheduleValidate();
  }

  /**
   * Store the player-entered aging bonus, or `null` for "not entered".
   *
   * Clearing the input must reach here as `null`, never as the 0 that
   * `Number('')` yields: a stored 0 is a deliberate claim ("this ritual grants
   * nothing"), and writing one would drop the not-entered marker for good — the
   * exact state every pre-5.5a save loads in. A non-finite value (mid-typing a
   * lone "-") is likewise "not entered", not a stored 0.
   */
  setLongevityBonus(bonus: number | null): void {
    if (!this.entity.longevity_ritual) return;
    const entered = bonus != null && Number.isFinite(bonus);
    this.entity.longevity_ritual = {
      ...this.entity.longevity_ritual,
      bonus: entered ? clampInt(bonus, I8_MIN, I8_MAX) : null,
    };
    this.#scheduleValidate();
  }

  setLongevityFocus(focus: string): void {
    if (!this.entity.longevity_ritual) return;
    this.entity.longevity_ritual = { ...this.entity.longevity_ritual, focus };
    this.#scheduleValidate();
  }

  // --- Aged / warped state + identity (directly-entered; effects computed by
  // the engine, never here) ---

  /** Set accrued aging points for a Characteristic (0 removes the entry). */
  setAgingPoints(characteristic: Characteristic, points: number): void {
    this.entity.aging_points = this.#withCharCount(
      this.entity.aging_points,
      characteristic,
      points,
    );
    this.#scheduleValidate();
  }

  /** Shared helper: set a per-Characteristic count in u8 range, pruning zeros. */
  #withCharCount(
    map: Partial<Record<Characteristic, number>> | undefined,
    characteristic: Characteristic,
    value: number,
  ): Partial<Record<Characteristic, number>> {
    const next = { ...(map ?? {}) };
    const clamped = clampInt(value, 0, U8_MAX);
    if (clamped === 0) {
      delete next[characteristic];
    } else {
      next[characteristic] = clamped;
    }
    return next;
  }

  /** Set accrued Warping Points (the engine derives the Warping Score). */
  setWarpingPoints(points: number): void {
    this.entity.warping_points = clampInt(points, 0, U32_MAX);
    this.#scheduleValidate();
  }

  /**
   * Choose the specific Virtue/Flaw filling one owed Warping slot, keyed by the
   * owed grant's `choice_key` (from `effective.warping_owed_grants`). Off-budget:
   * the fill resolves to a derived selection engine-side and never touches the
   * V/F budget. Passing `null` clears the slot. Debounced like the other picker
   * edits. Mirrors {@link setHouseChoice}.
   */
  setWarpingChoice(choiceKey: string, selection: Selection | null): void {
    const next = { ...(this.entity.warping_choices ?? {}) };
    if (selection) {
      next[choiceKey] = selection;
    } else {
      delete next[choiceKey];
    }
    this.entity.warping_choices = next;
    this.#scheduleValidate();
  }

  addTwilightScar(): void {
    this.entity.twilight_scars = [...(this.entity.twilight_scars ?? []), { description: '' }];
    this.#scheduleValidate();
  }

  removeTwilightScarAt(index: number): void {
    this.entity.twilight_scars = (this.entity.twilight_scars ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  setTwilightScarDescription(index: number, description: string): void {
    this.entity.twilight_scars = (this.entity.twilight_scars ?? []).map((s, i) =>
      i === index ? { ...s, description } : s,
    );
    this.#scheduleValidate();
  }

  /** Set the free-text description of how the character's Warping manifests. */
  setWarpingEffect(effect: string): void {
    this.entity.warping_effect = effect;
    this.#scheduleValidate();
  }

  /** Set the free-text narrative of the character's overall aging/decrepitude. */
  setDecrepitudeEffect(effect: string): void {
    this.entity.decrepitude_effect = effect;
    this.#scheduleValidate();
  }

  addAgingLogEntry(): void {
    this.entity.aging_log = [...(this.entity.aging_log ?? []), { year: 0, effect: '' }];
    this.#scheduleValidate();
  }

  removeAgingLogEntryAt(index: number): void {
    this.entity.aging_log = (this.entity.aging_log ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  setAgingLogEntryYear(index: number, year: number): void {
    this.entity.aging_log = (this.entity.aging_log ?? []).map((e, i) =>
      i === index ? { ...e, year: clampInt(year, I32_MIN, I32_MAX) } : e,
    );
    this.#scheduleValidate();
  }

  setAgingLogEntryEffect(index: number, effect: string): void {
    this.entity.aging_log = (this.entity.aging_log ?? []).map((e, i) =>
      i === index ? { ...e, effect } : e,
    );
    this.#scheduleValidate();
  }

  /** Add a carried equipment slot referencing a catalogue weapon/shield/armor id. */
  addEquipment(item: string): void {
    if (!item) return;
    this.entity.equipment = [...(this.entity.equipment ?? []), { item, equipped: true }];
    this.#scheduleValidate();
  }

  removeEquipmentAt(index: number): void {
    this.entity.equipment = (this.entity.equipment ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  setEquipmentEquipped(index: number, equipped: boolean): void {
    this.entity.equipment = (this.entity.equipment ?? []).map((slot, i) =>
      i === index ? { ...slot, equipped } : slot,
    );
    this.#scheduleValidate();
  }

  /** Toggle whether the weapon's Ability specialization applies (+1 Atk/Def). */
  setEquipmentSpecialization(index: number, specialization_applies: boolean): void {
    this.entity.equipment = (this.entity.equipment ?? []).map((slot, i) =>
      i === index ? { ...slot, specialization_applies } : slot,
    );
    this.#scheduleValidate();
  }

  /** Set a free-text identity/flavor field (no mechanical effect). */
  setIdentity(
    field: 'name' | 'description' | 'concept' | 'gender' | 'sigil' | 'covenant_name' | 'parens',
    value: string,
  ): void {
    this.entity[field] = value;
    this.#scheduleValidate();
  }

  setBirthYear(year: number | null): void {
    this.entity.birth_year =
      year != null && Number.isFinite(year) ? clampInt(year, I32_MIN, I32_MAX) : null;
    this.#scheduleValidate();
  }

  /**
   * Save to the current file. A never-saved document (no {@link currentPath})
   * falls back to {@link saveAs} so the user picks a destination; otherwise it
   * writes straight to the tracked file with no prompt (standard document-app
   * behavior). No-op while another file operation is in flight.
   */
  async save(): Promise<void> {
    if (this.#opInFlight) return;
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
    if (this.#opInFlight) return;
    await this.#writeTo(null);
  }

  /**
   * Shared write path. `path === null` prompts (Save As / first Save); a concrete
   * path writes directly. On success clears dirty and records the written path as
   * the current file. The baseline is captured BEFORE awaiting, so edits made
   * while a dialog is open stay marked dirty.
   */
  async #writeTo(path: string | null): Promise<void> {
    this.#opInFlight = true;
    this.error = null;
    const snapshot = this.#snapshot();
    try {
      const written = await ipc.saveEntity($state.snapshot(this.entity), path);
      // A null return means the dialog was cancelled — nothing was written.
      if (written !== null) {
        this.currentPath = written;
        this.#savedSnapshot = snapshot;
      }
    } catch (e) {
      this.error = e as AppError;
    } finally {
      this.#opInFlight = false;
    }
  }

  /**
   * Open a document from a file. Prompts to discard first when the current
   * document has unsaved edits; a cancelled prompt aborts without loading. On
   * success the opened file becomes the current file. No-op while another file
   * operation is in flight.
   */
  async open(): Promise<void> {
    if (this.#opInFlight || this.discardPromptOpen) return;
    if (this.dirty && !(await this.#confirmDiscard())) return;
    this.#opInFlight = true;
    this.error = null;
    try {
      const loaded = await ipc.loadEntity();
      if (loaded) {
        this.entity = loaded.entity;
        this.currentPath = loaded.path;
        this.#savedSnapshot = this.#snapshot();
        await this.revalidate();
      }
    } catch (e) {
      this.error = e as AppError;
    } finally {
      this.#opInFlight = false;
    }
  }

  /**
   * Reset to a fresh, empty document. Prompts to discard first when the current
   * document has unsaved edits; a cancelled prompt aborts. Clears the current
   * file, the picker filters, and the saved baseline.
   */
  async newDocument(): Promise<void> {
    if (this.#opInFlight || this.discardPromptOpen) return;
    if (this.dirty && !(await this.#confirmDiscard())) return;
    const { id, version } = this.ruleset?.ruleset ?? this.entity.ruleset;
    this.entity = newEntity(id, version);
    this.currentPath = null;
    this.filters = defaultPickerFilters();
    this.result = null;
    this.effective = null;
    this.derived = null;
    this.#savedSnapshot = this.#snapshot();
    await this.revalidate();
  }

  /**
   * Show the discard-changes prompt and resolve once the user answers via
   * {@link resolveDiscardPrompt}. Resolves `true` to discard and proceed, `false`
   * to cancel. The UI renders a modal keyed off {@link discardPromptOpen}.
   */
  #confirmDiscard(): Promise<boolean> {
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

  /** Validate now, ignoring any in-flight response that finishes out of order. */
  async revalidate(): Promise<void> {
    if (!this.ruleset) return;
    const seq = ++this.#seq;
    const snapshot = $state.snapshot(this.entity);
    try {
      const [result, effective, derived] = await Promise.all([
        ipc.validateEntity(snapshot, this.mode),
        ipc.effectiveScores(snapshot),
        ipc.derivedTotals(snapshot),
      ]);
      if (seq === this.#seq) {
        this.result = result;
        this.effective = effective;
        this.derived = derived;
        // A succeeding pass retires whatever the last rejected payload latched —
        // otherwise one bad value keeps the error banner up for the rest of the
        // session even after the user corrects it. Inside the sequence guard, so a
        // stale response cannot clear an error a newer pass just raised.
        this.error = null;
      }
    } catch (e) {
      // Guarded the same way, and for the mirror reason: a direct revalidate()
      // does not cancel a pending debounced one, so the rejection of a payload
      // the user has already corrected can land after the corrected pass
      // succeeded. Only the current pass may set *or* clear `error`.
      if (seq === this.#seq) this.error = e as AppError;
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
        // A fresh entity is a clean baseline. A language reload (else branch)
        // keeps the edited entity, so it must NOT reset the baseline — doing so
        // would drop `dirty` to false while unsaved edits still exist.
        this.#savedSnapshot = this.#snapshot();
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
