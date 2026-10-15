import { browser } from '$app/environment';

export interface Settings {
  terminal: {
    fontFamily: string;
    fontSize: number;
    cursorStyle: 'block' | 'underline' | 'bar';
    cursorBlink: boolean;
    scrollback: number;
    theme: string;
    bell: boolean;
  };
  security: {
    defaultKeyPath: string;
    savePasswords: boolean;
  };
  keyboard: {
    commandPalette: string;
    newSession: string;
    closeSession: string;
    pasteOnRightClick: boolean;
  };
  notifications: {
    connectionAlerts: boolean;
  };
}

export const DEFAULT_SETTINGS: Settings = {
  terminal: {
    fontFamily: 'JetBrains Mono',
    fontSize: 14,
    cursorStyle: 'block',
    cursorBlink: true,
    scrollback: 1000,
    theme: 'dracula',
    bell: true
  },
  security: {
    defaultKeyPath: '~/.ssh/id_rsa',
    savePasswords: true
  },
  keyboard: {
    commandPalette: 'k',
    newSession: 't',
    closeSession: 'w',
    pasteOnRightClick: true
  },
  notifications: {
    connectionAlerts: true
  }
};

/**
 * Merge persisted settings over the defaults, section by section, so newly
 * added keys always fall back to their default even when an older payload is
 * loaded from localStorage. Exported for unit testing.
 */
export function mergeSettings(defaults: Settings, saved: unknown): Settings {
  if (!saved || typeof saved !== 'object') return structuredCloneSafe(defaults);
  const merged = structuredCloneSafe(defaults);
  for (const section of Object.keys(defaults) as (keyof Settings)[]) {
    const savedSection = (saved as Record<string, unknown>)[section];
    if (savedSection && typeof savedSection === 'object') {
      Object.assign(merged[section] as object, savedSection);
    }
  }
  return merged;
}

function structuredCloneSafe<T>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}

function createSettings() {
  let initialSettings = structuredCloneSafe(DEFAULT_SETTINGS);

  if (browser) {
    const saved = localStorage.getItem('bentossh_settings');
    if (saved) {
      try {
        initialSettings = mergeSettings(DEFAULT_SETTINGS, JSON.parse(saved));
      } catch (e) {
        console.error('Failed to parse settings', e);
      }
    }
  }

  const settings = $state<Settings>(initialSettings);

  return {
    get current() {
      return settings;
    },
    update(newSettings: Partial<Settings>) {
      Object.assign(settings, newSettings);
      if (browser) {
        localStorage.setItem('bentossh_settings', JSON.stringify(settings));
      }
    },
    reset() {
      Object.assign(settings, structuredCloneSafe(DEFAULT_SETTINGS));
      if (browser) {
        localStorage.removeItem('bentossh_settings');
      }
    }
  };
}

export const settingsState = createSettings();
