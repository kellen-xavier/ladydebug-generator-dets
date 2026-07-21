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

    /// Planilha `teste_selected_*.xlsx` mais recente na raiz (por data de mod.).
    pub fn achar_planilha_recente(&self) -> Option<PathBuf> {
        let mut cand: Vec<(SystemTime, PathBuf)> = fs::read_dir(&self.raiz)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                ext_eh(p, "xlsx")
                    && nome_de(p).to_ascii_lowercase().starts_with("teste_selected")
            })
            .filter_map(|p| {
                let t = fs::metadata(&p).and_then(|m| m.modified()).ok()?;
                Some((t, p))
            })
            .collect();
        cand.sort_by(|a, b| a.0.cmp(&b.0));
        cand.into_iter().last().map(|(_, p)| p)
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
