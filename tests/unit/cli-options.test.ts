import { describe, expect, it } from 'vitest';
import { getCliProgram } from '../../bin/helpers/cli-program.js';
import { validateNumberInput } from '../../bin/utils/validate.js';

describe('CLI options', () => {
  const program = getCliProgram();

  it('shows meta options in help', () => {
    const help = program.helpInformation();

    expect(help).toContain('-h, --help');
    expect(help).toContain('-v, --version');
  });

  it('shows advanced options in help', () => {
    const help = program.helpInformation();

    expect(help).toContain('--enable-find');
    expect(help).toContain('--internal-url-regex');
    expect(help).toContain('--hide-on-close');
  });

  it('registers hidden --multi-window option', () => {
    const option = program.options.find(
      (item) => item.long === '--multi-window',
    );

    expect(option).toBeDefined();
    expect(option?.defaultValue).toBe(false);
  });

  it('exposes --internal-url-regex option', () => {
    const option = program.options.find(
      (item) => item.long === '--internal-url-regex',
    );

    expect(option).toBeDefined();
    expect(option?.defaultValue).toBe('');
    expect(option?.hidden).toBeFalsy();
  });

  it('exposes --safe-domain option', () => {
    const option = program.options.find(
      (item) => item.long === '--safe-domain',
    );

    expect(option).toBeDefined();
    expect(option?.defaultValue).toBe('');
    expect(option?.hidden).toBeFalsy();
  });

  it('exposes --force-internal-navigation option', () => {
    const option = program.options.find(
      (item) => item.long === '--force-internal-navigation',
    );

    expect(option).toBeDefined();
    expect(option?.defaultValue).toBe(false);
    expect(option?.hidden).toBeFalsy();
  });

  it('exposes --new-window option', () => {
    const option = program.options.find((item) => item.long === '--new-window');

    expect(option).toBeDefined();
    expect(option?.defaultValue).toBe(false);
    expect(option?.hidden).toBeFalsy();
  });

  it('registers hidden --identifier option', () => {
    const option = program.options.find((item) => item.long === '--identifier');

    expect(option).toBeDefined();
    expect(option?.hidden).toBe(true);
  });

  it('registers hidden --install option', () => {
    const option = program.options.find((item) => item.long === '--install');

    expect(option).toBeDefined();
    expect(option?.defaultValue).toBe(false);
    expect(option?.hidden).toBe(true);
  });

  it('registers hidden --enable-find option', () => {
    const option = program.options.find(
      (item) => item.long === '--enable-find',
    );

    expect(option).toBeDefined();
    expect(option?.defaultValue).toBe(false);
    expect(option?.hidden).toBe(true);
  });

  it('registers --clipboard option', () => {
    const option = program.options.find((item) => item.long === '--clipboard');

    expect(option).toBeDefined();
    expect(option?.defaultValue).toBe(false);
    expect(option?.hidden).toBe(false);
  });

  it('validates --clipboard-max range', () => {
    const option = program.options.find(
      (item) => item.long === '--clipboard-max',
    );

    expect(option).toBeDefined();
    expect(option?.defaultValue).toBe(2000);
    expect(option?.parseArg?.('5000', undefined)).toBe(5000);
    expect(() => option?.parseArg?.('499', undefined)).toThrow(
      '--clipboard-max must be an integer between 500 and 5000',
    );
    expect(() => option?.parseArg?.('6000', undefined)).toThrow(
      '--clipboard-max must be an integer between 500 and 5000',
    );
  });

  it('rejects malformed zoom values instead of truncating them', () => {
    const option = program.options.find((item) => item.long === '--zoom');

    expect(option).toBeDefined();
    expect(option?.parseArg?.('80', undefined)).toBe(80);
    expect(() => option?.parseArg?.('80abc', undefined)).toThrow(
      '--zoom must be an integer between 50 and 200',
    );
    // Fractional in-range values cannot deserialize into the Rust u32 zoom
    // field, so they must be rejected rather than forwarded to pake.json.
    expect(() => option?.parseArg?.('99.5', undefined)).toThrow(
      '--zoom must be an integer between 50 and 200',
    );
  });

  it('rejects non-finite numeric option values', () => {
    expect(() => validateNumberInput('Infinity')).toThrow('Not a number.');
    expect(() => validateNumberInput('-Infinity')).toThrow('Not a number.');
    expect(validateNumberInput('1200')).toBe(1200);
  });

  it('rejects blank numeric option values', () => {
    expect(() => validateNumberInput('')).toThrow('Not a number.');
    expect(() => validateNumberInput('   ')).toThrow('Not a number.');
  });

  it('rejects negative numeric option values', () => {
    expect(() => validateNumberInput('-100')).toThrow('Must not be negative.');
    expect(validateNumberInput('0')).toBe(0);
  });

  it('parses clipboard packaging flags together', () => {
    program.parse(
      ['https://example.com', '--clipboard', '--clipboard-max', '5000'],
      { from: 'user' },
    );

    expect(program.opts().clipboard).toBe(true);
    expect(program.opts().clipboardMax).toBe(5000);
  });
});
