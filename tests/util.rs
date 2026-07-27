//! Testes dos utilitarios usados por todas as funcionalidades:
//! ordenacao natural, normalizacao/sanitizacao, datas e escape XML.

use gestor_det::util::*;

#[test]
fn ordenacao_natural_1_2_10() {
    let mut v = vec!["img_10", "img_2", "img_1", "img_100"];
    v.sort_by(|a, b| natural_key(a).cmp(&natural_key(b)));
    assert_eq!(v, vec!["img_1", "img_2", "img_10", "img_100"]);
}

#[test]
fn normaliza_cabecalho_tolerante() {
    assert_eq!(normalize("  Status  Nativo! "), "status nativo");
    assert_eq!(normalize("Executado por"), "executado por");
    assert_eq!(normalize("Nao-Acao"), "nao acao");
}

#[test]
fn remove_acentos() {
    assert_eq!(strip_accents("Cao Artao"), "Cao Artao");
    assert_eq!(strip_accents("Configuração"), "Configuracao");
}

#[test]
fn sanitiza_nome_de_arquivo() {
    assert_eq!(sanitize("a/b:c*?"), "a_b_c__");
    assert_eq!(sanitize("Login Web"), "Login_Web");
}

#[test]
fn trunca_preservando_utf8() {
    assert_eq!(truncar("Configuracao", 6), "Config");
    assert_eq!(truncar("çãé", 2), "çã");
}

#[test]
fn calendario_e_data() {
    assert_eq!(civil_from_days(18628), (2021, 1, 1));
    let d = data_ddmmaaaa();
    assert_eq!(d.len(), 10);
    assert_eq!(&d[2..3], "/");
    assert_eq!(&d[5..6], "/");
}

// ─── BUG 1: virada de dia por fuso horario ─────────────────────────────────

/// Regressao: DET gerado a noite no Brasil nao pode sair com a data de amanha.
/// 1785112200 = 2026-07-27T00:30:00Z, que em UTC-3 e 26/07/2026 as 21:30.
#[test]
fn data_noturna_no_brasil_nao_avanca_um_dia() {
    const NOITE_26_JUL_BRT: i64 = 1785112200;

    // Com o fuso de Brasilia (o padrao do projeto): ainda e dia 26.
    assert_eq!(ymd_de_epoch(NOITE_26_JUL_BRT, -3), (2026, 7, 26));
    assert_eq!(data_ddmmaaaa_de(NOITE_26_JUL_BRT, -3), "26/07/2026");

    // Em UTC (o comportamento antigo, com offset fixo 0) ja seria dia 27 —
    // este era exatamente o bug.
    assert_eq!(ymd_de_epoch(NOITE_26_JUL_BRT, 0), (2026, 7, 27));
}

#[test]
fn virada_de_dia_no_limite_das_23h59() {
    // 2026-07-27T02:59:59Z = 26/07/2026 23:59:59 em UTC-3 (ultimo segundo do dia)
    assert_eq!(ymd_de_epoch(1785121199, -3), (2026, 7, 26));
    // um segundo depois ja e dia 27 em Brasilia
    assert_eq!(ymd_de_epoch(1785121200, -3), (2026, 7, 27));
}

#[test]
fn virada_de_ano_respeita_o_fuso() {
    // 2026-01-01T01:00:00Z = 31/12/2025 22:00 em UTC-3
    assert_eq!(data_ddmmaaaa_de(1767229200, -3), "31/12/2025");
    assert_eq!(data_ddmmaaaa_de(1767229200, 0), "01/01/2026");
}

#[test]
fn fuso_padrao_e_brasilia() {
    assert_eq!(FUSO_PADRAO_HORAS, -3);
    assert_eq!(fuso_horas(), -3, "sem --fuso, o padrao deve ser -3");
}

#[test]
fn validar_fuso_aceita_faixa_e_rejeita_lixo() {
    assert_eq!(validar_fuso("-3"), Some(-3));
    assert_eq!(validar_fuso("0"), Some(0));
    assert_eq!(validar_fuso("+5"), Some(5));
    assert_eq!(validar_fuso(" -3 "), Some(-3));
    assert_eq!(validar_fuso("14"), Some(14));
    assert_eq!(validar_fuso("-12"), Some(-12));

    assert_eq!(validar_fuso("15"), None, "acima de UTC+14");
    assert_eq!(validar_fuso("-13"), None, "abaixo de UTC-12");
    assert_eq!(validar_fuso("abc"), None);
    assert_eq!(validar_fuso(""), None);
    assert_eq!(validar_fuso("-3.5"), None, "so horas inteiras");
}

#[test]
fn escape_e_unescape_xml() {
    assert_eq!(xml_escape("a<b>&\"'"), "a&lt;b&gt;&amp;&quot;&apos;");
    assert_eq!(xml_unescape("a&lt;b&gt;&amp;"), "a<b>&");
    assert_eq!(xml_unescape("&#65;&#x42;"), "AB");
}
