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

/**
 * Show a styled, high-contrast tooltip on hover or keyboard focus. The popup is
 * appended to `document.body` and positioned with `getBoundingClientRect`, so it
 * is never clipped by a scrolling panel. Reveals on `mouseenter`/`focusin`,
 * hides on `mouseleave`/`focusout`/click and on scroll or resize. A no-op when
 * there is no content, so plain rows stay tooltip-free.
 */
export const tooltip: Action<HTMLElement, TooltipContent | undefined> = (node, content) => {
  let current: TooltipContent | undefined = content;
  let pop: HTMLDivElement | null = null;
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
    let top = rect.bottom + 6;
    if (top + pop.offsetHeight > window.innerHeight - margin) {
      top = Math.max(margin, rect.top - pop.offsetHeight - 6);
    }
    pop.style.left = `${left}px`;
    pop.style.top = `${top}px`;
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
      r.textContent = current.reason;
      pop.appendChild(r);
    }
    if (current?.text) {
      const p = document.createElement('p');
      p.className = 'tooltip-text';
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
    document.body.appendChild(pop);
    node.setAttribute('aria-describedby', id);
    position();
  };

  const hide = () => {
    pop?.remove();
    pop = null;
    node.removeAttribute('aria-describedby');
  };

  node.addEventListener('mouseenter', show);
  node.addEventListener('mouseleave', hide);
  node.addEventListener('focusin', show);
  node.addEventListener('focusout', hide);
  node.addEventListener('click', hide);
  window.addEventListener('scroll', hide, true);
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
      node.removeEventListener('mouseenter', show);
      node.removeEventListener('mouseleave', hide);
      node.removeEventListener('focusin', show);
      node.removeEventListener('focusout', hide);
      node.removeEventListener('click', hide);
      window.removeEventListener('scroll', hide, true);
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
