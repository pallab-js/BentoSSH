<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import type { Terminal } from 'xterm';
  import type { FitAddon } from '@xterm/addon-fit';

  export const csr = true;

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
  let sessionId = $state(autoConnectId);
  let isConnected = $state(!!autoConnectId);
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
        theme: {
          background: '#0f0f0f',
          foreground: '#F5F5F5',
          cursor: '#3ecf8e',
          selectionBackground: 'rgba(62, 207, 142, 0.3)'
        },
        rows: 24,
        cols: 80
      });

      fitAddon = new FitAddon();
      term.loadAddon(fitAddon);

      term.open(terminalContainer);

      if (sessionId) {
        term.write(`Reconnected to session: ${sessionId}\r\n`);
        setupOutputListener(sessionId);
      } else {
        term.write('Terminal ready. Click a host to start SSH session.\r\n');
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

  export function connect(host: string, port: number, username: string) {
    if (!term) return;
    term.clear();
    term.write(`Connecting to ${host}:${port} as ${username}...\r\n`);
    invoke('ssh_connect', {
      host,
      port,
      username,
      password: credentials?.password || null,
      keyPath: credentials?.keyPath || null
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

<div bind:this={terminalContainer} class="w-full h-full min-h-[300px] {visible ? 'block' : 'hidden'}"></div>


<div bind:this={terminalContainer} class="w-full h-full min-h-[300px] {visible ? 'block' : 'hidden'}"></div>
