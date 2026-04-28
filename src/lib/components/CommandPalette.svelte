<script lang="ts">
  import { Search, Command, ArrowRight, Zap, Server } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';

  interface Item {
    id: string;
    name: string;
    type: 'host' | 'action';
    ip?: string;
  }

  let { 
    isOpen = false, 
    items = [], 
    onclose = () => {}, 
    onselect = (item: Item) => {} 
  } = $props<{
    isOpen: boolean;
    items: Item[];
    onclose?: () => void;
    onselect?: (item: Item) => void;
  }>();

  let query = $state('');
  let selectedIndex = $state(0);
  let inputElement = $state<HTMLInputElement>();

  let filteredItems = $derived(
    items.filter((item: Item) => 
      item.name.toLowerCase().includes(query.toLowerCase()) || 
      (item.ip && item.ip.includes(query))
    )
  );

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;

    if (e.key === 'Escape') {
      onclose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % filteredItems.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + filteredItems.length) % filteredItems.length;
    } else if (e.key === 'Enter') {
      if (filteredItems[selectedIndex]) {
        onselect(filteredItems[selectedIndex]);
      }
    }
  }

  $effect(() => {
    if (isOpen) {
      selectedIndex = 0;
      query = '';
      setTimeout(() => inputElement?.focus(), 50);
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="command-palette-overlay" onclick={() => onclose()} transition:fade={{ duration: 150 }}>
    <div class="command-palette" onclick={(e) => e.stopPropagation()} transition:fly={{ y: -20, duration: 200 }}>
      <div class="flex items-center px-4 gap-3 border-b border-border-dark">
        <Search size={20} class="text-neutral-dark" />
        <input
          bind:this={inputElement}
          bind:value={query}
          class="command-palette-input"
          placeholder="Search for hosts or commands..."
        />
        <div class="flex items-center gap-1 px-1.5 py-0.5 rounded border border-border-std bg-surface-near-black text-[10px] text-neutral-mid font-medium">
          <Command size={10} />
          <span>K</span>
        </div>
      </div>

      <div class="command-palette-results">
        {#if filteredItems.length === 0}
          <div class="p-8 text-center text-neutral-dark text-sm font-sans">
            No results found for "{query}"
          </div>
        {:else}
          {#each filteredItems as item, i}
            <div
              class="command-result {i === selectedIndex ? 'selected' : ''}"
              onmouseenter={() => selectedIndex = i}
              onclick={() => onselect(item)}
            >
              <div class="flex items-center gap-3">
                {#if item.type === 'host'}
                  <Server size={16} class={i === selectedIndex ? 'text-neutral-off-white' : 'text-brand-green'} />
                {:else}
                  <Zap size={16} class={i === selectedIndex ? 'text-neutral-off-white' : 'text-brand-link'} />
                {/if}
                <div class="flex flex-col">
                  <span class="font-normal text-neutral-off-white">{item.name}</span>
                  {#if item.ip}
                    <span class="text-[11px] text-neutral-mid">{item.ip}</span>
                  {/if}
                </div>
              </div>
              {#if i === selectedIndex}
                <div class="flex items-center gap-1 text-[11px] font-medium text-neutral-off-white opacity-80">
                  <span>Open</span>
                  <ArrowRight size={12} />
                </div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
      
      <div class="px-4 py-2 bg-surface-near-black border-t border-border-dark flex items-center justify-between text-[11px] text-neutral-dark">
        <div class="flex gap-4">
          <div class="flex gap-1.5 items-center">
            <kbd class="px-1 py-0.5 rounded bg-surface-dark border border-border-std">↑↓</kbd>
            <span>to navigate</span>
          </div>
          <div class="flex gap-1.5 items-center">
            <kbd class="px-1 py-0.5 rounded bg-surface-dark border border-border-std">↵</kbd>
            <span>to select</span>
          </div>
        </div>
        <div class="flex gap-1.5 items-center">
          <kbd class="px-1 py-0.5 rounded bg-surface-dark border border-border-std">ESC</kbd>
          <span>to dismiss</span>
        </div>
      </div>

    </div>
  </div>
{/if}
