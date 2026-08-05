---
name: windows-shell-diagnostics
description: Use esta skill quando houver falha de execução no PowerShell do app, erro de runner no Windows, problema de sandbox, falha em leitura de arquivos, comandos travando, ou necessidade de fallback para PowerShell clássico/cmd em modo somente leitura.
---

# Windows Shell Diagnostics

## Objetivo

Diagnosticar falhas de shell/runner no Windows sem modificar arquivos do projeto.

## Quando usar

Use esta skill quando ocorrer:

- erro no runner do PowerShell do app;
- falha ao listar arquivos;
- falha ao ler conteúdo da pasta;
- erro de sandbox Windows;
- comando PowerShell sem retorno;
- problema causado por profile do PowerShell;
- necessidade de alternar para PowerShell clássico ou cmd.

## Regra principal

Antes de qualquer alteração, executar inspeção somente leitura.

Não executar comandos destrutivos.

Não executar comandos de escrita antes de validar:

- diretório atual;
- shell disponível;
- política de execução;
- status do Git;
- leitura de arquivos;
- permissões do sandbox.

## Scripts oficiais

Executar nesta ordem:

```powershell
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File scripts\diagnostics\windows-shell-health.ps1
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File scripts\diagnostics\windows-readonly-inspection.ps1
cmd.exe /d /s /c scripts\diagnostics\windows-readonly-inspection.cmd
```
