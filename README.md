# Gestor de Documentos de Testes (DET)

Ferramenta de terminal TUI (Terminal User Interface) para **gerar e organizar as evidências de testes manuais** na sua máquina local e Gerar relatórios auditáveis dos Testes. Você baixa no seu PC, abre o terminal na raiz do projeto, executa o comando `det` e escolhe a ação no menu.

```txt
==========================================================
  Gestor de Documentos de Testes (DET)
  Area de trabalho: C:\testes\meu-projeto
==========================================================
  -- Organizar ----------------------------------------
   [1] Criar Pasta da Release  (Release Julho 2026)
   [2] Criar subpastas dos testes  (ID - nome)
  -- Gerar --------------------------------------------
   [3] Gerar DET - docx
   [4] Gerar DET PDF
   [5] Gerar DET compilado
  ----------------------------------------------------
   [0] Sair
==========================================================
  Selecione a acao [0-5]:
```

## Instalação

Requer o [Rust](https://www.rust-lang.org/tools/install) (comando `cargo`). Depois:

```bash
git clone <este-repositorio>
cd gestor-det
cargo install --path .
```

Isso compila e coloca o binário `det` no PATH (`~/.cargo/bin` no Linux/macOS, `%USERPROFILE%\.cargo\bin` no Windows). A partir daí, `det` roda de qualquer pasta.

Para apenas compilar sem instalar:

```bash
cargo build --release
# binario em target/release/det (Windows: det.exe)
```

## Como usar

Abra o terminal **na pasta de trabalho** do projeto e rode:

```bash
det
```

Ou aponte a pasta explicitamente:

```bash
det --base "C:\testes\meu-projeto"
```

O menu abre; você digita o número da ação e responde às perguntas (os caminhos vêm preenchidos com valores detectados — basta apertar Enter para aceitar).

## Estrutura da área de trabalho

```txt
<pasta-de-trabalho>/
├─ modelos/
│    └─ modelo_det.docx            # modelo do DET (com os tokens {{...}})
├─ Release/                        # pasta central das releases
│    └─ Release <Mês> <AAAA>/      # criada pela ação [1]
│         ├─ 13 - Login/           # subpasta do teste (ação [2]); coloque aqui as imagens
│         │    ├─ captura_1.png
│         │    ├─ captura_2.png
│         │    ├─ DET_13_....docx  # gerado pela ação [3]
│         │    └─ DET_13_....pdf   # gerado pela ação [4]
│         └─ DET_Compilado.pdf     # gerado pela ação [5]
└─ teste_selected_16_07_2026_10_30_00.xlsx   # planilha exportada (a mais recente é usada)
```

## As cinco ações

1. **Criar Pasta da Release** — cria `Release/Release <Mês> <AAAA>` com base na data local da máquina.
2. **Criar subpastas dos testes** — lê a planilha e cria uma subpasta `ID - nome` (nome truncado em 10 caracteres) por teste. Coloque as imagens de evidência dentro de cada subpasta.
3. **Gerar DET - docx** — para cada teste (pulando `Status nativo = Ignorado`), copia o modelo, preenche os campos e insere as imagens da subpasta em **ordem numérica natural** (`1, 2, 10`).
4. **Gerar DET PDF** — converte cada `DET_*.docx` gerado em PDF (via LibreOffice).
5. **Gerar DET compilado** — junta todos os PDFs das subpastas em um único `DET_Compilado.pdf`. Se passar de **30 MB**, divide em `DET_Compilado_Parte_1.pdf`, `Parte_2`, ... na ordem natural.

## Modelo do DET (`modelos/modelo_det.docx`)

O repositório já traz um **modelo inicial** pronto. Reestilize à vontade no Word, mantendo os **tokens** (podem estar em qualquer parágrafo, tabela ou cabeçalho do corpo):

| Token | Preenchido com |
|---|---|
| `{{ID}}` | coluna `ID` |
| `{{NOME_TESTE}}` | coluna `Nome do teste (inicial)` |
| `{{EXECUTADO_POR}}` | coluna `Executado por` |
| `{{STATUS}}` | coluna `Status nativo` |
| `{{DATA}}` | data local (DD/MM/AAAA) |
| `{{EVIDENCIAS}}` | parágrafo substituído pelas imagens da subpasta |

O preenchimento é **robusto a "run splitting"**: mesmo que o Word divida `{{NOME_TESTE}}` internamente em vários trechos, o token é reconstituído e substituído corretamente.

## Colunas obrigatórias da planilha

`ID` · `Nome do teste (inicial)` · `Executado por` · `Status nativo`

O casamento de cabeçalho é **tolerante** (sem acento, maiúsc./minúsc., espaços e pontuação). Aceita variações como `Executado por` / `Executador por` / `Executor` / `Responsável` e `Nome do teste` / `Nome do teste (inicial)`.

## Requisitos externos (ações 4 e 5)

As ações de PDF chamam ferramentas de linha de comando, seguindo o mesmo padrão dos scripts do projeto:

- **Gerar DET PDF** → [LibreOffice](https://www.libreoffice.org/) (`soffice`). No Windows, o caminho padrão de instalação é detectado automaticamente.
- **Gerar DET compilado** → [`qpdf`](https://qpdf.sourceforge.io/) para juntar; [Ghostscript](https://www.ghostscript.com/) (`gs`) **opcional** para comprimir (se ausente, o compilado é gerado sem recompressão).

A geração do `.docx` (ações 1–3) **não** depende de nada externo — apenas do binário.

## Notas e limitações

- **Fuso horário**: a data usa o relógio do sistema em UTC (a `std` do Rust não expõe o fuso). Para a maioria dos fusos a data cai no dia correto; o ponto de ajuste está isolado em `src/util.rs` (`offset_local`).
- **Nome da release**: o formato é `Release <Mês> <Ano>` (ex.: `Release Julho 2026`). Para mudar, edite `nome_release_atual` em `src/workspace.rs`.
- **Divisão do compilado**: o corte de 30 MB agrupa **arquivos inteiros** (não divide por página). Um PDF isolado maior que o limite vira uma parte própria.
- **Portabilidade**: validado com foco no Windows; como usa só `std` + o crate `zip`, roda também em Linux/macOS.

## Estrutura do código

```txt
src/
├─ main.rs        # ponto de entrada + laço do menu + dispatch
├─ menu.rs        # desenho do menu e leitura das respostas
├─ workspace.rs   # descoberta de Release/, modelos/ e da planilha mais recente
├─ acoes.rs       # as cinco ações
├─ xlsx.rs        # leitor de .xlsx (shared strings + worksheet)
├─ docx.rs        # motor de .docx (preenche tokens + insere imagens)
├─ pdf.rs         # integração com LibreOffice / qpdf / Ghostscript
└─ util.rs        # logging, ordenação natural, normalização, datas, XML
```

Dependência única: o crate [`zip`](https://crates.io/crates/zip). O `.docx` e o `.xlsx` são manipulados diretamente como ZIP + XML.

## Testes

```bash

cargo build 

cargo build --release

cargo test

```
