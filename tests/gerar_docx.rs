//! Funcionalidade 3: gerar o DET .docx por ID (fim a fim), pulando os testes
//! com Status nativo = Ignorado.

mod common;

use gestor_det::acoes;

#[test]
fn gera_det_por_id_e_pula_ignorado() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1399338");
    let sub = tw.criar_dir("Release/Release Julho 2026/ID CARD 1399338/13 - Login");
    std::fs::write(sub.join("01.png"), common::png_falso(400, 300)).unwrap();
    std::fs::write(sub.join("02.png"), common::png_falso(300, 200)).unwrap();

    let modelo = tw.join("modelo.docx");
    common::escrever_template(
        &modelo,
        &[
            "ID: {{ID}}",
            "Nome: {{NOME_TESTE}}",
            "Por: {{EXECUTADO_POR}}",
            "Status: {{STATUS}}",
            "Data: {{DATA}}",
            "{{EVIDENCIAS}}",
        ],
    );

    let xlsx = tw.join("teste_selected_a.xlsx");
    common::escrever_xlsx(
        &xlsx,
        &[
            vec!["ID", "Nome do teste (inicial)", "Executado por", "Status nativo"],
            vec!["13", "Login", "Fulano", "Aprovado"],
            vec!["20", "Logout", "Beltrano", "Ignorado"],
        ],
    );

    let r = acoes::gerar_docx(&card, &xlsx, &modelo, false).unwrap();
    assert_eq!(r.gerados, 1);
    assert_eq!(r.ignorados, 1);

    let det = sub.join("DET_13_Login.docx");
    assert!(det.is_file());
    let xml = common::ler_entrada_zip(&det, "word/document.xml");
    assert!(xml.contains("Login"));
    assert!(xml.contains("Fulano"));
    assert!(xml.contains("Aprovado"));
    assert!(!xml.contains("{{"));

    let midia = common::nomes_zip(&det)
        .into_iter()
        .filter(|n| n.starts_with("word/media/"))
        .count();
    assert_eq!(midia, 2);
}

#[test]
fn id_sem_subpasta_gera_aviso_e_nao_falha() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1");
    let modelo = tw.join("m.docx");
    common::escrever_template(&modelo, &["ID: {{ID}}", "{{EVIDENCIAS}}"]);
    let xlsx = tw.join("t.xlsx");
    common::escrever_xlsx(
        &xlsx,
        &[
            vec!["ID", "Nome do teste (inicial)", "Executado por", "Status nativo"],
            vec!["99", "Sem Pasta", "Fulano", "Aprovado"],
        ],
    );
    let r = acoes::gerar_docx(&card, &xlsx, &modelo, false).unwrap();
    assert_eq!(r.gerados, 0);
    assert_eq!(r.avisos, 1);
}
