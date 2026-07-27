//! Interface de terminal: menu com contexto (Release + ID CARD), navegacao com
//! "Voltar ao menu anterior/principal" e prompts. Selecao por numero, sem deps.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::util::{hoje_ymd, log, nome_mes, ok};
use crate::workspace::{nome_de, Workspace};

/// Resultado de um sub-menu de selecao.
pub enum Nav<T> {
    Escolha(T),
    Anterior,  // voltar ao menu anterior
    Principal, // voltar ao menu principal
}

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
    perguntar_sn_padrao(prompt, false)
}

/// Pergunta sim/nao com default configuravel.
pub fn perguntar_sn_padrao(prompt: &str, default_sim: bool) -> bool {
    let dica = if default_sim { "(S/n)" } else { "(s/N)" };
    print!("{prompt} {dica}: ");
    let _ = io::stdout().flush();
    let r = ler_linha().trim().to_ascii_lowercase();
    if r.is_empty() {
        default_sim
    } else {
        matches!(r.as_str(), "s" | "sim" | "y")
    }
}

/// "Enter para voltar ao menu" — forma facil de retornar apos uma acao.
pub fn pausar() {
    print!("\n  [Enter] para voltar ao menu... ");
    let _ = io::stdout().flush();
    let _ = ler_linha();
}

fn ctx_txt(p: Option<&Path>, vazio: &str) -> String {
    match p {
        Some(p) => nome_de(p).to_string(),
        None => vazio.to_string(),
    }
}

/// Desenha o menu principal (com o contexto atual) e devolve a acao (0..=6).
pub fn selecionar_acao(ws: &Workspace, release: Option<&Path>, card: Option<&Path>) -> u8 {
    let barra = "=".repeat(58);
    let linha = "-".repeat(52);
    println!("\n{barra}");
    println!("  Gestor de Documentos de Testes (DET)");
    println!("  Pasta de trabalho: {}", ws.raiz.display());
    println!("{barra}");
    println!("  Release atual: {}", ctx_txt(release, "(nenhuma selecionada)"));
    println!("  Card atual:    {}", ctx_txt(card, "(nenhum selecionado)"));
    println!("  {linha}");
    let releases = ws.listar_releases();
    if releases.is_empty() {
        println!("  Ainda nao ha Release nesta pasta -- use [1] para criar uma.");
    } else {
        println!("  Releases nesta pasta ([*] = a que voce esta usando):");
        for r in &releases {
            let marca = if Some(r.as_path()) == release { "*" } else { " " };
            println!("     [{marca}] {}", nome_de(r));
        }
    }
    println!("{barra}");
    let dica = if release.is_none() { "   <- comece por aqui" } else { "" };
    println!("  -- Organizar {}", "-".repeat(40));
    println!("   [1] Selecionar / criar Release e ID CARD{dica}");
    println!("   [2] Criar subpastas dos testes  (ID - nome)");
    println!("  -- Gerar {}", "-".repeat(44));
    println!("   [3] Gerar DET (.docx)");
    println!("   [4] Gerar DET em PDF");
    println!("   [5] Gerar DET compilado (PDF unico)");
    println!("  {linha}");
    println!("   [6] Verificar ambiente");
    println!("   [0] Sair");
    println!("{barra}");
    loop {
        print!("  Selecione uma opcao [0-6]: ");
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
            "6" => return 6,
            outro => println!("  Opcao invalida: '{outro}'. Digite um numero de 0 a 6."),
        }
    }
}

/// Passo 1a: escolher/criar uma Release. Aqui, "Voltar" = menu principal.
pub fn selecionar_release(ws: &Workspace) -> Nav<PathBuf> {
    loop {
        let releases = ws.listar_releases();
        let barra = "-".repeat(52);
        println!("\n{barra}");
        println!("  Passo 1 de 2  -  Selecione a Release (Mes Ano)");
        println!("{barra}");
        if releases.is_empty() {
            println!("   (nenhuma release ainda -- crie uma com a opcao N)");
        }
        for (i, r) in releases.iter().enumerate() {
            println!("   [{}] {}", i + 1, nome_de(r));
        }
        println!("   [N] Criar nova Release (escolhe mes e ano)...");
        println!("   [0] Voltar ao menu principal");
        let escolha = perguntar_com_default("  Opcao", "").to_ascii_lowercase();
        match escolha.as_str() {
            "" | "0" => return Nav::Principal,
            "n" => {
                if let Some(r) = criar_nova_release(ws) {
                    return Nav::Escolha(r);
                }
            }
            outro => {
                if let Ok(idx) = outro.parse::<usize>() {
                    if idx >= 1 && idx <= releases.len() {
                        return Nav::Escolha(releases[idx - 1].clone());
                    }
                }
                println!("  Opcao invalida.");
            }
        }
    }
}

/// Passo 1b: escolher/criar o ID CARD dentro da release.
/// "Voltar ao menu anterior" = trocar release; "Voltar ao menu principal".
pub fn selecionar_card(ws: &Workspace, release: &Path) -> Nav<PathBuf> {
    loop {
        let cards = ws.listar_cards(release);
        let barra = "-".repeat(52);
        println!("\n{barra}");
        println!("  Passo 2 de 2  -  Selecione o ID CARD");
        println!("  Release: {}", nome_de(release));
        println!("{barra}");
        if cards.is_empty() {
            println!("   (nenhum card nesta release ainda)");
        } else {
            println!("  Cards ja nesta release:");
            for c in &cards {
                println!("    - {}", nome_de(c));
            }
        }
        println!("  Informe o numero do ID CARD, ou:");
        println!("   [A] Voltar ao menu anterior (trocar release)");
        println!("   [P] Voltar ao menu principal");
        let resp = perguntar_com_default("  ID CARD", "");
        match resp.to_ascii_lowercase().as_str() {
            "" => println!("  Informe um numero ou A / P."),
            "a" => return Nav::Anterior,
            "p" | "0" => return Nav::Principal,
            _ => {
                if !resp.chars().all(|c| c.is_ascii_digit()) {
                    println!("  O ID CARD deve ser numerico (ex.: 1399338).");
                    continue;
                }
                let card = release.join(ws.nome_card(&resp));
                log(&format!("Conferindo \"{}\" em {}...", nome_de(&card), nome_de(release)));
                if card.is_dir() {
                    ok("Pasta encontrada.");
                    return Nav::Escolha(card);
                }
                println!("  Essa pasta nao existe.");
                if perguntar_sn_padrao("  Deseja criar?", true) {
                    match fs::create_dir_all(&card) {
                        Ok(_) => {
                            ok(&format!("Pasta criada: {}", nome_de(&card)));
                            return Nav::Escolha(card);
                        }
                        Err(e) => println!("  Nao criei a pasta: {e}"),
                    }
                }
            }
        }
    }
}

// ─── Desempate de subpastas com o mesmo prefixo de ID ──────────────────────

/// Pergunta qual subpasta usar quando mais de uma tem o mesmo prefixo de ID.
/// Devolve `None` para pular o ID (opcao 0 ou entrada invalida/EOF).
///
/// Ligado em `main.rs` como callback de `acoes::gerar_docx`.
pub fn escolher_subpasta(id: &str, nome: &str, candidatas: &[PathBuf]) -> Option<PathBuf> {
    let barra = "-".repeat(52);
    println!("\n{barra}");
    println!("  Atencao: mais de uma subpasta para o ID {id} ('{nome}')");
    println!("{barra}");
    println!("  Isso costuma acontecer quando o nome do teste mudou na planilha");
    println!("  e uma segunda pasta foi criada. Escolha onde gravar o DET:");
    for (i, c) in candidatas.iter().enumerate() {
        println!("   [{}] {}", i + 1, nome_de(c));
    }
    println!("   [0] Pular este ID (nao gerar DET agora)");
    let resp = perguntar_com_default("  Opcao", "0");
    match resp.trim().parse::<usize>() {
        Ok(i) if i >= 1 && i <= candidatas.len() => Some(candidatas[i - 1].clone()),
        _ => None,
    }
}

// ─── Regras puras de criacao de release (testaveis) ────────────────────────

/// Faixa de anos aceita para uma release.
pub const ANO_MIN: i64 = 2000;
pub const ANO_MAX: i64 = 2100;

/// Valida o ano digitado. Aceita so digitos, dentro de ANO_MIN..=ANO_MAX.
pub fn validar_ano(s: &str) -> Option<i64> {
    let t = s.trim();
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let ano: i64 = t.parse().ok()?;
    if (ANO_MIN..=ANO_MAX).contains(&ano) {
        Some(ano)
    } else {
        None
    }
}

/// Valida o mes digitado (1..=12).
pub fn validar_mes(s: &str) -> Option<u32> {
    match s.trim().parse::<u32>() {
        Ok(v) if (1..=12).contains(&v) => Some(v),
        _ => None,
    }
}

/// Nome padronizado da pasta de release: "Release <Mes> <Ano>".
pub fn montar_nome_release(mes: u32, ano: i64) -> String {
    format!("Release {} {}", nome_mes(mes), ano)
}

/// Cria uma nova pasta de release `Release <Mes> <Ano>` a partir de mes/ano.
/// Mes e ano sugeridos = os de hoje (no fuso configurado).
fn criar_nova_release(ws: &Workspace) -> Option<PathBuf> {
    let (ano_hoje, mes_hoje, _) = hoje_ymd();

    println!("\n  Meses:");
    for m in 1..=12u32 {
        print!("   [{m:>2}] {:<10}", nome_mes(m));
        if m % 3 == 0 {
            println!();
        }
    }
    let mnum = perguntar_com_default("  Numero do mes (1-12)", &mes_hoje.to_string());
    let m: u32 = match validar_mes(&mnum) {
        Some(v) => v,
        None => {
            println!("  Mes invalido: informe um numero de 1 a 12.");
            return None;
        }
    };
    let resp_ano = perguntar_com_default("  Ano", &ano_hoje.to_string());
    let ano = match validar_ano(&resp_ano) {
        Some(a) => a,
        None => {
            println!("  Ano invalido: informe um ano entre {ANO_MIN} e {ANO_MAX}.");
            return None;
        }
    };
    let nome = montar_nome_release(m, ano);
    let destino = ws.dir_release.join(&nome);
    if destino.is_dir() {
        log(&format!("Release ja existe: {nome}"));
    } else if let Err(e) = fs::create_dir_all(&destino) {
        println!("  Nao criei a release: {e}");
        return None;
    } else {
        ok(&format!("Release criada: {nome}"));
    }
    Some(destino)
}
