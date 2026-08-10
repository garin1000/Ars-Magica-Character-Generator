// End-to-end: the three ValidationModes change how the SAME illegal entity is
// reported. Enforced keeps errors, Advisory downgrades them to warnings, Silent
// clears the panel entirely. Drives the real binary.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`, so this cannot run without that build step.

import { $, $$, expect } from '@wdio/globals';

// `flaw.blatant_gift` (hermetic category) is forbidden for the default
// `companion` profile, so it yields a deterministic error in Enforced mode.
const FORBIDDEN = '[data-testid="add-flaw.blatant_gift"]';
const MODE_SELECT = '[data-testid="mode-select"]';

async function severities() {
  // Index-based loop: in webdriverio v9 the awaited `$$` result's `.map` does
  // not yield a plain iterable, so `Promise.all(items.map(...))` throws. Element
  // indexing (`items[i]`) and `.length` are stable, so read each in turn.
  const items = await $$('[data-testid="issue-list"] li');
  const result = [];
  for (let i = 0; i < items.length; i++) {
    result.push(await items[i].getAttribute('data-severity'));
  }
  return result;
}

describe('validation modes', () => {
  it('reports the same illegal entity differently per mode', async () => {
    // V/F add buttons live in the Virtues & Flaws tab; the mode select and the
    // shared validation bar are always visible.
    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();
    const addForbidden = await $(FORBIDDEN);
    await addForbidden.waitForExist({ timeout: 10000 });
    await addForbidden.click();

    const modeSelect = await $(MODE_SELECT);

    // Enforced (default): at least one error-severity issue, no warnings.
    await modeSelect.selectByAttribute('value', 'enforced');
    await $('[data-testid="issue-list"]').waitForExist({ timeout: 5000 });
    let sev = await severities();
    expect(sev).toContain('error');
    expect(sev).not.toContain('warning');

    // Advisory: every issue is downgraded to a warning; no errors remain.
    await modeSelect.selectByAttribute('value', 'advisory');
    await browserWaitFor(async () => {
      const s = await severities();
      return s.length > 0 && !s.includes('error');
    });
    sev = await severities();
    expect(sev).toContain('warning');
    expect(sev).not.toContain('error');

    // Silent: the panel reports no issues at all.
    await modeSelect.selectByAttribute('value', 'silent');
    await $('[data-testid="no-issues"]').waitForExist({ timeout: 5000 });
    expect(await $$('[data-testid="issue-list"] li').then((l) => l.length)).toBe(0);
  });
});

// Small polling helper (validation is debounced + async, so the DOM settles a
// tick after the mode change).
async function browserWaitFor(predicate, timeout = 5000) {
  const { browser } = await import('@wdio/globals');
  await browser.waitUntil(predicate, { timeout, timeoutMsg: 'condition not met in time' });
}
