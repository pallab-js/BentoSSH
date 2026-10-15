import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';

export interface Session {
  id: string;
  name: string;
  host: any;
  status: string;
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
      try {
        const sessionId = await invoke('connect_saved_host', { id: host.id }) as string;

        const newSession = {
          id: sessionId,
          name: host.name,
          host: host,
          status: 'connected'
        };
        
        sessions = [...sessions, newSession];
        activeSessionId = sessionId;
        
        startHealthPolling(sessionId);
        toast.success(`Connected to ${host.name}`);
        return sessionId;
      } catch (e) {
        toast.error('Connection failed: ' + e);
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
