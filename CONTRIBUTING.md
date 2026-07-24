# Contribuindo com o Gerador de DETs

Obrigado pelo interesse em contribuir! 🎉 Este é um projeto **open-source** (licença [MIT](LICENSE)) escrito em **Rust**: uma ferramenta de terminal (TUI) que organiza evidências de testes manuais e gera os DETs (Documento de Evidência de Testes).

Leia este guia junto com:

- [`README.md`](README.md) — como **usar** a ferramenta (passo a passo do usuário).
- [`AGENTS.md`](AGENTS.md) — **arquitetura**, módulos e regras do projeto junto com IA de sua preferência.

## Sumário

- [Visão geral em 1 minuto](#visão-geral-em-1-minuto)
- [Preparar o ambiente de desenvolvimento](#preparar-o-ambiente-de-desenvolvimento)
- [Fluxo de trabalho com Git](#fluxo-de-trabalho-com-git)
- [Padrão de commits](#padrão-de-commits)
- [Pull Requests](#pull-requests)
- [Padrões de código](#padrões-de-código)
- [Testes e verificação](#testes-e-verificação)
- [Empacotar e distribuir](#empacotar-e-distribuir)
- [Documentação](#documentação)
- [Manutenibilidade](#manutenibilidade)
- [Reportar bugs e sugerir melhorias](#reportar-bugs-e-sugerir-melhorias)
- [O que nunca fazer](#o-que-nunca-fazer)

---

## Visão geral em 1 minuto

O projeto é um **binário** (`gerador-dets`) apoiado numa **biblioteca** (`gestor_det`) — a lib existe para deixar a lógica **testável**. **Dependência única de crate: [`zip`](https://crates.io/crates/zip)**; `.docx` e `.xlsx` são manipulados como ZIP + XML na mão.

| Camada | Arquivo | Papel |
| --- | --- | --- |
| Binário | `src/main.rs` | Laço do menu, contexto (Release + ID CARD), dispatch, check de ambiente |
| Lib | `src/lib.rs` | Expõe os módulos para o binário **e** para os testes |
| Interface | `src/menu.rs` | Desenho do menu, navegação (passos, "voltar"), prompts |
| Descoberta | `src/workspace.rs` | Localiza `Release/`, `ID CARD`, `modelos/` e a planilha recente |
| Ações | `src/acoes.rs` | Criar subpastas, gerar docx, gerar pdf, gerar compilado |
| Formatos | `src/xlsx.rs`, `src/docx.rs` | Leitor de `.xlsx`; motor de `.docx` (tokens + imagens) |
| Externos | `src/pdf.rs` | LibreOffice / qpdf / Ghostscript |
| Utilidades | `src/util.rs` | Ordenação natural, normalização, datas, XML |

**Princípio central:** a UI (`menu.rs`) e as ferramentas externas (`pdf.rs`) **não guardam regra de negócio** — a lógica vive nos módulos de domínio (`acoes`, `docx`, `xlsx`, `util`) e é coberta por `tests/`.

---

## Preparar o ambiente de desenvolvimento

1. **Rust** (edition 2021). Instale via [rustup](https://rustup.rs) — ele traz o `cargo`.

2. **(Opcional, só para as ações de PDF)** [LibreOffice](https://www.libreoffice.org/) (`soffice`), [qpdf](https://qpdf.sourceforge.io/) e [Ghostscript](https://www.ghostscript.com/) (`gs`). A geração de `.docx` **não** depende de nada externo.

3. **Clonar e rodar:**

   ```bash
   git clone https://github.com/kellen-xavier/ladydebug-generator-dets
   cd ladydebug-generator-dets

   cargo run              # abre o menu (pasta atual como area de trabalho; use --base <dir>)
   cargo build --release  # binario otimizado em target/release/gerador-dets
   cargo test             # roda a suite de testes
   ```

> 💡 Para experimentar, use uma **pasta de teste descartável** (ou `cargo run -- --base <pasta_temp>`) — **nunca** rode experimentos sobre evidências reais.

---

## Fluxo de trabalho com Git

O projeto usa um **Git Flow simplificado**. Contribuições externas entram por **fork + Pull Request**.

| Branch | Para quê |
| --- | --- |
| `main` | Estável / pronto para uso. Não recebe commit direto. |
| `develop` | Integração do trabalho em andamento (**base das contribuições**). |
| `feature/<nome-curto>` | Uma funcionalidade/melhoria. Sai de `develop`. |
| `fix/<nome-curto>` | Correção de bug. Sai de `develop`. |
| `docs/<nome-curto>` | Só documentação. |

**Passo a passo (contribuidor externo):**

```bash
# 1. Faça um fork no GitHub, depois clone o SEU fork
git clone https://github.com/<seu-usuario>/ladydebug-generator-dets
cd ladydebug-generator-dets
git remote add upstream https://github.com/kellen-xavier/ladydebug-generator-dets

# 2. Crie a branch a partir de develop
git fetch upstream
git checkout -b feature/minha-melhoria upstream/develop

# 3. ... faça as alterações e commits ...
git push -u origin feature/minha-melhoria

# 4. Abra o Pull Request do seu fork para develop
```

Regras:

- **Uma branch por assunto** — não misture uma correção com uma nova feature.
- Mantenha a branch **atualizada** com `develop` (`git merge upstream/develop` ou `rebase`).
- **Nunca** versione evidências nem saídas: `Release/`, `dist/`, `target/`, `DET_*` e `teste_selected_*.xlsx` já estão no `.gitignore`.

---

## Padrão de commits

Usamos **[Conventional Commits](https://www.conventionalcommits.org/)** (o histórico já segue isso):

```txt
<tipo>: <resumo no imperativo, minúsculo>
```

| Tipo | Quando usar |
| --- | --- |
| `feat` | nova funcionalidade |
| `fix` | correção de bug |
| `docs` | só documentação |
| `refactor` | mudança de código sem alterar comportamento |
| `test` | testes |
| `chore` | build, dependências, `.gitignore`, empacotamento |

Exemplos:

```txt
feat: adiciona opcao de gerar DET so de um ID no menu
fix: corrige ordenacao natural de imagens com Screenshot_10
test: cobre o caminho de compilado sem qpdf
docs: atualiza README com o passo a passo da TUI
```

Commits **pequenos e coesos**, um assunto por commit, mensagem que explica **o quê/por quê**.

---

## Pull Requests

- Abra o PR **para `develop`** (não para `main`).
- Descreva **o que muda e por quê**; referencie a issue relacionada, se houver.
- **Checklist antes de pedir revisão:**
  - [ ] `cargo build` e `cargo test` passam.
  - [ ] `cargo fmt --all` aplicado e `cargo clippy --all-targets` sem novos avisos.
  - [ ] Sem IDs, caminhos ou nomes de release **hardcoded**.
  - [ ] Comportamento novo/alterado **coberto por teste** em `tests/`.
  - [ ] `README.md` / `AGENTS.md` atualizados se o comportamento mudou.
  - [ ] Nada de evidências, saídas (`dist/`, `target/`) ou arquivos grandes no diff.
- Prefira **squash** ao mergear, mantendo o histórico limpo.

---

## Padrões de código

- **Formatação:** `cargo fmt --all` (rustfmt padrão). **Lints:** `cargo clippy --all-targets` — resolva os avisos.
- **Idioma:** mensagens ao usuário, nomes de função/variável e comentários em **português**. Ao editar um arquivo, **siga a convenção que já existe nele**.
- **Dependências:** o projeto tem **uma única** dependência (`zip`) de propósito. Parsing de `.docx`/`.xlsx` continua **manual** (ZIP+XML). **Não adicione crates** sem uma justificativa forte no PR.
- **Funções pequenas e testáveis** — uma responsabilidade cada; extraia helpers em vez de inflar uma função.
- **I/O seguro:** nunca sobrescrever evidência existente; proteger contra *zip-slip* ao extrair; **degradar com clareza** quando faltar ferramenta externa (avisar, nunca estourar *panic*/traceback — veja a opção 6 do menu).

**Regra de ouro (também no [`AGENTS.md`](AGENTS.md)):** IDs, release e caminhos vêm **por parâmetro, pela planilha ou pela seleção no menu** — **nunca** hardcoded.

---

## Testes e verificação

O projeto tem uma suíte de testes de integração em **`tests/`**, com **um arquivo por funcionalidade** (`organizar`, `gerar_docx`, `gerar_pdf`, `gerar_compilado`, `leitura_planilha`, `docx_engine`, `util`, `ambiente`) e as *fixtures* em `tests/common/mod.rs`.

- Os testes são **determinísticos** e **não dependem** de LibreOffice/qpdf — as ações externas são cobertas pelos caminhos de erro e por lógica pura (`bin_pack`). Assim `cargo test` roda em qualquer máquina.
- **Como adicionar um teste:** use `common::TempWs` (área de trabalho temporária, removida no `Drop`) e os helpers `escrever_xlsx` / `escrever_template` / `png_falso` (geram `.xlsx`/`.docx`/PNG em memória). Coloque o teste no arquivo da funcionalidade correspondente.

**Verificação rápida antes do PR:**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

> Ao adicionar uma função de lógica pura, deixe-a **fácil de testar** (entrada → saída, sem efeitos colaterais escondidos).

---

## Empacotar e distribuir

- **Binário nativo:** `cargo build --release` → `target/release/gerador-dets` (`.exe` no Windows), autônomo, sem runtime.
- **Pacote para usuários finais:** rode o `empacotar.ps1` — ele compila e monta `dist/gerador-dets.zip` com o `.exe`, o modelo, o launcher (`ABRIR MENU.bat` + `menu.ps1`) e o `COMO USAR.txt`:

  ```powershell
  powershell -ExecutionPolicy Bypass -File .\empacotar.ps1
  ```

O `dist/` é ignorado pelo Git (é artefato de build).

---

## Documentação

Mantenha em sincronia quando o comportamento mudar:

- [`AGENTS.md`](AGENTS.md) — **arquitetura e regras**. Mude aqui primeiro ao alterar uma regra.
- [`README.md`](README.md) — guia do usuário (a TUI, passo a passo).
- Comentários e *doc-comments* (`///`) nos módulos.

Uma mudança de comportamento **sem** atualização de doc é considerada **incompleta**.

---

## Manutenibilidade

Práticas para o projeto envelhecer bem:

- **Fonte única de verdade.** Evite duplicar lógica; a regra vive no domínio (`acoes`/`docx`/`xlsx`/`util`), a UI só chama.
- **Uma dependência.** Pense duas vezes antes de adicionar um crate — grande parte do valor do projeto é ser leve e sem runtime.
- **Degradação elegante.** Recursos que dependem de ferramentas externas detectam a ausência e **avisam** (veja a opção 6, "Verificar ambiente").
- **Estrutura como contrato.** O padrão `Release/Release <Mês> <Ano>/ID CARD <n>/<ID> - <nome>/` e os **tokens** do modelo (`{{ID}}`, `{{EVIDENCIAS}}`, ...) são contrato: mudá-los exige atualizar **código, docs e testes juntos**.
- **Compatibilidade Windows.** Cuidado com caminhos com espaços/acentos; a saída do programa é mantida em ASCII de propósito.

---

## Reportar bugs e sugerir melhorias

Abra uma **issue** no GitHub descrevendo:

- o que você esperava e o que aconteceu;
- **passos para reproduzir** (e a planilha/modelo mínimos, se aplicável);
- sistema operacional e versão do Rust (`rustc --version`);
- se for sobre PDF, a saída da opção **[6] Verificar ambiente**.

Sugestões de melhoria também são bem-vindas como issue antes de abrir um PR grande — assim alinhamos a direção primeiro.

---

## O que nunca fazer

- ❌ Commitar direto em `main` ou `develop` (use branch + PR).
- ❌ Versionar evidências, saídas (`dist/`, `target/`) ou arquivos grandes.
- ❌ Colocar IDs, cards ou caminhos fixos no código.
- ❌ Quebrar os *tokens* do modelo (`{{...}}`) ou o contrato de pastas sem atualizar código, docs e testes.
- ❌ Adicionar dependências pesadas sem necessidade real.
- ❌ Mudar comportamento sem atualizar a documentação e os testes.
