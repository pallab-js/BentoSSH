import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock Tauri IPC and toast before importing the store under test.
const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { sessionState } from './session.svelte';

const host = { id: 1, name: 'Prod', ip: '10.0.0.1', username: 'root' };

beforeEach(() => {
  invoke.mockReset();
  // Reset store between tests.
  sessionState.sessions = [];
  sessionState.activeSessionId = null;
});

describe('sessionState.connectToHost', () => {
  it('shows a connecting placeholder, then swaps to the connected session on success', async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === 'connect_saved_host') return Promise.resolve('real-session-id');
      if (cmd === 'get_server_health') return Promise.resolve(['1.0', '2.0']);
      return Promise.resolve();
    });

    const promise = sessionState.connectToHost(host);

    // Placeholder created synchronously before the await resolves.
    expect(sessionState.sessions).toHaveLength(1);
    expect(sessionState.sessions[0].status).toBe('connecting');

    await promise;

    expect(sessionState.sessions).toHaveLength(1);
    expect(sessionState.sessions[0].id).toBe('real-session-id');
    expect(sessionState.sessions[0].status).toBe('connected');
    expect(sessionState.activeSessionId).toBe('real-session-id');
  });

  it('marks the session as errored on failure', async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === 'connect_saved_host') return Promise.reject('auth failed');
      return Promise.resolve();
    });

    await expect(sessionState.connectToHost(host)).rejects.toBeTruthy();

    expect(sessionState.sessions).toHaveLength(1);
    expect(sessionState.sessions[0].status).toBe('error');
  });
});

describe('sessionState.closeSession', () => {
  it('removes the session, clears active id, and calls ssh_disconnect', async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === 'connect_saved_host') return Promise.resolve('sid-1');
      if (cmd === 'get_server_health') return Promise.resolve(['0', '0']);
      return Promise.resolve();
    });

    await sessionState.connectToHost(host);
    expect(sessionState.sessions).toHaveLength(1);

    invoke.mockClear();
    sessionState.closeSession('sid-1');

    expect(sessionState.sessions).toHaveLength(0);
    expect(sessionState.activeSessionId).toBeNull();
    expect(invoke).toHaveBeenCalledWith('ssh_disconnect', { sessionId: 'sid-1' });
  });
});
