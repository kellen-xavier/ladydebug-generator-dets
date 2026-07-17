//! Gestor de Documentos de Testes (DET).
//!
//! Menu de terminal para gerar e organizar evidencias de teste manual:
//!   1. Criar Pasta da Release        (Release <Mes> <AAAA>)
//!   2. Criar subpastas dos testes    (ID - nome, ate 10 chars)
//!   3. Gerar DET - docx              (preenche o modelo + insere evidencias)
//!   4. Gerar DET PDF                 (docx -> pdf via LibreOffice)
//!   5. Gerar DET compilado           (junta os PDFs, limite 30 MB)
//!
//! Area de trabalho (por padrao, a pasta atual) deve conter:
//!   modelos/                 -> modelo_det.docx (com os tokens {{...}})
//!   Release/                 -> pasta central das releases
//!   teste_selected_*.xlsx    -> planilha exportada (a mais recente e usada)

mod acoes;
mod docx;
mod menu;
mod pdf;
mod util;
mod workspace;
mod xlsx;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use util::*;
use workspace::Workspace;

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

    loop {
        match menu::selecionar_acao(&ws) {
            0 => {
                log("Ate mais.");
                return ExitCode::SUCCESS;
            }
            1 => executar_1(&ws),
            2 => executar_2(&ws),
            3 => executar_3(&ws),
            4 => executar_4(&ws),
            5 => executar_5(&ws),
            _ => {}
        }
    }
}

fn ajuda(prog: &str) {
    println!(
        "Gestor de Documentos de Testes (DET)\n\n\
         Uso:\n  {prog} [--base <pasta_de_trabalho>]\n\n\
         Sem argumentos, usa a pasta atual como area de trabalho.\n\
         Abra o terminal na sua pasta de trabalho e rode o comando para ver o menu.\n\n\
         Estrutura esperada da area de trabalho:\n\
           modelos/                modelo_det.docx (com tokens {{{{ID}}}}, {{{{EVIDENCIAS}}}}, ...)\n\
           Release/                pasta central das releases\n\
           teste_selected_*.xlsx   planilha exportada (a mais recente e usada)"
    );
}

// ─── Acao 1 ────────────────────────────────────────────────────────────────

fn executar_1(ws: &Workspace) {
    match acoes::criar_release(ws) {
        Ok(_) => {}
        Err(e) => erro(&e),
    }
}

// ─── Acao 2 ────────────────────────────────────────────────────────────────

fn executar_2(ws: &Workspace) {
    let release = match menu::escolher_release(ws) {
        Some(r) => r,
        None => return,
    };
    let planilha = match resolver_planilha(ws) {
        Some(p) => p,
        None => return,
    };
    match acoes::criar_subpastas(&release, &planilha) {
        Ok(n) => log(&format!("Concluido: {n} subpasta(s) criada(s).")),
        Err(e) => erro(&e),
    }
}

// ─── Acao 3 ────────────────────────────────────────────────────────────────

fn executar_3(ws: &Workspace) {
    let release = match menu::escolher_release(ws) {
        Some(r) => r,
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
    match acoes::gerar_docx(&release, &planilha, &modelo, legendas) {
        Ok(r) => log(&format!(
            "Concluido: {} gerado(s), {} ignorado(s), {} com aviso.",
            r.gerados, r.ignorados, r.avisos
        )),
        Err(e) => erro(&e),
    }
}

// ─── Acao 4 ────────────────────────────────────────────────────────────────

fn executar_4(ws: &Workspace) {
    let release = match menu::escolher_release(ws) {
        Some(r) => r,
        None => return,
    };
    match acoes::gerar_pdf(&release) {
        Ok(n) => log(&format!("Concluido: {n} PDF(s) gerado(s).")),
        Err(e) => erro(&e),
    }
}

// ─── Acao 5 ────────────────────────────────────────────────────────────────

fn executar_5(ws: &Workspace) {
    let release = match menu::escolher_release(ws) {
        Some(r) => r,
        None => return,
    };
    let limite = menu::perguntar_com_default("  Limite por arquivo (MB)", "30");
    let limite_mb: u64 = limite.parse().unwrap_or(30);
    match acoes::gerar_compilado(&release, limite_mb) {
        Ok(v) => log(&format!("Concluido: {} arquivo(s) gerado(s).", v.len())),
        Err(e) => erro(&e),
    }
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

// ─── Utilitario de argumentos ──────────────────────────────────────────────

fn arg_valor(args: &[String], flag: &str) -> Option<String> {
    let i = args.iter().position(|a| a == flag)?;
    args.get(i + 1).cloned()
}
