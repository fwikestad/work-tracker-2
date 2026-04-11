---
name: UI Component Development
applyTo: "frontend/**/*.{ts,svelte,tsx,jsx}"
description: "Use when: building UI components, designing layouts, implementing user interactions, styling components, creating responsive designs. Focus on speed, visual hierarchy, and intuitive state management for work tracking."
---

# UI Component Development

## Component Architecture Patterns

### Principles
1. **Speed First**: Every interaction should feel instant; defer heavy operations to backend
2. **Single Responsibility**: Each component does one thing well
3. **Accessibility**: Keyboard navigation required; screen reader compatible
4. **Responsive**: Works on desktop and wide tablets

### Recommended Patterns

#### Container Components (Orchestrators)
- Handle routing, data fetching, state management
- Rarely render UI directly; compose from smaller components
- Bridge between UI state and backend services

#### Presentational Components
- Pure functions that receive props and render UI
- No external data fetching
- Predictable and testable

#### Form Components
- Handle data input and validation
- Emit events or update state on change
- Provide clear validation feedback

---

## Key Feature Areas & Their Requirements

### 0. Taskbar Quick Switch & Tracking Status
**Purpose**: Provide instant context switching and visibility from the OS taskbar/system tray

**Requirements**:
- Taskbar/system tray entry always available while app runs
- Label or tooltip shows currently tracked entry and elapsed time
- One-click menu for frequently used entries (favorites + recent)
- Quick actions from taskbar menu: switch entry, pause, resume, stop
- Optional global hotkey to open quick-switch from anywhere
- Switching from taskbar must be as fast as in-app switching

**Performance**: Taskbar switch action should complete in <3 seconds end-to-end

---

### 1. Active Work Indicator
**Purpose**: Display and control what the consultant is actively working on

**Requirements**:
- Large, prominent display of current work order/customer
- Real-time elapsed time indicator (if timer running)
- Clear state indicators (running, paused, stopped)
- Quick access to start/stop controls
- Keyboard shortcuts for start/stop/pause

**Performance**: Updates should feel real-time (<100ms latency)

---

### 2. Context Switching Interface
**Purpose**: Fast switching between customers and work orders

**Requirements**:
- Searchable/filterable list of customers and projects
- Display of recent/favorite items for quick access
- Support for keyboard shortcuts to switch
- Minimal dialog/overlay UI (avoid full-page forms)
- Clear indication of which is currently selected

**Performance**: Search/filter results in <50ms

---

### 3. Daily Work Log
**Purpose**: Display all work done today in chronological order

**Requirements**:
- Sortable view (by time, customer, project, activity)
- Inline editing capabilities (duration, notes, activity metadata)
- Easy duplicate/copy functionality
- Expandable details for each entry

**Performance**: Rendering 50+ entries should be smooth

---

### 4. Time Entry Creation
**Purpose**: Quick data entry for new work sessions

**Requirements**:
- Minimal required fields (customer, work order, activity type)
- Support for both quick entry (now) and manual timing (past)
- Optional metadata fields (notes, tags)
- Field validation with user-friendly error messages
- Keyboard shortcut to submit

**Performance**: Form submission in <300ms

---

### 5. Daily Summary/Analytics
**Purpose**: Show consultant their work day at a glance

**Requirements**:
- Total hours worked today
- Breakdown by customer/project
- Breakdown by activity type
- Real-time updates as entries change

**Performance**: Summary updates instantly on entry change

---

### 6. Customer/Project Management
**Purpose**: CRUD operations for customers and work orders

**Requirements**:
- List view with search/filter
- Add/edit/delete operations
- Bulk operations if applicable
- Visual indicators (color coding, status) — Phase 2
- Confirmation for destructive actions

**Performance**: List loads in <1s, search in <50ms

---

### 7. Quick-Add Component (Phase 1)
**Purpose**: Create new customer + work order and start tracking without navigating away

**Requirements**:
- Minimal overlay/popover triggered by Cmd/Ctrl+N from anywhere in the app
- Single text input for name (format: "Customer / Work Order" or just "Work Order" for existing customer)
- Enter to create + immediately start tracking
- Escape to dismiss without action
- Inline validation: warn if duplicate name exists
- Should NOT navigate away from current view — overlay closes on completion
- Autocomplete suggestions for existing customers as user types

**Performance**: Create + start tracking in <500ms

---

## State Management Patterns

### Core State Concerns
1. **Current Session State** - What's being worked on now, timer status
2. **Today's Entries** - All completed/paused work from today
3. **UI State** - Sidebar open/closed, expanded items, filters
4. **Customer/Project Data** - Cached list for quick access

### Update Patterns
- **Optimistic updates**: Update UI immediately, sync with backend
- **Real-time sync**: Subscribe to local backend/state changes
- **Resilience**: Continue operation when network is unavailable
- **Undo support**: For reversible actions (delete, status change)

---

## Design Principles

### Visual Hierarchy
- **Active work**: Largest, most prominent
- **Today's work**: Secondary, compact layout
- **Past work**: Tertiary or collapsed by default
- **State cues**: Phase 1: Running (green) / Stopped (grey). Phase 2: add Paused (amber)

### Keyboard-First Interactions
- **Global shortcuts**: Start/stop, switch customer, open search
- **Form navigation**: Tab through fields, Enter to submit
- **List navigation**: Arrow keys to move between items
- **Escape key**: Cancel/close operations
- **No mouse required**: All features accessible via keyboard

### Responsive Layout
- **Mobile/Narrow** (<480px): Single column, stacked layout
- **Tablet/Desktop** (>1024px): Multi-column (timer + summary on left, list on right)
- **Touch-friendly**: Large targets (44px min) for stylus/glove use
- **Minimize scrolling**: Essential data visible on first load

### Color & Contrast
- **Accessibility**: WCAG AA compliant (at least 4.5:1 contrast)
- **Color coding**: Use alongside other indicators (not color-only)
- **Visual distinction**: Activity types and session states clearly marked
- **State indication**: Phase 1: Running/Stopped clearly differentiated. Phase 2: add Paused state

---

## Performance Targets

| Operation | Target |
|-----------|--------|
| Initial app load | <1s |
| Timer update | <100ms |
| Customer search | <50ms |
| Entry create/save | <300ms |
| Form submission | <500ms |
| List render (50 items) | <200ms |
| Context switch | <3 seconds end-to-end |
| Taskbar quick switch | <3 seconds end-to-end |
| Quick-add create+start | <500ms |

---

## Testing Checklist

- [ ] All interactions respond to keyboard
- [ ] Timer updates smoothly every 1 second
- [ ] Search/filter responds quickly (<100ms)
- [ ] Inline edits save without page reload
- [ ] Taskbar/system tray shows active tracking state
- [ ] Taskbar quick-switch updates active entry correctly
- [ ] Quick-add overlay opens with Cmd/Ctrl+N
- [ ] Quick-add creates + starts tracking in one action
- [ ] Forms validate before submission
- [ ] Error messages are clear and actionable
- [ ] Layout adapts to container size
- [ ] Touch targets are at least 44px
- [ ] Colors meet WCAG AA contrast standards
- [ ] No data loss on accidental navigation away
- [ ] Undo works for reversible actions

