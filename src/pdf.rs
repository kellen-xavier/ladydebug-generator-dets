//! Integracao com ferramentas externas para PDF:
//!   - docx -> pdf via LibreOffice (`soffice`/`libreoffice`)
//!   - juntar PDFs via `qpdf`
//!   - comprimir via Ghostscript (`gs`)
//! Todas sao ferramentas externas opcionais; nenhuma dependencia de crate.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const IS_WINDOWS: bool = cfg!(target_os = "windows");

/// Procura um executavel no PATH (considerando PATHEXT no Windows).
pub fn find_in_path(names: &[&str]) -> Option<PathBuf> {
    let exts: Vec<String> = if IS_WINDOWS {
        env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.BAT;.CMD".to_string())
            .split(';')
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![String::new()]
    };
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        for name in names {
            for ext in &exts {
                let fname = if name.contains('.') || ext.is_empty() {
                    name.to_string()
                } else {
                    format!("{name}{ext}")
                };
                let cand = dir.join(&fname);
                if cand.is_file() {
                    return Some(cand);
                }
            }
        }
    }
    None
}

/// Localiza o LibreOffice: PATH + caminhos tipicos de instalacao no Windows.
pub fn soffice_bin() -> Option<PathBuf> {
    if let Some(p) = find_in_path(&["soffice", "libreoffice"]) {
        return Some(p);
    }
    if IS_WINDOWS {
        for c in [
            r"C:\Program Files\LibreOffice\program\soffice.exe",
            r"C:\Program Files (x86)\LibreOffice\program\soffice.exe",
        ] {
            let p = PathBuf::from(c);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Converte um .docx para .pdf na mesma pasta. Devolve o caminho do PDF.
pub fn docx_to_pdf(docx: &Path) -> Result<PathBuf, String> {
    let soffice = soffice_bin()
        .ok_or("LibreOffice nao encontrado (instale o LibreOffice ou ajuste o PATH).")?;
    let outdir = docx.parent().unwrap_or_else(|| Path::new("."));

    let status = Command::new(&soffice)
        .arg("--headless")
        .arg("--convert-to")
        .arg("pdf")
        .arg("--outdir")
        .arg(outdir)
        .arg(docx)
        .status()
        .map_err(|e| format!("falha ao executar soffice: {e}"))?;
    if !status.success() {
        return Err(format!("soffice retornou erro ao converter {}", docx.display()));
    }
    let pdf = docx.with_extension("pdf");
    if pdf.is_file() {
        Ok(pdf)
    } else {
        Err(format!("PDF nao gerado para {}", docx.display()))
    }
}

/// Junta uma lista de PDFs (na ordem dada) em um unico arquivo via qpdf.
pub fn merge_pdfs(entradas: &[PathBuf], saida: &Path) -> Result<(), String> {
    let qpdf = find_in_path(&["qpdf"]).ok_or("qpdf nao encontrado (instale o qpdf).")?;
    if entradas.is_empty() {
        return Err("nada para juntar (lista vazia).".into());
    }
    let mut cmd = Command::new(&qpdf);
    cmd.arg("--empty").arg("--pages");
    for e in entradas {
        cmd.arg(e);
    }
    cmd.arg("--").arg(saida);
    let status = cmd.status().map_err(|e| format!("falha ao executar qpdf: {e}"))?;
    if !status.success() {
        return Err("qpdf retornou erro ao juntar os PDFs.".into());
    }
    Ok(())
}

/// Comprime um PDF via Ghostscript (qualidade "printer"). Se `gs` nao existir,
/// devolve Ok(false) e o chamador segue com o arquivo nao comprimido.
pub fn comprimir_pdf(entrada: &Path, saida: &Path) -> Result<bool, String> {
    let gs = match find_in_path(&["gs", "gswin64c", "gswin32c"]) {
        Some(g) => g,
        None => return Ok(false),
    };
    let status = Command::new(&gs)
        .arg("-sDEVICE=pdfwrite")
        .arg("-dCompatibilityLevel=1.4")
        .arg("-dPDFSETTINGS=/printer")
        .arg("-dNOPAUSE")
        .arg("-dBATCH")
        .arg("-dQUIET")
        .arg(format!("-sOutputFile={}", saida.display()))
        .arg(entrada)
        .status()
        .map_err(|e| format!("falha ao executar gs: {e}"))?;
    if !status.success() {
        return Err("ghostscript retornou erro ao comprimir.".into());
    }
    Ok(true)
}

pub fn tamanho(p: &Path) -> u64 {
    fs::metadata(p).map(|m| m.len()).unwrap_or(0)
}
