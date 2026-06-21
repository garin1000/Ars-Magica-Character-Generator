// End-to-end: load ruleset → add a virtue → see validation → save → reload.
// Drives the real binary; assertions read the DOM and the saved file.

import { browser, $, expect } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const e2eFile = path.resolve(os.tmpdir(), 'arm-e2e-character.json');

describe('character editor', () => {
  it('loads rules, edits selections, validates, and round-trips a save', async () => {
    // Ruleset load completed once an Add button for a known virtue appears.
    const addKeenVision = await $('[data-testid="add-virtue.keen_vision"]');
    await addKeenVision.waitForExist({ timeout: 30000 });
    await addKeenVision.click();

    // The selection now shows up with a Remove control.
    const removeKeenVision = await $('[data-testid="remove-virtue.keen_vision"]');
    await removeKeenVision.waitForExist({ timeout: 5000 });

    // Validation panel renders (the companion sample + one minor virtue is legal).
    await expect($('[data-testid="no-issues"]')).toExist();

    // Save through the ARM_E2E_FILE seam, then confirm canonical JSON on disk.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.selections.some((s) => s.ref === 'virtue.keen_vision')).toBe(true);

    // Remove it, reload the saved file, and confirm the selection returns.
    await removeKeenVision.click();
    await $('[data-testid="load-button"]').click();
    await $('[data-testid="remove-virtue.keen_vision"]').waitForExist({ timeout: 10000 });
  });
});
