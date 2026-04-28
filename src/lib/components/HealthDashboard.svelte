<script lang="ts">
  import { X, Activity, HardDrive, Server, ExternalLink, RefreshCw, ChevronRight } from 'lucide-svelte';
  import { uiState } from '$lib/state.svelte';
  import { sessionState } from '$lib/session.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';

  interface SavedHost {
    id: number;
    name: string;
    ip: string;
    port: number;
    username: string;
    last_connected: string;
  }

  let hosts = $state<SavedHost[]>([]);
  let isLoading = $state(true);

  async function loadHosts() {
    try {
      hosts = await invoke('get_hosts') as SavedHost[];
    } catch (e) {
      console.error('Failed to load hosts', e);
    } finally {
      isLoading = false;
    }
  }

  function getSessionForHost(hostId: number) {
    return sessionState.sessions.find(s => s.host.id === hostId);
  }

  onMount(() => {
    loadHosts();
    window.addEventListener('refresh-hosts', loadHosts);
    return () => window.removeEventListener('refresh-hosts', loadHosts);
  });
</script>

{#if uiState.showHealth}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-md"
    onclick={() => uiState.showHealth = false}
    transition:fade={{ duration: 200 }}
  >
    <div 
      class="bg-surface-dark border border-border-std rounded-large w-[900px] max-h-[80vh] shadow-2xl flex flex-col overflow-hidden"
      onclick={(e) => e.stopPropagation()}
      transition:scale={{ duration: 200, start: 0.95 }}
    >
      <!-- Header -->
      <div class="px-8 py-6 border-b border-border-dark flex justify-between items-center bg-surface-dark/50">
        <div class="flex items-center gap-4">
          <div class="w-10 h-10 rounded-comfortable bg-brand-green/10 flex items-center justify-center text-brand-green">
            <Activity size={24} />
          </div>
          <div>
            <h3 class="text-xl font-medium text-neutral-off-white leading-none mb-1">Server Health Dashboard</h3>
            <p class="text-xs text-neutral-mid uppercase tracking-widest font-mono">Real-time infrastructure monitoring</p>
          </div>
        </div>
        <button 
          onclick={() => uiState.showHealth = false}
          class="p-2 hover:bg-white/5 rounded-std text-neutral-mid hover:text-neutral-off-white transition-all"
        >
          <X size={20} />
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-8 bg-surface-near-black/30">
        {#if isLoading}
          <div class="h-64 flex flex-col items-center justify-center gap-4">
            <RefreshCw size={32} class="text-brand-green animate-spin" />
            <span class="text-neutral-mid font-mono text-xs uppercase tracking-widest">Gathering metrics...</span>
          </div>
        {:else if hosts.length === 0}
          <div class="h-64 flex flex-col items-center justify-center text-center">
            <Server size={48} class="text-neutral-dark mb-4" />
            <p class="text-neutral-mid mb-4">No saved hosts found to monitor.</p>
            <button 
              class="btn-primary-pill"
              onclick={() => { uiState.showHealth = false; uiState.showAddHost = true; }}
            >
              Add your first host
            </button>
          </div>
        {:else}
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            {#each hosts as host}
              {@const session = getSessionForHost(host.id)}
              <div class="card group border-border-dark hover:border-brand-green/30 transition-all duration-300 relative overflow-hidden">
                <!-- Status indicator glow -->
                <div class="absolute -top-12 -right-12 w-24 h-24 blur-3xl opacity-10 transition-opacity group-hover:opacity-20 {session ? 'bg-brand-green' : 'bg-neutral-mid'}"></div>
                
                <div class="flex justify-between items-start mb-6">
                  <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-std bg-surface-dark flex items-center justify-center border border-border-dark group-hover:border-brand-green/20">
                      <Server size={14} class={session ? 'text-brand-green' : 'text-neutral-mid'} />
                    </div>
                    <div>
                      <h4 class="text-sm font-medium text-neutral-off-white group-hover:text-brand-green transition-colors">{host.name}</h4>
                      <p class="text-[10px] font-mono text-neutral-dark">{host.username}@{host.ip}</p>
                    </div>
                  </div>
                  <div class="flex flex-col items-end gap-1">
                    <div class="flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] uppercase font-mono tracking-wider {session ? 'bg-brand-green/10 text-brand-green border border-brand-green/20' : 'bg-neutral-dark/10 text-neutral-mid border border-border-dark'}">
                      <div class="w-1 h-1 rounded-full {session ? 'bg-brand-green animate-pulse' : 'bg-neutral-mid'}"></div>
                      {session ? 'Connected' : 'Offline'}
                    </div>
                  </div>
                </div>

                {#if session && sessionState.healthData[session.id]}
                  <div class="grid grid-cols-2 gap-4 mb-6">
                    <!-- CPU Meter -->
                    <div class="space-y-2">
                      <div class="flex justify-between items-end">
                        <span class="text-[10px] uppercase font-mono tracking-widest text-neutral-mid">CPU Load</span>
                        <span class="text-xs font-mono text-neutral-off-white">{sessionState.healthData[session.id].cpu}%</span>
                      </div>
                      <div class="h-1 w-full bg-surface-dark rounded-full overflow-hidden border border-border-dark/50">
                        <div 
                          class="h-full bg-brand-green transition-all duration-1000 ease-out" 
                          style="width: {sessionState.healthData[session.id].cpu}%"
                        ></div>
                      </div>
                    </div>
                    <!-- RAM Meter -->
                    <div class="space-y-2">
                      <div class="flex justify-between items-end">
                        <span class="text-[10px] uppercase font-mono tracking-widest text-neutral-mid">Memory</span>
                        <span class="text-xs font-mono text-neutral-off-white">{sessionState.healthData[session.id].ram}%</span>
                      </div>
                      <div class="h-1 w-full bg-surface-dark rounded-full overflow-hidden border border-border-dark/50">
                        <div 
                          class="h-full bg-brand-link transition-all duration-1000 ease-out" 
                          style="width: {sessionState.healthData[session.id].ram}%"
                        ></div>
                      </div>
                    </div>
                  </div>
                {:else}
                  <div class="h-[52px] flex items-center justify-center border border-dashed border-border-dark rounded-std mb-6">
                    <span class="text-[10px] font-mono text-neutral-dark uppercase tracking-widest">Connect to view metrics</span>
                  </div>
                {/if}

                <div class="flex justify-between items-center pt-4 border-t border-border-dark">
                  <span class="text-[9px] font-mono text-neutral-dark uppercase">Last seen: {host.last_connected ? new Date(host.last_connected).toLocaleDateString() : 'Never'}</span>
                  {#if !session}
                    <button 
                      class="flex items-center gap-1.5 text-[10px] font-medium text-neutral-mid hover:text-brand-green transition-colors uppercase tracking-widest"
                      onclick={() => sessionState.connectToHost(host)}
                    >
                      Connect <ChevronRight size={12} />
                    </button>
                  {:else}
                     <button 
                      class="flex items-center gap-1.5 text-[10px] font-medium text-neutral-mid hover:text-neutral-off-white transition-colors uppercase tracking-widest"
                      onclick={() => { uiState.showHealth = false; sessionState.activeSessionId = session.id; }}
                    >
                      Switch to Tab <ExternalLink size={12} />
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-8 py-4 bg-surface-near-black border-t border-border-dark flex justify-between items-center">
        <div class="flex items-center gap-6 text-[10px] font-mono text-neutral-dark uppercase tracking-widest">
           <div class="flex items-center gap-2">
             <div class="w-2 h-2 rounded-full bg-brand-green"></div>
             <span>{sessionState.sessions.length} Active Sessions</span>
           </div>
           <div class="flex items-center gap-2">
             <div class="w-2 h-2 rounded-full bg-neutral-dark"></div>
             <span>{hosts.length - sessionState.sessions.length} Saved</span>
           </div>
        </div>
        <button 
          class="btn-ghost text-xs px-4"
          onclick={() => uiState.showHealth = false}
        >
          Close Dashboard
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .card {
    background: var(--surface-dark);
    padding: 1.5rem;
    border-radius: var(--radius-comfortable);
  }
</style>
