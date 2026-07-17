import { describe, expect, it } from 'vitest';
import { buildWindowConfigOverrides } from '../../bin/helpers/merge';
import { DEFAULT_PAKE_OPTIONS } from '../../bin/defaults';
import type { PakeAppOptions } from '../../bin/types';

function makeOptions(overrides: Partial<PakeAppOptions> = {}): PakeAppOptions {
  return {
    ...DEFAULT_PAKE_OPTIONS,
    identifier: 'com.pake.test',
    name: 'TestApp',
    ...overrides,
  };
}

describe('adblock CLI options', () => {
  it('exposes blockAds and adblockRules defaults', () => {
    expect(DEFAULT_PAKE_OPTIONS.blockAds).toBe(false);
    expect(DEFAULT_PAKE_OPTIONS.adblockRules).toBe('');
  });

  it('does not affect window config overrides', () => {
    const overrides = buildWindowConfigOverrides(
      makeOptions({ blockAds: true, adblockRules: './rules.txt' }),
    );
    expect(overrides.width).toBe(1200);
    expect(overrides.height).toBe(780);
  });
});
