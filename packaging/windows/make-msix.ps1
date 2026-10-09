<#
Packs the already-built app into an .msix for the Microsoft Store.

Before running:   npx tauri build --no-bundle --features msstore
Then:             ./packaging/windows/make-msix.ps1 -IdentityName "12345Name.SoftMouseWiggler" `
                      -Publisher "CN=XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX" -PublisherDisplayName "Your Name"

The three values are shown in Partner Center under
Product management > Product identity. The package is left unsigned on
purpose: the Store signs it during certification.
#>
param(
  [Parameter(Mandatory = $true)][string]$IdentityName,
  [Parameter(Mandatory = $true)][string]$Publisher,
  [Parameter(Mandatory = $true)][string]$PublisherDisplayName,
  [string]$OutDir = "."
)
$ErrorActionPreference = "Stop"
$root = Resolve-Path "$PSScriptRoot/../.."
$exe = Join-Path $root "target/release/softmousewiggler.exe"
if (-not (Test-Path $exe)) { throw "Build first: npx tauri build --no-bundle --features msstore" }

# version 0.1.0 in tauri.conf.json becomes 0.1.0.0 (the Store wants four parts, the last one 0)
$conf = Get-Content (Join-Path $root "src-tauri/tauri.conf.json") -Raw | ConvertFrom-Json
$version = "$($conf.version).0"

$stage = Join-Path $env:TEMP "smw-msix"
Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force (Join-Path $stage "Assets") | Out-Null
Copy-Item $exe $stage
foreach ($png in "StoreLogo.png", "Square150x150Logo.png", "Square44x44Logo.png") {
  Copy-Item (Join-Path $root "src-tauri/icons/$png") (Join-Path $stage "Assets")
}
$manifest = Get-Content (Join-Path $PSScriptRoot "AppxManifest.xml") -Raw
$manifest = $manifest.Replace("{{IDENTITY_NAME}}", $IdentityName).Replace("{{PUBLISHER}}", $Publisher)
$manifest = $manifest.Replace("{{PUBLISHER_DISPLAY_NAME}}", $PublisherDisplayName).Replace("{{VERSION}}", $version)
Set-Content -Path (Join-Path $stage "AppxManifest.xml") -Value $manifest -Encoding UTF8

# makeappx.exe ships with the Windows SDK (already present on GitHub's Windows runners)
$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\makeappx.exe" |
  Sort-Object FullName -Descending | Select-Object -First 1
if (-not $makeappx) { throw "makeappx.exe not found. Install the Windows SDK." }

New-Item -ItemType Directory -Force $OutDir | Out-Null
$out = Join-Path (Resolve-Path $OutDir) "SoftMouseWiggler_${version}_x64.msix"
& $makeappx.FullName pack /d $stage /p $out /o
if ($LASTEXITCODE -ne 0) { throw "makeappx failed" }
Write-Host "Created $out"
