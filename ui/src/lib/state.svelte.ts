// Central application state (Svelte 5 runes). A single instance is shared
// across components. It owns the entity being edited, the loaded ruleset, the
// validation result, and the active language/mode — and drives the live
// validation loop with debouncing plus a sequence guard against stale results.

import { AbilityWorkflow } from './ability-workflow.svelte';
import {
  AgingWorkflow,
  defaultAgingDraft,
  type AgingDraft,
  type AgingPreview,
} from './aging-workflow.svelte';
import { ArtWorkflow } from './art-workflow.svelte';
import {
  CORD_MAX,
  I32_MAX,
  I32_MIN,
  I8_MAX,
  I8_MIN,
  U16_MAX,
  U32_MAX,
  U8_MAX,
  clampInt,
} from './clamp';
import {
  ChildhoodWorkflow,
  defaultChildhoodDraft,
  type ChildhoodDraft,
} from './childhood-workflow.svelte';
import {
  abilityFloor,
  agingMigrationNotice,
  mandatoryTraitRefs,
  migratedCatalogueParameterNotice,
  resolvedLinksFrom,
  unresolvedCatalogueParameterNotice,
} from './derive';
import { EquipmentWorkflow } from './equipment-workflow.svelte';
import { FileOperations } from './file-operations.svelte';
import { HouseWorkflow } from './house-workflow.svelte';
import { AVAILABLE_LANGS, buildBundle, translate, type Lang, type TranslateArgs } from './i18n';
import * as ipc from './ipc';
import { MythicWorkflow } from './mythic-workflow.svelte';
import { SelectionWorkflow } from './selection-workflow.svelte';
import { SpellWorkflow } from './spell-workflow.svelte';
import type {
  AgingNote,
  CloseGuardLabels,
  MigratedCatalogueParameter,
  UnresolvedCatalogueParameter,
} from './ipc';
import type { DocumentAction, MenuFlags } from './menu';
import type {
  AbilityFunding,
  AbilityParamValue,
  AppError,
  Characteristic,
  DerivedTotals,
  EffectiveScores,
  Entity,
  LoadoutState,
  LocalizedRuleset,
  LongevitySource,
  Realm,
  ReputationType,
  Selection,
  ValidationIssue,
  ValidationMode,
  ValidationResult,
} from './types';
import { WizardNavigation } from './wizard-navigation.svelte';

// Re-exported so existing importers of `AgingDraft`/`defaultAgingDraft` (e.g.
// `AgingRollCalculator.test.ts`) keep working unchanged after the aging
// workflow moved to its own module (VA6) — the type/factory's home changed,
// not its shape or its public import path.
export { defaultAgingDraft };
export type { AgingDraft };
// Likewise for the childhood-draft workflow (extracted to
// `childhood-workflow.svelte.ts`).
export { defaultChildhoodDraft };
export type { ChildhoodDraft };

const VALIDATE_DEBOUNCE_MS = 150;
// Numeric clamp helpers (I8_MIN/I8_MAX/…/clampInt/CORD_MAX) live in `./clamp` —
// shared with `aging-workflow.svelte.ts`, which needs the same rules-vs-serde
// bounds for the aging calculator's die/points fields.

/**
 * Save-format schema version written into every new entity. Exported so test
 * harnesses seed the current version instead of a literal that silently rots.
 *
 * Mirrors `arm_rules::SCHEMA_VERSION` by hand; the Rust constant is the source
 * and `the_frontend_mirrors_the_engine_schema_version` pins the two together.
 */
export const SCHEMA_VERSION = 21;

/**
 * The saga year a document starts at when nothing else says otherwise — the
 * published setting's own year.
 *
 * Mirrors `arm_rules::DEFAULT_SAGA_YEAR` by hand, exactly as {@link SCHEMA_VERSION}
 * mirrors the schema constant, and pinned to it by the Rust-side
 * `the_frontend_mirrors_the_engine_default_saga_year`. A mirror is needed because
 * {@link AppStore.entity}'s initial placeholder is built at module scope, before any
 * IPC call can have answered — and a required `saga_year` has to hold a real year
 * from that first instant. Every year the user can actually see comes from
 * `read_settings` or from the opened document; this is the value of last resort.
 */
export const DEFAULT_SAGA_YEAR = 1220;

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
function newEntity(
  rulesetId: string,
  version: string,
  typeId: string,
  sagaYear: number = DEFAULT_SAGA_YEAR,
): Entity {
  return {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: rulesetId, version },
    entity_kind: 'character',
    type_id: typeId,
    selections: [],
    characteristics: {},
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    // Written explicitly, never left to a default. A Tauri command deserializes the
    // payload with plain serde — `load_entity_migrating` does not run on an IPC
    // payload — so an absent key would read as `Pool` in Rust and quietly strip
    // every life-stage pool off a life-stage-funded character.
    ability_funding: 'pool',
    // Written explicitly for the same reason `ability_funding` is: the engine's
    // migration dispatches on this key's absence, and an IPC payload is
    // deserialized with plain serde, so an omitted key would read as the engine's
    // own 1220 and silently overwrite the saga the user configured.
    saga_year: sagaYear,
    art_scores: [],
    spells: [],
    house: null,
    mythic_type: null,
    age: null,
    personality_traits: [],
    reputations: [],
  };
}

// Where a character's Ability/Art experience comes from. Defined in `./types`
// alongside the `Entity` field that carries it (a store-side definition would make
// `types.ts` import the store), and re-exported here so the existing import path
// `import { type AbilityFunding } from '../state.svelte'` keeps working.
export type { AbilityFunding };

/**
 * Which palette the app paints with, in the order the settings control offers
 * them. `auto` follows the OS and is the default, so a fresh install is already
 * correct with nothing configured.
 *
 * **This array is the theme taxonomy's only home, and `auto` its only default.**
 * `arm-rules` does not and must not know what a palette is — it is a pure rules
 * engine, and a colour scheme is not an Ars Magica rule — and `arm-app` carries
 * the stored value as an opaque string precisely so it cannot become a second,
 * drifting copy of this list.
 */
export const THEMES = ['auto', 'light', 'dark'] as const;

/** @see THEMES */
export type Theme = (typeof THEMES)[number];

/** Whether `value` is a palette this build can actually paint. */
function isTheme(value: unknown): value is Theme {
  return THEMES.some((theme) => theme === value);
}

/** Whether `value` is a language this build ships a bundle for. */
function isLang(value: unknown): value is Lang {
  return AVAILABLE_LANGS.some((lang) => lang === value);
}

/**
 * Deliberately the LIGHT query and not the dark one. The app's own default is
 * dark, so an engine that does not know the feature reports `matches: false` and
 * lands exactly where an un-themed app lands, rather than flipping to a palette
 * nobody asked for.
 */
const PREFERS_LIGHT = '(prefers-color-scheme: light)';

class AppStore {
  lang = $state<Lang>('en');
  /** @see Theme */
  theme = $state<Theme>('auto');
  /**
   * The OS's colour-scheme preference, mirrored into reactive state by
   * {@link watchSystemTheme}. Written by that watcher and by nothing else.
   */
  osPrefersLight = $state(false);
  ruleset = $state<LocalizedRuleset | null>(null);
  // The startup screen's placeholder: no character exists yet, so its type is
  // empty. A real one is only ever built by {@link createCharacter} or a load.
  entity = $state<Entity>(newEntity('', '', ''));
  mode = $state<ValidationMode>('enforced');
  result = $state<ValidationResult | null>(null);
  effective = $state<EffectiveScores | null>(null);
  derived = $state<DerivedTotals | null>(null);
  /**
   * The entity snapshot the currently-published `effective`/`derived` payloads
   * were computed from — the *settled* character, as against the live one under
   * the cursor. Written only inside {@link revalidate}'s sequence guard, from the
   * very snapshot that was sent, so basis and payload can never drift apart, not
   * even when responses land out of order. Never written anywhere else.
   * @see readSettled
   */
  #effectiveBasis = $state<Entity | null>(null);
  error = $state<AppError | null>(null);
  loading = $state(false);
  // Per-picker filter/search state; persists across tab switches (see
  // {@link PickerFilters}). Not part of the entity, so it is never saved.
  filters = $state<PickerFilters>(defaultPickerFilters());

  /**
   * The Sample Childhood draft workflow, extracted to {@link ChildhoodWorkflow}
   * (VA6). It owns the draft/rejections state and `apply()`, which DOES replace
   * the whole `entity` on acceptance, routed through this `host` so `AppStore`
   * stays the sole owner of `entity`/`error`.
   */
  #childhoodWorkflow = new ChildhoodWorkflow({
    ruleset: () => this.ruleset,
    entity: () => this.entity,
    setEntity: (entity) => {
      this.entity = entity;
    },
    revalidate: () => this.revalidate(),
    setError: (error) => {
      this.error = error;
    },
  });

  /** @see ChildhoodWorkflow.draft */
  get childhoodDraft(): ChildhoodDraft {
    return this.#childhoodWorkflow.draft;
  }
  set childhoodDraft(draft: ChildhoodDraft) {
    this.#childhoodWorkflow.draft = draft;
  }

  /** @see ChildhoodWorkflow.rejections */
  get childhoodRejections(): ValidationIssue[] {
    return this.#childhoodWorkflow.rejections;
  }
  set childhoodRejections(rejections: ValidationIssue[]) {
    this.#childhoodWorkflow.rejections = rejections;
  }

  /**
   * The aging roll draft/preview/apply/revert workflow, extracted to
   * {@link AgingWorkflow} (VA6). It owns the draft/preview/notes/rejections
   * state and every method that touches them; `apply`/`revert` DO replace the
   * whole `entity` (the engine's own single writer for aging), routed back
   * through this `host` so `AppStore` stays the sole owner of `entity`/`error`.
   */
  #agingWorkflow = new AgingWorkflow({
    entity: () => this.entity,
    setEntity: (entity) => {
      this.entity = entity;
    },
    agingSchedule: () => this.effective?.aging?.schedule ?? [],
    revalidate: () => this.revalidate(),
    setError: (error) => {
      this.error = error;
    },
  });

  /** @see AgingWorkflow.draft */
  get agingDraft(): AgingDraft {
    return this.#agingWorkflow.draft;
  }
  set agingDraft(draft: AgingDraft) {
    this.#agingWorkflow.draft = draft;
  }

  /** @see AgingWorkflow.preview */
  get agingPreview(): AgingPreview {
    return this.#agingWorkflow.preview;
  }
  set agingPreview(preview: AgingPreview) {
    this.#agingWorkflow.preview = preview;
  }

  /** @see AgingWorkflow.notes */
  get agingNotes(): AgingNote[] {
    return this.#agingWorkflow.notes;
  }
  set agingNotes(notes: AgingNote[]) {
    this.#agingWorkflow.notes = notes;
  }

  /** @see AgingWorkflow.rejections */
  get agingRejections(): ValidationIssue[] {
    return this.#agingWorkflow.rejections;
  }
  set agingRejections(rejections: ValidationIssue[]) {
    this.#agingWorkflow.rejections = rejections;
  }

  /** @see AgingWorkflow.year */
  get agingYear(): number | null {
    return this.#agingWorkflow.year;
  }

  // --- Entity-mutator workflows -------------------------------------------
  //
  // The per-domain document editors, continuing the extraction the four
  // workflows above began (Viktor #7). Each owns one region of the entity and
  // holds no state of its own: they write in place through `host.entity()`,
  // which returns `AppStore`'s own `$state` object, so the store remains the
  // sole owner of the document and `dirty` still compares the very object these
  // mutators edit. The store's methods below are one-line delegations, exactly
  // as the aging ones are.
  //
  // **The extraction is NOT finished, and which domains have a module is not yet
  // a principle** (full-audit round 2, V6 — this comment previously claimed
  // otherwise). Roughly 860 further lines of entity mutators still live inline
  // below: Familiar, Talisman, magic items/devices/powers/focus powers, warping
  // and twilight scars, the aging log and living conditions, reputations,
  // personality traits, the longevity ritual, the life-stage plan and
  // characteristics. Several are larger than modules that were extracted —
  // `EquipmentWorkflow` is 58 lines and four one-line mutators; Familiar is
  // three times that and stayed. The boundary is therefore historical: it is
  // the list one review round happened to name, not a rule anyone can apply to
  // the next domain.
  //
  // The intended end state, for whoever continues: one module per domain until
  // `AppStore` holds only cross-domain state — `lang`, `theme`, `view`,
  // `entity`, `result`, `effective`, `derived`, `currentPath` — plus the
  // `dirty`/close-guard surface, which is a mandatory product behaviour and the
  // reason the co-location costs anything at all: every Familiar or Talisman
  // edit currently lands in the same file, and the same diff, as the
  // unsaved-changes guard.

  #abilityWorkflow = new AbilityWorkflow({
    entity: () => this.entity,
    ruleset: () => this.ruleset,
    scheduleValidate: () => this.#scheduleValidate(),
  });

  #artWorkflow = new ArtWorkflow({
    entity: () => this.entity,
    scheduleValidate: () => this.#scheduleValidate(),
  });

  #equipmentWorkflow = new EquipmentWorkflow({
    entity: () => this.entity,
    scheduleValidate: () => this.#scheduleValidate(),
  });

  #houseWorkflow = new HouseWorkflow({
    entity: () => this.entity,
    ruleset: () => this.ruleset,
    revalidate: () => this.revalidate(),
    scheduleValidate: () => this.#scheduleValidate(),
  });

  #mythicWorkflow = new MythicWorkflow({
    entity: () => this.entity,
    ruleset: () => this.ruleset,
    revalidate: () => this.revalidate(),
    scheduleValidate: () => this.#scheduleValidate(),
  });

  #selectionWorkflow = new SelectionWorkflow({
    entity: () => this.entity,
    ruleset: () => this.ruleset,
    effective: () => this.effective,
    scheduleValidate: () => this.#scheduleValidate(),
  });

  #spellWorkflow = new SpellWorkflow({
    entity: () => this.entity,
    ruleset: () => this.ruleset,
    scheduleValidate: () => this.#scheduleValidate(),
  });

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
   * Guided-wizard step navigation, extracted to {@link WizardNavigation}
   * (VA6) — it owns the step/furthest counters and every derived reading of
   * them, and only ever reads the entity/ruleset/result through this `host`.
   * `AppStore` stays the sole owner of `entity`/`ruleset`/`result` themselves.
   */
  #wizardNav = new WizardNavigation({
    ruleset: () => this.ruleset,
    entityTypeId: () => this.entity.type_id,
    result: () => this.result,
    phasesInForce: () => this.effective?.phases_in_force ?? null,
    // The one write the rail makes to the document (#31), and the reason a Next
    // dirties it. A slug, never the index it was resolved from: `creation_phases`
    // is ruleset data, so a position means nothing across builds.
    recordFurthestPhase: (phase) => {
      this.entity.wizard_furthest_phase = phase;
    },
  });

  /** @see WizardNavigation.step */
  get wizardStep(): number {
    return this.#wizardNav.step;
  }
  set wizardStep(step: number) {
    this.#wizardNav.step = step;
  }

  /** @see WizardNavigation.furthest */
  get wizardFurthest(): number {
    return this.#wizardNav.furthest;
  }
  set wizardFurthest(furthest: number) {
    this.#wizardNav.furthest = furthest;
  }

  /** @see WizardNavigation.phases */
  get wizardPhases() {
    return this.#wizardNav.phases;
  }

  /** @see WizardNavigation.phase */
  get wizardPhase() {
    return this.#wizardNav.phase;
  }

  /** @see WizardNavigation.canAdvance */
  get wizardCanAdvance() {
    return this.#wizardNav.canAdvance;
  }

  /** @see WizardNavigation.canFinish */
  get wizardCanFinish() {
    return this.#wizardNav.canFinish;
  }

  /** @see WizardNavigation.incompletePhases */
  get wizardIncompletePhases() {
    return this.#wizardNav.incompletePhases;
  }

  /**
   * Save/Save As/Export-Markdown and the discard-confirmation prompt,
   * extracted to {@link FileOperations} (VA6). `markSaved` is the only path
   * that writes {@link #savedSnapshot} from outside `AppStore`'s own body —
   * the callback closure is defined here, so the baseline keeps exactly one
   * owner even though a successful save is what triggers it.
   */
  #fileOps = new FileOperations({
    entity: () => this.entity,
    snapshot: () => this.#snapshot(),
    markSaved: (snapshot) => {
      this.#savedSnapshot = snapshot;
      // The moment the schema-migration notice's advice is consumed — its
      // closing clause reads "Saving will keep the rebuilt values", which is a
      // false statement about the future from here on (S3, full-audit round 2).
      // A cancelled save never reaches this callback, so the notice survives a
      // dialog the player backed out of.
      this.migratedAgingCharacteristics = [];
      this.unresolvedCatalogueParameters = [];
      this.migratedCatalogueParameters = [];
    },
    setError: (error) => {
      this.error = error;
    },
    ruleset: () => this.ruleset,
    t: (key, args) => this.t(key, args),
  });

  /** @see FileOperations.currentPath */
  get currentPath(): string | null {
    return this.#fileOps.currentPath;
  }
  set currentPath(path: string | null) {
    this.#fileOps.currentPath = path;
  }

  /** @see FileOperations.currentFileName */
  get currentFileName(): string | null {
    return this.#fileOps.currentFileName;
  }

  /** Whether a file operation (Save/Save As/Open) is running; disables the toolbar. */
  get busy(): boolean {
    return this.#fileOps.busy;
  }

  /**
   * Characteristics whose Aging Points the load-time schema migration rebuilt,
   * for the document currently open. Empty when the save needed no migration.
   *
   * Kept as the raw ids rather than the finished sentence so
   * {@link migrationNotice} re-composes in the new language when the user
   * switches locale — a notice frozen in the language it was opened in would be
   * a second place the UI language fails to reach.
   *
   * Cleared when the document changes (`open`, `newDocument`,
   * `#instantiateCharacter`) and, since S3, on a **successful save** — see the
   * `markSaved` callback above: that is the moment the notice's own closing
   * instruction becomes a statement about the past.
   */
  migratedAgingCharacteristics = $state<Characteristic[]>([]);

  /**
   * The localized "your Aging Points were rebuilt" notice for the open document,
   * or `null` when there is nothing to say.
   *
   * This matters because the migration is **lossy**: the engine reconstructs the
   * minimal Aging Point total that still reproduces the recorded scores, so the
   * original total is unrecoverable — and the next Save writes the
   * reconstruction back as the document's own figures. Telling the user nothing
   * would make that permanent silently.
   */
  get migrationNotice(): string | null {
    return agingMigrationNotice(this.migratedAgingCharacteristics, this.t);
  }

  /**
   * Parameterized Ability rows whose free-text value did NOT match any
   * catalogue entry's name in either locale, and so stayed free text (CV4b,
   * design § 5.5), for the document currently open. Empty when nothing was
   * left unresolved.
   *
   * Kept as `(ability, text)` rather than a finished sentence, exactly like
   * {@link migratedAgingCharacteristics}, so {@link unresolvedCatalogueNotice}
   * re-composes in the new language when the user switches locale.
   *
   * Cleared on the same lifecycle as {@link migratedAgingCharacteristics}:
   * `open`, `newDocument`, `#instantiateCharacter`, and a successful save.
   */
  unresolvedCatalogueParameters = $state<UnresolvedCatalogueParameter[]>([]);

  /**
   * The localized "these values could not be matched to a known entry" notice
   * for the open document, or `null` when there is nothing to say.
   *
   * This matters because § 4 rule 1's Literal-only matching means a
   * previously working-by-luck authorization or restricted-pool funding can
   * silently stop applying the moment a rename or a case/language mismatch
   * puts a value outside exact-string luck — the player is entitled to know
   * which rows are affected, not discover it as an unexplained validation
   * error later.
   */
  get unresolvedCatalogueNotice(): string | null {
    return unresolvedCatalogueParameterNotice(
      this.ruleset,
      this.unresolvedCatalogueParameters,
      this.t,
    );
  }

  /**
   * Parameterized Ability rows whose free-text value WAS recognized as a
   * catalogue entry's name and folded into `Catalogued` (CV4b, design § 5.5),
   * for the document currently open. Empty when nothing was recognized. The
   * positive counterpart to {@link unresolvedCatalogueParameters}.
   */
  migratedCatalogueParameters = $state<MigratedCatalogueParameter[]>([]);

  /**
   * The localized "these values were recognized and linked to their catalogue
   * entry" notice for the open document, or `null` when there is nothing to
   * say.
   */
  get migratedCatalogueNotice(): string | null {
    return migratedCatalogueParameterNotice(this.ruleset, this.migratedCatalogueParameters, this.t);
  }

  /**
   * How many documents this session has opened, counting the one it started
   * with — a monotonic token whose only meaningful property is that it CHANGES
   * whenever the open document is replaced.
   *
   * It exists because a component inside the editor is not recreated when that
   * happens (Sabine #1/#2, full-audit round 3). {@link open} leaves
   * `view === 'editor'` and never touches the active tab, so `App.svelte`'s
   * editor branch is unchanged and every instance inside it — and every piece of
   * per-document UI state it keeps locally — survives into the next character.
   * The two that were caught doing so are the aura clamp hint
   * (`MagicPossessions.svelte`, `DerivedAuraField.svelte`) and the familiar's and
   * talisman's unanswered remove confirmations (`FamiliarPanel.svelte`,
   * `TalismanPanel.svelte`); the second is an undo-free delete that would land on
   * a character the player never selected.
   *
   * ONE token rather than a flag per feature, deliberately: a store flag for each
   * would need clearing in all three replace routes every time one is added, and
   * that is the list this counter makes it impossible to forget. Components key
   * off it in an `$effect` and reset their own state — so the store never learns
   * what any of them are for.
   */
  get documentEpoch(): number {
    return this.#documentEpoch;
  }

  #documentEpoch = $state(0);

  /**
   * Whether a New/Open discard confirmation is on screen and unanswered —
   * the native dialog or, where there is none, the in-app fallback.
   * @see FileOperations.discardConfirmPending
   */
  get discardConfirmPending(): boolean {
    return this.#fileOps.discardConfirmPending;
  }

  /** Whether the in-app fallback discard modal is currently shown. */
  get discardPromptOpen(): boolean {
    return this.#fileOps.discardPromptOpen;
  }

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
   * Read off the entity's own stored `ability_funding` (schema 16) rather than held
   * in a separate `$state`, so a loaded save lands in its recorded mode with no
   * reconciliation code — and so the value the frontend sends to Rust is the value
   * Rust reads.
   *
   * It is **not** derived from `entity.life_stages` any more. It was until schema
   * 16, and that inference was the structural cause of #29: while a plan's presence
   * *is* the mode, discarding the plan is the only way to record "pool", so
   * switching mode had to destroy the side switched away from. With the mode stored,
   * both sides coexist and the inactive one is merely inert
   * (`LifeStageRules::budget` returns `None` under `Pool`, so nothing double-counts).
   *
   * The `?? 'pool'` fallback is not slack in the type — the field is required, and
   * every entity the store builds or loads carries it — it mirrors the engine's
   * `#[serde(default)]`, so a hand-edited file missing the key reads the same mode
   * on both sides of the IPC boundary instead of two different ones.
   */
  abilityFunding = $derived<AbilityFunding>(this.entity.ability_funding ?? 'pool');

  #bundle = $derived(buildBundle(this.lang));
  #timer: ReturnType<typeof setTimeout> | undefined;
  #seq = 0;
  // The aging preview's own debounce and sequence guard lives on
  // `AgingWorkflow` now (VA6) — deliberately separate from the validation pair
  // above, since the die is not an entity edit.
  // The exact error object the last rejected validate published to `error`, so a
  // later succeeding validate can tell its own banner apart from a file-operation
  // failure that landed in the same shared field. Not reactive: it never renders.
  #validateError: AppError | null = null;
  // The same idea for the ruleset load: the exact error object its last failure
  // published, so a succeeding load retires its own banner and no one else's.
  // Not reactive — it never renders. @see #reloadRuleset
  #rulesetError: AppError | null = null;

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

  /**
   * Whether `action` may run right now — the app's ONE answer to that question.
   *
   * Before the native menu (C3a) the answer existed twice: the toolbar buttons
   * carried `disabled={store.busy}`, and `App.svelte`'s window-level keydown
   * handler repeated the check by hand, because `inert` on the shell does not
   * reach a window listener. A menu item obeys neither mechanism — it is
   * neither a button in the shell nor a key event — so the choice was a third
   * copy or one predicate. This is the predicate: the shortcut handler, the
   * menu's enabled state ({@link menuFlags}) and {@link runDocumentAction} all
   * read it.
   *
   * C3c then removed the toolbar, so the two remaining callers are the menu and
   * the keyboard. That makes this the ONLY place the question is answered — and
   * the only place it can be asserted, since neither a native menu item's
   * enabled state nor a window key handler is anything a rendered string or
   * WebDriver can observe.
   *
   * `settings` is the one member that is not a *document* action at all: the
   * preferences belong to the app, not to the character, so unlike Save/Save
   * As/Export they are offered on the startup screen too. It rides here anyway
   * because the menu and the dispatcher must answer for it exactly once, and
   * `busy` gates it like everything else — nothing may open over a native file
   * dialog.
   */
  documentActionEnabled(action: DocumentAction): boolean {
    // A native file dialog is open: the shell is inert and the store actions
    // no-op, so nothing may start a second one.
    if (this.busy) return false;
    switch (action) {
      case 'new':
      case 'open':
        return true;
      case 'save':
      case 'saveAs':
      case 'export':
        // These write the document being edited. The startup screen has none —
        // only the typeless placeholder — so writing there would save a
        // character that does not exist.
        return this.view !== 'start';
      case 'settings':
        return true;
    }
  }

  /**
   * Run `action` if {@link documentActionEnabled} allows it, through the very
   * method that owns the file operation. Nothing here reimplements one.
   */
  async runDocumentAction(action: DocumentAction): Promise<void> {
    if (!this.documentActionEnabled(action)) return;
    switch (action) {
      case 'new':
        await this.newDocument();
        return;
      case 'open':
        await this.open();
        return;
      case 'save':
        await this.save();
        return;
      case 'saveAs':
        await this.saveAs();
        return;
      case 'export':
        await this.exportMarkdown();
        return;
      case 'settings':
        this.openSettings();
        return;
    }
  }

  /** {@link documentActionEnabled} in the shape the native menu takes it in. */
  menuFlags(): MenuFlags {
    return {
      new: this.documentActionEnabled('new'),
      open: this.documentActionEnabled('open'),
      save: this.documentActionEnabled('save'),
      saveAs: this.documentActionEnabled('saveAs'),
      export: this.documentActionEnabled('export'),
      settings: this.documentActionEnabled('settings'),
    };
  }

  /**
   * Read the persisted settings, then load the ruleset for the language they name
   * and validate the initial entity.
   *
   * **Sequential, and that is the whole point.** Until C4 these two ran in
   * `Promise.all`, which was correct precisely because neither needed the other: the
   * saga year is an app setting and the ruleset knows nothing of it. A persisted
   * LANGUAGE destroys that premise — rules display text is per-language, so the
   * language has to be known *before* the load. The two alternatives were both worse
   * than one extra await: loading in English and reloading afterwards would show a
   * German user a screen of English on first paint and then throw a whole ruleset
   * away, and leaving the order to chance would make which of the two happened a
   * race. The cost is one small local file read ahead of a much larger one, and
   * `read_settings` fetches all four settings in a single call precisely so it stays
   * one round trip rather than four.
   *
   * Deliberately here and not in an `$effect`: the settings are read once at launch,
   * and `onMount` already owns this call.
   */
  async init(): Promise<void> {
    await this.loadSettings();
    await this.#reloadRuleset(true);
  }

  async setLang(lang: Lang): Promise<void> {
    if (lang === this.lang) return;
    this.lang = lang;
    this.#persist({ lang });
    // Rules display text is per-language, so reload the ruleset too.
    await this.#reloadRuleset(false);
  }

  /**
   * Choose the palette (see {@link Theme}). Persisted, and it touches no document:
   * a colour scheme is an app preference, not character state.
   */
  async setTheme(theme: Theme): Promise<void> {
    this.theme = theme;
    this.#persist({ theme });
  }

  /**
   * Write the settings the patch names, and nothing else. The Rust side is
   * read-modify-write, so naming one key here cannot destroy another.
   *
   * A rejection is SURFACED, never swallowed: a preference that silently failed to
   * persist looks saved until the next launch contradicts it, which is worse than a
   * banner. Mirrors what {@link setSagaYear} already did.
   */
  #persist(patch: ipc.SettingsPatch): void {
    void ipc.writeSettings(patch).catch((e: unknown) => {
      this.error = e as AppError;
    });
  }

  /**
   * The palette `<html data-theme>` names — resolved, so never `auto`: the
   * stylesheet has a `:root` and a `:root[data-theme='light']` and no rule at
   * all for a theme that has not been decided yet.
   */
  get resolvedTheme(): 'light' | 'dark' {
    if (this.theme !== 'auto') return this.theme;
    return this.osPrefersLight ? 'light' : 'dark';
  }

  /**
   * Track the OS colour-scheme preference for as long as the returned teardown
   * has not been called.
   *
   * A ONE-SHOT read at startup is the tempting shape and the wrong one: the user
   * switching their desktop to light mid-session would leave this app dark for
   * the rest of the session, which is precisely what `auto` promises not to do.
   * So it subscribes — and hands back the unsubscribe rather than keeping it,
   * because a listener on a `MediaQueryList` outlives whatever registered it and
   * a leaked one keeps writing into a store the caller has finished with.
   *
   * Returns a no-op teardown where there is no media-query API at all: this
   * module is imported by the whole node-environment SSR test suite, and by the
   * app itself before the webview has run anything.
   */
  watchSystemTheme(): () => void {
    const query =
      typeof globalThis.matchMedia === 'function' ? globalThis.matchMedia(PREFERS_LIGHT) : null;
    if (query === null) return () => {};

    this.osPrefersLight = query.matches;
    const onPreferenceChanged = (event: MediaQueryListEvent): void => {
      this.osPrefersLight = event.matches;
    };
    query.addEventListener('change', onPreferenceChanged);
    return () => query.removeEventListener('change', onPreferenceChanged);
  }

  async setMode(mode: ValidationMode): Promise<void> {
    this.mode = mode;
    this.#persist({ validation_mode: mode });
    await this.revalidate();
  }

  /**
   * Whether the settings modal is on screen. App preferences, not a document
   * action's confirmation, so it is plain state rather than a pending promise —
   * closing it answers nothing and cancels nothing.
   */
  settingsOpen = $state(false);

  /** Show the settings modal. @see documentActionEnabled */
  openSettings(): void {
    this.settingsOpen = true;
  }

  /** Dismiss the settings modal. Every choice inside it took effect as it was made. */
  closeSettings(): void {
    this.settingsOpen = false;
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
    await this.#houseWorkflow.setHouse(house);
  }

  /**
   * Set the specialisation pick for one of the current House's grants, keyed by
   * the grant's `choice_key`.
   *
   * @see HouseWorkflow.setChoice
   */
  setHouseChoice(choiceKey: string, selection: Selection): void {
    this.#houseWorkflow.setChoice(choiceKey, selection);
  }

  /**
   * Select the Mythic Companion type (or clear it with `null`). A discrete
   * action, so it validates immediately. Switching auto-manages the type's
   * required package. Mirrors {@link setHouse}.
   *
   * @see MythicWorkflow.setType
   */
  async setMythicType(mythicType: string | null): Promise<void> {
    await this.#mythicWorkflow.setType(mythicType);
  }

  /**
   * Set a Mythic Companion type grant pick keyed by the grant's `choice_key`.
   * Mirrors {@link setHouseChoice}.
   *
   * @see MythicWorkflow.setChoice
   */
  setMythicChoice(choiceKey: string, selection: Selection): void {
    this.#mythicWorkflow.setChoice(choiceKey, selection);
  }

  /**
   * Swap a required Flaw for a "suitable substitute agreed with the troupe".
   *
   * @see MythicWorkflow.setRequiredFlaw
   */
  async setMythicRequiredFlaw(previousRef: string, nextRef: string): Promise<void> {
    await this.#mythicWorkflow.setRequiredFlaw(previousRef, nextRef);
  }

  /**
   * Add a virtue/flaw selection. A repeatable item — one carrying parameters
   * (e.g. Great Characteristic) or with `max_per_target > 1` — can be added
   * several times, each instance choosing its own target; a plain item is added
   * once.
   *
   * @see SelectionWorkflow.add
   */
  addSelection(ref: string): void {
    this.#selectionWorkflow.add(ref);
  }

  /**
   * Selection edits are by row index, since a repeatable item has several
   * rows.
   *
   * CV7 (design § 5.5, `docs/vf-audit/design-cv-catalogued-values.md`): a
   * removed Virtue/Flaw may be the source a bought Ability's parameter is
   * `Linked` to — the engine owns that conversion
   * (`arm_rules::unlink_ability_parameters`), so this calls it BEFORE
   * splicing the selection out, exactly as the load-time dangling-link fold
   * does. Never reimplemented here.
   *
   * Calling the engine is skipped when nothing currently offered as a link
   * target names this item — the common case (removing a plain Virtue/Flaw)
   * stays fully synchronous, so this never turns an ordinary removal into an
   * awaited round trip. `unlink_ability_parameters` is itself a correct
   * no-op when nothing is linked to `ref`, so this check is a latency
   * optimization, not a second copy of the engine's own decision.
   *
   * @see SelectionWorkflow.removeAt
   */
  async removeSelectionAt(index: number): Promise<void> {
    const ref = this.entity.selections?.[index]?.ref;
    if (ref && this.#isLinkSource(ref)) {
      const updated = await ipc.unlinkAbilityParameters(this.entity, ref);
      this.entity.ability_scores = updated.ability_scores;
    }
    this.#selectionWorkflow.removeAt(index);
  }

  /** Whether any currently-offered link target names `ref` as its source
   *  item (`AbilityParameterOptions.linked`, design § 6.3) — the cheap,
   *  engine-derived check {@link removeSelectionAt}/{@link setParamAt} use
   *  to decide whether the unlink round trip is worth making at all. */
  #isLinkSource(ref: string): boolean {
    return (this.effective?.ability_parameter_options ?? []).some((o) =>
      o.linked.some((lt) => lt.item === ref),
    );
  }

  /**
   * Set one parameter of one selection row. The value is trimmed, never
   * case-folded — parameter values decide a selection's identity.
   *
   * CV7: clearing a Bound-declaring item's own parameter back to empty is the
   * OTHER way a link's target can disappear (design § 5.5) — the same engine
   * operation runs first, converting any Linked Ability score to Text
   * holding the last resolvable value, before the empty value is applied.
   *
   * Only a transition TO empty calls the engine — a non-empty edit (a
   * rename) must keep a live link tracking the NEW value (D59 point 2), never
   * freeze it to the old one.
   *
   * @see SelectionWorkflow.setParamAt
   */
  async setParamAt(index: number, key: string, value: string): Promise<void> {
    const ref = this.entity.selections?.[index]?.ref;
    if (ref && value.trim() === '' && this.#isLinkSource(ref)) {
      const updated = await ipc.unlinkAbilityParameters(this.entity, ref);
      this.entity.ability_scores = updated.ability_scores;
    }
    this.#selectionWorkflow.setParamAt(index, key, value);
  }

  /**
   * Set a `multi_ref` parameter (D9 part 3, C5b) to the given set of values —
   * the picker's multi-select control hands over the WHOLE checked set on
   * every toggle, never one member at a time.
   *
   * @see SelectionWorkflow.setMultiParamAt
   */
  setMultiParamAt(index: number, key: string, values: Iterable<string>): void {
    this.#selectionWorkflow.setMultiParamAt(index, key, values);
  }

  /**
   * Point an ability-bonus selection (Puissant Ability) at a specific ability
   * *instance*.
   *
   * @see SelectionWorkflow.setAbilityBonusTarget
   */
  setAbilityBonusTarget(index: number, abilityId: string, parameter?: string | null): void {
    this.#selectionWorkflow.setAbilityBonusTarget(index, abilityId, parameter);
  }

  /** Set or clear a Characteristic score (score 0 removes the explicit entry). */
  setCharacteristic(characteristic: Characteristic, score: number): void {
    const chars = { ...(this.entity.characteristics ?? {}) };
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
   * XP — the first point is bought by raising it.
   *
   * @see AbilityWorkflow.add
   */
  addAbility(ability: string): void {
    this.#abilityWorkflow.add(ability);
  }

  /** Ability edits are by row index, since a parameterized ability has several rows.
   *  @see AbilityWorkflow.removeAt */
  removeAbilityAt(index: number): void {
    this.#abilityWorkflow.removeAt(index);
  }

  /**
   * "+" on the tab's unbought row (a granted floor or bonus, no bought entry):
   * buys the Ability raised one visible step, as one edit (R7).
   *
   * @see AbilityWorkflow.buyUnbought
   */
  raiseUnboughtAbility(ability: string, max: number): void {
    this.#abilityWorkflow.buyUnbought(ability, this.#abilityFloor(ability, undefined), max);
  }

  /**
   * "+" on a bought row: one visible step, so a row at or below its granted
   * floor jumps to floor + 1 (R7).
   *
   * @see AbilityWorkflow.raiseAt
   */
  raiseAbilityAt(index: number, max: number): void {
    const row = this.entity.ability_scores?.[index];
    if (!row) return;
    this.#abilityWorkflow.raiseAt(index, this.#abilityFloor(row.ability, row.parameter), max);
  }

  /** The granted floor on one Ability instance, from the last engine pass. */
  #abilityFloor(ability: string, parameter: AbilityParamValue | null | undefined): number {
    return abilityFloor(
      this.effective?.ability_score_floors ?? [],
      ability,
      parameter,
      resolvedLinksFrom(this.effective?.ability_parameter_options ?? []),
    );
  }

  /** @see AbilityWorkflow.adjustAt */
  adjustAbilityAt(index: number, delta: number, max: number): void {
    this.#abilityWorkflow.adjustAt(index, delta, max);
  }

  /** @see AbilityWorkflow.setSpecialtyAt */
  setAbilitySpecialtyAt(index: number, specialty: string): void {
    this.#abilityWorkflow.setSpecialtyAt(index, specialty);
  }

  /** @see AbilityWorkflow.setBankedXpAt */
  setAbilityBankedXpAt(index: number, xp: number): void {
    this.#abilityWorkflow.setBankedXpAt(index, xp);
  }

  /** @see AbilityWorkflow.setParameterAt */
  setAbilityParameterAt(index: number, value: string): void {
    this.#abilityWorkflow.setParameterAt(index, value);
  }

  /**
   * Writes a full parameter value chosen from the picker's combo box — a
   * catalogue entry or a link target (CV7). `undefined` clears it.
   *
   * @see AbilityWorkflow.setParameterValueAt
   */
  setAbilityParameterValueAt(index: number, value: AbilityParamValue | undefined): void {
    this.#abilityWorkflow.setParameterValueAt(index, value);
  }

  setXpPool(xp: number): void {
    this.entity.xp_pool = clampInt(xp, 0, U32_MAX);
    this.#scheduleValidate();
  }

  /**
   * Switch how Abilities are funded (see {@link abilityFunding}). A discrete
   * action, so it validates immediately like {@link setHouse}.
   *
   * **A pure mode set: it destroys no entity data in either direction.** It used to
   * destroy plenty — entering the guided mode zeroed a hand-typed `xp_pool`, and
   * leaving it deleted the whole `life_stages` plan (native language, Gauntlet age,
   * lab seasons, spell levels, childhood package) — so a round trip lost everything
   * typed on either side, unprompted and unrecoverable. That was not a policy but a
   * consequence of the mode being *inferred* from the plan's presence: discarding
   * the plan was the only way to record "pool". Schema 16 stores the mode, so both
   * sides now coexist on the entity and the inactive one is simply inert.
   *
   * That is a deliberate departure from this file's sparse-save habit: the save
   * keeps data the active mode ignores. Do not "fix" it back. Nothing double-counts
   * — `LifeStageRules::budget` returns `None` under pool funding, and the engine's
   * `life_stage_xp_pool_conflict` finding was retired with the invariant it asserted.
   *
   * Entering the guided mode still *creates* a plan when none exists, because the
   * plan is where every life-stage field is written ({@link setNativeLanguage} and
   * friends are no-ops without one). `??=`, never `=`: an existing plan is picked up
   * where it was left rather than restarted empty.
   *
   * Bought `ability_scores` survive either switch untouched, as they always did:
   * funding less experience than the rows demand is reported as `not_enough_xp` —
   * visible and fixable — which is strictly better than silently discarding work.
   *
   * **The drafts are still pruned on the way out**, on one blanket rule that
   * predates this change and outlives it: an un-submitted draft never outlives a
   * change to how the document is built. So leaving the guided mode clears the
   * {@link childhoodDraft}, its {@link childhoodRejections} and the
   * {@link agingDraft}. Deliberately asymmetric — *entering* guided mode keeps an
   * in-progress draft, since toggling the radio back and forth without ever leaving
   * would otherwise destroy typed slot values.
   */
  async setAbilityFunding(funding: AbilityFunding): Promise<void> {
    if (this.abilityFunding === funding) return;
    this.entity.ability_funding = funding;
    if (funding === 'life_stages') {
      this.entity.life_stages ??= {};
    } else {
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
   * reads blank as unset, and a sparse save is the canonical one. A no-op when no
   * plan exists, so no surface can conjure one into being: {@link setAbilityFunding}
   * (a manual switch) and {@link startWizard}'s default (M6/D2, via
   * `#instantiateCharacter`) are the only two places that allocate a plan — always
   * together with the mode, never on its own — so this setter never has to. Note
   * the guard is the plan's *presence*, not the funding mode — since schema 16 a
   * pool-funded character may legitimately carry one, and writing into an inert
   * plan is how its contents survive a mode switch rather than a bug.
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
   * A blank or zero age deletes the key, so the save keeps only what the player
   * actually chose. The engine then reads the ruleset's own baseline —
   * `apprenticeship.default_gauntlet_age`, clamped to the character's age — which is
   * the number `LifeStagePanel` shows as the field's placeholder. Deliberately NOT
   * prefilled here: writing a 25 into the plan would dirty the entity for a field the
   * player never touched, and a stored 25 against an age of 22 typed later would
   * raise a blocking `life_stage_gauntlet_age_after_age` the blank field never did.
   *
   * A no-op when no plan exists, like {@link setNativeLanguage}: no surface can
   * conjure one into being.
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
   *
   * A no-op when no plan exists, exactly like {@link setNativeLanguage}.
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
   * @see ChildhoodWorkflow.setDraftPackage
   */
  setChildhoodDraftPackage(packageId: string | null): void {
    this.#childhoodWorkflow.setDraftPackage(packageId);
  }

  /** @see ChildhoodWorkflow.setDraftSlot */
  setChildhoodDraftSlot(slot: string, value: string): void {
    this.#childhoodWorkflow.setDraftSlot(slot, value);
  }

  /**
   * Take the drafted Sample Childhood package: submit it with its slot answers and
   * keep whatever the engine decides. A no-op while no package is drafted — there is
   * nothing to submit.
   *
   * @see ChildhoodWorkflow.apply
   */
  async applyChildhoodPackage(): Promise<void> {
    await this.#childhoodWorkflow.apply();
  }

  /**
   * Pick which owed year the calculator is rolling for, or fall back to the
   * default with `null`. Draft state only (see {@link AgingDraft}).
   *
   * @see AgingWorkflow.setYear
   */
  setAgingYear(age: number | null): void {
    this.#agingWorkflow.setYear(age);
  }

  /** @see AgingWorkflow.setDie */
  setAgingDie(die: number | null): void {
    this.#agingWorkflow.setDie(die);
  }

  /** @see AgingWorkflow.setDistribution */
  setAgingDistribution(characteristic: Characteristic, points: number | null): void {
    this.#agingWorkflow.setDistribution(characteristic, points);
  }

  /** @see AgingWorkflow.setCrisisDie */
  setAgingCrisisDie(die: number | null): void {
    this.#agingWorkflow.setCrisisDie(die);
  }

  /** Abandon the drafted roll: the year, both dice, the points, the last refusal
   *  and whatever the last applied year had to say. @see AgingWorkflow.clear */
  clearAgingDraft(): void {
    this.#agingWorkflow.clear();
  }

  /**
   * Ask the engine what the drafted die makes of the drafted year, and keep the
   * answer in {@link agingPreview}. Writes nothing to the character.
   *
   * @see AgingWorkflow.runPreview
   */
  async previewAgingRoll(): Promise<void> {
    await this.#agingWorkflow.runPreview();
  }

  /**
   * Apply the drafted roll: the engine resolves the year, writes the Aging Points,
   * advances the apparent age and appends the log entry, all in one move
   * (`resolve_year` is the aging subsystem's single writer).
   *
   * @see AgingWorkflow.apply
   */
  async applyAgingRoll(): Promise<void> {
    await this.#agingWorkflow.apply();
  }

  /**
   * Take one recorded year back off, exactly — a pre-play catch-up of 25 rolls
   * with no undo would not be shippable. The engine subtracts precisely the points
   * the log entry recorded and removes the entry.
   *
   * @see AgingWorkflow.revert
   */
  async revertAgingRoll(age: number): Promise<void> {
    await this.#agingWorkflow.revert(age);
  }

  /**
   * Adjust an Art's bought score by `delta`, clamped to [0, max].
   *
   * @see ArtWorkflow.adjust
   */
  adjustArt(art: string, delta: number, max: number): void {
    this.#artWorkflow.adjust(art, delta, max);
  }

  /** @see ArtWorkflow.setBankedXp */
  setArtBankedXp(art: string, xp: number): void {
    this.#artWorkflow.setBankedXp(art, xp);
  }

  /**
   * Point an Art-domain parameter of a selection at a specific Art (by id).
   *
   * @see ArtWorkflow.setBonusTarget
   */
  setArtBonusTarget(index: number, key: string, artId: string): void {
    this.#artWorkflow.setBonusTarget(index, key, artId);
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
    this.#spellWorkflow.add(spellId, level, parameter);
  }

  /**
   * Add a spell already marked within the character's Magical Focus — the
   * picker's "add within focus" action (D81.5).
   *
   * @see SpellWorkflow.addWithinFocus
   */
  addSpellWithinFocus(spellId: string, level?: number | null, parameter?: string | null): void {
    this.#spellWorkflow.addWithinFocus(spellId, level, parameter);
  }

  /**
   * Set (or clear) the target Form of a parametrized spell at `index` — part of
   * the spell's identity, so distinct Forms are distinct instances.
   *
   * @see SpellWorkflow.setParameterAt
   */
  setSpellParameterAt(index: number, parameter: string | null): void {
    this.#spellWorkflow.setParameterAt(index, parameter);
  }

  /** @see SpellWorkflow.setWithinFocusAt */
  setSpellWithinFocusAt(index: number, value: boolean): void {
    this.#spellWorkflow.setWithinFocusAt(index, value);
  }

  /** @see SpellWorkflow.setWithinPotentFieldAt */
  setSpellWithinPotentFieldAt(index: number, value: boolean): void {
    this.#spellWorkflow.setWithinPotentFieldAt(index, value);
  }

  /**
   * Adjust the bought Spell Mastery score of the spell at `index` by `delta`,
   * clamped to [0, max].
   *
   * @see SpellWorkflow.adjustMasteryAt
   */
  adjustSpellMasteryAt(index: number, delta: number, max: number): void {
    this.#spellWorkflow.adjustMasteryAt(index, delta, max);
  }

  /**
   * Add a Spell Mastery special ability (a `spell_mastery_ability.*` id) to the
   * spell at `index`.
   *
   * @see SpellWorkflow.addMasteryAbilityAt
   */
  addMasteryAbilityAt(index: number, abilityId: string): void {
    this.#spellWorkflow.addMasteryAbilityAt(index, abilityId);
  }

  /**
   * Remove the mastery special ability at position `abilityIndex` within the
   * spell at `index`.
   *
   * @see SpellWorkflow.removeMasteryAbilityAt
   */
  removeMasteryAbilityAt(index: number, abilityIndex: number): void {
    this.#spellWorkflow.removeMasteryAbilityAt(index, abilityIndex);
  }

  /** Set the level of the (General) spell at `index`.
   *  @see SpellWorkflow.setLevelAt */
  setSpellLevelAt(index: number, level: number): void {
    this.#spellWorkflow.setLevelAt(index, level);
  }

  /** Spell edits are by row index, since a General spell can appear at several levels.
   *  @see SpellWorkflow.removeAt */
  removeSpellAt(index: number): void {
    this.#spellWorkflow.removeAt(index);
  }

  /**
   * Set (or clear) the per-character spell-levels budget override.
   *
   * @see SpellWorkflow.setLevelsOverride
   */
  setSpellLevelsOverride(levels: number | null): void {
    this.#spellWorkflow.setLevelsOverride(levels);
  }

  /**
   * Set (or clear) the character's age; drives the age → Ability-cap check, and
   * carries the birth year with it (see {@link sagaYear}).
   */
  setAge(age: number | null): void {
    this.entity.age =
      age != null && Number.isFinite(age) && age > 0 ? clampInt(age, 1, U32_MAX) : null;
    this.#deriveBirthYearFromAge();
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

  /**
   * Record a Reputation the character's V/F grant: kind and score come from the
   * grant, `content` from whatever the player has typed so far.
   *
   * Called lazily, on the first thing the player actually chooses in a granted
   * slot — the first keystroke of a description, or the type picked on a
   * wildcard (Famous) slot. The panel derives its rows from the grants, so
   * nothing is written merely because a grant exists; opening a character with
   * granted Reputations must not dirty it.
   */
  addReputation(kind: ReputationType, score: number, content = ''): void {
    this.entity.reputations = [...(this.entity.reputations ?? []), { kind, score, content }];
    this.#scheduleValidate();
  }

  removeReputationAt(index: number): void {
    this.entity.reputations = (this.entity.reputations ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  /**
   * Rewrite a stored Reputation's description — and **delete the row** when the
   * description is emptied, since an undescribed Reputation records nothing the
   * grant does not already say. Leaving it would also put an empty row into the
   * save, where `Entity::normalize`'s (kind, score, content) sort files it ahead
   * of every described one.
   */
  setReputationContent(index: number, content: string): void {
    if (content === '') {
      this.removeReputationAt(index);
      return;
    }
    this.entity.reputations = (this.entity.reputations ?? []).map((r, i) =>
      i === index ? { ...r, content } : r,
    );
    this.#scheduleValidate();
  }

  /**
   * Retype a stored Reputation. Only a wildcard grant (Famous fixes no type)
   * leaves the choice open, and that choice is the player's, so it persists on
   * its own without waiting for a description.
   */
  setReputationKind(index: number, kind: ReputationType): void {
    this.entity.reputations = (this.entity.reputations ?? []).map((r, i) =>
      i === index ? { ...r, kind } : r,
    );
    this.#scheduleValidate();
  }

  /**
   * Relevel a stored Reputation. Only a grant stating `max_score` (D11/Q5 —
   * Outsider alone, "a bad Reputation of level 1 to 3") offers a choice here;
   * every exact grant fixes the level and this is never called for it. Not
   * clamped in the store: the caller (the panel's stepper, bounded to
   * `[grant.score, grant.max_score]`) is the only legal caller, exactly like
   * `setReputationKind` trusts its own `<select>`'s option list.
   */
  setReputationScore(index: number, score: number): void {
    this.entity.reputations = (this.entity.reputations ?? []).map((r, i) =>
      i === index ? { ...r, score } : r,
    );
    this.#scheduleValidate();
  }

  // --- Magic Items tab: aura, devices, familiar, talisman, longevity ---

  /**
   * Set the realm aura modifier (signed; Divine can be a penalty).
   *
   * Clamped to the RULES range the engine surfaces
   * (`Ruleset.aura_modifier_min`/`max`), not to the serde width — the same bound
   * both aura inputs already render as their `min`/`max`
   * (`MagicPossessions.svelte`, `DerivedAuraField.svelte`), so the number cannot
   * disagree with the field it was typed into. i32 is not a constraint here: no
   * aura a player can plausibly type falls outside it, so clamping only to the
   * width meant an out-of-range value reached the engine intact and every derived
   * read-out was computed from it, until the next save+reload silently replaced
   * it with `Entity::normalize`'s own clamp — two different answers for one
   * document.
   *
   * The i32 fallback stays for a ruleset payload predating those fields: it
   * constrains nothing, but it keeps the value REPRESENTABLE, without which serde
   * rejects the whole payload at the Tauri boundary.
   */
  setAura(aura: number | null): void {
    const min = this.ruleset?.ruleset.aura_modifier_min ?? I32_MIN;
    const max = this.ruleset?.ruleset.aura_modifier_max ?? I32_MAX;
    this.entity.aura = aura == null ? 0 : clampInt(aura, min, max);
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

  /**
   * Levels spent one-for-one on the power's Penetration. Charged against the
   * SAME power-levels budget as the level itself ("You may also spend levels
   * one-for-one to give the power Penetration", ArMDE:4019), which is why
   * the engine's `power_levels_used` counts both — so this must be editable, or
   * the bar would charge for something no control can enter. Clamped to u16 like
   * the level.
   */
  setPowerPenetration(index: number, penetration: number): void {
    const clamped = clampInt(penetration, 0, U16_MAX);
    this.entity.powers = (this.entity.powers ?? []).map((p, i) =>
      i === index ? { ...p, penetration: clamped } : p,
    );
    this.#scheduleValidate();
  }

  // --- Focus Powers: the Focus Power Virtue's own 25-point pool ---
  //
  // A separate list from `powers` because the numbers are a separate currency:
  // "This Virtue grants a pool of 25 points… It costs 2 points to raise the
  // maximum level of effect by 1, and 1 point to raise the Penetration by 1"
  // (ArMDE:3899). `max_level` is a CEILING on what the character may create, not a
  // level that was spent, so it is never charged against `power_levels_used`.

  addFocusPower(): void {
    this.entity.focus_powers = [...(this.entity.focus_powers ?? []), { name: '', max_level: 0 }];
    this.#scheduleValidate();
  }

  removeFocusPowerAt(index: number): void {
    this.entity.focus_powers = (this.entity.focus_powers ?? []).filter((_, i) => i !== index);
    this.#scheduleValidate();
  }

  setFocusPowerName(index: number, name: string): void {
    this.entity.focus_powers = (this.entity.focus_powers ?? []).map((p, i) =>
      i === index ? { ...p, name } : p,
    );
    this.#scheduleValidate();
  }

  /** The maximum level of effect, at 2 points each. Clamped to u16 like a power level. */
  setFocusPowerMaxLevel(index: number, maxLevel: number): void {
    const clamped = clampInt(maxLevel, 0, U16_MAX);
    this.entity.focus_powers = (this.entity.focus_powers ?? []).map((p, i) =>
      i === index ? { ...p, max_level: clamped } : p,
    );
    this.#scheduleValidate();
  }

  /** The power's Penetration, at 1 point each out of the same pool. */
  setFocusPowerPenetration(index: number, penetration: number): void {
    const clamped = clampInt(penetration, 0, U16_MAX);
    this.entity.focus_powers = (this.entity.focus_powers ?? []).map((p, i) =>
      i === index ? { ...p, penetration: clamped } : p,
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
   * with each other" (ArMDE:16594), so several rows can hold at once. The
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

  /**
   * Take one aging-log row back off — the log's own undo, and the only thing the
   * row's × may mean.
   *
   * **An engine-recorded row goes through the engine.** Such a row is the record
   * of a year that placed Aging Points and may have advanced the apparent age, so
   * dropping the row alone leaves every one of those effects standing on the
   * character with nothing on the sheet left to explain them — silent corruption,
   * and a log that no longer says where the points came from. It is handed to
   * `aging::revert_year` (via {@link revertAgingRoll}), which subtracts exactly
   * what the entry recorded, steps the appearance back if that year moved it, and
   * removes the entry itself. That is the same path the calculator's "take back"
   * button uses, so the two controls cannot disagree about what an undo is.
   *
   * **The key is `age`, and nothing narrower.** `age` is precisely what
   * `aging.rs::revert_year` addresses entries by and what its `retain`
   * removes, so "carries an age" is the engine's own definition of a row within
   * its reach — a stricter test (a recorded die, say) would plain-filter a row the
   * engine would have reverted, which is the corruption again. A hand-written row
   * carries no age by construction (`addAgingLogEntry` writes only `year` and
   * `effect`), has nothing mechanical to undo, and is simply dropped here.
   *
   * The index is the UI's address; the age is the engine's. Reading the entry
   * first is what translates between them, so the clicked row is the one taken
   * back even when hand-written rows sit among the recorded ones.
   *
   * Refusals and failures are the calculator's: a rejected revert leaves the
   * character (and the row) untouched and lands in {@link agingRejections}; a
   * failed IPC call goes to the error banner. Neither claims an undo that did not
   * happen.
   */
  async removeAgingLogEntryAt(index: number): Promise<void> {
    const entry = (this.entity.aging_log ?? [])[index];
    if (entry == null) return;
    if (entry.age != null) {
      await this.revertAgingRoll(entry.age);
      return;
    }
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

  /** Add a carried equipment slot referencing a catalogue weapon/shield/armor id.
   *  @see EquipmentWorkflow.add */
  addEquipment(item: string): void {
    this.#equipmentWorkflow.add(item);
  }

  /** @see EquipmentWorkflow.removeAt */
  removeEquipmentAt(index: number): void {
    this.#equipmentWorkflow.removeAt(index);
  }

  /** @see EquipmentWorkflow.setLoadout */
  setEquipmentLoadout(index: number, loadout: LoadoutState): void {
    this.#equipmentWorkflow.setLoadout(index, loadout);
  }

  /** Toggle whether the weapon's Ability specialization applies (+1 Atk/Def).
   *  @see EquipmentWorkflow.setSpecialization */
  setEquipmentSpecialization(index: number, specialization_applies: boolean): void {
    this.#equipmentWorkflow.setSpecialization(index, specialization_applies);
  }

  /** Toggle whether the character is currently fighting mounted (K3): adds
   *  min(Ride, 3) to Attack/Defense on every non-body-attack Combat line. */
  setMounted(mounted: boolean): void {
    this.entity.mounted = mounted;
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
    this.#deriveAgeFromBirthYear();
    this.#scheduleValidate();
  }

  /** D42: the concept's optional default realm — a default SOURCE for every
   *  Supernatural entry's realm (ArMDE:2960), never the character's own realm
   *  and never mechanical on its own. `null` clears it back to unset. */
  setConceptRealm(realm: Realm | null): void {
    this.entity.concept_realm = realm;
    this.#scheduleValidate();
  }

  // --- The saga year, and age ↔ birth year as two views of one fact ----------
  //
  // Three values exist and only two can be authoritative. `age` and `birth_year`
  // are THE STORED PAIR — both already on the entity, both dirtying the document —
  // and the saga year is a reference for derivation only. Editing either half
  // recomputes the other; editing the saga year recomputes NOTHING and does not
  // dirty the document, because advancing a character by N years needs aging rolls,
  // Living Conditions and any Longevity Ritual applied per year. A silent recompute
  // would fabricate ages that skipped their aging rolls
  // (guided-creation-review-2026-08 #25 / D3.3).

  /**
   * The saga year a **new** document starts at — a preference, and the only part of
   * the saga year that is still machine-global.
   *
   * C8 split the one number in two. The saga year proper is document state
   * ({@link Entity.saga_year}): a storyguide runs a 1220 Rhine saga and a 1197
   * Iberia saga, and a single stored year was wrong for all but one of them — every
   * derived age with it. What remains here is the seed a fresh character is stamped
   * with, which is a genuine preference in the way language and appearance are.
   *
   * Seeded from {@link DEFAULT_SAGA_YEAR} and replaced by {@link loadSettings}; the
   * settings reader resolves it against the engine's own constant, so the year a
   * user sees is never one this file invented.
   */
  defaultSagaYear = $state<number>(DEFAULT_SAGA_YEAR);

  /**
   * Advisories from the age ↔ birth-year derivation — today only "the saga year is
   * before the birth year", which clamps the age to 0.
   *
   * Kept apart from {@link result} because it is not a reading of the entity: the
   * engine cannot emit it from `validate`, since the saga year never reaches it as
   * entity data. `ValidationPanel` shows both lists.
   */
  sagaIssues = $state<ValidationIssue[]>([]);

  // Monotonic guard for the derivation round trips. Typing a birth year fires one
  // per keystroke and the answers come back over IPC, so a slow early reply must
  // not land on top of a later one. Same shape as `#seq` for validation.
  #sagaSeq = 0;

  /**
   * Read every persisted setting and apply it. Called from {@link init} before the
   * ruleset load, because the language decides what that load returns.
   *
   * **Each stored value is checked before it is believed.** The settings file is
   * hand-editable, which makes it the trust boundary (CLAUDE.md): a `theme` of
   * `purple` names no palette and a `lang` of `fr` names no bundle, so taking either
   * at face value would paint an unstyled app or an untranslated one. An
   * unrecognised value falls back to this store's own default, which is where the
   * theme's and the language's defaults live.
   *
   * The saga year needs no such check — it arrives already resolved, because its
   * default is `arm_rules::DEFAULT_SAGA_YEAR`, a rules value the engine owns.
   *
   * Never fails loudly: the Rust command is infallible, so a rejection means the
   * bridge itself is gone, and the app starts on its defaults rather than refusing
   * to start over a preference.
   */
  async loadSettings(): Promise<void> {
    try {
      const stored = await ipc.readSettings();
      this.defaultSagaYear = stored.default_saga_year;
      if (isLang(stored.lang)) this.lang = stored.lang;
      if (isTheme(stored.theme)) this.theme = stored.theme;
      if (stored.validation_mode !== null) this.mode = stored.validation_mode;
    } catch {
      // See above: defaults stand, and the launch proceeds.
    }
  }

  /**
   * Set the year **this document's** saga stands in.
   *
   * It is stored state from C8 on, so it dirties the document — the year is part of
   * what an unsaved file would lose, and the unsaved-changes guard has to see it.
   * It still rewrites neither the age nor the birth year (D3.3): advancing a saga by
   * N years means an aging roll, Living Conditions and any Longevity Ritual applied
   * *per year*, so a silent recompute would fabricate ages that skipped their rolls.
   * It governs only what the next age or birth-year edit derives.
   */
  setSagaYear(year: number): void {
    if (!Number.isFinite(year)) return;
    this.entity.saga_year = clampInt(year, I32_MIN, I32_MAX);
    this.#scheduleValidate();
  }

  /**
   * Choose the saga year **new documents** start at. A preference, so it is
   * persisted to the settings file and touches neither the open document nor the
   * dirty flag — a character already built for another saga keeps its own year.
   */
  setDefaultSagaYear(year: number): void {
    if (!Number.isFinite(year)) return;
    const clamped = clampInt(year, I32_MIN, I32_MAX);
    this.defaultSagaYear = clamped;
    this.#persist({ default_saga_year: clamped });
  }

  /**
   * Fill in the age from the birth year just typed. The engine owns the arithmetic
   * and the clamp, so the impossible-pair advisory has exactly one wording and the
   * frontend states no policy of its own.
   */
  #deriveAgeFromBirthYear(): void {
    const sagaYear = this.entity.saga_year;
    const birthYear = this.entity.birth_year;
    if (sagaYear == null || birthYear == null) {
      // An emptied field derives nothing — the same treatment `setBirthYear` gives
      // it, rather than dating the character to year 0.
      this.sagaIssues = [];
      return;
    }
    const seq = ++this.#sagaSeq;
    void ipc
      .deriveAge(sagaYear, birthYear)
      .then((derived) => {
        if (seq !== this.#sagaSeq) return;
        // Assigned straight to the entity, not through `setAge`: that setter reads a
        // non-positive number as "field cleared", while a clamped 0 here is the
        // derived answer and has to survive.
        this.entity.age = derived.age;
        this.sagaIssues = derived.issues;
        this.#scheduleValidate();
      })
      .catch(() => {
        // Leave the typed birth year standing; the age simply does not follow.
      });
  }

  /** The other direction: the birth year follows the age just typed. */
  #deriveBirthYearFromAge(): void {
    const sagaYear = this.entity.saga_year;
    const age = this.entity.age;
    if (sagaYear == null || age == null) {
      this.sagaIssues = [];
      return;
    }
    const seq = ++this.#sagaSeq;
    void ipc
      .deriveBirthYear(sagaYear, age)
      .then((year) => {
        if (seq !== this.#sagaSeq) return;
        this.entity.birth_year = year;
        // Setting the age makes the pair consistent by construction, so whatever the
        // other direction advised no longer holds.
        this.sagaIssues = [];
        this.#scheduleValidate();
      })
      .catch(() => {});
  }

  /**
   * Save to the current file. A never-saved document (no {@link currentPath})
   * falls back to {@link saveAs} so the user picks a destination; otherwise it
   * writes straight to the tracked file with no prompt (standard document-app
   * behavior). No-op while another file operation is in flight.
   *
   * @see FileOperations.save
   */
  async save(): Promise<void> {
    await this.#fileOps.save();
  }

  /**
   * Always prompt for a destination and, on success, adopt it as the current
   * file. A cancelled prompt (null path) leaves the current file untouched and
   * the document dirty. No-op while another file operation is in flight.
   *
   * @see FileOperations.saveAs
   */
  async saveAs(): Promise<void> {
    await this.#fileOps.saveAs();
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
   *
   * @see FileOperations.exportMarkdown
   */
  async exportMarkdown(): Promise<void> {
    await this.#fileOps.exportMarkdown();
  }

  /**
   * Open a document from a file. Prompts to discard first when the current
   * document has unsaved edits; a cancelled prompt aborts without loading. On
   * success the opened file becomes the current file. No-op while another file
   * operation is in flight.
   *
   * Stays on `AppStore` rather than moving into {@link FileOperations}: unlike
   * Save/Export it resets several axes at once (entity, wizard rail, picker
   * filters, childhood and aging drafts, view) that belong to other modules or to
   * `AppStore` itself, so it borrows only the busy flag and the discard prompt from
   * `#fileOps` rather than folding those resets into that module.
   *
   * Returns whether a document was actually loaded, so {@link openIntoWizard} can
   * tell a cancelled dialog (or a failed read) from a successful open instead of
   * entering the wizard on the character that was already there.
   */
  async open(): Promise<boolean> {
    if (this.#fileOps.busy || this.discardConfirmPending) return false;
    if (this.dirty && !(await this.#fileOps.confirmDiscard())) return false;
    this.#fileOps.busy = true;
    this.error = null;
    try {
      const loaded = await ipc.loadEntity();
      if (loaded) {
        this.entity = loaded.entity;
        this.currentPath = loaded.path;
        // What the load-time schema migration had to rebuild, if anything. Set
        // before the snapshot for no reason but order-of-reading; it is view
        // state about the load, never part of the document.
        this.migratedAgingCharacteristics = loaded.migrated_aging_characteristics ?? [];
        this.unresolvedCatalogueParameters = loaded.unresolved_catalogued_parameters ?? [];
        this.migratedCatalogueParameters = loaded.migrated_catalogued_parameters ?? [];
        // A different document from here on, so the components that keep
        // per-document UI state locally retire theirs (@see documentEpoch).
        // Bumped only on a SUCCESSFUL load: a cancelled dialog replaced nothing.
        this.#documentEpoch += 1;
        this.#savedSnapshot = this.#snapshot();
        // And the picker filters, which are per-document exactly as they are in
        // `newDocument()` and `#instantiateCharacter()` — Open was the one
        // replacement route that let them through (Sabine 3, full-audit round 4).
        // `derivedArtPicker` is the case that decides it: it is not a filter but a
        // SELECTION, naming the Technique/Form pair the Derived tab computes Lab
        // and Casting Totals for, so carrying it over shows the new character's
        // totals for a pair chosen for the previous one.
        this.filters = defaultPickerFilters();
        // A loaded character's recorded childhood package is history, not a draft:
        // its slot answers already live in its Ability rows. Starting the draft
        // empty is what keeps that coherent — nothing pre-fills a package whose
        // slots are gone, so no applied package can show a spurious empty slot.
        this.childhoodDraft = defaultChildhoodDraft();
        // Likewise the aging draft: a die typed against the year the *previous*
        // character owed says nothing about this one, and the years it already
        // recorded arrive in its own log.
        this.clearAgingDraft();
        // And the saga-year advisory, which was about the previous character's pair.
        // Nothing is re-derived for this one: a character built in a 1220 saga and
        // opened under a 1230 setting keeps both stored values (#25).
        this.sagaIssues = [];
        // Opening is reachable from any screen, so a load always lands in the
        // editor; a cancelled dialog leaves the current screen alone. A save
        // records no wizard progress, so an opened character is a finished
        // document and never resumes mid-flow — hence the rail reset.
        this.view = 'editor';
        this.#resetWizardNav();
        await this.revalidate();
        return true;
      }
    } catch (e) {
      this.error = e as AppError;
    } finally {
      this.#fileOps.busy = false;
    }
    return false;
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
    if (this.#fileOps.busy || this.discardConfirmPending) return;
    if (this.dirty && !(await this.#fileOps.confirmDiscard())) return;
    const { id, version } = this.ruleset?.ruleset ?? this.entity.ruleset;
    this.entity = newEntity(id, version, '', this.defaultSagaYear);
    this.view = 'start';
    this.#resetWizardNav();
    this.currentPath = null;
    // The migration notice describes the document being discarded, so it goes
    // with it — leaving it up over a new character would report a rewrite that
    // never happened to it.
    this.migratedAgingCharacteristics = [];
    this.unresolvedCatalogueParameters = [];
    this.migratedCatalogueParameters = [];
    // Likewise every component's own per-document state (@see documentEpoch).
    this.#documentEpoch += 1;
    this.filters = defaultPickerFilters();
    this.childhoodDraft = defaultChildhoodDraft();
    this.clearAgingDraft();
    this.result = null;
    this.sagaIssues = [];
    this.effective = null;
    this.derived = null;
    // The basis goes with the payloads it describes; leaving the outgoing
    // character's scores behind would hold badges over a document that no longer
    // exists (#16).
    this.#effectiveBasis = null;
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
   *
   * `abilityFunding` defaults to `'pool'` — {@link newEntity}'s own default,
   * which {@link createCharacter} takes as-is. {@link startWizard} passes
   * `'life_stages'` instead (owner request, M6/D2): direct entry is the "I
   * already know the numbers" mode, where a hand-typed total is the point, while
   * the guided wizard is where the rules-faithful, life-history construction
   * belongs.
   *
   * Life-stage funding always carries an allocated (possibly empty) plan, so this
   * allocates one here too, exactly as {@link setAbilityFunding} does for a manual
   * switch — every engine read of the plan (`LifeStageRules::budget`,
   * `validate_life_stage_plan`) is gated on the plan's mere *presence*, so an age
   * typed on `concept` before the player ever opens `experience` would otherwise
   * raise no `life_stage_age_before_gauntlet`/`life_stage_age_unset` finding at
   * all. An *empty* plan is not itself a choice, though: `completeness.rs`'s
   * `Experience` criterion reads the plan's *content*, not its mere presence, so
   * this does not mark the step engaged before the player has done anything (see
   * `phase_is_engaged`).
   */
  #instantiateCharacter(typeId: string, abilityFunding: AbilityFunding = 'pool'): void {
    const { id, version } = this.ruleset?.ruleset ?? this.entity.ruleset;
    this.entity = newEntity(id, version, typeId, this.defaultSagaYear);
    this.entity.ability_funding = abilityFunding;
    if (abilityFunding === 'life_stages') {
      this.entity.life_stages = {};
    }
    this.entity.selections = [...this.#mandatoryTraitRefs(typeId)].map((ref) => ({ ref }));
    this.currentPath = null;
    // Goes with the outgoing document, exactly as in `newDocument()`.
    this.migratedAgingCharacteristics = [];
    this.unresolvedCatalogueParameters = [];
    this.migratedCatalogueParameters = [];
    // As does every component's own per-document state (@see documentEpoch).
    this.#documentEpoch += 1;
    this.filters = defaultPickerFilters();
    this.childhoodDraft = defaultChildhoodDraft();
    this.clearAgingDraft();
    this.result = null;
    this.sagaIssues = [];
    this.effective = null;
    this.derived = null;
    // As in `newDocument`: the basis is cleared with the payloads it describes (#16).
    this.#effectiveBasis = null;
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
   *
   * Defaults to life-stage funding, unlike {@link createCharacter} — see
   * {@link #instantiateCharacter}.
   */
  async startWizard(typeId: string): Promise<void> {
    this.#instantiateCharacter(typeId, 'life_stages');
    this.view = 'wizard';
    this.#resetWizardNav();
    await this.revalidate();
  }

  /**
   * Whether the character in hand can be walked through the guided flow at all
   * (#31): the loaded ruleset must declare a profile for its type, because the
   * profile's `creation_phases` ARE the wizard's steps. A save from another
   * ruleset may name a type this build knows nothing about, and a wizard with no
   * rail is not a screen to enter — so the action is not offered rather than
   * offered and then broken.
   */
  get canEnterWizard(): boolean {
    return this.ruleset?.ruleset.type_profiles[this.entity.type_id] !== undefined;
  }

  /**
   * Walk the character already in hand through the guided flow, resuming from the
   * furthest phase its document recorded — the second entry point (#31), and the
   * one that skips instantiation: nothing about the character changes, only which
   * screen it is edited on.
   *
   * A no-op when {@link canEnterWizard} is false, which is what keeps a save from
   * an unknown type out of a wizard with no steps.
   */
  enterWizard(): void {
    if (!this.canEnterWizard) return;
    this.view = 'wizard';
    this.#wizardNav.restore(this.entity.wizard_furthest_phase);
  }

  /**
   * Open a document from a file and land it in the guided flow rather than the
   * editor — {@link open} followed by {@link enterWizard}.
   *
   * Composed rather than a second load path, so the discard prompt, the draft
   * resets and the immediate revalidation all stay in one place. A cancelled
   * dialog loads nothing and therefore enters nothing, and a file naming a type
   * this ruleset has no profile for stays in the editor `open` left it in.
   */
  async openIntoWizard(): Promise<void> {
    if (!(await this.open())) return;
    this.enterWizard();
  }

  /** Advance one step, unless the current phase holds an error. */
  wizardNext(): void {
    this.#wizardNav.next();
  }

  /**
   * Step back one. Never gated: the user must always be able to reach the step
   * that needs fixing, including the one they have just broken.
   */
  wizardBack(): void {
    this.#wizardNav.back();
  }

  /**
   * Jump to an already-visited step from the rail, in either direction. Never
   * clamped by an error (#9a); a step never reached is refused.
   * @see WizardNavigation.goTo
   */
  wizardGoTo(step: number): void {
    this.#wizardNav.goTo(step);
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
    // The guided run is over, so the record of how far it got stops being true and
    // goes with it (#31). Leaving it would gate a *completed* character on reopen:
    // it would take the restored branch, and if the player had since introduced an
    // error in the editor, Finish would stay shut over it — although that character
    // already finished once. The ungated branch is for a character that is not
    // mid-run, and a finished one is not. Nothing is lost: `wizardCanFinish`
    // guarantees this character already reached every step with no error anywhere.
    delete this.entity.wizard_furthest_phase;
    this.#resetWizardNav();
  }

  /** Send the rail back to the first step. */
  #resetWizardNav(): void {
    this.#wizardNav.reset();
  }

  /** Answer the open discard prompt (called by the modal's buttons).
   *  @see FileOperations.resolveDiscardPrompt */
  resolveDiscardPrompt(discard: boolean): void {
    this.#fileOps.resolveDiscardPrompt(discard);
  }

  /**
   * Read a value out of the entity the current `effective`/`derived` payloads
   * describe, rather than out of the live entity.
   *
   * The one fix for manual-testing finding #16 (2026-09-03), and the reason it is
   * here rather than in each component: an effective-score badge is a *pair* — a bought score
   * plus an engine-computed modifier — but only one half of that pair moves
   * synchronously. `adjustArt`/`adjustAbilityAt`/`setCharacteristic` mutate the
   * entity at once, while the modifier behind it is a debounce plus an IPC round
   * trip behind, so a badge built from the live score renders `newScore +
   * oldModifier`: a value that is true of no character, for ~150 ms, once per
   * keystroke. Reading the score through here pins both halves to the same
   * generation, so the badge makes exactly one transition per committed edit —
   * holding its previous value while the recompute is open, never a blend.
   *
   * Cheap by construction: the basis is the snapshot {@link revalidate} already
   * takes, so retaining it costs one reference, and a read is `read`'s own lookup
   * with no comparison against the live entity.
   *
   * Falls back to the live entity until the first pass has settled — there is no
   * previous value to hold then, and `effective` is still null at that point, so
   * every badge is hidden by its own "no modifier" gate anyway.
   */
  readSettled<T>(read: (entity: Entity) => T): T {
    return read(this.#effectiveBasis ?? this.entity);
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
        // In lockstep with the payloads, inside the same guard and from the same
        // snapshot that produced them (#16): a basis published outside this guard —
        // eagerly at call time, or from the live entity on arrival — would pair a
        // superseded response's numbers with someone else's scores, which is the
        // very blend the basis exists to prevent.
        this.#effectiveBasis = snapshot;
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
    // Retire the error THIS path last raised, and nothing else. `error` is one
    // shared banner — the menu build, the file operations and validation all write
    // to it — so an unconditional clear here erases a failure that has nothing to do
    // with the ruleset. It used to be harmless by accident: `init()` started the
    // load synchronously, so the clear ran before anything else could raise
    // anything. C4 put the settings read in front of it, which moved the clear a
    // microtask later — straight over `App.svelte`'s menu-build effect, whose
    // `AppError::Menu` would then vanish with no diagnostic. Same rule as
    // {@link revalidate}: a succeeding pass may retire only its own error.
    if (this.error === this.#rulesetError) this.error = null;
    this.#rulesetError = null;
    try {
      const localized = await ipc.loadRuleset(this.lang);
      this.ruleset = localized;
      if (resetEntity) {
        const { id, version } = localized.ruleset;
        this.entity = newEntity(id, version, '', this.defaultSagaYear);
        // A fresh entity is a clean baseline. A language reload keeps the
        // edited entity, so it must NOT reset the baseline — doing so would
        // drop `dirty` to false while unsaved edits still exist.
        this.#savedSnapshot = this.#snapshot();
      }
      // A language reload deliberately touches NOTHING on the entity — S1
      // (full-audit round 2). It used to re-stamp `entity.ruleset` with the
      // freshly loaded identity, which rewrites the same two values in the
      // ordinary case and changes something in exactly one: when the document's
      // recorded ruleset differs from the loaded one, which is precisely what
      // `validation/mod.rs::validate_ruleset_identity` exists to report. That
      // erased the warning with no user action addressing it, made the save's
      // provenance field a false claim, and dirtied a document nobody edited.
      // Adopting a different ruleset is a decision, not a side effect of
      // choosing a display language; a loaded entity carries its own identity
      // and a fresh one is stamped above.
      await this.revalidate();
    } catch (e) {
      this.#rulesetError = e as AppError;
      this.error = this.#rulesetError;
    } finally {
      this.loading = false;
    }
  }
}

export const store = new AppStore();
