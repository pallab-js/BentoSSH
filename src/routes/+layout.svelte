<script lang="ts">
  import '../app.css';
  import { Toaster } from 'svelte-sonner';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import HealthDashboard from '$lib/components/HealthDashboard.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { Server, Settings, Plus, Terminal as TerminalIcon, Search, Activity } from 'lucide-svelte';
  import { uiState } from '$lib/state.svelte';
  import { settingsState } from '$lib/settings.svelte';
  import { sessionState } from '$lib/session.svelte';

  let { children } = $props();

  interface Host {
    id: number;
    name: string;
    ip: string;
    port: number;
    username: string;
  }

  interface PaletteItem {
    id: string;
    name: string;
    type: 'host' | 'action';
    ip?: string;
    original?: any;
  }

  let hosts = $state<Host[]>([]);
  let paletteItems = $state<PaletteItem[]>([]);

  async function loadHosts() {
    try {
      const rawHosts = await invoke('get_hosts') as any[];
      hosts = rawHosts.map(h => ({
        id: h.id,
        name: h.name,
        ip: h.ip,
        port: h.port,
        username: h.username
      }));
      
      paletteItems = [
        ...hosts.map(h => ({ id: String(h.id), name: h.name, type: 'host' as const, ip: h.ip, original: h } as PaletteItem)),
        { id: 'new-host', name: 'New SSH Session...', type: 'action' as const } as PaletteItem,
        { id: 'restart-service', name: 'Restart Service (Active)', type: 'action' as const } as PaletteItem,
        { id: 'tail-logs', name: 'Tail Syslog (Active)', type: 'action' as const } as PaletteItem
      ];
    } catch (e) {
      console.error('Failed to load hosts', e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    const paletteKey = settingsState.current.keyboard.commandPalette.toLowerCase();
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === paletteKey) {
      e.preventDefault();
      uiState.isPaletteOpen = !uiState.isPaletteOpen;
    }
  }

  function handlePaletteSelect(item: PaletteItem) {
    uiState.isPaletteOpen = false;
    
    if (item.type === 'host') {
      // Dispatch a custom event that +page.svelte can listen to
      window.dispatchEvent(new CustomEvent('connect-host', { detail: item.original }));
    } else if (item.id === 'new-host') {
       window.dispatchEvent(new CustomEvent('show-add-host'));
    } else if (item.type === 'action') {
       window.dispatchEvent(new CustomEvent('quick-action', { detail: item.id }));
    }
  }

  onMount(() => {
    loadHosts();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex h-screen overflow-hidden bg-surface-near-black font-sans theme-emerald">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="flex items-center gap-2">
        <div class="w-6 h-6 flex items-center justify-center text-brand-green">
          <svg viewBox="0 0 24 24" class="w-5 h-5 fill-current" xmlns="http://www.w3.org/2000/svg">
            <path d="M21.362 9.354H12V.396L2.638 14.646H12v8.958z"/>
          </svg>
        </div>
        <span class="font-normal text-sm tracking-tight text-neutral-off-white">BentoSSH</span>
      </div>
    </div>

    <div class="sidebar-content">
      <div class="sidebar-section">
        <div class="sidebar-section-title">Saved Hosts</div>
        {#each hosts as host}
          <button 
            type="button"
            class="sidebar-item group w-full text-left" 
            onclick={() => window.dispatchEvent(new CustomEvent('connect-host', { detail: host }))}
          >
            <Server size={14} class="text-neutral-mid group-hover:text-brand-green transition-colors" />
            <span class="truncate">{host.name}</span>
          </button>
        {/each}
        <button 
          type="button"
          class="sidebar-item group w-full text-left text-neutral-mid mt-1 hover:text-neutral-off-white" 
          onclick={() => window.dispatchEvent(new CustomEvent('show-add-host'))}
        >
          <Plus size={14} />
          <span>Add New Host</span>
        </button>
      </div>

      <div class="sidebar-section mt-6">
        <div class="sidebar-section-title">Maintenance</div>
        <button 
          type="button" 
          class="sidebar-item group w-full text-left"
          onclick={() => uiState.showHealth = true}
        >
          <Activity size={14} class="text-neutral-mid group-hover:text-neutral-off-white" />
          <span>Server Health</span>
        </button>
        <button 
          type="button"
          class="sidebar-item group w-full text-left" 
          onclick={() => uiState.isPaletteOpen = true}
        >
          <Search size={14} class="text-neutral-mid group-hover:text-neutral-off-white" />
          <span>Find...</span>
          <span class="ml-auto text-[10px] text-neutral-dark group-hover:text-neutral-mid uppercase">⌘{settingsState.current.keyboard.commandPalette}</span>
        </button>
      </div>
    </div>

    <div class="p-4 border-t border-border-dark flex items-center justify-between">
      <div class="flex items-center gap-2 text-xs text-neutral-mid">
        <div class="w-1.5 h-1.5 rounded-full bg-brand-green"></div>
        <span>System Ready</span>
      </div>
      <button 
        type="button" 
        class="text-neutral-mid hover:text-neutral-off-white cursor-pointer transition-colors p-1 hover:bg-white/5 rounded-std"
        onclick={() => uiState.showSettings = true}
        aria-label="Settings"
      >
        <Settings size={14} />
      </button>
    </div>
  </aside>

  <!-- Main Content Area -->
  <main class="flex-1 flex flex-col bg-surface-near-black">
    {@render children()}
  </main>
</div>


<CommandPalette
  isOpen={uiState.isPaletteOpen}
  items={paletteItems}
  onclose={() => uiState.isPaletteOpen = false}
  onselect={handlePaletteSelect}
/>

<SettingsModal />
<HealthDashboard />

<Toaster theme="dark" position="bottom-right" />
