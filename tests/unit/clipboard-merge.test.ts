import { describe, expect, it } from 'vitest';
import { DEFAULT_PAKE_OPTIONS } from '../../bin/defaults.js';
import { applyClipboardConfig } from '../../bin/helpers/merge.js';

describe('clipboard CLI configuration', () => {
  it('defaults clipboard off with 2000 max items', () => {
    expect(DEFAULT_PAKE_OPTIONS.clipboard).toBe(false);
    expect(DEFAULT_PAKE_OPTIONS.clipboardMax).toBe(2000);
  });

  it('propagates clipboard options to pake config', () => {
    const config = { clipboard: false, clipboard_max: 2000 };
    applyClipboardConfig({ clipboard: true, clipboardMax: 4500 }, config);
    expect(config).toEqual({ clipboard: true, clipboard_max: 4500 });
  });

  it('keeps the configured maximum while clipboard is disabled', () => {
    const config = { clipboard: true, clipboard_max: 2000 };
    applyClipboardConfig({ clipboard: false, clipboardMax: 500 }, config);
    expect(config).toEqual({ clipboard: false, clipboard_max: 500 });
  });
});
