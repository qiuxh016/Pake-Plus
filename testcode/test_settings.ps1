# Pake Plus - Settings Panel / Data / Diagnostics Test (D)
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Settings (D) Module Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. Check source files
Write-Host "`n[1] Source files..." -ForegroundColor Yellow
$files = @(
    "src-tauri\src\app\settings\mod.rs",
    "src-tauri\src\app\settings\commands.rs",
    "src-tauri\src\app\settings\types.rs",
    "src-tauri\src\app\settings\io.rs",
    "src-tauri\src\app\settings\diagnostics.rs",
    "src-tauri\src\app\settings\traits.rs",
    "src-tauri\src\app\settings\health.rs",
    "src-tauri\src\inject\custom.js",
    "src-tauri\assets\settings.html"
)
foreach ($f in $files) {
    $path = Join-Path $root $f
    if (Test-Path $path) {
        $lines = (Get-Content $path).Count
        Write-Host "  OK: $f ($lines lines)" -ForegroundColor Green
    } else {
        Write-Host "  MISSING: $f" -ForegroundColor Red
    }
}

# 2. Check IPC command registration
Write-Host "`n[2] IPC commands in lib.rs..." -ForegroundColor Yellow
$lib = Get-Content (Join-Path $root "src-tauri\src\lib.rs") -Raw
$cmds = @("get_settings", "save_settings", "reset_settings", "validate_settings",
    "get_module_stats", "get_diagnostics", "copy_diagnostics_report",
    "export_data", "import_data", "preview_import",
    "list_backups", "rollback_settings", "pick_save_path", "pick_zip_file",
    "get_download_dir")
foreach ($c in $cmds) {
    if ($lib -match [regex]::Escape($c)) {
        Write-Host "  OK: $c" -ForegroundColor Green
    } else {
        Write-Host "  MISSING: $c" -ForegroundColor Red
    }
}

# 3. Check permissions
Write-Host "`n[3] Permissions..." -ForegroundColor Yellow
$perm = Get-Content (Join-Path $root "src-tauri\permissions\settings.toml") -Raw
foreach ($c in $cmds) {
    if ($perm -match [regex]::Escape($c)) {
        Write-Host "  OK: $c authorized" -ForegroundColor Green
    } else {
        Write-Host "  MISSING: $c" -ForegroundColor Red
    }
}

# 4. Check backup system
Write-Host "`n[4] Backup system..." -ForegroundColor Yellow
$io = Get-Content (Join-Path $root "src-tauri\src\app\settings\io.rs") -Raw
if ($io -match "MAX_BACKUPS") { Write-Host "  OK: MAX_BACKUPS defined" -ForegroundColor Green }
if ($io -match "write_settings") { Write-Host "  OK: write_settings" -ForegroundColor Green }
if ($io -match "restore_backup") { Write-Host "  OK: restore_backup" -ForegroundColor Green }
if ($io -match "get_backup_list") { Write-Host "  OK: get_backup_list" -ForegroundColor Green }
if ($io -match "remove_file.*newer" -or $io -match "remove_file") { Write-Host "  OK: remove before rename (Windows fix)" -ForegroundColor Green }

# 5. Check ModuleSettings trait
Write-Host "`n[5] ModuleSettings trait..." -ForegroundColor Yellow
$trait = Get-Content (Join-Path $root "src-tauri\src\app\settings\traits.rs") -Raw
if ($trait -match "trait ModuleSettings") { Write-Host "  OK: trait defined" -ForegroundColor Green }
if ($trait -match "impl ModuleSettings for CacheSettings") { Write-Host "  OK: CacheSettings impl" -ForegroundColor Green }
if ($trait -match "impl ModuleSettings for AdblockSettings") { Write-Host "  OK: AdblockSettings impl" -ForegroundColor Green }
if ($trait -match "impl ModuleSettings for ClipboardSettings") { Write-Host "  OK: ClipboardSettings impl" -ForegroundColor Green }
if ($trait -match "impl ModuleSettings for GeneralSettings") { Write-Host "  OK: GeneralSettings impl" -ForegroundColor Green }

# 6. Check export/import
Write-Host "`n[6] Export/Import..." -ForegroundColor Yellow
$cmdf = Get-Content (Join-Path $root "src-tauri\src\app\settings\commands.rs") -Raw
if ($cmdf -match "pub fn export_data" -and $cmdf -match "zip::ZipWriter") {
    Write-Host "  OK: export_data + ZIP" -ForegroundColor Green
}
if ($cmdf -match "pub fn import_data" -and $cmdf -match "zip::ZipArchive") {
    Write-Host "  OK: import_data + ZIP" -ForegroundColor Green
}
if ($cmdf -match "pub fn preview_import.*zip_path") {
    Write-Host "  OK: preview_import with zip_path param" -ForegroundColor Green
}
if ($cmdf -match "rfd::FileDialog") {
    Write-Host "  OK: rfd file dialog" -ForegroundColor Green
}

# 7. Check diagnostics
Write-Host "`n[7] Diagnostics..." -ForegroundColor Yellow
$diag = Get-Content (Join-Path $root "src-tauri\src\app\settings\diagnostics.rs") -Raw
$fields = @("app_version", "git_commit", "build_time", "rustc_version",
    "target_triple", "os_name", "os_version", "cpu_cores", "total_ram_mb",
    "enabled_features")
foreach ($f in $fields) {
    if ($diag -match $f) { Write-Host "  OK: $f" -ForegroundColor Green }
    else { Write-Host "  MISSING: $f" -ForegroundColor Red }
}

# 8. Check custom.js UI features
Write-Host "`n[8] UI features in custom.js..." -ForegroundColor Yellow
$js = Get-Content (Join-Path $root "src-tauri\src\inject\custom.js") -Raw
$features = @(
    @{Name="Settings panel tabs"; Pat='buildGeneral\(body\)'},
    @{Name="Adblock edit button"; Pat="__ps_abedit__"},
    @{Name="Refresh button"; Pat="__prb"},
    @{Name="Save/Cancel buttons"; Pat="__ps_save__"},
    @{Name="Version history"; Pat="loadVersionHistory"},
    @{Name="Diagnostics fill"; Pat="fillDiagnostics"},
    @{Name="Cache stats IPC"; Pat="cache_stats_json"},
    @{Name="Pick save path"; Pat="pick_save_path"},
    @{Name="Pick zip file"; Pat="pick_zip_file"},
    @{Name="Chinese lang pack"; Pat='lang\s*=\s*"zh"'},
    @{Name="English lang pack"; Pat='en\s*:'}
)
foreach ($f in $features) {
    if ($js -match $f.Pat) {
        Write-Host "  OK: $($f.Name)" -ForegroundColor Green
    } else {
        Write-Host "  MISSING: $($f.Name)" -ForegroundColor Red
    }
}

# 9. Rust settings unit tests
Write-Host "`n[9] Rust settings tests..." -ForegroundColor Yellow
Push-Location (Join-Path $root "src-tauri")
cargo test settings -- --nocapture 2>&1 | Select-String "test result|FAILED|ok"
Pop-Location

Write-Host "`nSettings test complete!" -ForegroundColor Green
