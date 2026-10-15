import { browser } from '$app/environment';

const SIDEBAR_KEY = 'bentossh_sidebar_collapsed';

function loadSidebarCollapsed(): boolean {
  if (!browser) return false;
  return localStorage.getItem(SIDEBAR_KEY) === 'true';
}

export const uiState = $state({
  isPaletteOpen: false,
  showAddHost: false,
  showSettings: false,
  showHealth: false,
  sidebarCollapsed: loadSidebarCollapsed()
});

export function toggleSidebar() {
  uiState.sidebarCollapsed = !uiState.sidebarCollapsed;
  if (browser) {
    localStorage.setItem(SIDEBAR_KEY, String(uiState.sidebarCollapsed));
  }
}

export function openPalette() {
  uiState.isPaletteOpen = true;
}

export function closePalette() {
  uiState.isPaletteOpen = false;
}

export function openAddHost() {
  uiState.showAddHost = true;
}

export function closeAddHost() {
  uiState.showAddHost = false;
}

export function openSettings() {
  uiState.showSettings = true;
}

export function closeSettings() {
  uiState.showSettings = false;
}
