@echo off
chcp 65001 >nul
cd /d "%~dp0"
title Gerador de DETs
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0menu.ps1"
echo.
echo Pode fechar esta janela.
pause >nul
