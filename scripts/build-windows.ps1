$ErrorActionPreference = 'Stop'
$raiz = Split-Path -Parent $PSScriptRoot
Push-Location $raiz
try {
  cargo build --release --bin det-api
  if ($LASTEXITCODE -ne 0) { throw 'Falha no build Rust' }
  Push-Location app
  try { flutter build windows --release; if ($LASTEXITCODE -ne 0) { throw 'Falha no build Flutter' } }
  finally { Pop-Location }
  $pacote = Join-Path $raiz 'app\build\windows\x64\runner\Release'
  Copy-Item (Join-Path $raiz 'target\release\det-api.exe') $pacote
  Write-Host "Pacote Windows: $pacote"
}
finally { Pop-Location }
