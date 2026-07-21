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

#[test]
fn escape_e_unescape_xml() {
    assert_eq!(xml_escape("a<b>&\"'"), "a&lt;b&gt;&amp;&quot;&apos;");
    assert_eq!(xml_unescape("a&lt;b&gt;&amp;"), "a<b>&");
    assert_eq!(xml_unescape("&#65;&#x42;"), "AB");
}
