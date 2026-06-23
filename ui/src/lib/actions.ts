import type { Action } from 'svelte/action';

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
