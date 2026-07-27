//! Funcionalidades 1 e 2: descoberta de Release/ID CARD e criacao das
//! subpastas dos testes a partir da planilha.

mod common;

use gestor_det::acoes;
use gestor_det::menu;
use gestor_det::util::hoje_ymd;
use gestor_det::workspace::Workspace;

// ─── BUG 2: ano/mes default da nova release ────────────────────────────────

/// Regressao: o default do ano nao pode ser um literal (era "2026" fixo).
/// Ele tem de acompanhar o relogio, senao em 2027 o usuario cria a release
/// no ano errado por inercia.
#[test]
fn ano_sugerido_acompanha_o_relogio() {
    let (ano_hoje, mes_hoje, _) = hoje_ymd();

    assert!(
        (2000..=2100).contains(&ano_hoje),
        "ano de hoje fora da faixa aceita: {ano_hoje}"
    );
    assert_eq!(menu::validar_ano(&ano_hoje.to_string()), Some(ano_hoje));
    assert_eq!(menu::validar_mes(&mes_hoje.to_string()), Some(mes_hoje));
}

#[test]
fn validar_ano_rejeita_fora_da_faixa_e_lixo() {
    assert_eq!(menu::validar_ano("2026"), Some(2026));
    assert_eq!(menu::validar_ano(" 2027 "), Some(2027));

    assert_eq!(menu::validar_ano("1999"), None, "antes de ANO_MIN");
    assert_eq!(menu::validar_ano("2101"), None, "depois de ANO_MAX");
    assert_eq!(menu::validar_ano("20a6"), None);
    assert_eq!(menu::validar_ano("-2026"), None);
    assert_eq!(menu::validar_ano(""), None);
}

#[test]
fn validar_mes_aceita_1_a_12() {
    assert_eq!(menu::validar_mes("1"), Some(1));
    assert_eq!(menu::validar_mes("12"), Some(12));
    assert_eq!(menu::validar_mes("0"), None);
    assert_eq!(menu::validar_mes("13"), None);
    assert_eq!(menu::validar_mes("julho"), None);
}

#[test]
fn nome_da_release_segue_o_padrao() {
    assert_eq!(menu::montar_nome_release(7, 2026), "Release Julho 2026");
    assert_eq!(menu::montar_nome_release(3, 2027), "Release Marco 2027");
    assert_eq!(menu::montar_nome_release(12, 2025), "Release Dezembro 2025");
}

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
