// Inclusive ranges of the fixed-width Rust integer fields the entity's numbers
// land in. A value outside its field's range makes serde reject the whole payload
// at the Tauri boundary, which fails `validate`, `effective_scores` and
// `derived_totals` at once — leaving every read-out frozen on stale numbers that
// still look current. Mutators clamp instead, so the engine always gets a
// representable value and the panel inputs carry the matching min/max.
export const I8_MIN = -128;
export const I8_MAX = 127;
export const U8_MAX = 255;
export const U16_MAX = 65535;
export const I32_MIN = -2147483648;
export const I32_MAX = 2147483647;
export const U32_MAX = 4294967295;

// A few fields are bounded by the RULES more tightly than by their serde width,
// and the rule is the bound that belongs at the point of entry — a value the
// engine's consumers disagree about is worse than one it rejects outright.
/** "The strength of each of these cords is rated from 0 to +5 … a score of +5
 * (the maximum)" — Source: Ars Magica - Definitive Edition (Core Rules).md:10836. */
export const CORD_MAX = 5;

/** Truncate to an integer inside an inclusive range; a non-finite value becomes 0
 * (itself clamped into range), the same fallback the field's default carries. */
export function clampInt(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return Math.min(Math.max(0, min), max);
  return Math.min(max, Math.max(min, Math.trunc(value)));
}
