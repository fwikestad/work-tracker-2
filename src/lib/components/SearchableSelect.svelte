<script lang="ts">
  interface Option {
    value: string;
    label: string;
    color?: string | null;
  }

  let {
    options = [],
    value = $bindable(''),
    placeholder = 'Select...',
    disabled = false
  }: {
    options: Option[];
    value: string;
    placeholder?: string;
    disabled?: boolean;
  } = $props();

  let isOpen = $state(false);
  let filterQuery = $state('');
  let highlightIndex = $state(0);
  let containerRef = $state<HTMLDivElement | undefined>(undefined);
  let filterInputRef = $state<HTMLInputElement | undefined>(undefined);

  const selectedOption = $derived(options.find((opt) => opt.value === value));
  const displayLabel = $derived(selectedOption?.label || placeholder);
  const isPlaceholder = $derived(!selectedOption);

  const filteredOptions = $derived(() => {
    if (!filterQuery.trim()) return options;
    const lower = filterQuery.toLowerCase();
    return options.filter((opt) => opt.label.toLowerCase().includes(lower));
  });

  function open() {
    if (disabled) return;
    isOpen = true;
    filterQuery = '';
    highlightIndex = 0;
    setTimeout(() => filterInputRef?.focus(), 0);
  }

  function close() {
    isOpen = false;
    filterQuery = '';
    highlightIndex = 0;
  }

  function selectOption(opt: Option) {
    value = opt.value;
    close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;

    const opts = filteredOptions();
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightIndex = Math.min(highlightIndex + 1, opts.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightIndex = Math.max(highlightIndex - 1, 0);
    } else if (e.key === 'Enter' && opts[highlightIndex]) {
      e.preventDefault();
      selectOption(opts[highlightIndex]);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      close();
    }
  }

  function handleClickOutside(e: MouseEvent) {
    if (containerRef && !containerRef.contains(e.target as Node)) {
      close();
    }
  }

  $effect(() => {
    if (isOpen) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="searchable-select" class:disabled bind:this={containerRef}>
  {#if !isOpen}
    <button type="button" class="trigger" class:placeholder={isPlaceholder} onclick={open}>
      {#if selectedOption?.color}
        <span class="dot" style="background: {selectedOption.color}"></span>
      {/if}
      <span class="label">{displayLabel}</span>
      <span class="arrow">▼</span>
    </button>
  {:else}
    <div class="dropdown-container">
      <input
        type="text"
        class="filter-input"
        bind:value={filterQuery}
        bind:this={filterInputRef}
        placeholder="Type to filter..."
      />
      <div class="dropdown">
        {#if filteredOptions().length === 0}
          <div class="empty">No options found</div>
        {:else}
          {#each filteredOptions() as opt, i}
            <button
              type="button"
              class="option"
              class:highlighted={i === highlightIndex}
              onclick={() => selectOption(opt)}
            >
              {#if opt.color}
                <span class="dot" style="background: {opt.color}"></span>
              {/if}
              <span>{opt.label}</span>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .searchable-select {
    position: relative;
    width: 100%;
  }

  .searchable-select.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .trigger {
    width: 100%;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    padding: 10px;
    font-family: inherit;
    font-size: 14px;
    min-height: 44px;
    text-align: left;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .trigger:hover:not(:disabled) {
    border-color: #333;
  }

  .trigger:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  .trigger.placeholder .label {
    color: var(--text-muted);
  }

  .label {
    flex: 1;
  }

  .arrow {
    font-size: 10px;
    color: var(--text-muted);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .dropdown-container {
    position: relative;
    width: 100%;
  }

  .filter-input {
    width: 100%;
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    color: var(--text);
    padding: 10px;
    font-family: inherit;
    font-size: 14px;
    min-height: 44px;
    outline: 1px solid var(--accent);
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    max-height: 240px;
    overflow-y: auto;
    z-index: 100;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .option {
    width: 100%;
    background: none;
    border: none;
    color: var(--text);
    padding: 10px;
    font-family: inherit;
    font-size: 14px;
    text-align: left;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 44px;
  }

  .option:hover,
  .option.highlighted {
    background: #1f1f1f;
  }

  .empty {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 14px;
  }
</style>
