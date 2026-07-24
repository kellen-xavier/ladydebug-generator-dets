# Gestor de Documentos de Testes (DET)

Ferramenta de terminal TUI (Terminal User Interface) para **gerar e organizar as evidências de testes manuais** na sua máquina local e gerar relatórios auditáveis dos testes. Você baixa no seu PC, abre o terminal na pasta de trabalho, executa o comando `det` e escolhe a ação num menu — **sem precisar programar**.

```txt
==========================================================
  Gestor de Documentos de Testes (DET)
  Pasta de trabalho: C:\testes\meu-projeto
==========================================================
  Release atual: (nenhuma selecionada)
  Card atual:    (nenhum selecionado)
  ----------------------------------------------------
  Releases nesta pasta ([*] = a que voce esta usando):
     [ ] Release Julho 2026
==========================================================
  -- Organizar ----------------------------------------
   [1] Selecionar / criar Release e ID CARD   <- comece por aqui
   [2] Criar subpastas dos testes  (ID - nome)
  -- Gerar --------------------------------------------
   [3] Gerar DET (.docx)
   [4] Gerar DET em PDF
   [5] Gerar DET compilado (PDF unico)
  ----------------------------------------------------
   [6] Verificar ambiente
   [0] Sair
==========================================================
  Selecione uma opcao [0-6]:
```

---

## O que este programa faz

- **Organiza** suas evidências (os *prints* dos testes) em pastas padronizadas.
- **Monta o documento de evidências (DET)** de cada teste automaticamente, a partir de um modelo Word e da sua planilha de testes.
- **Converte** os documentos em PDF e pode **juntar tudo** num PDF único para anexar.

Você não escreve nenhum comando complicado: é só rodar `det` e escolher números no menu.

## Glossário rápido (para quem está começando)

| Termo | O que é, em linguagem simples |
|---|---|
| **DET** | *Documento de Evidência de Testes* — o Word/PDF com o passo a passo e os prints de um teste. |
| **Release** | O "pacote" do mês, ex.: `Release Julho 2026`. Agrupa o trabalho daquele período. |
| **ID CARD** | O cartão/tarefa que você está testando (um número, ex.: `ID CARD 1399338`). Fica dentro da Release. |
| **Teste (ID)** | Cada caso de teste tem um número (ex.: `13`) e um nome. Vira uma subpasta `13 - Login`. |
| **Evidência** | As imagens (prints) que provam que o teste foi feito. |
| **Planilha** | O arquivo Excel exportado do seu sistema de testes (nome começa com `teste_selected`). |
| **Modelo** | O Word base (`modelos/modelo_det.docx`) que o programa preenche para cada teste. |

---

## Instalação (só uma vez)

1. Instale o **[Rust](https://www.rust-lang.org/tools/install)** (ele traz o comando `cargo`). É um instalador simples, siga o padrão.
2. Abra o terminal (no Windows: *PowerShell*) e rode:

```bash
git clone <endereco-deste-repositorio>
cd ladydebug-generator-dets
cargo install --path .
```

Pronto: o comando `det` fica disponível em **qualquer pasta** do seu computador.

> 💡 Se aparecer *"det não é reconhecido"* logo após instalar, **feche e abra o terminal** de novo (para ele enxergar o novo comando).

*(Só quer testar sem instalar? Use `cargo run` dentro da pasta do projeto.)*

---

## Passo a passo — do zero ao DET pronto

### Antes de começar: prepare a pasta de trabalho

Crie (ou escolha) uma pasta para o seu trabalho e coloque nela **duas coisas**:

1. Uma pasta `modelos/` com o arquivo **`modelo_det.docx`** (já vem um pronto neste repositório — copie-o para lá).
2. A **planilha** exportada do seu sistema de testes (o arquivo `teste_selected_....xlsx`).

### Agora, com o programa

1. **Abra o terminal na sua pasta de trabalho** e digite:

   ```bash
   det
   ```

2. A primeira tela mostra o **Ambiente** (o que está instalado). Leia e aperte **Enter** para ir ao menu.

3. Escolha **`[1] Selecionar / criar Release e ID CARD`**:
   - **Passo 1 de 2** — escolha a Release na lista, ou tecle `N` para criar uma nova (o programa pergunta o mês e o ano).
   - **Passo 2 de 2** — digite o número do **ID CARD**. Se a pasta não existir, ele pergunta *"Essa pasta não existe. Deseja criar?"* — responda `S`.
   - No topo do menu agora aparece **`Release atual`** e **`Card atual`**: é onde tudo será salvo.

4. Escolha **`[2] Criar subpastas dos testes`**. O programa lê a planilha e cria uma pasta para cada teste (ex.: `13 - Login`).

5. **Coloque as imagens (prints)** de cada teste **dentro da subpasta correspondente**. Nomeie em ordem (`01`, `02`, ...) para elas entrarem na sequência certa.

6. Escolha **`[3] Gerar DET (.docx)`**. Ele preenche o modelo com os dados da planilha e insere as imagens. Pronto: cada teste ganha o seu `DET_...docx`.

7. *(Opcional)* **`[4] Gerar DET em PDF`** e **`[5] Gerar DET compilado`** para ter os PDFs e um arquivo único para anexar. *(Essas duas precisam de programas extras — veja a seção de PDF abaixo.)*

8. Para sair, escolha **`[0] Sair`**. Depois de qualquer ação, é só apertar **Enter** para voltar ao menu.

> ℹ️ As opções `[2]` a `[5]` só funcionam depois que você fez a `[1]` — se não, o programa avisa *"Selecione uma Release e um ID CARD primeiro"*.

---

## O menu, opção por opção

| Opção | O que faz |
|---|---|
| **[1] Selecionar / criar Release e ID CARD** | Escolhe (ou cria) a Release do mês e o ID CARD. Define **onde** tudo será salvo. |
| **[2] Criar subpastas dos testes** | Lê a planilha e cria uma subpasta `ID - nome` por teste, dentro do card. |
| **[3] Gerar DET (.docx)** | Para cada teste (pulando `Status = Ignorado`), preenche o modelo e insere as imagens em ordem natural (`1, 2, 10`). |
| **[4] Gerar DET em PDF** | Converte cada `DET_*.docx` em PDF. *(Precisa do LibreOffice.)* |
| **[5] Gerar DET compilado (PDF único)** | Junta os PDFs num só. Acima de 30 MB, divide em `Parte_1`, `Parte_2`... *(Precisa do qpdf.)* |
| **[6] Verificar ambiente** | Mostra o que está instalado (LibreOffice, qpdf, Ghostscript). |
| **[0] Sair** | Encerra o programa. |

---

## Como as pastas ficam organizadas

```txt
<pasta-de-trabalho>/
├─ modelos/
│   └─ modelo_det.docx                 # o modelo do DET (com os tokens {{...}})
├─ Release/                            # pasta central de todas as releases
│   └─ Release Julho 2026/             # a Release (criada no passo [1])
│       └─ ID CARD 1399338/            # o card do trabalho (passo [1])
│           ├─ 13 - Login/             # uma subpasta por teste (passo [2])
│           │   ├─ 01_print.png        # suas evidências (prints) vão AQUI
│           │   ├─ 02_print.png
│           │   ├─ DET_13_Login.docx   # gerado no passo [3]
│           │   └─ DET_13_Login.pdf    # gerado no passo [4]
│           └─ DET_Compilado.pdf       # gerado no passo [5]
└─ teste_selected_16_07_2026.xlsx      # a planilha (o programa usa a mais recente)
```

---

## O modelo do DET e os "tokens"

O repositório já traz um **modelo pronto** em `modelos/modelo_det.docx`. Você pode reestilizá-lo no Word à vontade (cores, logo, layout) — **só não apague os *tokens***. Token é um marcador escrito entre chaves duplas que o programa troca pelo valor real:

| Token no modelo | É trocado por |
|---|---|
| `{{ID}}` | o número do teste (coluna `ID`) |
| `{{NOME_TESTE}}` | o nome do teste (coluna `Nome do teste (inicial)`) |
| `{{EXECUTADO_POR}}` | quem executou (coluna `Executado por`) |
| `{{STATUS}}` | o status (coluna `Status nativo`) |
| `{{DATA}}` | a data de hoje (DD/MM/AAAA) |
| `{{EVIDENCIAS}}` | este parágrafo vira **as imagens** da subpasta |

Funciona mesmo que o Word "quebre" um token internamente — o programa reconstrói e substitui certo.

## A planilha (colunas necessárias)

O programa precisa encontrar estas colunas: **`ID`**, **`Nome do teste (inicial)`**, **`Executado por`**, **`Status nativo`**.

O reconhecimento é **tolerante** a acento, maiúsculas/minúsculas, espaços e pontuação. Aceita variações como `Executado por` / `Executador por` / `Executor` / `Responsável`, e `Nome do teste` / `Nome do teste (inicial)`.

---

## Para gerar PDF (opcional): programas extras

As ações `[3]`, `[2]` e `[1]` **não precisam de nada além do `det`**. As de PDF, sim:

- **`[4] Gerar DET em PDF`** → precisa do **[LibreOffice](https://www.libreoffice.org/)** instalado. No Windows, o programa acha sozinho no caminho padrão de instalação.
- **`[5] Gerar DET compilado`** → precisa do **[qpdf](https://qpdf.sourceforge.io/)** (para juntar). O **[Ghostscript](https://www.ghostscript.com/)** é **opcional** (comprime o resultado; se não tiver, gera sem comprimir).

Não sabe o que já tem? Rode o programa e use **`[6] Verificar ambiente`** — ele lista o que está `[ ok ]` e o que está `[falta]`.

---

## Deu problema? Soluções rápidas

| O que aconteceu | O que fazer |
|---|---|
| *"det não é reconhecido"* | Instalou? Então **feche e reabra o terminal**. Se persistir, rode de novo `cargo install --path .`. |
| *"nenhuma planilha informada / não encontrada"* | Coloque o arquivo `teste_selected_....xlsx` **na raiz da pasta de trabalho**. |
| *"nenhum modelo informado"* | Você precisa de um `modelo_det.docx` dentro da pasta `modelos/`. |
| *"Selecione uma Release e um ID CARD primeiro"* | Faça a opção **[1]** antes das opções [2]–[5]. |
| *"LibreOffice não encontrado"* (ação 4) | Instale o LibreOffice. Confira depois em **[6] Verificar ambiente**. |
| *"qpdf não encontrado"* (ação 5) | Instale o qpdf. O Ghostscript é opcional. |
| Ao reinstalar: *"Acesso negado"* | Você tem um `det` aberto em outro terminal — **feche-o** (opção `0`) e rode `cargo install --path .` de novo. |

---

## Detalhes técnicos (para desenvolvedores)

Estrutura do código:

```txt
src/
├─ main.rs        # binário: laço do menu + dispatch das ações
├─ lib.rs         # expõe os módulos (permite os testes de integração)
├─ menu.rs        # desenho do menu, contexto e navegação (voltar / passos)
├─ workspace.rs   # descoberta de Release/, ID CARD, modelos/ e da planilha recente
├─ acoes.rs       # as ações (criar subpastas, gerar docx/pdf/compilado)
├─ xlsx.rs        # leitor de .xlsx (shared strings + worksheet)
├─ docx.rs        # motor de .docx (preenche tokens + insere imagens)
├─ pdf.rs         # integração com LibreOffice / qpdf / Ghostscript
└─ util.rs        # ordenação natural, normalização, datas, XML
tests/            # um arquivo de teste por funcionalidade (+ fixtures em common/)
```

Dependência única: o crate [`zip`](https://crates.io/crates/zip). O `.docx` e o `.xlsx` são manipulados diretamente como ZIP + XML.

Compilar e testar:

```bash
cargo build            # compila (debug)
cargo build --release  # compila otimizado (binário em target/release/det)
cargo test             # roda a suíte de testes
```

Notas e limitações:

- **Fuso horário**: a data usa o relógio do sistema em UTC. Para a maioria dos fusos cai no dia certo; o ajuste fica isolado em `src/util.rs` (`offset_local`).
- **Nome da release**: o formato é `Release <Mês> <Ano>` (ex.: `Release Julho 2026`). Para mudar, edite `criar_nova_release` em `src/menu.rs`.
- **Divisão do compilado**: o corte de 30 MB agrupa **arquivos inteiros** (não divide por página). Um PDF isolado maior que o limite vira uma parte própria.
- **Portabilidade**: validado com foco no Windows; como usa só `std` + o crate `zip`, roda também em Linux/macOS.

---

## Como Contribuir

## Referências Técnicas
