// The magus's Arts: the bought Technique/Form scores, and pointing an
// Art-domain selection parameter (Puissant Art and friends) at one of them.
// Extracted out of `AppStore` (VA6 pattern), which keeps the composition root
// and the dirty/close-guard surface.
//
// Every edit is an in-place write to the host's own `entity` — the module never
// holds a copy, so `AppStore` stays the sole owner of the document and `dirty`
// keeps comparing the very object these methods mutate.

import type { Entity } from './types';

/** The slice of `AppStore` the Art workflow needs, as live accessors so it
 * always reads/writes the host's *current* state — `entity` stays owned by
 * `AppStore` alone, never copied here. */
export interface ArtWorkflowHost {
  entity: () => Entity;
  /** The host's debounced validation pass, shared with every other picker edit. */
  scheduleValidate: () => void;
}

export class ArtWorkflow {
  #host: ArtWorkflowHost;

  constructor(host: ArtWorkflowHost) {
    this.#host = host;
  }

  /**
   * Adjust an Art's bought score by `delta`, clamped to [0, max]. All 15 Arts are
   * always present for a magus, so an Art is addressed by id (not a row index)
   * and upserted: the score is stored only while non-zero (score 0 is the default
   * and is dropped to keep saves sparse and canonical).
   */
  adjust(art: string, delta: number, max: number): void {
    const entity = this.#host.entity();
    const scores = entity.art_scores ?? [];
    const current = scores.find((a) => a.art === art)?.score ?? 0;
    const next = Math.max(0, Math.min(max, current + delta));
    if (next === 0) {
      entity.art_scores = scores.filter((a) => a.art !== art);
    } else if (scores.some((a) => a.art === art)) {
      entity.art_scores = scores.map((a) => (a.art === art ? { ...a, score: next } : a));
    } else {
      entity.art_scores = [...scores, { art, score: next }];
    }
    this.#host.scheduleValidate();
  }

  /**
   * Point an Art-domain parameter of a selection at a specific Art (by id).
   * `key` is the *declaring* parameter's key — usually `art` (Puissant Art), but
   * an Art-domain parameter may be keyed otherwise (Master of (Form) Creatures
   * declares `form` over the Art catalogue), and the value must land under the
   * key the item declared or the engine reports it missing.
   *
   * Trimmed like every other parameter write path. An Art id comes from a
   * `<select>` and so is already canonical — this is defence in depth, kept only so
   * that no write path is the odd one out.
   */
  setBonusTarget(index: number, key: string, artId: string): void {
    const trimmed = artId.trim();
    const entity = this.#host.entity();
    entity.selections = (entity.selections ?? []).map((s, i) =>
      i === index ? { ...s, params: { ...(s.params ?? {}), [key]: trimmed } } : s,
    );
    this.#host.scheduleValidate();
  }
}
