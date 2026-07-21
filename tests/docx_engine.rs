//! Funcionalidade: motor de .docx — preenchimento de tokens (com escape) e
//! insercao de imagens no ponto {{EVIDENCIAS}} (nucleo da acao 3).

mod common;

use gestor_det::docx::Docx;

#[test]
fn preenche_tokens_e_escapa_texto() {
    let ws = common::TempWs::nova();
    let modelo = ws.join("modelo.docx");
    common::escrever_template(
        &modelo,
        &[
            "ID: {{ID}}",
            "Nome: {{NOME_TESTE}}",
            "Fixo: Tom &amp; Jerry {{STATUS}}",
            "{{EVIDENCIAS}}",
        ],
    );

    let mut doc = Docx::read(&modelo).unwrap();
    doc.fill_placeholders(&[
        ("{{ID}}", "13"),
        ("{{NOME_TESTE}}", "Login & Logout"),
        ("{{STATUS}}", "Aprovado"),
    ]);
    // sem imagens: o paragrafo {{EVIDENCIAS}} e removido
    assert_eq!(doc.insert_images(&[], false).unwrap(), 0);

    let saida = ws.join("saida.docx");
    doc.write(&saida).unwrap();
    let xml = common::ler_entrada_zip(&saida, "word/document.xml");

    assert!(xml.contains("13"));
    // valor com & e escapado
    assert!(xml.contains("Login &amp; Logout"));
    // texto fixo do modelo continua escapado (regressao do bug de escape)
    assert!(xml.contains("Tom &amp; Jerry"));
    // nenhum token sobrou
    assert!(!xml.contains("{{"));
}

#[test]
fn insere_imagens_no_ponto_evidencias() {
    let ws = common::TempWs::nova();
    let modelo = ws.join("m.docx");
    common::escrever_template(&modelo, &["Cabecalho", "{{EVIDENCIAS}}"]);

    let img1 = ws.join("01.png");
    let img2 = ws.join("02.png");
    std::fs::write(&img1, common::png_falso(400, 300)).unwrap();
    std::fs::write(&img2, common::png_falso(900, 700)).unwrap();

    let mut doc = Docx::read(&modelo).unwrap();
    assert_eq!(doc.insert_images(&[img1, img2], false).unwrap(), 2);

    let saida = ws.join("s.docx");
    doc.write(&saida).unwrap();

    let midia = common::nomes_zip(&saida)
        .into_iter()
        .filter(|n| n.starts_with("word/media/"))
        .count();
    assert_eq!(midia, 2);

    let rels = common::ler_entrada_zip(&saida, "word/_rels/document.xml.rels");
    assert_eq!(rels.matches("/image").count(), 2);
}

#[test]
fn modelo_sem_document_xml_falha() {
    let ws = common::TempWs::nova();
    let falso = ws.join("nao_e_docx.docx");
    std::fs::write(&falso, b"conteudo qualquer").unwrap();
    assert!(Docx::read(&falso).is_err());
}
