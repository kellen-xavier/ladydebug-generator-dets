//! Funcionalidade: leitura da planilha .xlsx e mapeamento dos cabecalhos
//! obrigatorios (usada pelas acoes 2 e 3).

mod common;

use gestor_det::xlsx::{read_xlsx, HeaderMap};

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
