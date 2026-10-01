param(
  [Parameter(Mandatory = $true)]
  [string]$RubyPrefix,

  [Parameter(Mandatory = $true)]
  [string]$SemauriPlatform,

  [string]$OutDir
)

$ErrorActionPreference = 'Stop'
$RootDir = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not $OutDir) {
  $OutDir = Join-Path $RootDir 'dist'
}

$VersionFile = Get-Content (Join-Path $RootDir 'lib\semauri\version.rb') -Raw
if ($VersionFile -notmatch 'VERSION = "([^"]+)"') {
  throw 'Unable to determine Semauri version'
}
$Version = $Matches[1]

$RubyExe = Join-Path $RubyPrefix 'bin\ruby.exe'
if (-not (Test-Path $RubyExe -PathType Leaf)) {
  throw "Ruby runtime is missing: $RubyExe"
}

$RubyVersion = (& $RubyExe -e 'print RUBY_VERSION')
if ($LASTEXITCODE -ne 0) {
  throw 'Unable to execute the private Ruby runtime'
}

$ArchiveBase = "semauri-$Version-$SemauriPlatform"
$WorkDir = Join-Path ([IO.Path]::GetTempPath()) ("semauri-dist-" + [Guid]::NewGuid().ToString('N'))
$StageDir = Join-Path $WorkDir $ArchiveBase
$ArchivePath = Join-Path $OutDir "$ArchiveBase.zip"

try {
  New-Item -ItemType Directory -Force -Path (Join-Path $StageDir 'bin'), (Join-Path $StageDir 'app'), (Join-Path $StageDir 'runtime'), $OutDir | Out-Null

  Copy-Item (Join-Path $RootDir 'distribution\bin\semauri.cmd') (Join-Path $StageDir 'bin\semauri.cmd')
  Copy-Item (Join-Path $RootDir 'bin') (Join-Path $StageDir 'app') -Recurse
  Copy-Item (Join-Path $RootDir 'lib') (Join-Path $StageDir 'app') -Recurse
  Copy-Item (Join-Path $RootDir 'LICENSE'), (Join-Path $RootDir 'NOTICE'), (Join-Path $RootDir 'README.md') $StageDir
  Copy-Item $RubyPrefix (Join-Path $StageDir 'runtime\ruby') -Recurse

  $Manifest = [ordered]@{
    name = 'semauri'
    version = $Version
    platform = $SemauriPlatform
    compiler = 'ruby-reference'
    private_runtime = [ordered]@{
      name = 'ruby'
      version = $RubyVersion
    }
  }
  $Manifest | ConvertTo-Json -Depth 4 | Set-Content -Path (Join-Path $StageDir 'manifest.json') -Encoding utf8

  # Prove the copied runtime and Windows launcher work after relocation.
  & (Join-Path $StageDir 'bin\semauri.cmd') version
  if ($LASTEXITCODE -ne 0) { throw 'semauri version smoke test failed' }

  & (Join-Path $StageDir 'bin\semauri.cmd') check (Join-Path $RootDir 'examples\hello.sema')
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
