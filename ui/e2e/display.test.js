import { describe, expect, it } from 'vitest';

import { preflightDisplay } from './display.js';

// The suite drives a real GUI binary, so it needs an X display: the desktop
// session when there is one, Xvfb otherwise. WebdriverIO picks between those two
// itself (see display.js) — what it does badly is the missing-Xvfb case, which it
// only warns about before failing later inside WebDriver. `preflightDisplay` is
// the check that turns that into an upfront, actionable error.
//
// `platform` is passed explicitly throughout so the expectations hold wherever
// the unit suite itself is run.
describe('preflightDisplay', () => {
  const linux = { platform: 'linux' };

  it('passes when a display is available', () => {
    expect(preflightDisplay({ DISPLAY: ':0' }, { ...linux, xvfbRunAvailable: false })).toBeNull();
  });

  it('passes without a display when Xvfb can supply one', () => {
    expect(preflightDisplay({}, { ...linux, xvfbRunAvailable: true })).toBeNull();
  });

  it('reports the missing package when there is neither a display nor Xvfb', () => {
    expect(preflightDisplay({}, { ...linux, xvfbRunAvailable: false })).toContain('xvfb');
  });

  it('treats a blank DISPLAY as no display', () => {
    const problem = preflightDisplay({ DISPLAY: '   ' }, { ...linux, xvfbRunAvailable: false });
    expect(problem).not.toBeNull();
  });

  // WebdriverIO keys its decision on DISPLAY alone, so a Wayland-only session
  // (no XWayland) still goes down the Xvfb path. Mirroring that exactly keeps the
  // check from passing a run that the framework will then fail.
  it('still requires Xvfb on a Wayland-only session', () => {
    const wayland = { WAYLAND_DISPLAY: 'wayland-0' };
    expect(preflightDisplay(wayland, { ...linux, xvfbRunAvailable: false })).not.toBeNull();
    expect(preflightDisplay(wayland, { ...linux, xvfbRunAvailable: true })).toBeNull();
  });

  // X11 is a Linux concern. macOS (safaridriver) and Windows (msedgedriver)
  // render natively and never set DISPLAY, so requiring one there would block
  // perfectly healthy runs — WebdriverIO's own Xvfb path is Linux-only too.
  it('never demands a display off Linux', () => {
    const noDisplay = { xvfbRunAvailable: false };
    expect(preflightDisplay({}, { ...noDisplay, platform: 'darwin' })).toBeNull();
    expect(preflightDisplay({}, { ...noDisplay, platform: 'win32' })).toBeNull();
    expect(preflightDisplay({}, { ...noDisplay, platform: 'linux' })).not.toBeNull();
  });
});
