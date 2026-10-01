param(
  [string]$Version = $env:SEMAURI_VERSION,
  [string]$InstallRoot = $env:SEMAURI_INSTALL_ROOT
)

$ErrorActionPreference = 'Stop'
$Repo = 'edujbarrios/semauri'

if (-not $InstallRoot) {
  $InstallRoot = Join-Path $HOME '.semauri'
}
$InstallRoot = [IO.Path]::GetFullPath($InstallRoot)
$BinDir = Join-Path $InstallRoot 'bin'
$VersionsDir = Join-Path $InstallRoot 'versions'

function Fail([string]$Message) {
  throw "semauri-install: $Message"
}

try {
  $Architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
} catch {
  $Architecture = $env:PROCESSOR_ARCHITECTURE.ToLowerInvariant()
}

switch ($Architecture) {
  'x64'   { $Arch = 'x86_64' }
  'amd64' { $Arch = 'x86_64' }
  'arm64' { $Arch = 'arm64' }
  default { Fail "unsupported Windows architecture: $Architecture. Supported: x86_64 and arm64." }
}

if (-not $Version) {
  try {
    $Latest = Invoke-RestMethod -UseBasicParsing -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $Latest.tag_name -replace '^v', ''
  } catch {
    Fail "unable to resolve the latest Semauri release: $($_.Exception.Message)"
  }
}
$Version = $Version -replace '^v', ''

$Platform = "windows-$Arch"
$ArchiveBase = "semauri-$Version-$Platform"
$Asset = "$ArchiveBase.zip"
$BaseUrl = "https://github.com/$Repo/releases/download/v$Version"
$TempDir = Join-Path ([IO.Path]::GetTempPath()) ("semauri-install-" + [Guid]::NewGuid().ToString('N'))
$Archive = Join-Path $TempDir $Asset
$Checksums = Join-Path $TempDir 'SHA256SUMS'
$ExtractDir = Join-Path $TempDir 'extract'
$Destination = Join-Path $VersionsDir $Version

try {
  New-Item -ItemType Directory -Force -Path $TempDir, $ExtractDir, $VersionsDir, $BinDir | Out-Null

  Write-Host "Installing Semauri $Version for $Platform..."
  try {
    Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/$Asset" -OutFile $Archive
    Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/SHA256SUMS" -OutFile $Checksums
  } catch {
    Fail "unable to download release assets for Semauri ${Version}: $($_.Exception.Message)"
  }

  $ChecksumLine = Get-Content $Checksums | Where-Object { $_ -match "^[0-9a-fA-F]{64}\s+$([Regex]::Escape($Asset))$" } | Select-Object -First 1
  if (-not $ChecksumLine) {
    Fail "checksum for $Asset is missing"
  }
  $Expected = ($ChecksumLine -split '\s+')[0].ToLowerInvariant()
  $Actual = (Get-FileHash -Algorithm SHA256 -Path $Archive).Hash.ToLowerInvariant()
  if ($Expected -ne $Actual) {
    Fail "checksum verification failed for $Asset"
  }

  Expand-Archive -Path $Archive -DestinationPath $ExtractDir -Force
  $ExtractedRoot = Join-Path $ExtractDir $ArchiveBase
  if (-not (Test-Path $ExtractedRoot -PathType Container)) {
    Fail "release archive has an unexpected layout"
  }

  if (Test-Path $Destination) {
    Remove-Item $Destination -Recurse -Force
  }
  Move-Item $ExtractedRoot $Destination

  Set-Content -Path (Join-Path $InstallRoot 'current.txt') -Value $Version -Encoding ascii

  $ShimPath = Join-Path $BinDir 'semauri.cmd'
  @'
@echo off
setlocal
for %%I in ("%~dp0..") do set "SEMAURI_INSTALL_ROOT=%%~fI"
set /p SEMAURI_VERSION=<"%SEMAURI_INSTALL_ROOT%\current.txt"
if not defined SEMAURI_VERSION (
  echo Semauri installation is incomplete: current.txt is missing or empty. 1>&2
  exit /b 70
)
call "%SEMAURI_INSTALL_ROOT%\versions\%SEMAURI_VERSION%\bin\semauri.cmd" %*
exit /b %ERRORLEVEL%
'@ | Set-Content -Path $ShimPath -Encoding ascii

  $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  $PathEntries = @()
  if ($UserPath) {
    $PathEntries = $UserPath.Split(';', [StringSplitOptions]::RemoveEmptyEntries)
  }
  $AlreadyInPath = $PathEntries | Where-Object { $_.TrimEnd('\') -ieq $BinDir.TrimEnd('\') }
  if (-not $AlreadyInPath) {
    $NewUserPath = if ($UserPath) { "$BinDir;$UserPath" } else { $BinDir }
    [Environment]::SetEnvironmentVariable('Path', $NewUserPath, 'User')
  }
  if (-not (($env:Path -split ';') | Where-Object { $_.TrimEnd('\') -ieq $BinDir.TrimEnd('\') })) {
    $env:Path = "$BinDir;$env:Path"
  }

  & $ShimPath version
  if ($LASTEXITCODE -ne 0) {
    Fail 'installed Semauri failed its version smoke test'
  }

  Write-Host ''
  Write-Host "Semauri $Version installed successfully."
  Write-Host "Command: $ShimPath"
  Write-Host 'Open a new terminal if the semauri command is not yet visible in PATH.'
}
finally {
  if (Test-Path $TempDir) {
    Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue
  }
}
