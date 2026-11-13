mod common;

use common::TempWs;
use std::process::Command;

#[test]
fn app_lista_e_cria_contexto_sem_interacao() {
    let ws = TempWs::nova();
    let bin = env!("CARGO_BIN_EXE_det-api");
    let snap = Command::new(bin)
        .args(["snapshot", ws.raiz.to_str().unwrap(), "-3"])
        .output()
        .unwrap();
    assert!(snap.status.success());
    assert!(String::from_utf8_lossy(&snap.stdout).contains("\"releases\":[]"));

    let criado = Command::new(bin)
        .args([
            "contexto",
            ws.raiz.to_str().unwrap(),
            "-3",
            "Release Novembro 2026",
            "123",
        ])
        .output()
        .unwrap();
    assert!(
        criado.status.success(),
        "{}",
        String::from_utf8_lossy(&criado.stderr)
    );
    assert!(ws
        .join("Release/Release Novembro 2026/ID CARD 123")
        .is_dir());
    assert!(String::from_utf8_lossy(&criado.stdout).contains("\"ok\":true"));
}

#[test]
fn app_organiza_e_gera_docx_com_o_motor_existente() {
    let ws = TempWs::nova();
    let bin = env!("CARGO_BIN_EXE_det-api");
    let base = ws.raiz.to_str().unwrap();
    let card = ws.criar_dir("Release/Release Novembro 2026/ID CARD 123");
    let xlsx = ws.join("teste_selected_casos.xlsx");
    common::escrever_xlsx(
        &xlsx,
        &[
            vec![
                "ID",
                "Nome do teste (inicial)",
                "Executado por",
                "Status nativo",
            ],
            vec!["13", "Login", "Ana", "Aprovado"],
        ],
    );
    let modelo = ws.join("modelo.docx");
    common::escrever_template(
        &modelo,
        &[
            "{{ID}}",
            "{{NOME_TESTE}}",
            "{{EXECUTADO_POR}}",
            "{{EVIDENCIAS}}",
        ],
    );
    let organizar = Command::new(bin)
        .args([
            "organizar",
            base,
            "-3",
            "Release Novembro 2026",
            "123",
            xlsx.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(organizar.status.success());
    assert!(card.join("13 - Login").is_dir());
    let gerar = Command::new(bin)
        .args([
            "docx",
            base,
            "-3",
            "Release Novembro 2026",
            "123",
            xlsx.to_str().unwrap(),
            modelo.to_str().unwrap(),
            "false",
        ])
        .output()
        .unwrap();
    assert!(
        gerar.status.success(),
        "{}",
        String::from_utf8_lossy(&gerar.stdout)
    );
    let saida = card.join("13 - Login/DET_13_Login.docx");
    assert!(saida.is_file());
    assert!(common::ler_entrada_zip(&saida, "word/document.xml").contains("Ana"));
}
