//! Motor de docx: le um modelo (ZIP+XML), preenche tokens de texto (robusto a
//! runs quebrados) e insere imagens inline no ponto {{EVIDENCIAS}}.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::util::*;

const EMU_PER_PX: u64 = 9525; // 914400 EMU/pol / 96 px/pol
const MAX_W_EMU: u64 = 5_760_000; // ~160mm: largura util em A4 (margens 2,5cm)
const MAX_H_EMU: u64 = 8_000_000; // folga vertical

#[derive(Clone)]
pub struct Docx {
    entries: Vec<(String, Vec<u8>)>,
}

impl Docx {
    pub fn read(path: &Path) -> Result<Docx, String> {
        let file =
            File::open(path).map_err(|e| format!("nao abri modelo '{}': {e}", path.display()))?;
        let mut zip = ZipArchive::new(file).map_err(|e| format!("modelo docx invalido: {e}"))?;
        let mut entries = Vec::with_capacity(zip.len());
        for i in 0..zip.len() {
            let mut f = zip.by_index(i).map_err(|e| format!("erro no zip: {e}"))?;
            let name = f.name().to_string();
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| format!("erro lendo {name}: {e}"))?;
            entries.push((name, buf));
        }
        if !entries.iter().any(|(n, _)| n == "word/document.xml") {
            return Err("modelo nao contem word/document.xml".into());
        }
        Ok(Docx { entries })
    }

    fn entry_str(&self, nome: &str) -> Option<String> {
        self.entries
            .iter()
            .find(|(n, _)| n == nome)
            .map(|(_, b)| String::from_utf8_lossy(b).into_owned())
    }

    fn set_entry(&mut self, nome: &str, conteudo: Vec<u8>) {
        if let Some(e) = self.entries.iter_mut().find(|(n, _)| n == nome) {
            e.1 = conteudo;
        } else {
            self.entries.push((nome.to_string(), conteudo));
        }
    }

    /// Preenche os tokens de texto no document.xml, robusto a runs quebrados.
    pub fn fill_placeholders(&mut self, vars: &[(&str, &str)]) {
        if let Some(doc) = self.entry_str("word/document.xml") {
            let novo = fill_text(&doc, vars);
            self.set_entry("word/document.xml", novo.into_bytes());
        }
    }

    /// Insere as imagens no lugar do paragrafo {{EVIDENCIAS}}.
    /// Devolve quantas imagens foram inseridas.
    pub fn insert_images(
        &mut self,
        imagens: &[std::path::PathBuf],
        legendas: bool,
    ) -> Result<usize, String> {
        let doc = self.entry_str("word/document.xml").ok_or("document.xml ausente")?;

        if !doc.contains("{{EVIDENCIAS}}") {
            if !imagens.is_empty() {
                warn("modelo sem {{EVIDENCIAS}}: imagens nao inseridas.");
            }
            return Ok(0);
        }
        if imagens.is_empty() {
            let limpo = remover_paragrafo_com(&doc, "{{EVIDENCIAS}}");
            self.set_entry("word/document.xml", limpo.into_bytes());
            return Ok(0);
        }

        let mut rels = self
            .entry_str("word/_rels/document.xml.rels")
            .unwrap_or_else(rels_vazio);
        let mut ctypes = self
            .entry_str("[Content_Types].xml")
            .ok_or("[Content_Types].xml ausente")?;

        let mut prox_rid = max_rid(&rels) + 1;
        let mut media_idx = max_media_idx(&self.entries) + 1;
        let mut docpr = 900_000u32;

        let mut xml_imgs = String::new();
        let mut novas_media: Vec<(String, Vec<u8>)> = Vec::new();
        let mut inseridas = 0usize;

        for img in imagens {
            let bytes = match fs::read(img) {
                Ok(b) => b,
                Err(e) => {
                    warn(&format!("nao li imagem {}: {e}", img.display()));
                    continue;
                }
            };
            let ext = img
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("png")
                .to_ascii_lowercase();
            let (cx, cy) = match image_size(&bytes, &ext) {
                Some((w, h)) => escalar(w, h),
                None => {
                    warn(&format!("dimensoes ilegiveis em {} -> pulada.", img.display()));
                    continue;
                }
            };

            let media_name = format!("word/media/det_img{media_idx}.{ext}");
            let rel_target = format!("media/det_img{media_idx}.{ext}");
            let rid = format!("rId{prox_rid}");
            let nome_arq = img.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();

            garantir_content_type(&mut ctypes, &ext);
            rels = adicionar_rel(&rels, &rid, &rel_target);

            xml_imgs.push_str(&paragrafo_imagem(&rid, cx, cy, docpr, &nome_arq));
            if legendas {
                xml_imgs.push_str(&paragrafo_legenda(&nome_arq));
            }

            novas_media.push((media_name, bytes));
            prox_rid += 1;
            media_idx += 1;
            docpr += 1;
            inseridas += 1;
        }

        let doc = substituir_paragrafo_com(&doc, "{{EVIDENCIAS}}", &xml_imgs);

        self.set_entry("word/document.xml", doc.into_bytes());
        self.set_entry("word/_rels/document.xml.rels", rels.into_bytes());
        self.set_entry("[Content_Types].xml", ctypes.into_bytes());
        for (nome, bytes) in novas_media {
            self.set_entry(&nome, bytes);
        }
        Ok(inseridas)
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        if let Some(pai) = path.parent() {
            fs::create_dir_all(pai).ok();
        }
        let file = File::create(path).map_err(|e| format!("nao criei '{}': {e}", path.display()))?;
        let mut zip = ZipWriter::new(file);
        for (nome, bytes) in &self.entries {
            let metodo = if nome.starts_with("word/media/") {
                CompressionMethod::Stored
            } else {
                CompressionMethod::Deflated
            };
            let opts = FileOptions::default().compression_method(metodo);
            zip.start_file(nome, opts).map_err(|e| format!("zip start {nome}: {e}"))?;
            zip.write_all(bytes).map_err(|e| format!("zip write {nome}: {e}"))?;
        }
        zip.finish().map_err(|e| format!("zip finish: {e}"))?;
        Ok(())
    }
}

// ─── Preenchimento de texto robusto a runs quebrados ───────────────────────

fn fill_text(doc: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(doc.len() + 256);
    let bytes = doc.as_bytes();
    let mut i = 0;
    while let Some(ini) = find_from(bytes, i, b"<w:p") {
        let dc = bytes.get(ini + 4).copied().unwrap_or(b'x');
        if dc != b'>' && dc != b' ' && dc != b'/' {
            out.push_str(&doc[i..ini + 4]);
            i = ini + 4;
            continue;
        }
        out.push_str(&doc[i..ini]);
        let gt = match find_from(bytes, ini, b">") {
            Some(g) => g,
            None => {
                out.push_str(&doc[ini..]);
                return out;
            }
        };
        if bytes.get(gt - 1) == Some(&b'/') {
            out.push_str(&doc[ini..=gt]);
            i = gt + 1;
            continue;
        }
        let fim = match find_from(bytes, gt + 1, b"</w:p>") {
            Some(f) => f + 6,
            None => {
                out.push_str(&doc[ini..]);
                return out;
            }
        };
        out.push_str(&processar_paragrafo(&doc[ini..fim], vars));
        i = fim;
    }
    out.push_str(&doc[i..]);
    out
}

fn processar_paragrafo(par: &str, vars: &[(&str, &str)]) -> String {
    let concat: String = iter_text_of(par, "w:t").concat();
    if !vars.iter().any(|(tok, _)| concat.contains(tok)) {
        return par.to_string();
    }
    // Escapa todo o texto do paragrafo antes de reescrever os runs: os tokens
    // {{ }} nao tem caracteres especiais e sobrevivem ao escape; os valores sao
    // escapados na substituicao. Assim texto estatico com & < > nao corrompe o XML.
    let mut novo = xml_escape(&concat);
    for (tok, val) in vars {
        if novo.contains(tok) {
            novo = novo.replace(tok, &xml_escape(val));
        }
    }
    reescrever_runs(par, &novo)
}

/// Reescreve os `<w:t>`: o primeiro recebe `texto`, os demais ficam vazios.
fn reescrever_runs(par: &str, texto: &str) -> String {
    let bytes = par.as_bytes();
    let mut out = String::with_capacity(par.len() + texto.len());
    let mut i = 0;
    let mut primeiro = true;
    while let Some(p) = find_from(bytes, i, b"<w:t") {
        let c = bytes.get(p + 4).copied().unwrap_or(b' ');
        if c != b'>' && c != b' ' && c != b'/' {
            out.push_str(&par[i..p + 4]);
            i = p + 4;
            continue;
        }
        let gt = match find_from(bytes, p, b">") {
            Some(g) => g,
            None => break,
        };
        out.push_str(&par[i..p]);
        if bytes.get(gt - 1) == Some(&b'/') {
            if primeiro {
                out.push_str("<w:t xml:space=\"preserve\">");
                out.push_str(texto);
                out.push_str("</w:t>");
                primeiro = false;
            } else {
                out.push_str("<w:t></w:t>");
            }
            i = gt + 1;
            continue;
        }
        let fim = match find_from(bytes, gt + 1, b"</w:t>") {
            Some(f) => f,
            None => break,
        };
        if primeiro {
            out.push_str("<w:t xml:space=\"preserve\">");
            out.push_str(texto);
            out.push_str("</w:t>");
            primeiro = false;
        } else {
            out.push_str("<w:t></w:t>");
        }
        i = fim + 6;
    }
    out.push_str(&par[i..]);
    out
}

// ─── Manipulacao de paragrafos por token ───────────────────────────────────

fn inicio_paragrafo_antes(doc: &str, pos: usize) -> Option<usize> {
    let bytes = doc.as_bytes();
    let mut ultimo = None;
    let mut i = 0;
    while let Some(p) = find_from(bytes, i, b"<w:p") {
        if p >= pos {
            break;
        }
        let c = bytes.get(p + 4).copied().unwrap_or(b'x');
        if c == b'>' || c == b' ' {
            ultimo = Some(p);
        }
        i = p + 4;
    }
    ultimo
}

fn intervalo_paragrafo(doc: &str, token: &str) -> Option<(usize, usize)> {
    let pos = doc.find(token)?;
    let bytes = doc.as_bytes();
    let ini = inicio_paragrafo_antes(doc, pos)?;
    let gt = find_from(bytes, ini, b">")?;
    if bytes.get(gt - 1) == Some(&b'/') {
        return Some((ini, gt + 1));
    }
    let fim = find_from(bytes, gt + 1, b"</w:p>")? + 6;
    Some((ini, fim))
}

fn substituir_paragrafo_com(doc: &str, token: &str, xml_novo: &str) -> String {
    match intervalo_paragrafo(doc, token) {
        Some((ini, fim)) => {
            let mut out = String::with_capacity(doc.len() + xml_novo.len());
            out.push_str(&doc[..ini]);
            out.push_str(xml_novo);
            out.push_str(&doc[fim..]);
            out
        }
        None => doc.to_string(),
    }
}

fn remover_paragrafo_com(doc: &str, token: &str) -> String {
    substituir_paragrafo_com(doc, token, "")
}

// ─── XML de imagem (drawing inline) ────────────────────────────────────────

fn escalar(w: u32, h: u32) -> (u64, u64) {
    let cx = w as u64 * EMU_PER_PX;
    let cy = h as u64 * EMU_PER_PX;
    let mut fator = 1.0f64;
    if cx > MAX_W_EMU {
        fator = fator.min(MAX_W_EMU as f64 / cx as f64);
    }
    if (cy as f64 * fator) as u64 > MAX_H_EMU {
        fator = fator.min(MAX_H_EMU as f64 / cy as f64);
    }
    let fcx = (cx as f64 * fator).round() as u64;
    let fcy = (cy as f64 * fator).round() as u64;
    (fcx.max(1), fcy.max(1))
}

fn paragrafo_imagem(rid: &str, cx: u64, cy: u64, docpr: u32, descr: &str) -> String {
    let d = xml_escape(descr);
    format!(
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:drawing>\
<wp:inline xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\" \
xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
<wp:extent cx=\"{cx}\" cy=\"{cy}\"/>\
<wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/>\
<wp:docPr id=\"{docpr}\" name=\"Imagem {docpr}\" descr=\"{d}\"/>\
<wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect=\"1\"/></wp:cNvGraphicFramePr>\
<a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
<pic:pic><pic:nvPicPr><pic:cNvPr id=\"{docpr}\" name=\"{d}\"/><pic:cNvPicPr/></pic:nvPicPr>\
<pic:blipFill><a:blip r:embed=\"{rid}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
<pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm>\
<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic>\
</a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"
    )
}

fn paragrafo_legenda(nome: &str) -> String {
    let n = xml_escape(nome);
    format!(
        "<w:p><w:pPr><w:jc w:val=\"center\"/><w:spacing w:after=\"120\"/></w:pPr>\
<w:r><w:rPr><w:i/><w:sz w:val=\"16\"/><w:szCs w:val=\"16\"/><w:color w:val=\"808080\"/></w:rPr>\
<w:t xml:space=\"preserve\">{n}</w:t></w:r></w:p>"
    )
}

// ─── Content-Types e Relationships ─────────────────────────────────────────

fn garantir_content_type(ctypes: &mut String, ext: &str) {
    let (e, mime) = match ext {
        "jpg" | "jpeg" => ("jpeg", "image/jpeg"),
        _ => ("png", "image/png"),
    };
    for extensao in [e, ext] {
        let marca = format!("Extension=\"{extensao}\"");
        if !ctypes.contains(&marca) {
            let decl = format!("<Default Extension=\"{extensao}\" ContentType=\"{mime}\"/>");
            if let Some(pos) = ctypes.find("</Types>") {
                ctypes.insert_str(pos, &decl);
            }
        }
    }
}

fn rels_vazio() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#
        .to_string()
}

fn adicionar_rel(rels: &str, rid: &str, target: &str) -> String {
    let decl = format!(
        "<Relationship Id=\"{rid}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"{target}\"/>"
    );
    if let Some(pos) = rels.find("</Relationships>") {
        let mut novo = String::with_capacity(rels.len() + decl.len());
        novo.push_str(&rels[..pos]);
        novo.push_str(&decl);
        novo.push_str(&rels[pos..]);
        novo
    } else {
        rels.to_string()
    }
}

fn max_rid(rels: &str) -> u32 {
    let mut max = 0u32;
    let bytes = rels.as_bytes();
    let mut i = 0;
    while let Some(p) = find_from(bytes, i, b"Id=\"rId") {
        let ini = p + 7;
        let mut n = String::new();
        for &b in &bytes[ini..] {
            if b.is_ascii_digit() {
                n.push(b as char);
            } else {
                break;
            }
        }
        if let Ok(v) = n.parse::<u32>() {
            max = max.max(v);
        }
        i = ini;
    }
    max
}

fn max_media_idx(entries: &[(String, Vec<u8>)]) -> u32 {
    let mut max = 0u32;
    for (nome, _) in entries {
        if let Some(resto) = nome.strip_prefix("word/media/") {
            let digitos: String = resto.chars().filter(|c| c.is_ascii_digit()).collect();
            if let Ok(v) = digitos.parse::<u32>() {
                max = max.max(v);
            }
        }
    }
    max
}

// ─── Dimensoes de imagem (PNG / JPEG) ──────────────────────────────────────

fn image_size(bytes: &[u8], ext: &str) -> Option<(u32, u32)> {
    match ext {
        "png" => png_size(bytes),
        "jpg" | "jpeg" => jpeg_size(bytes),
        _ => None,
    }
}

fn png_size(b: &[u8]) -> Option<(u32, u32)> {
    if b.len() < 24 || &b[0..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let w = u32::from_be_bytes([b[16], b[17], b[18], b[19]]);
    let h = u32::from_be_bytes([b[20], b[21], b[22], b[23]]);
    Some((w, h))
}

fn jpeg_size(b: &[u8]) -> Option<(u32, u32)> {
    if b.len() < 4 || b[0] != 0xFF || b[1] != 0xD8 {
        return None;
    }
    let mut i = 2;
    while i + 9 < b.len() {
        if b[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = b[i + 1];
        let eh_sof = matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF);
        if eh_sof {
            let h = u16::from_be_bytes([b[i + 5], b[i + 6]]) as u32;
            let w = u16::from_be_bytes([b[i + 7], b[i + 8]]) as u32;
            return Some((w, h));
        }
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            i += 2;
        } else {
            let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
            i += 2 + len;
        }
    }
    None
}
