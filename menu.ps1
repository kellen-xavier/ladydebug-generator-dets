# Launcher do Gerador de DETs.
# Encontra (ou compila) o gerador-dets.exe e abre o menu usando ESTA pasta
# como area de trabalho. Funciona tanto no pacote baixado quanto no repositorio.

$ErrorActionPreference = "Stop"
$raiz = Split-Path -Parent $MyInvocation.MyCommand.Path

# 1) exe ao lado deste script (pacote)  2) build local (repositorio)
$candidatos = @(
    (Join-Path $raiz "gerador-dets.exe"),
    (Join-Path $raiz "target\release\gerador-dets.exe")
)
$exe = $candidatos | Where-Object { Test-Path $_ } | Select-Object -First 1

# 3) nao achou: se tiver cargo, compila na hora
if (-not $exe) {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "gerador-dets.exe nao encontrado. Compilando (primeira vez, ~15s)..." -ForegroundColor Yellow
        Push-Location $raiz
        cargo build --release
        Pop-Location
        $exe = Join-Path $raiz "target\release\gerador-dets.exe"
    }
}

if (-not $exe -or -not (Test-Path $exe)) {
    Write-Host "Nao encontrei o gerador-dets.exe nesta pasta." -ForegroundColor Red
    Write-Host "Copie o gerador-dets.exe para esta pasta, ou instale o Rust (https://rustup.rs) e rode de novo."
    exit 1
}

# Abre o menu usando a pasta do launcher como area de trabalho.
& $exe --base $raiz
