param(
  [Parameter(Mandatory = $true)]
  [string]$BinaryPath,

  [Parameter(Mandatory = $true)]
  [string]$SemauriPlatform,

  [string]$OutDir
)

$ErrorActionPreference = 'Stop'
$RootDir = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
if (-not $OutDir) {
  $OutDir = Join-Path $RootDir 'dist'
}

$CargoToml = Get-Content (Join-Path $RootDir 'Cargo.toml') -Raw
if ($CargoToml -notmatch '(?m)^version = "([^"]+)"') {
  throw 'Unable to determine Semauri version from Cargo.toml'
}
$Version = $Matches[1]

$ResolvedBinary = (Resolve-Path $BinaryPath).Path
if (-not (Test-Path $ResolvedBinary -PathType Leaf)) {
  throw "Semauri binary is missing: $BinaryPath"
}

$ArchiveBase = "semauri-$Version-$SemauriPlatform"
$WorkDir = Join-Path ([IO.Path]::GetTempPath()) ("semauri-dist-" + [Guid]::NewGuid().ToString('N'))
$StageDir = Join-Path $WorkDir $ArchiveBase
$ArchivePath = Join-Path $OutDir "$ArchiveBase.zip"

try {
  New-Item -ItemType Directory -Force -Path (Join-Path $StageDir 'bin'), $OutDir | Out-Null
  Copy-Item $ResolvedBinary (Join-Path $StageDir 'bin\semauri.exe')
  Copy-Item (Join-Path $RootDir 'LICENSE'), (Join-Path $RootDir 'NOTICE'), (Join-Path $RootDir 'README.md') $StageDir

  $Manifest = [ordered]@{
    name = 'semauri'
    version = $Version
    platform = $SemauriPlatform
    compiler = 'rust-native'
  }
  $Manifest | ConvertTo-Json -Depth 3 | Set-Content -Path (Join-Path $StageDir 'manifest.json') -Encoding utf8

  & (Join-Path $StageDir 'bin\semauri.exe') version
  if ($LASTEXITCODE -ne 0) { throw 'semauri version smoke test failed' }

  & (Join-Path $StageDir 'bin\semauri.exe') check (Join-Path $RootDir 'examples\hello.sema')
  if ($LASTEXITCODE -ne 0) { throw 'semauri check smoke test failed' }

  if (Test-Path $ArchivePath) { Remove-Item $ArchivePath -Force }
  Compress-Archive -Path $StageDir -DestinationPath $ArchivePath -CompressionLevel Optimal
  Write-Output $ArchivePath
}
finally {
  if (Test-Path $WorkDir) {
    Remove-Item $WorkDir -Recurse -Force
  }
}
