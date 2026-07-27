//! Utilitarios compartilhados: logging, ordenacao natural, normalizacao de
//! texto, datas e helpers de string/XML. Sem dependencias alem da std.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// ─── Logging (ASCII, seguro em qualquer console) ───────────────────────────

pub fn log(msg: &str) {
    println!("[INFO] {msg}");
}
pub fn ok(msg: &str) {
    println!("[ OK ] {msg}");
}
pub fn warn(msg: &str) {
    eprintln!("[WARN] {msg}");
}
pub fn erro(msg: &str) {
    eprintln!("[ERR ] {msg}");
}

// ─── Ordenacao natural ─────────────────────────────────────────────────────

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Parte {
    Num(u64),
    Txt(String),
}

/// Chave de ordenacao natural: separa blocos numericos e textuais.
/// ["img_10","img_2","img_1"] -> ["img_1","img_2","img_10"].
pub fn natural_key(s: &str) -> Vec<Parte> {
    let mut partes = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            let mut num = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            partes.push(Parte::Num(num.parse().unwrap_or(0)));
        } else {
            let mut txt = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    break;
                }
                txt.push(d.to_ascii_lowercase());
                chars.next();
            }
            partes.push(Parte::Txt(txt));
        }
    }
    partes
}

// ─── Normalizacao / sanitizacao de texto ───────────────────────────────────

/// Normaliza para comparacao: minusculas, sem acento, espacos colapsados,
/// pontuacao removida.
pub fn normalize(s: &str) -> String {
    let sem_acento = strip_accents(s);
    let mut out = String::new();
    let mut espaco_pendente = false;
    for c in sem_acento.chars() {
        if c.is_alphanumeric() {
            if espaco_pendente && !out.is_empty() {
                out.push(' ');
            }
            espaco_pendente = false;
            out.extend(c.to_lowercase());
        } else {
            espaco_pendente = true;
        }
    }
    out
}

pub fn strip_accents(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'ã' | 'â' | 'ä' | 'Á' | 'À' | 'Ã' | 'Â' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'õ' | 'ô' | 'ö' | 'Ó' | 'Ò' | 'Õ' | 'Ô' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            'ñ' | 'Ñ' => 'n',
            outro => outro,
        })
        .collect()
}

/// Sanitiza para uso em nome de arquivo/pasta (remove caracteres proibidos).
pub fn sanitize(s: &str) -> String {
    let s: String = s
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();
    s.trim().replace(' ', "_")
}

/// Trunca por caracteres (nao bytes), preservando UTF-8. Ex.: nome ate 10.
pub fn truncar(s: &str, n: usize) -> String {
    s.chars().take(n).collect::<String>().trim().to_string()
}

// ─── Calendario ────────────────────────────────────────────────────────────

const MESES: [&str; 12] = [
    "Janeiro", "Fevereiro", "Marco", "Abril", "Maio", "Junho", "Julho",
    "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
];

// ─── Fuso horario ──────────────────────────────────────────────────────────
//
// A std nao expoe o fuso do sistema e o projeto nao usa crates de data. O fuso
// e um valor global em horas, com padrao -3 (Horario de Brasilia) e ajustavel
// pela flag `--fuso <horas>` do binario. Sem isso a data cairia em UTC e todo
// DET gerado depois das 21h no Brasil sairia com a data do dia seguinte.

/// Fuso padrao: -3 = Horario de Brasilia (o publico-alvo da ferramenta).
pub const FUSO_PADRAO_HORAS: i64 = -3;

/// Faixa aceita para o fuso, em horas (UTC-12 a UTC+14).
pub const FUSO_MIN_HORAS: i64 = -12;
pub const FUSO_MAX_HORAS: i64 = 14;

static FUSO_HORAS: AtomicI64 = AtomicI64::new(FUSO_PADRAO_HORAS);

/// Define o fuso usado por `hoje_ymd`/`data_ddmmaaaa`. Chamado uma vez na
/// inicializacao do binario (a partir de `--fuso`).
pub fn definir_fuso(horas: i64) {
    FUSO_HORAS.store(horas, Ordering::Relaxed);
}

/// Fuso atual, em horas.
pub fn fuso_horas() -> i64 {
    FUSO_HORAS.load(Ordering::Relaxed)
}

/// Valida um fuso vindo da linha de comando ("-3", "+5", "0").
pub fn validar_fuso(s: &str) -> Option<i64> {
    let t = s.trim().trim_start_matches('+');
    let h: i64 = t.parse().ok()?;
    if (FUSO_MIN_HORAS..=FUSO_MAX_HORAS).contains(&h) {
        Some(h)
    } else {
        None
    }
}

/// Offset local em segundos, derivado do fuso configurado.
fn offset_local() -> i64 {
    fuso_horas() * 3600
}

// ─── Datas ─────────────────────────────────────────────────────────────────

/// (ano, mes, dia) para um instante epoch (UTC) num dado fuso, em horas.
/// Funcao pura — e o ponto de teste da regra de virada de dia.
pub fn ymd_de_epoch(secs_utc: i64, fuso: i64) -> (i64, u32, u32) {
    let local = secs_utc + fuso * 3600;
    civil_from_days(local.div_euclid(86400))
}

/// DD/MM/AAAA para um instante epoch (UTC) num dado fuso. Funcao pura.
pub fn data_ddmmaaaa_de(secs_utc: i64, fuso: i64) -> String {
    let (a, m, d) = ymd_de_epoch(secs_utc, fuso);
    format!("{d:02}/{m:02}/{a:04}")
}

/// Segundos desde o epoch (UTC), agora.
fn agora_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// (ano, mes, dia) local, usando o fuso configurado.
pub fn hoje_ymd() -> (i64, u32, u32) {
    civil_from_days((agora_epoch() + offset_local()).div_euclid(86400))
}

/// Data local em DD/MM/AAAA, usando o fuso configurado.
pub fn data_ddmmaaaa() -> String {
    let (a, m, d) = hoje_ymd();
    format!("{d:02}/{m:02}/{a:04}")
}

/// Nome do mes em portugues (sem acento, seguro para nome de pasta).
pub fn nome_mes(m: u32) -> &'static str {
    MESES.get((m as usize).wrapping_sub(1)).copied().unwrap_or("Mes")
}

/// Dias desde 1970-01-01 -> (ano, mes, dia). Algoritmo de Howard Hinnant.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// ─── Helpers de busca em bytes ─────────────────────────────────────────────

/// Encontra `needle` em `hay` a partir de `from`; devolve o indice absoluto.
pub fn find_from(hay: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from > hay.len() || needle.is_empty() {
        return None;
    }
    hay[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

/// Itera blocos delimitados por `abre`(prefixo, sem '>')..`fecha`, incluindo o
/// conteudo entre a abertura e o fechamento. `vazio` cobre a forma self-closing.
pub fn iter_blocos(xml: &str, abre: &str, fecha: &str, vazio: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = xml.as_bytes();
    let ab = abre.as_bytes();
    let mut i = 0;
    while let Some(p) = find_from(bytes, i, ab) {
        let after = p + ab.len();
        let c = bytes.get(after).copied().unwrap_or(b' ');
        if c != b'>' && c != b' ' && c != b'/' {
            i = after;
            continue;
        }
        let gt = match find_from(bytes, p, b">") {
            Some(g) => g,
            None => break,
        };
        if bytes.get(gt - 1) == Some(&b'/') {
            out.push(vazio.to_string());
            i = gt + 1;
            continue;
        }
        let fim = match find_from(bytes, gt + 1, fecha.as_bytes()) {
            Some(f) => f,
            None => break,
        };
        out.push(xml[p..fim].to_string());
        i = fim + fecha.len();
    }
    out
}

/// Percorre todas as tags `<TAG ...>...</TAG>` e devolve os textos (unescaped).
/// `tag` = "t" (xlsx) ou "w:t" (docx).
pub fn iter_text_of(trecho: &str, tag: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = trecho.as_bytes();
    let abre = format!("<{tag}");
    let fecha = format!("</{tag}>");
    let ab = abre.as_bytes();
    let fc = fecha.as_bytes();
    let mut i = 0;
    while let Some(p) = find_from(bytes, i, ab) {
        let after = p + ab.len();
        let c = bytes.get(after).copied().unwrap_or(b' ');
        if c != b'>' && c != b' ' && c != b'/' {
            i = after;
            continue;
        }
        let gt = match find_from(bytes, p, b">") {
            Some(g) => g,
            None => break,
        };
        if bytes.get(gt - 1) == Some(&b'/') {
            i = gt + 1;
            out.push(String::new());
            continue;
        }
        let fim = match find_from(bytes, gt + 1, fc) {
            Some(f) => f,
            None => break,
        };
        out.push(xml_unescape(&trecho[gt + 1..fim]));
        i = fim + fc.len();
    }
    out
}

/// Valor de um atributo da tag de abertura contida em `tag`.
pub fn attr(tag: &str, nome: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let alvo = format!("{nome}=\"");
    let p = find_from(bytes, 0, alvo.as_bytes())?;
    let ini = p + alvo.len();
    let fim = find_from(bytes, ini, b"\"")?;
    Some(tag[ini..fim].to_string())
}

/// Conteudo interno da primeira tag `<nome ...>...</nome>` em `s`.
pub fn inner_tag(s: &str, nome: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let abre = format!("<{nome}");
    let p = find_from(bytes, 0, abre.as_bytes())?;
    let gt = find_from(bytes, p, b">")?;
    if bytes.get(gt - 1) == Some(&b'/') {
        return Some(String::new());
    }
    let fecha = format!("</{nome}>");
    let fim = find_from(bytes, gt + 1, fecha.as_bytes())?;
    Some(s[gt + 1..fim].to_string())
}

pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

pub fn xml_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some(fim) = find_from(bytes, i, b";") {
                let ent = &s[i + 1..fim];
                let sub = match ent {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    _ if ent.starts_with("#x") || ent.starts_with("#X") => {
                        u32::from_str_radix(&ent[2..], 16).ok().and_then(char::from_u32)
                    }
                    _ if ent.starts_with('#') => {
                        ent[1..].parse::<u32>().ok().and_then(char::from_u32)
                    }
                    _ => None,
                };
                if let Some(c) = sub {
                    out.push(c);
                    i = fim + 1;
                    continue;
                }
            }
        }
        let ch_len = utf8_len(bytes[i]);
        out.push_str(&s[i..i + ch_len]);
        i += ch_len;
    }
    out
}

fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else {
        4
    }
}
