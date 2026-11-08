# StoryQuest Studio — Windows 10/11 Build & Installer Packaging Script
$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host " StoryQuest Studio — Windows Desktop Build & Packaging      " -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Cyan

# 1. Verify Prerequisites
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Error "Rust toolchain (cargo/rustc) not found. Install from https://rustup.rs"
}
if (-not (Get-Command "npm" -ErrorAction SilentlyContinue)) {
    Write-Error "Node.js / npm not found. Install Node.js LTS from https://nodejs.org"
}

# 2. Run SQLite & Core Logic Verification Suite
Write-Host "[1/4] Running SQLite schema migrations & automated verification tests..." -ForegroundColor Green
python tests/verify_all_milestones.py

# 3. Install Frontend Dependencies & Compile TypeScript/Vite Bundle
Write-Host "[2/4] Installing frontend dependencies & compiling Vite production bundle..." -ForegroundColor Green
npm install
npm run build

# 4. Run Rust Unit Tests
Write-Host "[3/4] Running Rust backend cargo tests..." -ForegroundColor Green
Push-Location src-tauri
cargo test
Pop-Location

# 5. Build Tauri 2 Windows Installer (NSIS .exe + WiX .msi)
Write-Host "[4/4] Building Tauri 2 Windows Desktop Application & Installers..." -ForegroundColor Green
npx tauri build

Write-Host "Build Complete! Installers generated at:" -ForegroundColor Green
Write-Host " - src-tauri/target/release/bundle/nsis/StoryQuest Studio_1.0.0_x64-setup.exe"
Write-Host " - src-tauri/target/release/bundle/msi/StoryQuest Studio_1.0.0_x64_en-US.msi"
