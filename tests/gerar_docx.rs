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

    let r = acoes::gerar_docx(&card, &xlsx, &modelo, false, &acoes::sem_desempate).unwrap();
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
    let r = acoes::gerar_docx(&card, &xlsx, &modelo, false, &acoes::sem_desempate).unwrap();
    assert_eq!(r.gerados, 0);
    assert_eq!(r.avisos, 1);
}

// ─── BUG 3: subpastas ambiguas com o mesmo prefixo de ID ───────────────────

/// Monta um card com duas subpastas para o mesmo ID 13 (cenario real: o nome
/// do teste mudou na planilha e o truncamento em 10 chars gerou outra pasta).
/// Cada pasta recebe um numero diferente de imagens para dar pra provar em
/// qual delas o DET foi gravado.
fn card_com_id_13_duplicado(tw: &common::TempWs) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let base = "Release/Release Julho 2026/ID CARD 1399338";
    let card = tw.criar_dir(base);
    let antiga = tw.criar_dir(&format!("{base}/13 - Login_da_A"));
    let nova = tw.criar_dir(&format!("{base}/13 - Login_de_U"));
    std::fs::write(antiga.join("01.png"), common::png_falso(400, 300)).unwrap();
    std::fs::write(nova.join("01.png"), common::png_falso(400, 300)).unwrap();
    std::fs::write(nova.join("02.png"), common::png_falso(300, 200)).unwrap();

    let modelo = tw.join("m.docx");
    common::escrever_template(&modelo, &["ID: {{ID}}", "{{EVIDENCIAS}}"]);
    let xlsx = tw.join("t.xlsx");
    common::escrever_xlsx(
        &xlsx,
        &[
            vec!["ID", "Nome do teste (inicial)", "Executado por", "Status nativo"],
            vec!["13", "Login de Usuario", "Fulano", "Aprovado"],
        ],
    );
    (card, modelo, xlsx)
}

#[test]
fn achar_subpastas_devolve_todas_em_ordem_natural() {
    let tw = common::TempWs::nova();
    let (card, _, _) = card_com_id_13_duplicado(&tw);

    let achadas = acoes::achar_subpastas(&card, "13");
    assert_eq!(achadas.len(), 2, "as duas pastas com prefixo 13 devem aparecer");

    // Ordem natural = deterministica, independente do que o read_dir devolveu.
    let nomes: Vec<String> = achadas
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(nomes, vec!["13 - Login_da_A", "13 - Login_de_U"]);
}

#[test]
fn prefixo_de_id_nao_casa_por_engano() {
    let tw = common::TempWs::nova();
    let base = "Release/Release Julho 2026/ID CARD 1";
    let card = tw.criar_dir(base);
    tw.criar_dir(&format!("{base}/13 - Login"));
    tw.criar_dir(&format!("{base}/130 - Outro"));
    tw.criar_dir(&format!("{base}/1 - Primeiro"));

    // "13" nao pode casar com "130" nem com "1".
    assert_eq!(acoes::achar_subpastas(&card, "13").len(), 1);
    assert_eq!(acoes::achar_subpastas(&card, "130").len(), 1);
    assert_eq!(acoes::achar_subpastas(&card, "1").len(), 1);
    assert_eq!(acoes::achar_subpastas(&card, "99").len(), 0);
}

/// Regressao principal do BUG 3: sem desempate, o ID ambiguo e PULADO com
/// aviso — nunca gravado numa pasta escolhida ao acaso pelo read_dir.
#[test]
fn id_ambiguo_sem_desempate_e_pulado_e_nao_grava_nada() {
    let tw = common::TempWs::nova();
    let (card, modelo, xlsx) = card_com_id_13_duplicado(&tw);

    let r = acoes::gerar_docx(&card, &xlsx, &modelo, false, &acoes::sem_desempate).unwrap();
    assert_eq!(r.gerados, 0, "nao pode gerar DET com o ID ambiguo");
    assert_eq!(r.avisos, 1, "tem de avisar o usuario");

    // Nenhuma das duas pastas pode ter recebido um DET.
    for sub in ["13 - Login_da_A", "13 - Login_de_U"] {
        let achou = std::fs::read_dir(card.join(sub))
            .unwrap()
            .flatten()
            .any(|e| e.file_name().to_string_lossy().starts_with("DET_"));
        assert!(!achou, "DET gravado em '{sub}' sem o usuario escolher");
    }
}

/// Com desempate, o DET vai exatamente para a pasta escolhida — e so para ela.
#[test]
fn id_ambiguo_com_desempate_grava_na_pasta_escolhida() {
    let tw = common::TempWs::nova();
    let (card, modelo, xlsx) = card_com_id_13_duplicado(&tw);
    let escolhida = card.join("13 - Login_de_U");

    let alvo = escolhida.clone();
    let desempatar = move |id: &str, _nome: &str, cands: &[std::path::PathBuf]| {
        assert_eq!(id, "13");
        assert_eq!(cands.len(), 2, "o callback recebe todas as candidatas");
        Some(alvo.clone())
    };

    let r = acoes::gerar_docx(&card, &xlsx, &modelo, false, &desempatar).unwrap();
    assert_eq!(r.gerados, 1);
    assert_eq!(r.avisos, 0);

    // O DET esta na escolhida...
    let det = escolhida.join("DET_13_Login_de_Usuario.docx");
    assert!(det.is_file(), "DET nao foi gravado na pasta escolhida");
    // ...com as 2 imagens dela (a outra pasta tem 1 — prova que nao trocou).
    let midia = common::nomes_zip(&det)
        .into_iter()
        .filter(|n| n.starts_with("word/media/"))
        .count();
    assert_eq!(midia, 2);

    // ...e a outra pasta continua intacta.
    let sobrou = std::fs::read_dir(card.join("13 - Login_da_A"))
        .unwrap()
        .flatten()
        .any(|e| e.file_name().to_string_lossy().starts_with("DET_"));
    assert!(!sobrou, "DET vazou para a pasta nao escolhida");
}
