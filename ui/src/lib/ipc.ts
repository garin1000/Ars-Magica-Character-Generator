// Typed wrappers around the Tauri command bridge. Every backend call goes
// through here so the rest of the app never touches `invoke` directly.

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { MENU_ACTION_EVENT, type MenuFlags, type MenuLabels } from './menu';
import type { MenuShortcut } from './menu-shortcuts';
import type {
  AgeInSagaYear,
  Characteristic,
  CrisisOutcome,
  DerivedTotals,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
  ValidationIssue,
  ValidationMode,
  ValidationResult,
} from './types';

export function loadRuleset(lang: string): Promise<LocalizedRuleset> {
  return invoke('load_ruleset', { lang });
}

export function validateEntity(entity: Entity, mode: ValidationMode): Promise<ValidationResult> {
  return invoke('validate_entity', { entity, mode });
}

export function effectiveScores(entity: Entity): Promise<EffectiveScores> {
  return invoke('effective_scores', { entity });
}

export function derivedTotals(entity: Entity): Promise<DerivedTotals> {
  return invoke('derived_totals', { entity });
}

/**
 * The settings that survive a restart, as `arm_app::settings::Settings` reports
 * them. Read from an app-settings file `arm-app` owns, NOT from the character and
 * not from the ruleset: none of the four is character state and none is a rule.
 *
 * Only `default_saga_year` arrives resolved, because its default is a rules value the
 * engine owns (`arm_rules::DEFAULT_SAGA_YEAR`). The other three are `null` when never
 * chosen, so their defaults stay in the single place each already lives — the store.
 *
 * `default_saga_year` was `saga_year` until C8, when the saga year itself became
 * document state. The Rust side reads the old key through a serde `alias`, so a
 * settings file written by an earlier build keeps the year its owner chose.
 */
export interface PersistedSettings {
  default_saga_year: number;
  lang: string | null;
  theme: string | null;
  validation_mode: ValidationMode | null;
}

/**
 * A partial update: exactly the settings being changed. The Rust side is
 * read-modify-write, so a patch naming one key cannot destroy another — which is
 * what makes choosing one setting safe now that the file holds four things.
 */
export interface SettingsPatch {
  default_saga_year?: number;
  lang?: string;
  theme?: string;
  validation_mode?: ValidationMode;
}

/**
 * Every persisted setting, in ONE call. Deliberately not four: the language decides
 * which ruleset text is loaded, so it has to be known before the load rather than
 * beside it, and a single small read is what makes serializing the two affordable.
 *
 * Infallible on the Rust side, so a first launch with no settings file simply
 * reports "nothing chosen" with the engine's default saga year.
 */
export function readSettings(): Promise<PersistedSettings> {
  return invoke('read_settings');
}

/** Persist the settings the patch names, leaving every other key alone. */
export function writeSettings(patch: SettingsPatch): Promise<void> {
  return invoke('write_settings', { patch });
}

/**
 * How old a character born in `birthYear` is in `sagaYear`, plus any advisory the
 * pair warrants — a saga year before the birth year clamps the age to 0 rather than
 * underflowing the entity's unsigned `age`.
 *
 * Asked of the engine rather than computed here: the clamp policy and the finding it
 * emits have one home, and the frontend computes no mechanics of its own. `maxAge` is
 * the ruleset's app maximum age (`null` when it states none); the age clamps at it.
 */
export function deriveAge(
  sagaYear: number,
  birthYear: number,
  maxAge: number | null,
): Promise<AgeInSagaYear> {
  return invoke('derive_age', { sagaYear, birthYear, maxAge });
}

/** Which year a character aged `age` in `sagaYear` was born in. */
export function deriveBirthYear(sagaYear: number, age: number): Promise<number> {
  return invoke('derive_birth_year', { sagaYear, age });
}

/**
 * Write the entity to disk. `path === null` prompts (Save As / first Save);
 * a concrete path writes directly with no dialog. Resolves to the written path,
 * or `null` when a prompt was cancelled.
 */
export function saveEntity(entity: Entity, path: string | null): Promise<string | null> {
  return invoke('save_entity', { entity, path });
}

/**
 * Write the entity as a Markdown character sheet. `labels` is the document chrome
 * the engine prints — `{ Fluent message name -> resolved text }`, so no
 * user-facing string lives in Rust. `path === null` prompts for a destination.
 * `currentPath` is the document's own save file, used only to prefill that prompt's
 * name and directory — never as a write target.
 * Resolves to the written path, or `null` when a prompt was cancelled.
 */
export function exportMarkdown(
  entity: Entity,
  labels: Record<string, string>,
  path: string | null = null,
  currentPath: string | null = null,
): Promise<string | null> {
  return invoke('export_markdown', { entity, labels, path, currentPath });
}

/**
 * The document-chrome label keys the Markdown export needs. The engine owns the
 * list, so the frontend resolves exactly those rather than keeping a copy.
 */
export function exportLabelKeys(): Promise<string[]> {
  return invoke('export_label_keys');
}

/**
 * The outcome of taking a Sample Childhood package: either the rewritten entity,
 * or the reasons the package could not be taken.
 *
 * A rejection is an ordinary outcome, not an error: an unanswered slot is a finding
 * about the form the player just submitted, so the engine reports it as localizable
 * `ValidationIssue`s (rendered through the same `issue-<code>` path as any other
 * finding) rather than failing the command.
 */
export type ChildhoodApplication =
  | { status: 'applied'; entity: Entity }
  | { status: 'rejected'; issues: ValidationIssue[] };

/**
 * Take the Sample Childhood package `packageId`, answering its parameter slots with
 * `slotValues` (slot key -> the player's value). The engine owns every decision:
 * what the package writes, that the write is a monotone raise, and what makes it
 * impossible.
 */
export function applyChildhoodPackage(
  entity: Entity,
  packageId: string,
  slotValues: Record<string, string>,
): Promise<ChildhoodApplication> {
  return invoke('apply_childhood_package', { entity, packageId, slotValues });
}

/**
 * Converts every bought Ability score `Linked` to `removedItem`'s own
 * parameter into free text holding its last resolvable value (design § 5.5,
 * `docs/vf-audit/design-cv-catalogued-values.md`), returning the whole
 * updated entity. The Virtue/Flaw removal flow calls this BEFORE the
 * selection is actually removed — the engine owns the conversion; the
 * frontend must not reimplement it.
 */
export function unlinkAbilityParameters(entity: Entity, removedItem: string): Promise<Entity> {
  return invoke('unlink_ability_parameters', { entity, removedItem });
}

/**
 * Where one aging row's points go. `kind` is the question to ask: the table names
 * the Characteristic itself, the player picks it, or the points must reach the next
 * level of Decrepitude. `characteristic` is present only for `named`.
 */
export type AgingPointTarget =
  | { kind: 'named'; characteristic: Characteristic }
  | { kind: 'player_choice' }
  | { kind: 'next_decrepitude_level' };

/**
 * One award an aging row makes. `points` is null only when the next Decrepitude
 * level lies beyond the advancement table — reported as unpriceable, never
 * silently costed at zero.
 */
export interface AgingPointAward {
  target: AgingPointTarget;
  points: number | null;
}

/** The Living Conditions modifier, split into the two places it comes from. */
export interface LivingConditionsModifier {
  rows: string[];
  from_table: number;
  from_traits: number;
  total: number;
}

/**
 * One year's AGING TOTAL with every term that made it, so the calculator shows the
 * arithmetic instead of a bare number. The two modifiers carry the book's own sign
 * and are SUBTRACTED; the trait modifier is ADDED with its stored sign.
 */
export interface AgingTotal {
  age: number;
  die: number;
  age_modifier: number;
  living_conditions: LivingConditionsModifier;
  longevity_bonus: number;
  trait_modifier: number;
  uncapped_total: number;
  total: number;
  // Whether a Longevity Ritual's under-35 ceiling cut THIS roll down.
  capped_by_longevity: boolean;
}

/** What the aging table does at a total. A crisis is flagged here, never resolved. */
export interface AgingOutcome {
  total: number;
  apparent_age_increases: boolean;
  awards: AgingPointAward[];
  crisis: boolean;
}

/**
 * One Crisis's CRISIS TOTAL with every term that made it — "Simple die + age/10
 * (round up) + Decrepitude Score" (ArMDE:16621). All three are ADDED, and
 * the Decrepitude is the one the crisis year itself raised (`ArMDE:16619`), never
 * today's.
 */
export interface CrisisTotal {
  age: number;
  die: number;
  age_modifier: number;
  decrepitude_score: number;
  total: number;
}

/**
 * Where one crisis-survival modifier comes from. A tagged union rather than a
 * string so each case can be named its own way: a Virtue resolves through the item
 * catalogue by `item` id, the familiar's bronze cord (`ArMDE:10844`) through Fluent.
 */
export type CrisisModifierSource = { kind: 'trait'; item: string } | { kind: 'bronze_cord' };

/** One modifier to the crisis survival roll, ADDED with its stored sign. */
export interface CrisisModifier {
  source: CrisisModifierSource;
  amount: number;
}

/**
 * Something the rules PERMIT at a crisis, as opposed to a number the engine adds:
 * the attending doctor of `ArMDE:16634`, whose Medicine belongs to a character this
 * sheet does not hold. `botch_penalty` is stored signed and added, like every other
 * aging modifier.
 */
export type CrisisAllowance = {
  kind: 'attendant';
  ability: string;
  characteristic: Characteristic;
  ease_factor: number;
  botch_penalty: number;
};

/**
 * What surviving one Crisis would take, and what the character brings to it
 * (`ArMDE:16628-16638`). A read-out: the engine never throws the Stamina die and never
 * pronounces a character dead.
 *
 * `ease_factor` is absent for the Terminal row, which offers no roll at all
 * (`ArMDE:16632`) — not an unbeatable one. The modifiers are itemized *and* summed
 * because the panel has to name each one; a character carrying no bronze cord shows
 * no cord line rather than a +0.
 */
export interface CrisisSurvival {
  ease_factor?: number | null;
  ritual_level: number;
  modifiers: CrisisModifier[];
  modifier_total: number;
  allowances: CrisisAllowance[];
}

/**
 * One Crisis read whole: the total, the row it landed on, what that row costs, and
 * what surviving it would take. `row` is an id — its text lives in
 * `rules/i18n/<lang>/aging.json` — and `survival` is absent for a bedridden row,
 * which is time rather than a roll (`ArMDE:16626, :16627`).
 */
export interface CrisisPreview {
  total: CrisisTotal;
  row: string;
  outcome: CrisisOutcome;
  survival?: CrisisSurvival | null;
}

/**
 * Something a resolved year has to TELL the player, as opposed to something it
 * writes. Rendered through `aging-note-<kind>`, never as a raw tag.
 *
 * Both of today's follow the Crisis rather than the Crisis roll, so an unrolled
 * Crisis carries them: the Longevity Ritual it spends without deleting (`ArMDE:16573`),
 * and the Heavy Wound a leper takes "in addition to any other result" (`ArMDE:6340`)
 * that the app records nowhere, because the health track is the player's.
 */
export type AgingNote = { kind: 'longevity_ritual_spent' } | { kind: 'heavy_wound' };

/**
 * The outcome of previewing, applying or reverting one aging roll.
 *
 * A refusal is an ordinary outcome, not an error — a die typed against a year
 * already rolled is a finding about the form the player just submitted — so the
 * engine reports it as localizable `ValidationIssue`s, rendered through the same
 * `issue-<code>` path as any other finding. Shaped exactly like
 * `ChildhoodApplication`.
 */
export type AgingProjection =
  | {
      status: 'previewed';
      total: AgingTotal;
      outcome: AgingOutcome;
      crisis?: CrisisPreview | null;
    }
  | { status: 'rejected'; issues: ValidationIssue[] };

export type AgingApplication =
  | {
      status: 'applied';
      entity: Entity;
      total: AgingTotal;
      outcome: AgingOutcome;
      crisis?: CrisisPreview | null;
      notes?: AgingNote[];
    }
  | { status: 'rejected'; issues: ValidationIssue[] };

export type AgingReversion =
  | { status: 'reverted'; entity: Entity }
  | { status: 'rejected'; issues: ValidationIssue[] };

/**
 * Read one year's aging roll without writing anything: the total the typed `die`
 * makes at `age`, and the row it lands on. The die stays out of the entity, which
 * is what makes "the calculator does not dirty the document" mechanically true.
 *
 * `distribution` and `crisisDie` are the very arguments {@link agingApply} takes,
 * because the engine answers them by resolving the year in memory and throwing the
 * character away: the Aging Points a Crisis row awards ARE the Decrepitude increase
 * `ArMDE:16619` puts first, so a Crisis read off the character as it stands would be one
 * short of the one Apply writes.
 */
export function agingPreview(
  entity: Entity,
  age: number,
  die: number,
  distribution: Partial<Record<Characteristic, number>>,
  crisisDie: number | null,
): Promise<AgingProjection> {
  return invoke('aging_preview', { entity, age, die, distribution, crisisDie });
}

/**
 * Apply one year's aging roll. `distribution` places the Aging Points the row left
 * to the player, per Characteristic; it is empty for a row that names its own.
 * `crisisDie` is the Simple Die thrown at the Crisis Table (`ArMDE:16621`), or `null`
 * for a Crisis nobody has rolled yet — which the year records as owed and unrolled
 * rather than refusing.
 */
export function agingApply(
  entity: Entity,
  age: number,
  die: number,
  distribution: Partial<Record<Characteristic, number>>,
  crisisDie: number | null,
): Promise<AgingApplication> {
  return invoke('aging_apply', { entity, age, die, distribution, crisisDie });
}

/** Take one applied aging year back off, exactly. */
export function agingRevert(entity: Entity, age: number): Promise<AgingReversion> {
  return invoke('aging_revert', { entity, age });
}

/**
 * An opened document: the entity, the file it was read from, and what a schema
 * migration had to rewrite on the way in. Mirrors `commands.rs::OpenedDocument`
 * field for field (named for it, so the two ends of the wire are searchable as
 * one thing).
 */
export interface OpenedDocument {
  path: string;
  entity: Entity;
  /**
   * Characteristics whose Aging Points a schema migration rebuilt — empty for a
   * save that needed none. The rebuild is **lossy** (the engine reconstructs the
   * minimal total that still reproduces the recorded scores) and the next Save
   * writes it back as the document's own figures, so a non-empty array is
   * something to tell the user about: `derive.ts::agingMigrationNotice` composes
   * the localized sentence from it.
   *
   * Required, because Rust always serializes it — empty array included. It was
   * briefly optional so the `loadEntity` test doubles need not spell out a field
   * they were not exercising, but an optional field here misdescribed the wire:
   * `undefined` is a shape the backend cannot produce, and typing it as possible
   * invites a consumer to branch on a case that never occurs.
   */
  migrated_aging_characteristics: Characteristic[];
  /**
   * Parameterized Ability rows whose free-text value did NOT match any
   * catalogue entry's name in either locale, and so stayed free text (CV4b,
   * design § 5.5) — absent/empty for a save with nothing unresolved.
   *
   * A previously working-by-luck authorization or restricted-pool funding can
   * silently stop applying once a Literal instance is satisfied only by a
   * `Catalogued` value, so this must reach the player. The Ability travels as
   * its own id, never a sentence: `derive.ts::unresolvedCatalogueParameterNotice`
   * resolves the localized name from it, exactly as
   * `migrated_aging_characteristics` does for its own case.
   *
   * Optional, unlike its sibling above: this field is new, and test doubles
   * built before it existed should not all need updating to spell out a case
   * they are not exercising.
   */
  unresolved_catalogued_parameters?: UnresolvedCatalogueParameter[];
  /**
   * Parameterized Ability rows whose free-text value WAS recognized as a
   * catalogue entry's name and folded into `Catalogued` (CV4b, design § 5.5)
   * — a positive counterpart to {@link OpenedDocument.unresolved_catalogued_parameters}.
   */
  migrated_catalogued_parameters?: MigratedCatalogueParameter[];
  /**
   * Ability instances a pre-22 save stored under the sibling Ability whose
   * catalogue no longer holds their value, moved by the load (L1b, try-out
   * finding 6). Ids and scores only; `derive.ts::movedAbilityParameterNotice`
   * composes the one-time notice. Optional for the same reason as its CV4b
   * siblings: older test doubles need not spell it out.
   */
  moved_ability_parameters?: MovedAbilityParameter[];
}

/**
 * One Ability instance the load moved to the sibling Ability whose catalogue
 * holds its value. Mirrors `migration.rs::MovedAbilityParameter`:
 * `existing_score` is present only on a collision, and is the score of the
 * instance that already held `value` under `to` (the higher one was kept).
 */
export interface MovedAbilityParameter {
  from: string;
  to: string;
  value: string;
  score: number;
  existing_score?: number;
}

/**
 * One parameterized Ability instance whose stored value did not match any
 * catalogue entry's name in either locale, so it stayed free text. Mirrors
 * `commands.rs::UnresolvedCatalogueParameter`.
 */
export interface UnresolvedCatalogueParameter {
  ability: string;
  text: string;
}

/**
 * One parameterized Ability instance whose free-text value WAS recognized as
 * a catalogue entry's name and folded into `Catalogued`. Mirrors
 * `commands.rs::MigratedCatalogueParameter`.
 */
export interface MigratedCatalogueParameter {
  ability: string;
  text: string;
  resolved: string;
}

export function loadEntity(): Promise<OpenedDocument | null> {
  return invoke('load_entity');
}

/** Localized strings for the "discard unsaved changes?" dialog, shown from Rust. */
export interface CloseGuardLabels {
  title: string;
  message: string;
  discard: string;
  cancel: string;
}

/**
 * Mirror the frontend's dirty flag (and the localized dialog strings) to the
 * backend close/quit guard, which owns the actual confirmation prompt so it can
 * intercept every quit path — window close and macOS Cmd+Q alike.
 */
export function updateCloseGuard(dirty: boolean, labels: CloseGuardLabels): Promise<void> {
  return invoke('update_close_guard', { dirty, labels });
}

/**
 * Ask the backend to confirm discarding unsaved changes before New or Open
 * replaces the document — the SAME native dialog the close/quit guard shows, so
 * the app confirms a discard exactly one way whatever triggered it. `labels`
 * carries every word, resolved from Fluent here, exactly as
 * {@link updateCloseGuard} does.
 *
 * Resolves `true` to discard, `false` to cancel, and **`null` when this build
 * offers no native confirmation** — the C3b test seam. A native dialog is
 * un-dismissable by WebDriver, so the `e2e-testing` build declines and the
 * caller falls back to the in-app prompt the specs can click
 * (see `crates/arm-app/src/commands.rs`'s `native_discard_confirmation_enabled`).
 */
export function confirmDiscard(labels: CloseGuardLabels): Promise<boolean | null> {
  return invoke('confirm_discard', { labels });
}

/**
 * Install (or reinstall) the native application menu.
 *
 * Rust owns the menu's shape and platform placement; `labels` carries every
 * word of it, resolved from Fluent here, so no menu text is authored in Rust —
 * the same division `updateCloseGuard` uses. `flags` mirrors the store's single
 * document-action predicate, which the menu can consult no other way.
 *
 * Called again on every language switch and on every change of that predicate:
 * a menu built once keeps the language and the enabled state it was built with.
 */
export function setAppMenu(labels: MenuLabels, flags: MenuFlags): Promise<void> {
  return invoke('set_app_menu', { labels, flags });
}

/**
 * The menu chords the webview must answer to itself: the menu's own
 * accelerators on Windows, where WebView2 keeps the keys from the menu, and
 * `[]` on every other desktop (U4). See `menu-shortcuts.ts`.
 */
export function menuShortcuts(): Promise<MenuShortcut[]> {
  return invoke('menu_shortcuts');
}

/**
 * Subscribe to menu clicks. Rust handles the native event and forwards the
 * item's id; the frontend owns the document, so it owns what the id means.
 * Resolves to the unsubscribe.
 */
export function onMenuAction(handler: (id: string) => void): Promise<UnlistenFn> {
  return listen<string>(MENU_ACTION_EVENT, (event) => handler(event.payload));
}
