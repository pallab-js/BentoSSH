import { describe, it, expect } from 'vitest';
import { DEFAULT_SETTINGS, mergeSettings, type Settings } from './settings.svelte';

describe('DEFAULT_SETTINGS', () => {
  it('enables the terminal bell and right-click paste by default', () => {
    expect(DEFAULT_SETTINGS.terminal.bell).toBe(true);
    expect(DEFAULT_SETTINGS.keyboard.pasteOnRightClick).toBe(true);
  });

  it('has sensible terminal defaults', () => {
    expect(DEFAULT_SETTINGS.terminal.fontSize).toBeGreaterThan(0);
    expect(DEFAULT_SETTINGS.terminal.scrollback).toBeGreaterThan(0);
  });
});

describe('mergeSettings', () => {
  it('returns a deep copy of defaults when saved is empty/invalid', () => {
    const a = mergeSettings(DEFAULT_SETTINGS, null);
    const b = mergeSettings(DEFAULT_SETTINGS, 'not an object');
    expect(a).toEqual(DEFAULT_SETTINGS);
    expect(b).toEqual(DEFAULT_SETTINGS);
    // Must be a copy, not the same reference (no mutation of defaults).
    expect(a).not.toBe(DEFAULT_SETTINGS);
    expect(a.terminal).not.toBe(DEFAULT_SETTINGS.terminal);
  });

  it('overrides only the provided keys and keeps other defaults', () => {
    const saved = { terminal: { fontSize: 20 } };
    const result = mergeSettings(DEFAULT_SETTINGS, saved);
    expect(result.terminal.fontSize).toBe(20);
    // Untouched keys fall back to defaults.
    expect(result.terminal.bell).toBe(DEFAULT_SETTINGS.terminal.bell);
    expect(result.keyboard.pasteOnRightClick).toBe(DEFAULT_SETTINGS.keyboard.pasteOnRightClick);
  });

  it('backfills new keys for older saved payloads (deep per-section merge)', () => {
    // Simulate a saved blob from before `bell` / `pasteOnRightClick` existed.
    const legacy = {
      terminal: { fontFamily: 'Fira Code', fontSize: 12, cursorStyle: 'bar', cursorBlink: false, scrollback: 500, theme: 'nord' },
      keyboard: { commandPalette: 'p', newSession: 'n', closeSession: 'q' }
    };
    const result = mergeSettings(DEFAULT_SETTINGS, legacy);
    expect(result.terminal.theme).toBe('nord');
    expect(result.keyboard.commandPalette).toBe('p');
    // Newly added keys get their defaults rather than becoming undefined.
    expect(result.terminal.bell).toBe(true);
    expect(result.keyboard.pasteOnRightClick).toBe(true);
  });

  it('does not mutate the defaults object', () => {
    const before = JSON.stringify(DEFAULT_SETTINGS);
    mergeSettings(DEFAULT_SETTINGS, { terminal: { fontSize: 99 } });
    expect(JSON.stringify(DEFAULT_SETTINGS)).toBe(before);
  });

  it('ignores non-object sections in saved payload', () => {
    const result = mergeSettings(DEFAULT_SETTINGS, { terminal: 'bogus', keyboard: 42 } as unknown as Partial<Settings>);
    expect(result.terminal).toEqual(DEFAULT_SETTINGS.terminal);
    expect(result.keyboard).toEqual(DEFAULT_SETTINGS.keyboard);
  });
});
