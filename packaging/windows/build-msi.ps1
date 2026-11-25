# Builds ccdc.exe and the MSI (dist\CCDC-<version>-windows-x64.msi). Run on Windows.
# Needs Rust and the .NET SDK; installs WiX 5 as a .NET tool if missing.
# WiX 5.0.2 is pinned: later versions ask you to accept an extra license (OSMF EULA).
$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
Set-Location $root
$version = (Select-String -Path Cargo.toml -Pattern '^windows = "(.*)"' | Select-Object -First 1).Matches[0].Groups[1].Value

cargo build --release
if ($LASTEXITCODE) { exit $LASTEXITCODE }

$tools = Join-Path $env:USERPROFILE '.dotnet\tools'
if ($env:PATH -notlike "*$tools*") { $env:PATH += ";$tools" }
if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
    dotnet tool install --global wix --version 5.0.2
}
wix extension add -g WixToolset.Util.wixext/5.0.2
New-Item -ItemType Directory -Force dist | Out-Null
$msi = "dist\CCDC-$version-windows-x64.msi"
wix build packaging\windows\ccdc.wxs -ext WixToolset.Util.wixext -arch x64 -d Version=$version -o $msi
if ($LASTEXITCODE) { exit $LASTEXITCODE }
Remove-Item "dist\CCDC-$version-windows-x64.wixpdb" -ErrorAction SilentlyContinue
Write-Host $msi
