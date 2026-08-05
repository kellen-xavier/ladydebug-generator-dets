---
name: descompactar-attachments-run
description: Descompacta arquivos attachments_run_ID-NUMERO-DO-TESTE_MM-DD-AAAA_HH-MM-SS.zip dentro das pastas Evidências de cada ID, move as imagens extraídas para a própria pasta Evidências e remove a pasta extraída e o zip original. Use quando o usuário pedir para descompactar anexos, extrair evidências de zip, processar attachments_run ou somente preparar imagens nas pastas de evidências.
---

# Descompactar Attachments Run

## Objetivo

Executar somente a etapa de descompactação dos arquivos `attachments_run_ID-NUMERO-DO-TESTE_MM-DD-AAAA_HH-MM-SS.zip` nas pastas de evidências.

Não gerar DET, não converter DOCX para PDF e não consolidar PDFs quando o pedido for apenas descompactar `.zip`.

## Estrutura Esperada

```txt
ID NUMERO-DO-TESTE/
  Evidências/
    attachments_run_ID-NUMERO-DO-TESTE_MM-DD-AAAA_HH-MM-SS.zip
```

Exemplo:

```txt
ID 224517/
  Evidências/
    attachments_run_224517_06-26-2026_17-41-41.zip
```

## Regras

- Seguir sempre as regras do `AGENTS.md`.
- Processar somente pastas cujo nome comece com `ID`.
- Procurar o `.zip` dentro da pasta `Evidências` de cada ID.
- Aceitar somente o padrão `attachments_run_ID-NUMERO-DO-TESTE_MM-DD-AAAA_HH-MM-SS.zip`.
- Descompactar o `.zip` dentro de `Evidências`.
- Deixar todas as imagens diretamente em `Evidências`.
- Apagar a pasta extraída `attachments_run_ID-NUMERO-DO-TESTE_MM-DD-AAAA_HH-MM-SS` somente depois de confirmar que as imagens foram movidas para `Evidências`.
- Apagar o `.zip` original somente depois de confirmar que as imagens foram extraídas para `Evidências`.
- Não alterar nomes de imagens.
- Não processar imagens de outras subpastas, exceto quando a regra geral do `AGENTS.md` permitir `BUGs`.

## Passos

1. Identificar a pasta da release ou a raiz informada pelo usuário.
2. Listar as pastas `ID *` em ordem numérica natural.
3. Entrar em `ID NUMERO-DO-TESTE\Evidências`.
4. Procurar arquivos `.zip` no padrão `attachments_run_ID-NUMERO-DO-TESTE_MM-DD-AAAA_HH-MM-SS.zip`.
5. Descompactar cada `.zip` encontrado.
6. Mover as imagens extraídas para a própria pasta `Evidências`.
7. Confirmar que as imagens existem em `Evidências`.
8. Remover a pasta extraída `attachments_run_ID-NUMERO-DO-TESTE_MM-DD-AAAA_HH-MM-SS`.
9. Remover o `.zip` original.
10. Entregar uma lista com IDs processados, zips extraídos e IDs sem zip.

## Validação

- Confirmar que as imagens ficaram diretamente em `Evidências`.
- Confirmar que a pasta extraída `attachments_run_ID-NUMERO-DO-TESTE_MM-DD-AAAA_HH-MM-SS` foi removida.
- Confirmar que o `.zip` original foi removido.
- Confirmar que nenhuma etapa de DET ou PDF foi executada quando o pedido era somente descompactar `.zip`.
