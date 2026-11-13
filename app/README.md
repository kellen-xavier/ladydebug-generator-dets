# Aplicativo Flutter para DETs

Interface desktop para Linux e Windows. O motor de planilhas, DOCX e PDF permanece
no crate Rust do repositório. A interface chama `det-api` localmente, sem rede
ou servidor. A TUI `gerador-dets` permanece disponível.

## Omarchy / Arch Linux

No repositório:

```bash
./scripts/install-omarchy.sh
```

O script gera o pacote Linux, copia para `~/.local/share/gestor-det` e cria
`~/.local/share/applications/gestor-det.desktop`. O aplicativo aparecerá no
lançador como **Gestor de DETs**. Não altera a configuração do Hyprland.

Para desenvolver:

```bash
cargo build --bin det-api
cd app && flutter run -d linux
```

O `.mise.toml` do repositório seleciona Flutter 3.47.9 no Omarchy. Para gerar
apenas o pacote, use `./scripts/build-linux.sh`.

## Windows

No PowerShell com Rust e Flutter instalados:

```powershell
.\scripts\build-windows.ps1
```

Distribua a pasta `app\build\windows\x64\runner\Release` inteira. O
`det-api.exe` fica ao lado do executável Flutter. O build Windows precisa
ser executado em Windows; não foi verificado nesta máquina Linux.

## Uso

Selecione uma pasta de trabalho que contenha `modelos/modelo_det.docx` e uma
planilha `.xlsx`. Selecione/crie a Release e o ID CARD. Execute as etapas na
ordem mostrada. A data usa UTC−3 por padrão; o fuso pode ser alterado na barra.
Para PDF, instale LibreOffice e qpdf; Ghostscript é opcional.

A variável `DET_API_BIN` permite apontar para outro binário Rust durante o
desenvolvimento. O aplicativo também procura `target/debug/det-api` e
`target/release/det-api` no repositório.

## Escolha da ponte

`flutter_rust_bridge` é uma opção para muitas chamadas Rust, streaming ou
compartilhamento de tipos. Aqui as ações são poucas, lentas e orientadas a
arquivos. O executável local evita dependência de runtime/codegen FFI e mantém
o binário TUI independente. A ponte usa argumentos separados (sem shell) e
uma última linha JSON marcada `DET_JSON:`; o Rust valida base, release e card.
