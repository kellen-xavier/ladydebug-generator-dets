//! Funcionalidades 1 e 2: descoberta de Release/ID CARD e criacao das
//! subpastas dos testes a partir da planilha.

mod common;

use gestor_det::acoes;
use gestor_det::workspace::Workspace;

#[test]
fn lista_releases_e_cards() {
    let tw = common::TempWs::nova();
    let ws = Workspace::detectar(&tw.raiz);

    assert!(ws.listar_releases().is_empty());

    tw.criar_dir("Release/Release Junho 2026");
    tw.criar_dir("Release/Release Julho 2026/ID CARD 1399338");
    tw.criar_dir("Release/Release Julho 2026/ID CARD 1410037");
    tw.criar_dir("Release/Release Julho 2026/pasta-solta"); // nao e card

    assert_eq!(ws.listar_releases().len(), 2);

    let julho = tw.join("Release/Release Julho 2026");
    let cards = ws.listar_cards(&julho);
    assert_eq!(cards.len(), 2); // ignora "pasta-solta"
    assert_eq!(ws.nome_card("1399338"), "ID CARD 1399338");
}

#[test]
fn cria_subpastas_id_nome_e_pula_sem_id() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1399338");
    let xlsx = tw.join("teste_selected_a.xlsx");
    common::escrever_xlsx(
        &xlsx,
        &[
            vec!["ID", "Nome do teste (inicial)", "Executado por", "Status nativo"],
            vec!["13", "Login da Aplicacao", "Fulano", "Aprovado"],
            vec!["", "sem id", "x", "Aprovado"], // sem ID -> pulado
        ],
    );

    let n = acoes::criar_subpastas(&card, &xlsx).unwrap();
    assert_eq!(n, 1);
    // nome truncado em 10 chars e com espacos sanitizados
    assert!(card.join("13 - Login_da_A").is_dir());
}

#[test]
fn subpastas_reexecucao_nao_duplica() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1");
    let xlsx = tw.join("t.xlsx");
    common::escrever_xlsx(
        &xlsx,
        &[
            vec!["ID", "Nome do teste (inicial)", "Executado por", "Status nativo"],
            vec!["13", "Login", "Fulano", "Aprovado"],
        ],
    );
    assert_eq!(acoes::criar_subpastas(&card, &xlsx).unwrap(), 1);
    // segunda vez: ja existe, nao cria de novo
    assert_eq!(acoes::criar_subpastas(&card, &xlsx).unwrap(), 0);
}
