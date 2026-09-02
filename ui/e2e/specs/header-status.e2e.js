// End-to-end: the header's on-screen document status (Slice A). A never-saved
// document shows an "unsaved" label; editing marks it dirty (ASCII `*`); saving
// shows the file name and clears the marker. Drives the real binary.

import { browser, $, expect } from '@wdio/globals';
import fs from 'node:fs';

import { clean, startCharacter } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

const STATUS = '[data-testid="doc-status"]';

describe('header document status', () => {
  it('shows unsaved, then dirty, then the file name after save', async () => {
    // A brand-new character has never been saved, which is the state the first
    // assertion below is about — and the status only exists in the editor, never
    // on the startup screen.
    await startCharacter('companion');
    await $('[data-testid="tab-characteristics"]').waitForExist({ timeout: 30000 });
    const status = await $(STATUS);
    await status.waitForExist({ timeout: 10000 });

    // A fresh, never-saved document: an "unsaved" label, no file name, no marker.
    const initial = clean(await status.getText());
    expect(initial).not.toContain('.json');
    expect(initial.startsWith('*')).toBe(false);

    // Editing a value marks the document dirty -> the ASCII `*` marker appears.
    await $('[data-testid="tab-characteristics"]').click();
    const intInc = await $('[data-testid="char-inc-int"]');
    await intInc.waitForExist({ timeout: 10000 });
    await intInc.click();
    await browser.waitUntil(async () => clean(await status.getText()).startsWith('*'), {
      timeout: 5000,
      timeoutMsg: 'editing should show the dirty marker in the header',
    });

    // Saving shows the file name on-screen and clears the dirty marker.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    await browser.waitUntil(
      async () => {
        const t = clean(await status.getText());
        return t.includes('arm-e2e-character.json') && !t.startsWith('*');
      },
      {
        timeout: 5000,
        timeoutMsg: 'header should show the saved file name without a dirty marker',
      },
    );
  });
});
