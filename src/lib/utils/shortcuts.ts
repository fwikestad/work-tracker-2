/** Register global keyboard shortcuts. Call once in +layout.svelte. */
export function registerShortcuts(handlers: {
  onQuickAdd: () => void;
  onSearch: () => void;
  onStop: () => void;
  onEscape: () => void;
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
  }
  window.addEventListener('keydown', handler);
  return () => window.removeEventListener('keydown', handler);
}
