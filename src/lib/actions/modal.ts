/**
 * Svelte action for modal dialogs: traps Tab focus within the node, closes on
 * Escape, moves focus into the dialog on mount, and restores focus to the
 * previously-focused element on destroy.
 *
 * Usage:
 *   <div use:modal={{ onClose: () => (open = false) }} role="dialog" aria-modal="true"> … </div>
 */
export function modal(node: HTMLElement, params: { onClose?: () => void } = {}) {
  let onClose = params.onClose;
  const previouslyFocused = document.activeElement as HTMLElement | null;

  const FOCUSABLE =
    'a[href], button:not([disabled]), textarea, input, select, [tabindex]:not([tabindex="-1"])';

  function getFocusable(): HTMLElement[] {
    return Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
      (el) => el.offsetParent !== null || el === document.activeElement
    );
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose?.();
      return;
    }
    if (e.key !== 'Tab') return;

    const focusable = getFocusable();
    if (focusable.length === 0) {
      e.preventDefault();
      return;
    }
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement as HTMLElement | null;

    if (e.shiftKey && active === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && active === last) {
      e.preventDefault();
      first.focus();
    }
  }

  // Move focus into the dialog (first focusable element, or the node itself).
  const focusTarget = getFocusable()[0] ?? node;
  // Defer so the element is rendered/visible before focusing.
  const raf = requestAnimationFrame(() => focusTarget.focus());

  node.addEventListener('keydown', handleKeydown);

  return {
    update(newParams: { onClose?: () => void } = {}) {
      onClose = newParams.onClose;
    },
    destroy() {
      cancelAnimationFrame(raf);
      node.removeEventListener('keydown', handleKeydown);
      previouslyFocused?.focus?.();
    }
  };
}
