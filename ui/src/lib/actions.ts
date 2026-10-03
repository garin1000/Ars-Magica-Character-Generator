import type { Action } from 'svelte/action';

/**
 * Rich tooltip content: an optional non-takeable REASON shown first (emphasized),
 * then the main description paragraph, then an optional labelled list. For a
 * takeable item `reason` is omitted and only the description/list show; when an
 * item cannot be taken the reason appears ABOVE its normal description rather
 * than replacing it.
 */
export interface TooltipContent {
  reason?: string;
  text?: string;
  listLabel?: string;
  list?: string[];
}

/**
 * Shared composer used by every picker: attach a non-takeable `reason` to a
 * tooltip's normal content so the reason renders above the description. With no
 * reason (a takeable item) the content is returned untouched, so takeable rows
 * keep exactly their previous tooltip.
 */
export function withReason(content: TooltipContent, reason: string | undefined): TooltipContent {
  return reason ? { ...content, reason } : content;
}

let tooltipSeq = 0;

/** How long hover or keyboard focus must rest on a trigger before its tooltip
 *  opens (try-out finding 3, decided by Norbert: the same for both). */
const OPEN_DELAY_MS = 500;
/** How long the popup survives the pointer leaving it or its trigger, so the
 *  pointer can travel from one to the other (try-out finding 2). */
const CLOSE_GRACE_MS = 150;
/** The popup overlaps its trigger by this much, so no gap lies between them. */
const OVERLAP_PX = 1;

/**
 * Show a styled, high-contrast tooltip on hover or keyboard focus. The popup is
 * appended to `document.body` and positioned with `getBoundingClientRect`, so it
 * is never clipped by a scrolling panel. It opens after `OPEN_DELAY_MS` of hover
 * or focus; leaving or blurring first cancels it. It sits flush against the
 * trigger and takes the pointer: leaving the trigger or the popup closes it after
 * `CLOSE_GRACE_MS`, unless the pointer reaches the other one first. Focus loss,
 * a click on the trigger, Escape, a page scroll (not one inside the popup) and a
 * resize close it at once. While it is open, PageUp/PageDown on the focused
 * trigger scroll its text. A no-op when there is no content, so plain rows stay
 * tooltip-free.
 */
export const tooltip: Action<HTMLElement, TooltipContent | undefined> = (node, content) => {
  let current: TooltipContent | undefined = content;
  let pop: HTMLDivElement | null = null;
  let openTimer: ReturnType<typeof setTimeout> | undefined;
  let closeTimer: ReturnType<typeof setTimeout> | undefined;
  /** The trigger's own `aria-describedby` from before the popup opened. */
  let ownDescribedBy: string | null = null;
  const id = `tooltip-${(tooltipSeq += 1)}`;

  const hasContent = (c: TooltipContent | undefined): boolean =>
    !!c && (!!c.reason || !!c.text || (!!c.list && c.list.length > 0));

  const position = () => {
    if (!pop) return;
    const rect = node.getBoundingClientRect();
    const margin = 8;
    const left = Math.max(
      margin,
      Math.min(rect.left, window.innerWidth - pop.offsetWidth - margin),
    );
    let top = rect.bottom - OVERLAP_PX;
    if (top + pop.offsetHeight > window.innerHeight - margin) {
      top = Math.max(margin, rect.top - pop.offsetHeight + OVERLAP_PX);
    }
    pop.style.left = `${left}px`;
    pop.style.top = `${top}px`;
  };

  const cancelOpen = () => {
    clearTimeout(openTimer);
    openTimer = undefined;
  };

  const cancelClose = () => {
    clearTimeout(closeTimer);
    closeTimer = undefined;
  };

  const show = () => {
    if (pop || !hasContent(current)) return;
    pop = document.createElement('div');
    pop.className = 'tooltip-pop';
    pop.id = id;
    pop.setAttribute('role', 'tooltip');
    if (current?.reason) {
      const r = document.createElement('p');
      r.className = 'tooltip-reason';
      r.dataset.testid = 'tooltip-reason';
      r.textContent = current.reason;
      pop.appendChild(r);
    }
    if (current?.text) {
      const p = document.createElement('p');
      p.className = 'tooltip-text';
      p.dataset.testid = 'tooltip-text';
      p.textContent = current.text;
      pop.appendChild(p);
    }
    if (current?.list && current.list.length > 0) {
      const list = document.createElement('p');
      list.className = 'tooltip-list';
      const label = current.listLabel ? `${current.listLabel}: ` : '';
      list.textContent = `${label}${current.list.join(', ')}`;
      pop.appendChild(list);
    }
    pop.addEventListener('mouseenter', cancelClose);
    pop.addEventListener('mouseleave', scheduleClose);
    document.body.appendChild(pop);
    ownDescribedBy = node.getAttribute('aria-describedby');
    node.setAttribute('aria-describedby', ownDescribedBy ? `${ownDescribedBy} ${id}` : id);
    position();
  };

  /** Close at once, dropping any pending open or close. */
  function hide() {
    cancelOpen();
    cancelClose();
    if (!pop) return;
    pop.remove();
    pop = null;
    // Put back the trigger's own description rather than wiping it.
    if (ownDescribedBy === null) node.removeAttribute('aria-describedby');
    else node.setAttribute('aria-describedby', ownDescribedBy);
    ownDescribedBy = null;
  }

  function scheduleClose() {
    cancelClose();
    closeTimer = setTimeout(hide, CLOSE_GRACE_MS);
  }

  /** Hover or focus arrived: keep an open popup, or open one after the delay. */
  const scheduleOpen = () => {
    cancelClose();
    if (pop || openTimer !== undefined || !hasContent(current)) return;
    openTimer = setTimeout(() => {
      openTimer = undefined;
      show();
    }, OPEN_DELAY_MS);
  };

  const onTriggerLeave = () => {
    if (pop) scheduleClose();
    else cancelOpen();
  };

  /** A scroll inside the popup is the reader scrolling it; any other closes it. */
  const onScroll = (event: Event) => {
    if (pop && event.target instanceof Node && pop.contains(event.target)) return;
    hide();
  };

  /** Scroll the open popup's text by about one page (keyboard half of finding 2). */
  const scrollText = (direction: 1 | -1): boolean => {
    const text = pop?.querySelector<HTMLElement>('.tooltip-text');
    if (!text) return false;
    text.scrollTop += direction * Math.max(1, text.clientHeight * 0.9);
    return true;
  };

  // S2 (full-audit a11y): hover/focus opens the popup, but nothing offered a
  // keyboard way to dismiss it short of moving focus elsewhere. Escape is the
  // conventional dismiss key for a transient popup (menus, dialogs); a sighted
  // keyboard user gets the same escape hatch a mouse user already had via
  // mouseleave.
  const onKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      hide();
      return;
    }
    const direction = event.key === 'PageDown' ? 1 : event.key === 'PageUp' ? -1 : 0;
    if (direction !== 0 && scrollText(direction)) event.preventDefault();
  };

  node.addEventListener('mouseenter', scheduleOpen);
  node.addEventListener('mouseleave', onTriggerLeave);
  node.addEventListener('focusin', scheduleOpen);
  node.addEventListener('focusout', hide);
  node.addEventListener('click', hide);
  node.addEventListener('keydown', onKeydown);
  window.addEventListener('scroll', onScroll, true);
  window.addEventListener('resize', hide);

  return {
    update(next: TooltipContent | undefined) {
      current = next;
      if (pop) {
        hide();
        show();
      }
    },
    destroy() {
      hide();
      node.removeEventListener('mouseenter', scheduleOpen);
      node.removeEventListener('mouseleave', onTriggerLeave);
      node.removeEventListener('focusin', scheduleOpen);
      node.removeEventListener('focusout', hide);
      node.removeEventListener('click', hide);
      node.removeEventListener('keydown', onKeydown);
      window.removeEventListener('scroll', onScroll, true);
      window.removeEventListener('resize', hide);
    },
  };
};

/**
 * Reserve a right-edge gutter on the `.item-name` equal to the width of the
 * `.badges` overlaid on top of it. The name then word-wraps within the narrower
 * box (beside the tags, never overlapping them); a word too long to fit simply
 * overflows the box and runs *under* the opaque tags rather than dropping to a
 * line below. Re-measures when the row or the tags change size (e.g. language
 * switch), since tag width is language-dependent.
 */
export const reserveTagSpace: Action<HTMLElement> = (node) => {
  const name = node.querySelector<HTMLElement>('.item-name');
  const badges = node.querySelector<HTMLElement>('.badges');

  const apply = () => {
    if (!name) return;
    const width = badges?.offsetWidth ?? 0;
    name.style.paddingRight = width ? `${width + 6}px` : '';
  };

  const observer = new ResizeObserver(apply);
  observer.observe(node);
  if (badges) observer.observe(badges);
  apply();

  return {
    destroy() {
      observer.disconnect();
    },
  };
};
