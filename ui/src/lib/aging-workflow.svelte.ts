// The in-progress aging-roll calculator: draft (year/dice/point placement),
// preview, apply, revert. Extracted out of `AppStore` (VA6). The die is player
// input the engine never sees stored, so the draft/preview live only here and
// never touch the entity — except `apply`/`revert`, which DO replace the whole
// entity wholesale (the engine's own single writer for the aging subsystem),
// routed through the `host` so `AppStore` stays the sole owner of `entity`.

import { I32_MAX, U8_MAX, clampInt } from './clamp';
import * as ipc from './ipc';
import type { AgingApplication, AgingNote, AgingOutcome, AgingTotal, CrisisPreview } from './ipc';
import type { AppError, Characteristic, AgingScheduleYear, Entity, ValidationIssue } from './types';

const AGING_PREVIEW_DEBOUNCE_MS = 150;

/**
 * The aging roll the player is working on: which owed year it is for, the stress
 * die they typed, and where they are placing the Aging Points the table left to
 * them.
 *
 * UI-only state, exactly like `ChildhoodDraft` and for the same reason — only
 * more strictly. The die is not a choice the character records: the engine
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

/** The slice of `AppStore` the aging workflow needs, as live accessors so it
 * always reads/writes the host's *current* state — `entity` in particular
 * stays owned by `AppStore` alone, never copied here. */
export interface AgingWorkflowHost {
  entity: () => Entity;
  setEntity: (entity: Entity) => void;
  /** The character's aging schedule (from `effective.aging.schedule`), or an
   * empty array for a ruleset/character with no aging rules loaded yet. */
  agingSchedule: () => AgingScheduleYear[];
  revalidate: () => Promise<void>;
  setError: (error: AppError | null) => void;
}

/** The engine's reading of a drafted roll (see {@link AgingWorkflow.preview}). */
export type AgingPreview = {
  total: AgingTotal;
  outcome: AgingOutcome;
  /**
   * The Crisis the year would send the character to, once the row demands one,
   * the Simple Die is typed and the Aging Points are placed — the engine reads
   * it off the year it would apply, so what is shown is what Apply writes.
   */
  crisis?: CrisisPreview | null;
} | null;

export class AgingWorkflow {
  #host: AgingWorkflowHost;
  #timer: ReturnType<typeof setTimeout> | undefined;
  #seq = 0;

  constructor(host: AgingWorkflowHost) {
    this.#host = host;
  }

  /**
   * The in-progress aging roll (see {@link AgingDraft}). UI state: never part
   * of the entity, so it is never saved and typing a die never dirties the
   * document.
   */
  draft = $state<AgingDraft>(defaultAgingDraft());

  /**
   * The engine's answer for the drafted roll: the AGING TOTAL with every term
   * that made it, and the row it lands on. `null` until a year and a die are both
   * given (and while the debounced round trip is still out).
   *
   * Held rather than derived because it is the *engine's* reading — a stress die
   * explodes, so no bounded lookup table in JS could stand in for it, and
   * re-deriving the outcome here would be a second implementation of the table.
   */
  preview = $state<AgingPreview>(null);

  /**
   * What the year just applied had to TELL the player, as opposed to what it
   * wrote — today only the Longevity Ritual a Crisis spends (`:16573`), which the
   * entity cannot show because it deliberately keeps the stored choice.
   *
   * Cleared whenever the draft is, and by the next apply or revert: a note about
   * a year the player has since taken back would be a lie about the character in
   * front of them.
   */
  notes = $state<AgingNote[]>([]);

  /**
   * Why the last preview, apply or revert was refused, for the calculator to
   * render. Empty when there is nothing to say.
   */
  rejections = $state<ValidationIssue[]>([]);

  /**
   * The owed year the calculator is on: the player's own pick, or the first year
   * the aging log does not yet record.
   *
   * A plain getter, not `$derived`: it reads `this.#host`, which (like
   * `WizardNavigation`) is only assigned in the constructor body, after every
   * class-field initializer has already run. Deferring the read to call time
   * keeps it reactive without hitting that ordering trap — see
   * `wizard-navigation.svelte.ts` for the same fix with the fuller explanation.
   */
  get year(): number | null {
    return this.draft.age ?? this.#host.agingSchedule().find((year) => !year.recorded)?.age ?? null;
  }

  /**
   * Pick which owed year the calculator is rolling for, or fall back to the
   * default with `null`.
   *
   * Drops the point distribution: it was placed against the award the *other*
   * year's roll produced, and carrying it over would let a player apply points
   * they never re-confirmed. The typed die survives, since it is the number the
   * player has in front of them either way.
   */
  setYear(age: number | null): void {
    this.rejections = [];
    this.draft = { ...this.draft, age, distribution: {} };
    this.#schedulePreview();
  }

  /**
   * Record the stress die the player rolled, or clear it with `null`.
   *
   * "AGING TOTAL: Stress die (no botch) + age/10 (round up) …"
   * Source: Ars Magica - Definitive Edition (Core Rules).md:16567
   *
   * A stress die explodes, so the value has a floor of 0 and no ceiling; it is
   * clamped only to what the command's `i32` can carry.
   */
  setDie(die: number | null): void {
    this.rejections = [];
    const value = die != null && Number.isFinite(die) ? clampInt(die, 0, I32_MAX) : null;
    this.draft = { ...this.draft, die: value, distribution: {} };
    this.#schedulePreview();
  }

  /**
   * Place (or, with 0, un-place) Aging Points in one Characteristic.
   *
   * "If an Aging Point 'in any Characteristic' is gained, the player may choose
   * the Characteristic." Source: Ars Magica - Definitive Edition (Core
   * Rules).md:16615 — and `:16602`/`:16611` say "in any Characteristic**s**",
   * plural, so this is a map and not a single pick.
   */
  setDistribution(characteristic: Characteristic, points: number | null): void {
    this.rejections = [];
    const distribution = { ...this.draft.distribution };
    if (points != null && Number.isFinite(points) && points > 0) {
      distribution[characteristic] = clampInt(points, 1, U8_MAX);
    } else {
      delete distribution[characteristic];
    }
    this.draft = { ...this.draft, distribution };
    // Placing the points moves the CRISIS TOTAL, because those points ARE the
    // Decrepitude increase `:16619` puts first — so the reading is asked for
    // again rather than left standing at a number the year will not write.
    this.#schedulePreview();
  }

  /**
   * Record the Simple Die the player threw at the Crisis Table, or clear it with
   * `null`.
   *
   * "CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score"
   * Source: Ars Magica - Definitive Edition (Core Rules).md:16621
   */
  setCrisisDie(die: number | null): void {
    this.rejections = [];
    const value = die != null && Number.isFinite(die) ? clampInt(die, 0, I32_MAX) : null;
    this.draft = { ...this.draft, crisisDie: value };
    this.#schedulePreview();
  }

  /** Abandon the drafted roll: the year, both dice, the points, the last refusal
   *  and whatever the last applied year had to say. */
  clear(): void {
    clearTimeout(this.#timer);
    this.#timer = undefined;
    this.#seq++;
    this.draft = defaultAgingDraft();
    this.preview = null;
    this.rejections = [];
    this.notes = [];
  }

  /**
   * Ask the engine what the drafted die makes of the drafted year, and keep the
   * answer in {@link preview}. Writes nothing to the character.
   *
   * Guarded by its own sequence counter, the `revalidate` idiom: the die field
   * is typed into, so several previews can be in flight and they may finish out
   * of order — a stale answer must never overwrite a newer one, or the player
   * sees an outcome for a die they have already changed.
   */
  async runPreview(): Promise<void> {
    const seq = ++this.#seq;
    const { age, die, distribution, crisisDie } = this.draft;
    const year = age ?? this.year;
    if (year == null || die == null) {
      this.preview = null;
      return;
    }
    try {
      const projection = await ipc.agingPreview(
        $state.snapshot(this.#host.entity()),
        year,
        die,
        $state.snapshot(distribution),
        crisisDie,
      );
      if (seq !== this.#seq) return;
      if (projection.status === 'rejected') {
        this.preview = null;
        this.rejections = projection.issues;
        return;
      }
      this.preview = {
        total: projection.total,
        outcome: projection.outcome,
        crisis: projection.crisis ?? null,
      };
      this.rejections = [];
    } catch (e) {
      if (seq === this.#seq) this.#host.setError(e as AppError);
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
   * character is untouched and the findings land in {@link rejections}.
   */
  async apply(): Promise<void> {
    const { die, distribution, crisisDie } = this.draft;
    const year = this.year;
    if (year == null || die == null) return;
    this.rejections = [];
    this.notes = [];
    try {
      const application: AgingApplication = await ipc.agingApply(
        $state.snapshot(this.#host.entity()),
        year,
        die,
        $state.snapshot(distribution),
        crisisDie,
      );
      if (application.status === 'rejected') {
        this.rejections = application.issues;
        return;
      }
      this.#host.setEntity(application.entity);
      this.draft = defaultAgingDraft();
      this.preview = null;
      // Kept past the draft it came from: the note is about the character now on
      // screen, not about the form that has just been spent.
      this.notes = application.notes ?? [];
      await this.#host.revalidate();
    } catch (e) {
      this.#host.setError(e as AppError);
    }
  }

  /**
   * Take one recorded year back off, exactly — a pre-play catch-up of 25 rolls
   * with no undo would not be shippable. The engine subtracts precisely the points
   * the log entry recorded and removes the entry.
   */
  async revert(age: number): Promise<void> {
    this.rejections = [];
    // The year is going away, so what it had to say goes with it.
    this.notes = [];
    try {
      const reversion = await ipc.agingRevert($state.snapshot(this.#host.entity()), age);
      if (reversion.status === 'rejected') {
        this.rejections = reversion.issues;
        return;
      }
      this.#host.setEntity(reversion.entity);
      await this.#host.revalidate();
    } catch (e) {
      this.#host.setError(e as AppError);
    }
  }

  #schedulePreview(): void {
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.runPreview(), AGING_PREVIEW_DEBOUNCE_MS);
  }
}
