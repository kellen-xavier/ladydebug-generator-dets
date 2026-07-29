# Launcher do Gerador de DETs.
#   1) garante que o gerador-dets.exe existe (usa o do pacote ou compila);
#   2) verifica as ferramentas de PDF e oferece instalar o que faltar (winget);
#   3) abre o menu usando ESTA pasta como area de trabalho.
# As acoes [1] a [4] (organizar, .zip, gerar .docx) NAO dependem de nada externo.

$ErrorActionPreference = "Stop"
$raiz = Split-Path -Parent $MyInvocation.MyCommand.Path

# ── 1) exe: ao lado do script (pacote) ou build local (repositorio) ──────────
$candidatos = @(
    (Join-Path $raiz "gerador-dets.exe"),
    (Join-Path $raiz "target\release\gerador-dets.exe")
)
$exe = $candidatos | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $exe) {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "Compilando o gerador-dets.exe (primeira vez, ~15s)..." -ForegroundColor Yellow
        Push-Location $raiz; cargo build --release; Pop-Location
        $exe = Join-Path $raiz "target\release\gerador-dets.exe"
    }
}
if (-not $exe -or -not (Test-Path $exe)) {
    Write-Host "Nao encontrei o gerador-dets.exe nesta pasta." -ForegroundColor Red
    Write-Host "Copie o gerador-dets.exe para esta pasta, ou instale o Rust (https://rustup.rs) e rode de novo."
    exit 1
}

# ── 2) ferramentas de PDF (opcionais): oferece instalar o que faltar ─────────
function Test-Comando([string]$nome) { [bool](Get-Command $nome -ErrorAction SilentlyContinue) }
function Test-LibreOffice {
    if (Test-Comando "soffice") { return $true }
    return (Test-Path "C:\Program Files\LibreOffice\program\soffice.exe") -or
           (Test-Path "C:\Program Files (x86)\LibreOffice\program\soffice.exe")
}

$faltam = @()
if (-not (Test-LibreOffice))    { $faltam += [pscustomobject]@{ Nome = "LibreOffice"; Id = "TheDocumentFoundation.LibreOffice" } }
if (-not (Test-Comando "qpdf")) { $faltam += [pscustomobject]@{ Nome = "qpdf";        Id = "QPDF.QPDF" } }

if ($faltam.Count -gt 0) {
    Write-Host ""
    Write-Host ("Para EXPORTAR EM PDF (opcoes [5] e [6]) falta: " + ($faltam.Nome -join ", ") + ".") -ForegroundColor Yellow
    Write-Host "As opcoes [1] a [4] (organizar, .zip, gerar .docx) ja funcionam sem isso." -ForegroundColor DarkGray

    if (Test-Comando "winget") {
        $resp = Read-Host "Instalar agora automaticamente? (S/n)"
        if ($resp -eq "" -or $resp -match '^[sSyY]') {
            foreach ($f in $faltam) {
                Write-Host ("Instalando " + $f.Nome + " ...") -ForegroundColor Cyan
                winget install --id $f.Id -e --accept-package-agreements --accept-source-agreements --silent
            }
            Write-Host "Concluido. Se o PDF nao funcionar de primeira, feche e abra o menu de novo (o PATH atualiza)." -ForegroundColor Green
        } else {
            Write-Host "Ok, seguindo sem PDF. Voce pode instalar depois: e so abrir o menu novamente." -ForegroundColor DarkGray
        }
    } else {
        Write-Host "winget nao esta disponivel nesta maquina. Para PDF, instale manualmente:" -ForegroundColor DarkYellow
        Write-Host "  LibreOffice: https://www.libreoffice.org/download/"
        Write-Host "  qpdf:        https://github.com/qpdf/qpdf/releases"
    }
    Write-Host ""
}

# ── 3) abre o menu usando a pasta do launcher como area de trabalho ──────────
& $exe --base $raiz
