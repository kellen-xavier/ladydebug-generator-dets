---
name: gerar-dets
description: Para cada pasta de \ID NUMERO-DO-TESTE deve criar um arquivo .docx que deve ser o DET do teste. O arquivo modelo base é `\modelos\[DET] ID CARD modelo.docx`. >

---

# Gerar DETs

## O que deve fazer

- Usar modelos/[DET] ID CARD modelo.docx;
- Localizar automaticamente o arquivo base `modelos\[DET] ID CARD modelo.docx` que serve como modelo para copiar o arquivo e anexar em cada pasta `\Releases\Release de Mês Anos\ID CARD NUMERO\ID NUMERO-DO-TESTE`;
- Gerar `[DET] ID NUMERO-ID-TESTE (EDITADO).docx`.
- Não deve realizar alterações já existentes no `\[DET] ID CARD modelo.docx` apenas preencher com as informações necessárias para cada Teste.
- Ler ID, Nome do teste (inicial), Executado por e Status nativo.
- Preencher RESPONSÁVEL EXECUÇÃO: com o nome encontrado na coluna `Executador por` igual esta no arquivo teste_selected_DD_MM_AAAA_HH_MM_SS.xlsx
- Preencher Cenário do Teste: usando os nomes oficiais dos arquivos teste_selected_DD_MM_AAAA_HH_MM_SS.xlsx.
- Inserir somente as imagens que estão diretamente na pasta `Evidências` do ID, abaixo de IMAGENS QUE EVIDENCIAM A AÇÃO, em ordem numérica natural.
- Se a pasta `Evidências` do ID estiver vazia, ignorar esse ID e seguir para o próximo — não gerar DET a partir de imagens soltas fora de `Evidências`.
- Ignorar Status nativo = Ignorado.
- Tratar BUGs como DET separado;
- Nunca gerar scripts hardcoded no código.
- Deve seguir as regras conforme o arquivo `AGENTS.md` no tópico - `## Exemplo de como deve preencher o arquivo [DET] ID CARD modelo.docx`

## Rules

- Nunca criar scripts pontuais dentro da release com IDs no nome. Se a automação precisar mudar, ajustar os scripts genéricos em scripts/. IDs, release e caminhos devem vir por parâmetro ou pela planilha da release.

## Validations

- Conferir que cada DOCX gerado contém responsável, cenário e quantidade esperada de imagens.
