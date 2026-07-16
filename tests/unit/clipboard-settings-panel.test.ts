import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '../..',
);
const panel = fs.readFileSync(
  path.join(root, 'src-tauri/src/inject/settings.js'),
  'utf8',
);
const windowSource = fs.readFileSync(
  path.join(root, 'src-tauri/src/app/window.rs'),
  'utf8',
);
const runtime = fs.readFileSync(
  path.join(root, 'src-tauri/src/lib.rs'),
  'utf8',
);

describe('feature C settings sidebar', () => {
  it('shows all nine clipboard features', () => {
    for (let index = 1; index <= 9; index += 1) {
      expect(panel).toContain(`data-feature="C${index}"`);
    }
  });

  it('opens feature rules without executing clipboard actions', () => {
    expect(panel).toContain('功能规则与测试方法');
    expect(panel).not.toContain('clipboard_clear_all');
    expect(panel).not.toContain('clipboard_update_settings');
  });

  it('embeds the sidebar inside the Weekly webview', () => {
    expect(windowSource).toContain('inject/settings.js');
    expect(panel).toContain('attachShadow({ mode: "closed" })');
    expect(panel).toContain('document.documentElement.append(host)');
    expect(runtime).not.toContain('show_settings_panel(');
    expect(
      fs.existsSync(
        path.join(root, 'src-tauri/src/app/clipboard/settings_panel.rs'),
      ),
    ).toBe(false);
  });

  it('documents how to test the two CLI options', () => {
    expect(panel).toContain('--name Weekly --clipboard');
    expect(panel).toContain('--clipboard --clipboard-max 500');
    expect(panel).toContain('写入第 501 条会固定删除最早 500 条');
  });

  it('collapses to a narrow embedded sidebar from the triangle below P', () => {
    expect(panel).toContain('class="collapse-toggle"');
    expect(panel).toContain('▶');
    expect(panel).toContain('host.style.width = collapsed ? "72px"');
  });

  it('does not retain the former always-on-top window or label', () => {
    expect(runtime).not.toContain('SETTINGS_PANEL_LABEL');
    expect(runtime).not.toContain('pake-settings');
    expect(panel).not.toContain('常驻边栏');
  });
});
