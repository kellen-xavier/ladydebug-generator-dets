//! Gestor de Documentos de Testes (DET).
//!
//! Menu de terminal para gerar e organizar evidencias de teste manual, com
//! contexto (Release + ID CARD) e navegacao guiada:
//!   1. Selecionar Release e ID CARD  (cria se nao existir)
//!   2. Criar subpastas dos testes    (ID - nome, dentro do card)
//!   3. Gerar DET - docx              (preenche o modelo + insere evidencias)
//!   4. Gerar DET PDF                 (docx -> pdf via LibreOffice)
//!   5. Gerar DET compilado           (junta os PDFs, limite 30 MB)
//!   6. Verificar ambiente            (ferramentas externas de PDF)
//!
//! Estrutura da area de trabalho:
//!   modelos/                 -> modelo_det.docx (com os tokens {{...}})
//!   Release/                 -> Release <Mes> <AAAA>/ ID CARD <n>/ <ID> - <nome>/
//!   teste_selected_*.xlsx    -> planilha exportada (a mais recente e usada)

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use gestor_det::acoes;
use gestor_det::menu::{self, Nav};
use gestor_det::pdf;
use gestor_det::util::*;
use gestor_det::workspace::{nome_de, Workspace};

/// Contexto de trabalho selecionado na opcao 1.
#[derive(Default)]
struct Contexto {
    release: Option<PathBuf>,
    card: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        ajuda(args.first().map(String::as_str).unwrap_or("det"));
        return ExitCode::SUCCESS;
    }

    // Base opcional via `--base <dir>`; padrao = diretorio atual.
    let base = match arg_valor(&args, "--base") {
        Some(v) => PathBuf::from(v),
        None => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };
    let ws = Workspace::detectar(&base);

    // Primeiro contato: mostrar se o ambiente esta ok, com retorno facil.
    verificar_ambiente();
    menu::pausar();

    let mut ctx = Contexto::default();
    loop {
        match menu::selecionar_acao(&ws, ctx.release.as_deref(), ctx.card.as_deref()) {
            0 => {
                log("Ate mais.");
                return ExitCode::SUCCESS;
            }
            1 => {
                if executar_1(&ws, &mut ctx) {
                    menu::pausar();
                }
            }
            2 => {
                executar_2(&ws, &ctx);
                menu::pausar();
            }
            3 => {
                executar_3(&ws, &ctx);
                menu::pausar();
            }
            4 => {
                executar_4(&ctx);
                menu::pausar();
            }
            5 => {
                executar_5(&ctx);
                menu::pausar();
            }
            6 => {
                verificar_ambiente();
                menu::pausar();
            }
            _ => {}
        }
    }
}

// ─── Acao 1: selecionar Release + ID CARD (define o contexto) ───────────────

/// Devolve `true` se o contexto (release + card) foi definido.
fn executar_1(ws: &Workspace, ctx: &mut Contexto) -> bool {
    loop {
        let release = match menu::selecionar_release(ws) {
            Nav::Escolha(r) => r,
            Nav::Anterior | Nav::Principal => return false,
        };
        match menu::selecionar_card(ws, &release) {
            Nav::Anterior => continue, // volta a escolher a release
            Nav::Principal => return false,
            Nav::Escolha(card) => {
                log(&format!("Contexto: {} | {}", nome_de(&release), nome_de(&card)));
                ctx.release = Some(release);
                ctx.card = Some(card);
                return true;
            }
        }
    }
}

// ─── Acoes 2..5: exigem um card selecionado ─────────────────────────────────

fn exigir_card(ctx: &Contexto) -> Option<&Path> {
    match ctx.card.as_deref() {
        Some(c) => Some(c),
        None => {
            erro("Selecione uma Release e um ID CARD primeiro (opcao 1).");
            None
        }
    }
}

fn executar_2(ws: &Workspace, ctx: &Contexto) {
    let card = match exigir_card(ctx) {
        Some(c) => c,
        None => return,
    };
    let planilha = match resolver_planilha(ws) {
        Some(p) => p,
        None => return,
    };
    match acoes::criar_subpastas(card, &planilha) {
        Ok(n) => log(&format!("Concluido: {n} subpasta(s) criada(s).")),
        Err(e) => erro(&e),
    }
}

fn executar_3(ws: &Workspace, ctx: &Contexto) {
    let card = match exigir_card(ctx) {
        Some(c) => c,
        None => return,
    };
    let planilha = match resolver_planilha(ws) {
        Some(p) => p,
        None => return,
    };
    let modelo = match resolver_modelo(ws) {
        Some(m) => m,
        None => return,
    };
    let legendas = menu::perguntar_sn("  Adicionar legenda (nome do arquivo) sob cada imagem?");
    match acoes::gerar_docx(card, &planilha, &modelo, legendas) {
        Ok(r) => log(&format!(
            "Concluido: {} gerado(s), {} ignorado(s), {} com aviso.",
            r.gerados, r.ignorados, r.avisos
        )),
        Err(e) => erro(&e),
    }
}

fn executar_4(ctx: &Contexto) {
    let card = match exigir_card(ctx) {
        Some(c) => c,
        None => return,
    };
    match acoes::gerar_pdf(card) {
        Ok(n) => log(&format!("Concluido: {n} PDF(s) gerado(s).")),
        Err(e) => erro(&e),
    }
}

fn executar_5(ctx: &Contexto) {
    let card = match exigir_card(ctx) {
        Some(c) => c,
        None => return,
    };
    let limite = menu::perguntar_com_default("  Limite por arquivo (MB)", "30");
    let limite_mb: u64 = limite.parse().unwrap_or(30);
    match acoes::gerar_compilado(card, limite_mb) {
        Ok(v) => log(&format!("Concluido: {} arquivo(s) gerado(s).", v.len())),
        Err(e) => erro(&e),
    }
}

// ─── Ambiente ───────────────────────────────────────────────────────────────

fn verificar_ambiente() {
    let barra = "=".repeat(58);
    println!("\n{barra}");
    println!("  Ambiente");
    println!("{barra}");
    status("LibreOffice (soffice)", pdf::soffice_bin().is_some(), true, "necessario p/ [4] Gerar DET PDF");
    status("qpdf", pdf::find_in_path(&["qpdf"]).is_some(), true, "necessario p/ [5] Gerar DET compilado");
    status(
        "Ghostscript (gs)",
        pdf::find_in_path(&["gs", "gswin64c", "gswin32c"]).is_some(),
        false,
        "opcional: comprime o compilado",
    );
    println!("{barra}");
    println!("  Gerar .docx (acoes 1-3) nao depende de nada externo.");
}

fn status(nome: &str, presente: bool, obrigatorio: bool, obs: &str) {
    let marca = if presente {
        "[ ok  ]"
    } else if obrigatorio {
        "[falta]"
    } else {
        "[ --  ]"
    };
    println!("  {marca} {nome:<22} {obs}");
}

// ─── Resolucao de planilha e modelo (com confirmacao) ──────────────────────

fn resolver_planilha(ws: &Workspace) -> Option<PathBuf> {
    let sugestao = ws
        .achar_planilha_recente()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let resp = menu::perguntar_com_default("  Planilha .xlsx", &sugestao);
    if resp.is_empty() {
        erro("nenhuma planilha informada.");
        return None;
    }
    let p = PathBuf::from(&resp);
    if !p.is_file() {
        erro(&format!("planilha nao encontrada: {}", p.display()));
        return None;
    }
    Some(p)
}

fn resolver_modelo(ws: &Workspace) -> Option<PathBuf> {
    let sugestao = ws
        .achar_modelo()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let resp = menu::perguntar_com_default("  Modelo .docx", &sugestao);
    if resp.is_empty() {
        erro("nenhum modelo informado (coloque um .docx em modelos/).");
        return None;
    }
    let p = PathBuf::from(&resp);
    if !p.is_file() {
        erro(&format!("modelo nao encontrado: {}", p.display()));
        return None;
    }
    Some(p)
}

// ─── Ajuda e utilitario de argumentos ──────────────────────────────────────

fn ajuda(prog: &str) {
    println!(
        "Gestor de Documentos de Testes (DET)\n\n\
         Uso:\n  {prog} [--base <pasta_de_trabalho>]\n\n\
         Sem argumentos, usa a pasta atual como area de trabalho.\n\
         Abra o terminal na sua pasta de trabalho e rode o comando para ver o menu.\n\n\
         Estrutura esperada da area de trabalho:\n\
           modelos/                modelo_det.docx (com tokens {{{{ID}}}}, {{{{EVIDENCIAS}}}}, ...)\n\
           Release/                Release <Mes> <AAAA>/ ID CARD <n>/ <ID> - <nome>/\n\
           teste_selected_*.xlsx   planilha exportada (a mais recente e usada)"
    );
}

fn arg_valor(args: &[String], flag: &str) -> Option<String> {
    let i = args.iter().position(|a| a == flag)?;
    args.get(i + 1).cloned()
}
