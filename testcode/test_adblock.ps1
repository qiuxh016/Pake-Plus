# Pake Plus - Adblock Module Test (A)
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Adblock (A) Module Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. Check adblock source files
Write-Host "`n[1] Source files..." -ForegroundColor Yellow
$files = @(
    "src-tauri\src\adblock\mod.rs",
    "src-tauri\src\adblock\engine.rs",
    "src-tauri\src\adblock\rules.rs",
    "src-tauri\src\inject\adblock.js"
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
@("mod adblock", "AdblockState", "AdblockState::new", "adblock_report_blocked",
  "adblock_get_stats", "adblock_add_rule", "adblock_remove_rule") | ForEach-Object {
    if ($lib -match [regex]::Escape($_)) {
        Write-Host "  OK: $_ registered" -ForegroundColor Green
    } else {
        Write-Host "  MISSING: $_" -ForegroundColor Red
    }
}

# 3. Check permissions
Write-Host "`n[3] Permissions..." -ForegroundColor Yellow
$perm = Get-Content (Join-Path $root "src-tauri\permissions\settings.toml") -Raw
@("adblock_report_blocked", "adblock_get_stats", "adblock_add_rule", "adblock_remove_rule") | ForEach-Object {
    if ($perm -match [regex]::Escape($_)) {
        Write-Host "  OK: $_ authorized" -ForegroundColor Green
    } else {
        Write-Host "  MISSING: $_" -ForegroundColor Red
    }
}

# 4. Check pake.json config
Write-Host "`n[4] pake.json config..." -ForegroundColor Yellow
$pake = Get-Content (Join-Path $root "src-tauri\pake.json") -Raw | ConvertFrom-Json
Write-Host "  block_ads = $($pake.block_ads)"
Write-Host "  adblock_rules = '$($pake.adblock_rules)'"

# 5. Check CLI --block-ads option
Write-Host "`n[5] CLI --block-ads..." -ForegroundColor Yellow
Push-Location $root
$help = node "$root\dist\cli.js" --help 2>&1
if ($help -match "block-ads") { Write-Host "  OK: --block-ads exists" -ForegroundColor Green }
if ($help -match "adblock-rules") { Write-Host "  OK: --adblock-rules exists" -ForegroundColor Green }
Pop-Location

# 6. Check window.rs injection
Write-Host "`n[6] window.rs injection..." -ForegroundColor Yellow
$win = Get-Content (Join-Path $root "src-tauri\src\app\window.rs") -Raw
if ($win -match "adblock") {
    Write-Host "  OK: adblock injection logic found" -ForegroundColor Green
} else {
    Write-Host "  MISSING: adblock injection in window.rs" -ForegroundColor Red
}

# 7. Rust unit tests
Write-Host "`n[7] Rust adblock tests..." -ForegroundColor Yellow
Push-Location (Join-Path $root "src-tauri")
cargo test adblock -- --nocapture 2>&1 | Select-String "test result|FAILED|ok"
Pop-Location

Write-Host "`nAdblock test complete!" -ForegroundColor Green
