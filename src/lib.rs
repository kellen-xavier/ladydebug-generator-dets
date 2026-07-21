//! Biblioteca do Gestor de DET.
//!
//! Expoe os modulos para o binario (`src/main.rs`) e para os testes de
//! integracao (`tests/`). O binario e apenas a casca de terminal; toda a
//! logica testavel (planilha, docx, acoes, util) vive aqui.

pub mod acoes;
pub mod docx;
pub mod menu;
pub mod pdf;
pub mod util;
pub mod workspace;
pub mod xlsx;
