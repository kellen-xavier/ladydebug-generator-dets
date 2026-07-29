//! As acoes de geracao do gestor de DET. Operam dentro da pasta do ID CARD
//! selecionado (que contem as subpastas `ID - nome` de cada teste).

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use crate::docx::Docx;
use crate::pdf;
use crate::util::*;
use crate::workspace::nome_de;
use crate::xlsx::{read_xlsx, HeaderMap};

// ─── Criar subpastas dos testes ────────────────────────────────────────────

/// Cria `<ID> - <nome ate 10 chars>` para cada linha da planilha, dentro do card.
pub fn criar_subpastas(card_dir: &Path, planilha: &Path) -> Result<usize, String> {
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
        let destino = card_dir.join(&pasta);
        if destino.is_dir() {
            log(&format!("Linha {n}: ja existe '{pasta}'"));
        } else {
            // Ja existe outra pasta com o mesmo prefixo de ID? Normalmente o
            // nome do teste mudou na planilha. Criar a segunda pasta deixaria
            // o ID ambiguo na hora de gerar o DET — avisa antes.
            let existentes = achar_subpastas(card_dir, &id);
            if !existentes.is_empty() {
                let nomes: Vec<&str> = existentes.iter().map(|p| nome_de(p)).collect();
                warn(&format!(
                    "Linha {n}: ID {id} ja tem a pasta '{}' — criando tambem '{pasta}'. \
                     Na geracao do DET sera preciso escolher entre elas; \
                     considere apagar ou renomear a que nao usa.",
                    nomes.join("', '")
                ));
            }
            fs::create_dir_all(&destino)
                .map_err(|e| format!("nao criei '{}': {e}", destino.display()))?;
            ok(&format!("Subpasta criada: {pasta}"));
            criadas += 1;
        }
    }
    Ok(criadas)
}

// ─── Descompactar .zip de evidencias ───────────────────────────────────────

/// Resumo de uma passada de descompactacao.
#[derive(Default)]
pub struct ResumoZip {
    pub zips: usize,     // .zip processados (que tinham imagens)
    pub imagens: usize,  // imagens extraidas
    pub mantidas: usize, // imagens ja existentes, preservadas (nao sobrescreve)
    pub falhas: usize,   // .zip que deram erro
}

const IMG_EXTS_ZIP: [&str; 8] = ["png", "jpg", "jpeg", "gif", "bmp", "webp", "tif", "tiff"];

fn nome_eh_imagem(nome: &str) -> bool {
    nome.rsplit('.')
        .next()
        .map(|ext| IMG_EXTS_ZIP.iter().any(|e| e.eq_ignore_ascii_case(ext)))
        .unwrap_or(false)
}

/// Extrai as imagens de UM `.zip` para a pasta dele, achatando (usa so o nome
/// do arquivo — imune a *zip-slip* por construcao) e **sem sobrescrever**
/// arquivos existentes. Devolve (imagens_extraidas, imagens_mantidas_por_colisao).
pub fn descompactar_zip(zip_path: &Path) -> Result<(usize, usize), String> {
    let destino = zip_path.parent().unwrap_or_else(|| Path::new("."));
    let file =
        fs::File::open(zip_path).map_err(|e| format!("nao abri '{}': {e}", zip_path.display()))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("zip invalido: {e}"))?;

    let mut imagens = 0usize;
    let mut mantidas = 0usize;
    for i in 0..zip.len() {
        let mut entrada = zip.by_index(i).map_err(|e| format!("erro no zip: {e}"))?;
        if !entrada.is_file() {
            continue;
        }
        let nome_interno = entrada.name().to_string();
        let base = match nome_interno.rsplit(|c| c == '/' || c == '\\').next() {
            Some(b) if !b.is_empty() => b,
            _ => continue,
        };
        if !nome_eh_imagem(base) {
            continue; // "somente as imagens": ignora .txt/.json/etc.
        }
        let alvo = destino.join(base);
        if alvo.exists() {
            mantidas += 1; // nunca sobrescreve evidencia existente
            continue;
        }
        let mut buf = Vec::new();
        entrada.read_to_end(&mut buf).map_err(|e| format!("erro lendo {base}: {e}"))?;
        fs::write(&alvo, &buf).map_err(|e| format!("nao gravei {}: {e}", alvo.display()))?;
        imagens += 1;
    }
    Ok((imagens, mantidas))
}

/// Descompacta os `.zip` das subpastas do card: extrai as imagens direto na
/// subpasta e **apaga o `.zip`** apos extrair. Nao sobrescreve imagens
/// existentes; um `.zip` sem nenhuma imagem e mantido (nada a perder).
pub fn descompactar_zips(card_dir: &Path) -> Result<ResumoZip, String> {
    let zips = achar_zips(card_dir);
    if zips.is_empty() {
        return Err("nenhum .zip encontrado nas subpastas do card.".into());
    }
    let mut r = ResumoZip::default();
    for z in &zips {
        match descompactar_zip(z) {
            Ok((0, 0)) => warn(&format!("{}: nenhuma imagem no zip -> mantido.", nome_de(z))),
            Ok((imgs, mant)) => {
                if let Err(e) = fs::remove_file(z) {
                    warn(&format!("extrai mas nao apaguei {}: {e}", nome_de(z)));
                }
                let pasta = z.parent().map(nome_de).unwrap_or("");
                ok(&format!(
                    "{} -> {imgs} imagem(ns) em '{pasta}' (zip removido)",
                    nome_de(z)
                ));
                r.zips += 1;
                r.imagens += imgs;
                r.mantidas += mant;
            }
            Err(e) => {
                warn(&format!("{}: {e}", nome_de(z)));
                r.falhas += 1;
            }
        }
    }
    Ok(r)
}

/// Acha todos os `.zip` nas subpastas do card, em ordem natural.
fn achar_zips(card_dir: &Path) -> Vec<PathBuf> {
    let mut subs: Vec<PathBuf> = match fs::read_dir(card_dir) {
        Ok(rd) => rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect(),
        Err(_) => return Vec::new(),
    };
    subs.sort_by(|a, b| natural_key(nome_de(a)).cmp(&natural_key(nome_de(b))));
    let mut zips = Vec::new();
    for sub in subs {
        let mut z: Vec<PathBuf> = match fs::read_dir(&sub) {
            Ok(rd) => rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && p.extension()
                            .and_then(|s| s.to_str())
                            .map(|s| s.eq_ignore_ascii_case("zip"))
                            .unwrap_or(false)
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        z.sort_by(|a, b| natural_key(nome_de(a)).cmp(&natural_key(nome_de(b))));
        zips.extend(z);
    }
    zips
}

// ─── Gerar DET - docx ───────────────────────────────────────────────────────

pub struct ResumoDocx {
    pub gerados: usize,
    pub ignorados: usize,
    pub avisos: usize,
}

/// Callback de desempate: chamado quando mais de uma subpasta tem o mesmo
/// prefixo numerico de ID. Recebe (id, nome do teste, candidatas em ordem
/// natural) e devolve a escolhida, ou `None` para pular o ID.
///
/// Existe para manter `gerar_docx` livre de I/O de terminal: o binario passa um
/// callback que pergunta ao usuario, os testes passam um deterministico.
pub type Desempate<'a> = &'a dyn Fn(&str, &str, &[PathBuf]) -> Option<PathBuf>;

/// Desempate que nunca escolhe — o ID ambiguo e pulado com aviso.
/// Usado nos testes e como fallback nao interativo.
pub fn sem_desempate(_id: &str, _nome: &str, _candidatas: &[PathBuf]) -> Option<PathBuf> {
    None
}

pub fn gerar_docx(
    card_dir: &Path,
    planilha: &Path,
    modelo: &Path,
    legendas: bool,
    desempatar: Desempate,
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

        // Pode haver mais de uma subpasta com o mesmo prefixo de ID (o nome e
        // truncado em 10 chars, entao renomear o teste na planilha cria uma
        // segunda pasta). Nunca escolher "a primeira que o read_dir devolver":
        // isso gravaria a evidencia na pasta errada em silencio.
        let candidatas = achar_subpastas(card_dir, &id);
        let subpasta = match candidatas.len() {
            0 => {
                warn(&format!(
                    "Linha {n}: ID {id} '{nome}' -> subpasta nao encontrada -> pulado."
                ));
                r.avisos += 1;
                continue;
            }
            1 => candidatas.into_iter().next().unwrap_or_default(),
            qtd => {
                warn(&format!(
                    "Linha {n}: ID {id} '{nome}' -> {qtd} subpastas com o prefixo '{id}'."
                ));
                match desempatar(&id, &nome, &candidatas) {
                    Some(p) => {
                        log(&format!("Linha {n}: ID {id} -> usando '{}'.", nome_de(&p)));
                        p
                    }
                    None => {
                        warn(&format!(
                            "Linha {n}: ID {id} '{nome}' -> ambiguidade nao resolvida -> pulado."
                        ));
                        r.avisos += 1;
                        continue;
                    }
                }
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

// ─── Gerar DET PDF (docx -> pdf) ────────────────────────────────────────────

pub fn gerar_pdf(card_dir: &Path) -> Result<usize, String> {
    let docs = coletar_arquivos(card_dir, "docx", Some("DET_"));
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

// ─── Gerar DET compilado (juntar PDFs, limite 30 MB) ────────────────────────

pub fn gerar_compilado(card_dir: &Path, limite_mb: u64) -> Result<Vec<PathBuf>, String> {
    let limite = limite_mb * 1024 * 1024;
    let mut pdfs = coletar_arquivos(card_dir, "pdf", None);
    // ignora arquivos ja compilados
    pdfs.retain(|p| !nome_de(p).to_ascii_lowercase().starts_with("det_compilado"));
    if pdfs.is_empty() {
        return Err("nenhum PDF encontrado nas subpastas (rode 'Gerar DET PDF' antes).".into());
    }

    log(&format!("{} PDF(s) para compilar (limite {} MB).", pdfs.len(), limite_mb));

    // Tentativa unica: junta tudo e comprime.
    let tmp = card_dir.join(".det_merge_tmp.pdf");
    pdf::merge_pdfs(&pdfs, &tmp)?;
    let unico = card_dir.join("DET_Compilado.pdf");
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
        let parte = card_dir.join(format!("DET_Compilado_Parte_{}.pdf", i + 1));
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
        false => fs::copy(entrada, saida)
            .map(|_| ())
            .map_err(|e| format!("nao copiei o PDF: {e}")),
    }
}

/// Agrupa PDFs em partes cujo tamanho somado (bruto) nao passa de `limite`.
/// Um PDF maior que o limite vira uma parte propria (sem divisao por pagina).
/// `pub` para ser exercitado nos testes de integracao.
pub fn bin_pack(pdfs: &[PathBuf], limite: u64) -> Vec<Vec<PathBuf>> {
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

/// Coleta arquivos com a extensao dada nas subpastas de `dir`, em ordem natural
/// (por subpasta e por nome). `prefixo` filtra pelo inicio do nome.
fn coletar_arquivos(dir: &Path, ext: &str, prefixo: Option<&str>) -> Vec<PathBuf> {
    let mut subs: Vec<PathBuf> = match fs::read_dir(dir) {
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

/// Acha **todas** as subpastas cujo prefixo numerico e igual a ID, em ordem
/// natural (deterministica — `read_dir` nao tem ordem garantida).
/// `pub` para ser exercitado nos testes de integracao.
pub fn achar_subpastas(dir: &Path, id: &str) -> Vec<PathBuf> {
    let alvo = id.trim();
    let mut achadas: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir() && prefixo_numerico(nome_de(p)).as_deref() == Some(alvo))
            .collect(),
        Err(_) => Vec::new(),
    };
    achadas.sort_by(|a, b| natural_key(nome_de(a)).cmp(&natural_key(nome_de(b))));
    achadas
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
