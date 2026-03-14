import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { readFileSync } from 'fs';
import { join } from 'path';

// Mock the shell plugin before importing the module
vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn(),
}));

// Import after mocking
import { open } from '@tauri-apps/plugin-shell';
import { openUrl } from '../src/lib/tauri';

describe('Tauri capabilities', () => {
  it('should include shell:allow-open permission', () => {
    const caps = JSON.parse(
      readFileSync(join(__dirname, '../src-tauri/capabilities/default.json'), 'utf-8')
    );
    expect(caps.permissions).toContain('shell:allow-open');
  });
});

describe('openUrl', () => {
  const mockOpen = vi.mocked(open);
  let mockWindowOpen: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    mockWindowOpen = vi.fn();
    global.window = {
      open: mockWindowOpen,
    } as unknown as Window & typeof globalThis;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('when shell plugin open() succeeds', () => {
    it('should open URL with shell plugin and not trigger fallbacks', async () => {
      mockOpen.mockResolvedValueOnce(undefined);

      await openUrl('https://example.com');

      expect(mockOpen).toHaveBeenCalledTimes(1);
      expect(mockOpen).toHaveBeenCalledWith('https://example.com');
      expect(mockWindowOpen).not.toHaveBeenCalled();
    });
  });

  describe('when shell plugin fails', () => {
    it('should fall back to window.Tauri.invoke when available', async () => {
      const mockTauriInvoke = vi.fn().mockResolvedValueOnce(undefined);
      (global.window as Window & { Tauri?: { invoke: typeof mockTauriInvoke } }).Tauri = {
        invoke: mockTauriInvoke,
      };

      mockOpen.mockRejectedValueOnce(new Error('Shell plugin failed'));

      await openUrl('https://example.com');

      expect(mockOpen).toHaveBeenCalledTimes(1);
      expect(mockTauriInvoke).toHaveBeenCalledTimes(1);
      expect(mockTauriInvoke).toHaveBeenCalledWith('open_url', { url: 'https://example.com' });
      expect(mockWindowOpen).not.toHaveBeenCalled();
    });

    it('should fall back to window.open when both shell and Tauri.invoke fail for http URLs', async () => {
      const mockTauriInvoke = vi.fn().mockRejectedValueOnce(new Error('Backend command failed'));
      (global.window as Window & { Tauri?: { invoke: typeof mockTauriInvoke } }).Tauri = {
        invoke: mockTauriInvoke,
      };

      mockOpen.mockRejectedValueOnce(new Error('Shell plugin failed'));

      await openUrl('https://example.com');

      expect(mockOpen).toHaveBeenCalledTimes(1);
      expect(mockTauriInvoke).toHaveBeenCalledTimes(1);
      expect(mockWindowOpen).toHaveBeenCalledTimes(1);
      expect(mockWindowOpen).toHaveBeenCalledWith('https://example.com', '_blank');
    });

    it('should NOT fall back to window.open for non-http URLs when all methods fail', async () => {
      const mockTauriInvoke = vi.fn().mockRejectedValueOnce(new Error('Backend command failed'));
      (global.window as Window & { Tauri?: { invoke: typeof mockTauriInvoke } }).Tauri = {
        invoke: mockTauriInvoke,
      };

      mockOpen.mockRejectedValueOnce(new Error('Shell plugin failed'));

      await openUrl('ftp://example.com/file.txt');

      expect(mockOpen).toHaveBeenCalledTimes(1);
      expect(mockTauriInvoke).toHaveBeenCalledTimes(1);
      expect(mockWindowOpen).not.toHaveBeenCalled();
    });
  });

  describe('when window.Tauri is not available', () => {
    it('should fall back to window.open for http URLs', async () => {
      // Ensure window.Tauri is undefined
      (global.window as Window & { Tauri?: unknown }).Tauri = undefined;

      mockOpen.mockRejectedValueOnce(new Error('Shell plugin failed'));

      await openUrl('https://example.com');

      expect(mockOpen).toHaveBeenCalledTimes(1);
      expect(mockWindowOpen).toHaveBeenCalledTimes(1);
      expect(mockWindowOpen).toHaveBeenCalledWith('https://example.com', '_blank');
    });

    it('should NOT fall back to window.open for non-http URLs', async () => {
      // Ensure window.Tauri is undefined
      (global.window as Window & { Tauri?: unknown }).Tauri = undefined;

      mockOpen.mockRejectedValueOnce(new Error('Shell plugin failed'));

      await openUrl('file:///path/to/file.txt');

      expect(mockOpen).toHaveBeenCalledTimes(1);
      expect(mockWindowOpen).not.toHaveBeenCalled();
    });
  });
});