# Pake Plus - Cache Module Test (B)
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Cache (B) Module Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. Check cache source files
Write-Host "`n[1] Source files..." -ForegroundColor Yellow
$files = @(
    "src-tauri\src\cache\mod.rs",
    "src-tauri\src\cache\engine.rs",
    "src-tauri\src\cache\commands.rs",
    "src-tauri\src\inject\cache.js"
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

# 2. Check lib.rs wiring
Write-Host "`n[2] lib.rs wiring..." -ForegroundColor Yellow
$lib = Get-Content (Join-Path $root "src-tauri\src\lib.rs") -Raw
@("mod cache", "CacheState", "CacheState::new", "cache_fetch",
  "cache_get_stats", "cache_clear", "cache_stats_json", "cache_enabled", "cache_size") | ForEach-Object {
    if ($lib -match [regex]::Escape($_)) {
        Write-Host "  OK: $_" -ForegroundColor Green
    } else {
        Write-Host "  MISSING: $_" -ForegroundColor Red
    }
}

# 3. Check permissions
Write-Host "`n[3] Permissions..." -ForegroundColor Yellow
$perm = Get-Content (Join-Path $root "src-tauri\permissions\settings.toml") -Raw
@("cache_fetch", "cache_get_stats", "cache_clear", "cache_stats_json") | ForEach-Object {
    if ($perm -match [regex]::Escape($_)) {
        Write-Host "  OK: $_ authorized" -ForegroundColor Green
    } else {
        Write-Host "  MISSING: $_" -ForegroundColor Red
    }
}

# 4. Check PakeConfig fields
Write-Host "`n[4] PakeConfig..." -ForegroundColor Yellow
$cfg = Get-Content (Join-Path $root "src-tauri\src\app\config.rs") -Raw
if ($cfg -match "pub cache:" -and $cfg -match "pub cache_size:") {
    Write-Host "  OK: cache + cache_size fields defined" -ForegroundColor Green
} else {
    Write-Host "  MISSING: cache fields in config" -ForegroundColor Red
}

# 5. Check window.rs injection
Write-Host "`n[5] window.rs injection..." -ForegroundColor Yellow
$win = Get-Content (Join-Path $root "src-tauri\src\app\window.rs") -Raw
if ($win -match "config\.cache" -and $win -match "cache\.js") {
    Write-Host "  OK: cache.js injection logic found" -ForegroundColor Green
} else {
    Write-Host "  MISSING: cache injection in window.rs" -ForegroundColor Red
}

# 6. Check diagnostics
Write-Host "`n[6] Diagnostics..." -ForegroundColor Yellow
$diag = Get-Content (Join-Path $root "src-tauri\src\app\settings\diagnostics.rs") -Raw
if ($diag -match "settings\.cache\.enabled") {
    Write-Host "  OK: cache enabled check in diagnostics" -ForegroundColor Green
} else {
    Write-Host "  MISSING: cache in diagnostics" -ForegroundColor Red
}

# 7. Check save_settings sync
Write-Host "`n[7] save_settings cache sync..." -ForegroundColor Yellow
$cmds = Get-Content (Join-Path $root "src-tauri\src\app\settings\commands.rs") -Raw
if ($cmds -match "sync_cache_config") {
    Write-Host "  OK: sync_cache_config called in save_settings" -ForegroundColor Green
} else {
    Write-Host "  MISSING: cache sync in save" -ForegroundColor Red
}

# 8. Check cache.js offline features
Write-Host "`n[8] cache.js offline features..." -ForegroundColor Yellow
$js = Get-Content (Join-Path $root "src-tauri\src\inject\cache.js") -Raw
$features = @{
    "online/offline events" = "addEventListener.*online"
    "Offline banner" = "__pake_offline_bnr__"
    "OFFLINE badge" = "__pake_offline__"
    "Cache hit toast" = "__pake_cache_toast__"
    "Offline HTML page" = "offlineHTML"
    "fetch interception" = 'window\.fetch\s*='
    "XHR interception" = "window\.XMLHttpRequest\s*="
}
foreach ($name in $features.Keys) {
    if ($js -match $features[$name]) {
        Write-Host "  OK: $name" -ForegroundColor Green
    } else {
        Write-Host "  MISSING: $name" -ForegroundColor Red
    }
}

# 9. CLI options
Write-Host "`n[9] CLI options..." -ForegroundColor Yellow
Push-Location $root
$help = node "$root\dist\cli.js" --help 2>&1
if ($help -match "--cache\b") { Write-Host "  OK: --cache exists" -ForegroundColor Green }
if ($help -match "cache-size") { Write-Host "  OK: --cache-size exists" -ForegroundColor Green }
Pop-Location

# 10. Cargo check
Write-Host "`n[10] Cargo check..." -ForegroundColor Yellow
Push-Location (Join-Path $root "src-tauri")
$out = cargo check --no-default-features --features "custom-protocol,clipboard" 2>&1
$errs = ($out | Select-String "error\[").Count
if ($errs -eq 0) { Write-Host "  OK: 0 errors" -ForegroundColor Green }
else { Write-Host "  ERRORS: $errs" -ForegroundColor Red }
Pop-Location

Write-Host "`nCache test complete!" -ForegroundColor Green
