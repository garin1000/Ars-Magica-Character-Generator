import { describe, expect, it } from 'vitest';

import { withReason, type TooltipContent } from './actions';

// `withReason` is the single, shared composer every picker uses to turn a
// non-takeable item's tooltip into "reason first, then the normal description"
// (rather than the reason REPLACING the description). Takeable items pass no
// reason and get their content back untouched.
describe('withReason', () => {
  it('attaches the reason to existing content, preserving text and list', () => {
    const base: TooltipContent = {
      text: 'A bolt of flame that leaps from your palm.',
      listLabel: 'Specialties',
      list: ['fire'],
    };
    expect(withReason(base, 'Above your casting cap (3)')).toEqual({
      ...base,
      reason: 'Above your casting cap (3)',
    });
  });

  it('leaves the content unchanged (no reason field) when there is no reason', () => {
    const base: TooltipContent = { text: 'A bolt of flame.' };
    const out = withReason(base, undefined);
    expect(out).toEqual(base);
    expect(out).not.toHaveProperty('reason');
  });
});
