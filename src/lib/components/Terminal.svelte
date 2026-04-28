<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import type { Terminal } from 'xterm';
  import type { FitAddon } from '@xterm/addon-fit';
  import { settingsState } from '$lib/settings.svelte';

  // Props
  let { visible = true, onconnect = () => {}, onerror = () => {}, credentials = null, autoConnectId = null } = $props<{
    visible?: boolean;
    onconnect?: () => void;
    onerror?: (err: any) => void;
    credentials?: { password?: string; keyPath?: string } | null;
    autoConnectId?: string | null;
  }>();

  let terminalContainer: HTMLElement;
  let term: Terminal;
  let fitAddon: FitAddon;
  let sessionId = $state<string | null>(null);
  let isConnected = $state(false);

  const THEMES: Record<string, any> = {
    dracula: {
      background: '#282a36',
      foreground: '#f8f8f2',
      cursor: '#f8f8f2',
      selectionBackground: 'rgba(255, 255, 255, 0.3)',
      black: '#21222c',
      red: '#ff5555',
      green: '#50fa7b',
      yellow: '#f1fa8c',
      blue: '#bd93f9',
      magenta: '#ff79c6',
      cyan: '#8be9fd',
      white: '#f8f8f2',
      brightBlack: '#6272a4',
      brightRed: '#ff6e6e',
      brightGreen: '#69ff94',
      brightYellow: '#ffffa5',
      brightBlue: '#d6acff',
      brightMagenta: '#ff92df',
      brightCyan: '#a4ffff',
      brightWhite: '#ffffff'
    },
    nord: {
      background: '#2e3440',
      foreground: '#d8dee9',
      cursor: '#d8dee9',
      selectionBackground: 'rgba(255, 255, 255, 0.3)',
      black: '#3b4252',
      red: '#bf616a',
      green: '#a3be8c',
      yellow: '#ebcb8b',
      blue: '#81a1c1',
      magenta: '#b48ead',
      cyan: '#88c0d0',
      white: '#e5e9f0',
      brightBlack: '#4c566a',
      brightRed: '#bf616a',
      brightGreen: '#a3be8c',
      brightYellow: '#ebcb8b',
      brightBlue: '#81a1c1',
      brightMagenta: '#b48ead',
      brightCyan: '#8fbcbb',
      brightWhite: '#eceff4'
    },
    'one-dark': {
      background: '#282c34',
      foreground: '#abb2bf',
      cursor: '#528bff',
      selectionBackground: 'rgba(255, 255, 255, 0.3)',
      black: '#282c34',
      red: '#e06c75',
      green: '#98c379',
      yellow: '#d19a66',
      blue: '#61afef',
      magenta: '#c678dd',
      cyan: '#56b6c2',
      white: '#abb2bf',
      brightBlack: '#5c6370',
      brightRed: '#e06c75',
      brightGreen: '#98c379',
      brightYellow: '#d19a66',
      brightBlue: '#61afef',
      brightMagenta: '#c678dd',
      brightCyan: '#56b6c2',
      brightWhite: '#ffffff'
    },
    standard: {
      background: '#0f0f0f',
      foreground: '#efefef',
      cursor: '#3ecf8e',
      selectionBackground: 'rgba(62, 207, 142, 0.3)',
      black: '#0f0f0f',
      red: '#ea4e5b',
      green: '#3ecf8e',
      yellow: '#f5d90a',
      blue: '#705df2',
      magenta: '#f81ce5',
      cyan: '#79ffe1',
      white: '#efefef',
      brightBlack: '#4d4d4d',
      brightRed: '#ea4e5b',
      brightGreen: '#3ecf8e',
      brightYellow: '#f5d90a',
      brightBlue: '#705df2',
      brightMagenta: '#f81ce5',
      brightCyan: '#79ffe1',
      brightWhite: '#fafafa'
    }
  };

  // Apply terminal settings reactively
  $effect(() => {
    if (term) {
      term.options.fontFamily = settingsState.current.terminal.fontFamily;
      term.options.fontSize = settingsState.current.terminal.fontSize;
      term.options.cursorStyle = settingsState.current.terminal.cursorStyle;
      term.options.cursorBlink = settingsState.current.terminal.cursorBlink;
      term.options.scrollback = settingsState.current.terminal.scrollback;
      
      // Get base theme from presets
      const baseTheme = THEMES[settingsState.current.terminal.theme] || THEMES.standard;
      
      // Override cursor and selection with global brand color if in "standard" theme
      // or if you want it to always follow global theme. 
      // For now, let's let presets decide their own colors, but if "standard" is picked, 
      // it follows the global theme.
      if (settingsState.current.terminal.theme === 'standard') {
        const brandColor = '#3ecf8e';
        const selectionColor = 'rgba(62, 207, 142, 0.3)';
        
        baseTheme.cursor = brandColor;
        baseTheme.selectionBackground = selectionColor;
        baseTheme.green = brandColor; // Also make green match brand in standard
      }
      
      term.options.theme = baseTheme;

      // Delay fit to ensure font size changes are applied
      setTimeout(() => fitAddon?.fit(), 10);
    }
  });

  // Initialize from props
  $effect(() => {
    if (autoConnectId && !sessionId) {
      sessionId = autoConnectId;
      isConnected = true;
    }
  });
  let unlisten: UnlistenFn | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let cleanup: (() => void) | null = null;

  onMount(() => {
    const init = async () => {
      const { Terminal } = await import('xterm');
      const { FitAddon } = await import('@xterm/addon-fit');
      await import('xterm/css/xterm.css');

      term = new Terminal({
        allowProposedApi: true,
        theme: THEMES[settingsState.current.terminal.theme] || THEMES.standard,
        fontFamily: settingsState.current.terminal.fontFamily,
        fontSize: settingsState.current.terminal.fontSize,
        lineHeight: 1.4,
        cursorBlink: settingsState.current.terminal.cursorBlink,
        cursorStyle: settingsState.current.terminal.cursorStyle,
        scrollback: settingsState.current.terminal.scrollback,
        rows: 24,
        cols: 80
      });

      fitAddon = new FitAddon();
      term.loadAddon(fitAddon);

      term.open(terminalContainer);

      if (sessionId) {
        term.write(`\r\n\x1b[32m✔\x1b[0m Reconnected to session: ${sessionId}\r\n`);
        setupOutputListener(sessionId);
      } else {
        term.write('\r\n  \x1b[32mBentoSSH\x1b[0m \x1b[90m—\x1b[0m \x1b[37mTerminal Session\x1b[0m\r\n');
        term.write('  \x1b[90mPress\x1b[0m \x1b[37m⌘K\x1b[0m \x1b[90mto discover and connect to hosts\x1b[0m\r\n\r\n');
      }


      // Fit terminal initially and on resize
      setTimeout(() => fitAddon.fit(), 100);
      resizeObserver = new ResizeObserver(() => {
        if (visible && term) {
          fitAddon.fit();
          const dims = fitAddon.proposeDimensions();
          if (dims && sessionId) {
            invoke('ssh_resize', { sessionId, cols: dims.cols, rows: dims.rows }).catch(console.error);
          }
        }
      });
      resizeObserver.observe(terminalContainer);

      // Handle terminal input
      term.onData((data) => {
        if (sessionId) {
          invoke('ssh_send_input', { sessionId, data }).catch(err => {
            term.write(`\r\nInput error: ${err}\r\n`);
          });
        }
      });

      // Set cleanup function
      cleanup = () => {
        if (resizeObserver) resizeObserver.disconnect();
        if (unlisten) unlisten();
        if (term) term.dispose();
      };
    };

    init();

    return () => {
      if (cleanup) cleanup();
    };
  });

  async function setupOutputListener(id: string) {
    if (unlisten) unlisten();
    unlisten = await listen(`ssh-output-${id}`, (event: any) => {
      if (term && sessionId === id) {
        term.write(event.payload);
      }
    });
  }

  export function connect(host: string, port: number, username: string, creds?: { password?: string, keyPath?: string }) {
    if (!term) return;
    term.clear();
    term.write(`Connecting to ${host}:${port} as ${username}...\r\n`);
    
    const finalCreds = creds || credentials;

    invoke('ssh_connect', {
      host,
      port,
      username,
      password: finalCreds?.password || null,
      keyPath: finalCreds?.keyPath || null
    }).then(id => {
      sessionId = id as string;
      isConnected = true;
      term.write(`Connected! Session ID: ${id}\r\n`);
      setupOutputListener(sessionId);
      onconnect();
      setTimeout(() => fitAddon?.fit(), 100);
    }).catch(err => {
      term.write(`Connection failed: ${err}\r\n`);
      onerror(err);
    });
  }

  export function disconnect() {
    if (sessionId) {
      invoke('ssh_disconnect', { sessionId }).then(() => {
        term.write('\r\nDisconnected.\r\n');
        sessionId = null;
        isConnected = false;
      });
    }
  }

  export function write(data: string) {
    if (term) term.write(data);
  }

  export function clear() {
    if (term) term.clear();
  }
</script>

<div bind:this={terminalContainer} class="w-full h-full {visible ? 'block' : 'hidden'}"></div>
