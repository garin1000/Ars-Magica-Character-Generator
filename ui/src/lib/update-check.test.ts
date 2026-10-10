import { beforeEach, describe, expect, it, vi } from 'vitest';

// `ssr`: `UpdateCheck` is plain rune state plus IPC calls. Nothing here needs a
// mounted component, so it runs in the default project (CLAUDE.md, "Frontend test
// environments"). The component-level twins live in the `*.client.test.ts` files.
//
// M6.6, decisions 6, 7 and 9: the prompt records an answer, one check runs per
// session once consent is known, write failures surface and read failures stay
// silent.
vi.mock('./ipc', () => ({
  updateCheckStatus: vi.fn(),
  answerUpdatePrompt: vi.fn(),
  checkForUpdate: vi.fn(),
  openReleasePage: vi.fn(),
  writeSettings: vi.fn(),
}));

import * as ipc from './ipc';
import type { AppError } from './types';
import { UpdateCheck } from './update-check.svelte';

/** A promise the test settles by hand, to pin what may happen before it does. */
function deferred<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (error: unknown) => void;
} {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

/** Let every queued microtask run. */
async function settle(): Promise<void> {
  for (let i = 0; i < 10; i += 1) await Promise.resolve();
}

let setError: ReturnType<typeof vi.fn<(error: AppError) => void>>;

function make(): UpdateCheck {
  setError = vi.fn<(error: AppError) => void>();
  return new UpdateCheck({ setError });
}

const ioError = { kind: 'io' } as unknown as AppError;

beforeEach(() => {
  vi.mocked(ipc.updateCheckStatus)
    .mockReset()
    .mockResolvedValue({ prompt_due: false, enabled: false });
  vi.mocked(ipc.answerUpdatePrompt).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.checkForUpdate).mockReset().mockResolvedValue(null);
  vi.mocked(ipc.openReleasePage).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.writeSettings).mockReset().mockResolvedValue(undefined);
});

describe('starting the update check', () => {
  it('opens the prompt when it is due, and checks nothing yet', async () => {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: true, enabled: true });
    const check = make();

    await check.start();

    expect(check.promptOpen).toBe(true);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
  });

  it('checks once when consent is on and no prompt is due', async () => {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: false, enabled: true });
    vi.mocked(ipc.checkForUpdate).mockResolvedValue('99.0.0');
    const check = make();

    await check.start();

    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
    expect(check.enabled).toBe(true);
    expect(check.noticeVersion).toBe('99.0.0');
  });

  it('checks nothing when the check is off', async () => {
    const check = make();

    await check.start();

    expect(ipc.updateCheckStatus).toHaveBeenCalledTimes(1);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
    expect(check.promptOpen).toBe(false);
  });

  it('stays silent when the status cannot be read', async () => {
    vi.mocked(ipc.updateCheckStatus).mockRejectedValue(ioError);
    const check = make();

    await expect(check.start()).resolves.toBeUndefined();

    expect(ipc.updateCheckStatus).toHaveBeenCalledTimes(1);
    expect(setError).not.toHaveBeenCalled();
    expect(check.promptOpen).toBe(false);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
  });

  it('stays silent when the check itself fails', async () => {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: false, enabled: true });
    vi.mocked(ipc.checkForUpdate).mockRejectedValue(ioError);
    const check = make();

    await expect(check.start()).resolves.toBeUndefined();

    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
    expect(setError).not.toHaveBeenCalled();
    expect(check.noticeVersion).toBeNull();
  });

  it('shows no notice when no newer release exists', async () => {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: false, enabled: true });
    const check = make();

    await check.start();

    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
    expect(check.noticeVersion).toBeNull();
  });
});

describe('answering the prompt', () => {
  async function promptShown(): Promise<UpdateCheck> {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: true, enabled: false });
    const check = make();
    await check.start();
    return check;
  }

  // Rust re-reads `settings.json` before it fetches, so a check sent before the
  // consent is on disk would read the old answer and do nothing.
  it('records a yes before it checks', async () => {
    const check = await promptShown();
    const write = deferred<void>();
    vi.mocked(ipc.answerUpdatePrompt).mockReturnValue(write.promise);

    const answering = check.answerPrompt(true);
    await settle();
    expect(ipc.answerUpdatePrompt).toHaveBeenCalledWith(true);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();

    write.resolve();
    await answering;

    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
    expect(check.promptOpen).toBe(false);
    expect(check.enabled).toBe(true);
  });

  it('records a no and checks nothing', async () => {
    const check = await promptShown();

    await check.answerPrompt(false);

    expect(ipc.answerUpdatePrompt).toHaveBeenCalledWith(false);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
    expect(check.promptOpen).toBe(false);
    expect(check.enabled).toBe(false);
  });

  it('surfaces a failed answer write and checks nothing', async () => {
    const check = await promptShown();
    vi.mocked(ipc.answerUpdatePrompt).mockRejectedValue(ioError);

    await check.answerPrompt(true);

    expect(setError).toHaveBeenCalledWith(ioError);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
  });
});

describe('the settings toggle', () => {
  it('writes the consent before it checks', async () => {
    const check = make();
    const write = deferred<void>();
    vi.mocked(ipc.writeSettings).mockReturnValue(write.promise);

    const enabling = check.enable();
    await settle();
    expect(ipc.writeSettings).toHaveBeenCalledWith({ update_check: true });
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();

    write.resolve();
    await enabling;

    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
    expect(check.enabled).toBe(true);
  });

  it('checks at most once per session', async () => {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: false, enabled: true });
    const check = make();
    await check.start();

    await check.disable();
    await check.enable();
    await check.enable();

    expect(ipc.writeSettings).toHaveBeenCalledWith({ update_check: true });
    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
  });

  it('writes a refusal and checks nothing when turned off', async () => {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: true, enabled: true });
    const check = make();
    await check.start();

    await check.disable();

    expect(ipc.writeSettings).toHaveBeenCalledWith({ update_check: false });
    expect(check.enabled).toBe(false);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
  });

  it('surfaces a failed write and keeps the stored state', async () => {
    const check = make();
    vi.mocked(ipc.writeSettings).mockRejectedValue(ioError);

    await check.enable();

    expect(setError).toHaveBeenCalledWith(ioError);
    expect(check.enabled).toBe(false);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
  });

  it('surfaces a failed switch-off and stays on', async () => {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: true, enabled: true });
    const check = make();
    await check.start();
    vi.mocked(ipc.writeSettings).mockRejectedValue(ioError);

    await check.disable();

    expect(setError).toHaveBeenCalledWith(ioError);
    expect(check.enabled).toBe(true);
  });
});

describe('the notice', () => {
  async function newerFound(): Promise<UpdateCheck> {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: false, enabled: true });
    vi.mocked(ipc.checkForUpdate).mockResolvedValue('99.0.0');
    const check = make();
    await check.start();
    return check;
  }

  it('is hidden for the rest of the session once dismissed', async () => {
    const check = await newerFound();
    expect(check.noticeVersion).toBe('99.0.0');

    check.dismiss();

    expect(check.noticeVersion).toBeNull();
  });

  it('opens the fixed release page', async () => {
    const check = await newerFound();

    await check.openReleasePage();

    expect(ipc.openReleasePage).toHaveBeenCalledTimes(1);
    expect(vi.mocked(ipc.openReleasePage).mock.calls[0]).toEqual([]);
  });

  it('surfaces a release page that could not be opened', async () => {
    const check = await newerFound();
    vi.mocked(ipc.openReleasePage).mockRejectedValue(ioError);

    await check.openReleasePage();

    expect(setError).toHaveBeenCalledWith(ioError);
  });
});
