// The opt-in update check (M6.6): the consent prompt, the one check per session,
// and the newer-release notice. Extracted like `FileOperations`: it owns its own
// state and reaches `AppStore` only through `host`, for the shared error banner.
//
// Failure policy (the plan's decision 9): the two calls that WRITE settings
// (`answerUpdatePrompt`, the toggle's `writeSettings`) surface a failure, as
// `AppStore.#persist` does, because a preference that silently failed to stick
// looks saved until the next launch contradicts it. The two that only READ
// (`updateCheckStatus`, `checkForUpdate`) stay silent: a missing notice is the
// right outcome of a failed check. That silence also covers the many test files
// whose `vi.mock('./ipc')` factory predates these commands — reading a missing
// export throws, and `start` turns that into a swallowed rejection.
//
// Consent is enforced in Rust, which re-reads `settings.json` before it fetches.
// So every path that turns the check on AWAITS its write before it checks;
// checking right after a fire-and-forget write would read the old answer.

import * as ipc from './ipc';
import type { AppError } from './types';

/** The slice of `AppStore` the update check needs. */
export interface UpdateCheckHost {
  setError: (error: AppError) => void;
}

export class UpdateCheck {
  #host: UpdateCheckHost;

  constructor(host: UpdateCheckHost) {
    this.#host = host;
  }

  /** The consent prompt is on screen and unanswered. */
  promptOpen = $state(false);
  /** The newer release's version, once a check has found one. */
  available = $state<string | null>(null);
  /** The notice was dismissed; it stays hidden for the rest of the session. */
  dismissed = $state(false);
  /** The persisted consent, as the Settings checkbox shows it. */
  enabled = $state(false);
  /** A check has been sent this session; there is at most one. */
  checkedThisSession = $state(false);

  /** The version the notice names, or `null` while there is nothing to show. */
  get noticeVersion(): string | null {
    return this.dismissed ? null : this.available;
  }

  /**
   * Called once at launch, after the settings are read. Opens the prompt when it
   * is due; otherwise checks if the user agreed. Never rejects.
   */
  async start(): Promise<void> {
    try {
      const status = await ipc.updateCheckStatus();
      this.enabled = status.enabled;
      if (status.prompt_due) {
        this.promptOpen = true;
        return;
      }
      if (status.enabled) await this.#checkOnce();
    } catch {
      // A read failure: no prompt, no check, no banner.
    }
  }

  /** Record the prompt's answer with the running version; a yes then checks. */
  async answerPrompt(enabled: boolean): Promise<void> {
    this.promptOpen = false;
    try {
      await ipc.answerUpdatePrompt(enabled);
    } catch (e) {
      this.#host.setError(e as AppError);
      return;
    }
    this.enabled = enabled;
    if (enabled) await this.#checkOnce();
  }

  /** The Settings toggle turned on: write the consent, then check. */
  async enable(): Promise<void> {
    if (!(await this.#writeEnabled(true))) return;
    await this.#checkOnce();
  }

  /** The Settings toggle turned off. */
  async disable(): Promise<void> {
    await this.#writeEnabled(false);
  }

  /** Hide the notice for the rest of the session. */
  dismiss(): void {
    this.dismissed = true;
  }

  /** Open the fixed release page Rust builds; a failure is reported. */
  async openReleasePage(): Promise<void> {
    try {
      await ipc.openReleasePage();
    } catch (e) {
      this.#host.setError(e as AppError);
    }
  }

  /**
   * Persist the consent, showing it at once and taking it back if the write
   * fails, so the checkbox always ends on the state that stuck. Resolves whether
   * the write succeeded.
   */
  async #writeEnabled(enabled: boolean): Promise<boolean> {
    const previous = this.enabled;
    this.enabled = enabled;
    try {
      await ipc.writeSettings({ update_check: enabled });
      return true;
    } catch (e) {
      this.enabled = previous;
      this.#host.setError(e as AppError);
      return false;
    }
  }

  /** The session's one check. A failure only means no notice. */
  async #checkOnce(): Promise<void> {
    if (this.checkedThisSession) return;
    this.checkedThisSession = true;
    try {
      const version = await ipc.checkForUpdate();
      if (version !== null) this.available = version;
    } catch {
      // Silent by design (decision 9).
    }
  }
}
