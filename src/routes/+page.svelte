<script>
  import Terminal from '$lib/components/Terminal.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { fade, fly } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import { Server, Zap, HardDrive, RefreshCw, Plus, Trash2, X, Command } from 'lucide-svelte';
  import { toast } from 'svelte-sonner';
  import { dndzone } from 'svelte-dnd-action';
  import { flip } from 'svelte/animate';

  let theme = $state('dark');
  let hosts = $state([]);
  let sessions = $state([]);
  let activeSessionId = $state(null);
  let connecting = $state(false);
  let healthData = $state({}); // Map of sessionId to health
  let showAddHost = $state(false);
  let newHost = $state({ name: '', ip: '', port: 22, username: '', password: '', keyPath: '' });

  // Tiles now include dynamic terminals
  let tiles = $state([
    { id: 'health', cols: 4, rows: 2, title: 'Server Health' },
    { id: 'actions', cols: 4, rows: 2, title: 'Quick Actions' },
    { id: 'hosts', cols: 12, rows: 2, title: 'Pinned Hosts' }
  ]);

  const flipDurationMs = 200;

  function handleDndConsider(e) {
    tiles = e.detail.items;
  }

  function handleDndFinalize(e) {
    tiles = e.detail.items;
  }

  async function loadHosts() {
    try {
      const rawHosts = await invoke('get_hosts');
      hosts = rawHosts.map(h => ({
        id: h[0],
        name: h[1],
        ip: h[2],
        port: h[3],
        username: h[4],
        keychain_id: h[5],
        last_connected: h[6]
      }));
    } catch (e) {
      toast.error('Failed to load hosts: ' + e);
    }
  }

  async function connectToHost(host) {
    connecting = true;
    try {
      const [password, keyPath] = await invoke('get_host_creds', { id: host.id });
      const sessionId = await invoke('ssh_connect', {
        host: host.ip,
        port: host.port,
        username: host.username,
        password: password || null,
        keyPath: keyPath || null
      });

      const newSession = {
        id: sessionId,
        host,
        status: 'connected',
        creds: { password, keyPath }
      };
      
      sessions = [...sessions, newSession];
      activeSessionId = sessionId;
      
      // Add a terminal tile for this session
      tiles = [{ id: `terminal-${sessionId}`, cols: 8, rows: 4, title: host.name, sessionId }, ...tiles];
      
      startHealthPolling(sessionId);
      toast.success(`Connected to ${host.name}`);
    } catch (e) {
      toast.error('Connection failed: ' + e);
    } finally {
      connecting = false;
    }
  }

  function startHealthPolling(sessionId) {
    const poll = async () => {
      if (!sessions.find(s => s.id === sessionId)) return;
      try {
        const [cpu, ram] = await invoke('get_server_health', { sessionId });
        healthData = { ...healthData, [sessionId]: { cpu, ram } };
      } catch (e) {
        console.error('Health poll failed', e);
      }
      setTimeout(poll, 5000);
    };
    poll();
  }

  function removeSession(sessionId) {
    sessions = sessions.filter(s => s.id !== sessionId);
    tiles = tiles.filter(t => t.id !== `terminal-${sessionId}`);
    if (activeSessionId === sessionId) {
      activeSessionId = sessions[0]?.id || null;
    }
    invoke('ssh_disconnect', { sessionId }).catch(console.error);
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
      showAddHost = false;
      newHost = { name: '', ip: '', port: 22, username: '', password: '', keyPath: '' };
      loadHosts();
      toast.success('Host saved successfully');
    } catch (e) {
      toast.error('Save host failed: ' + e);
    }
  }

  async function deleteHost(id) {
    try {
      await invoke('delete_host', { id });
      hosts = hosts.filter(h => h.id !== id);
      toast.success('Host deleted');
    } catch (e) {
      toast.error('Delete host failed: ' + e);
    }
  }

  $effect(() => { loadHosts(); });
</script>

<div class="min-h-screen bg-base p-6 lg:p-12">
  <!-- Header -->
  <header class="mb-24 flex items-center justify-between">
    <div class="flex items-center gap-3">
      <div class="w-8 h-8 bg-brand-green rounded-md flex items-center justify-center text-white">
        <svg viewBox="0 0 24 24" class="w-5 h-5 fill-current" xmlns="http://www.w3.org/2000/svg">
          <path d="M21.362 9.354H12V.396L2.638 14.646H12v8.958z"/>
        </svg>
      </div>
      <h1 class="text-xl font-medium tracking-tight text-text-primary">BentoSSH</h1>
    </div>
    
    <div class="flex items-center gap-6">
      {#if sessions.length > 0}
        <div class="flex gap-1 bg-base border border-border-std p-1 rounded-full">
          {#each sessions as session (session.id)}
            <button
              class="px-4 py-1.5 text-[13px] rounded-full transition-all font-medium {activeSessionId === session.id ? 'bg-brand-green text-base' : 'text-text-muted hover:text-text-primary'}"
              onclick={() => activeSessionId = session.id}
            >
              {session.host.name}
            </button>
          {/each}
        </div>
      {/if}
      <button class="btn-pill-primary" onclick={() => showAddHost = true}>
        Start New Session
      </button>
    </div>
  </header>

  <!-- 12-Column Bento Grid -->
  <section
    class="grid grid-cols-12 gap-6"
    use:dndzone={{ items: tiles, flipDurationMs, type: 'tiles' }}
    onconsider={handleDndConsider}
    onfinalize={handleDndFinalize}
  >
    {#each tiles as tile (tile.id)}
      <div
        class="glass flex flex-col group"
        class:col-span-8={tile.cols === 8}
        class:col-span-4={tile.cols === 4}
        class:col-span-12={tile.cols === 12}
        class:connection-active={tile.sessionId === activeSessionId}
        style="grid-row: span {tile.rows};"
        animate:flip={{ duration: flipDurationMs }}
      >
        <div class="flex items-center justify-between px-5 py-4 border-b border-border-std bg-base/50">
           <div class="flex items-center gap-3">
            <span class="font-mono-label">
              {#if tile.id.startsWith('terminal')}
                SESSION
              {:else if tile.id === 'health'}
                SYSTEM
              {:else if tile.id === 'actions'}
                EXEC
              {:else}
                NETWORK
              {/if}
            </span>
            <h2 class="text-sm font-medium text-text-primary tracking-tight">
              {tile.title}
            </h2>
          </div>
          {#if tile.sessionId}
            <button class="text-text-muted hover:text-red-500 transition-colors" onclick={() => removeSession(tile.sessionId)}>
              <X size={16} />
            </button>
          {/if}
        </div>

        <div class="p-5 flex-grow overflow-hidden">
          {#if tile.id.startsWith('terminal')}
            <div class="h-full rounded-lg overflow-hidden border border-border-std bg-surface">
              <Terminal
                visible={true}
                autoConnectId={tile.sessionId}
              />
            </div>
          {:else if tile.id === 'health'}
            <!-- Server Health -->
            <div class="space-y-6">
              {#if activeSessionId && healthData[activeSessionId]}
                <div class="grid grid-cols-1 gap-6">
                  <div class="p-4 rounded-xl border border-border-std bg-base">
                    <div class="font-mono-label mb-2">CPU Utilization</div>
                    <div class="flex items-end justify-between">
                      <div class="text-4xl font-normal text-brand-green">{healthData[activeSessionId].cpu}%</div>
                      <div class="h-1 flex-grow mx-4 bg-border-std rounded-full overflow-hidden mb-2">
                        <div class="h-full bg-brand-green" style="width: {healthData[activeSessionId].cpu}%"></div>
                      </div>
                    </div>
                  </div>
                  <div class="p-4 rounded-xl border border-border-std bg-base">
                    <div class="font-mono-label mb-2">Memory Load</div>
                    <div class="flex items-end justify-between">
                      <div class="text-4xl font-normal text-brand-green">{healthData[activeSessionId].ram}%</div>
                      <div class="h-1 flex-grow mx-4 bg-border-std rounded-full overflow-hidden mb-2">
                        <div class="h-full bg-brand-green" style="width: {healthData[activeSessionId].ram}%"></div>
                      </div>
                    </div>
                  </div>
                </div>
              {:else}
                <div class="flex flex-col items-center justify-center py-12 text-center">
                  <HardDrive size={32} class="text-border-std mb-4" />
                  <p class="text-sm text-text-muted">Select an active session to monitor health metrics.</p>
                </div>
              {/if}
            </div>
          {:else if tile.id === 'actions'}
            <!-- Quick Actions -->
            <div class="grid grid-cols-1 gap-3">
              <button class="btn-pill-secondary w-full flex items-center justify-between disabled:opacity-30"
                onclick={async () => {
                  try {
                    await invoke('quick_action', { sessionId: activeSessionId, action: 'restart_service' });
                    toast.success('Service restart triggered');
                  } catch (e) { toast.error('Action failed: ' + e); }
                }}
                disabled={!activeSessionId}
              >
                <span>Restart Service</span>
                <Zap size={14} class="text-brand-green" />
              </button>
              <button class="btn-pill-secondary w-full flex items-center justify-between disabled:opacity-30"
                onclick={async () => {
                  try {
                    await invoke('quick_action', { sessionId: activeSessionId, action: 'tail_logs' });
                    toast.success('Log retrieval started');
                  } catch (e) { toast.error('Action failed: ' + e); }
                }}
                disabled={!activeSessionId}
              >
                <span>Tail Syslog</span>
                <RefreshCw size={14} class="text-brand-green" />
              </button>
              <button class="btn-pill-secondary w-full flex items-center justify-between disabled:opacity-30"
                onclick={async () => {
                  try {
                    await invoke('quick_action', { sessionId: activeSessionId, action: 'clear_cache' });
                    toast.success('Cache clearing initiated');
                  } catch (e) { toast.error('Action failed: ' + e); }
                }}
                disabled={!activeSessionId}
              >
                <span>Clear Memory Cache</span>
                <Command size={14} class="text-brand-green" />
              </button>
            </div>
          {:else if tile.id === 'hosts'}
            <!-- Pinned Hosts -->
            <div class="flex gap-6 overflow-x-auto pb-4 px-1">
              {#each hosts as host (host.id)}
                <div class="relative group flex-shrink-0">
                  <button
                    class="w-[240px] p-6 rounded-2xl bg-base border border-border-std text-left hover:border-brand-green hover:bg-[#1c1c1c] transition-all"
                    onclick={() => connectToHost(host)}
                    disabled={connecting}
                  >
                    <div class="font-mono-label mb-3 text-[10px]">HOST</div>
                    <div class="text-lg font-medium text-text-primary mb-1 tracking-tight truncate">{host.name}</div>
                    <div class="text-sm text-text-muted font-mono">{host.username}@{host.ip}</div>
                    
                    {#if connecting && activeSessionId === null}
                      <div class="absolute bottom-0 left-0 h-0.5 bg-brand-green animate-progress"></div>
                    {/if}
                  </button>
                  <button
                    class="absolute -top-2 -right-2 w-8 h-8 rounded-full bg-surface border border-border-std text-red-500 flex items-center justify-center opacity-0 group-hover:opacity-100 hover:border-red-500 transition-all"
                    onclick={(e) => { e.stopPropagation(); deleteHost(host.id); }}
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              {/each}
              
              <button
                class="w-[180px] p-6 rounded-2xl border-2 border-dashed border-border-std flex flex-col items-center justify-center text-text-muted hover:border-brand-green hover:text-brand-green transition-all flex-shrink-0 group"
                onclick={() => showAddHost = true}
              >
                <Plus size={32} class="mb-2 group-hover:scale-110 transition-transform" />
                <span class="font-mono-label text-[10px]">NEW HOST</span>
              </button>
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </section>

  <!-- Add Host Modal -->
  {#if showAddHost}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/90 backdrop-blur-md" role="presentation" onclick={() => showAddHost = false}>
      <div class="glass p-8 w-full max-w-xl bg-surface" role="presentation" onclick={(e) => e.stopPropagation()}>
        <div class="flex justify-between items-center mb-8">
          <div>
            <span class="font-mono-label block mb-1">PROVISIONING</span>
            <h3 class="text-2xl font-medium text-text-primary tracking-tight">Add New Host</h3>
          </div>
          <button onclick={() => showAddHost = false} class="text-text-muted hover:text-text-primary"><X size={24} /></button>
        </div>
        
        <div class="space-y-6">
          <div class="grid grid-cols-2 gap-6">
             <div class="col-span-2">
              <label for="host-name" class="font-mono-label block mb-2">Display Name</label>
              <input id="host-name" bind:value={newHost.name} class="w-full rounded-lg bg-base border border-border-std px-4 py-3 text-text-primary focus:border-brand-green outline-none transition-colors" placeholder="e.g. Production Cluster" />
            </div>
            <div class="col-span-2">
              <label for="host-ip" class="font-mono-label block mb-2">Hostname / IPv4 / IPv6</label>
              <input id="host-ip" bind:value={newHost.ip} class="w-full rounded-lg bg-base border border-border-std px-4 py-3 text-text-primary focus:border-brand-green outline-none transition-colors" placeholder="ssh.domain.com" />
            </div>
            <div>
              <label for="host-port" class="font-mono-label block mb-2">SSH Port</label>
              <input id="host-port" bind:value={newHost.port} type="number" class="w-full rounded-lg bg-base border border-border-std px-4 py-3 text-text-primary focus:border-brand-green outline-none" />
            </div>
            <div>
              <label for="host-username" class="font-mono-label block mb-2">Username</label>
              <input id="host-username" bind:value={newHost.username} class="w-full rounded-lg bg-base border border-border-std px-4 py-3 text-text-primary focus:border-brand-green outline-none" placeholder="root" />
            </div>
          </div>
          
          <div class="pt-4 border-t border-border-std">
            <label for="host-password" class="font-mono-label block mb-2">Authentication Password</label>
            <input id="host-password" bind:value={newHost.password} type="password" class="w-full rounded-lg bg-base border border-border-std px-4 py-3 text-text-primary focus:border-brand-green outline-none" placeholder="••••••••••••" />
          </div>

          <div>
            <label for="host-keypath" class="font-mono-label block mb-2">Private Key Identity Path</label>
            <input id="host-keypath" bind:value={newHost.keyPath} class="w-full rounded-lg bg-base border border-border-std px-4 py-3 text-text-primary focus:border-brand-green outline-none" placeholder="~/.ssh/id_rsa" />
          </div>

          <button class="btn-pill-primary w-full py-4 text-lg mt-6" onclick={saveHost}>
            Save Host Configuration
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes progress {
    0% { width: 0; }
    100% { width: 100%; }
  }
  .animate-progress {
    animation: progress 2s ease-out forwards;
  }
</style>
