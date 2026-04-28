export const uiState = $state({
  isPaletteOpen: false,
  showAddHost: false,
  showSettings: false,
  showHealth: false
});

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
