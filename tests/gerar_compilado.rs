//! Funcionalidade 5: gerar DET compilado. Cobrimos o agrupamento por tamanho
//! (bin_pack) e o caminho de erro sem depender do qpdf/Ghostscript.

mod common;

use gestor_det::acoes;
use std::path::PathBuf;

#[test]
fn sem_pdf_retorna_erro() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1");
    assert!(acoes::gerar_compilado(&card, 30).is_err());
}

#[test]
fn bin_pack_agrupa_por_limite_de_tamanho() {
    let tw = common::TempWs::nova();
    let mk = |nome: &str, bytes: usize| -> PathBuf {
        let p = tw.join(nome);
        std::fs::write(&p, vec![0u8; bytes]).unwrap();
        p
    };
    let a = mk("a.pdf", 40);
    let b = mk("b.pdf", 40);
    let c = mk("c.pdf", 40);

    // limite 100: [a,b] = 80 cabe; +c = 120 estoura -> fecha grupo, c em novo grupo
    let grupos = acoes::bin_pack(&[a, b, c], 100);
    assert_eq!(grupos.len(), 2);
    assert_eq!(grupos[0].len(), 2);
    assert_eq!(grupos[1].len(), 1);
}
