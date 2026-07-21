#![allow(dead_code)]
//! Utilitarios de fixture para os testes de integracao: area de trabalho
//! temporaria (removida no Drop), e geracao de .xlsx / .docx / PNG minimos.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

static CONTADOR: AtomicUsize = AtomicUsize::new(0);

/// Area de trabalho temporaria unica, apagada automaticamente no fim do teste.
pub struct TempWs {
    pub raiz: PathBuf,
}

impl TempWs {
    pub fn nova() -> TempWs {
        let n = CONTADOR.fetch_add(1, Ordering::SeqCst);
        let nome = format!("gestor_det_test_{}_{}", std::process::id(), n);
        let raiz = std::env::temp_dir().join(nome);
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz).unwrap();
        TempWs { raiz }
    }
    pub fn join(&self, rel: &str) -> PathBuf {
        self.raiz.join(rel)
    }
    pub fn criar_dir(&self, rel: &str) -> PathBuf {
        let p = self.raiz.join(rel);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for TempWs {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.raiz);
    }
}

fn escapar_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn coluna_letra(mut c: usize) -> String {
    let mut s = String::new();
    c += 1;
    while c > 0 {
        let r = (c - 1) % 26;
        s.insert(0, (b'A' + r as u8) as char);
        c = (c - 1) / 26;
    }
    s
}

fn zipar(caminho: &Path, entradas: &[(&str, &[u8])]) {
    let arquivo = std::fs::File::create(caminho).unwrap();
    let mut zip = ZipWriter::new(arquivo);
    let opts = FileOptions::default().compression_method(CompressionMethod::Deflated);
    for (nome, bytes) in entradas {
        zip.start_file(*nome, opts).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
}

/// Escreve um .xlsx minimo (celulas inlineStr). A 1a linha e o cabecalho.
pub fn escrever_xlsx(caminho: &Path, linhas: &[Vec<&str>]) {
    let mut sheet = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#,
    );
    for (r, linha) in linhas.iter().enumerate() {
        sheet.push_str(&format!("<row r=\"{}\">", r + 1));
        for (c, val) in linha.iter().enumerate() {
            sheet.push_str(&format!(
                "<c r=\"{}{}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
                coluna_letra(c),
                r + 1,
                escapar_xml(val)
            ));
        }
        sheet.push_str("</row>");
    }
    sheet.push_str("</sheetData></worksheet>");

    let ct = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#;
    zipar(
        caminho,
        &[
            ("[Content_Types].xml", ct.as_bytes()),
            ("xl/worksheets/sheet1.xml", sheet.as_bytes()),
        ],
    );
}

/// Escreve um modelo .docx minimo com os paragrafos dados (texto ja XML-valido).
pub fn escrever_template(caminho: &Path, paragrafos: &[&str]) {
    let mut body = String::new();
    for p in paragrafos {
        body.push_str(&format!(
            "<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
            p
        ));
    }
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{}</w:body></w:document>"#,
        body
    );
    let ct = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    zipar(
        caminho,
        &[
            ("[Content_Types].xml", ct.as_bytes()),
            ("_rels/.rels", rels.as_bytes()),
            ("word/document.xml", document.as_bytes()),
        ],
    );
}

/// Bytes de um PNG com IHDR legivel (largura/altura corretas) para os testes.
pub fn png_falso(w: u32, h: u32) -> Vec<u8> {
    let mut b = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    b.extend_from_slice(&[0, 0, 0, 13]);
    b.extend_from_slice(b"IHDR");
    b.extend_from_slice(&w.to_be_bytes());
    b.extend_from_slice(&h.to_be_bytes());
    b.extend_from_slice(&[8, 2, 0, 0, 0]);
    b
}

/// Le uma entrada de texto (ex.: word/document.xml) de dentro de um zip/docx.
pub fn ler_entrada_zip(caminho: &Path, nome: &str) -> String {
    let f = std::fs::File::open(caminho).unwrap();
    let mut zip = ZipArchive::new(f).unwrap();
    let mut e = zip.by_name(nome).unwrap();
    let mut s = String::new();
    e.read_to_string(&mut s).unwrap();
    s
}

/// Lista os nomes de todas as entradas de um zip/docx.
pub fn nomes_zip(caminho: &Path) -> Vec<String> {
    let f = std::fs::File::open(caminho).unwrap();
    let mut zip = ZipArchive::new(f).unwrap();
    (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect()
}
