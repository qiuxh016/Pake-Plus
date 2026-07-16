import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '../..',
);

describe('clipboard capability isolation', () => {
  it('does not grant clipboard history commands to the remote pake webview', () => {
    const capability = JSON.parse(
      fs.readFileSync(
        path.join(root, 'src-tauri/capabilities/default.json'),
        'utf8',
      ),
    );
    expect(capability.webviews).toEqual(['pake']);
    expect(
      capability.permissions.some((permission: string) =>
        permission.includes('clipboard'),
      ),
    ).toBe(false);
  });

  it('needs no permission or separate capability for the static embedded sidebar', () => {
    const permissions = fs.readFileSync(
      path.join(root, 'src-tauri/permissions/clipboard.toml'),
      'utf8',
    );
    expect(
      fs.existsSync(
        path.join(root, 'src-tauri/capabilities/settings-panel.json'),
      ),
    ).toBe(false);
    expect(permissions).not.toContain('settings_panel');
  });

  it('grants history and privacy settings only to the local clipboard panel', () => {
    const capability = JSON.parse(
      fs.readFileSync(
        path.join(root, 'src-tauri/capabilities/clipboard-panel.json'),
        'utf8',
      ),
    );
    const permissions = fs.readFileSync(
      path.join(root, 'src-tauri/permissions/clipboard.toml'),
      'utf8',
    );
    expect(capability.webviews).toEqual(['clipboard-panel']);
    expect(capability.remote).toBeUndefined();
    expect(capability.permissions).toEqual([
      'allow-clipboard-panel',
      'allow-clipboard-settings',
    ]);
    expect(permissions).toContain('clipboard_list');
    expect(permissions).toContain('clipboard_hide_panel');
    expect(permissions).toContain('clipboard_get_settings');
    expect(permissions).toContain('clipboard_update_settings');
    expect(permissions).not.toContain('clipboard_toggle_panel');
  });
});
