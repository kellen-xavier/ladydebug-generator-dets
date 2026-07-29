//! Funcionalidade: descompactar os `.zip` de evidencias — extrai as imagens
//! direto na subpasta, apaga o `.zip` e nunca sobrescreve o que ja existe.

mod common;

use gestor_det::acoes;

#[test]
fn extrai_imagens_achatadas_e_apaga_o_zip() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1");
    let sub = tw.criar_dir("Release/Release Julho 2026/ID CARD 1/13 - Login");
    let zip = sub.join("attachments_run_13_06-26-2026.zip");

    let img1 = common::png_falso(400, 300);
    let img2 = common::png_falso(300, 200);
    common::escrever_zip(
        &zip,
        &[
            // com subpasta interna -> deve achatar (usar so o nome do arquivo)
            ("attachments_run_13/Step_1.png", img1.as_slice()),
            ("attachments_run_13/Step_2.png", img2.as_slice()),
            ("attachments_run_13/nota.txt", b"texto".as_slice()),
        ],
    );

    let r = acoes::descompactar_zips(&card).unwrap();
    assert_eq!(r.zips, 1);
    assert_eq!(r.imagens, 2);

    assert!(sub.join("Step_1.png").is_file(), "imagem achatada na subpasta");
    assert!(sub.join("Step_2.png").is_file());
    assert!(!sub.join("nota.txt").exists(), "so imagens: nao extrai .txt");
    assert!(!zip.exists(), "o .zip deve ser apagado apos extrair");
}

#[test]
fn nao_sobrescreve_imagem_existente() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1");
    let sub = tw.criar_dir("Release/Release Julho 2026/ID CARD 1/13 - Login");
    // evidencia que ja existe e NAO pode ser sobrescrita
    std::fs::write(sub.join("Step_1.png"), b"ORIGINAL").unwrap();

    let zip = sub.join("attachments_run_13.zip");
    let nova = common::png_falso(400, 300);
    common::escrever_zip(&zip, &[("Step_1.png", nova.as_slice())]);

    let r = acoes::descompactar_zips(&card).unwrap();
    assert_eq!(r.mantidas, 1);
    assert_eq!(std::fs::read(sub.join("Step_1.png")).unwrap(), b"ORIGINAL".to_vec());
    assert!(!zip.exists());
}

#[test]
fn zip_sem_imagens_e_mantido() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1");
    let sub = tw.criar_dir("Release/Release Julho 2026/ID CARD 1/13 - Login");
    let zip = sub.join("attachments_run_13.zip");
    common::escrever_zip(&zip, &[("leia-me.txt", b"nada de imagem".as_slice())]);

    let r = acoes::descompactar_zips(&card).unwrap();
    assert_eq!(r.zips, 0);
    assert_eq!(r.imagens, 0);
    assert!(zip.exists(), "zip sem imagem nao deve ser apagado");
}

#[test]
fn sem_zip_retorna_erro() {
    let tw = common::TempWs::nova();
    let card = tw.criar_dir("Release/Release Julho 2026/ID CARD 1");
    tw.criar_dir("Release/Release Julho 2026/ID CARD 1/13 - Login");
    assert!(acoes::descompactar_zips(&card).is_err());
}
