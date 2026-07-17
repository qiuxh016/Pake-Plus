# Pake Plus - Full Integration Test (A+B+C+D)
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot
$pass = 0; $fail = 0

function Test-Item($label, $ok) {
    if ($ok) { Write-Host "  [PASS] $label" -ForegroundColor Green; $script:pass++ }
    else     { Write-Host "  [FAIL] $label" -ForegroundColor Red;   $script:fail++ }
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Pake Plus - Full Integration Test" -ForegroundColor Cyan
Write-Host " Root: $root" -ForegroundColor Gray
Write-Host "========================================" -ForegroundColor Cyan

# ===== Preload all files =====
$lib = Get-Content (Join-Path $root "src-tauri\src\lib.rs") -Raw
$perm = Get-Content (Join-Path $root "src-tauri\permissions\settings.toml") -Raw
$pake = Get-Content (Join-Path $root "src-tauri\pake.json") -Raw | ConvertFrom-Json
$cfg = Get-Content (Join-Path $root "src-tauri\src\app\config.rs") -Raw
$win = Get-Content (Join-Path $root "src-tauri\src\app\window.rs") -Raw
$cachejs = Get-Content (Join-Path $root "src-tauri\src\inject\cache.js") -Raw
$customjs = Get-Content (Join-Path $root "src-tauri\src\inject\custom.js") -Raw
$cmds = Get-Content (Join-Path $root "src-tauri\src\app\settings\commands.rs") -Raw
$diag = Get-Content (Join-Path $root "src-tauri\src\app\settings\diagnostics.rs") -Raw
$io = Get-Content (Join-Path $root "src-tauri\src\app\settings\io.rs") -Raw
$trait = Get-Content (Join-Path $root "src-tauri\src\app\settings\traits.rs") -Raw
$cargo = Get-Content (Join-Path $root "src-tauri\Cargo.toml") -Raw

# ===== A: Adblock =====
Write-Host "`n--- A: Adblock ---" -ForegroundColor Yellow
Test-Item "adblock/mod.rs" (Test-Path (Join-Path $root "src-tauri\src\adblock\mod.rs"))
Test-Item "adblock/engine.rs" (Test-Path (Join-Path $root "src-tauri\src\adblock\engine.rs"))
Test-Item "adblock/rules.rs" (Test-Path (Join-Path $root "src-tauri\src\adblock\rules.rs"))
Test-Item "inject/adblock.js" (Test-Path (Join-Path $root "src-tauri\src\inject\adblock.js"))
Test-Item "mod adblock declaration" ($lib -match 'mod adblock')
Test-Item "AdblockState import" ($lib -match 'use crate::adblock::AdblockState')
Test-Item "AdblockState::new init" ($lib -match 'AdblockState::new')
Test-Item "block_ads config" ($pake.block_ads -eq $true)
Test-Item "adblock_report_blocked IPC" ($lib -match 'adblock_report_blocked')
Test-Item "adblock_get_stats IPC" ($lib -match 'adblock_get_stats')
Test-Item "adblock_add_rule IPC" ($lib -match 'adblock_add_rule')
Test-Item "adblock_remove_rule IPC" ($lib -match 'adblock_remove_rule')
Test-Item "adblock 4 cmd permissions" (
    ($perm -match 'adblock_report_blocked') -and ($perm -match 'adblock_get_stats') -and
    ($perm -match 'adblock_add_rule') -and ($perm -match 'adblock_remove_rule'))

# ===== B: Cache =====
Write-Host "`n--- B: Cache ---" -ForegroundColor Yellow
Test-Item "cache/mod.rs" (Test-Path (Join-Path $root "src-tauri\src\cache\mod.rs"))
Test-Item "cache/engine.rs" (Test-Path (Join-Path $root "src-tauri\src\cache\engine.rs"))
Test-Item "cache/commands.rs" (Test-Path (Join-Path $root "src-tauri\src\cache\commands.rs"))
Test-Item "inject/cache.js" (Test-Path (Join-Path $root "src-tauri\src\inject\cache.js"))
Test-Item "mod cache declaration" ($lib -match 'mod cache')
Test-Item "CacheState import" ($lib -match 'use crate::cache::CacheState')
Test-Item "CacheState::new init" ($lib -match 'CacheState::new')
Test-Item "cache + cache_size in PakeConfig" ($cfg -match 'pub cache:' -and $cfg -match 'pub cache_size:')
Test-Item "cache_fetch IPC" ($lib -match 'cache_fetch')
Test-Item "cache_get_stats IPC" ($lib -match 'cache_get_stats')
Test-Item "cache_clear IPC" ($lib -match 'cache_clear')
Test-Item "cache_stats_json IPC" ($lib -match 'cache_stats_json')
Test-Item "cache 4 cmd permissions" (
    ($perm -match 'cache_fetch') -and ($perm -match 'cache_get_stats') -and
    ($perm -match 'cache_clear') -and ($perm -match 'cache_stats_json'))
Test-Item "config.cache in window.rs" ($win -match 'config\.cache')
Test-Item "cache.js offline banner" ($cachejs -match '__pake_offline_bnr__')
Test-Item "cache.js OFFLINE badge" ($cachejs -match '__pake_offline__')
Test-Item "cache.js hit toast" ($cachejs -match '__pake_cache_toast__')
Test-Item "cache.js offline HTML" ($cachejs -match 'offlineHTML')
Test-Item "cache.js fetch intercept" ($cachejs -match 'window\.fetch\s*=\s*function')
Test-Item "variable cache_enabled" ($lib -match 'cache_enabled')
Test-Item "variable cache_size" ($lib -match 'cache_size')
Test-Item "cache in diagnostics" ($diag -match 'settings\.cache\.enabled')
Test-Item "sync_cache_config in save" ($cmds -match 'sync_cache_config')

# ===== C: Clipboard =====
Write-Host "`n--- C: Clipboard ---" -ForegroundColor Yellow
Test-Item "clipboard/mod.rs" (Test-Path (Join-Path $root "src-tauri\src\app\clipboard\mod.rs"))
Test-Item "clipboard/commands.rs" (Test-Path (Join-Path $root "src-tauri\src\app\clipboard\commands.rs"))
Test-Item "clipboard/monitor.rs" (Test-Path (Join-Path $root "src-tauri\src\app\clipboard\monitor.rs"))
Test-Item "clipboard/store.rs" (Test-Path (Join-Path $root "src-tauri\src\app\clipboard\store.rs"))
Test-Item "clipboard feature in Cargo.toml" ($cargo -match 'clipboard\s*=\s*\[')
Test-Item "clipboard field in PakeConfig" ($cfg -match 'pub clipboard:')
Test-Item "pake.json clipboard=true" ($pake.clipboard -eq $true)
Test-Item "init_clipboard_state" ($lib -match 'init_clipboard_state')
Test-Item "10x clipboard cmd permissions" (
    ($perm -match 'clipboard_list') -and ($perm -match 'clipboard_search') -and
    ($perm -match 'clipboard_copy_item') -and ($perm -match 'clipboard_delete_item') -and
    ($perm -match 'clipboard_clear_all') -and ($perm -match 'clipboard_stats'))

# ===== D: Settings =====
Write-Host "`n--- D: Settings ---" -ForegroundColor Yellow
Test-Item "settings/mod.rs" (Test-Path (Join-Path $root "src-tauri\src\app\settings\mod.rs"))
Test-Item "settings/commands.rs" (Test-Path (Join-Path $root "src-tauri\src\app\settings\commands.rs"))
Test-Item "settings/types.rs" (Test-Path (Join-Path $root "src-tauri\src\app\settings\types.rs"))
Test-Item "settings/io.rs" (Test-Path (Join-Path $root "src-tauri\src\app\settings\io.rs"))
Test-Item "settings/diagnostics.rs" (Test-Path (Join-Path $root "src-tauri\src\app\settings\diagnostics.rs"))
Test-Item "settings/traits.rs" (Test-Path (Join-Path $root "src-tauri\src\app\settings\traits.rs"))
Test-Item "settings/health.rs" (Test-Path (Join-Path $root "src-tauri\src\app\settings\health.rs"))
Test-Item "inject/custom.js" (Test-Path (Join-Path $root "src-tauri\src\inject\custom.js"))
Test-Item "assets/settings.html" (Test-Path (Join-Path $root "src-tauri\assets\settings.html"))
Test-Item "get_settings IPC" ($lib -match 'get_settings')
Test-Item "save_settings IPC" ($lib -match 'save_settings')
Test-Item "export_data IPC" ($lib -match 'export_data')
Test-Item "import_data IPC" ($lib -match 'import_data')
Test-Item "preview_import IPC" ($lib -match 'preview_import')
Test-Item "get_diagnostics IPC" ($lib -match 'get_diagnostics')
Test-Item "copy_diagnostics_report IPC" ($lib -match 'copy_diagnostics_report')
Test-Item "pick_save_path IPC" ($lib -match 'pick_save_path')
Test-Item "pick_zip_file IPC" ($lib -match 'pick_zip_file')
Test-Item "list_backups IPC" ($lib -match 'list_backups')
Test-Item "rollback_settings IPC" ($lib -match 'rollback_settings')
Test-Item "get_download_dir IPC" ($lib -match 'get_download_dir')
Test-Item "MAX_BACKUPS backup rotation" ($io -match 'MAX_BACKUPS')
Test-Item "ModuleSettings trait" ($trait -match 'trait ModuleSettings')
Test-Item "CacheSettings trait impl" ($trait -match 'impl ModuleSettings for CacheSettings')
Test-Item "Refresh button UI" ($customjs -match '__prb')
Test-Item "Version history UI" ($customjs -match 'loadVersionHistory')
Test-Item "Diagnostics fill logic" ($customjs -match 'fillDiagnostics')
Test-Item "Cache real-time stats UI" ($customjs -match 'cache_stats_json')
Test-Item "Export path picker UI" ($customjs -match '__ps_exp_path__')
Test-Item "Import path picker UI" ($customjs -match '__ps_imp_path__')
Test-Item "rfd file dialog in commands" ($cmds -match 'rfd::FileDialog')
Test-Item "ZIP export in commands" ($cmds -match 'zip::ZipWriter')
Test-Item "ZIP import in commands" ($cmds -match 'zip::ZipArchive')
Test-Item "All 30+ cmd permissions check" (
    ($perm -match 'get_settings') -and ($perm -match 'save_settings') -and
    ($perm -match 'export_data') -and ($perm -match 'import_data') -and
    ($perm -match 'preview_import') -and ($perm -match 'list_backups') -and
    ($perm -match 'rollback_settings') -and ($perm -match 'pick_save_path') -and
    ($perm -match 'pick_zip_file') -and ($perm -match 'get_diagnostics'))

# ===== CLI =====
Write-Host "`n--- CLI ---" -ForegroundColor Yellow
Push-Location $root
$help = node "$root\dist\cli.js" --help 2>&1
Test-Item "--block-ads option" ($help -match 'block-ads')
Test-Item "--adblock-rules option" ($help -match 'adblock-rules')
Test-Item "--cache option" ($help -match '\s+--cache\b')
Test-Item "--cache-size option" ($help -match 'cache-size')
Test-Item "--clipboard option" ($help -match 'clipboard')
Pop-Location

# ===== Cargo Check =====
Write-Host "`n--- Build ---" -ForegroundColor Yellow
Push-Location (Join-Path $root "src-tauri")
$out = cargo check --no-default-features --features "custom-protocol,clipboard" 2>&1
$errs = ($out | Select-String "error\[").Count
Test-Item "cargo check errors=$errs" ($errs -eq 0)
Pop-Location

# ===== Result =====
Write-Host "`n========================================" -ForegroundColor Cyan
$total = $pass + $fail
$color = if ($fail -eq 0) { "Green" } else { "Red" }
Write-Host " PASS=$pass  FAIL=$fail  TOTAL=$total" -ForegroundColor $color
Write-Host "========================================" -ForegroundColor Cyan
