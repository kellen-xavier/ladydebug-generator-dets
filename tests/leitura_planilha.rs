//! Funcionalidade: leitura da planilha .xlsx e mapeamento dos cabecalhos
//! obrigatorios (usada pelas acoes 2 e 3).

mod common;

use gestor_det::workspace::Workspace;
use gestor_det::xlsx::{read_xlsx, HeaderMap};

// ─── Descoberta automatica da planilha dentro da pasta do card ─────────────

/// Cria um card e devolve (workspace, caminho_do_card).
fn card_de_teste(tw: &common::TempWs) -> (Workspace, std::path::PathBuf) {
    let ws = Workspace::detectar(&tw.raiz);
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1");
    (ws, card)
}

#[test]
fn acha_planilha_dentro_da_pasta_do_card() {
    let tw = common::TempWs::nova();
    let (ws, card) = card_de_teste(&tw);
    common::escrever_xlsx(&card.join("teste_selected_x.xlsx"), &[vec!["ID"], vec!["13"]]);

    let achada = ws.achar_planilha_em(&card).expect("deveria achar a planilha");
    assert_eq!(
        achada.file_name().unwrap().to_string_lossy(),
        "teste_selected_x.xlsx"
    );
}

#[test]
fn pasta_sem_xlsx_devolve_none() {
    let tw = common::TempWs::nova();
    let (ws, card) = card_de_teste(&tw);
    assert!(ws.achar_planilha_em(&card).is_none());
}

#[test]
fn prefere_teste_selected_sobre_outro_xlsx() {
    let tw = common::TempWs::nova();
    let (ws, card) = card_de_teste(&tw);
    common::escrever_xlsx(&card.join("relatorio.xlsx"), &[vec!["X"]]);
    // nome com prefixo antes de "teste_selected" tambem deve ser reconhecido
    common::escrever_xlsx(&card.join("execucoes teste_selected_30_06.xlsx"), &[vec!["ID"]]);

    let achada = ws.achar_planilha_em(&card).unwrap();
    assert!(achada
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("teste_selected"));
}

#[test]
fn um_unico_xlsx_sem_teste_selected_ainda_e_usado() {
    let tw = common::TempWs::nova();
    let (ws, card) = card_de_teste(&tw);
    common::escrever_xlsx(&card.join("planilha_dados.xlsx"), &[vec!["ID"]]);
    assert!(ws.achar_planilha_em(&card).is_some());
}

#[test]
fn varios_xlsx_sem_teste_selected_e_ambiguo_none() {
    let tw = common::TempWs::nova();
    let (ws, card) = card_de_teste(&tw);
    common::escrever_xlsx(&card.join("a.xlsx"), &[vec!["ID"]]);
    common::escrever_xlsx(&card.join("b.xlsx"), &[vec!["ID"]]);
    // ambiguo: sem "teste_selected" e mais de um -> deixa o usuario escolher
    assert!(ws.achar_planilha_em(&card).is_none());
}

#[test]
fn le_linhas_e_mapeia_cabecalhos_tolerantes() {
    let ws = common::TempWs::nova();
    let xlsx = ws.join("teste_selected_x.xlsx");
    common::escrever_xlsx(
        &xlsx,
        &[
            vec!["ID", "Nome do teste (inicial)", "Executador por", "Status nativo"],
            vec!["13", "Login", "Fulano", "Aprovado"],
            vec!["20", "Logout", "Beltrano", "Ignorado"],
        ],
    );
    let linhas = read_xlsx(&xlsx).unwrap();
    assert_eq!(linhas.len(), 3);

    let col = HeaderMap::from_headers(&linhas[0]).unwrap();
    assert_eq!(col.get(&linhas[1], col.id), "13");
    assert_eq!(col.get(&linhas[1], col.nome), "Login");
    // "Executador por" e um alias aceito para o executor
    assert_eq!(col.get(&linhas[1], col.executor), "Fulano");
    assert_eq!(col.get(&linhas[2], col.status), "Ignorado");
}

#[test]
fn texto_com_e_comercial_preservado() {
    let ws = common::TempWs::nova();
    let xlsx = ws.join("t.xlsx");
    common::escrever_xlsx(
        &xlsx,
        &[vec!["ID", "Nome do teste"], vec!["7", "Login & Logout"]],
    );
    let linhas = read_xlsx(&xlsx).unwrap();
    assert_eq!(linhas[1][1], "Login & Logout");
}

#[test]
fn cabecalho_incompleto_da_erro() {
    let ws = common::TempWs::nova();
    let xlsx = ws.join("t.xlsx");
    // faltam as colunas de executor e status
    common::escrever_xlsx(&xlsx, &[vec!["ID", "Nome do teste"]]);
    let linhas = read_xlsx(&xlsx).unwrap();
    assert!(HeaderMap::from_headers(&linhas[0]).is_err());
}
