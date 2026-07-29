//! Descoberta da area de trabalho: pasta central `Release`, `modelos` e a
//! planilha exportada mais recente (`teste_selected_*.xlsx`).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::util::*;

pub struct Workspace {
    pub raiz: PathBuf,
    pub dir_release: PathBuf, // <raiz>/Release
    pub dir_modelos: PathBuf, // <raiz>/modelos
}

impl Workspace {
    pub fn detectar(base: &Path) -> Workspace {
        Workspace {
            raiz: base.to_path_buf(),
            dir_release: base.join("Release"),
            dir_modelos: base.join("modelos"),
        }
    }

    /// Lista as pastas `ID CARD <n>` dentro de uma release (ordem natural).
    pub fn listar_cards(&self, release: &Path) -> Vec<PathBuf> {
        let mut v: Vec<PathBuf> = match fs::read_dir(release) {
            Ok(rd) => rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir() && nome_de(p).to_ascii_uppercase().starts_with("ID CARD"))
                .collect(),
            Err(_) => Vec::new(),
        };
        v.sort_by(|a, b| natural_key(nome_de(a)).cmp(&natural_key(nome_de(b))));
        v
    }

    /// Nome padronizado da pasta de card: "ID CARD <numero>".
    pub fn nome_card(&self, numero: &str) -> String {
        format!("ID CARD {}", numero.trim())
    }

    /// Lista as pastas de release existentes em `Release/` (ordem natural).
    pub fn listar_releases(&self) -> Vec<PathBuf> {
        let mut v: Vec<PathBuf> = match fs::read_dir(&self.dir_release) {
            Ok(rd) => rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect(),
            Err(_) => Vec::new(),
        };
        v.sort_by(|a, b| natural_key(nome_de(a)).cmp(&natural_key(nome_de(b))));
        v
    }

    /// Primeiro `.docx` encontrado em `modelos/`.
    pub fn achar_modelo(&self) -> Option<PathBuf> {
        let mut docs: Vec<PathBuf> = fs::read_dir(&self.dir_modelos)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| ext_eh(p, "docx"))
            .collect();
        docs.sort_by(|a, b| natural_key(nome_de(a)).cmp(&natural_key(nome_de(b))));
        docs.into_iter().next()
    }

    /// Planilha `.xlsx` mais recente dentro de `dir` (por data de modificacao).
    ///
    /// Prefere nomes que contenham `teste_selected` (tolerante a prefixo, ex.:
    /// "execucoes teste_selected_...xlsx"). Se nenhum contiver mas houver
    /// exatamente **um** `.xlsx`, usa esse. Com varios `.xlsx` nao-teste_selected
    /// devolve `None` (ambiguo) para o chamador pedir ao usuario.
    pub fn achar_planilha_em(&self, dir: &Path) -> Option<PathBuf> {
        let xlsx: Vec<(SystemTime, PathBuf)> = fs::read_dir(dir)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| ext_eh(p, "xlsx"))
            .filter_map(|p| {
                let t = fs::metadata(&p).and_then(|m| m.modified()).ok()?;
                Some((t, p))
            })
            .collect();
        if xlsx.is_empty() {
            return None;
        }
        let preferidas: Vec<(SystemTime, PathBuf)> = xlsx
            .iter()
            .filter(|(_, p)| nome_de(p).to_ascii_lowercase().contains("teste_selected"))
            .cloned()
            .collect();
        let conjunto = if !preferidas.is_empty() {
            preferidas
        } else if xlsx.len() == 1 {
            xlsx
        } else {
            return None;
        };
        conjunto.into_iter().max_by(|a, b| a.0.cmp(&b.0)).map(|(_, p)| p)
    }

    /// Planilha mais recente na raiz da area de trabalho (fallback).
    pub fn achar_planilha_recente(&self) -> Option<PathBuf> {
        self.achar_planilha_em(&self.raiz)
    }
}

pub fn nome_de(p: &Path) -> &str {
    p.file_name().and_then(|s| s.to_str()).unwrap_or("")
}

fn ext_eh(p: &Path, ext: &str) -> bool {
    p.is_file()
        && p.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case(ext))
            .unwrap_or(false)
}
