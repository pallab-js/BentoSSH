import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { settingsState } from '$lib/settings.svelte';

export interface Session {
  id: string;
  name: string;
  host: any;
  status: 'connecting' | 'connected' | 'error';
}

export interface HealthData {
  cpu: string;
  ram: string;
}

function createSessionState() {
  let sessions = $state<Session[]>([]);
  let activeSessionId = $state<string | null>(null);
  let healthData = $state<Record<string, HealthData>>({}); // sessionId -> {cpu, ram}

  // Track pending poll timers so they can be cancelled on disconnect.
  const pollTimers: Record<string, ReturnType<typeof setTimeout>> = {};

  function startHealthPolling(sessionId: string) {
    const poll = async () => {
      // Stop if the session was closed.
      if (!sessions.find((s) => s.id === sessionId)) {
        delete pollTimers[sessionId];
        return;
      }
      try {
        const [cpu, ram] = (await invoke('get_server_health', { sessionId })) as [string, string];
        healthData[sessionId] = { cpu, ram };
      } catch (e) {
        // Silently ignore transient health poll failures.
      }
      // Reschedule only if still active.
      if (sessions.find((s) => s.id === sessionId)) {
        pollTimers[sessionId] = setTimeout(poll, 5000);
      }
    };
    poll();
  }

  function stopHealthPolling(sessionId: string) {
    const timer = pollTimers[sessionId];
    if (timer) {
      clearTimeout(timer);
      delete pollTimers[sessionId];
    }
  }

  return {
    get sessions() { return sessions; },
    set sessions(val) { sessions = val; },
    get activeSessionId() { return activeSessionId; },
    set activeSessionId(val) { activeSessionId = val; },
    get healthData() { return healthData; },

    async connectToHost(host: any) {
      // Create a placeholder session immediately so the tab shows a
      // "connecting" state while the backend establishes the SSH connection.
      const tempId = `pending-${crypto.randomUUID()}`;
      const placeholder: Session = { id: tempId, name: host.name, host, status: 'connecting' };
      sessions = [...sessions, placeholder];
      activeSessionId = tempId;

      try {
        const sessionId = (await invoke('connect_saved_host', { id: host.id })) as string;

        // Swap the placeholder for the real session id and mark it connected.
        sessions = sessions.map((s) =>
          s.id === tempId ? { ...s, id: sessionId, status: 'connected' as const } : s
        );
        if (activeSessionId === tempId) {
          activeSessionId = sessionId;
        }

        startHealthPolling(sessionId);
        if (settingsState.current.notifications.connectionAlerts) {
          toast.success(`Connected to ${host.name}`);
        }
        return sessionId;
      } catch (e) {
        // Mark the placeholder as errored so the tab shows a failure state;
        // leave it visible briefly, then remove it.
        sessions = sessions.map((s) => (s.id === tempId ? { ...s, status: 'error' as const } : s));
        toast.error('Connection failed: ' + e);
        setTimeout(() => {
          sessions = sessions.filter((s) => s.id !== tempId);
          if (activeSessionId === tempId) {
            activeSessionId = sessions[sessions.length - 1]?.id || null;
          }
        }, 2500);
        throw e;
      }
    },

    closeSession(sessionId: string) {
      stopHealthPolling(sessionId);
      sessions = sessions.filter(s => s.id !== sessionId);
      if (activeSessionId === sessionId) {
        activeSessionId = sessions[sessions.length - 1]?.id || null;
      }
      delete healthData[sessionId];
      invoke('ssh_disconnect', { sessionId }).catch(console.error);
    }
  };
}

export const sessionState = createSessionState();
