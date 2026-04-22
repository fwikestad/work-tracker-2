/** Format elapsed seconds as HH:MM:SS */
export function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/** Format seconds as human-readable "2h 34m" */
export function formatHuman(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0 && m > 0) return `${h}h ${m}m`;
  if (h > 0) return `${h}h`;
  if (m > 0) return `${m}m`;
  return `<1m`;
}

/**
 * Parse a timestamp string to a Date object.
 *
 * Handles both RFC3339 (`2026-04-22T07:30:00Z`) and legacy SQLite space-separated
 * format (`2026-04-22 07:30:00`). Space-separated strings without a timezone
 * designator are treated as UTC (matching how the backend stores them).
 */
export function parseTimestamp(s: string): Date {
  // Normalize SQLite space-separated format: "YYYY-MM-DD HH:MM:SS" → RFC3339 UTC
  if (s.length === 19 && s[10] === ' ') {
    return new Date(s.replace(' ', 'T') + 'Z');
  }
  return new Date(s);
}

/** Format ISO timestamp as "9:30 AM" */
export function formatTime(iso: string): string {
  return parseTimestamp(iso).toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' });
}

/** Format a "YYYY-MM-DD" date string as a human-readable day label (e.g. "Monday, April 21, 2026") */
export function formatDay(dateStr: string): string {
  const [year, month, day] = dateStr.split('-').map(Number);
  const d = new Date(year, month - 1, day); // local timezone — avoids UTC off-by-one
  return d.toLocaleDateString(undefined, { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' });
}

/** Today's date as YYYY-MM-DD in local timezone */
export function today(): string {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}
