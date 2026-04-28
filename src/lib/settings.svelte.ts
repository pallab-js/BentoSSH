import { browser } from '$app/environment';

export interface Settings {
  terminal: {
    fontFamily: string;
    fontSize: number;
    cursorStyle: 'block' | 'underline' | 'bar';
    cursorBlink: boolean;
    scrollback: number;
    theme: string;
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
    terminalBell: boolean;
    commandFinished: boolean;
  };
}

const DEFAULT_SETTINGS: Settings = {
  terminal: {
    fontFamily: 'JetBrains Mono',
    fontSize: 14,
    cursorStyle: 'block',
    cursorBlink: true,
    scrollback: 1000,
    theme: 'dracula'
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
    connectionAlerts: true,
    terminalBell: true,
    commandFinished: false
  }
};

function createSettings() {
  let initialSettings = DEFAULT_SETTINGS;
  
  if (browser) {
    const saved = localStorage.getItem('bentossh_settings');
    if (saved) {
      try {
        initialSettings = { ...DEFAULT_SETTINGS, ...JSON.parse(saved) };
      } catch (e) {
        console.error('Failed to parse settings', e);
      }
    }
  }

  const settings = $state<Settings>(initialSettings);

  return {
    get current() { return settings; },
    update(newSettings: Partial<Settings>) {
      Object.assign(settings, newSettings);
      if (browser) {
        localStorage.setItem('bentossh_settings', JSON.stringify(settings));
      }
    },
    reset() {
      Object.assign(settings, DEFAULT_SETTINGS);
      if (browser) {
        localStorage.removeItem('bentossh_settings');
      }
    }
  };
}

export const settingsState = createSettings();
