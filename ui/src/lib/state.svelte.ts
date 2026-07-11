// Central application state (Svelte 5 runes). A single instance is shared
// across components. It owns the entity being edited, the loaded ruleset, the
// validation result, and the active language/mode — and drives the live
// validation loop with debouncing plus a sequence guard against stale results.

import { mandatoryTraitRefs, sameSelection } from './derive';
import { buildBundle, translate, type Lang, type TranslateArgs } from './i18n';
import * as ipc from './ipc';
import type {
  AppError,
  Characteristic,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
  ReputationType,
  Selection,
  ValidationMode,
  ValidationResult,
} from './types';

const VALIDATE_DEBOUNCE_MS = 150;
const SCHEMA_VERSION = 7;

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
    if (score === 0) {
      delete chars[characteristic];
    } else {
      chars[characteristic] = score;
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
    this.entity.xp_pool = Number.isFinite(xp) && xp > 0 ? Math.floor(xp) : 0;
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
   * (the chosen level); a fixed spell derives its level from the catalogue. The
   * same (spell, level) pair is not added twice — different General levels are
   * different spells, so they may coexist.
   */
  addSpell(spellId: string, level?: number | null): void {
    const lvl = typeof level === 'number' ? level : undefined;
    const present = (this.entity.spells ?? []).some(
      (s) => s.spell === spellId && (s.level ?? undefined) === lvl,
    );
    if (present) return;
    this.entity.spells = [
      ...(this.entity.spells ?? []),
      lvl === undefined ? { spell: spellId } : { spell: spellId, level: lvl },
    ];
    this.#scheduleValidate();
  }

  /** Spell edits are by row index, since a General spell can appear at several levels. */
  removeSpellAt(index: number): void {
    this.entity.spells = (this.entity.spells ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  /** Set (or clear) the character's age; drives the age → Ability-cap check. */
  setAge(age: number | null): void {
    this.entity.age = age != null && Number.isFinite(age) && age > 0 ? Math.floor(age) : null;
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
    const snapshot = $state.snapshot(this.entity);
    try {
      const [result, effective] = await Promise.all([
        ipc.validateEntity(snapshot, this.mode),
        ipc.effectiveScores(snapshot),
      ]);
      if (seq === this.#seq) {
        this.result = result;
        this.effective = effective;
      }
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
