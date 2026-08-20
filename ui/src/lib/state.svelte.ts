// Central application state (Svelte 5 runes). A single instance is shared
// across components. It owns the entity being edited, the loaded ruleset, the
// validation result, and the active language/mode — and drives the live
// validation loop with debouncing plus a sequence guard against stale results.

import {
  firstBlockedPhaseIndex,
  incompletePhases,
  mandatoryTraitRefs,
  phaseHasBlockingIssue,
  sameSelection,
  wizardPhases,
} from './derive';
import { buildBundle, translate, type Lang, type TranslateArgs } from './i18n';
import * as ipc from './ipc';
import type { AgingNote, AgingOutcome, AgingTotal, CloseGuardLabels, CrisisPreview } from './ipc';
import type {
  AppError,
  Characteristic,
  CreationPhase,
  DerivedTotals,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
  LongevitySource,
  Realm,
  ReputationType,
  Selection,
  ValidationIssue,
  ValidationMode,
  ValidationResult,
} from './types';

const VALIDATE_DEBOUNCE_MS = 150;

/**
 * Save-format schema version written into every new entity. Exported so test
 * harnesses seed the current version instead of a literal that silently rots.
 *
 * Mirrors `arm_rules::SCHEMA_VERSION` by hand; the Rust constant is the source
 * and `the_frontend_mirrors_the_engine_schema_version` pins the two together.
 */
export const SCHEMA_VERSION = 15;

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
 *
 * **Keyed by panel, deliberately not by view** — settled in slice 6b8c, having been
 * frozen in 6b1b when the wizard arrived and left open until now. The wizard and the
 * editor mount the very same picker components over the very same character, and
 * `store.view` puts exactly one of them on screen at a time, so a per-view split
 * could only ever double the state and then have to decide which copy a filter
 * *survives* into: a player who narrowed the Virtue list to Minor Hermetic in the
 * wizard and pressed Finish would land on the editor's Virtues tab with the filter
 * silently reset, mid-task. Carrying it across is the behaviour worth having, and it
 * is what "keyed by panel identity" already gives. Do not "fix" this by adding a
 * view dimension.
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

/**
 * A blank character of `typeId`. The type is a required argument because it is
 * fixed at creation time (see {@link AppStore.createCharacter}); no id is
 * defaulted here, so the caller decides and the profile catalogue stays data.
 */
function newEntity(rulesetId: string, version: string, typeId: string): Entity {
  return {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: rulesetId, version },
    entity_kind: 'character',
    type_id: typeId,
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

/**
 * Where a character's Ability/Art experience comes from: a typed `xp_pool`
 * (direct entry) or the blocks its life stages earn (the guided flow).
 */
export type AbilityFunding = 'pool' | 'life_stages';

/**
 * The Sample Childhood package the player is *considering*, plus the values typed
 * into its parameter slots — the in-progress form, before it is applied.
 *
 * Deliberately UI state and never part of the entity. The entity records only the
 * package actually taken (`life_stages.childhood_package`) and the Ability rows the
 * application writes, whose `parameter` values *are* these slot values; keeping a
 * parallel draft on the entity would give one decision two representations that can
 * diverge — and would dirty the document for merely opening a picker. Held on the
 * store rather than in the component so it survives a tab switch, exactly like
 * {@link PickerFilters}.
 */
export interface ChildhoodDraft {
  /** The package the picker has selected; `null` while none is chosen. */
  packageId: string | null;
  /** Slot key (`area_a`, `language`, …) -> the player's value. Blanks are absent. */
  slots: Record<string, string>;
}

/** A fresh, empty childhood draft (the initial/reset state). */
export function defaultChildhoodDraft(): ChildhoodDraft {
  return { packageId: null, slots: {} };
}

/**
 * The aging roll the player is working on: which owed year it is for, the stress
 * die they typed, and where they are placing the Aging Points the table left to
 * them.
 *
 * UI-only state, exactly like {@link ChildhoodDraft} and for the same reason —
 * only more strictly. The die is not a choice the character records: the engine
 * carries no `rand` dependency, so the player rolls a stress die at the table and
 * types it in, and what the character keeps is the *result* the year applied
 * (`Entity.aging_log`), never the input. Since `dirty` is a snapshot compare of
 * the entity, holding the die here is what makes "the calculator does not persist"
 * mechanically true rather than merely intended.
 *
 * The distribution rides along because it is one form with the die: "Gain
 * sufficient Aging Points (in any Characteristic**s**)"
 * (Core Rules.md:16602/:16611) is plural, so the player may spread the points,
 * and the map is only meaningful against the award the current die produced.
 */
export interface AgingDraft {
  /** The age of the owed year being rolled for; `null` while none is picked. */
  age: number | null;
  /** The stress die the player typed; `null` while the field is blank. */
  die: number | null;
  /** Characteristic -> Aging Points placed there. Zeroes are absent. */
  distribution: Partial<Record<Characteristic, number>>;
  /**
   * The **Simple Die** thrown at the Crisis Table (Core Rules.md:16621), for a
   * year the aging row sent there; `null` while the field is blank, which the
   * engine records as a Crisis owed and unrolled rather than refusing.
   *
   * Draft state for the same reason the stress die is: the engine rolls neither,
   * and what the character keeps is the year's result, never its inputs.
   */
  crisisDie: number | null;
}

/** A fresh, empty aging draft (the initial/reset state). */
export function defaultAgingDraft(): AgingDraft {
  return { age: null, die: null, distribution: {}, crisisDie: null };
}

class AppStore {
  lang = $state<Lang>('en');
  ruleset = $state<LocalizedRuleset | null>(null);
  // The startup screen's placeholder: no character exists yet, so its type is
  // empty. A real one is only ever built by {@link createCharacter} or a load.
  entity = $state<Entity>(newEntity('', '', ''));
  mode = $state<ValidationMode>('enforced');
  result = $state<ValidationResult | null>(null);
  effective = $state<EffectiveScores | null>(null);
  derived = $state<DerivedTotals | null>(null);
  error = $state<AppError | null>(null);
  loading = $state(false);
  // Per-picker filter/search state; persists across tab switches (see
  // {@link PickerFilters}). Not part of the entity, so it is never saved.
  filters = $state<PickerFilters>(defaultPickerFilters());

  // The in-progress Sample Childhood choice (see {@link ChildhoodDraft}). Like
  // {@link filters} it is UI state: never part of the entity, so it is never saved
  // and drafting never dirties the document.
  childhoodDraft = $state<ChildhoodDraft>(defaultChildhoodDraft());

  /**
   * Why the last attempt to take a Sample Childhood package was refused, for the
   * picker to render beside the offending slots. Empty when there is nothing to say.
   *
   * Deliberately its own field rather than part of {@link result}: these findings
   * describe the *command input* the player just submitted, not the state of the
   * entity — which is unchanged by a rejection — so mixing them into the validation
   * results `revalidate()` owns would put issues about a rejected form on a
   * character sheet that never took it. Cleared on the next apply and by every draft
   * edit, since a rejection pointing at a field the user has just corrected is worse
   * than none at all.
   */
  childhoodRejections = $state<ValidationIssue[]>([]);

  /**
   * The in-progress aging roll (see {@link AgingDraft}). UI state like
   * {@link childhoodDraft}: never part of the entity, so it is never saved and
   * typing a die never dirties the document.
   */
  agingDraft = $state<AgingDraft>(defaultAgingDraft());

  /**
   * The engine's answer for the drafted roll: the AGING TOTAL with every term
   * that made it, and the row it lands on. `null` until a year and a die are both
   * given (and while the debounced round trip is still out).
   *
   * Held rather than derived because it is the *engine's* reading — a stress die
   * explodes, so no bounded lookup table in JS could stand in for it, and
   * re-deriving the outcome here would be a second implementation of the table.
   */
  agingPreview = $state<{
    total: AgingTotal;
    outcome: AgingOutcome;
    /**
     * The Crisis the year would send the character to, once the row demands one,
     * the Simple Die is typed and the Aging Points are placed — the engine reads
     * it off the year it would apply, so what is shown is what Apply writes.
     */
    crisis?: CrisisPreview | null;
  } | null>(null);

  /**
   * What the year just applied had to TELL the player, as opposed to what it
   * wrote — today only the Longevity Ritual a Crisis spends (`:16573`), which the
   * entity cannot show because it deliberately keeps the stored choice.
   *
   * Cleared whenever the draft is, and by the next apply or revert: a note about
   * a year the player has since taken back would be a lie about the character in
   * front of them.
   */
  agingNotes = $state<AgingNote[]>([]);

  /**
   * Why the last preview, apply or revert was refused, for the calculator to
   * render. Empty when there is nothing to say.
   *
   * Its own field rather than part of {@link result}, on the
   * {@link childhoodRejections} precedent: these findings are about the form the
   * player just submitted, not about the character — which a refusal leaves
   * untouched.
   */
  agingRejections = $state<ValidationIssue[]>([]);

  /**
   * The owed year the calculator is on: the player's own pick, or the first year
   * the aging log does not yet record.
   *
   * Defaulted here rather than in the component so the store and the screen can
   * never disagree about which year a typed die belongs to — the component only
   * renders what this says.
   */
  agingYear = $derived<number | null>(
    this.agingDraft.age ??
      (this.effective?.aging?.schedule ?? []).find((year) => !year.recorded)?.age ??
      null,
  );

  /**
   * Which screen the app is on: the startup choice screen, the guided wizard, or
   * the character editor. The app boots on `start`; {@link createCharacter} and a
   * successful {@link open} enter the editor, {@link startWizard} enters the
   * wizard, and {@link newDocument} is the way back. While it is `start` the
   * entity is a placeholder, not a character — which is why {@link revalidate}
   * refuses to send it to the engine. The wizard's entity IS a character, so
   * validation runs there normally.
   */
  view = $state<'start' | 'editor' | 'wizard'>('start');

  /**
   * Index of the wizard's current step within {@link wizardPhases}.
   *
   * Navigation state, deliberately not part of the entity: moving through the
   * flow is not an edit, so it never dirties the document and a save records no
   * progress through it.
   */
  wizardStep = $state(0);

  /**
   * The furthest step reached by advancing. Raised only by {@link wizardNext},
   * never lowered by going back, so the rail can offer every step the user has
   * already seen while still refusing to skip ahead into unseen ones.
   */
  wizardFurthest = $state(0);

  /**
   * The wizard's steps for the current character: the type profile's own ordered
   * phases, then the terminal `review` step. Empty when no profile is loaded.
   */
  wizardPhases = $derived(wizardPhases(this.ruleset?.ruleset.type_profiles[this.entity.type_id]));

  /** The phase the wizard is currently on. */
  wizardPhase = $derived<CreationPhase>(this.wizardPhases[this.wizardStep] ?? 'review');

  /**
   * Whether the wizard may advance: the current phase carries no error.
   *
   * Errors only, so a warning never gates — which means a phase can be legal but
   * empty (a magus may pass the House step with no House, since `house_unset` is
   * an advisory). In Advisory mode the engine downgrades every error to a warning
   * and in Silent mode it reports none, so in both the wizard stops gating
   * entirely; the validation-mode control is the intended escape hatch.
   */
  wizardCanAdvance = $derived(!phaseHasBlockingIssue(this.result?.issues ?? [], this.wizardPhase));

  /**
   * Whether the wizard may finish: no error remains anywhere in the character.
   *
   * Deliberately wider than {@link wizardCanAdvance}: the findings no creation
   * phase owns (equipment, Might, Warping) gate no single step, and a phase the
   * character type never declares has no step at all — Finish is where both still
   * have to be answered.
   */
  wizardCanFinish = $derived(!(this.result?.issues ?? []).some((i) => i.severity === 'error'));

  /**
   * The steps of this flow the player has recorded nothing for, in rail order, as
   * the engine reports them.
   *
   * Purely informational, and deliberately kept out of {@link wizardCanAdvance}
   * and {@link wizardCanFinish}: legal is not the same as finished, so an empty
   * step is marked, never blocked.
   */
  wizardIncompletePhases = $derived(incompletePhases(this.result));

  /** Whether the step currently on screen is one of those. */
  wizardPhaseIncomplete = $derived(this.wizardIncompletePhases.includes(this.wizardPhase));

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

  /**
   * How the character's Abilities (and Arts) are funded: by a typed `xp_pool`, or
   * by the experience its life stages earn.
   *
   * Read off the entity rather than held in its own `$state`, because the presence
   * of `life_stages` already *is* the switch everywhere in the engine
   * (`LifeStageRules::budget`, `validate_life_stage_plan`, `EffectiveScores.life_stage`).
   * A second flag could disagree with a loaded save; a derivation cannot, so a save
   * carrying a plan lands in the guided mode with no reconciliation code at all.
   */
  abilityFunding = $derived<AbilityFunding>(this.entity.life_stages ? 'life_stages' : 'pool');

  #bundle = $derived(buildBundle(this.lang));
  #timer: ReturnType<typeof setTimeout> | undefined;
  #seq = 0;
  // The aging preview's own debounce and sequence guard. Deliberately separate
  // from the validation pair above: the die is not an entity edit, so it must not
  // ride on `#scheduleValidate` — and a keystroke in the die field must not cancel
  // a pending validation of the character (or the other way round).
  #agingTimer: ReturnType<typeof setTimeout> | undefined;
  #agingSeq = 0;
  // The exact error object the last rejected validate published to `error`, so a
  // later succeeding validate can tell its own banner apart from a file-operation
  // failure that landed in the same shared field. Not reactive: it never renders.
  #validateError: AppError | null = null;

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

  /** The item ids the given type profile mandates (required traits + required Gift). */
  #mandatoryTraitRefs(typeId: string | undefined): Set<string> {
    const profile = typeId ? this.ruleset?.ruleset.type_profiles[typeId] : undefined;
    return mandatoryTraitRefs(profile);
  }

  /**
   * Select the Hermetic House (or clear it with `null`). A discrete action, so
   * it validates immediately like {@link setMode}. Switching House drops any
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
   * its first option. Mirrors {@link setHouse}.
   */
  async setMythicType(mythicType: string | null): Promise<void> {
    if ((this.entity.mythic_type ?? null) === mythicType) return;
    const previousPackage = this.#mythicPackage(this.entity.mythic_type ?? null);
    const nextPackage = this.#mythicPackage(mythicType);
    // Drop the previous type's seeded package rows the new type doesn't require.
    const kept = (this.entity.selections ?? []).filter(
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
      if (!kept.some((s) => sameSelection(s, pkg))) kept.push(pkg);
    }
    this.entity.selections = kept;
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
    const selections = [...(this.entity.selections ?? [])];
    const idx = selections.findIndex((s) => s.ref === previousRef);
    if (idx >= 0) selections.splice(idx, 1);
    if (!selections.some((s) => s.ref === nextRef)) selections.push({ ref: nextRef });
    this.entity.selections = selections;
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
    const present = (this.entity.selections ?? []).some((s) => s.ref === ref);
    if (!repeatable && present) return;
    this.entity.selections = [...(this.entity.selections ?? []), { ref }];
    this.#scheduleValidate();
  }

  /** Selection edits are by row index, since a repeatable item has several rows. */
  removeSelectionAt(index: number): void {
    this.entity.selections = (this.entity.selections ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  setParamAt(index: number, key: string, value: string): void {
    this.entity.selections = (this.entity.selections ?? []).map((s, i) =>
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
    this.entity.selections = (this.entity.selections ?? []).map((s, i) =>
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
   * Switch how Abilities are funded (see {@link abilityFunding}). A discrete
   * action, so it validates immediately like {@link setHouse}.
   *
   * Entering the guided mode adds an empty plan and zeroes `xp_pool`, since the
   * engine makes a plan and a typed pool mutually exclusive; leaving it deletes
   * the plan key outright, which keeps the save sparse and *is* the pruning of the
   * plan's now-stale contents (the {@link #prunedHouseChoices} precedent).
   *
   * Bought `ability_scores` survive either switch untouched: funding less
   * experience than the rows demand is reported as `not_enough_xp` — visible and
   * fixable — which is strictly better than silently discarding the player's work.
   *
   * Leaving guided mode also prunes the {@link childhoodDraft} and its
   * {@link childhoodRejections}: the plan is what a draft is *for*, so a surviving
   * one would prefill a package for a future plan that starts from nothing, showing
   * stale slot faults for a decision nobody has made yet. Deliberately asymmetric —
   * *entering* guided mode keeps an in-progress draft, since toggling the radio back
   * and forth without ever leaving would otherwise destroy typed slot values.
   *
   * The {@link agingDraft} goes with it, on one blanket rule: an un-submitted draft
   * never outlives a change to how the document is built. One rule covering every
   * draft keeps their lifetimes auditable in a single place, which is worth more
   * than sparing a typed die across a funding switch.
   */
  async setAbilityFunding(funding: AbilityFunding): Promise<void> {
    if (this.abilityFunding === funding) return;
    if (funding === 'life_stages') {
      this.entity.life_stages = {};
      this.entity.xp_pool = 0;
    } else {
      delete this.entity.life_stages;
      this.childhoodDraft = defaultChildhoodDraft();
      this.childhoodRejections = [];
      this.clearAgingDraft();
    }
    await this.revalidate();
  }

  /**
   * Name the character's native language — the Ability the childhood's language
   * experience is spent on. Debounced like the other typed fields.
   *
   * A blank value deletes the key rather than storing an empty string: the engine
   * reads blank as unset, and a sparse save is the canonical one. A no-op without a
   * plan, so a surface shown in the flat mode can never conjure one into being;
   * {@link setAbilityFunding} is the only way into the guided mode.
   */
  setNativeLanguage(value: string): void {
    const plan = this.entity.life_stages;
    if (!plan) return;
    const language = value.trim();
    if (language) {
      plan.native_language = language;
    } else {
      delete plan.native_language;
    }
    this.#scheduleValidate();
  }

  /**
   * Set (or clear) the age the magus was gauntleted at — the one post-Gauntlet
   * figure stored, with the years, points and experience all derived from it.
   *
   * A blank or zero age deletes the key, which is what "the magus stands AT its
   * Gauntlet" means: with no Gauntlet age the engine reads the character's own age
   * as the Gauntlet age, exactly how a magus was built before the field existed. So
   * a magus at its Gauntlet still saves nothing but its native language.
   *
   * A no-op without a plan, like {@link setNativeLanguage}: a surface shown in the
   * flat mode can never conjure one into being.
   */
  setGauntletAge(age: number | null): void {
    this.#setPlanCount('gauntlet_age', age);
  }

  /**
   * Set (or clear) the lab seasons charged against the post-Gauntlet years — 10
   * points each, and at most three a year count (the third already takes the whole
   * 30, so a fourth is free). Zero deletes the key, like {@link setGauntletAge}.
   */
  setPostGauntletLabSeasons(seasons: number | null): void {
    this.#setPlanCount('post_gauntlet_lab_seasons', seasons);
  }

  /**
   * Set (or clear) how many post-Gauntlet points the player took as levels of
   * spells rather than experience. Zero deletes the key, like
   * {@link setGauntletAge}.
   */
  setPostGauntletSpellLevels(levels: number | null): void {
    this.#setPlanCount('post_gauntlet_spell_levels', levels);
  }

  /**
   * Write one of the plan's optional u32 counts, deleting the key when the input is
   * blank, zero or not a number. The three post-Gauntlet fields share this shape:
   * absent is the meaningful default for every one of them, so a sparse save is the
   * canonical one.
   */
  #setPlanCount(
    field: 'gauntlet_age' | 'post_gauntlet_lab_seasons' | 'post_gauntlet_spell_levels',
    value: number | null,
  ): void {
    const plan = this.entity.life_stages;
    if (!plan) return;
    if (value != null && Number.isFinite(value) && value > 0) {
      plan[field] = clampInt(value, 1, U32_MAX);
    } else {
      delete plan[field];
    }
    this.#scheduleValidate();
  }

  /**
   * Select the Sample Childhood package the player is considering, or clear the
   * consideration with `null`. Draft state only (see {@link ChildhoodDraft}):
   * nothing is written to the entity until {@link applyChildhoodPackage}.
   *
   * Switching packages drops the slot values the new package does not declare, so a
   * stale answer from the previous one can never be submitted — the
   * {@link #prunedHouseChoices} precedent. Slots both packages ask for survive, so
   * comparing two childhoods does not mean re-typing the shared answers. Clearing
   * drops the selection and every slot.
   *
   * Not validated and not debounced: a draft is a form the engine has not been shown
   * yet, so there is nothing to check until it is submitted. Editing the draft does
   * retire the last rejection ({@link childhoodRejections}), which was about the form
   * as it stood before the edit.
   */
  setChildhoodDraftPackage(packageId: string | null): void {
    this.childhoodRejections = [];
    if (packageId === null) {
      this.childhoodDraft = defaultChildhoodDraft();
      return;
    }
    this.childhoodDraft = { packageId, slots: this.#prunedChildhoodSlots(packageId) };
  }

  /**
   * Answer one of the drafted package's parameter slots (the Area Lore region, the
   * language, …). A blank or whitespace-only value deletes the key rather than
   * storing an empty string: an unanswered slot is absent, which is what the
   * engine's "slot unanswered" rejection is about. Draft state only, like
   * {@link setChildhoodDraftPackage}.
   */
  setChildhoodDraftSlot(slot: string, value: string): void {
    this.childhoodRejections = [];
    const answer = value.trim();
    const slots = { ...this.childhoodDraft.slots };
    if (answer) {
      slots[slot] = answer;
    } else {
      delete slots[slot];
    }
    this.childhoodDraft.slots = slots;
  }

  /** The parameter slot keys the given package's entries declare. */
  #childhoodSlotKeys(packageId: string): Set<string> {
    const keys = new Set<string>();
    for (const entry of this.ruleset?.ruleset.childhoods?.[packageId]?.entries ?? []) {
      if (entry.slot) keys.add(entry.slot);
    }
    return keys;
  }

  /** Drafted slot answers kept only where the target package still asks for them. */
  #prunedChildhoodSlots(packageId: string): Record<string, string> {
    const asked = this.#childhoodSlotKeys(packageId);
    const kept: Record<string, string> = {};
    for (const [slot, answer] of Object.entries(this.childhoodDraft.slots)) {
      if (asked.has(slot)) kept[slot] = answer;
    }
    return kept;
  }

  /**
   * Take the drafted Sample Childhood package: submit it with its slot answers and
   * keep whatever the engine decides. A no-op while no package is drafted — there is
   * nothing to submit.
   *
   * The engine owns the whole mechanic, so this only routes its two outcomes. On
   * `applied` the returned entity replaces the current one wholesale (the Ability
   * rows the package writes and the record of the package taken arrive together);
   * `dirty` needs no help, since it derives from the entity snapshot. On `rejected`
   * the entity is left exactly as it was and the findings land in
   * {@link childhoodRejections} for the picker to show against the offending slots.
   */
  async applyChildhoodPackage(): Promise<void> {
    const packageId = this.childhoodDraft.packageId;
    if (!packageId) return;
    this.childhoodRejections = [];
    try {
      const outcome = await ipc.applyChildhoodPackage(
        $state.snapshot(this.entity),
        packageId,
        $state.snapshot(this.childhoodDraft.slots),
      );
      if (outcome.status === 'rejected') {
        this.childhoodRejections = outcome.issues;
        return;
      }
      this.entity = outcome.entity;
      await this.revalidate();
    } catch (e) {
      this.error = e as AppError;
    }
  }

  /**
   * Pick which owed year the calculator is rolling for, or fall back to the
   * default with `null`. Draft state only (see {@link AgingDraft}).
   *
   * Drops the point distribution: it was placed against the award the *other*
   * year's roll produced, and carrying it over would let a player apply points
   * they never re-confirmed. The typed die survives, since it is the number the
   * player has in front of them either way.
   */
  setAgingYear(age: number | null): void {
    this.agingRejections = [];
    this.agingDraft = { ...this.agingDraft, age, distribution: {} };
    this.#scheduleAgingPreview();
  }

  /**
   * Record the stress die the player rolled, or clear it with `null`.
   *
   * "AGING TOTAL: Stress die (no botch) + age/10 (round up) …"
   * Source: Ars Magica - Definitive Edition (Core Rules).md:16567
   *
   * A stress die explodes, so the value has a floor of 0 and no ceiling; it is
   * clamped only to what the command's `i32` can carry. Debounced through
   * {@link #scheduleAgingPreview} and NOT through `#scheduleValidate` — the die is
   * not an edit of the character, so it must never be sent as one.
   */
  setAgingDie(die: number | null): void {
    this.agingRejections = [];
    const value = die != null && Number.isFinite(die) ? clampInt(die, 0, I32_MAX) : null;
    this.agingDraft = { ...this.agingDraft, die: value, distribution: {} };
    this.#scheduleAgingPreview();
  }

  /**
   * Place (or, with 0, un-place) Aging Points in one Characteristic.
   *
   * "If an Aging Point 'in any Characteristic' is gained, the player may choose
   * the Characteristic." Source: Ars Magica - Definitive Edition (Core
   * Rules).md:16615 — and `:16602`/`:16611` say "in any Characteristic**s**",
   * plural, so this is a map and not a single pick.
   *
   * The engine is the authority on whether the map is legal; this only records
   * it, and the calculator refuses to submit one that does not sum to the award.
   */
  setAgingDistribution(characteristic: Characteristic, points: number | null): void {
    this.agingRejections = [];
    const distribution = { ...this.agingDraft.distribution };
    if (points != null && Number.isFinite(points) && points > 0) {
      distribution[characteristic] = clampInt(points, 1, U8_MAX);
    } else {
      delete distribution[characteristic];
    }
    this.agingDraft = { ...this.agingDraft, distribution };
    // Placing the points moves the CRISIS TOTAL, because those points ARE the
    // Decrepitude increase `:16619` puts first — so the reading is asked for
    // again rather than left standing at a number the year will not write.
    this.#scheduleAgingPreview();
  }

  /**
   * Record the Simple Die the player threw at the Crisis Table, or clear it with
   * `null`.
   *
   * "CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score"
   * Source: Ars Magica - Definitive Edition (Core Rules).md:16621
   *
   * Draft state and a preview, never a validate: like the stress die this is
   * player input the character must not hold, so typing it cannot dirty the
   * document.
   */
  setAgingCrisisDie(die: number | null): void {
    this.agingRejections = [];
    const value = die != null && Number.isFinite(die) ? clampInt(die, 0, I32_MAX) : null;
    this.agingDraft = { ...this.agingDraft, crisisDie: value };
    this.#scheduleAgingPreview();
  }

  /** Abandon the drafted roll: the year, both dice, the points, the last refusal
   *  and whatever the last applied year had to say. */
  clearAgingDraft(): void {
    clearTimeout(this.#agingTimer);
    this.#agingTimer = undefined;
    this.#agingSeq++;
    this.agingDraft = defaultAgingDraft();
    this.agingPreview = null;
    this.agingRejections = [];
    this.agingNotes = [];
  }

  /**
   * Ask the engine what the drafted die makes of the drafted year, and keep the
   * answer in {@link agingPreview}. Writes nothing to the character.
   *
   * Guarded by its own sequence counter, the {@link revalidate} idiom: the die
   * field is typed into, so several previews can be in flight and they may finish
   * out of order — a stale answer must never overwrite a newer one, or the player
   * sees an outcome for a die they have already changed.
   */
  async previewAgingRoll(): Promise<void> {
    const seq = ++this.#agingSeq;
    const { age, die, distribution, crisisDie } = this.agingDraft;
    const year = age ?? this.agingYear;
    if (year == null || die == null) {
      this.agingPreview = null;
      return;
    }
    try {
      const projection = await ipc.agingPreview(
        $state.snapshot(this.entity),
        year,
        die,
        $state.snapshot(distribution),
        crisisDie,
      );
      if (seq !== this.#agingSeq) return;
      if (projection.status === 'rejected') {
        this.agingPreview = null;
        this.agingRejections = projection.issues;
        return;
      }
      this.agingPreview = {
        total: projection.total,
        outcome: projection.outcome,
        crisis: projection.crisis ?? null,
      };
      this.agingRejections = [];
    } catch (e) {
      if (seq === this.#agingSeq) this.error = e as AppError;
    }
  }

  /**
   * Apply the drafted roll: the engine resolves the year, writes the Aging Points,
   * advances the apparent age and appends the log entry, all in one move
   * (`resolve_year` is the aging subsystem's single writer).
   *
   * On acceptance the returned character replaces the current one wholesale and
   * the draft is spent, so the calculator moves on to the next owed year; `dirty`
   * needs no help, since it derives from the entity snapshot. On refusal the
   * character is untouched and the findings land in {@link agingRejections}.
   */
  async applyAgingRoll(): Promise<void> {
    const { die, distribution, crisisDie } = this.agingDraft;
    const year = this.agingYear;
    if (year == null || die == null) return;
    this.agingRejections = [];
    this.agingNotes = [];
    try {
      const application = await ipc.agingApply(
        $state.snapshot(this.entity),
        year,
        die,
        $state.snapshot(distribution),
        crisisDie,
      );
      if (application.status === 'rejected') {
        this.agingRejections = application.issues;
        return;
      }
      this.entity = application.entity;
      this.agingDraft = defaultAgingDraft();
      this.agingPreview = null;
      // Kept past the draft it came from: the note is about the character now on
      // screen, not about the form that has just been spent.
      this.agingNotes = application.notes ?? [];
      await this.revalidate();
    } catch (e) {
      this.error = e as AppError;
    }
  }

  /**
   * Take one recorded year back off, exactly — a pre-play catch-up of 25 rolls
   * with no undo would not be shippable. The engine subtracts precisely the points
   * the log entry recorded and removes the entry.
   */
  async revertAgingRoll(age: number): Promise<void> {
    this.agingRejections = [];
    // The year is going away, so what it had to say goes with it.
    this.agingNotes = [];
    try {
      const reversion = await ipc.agingRevert($state.snapshot(this.entity), age);
      if (reversion.status === 'rejected') {
        this.agingRejections = reversion.issues;
        return;
      }
      this.entity = reversion.entity;
      await this.revalidate();
    } catch (e) {
      this.error = e as AppError;
    }
  }

  #scheduleAgingPreview(): void {
    clearTimeout(this.#agingTimer);
    this.#agingTimer = setTimeout(() => void this.previewAgingRoll(), VALIDATE_DEBOUNCE_MS);
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

  /**
   * Point an Art-domain parameter of a selection at a specific Art (by id).
   * `key` is the *declaring* parameter's key — usually `art` (Puissant Art), but
   * an Art-domain parameter may be keyed otherwise (Master of (Form) Creatures
   * declares `form` over the Art catalogue), and the value must land under the
   * key the item declared or the engine reports it missing.
   */
  setArtBonusTarget(index: number, key: string, artId: string): void {
    this.entity.selections = (this.entity.selections ?? []).map((s, i) =>
      i === index ? { ...s, params: { ...(s.params ?? {}), [key]: artId } } : s,
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

  /**
   * Take or drop one Living Conditions row, by its id into the aging catalogue.
   *
   * A SET, not a single pick: "Modifiers marked with an asterisk are cumulative
   * with each other" (Core Rules.md:16594), so several rows can hold at once. The
   * engine decides which combinations are legal
   * (`living_conditions_conflict`); this only records the choice.
   *
   * Written sorted — canonical serialization, so ticking two rows in either order
   * produces the same save and a zero-noise diff. The last row off deletes the key
   * entirely: an empty set is the engine's own default (and the table's "Average
   * peasant 0"), so a sparse save is the canonical one, exactly as
   * {@link #setPlanCount} treats a zeroed count.
   */
  setLivingCondition(id: string, chosen: boolean): void {
    const next = new Set(this.entity.living_conditions ?? []);
    if (chosen) {
      next.add(id);
    } else {
      next.delete(id);
    }
    if (next.size === 0) {
      delete this.entity.living_conditions;
    } else {
      this.entity.living_conditions = [...next].sort();
    }
    this.#scheduleValidate();
  }

  /**
   * Append a blank, hand-written log row. It names no year: `AgingLogEntry.year`
   * is optional, so an undated entry is a supported shape rather than a claim
   * that the roll happened in the year 0.
   */
  addAgingLogEntry(): void {
    this.entity.aging_log = [...(this.entity.aging_log ?? []), { year: null, effect: '' }];
    this.#scheduleValidate();
  }

  removeAgingLogEntryAt(index: number): void {
    this.entity.aging_log = (this.entity.aging_log ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  /**
   * Set (or, with `null`, clear) the calendar year of a log entry. Clearing
   * un-dates the entry rather than dating it to year 0 — the same treatment
   * {@link setBirthYear} gives an emptied field, and the shape a character with
   * no birth year gets from the engine.
   */
  setAgingLogEntryYear(index: number, year: number | null): void {
    const clamped = year != null && Number.isFinite(year) ? clampInt(year, I32_MIN, I32_MAX) : null;
    this.entity.aging_log = (this.entity.aging_log ?? []).map((e, i) =>
      i === index ? { ...e, year: clamped } : e,
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
    if (this.#opInFlight) return;
    this.#opInFlight = true;
    this.error = null;
    try {
      await ipc.exportMarkdown(
        $state.snapshot(this.entity),
        await this.#exportLabels(),
        null,
        this.currentPath,
      );
    } catch (e) {
      this.error = e as AppError;
    } finally {
      this.#opInFlight = false;
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
    for (const key of composedExportLabelKeys(this.ruleset)) keys.add(key);
    const labels: Record<string, string> = {};
    for (const key of keys) labels[key] = this.t(key);
    return labels;
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
        // A loaded character's recorded childhood package is history, not a draft:
        // its slot answers already live in its Ability rows. Starting the draft
        // empty is what keeps that coherent — nothing pre-fills a package whose
        // slots are gone, so no applied package can show a spurious empty slot.
        this.childhoodDraft = defaultChildhoodDraft();
        // Likewise the aging draft: a die typed against the year the *previous*
        // character owed says nothing about this one, and the years it already
        // recorded arrive in its own log.
        this.clearAgingDraft();
        // Opening is reachable from any screen, so a load always lands in the
        // editor; a cancelled dialog leaves the current screen alone. A save
        // records no wizard progress, so an opened character is a finished
        // document and never resumes mid-flow — hence the rail reset.
        this.view = 'editor';
        this.#resetWizardNav();
        await this.revalidate();
      }
    } catch (e) {
      this.error = e as AppError;
    } finally {
      this.#opInFlight = false;
    }
  }

  /**
   * Discard the current document and return to the startup screen — the only way
   * back to the character-type choice, since a type is fixed at creation. Prompts
   * to discard first when the current document has unsaved edits; a cancelled
   * prompt aborts. Clears the current file, the picker filters, and the saved
   * baseline, and leaves behind the typeless placeholder (no character exists
   * again, so no type may be implied).
   */
  async newDocument(): Promise<void> {
    if (this.#opInFlight || this.discardPromptOpen) return;
    if (this.dirty && !(await this.#confirmDiscard())) return;
    const { id, version } = this.ruleset?.ruleset ?? this.entity.ruleset;
    this.entity = newEntity(id, version, '');
    this.view = 'start';
    this.#resetWizardNav();
    this.currentPath = null;
    this.filters = defaultPickerFilters();
    this.childhoodDraft = defaultChildhoodDraft();
    this.clearAgingDraft();
    this.result = null;
    this.effective = null;
    this.derived = null;
    this.#savedSnapshot = this.#snapshot();
    await this.revalidate();
  }

  /**
   * Start a brand-new character of `typeId` and enter the editor, replacing
   * whatever was being edited. The type is fixed at creation, so the profile's
   * mandatory free traits (a magus's The Gift + Hermetic Magus) are seeded right
   * away, from the profile's own `required_traits` + `gift_id`, so no id is
   * hardcoded here. Otherwise it resets exactly what {@link newDocument} does:
   * the current file, the picker filters, the last results, and the saved
   * baseline (so a freshly created character is not dirty).
   *
   * Deliberately does NOT prompt about unsaved changes: the caller is the
   * startup screen, which is only reached from a state with nothing to discard.
   * Creation is a discrete action, so it validates immediately rather than
   * through the debounce — like {@link setHouse}.
   */
  async createCharacter(typeId: string): Promise<void> {
    this.#instantiateCharacter(typeId);
    this.view = 'editor';
    await this.revalidate();
  }

  /**
   * Build a blank character of `typeId` and reset the document around it. The
   * shared half of {@link createCharacter} and {@link startWizard} — one place
   * seeds the profile's mandatory free traits, clears the current file, the
   * picker filters and the last results, and re-seeds the saved baseline so a
   * freshly created character is not dirty.
   */
  #instantiateCharacter(typeId: string): void {
    const { id, version } = this.ruleset?.ruleset ?? this.entity.ruleset;
    this.entity = newEntity(id, version, typeId);
    this.entity.selections = [...this.#mandatoryTraitRefs(typeId)].map((ref) => ({ ref }));
    this.currentPath = null;
    this.filters = defaultPickerFilters();
    this.childhoodDraft = defaultChildhoodDraft();
    this.clearAgingDraft();
    this.result = null;
    this.effective = null;
    this.derived = null;
    this.#savedSnapshot = this.#snapshot();
  }

  /**
   * Create a character of `typeId` and walk it through the guided wizard.
   *
   * The same instantiation {@link createCharacter} performs — the wizard edits a
   * real character from its first step, not a draft that is materialized at the
   * end — followed by the wizard view and a rail reset. Validates immediately
   * rather than through the debounce, so the first step is gated before the user
   * can act on it.
   */
  async startWizard(typeId: string): Promise<void> {
    this.#instantiateCharacter(typeId);
    this.view = 'wizard';
    this.#resetWizardNav();
    await this.revalidate();
  }

  /** Advance one step, unless the current phase holds an error. */
  wizardNext(): void {
    if (!this.wizardCanAdvance) return;
    if (this.wizardStep >= this.wizardPhases.length - 1) return;
    this.wizardStep += 1;
    this.wizardFurthest = Math.max(this.wizardFurthest, this.wizardStep);
  }

  /**
   * Step back one. Never gated: the user must always be able to reach the step
   * that needs fixing, including the one they have just broken.
   */
  wizardBack(): void {
    this.wizardStep = Math.max(this.wizardStep - 1, 0);
  }

  /**
   * Jump to an already-visited step from the rail.
   *
   * Forward jumps are held to the same gate as Next: the jump clamps to the first
   * blocking phase between here and there, **including the step being left**.
   * Otherwise the rail would be a way around the very gate that blocks Next.
   * Backward jumps are free, like {@link wizardBack}.
   */
  wizardGoTo(step: number): void {
    if (step < 0 || step > this.wizardFurthest) return;
    if (step <= this.wizardStep) {
      this.wizardStep = step;
      return;
    }
    const blocked = firstBlockedPhaseIndex(
      this.wizardPhases,
      this.result?.issues ?? [],
      this.wizardStep,
      step,
    );
    this.wizardStep = blocked ?? step;
  }

  /**
   * Leave the wizard for the editor, with the character exactly as the wizard left
   * it. Blocked while any error remains ({@link wizardCanFinish}).
   *
   * Nothing about the entity changes, so this deliberately does not revalidate —
   * the results on screen are already the ones for this character.
   */
  finishWizard(): void {
    if (!this.wizardCanFinish) return;
    this.view = 'editor';
    this.#resetWizardNav();
  }

  /** Send the rail back to the first step. */
  #resetWizardNav(): void {
    this.wizardStep = 0;
    this.wizardFurthest = 0;
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
    // The startup screen holds a placeholder, not a character: its empty type
    // would come back as an `unknown_type` error about a character that does not
    // exist. Skip the round trip — but leave nothing stranded behind: cancel a
    // pending debounced pass, retire any in-flight one through the sequence
    // guard, and clear a banner the validation path itself raised (never a file
    // operation's, which shares the channel and outlives this screen).
    //
    // Tests `start` specifically, NOT `!== 'editor'`: the wizard's entity is a
    // real character and must be validated on every keystroke, since that is what
    // gates its steps. Do not "tidy" this into a negation.
    if (this.view === 'start') {
      clearTimeout(this.#timer);
      this.#timer = undefined;
      this.#seq++;
      if (this.error === this.#validateError) this.error = null;
      this.#validateError = null;
      return;
    }
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
        // session even after the user corrects it. It may retire ONLY its own
        // error: `error` is one shared banner channel that file operations write
        // to as well, and a failed save must stay visible while the document is
        // still unsaved. Inside the sequence guard, so a stale response cannot
        // clear an error a newer pass just raised.
        if (this.error === this.#validateError) this.error = null;
        this.#validateError = null;
      }
    } catch (e) {
      // Guarded the same way, and for the mirror reason: a direct revalidate()
      // does not cancel a pending debounced one, so the rejection of a payload
      // the user has already corrected can land after the corrected pass
      // succeeded. Only the current pass may set *or* clear `error`.
      if (seq === this.#seq) {
        this.#validateError = e as AppError;
        this.error = this.#validateError;
      }
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
        this.entity = newEntity(id, version, '');
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
