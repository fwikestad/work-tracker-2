/** Register global keyboard shortcuts. Call once in +layout.svelte. */
export function registerShortcuts(handlers: {
  onQuickAdd: () => void;
  onSearch: () => void;
  onStop: () => void;
  onEscape: () => void;
  onPause?: () => void;
  onResume?: () => void;
}) {
  function handler(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey;
    if (e.key === 'Escape') {
      handlers.onEscape();
      return;
    }
    if (ctrl && e.key === 'n') {
      e.preventDefault();
      handlers.onQuickAdd();
    }
    if (ctrl && e.key === 'k') {
      e.preventDefault();
      handlers.onSearch();
    }
    if (ctrl && e.key === 's') {
      e.preventDefault();
      handlers.onStop();
    }
    // P/R without modifier: only when not focused inside a form field
    const inFormField = (e.target as Element)?.closest?.('input, textarea, select');
    if (!inFormField && !ctrl) {
      if (e.key === 'p' || e.key === 'P') {
        e.preventDefault();
        handlers.onPause?.();
      }
      if (e.key === 'r' || e.key === 'R') {
        e.preventDefault();
        handlers.onResume?.();
      }
    }
  }
  window.addEventListener('keydown', handler);
  return () => window.removeEventListener('keydown', handler);
}
