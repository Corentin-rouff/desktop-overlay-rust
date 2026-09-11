@echo off
setlocal
cd /d "%~dp0"
cargo build --release
if errorlevel 1 (
    echo.
    echo ECHEC DE COMPILATION
    pause
    exit /b 1
)
echo.
echo OK: target\release\desktop-overlay.exe
pause
