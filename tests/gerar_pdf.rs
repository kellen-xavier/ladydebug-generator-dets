//! Funcionalidade 4: gerar DET PDF. Aqui cobrimos o caminho deterministico
//! (sem depender do LibreOffice): quando nao ha .docx, deve retornar erro claro.

mod common;

use gestor_det::acoes;

#[test]
fn sem_docx_retorna_erro() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1");
    tw.criar_dir("Release/Release Julho 2026/ID CARD 1/13 - Login"); // subpasta vazia
    let res = acoes::gerar_pdf(&card);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("nenhum DET"));
}
