<script lang="ts">
  import { X, Shield, Keyboard, Info, Bell, Terminal, Activity, Check, RotateCcw, Github, ExternalLink, FolderOpen } from 'lucide-svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { uiState } from '$lib/state.svelte';
  import { settingsState } from '$lib/settings.svelte';
  import { fade, scale } from 'svelte/transition';

  let activeTab = $state('terminal');

  const tabs = [
    { id: 'terminal', label: 'Terminal', icon: Terminal },
    { id: 'security', label: 'Security', icon: Shield },
    { id: 'keyboard', label: 'Keyboard', icon: Keyboard },
    { id: 'notifications', label: 'Notifications', icon: Bell },
    { id: 'about', label: 'About', icon: Info },
  ];

  function close() {
    uiState.showSettings = false;
  }

  function handleSave() {
    settingsState.update(settingsState.current);
    close();
  }

  function handleReset() {
    if (confirm('Are you sure you want to reset all settings to default?')) {
      settingsState.reset();
    }
  }

  async function handleBrowseKeyPath() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: 'Select SSH Private Key',
        filters: [{
          name: 'SSH Key',
          extensions: ['*', 'pub', 'key', 'pem']
        }]
      });
      
      if (selected && typeof selected === 'string') {
        settingsState.current.security.defaultKeyPath = selected;
      }
    } catch (e) {
      console.error('Failed to open dialog', e);
    }
  }
</script>

{#if uiState.showSettings}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 backdrop-blur-sm" 
    onclick={close}
    transition:fade={{ duration: 200 }}
  >
    <div 
      class="border border-border-std rounded-large w-[850px] h-[600px] shadow-2xl overflow-hidden flex" 
      style="background-color: var(--color-surface-glass); backdrop-filter: blur(var(--glass-blur));"
      onclick={(e) => e.stopPropagation()}
      transition:scale={{ duration: 200, start: 0.95 }}
    >
      <!-- Sidebar -->
      <div class="w-52 border-r border-border-dark flex flex-col p-4 bg-surface-near-black/50">
        <h2 class="text-xs font-mono uppercase tracking-widest text-neutral-dark mb-6 px-3">Settings</h2>
        <div class="space-y-1">
          {#each tabs as tab}
            <button
              class="w-full flex items-center gap-3 px-3 py-2 rounded-std text-sm transition-colors {activeTab === tab.id ? 'bg-brand-green/10 text-brand-green' : 'text-neutral-mid hover:text-neutral-off-white hover:bg-white/5'}"
              onclick={() => activeTab = tab.id}
            >
              <tab.icon size={16} />
              <span>{tab.label}</span>
            </button>
          {/each}
        </div>
        
        <button 
          class="mt-auto flex items-center gap-2 px-3 py-4 text-xs text-neutral-dark hover:text-crimson-400 transition-colors border-t border-border-dark/50"
          onclick={handleReset}
        >
          <RotateCcw size={12} />
          <span>Reset to Defaults</span>
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 flex flex-col">
        <div class="px-8 py-5 border-b border-border-dark flex justify-between items-center bg-surface-near-black/30">
          <div>
            <h3 class="text-lg font-normal text-neutral-off-white capitalize">{activeTab}</h3>
            <p class="text-xs text-neutral-dark">Manage your {activeTab} preferences and defaults.</p>
          </div>
          <button onclick={close} class="text-neutral-mid hover:text-neutral-off-white transition-colors p-2 hover:bg-white/5 rounded-full">
            <X size={20} />
          </button>
        </div>

        <div class="flex-1 overflow-y-auto p-8 custom-scrollbar">
          {#if activeTab === 'terminal'}
             <div class="space-y-8">
              <section>
                <h4 class="mono-label text-[10px] mb-4 text-neutral-mid">Font Family</h4>
                <select 
                  bind:value={settingsState.current.terminal.fontFamily}
                  class="w-full bg-surface-near-black border border-border-dark rounded-std px-3 py-2 text-sm outline-none focus:border-brand-green/50 transition-colors"
                >
                  <option value="JetBrains Mono">JetBrains Mono</option>
                  <option value="Fira Code">Fira Code</option>
                  <option value="Source Code Pro">Source Code Pro</option>
                  <option value="Menlo">Menlo / Monaco</option>
                </select>
              </section>
              
              <section class="grid grid-cols-2 gap-8">
                <div>
                  <h4 class="mono-label text-[10px] mb-4 text-neutral-mid">Scrollback Lines</h4>
                  <input 
                    type="number" 
                    bind:value={settingsState.current.terminal.scrollback}
                    class="w-full bg-surface-near-black border border-border-dark rounded-std px-3 py-2 text-sm outline-none focus:border-brand-green/50 transition-colors"
                    min="100"
                    max="10000"
                  />
                </div>
                <div>
                  <h4 class="mono-label text-[10px] mb-4 text-neutral-mid">Terminal Color Scheme</h4>
                  <select 
                    bind:value={settingsState.current.terminal.theme}
                    class="w-full bg-surface-near-black border border-border-dark rounded-std px-3 py-2 text-sm outline-none focus:border-brand-green/50 transition-colors"
                  >
                    <option value="dracula">Dracula</option>
                    <option value="nord">Nord</option>
                    <option value="one-dark">One Dark</option>
                    <option value="standard">Standard Dark</option>
                  </select>
                </div>
              </section>

              <section>
                <div class="flex justify-between items-center mb-4">
                  <h4 class="mono-label text-[10px] text-neutral-mid">Font Size</h4>
                  <span class="text-xs font-mono text-brand-green">{settingsState.current.terminal.fontSize}px</span>
                </div>
                <input 
                  type="range" 
                  min="10" 
                  max="24" 
                  bind:value={settingsState.current.terminal.fontSize} 
                  class="w-full accent-brand-green" 
                />
              </section>

              <section class="grid grid-cols-2 gap-8 pt-6 border-t border-border-dark">
                <div>
                  <h4 class="mono-label text-[10px] mb-4 text-neutral-mid">Cursor Style</h4>
                  <div class="grid grid-cols-3 gap-2">
                    {#each ['block', 'underline', 'bar'] as style}
                      <button 
                        class="py-2 border text-xs rounded-std capitalize transition-colors {settingsState.current.terminal.cursorStyle === style ? 'border-brand-green bg-brand-green/5 text-neutral-off-white' : 'border-border-dark bg-surface-near-black text-neutral-mid hover:border-border-std'}"
                        onclick={() => settingsState.current.terminal.cursorStyle = style as any}
                      >{style}</button>
                    {/each}
                  </div>
                </div>
                <div>
                  <h4 class="mono-label text-[10px] mb-4 text-neutral-mid">Cursor Blink</h4>
                  <button 
                    class="w-full py-2 border text-xs rounded-std transition-colors {settingsState.current.terminal.cursorBlink ? 'border-brand-green bg-brand-green/5 text-neutral-off-white' : 'border-border-dark bg-surface-near-black text-neutral-mid'}"
                    onclick={() => settingsState.current.terminal.cursorBlink = !settingsState.current.terminal.cursorBlink}
                  >
                    {settingsState.current.terminal.cursorBlink ? 'Enabled' : 'Disabled'}
                  </button>
                </div>
              </section>
            </div>
          {:else if activeTab === 'security'}
            <div class="space-y-8">
              <section>
                <h4 class="mono-label text-[10px] mb-4 text-neutral-mid">Default Private Key Path</h4>
                <div class="flex gap-2">
                  <input 
                    bind:value={settingsState.current.security.defaultKeyPath}
                    class="flex-1 bg-surface-near-black border border-border-dark rounded-std px-3 py-2 text-sm outline-none focus:border-brand-green/50 transition-colors" 
                    placeholder="~/.ssh/id_rsa"
                  />
                  <button 
                    class="px-4 bg-surface-near-black border border-border-dark rounded-std text-xs hover:border-border-std flex items-center gap-2"
                    onclick={handleBrowseKeyPath}
                  >
                    <FolderOpen size={14} />
                    <span>Browse</span>
                  </button>
                </div>
              </section>

              <section class="pt-6 border-t border-border-dark">
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="mono-label text-[10px] text-neutral-mid">Save Passwords</h4>
                    <p class="text-xs text-neutral-dark mt-1">Securely store passwords in your system keychain.</p>
                  </div>
                  <button 
                    class="w-10 h-5 rounded-full transition-colors relative {settingsState.current.security.savePasswords ? 'bg-brand-green' : 'bg-neutral-dark'}"
                    onclick={() => settingsState.current.security.savePasswords = !settingsState.current.security.savePasswords}
                    aria-label="Toggle Save Passwords"
                    aria-pressed={settingsState.current.security.savePasswords}
                  >
                    <div class="absolute top-1 w-3 h-3 bg-white rounded-full transition-all {settingsState.current.security.savePasswords ? 'left-6' : 'left-1'}"></div>
                  </button>
                </div>
              </section>
            </div>
          {:else if activeTab === 'keyboard'}
            <div class="space-y-8">
              <section>
                <h4 class="mono-label text-[10px] mb-4 text-neutral-mid">Application Shortcuts</h4>
                <div class="space-y-4">
                  <div class="flex items-center justify-between p-3 bg-surface-near-black border border-border-dark rounded-std">
                    <span class="text-sm text-neutral-mid">Command Palette</span>
                    <div class="flex items-center gap-1">
                      <kbd class="px-2 py-1 bg-border-dark rounded text-[10px] font-mono">⌘</kbd>
                      <input 
                        bind:value={settingsState.current.keyboard.commandPalette}
                        class="w-8 bg-transparent border-b border-brand-green text-center text-sm uppercase outline-none" 
                        maxlength="1"
                      />
                    </div>
                  </div>
                  <div class="flex items-center justify-between p-3 bg-surface-near-black border border-border-dark rounded-std">
                    <span class="text-sm text-neutral-mid">New Session</span>
                    <div class="flex items-center gap-1">
                      <kbd class="px-2 py-1 bg-border-dark rounded text-[10px] font-mono">⌘</kbd>
                      <input 
                        bind:value={settingsState.current.keyboard.newSession}
                        class="w-8 bg-transparent border-b border-brand-green text-center text-sm uppercase outline-none" 
                        maxlength="1"
                      />
                    </div>
                  </div>
                  <div class="flex items-center justify-between p-3 bg-surface-near-black border border-border-dark rounded-std">
                    <span class="text-sm text-neutral-mid">Close Session</span>
                    <div class="flex items-center gap-1">
                      <kbd class="px-2 py-1 bg-border-dark rounded text-[10px] font-mono">⌘</kbd>
                      <input 
                        bind:value={settingsState.current.keyboard.closeSession}
                        class="w-8 bg-transparent border-b border-brand-green text-center text-sm uppercase outline-none" 
                        maxlength="1"
                      />
                    </div>
                  </div>
                </div>
              </section>

              <section class="pt-6 border-t border-border-dark">
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="mono-label text-[10px] text-neutral-mid">Paste on Right Click</h4>
                    <p class="text-xs text-neutral-dark mt-1">Instantly paste clipboard content on mouse right click.</p>
                  </div>
                  <button 
                    class="w-10 h-5 rounded-full transition-colors relative {settingsState.current.keyboard.pasteOnRightClick ? 'bg-brand-green' : 'bg-neutral-dark'}"
                    onclick={() => settingsState.current.keyboard.pasteOnRightClick = !settingsState.current.keyboard.pasteOnRightClick}
                    aria-label="Toggle Paste on Right Click"
                    aria-pressed={settingsState.current.keyboard.pasteOnRightClick}
                  >
                    <div class="absolute top-1 w-3 h-3 bg-white rounded-full transition-all {settingsState.current.keyboard.pasteOnRightClick ? 'left-6' : 'left-1'}"></div>
                  </button>
                </div>
              </section>
            </div>
          {:else if activeTab === 'notifications'}
            <div class="space-y-8">
              <div class="space-y-6">
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="mono-label text-[10px] text-neutral-mid">Connection Alerts</h4>
                    <p class="text-xs text-neutral-dark mt-1">Show desktop notifications for connection events.</p>
                  </div>
                  <button 
                    class="w-10 h-5 rounded-full transition-colors relative {settingsState.current.notifications.connectionAlerts ? 'bg-brand-green' : 'bg-neutral-dark'}"
                    onclick={() => settingsState.current.notifications.connectionAlerts = !settingsState.current.notifications.connectionAlerts}
                    aria-label="Toggle Connection Alerts"
                    aria-pressed={settingsState.current.notifications.connectionAlerts}
                  >
                    <div class="absolute top-1 w-3 h-3 bg-white rounded-full transition-all {settingsState.current.notifications.connectionAlerts ? 'left-6' : 'left-1'}"></div>
                  </button>
                </div>

                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="mono-label text-[10px] text-neutral-mid">Terminal Bell</h4>
                    <p class="text-xs text-neutral-dark mt-1">Play a sound when the terminal bell is triggered.</p>
                  </div>
                  <button 
                    class="w-10 h-5 rounded-full transition-colors relative {settingsState.current.notifications.terminalBell ? 'bg-brand-green' : 'bg-neutral-dark'}"
                    onclick={() => settingsState.current.notifications.terminalBell = !settingsState.current.notifications.terminalBell}
                    aria-label="Toggle Terminal Bell"
                    aria-pressed={settingsState.current.notifications.terminalBell}
                  >
                    <div class="absolute top-1 w-3 h-3 bg-white rounded-full transition-all {settingsState.current.notifications.terminalBell ? 'left-6' : 'left-1'}"></div>
                  </button>
                </div>

                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="mono-label text-[10px] text-neutral-mid">Command Completion</h4>
                    <p class="text-xs text-neutral-dark mt-1">Notify when a long-running background command finishes.</p>
                  </div>
                  <button 
                    class="w-10 h-5 rounded-full transition-colors relative {settingsState.current.notifications.commandFinished ? 'bg-brand-green' : 'bg-neutral-dark'}"
                    onclick={() => settingsState.current.notifications.commandFinished = !settingsState.current.notifications.commandFinished}
                    aria-label="Toggle Command Completion Notifications"
                    aria-pressed={settingsState.current.notifications.commandFinished}
                  >
                    <div class="absolute top-1 w-3 h-3 bg-white rounded-full transition-all {settingsState.current.notifications.commandFinished ? 'left-6' : 'left-1'}"></div>
                  </button>
                </div>
              </div>
            </div>
          {:else if activeTab === 'about'}
            <div class="h-full flex flex-col items-center justify-center text-center space-y-6">
              <div class="w-20 h-20 text-brand-green">
                <svg viewBox="0 0 24 24" class="w-full h-full fill-current" xmlns="http://www.w3.org/2000/svg">
                  <path d="M21.362 9.354H12V.396L2.638 14.646H12v8.958z"/>
                </svg>
              </div>
              <div>
                <h2 class="text-2xl font-normal text-neutral-off-white">BentoSSH</h2>
                <p class="text-sm text-neutral-mid mt-1">Version 0.1.0 (Alpha)</p>
              </div>
              <p class="text-sm text-neutral-dark max-w-sm">
                A premium, privacy-first SSH client designed for the modern developer soul. Built with Rust, Tauri, and Svelte.
              </p>
              <div class="flex gap-4">
                <a href="https://github.com/bentossh/bentossh" target="_blank" class="flex items-center gap-2 px-4 py-2 bg-surface-near-black border border-border-dark rounded-std text-xs text-neutral-mid hover:text-neutral-off-white hover:border-border-std transition-all">
                  <Github size={14} />
                  <span>GitHub</span>
                </a>
                <a href="https://bentossh.com" target="_blank" class="flex items-center gap-2 px-4 py-2 bg-surface-near-black border border-border-dark rounded-std text-xs text-neutral-mid hover:text-neutral-off-white hover:border-border-std transition-all">
                  <ExternalLink size={14} />
                  <span>Website</span>
                </a>
              </div>
              <div class="pt-8 text-[10px] text-neutral-dark font-mono">
                © 2026 BentoSSH Contributors. Released under MIT License.
              </div>
            </div>
          {/if}
        </div>
        
        <div class="px-8 py-5 bg-surface-near-black border-t border-border-dark flex justify-end gap-3 shadow-inner">
          <button class="px-6 py-2 text-sm text-neutral-mid hover:text-neutral-off-white transition-colors" onclick={close}>Cancel</button>
          <button class="btn-primary-pill !px-10 !py-2.5" onclick={handleSave}>Save Preferences</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .btn-primary-pill {
    background: #0f0f0f;
    color: #fafafa;
    border-radius: 9999px;
    border: 1px solid #fafafa;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s;
  }
  
  .btn-primary-pill:hover {
    background: #1a1a1a;
    border-color: #3ecf8e;
    color: #3ecf8e;
  }

  .rounded-std { border-radius: 6px; }
  .rounded-large { border-radius: 12px; }
</style>
