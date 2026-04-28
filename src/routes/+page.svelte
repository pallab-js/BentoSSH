<script lang="ts">
  import Terminal from '$lib/components/Terminal.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { X, Plus, Terminal as TerminalIcon, HardDrive, Zap, RefreshCw, Command, Activity } from 'lucide-svelte';
  import { toast } from 'svelte-sonner';
  import { onMount } from 'svelte';
  import { uiState } from '$lib/state.svelte';
  import { settingsState } from '$lib/settings.svelte';

  import { sessionState } from '$lib/session.svelte';

  let newHost = $state({ name: '', ip: '', port: 22, username: '', password: '', keyPath: '' });

  async function connectToHost(host: any) {
    await sessionState.connectToHost(host);
  }

  function closeSession(sessionId: string) {
    sessionState.closeSession(sessionId);
  }

  async function saveHost() {
    try {
      await invoke('save_host', {
        name: newHost.name,
        ip: newHost.ip,
        port: newHost.port,
        username: newHost.username,
        password: newHost.password || null,
        keyPath: newHost.keyPath || null
      });
      uiState.showAddHost = false;
      newHost = { name: '', ip: '', port: 22, username: '', password: '', keyPath: '' };
      // Refresh hosts in layout
      window.dispatchEvent(new CustomEvent('refresh-hosts'));
      toast.success('Host saved successfully');
    } catch (e) {
      toast.error('Save host failed: ' + e);
    }
  }

  async function handleQuickAction(action: string) {
    if (!sessionState.activeSessionId) {
      toast.error('No active session selected');
      return;
    }
    try {
      await invoke('quick_action', { sessionId: sessionState.activeSessionId, action });
      toast.success(`Action '${action}' triggered`);
    } catch (e) {
      toast.error('Action failed: ' + e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    const newSessionKey = settingsState.current.keyboard.newSession.toLowerCase();
    const closeSessionKey = settingsState.current.keyboard.closeSession.toLowerCase();

    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === newSessionKey) {
      e.preventDefault();
      uiState.showAddHost = true;
    }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === closeSessionKey) {
      e.preventDefault();
      if (sessionState.activeSessionId) closeSession(sessionState.activeSessionId);
    }
    if ((e.metaKey || e.ctrlKey) && (e.key === '[' || e.key === '{' || e.shiftKey && e.key === '[')) {
      // Switch to previous tab
      const idx = sessionState.sessions.findIndex(s => s.id === sessionState.activeSessionId);
      if (idx > 0) sessionState.activeSessionId = sessionState.sessions[idx - 1].id;
    }
    if ((e.metaKey || e.ctrlKey) && (e.key === ']' || e.key === '}' || e.shiftKey && e.key === ']')) {
      // Switch to next tab
      const idx = sessionState.sessions.findIndex(s => s.id === sessionState.activeSessionId);
      if (idx < sessionState.sessions.length - 1) sessionState.activeSessionId = sessionState.sessions[idx + 1].id;
    }
  }

  onMount(() => {
    const connectHandler = (e: any) => connectToHost(e.detail);
    const showAddHandler = () => uiState.showAddHost = true;
    const actionHandler = (e: any) => handleQuickAction(e.detail);

    window.addEventListener('connect-host', connectHandler);
    window.addEventListener('show-add-host', showAddHandler);
    window.addEventListener('quick-action', actionHandler);

    return () => {
      window.removeEventListener('connect-host', connectHandler);
      window.removeEventListener('show-add-host', showAddHandler);
      window.removeEventListener('quick-action', actionHandler);
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Tab Bar -->
<div class="tab-bar">
  {#each sessionState.sessions as session (session.id)}
    <div
      role="button"
      tabindex="0"
      class="tab {sessionState.activeSessionId === session.id ? 'active' : ''}"
      onclick={() => sessionState.activeSessionId = session.id}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') sessionState.activeSessionId = session.id; }}
    >
      <TerminalIcon size={12} class={sessionState.activeSessionId === session.id ? 'text-brand-green' : ''} />
      <span class="truncate">{session.name}</span>
      <button
        type="button"
        class="tab-close ml-auto hover:bg-white/10 rounded p-0.5"
        onclick={(e) => { e.stopPropagation(); closeSession(session.id); }}
      >
        <X size={10} />
      </button>
    </div>
  {/each}
  <button
    class="px-3 flex items-center justify-center hover:bg-white/5 text-neutral-mid transition-colors"
    onclick={() => uiState.showAddHost = true}
  >
    <Plus size={14} />
  </button>
</div>

<!-- Terminal Area -->
<div class="terminal-container">
  {#if sessionState.sessions.length === 0}
    <div class="absolute inset-0 flex flex-col items-center justify-center text-neutral-mid gap-12 bg-surface-dark">
      <div class="flex flex-col items-center">
        <div class="w-16 h-16 mb-8 text-brand-green">
          <svg viewBox="0 0 24 24" class="w-full h-full fill-current" xmlns="http://www.w3.org/2000/svg">
            <path d="M21.362 9.354H12V.396L2.638 14.646H12v8.958z"/>
          </svg>
        </div>
        <div class="text-center">
          <h1 class="hero-text text-neutral-off-white mb-4">BentoSSH</h1>
          <p class="text-[18px] text-neutral-mid font-sans max-w-md mx-auto leading-relaxed">
            A premium, high-performance SSH client designed for the modern developer soul.
          </p>
        </div>
      </div>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 max-w-2xl w-full px-8">
        <div class="card flex flex-col gap-4 border-border-dark hover:border-border-std transition-colors">
          <div class="mono-label">Quick Connect</div>
          <div class="flex flex-col gap-2">
            <button type="button" class="w-full flex justify-between items-center bg-surface-near-black border border-border-dark px-3 py-2 rounded-std group cursor-pointer hover:border-brand-green/30" onclick={() => uiState.isPaletteOpen = true}>
              <span class="text-sm">Search Saved Hosts</span>
              <kbd class="text-[10px] text-neutral-dark font-mono group-hover:text-neutral-mid uppercase">⌘{settingsState.current.keyboard.commandPalette}</kbd>
            </button>
            <button type="button" class="w-full flex justify-between items-center bg-surface-near-black border border-border-dark px-3 py-2 rounded-std group cursor-pointer hover:border-brand-green/30" onclick={() => uiState.showAddHost = true}>
              <span class="text-sm">Create New Session</span>
              <kbd class="text-[10px] text-neutral-dark font-mono group-hover:text-neutral-mid uppercase">⌘{settingsState.current.keyboard.newSession}</kbd>
            </button>
          </div>
        </div>
        
        <div class="card flex flex-col gap-4 border-border-dark hover:border-border-std transition-colors">
          <div class="mono-label">Navigation</div>
          <div class="flex flex-col gap-2">
            <div class="flex justify-between items-center bg-surface-near-black border border-border-dark px-3 py-2 rounded-std">
              <span class="text-sm">Switch Active Tabs</span>
              <kbd class="text-[10px] text-neutral-dark font-mono">⌘⇧[ / ]</kbd>
            </div>
            <div class="flex justify-between items-center bg-surface-near-black border border-border-dark px-3 py-2 rounded-std">
              <span class="text-sm">Close Current Session</span>
              <kbd class="text-[10px] text-neutral-dark font-mono uppercase">⌘{settingsState.current.keyboard.closeSession}</kbd>
            </div>
          </div>
        </div>
      </div>

      <button class="btn-primary-pill" onclick={() => uiState.showAddHost = true}>
        Start your project
      </button>
    </div>
  {:else}
    {#each sessionState.sessions as session (session.id)}
      <div class="h-full {sessionState.activeSessionId === session.id ? 'block' : 'hidden'}">
        <Terminal
          visible={sessionState.activeSessionId === session.id}
          autoConnectId={session.id}
        />
        
        <!-- Status Overlay -->
        {#if sessionState.healthData[session.id]}
          <div class="absolute bottom-6 right-6 flex gap-3 pointer-events-none">
            <div class="bg-surface-near-black/80 backdrop-blur-md border border-border-std rounded-comfortable px-3 py-1.5 flex items-center gap-2 text-[11px]">
              <Activity size={10} class="text-brand-green" />
              <span class="text-neutral-mid uppercase tracking-wider font-mono">CPU</span>
              <span class="font-mono text-neutral-off-white">{sessionState.healthData[session.id].cpu}%</span>
            </div>
            <div class="bg-surface-near-black/80 backdrop-blur-md border border-border-std rounded-comfortable px-3 py-1.5 flex items-center gap-2 text-[11px]">
              <HardDrive size={10} class="text-brand-link" />
              <span class="text-neutral-mid uppercase tracking-wider font-mono">RAM</span>
              <span class="font-mono text-neutral-off-white">{sessionState.healthData[session.id].ram}%</span>
            </div>
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>

<!-- Add Host Modal -->
{#if uiState.showAddHost}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-[101] flex items-center justify-center bg-black/60 backdrop-blur-sm" onclick={() => uiState.showAddHost = false}>
    <div class="bg-surface-dark border border-border-std rounded-large w-[480px] shadow-2xl overflow-hidden" onclick={(e) => e.stopPropagation()}>
      <div class="px-6 py-4 border-b border-border-dark flex justify-between items-center">
        <h3 class="card-title text-[18px]">New SSH Connection</h3>
        <button onclick={() => uiState.showAddHost = false} class="text-neutral-mid hover:text-neutral-off-white transition-colors"><X size={18} /></button>
      </div>
      
      <div class="p-6 space-y-5">
        <div class="flex flex-col gap-1.5">
          <label for="host-name" class="mono-label text-[10px]">Session Label</label>
          <input id="host-name" bind:value={newHost.name} class="w-full bg-surface-near-black border border-border-dark rounded-std px-3 py-2 text-sm outline-none focus:border-brand-green/50 transition-colors" placeholder="e.g. Production API" />
        </div>
        <div class="flex flex-col gap-1.5">
          <label for="host-ip" class="mono-label text-[10px]">Host Address</label>
          <input id="host-ip" bind:value={newHost.ip} class="w-full bg-surface-near-black border border-border-dark rounded-std px-3 py-2 text-sm outline-none focus:border-brand-green/50 transition-colors" placeholder="10.0.1.1 or example.com" />
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-1.5">
            <label for="host-user" class="mono-label text-[10px]">Username</label>
            <input id="host-user" bind:value={newHost.username} class="w-full bg-surface-near-black border border-border-dark rounded-std px-3 py-2 text-sm outline-none focus:border-brand-green/50 transition-colors" placeholder="root" />
          </div>
          <div class="flex flex-col gap-1.5">
            <label for="host-port" class="mono-label text-[10px]">Port</label>
            <input id="host-port" type="number" bind:value={newHost.port} class="w-full bg-surface-near-black border border-border-dark rounded-std px-3 py-2 text-sm outline-none focus:border-brand-green/50 transition-colors" placeholder="22" />
          </div>
        </div>
        <div class="flex flex-col gap-1.5 pt-2 border-t border-border-dark">
          <label for="host-pass" class="mono-label text-[10px]">Authentication</label>
          <input id="host-pass" type="password" bind:value={newHost.password} class="w-full bg-surface-near-black border border-border-dark rounded-std px-3 py-2 text-sm outline-none focus:border-brand-green/50 transition-colors" placeholder="Password or Key Passphrase" />
        </div>
      </div>
      
      <div class="px-6 py-4 bg-surface-near-black border-t border-border-dark flex justify-end gap-3">
        <button class="btn-ghost px-4" onclick={() => uiState.showAddHost = false}>Cancel</button>
        <button class="btn-primary-pill !px-6 !py-1.5" onclick={saveHost}>Connect Session</button>
      </div>
    </div>
  </div>
{/if}

