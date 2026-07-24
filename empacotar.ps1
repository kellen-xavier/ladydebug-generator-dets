# Monta o pacote distribuivel do Gerador de DETs (exe + modelo + launcher +
# instrucoes) em dist\gerador-dets\ e gera dist\gerador-dets.zip.
#
# Uso:  powershell -ExecutionPolicy Bypass -File .\empacotar.ps1

$ErrorActionPreference = "Stop"
$raiz = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "1/3  Compilando release..." -ForegroundColor Cyan
Push-Location $raiz
cargo build --release
Pop-Location

$exe = Join-Path $raiz "target\release\gerador-dets.exe"
if (-not (Test-Path $exe)) {
    Write-Host "Build falhou: gerador-dets.exe nao encontrado." -ForegroundColor Red
    exit 1
}

Write-Host "2/3  Montando a pasta do pacote..." -ForegroundColor Cyan
$out = Join-Path $raiz "dist\gerador-dets"
if (Test-Path $out) { Remove-Item $out -Recurse -Force }
New-Item -ItemType Directory -Path (Join-Path $out "modelos") -Force | Out-Null

Copy-Item $exe $out
Copy-Item (Join-Path $raiz "menu.ps1") $out
Copy-Item (Join-Path $raiz "ABRIR MENU.bat") $out
Copy-Item (Join-Path $raiz "modelos\modelo_det.docx") (Join-Path $out "modelos")

$leiame = @"
GERADOR DE DETs - COMO USAR

1) Coloque NESTA pasta a sua planilha exportada
   (o arquivo comeca com "teste_selected" e termina em .xlsx).
2) De um duplo-clique em "ABRIR MENU.bat".
3) No menu:
   [1] Selecionar / criar a Release e o ID CARD   (comece por aqui)
   [2] Criar as subpastas dos testes
       -> depois, coloque os prints (imagens) dentro de cada subpasta
   [3] Gerar os DETs (.docx)
   [4] e [5] geram PDF (precisam do LibreOffice; o compilado tambem do qpdf)

Nada precisa ser instalado para as opcoes [1], [2] e [3].
"@
Set-Content -Path (Join-Path $out "COMO USAR.txt") -Value $leiame -Encoding UTF8

Write-Host "3/3  Gerando o .zip..." -ForegroundColor Cyan
$zip = Join-Path $raiz "dist\gerador-dets.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $out "*") -DestinationPath $zip

Write-Host ""
Write-Host "Pacote pronto:" -ForegroundColor Green
Write-Host "  Pasta: $out"
Write-Host "  Zip:   $zip"
