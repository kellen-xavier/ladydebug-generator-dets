//! Leitura minima de .xlsx (ZIP + XML): shared strings + primeira worksheet.
//! Devolve linhas como Vec<Vec<String>> e mapeia os cabecalhos obrigatorios.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use zip::ZipArchive;

use crate::util::*;

/// Le a primeira planilha do .xlsx e devolve as linhas (celulas como texto).
pub fn read_xlsx(path: &Path) -> Result<Vec<Vec<String>>, String> {
    let file = File::open(path).map_err(|e| format!("nao abri '{}': {e}", path.display()))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("xlsx invalido: {e}"))?;

    let shared = match ler_entry(&mut zip, "xl/sharedStrings.xml") {
        Some(xml) => parse_shared_strings(&xml),
        None => Vec::new(),
    };

    let sheet_name = primeira_worksheet(&mut zip).ok_or("nenhuma worksheet no xlsx")?;
    let sheet_xml = ler_entry(&mut zip, &sheet_name).ok_or_else(|| format!("nao li {sheet_name}"))?;
    Ok(parse_sheet(&sheet_xml, &shared))
}

fn ler_entry<R: Read + std::io::Seek>(zip: &mut ZipArchive<R>, nome: &str) -> Option<String> {
    let mut f = zip.by_name(nome).ok()?;
    let mut buf = String::new();
    f.read_to_string(&mut buf).ok()?;
    Some(buf)
}

fn primeira_worksheet<R: Read + std::io::Seek>(zip: &mut ZipArchive<R>) -> Option<String> {
    let mut nomes: Vec<String> = Vec::new();
    for i in 0..zip.len() {
        if let Ok(f) = zip.by_index(i) {
            let n = f.name().to_string();
            if n.starts_with("xl/worksheets/sheet") && n.ends_with(".xml") {
                nomes.push(n);
            }
        }
    }
    nomes.sort_by(|a, b| natural_key(a).cmp(&natural_key(b)));
    nomes.into_iter().next()
}

fn parse_shared_strings(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    for si in iter_blocos(xml, "<si", "</si>", "<si/>") {
        let mut texto = String::new();
        for t in iter_text_of(&si, "t") {
            texto.push_str(&t);
        }
        out.push(texto);
    }
    out
}

fn parse_sheet(xml: &str, shared: &[String]) -> Vec<Vec<String>> {
    let mut linhas = Vec::new();
    let mut max_cols = 0usize;

    for row in iter_blocos(xml, "<row", "</row>", "<row/>") {
        let mut celulas: Vec<(usize, String)> = Vec::new();
        for c in iter_celulas(&row) {
            let (col, val) = parse_celula(&c, shared);
            if col + 1 > max_cols {
                max_cols = col + 1;
            }
            celulas.push((col, val));
        }
        let mut linha = vec![String::new(); max_cols];
        for (col, val) in celulas {
            if col >= linha.len() {
                linha.resize(col + 1, String::new());
            }
            linha[col] = val;
        }
        linhas.push(linha);
    }
    for l in linhas.iter_mut() {
        if l.len() < max_cols {
            l.resize(max_cols, String::new());
        }
    }
    linhas
}

fn iter_celulas(row: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = row.as_bytes();
    let mut i = 0;
    while let Some(p) = find_from(bytes, i, b"<c") {
        let after = p + 2;
        let ch = bytes.get(after).copied().unwrap_or(b' ');
        if ch != b'>' && ch != b' ' && ch != b'/' {
            i = after;
            continue;
        }
        let gt = match find_from(bytes, p, b">") {
            Some(g) => g,
            None => break,
        };
        if bytes.get(gt - 1) == Some(&b'/') {
            out.push(row[p..=gt].to_string());
            i = gt + 1;
            continue;
        }
        let fim = match find_from(bytes, gt + 1, b"</c>") {
            Some(f) => f,
            None => break,
        };
        out.push(row[p..fim + 4].to_string());
        i = fim + 4;
    }
    out
}

fn parse_celula(c: &str, shared: &[String]) -> (usize, String) {
    let r = attr(c, "r").unwrap_or_default();
    let col = col_index(&r);
    let t = attr(c, "t").unwrap_or_default();

    if t == "inlineStr" {
        let mut s = String::new();
        for txt in iter_text_of(c, "t") {
            s.push_str(&txt);
        }
        return (col, s);
    }

    let v = inner_tag(c, "v").unwrap_or_default();
    if t == "s" {
        let idx: usize = v.trim().parse().unwrap_or(usize::MAX);
        (col, shared.get(idx).cloned().unwrap_or_default())
    } else if t == "str" {
        (col, xml_unescape(&v))
    } else {
        (col, formatar_numero(&xml_unescape(&v)))
    }
}

/// "13.0" -> "13"; mantem demais numeros/textos como estao.
fn formatar_numero(v: &str) -> String {
    let t = v.trim();
    if let Ok(f) = t.parse::<f64>() {
        if f.fract() == 0.0 && f.abs() < 1e15 {
            return format!("{}", f as i64);
        }
    }
    t.to_string()
}

/// "B7" -> 1 (indice 0-based da coluna).
fn col_index(cell_ref: &str) -> usize {
    let mut idx = 0usize;
    for c in cell_ref.chars() {
        if c.is_ascii_alphabetic() {
            idx = idx * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
        } else {
            break;
        }
    }
    idx.saturating_sub(1)
}

// ─── Mapeamento dos cabecalhos obrigatorios ────────────────────────────────

pub struct HeaderMap {
    pub id: usize,
    pub nome: usize,
    pub executor: usize,
    pub status: usize,
}

impl HeaderMap {
    pub fn from_headers(cab: &[String]) -> Result<HeaderMap, String> {
        let achar = |aliases: &[&str], campo: &str| -> Result<usize, String> {
            for (i, h) in cab.iter().enumerate() {
                let hn = normalize(h);
                if aliases.iter().any(|a| hn == *a) {
                    return Ok(i);
                }
            }
            Err(format!(
                "coluna obrigatoria nao encontrada: {campo} (cabecalhos: {})",
                cab.join(" | ")
            ))
        };
        Ok(HeaderMap {
            id: achar(&["id"], "ID")?,
            nome: achar(
                &[
                    "nome do teste inicial",
                    "nome do teste",
                    "nome do teste (inicial)",
                    "nome teste",
                ],
                "Nome do teste (inicial)",
            )?,
            executor: achar(
                &["executado por", "executador por", "executor", "responsavel"],
                "Executado por",
            )?,
            status: achar(&["status nativo", "status"], "Status nativo")?,
        })
    }

    pub fn get(&self, linha: &[String], idx: usize) -> String {
        linha.get(idx).cloned().unwrap_or_default().trim().to_string()
    }
}
