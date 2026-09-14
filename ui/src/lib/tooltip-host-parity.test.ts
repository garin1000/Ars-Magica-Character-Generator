import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// Sabine 3 (full-audit round 1) kept reopening, and this file is why it should
// now stop. `use:tooltip` writes `aria-describedby` onto the node the action is
// applied to (`actions.ts::tooltip`) and opens the popup on `focusin` of that
// same node. `focusin` BUBBLES, so hanging the action on a non-focusable wrapper
// still opens the popup when a descendant button takes focus — the defect is
// invisible to any "does the tooltip appear" test. But `aria-describedby` is not
// inherited: a screen reader consults it only for the element that actually has
// focus. So a tooltip on a bare `<span>`/`<li>` wrapper is announced to nobody.
//
// Round 1 named four components and fixed exactly those four; round 4 found two
// more that nobody had named (`SpellMasteryAbilityPicker`, `SpellTab`). Fixing
// the reported instances and not the class is how that finding survived three
// rounds, so the close is a sweep over every `.svelte` file rather than an
// assertion per site: a seventh host added tomorrow fails here on its own.
//
// `ssr` project (plain `*.test.ts`): pure text parsing over repo files — no
// component is mounted, no DOM is touched, no `$effect` runs. The outcome the
// markup produces (focus opening the popup) is proved once for the whole action
// in `ArtGrid.client.test.ts`; re-mounting every component would test the shared
// action N times and the markup not at all. Same technique as
// `prereq-parity.test.ts`, which diffs Rust against TS as text.

const srcDir = fileURLToPath(new URL('..', import.meta.url));

/** Every `.svelte` file under `ui/src`, in stable path order. */
function svelteFiles(): string[] {
  return readdirSync(srcDir, { recursive: true, encoding: 'utf-8' })
    .filter((path) => path.endsWith('.svelte'))
    .sort();
}

/**
 * Markup only. The `<script>` block and HTML comments both mention `use:tooltip`
 * in prose — including two deliberate "no tooltip here" notes — and neither can
 * host an action, so scanning them would invent hosts that do not exist.
 */
function markupOf(source: string): string {
  return source.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<!--[\s\S]*?-->/g, '');
}

/**
 * The full start tag enclosing the character at `at`. Walks forward from the
 * opening `<` tracking quotes and `{}` depth, because a Svelte start tag is full
 * of `>` characters that do not end it — arrow functions in handlers, `a > b` in
 * an attribute expression.
 */
function startTagAt(markup: string, at: number): string {
  const open = markup.lastIndexOf('<', at);
  if (open === -1) throw new Error('no start tag opens before a `use:tooltip`');
  let quote: string | null = null;
  let depth = 0;
  for (let i = open; i < markup.length; i += 1) {
    const char = markup[i];
    if (quote) {
      if (char === quote) quote = null;
      continue;
    }
    if (char === '"' || char === "'") quote = char;
    else if (char === '{') depth += 1;
    else if (char === '}') depth -= 1;
    else if (char === '>' && depth === 0) return markup.slice(open, i + 1);
  }
  throw new Error('unterminated start tag around a `use:tooltip`');
}

type TooltipHost = { file: string; element: string; tag: string };

/** Every element in the frontend that `use:tooltip` is applied to. */
function tooltipHosts(): TooltipHost[] {
  const hosts: TooltipHost[] = [];
  for (const file of svelteFiles()) {
    const markup = markupOf(readFileSync(`${srcDir}${file}`, 'utf-8'));
    for (const match of markup.matchAll(/use:tooltip/g)) {
      const tag = startTagAt(markup, match.index);
      hosts.push({ file, element: /^<([a-zA-Z][\w-]*)/.exec(tag)?.[1] ?? '', tag });
    }
  }
  return hosts;
}

/**
 * Can assistive technology reach the description? Only if the host is in the tab
 * order itself: natively, as a `<button>`, or explicitly via `tabindex="0"`.
 */
function isFocusable(host: TooltipHost): boolean {
  return host.element === 'button' || /\btabindex="0"/.test(host.tag);
}

describe('every `use:tooltip` host is keyboard-reachable (Sabine 3, class-level)', () => {
  it('finds the hosts at all, and reads the right tag for each', () => {
    // Guards the parser: a `use:tooltip` whose enclosing tag were mis-sliced
    // would make the assertion below vacuous rather than loud. Every tag the
    // walker returns must be a real start tag that actually carries the action.
    const hosts = tooltipHosts();
    expect(hosts.length).toBeGreaterThan(0);
    for (const host of hosts) {
      expect(host.element, `a start tag with no element name in ${host.file}`).not.toBe('');
      expect(host.tag, `the tag sliced for ${host.file} lost its action`).toContain('use:tooltip');
    }
  });

  it('discriminates — some hosts are buttons and some are explicit tabindex spans', () => {
    // Both arms of `isFocusable` must be exercised by the real codebase, or a
    // regression in either one would go unnoticed while the suite stayed green.
    const hosts = tooltipHosts();
    expect(hosts.some((host) => host.element === 'button')).toBe(true);
    expect(hosts.some((host) => host.element !== 'button' && isFocusable(host))).toBe(true);
  });

  it('applies the action only to an element that can take focus', () => {
    const unreachable = tooltipHosts()
      .filter((host) => !isFocusable(host))
      .map((host) => `${host.file}: <${host.element}>`);
    expect(unreachable).toEqual([]);
  });
});
