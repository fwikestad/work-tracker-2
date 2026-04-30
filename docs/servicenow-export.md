# ServiceNow Export

Work Tracker 2 can export your tracked time as a CSV ready to import into ServiceNow.

---

## Setup: Add ServiceNow Task IDs

Before exporting, link your work orders to ServiceNow incidents or tasks.

1. Go to **Manage → Work Orders**
2. Click **Edit** on any work order
3. Enter the **ServiceNow Task ID** (e.g. `INC1234567`, `TASK0012345`)
4. Save

This is optional — if left blank, the export uses the work order **code** as the Task ID, falling back to the work order **name** if there's no code either.

---

## How to Export

1. Go to **Reports**
2. Select a date range (Today / This week / Last week / This month / Custom)
3. Click **Export ServiceNow**
4. Choose where to save the `.csv` file

The file opens in Excel, Google Sheets, or can be imported directly into ServiceNow.

---

## CSV Format

```
Date, Task ID, Work Order, Time Worked (hours), Category, Work Notes
2026-05-01, INC1234567, My Project, 1.5, Development, Fixed login bug; Reviewed PR
2026-05-01, INC9998888, Other Project, 0.5, Meeting,
```

| Column | Description |
|---|---|
| **Date** | Day the work occurred (YYYY-MM-DD) |
| **Task ID** | ServiceNow Task ID → work order code → work order name |
| **Work Order** | Work order name |
| **Time Worked (hours)** | Total hours that day, rounded up to nearest 0.5h |
| **Category** | Activity type from your first session that day |
| **Work Notes** | All session notes for that day, joined with `"; "` |

**One row per day per work order** — multiple sessions on the same work order are aggregated.

---

## Duration Rounding

Time is rounded **up** to the nearest 30 minutes:

| Actual time | Exported |
|---|---|
| 1 min | 0.5h |
| 29 min | 0.5h |
| 30 min | 0.5h |
| 31 min | 1.0h |
| 1h 01m | 1.5h |

This ensures you don't under-bill for short sessions.

---

## Tips

**Missing Task ID?** The export still works — it falls back to the work order code or name. To add one later: Manage → Work Orders → Edit.

**Only completed sessions are included.** If your timer is running when you export, that session won't appear until you stop it.

**Notes are optional.** Empty notes produce a blank Work Notes cell — that's fine for ServiceNow import.
