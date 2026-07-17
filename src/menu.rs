//! Interface de terminal: desenha o menu de acoes padronizadas e le a escolha.
//! Selecao por numero (sem modo raw) — robusto em Windows e Linux, sem deps.

use std::io::{self, Write};
use std::path::PathBuf;

use crate::util::log;
use crate::workspace::{nome_de, Workspace};

/// Le uma linha do stdin (sem a quebra). Vazio se EOF.
pub fn ler_linha() -> String {
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(0) => String::new(),
        _ => buf.trim_end_matches(['\r', '\n']).to_string(),
    }
}

/// Mostra `prompt [default]:` e devolve a resposta (default se vazio).
pub fn perguntar_com_default(prompt: &str, default: &str) -> String {
    if default.is_empty() {
        print!("{prompt}: ");
    } else {
        print!("{prompt} [{default}]: ");
    }
    let _ = io::stdout().flush();
    let r = ler_linha();
    if r.trim().is_empty() {
        default.to_string()
    } else {
        r.trim().to_string()
    }
}

/// Pergunta sim/nao (default = nao).
pub fn perguntar_sn(prompt: &str) -> bool {
    print!("{prompt} (s/N): ");
    let _ = io::stdout().flush();
    matches!(ler_linha().trim().to_ascii_lowercase().as_str(), "s" | "sim" | "y")
}

/// Desenha o menu principal e devolve a acao escolhida (0..=5).
pub fn selecionar_acao(ws: &Workspace) -> u8 {
    let barra = "=".repeat(58);
    let linha = "-".repeat(52);
    println!("\n{barra}");
    println!("  Gestor de Documentos de Testes (DET)");
    println!("  Area de trabalho: {}", ws.raiz.display());
    println!("{barra}");
    println!("  -- Organizar {}", "-".repeat(40));
    println!("   [1] Criar Pasta da Release  ({})", ws.nome_release_atual());
    println!("   [2] Criar subpastas dos testes  (ID - nome)");
    println!("  -- Gerar {}", "-".repeat(44));
    println!("   [3] Gerar DET - docx");
    println!("   [4] Gerar DET PDF");
    println!("   [5] Gerar DET compilado");
    println!("  {linha}");
    println!("   [0] Sair");
    println!("{barra}");
    loop {
        print!("  Selecione a acao [0-5]: ");
        let _ = io::stdout().flush();
        let r = ler_linha();
        if r.is_empty() {
            // EOF (stdin fechado): encerra com seguranca.
            return 0;
        }
        match r.trim() {
            "0" => return 0,
            "1" => return 1,
            "2" => return 2,
            "3" => return 3,
            "4" => return 4,
            "5" => return 5,
            outro => println!("  Opcao invalida: '{outro}'. Digite um numero de 0 a 5."),
        }
    }
}

/// Escolhe uma pasta de Release. Se so houver uma, usa direto; se nao houver,
/// devolve None (o chamador orienta a criar). Havendo varias, pergunta.
pub fn escolher_release(ws: &Workspace) -> Option<PathBuf> {
    let releases = ws.listar_releases();
    match releases.len() {
        0 => {
            log("Nenhuma pasta de Release encontrada. Use a opcao [1] primeiro.");
            None
        }
        1 => {
            log(&format!("Release: {}", nome_de(&releases[0])));
            Some(releases[0].clone())
        }
        _ => {
            println!("  Releases disponiveis:");
            for (i, r) in releases.iter().enumerate() {
                println!("   [{}] {}", i + 1, nome_de(r));
            }
            let escolha = perguntar_com_default("  Numero da release", "1");
            let idx: usize = escolha.parse().unwrap_or(1);
            releases.get(idx.saturating_sub(1)).cloned()
        }
    }
}
