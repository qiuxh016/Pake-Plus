import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '../..',
);
const panel = fs.readFileSync(
  path.join(root, 'src-tauri/assets/clipboard-panel.html'),
  'utf8',
);
const panelSource = fs.readFileSync(
  path.join(root, 'src-tauri/src/app/clipboard/panel.rs'),
  'utf8',
);
const runtime = fs.readFileSync(
  path.join(root, 'src-tauri/src/lib.rs'),
  'utf8',
);

describe('clipboard history panel', () => {
  it('offers an explicit copy action while preserving row click reuse', () => {
    expect(panel).toContain('copy.textContent = text.copy');
    expect(panel).toContain('call("clipboard_copy_item", { id })');
    expect(panel).toContain('.item:hover .actions');
  });

  it('exposes configurable privacy filters to the trusted local panel', () => {
    expect(panel).toContain('call("clipboard_get_settings")');
    expect(panel).toContain('call("clipboard_update_settings"');
    expect(panel).toContain('ignore_password_like');
    expect(panel).toContain('ignore_credit_card_like');
    expect(panel).toContain('ignored_apps');
  });

  it('shows a detected source application with each history item', () => {
    expect(panel).toContain('item.source_app');
  });

  it('accepts a query from the settings sidebar', () => {
    expect(panel).toContain('async function openWithQuery(query)');
    expect(panel).toContain(
      'window.pakeClipboardPanel = { refresh, openWithQuery }',
    );
  });

  it('stays open after reuse and closes through the close button or Escape', () => {
    expect(panel).toContain('id="close-panel"');
    expect(panel).toContain('call("clipboard_hide_panel")');
    expect(panel).toContain('if (event.key !== "Escape") return');
    expect(panelSource).not.toContain('hide_panel(app);');
    expect(runtime).not.toContain('tauri::WindowEvent::Focused(false)');
  });

  it('uses a fully visible in-panel clear confirmation', () => {
    expect(panel).toContain('id="confirm-overlay"');
    expect(panel).toContain('id="confirm-cancel"');
    expect(panel).toContain('id="confirm-clear"');
    expect(panel).not.toContain('window.confirm');
  });

  it('shows copy feedback at the pointer for three seconds', () => {
    expect(panel).toContain('function showCopyToast(x, y)');
    expect(panel).toContain('已复制到剪贴板');
    expect(panel).toContain('}, 3000);');
  });

  it('allows users to resize the history panel from its edges', () => {
    expect(panelSource).toContain('.resizable(true)');
    expect(panelSource).toContain('.min_inner_size(320.0, 360.0)');
  });
});
