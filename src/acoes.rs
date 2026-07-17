//! As cinco acoes do gestor de DET.

use std::fs;
use std::path::{Path, PathBuf};

use crate::docx::Docx;
use crate::pdf;
use crate::util::*;
use crate::workspace::{nome_de, Workspace};
use crate::xlsx::{read_xlsx, HeaderMap};

// ─── 1. Criar pasta da Release ─────────────────────────────────────────────

pub fn criar_release(ws: &Workspace) -> Result<PathBuf, String> {
    let destino = ws.caminho_release_atual();
    if destino.is_dir() {
        log(&format!("Ja existe: {}", destino.display()));
    } else {
        fs::create_dir_all(&destino)
            .map_err(|e| format!("nao criei '{}': {e}", destino.display()))?;
        ok(&format!("Pasta da release criada: {}", destino.display()));
    }
    Ok(destino)
}

// ─── 2. Criar subpastas dos testes ─────────────────────────────────────────

/// Cria `<ID> - <nome ate 10 chars>` para cada linha da planilha.
pub fn criar_subpastas(release_dir: &Path, planilha: &Path) -> Result<usize, String> {
    let linhas = read_xlsx(planilha)?;
    if linhas.is_empty() {
        return Err("planilha sem linhas".into());
    }
    let col = HeaderMap::from_headers(&linhas[0])?;

    let mut criadas = 0usize;
    for (n, linha) in linhas.iter().enumerate().skip(1) {
        let id = col.get(linha, col.id);
        if id.is_empty() {
            continue;
        }
        let nome = col.get(linha, col.nome);
        let nome_curto = sanitize(&truncar(&nome, 10));
        let pasta = format!("{id} - {nome_curto}");
        let destino = release_dir.join(&pasta);
        if destino.is_dir() {
            log(&format!("Linha {n}: ja existe '{pasta}'"));
        } else {
            fs::create_dir_all(&destino)
                .map_err(|e| format!("nao criei '{}': {e}", destino.display()))?;
            ok(&format!("Subpasta criada: {pasta}"));
            criadas += 1;
        }
    }
    Ok(criadas)
}

// ─── 3. Gerar DET - docx ───────────────────────────────────────────────────

pub struct ResumoDocx {
    pub gerados: usize,
    pub ignorados: usize,
    pub avisos: usize,
}

pub fn gerar_docx(
    release_dir: &Path,
    planilha: &Path,
    modelo: &Path,
    legendas: bool,
) -> Result<ResumoDocx, String> {
    let linhas = read_xlsx(planilha)?;
    if linhas.is_empty() {
        return Err("planilha sem linhas".into());
    }
    let col = HeaderMap::from_headers(&linhas[0])?;
    let template = Docx::read(modelo)?;
    let data_hoje = data_ddmmaaaa();

    let mut r = ResumoDocx { gerados: 0, ignorados: 0, avisos: 0 };

    for (n, linha) in linhas.iter().enumerate().skip(1) {
        let id = col.get(linha, col.id);
        if id.is_empty() {
            continue;
        }
        let nome = col.get(linha, col.nome);
        let executor = col.get(linha, col.executor);
        let status = col.get(linha, col.status);

        if normalize(&status) == "ignorado" {
            log(&format!("Linha {n}: ID {id} '{nome}' -> Ignorado (DET nao gerado)."));
            r.ignorados += 1;
            continue;
        }
        if nome.is_empty() || executor.is_empty() {
            warn(&format!("Linha {n}: ID {id} sem nome/executor -> pulado."));
            r.avisos += 1;
            continue;
        }

        let subpasta = match achar_subpasta(release_dir, &id) {
            Some(p) => p,
            None => {
                warn(&format!(
                    "Linha {n}: ID {id} '{nome}' -> subpasta nao encontrada -> pulado."
                ));
                r.avisos += 1;
                continue;
            }
        };

        let imagens = listar_imagens(&subpasta);
        let mut doc = template.clone();
        let vars = [
            ("{{ID}}", id.as_str()),
            ("{{NOME_TESTE}}", nome.as_str()),
            ("{{EXECUTADO_POR}}", executor.as_str()),
            ("{{STATUS}}", status.as_str()),
            ("{{DATA}}", data_hoje.as_str()),
        ];
        doc.fill_placeholders(&vars);
        let n_img = doc.insert_images(&imagens, legendas)?;

        let arquivo = format!("DET_{}_{}.docx", sanitize(&id), sanitize(&nome));
        let destino = subpasta.join(&arquivo);
        doc.write(&destino)?;
        ok(&format!("ID {id} '{nome}' -> {} ({n_img} imagem(ns))", destino.display()));
        r.gerados += 1;
    }
    Ok(r)
}

// ─── 4. Gerar DET PDF (docx -> pdf) ────────────────────────────────────────

pub fn gerar_pdf(release_dir: &Path) -> Result<usize, String> {
    let docs = coletar_arquivos(release_dir, "docx", Some("DET_"));
    if docs.is_empty() {
        return Err("nenhum DET .docx encontrado (rode 'Gerar DET - docx' antes).".into());
    }
    if pdf::soffice_bin().is_none() {
        return Err("LibreOffice nao encontrado (instale o LibreOffice ou ajuste o PATH).".into());
    }
    let mut n = 0usize;
    for d in &docs {
        match pdf::docx_to_pdf(d) {
            Ok(p) => {
                ok(&format!("PDF: {}", p.display()));
                n += 1;
            }
            Err(e) => warn(&format!("{}: {e}", d.display())),
        }
    }
    Ok(n)
}

// ─── 5. Gerar DET compilado (juntar PDFs, limite 30 MB) ────────────────────

pub fn gerar_compilado(release_dir: &Path, limite_mb: u64) -> Result<Vec<PathBuf>, String> {
    let limite = limite_mb * 1024 * 1024;
    let mut pdfs = coletar_arquivos(release_dir, "pdf", None);
    // ignora arquivos ja compilados
    pdfs.retain(|p| !nome_de(p).to_ascii_lowercase().starts_with("det_compilado"));
    if pdfs.is_empty() {
        return Err("nenhum PDF encontrado nas subpastas (rode 'Gerar DET PDF' antes).".into());
    }

    log(&format!("{} PDF(s) para compilar (limite {} MB).", pdfs.len(), limite_mb));

    // Tentativa unica: junta tudo e comprime.
    let tmp = release_dir.join(".det_merge_tmp.pdf");
    pdf::merge_pdfs(&pdfs, &tmp)?;
    let unico = release_dir.join("DET_Compilado.pdf");
    aplicar_compressao(&tmp, &unico)?;

    if pdf::tamanho(&unico) <= limite {
        let _ = fs::remove_file(&tmp);
        ok(&format!(
            "Compilado: {} ({:.1} MB)",
            unico.display(),
            pdf::tamanho(&unico) as f64 / 1_048_576.0
        ));
        return Ok(vec![unico]);
    }

    // Excedeu o limite: divide em partes por tamanho (ordem natural preservada).
    let _ = fs::remove_file(&unico);
    log("Acima do limite -> dividindo em partes.");
    let grupos = bin_pack(&pdfs, (limite as f64 * 0.9) as u64);
    let mut saidas = Vec::new();
    for (i, grupo) in grupos.iter().enumerate() {
        let parte = release_dir.join(format!("DET_Compilado_Parte_{}.pdf", i + 1));
        pdf::merge_pdfs(grupo, &tmp)?;
        aplicar_compressao(&tmp, &parte)?;
        ok(&format!(
            "Parte {}: {} ({:.1} MB)",
            i + 1,
            parte.display(),
            pdf::tamanho(&parte) as f64 / 1_048_576.0
        ));
        saidas.push(parte);
    }
    let _ = fs::remove_file(&tmp);
    Ok(saidas)
}

/// Comprime `entrada` -> `saida`; se o Ghostscript nao existir, apenas copia.
fn aplicar_compressao(entrada: &Path, saida: &Path) -> Result<(), String> {
    match pdf::comprimir_pdf(entrada, saida)? {
        true => Ok(()),
        false => {
            fs::copy(entrada, saida)
                .map(|_| ())
                .map_err(|e| format!("nao copiei o PDF: {e}"))
        }
    }
}

/// Agrupa PDFs em partes cujo tamanho somado (bruto) nao passa de `limite`.
/// Um PDF maior que o limite vira uma parte propria (sem divisao por pagina).
fn bin_pack(pdfs: &[PathBuf], limite: u64) -> Vec<Vec<PathBuf>> {
    let mut grupos: Vec<Vec<PathBuf>> = Vec::new();
    let mut atual: Vec<PathBuf> = Vec::new();
    let mut soma = 0u64;
    for p in pdfs {
        let t = pdf::tamanho(p);
        if !atual.is_empty() && soma + t > limite {
            grupos.push(std::mem::take(&mut atual));
            soma = 0;
        }
        atual.push(p.clone());
        soma += t;
    }
    if !atual.is_empty() {
        grupos.push(atual);
    }
    grupos
}

// ─── Helpers de localizacao ────────────────────────────────────────────────

/// Coleta arquivos com a extensao dada nas subpastas da release, em ordem
/// natural (por subpasta e por nome). `prefixo` filtra pelo inicio do nome.
fn coletar_arquivos(release_dir: &Path, ext: &str, prefixo: Option<&str>) -> Vec<PathBuf> {
    let mut subs: Vec<PathBuf> = match fs::read_dir(release_dir) {
        Ok(rd) => rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect(),
        Err(_) => return Vec::new(),
    };
    subs.sort_by(|a, b| natural_key(nome_de(a)).cmp(&natural_key(nome_de(b))));

    let mut out = Vec::new();
    for sub in subs {
        let mut arquivos: Vec<PathBuf> = match fs::read_dir(&sub) {
            Ok(rd) => rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && p.extension()
                            .and_then(|s| s.to_str())
                            .map(|s| s.eq_ignore_ascii_case(ext))
                            .unwrap_or(false)
                        && prefixo
                            .map(|pre| nome_de(p).starts_with(pre))
                            .unwrap_or(true)
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        arquivos.sort_by(|a, b| natural_key(nome_de(a)).cmp(&natural_key(nome_de(b))));
        out.extend(arquivos);
    }
    out
}

/// Acha a subpasta cujo prefixo numerico e igual a ID.
fn achar_subpasta(release: &Path, id: &str) -> Option<PathBuf> {
    let alvo = id.trim();
    for e in fs::read_dir(release).ok()?.flatten() {
        let path = e.path();
        if path.is_dir() {
            if let Some(nome) = path.file_name().and_then(|s| s.to_str()) {
                if prefixo_numerico(nome).as_deref() == Some(alvo) {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn prefixo_numerico(nome: &str) -> Option<String> {
    let d: String = nome.chars().take_while(|c| c.is_ascii_digit()).collect();
    if d.is_empty() {
        None
    } else {
        Some(d)
    }
}

fn listar_imagens(pasta: &Path) -> Vec<PathBuf> {
    let mut imgs: Vec<PathBuf> = match fs::read_dir(pasta) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file() && eh_imagem(p))
            .collect(),
        Err(_) => Vec::new(),
    };
    imgs.sort_by(|a, b| natural_key(nome_de(a)).cmp(&natural_key(nome_de(b))));
    imgs
}

fn eh_imagem(p: &Path) -> bool {
    matches!(
        p.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()).as_deref(),
        Some("png") | Some("jpg") | Some("jpeg")
    )
}
