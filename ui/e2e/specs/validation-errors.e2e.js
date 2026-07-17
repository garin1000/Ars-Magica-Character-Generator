// End-to-end: an illegal selection surfaces a localized error in the validation
// panel. Drives the real binary; assertions read the rendered DOM.
//
// NOTE: requires a display + the production binary (see e2e/README.md). The
// wdio `onPrepare` hook builds `target/release/arm-app` via `cargo tauri build`,
// so this cannot run without that build step.
//
// Why a forbidden-category flaw rather than an over-budget case: the shipped
// sample ruleset (rules/core/) has too few selectable virtues to exceed the
// companion's 10-point virtue budget, so over_budget_virtues is not reachable
// purely through clicks. `flaw.blatant_gift` is in the `hermetic` category,
// which the default `companion` profile forbids, giving a deterministic
// error-severity issue through the same render path the over-budget case uses.

import { $, $$, expect } from '@wdio/globals';

describe('validation errors', () => {
  it('shows a localized, error-severity issue for a forbidden-category selection', async () => {
    // The shared validation bar (bottom) reports for the whole character; the V/F
    // add buttons live in the Virtues & Flaws tab.
    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });

    // No issues before any selection.
    await expect($('[data-testid="no-issues"]')).toExist();

    await vfTab.click();
    const addForbidden = await $('[data-testid="add-flaw.blatant_gift"]');
    await addForbidden.waitForExist({ timeout: 10000 });
    await addForbidden.click();

    // The issue list now contains at least one error-severity issue.
    const issueList = await $('[data-testid="issue-list"]');
    await issueList.waitForExist({ timeout: 5000 });

    const errors = await $$('[data-severity="error"]');
    expect(await errors.length).toBeGreaterThan(0);

    // The message is localized (names the offending item, never a raw id or the
    // i18n key): a raw slug rendered as a label would violate the strict
    // data-kind separation in CLAUDE.md.
    const firstError = errors[0];
    const text = await firstError.getText();
    expect(text).not.toContain('issue-');
    expect(text).not.toContain('flaw.blatant_gift');
    expect(text).toContain('Blatant Gift');
  });
});
