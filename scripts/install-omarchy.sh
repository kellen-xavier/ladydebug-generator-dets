#!/usr/bin/env bash
set -euo pipefail
raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
"$raiz/scripts/build-linux.sh"
destino="${XDG_DATA_HOME:-$HOME/.local/share}/gestor-det"
atalho="${XDG_DATA_HOME:-$HOME/.local/share}/applications/gestor-det.desktop"
mkdir -p "$destino" "$(dirname "$atalho")"
cp -a "$raiz/app/build/linux/x64/release/bundle/." "$destino/"
cat > "$atalho" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Gestor de DETs
Comment=Organizar evidências e gerar documentos de testes
Exec="$destino/det_app"
Icon=x-office-document
Terminal=false
Categories=Office;Utility;
StartupNotify=true
DESKTOP
printf 'Instalado: %s\nAtalho: %s\n' "$destino" "$atalho"
