

# init.ps1 - Project Environment Bootstrapper
Write-Host "--- Initializing MVSC build tool environment ---" -ForegroundColor Cyan

# 1. Find Visual Studio Path using vswhere
$vsPath = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath
if (-not $vsPath) {
    Write-Error "Visual Studio Build Tools not found. Check your installation."
    return
}

# 2. Locate and Import the Official DevShell Module
$modulePath = Join-Path $vsPath "Common7\Tools\Microsoft.VisualStudio.DevShell.dll"
if (Test-Path $modulePath) {
    Import-Module $modulePath
    # Enter the shell for 64-bit (Vermeer/5600X)
    Enter-VsDevShell -InstallPath $vsPath
    Write-Host "SUCCESS: MSVC Toolchain (nmake/cl) injected." -ForegroundColor Green
} else {
    Write-Error "DevShell DLL missing at $modulePath"
}

# 3. Verify Stage 2 Toolchain
if (Get-Command rustc -ErrorAction SilentlyContinue) {
    $version = rustc --version
    Write-Host "READY: Using $version" -ForegroundColor Gray
}

Write-Host "--------------------------------------------"