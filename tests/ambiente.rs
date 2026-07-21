//! Funcionalidade "Verificar ambiente": deteccao de ferramentas externas no
//! PATH (usada pelo check de ambiente e pelas acoes 4 e 5).

use gestor_det::pdf;

#[test]
fn ferramenta_inexistente_nao_e_encontrada() {
    assert!(pdf::find_in_path(&["ferramenta_que_nao_existe_xyz_123"]).is_none());
}
