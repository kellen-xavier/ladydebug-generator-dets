//! Ponte local entre Flutter e o motor Rust. A última linha contém JSON.
use gestor_det::{
    acoes, menu, pdf, util,
    workspace::{nome_de, Workspace},
    xlsx::{read_xlsx, HeaderMap},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn json(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn nomes(v: &[PathBuf]) -> String {
    format!(
        "[{}]",
        v.iter()
            .map(|p| json(nome_de(p)))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn arg<'a>(a: &'a [String], n: usize, nome: &str) -> Result<&'a str, String> {
    a.get(n)
        .map(String::as_str)
        .ok_or_else(|| format!("informe {nome}"))
}
fn card(ws: &Workspace, a: &[String]) -> Result<PathBuf, String> {
    let nome = arg(a, 4, "a release")?;
    let numero = arg(a, 5, "o número do card")?;
    if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
        return Err("ID CARD deve ser numérico".into());
    }
    let release = ws
        .listar_releases()
        .into_iter()
        .find(|p| nome_de(p) == nome)
        .ok_or("release não encontrada")?;
    let c = release.join(ws.nome_card(numero));
    if c.is_dir() {
        Ok(c)
    } else {
        Err("card não encontrado".into())
    }
}
fn arquivo(p: Option<PathBuf>, nome: &str) -> Result<PathBuf, String> {
    let p = p.ok_or_else(|| format!("{nome} não encontrado"))?;
    if p.is_file() {
        Ok(p)
    } else {
        Err(format!("{nome} não encontrado: {}", p.display()))
    }
}
fn planilha(ws: &Workspace, c: &Path, a: &[String]) -> Result<PathBuf, String> {
    arquivo(
        a.get(6)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| ws.achar_planilha_em(c))
            .or_else(|| ws.achar_planilha_recente()),
        "planilha",
    )
}
fn modelo(ws: &Workspace, a: &[String]) -> Result<PathBuf, String> {
    arquivo(
        a.get(7)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| ws.achar_modelo()),
        "modelo",
    )
}
fn ambiguidades(c: &Path, p: &Path) -> Result<String, String> {
    let linhas = read_xlsx(p)?;
    let col = HeaderMap::from_headers(linhas.first().ok_or("planilha sem linhas")?)?;
    let mut ids = Vec::new();
    let mut itens = Vec::new();
    for linha in linhas.iter().skip(1) {
        let id = col.get(linha, col.id);
        if id.is_empty()
            || util::normalize(&col.get(linha, col.status)) == "ignorado"
            || ids.contains(&id)
        {
            continue;
        }
        ids.push(id.clone());
        let pastas = acoes::achar_subpastas(c, &id);
        if pastas.len() > 1 {
            itens.push(format!(
                "{{\"id\":{},\"pastas\":{}}}",
                json(&id),
                nomes(&pastas)
            ));
        }
    }
    Ok(format!("[{}]", itens.join(",")))
}
fn executar(a: &[String]) -> Result<String, String> {
    let acao = arg(a, 1, "a ação")?;
    let base = PathBuf::from(arg(a, 2, "a pasta de trabalho")?);
    if !base.is_dir() {
        return Err(format!(
            "pasta de trabalho não encontrada: {}",
            base.display()
        ));
    }
    let fuso = util::validar_fuso(arg(a, 3, "o fuso")?).ok_or("fuso inválido")?;
    util::definir_fuso(fuso);
    let ws = Workspace::detectar(&base);
    match acao {
        "snapshot" => {
            let releases = ws
                .listar_releases()
                .iter()
                .map(|r| {
                    format!(
                        "{{\"nome\":{},\"cards\":{}}}",
                        json(nome_de(r)),
                        nomes(&ws.listar_cards(r))
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let p = ws
                .achar_planilha_recente()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            let m = ws
                .achar_modelo()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            Ok(format!("{{\"releases\":[{releases}],\"planilha\":{},\"modelo\":{},\"data\":{},\"libreoffice\":{},\"qpdf\":{},\"ghostscript\":{}}}", json(&p), json(&m), json(&util::data_ddmmaaaa()), pdf::soffice_bin().is_some(), pdf::find_in_path(&["qpdf"]).is_some(), pdf::find_in_path(&["gs", "gswin64c", "gswin32c"]).is_some()))
        }
        "contexto" => {
            let nome = arg(a, 4, "a release")?;
            let numero = arg(a, 5, "o número do card")?;
            if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
                return Err("ID CARD deve ser numérico".into());
            }
            let existente = ws
                .listar_releases()
                .into_iter()
                .find(|p| nome_de(p) == nome);
            let r = if let Some(r) = existente {
                r
            } else {
                let valido = (1..=12)
                    .any(|m| (2020..=2100).any(|ano| menu::montar_nome_release(m, ano) == nome));
                if !valido {
                    return Err("nome de release inválido".into());
                }
                let r = ws.dir_release.join(nome);
                fs::create_dir_all(&r).map_err(|e| format!("não criei release: {e}"))?;
                r
            };
            let c = r.join(ws.nome_card(numero));
            fs::create_dir_all(&c).map_err(|e| format!("não criei card: {e}"))?;
            Ok(format!(
                "{{\"mensagem\":{}}}",
                json(&format!("Contexto: {nome} / {}", nome_de(&c)))
            ))
        }
        "ambiguidade" => {
            let c = card(&ws, a)?;
            Ok(format!(
                "{{\"itens\":{}}}",
                ambiguidades(&c, &planilha(&ws, &c, a)?)?
            ))
        }
        "organizar" => {
            let c = card(&ws, a)?;
            let n = acoes::criar_subpastas(&c, &planilha(&ws, &c, a)?)?;
            Ok(format!(
                "{{\"mensagem\":{}}}",
                json(&format!("{n} subpasta(s) criada(s)."))
            ))
        }
        "descompactar" => {
            let r = acoes::descompactar_zips(&card(&ws, a)?)?;
            Ok(format!(
                "{{\"mensagem\":{}}}",
                json(&format!(
                    "{} zip(s), {} imagem(ns), {} mantida(s), {} falha(s).",
                    r.zips, r.imagens, r.mantidas, r.falhas
                ))
            ))
        }
        "docx" => {
            let c = card(&ws, a)?;
            let p = planilha(&ws, &c, a)?;
            let m = modelo(&ws, a)?;
            let escolhas = a
                .iter()
                .skip(9)
                .filter_map(|s| s.split_once('='))
                .collect::<Vec<_>>();
            let desempatar = |id: &str, _: &str, candidatas: &[PathBuf]| {
                let nome = escolhas.iter().find(|(i, _)| *i == id)?.1;
                candidatas.iter().find(|p| nome_de(p) == nome).cloned()
            };
            let r = acoes::gerar_docx(
                &c,
                &p,
                &m,
                a.get(8).map(String::as_str) == Some("true"),
                &desempatar,
            )?;
            Ok(format!(
                "{{\"mensagem\":{}}}",
                json(&format!(
                    "{} DET(s) gerado(s), {} ignorado(s), {} aviso(s).",
                    r.gerados, r.ignorados, r.avisos
                ))
            ))
        }
        "pdf" => {
            let n = acoes::gerar_pdf(&card(&ws, a)?)?;
            Ok(format!(
                "{{\"mensagem\":{}}}",
                json(&format!("{n} PDF(s) gerado(s)."))
            ))
        }
        "compilado" => {
            let n = arg(a, 6, "o limite em MB")?
                .parse::<u64>()
                .map_err(|_| "limite inválido")?;
            if n == 0 {
                return Err("limite deve ser maior que zero".into());
            }
            let v = acoes::gerar_compilado(&card(&ws, a)?, n)?;
            Ok(format!(
                "{{\"mensagem\":{}}}",
                json(&format!("{} compilado(s) gerado(s).", v.len()))
            ))
        }
        _ => Err("ação desconhecida".into()),
    }
}
fn main() -> ExitCode {
    let a: Vec<String> = env::args().collect();
    match executar(&a) {
        Ok(dados) => {
            println!("DET_JSON:{{\"ok\":true,\"dados\":{dados}}}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("DET_JSON:{{\"ok\":false,\"erro\":{}}}", json(&e));
            ExitCode::FAILURE
        }
    }
}
